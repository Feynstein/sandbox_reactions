//! The active box (m0_contrat.md §1.3.3, §1.3.2, §1.3.4; TW-R1 A2), on shaders/reduce.wgsl.
//!
//! The bounding box of non-vacuum cells (Σ ≥ Σ_vac), grown by 8 cells, rounded out to multiples of 8, clamped to the
//! world — empty when no cell is non-vacuum. `box_cells` takes 256 cells of the whole world per workgroup and halves
//! their (min x, min y, max x + 1, max y + 1) in shared memory; `box_fit`, one workgroup, walks the partials in index
//! order, halves the same way and writes the box: the record the passes bind as a uniform ([`BoxFit::active`]), their
//! indirect dispatch sizes ([`BoxFit::args`]) and the FFT size per axis — the smallest power of two ≥ 2 × the box, at
//! least 32 (§1.3.3). Integer min and max in a fixed-order tree (§1.3.4); nothing is read back to size a dispatch.
//!
//! When: before step 0 (ahead of the initial Δt reduction), after P9 at step indices ≡ 0 (mod [`REFIT_EVERY`]), and in
//! P0 when an edit lands outside the box — P0's edit pass sets [`BoxFit::edit_flag`]'s word (atomicOr 1); `box_gate`
//! turns it into the scan's and the fit's indirect sizes (zero when unset), and `box_fit` clears it.
//! [`BoxMode::WholeWorld`] holds the box at the world, for tests that compare the two (G-BOX).

use sr_physics::registry::Physics;

use super::{compute_pipeline, init_buffer, Dispatch, Size};
use crate::state::{read_buffers, State};

/// The box is re-fit after P9 at step indices ≡ 0 (mod `REFIT_EVERY`).
pub const REFIT_EVERY: u64 = 16;
/// Byte offsets in [`BoxFit::args`]: workgroups of 8 × 8 cells over the box, and of 256 of its cells (row-major).
pub const ARGS_TILES: u64 = 0;
pub const ARGS_CELLS: u64 = 12;

const WG: u32 = 256;
/// reduce.wgsl's `ActiveBox`: x0, y0, w, h, fft_w, fft_h, from_step, fits.
const RECORD_WORDS: usize = 8;

/// Whether the box is fit to the state or held at the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxMode {
    Fit,
    WholeWorld,
}

/// The box as read back (tests and headless runs only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoxReport {
    /// Origin and size in cells, multiples of 8; (0, 0, 0, 0) when empty.
    pub x0: u32,
    pub y0: u32,
    pub w: u32,
    pub h: u32,
    /// The FFT size per axis (§1.3.3).
    pub fft: [u32; 2],
    /// The first step that runs inside this box (from that step's P1 when P0 fit it).
    pub from_step: u32,
    /// Fits written since the record was made.
    pub fits: u32,
    /// The indirect sizes the passes read: 8 × 8 tiles, then 256-cell workgroups.
    pub tiles: [u32; 3],
    pub cells: [u32; 3],
}

/// The box's buffers and dispatches, built once for a state.
pub struct BoxFit {
    record: wgpu::Buffer,
    args: wgpu::Buffer,
    gate: wgpu::Buffer,
    flags: wgpu::Buffer,
    partials: wgpu::Buffer,
    params: wgpu::Buffer,
    /// Before step 0 and after P9 every [`REFIT_EVERY`] steps: `box_cells`, `box_fit`.
    pub(super) refit: Vec<Dispatch>,
    /// P0, after the edits: `box_gate`, then `box_cells` and `box_fit` sized by the gate.
    pub(super) on_edit: Vec<Dispatch>,
}

impl BoxFit {
    /// The buffers for `state`: an empty box until the first fit. [`BoxFit::build`] adds the dispatches.
    pub fn new(device: &wgpu::Device, state: &State, physics: &Physics, mode: BoxMode) -> BoxFit {
        let sigma_vac = physics.get("sigma_vac").expect("physics.json key sigma_vac") as f32;
        let storage = wgpu::BufferUsages::STORAGE;
        let partials = (state.world().cells() as u32).div_ceil(WG);
        BoxFit {
            record: init_buffer(device, "box.record", storage | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC, &[0; RECORD_WORDS]),
            args: init_buffer(device, "box.args", storage | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC, &[0, 0, 1, 0, 1, 1]),
            gate: init_buffer(device, "box.gate", storage | wgpu::BufferUsages::INDIRECT, &[0, 1, 1, 0, 1, 1]),
            flags: init_buffer(device, "box.flags", storage | wgpu::BufferUsages::COPY_DST, &[0]),
            partials: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("box.partials"),
                size: u64::from(partials) * 16,
                usage: storage,
                mapped_at_creation: false,
            }),
            params: init_buffer(
                device,
                "box.params",
                wgpu::BufferUsages::UNIFORM,
                &[sigma_vac.to_bits(), u32::from(mode == BoxMode::WholeWorld), 0, 0],
            ),
            refit: Vec::new(),
            on_edit: Vec::new(),
        }
    }

    /// The dispatches, on `module` (reduce.wgsl with eos.wgsl prepended); `dt_record` gives `box_fit` the step count.
    pub(super) fn build(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule, state: &State, dt_record: &wgpu::Buffer) {
        let entry = |binding: u32| -> wgpu::BindingResource<'_> {
            match binding {
                0 => state.dims().as_entire_binding(),
                1 => state.group(0).as_entire_binding(),
                5 => dt_record.as_entire_binding(),
                8 => self.record.as_entire_binding(),
                9 => self.args.as_entire_binding(),
                10 => self.gate.as_entire_binding(),
                11 => self.flags.as_entire_binding(),
                12 => self.partials.as_entire_binding(),
                _ => self.params.as_entire_binding(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size| {
            let pipeline = compute_pipeline(device, module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: entry(b) }).collect();
            let bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            Dispatch { label, pipeline, bind_groups, size }
        };
        let scan = (state.world().cells() as u32).div_ceil(WG);
        let (cells, fit) = (&[0, 1, 12, 13][..], &[0, 5, 8, 9, 11, 12, 13][..]);
        self.refit = vec![dispatch("box_cells", cells, Size::Fixed([scan, 1, 1])), dispatch("box_fit", fit, Size::Fixed([1, 1, 1]))];
        self.on_edit = vec![
            dispatch("box_gate", &[0, 10, 11], Size::Fixed([1, 1, 1])),
            dispatch("box_cells", cells, Size::Indirect(self.gate.clone(), 0)),
            dispatch("box_fit", fit, Size::Indirect(self.gate.clone(), 12)),
        ];
    }

    /// The box record, `ActiveBox` in the shaders: every box-sized pass binds it as a uniform.
    pub fn active(&self) -> &wgpu::Buffer {
        &self.record
    }

    /// The indirect dispatch sizes, at [`ARGS_TILES`] and [`ARGS_CELLS`].
    pub fn args(&self) -> &wgpu::Buffer {
        &self.args
    }

    /// One u32: P0's edit pass ORs 1 into it when an edit lands outside the box; the re-fit clears it.
    pub fn edit_flag(&self) -> &wgpu::Buffer {
        &self.flags
    }

    /// Reads the box back. Blocks on the device (native and headless only, §6.3) — never used to size a dispatch.
    pub fn read(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> BoxReport {
        let w: Vec<u32> = read_buffers(device, queue, "box.readback", [&self.record, &self.args].into_iter())
            .iter()
            .map(|v| v.to_bits())
            .collect();
        BoxReport {
            x0: w[0],
            y0: w[1],
            w: w[2],
            h: w[3],
            fft: [w[4], w[5]],
            from_step: w[6],
            fits: w[7],
            tiles: [w[8], w[9], w[10]],
            cells: [w[11], w[12], w[13]],
        }
    }
}
