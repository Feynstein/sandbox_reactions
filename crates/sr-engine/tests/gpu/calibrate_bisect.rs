//! M0-T68 — `calibrate` items 3–4: the bisection driver and the items' sides (contract §2.11 items 3–4, §3.3, §6.4). Every
//! test here but the last runs no life — every run is a stub — and the module lives in the one test binary (R11) so the scope
//! runs through `tests/gpu.sh`. The real lives are the lead's, in M0-V9's window; the last test (M0-D60) runs two short real
//! ones, a few hundred steps each, on the test adapter (`SR_TEST_ADAPTER`), to see what a life leaves behind.
//!
//! The driver is graded on stubbed outcomes: a threshold at a known mass, the runs it spends and the masses it asks for. The
//! items' mapping from a life to a side is graded through [`measure`], the same call `calibrate` makes with a real run.
//! Plant: `bisect-tolerance` (the driver stops at 20 %).
//!
//! M0-D75: m_up_sb's lives stop once their side is known (`Until::CollapseOrEnding`) and `--progress-every N` sets a life's tick.
//! M0-D77: `--checkpoint-every M` saves a running life's checkpoint apart from the tick, and `--resume` continues a cut life from it.
//! Plants: `calibrate-checkpoint-without-observers`, `calibrate-checkpoint-on-the-tick`, `calibrate-checkpoint-step-unchecked`.
//! M0-D80: a life under a progress hook runs one step a frame while the physics is hard (P5's ceiling, a core collapse), each a
//! progress line with the frame's and the heaviest object's numbers. Plant: `calibrate-one-step-never`.

use sr_engine::headless::{capturing, run_captured, run_scene_captured, say, unconverged_steps, Args};
use sr_engine::calibrate::{bisect, measure, run_with, run_with_progress, side_bh, side_ign, side_up, CalError, Life, Side, Threshold, BISECT_TOLERANCE};
use sr_engine::scene::Scene;
use sr_engine::watch::{RunEvent, Until, Watch};
use sr_physics::registry::Physics;

/// A stub that is Light under `threshold` and Heavy from it, recording every mass asked for.
fn stub(threshold: f64, asked: &mut Vec<f64>) -> impl FnMut(f64) -> Result<Side, CalError> + '_ {
    move |mass| {
        asked.push(mass);
        Ok(if mass < threshold { Side::Light } else { Side::Heavy })
    }
}

fn life(mass: f64, ignited: bool, core_collapse: bool, ending: Option<&str>) -> Life {
    Life { mass, ignited, core_collapse, collapsed: false, ending: ending.map(str::to_string), steps: 1000, sim_time: 50.0, wall_s: 1.0 }
}

#[test]
fn the_tolerance_is_2_percent_and_the_bracket_closes_on_the_threshold() {
    assert_eq!(BISECT_TOLERANCE, 0.02, "§2.11: bisection to 2 %");
    for threshold in [412.3, 1234.5, 3300.0, 9100.0, 19_999.0] {
        let mut asked = Vec::new();
        let found = bisect((100.0, 20_000.0), BISECT_TOLERANCE, &mut stub(threshold, &mut asked)).expect("a bracket around the threshold");
        let (lo, hi) = found.bracket;
        assert!(lo <= threshold && threshold <= hi, "{threshold} is inside the final bracket {lo}..{hi}");
        assert!(hi / lo - 1.0 <= BISECT_TOLERANCE, "{threshold}: bracket {lo}..{hi} is wider than 2 %");
        assert!((found.value / threshold - 1.0).abs() <= BISECT_TOLERANCE / 2.0 + 1e-12, "{threshold}: value {} is not within 1 %", found.value);
        assert_eq!(found.runs, asked.len(), "the count is the runs spent");
    }
}

#[test]
fn it_asks_both_ends_first_and_spends_the_expected_number_of_runs() {
    let mut asked = Vec::new();
    let found = bisect((100.0, 20_000.0), BISECT_TOLERANCE, &mut stub(1234.5, &mut asked)).unwrap();
    assert_eq!(asked[..2], [100.0, 20_000.0], "the bracket's ends come first, low then high");
    assert!(asked.iter().all(|m| (100.0..=20_000.0).contains(m)), "every mass asked for is inside the bracket: {asked:?}");
    // Each run halves the log-width: ln(200) / 2ⁿ must fall to ln(1.02), n bisections after the two ends.
    let n = (0..40i32).find(|n| (20_000f64 / 100.0).ln() / 2f64.powi(*n) <= 1.02f64.ln()).unwrap();
    assert_eq!(found.runs, 2 + n as usize, "runs for a 200-fold bracket at 2 %");
    assert!(found.runs <= 20, "a bisection is a handful of lives, not a scan: {}", found.runs);
    // A bracket already inside the tolerance costs the two ends and nothing more.
    let mut asked = Vec::new();
    let found = bisect((1000.0, 1010.0), BISECT_TOLERANCE, &mut stub(1005.0, &mut asked)).unwrap();
    assert_eq!((found.runs, asked.len()), (2, 2));
    assert!((found.value - 1005.0).abs() < 1.0);
}

#[test]
fn a_bracket_whose_ends_agree_is_refused_after_two_runs() {
    for (name, threshold) in [("both Heavy", 50.0), ("both Light", 50_000.0)] {
        let mut asked = Vec::new();
        let error = bisect((100.0, 20_000.0), BISECT_TOLERANCE, &mut stub(threshold, &mut asked)).expect_err(name);
        assert_eq!(asked, [100.0, 20_000.0], "{name}: only the two ends were run");
        match error {
            CalError::Failed(m) => assert!(m.contains("ends agree") && m.contains("--bracket"), "{name}: {m}"),
            other => panic!("{name}: {other:?}"),
        }
    }
    // Wrong way round (Heavy under, Light over): not monotone, refused as well, naming it.
    let mut asked = Vec::new();
    let mut backwards = |mass: f64| {
        asked.push(mass);
        Ok(if mass < 1000.0 { Side::Heavy } else { Side::Light })
    };
    match bisect((100.0, 20_000.0), BISECT_TOLERANCE, &mut backwards) {
        Err(CalError::Failed(m)) => assert!(m.contains("wrong way round"), "{m}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(asked.len(), 2);
    // A bad bracket or tolerance is a bad argument (exit 4), no run made.
    let mut never = |_: f64| -> Result<Side, CalError> { panic!("no run for a bad bracket") };
    for bracket in [(0.0, 5.0), (5.0, 5.0), (9.0, 5.0), (f64::NAN, 5.0)] {
        assert_eq!(bisect(bracket, 0.02, &mut never).unwrap_err().exit_code(), 4, "{bracket:?}");
    }
    assert_eq!(bisect((1.0, 5.0), 0.0, &mut never).unwrap_err().exit_code(), 4);
}

#[test]
fn a_run_that_fails_stops_the_bisection_and_is_reported() {
    let mut asked = Vec::new();
    let mut failing = |mass: f64| {
        asked.push(mass);
        if asked.len() == 3 {
            Err(CalError::Failed("the run reached no ending within --max-steps 5".to_string()))
        } else {
            Ok(if mass < 1234.5 { Side::Light } else { Side::Heavy })
        }
    };
    let error = bisect((100.0, 20_000.0), BISECT_TOLERANCE, &mut failing).expect_err("the third run fails");
    assert_eq!(error, CalError::Failed("the run reached no ending within --max-steps 5".to_string()), "the run's own error, not a guess");
    assert_eq!(asked.len(), 3, "no run after the failure");
    assert_eq!(error.exit_code(), 6);
}

#[test]
fn each_item_places_a_life_on_a_side_or_refuses_it() {
    // m_ign: ignition over, a failed star's ending under; anything else is neither.
    assert_eq!(side_ign(&life(1.0, true, false, None)).unwrap(), Side::Heavy);
    assert_eq!(side_ign(&life(1.0, false, false, Some("failed_star"))).unwrap(), Side::Light);
    for bad in [life(1.0, false, false, None), life(1.0, false, false, Some("white_dwarf"))] {
        assert!(matches!(side_ign(&bad), Err(CalError::Failed(m)) if m.contains("neither side")), "{bad:?}");
    }
    // m_up: core collapse over (its ending a neutron star or a black hole), a white dwarf under.
    assert_eq!(side_up(&life(1.0, true, true, Some("neutron_star"))).unwrap(), Side::Heavy);
    assert_eq!(side_up(&life(1.0, true, false, Some("white_dwarf"))).unwrap(), Side::Light);
    assert!(side_up(&life(1.0, false, false, Some("failed_star"))).is_err());
    assert!(side_up(&life(1.0, true, false, None)).is_err());
    // m_bh: a black hole over, a neutron star under; a white dwarf is below the bracket, not a side.
    assert_eq!(side_bh(&life(1.0, true, true, Some("black_hole"))).unwrap(), Side::Heavy);
    assert_eq!(side_bh(&life(1.0, true, true, Some("neutron_star"))).unwrap(), Side::Light);
    assert!(side_bh(&life(1.0, true, false, Some("white_dwarf"))).is_err());
}

#[test]
fn measure_runs_the_items_mapping_and_logs_every_run() {
    // A stub physics: ignites from 280, core-collapses from 5,400, forms a black hole from 11,000.
    let live = |threshold: Threshold, mass: f64| -> Result<Life, CalError> {
        Ok(match threshold {
            Threshold::Ign if mass < 280.0 => life(mass, false, false, Some("failed_star")),
            Threshold::Ign => life(mass, true, false, None),
            _ if mass < 5400.0 => life(mass, true, false, Some("white_dwarf")),
            _ if mass < 11_000.0 => life(mass, true, true, Some("neutron_star")),
            _ => life(mass, true, true, Some("black_hole")),
        })
    };
    for (threshold, true_mass, bracket) in [
        (Threshold::Ign, 280.0, (100.0, 3351.0)),
        (Threshold::Up, 5400.0, (3351.0, 46_900.0)),
        (Threshold::Bh, 11_000.0, (6750.0, 67_000.0)),
    ] {
        let mut lines = Vec::new();
        let found = measure(threshold, bracket, &mut |t, m| live(t, m), &mut |l| lines.push(l.to_string())).unwrap();
        assert!((found.value / true_mass - 1.0).abs() <= 0.01 + 1e-12, "{}: {} against {true_mass}", threshold.item(), found.value);
        assert_eq!(lines.len(), found.runs, "one SR-CALIBRATE run line per run");
        let item = format!("item={}", threshold.item());
        assert!(lines.iter().all(|l| l.starts_with("SR-CALIBRATE run") && l.contains(&item) && l.contains("steps=") && l.contains("wall_s=") && l.contains("mass=")), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("ending=")), "the ending of each run is logged: {lines:?}");
    }
    // A run on neither side stops the item, logged and with the life in the message.
    let mut lines = Vec::new();
    let error = measure(Threshold::Bh, (6750.0, 67_000.0), &mut |_, m| Ok(life(m, true, false, None)), &mut |l| lines.push(l.to_string())).unwrap_err();
    assert!(matches!(&error, CalError::Failed(m) if m.contains("m_bh_sb") && m.contains("neither side")), "{error:?}");
    assert!(lines.len() == 1 && lines[0].contains("side=none"), "the unplaced run is logged, with no side: {lines:?}");
}

#[test]
fn the_command_checks_the_new_arguments_before_any_run() {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-bisect-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("calibration.json");
    let out = out.to_str().unwrap();
    for (args, needle) in [
        (vec!["--out", out, "--bracket", "100,300"], "one bisected item"),
        (vec!["--out", out, "--only", "m_ch_sb", "--bracket", "100,300"], "one bisected item"),
        (vec!["--out", out, "--only", "m_ign_sb,m_up_sb", "--bracket", "100,300"], "one bisected item"),
        (vec!["--out", out, "--only", "m_up_sb", "--bracket", "300,100"], "0 < LO < HI"),
        (vec!["--out", out, "--only", "m_up_sb", "--bracket", "100"], "LO,HI"),
        (vec!["--out", out, "--only", "m_up_sb", "--max-steps", "0"], "--max-steps"),
        (vec!["--out", out, "--only", "m_ign_sb,nonsense"], "not an item"),
        (vec!["--out", out, "--only", "render_reserve_ms"], "--measure-ui"),
        // M0-D75: a tick is a whole number of steps above 0.
        (vec!["--out", out, "--only", "m_up_sb", "--progress-every", "0"], "--progress-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--progress-every", "-5"], "--progress-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--progress-every", "often"], "--progress-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--progress-every", "1.5"], "--progress-every"),
        (vec!["--out", out, "--only", "m_ch_sb", "--progress-every", "0"], "--progress-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--progress-every"], "--progress-every needs a value"),
        // M0-D77: a save interval is a whole number of steps above 0, as the tick is.
        (vec!["--out", out, "--only", "m_up_sb", "--checkpoint-every", "0"], "--checkpoint-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--checkpoint-every", "-5"], "--checkpoint-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--checkpoint-every", "often"], "--checkpoint-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--checkpoint-every", "1.5"], "--checkpoint-every"),
        (vec!["--out", out, "--only", "m_ch_sb", "--checkpoint-every", "0"], "--checkpoint-every"),
        (vec!["--out", out, "--only", "m_up_sb", "--checkpoint-every"], "--checkpoint-every needs a value"),
    ] {
        let mut lines = Vec::new();
        let code = run_with(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &[], &mut |l| lines.push(l.to_string()));
        assert_eq!(code, 4, "{args:?}: {lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("SR-ERROR") && l.contains(needle)), "{args:?} should say {needle:?}: {lines:?}");
        assert!(lines.iter().all(|l| !l.starts_with("SR-ADAPTER")), "no GPU was touched: {lines:?}");
    }
    assert!(!std::path::Path::new(out).exists());
}

/// M0-D60: a life's summary.json and events.jsonl outlive it (its final dump does not), a run line names them, a failed life's
/// record stays and its error names it, and a progress line comes while a life runs. Two real lives of the nominal cloud and
/// of 1.02 × it: both ignite within a few hundred steps, so m_ign's bracket is refused after two runs, with both runs logged.
#[test]
fn a_life_keeps_its_record_and_says_where_it_is() {
    let m0 = sr_engine::scene::UNIT_MASS;
    let bracket = format!("{},{}", m0, 1.02 * m0);
    let out_dir = std::env::temp_dir().join(format!("sr-calibrate-record-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.join("calibration.json");
    let mut args: Vec<String> = ["--out", out.to_str().unwrap(), "--only", "m_ign_sb", "--bracket", bracket.as_str()].map(String::from).to_vec();
    if let Some(adapter) = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty()) {
        args.extend(["--adapter".to_string(), adapter]);
    }
    let record_of = |line: &str| line.split("record=").nth(1).map(|r| std::path::PathBuf::from(r.trim()));
    let mut stamps = Vec::new();
    let mut clean = |records: &[std::path::PathBuf]| {
        stamps.extend(records.iter().filter_map(|r| r.parent().map(|p| p.to_path_buf())));
    };

    // Two whole lives (ignition ends each), a progress line every 50 steps, the bracket's ends agreeing.
    let mut lines = Vec::new();
    let code = run_with_progress(&args, &[], 50, &mut |l| lines.push(l.to_string()));
    let runs: Vec<&String> = lines.iter().filter(|l| l.starts_with("SR-CALIBRATE run")).collect();
    assert_eq!(code, 6, "the ends agree: {lines:?}");
    assert_eq!(runs.len(), 2, "{lines:?}");
    let records: Vec<_> = runs.iter().map(|l| record_of(l).unwrap_or_else(|| panic!("a run line names its record: {l}"))).collect();
    clean(&records);
    for (record, line) in records.iter().zip(&runs) {
        assert!(line.contains("ignited=true"), "{line}");
        assert!(record.join("summary.json").is_file() && record.join("events.jsonl").is_file(), "{record:?} keeps summary.json and events.jsonl");
        assert!(!record.join("dumps").exists(), "{record:?}: the final dump goes");
        assert!(!record.join("progress.json").exists(), "{record:?}: a finished life's progress.json goes (M0-D74)");
        let summary: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(record.join("summary.json")).unwrap()).unwrap();
        assert!(summary["events"].as_array().is_some_and(|e| e.iter().any(|e| e["kind"] == "ignition")), "{record:?}");
    }
    assert_ne!(records[0], records[1], "each life has its own folder");
    let progress: Vec<&String> = lines.iter().filter(|l| l.starts_with("SR-CALIBRATE progress")).collect();
    assert!(!progress.is_empty(), "a life of a few hundred steps shows a progress line every 50: {lines:?}");
    for l in &progress {
        for key in ["item=m_ign_sb", "mass=", "step=", "sim_time=", "stage="] {
            assert!(l.contains(key), "{key} in {l}");
        }
        // M0-D80: the raw stage beside the shown one.
        assert!(l.contains(" | raw="), "{l}");
    }
    // M0-D80: under each progress line, six rows of the frame's, the star's and the world's numbers, under 80 columns.
    for (i, l) in lines.iter().enumerate().filter(|(_, l)| l.starts_with("SR-CALIBRATE progress")) {
        let rows: Vec<&str> = lines[i + 1..i + 7].iter().map(String::as_str).collect();
        let labels: Vec<&str> = rows.iter().map(|r| r.split_whitespace().next().unwrap_or("")).collect();
        assert_eq!(labels, ["frame", "star", "core", "centre", "light", "world"], "{l}: {rows:?}");
        for (row, names) in rows.iter().zip([
            ["steps", "dt", "s/step", "P5_ceil"],
            ["M", "R", "tau_dyn", "M_unb"],
            ["sigma_c", "T_c", "d_c", "Ye_c"],
            ["X_Fe", "X_n", "v_r", "vr/vdyn"],
            ["L", "L_nuc", "L_N", "L_nu"],
            ["escaped", "objects", "sinks", "sink_M"],
        ]) {
            let words: Vec<&str> = row.split_whitespace().skip(1).collect();
            assert_eq!(words.iter().step_by(2).copied().collect::<Vec<_>>(), names, "{row}");
            assert!(words.iter().skip(1).step_by(2).all(|v| v.parse::<f64>().is_ok()), "a number in each column: {row}");
            assert!(row.starts_with("  ") && row.chars().count() <= 80, "{row}");
        }
    }
    let first_run = lines.iter().position(|l| l.starts_with("SR-CALIBRATE run")).unwrap();
    assert!(lines[..first_run].iter().any(|l| l.starts_with("SR-CALIBRATE progress")), "progress comes while the life runs, before its run line: {lines:?}");

    // A life that fails (no ending within 64 steps) leaves its record, and the error names it.
    let mut failing = args.clone();
    failing.extend(["--max-steps".to_string(), "64".to_string()]);
    let mut lines = Vec::new();
    let code = run_with_progress(&failing, &[], 50, &mut |l| lines.push(l.to_string()));
    assert_eq!(code, 6, "{lines:?}");
    let error = lines.iter().find(|l| l.starts_with("SR-ERROR")).unwrap_or_else(|| panic!("{lines:?}"));
    let record = std::path::PathBuf::from(error.split("record kept: ").nth(1).unwrap_or_else(|| panic!("the error names the record: {error}")).trim_end_matches(')'));
    clean(&[record.clone()]);
    assert!(record.join("summary.json").is_file() && record.join("events.jsonl").is_file(), "{record:?} is kept for a failed life");
    assert!(!record.join("dumps").exists());
    // M0-D74: the failed life's progress.json stays as the trace of where it stopped, and says what the terminal's last progress
    // line of the life said — the same stage and step — with the heat-warning count, the stages shown and the life's start.
    let last = lines.iter().rev().find(|l| l.starts_with("SR-CALIBRATE progress")).unwrap_or_else(|| panic!("a tick before the ceiling: {lines:?}"));
    let said = |key: &str| last.split(&format!("{key}=")).nth(1).and_then(|v| v.split(' ').next()).unwrap_or_else(|| panic!("{key} in {last}")).to_string();
    let progress: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(record.join("progress.json")).unwrap_or_else(|e| panic!("{record:?} keeps progress.json: {e}"))).unwrap();
    assert_eq!(progress["item"], "m_ign_sb");
    assert_eq!(progress["stage"].as_str().unwrap(), said("stage"), "progress.json's stage is the terminal line's: {last}");
    assert_eq!(progress["step"].as_u64().unwrap().to_string(), said("step"), "{last}");
    assert!(progress["sim_time"].as_f64().is_some_and(f64::is_finite) && progress["mass"].as_f64().is_some(), "{progress}");
    assert_eq!(progress["heat_unconverged"].as_u64().unwrap().to_string(), said("heat_unconverged"), "{last}");
    assert_eq!(progress["stages"].as_array().and_then(|s| s.last()).map(|s| s["stage"].clone()), Some(progress["stage"].clone()), "the trail ends on the stage now: {progress}");
    assert!(progress["started"].as_str().is_some_and(|t| t.ends_with('Z') && t.len() == 20), "started is UTC: {progress}");
    assert!(!record.join("progress.json.tmp").exists(), "the temp file is renamed away");
    for key in ["frame", "dt", "s_per_step", "p5_ceiling", "escaped", "objects", "M", "T_c", "Ye_c", "L_N", "raw"] {
        assert!(!progress[key].is_null(), "progress.json carries the line's {key} (M0-D80): {progress}");
    }
    assert!(lines.iter().all(|l| !l.starts_with("SR-WARN heat unconverged")), "a life's heat warnings are counted, not printed: {lines:?}");
    for stamp in stamps {
        let _ = std::fs::remove_dir_all(stamp);
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// M0-D72: `--resume DIR` reads an earlier invocation's finished lives back instead of running them. Real short lives as M0-D60's
/// (the nominal cloud and 1.02 × it, ignition ends each): an invocation resumed whole replays both lives, its run lines and error
/// the unbroken invocation's; one cut during its second life (the record gone, or cut while it was written) replays the first and
/// runs only the second, in the same folder; a record of another mass or an adapter `--adapter` does not match is exit 4 before
/// any life runs; a folder that is not one is exit 4; items 1–2 ignore the flag.
#[test]
fn a_stopped_item_resumes_without_rerunning_a_finished_life() {
    let m0 = sr_engine::scene::UNIT_MASS;
    let bracket = format!("{},{}", m0, 1.02 * m0);
    let out_dir = std::env::temp_dir().join(format!("sr-calibrate-resume-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.join("calibration.json");
    let mut args: Vec<String> = ["--out", out.to_str().unwrap(), "--only", "m_ign_sb", "--bracket", bracket.as_str()].map(String::from).to_vec();
    if let Some(adapter) = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty()) {
        args.extend(["--adapter".to_string(), adapter]);
    }
    let with = |extra: &[&str]| -> Vec<String> { args.iter().cloned().chain(extra.iter().map(|s| s.to_string())).collect() };
    let go = |args: &[String]| {
        let mut lines = Vec::new();
        let code = run_with_progress(args, &[], 50, &mut |l| lines.push(l.to_string()));
        (code, lines)
    };
    let runs_of = |lines: &[String]| lines.iter().filter(|l| l.starts_with("SR-CALIBRATE run")).cloned().collect::<Vec<_>>();
    // A run line without its wall time and its resume mark: what the unbroken invocation said of the same life.
    let same_life = |line: &str| line.replace(" replayed", "").split(" wall_s=").next().unwrap().to_string();
    let error_of = |lines: &[String]| lines.iter().find(|l| l.starts_with("SR-ERROR")).cloned().unwrap_or_default();

    // The unbroken invocation: two lives, the ends agree (exit 6).
    let (code, unbroken) = go(&args);
    assert_eq!(code, 6, "{unbroken:?}");
    let first = runs_of(&unbroken);
    assert_eq!(first.len(), 2, "{unbroken:?}");
    let record_of = |line: &str| std::path::PathBuf::from(line.split("record=").nth(1).unwrap().trim());
    let (rec1, rec2) = (record_of(&first[0]), record_of(&first[1]));
    let dir = rec1.parent().unwrap().to_path_buf();
    let dir_arg = dir.to_str().unwrap().to_string();
    let resumed = |extra: &[&str]| go(&with(&[&["--resume", dir_arg.as_str()], extra].concat()));
    let ran_nothing = |lines: &[String]| lines.iter().all(|l| !l.starts_with("SR-CALIBRATE progress") && !l.starts_with("SR-ADAPTER"));
    let summary_1 = std::fs::read_to_string(rec1.join("summary.json")).unwrap();

    // Resumed whole: both lives read back, no life run, the same lines and the same refusal, in the same folder.
    let (code, lines) = resumed(&[]);
    let runs = runs_of(&lines);
    assert_eq!(code, 6, "{lines:?}");
    assert!(ran_nothing(&lines), "a replay opens no adapter and shows no progress: {lines:?}");
    assert_eq!(runs.len(), 2, "{lines:?}");
    assert!(runs.iter().all(|l| l.contains(" replayed record=")), "{runs:?}");
    assert_eq!(runs.iter().map(|l| same_life(l)).collect::<Vec<_>>(), first.iter().map(|l| same_life(l)).collect::<Vec<_>>());
    assert_eq!(runs.iter().map(|l| l.split("wall_s=").nth(1).unwrap().split(' ').next().unwrap().to_string()).collect::<Vec<_>>(), first.iter().map(|l| l.split("wall_s=").nth(1).unwrap().split(' ').next().unwrap().to_string()).collect::<Vec<_>>(), "a replayed life says the wall time it took");
    assert_eq!(error_of(&lines), error_of(&unbroken));
    assert_eq!(std::fs::read_to_string(rec1.join("summary.json")).unwrap(), summary_1, "a record read back is not rewritten");

    // Cut during the second life — its record gone, then cut while it was written: life 1 read back, life 2 run, in `dir`.
    for cut in [None, Some("{\"version\": 1, \"adapter\": {\"na")] {
        match cut {
            None => std::fs::remove_file(rec2.join("summary.json")).unwrap(),
            Some(half) => std::fs::write(rec2.join("summary.json"), half).unwrap(),
        }
        let (code, lines) = resumed(&[]);
        let runs = runs_of(&lines);
        assert_eq!(code, 6, "{lines:?}");
        assert_eq!(runs.len(), 2, "{lines:?}");
        assert!(runs[0].contains(" replayed record=") && !runs[1].contains("replayed"), "{runs:?}");
        assert_eq!(runs.iter().map(|l| same_life(l)).collect::<Vec<_>>(), first.iter().map(|l| same_life(l)).collect::<Vec<_>>(), "the run sequence is the unbroken one's");
        assert_eq!(record_of(&runs[1]), rec2, "the life runs in the numbering's own folder");
        assert!(lines.iter().filter(|l| l.starts_with("SR-CALIBRATE progress")).all(|l| l.contains(&format!("mass={:.3}", 1.02 * m0))), "only the second life ran: {lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("SR-CALIBRATE progress")), "{lines:?}");
        assert_eq!(error_of(&lines), error_of(&unbroken));
        let summary: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(rec2.join("summary.json")).unwrap()).unwrap();
        assert!(summary["until"]["met"] == true && summary["scene"] == format!("calibrate:m_ign_sb:{:.3}", 1.02 * m0), "{summary}");
        assert_eq!(std::fs::read_to_string(rec1.join("summary.json")).unwrap(), summary_1);
    }

    // Not this calibration's: another mass at the first run or at the second, an adapter that does not match — exit 4, no life run.
    let other_first = format!("{},{}", 0.9 * m0, 1.02 * m0);
    let other_second = format!("{},{}", m0, 1.04 * m0);
    let mut foreign = vec![("another mass at run 1", with(&["--resume", dir_arg.as_str(), "--bracket", other_first.as_str()])), ("another mass at run 2", with(&["--resume", dir_arg.as_str(), "--bracket", other_second.as_str()]))];
    foreign.push(("another adapter", with(&["--resume", dir_arg.as_str(), "--adapter", "no such adapter"])));
    let mut other_label = args.clone();
    other_label[args.iter().position(|a| a == "m_ign_sb").unwrap()] = "m_up_sb".to_string();
    foreign.push(("another item", [other_label, vec!["--resume".to_string(), dir_arg.clone()]].concat()));
    for (what, args) in foreign {
        let (code, lines) = go(&args);
        assert_eq!(code, 4, "{what}: {lines:?}");
        assert!(error_of(&lines).contains("is not this calibration's: start without --resume"), "{what}: {lines:?}");
        assert!(ran_nothing(&lines), "{what}: no life ran: {lines:?}");
    }
    assert_eq!(std::fs::read_to_string(rec1.join("summary.json")).unwrap(), summary_1, "a refused resume leaves the records");

    // A folder that is not one is refused before any run; items 1–2 run no life and ignore the flag.
    let nothing = out_dir.join("no-such-run");
    let (code, lines) = go(&with(&["--resume", nothing.to_str().unwrap()]));
    assert_eq!(code, 4, "{lines:?}");
    assert!(error_of(&lines).contains("--resume") && error_of(&lines).contains("not a folder") && ran_nothing(&lines), "{lines:?}");
    let sheet: Vec<String> = ["--out", out_dir.join("sheet.json").to_str().unwrap(), "--only", "m_ch_sb", "--resume", nothing.to_str().unwrap()].map(String::from).to_vec();
    let (code, lines) = go(&sheet);
    assert_eq!(code, 0, "{lines:?}");

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// M0-D74: a `calibrate` life's `SR-WARN heat unconverged` lines are counted, not printed or kept — a long life reaches P5's
/// ceiling thousands of times and they bury `calibrate`'s own lines; every other line is said as before, and a run that does not
/// count (headless's own, the tests') says them all, never silent.
#[test]
fn a_lifes_heat_warnings_are_counted_not_printed() {
    let warn = "SR-WARN heat unconverged step 10275 passes 13 rounds 4096";
    let ((), kept) = capturing(true, || {
        say(warn);
        say("SR-CALIBRATE progress item=m_up_sb mass=1.000 step=10000 sim_time=80.500 heat_unconverged=1 stage=cloud");
        say(warn);
        say("SR-WARN overrides given: measured and checked, nothing written (§2.12.1)");
    });
    assert_eq!(unconverged_steps(), 2, "both heat warnings counted");
    assert_eq!(kept.len(), 2, "only the progress line and the other warning are said: {kept:?}");
    assert!(kept.iter().all(|l| !l.contains("heat unconverged step")), "{kept:?}");
    assert!(kept.iter().any(|l| l.starts_with("SR-WARN overrides")), "another SR-WARN still shows: {kept:?}");
    let ((), kept) = capturing(false, || say(warn));
    assert_eq!((kept.as_slice(), unconverged_steps()), ([warn.to_string()].as_slice(), 0), "a run that does not count says it, and counts nothing");
}

/// M0-D75: each item's `until`, and what the observers say of it on events — no GPU. m_up_sb's lives stop at the first frame that
/// fires `core_collapse` (or ends), m_ign_sb's at the ignition, m_bh_sb's and the presets' only at their ending: a neutron star
/// can still become a black hole, so m_bh_sb waits for the stage to hold its max(10 t.u., 20 τ_dyn) (§0.5 [M0-D78]).
/// M0-D95: an `ending` event alone stops nothing — the life's ending is the heaviest object's (§0.5 [M0-D95]), and a watch that
/// has seen no frame has no object.
#[test]
fn m_up_stops_at_a_collapse_and_the_others_keep_their_conditions() {
    assert_eq!(Threshold::Up.until(), Until::CollapseOrEnding);
    assert_eq!(Threshold::Ign.until(), Until::IgnitionOrEnding);
    assert_eq!(Threshold::Bh.until(), Until::Ending, "m_bh_sb waits for the ending");
    assert_eq!(Until::CollapseOrEnding.condition(), "collapse|ending");
    assert!(Until::parse("collapse|ending").is_err(), "--until does not parse the built-in condition");
    let watch = Watch::new(&Physics::shipped().expect("the shipped physics"), 600, 0.0);
    let event = |kind: &'static str| RunEvent { step: 100, sim_time: 5.0, kind, object: 1, detail: serde_json::json!({"mass_sb": 1.0}) };
    for (kind, collapse_or_ending, ending) in [
        ("core_collapse", true, false),
        ("ending", false, false),
        ("ignition", false, false),
        ("supernova", false, false),
        ("sink_formed", false, false),
    ] {
        assert_eq!(watch.holds(Until::CollapseOrEnding, &[event(kind)]), collapse_or_ending, "{kind} under m_up_sb's condition");
        assert_eq!(watch.holds(Until::Ending, &[event(kind)]), ending, "{kind} under the ending's");
    }
    assert!(!watch.holds(Until::CollapseOrEnding, &[]), "a frame with no event and no remnant shown does not stop it");
}

/// M0-D75: a life's side from the record the engine wrote — read back by `--resume`, no GPU. A record of the binary before
/// M0-D75 (run to its ending) sides as it did; one cut at the core collapse (no `ending` event) sides heavy; so does one that
/// shows a neutron star or a black hole without the event; a white dwarf's ending stays light.
#[test]
fn a_record_sides_the_same_whichever_condition_stopped_it() {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-sides-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (lo, hi) = (3351.0, 3380.0);
    let record = |nn: u32, mass: f64, condition: &str, events: &[&str], stages: &[&str], ending: Option<&str>| {
        let mut events: Vec<serde_json::Value> = events.iter().map(|k| serde_json::json!({"step": 9, "sim_time": 1.0, "kind": k, "object": 1, "detail": {}})).collect();
        if let Some(stage) = ending {
            events.push(serde_json::json!({"step": 99, "sim_time": 11.0, "kind": "ending", "object": 1, "detail": {"stage": stage}}));
        }
        let stages: Vec<serde_json::Value> = stages.iter().map(|s| serde_json::json!({"stage": s, "from_step": 5, "sim_time": 0.5})).collect();
        let summary = serde_json::json!({
            "scene": format!("calibrate:m_up_sb:{mass:.3}"), "until": {"condition": condition, "met": true}, "steps": 1000, "sim_time": 50.0, "wall_s": 1.0,
            "adapter": {"name": "test adapter"}, "max_gas_speed": 1.0, "events": events,
            "objects": [{"id": 1, "stage": null, "mass_sb": mass, "stages": stages}],
        });
        let folder = dir.join(format!("m_up_sb-{nn:02}"));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("summary.json"), summary.to_string()).unwrap();
    };
    // Life 01: a white dwarf, light. Life 02 under each way a record ends; every one sides heavy.
    type Heavy = (&'static str, &'static str, &'static [&'static str], &'static [&'static str], Option<&'static str>);
    let heavy: [Heavy; 4] = [
        ("to its ending, before M0-D75", "ending", &["core_collapse", "supernova"], &["neutron_star"], Some("neutron_star")),
        ("at the core collapse", "collapse|ending", &["core_collapse"], &["collapsing"], None),
        ("at a neutron star shown, no event", "collapse|ending", &["supernova"], &["neutron_star"], None),
        ("at a black hole shown, no event", "collapse|ending", &["sink_formed"], &["black_hole"], None),
    ];
    for (what, condition, events, stages, ending) in heavy {
        record(1, lo, "ending", &["ignition"], &["main_sequence", "white_dwarf"], Some("white_dwarf"));
        record(2, hi, condition, events, stages, ending);
        let out = dir.join("calibration.json");
        let args: Vec<String> = ["--out", out.to_str().unwrap(), "--only", "m_up_sb", "--bracket", "3351,3380", "--resume", dir.to_str().unwrap()].map(String::from).to_vec();
        let mut lines = Vec::new();
        let code = run_with(&args, &[], &mut |l| lines.push(l.to_string()));
        assert_eq!(code, 0, "{what}: {lines:?}");
        let runs: Vec<&String> = lines.iter().filter(|l| l.starts_with("SR-CALIBRATE run")).collect();
        assert_eq!(runs.len(), 2, "{what}: {lines:?}");
        assert!(runs[0].contains("side=Light") && runs[0].contains("ending=white_dwarf") && runs[0].contains(" replayed"), "{what}: {}", runs[0]);
        assert!(runs[1].contains("side=Heavy") && runs[1].contains(" replayed"), "{what}: {}", runs[1]);
        assert!(lines.iter().any(|l| l.starts_with("SR-CALIBRATE item=m_up_sb value=")), "{what}: the item closes: {lines:?}");
        let _ = std::fs::remove_file(&out);
    }
    // A record with a collapse remnant that is not a white dwarf, no event and no ending, is on no side: still refused.
    record(1, lo, "ending", &["ignition"], &["main_sequence", "white_dwarf"], Some("white_dwarf"));
    record(2, hi, "collapse|ending", &["supernova"], &["main_sequence"], None);
    let args: Vec<String> = ["--out", dir.join("calibration.json").to_str().unwrap(), "--only", "m_up_sb", "--bracket", "3351,3380", "--resume", dir.to_str().unwrap()].map(String::from).to_vec();
    let mut lines = Vec::new();
    assert_eq!(run_with(&args, &[], &mut |l| lines.push(l.to_string())), 6, "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("neither side of the threshold")), "{lines:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// M0-D95: a life's ending is its heaviest object's (§0.5 [M0-D95]) — read back by `--resume`, no GPU. m_bh_sb's heavy life
/// whose fragment (1,700) ended neutron_star before the star (30,000) ended black_hole sides heavy on the star's ending; the same
/// life stopped on the fragment's ending alone (the binary before M0-D95: any object's) is on no side, refused — never light.
#[test]
fn a_fragments_ending_is_none_of_the_lifes() {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-fragment-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // `objects`: (id, mass, stage); `endings`: (object, stage), in the order they fired.
    let record = |nn: u32, mass: f64, objects: &[(u32, f64, &str)], endings: &[(u32, &str)]| {
        let events: Vec<serde_json::Value> = endings
            .iter()
            .map(|(id, stage)| serde_json::json!({"step": 99, "sim_time": 11.0, "kind": "ending", "object": id, "detail": {"stage": stage}}))
            .collect();
        let objects: Vec<serde_json::Value> = objects
            .iter()
            .map(|(id, m, stage)| serde_json::json!({"id": id, "stage": stage, "mass_sb": m, "stages": [{"stage": stage, "from_step": 5, "sim_time": 0.5}]}))
            .collect();
        let summary = serde_json::json!({
            "scene": format!("calibrate:m_bh_sb:{mass:.3}"), "until": {"condition": "ending", "met": true}, "steps": 1000, "sim_time": 50.0,
            "wall_s": 1.0, "adapter": {"name": "test adapter"}, "max_gas_speed": 1.0, "events": events, "objects": objects,
        });
        let folder = dir.join(format!("m_bh_sb-{nn:02}"));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("summary.json"), summary.to_string()).unwrap();
    };
    let (lo, hi) = (4500.0, 4560.0);
    let resume = || {
        let args: Vec<String> = ["--out", dir.join("calibration.json").to_str().unwrap(), "--only", "m_bh_sb", "--bracket", "4500,4560", "--resume", dir.to_str().unwrap()]
            .map(String::from)
            .to_vec();
        let mut lines = Vec::new();
        let code = run_with(&args, &[], &mut |l| lines.push(l.to_string()));
        (code, lines)
    };
    record(1, lo, &[(1, 2380.0, "neutron_star")], &[(1, "neutron_star")]);
    record(2, hi, &[(1, 30_000.0, "black_hole"), (2, 1700.0, "neutron_star")], &[(2, "neutron_star"), (1, "black_hole")]);
    let (code, lines) = resume();
    assert_eq!(code, 0, "{lines:?}");
    let runs: Vec<&String> = lines.iter().filter(|l| l.starts_with("SR-CALIBRATE run")).collect();
    assert_eq!(runs.len(), 2, "{lines:?}");
    assert!(runs[0].contains("side=Light") && runs[0].contains("ending=neutron_star"), "{}", runs[0]);
    assert!(runs[1].contains("side=Heavy") && runs[1].contains("ending=black_hole"), "the star's ending, not the fragment's: {}", runs[1]);

    record(2, hi, &[(1, 30_000.0, "black_hole"), (2, 1700.0, "neutron_star")], &[(2, "neutron_star")]);
    let (code, lines) = resume();
    assert_eq!(code, 6, "a fragment's ending places no life: {lines:?}");
    assert!(lines.iter().any(|l| l.contains("neither side of the threshold") && l.contains("ending=none")), "{lines:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// M0-D75: `--progress-every N` sets the tick of every life of the invocation — a progress line and a progress.json write each
/// time the step count passes a multiple of N, the file saying N — and items 1–2 (no lives) ignore it. Two real short lives
/// (the nominal cloud and 1.02 × it, ignition ends each) on the test adapter; the default tick, 10,000, shows none in them.
#[test]
fn progress_every_sets_the_lives_tick() {
    let m0 = sr_engine::scene::UNIT_MASS;
    let bracket = format!("{},{}", m0, 1.02 * m0);
    let out_dir = std::env::temp_dir().join(format!("sr-calibrate-tick-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.join("calibration.json");
    let mut args: Vec<String> = ["--out", out.to_str().unwrap(), "--only", "m_ign_sb", "--bracket", bracket.as_str()].map(String::from).to_vec();
    if let Some(adapter) = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty()) {
        args.extend(["--adapter".to_string(), adapter]);
    }
    let progress_steps = |lines: &[String]| -> Vec<u64> {
        lines.iter().filter(|l| l.starts_with("SR-CALIBRATE progress")).map(|l| l.split(" step=").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap()).collect()
    };
    let mut stamps = Vec::new();
    let mut note = |lines: &[String]| {
        stamps.extend(lines.iter().filter_map(|l| l.split("record=").nth(1)).filter_map(|r| std::path::Path::new(r.trim()).parent().map(|p| p.to_path_buf())));
    };

    // The default tick: a few hundred steps say nothing. `--progress-every 16`: one line a frame of 16 steps, a life at a time.
    let mut lines = Vec::new();
    assert_eq!(run_with(&args, &[], &mut |l| lines.push(l.to_string())), 6, "the ends agree: {lines:?}");
    note(&lines);
    assert!(progress_steps(&lines).is_empty(), "the default tick is 10,000 steps: {lines:?}");
    let mut sixteen = args.clone();
    sixteen.extend(["--progress-every".to_string(), "16".to_string()]);
    let mut lines = Vec::new();
    assert_eq!(run_with(&sixteen, &[], &mut |l| lines.push(l.to_string())), 6, "{lines:?}");
    note(&lines);
    let first_life: Vec<u64> = progress_steps(&lines[..lines.iter().position(|l| l.starts_with("SR-CALIBRATE run")).unwrap()]);
    assert!(first_life.len() >= 3, "a life of a few hundred steps shows a line every 16: {lines:?}");
    assert!(first_life.windows(2).all(|w| w[1] - w[0] == 16), "a line every 16 steps: {first_life:?}");

    // A life that fails keeps its progress.json, which names the tick it was written at.
    let mut failing = sixteen.clone();
    failing.extend(["--max-steps".to_string(), "64".to_string()]);
    let mut lines = Vec::new();
    assert_eq!(run_with(&failing, &[], &mut |l| lines.push(l.to_string())), 6, "{lines:?}");
    let error = lines.iter().find(|l| l.starts_with("SR-ERROR")).unwrap_or_else(|| panic!("{lines:?}"));
    let record = std::path::PathBuf::from(error.split("record kept: ").nth(1).unwrap_or_else(|| panic!("{error}")).trim_end_matches(')'));
    stamps.extend(record.parent().map(|p| p.to_path_buf()));
    let progress: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(record.join("progress.json")).unwrap()).unwrap();
    assert_eq!(progress["progress_every"], 16, "{progress}");
    assert!(progress["step"].as_u64().is_some_and(|s| (17..=64).contains(&s)), "the last tick is one the 64-step life reached: {progress}");

    // Items 1–2 run no life and ignore the flag.
    let sheet: Vec<String> = ["--out", out_dir.join("sheet.json").to_str().unwrap(), "--only", "m_ch_sb", "--progress-every", "16"].map(String::from).to_vec();
    let mut lines = Vec::new();
    assert_eq!(run_with(&sheet, &[], &mut |l| lines.push(l.to_string())), 0, "{lines:?}");
    assert!(progress_steps(&lines).is_empty(), "{lines:?}");
    for stamp in stamps {
        let _ = std::fs::remove_dir_all(stamp);
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// M0-D75: a life that shows a black hole without the `core_collapse` event is over for m_up_sb the frame it shows it — the run
/// stops there, `until.met`, with no `ending` (the stage had not held its max(10 t.u., 20 τ_dyn)) — where `Until::Ending` runs on
/// to the hold. The observe scope's seeded cloud (a sink of 50 in the sink_cloud scene at T 0.5), under `calibrate`'s own
/// condition.
#[test]
fn a_life_showing_a_black_hole_stops_before_its_ending() {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-collapse-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let scenes = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes");
    let mut cloud: serde_json::Value = serde_json::from_slice(&std::fs::read(scenes.join("sink_cloud.json")).unwrap()).unwrap();
    cloud["objects"][0]["t"] = 0.5.into();
    let scene = dir.join("sink_cloud.json");
    std::fs::write(&scene, cloud.to_string()).unwrap();
    let adapter = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty());

    let seed_out = dir.join("seed");
    let mut seed: Vec<String> = ["--scene", scene.to_str().unwrap(), "--steps", "40", "--out", seed_out.to_str().unwrap()].map(String::from).to_vec();
    seed.extend(adapter.iter().flat_map(|a| ["--adapter".to_string(), a.clone()]));
    let (code, lines) = run_captured(&seed);
    assert_eq!(code, 0, "{lines:?}");
    let mut dump = sr_engine::dump::Dump::read(&seed_out.join("dumps").join("s_40.bin")).expect("the seed run's dump");
    let c = (dump.world.width / 2, dump.world.height / 2);
    dump.sinks = vec![sr_engine::dump::Sink { x: f64::from(c.0) + 0.5, y: f64::from(c.1) + 0.5, vx: 0.0, vy: 0.0, m: 50.0 }];
    let seeded = dir.join("seeded.bin");
    dump.write(&seeded).unwrap();

    let go = |name: &str, until: Until| {
        let out = dir.join(name);
        let args = Args {
            scene: format!("calibrate:m_up_sb:{name}"),
            steps: None,
            until: Some(until),
            world: None,
            edge: None,
            max_steps: 20_000,
            out: Some(out.clone()),
            adapter: adapter.clone(),
            timing: false,
            dump_every: None,
            load_dump: Some(seeded.clone()),
            continue_life: false,
            cpu_reference: false,
            edits: None,
        };
        let (code, lines) = run_scene_captured(args, Scene::load(&scene).unwrap(), None);
        assert_eq!(code, 0, "{name}: {lines:?}");
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(out.join("summary.json")).unwrap()).unwrap()
    };
    let stopped = go("collapse", Until::CollapseOrEnding);
    assert_eq!(stopped["until"], serde_json::json!({"condition": "collapse|ending", "met": true}), "{}", stopped["until"]);
    let kinds: Vec<&str> = stopped["events"].as_array().unwrap().iter().map(|e| e["kind"].as_str().unwrap()).collect();
    assert!(!kinds.contains(&"ending") && !kinds.contains(&"core_collapse"), "no ending, no collapse event: the black hole's showing stopped it: {kinds:?}");
    let shows_black_hole = stopped["objects"].as_array().unwrap().iter().any(|o| o["stages"].as_array().unwrap().iter().any(|s| s["stage"] == "black_hole"));
    assert!(shows_black_hole, "an object shows black_hole: {}", stopped["objects"]);
    let waited = go("ending", Until::Ending);
    assert!(waited["steps"].as_u64() > stopped["steps"].as_u64(), "the ending's run goes on past it: {} against {}", waited["steps"], stopped["steps"]);
    assert!(waited["events"].as_array().unwrap().iter().any(|e| e["kind"] == "ending"), "{}", waited["events"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Copies the folder `from` and what it holds into `to` (a test's scratch; no links).
fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// M0-D77: `--checkpoint-every M` is the save interval, a flag of its own beside `--progress-every N` (the display interval), and a
/// life cut part-way is continued by `--resume` from its last checkpoint instead of from step 0. Two real short lives (the
/// nominal cloud and 1.02 × it, ignition ends each): the unbroken invocation; the same cut at `--max-steps 130` with the two
/// intervals set apart (progress lines every frame of 16 steps, a checkpoint every 48) — the folder holds the last whole pair and
/// progress.json says where it is; `--resume` continues the first life from it and writes the record of the unbroken life
/// (`wall_s` aside); a checkpoint of another mass, of other physics, a snapshot of another step beside its dump, a torn dump
/// and a dump with no snapshot are not used and the life runs from step 0 saying why; a torn newer pair leaves the older one.
#[test]
fn a_cut_life_resumes_at_its_last_checkpoint() {
    let m0 = sr_engine::scene::UNIT_MASS;
    let bracket = format!("{},{}", m0, 1.02 * m0);
    let out_dir = std::env::temp_dir().join(format!("sr-calibrate-checkpoint-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.join("calibration.json");
    let mut args: Vec<String> = ["--out", out.to_str().unwrap(), "--only", "m_ign_sb", "--bracket", bracket.as_str()].map(String::from).to_vec();
    let adapter = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty());
    if let Some(adapter) = &adapter {
        args.extend(["--adapter".to_string(), adapter.clone()]);
    }
    let with = |extra: &[&str]| -> Vec<String> { args.iter().cloned().chain(extra.iter().map(|s| s.to_string())).collect() };
    let go = |args: &[String]| {
        let mut lines = Vec::new();
        let code = run_with_progress(args, &[], 10_000, &mut |l| lines.push(l.to_string()));
        (code, lines)
    };
    let runs_of = |lines: &[String]| lines.iter().filter(|l| l.starts_with("SR-CALIBRATE run")).cloned().collect::<Vec<_>>();
    // A run line up to its wall time: what the unbroken invocation said of the same life.
    let same_life = |line: &str| line.split(" wall_s=").next().unwrap().to_string();
    let record_of = |line: &str| std::path::PathBuf::from(line.split("record=").nth(1).unwrap().trim());
    let error_of = |lines: &[String]| lines.iter().find(|l| l.starts_with("SR-ERROR")).cloned().unwrap_or_default();
    let steps_of = |lines: &[String]| -> Vec<u64> {
        lines.iter().filter(|l| l.starts_with("SR-CALIBRATE progress")).map(|l| l.split(" step=").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap()).collect()
    };
    let json_of = |path: std::path::PathBuf| -> serde_json::Value { serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap() };
    // A summary.json without the wall time, which a continued life sums over its segments.
    let unwalled = |mut v: serde_json::Value| {
        v.as_object_mut().unwrap().remove("wall_s");
        v
    };
    let mut stamps = Vec::new();

    // The unbroken invocation, the defaults: two lives, the ends agree (exit 6), no checkpoint left behind.
    let (code, unbroken) = go(&args);
    assert_eq!(code, 6, "{unbroken:?}");
    let first = runs_of(&unbroken);
    assert_eq!(first.len(), 2, "{unbroken:?}");
    let (rec1, rec2) = (record_of(&first[0]), record_of(&first[1]));
    stamps.extend(rec1.parent().map(|p| p.to_path_buf()));
    let unbroken_summary = json_of(rec1.join("summary.json"));
    let steps_1 = unbroken_summary["steps"].as_u64().unwrap();
    assert!(steps_1 > 150, "the nominal cloud's life must outlast the cut at 130 steps: {steps_1}");
    assert!(!rec1.join("checkpoint").exists() && !rec2.join("checkpoint").exists(), "a finished life leaves no checkpoint");

    // Cut at 130 steps, progress every 16 and a checkpoint every 48 (frames end at 1 + 16k: the saves come at 49 and 97).
    let (code, cut) = go(&with(&["--progress-every", "16", "--checkpoint-every", "48", "--max-steps", "130"]));
    assert_eq!(code, 6, "{cut:?}");
    let life_dir = record_of(&format!("record={}", error_of(&cut).split("record kept: ").nth(1).unwrap_or_else(|| panic!("{cut:?}")).trim_end_matches(')')));
    let cut_dir = life_dir.parent().unwrap().to_path_buf();
    stamps.push(cut_dir.clone());
    let listing = |dir: &std::path::Path| -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        names
    };
    assert_eq!(listing(&life_dir.join("checkpoint")), ["s_97.bin", "s_97.observers.json"], "one whole pair, the last save's");
    let ticks = steps_of(&cut);
    assert!(ticks.first() == Some(&17) && ticks.last() == Some(&129) && ticks.windows(2).all(|w| w[1] - w[0] == 16), "progress lines at their own 16: {ticks:?}");
    let note = json_of(life_dir.join("progress.json"));
    assert_eq!((note["step"].as_u64(), note["checkpoint_step"].as_u64(), note["continued_from_step"].is_null()), (Some(129), Some(97), true), "{note}");
    let snapshot = json_of(life_dir.join("checkpoint").join("s_97.observers.json"));
    assert!(snapshot["physics_hash"].as_str().is_some_and(|h| h.len() == 64) && snapshot["adapter"].is_string(), "{snapshot}");

    // Not used: each runs from step 0 on its own copy of the cut invocation, and says why.
    let reasons: Vec<(&str, Vec<String>)> = vec![
        ("another mass", with(&["--bracket", &format!("{},{}", 0.9 * m0, 1.02 * m0)])),
        ("physics_hash", with(&[])),
        ("version 1", with(&[])),
        ("another step", with(&[])),
        ("torn dump", with(&[])),
        ("no snapshot", with(&[])),
        ("another adapter", with(&["--adapter", "no such adapter"])),
    ];
    for (what, base) in reasons {
        let copy = out_dir.join(format!("copy-{}", what.replace(' ', "-")));
        copy_tree(&cut_dir, &copy);
        let checkpoint = copy.join("m_ign_sb-01").join("checkpoint");
        let snapshot_path = checkpoint.join("s_97.observers.json");
        let edit = |key: &str, value: serde_json::Value| {
            let mut v = json_of(snapshot_path.clone());
            v[key] = value;
            std::fs::write(&snapshot_path, v.to_string()).unwrap();
        };
        let needle = match what {
            "physics_hash" => {
                edit("physics_hash", "0".repeat(64).into());
                "physics_hash"
            }
            // A snapshot of M0-D77's binary, before each object's history carried `x_h_max` (§0.5 [M0-D78]).
            "version 1" => {
                edit("version", 1.into());
                "is version 1, this build reads 2"
            }
            "another step" => {
                edit("step", 81.into());
                "holds step 81, beside a dump of step 97"
            }
            "torn dump" => {
                let bin = checkpoint.join("s_97.bin");
                let bytes = std::fs::read(&bin).unwrap();
                std::fs::write(&bin, &bytes[..bytes.len() / 2]).unwrap();
                "s_97.bin"
            }
            "no snapshot" => {
                std::fs::remove_file(&snapshot_path).unwrap();
                "no snapshot beside it"
            }
            "another adapter" => "--adapter names",
            _ => "this life is",
        };
        // (for "another adapter" the later --adapter wins over the test adapter's)
        let argv = [base, vec!["--resume".to_string(), copy.to_str().unwrap().to_string(), "--progress-every".to_string(), "16".to_string(), "--max-steps".to_string(), "130".to_string()]].concat();
        let (code, lines) = go(&argv);
        let warn = lines.iter().find(|l| l.starts_with("SR-WARN checkpoint not used")).unwrap_or_else(|| panic!("{what}: a checkpoint not used says so: {lines:?}"));
        assert!(warn.contains(needle), "{what}: should say {needle:?}: {warn}");
        assert!(lines.iter().all(|l| !l.starts_with("SR-CALIBRATE continue")), "{what}: {lines:?}");
        if what == "another adapter" {
            assert_ne!(code, 0, "{what}: no such adapter opens: {lines:?}");
        } else {
            assert_eq!(code, 6, "{what}: the life ran fresh into the cut again: {lines:?}");
            assert_eq!(steps_of(&lines).first(), Some(&17), "{what}: from step 0: {lines:?}");
        }
        let _ = std::fs::remove_dir_all(&copy);
    }

    // A torn newer pair (its dump half-written, no snapshot yet) leaves the older whole one: the life continues from it.
    let torn = out_dir.join("copy-torn-newer");
    copy_tree(&cut_dir, &torn);
    let checkpoint = torn.join("m_ign_sb-01").join("checkpoint");
    let bytes = std::fs::read(checkpoint.join("s_97.bin")).unwrap();
    std::fs::write(checkpoint.join("s_113.bin"), &bytes[..bytes.len() / 3]).unwrap();
    let (code, lines) = go(&with(&["--resume", torn.to_str().unwrap()]));
    assert_eq!(code, 6, "{lines:?}");
    assert!(runs_of(&lines)[0].contains(" continued from step 97"), "{lines:?}");
    let _ = std::fs::remove_dir_all(&torn);

    // A continued life cut again: progress.json says where this process began and the checkpoint it saved, and the stage trail
    // of the cut run carries over.
    let again = out_dir.join("copy-cut-again");
    copy_tree(&cut_dir, &again);
    let (code, lines) = go(&with(&["--resume", again.to_str().unwrap(), "--progress-every", "16", "--checkpoint-every", "16", "--max-steps", "125"]));
    assert_eq!(code, 6, "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-CALIBRATE continue item=m_ign_sb") && l.contains("from step 97")), "{lines:?}");
    assert_eq!(steps_of(&lines), [113], "no line at the step it continued from: {lines:?}");
    let note = json_of(again.join("m_ign_sb-01").join("progress.json"));
    assert_eq!((note["continued_from_step"].as_u64(), note["checkpoint_step"].as_u64(), note["step"].as_u64()), (Some(97), Some(113), Some(113)), "{note}");
    let trail = |v: &serde_json::Value| v["stages"].as_array().unwrap().iter().map(|s| (s["stage"].as_str().unwrap().to_string(), s["since_step"].as_u64().unwrap())).collect::<Vec<_>>();
    let before = trail(&json_of(life_dir.join("progress.json")));
    let after = trail(&note);
    assert!(!before.is_empty() && after.starts_with(&before.iter().filter(|(_, at)| *at <= 97).cloned().collect::<Vec<_>>()), "the cut run's trail goes on: {before:?} then {after:?}");
    let _ = std::fs::remove_dir_all(&again);

    // `--resume`: the first life continues from step 97 and is the unbroken life's — its summary.json but for `wall_s`, its
    // events.jsonl byte for byte — the second runs fresh, the invocation says what the unbroken one did.
    let (code, lines) = go(&with(&["--resume", cut_dir.to_str().unwrap(), "--progress-every", "16"]));
    assert_eq!(code, 6, "{lines:?}");
    let runs = runs_of(&lines);
    assert_eq!(runs.len(), 2, "{lines:?}");
    assert!(runs[0].contains(" continued from step 97") && !runs[1].contains("continued"), "{runs:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-CALIBRATE continue item=m_ign_sb") && l.contains("from step 97")), "{lines:?}");
    assert_eq!(runs.iter().map(|l| same_life(l)).collect::<Vec<_>>(), first.iter().map(|l| same_life(l)).collect::<Vec<_>>(), "the run sequence is the unbroken one's");
    assert_eq!(record_of(&runs[0]), life_dir);
    let continued_summary = json_of(life_dir.join("summary.json"));
    assert_eq!(unwalled(continued_summary.clone()), unwalled(unbroken_summary.clone()), "the continued life is the unbroken life's");
    assert_eq!(std::fs::read(life_dir.join("events.jsonl")).unwrap(), std::fs::read(rec1.join("events.jsonl")).unwrap());
    assert!(continued_summary["wall_s"].as_f64().unwrap() > 0.0);
    assert_eq!(steps_of(&lines).first(), Some(&113), "no line at or before the checkpoint's step: {lines:?}");
    assert!(!life_dir.join("checkpoint").exists() && !life_dir.join("progress.json").exists(), "a finished life leaves no checkpoint and no progress.json");
    for stamp in stamps {
        let _ = std::fs::remove_dir_all(stamp);
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// M0-D77: a continued life whose restored state already meets its `until` ends at once — the condition is read on a frame's
/// fresh events, and an event fired before the cut never fires again. The nominal cloud run to its ignition; a second run that
/// continues from the dump it left (the same label and `until`) stops where it started, `until.met`, not at `--max-steps`.
#[test]
fn a_continued_life_that_already_met_its_until_ends_at_once() {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-ends-at-once-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let adapter = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty());
    let scene = || Scene::preset_mass("sun", sr_engine::scene::UNIT_MASS, sr_engine::state::WorldConfig::default()).unwrap();
    let go = |name: &str, load: Option<std::path::PathBuf>| {
        let out = dir.join(name);
        let args = Args {
            scene: "calibrate:ends_at_once:m0".to_string(),
            steps: None,
            until: Some(Until::parse("event:ignition").unwrap()),
            world: None,
            edge: None,
            max_steps: 20_000,
            out: Some(out.clone()),
            adapter: adapter.clone(),
            timing: false,
            dump_every: None,
            continue_life: load.is_some(),
            load_dump: load,
            cpu_reference: false,
            edits: None,
        };
        let (code, lines) = run_scene_captured(args, scene(), None);
        (code, lines, out)
    };
    let (code, lines, first) = go("first", None);
    assert_eq!(code, 0, "{lines:?}");
    let summary: serde_json::Value = serde_json::from_slice(&std::fs::read(first.join("summary.json")).unwrap()).unwrap();
    let steps = summary["steps"].as_u64().unwrap();
    let dump = first.join("dumps").join(format!("s_{steps}.bin"));
    let (code, lines, second) = go("second", Some(dump));
    assert_eq!(code, 0, "the restored life met its until already: {lines:?}");
    let again: serde_json::Value = serde_json::from_slice(&std::fs::read(second.join("summary.json")).unwrap()).unwrap();
    assert_eq!((again["steps"].as_u64(), &again["until"]["met"]), (Some(steps), &serde_json::json!(true)), "{again}");
    assert_eq!(again["events"], summary["events"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// M0-D80: the trigger of one-step frames reads the state alone — a step at P5's ceiling in the frame just run, or the heaviest
/// object's raw stage core_collapse — never wall time.
#[test]
fn one_step_frames_follow_the_physics() {
    use sr_engine::headless::one_step_next;
    assert!(one_step_next(1, Some("cloud")), "a step at P5's ceiling");
    assert!(one_step_next(0, Some("core_collapse")), "a core collapse");
    assert!(one_step_next(3, None), "a ceiling with no object yet");
    for raw in [None, Some("cloud"), Some("supernova"), Some("neutron_star"), Some("black_hole")] {
        assert!(!one_step_next(0, raw), "{raw:?}: frames as before");
    }
    // The terminal's colours (M0-D80): the same text once the codes are stripped, hard physics in red, a one-step frame in yellow.
    use sr_engine::calibrate::paint;
    let strip = |s: &str| {
        let mut out = String::new();
        let mut it = s.chars();
        while let Some(c) = it.next() {
            if c == '\x1b' {
                it.by_ref().find(|&c| c == 'm');
            } else {
                out.push(c);
            }
        }
        out
    };
    let head = "SR-CALIBRATE progress item=m_bh_sb mass=6702.064 step=1010 sim_time=10.469 heat_unconverged=6 stage=white_dwarf | raw=core_collapse";
    // Rows as `tick_rows` lays them out: a label in 6, then cells of a name in 7 and a value right-aligned in 9.
    let row = |label: &str, cells: [(&str, &str); 4]| format!("  {label:<6} {}", cells.map(|(n, v)| format!("{n:<7} {v:>9}")).join(" "));
    let rows = [
        row("frame", [("steps", "1"), ("dt", "8.714e-3"), ("s/step", "0.05189"), ("P5_ceil", "2")]),
        row("light", [("L", "42033"), ("L_nuc", "1.802e-11"), ("L_N", "-3.100"), ("L_nu", "0")]),
    ];
    for line in [head, rows[0].as_str(), rows[1].as_str(), "SR-WARN something", "SR-CALIBRATE run item=m_bh_sb side=Light", "plain words"] {
        assert_eq!(strip(&paint(line)), line, "paint keeps the text");
    }
    assert!(paint(head).contains("\x1b[1;31mcore_collapse"), "{:?}", paint(head));
    let value = |tone: &str, v: &str| format!("\x1b[{tone}m {v:>9}");
    assert!(paint(&rows[0]).contains(&value("1;33", "1")) && paint(&rows[0]).contains(&value("1;31", "2")), "{:?}", paint(&rows[0]));
    assert!(paint(&rows[1]).contains(&value("1;31", "-3.100")) && paint(&rows[1]).contains(&value("1", "0")), "{:?}", paint(&rows[1]));
    assert_eq!(paint("plain words"), "plain words");
}

/// M0-D80: a life under a progress hook runs one step a frame through its core collapse, a progress tick each, and goes back to
/// frames once the collapse stops. A 16 × 16 iron block at Σ 400 (Y_e Σ 186 over Σ_N 160), gravity on and nothing moving, in a
/// 64 × 64 world: N_Fe converts it from the first step (raw core_collapse) until the gate closes some 22 steps later (§0.5 [M0-R10]'s rate, M0-D87;
/// ~60 on the capped sub-cycles before it). The same
/// life twice is the same; one cut at a checkpoint inside the stretch and continued is the unbroken one (summary.json but for
/// `wall_s`, events.jsonl and the ticks after the cut; the cut run is the second life, its ticks the unbroken one's). Without a hook the frames are as before: scopes `observe_run` and `dump`
/// run every life there through plain `headless` and grade its frame boundaries. Each collapse step cost 0.6–3 s before M0-D87 (RTX 4090, ~all in
/// P6's N_Fe); in closed form it costs what any step does.
#[test]
fn a_life_steps_one_at_a_time_through_its_core_collapse() {
    use sr_engine::headless::{Progress, Tick};
    let dir = std::env::temp_dir().join(format!("sr-calibrate-one-step-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let scene_file = dir.join("iron.json");
    let scene = serde_json::json!({"version": 1, "world": {"width": 64, "height": 64, "edge": "bounce"}, "slowdown": false,
        "switches": {"gravity": true, "hydro": false, "heat": false, "reactions": true, "neutrinos": false, "sinks": false},
        "objects": [{"kind": "rect", "x0": 24, "y0": 24, "w": 16, "h": 16, "sigma": 400.0, "t": 1.0, "composition": {"Fe": 1.0}, "v": [0, 0]}]});
    std::fs::write(&scene_file, scene.to_string()).unwrap();
    let adapter = std::env::var("SR_TEST_ADAPTER").ok().filter(|a| !a.trim().is_empty());
    #[derive(Debug, PartialEq)]
    struct Seen {
        step: u64,
        frame: u64,
        p5_ceiling: u64,
        raw: String,
    }
    // A life to `max_steps` (its until never met: exit 2), from `load` when given, ticking every 64 steps, a checkpoint every
    // `checkpoint` steps.
    let life = |name: &str, max_steps: u64, load: Option<std::path::PathBuf>, checkpoint: Option<u64>| {
        let out = dir.join(name);
        let args = Args {
            scene: "calibrate:one_step:iron".to_string(),
            steps: None,
            until: Some(Until::parse("stage:black_hole").unwrap()),
            world: None,
            edge: None,
            max_steps,
            out: Some(out.clone()),
            adapter: adapter.clone(),
            timing: false,
            dump_every: None,
            continue_life: load.is_some(),
            load_dump: load,
            cpu_reference: false,
            edits: None,
        };
        let mut seen = Vec::new();
        let mut tick = |t: &Tick| {
            let star = t.star.as_ref().expect("the block is an object");
            for x in [t.dt, t.s_per_step, t.escaped, star.mass, star.radius, star.tau_dyn, star.sigma_c, star.t_c, star.d_c, star.ye_c, star.v_r_dyn, star.l_n] {
                assert!(x.is_finite(), "step {}: {x}", t.step);
            }
            assert_eq!(t.objects, 1, "step {}", t.step);
            seen.push(Seen { step: t.step, frame: t.frame, p5_ceiling: t.p5_ceiling, raw: star.raw.unwrap_or("none").to_string() });
        };
        let mut saved = |_: u64, _: f64, _: &str| {};
        let progress = Progress { every: 64, tick: &mut tick, checkpoint_every: checkpoint, saved: &mut saved };
        let (code, lines) = run_scene_captured(args, Scene::load(&scene_file).unwrap(), Some(progress));
        assert_eq!(code, 2, "{name}: {lines:?}");
        let mut summary: serde_json::Value = serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
        summary.as_object_mut().unwrap().remove("wall_s");
        (seen, summary, std::fs::read(out.join("events.jsonl")).unwrap(), out)
    };

    // Engaged from the first step's collapse: ticks at steps 2, 3, … each a frame of one step; the stretch's last frame shows
    // the collapse over, and frames of 16 follow, ticking only on the 64-step grid.
    let (seen, summary, events, _) = life("unbroken", 160, None, None);
    let stretch = seen.iter().take_while(|s| s.frame == 1).count();
    // 22 measured (M0-D87, RTX 4090); the cut below at 20 must fall inside it.
    assert!(stretch >= 20, "a stretch of one-step frames through the collapse: {seen:?}");
    assert!(seen[..stretch].iter().enumerate().all(|(i, s)| s.step == 2 + i as u64), "one tick a step from step 2: {seen:?}");
    assert!(seen[..stretch - 1].iter().all(|s| s.raw == "core_collapse"), "{seen:?}");
    assert_ne!(seen[stretch - 1].raw, "core_collapse", "the stretch ends when the collapse does: {seen:?}");
    let after = &seen[stretch..];
    assert!(!after.is_empty() && after.iter().all(|s| s.frame == 16), "frames again after the stretch: {seen:?}");
    assert!(after.windows(2).all(|w| w[1].step / 64 > w[0].step / 64), "only the 64-step grid ticks after it: {after:?}");
    assert!(seen.iter().all(|s| s.p5_ceiling == 0), "no heat pass in this scene: {seen:?}");
    assert_eq!(summary["steps"], 160, "{summary}");

    // The same life again, cut at 30 with a checkpoint every 20 (a one-step frame ends at 20 exactly): its ticks and events
    // through 30 are the unbroken life's.
    let (again, summary_2, _, cut) = life("cut", 30, None, Some(20));
    let first_30: Vec<&Seen> = seen.iter().filter(|s| s.step <= 30).collect();
    assert_eq!(again.iter().collect::<Vec<_>>(), first_30, "the same life, the same ticks");
    let early = |v: &serde_json::Value| v["events"].as_array().unwrap().iter().filter(|e| e["step"].as_u64() <= Some(30)).cloned().collect::<Vec<_>>();
    assert_eq!(early(&summary_2), early(&summary), "the same events");

    // Continued from the checkpoint at 20: the unbroken life.
    let pair = cut.join("checkpoint").join("s_20.observers.json");
    let snapshot: serde_json::Value = serde_json::from_slice(&std::fs::read(&pair).unwrap_or_else(|e| panic!("{}: {e}", pair.display()))).unwrap();
    assert_eq!(snapshot["one_step"], true, "the snapshot carries the stretch");
    let (continued, summary_c, events_c, _) = life("continued", 160, Some(cut.join("checkpoint").join("s_20.bin")), None);
    let unbroken_after: Vec<&Seen> = seen.iter().filter(|s| s.step > 20).collect();
    assert_eq!(continued.iter().collect::<Vec<_>>(), unbroken_after, "the continued life frames as the unbroken one");
    let mut unbroken = summary.clone();
    unbroken.as_object_mut().unwrap().remove("adapter");
    let mut summary_c = summary_c;
    summary_c.as_object_mut().unwrap().remove("adapter");
    assert_eq!((summary_c, events_c), (unbroken, events), "the continued life's record is the unbroken one's");
    let _ = std::fs::remove_dir_all(&dir);
}
