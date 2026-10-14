//! P5 heat (m0_contrat.md §1.3.2, §2.4, §2.9's radiated, §1.9.1; M0-T42), on shaders/heat.wgsl: M0-T41's twin
//! (sr_physics::reference::heat) in f32 — face conductances frozen at the pass's start, RKL2's stages as dispatches, the
//! Levermore–Pomraning limiter, the light leaving booked `radiated` at the emitting cell, F kept for step n + 1's P4.
//!
//! Every dispatch but two runs over the box's 8 × 8 tiles ([`Size::Indirect`] at [`ARGS_TILES`]); `heat_begin` and
//! `heat_plan` are one invocation each. The stages are [`MAX_STAGES`] recorded dispatches, each told its index by a
//! uniform of its own, and a stage past the s P5 picked returns at once: s is the GPU's, never read back to size a
//! dispatch, and the latch reaches every one of them through the box's sizes and its own fixed-size slot. A cell that
//! would alone need more than 32 stages over Δt_n has its faces' conductance capped so it fits 32 (§2.4.3 [M0-T42], the
//! lead's ruling, 2026-10-14) and the record keeps the s Δt needed ([`HeatReport::warning`]) — the shader's header.
//!
//! The pass's scratch — the frozen planes, the faces' K and F, the stages' Y and R — lives as long as its bind groups;
//! the record ([`Step::read_heat`]) and the face fluxes ([`Step::read_heat_flux`]) are read back by tests and headless.

use sr_physics::reference::heat::MAX_STAGES;
use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size, Step};
use crate::scene::TestSettings;
use crate::state::{read_buffers, Booking, SideFields, State, WorldConfig, BOOK_GROUPS, TERMS};

/// heat.wgsl's frozen planes (T0 … OUT0), and Y's and R's three slots.
const FROZEN_PLANES: u64 = 7;
const STAGE_PLANES: u64 = 6;
/// heat.wgsl's `HeatRecord`.
const RECORD_WORDS: usize = 8;
/// heat.wgsl's flag bits.
const CONSTANT_DIFFUSIVITY: u32 = 1;
const FLUX_LIMITER: u32 = 2;
const PIN_TEMPERATURE: u32 = 4;

const _: () = {
    let max = WorldConfig::MAX as u64;
    assert!(FROZEN_PLANES * max * max * 4 <= 128 << 20);
    assert!(4 * (max + 1) * max * 4 <= 128 << 20);
};

/// What P5 needs beyond the state: the side fields F is kept in, and the scene's test flags (§2.12.1).
#[derive(Clone, Copy)]
pub struct HeatSetup<'a> {
    pub side: &'a SideFields,
    pub test: TestSettings,
}

/// P5's dispatches, its record and its faces.
pub struct Heat {
    pub(super) dispatches: Vec<Dispatch>,
    record: wgpu::Buffer,
    faces: wgpu::Buffer,
    /// `side.flux`, F's two planes: P5 writes it, the next step's P4 reads it, and a dump carries it (§0.5 [M0-D33]).
    flux: wgpu::Buffer,
    world: WorldConfig,
}

/// P5's record after the last step run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatReport {
    /// RKL2's stages the step ran: 0 when nothing could flow or the temperatures are pinned, at most [`MAX_STAGES`].
    pub stages: u32,
    /// The stages Δt_n needed before any cap (§2.4.3), and the most any step has needed since the step was built; over
    /// [`MAX_STAGES`], the stiff cells' faces were capped.
    pub need: u32,
    pub worst: u32,
    /// The time the stages covered: Δt_n, or less when it needed more than 32 stages.
    pub dt: f32,
    /// Δt_p of the start-of-pass coefficients, after any cap; none when nothing could flow.
    pub dt_p: Option<f32>,
}

impl HeatReport {
    /// `SR-WARN heat stages <s>` once any step has needed more than [`MAX_STAGES`] (§2.4.3).
    pub fn warning(&self) -> Option<String> {
        (self.worst as usize > MAX_STAGES).then(|| format!("SR-WARN heat stages {}", self.worst))
    }
}

impl Heat {
    /// P5 for `state`'s world over `boxfit`'s box, reading Δt_n from `dt_record`, keeping F in `setup.side` and booking
    /// `radiated` into `booking`, with the constants of `physics`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        boxfit: &BoxFit,
        dt_record: &wgpu::Buffer,
        setup: &HeatSetup<'_>,
    ) -> Heat {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("heat.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/heat.wgsl")).into()),
        });
        let world = state.world();
        let (w, h, cells) = (u64::from(world.width), u64::from(world.height), world.cells() as u64);
        let scratch = |label: &str, words: u64, copy: bool| {
            let copy = if copy { wgpu::BufferUsages::COPY_SRC } else { wgpu::BufferUsages::empty() };
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 4 * words,
                usage: wgpu::BufferUsages::STORAGE | copy,
                mapped_at_creation: false,
            })
        };
        let frozen = scratch("heat.frozen", FROZEN_PLANES * cells, false);
        let faces = scratch("heat.faces", 2 * ((w + 1) * h + w * (h + 1)), true);
        let yr = scratch("heat.stages", STAGE_PLANES * cells, false);
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        let record = init_buffer(device, "heat.record", storage, &[0; RECORD_WORDS]);
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let test = setup.test;
        let flags = u32::from(test.constant_diffusivity.is_some()) * CONSTANT_DIFFUSIVITY
            + u32::from(test.flux_limiter) * FLUX_LIMITER
            + u32::from(test.pin_temperature) * PIN_TEMPERATURE;
        let chi = test.constant_diffusivity.unwrap_or(0.0) as f32;
        let constants = [get("a_r"), get("c_sb"), get("kappa0"), get("k_cond"), get("kappa_dust"), get("t_dust"), chi];
        let mut words: Vec<u32> = constants.iter().map(|v| v.to_bits()).collect();
        words.extend([flags, get("t_floor").to_bits(), get("sigma_floor").to_bits(), 0, 0]);
        let params = init_buffer(device, "heat.params", wgpu::BufferUsages::UNIFORM, &words);
        let book = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == Pass::Heat).expect("a booking buffer written by P5");
        let stages: Vec<wgpu::Buffer> =
            (1..=MAX_STAGES as u32).map(|j| init_buffer(device, "heat.stage", wgpu::BufferUsages::UNIFORM, &[j, 0, 0, 0])).collect();

        let resource = |binding: u32| -> &wgpu::Buffer {
            match binding {
                0 => state.dims(),
                1 => state.group(0),
                2 => state.group(1),
                3 => state.group(2),
                4 => &frozen,
                5 => &faces,
                6 => &yr,
                7 => &record,
                8 => boxfit.active(),
                9 => dt_record,
                10 => &params,
                11 => setup.side.flux(),
                _ => booking.group(book),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], stage: Option<&wgpu::Buffer>, size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let mut entries: Vec<wgpu::BindGroupEntry> = bindings
                .iter()
                .map(|&b| wgpu::BindGroupEntry { binding: b, resource: resource(b).as_entire_binding() })
                .collect();
            if let Some(stage) = stage {
                entries.push(wgpu::BindGroupEntry { binding: 13, resource: stage.as_entire_binding() });
            }
            let mut bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            // Only `heat_cells` reads eos.wgsl (the floors ride in P5's own params).
            if label == "heat_cells" {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size }
        };
        let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);
        let one = || Size::Fixed([1, 1, 1]);
        let mut list = vec![
            dispatch("heat_begin", &[7], None, one()),
            dispatch("heat_cells", &[0, 1, 2, 3, 4, 8, 10], None, tiles()),
            dispatch("heat_stiff", &[0, 4, 7, 8, 10], None, tiles()),
            dispatch("heat_faces", &[0, 4, 5, 8, 9, 10], None, tiles()),
            dispatch("heat_rate", &[0, 4, 5, 6, 7, 8, 10, 11], None, tiles()),
            dispatch("heat_plan", &[7, 9, 10], None, one()),
        ];
        list.extend(stages.iter().map(|s| dispatch("heat_stage", &[0, 4, 5, 6, 7, 8, 10], Some(s), tiles())));
        list.push(dispatch("heat_commit", &[0, 1, 4, 6, 7, 8, 12], None, tiles()));
        Heat { dispatches: list, record, faces, flux: setup.side.flux().clone(), world }
    }
}

impl Step {
    /// P5's record after every submitted step; none when the step runs no P5. Blocks on the device (tests and headless).
    pub fn read_heat(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<HeatReport> {
        let heat = self.heat.as_ref()?;
        let w: Vec<u32> = read_buffers(device, queue, "heat.record", std::iter::once(&heat.record)).iter().map(|v| v.to_bits()).collect();
        let dt_p = f32::from_bits(w[6]);
        Some(HeatReport { stages: w[2], need: w[3], worst: w[4], dt: f32::from_bits(w[5]), dt_p: (dt_p > 0.0).then_some(dt_p) })
    }

    /// F at every face from the last P5's start-of-pass temperatures, in the twin's `Flux` layout: the x faces
    /// ((width + 1) × height), then the y faces (width × (height + 1)); inside the box only. Blocks on the device.
    pub fn read_heat_flux(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<(Vec<f32>, Vec<f32>)> {
        let heat = self.heat.as_ref()?;
        let (w, h) = (heat.world.width as usize, heat.world.height as usize);
        let all = read_buffers(device, queue, "heat.faces", std::iter::once(&heat.faces));
        let (nx, ny) = ((w + 1) * h, w * (h + 1));
        Some((all[nx + ny..2 * nx + ny].to_vec(), all[2 * nx + ny..].to_vec()))
    }

    /// `side.flux` (F_x, then F_y, row-major) when the step runs P5: the F its next step's P4 pushes with, which a dump
    /// carries and `load_dump` restores (§0.5 [M0-D33]); none without P5.
    pub fn side_flux(&self) -> Option<&wgpu::Buffer> {
        self.heat.as_ref().map(|heat| &heat.flux)
    }
}
