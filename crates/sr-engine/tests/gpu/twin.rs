//! M0-T37 — the twin's step driver and `headless --cpu-reference` (contract §1.3.2, §1.7, §3.3, R13): `sr_physics::reference`'s
//! [`Twin`] runs the passes built so far in §1.3.2's order on f64 planes, the scene's switches honoured, and the headless
//! command runs it from the same scene with the GPU run's summary.json and dumps.
//!
//! The cases: a 64 × 64 sun-like disk (`preset:sun --world 64x64`, every switch on, leave edges) run by
//! `--cpu-reference` and in-process gives the same dumps byte for byte and the in-process sim time; a world over 128 × 128
//! is refused with exit 4 and 128 × 128 runs; the GPU's own flags are refused with it; the driver's pass list — printed
//! beside the GPU step's, for each of the four gravity/hydro switch pairs — is the GPU's; P8 in the driver books exactly
//! what it changes (hydro and gravity off, so the floors alone move the state); and the driver steps the GPU's way — Δt_n
//! from the end of step n − 1, P9's gravity term from step n's own P2 — as an independent hand-written loop does, and
//! visibly not as a loop that takes g fresh from the state before each step (tests/gpu/collapse.rs's).

use std::path::{Path, PathBuf};

use sr_engine::headless::{self, twin_dump};
use sr_engine::scene::{EdgeMode, Scene};
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::{EosGpu, Pass, Passes, Step};
use sr_physics::reference::grav::Gravity;
use sr_physics::reference::hydro::{Edge, Hydro, State};
use sr_physics::reference::{Switches, Twin, MAX_WORLD};
use sr_physics::registry::{Elements, Physics};

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-twin-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn s(p: &Path) -> String {
    p.display().to_string()
}

/// Runs `headless` in-process with `args` as given — no adapter appended: the twin refuses one.
fn headless(args: &[&str]) -> i32 {
    headless::run(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
}

/// A scene's world, constants and canonical f32 planes, as headless reads them.
fn read(scene: &Scene) -> (WorldConfig, Physics, Vec<f32>) {
    let world = scene.world;
    let physics = scene.physics().unwrap();
    let (planes, warnings) = scene.planes(world, &physics, &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    (world, physics, planes)
}

/// The sun-like preset on a `size` × `size` world, at calibration's mass as `headless --scene preset:sun` loads it (M0-T74).
fn sun(size: u32) -> (WorldConfig, Physics, Vec<f32>) {
    read(&Scene::preset("sun", WorldConfig::new(size, size).unwrap()).unwrap().with_embedded_masses())
}

/// scenes/cold_disk.json (M0-T36): the cold collapse, where gravity's term limits Δt.
fn cold_disk() -> (WorldConfig, Physics, Vec<f32>) {
    read(&Scene::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes/cold_disk.json")).unwrap())
}

fn widened(planes: &[f32]) -> Vec<f64> {
    planes.iter().map(|&v| f64::from(v)).collect()
}

fn twin_of((world, physics, planes): (WorldConfig, Physics, Vec<f32>), switches: Switches) -> (WorldConfig, Physics, Twin) {
    let (w, h) = (world.width as usize, world.height as usize);
    let twin = Twin::new(&physics, &elements(), Edge::Leave, switches, None, w, h, widened(&planes)).unwrap();
    (world, physics, twin)
}

const ALL: Switches = Switches { gravity: true, hydro: true, heat: false, reactions: false };

#[test]
fn cpu_reference_run_writes_the_in_process_twins_dumps_and_sim_time() {
    let out = temp("run");
    let args = ["--scene", "preset:sun", "--world", "64x64", "--steps", "24", "--dump-every", "8", "--out", &s(&out), "--cpu-reference"];
    assert_eq!(headless(&args), 0);
    let (world, _, mut twin) = twin_of(sun(64), ALL);
    let mut dumps = Vec::new();
    for step in 1..=24u64 {
        twin.step();
        if step % 8 == 0 {
            dumps.push((step, twin_dump(&twin, world).to_bytes()));
        }
    }
    assert_eq!(dumps.len(), 3);
    for (step, bytes) in &dumps {
        let file = std::fs::read(out.join("dumps").join(format!("s_{step}.bin"))).unwrap_or_else(|e| panic!("s_{step}.bin: {e}"));
        assert!(&file == bytes, "s_{step}.bin differs from the in-process twin's dump ({} bytes against {})", file.len(), bytes.len());
    }
    // The dump is §2.12.3's: it reads back, the planes the twin's in f32, and its sim time the twin's.
    let last = sr_engine::dump::Dump::read(&out.join("dumps").join("s_24.bin")).unwrap();
    assert_eq!((last.step, last.sim_time), (24, twin.sim_time()));
    assert!(last.planes.iter().zip(&twin.state.planes).all(|(a, b)| a.to_bits() == (*b as f32).to_bits()));
    let summary: serde_json::Value = serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["adapter"]["name"], "cpu-reference");
    assert_eq!(summary["steps"], 24);
    assert_eq!(summary["sim_time"].as_f64(), Some(twin.sim_time()));
    assert_eq!(summary["until"]["met"], true);
    assert_eq!(summary["world"]["edge"], "leave");
    println!("SR-TWIN run: 24 steps, sim time {:.6}, 3 dumps byte for byte the in-process twin's", twin.sim_time());
}

#[test]
fn a_world_over_128_is_refused_and_128_runs() {
    assert_eq!(MAX_WORLD, 128);
    for world in ["200x200", "136x64", "64x136"] {
        let out = temp(world);
        let code = headless(&["--scene", "preset:sun", "--world", world, "--steps", "1", "--out", &s(&out), "--cpu-reference"]);
        assert_eq!(code, 4, "--world {world}");
        assert!(!out.join("summary.json").exists(), "--world {world}: a refused run wrote a summary");
    }
    let out = temp("128");
    let code = headless(&["--scene", "preset:sun", "--world", "128x128", "--steps", "2", "--out", &s(&out), "--cpu-reference"]);
    assert_eq!(code, 0, "--world 128x128");
    assert!(out.join("summary.json").exists() && out.join("dumps").join("s_2.bin").exists());
}

#[test]
fn the_gpus_own_flags_are_refused_with_the_twin() {
    for extra in [&["--adapter", "RTX"][..], &["--timing"][..], &["--load-dump", "nothing.bin"][..]] {
        let args = [&["--scene", "preset:sun", "--world", "64x64", "--steps", "1", "--cpu-reference"][..], extra].concat();
        assert_eq!(headless(&args), 4, "{extra:?}");
    }
}

#[test]
fn the_drivers_pass_list_is_the_gpus() {
    let gpu = super::device();
    let (world, physics, planes) = sun(64);
    for (gravity, hydro) in [(true, true), (true, false), (false, true), (false, false)] {
        let switches = Switches { gravity, hydro, heat: false, reactions: false };
        let state = GpuState::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let side = SideFields::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &physics, &elements());
        let passes = Passes { mode: BoxMode::Fit, gravity: gravity.then_some(&side), hydro: hydro.then_some(EdgeMode::Leave) };
        let step = Step::with_passes(&gpu.device, &state, &booking, &eos, &physics, &passes).unwrap();
        let counts = step.dispatch_counts();
        let on_gpu: Vec<&str> = Pass::ORDER.iter().zip(counts).filter(|&(_, n)| n > 0).map(|(p, _)| p.label()).collect();
        let (_, _, mut twin) = twin_of(sun(64), switches);
        assert!(twin.passes_run().is_empty(), "before the first step");
        twin.step();
        println!("SR-TWIN passes (gravity {gravity}, hydro {hydro}):\n  twin driver: {:?}\n  GPU step:    {on_gpu:?}", twin.passes_run());
        assert_eq!(twin.passes_run(), on_gpu, "the driver's passes against the GPU's, gravity {gravity}, hydro {hydro}");
        assert_eq!(Twin::pass_list(switches), on_gpu);
    }
}

#[test]
fn p8_in_the_driver_books_what_it_changes() {
    // Hydro and gravity off: the one step changes the state through P8 alone — on a state whose vacuum is Σ = 0, as a dump
    // made before M0-D101 holds it (a scene now writes the floor state there, which leaves P8 nothing to reset).
    let (world, physics, planes) = sun(64);
    let mut before = widened(&planes);
    let (n, floor) = (world.cells(), f64::from(physics.get("sigma_floor").unwrap() as f32));
    for i in 0..n {
        if before[i] == floor {
            before[i] = 0.0;
            before[3 * n + i] = 0.0;
        }
    }
    let mut twin = Twin::new(&physics, &elements(), Edge::Leave, Switches { gravity: false, hydro: false, heat: false, reactions: false }, None, 64, 64, before.clone()).unwrap();
    twin.step();
    // Each cell's change, then summed: the planes' own sums (~3 × 10³) would drown a change of 10⁻⁶ in rounding.
    let change = |c: usize| (c * n..(c + 1) * n).map(|k| twin.state.planes[k] - before[k]).sum::<f64>();
    let (dm, de) = (change(0), change(3));
    let booked = twin.booked;
    println!("SR-TWIN floors: ΔΣ {dm:e} (vacuum_reset {:e}), ΔE {de:e} (floor_added {:e})", booked.vacuum_reset, booked.floor_added);
    assert!(booked.vacuum_reset < 0.0 && booked.floor_added > 0.0, "the disk's vacuum is reset to the floor: {booked:?}");
    assert!((dm + booked.vacuum_reset).abs() <= 1e-9 * dm.abs(), "Σ rose by {dm:e}, vacuum_reset booked {:e}", booked.vacuum_reset);
    assert!((de - booked.floor_added).abs() <= 1e-9 * de.abs(), "E rose by {de:e}, floor_added booked {:e}", booked.floor_added);
    let floor = f64::from(physics.get("sigma_floor").unwrap() as f32);
    assert_eq!(twin.state.planes[0], floor, "the corner cell is the floor state");
    for i in 0..n {
        let x: f64 = (4..14).map(|c| twin.state.planes[c * n + i]).sum();
        assert!((x - 1.0).abs() <= 1e-12, "cell {i}: ΣX = {x}");
    }
}

/// The GPU's order by hand: Δt_0 after P2 on the initial state; step n runs P2 (g of its own state), P4 with Δt_n, P8, then
/// Δt_{n+1} from the new state with that same g. `fresh` takes g from the state at the start of each step for Δt too —
/// tests/gpu/collapse.rs's twin loop, not the GPU's.
fn by_hand(steps: usize, fresh: bool) -> (State, f64) {
    let (world, physics, planes) = cold_disk();
    let (hydro, floors) = (Hydro::new(&physics, &elements(), Edge::Leave).unwrap(), sr_physics::reference::floors::Floors::new(&physics, &elements()).unwrap());
    let vac = physics.get("sigma_vac").unwrap();
    let (w, h) = (world.width as usize, world.height as usize);
    let mut gravity = Gravity::new(1.0);
    let mut st = State::new(w, h, widened(&planes));
    let mut field = gravity.field(st.plane(0), vac, w, h).unwrap();
    let mut dt = hydro.dt(&st, Some((&field.gx, &field.gy)));
    let mut t = 0.0;
    for _ in 0..steps {
        t += dt;
        field = gravity.field(st.plane(0), vac, w, h).unwrap();
        hydro.step_with(&mut st, dt, Some(&field));
        floors.apply(&mut st);
        if fresh {
            field = gravity.field(st.plane(0), vac, w, h).unwrap();
        }
        dt = hydro.dt(&st, Some((&field.gx, &field.gy)));
    }
    (st, t)
}

#[test]
fn the_driver_steps_the_gpus_way_not_with_a_fresh_g_each_step() {
    const STEPS: usize = 12;
    let (_, _, mut twin) = twin_of(cold_disk(), ALL);
    for _ in 0..STEPS {
        twin.step();
    }
    let (hand, t) = by_hand(STEPS, false);
    assert!(twin.state.planes.iter().zip(&hand.planes).all(|(a, b)| a.to_bits() == b.to_bits()), "the driver against the GPU-order loop");
    assert_eq!(twin.sim_time().to_bits(), t.to_bits());
    let (fresh, tf) = by_hand(STEPS, true);
    let worst = twin.state.planes.iter().zip(&fresh.planes).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
    println!("SR-TWIN order: {STEPS} steps, the driver = the GPU-order loop bit for bit; the fresh-g loop differs by {worst:e} (sim time {:e})", (t - tf).abs());
    assert!(worst > 0.0 || t != tf, "the fresh-g order is indistinguishable from the GPU's here: the case grades nothing");
}
