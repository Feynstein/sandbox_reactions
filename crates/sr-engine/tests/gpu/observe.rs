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

use serde_json::{json, Value};
use sr_engine::dump::{Dump, Sink};
use sr_engine::headless::{observers_path, run_captured};
use sr_engine::ledger::At;
use sr_engine::summary::{Block, BlockSummary};
use sr_engine::watch::{embedded_calibration, Star, Watch};
use sr_physics::observe::age::AgeClock;
use sr_physics::observe::ending::{Prediction, Predictor, Readout};
use sr_physics::observe::tracker::SUN_MASS_NOMINAL;
use sr_physics::observe::translate::MassScale;
use sr_physics::observe::{Object, Stage};
use sr_physics::registry::Physics;

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
    // ≤ 1e-12 relative, not bit-equal (M0-D100): serde_json's default float parse is not correctly rounded — on the RTX 5090 it
    // read events.jsonl's sim_time 11.638221981935203 as …205, one ulp off, while held_t is exactly the file's two times' difference.
    let held_t = ending["detail"]["held_t"].as_f64().expect("the ending names how long it held");
    assert!((held_t - held).abs() <= 1e-12 * held, "held_t {held_t} against {held}: {ending}");
    assert!(held >= hold, "an ending after {held} t.u. of a remnant on display, not its hold {hold}");
    assert_eq!(shown["from_step"].as_u64().map(|s| s > 40), Some(true), "from_step counts from the scene too: {shown}");
    println!("ending at step {}: black_hole shown from step {}, held {held:.3} t.u. of {hold:.3}", ending["step"], shown["from_step"]);

    let shorter = go("ending-short", &scene, &["--load-dump", &from, "--until", "ending", "--max-steps", &(out.steps() - FRAME).to_string()]);
    assert_eq!(shorter.code, 2, "one frame shorter the remnant has not held its hold: {:?}", shorter.lines);
    assert!(shorter.summary["events"].as_array().unwrap().iter().all(|e| e["kind"] != "ending"));
}

/// A black hole that eats all the gas in the world still ends (M0-D96; M0-D66's T 3 fix, M0-D70's case): sink_cloud at T 3,
/// a [`EATEN_SINK`] sink dropped at its centre after 40 steps, eats the whole cloud — on the RTX 4090 (M0-D96, a 50 sink) the
/// box emptied at step 1,521 (t 11.0), where Δt was the largest f32: the clock froze and P3 drifted the sink to infinity, so
/// `--until ending` never met. With an empty box stepping on the floor gas (§0.5 [M0-D96]) the clock runs on, the sink stays a
/// finite object of its own (§0.5 [M0-D70]) and its black_hole holds to the ending; the final dump's sink is finite and in the
/// world. The box must empty before the ending, or the floor's Δt is never read (M0-D102: a 50 sink left thin wisps in 611
/// cells at its ending, t 11.77, the box the whole world) — so the final dump's box is empty and its Δt the floor gas's.
#[test]
fn a_black_hole_that_eats_its_whole_cloud_meets_its_ending() {
    let dir = temp("eaten");
    let mut cloud: Value = serde_json::from_slice(&std::fs::read(scenes_dir().join("sink_cloud.json")).unwrap()).unwrap();
    cloud["objects"][0]["t"] = 3.0.into();
    let scene = dir.join("sink_cloud.json");
    std::fs::write(&scene, cloud.to_string()).unwrap();

    let seed = go("eaten-seed", &scene, &["--steps", "40"]);
    assert_eq!(seed.code, 0, "{:?}", seed.lines);
    let mut dump = Dump::read(&seed.dir.join("dumps").join("s_40.bin")).expect("the seed run's dump");
    let (w, h) = (dump.world.width, dump.world.height);
    dump.sinks = vec![Sink { x: f64::from(w / 2) + 0.5, y: f64::from(h / 2) + 0.5, vx: 0.0, vy: 0.0, m: EATEN_SINK }];
    let seeded = dir.join("seeded.bin");
    dump.write(&seeded).unwrap();

    let out = go("eaten-run", &scene, &["--load-dump", &seeded.display().to_string(), "--until", "ending", "--max-steps", "4000"]);
    assert_eq!(out.code, 0, "{:?}", out.lines);
    assert!(out.done().ends_with("until=met"), "{}", out.done());
    let lines: Vec<Value> = out.events.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let ending = lines.last().expect("events");
    assert_eq!((ending["kind"].as_str(), ending["detail"]["stage"].as_str()), (Some("ending"), Some("black_hole")), "{lines:?}");
    let o = out.summary["objects"].as_array().unwrap().iter().find(|o| o["id"] == ending["object"]).expect("the ending's object");
    assert_eq!(o["stage"], "black_hole", "{o}");
    let last = Dump::read(&out.dir.join("dumps").join(format!("s_{}.bin", 40 + out.steps()))).expect("the final dump");
    assert_eq!(last.sinks.len(), 1, "{:?}", last.sinks);
    let s = &last.sinks[0];
    assert!(s.x.is_finite() && s.y.is_finite(), "the sink is somewhere: {s:?}");
    assert!((0.0..f64::from(w)).contains(&s.x) && (0.0..f64::from(h)).contains(&s.y), "the sink is in the world: {s:?}");
    // The box emptied before the ending and the clock ran on the floor gas of hydrogen (1/μ = 2): Δt = C / (2c), c² = 2 T_floor/μ.
    let [_, _, bw, bh, ..] = last.resume.box_record;
    assert_eq!(bw * bh, 0, "the cloud never left an empty box before its ending: box {:?}", last.resume.box_record);
    let physics = Physics::shipped().expect("shipped physics.json");
    let key = |k: &str| physics.get(k).unwrap() as f32 as f64;
    let floor = key("cfl") / (2.0 * (4.0 * key("t_floor")).sqrt());
    let dt = f64::from(last.resume.dt);
    assert!((dt - floor).abs() <= 1e-6 * floor, "the empty box's Δt {dt} against the floor gas's {floor}");
    println!(
        "T 3: ending at step {} (t {:.3}), object {} of {} sb, the sink at ({:.2}, {:.2}), {} events; the box empty, Δt {dt:.6}",
        ending["step"], out.sim_time(), o["id"], o["mass_sb"], s.x, s.y, lines.len()
    );
}

/// The eaten cloud's sink (M0-D102): 200 eats the T 3 cloud's last wisps by t ≈ 5.8 (the box empty at step 800's re-fit on
/// the RTX 4090), the ending at step 816 (t 12.09) — 6 t.u. on the floor's Δt; a 50 sink's wisps outlived its ending.
const EATEN_SINK: f64 = 200.0;

/// A sink with no gas around it is an object of its own (§0.5 [M0-D70]): the [`ENDING_T`] cloud in a 256 × 256 world, a 50
/// sink dropped into the vacuum at (16.5, 16.5), outside the box (the box follows the gas, §1.3.3, about the cloud at the
/// centre), so no block of the summary holds it — `sink_formed` at the first frame on an object of 50 exactly (a block would
/// add its gas, the floor's 1.6 × 10⁻⁷ at least) and `--until stage:black_hole` met on that object, still 50, the same id.
/// Later the cloud's thin gas reaches the sink and §1.9.2's merge and split rules take over (M0-D70's probe, RTX 4090).
#[test]
fn a_sink_outside_the_box_is_an_object_of_its_own() {
    let dir = temp("lone");
    let mut cloud: Value = serde_json::from_slice(&std::fs::read(scenes_dir().join("sink_cloud.json")).unwrap()).unwrap();
    cloud["objects"][0]["t"] = ENDING_T.into();
    let scene = dir.join("sink_cloud.json");
    std::fs::write(&scene, cloud.to_string()).unwrap();
    let world = ["--world", "256x256"];

    let seed = go("lone-seed", &scene, &[&world[..], &["--steps", "40"]].concat());
    assert_eq!(seed.code, 0, "{:?}", seed.lines);
    let mut dump = Dump::read(&seed.dir.join("dumps").join("s_40.bin")).expect("the seed run's dump");
    dump.sinks = vec![Sink { x: 16.5, y: 16.5, vx: 0.0, vy: 0.0, m: 50.0 }];
    let seeded = dir.join("seeded.bin");
    dump.write(&seeded).unwrap();
    let from = seeded.display().to_string();

    let out = go("lone-run", &scene, &[&world[..], &["--load-dump", &from, "--until", "stage:black_hole", "--max-steps", "2000"]].concat());
    assert_eq!(out.code, 0, "{:?}", out.lines);
    assert!(out.done().ends_with("until=met"), "{}", out.done());
    let lines: Vec<Value> = out.events.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    let formed = &lines[0];
    assert_eq!((formed["kind"].as_str(), formed["step"].as_u64()), (Some("sink_formed"), Some(41)), "the first frame: {formed}");
    assert_eq!(formed["detail"]["mass_sb"].as_f64(), Some(50.0), "the sink alone, no block's gas: {formed}");
    let objects = out.summary["objects"].as_array().unwrap();
    let o = objects.iter().find(|o| o["id"] == formed["object"]).expect("the sink's object, the same id");
    assert_eq!((o["stage"].as_str(), o["mass_sb"].as_f64()), (Some("black_hole"), Some(50.0)), "{o}");
    assert!(objects.iter().any(|c| c["id"] != o["id"] && c["mass_sb"].as_f64() > Some(1000.0)), "the cloud beside it: {objects:?}");
    println!("lone sink: object {} black_hole at step {} beside the cloud", o["id"], 40 + out.steps());
}

/// A life's ending is its heaviest object's (§0.5 [M0-D95]; the lead at M0-D95, «Heaviest object's (Recommended)»): two
/// [`ENDING_T`] clouds 324 cells apart in a 448 × 96 world — sink_cloud's 40 × 40 (1,600) and a 16 × 16 (256) — a 50 sink
/// dropped into the lighter after 40 steps and, the life cut at step 240 and continued, one into the heavier, so the lighter's
/// black hole holds its 10 t.u. first. Its `ending` is logged and the run goes on; the life ends on the heavier's, which
/// `until.object` names (a run with no life's ending names none). Closer, the heavier's spreading gas (R ≈ 100 by t 9) takes
/// the lighter's sink in before its hold (M0-D95's probe, 192 × 96). RTX 4090: the lighter's ending at step 2,496 (t 11.70),
/// the heavier's at 2,720 (t 12.94); the binary before M0-D95 stopped on the lighter's.
#[test]
fn a_lighter_objects_ending_never_ends_the_life() {
    let dir = temp("two");
    let mut scene: Value = serde_json::from_slice(&std::fs::read(scenes_dir().join("sink_cloud.json")).unwrap()).unwrap();
    scene["world"] = serde_json::json!({"width": 448, "height": 96, "edge": "leave"});
    let rect = |x0: u32, y0: u32, side: u32| {
        serde_json::json!({"kind": "rect", "x0": x0, "y0": y0, "w": side, "h": side, "sigma": 1.0, "t": ENDING_T, "composition": {"H": 1.0}, "v": [0, 0]})
    };
    scene["objects"] = serde_json::json!([rect(16, 28, 40), rect(380, 40, 16)]);
    let scene_file = dir.join("two_clouds.json");
    std::fs::write(&scene_file, scene.to_string()).unwrap();
    let add_sink = |from: &Path, to: &Path, x: f64, y: f64| {
        let mut dump = Dump::read(from).expect("a dump");
        dump.sinks.push(Sink { x, y, vx: 0.0, vy: 0.0, m: 50.0 });
        dump.write(to).unwrap();
    };

    let seed = go("two-seed", &scene_file, &["--steps", "40"]);
    assert_eq!(seed.code, 0, "{:?}", seed.lines);
    let lighter = dir.join("lighter.bin");
    add_sink(&seed.dir.join("dumps").join("s_40.bin"), &lighter, 388.5, 48.5);
    let cut = go("two-cut", &scene_file, &["--load-dump", &lighter.display().to_string(), "--until", "ending", "--max-steps", "200"]);
    assert_eq!(cut.code, 2, "{:?}", cut.lines);
    assert_eq!(cut.summary["until"], serde_json::json!({"condition": "ending", "met": false}), "no life's ending, no object named");
    let both = dir.join("both").join("s_240.bin");
    std::fs::create_dir_all(both.parent().unwrap()).unwrap();
    let cut_dump = cut.dir.join("dumps").join("s_240.bin");
    add_sink(&cut_dump, &both, 36.5, 48.5);
    std::fs::copy(observers_path(&cut_dump), observers_path(&both)).unwrap();

    let out = go("two-run", &scene_file, &["--load-dump", &both.display().to_string(), "--continue", "--until", "ending", "--max-steps", "20000"]);
    assert_eq!(out.code, 0, "{:?}", out.lines);
    assert!(out.done().ends_with("until=met"), "{}", out.done());
    let lines: Vec<Value> = out.events.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let endings: Vec<&Value> = lines.iter().filter(|e| e["kind"] == "ending").collect();
    assert_eq!(endings.len(), 2, "the lighter's ending, then the heavier's: {lines:?}");
    let (first, last) = (endings[0], endings[1]);
    let mass = |e: &Value| e["detail"]["mass_sb"].as_f64().expect("the ending's mass");
    assert!(mass(first) < mass(last) && first["step"].as_u64() < last["step"].as_u64(), "the lighter ends first: {endings:?}");
    assert_eq!((first["detail"]["stage"].as_str(), last["detail"]["stage"].as_str()), (Some("black_hole"), Some("black_hole")), "{endings:?}");
    assert_eq!(last["step"].as_u64(), Some(40 + out.steps()), "the run stops at the heavier's ending");
    assert_eq!(out.summary["until"], serde_json::json!({"condition": "ending", "met": true, "object": last["object"]}), "the life's ending names its object");
    let objects = out.summary["objects"].as_array().unwrap();
    let heaviest = objects.iter().max_by(|a, b| a["mass_sb"].as_f64().unwrap().total_cmp(&b["mass_sb"].as_f64().unwrap())).unwrap();
    assert_eq!(heaviest["id"], last["object"], "the life's ending is the heaviest object's: {objects:?}");
    println!(
        "two clouds: object {} ({:.1}) ended at step {} (t {:.2}), the life on object {} ({:.1}) at step {} (t {:.2})",
        first["object"], mass(first), first["step"], first["sim_time"].as_f64().unwrap(), last["object"], mass(last), last["step"],
        last["sim_time"].as_f64().unwrap()
    );
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
/// `--load-dump` are exit 4. The life ran to `stage:main_sequence` (step 1,616) until §0.5 [M0-D141] showed main_sequence
/// three frames after the ignition (672 → 704 on the RTX 4090), ahead of the cut; it now runs to sim time 20 (step 1,623),
/// main_sequence shown from 704 on.
const CONTINUE_UNTIL: &str = "sim_time:20";
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

/// The life's arguments, capped at `cap` steps. The cap a little past the unbroken life's 1,616 steps (1,623 to sim time 20,
/// M0-D141) on the V-cycle (M0-D84; 1,184 on Jacobi's looser solve, the lead at M0-D84: «Raise cap to 2,000 (Recommended)»):
/// a plant that never meets the condition stops there.
fn continue_args(cap: u64) -> Vec<String> {
    ["--until", CONTINUE_UNTIL, "--dump-every", &CONTINUE_EVERY.to_string(), "--max-steps", &cap.to_string()].map(String::from).to_vec()
}

/// The continued life's three runs — `whole/` unbroken, `cut/` stopped at the dump [`CONTINUE_CUT`], `on/` continued from it —
/// run once and read by the two cases that grade it (M0-D76's and M0-D103's), and the unbroken life's step count.
struct Continued {
    dir: PathBuf,
    steps: u64,
}

fn text(d: &Path, f: &str) -> String {
    std::fs::read_to_string(d.join(f)).unwrap_or_else(|e| panic!("{}: {e}", d.join(f).display()))
}

fn continued_life() -> &'static Continued {
    static LIFE: std::sync::OnceLock<Continued> = std::sync::OnceLock::new();
    LIFE.get_or_init(|| {
        let dir = temp("continue");
        let life = continue_args(2000);
        let life: Vec<&str> = life.iter().map(String::as_str).collect();
        let (code, lines) = sun_life(&dir.join("whole"), &life);
        assert_eq!(code, 0, "{lines:?}");
        let summary: Value = serde_json::from_str(&text(&dir.join("whole"), "summary.json")).unwrap();
        let steps = summary["steps"].as_u64().unwrap();
        println!("unbroken: {steps} steps, events {}", summary["events"]);
        println!("stages {}", summary["objects"][0]["stages"]);

        // The cut: the same life stopped at the dump CONTINUE_CUT, then continued from it.
        let cut_life = continue_args(CONTINUE_CUT);
        let cut_life: Vec<&str> = cut_life.iter().map(String::as_str).collect();
        assert_eq!(sun_life(&dir.join("cut"), &cut_life).0, 2, "cut by --max-steps");
        let from = dir.join("cut").join("dumps").join(format!("s_{CONTINUE_CUT}.bin")).display().to_string();
        let (code, lines) = sun_life(&dir.join("on"), &[&life[..], &["--load-dump", &from, "--continue"]].concat());
        assert_eq!(code, 0, "{lines:?}");
        Continued { dir, steps }
    })
}

#[test]
fn a_life_cut_at_a_dump_and_continued_writes_the_unbroken_lifes_summary() {
    let Continued { dir, steps } = continued_life();
    let (whole, cut, on) = (dir.join("whole"), dir.join("cut"), dir.join("on"));
    let life = continue_args(2000);
    let life: Vec<&str> = life.iter().map(String::as_str).collect();
    let from = cut.join("dumps").join(format!("s_{CONTINUE_CUT}.bin")).display().to_string();

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

/// M0-D103 — summary.json's four readouts (§2.12.2): a short life of the Sun-like preset at its calibrated mass, as headless
/// loads `preset:sun` (`Scene::with_embedded_masses`), to [`READOUT_UNTIL`] with a progress tick every frame — the summary and
/// each frame's heaviest object as the Watch saw it (none on a frame without one). Run once, read by the two cases below.
const READOUT_UNTIL: &str = "sim_time:20";

struct Watched {
    summary: Value,
    frames: Vec<(f64, Option<Star>)>,
}

fn watched_sun() -> &'static Watched {
    use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick};
    static LIFE: std::sync::OnceLock<Watched> = std::sync::OnceLock::new();
    LIFE.get_or_init(|| {
        let out = temp("readouts").join("out");
        let mut args: Vec<String> =
            ["--scene", "preset:sun", "--until", READOUT_UNTIL, "--max-steps", "2000", "--out", &out.display().to_string()].map(String::from).to_vec();
        if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
            args.extend(["--adapter".to_string(), a]);
        }
        let scene = sr_engine::scene::Scene::preset("sun", Default::default()).unwrap().with_embedded_masses();
        let mut frames = Vec::new();
        let (code, lines) = {
            let mut tick = |t: &Tick| frames.push((t.sim_time, t.star.clone()));
            let mut saved = |_: u64, _: f64, _: &str| {};
            let progress = Progress { every: 1, tick: &mut tick, checkpoint_every: None, saved: &mut saved };
            run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress))
        };
        assert_eq!(code, 0, "{lines:?}");
        let summary: Value = serde_json::from_str(&text(&out, "summary.json")).unwrap();
        println!("readouts life: {} steps, {} frames, objects {}", summary["steps"], frames.len(), summary["objects"]);
        Watched { summary, frames }
    })
}

/// A fate's or an ending's id: its strings-table key without the prefix.
fn key_id(key: &str) -> &str {
    key.split_once('.').map_or(key, |(_, id)| id)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * b.abs()
}

/// Each listed object's `mass_suns`, `predicted_ending` and `ending` are the library's for its `mass_sb` — MassScale::to_suns,
/// Predictor::predict (the fate ids), no ending in a life's first t.u. — and the last frame's heaviest object's `age_years` is
/// what the library's AgeClock reads when fed every frame's heaviest object (its mass, central hydrogen and shown stage) — the
/// stages a life this short shows have no fuel but hydrogen. Within 1e-12: serde_json's float parse is not correctly rounded
/// (M0-D100); the Watch's own reading of the last frame equals the library's bit for bit.
#[test]
fn the_four_readouts_are_the_librarys_for_the_last_frame() {
    let cal = embedded_calibration().expect("assets/calibration.json is embedded: summary.json's readouts read it");
    let life = watched_sun();
    let scale = MassScale::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).unwrap();
    let predictor = Predictor::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).unwrap();
    let objects = life.summary["objects"].as_array().expect("objects");
    assert!(!objects.is_empty(), "the Sun-like cloud is listed");
    for o in objects {
        let m = o["mass_sb"].as_f64().unwrap();
        assert!(close(o["mass_suns"].as_f64().expect("mass_suns"), scale.to_suns(m)), "{o}");
        let want = match predictor.predict(m).unwrap() {
            Prediction::Sure(f) => json!([key_id(f.key())]),
            Prediction::CloseCall(a, b) => json!([key_id(a.key()), key_id(b.key())]),
        };
        assert_eq!(o["predicted_ending"], want, "{o}");
        assert!(matches!(predictor.readout(m, None), Some(Readout::Predicted(_))));
        assert!(o.get("ending").is_some_and(Value::is_null), "no remnant has held in {READOUT_UNTIL}: {o}");
    }
    let mut clock = AgeClock::from_calibration(&cal).unwrap();
    for (t, star) in &life.frames {
        let Some(s) = star else {
            clock.update(*t, *t, &[], |_| None);
            continue;
        };
        let shown = s.shown.map(|id| Stage::from_id(&json!(id)).unwrap());
        assert!(matches!(shown, None | Some(Stage::Cloud | Stage::Collapsing | Stage::Protostar | Stage::MainSequence)), "{shown:?}");
        let mut o = Object { id: s.id, mass: s.mass, ..Object::default() };
        o.x_c[0] = s.x_h_c;
        clock.update(*t, *t, &[o], |_| shown);
    }
    let last = life.frames.last().and_then(|(_, s)| s.as_ref()).expect("the last frame has an object");
    let want = clock.age(last.id).expect("the heaviest object has shown a stage");
    assert_eq!(last.age_years, Some(want), "the Watch's clock is the library's, fed every frame");
    let o = objects.iter().find(|o| o["id"] == last.id).expect("the last frame's heaviest object is listed");
    let age = o["age_years"].as_f64().expect("age_years");
    assert!(age > 0.0 && close(age, want), "age_years {age} against the library's {want}: {o}");
    println!("readouts: {:.4} Suns, {}, age {age:.6e} years ({} shown)", o["mass_suns"], o["predicted_ending"], o["stage"]);
}

/// The age never decreases over the frames and is not one jump (the adversarial line: an age computed once from the last
/// frame reads 0, then everything at once): the Watch's per-frame readings of the life's heaviest object rise frame by frame,
/// on at least [`AGE_RISES`] frames, no frame adding more than half the final age.
const AGE_RISES: usize = 10;

#[test]
fn the_age_never_decreases_over_the_frames() {
    let life = watched_sun();
    let id = life.frames.last().and_then(|(_, s)| s.as_ref()).expect("an object").id;
    let ages: Vec<f64> = life.frames.iter().filter_map(|(_, s)| s.as_ref().filter(|s| s.id == id).and_then(|s| s.age_years)).collect();
    for (k, w) in ages.windows(2).enumerate() {
        assert!(w[1] >= w[0], "frame {k}: the age fell from {} to {}", w[0], w[1]);
    }
    let rises = ages.windows(2).filter(|w| w[1] > w[0]).count();
    let total = *ages.last().expect("ages");
    let biggest = ages.windows(2).map(|w| w[1] - w[0]).fold(0.0, f64::max);
    println!("age over {} frames: {rises} rises, the largest {biggest:.4e} of {total:.4e} years", ages.len());
    assert!(rises >= AGE_RISES, "the age rose on {rises} frames: {ages:?}");
    assert!(total > 0.0 && biggest <= 0.5 * total, "one frame added {biggest} of {total}");
}

/// A life cut at a dump and continued counts its age on from the cut (§0.5 [M0-D103]): the continued life's `age_years` are
/// the unbroken life's, object by object, bit for bit — and the clock had already run at the cut (the cut run's own
/// summary.json lists a positive age), so a snapshot without the clock would restart it.
#[test]
fn a_continued_lifes_ages_are_the_unbroken_lifes() {
    let Continued { dir, .. } = continued_life();
    let objects = |d: &str| -> Vec<Value> {
        let summary: Value = serde_json::from_str(&text(&dir.join(d), "summary.json")).unwrap();
        summary["objects"].as_array().expect("objects").clone()
    };
    let at_cut = objects("cut");
    assert!(at_cut.iter().any(|o| o["age_years"].as_f64().is_some_and(|a| a > 0.0)), "the clock ran before the cut: {at_cut:?}");
    let (whole, on) = (objects("whole"), objects("on"));
    assert_eq!(whole.len(), on.len(), "{whole:?} against {on:?}");
    for (w, o) in whole.iter().zip(&on) {
        assert_eq!(w["id"], o["id"]);
        let age = w["age_years"].as_f64().expect("the unbroken life's age");
        assert_eq!(o["age_years"].as_f64(), Some(age), "object {}: the continued life's age", w["id"]);
    }
    println!("ages at the cut {:?}, unbroken and continued {:?}", at_cut.iter().map(|o| &o["age_years"]).collect::<Vec<_>>(), whole.iter().map(|o| &o["age_years"]).collect::<Vec<_>>());
}

/// The wisp cut is 1 % of calibration's Sun-like preset (§1.9.2; M0-V10's F2, merged here by M0-V22): one cold rect in a
/// 128 × 128 bounce world, Σ 0.1 over 25 × 12 (30.0) or 25 × 13 (32.5), one step — 32.5 listed, 30.0 not, the cut
/// 0.01 × `preset_mass_sb.sun` between them; the nominal Sun-like cloud's cut (33.5) would drop both (M0-V10's probe, RTX 4090).
#[test]
fn a_cloud_over_a_hundredth_of_the_calibrated_sun_is_listed() {
    let cal = embedded_calibration().expect("assets/calibration.json is embedded");
    let cut = 0.01 * cal.preset_mass_sb.sun;
    assert!(30.0 < cut && cut < 32.5 && 32.5 < 0.01 * SUN_MASS_NOMINAL, "the rects straddle the cut {cut}, and the nominal one drops both");
    let dir = temp("wisp");
    let listed = |h: u32| -> Vec<Value> {
        let scene = json!({"version": 1, "world": {"width": 128, "height": 128, "edge": "bounce"}, "objects": [
            {"kind": "rect", "x0": 50, "y0": 50, "w": 25, "h": h, "sigma": 0.1, "t": 0.05, "composition": {"H": 1.0}, "v": [0, 0]}]});
        let file = dir.join(format!("wisp_{h}.json"));
        std::fs::write(&file, scene.to_string()).unwrap();
        let out = go(&format!("wisp-{h}"), &file, &["--steps", "1"]);
        assert_eq!(out.code, 0, "{:?}", out.lines);
        out.summary["objects"].as_array().expect("objects").clone()
    };
    let (light, heavy) = (listed(12), listed(13));
    assert!(light.is_empty(), "30.0 is a wisp under {cut}: {light:?}");
    assert_eq!(heavy.len(), 1, "32.5 is over {cut}: {heavy:?}");
    assert!((heavy[0]["mass_sb"].as_f64().unwrap() - 32.5).abs() < 0.01, "{heavy:?}");
}

/// A frame of one 4 × 4 block holding `mass` of cold hydrogen at cell (2, 2) of a 64-wide world, three empty blocks beside it.
fn one_block(mass: f32) -> BlockSummary {
    let block = |m: f32| Block {
        mass: m,
        mx: 2.5 * m,
        my: 2.5 * m,
        momentum: [0.0; 2],
        kinetic: 0.0,
        thermal: 0.1 * m,
        cold: 0.0,
        pressure: 0.1 * m,
        species: std::array::from_fn(|i| if i == 0 { m } else { 0.0 }),
        phi_m: 0.0,
        max_cell: if m > 0.0 { 2 * 64 + 2 } else { 0 },
        max_sigma: m / 16.0,
        max_t: 0.05,
        max_x: std::array::from_fn(|i| if i == 0 && m > 0.0 { 1.0 } else { 0.0 }),
        cold_ratio: 0.0,
        radiated: 0.0,
        nuclear: [0.0; 7],
        neutrino_lost: 0.0,
        neutrino_deposited: 0.0,
        unbound: 0.0,
        sinks: 0,
    };
    BlockSummary { origin: [0, 0], size: [8, 8], columns: 2, blocks: vec![block(mass), block(0.0), block(0.0), block(0.0)], sinks: Vec::new() }
}

/// With no calibration embedded the four readouts are written null (M0-D103's Deliver): the same frame through a Watch on no
/// calibration and on the embedded one — the object listed by both, its four keys there and null on the first, the mass and
/// the prediction filled on the second (no stage has held after one frame: the age is null on both).
#[test]
fn a_watch_with_no_calibration_writes_the_four_as_null() {
    let physics = Physics::shipped().expect("shipped physics.json");
    let cal = embedded_calibration().expect("assets/calibration.json is embedded");
    let at = At { step: 1, sim_time: 0.1 };
    let first = |cal| {
        let mut watch = Watch::with_calibration(&physics, 64, 0.0, cal);
        watch.update(&one_block(100.0), at);
        let objects = watch.objects_json();
        assert_eq!(objects.len(), 1, "{objects:?}");
        objects[0].clone()
    };
    let (none, some) = (first(None), first(Some(&cal)));
    for key in ["mass_suns", "predicted_ending", "ending", "age_years"] {
        assert!(none.get(key).is_some_and(Value::is_null), "{key} null with no calibration: {none}");
    }
    assert!(some["mass_suns"].is_f64() && some["predicted_ending"].is_array(), "{some}");
    assert!(some["ending"].is_null() && some["age_years"].is_null(), "{some}");
}
