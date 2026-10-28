//! M0-T74 — the three presets at calibration's masses and several clouds in one world (contract §2.10, §3.3, §5.2's G-MULTI).
//! The presets' planes are the scene's own (no GPU); the runs go through `headless::run_captured` on the physics adapter
//! (R12), their stdout lines kept so `SR-ADAPTER …` is in the log (R13). Two Sun-like presets 240 cells apart
//! (scenes/two_suns.json) run until `sim_time:40` — [M0-T74]: not the contract's 20 τ_dyn (≈ 72 t.u.), by which gravity
//! has pulled the two clouds into one object (measured: two objects at t = 40, one at t = 72; the lead, 2026-10-28,
//! «Shorten to 40 t.u. (Recommended)») — and must stay two objects with the ids they began with, finite, each reading out.

use std::path::{Path, PathBuf};

use serde_json::Value;
use sr_engine::headless::run_captured;
use sr_engine::scene::{cloud_shape, Object, Scene};
use sr_engine::state::WorldConfig;
use sr_physics::calibration::Calibration;
use sr_physics::observe::age::AgeClock;
use sr_physics::observe::ending::{Fate, Prediction, Predictor};
use sr_physics::observe::tracker::Object as Tracked;
use sr_physics::observe::translate::MassScale;
use sr_physics::observe::Stage;
use sr_physics::registry::{Elements, Physics};

/// The run's end, in t.u., and the run that fixes the ids the objects begin with, in steps.
const RUN_T: f64 = 40.0;
const EARLY_STEPS: &str = "32";
/// §5.2: the two presets' distance, in cells, and the gap of empty cells the scene must leave between their edges
/// (a block is 4 cells, §1.9.2: far more than the one empty block that separates two objects).
const APART: f64 = 240.0;
const MIN_GAP: f64 = 100.0;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world() -> WorldConfig {
    WorldConfig::new(600, 400).unwrap()
}

fn calibration() -> Calibration {
    let text = std::fs::read_to_string(root().join("assets/calibration.json")).expect("assets/calibration.json");
    Calibration::parse(&text).expect("the shipped calibration parses")
}

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-multi-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `headless <args…> --out <dir>` on the physics adapter: the exit code, stdout lines and summary.json.
fn headless(out: &Path, args: &[&str]) -> (i32, Vec<String>, Value) {
    let mut args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    args.extend(["--out".to_string(), out.display().to_string()]);
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let (code, lines) = run_captured(&args);
    let summary = std::fs::read(out.join("summary.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    (code, lines, summary)
}

fn two_suns() -> PathBuf {
    root().join("scenes/two_suns.json")
}

fn ids(summary: &Value) -> Vec<u64> {
    let mut ids: Vec<u64> = summary["objects"].as_array().expect("objects").iter().map(|o| o["id"].as_u64().expect("id")).collect();
    ids.sort_unstable();
    ids
}

/// §2.10: each preset's cloud holds calibration's `preset_mass_sb` (a scene's presets are read at it, through
/// [`Scene::with_preset_masses`]) and its radius follows [`cloud_shape`]; the planes hold that mass, whichever preset.
#[test]
fn the_three_presets_hold_calibrations_masses() {
    let cal = calibration();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let masses = [("sun", cal.preset_mass_sb.sun), ("massive", cal.preset_mass_sb.massive), ("giant", cal.preset_mass_sb.giant)];
    for (name, mass) in masses {
        let scene = Scene::preset(name, world()).unwrap().with_preset_masses(&cal.preset_mass_sb);
        let held = match scene.objects.as_slice() {
            [Object::Preset { mass: m, .. }] => *m,
            other => panic!("preset:{name} is one preset object, got {other:?}"),
        };
        assert_eq!(held, mass, "preset:{name}'s mass is calibration's");
        let (planes, warnings) = scene.planes(world(), &physics, &elements).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(scene.warnings().is_empty(), "{name}: nothing nominal is left to warn about: {:?}", scene.warnings());
        let sum: f64 = planes[..world().cells()].iter().map(|&s| f64::from(s)).sum();
        println!("preset:{name}: calibrated mass {mass:.3}, the cells hold {sum:.3}, a = {:.2}", cloud_shape(mass).a);
        assert!((sum - mass).abs() <= 1e-5 * mass, "preset:{name} holds {sum}, not {mass}");
    }
    assert!(cal.preset_mass_sb.sun < cal.preset_mass_sb.massive && cal.preset_mass_sb.massive < cal.preset_mass_sb.giant);
}

/// `headless --scene preset:<name>` loads the calibrated mass: the ledger's start mass is `preset_mass_sb`, and no
/// `SR-WARN` says a preset is still the nominal disk.
#[test]
fn headless_presets_start_at_calibrations_masses() {
    let cal = calibration();
    let masses = [("sun", cal.preset_mass_sb.sun), ("massive", cal.preset_mass_sb.massive), ("giant", cal.preset_mass_sb.giant)];
    for (name, mass) in masses {
        let out = temp(&format!("preset-{name}")).join("out");
        let (code, lines, summary) = headless(&out, &["--scene", &format!("preset:{name}"), "--steps", "1"]);
        assert_eq!(code, 0, "preset:{name}: {:?}", lines.last());
        let start = summary["ledger"]["start"]["mass"]["grid"].as_f64().expect("ledger start mass");
        println!("preset:{name}: ledger start mass {start:.3} against calibration's {mass:.3}");
        assert!((start - mass).abs() <= 1e-5 * mass, "preset:{name} starts at {start}, not {mass}");
        assert!(!lines.iter().any(|l| l.contains("nominal")), "preset:{name}: {lines:?}");
    }
}

/// Adversarial: two presets close enough for their clouds to touch make one object by design — so the scene's distance
/// is asserted against each cloud's radius before the run: the gap between the edges is ≥ [`MIN_GAP`] cells, the columns
/// between hold no gas, and each cloud holds the Sun-like mass alone.
#[test]
fn the_two_suns_are_far_enough_apart_not_to_be_one_cloud() {
    let cal = calibration();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let scene = Scene::load(&two_suns()).unwrap().with_preset_masses(&cal.preset_mass_sb);
    let at: Vec<(f64, f64, f64)> = scene
        .objects
        .iter()
        .map(|o| match o {
            Object::Preset { preset, x, y, mass } => {
                assert_eq!(preset, "sun");
                (*x, *y, *mass)
            }
            other => panic!("two_suns.json holds presets only: {other:?}"),
        })
        .collect();
    assert_eq!(at.len(), 2, "two presets");
    assert_eq!(at[0].1, at[1].1, "side by side");
    let (a1, a2) = (cloud_shape(at[0].2).a, cloud_shape(at[1].2).a);
    let apart = at[1].0 - at[0].0;
    let gap = apart - a1 - a2;
    println!("two_suns: {apart} cells apart, a = {a1:.2} and {a2:.2}: {gap:.2} cells of vacuum between the edges");
    assert_eq!(apart, APART, "§5.2's 240 cells");
    assert!(gap >= MIN_GAP, "the clouds are {gap} cells apart edge to edge, under {MIN_GAP}");
    let (planes, warnings) = scene.planes(world(), &physics, &elements).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let w = world().width as usize;
    let sigma = &planes[..world().cells()];
    let (lo, hi) = ((at[0].0 + a1).ceil() as usize, (at[1].0 - a2).floor() as usize);
    for (i, &s) in sigma.iter().enumerate() {
        let x = i % w;
        assert!(x < lo || x >= hi || s == 0.0, "gas at x = {x} between the clouds");
    }
    for (k, m) in [(0usize, at[0].2), (1, at[1].2)] {
        let held: f64 = sigma.iter().enumerate().filter(|(i, _)| ((i % w) as f64 > at[0].0 + 0.5 * apart) == (k == 1)).map(|(_, &s)| f64::from(s)).sum();
        assert!((held - m).abs() <= 1e-5 * m, "cloud {k} holds {held}, not {m}");
    }
}

/// G-MULTI: the two Sun-like presets, 240 cells apart, to `sim_time:40` — two objects, ids as they began (an early run's),
/// every mass finite, no non-finite value anywhere (exit 5 is P9's guard and the last step's scan) and both readouts filled:
/// the mass in Suns, the predicted ending (a Sun-like end: white dwarf) and the age.
#[test]
fn two_suns_stay_two_objects_with_stable_ids() {
    let cal = calibration();
    let early = temp("early").join("out");
    let (code, lines, first) = headless(&early, &["--scene", &two_suns().display().to_string(), "--steps", EARLY_STEPS]);
    assert_eq!(code, 0, "{:?}", lines.last());
    let began = ids(&first);
    assert_eq!(began.len(), 2, "two objects from the drop: {:?}", first["objects"]);

    let out = temp("run").join("out");
    let until = format!("sim_time:{RUN_T}");
    let (code, lines, summary) = headless(&out, &["--scene", &two_suns().display().to_string(), "--until", &until, "--max-steps", "100000"]);
    assert_eq!(code, 0, "{:?}", lines.last());
    assert!(lines.last().unwrap().ends_with("until=met"), "{:?}", lines.last());
    let sim_time = summary["sim_time"].as_f64().expect("sim_time");
    println!("two_suns to t = {sim_time:.3} in {} steps", summary["steps"]);
    assert!(sim_time >= RUN_T, "{sim_time}");
    assert_eq!(ids(&summary), began, "the ids the objects began with, none new and none gone: {:?}", summary["objects"]);

    let scale = MassScale::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).unwrap();
    let predictor = Predictor::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).unwrap();
    let mut clock = AgeClock::from_calibration(&cal).unwrap();
    let objects = summary["objects"].as_array().unwrap();
    let tracked: Vec<Tracked> = objects.iter().map(|o| Tracked { id: o["id"].as_u64().unwrap() as u32, mass: o["mass_sb"].as_f64().unwrap(), ..Tracked::default() }).collect();
    let shown = |id: u32, last: bool| {
        let o = objects.iter().find(|o| o["id"].as_u64() == Some(u64::from(id)))?;
        let stages = o["stages"].as_array()?;
        let pick = if last { stages.last()? } else { stages.first()? };
        Stage::from_id(&pick["stage"]).ok()
    };
    clock.update(0.0, 0.0, &tracked, |id| shown(id, false));
    clock.update(sim_time, 1.0, &tracked, |id| shown(id, true));
    for o in objects {
        let (id, mass) = (o["id"].as_u64().unwrap() as u32, o["mass_sb"].as_f64().unwrap());
        assert!(mass.is_finite() && mass > 0.0, "object {id}: mass {mass}");
        let suns = scale.to_suns(mass);
        let age = clock.age(id);
        let fate = predictor.predict(mass);
        println!("object {id}: {mass:.1} sb = {suns:.3} Suns, {fate:?}, age {age:?} years, stage {}", o["stage"]);
        assert!(suns.is_finite() && (0.5..2.0).contains(&suns), "object {id}: {suns} Suns");
        assert_eq!(fate, Some(Prediction::Sure(Fate::WhiteDwarf)), "object {id}'s predicted ending");
        assert!(age.is_some_and(|y| y.is_finite() && y > 0.0), "object {id}'s age: {age:?}");
    }
}

/// The tracker through the engine: two clouds one empty block (4 cells) apart are two objects, never one — the case the
/// `one-object` plant (the link reaching across an empty block) fails. Two 24 × 24 rects 10 cells apart in a 128 × 128
/// world, after one step: the gas has spread into the gap by then, and measured on the RTX 5090 the clouds are one object
/// at a gap of 8 or less and two from 9, while the plant's link joins them up to 12 — 10 sits inside the window, one empty
/// block (4 cells) left between the clouds.
#[test]
fn clouds_one_empty_block_apart_are_two_objects() {
    let dir = temp("block");
    let scene = dir.join("block_apart.json");
    std::fs::write(
        &scene,
        r#"{"version": 1, "world": {"width": 128, "height": 128, "edge": "leave"},
 "objects": [
   {"kind": "rect", "x0": 40, "y0": 50, "w": 24, "h": 24, "sigma": 1.0, "t": 0.5, "composition": {"H": 1.0}, "v": [0, 0]},
   {"kind": "rect", "x0": 74, "y0": 50, "w": 24, "h": 24, "sigma": 1.0, "t": 0.5, "composition": {"H": 1.0}, "v": [0, 0]}
 ]}"#,
    )
    .unwrap();
    let (code, lines, summary) = headless(&dir.join("out"), &["--scene", &scene.display().to_string(), "--steps", "1"]);
    assert_eq!(code, 0, "{:?}", lines.last());
    println!("{}", summary["objects"]);
    assert_eq!(ids(&summary).len(), 2, "two clouds with one empty block between: {:?}", summary["objects"]);
}
