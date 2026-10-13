//! M0-T41 — heat and light on the CPU twin (contract §2.4, §1.7, §5.1's G-HEAT1, G-HEAT2 and G-FLD, R8, R13): P5's f64
//! reference, `sr_physics::reference::heat`, one-temperature flux-limited diffusion stepped by RKL2. No adapter is touched;
//! the GPU's cases join this file with M0-T42, the same oracles on `step/heat.rs`.
//!
//! Three scopes, one module each. `gauss` — G-HEAT1: a Gaussian heat pulse in a uniform medium whose diffusivity is the
//! scene's `test.constant_diffusivity` (scenes/heat_pulse.json), the variance per axis σ²(t) = σ₀² + 2χt within 1 % (SM-O7,
//! SM-E21) and the total heat constant to 1e-5; the pass's F at a face against −χ·C·ΔT; the light leaving (a hot slab beside
//! vacuum and at the world's edge: what the pass books `radiated` equals the heat the grid lost, to 1e-6, and sits in the
//! cells that emitted it); the scene's `pin_temperature`. `rkl2` — G-HEAT2: the same pulse in one RKL2 pass against explicit
//! sub-steps written here (σ² within 1 %, SM-O8, SM-E18), the stage count against the contract's formula worked by hand,
//! `SR-WARN heat stages` above 32, and a one-cell pulse in many passes at a step just over the stability limit of one
//! stage fewer (the adversarial case: a smooth pulse has no content at the grid's highest wavenumbers, so a missing stage
//! leaves its σ² alone and only a sharp one shows it). `limiter` — G-FLD: thin gas (Σ = 10⁻⁴, scenes/heat_spot.json) around
//! a hot spot, |F| ≤ c_sb a_r T⁴ (1 + 10⁻⁶) at every face (SM-E19); thick gas, where the limiter must not act, carries the
//! radiative conductivity (4/3) a_r c_sb T³/(κΣ) (SM-E20); the scene's `flux_limiter` off gives λ ≡ 1/3.
//!
//! The oracles are written here from the contract, never read from the scheme: temperatures from E with the cold pressure
//! off and the gas at rest and pure hydrogen (μ = 1/2), the explicit stepping, the moments, the closed forms.

use std::path::{Path, PathBuf};

use sr_engine::scene::Scene;
use sr_physics::reference::heat::{lambda, stages_for, Heat, Pass, TestFlags, MAX_STAGES};
use sr_physics::reference::hydro::State;
use sr_physics::registry::{Elements, Physics};

/// Hydrogen's 1/μ (§1.5): ε_th = T/μ = 2T.
const INV_MU: f64 = 2.0;
/// Hydrogen's κ = κ₀ (1 + X_H) with κ₀ = 1 (§2.4.2, §2.7).
const KAPPA_H: f64 = 2.0;
/// The Gaussian of G-HEAT1: its σ₀ in cells, its peak over the background, the diffusivity of heat_pulse.json.
const SIGMA0: f64 = 4.0;
const PEAK: f64 = 1.0;
const CHI: f64 = 0.5;

fn scenes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scene(name: &str) -> Scene {
    Scene::load(&scenes_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn flags(scene: &Scene) -> TestFlags {
    let t = scene.test;
    TestFlags { pin_temperature: t.pin_temperature, constant_diffusivity: t.constant_diffusivity, flux_limiter: t.flux_limiter }
}

/// A scene's planes as the twin's state (f32 widened to f64), its constants and the heat pass it asks for.
fn world(scene: &Scene) -> (State, Physics, Heat) {
    let physics = scene.physics().unwrap();
    let (planes, warnings) = scene.planes(scene.world, &physics, &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let st = State::new(scene.world.width as usize, scene.world.height as usize, planes.iter().map(|&v| f64::from(v)).collect());
    let heat = Heat::new(&physics, &elements(), flags(scene)).unwrap();
    (st, physics, heat)
}

/// An inline scene: heat only, the cold pressure off, one rect of hydrogen at `sigma` and `t`.
fn rect_world(width: u32, height: u32, rect: [u32; 4], sigma: f64, t: f64) -> Scene {
    let [x0, y0, w, h] = rect;
    let text = format!(
        r#"{{"version": 1, "world": {{"width": {width}, "height": {height}, "edge": "bounce"}}, "slowdown": false,
 "switches": {{"gravity": false, "hydro": false, "heat": true, "reactions": false, "neutrinos": false, "sinks": false}},
 "objects": [{{"kind": "rect", "x0": {x0}, "y0": {y0}, "w": {w}, "h": {h}, "sigma": {sigma}, "t": {t}, "composition": {{"H": 1.0}}, "v": [0, 0]}}],
 "overrides": {{"k1e": 0, "k2e": 0, "k1n": 0, "k2n": 0}}}}"#
    );
    Scene::parse(&text, &elements()).unwrap_or_else(|e| panic!("{e}"))
}

/// T of every cell from E: ε_th = E/Σ (at rest, no cold energy), T = ε_th/(1/μ).
fn temps(st: &State) -> Vec<f64> {
    let n = st.cells();
    (0..n).map(|i| st.planes[3 * n + i] / st.planes[i] / INV_MU).collect()
}

/// Sets the temperatures of `st` (E = Σ · (1/μ) · T).
fn set_temps(st: &mut State, t: &[f64]) {
    let n = st.cells();
    for i in 0..n {
        st.planes[3 * n + i] = st.planes[i] * INV_MU * t[i];
    }
}

/// The heat_pulse.json medium with a Gaussian of `peak` over its background: T = T_bg + peak · exp(−r²/2σ₀²) about (64, 64).
fn gaussian(scene: &Scene, sigma0: f64, peak: f64) -> (State, Heat, f64) {
    gaussian_on(scene, None, sigma0, peak)
}

/// [`gaussian`] on a background of `bg` instead of T_floor (a background above the floor loses heat through the edge).
fn gaussian_on(scene: &Scene, bg: Option<f64>, sigma0: f64, peak: f64) -> (State, Heat, f64) {
    let (mut st, physics, heat) = world(scene);
    let w = st.width;
    // The scene's f32 planes hold T = 0.05 to 1e-8; the background is set here at T_floor to the last bit, so the vacuum
    // beyond the edge takes exactly nothing from it.
    let bg = bg.unwrap_or_else(|| physics.get("t_floor").unwrap());
    let mut t = vec![bg; st.cells()];
    for (i, ti) in t.iter_mut().enumerate() {
        let (dx, dy) = ((i % w) as f64 - 64.0, (i / w) as f64 - 64.0);
        *ti += peak * (-(dx * dx + dy * dy) / (2.0 * sigma0 * sigma0)).exp();
    }
    set_temps(&mut st, &t);
    (st, heat, bg)
}

/// The heat above `bg` (energy per area), and its variance along x and y about its own centre.
fn moments(st: &State, bg: f64) -> (f64, f64, f64) {
    let (w, n) = (st.width, st.cells());
    let t = temps(st);
    let (mut total, mut mx, mut my, mut mxx, mut myy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        let wgt = st.planes[i] * INV_MU * (t[i] - bg);
        let (x, y) = ((i % w) as f64, (i / w) as f64);
        total += wgt;
        mx += wgt * x;
        my += wgt * y;
        mxx += wgt * x * x;
        myy += wgt * y * y;
    }
    (total, mxx / total - (mx / total).powi(2), myy / total - (my / total).powi(2))
}

/// The sum of E over the world.
fn energy(st: &State) -> f64 {
    let n = st.cells();
    st.planes[3 * n..4 * n].iter().sum()
}

/// `steps` explicit sub-steps of dT/dt = χ ∇²T (five points, Δx = 1, the edges reflecting): the independent stepping G-HEAT2
/// compares RKL2 with. Stable for χ dt ≤ 1/4, but at 1/4 exactly a one-cell pulse keeps its checkerboard component for ever
/// (the Nyquist mode's factor is −1): the tests step it at half that.
fn explicit(t: &[f64], w: usize, h: usize, chi: f64, dt: f64, steps: usize) -> Vec<f64> {
    assert!(chi * dt <= 0.25 + 1e-12, "the explicit step must be stable");
    let mut t = t.to_vec();
    let mut next = t.clone();
    for _ in 0..steps {
        for y in 0..h {
            for x in 0..w {
                let at = |xx: usize, yy: usize| t[yy * w + xx];
                let c = at(x, y);
                let (l, r) = (at(x.saturating_sub(1), y), at((x + 1).min(w - 1), y));
                let (u, d) = (at(x, y.saturating_sub(1)), at(x, (y + 1).min(h - 1)));
                next[y * w + x] = c + chi * dt * (l + r + u + d - 4.0 * c);
            }
        }
        std::mem::swap(&mut t, &mut next);
    }
    t
}

/// The heat above `bg` of a plain temperature field of Σ = 1 (per unit C), and its variance along x and y, over the cells
/// within `half` of the pulse's centre (64, 64) on each axis — `usize::MAX` for the whole world.
fn stats(t: &[f64], w: usize, bg: f64, half: usize) -> (f64, f64, f64) {
    let (mut total, mut mx, mut my, mut mxx, mut myy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (i, ti) in t.iter().enumerate() {
        let (x, y) = (i % w, i / w);
        if x.abs_diff(64) > half || y.abs_diff(64) > half {
            continue;
        }
        let wgt = ti - bg;
        let (x, y) = (x as f64, y as f64);
        total += wgt;
        mx += wgt * x;
        my += wgt * y;
        mxx += wgt * x * x;
        myy += wgt * y * y;
    }
    (total, mxx / total - (mx / total).powi(2), myy / total - (my / total).powi(2))
}

/// The scene's pulse is far from every edge: nothing may leave, and what leaves is a failure of the set-up.
fn assert_inside(p: &Pass) {
    assert!(p.radiated.abs() < 1e-12, "radiated {}", p.radiated);
}

/// G-HEAT1: σ²(t) = σ₀² + 2χt per axis within 1 %, total heat constant to 1e-5, F at a face.
mod gauss {
    use super::*;

    #[test]
    fn the_scene_files_carry_what_the_oracles_assume() {
        let pulse = scene("heat_pulse.json");
        let (st, physics, _) = world(&pulse);
        assert_eq!((pulse.world.width, pulse.world.height), (128, 128));
        assert_eq!(pulse.edge, sr_engine::scene::EdgeMode::Bounce);
        let s = pulse.switches;
        assert!(s.heat && !s.gravity && !s.hydro && !s.reactions && !s.neutrinos && !s.sinks, "{s:?}");
        let t = pulse.test;
        assert_eq!((t.pin_temperature, t.constant_diffusivity, t.flux_limiter), (false, Some(CHI), true));
        for key in ["k1e", "k2e", "k1n", "k2n"] {
            assert_eq!(physics.get(key), Some(0.0), "cold pressure off: {key}");
        }
        // The background sits at the floor temperature, so the vacuum beyond the edge takes nothing from it.
        let bg = temps(&st);
        assert!(bg.iter().all(|&t| (t - physics.get("t_floor").unwrap()).abs() < 1e-6), "{:?}", &bg[..3]);
        assert!(st.plane(0).iter().all(|&s| s == 1.0));

        let spot = scene("heat_spot.json");
        let (st, physics, _) = world(&spot);
        assert_eq!(spot.edge, sr_engine::scene::EdgeMode::Leave);
        assert!(spot.switches.heat && !spot.switches.hydro && !spot.switches.gravity);
        assert_eq!(spot.test.constant_diffusivity, None);
        assert!(spot.test.flux_limiter && !spot.test.pin_temperature);
        // Thin gas, far above the vacuum threshold, with one hot 8 × 8 spot at T = 2.
        let sigma_vac = physics.get("sigma_vac").unwrap();
        assert!(st.plane(0).iter().all(|&s| (s - 1e-4).abs() < 1e-4 * 1e-6 && s >= 10.0 * sigma_vac), "thin gas everywhere");
        let t = temps(&st);
        let hot = t.iter().filter(|&&t| t > 1.0).count();
        assert_eq!(hot, 64);
        assert!(t.iter().all(|&t| (t - 2.0).abs() < 1e-6 || (t - 0.05).abs() < 1e-6));
    }

    /// σ_x² and σ_y² at t = 8, 16, 24 against σ₀² + 2χt: 1 passes of Δt = 1 (three stages each).
    #[test]
    fn the_variance_grows_as_sigma0_squared_plus_two_chi_t_per_axis() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (_, vx0, vy0) = moments(&st, bg);
        assert!((vx0 - SIGMA0 * SIGMA0).abs() < 1e-6 && (vy0 - SIGMA0 * SIGMA0).abs() < 1e-6, "{vx0} {vy0}");
        let mut worst: f64 = 0.0;
        for pass in 1..=24 {
            let p = heat.step(&mut st, 1.0);
            assert_eq!(p.stages, 3, "dt_p {} (0.5 for χ = 1/2)", p.dt_p);
            assert_inside(&p);
            if pass % 8 == 0 {
                let t = f64::from(pass);
                let want = SIGMA0 * SIGMA0 + 2.0 * CHI * t;
                let (_, vx, vy) = moments(&st, bg);
                let err = ((vx - want).abs() / want).max((vy - want).abs() / want);
                println!("SR-HEAT1 t = {t}: sigma2 x {vx:.6}, y {vy:.6}, expected {want}, error {err:.2e}");
                assert!(err <= 0.01, "t = {t}: σ² x {vx}, y {vy}, expected {want}");
                worst = worst.max(err);
            }
        }
        println!("SR-HEAT1 worst sigma2 error {worst:.2e}");
    }

    #[test]
    fn the_total_heat_is_constant() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (h0, _, _) = moments(&st, bg);
        let e0 = energy(&st);
        let mut worst: f64 = 0.0;
        for _ in 0..24 {
            let p = heat.step(&mut st, 1.0);
            assert_inside(&p);
            let (h, _, _) = moments(&st, bg);
            worst = worst.max((h - h0).abs() / h0);
        }
        println!("SR-HEAT1 heat {h0:.6}, worst change {worst:.2e} of the pulse, E {:.2e} of the world", (energy(&st) - e0).abs() / e0);
        assert!(worst <= 1e-5, "{worst}");
        assert!(h0 > 100.0, "a real pulse: {h0}");
    }

    /// F is −χ·C·ΔT at a face (C = Σ/μ = 2), positive toward +x, +y; the cell field is the mean of a cell's two faces.
    #[test]
    fn the_flux_kept_is_chi_c_times_the_temperature_difference() {
        let (mut st, heat, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let t = temps(&st);
        let (w, h) = (st.width, st.height);
        let p = heat.step(&mut st, 1.0);
        let c = 1.0 * INV_MU;
        for &(x, y) in &[(70usize, 64usize), (64, 64), (60, 61), (1, 1), (127, 40)] {
            let want = -CHI * c * (t[y * w + x] - t[y * w + x - 1]);
            let got = p.flux.x[y * (w + 1) + x];
            assert!((got - want).abs() <= 1e-12 * (1.0 + want.abs()), "x face ({x}, {y}): {got} vs {want}");
            let want = -CHI * c * (t[y * w + x] - t[(y - 1) * w + x]);
            let got = p.flux.y[y * w + x];
            assert!((got - want).abs() <= 1e-12 * (1.0 + want.abs()), "y face ({x}, {y}): {got} vs {want}");
        }
        // Heat flows away from the peak: at the centre's right face F > 0 would mean toward +x from a cooler cell.
        assert!(p.flux.x[64 * (w + 1) + 70] > 0.0 && p.flux.x[64 * (w + 1) + 58] < 0.0);
        assert!(p.flux.y[70 * w + 64] > 0.0 && p.flux.y[58 * w + 64] < 0.0);
        let (fx, fy) = p.flux.cells();
        let i = 64 * w + 70;
        assert!((fx[i] - 0.5 * (p.flux.x[64 * (w + 1) + 70] + p.flux.x[64 * (w + 1) + 71])).abs() < 1e-15);
        assert!((fy[i] - 0.5 * (p.flux.y[64 * w + 70] + p.flux.y[65 * w + 70])).abs() < 1e-15);
        // The world's faces: x = 0 and x = w are the edges, and the background is at the floor, so nothing flows there.
        assert_eq!(p.flux.x.len(), (w + 1) * h);
        assert_eq!(p.flux.y.len(), w * (h + 1));
    }

    /// A hot slab at the world's left edge and beside vacuum (§2.4.4): what the pass books `radiated` equals what the grid
    /// lost, to 1e-6, and each cell's share sits where the light left.
    #[test]
    fn a_hot_slab_beside_vacuum_books_radiated_equal_to_the_heat_the_grid_lost() {
        let scene = rect_world(64, 64, [0, 16, 32, 32], 1.0, 30.0);
        let (mut st, physics, heat) = world(&scene);
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let n = st.cells();
        let before = st.planes.clone();
        let e_before = energy(&st);
        let slab_energy: f64 = (0..n).filter(|&i| st.planes[i] >= sigma_vac).map(|i| st.planes[3 * n + i]).sum();
        let (mut radiated, mut emitted) = (0.0, vec![0.0; n]);
        let mut max_stages = 0;
        for _ in 0..12 {
            let p = heat.step(&mut st, 2.0);
            radiated += p.radiated;
            for (e, q) in emitted.iter_mut().zip(&p.emitted) {
                *e += q;
            }
            max_stages = max_stages.max(p.stages);
        }
        let lost = e_before - energy(&st);
        println!("SR-HEAT-RAD slab {slab_energy:.4}: lost {lost:.9}, radiated {radiated:.9}, stages up to {max_stages}");
        assert!(radiated / slab_energy > 0.01, "the slab must lose something real: {}", radiated / slab_energy);
        assert!(max_stages >= 3, "{max_stages}");
        assert!((lost - radiated).abs() <= 1e-6 * radiated, "lost {lost}, radiated {radiated}");
        assert!((emitted.iter().sum::<f64>() - radiated).abs() <= 1e-12 * radiated);
        // The emitting cells are the slab's surface: the left edge (x = 0), its right face (x = 31) and its top and bottom rows
        // (y = 16, 47); a cell inside books nothing, a vacuum cell neither, and no vacuum cell was touched.
        let mut surface = 0;
        for i in 0..n {
            let (x, y) = (i % 64, i / 64);
            let in_slab = x < 32 && (16..48).contains(&y);
            let on_surface = in_slab && (x == 0 || x == 31 || y == 16 || y == 47);
            if on_surface {
                assert!(emitted[i] > 0.0, "surface cell ({x}, {y}) emitted {}", emitted[i]);
                surface += 1;
            } else {
                assert_eq!(emitted[i], 0.0, "cell ({x}, {y}) emitted {}", emitted[i]);
            }
            if !in_slab {
                for c in 0..14 {
                    assert_eq!(st.planes[c * n + i].to_bits(), before[c * n + i].to_bits(), "vacuum cell ({x}, {y}) channel {c}");
                }
            }
        }
        assert_eq!(surface, 32 + 32 + 32 + 32 - 4, "the slab's ring: two columns and two rows of 32, four corners counted twice");
    }

    /// `test.pin_temperature` (§2.12.1): the pass leaves the state alone, books nothing, and still keeps F.
    #[test]
    fn pinned_temperature_leaves_the_state_and_keeps_the_flux() {
        let (mut st, _, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (_, physics, _) = world(&scene("heat_pulse.json"));
        let pinned = Heat::new(&physics, &elements(), TestFlags { pin_temperature: true, constant_diffusivity: Some(CHI), flux_limiter: true }).unwrap();
        let free = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: Some(CHI), flux_limiter: true }).unwrap();
        let before = st.clone();
        let p = pinned.step(&mut st, 1.0);
        assert_eq!(st, before, "the planes are bit for bit what they were");
        assert_eq!((p.stages, p.radiated), (0, 0.0));
        let q = free.step(&mut st.clone(), 1.0);
        assert_eq!(p.flux, q.flux, "F is the same pass's");
        assert!(p.flux.x.iter().any(|&f| f != 0.0));
        let mut moved = before.clone();
        free.step(&mut moved, 1.0);
        assert_ne!(moved, before, "the free pass moves heat");
    }
}

/// G-HEAT2: RKL2 against explicit sub-steps; the stage count; the warning above 32.
mod rkl2 {
    use super::*;

    /// One RKL2 pass over t = 24 (14 stages) against 96 explicit sub-steps of Δt_p/2 = 0.25, the pulse of G-HEAT1: σ² of each
    /// within 1 % of the other and of σ₀² + 2χt.
    #[test]
    fn one_rkl2_pass_matches_explicit_sub_steps_in_sigma_squared() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let t0 = temps(&st);
        let (w, h) = (st.width, st.height);
        let explicit_t = explicit(&t0, w, h, CHI, 0.25, 96);
        let (_, ex, ey) = stats(&explicit_t, w, bg, usize::MAX);
        let p = heat.step(&mut st, 24.0);
        let (total, vx, vy) = moments(&st, bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 24.0;
        println!("SR-HEAT2 {} stages for dt/dt_p = {}: RKL2 sigma2 x {vx:.6} y {vy:.6}, explicit x {ex:.6} y {ey:.6}, expected {want}", p.stages, 24.0 / p.dt_p);
        assert_eq!(p.stages, 14);
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "RKL2 {vx} {vy}, explicit {ex} {ey}");
        assert!((vx - want).abs() / want <= 0.01 && (vy - want).abs() / want <= 0.01, "{vx} {vy} vs {want}");
        assert!(total > 100.0);
    }

    /// The fields, not only their second moments: six RKL2 passes of Δt = 4 (6 stages each) against 96 explicit sub-steps, the
    /// same t = 24 — second order in the step against the five-point Euler's first, each within 1 % of the pulse's peak.
    #[test]
    fn six_rkl2_passes_match_the_explicit_field() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let t0 = temps(&st);
        let (w, h) = (st.width, st.height);
        let explicit_t = explicit(&t0, w, h, CHI, 0.25, 96);
        for _ in 0..6 {
            let p = heat.step(&mut st, 4.0);
            assert_eq!(p.stages, 6, "dt/dt_p = 8");
        }
        let t1 = temps(&st);
        let diff = t1.iter().zip(&explicit_t).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        let analytic = |i: usize| {
            let (dx, dy) = ((i % w) as f64 - 64.0, (i / w) as f64 - 64.0);
            PEAK * SIGMA0 * SIGMA0 / 40.0 * (-(dx * dx + dy * dy) / (2.0 * 40.0)).exp()
        };
        let exact = t1.iter().enumerate().map(|(i, a)| (a - bg - analytic(i)).abs()).fold(0.0, f64::max);
        println!("SR-HEAT2 six passes: largest |T_rkl2 - T_explicit| {diff:.3e}, |T_rkl2 - analytic| {exact:.3e}, peak {:.4}", t1[64 * w + 64] - bg);
        assert!(diff <= 0.01 * PEAK && exact <= 0.01 * PEAK, "{diff} {exact}");
        assert!(t1.iter().cloned().fold(f64::MAX, f64::min) - bg >= -1e-12, "a smooth pulse does not undershoot");
    }

    /// s from the contract's formula, worked by hand here: Δt_p = 0.5/(χ + χ) = 0.5 and the smallest s ≥ 2 with
    /// Δt_p (s² + s − 2)/4 ≥ Δt.
    #[test]
    fn the_stage_count_is_the_smallest_that_covers_dt() {
        let scene = scene("heat_pulse.json");
        let by_hand = |dt: f64| -> usize {
            let dt_p = 0.5 / (CHI + CHI);
            let mut s = 2usize;
            while dt_p * ((s * s + s) as f64 - 2.0) / 4.0 < dt {
                s += 1;
            }
            s
        };
        for dt in [0.05, 0.4999, 0.5, 0.5001, 1.0, 3.0, 24.0, 100.0] {
            let (mut st, heat, _) = gaussian(&scene, SIGMA0, PEAK);
            let p = heat.step(&mut st, dt);
            let want = by_hand(dt);
            assert!((p.dt_p - 0.5).abs() < 1e-12, "dt_p {}", p.dt_p);
            assert_eq!(p.stages, want, "dt {dt}");
            let dt_h = |s: usize| 0.5 * ((s * s + s) as f64 - 2.0) / 4.0;
            assert!(dt_h(p.stages) >= dt && (p.stages == 2 || dt_h(p.stages - 1) < dt), "dt {dt}: s {}", p.stages);
            assert_eq!(p.stages, stages_for(p.dt_p, dt));
        }
        // And the pass takes it all the way: the variance of the largest step is the analytic one.
        let (mut st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        heat.step(&mut st, 100.0);
        let (_, vx, _) = moments(&st, bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 100.0;
        assert!((vx - want).abs() / want <= 0.01, "{vx} {want}");
    }

    #[test]
    fn more_than_32_stages_prints_the_warning_and_32_does_not() {
        let scene = scene("heat_pulse.json");
        // s = 32 covers Δt_p (1024 + 32 − 2)/4 = 263.5 Δt_p = 131.75; the next step needs 33.
        let (mut st, heat, _) = gaussian(&scene, SIGMA0, PEAK);
        let p = heat.step(&mut st, 131.75);
        assert_eq!((p.stages, MAX_STAGES), (32, 32));
        assert_eq!(p.warning(), None);
        let (mut st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let p = heat.step(&mut st, 131.76);
        assert_eq!(p.stages, 33);
        assert_eq!(p.warning().as_deref(), Some("SR-WARN heat stages 33"));
        // The pass still runs, and keeps its heat: a warning is not a switch to anything else.
        let (total, _, _) = moments(&st, bg);
        let (t0, _, _) = moments(&gaussian(&scene, SIGMA0, PEAK).0, bg);
        assert!((total - t0).abs() / t0 <= 1e-9, "{total} {t0}");
    }

    /// The adversarial case: a one-cell pulse (every wavenumber present) in 15 passes of Δt = 6.4 Δt_p — s = 5, the step just
    /// over what 4 stages cover (4.5 Δt_p) — stays bounded and keeps σ² equal to an explicit run's.
    ///
    /// The pulse is the difference of two runs on a background of 2.0, with the pulse and without: the scheme is linear
    /// here (constant χ), so the difference is the pulse's own response with the edges' cooling of the background taken out.
    /// The background also keeps the response's ringing clear of T_floor: RKL2 undershoots a one-cell pulse by 2–4 % of it
    /// in the first passes (measured; the ring fades by the 15th), and a pass reads a cell below T_floor as T_floor — P8
    /// books that floor, P5 does not — which would drain the pulse's second moment. σ² is read over the whole world: the
    /// ripples are small but far, and carry second moment.
    #[test]
    fn a_sharp_pulse_stays_stable_at_a_step_just_over_one_stage_fewer() {
        let scene = scene("heat_pulse.json");
        let (mut with, heat, bg) = gaussian_on(&scene, Some(2.0), 0.01, PEAK);
        let (mut without, _, _) = gaussian_on(&scene, Some(2.0), 0.01, 0.0);
        let (t0, w, h) = (temps(&with), with.width, with.height);
        let mut stages = 0;
        let mut ring: f64 = 0.0;
        for _ in 0..15 {
            stages = heat.step(&mut with, 3.2).stages;
            assert_eq!(heat.step(&mut without, 3.2).stages, 5, "dt/dt_p = 6.4");
            let low = temps(&with).iter().zip(temps(&without)).map(|(a, b)| a - b).fold(f64::MAX, f64::min);
            ring = ring.min(low);
        }
        assert_eq!(stages, 5);
        let response: Vec<f64> = temps(&with).iter().zip(temps(&without)).map(|(a, b)| a - b).collect();
        let pulse: Vec<f64> = t0.iter().map(|t| t - bg).collect();
        let explicit_t = explicit(&pulse, w, h, CHI, 0.2, 240);
        let (_, ex, ey) = stats(&explicit_t, w, 0.0, usize::MAX);
        let (total, vx, vy) = stats(&response, w, 0.0, usize::MAX);
        let (lowest, highest) = (response.iter().cloned().fold(f64::MAX, f64::min), response.iter().cloned().fold(f64::MIN, f64::max));
        println!("SR-HEAT2 sharp pulse, {stages} stages x 15: sigma2 x {vx:.5} y {vy:.5}, explicit x {ex:.5} y {ey:.5}, response in [{lowest:.2e}, {highest:.4e}], deepest ring {ring:.3e}, heat {total:.9}");
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "{vx} {vy} vs {ex} {ey}");
        assert!((vx - 48.0).abs() / 48.0 <= 0.01, "{vx} vs 2 chi t = 48");
        assert!((total - PEAK).abs() / PEAK <= 1e-6, "heat {total}");
        // Bounded: the ringing is a few percent of the one-cell pulse at most (never growing), the response ends smooth.
        assert!(ring >= -0.05 * PEAK && lowest >= -1e-9 && highest <= 0.01, "ring {ring}, [{lowest}, {highest}]");
    }
}

/// G-FLD: |F| ≤ c_sb a_r T⁴ in thin gas; the radiative conductivity in thick gas; λ.
mod limiter {
    use super::*;

    fn constants(physics: &Physics) -> (f64, f64) {
        (physics.get("a_r").unwrap(), physics.get("c_sb").unwrap())
    }

    /// The largest ratio |F|/(c_sb a_r T⁴) over every face of `p` for the state `t` (Σ and the hotter side's T of the face;
    /// the world's edges face the vacuum at T_floor), and the faces where the flux is not 0.
    fn worst_ratio(p: &Pass, t: &[f64], w: usize, h: usize, a_r: f64, c_sb: f64) -> f64 {
        let t_floor = 0.05;
        let bound = |ta: f64, tb: f64| c_sb * a_r * ta.max(tb).powi(4);
        let mut worst: f64 = 0.0;
        for y in 0..h {
            for x in 0..=w {
                let ta = if x > 0 { t[y * w + x - 1] } else { t_floor };
                let tb = if x < w { t[y * w + x] } else { t_floor };
                worst = worst.max(p.flux.x[y * (w + 1) + x].abs() / bound(ta, tb));
            }
        }
        for y in 0..=h {
            for x in 0..w {
                let ta = if y > 0 { t[(y - 1) * w + x] } else { t_floor };
                let tb = if y < h { t[y * w + x] } else { t_floor };
                worst = worst.max(p.flux.y[y * w + x].abs() / bound(ta, tb));
            }
        }
        worst
    }

    /// The hot spot of heat_spot.json: every pass's flux, every face, under c_sb a_r T⁴ of its hotter side; the limiter binds.
    #[test]
    fn in_thin_gas_the_flux_never_passes_c_sb_a_r_t4() {
        let (mut st, physics, heat) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let (w, h) = (st.width, st.height);
        let mut worst: f64 = 0.0;
        let mut stages = 0;
        for pass in 0..20 {
            let t = temps(&st);
            let p = heat.step(&mut st, 0.05);
            let r = worst_ratio(&p, &t, w, h, a_r, c_sb);
            stages = stages.max(p.stages);
            assert!(r <= 1.0 + 1e-6, "pass {pass}: |F| = {r} c_sb a_r T^4");
            worst = worst.max(r);
        }
        println!("SR-FLD worst |F| / (c_sb a_r T^4) {worst:.6} over 20 passes of 64x64 thin gas, up to {stages} stages");
        // The oracle is not vacuous: the limiter is the thing holding the flux, within a factor of 3 of the bound.
        assert!(worst > 0.3, "the limiter binds: {worst}");
        // And it can fail: the same spot without the limiter (λ ≡ 1/3) puts the flux orders of magnitude over it.
        let (mut st, physics, _) = world(&scene("heat_spot.json"));
        let off = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: None, flux_limiter: false }).unwrap();
        let t = temps(&st);
        let p = off.step(&mut st, 0.0005);
        let r = worst_ratio(&p, &t, w, h, a_r, c_sb);
        println!("SR-FLD limiter off: |F| = {r:.1} c_sb a_r T^4");
        assert!(r > 100.0, "{r}");
    }

    /// Thick gas (κΣ = 200), a gentle gradient: λ = 1/3 and F is the radiative conductivity (4/3) a_r c_sb T³/(κΣ) times −∇T
    /// (SM-E20) to 1 % — the limiter must not touch gas it has no business in.
    #[test]
    fn in_thick_gas_the_flux_is_the_radiative_conductivity() {
        let scene = rect_world(64, 64, [0, 0, 64, 64], 100.0, 10.0);
        let (mut st, physics, heat) = world(&scene);
        let (a_r, c_sb) = constants(&physics);
        let (w, h) = (st.width, st.height);
        let t: Vec<f64> = (0..w * h).map(|i| 10.0 + 0.02 * (i % w) as f64).collect();
        set_temps(&mut st, &t);
        let p = heat.step(&mut st, 0.001);
        let ks = KAPPA_H * 100.0;
        let mut worst: f64 = 0.0;
        for y in 1..h - 1 {
            for x in 1..w {
                let mid = 0.5 * (t[y * w + x - 1] + t[y * w + x]);
                let want = -(4.0 / 3.0) * a_r * c_sb * mid.powi(3) / ks * (t[y * w + x] - t[y * w + x - 1]);
                let got = p.flux.x[y * (w + 1) + x];
                worst = worst.max((got - want).abs() / want.abs());
            }
        }
        println!("SR-FLD thick gas: worst |F - (4/3) a c T^3/(kappa Sigma) (-dT)| / |F| = {worst:.2e}");
        assert!(worst <= 0.01, "{worst}");
        // R is ~10^-4 here, so λ is 1/3 to 10^-4.
        assert!((lambda(1e-4) * 3.0 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn lambda_is_levermore_pomraning() {
        // λ(R) = (2 + R)/(6 + 3R + R²): 1/3 at 0, 3/10 at 1, 4/16 at 2, and λR < 1 → 1.
        for (r, want) in [(0.0, 1.0 / 3.0), (1.0, 0.3), (2.0, 0.25), (3.0, 5.0 / 24.0), (10.0, 12.0 / 136.0)] {
            assert!((lambda(r) - want).abs() < 1e-15, "R = {r}: {}", lambda(r));
        }
        for r in [0.01, 1.0, 100.0, 1e6, 1e12] {
            assert!(lambda(r) * r < 1.0, "R = {r}");
        }
        assert!(lambda(1e12) * 1e12 > 1.0 - 1e-9);
        // λ falls monotonically, so the flux (c λ ∇E/κΣ) grows more slowly than the gradient.
        assert!((0..60).map(|k| lambda(f64::from(k))).collect::<Vec<_>>().windows(2).all(|w| w[1] < w[0]));
    }

    /// `test.flux_limiter` = false (§2.12.1) is λ ≡ 1/3 even at R = 10⁴: F = c_sb ΔE/(3 κΣ_face) across a face of heat_spot.json.
    #[test]
    fn the_scene_flag_turns_the_limiter_into_one_third() {
        let (st, physics, _) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let w = st.width;
        let off = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: None, flux_limiter: false }).unwrap();
        let on = Heat::new(&physics, &elements(), TestFlags::default()).unwrap();
        let t = temps(&st);
        // The face between a hot cell (T = 2, x = 28) and its cooler neighbour (T = 0.05, x = 27) on row 30.
        let (x, y) = (28usize, 30usize);
        let (ta, tb) = (t[y * w + x - 1], t[y * w + x]);
        assert!((ta - 0.05).abs() < 1e-6 && (tb - 2.0).abs() < 1e-6, "{ta} {tb}");
        let ks = KAPPA_H * 0.5 * (st.get(0, y * w + x - 1) + st.get(0, y * w + x));
        let f_off = off.step(&mut st.clone(), 1e-6).flux.x[y * (w + 1) + x];
        let f_on = on.step(&mut st.clone(), 1e-6).flux.x[y * (w + 1) + x];
        let thick = -c_sb / (3.0 * ks) * a_r * (tb.powi(4) - ta.powi(4));
        let bound = c_sb * a_r * tb.powi(4);
        println!("SR-FLD face of the spot: unlimited {f_off:.4e} (thick form {thick:.4e}), limited {f_on:.4e}, bound {bound:.4e}");
        assert!((f_off - thick).abs() <= 1e-9 * thick.abs(), "{f_off} vs {thick}");
        assert!(f_off.abs() > 1000.0 * bound);
        assert!(f_on.abs() <= bound * (1.0 + 1e-6) && f_on.abs() > 0.1 * bound, "{f_on}");
        assert!(f_on * f_off > 0.0, "the same direction");
    }
}
