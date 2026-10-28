//! M0-T26 — state dumps (contract §2.12.3 with §0.5 [M0-T26]'s `resume` and [M0-D33]'s F, §3.1's `load_dump` and
//! `dump_state`, §3.3's `--dump-every` and `--load-dump`, §1.3.4).
//!
//! The oracles: the format is read back by hand from the bytes (§2.12.3's offsets, §2.2's channel order — Σ, mom_x and
//! mom_y identified by the values the start gave them), never through the reader under test; a resumed run is graded
//! against the straight run of the same start, bit for bit — the state, the Δt record, the box and every frame report.
//! The resume case dumps right after the latch fired on the dump's last step, with a second flag two steps later, so a
//! restore that loses the latch's arming time (or the step index) fires where the straight run does not. Every number is
//! exact (bits and integers) but the sim time's own (M0-D21): one value per step count — the same dump bytes whatever
//! the frames, bare steps included — within 10⁻⁹ relative of the f64 sum of each step's f32 Δt in step order, each Δt
//! read from a run of one-step frames or one-step bare runs. A run with P5 (preset:sun) carries F after the channels
//! (M0-D33): its headless resume is graded byte for byte, so a resume whose first step pushes with F = 0 fails it.
//! P5's carry (§0.5 [M0-TJ4] (6), M0-T121): scenes/sun_disk.json's 200 steps with every submission recording 8 of P5's
//! slots — bare, and in frames under the latch — dump the bytes of the run at the engine's own choice.
//! P5's warm start (§0.5 [M0-D85]): the rate is the dump's 17th plane, after F — a sun_disk life cut by a dump and continued
//! is the unbroken one byte for byte 10 steps on, and the same dump without its rate visibly is not.

use super::device;
use sr_engine::dump::{dump_state, load_dump, Dump, Resume, Sink};
use sr_engine::gpu::Gpu;
use sr_engine::scene::{EdgeMode, Scene, TestSettings};
use sr_engine::state::{Booking, SideFields, State, WorldConfig, N_CHANNELS};
use sr_engine::step::boxfit::{BoxMode, BoxReport};
use sr_engine::step::heat::{HeatPrecond, HeatSetup, MIN_ROUNDS};
use sr_engine::step::react::ReactSetup;
use sr_engine::step::dt::DtReport;
use sr_engine::step::latch::{FramePlan, FrameReport};
use sr_engine::step::sinks::SinkSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::registry::{Elements, Physics, Reactions};

/// §2.2's channels, written out from the contract (not from `state::CHANNELS`).
const SPEC_CHANNELS: [&str; 14] =
    ["sigma", "mom_x", "mom_y", "energy", "x_H", "x_He", "x_C", "x_O", "x_Ne", "x_Mg", "x_Si", "x_S", "x_Fe", "x_n"];

struct Constants {
    physics: Physics,
    elements: Elements,
}

/// The shipped constants, the cold pairs off (as tests/gpu/latch.rs).
fn constants() -> Constants {
    let physics = Physics::shipped().unwrap().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).unwrap();
    Constants { physics, elements: Elements::shipped().unwrap() }
}

/// A `w` × `h` world of vacuum hydrogen and a 10 × 10 block of gas on x 40..50, y 20..30 with `block` = (Σ, mom_x,
/// mom_y, E) and the species `x` (§1.5's order).
fn start(w: u32, h: u32, block: [f32; 4], x: [f32; 10]) -> (WorldConfig, Vec<f32>) {
    let world = WorldConfig::new(w, h).unwrap();
    let n = world.cells();
    let mut p = vec![0f32; N_CHANNELS * n];
    p[4 * n..5 * n].fill(1.0);
    for y in 20..30 {
        for xx in 40..50 {
            let i = y * w as usize + xx;
            for (c, v) in block.iter().chain(&x).enumerate() {
                p[c * n + i] = *v;
            }
        }
    }
    (world, p)
}

/// Cold hydrogen at rest: Δt ≈ 0.447 on every step (tests/gpu/latch.rs), so 1.0 t.u. is 3 steps, never 2.
fn cold_start() -> (WorldConfig, Vec<f32>) {
    let mut h = [0f32; 10];
    h[0] = 1.0;
    start(256, 128, [1.0, 0.0, 0.0, 0.0], h)
}

/// Hot, moving gas of ten species: every channel of the block distinct from its neighbours in §2.2's order.
fn mixed_start() -> (WorldConfig, Vec<f32>) {
    let x = [0.40, 0.25, 0.10, 0.08, 0.06, 0.05, 0.03, 0.015, 0.01, 0.005];
    start(128, 64, [1.0, 0.25, -0.5, 2.0], x)
}

struct Run {
    state: State,
    step: Step,
}

impl Run {
    fn new(gpu: &Gpu, c: &Constants, (world, planes): (WorldConfig, Vec<f32>)) -> Run {
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let step = Step::new(&gpu.device, &state, &booking, &eos, &c.physics);
        Run { state, step }
    }

    /// The step headless builds for a scene with every switch on, but for P6, P3 and P7 (P2, P4 since M0-T34, P5 since M0-T42) on
    /// `edge`: preset:sun's first 100 steps burn and sink nothing, so its state is headless's.
    fn as_headless(gpu: &Gpu, c: &Constants, (world, planes): (WorldConfig, Vec<f32>), edge: EdgeMode) -> Run {
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let side = SideFields::new(&gpu.device, world).unwrap();
        let passes = Passes { mode: BoxMode::Fit, gravity: Some(&side), hydro: Some(edge) };
        let heat = HeatSetup { side: &side, test: TestSettings::default() };
        let step = Step::with_heat(&gpu.device, &state, &booking, &eos, &c.physics, &passes, Some(&heat)).unwrap();
        Run { state, step }
    }

    /// The step headless builds for scenes/sun_disk.json but for P3 and P7 (its switches: P2, P4, P5, P6, and since M0-D47 P3 and
    /// P7, which this one leaves out: P5's carry is what the case grades), from the scene's state.
    fn sun_disk(gpu: &Gpu) -> Run {
        let scene = Scene::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes/sun_disk.json")).unwrap();
        let (physics, elements) = (scene.physics().unwrap(), Elements::shipped().unwrap());
        let reactions = Reactions::shipped(&elements, &physics).unwrap();
        let world = scene.world;
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &scene.planes(world, &physics, &elements).unwrap().0).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &physics, &elements);
        let side = SideFields::new(&gpu.device, world).unwrap();
        let on = scene.switches;
        assert!(on.heat, "sun_disk runs P5");
        let passes = Passes { mode: BoxMode::Fit, gravity: on.gravity.then_some(&side), hydro: on.hydro.then_some(scene.edge) };
        let heat = HeatSetup { side: &side, test: scene.test };
        let react = on.reactions.then_some(ReactSetup { reactions: &reactions, test: scene.test, neutrinos: on.neutrinos, side: &side });
        let step = Step::with_reactions(&gpu.device, &state, &booking, &eos, &physics, &passes, Some(&heat), react.as_ref()).unwrap();
        Run { state, step }
    }

    /// One frame with the slow-down on, submitted, waited for, and its report.
    fn frame(&self, gpu: &Gpu, steps: u32) -> FrameReport {
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.step.encode_frame(&FramePlan { steps, slow_down: true }, &mut encoder);
        gpu.queue.submit([encoder.finish()]);
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let reports = self.step.poll(&gpu.device);
        assert_eq!(reports.len(), 1, "one report per frame: {reports:?}");
        reports[0]
    }

    fn outcome(&self, gpu: &Gpu) -> (Vec<u32>, DtReport, BoxReport) {
        let state = self.state.readback(&gpu.device, &gpu.queue).iter().map(|v| v.to_bits()).collect();
        (state, self.step.read_dt(&gpu.device, &gpu.queue), self.step.read_box(&gpu.device, &gpu.queue))
    }
}

/// A small dump built on the CPU, for the refusal cases.
fn small_dump() -> Dump {
    let world = WorldConfig::new(64, 64).unwrap();
    let planes = (0..N_CHANNELS * world.cells()).map(|i| i as f32 * 0.5).collect();
    let resume = Resume {
        dt: 0.447,
        dt_next: 0.448,
        latch_since: 0.25,
        box_record: [8, 16, 24, 32, 64, 64, 17, 3],
        box_args: [3, 4, 1, 3, 1, 1],
    };
    Dump { world, step: 17, sim_time: 7.123456789012345, sinks: Vec::new(), resume, planes, flux: None, rate: None }
}

/// The header's bytes and the planes' offset, read by hand.
fn split(bytes: &[u8]) -> (serde_json::Value, usize) {
    let len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    (serde_json::from_slice(&bytes[12..12 + len]).expect("the header is JSON"), 12 + len)
}

/// Runs `headless` in-process with `args` (and the test adapter); returns the exit code.
fn headless(args: &[&str]) -> i32 {
    let mut args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    sr_engine::headless::run(&args)
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-dump-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn write_then_read_gives_a_bit_identical_state_and_the_bytes_follow_2_12_3() {
    let gpu = device();
    let c = constants();
    let run = Run::new(&gpu, &c, mixed_start());
    // The latch fires after GPU step index 17: the dump carries a fired latch's records, 18 steps in.
    run.step.set_latch_hook(&gpu.queue, Some((17, 18, 1)));
    let r = run.frame(&gpu, 30);
    assert_eq!((r.steps_run, r.latch_fired), (18, Some(18)), "{r:?}");
    let dump = dump_state(&gpu.device, &gpu.queue, &run.state, &run.step);
    let bytes = dump.to_bytes();

    // The bytes, by hand: magic, header length, header, then 14 f32 planes in §2.2's order, row-major.
    assert_eq!(&bytes[..8], b"SRDUMP01");
    let (header, base) = split(&bytes);
    println!("header: {header}");
    let keys: Vec<&str> = header.as_object().unwrap().keys().map(String::as_str).collect();
    let mut want = vec!["version", "width", "height", "step", "sim_time", "channels", "sinks", "resume"];
    want.sort();
    let mut got = keys.clone();
    got.sort();
    assert_eq!(got, want, "the header's keys");
    assert_eq!((header["version"].as_u64(), header["width"].as_u64(), header["height"].as_u64()), (Some(1), Some(128), Some(64)));
    assert_eq!(header["step"].as_u64(), Some(18), "the next step's index");
    assert!(r.sim_time > 0.0 && header["sim_time"].is_number(), "the frames' sim time: {}", header["sim_time"]);
    assert_eq!(header["channels"], serde_json::json!(SPEC_CHANNELS));
    assert_eq!(header["sinks"], serde_json::json!([]));
    assert_eq!(header["resume"]["latch_since"].as_f64(), Some(0.0), "the latch set on the dump's last step");
    assert_eq!((header["resume"]["flux"].as_bool(), dump.flux.is_none()), (Some(false), true), "no P5, no F");
    let (w, cells) = (128usize, 128 * 64);
    assert_eq!(bytes.len(), base + 4 * N_CHANNELS * cells, "nothing past the last plane");
    let at = |ch: usize, x: usize, y: usize| {
        let o = base + 4 * (ch * cells + y * w + x);
        f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap())
    };
    // P8 leaves hot gas's Σ and momentum as they were: the start's values sit at §2.2's channel indices.
    for (x, y) in [(40, 20), (49, 29), (45, 23)] {
        assert_eq!([at(0, x, y), at(1, x, y), at(2, x, y)], [1.0, 0.25, -0.5], "Σ, mom_x, mom_y at cell ({x}, {y})");
    }
    let planes = run.state.readback(&gpu.device, &gpu.queue);
    for ch in 0..N_CHANNELS {
        for (x, y) in [(0, 0), (127, 63), (44, 25), (50, 30)] {
            assert_eq!(at(ch, x, y).to_bits(), planes[ch * cells + y * w + x].to_bits(), "{} at ({x}, {y})", SPEC_CHANNELS[ch]);
        }
    }

    // Read back: the same dump, the same bytes; through a file too.
    let read = Dump::from_bytes(&bytes).expect("its own dump is read");
    assert_eq!(read.sim_time.to_bits(), r.sim_time.to_bits(), "sim_time back bit for bit");
    assert_eq!(read, dump);
    assert_eq!(read.to_bytes(), bytes);
    let dir = temp("roundtrip");
    let path = dir.join("s_18.bin");
    dump.write(&path).unwrap();
    assert_eq!(Dump::read(&path).unwrap().to_bytes(), bytes);
    let _ = std::fs::remove_dir_all(&dir);

    // Loaded into a fresh state and step, dumped again: bit for bit.
    let fresh = Run::new(&gpu, &c, (dump.world, vec![0.0; N_CHANNELS * cells]));
    load_dump(&gpu.queue, &fresh.state, &fresh.step, &read).unwrap();
    let again = dump_state(&gpu.device, &gpu.queue, &fresh.state, &fresh.step);
    assert_eq!(again.to_bytes(), bytes, "load_dump then dump_state");
}

#[test]
fn two_hundred_steps_through_a_dump_equal_two_hundred_straight_steps_state_and_reports() {
    let gpu = device();
    let c = constants();
    let dt = {
        let run = Run::new(&gpu, &c, cold_start());
        run.step.run(&gpu.device, &gpu.queue, 1);
        run.step.read_dt(&gpu.device, &gpu.queue).dt
    };
    assert!(2.0 * dt < 1.0 && 3.0 * dt >= 1.0, "Δt {dt}: a set two steps after another must be inside the window");
    // Flags at GPU step indices 99 and 101: the first fires (armed) on the dump's last step, the second falls 2 Δt later
    // and must not fire — unless the arming time was lost.
    let hook = Some((99, 102, 2));
    let frames = [50u32; 4];

    let straight = Run::new(&gpu, &c, cold_start());
    straight.step.set_latch_hook(&gpu.queue, hook);
    let want: Vec<FrameReport> = frames.iter().map(|&n| straight.frame(&gpu, n)).collect();
    println!("straight: {want:?}");
    assert_eq!((want[1].steps_run, want[1].latch_fired), (50, Some(50)), "fired after index 99");
    assert_eq!((want[2].steps_run, want[2].latch_fired), (50, None), "index 101 inside the window");
    let want_end = straight.outcome(&gpu);
    assert_eq!(want_end.1.steps, 200);

    let first = Run::new(&gpu, &c, cold_start());
    first.step.set_latch_hook(&gpu.queue, hook);
    for (k, &n) in frames[..2].iter().enumerate() {
        assert_eq!(first.frame(&gpu, n), want[k], "frame {k} before the dump");
    }
    let bytes = dump_state(&gpu.device, &gpu.queue, &first.state, &first.step).to_bytes();
    let dump = Dump::from_bytes(&bytes).unwrap();
    assert_eq!(dump.step, 100);

    let resumed = Run::new(&gpu, &c, cold_start());
    load_dump(&gpu.queue, &resumed.state, &resumed.step, &dump).unwrap();
    resumed.step.set_latch_hook(&gpu.queue, hook);
    for (k, &n) in frames.iter().enumerate().skip(2) {
        let got = resumed.frame(&gpu, n);
        println!("resumed frame {k}: {got:?}");
        assert_eq!(got, want[k], "frame {k} after the dump: the whole report");
    }
    let got_end = resumed.outcome(&gpu);
    assert_eq!(got_end.1, want_end.1, "the Δt record");
    assert_eq!(got_end.2, want_end.2, "the box (re-fits counted)");
    let diff = got_end.0.iter().zip(&want_end.0).filter(|(a, b)| a != b).count();
    assert_eq!(diff, 0, "{diff} state words differ from the straight run");

    // The case can see the bug it guards: the cells alone, on a fresh step, run ahead of the straight run.
    let cells_only = Run::new(&gpu, &c, (dump.world, dump.planes.clone()));
    cells_only.step.set_latch_hook(&gpu.queue, hook);
    let wrong = cells_only.frame(&gpu, 50);
    println!("cells only: {wrong:?}");
    assert_ne!(wrong, want[2], "restoring the cells alone must not pass for a resume");
    // And every record but the latch's arming time: the flag at index 101 then fires, 2 steps into the frame.
    let unarmed = Dump { resume: Resume { latch_since: 1.0, ..dump.resume }, ..dump.clone() };
    let lost = Run::new(&gpu, &c, cold_start());
    load_dump(&gpu.queue, &lost.state, &lost.step, &unarmed).unwrap();
    lost.step.set_latch_hook(&gpu.queue, hook);
    let wrong = lost.frame(&gpu, 50);
    println!("arming time lost: {wrong:?}");
    assert_eq!((wrong.steps_run, wrong.latch_fired), (2, Some(2)), "the control fires where the straight run does not");
}

#[test]
fn headless_two_runs_of_100_through_a_dump_write_the_200_step_runs_dumps_byte_for_byte() {
    let gpu = device();
    let root = temp("headless");
    let (a, b, c) = (root.join("straight"), root.join("first"), root.join("resumed"));
    let base = ["--scene", "preset:sun", "--world", "256x256"];
    let s = |p: &std::path::Path| p.display().to_string();
    let run = |extra: &[&str]| headless(&[&base[..], extra].concat());
    assert_eq!(run(&["--steps", "200", "--dump-every", "50", "--out", &s(&a)]), 0);
    assert_eq!(run(&["--steps", "100", "--out", &s(&b)]), 0);
    let from = s(&b.join("dumps").join("s_100.bin"));
    let resumed = ["--scene", "preset:sun", "--load-dump", &from, "--steps", "100", "--dump-every", "50", "--out", &s(&c)];
    assert_eq!(headless(&resumed), 0);

    let list = |d: &std::path::Path| {
        let mut names: Vec<String> =
            std::fs::read_dir(d.join("dumps")).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).filter(|n| n.ends_with(".bin")).collect();
        names.sort();
        names
    };
    assert_eq!(list(&a), ["s_100.bin", "s_150.bin", "s_200.bin", "s_50.bin"], "every 50 steps, the end once");
    assert_eq!(list(&b), ["s_100.bin"], "the final dump, always");
    assert_eq!(list(&c), ["s_150.bin", "s_200.bin"], "step indices count from the scene");
    let read = |d: &std::path::Path, f: &str| std::fs::read(d.join("dumps").join(f)).unwrap();
    assert_eq!(read(&b, "s_100.bin"), read(&a, "s_100.bin"), "100 steps either way");
    for f in ["s_150.bin", "s_200.bin"] {
        assert!(read(&c, f) == read(&a, f), "{f}: the resumed run's dump differs from the straight run's");
    }
    let end = Dump::from_bytes(&read(&c, "s_200.bin")).unwrap();
    assert_eq!((end.step, end.world), (200, WorldConfig::new(256, 256).unwrap()));

    // P5 runs (preset:sun, heat on): F's 2 planes follow the 14 channels, flagged, then P5's rate, flagged (M0-D85), and the
    // light leaving the star is not 0 — else the byte-for-byte resume above could not tell a lost F (M0-D33).
    let bytes = read(&b, "s_100.bin");
    let (header, base) = split(&bytes);
    let cells = 256 * 256;
    assert_eq!(header["resume"]["flux"].as_bool(), Some(true), "P5 runs: the flag");
    assert_eq!(header["resume"]["rate"].as_bool(), Some(true), "P5 runs: the rate's flag");
    assert_eq!(bytes.len(), base + 4 * (N_CHANNELS + 3) * cells, "F's 2 planes after the 14 channels, the rate's 1 after F");
    let f: Vec<f32> = bytes[base + 4 * N_CHANNELS * cells..base + 4 * (N_CHANNELS + 2) * cells]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    let (fx, fy) = f.split_at(cells);
    let lit = |p: &[f32]| p.iter().filter(|v| **v != 0.0).count();
    println!("s_100.bin: F_x non-zero in {} cells, F_y in {}", lit(fx), lit(fy));
    assert!(lit(fx) > 0 && lit(fy) > 0, "F is 0 at step 100: the resume cannot see it lost");
    // A step that cannot take the dump's F, or would start P5 without it, refuses it.
    let c = constants();
    let dump = Dump::from_bytes(&bytes).unwrap();
    let no_p5 = Run::new(&gpu, &c, (dump.world, dump.planes.clone()));
    let err = load_dump(&gpu.queue, &no_p5.state, &no_p5.step, &dump).unwrap_err();
    println!("F into a step without P5: {err}");
    assert!(err.contains("holds F"), "{err}");
    let with_p5 = Run::as_headless(&gpu, &c, (dump.world, dump.planes.clone()), EdgeMode::Leave);
    let err = load_dump(&gpu.queue, &with_p5.state, &with_p5.step, &Dump { flux: None, ..dump.clone() }).unwrap_err();
    println!("no F into a step with P5: {err}");
    assert!(err.contains("holds no F"), "{err}");

    // A --world other than the dump's is refused before any step, as are the flags' misuses.
    let other = root.join("other");
    assert_eq!(headless(&["--scene", "preset:sun", "--world", "128x128", "--load-dump", &from, "--steps", "1", "--out", &s(&other)]), 4);
    assert!(!other.join("summary.json").exists());
    assert_eq!(run(&["--steps", "10", "--dump-every", "5"]), 4, "--dump-every without --out");
    assert_eq!(run(&["--steps", "10", "--dump-every", "0", "--out", &s(&other)]), 4);
    let _ = std::fs::remove_dir_all(&root);
}

/// M0-D21's bound: the sim time within 10⁻⁹ relative of the f64 sum of the steps' f32 Δt, added in step order.
const SIM_TIME_REL: f64 = 1e-9;

/// `t`'s distance from the f64 sum of `dts` in order, relative to that sum.
fn off_the_sum(t: f64, dts: &[f32]) -> f64 {
    let want = dts.iter().fold(0f64, |s, &d| s + f64::from(d));
    (t - want).abs() / want
}

#[test]
fn forty_steps_as_one_two_or_forty_frames_or_bare_give_one_dump_and_one_sim_time() {
    let gpu = device();
    let c = constants();
    // Each step's Δt, from 40 one-step frames: a report's Δt is its frame's last step's.
    let single = Run::new(&gpu, &c, cold_start());
    let reports: Vec<FrameReport> = (0..40).map(|_| single.frame(&gpu, 1)).collect();
    let dts: Vec<f32> = reports.iter().map(|r| r.dt).collect();
    let mut dumps = vec![("40 × 1", reports[39].sim_time, dump_state(&gpu.device, &gpu.queue, &single.state, &single.step))];
    for frames in [&[40u32][..], &[20, 20]] {
        let run = Run::new(&gpu, &c, cold_start());
        let last = frames.iter().map(|&n| run.frame(&gpu, n)).last().unwrap();
        assert_eq!(last.steps, 40);
        dumps.push((if frames.len() == 1 { "1 × 40" } else { "2 × 20" }, last.sim_time, dump_state(&gpu.device, &gpu.queue, &run.state, &run.step)));
    }
    let bare = Run::new(&gpu, &c, cold_start());
    bare.step.run(&gpu.device, &gpu.queue, 40);
    let t = bare.step.read_dt(&gpu.device, &gpu.queue).sim_time;
    dumps.push(("bare", t, dump_state(&gpu.device, &gpu.queue, &bare.state, &bare.step)));

    let bytes = dumps[0].2.to_bytes();
    for (what, reported, dump) in &dumps {
        let off = off_the_sum(dump.sim_time, &dts);
        println!("{what}: sim_time {:.17} (reported {reported:.17}), {off:.3e} off the f64 sum", dump.sim_time);
        assert_eq!(dump.step, 40, "{what}");
        assert_eq!(reported.to_bits(), dump.sim_time.to_bits(), "{what}: the report's sim time is the dump's");
        assert!(off <= SIM_TIME_REL, "{what}: sim_time {} is {off:e} off the f64 sum", dump.sim_time);
        assert!(dump.to_bytes() == bytes, "{what}: the dump's bytes differ from the 40 × 1 frames'");
    }
    // The case can see the bug it guards: the same Δt summed in f32, as a frame's `advanced` is, falls outside the bound.
    let f32_sum = dts.iter().fold(0f32, |s, &d| s + d);
    let off = off_the_sum(f64::from(f32_sum), &dts);
    println!("the f32 running sum: {off:.3e} off");
    assert!(off > SIM_TIME_REL, "the f32 sum {f32_sum} is within the bound: the case cannot tell the two apart");
}

#[test]
fn an_empty_worlds_steps_run_the_clock_on_the_floor_gas() {
    let gpu = device();
    let c = constants();
    // No cell at or above Σ_vac: the box is empty and Δt the floor gas's step (§0.5 [M0-D96]; M0-T21's largest f32
    // froze the clock), the same in bare steps and in a frame, each step's Δt on the clock.
    let run = Run::new(&gpu, &c, start(128, 64, [0.0; 4], [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
    run.step.run(&gpu.device, &gpu.queue, 10);
    let bare = run.step.read_dt(&gpu.device, &gpu.queue);
    let r = run.frame(&gpu, 10);
    println!("bare: {bare:?}; frame: {r:?}");
    let clock = |steps: u32| (0..steps).fold(0f64, |t, _| t + f64::from(bare.dt));
    assert!(bare.dt > 0.0 && bare.dt < f32::MAX, "an empty box's Δt is a step: {}", bare.dt);
    assert_eq!((bare.steps, bare.sim_time.to_bits()), (10, clock(10).to_bits()), "10 bare steps");
    assert_eq!((r.dt.to_bits(), r.steps, r.steps_run, r.sim_time.to_bits()), (bare.dt.to_bits(), 20, 10, clock(20).to_bits()), "then a 10-step frame");
}

#[test]
fn a_headless_run_writes_one_sim_time_in_its_summary_and_its_final_dump() {
    let gpu = device();
    let root = temp("sim-time");
    let world = WorldConfig::new(256, 256).unwrap();
    let args = ["--scene", "preset:sun", "--world", "256x256", "--steps", "100", "--out", &root.display().to_string()];
    assert_eq!(headless(&args), 0);
    // serde_json reads floats best-effort: the summary's sim_time is parsed from its own token (as dump.rs does).
    let summary = std::fs::read_to_string(root.join("summary.json")).unwrap();
    let token = summary.split("\"sim_time\":").nth(1).and_then(|s| s.split([',', '\n']).next()).expect("summary.json's sim_time");
    let end = Dump::read(&root.join("dumps").join("s_100.bin")).unwrap();
    let _ = std::fs::remove_dir_all(&root);

    // The same run in-process, one bare step at a time: each step's Δt, and the state headless ended on.
    let scene = sr_engine::scene::Scene::preset("sun", world).unwrap().with_embedded_masses(); // as headless loads it (M0-T74)
    let c = Constants { physics: scene.physics().unwrap(), elements: Elements::shipped().unwrap() };
    let (planes, _) = scene.planes(world, &c.physics, &c.elements).unwrap();
    let twin = Run::as_headless(&gpu, &c, (world, planes), scene.edge);
    let dts: Vec<f32> = (0..100)
        .map(|_| {
            twin.step.run(&gpu.device, &gpu.queue, 1);
            twin.step.read_dt(&gpu.device, &gpu.queue).dt
        })
        .collect();
    let twin_end = dump_state(&gpu.device, &gpu.queue, &twin.state, &twin.step);
    assert!(twin_end.planes.iter().zip(&end.planes).all(|(a, b)| a.to_bits() == b.to_bits()), "the twin is headless's run");
    let bits = |d: &Dump| d.flux.as_ref().map(|f| f.iter().map(|v| v.to_bits()).collect::<Vec<u32>>());
    assert!(bits(&end).is_some() && bits(&twin_end) == bits(&end), "the final dump's F is side.flux after step 100");

    let t: f64 = token.trim().parse().expect("summary.json's sim_time is a number");
    let off = off_the_sum(end.sim_time, &dts);
    println!("summary {t:.17}, final dump {:.17}, twin {:.17}; {off:.3e} off the f64 sum", end.sim_time, twin_end.sim_time);
    assert_eq!(t.to_bits(), end.sim_time.to_bits(), "summary.json's sim_time is the final dump's");
    assert_eq!(twin_end.sim_time.to_bits(), end.sim_time.to_bits(), "100 steps in one submission or one by one");
    assert!(off <= SIM_TIME_REL, "sim_time {} is {off:e} off the f64 sum", end.sim_time);
}

#[test]
fn a_truncated_dump_is_refused_wherever_it_is_cut() {
    let _gpu = device();
    let bytes = small_dump().to_bytes();
    let (_, base) = split(&bytes);
    assert_eq!(Dump::from_bytes(&bytes).unwrap(), small_dump());
    for cut in [0, 5, 8, 11, 12, base / 2, base - 1, base, base + 3, base + 4 * 64 * 64, bytes.len() - 4, bytes.len() - 1] {
        let err = Dump::from_bytes(&bytes[..cut]).expect_err(&format!("cut at {cut} of {}", bytes.len()));
        println!("cut at {cut}: {err}");
        assert!(err.contains("truncated"), "cut at {cut}: {err}");
    }
    let mut long = bytes.clone();
    long.push(0);
    assert_eq!(Dump::from_bytes(&long).unwrap_err(), "1 bytes past the last plane");

    // Through headless: exit 4, no summary, no dump.
    let root = temp("truncated");
    let file = root.join("cut.bin");
    std::fs::write(&file, &bytes[..bytes.len() - 1]).unwrap();
    let out = root.join("out");
    assert_eq!(headless(&["--scene", "preset:sun", "--load-dump", &file.display().to_string(), "--steps", "1", "--out", &out.display().to_string()]), 4);
    assert!(!out.join("summary.json").exists() && !out.join("dumps").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_wrong_magic_or_a_header_off_the_contract_is_refused() {
    let _gpu = device();
    let bytes = small_dump().to_bytes();
    for magic in [b"SRDUMP02", b"srdump01", b"\x89PNG\r\n\x1a\n", b"\0\0\0\0\0\0\0\0"] {
        let mut bad = bytes.clone();
        bad[..8].copy_from_slice(magic);
        let err = Dump::from_bytes(&bad).unwrap_err();
        println!("magic {magic:?}: {err}");
        assert!(err.starts_with("not a state dump"), "{err}");
    }
    // The header rewritten: each departure from §2.12.3 (with §0.5's resume) is refused, naming it.
    let (header, base) = split(&bytes);
    let with = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut h = header.clone();
        edit(&mut h);
        let text = h.to_string();
        let mut out = b"SRDUMP01".to_vec();
        out.extend((text.len() as u32).to_le_bytes());
        out.extend(text.as_bytes());
        out.extend(&bytes[base..]);
        Dump::from_bytes(&out)
    };
    assert_eq!(with(&|_| {}).unwrap(), small_dump(), "the header rewritten unchanged reads the same");
    let cases: [(&str, &dyn Fn(&mut serde_json::Value)); 10] = [
        ("header.extra: unknown key", &|h| h["extra"] = 1.into()),
        ("header.resume.box_args: missing", &|h| {
            h["resume"].as_object_mut().unwrap().remove("box_args");
        }),
        ("header.resume: missing", &|h| {
            h.as_object_mut().unwrap().remove("resume");
        }),
        ("header.version: 2", &|h| h["version"] = 2.into()),
        ("header.channels", &|h| h["channels"][1] = "mom_y".into()),
        ("header: world 60", &|h| h["width"] = 60.into()),
        ("header.resume.box", &|h| h["resume"]["box"] = serde_json::json!([1, 2, 3])),
        ("header.resume.flux: missing", &|h| {
            h["resume"].as_object_mut().unwrap().remove("flux");
        }),
        ("header.resume.flux: not true or false", &|h| h["resume"]["flux"] = 1.into()),
        ("header.resume.flux: true, but no F planes", &|h| h["resume"]["flux"] = true.into()),
    ];
    for (want, edit) in cases {
        let err = with(edit).unwrap_err();
        println!("{want}: {err}");
        assert!(err.starts_with(want), "expected {want:?}, got {err:?}");
    }
    // F (M0-D33): carried and read back; its flag cleared, or a plane cut, refused.
    let cells = 64 * 64;
    let with_f = Dump { flux: Some((0..2 * cells).map(|i| i as f32 * -0.25).collect()), ..small_dump() };
    let f_bytes = with_f.to_bytes();
    let (f_header, f_base) = split(&f_bytes);
    assert_eq!((f_header["resume"]["flux"].as_bool(), f_bytes.len()), (Some(true), f_base + 4 * (N_CHANNELS + 2) * cells));
    assert_eq!(Dump::from_bytes(&f_bytes).unwrap(), with_f, "a dump with F reads back the same");
    let mut cleared = f_header.clone();
    cleared["resume"]["flux"] = false.into();
    let text = cleared.to_string();
    let mut out = b"SRDUMP01".to_vec();
    out.extend((text.len() as u32).to_le_bytes());
    out.extend(text.as_bytes());
    out.extend(&f_bytes[f_base..]);
    let err = Dump::from_bytes(&out).unwrap_err();
    println!("flag cleared: {err}");
    assert!(err.starts_with("header.resume.flux: false, but F's 2 planes"), "{err}");
    let err = Dump::from_bytes(&f_bytes[..f_bytes.len() - 4 * cells]).unwrap_err();
    println!("F_y cut: {err}");
    assert!(err.contains("truncated"), "{err}");
    // P5's rate (M0-D85): the 17th plane, read back; a header without the key (an older dump) reads none; the flag set with no
    // plane, cleared over one, or set without F, refused.
    let with_r = Dump { rate: Some((0..cells).map(|i| i as f32 * 0.125).collect()), ..with_f.clone() };
    let r_bytes = with_r.to_bytes();
    let (r_header, r_base) = split(&r_bytes);
    assert_eq!((r_header["resume"]["rate"].as_bool(), r_bytes.len()), (Some(true), r_base + 4 * (N_CHANNELS + 3) * cells));
    assert_eq!(Dump::from_bytes(&r_bytes).unwrap(), with_r, "a dump with the rate reads back the same");
    let rewrite = |h: &serde_json::Value, body: &[u8]| {
        let text = h.to_string();
        let mut out = b"SRDUMP01".to_vec();
        out.extend((text.len() as u32).to_le_bytes());
        out.extend(text.as_bytes());
        out.extend(body);
        Dump::from_bytes(&out)
    };
    let mut older = f_header.clone();
    older["resume"].as_object_mut().unwrap().remove("rate");
    assert_eq!(rewrite(&older, &f_bytes[f_base..]).unwrap(), with_f, "a header without `rate` reads no rate");
    let mut set = f_header.clone();
    set["resume"]["rate"] = true.into();
    let mut cleared = r_header.clone();
    cleared["resume"]["rate"] = false.into();
    let mut no_f = header.clone();
    no_f["resume"]["rate"] = true.into();
    for (want, got) in [
        ("header.resume.rate: true, but no rate plane", rewrite(&set, &f_bytes[f_base..])),
        ("header.resume.rate: false, but a plane follows", rewrite(&cleared, &r_bytes[r_base..])),
        ("header.resume.rate: true without F", rewrite(&no_f, &bytes[base..])),
    ] {
        let err = got.unwrap_err();
        println!("{want}: {err}");
        assert!(err.starts_with(want), "expected {want:?}, got {err:?}");
    }

    let root = temp("magic");
    let file = root.join("png.bin");
    let mut bad = bytes.clone();
    bad[..8].copy_from_slice(b"SRDUMP02");
    std::fs::write(&file, bad).unwrap();
    assert_eq!(headless(&["--scene", "preset:sun", "--load-dump", &file.display().to_string(), "--steps", "1"]), 4);
    let _ = std::fs::remove_dir_all(&root);
}

/// §0.5 [M0-TJ4] (6): nothing a step computes depends on where a submission ends. scenes/sun_disk.json (the Sun-like preset
/// in the 600 × 400 world), 200 steps at the engine's own choice of P5's slots, then with every submission recording 8 —
/// the fewest a submission carries — as bare runs and as frames of up to 50 steps under the latch: the three dumps (the
/// cells, F, the Δt record, the box, the latch's arming time) byte for byte. Adversarial: the 8-round runs carry (more
/// submissions than steps; frames that end with a step held), so a carry that drops or repeats a round, or lets a dispatch
/// after it run, shows in the bytes.
#[test]
fn sun_disk_200_steps_carried_over_submissions_of_8_rounds_dump_the_engines_own_choice_byte_for_byte() {
    let gpu = device();
    let engine = Run::sun_disk(&gpu);
    let own = engine.step.run_counted(&gpu.device, &gpu.queue, 200);
    let want = dump_state(&gpu.device, &gpu.queue, &engine.state, &engine.step);
    assert_eq!(want.step, 200);
    let want = want.to_bytes();

    let bare = Run::sun_disk(&gpu);
    bare.step.set_heat_rounds(Some(MIN_ROUNDS));
    assert_eq!(bare.step.heat_rounds(), Some(8), "the contract's smallest carry");
    let eight = bare.step.run_counted(&gpu.device, &gpu.queue, 200);
    let got = dump_state(&gpu.device, &gpu.queue, &bare.state, &bare.step).to_bytes();

    let framed = Run::sun_disk(&gpu);
    framed.step.set_heat_rounds(Some(MIN_ROUNDS));
    let (mut ended, mut frames, mut held) = (0u64, 0u32, 0u32);
    while ended < 200 {
        let r = framed.frame(&gpu, (200 - ended).min(50) as u32);
        assert_eq!(r.latch_fired, None, "{r:?}");
        ended = r.steps - u64::from(r.carried);
        frames += 1;
        held += u32::from(r.carried);
    }
    let in_frames = dump_state(&gpu.device, &gpu.queue, &framed.state, &framed.step).to_bytes();
    println!("sun_disk 200 steps: {own} submission(s) at the engine's choice; {eight} bare submissions of 8 rounds; {frames} frames of 8 rounds, {held} ending with a step held");
    assert!(eight > 200, "8 rounds a submission carried no step: {eight} submissions for 200 steps");
    assert!(held > 0, "no frame ended with a step held");
    assert!(got == want, "the bare run at 8 rounds a submission: its dump's bytes differ from the engine's own choice");
    assert!(in_frames == want, "the frames at 8 rounds a submission: their dump's bytes differ from the engine's own choice");
}

/// §0.5 [M0-D85]: on the V-cycle a step's first Newton iterate is T⁰ + the last step's P5 rate × Δt, so a resume needs the
/// rate. scenes/sun_disk.json, 30 steps, dumped (the rate read by hand as the 17th plane, after F), then 10 more: the dump
/// loaded into a fresh step and run 10 steps writes the unbroken life's bytes. Adversarial: the same dump without its rate
/// (the continued life's first step from T⁰) differs by then.
#[test]
fn a_warm_life_cut_by_a_dump_and_continued_is_the_unbroken_one_byte_for_byte_ten_steps_on() {
    let gpu = device();
    let unbroken = Run::sun_disk(&gpu);
    assert_eq!(unbroken.step.heat_precond(), Some(HeatPrecond::Multigrid), "the warm start is the V-cycle's");
    unbroken.step.run(&gpu.device, &gpu.queue, 30);
    let cut = dump_state(&gpu.device, &gpu.queue, &unbroken.state, &unbroken.step);
    assert_eq!(cut.step, 30);
    let bytes = cut.to_bytes();
    let (header, base) = split(&bytes);
    let cells = cut.world.cells();
    assert_eq!(header["resume"]["rate"].as_bool(), Some(true), "a P5 run dumps its rate");
    assert_eq!(bytes.len(), base + 4 * (N_CHANNELS + 3) * cells, "14 channels, F's 2, the rate's 1");
    let rate = cut.rate.as_ref().expect("the rate read back");
    for k in (0..cells).step_by(997) {
        let o = base + 4 * ((N_CHANNELS + 2) * cells + k);
        assert_eq!(f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()).to_bits(), rate[k].to_bits(), "the 17th plane at cell {k}");
    }
    let moving = rate.iter().filter(|&&r| r != 0.0).count();
    println!("sun_disk step 30: {moving} of {cells} cells with a non-zero P5 rate");
    assert!(moving > 0, "no cell's T moved in P5: the case cannot see a lost rate");
    unbroken.step.run(&gpu.device, &gpu.queue, 10);
    let want = dump_state(&gpu.device, &gpu.queue, &unbroken.state, &unbroken.step).to_bytes();

    let read = Dump::from_bytes(&bytes).unwrap();
    let continued = Run::sun_disk(&gpu);
    load_dump(&gpu.queue, &continued.state, &continued.step, &read).unwrap();
    continued.step.run(&gpu.device, &gpu.queue, 10);
    let got = dump_state(&gpu.device, &gpu.queue, &continued.state, &continued.step).to_bytes();
    assert!(got == want, "the continued life's dump at step 40 differs from the unbroken one's");

    // The case can see the bug it guards: without the rate the continued life's first step starts from T⁰.
    let cold = Run::sun_disk(&gpu);
    load_dump(&gpu.queue, &cold.state, &cold.step, &Dump { rate: None, ..read.clone() }).unwrap();
    cold.step.run(&gpu.device, &gpu.queue, 10);
    let lost = dump_state(&gpu.device, &gpu.queue, &cold.state, &cold.step);
    let off = lost.planes.iter().zip(&Dump::from_bytes(&want).unwrap().planes).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    println!("the same dump without its rate: {off} state values differ at step 40");
    assert!(off > 0, "a resume that drops the rate must not pass for one");
}

/// A world of [`mixed_start`]'s gas with P3's sinks and nothing else on (M0-T52b): the step P3 runs, its sinks set by the
/// test.
fn sinking(gpu: &Gpu, c: &Constants, (world, planes): (WorldConfig, Vec<f32>)) -> Run {
    let state = State::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &planes).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
    let side = SideFields::new(&gpu.device, world).unwrap();
    let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: None };
    let step = Step::with_sinks(&gpu.device, &state, &booking, &eos, &c.physics, &passes, None, None, Some(&SinkSetup { side: &side })).unwrap();
    Run { state, step }
}

/// Each sink's five numbers as bits: the comparison is of the values themselves, not of their printing.
fn sink_bits(sinks: &[Sink]) -> Vec<[u64; 5]> {
    sinks.iter().map(|s| [s.x, s.y, s.vx, s.vy, s.m].map(f64::to_bits)).collect()
}

/// M0-T52b — the sinks in the dump (contract §2.12.3's `sinks`, §3.1's `load_dump`): two sinks, one in the gas and one in
/// the vacuum, after steps that move them, dump with their state bit for bit: the header lists both, the file reads back
/// to the same dump, and a fresh step loaded from it holds the sinks the run held and dumps the same bytes again. A dump
/// that drops the sinks fails the first comparison.
#[test]
fn a_dump_carries_two_sinks_bit_for_bit_through_the_file_and_the_load() {
    let gpu = device();
    let c = constants();
    let (world, planes) = mixed_start();
    let run = sinking(&gpu, &c, (world, planes));
    run.step.set_sinks(&gpu.queue, &[
        Sink { x: 45.5, y: 25.5, vx: 0.125, vy: -0.0625, m: 2.1 },
        Sink { x: 100.25, y: 40.75, vx: 0.0, vy: 0.5, m: 1.75 },
    ]).unwrap();
    run.step.run(&gpu.device, &gpu.queue, 3);
    let before = run.step.read_sinks(&gpu.device, &gpu.queue).unwrap().sinks;
    assert_eq!(before.len(), 2, "two sinks after 3 steps: {before:?}");

    let dump = dump_state(&gpu.device, &gpu.queue, &run.state, &run.step);
    assert_eq!(sink_bits(&dump.sinks), sink_bits(&before), "dump_state reads the sinks the step holds");
    let bytes = dump.to_bytes();
    let (header, _) = split(&bytes);
    assert_eq!(header["sinks"].as_array().map(Vec::len), Some(2), "the header lists both sinks: {}", header["sinks"]);

    // Through the file and back: the same dump, the same bytes, the sinks bit for bit.
    let read = Dump::from_bytes(&bytes).expect("its own dump is read");
    assert_eq!(sink_bits(&read.sinks), sink_bits(&before), "the header's sinks read back bit for bit");
    assert_eq!(read, dump);
    assert_eq!(read.to_bytes(), bytes);
    let dir = temp("sinks");
    let path = dir.join("s_3.bin");
    dump.write(&path).unwrap();
    assert_eq!(Dump::read(&path).unwrap().to_bytes(), bytes);
    let _ = std::fs::remove_dir_all(&dir);

    // Loaded into a fresh step, the sinks are the run's again, and dumped again: the same bytes.
    let fresh = sinking(&gpu, &c, (world, vec![0.0; N_CHANNELS * world.cells()]));
    load_dump(&gpu.queue, &fresh.state, &fresh.step, &read).unwrap();
    let restored = fresh.step.read_sinks(&gpu.device, &gpu.queue).unwrap().sinks;
    assert_eq!(sink_bits(&restored), sink_bits(&before), "load_dump restores the sinks");
    let again = dump_state(&gpu.device, &gpu.queue, &fresh.state, &fresh.step);
    assert_eq!(again.to_bytes(), bytes, "load_dump then dump_state");
}

/// A scene file in `dir`: a rect of hydrogen in a `w` × `h` world with the given `switches` object (§2.12.1), its path.
fn scene_file(dir: &std::path::Path, name: &str, (w, h): (u32, u32), switches: &str) -> String {
    let text = format!(
        r#"{{"version": 1, "world": {{"width": {w}, "height": {h}, "edge": "leave"}}, "switches": {switches},
 "objects": [{{"kind": "rect", "x0": 40, "y0": 20, "w": 10, "h": 10, "sigma": 1.0, "t": 2.0, "composition": {{"H": 1.0}}, "v": [0, 0]}}]}}"#
    );
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.display().to_string()
}

/// The passes `--timing` listed for a run of `scene`: `per_pass_ms`'s keys in summary.json (§2.12.2), and its source.
fn timed_passes(dir: &std::path::Path, scene: &str) -> Vec<String> {
    let out = dir.join(format!("out-{}", dir.read_dir().unwrap().count()));
    assert_eq!(headless(&["--scene", scene, "--steps", "5", "--timing", "--out", &out.display().to_string()]), 0, "{scene}");
    let summary: serde_json::Value = serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
    let timing = &summary["timing"];
    assert_eq!(timing["source"], "gpu-timestamp", "per-pass timing needs the adapter's timestamps: {timing}");
    timing["per_pass_ms"].as_object().unwrap().keys().cloned().collect()
}

/// M0-D47 — headless builds P3 and P7 from the scene's switches (§2.12.1): `sinks` is P3 and `neutrinos` P7, which reads
/// the S_ν plane P6 writes, so with `reactions` off it is not built (the case names that choice). The per-pass timing of
/// five steps lists each pass the step recorded; and a dump holding two sinks (M0-T52b's) loaded into a scene with
/// `sinks` on runs on with them and writes them in its final dump, and into a scene with `sinks` off is refused, exit 4,
/// naming P3. A headless that builds no P3 or P7 fails the timing lists and the resume.
#[test]
fn headless_builds_p3_and_p7_from_the_scenes_switches_and_resumes_a_dump_holding_sinks() {
    let gpu = device();
    let root = temp("switches");
    let (p3, p7) = ("P3 sinks", "P7 neutrino heating");
    let size = (256, 256);
    let listed = |switches: &str, name: &str| timed_passes(&root, &scene_file(&root, name, size, switches));

    let all_on = listed("{}", "on.json");
    assert!(all_on.iter().any(|p| p == p3) && all_on.iter().any(|p| p == p7), "every switch on lists P3 and P7: {all_on:?}");
    let all_off = listed(r#"{"sinks": false, "neutrinos": false}"#, "off.json");
    assert!(!all_off.iter().any(|p| p == p3 || p == p7), "sinks and neutrinos off list neither: {all_off:?}");
    let sinks_only = listed(r#"{"neutrinos": false}"#, "sinks.json");
    assert!(sinks_only.iter().any(|p| p == p3) && !sinks_only.iter().any(|p| p == p7), "sinks alone: P3 only: {sinks_only:?}");
    let nu_only = listed(r#"{"sinks": false}"#, "nu.json");
    assert!(nu_only.iter().any(|p| p == p7) && !nu_only.iter().any(|p| p == p3), "neutrinos alone: P7 only: {nu_only:?}");
    // P7 with no P6 to fill S_ν would run its gate every step over a zero plane: not built.
    let no_p6 = listed(r#"{"reactions": false}"#, "no-p6.json");
    assert!(!no_p6.iter().any(|p| p == p7) && no_p6.iter().any(|p| p == p3), "reactions off: P3, no P7: {no_p6:?}");

    // Two sinks dumped from a P3 step, loaded through headless.
    let c = constants();
    let (world, planes) = mixed_start();
    let run = sinking(&gpu, &c, (world, planes));
    let held = [
        Sink { x: 45.5, y: 25.5, vx: 0.125, vy: -0.0625, m: 2.1 },
        Sink { x: 100.25, y: 40.75, vx: 0.0, vy: 0.5, m: 1.75 },
    ];
    run.step.set_sinks(&gpu.queue, &held).unwrap();
    run.step.run(&gpu.device, &gpu.queue, 3);
    let dump = dump_state(&gpu.device, &gpu.queue, &run.state, &run.step);
    assert_eq!(dump.sinks.len(), 2, "two sinks dumped: {:?}", dump.sinks);
    let file = root.join("s_3.bin");
    dump.write(&file).unwrap();
    let from = file.display().to_string();

    let small = (128, 64);
    let on = scene_file(&root, "resume-on.json", small, r#"{"heat": false}"#);
    let off = scene_file(&root, "resume-off.json", small, r#"{"heat": false, "sinks": false}"#);
    let (out, refused) = (root.join("resumed"), root.join("refused"));
    assert_eq!(headless(&["--scene", &on, "--load-dump", &from, "--steps", "2", "--out", &out.display().to_string()]), 0);
    let end = Dump::read(&out.join("dumps").join("s_5.bin")).unwrap();
    assert_eq!(end.sinks.len(), 2, "the final dump holds the two sinks the run carried: {:?}", end.sinks);
    assert!(end.sinks.iter().zip(&dump.sinks).all(|(a, b)| a.m.to_bits() != 0 && (a.x - b.x).abs() < 1.0), "the sinks ran on");
    println!("sinks after 2 more steps: {:?}", end.sinks);
    assert_eq!(headless(&["--scene", &off, "--load-dump", &from, "--steps", "2", "--out", &refused.display().to_string()]), 4);
    assert!(!refused.join("summary.json").exists(), "refused before any step: no summary");
    // The refusal names P3 (load_dump's message, read in-process).
    let no_p3 = Run::new(&gpu, &c, (world, vec![0.0; N_CHANNELS * world.cells()]));
    let err = load_dump(&gpu.queue, &no_p3.state, &no_p3.step, &dump).unwrap_err();
    assert!(err.contains("P3"), "{err}");
    let _ = std::fs::remove_dir_all(&root);
}
