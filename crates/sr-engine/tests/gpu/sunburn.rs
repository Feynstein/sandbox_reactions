//! M0-D50 — the Sun-like preset lives a main sequence on the shipped constants (contract §0.5 [M0-D50]; §2.5.1, §2.7, §2.10;
//! the lead's ruling at M0-V18, «Keep white dwarfs, D50 writes (Recommended)»). `headless --scene preset:sun` (the 600 × 400
//! leave world) on the physics adapter (R12): 3,000 steps straight — the collapse, the bounce near step 300 and its flash, and
//! the settling onto the burn (L_nuc/L 0.31 over steps 1,000–2,000 and 0.53 over 2,000–3,000 on the disk runs) — then 2,000
//! more resumed from that run's end dump (`--load-dump`, bit-identical to a straight run: §1.3.4, [M0-T26], [M0-D33]) with a
//! dump every 1,000, so the second run's ledger is the window's own. Graded in f64 from the window's three dumps (the EOS
//! twin, the twin's gravity) and its ledger — never from the stage label, whose L is M0-D53's:
//!
//! - **hydrogen burns steadily** — the centre's X_H (the densest cell's, ties to the lowest index) falls by at least 0.002
//!   over the window and by at least 0.001 in each 1,000 steps, and stays ≥ 0.5 (the disk runs: 0.7247 → 0.7214 → 0.7179);
//! - **the star stays bound** — the bound gas (Σ ≥ σ_vac and ½|u|² + ε_th + φ < 0, φ one solve of §1.4.2's kernel over every
//!   gas cell) holds at least 95 % of the preset's mass at each dump, and no sink forms (the disk runs: 99.4 %);
//! - **it shines what it burns, and only hydrogen burns** — over the window 0.5 L ≤ L_nuc ≤ 2 L (§0.5 [M0-D53]'s band, L and
//!   L_nuc the ledger's radiated and nuclear_released over the window's sim time; the disk runs: 0.70), and every gas cell
//!   below 0.5 T_He at each dump — He_burn's and C_alpha's threshold, the lowest after H_burn's — so L_nuc is L_H.
//!
//! The adversarial line: a flash that reads `main_sequence` for a few frames and runs away (M0-V8's P2) burns the centre's
//! hydrogen out and unbinds the star; a star held up without burning (M0-V8's stall) keeps X_H,c flat — the cases read one
//! run, so neither passes. The margins were stated before the case's first run, from §2.10's cloud as a disk scene on the same
//! constants (win-laptop, RTX 4080 Laptop, M0-D50; reports/ignition.md). Plants: `stall-constants` (physics.json's ladder, T_ν
//! and a_r back to M0-TC's) and `burn-runaway` (A_H 1 × 10⁻³).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;
use sr_engine::dump::Dump;
use sr_engine::headless::run_captured;
use sr_engine::state::SPECIES;
use sr_physics::eos::Eos;
use sr_physics::reference::grav::Gravity;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// The straight run, then the window resumed from its end dump, with a dump every `HALF` steps.
const SETTLE: u64 = 3_000;
const WINDOW: u64 = 2_000;
const HALF: u64 = 1_000;
/// The stated margins: X_H,c falls by at least `FALL` over the window and `HALF_FALL` in each half, and stays ≥ `X_MIN`.
const FALL: f64 = 0.002;
const HALF_FALL: f64 = 0.001;
const X_MIN: f64 = 0.5;
/// The bound gas's least share of the preset's mass (M_unb < 5 % of M).
const BOUND_SHARE: f64 = 0.95;
/// §0.5 [M0-D53]'s band on L_nuc / L.
const BAND: (f64, f64) = (0.5, 2.0);
/// Every record's T_thr is 0.5 T_k (reactions.json's `t_thr_factor`, §1.6.2); He_burn's and C_alpha's T_k is T_He.
const T_THR_FACTOR: f64 = 0.5;

/// What one state says, read in f64.
#[derive(Clone, Debug)]
struct Reading {
    /// The densest cell (ties to the lowest index) and its Σ, X_H and T.
    cell: (usize, usize),
    sigma_c: f64,
    x_h_c: f64,
    t_c: f64,
    /// The hottest gas cell's T.
    t_max: f64,
    /// The grid's mass and its bound part.
    mass: f64,
    bound: f64,
}

/// One state's reading: T by the EOS twin (ε_th floored as P8 does), φ by the twin's solve over the gas cells (Σ ≥ σ_vac).
fn reading(planes: &[f32], w: usize, h: usize, eos: &Eos, elements: &Elements, sigma_vac: f64) -> Reading {
    let cells = w * h;
    let p = |channel: usize, i: usize| f64::from(planes[channel * cells + i]);
    let sigma: Vec<f64> = (0..cells).map(|i| p(0, i)).collect();
    let densest = (0..cells).fold(0, |best, i| if sigma[i] > sigma[best] { i } else { best });
    let source: Vec<f64> = sigma.iter().map(|&s| if s >= sigma_vac { s } else { 0.0 }).collect();
    let phi = Gravity::new(1.0).potential(&source, w, h).expect("the twin's gravity");
    let (mut t_c, mut t_max, mut mass, mut bound) = (0.0, 0.0f64, 0.0, 0.0);
    for i in 0..cells {
        let s = sigma[i];
        mass += s;
        if s < sigma_vac {
            continue;
        }
        let x: [f64; N_SPECIES] = std::array::from_fn(|k| p(SPECIES.start + k, i));
        let comp = elements.composition(&x);
        let mu = 1.0 / comp.inv_mu;
        let mom = [p(1, i), p(2, i)];
        let th = eos.thermal(p(3, i), s, mom, comp.y_e, x[N_SPECIES - 1], mu);
        let t = Eos::temperature(th.eps_th, mu);
        let kinetic = 0.5 * (mom[0] * mom[0] + mom[1] * mom[1]) / (s * s);
        if kinetic + th.eps_th + phi[i] < 0.0 {
            bound += s;
        }
        t_max = t_max.max(t);
        if i == densest {
            t_c = t;
        }
    }
    Reading { cell: (densest % w, densest / w), sigma_c: sigma[densest], x_h_c: p(SPECIES.start, densest), t_c, t_max, mass, bound }
}

/// One dump of the window.
#[derive(Clone, Debug)]
struct Point {
    step: u64,
    sim_time: f64,
    sinks: usize,
    r: Reading,
}

struct Run {
    /// The preset's mass, the straight run's ledger at its start.
    start_mass: f64,
    /// The dumps at steps 3,000, 4,000 and 5,000.
    points: Vec<Point>,
    /// Over the window: L = Δradiated/Δt, L_nuc = Δnuclear_released/Δt, and Δt.
    l: f64,
    l_nuc: f64,
    window_dt: f64,
    adapter: String,
}

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-sunburn-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `headless --scene preset:sun --out <out> <extra…>` on the physics adapter: its stdout lines and summary.json.
fn headless(out: &Path, extra: &[String]) -> (Vec<String>, Value) {
    let mut args: Vec<String> = ["--scene", "preset:sun", "--out", &out.display().to_string()].map(String::from).to_vec();
    args.extend(extra.iter().cloned());
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let (code, lines) = run_captured(&args);
    assert_eq!(code, 0, "headless {args:?}: {:?}", lines.last());
    let summary = serde_json::from_slice(&std::fs::read(out.join("summary.json")).expect("summary.json")).expect("summary.json parses");
    (lines, summary)
}

fn num(v: &Value, path: &[&str]) -> f64 {
    path.iter().fold(v, |v, k| &v[*k]).as_f64().unwrap_or_else(|| panic!("summary.json has no number at {path:?}"))
}

/// The run, once for every case that reads it; its folders are removed once the dumps are read.
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
        let eos = Eos::from_physics(&physics).unwrap();
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let dir = temp("run");
        let (settle, window) = (dir.join("settle"), dir.join("window"));
        let (lines, first) = headless(&settle, &["--steps".to_string(), SETTLE.to_string()]);
        let start_mass = num(&first, &["ledger", "start", "mass", "grid"]);
        let resume = settle.join("dumps").join(format!("s_{SETTLE}.bin"));
        let args = ["--load-dump".to_string(), resume.display().to_string(), "--steps".to_string(), WINDOW.to_string(), "--dump-every".to_string(), HALF.to_string()];
        let (_, second) = headless(&window, &args);
        let mut dumps = vec![resume];
        dumps.extend((1..=WINDOW / HALF).map(|k| window.join("dumps").join(format!("s_{}.bin", SETTLE + k * HALF))));
        let points = dumps
            .iter()
            .map(|path| {
                let d = Dump::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let r = reading(&d.planes, d.world.width as usize, d.world.height as usize, &eos, &elements, sigma_vac);
                Point { step: d.step, sim_time: d.sim_time, sinks: d.sinks.len(), r }
            })
            .collect();
        let delta = |key: &str| num(&second, &["ledger", "end", "energy", key]) - num(&second, &["ledger", "start", "energy", key]);
        let window_dt = num(&second, &["ledger", "end", "sim_time"]) - num(&second, &["ledger", "start", "sim_time"]);
        let adapter = lines.iter().find(|l| l.starts_with("SR-ADAPTER")).cloned().unwrap_or_else(|| "SR-ADAPTER (not printed)".to_string());
        let _ = std::fs::remove_dir_all(&dir);
        Run { start_mass, points, l: delta("radiated") / window_dt, l_nuc: delta("nuclear_released") / window_dt, window_dt, adapter }
    })
}

fn print_points(run: &Run) {
    println!("{}", run.adapter);
    for p in &run.points {
        let r = &p.r;
        println!(
            "SR-SUNBURN step {} t {:.3}: centre {:?} Σ {:.3} T {:.3} X_H {:.5}; T_max {:.3}; bound {:.2} of {:.2} ({:.4} of the preset's {:.2}); sinks {}",
            p.step, p.sim_time, r.cell, r.sigma_c, r.t_c, r.x_h_c, r.t_max, r.bound, r.mass, r.bound / run.start_mass, run.start_mass, p.sinks
        );
    }
}

// ---- the readings' own arithmetic ----

/// On hand-made planes (64 × 64): a bound disk at rest, a hotter, denser cell inside it with its own mix, and one fast thin
/// cell far out — the densest cell, its X_H and T, the hottest T, the mass and the bound mass read what was put there.
#[test]
fn the_readings_see_what_the_planes_hold() {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let eos = Eos::from_physics(&physics).unwrap();
    let (n, vac) = (64usize, physics.get("sigma_vac").unwrap());
    let cells = n * n;
    let mut planes = vec![0.0f32; 14 * cells];
    let mut put = |x: usize, y: usize, sigma: f64, t: f64, x_h: f64, u: f64| {
        let i = y * n + x;
        let mut frac = [0.0; N_SPECIES];
        (frac[0], frac[1]) = (x_h, 1.0 - x_h);
        let comp = elements.composition(&frac);
        let energy = sigma * (Eos::eps_th_of_t(t, 1.0 / comp.inv_mu) + eos.eps_cold(sigma, comp.y_e, 0.0)) + 0.5 * sigma * u * u;
        for (channel, v) in [(0, sigma), (1, sigma * u), (2, 0.0), (3, energy)] {
            planes[channel * cells + i] = v as f32;
        }
        for (k, &f) in frac.iter().enumerate() {
            planes[(SPECIES.start + k) * cells + i] = f as f32;
        }
    };
    let mut disk = 0.0;
    for y in 0..n {
        for x in 0..n {
            if (x as f64 - 30.5).powi(2) + (y as f64 - 30.5).powi(2) <= 100.0 {
                put(x, y, 1.0, 2.0, 0.75, 0.0);
                disk += 1.0;
            }
        }
    }
    put(30, 31, 2.0, 7.0, 0.6, 0.0);
    put(60, 60, 0.01, 2.0, 0.75, 100.0);
    let r = reading(&planes, n, n, &eos, &elements, vac);
    println!("SR-SUNBURN reading on hand-made planes: {r:?} (NOT PROVEN on the star)");
    assert_eq!(r.cell, (30, 31), "the densest cell");
    assert!((r.sigma_c - 2.0).abs() < 1e-6 && (r.x_h_c - 0.6).abs() < 1e-6, "Σ {} X_H {}", r.sigma_c, r.x_h_c);
    assert!((r.t_c / 7.0 - 1.0).abs() < 1e-4 && (r.t_max / 7.0 - 1.0).abs() < 1e-4, "T_c {} T_max {}", r.t_c, r.t_max);
    let mass = disk + 1.0 + 0.01;
    assert!((r.mass - mass).abs() < 1e-4, "mass {} against {mass}", r.mass);
    assert!((r.bound - (mass - 0.01)).abs() < 1e-4, "bound {}: the disk and its core, not the fast cell", r.bound);
}

// ---- the preset's main sequence on the GPU ----

#[test]
fn hydrogen_burns_steadily_at_the_centre() {
    let run = run();
    print_points(run);
    let x: Vec<f64> = run.points.iter().map(|p| p.r.x_h_c).collect();
    let falls: Vec<f64> = x.windows(2).map(|w| w[0] - w[1]).collect();
    let fall = x[0] - x[x.len() - 1];
    println!("SR-SUNBURN X_H,c: {x:.5?}; falls {falls:.5?} (each ≥ {HALF_FALL}), {fall:.5} over the window (≥ {FALL}); least ≥ {X_MIN}");
    assert_eq!(run.points.len() as u64, 1 + WINDOW / HALF, "the window's dumps");
    for p in &run.points {
        assert!(p.r.x_h_c >= X_MIN, "X_H,c {:.5} < {X_MIN} at step {}: the centre's hydrogen burnt out", p.r.x_h_c, p.step);
    }
    for (k, f) in falls.iter().enumerate() {
        let (a, b) = (run.points[k].step, run.points[k + 1].step);
        assert!(*f >= HALF_FALL, "X_H,c fell {f:.5} from step {a} to {b}, under {HALF_FALL}: no steady burn");
    }
    assert!(fall >= FALL, "X_H,c fell {fall:.5} over the window, under {FALL}");
}

#[test]
fn the_star_stays_bound() {
    let run = run();
    for p in &run.points {
        let share = p.r.bound / run.start_mass;
        println!("SR-SUNBURN bound at step {}: {:.2} of the preset's {:.2} ({share:.4}), grid {:.2}, sinks {}", p.step, p.r.bound, run.start_mass, p.r.mass, p.sinks);
        assert_eq!(p.sinks, 0, "a sink formed by step {}", p.step);
        assert!(share >= BOUND_SHARE, "the bound gas holds {share:.4} of the preset's mass at step {} (< {BOUND_SHARE})", p.step);
    }
}

#[test]
fn it_shines_what_it_burns_and_only_hydrogen_burns() {
    let run = run();
    let physics = Physics::shipped().unwrap();
    let t_thr_he = T_THR_FACTOR * physics.get("t_he").unwrap();
    let ratio = run.l_nuc / run.l;
    println!(
        "SR-SUNBURN window: Δt {:.3}, L {:.1}, L_nuc {:.1}, L_nuc/L {ratio:.4} (band {:?}); T_max {:.3?} against 0.5 T_He = {t_thr_he}",
        run.window_dt,
        run.l,
        run.l_nuc,
        BAND,
        run.points.iter().map(|p| p.r.t_max).collect::<Vec<_>>()
    );
    for p in &run.points {
        assert!(p.r.t_max < t_thr_he, "T_max {:.3} ≥ 0.5 T_He = {t_thr_he} at step {}: helium can burn", p.r.t_max, p.step);
    }
    assert!(run.l > 0.0 && (BAND.0..=BAND.1).contains(&ratio), "L_nuc/L = {ratio:.4} over the window (L {:.1}, L_nuc {:.1}), outside {BAND:?}", run.l, run.l_nuc);
}
