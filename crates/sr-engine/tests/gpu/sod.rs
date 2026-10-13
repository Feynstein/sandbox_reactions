//! M0-T33b — gas flow on the CPU twin (contract §2.3.4, §2.8's bounce, §5.1's G-SOD): Sod's tube (SM-E14: ρ 1 | 0.125,
//! P 1 | 0.1, at rest) on a 400 × 4 strip with bounce walls, γ = 2, cold pressure off through the scene's `overrides`, read at
//! t = 80 against the exact Riemann solution of M0-T33a (`sr_physics::reference::riemann`, computed here, never read from
//! the scheme): the shock within 1 cell, the L1 density error ≤ 2 % (SM-O5). The twin is `sr_physics::reference::hydro`.
//!
//! The strip is cut from scenes/sod.json, the file M0-T34's GPU case runs: §2.8 keeps a world at 64 cells or more on each
//! axis, so the file's world is 400 × 64 with every row the same gas, and the twin's 400 × 4 strip is its first four
//! rows. Beside the oracle cases, the twin's own symmetries (a strip laid along y, the mirrored tube, rows of a taller
//! world) and its conservation (mass, energy, and the momentum the walls give) grade what the 1D oracle cannot.
//!
//! M0-T34 — the same oracle on the GPU's P4 (`sr_engine::step::hydro`, shaders/hydro.wgsl), on the physics adapter (R12,
//! R13): the scene's 400 × 64 world run, hydro only, to the first step whose sim time reaches 80 (the GPU's Δt is never
//! cut to land on it), graded at that time against the exact solution — the shock within 1 cell and the L1 density error
//! ≤ 2 % on every row; the rows the same gas (a y-flux defect shows as a row that differs); mass, energy and the walls'
//! momentum in f32's reach; the tube laid along y (64 × 400) the x run, step for step; the cold pressure on, conserving.
//! Tolerances of the f32 cases, fixed before any GPU run: rows and the y-laid run within 1e-5 of Σ's unit, mass and
//! energy within 1e-5 relative, the walls' momentum within 1e-4 relative.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use sr_engine::gpu::Gpu;
use sr_engine::scene::{EdgeMode, Scene};
use sr_engine::state::{Booking, State as GpuState, WorldConfig};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::hydro::{Edge, Hydro, State, COMPONENTS};
use sr_physics::reference::riemann::{self, Solution, Wave};
use sr_physics::registry::{Elements, Physics};

/// Sod's read time (the 0.2 of the unit tube, scaled to 400 cells), and where the diaphragm sits (§5.1).
const T_END: f64 = 80.0;
const X0: f64 = 200.0;

fn scenes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn sod_scene() -> Scene {
    Scene::load(&scenes_dir().join("sod.json")).unwrap_or_else(|e| panic!("{e}"))
}

/// The scene's planes as the twin's state, f32 widened to f64.
fn world_of(scene: &Scene, physics: &Physics) -> State {
    let (planes, warnings) = scene.planes(scene.world, physics, &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    State::new(scene.world.width as usize, scene.world.height as usize, planes.iter().map(|&v| f64::from(v)).collect())
}

/// The twin of a scene's constants, bounce walls.
fn twin_of(physics: &Physics) -> Hydro {
    Hydro::new(physics, &elements(), Edge::Bounce).unwrap()
}

/// The scene's first four rows: G-SOD's 400 × 4 strip.
fn strip() -> State {
    let scene = sod_scene();
    world_of(&scene, &scene.physics().unwrap()).rows(0, 4)
}

fn twin() -> Hydro {
    twin_of(&sod_scene().physics().unwrap())
}

/// The strip run to t = 80, once for every case that reads it.
fn run_80() -> &'static State {
    static RUN: OnceLock<State> = OnceLock::new();
    RUN.get_or_init(|| {
        let mut st = strip();
        let steps = twin().advance(&mut st, 0.0, T_END);
        println!("SR-SOD-RUN twin 400x4 strip: {steps} steps to t = {T_END}");
        st
    })
}

fn exact() -> Solution {
    Solution::solve(riemann::State::new(1.0, 0.0, 1.0), riemann::State::new(0.125, 0.0, 0.1)).unwrap()
}

/// The exact density averaged over cell `i` (it spans [i, i + 1], the diaphragm on the face between cells 199 and 200),
/// by 400 midpoints — a cell holding a jump is graded by its average, as a finite volume holds it.
fn exact_cell_density(sol: &Solution, i: usize, t: f64) -> f64 {
    const N: usize = 400;
    (0..N).map(|k| sol.at(i as f64 + (k as f64 + 0.5) / N as f64, X0, t).rho).sum::<f64>() / N as f64
}

fn row0_density(st: &State) -> Vec<f64> {
    st.plane(0)[..st.width].to_vec()
}

/// The largest |a − b| over two equal-length slices.
fn max_diff(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

#[test]
fn the_scene_is_sods_tube() {
    let scene = sod_scene();
    let physics = scene.physics().unwrap();
    for key in ["k1e", "k2e", "k1n", "k2n"] {
        assert_eq!(physics.get(key), Some(0.0), "cold pressure off: {key}");
    }
    assert_eq!(scene.edge, sr_engine::scene::EdgeMode::Bounce);
    let s = scene.switches;
    assert!(s.hydro && !s.gravity && !s.heat && !s.reactions && !s.neutrinos && !s.sinks, "{s:?}");
    let world = world_of(&scene, &physics);
    assert_eq!((world.width, world.height), (400, 64));
    // Every cell at rest, pure hydrogen; Π = E (γ = 2, at rest, cold off) is 1 on the left 200 columns and 0.1 on the right.
    for y in 0..world.height {
        for x in 0..world.width {
            let i = y * world.width + x;
            let (sigma, pi) = if x < 200 { (1.0, 1.0) } else { (0.125, 0.1) };
            assert_eq!(world.get(0, i), sigma, "Σ at ({x}, {y})");
            assert_eq!((world.get(1, i), world.get(2, i)), (0.0, 0.0), "at rest at ({x}, {y})");
            assert!((world.get(3, i) - pi).abs() < 1e-7 * pi, "E = Π at ({x}, {y}): {}", world.get(3, i));
            assert_eq!(world.get(4, i), 1.0, "X_H at ({x}, {y})");
        }
    }
}

/// The shock in a row's density at time `t`: (numerical, exact) positions — where the density crosses halfway between
/// the post-shock plateau and the undisturbed gas, found from the right, interpolated between cell centres (cell i's
/// centre is at i + ½).
fn shock_at(rho: &[f64], t: f64) -> (f64, f64) {
    let sol = exact();
    let Wave::Shock { speed } = sol.right_wave else { panic!("Sod's right wave is a shock") };
    let mid = 0.5 * (sol.rho_star_right + sol.right.rho);
    let i = (0..rho.len()).rev().find(|&i| rho[i] >= mid).expect("the density never reaches the plateau");
    ((i as f64 + 0.5) + (rho[i] - mid) / (rho[i] - rho[i + 1]), X0 + speed * t)
}

/// A row's L1 density error at time `t`, Σ|ρ − ρ_exact| / Σρ_exact, against the exact density averaged over each cell.
fn l1_at(rho: &[f64], t: f64) -> f64 {
    let sol = exact();
    let (mut err, mut norm) = (0.0, 0.0);
    for (i, &r) in rho.iter().enumerate() {
        let e = exact_cell_density(&sol, i, t);
        err += (r - e).abs();
        norm += e;
    }
    err / norm
}

#[test]
fn the_shock_is_within_one_cell() {
    let (x_num, x_exact) = shock_at(&row0_density(run_80()), T_END);
    println!(
        "SR-SOD-SHOCK twin: numerical {x_num:.3}, exact {x_exact:.3}, off {:.3} cells ≤ 1",
        (x_num - x_exact).abs()
    );
    assert!((x_num - x_exact).abs() <= 1.0, "shock at {x_num}, exact {x_exact}");
}

#[test]
fn the_l1_density_error_is_within_two_percent() {
    let l1 = l1_at(&row0_density(run_80()), T_END);
    println!("SR-SOD-L1 twin: Σ|ρ − ρ_exact| / Σρ_exact = {:.4} % ≤ 2 %", 100.0 * l1);
    assert!(l1 <= 0.02, "L1 density error {:.4} %", 100.0 * l1);
}

#[test]
fn mass_energy_and_the_walls_momentum_are_exact() {
    let (start, end) = (strip().totals(), run_80().totals());
    let rel = |a: f64, b: f64| (a - b).abs() / b.abs();
    println!("SR-SOD-CONS twin: mass {:.15e} → {:.15e}, energy {:.15e} → {:.15e}", start[0], end[0], start[3], end[3]);
    assert!(rel(end[0], start[0]) < 1e-12, "mass");
    assert!(rel(end[3], start[3]) < 1e-12, "energy: a wall passes none");
    // The only force on the gas is the walls' pressure — the first cell's on the left, the last cell's on the right, over
    // 4 rows for t = 80, while no wave has reached either wall. At rest with the cold terms off, Π = E; the planes hold
    // f32, so the pressures are the file's 1 and 0.1 as f32 holds them.
    let first = strip();
    let (p_left, p_right) = (first.get(3, 0), first.get(3, first.width - 1));
    let pushed = (p_left - p_right) * T_END * 4.0;
    println!("SR-SOD-CONS twin: momentum {:.10} against the walls' (P_L − P_R) · t · 4 = {pushed:.10}", end[1]);
    assert!(rel(end[1], pushed) < 1e-10, "x momentum {} ≠ {pushed}", end[1]);
    assert!(end[2].abs() < 1e-12, "y momentum {}", end[2]);
    assert!(rel(end[4], start[4]) < 1e-12, "hydrogen mass");
    for s in 5..COMPONENTS {
        assert_eq!(end[s], 0.0, "species {} stays absent", s - 4);
    }
}

#[test]
fn the_tall_scene_world_runs_as_its_strip() {
    // §2.8 keeps a world to 64 cells a side: the scene's 400 × 64 world, run 10 t.u., has every row the strip's.
    let scene = sod_scene();
    let physics = scene.physics().unwrap();
    let hydro = twin_of(&physics);
    let (mut tall, mut thin) = (world_of(&scene, &physics), strip());
    let n_tall = hydro.advance(&mut tall, 0.0, 10.0);
    let n_thin = hydro.advance(&mut thin, 0.0, 10.0);
    assert_eq!(n_tall, n_thin, "the same Δt sequence");
    let w = tall.width;
    let mut worst = 0.0f64;
    for c in 0..COMPONENTS {
        for y in 0..tall.height {
            let row = &tall.plane(c)[y * w..(y + 1) * w];
            worst = worst.max(max_diff(row, &thin.plane(c)[..w]));
        }
    }
    println!("SR-SOD-ROWS twin: 64 rows over {n_tall} steps, every row within {worst:.2e} of the strip's");
    assert!(worst < 1e-13, "rows differ by {worst:e}");
}

/// The state with x reversed and Σu negated: the same problem seen in a mirror.
fn mirrored(st: &State) -> State {
    let (w, h) = (st.width, st.height);
    let mut out = st.clone();
    for c in 0..COMPONENTS {
        for y in 0..h {
            for x in 0..w {
                let v = st.planes[c * w * h + y * w + (w - 1 - x)];
                out.planes[c * w * h + y * w + x] = if c == 1 { -v } else { v };
            }
        }
    }
    out
}

#[test]
fn the_mirrored_tube_gives_the_mirrored_run() {
    // A scheme that treats left and right differently (a flux that keeps one side's state) fails here before an oracle
    // sees it; the oracle's own grade is above.
    let hydro = twin();
    let mut flipped = mirrored(&strip());
    hydro.advance(&mut flipped, 0.0, T_END);
    let back = mirrored(&flipped);
    let mut worst = 0.0f64;
    for c in 0..COMPONENTS {
        worst = worst.max(max_diff(back.plane(c), run_80().plane(c)));
    }
    println!("SR-SOD-MIRROR twin: the mirrored run within {worst:.2e} of the straight one");
    assert!(worst < 1e-12, "mirror differs by {worst:e}");
}

#[test]
fn a_strip_laid_along_y_is_the_same_run() {
    // The unsplit scheme's y fluxes, graded: the strip as 4 columns × 400 rows, run to t = 80 and laid back along x.
    let hydro = twin();
    let mut along_y = strip().transposed();
    assert_eq!((along_y.width, along_y.height), (4, 400));
    hydro.advance(&mut along_y, 0.0, T_END);
    let back = along_y.transposed();
    let mut worst = 0.0f64;
    for c in 0..COMPONENTS {
        worst = worst.max(max_diff(back.plane(c), run_80().plane(c)));
    }
    println!("SR-SOD-Y twin: the y-oriented strip within {worst:.2e} of the x-oriented one");
    assert!(worst < 1e-12, "y differs from x by {worst:e}");
}

#[test]
fn species_ride_the_mass_flux_and_change_nothing_else() {
    // Hydrogen on the left, helium on the right, the same Σ and Π: pressure and density never see the species (cold off),
    // so the density is the pure-hydrogen run's, and X_H marks the gas that started on the left.
    let scene = sod_scene();
    let hydro = twin_of(&scene.physics().unwrap());
    let mut st = strip();
    let n = st.cells();
    for i in 0..n {
        let right = i % st.width >= 200;
        st.planes[4 * n + i] = if right { 0.0 } else { 1.0 };
        st.planes[5 * n + i] = if right { 1.0 } else { 0.0 };
    }
    hydro.advance(&mut st, 0.0, T_END);
    let straight = run_80();
    for c in 0..4 {
        let d = max_diff(st.plane(c), straight.plane(c));
        assert!(d < 1e-12, "channel {c} differs by {d:e}");
    }
    let w = st.width;
    let (mut worst_sum, mut lo, mut hi) = (0.0f64, f64::MAX, f64::MIN);
    for i in 0..n {
        worst_sum = worst_sum.max((((4..14).map(|c| st.get(c, i)).sum::<f64>()) - 1.0).abs());
        lo = lo.min(st.get(4, i));
        hi = hi.max(st.get(4, i));
    }
    // The contact is where the gas that started at the diaphragm is: u*·t from it; X_H crosses ½ within 2 cells of it.
    let sol = exact();
    let contact = X0 + sol.u_star * T_END;
    let x_h = &st.plane(4)[..w];
    let i = (0..w).find(|&i| x_h[i] < 0.5).expect("hydrogen ends");
    let crossing = (i as f64 - 0.5) + (x_h[i - 1] - 0.5) / (x_h[i - 1] - x_h[i]);
    println!(
        "SR-SOD-SPECIES twin: ΣX_i − 1 within {worst_sum:.1e}, X_H in [{lo:.2e}, {hi:.15}], contact {crossing:.3} for exact {contact:.3}"
    );
    assert!(worst_sum < 1e-12, "ΣX_i = 1 broken by {worst_sum:e}");
    assert!(lo > -1e-12 && hi < 1.0 + 1e-12, "X_H leaves [0, 1]: [{lo:e}, {hi:e}]");
    assert!((crossing - contact).abs() <= 2.0, "contact at {crossing}, exact {contact}");
}

#[test]
fn with_cold_pressure_on_a_flow_conserves_and_stays_physical() {
    // The shipped constants (electron and neutron-matter pairs on): a dense hydrogen disk-edge, Σ 2 | 0.5 at T = 2 and at
    // rest, run 20 t.u. — the waves stay off the walls, so the momentum the walls give is (Π_L − Π_R) · t · rows.
    let text = r#"{"version": 1, "world": {"width": 400, "height": 64, "edge": "bounce"},
        "objects": [
          {"kind": "rect", "x0": 0, "y0": 0, "w": 200, "h": 64, "sigma": 2.0, "t": 2.0, "composition": {"H": 1.0}, "v": [0, 0]},
          {"kind": "rect", "x0": 200, "y0": 0, "w": 200, "h": 64, "sigma": 0.5, "t": 2.0, "composition": {"H": 1.0}, "v": [0, 0]}]}"#;
    let scene = Scene::parse(text, &elements()).unwrap();
    let physics = scene.physics().unwrap();
    assert_eq!(physics.get("k1e"), Some(21.3));
    let hydro = twin_of(&physics);
    let mut st = world_of(&scene, &physics).rows(0, 4);
    let start = st.totals();
    let eos = sr_physics::eos::Eos::from_physics(&physics).unwrap();
    // H: Y_e = 1. A wall cell's Π from the planes' own energy (f32), at rest: ε_th = E/Σ − ε_cold.
    let pi = |i: usize| {
        let sigma = st.get(0, i);
        eos.pi(sigma, st.get(3, i) / sigma - eos.eps_cold(sigma, 1.0, 0.0), 1.0, 0.0)
    };
    let (pi_left, pi_right) = (pi(0), pi(st.width - 1));
    let steps = hydro.advance(&mut st, 0.0, 20.0);
    let end = st.totals();
    let pushed = (pi_left - pi_right) * 20.0 * 4.0;
    println!(
        "SR-SOD-COLD twin: Π {:.3} | {:.3} (cold {:.3} | {:.3}), {steps} steps, mass {:.14e} → {:.14e}, momentum {:.9} for {pushed:.9}",
        pi_left,
        pi_right,
        eos.pi_e(2.0, 1.0),
        eos.pi_e(0.5, 1.0),
        start[0],
        end[0],
        end[1]
    );
    assert!(st.plane(0).iter().all(|&s| s.is_finite() && s > 0.0), "Σ stays finite and positive");
    assert!(st.plane(3).iter().all(|e| e.is_finite()), "E stays finite");
    assert!((end[0] - start[0]).abs() / start[0] < 1e-12, "mass");
    assert!((end[3] - start[3]).abs() / start[3] < 1e-12, "energy");
    assert!((end[1] - pushed).abs() / pushed < 1e-10, "x momentum {} ≠ {pushed}", end[1]);
    // The cold pressure is a real share of Π here, or the case grades nothing the ideal gas does not.
    assert!(eos.pi_e(2.0, 1.0) > 0.2 * pi_left, "the electrons' share of Π");
}

// ---- M0-T34: the GPU's P4 ----

/// When a GPU run stops: at the first step whose sim time reaches a time, or after a number of steps.
#[derive(Clone, Copy)]
enum Until {
    Time(f64),
    Steps(u64),
}

/// A GPU run's end: its planes widened to the twin's state, its sim time and steps, the adapter it ran on.
struct GpuRun {
    state: State,
    sim_time: f64,
    steps: u64,
    adapter: String,
}

/// Runs `start` through the GPU's step with P4 alone (bounce walls), the box fit to it, until `until`. Toward a time, the
/// steps are submitted in batches of 4/5 of the remaining time over the last Δt, the last ones one at a time; a batch of
/// more than one step that crosses the time is refused (the read time would not be the first step's ≥ it).
fn gpu_run(physics: &Physics, start: &State, until: Until) -> GpuRun {
    let gpu: Gpu = super::device();
    let world = WorldConfig::new(start.width as u32, start.height as u32).unwrap();
    let state = GpuState::new(&gpu.device, world).unwrap();
    let planes: Vec<f32> = start.planes.iter().map(|&v| v as f32).collect();
    state.upload(&gpu.queue, &planes).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let eos = EosGpu::new(&gpu.device, physics, &elements());
    let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: Some(EdgeMode::Bounce) };
    let step = Step::with_passes(&gpu.device, &state, &booking, &eos, physics, &passes).unwrap();
    let report = loop {
        let before = step.read_dt(&gpu.device, &gpu.queue);
        let batch = match until {
            Until::Steps(n) if before.steps >= n => break before,
            Until::Steps(n) => (n - before.steps).min(256),
            Until::Time(t) if before.steps > 0 && before.sim_time >= t => break before,
            Until::Time(t) => {
                let dt = f64::from(if before.steps == 0 { before.dt_next } else { before.dt_next.max(before.dt) });
                (((t - before.sim_time) / dt * 0.8).floor() as u64).clamp(1, 256)
            }
        };
        step.run(&gpu.device, &gpu.queue, batch as u32);
        let after = step.read_dt(&gpu.device, &gpu.queue);
        assert!(after.non_finite.is_none(), "non-finite state: {:?}", after.non_finite);
        if let Until::Time(t) = until {
            assert!(batch == 1 || after.sim_time < t, "a batch of {batch} steps crossed t = {t}: {after:?}");
        }
    };
    let planes = state.readback(&gpu.device, &gpu.queue).iter().map(|&v| f64::from(v)).collect();
    GpuRun {
        state: State::new(start.width, start.height, planes),
        sim_time: report.sim_time,
        steps: report.steps,
        adapter: gpu.info.sr_adapter_line(),
    }
}

/// The scene's 400 × 64 world on the GPU, to the first step at t ≥ 80, once for every case that reads it.
fn gpu_80() -> &'static GpuRun {
    static RUN: OnceLock<GpuRun> = OnceLock::new();
    RUN.get_or_init(|| {
        let scene = sod_scene();
        let physics = scene.physics().unwrap();
        let run = gpu_run(&physics, &world_of(&scene, &physics), Until::Time(T_END));
        println!("SR-SOD-RUN gpu 400x64 world: {} steps to t = {:.6} ({})", run.steps, run.sim_time, run.adapter);
        run
    })
}

/// The largest |a − b| between row `y` of plane `c` and row 0 of the same plane.
fn row_diff(st: &State, c: usize, y: usize) -> f64 {
    let w = st.width;
    max_diff(&st.plane(c)[y * w..(y + 1) * w], &st.plane(c)[..w])
}

#[test]
fn gpu_the_shock_is_within_one_cell() {
    let run = gpu_80();
    println!("{}", run.adapter);
    let w = run.state.width;
    let mut worst = (0.0f64, 0usize, 0.0, 0.0);
    for y in 0..run.state.height {
        let (x_num, x_exact) = shock_at(&run.state.plane(0)[y * w..(y + 1) * w], run.sim_time);
        if (x_num - x_exact).abs() >= worst.0 {
            worst = ((x_num - x_exact).abs(), y, x_num, x_exact);
        }
    }
    let (off, y, x_num, x_exact) = worst;
    println!("SR-SOD-SHOCK gpu at t = {:.4}: worst row {y}, numerical {x_num:.3}, exact {x_exact:.3}, off {off:.3} cells ≤ 1", run.sim_time);
    assert!(off <= 1.0, "row {y}: shock at {x_num}, exact {x_exact}");
}

#[test]
fn gpu_the_l1_density_error_is_within_two_percent() {
    let run = gpu_80();
    println!("{}", run.adapter);
    let w = run.state.width;
    let worst =
        (0..run.state.height).map(|y| l1_at(&run.state.plane(0)[y * w..(y + 1) * w], run.sim_time)).fold(0.0, f64::max);
    println!(
        "SR-SOD-L1 gpu at t = {:.4}: worst row's Σ|ρ − ρ_exact| / Σρ_exact = {:.4} % ≤ 2 %",
        run.sim_time,
        100.0 * worst
    );
    assert!(worst <= 0.02, "L1 density error {:.4} %", 100.0 * worst);
}

#[test]
fn gpu_every_row_is_the_same_gas() {
    // 64 identical rows between two bounce walls stay identical: a y flux that moves gas, momentum or energy between equal
    // rows (a wrong neighbour, a wrong wall) shows here as a row that differs.
    let run = gpu_80();
    println!("{}", run.adapter);
    let st = &run.state;
    let mut worst = (0.0f64, 0usize, 0usize);
    for c in 0..COMPONENTS {
        for y in 1..st.height {
            let d = row_diff(st, c, y);
            if d > worst.0 {
                worst = (d, c, y);
            }
        }
    }
    let v_max = st.plane(2).iter().zip(st.plane(0)).map(|(m, s)| (m / s).abs()).fold(0.0, f64::max);
    println!(
        "SR-SOD-ROWS gpu: every row within {:.2e} of row 0 (channel {}, row {}), |v| ≤ {v_max:.2e}",
        worst.0, worst.1, worst.2
    );
    assert!(worst.0 <= 1e-5, "row {} differs from row 0 by {:e} in channel {}", worst.2, worst.0, worst.1);
    assert!(v_max <= 1e-5, "|v| reaches {v_max:e}");
}

#[test]
fn gpu_mass_energy_and_the_walls_momentum_hold() {
    let run = gpu_80();
    println!("{}", run.adapter);
    let scene = sod_scene();
    let first = world_of(&scene, &scene.physics().unwrap());
    let (start, end) = (first.totals(), run.state.totals());
    let rel = |a: f64, b: f64| (a - b).abs() / b.abs();
    // As the twin's case: the walls' pressures over the 64 rows for the run's time, the planes' f32 pressures (Π = E).
    let pushed = (first.get(3, 0) - first.get(3, first.width - 1)) * run.sim_time * first.height as f64;
    println!(
        "SR-SOD-CONS gpu: mass {:.3e}, energy {:.3e} relative; momentum {:.4} for {pushed:.4} ({:.3e}), y momentum {:.3e}",
        rel(end[0], start[0]),
        rel(end[3], start[3]),
        end[1],
        rel(end[1], pushed),
        end[2]
    );
    assert!(rel(end[0], start[0]) <= 1e-5, "mass");
    assert!(rel(end[3], start[3]) <= 1e-5, "energy: a wall passes none");
    assert!(rel(end[1], pushed) <= 1e-4, "x momentum {} ≠ {pushed}", end[1]);
    assert!(end[2].abs() <= 1e-4 * pushed, "y momentum {}", end[2]);
}

#[test]
fn gpu_the_tube_laid_along_y_is_the_same_run() {
    // The adversarial case of M0-T34: the tube along y (64 columns × 400 rows) for the x run's steps, laid back along x,
    // is the x run — the y fluxes and the y walls graded on the GPU as the x ones.
    let x_run = gpu_80();
    let scene = sod_scene();
    let physics = scene.physics().unwrap();
    let y_run = gpu_run(&physics, &world_of(&scene, &physics).transposed(), Until::Steps(x_run.steps));
    println!("{}", y_run.adapter);
    let back = y_run.state.transposed();
    let worst = (0..COMPONENTS).map(|c| max_diff(back.plane(c), x_run.state.plane(c))).fold(0.0, f64::max);
    println!(
        "SR-SOD-Y gpu: {} steps, t = {:.6} along y and {:.6} along x, states within {worst:.2e}",
        y_run.steps, y_run.sim_time, x_run.sim_time
    );
    assert_eq!(y_run.steps, x_run.steps);
    assert!((y_run.sim_time - x_run.sim_time).abs() <= 1e-5 * x_run.sim_time, "sim times differ");
    assert!(worst <= 1e-5, "y differs from x by {worst:e}");
}

#[test]
fn gpu_with_cold_pressure_on_a_flow_conserves_and_stays_physical() {
    // The twin's cold-pressure case on the GPU's 400 × 64 world: eos.wgsl's cold terms in every face state.
    let text = r#"{"version": 1, "world": {"width": 400, "height": 64, "edge": "bounce"},
        "objects": [
          {"kind": "rect", "x0": 0, "y0": 0, "w": 200, "h": 64, "sigma": 2.0, "t": 2.0, "composition": {"H": 1.0}, "v": [0, 0]},
          {"kind": "rect", "x0": 200, "y0": 0, "w": 200, "h": 64, "sigma": 0.5, "t": 2.0, "composition": {"H": 1.0}, "v": [0, 0]}]}"#;
    let scene = Scene::parse(text, &elements()).unwrap();
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics);
    let run = gpu_run(&physics, &first, Until::Time(20.0));
    println!("{}", run.adapter);
    let eos = sr_physics::eos::Eos::from_physics(&physics).unwrap();
    let pi = |i: usize| {
        let sigma = first.get(0, i);
        eos.pi(sigma, first.get(3, i) / sigma - eos.eps_cold(sigma, 1.0, 0.0), 1.0, 0.0)
    };
    let (start, end) = (first.totals(), run.state.totals());
    let pushed = (pi(0) - pi(first.width - 1)) * run.sim_time * first.height as f64;
    let rel = |a: f64, b: f64| (a - b).abs() / b.abs();
    println!(
        "SR-SOD-COLD gpu: {} steps to t = {:.4}, mass {:.3e}, energy {:.3e} relative, momentum {:.4} for {pushed:.4} ({:.3e})",
        run.steps,
        run.sim_time,
        rel(end[0], start[0]),
        rel(end[3], start[3]),
        end[1],
        rel(end[1], pushed)
    );
    assert!(run.state.plane(0).iter().all(|&s| s.is_finite() && s > 0.0), "Σ stays finite and positive");
    assert!(run.state.plane(3).iter().all(|e| e.is_finite()), "E stays finite");
    assert!(rel(end[0], start[0]) <= 1e-5, "mass");
    assert!(rel(end[3], start[3]) <= 1e-5, "energy");
    assert!(rel(end[1], pushed) <= 1e-4, "x momentum {} ≠ {pushed}", end[1]);
}
