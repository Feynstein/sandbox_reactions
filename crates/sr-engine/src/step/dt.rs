//! P1 Δt and P9's Δt reduction and non-finite guard (m0_contrat.md §1.3.2, §1.3.4, §2.3.5, §3.3), on
//! shaders/reduce.wgsl.
//!
//! P9 reduces Δt_{n+1} = min(C · min_cells [(|u| + c)/Δx + (|v| + c)/Δy]⁻¹, η_g · min_cells √(Δx/|g|)) by a fixed-order
//! tree (`dt_cells`, then `dt_partials`); P1 (`dt_advance`) makes that value step n + 1's Δt, so step n runs on the Δt
//! P9 made at the end of step n − 1, and step 0 on the initial state's — the same reduction, recorded once before step 0
//! ([`super::Step::encode`]).
//! Every 64 steps P9 also runs the non-finite guard (`guard_scan`, `guard_stamp`): the first NaN or Inf found, by step,
//! cell and channel, stays in the record until the run reads it ([`Dt::read`]) and stops (exit 5, §3.3).
//!
//! Later passes read Δt_n from [`Dt::record`]'s first word (an f32). The reduction runs over the active box
//! ([`super::boxfit`]): `dt_cells` takes its workgroup count from the box's indirect sizes and `dt_partials` the box's
//! partials; the guard scans the whole world.

use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_CELLS};
use super::{compute_pipeline, init_buffer, Dispatch, EosGpu, Size};
use crate::state::{read_buffers, State, CHANNELS};

/// The guard runs at step indices ≡ 0 (mod `GUARD_EVERY`).
pub const GUARD_EVERY: u64 = 64;

/// reduce.wgsl's `DtState`: dt, dt_next, step, steps, found, bad_step, bad_code, flagged.
const RECORD_WORDS: usize = 8;
const WG: u32 = 256;

/// The first non-finite value the guard found: the step it was found at, the cell and the channel (§2.2's name).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonFinite {
    pub step: u64,
    pub x: u32,
    pub y: u32,
    pub channel: &'static str,
}

/// The Δt record as read back after a submission.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DtReport {
    /// Δt_n of the last step run.
    pub dt: f32,
    /// Δt_{n+1}, the next step's.
    pub dt_next: f32,
    /// Steps run since the record was made.
    pub steps: u64,
    pub non_finite: Option<NonFinite>,
}

/// The Δt record and the dispatches that write it, built once for a state.
pub struct Dt {
    record: wgpu::Buffer,
    world_width: u32,
    /// P1: `dt_advance`.
    pub(super) advance: Vec<Dispatch>,
    /// P9: `dt_cells`, `dt_partials`.
    pub(super) reduce: Vec<Dispatch>,
    /// P9, every [`GUARD_EVERY`] steps: `guard_scan`, `guard_stamp`.
    pub(super) guard: Vec<Dispatch>,
}

impl Dt {
    /// The record (zeroed) and the dispatches for `state` on `module` (reduce.wgsl with eos.wgsl prepended), with
    /// C = `cfl` and η_g = `eta_g` from `physics`, the reduction over `boxfit`'s box.
    pub fn new(
        device: &wgpu::Device,
        module: &wgpu::ShaderModule,
        state: &State,
        eos: &EosGpu,
        physics: &Physics,
        boxfit: &BoxFit,
    ) -> Dt {
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let world = state.world();
        let partials = (world.cells() as u32).div_ceil(WG);
        let params = init_buffer(
            device,
            "dt.params",
            wgpu::BufferUsages::UNIFORM,
            &[get("cfl").to_bits(), get("eta_g").to_bits(), partials, 0],
        );
        let partials_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dt.partials"),
            size: u64::from(partials) * 8,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let record = init_buffer(
            device,
            "dt.record",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            &[0; RECORD_WORDS],
        );

        // Every entry point binds the subset of group 0 it uses (auto layouts); `entry` maps a binding to its buffer.
        let entry = |binding: u32| -> wgpu::BindingResource<'_> {
            match binding {
                0 => state.dims().as_entire_binding(),
                1 => state.group(0).as_entire_binding(),
                2 => state.group(1).as_entire_binding(),
                3 => state.group(2).as_entire_binding(),
                4 => partials_buf.as_entire_binding(),
                5 => record.as_entire_binding(),
                6 => params.as_entire_binding(),
                _ => boxfit.active().as_entire_binding(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size, uses_eos: bool| {
            let pipeline = compute_pipeline(device, module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: entry(b) }).collect();
            let mut bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            if uses_eos {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size }
        };
        let one = || Size::Fixed([1, 1, 1]);
        let tiles = Size::Fixed([world.width.div_ceil(8), world.height.div_ceil(8), 1]);
        Dt {
            advance: vec![dispatch("dt_advance", &[5], one(), false)],
            reduce: vec![
                dispatch("dt_cells", &[0, 1, 2, 3, 4, 7], Size::Indirect(boxfit.args().clone(), ARGS_CELLS), true),
                dispatch("dt_partials", &[4, 5, 6, 7], one(), false),
            ],
            guard: vec![dispatch("guard_scan", &[0, 1, 2, 3, 5], tiles, false), dispatch("guard_stamp", &[5], one(), false)],
            world_width: world.width,
            record,
        }
    }

    /// The record buffer: Δt_n (f32) is its first word.
    pub fn record(&self) -> &wgpu::Buffer {
        &self.record
    }

    /// Reads the record back, after every submitted step. Blocks on the device (native and headless only, §6.3).
    pub fn read(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> DtReport {
        let w: Vec<u32> =
            read_buffers(device, queue, "dt.readback", std::iter::once(&self.record)).iter().map(|v| v.to_bits()).collect();
        let non_finite = (w[7] != 0).then(|| {
            let (cell, channel) = (w[6] / 16, w[6] % 16);
            NonFinite {
                step: u64::from(w[5]),
                x: cell % self.world_width,
                y: cell / self.world_width,
                channel: CHANNELS[channel as usize],
            }
        });
        DtReport { dt: f32::from_bits(w[0]), dt_next: f32::from_bits(w[1]), steps: u64::from(w[3]), non_finite }
    }
}
