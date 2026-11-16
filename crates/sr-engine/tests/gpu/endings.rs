//! M0-T104 — each ending as predicted (contract §5.2's G-END; §1.9.5, §2.12.2): the three presets at calibration's masses end as
//! the readout predicted at the drop (the Sun-like a white dwarf; the Massive a supernova and then a neutron star; the Giant a
//! black hole), and a Sun-like-shaped cloud at each calibrated threshold ÷ 1.25 and × 1.25 in sandbox mass — outside the ±10 %
//! close-call band — ends on its side, as predicted: failed star against white dwarf (m_ign_sb), white dwarf against neutron
//! star (m_up_sb), neutron star against black hole (m_bh_sb). Nine cases, one life each, every one a whole life to its ending
//! (`--until ending`: the remnant stage held for `ending_hold`, §0.5 [M0-D78] — a stage passing through is no ending).
//!
//! What a case reads, and from where:
//! - the prediction at the drop: the engine's own `Predictor` (§1.9.5, the one summary.json's `predicted_ending` and the readout
//!   panel use) on the heaviest object's mass at the run's first frame, read through the progress hook — a sure fate, never a close
//!   call, and the fate the case's side of its threshold says (the case's own table, never read back from the predictor);
//! - the ending: summary.json's `objects[until.object].ending` — non-null only once the remnant held — which must be the ending the
//!   fate leads to (a supernova leads to a neutron star: a black hole is the other branch the prediction would name);
//! - `standins`: summary.json records whether S1 or S2 was enabled (§5.2); the shipped physics has both off.
//!
//! Before any life each case grades the grader on hand-made values (an ending other than the predicted, a close call where a sure
//! fate is owed, an ending not yet held, a stand-in on). The lives are long — the Sun-like's about 197,000 steps, over ten minutes:
//! R12's ⏱ scope, the lead's run.

use serde_json::Value;
use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick};
use sr_engine::scene::Scene;
use sr_physics::calibration::{self, Calibration};
use sr_physics::observe::ending::{Fate, Prediction, Predictor};

/// What a case expects: the preset whose cloud it runs, the mass (from calibration), the fate §1.9.5 predicts for that mass and the
/// ending that fate leads to.
struct Case {
    id: &'static str,
    preset: &'static str,
    /// The mass in sandbox units from calibration (the preset's own, or a threshold's ÷ or × 1.25).
    mass: fn(&Calibration) -> f64,
    fate: Fate,
    ending: &'static str,
    max_steps: u64,
}

/// The ending a fate leads to: a supernova, then a neutron star (§1.9.5).
fn leads_to(fate: Fate) -> &'static str {
    match fate {
        Fate::FailedStar => "failed_star",
        Fate::WhiteDwarf => "white_dwarf",
        Fate::Supernova => "neutron_star",
        Fate::BlackHole => "black_hole",
    }
}

/// The band's far side of §1.9.5's ±10 %: ÷ and × this keep a cloud outside the close call by 2.3 × the band.
const SIDE: f64 = 1.25;

/// What one life gave: the run's exit code, its summary.json and the heaviest object's mass at the first frame.
struct Life {
    code: i32,
    summary: Value,
    drop_mass: f64,
    steps: u64,
}

/// A Sun-like-shaped (or the named preset's) cloud of `mass` run to its ending on the physics adapter, summary.json kept.
fn life(id: &str, preset: &str, mass: f64, max_steps: u64) -> Life {
    let scene = Scene::preset_mass(preset, mass, Default::default()).unwrap();
    let out = std::env::temp_dir().join(format!("sr-endings-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let max = max_steps.to_string();
    let scene_label = format!("endings:{id}");
    let out_dir = out.display().to_string();
    let mut args: Vec<String> = ["--scene", &scene_label, "--until", "ending", "--max-steps", &max, "--out", &out_dir].map(String::from).to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let mut drop_mass = f64::NAN;
    let mut last_step = 0u64;
    let mut frame = 0u64;
    let mut tick = |t: &Tick| {
        last_step = t.step;
        frame += 1;
        if drop_mass.is_nan() {
            if let Some(s) = &t.star {
                drop_mass = s.mass;
                println!("{id} the drop: step={} M={:.3} sb", t.step, s.mass);
            }
        } else if frame.is_multiple_of(4000) {
            println!("{id} step={} t={:.2} stage={}", t.step, t.sim_time, t.stage);
        }
    };
    let mut saved = |_: u64, _: f64, _: &str| {};
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: None, saved: &mut saved };
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress));
    assert!(code == 0 || code == 2, "{id}: the run ended with exit {code}: {lines:?}");
    let text = std::fs::read_to_string(out.join("summary.json")).unwrap_or_else(|e| panic!("{id}: no summary.json: {e}"));
    let _ = std::fs::remove_dir_all(&out);
    let summary: Value = serde_json::from_str(&text).expect("summary.json parses");
    Life { code, summary, drop_mass, steps: last_step }
}

/// The grade, on plain values: `Ok` when the prediction read at the drop is the sure `fate`, the life met its ending, the object
/// whose ending it is holds `ending` (its summary `ending` non-null: the remnant held), and neither stand-in was enabled.
fn grade_life(fate: Fate, ending: &str, predicted: Option<Prediction>, summary: &Value, code: i32) -> Result<(), String> {
    if predicted != Some(Prediction::Sure(fate)) {
        return Err(format!("the prediction at the drop is {predicted:?}, not the sure {fate:?}"));
    }
    if code != 0 || summary["until"]["met"] != true {
        return Err(format!("no ending within the step cap (exit {code}, until {})", summary["until"]));
    }
    let object = &summary["until"]["object"];
    let held = summary["objects"].as_array().and_then(|os| os.iter().find(|o| o["id"] == *object)).map(|o| o["ending"].clone());
    match held.as_ref().and_then(Value::as_str) {
        Some(got) if got == ending => {}
        Some(got) => return Err(format!("ended {got}, the prediction ({fate:?}) leads to {ending}")),
        None => return Err(format!("the ending object {object} holds no ended remnant (objects {})", summary["objects"])),
    }
    if summary["standins"] != serde_json::json!({"s1": false, "s2": false}) {
        return Err(format!("summary.json's standins read {}, not both off", summary["standins"]));
    }
    Ok(())
}

/// The grader on hand-made values, before any life: the right ending passes; the other three endings, a close call, an ending not
/// held, a life that met nothing, a missing standins record and a stand-in enabled are each refused.
fn grader_is_graded() {
    let object = |ending: Value| serde_json::json!({"id": 3, "ending": ending});
    let summary = |ending: Value, met: bool, standins: Value| {
        serde_json::json!({"until": {"met": met, "object": 3}, "objects": [object(ending)], "standins": standins})
    };
    let off = serde_json::json!({"s1": false, "s2": false});
    let sure = Some(Prediction::Sure(Fate::WhiteDwarf));
    let ok = summary("white_dwarf".into(), true, off.clone());
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &ok, 0).is_ok(), "the right ending refused");
    for other in ["failed_star", "neutron_star", "black_hole"] {
        let s = summary(other.into(), true, off.clone());
        assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &s, 0).is_err(), "accepted {other} for a white dwarf");
    }
    let close = Some(Prediction::CloseCall(Fate::WhiteDwarf, Fate::Supernova));
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", close, &ok, 0).is_err(), "accepted a close call where a sure fate is owed");
    let other_fate = Some(Prediction::Sure(Fate::Supernova));
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", other_fate, &ok, 0).is_err(), "accepted a prediction of another fate");
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", None, &ok, 0).is_err(), "accepted no prediction");
    let unheld = summary(Value::Null, true, off.clone());
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &unheld, 0).is_err(), "accepted an ending not held");
    let unmet = summary("white_dwarf".into(), false, off.clone());
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &unmet, 2).is_err(), "accepted a life that met no ending");
    let missing = summary("white_dwarf".into(), true, Value::Null);
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &missing, 0).is_err(), "accepted a summary without standins");
    let on = summary("white_dwarf".into(), true, serde_json::json!({"s1": true, "s2": false}));
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &on, 0).is_err(), "accepted S1 enabled");
    assert_eq!(leads_to(Fate::Supernova), "neutron_star");
    let other_object = serde_json::json!({"until": {"met": true, "object": 9}, "objects": [object("white_dwarf".into())], "standins": off});
    assert!(grade_life(Fate::WhiteDwarf, "white_dwarf", sure, &other_object, 0).is_err(), "read an ending off another object");
}

fn run_case(c: &Case) {
    grader_is_graded();
    let cal = calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration");
    let predictor = Predictor::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).expect("calibration's anchors are in order");
    let mass = (c.mass)(&cal);
    println!("{} {} cloud of {mass:.3} sb (m_ign {:.1}, m_up {:.1}, m_bh {:.1}) — expected {:?} then {}", c.id, c.preset, cal.m_ign_sb, cal.m_up_sb, cal.m_bh_sb, c.fate, c.ending);
    let l = life(c.id, c.preset, mass, c.max_steps);
    let predicted = predictor.predict(l.drop_mass);
    println!("{} the readout at the drop (M {:.3}): {predicted:?}; {} steps, exit {}, summary ending {}", c.id, l.drop_mass, l.steps, l.code, l.summary["objects"]);
    if let Err(e) = grade_life(c.fate, c.ending, predicted, &l.summary, l.code) {
        panic!("{}: {e}", c.id);
    }
}

#[test]
fn the_sun_like_preset_ends_a_white_dwarf_as_predicted() {
    run_case(&Case { id: "sun", preset: "sun", mass: |c| c.preset_mass_sb.sun, fate: Fate::WhiteDwarf, ending: "white_dwarf", max_steps: 400_000 });
}

#[test]
fn the_massive_preset_ends_a_supernova_then_a_neutron_star_as_predicted() {
    run_case(&Case { id: "massive", preset: "massive", mass: |c| c.preset_mass_sb.massive, fate: Fate::Supernova, ending: "neutron_star", max_steps: 120_000 });
}

#[test]
fn the_giant_preset_ends_a_black_hole_as_predicted() {
    run_case(&Case { id: "giant", preset: "giant", mass: |c| c.preset_mass_sb.giant, fate: Fate::BlackHole, ending: "black_hole", max_steps: 100_000 });
}

#[test]
fn a_cloud_at_m_ign_over_1_25_fails_to_light() {
    run_case(&Case { id: "ign_under", preset: "sun", mass: |c| c.m_ign_sb / SIDE, fate: Fate::FailedStar, ending: "failed_star", max_steps: 400_000 });
}

#[test]
fn a_cloud_at_m_ign_times_1_25_ends_a_white_dwarf() {
    run_case(&Case { id: "ign_over", preset: "sun", mass: |c| c.m_ign_sb * SIDE, fate: Fate::WhiteDwarf, ending: "white_dwarf", max_steps: 400_000 });
}

#[test]
fn a_cloud_at_m_up_over_1_25_ends_a_white_dwarf() {
    run_case(&Case { id: "up_under", preset: "sun", mass: |c| c.m_up_sb / SIDE, fate: Fate::WhiteDwarf, ending: "white_dwarf", max_steps: 400_000 });
}

#[test]
fn a_cloud_at_m_up_times_1_25_ends_a_neutron_star() {
    run_case(&Case { id: "up_over", preset: "sun", mass: |c| c.m_up_sb * SIDE, fate: Fate::Supernova, ending: "neutron_star", max_steps: 400_000 });
}

#[test]
fn a_cloud_at_m_bh_over_1_25_ends_a_neutron_star() {
    run_case(&Case { id: "bh_under", preset: "sun", mass: |c| c.m_bh_sb / SIDE, fate: Fate::Supernova, ending: "neutron_star", max_steps: 400_000 });
}

#[test]
fn a_cloud_at_m_bh_times_1_25_ends_a_black_hole() {
    run_case(&Case { id: "bh_over", preset: "sun", mass: |c| c.m_bh_sb * SIDE, fate: Fate::BlackHole, ending: "black_hole", max_steps: 400_000 });
}
