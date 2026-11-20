//! M0-T99b — every life fits the world (contract §5.2's G-FIT, as §0.5 [M0-T99a] words it): through each preset's life, at the
//! calibration M0-V14's run wrote (`calibration::resolve`, refused when stale — G-CAL), **no bound gas denser than Σ = 10⁻² comes
//! within 16 cells of an edge of the default world**; a fainter bound halo may. Bound is the contract's own, ½|u|² + ε_th + φ < 0.
//!
//! The life is run once through `run_scene_captured` to its ending (`--until ending`), the preset built at calibration's mass
//! exactly as `squeeze.rs` and `preset:<name>` build it (never a disk: lives.md § The rerun, M0-D158 — the Giant's fit is chaotic,
//! so the plain preset is graded, on the adapter named), the world and the edge mode the scene's own (the default world, 600 × 400).
//! Each [`Progress::checkpoint_every`] steps the run saves a checkpoint — a dump, which moves nothing the life computes (M0-D77) —
//! and the dump is read here:
//! - **φ** is the §1.4.2 kernel's solve over the dump's Σ (vacuum, Σ < σ_vac, no source), G = 1, by the CPU reference
//!   (`reference::grav::Gravity`), the sinks' softened pull added (r_acc = max(2, 2GM_s/c_sb²), §2.6) — lives.md's reading, which
//!   `probe/dumpread.py` checked against the ledger's Σ½Σφ to 10⁻⁵;
//! - **ε_th** is the equation of state's own (`Eos::thermal`: E/Σ − ½|u|² − ε_cold, floored at T_floor/μ) from the dump's energy,
//!   momentum and mass fractions — exact, where lives.md bracketed it (a dump holds no ε_th, the old probe's `nec`/`suf`);
//! - a cell is a candidate when Σ ≥ the floor and it is bound; its distance is from its centre to the nearest world edge, in cells.
//!
//! Readings declared (M0-T99b; cheap to reverse):
//! 1. **Within 16 cells** is a distance of at most 16 from the cell's centre to the edge (centres sit at half-integers: 15.5 is in,
//!    16.5 is out, as `probe/edge_halo.py` reads it).
//! 2. **The first violation cuts the life** (the unwind is caught, the readings so far kept; `SR_FIT_NO_CUT=1` reads on, every reading still graded): a G-FIT red is a D or a question, never a
//!    looser number, and a plant that crowds the edge at once is red in seconds. The case prints each life's closest approach at
//!    Σ ≥ 10⁻⁴, 10⁻³, 10⁻², 10⁻¹ — beside M0-T99a's table (lives.md § The rerun) — and the densest bound cell inside the band.
//! 3. **Checkpoint cadence**: every [`EVERY_DENSE`] steps for the Massive and the Giant (M0-D163's 100), every [`EVERY_SUN`] for the
//!    Sun-like (M0-V14's 1,000) — the lives are the same at any cadence (a checkpoint cuts no chunk, M0-D77).
//! 4. **A life that ends early or never** — a run cut by `--max-steps` (exit 2) or one that read no checkpoint is red: no fit was
//!    shown. The Sun-like life is about 180,000 steps (over 10 minutes: R12's ⏱ scope, the lead's run).
//!
//! Each of the three cases (one life each) grades the graders on hand-made values first (a bound dense clump 5 cells from an edge refused, the same clump unbound,
//! under the floor, or 20 cells out accepted, a sink's pull counted, the distance measured to every edge).

use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use sr_engine::dump::{Dump, Sink};
use sr_engine::headless::{parse_args, run_scene_captured, Progress, Tick, CHECKPOINT_DIR};
use sr_engine::scene::Scene;
use sr_physics::calibration::{self, Calibration};
use sr_physics::eos::Eos;
use sr_physics::reference::grav::{Gravity, SinkSource};
use sr_physics::reference::sinks::r_acc;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// §0.5 [M0-T99a]: the density floor of the graded bound gas, Σ (sandbox units).
const FLOOR: f64 = 1e-2;
/// §5.2: "within 16 cells of an edge".
const BAND: f64 = 16.0;
/// The floors printed beside the graded one (M0-T99a's table: lives.md § The rerun), the graded one's index.
const FLOORS: [f64; 4] = [1e-4, 1e-3, FLOOR, 1e-1];
const GRADED: usize = 2;
/// G in the sandbox's units (§2.1).
const G: f64 = 1.0;
/// The checkpoint cadences (declared reading 3).
const EVERY_DENSE: u64 = 100;
const EVERY_SUN: u64 = 1_000;
/// The species channel of the neutron fraction X_n (§1.5's order, registry's `ID_N`).
const ID_N: usize = 9;
/// A life graded over fewer checkpoints than this is no reading of a life.
const MIN_READINGS: usize = 10;

// ---- the constants ----

/// What a reading needs of physics.json and elements.json.
struct Constants {
    eos: Eos,
    elements: Elements,
    sigma_vac: f64,
    c_sb: f64,
}

fn constants() -> Constants {
    let physics = Physics::shipped().expect("the shipped constants");
    let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("{k} is a physics.json key"));
    Constants {
        eos: Eos::from_physics(&physics).expect("the shipped equation of state"),
        elements: Elements::shipped().expect("the shipped elements"),
        sigma_vac: get("sigma_vac"),
        c_sb: get("c_sb"),
    }
}

fn calibration() -> Calibration {
    calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("a calibration")
}

thread_local! {
    /// One gravity pass per thread: its kernel's transform is cached per padded size.
    static GRAVITY: RefCell<Gravity> = RefCell::new(Gravity::new(G));
}

// ---- the reading of one dump ----

/// One checkpoint's reading: for each of [`FLOORS`], the distance of the nearest bound cell at or over it (`None`: none is bound),
/// and the densest bound cell inside the band.
#[derive(Clone, Debug)]
struct Fit {
    step: u64,
    sim_time: f64,
    mass: f64,
    sinks: usize,
    nearest: [Option<f64>; FLOORS.len()],
    /// The densest bound Σ within [`BAND`] cells of an edge (0 when none), and its cell.
    band_sigma: f64,
    band_cell: (usize, usize),
}

impl Fit {
    /// The graded distance: the nearest bound gas at Σ ≥ [`FLOOR`], `None` when none is bound at all.
    fn graded(&self) -> Option<f64> {
        self.nearest[GRADED]
    }
    fn line(&self) -> String {
        let d = |v: Option<f64>| v.map_or("none".to_string(), |d| format!("{d:.1}"));
        format!(
            "step={} t={:.2} mass={:.1} sinks={} nearest bound gas (cells) at Σ≥1e-4/1e-3/1e-2/1e-1: {}/{}/{}/{}; densest in the band {:.3e} at cell {:?}",
            self.step,
            self.sim_time,
            self.mass,
            self.sinks,
            d(self.nearest[0]),
            d(self.nearest[1]),
            d(self.nearest[2]),
            d(self.nearest[3]),
            self.band_sigma,
            self.band_cell
        )
    }
}

/// The distance in cells from the centre of cell (x, y) to the nearest edge of a `w` × `h` world.
fn edge_distance(x: usize, y: usize, w: usize, h: usize) -> f64 {
    let (x, y) = (x as f64 + 0.5, y as f64 + 0.5);
    x.min(w as f64 - x).min(y).min(h as f64 - y)
}

/// G-FIT's reading of a state: `planes` the 14 canonical planes of a `w` × `h` world (σ, mom_x, mom_y, energy, the ten mass
/// fractions), `sinks` the dump's.
fn read_fit(c: &Constants, planes: &[f32], w: usize, h: usize, sinks: &[Sink], step: u64, sim_time: f64) -> Result<Fit, String> {
    let cells = w * h;
    if planes.len() != 14 * cells {
        return Err(format!("{} values for the {w} × {h} world's 14 planes", planes.len()));
    }
    let plane = |k: usize| &planes[k * cells..(k + 1) * cells];
    let sigma: Vec<f64> = plane(0).iter().map(|&v| f64::from(v)).collect();
    if sigma.iter().any(|s| !s.is_finite()) {
        return Err("a non-finite Σ".into());
    }
    let sources: Vec<SinkSource> = sinks
        .iter()
        .map(|s| SinkSource { x: s.x, y: s.y, m: s.m, r_acc: r_acc(G, s.m, c.c_sb) })
        .collect();
    let field = GRAVITY
        .with(|g| g.borrow_mut().field_with_sinks(&sigma, c.sigma_vac, w, h, G, &sources))
        .map_err(|e| format!("the potential: {e:?}"))?;
    let mut fit = Fit {
        step,
        sim_time,
        mass: sigma.iter().filter(|&&s| s >= c.sigma_vac).sum::<f64>() + sinks.iter().map(|s| s.m).sum::<f64>(),
        sinks: sinks.len(),
        nearest: [None; FLOORS.len()],
        band_sigma: 0.0,
        band_cell: (0, 0),
    };
    for i in 0..cells {
        let s = sigma[i];
        // Under the lowest floor nothing is read: the gas the contract's floor leaves out, and the vacuum, are no candidates.
        if !(s >= FLOORS[0]) {
            continue;
        }
        let (mx, my) = (f64::from(plane(1)[i]), f64::from(plane(2)[i]));
        let mut x = [0.0f64; N_SPECIES];
        for (k, xk) in x.iter_mut().enumerate() {
            *xk = f64::from(plane(4 + k)[i]);
        }
        let comp = c.elements.composition(&x);
        let thermal = c.eos.thermal(f64::from(plane(3)[i]), s, [mx, my], comp.y_e, x[ID_N], 1.0 / comp.inv_mu);
        let bound = 0.5 * (mx * mx + my * my) / (s * s) + thermal.eps_th + field.phi[i] < 0.0;
        if !bound {
            continue;
        }
        let (cx, cy) = (i % w, i / w);
        let d = edge_distance(cx, cy, w, h);
        for (k, floor) in FLOORS.iter().enumerate() {
            if s >= *floor && fit.nearest[k].is_none_or(|n| d < n) {
                fit.nearest[k] = Some(d);
            }
        }
        if d <= BAND && s > fit.band_sigma {
            fit.band_sigma = s;
            fit.band_cell = (cx, cy);
        }
    }
    Ok(fit)
}

/// §5.2 / §0.5 [M0-T99a]: no bound gas at Σ ≥ [`FLOOR`] within [`BAND`] cells of an edge. A NaN never reaches here (an unreadable dump is `Err`).
fn fits(f: &Fit) -> Result<(), String> {
    match f.graded() {
        Some(d) if d <= BAND => Err(format!("bound gas at Σ ≥ {FLOOR:e} {d:.1} cells from an edge (≤ {BAND}): {}", f.line())),
        _ => Ok(()),
    }
}

// ---- the lives ----

/// What a life gave: the exit code, the checkpoints' readings and, when the fit broke, the unwind that cut the run.
struct Life {
    id: &'static str,
    code: i32,
    steps: u64,
    readings: Vec<Fit>,
    aborted: Option<String>,
}

/// `preset` at calibration's mass to its ending (or `--max-steps`, exit 2) on the physics adapter, a checkpoint read each `every`
/// steps; the first reading that does not fit cuts the run.
fn run_life(id: &'static str, preset: &str, every: u64, max_steps: u64) -> Life {
    let cal = calibration();
    let mass = match preset {
        "sun" => cal.preset_mass_sb.sun,
        "massive" => cal.preset_mass_sb.massive,
        _ => cal.preset_mass_sb.giant,
    };
    println!("{id}: preset {preset} at {mass:.1}; calibration physics_hash {}; checkpoint every {every} steps", cal.physics_hash);
    let scene = Scene::preset_mass(preset, mass, Default::default()).unwrap();
    let out = std::env::temp_dir().join(format!("sr-fit-{}-{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let (max, out_dir, label) = (max_steps.to_string(), out.display().to_string(), format!("fit:{id}"));
    let mut args: Vec<String> =
        ["--scene", &label, "--until", "ending", "--max-steps", &max, "--out", &out_dir].map(String::from).to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let consts = constants();
    // `SR_FIT_NO_CUT=1` reads a red life to its end (the whole extent of a violation, for a D): every reading is still graded at the end.
    let no_cut = std::env::var("SR_FIT_NO_CUT").is_ok_and(|v| !v.is_empty() && v != "0");
    let readings = RefCell::new(Vec::<Fit>::new());
    let last_step = RefCell::new(0u64);
    let mut tick = |t: &Tick| {
        *last_step.borrow_mut() = t.step;
    };
    let mut saved = |step: u64, sim_time: f64, _stage: &str| {
        let bin = out.join(CHECKPOINT_DIR).join(format!("s_{step}.bin"));
        let dump = Dump::read(&bin).unwrap_or_else(|e| panic!("{id}: the checkpoint {}: {e}", bin.display()));
        let (w, h) = (dump.world.width as usize, dump.world.height as usize);
        let fit = read_fit(&consts, &dump.planes, w, h, &dump.sinks, step, sim_time).unwrap_or_else(|e| panic!("{id}: step {step}: {e}"));
        println!("{id} {}", fit.line());
        let verdict = fits(&fit);
        readings.borrow_mut().push(fit);
        if let Err(e) = verdict {
            if !no_cut {
                panic!("cut: {e}");
            }
            println!("{id}: NOT FITTING (SR_FIT_NO_CUT set, the life read on): {e}");
        }
    };
    let progress = Progress { every: 16, tick: &mut tick, checkpoint_every: Some(every), saved: &mut saved };
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
    let (readings, steps) = (readings.into_inner(), last_step.into_inner());
    println!("{id}: {steps} steps, exit {code}, {} checkpoints", readings.len());
    Life { id, code, steps, readings, aborted }
}

fn sun() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| run_life("sun", "sun", EVERY_SUN, 400_000))
}

fn massive() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| run_life("massive", "massive", EVERY_DENSE, 120_000))
}

fn giant() -> &'static Life {
    static LIFE: OnceLock<Life> = OnceLock::new();
    LIFE.get_or_init(|| run_life("giant", "giant", EVERY_DENSE, 120_000))
}

/// A life's verdict: it ended (exit 0), read enough checkpoints, and every one fits; its closest approach is printed beside M0-T99a's.
fn grade(l: &Life) {
    let closest = |k: usize| {
        l.readings
            .iter()
            .filter_map(|f| f.nearest[k].map(|d| (d, f.step, f.sim_time)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map_or("none".to_string(), |(d, s, t)| format!("{d:.1} cells (step {s}, t {t:.1})"))
    };
    println!(
        "{}: closest bound gas over {} checkpoints, steps {} → {}: Σ≥1e-4 {}; Σ≥1e-3 {}; Σ≥1e-2 {}; Σ≥1e-1 {} (calibration physics_hash {})",
        l.id,
        l.readings.len(),
        l.readings.first().map_or(0, |f| f.step),
        l.readings.last().map_or(0, |f| f.step),
        closest(0),
        closest(1),
        closest(2),
        closest(3),
        calibration().physics_hash
    );
    assert!(l.aborted.is_none(), "{}: G-FIT broke: {:?}", l.id, l.aborted);
    let broken: Vec<&Fit> = l.readings.iter().filter(|f| fits(f).is_err()).collect();
    if let (Some(a), Some(b)) = (broken.first(), broken.last()) {
        let worst = broken.iter().map(|f| f.graded().unwrap_or(f64::MAX)).fold(f64::MAX, f64::min);
        println!("{}: {} of {} checkpoints do not fit, steps {} → {} (t {:.1} → {:.1}), nearest {worst:.1} cells", l.id, broken.len(), l.readings.len(), a.step, b.step, a.sim_time, b.sim_time);
    }
    assert_eq!(l.code, 0, "{}: the life did not reach its ending within the step cap ({} steps, {} checkpoints)", l.id, l.steps, l.readings.len());
    assert!(l.readings.len() >= MIN_READINGS, "{}: read at {} checkpoints, fewer than the {MIN_READINGS} a life gives", l.id, l.readings.len());
    for f in &l.readings {
        if let Err(e) = fits(f) {
            panic!("{}: {e}", l.id);
        }
    }
    println!("{}: every one of {} checkpoints fits (no bound gas at Σ ≥ {FLOOR:e} within {BAND} cells of an edge)", l.id, l.readings.len());
}

// ---- the graders on hand-made values ----

/// A 64 × 64 world: a heavy lump at its centre (Σ 100 over 5 × 5 cells, 2,500 sb: φ ≈ −G·M/r), and one `probe` cell at (px, py) of Σ
/// `s` at rest whose thermal energy per unit mass is `eps` (pure hydrogen, so μ = 1/2: ε_th = E/Σ, T_floor/μ = 0.1 under it).
fn hand_made(probe: (usize, usize), s: f32, eps: f32, velocity: f32) -> Vec<f32> {
    let (w, h) = (64usize, 64usize);
    let cells = w * h;
    let mut planes = vec![0.0f32; 14 * cells];
    let mut put = |x: usize, y: usize, s: f32, eps: f32, vx: f32| {
        let i = y * w + x;
        planes[i] = s;
        planes[cells + i] = s * vx;
        planes[3 * cells + i] = s * eps + 0.5 * s * vx * vx;
        planes[4 * cells + i] = 1.0; // x_H
    };
    for y in 30..35 {
        for x in 30..35 {
            put(x, y, 100.0, 0.0, 0.0);
        }
    }
    put(probe.0, probe.1, s, eps, velocity);
    planes
}

fn graded_distance(c: &Constants, planes: &[f32], sinks: &[Sink]) -> Option<f64> {
    read_fit(c, planes, 64, 64, sinks, 0, 0.0).expect("a hand-made world reads").graded()
}

fn graders_are_graded() {
    let c = constants();
    // Bound: at rest, cold (ε_th at its floor), 25 cells from a 2,500 sb lump: φ ≈ −100 against ε_th ≈ 0.1.
    let bound = hand_made((5, 32), 0.05, 0.0, 0.0);
    let d = graded_distance(&c, &bound, &[]).expect("a bound dense clump read as none");
    assert!((d - 5.5).abs() < 1e-12, "the clump at x = 5 is 5.5 cells from the left edge, read {d}");
    // The edges: each of the four, by cell centre (the lump's own cells are bound to themselves at 30–34: 29.5 or more).
    for (probe, want) in [((32, 3), 3.5), ((58, 32), 5.5), ((32, 60), 3.5), ((0, 32), 0.5)] {
        let got = graded_distance(&c, &hand_made(probe, 0.05, 0.0, 0.0), &[]).unwrap();
        assert!((got - want).abs() < 1e-12, "cell {probe:?}: {got} cells from an edge, wanted {want}");
    }
    assert!(fits(&read_fit(&c, &bound, 64, 64, &[], 0, 0.0).unwrap()).is_err(), "bound dense gas 5.5 cells from an edge accepted");
    // The same clump unbound: hot (ε_th 500 ≫ |φ|) or fast (½u² = 200).
    let hot = hand_made((5, 32), 0.05, 500.0, 0.0);
    assert!(graded_distance(&c, &hot, &[]).is_none_or(|d| d > 25.0), "a hot clump read as bound");
    let fast = hand_made((5, 32), 0.05, 0.0, 20.0);
    assert!(graded_distance(&c, &fast, &[]).is_none_or(|d| d > 25.0), "a fast clump read as bound");
    // Under the floor: a bound halo of Σ 5e-3 is allowed (§0.5 [M0-T99a]); at 1e-2 it is not.
    let faint = hand_made((5, 32), 0.005, 0.0, 0.0);
    assert!(fits(&read_fit(&c, &faint, 64, 64, &[], 0, 0.0).unwrap()).is_ok(), "a bound halo under the floor refused");
    assert!(read_fit(&c, &faint, 64, 64, &[], 0, 0.0).unwrap().nearest[1].is_some(), "the 1e-3 floor lost a Σ 5e-3 cell");
    // (f32 0.01 reads 0.0099999998 in f64, under the floor by rounding: a hair over it is the floor's side.)
    let at_floor = hand_made((5, 32), 0.010_001, 0.0, 0.0);
    assert!(fits(&read_fit(&c, &at_floor, 64, 64, &[], 0, 0.0).unwrap()).is_err(), "bound gas at the floor accepted");
    // The band: 16.5 cells from the edge is out, 15.5 in.
    let out = hand_made((16, 32), 0.05, 0.0, 0.0);
    assert!(fits(&read_fit(&c, &out, 64, 64, &[], 0, 0.0).unwrap()).is_ok(), "bound gas 16.5 cells out refused");
    let inside = hand_made((15, 32), 0.05, 0.0, 0.0);
    assert!(fits(&read_fit(&c, &inside, 64, 64, &[], 0, 0.0).unwrap()).is_err(), "bound gas 15.5 cells out accepted");
    // The lump alone, no probe: its own cells are 29.5 from the edge, bound, and fit.
    let alone = hand_made((32, 32), 100.0, 0.0, 0.0);
    let f = read_fit(&c, &alone, 64, 64, &[], 0, 0.0).unwrap();
    assert!(fits(&f).is_ok() && f.graded().is_some_and(|d| d > 25.0), "the lump alone: {f:?}");
    // A sink's pull is counted: a gas cell hot enough to escape the lump (ε_th 150 against φ ≈ −100) is bound once a sink
    // of 2,500 sb sits beside it.
    let warm = hand_made((5, 32), 0.05, 150.0, 0.0);
    assert!(graded_distance(&c, &warm, &[]).is_none_or(|d| d > 25.0), "a warm clump bound with no sink");
    let sink = Sink { x: 8.0, y: 32.5, vx: 0.0, vy: 0.0, m: 2_500.0 };
    assert_eq!(graded_distance(&c, &warm, &[sink]), Some(5.5), "the sink's pull left out of φ");
    // A non-finite Σ and a short plane are refused, not graded.
    let mut bad = hand_made((5, 32), 0.05, 0.0, 0.0);
    bad[0] = f32::NAN;
    assert!(read_fit(&c, &bad, 64, 64, &[], 0, 0.0).is_err(), "a NaN Σ read");
    assert!(read_fit(&c, &bad[..100], 64, 64, &[], 0, 0.0).is_err(), "a short plane read");
}

#[test]
fn the_sun_like_life_keeps_its_bound_gas_16_cells_from_the_edge() {
    graders_are_graded();
    grade(sun());
}

#[test]
fn the_massive_life_keeps_its_bound_gas_16_cells_from_the_edge() {
    graders_are_graded();
    grade(massive());
}

#[test]
fn the_giant_life_keeps_its_bound_gas_16_cells_from_the_edge() {
    graders_are_graded();
    grade(giant());
}
