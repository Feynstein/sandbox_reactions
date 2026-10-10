//! M0-T22 — a frame of steps and the collapse latch (contract §1.8.3, §1.8.5, §1.8.7, §3.1; TW-R4, TW-E16; G-LATCH's
//! mechanism — its contract scenario, the Massive preset from an iron-core dump, comes with M0-T111: until then
//! NOT PROVEN (synthetic)).
//!
//! The oracle is a run that never had the latch fire: a frame stopped after its k-th step must leave the state, the Δt
//! record and the box bit-identical to a k-step frame of the same start, and no box-sized dispatch may run after the
//! k-th step (the hook's probe counts them). Every number is exact (integers and bits). The flag is set by the test hook
//! in P6, at chosen GPU step indices; the start is cold hydrogen at rest, so Δt = C / (2 √(2 T_floor (Z + 1)/A)) ≈ 0.447
//! is the same on every step and 1.0 t.u. is 3 steps, never 2 (the case that needs it checks Δt first).

use super::device;
use sr_engine::gpu::Gpu;
use sr_engine::state::{Booking, State, WorldConfig, CHANNELS, N_CHANNELS};
use sr_engine::step::boxfit::BoxReport;
use sr_engine::step::dt::DtReport;
use sr_engine::step::latch::{FramePlan, FrameReport};
use sr_engine::step::{EosGpu, Step};
use sr_physics::registry::{Elements, Physics};

/// A 40-step frame (the block's), and §1.8.5's window.
const FRAME: u32 = 40;
const WINDOW: f32 = 1.0;

struct Constants {
    physics: Physics,
    elements: Elements,
}

/// The shipped constants, the cold pairs off (as tests/gpu/box.rs).
fn constants() -> Constants {
    let physics = Physics::shipped().unwrap().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).unwrap();
    Constants { physics, elements: Elements::shipped().unwrap() }
}

/// A 256 × 128 world of vacuum hydrogen with a 10 × 10 block of gas at rest (Σ = 1, E = 0) on x 100..110, y 50..60.
fn start() -> (WorldConfig, Vec<f32>) {
    let world = WorldConfig::new(256, 128).unwrap();
    let n = world.cells();
    let h = CHANNELS.iter().position(|&c| c == "x_H").unwrap();
    let mut p = vec![0f32; N_CHANNELS * n];
    p[h * n..(h + 1) * n].fill(1.0);
    for y in 50..60 {
        for x in 100..110 {
            p[y * 256 + x] = 1.0;
        }
    }
    (world, p)
}

/// Everything a step leaves behind that the test can read: the state's bits, the Δt record, the box.
#[derive(Debug, PartialEq)]
struct Outcome {
    state: Vec<u32>,
    dt: DtReport,
    abox: BoxReport,
}

struct Run {
    state: State,
    step: Step,
}

impl Run {
    fn new(gpu: &Gpu, c: &Constants) -> Run {
        let (world, planes) = start();
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let step = Step::new(&gpu.device, &state, &booking, &eos, &c.physics);
        Run { state, step }
    }

    /// One frame, submitted, waited for, and its report.
    fn frame(&self, gpu: &Gpu, steps: u32, slow_down: bool) -> FrameReport {
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.step.encode_frame(&FramePlan { steps, slow_down }, &mut encoder);
        gpu.queue.submit([encoder.finish()]);
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let reports = self.step.poll(&gpu.device);
        assert_eq!(reports.len(), 1, "one report per frame: {reports:?}");
        reports[0]
    }

    fn outcome(&self, gpu: &Gpu) -> Outcome {
        Outcome {
            state: self.state.readback(&gpu.device, &gpu.queue).iter().map(|v| v.to_bits()).collect(),
            dt: self.step.read_dt(&gpu.device, &gpu.queue),
            abox: self.step.read_box(&gpu.device, &gpu.queue),
        }
    }
}

/// A run of `frames` (steps per frame) on a fresh start, slow-down on, the latch never set.
fn reference(gpu: &Gpu, c: &Constants, frames: &[u32]) -> Outcome {
    let run = Run::new(gpu, c);
    for &n in frames {
        let r = run.frame(gpu, n, true);
        assert_eq!((r.steps_run, r.latch_fired), (n, None), "the reference never fires");
    }
    run.outcome(gpu)
}

/// Δt of the start, read from a one-step bare run; the window cases need 2 Δt < 1.0 ≤ 3 Δt.
fn dt_of_start(gpu: &Gpu, c: &Constants) -> f32 {
    let run = Run::new(gpu, c);
    run.step.run(&gpu.device, &gpu.queue, 1);
    run.step.read_dt(&gpu.device, &gpu.queue).dt
}

fn assert_same(got: &Outcome, want: &Outcome, what: &str) {
    assert_eq!(got.dt, want.dt, "{what}: the Δt record");
    assert_eq!(got.abox, want.abox, "{what}: the box (re-fits counted)");
    let diff = got.state.iter().zip(&want.state).filter(|(a, b)| a != b).count();
    assert_eq!(diff, 0, "{what}: {diff} state words differ from the reference");
}

#[test]
fn a_flag_set_at_step_k_of_a_40_step_frame_runs_exactly_k_steps_bit_identical_to_a_k_step_run() {
    let gpu = device();
    let c = constants();
    // k = 15: the re-fit after step index 16 falls inside the stopped stretch (steps 16 … 40 of the frame).
    for k in [1u32, 15, 40] {
        let run = Run::new(&gpu, &c);
        run.step.set_latch_hook(&gpu.queue, Some((k - 1, k, 1)));
        let r = run.frame(&gpu, FRAME, true);
        let probe = run.step.latch_probe(&gpu.device, &gpu.queue);
        println!("k = {k}: {r:?}, box-sized hook dispatches run {probe}");
        assert_eq!((r.steps_run, r.latch_fired, r.steps), (k, Some(k), u64::from(k)), "k = {k}");
        assert_eq!(probe, k, "k = {k}: a box-sized dispatch ran after the latch fired");
        assert_same(&run.outcome(&gpu), &reference(&gpu, &c, &[k]), &format!("k = {k}"));

        // The next frame runs whole: the box's sizes are back and the slots open on the GPU's step index.
        run.step.set_latch_hook(&gpu.queue, None);
        let r = run.frame(&gpu, 10, true);
        assert_eq!((r.steps_run, r.latch_fired, r.steps), (10, None, u64::from(k + 10)), "k = {k}, the frame after");
        assert_same(&run.outcome(&gpu), &reference(&gpu, &c, &[k + 10]), &format!("k = {k}, then 10"));
    }
}

#[test]
fn with_the_slow_down_off_the_frame_runs_all_40_steps_through_the_flag() {
    let gpu = device();
    let c = constants();
    let run = Run::new(&gpu, &c);
    run.step.set_latch_hook(&gpu.queue, Some((14, 15, 1)));
    let r = run.frame(&gpu, FRAME, false);
    let probe = run.step.latch_probe(&gpu.device, &gpu.queue);
    println!("slow-down off, flag at step 15: {r:?}, box-sized hook dispatches run {probe}");
    assert_eq!((r.steps_run, r.latch_fired, r.steps, probe), (FRAME, None, u64::from(FRAME), FRAME));
    assert_same(&run.outcome(&gpu), &reference(&gpu, &c, &[FRAME]), "slow-down off");
}

#[test]
fn the_latch_re_arms_only_after_1_tu_without_a_set() {
    let gpu = device();
    let c = constants();
    let dt = dt_of_start(&gpu, &c);
    println!("Δt of the start {dt}");
    assert!(dt + dt < WINDOW && dt + dt + dt >= WINDOW, "Δt {dt}: the case needs 2 Δt < 1.0 ≤ 3 Δt");
    let run = Run::new(&gpu, &c);
    // (flag at step indices lo, lo + stride, … below hi; steps run; fired after the frame's step)
    let script: [((u32, u32, u32), u32, Option<u32>); 4] = [
        ((4, 5, 1), 5, Some(5)),          // never set before: armed — fires after index 4, 5 steps
        ((6, 45, 2), FRAME, None),        // every 2 Δt from index 6: never 1.0 since the last set, though ≫ 1.0 since the fire
        ((46, 50, 3), 5, Some(5)),        // index 46: 2 Δt since 44 — held, and restarted; index 49: 3 Δt since 46 — fires
        ((88, 89, 1), FRAME, None),       // index 88: armed, but the slow-down is off — no fire, the window restarts
    ];
    let mut steps = 0u64;
    for (i, (hook, run_steps, fired)) in script.into_iter().enumerate() {
        run.step.set_latch_hook(&gpu.queue, Some(hook));
        let r = run.frame(&gpu, FRAME, i != 3);
        steps += u64::from(run_steps);
        println!("frame {i}, hook {hook:?}: {r:?}");
        assert_eq!((r.steps_run, r.latch_fired, r.steps), (run_steps, fired, steps), "frame {i}, hook {hook:?}");
    }
    // The slow-down back on: index 90 is 2 Δt after the set at 88 made with it off — held; index 93, 3 Δt after 90, fires.
    run.step.set_latch_hook(&gpu.queue, Some((90, 94, 3)));
    let r = run.frame(&gpu, FRAME, true);
    assert_eq!((r.steps_run, r.latch_fired, r.steps), (4, Some(4), steps + 4), "flags at indices 90 and 93");
}

#[test]
fn an_edit_flagged_on_the_step_the_latch_fires_refits_the_box_once_then_nothing_runs() {
    let gpu = device();
    let c = constants();
    let flagged = |run: &Run| gpu.queue.write_buffer(run.step.box_edit_flag(), 0, &1u32.to_le_bytes());
    // The reference: 3 steps, the edit flag set, 1 step (its P0 re-fits the box), the latch never set.
    let want = {
        let run = Run::new(&gpu, &c);
        run.frame(&gpu, 3, true);
        flagged(&run);
        run.frame(&gpu, 1, true);
        run.outcome(&gpu)
    };
    let run = Run::new(&gpu, &c);
    run.frame(&gpu, 3, true);
    flagged(&run);
    run.step.set_latch_hook(&gpu.queue, Some((3, 4, 1)));
    let r = run.frame(&gpu, FRAME, true);
    let got = run.outcome(&gpu);
    println!("edit flagged, latch at the frame's step 1: {r:?}, box {:?}", got.abox);
    assert_eq!((r.steps_run, r.latch_fired), (1, Some(1)));
    assert_eq!(want.abox.fits, 3, "the reference: the initial fit, step 0's, step 3's P0");
    assert_same(&got, &want, "P0's gate left open after the latch");
}

#[test]
fn poll_never_waits_and_reports_each_frame_once_in_order() {
    let gpu = device();
    let c = constants();
    let dt = dt_of_start(&gpu, &c);
    let run = Run::new(&gpu, &c);
    run.step.set_latch_hook(&gpu.queue, Some((30, 31, 1)));
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("two frames") });
    for _ in 0..2 {
        run.step.encode_frame(&FramePlan { steps: 20, slow_down: true }, &mut encoder);
    }
    assert!(run.step.poll(&gpu.device).is_empty(), "nothing submitted, nothing reported");
    gpu.queue.submit([encoder.finish()]);
    let mut reports = run.step.poll(&gpu.device);
    let early = reports.len();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while reports.len() < 2 {
        assert!(std::time::Instant::now() < deadline, "no report within 60 s: {reports:?}");
        reports.extend(run.step.poll(&gpu.device));
    }
    println!("{early} report(s) on the first poll after submit; {reports:?}");
    assert!(run.step.poll(&gpu.device).is_empty(), "each frame reported once");
    // Δt is the same on every step of this start, so a frame's sim time is Δt added in order, in f32.
    let sum = |n: u32| (0..n).fold(0f32, |t, _| t + dt);
    let want = [(20, None, 20, sum(20)), (11, Some(11), 31, sum(11))];
    let mut sim_time = 0f64;
    for (r, (steps_run, fired, steps, advanced)) in reports.iter().zip(want) {
        sim_time += f64::from(advanced);
        assert_eq!((r.steps_run, r.latch_fired, r.steps), (steps_run, fired, steps));
        assert_eq!((r.advanced.to_bits(), r.dt.to_bits(), r.dt_next.to_bits()), (advanced.to_bits(), dt.to_bits(), dt.to_bits()));
        assert_eq!(r.sim_time, sim_time);
    }
    assert_eq!(run.step.read_dt(&gpu.device, &gpu.queue).steps, 31);
}
