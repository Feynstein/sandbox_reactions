//! One simulation step (m0_contrat.md §1.3.2): the passes P0–P9, in that order, each a fixed list of compute
//! dispatches built once for the state's world. A step records the lists as they are — it reads no wall clock, no
//! rung, no frame and no label — so the same state and step count give the same state, bit for bit (§1.3.4).
//!
//! Today P1, P2, P4, P5, P6, P8 and P9 hold dispatches: P1 and P9 the Δt and the non-finite guard (shaders/reduce.wgsl, [`dt`],
//! M0-T20), P2 gravity when the step is built with it ([`grav`], [`Step::with_gravity`], M0-T31), P4 the gas flow when
//! the step is built with it ([`hydro`], [`Step::with_passes`], M0-T34; with gravity's source when P2 runs too,
//! M0-T36), P5 heat when the step is built with it ([`heat`], [`Step::with_heat`], M0-T42), P6 burning when the step is
//! built with it ([`react`], [`Step::with_reactions`], M0-T46), P8 the floors, then the
//! species renormalisation (shaders/floors.wgsl, M0-T19); every other pass is an empty slot its lot fills, in place,
//! without reordering the others. A step knows its index — the guard runs on every 64th
//! (index ≡ 0 mod 64), and step 0 is preceded by the initial state's Δt reduction — after P2 on that state, so step 0's
//! Δt has its gravity term.
//!
//! Every pass runs inside the active box ([`boxfit`], §1.3.3, M0-T21): a box-sized dispatch takes its workgroup count
//! from the box's indirect sizes on the GPU ([`Size::Indirect`]), never from a readback. The box is fit before step 0
//! (ahead of the initial Δt reduction), after P9 at step indices ≡ 0 (mod 16), and in P0 after an edit outside it.
//!
//! A frame ([`latch`], §1.8.3, §1.8.5, M0-T22) records N steps under the latch: every dispatch of a step then takes its
//! workgroup count from a buffer the latch's controller can zero, and the guard's and the re-fit's schedule is the GPU's
//! step index, not this step's CPU count. A step with P5 records its bare runs under the latch too, the slow-down off:
//! its P5 may carry over into the next submission (§0.5 [M0-TJ4] (6), M0-T121), so [`Step::run`] submits until its
//! steps have ended.
//!
//! The FFT and the zero-padded convolution over the box ([`fft`], shaders/fft.wgsl, M0-T30) are machinery, not a pass:
//! P2 builds on them ([`grav`]), P7 when its lot builds it.
//!
//! The equation of state every pass shares (shaders/eos.wgsl) is prepended to a pass's shader ([`with_eos`]) and bound
//! as group 1 from [`EosGpu`], built once per run from physics.json and elements.json: the species table, the cold
//! pairs, M0-T12's u(x) table (sr_physics::eos::ColdTable) and the floors.
//!
//! The cost of a step (§1.8.3's c_step, §3.1, §2.12.2's `timing`; M0-T23), off until [`Step::set_timing`]: each pass of
//! a timed step is a compute pass of its own that writes a timestamp at its beginning and its end where the device has
//! WebGPU's `timestamp-query` — at pass boundaries only, never inside a pass (wgpu's TIMESTAMP_QUERY_INSIDE_PASSES is
//! native-only, §6.2.2) — else the batch is timed by the wall clock and says so ([`CostSource`]). A step nothing times
//! is one compute pass (M0-D26). A batch is a frame, or the bare steps recorded between two
//! [`Step::read_cost`] calls; it times as many whole steps as one query set holds.

pub mod boxfit;
pub mod dt;
pub mod fft;
pub mod grav;
pub mod heat;
pub mod hydro;
pub mod latch;
pub mod react;

use std::cell::{Cell, RefCell};

use sr_physics::eos::{ColdTable, Eos, TABLE_POINTS, X_MIN};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

use crate::scene::EdgeMode;
use crate::state::{Booking, SideFields, State, BOOK_GROUPS, TERMS};
use boxfit::{BoxFit, BoxMode, BoxReport, ARGS_TILES, REFIT_EVERY};
use dt::{Dt, DtReport, GUARD_EVERY};
use fft::FftError;
use grav::Gravity;
use heat::HeatSetup;
use latch::Latch;
use react::ReactSetup;

/// The passes of one step, in §1.3.2's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pass {
    Edits,
    Dt,
    Gravity,
    Sinks,
    Hydro,
    Heat,
    Reactions,
    NeutrinoHeating,
    Floors,
    Reductions,
}

impl Pass {
    pub const ORDER: [Pass; 10] = [
        Pass::Edits,
        Pass::Dt,
        Pass::Gravity,
        Pass::Sinks,
        Pass::Hydro,
        Pass::Heat,
        Pass::Reactions,
        Pass::NeutrinoHeating,
        Pass::Floors,
        Pass::Reductions,
    ];

    /// The debugger's name for the pass, "P0 edits" … "P9 reductions".
    pub fn label(self) -> &'static str {
        match self {
            Pass::Edits => "P0 edits",
            Pass::Dt => "P1 dt",
            Pass::Gravity => "P2 gravity",
            Pass::Sinks => "P3 sinks",
            Pass::Hydro => "P4 hydro",
            Pass::Heat => "P5 heat",
            Pass::Reactions => "P6 reactions",
            Pass::NeutrinoHeating => "P7 neutrino heating",
            Pass::Floors => "P8 floors",
            Pass::Reductions => "P9 reductions",
        }
    }
}

/// One compute dispatch of a pass: its entry point, a pipeline, its bind groups (group g at index g) and its workgroup
/// count.
pub struct Dispatch {
    pub label: &'static str,
    pub pipeline: wgpu::ComputePipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub size: Size,
}

/// A dispatch's workgroup count: fixed when built, or read by the GPU from a buffer at a byte offset (the box's sizes).
#[derive(Clone, Debug)]
pub enum Size {
    Fixed([u32; 3]),
    Indirect(wgpu::Buffer, u64),
    /// Read from a buffer its own pass writes: a chain of dispatches, each sized by the one before it, every size zeroed
    /// at the pass's end — P5's rounds ([`heat`], M0-T118). The latch reaches the chain through the fixed-size dispatch
    /// that opens it, so it needs no slot of its own.
    Chained(wgpu::Buffer, u64),
}

/// The optional passes a step is built with (§2.12.1's switches) and its box mode — [`Step::with_passes`].
#[derive(Clone, Copy)]
pub struct Passes<'a> {
    pub mode: BoxMode,
    /// P2 gravity, φ into these side fields.
    pub gravity: Option<&'a SideFields>,
    /// P4 hydro, starting in this edge mode ([`Step::set_edge_mode`] changes it).
    pub hydro: Option<EdgeMode>,
}

/// The step: one dispatch list per pass, indexed as `Pass::ORDER`, the Δt record, the active box, gravity's field when
/// it runs P2, the latch and the index of the next step.
pub struct Step {
    passes: [Vec<Dispatch>; 10],
    dt: Dt,
    gravity: Option<Gravity>,
    hydro: Option<hydro::Hydro>,
    heat: Option<heat::Heat>,
    react: Option<react::React>,
    boxfit: BoxFit,
    latch: Latch,
    next: Cell<u64>,
    timer: Timer,
}

/// Whether and how a step's cost is measured ([`Step::set_timing`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing {
    Off,
    /// GPU timestamps where the device has `timestamp-query`, else the wall clock.
    Auto,
    /// The wall clock even where timestamps are offered (the fallback's own test).
    Wall,
}

/// Where a cost came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostSource {
    GpuTimestamp,
    Wall,
}

impl CostSource {
    /// The name summary.json and the logs give it.
    pub fn name(self) -> &'static str {
        match self {
            CostSource::GpuTimestamp => "gpu-timestamp",
            CostSource::Wall => "wall",
        }
    }
}

/// The cost of a batch's steps (§1.8.3's c_step input, §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepCost {
    pub source: CostSource,
    /// Steps timed: the batch's first whole recorded steps, as many as the query set holds; every step under wall time.
    /// A step whose P5 carried over records again in the next submission, and each recording counts here.
    pub steps: u32,
    /// Steps the batch ended (§0.5 [M0-TJ4] (6)): its recorded steps but those a frame's latch stopped and the recordings
    /// a carried step took before the one it ended in.
    pub ended: u32,
    /// Mean ms per step ended, from each recorded step's first pass's beginning to its last pass's end (the gaps between
    /// passes included): the timed steps' mean scaled by the recorded steps over the ended ones, so a carried step's rounds
    /// in every submission are its own; under wall time, the batch's wall time ÷ its ended steps — recording, submission
    /// and readback included.
    pub step_ms: f64,
    /// Mean ms per step ended spent in each pass, in `Pass::ORDER` (step 0's initial reduction is P9's), scaled likewise;
    /// none for a pass the step does not record, and every pass under wall time. A recorded step a frame's latch or a
    /// carry stopped runs its dispatches zero-sized: their near-zero cost lands on the steps that ended.
    pub pass_ms: [Option<f64>; 10],
    /// A frame's own passes, `frame begin` + `frame end`, in ms; none for bare steps and under wall time.
    pub frame_ms: Option<f64>,
}

/// The query set's size: wgpu's maximum.
const QUERIES: u32 = wgpu::QUERY_SET_MAX_QUERIES;
/// A step records at most 11 compute passes (P0–P9 and step 0's initial reduction); a step is timed only when 12
/// passes' queries still fit, with `frame end`'s two kept back.
const STEP_QUERIES: u32 = 2 * 12;
/// The slot of a frame's own passes in a batch's list (the passes are 0–9, `Pass::ORDER`).
const FRAME_SLOT: usize = 10;

/// The step's timer: the query set, its resolve buffer and the timestamp period when the GPU times, and the batch being
/// recorded.
#[derive(Default)]
struct Timer {
    mode: Cell<Option<CostSource>>,
    gpu: RefCell<Option<GpuTimer>>,
    batch: RefCell<Batch>,
}

struct GpuTimer {
    queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    period_ns: f64,
}

/// The passes a batch timed, in query order (pass i wrote queries 2i and 2i + 1): (slot, the step within the batch —
/// none for a frame's own passes).
#[derive(Clone, Debug, Default)]
pub(crate) struct Batch {
    live: bool,
    passes: Vec<(usize, Option<u32>)>,
    steps: u32,
    /// The steps a gated bare batch ended ([`Step::run`]); none when every recorded step ends.
    ended: Option<u32>,
    timed: u32,
    open: bool,
    started_ms: Option<f64>,
}

impl Timer {
    /// Starts a batch (a frame, or the first bare step since the last read).
    fn start(&self) {
        *self.batch.borrow_mut() = Batch { live: true, started_ms: wall_ms(), ..Batch::default() };
    }

    /// A step begins: it is timed when every pass it may record still fits.
    fn begin_step(&self) {
        let mut b = self.batch.borrow_mut();
        b.steps += 1;
        b.open = self.mode.get() == Some(CostSource::GpuTimestamp)
            && b.timed + 1 == b.steps
            && 2 * b.passes.len() as u32 + STEP_QUERIES + 2 <= QUERIES;
        if b.open {
            b.timed += 1;
        }
    }

    /// Whether the step being recorded times its passes: each is then a compute pass of its own.
    fn times_passes(&self) -> bool {
        self.mode.get() == Some(CostSource::GpuTimestamp) && self.batch.borrow().open
    }

    /// Begins a compute pass labelled `label`, its two timestamps booked to `slot` when the batch times it.
    fn begin<'e>(&self, encoder: &'e mut wgpu::CommandEncoder, label: &str, slot: usize) -> wgpu::ComputePass<'e> {
        let gpu = self.gpu.borrow();
        let mut b = self.batch.borrow_mut();
        let timed = self.mode.get() == Some(CostSource::GpuTimestamp) && (slot == FRAME_SLOT || b.open);
        let writes = match (&*gpu, timed) {
            (Some(g), true) => {
                let i = 2 * b.passes.len() as u32;
                let step = (slot != FRAME_SLOT).then(|| b.steps - 1);
                b.passes.push((slot, step));
                Some(wgpu::ComputePassTimestampWrites {
                    query_set: &g.queries,
                    beginning_of_pass_write_index: Some(i),
                    end_of_pass_write_index: Some(i + 1),
                })
            }
            _ => None,
        };
        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(label), timestamp_writes: writes })
    }

    /// Ends the batch: records the resolve of its queries (into the resolve buffer, from byte 0) and returns the batch,
    /// with the resolve buffer and the bytes to copy when the GPU timed it. None when timing is off or nothing ran.
    fn finish(&self, encoder: &mut wgpu::CommandEncoder) -> Option<(Batch, Option<(wgpu::Buffer, u64)>)> {
        let batch = std::mem::take(&mut *self.batch.borrow_mut());
        if self.mode.get().is_none() || batch.steps == 0 {
            return None;
        }
        let gpu = self.gpu.borrow();
        let resolved = match (&*gpu, batch.passes.len() as u32) {
            (Some(g), n) if n > 0 => {
                encoder.resolve_query_set(&g.queries, 0..2 * n, &g.resolve, 0);
                Some((g.resolve.clone(), 16 * u64::from(n)))
            }
            _ => None,
        };
        Some((batch, resolved))
    }

    /// The cost of a finished batch: from its resolved timestamps (`ticks`), or from the wall clock now when it has
    /// none; `ended` is how many steps it ended (a frame's, after the latch and a carry).
    fn cost(&self, batch: &Batch, ticks: Option<&[u64]>, ended: u32) -> Option<StepCost> {
        let period_ns = self.gpu.borrow().as_ref().map_or(0.0, |g| g.period_ns);
        let Some(ticks) = ticks else {
            let ms = wall_ms()? - batch.started_ms?;
            let steps = ended.max(1);
            let step_ms = ms / f64::from(steps);
            return Some(StepCost { source: CostSource::Wall, steps, ended, step_ms, pass_ms: [None; 10], frame_ms: None });
        };
        let ms = |b: u64, e: u64| e.saturating_sub(b) as f64 * period_ns / 1e6;
        let (mut pass_ms, mut frame_ms) = ([None::<f64>; 10], None::<f64>);
        let mut spans = vec![(u64::MAX, 0u64); batch.timed as usize];
        for (k, &(slot, step)) in batch.passes.iter().enumerate() {
            let (b, e) = (ticks[2 * k], ticks[2 * k + 1]);
            match step {
                Some(s) => {
                    *pass_ms[slot].get_or_insert(0.0) += ms(b, e);
                    let span = &mut spans[s as usize];
                    *span = (span.0.min(b), span.1.max(e));
                }
                None => *frame_ms.get_or_insert(0.0) += ms(b, e),
            }
        }
        // Per timed recording, then per step ended: the batch's recordings over the steps they ended.
        let n = f64::from(batch.timed.max(1)) * f64::from(ended.max(1)) / f64::from(batch.steps.max(1));
        let step_ms = spans.iter().map(|&(b, e)| ms(b, e)).sum::<f64>() / n;
        let pass_ms = pass_ms.map(|p| p.map(|v| v / n));
        Some(StepCost { source: CostSource::GpuTimestamp, steps: batch.timed, ended, step_ms, pass_ms, frame_ms })
    }
}

/// Milliseconds on the wall clock since the first call; none on wasm32, which has no clock without a new dependency.
fn wall_ms() -> Option<f64> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        Some(EPOCH.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() * 1e3)
    }
    #[cfg(target_arch = "wasm32")]
    None
}

impl Step {
    /// Builds every pass's dispatch list for `state`'s buffers and world, booking into `booking`, with the equation of
    /// state `eos` and the run's constants `physics` (C, η_g, Σ_vac), the box fit to the state. The next step it
    /// records is step 0.
    pub fn new(device: &wgpu::Device, state: &State, booking: &Booking, eos: &EosGpu, physics: &Physics) -> Step {
        Step::with_box(device, state, booking, eos, physics, BoxMode::Fit)
    }

    /// As [`Step::new`], the box fit to the state or held at the world (the switch for tests, G-BOX).
    pub fn with_box(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        mode: BoxMode,
    ) -> Step {
        Step::with_passes(device, state, booking, eos, physics, &Passes { mode, gravity: None, hydro: None })
            .expect("a step without gravity builds")
    }

    /// As [`Step::with_box`], with P2 gravity (§1.4; §2.12.1's `gravity` switch on): φ into `side`'s φ plane, g kept by
    /// the step ([`Step::gravity_field`]), Δt's gravity term from g. Refused when a world axis pads past the FFT's
    /// largest length (over 1024 cells).
    pub fn with_gravity(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        mode: BoxMode,
        side: &SideFields,
    ) -> Result<Step, FftError> {
        Step::with_passes(device, state, booking, eos, physics, &Passes { mode, gravity: Some(side), hydro: None })
    }

    /// The step with the passes `passes` names beyond P1, P8 and P9: P2 gravity as [`Step::with_gravity`] (refused
    /// likewise), P4 hydro (§2.3, §2.8; M0-T34).
    pub fn with_passes(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        passes: &Passes<'_>,
    ) -> Result<Step, FftError> {
        Step::with_heat(device, state, booking, eos, physics, passes, None)
    }

    /// As [`Step::with_passes`], with P5 heat when `heat` is given (§2.4; §2.12.1's `heat` switch on; M0-T42): F kept in
    /// its side fields, the scene's test flags applied; P4, when the step runs it, pushes the gas with the F the last step
    /// left there (§2.4.4; M0-T43).
    pub fn with_heat(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        passes: &Passes<'_>,
        heat: Option<&HeatSetup<'_>>,
    ) -> Result<Step, FftError> {
        Step::with_reactions(device, state, booking, eos, physics, passes, heat, None)
    }

    /// As [`Step::with_heat`], with P6 burning when `react` is given (§2.5.1, §2.5.2; §2.12.1's `reactions` switch on;
    /// M0-T46): the enabled records of `react.reactions`, the scene's `pin_temperature`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_reactions(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        passes: &Passes<'_>,
        heat: Option<&HeatSetup<'_>>,
        react: Option<&ReactSetup<'_>>,
    ) -> Result<Step, FftError> {
        let (mode, side, edge) = (passes.mode, passes.gravity, passes.hydro);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("reduce.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/reduce.wgsl")).into()),
        });
        let mut boxfit = BoxFit::new(device, state, physics, mode);
        let mut gravity = side.map(|side| Gravity::new(device, state, side, physics, &boxfit)).transpose()?;
        let mut dt = Dt::new(device, &module, state, eos, physics, &boxfit, gravity.as_ref().map(Gravity::field));
        boxfit.build(device, &module, state, dt.record());
        let mut passes: [Vec<Dispatch>; 10] = Default::default();
        passes[Pass::Dt as usize] = std::mem::take(&mut dt.advance);
        passes[Pass::Gravity as usize] = gravity.as_mut().map(|g| std::mem::take(&mut g.dispatches)).unwrap_or_default();
        let g = gravity.as_ref().map(Gravity::field);
        let flux = heat.map(|h| h.side.flux());
        let mut flow = edge.map(|edge| hydro::Hydro::new(device, state, booking, eos, physics, &boxfit, dt.record(), g, flux, edge));
        passes[Pass::Hydro as usize] = flow.as_mut().map(|h| std::mem::take(&mut h.dispatches)).unwrap_or_default();
        let mut warm = heat.map(|h| heat::Heat::new(device, state, booking, eos, physics, &boxfit, dt.record(), h));
        passes[Pass::Heat as usize] = warm.as_mut().map(|h| std::mem::take(&mut h.dispatches)).unwrap_or_default();
        let mut burn = react.map(|r| react::React::new(device, state, booking, eos, physics, &boxfit, dt.record(), r));
        passes[Pass::Reactions as usize] = burn.as_mut().map(|r| std::mem::take(&mut r.dispatches)).unwrap_or_default();
        passes[Pass::Floors as usize] = floors(device, state, booking, eos, &boxfit);
        passes[Pass::Reductions as usize] = std::mem::take(&mut dt.reduce);
        let latch = Latch::new(device, &passes, &dt, &boxfit, warm.as_ref().map(heat::Heat::gate_buffers));
        // P6 sets the latch after its cells: its flag copy needs the latch's record, so it joins the list now (one fixed
        // invocation, a size the latch already holds a slot for).
        if let Some(b) = &burn {
            passes[Pass::Reactions as usize].push(b.latch_dispatch(device, latch.flag()));
        }
        Ok(Step { passes, dt, gravity, hydro: flow, heat: warm, react: burn, boxfit, latch, next: Cell::new(0), timer: Timer::default() })
    }

    /// P2's g = −∇φ (the x plane, then the y plane, f32, row-major over the world) when the step runs gravity; φ is the
    /// [`SideFields`] plane it was built with. Inside the box only: outside, the last values P2 wrote there.
    pub fn gravity_field(&self) -> Option<&wgpu::Buffer> {
        self.gravity.as_ref().map(Gravity::field)
    }

    /// The edge mode (§2.8, §3.1) for every step submitted after this call — at the next step boundary, since a write
    /// lands between submissions. Nothing to change when the step runs no P4.
    pub fn set_edge_mode(&self, queue: &wgpu::Queue, edge: EdgeMode) {
        if let Some(h) = &self.hydro {
            h.set_edge_mode(queue, edge);
        }
    }

    /// How many dispatches each pass records on a step the guard skips, in `Pass::ORDER`.
    pub fn dispatch_counts(&self) -> [usize; 10] {
        std::array::from_fn(|p| self.passes[p].len())
    }

    /// The entry points a pass dispatches on a step the guard skips, in its order.
    pub fn dispatch_labels(&self, pass: Pass) -> Vec<&'static str> {
        self.passes[pass as usize].iter().map(|d| d.label).collect()
    }

    /// The entry points P9 adds after the Δt reduction on every 64th step: the non-finite guard.
    pub fn guard_labels(&self) -> Vec<&'static str> {
        self.dt.guard.iter().map(|d| d.label).collect()
    }

    /// The box's entry points: its re-fit (before step 0, and after P9 every 16th step), then P0's, after the edits.
    pub fn box_labels(&self) -> [Vec<&'static str>; 2] {
        [&self.boxfit.refit, &self.boxfit.on_edit].map(|list| list.iter().map(|d| d.label).collect())
    }

    /// Every dispatch of a step whose workgroup count the GPU reads from the box's sizes: (entry point, byte offset in
    /// [`BoxFit::args`]).
    pub fn box_sized(&self) -> Vec<(&'static str, u64)> {
        let all = self.passes.iter().flatten().chain(&self.dt.guard);
        all.filter_map(|d| match &d.size {
            Size::Indirect(b, offset) if b == self.boxfit.args() => Some((d.label, *offset)),
            _ => None,
        })
        .collect()
    }

    /// The box as the GPU last wrote it. Blocks on the device (tests and headless only) — never sizes a dispatch.
    pub fn read_box(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> BoxReport {
        self.boxfit.read(device, queue)
    }

    /// The box's edit flag, set by P0's edit pass when an edit lands outside the box ([`BoxFit::edit_flag`]).
    pub fn box_edit_flag(&self) -> &wgpu::Buffer {
        self.boxfit.edit_flag()
    }

    /// Records the next bare step of a step without P5: every non-empty slot, P0 to P9 — before step 0, the initial
    /// state's Δt — a compute pass each when the step's passes are timed, else one for the step. The
    /// guard and the re-fit follow this step's CPU count; after a frame the latch stopped, that count runs ahead of the
    /// GPU's, so a run uses frames ([`Step::encode_frame`]) or bare steps, not both.
    fn encode(&self, encoder: &mut wgpu::CommandEncoder) {
        if !self.timer.batch.borrow().live {
            self.timer.start();
        }
        self.encode_step(encoder, None);
    }

    /// One step, bare (`latch` none) or under the latch: then every fixed-size dispatch reads its workgroup count from
    /// the latch's sizes, the guard and the re-fit are recorded on every step and opened by the GPU's step index, and the
    /// controller closes P9. With P5, its gate too: `latch_hold` opens P0, and `heat_park` and `latch_carry` follow the
    /// slots P5 records ([`heat`]'s rounds), its commit after them.
    ///
    /// The step's passes are compute passes of their own only when its passes are timed; otherwise the whole step is one
    /// compute pass, the same dispatches in the same order (wgpu still orders each dispatch after the last that wrote
    /// what it reads) — the pass boundaries cost ~0.06 ms a step on a Sun-sized box (M0-D26).
    fn encode_step(&self, encoder: &mut wgpu::CommandEncoder, latch: Option<&Latch>) {
        let n = self.next.get();
        self.timer.begin_step();
        let mut passes: Vec<(&'static str, usize, Lists<'_>)> = Vec::with_capacity(11);
        if n == 0 {
            // The box fit to the initial state, P2 on it, then P9's Δt reduction over it; the dispatches are P2's and P9's
            // own lists.
            let lists = vec![
                (&self.boxfit.refit[..], REFIT_EVERY, false),
                (&self.passes[Pass::Gravity as usize][..], 1, false),
                (&self.passes[Pass::Reductions as usize][..], 1, false),
            ];
            passes.push(("P9 reductions (initial state)", Pass::Reductions as usize, lists));
        }
        let gate = latch.and_then(Latch::gate);
        for (pass, dispatches) in Pass::ORDER.iter().zip(&self.passes) {
            let hooked = latch.is_some_and(|l| l.hooked()) && *pass == Pass::Reactions;
            if dispatches.is_empty() && *pass != Pass::Edits && !hooked {
                continue;
            }
            let mut lists = vec![(&dispatches[..], 1, false)];
            if *pass == Pass::Edits {
                if let Some((hold, _)) = gate {
                    lists.insert(0, (std::slice::from_ref(hold), 1, true));
                }
                lists.push((&self.boxfit.on_edit, 1, false));
            }
            if let (Pass::Heat, Some((_, carry)), Some(h)) = (*pass, gate, &self.heat) {
                let (head, rest) = dispatches.split_at(heat::HEAD);
                let (rounds, tail) = rest.split_at(heat::SLOTS);
                let park = std::slice::from_ref(&h.park);
                lists = vec![(head, 1, false), (&rounds[..h.rounds()], 1, false), (park, 1, true), (std::slice::from_ref(carry), 1, true), (tail, 1, false)];
            }
            if let (true, Some(l)) = (hooked, latch) {
                lists.push((l.hook_dispatches(), 1, false));
            }
            if *pass == Pass::Reductions && (latch.is_some() || n.is_multiple_of(GUARD_EVERY)) {
                lists.push((&self.dt.guard, GUARD_EVERY, false));
            }
            if *pass == Pass::Reductions && (latch.is_some() || n.is_multiple_of(REFIT_EVERY)) {
                lists.push((&self.boxfit.refit, REFIT_EVERY, false));
            }
            if let (Pass::Reductions, Some(l)) = (*pass, latch) {
                lists.push((std::slice::from_ref(l.controller()), 1, true));
            }
            passes.push((pass.label(), *pass as usize, lists));
        }
        let record = |cpass: &mut wgpu::ComputePass<'_>, lists: &Lists<'_>| {
            for &(list, every, bare) in lists {
                match latch {
                    Some(l) if !bare => l.record(cpass, list, every),
                    _ => record_dispatches(cpass, list),
                }
            }
        };
        if self.timer.times_passes() {
            for (label, slot, lists) in &passes {
                record(&mut self.timer.begin(encoder, label, *slot), lists);
            }
        } else {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("step"), timestamp_writes: None });
            for (_, _, lists) in &passes {
                record(&mut cpass, lists);
            }
        }
        self.next.set(n + 1);
    }

    /// The Δt record after every submitted step: Δt_n, Δt_{n+1}, the steps run and the guard's finding. Blocks on the
    /// device (native and headless only, §6.3).
    pub fn read_dt(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> DtReport {
        self.dt.read(device, queue)
    }

    /// The Δt record buffer, which later passes bind to read Δt_n (its first word, f32).
    pub fn dt_record(&self) -> &wgpu::Buffer {
        self.dt.record()
    }

    /// Runs `steps` steps, in order: one submission for a step without P5 (submitted, not waited for); with P5, a
    /// submission of the steps left under the latch's gate, the slow-down off, until they have ended — after a carry the
    /// next submission holds the carried step alone (§0.5 [M0-TJ4] (6)), each waited for (tests and headless).
    pub fn run(&self, device: &wgpu::Device, queue: &wgpu::Queue, steps: u32) {
        self.run_counted(device, queue, steps);
    }

    /// [`Step::run`], returning the submissions it took.
    pub fn run_counted(&self, device: &wgpu::Device, queue: &wgpu::Queue, steps: u32) -> u32 {
        if self.heat.is_some() {
            return self.run_gated(device, queue, steps);
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("step") });
        for _ in 0..steps {
            self.encode(&mut encoder);
        }
        queue.submit([encoder.finish()]);
        1
    }

    /// Turns the cost measurement on or off from the next batch on, and returns the source it will use: GPU timestamps
    /// when `timing` is Auto and the device has `timestamp-query` (the adapter offered it, §6.2.2), else the wall clock.
    pub fn set_timing(&self, device: &wgpu::Device, queue: &wgpu::Queue, timing: Timing) -> Option<CostSource> {
        let stamps = timing == Timing::Auto
            && device.features().contains(wgpu::Features::TIMESTAMP_QUERY)
            && queue.get_timestamp_period() > 0.0;
        let mode = match timing {
            Timing::Off => None,
            _ if stamps => Some(CostSource::GpuTimestamp),
            _ => Some(CostSource::Wall),
        };
        if stamps && self.timer.gpu.borrow().is_none() {
            let queries = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("step.timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: QUERIES,
            });
            let resolve = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("step.timestamps.resolve"),
                size: 8 * u64::from(QUERIES),
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let period_ns = f64::from(queue.get_timestamp_period());
            *self.timer.gpu.borrow_mut() = Some(GpuTimer { queries, resolve, period_ns });
        }
        self.timer.mode.set(mode);
        *self.timer.batch.borrow_mut() = Batch::default();
        mode
    }

    /// The cost of the bare steps submitted since the last read ([`Step::run`]); none when timing is off or no step
    /// ran. Waits for the device (tests and headless only); the next bare step starts a new batch.
    pub fn read_cost(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<StepCost> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("step.cost") });
        let (batch, resolved) = self.timer.finish(&mut encoder)?;
        let ticks = resolved.map(|(resolve, bytes)| {
            let staging = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("step.timestamps.read"),
                size: bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            encoder.copy_buffer_to_buffer(&resolve, 0, &staging, 0, bytes);
            staging
        });
        queue.submit([encoder.finish()]);
        let ticks = ticks.map(|staging| {
            staging.map_async(wgpu::MapMode::Read, .., |r| r.expect("timestamps: map failed"));
            device.poll(wgpu::PollType::wait_indefinitely()).expect("the device waits");
            let view = staging.get_mapped_range(..).expect("mapped for reading");
            view.as_chunks::<8>().0.iter().map(|b| u64::from_le_bytes(*b)).collect::<Vec<u64>>()
        });
        if ticks.is_none() {
            device.poll(wgpu::PollType::wait_indefinitely()).expect("the device waits");
        }
        self.timer.cost(&batch, ticks.as_deref(), batch.ended.unwrap_or(batch.steps))
    }
}

/// A pass's dispatch lists in order: (the dispatches, the schedule a frame's latch opens them on, recorded bare even
/// under the latch — its controller).
type Lists<'a> = Vec<(&'a [Dispatch], u64, bool)>;

/// shaders/eos.wgsl, the helpers every pass shares.
pub const EOS_WGSL: &str = include_str!("../../shaders/eos.wgsl");

/// A pass's shader source with eos.wgsl prepended: its own bindings in group 0, the EOS's in group 1.
pub fn with_eos(source: &str) -> String {
    format!("{EOS_WGSL}\n{source}")
}

/// The size of eos.wgsl's `EosParams` uniform: two species arrays of three vec4, then three rows of four words.
const EOS_PARAMS_BYTES: usize = 2 * 48 + 3 * 16;

/// The equation of state on the GPU, uploaded once per run: `EosParams` (the species table's (Z + 1)/A and Z/A, the
/// cold pairs, the table's geometry, T_floor, Σ_floor, Σ_vac) and both cold tables, electrons then neutron matter, each
/// [`TABLE_POINTS`] ln u then as many segment slopes taken in f64 (zeros, and its flag 0, for a pair that is off).
pub struct EosGpu {
    params: wgpu::Buffer,
    table: wgpu::Buffer,
}

impl EosGpu {
    /// From a run's constants (a scene's `overrides` included) and the element registry. `physics` is validated, so
    /// each cold pair is on or off as a whole.
    pub fn new(device: &wgpu::Device, physics: &Physics, elements: &Elements) -> EosGpu {
        let eos = Eos::from_physics(physics).expect("validated physics.json: each cold pair is on or off");
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let (mut inv_mu, mut y_e) = ([0f32; 12], [0f32; 12]);
        for (i, s) in elements.species().iter().enumerate().take(N_SPECIES) {
            let (a, z) = (f64::from(s.a), f64::from(s.z));
            inv_mu[i] = ((z + 1.0) / a) as f32;
            y_e[i] = (z / a) as f32;
        }
        let pair = |t: Option<&ColdTable>| t.map_or((0.0, 0.0, 0u32), |t| (t.k1() as f32, t.k2() as f32, 1));
        let (k1e, k2e, cold_e) = pair(eos.electron_table());
        let (k1n, k2n, cold_n) = pair(eos.neutron_table());
        let mut words: Vec<u32> = inv_mu.iter().chain(&y_e).map(|v| v.to_bits()).collect();
        words.extend([k1e, k2e, k1n, k2n].map(f32::to_bits));
        let geometry = [X_MIN.ln() as f32, (1.0 / ColdTable::step()) as f32, get("t_floor"), get("sigma_floor")];
        words.extend(geometry.map(f32::to_bits));
        words.extend([get("sigma_vac").to_bits(), cold_e, cold_n, TABLE_POINTS as u32]);
        assert_eq!(words.len() * 4, EOS_PARAMS_BYTES);

        let mut table: Vec<f32> = Vec::with_capacity(4 * TABLE_POINTS);
        for t in [eos.electron_table(), eos.neutron_table()] {
            match t {
                Some(t) => {
                    let ln_u = t.ln_u();
                    table.extend(ln_u.iter().map(|&v| v as f32));
                    table.extend(ln_u.windows(2).map(|w| (w[1] - w[0]) as f32));
                    table.push(0.0);
                }
                None => table.extend(std::iter::repeat_n(0.0, 2 * TABLE_POINTS)),
            }
        }
        let table: Vec<u32> = table.iter().map(|v| v.to_bits()).collect();
        EosGpu {
            params: init_buffer(device, "eos.params", wgpu::BufferUsages::UNIFORM, &words),
            table: init_buffer(device, "eos.table", wgpu::BufferUsages::STORAGE, &table),
        }
    }

    /// The group-1 bind group of a pipeline compiled with [`with_eos`].
    pub fn bind_group(&self, device: &wgpu::Device, pipeline: &wgpu::ComputePipeline) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("eos"),
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.params.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.table.as_entire_binding() },
            ],
        })
    }
}

/// A buffer created holding `words`, little-endian.
fn init_buffer(device: &wgpu::Device, label: &str, usage: wgpu::BufferUsages, words: &[u32]) -> wgpu::Buffer {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage,
        mapped_at_creation: true,
    });
    buffer.get_mapped_range_mut(..).expect("mapped at creation").copy_from_slice(&bytes);
    buffer.unmap();
    buffer
}

/// Sets each dispatch's pipeline and bind groups and dispatches it, in order.
fn record_dispatches(cpass: &mut wgpu::ComputePass<'_>, dispatches: &[Dispatch]) {
    for d in dispatches {
        record_sized(cpass, d, &d.size);
    }
}

/// Sets `d`'s pipeline and bind groups and dispatches it with `size` (its own, or the latch's in its place).
fn record_sized(cpass: &mut wgpu::ComputePass<'_>, d: &Dispatch, size: &Size) {
    cpass.set_pipeline(&d.pipeline);
    for (g, bind_group) in d.bind_groups.iter().enumerate() {
        cpass.set_bind_group(g as u32, bind_group, &[]);
    }
    match size {
        Size::Fixed([x, y, z]) => cpass.dispatch_workgroups(*x, *y, *z),
        Size::Indirect(buffer, offset) | Size::Chained(buffer, offset) => cpass.dispatch_workgroups_indirect(buffer, *offset),
    }
}

fn compute_pipeline(device: &wgpu::Device, module: &wgpu::ShaderModule, entry: &'static str) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry),
        layout: None,
        module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
    })
}

/// P8 floors (§1.3.2): `floor_cells` — the vacuum reset and the temperature floor, booked into P8's booking buffer —
/// then `renormalise_species`, both over the box's 8 × 8 tiles.
fn floors(device: &wgpu::Device, state: &State, booking: &Booking, eos: &EosGpu, boxfit: &BoxFit) -> Vec<Dispatch> {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("floors.wgsl"),
        source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/floors.wgsl")).into()),
    });
    let book = BOOK_GROUPS
        .iter()
        .position(|g| TERMS[g.terms.start].writer == Pass::Floors)
        .expect("a booking buffer written by P8");

    let floor_cells = compute_pipeline(device, &module, "floor_cells");
    let cells_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P8 floor_cells"),
        layout: &floor_cells.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: state.dims().as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: state.group(1).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: state.group(2).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: state.group(0).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: booking.group(book).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: boxfit.active().as_entire_binding() },
        ],
    });
    let eos_group = eos.bind_group(device, &floor_cells);

    let renormalise = compute_pipeline(device, &module, "renormalise_species");
    let species_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P8 renormalise_species"),
        layout: &renormalise.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: state.dims().as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: state.group(1).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: state.group(2).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: boxfit.active().as_entire_binding() },
        ],
    });
    let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);

    vec![
        Dispatch { label: "floor_cells", pipeline: floor_cells, bind_groups: vec![cells_group, eos_group], size: tiles() },
        Dispatch { label: "renormalise_species", pipeline: renormalise, bind_groups: vec![species_group], size: tiles() },
    ]
}
