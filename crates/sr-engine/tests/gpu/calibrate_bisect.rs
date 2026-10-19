//! M0-T68 — `calibrate` items 3–4: the bisection driver and the items' sides (contract §2.11 items 3–4, §3.3, §6.4). Every
//! test here but the last runs no life — every run is a stub — and the module lives in the one test binary (R11) so the scope
//! runs through `tests/gpu.sh`. The real lives are the lead's, in M0-V9's window; the last test (M0-D60) runs two short real
//! ones, a few hundred steps each, on the test adapter (`SR_TEST_ADAPTER`), to see what a life leaves behind.
//!
//! The driver is graded on stubbed outcomes: a threshold at a known mass, the runs it spends and the masses it asks for. The
//! items' mapping from a life to a side is graded through [`measure`], the same call `calibrate` makes with a real run.
//! Plant: `bisect-tolerance` (the driver stops at 20 %).

use sr_engine::calibrate::{bisect, measure, run_with, run_with_progress, side_bh, side_ign, side_up, CalError, Life, Side, Threshold, BISECT_TOLERANCE};

/// A stub that is Light under `threshold` and Heavy from it, recording every mass asked for.
fn stub(threshold: f64, asked: &mut Vec<f64>) -> impl FnMut(f64) -> Result<Side, CalError> + '_ {
    move |mass| {
        asked.push(mass);
        Ok(if mass < threshold { Side::Light } else { Side::Heavy })
    }
}

fn life(mass: f64, ignited: bool, core_collapse: bool, ending: Option<&str>) -> Life {
    Life { mass, ignited, core_collapse, ending: ending.map(str::to_string), steps: 1000, sim_time: 50.0, wall_s: 1.0 }
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
    for stamp in stamps {
        let _ = std::fs::remove_dir_all(stamp);
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}
