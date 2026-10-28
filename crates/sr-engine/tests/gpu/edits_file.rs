//! M0-T77 — the edits file and `headless --edits` (contract §2.12.5, §3.3, §1.12; crates/sr-engine/src/headless.rs's `EditPlan`)
//! on the physics adapter (R12). The scene (scenes/edits_triggers.json) is a 64 × 64 world with one H disk and every physics
//! switch off, so the only thing that moves a cell's Σ is an edit: a dump after every step (`--dump-every 1`) shows the step
//! each edit landed in, to the cell, and the run summary's ledger shows how much was painted. Each trigger kind is graded by
//! the step its edit appears in — `{"step": n}` lands in step n's P0, a state trigger in the first step after the boundary
//! that held it — and a repeating mass trigger by the number of applications that takes the object's mass past m, no more.
//! The refusals and the two-run identity (§2.12.5: never the wall clock) close it.

use sr_engine::dump::Dump;
use sr_engine::headless::EditPlan;
use sr_engine::state::WorldConfig;
use sr_physics::registry::{Elements, Physics};

const N: usize = 64;
const CELLS: usize = N * N;
const SCENE: &str = "edits_triggers.json";

struct Run {
    code: i32,
    summary: Option<serde_json::Value>,
    /// The planes of the dump after step k (k = 1 … steps), index k − 1; empty without `--dump-every`.
    planes: Vec<Vec<f32>>,
    /// The planes of the last dump.
    last: Vec<f32>,
}

fn scenes_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-edits-file-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// `headless` on the scene, `steps` steps, with `edits` as the edits file when given; a dump after every step when `each`.
fn run(name: &str, edits: Option<&str>, steps: u64, each: bool) -> Run {
    let out = temp(name);
    let _ = std::fs::remove_dir_all(&out);
    let scene = scenes_dir().join(SCENE).display().to_string();
    let mut args: Vec<String> = ["--scene", &scene, "--steps", &steps.to_string(), "--out", &out.display().to_string()]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if each {
        args.extend(["--dump-every".to_string(), "1".to_string()]);
    }
    if let Some(text) = edits {
        let file = temp(&format!("{name}.edits.json"));
        std::fs::write(&file, text).unwrap();
        args.extend(["--edits".to_string(), file.display().to_string()]);
    }
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let code = sr_engine::headless::run(&args);
    let summary = std::fs::read_to_string(out.join("summary.json")).ok().map(|t| serde_json::from_str(&t).unwrap());
    let read = |k: u64| Dump::read(&out.join("dumps").join(format!("s_{k}.bin"))).ok().map(|d| d.planes);
    let planes = if each { (1..=steps).filter_map(read).collect() } else { Vec::new() };
    let last = read(steps).unwrap_or_default();
    Run { code, summary, planes, last }
}

fn paint_sigma() -> f64 {
    Physics::shipped().unwrap().get("paint_sigma").unwrap()
}

/// The cells of a brush of `radius` about (x, y): `dx² + dy² ≤ r²` (the shader's disk).
fn brush(x: i64, y: i64, radius: i64) -> Vec<usize> {
    let mut cells = Vec::new();
    for cy in 0..N as i64 {
        for cx in 0..N as i64 {
            if (cx - x).pow(2) + (cy - y).pow(2) <= radius * radius {
                cells.push(cy as usize * N + cx as usize);
            }
        }
    }
    cells
}

fn painted(run: &Run) -> f64 {
    run.summary.as_ref().expect("summary.json")["ledger"]["end"]["mass"]["painted"].as_f64().expect("painted")
}

fn erased(run: &Run) -> f64 {
    run.summary.as_ref().expect("summary.json")["ledger"]["end"]["mass"]["erased"].as_f64().expect("erased")
}

fn mass(run: &Run) -> f64 {
    run.summary.as_ref().expect("summary.json")["objects"][0]["mass_sb"].as_f64().expect("the object's mass")
}

/// Grades each dump against `base` (the unedited run's plane): in the cells of `at` brushes Σ is `base + p · apps(k)`,
/// k the dump's step count; every other cell is bit-identical to `base`.
fn grade(name: &str, run: &Run, base: &[f32], brushes: &[(Vec<usize>, Box<dyn Fn(usize) -> usize>)]) {
    let p = paint_sigma();
    for (i, planes) in run.planes.iter().enumerate() {
        let k = i + 1;
        let mut touched = vec![false; CELLS];
        for (cells, apps) in brushes {
            for &c in cells {
                touched[c] = true;
                let want = f64::from(base[c]) + p * apps(k) as f64;
                let got = f64::from(planes[c]);
                assert!((got - want).abs() < 2e-6, "{name}: after {k} steps cell {c}: Σ {got} where {want} was expected ({} applications)", apps(k));
            }
        }
        let moved = (0..CELLS).filter(|&c| !touched[c] && planes[c].to_bits() != base[c].to_bits()).count();
        assert_eq!(moved, 0, "{name}: after {k} steps {moved} cells outside the brushes differ from the unedited run's");
    }
}

fn base_run(steps: u64) -> Run {
    let base = run("base", None, steps, true);
    assert_eq!(base.code, 0);
    assert!(base.planes.windows(2).all(|w| w[0][..CELLS] == w[1][..CELLS]), "the scene is static with every switch off");
    base
}

/// E1 paints He twice at step 3; E2 paints C when the heaviest object's central hydrogen first falls under 0.97 — which E1's
/// He does, read at the boundary after step 3 — so E2 lands in step 4.
const CASCADE: &str = r#"[
  {"at": {"step": 3}, "edit": {"kind": "paint", "x": 30, "y": 30, "radius": 3, "species": "He", "frames": 2}},
  {"at": {"object_until": {"x_h_c_below": 0.97}}, "edit": {"kind": "paint", "x": 36, "y": 34, "radius": 2, "species": "C"}}
]"#;

#[test]
fn a_step_trigger_and_a_state_trigger_land_in_the_step_they_name() {
    let base = base_run(8);
    let r = run("cascade", Some(CASCADE), 8, true);
    assert_eq!(r.code, 0);
    let (e1, e2) = (brush(30, 30, 3), brush(36, 34, 2));
    assert_eq!((e1.len(), e2.len()), (29, 13));
    // After k steps: E1 (twice) once step 3 has run, E2 (once) once step 4 has.
    grade(
        "cascade",
        &r,
        &base.planes[0],
        &[(e1.clone(), Box::new(|k| if k >= 4 { 2 } else { 0 })), (e2.clone(), Box::new(|k| usize::from(k >= 5)))],
    );
    let want = paint_sigma() * (2.0 * 29.0 + 13.0);
    assert!((painted(&r) - want).abs() < 1e-4, "painted {} against {want}", painted(&r));
    println!("cascade: E1 in step 3 (2×{} cells), E2 in step 4 ({} cells), painted {:.4}", e1.len(), e2.len(), painted(&r));
}

#[test]
fn a_chunked_run_stops_at_the_step_trigger() {
    // No --dump-every: the run takes chunks, and a chunk must end at step 3 for the edit to be queued there.
    let only_e1 = r#"[{"at": {"step": 3}, "edit": {"kind": "paint", "x": 30, "y": 30, "radius": 3, "species": "He", "frames": 2}}]"#;
    let ends = run("chunk_3", Some(only_e1), 3, false);
    let past = run("chunk_4", Some(only_e1), 4, false);
    let long = run("chunk_40", Some(only_e1), 40, false);
    assert_eq!((ends.code, past.code, long.code), (0, 0, 0));
    // Step 3 is the fourth step: a run of 3 steps never reaches it.
    assert_eq!(painted(&ends), 0.0, "a run of 3 steps applied an edit of step 3");
    let want = paint_sigma() * 2.0 * 29.0;
    for (name, r) in [("4 steps", &past), ("40 steps", &long)] {
        assert!((painted(r) - want).abs() < 1e-4, "{name}: painted {} against {want}", painted(r));
    }
    println!("chunked: painted 0 after 3 steps, {:.4} after 4 and after 40", painted(&past));
}

#[test]
fn a_mass_repeat_stops_once_the_mass_passes() {
    let base = base_run(14);
    let base_mass = mass(&base);
    // Each application paints 29 cells × paint_sigma of H on the object: 0.58. Five fall short of base + 3.05, six pass it.
    let per = 29.0 * paint_sigma();
    let m = base_mass + 3.05;
    let text = format!(
        r#"[{{"at": {{"mass_until": {{"at_least": {m}}}}}, "edit": {{"kind": "paint", "x": 32, "y": 32, "radius": 3, "species": "H"}}}}]"#
    );
    let r = run("mass_up", Some(&text), 14, true);
    assert_eq!(r.code, 0);
    // The first object is found after step 1, so the first application is step 1's and the sixth step 6's: after k steps
    // there are min(max(k − 1, 0), 6) of them.
    grade("mass_up", &r, &base.planes[0], &[(brush(32, 32, 3), Box::new(|k| k.saturating_sub(1).min(6)))]);
    assert!((painted(&r) - 6.0 * per).abs() < 1e-4, "painted {} against {}", painted(&r), 6.0 * per);
    assert!(mass(&r) >= m && mass(&r) - per < m, "the object's mass {} is not the first to pass {m} (one application less: {})", mass(&r), mass(&r) - per);
    // At most: an object already lighter than m takes nothing; an erase that takes it under m repeats no more.
    let above = format!(
        r#"[{{"at": {{"mass_until": {{"at_most": {}}}}}, "edit": {{"kind": "erase", "x": 32, "y": 32, "radius": 4}}}}]"#,
        base_mass + 1.0
    );
    let none = run("mass_down_none", Some(&above), 6, false);
    assert_eq!((none.code, erased(&none)), (0, 0.0), "a mass already at most m erased something");
    let under = format!(
        r#"[{{"at": {{"mass_until": {{"at_most": {}}}}}, "edit": {{"kind": "erase", "x": 32, "y": 32, "radius": 4}}}}]"#,
        base_mass - 20.0
    );
    let once = run("mass_down", Some(&under), 10, true);
    let first = erased(&once);
    assert!(first > 20.0, "one erase of 49 cells took only {first}");
    // The erase of step 1 took the object under m; it repeated no more: the ledger holds that one erase's mass, and the
    // dumps hold the erased cells at the floor from step 2 on.
    let after = once.planes.last().unwrap();
    assert!(brush(32, 32, 4).iter().all(|&c| after[c] < 0.1), "the erased cells hold gas");
    let early = run("mass_down_2", Some(&under), 3, false);
    assert!((erased(&early) - first).abs() < 1e-3, "a second erase: {} after 3 steps, {first} after 10", erased(&early));
    println!("mass: 6 applications of {per:.2} to pass {m:.2}; at_most: 0 erases above, {first:.1} erased once under");
}

#[test]
fn the_queue_takes_eight_edits_a_step_in_the_files_order() {
    let base = base_run(6);
    // Two edits of eight frames at step 2: the first's eight apply in step 2, the second's in step 3.
    let text = r#"[
      {"at": {"step": 2}, "edit": {"kind": "paint", "x": 20, "y": 20, "radius": 2, "species": "He", "frames": 8}},
      {"at": {"step": 2}, "edit": {"kind": "paint", "x": 50, "y": 50, "radius": 2, "species": "C", "frames": 8}}
    ]"#;
    let r = run("queue", Some(text), 6, true);
    assert_eq!(r.code, 0);
    grade(
        "queue",
        &r,
        &base.planes[0],
        &[(brush(20, 20, 2), Box::new(|k| if k >= 3 { 8 } else { 0 })), (brush(50, 50, 2), Box::new(|k| if k >= 4 { 8 } else { 0 }))],
    );
    println!("queue: eight in step 2, the next eight in step 3");
}

#[test]
fn two_runs_are_bit_identical() {
    let (a, b) = (run("twice_a", Some(CASCADE), 8, true), run("twice_b", Some(CASCADE), 8, true));
    assert_eq!((a.code, b.code), (0, 0));
    assert_eq!(a.planes.len(), 8);
    for (k, (x, y)) in a.planes.iter().zip(&b.planes).enumerate() {
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "the dumps after {} steps differ between two runs", k + 1);
    }
    let (sa, sb) = (a.summary.unwrap(), b.summary.unwrap());
    for key in ["steps", "sim_time", "ledger", "events", "objects"] {
        assert_eq!(sa[key], sb[key], "summary.{key} differs between two runs");
    }
    // The chunked run (no dump after each step) ends on the same state: a trigger reads the step boundary, not the chunk.
    let chunked = run("twice_chunked", Some(CASCADE), 8, false);
    assert!(a.last.iter().zip(&chunked.last).all(|(p, q)| p.to_bits() == q.to_bits()), "a chunked run ends on another state");
    println!("two runs: 8 dumps and the summary bit-identical, the chunked run's last dump too");
}

#[test]
fn a_bad_edits_file_is_refused() {
    let (elements, world) = (Elements::shipped().unwrap(), WorldConfig::new(64, 64).unwrap());
    let edit = |extra: &str| format!(r#"{{"kind": "paint", "x": 10, "y": 10, "radius": 3, "species": "H"{extra}}}"#);
    let entry = |at: &str, edit: &str| format!(r#"[{{"at": {at}, "edit": {edit}}}]"#);
    assert!(EditPlan::parse(&entry(r#"{"step": 0}"#, &edit("")), world, &elements).is_ok());
    assert!(EditPlan::parse("[]", world, &elements).is_ok());
    let cases: Vec<(&str, String)> = vec![
        ("not a list", r#"{"at": {"step": 1}}"#.to_string()),
        ("unknown entry key", format!(r#"[{{"at": {{"step": 1}}, "edit": {}, "when": 1}}]"#, edit(""))),
        ("unknown edit key", entry(r#"{"step": 1}"#, &edit(r#", "colour": "red""#))),
        ("two triggers", entry(r#"{"step": 1, "mass_until": {"at_least": 1}}"#, &edit(""))),
        ("a fractional step", entry(r#"{"step": 1.5}"#, &edit(""))),
        ("a negative step", entry(r#"{"step": -1}"#, &edit(""))),
        ("an unknown condition", entry(r#"{"object_until": {"x_he_c_below": 0.5}}"#, &edit(""))),
        ("both mass bounds", entry(r#"{"mass_until": {"at_least": 1, "at_most": 2}}"#, &edit(""))),
        ("a negative mass", entry(r#"{"mass_until": {"at_least": -1}}"#, &edit(""))),
        ("a cell outside the world", entry(r#"{"step": 1}"#, &edit("").replace("\"x\": 10", "\"x\": 64"))),
        ("a radius of 0", entry(r#"{"step": 1}"#, &edit("").replace("\"radius\": 3", "\"radius\": 0"))),
        ("a radius of 41", entry(r#"{"step": 1}"#, &edit("").replace("\"radius\": 3", "\"radius\": 41"))),
        ("an unknown species", entry(r#"{"step": 1}"#, &edit("").replace("\"H\"", "\"Xx\""))),
        ("a paint with no species", entry(r#"{"step": 1}"#, r#"{"kind": "paint", "x": 1, "y": 1, "radius": 1}"#)),
        ("an erase with a species", entry(r#"{"step": 1}"#, r#"{"kind": "erase", "x": 1, "y": 1, "radius": 1, "species": "H"}"#)),
        ("an unknown kind", entry(r#"{"step": 1}"#, &edit("").replace("paint", "smear"))),
        ("frames of 0", entry(r#"{"step": 1}"#, &edit(r#", "frames": 0"#))),
        ("frames of 9", entry(r#"{"step": 1}"#, &edit(r#", "frames": 9"#))),
    ];
    for (what, text) in &cases {
        let e = EditPlan::parse(text, world, &elements).expect_err(what);
        println!("refused: {what}: {e}");
    }
    // The binary: exit 4 and an `SR-ERROR bad edits` line, before any GPU is touched; --cpu-reference has no edits.
    let scene = scenes_dir().join(SCENE).display().to_string();
    let bad = temp("bad.edits.json");
    std::fs::write(&bad, &cases[0].1).unwrap();
    let args = |extra: &[&str]| -> Vec<String> {
        ["--scene", &scene, "--steps", "2"].iter().chain(extra).map(|s| s.to_string()).collect()
    };
    let (code, lines) = sr_engine::headless::run_captured(&args(&["--edits", &bad.display().to_string()]));
    assert_eq!(code, 4, "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-ERROR bad edits")), "{lines:?}");
    let (code, _) = sr_engine::headless::run_captured(&args(&["--edits", &temp("missing.json").display().to_string()]));
    assert_eq!(code, 4, "a missing file");
    let (code, lines) = sr_engine::headless::run_captured(&args(&["--cpu-reference", "--edits", &bad.display().to_string()]));
    assert_eq!(code, 4, "{lines:?}");
}
