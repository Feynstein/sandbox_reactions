//! M0-T61 — the preset cloud at a given mass (contract §2.1, §2.10): the mass the cells hold against M, the radius and
//! central Σ per formula including the 150-cell cap, the uniform temperature that makes U_th = 0.1 |W| with W from one
//! gravity solve of the cloud's own cells, and the Sun's composition at rest. No GPU: the planes are the scene's own
//! (what a drop uploads), so no `SR-ADAPTER` line.

use sr_engine::scene::{cloud_shape, Scene, MAX_RADIUS, SUN_MIX, UNIT_MASS};
use sr_engine::state::{WorldConfig, SPECIES};
use sr_physics::eos::Eos;
use sr_physics::reference::grav::Gravity;
use sr_physics::registry::{Elements, Physics};

/// In units of M₀: below the cap (a < 150 up to 14.06 M₀), at it, and well past it.
const MASSES: [f64; 6] = [0.3, 1.0, 3.0, 9.0, 20.0, 60.0];

fn world() -> WorldConfig {
    WorldConfig::new(600, 400).unwrap()
}

fn planes(mass: f64) -> Vec<f32> {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let scene = Scene::preset_mass("sun", mass, world()).unwrap();
    let (planes, warnings) = scene.planes(world(), &physics, &elements).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    planes
}

/// Σ of the cloud's cells, 0 where the scene leaves the floor state (no object there, M0-D101).
fn sigma(planes: &[f32]) -> Vec<f64> {
    let floor = Physics::shipped().unwrap().get("sigma_floor").unwrap() as f32;
    planes[..world().cells()].iter().map(|&s| if s == floor { 0.0 } else { f64::from(s) }).collect()
}

#[test]
fn the_cells_hold_the_mass_asked_for() {
    for m in MASSES {
        let mass = m * UNIT_MASS;
        let held: f64 = sigma(&planes(mass)).iter().sum();
        let err = (held - mass).abs() / mass;
        println!("M = {m} M0 = {mass:.3}: cells hold {held:.3}, relative error {err:.2e}");
        assert!(err <= 1e-6, "M = {m} M0: the cells hold {held}, not {mass}");
    }
}

#[test]
fn the_radius_and_central_sigma_follow_the_formula_and_the_cap() {
    let w = world().width as usize;
    for m in MASSES {
        let mass = m * UNIT_MASS;
        let shape = cloud_shape(mass);
        let want_a = (40.0 * m.sqrt()).min(MAX_RADIUS);
        let want_sigma_c = 3.0 * mass / (2.0 * std::f64::consts::PI * want_a * want_a);
        assert_eq!(shape.a, want_a, "a at {m} M0");
        assert!((shape.sigma_c - want_sigma_c).abs() <= 1e-12 * want_sigma_c, "Σc at {m} M0");
        // The planes agree: the cloud spans 2a cells and its centre cell holds Σc (sampling moves both by at most the
        // edge bound 2√2 a^-3/2 of the mass, which scales Σ).
        let s = sigma(&planes(mass));
        let (mut lo, mut hi) = (usize::MAX, 0);
        for (i, &v) in s.iter().enumerate() {
            if v > 0.0 {
                lo = lo.min(i % w);
                hi = hi.max(i % w);
            }
        }
        let span = (hi - lo + 1) as f64;
        let centre = s[199 * w + 299];
        let bound = 2.0 * 2f64.sqrt() * want_a.powf(-1.5) + 1e-3;
        println!("M = {m} M0: a = {:.2} (spans {span} cells), Σc = {:.5} (centre cell {centre:.5})", shape.a, shape.sigma_c);
        assert!((span - 2.0 * want_a).abs() <= 2.0, "the cloud spans {span} cells, 2a = {}", 2.0 * want_a);
        assert!((centre / want_sigma_c - 1.0).abs() <= bound, "centre Σ {centre} against Σc {want_sigma_c}");
    }
    // The nominal cloud: Σc = 1 at a = 40 (§2.1); past the cap the radius holds and Σc grows.
    let nominal = cloud_shape(UNIT_MASS);
    assert!((nominal.a - 40.0).abs() < 1e-9 && (nominal.sigma_c - 1.0).abs() < 1e-9, "{nominal:?}");
    let (capped, heavier) = (cloud_shape(20.0 * UNIT_MASS), cloud_shape(60.0 * UNIT_MASS));
    assert!(capped.a == MAX_RADIUS && heavier.a == MAX_RADIUS && heavier.sigma_c > capped.sigma_c && capped.sigma_c > 1.0);
}

#[test]
fn the_thermal_energy_is_a_tenth_of_the_grids_own_binding_energy() {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let eos = Eos::from_physics(&physics).unwrap();
    let comp = elements.composition(&SUN_MIX);
    let sigma_vac = physics.get("sigma_vac").unwrap();
    let (w, h) = (world().width as usize, world().height as usize);
    let mut gravity = Gravity::new(1.0);
    for m in MASSES {
        let p = planes(m * UNIT_MASS);
        let cells = world().cells();
        let s = sigma(&p);
        // W = ½ Σ m φ from one solve of the planes as stored (f32 Σ), as P2 solves them.
        let phi = gravity.potential(&gravity_source(&s, sigma_vac), w, h).unwrap();
        let big_w = 0.5 * s.iter().zip(&phi).map(|(m, p)| m * p).sum::<f64>();
        // E = Σ(ε_th + ε_cold) + ½|u|² (§2.2) at rest: the thermal part is what is left of E.
        let u_th: f64 = (0..cells)
            .filter(|&i| s[i] > 0.0)
            .map(|i| f64::from(p[3 * cells + i]) - s[i] * eos.eps_cold(s[i], comp.y_e, 0.0))
            .sum();
        let ratio = u_th / big_w.abs();
        println!("M = {m} M0: W = {big_w:.2}, U_th = {u_th:.3}, U_th/|W| = {ratio:.6}");
        assert!((ratio / 0.1 - 1.0).abs() <= 1e-4, "U_th/|W| = {ratio}, not 0.1");
    }
}

fn gravity_source(sigma: &[f64], sigma_vac: f64) -> Vec<f64> {
    sigma.iter().map(|&s| if s >= sigma_vac { s } else { 0.0 }).collect()
}

#[test]
fn the_composition_is_the_suns_at_rest_and_the_temperature_uniform() {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let eos = Eos::from_physics(&physics).unwrap();
    let comp = elements.composition(&SUN_MIX);
    let mu = 1.0 / comp.inv_mu;
    let cells = world().cells();
    for m in [1.0, 20.0] {
        let p = planes(m * UNIT_MASS);
        let s = sigma(&p);
        let (mut temps, mut covered) = (Vec::new(), 0);
        for i in 0..cells {
            let species: Vec<f64> = SPECIES.map(|c| f64::from(p[c * cells + i])).collect();
            if s[i] == 0.0 {
                assert_eq!(species[0], 1.0, "a vacuum cell is x_H = 1 (the floor state, P8's)");
                continue;
            }
            covered += 1;
            let sum: f64 = species.iter().sum();
            assert!((sum - 1.0).abs() <= 1e-6, "cell {i}: the fractions sum to {sum}");
            for (got, want) in species.iter().zip(SUN_MIX) {
                assert!((got - want).abs() <= 1e-6, "cell {i}: {species:?} against {SUN_MIX:?}");
            }
            assert!(p[cells + i] == 0.0 && p[2 * cells + i] == 0.0, "cell {i} is not at rest");
            // T = μ ε_th with ε_th = E/Σ − ε_cold (§2.3.1), the same everywhere.
            temps.push(mu * (f64::from(p[3 * cells + i]) / s[i] - eos.eps_cold(s[i], comp.y_e, 0.0)));
        }
        let (lo, hi) = temps.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &t| (lo.min(t), hi.max(t)));
        println!("M = {m} M0: {covered} cells, T from {lo:.6} to {hi:.6}, X = {SUN_MIX:?}");
        assert!(covered > 1000, "{covered} cells");
        // E is stored in f32 and the thinnest edge cells carry the largest cold part, so the spread is read where it
        // is not rounding: the cells holding most of the mass.
        let mean = temps.iter().sum::<f64>() / temps.len() as f64;
        let core: Vec<f64> = (0..cells)
            .filter(|&i| s[i] > 0.5 * s.iter().cloned().fold(0.0, f64::max))
            .map(|i| mu * (f64::from(p[3 * cells + i]) / s[i] - eos.eps_cold(s[i], comp.y_e, 0.0)))
            .collect();
        assert!(!core.is_empty() && core.iter().all(|t| (t / mean - 1.0).abs() <= 1e-3), "T is not uniform: {lo}..{hi}");
    }
}

#[test]
fn a_scene_file_preset_is_the_nominal_mass_and_a_bad_mass_is_refused() {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    for name in ["sun", "massive", "giant"] {
        let scene = Scene::preset(name, world()).unwrap();
        let held: f64 = sigma(&scene.planes(world(), &physics, &elements).unwrap().0).iter().sum();
        assert!((held - UNIT_MASS).abs() <= 1e-6 * UNIT_MASS, "preset:{name} holds {held}");
    }
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(Scene::preset_mass("sun", bad, world()).is_err(), "mass {bad} must be refused");
    }
    assert!(Scene::preset_mass("moon", 1.0, world()).is_err());
}
