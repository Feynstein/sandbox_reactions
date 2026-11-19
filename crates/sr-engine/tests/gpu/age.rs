//! M0-T107 — the age clock over a touched life (contract §5.4's G-AGE; §1.10.4, §2.12.5, §0.5 [M0-D103]): the readout's age
//! (`age_years`, the Watch's `AgeClock` on every frame's displayed stages) never decreases and never jumps when the player
//! changes the star, and a Sun-like star's whole life reads about the 1.21 × 10¹⁰ years §1.10.4's table sums to. Two lives:
//! - `age_touch`: the Massive preset under scenes/age_touch.json — hydrogen painted onto the core when X_H,c first falls under
//!   0.5 (the fuel painted back in: the clock's Δφ goes negative and is clamped, never subtracted), and later, when X_H,c first
//!   falls under 0.3 on the burn that follows, 30 % of the mass erased from outside in (the mass changes the rate going forward,
//!   never the age banked) — to its ending;
//! - `sun`: the Sun-like preset, untouched, to its white dwarf.
//!
//! What a case reads, and from where (R12's one adapter): every progress tick's heaviest object — its tracker id, shown stage,
//! sim time, mass, X_H,c and `age_years` (`Tick::star`) — and, for the Sun-like life, summary.json's `objects[until.object]`
//! (`age_years`, `ending`). Readings declared (M0-T107; cheap to reverse):
//! - **Never decreases**: between two consecutive ticks of one object id (a different heaviest object is another clock, §1.10.4)
//!   the age never falls.
//! - **No jump** (the lead's reading, 2026-11-19, at M0-T107: «Per-visit, to core collapse»): an increment is the age gained
//!   between two consecutive ticks of one object, per t.u. of sim time between them (ticks fall every 16 steps in the one-step
//!   chunks an unspent edit trigger forces and every frame otherwise, so a raw per-tick increment would differ 16-fold within one
//!   stage on the chunking alone). It is banked to the stage in effect over the interval, the earlier tick's, as the clock banks it
//!   (the tick that shows a new stage carries the old one's last share). «Its stage's median increment» is the median of the
//!   increments of one *visit* — the run of ticks of one object showing one stage without a break — and the ticks graded are
//!   those before the first that shows `core_collapse` (the star's life, not the collapse transient). Measured on the RTX 5090
//!   (M0-T107): the displayed stage of a touched star flaps (main_sequence ↔ he_core ↔ o_burn after the paint, and
//!   main_sequence shown again for the collapsing remnant at X_H,c ≈ 0.1, earning ~100 × the main sequence's rate), so one median
//!   over a whole stage reads 5,572 × there; it is printed beside the graded reading, never graded. A visit with fewer than
//!   [`MIN_TICKS`] increments is not graded; a visit's median is of its positive increments when that is 0; no increment of it is
//!   over [`JUMP`] × that median.
//! - **The Sun-like age** is read at the ending, after the remnant held `ending_hold` (§0.5 [M0-D95]): the remnant's 10⁸ years
//!   per t.u. are in it (about 10⁹ years), well inside the factor 1.5.
//! - Each case prints each stage's share of the age, so a clock that ignores the measured stage durations and still lands on
//!   1.21 × 10¹⁰ shows it (the adversarial reading).
//!
//! Each case first grades its grader on hand-made values. The lives are long (the Sun-like's about 179,000 steps; the touched
//! Massive's about 100,000): R12's ⏱ scope, the lead's run.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::Value;
use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick};
use sr_engine::scene::Scene;
use sr_physics::calibration::{self, Calibration};

/// §5.4's «no summary adds more than 10 × its stage's median increment».
const JUMP: f64 = 10.0;
/// A visit with fewer increments than this has no median worth a bound: printed, not graded.
const MIN_TICKS: usize = 8;
/// §5.4: the Sun-like preset reaches its white dwarf reading within this factor of [`SUN_YEARS`].
const FACTOR: f64 = 1.5;
/// §1.10.4's τ_s summed over a Sun-like star's life (SP's ~11 Gyr arithmetic), in years.
const SUN_YEARS: f64 = 1.21e10;
/// The painted core's central hydrogen must come back up by at least this much after the paint (else the clock's Δφ < 0 was
/// never exercised and the case grades nothing about it).
const PAINT_RISE: f64 = 0.02;
/// The erase must take at least this share of the star's mass (the file asks for 30 %: the band reads under it).
const ERASE_SHARE: f64 = 0.2;
/// A mass moving this much (relative) between two ticks is an edit landing.
const MOVED: f64 = 0.01;

/// One progress tick of the heaviest object.
#[derive(Clone, Debug)]
struct Sample {
    step: u64,
    sim_time: f64,
    id: u32,
    stage: String,
    age: f64,
    mass: f64,
    x_h_c: f64,
}

fn scenes_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

/// What one life gave: the exit code, summary.json and every tick the heaviest object had an age on.
struct Life {
    code: i32,
    summary: Value,
    samples: Vec<Sample>,
    steps: u64,
}

/// The preset at `mass` run to its ending (with `edits` when given, in the world `edge`) on the physics adapter, summary.json kept.
fn life(id: &str, preset: &str, mass: f64, edits: Option<&str>, edge: &str) -> Life {
    let scene = Scene::preset_mass(preset, mass, Default::default()).unwrap();
    let out = std::env::temp_dir().join(format!("sr-age-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let out_dir = out.display().to_string();
    let label = format!("age:{id}");
    let mut args: Vec<String> = ["--scene", &label, "--until", "ending", "--max-steps", "600000", "--out", &out_dir, "--edge", edge]
        .map(String::from)
        .to_vec();
    if let Some(e) = edits {
        args.extend(["--edits".to_string(), scenes_dir().join(e).display().to_string()]);
    }
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let mut samples: Vec<Sample> = Vec::new();
    let mut last_step = 0u64;
    let mut ticks = 0u64;
    let mut tick = |t: &Tick| {
        last_step = t.step;
        ticks += 1;
        let Some(star) = &t.star else { return };
        let Some(age) = star.age_years else { return };
        samples.push(Sample { step: t.step, sim_time: t.sim_time, id: star.id, stage: t.stage.to_string(), age, mass: star.mass, x_h_c: star.x_h_c });
        if ticks.is_multiple_of(200) {
            println!(
                "{id} step={} t={:.2} stage={} id={} M={:.1} X_H,c={:.3} Σ_c={:.3} R={:.1} age={:.4e} yr",
                t.step, t.sim_time, t.stage, star.id, star.mass, star.x_h_c, star.sigma_c, star.radius, age
            );
        }
    };
    let mut saved = |_: u64, _: f64, _: &str| {};
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: None, saved: &mut saved };
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress));
    assert!(code == 0 || code == 2, "{id}: the run ended with exit {code}: {lines:?}");
    let text = std::fs::read_to_string(out.join("summary.json")).unwrap_or_else(|e| panic!("{id}: no summary.json: {e}"));
    let _ = std::fs::remove_dir_all(&out);
    let summary: Value = serde_json::from_str(&text).expect("summary.json parses");
    // SR_AGE_DUMP=<folder>: every tick of the life as `<folder>/<case>.csv`, for reading a red run by hand.
    if let Some(dir) = std::env::var_os("SR_AGE_DUMP") {
        let rows: String = samples
            .iter()
            .map(|s| format!("{},{:.9},{},{},{:.9e},{:.6},{:.6}\n", s.step, s.sim_time, s.id, s.stage, s.age, s.mass, s.x_h_c))
            .collect();
        let path = std::path::Path::new(&dir).join(format!("{id}.csv"));
        let _ = std::fs::write(&path, format!("step,sim_time,id,stage,age_years,mass_sb,x_h_c\n{rows}"));
    }
    Life { code, summary, samples, steps: last_step }
}

fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    match s.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => s[n / 2],
        n => 0.5 * (s[n / 2 - 1] + s[n / 2]),
    }
}

/// What a stage's ticks measured, over every visit of it.
#[derive(Debug, Default, PartialEq)]
struct StageAge {
    /// The years its ticks gained, in total.
    gained: f64,
    /// Its increments (years per t.u.) between consecutive ticks of one object, over the whole life.
    rates: Vec<f64>,
    /// The largest increment of a graded visit over that visit's median (the graded reading; 0 while no visit of the stage is
    /// graded), and the largest increment over the median of all the stage's increments (printed, never graded).
    worst_visit: f64,
    worst_stage: f64,
}

/// One visit: consecutive ticks of one object showing one stage — the run of increments the clock banked to that stage.
struct Visit {
    stage: String,
    rates: Vec<f64>,
    steps: Vec<u64>,
}

/// The median an increment is bound by: the median of the increments, of the positive ones when that is 0, `None` when none is.
fn bound_median(rates: &[f64]) -> Option<f64> {
    let m = median(rates);
    if m > 0.0 {
        return Some(m);
    }
    let positive: Vec<f64> = rates.iter().copied().filter(|r| *r > 0.0).collect();
    (!positive.is_empty()).then(|| median(&positive))
}

/// Each stage's tally and each visit's increments; `Err` names the first tick whose age fell (between two ticks of one object).
/// An interval's increment is banked to the stage in effect over it — the earlier tick's, as the clock does (a stage that ends is
/// banked up to the frame that ends it) — so a tick that shows a new stage carries the old stage's last share, not the new one's.
fn tally(samples: &[Sample]) -> Result<(BTreeMap<String, StageAge>, Vec<Visit>), String> {
    let mut stages: BTreeMap<String, StageAge> = BTreeMap::new();
    let mut visits: Vec<Visit> = Vec::new();
    let mut open: Option<(u32, String)> = None;
    for pair in samples.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if a.id != b.id {
            open = None;
            continue;
        }
        if !(b.age >= a.age) {
            return Err(format!(
                "the age fell between step {} and step {} (object {}, stage {} → {}): {:.6e} → {:.6e} years",
                a.step, b.step, a.id, a.stage, b.stage, a.age, b.age
            ));
        }
        let entry = stages.entry(a.stage.clone()).or_default();
        entry.gained += b.age - a.age;
        let dt = b.sim_time - a.sim_time;
        if dt <= 0.0 {
            continue;
        }
        let rate = (b.age - a.age) / dt;
        entry.rates.push(rate);
        if open.as_ref() != Some(&(a.id, a.stage.clone())) {
            visits.push(Visit { stage: a.stage.clone(), rates: Vec::new(), steps: Vec::new() });
            open = Some((a.id, a.stage.clone()));
        }
        let v = visits.last_mut().expect("a visit is open");
        v.rates.push(rate);
        v.steps.push(b.step);
    }
    Ok((stages, visits))
}

/// The ticks before the first that shows `core_collapse`: the star's life up to its collapse.
fn until_collapse(samples: &[Sample]) -> &[Sample] {
    let end = samples.iter().position(|s| s.stage == "core_collapse").unwrap_or(samples.len());
    &samples[..end]
}

/// The age series graded on plain values: `Err` names the first tick of the whole life whose age fell, or the first increment, among
/// the ticks before the first `core_collapse`, over [`JUMP`] × the median of its visit; `Ok` is each stage's tally over the whole
/// life. A visit with fewer than [`MIN_TICKS`] increments is not graded.
fn grade_series(samples: &[Sample]) -> Result<BTreeMap<String, StageAge>, String> {
    let (mut stages, _) = tally(samples)?;
    for entry in stages.values_mut() {
        if let Some(m) = bound_median(&entry.rates) {
            entry.worst_stage = entry.rates.iter().copied().fold(0.0, f64::max) / m;
        }
    }
    let (_, visits) = tally(until_collapse(samples))?;
    for v in &visits {
        if v.rates.len() < MIN_TICKS {
            continue;
        }
        let Some(m) = bound_median(&v.rates) else { continue };
        let entry = stages.get_mut(&v.stage).expect("a visit's stage is tallied");
        entry.worst_visit = entry.worst_visit.max(v.rates.iter().copied().fold(0.0, f64::max) / m);
        if let Some(i) = v.rates.iter().position(|r| *r > JUMP * m) {
            return Err(format!(
                "the age jumped in {} at step {}: {:.4e} years per t.u., {:.1} × the median {m:.4e} of this visit of the stage (bound {JUMP} ×)",
                v.stage,
                v.steps[i],
                v.rates[i],
                v.rates[i] / m
            ));
        }
    }
    Ok(stages)
}

/// Each stage's share of the years the series gained, in stage order of first appearance, as a printable table.
fn shares(samples: &[Sample], stages: &BTreeMap<String, StageAge>) -> String {
    let total: f64 = stages.values().map(|s| s.gained).sum();
    let mut order: Vec<&str> = Vec::new();
    for s in samples {
        if !order.contains(&s.stage.as_str()) {
            order.push(&s.stage);
        }
    }
    order
        .iter()
        .filter_map(|name| stages.get(*name).map(|s| (name, s)))
        .map(|(name, s)| {
            format!(
                "{name}: {:.3e} yr = {:.2} % (worst {:.1} × its visit's median, {:.1} × the stage's)",
                s.gained,
                100.0 * s.gained / total,
                s.worst_visit,
                s.worst_stage
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The Sun-like life graded on plain values: the life met its ending, the object that ended is a white dwarf with an age, and the
/// age is within [`FACTOR`] of [`SUN_YEARS`] either side.
fn grade_sun(summary: &Value, code: i32) -> Result<f64, String> {
    if code != 0 || summary["until"]["met"] != true {
        return Err(format!("no ending within the step cap (exit {code}, until {})", summary["until"]));
    }
    let object = &summary["until"]["object"];
    let Some(o) = summary["objects"].as_array().and_then(|os| os.iter().find(|o| o["id"] == *object)) else {
        return Err(format!("the ending object {object} is not in summary.json's objects"));
    };
    if o["ending"] != "white_dwarf" {
        return Err(format!("ended {}, the Sun-like preset's life ends a white dwarf", o["ending"]));
    }
    let Some(age) = o["age_years"].as_f64() else {
        return Err(format!("the ending object {object} carries no age_years ({})", o["age_years"]));
    };
    if !(age >= SUN_YEARS / FACTOR && age <= SUN_YEARS * FACTOR) {
        return Err(format!("the white dwarf reads {age:.4e} years, not within a factor {FACTOR} of {SUN_YEARS:.3e} ([{:.3e}, {:.3e}])", SUN_YEARS / FACTOR, SUN_YEARS * FACTOR));
    }
    Ok(age)
}

/// What the touched life's ticks say the edits did: the paint landed (the mass jumped up) and the central hydrogen came back up
/// by [`PAINT_RISE`] after it; the erase landed (the mass fell by [`ERASE_SHARE`] of what it was on a later tick); the life ran
/// on one object across both. `Ok` is (the hydrogen's rise after the paint, the erase's share of the mass).
fn grade_edits(samples: &[Sample]) -> Result<(f64, f64), String> {
    let jump = samples
        .windows(2)
        .position(|p| p[0].id == p[1].id && p[1].mass > p[0].mass * (1.0 + MOVED))
        .ok_or("no paint landed: the mass never jumped up between two ticks of one object (X_H,c never under 0.5, or P0 dropped the edit)")?;
    let landed = &samples[jump + 1];
    let before = samples[jump].x_h_c;
    let after = samples[jump + 1..].iter().take_while(|s| s.id == landed.id).map(|s| s.x_h_c).fold(f64::MIN, f64::max);
    let rise = after - before;
    if rise < PAINT_RISE {
        return Err(format!(
            "the painted core's X_H,c rose {rise:.4} (from {before:.4} to at most {after:.4}), under {PAINT_RISE}: the clock's Δφ < 0 was never exercised — regenerate scenes/age_touch.json"
        ));
    }
    let top = landed.mass;
    let erased = samples[jump + 1..]
        .windows(2)
        .position(|p| p[0].id == p[1].id && p[1].mass < p[0].mass * (1.0 - MOVED))
        .ok_or("no erase landed: the mass never fell between two ticks of the object after the paint (X_H,c never under 0.3, or P0 dropped the edit)")?;
    let low = samples[jump + 1 + erased + 1..].iter().take(64).map(|s| s.mass).fold(f64::MAX, f64::min);
    let share = (top - low) / top;
    if !(share >= ERASE_SHARE) {
        return Err(format!("the erase took {:.1} % of the mass ({top:.1} → {low:.1} sb), under {} %", 100.0 * share, ERASE_SHARE * 100.0));
    }
    Ok((rise, share))
}

fn calibration() -> Calibration {
    calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration")
}

/// One touched Massive life, run once for the cases that read it.
fn touched() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| {
        let cal = calibration();
        let l = life("touch", "massive", cal.preset_mass_sb.massive, Some("age_touch.json"), "leave");
        println!("touch: {} steps, exit {}, {} ticks with an age, summary objects {}", l.steps, l.code, l.samples.len(), l.summary["objects"]);
        l
    })
}

fn sample(step: u64, id: u32, stage: &str, age: f64, mass: f64, x_h_c: f64) -> Sample {
    Sample { step, sim_time: step as f64, id, stage: stage.to_string(), age, mass, x_h_c }
}

/// The graders on hand-made values, before any life: a steady series passes; an age that falls (one tick, and a fall across a
/// stage change), a jump (an increment over ten times its stage's median), a jump in one stage among steady ones and a mass jump
/// with no age jump alongside are each handled; a short stage is not graded; another object's clock is not compared; the Sun-like
/// reading inside and outside its factor; an ending that is not a white dwarf, none, no age and an unmet life are refused; a paint
/// that raised no hydrogen and an erase that took too little are refused.
fn graders_are_graded() {
    let steady = |n: u64| -> Vec<Sample> { (0..n).map(|i| sample(i + 1, 1, "main_sequence", 100.0 + 5.0 * i as f64, 9000.0, 0.6)).collect() };
    assert!(grade_series(&steady(40)).is_ok(), "refused a steady series");
    let mut falls = steady(40);
    falls[20].age -= 50.0;
    assert!(grade_series(&falls).unwrap_err().contains("fell"), "accepted an age that fell");
    let mut across = steady(40);
    for s in across[21..].iter_mut() {
        s.stage = "h_shell".to_string();
        s.age -= 30.0;
    }
    assert!(grade_series(&across).unwrap_err().contains("fell"), "accepted a fall across a stage change");
    let mut jumps = steady(40);
    for s in jumps[25..].iter_mut() {
        s.age += 500.0;
    }
    assert!(grade_series(&jumps).unwrap_err().contains("jumped"), "accepted a jump of 100 × the median");
    // A stage shown again at 100 × the rate is another visit with its own median, never a jump; the same rate jumping inside one
    // visit is one.
    let mut flap: Vec<Sample> = steady(20);
    let top = flap.last().unwrap().age;
    flap.extend((0..20).map(|i| sample(21 + i, 1, "he_core", top + 5.0 * (i + 1) as f64, 9000.0, 0.4)));
    let top = flap.last().unwrap().age;
    flap.extend((0..20).map(|i| sample(41 + i, 1, "main_sequence", top + 5.0 + 500.0 * i as f64, 9000.0, 0.4)));
    assert!(grade_series(&flap).is_ok(), "refused a stage shown again at another rate");
    flap[50].age += 100000.0;
    for s in flap[51..].iter_mut() {
        s.age += 100000.0;
    }
    assert!(grade_series(&flap).unwrap_err().contains("jumped"), "accepted a jump inside the revisit");
    // The stage's last share lands on the tick that shows the next one: banked to the old stage's visit, 3 × its median is fine,
    // 30 × is a jump.
    let banked = |extra: f64| -> Vec<Sample> {
        let mut v = steady(20);
        v.extend((0..20).map(|i| sample(21 + i, 1, "h_shell", 100.0 + 5.0 * 19.0 + 5.0 * (i + 1) as f64 + extra, 9000.0, 0.4)));
        v
    };
    assert!(grade_series(&banked(10.0)).is_ok(), "refused a banked remainder of 3 × the median");
    assert!(grade_series(&banked(150.0)).unwrap_err().contains("jumped in main_sequence"), "accepted a banked jump of 30 ×");
    // A rate that drifts (the mass changed: ×8 over the visit) is no jump; one tick far over the visit's median is.
    let mut age = 0.0;
    let drift: Vec<Sample> = (0..120)
        .map(|i| {
            age += 5.0 * (1.0 + 7.0 * i as f64 / 119.0);
            sample(i + 1, 1, "main_sequence", age, 9000.0, 0.6)
        })
        .collect();
    assert!(grade_series(&drift).is_ok(), "refused a rate that drifted ×8 over its visit");
    let mut spike = drift.clone();
    for s in spike[90..].iter_mut() {
        s.age += 1000.0;
    }
    assert!(grade_series(&spike).unwrap_err().contains("jumped"), "accepted a one-tick jump of 40 × inside a drifting rate");
    // Only the star's life up to its core collapse is graded for jumps; the age never falls at any point of the whole life.
    let mut collapsed = steady(40);
    collapsed.extend((0..20).map(|i| sample(41 + i, 1, "core_collapse", 100.0 + 5.0 * 39.0 + 5.0 + 1.0e6 * i as f64, 9000.0, 0.1)));
    assert!(grade_series(&collapsed).is_ok(), "graded a jump in the collapse");
    collapsed[50].age -= 3.0e6;
    assert!(grade_series(&collapsed).unwrap_err().contains("fell"), "accepted an age that fell in the collapse");
    let before_collapse: Vec<Sample> = steady(40).into_iter().chain((0..2).map(|i| sample(41 + i, 1, "core_collapse", 400.0, 9000.0, 0.1))).collect();
    assert_eq!(until_collapse(&before_collapse).len(), 40, "the collapse's first tick is not the star's life");
    let mut near = steady(40);
    for s in near[25..].iter_mut() {
        s.age += 40.0;
    }
    assert!(grade_series(&near).is_ok(), "refused an increment 9 × the median");
    let mut few = steady(5);
    few[3].age += 5000.0;
    for s in few[4..].iter_mut() {
        s.age += 5000.0;
    }
    assert!(grade_series(&few).is_ok(), "graded a stage of 4 increments");
    let mut other = steady(40);
    for s in other[20..].iter_mut() {
        s.id = 2;
        s.age = 1.0;
    }
    assert!(grade_series(&other).is_ok(), "compared one object's age with another's");
    // A mass jump on its own is no age jump: the rate follows, the series stays steady.
    let mut massed = steady(40);
    for s in massed[20..].iter_mut() {
        s.mass = 12000.0;
    }
    assert!(grade_series(&massed).is_ok(), "refused a steady age under a mass jump");
    // Zero increments are no median: the positive ones are.
    let mut idle: Vec<Sample> = (0..40).map(|i| sample(i + 1, 1, "he_core", 7.0, 9000.0, 0.0)).collect();
    for (k, s) in idle[30..].iter_mut().enumerate() {
        s.age = 7.0 + 2.0 * (k as f64 + 1.0);
    }
    assert!(grade_series(&idle).is_ok(), "refused a stage idle for 30 ticks that then ran steadily");
    let table = shares(&steady(40), &grade_series(&steady(40)).unwrap());
    assert!(table.contains("main_sequence") && table.contains("100.00 %"), "the shares table: {table}");

    let sun = |ending: Value, age: Value, met: bool| serde_json::json!({"until": {"met": met, "object": 3}, "objects": [{"id": 3, "ending": ending, "age_years": age}]});
    assert!(grade_sun(&sun("white_dwarf".into(), SUN_YEARS.into(), true), 0).is_ok(), "refused 1.21e10");
    assert!(grade_sun(&sun("white_dwarf".into(), (SUN_YEARS * 1.49).into(), true), 0).is_ok(), "refused 1.49 ×");
    assert!(grade_sun(&sun("white_dwarf".into(), (SUN_YEARS / 1.49).into(), true), 0).is_ok(), "refused ÷ 1.49");
    assert!(grade_sun(&sun("white_dwarf".into(), (SUN_YEARS * 1.51).into(), true), 0).is_err(), "accepted 1.51 ×");
    assert!(grade_sun(&sun("white_dwarf".into(), (SUN_YEARS / 1.51).into(), true), 0).is_err(), "accepted ÷ 1.51");
    assert!(grade_sun(&sun("white_dwarf".into(), 1.0e9.into(), true), 0).is_err(), "accepted the remnant's age alone");
    assert!(grade_sun(&sun("neutron_star".into(), SUN_YEARS.into(), true), 0).is_err(), "accepted another ending");
    assert!(grade_sun(&sun(Value::Null, SUN_YEARS.into(), true), 0).is_err(), "accepted an ending not held");
    assert!(grade_sun(&sun("white_dwarf".into(), Value::Null, true), 0).is_err(), "accepted no age");
    assert!(grade_sun(&sun("white_dwarf".into(), SUN_YEARS.into(), false), 2).is_err(), "accepted a life that met no ending");
    let other_object = serde_json::json!({"until": {"met": true, "object": 9}, "objects": [{"id": 3, "ending": "white_dwarf", "age_years": SUN_YEARS}]});
    assert!(grade_sun(&other_object, 0).is_err(), "read an age off another object");

    // The edits' grader: paint (the mass up, X_H,c up), then erase (the mass down).
    let mut edited: Vec<Sample> = (0..20).map(|i| sample(i + 1, 1, "main_sequence", 10.0 + i as f64, 9000.0, 0.5 - 0.001 * i as f64)).collect();
    for i in 20..40 {
        edited.push(sample(i + 1, 1, "main_sequence", 10.0 + i as f64, 11000.0, 0.7 - 0.001 * i as f64));
    }
    for i in 40..60 {
        edited.push(sample(i + 1, 1, "main_sequence", 10.0 + i as f64, 7600.0, 0.3 - 0.001 * i as f64));
    }
    let (rise, share) = grade_edits(&edited).expect("the right edits refused");
    assert!(rise > 0.15 && share > 0.3, "the right edits read {rise} {share}");
    let mut no_paint = edited.clone();
    for s in no_paint.iter_mut() {
        s.mass = 9000.0;
    }
    assert!(grade_edits(&no_paint).is_err(), "accepted a life no paint landed in");
    let mut no_rise = edited.clone();
    for s in no_rise.iter_mut() {
        s.x_h_c = 0.4;
    }
    assert!(grade_edits(&no_rise).unwrap_err().contains("never exercised"), "accepted a paint that raised no hydrogen");
    let mut no_erase = edited.clone();
    for s in no_erase[40..].iter_mut() {
        s.mass = 11000.0;
    }
    assert!(grade_edits(&no_erase).unwrap_err().contains("no erase"), "accepted a life no erase landed in");
    let mut shallow = edited.clone();
    for s in shallow[40..].iter_mut() {
        s.mass = 10000.0;
    }
    assert!(grade_edits(&shallow).unwrap_err().contains("took"), "accepted an erase of 9 %");
    println!("graders graded on hand-made values");
}

#[test]
fn the_graders_refuse_a_falling_age_a_jump_and_a_clock_off_the_suns_life() {
    graders_are_graded();
}

#[test]
fn the_age_of_the_massive_star_never_falls_and_never_jumps_under_a_paint_and_an_erase() {
    graders_are_graded();
    let l = touched();
    println!("touch: the edits: {:?}", grade_edits(&l.samples));
    match grade_edits(&l.samples) {
        Ok((rise, share)) => println!("touch: the paint raised X_H,c by {rise:.3}; the erase took {:.1} % of the mass", 100.0 * share),
        Err(e) => panic!("touch: {e}"),
    }
    let stages = grade_series(&l.samples).unwrap_or_else(|e| panic!("touch: {e}"));
    println!("touch: each stage's share of the age: {}", shares(&l.samples, &stages));
    assert!(l.code == 0 && l.summary["until"]["met"] == true, "touch: no ending within the step cap (exit {}, until {})", l.code, l.summary["until"]);
}

#[test]
fn the_sun_like_star_reaches_its_white_dwarf_reading_about_the_age_the_table_sums_to() {
    graders_are_graded();
    let cal = calibration();
    let l = life("sun", "sun", cal.preset_mass_sb.sun, None, "leave");
    let (stages, _) = tally(&l.samples).unwrap_or_else(|e| panic!("sun: {e}"));
    println!("sun: {} steps, exit {}; each stage's share of the age: {}", l.steps, l.code, shares(&l.samples, &stages));
    let age = grade_sun(&l.summary, l.code).unwrap_or_else(|e| panic!("sun: {e}"));
    println!("sun: the white dwarf reads {age:.4e} years ({:.2} × {SUN_YEARS:.2e})", age / SUN_YEARS);
}
