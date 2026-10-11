//! M0-T25 — scene files (contract §2.12.1, §2.12.4, §2.10's cloud, §2.8's world and edge, §3.3's `headless --scene
//! --world --edge`): the refusals (an unknown key at every level, `t` with `pressure`, a bad override), where objects
//! land, the energy `t` and `pressure` give through the EOS, a disk's mass against the Maclaurin integral, every file in
//! scenes/ parsing, and `headless` running a scene file as given.

use super::device;
use sr_engine::scene::{sun_disk_planes, Scene, SUN_MIX};
use sr_engine::state::{State, WorldConfig, N_CHANNELS};
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics};

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn parse(text: &str) -> Result<Scene, String> {
    Scene::parse(text, &elements())
}

/// A scene that holds every level a key can sit at: world, slowdown, switches, test, a disk, a rect, a preset and
/// overrides. `extra` is spliced into the level its marker names (`@top`, `@world`, …), as `"key": value,`.
fn base(marker: &str, extra: &str) -> String {
    let at = |m: &str| if m == marker { extra } else { "" };
    format!(
        r#"{{{} "version": 1,
 "world": {{{} "width": 64, "height": 64, "edge": "leave"}},
 "slowdown": true,
 "switches": {{{} "gravity": true, "hydro": true, "heat": true, "reactions": true, "neutrinos": true, "sinks": true}},
 "test": {{{} "pin_temperature": false, "constant_diffusivity": null, "flux_limiter": true}},
 "objects": [
   {{{} "kind": "disk", "x": 32, "y": 32, "a": 10, "sigma_c": 0.5, "t": 2.0,
    "composition": {{{} "H": 0.7346, "He": 0.2485, "O": 0.0077, "C": 0.0092}}, "v": [0, 0]}},
   {{{} "kind": "rect", "x0": 0, "y0": 0, "w": 64, "h": 4, "sigma": 1.0, "pressure": 100.0,
    "composition": {{"H": 1.0}}, "v": [0, 0]}},
   {{{} "kind": "preset", "preset": "sun", "x": 32, "y": 32}}
 ],
 "overrides": {{{} "cfl": 0.3}}}}"#,
        at("@top"),
        at("@world"),
        at("@switches"),
        at("@test"),
        at("@disk"),
        at("@composition"),
        at("@rect"),
        at("@preset"),
        at("@overrides"),
    )
}

/// Writes `text` to a fresh file under the temp folder and returns its path.
fn scene_file(name: &str, text: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-scene-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// `headless` in-process (the function sr-app's `sandbox-reactions headless` forwards to); its exit code and the
/// summary.json it wrote, if any.
fn headless(args: &[&str], out: &std::path::Path) -> (i32, Option<serde_json::Value>) {
    let _ = std::fs::remove_dir_all(out);
    let mut args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    args.extend(["--out".to_string(), out.display().to_string()]);
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let code = sr_engine::headless::run(&args);
    let summary = std::fs::read_to_string(out.join("summary.json")).ok().map(|t| serde_json::from_str(&t).unwrap());
    (code, summary)
}

fn scenes_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

#[test]
fn unknown_keys_are_refused_at_every_level() {
    let scene = parse(&base("", "")).expect("the base scene parses");
    assert_eq!(scene.objects.len(), 3);
    // The adversarial case first: a misspelt switch must not leave hydro silently on.
    let cases = [
        ("@switches", r#""hydor": false,"#, "switches.hydor: unknown key"),
        ("@top", r#""colour": 1,"#, "colour: unknown key"),
        ("@world", r#""depth": 8,"#, "world.depth: unknown key"),
        ("@test", r#""pin_temp": true,"#, "test.pin_temp: unknown key"),
        ("@disk", r#""radius": 3,"#, "objects[0].radius: unknown key"),
        ("@composition", r#""Hx": 0.0,"#, "objects[0].composition.Hx: unknown key"),
        ("@rect", r#""colour": 1,"#, "objects[1].colour: unknown key"),
        ("@preset", r#""mass": 1,"#, "objects[2].mass: unknown key"),
    ];
    for (marker, extra, expected) in cases {
        let err = parse(&base(marker, extra)).expect_err(expected);
        println!("refused: {err}");
        assert!(err.contains(expected), "{marker}: expected {expected:?}, got {err:?}");
    }
    // Through headless: exit 4, an SR-ERROR line, no summary — before any GPU is touched.
    let path = scene_file("hydor.json", &base("@switches", r#""hydor": false,"#));
    let out = std::env::temp_dir().join(format!("sr-scene-{}-hydor-out", std::process::id()));
    let (code, summary) = headless(&["--scene", &path.display().to_string(), "--steps", "1"], &out);
    assert_eq!(code, 4, "an unknown key is exit 4 (§2.12.1)");
    assert!(summary.is_none(), "a refused scene writes no summary");
}

#[test]
fn t_and_pressure_are_one_or_the_other() {
    let both = base("@disk", r#""pressure": 1.0,"#);
    let err = parse(&both).expect_err("t and pressure together");
    assert!(err.contains("objects[0]: takes t or pressure, not both"), "{err}");
    let neither = base("", "").replace(r#""pressure": 100.0,"#, "");
    let err = parse(&neither).expect_err("neither t nor pressure");
    assert!(err.contains("objects[1]: needs t or pressure"), "{err}");
    let rect_both = base("@rect", r#""t": 1.0,"#);
    assert!(parse(&rect_both).expect_err("rect, both").contains("objects[1]: takes t or pressure, not both"));
}

#[test]
fn overrides_are_physics_keys_within_their_bounds() {
    let bad_key = base("@overrides", r#""gravity_const": 2.0,"#);
    let err = parse(&bad_key).expect_err("not a physics.json key");
    assert!(err.contains("overrides.gravity_const: unknown key"), "{err}");
    // A physics.json key out of its bound is refused by physics.json's own rule (§2.7).
    let out_of_bound = base("", "").replace(r#""cfl": 0.3"#, r#""cfl": -1.0"#);
    let err = parse(&out_of_bound).expect_err("cfl below 0");
    assert!(err.contains("overrides") && err.contains("cfl"), "{err}");
    let half_pair = base("", "").replace(r#""cfl": 0.3"#, r#""k1e": 0"#);
    assert!(parse(&half_pair).is_err(), "a K pair half off is refused");
    // A good one applies and the scene never writes calibration; none at all writes it.
    let pair_off = parse(&base("", "").replace(r#""cfl": 0.3"#, r#""k1e": 0, "k2e": 0"#)).unwrap();
    assert!(!pair_off.writes_calibration(), "§2.12.1: a run with overrides never writes calibration");
    assert_eq!(pair_off.physics().unwrap().get("k1e"), Some(0.0));
    let plain = parse(&base("", "").replace("],\n \"overrides\": { \"cfl\": 0.3}", "]")).unwrap();
    assert!(plain.overrides.is_empty() && plain.writes_calibration());
    assert_eq!(plain.physics().unwrap(), Physics::shipped().unwrap());
}

/// A one-object scene in a 64 × 64 world.
fn one(object: &str) -> Scene {
    parse(&format!(r#"{{"version": 1, "world": {{"width": 64, "height": 64}}, "objects": [{object}]}}"#)).unwrap()
}

fn planes_of(scene: &Scene, world: WorldConfig) -> Vec<f32> {
    let (planes, warnings) = scene.planes(world, &Physics::shipped().unwrap(), &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    planes
}

#[test]
fn objects_land_where_section_2_12_4_puts_them() {
    let w64 = WorldConfig::new(64, 64).unwrap();
    let cells = 64 * 64;
    let sigma_at = |p: &[f32], x: usize, y: usize, w: usize| p[y * w + x];
    // x to the right, y down, (0, 0) top-left: a 3 × 2 rect from (8, 16) is rows 16–17, columns 8–10.
    let rect = one(r#"{"kind": "rect", "x0": 8, "y0": 16, "w": 3, "h": 2, "sigma": 0.25, "t": 1.0, "composition": {"H": 1}}"#);
    let p = planes_of(&rect, w64);
    let lit: Vec<(usize, usize)> = (0..cells).filter(|&i| p[i] > 0.0).map(|i| (i % 64, i / 64)).collect();
    assert_eq!(lit, vec![(8, 16), (9, 16), (10, 16), (8, 17), (9, 17), (10, 17)]);
    // A disk at the point (20, 40), a = 1: the four cells whose centres lie within 1 of it.
    let disk = one(r#"{"kind": "disk", "x": 20, "y": 40, "a": 1, "sigma_c": 1, "t": 1, "composition": {"H": 1}}"#);
    let p = planes_of(&disk, w64);
    let lit: Vec<(usize, usize)> = (0..cells).filter(|&i| p[i] > 0.0).map(|i| (i % 64, i / 64)).collect();
    assert_eq!(lit, vec![(19, 39), (20, 39), (19, 40), (20, 40)]);
    assert!((sigma_at(&p, 19, 39, 64) - 0.5f32.sqrt()).abs() < 1e-6, "Σc√(1 − r²/a²) at r² = ½");
    // A wider world keeps the objects where they stood relative to the centre: (+32, +8) for 128 × 80.
    let p = planes_of(&rect, WorldConfig::new(128, 80).unwrap());
    assert!(sigma_at(&p, 40, 24, 128) > 0.0 && sigma_at(&p, 42, 25, 128) > 0.0 && sigma_at(&p, 43, 25, 128) == 0.0);
    // Overlapping objects add mass, momentum and each species' mass; vacuum elsewhere is x_H = 1.
    let two = parse(
        r#"{"version": 1, "world": {"width": 64, "height": 64}, "objects": [
        {"kind": "rect", "x0": 0, "y0": 0, "w": 2, "h": 1, "sigma": 0.3, "t": 1, "composition": {"H": 1}, "v": [1, 0]},
        {"kind": "rect", "x0": 1, "y0": 0, "w": 2, "h": 1, "sigma": 0.1, "t": 1, "composition": {"He": 1}, "v": [0, -2]}]}"#,
    )
    .unwrap();
    let p = planes_of(&two, w64);
    let ch = |c: usize, x: usize| p[c * cells + x];
    assert!((ch(0, 1) - 0.4).abs() < 1e-7 && (ch(1, 1) - 0.3).abs() < 1e-7 && (ch(2, 1) + 0.2).abs() < 1e-7);
    assert!((ch(4, 1) - 0.75).abs() < 1e-7 && (ch(5, 1) - 0.25).abs() < 1e-7, "X by mass");
    assert_eq!((ch(0, 5), ch(4, 5), ch(5, 5)), (0.0, 1.0, 0.0), "vacuum");
    // An object outside the world is a warning, never a silent nothing.
    let gone = one(r#"{"kind": "disk", "x": 500, "y": 500, "a": 5, "sigma_c": 1, "t": 1, "composition": {"H": 1}}"#);
    let (_, warnings) = gone.planes(w64, &Physics::shipped().unwrap(), &elements()).unwrap();
    assert_eq!(warnings, vec!["objects[0] covers no cell of the 64x64 world".to_string()]);
}

#[test]
fn t_and_pressure_set_the_energy_through_the_eos() {
    let physics = Physics::shipped().unwrap();
    let eos = Eos::from_physics(&physics).unwrap();
    let comp = elements().composition(&SUN_MIX);
    let (mu, y_e) = (1.0 / comp.inv_mu, comp.y_e);
    let mix = r#""composition": {"H": 0.7346, "He": 0.2485, "O": 0.0077, "C": 0.0092}"#;
    let w64 = WorldConfig::new(64, 64).unwrap();
    let cells = 64 * 64;
    // E = Σ(ε_th + ε_cold) + ½Σ|u|² (§2.2), ε_th = T/μ (§2.3.1).
    let hot = one(&format!(r#"{{"kind": "rect", "x0": 0, "y0": 0, "w": 1, "h": 1, "sigma": 2.0, "t": 3.0, {mix}, "v": [0.5, -1.5]}}"#));
    let p = planes_of(&hot, w64);
    let expected = 2.0 * (3.0 / mu + eos.eps_cold(2.0, y_e, 0.0)) + 0.5 * 2.0 * (0.25 + 2.25);
    let e = f64::from(p[3 * cells]);
    println!("t = 3: E = {e}, expected {expected}");
    assert!((e - expected).abs() <= 1e-6 * expected, "E from t");
    // Π = Σ ε_th + Π_e + Π_n (§2.3.3): the cell's pressure from its energy is the one asked for.
    let pressed = one(&format!(r#"{{"kind": "rect", "x0": 0, "y0": 0, "w": 1, "h": 1, "sigma": 2.0, "pressure": 500.0, {mix}}}"#));
    let p = planes_of(&pressed, w64);
    let eps_th = f64::from(p[3 * cells]) / 2.0 - eos.eps_cold(2.0, y_e, 0.0);
    let pi = eos.pi(2.0, eps_th, y_e, 0.0);
    println!("pressure 500: Π = {pi}");
    assert!((pi - 500.0).abs() <= 1e-5 * 500.0, "Π from pressure: {pi}");
    // A pressure below the cold pressure has no thermal energy to give: refused, naming the cell.
    let low = one(&format!(r#"{{"kind": "rect", "x0": 3, "y0": 5, "w": 1, "h": 1, "sigma": 2.0, "pressure": 1e-6, {mix}}}"#));
    let err = low.planes(w64, &physics, &elements()).expect_err("below the cold pressure");
    assert!(err.contains("objects[0].pressure") && err.contains("cell (3, 5)"), "{err}");
}

/// The mass of a disk sampled at cell centres, read back from the GPU's state, against M = 2πΣc a²/3 (§2.1).
/// The bound it states: the sampling only misjudges cells the edge crosses, so the error is under the mass of the
/// outermost one-cell ring, ∫ 2πr Σc √(1 − r²/a²) dr over [a − 1, a] ≈ (4π/3)√2 Σc a^{1/2} — relative to M, 2√2 a^{−3/2}.
#[test]
fn a_disks_mass_is_the_maclaurin_integral_within_its_sampling_bound() {
    let gpu = device();
    let world = WorldConfig::new(600, 400).unwrap();
    let state = State::new(&gpu.device, world).unwrap();
    let mix = r#""composition": {"H": 0.7346, "He": 0.2485, "O": 0.0077, "C": 0.0092}"#;
    for (x, y, a, sigma_c) in [(300.0, 200.0, 40.0, 1.0), (100.3, 57.8, 12.0, 0.5), (420.0, 300.25, 25.5, 2.0), (300.5, 200.5, 150.0, 0.1)]
    {
        let text = format!(
            r#"{{"version": 1, "world": {{"width": 600, "height": 400}}, "objects": [{{"kind": "disk", "x": {x}, "y": {y}, "a": {a}, "sigma_c": {sigma_c}, "t": 2.0, {mix}}}]}}"#
        );
        state.upload(&gpu.queue, &planes_of(&parse(&text).unwrap(), world)).unwrap();
        let back = state.readback(&gpu.device, &gpu.queue);
        let mass: f64 = back[..world.cells()].iter().map(|&s| f64::from(s)).sum();
        let exact = 2.0 * std::f64::consts::PI * sigma_c * a * a / 3.0;
        let (off, bound) = ((mass - exact).abs() / exact, 2.0 * 2f64.sqrt() * a.powf(-1.5));
        println!("disk a = {a} at ({x}, {y}): M = {mass:.4}, 2πΣc a²/3 = {exact:.4}, off {off:.2e} ≤ bound {bound:.2e}");
        assert!(off <= bound, "a = {a}: off {off:e} > {bound:e}");
        let x_sum: f64 = (4..N_CHANNELS).map(|c| f64::from(back[c * world.cells() + (y as usize) * 600 + x as usize])).sum();
        assert!((x_sum - 1.0).abs() < 1e-6, "ΣX = 1 at the centre");
    }
}

#[test]
fn every_scene_file_parses() {
    let mut files: Vec<_> = std::fs::read_dir(scenes_dir())
        .expect("scenes/ exists")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    assert!(files.len() >= 2, "scenes/ holds sun_disk.json and fill_world.json at least: {files:?}");
    let physics = Physics::shipped().unwrap();
    for path in &files {
        let scene = Scene::load(path).unwrap_or_else(|e| panic!("{e}"));
        let (_, warnings) = scene.planes(scene.world, &scene.physics().unwrap(), &elements()).unwrap();
        println!("{}: {} objects, {}x{}, no warning: {}", path.display(), scene.objects.len(), scene.world.width, scene.world.height, warnings.is_empty());
        assert!(warnings.is_empty(), "{}: {warnings:?}", path.display());
    }
    // sun_disk.json is the preset's own disk, bit for bit; fill_world.json leaves no vacuum cell in any world shape.
    let sun = Scene::load(&scenes_dir().join("sun_disk.json")).unwrap();
    let world = WorldConfig::default();
    assert!(sun.planes(world, &physics, &elements()).unwrap().0 == sun_disk_planes(world), "sun_disk.json ≠ preset:sun");
    let fill = Scene::load(&scenes_dir().join("fill_world.json")).unwrap();
    for (w, h) in [(64, 64), (600, 400), (2048, 64), (64, 2048)] {
        let world = WorldConfig::new(w, h).unwrap();
        let (p, _) = fill.planes(world, &physics, &elements()).unwrap();
        assert!(p[..world.cells()].iter().all(|&s| s > 0.0), "fill_world leaves vacuum in {w}x{h}");
    }
}

#[test]
fn headless_runs_a_scene_file_as_given() {
    let scene = scenes_dir().join("sun_disk.json").display().to_string();
    let out = std::env::temp_dir().join(format!("sr-scene-{}-run", std::process::id()));
    let (code, summary) =
        headless(&["--scene", &scene, "--world", "256x256", "--edge", "bounce", "--steps", "5"], &out);
    assert_eq!(code, 0);
    let summary = summary.expect("summary.json written");
    println!("summary: scene {} world {} steps {}", summary["scene"], summary["world"], summary["steps"]);
    assert_eq!(summary["scene"], scene.as_str(), "§2.12.2's scene: the argument as given");
    assert_eq!(summary["world"], serde_json::json!({"width": 256, "height": 256, "edge": "bounce"}));
    assert_eq!(summary["steps"], 5);
    // The scene's own world and edge when the flags are absent.
    let (code, summary) = headless(&["--scene", &scene, "--steps", "1"], &out);
    assert_eq!(code, 0);
    assert_eq!(summary.unwrap()["world"], serde_json::json!({"width": 600, "height": 400, "edge": "leave"}));
    // A bad --edge is a bad argument.
    assert_eq!(headless(&["--scene", &scene, "--edge", "wrap", "--steps", "1"], &out).0, 4);
}
