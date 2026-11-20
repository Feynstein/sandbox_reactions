//! M0-T98b — the clocks' order (contract §5.2's G-SQUEEZE; K1–K3, K9, §2.5.1's Q_H row, §0.5 [M0-D169]): on the presets, at the
//! calibration M0-V14's run wrote (`calibration::resolve`, refused when stale — G-CAL), three orders read off whole lives:
//! - **K1, the gaps** — the Sun-like main sequence's τ_KH/τ_dyn ≥ 10 and τ_nuc/τ_KH ≥ 10, τ_dyn = the tracker's √(R³/GM) (the
//!   snapshot's `tau_dyn`), τ_KH = |W|/2L (its `potential` and `luminosity`), τ_nuc = X_H,c · M_core · Q_H / L (its `x_c[0]`; M_core
//!   the mass within 8 cells of its centre, from the checkpoint's Σ plane, M0-D50's reading; Q_H from `physics.json`). Read at a
//!   checkpoint every [`EVERY`] steps of one life, at those showing `main_sequence`, as `probe/clocks.py` does (lives.md § Appendix);
//! - **K2, the burning stages** — the Massive preset's displayed stage durations H > He > C > {Ne, O} > Si;
//! - **K3, heavier burns faster** — the displayed main-sequence durations Sun-like > Massive > Giant.
//!
//! Readings declared (M0-T98b; cheap to reverse):
//! - **The window of τ_nuc/τ_KH**: the main sequence while X_H,c ≥ [`X_H_FLOOR`] (0.35, §1.10.2's Sun anchor, where T_eff is read).
//!   τ_nuc is X_H,c · M_core · Q_H / L: it goes to 0 with the fuel by its own definition, so the gap of 10 cannot hold to the last
//!   cell of hydrogen; the readings under the floor are printed, never graded. τ_KH/τ_dyn is graded over the whole main sequence.
//! - **A stage's duration** is the sim time its displayed stage (`Tick::star.shown`, §1.9.3's held stage, never the raw one) was
//!   shown before the first tick that shows `core_collapse` (the burning life, not the collapse), summed over its visits: the
//!   sim time between two consecutive ticks of the heaviest object is banked to the earlier tick's stage, as the age clock banks it
//!   (age.rs). The Sun-like life is read to its first `he_core`, the Massive's and the Giant's to their `core_collapse`.
//! - **C and Ne**: §0.5 [M0-D169] parenthesised `c_burn` and `ne_burn` in G-STAGES (carbon has no phase of its own: `c_burn` is a
//!   parenthesised stage of the clock physics, reports/heavy_end.md § 2.3). So G-SQUEEZE's chain is graded on the stages the life
//!   displays: `main_sequence`, `he_core`, `o_burn` and `si_burn` are required (as G-STAGES requires them); `c_burn` and `ne_burn`
//!   are graded when they show, and a stage that shows must run shorter than every one before it and longer than every one after
//!   it (strict: a tie is no order). The ones that did not show are printed.
//! - **Orders exact, the gap of 10 precedent** (§5.2): each ratio is read as `≥ 10`, a NaN is red, and the first violation stops the
//!   Sun-like life (the run is cut by an unwind, the readings so far kept), as is a Sun-like cloud that shows `failed_star` before any `main_sequence` — a red is a D or a question, never a looser number.
//!
//! Each case prints the calibration's `physics_hash` and Q_H beside its verdict, and each life its durations and clocks (a run on a
//! stale calibration is refused as `SR-ERROR calibration out of date`, never read as a physics red). The first case grades the
//! graders on hand-made values. The Sun-like life's main sequence is about 130,000 steps (over 10 minutes: R12's ⏱ scope, the lead's
//! run); the Massive's and the Giant's lives are shorter.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use serde_json::{json, Value};
use sr_engine::dump::Dump;
use sr_engine::headless::{observers_path, parse_args, run_scene_captured, Progress, Tick, CHECKPOINT_DIR};
use sr_engine::scene::Scene;
use sr_physics::calibration::{self, Calibration};
use sr_physics::observe::tracker::read_exact;
use sr_physics::registry::Physics;

/// §5.2's gap: each clock ratio at least this (precedent, not proof).
const RATIO_MIN: f64 = 10.0;
/// M0-D50's M_core: the mass within this many cells of the object's centre.
const R_CORE: f64 = 8.0;
/// A checkpoint every this many steps of the Sun-like life.
const EVERY: u64 = 2_000;
/// τ_nuc/τ_KH is graded while the centre keeps at least this much hydrogen (§1.10.2's Sun anchor).
const X_H_FLOOR: f64 = 0.35;
/// A ratio graded over fewer readings than this is no reading of a main sequence.
const MIN_READINGS: usize = 10;

/// The stage chain of §5.2 as G-STAGES' rows read it (§0.5 [M0-D169]): each group's members outlast every member of the next group.
const CHAIN: &[&[&str]] = &[&["main_sequence"], &["he_core"], &["c_burn"], &["ne_burn", "o_burn"], &["si_burn"]];
/// The stages of the chain that must show (G-STAGES' required ones).
const REQUIRED: &[&str] = &["main_sequence", "he_core", "o_burn", "si_burn"];

// ---- the clocks ----

/// One main-sequence checkpoint's clocks.
#[derive(Clone, Debug)]
struct Clocks {
    step: u64,
    sim_time: f64,
    x_h_c: f64,
    mass: f64,
    m_core: f64,
    tau_dyn: f64,
    tau_kh: f64,
    tau_nuc: f64,
}

impl Clocks {
    fn kh_over_dyn(&self) -> f64 {
        self.tau_kh / self.tau_dyn
    }
    fn nuc_over_kh(&self) -> f64 {
        self.tau_nuc / self.tau_kh
    }
    fn line(&self) -> String {
        format!(
            "step={} t={:.1} X_H,c={:.3} M={:.1} M_core={:.1} τ_dyn={:.3} τ_KH={:.2} τ_nuc={:.1} τ_KH/τ_dyn={:.1} τ_nuc/τ_KH={:.1}",
            self.step,
            self.sim_time,
            self.x_h_c,
            self.mass,
            self.m_core,
            self.tau_dyn,
            self.tau_kh,
            self.tau_nuc,
            self.kh_over_dyn(),
            self.nuc_over_kh()
        )
    }
}

/// The clocks of the heaviest object of an observers snapshot, `sigma` the dump's Σ plane (`width` cells a row), `q_h` hydrogen's
/// energy per unit mass.
fn clocks(snapshot: &Value, step: u64, sigma: &[f32], width: usize, q_h: f64) -> Result<Clocks, String> {
    let objects = snapshot["watch"]["tracker"]["objects"].as_array().ok_or("the snapshot has no tracker objects")?;
    let mut best: Option<(f64, &Value)> = None;
    for o in objects {
        let m = read_exact(&o["mass"], "mass")?;
        if best.is_none_or(|(bm, _)| m > bm) {
            best = Some((m, o));
        }
    }
    let (mass, o) = best.ok_or("the snapshot has no object")?;
    let f = |k: &str| read_exact(&o[k], k);
    let (luminosity, potential, tau_dyn) = (f("luminosity")?, f("potential")?, f("tau_dyn")?);
    let centre = o["centre"].as_array().filter(|c| c.len() == 2).ok_or("centre: not two numbers")?;
    let (cx, cy) = (read_exact(&centre[0], "centre[0]")?, read_exact(&centre[1], "centre[1]")?);
    let x_h_c = read_exact(&o["x_c"][0], "x_c[0]")?;
    if !(luminosity > 0.0 && tau_dyn > 0.0) {
        return Err(format!("step {step}: luminosity {luminosity} and τ_dyn {tau_dyn} must be positive for a clock"));
    }
    let m_core: f64 = sigma
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            let (x, y) = ((i % width) as f64 + 0.5, (i / width) as f64 + 0.5);
            (x - cx).hypot(y - cy) <= R_CORE
        })
        .map(|(_, s)| f64::from(*s))
        .sum();
    Ok(Clocks {
        step,
        sim_time: read_exact(&snapshot["sim_time"], "sim_time")?,
        x_h_c,
        mass,
        m_core,
        tau_dyn,
        tau_kh: potential.abs() / (2.0 * luminosity),
        tau_nuc: x_h_c * m_core * q_h / luminosity,
    })
}

/// §5.2's gaps at one reading: τ_KH/τ_dyn always, τ_nuc/τ_KH while the centre has [`X_H_FLOOR`] of hydrogen; a NaN is red.
fn gaps_hold(c: &Clocks) -> Result<(), String> {
    let (kh, nuc) = (c.kh_over_dyn(), c.nuc_over_kh());
    if !(kh >= RATIO_MIN) {
        return Err(format!("τ_KH/τ_dyn {kh:.2} < {RATIO_MIN}: {}", c.line()));
    }
    if c.x_h_c >= X_H_FLOOR && !(nuc >= RATIO_MIN) {
        return Err(format!("τ_nuc/τ_KH {nuc:.2} < {RATIO_MIN} at X_H,c {:.3} ≥ {X_H_FLOOR}: {}", c.x_h_c, c.line()));
    }
    Ok(())
}

// ---- the stages ----

/// A tick of the heaviest object: its sim time and the stage it shows.
#[derive(Clone, Copy, Debug)]
struct Sample {
    t: f64,
    shown: Option<&'static str>,
}

/// Each stage's displayed sim time before the first tick showing `core_collapse`: the time between two consecutive ticks banked to
/// the earlier tick's stage.
fn durations(samples: &[Sample]) -> BTreeMap<&'static str, f64> {
    let mut d = BTreeMap::new();
    for w in samples.windows(2) {
        let Some(stage) = w[0].shown else { continue };
        if stage == "core_collapse" {
            break;
        }
        *d.entry(stage).or_insert(0.0) += w[1].t - w[0].t;
    }
    d
}

/// The chain of §5.2 on the durations of one life: every required stage shows, and a stage of a group runs longer than every stage of
/// every later group that shows (strictly).
fn chain_holds(d: &BTreeMap<&'static str, f64>) -> Result<(), String> {
    for r in REQUIRED {
        if d.get(r).is_none_or(|v| !(*v > 0.0)) {
            return Err(format!("{r} never showed before the core collapse: {d:?}"));
        }
    }
    let shown: Vec<Vec<(&str, f64)>> = CHAIN
        .iter()
        .map(|g| g.iter().filter_map(|s| d.get(s).filter(|v| **v > 0.0).map(|v| (*s, *v))).collect::<Vec<_>>())
        .filter(|g| !g.is_empty())
        .collect();
    for (i, earlier) in shown.iter().enumerate() {
        for later in &shown[i + 1..] {
            for &(a, da) in earlier {
                for &(b, db) in later {
                    if !(da > db) {
                        return Err(format!("{a} ({da:.2} t.u.) does not outlast {b} ({db:.2} t.u.): {d:?}"));
                    }
                }
            }
        }
    }
    Ok(())
}

/// K3: the main sequences Sun-like > Massive > Giant, each shown, strictly.
fn main_sequences_hold(sun: f64, massive: f64, giant: f64) -> Result<(), String> {
    for (name, v) in [("Sun-like", sun), ("Massive", massive), ("Giant", giant)] {
        if !(v > 0.0) {
            return Err(format!("the {name} preset never showed main_sequence ({v})"));
        }
    }
    if !(sun > massive && massive > giant) {
        return Err(format!("main sequences Sun-like {sun:.2} > Massive {massive:.2} > Giant {giant:.2} t.u. do not hold"));
    }
    Ok(())
}

// ---- the lives ----

/// What a life gave: the exit code, the heaviest object's ticks, the main-sequence checkpoints' clocks and, when a gap failed, the
/// unwind that cut the run.
struct Life {
    id: &'static str,
    code: i32,
    steps: u64,
    samples: Vec<Sample>,
    readings: Vec<Clocks>,
    aborted: Option<String>,
}

fn calibration() -> Calibration {
    calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration")
}

fn q_h() -> f64 {
    Physics::shipped().expect("the shipped constants").get("q_h").expect("q_h is a physics.json key")
}

/// `preset` at calibration's mass to its displayed `last` stage (or `--max-steps`, exit 2) on the physics adapter; with `gaps` the
/// Sun-like checkpoints' clocks read each [`EVERY`] steps and the first broken gap, or a `failed_star` before any `main_sequence`, cuts the run.
fn run_life(id: &'static str, preset: &str, last: &str, max_steps: u64, gaps: bool) -> Life {
    let cal = calibration();
    let mass = match preset {
        "sun" => cal.preset_mass_sb.sun,
        "massive" => cal.preset_mass_sb.massive,
        _ => cal.preset_mass_sb.giant,
    };
    let q = q_h();
    println!("{id}: preset {preset} at {mass:.1}; calibration physics_hash {}; Q_H {q}", cal.physics_hash);
    let scene = Scene::preset_mass(preset, mass, Default::default()).unwrap();
    let out = std::env::temp_dir().join(format!("sr-squeeze-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let (max, until, out_dir, label) = (max_steps.to_string(), format!("stage:{last}"), out.display().to_string(), format!("squeeze:{id}"));
    let mut args: Vec<String> =
        ["--scene", &label, "--until", &until, "--max-steps", &max, "--out", &out_dir].map(String::from).to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let samples = RefCell::new(Vec::<Sample>::new());
    let readings = RefCell::new(Vec::<Clocks>::new());
    let last_step = RefCell::new(0u64);
    let mut tick = |t: &Tick| {
        *last_step.borrow_mut() = t.step;
        let Some(star) = &t.star else { return };
        let mut s = samples.borrow_mut();
        if s.last().is_none_or(|p| p.shown != star.shown) {
            println!("{id} step={} t={:.2} shown={} M={:.1} X_H,c={:.3}", t.step, t.sim_time, star.shown.unwrap_or("none"), star.mass, star.x_h_c);
        }
        s.push(Sample { t: t.sim_time, shown: star.shown });
        // A Sun-like cloud that fails before it has shone has no main sequence to read the clocks of: cut at once, not at the step cap.
        let shone = s.iter().any(|p| p.shown == Some("main_sequence"));
        if gaps && !shone && star.shown == Some("failed_star") {
            drop(s);
            panic!("cut: failed_star shown at step {} (t {:.2}) before any main_sequence", t.step, t.sim_time);
        }
    };
    let mut saved = |step: u64, _t: f64, stage: &str| {
        if !gaps || stage != "main_sequence" {
            return;
        }
        let bin = out.join(CHECKPOINT_DIR).join(format!("s_{step}.bin"));
        let snapshot: Value = serde_json::from_str(&std::fs::read_to_string(observers_path(&bin)).expect("the checkpoint's snapshot"))
            .expect("the snapshot parses");
        let dump = Dump::read(&bin).unwrap_or_else(|e| panic!("{id}: the checkpoint {}: {e}", bin.display()));
        let cells = dump.world.cells();
        let c = clocks(&snapshot, step, &dump.planes[..cells], dump.world.width as usize, q).unwrap_or_else(|e| panic!("{id}: {e}"));
        println!("{id} {}", c.line());
        let verdict = gaps_hold(&c);
        readings.borrow_mut().push(c);
        if let Err(e) = verdict {
            panic!("cut: gap broken: {e}");
        }
    };
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: gaps.then_some(EVERY), saved: &mut saved };
    let run = catch_unwind(AssertUnwindSafe(|| run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress))));
    let _ = std::fs::remove_dir_all(&out);
    let (code, aborted) = match run {
        Ok((code, lines)) => {
            assert!(code == 0 || code == 2, "{id}: the run ended with exit {code}: {lines:?}");
            (code, None)
        }
        Err(p) => {
            let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
            assert!(msg.starts_with("cut:"), "{id}: the run panicked: {msg}");
            (-1, Some(msg))
        }
    };
    let (samples, readings, steps) = (samples.into_inner(), readings.into_inner(), last_step.into_inner());
    println!("{id}: {steps} steps, exit {code}, durations (t.u.) {:?}", durations(&samples));
    Life { id, code, steps, samples, readings, aborted }
}

fn sun() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| run_life("sun", "sun", "he_core", 320_000, true))
}

fn massive() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| run_life("massive", "massive", "core_collapse", 120_000, false))
}

fn giant() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| run_life("giant", "giant", "core_collapse", 120_000, false))
}

/// The verdict line every case prints beside the calibration it read.
fn verdict(case: &str, text: &str) {
    println!("{case}: {text} (calibration physics_hash {}, Q_H {})", calibration().physics_hash, q_h());
}

/// A Sun-like life's readings for one ratio: a broken gap, a life that did not reach its `he_core`, or too few readings are red.
fn grade_gap(l: &Life, name: &str, graded: impl Fn(&Clocks) -> bool, ratio: impl Fn(&Clocks) -> f64) {
    let rows: Vec<&Clocks> = l.readings.iter().filter(|c| graded(c)).collect();
    let worst = rows.iter().copied().min_by(|a, b| ratio(a).total_cmp(&ratio(b)));
    if let Some(w) = worst {
        println!("{}: {name} over {} readings of {} (steps {} → {}): worst {:.2} at step {} (t {:.1}, X_H,c {:.3})",
            l.id, rows.len(), l.readings.len(), rows[0].step, rows[rows.len() - 1].step, ratio(w), w.step, w.sim_time, w.x_h_c);
    }
    for c in &rows {
        assert!(ratio(c) >= RATIO_MIN, "{}: {name} {:.2} < {RATIO_MIN} at step {}: {}", l.id, ratio(c), c.step, c.line());
    }
    assert!(l.aborted.is_none(), "{}: the run was cut: {:?}", l.id, l.aborted);
    assert_eq!(l.code, 0, "{}: the main sequence never ended in he_core within the step cap ({} steps, {} readings)", l.id, l.steps, l.readings.len());
    assert!(rows.len() >= MIN_READINGS, "{}: {name} read at {} checkpoints, fewer than the {MIN_READINGS} a main sequence gives", l.id, rows.len());
    verdict(&format!("sun {name}"), &format!("{} readings, all ≥ {RATIO_MIN}, worst {:.2}", rows.len(), worst.map_or(f64::NAN, &ratio)));
}

// ---- the graders on hand-made values ----

fn graders_are_graded() {
    // A 40 × 40 Σ plane with 5 at cell (20, 20) and 7 at (35, 35); the object's centre (20.5, 20.5): M_core is 5, the 7 out of reach.
    let mut sigma = vec![0.0f32; 1600];
    sigma[20 * 40 + 20] = 5.0;
    sigma[35 * 40 + 35] = 7.0;
    let snap = |l: &str, w: &str, x: &str| {
        json!({"sim_time": "12.5", "watch": {"tracker": {"objects": [
            {"mass": "10.0", "luminosity": "1.0", "potential": "-1.0", "tau_dyn": "1.0", "centre": ["30.5", "30.5"], "x_c": ["0.9"]},
            {"mass": "100.0", "luminosity": l, "potential": w, "tau_dyn": "0.5", "centre": ["20.5", "20.5"], "x_c": [x]}]}}})
    };
    // τ_KH = 80/8 = 10, τ_KH/τ_dyn = 20, τ_nuc = 0.5 · 5 · 200 / 4 = 125, τ_nuc/τ_KH = 12.5.
    let c = clocks(&snap("4.0", "-80.0", "0.5"), 7, &sigma, 40, 200.0).unwrap();
    assert_eq!((c.mass, c.m_core, c.tau_dyn, c.tau_kh, c.tau_nuc, c.step), (100.0, 5.0, 0.5, 10.0, 125.0, 7), "the clocks of the heaviest object: {c:?}");
    assert!((c.kh_over_dyn() - 20.0).abs() < 1e-12 && (c.nuc_over_kh() - 12.5).abs() < 1e-12, "the ratios: {c:?}");
    assert!(gaps_hold(&c).is_ok(), "both gaps over 10 refused");
    let low_q = clocks(&snap("4.0", "-80.0", "0.5"), 7, &sigma, 40, 100.0).unwrap();
    assert!(gaps_hold(&low_q).is_err(), "τ_nuc/τ_KH 6.25 accepted (Q_H halved)");
    let shallow = clocks(&snap("4.0", "-8.0", "0.5"), 7, &sigma, 40, 200.0).unwrap();
    assert!(gaps_hold(&shallow).is_err(), "τ_KH/τ_dyn 2 accepted");
    let spent = clocks(&snap("4.0", "-80.0", "0.3"), 7, &sigma, 40, 100.0).unwrap();
    assert!(gaps_hold(&spent).is_ok(), "τ_nuc/τ_KH 3.75 graded under X_H,c 0.35: the fuel is going, not the star");
    assert!(gaps_hold(&Clocks { tau_nuc: f64::NAN, ..c.clone() }).is_err(), "a NaN τ_nuc accepted");
    assert!(gaps_hold(&Clocks { tau_kh: f64::NAN, ..c.clone() }).is_err(), "a NaN τ_KH accepted");
    assert!(clocks(&snap("0.0", "-80.0", "0.5"), 7, &sigma, 40, 200.0).is_err(), "no luminosity read as a clock");
    assert!(clocks(&json!({"watch": {"tracker": {"objects": []}}}), 7, &sigma, 40, 200.0).is_err(), "no object read as a clock");

    // The stage clocks: the time between ticks banked to the earlier tick's stage, to the first core_collapse.
    let s = |t, shown| Sample { t, shown };
    let ticks = [s(0.0, Some("a")), s(2.0, Some("b")), s(5.0, Some("a")), s(6.0, None), s(7.0, Some("core_collapse")), s(9.0, Some("a"))];
    let d = durations(&ticks);
    assert_eq!((d["a"], d["b"], d.len()), (2.0 + 1.0, 3.0, 2), "durations: {d:?} (the gap after a tick with no stage, the core collapse's tail left out)");

    let life = |v: [f64; 6]| {
        ["main_sequence", "he_core", "c_burn", "ne_burn", "o_burn", "si_burn"]
            .into_iter()
            .zip(v)
            .filter(|(_, d)| *d != 0.0)
            .collect::<BTreeMap<_, _>>()
    };
    // H > He > C > {Ne, O} > Si: the order, with the parenthesised C and Ne shown and not.
    assert!(chain_holds(&life([150.0, 50.0, 8.0, 3.0, 4.0, 2.0])).is_ok(), "the full order refused");
    assert!(chain_holds(&life([150.0, 50.0, 0.0, 0.0, 28.0, 18.0])).is_ok(), "the order without C and Ne refused (M0-D169)");
    assert!(chain_holds(&life([150.0, 50.0, 0.0, 5.0, 4.0, 2.0])).is_ok(), "Ne shown without C refused");
    assert!(chain_holds(&life([150.0, 50.0, 0.0, 0.0, 18.0, 28.0])).is_err(), "Si outlasting O accepted");
    assert!(chain_holds(&life([50.0, 150.0, 0.0, 0.0, 28.0, 18.0])).is_err(), "He outlasting H accepted");
    assert!(chain_holds(&life([150.0, 20.0, 0.0, 0.0, 28.0, 18.0])).is_err(), "O outlasting He accepted");
    assert!(chain_holds(&life([150.0, 50.0, 3.0, 0.0, 28.0, 18.0])).is_err(), "O outlasting C accepted");
    assert!(chain_holds(&life([150.0, 50.0, 8.0, 9.0, 4.0, 2.0])).is_err(), "Ne outlasting C accepted");
    assert!(chain_holds(&life([150.0, 50.0, 0.0, 0.0, 18.0, 18.0])).is_err(), "a tie of O and Si accepted");
    assert!(chain_holds(&life([150.0, 50.0, 0.0, 0.0, 28.0, 0.0])).is_err(), "no si_burn accepted");
    assert!(chain_holds(&life([0.0, 50.0, 0.0, 0.0, 28.0, 18.0])).is_err(), "no main_sequence accepted");

    // K3.
    assert!(main_sequences_hold(1500.0, 150.0, 5.0).is_ok(), "the order refused");
    assert!(main_sequences_hold(150.0, 1500.0, 5.0).is_err(), "Massive over Sun-like accepted");
    assert!(main_sequences_hold(1500.0, 5.0, 150.0).is_err(), "Giant over Massive accepted");
    assert!(main_sequences_hold(1500.0, 150.0, 150.0).is_err(), "a tie accepted");
    assert!(main_sequences_hold(1500.0, 150.0, 0.0).is_err(), "a Giant with no main sequence accepted");
}

#[test]
fn the_graders_are_graded_on_hand_made_values() {
    graders_are_graded();
    verdict("graders", "graded on hand-made values");
}

#[test]
fn the_sun_like_main_sequence_keeps_tau_kh_over_tau_dyn_at_ten() {
    graders_are_graded();
    grade_gap(sun(), "τ_KH/τ_dyn", |_| true, Clocks::kh_over_dyn);
}

#[test]
fn the_sun_like_main_sequence_keeps_tau_nuc_over_tau_kh_at_ten() {
    graders_are_graded();
    grade_gap(sun(), "τ_nuc/τ_KH", |c| c.x_h_c >= X_H_FLOOR, Clocks::nuc_over_kh);
}

#[test]
fn the_massive_stage_durations_run_h_he_c_ne_o_si() {
    graders_are_graded();
    let l = massive();
    let d = durations(&l.samples);
    let absent: Vec<&str> = CHAIN.iter().flat_map(|g| g.iter().copied()).filter(|s| !d.contains_key(s)).collect();
    println!("massive: durations before the core collapse {d:?}; not shown (parenthesised, §0.5 [M0-D169]): {absent:?}");
    assert_eq!(l.code, 0, "massive: the life never showed core_collapse in {} steps: {d:?}", l.steps);
    if let Err(e) = chain_holds(&d) {
        panic!("massive: {e}");
    }
    verdict("massive", &format!("H > He > C > {{Ne, O}} > Si over {d:?}"));
}

#[test]
fn the_main_sequences_run_sun_like_massive_giant() {
    graders_are_graded();
    let ms = |l: &Life| {
        assert_eq!(l.code, 0, "{}: the life ended with exit {} after {} steps (cut: {:?})", l.id, l.code, l.steps, l.aborted);
        durations(&l.samples).get("main_sequence").copied().unwrap_or(0.0)
    };
    let (s, m, g) = (ms(sun()), ms(massive()), ms(giant()));
    println!("main sequences (t.u.): Sun-like {s:.2}, Massive {m:.2}, Giant {g:.2}");
    if let Err(e) = main_sequences_hold(s, m, g) {
        panic!("{e}");
    }
    verdict("main sequences", &format!("Sun-like {s:.2} > Massive {m:.2} > Giant {g:.2} t.u."));
}
