//! M0-T60 — the observers in headless runs (contract §1.9, §2.12.2, §3.3; crates/sr-engine/src/watch.rs and headless.rs): the
//! `--until` conditions, `--max-steps`, DIR/events.jsonl and summary.json's `events`, `objects` and `until`, on the engine's
//! own block summary (the tracker and the stages have run on synthetic summaries alone until here — scopes `objects` and
//! `classify`). Every run goes through `headless::run_captured` on the physics adapter (R12), its stdout lines kept so the
//! `SR-HEADLESS DONE` line is read as printed.
//!
//! The overshoot each condition allows (the adversarial line): `sim_time:<t>` stops at the first step past `t` — the run
//! one step shorter is still under `t`; `stage:` and `event:` and `ending` are read on a frame's summary, a frame being
//! 16 steps (headless's FRAME) after the run's first step, so the stop falls on a frame boundary, at most one frame past the
//! step the condition first held, and the run one frame shorter is `unmet`. Both are asserted below.

use std::path::{Path, PathBuf};

use serde_json::Value;
use sr_engine::dump::{Dump, Sink};
use sr_engine::headless::run_captured;

/// A frame while `--until` watches (headless's FRAME), in steps: the first step stands alone.
const FRAME: u64 = 16;

fn scenes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-observe-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A finished run: the exit code, stdout, summary.json and events.jsonl, and the folder it wrote them in.
struct Out {
    dir: PathBuf,
    code: i32,
    lines: Vec<String>,
    summary: Value,
    events: String,
}

impl Out {
    fn done(&self) -> &str {
        self.lines.last().expect("headless printed a line")
    }

    fn steps(&self) -> u64 {
        self.summary["steps"].as_u64().expect("steps")
    }

    fn sim_time(&self) -> f64 {
        self.summary["sim_time"].as_f64().expect("sim_time")
    }
}

/// `headless --scene <scene> <extra…> --out <dir>` on the physics adapter.
fn go(name: &str, scene: &Path, extra: &[&str]) -> Out {
    let out = temp(name).join("out");
    let mut args: Vec<String> = ["--scene", &scene.display().to_string(), "--out", &out.display().to_string()].map(String::from).to_vec();
    args.extend(extra.iter().map(|s| s.to_string()));
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let (code, lines) = run_captured(&args);
    let summary = serde_json::from_slice(&std::fs::read(out.join("summary.json")).expect("summary.json")).expect("summary.json parses");
    let events = std::fs::read_to_string(out.join("events.jsonl")).expect("events.jsonl");
    Out { dir: out, code, lines, summary, events }
}

fn cold_disk() -> PathBuf {
    scenes_dir().join("cold_disk.json")
}

/// `--until stage:collapsing` on the cold disk: met, the stop on a frame boundary, the object showing the stage from the very
/// step the run stopped at, and the run one frame shorter unmet — the condition is read at the first frame it holds.
#[test]
fn until_a_stage_stops_at_the_frame_that_shows_it() {
    let out = go("stage", &cold_disk(), &["--until", "stage:collapsing", "--max-steps", "2000"]);
    assert_eq!(out.code, 0, "{:?}", out.lines);
    let steps = out.steps();
    assert!(out.done().starts_with(&format!("SR-HEADLESS DONE steps={steps} sim_time=")), "{}", out.done());
    assert!(out.done().ends_with("until=met"), "{}", out.done());
    assert_eq!(out.summary["until"]["condition"], "stage:collapsing");
    assert_eq!(out.summary["until"]["met"], true);
    assert_eq!((steps - 1) % FRAME, 0, "the stop is a frame boundary (first step, then {FRAME} a frame): {steps}");
    let objects = out.summary["objects"].as_array().expect("objects");
    assert_eq!(objects.len(), 1, "one cloud: {objects:?}");
    let o = &objects[0];
    assert_eq!(o["stage"], "collapsing", "{o}");
    let stages = o["stages"].as_array().expect("stages");
    let last = stages.last().expect("the object has displayed a stage");
    assert_eq!(last["stage"], "collapsing");
    assert_eq!(last["from_step"].as_u64(), Some(steps), "shown from the step the run stopped at: {stages:?}");
    assert!((last["sim_time"].as_f64().unwrap() - out.sim_time()).abs() <= 1e-12 * out.sim_time(), "{last} against {}", out.sim_time());
    assert!(o["mass_sb"].as_f64().unwrap() > 3000.0, "the disk's mass (a = 40, Σc = 1: 3,351): {o}");
    for key in ["id", "stage", "mass_sb", "stages"] {
        assert!(o.get(key).is_some(), "objects[] has {key}: {o}");
    }

    // One frame shorter, the stage has not yet held (the 3rd matching frame is this one): unmet, exit 2.
    let shorter = go("stage-short", &cold_disk(), &["--until", "stage:collapsing", "--max-steps", &(steps - FRAME).to_string()]);
    assert_eq!(shorter.code, 2, "{:?}", shorter.lines);
    assert!(shorter.done().ends_with("until=unmet"), "{}", shorter.done());
}

/// `--until sim_time:5` stops at the first step past 5: the same run cut one step earlier is still under 5.
#[test]
fn until_sim_time_stops_at_the_first_step_past_it() {
    let out = go("sim", &cold_disk(), &["--until", "sim_time:5", "--max-steps", "200000"]);
    assert_eq!(out.code, 0, "{:?}", out.lines);
    assert!(out.done().ends_with("until=met"), "{}", out.done());
    assert_eq!(out.summary["until"]["condition"], "sim_time:5");
    assert_eq!(out.summary["until"]["met"], true);
    let (steps, sim) = (out.steps(), out.sim_time());
    assert!(sim >= 5.0, "met at sim time {sim}");
    let before = go("sim-before", &cold_disk(), &["--steps", &(steps - 1).to_string()]);
    assert!(before.sim_time() < 5.0, "step {} is already at {} ≥ 5: the stop at {steps} is late", steps - 1, before.sim_time());
    println!("sim_time:5 met at step {steps}, sim time {sim:.9}; the step before: {:.9}", before.sim_time());
}

/// A condition that cannot hold within `--max-steps`: exit 2, the DONE line reading `until=unmet`, the summary's `until` met
/// false at the cap, and events.jsonl there all the same.
#[test]
fn an_unmet_condition_is_exit_2_with_until_unmet() {
    let out = go("unmet", &cold_disk(), &["--until", "stage:black_hole", "--max-steps", "100"]);
    assert_eq!(out.code, 2, "{:?}", out.lines);
    assert_eq!(out.steps(), 100);
    assert!(out.done().starts_with("SR-HEADLESS DONE steps=100 sim_time="), "{}", out.done());
    assert!(out.done().ends_with("until=unmet"), "{}", out.done());
    assert_eq!(out.summary["until"]["condition"], "stage:black_hole");
    assert_eq!(out.summary["until"]["met"], false);
    assert_eq!(out.events.lines().count(), out.summary["events"].as_array().unwrap().len(), "events.jsonl matches summary.json");

    // The same cap on `--steps` is the old exit 2 with `until=unmet` too, and a run that reaches its steps says `none`.
    let cut = go("steps-cut", &cold_disk(), &["--steps", "50", "--max-steps", "20"]);
    assert_eq!((cut.code, cut.steps()), (2, 20), "{:?}", cut.lines);
    assert!(cut.done().ends_with("until=unmet"), "{}", cut.done());
    let plain = go("steps-plain", &cold_disk(), &["--steps", "20"]);
    assert_eq!(plain.code, 0);
    assert!(plain.done().ends_with("until=none"), "{}", plain.done());
    assert_eq!(plain.summary["until"]["condition"], "steps:20");
}

/// events.jsonl on an engine run with events: scenes/sink_cloud.json (a 40 × 40 square of gas, whose steps are ~0.0165
/// t.u. apiece — the cold disk's collapse shrinks Δt until 20,000 steps pass only 6) at [`ENDING_T`], with a sink dropped
/// into its middle (the dump written by the test)
/// — `sink_formed` at the first frame, the object a black hole, and `--until ending` stopping when that remnant has been
/// displayed for its hold, max(10 t.u., 20 τ_dyn) (§0.5 [M0-D78]: the detail's `hold_t`, `held_t` at or past it). Every line
/// of the file is one JSON object with the §2.12.2 keys, the same as summary.json's `events`; the run one frame shorter has no
/// `ending` yet.
/// The ending case's cloud T, the scene's 0.05 raised (M0-D66, the lead's «Case's own T 0.5 + D», 2026-10-20; the scene file,
/// the `sink` scope's, unchanged): at T 0.05 the cold sheet is Jeans-unresolved, and once §0.5 [M0-D64]'s face work stopped
/// the Σu·g source heating it, it fragments into 7 sinks that eat the gas, leaving no object to show `black_hole`; at 0.5 one
/// sink forms and the ending comes at t 11.21 (RTX 5090). A sink that has eaten all its gas is no object (§1.9.2): M0-D66's D.
const ENDING_T: f64 = 0.5;

#[test]
fn events_jsonl_and_the_ending_of_a_black_hole() {
    let dir = temp("ending");
    let mut cloud: Value = serde_json::from_slice(&std::fs::read(scenes_dir().join("sink_cloud.json")).unwrap()).unwrap();
    cloud["objects"][0]["t"] = ENDING_T.into();
    let scene = dir.join("sink_cloud.json");
    std::fs::write(&scene, cloud.to_string()).unwrap();

    let seed = go("ending-seed", &scene, &["--steps", "40"]);
    assert_eq!(seed.code, 0, "{:?}", seed.lines);
    let mut dump = Dump::read(&seed.dir.join("dumps").join("s_40.bin")).expect("the seed run's dump");
    let c = (dump.world.width / 2, dump.world.height / 2);
    dump.sinks = vec![Sink { x: f64::from(c.0) + 0.5, y: f64::from(c.1) + 0.5, vx: 0.0, vy: 0.0, m: 50.0 }];
    let seeded = dir.join("seeded.bin");
    dump.write(&seeded).unwrap();
    let from = seeded.display().to_string();

    let out = go("ending-run", &scene, &["--load-dump", &from, "--until", "ending", "--max-steps", "20000"]);
    assert_eq!(out.code, 0, "{:?}", out.lines);
    assert!(out.done().ends_with("until=met"), "{}", out.done());
    let events = out.summary["events"].as_array().expect("events");
    let lines: Vec<Value> = out.events.lines().map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("{l:?}: {e}"))).collect();
    assert_eq!(&lines, events, "events.jsonl is summary.json's events, one JSON object a line");
    for e in &lines {
        let mut keys: Vec<&str> = e.as_object().expect("an object").keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["detail", "kind", "object", "sim_time", "step"], "{e}");
    }
    let kinds: Vec<&str> = lines.iter().map(|e| e["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["sink_formed", "ending"], "a sink inside the cloud: forms once, then the remnant ends: {lines:?}");
    let ending = &lines[1];
    assert_eq!(ending["detail"]["stage"], "black_hole", "{ending}");
    // A step index counts from the scene, as the dumps' do: this run's `steps` on top of the seed dump's 40.
    assert_eq!(ending["step"].as_u64(), Some(40 + out.steps()), "the run stops at the frame that saw the ending");

    let o = &out.summary["objects"][0];
    assert_eq!(o["stage"], "black_hole", "{o}");
    let shown = o["stages"].as_array().unwrap().iter().find(|s| s["stage"] == "black_hole").expect("black_hole was displayed");
    let held = ending["sim_time"].as_f64().unwrap() - shown["sim_time"].as_f64().unwrap();
    let hold = ending["detail"]["hold_t"].as_f64().expect("the ending names its hold");
    assert!(hold >= 10.0, "the hold is max(10 t.u., 20 τ_dyn): {ending}");
    assert_eq!(ending["detail"]["held_t"].as_f64(), Some(held), "{ending}");
    assert!(held >= hold, "an ending after {held} t.u. of a remnant on display, not its hold {hold}");
    assert_eq!(shown["from_step"].as_u64().map(|s| s > 40), Some(true), "from_step counts from the scene too: {shown}");
    println!("ending at step {}: black_hole shown from step {}, held {held:.3} t.u. of {hold:.3}", ending["step"], shown["from_step"]);

    let shorter = go("ending-short", &scene, &["--load-dump", &from, "--until", "ending", "--max-steps", &(out.steps() - FRAME).to_string()]);
    assert_eq!(shorter.code, 2, "one frame shorter the remnant has not held its hold: {:?}", shorter.lines);
    assert!(shorter.summary["events"].as_array().unwrap().iter().all(|e| e["kind"] != "ending"));
}

/// The arguments: one of `--steps` and `--until`, a condition of the four forms, the twin never asked to observe — each
/// refusal exit 4 before any device is touched.
#[test]
fn until_is_checked_before_a_device_is_touched() {
    let scene = cold_disk().display().to_string();
    let cases: [(&[&str], &str); 7] = [
        (&["--steps", "5", "--until", "ending"], "one of the two"),
        (&[], "one of the two"),
        (&["--until", "later"], "expected sim_time"),
        (&["--until", "stage:nova"], "a stage is one of"),
        (&["--until", "event:ending"], "an event is one of"),
        (&["--until", "sim_time:-3"], "sim_time takes a time"),
        (&["--until", "ending", "--cpu-reference"], "--until is the GPU's"),
    ];
    for (extra, message) in cases {
        let mut args = vec!["--scene".to_string(), scene.clone()];
        args.extend(extra.iter().map(|s| s.to_string()));
        let (code, lines) = run_captured(&args);
        assert_eq!(code, 4, "{extra:?}: {lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("SR-ERROR bad arguments:") && l.contains(message)), "{extra:?}: {lines:?}");
    }
}

/// M0-D76 — a life cut at a dump and continued (`--load-dump FILE --continue`, contract §0.5 [M0-D76]): the Sun-like preset
/// built in code as `calibrate` builds its lives (`run_scene_captured`: a scene that reads no calibration, so it runs on a
/// partial calibration file too, O13), `--until` [`CONTINUE_UNTIL`] with a dump every [`CONTINUE_EVERY`] steps, run unbroken
/// and cut at the dump [`CONTINUE_CUT`] — after the ignition and before a stage change, so a lost `ignited` or ledger start
/// shows past it. The continued run writes the unbroken run's summary.json line for line but `wall_s`, its events.jsonl and
/// its final dump byte for byte; a snapshot of another step beside the dump, none beside it, and `--continue` without
/// `--load-dump` are exit 4.
const CONTINUE_UNTIL: &str = "stage:main_sequence";
const CONTINUE_EVERY: u64 = 256;
const CONTINUE_CUT: u64 = 768;

/// `--scene <label> <extra…> --out <out>` on the Sun-like preset built in code: the exit code and the lines it said.
fn sun_life(out: &Path, extra: &[&str]) -> (i32, Vec<String>) {
    use sr_engine::headless::{parse_args, run_scene_captured};
    let mut args: Vec<String> = ["--scene", "life:sun", "--out", &out.display().to_string()].map(String::from).to_vec();
    args.extend(extra.iter().map(|s| s.to_string()));
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    run_scene_captured(parse_args(&args).unwrap(), sr_engine::scene::Scene::preset("sun", Default::default()).unwrap(), None)
}

#[test]
fn a_life_cut_at_a_dump_and_continued_writes_the_unbroken_lifes_summary() {
    let dir = temp("continue");
    let every = CONTINUE_EVERY.to_string();
    // The cap a little past the unbroken life's 1,184 steps: a plant that never meets the condition stops there.
    let life = ["--until", CONTINUE_UNTIL, "--dump-every", &every, "--max-steps", "1600"];
    let (whole, cut, on) = (dir.join("whole"), dir.join("cut"), dir.join("on"));
    let (code, lines) = sun_life(&whole, &life);
    assert_eq!(code, 0, "{lines:?}");
    let text = |d: &Path, f: &str| std::fs::read_to_string(d.join(f)).unwrap_or_else(|e| panic!("{}: {e}", d.join(f).display()));
    let summary: Value = serde_json::from_str(&text(&whole, "summary.json")).unwrap();
    let steps = summary["steps"].as_u64().unwrap();
    println!("unbroken: {steps} steps, events {}", summary["events"]);
    println!("stages {}", summary["objects"][0]["stages"]);

    // The cut: the same life stopped at the dump CONTINUE_CUT, then continued from it.
    let cap = CONTINUE_CUT.to_string();
    let mut cut_life = life.to_vec();
    cut_life[5] = &cap;
    assert_eq!(sun_life(&cut, &cut_life).0, 2, "cut by --max-steps");
    let from = cut.join("dumps").join(format!("s_{CONTINUE_CUT}.bin"));
    let from = from.display().to_string();
    let (code, lines) = sun_life(&on, &[&life[..], &["--load-dump", &from, "--continue"]].concat());
    assert_eq!(code, 0, "{lines:?}");

    let but_wall = |d: &Path| text(d, "summary.json").lines().filter(|l| !l.trim_start().starts_with("\"wall_s\"")).map(String::from).collect::<Vec<_>>();
    assert_eq!(but_wall(&on), but_wall(&whole), "summary.json, wall_s aside");
    assert_eq!(text(&on, "events.jsonl"), text(&whole, "events.jsonl"));
    let last = format!("dumps/s_{steps}.bin");
    assert!(std::fs::read(on.join(&last)).unwrap() == std::fs::read(whole.join(&last)).unwrap(), "the final dump, byte for byte");

    // Refused, exit 4: a snapshot of another step beside the dump, none beside it, --continue without --load-dump.
    let snapshot = |step: u64| cut.join("dumps").join(format!("s_{step}.observers.json"));
    std::fs::copy(snapshot(CONTINUE_CUT - CONTINUE_EVERY), snapshot(CONTINUE_CUT)).unwrap();
    let (code, lines) = sun_life(&dir.join("other"), &[&life[..], &["--load-dump", &from, "--continue"]].concat());
    assert_eq!(code, 4, "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-ERROR bad dump: --continue:") && l.contains(&format!("the dump's {CONTINUE_CUT}"))), "{lines:?}");
    std::fs::remove_file(snapshot(CONTINUE_CUT)).unwrap();
    assert_eq!(sun_life(&dir.join("none"), &[&life[..], &["--load-dump", &from, "--continue"]].concat()).0, 4);
    let (code, lines) = run_captured(&["--scene", "preset:sun", "--until", "ending", "--continue"].map(String::from));
    assert_eq!(code, 4, "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("--continue needs --load-dump")), "{lines:?}");
}
