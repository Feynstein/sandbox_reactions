//! M0-T66b — `sandbox-reactions calibrate`, items 1–2: the cold ceilings from the sheet (contract §2.11 items 1–2 and 5
//! [M0-R4], §2.6, §2.7, §3.3). No test here touches a GPU — the command opens no adapter — and the module lives in the one test
//! binary (R11) so the scope runs through `tests/gpu.sh`.
//!
//! The command is driven through `calibrate::run_with` (the binary's `run` is the same call with no overrides) and read from
//! the file it writes, never from its own printout. The expected ceilings are literals from reports/cold_ceiling.md (3,216.5
//! and 4,853.2, the sheet's), and the sheet is run again here on the test's own composition for the exact-equality case.
//! Plant: `calibrate-cold-matter` (item 1 computed on neutron matter's composition: K9's band reads 1.0).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sr_engine::calibrate::{cold_disk_scene, radius_holding, run_with, Matter};
use sr_physics::calibration::physics_hash;
use sr_physics::eos::Eos;
use sr_physics::reference::sheet::{self, ColdMatter};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const PHYSICS: &str = include_str!("../../../../assets/physics.json");

/// reports/cold_ceiling.md M3: the sheet's ceilings for the shipped K's, C/O and neutron matter.
const M_CH: f64 = 3216.5;
const M_TOV: f64 = 4853.2;

/// A fresh directory for one case's files.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-cold-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn calibrate(args: &[&str], overrides: &[(&str, f64)]) -> (i32, Vec<String>) {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let overrides: Vec<(String, f64)> = overrides.iter().map(|&(k, x)| (k.to_string(), x)).collect();
    let mut lines = Vec::new();
    let code = run_with(&args, &overrides, &mut |line| lines.push(line.to_string()));
    for line in &lines {
        println!("{line}");
    }
    (code, lines)
}

fn read(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("the calibration file")).expect("the file is JSON")
}

fn number(file: &Value, key: &str) -> f64 {
    file[key].as_f64().unwrap_or_else(|| panic!("{key} is not a number in {file}"))
}

fn near(value: f64, expected: f64, share: f64, what: &str) {
    assert!((value / expected - 1.0).abs() <= share, "{what}: {value} is not within {share} of {expected}");
}

/// A base file under this build's physics hash, as an earlier `calibrate` run would have left it.
fn write_base(path: &Path, items: Value) {
    let mut file = items;
    file["version"] = json!(1);
    file["physics_hash"] = json!(physics_hash().expect("the shipped assets are JSON"));
    std::fs::write(path, serde_json::to_string_pretty(&file).unwrap()).unwrap();
}

#[test]
fn the_values_written_are_the_sheets_limits() {
    let dir = scratch("limits");
    let out = dir.join("calibration.json");
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb,m_tov_sb"], &[]);
    assert_eq!(code, 0, "{lines:?}");
    assert!(lines.iter().all(|l| !l.starts_with("SR-ADAPTER")), "items 1–2 open no adapter: {lines:?}");
    assert!(lines.last().is_some_and(|l| l.starts_with("SR-CALIBRATE DONE")), "{lines:?}");
    let file = read(&out);

    // The sheet again, on this test's own composition (C 0.5 + O 0.5, X_n = 1) through the registry.
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let eos = Eos::from_physics(&physics).unwrap();
    let matter = |pairs: &[(&str, f64)]| {
        let mut x = [0.0; N_SPECIES];
        for &(key, share) in pairs {
            x[elements.index_of(key).unwrap()] = share;
        }
        let c = elements.composition(&x);
        ColdMatter { y_e: c.y_e, x_n: x[elements.index_of("n").unwrap()], inv_mu: c.inv_mu }
    };
    let vac = physics.get("sigma_vac").unwrap();
    let co = sheet::ceiling(&eos, matter(&[("C", 0.5), ("O", 0.5)]), vac).unwrap().limit;
    let n = sheet::ceiling(&eos, matter(&[("n", 1.0)]), vac).unwrap().limit;
    assert_eq!(number(&file, "m_ch_sb"), co, "m_ch_sb is the sheet's C/O limit");
    assert_eq!(number(&file, "m_tov_sb"), n, "m_tov_sb is the sheet's neutron-matter limit");
    assert_eq!(number(&file, "ns_sigma_max_sb"), n, "ns_sigma_max_sb is m_tov_sb per cell area (a cell's area is 1)");
    near(co, M_CH, 0.005, "m_ch_sb against reports/cold_ceiling.md");
    near(n, M_TOV, 0.005, "m_tov_sb against reports/cold_ceiling.md");

    // The file's frame (§2.11): version, hash, measured — and nothing of the items not built yet.
    assert_eq!(file["version"], json!(1));
    assert_eq!(file["physics_hash"], json!(physics_hash().unwrap()));
    assert_eq!(file["measured"]["adapter"], json!("cpu"), "no GPU item measured");
    let date = file["measured"]["date"].as_str().unwrap();
    assert!(date.len() == 20 && date.ends_with('Z') && date.as_bytes()[10] == b'T', "date {date:?}");
    assert!(file["measured"]["git"].as_str().is_some_and(|g| !g.is_empty()));
    assert!(file["measured"]["box"].as_str().is_some_and(|b| !b.is_empty()));
    let mut keys: Vec<&str> = file.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["m_ch_sb", "m_tov_sb", "measured", "ns_sigma_max_sb", "physics_hash", "version"]);
}

#[test]
fn only_re_measures_one_item_and_keeps_the_rest() {
    let dir = scratch("only");
    let out = dir.join("calibration.json");
    // m_ch_sb is a stale value; the others are not the sheet's and not this run's to touch.
    write_base(&out, json!({"m_ch_sb": 1.0, "m_tov_sb": 4800.0, "ns_sigma_max_sb": 4800.0, "m_ign_sb": 111.0}));
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb"], &[]);
    assert_eq!(code, 0, "{lines:?}");
    let file = read(&out);
    near(number(&file, "m_ch_sb"), M_CH, 0.005, "m_ch_sb re-measured");
    assert_eq!(number(&file, "m_tov_sb"), 4800.0, "m_tov_sb kept");
    assert_eq!(number(&file, "ns_sigma_max_sb"), 4800.0, "ns_sigma_max_sb kept");
    assert_eq!(number(&file, "m_ign_sb"), 111.0, "an item of a later task kept");
    assert!(lines.iter().filter(|l| l.starts_with("SR-CALIBRATE item=")).count() == 1, "one item measured: {lines:?}");
}

#[test]
fn a_file_under_another_hash_is_dropped_with_sr_warn() {
    let dir = scratch("hash");
    let out = dir.join("calibration.json");
    // 9,999 would break K9 against this run's m_ch_sb if it were kept: the run succeeding shows it was dropped.
    let stale = json!({"version": 1, "physics_hash": "0".repeat(64), "m_tov_sb": 9999.0, "m_ign_sb": 111.0});
    std::fs::write(&out, stale.to_string()).unwrap();
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb"], &[]);
    assert_eq!(code, 0, "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-WARN") && l.contains("physics_hash")), "a warning names the hash: {lines:?}");
    let file = read(&out);
    assert_eq!(file["physics_hash"], json!(physics_hash().unwrap()));
    assert!(file.get("m_tov_sb").is_none() && file.get("m_ign_sb").is_none(), "another build's items dropped: {file}");
    near(number(&file, "m_ch_sb"), M_CH, 0.005, "m_ch_sb");

    // A file that is not JSON is replaced, with a warning as well.
    std::fs::write(&out, "not json").unwrap();
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb"], &[]);
    assert_eq!(code, 0, "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-WARN")), "{lines:?}");
    near(number(&read(&out), "m_ch_sb"), M_CH, 0.005, "m_ch_sb after a corrupt file");
}

#[test]
fn a_run_with_overrides_writes_nothing() {
    let dir = scratch("overrides");
    let out = dir.join("calibration.json");
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb,m_tov_sb"], &[("paint_sigma", 0.03)]);
    assert_eq!(code, 0, "{lines:?}");
    assert!(!out.exists(), "a run with overrides never writes calibration (§2.12.1)");
    assert!(lines.iter().any(|l| l.starts_with("SR-WARN") && l.contains("nothing written")), "{lines:?}");

    // …and a file already there is left as it was, byte for byte.
    write_base(&out, json!({"m_ign_sb": 111.0}));
    let before = std::fs::read(&out).unwrap();
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb,m_tov_sb"], &[("paint_sigma", 0.03)]);
    assert_eq!(code, 0, "{lines:?}");
    assert_eq!(std::fs::read(&out).unwrap(), before);
}

#[test]
fn k9s_order_and_band_are_exit_6_with_nothing_written() {
    let dir = scratch("k9");
    let out = dir.join("calibration.json");
    // m_ch_sb measures 3,216.5, so the file's m_tov_sb decides: it must be above m_ch and 1.4–2.0 times it, i.e. in
    // [4,503, 6,433]. Below m_ch (order), 1.03 × (under the band), 2.18 × (over it): all refused; 4,800 (1.49 ×) is fine.
    for (tov, why) in [(3000.0, "m_tov below m_ch"), (3300.0, "ratio 1.03 under 1.4"), (7000.0, "ratio 2.18 over 2.0")] {
        write_base(&out, json!({"m_tov_sb": tov}));
        let before = std::fs::read(&out).unwrap();
        let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb"], &[]);
        assert_eq!(code, 6, "{why}: {lines:?}");
        let error = lines.iter().find(|l| l.starts_with("SR-ERROR")).unwrap_or_else(|| panic!("{why}: no SR-ERROR in {lines:?}"));
        assert!(error.contains("K9") && error.contains("m_tov/m_ch"), "{why}: the message names the rule: {error}");
        assert_eq!(std::fs::read(&out).unwrap(), before, "{why}: nothing written");
    }
    write_base(&out, json!({"m_tov_sb": 4800.0}));
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb"], &[]);
    assert_eq!(code, 0, "4,800 is in the band: {lines:?}");
}

#[test]
fn a_sigma_bh_under_ns_sigma_max_sb_is_exit_6_naming_it() {
    let dir = scratch("sigma-bh");
    let out = dir.join("calibration.json");
    // The sheet's m_tov_sb is 4,853.17: a sigma_bh just under it fails, just over it passes. (Overrides, so neither run writes.)
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb,m_tov_sb"], &[("sigma_bh", 4853.0)]);
    assert_eq!(code, 6, "{lines:?}");
    let error = lines.iter().find(|l| l.starts_with("SR-ERROR")).expect("an SR-ERROR line");
    assert!(error.contains("sigma_bh") && error.contains("ns_sigma_max_sb"), "the message names both: {error}");
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb,m_tov_sb"], &[("sigma_bh", 4854.0)]);
    assert_eq!(code, 0, "{lines:?}");
    assert!(!out.exists(), "neither run, having overrides, wrote");

    // The shipped constant read from the file's own text, not through the path that checks it.
    let shipped: Value = serde_json::from_str(PHYSICS).unwrap();
    assert_eq!(shipped["sigma_bh"], json!(6000), "§2.7 [M0-R4]: sigma_bh 6,000");
    let (code, lines) = calibrate(&["--out", out.to_str().unwrap(), "--only", "m_ch_sb,m_tov_sb"], &[]);
    assert_eq!(code, 0, "the shipped sigma_bh clears the ceiling: {lines:?}");
    assert!(number(&read(&out), "ns_sigma_max_sb") <= shipped["sigma_bh"].as_f64().unwrap());
}

#[test]
fn arguments_are_checked_and_adapter_is_accepted() {
    let dir = scratch("args");
    let out = dir.join("calibration.json");
    let out = out.to_str().unwrap();
    for (args, needle) in [
        (vec![], "--out"),
        (vec!["--out", out, "--bogus"], "unknown argument"),
        (vec!["--out"], "needs a value"),
        (vec!["--out", out, "--only", "nonsense"], "not an item"),
        (vec!["--out", out, "--only", "render_reserve_ms"], "--measure-ui"),
    ] {
        let (code, lines) = calibrate(&args, &[]);
        assert_eq!(code, 4, "{args:?}: {lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("SR-ERROR") && l.contains(needle)), "{args:?} should say {needle:?}: {lines:?}");
    }
    assert!(!Path::new(out).exists(), "a refused command wrote nothing");
    // --adapter names a GPU for the items that run one; these run none, and open none.
    let (code, lines) = calibrate(&["--adapter", "no-such-gpu", "--out", out, "--only", "m_tov_sb"], &[]);
    assert_eq!(code, 0, "{lines:?}");
    assert!(lines.iter().all(|l| !l.starts_with("SR-ADAPTER")), "{lines:?}");
}

#[test]
fn the_grid_disk_is_an_equilibrium_start_for_g_core_d() {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let vac = physics.get("sigma_vac").unwrap();
    // G-CORE (d)'s disk: 100 of C/O with k1e × 6, R₉₀ 18.8 cells (contract §5.1).
    let overrides = [("k1e".to_string(), 6.0 * physics.get("k1e").unwrap())];
    let (scene, planes) = cold_disk_scene(100.0, Matter::CarbonOxygen, &physics, &elements, &overrides)
        .expect("the SCF runs")
        .expect("a disk of 100 settles");
    let cells = 64 * 64;
    let sigma = &planes[..cells];
    let mass: f64 = sigma.iter().map(|&s| f64::from(s)).sum();
    near(mass, 100.0, 1e-6, "the scene's mass");
    let r90 = radius_holding(sigma, 64, vac, 0.9);
    assert!((18.0..19.6).contains(&r90), "R₉₀ {r90} cells, contract §5.1 says 18.8");
    assert_eq!(scene.overrides, overrides);
    assert!(!scene.writes_calibration());
    assert!(scene.switches.gravity && scene.switches.hydro && !scene.switches.heat && !scene.switches.sinks);
}
