//! M0-T105 — touch any time (contract §5.2's G-TOUCH; §1.12, §2.12.5, §1.9.5): a star's life can be changed while it burns, and its
//! fate follows the change (B7: "Add gas to a shining star (it gets heavier and its fate changes), erase a chunk"). Two lives,
//! each a preset run under an edits file (`--edits`, §2.12.5) whose every entry fires at the first step the heaviest object's central
//! hydrogen is under 0.5 (`object_until`), the entries going out in the file's order, at most eight a step:
//! - `touch_paint`: the Sun-like preset, hydrogen painted onto its centre (radius 40, 51 frames of the brush) until it weighs
//!   about 1.25 × m_up_sb — the prediction moves from a white dwarf to a supernova, and the life ends a neutron star (or a black hole,
//!   whichever the new prediction names);
//! - `touch_erase`: the Massive preset, erased from outside in (four rings of sixteen radius-40 disks, 100, 80, 62 and 54 cells from
//!   the centre, the outer ring first) down to about m_up_sb ÷ 2 — the prediction moves from a supernova to a white dwarf, and the
//!   life ends a white dwarf. [M0-D191] Erased only to m_up_sb ÷ 1.25 (three rings), the star kept a ~3,200 sb iron remnant, just over
//!   m_ch_sb, that went neutron-rich and ended a neutron star (§0.5 [M0-D190]); erased deeper, no core can outgrow m_ch_sb (§0.5
//!   [M0-D191], the lead's «Erase deeper»).
//!
//! What a case reads, and from where (R12's one adapter):
//! - the prediction at the drop, the engine's own `Predictor` on the heaviest object's mass at the run's first frame, and the
//!   prediction after the edit: the same `Predictor` on the mass `SETTLE` steps after the first frame whose mass jumped (> 1 % on
//!   the frame before) — read after the edit has landed, never at the drop (a prediction re-read at the drop passes by luck: the
//!   star's fate would be the old one's);
//! - the post-edit prediction is sure (never a close call), names one of the fates the case allows, and is not the fate at the drop;
//! - the post-edit mass sits within `BAND` of the case's target (1.25 × or ÷ 2 m_up_sb): the edits files carry cells and frames,
//!   not masses, so a calibration or a physics change that moves the star at X_H,c = 0.5 turns this red — regenerate the file then;
//! - the ending: summary.json's `objects[until.object].ending` (non-null once the remnant held `ending_hold`, `--until ending`) is
//!   the ending that post-edit fate leads to;
//! - [M0-D180] the paint case runs in a world whose edges bounce (§0.5 [M0-D180]): the painted star outgrows the 600 × 400 world
//!   (R 170–280) and, under `leave`, 5,100–5,500 sb flow out and it ends a white dwarf. In the closed box the ejecta cannot leave
//!   and may fall back, so a supernova fate is graded as a `supernova` event of the ending's object, then a neutron star or a black
//!   hole; summary.json's `world.edge` is the case's.
//!
//! Before any life each case grades its grader on hand-made values. The lives are long (the Sun-like's to X_H,c = 0.5 alone is tens
//! of thousands of steps): R12's ⏱ scope, the lead's run.

use serde_json::Value;
use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick};
use sr_engine::scene::Scene;
use sr_physics::calibration::{self, Calibration};
use sr_physics::observe::ending::{Fate, Prediction, Predictor};

/// §1.9.5's far side of the ±10 % close-call band: the paint case's target is m_up_sb × this.
const SIDE: f64 = 1.25;
/// [M0-D191] The erase case's target is m_up_sb ÷ this (§0.5 [M0-D191]): ≈ 3,258 sb, under m_ch_sb and over m_ign_sb × 1.1 with
/// `BAND` either side — a sure white dwarf whose core cannot outgrow m_ch_sb (at ÷ `SIDE` the remnant went neutron-rich).
const ERASE_DEPTH: f64 = 2.0;
/// How far the post-edit mass may sit from its target, relative: the edits are cells and frames, so the mass is the file's, not
/// a trigger's (a mass trigger cannot wait for central hydrogen, §2.12.5). The sure window around the paint target is ± 10 % of
/// m_up_sb ÷ it; around the erase target, the band's floor (3,062 sb) stays over m_ign_sb × 1.1 (2,900 sb).
const BAND: f64 = 0.06;
/// Steps after the first frame whose mass jumped at which the post-edit mass is read: the edits land in at most eight steps (64
/// entries, eight a step).
const SETTLE: u64 = 64;
/// A frame's mass moving this much (relative) on the one before is the edit landing.
const JUMP: f64 = 0.01;

struct Case {
    id: &'static str,
    preset: &'static str,
    mass: fn(&Calibration) -> f64,
    edits: &'static str,
    /// The post-edit mass the file aims at, in sandbox units.
    target: fn(&Calibration) -> f64,
    /// The fates the post-edit prediction may be, and the fate at the drop it must differ from.
    fates: &'static [Fate],
    drop_fate: Fate,
    max_steps: u64,
    /// The world's edge (`--edge`): `leave`, or `bounce` for the paint case (§0.5 [M0-D180]).
    edge: &'static str,
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

/// The endings the post-edit `fate` may end in: the one it leads to, and in a world whose edges bounce a supernova's black hole too —
/// its ejecta cannot leave the closed box and fall back (§0.5 [M0-D180]).
fn endings_for(c: &Case, fate: Fate) -> &'static [&'static str] {
    match (c.edge, fate) {
        ("bounce", Fate::Supernova) => &["neutron_star", "black_hole"],
        (_, Fate::FailedStar) => &["failed_star"],
        (_, Fate::WhiteDwarf) => &["white_dwarf"],
        (_, Fate::Supernova) => &["neutron_star"],
        (_, Fate::BlackHole) => &["black_hole"],
    }
}

/// What one life gave: the exit code, summary.json, the mass at the first frame and the one read after the edit.
struct Life {
    code: i32,
    summary: Value,
    drop_mass: f64,
    /// `(the step of the first frame whose mass jumped, the mass SETTLE steps later)`, none when no edit ever landed.
    edited: Option<(u64, f64)>,
    steps: u64,
}

fn scenes_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

/// The case's preset at `mass` run to its ending under the case's edits file on the physics adapter, summary.json kept.
fn life(c: &Case, mass: f64) -> Life {
    let scene = Scene::preset_mass(c.preset, mass, Default::default()).unwrap();
    let out = std::env::temp_dir().join(format!("sr-touch-{}-{}", std::process::id(), c.id));
    let _ = std::fs::remove_dir_all(&out);
    let max = c.max_steps.to_string();
    let scene_label = format!("touch:{}", c.id);
    let edits = scenes_dir().join(c.edits).display().to_string();
    let out_dir = out.display().to_string();
    let mut args: Vec<String> = [
        "--scene",
        &scene_label,
        "--until",
        "ending",
        "--max-steps",
        &max,
        "--edits",
        &edits,
        "--out",
        &out_dir,
        "--edge",
        c.edge,
    ]
    .map(String::from)
    .to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let id = c.id;
    let mut drop_mass = f64::NAN;
    let mut previous = f64::NAN;
    let mut jumped: Option<u64> = None;
    let mut edited: Option<(u64, f64)> = None;
    let mut last_step = 0u64;
    let mut frame = 0u64;
    let mut tick = |t: &Tick| {
        last_step = t.step;
        frame += 1;
        let Some(star) = &t.star else { return };
        if drop_mass.is_nan() {
            drop_mass = star.mass;
            println!("{id} the drop: step={} M={:.3} sb", t.step, star.mass);
        } else if jumped.is_none() && (star.mass - previous).abs() > JUMP * previous {
            jumped = Some(t.step);
            println!(
                "{id} an edit landed: step={} M {previous:.3} -> {:.3} sb, x_h_c {:.3}",
                t.step, star.mass, star.x_h_c
            );
        }
        if let (Some(at), None) = (jumped, edited) {
            if t.step >= at + SETTLE {
                edited = Some((at, star.mass));
                println!(
                    "{id} after the edit: step={} M={:.3} sb, stage {}",
                    t.step, star.mass, t.stage
                );
            }
        }
        previous = star.mass;
        if frame.is_multiple_of(4000) {
            println!(
                "{id} step={} t={:.2} stage={} M={:.1}",
                t.step, t.sim_time, t.stage, star.mass
            );
        }
    };
    let mut saved = |_: u64, _: f64, _: &str| {};
    let progress = Progress {
        every: 16,
        tick: &mut tick,
        checkpoint_every: None,
        saved: &mut saved,
    };
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress));
    assert!(
        code == 0 || code == 2,
        "{id}: the run ended with exit {code}: {lines:?}"
    );
    let text = std::fs::read_to_string(out.join("summary.json"))
        .unwrap_or_else(|e| panic!("{id}: no summary.json: {e}"));
    let _ = std::fs::remove_dir_all(&out);
    let summary: Value = serde_json::from_str(&text).expect("summary.json parses");
    Life {
        code,
        summary,
        drop_mass,
        edited,
        steps: last_step,
    }
}

/// The grade, on plain values: `Ok` when the edit landed, the prediction at the drop is the sure `drop_fate`, the one read after the
/// edit is sure, one of `fates` and another than the drop's, the mass read after the edit is within [`BAND`] of `target`, the life
/// ran in the case's world edge and met its ending, and the object whose ending it is holds one of the endings that post-edit fate may
/// end in ([`endings_for`]) — after a `supernova` event of its own in a world whose edges bounce (§0.5 [M0-D180]).
#[allow(clippy::too_many_arguments)]
fn grade_life(
    c: &Case,
    target: f64,
    at_drop: Option<Prediction>,
    edited: Option<(u64, f64)>,
    after: Option<Prediction>,
    summary: &Value,
    code: i32,
) -> Result<(), String> {
    if at_drop != Some(Prediction::Sure(c.drop_fate)) {
        return Err(format!(
            "the prediction at the drop is {at_drop:?}, not the sure {:?}",
            c.drop_fate
        ));
    }
    let Some((_, mass)) = edited else {
        return Err("no edit ever landed: the mass never jumped (X_H,c never under 0.5, or P0 dropped the edits)".to_string());
    };
    if (mass - target).abs() > BAND * target {
        return Err(format!("the mass after the edit is {mass:.1} sb, {:.1} % from the file's target {target:.1} (band {} %): regenerate {}", 100.0 * (mass - target) / target, BAND * 100.0, c.edits));
    }
    let fate = match after {
        Some(Prediction::Sure(f)) => f,
        other => {
            return Err(format!(
                "the prediction after the edit is {other:?}, not a sure fate"
            ))
        }
    };
    if !c.fates.contains(&fate) {
        return Err(format!(
            "the prediction after the edit is the sure {fate:?}, the case owes one of {:?}",
            c.fates
        ));
    }
    if fate == c.drop_fate {
        return Err(format!(
            "the edit did not change the fate: {fate:?} at the drop and after it"
        ));
    }
    if summary["world"]["edge"] != c.edge {
        return Err(format!(
            "the life ran with edge {}, the case's is {}",
            summary["world"]["edge"], c.edge
        ));
    }
    if code != 0 || summary["until"]["met"] != true {
        return Err(format!(
            "no ending within the step cap (exit {code}, until {})",
            summary["until"]
        ));
    }
    let object = &summary["until"]["object"];
    let endings = endings_for(c, fate);
    if c.edge == "bounce" && fate == Fate::Supernova {
        let exploded = summary["events"]
            .as_array()
            .is_some_and(|es| es.iter().any(|e| e["kind"] == "supernova" && e["object"] == *object));
        if !exploded {
            return Err(format!(
                "no supernova event of the ending's object {object}: a supernova fate in a world whose edges bounce is graded on it (events {})",
                summary["events"]
            ));
        }
    }
    let held = summary["objects"]
        .as_array()
        .and_then(|os| os.iter().find(|o| o["id"] == *object))
        .map(|o| o["ending"].clone());
    match held.as_ref().and_then(Value::as_str) {
        Some(got) if endings.contains(&got) => Ok(()),
        Some(got) => Err(format!(
            "ended {got}, the prediction after the edit ({fate:?}) leads to one of {endings:?}"
        )),
        None => Err(format!(
            "the ending object {object} holds no ended remnant (objects {})",
            summary["objects"]
        )),
    }
}

/// The grader on hand-made values, before any life: the right life passes; an edit that never landed, a mass off its target either
/// side, the old fate kept, a close call, a fate the case does not allow, the prediction read at the drop, an ending of the old fate,
/// an ending not held and a life that met nothing are each refused.
fn grader_is_graded(c: &Case) {
    let target = 5000.0;
    let new = c.fates[0];
    let sure = Some(Prediction::Sure(new));
    let old = Some(Prediction::Sure(c.drop_fate));
    let ending = |e: Value| serde_json::json!({"world": {"edge": c.edge}, "until": {"met": true, "object": 3}, "events": [{"kind": "supernova", "object": 3}], "objects": [{"id": 3, "ending": e}]});
    let ok = ending(leads_to(new).into());
    let landed = Some((100, target));
    assert!(
        grade_life(c, target, old, landed, sure, &ok, 0).is_ok(),
        "the right life refused"
    );
    assert!(
        grade_life(c, target, old, None, sure, &ok, 0).is_err(),
        "accepted a life no edit landed in"
    );
    for off in [target * (1.0 + 2.0 * BAND), target * (1.0 - 2.0 * BAND)] {
        assert!(
            grade_life(c, target, old, Some((100, off)), sure, &ok, 0).is_err(),
            "accepted a mass {off} off its target {target}"
        );
    }
    assert!(
        grade_life(c, target, old, landed, old, &ok, 0).is_err(),
        "accepted the old fate kept"
    );
    assert!(
        grade_life(
            c,
            target,
            old,
            landed,
            old,
            &ending(leads_to(c.drop_fate).into()),
            0
        )
        .is_err(),
        "accepted the old fate's ending"
    );
    let close = Some(Prediction::CloseCall(new, c.drop_fate));
    assert!(
        grade_life(c, target, old, landed, close, &ok, 0).is_err(),
        "accepted a close call after the edit"
    );
    assert!(
        grade_life(
            c,
            target,
            old,
            landed,
            Some(Prediction::Sure(Fate::FailedStar)),
            &ok,
            0
        )
        .is_err(),
        "accepted a fate the case does not allow"
    );
    assert!(
        grade_life(c, target, old, landed, None, &ok, 0).is_err(),
        "accepted no prediction after the edit"
    );
    assert!(
        grade_life(c, target, sure, landed, sure, &ok, 0).is_err(),
        "accepted a drop that already read the new fate"
    );
    let other = if leads_to(new) == "white_dwarf" {
        "black_hole"
    } else {
        "white_dwarf"
    };
    assert!(
        grade_life(c, target, old, landed, sure, &ending(other.into()), 0).is_err(),
        "accepted an ending of another fate"
    );
    assert!(
        grade_life(c, target, old, landed, sure, &ending(Value::Null), 0).is_err(),
        "accepted an ending not held"
    );
    let unmet = serde_json::json!({"world": {"edge": c.edge}, "until": {"met": false, "object": 3}, "events": [{"kind": "supernova", "object": 3}], "objects": [{"id": 3, "ending": leads_to(new)}]});
    assert!(
        grade_life(c, target, old, landed, sure, &unmet, 2).is_err(),
        "accepted a life that met no ending"
    );
    let other_object = serde_json::json!({"world": {"edge": c.edge}, "until": {"met": true, "object": 9}, "events": [{"kind": "supernova", "object": 9}], "objects": [{"id": 3, "ending": leads_to(new)}]});
    assert!(
        grade_life(c, target, old, landed, sure, &other_object, 0).is_err(),
        "read an ending off another object"
    );
    let mut other_edge = ok.clone();
    other_edge["world"]["edge"] = if c.edge == "bounce" { "leave" } else { "bounce" }.into();
    assert!(
        grade_life(c, target, old, landed, sure, &other_edge, 0).is_err(),
        "accepted a life run with another edge than the case's"
    );
    // §0.5 [M0-D180]: in the closed box a supernova fate ends a neutron star or a black hole, after a supernova event of its own.
    if c.edge == "bounce" && new == Fate::Supernova {
        assert!(
            grade_life(c, target, old, landed, sure, &ending("black_hole".into()), 0).is_ok(),
            "refused a supernova's black hole in the closed box"
        );
        for (events, what) in [
            (serde_json::json!([]), "no supernova event"),
            (serde_json::json!([{"kind": "supernova", "object": 4}]), "another object's supernova"),
            (serde_json::json!([{"kind": "core_collapse", "object": 3}]), "a collapse with no supernova"),
        ] {
            for remnant in ["neutron_star", "black_hole"] {
                let mut life = ending(remnant.into());
                life["events"] = events.clone();
                assert!(
                    grade_life(c, target, old, landed, sure, &life, 0).is_err(),
                    "accepted {remnant} after {what}"
                );
            }
        }
    }
}

fn run_case(c: &Case) {
    grader_is_graded(c);
    let cal = calibration::resolve(calibration::EMBEDDED, true)
        .unwrap()
        .expect("a calibration");
    let predictor = Predictor::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb)
        .expect("calibration's anchors are in order");
    let (mass, target) = ((c.mass)(&cal), (c.target)(&cal));
    println!("{} {} preset of {mass:.3} sb under {} — the edit aims at {target:.1} sb (m_up {:.1}), the fate leaving {:?} for one of {:?}", c.id, c.preset, c.edits, cal.m_up_sb, c.drop_fate, c.fates);
    let l = life(c, mass);
    let at_drop = predictor.predict(l.drop_mass);
    let after = l.edited.and_then(|(_, m)| predictor.predict(m));
    println!("{} the readout at the drop (M {:.3}): {at_drop:?}; after the edit {:?}: {after:?}; {} steps, exit {}, summary ending {}", c.id, l.drop_mass, l.edited, l.steps, l.code, l.summary["objects"]);
    if let Err(e) = grade_life(c, target, at_drop, l.edited, after, &l.summary, l.code) {
        panic!("{}: {e}", c.id);
    }
}

#[test]
fn hydrogen_painted_onto_the_sun_like_star_ends_it_a_neutron_star_or_a_black_hole_as_its_new_prediction_says(
) {
    run_case(&Case {
        id: "paint",
        preset: "sun",
        mass: |c| c.preset_mass_sb.sun,
        edits: "touch_paint.json",
        target: |c| c.m_up_sb * SIDE,
        fates: &[Fate::Supernova, Fate::BlackHole],
        drop_fate: Fate::WhiteDwarf,
        max_steps: 600_000,
        edge: "bounce",
    });
}

#[test]
fn the_massive_star_erased_from_outside_in_ends_a_white_dwarf_as_its_new_prediction_says() {
    run_case(&Case {
        id: "erase",
        preset: "massive",
        mass: |c| c.preset_mass_sb.massive,
        edits: "touch_erase.json",
        target: |c| c.m_up_sb / ERASE_DEPTH,
        fates: &[Fate::WhiteDwarf],
        drop_fate: Fate::Supernova,
        max_steps: 600_000,
        edge: "leave",
    });
}
