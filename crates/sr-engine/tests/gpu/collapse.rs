//! M0-T36 — gravity's source in the flow, graded by the cold collapse (contract §1.4.2, §5.1's G-COLL, SM-O2, SM-C2): a
//! cold Maclaurin disk Σc√(1 − r²/a²), Σc = 1, a = 40, at T_floor, heat and reactions off, falls in homologously — every
//! radius scaled by cos²η at t/t_ff = (2/π)(η + ½ sin 2η), t_ff = ½√(a/GΣc) = 3.162 — so the radius holding half the
//! mass reads r₅₀(t)/r₅₀(0) = 0.8368 at 0.5 t_ff and 0.5279 at 0.8 t_ff, each within 3 % (§5.1).
//!
//! The scene is scenes/cold_disk.json: a 128 × 128 world, the disk about its centre, gravity and hydro on, the rest off,
//! and the cold pressure off through `overrides` — its four K's and nothing else (the adversarial line: a collapse that
//! passes because the overrides switched off more than cold pressure). G-COLL's "pressure is negligible" holds only so: at
//! Σ = 1 §2.7's K's give Π_cold ≈ 14 against Π_th = Σ T_floor/μ = 0.1.
//!
//! Both the CPU twin (`sr_physics::reference::hydro`'s `step_with`, g from `reference::grav` over the world each step, Δt
//! with its gravity term, the last step cut to land on the read time) and the GPU's P2 + P4 (`Step::with_passes`, gravity
//! and hydro, on the physics adapter, R12; read at the first step whose sim time reaches the read time, the oracle taken
//! at that time) are graded. r₅₀ is measured about the centre of mass, each non-vacuum cell's mass spread over 4 × 4
//! points of its area, the crossing interpolated between the sorted radii — at t = 0 by the same rule, so the ratio is
//! the grid's own.

use std::f64::consts::{FRAC_PI_2, PI};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use sr_engine::scene::{EdgeMode, Scene};
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::grav::Gravity;
use sr_physics::reference::hydro::{Edge, Hydro, State};
use sr_physics::registry::{Elements, Physics};

/// The disk (§5.1): Σc = 1, a = 40, G = 1 — t_ff = ½√(a/GΣc).
const A: f64 = 40.0;
const SIGMA_C: f64 = 1.0;
/// The read times, in t_ff, and G-COLL's tolerance on the ratio, relative.
const READS: [f64; 2] = [0.5, 0.8];
const TOLERANCE: f64 = 0.03;

fn t_ff() -> f64 {
    0.5 * (A / SIGMA_C).sqrt()
}

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scene() -> Scene {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes");
    Scene::load(&dir.join("cold_disk.json")).unwrap_or_else(|e| panic!("{e}"))
}

fn physics() -> Physics {
    scene().physics().unwrap()
}

fn sigma_vac() -> f64 {
    physics().get("sigma_vac").unwrap()
}

/// The scene's planes as the twin's state, f32 widened to f64.
fn start() -> State {
    let scene = scene();
    let (planes, warnings) = scene.planes(scene.world, &scene.physics().unwrap(), &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    State::new(scene.world.width as usize, scene.world.height as usize, planes.iter().map(|&v| f64::from(v)).collect())
}

/// The homologous answer r₅₀(t)/r₅₀(0) = cos²η at t/t_ff = `tau`, η from (2/π)(η + ½ sin 2η) = tau by bisection.
fn homologous(tau: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, FRAC_PI_2);
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        if (2.0 / PI) * (mid + 0.5 * (2.0 * mid).sin()) < tau {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (0.5 * (lo + hi)).cos().powi(2)
}

/// The radius holding half the mass about the centre of mass: each cell at or above Σ_vac spread over 4 × 4 points of
/// its area (cell (x, y) spans [x, x + 1] × [y, y + 1]), the half-mass crossing interpolated between sorted radii.
fn r50(st: &State) -> f64 {
    const SUB: usize = 4;
    let vac = sigma_vac();
    let w = st.width;
    let cells: Vec<(f64, f64, f64)> = st
        .plane(0)
        .iter()
        .enumerate()
        .filter(|&(_, &s)| s >= vac)
        .map(|(i, &s)| ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5, s))
        .collect();
    let mass: f64 = cells.iter().map(|c| c.2).sum();
    let cx = cells.iter().map(|c| c.0 * c.2).sum::<f64>() / mass;
    let cy = cells.iter().map(|c| c.1 * c.2).sum::<f64>() / mass;
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(cells.len() * SUB * SUB);
    for &(x, y, s) in &cells {
        for a in 0..SUB {
            for b in 0..SUB {
                let px = x - 0.5 + (a as f64 + 0.5) / SUB as f64;
                let py = y - 0.5 + (b as f64 + 0.5) / SUB as f64;
                points.push(((px - cx).hypot(py - cy), s / (SUB * SUB) as f64));
            }
        }
    }
    points.sort_by(|p, q| p.0.total_cmp(&q.0));
    let (mut below, mut r_prev) = (0.0, 0.0);
    for (r, m) in points {
        if below + m >= 0.5 * mass {
            return r_prev + (r - r_prev) * (0.5 * mass - below) / m;
        }
        below += m;
        r_prev = r;
    }
    unreachable!("half the mass lies within the points")
}

/// A run's reads: (sim time, r₅₀(t)/r₅₀(0)) at each of `READS`.
struct Run {
    reads: Vec<(f64, f64)>,
    steps: usize,
    adapter: Option<String>,
}

/// Grades read `k` of `run` against the homologous answer at its own time; prints `SR-COLL`.
fn grade(name: &str, run: &Run, k: usize) {
    let (t, ratio) = run.reads[k];
    let tau = t / t_ff();
    let expected = homologous(tau);
    let err = ratio / expected - 1.0;
    println!(
        "SR-COLL {name}: t = {t:.5} ({tau:.5} t_ff), r50/r50(0) = {ratio:.5}, homologous {expected:.5}, error {:+.3} % \
         ({} steps in the run{})",
        100.0 * err,
        run.steps,
        run.adapter.as_ref().map_or(String::new(), |a| format!("; {a}"))
    );
    assert!(
        (tau - READS[k]).abs() < 0.05,
        "{name}: read at {tau} t_ff, not near {} t_ff (the GPU stops at the first step past it)",
        READS[k]
    );
    assert!(err.abs() <= TOLERANCE, "{name}: r50 ratio {ratio} against {expected} at {tau} t_ff: {:.3} % > 3 %", 100.0 * err);
}

#[test]
fn the_scene_is_a_cold_maclaurin_disk_with_only_the_cold_pressure_off() {
    let scene = scene();
    // Only the four K's are overridden: every other constant is the shipped one (the adversarial line).
    let mut keys: Vec<&str> = scene.overrides.iter().map(|(k, _)| k.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["k1e", "k1n", "k2e", "k2n"], "the overrides");
    let (physics, shipped) = (scene.physics().unwrap(), Physics::shipped().unwrap());
    for key in Physics::keys() {
        let expected = if ["k1e", "k2e", "k1n", "k2n"].contains(&key) { Some(0.0) } else { shipped.get(key) };
        assert_eq!(physics.get(key), expected, "physics key {key}");
    }
    let s = scene.switches;
    assert!(s.gravity && s.hydro && !s.heat && !s.reactions && !s.neutrinos && !s.sinks, "{s:?}");
    // Σ = Σc√(1 − r²/a²) about the world's centre, at rest, hydrogen, E = Σ T_floor/μ (μ = ½: ionised hydrogen).
    let st = start();
    let t_floor = physics.get("t_floor").unwrap();
    let (mut mass, mut cells) = (0.0, 0);
    for y in 0..st.height {
        for x in 0..st.width {
            let i = y * st.width + x;
            let r = (x as f64 + 0.5 - 64.0).hypot(y as f64 + 0.5 - 64.0);
            let sigma = if r < A { SIGMA_C * (1.0 - r * r / (A * A)).sqrt() } else { 0.0 };
            assert!((st.get(0, i) - sigma).abs() <= 1e-6, "Σ at ({x}, {y}): {} against {sigma}", st.get(0, i));
            if sigma > 0.0 {
                assert_eq!((st.get(1, i), st.get(2, i)), (0.0, 0.0), "at rest at ({x}, {y})");
                let e = st.get(0, i) * t_floor * 2.0;
                assert!((st.get(3, i) - e).abs() <= 1e-6 * e, "E at ({x}, {y}): {} against {e}", st.get(3, i));
                assert_eq!(st.get(4, i), 1.0, "X_H at ({x}, {y})");
                mass += st.get(0, i);
                cells += 1;
            }
        }
    }
    let exact = 2.0 * PI * SIGMA_C * A * A / 3.0;
    println!("SR-COLL scene: {cells} cells, mass {mass:.3} (2πΣc a²/3 = {exact:.3}), t_ff {:.4}", t_ff());
    assert!((mass / exact - 1.0).abs() < 2.0 * 2f64.sqrt() * A.powf(-1.5), "the disk's mass {mass} against {exact}");
}

#[test]
fn the_oracle_gives_the_contracts_numbers() {
    // §5.1: 0.8368 at 0.5 t_ff and 0.5279 at 0.8 t_ff, t_ff = 3.162; and every radius at the centre at t_ff. The
    // contract's four digits are cut, not rounded (0.83681, 0.52796), hence 1e-4.
    assert!((homologous(0.5) - 0.8368).abs() < 1e-4, "{}", homologous(0.5));
    assert!((homologous(0.8) - 0.5279).abs() < 1e-4, "{}", homologous(0.8));
    assert!((t_ff() - 3.162).abs() < 5e-4, "{}", t_ff());
    // At t_ff the curve is flat in η (dt/dη ∝ 1 + cos 2η → 0), so f64 pins η there only to ~1e-5: cos²η ~ 1e-11.
    assert!(homologous(1.0) < 1e-9 && (homologous(0.0) - 1.0).abs() < 1e-12, "{} {}", homologous(1.0), homologous(0.0));
}

// ---- the CPU twin ----

/// The twin's run, once for every case that reads it: g from P2's twin over the world each step, Δt per §2.3.5 with its
/// gravity term, the last step to each read time cut to land on it.
fn twin_run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let physics = physics();
        let hydro = Hydro::new(&physics, &elements(), Edge::Leave).unwrap();
        let mut gravity = Gravity::new(1.0);
        let mut st = start();
        let (w, h, vac) = (st.width, st.height, sigma_vac());
        let r0 = r50(&st);
        let (mut t, mut steps, mut reads) = (0.0, 0, Vec::new());
        for tau in READS {
            let until = tau * t_ff();
            while t < until {
                let field = gravity.field(st.plane(0), vac, w, h).unwrap();
                let g = Some((&field.gx[..], &field.gy[..]));
                let dt = hydro.dt(&st, g).min(until - t);
                assert!(dt.is_finite() && dt > 1e-9, "Δt collapsed to {dt} at t = {t} after {steps} steps");
                hydro.step_with(&mut st, dt, Some(&field));
                t += dt;
                steps += 1;
            }
            reads.push((t, r50(&st) / r0));
        }
        Run { reads, steps, adapter: None }
    })
}

#[test]
fn twin_half_mass_radius_at_half_the_free_fall_time() {
    grade("twin", twin_run(), 0);
}

#[test]
fn twin_half_mass_radius_at_0_8_of_the_free_fall_time() {
    grade("twin", twin_run(), 1);
}

// ---- the GPU's P2 + P4 ----

/// The GPU's run, once for every case that reads it: P2 and P4 (leave, the scene's edge), the box fit, steps submitted
/// in batches of 4/5 of the remaining time over the last Δt, the last ones one at a time, each read at the first step
/// whose sim time reaches it.
fn gpu_run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let physics = physics();
        let st = start();
        let gpu = super::device();
        let world = WorldConfig::new(st.width as u32, st.height as u32).unwrap();
        let state = GpuState::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &st.planes.iter().map(|&v| v as f32).collect::<Vec<_>>()).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let side = SideFields::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &physics, &elements());
        let passes = Passes { mode: BoxMode::Fit, gravity: Some(&side), hydro: Some(EdgeMode::Leave) };
        let step = Step::with_passes(&gpu.device, &state, &booking, &eos, &physics, &passes).unwrap();
        let read = || {
            let planes = state.readback(&gpu.device, &gpu.queue).iter().map(|&v| f64::from(v)).collect();
            State::new(st.width, st.height, planes)
        };
        let r0 = r50(&read());
        let mut reads = Vec::new();
        let mut report = step.read_dt(&gpu.device, &gpu.queue);
        for tau in READS {
            let until = tau * t_ff();
            while report.steps == 0 || report.sim_time < until {
                // Before step 0 the record holds no Δt yet: that step runs alone.
                let dt = f64::from(report.dt_next.max(report.dt));
                let batch =
                    if report.steps == 0 { 1 } else { (((until - report.sim_time) / dt * 0.8).floor() as u32).clamp(1, 256) };
                step.run(&gpu.device, &gpu.queue, batch);
                report = step.read_dt(&gpu.device, &gpu.queue);
                assert!(report.non_finite.is_none(), "non-finite state: {:?}", report.non_finite);
                assert!(batch == 1 || report.sim_time < until, "a batch of {batch} steps crossed t = {until}: {report:?}");
            }
            reads.push((report.sim_time, r50(&read()) / r0));
        }
        Run { reads, steps: report.steps as usize, adapter: Some(gpu.info.sr_adapter_line()) }
    })
}

#[test]
fn gpu_half_mass_radius_at_half_the_free_fall_time() {
    grade("gpu", gpu_run(), 0);
}

#[test]
fn gpu_half_mass_radius_at_0_8_of_the_free_fall_time() {
    grade("gpu", gpu_run(), 1);
}
