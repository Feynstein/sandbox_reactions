//! P5 heat (m0_contrat.md §1.3.2, §2.4, §0.5 [M0-TJ4], §2.9's radiated, §1.9.1; M0-T42, M0-T118, M0-T119), on
//! shaders/heat.wgsl: the twin's implicit step (sr_physics::reference::heat `Heat::step`) in f32 — one backward-Euler step in
//! u = a_r T⁴ (u = T under the scene's `test.constant_diffusivity`), each face's D_f frozen at the pass's start with the
//! Levermore–Pomraning limiter, Newton passes and PCG rounds (the multigrid V-cycle by default, Jacobi for tests), the light
//! leaving booked `radiated` at the emitting cell, F (the flux the step moved) kept for step n + 1's P4. RKL2 and its
//! 32-stage face cap are retired (§0.5 [M0-TJ4]).
//!
//! The dispatches: `heat_cells` (the start state), `heat_law` (each face's D_f), `heat_open` (one invocation), [`SLOTS`]
//! recorded `heat_round`s — Newton passes and PCG rounds, one dispatch a round on Jacobi, each sized by a word the one before
//! it wrote ([`Size::Chained`]), so a converged step's later rounds have no workgroups — then `heat_close` (F, side.flux,
//! the booking), `heat_sum` (one workgroup: the closure δ, every round's size zeroed) and `heat_apply` (E). The box-sized
//! ones run over its 8 × 8 tiles ([`Size::Indirect`] at [`ARGS_TILES`]), and the latch reaches the chain through
//! `heat_open`, its fixed-size opener.
//!
//! Speed gives first (§0.5 [M0-TJ4] (6), M0-T121): a step records only the first [`Step::heat_rounds`] of the slots — the
//! engine's choice, learnt from the slots recent steps used, or the caller's ([`Step::set_heat_rounds`]) — and a step whose
//! rounds have not ended after them is carried into the next submission by the latch's gate ([`super::latch`]); nothing it
//! computes depends on where a submission ends. A step that reaches the ceiling is logged for headless's warning
//! ([`Step::heat_warnings`]).
//!
//! The pass's scratch — the frozen planes, the faces' D_f and F, the solve's vectors — lives as long as its bind groups;
//! the record ([`Step::read_heat`]) and the face fluxes ([`Step::read_heat_flux`]) are read back by tests and headless.
//!
//! The multigrid preconditioner (§0.5 [M0-D82]; heat.wgsl's last section): the levels of a pass's system (`mg_level0`, one
//! `mg_coarsen` a level) and a symmetric V-cycle (`mg_smooth` and `mg_restrict` on each level of more than 256 cells,
//! `mg_coarse` for the rest, `mg_smooth` again going up) — P5's default since M0-D84, Jacobi kept for tests behind
//! [`Step::set_heat_precond`] and [`set_default_precond`].
//! In the round machine (M0-D83) a slot on the V-cycle is a group ([`Heat::slots`]): its `heat_round`, then the levels, the
//! V-cycle and `heat_matvec`, each [`Size::Chained`] on the words its `heat_round` writes after the slots' words of its own
//! args buffer — the carry, the learnt count and every bound read slots as before, a slot one round. [`Step::heat_first_system`]
//! and [`Step::apply_heat_precond`] read the V-cycle alone.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use sr_physics::reference::heat::{MAX_PASSES, MAX_ROUNDS};
use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, record_dispatches, with_eos, Dispatch, EosGpu, Pass, Size, Step};
use crate::scene::TestSettings;
use crate::state::{read_buffers, Booking, SideFields, State, WorldConfig, BOOK_GROUPS, TERMS};

/// heat.wgsl's frozen planes (T0 … KS).
const FROZEN_PLANES: u64 = 5;
/// heat.wgsl's `solve` (x, T, p, the diagonal, x's low word) and `krylov` (r, w, s in two parities) planes.
const SOLVE_PLANES: u64 = 5;
const KRYLOV_PLANES: u64 = 6;
/// heat.wgsl's `Machines`: two `Machine` records (Δt the ninth word of each), the `Report`, then the log of [`LOG`]
/// unconverged steps.
const MACHINE_WORDS: usize = 2 * 16 + REPORT_WORDS + 4 * LOG;
const REPORT_WORDS: usize = 12;
/// The `Report`'s `logged` word; the log follows the report.
const LOGGED_WORD: usize = 32 + 10;
/// The `Report`'s `most` word: the most slots a step ended since the submission began.
pub(super) const MOST_BYTE: u64 = 4 * (32 + 11);
/// The unconverged steps the log keeps between two reads (heat.wgsl `LOG`): headless reads it after each run of at most
/// as many steps.
pub const LOG: usize = 256;
/// The implicit step's slots (§0.5 [M0-TJ4] (6)): every round of the ceiling, three slots a pass (its residual, its first
/// direction, the update that ends its solve), the first pass's build and the last end test — all a step can use.
pub const SLOTS: usize = MAX_ROUNDS + 3 * MAX_PASSES + 2;
/// P5's dispatches before its slots (`heat_cells`, `heat_law`, `heat_open`).
pub(super) const HEAD: usize = 3;
/// The fewest slots a step records (8 is the fewest a submission carries, G-STIFF (d)); [`FIRST_ROUNDS`] is the engine's
/// first choice before any step was read back, past the 84 the Sun-like disk's hardest early step uses (12 passes, 46
/// rounds; measured on the RTX 5090, M0-T121).
pub const MIN_ROUNDS: usize = 8;
const FIRST_ROUNDS: usize = 128;
/// heat.wgsl's flag bits.
const CONSTANT_DIFFUSIVITY: u32 = 1;
const FLUX_LIMITER: u32 = 2;
const PIN_TEMPERATURE: u32 = 4;
/// heat.wgsl's `MG_LEVELS` (the world's levels fit: 10 at 2048 × 2048), `MG_LAST` and `MG_COARSE`.
const MG_LEVELS: usize = 12;
const MG_LAST: u64 = 4;
const MG_COARSE: u64 = 256;
/// heat.wgsl's MGA_* entries (3 words each) after the slots' [`ROUND_WORDS`] in each parity's args buffer.
const MGA_LEVEL0: usize = 0;
const MGA_COARSEN: usize = 1;
const MGA_SMOOTH: usize = 1 + MG_LEVELS;
const MGA_RESTRICT: usize = 1 + 2 * MG_LEVELS;
const MGA_COARSE: usize = 1 + 3 * MG_LEVELS;
const MGA_MATVEC: usize = 2 + 3 * MG_LEVELS;
const MGA_COUNT: usize = 3 + 3 * MG_LEVELS;
const ROUND_WORDS: usize = 3 * SLOTS.div_ceil(2);

const _: () = {
    let max = WorldConfig::MAX as u64;
    assert!(FROZEN_PLANES * max * max * 4 <= 128 << 20);
    assert!(KRYLOV_PLANES * max * max * 4 <= 128 << 20);
    assert!(4 * (max + 1) * max * 4 <= 128 << 20);
    let (_, [coef, vec]) = mg_layout(max, max);
    assert!(coef * 4 <= 128 << 20 && vec * 4 <= 128 << 20 && mg_levels(max, max).0 < MG_LEVELS);
};

/// Level l's sides for level 0's `w` × `h` (heat.wgsl `mg_dims`).
const fn mg_dims(w: u64, h: u64, l: usize) -> (u64, u64) {
    (w.div_ceil(1 << l), h.div_ceil(1 << l))
}

/// (the coarsest level, the first of at most [`MG_COARSE`] cells) for level 0's `w` × `h` (heat.wgsl `mg_span`).
const fn mg_levels(w: u64, h: u64) -> (usize, usize) {
    let mut last = 0;
    while last + 1 < MG_LEVELS && (mg_dims(w, h, last).0 > MG_LAST || mg_dims(w, h, last).1 > MG_LAST) {
        last += 1;
    }
    let mut first = 0;
    while mg_dims(w, h, first).0 * mg_dims(w, h, first).1 > MG_COARSE {
        first += 1;
    }
    (last, first)
}

/// heat.wgsl's `MgLayout` for a `w` × `h` world — [coef, vec, w, h] a level: per level, own and the two face planes in
/// `mg_coef`, b and z down and up in `mg_vec` — and the two buffers' words.
const fn mg_layout(w: u64, h: u64) -> ([[u32; 4]; MG_LEVELS], [u64; 2]) {
    let (mut layout, mut coef, mut vec, mut l) = ([[0; 4]; MG_LEVELS], 0, 0, 0);
    while l < MG_LEVELS {
        let (lw, lh) = mg_dims(w, h, l);
        layout[l] = [coef as u32, vec as u32, lw as u32, lh as u32];
        coef += lw * lh + (lw + 1) * lh + lw * (lh + 1);
        vec += 3 * lw * lh;
        l += 1;
    }
    (layout, [coef, vec])
}

/// P5's preconditioner (§0.5 [M0-D82]): Jacobi's D⁻¹ or the multigrid V-cycle (the default, §0.5 [M0-D84]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeatPrecond {
    Jacobi,
    #[default]
    Multigrid,
}

/// Whether each P5 built starts on the V-cycle ([`set_default_precond`]).
static DEFAULT_MULTIGRID: AtomicBool = AtomicBool::new(true);

/// Tests only: the preconditioner every P5 built from now on starts with — a case's parameter that reaches the steps it
/// builds, `headless` in-process included (the GPU tests run one at a time); the V-cycle until set. Returns the one before.
pub fn set_default_precond(precond: HeatPrecond) -> HeatPrecond {
    match DEFAULT_MULTIGRID.swap(precond == HeatPrecond::Multigrid, Ordering::Relaxed) {
        true => HeatPrecond::Multigrid,
        false => HeatPrecond::Jacobi,
    }
}

/// Level 0 of the multigrid hierarchy over the box at (x0, y0), `w` × `h` (cell (i, j) at j·w + i): each gas cell's own
/// term c_i + Σ_vac D_f (−1 off the gas) and the D_f of its vacuum faces alone, the D_f of the x faces ((w + 1) × h) and the
/// y faces (w × (h + 1)) between two gas cells; and the levels' count and the first that `mg_coarse` runs.
#[derive(Clone, Debug)]
pub struct HeatSystem {
    pub x0: usize,
    pub y0: usize,
    pub w: usize,
    pub h: usize,
    pub own: Vec<f32>,
    pub vac: Vec<f32>,
    pub fx: Vec<f32>,
    pub fy: Vec<f32>,
    pub levels: usize,
    pub coarse: usize,
}

/// What P5 needs beyond the state: the side fields F is kept in, and the scene's test flags (§2.12.1).
#[derive(Clone, Copy)]
pub struct HeatSetup<'a> {
    pub side: &'a SideFields,
    pub test: TestSettings,
}

/// P5's dispatches, its record and its faces.
pub struct Heat {
    pub(super) dispatches: Vec<Dispatch>,
    /// `heat_park`, recorded bare after the rounds a step records (the latch's gate follows it).
    pub(super) park: Dispatch,
    /// The implicit step's `Machines`: its records and its report.
    machine: wgpu::Buffer,
    faces: wgpu::Buffer,
    /// `side.flux`, F's two planes: P5 writes it, the next step's P4 reads it, and a dump carries it (§0.5 [M0-D33]).
    flux: wgpu::Buffer,
    /// The slots' sizes, parity 0 then 1 (latch.wgsl's gate binds them).
    sizes: [wgpu::Buffer; 2],
    world: WorldConfig,
    /// The slots a step records: the caller's when set, else the engine's.
    fixed: Cell<Option<usize>>,
    learnt: Cell<usize>,
    /// The multigrid's planes, its level layout ([`mg_layout`]), the dispatches that build its levels at T⁰ (the first
    /// pass's system) and those of one V-cycle (the tests' window), and the preconditioner chosen.
    mg_coef: wgpu::Buffer,
    mg_vec: wgpu::Buffer,
    mg_layout: [[u32; 4]; MG_LEVELS],
    mg_build: Vec<Dispatch>,
    mg_cycle: Vec<Dispatch>,
    precond: Cell<HeatPrecond>,
    /// The round machine on the V-cycle: the slots' `heat_round`s, and the rest of a slot's group per parity.
    mg_rounds: Vec<Dispatch>,
    mg_groups: [Vec<Dispatch>; 2],
}

/// P5's record after the last step run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatReport {
    /// The time the step covered: Δt_n; 0 when it did not run (pinned temperatures, Δt ≤ 0, an empty box).
    pub dt: f32,
    /// The step's Newton passes and PCG rounds over all of them (0 when it did not run).
    pub passes: u32,
    pub rounds: u32,
    /// False when the step reached its ceiling (§0.5 [M0-TJ4] (6)) and committed its last iterate.
    pub converged: bool,
    /// The heat balance's closure δ (§0.5 [M0-TJ4] (5)).
    pub delta: f32,
}

impl HeatReport {
    /// `SR-WARN heat unconverged step <n> passes <k> rounds <r>` when the step hit its ceiling (§0.5 [M0-TJ4] (6)); `step`
    /// is the step's index.
    pub fn warning(&self, step: u64) -> Option<String> {
        (!self.converged).then(|| format!("SR-WARN heat unconverged step {step} passes {} rounds {}", self.passes, self.rounds))
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
        let test = setup.test;
        let scratch = |label: &str, words: u64, usage: wgpu::BufferUsages| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 4 * words.max(1),
                usage: wgpu::BufferUsages::STORAGE | usage,
                mapped_at_creation: false,
            })
        };
        let none = wgpu::BufferUsages::empty();
        let frozen = scratch("heat.frozen", FROZEN_PLANES * cells, none);
        let faces = scratch("heat.faces", 2 * ((w + 1) * h + w * (h + 1)), wgpu::BufferUsages::COPY_SRC);
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        // The solve's planes, the machine, the partial sums (two parities of 16 × 16 tiles, then the commit's 8 × 8 tiles)
        // and the rounds' sizes, a buffer per parity (zero: no round runs until `heat_open`).
        let tiles = w.div_ceil(16) * h.div_ceil(16);
        let solve = scratch("heat.solve", SOLVE_PLANES * cells, none);
        let krylov = scratch("heat.krylov", KRYLOV_PLANES * cells, none);
        let machine = init_buffer(device, "heat.machine", storage | wgpu::BufferUsages::COPY_DST, &[0; MACHINE_WORDS]);
        let partials = scratch("heat.partials", 4 * (2 * tiles + w.div_ceil(8) * h.div_ceil(8)), none);
        let sizes = [0, 1].map(|_| init_buffer(device, "heat.rounds", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT, &[0; ROUND_WORDS + 3 * MGA_COUNT]));
        // Per parity on Jacobi, then on the V-cycle (`mg`).
        let parity = [0, 1, 2, 3].map(|k| init_buffer(device, "heat.parity", wgpu::BufferUsages::UNIFORM, &[k % 2, k / 2, 0, 0]));
        let (mg_layout, [coef_words, vec_words]) = mg_layout(w, h);
        let mg_coef = scratch("heat.mg_coef", coef_words, wgpu::BufferUsages::COPY_SRC);
        let mg_vec = scratch("heat.mg_vec", vec_words, wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST);
        let layout = init_buffer(device, "heat.mg_layout", wgpu::BufferUsages::UNIFORM, mg_layout.as_flattened());
        // `mg_step` per (level, up), at the iterate; the tests' window builds level 0 at T⁰ (`start`: the first pass's system).
        let steps: Vec<wgpu::Buffer> =
            (0..2 * MG_LEVELS as u32).map(|k| init_buffer(device, "heat.mg_step", wgpu::BufferUsages::UNIFORM, &[k / 2, k % 2, 0, 0])).collect();
        let at_t0 = init_buffer(device, "heat.mg_step", wgpu::BufferUsages::UNIFORM, &[0, 0, 1, 0]);

        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let flags = u32::from(test.constant_diffusivity.is_some()) * CONSTANT_DIFFUSIVITY
            + u32::from(test.flux_limiter) * FLUX_LIMITER
            + u32::from(test.pin_temperature) * PIN_TEMPERATURE;
        let chi = test.constant_diffusivity.unwrap_or(0.0) as f32;
        let constants = [get("a_r"), get("c_sb"), get("kappa0"), get("k_cond"), get("kappa_dust"), get("t_dust"), chi];
        let mut words: Vec<u32> = constants.iter().map(|v| v.to_bits()).collect();
        words.extend([flags, get("t_floor").to_bits(), get("sigma_floor").to_bits(), tiles as u32, 0]);
        let params = init_buffer(device, "heat.params", wgpu::BufferUsages::UNIFORM, &words);
        let book = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == Pass::Heat).expect("a booking buffer written by P5");

        let resource = |binding: u32| -> &wgpu::Buffer {
            match binding {
                0 => state.dims(),
                1 => state.group(0),
                2 => state.group(1),
                3 => state.group(2),
                4 => &frozen,
                5 => &faces,
                8 => boxfit.active(),
                9 => dt_record,
                10 => &params,
                11 => setup.side.flux(),
                12 => booking.group(book),
                14 => &solve,
                15 => &krylov,
                16 => &machine,
                17 => &partials,
                18 => &sizes[0],
                19 => &sizes[1],
                21 => &mg_coef,
                22 => &mg_vec,
                23 => &layout,
                _ => unreachable!("heat.wgsl binding {binding}"),
            }
        };
        // A dispatch's bind group: `bindings` from `resource`, then `extra` (binding, buffer) pairs in their place.
        let bind = |pipeline: &wgpu::ComputePipeline, label: &'static str, bindings: &[u32], extra: &[(u32, &wgpu::Buffer)]| {
            let mut entries: Vec<wgpu::BindGroupEntry> = bindings
                .iter()
                .map(|&b| wgpu::BindGroupEntry { binding: b, resource: resource(b).as_entire_binding() })
                .collect();
            entries.extend(extra.iter().map(|&(b, buffer)| wgpu::BindGroupEntry { binding: b, resource: buffer.as_entire_binding() }));
            device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some(label), layout: &pipeline.get_bind_group_layout(0), entries: &entries })
        };
        let dispatch = |label: &'static str, bindings: &[u32], extra: &[(u32, &wgpu::Buffer)], size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let mut bind_groups = vec![bind(&pipeline, label, bindings, extra)];
            // Only `heat_cells` reads eos.wgsl (the floors ride in P5's own params).
            if label == "heat_cells" {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size }
        };
        let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);
        let one = || Size::Fixed([1, 1, 1]);
        // A dispatch of a pipeline built once: its label, bindings and size.
        let on = |&(label, ref pipeline): &(&'static str, wgpu::ComputePipeline), bindings: &[u32], extra: &[(u32, &wgpu::Buffer)], size: Size| Dispatch {
            label,
            pipeline: pipeline.clone(),
            bind_groups: vec![bind(pipeline, label, bindings, extra)],
            size,
        };
        let pipeline = |label: &'static str| (label, compute_pipeline(device, &module, label));
        // Slot j writes parity j % 2 and sizes slot j + 1 in the other parity's buffer, and its own group's multigrid there;
        // it is sized by its own. On Jacobi or on the V-cycle (`mg`, its parity record's second word).
        let round = pipeline("heat_round");
        let slot = |j: usize, mg: usize| on(&round, &[0, 4, 5, 8, 10, 14, 15, 16, 17, 22, 23], &[(18, &sizes[1 - j % 2]), (20, &parity[2 * mg + j % 2])], Size::Chained(sizes[j % 2].clone(), (j / 2 * 12) as u64));
        let mut list = vec![
            dispatch("heat_cells", &[0, 1, 2, 3, 4, 8, 10], &[], tiles()),
            dispatch("heat_law", &[0, 4, 5, 8, 10], &[], tiles()),
            dispatch("heat_open", &[8, 9, 10, 16, 18], &[], one()),
        ];
        let rounds = [0, 1].map(|j| slot(j, 0));
        let mg_round = [0, 1].map(|j| slot(j, 1));
        let again = |r: &Dispatch, j: usize| Dispatch {
            label: r.label,
            pipeline: r.pipeline.clone(),
            bind_groups: r.bind_groups.clone(),
            size: Size::Chained(sizes[j % 2].clone(), (j / 2 * 12) as u64),
        };
        list.extend((0..SLOTS).map(|j| again(&rounds[j % 2], j)));
        let mg_rounds = (0..SLOTS).map(|j| again(&mg_round[j % 2], j)).collect();
        list.push(dispatch("heat_close", &[0, 4, 5, 8, 10, 11, 12, 14, 16, 17], &[], tiles()));
        list.push(dispatch("heat_sum", &[8, 10, 16, 17, 18, 19], &[], one()));
        list.push(dispatch("heat_apply", &[0, 1, 4, 8, 14, 16], &[], tiles()));
        let park = dispatch("heat_park", &[16, 18, 19], &[], one());

        // The multigrid: each entry's dispatch (heat.wgsl MGA_*) over the world's levels — the box's are no more —, sized
        // by `size`: the tests' window over the world's level (the shader leaves the rest idle), a slot group by its round.
        let (level0, coarsen, smooth, restrict, coarse) =
            (pipeline("mg_level0"), pipeline("mg_coarsen"), pipeline("mg_smooth"), pipeline("mg_restrict"), pipeline("mg_coarse"));
        let (last, first) = mg_levels(w, h);
        let mg = [8, 21, 22, 23];
        let build = |size: &dyn Fn(usize) -> Size, start: &wgpu::Buffer| -> Vec<Dispatch> {
            let mut list = vec![on(&level0, &[0, 4, 5, 8, 10, 14, 16, 21, 23], &[(24, start)], size(MGA_LEVEL0))];
            list.extend((0..last).map(|l| on(&coarsen, &[8, 21, 23], &[(24, &steps[2 * l])], size(MGA_COARSEN + l))));
            list
        };
        let cycle = |size: &dyn Fn(usize) -> Size| -> Vec<Dispatch> {
            let mut list = Vec::new();
            for l in 0..first {
                list.push(on(&smooth, &mg, &[(24, &steps[2 * l])], size(MGA_SMOOTH + l)));
                list.push(on(&restrict, &mg, &[(24, &steps[2 * l])], size(MGA_RESTRICT + l)));
            }
            list.push(on(&coarse, &mg, &[], size(MGA_COARSE)));
            list.extend((0..first).rev().map(|l| on(&smooth, &mg, &[(24, &steps[2 * l + 1])], size(MGA_SMOOTH + l))));
            list
        };
        let tiles_of = |l: usize, side: u64| {
            let (lw, lh) = mg_dims(w, h, l);
            Size::Fixed([lw.div_ceil(side) as u32, lh.div_ceil(side) as u32, 1])
        };
        let fixed = |k: usize| match k {
            MGA_LEVEL0 => tiles(),
            k if k < MGA_SMOOTH => tiles_of(k - MGA_COARSEN + 1, 8),
            k if k < MGA_RESTRICT => tiles_of(k - MGA_SMOOTH, 16),
            k if k < MGA_COARSE => tiles_of(k - MGA_RESTRICT + 1, 8),
            _ => one(),
        };
        let (mg_build, mg_cycle) = (build(&fixed, &at_t0), cycle(&fixed));
        let matvec = pipeline("heat_matvec");
        let mg_groups = [0, 1].map(|p| {
            let chained = |k: usize| Size::Chained(sizes[1 - p].clone(), (4 * (ROUND_WORDS + 3 * k)) as u64);
            let mut group = build(&chained, &steps[0]);
            group.extend(cycle(&chained));
            group.push(on(&matvec, &[0, 4, 5, 8, 10, 14, 15, 16, 17, 22, 23], &[(20, &parity[2 + p])], chained(MGA_MATVEC)));
            group
        });

        let flux = setup.side.flux().clone();
        let (fixed, learnt) = (Cell::new(None), Cell::new(FIRST_ROUNDS));
        let precond = Cell::new(if DEFAULT_MULTIGRID.load(Ordering::Relaxed) { HeatPrecond::Multigrid } else { HeatPrecond::Jacobi });
        Heat { dispatches: list, park, machine, faces, flux, sizes, world, fixed, learnt, mg_coef, mg_vec, mg_layout, mg_build, mg_cycle, precond, mg_rounds, mg_groups }
    }

    /// heat.wgsl's `Machines` and the slots' sizes of parity 0, for the latch's gate.
    pub(super) fn gate_buffers(&self) -> [&wgpu::Buffer; 2] {
        [&self.machine, &self.sizes[0]]
    }

    /// The slots each step of the next submission records: even (a carried step's next round is then parity 0's), at least
    /// [`MIN_ROUNDS`], at most [`SLOTS`].
    pub(super) fn rounds(&self) -> usize {
        self.fixed.get().unwrap_or(self.learnt.get())
    }

    /// The dispatches of the [`Heat::rounds`] slots a step records, in order: the first of `jacobi` (the step's own
    /// `heat_round`s), or on the V-cycle a group a slot — its `heat_round`, then its parity's levels, V-cycle and matvec.
    pub(super) fn slots<'a>(&'a self, jacobi: &'a [Dispatch]) -> Vec<&'a [Dispatch]> {
        let n = self.rounds();
        match self.precond.get() {
            HeatPrecond::Jacobi => vec![&jacobi[..n]],
            HeatPrecond::Multigrid => (0..n).flat_map(|j| [&self.mg_rounds[j..=j], &self.mg_groups[j % 2][..]]).collect(),
        }
    }

    /// The engine's choice after a submission read back whose steps used at most `most` slots: that and a quarter more,
    /// four more, rounded up to even — at once when that is more than today's, else falling by at most a quarter a read,
    /// so a run of easy steps does not make the next hard one carry at once (none to learn when no step ended).
    pub(super) fn learn(&self, most: u32) {
        if most > 0 {
            let used = most as usize;
            let was = self.learnt.get();
            self.learnt.set(even((used + used / 4 + 4).max(was - was / 4).clamp(MIN_ROUNDS, SLOTS)));
        }
    }
}

/// `n` rounded up to even.
fn even(n: usize) -> usize {
    n + n % 2
}

impl Step {
    /// P5's record after every submitted step; none when the step runs no P5. Blocks on the device (tests and headless).
    pub fn read_heat(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<HeatReport> {
        let heat = self.heat.as_ref()?;
        let w = machine_words(device, queue, heat);
        // The `Report` follows the two records; Δt is the second record's (`heat_open` writes it there).
        let r = &w[32..];
        let stepped = r[3] != 0;
        Some(HeatReport {
            dt: if stepped { f32::from_bits(w[16 + 8]) } else { 0.0 },
            passes: r[0],
            rounds: r[1],
            converged: r[2] != 0,
            delta: f32::from_bits(r[4]),
        })
    }

    /// `SR-WARN heat unconverged step <n> passes <k> rounds <r>` for every step that reached the ceiling since the last
    /// call (§0.5 [M0-TJ4] (6)), in step order — the first [`LOG`] of them, then the log is cleared; none without P5.
    /// Blocks on the device (headless).
    pub fn heat_warnings(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<String> {
        let Some(heat) = self.heat.as_ref() else { return Vec::new() };
        let w = machine_words(device, queue, heat);
        let logged = (w[LOGGED_WORD] as usize).min(LOG);
        queue.write_buffer(&heat.machine, 4 * LOGGED_WORD as u64, &0u32.to_le_bytes());
        let log = &w[32 + REPORT_WORDS..];
        (0..logged).map(|i| format!("SR-WARN heat unconverged step {} passes {} rounds {}", log[4 * i], log[4 * i + 1], log[4 * i + 2])).collect()
    }

    /// The slots each step records from the next submission on (§0.5 [M0-TJ4] (6)): `Some(n)` fixes them — n rounded up to
    /// even within [[`MIN_ROUNDS`], [`SLOTS`]] (8 is the fewest a submission carries, G-STIFF (d)) — and `None` leaves them to
    /// the engine. Nothing a step computes depends on them. No effect without P5.
    pub fn set_heat_rounds(&self, rounds: Option<usize>) {
        if let Some(heat) = &self.heat {
            heat.fixed.set(rounds.map(|n| even(n.clamp(MIN_ROUNDS, SLOTS))));
        }
    }

    /// The slots each step of the next submission records; none without P5.
    pub fn heat_rounds(&self) -> Option<usize> {
        self.heat.as_ref().map(Heat::rounds)
    }

    /// F at every face from the last P5, in the twin's `Flux` layout: the x faces ((width + 1) × height), then the y
    /// faces (width × (height + 1)); inside the box only: the flux the step moved (the start state's law when pinned). Blocks on the device.
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

impl Step {
    /// The preconditioner of P5's PCG from the next submission on, and the one [`Step::apply_heat_precond`] applies (§0.5
    /// [M0-D82]; tests only, like [`Step::set_heat_rounds`]): the V-cycle unless [`set_default_precond`] said otherwise. Nothing
    /// changes it under a carried step, whose pass would go on with the other. No effect without P5.
    pub fn set_heat_precond(&self, precond: HeatPrecond) {
        if let Some(heat) = &self.heat {
            heat.precond.set(precond);
        }
    }

    /// The preconditioner of P5's PCG; none without P5.
    pub fn heat_precond(&self) -> Option<HeatPrecond> {
        self.heat.as_ref().map(|heat| heat.precond.get())
    }

    /// The multigrid's levels built for the first Newton pass of the last step run (its system at T⁰, from the frozen planes
    /// and faces a step leaves), and level 0 read back; none without P5. Blocks on the device (tests).
    pub fn heat_first_system(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<HeatSystem> {
        let heat = self.heat.as_ref()?;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("heat.mg_build") });
        record_dispatches(&mut encoder.begin_compute_pass(&Default::default()), &heat.mg_build);
        queue.submit([encoder.finish()]);
        let b = self.read_box(device, queue);
        let (x0, y0, w, h) = (b.x0 as usize, b.y0 as usize, b.w as usize, b.h as usize);
        let (ww, wh) = (heat.world.width as usize, heat.world.height as usize);
        let coef = read_buffers(device, queue, "heat.mg_coef", std::iter::once(&heat.mg_coef));
        let d = read_buffers(device, queue, "heat.faces", std::iter::once(&heat.faces));
        let [base, _, lw, lh] = heat.mg_layout[0].map(|v| v as usize);
        let fx = base + lw * lh;
        let fy = fx + (lw + 1) * lh;
        let own = coef[base..base + w * h].to_vec();
        let gas = |i: isize, j: isize| i >= 0 && j >= 0 && (i as usize) < w && (j as usize) < h && own[j as usize * w + i as usize] >= 0.0;
        // The D_f of each gas cell's faces whose far side is no gas cell of the box (heat.wgsl `face_d`'s layout).
        let vac = (0..w * h)
            .map(|k| {
                let (i, j) = ((k % w) as isize, (k / w) as isize);
                let (x, y) = (x0 + i as usize, y0 + j as usize);
                let faces = [(y * (ww + 1) + x, -1, 0), (y * (ww + 1) + x + 1, 1, 0), ((ww + 1) * wh + y * ww + x, 0, -1), ((ww + 1) * wh + (y + 1) * ww + x, 0, 1)];
                if !gas(i, j) {
                    return 0.0;
                }
                faces.iter().filter(|&&(_, di, dj)| !gas(i + di, j + dj)).map(|&(f, _, _)| d[f]).sum()
            })
            .collect();
        let (last, coarse) = mg_levels(w as u64, h as u64);
        let (fx, fy) = (coef[fx..fx + (w + 1) * h].to_vec(), coef[fy..fy + w * (h + 1)].to_vec());
        Some(HeatSystem { x0, y0, w, h, own, vac, fx, fy, levels: last + 1, coarse })
    }

    /// M⁻¹ `r` by the preconditioner chosen, on the system [`Step::heat_first_system`] last built: `r` and the result over
    /// the box as its level 0 (0 off the gas). Panics without P5. Blocks on the device (tests).
    pub fn apply_heat_precond(&self, device: &wgpu::Device, queue: &wgpu::Queue, r: &[f32]) -> Vec<f32> {
        let heat = self.heat.as_ref().expect("the step runs P5");
        let [base, vec, lw, lh] = heat.mg_layout[0].map(|v| v as usize);
        let b = self.read_box(device, queue);
        let (w, h) = (b.w as usize, b.h as usize);
        assert_eq!(r.len(), w * h, "r over the box");
        if heat.precond.get() == HeatPrecond::Multigrid {
            let bytes: Vec<u8> = r.iter().flat_map(|v| v.to_le_bytes()).collect();
            queue.write_buffer(&heat.mg_vec, 4 * vec as u64, &bytes);
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("heat.mg_cycle") });
            record_dispatches(&mut encoder.begin_compute_pass(&Default::default()), &heat.mg_cycle);
            queue.submit([encoder.finish()]);
        }
        let all = read_buffers(device, queue, "heat.mg", [&heat.mg_coef, &heat.mg_vec].into_iter());
        let (coef, v) = all.split_at(heat.mg_coef.size() as usize / 4);
        let (fx, fy) = (base + lw * lh, base + lw * lh + (lw + 1) * lh);
        (0..w * h)
            .map(|k| {
                let (i, j) = (k % w, k / w);
                match (coef[base + k] >= 0.0, heat.precond.get()) {
                    (false, _) => 0.0,
                    (true, HeatPrecond::Multigrid) => v[vec + 2 * lw * lh + k],
                    (true, HeatPrecond::Jacobi) => {
                        let faces = coef[fx + j * (w + 1) + i] + coef[fx + j * (w + 1) + i + 1] + coef[fy + j * w + i] + coef[fy + (j + 1) * w + i];
                        r[k] / (coef[base + k] + faces)
                    }
                }
            })
            .collect()
    }
}

/// heat.wgsl's `Machines` as words. Blocks on the device.
fn machine_words(device: &wgpu::Device, queue: &wgpu::Queue, heat: &Heat) -> Vec<u32> {
    read_buffers(device, queue, "heat.machine", std::iter::once(&heat.machine)).iter().map(|v| v.to_bits()).collect()
}
