//! M0-T106 — conservation over whole lives (contract §5.1's G-CONS; §0.4, §2.9): the ledger's three invariants, read along two
//! whole lives — the Sun-like preset in a world whose edges bounce, to its white dwarf, and a supernova in a world whose edges
//! leave, the Massive preset from its iron-core dump to its neutron star:
//! - mass: grid + sinks + escaped + erased + vacuum_reset − painted − preset_dropped constant to 10⁻⁶ relative to its start;
//! - momentum: grid + sinks + escaped + held constant to 10⁻⁵ of the running peak Σm|u| (§0.5 [M0-D186]: f32's floor with every route booked);
//! - energy: the invariant of §2.9 drifting at most 10⁻³ |W| per τ_dyn (`UNVERIFIED`: source-term gravity conserves it only
//!   approximately; a red here is a D for a conservative gravity formulation, never a looser bound, §0.4).
//!
//! Where the numbers come from. summary.json's `ledger` holds the start and the end of a run only, so a life is read along its way:
//! the run's checkpoints (`Progress::checkpoint_every`, §0.5 [M0-D77]: a dump and the observers' snapshot, saved at a frame boundary
//! — no chunk is shortened, nothing the life computes moves) each carry the ledger's latest record (`ledger.last`, every f64 [`exact`])
//! and the dump's planes, which give Σm|u|. A life is the records at each checkpoint after the ledger's start (the run's first step).
//!
//! The grades, and what each reads (mine where the contract is silent, and declared):
//! - mass: at each checkpoint |invariant − the start's| / the start's — the strictest reading;
//! - momentum: at each checkpoint |p − p₀| (the vector's length) over **the largest Σm|u| the life has had so far** (grid planes from
//!   the checkpoint's dump and the sinks' M|v|). A life that starts at rest has no scale at its start and a quiet main sequence
//!   a tiny one beside its collapse: the instant Σm|u| would grade a correct integrator red on f32 noise, and the running peak is the
//!   momentum the run has had to conserve. Both readings are printed, the instant one is not graded;
//! - energy: between two consecutive records, |Δ invariant| / (|W| × Δt / τ_dyn) ≤ 10⁻³, with |W| the smaller of the two records'
//!   (the strictest) and τ_dyn the largest the heaviest object's readout showed in the window (a longer τ_dyn is fewer of them in the
//!   window — the strictest).
//!
//! Each case prints the table beside the drift: every energy term's change over the worst window and over the life (a missing booked
//! term — escaped, swallowed — shows up as drift to be blamed on gravity; the terms say where it is not). The grader is graded on
//! hand-made records before any life. A life is long (the Sun-like's about 179,000 steps, over ten minutes on the RTX 4090): R12's
//! ⏱ scope, the lead's run.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde_json::Value;
use sr_engine::dump::Dump;
use sr_engine::headless::{observers_path, parse_args, run_scene_captured, Progress, Tick, CHECKPOINT_DIR};
use sr_engine::scene::Scene;
use sr_physics::calibration;
use sr_physics::observe::tracker::read_exact;

/// §5.1's mass bound, relative to the start's.
const MASS_BAND: f64 = 1e-6;
/// §5.1's momentum bound, of the running peak Σm|u| (§0.5 [M0-D186], the lead's ruling: 10⁻⁶ → 10⁻⁵).
const MOMENTUM_BAND: f64 = 1e-5;
/// §5.1's energy bound: |W| per τ_dyn.
const CONS_PER_TAU: f64 = 1e-3;

/// The momentum invariant's terms (§2.9).
const MOMENTUM_TERMS: [&str; 4] = ["grid", "sinks", "escaped", "held"];

/// The energy invariant's terms and the sign each carries in it (§2.9).
const ENERGY_TERMS: [(&str, f64); 13] = [
    ("kinetic", 1.0),
    ("thermal", 1.0),
    ("cold", 1.0),
    ("potential", 1.0),
    ("radiated", 1.0),
    ("neutrino_lost", 1.0),
    ("neutrino_deposited", -1.0),
    ("escaped", 1.0),
    ("swallowed", 1.0),
    ("nuclear_released", -1.0),
    ("floor_added", -1.0),
    ("tools", -1.0),
    ("edit_potential", -1.0),
];

/// One ledger record (§2.9), the terms in f64 and the invariants recomputed from them, never read back.
#[derive(Clone, Debug, PartialEq)]
struct Rec {
    step: u64,
    t: f64,
    mass: f64,
    /// Each energy term, signed as it enters the invariant.
    energy: Vec<(&'static str, f64)>,
    /// ½Σmφ, |W|'s signed value.
    w: f64,
    momentum: [f64; 2],
    /// The momentum terms (grid, sinks, escaped, held), each (x, y).
    mom_terms: [[f64; 2]; 4],
}

impl Rec {
    fn energy_invariant(&self) -> f64 {
        self.energy.iter().map(|(_, v)| v).sum()
    }
}

/// A record as the observers' snapshot writes it (every f64 a string, [`read_exact`]).
fn rec_from(v: &Value) -> Rec {
    let n = |group: &str, k: &str| read_exact(&v[group][k], &format!("ledger.{group}.{k}")).unwrap_or_else(|e| panic!("{e}"));
    let pair = |k: &str| -> [f64; 2] {
        let at = format!("ledger.momentum.{k}");
        [0, 1].map(|i| read_exact(&v["momentum"][k][i], &at).unwrap_or_else(|e| panic!("{e}")))
    };
    let mass = ["grid", "sinks", "escaped", "erased", "vacuum_reset"].iter().map(|k| n("mass", k)).sum::<f64>()
        - n("mass", "painted")
        - n("mass", "preset_dropped");
    let mom_terms = MOMENTUM_TERMS.map(pair);
    let momentum = [0, 1].map(|i| mom_terms.iter().map(|t| t[i]).sum());
    let energy = ENERGY_TERMS.iter().map(|&(name, sign)| (name, sign * n("energy", name))).collect();
    Rec {
        step: v["step"].as_u64().expect("a record's step"),
        t: read_exact(&v["sim_time"], "ledger.sim_time").unwrap_or_else(|e| panic!("{e}")),
        mass,
        energy,
        w: n("energy", "potential"),
        momentum,
        mom_terms,
    }
}

/// A checkpoint: the ledger's record there, the momentum's scale there and the largest τ_dyn the readout showed since the one before.
#[derive(Clone, Debug)]
struct Point {
    rec: Rec,
    /// Σm|u| at this checkpoint: the grid's Σ|σu| and the sinks' M|v|.
    scale: f64,
    /// The largest τ_dyn of the heaviest object since the previous point (NaN if the readout showed none).
    tau_dyn: f64,
}

/// A life as read along its way.
struct Life {
    id: &'static str,
    /// The ledger's start record and then each checkpoint's.
    start: Rec,
    points: Vec<Point>,
    code: i32,
    summary: Value,
    /// Every shown stage the heaviest object had (ticks).
    stages: BTreeSet<String>,
    steps: u64,
}

/// Σm|u| of a checkpoint's dump: the grid's Σ|σu| and the sinks' M|v|.
fn momentum_scale(dump: &Dump) -> f64 {
    let cells = dump.world.cells();
    let grid: f64 = (0..cells).map(|i| f64::from(dump.planes[cells + i]).hypot(f64::from(dump.planes[2 * cells + i]))).sum();
    grid + dump.sinks.iter().map(|s| s.m * s.vx.hypot(s.vy)).sum::<f64>()
}

/// A life run in-process on the physics adapter, the checkpoints read each `every` steps.
fn run_life(id: &'static str, scene: Scene, extra: &[&str], every: u64, max_steps: u64) -> Life {
    let out = std::env::temp_dir().join(format!("sr-conservation-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let max = max_steps.to_string();
    let label = format!("conservation:{id}");
    let out_dir = out.display().to_string();
    let mut args: Vec<String> =
        ["--scene", &label, "--until", "ending", "--max-steps", &max, "--out", &out_dir].map(String::from).to_vec();
    args.extend(extra.iter().map(|s| s.to_string()));
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let (last_tau, window_tau) = (Cell::new(f64::NAN), Cell::new(f64::NAN));
    let (last_step, frames) = (Cell::new(0u64), Cell::new(0u64));
    let stages = RefCell::new(BTreeSet::<String>::new());
    let points = RefCell::new(Vec::<Point>::new());
    let start = RefCell::new(None::<Rec>);
    let mut tick = |t: &Tick| {
        last_step.set(t.step);
        frames.set(frames.get() + 1);
        if t.stage != "none" {
            stages.borrow_mut().insert(t.stage.to_string());
        }
        if let Some(s) = &t.star {
            last_tau.set(s.tau_dyn);
            window_tau.set(window_tau.get().max(s.tau_dyn));
        }
        if frames.get().is_multiple_of(4000) {
            println!("{id} step={} t={:.2} stage={}", t.step, t.sim_time, t.stage);
        }
    };
    let mut saved = |step: u64, _t: f64, _stage: &str| {
        let folder = out.join(CHECKPOINT_DIR);
        let bin = folder.join(format!("s_{step}.bin"));
        let snapshot: Value = serde_json::from_str(&std::fs::read_to_string(observers_path(&bin)).expect("the checkpoint's snapshot"))
            .expect("the snapshot parses");
        let dump = Dump::read(&bin).unwrap_or_else(|e| panic!("{id}: the checkpoint {}: {e}", bin.display()));
        if start.borrow().is_none() {
            *start.borrow_mut() = Some(rec_from(&snapshot["ledger"]["start"]));
        }
        points.borrow_mut().push(Point {
            rec: rec_from(&snapshot["ledger"]["last"]),
            scale: momentum_scale(&dump),
            tau_dyn: window_tau.get(),
        });
        // The next window opens with the readout the checkpoint stands on.
        window_tau.set(last_tau.get());
    };
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: Some(every), saved: &mut saved };
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, Some(progress));
    assert!(code == 0 || code == 2, "{id}: the run ended with exit {code}: {lines:?}");
    let text = std::fs::read_to_string(out.join("summary.json")).unwrap_or_else(|e| panic!("{id}: no summary.json: {e}"));
    let summary: Value = serde_json::from_str(&text).expect("summary.json parses");
    let _ = std::fs::remove_dir_all(&out);
    let start = start.into_inner().unwrap_or_else(|| panic!("{id}: no checkpoint was saved in {} steps (every {every})", last_step.get()));
    Life { id, start, points: points.into_inner(), code, summary, stages: stages.into_inner(), steps: last_step.get() }
}

// ---- the grader, on plain records ----

/// The mass invariant's drift from the start, relative to the start's, at `p`.
fn mass_drift(start: &Rec, p: &Rec) -> f64 {
    (p.mass - start.mass).abs() / start.mass.abs()
}

/// The momentum invariant's drift from the start (the vector's length) over `scale`.
fn momentum_drift(start: &Rec, p: &Rec, scale: f64) -> f64 {
    ((p.momentum[0] - start.momentum[0]).powi(2) + (p.momentum[1] - start.momentum[1]).powi(2)).sqrt() / scale
}

/// The energy invariant's drift between two records in |W| per τ_dyn: |Δ| / (|W| × Δt / τ_dyn), |W| the smaller of the two records'.
fn energy_per_tau(a: &Rec, b: &Rec, tau_dyn: f64) -> f64 {
    let d = (b.energy_invariant() - a.energy_invariant()).abs();
    let w = a.w.abs().min(b.w.abs());
    d / w / ((b.t - a.t) / tau_dyn)
}

/// The worst of `(value, step)`: the largest value, a NaN the worst of all.
fn worst(values: impl Iterator<Item = (f64, u64)>) -> (f64, u64) {
    values.fold((0.0, 0), |w, v| if v.0.is_nan() || v.0 > w.0 { v } else { w })
}

/// The whole life's mass drift: the worst over the checkpoints.
fn grade_mass(l: &Life) -> (f64, u64) {
    worst(l.points.iter().map(|p| (mass_drift(&l.start, &p.rec), p.rec.step)))
}

/// The whole life's momentum drift over the running peak of Σm|u|, and over the instant one (reported only).
fn grade_momentum(l: &Life) -> ((f64, u64), (f64, u64)) {
    let mut peak = 0.0f64;
    let mut over_peak = Vec::new();
    for p in &l.points {
        peak = peak.max(p.scale);
        over_peak.push((momentum_drift(&l.start, &p.rec, peak), p.rec.step));
    }
    let instant = worst(l.points.iter().map(|p| (momentum_drift(&l.start, &p.rec, p.scale), p.rec.step)));
    (worst(over_peak.into_iter()), instant)
}

/// The energy windows: (the drift per τ_dyn, the window's end step, its start record, its end record).
fn windows(l: &Life) -> Vec<(f64, &Rec, &Rec)> {
    let mut out = Vec::new();
    let mut a = &l.start;
    let mut tau_before = f64::NAN;
    for p in &l.points {
        // The readout of the window's own ticks, else the one before it (a window with no object shown).
        let tau = if p.tau_dyn.is_nan() { tau_before } else { p.tau_dyn };
        out.push((energy_per_tau(a, &p.rec, tau), a, &p.rec));
        if !p.tau_dyn.is_nan() {
            tau_before = p.tau_dyn;
        }
        a = &p.rec;
    }
    out
}

fn grade_energy(l: &Life) -> (f64, &Rec, &Rec) {
    let ws = windows(l);
    let (mut worst_w, mut at) = (0.0, 0usize);
    for (i, (v, ..)) in ws.iter().enumerate() {
        if v.is_nan() || *v > worst_w {
            (worst_w, at) = (*v, i);
        }
    }
    (worst_w, ws[at].1, ws[at].2)
}

/// Every energy term's change between two records, signed as in the invariant, one line each, the biggest first.
fn term_table(a: &Rec, b: &Rec) -> String {
    let mut terms: Vec<(&str, f64)> = a.energy.iter().zip(&b.energy).map(|((name, x), (_, y))| (*name, y - x)).collect();
    terms.sort_by(|p, q| q.1.abs().total_cmp(&p.1.abs()));
    let rows: Vec<String> = terms.iter().map(|(n, d)| format!("{n} {d:+.4e}")).collect();
    format!("steps {}..{}, Δinvariant {:+.4e}: {}", a.step, b.step, b.energy_invariant() - a.energy_invariant(), rows.join(" · "))
}

/// Every momentum term's change between two records (x, y), the invariant's beside them.
fn momentum_table(a: &Rec, b: &Rec) -> String {
    let rows: Vec<String> = MOMENTUM_TERMS
        .iter()
        .zip(a.mom_terms.iter().zip(&b.mom_terms))
        .map(|(n, (x, y))| format!("{n} ({:+.4e}, {:+.4e})", y[0] - x[0], y[1] - x[1]))
        .collect();
    format!("steps {}..{}, Δinvariant ({:+.4e}, {:+.4e}): {}", a.step, b.step, b.momentum[0] - a.momentum[0], b.momentum[1] - a.momentum[1], rows.join(" · "))
}

/// Whether the life met its ending and which object's ending it was: summary.json's `until.met` and that object's `ending`.
fn ending_of(summary: &Value, code: i32) -> Result<String, String> {
    if code != 0 || summary["until"]["met"] != true {
        return Err(format!("no ending within the step cap (exit {code}, until {})", summary["until"]));
    }
    let object = &summary["until"]["object"];
    summary["objects"]
        .as_array()
        .and_then(|os| os.iter().find(|o| o["id"] == *object))
        .and_then(|o| o["ending"].as_str().map(String::from))
        .ok_or_else(|| format!("the ending object {object} holds no ended remnant (objects {})", summary["objects"]))
}

/// A life is what it is named: it ended as `ending`, the heaviest object showed each of `stages`, the ending's object had an event of each
/// kind in `events` (a supernova is an event, not a stage the readout holds), and it was read at `min_points` checkpoints or more, both
/// stand-ins off.
fn check_life(l: &Life, ending: &str, stages: &[&str], events: &[&str], min_points: usize) -> Result<(), String> {
    let got = ending_of(&l.summary, l.code)?;
    if got != ending {
        return Err(format!("ended {got}, not {ending}"));
    }
    for s in stages {
        if !l.stages.contains(*s) {
            return Err(format!("the heaviest object never showed {s} (saw {:?})", l.stages));
        }
    }
    let object = &l.summary["until"]["object"];
    for kind in events {
        let seen = l.summary["events"].as_array().is_some_and(|es| es.iter().any(|e| e["kind"] == *kind && e["object"] == *object));
        if !seen {
            return Err(format!("no {kind} event of the ending's object {object} (events {})", l.summary["events"]));
        }
    }
    if l.points.len() < min_points {
        return Err(format!("{} checkpoints, fewer than the {min_points} a life of this length gives", l.points.len()));
    }
    if l.summary["standins"] != serde_json::json!({"s1": false, "s2": false}) {
        return Err(format!("a stand-in was on (standins {})", l.summary["standins"]));
    }
    Ok(())
}

fn assert_life_is(l: &Life, ending: &str, stages: &[&str], events: &[&str], min_points: usize) {
    if let Err(e) = check_life(l, ending, stages, events, min_points) {
        panic!("{}: {e}", l.id);
    }
}

fn report(l: &Life) {
    let last = l.points.last().expect("a checkpoint");
    println!(
        "{}: {} steps, {} checkpoints, t {:.2} → {:.2}; edge {}; invariants at the end: mass {:.6e} (start {:.6e}), momentum ({:+.4e}, {:+.4e}) \
         (start ({:+.4e}, {:+.4e})), energy {:+.6e} (start {:+.6e}); |W| {:.4e}",
        l.id,
        l.steps,
        l.points.len(),
        l.start.t,
        last.rec.t,
        l.summary["world"]["edge"],
        last.rec.mass,
        l.start.mass,
        last.rec.momentum[0],
        last.rec.momentum[1],
        l.start.momentum[0],
        l.start.momentum[1],
        last.rec.energy_invariant(),
        l.start.energy_invariant(),
        last.rec.w.abs(),
    );
}

// ---- the two lives, each run once and graded by three cases ----

static SUN: OnceLock<Life> = OnceLock::new();
static SUPERNOVA: OnceLock<Life> = OnceLock::new();

/// The Sun-like preset at calibration's mass in a bounce world to its white dwarf; a checkpoint each 1,000 steps (about 180 windows).
fn sun() -> &'static Life {
    SUN.get_or_init(|| {
        let cal = calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration");
        let scene = Scene::preset_mass("sun", cal.preset_mass_sb.sun, Default::default()).unwrap();
        let l = run_life("sun", scene, &["--edge", "bounce"], 1_000, 400_000);
        report(&l);
        l
    })
}

/// The Massive preset at calibration's mass in a leave world, its whole life to the supernova and the neutron star; a checkpoint each
/// 500 steps. The contract names the supernova "from its iron core" (a dump saves the steps before it); M0-V14's dump no longer gives one
/// on today's tree (see the module doc), so the life starts from the cloud and the ledger covers the burning stages too.
fn supernova() -> &'static Life {
    SUPERNOVA.get_or_init(|| {
        let cal = calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration");
        let scene = Scene::preset_mass("massive", cal.preset_mass_sb.massive, Default::default()).unwrap();
        let l = run_life("supernova", scene, &["--edge", "leave"], 500, 200_000);
        report(&l);
        l
    })
}

/// The Sun-like life as a life: a white dwarf, the stages between, many windows.
fn sun_life() -> &'static Life {
    let l = sun();
    assert_life_is(l, "white_dwarf", &["main_sequence"], &[], 100);
    assert_eq!(l.summary["world"]["edge"], "bounce", "the Sun-like life runs in a bounce world");
    l
}

/// The supernova as a life: a `supernova` stage, a neutron star, leave edges, mass that left.
fn supernova_life() -> &'static Life {
    let l = supernova();
    assert_life_is(l, "neutron_star", &[], &["supernova"], 20);
    assert_eq!(l.summary["world"]["edge"], "leave", "the supernova runs in a world whose edges leave");
    l
}

fn mass_case(l: &Life) {
    let (d, at) = grade_mass(l);
    println!("{}: mass invariant drift {d:.3e} relative at worst (step {at}); bound {MASS_BAND:.0e}", l.id);
    assert!(d <= MASS_BAND, "{}: mass invariant drift {d:.3e} relative at step {at}, over {MASS_BAND:.0e}", l.id);
}

fn momentum_case(l: &Life) {
    let ((d, at), (instant, instant_at)) = grade_momentum(l);
    let last = l.points.last().expect("a checkpoint");
    println!(
        "{}: momentum invariant drift {d:.3e} of the running peak Σm|u| at worst (step {at}); over the instant Σm|u| {instant:.3e} (step {instant_at}, not graded); \
         at the end Δp ({:+.4e}, {:+.4e}) of Σm|u| {:.4e}; bound {MOMENTUM_BAND:.0e}",
        l.id,
        last.rec.momentum[0] - l.start.momentum[0],
        last.rec.momentum[1] - l.start.momentum[1],
        last.scale,
    );
    println!("{}: the whole life's momentum terms — {}", l.id, momentum_table(&l.start, &last.rec));
    let trail: Vec<String> = l.points.iter().step_by((l.points.len() / 12).max(1)).map(|p| format!("{}: {:.2e} (Σm|u| {:.2e})", p.rec.step, momentum_drift(&l.start, &p.rec, 1.0), p.scale)).collect();
    println!("{}: |Δp| along the life — {}", l.id, trail.join(" · "));
    assert!(d <= MOMENTUM_BAND, "{}: momentum invariant drift {d:.3e} of Σm|u| at step {at}, over {MOMENTUM_BAND:.0e}", l.id);
}

fn energy_case(l: &Life) {
    let (d, a, b) = grade_energy(l);
    let last = l.points.last().expect("a checkpoint");
    println!("{}: energy invariant drift {d:.3e} |W| per τ_dyn at worst, bound {CONS_PER_TAU:.0e}", l.id);
    let mut ws = windows(l);
    let over = ws.iter().filter(|w| w.0.is_nan() || w.0 > CONS_PER_TAU).count();
    ws.sort_by(|p, q| q.0.total_cmp(&p.0));
    let top: Vec<String> = ws.iter().take(5).map(|(v, x, y)| format!("{}..{}: {v:.2e}", x.step, y.step)).collect();
    println!("{}: {over} of {} windows over the bound; the five worst — {}", l.id, ws.len(), top.join(" · "));
    println!("{}: the worst window — {}", l.id, term_table(a, b));
    println!("{}: the whole life — {}", l.id, term_table(&l.start, &last.rec));
    assert!(d <= CONS_PER_TAU, "{}: energy invariant drift {d:.3e} |W| per τ_dyn, steps {}..{}, over {CONS_PER_TAU:.0e} (UNVERIFIED: a D for a conservative gravity, §0.4)", l.id, a.step, b.step);
}

#[test]
fn the_sun_like_life_in_a_bounce_world_keeps_the_mass_invariant() {
    grader_is_graded();
    mass_case(sun_life());
}

#[test]
fn the_sun_like_life_in_a_bounce_world_keeps_the_momentum_invariant() {
    grader_is_graded();
    momentum_case(sun_life());
}

#[test]
fn the_sun_like_life_in_a_bounce_world_keeps_the_energy_invariant_within_g_cons() {
    grader_is_graded();
    energy_case(sun_life());
}

#[test]
fn the_supernova_in_a_leave_world_keeps_the_mass_invariant() {
    grader_is_graded();
    mass_case(supernova_life());
}

#[test]
fn the_supernova_in_a_leave_world_keeps_the_momentum_invariant() {
    grader_is_graded();
    momentum_case(supernova_life());
}

#[test]
fn the_supernova_in_a_leave_world_keeps_the_energy_invariant_within_g_cons() {
    grader_is_graded();
    energy_case(supernova_life());
}

// ---- the grader, graded on hand-made records ----

#[test]
fn the_grader_is_graded_on_hand_made_records() {
    grader_is_graded();
}

fn rec(step: u64, t: f64, mass: f64, momentum: [f64; 2], energy: f64, w: f64) -> Rec {
    // The whole energy invariant in the first term, the rest 0; W is a record's own value.
    let mut terms: Vec<(&'static str, f64)> = ENERGY_TERMS.iter().map(|&(n, _)| (n, 0.0)).collect();
    terms[0].1 = energy;
    Rec { step, t, mass, energy: terms, w, momentum, mom_terms: [[0.0; 2]; 4] }
}

fn point(r: Rec, scale: f64, tau_dyn: f64) -> Point {
    Point { rec: r, scale, tau_dyn }
}

fn life_of(start: Rec, points: Vec<Point>) -> Life {
    Life { id: "hand-made", start, points, code: 0, summary: serde_json::json!({}), stages: BTreeSet::new(), steps: 0 }
}

/// A record in the snapshot's form: every f64 a string, the terms as the life's checkpoints write them.
fn snapshot_record(step: u64, t: f64) -> Value {
    let s = |x: f64| Value::String(format!("{x:?}"));
    let energy: serde_json::Map<String, Value> =
        ENERGY_TERMS.iter().enumerate().map(|(i, &(n, _))| (n.to_string(), s(1.0 + i as f64))).collect();
    serde_json::json!({
        "step": step, "sim_time": s(t),
        "mass": {"grid": s(10.0), "sinks": s(1.0), "escaped": s(2.0), "painted": s(0.5), "erased": s(0.25), "preset_dropped": s(3.0),
                 "vacuum_reset": s(0.125), "cleared": s(99.0)},
        "energy": energy,
        "momentum": {"grid": [s(1.0), s(2.0)], "sinks": [s(0.5), s(0.25)], "escaped": [s(0.125), s(0.0625)], "held": [s(4.0), s(8.0)]},
    })
}

/// The grader on hand-made values: a drift just under each bound passes and just over is refused, on each of the three; a window
/// shorter than a τ_dyn is held to its share; the terms and the signs of the records are §2.9's (every energy term in the table
/// and signed as the invariant signs it; `cleared` in no mass sum), and a life that ended otherwise or was read at too few
/// checkpoints is refused.
fn grader_is_graded() {
    // Mass: 1e-6 relative to the start's.
    let start = rec(1, 0.0, 1000.0, [0.0, 0.0], 0.0, -5.0);
    assert!(mass_drift(&start, &rec(2, 1.0, 1000.0 * (1.0 + 0.9e-6), [0.0; 2], 0.0, -5.0)) <= MASS_BAND, "a mass drift of 0.9e-6 refused");
    assert!(mass_drift(&start, &rec(2, 1.0, 1000.0 * (1.0 - 1.1e-6), [0.0; 2], 0.0, -5.0)) > MASS_BAND, "a mass drift of 1.1e-6 accepted");
    // Momentum: the vector's length over the scale, 1e-5 (a mass-sized 1.13e-6 is inside it, 5e-6 — the supernova's — too).
    assert!(momentum_drift(&start, &rec(2, 1.0, 1000.0, [0.6e-5, 0.6e-5], 0.0, -5.0), 1.0) <= MOMENTUM_BAND, "0.85e-5 of Σm|u| refused");
    assert!(momentum_drift(&start, &rec(2, 1.0, 1000.0, [0.8e-5, 0.8e-5], 0.0, -5.0), 1.0) > MOMENTUM_BAND, "1.13e-5 of Σm|u| accepted");
    assert!(momentum_drift(&start, &rec(2, 1.0, 1000.0, [0.9e-5, 0.0], 0.0, -5.0), 0.5) > MOMENTUM_BAND, "a halved scale not held to its share");
    assert!(momentum_drift(&start, &rec(2, 1.0, 1000.0, [0.8e-6, 0.8e-6], 0.0, -5.0), 1.0) <= MOMENTUM_BAND, "1.13e-6 of Σm|u| refused: the old bound still read");
    assert!(momentum_drift(&start, &rec(2, 1.0, 1000.0, [3.5e-6, 3.5e-6], 0.0, -5.0), 1.0) <= MOMENTUM_BAND, "4.95e-6 of Σm|u| refused");
    // Energy: |Δ| / |W| per τ_dyn. One τ_dyn long, 0.9e-3 |W| passes, 1.1e-3 does not; half a τ_dyn long, 0.9e-3 does not.
    let (a, tau) = (rec(1, 0.0, 1.0, [0.0; 2], 0.0, -100.0), 2.0);
    assert!(energy_per_tau(&a, &rec(2, tau, 1.0, [0.0; 2], 0.09, -100.0), tau) <= CONS_PER_TAU, "0.9e-3 |W| per τ_dyn refused");
    assert!(energy_per_tau(&a, &rec(2, tau, 1.0, [0.0; 2], -0.11, -100.0), tau) > CONS_PER_TAU, "1.1e-3 |W| per τ_dyn accepted");
    assert!(energy_per_tau(&a, &rec(2, tau / 2.0, 1.0, [0.0; 2], 0.09, -100.0), tau) > CONS_PER_TAU, "a half-τ_dyn window not held to its share");
    // The smaller |W| of the two records holds the window (the strictest): a collapse that deepens W ten-fold.
    let deep = rec(2, tau, 1.0, [0.0; 2], 0.09, -1000.0);
    assert!(energy_per_tau(&rec(1, 0.0, 1.0, [0.0; 2], 0.0, -100.0), &deep, tau) <= CONS_PER_TAU);
    assert!(energy_per_tau(&rec(1, 0.0, 1.0, [0.0; 2], 0.0, -90.0), &deep, tau) <= CONS_PER_TAU);
    assert!(energy_per_tau(&rec(1, 0.0, 1.0, [0.0; 2], 0.0, -80.0), &rec(2, tau, 1.0, [0.0; 2], 0.09, -1000.0), tau) > CONS_PER_TAU, "|W| not the smaller");
    // A life: a window over the bound anywhere in it is the worst, a clean life reads clean.
    let clean = life_of(start.clone(), vec![
        point(rec(2, 2.0, 1000.0, [0.0; 2], 0.0, -5.0), 1.0, 2.0),
        point(rec(3, 4.0, 1000.0, [0.0; 2], 0.004, -5.0), 1.0, 2.0),
    ]);
    assert!(grade_energy(&clean).0 <= CONS_PER_TAU && grade_mass(&clean).0 <= MASS_BAND && (grade_momentum(&clean).0).0 <= MOMENTUM_BAND);
    let leak = life_of(start.clone(), vec![
        point(rec(2, 2.0, 1000.0, [0.0; 2], 0.0, -5.0), 1.0, 2.0),
        point(rec(3, 4.0, 1000.0, [0.0; 2], 0.0, -5.0), 1.0, 2.0),
        point(rec(4, 6.0, 1000.0, [0.0; 2], 0.04, -5.0), 1.0, 2.0),
    ]);
    let (d, x, y) = grade_energy(&leak);
    assert!(d > CONS_PER_TAU && (x.step, y.step) == (3, 4), "a leak in the last window not found: {d}, {}..{}", x.step, y.step);
    // The running peak: a drift fixed from a collapse of scale 100 passes in the quiet life after it, where the instant scale (1e-3)
    // would have refused it; and the same drift refused when the scale never grew.
    let quiet = life_of(start.clone(), vec![
        point(rec(2, 1.0, 1000.0, [5e-5, 0.0], 0.0, -5.0), 100.0, 1.0),
        point(rec(3, 2.0, 1000.0, [5e-5, 0.0], 0.0, -5.0), 1e-3, 1.0),
    ]);
    let ((over_peak, _), (instant, _)) = grade_momentum(&quiet);
    assert!(over_peak <= MOMENTUM_BAND && instant > MOMENTUM_BAND, "the running peak not the scale: {over_peak}, instant {instant}");
    let never_grew = life_of(start.clone(), vec![point(rec(2, 1.0, 1000.0, [5e-5, 0.0], 0.0, -5.0), 1.0, 1.0)]);
    assert!(grade_momentum(&never_grew).0 .0 > MOMENTUM_BAND);
    // A NaN anywhere is the worst: a non-finite invariant is never a pass.
    let nan = life_of(start.clone(), vec![point(rec(2, 1.0, f64::NAN, [f64::NAN; 2], f64::NAN, -5.0), 1.0, 1.0)]);
    assert!(grade_mass(&nan).0.is_nan() && grade_momentum(&nan).0 .0.is_nan() && grade_energy(&nan).0.is_nan());
    // The records read as §2.9 writes them: every term of the invariant is in the table, signed, and `cleared` is in no mass sum.
    let r = rec_from(&snapshot_record(7, 1.5));
    assert_eq!(r.energy.len(), 13, "every energy term of §2.9 is read");
    let expect: f64 = ENERGY_TERMS.iter().enumerate().map(|(i, &(_, s))| s * (1.0 + i as f64)).sum();
    assert_eq!(r.energy_invariant(), expect, "the energy invariant's signs");
    assert_eq!(r.mass, 10.0 + 1.0 + 2.0 + 0.25 + 0.125 - 0.5 - 3.0, "grid + sinks + escaped + erased + vacuum_reset − painted − preset_dropped");
    assert_eq!(r.momentum, [1.0 + 0.5 + 0.125 + 4.0, 2.0 + 0.25 + 0.0625 + 8.0], "grid + sinks + escaped + held");
    assert_eq!((r.step, r.t, r.w), (7, 1.5, 4.0), "the record's own step, time and W");
    // The ending: unmet, unheld and the wrong one are each refused.
    let met = |ending: Value, met: bool| serde_json::json!({"until": {"met": met, "object": 3}, "objects": [{"id": 3, "ending": ending}]});
    assert_eq!(ending_of(&met("white_dwarf".into(), true), 0).as_deref(), Ok("white_dwarf"));
    assert!(ending_of(&met("white_dwarf".into(), false), 2).is_err() && ending_of(&met(Value::Null, true), 0).is_err());
    assert!(ending_of(&serde_json::json!({"until": {"met": true, "object": 9}, "objects": [{"id": 3, "ending": "white_dwarf"}]}), 0).is_err());
    let mut life = life_of(start, vec![point(rec(2, 1.0, 1000.0, [0.0; 2], 0.0, -5.0), 1.0, 1.0)]);
    life.summary = serde_json::json!({
        "until": {"met": true, "object": 3}, "objects": [{"id": 3, "ending": "neutron_star"}], "standins": {"s1": false, "s2": false},
        "events": [{"kind": "supernova", "object": 3}, {"kind": "core_collapse", "object": 4}],
    });
    assert!(check_life(&life, "neutron_star", &[], &["supernova"], 1).is_ok(), "the right life refused");
    assert!(check_life(&life, "white_dwarf", &[], &[], 1).is_err(), "another ending accepted");
    assert!(check_life(&life, "neutron_star", &["main_sequence"], &[], 1).is_err(), "a stage never seen accepted");
    assert!(check_life(&life, "neutron_star", &[], &["core_collapse"], 1).is_err(), "another object's event accepted");
    assert!(check_life(&life, "neutron_star", &[], &["supernova"], 2).is_err(), "too few checkpoints accepted");
    life.summary["standins"] = serde_json::json!({"s1": true, "s2": false});
    assert!(check_life(&life, "neutron_star", &[], &["supernova"], 1).is_err(), "a stand-in on accepted");
}
