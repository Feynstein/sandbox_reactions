//! M0-T103 — the visible life (contract §5.2's G-STAGES as §0.5 [M0-T103] reads it): each preset's whole life, run headless on
//! the physics adapter at calibration's masses through the run's progress hook in frames of 16 steps (an `--until` that meets
//! only at the row's last stage, so headless's FRAME and not its 256-step chunk — M0-V8's flag: at the coarse chunk the early
//! stages never show), the heaviest object's DISPLAYED stage read every frame — the stage the app's readout shows, its `Stars`
//! running the same Tracker and StageTracker (stage_order.rs). The displayed ids must contain the preset's row as a
//! subsequence, in order; parenthesised stages may be missing and other stages may appear between.
//!
//! Three things keep a green from being a flicker's: the case reads `shown`, never `raw` (§1.9.3: a stage other than
//! core_collapse and supernova is displayed after 3 consecutive identical results), and it checks that very property on the
//! life it read — each stage first displayed (but those two) had a raw result equal to it on the 3 frames up to and
//! including that one; the matcher is itself graded on hand-made lists before any life runs (a missing stage, a wrong
//! order, a parenthesised stage left out, the unordered slot swapped); and the row's last stage is what ends the run.
//! The Massive's h_shell and he_core are the "Supergiant" labels (§1.9.3): the first time each is displayed its object's
//! mass is at or over m_up_sb, the app's own test (crates/sr-app/src/ui/right.rs `Stars`).
//!
//! A life is long (the Sun-like's about 197,000 steps, over ten minutes: R12's ⏱ scope, the lead's run).

use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick};
use sr_engine::scene::Scene;
use sr_physics::calibration;

/// A stage of a row: its id and whether it must show (the contract's parenthesised stages need not).
type Member = (&'static str, bool);

/// A row's slot: one stage, or the stages that may show in either order (§5.2's massive «c_burn, ne_burn and o_burn in either
/// order»); the next slot's stages come after all of the required ones of this.
type Slot = &'static [Member];

const fn req(id: &'static str) -> Member {
    (id, true)
}

const fn opt(id: &'static str) -> Member {
    (id, false)
}

/// §5.2 G-STAGES' sun row (§0.5 [M0-T103]).
const SUN: &[Slot] = &[
    &[opt("cloud")],
    &[opt("collapsing")],
    &[opt("protostar")],
    &[req("main_sequence")],
    &[opt("h_shell")],
    &[req("he_core")],
    &[req("he_shell")],
    &[opt("planetary_nebula")],
    &[req("white_dwarf")],
];

/// The massive row (§0.5 [M0-D169], [M0-T103]).
const MASSIVE: &[Slot] = &[
    &[opt("cloud")],
    &[opt("collapsing")],
    &[req("protostar")],
    &[req("main_sequence")],
    &[opt("h_shell")],
    &[req("he_core")],
    &[opt("c_burn"), opt("ne_burn"), req("o_burn")],
    &[req("si_burn")],
    &[req("iron_core")],
    &[req("core_collapse")],
    &[req("supernova")],
    &[req("neutron_star")],
];

/// The giant row (§0.5 [M0-T103]).
const GIANT: &[Slot] = &[
    &[opt("cloud")],
    &[opt("collapsing")],
    &[req("protostar")],
    &[req("main_sequence")],
    &[req("he_core")],
    &[req("core_collapse")],
    &[opt("supernova")],
    &[req("black_hole")],
];

/// Whether `shown` — a life's displayed stages, one entry per change — holds `row` as a subsequence in order: each required
/// stage of a slot is looked for from where the last slot's required stages ended, and the slot ends at the latest of them;
/// a parenthesised stage never moves that point, so a stray one cannot fail a later required stage. Err names the first
/// required stage that is missing or out of order.
fn holds(row: &[Slot], shown: &[&str]) -> Result<(), String> {
    let mut from = 0;
    for slot in row {
        let mut end = from;
        for &(id, required) in *slot {
            match shown[from..].iter().position(|s| *s == id) {
                Some(at) if required => end = end.max(from + at + 1),
                None if required => return Err(format!("{id} is missing, or comes before the stages ahead of it (searched from displayed #{from})")),
                _ => {}
            }
        }
        from = end;
    }
    Ok(())
}

/// The matcher on hand-made lists, before any life: every row accepts its own stages in order, refuses one without a required
/// stage and one with two of them swapped, accepts one without its parenthesised stages, and takes the unordered slot either
/// way round.
fn matcher_is_graded() {
    for (name, row) in [("sun", SUN), ("massive", MASSIVE), ("giant", GIANT)] {
        let all: Vec<&str> = row.iter().flat_map(|slot| slot.iter().map(|m| m.0)).collect();
        assert!(holds(row, &all).is_ok(), "{name}: its own stages in order refused");
        let required: Vec<&str> = row.iter().flat_map(|slot| slot.iter().filter(|m| m.1).map(|m| m.0)).collect();
        assert!(holds(row, &required).is_ok(), "{name}: without its parenthesised stages refused");
        for gone in &required {
            let without: Vec<&str> = all.iter().copied().filter(|s| s != gone).collect();
            let err = holds(row, &without).expect_err(&format!("{name}: accepted with {gone} missing"));
            assert!(err.starts_with(gone), "{name}: refused for {err}, not for {gone}");
        }
        let mut swapped = required.clone();
        let n = swapped.len();
        swapped.swap(n - 2, n - 1);
        assert!(holds(row, &swapped).is_err(), "{name}: accepted with its last two required stages swapped");
    }
    let unordered = ["protostar", "main_sequence", "he_core", "o_burn", "ne_burn", "si_burn", "iron_core", "core_collapse", "supernova", "neutron_star"];
    assert!(holds(MASSIVE, &unordered).is_ok(), "the massive row refused ne_burn after o_burn");
    let reversed = ["protostar", "main_sequence", "he_core", "ne_burn", "o_burn", "si_burn", "iron_core", "core_collapse", "supernova", "neutron_star"];
    assert!(holds(MASSIVE, &reversed).is_ok(), "the massive row refused ne_burn before o_burn: it takes either order");
    // A required stage shown only before its predecessors does not count: white_dwarf first, the sun's row not held.
    let early = ["white_dwarf", "main_sequence", "he_core", "he_shell"];
    assert!(holds(SUN, &early).is_err(), "the sun row accepted white_dwarf shown before the rest");
}

/// What a life showed: the displayed stages with the step each first showed at (a change only), the frames whose displayed
/// stage did not have 3 identical raw results behind it, the Supergiant labels shown under m_up_sb, and the run's exit code.
struct Life {
    shown: Vec<(&'static str, u64)>,
    unheld: Vec<(u64, &'static str)>,
    light: Vec<(u64, &'static str, f64)>,
    code: i32,
    steps: u64,
}

/// `preset` at calibration's masses on the physics adapter until it displays `last` (`--max-steps` ends it, exit 2).
fn life(preset: &str, last: &str, max_steps: u64, supergiant_row: bool) -> Life {
    let cal = calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration");
    let scene = Scene::preset(preset, Default::default()).unwrap().with_preset_masses(&cal.preset_mass_sb);
    let out = std::env::temp_dir().join(format!("sr-stages-{}-{preset}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let max = max_steps.to_string();
    let until = format!("stage:{last}");
    let mut args: Vec<String> = ["--scene", &format!("life:{preset}"), "--until", &until, "--max-steps", &max, "--out", &out.display().to_string()]
        .map(String::from)
        .to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let mut shown: Vec<(&'static str, u64)> = Vec::new();
    let mut unheld: Vec<(u64, &'static str)> = Vec::new();
    let mut light: Vec<(u64, &'static str, f64)> = Vec::new();
    // The raw results of the last frames the object had, oldest first.
    let mut raws: Vec<Option<&'static str>> = Vec::new();
    let mut frame = 0u64;
    let mut last_step = 0u64;
    let mut tick = |t: &Tick| {
        last_step = t.step;
        let Some(s) = &t.star else { return };
        raws.push(s.raw);
        frame += 1;
        if let Some(stage) = s.shown.filter(|st| shown.last().is_none_or(|(prev, _)| prev != st)) {
            println!("{preset} step={} t={:.2} shown={stage} raw={} M={:.1} (m_up_sb {:.1})", t.step, t.sim_time, s.raw.unwrap_or("none"), s.mass, cal.m_up_sb);
            // §1.9.3's hold: all but core_collapse and supernova need 3 identical results — this frame's and the two before.
            let immediate = stage == "core_collapse" || stage == "supernova";
            let n = raws.len();
            if !immediate && n >= 3 && !raws[n - 3..].iter().all(|r| *r == Some(stage)) {
                unheld.push((t.step, stage));
            }
            // The Supergiant labels are the object's own mass at or over m_up_sb (the readout's test), read at the stage's first
            // display — the one that meets the row; a later flicker of he_core, after the star has lost mass, is printed, not graded
            // (M0-T103: the Massive's two at t 164.6 and 166.5, M 5,497 and 5,342 under m_up_sb 6,515).
            if supergiant_row && (stage == "h_shell" || stage == "he_core") && !shown.iter().any(|(p, _)| *p == stage) && s.mass < cal.m_up_sb {
                light.push((t.step, stage, s.mass));
            }
            shown.push((stage, t.step));
        } else if frame.is_multiple_of(2000) {
            println!("{preset} step={} t={:.2} shown={} raw={}", t.step, t.sim_time, s.shown.unwrap_or("none"), s.raw.unwrap_or("none"));
        }
    };
    let mut saved = |_: u64, _: f64, _: &str| {};
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: None, saved: &mut saved };
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress));
    assert!(code == 0 || code == 2, "{preset}: the run ended with exit {code}: {lines:?}");
    let _ = std::fs::remove_dir_all(&out);
    println!("{preset} shown ({last_step} steps, exit {code}): {shown:?}");
    Life { shown, unheld, light, code, steps: last_step }
}

/// Pass: the displayed ids hold `row`, every displayed stage had its 3 identical results (§1.9.3), and the run met its last
/// stage; the Massive's h_shell and he_core only at or over m_up_sb.
fn grade(preset: &str, row: &[Slot], l: &Life) {
    let ids: Vec<&str> = l.shown.iter().map(|(s, _)| *s).collect();
    let steps = l.steps;
    assert!(l.unheld.is_empty(), "{preset}: displayed (step, stage) without 3 identical raw results behind it: {:?}; shown {:?}", l.unheld, l.shown);
    assert!(l.light.is_empty(), "{preset}: h_shell / he_core displayed below m_up_sb — the label would read Red giant, not Supergiant: (step, stage, mass) {:?}", l.light);
    if let Err(e) = holds(row, &ids) {
        panic!("{preset}: {e}; the run ended at step {steps} (exit {}); displayed stages with the step each first showed at: {:?}", l.code, l.shown);
    }
    assert_eq!(l.code, 0, "{preset}: the row holds yet the run did not meet its last stage: {:?}", l.shown);
}

#[test]
fn the_sun_like_preset_shows_its_stages_down_to_white_dwarf() {
    matcher_is_graded();
    grade("sun", SUN, &life("sun", "white_dwarf", 320_000, false));
}

#[test]
fn the_massive_preset_shows_its_stages_down_to_neutron_star() {
    matcher_is_graded();
    grade("massive", MASSIVE, &life("massive", "neutron_star", 80_000, true));
}

#[test]
fn the_giant_preset_shows_its_stages_down_to_black_hole() {
    matcher_is_graded();
    grade("giant", GIANT, &life("giant", "black_hole", 60_000, false));
}
