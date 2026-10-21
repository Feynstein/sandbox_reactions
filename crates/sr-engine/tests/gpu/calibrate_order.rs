//! M0-T70 — `calibrate` items 5–6: the order (K9), the presets' masses, the Sun's life, the top rung, the stage durations and the
//! gas speed (contract §2.11 items 5–7, §1.8.2, §1.10.1, §3.3, §6.4). No test here touches a GPU or runs a life — every preset
//! life is a stub — and the module lives in the one test binary (R11) so the scope runs through `tests/gpu.sh`. The real lives
//! are the lead's, in M0-V9's window.
//!
//! The order is graded on a stubbed table, one refusal per inequality; the presets on a stubbed life (the masses asked for, the
//! endings checked, T_eff off the scale refused, TOP's rounding, the durations); the command's own order — the K9 check before
//! any preset life — through `run_with` on a file under the shipped physics hash.
//! Plant: `calibrate-order-skip` (m_tov/m_ch left unchecked).

use std::path::PathBuf;

use serde_json::{json, Map, Value};
use sr_engine::calibrate::{check_order, check_speed, measure_presets, run_with, CalError, Life, PresetRun, PRESET_TABLE};
use sr_physics::calibration::physics_hash;
use sr_physics::observe::translate::MassScale;
use sr_physics::registry::Physics;

/// A table that keeps the order: m_ign < m_ch < m_up < m_bh, m_tov/m_ch = 1.5, a neutron star under sigma_bh = 6,000.
fn table() -> Map<String, Value> {
    let Value::Object(map) = json!({
        "m_ign_sb": 3000.0, "m_ch_sb": 3200.0, "m_tov_sb": 4800.0, "ns_sigma_max_sb": 4800.0, "m_up_sb": 26_000.0, "m_bh_sb": 80_000.0
    }) else {
        unreachable!()
    };
    map
}

fn with(key: &str, value: f64) -> Map<String, Value> {
    let mut t = table();
    t.insert(key.to_string(), json!(value));
    t
}

fn refusal(file: &Map<String, Value>, sigma_bh: f64) -> String {
    match check_order(file, sigma_bh) {
        Err(CalError::Failed(m)) => {
            assert_eq!(CalError::Failed(m.clone()).exit_code(), 6);
            m
        }
        other => panic!("{file:?} with sigma_bh {sigma_bh}: {other:?}"),
    }
}

#[test]
fn a_table_in_order_passes_and_each_inequality_has_its_own_refusal() {
    check_order(&table(), 6000.0).expect("the stubbed table keeps K9");
    // The ratio's ends belong to K9's band.
    check_order(&with("m_tov_sb", 3200.0 * 1.4 + 1e-6), 6000.0).expect("the ratio 1.4 is inside");
    check_order(&with("m_tov_sb", 3200.0 * 2.0), 6000.0).expect("the ratio 2.0 is inside");
    for (key, value, needle) in [
        ("m_ign_sb", 3300.0, "m_ign_sb 3300.0 must be under m_ch_sb"),
        ("m_up_sb", 3100.0, "m_ch_sb 3200.0 must be under m_up_sb"),
        ("m_bh_sb", 25_000.0, "m_up_sb 26000.0 must be under m_bh_sb"),
        ("m_tov_sb", 3000.0, "m_ch_sb 3200.0, m_tov_sb 3000.0, m_tov/m_ch 0.938 — needs m_ch < m_tov"),
    ] {
        let message = refusal(&with(key, value), 6000.0);
        assert!(message.contains("K9") && message.contains(needle) && message.contains("never re-tuned"), "{key}: {message}");
    }
    // Equal is not under: each strict inequality refuses its tie.
    assert!(refusal(&with("m_ign_sb", 3200.0), 6000.0).contains("m_ign_sb"));
    assert!(refusal(&with("m_up_sb", 3200.0), 6000.0).contains("m_up_sb"));
}

#[test]
fn the_tov_over_chandrasekhar_ratio_is_held_to_1_4_through_2() {
    for (tov, side) in [(3200.0 * 1.39, "1.390"), (3200.0 * 2.01, "2.010"), (3200.0 * 1.0001, "1.000")] {
        let message = refusal(&with("m_tov_sb", tov), 6000.0);
        assert!(message.contains("m_tov/m_ch") && message.contains(side) && message.contains("[1.4, 2.0]"), "{tov}: {message}");
    }
}

#[test]
fn sigma_bh_must_not_sit_under_the_densest_holding_neutron_star() {
    check_order(&table(), 4800.0).expect("sigma_bh at ns_sigma_max_sb is enough");
    let message = refusal(&table(), 4799.0);
    assert!(message.contains("sigma_bh 4799") && message.contains("ns_sigma_max_sb 4800.0"), "{message}");
    // A table without the pair checks what it has: nothing to refuse.
    check_order(&Map::new(), 1.0).expect("an empty file has no order to break");
}

#[test]
fn the_gas_speed_must_stay_under_half_the_speed_of_light() {
    check_speed(99.9, 200.0).expect("c_sb 200 beats twice 99.9");
    for speed in [100.0, 150.0, f64::NAN] {
        match check_speed(speed, 200.0) {
            Err(CalError::Failed(m)) => assert!(m.contains("c_sb 200 must exceed twice it"), "{speed}: {m}"),
            other => panic!("{speed}: {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The presets, on stubbed lives

const ANCHORS: [f64; 4] = [3000.0, 3200.0, 26_000.0, 80_000.0];

fn life(mass: f64, ending: &str, sim_time: f64) -> Life {
    Life { mass, ignited: true, core_collapse: ending != "white_dwarf", collapsed: ending != "white_dwarf", ending: Some(ending.to_string()), steps: 5000, sim_time, wall_s: 2.0 }
}

/// A stubbed preset life: `stages` as (id, first-frame sim time), the Sun's L and R at X_H,c 0.35.
fn run(mass: f64, ending: &str, stages: &[(&str, f64)], at_x_h: Option<(f64, f64)>, gas: f64) -> PresetRun {
    PresetRun {
        life: life(mass, ending, stages.last().map_or(0.0, |s| s.1) + 10.0),
        stages: stages.iter().map(|(s, t)| (s.to_string(), *t)).collect(),
        at_x_h,
        max_gas_speed: gas,
    }
}

/// L, R giving T_eff = 0.3 with σ = a_r c_sb / 4 = 2e-3 and R = 15: L = 2π R σ T⁴.
fn l_for(t_eff: f64) -> f64 {
    2.0 * std::f64::consts::PI * 15.0 * 2e-3 * t_eff.powi(4)
}

fn stub(asked: &mut Vec<(String, f64)>, sun_life: f64) -> impl FnMut(&str, f64) -> Result<PresetRun, CalError> + '_ {
    move |name, mass| {
        asked.push((name.to_string(), mass));
        Ok(match name {
            "sun" => run(
                mass,
                "white_dwarf",
                &[("collapsing", 0.0), ("protostar", 100.0), ("main_sequence", 150.0), ("h_shell", 900.0), ("white_dwarf", sun_life)],
                Some((l_for(0.3), 15.0)),
                8.0,
            ),
            "massive" => run(
                mass,
                "neutron_star",
                &[("collapsing", 0.0), ("main_sequence", 60.0), ("h_shell", 200.0), ("si_burn", 260.0), ("core_collapse", 280.0), ("neutron_star", 290.0)],
                None,
                31.5,
            ),
            _ => run(mass, "black_hole", &[("collapsing", 0.0), ("main_sequence", 30.0), ("core_collapse", 90.0), ("black_hole", 91.0)], None, 12.0),
        })
    }
}

fn physics() -> Physics {
    Physics::shipped().expect("the shipped constants")
}

#[test]
fn the_presets_are_asked_at_the_inverse_translation_of_1_15_and_40_suns() {
    let (mut asked, mut said) = (Vec::new(), Vec::new());
    let presets = measure_presets(ANCHORS, &physics(), &mut stub(&mut asked, 1234.5), &mut |l| said.push(l.to_string())).expect("a clean set of lives");
    let scale = MassScale::new(ANCHORS[0], ANCHORS[1], ANCHORS[2], ANCHORS[3]).unwrap();
    let want = [scale.to_sb(1.0), scale.to_sb(15.0), scale.to_sb(40.0)];
    assert_eq!(presets.masses, want, "preset_mass_sb is MassScale::to_sb at 1, 15 and 40 Suns");
    assert_eq!(asked.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["sun", "massive", "giant"], "the three presets, in order");
    assert_eq!(asked.iter().map(|(_, m)| *m).collect::<Vec<_>>(), want);
    assert!(want[0] > ANCHORS[0] && want[0] < ANCHORS[2] && want[1] > ANCHORS[2] && want[1] < ANCHORS[3] && want[2] > ANCHORS[3], "{want:?}: 1 Sun is between ignition and the collapse, 15 between M_up and M_BH, 40 over M_BH");
    assert_eq!(PRESET_TABLE.map(|p| p.1), ["white_dwarf", "neutron_star", "black_hole"]);
    assert_eq!(said.len(), 3, "one run line a preset: {said:?}");
    assert!(said.iter().all(|l| l.starts_with("SR-CALIBRATE run item=preset_") && l.contains("ending=") && l.contains("wall_s=")), "{said:?}");
}

#[test]
fn the_suns_life_the_top_rung_and_t_eff_come_from_the_sun_like_life() {
    let (mut asked, mut said) = (Vec::new(), Vec::new());
    let presets = measure_presets(ANCHORS, &physics(), &mut stub(&mut asked, 1234.5), &mut |l| said.push(l.to_string())).unwrap();
    assert_eq!(presets.life_sun_sb, 1234.5, "life_sun_sb is the first frame of the white-dwarf stage, from the drop at 0");
    assert_eq!(presets.top_rung, 120.0, "TOP = 1234.5 / 10 = 123.45, two significant digits");
    assert!((presets.t_eff_sun_sb / 0.3 - 1.0).abs() < 1e-9, "T_eff = (L / 2πRσ)^¼ from the stubbed L and R: {}", presets.t_eff_sun_sb);
    // TOP's rounding at the edges of its decades, away from ties.
    for (life_sun, top) in [(423.0, 42.0), (1260.0, 130.0), (9990.0, 1000.0), (56.0, 5.6), (98_700.0, 9900.0)] {
        let presets = measure_presets(ANCHORS, &physics(), &mut stub(&mut Vec::new(), life_sun), &mut |_| {}).unwrap();
        assert!((presets.top_rung / top - 1.0).abs() < 1e-12, "life_sun_sb {life_sun}: {} is not {top}", presets.top_rung);
    }
}

#[test]
fn a_transient_white_dwarf_before_the_main_sequence_is_not_the_suns_life_and_books_no_duration() {
    // M0-D61: the shipped preset shows failed_star, then a white dwarf at its bounce, before its main sequence; the ending is
    // the white dwarf the life holds at its end.
    let mut asked = Vec::new();
    let mut stub = stub(&mut asked, 1234.5);
    let mut sun = |name: &str, mass: f64| -> Result<PresetRun, CalError> {
        let mut r = stub(name, mass)?;
        if name == "sun" {
            r.stages = [("failed_star", 8.0), ("collapsing", 78.0), ("white_dwarf", 579.0), ("cloud", 906.0), ("main_sequence", 1000.0), ("white_dwarf", 5000.0)]
                .map(|(s, t)| (s.to_string(), t))
                .to_vec();
        }
        Ok(r)
    };
    let presets = measure_presets(ANCHORS, &physics(), &mut sun, &mut |_| {}).expect("a clean set of lives");
    assert_eq!(presets.life_sun_sb, 5000.0, "life_sun_sb is the start of the white dwarf the ending closes, not the bounce's at 579");
    assert_eq!(presets.top_rung, 500.0, "TOP = 5000 / 10");
    let d = &presets.stage_durations_sb;
    for stage in ["white_dwarf", "failed_star"] {
        assert!(!d.contains_key(stage), "{stage} is a remnant stage: no duration, shown at the bounce or not: {d:?}");
    }
    assert_eq!(d["main_sequence"][0], (presets.masses[0], 4000.0));
}

#[test]
fn the_stage_durations_are_one_pair_a_life_with_the_remnant_left_out() {
    let presets = measure_presets(ANCHORS, &physics(), &mut stub(&mut Vec::new(), 1234.5), &mut |_| {}).unwrap();
    let d = &presets.stage_durations_sb;
    let [sun, massive, giant] = presets.masses;
    assert_eq!(d["main_sequence"], vec![(sun, 750.0), (massive, 140.0), (giant, 60.0)], "main_sequence in all three lives, in preset order");
    assert_eq!(d["h_shell"], vec![(sun, 1234.5 - 900.0), (massive, 60.0)]);
    assert_eq!(d["collapsing"], vec![(sun, 100.0), (massive, 60.0), (giant, 30.0)]);
    assert_eq!(d["core_collapse"], vec![(massive, 10.0), (giant, 1.0)]);
    assert_eq!(d["si_burn"], vec![(massive, 20.0)]);
    for remnant in ["white_dwarf", "neutron_star", "black_hole"] {
        assert!(!d.contains_key(remnant), "{remnant} is a remnant: its clock is the per-t.u. rate");
    }
    assert!(d.values().flatten().all(|&(m, t)| m > 0.0 && t > 0.0), "every pair is what the file's parser accepts: {d:?}");
    // The gas speed is the largest of the three lives.
    assert_eq!(presets.max_gas_speed, 31.5);
}

#[test]
fn a_preset_that_does_not_end_as_promised_is_exit_6_and_stops_the_lives() {
    let mut asked = Vec::new();
    let mut wrong = |name: &str, mass: f64| -> Result<PresetRun, CalError> {
        asked.push(name.to_string());
        Ok(run(mass, if name == "massive" { "black_hole" } else { "white_dwarf" }, &[("collapsing", 0.0), ("white_dwarf", 50.0)], Some((l_for(0.3), 15.0)), 1.0))
    };
    let error = measure_presets(ANCHORS, &physics(), &mut wrong, &mut |_| {}).expect_err("15 Suns ended as a black hole");
    assert_eq!(error.exit_code(), 6);
    let CalError::Failed(m) = &error else { panic!("{error:?}") };
    assert!(m.contains("preset_massive") && m.contains("ended as black_hole, not neutron_star") && m.contains("not re-tuned"), "{m}");
    assert_eq!(asked, ["sun", "massive"], "no giant life after a failed one");
}

#[test]
fn a_t_eff_off_the_temperature_scale_or_a_sun_that_never_burned_down_is_refused() {
    let t_h = physics().get("t_h").unwrap();
    let t_floor = physics().get("t_floor").unwrap();
    for (what, at_x_h, needle) in [
        ("T_eff over T_H", Some((l_for(t_h * 1.5), 15.0)), "off the temperature scale"),
        ("T_eff under T_floor", Some((l_for(t_floor * 0.5), 15.0)), "off the temperature scale"),
        ("no luminosity", Some((0.0, 15.0)), "off the temperature scale"),
        ("hydrogen never fell to 0.35", None, "never reached X_H,c 0.35"),
    ] {
        let mut sun = |name: &str, mass: f64| -> Result<PresetRun, CalError> {
            assert_eq!(name, "sun", "{what}: the Sun-like life decides before the others run");
            Ok(run(mass, "white_dwarf", &[("main_sequence", 0.0), ("white_dwarf", 900.0)], at_x_h, 1.0))
        };
        match measure_presets(ANCHORS, &physics(), &mut sun, &mut |_| {}) {
            Err(CalError::Failed(m)) => assert!(m.contains(needle), "{what}: {m}"),
            other => panic!("{what}: {other:?}"),
        }
    }
    // A Sun that never showed a white dwarf has no life to rate.
    let mut no_wd = |_: &str, mass: f64| -> Result<PresetRun, CalError> {
        let mut r = run(mass, "white_dwarf", &[("main_sequence", 0.0)], Some((l_for(0.3), 15.0)), 1.0);
        r.life.ending = Some("white_dwarf".into());
        Ok(r)
    };
    assert!(matches!(measure_presets(ANCHORS, &physics(), &mut no_wd, &mut |_| {}), Err(CalError::Failed(m)) if m.contains("life_sun_sb")));
}

// ---------------------------------------------------------------------------------------------------------------
// The command: the order before the lives, item 7 kept

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-calibrate-order-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn calibrate(args: &[&str]) -> (i32, Vec<String>) {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut lines = Vec::new();
    let code = run_with(&args, &[], &mut |l| lines.push(l.to_string()));
    for line in &lines {
        println!("{line}");
    }
    (code, lines)
}

fn base(path: &std::path::Path, mut items: Value) {
    items["version"] = json!(1);
    items["physics_hash"] = json!(physics_hash().unwrap());
    items["measured"] = json!({"date": "2026-10-19T00:00:00Z", "git": "test", "adapter": "RTX 5090", "box": "test"});
    std::fs::write(path, items.to_string()).unwrap();
}

#[test]
fn the_command_checks_the_order_before_it_runs_any_preset_life() {
    let dir = scratch("order");
    let path = dir.join("calibration.json");
    let out = path.to_str().unwrap();
    // m_ign over m_ch: exit 6 on K9 and no adapter opened — the presets would be hours on a broken order.
    base(&path, json!({"m_ign_sb": 3300.0, "m_ch_sb": 3200.0, "m_up_sb": 26_000.0, "m_bh_sb": 80_000.0}));
    let before = std::fs::read_to_string(&path).unwrap();
    let (code, lines) = calibrate(&["--out", out, "--only", "preset_mass_sb"]);
    assert_eq!(code, 6, "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("SR-ERROR calibration failed: K9:") && l.contains("m_ign_sb 3300.0 must be under m_ch_sb")), "{lines:?}");
    assert!(lines.iter().all(|l| !l.starts_with("SR-ADAPTER") && !l.contains("SR-CALIBRATE run")), "no GPU, no life: {lines:?}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "a K9 failure writes nothing");
    // An ordered file without its anchors cannot place the presets: exit 6, naming them, still no GPU.
    base(&path, json!({"m_ch_sb": 3200.0, "m_up_sb": 26_000.0}));
    let (code, lines) = calibrate(&["--out", out, "--only", "preset_mass_sb,life_sun_sb,top_rung"]);
    assert_eq!(code, 6, "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("needs m_ign_sb, m_ch_sb, m_up_sb and m_bh_sb")), "{lines:?}");
    assert!(lines.iter().all(|l| !l.starts_with("SR-ADAPTER")), "{lines:?}");
    // The whole item-6 family is one job, and item 7 is the drawing lot's.
    let (code, lines) = calibrate(&["--out", out, "--only", "render_reserve_ms"]);
    assert_eq!(code, 4, "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("--measure-ui")), "{lines:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn render_reserve_ms_is_kept_across_a_rerun_and_the_gas_speed_bound_is_enforced_on_the_file() {
    let dir = scratch("keep");
    let path = dir.join("calibration.json");
    let out = path.to_str().unwrap();
    // Item 7 is the drawing lot's: a re-run of other items keeps it, and a base file's other items with it.
    base(&path, json!({"render_reserve_ms": 4.25, "life_sun_sb": 1200.0, "max_gas_speed": 20.0}));
    let (code, lines) = calibrate(&["--out", out, "--only", "m_ch_sb"]);
    assert_eq!(code, 0, "{lines:?}");
    let file: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(file["render_reserve_ms"], json!(4.25), "calibrate keeps the drawing lot's measurement");
    assert_eq!(file["life_sun_sb"], json!(1200.0));
    assert_eq!(file["max_gas_speed"], json!(20.0), "no life ran: the file's gas speed stands");
    assert!(file["m_ch_sb"].as_f64().is_some_and(|m| m > 3000.0));
    // c_sb = 200 must beat twice the file's gas speed: 100 does not, and nothing is written.
    base(&path, json!({"render_reserve_ms": 4.25, "max_gas_speed": 100.0}));
    let before = std::fs::read_to_string(&path).unwrap();
    let (code, lines) = calibrate(&["--out", out, "--only", "m_ch_sb"]);
    assert_eq!(code, 6, "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("max_gas_speed 100.000") && l.contains("c_sb 200 must exceed twice it")), "{lines:?}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    let _ = std::fs::remove_dir_all(&dir);
}
