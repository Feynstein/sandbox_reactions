//! `sandbox-reactions calibrate --out <file> [--adapter S] [--only <item>[,<item>…]] [--max-steps N] [--bracket LO,HI] [--resume DIR] [--progress-every N] [--checkpoint-every M]`
//! (m0_contrat.md §2.11 items 1–6, §3.3, §6.4; M0-T66b, M0-T68, M0-T70): the cold ceilings, the ignition and ending
//! thresholds, the order they must keep, and what the three presets' lives say.
//!
//! **Items 1–2, m_ch_sb, m_tov_sb and ns_sigma_max_sb [M0-R4].** The sheet's cold ceiling for this equation of state,
//! computed on the CPU in f64 by `sr_physics::reference::sheet` — a cold axisymmetric disk at T_floor, C 0.5 + O 0.5
//! (Y_e = 0.5) for m_ch_sb and X_n = 1 for m_tov_sb, no grid and no adapter. ns_sigma_max_sb is m_tov_sb per cell area (one
//! cell is one unit of area): the densest a holding neutron star can make one cell. `--adapter` is accepted and needed only by
//! a later item that runs the GPU; `measured.adapter` reads `cpu` until one does.
//!
//! **Items 3–4 [M0-T68]: m_ign_sb, m_up_sb, m_bh_sb.** Each is a bisection on the sandbox mass of the preset cloud (§2.10, the
//! Sun's mix) over whole headless lives on the GPU — [`bisect`]: both ends of the bracket first (a bracket whose ends fall on
//! the same side is refused, two runs spent, not hours), then the geometric midpoint until the bracket is within
//! [`BISECT_TOLERANCE`] (2 %), the value its midpoint. A run is one [`Life`], read from the summary.json `headless` writes; the
//! mapping of a life to a side is the item's own ([`side_ign`], [`side_up`], [`side_bh`]) and a life that is on neither side
//! (no ending within `--max-steps`, an ending the item cannot place) is a failure, reported and never guessed (exit 6).
//! m_ign_sb stops each run at the first of the ignition and the ending (`Until::IgnitionOrEnding`): the failed star's ending is
//! the stage held for [`ending_hold`](sr_physics::observe::stages::ending_hold), max(10 t.u., 20 τ_dyn) — §2.11's 20 τ_dyn
//! (§0.5 [M0-D78]); ignition is the light window or a centre that burned a tenth of its hydrogen, so a flash's burnt core is
//! no failed star.
//! m_up_sb stops each run once its side is known (`Until::CollapseOrEnding`, M0-D75): the first frame where the `core_collapse`
//! event fires or an object shows a neutron star or a black hole, else the white dwarf's ending; m_bh_sb's runs and the presets'
//! go to their ending. A life's ending is its heaviest object's (§0.5 [M0-D95]; the lead at M0-D95, «Heaviest object's
//! (Recommended)»): a fragment's remnant is logged, never stops the run and never places it.
//! The default bracket of each item is in units of the nominal M₀ ([`UNIT_MASS`]); `--bracket LO,HI` (sandbox mass, with one
//! `--only` item) replaces it. Every run's mass, side, ending, steps and wall time is a `SR-CALIBRATE run` line, closing on
//! `record=<dir>`: the life's summary.json and events.jsonl are kept there, under `build/calibrate_runs/<stamp>/<item>-<NN>/`
//! (git-ignored; the final dump goes, a failed life's record stays and its error names it), and while a life runs a
//! `SR-CALIBRATE progress item=… mass=… step=… sim_time=… heat_unconverged=… stage=…` line comes every `--progress-every` steps (default [`PROGRESS_EVERY`], M0-D60, M0-D75)
//! and the same fields land in the life's `progress.json`, atomically (M0-D74); a life's `SR-WARN heat unconverged` lines are
//! counted into `heat_unconverged`, not printed. While a step reaches P5's ceiling or the heaviest object collapses, a life runs
//! one step a frame and each is a progress line (M0-D80, `headless::one_step_next`); every line names the heaviest object's raw
//! stage beside the shown one and is followed by six rows of the frame's, the star's and the world's numbers (`tick_rows`, coloured
//! by [`paint`] on a terminal), and progress.json carries the same keys.
//! A running life saves its checkpoint (the dump and the observers' snapshot, M0-D76) into `<life>/checkpoint/` every
//! `--checkpoint-every` steps (default [`CHECKPOINT_EVERY`], M0-D77), apart from the progress tick, and `--resume` continues a
//! cut life from it — `continued from step <k>` on its run line — instead of from step 0.
//! `measured.adapter` names the adapter the first of them opened. Runs are whole lives: the lead's, in M0-V9's window (§6.4).
//!
//! **Item 5 [M0-T70]: the order (K9)** — [`check_order`] over the merged file: m_ign < m_ch < m_up < m_bh, m_ch < m_tov,
//! m_tov/m_ch in [1.4, 2.0] and sigma_bh at or over ns_sigma_max_sb; each inequality is checked when both its sides are in the
//! file. Exit 6 with the numbers, nothing written, and never a retry with other constants — and it runs *before* the presets'
//! lives, which would be hours spent on a broken order.
//!
//! **Item 6 [M0-T70]: the presets' lives** — [`measure_presets`]. `preset_mass_sb` is the inverse of the mass translation
//! (§1.10.1, M0-T69's [`MassScale`]) at 1, 15 and 40 Suns; each of the three is run to its ending (`Until::Ending`) and must
//! end as the preset promises (white dwarf, neutron star, black hole) — else exit 6. From the Sun-like life: `life_sun_sb`
//! (drop → the first frame of the white-dwarf stage the `ending` closes, not a transient one before the main sequence), `top_rung` (`sr_physics::time::top_rung_from_life`, ÷ 10 s to two significant
//! digits, §1.8.2) and `t_eff_sun_sb` (§1.10.3's T_eff = (L / 2πRσ)^¼ from the L and R of the first frame whose central
//! hydrogen is at or under [`T_EFF_X_H`](crate::watch::T_EFF_X_H) = 0.35, summary.json's `at_x_h`; refused unless it sits
//! between T_floor and T_H, the temperature scale's own condition). `stage_durations_sb` is, per stage, the sim time the three
//! lives spent showing it (remnant stages excepted: their clock is §1.10.4's 10⁸ years per t.u.), one `[mass_sb, t.u.]` pair a
//! life. `max_gas_speed` is the largest block rms gas speed (√(2K/m), summary.json's `max_gas_speed`) over every run this
//! process made, kept against the file's — exit 6 unless c_sb exceeds twice it ([`check_speed`]). `render_reserve_ms` is
//! written by the drawing lot's `--measure-ui` run (§3.3, item 7): `calibrate` keeps the file's and refuses `--only` for it.
//!
//! **The file.** `--out` is plain JSON with §2.11's keys — `version`, `physics_hash` (the assets' now), `measured` {date, git,
//! adapter, box} and the items measured so far. A file already there under this physics hash is the base: `--only`
//! re-measures one item and keeps the rest; a file under another hash is dropped whole (`SR-WARN`), its numbers belong to
//! other constants. It is a complete [`Calibration`](sr_physics::calibration::Calibration) once every item has run; until the
//! drawing lot writes `render_reserve_ms` it has none, and R_reserve is §1.8.3's 3.0 (M0-D62). The next build embeds it
//! (`sr-physics`'s build.rs). `calibrate` needs no calibration itself and runs on a stale one.
//!
//! **What exits 6** (reported, never re-tuned inside the run, nothing written): a sheet that has no ceiling
//! ([`SheetError`](sr_physics::reference::sheet::SheetError)), the order of item 5, a preset that does not end as promised, a
//! T_eff off the temperature scale and a gas speed over c_sb / 2. A run with overrides ([`run_with`]) measures and checks the same
//! and never writes (§2.12.1).
//!
//! **The grid's own disk.** [`cold_disk_scene`] solves a cold disk's hydrostatic equilibrium on the 64 × 64 grid by a
//! self-consistent field over the twin's gravity kernel (§1.4.2) — the start of G-CORE (d)'s resolved hold (M0-T67). It is
//! not the ceiling: that is a continuum limit, and no grid disk holds near it (reports/cold_ceiling.md).
//!
//! stdout: one `SR-CALIBRATE item=… value=… wall_s=…` per item, last `SR-CALIBRATE DONE …`. Exit codes as `headless`'s: 0 done
//! · 4 bad arguments · 6 the calibration failed.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};
use sr_physics::calibration;
use sr_physics::eos::Eos;
use sr_physics::observe::translate::{surface_temperature_sb, MassScale, TempScale, PRESET_SUNS};
use sr_physics::time::top_rung_from_life;
use sr_physics::reference::grav::Gravity;
use sr_physics::reference::sheet::{self, ColdMatter, Fluid};
use sr_physics::registry::{Elements, Physics, N_SPECIES};
use sr_physics::units::G;

use crate::dump::Dump;
use crate::headless::{
    git_rev, run_scene_captured, unconverged_steps, Args, Progress, Tick, CHECKPOINT_DIR, DEFAULT_MAX_STEPS, EXIT_ARGS, EXIT_CALIBRATION, EXIT_DONE, EXIT_GPU, EXIT_NON_FINITE,
    EXIT_UNMET, OBSERVERS_VERSION,
};
use crate::scene::{EdgeMode, Gas, Heat, Object, Scene, Switches, TestSettings, UNIT_MASS};
use crate::state::WorldConfig;
use crate::watch::{Until, T_EFF_X_H};

/// The world a grid disk runs in: the smallest §2.8 allows, the disk about its centre.
pub const WORLD: u32 = 64;
/// §2.11 item 5's K9 bounds on m_tov/m_ch.
pub const K9_RATIO: (f64, f64) = (1.4, 2.0);

/// The items `--only` takes, by the keys of §2.11's schema; `m_tov_sb` and `ns_sigma_max_sb` come from one sheet, and the last
/// six from the three presets' lives. `render_reserve_ms` is the drawing lot's (item 7).
pub const ITEMS: [&str; 12] = [
    "m_ch_sb",
    "m_tov_sb",
    "ns_sigma_max_sb",
    "m_ign_sb",
    "m_up_sb",
    "m_bh_sb",
    "preset_mass_sb",
    "t_eff_sun_sb",
    "life_sun_sb",
    "top_rung",
    "stage_durations_sb",
    "max_gas_speed",
];

/// What a cold disk is made of: electrons' matter (C 0.5 + O 0.5, Y_e = 0.5) or neutron matter (X_n = 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matter {
    CarbonOxygen,
    Neutron,
}

impl Matter {
    fn composition(self, elements: &Elements) -> Result<[f64; N_SPECIES], CalError> {
        let index = |key: &str| elements.index_of(key).ok_or_else(|| CalError::Failed(format!("elements.json has no {key}")));
        let mut x = [0.0; N_SPECIES];
        match self {
            Matter::CarbonOxygen => {
                x[index("C")?] = 0.5;
                x[index("O")?] = 0.5;
            }
            Matter::Neutron => x[index("n")?] = 1.0,
        }
        Ok(x)
    }

    /// Electrons per nucleon, neutron fraction and 1/μ of this matter, through the registry's own composition.
    fn cold(self, elements: &Elements) -> Result<ColdMatter, CalError> {
        let comp = elements.composition(&self.composition(elements)?);
        Ok(ColdMatter { y_e: comp.y_e, x_n: if self == Matter::Neutron { 1.0 } else { 0.0 }, inv_mu: comp.inv_mu })
    }
}

/// Why a calibrate run stopped; each is an exit code of §3.3.
#[derive(Debug, PartialEq)]
pub enum CalError {
    /// Exit 4.
    Args(String),
    /// Exit 6.
    Failed(String),
    /// Exit 3: no adapter, or a GPU error in a run.
    Gpu(String),
    /// Exit 5: a run found a non-finite value in the state.
    NonFinite(String),
}

impl CalError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CalError::Args(_) => EXIT_ARGS,
            CalError::Failed(_) => EXIT_CALIBRATION,
            CalError::Gpu(_) => EXIT_GPU,
            CalError::NonFinite(_) => EXIT_NON_FINITE,
        }
    }

    pub fn line(&self) -> String {
        match self {
            CalError::Args(m) => format!("SR-ERROR bad arguments: {m}"),
            CalError::Failed(m) => format!("SR-ERROR calibration failed: {m}"),
            CalError::Gpu(m) | CalError::NonFinite(m) => format!("SR-ERROR calibration run: {m}"),
        }
    }
}

impl std::fmt::Display for CalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.line())
    }
}

impl std::error::Error for CalError {}

fn constant(physics: &Physics, name: &str) -> Result<f64, CalError> {
    physics.get(name).ok_or_else(|| CalError::Failed(format!("physics.json has no {name}")))
}

/// m_ch_sb (§2.11 item 1) for `physics`: the sheet's C/O ceiling, the value `--only m_ch_sb` writes (a few hundredths of a
/// second, no adapter). The observers' white dwarf and planetary nebula rows read it (§0.5 [M0-D81]).
pub fn m_ch_sb(physics: &Physics) -> Result<f64, CalError> {
    let elements = Elements::shipped().map_err(|e| CalError::Failed(e.to_string()))?;
    let eos = Eos::from_physics(physics).map_err(|e| CalError::Failed(e.to_string()))?;
    let ceiling = sheet::ceiling(&eos, Matter::CarbonOxygen.cold(&elements)?, constant(physics, "sigma_vac")?);
    ceiling.map(|c| c.limit).map_err(|e| CalError::Failed(format!("m_ch_sb: {e}")))
}

// ---------------------------------------------------------------------------------------------------------------
// The grid's own cold disk (G-CORE (d))

/// A cold disk's hydrostatic equilibrium on the grid, by the self-consistent-field iteration: φ from Σ through the grid's own
/// kernel (§1.4.2, the CPU twin's [`Gravity`]), then Σ = h⁻¹(C − φ) with the constant C fixing the mass, mixed with the last Σ.
pub struct Equilibrium {
    pub sigma: Vec<f64>,
    pub iterations: usize,
}

const SCF_MAX_ITERATIONS: usize = 6_000;
/// The iteration has settled when no cell's Σ moves by more than this share of Σ_max in one round.
const SCF_TOLERANCE: f64 = 1e-9;
/// A central Σ past this is a collapse, not an equilibrium.
const SCF_COLLAPSED_SIGMA: f64 = 1e6;
/// The enthalpy table is read up to this Σ.
const SCF_TOP_SIGMA: f64 = 1e8;

/// `None` when it does not settle: no hydrostatic state of this mass exists on the grid.
pub fn equilibrium(mass: f64, matter: Matter, physics: &Physics, elements: &Elements) -> Result<Option<Equilibrium>, CalError> {
    let eos = Eos::from_physics(physics).map_err(|e| CalError::Failed(e.to_string()))?;
    let vac = constant(physics, "sigma_vac")?;
    let cold = matter.cold(elements)?;
    let eps_th = eos.t_floor() * cold.inv_mu;
    let table = Fluid::new({
        let eos = eos.clone();
        move |sigma| eos.pi(sigma, eps_th, cold.y_e, cold.x_n)
    }, vac);
    let top = table.enthalpy(SCF_TOP_SIGMA);
    let (w, h) = (WORLD as usize, WORLD as usize);
    let mut gravity = Gravity::new(G);
    let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
    // Start from a Gaussian a few cells wide.
    let mut sigma: Vec<f64> = (0..w * h)
        .map(|i| {
            let (dx, dy) = ((i % w) as f64 + 0.5 - cx, (i / w) as f64 + 0.5 - cy);
            (-(dx * dx + dy * dy) / 16.0).exp()
        })
        .collect();
    let total: f64 = sigma.iter().sum();
    sigma.iter_mut().for_each(|s| *s *= mass / total);
    let (mut omega, mut previous, mut rising) = (0.5f64, f64::INFINITY, 0);
    for iteration in 1..=SCF_MAX_ITERATIONS {
        let source: Vec<f64> = sigma.iter().map(|&s| if s >= vac { s } else { 0.0 }).collect();
        let phi = gravity.potential(&source, w, h).map_err(|e| CalError::Failed(e.to_string()))?;
        let phi_min = phi.iter().copied().fold(f64::INFINITY, f64::min);
        // The constant C that makes Σ(C) = mass: Σ(C) grows with C, 0 at C = φ_min.
        let (mut lo, mut hi) = (phi_min, phi_min + top);
        let mass_of = |c: f64| phi.iter().map(|&p| table.sigma_at(c - p)).sum::<f64>();
        if mass_of(hi) < mass {
            return Ok(None);
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if mass_of(mid) < mass {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let c = 0.5 * (lo + hi);
        let next: Vec<f64> = phi.iter().map(|&p| table.sigma_at(c - p)).collect();
        let peak = sigma.iter().copied().fold(0.0, f64::max);
        let err = sigma.iter().zip(&next).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max) / peak;
        if err < SCF_TOLERANCE {
            return Ok(Some(Equilibrium { sigma: next, iterations: iteration }));
        }
        if err >= previous {
            rising += 1;
            if rising >= 3 {
                omega = (omega * 0.5).max(0.01);
                rising = 0;
            }
        } else {
            rising = 0;
        }
        previous = err;
        for (s, n) in sigma.iter_mut().zip(&next) {
            *s = (1.0 - omega) * *s + omega * n;
        }
        if sigma.iter().copied().fold(0.0, f64::max) > SCF_COLLAPSED_SIGMA {
            return Ok(None);
        }
    }
    Ok(None)
}

/// The equilibrium disk of `mass` as a scene — gravity and hydro on and the rest off, bounce edges (no mass leaves), at
/// T_floor, one 1 × 1 rect per cell of Σ ≥ Σ_vac so the scene's own loader does the energies — and its planes. `physics` is
/// the shipped constants; `overrides` go on top of them for the solve and stay in the scene. `None` when no equilibrium
/// exists ([`equilibrium`]).
pub fn cold_disk_scene(
    mass: f64,
    matter: Matter,
    physics: &Physics,
    elements: &Elements,
    overrides: &[(String, f64)],
) -> Result<Option<(Scene, Vec<f32>)>, CalError> {
    let pairs: Vec<(&str, f64)> = overrides.iter().map(|(k, x)| (k.as_str(), *x)).collect();
    let physics = physics.with(&pairs).map_err(|e| CalError::Failed(format!("overrides: {e}")))?;
    let world = WorldConfig::new(WORLD, WORLD).map_err(|e| CalError::Failed(e.to_string()))?;
    let (t_floor, vac) = (constant(&physics, "t_floor")?, constant(&physics, "sigma_vac")?);
    let composition = matter.composition(elements)?;
    let Some(eq) = equilibrium(mass, matter, &physics, elements)? else { return Ok(None) };
    let w = WORLD as usize;
    let gas = Gas { heat: Heat::T(t_floor), composition, v: [0.0, 0.0] };
    let objects = eq
        .sigma
        .iter()
        .enumerate()
        .filter(|&(_, &s)| s >= vac)
        .map(|(i, &s)| Object::Rect { x0: (i % w) as i64, y0: (i / w) as i64, w: 1, h: 1, sigma: s, gas })
        .collect();
    let scene = Scene {
        world,
        edge: EdgeMode::Bounce,
        slowdown: false,
        switches: Switches { gravity: true, hydro: true, heat: false, reactions: false, neutrinos: false, sinks: false },
        test: TestSettings::default(),
        objects,
        overrides: overrides.to_vec(),
    };
    let (planes, _) = scene.planes(world, &physics, elements).map_err(CalError::Failed)?;
    Ok(Some((scene, planes)))
}

/// The radius about the centre of mass holding `fraction` of the mass: each cell of Σ ≥ `vac` spread over 4 × 4 points of
/// its area, the crossing interpolated between the sorted radii (the rule of G-COLL's r₅₀, M0-T36, at 0.9).
pub fn radius_holding(sigma: &[f32], width: usize, vac: f64, fraction: f64) -> f64 {
    const SUB: usize = 4;
    let cells: Vec<(f64, f64, f64)> = sigma
        .iter()
        .enumerate()
        .map(|(i, &s)| (i, f64::from(s)))
        .filter(|&(_, s)| s >= vac)
        .map(|(i, s)| ((i % width) as f64 + 0.5, (i / width) as f64 + 0.5, s))
        .collect();
    let mass: f64 = cells.iter().map(|c| c.2).sum();
    if cells.is_empty() || mass <= 0.0 {
        return 0.0;
    }
    let cx = cells.iter().map(|c| c.0 * c.2).sum::<f64>() / mass;
    let cy = cells.iter().map(|c| c.1 * c.2).sum::<f64>() / mass;
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(cells.len() * SUB * SUB);
    for &(x, y, s) in &cells {
        for a in 0..SUB {
            for b in 0..SUB {
                let px = x - 0.5 + (a as f64 + 0.5) / SUB as f64;
                let py = y - 0.5 + (b as f64 + 0.5) / SUB as f64;
                points.push(((px - cx).hypot(py - cy), s / (SUB * SUB) as f64));
            }
        }
    }
    points.sort_by(|p, q| p.0.total_cmp(&q.0));
    let (mut below, mut r_prev) = (0.0, 0.0);
    for (r, m) in points {
        if below + m >= fraction * mass {
            return r_prev + (r - r_prev) * (fraction * mass - below) / m;
        }
        below += m;
        r_prev = r;
    }
    r_prev
}

// ---------------------------------------------------------------------------------------------------------------
// Items 3–4: the thresholds, by bisection over whole lives (M0-T68)

/// §2.11 items 3–4: a bisection is done when the bracket is within this share of its lower end.
pub const BISECT_TOLERANCE: f64 = 0.02;

/// Which side of a threshold a run's mass fell on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Under the threshold: no ignition · a white dwarf · a neutron star.
    Light,
    /// Over it: ignition · core collapse · a black hole.
    Heavy,
}

/// What [`bisect`] found: the geometric midpoint of the final bracket, the bracket, and the runs it cost.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub value: f64,
    pub bracket: (f64, f64),
    pub runs: usize,
}

/// Bisects `lo..hi` (sandbox mass) for the mass at which `probe` turns from [`Side::Light`] to [`Side::Heavy`], to `tolerance`
/// (hi/lo − 1). Both ends are run first: ends on the same side — or the wrong way round — mean the threshold is not in the
/// bracket (or the item is not monotone) and the driver refuses with the two runs spent, never bisecting on a guess. A failing
/// probe stops the bisection with its error: nothing is guessed from a run that did not reach a side.
pub fn bisect(
    (lo, hi): (f64, f64),
    tolerance: f64,
    probe: &mut dyn FnMut(f64) -> Result<Side, CalError>,
) -> Result<Found, CalError> {
    if !(lo.is_finite() && hi.is_finite() && 0.0 < lo && lo < hi) {
        return Err(CalError::Args(format!("bracket {lo}..{hi}: needs 0 < LO < HI")));
    }
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(CalError::Args(format!("tolerance {tolerance}: needs a share above 0")));
    }
    let (at_lo, at_hi) = (probe(lo)?, probe(hi)?);
    if (at_lo, at_hi) != (Side::Light, Side::Heavy) {
        let why = if at_lo == at_hi { "agree" } else { "are the wrong way round" };
        return Err(CalError::Failed(format!(
            "the bracket {lo:.1}..{hi:.1}: its ends {why} ({lo:.1} is {at_lo:?}, {hi:.1} is {at_hi:?}), so the threshold is not in it — widen it with --bracket (two runs spent)"
        )));
    }
    let (mut lo, mut hi, mut runs) = (lo, hi, 2);
    while hi / lo - 1.0 > tolerance {
        let mid = (lo * hi).sqrt();
        match probe(mid)? {
            Side::Light => lo = mid,
            Side::Heavy => hi = mid,
        }
        runs += 1;
    }
    Ok(Found { value: (lo * hi).sqrt(), bracket: (lo, hi), runs })
}

/// One headless life of a preset cloud, as summary.json recorded it.
#[derive(Clone, Debug, PartialEq)]
pub struct Life {
    pub mass: f64,
    /// The `ignition` event fired.
    pub ignited: bool,
    /// The `core_collapse` event fired.
    pub core_collapse: bool,
    /// Any object showed `neutron_star` or `black_hole` (or the life ended as one): a remnant of a collapse, which can show
    /// without the `core_collapse` event (M0-D75).
    pub collapsed: bool,
    /// The stage of the life's ending — the `ending` event of the run's heaviest object (`white_dwarf`, `neutron_star`,
    /// `black_hole`, `failed_star`), when it fired; a lighter object's ending is none of the life's (§0.5 [M0-D95]).
    pub ending: Option<String>,
    pub steps: u64,
    pub sim_time: f64,
    pub wall_s: f64,
}

impl Life {
    fn describe(&self) -> String {
        format!(
            "mass={:.3} ignited={} core_collapse={} ending={} steps={} sim_time={:.3} wall_s={:.1}",
            self.mass,
            self.ignited,
            self.core_collapse,
            self.ending.as_deref().unwrap_or("none"),
            self.steps,
            self.sim_time,
            self.wall_s
        )
    }
}

fn unplaced(item: &str, life: &Life) -> CalError {
    CalError::Failed(format!("{item}: the run at mass {:.3} is on neither side of the threshold ({}) — reported, not guessed", life.mass, life.describe()))
}

/// m_ign_sb: ignition is over the threshold; a failed star's ending is under it.
pub fn side_ign(life: &Life) -> Result<Side, CalError> {
    match (life.ignited, life.ending.as_deref()) {
        (true, _) => Ok(Side::Heavy),
        (false, Some("failed_star")) => Ok(Side::Light),
        _ => Err(unplaced("m_ign_sb", life)),
    }
}

/// m_up_sb: core collapse is over the threshold — the event, or a neutron star or a black hole shown or ended — and a white
/// dwarf ending is under it.
pub fn side_up(life: &Life) -> Result<Side, CalError> {
    match (life.core_collapse || life.collapsed, life.ending.as_deref()) {
        (true, _) => Ok(Side::Heavy),
        (false, Some("white_dwarf")) => Ok(Side::Light),
        _ => Err(unplaced("m_up_sb", life)),
    }
}

/// m_bh_sb: a black hole is over the threshold; a neutron star is under it.
pub fn side_bh(life: &Life) -> Result<Side, CalError> {
    match life.ending.as_deref() {
        Some("black_hole") => Ok(Side::Heavy),
        Some("neutron_star") => Ok(Side::Light),
        _ => Err(unplaced("m_bh_sb", life)),
    }
}

/// The three thresholds of items 3–4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Threshold {
    Ign,
    Up,
    Bh,
}

impl Threshold {
    pub const ALL: [Threshold; 3] = [Threshold::Ign, Threshold::Up, Threshold::Bh];

    pub fn item(self) -> &'static str {
        match self {
            Threshold::Ign => "m_ign_sb",
            Threshold::Up => "m_up_sb",
            Threshold::Bh => "m_bh_sb",
        }
    }

    /// The condition a life of this item stops on (§2.11 items 3–4; M0-D75).
    pub fn until(self) -> Until {
        match self {
            Threshold::Ign => Until::IgnitionOrEnding,
            Threshold::Up => Until::CollapseOrEnding,
            Threshold::Bh => Until::Ending,
        }
    }

    pub fn side(self, life: &Life) -> Result<Side, CalError> {
        match self {
            Threshold::Ign => side_ign(life),
            Threshold::Up => side_up(life),
            Threshold::Bh => side_bh(life),
        }
    }

    /// Where the search starts, in sandbox mass, when `--bracket` does not say. M₀ is the nominal Sun-like cloud (it ignites
    /// and ends a white dwarf, M0-D50); a preset's radius is capped at 150 cells from 14 M₀ (§2.10). m_bh starts at 1.25 ×
    /// the file's m_up_sb (the close-call band's factor, §1.9.5), else 2 M₀. A wrong guess costs two runs: [`bisect`] refuses.
    pub fn default_bracket(self, file: &Map<String, Value>) -> (f64, f64) {
        match self {
            Threshold::Ign => (0.05 * UNIT_MASS, UNIT_MASS),
            Threshold::Up => (UNIT_MASS, 14.0 * UNIT_MASS),
            Threshold::Bh => (file.get("m_up_sb").and_then(Value::as_f64).map_or(2.0 * UNIT_MASS, |m| 1.25 * m), 20.0 * UNIT_MASS),
        }
    }
}

/// Measures one threshold: [`bisect`] over `live`, each run sided by the item's own mapping and logged behind `say`.
pub fn measure(
    threshold: Threshold,
    bracket: (f64, f64),
    live: &mut dyn FnMut(Threshold, f64) -> Result<Life, CalError>,
    say: &mut dyn FnMut(&str),
) -> Result<Found, CalError> {
    let item = threshold.item();
    bisect(bracket, BISECT_TOLERANCE, &mut |mass| {
        let life = live(threshold, mass)?;
        let side = threshold.side(&life);
        let shown = side.as_ref().map_or_else(|_| "none".to_string(), |s| format!("{s:?}"));
        say(&format!("SR-CALIBRATE run item={item} side={shown} {}", life.describe()));
        side
    })
}

/// A life from the lines and the exit code of a headless run: exit 2 is no ending within `--max-steps`.
fn life_error(code: i32, lines: &[String], max_steps: u64, mass: f64, record: &Path) -> CalError {
    let last_error = lines.iter().rev().find(|l| l.starts_with("SR-ERROR")).map(|l| l.trim_start_matches("SR-ERROR ").to_string());
    let said = last_error.unwrap_or_else(|| format!("exit {code}"));
    let kept = format!(" (record kept: {})", record.display());
    match code {
        EXIT_UNMET => CalError::Failed(format!("the run at mass {mass:.3} reached no ending within --max-steps {max_steps} — reported, not guessed{kept}")),
        EXIT_GPU => CalError::Gpu(said),
        EXIT_NON_FINITE => CalError::NonFinite(format!("the run at mass {mass:.3}: {said}{kept}")),
        _ => CalError::Failed(format!("the run at mass {mass:.3}: headless {said}{kept}")),
    }
}

/// `<life>/progress.json` (M0-D74): where the running life is, written through a temp file and a rename so a reader (the status
/// script, from a second terminal) never sees half of it. Best effort — a life never fails over its progress note.
fn write_progress(path: &Path, record: &Value) {
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, record.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

/// A progress tick's numbers (M0-D80), as progress.json writes them: the frame (`frame` steps, its mean `dt`, `s_per_step` wall
/// seconds — display only — and `p5_ceiling`, its steps at P5's ceiling), the world (`escaped` mass, `objects`) and the heaviest
/// object (§1.9.2, sandbox units; `raw` its stage before the hold). The terminal shows them as [`tick_rows`].
fn tick_numbers(t: &Tick) -> Vec<(&'static str, Value)> {
    let mut v: Vec<(&'static str, Value)> = vec![
        ("frame", json!(t.frame)),
        ("dt", json!(t.dt)),
        ("s_per_step", json!(t.s_per_step)),
        ("p5_ceiling", json!(t.p5_ceiling)),
        ("escaped", json!(t.escaped)),
        ("objects", json!(t.objects)),
    ];
    if let Some(s) = &t.star {
        v.extend([
            ("M", json!(s.mass)),
            ("R", json!(s.radius)),
            ("tau_dyn", json!(s.tau_dyn)),
            ("sigma_c", json!(s.sigma_c)),
            ("T_c", json!(s.t_c)),
            ("d_c", json!(s.d_c)),
            ("Ye_c", json!(s.ye_c)),
            ("X_Fe_c", json!(s.x_fe_c)),
            ("X_n_c", json!(s.x_n_c)),
            ("v_r", json!(s.v_r)),
            ("v_r_dyn", json!(s.v_r_dyn)),
            ("M_unb", json!(s.unbound)),
            ("L", json!(s.luminosity)),
            ("L_nuc", json!(s.l_nuc)),
            ("L_N", json!(s.l_n)),
            ("L_nu", json!(s.l_nu)),
            ("sinks", json!(s.sinks)),
            ("sink_M", json!(s.sink_mass)),
            ("raw", json!(s.raw.unwrap_or("none"))),
        ]);
    }
    v
}

/// progress.json's record with the last tick's [`tick_numbers`] beside its own keys.
fn with_numbers(mut record: Value, numbers: &[(&'static str, Value)]) -> Value {
    for (key, value) in numbers {
        record[*key] = value.clone();
    }
    record
}

/// The rows under a progress line (M0-D80), each a label and four columns of a [`tick_numbers`] key and the name it shows; a
/// row whose keys a tick lacks (no object yet) is left out.
const ROWS: [(&str, [(&str, &str); 4]); 6] = [
    ("frame", [("frame", "steps"), ("dt", "dt"), ("s_per_step", "s/step"), ("p5_ceiling", "P5_ceil")]),
    ("star", [("M", "M"), ("R", "R"), ("tau_dyn", "tau_dyn"), ("M_unb", "M_unb")]),
    ("core", [("sigma_c", "sigma_c"), ("T_c", "T_c"), ("d_c", "d_c"), ("Ye_c", "Ye_c")]),
    ("centre", [("X_Fe_c", "X_Fe"), ("X_n_c", "X_n"), ("v_r", "v_r"), ("v_r_dyn", "vr/vdyn")]),
    ("light", [("L", "L"), ("L_nuc", "L_nuc"), ("L_N", "L_N"), ("L_nu", "L_nu")]),
    ("world", [("escaped", "escaped"), ("objects", "objects"), ("sinks", "sinks"), ("sink_M", "sink_M")]),
];

/// A number as the rows show it: an integer as it is, else 4 significant digits, very large or small ones as `1.234e5`.
fn shown(value: &Value) -> String {
    match value {
        Value::Number(n) if n.is_f64() => {
            let x = n.as_f64().unwrap_or(f64::NAN);
            if x == 0.0 {
                "0".to_string()
            } else if (1e-3..1e5).contains(&x.abs()) {
                format!("{x:.*}", (3 - x.abs().log10().floor() as i32).max(0) as usize)
            } else {
                format!("{x:.3e}")
            }
        }
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// One cell of a row: a name in 7 columns, a value right-aligned in 9.
fn cell(name: &str, value: &str) -> String {
    format!("{name:<7} {value:>9}")
}

/// The lines under a progress line (M0-D80): one row per theme of [`ROWS`], `  <label> <name> <value> …` in a fixed grid so that
/// the same number sits in the same column from one step to the next, 80 columns.
fn tick_rows(numbers: &[(&'static str, Value)]) -> Vec<String> {
    let find = |key: &str| numbers.iter().find(|(k, _)| *k == key).map(|(_, v)| v);
    ROWS.iter()
        .filter_map(|(label, columns)| {
            let cells: Option<Vec<String>> = columns.iter().map(|(key, name)| find(key).map(|v| cell(name, &shown(v)))).collect();
            cells.map(|c| format!("  {label:<6} {}", c.join(" ")))
        })
        .collect()
}

/// How often a life says where it is by default: one `SR-CALIBRATE progress` line every this many steps (M0-D60);
/// `--progress-every N` replaces it for the invocation (M0-D75).
pub const PROGRESS_EVERY: u64 = 10_000;

/// How often a running life saves its checkpoint by default: every this many steps; `--checkpoint-every M` replaces it for the
/// invocation (M0-D77), apart from `--progress-every`.
pub const CHECKPOINT_EVERY: u64 = 10_000;

/// A checkpoint `--resume` continues a cut life from (M0-D77): the dump of `step` in `<life>/checkpoint/`, its snapshot beside it.
struct Checkpoint {
    dump: PathBuf,
    step: u64,
}

/// The preset cloud of `mass` run on the GPU until `until`, its summary.json read back. `dir` holds the invocation's runs, one
/// folder each (`<item>-<NN>`) that keeps summary.json and events.jsonl — the final dump alone is removed — and `record` is the
/// last run's folder, which the run line names. The adapter's name is `adapter_used` once a run has opened one.
struct Lives<'a> {
    dir: PathBuf,
    say: &'a dyn Fn(&str),
    record: &'a RefCell<String>,
    progress_every: u64,
    checkpoint_every: u64,
    adapter: Option<String>,
    adapter_used: Option<String>,
    max_steps: u64,
    overrides: &'a [(String, f64)],
    runs: usize,
    /// The largest rms gas speed any run of this process showed (§2.11 item 6).
    gas_peak: f64,
    /// `--resume`: `dir` is an earlier invocation's, and its finished lives are read back until one is missing (M0-D72).
    replaying: bool,
    /// Whether the run just made was read back instead of run; the run line says so.
    replayed: &'a Cell<bool>,
    /// What the run line adds for the run just made: `continued from step <k>`, or why a checkpoint was not used (M0-D77).
    note: &'a RefCell<String>,
}

impl Lives<'_> {
    fn live(&mut self, threshold: Threshold, mass: f64) -> Result<Life, CalError> {
        Ok(self.run(threshold.item(), "sun", threshold.until(), mass)?.life)
    }

    /// Item 6's life of the preset `name` at `mass`, run to its ending.
    fn preset(&mut self, name: &str, mass: f64) -> Result<PresetRun, CalError> {
        self.run(&format!("preset_{name}"), name, Until::Ending, mass)
    }

    fn run(&mut self, label: &str, preset: &str, until: Until, mass: f64) -> Result<PresetRun, CalError> {
        self.runs += 1;
        let out = self.dir.join(format!("{label}-{:02}", self.runs));
        self.replayed.set(false);
        self.note.borrow_mut().clear();
        // The first run with no finished record continues from its checkpoint when one is there and matches (M0-D77).
        let mut resuming = false;
        if self.replaying {
            match self.replay(&out, label, mass)? {
                Some(run) => {
                    self.replayed.set(true);
                    *self.record.borrow_mut() = out.display().to_string();
                    return Ok(run);
                }
                // Every run after it runs fresh in `dir`.
                None => (self.replaying, resuming) = (false, true),
            }
        }
        let scene_label = format!("calibrate:{label}:{mass:.3}");
        let checkpoint_dir = out.join(CHECKPOINT_DIR);
        let from = if resuming {
            self.checkpoint(&checkpoint_dir, &scene_label).unwrap_or_else(|why| {
                (self.say)(&format!("SR-WARN checkpoint not used ({why}): the {label} life at mass {mass:.3} runs from step 0"));
                *self.note.borrow_mut() = format!(" checkpoint not used: {why}");
                None
            })
        } else {
            None
        };
        if let Some(c) = &from {
            (self.say)(&format!("SR-CALIBRATE continue item={label} mass={mass:.3} from step {} ({})", c.step, c.dump.display()));
            *self.note.borrow_mut() = format!(" continued from step {}", c.step);
        }
        let mut scene = Scene::preset_mass(preset, mass, WorldConfig::default()).map_err(CalError::Failed)?;
        scene.overrides = self.overrides.to_vec();
        let args = Args {
            scene: scene_label,
            steps: None,
            until: Some(until),
            world: None,
            edge: None,
            max_steps: self.max_steps,
            out: Some(out.clone()),
            adapter: self.adapter.clone(),
            timing: false,
            dump_every: None,
            load_dump: from.as_ref().map(|c| c.dump.clone()),
            continue_life: from.is_some(),
            cpu_reference: false,
            edits: None,
        };
        let say = self.say;
        let started = utc_iso(SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()));
        let progress_file = out.join("progress.json");
        // The stages the life has shown and the first tick each was seen at (a tick is every `progress_every` steps); a continued
        // life starts from the cut run's trail up to the checkpoint, which its own ticks go on from.
        let carried: Vec<Value> = from
            .as_ref()
            .and_then(|c| {
                let text = std::fs::read_to_string(&progress_file).ok()?;
                let old: Value = serde_json::from_str(&text).ok()?;
                let trail = old["stages"].as_array()?.iter().filter(|s| s["since_step"].as_u64().is_some_and(|at| at <= c.step));
                Some(trail.cloned().collect())
            })
            .unwrap_or_default();
        let stages = RefCell::new(carried);
        // The last checkpoint saved, and where this process's life began (a checkpoint's step, else none).
        let checkpoint_step = Cell::new(from.as_ref().map(|c| c.step));
        let continued_from = from.as_ref().map(|c| c.step);
        let every = self.progress_every;
        // The last tick's frame and star numbers (M0-D80), which progress.json carries beside its own keys.
        let numbers = RefCell::new(Vec::new());
        let note = |step: u64, sim_time: f64, stage: &str| {
            let record = json!({"item": label, "mass": mass, "step": step, "sim_time": sim_time, "stage": stage, "stages": *stages.borrow(), "heat_unconverged": unconverged_steps(), "started": started, "progress_every": every, "continued_from_step": continued_from, "checkpoint_step": checkpoint_step.get()});
            write_progress(&progress_file, &with_numbers(record, &numbers.borrow()));
        };
        let mut tick = |t: &Tick| {
            let (step, sim_time, stage) = (t.step, t.sim_time, t.stage);
            {
                let mut stages = stages.borrow_mut();
                if stages.last().and_then(|s| s["stage"].as_str()) != Some(stage) {
                    stages.push(json!({"stage": stage, "since_step": step}));
                }
            }
            *numbers.borrow_mut() = tick_numbers(t);
            // Heat warnings are counted, not printed (M0-D74): the line says how many steps reached P5's ceiling so far.
            let unconverged = unconverged_steps();
            // The heaviest object's raw stage beside the shown one, then the frame's, the star's and the world's numbers in rows (M0-D80).
            let raw = t.star.as_ref().map(|s| format!(" | raw={}", s.raw.unwrap_or("none"))).unwrap_or_default();
            say(&format!("SR-CALIBRATE progress item={label} mass={mass:.3} step={step} sim_time={sim_time:.3} heat_unconverged={unconverged} stage={stage}{raw}"));
            for row in tick_rows(&numbers.borrow()) {
                say(&row);
            }
            note(step, sim_time, stage);
        };
        // A saved checkpoint moves progress.json's `checkpoint_step` at once, not at the next tick: what a stop would lose.
        let mut saved = |step: u64, sim_time: f64, stage: &str| {
            checkpoint_step.set(Some(step));
            note(step, sim_time, stage);
        };
        let progress = Progress { every, tick: &mut tick, checkpoint_every: Some(self.checkpoint_every), saved: &mut saved };
        let (code, lines) = run_scene_captured(args, scene, Some(progress));
        *self.record.borrow_mut() = out.display().to_string();
        let read = (code == EXIT_DONE).then(|| read_life(&out, mass));
        // The final dump is 13.4 MB a life; summary.json and events.jsonl stay (M0-D60). A finished life's progress.json goes
        // too: the file names a life that is running (M0-D74), and a failed life's stays as the trace of where it stopped.
        let _ = std::fs::remove_dir_all(out.join("dumps"));
        // The checkpoint goes with a finished life and stays with a failed or a cut one (M0-D77).
        if code == EXIT_DONE {
            let _ = std::fs::remove_file(&progress_file);
            let _ = std::fs::remove_dir_all(&checkpoint_dir);
        }
        match read {
            Some(run) => {
                let (run, adapter) = run?;
                self.adapter_used.get_or_insert(adapter);
                self.gas_peak = self.gas_peak.max(run.max_gas_speed);
                Ok(run)
            }
            None => Err(life_error(code, &lines, self.max_steps, mass, &out)),
        }
    }
}

impl Lives<'_> {
    /// `--resume` (M0-D72): the life a finished record in `out` holds, when `out/summary.json` parses and its `until` was met; none
    /// when there is no such record (a file cut while it was written does not parse: the life runs fresh). A record whose `scene`
    /// is not this run's label and mass, or whose adapter `--adapter` does not match, is not this calibration's: refused (exit 4)
    /// before any life runs, as is a folder with no first life of this run's label. The match on every run is the guard, never
    /// the run number alone.
    fn replay(&mut self, out: &Path, label: &str, mass: f64) -> Result<Option<PresetRun>, CalError> {
        let foreign = || CalError::Args(format!("{} is not this calibration's: start without --resume", self.dir.display()));
        let Ok(text) = std::fs::read_to_string(out.join("summary.json")) else {
            // The first life's folder is made before it runs: a folder without it is another item's, never this one's.
            return if self.runs == 1 && !out.is_dir() { Err(foreign()) } else { Ok(None) };
        };
        let Ok(summary) = serde_json::from_str::<Value>(&text) else { return Ok(None) };
        if summary["until"]["met"] != true {
            return Ok(None);
        }
        if summary["scene"].as_str() != Some(format!("calibrate:{label}:{mass:.3}").as_str()) {
            return Err(foreign());
        }
        let (run, adapter) = life_of(&summary, mass);
        if self.adapter.as_ref().is_some_and(|q| !adapter.to_lowercase().contains(&q.to_lowercase())) {
            return Err(foreign());
        }
        self.adapter_used.get_or_insert(adapter);
        self.gas_peak = self.gas_peak.max(run.max_gas_speed);
        Ok(Some(run))
    }
}

impl Lives<'_> {
    /// `--resume` (M0-D77): the checkpoint in `folder` this life continues from — the newest whole pair that matches — or none
    /// when the life has none; `Err(why)` when a pair is there and none is usable. A pair is whole when its snapshot is beside
    /// its dump (the snapshot is renamed into place after the dump is written) and the dump reads; it matches when the
    /// snapshot's step is the dump's and the file name's, its scene label is this life's (the item and the mass), its adapter is
    /// the one `--adapter` names, its physics_hash the assets' and its world the default's.
    fn checkpoint(&self, folder: &Path, scene: &str) -> Result<Option<Checkpoint>, String> {
        let Ok(entries) = std::fs::read_dir(folder) else { return Ok(None) };
        let names: Vec<String> = entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        let mut steps: Vec<u64> = names.iter().filter_map(|n| n.strip_prefix("s_")?.strip_suffix(".observers.json")?.parse().ok()).collect();
        steps.sort_unstable_by(|a, b| b.cmp(a));
        let mut why = None;
        for step in steps {
            match self.pair(folder, step, scene) {
                Ok(c) => return Ok(Some(c)),
                Err(e) => {
                    why.get_or_insert(e);
                }
            }
        }
        match why {
            Some(why) => Err(why),
            None if names.iter().any(|n| n.starts_with("s_")) => Err("a dump with no snapshot beside it, a pair cut while it was written".into()),
            None => Ok(None),
        }
    }

    fn pair(&self, folder: &Path, step: u64, scene: &str) -> Result<Checkpoint, String> {
        let (dump, file) = (folder.join(format!("s_{step}.bin")), folder.join(format!("s_{step}.observers.json")));
        let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: not JSON: {e}", file.display()))?;
        if v["version"] != OBSERVERS_VERSION {
            return Err(format!("{} is version {}, this build reads {OBSERVERS_VERSION}", file.display(), v["version"]));
        }
        if v["step"].as_u64() != Some(step) {
            return Err(format!("{} holds step {}, beside a dump of step {step}", file.display(), v["step"]));
        }
        if v["scene"].as_str() != Some(scene) {
            return Err(format!("the checkpoint is of {}, this life is {scene:?}", v["scene"]));
        }
        let name = v["adapter"].as_str().unwrap_or("");
        if let Some(query) = self.adapter.as_ref().filter(|q| !name.to_lowercase().contains(&q.to_lowercase())) {
            return Err(format!("the checkpoint was made on {name:?}, --adapter names {query:?}"));
        }
        let hash = calibration::physics_hash().map_err(|e| e.line())?;
        if v["physics_hash"].as_str() != Some(hash.as_str()) {
            return Err(format!("the checkpoint's physics_hash is {}, the assets' {hash}", v["physics_hash"]));
        }
        let read = Dump::read(&dump).map_err(|e| format!("{}: {e}", dump.display()))?;
        let world = WorldConfig::default();
        if read.step != step || read.world != world || v["world"] != json!([world.width, world.height]) {
            return Err(format!("the dump {} is not step {step} of the {}x{} world", dump.display(), world.width, world.height));
        }
        Ok(Checkpoint { dump, step })
    }
}

/// summary.json of a finished run (§2.12.2): the events it fired and its cost, and the adapter's name.
fn read_life(dir: &Path, mass: f64) -> Result<(PresetRun, String), CalError> {
    let path = dir.join("summary.json");
    let text = std::fs::read_to_string(&path).map_err(|e| CalError::Failed(format!("{}: {e}", path.display())))?;
    let summary: Value = serde_json::from_str(&text).map_err(|e| CalError::Failed(format!("{}: {e}", path.display())))?;
    Ok(life_of(&summary, mass))
}

/// What a parsed summary.json says of the life at `mass`, and the adapter's name.
fn life_of(summary: &Value, mass: f64) -> (PresetRun, String) {
    let events: &[Value] = summary["events"].as_array().map_or(&[], Vec::as_slice);
    let fired = |kind: &str| events.iter().any(|e| e["kind"] == kind);
    // The star is the run's heaviest object (a wisp or a remnant's debris is lighter).
    let objects: &[Value] = summary["objects"].as_array().map_or(&[], Vec::as_slice);
    let star = objects.iter().max_by(|a, b| a["mass_sb"].as_f64().unwrap_or(0.0).total_cmp(&b["mass_sb"].as_f64().unwrap_or(0.0)));
    // The life's ending is the star's, a fragment's never (§0.5 [M0-D95]): the run stops on it, at a frame whose objects these are.
    let ending = star
        .and_then(|o| events.iter().find(|e| e["kind"] == "ending" && e["object"] == o["id"]))
        .and_then(|e| e["detail"]["stage"].as_str())
        .map(str::to_string);
    // Any object's trail, not the star's alone: the until stops on any object showing a collapse remnant (M0-D75).
    let collapsed_stage = |s: &str| s == "neutron_star" || s == "black_hole";
    let shown_collapsed = summary["objects"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|o| o["stages"].as_array().into_iter().flatten())
        .any(|e| e["stage"].as_str().is_some_and(collapsed_stage));
    let life = Life {
        mass,
        ignited: fired("ignition"),
        core_collapse: fired("core_collapse"),
        collapsed: shown_collapsed || ending.as_deref().is_some_and(collapsed_stage),
        ending,
        steps: summary["steps"].as_u64().unwrap_or(0),
        sim_time: summary["sim_time"].as_f64().unwrap_or(f64::NAN),
        wall_s: summary["wall_s"].as_f64().unwrap_or(f64::NAN),
    };
    let stages = star
        .and_then(|o| o["stages"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|e| Some((e["stage"].as_str()?.to_string(), e["sim_time"].as_f64()?)))
        .collect();
    let at_x_h = star.and_then(|o| Some((o["at_x_h"]["luminosity"].as_f64()?, o["at_x_h"]["radius"].as_f64()?)));
    let adapter = summary["adapter"]["name"].as_str().unwrap_or("unknown").to_string();
    (PresetRun { life, stages, at_x_h, max_gas_speed: summary["max_gas_speed"].as_f64().unwrap_or(0.0) }, adapter)
}

// ---------------------------------------------------------------------------------------------------------------
// Items 5–6: the order, the presets and what their lives say (M0-T70)

/// What one whole life left, as summary.json recorded it.
#[derive(Clone, Debug, PartialEq)]
pub struct PresetRun {
    pub life: Life,
    /// The star's displayed stages in order, each with the sim time of the first frame that showed it.
    pub stages: Vec<(String, f64)>,
    /// The star's luminosity and radius at the first frame its central hydrogen was at or under `T_EFF_X_H`.
    pub at_x_h: Option<(f64, f64)>,
    /// The largest block rms gas speed of the run.
    pub max_gas_speed: f64,
}

/// §2.10's presets: the name, the ending its mass is meant to be well inside of, and its mass in Suns.
pub const PRESET_TABLE: [(&str, &str, f64); 3] =
    [("sun", "white_dwarf", PRESET_SUNS[0]), ("massive", "neutron_star", PRESET_SUNS[1]), ("giant", "black_hole", PRESET_SUNS[2])];

/// §2.11 item 5 over the merged file: m_ign < m_ch < m_up < m_bh, m_ch < m_tov with m_tov/m_ch in K9's band, and sigma_bh
/// (§2.6) at or over ns_sigma_max_sb. An inequality is checked once both its sides are in the file; any failure is reported
/// with the numbers and never re-tuned (exit 6).
pub fn check_order(file: &Map<String, Value>, sigma_bh: f64) -> Result<(), CalError> {
    let item = |key: &str| file.get(key).and_then(Value::as_f64);
    let fail = |what: String| Err(CalError::Failed(format!("K9: {what} (reported, never re-tuned)")));
    for (lo, hi) in [("m_ign_sb", "m_ch_sb"), ("m_ch_sb", "m_up_sb"), ("m_up_sb", "m_bh_sb")] {
        if let (Some(a), Some(b)) = (item(lo), item(hi)) {
            if a >= b {
                return fail(format!("{lo} {a:.1} must be under {hi} {b:.1}"));
            }
        }
    }
    if let (Some(ch), Some(tov)) = (item("m_ch_sb"), item("m_tov_sb")) {
        let ratio = tov / ch;
        if ch >= tov || !(K9_RATIO.0..=K9_RATIO.1).contains(&ratio) {
            return fail(format!(
                "m_ch_sb {ch:.1}, m_tov_sb {tov:.1}, m_tov/m_ch {ratio:.3} — needs m_ch < m_tov and the ratio in [{:.1}, {:.1}]",
                K9_RATIO.0, K9_RATIO.1
            ));
        }
    }
    match item("ns_sigma_max_sb") {
        Some(ns) if sigma_bh < ns => fail(format!(
            "sigma_bh {sigma_bh} is under ns_sigma_max_sb {ns:.1} — a holding neutron star could trip a black hole (§2.6); raise sigma_bh"
        )),
        _ => Ok(()),
    }
}

/// §2.11 item 6's last bound: c_sb must exceed twice the fastest gas.
pub fn check_speed(max_gas_speed: f64, c_sb: f64) -> Result<(), CalError> {
    if c_sb > 2.0 * max_gas_speed {
        return Ok(());
    }
    Err(CalError::Failed(format!(
        "max_gas_speed {max_gas_speed:.3}: c_sb {c_sb} must exceed twice it, {:.3} (§2.11 item 6) — reported, never re-tuned",
        2.0 * max_gas_speed
    )))
}

/// What the three presets' lives gave: item 6's values.
#[derive(Clone, Debug, PartialEq)]
pub struct Presets {
    /// `preset_mass_sb` for sun, massive and giant.
    pub masses: [f64; 3],
    pub t_eff_sun_sb: f64,
    pub life_sun_sb: f64,
    pub top_rung: f64,
    pub stage_durations_sb: BTreeMap<String, Vec<(f64, f64)>>,
    /// The largest rms gas speed of the three lives.
    pub max_gas_speed: f64,
}

impl Presets {
    /// The values as §2.11 writes them, into the file's object (`max_gas_speed` is the caller's: it spans every run).
    fn insert_into(&self, file: &mut Map<String, Value>) {
        let [sun, massive, giant] = self.masses;
        file.insert("preset_mass_sb".into(), json!({"sun": sun, "massive": massive, "giant": giant}));
        file.insert("t_eff_sun_sb".into(), json!(self.t_eff_sun_sb));
        file.insert("life_sun_sb".into(), json!(self.life_sun_sb));
        file.insert("top_rung".into(), json!(self.top_rung));
        let stages: Map<String, Value> = self.stage_durations_sb.iter().map(|(id, pairs)| (id.clone(), json!(pairs))).collect();
        file.insert("stage_durations_sb".into(), Value::Object(stages));
    }
}

/// The stages that end a life or look like it for a while (`watch::is_remnant`, by id): their clock is §1.10.4's 10⁸ years per
/// t.u., so no duration is booked for them, the ending's or a transient one's (a bounce's dark centre shows `white_dwarf`).
const REMNANT_IDS: [&str; 4] = ["white_dwarf", "neutron_star", "black_hole", "failed_star"];

/// The sim time each stage of one life was shown (a stage shown twice adds up), from its first frame to the next stage's.
/// Remnant stages get none, wherever they come in the life.
pub fn stage_times(stages: &[(String, f64)]) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    for pair in stages.windows(2) {
        let (stage, next) = (&pair[0], &pair[1]);
        if next.1 > stage.1 && !REMNANT_IDS.contains(&stage.0.as_str()) {
            *out.entry(stage.0.clone()).or_insert(0.0) += next.1 - stage.1;
        }
    }
    out
}

/// Item 6: the presets' masses from the four anchors through the mass translation's inverse, each preset run by `live` to its
/// ending (which must be the one it promises), then the Sun-like life's `life_sun_sb`, `top_rung` and `t_eff_sun_sb`
/// (checked against the temperature scale before it can be written) and the three lives' stage durations.
pub fn measure_presets(
    anchors: [f64; 4],
    physics: &Physics,
    live: &mut dyn FnMut(&str, f64) -> Result<PresetRun, CalError>,
    say: &mut dyn FnMut(&str),
) -> Result<Presets, CalError> {
    let scale = MassScale::new(anchors[0], anchors[1], anchors[2], anchors[3]).map_err(|e| CalError::Failed(format!("preset_mass_sb: {e}")))?;
    let (mut masses, mut max_gas_speed) = ([0.0; 3], 0.0f64);
    let (mut life_sun, mut t_eff) = (f64::NAN, f64::NAN);
    let mut durations: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for (k, (name, ending, suns)) in PRESET_TABLE.into_iter().enumerate() {
        let item = format!("preset_{name}");
        let mass = scale.to_sb(suns);
        masses[k] = mass;
        let run = live(name, mass)?;
        say(&format!("SR-CALIBRATE run item={item} suns={suns} {}", run.life.describe()));
        if run.life.ending.as_deref() != Some(ending) {
            return Err(CalError::Failed(format!(
                "{item}: {suns} Suns at mass {mass:.1} ended as {}, not {ending} — the preset is not inside its ending (reported, not re-tuned)",
                run.life.ending.as_deref().unwrap_or("nothing")
            )));
        }
        max_gas_speed = max_gas_speed.max(run.max_gas_speed);
        for (stage, time) in stage_times(&run.stages) {
            durations.entry(stage).or_default().push((mass, time));
        }
        if k == 0 {
            // The last entry of the ending's stage: the `ending` event closes a hold of max(10 t.u., 20 τ_dyn) that a change of
            // stage breaks, so an earlier, transient showing of the same stage (a bounce's white dwarf) is not the life's end.
            life_sun = run.stages.iter().rev().find(|(stage, _)| stage == ending).map(|s| s.1).ok_or_else(|| {
                CalError::Failed(format!("life_sun_sb: the Sun-like life never showed {ending} (stages {:?})", run.stages))
            })?;
            let (l, r) = run.at_x_h.ok_or_else(|| {
                CalError::Failed(format!("t_eff_sun_sb: the Sun-like life never reached X_H,c {T_EFF_X_H} (central hydrogen did not fall that far)"))
            })?;
            let (a_r, c_sb) = (constant(physics, "a_r")?, constant(physics, "c_sb")?);
            t_eff = surface_temperature_sb(l, r, a_r, c_sb);
            let anchors = ["t_floor", "t_h", "t_he", "t_c", "t_ne", "t_o", "t_si"].map(|key| constant(physics, key));
            let [t_floor, t_h, t_he, t_c, t_ne, t_o, t_si] = anchors;
            TempScale::new([t_floor?, t_eff, t_h?, t_he?, t_c?, t_ne?, t_o?, t_si?])
                .map_err(|e| CalError::Failed(format!("t_eff_sun_sb {t_eff} (L {l}, R {r}) is off the temperature scale: {e}")))?;
        }
    }
    let top_rung = top_rung_from_life(life_sun);
    if !(top_rung.is_finite() && top_rung > 0.0) {
        return Err(CalError::Failed(format!("top_rung {top_rung} from life_sun_sb {life_sun}: not a positive finite speed")));
    }
    Ok(Presets { masses, t_eff_sun_sb: t_eff, life_sun_sb: life_sun, top_rung, stage_durations_sb: durations, max_gas_speed })
}

// ---------------------------------------------------------------------------------------------------------------
// The file

/// What one `--only` name asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Job {
    /// The sheet's ceiling for this matter (items 1–2).
    Sheet(Matter),
    /// A bisection on the GPU (items 3–4).
    Bisect(Threshold),
    /// The three presets' lives on the GPU (item 6): `preset_mass_sb`, `t_eff_sun_sb`, `life_sun_sb`, `top_rung`,
    /// `stage_durations_sb` and `max_gas_speed` all come from them.
    Presets,
}

/// `--only`'s item, by name: the job it asks for, or an error saying what the names are.
fn parse_only(name: &str) -> Result<Job, CalError> {
    if let Some(t) = Threshold::ALL.into_iter().find(|t| t.item() == name) {
        return Ok(Job::Bisect(t));
    }
    match name {
        "m_ch_sb" => Ok(Job::Sheet(Matter::CarbonOxygen)),
        "m_tov_sb" | "ns_sigma_max_sb" => Ok(Job::Sheet(Matter::Neutron)),
        "preset_mass_sb" | "t_eff_sun_sb" | "life_sun_sb" | "top_rung" | "stage_durations_sb" | "max_gas_speed" => Ok(Job::Presets),
        "render_reserve_ms" => Err(CalError::Args(
            "--only render_reserve_ms: the drawing lot's `--measure-ui` run writes it (§2.11 item 7, §3.3); calibrate keeps it".into(),
        )),
        _ => Err(CalError::Args(format!("--only {name:?}: not an item of §2.11 (built: {})", ITEMS.join(", ")))),
    }
}

/// Seconds since the epoch as `YYYY-MM-DDTHH:MM:SSZ` (UTC, §1).
pub fn utc_iso(secs: u64) -> String {
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Civil from days (H. Hinnant): the days since 1970-01-01 as a proleptic Gregorian date.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", rem / 3_600, rem % 3_600 / 60, rem % 60)
}

fn hostname() -> String {
    ["HOSTNAME", "COMPUTERNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.trim().is_empty()))
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "unknown".to_string())
}

/// The file at `path` as the base of this run: its object, when it parses and its `physics_hash` is `hash`; otherwise an
/// empty one (the second value says why a file was dropped).
fn base_of(path: &Path, hash: &str) -> (Map<String, Value>, Option<String>) {
    let Ok(text) = std::fs::read_to_string(path) else { return (Map::new(), None) };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) if map.get("physics_hash").and_then(Value::as_str) == Some(hash) => (map, None),
        Ok(Value::Object(_)) => (Map::new(), Some(format!("{} holds another physics_hash: its items are dropped", path.display()))),
        _ => (Map::new(), Some(format!("{} is not a JSON object: replaced", path.display()))),
    }
}

struct Opts {
    out: PathBuf,
    /// `--only`'s items, in the order given; none asks for every item built.
    only: Vec<Job>,
    adapter: Option<String>,
    max_steps: u64,
    bracket: Option<(f64, f64)>,
    /// `--resume DIR`: an earlier invocation's `build/calibrate_runs/<stamp>/`, whose finished lives are read back (M0-D72).
    resume: Option<PathBuf>,
    /// `--progress-every N`: steps between a life's progress lines and progress.json writes; none keeps the caller's default
    /// ([`PROGRESS_EVERY`]; M0-D75).
    progress_every: Option<u64>,
    /// `--checkpoint-every M`: steps between a life's checkpoints; none keeps [`CHECKPOINT_EVERY`] (M0-D77).
    checkpoint_every: Option<u64>,
}

/// `--adapter` names the GPU for items 3–4 and 6 and is otherwise unused (§3.3); `--only` takes one item or a comma list;
/// `--resume` and `--progress-every` are for the items that run lives and are ignored by items 1–2.
fn parse_args(args: &[String]) -> Result<Opts, CalError> {
    let (mut out, mut only, mut adapter, mut max_steps, mut bracket, mut resume) = (None, Vec::new(), None, DEFAULT_MAX_STEPS, None, None);
    let (mut progress_every, mut checkpoint_every) = (None, None);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (arg.as_str(), None),
        };
        if !["--adapter", "--out", "--only", "--max-steps", "--bracket", "--resume", "--progress-every", "--checkpoint-every"].contains(&flag) {
            return Err(CalError::Args(format!("unknown argument {arg:?}")));
        }
        let value = match inline {
            Some(v) => v,
            None => it.next().cloned().ok_or_else(|| CalError::Args(format!("{flag} needs a value")))?,
        };
        match flag {
            "--out" => out = Some(PathBuf::from(value)),
            "--only" => {
                for name in value.split(',') {
                    let job = parse_only(name.trim())?;
                    if !only.contains(&job) {
                        only.push(job);
                    }
                }
            }
            "--adapter" => adapter = Some(value),
            "--resume" => resume = Some(PathBuf::from(value)),
            "--progress-every" => {
                progress_every = Some(
                    value
                        .parse::<u64>()
                        .ok()
                        .filter(|n| *n > 0)
                        .ok_or_else(|| CalError::Args(format!("--progress-every takes a whole number of steps above 0, got {value:?}")))?,
                )
            }
            "--checkpoint-every" => {
                checkpoint_every = Some(
                    value
                        .parse::<u64>()
                        .ok()
                        .filter(|n| *n > 0)
                        .ok_or_else(|| CalError::Args(format!("--checkpoint-every takes a whole number of steps above 0, got {value:?}")))?,
                )
            }
            "--max-steps" => {
                max_steps = value.parse::<u64>().ok().filter(|n| *n > 0).ok_or_else(|| CalError::Args(format!("--max-steps takes a whole number above 0, got {value:?}")))?
            }
            _ => {
                let pair = value.split_once(',').and_then(|(lo, hi)| Some((lo.trim().parse::<f64>().ok()?, hi.trim().parse::<f64>().ok()?)));
                bracket = Some(pair.filter(|(lo, hi)| lo.is_finite() && hi.is_finite() && 0.0 < *lo && lo < hi).ok_or_else(|| {
                    CalError::Args(format!("--bracket takes LO,HI in sandbox mass with 0 < LO < HI, got {value:?}"))
                })?)
            }
        }
    }
    let out = out.ok_or_else(|| CalError::Args("--out is required (the calibration file to write)".into()))?;
    if bracket.is_some() && !matches!(only.as_slice(), [Job::Bisect(_)]) {
        return Err(CalError::Args("--bracket belongs to one bisected item: give --only m_ign_sb, m_up_sb or m_bh_sb".into()));
    }
    Ok(Opts { out, only, adapter, max_steps, bracket, resume, progress_every, checkpoint_every })
}

/// `calibrate`'s run on the shipped constants with `overrides` on top (tests only: a run with overrides never writes,
/// §2.12.1), behind `say`; returns the exit code. The file is written only when every measurement and check passed.
pub fn run_with(args: &[String], overrides: &[(String, f64)], say: &mut dyn FnMut(&str)) -> i32 {
    run_with_progress(args, overrides, PROGRESS_EVERY, say)
}

/// [`run_with`] with a life's progress line every `progress_every` steps instead of [`PROGRESS_EVERY`] (a test's short life);
/// `--progress-every` on the command line still wins.
pub fn run_with_progress(args: &[String], overrides: &[(String, f64)], progress_every: u64, say: &mut dyn FnMut(&str)) -> i32 {
    match run_inner(args, overrides, progress_every, say) {
        Ok(()) => EXIT_DONE,
        Err(e) => {
            say(&e.line());
            e.exit_code()
        }
    }
}

/// Runs the command; returns the process's exit code.
pub fn run(args: &[String]) -> i32 {
    // Colour only on a terminal, and never under NO_COLOR (no-color.org): a log kept with `| tee` or a redirect stays plain text.
    use std::io::IsTerminal;
    let colour = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    run_with(args, &[], &mut |line| if colour { println!("{}", paint(line)) } else { println!("{line}") })
}

/// ANSI colours for a terminal (M0-D80; the lead: «add some color too if you can»): a progress line's head in cyan, its shown
/// stage in green and its raw stage in magenta (red while core_collapse); a row's label in blue, its names dim, its values bold —
/// red where the physics is hard (P5_ceil above 0, L_N below 0), yellow for a one-step frame; `SR-CALIBRATE run` bold,
/// `SR-WARN` yellow, `SR-ERROR` red. The text is the same once the codes are stripped.
pub fn paint(line: &str) -> String {
    const RESET: &str = "\x1b[0m";
    if let Some(rest) = line.strip_prefix("SR-CALIBRATE progress") {
        let (fields, raw) = rest.split_once(" | raw=").map_or((rest, None), |(f, r)| (f, Some(r)));
        let (before, stage) = fields.split_once(" stage=").unwrap_or((fields, ""));
        let mut out = format!("\x1b[1;36mSR-CALIBRATE progress{RESET}{before} stage=\x1b[1;32m{stage}{RESET}");
        if let Some(raw) = raw {
            let tone = if raw == "core_collapse" { "1;31" } else { "35" };
            out.push_str(&format!(" | raw=\x1b[{tone}m{raw}{RESET}"));
        }
        return out;
    }
    let row = line.strip_prefix("  ").and_then(|r| r.split_once(' ')).filter(|(label, _)| ROWS.iter().any(|(l, _)| l == label));
    if let Some((label, cells)) = row {
        let words: Vec<&str> = cells.split_whitespace().collect();
        let painted: Vec<String> = words
            .chunks(2)
            .map(|pair| {
                let (name, value) = (pair[0], pair.get(1).copied().unwrap_or(""));
                let number = value.parse::<f64>().unwrap_or(0.0);
                let tone = match name {
                    "P5_ceil" if number > 0.0 => "1;31",
                    "L_N" if number < 0.0 => "1;31",
                    "steps" if number == 1.0 => "1;33",
                    _ => "1",
                };
                let plain = cell(name, value);
                let (n, v) = plain.split_at(7);
                format!("\x1b[2m{n}{RESET}\x1b[{tone}m{v}{RESET}")
            })
            .collect();
        return format!("  \x1b[1;34m{label:<6}{RESET} {}", painted.join(" "));
    }
    let tone = if line.starts_with("SR-ERROR") {
        "1;31"
    } else if line.starts_with("SR-WARN") {
        "33"
    } else if line.starts_with("SR-CALIBRATE run") {
        "1"
    } else {
        return line.to_string();
    };
    format!("\x1b[{tone}m{line}{RESET}")
}

fn run_inner(args: &[String], overrides: &[(String, f64)], progress_every: u64, say: &mut dyn FnMut(&str)) -> Result<(), CalError> {
    // `say` is shared by the command and the lives it runs (their progress lines come while they run).
    let say_cell = RefCell::new(say);
    let say = |line: &str| (say_cell.borrow_mut())(line);
    let opts = parse_args(args)?;
    let all = opts.only.is_empty();
    let asked = |job: Job| all || opts.only.contains(&job);
    let wanted: Vec<Matter> = [Matter::CarbonOxygen, Matter::Neutron].into_iter().filter(|m| asked(Job::Sheet(*m))).collect();
    let thresholds: Vec<Threshold> = Threshold::ALL.into_iter().filter(|t| asked(Job::Bisect(*t))).collect();
    let hash = calibration::physics_hash().map_err(|e| CalError::Failed(e.line()))?;
    let pairs: Vec<(&str, f64)> = overrides.iter().map(|(k, x)| (k.as_str(), *x)).collect();
    let physics = Physics::shipped().and_then(|p| p.with(&pairs)).map_err(|e| CalError::Failed(e.to_string()))?;
    let elements = Elements::shipped().map_err(|e| CalError::Failed(e.to_string()))?;
    let (eos, vac) = (Eos::from_physics(&physics).map_err(|e| CalError::Failed(e.to_string()))?, constant(&physics, "sigma_vac")?);
    let (mut file, dropped) = base_of(&opts.out, &hash);
    if let Some(why) = dropped {
        say(&format!("SR-WARN {why}"));
    }
    let started = Instant::now();
    for matter in wanted {
        let items: &[&str] = if matter == Matter::Neutron { &["m_tov_sb", "ns_sigma_max_sb"] } else { &["m_ch_sb"] };
        let t0 = Instant::now();
        let ceiling = sheet::ceiling(&eos, matter.cold(&elements)?, vac).map_err(|e| CalError::Failed(format!("{}: {e}", items[0])))?;
        let wall = t0.elapsed().as_secs_f64();
        let worst_virial = ceiling.sequence.iter().map(|e| e.virial().abs()).fold(0.0, f64::max);
        // ns_sigma_max_sb: m_tov_sb per cell area, a cell's area being 1.
        for item in items {
            file.insert((*item).to_string(), json!(ceiling.limit));
            say(&format!("SR-CALIBRATE item={item} value={:.6} worst_virial={worst_virial:.2e} wall_s={wall:.1}", ceiling.limit));
        }
    }
    // Items 3–4: whole lives on the GPU, in the order of the ladder (m_bh's default bracket reads m_up_sb).
    let record = RefCell::new(String::new());
    let replayed = Cell::new(false);
    let note = RefCell::new(String::new());
    let stamp = utc_iso(SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())).replace(':', "-");
    // `--resume` names the folder an earlier invocation made and continues in it; it is checked before any life runs.
    let runs_lives = !thresholds.is_empty() || asked(Job::Presets);
    if let Some(dir) = opts.resume.as_ref().filter(|d| runs_lives && !d.is_dir()) {
        return Err(CalError::Args(format!("--resume {}: not a folder — it is an earlier run's build/calibrate_runs/<stamp>/", dir.display())));
    }
    let mut lives = Lives {
        dir: opts.resume.clone().unwrap_or_else(|| Path::new("build").join("calibrate_runs").join(stamp)),
        say: &say,
        record: &record,
        progress_every: opts.progress_every.unwrap_or(progress_every),
        checkpoint_every: opts.checkpoint_every.unwrap_or(CHECKPOINT_EVERY),
        adapter: opts.adapter.clone(),
        adapter_used: None,
        max_steps: opts.max_steps,
        overrides,
        runs: 0,
        gas_peak: 0.0,
        replaying: opts.resume.is_some(),
        replayed: &replayed,
        note: &note,
    };
    // A run line names the folder its life's record is kept in.
    let run_say = |line: &str| {
        if line.starts_with("SR-CALIBRATE run") {
            let replay = if replayed.get() { " replayed".to_string() } else { note.borrow().clone() };
            // A life's steps at P5's ceiling, counted not printed (M0-D74); a replayed life ran in an earlier process.
            let unconverged = unconverged_steps();
            let heat = if unconverged > 0 && !replayed.get() { format!(" heat_unconverged={unconverged}") } else { String::new() };
            say(&format!("{line}{replay}{heat} record={}", record.borrow()));
        } else {
            say(line);
        }
    };
    for threshold in thresholds {
        let bracket = opts.bracket.unwrap_or_else(|| threshold.default_bracket(&file));
        let t0 = Instant::now();
        let found = measure(threshold, bracket, &mut |t, mass| lives.live(t, mass), &mut |l| run_say(l))?;
        file.insert(threshold.item().to_string(), json!(found.value));
        say(&format!(
            "SR-CALIBRATE item={} value={:.3} bracket={:.3}..{:.3} runs={} wall_s={:.1}",
            threshold.item(),
            found.value,
            found.bracket.0,
            found.bracket.1,
            found.runs,
            t0.elapsed().as_secs_f64()
        ));
    }
    // Item 5, before any preset life is spent on a broken order.
    check_order(&file, constant(&physics, "sigma_bh")?)?;
    // Item 6: the three presets' lives, at the masses the four anchors translate 1, 15 and 40 Suns to.
    if asked(Job::Presets) {
        let anchors = ["m_ign_sb", "m_ch_sb", "m_up_sb", "m_bh_sb"].map(|key| file.get(key).and_then(Value::as_f64));
        let [Some(ign), Some(ch), Some(up), Some(bh)] = anchors else {
            return Err(CalError::Failed("preset_mass_sb needs m_ign_sb, m_ch_sb, m_up_sb and m_bh_sb in the file: measure them first".into()));
        };
        let t0 = Instant::now();
        let presets = measure_presets([ign, ch, up, bh], &physics, &mut |name, mass| lives.preset(name, mass), &mut |l| run_say(l))?;
        presets.insert_into(&mut file);
        say(&format!(
            "SR-CALIBRATE item=presets preset_mass_sb={:.1}/{:.1}/{:.1} t_eff_sun_sb={:.5} life_sun_sb={:.3} top_rung={} stages={} wall_s={:.1}",
            presets.masses[0],
            presets.masses[1],
            presets.masses[2],
            presets.t_eff_sun_sb,
            presets.life_sun_sb,
            presets.top_rung,
            presets.stage_durations_sb.len(),
            t0.elapsed().as_secs_f64()
        ));
    }
    // The fastest gas of every run made, kept against the file's (the same constants measured it): c_sb must beat twice it.
    if lives.runs > 0 {
        let peak = file.get("max_gas_speed").and_then(Value::as_f64).unwrap_or(0.0).max(lives.gas_peak);
        file.insert("max_gas_speed".into(), json!(peak));
    }
    if let Some(peak) = file.get("max_gas_speed").and_then(Value::as_f64) {
        say(&format!("SR-CALIBRATE item=max_gas_speed value={peak:.4}"));
        check_speed(peak, constant(&physics, "c_sb")?)?;
    }
    if !overrides.is_empty() {
        say("SR-WARN overrides given: measured and checked, nothing written (§2.12.1)");
        return Ok(());
    }
    // The adapter a GPU item opened; a base file's, once one has measured, stays; items 1–2 add none.
    let adapter = lives
        .adapter_used
        .or_else(|| file.get("measured").and_then(|m| m.get("adapter")).and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| "cpu".to_string());
    file.insert("version".into(), json!(calibration::VERSION));
    file.insert("physics_hash".into(), json!(hash));
    file.insert(
        "measured".into(),
        json!({
            "date": utc_iso(SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())),
            "git": git_rev(),
            "adapter": adapter,
            "box": hostname(),
        }),
    );
    if let Some(parent) = opts.out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| CalError::Args(format!("--out {}: {e}", opts.out.display())))?;
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(file)).expect("a Value always serialises");
    text.push('\n');
    std::fs::write(&opts.out, text).map_err(|e| CalError::Args(format!("--out {}: {e}", opts.out.display())))?;
    say(&format!("SR-CALIBRATE DONE out={} wall_s={:.1}", opts.out.display(), started.elapsed().as_secs_f64()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_iso_reads_known_instants() {
        assert_eq!(utc_iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_iso(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc_iso(1_760_877_525), "2025-10-19T12:38:45Z");
    }
}
