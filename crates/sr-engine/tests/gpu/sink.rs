//! M0-T50 — sinks on the CPU twin (contract §1.4.3, §2.6, §2.9, §5.1's G-SINK, SM-E28, SM-E29); M0-T51 — P3 on the GPU
//! (`Step::with_sinks`, shaders/sinks.wgsl), graded against the twin from the same f32 start and on the conservation
//! itself (the plant moves the twin and the shader alike, so a GPU case never rests on the twin alone).
//!
//! G-SINK, *by construction*: a sink in a uniform bound cloud (scenes/sink_cloud.json: a 40 × 40 square of hydrogen at
//! Σ = 1 drifting at (0.3, −0.2), a 64 × 64 world, the cold pressure off by `overrides`) — grid + sink mass and momentum
//! constant to 10⁻⁶ relative through accretion, no cell beyond r_acc touched and no cell's Σ ever raised (so no mass
//! flows outward across r_acc), the gas's thermal and cold energy booked `swallowed`. Formation: all four criteria at one
//! cell, each with its own refusal case (SM-E28's warning: a density threshold alone forms sinks in a bouncing core).
//! The synthetic worlds of the formation cases are a bowl of φ with the gas converging (or not) on its bottom: a source
//! read alone, `NOT PROVEN (synthetic)` for anything beyond the criteria themselves.
//!
//! Motion (§0.5 [M0-V7]): the sinks pull the gas and are pulled by it as equal and opposite — each sink's acceleration is
//! the sum over the gas of its softened pull's reaction, graded by the no-net-force bound on the twin (M0-D45) and on the
//! GPU (M0-D46: P3 sums the box's tiles, so a sink outside the box feels the gas too); the `gpu_*` cases grade P3 against
//! the twin's `step` and `advance`.
//!
//! A sink from mass (§0.5 [M0-R6], M0-D90): under the shipped physics (the cold pressure on, so m_tov_sb is the sheet's), a
//! neutronized core holding m_tov_sb within 4 cells forms a sink that takes the disk; 0.95 of it, the same mass as iron
//! (Y_e 0.464) or diverging does not, and Σ ≥ Σ_BH alone still forms one that takes its cell — on the twin and on the GPU.

use std::path::{Path, PathBuf};

use sr_engine::dump::Sink as GpuSink;
use sr_engine::gpu::Gpu;
use sr_engine::scene::{EdgeMode, Scene};
use sr_engine::state::{book_totals, Booking, SideFields, State as GpuState, WorldConfig, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::fft::read_back;
use sr_engine::step::latch::{FramePlan, FrameReport};
use sr_engine::step::sinks::{SinkReport, SinkSetup};
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::grav::{add_sinks, Field, Gravity, SinkSource};
use sr_physics::reference::hydro::{Edge, Hydro, State, COMPONENTS};
use sr_physics::reference::sinks::{disk, r_acc, Sink, Sinks, MAX_SINKS, R_CORE};
use sr_physics::registry::{Elements, Physics};

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scene() -> Scene {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes");
    Scene::load(&dir.join("sink_cloud.json")).unwrap_or_else(|e| panic!("{e}"))
}

fn physics() -> Physics {
    scene().physics().unwrap()
}

/// The scene's planes as the twin's state, f32 widened to f64.
fn cloud() -> State {
    let scene = scene();
    let (planes, warnings) = scene.planes(scene.world, &physics(), &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    State::new(scene.world.width as usize, scene.world.height as usize, planes.iter().map(|&v| f64::from(v)).collect())
}

fn sinks() -> Sinks {
    Sinks::new(&physics(), &elements(), 1.0).unwrap()
}

fn sink(x: f64, y: f64, vx: f64, vy: f64, m: f64) -> Sink {
    Sink { x, y, vx, vy, m }
}

/// Mass and momentum of the grid, summed in cell order.
fn grid_totals(st: &State) -> (f64, [f64; 2]) {
    (st.plane(0).iter().sum(), [st.plane(1).iter().sum(), st.plane(2).iter().sum()])
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

fn zero_field(w: usize, h: usize) -> Field {
    Field { phi: vec![0.0; w * h], gx: vec![0.0; w * h], gy: vec![0.0; w * h] }
}

#[test]
fn the_scene_is_a_uniform_cloud_with_the_sinks_on() {
    let (scene, st) = (scene(), cloud());
    let s = scene.switches;
    assert!(s.gravity && s.hydro && !s.heat && !s.reactions && !s.neutrinos && s.sinks, "{s:?}");
    let (mass, mom) = grid_totals(&st);
    // 40 × 40 cells of Σ = 1 (the vacuum around it is Σ_floor), drifting at (0.3, −0.2).
    assert!(rel(mass, 1600.0) < 1e-6, "mass {mass}");
    assert!(rel(mom[0], 480.0) < 1e-6 && rel(mom[1], -320.0) < 1e-6, "momentum {mom:?}");
    println!("SR-SINK scene: mass {mass:.4}, momentum {mom:?}");
}

#[test]
fn r_acc_follows_two_gm_over_c_squared_with_a_floor_of_two_cells() {
    let c_sb = physics().get("c_sb").unwrap();
    assert_eq!(c_sb, 200.0, "§2.7's light speed");
    assert_eq!(r_acc(1.0, 2000.0, c_sb), 2.0, "2GM/c² = 0.1 < 2");
    assert!(rel(r_acc(1.0, 200_000.0, c_sb), 10.0) < 1e-12, "2GM/c² = 10");
    let mut s = sinks();
    s.sinks.push(sink(10.0, 10.0, 0.0, 0.0, 200_000.0));
    assert!(rel(s.r_acc(0), 10.0) < 1e-12);
}

/// φ and g of a sink (§1.4.3): −GM/√(r² + r_acc²) and its exact gradient, g the central difference of φ's own term.
#[test]
fn the_sinks_potential_and_pull_are_the_softened_point_mass() {
    let (w, h) = (24, 24);
    let src = SinkSource { x: 11.3, y: 12.6, m: 300.0, r_acc: 3.0 };
    let mut field = zero_field(w, h);
    add_sinks(&mut field, w, h, 1.0, &[src]);
    for (x, y) in [(11, 12), (3, 4), (20, 18), (11, 3)] {
        let i = y * w + x;
        let (dx, dy) = (x as f64 + 0.5 - src.x, y as f64 + 0.5 - src.y);
        let soft = dx * dx + dy * dy + 9.0;
        assert!(rel(field.phi[i], -300.0 / soft.sqrt()) < 1e-12, "φ at ({x}, {y})");
        // g = −∇φ of the analytic potential, a fine central difference of it: the exact gradient, not the grid's.
        let phi_at = |px: f64, py: f64| -300.0 / ((px - src.x).powi(2) + (py - src.y).powi(2) + 9.0).sqrt();
        let (px, py, e) = (x as f64 + 0.5, y as f64 + 0.5, 1e-5);
        let gx = -(phi_at(px + e, py) - phi_at(px - e, py)) / (2.0 * e);
        let gy = -(phi_at(px, py + e) - phi_at(px, py - e)) / (2.0 * e);
        let (ax, ay) = (field.gx[i], field.gy[i]);
        assert!((gx - ax).hypot(gy - ay) <= 1e-6 * ax.hypot(ay), "g at ({x}, {y}): {ax:.8} {ay:.8} against ∇φ {gx:.8} {gy:.8}");
        // Far from the core the grid's own central difference of the added φ agrees to its O(h²).
        if (dx * dx + dy * dy).sqrt() > 8.0 {
            let (cx, cy) = (-(field.phi[i + 1] - field.phi[i - 1]) / 2.0, -(field.phi[i + w] - field.phi[i - w]) / 2.0);
            assert!((cx - ax).hypot(cy - ay) <= 0.02 * ax.hypot(ay), "grid ∇φ at ({x}, {y}): {cx:.5} {cy:.5} against g {ax:.5} {ay:.5}");
        }
        // And it points at the sink.
        assert!(ax * dx + ay * dy < 0.0, "g away from the sink at ({x}, {y})");
    }
    // Added to the gravity pass's field, φ and g carry the same term.
    let mut gravity = Gravity::new(1.0);
    let sigma = vec![0.0; w * h];
    let with = gravity.field_with_sinks(&sigma, 1e-6, w, h, 1.0, &[src]).unwrap();
    assert!(with.phi.iter().zip(&field.phi).all(|(a, b)| (a - b).abs() < 1e-12) && with.gx.iter().zip(&field.gx).all(|(a, b)| (a - b).abs() < 1e-12));
}

#[test]
fn accretion_keeps_grid_and_sink_mass_and_momentum_and_books_the_energy() {
    let mut st = cloud();
    let before = st.clone();
    let (mass0, mom0) = grid_totals(&st);
    let mut s = sinks();
    s.sinks.push(sink(32.5, 32.5, 0.0, 0.0, 2000.0));
    let ra = s.r_acc(0);
    let taken = s.accrete(&mut st);
    let (mass1, mom1) = grid_totals(&st);
    let (sm, sp) = (s.mass(), s.momentum());
    println!("SR-SINK accretion: r_acc {ra}, took {taken:.6} of {mass0:.3}; sink mass {sm:.6}, momentum {sp:?}; swallowed {:.6}", s.swallowed);
    assert!(taken > 10.0, "a disc of r_acc = 2 holds about 12 cells of Σ = 1: took {taken}");
    // The sink had 2000 and took `taken`; grid + sinks as they were.
    assert!(rel(mass1 + sm, mass0 + 2000.0) < 1e-6, "mass {} against {}", mass1 + sm, mass0 + 2000.0);
    assert!(rel(mom1[0] + sp[0], mom0[0]) < 1e-6 && rel(mom1[1] + sp[1], mom0[1]) < 1e-6, "momentum {:?} + {sp:?} against {mom0:?}", mom1);
    assert!(rel(sm - 2000.0, taken) < 1e-9, "the sink gained what the pass reports");
    // What the sink was given is the gas's momentum at the cloud's velocity.
    assert!(rel(sp[0], 0.3 * taken) < 1e-6 && rel(sp[1], -0.2 * taken) < 1e-6, "{sp:?} for {taken}");
    // Thermal energy of ionised hydrogen at T = 0.05: ε_th = T/μ = 0.1 per mass, booked `swallowed`.
    assert!(rel(s.swallowed, 0.1 * taken) < 1e-6, "swallowed {} against {}", s.swallowed, 0.1 * taken);
    // Σ never raised, and nothing beyond r_acc touched: no mass crosses r_acc outward.
    let w = st.width;
    for i in 0..st.cells() {
        let (x, y) = ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5);
        let r = (x - 32.5).hypot(y - 32.5);
        assert!(st.get(0, i) <= before.get(0, i), "Σ raised at cell {i}");
        if r > ra {
            for c in 0..COMPONENTS {
                assert_eq!(st.get(c, i), before.get(c, i), "cell {i} (r = {r:.3}) channel {c} touched beyond r_acc");
            }
        }
    }
}

#[test]
fn accretion_takes_bound_gas_only_in_the_sinks_own_frame() {
    // Hot gas (ε_th = 10 000 against GM/r ≤ 4 000 at r = ½): not bound.
    let mut hot = cloud();
    let n = hot.cells();
    for i in 0..n {
        hot.planes[3 * n + i] *= 1e5;
    }
    let before = hot.clone();
    let mut s = sinks();
    s.sinks.push(sink(32.5, 32.5, 0.0, 0.0, 2000.0));
    assert_eq!(s.accrete(&mut hot), 0.0, "hot gas is not bound");
    assert_eq!(hot, before);
    assert_eq!((s.swallowed, s.sinks[0].m), (0.0, 2000.0));
    // Gas flying by at (0.3, −0.2) against a sink at rest is bound; at 100 cells per t.u. (½|u|² = 5 000) it is not …
    let mut fast = cloud();
    for i in 0..n {
        fast.planes[n + i] *= 300.0;
        fast.planes[2 * n + i] *= 300.0;
        let k = 0.5 * (fast.planes[n + i].powi(2) + fast.planes[2 * n + i].powi(2)) / fast.planes[i];
        fast.planes[3 * n + i] += k;
    }
    let mut s = sinks();
    s.sinks.push(sink(32.5, 32.5, 0.0, 0.0, 2000.0));
    assert_eq!(s.accrete(&mut fast.clone()), 0.0, "gas at 108 cells per t.u. is unbound at M = 2000");
    // … and bound again from a sink moving with it: the test is in the sink's frame, ½|u − v_s|².
    let mut s = sinks();
    s.sinks.push(sink(32.5, 32.5, 90.0, -60.0, 2000.0));
    let taken = s.accrete(&mut fast);
    assert!(taken > 10.0, "a sink riding with the gas takes it: {taken}");
}

#[test]
fn the_cloud_falls_into_a_sink_with_mass_conserved_each_step() {
    // P2 (the gas's field plus the sink's pull), P3, P4 as §1.3.2 orders them, a bounce world so nothing escapes.
    let physics = physics();
    let mut st = cloud();
    let (w, h) = (st.width, st.height);
    let hydro = Hydro::new(&physics, &elements(), Edge::Bounce).unwrap();
    let mut gravity = Gravity::new(1.0);
    let mut s = sinks();
    s.sinks.push(sink(32.5, 32.5, 0.0, 0.0, 2000.0));
    let total = grid_totals(&st).0 + s.mass();
    let p0 = grid_totals(&st).1;
    let mut last = s.mass();
    for step in 0..40 {
        let grid = gravity.field(st.plane(0), 1e-6, w, h).unwrap();
        let mut full = grid.clone();
        add_sinks(&mut full, w, h, 1.0, &s.sources());
        let dt = hydro.dt(&st, Some((&full.gx[..], &full.gy[..])));
        let report = s.step(&mut st, &grid, dt);
        assert!(report.formed.is_none() && report.merged == 0, "step {step}: {report:?}");
        hydro.step_with(&mut st, dt, Some(&full));
        let now = grid_totals(&st).0 + s.mass();
        assert!(rel(now, total) < 1e-6, "step {step}: grid + sink mass {now} against {total}");
        assert!(s.mass() >= last, "step {step}: the sink lost mass");
        last = s.mass();
    }
    let (gp, sp) = (grid_totals(&st).1, s.momentum());
    println!("SR-SINK fall: after 40 steps the sink holds {:.4} (2000 + {:.4}); grid + sink momentum ({:.4}, {:.4}) from ({:.4}, {:.4})", s.mass(), s.mass() - 2000.0, gp[0] + sp[0], gp[1] + sp[1], p0[0], p0[1]);
    assert!(s.mass() > 2000.0, "the cloud reached the sink");
}

// ---- formation ----

/// A `w` × `h` world of gas at Σ = 100 and T = 0.05 (ε_th = 0.1), converging on the nearest of the `wells` (or diverging,
/// `sign` = +1) at 0.1 per cell, each well's own cell at the density the well names and at rest, and the potential
/// −100 + ½d² about the nearest well plus `offset`: every well a local minimum, the criteria's inputs in plain sight.
fn world(w: usize, h: usize, wells: &[(usize, usize, f64)], sign: f64, offset: f64) -> (State, Vec<f64>) {
    let n = w * h;
    let mut planes = vec![0.0; COMPONENTS * n];
    let mut phi = vec![0.0; n];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let d2 = |&(wx, wy, _): &(usize, usize, f64)| ((x as f64 - wx as f64).powi(2) + (y as f64 - wy as f64).powi(2), wx, wy);
            let (d2, wx, wy) = wells.iter().map(d2).min_by(|a, b| a.0.total_cmp(&b.0)).unwrap();
            let at_well = wells.iter().find(|&&(wx, wy, _)| (wx, wy) == (x, y));
            let sigma = at_well.map_or(100.0, |&(_, _, s)| s);
            let u = [sign * 0.1 * (x as f64 - wx as f64), sign * 0.1 * (y as f64 - wy as f64)];
            planes[i] = sigma;
            planes[n + i] = sigma * u[0];
            planes[2 * n + i] = sigma * u[1];
            planes[3 * n + i] = sigma * (0.1 + 0.5 * (u[0] * u[0] + u[1] * u[1]));
            planes[4 * n + i] = 1.0;
            phi[i] = -100.0 + 0.5 * d2 + offset;
        }
    }
    (State::new(w, h, planes), phi)
}

#[test]
fn a_dense_converging_bound_potential_minimum_forms_a_sink_and_books_the_cell() {
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 0.0);
    let (mass0, _) = grid_totals(&st);
    let e0: f64 = st.plane(3).iter().sum();
    let mut s = sinks();
    assert_eq!(s.form(&mut st, &phi), Some((8, 8)));
    assert_eq!(s.sinks.len(), 1);
    let k = s.sinks[0];
    assert_eq!((k.x, k.y, k.vx, k.vy), (8.5, 8.5, 0.0, 0.0), "at the cell's centre, with the cell's (zero) velocity");
    // The sink takes the cell's mass above the floor; the cell keeps the floor's share, its fractions, its velocity.
    let floor = physics().get("sigma_floor").unwrap();
    let i = 8 * 16 + 8;
    assert!(rel(k.m, 7000.0 - floor) < 1e-12 && rel(st.get(0, i), floor) < 1e-9, "sink {} cell {}", k.m, st.get(0, i));
    assert_eq!(st.get(4, i), 1.0);
    assert!(rel(grid_totals(&st).0 + s.mass(), mass0) < 1e-12, "mass");
    // Thermal and cold energy booked `swallowed`: 7000 × 0.1; the energy leaving the grid is that and nothing else here.
    assert!(rel(s.swallowed, 700.0) < 1e-9, "swallowed {}", s.swallowed);
    let e1: f64 = st.plane(3).iter().sum();
    assert!(rel(e0 - e1, s.swallowed) < 1e-9, "the grid's energy fell by {} for {} booked", e0 - e1, s.swallowed);
}

/// Each of the four criteria has its own refusal: a world that meets the other three and not this one.
fn refused(name: &str, st: &mut State, phi: &[f64]) {
    let before = st.clone();
    let mut s = sinks();
    assert_eq!(s.form(st, phi), None, "{name}: a sink formed");
    assert!(s.sinks.is_empty() && s.swallowed == 0.0 && *st == before, "{name}: the world was touched");
}

#[test]
fn formation_refused_below_the_density_threshold() {
    let bh = physics().get("sigma_bh").unwrap();
    assert_eq!(bh, 6000.0);
    let (mut st, phi) = world(16, 16, &[(8, 8, bh * (1.0 - 1e-6))], -1.0, 0.0);
    refused("Σ just below Σ_BH", &mut st, &phi);
    // The same world just above it forms: the threshold is the whole difference.
    let (mut st, phi) = world(16, 16, &[(8, 8, bh)], -1.0, 0.0);
    assert_eq!(sinks().form(&mut st, &phi), Some((8, 8)), "Σ = Σ_BH forms");
}

#[test]
fn formation_refused_in_a_bouncing_core_with_only_the_density_threshold_met() {
    // SM-E28's warning: dense, a potential minimum and bound, but the gas is expanding (∇·u > 0).
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], 1.0, 0.0);
    refused("∇·u > 0", &mut st, &phi);
}

#[test]
fn formation_refused_when_the_cell_is_not_the_potential_minimum() {
    let (mut st, mut phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 0.0);
    // One of the eight neighbours, the diagonal one, lower; that neighbour is below Σ_BH, so it cannot form instead.
    phi[7 * 16 + 9] = phi[8 * 16 + 8] - 1.0;
    refused("a lower neighbour", &mut st, &phi);
}

#[test]
fn formation_refused_when_the_gas_is_not_bound() {
    // The bowl lifted to φ = +100: still a minimum, still dense and converging, ½|u|² + ε_th + φ > 0.
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 200.0);
    refused("½|u|² + ε_th + φ > 0", &mut st, &phi);
    // And just past the edge of boundness: φ = −ε_th + 0.01 (the bowl's bottom at u = 0).
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 100.0 - 0.1 + 0.01);
    refused("½|u|² + ε_th + φ = 0.01", &mut st, &phi);
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 100.0 - 0.1 - 0.01);
    assert_eq!(sinks().form(&mut st, &phi), Some((8, 8)), "and just inside it forms");
}

#[test]
fn the_densest_candidate_forms_ties_go_to_the_lowest_y_then_x_and_one_forms_a_step() {
    let phys = physics();
    let floor = phys.get("sigma_floor").unwrap();
    // Equal densities: the lowest y wins, whatever the x.
    let (mut st, phi) = world(24, 16, &[(18, 4, 7000.0), (5, 11, 7000.0)], -1.0, 0.0);
    let mut s = sinks();
    assert_eq!(s.form(&mut st, &phi), Some((18, 4)), "ties: the lowest y");
    // At most one per step: the other candidate waits for the next call, and the first cell, now at the floor, is not one.
    assert_eq!(s.sinks.len(), 1);
    assert_eq!(s.form(&mut st, &phi), Some((5, 11)));
    assert_eq!(s.form(&mut st, &phi), None, "nothing is left to form");
    assert!(s.sinks.iter().all(|k| rel(k.m, 7000.0 - floor) < 1e-12));
    // The same row: the lowest x.
    let (mut st, phi) = world(24, 16, &[(18, 4, 7000.0), (5, 4, 7000.0)], -1.0, 0.0);
    assert_eq!(sinks().form(&mut st, &phi), Some((5, 4)), "ties: then the lowest x");
    // Denser beats lower: the candidate with more mass forms first, wherever it sits.
    let (mut st, phi) = world(24, 16, &[(5, 4, 7000.0), (18, 11, 8000.0)], -1.0, 0.0);
    assert_eq!(sinks().form(&mut st, &phi), Some((18, 11)), "the densest");
}

#[test]
fn no_seventeenth_sink_forms() {
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 0.0);
    let mut s = sinks();
    for k in 0..MAX_SINKS {
        s.sinks.push(sink(100.0 + 10.0 * k as f64, 1.0, 0.0, 0.0, 1.0));
    }
    assert_eq!(s.form(&mut st, &phi), None, "16 sinks stand");
    s.sinks.pop();
    assert_eq!(s.form(&mut st, &phi), Some((8, 8)), "15 stand: the sixteenth forms");
}

// ---- a sink from mass (§0.5 [M0-R6]) ----

/// The shipped physics: the cold pressure on, so the mass test reads the sheet's m_tov_sb.
fn shipped() -> Physics {
    Physics::shipped().unwrap()
}

fn mass_sinks() -> Sinks {
    Sinks::new(&shipped(), &elements(), 1.0).unwrap()
}

/// A `w` × `h` world of hydrogen at Σ = 1 at rest with, about cell (cx, cy), a core of `mass` in the disk r ≤ 4 (49 cells,
/// Σ = mass/50, its centre twice that: the one local maximum) of pure `species`, converging on its centre at 0.1 per cell
/// (diverging, `sign` = +1); ε_th = 0.1 everywhere and φ = −100 + ½d² about the centre.
fn core_world(w: usize, h: usize, (cx, cy): (usize, usize), mass: f64, species: &str, sign: f64) -> (State, Vec<f64>) {
    let (n, k) = (w * h, elements().index_of(species).unwrap());
    let mut planes = vec![0.0; COMPONENTS * n];
    let mut phi = vec![0.0; n];
    let core: Vec<usize> = disk(cx, cy, R_CORE, w, h).collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let d = [x as f64 - cx as f64, y as f64 - cy as f64];
            let (sigma, u, sp) = match (core.contains(&i), (x, y) == (cx, cy)) {
                (true, centre) => (mass / 50.0 * if centre { 2.0 } else { 1.0 }, [sign * 0.1 * d[0], sign * 0.1 * d[1]], k),
                (false, _) => (1.0, [0.0; 2], 0),
            };
            planes[i] = sigma;
            planes[n + i] = sigma * u[0];
            planes[2 * n + i] = sigma * u[1];
            planes[3 * n + i] = sigma * (0.1 + 0.5 * (u[0] * u[0] + u[1] * u[1]));
            planes[(4 + sp) * n + i] = 1.0;
            phi[i] = -100.0 + 0.5 * (d[0] * d[0] + d[1] * d[1]);
        }
    }
    (State::new(w, h, planes), phi)
}

#[test]
fn a_neutronized_core_holding_m_tov_sb_within_4_cells_forms_a_sink_that_takes_the_disk() {
    let mut s = mass_sinks();
    let m_tov = s.m_tov();
    assert!((4800.0..4900.0).contains(&m_tov), "the sheet's m_tov_sb, 4,853 (reports/cold_ceiling.md): {m_tov}");
    let (mut st, phi) = core_world(24, 24, (12, 12), 1.05 * m_tov, "n", -1.0);
    let before = st.clone();
    let (mass0, mom0) = grid_totals(&st);
    let e0: f64 = st.plane(3).iter().sum();
    assert_eq!(s.form(&mut st, &phi), Some((12, 12)));
    let k = s.sinks[0];
    let floor = shipped().get("sigma_floor").unwrap();
    // The disk's 49 cells give their mass above the floor; every other cell is untouched.
    let core: Vec<usize> = disk(12, 12, R_CORE, 24, 24).collect();
    assert_eq!(core.len(), 49);
    let taken: f64 = core.iter().map(|&i| before.get(0, i) - floor).sum();
    assert!(rel(k.m, taken) < 1e-12 && rel(k.m, 1.05 * m_tov) < 1e-6, "sink {} against the disk's {taken}", k.m);
    assert_eq!((k.x, k.y), (12.5, 12.5), "at the core's centre");
    for i in 0..576 {
        if core.contains(&i) {
            assert!(rel(st.get(0, i), floor) < 1e-9 && st.get(13, i) == 1.0, "cell {i} left at the floor, neutron matter");
        } else {
            assert!((0..COMPONENTS).all(|c| st.get(c, i) == before.get(c, i)), "cell {i} beyond the disk touched");
        }
    }
    // Mass and momentum move from the grid to the sink; the energy leaving the grid is the kinetic part with the momentum and
    // the rest booked `swallowed`.
    let (mass1, mom1) = grid_totals(&st);
    assert!(rel(mass1 + s.mass(), mass0) < 1e-12, "mass");
    let p = s.momentum();
    assert!((mom1[0] + p[0] - mom0[0]).abs() < 1e-9 && (mom1[1] + p[1] - mom0[1]).abs() < 1e-9, "momentum");
    let kinetic: f64 = core.iter().map(|&i| (1.0 - floor / before.get(0, i)) * 0.5 * (before.get(1, i).powi(2) + before.get(2, i).powi(2)) / before.get(0, i)).sum();
    let e1: f64 = st.plane(3).iter().sum();
    assert!(rel(e0 - e1, s.swallowed + kinetic) < 1e-9, "the grid's energy fell by {} for {} booked + {kinetic} kinetic", e0 - e1, s.swallowed);
}

#[test]
fn a_core_under_m_tov_sb_as_iron_or_diverging_forms_none() {
    let m_tov = mass_sinks().m_tov();
    for (name, mass, species, sign) in [("0.95 m_tov_sb", 0.95, "n", -1.0), ("iron, Y_e 0.464", 1.05, "Fe", -1.0), ("∇·u > 0", 1.05, "n", 1.0)] {
        let (mut st, phi) = core_world(24, 24, (12, 12), mass * m_tov, species, sign);
        let before = st.clone();
        let mut s = mass_sinks();
        assert_eq!(s.form(&mut st, &phi), None, "{name}: a sink formed");
        assert!(s.sinks.is_empty() && s.swallowed == 0.0 && st == before, "{name}: the world was touched");
    }
}

#[test]
fn sigma_bh_alone_still_forms_a_sink_that_takes_its_cell() {
    // Hydrogen (Y_e 1): the disk holds 7,000 + 48 × 100 > m_tov_sb but is not neutronized, so Σ ≥ Σ_BH forms, as §2.6.
    let (mut st, phi) = world(16, 16, &[(8, 8, 7000.0)], -1.0, 0.0);
    let before = st.clone();
    let mut s = mass_sinks();
    let core_mass: f64 = disk(8, 8, R_CORE, 16, 16).map(|i| st.get(0, i)).sum();
    assert!(core_mass > s.m_tov(), "{core_mass}");
    assert_eq!(s.form(&mut st, &phi), Some((8, 8)));
    let floor = shipped().get("sigma_floor").unwrap();
    assert!(rel(s.sinks[0].m, 7000.0 - floor) < 1e-12, "the cell alone: {}", s.sinks[0].m);
    assert!((0..256).filter(|&i| i != 8 * 16 + 8).all(|i| (0..COMPONENTS).all(|c| st.get(c, i) == before.get(c, i))), "only the cell");
}

// ---- motion and merging ----

/// The direct sum, written here and not in the twin: the pull of the gas cells at Σ ≥ Σ_vac on a sink at `at`, softened by
/// `ra` — Σ_cells G m d / (|d|² + ra²)^{3/2}, d from the sink to the cell's centre.
fn gas_pull(st: &State, at: [f64; 2], ra: f64) -> [f64; 2] {
    let mut a = [0.0; 2];
    for y in 0..st.height {
        for x in 0..st.width {
            let m = st.get(0, y * st.width + x);
            if m < 1e-6 {
                continue;
            }
            let (dx, dy) = (x as f64 + 0.5 - at[0], y as f64 + 0.5 - at[1]);
            let k = m / (dx * dx + dy * dy + ra * ra).powf(1.5);
            a[0] += k * dx;
            a[1] += k * dy;
        }
    }
    a
}

/// A vacuum world at the cloud's size (every cell the cloud's corner cell: Σ_floor, below Σ_vac).
fn vacuum() -> State {
    vacuum_sized(64, 64)
}

/// A `w` × `h` vacuum world, every cell the cloud's corner cell.
fn vacuum_sized(w: usize, h: usize) -> State {
    let cloud = cloud();
    let (n, vacuum) = (w * h, (0..COMPONENTS).map(|c| cloud.get(c, 0)).collect::<Vec<_>>());
    let mut planes = vec![0.0; COMPONENTS * n];
    for c in 0..COMPONENTS {
        planes[c * n..(c + 1) * n].fill(vacuum[c]);
    }
    State::new(w, h, planes)
}

/// A vacuum world with a 5 × 3 blob of Σ = 4 on cells (10..15, 40..43), off-centre of the 64 × 64 world.
fn blob() -> State {
    let mut st = vacuum();
    let (w, n) = (st.width, st.cells());
    for y in 40..43 {
        for x in 10..15 {
            st.planes[y * w + x] = 4.0;
            st.planes[3 * n + y * w + x] = 4.0 * 0.1;
        }
    }
    st
}

#[test]
fn kick_drift_kick_pulls_by_the_softened_sum_over_the_gas_at_both_ends_of_the_step() {
    // A sink beside the blob: one step of Δt = 0.5 is v₀ + ½Δt (a(p₀) + a(p₁)) and p₀ + Δt (v₀ + ½Δt a(p₀)), the pull a
    // direct sum written here (r_acc = 2 for 50: max(2, 2·50/200²)).
    let st = blob();
    let mut s = sinks();
    let (p0, v0, dt) = ([20.5, 38.5], [0.3, 0.1], 0.5);
    s.sinks.push(sink(p0[0], p0[1], v0[0], v0[1], 50.0));
    let a0 = gas_pull(&st, p0, 2.0);
    assert!(a0[0].hypot(a0[1]) > 0.05, "the blob pulls: {a0:?}");
    let p1 = [p0[0] + dt * (v0[0] + 0.5 * dt * a0[0]), p0[1] + dt * (v0[1] + 0.5 * dt * a0[1])];
    let a1 = gas_pull(&st, p1, 2.0);
    s.advance(&st, dt);
    let k = s.sinks[0];
    let want = [(p1[0], p1[1]), (v0[0] + 0.5 * dt * (a0[0] + a1[0]), v0[1] + 0.5 * dt * (a0[1] + a1[1]))];
    assert!((k.x - want[0].0).abs() < 1e-12 && (k.y - want[0].1).abs() < 1e-12, "position ({}, {}) against {:?}", k.x, k.y, want[0]);
    assert!((k.vx - want[1].0).abs() < 1e-12 && (k.vy - want[1].1).abs() < 1e-12, "velocity ({}, {}) against {:?}", k.vx, k.vy, want[1]);
    // A cell below Σ_vac pulls nothing: the vacuum alone leaves a lone sink on its line.
    let mut s = sinks();
    s.sinks.push(sink(10.5, 20.5, 0.4, 0.1, 50.0));
    for _ in 0..40 {
        s.advance(&vacuum(), 0.25);
    }
    let k = s.sinks[0];
    assert!((k.x - 14.5).abs() < 1e-9 && (k.y - 21.5).abs() < 1e-9 && (k.vx - 0.4).abs() < 1e-15 && (k.vy - 0.1).abs() < 1e-15, "{k:?}");
}

#[test]
fn two_sinks_pull_each_other_softened_by_the_larger_r_acc_and_keep_their_total_momentum() {
    let st = vacuum();
    let mut s = sinks();
    s.sinks.push(sink(20.5, 30.5, 0.0, 0.1, 400.0));
    s.sinks.push(sink(40.5, 30.5, 0.0, -0.1, 400.0));
    let p0 = s.momentum();
    // The first half-kick from the softened pull: a = GM d / (d² + r_acc²)^{3/2}, r_acc = 2 here.
    let a = 400.0 * 20.0 / (20.0f64.powi(2) + 4.0).powf(1.5);
    let mut probe = sinks();
    probe.sinks = s.sinks.clone();
    probe.advance(&st, 1e-6);
    assert!((probe.sinks[0].vx / 1e-6 - a).abs() < 1e-3 * a && (probe.sinks[1].vx / 1e-6 + a).abs() < 1e-3 * a, "{} against {a}", probe.sinks[0].vx / 1e-6);
    for _ in 0..50 {
        s.advance(&st, 0.1);
    }
    let p1 = s.momentum();
    assert!((p1[0] - p0[0]).abs() < 1e-9 && (p1[1] - p0[1]).abs() < 1e-9, "momentum {p1:?} against {p0:?}");
    assert!(s.sinks[0].x > 20.5 && s.sinks[1].x < 40.5, "they fall toward each other");
    // Two sinks of r_acc 2 and 5 (400 and 100,000 at c_sb = 200): each is pulled softened by 5, the larger — M a is equal and opposite.
    let mut s = sinks();
    s.sinks.push(sink(20.5, 30.5, 0.0, 0.0, 400.0));
    s.sinks.push(sink(28.5, 30.5, 0.0, 0.0, 100_000.0));
    assert_eq!((s.r_acc(0), s.r_acc(1)), (2.0, 5.0));
    let a = s.pull(&st);
    let soft: f64 = 64.0 + 25.0;
    assert!(rel(a[0][0], 100_000.0 * 8.0 / soft.powf(1.5)) < 1e-12 && rel(a[1][0], -400.0 * 8.0 / soft.powf(1.5)) < 1e-12, "{a:?}");
    assert!((400.0 * a[0][0] + 100_000.0 * a[1][0]).abs() < 1e-12 * 400.0 * a[0][0].abs(), "M a, equal and opposite");
}

/// [M0-V7]'s bound, graded as G-GRAV2 is: |Σ_s M_s a_s + Σ_cells m g_s| ≤ 1e-5 (Σ_s M_s |a_s| + Σ_cells m |g_s|), a_s the
/// sink's acceleration from `Sinks::pull`, g_s the sinks' pull at a cell from `add_sinks` (P2's, the cells' side), over the
/// cells with Σ ≥ Σ_vac. Returns the ratio.
fn net_force_ratio(s: &Sinks, st: &State) -> f64 {
    let (w, h) = (st.width, st.height);
    let mut field = zero_field(w, h);
    add_sinks(&mut field, w, h, 1.0, &s.sources());
    let (mut net, mut scale) = ([0.0; 2], 0.0);
    for (sink, a) in s.sinks.iter().zip(s.pull(st)) {
        net[0] += sink.m * a[0];
        net[1] += sink.m * a[1];
        scale += sink.m * a[0].hypot(a[1]);
    }
    for i in 0..st.cells() {
        let m = st.get(0, i);
        if m < 1e-6 {
            continue;
        }
        net[0] += m * field.gx[i];
        net[1] += m * field.gy[i];
        scale += m * field.gx[i].hypot(field.gy[i]);
    }
    net[0].hypot(net[1]) / scale
}

#[test]
fn the_sinks_pull_on_the_gas_is_equal_and_opposite_on_asymmetric_worlds() {
    let mut worst = 0.0f64;
    let mut check = |name: &str, s: &Sinks, st: &State| {
        let ratio = net_force_ratio(s, st);
        println!("SR-SINK no net force: {name}: |Σ M a + Σ m g| / scale = {ratio:.3e}");
        assert!(ratio <= 1e-5, "{name}: {ratio:.3e}");
        worst = worst.max(ratio);
    };
    // One sink beside the off-centre blob, 2, 4, 8 and 16 cells off its edge (the V7 probe's distances).
    for d in [2.0, 4.0, 8.0, 16.0] {
        let mut s = sinks();
        s.sinks.push(sink(12.5 + d + 3.0, 41.5, 0.0, 0.0, 100.0));
        check(&format!("one sink {d} beside the blob"), &s, &blob());
    }
    // Two sinks of r_acc 2 and 5 beside it: each pulled softened by its own r_acc by the gas, the pair by the larger.
    let mut s = sinks();
    s.sinks.push(sink(24.5, 41.5, 0.0, 0.0, 400.0));
    s.sinks.push(sink(30.5, 30.5, 0.0, 0.0, 100_000.0));
    assert_eq!((s.r_acc(0), s.r_acc(1)), (2.0, 5.0));
    check("two sinks of r_acc 2 and 5 beside the blob", &s, &blob());
    // The cloud's corner, the sink off its cell's centre so the cells it takes are no symmetric disc: it accretes in the step it is graded, so the sum counts the cells accretion left — those it
    // took sit at Σ_floor, below Σ_vac, and pull nothing; the sum before accretion would not close.
    let mut st = cloud();
    let mut s = sinks();
    s.sinks.push(sink(13.2, 13.7, 0.0, 0.0, 2000.0));
    let stale = s.pull(&st);
    let taken = s.accrete(&mut st);
    assert!(taken > 3.0, "the sink takes the corner's gas: {taken}");
    check("the sink at the cloud's corner, after accretion", &s, &st);
    let fresh = s.pull(&st);
    let moved = (stale[0][0] - fresh[0][0]).hypot(stale[0][1] - fresh[0][1]) / fresh[0][0].hypot(fresh[0][1]);
    println!("SR-SINK no net force: accretion moves the sink's pull by {moved:.3e} of its value");
    assert!(moved > 1e-4, "the case is sensitive to the cells accretion took: {moved:.3e}");
    println!("SR-SINK no net force: worst ratio {worst:.3e}");
}

#[test]
fn sinks_closer_than_the_larger_r_acc_merge_with_mass_and_momentum_summed() {
    let mut s = sinks();
    s.sinks.push(sink(30.0, 30.0, 1.0, 0.0, 100.0));
    s.sinks.push(sink(31.5, 30.0, -1.0, 0.5, 300.0)); // 1.5 apart < r_acc = 2
    s.sinks.push(sink(50.0, 10.0, 0.2, 0.2, 40.0));
    let (mass0, p0) = (s.mass(), s.momentum());
    assert_eq!(s.merge(), 1);
    assert_eq!(s.sinks.len(), 2);
    let k = s.sinks[0];
    assert!(rel(k.m, 400.0) < 1e-12 && rel(k.x, 30.0 + 0.75 * 1.5) < 1e-12 && rel(k.vx, -0.5) < 1e-12 && rel(k.vy, 0.375) < 1e-12, "{k:?}");
    assert_eq!(s.sinks[1], sink(50.0, 10.0, 0.2, 0.2, 40.0), "the far one is untouched");
    assert!(rel(s.mass(), mass0) < 1e-12 && (s.momentum()[0] - p0[0]).abs() < 1e-9 && (s.momentum()[1] - p0[1]).abs() < 1e-9);
    // Exactly the larger r_acc apart does not merge; a chain does, until none is close: at 30, 31.9 and 32.5, r_acc 2.
    let mut s = sinks();
    s.sinks.push(sink(30.0, 30.0, 0.0, 0.0, 10.0));
    s.sinks.push(sink(32.0, 30.0, 0.0, 0.0, 10.0));
    assert_eq!(s.merge(), 0, "at exactly r_acc");
    let mut s = sinks();
    for x in [0.0, 1.9, 2.5] {
        s.sinks.push(sink(30.0 + x, 30.0, 0.0, 0.0, 10.0));
    }
    assert_eq!(s.merge(), 2, "30 and 31.9 merge at 30.95, which is 1.55 from the third");
    assert_eq!(s.sinks.len(), 1);
    assert!(rel(s.sinks[0].x, (10.0 * 30.0 + 10.0 * 31.9 + 10.0 * 32.5) / 30.0) < 1e-12, "{}", s.sinks[0].x);
}

// ---- P3 on the GPU (M0-T51) ----

/// A GPU world for P3: the state uploaded, φ uploaded when given, P2 and P4 when asked, the sinks set; one step at a time.
struct Rig {
    gpu: Gpu,
    state: GpuState,
    booking: Booking,
    side: SideFields,
    step: Step,
    w: usize,
    h: usize,
}

fn f32s(v: &[f64]) -> Vec<f32> {
    v.iter().map(|&x| x as f32).collect()
}

/// `st` rounded through f32, as the GPU holds it: the twin's start.
fn rounded(st: &State) -> State {
    State::new(st.width, st.height, st.planes.iter().map(|&v| f64::from(v as f32)).collect())
}

fn rig(st: &State, phi: Option<&[f64]>, gravity: bool, hydro: Option<EdgeMode>, sinks: &[Sink]) -> Rig {
    rig_with(&physics(), st, phi, gravity, hydro, sinks)
}

fn rig_with(physics: &Physics, st: &State, phi: Option<&[f64]>, gravity: bool, hydro: Option<EdgeMode>, sinks: &[Sink]) -> Rig {
    let gpu = super::device();
    let world = WorldConfig::new(st.width as u32, st.height as u32).unwrap();
    let state = GpuState::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &f32s(&st.planes)).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let side = SideFields::new(&gpu.device, world).unwrap();
    if let Some(phi) = phi {
        let bytes: Vec<u8> = f32s(phi).iter().flat_map(|v| v.to_le_bytes()).collect();
        gpu.queue.write_buffer(side.phi(), 0, &bytes);
    }
    let eos = EosGpu::new(&gpu.device, physics, &elements());
    let passes = Passes { mode: BoxMode::Fit, gravity: gravity.then_some(&side), hydro };
    let step = Step::with_sinks(&gpu.device, &state, &booking, &eos, physics, &passes, None, None, Some(&SinkSetup { side: &side })).unwrap();
    let set: Vec<GpuSink> = sinks.iter().map(|s| GpuSink { x: s.x, y: s.y, vx: s.vx, vy: s.vy, m: s.m }).collect();
    step.set_sinks(&gpu.queue, &set).unwrap();
    Rig { gpu, state, booking, side, step, w: st.width, h: st.height }
}

impl Rig {
    fn run(&self, steps: u32) -> SinkReport {
        self.step.run(&self.gpu.device, &self.gpu.queue, steps);
        let dt = self.step.read_dt(&self.gpu.device, &self.gpu.queue);
        assert!(dt.non_finite.is_none(), "non-finite state: {:?}", dt.non_finite);
        self.step.read_sinks(&self.gpu.device, &self.gpu.queue).unwrap()
    }

    /// One frame of `steps` steps, submitted and waited for (the latch's route, §1.8.5), and its report.
    fn frame(&self, steps: u32, slow_down: bool) -> FrameReport {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.step.encode_frame(&FramePlan { steps, slow_down }, &mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
        self.gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let reports = self.step.poll(&self.gpu.device);
        assert_eq!(reports.len(), 1, "one report per frame: {reports:?}");
        reports[0]
    }

    /// The cell state's bits.
    fn bits(&self) -> Vec<u32> {
        self.state.readback(&self.gpu.device, &self.gpu.queue).iter().map(|v| v.to_bits()).collect()
    }

    fn cells(&self) -> State {
        State::new(self.w, self.h, self.state.readback(&self.gpu.device, &self.gpu.queue).iter().map(|&v| f64::from(v)).collect())
    }

    /// Δt_n of the last step.
    fn dt(&self) -> f64 {
        f64::from(self.step.read_dt(&self.gpu.device, &self.gpu.queue).dt)
    }

    /// `energy.swallowed` booked since the rig was made.
    fn swallowed(&self) -> f64 {
        let at = TERMS.iter().position(|t| t.name == "energy.swallowed").unwrap();
        book_totals(self.booking.world(), &self.booking.readback(&self.gpu.device, &self.gpu.queue))[at]
    }

    /// The gas's own g, P2's gas field (φ left at 0).
    fn field(&self) -> Field {
        self.read_g(self.step.gravity_gas_field().expect("a step with P2"))
    }

    /// The g the gas feels, the sinks' pull included (P4 and Δt read it), with φ as P2 left it, sinks' potential included.
    fn total_field(&self) -> Field {
        let mut f = self.read_g(self.step.gravity_field().expect("a step with P2"));
        f.phi = read_back(&self.gpu.device, &self.gpu.queue, self.side.phi()).iter().map(|&v| f64::from(v)).collect();
        f
    }

    fn read_g(&self, buffer: &wgpu::Buffer) -> Field {
        let g = read_back(&self.gpu.device, &self.gpu.queue, buffer);
        let n = self.w * self.h;
        Field { phi: vec![0.0; n], gx: g[..n].iter().map(|&v| f64::from(v)).collect(), gy: g[n..].iter().map(|&v| f64::from(v)).collect() }
    }

    /// φ written by hand into the side fields, `sources`' potentials added as P2's `grav_sinks` would (a step without P2).
    fn set_phi(&self, phi: &Field, sources: &[SinkSource]) {
        let mut f = phi.clone();
        add_sinks(&mut f, self.w, self.h, 1.0, sources);
        let bytes: Vec<u8> = f32s(&f.phi).iter().flat_map(|v| v.to_le_bytes()).collect();
        self.gpu.queue.write_buffer(self.side.phi(), 0, &bytes);
    }
}

fn gpu_mass(r: &SinkReport) -> f64 {
    r.sinks.iter().map(|s| s.m).sum()
}

fn gpu_momentum(r: &SinkReport) -> [f64; 2] {
    r.sinks.iter().fold([0.0; 2], |p, s| [p[0] + s.m * s.vx, p[1] + s.m * s.vy])
}

/// The sinks' momentum before the step's kick-drift-kick: M v less M ½Δt (a_start + a_drifted), the pulls P3 kicked with
/// (a step that merged nothing).
fn unkicked(r: &SinkReport, dt: f64) -> [f64; 2] {
    assert_eq!(r.merged, 0, "{r:?}");
    let p = gpu_momentum(r);
    let kick = r.sinks.iter().zip(&r.pulls).fold([0.0; 2], |k, (s, [a0, a1])| [k[0] + s.m * 0.5 * dt * (a0[0] + a1[0]), k[1] + s.m * 0.5 * dt * (a0[1] + a1[1])]);
    [p[0] - kick[0], p[1] - kick[1]]
}

/// Each GPU sink against the twin's: mass and momentum to `tol` relative, position to `tol` cells.
fn same_sinks(name: &str, gpu: &SinkReport, twin: &Sinks, tol: f64) {
    assert_eq!(gpu.sinks.len(), twin.sinks.len(), "{name}: {:?} against {:?}", gpu.sinks, twin.sinks);
    for (k, (a, b)) in gpu.sinks.iter().zip(&twin.sinks).enumerate() {
        let scale = b.m * b.vx.hypot(b.vy).max(1e-3);
        let ok = rel(a.m, b.m) < tol
            && (a.m * a.vx - b.m * b.vx).abs() < tol * scale
            && (a.m * a.vy - b.m * b.vy).abs() < tol * scale
            && (a.x - b.x).abs() < tol.max(1e-5)
            && (a.y - b.y).abs() < tol.max(1e-5);
        assert!(ok, "{name}: sink {k}: GPU {a:?} against the twin's {b:?}");
    }
}

#[test]
fn gpu_accretion_matches_the_twin_keeps_mass_and_momentum_and_touches_nothing_beyond_r_acc() {
    let st = cloud();
    let (mass0, mom0) = grid_totals(&st);
    let at = sink(32.5, 32.5, 0.0, 0.0, 2000.0);
    let r = rig(&st, None, false, None, &[at]);
    let report = r.run(1);
    let after = r.cells();
    // The twin's P3 from the same start, the step's Δt, no field.
    let mut twin = self::sinks();
    twin.sinks.push(at);
    let mut st64 = st.clone();
    let tr = twin.step(&mut st64, &zero_field(st.width, st.height), r.dt());
    println!("SR-SINK gpu accretion: took {:.6} (twin {:.6}); sink {:?}; swallowed {:.6} (twin {:.6})", report.accreted, tr.accreted, report.sinks[0], r.swallowed(), twin.swallowed);
    assert!(report.formed.is_none() && report.merged == 0 && report.accreted > 10.0, "{report:?}");
    assert!(rel(report.accreted, tr.accreted) < 1e-6, "accreted {} against the twin's {}", report.accreted, tr.accreted);
    // The motion is the f32 sum over the cloud's cells, graded as [M0-V7] grades it (1e-5); the mass at 1e-6 above.
    same_sinks("accretion", &report, &twin, 1e-5);
    assert!(rel(r.swallowed(), twin.swallowed) < 1e-5, "swallowed {} against the twin's {}", r.swallowed(), twin.swallowed);
    // Grid + sink: the mass and the momentum as they were (G-SINK, 1e-6 relative), the sink's momentum before the gas's
    // pull kicked it (P3's motion; the gas's reaction is P4's).
    let (mass1, mom1) = grid_totals(&after);
    let (sm, sp) = (gpu_mass(&report), unkicked(&report, r.dt()));
    assert!(rel(mass1 + sm, mass0 + 2000.0) < 1e-6, "mass {} against {}", mass1 + sm, mass0 + 2000.0);
    assert!(rel(mom1[0] + sp[0], mom0[0]) < 1e-6 && rel(mom1[1] + sp[1], mom0[1]) < 1e-6, "momentum {mom1:?} + {sp:?} against {mom0:?}");
    // The same step with no sink: every cell beyond r_acc bit for bit, no Σ raised anywhere (nothing crosses r_acc outward).
    let bare = rig(&st, None, false, None, &[]);
    bare.run(1);
    let bare = bare.cells();
    let (w, ra) = (st.width, 2.0);
    for i in 0..st.cells() {
        let d = ((i % w) as f64 + 0.5 - 32.5).hypot((i / w) as f64 + 0.5 - 32.5);
        assert!(after.get(0, i) <= bare.get(0, i), "Σ raised at cell {i}: {} against {} (sink run {:?}, bare {:?})", after.get(0, i), bare.get(0, i), (0..4).map(|c| after.get(c, i)).collect::<Vec<_>>(), (0..4).map(|c| bare.get(c, i)).collect::<Vec<_>>());
        if d > ra {
            assert!((0..COMPONENTS).all(|c| after.get(c, i).to_bits() == bare.get(c, i).to_bits()), "cell {i} (r = {d:.3}) touched beyond r_acc");
        }
    }
    // The same step again: the same bits (§1.3.4).
    let again = rig(&st, None, false, None, &[at]);
    assert_eq!(again.run(1), report, "a second run's sinks");
    assert!(again.cells().planes.iter().zip(&after.planes).all(|(a, b)| a.to_bits() == b.to_bits()), "a second run's cells");
}

#[test]
fn gpu_accretion_takes_bound_gas_only_in_the_sinks_own_frame() {
    let n = cloud().cells();
    let mut hot = cloud();
    hot.planes[3 * n..4 * n].iter_mut().for_each(|e| *e *= 1e5);
    let r = rig(&hot, None, false, None, &[sink(32.5, 32.5, 0.0, 0.0, 2000.0)]);
    let report = r.run(1);
    assert!(report.accreted == 0.0 && report.sinks[0].m == 2000.0 && r.swallowed() == 0.0, "hot gas taken: {report:?}");
    let mut fast = cloud();
    for i in 0..n {
        fast.planes[n + i] *= 300.0;
        fast.planes[2 * n + i] *= 300.0;
        fast.planes[3 * n + i] += 0.5 * (fast.planes[n + i].powi(2) + fast.planes[2 * n + i].powi(2)) / fast.planes[i];
    }
    let report = rig(&fast, None, false, None, &[sink(32.5, 32.5, 0.0, 0.0, 2000.0)]).run(1);
    assert_eq!(report.accreted, 0.0, "gas at 108 cells per t.u. is unbound to a sink at rest");
    // A sink riding with the gas takes it, as the twin's does.
    let riding = sink(32.5, 32.5, 90.0, -60.0, 2000.0);
    let r = rig(&fast, None, false, None, &[riding]);
    let report = r.run(1);
    let mut twin = self::sinks();
    twin.sinks.push(riding);
    let tr = twin.step(&mut rounded(&fast), &zero_field(64, 64), r.dt());
    println!("SR-SINK gpu frame: riding sink took {:.6} (twin {:.6})", report.accreted, tr.accreted);
    assert!(report.accreted > 10.0 && rel(report.accreted, tr.accreted) < 1e-6, "{report:?} against {tr:?}");
}

/// Two sinks whose r_acc both reach cells (32, 32) and (33, 32): each cell goes to the first sink, once.
#[test]
fn gpu_two_sinks_over_one_cell_take_it_once() {
    let st = cloud();
    let (mass0, mom0) = grid_totals(&st);
    let pair = [sink(31.5, 32.5, 0.0, 0.0, 1.0), sink(34.5, 32.5, 0.0, 0.0, 1.0)];
    let r = rig(&st, None, false, None, &pair);
    let report = r.run(1);
    let mut twin = self::sinks();
    twin.sinks.extend(pair);
    let tr = twin.step(&mut st.clone(), &zero_field(64, 64), r.dt());
    println!("SR-SINK gpu shared cells: sinks {:?} (twin {:?})", report.sinks, twin.sinks);
    assert_eq!((report.merged, tr.merged), (0, 0));
    // 13 cells to the first, 11 to the second (the two shared went to the first): about 13 and 11 of Σ = 1 each.
    assert!((report.sinks[0].m - 14.0).abs() < 1e-3 && (report.sinks[1].m - 12.0).abs() < 1e-3, "{:?}", report.sinks);
    same_sinks("shared cells", &report, &twin, 1e-6);
    let (mass1, mom1) = grid_totals(&r.cells());
    let (sm, sp) = (gpu_mass(&report), unkicked(&report, r.dt()));
    assert!(rel(mass1 + sm, mass0 + 2.0) < 1e-6, "mass {} against {}: a cell taken twice", mass1 + sm, mass0 + 2.0);
    assert!(rel(mom1[0] + sp[0], mom0[0]) < 1e-6 && rel(mom1[1] + sp[1], mom0[1]) < 1e-6, "momentum {mom1:?} + {sp:?} against {mom0:?}");
}

/// The formation worlds at §2.8's smallest size, 64 × 64.
fn world64(wells: &[(usize, usize, f64)], sign: f64, offset: f64) -> (State, Vec<f64>) {
    let (st, phi) = world(64, 64, wells, sign, offset);
    (rounded(&st), phi.iter().map(|&v| f64::from(v as f32)).collect())
}

#[test]
fn gpu_formation_matches_the_twin_where_all_four_criteria_hold() {
    let (st, phi) = world64(&[(32, 32, 7000.0)], -1.0, 0.0);
    let (mass0, _) = grid_totals(&st);
    let r = rig(&st, Some(&phi), false, None, &[]);
    let report = r.run(1);
    let mut twin = self::sinks();
    let tr = twin.step(&mut st.clone(), &Field { phi: phi.clone(), gx: vec![0.0; 4096], gy: vec![0.0; 4096] }, r.dt());
    println!("SR-SINK gpu formation: {report:?}; twin {tr:?}, {:?}; swallowed {:.4} (twin {:.4})", twin.sinks, r.swallowed(), twin.swallowed);
    assert_eq!(report.formed, Some((32, 32)));
    assert_eq!(tr.formed, Some((32, 32)));
    // The new sink's mass at 1e-6; its motion, the f32 sum over the bowl's cells, at [M0-V7]'s 1e-5.
    assert!(rel(report.sinks[0].m, twin.sinks[0].m) < 1e-6, "{report:?}");
    same_sinks("formation", &report, &twin, 1e-5);
    assert!(rel(r.swallowed(), twin.swallowed) < 1e-5, "swallowed {} against {}", r.swallowed(), twin.swallowed);
    assert!(rel(grid_totals(&r.cells()).0 + gpu_mass(&report), mass0) < 1e-6, "mass");
    // The cell is left at the floor; a second step forms nothing more (no other candidate).
    assert!(rel(r.cells().get(0, 32 * 64 + 32), physics().get("sigma_floor").unwrap()) < 1e-6);
    assert_eq!(r.run(1).formed, None);
}

#[test]
fn gpu_formation_refuses_each_criterion_alone() {
    let bh = physics().get("sigma_bh").unwrap();
    let refuse = |name: &str, (st, phi): (State, Vec<f64>)| {
        let report = rig(&st, Some(&phi), false, None, &[]).run(1);
        assert!(report.formed.is_none() && report.sinks.is_empty(), "{name}: {report:?}");
    };
    refuse("Σ just below Σ_BH", world64(&[(32, 32, bh * (1.0 - 1e-6))], -1.0, 0.0));
    refuse("∇·u > 0", world64(&[(32, 32, 7000.0)], 1.0, 0.0));
    let (st, mut phi) = world64(&[(32, 32, 7000.0)], -1.0, 0.0);
    phi[31 * 64 + 33] = phi[32 * 64 + 32] - 1.0;
    refuse("a lower neighbour", (st, phi));
    refuse("½|u|² + ε_th + φ = 0.01", world64(&[(32, 32, 7000.0)], -1.0, 100.0 - 0.1 + 0.01));
    // Each refusal is the one criterion: the threshold met and just inside boundness, it forms.
    for (name, w) in [("Σ = Σ_BH", world64(&[(32, 32, bh)], -1.0, 0.0)), ("bound by 0.01", world64(&[(32, 32, 7000.0)], -1.0, 100.0 - 0.1 - 0.01))] {
        let report = rig(&w.0, Some(&w.1), false, None, &[]).run(1);
        assert_eq!(report.formed, Some((32, 32)), "{name}");
    }
}

#[test]
fn gpu_the_densest_forms_first_ties_the_lowest_y_then_x_one_a_step_none_past_16() {
    let first = |wells: &[(usize, usize, f64)]| {
        let (st, phi) = world64(wells, -1.0, 0.0);
        rig(&st, Some(&phi), false, None, &[]).run(1).formed
    };
    assert_eq!(first(&[(50, 10, 7000.0), (12, 40, 7000.0)]), Some((50, 10)), "ties: the lowest y");
    assert_eq!(first(&[(50, 10, 7000.0), (12, 10, 7000.0)]), Some((12, 10)), "ties: then the lowest x");
    assert_eq!(first(&[(12, 10, 7000.0), (50, 40, 8000.0)]), Some((50, 40)), "the densest");
    // One a step: the second candidate forms on the next, as the twin's — the bowls steepened ×100, so the first sink's
    // potential (§1.4.3: a pull of 6 200/48² ≈ 2.7 per cell at the second well) leaves the second its minimum.
    let (st, phi) = world64(&[(50, 10, 7000.0), (12, 40, 7000.0)], -1.0, 0.0);
    let phi: Vec<f64> = phi.iter().map(|&v| f64::from((100.0 * v) as f32)).collect();
    let r = rig(&st, Some(&phi), false, None, &[]);
    let field = Field { phi, gx: vec![0.0; 4096], gy: vec![0.0; 4096] };
    let (mut twin, mut st64) = (self::sinks(), st.clone());
    for expect in [Some((50, 10)), Some((12, 40)), None] {
        // No P2 here: φ is rewritten each step with the sinks' potential, as `grav_sinks` leaves it.
        r.set_phi(&field, &twin.sources());
        let report = r.run(1);
        let tr = twin.step(&mut st64, &field, r.dt());
        assert_eq!(report.formed, expect.map(|(x, y)| (x as u32, y as u32)));
        assert_eq!(tr.formed, expect);
        same_sinks("one a step", &report, &twin, 1e-5);
    }
    // At the shallow bowls the first sink's potential takes that minimum away: neither forms the second.
    let (st, phi) = world64(&[(50, 10, 7000.0), (12, 40, 7000.0)], -1.0, 0.0);
    let r = rig(&st, Some(&phi), false, None, &[]);
    let field = Field { phi: phi.clone(), gx: vec![0.0; 4096], gy: vec![0.0; 4096] };
    let (mut twin, mut st64) = (self::sinks(), st.clone());
    for _ in 0..2 {
        r.set_phi(&field, &twin.sources());
        assert_eq!(r.run(1).formed.map(|(x, y)| (x as usize, y as usize)), twin.step(&mut st64, &field, r.dt()).formed);
    }
    assert_eq!(twin.sinks.len(), 1, "the second well, beside a sink of 6 200, is not a minimum");
    // 16 sinks stand (off the world, light): none forms; 15: the sixteenth does.
    let (st, phi) = world64(&[(32, 32, 7000.0)], -1.0, 0.0);
    let far: Vec<Sink> = (0..MAX_SINKS).map(|k| sink(100.0 + 10.0 * k as f64, 1.0, 0.0, 0.0, 1.0)).collect();
    assert_eq!(rig(&st, Some(&phi), false, None, &far).run(1).formed, None, "16 sinks stand");
    let report = rig(&st, Some(&phi), false, None, &far[..15]).run(1);
    assert_eq!((report.formed, report.sinks.len()), (Some((32, 32)), 16), "15 stand: the sixteenth forms");
}

/// Kick-drift-kick by the sum over the cloud's cells (§0.5 [M0-V7]), the sinks' pull softened by the pair's larger r_acc
/// and merging, step by step against the twin's `advance` given the GPU's Δt: light sinks in the gas (m = 0.01 binds
/// nothing: ε_th = 0.1 > GM/½), a pair 1.5 apart that merges on the first step, and a heavy pair in the vacuum pulling on
/// each other.
#[test]
fn gpu_kick_drift_kick_and_merging_match_the_twin_in_the_gas() {
    let st = cloud();
    let start = [
        sink(20.5, 24.5, 0.3, 0.1, 0.01),
        sink(22.0, 24.5, -0.2, 0.0, 0.03),
        sink(40.25, 37.75, -0.1, 0.4, 0.01),
        sink(5.5, 6.5, 0.0, 0.0, 400.0),
        sink(58.5, 6.5, 0.0, 0.0, 400.0),
    ];
    let r = rig(&st, None, true, None, &start);
    let mut twin = self::sinks();
    twin.sinks.extend(start);
    for step in 0..6 {
        let report = r.run(1);
        let dt = r.dt();
        twin.advance(&st, dt);
        let merged = twin.merge();
        assert_eq!(report.accreted, 0.0, "step {step}: {report:?}");
        assert_eq!(report.merged as usize, merged, "step {step}");
        same_sinks(&format!("step {step}"), &report, &twin, 1e-5);
        println!("SR-SINK gpu motion: step {step}, Δt {dt:.5}, {} sinks, first {:?}", report.sinks.len(), report.sinks[0]);
    }
    assert_eq!(twin.sinks.len(), 4, "the pair merged");
    assert!(twin.sinks[2].x > 5.5 + 1e-3 && twin.sinks[3].x < 58.5 - 1e-3, "the heavy pair falls together: {:?}", twin.sinks);
}

/// P2, P3 and P4 together, a bounce world: the cloud falls on a sink at its centre (the gas's own gravity and, since
/// M0-T52a, the sink's pull on it), grid + sink mass constant to 1e-6 each step and the sink only ever gaining. From step 32 a
/// dense cell faces the hole the sink dug with a ≥ 4 × denser cell behind it — the f32 face Σ P4 predicts there was exactly
/// 0 and wrote NaN until M0-D44.
#[test]
fn gpu_the_cloud_falls_into_a_sink_with_mass_conserved_each_step() {
    let st = cloud();
    let r = rig(&st, None, true, Some(EdgeMode::Bounce), &[sink(32.5, 32.5, 0.0, 0.0, 2000.0)]);
    let total = grid_totals(&st).0 + 2000.0;
    let (p0, scale) = (grid_totals(&st).1, (0..st.cells()).map(|i| st.get(1, i).hypot(st.get(2, i))).sum::<f64>());
    let (mut last, mut p) = (2000.0, [0.0; 2]);
    for step in 0..40 {
        let report = r.run(1);
        let cells = r.cells();
        let now = grid_totals(&cells).0 + gpu_mass(&report);
        assert!(report.formed.is_none() && report.merged == 0, "step {step}: {report:?}");
        assert!(rel(now, total) < 1e-6, "step {step}: grid + sink mass {now} against {total}");
        assert!(gpu_mass(&report) >= last, "step {step}: the sink lost mass");
        last = gpu_mass(&report);
        let (gp, sp) = (grid_totals(&cells).1, gpu_momentum(&report));
        p = [gp[0] + sp[0], gp[1] + sp[1]];
    }
    let off = (p[0] - p0[0]).hypot(p[1] - p0[1]);
    println!("SR-SINK gpu fall: after 40 steps the sink holds {last:.4} (2000 + {:.4}); grid + sink momentum ({:.4}, {:.4}) from ({:.4}, {:.4}), {:.2} % of Σ m|u| = {scale:.2}", last - 2000.0, p[0], p[1], p0[0], p0[1], 100.0 * off / scale);
    assert!(last > 2000.0 + 10.0, "the cloud reached the sink");
}

/// [M0-V7]'s bound on the GPU: one step of P3 over `st` with `start` standing, then |Σ_s M_s a_s + Σ_cells m g_s| ≤ 1e-5
/// (Σ_s M_s |a_s| + Σ_cells m |g_s|) — a_s the pull of P3's closing half-kick, summed in f32 over the box's tiles at the
/// drifted position on the gas accretion left, g_s each sink's softened pull at a cell in f64 from the sinks P3 left, over
/// the cells with Σ ≥ Σ_vac. Also against the twin's `pull` from the same sinks and cells. Returns the ratio.
fn gpu_net_force(name: &str, st: &State, start: &[Sink]) -> f64 {
    let sigma_vac = physics().get("sigma_vac").unwrap();
    let (w, h) = (st.width, st.height);
    let r = rig(st, None, false, None, start);
    let report = r.run(1);
    assert_eq!((report.formed, report.merged, report.sinks.len()), (None, 0, start.len()), "{name}: {report:?}");
    let cells = r.cells();
    let mut twin = self::sinks();
    twin.sinks = report.sinks.iter().map(|s| sink(s.x, s.y, s.vx, s.vy, s.m)).collect();
    let mut field = zero_field(w, h);
    add_sinks(&mut field, w, h, 1.0, &twin.sources());
    let (mut net, mut scale, mut gap) = ([0.0; 2], 0.0, 0.0f64);
    for ((s, [_, a]), want) in report.sinks.iter().zip(&report.pulls).zip(twin.pull(&cells)) {
        net[0] += s.m * a[0];
        net[1] += s.m * a[1];
        scale += s.m * a[0].hypot(a[1]);
        gap = gap.max((a[0] - want[0]).hypot(a[1] - want[1]) / want[0].hypot(want[1]));
    }
    for i in (0..w * h).filter(|&i| cells.get(0, i) >= sigma_vac) {
        let m = cells.get(0, i);
        net[0] += m * field.gx[i];
        net[1] += m * field.gy[i];
        scale += m * field.gx[i].hypot(field.gy[i]);
    }
    let ratio = net[0].hypot(net[1]) / scale;
    println!("SR-SINK gpu no net force: {name}: |Σ M a + Σ m g| / scale = {ratio:.3e}; the pull against the twin's, {gap:.3e} relative");
    assert!(ratio <= 1e-5, "{name}: {ratio:.3e}");
    assert!(gap <= 1e-5, "{name}: the GPU's pull {:?} against the twin's {:?}", report.pulls, twin.pull(&cells));
    ratio
}

#[test]
fn gpu_the_sinks_pull_on_the_gas_is_equal_and_opposite_on_asymmetric_worlds() {
    let mut worst = 0.0f64;
    // M0-D45's worlds: one sink 2, 4, 8 and 16 cells off the blob's edge (the last outside the box), two of r_acc 2 and 5.
    for d in [2.0, 4.0, 8.0, 16.0] {
        worst = worst.max(gpu_net_force(&format!("one sink {d} beside the blob"), &blob(), &[sink(12.5 + d + 3.0, 41.5, 0.0, 0.0, 100.0)]));
    }
    let pair = [sink(24.5, 41.5, 0.0, 0.0, 400.0), sink(30.5, 30.5, 0.0, 0.0, 100_000.0)];
    worst = worst.max(gpu_net_force("two sinks of r_acc 2 and 5 beside the blob", &blob(), &pair));
    // The sink at the cloud's corner accretes in the step it is graded: a sum over the gas before accretion would count the
    // cells it took (the twin: 3.6e-2 of the pull), and the step's velocity against the twin's `step` catches a first
    // half-kick read before accretion.
    let corner = sink(13.2, 13.7, 0.0, 0.0, 2000.0);
    worst = worst.max(gpu_net_force("the sink at the cloud's corner, after accretion", &cloud(), &[corner]));
    let r = rig(&cloud(), None, false, None, &[corner]);
    let report = r.run(1);
    let mut twin = self::sinks();
    twin.sinks.push(corner);
    let tr = twin.step(&mut cloud(), &zero_field(64, 64), r.dt());
    assert!(report.accreted > 3.0 && rel(report.accreted, tr.accreted) < 1e-6, "{report:?} against {tr:?}");
    same_sinks("the corner's step", &report, &twin, 1e-5);
    println!("SR-SINK gpu no net force: worst ratio {worst:.3e}; the corner's sink {:?} (twin {:?})", report.sinks[0], twin.sinks[0]);
}

/// §0.5 [M0-V7]'s sum reaches a sink outside the box (P2 writes g over the box only, so a grid read there was stale or 0,
/// M0-V7 § A sink outside the box): a 128 × 128 world, an 8 × 8 blob of Σ = 1 on cells 8–15, a light sink at (100.5, 100.5)
/// and at (20.5, 20.5) inside the box, one step of P2 + P3 against the twin's `step` from the same start.
#[test]
fn gpu_a_sink_outside_the_box_moves_as_the_twins() {
    let mut st = vacuum_sized(128, 128);
    let (w, n) = (st.width, st.cells());
    for y in 8..16 {
        for x in 8..16 {
            st.planes[y * w + x] = 1.0;
            st.planes[3 * n + y * w + x] = 0.1;
        }
    }
    for (name, at, outside) in [("outside the box", 100.5, true), ("inside the box", 20.5, false)] {
        let start = sink(at, at, 0.0, 0.0, 1e-3);
        let r = rig(&st, None, true, None, &[start]);
        let report = r.run(1);
        let b = r.step.read_box(&r.gpu.device, &r.gpu.queue);
        assert_eq!((b.x0, b.y0, b.w, b.h), (0, 0, 24, 24), "the blob's box");
        assert_eq!(at > f64::from(b.x0 + b.w), outside, "{name}");
        let mut twin = self::sinks();
        twin.sinks.push(start);
        twin.step(&mut rounded(&st), &zero_field(128, 128), r.dt());
        let (g, t) = (report.sinks[0], twin.sinks[0]);
        println!("SR-SINK gpu {name}: velocity ({:.6e}, {:.6e}), the twin's ({:.6e}, {:.6e})", g.vx, g.vy, t.vx, t.vy);
        assert!(t.vx < -1e-4 && t.vy < -1e-4, "{name}: the blob pulls the twin's sink: {t:?}");
        assert!((g.vx - t.vx).hypot(g.vy - t.vy) <= 1e-5 * t.vx.hypot(t.vy), "{name}: GPU {g:?} against the twin's {t:?}");
        same_sinks(name, &report, &twin, 1e-5);
    }
}


// ---- the sinks' pull in P2, the latch on a formation (M0-T52a) ----

/// A vacuum world at the cloud's size with a 2 × 2 test mass (Σ = 10⁻³, above Σ_vac, at ε_th = 0.1 as the formation bowls,
/// the vacuum's fractions, at rest) on cells (20..22, 20..22): too light to pull on anything, so what its cells feel is the sinks'.
fn test_mass() -> State {
    let mut st = vacuum();
    let (w, n) = (st.width, st.cells());
    for (x, y) in [(20, 20), (21, 20), (20, 21), (21, 21)] {
        let i = y * w + x;
        st.planes[i] = 1e-3;
        st.planes[3 * n + i] = 1e-3 * 0.1;
    }
    st
}

/// One step of P2 + P3 over `st` with `start` standing, against the twin's φ and g with the sinks added
/// (`Gravity::field_with_sinks`) and the analytic pull: over every cell the gas fills, the g the gas feels less the gas's
/// own g is −G M d / (r² + r_acc²)^{3/2} summed over the sinks, and φ is the twin's φ with −GM/√(r² + r_acc²) — both, so
/// that a pull missing from φ (the ledger's W, the virial) or from g (the motion) is a failure on its own. The gas's own
/// g is the sinkless run's, bit for bit: P3 reads the gas's pull alone, never a sink's own. Returns the largest pull read.
fn check_pull(name: &str, st: &State, start: &[Sink]) -> f64 {
    let physics = physics();
    let (w, h) = (st.width, st.height);
    let r = rig(st, None, true, None, start);
    r.run(1);
    let (total, gas) = (r.total_field(), r.field());
    let mut twin = self::sinks();
    twin.sinks.extend(start.iter().copied());
    let sources = twin.sources();
    let sigma_vac = physics.get("sigma_vac").unwrap();
    let want = Gravity::new(1.0).field_with_sinks(st.plane(0), sigma_vac, w, h, 1.0, &sources).unwrap();
    let bare = rig(st, None, true, None, &[]);
    bare.run(1);
    let bare = bare.field();
    let (mut checked, mut largest) = (0, 0.0f64);
    for i in (0..w * h).filter(|&i| st.get(0, i) >= sigma_vac) {
        let (cx, cy) = ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5);
        let mut pull = [0.0f64; 2];
        for src in &sources {
            let (dx, dy) = (cx - src.x, cy - src.y);
            let soft = dx * dx + dy * dy + src.r_acc * src.r_acc;
            pull[0] -= src.m * dx / (soft * soft.sqrt());
            pull[1] -= src.m * dy / (soft * soft.sqrt());
        }
        let scale = pull[0].hypot(pull[1]);
        largest = largest.max(scale);
        let got = [total.gx[i] - gas.gx[i], total.gy[i] - gas.gy[i]];
        assert!(
            (got[0] - pull[0]).abs() <= 1e-5 * scale + 1e-4 && (got[1] - pull[1]).abs() <= 1e-5 * scale + 1e-4,
            "{name}: cell {i}: the sinks' pull {got:?} against −GM d/(r² + r_acc²)^(3/2) = {pull:?}"
        );
        assert!(
            (total.phi[i] - want.phi[i]).abs() <= 1e-5 * want.phi[i].abs() + 1e-3,
            "{name}: cell {i}: φ {} against the twin's, sinks' potential included, {}",
            total.phi[i],
            want.phi[i]
        );
        assert!(gas.gx[i].to_bits() == bare.gx[i].to_bits() && gas.gy[i].to_bits() == bare.gy[i].to_bits(), "{name}: cell {i}: the gas's g moved with a sink standing");
        checked += 1;
    }
    println!("SR-SINK pull {name}: {checked} cells, the largest sink pull {largest:.3}");
    assert!(checked >= 4 && largest > 1.0, "{name}: the case reads cells ({checked}) and a pull ({largest})");
    largest
}

#[test]
fn gpu_a_test_mass_near_a_sink_feels_the_softened_pull_and_phi_carries_the_same_term() {
    // M_s = 2000 sits at r_acc = 2 (the floor); 60 000 at r_acc = 2GM/c_sb² = 3 — the formula's own branch.
    check_pull("test mass, one sink", &test_mass(), &[sink(30.5, 20.5, 0.0, 0.0, 2000.0)]);
    check_pull("test mass, two sinks", &test_mass(), &[sink(30.5, 20.5, 0.0, 0.0, 2000.0), sink(8.5, 33.5, 0.0, 0.0, 60_000.0)]);
}

#[test]
fn gpu_the_sinks_pull_reaches_the_cloud_in_g_and_phi_and_the_gas_g_stays_the_gas_own() {
    // The cloud's cells: the sinks' pull is added over the whole box, the gas's g bit for bit the sinkless step's.
    let far = [sink(58.5, 8.5, 0.0, 0.0, 2000.0), sink(5.5, 58.5, 0.0, 0.0, 60_000.0)];
    check_pull("cloud", &cloud(), &far);
}

#[test]
fn gpu_a_formation_inside_a_frame_stops_it_with_the_slow_down_on_and_only_a_formation() {
    let (st, phi) = world64(&[(32, 32, 7000.0)], -1.0, 0.0);
    let start = |slow_down: bool, steps: u32| {
        let r = rig(&st, Some(&phi), false, None, &[]);
        let report = r.frame(steps, slow_down);
        (r, report)
    };
    // Slow-down on: step 1 forms the sink and the frame stops there — one step run, the state and the sink those of a
    // one-step run, the next frame's first step runs whole.
    let (stopped, report) = start(true, 10);
    assert_eq!((report.steps_run, report.latch_fired), (1, Some(1)), "{report:?}");
    let one = rig(&st, Some(&phi), false, None, &[]);
    let one_sinks = one.run(1);
    assert_eq!(one_sinks.formed, Some((32, 32)));
    assert_eq!(stopped.bits(), one.bits(), "the stopped frame's cells are a one-step run's");
    assert_eq!(stopped.step.read_sinks(&stopped.gpu.device, &stopped.gpu.queue).unwrap(), one_sinks);
    let again = stopped.frame(3, true);
    assert_eq!((again.steps_run, again.latch_fired), (3, None), "the latch is disarmed for 1.0 t.u. after it fired, then the frame runs: {again:?}");
    // Slow-down off: the same formation, every step of the frame runs.
    let (through, report) = start(false, 10);
    assert_eq!((report.steps_run, report.latch_fired), (10, None), "{report:?}");
    assert_eq!(through.step.read_sinks(&through.gpu.device, &through.gpu.queue).unwrap().sinks.len(), 1, "the sink formed");
    // No formation, no latch: sinks accreting through a whole frame never set it (the cloud, a sink at its centre).
    let r = rig(&cloud(), None, false, None, &[sink(32.5, 32.5, 0.0, 0.0, 2000.0)]);
    let report = r.frame(10, true);
    assert_eq!((report.steps_run, report.latch_fired), (10, None), "accretion is not a formation: {report:?}");
    assert!(r.step.read_sinks(&r.gpu.device, &r.gpu.queue).unwrap().accreted > 0.0 || r.step.read_sinks(&r.gpu.device, &r.gpu.queue).unwrap().sinks[0].m > 2000.0);
}

#[test]
fn gpu_a_neutronized_core_holding_m_tov_sb_forms_a_sink_from_mass_as_the_twin() {
    let phys = shipped();
    let m_tov = mass_sinks().m_tov();
    let world = |mass: f64, species: &str, sign: f64| {
        let (st, phi) = core_world(64, 64, (32, 32), mass * m_tov, species, sign);
        (rounded(&st), phi.iter().map(|&v| f64::from(v as f32)).collect::<Vec<f64>>())
    };
    let (st, phi) = world(1.05, "n", -1.0);
    let (mass0, _) = grid_totals(&st);
    let r = rig_with(&phys, &st, Some(&phi), false, None, &[]);
    let report = r.run(1);
    let mut twin = mass_sinks();
    let tr = twin.step(&mut st.clone(), &Field { phi: phi.clone(), gx: vec![0.0; 4096], gy: vec![0.0; 4096] }, r.dt());
    println!("SR-SINK gpu mass formation: {report:?}; twin {tr:?}, {:?}; swallowed {:.4} (twin {:.4})", twin.sinks, r.swallowed(), twin.swallowed);
    assert_eq!((report.formed, tr.formed), (Some((32, 32)), Some((32, 32))));
    assert!(rel(report.sinks[0].m, twin.sinks[0].m) < 1e-6 && rel(report.sinks[0].m, 1.05 * m_tov) < 1e-5, "{report:?}");
    same_sinks("mass formation", &report, &twin, 1e-5);
    assert!(rel(r.swallowed(), twin.swallowed) < 1e-5, "swallowed {} against {}", r.swallowed(), twin.swallowed);
    let cells = r.cells();
    assert!(rel(grid_totals(&cells).0 + gpu_mass(&report), mass0) < 1e-6, "mass");
    let floor = phys.get("sigma_floor").unwrap();
    let core: Vec<usize> = disk(32, 32, R_CORE, 64, 64).collect();
    assert!(core.iter().all(|&i| rel(cells.get(0, i), floor) < 1e-6), "the disk left at the floor");
    assert!((0..4096).filter(|i| !core.contains(i)).all(|i| cells.get(0, i) == st.get(0, i)), "nothing beyond the disk taken");
    // Refused: under m_tov_sb, iron, diverging — the world untouched.
    for (name, mass, species, sign) in [("0.95 m_tov_sb", 0.95, "n", -1.0), ("iron, Y_e 0.464", 1.05, "Fe", -1.0), ("∇·u > 0", 1.05, "n", 1.0)] {
        let (st, phi) = world(mass, species, sign);
        let r = rig_with(&phys, &st, Some(&phi), false, None, &[]);
        let report = r.run(1);
        assert!(report.formed.is_none() && report.sinks.is_empty(), "{name}: {report:?}");
        // The step's other passes (P8) may move E; no cell of the disk gave its mass.
        let cells = r.cells();
        assert!(core.iter().all(|&i| cells.get(0, i) == st.get(0, i)), "{name}: a cell of the disk was taken");
    }
    // Σ ≥ Σ_BH alone, hydrogen: the cell alone, as §2.6.
    let (st, phi) = world64(&[(32, 32, 7000.0)], -1.0, 0.0);
    let report = rig_with(&phys, &st, Some(&phi), false, None, &[]).run(1);
    assert_eq!(report.formed, Some((32, 32)));
    // Formation took the cell alone; the same step's accretion then took the bound cells within r_acc = 2 (12 × 100), not
    // the disk r ≤ 4 (48 × 100).
    assert!(rel(report.sinks[0].m - report.accreted, 7000.0 - floor) < 1e-6 && report.accreted < 1300.0, "the cell alone: {report:?}");
}
