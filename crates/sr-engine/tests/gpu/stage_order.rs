//! M0-D134 — the stage a live preset life shows (contract §1.9.3, §1.9.4, §4.6 as §0.5 [M0-D134] reads them): the Sun-like
//! and the 15-Sun preset at calibration's masses, run headless through the run's progress hook, the heaviest object read every
//! frame of 16 steps — its raw stage, its shown stage and the numbers §1.9.3's rows read. The app's star readout runs the same
//! tracker and stages (crates/sr-app/src/ui/right.rs `Stars`: sr_physics's Tracker and StageTracker), so the shown sequence here
//! is the readout's.
//!
//! Measured before the fix on linux-pc's lavapipe (logs/M0-D134.measure.log): the Sun-like preset read failed_star from step 17
//! to 161 and the 15-Sun from 17 to 193, in free fall with d_c 0.55–0.65; the Sun-like showed collapsing, then cloud again
//! after its ignition (step 321 on).
//!
//! M0-D140: the Sun-like then showed white_dwarf from step 433 — re-expanding at +0.2 √(GM/R) after its flash, its hydrogen
//! unburnt (X_H,c 0.7318), on lavapipe and the RTX 4090 alike (logs/M0-D140.measure.log); §0.5 [M0-D140] makes the remnant
//! rows settled. The Sun-like runs 2,000 steps since.
//!
//! M0-D141: with no false white dwarf left, the Sun-like showed «collapsing» from step 65 to 2,000 — ignited at step 273, its
//! rows raw cloud on every frame after, burning from nothing to 86 times what it shines, never inside main_sequence's
//! [0.5 L, 2 L] (logs/M0-D140.iterate.log); §0.5 [M0-D141] reads main_sequence by its largest burning group.

use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick};
use sr_engine::scene::Scene;
use sr_physics::calibration;

/// Where a label sits in the life before ignition: cloud, collapsing, protostar — forward only (§0.5 [M0-D134]).
fn early_rank(stage: &str) -> Option<u8> {
    match stage {
        "cloud" => Some(0),
        "collapsing" => Some(1),
        "protostar" => Some(2),
        _ => None,
    }
}

/// A remnant shown on a star that is none (§0.5 [M0-D140]): the step, its X_H,c and whether h_shell had shown before.
type FalseRemnant = (u64, f64, bool);

/// A frame of an ignited star still showing a label from before its ignition, or none (§0.5 [M0-D141]): the step, the frames
/// since its ignition and the shown stage.
type Unlit = (u64, u64, &'static str);

/// The frames an ignited star may take to show its stage: the display hold's 3 (§1.9.3).
const LIGHT_FRAMES: u64 = 3;

/// What a life showed: its stages with the step each first showed at, its false remnants and its unlit frames.
type Life = (Vec<(String, u64)>, Vec<FalseRemnant>, Vec<Unlit>);

/// `preset` at calibration's mass for `steps` steps on the physics adapter: the stages it showed, each with the step it
/// first showed at, the frames that showed white_dwarf with hydrogen at the centre (X_H,c ≥ 10⁻³) or before h_shell, and
/// those that showed cloud, collapsing, protostar or nothing more than [`LIGHT_FRAMES`] frames after the star's ignition;
/// each frame's numbers printed as they come.
fn life(preset: &str, steps: u64) -> Life {
    let cal = calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration");
    let scene = Scene::preset(preset, Default::default()).unwrap().with_preset_masses(&cal.preset_mass_sb);
    let out = std::env::temp_dir().join(format!("sr-stage-order-{}-{preset}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    // An `--until` the run never meets, so the observers read a frame of 16 steps (headless's FRAME), not a chunk of 256;
    // `--max-steps` ends it (exit 2, unmet).
    let max = steps.to_string();
    let mut args: Vec<String> =
        ["--scene", &format!("life:{preset}"), "--until", "stage:black_hole", "--max-steps", &max, "--out", &out.display().to_string()]
            .map(String::from)
            .to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let mut shown: Vec<(String, u64)> = Vec::new();
    let mut false_remnants: Vec<FalseRemnant> = Vec::new();
    let mut unlit: Vec<Unlit> = Vec::new();
    let mut frame = 0u64;
    let mut lit_at: Option<u64> = None;
    let mut tick = |t: &Tick| {
        let Some(s) = &t.star else { return };
        println!(
            "{preset} step={} lit={} raw={} shown={} M={:.1} R={:.2} Σc={:.3e} Tc={:.3e} d_c={:.4} X_H,c={:.4} v_r/v_dyn={:+.4} 2P/|W|={:.3} \
             L={:.3e} L_nuc={:.3e}",
            t.step,
            s.ignited,
            s.raw.unwrap_or("none"),
            s.shown.unwrap_or("none"),
            s.mass,
            s.radius,
            s.sigma_c,
            s.t_c,
            s.d_c,
            s.x_h_c,
            s.v_r_dyn,
            s.virial,
            s.luminosity,
            s.l_nuc
        );
        if let Some(stage) = s.shown.filter(|st| shown.last().is_none_or(|(last, _)| last != st)) {
            shown.push((stage.to_string(), t.step));
        }
        let after_h_shell = shown.iter().any(|(st, _)| st == "h_shell");
        if s.shown == Some("white_dwarf") && (s.x_h_c >= 1e-3 || !after_h_shell) {
            false_remnants.push((t.step, s.x_h_c, after_h_shell));
        }
        if s.ignited {
            let since = frame - *lit_at.get_or_insert(frame);
            let label = s.shown.unwrap_or("none");
            if since > LIGHT_FRAMES && (label == "none" || early_rank(label).is_some()) {
                unlit.push((t.step, since, label));
            }
        }
        frame += 1;
    };
    let mut saved = |_: u64, _: f64, _: &str| {};
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: None, saved: &mut saved };
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress));
    assert_eq!(code, 2, "{lines:?}");
    let _ = std::fs::remove_dir_all(&out);
    println!("{preset} shown: {shown:?}");
    (shown, false_remnants, unlit)
}

/// Pass: no failed star, the early labels forward only, none of them after a later stage showed, and no white dwarf while
/// the centre holds hydrogen or before h_shell (§0.5 [M0-D140]), and an ignited star past those labels within
/// [`LIGHT_FRAMES`] frames (§0.5 [M0-D141]).
fn grade(preset: &str, (shown, false_remnants, unlit): &Life) {
    assert!(!shown.is_empty(), "{preset}: no stage shown");
    let first = unlit.first();
    assert!(unlit.is_empty(), "{preset}: {} frames ignited still show an early label, the first (step, frames lit, shown) {first:?}: {shown:?}", unlit.len());
    assert!(false_remnants.is_empty(), "{preset} showed «White dwarf» at (step, X_H,c, after h_shell) {false_remnants:?}: {shown:?}");
    assert!(shown.iter().all(|(s, _)| s != "failed_star"), "{preset} showed «Failed star»: {shown:?}");
    let mut rank = 0;
    let mut later = false;
    for (stage, step) in shown {
        match early_rank(stage) {
            Some(r) => {
                assert!(!later && r >= rank, "{preset}: {stage} at step {step} steps back: {shown:?}");
                rank = r;
            }
            None => later = true,
        }
    }
}

#[test]
fn the_sun_like_preset_shows_its_stages_in_order() {
    grade("sun", &life("sun", 2000));
}

#[test]
fn the_15_sun_preset_shows_its_stages_in_order() {
    grade("massive", &life("massive", 500));
}
