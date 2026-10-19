//! M0-T69 — the real-equivalent translations (contract §1.10.1–§1.10.3, §5.4's G-READ). No test here touches a GPU: the
//! module lives in the one test binary (R11) so the scope runs through `tests/gpu.sh`.
//!
//! The oracle is the contract's table and the arithmetic of a log-linear segment written out here by hand — never the
//! module's own helpers. The calibration is the test one (`Calibration::test_calibration`) until M0-V9 writes
//! `assets/calibration.json`; the last case reads that file once it exists.

use sr_physics::calibration::{physics_hash, Calibration};
use sr_physics::observe::translate::{
    surface_temperature_sb, MassScale, TempScale, TranslateError, Translations, MASS_SUNS, PRESET_SUNS, TEMP_KELVIN,
};
use sr_physics::registry::Physics;

fn physics() -> Physics {
    Physics::shipped().expect("the shipped physics.json parses")
}

fn calibration() -> Calibration {
    Calibration::test_calibration(&physics_hash().expect("the shipped assets are JSON"))
}

fn translations() -> Translations {
    Translations::new(&calibration(), &physics()).expect("the test calibration translates")
}

/// The sandbox anchors of each side, in the contract's order.
fn mass_anchors(c: &Calibration) -> [f64; 4] {
    [c.m_ign_sb, c.m_ch_sb, c.m_up_sb, c.m_bh_sb]
}

fn temp_anchors(c: &Calibration, p: &Physics) -> [f64; 8] {
    let k = |key: &str| p.get(key).unwrap();
    [k("t_floor"), c.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")]
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs()
}

/// A log-spaced sweep from `lo` to `hi`, `n` points, ends included.
fn sweep(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    (0..n).map(|i| lo * (hi / lo).powf(i as f64 / (n - 1) as f64)).collect()
}

#[test]
fn mass_increases_strictly_over_its_whole_range() {
    let t = translations();
    // From far below M_ign to far above M_BH, across every anchor; 0 reads 0 and is under everything else.
    let xs = sweep(1e-3, 1e8, 4001);
    let ys: Vec<f64> = xs.iter().map(|&x| t.mass.to_suns(x)).collect();
    assert_eq!(t.mass.to_suns(0.0), 0.0);
    assert!(ys[0] > 0.0);
    for w in ys.windows(2) {
        assert!(w[1] > w[0], "mass translation not strictly increasing: {} then {}", w[0], w[1]);
    }
    // The inverse is strictly increasing too.
    let back: Vec<f64> = sweep(1e-6, 1e4, 4001).iter().map(|&y| t.mass.to_sb(y)).collect();
    for w in back.windows(2) {
        assert!(w[1] > w[0], "inverse not strictly increasing: {} then {}", w[0], w[1]);
    }
}

#[test]
fn temperature_increases_strictly_from_its_floor_up() {
    let t = translations();
    let p = physics();
    let floor = p.get("t_floor").unwrap();
    // 10 K at and below T_floor (the flat part is the floor, not a violation); strictly increasing above it.
    for x in [0.0, floor * 0.5, floor] {
        assert_eq!(t.temp.to_kelvin(x), 10.0, "below and at T_floor reads 10 K");
    }
    let xs = sweep(floor, 1e6, 4001);
    let ys: Vec<f64> = xs.iter().map(|&x| t.temp.to_kelvin(x)).collect();
    for w in ys.windows(2) {
        assert!(w[1] > w[0], "temperature translation not strictly increasing: {} then {}", w[0], w[1]);
    }
    let back: Vec<f64> = sweep(10.0, 1e11, 4001).iter().map(|&k| t.temp.to_sb(k)).collect();
    for w in back.windows(2) {
        assert!(w[1] > w[0], "inverse not strictly increasing: {} then {}", w[0], w[1]);
    }
}

#[test]
fn every_anchor_maps_exactly_and_the_inverse_returns_it() {
    let c = calibration();
    let p = physics();
    let t = translations();
    let tol = 1e-9;
    for (i, (&sb, &real)) in mass_anchors(&c).iter().zip(MASS_SUNS.iter()).enumerate() {
        assert!(rel(t.mass.to_suns(sb), real) < tol, "mass anchor {i}: {sb} reads {}, not {real}", t.mass.to_suns(sb));
        assert!(rel(t.mass.to_sb(real), sb) < tol, "mass anchor {i}: {real} Suns inverts to {}, not {sb}", t.mass.to_sb(real));
    }
    // The contract's real side, written out.
    assert_eq!(MASS_SUNS, [0.08, 1.44, 8.0, 25.0]);
    assert_eq!(TEMP_KELVIN, [10.0, 5772.0, 1.571e7, 1.8e8, 8.3e8, 1.6e9, 1.9e9, 3.3e9]);
    for (i, (&sb, &real)) in temp_anchors(&c, &p).iter().zip(TEMP_KELVIN.iter()).enumerate() {
        assert!(rel(t.temp.to_kelvin(sb), real) < tol, "temperature anchor {i}: {sb} reads {}, not {real}", t.temp.to_kelvin(sb));
        assert!(rel(t.temp.to_sb(real), sb) < tol, "temperature anchor {i}: {real} K inverts to {}, not {sb}", t.temp.to_sb(real));
    }
}

#[test]
fn between_anchors_the_map_is_log_linear_not_linear() {
    // The adversarial case: a map interpolated in linear space passes every anchor and errs between them. At the
    // geometric midpoint of a segment's sandbox ends a log-linear map reads the geometric mean of the real ends; a
    // linear one reads their arithmetic mean. One case per segment, both scales.
    let c = calibration();
    let p = physics();
    let t = translations();
    let masses = mass_anchors(&c);
    for i in 0..3 {
        let mid = (masses[i] * masses[i + 1]).sqrt();
        let want = (MASS_SUNS[i] * MASS_SUNS[i + 1]).sqrt();
        let arithmetic = 0.5 * (MASS_SUNS[i] + MASS_SUNS[i + 1]);
        let got = t.mass.to_suns(mid);
        assert!(rel(got, want) < 1e-9, "mass segment {i}: midpoint reads {got}, log-linear says {want}");
        assert!(rel(got, arithmetic) > 0.05, "mass segment {i}: the midpoint is the linear one, {arithmetic}");
        assert!(rel(t.mass.to_sb(want), mid) < 1e-9, "mass segment {i}: the inverse misses the midpoint");
    }
    let temps = temp_anchors(&c, &p);
    for i in 0..7 {
        let mid = (temps[i] * temps[i + 1]).sqrt();
        let want = (TEMP_KELVIN[i] * TEMP_KELVIN[i + 1]).sqrt();
        let arithmetic = 0.5 * (TEMP_KELVIN[i] + TEMP_KELVIN[i + 1]);
        let got = t.temp.to_kelvin(mid);
        assert!(rel(got, want) < 1e-9, "temperature segment {i}: midpoint reads {got}, log-linear says {want}");
        assert!(rel(got, arithmetic) > 1e-3, "temperature segment {i}: the midpoint is the linear one, {arithmetic}");
        assert!(rel(t.temp.to_sb(want), mid) < 1e-9, "temperature segment {i}: the inverse misses the midpoint");
    }
    // A quarter of the way through a segment: ln y is the straight line, not the midpoint alone.
    let (a, b) = (masses[1], masses[2]);
    let x = a * (b / a).powf(0.25);
    let want = MASS_SUNS[1] * (MASS_SUNS[2] / MASS_SUNS[1]).powf(0.25);
    assert!(rel(t.mass.to_suns(x), want) < 1e-9);
}

#[test]
fn the_end_segments_are_extended() {
    let c = calibration();
    let p = physics();
    let t = translations();
    let m = mass_anchors(&c);
    // Below M_ign the first segment goes on, above M_BH the last.
    let s_lo = (MASS_SUNS[1] / MASS_SUNS[0]).ln() / (m[1] / m[0]).ln();
    let s_hi = (MASS_SUNS[3] / MASS_SUNS[2]).ln() / (m[3] / m[2]).ln();
    let below = t.mass.to_suns(m[0] / 10.0);
    let above = t.mass.to_suns(m[3] * 10.0);
    assert!(rel(below, MASS_SUNS[0] * 10f64.powf(-s_lo)) < 1e-9, "below M_ign reads {below}");
    assert!(rel(above, MASS_SUNS[3] * 10f64.powf(s_hi)) < 1e-9, "above M_BH reads {above}");
    // Above T_Si the O–Si segment goes on; at T_floor and under, 10 K.
    let tk = temp_anchors(&c, &p);
    let s_osi = (TEMP_KELVIN[7] / TEMP_KELVIN[6]).ln() / (tk[7] / tk[6]).ln();
    let hot = t.temp.to_kelvin(tk[7] * 3.0);
    assert!(rel(hot, TEMP_KELVIN[7] * 3f64.powf(s_osi)) < 1e-9, "above T_Si reads {hot}");
    assert_eq!(t.temp.to_kelvin(tk[0] / 100.0), 10.0);
}

#[test]
fn the_presets_read_one_fifteen_and_forty_suns() {
    let c = calibration();
    let t = translations();
    // The preset masses are the inverse translation of 1, 15 and 40 Suns (§2.10); read forward they return 1.00, 15.0, 40.0.
    assert_eq!(PRESET_SUNS, [1.0, 15.0, 40.0]);
    let sb: Vec<f64> = PRESET_SUNS.iter().map(|&s| t.mass.to_sb(s)).collect();
    assert!(sb[0] < sb[1] && sb[1] < sb[2], "preset masses must increase: {sb:?}");
    for (&m, &want) in sb.iter().zip(PRESET_SUNS.iter()) {
        let got = t.mass.to_suns(m);
        assert!(rel(got, want) < 1e-9, "{m} reads {got} Suns, not {want}");
    }
    // 1 Sun lies between M_ign and M_Ch, 15 between M_up and M_BH, 40 above M_BH: the segments the contract names.
    assert!(c.m_ign_sb < sb[0] && sb[0] < c.m_ch_sb);
    assert!(c.m_up_sb < sb[1] && sb[1] < c.m_bh_sb);
    assert!(sb[2] > c.m_bh_sb);
    // And the arithmetic by hand for the Sun: M_ign · (M_Ch/M_ign)^(ln(1/0.08)/ln(1.44/0.08)).
    let hand = c.m_ign_sb * (c.m_ch_sb / c.m_ign_sb).powf((1.0f64 / 0.08).ln() / (1.44f64 / 0.08).ln());
    assert!(rel(sb[0], hand) < 1e-9, "the Sun-like preset is {}, by hand {hand}", sb[0]);
}

#[test]
fn the_sun_like_surface_reads_5772_kelvin() {
    let p = physics();
    let c = calibration();
    let t = translations();
    // t_eff_sun_sb is the anchor: a surface at it reads 5,772 K. Drive it through L = 2πRσT⁴ with σ = a_r c_sb / 4.
    let (a_r, c_sb) = (p.get("a_r").unwrap(), p.get("c_sb").unwrap());
    let sigma = a_r * c_sb / 4.0;
    let r = 15.0;
    let l = 2.0 * std::f64::consts::PI * r * sigma * c.t_eff_sun_sb.powi(4);
    assert!(rel(surface_temperature_sb(l, r, a_r, c_sb), c.t_eff_sun_sb) < 1e-12);
    let k = t.surface_kelvin(l, r);
    assert!(rel(k, 5772.0) < 0.01, "the Sun-like surface reads {k} K, not 5,772 within 1 %");
    // A star four times as luminous at the same radius is 4^(1/4) hotter in sandbox units, and reads hotter in kelvins.
    assert!(t.surface_kelvin(4.0 * l, r) > k);
    // No light or no radius reads the floor.
    assert_eq!(t.surface_kelvin(0.0, r), 10.0);
    assert_eq!(t.surface_kelvin(l, 0.0), 10.0);
}

#[test]
fn a_translation_that_would_not_be_monotonic_is_refused() {
    let c = calibration();
    let p = physics();
    // Sandbox anchors out of order (K9's order broken) or equal.
    let m = mass_anchors(&c);
    assert!(matches!(
        MassScale::new(m[0], m[2], m[1], m[3]),
        Err(TranslateError::NotIncreasing { side: "sandbox", index: 2, .. })
    ));
    assert!(matches!(MassScale::new(m[0], m[1], m[1], m[3]), Err(TranslateError::NotIncreasing { .. })));
    assert!(matches!(MassScale::new(0.0, m[1], m[2], m[3]), Err(TranslateError::NotPositive { index: 0, .. })));
    // A Sun-like T_eff above T_H, or under T_floor: the temperature side must increase strictly.
    let mut t = temp_anchors(&c, &p);
    t[1] = t[2] * 2.0;
    assert!(matches!(TempScale::new(t), Err(TranslateError::NotIncreasing { side: "sandbox", index: 2, .. })));
    let mut t = temp_anchors(&c, &p);
    t[1] = t[0] / 2.0;
    assert!(matches!(TempScale::new(t), Err(TranslateError::NotIncreasing { side: "sandbox", index: 1, .. })));
    let mut bad = c.clone();
    bad.t_eff_sun_sb = f64::NAN;
    assert!(Translations::new(&bad, &p).is_err());
    // The whole thing through a calibration whose masses are out of order.
    let mut bad = c.clone();
    bad.m_up_sb = bad.m_ch_sb / 2.0;
    assert!(Translations::new(&bad, &p).is_err());
}

#[test]
fn the_real_calibration_reads_the_presets_and_the_sun() {
    // Once M0-V9 writes assets/calibration.json: its presets read 1.00, 15.0 and 40.0 Suns and its t_eff_sun_sb reads
    // 5,772 K, on the translation its own anchors make. Until then there is no file and this is NOT PROVEN against a real
    // calibration — the case says so and passes only the part it can check.
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/calibration.json");
    let Ok(text) = std::fs::read_to_string(path) else {
        println!("SR-NOTE readouts: assets/calibration.json absent - NOT PROVEN against a real calibration (test calibration only)");
        return;
    };
    let cal = Calibration::parse(&text).expect("assets/calibration.json parses");
    let t = Translations::new(&cal, &physics()).expect("the real calibration translates");
    let p = cal.preset_mass_sb;
    for (m, want) in [(p.sun, 1.0), (p.massive, 15.0), (p.giant, 40.0)] {
        let got = t.mass.to_suns(m);
        assert!(rel(got, want) < 1e-3, "preset {m} reads {got} Suns, not {want}");
    }
    assert!(rel(t.temp.to_kelvin(cal.t_eff_sun_sb), 5772.0) < 0.01);
}
