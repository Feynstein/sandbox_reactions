//! M0-T23 — GPU timestamps, the cost per step and `headless --timing` (contract §1.8.3's c_step, §2.12.2's `timing`,
//! §3.1's `poll()`, §3.3, §6.2.2).
//!
//! The oracle is the step's own pass list: every pass a step records (`Step::dispatch_counts`, P0 always) is timed and
//! positive, no other pass is named, and their sum is within 40 % of the step's cost — the step spans its first pass's
//! beginning to its last pass's end, so the rest is the gaps between passes (TW's submission-overhead note,
//! reports/time_warp.md: timestamps and wall times differ by up to 40 %). The source is named: GPU timestamps exactly
//! when the device has `timestamp-query`, which it takes only where the adapter offers it, and no other feature
//! (§6.2.2: pass boundaries only — TIMESTAMP_QUERY_INSIDE_PASSES is native-only). No number here is a timing target:
//! costs are printed for the log, never graded against a speed (G-FPS, G-TOP are the Quadro's).

use super::device;
use sr_engine::gpu::Gpu;
use sr_engine::headless::sun_disk_planes;
use sr_engine::scene::{EdgeMode, TestSettings};
use sr_engine::state::{Booking, SideFields, State, WorldConfig};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::HeatSetup;
use sr_engine::step::latch::{FramePlan, FrameReport};
use sr_engine::step::nuheat::NuHeatSetup;
use sr_engine::step::react::ReactSetup;
use sr_engine::step::sinks::SinkSetup;
use sr_engine::step::{CostSource, EosGpu, Pass, Passes, Step, StepCost, Timing};
use sr_physics::registry::{Elements, Physics, Reactions};

/// The block's tolerance: the passes' sum within 40 % of the step's cost.
const SUM_WITHIN: f64 = 0.40;

/// A step on the nominal Sun-like disk (§2.1, §2.10) in a `w` × `h` world, the shipped constants, gravity on as a
/// scene's default switches have it (headless, M0-T31).
fn disk(gpu: &Gpu, w: u32, h: u32) -> Step {
    let world = WorldConfig::new(w, h).unwrap();
    let state = State::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &sun_disk_planes(world)).unwrap();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let booking = Booking::new(&gpu.device, world).unwrap();
    let (eos, side) = (EosGpu::new(&gpu.device, &physics, &elements), SideFields::new(&gpu.device, world).unwrap());
    // The passes headless records for `preset:sun` (every switch on, a *leave* world): P2 and, since M0-T34, P4; P5 since
    // M0-T42; P6 since M0-T46; P3 and P7 since M0-D47.
    let passes = Passes { mode: BoxMode::Fit, gravity: Some(&side), hydro: Some(EdgeMode::Leave) };
    let heat = HeatSetup { side: &side, test: TestSettings::default() };
    let reactions = Reactions::shipped(&elements, &physics).unwrap();
    let react = ReactSetup { reactions: &reactions, test: TestSettings::default(), neutrinos: true, side: &side };
    let (sinks, nuheat) = (SinkSetup { side: &side }, NuHeatSetup { side: &side });
    Step::with_neutrino_heating(&gpu.device, &state, &booking, &eos, &physics, &passes, Some(&heat), Some(&react), Some(&sinks), Some(&nuheat))
        .unwrap()
}

/// The source `Timing::Auto` must pick on this device, after checking the device holds no feature but timestamp-query
/// and that one only where the adapter offers it.
fn expected_source(gpu: &Gpu) -> CostSource {
    let offered = gpu.adapter.features() & wgpu::Features::TIMESTAMP_QUERY;
    assert_eq!(gpu.device.features(), offered, "the device's features: timestamp-query where offered, nothing else");
    if offered.is_empty() {
        CostSource::Wall
    } else {
        CostSource::GpuTimestamp
    }
}

/// The passes a step records (no test hook): P0 always, every other pass with a dispatch.
fn recorded(step: &Step) -> Vec<Pass> {
    let counts = step.dispatch_counts();
    Pass::ORDER.iter().copied().filter(|&p| p == Pass::Edits || counts[p as usize] > 0).collect()
}

/// A GPU-timed cost: `steps` steps, every recorded pass positive and finite, no other pass, the sum within 40 %.
fn check_timed(cost: &StepCost, passes: &[Pass], steps: u32, what: &str) {
    println!("{what}: {cost:?}");
    assert_eq!(cost.source, CostSource::GpuTimestamp, "{what}: the source");
    assert_eq!(cost.steps, steps, "{what}: steps timed");
    assert!(cost.step_ms.is_finite() && cost.step_ms > 0.0, "{what}: step_ms {}", cost.step_ms);
    let mut sum = 0.0;
    for p in Pass::ORDER {
        match (passes.contains(&p), cost.pass_ms[p as usize]) {
            (true, Some(ms)) => {
                assert!(ms.is_finite() && ms > 0.0, "{what}: {} timed at {ms} ms", p.label());
                sum += ms;
            }
            (true, None) => panic!("{what}: {} is recorded but not timed", p.label()),
            (false, Some(ms)) => panic!("{what}: {} is not recorded but timed at {ms} ms", p.label()),
            (false, None) => {}
        }
    }
    let off = (sum - cost.step_ms).abs() / cost.step_ms;
    println!("{what}: passes sum {sum:.6} ms, step {:.6} ms, off by {:.1} %", cost.step_ms, 100.0 * off);
    assert!(off <= SUM_WITHIN, "{what}: the passes' sum {sum} ms is {:.1} % off the step's {} ms", 100.0 * off, cost.step_ms);
}

/// One frame, submitted, waited for, and its report.
fn frame(gpu: &Gpu, step: &Step, steps: u32) -> FrameReport {
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
    step.encode_frame(&FramePlan { steps, slow_down: true }, &mut encoder);
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let reports = step.poll(&gpu.device);
    assert_eq!(reports.len(), 1, "one report per frame: {reports:?}");
    reports[0]
}

#[test]
fn bare_steps_time_every_pass_at_its_boundaries() {
    let gpu = device();
    let step = disk(&gpu, 600, 400);
    let source = expected_source(&gpu);
    assert_eq!(step.set_timing(&gpu.device, &gpu.queue, Timing::Auto), Some(source));
    if source == CostSource::Wall {
        panic!("{}: no timestamp-query — this box's physics adapter must offer it", gpu.info.name);
    }
    let passes = recorded(&step);
    println!("recorded passes: {:?}", passes.iter().map(|p| p.label()).collect::<Vec<_>>());
    // Step 0's batch holds the initial reduction (timed as P9), the guard and a re-fit; the second batch neither.
    step.run(&gpu.device, &gpu.queue, 64);
    check_timed(&step.read_cost(&gpu.device, &gpu.queue).expect("a cost"), &passes, 64, "steps 0–63");
    step.run(&gpu.device, &gpu.queue, 15);
    check_timed(&step.read_cost(&gpu.device, &gpu.queue).expect("a cost"), &passes, 15, "steps 64–78");
    assert_eq!(step.read_cost(&gpu.device, &gpu.queue), None, "no step since the last read, no cost");
}

#[test]
fn frames_report_their_cost_per_step_and_per_pass() {
    let gpu = device();
    let step = disk(&gpu, 600, 400);
    assert_eq!(frame(&gpu, &step, 8).cost, None, "timing off: no cost in the report");
    assert_eq!(step.set_timing(&gpu.device, &gpu.queue, Timing::Auto), Some(CostSource::GpuTimestamp));
    let passes = recorded(&step);
    for k in 0..3 {
        let r = frame(&gpu, &step, 40);
        assert_eq!((r.steps_run, r.latch_fired), (40, None));
        let cost = r.cost.expect("a timed frame's cost");
        check_timed(&cost, &passes, 40, &format!("frame {k}"));
        let frame_ms = cost.frame_ms.expect("the frame's own passes timed");
        assert!(frame_ms.is_finite() && frame_ms > 0.0, "frame begin + end: {frame_ms} ms");
    }
    // Two frames in flight: each report carries its own frame's cost, in order.
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frames") });
    step.encode_frame(&FramePlan { steps: 5, slow_down: true }, &mut encoder);
    step.encode_frame(&FramePlan { steps: 12, slow_down: true }, &mut encoder);
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let reports = step.poll(&gpu.device);
    let timed: Vec<u32> = reports.iter().map(|r| r.cost.expect("a cost").steps).collect();
    assert_eq!(timed, [5, 12], "each frame's own steps: {reports:?}");
}

#[test]
fn a_frame_longer_than_the_query_set_times_its_first_whole_steps() {
    let gpu = device();
    let step = disk(&gpu, 128, 128);
    // Rounds fixed so no step carries over (§0.5 [M0-TJ4] (6), M0-T121), and few enough that the frame's one submission
    // stays far inside Windows' 2 s GPU timeout (M0-D39): at SLOTS a step's P5 is 9.8 ms on the RTX 5090, ~2.3 µs a
    // recorded round, so 1000 steps were ~10 s and the RTX 4080 Laptop lost its device. The fresh disk's steps 0–335 use at
    // most 83 slots (its front from step 336 up to 1,627, RTX 5090, M0-D39), so FRAME steps at ROUNDS never carry.
    const FRAME: u32 = 300;
    const ROUNDS: usize = 256;
    step.set_heat_rounds(Some(ROUNDS));
    assert_eq!(step.set_timing(&gpu.device, &gpu.queue, Timing::Auto), Some(CostSource::GpuTimestamp));
    let passes = recorded(&step);
    // 4096 queries, two per pass: frame begin, step 0's five passes, then four per step, 24 kept free and frame end's 2.
    let per_step = 2 * passes.len() as u32;
    let fits = (wgpu::QUERY_SET_MAX_QUERIES - 2 - 2 * (passes.len() as u32 + 1) - 24 - 2) / per_step + 2;
    assert!(fits + 20 < FRAME, "the frame must outrun the query set: {FRAME} steps, ≈ {fits} fit");
    let r = frame(&gpu, &step, FRAME);
    assert_eq!(r.steps_run, FRAME);
    let cost = r.cost.expect("a cost");
    assert!(cost.steps < FRAME && cost.steps + 2 >= fits && cost.steps <= fits + 2, "{} steps timed, ≈ {fits} fit", cost.steps);
    check_timed(&cost, &passes, cost.steps, &format!("a {FRAME}-step frame"));
    assert!(cost.frame_ms.is_some_and(|ms| ms > 0.0), "frame end still timed: {cost:?}");
    let next = frame(&gpu, &step, 10).cost.expect("a cost");
    check_timed(&next, &passes, 10, "the next frame starts a fresh query set");
}

#[test]
fn the_wall_clock_fallback_is_named_and_times_no_pass() {
    let gpu = device();
    let step = disk(&gpu, 128, 128);
    assert_eq!(step.set_timing(&gpu.device, &gpu.queue, Timing::Wall), Some(CostSource::Wall));
    step.run(&gpu.device, &gpu.queue, 20);
    let bare = step.read_cost(&gpu.device, &gpu.queue).expect("a wall cost");
    println!("bare, wall: {bare:?}");
    assert_eq!((bare.source, bare.steps, bare.pass_ms, bare.frame_ms), (CostSource::Wall, 20, [None; 10], None));
    assert!(bare.step_ms.is_finite() && bare.step_ms > 0.0, "{bare:?}");
    let framed = frame(&gpu, &step, 20).cost.expect("a wall cost");
    println!("frame, wall: {framed:?}");
    assert_eq!((framed.source, framed.steps, framed.pass_ms, framed.frame_ms), (CostSource::Wall, 20, [None; 10], None));
    assert!(framed.step_ms.is_finite() && framed.step_ms > 0.0, "{framed:?}");
    assert_eq!(step.set_timing(&gpu.device, &gpu.queue, Timing::Off), None);
    step.run(&gpu.device, &gpu.queue, 3);
    assert_eq!(step.read_cost(&gpu.device, &gpu.queue), None, "timing off: no cost");
}

/// Runs `headless` in-process — the function sr-app's `sandbox-reactions headless` forwards to — and returns its exit
/// code and summary.json.
fn headless(out: &std::path::Path, extra: &[&str]) -> (i32, serde_json::Value) {
    let mut args: Vec<String> = ["--scene", "preset:sun", "--steps", "300", "--out"].map(String::from).to_vec();
    args.push(out.display().to_string());
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    args.extend(extra.iter().map(|s| s.to_string()));
    let code = sr_engine::headless::run(&args);
    let text = std::fs::read_to_string(out.join("summary.json")).expect("summary.json written");
    (code, serde_json::from_str(&text).expect("summary.json parses"))
}

#[test]
fn headless_timing_writes_every_pass_of_the_nominal_disk() {
    let gpu = device();
    let passes = recorded(&disk(&gpu, 600, 400));
    let root = std::env::temp_dir().join(format!("sr-timing-{}", std::process::id()));
    let (code, plain) = headless(&root.join("plain"), &[]);
    assert_eq!(code, 0);
    assert!(plain.get("timing").is_none(), "timing only with --timing (§2.12.2): {plain}");

    let (code, summary) = headless(&root.join("timed"), &["--timing"]);
    assert_eq!(code, 0);
    let timing = &summary["timing"];
    println!("summary.json timing: {timing}");
    assert_eq!(timing["source"], "gpu-timestamp", "{timing}");
    let step_ms = timing["step_ms_mean"].as_f64().expect("step_ms_mean a number");
    assert!(step_ms.is_finite() && step_ms > 0.0, "{timing}");
    let per_pass = timing["per_pass_ms"].as_object().expect("per_pass_ms an object");
    let want: Vec<&str> = passes.iter().map(|p| p.label()).collect();
    let mut got: Vec<&str> = per_pass.keys().map(String::as_str).collect();
    got.sort_by_key(|k| want.iter().position(|w| w == k));
    assert_eq!(got, want, "every recorded pass in per_pass_ms, and no other: {timing}");
    let mut sum = 0.0;
    for (label, ms) in per_pass {
        let ms = ms.as_f64().unwrap_or_else(|| panic!("{label}: not a number"));
        assert!(ms.is_finite() && ms > 0.0, "{label}: {ms} ms");
        sum += ms;
    }
    let off = (sum - step_ms).abs() / step_ms;
    println!("headless: passes sum {sum:.6} ms, step {step_ms:.6} ms, off by {:.1} %", 100.0 * off);
    assert!(off <= SUM_WITHIN, "the passes' sum {sum} ms is {:.1} % off the step's {step_ms} ms", 100.0 * off);
    let _ = std::fs::remove_dir_all(&root);
}
