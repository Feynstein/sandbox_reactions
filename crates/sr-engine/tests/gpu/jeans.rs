//! M0-T62 — no artificial fragmentation (contract §2.3.5, §1.9.2, §5.1's G-JEANS, SM-O6, SM-E16): a disk cloud (a = 40, Σc = 1,
//! the Sun's mix, at rest, a uniform T) with a deterministic ripple, collapsing on the GPU's P2 + P4 until its central Σ is
//! 100 × the start's. scenes/jeans_ripple.json: a 128 × 128 leave world, gravity and hydro on, the rest off, and the cold
//! pressure off through `overrides` — the four K's and nothing else (G-COLL's isolation; the shipped K's halt this cloud at
//! Σ_max 12.7, measured). **U_th = 0.07|W|, not §5.1's 0.1|W|** — contract §0.5 [M0-T62], [M0-D66]: a γ = 2 gas heats as Σ
//! while |W| grows as Σ^½, so a cloud at 0.1|W| turns round at Σ_max 62.7 (cold pressure off; 60.7 with P5 on; 61.1 on
//! M0-D64's source) and never meets 100 ×; at 0.02|W| it fragments (J ≈ 4). M0-T62 ruled 0.05|W| (the lead, 2026-10-16), J ≤
//! 0.129 on the Σu·g source; on §0.5 [M0-D64]'s face work the diffusive mass flux's work comes out of thermal energy and the
//! collapse loses entropy, J 1.04 at 0.05|W|, so M0-D66 raised it to 0.07|W| (the lead, 2026-10-20): ≥ 100 × in ~190 steps.
//! Graded at each block summary the run takes (every 8 steps, every 2 once Σ_max ≥ 30) and at the end:
//!
//! - **J ≤ ¼ everywhere** — the Jeans number of §2.3.5, Δx/λ_J with λ_J = c²/(GΣ), c² = 2Π/Σ (§2.3.3), Δx = 1: J = GΣ²/(2Π),
//!   per gas cell (Σ ≥ σ_vac) from the state read back, Π from the EOS twin (`sr_physics::eos`).
//! - **one object** — `observe::Tracker` on the engine's own block summary: exactly one object (§1.9.2) at the end.
//! - **no secondary peak** — a *peak* is a cell whose Σ is at least that of every cell within 2 cells of it (a 5 × 5
//!   window, a feature as wide as a block) and, among equals, the first in row order; a *secondary* one is any but the
//!   global maximum. None above 10 % of the central Σ at the end.
//! - **it gets there** (the adversarial line) — a collapse that pressure stops before Σ reaches 100 × never
//!   meets the condition: the case prints the largest Σ reached, the step and the time, and fails, however long the run.
//!
//! The ripple, declared (the contract names the shape, `cos 3θ + cos 5θ`, and its 1 %; the scene format has no field for
//! it, so the test applies it to the scene's planes): Σ → Σ · (1 + 0.01 (cos 3θ + cos 5θ)), θ about the cloud's centre
//! (64, 64) at each cell's centre; E is rebuilt so each cell keeps its ε_th at the new ε_cold; at rest, species unchanged.
//! Its tolerances are the contract's (§5.1: J ¼, 10 %, 100 ×, one object); only the cloud's U_th/|W| is the ruling's. The checks run on the
//! physics adapter (R12): the pass of this scope is `SR-ADAPTER … RTX 5090` on linux-pc.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use sr_engine::scene::{EdgeMode, Scene, SUN_MIX};
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig, SPECIES};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::HeatSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_engine::watch::report;
use sr_physics::eos::Eos;
use sr_physics::observe::tracker::SUN_MASS_NOMINAL;
use sr_physics::observe::{Tracker, TrackerConfig};
use sr_physics::reference::grav::Gravity;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// The world's side and the cloud's centre.
const N: usize = 128;
const CENTRE: f64 = 64.0;
/// G-JEANS's numbers (§5.1): J's bound, the secondary peak's share of the central Σ, and the collapse's factor.
const J_MAX: f64 = 0.25;
const PEAK_SHARE: f64 = 0.10;
const FACTOR: f64 = 100.0;
/// U_th/|W| of the cloud (the lead's rulings at M0-T62 and M0-D66; §5.1 reads 0.1, which never reaches 100 ×).
const U_OVER_W: f64 = 0.07;
/// The ripple's amplitude (1 %) on each of its two modes.
const RIPPLE: f64 = 0.01;
/// The run's cap in steps — far past the collapse's few hundred: a run that is stopped by the cap reports its largest Σ.
const MAX_STEPS: u64 = 20_000;
/// A peak's window: cells within this Chebyshev distance.
const WINDOW: i64 = 2;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scene() -> Scene {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes");
    Scene::load(&dir.join("jeans_ripple.json")).unwrap_or_else(|e| panic!("{e}"))
}

fn world() -> WorldConfig {
    WorldConfig::new(N as u32, N as u32).unwrap()
}

/// The scene's planes (the cloud, no ripple), f32.
fn cloud_planes() -> Vec<f32> {
    let scene = scene();
    let (planes, warnings) = scene.planes(scene.world, &scene.physics().unwrap(), &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    planes
}

/// 1 + 0.01 (cos 3θ + cos 5θ) at cell (x, y)'s centre, θ about (64, 64).
fn ripple(x: usize, y: usize) -> f64 {
    let theta = (y as f64 + 0.5 - CENTRE).atan2(x as f64 + 0.5 - CENTRE);
    1.0 + RIPPLE * ((3.0 * theta).cos() + (5.0 * theta).cos())
}

/// The cloud's planes with the ripple on Σ: E rebuilt at each cell's own ε_th, momentum 0, species as they were.
fn rippled_planes() -> Vec<f32> {
    let physics = scene().physics().unwrap();
    let eos = Eos::from_physics(&physics).unwrap();
    let comp = elements().composition(&SUN_MIX);
    let mut p = cloud_planes();
    let cells = N * N;
    for i in 0..cells {
        let sigma = f64::from(p[i]);
        if sigma <= 0.0 {
            continue;
        }
        let eps_th = f64::from(p[3 * cells + i]) / sigma - eos.eps_cold(sigma, comp.y_e, 0.0);
        let new = sigma * ripple(i % N, i / N);
        p[i] = new as f32;
        p[3 * cells + i] = (new * (eps_th + eos.eps_cold(new, comp.y_e, 0.0))) as f32;
    }
    p
}

/// J = Δx/λ_J = GΣ²/(2Π) (Δx = 1, G = 1): λ_J = c²/(GΣ) with c² = 2Π/Σ.
fn jeans_number(sigma: f64, pi: f64) -> f64 {
    sigma * sigma / (2.0 * pi)
}

/// The largest J over the gas cells of `planes` (Σ ≥ σ_vac) and where it is: Π from the EOS twin, ε_th floored as P8 does.
fn j_max(planes: &[f64], eos: &Eos, elements: &Elements, sigma_vac: f64) -> (f64, usize) {
    let cells = N * N;
    let mut worst = (0.0, 0);
    for i in 0..cells {
        let sigma = planes[i];
        if sigma < sigma_vac {
            continue;
        }
        let x: [f64; N_SPECIES] = std::array::from_fn(|s| planes[(SPECIES.start + s) * cells + i]);
        let comp = elements.composition(&x);
        let th = eos.thermal(planes[3 * cells + i], sigma, [planes[cells + i], planes[2 * cells + i]], comp.y_e, x[N_SPECIES - 1], 1.0 / comp.inv_mu);
        let j = jeans_number(sigma, eos.pi(sigma, th.eps_th, comp.y_e, x[N_SPECIES - 1]));
        if j > worst.0 {
            worst = (j, i);
        }
    }
    worst
}

/// The secondary peaks of a Σ plane (see the module's definition): their Σ, largest first, and the global maximum's.
fn secondary_peaks(sigma: &[f64]) -> (f64, Vec<f64>) {
    let at = |x: i64, y: i64| sigma[y as usize * N + x as usize];
    let top = sigma.iter().enumerate().fold((0, 0.0), |m, (i, &s)| if s > m.1 { (i, s) } else { m });
    let mut peaks = Vec::new();
    for y in 0..N as i64 {
        for x in 0..N as i64 {
            let s = at(x, y);
            if (y as usize * N + x as usize) == top.0 || s <= 0.0 {
                continue;
            }
            let mut peak = true;
            'window: for dy in -WINDOW..=WINDOW {
                for dx in -WINDOW..=WINDOW {
                    let (nx, ny) = (x + dx, y + dy);
                    if (dx, dy) == (0, 0) || nx < 0 || ny < 0 || nx >= N as i64 || ny >= N as i64 {
                        continue;
                    }
                    let n = at(nx, ny);
                    // An equal neighbour earlier in row order wins the tie.
                    if n > s || (n == s && (ny, nx) < (y, x)) {
                        peak = false;
                        break 'window;
                    }
                }
            }
            if peak {
                peaks.push(s);
            }
        }
    }
    peaks.sort_by(|a, b| b.total_cmp(a));
    (top.1, peaks)
}

/// One block summary of the run.
#[derive(Clone, Debug)]
struct Row {
    step: u64,
    sim_time: f64,
    sigma_max: f64,
    j: f64,
    j_cell: (usize, usize),
    objects: usize,
    object_mass: f64,
    /// The largest secondary peak's Σ over the central Σ (0 with none).
    peak_share: f64,
}

struct Run {
    rows: Vec<Row>,
    /// The central Σ the collapse must grow 100 × from: the start's, before the ripple.
    sigma_start: f64,
    adapter: String,
}

impl Run {
    fn last(&self) -> &Row {
        self.rows.last().expect("the run took summaries")
    }

    fn reached(&self) -> bool {
        self.last().sigma_max >= FACTOR * self.sigma_start
    }
}

/// The run, once for every case that reads it.
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let (physics, elements) = (scene().physics().unwrap(), elements());
        let eos = Eos::from_physics(&physics).unwrap();
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let sigma_start = cloud_planes()[..N * N].iter().fold(0.0f32, |m, &s| m.max(s)) as f64;
        let gpu = super::device();
        let state = GpuState::new(&gpu.device, world()).unwrap();
        state.upload(&gpu.queue, &rippled_planes()).unwrap();
        let booking = Booking::new(&gpu.device, world()).unwrap();
        let side = SideFields::new(&gpu.device, world()).unwrap();
        let gpu_eos = EosGpu::new(&gpu.device, &physics, &elements);
        let passes = Passes { mode: BoxMode::Fit, gravity: Some(&side), hydro: Some(EdgeMode::Leave) };
        // The scene's switches as `headless` reads them: P5 when heat is on (this scope's scene has it off).
        let scene = scene();
        let heat = scene.switches.heat.then_some(HeatSetup { side: &side, test: scene.test });
        let step = Step::with_neutrino_heating(&gpu.device, &state, &booking, &gpu_eos, &physics, &passes, heat.as_ref(), None, None, None).unwrap();
        let mut tracker = Tracker::new(TrackerConfig::from_physics(&physics, SUN_MASS_NOMINAL));
        let mut take = |steps: u64, sim_time: f64| -> Row {
            let planes: Vec<f64> = state.readback(&gpu.device, &gpu.queue).iter().map(|&v| f64::from(v)).collect();
            let summary = step.summarise(&gpu.device, &gpu.queue);
            let objects = tracker.update(&report(&summary, N as u32, 1.0)).to_vec();
            let (j, cell) = j_max(&planes, &eos, &elements, sigma_vac);
            let (sigma_max, peaks) = secondary_peaks(&planes[..N * N]);
            Row {
                step: steps,
                sim_time,
                sigma_max,
                j,
                j_cell: (cell % N, cell / N),
                objects: objects.len(),
                object_mass: objects.iter().map(|o| o.mass).sum(),
                peak_share: peaks.first().map_or(0.0, |p| p / sigma_max),
            }
        };
        let mut rows = vec![take(0, 0.0)];
        let mut report_dt = step.read_dt(&gpu.device, &gpu.queue);
        while report_dt.steps < MAX_STEPS {
            let sigma_max = rows.last().unwrap().sigma_max;
            if sigma_max >= FACTOR * sigma_start {
                break;
            }
            // Before step 0 the Δt record holds no Δt yet: that step runs alone.
            let batch = if report_dt.steps == 0 { 1 } else if sigma_max < 30.0 * sigma_start { 8 } else { 2 };
            step.run(&gpu.device, &gpu.queue, batch);
            report_dt = step.read_dt(&gpu.device, &gpu.queue);
            assert!(report_dt.non_finite.is_none(), "non-finite state: {:?}", report_dt.non_finite);
            rows.push(take(report_dt.steps, report_dt.sim_time));
        }
        Run { rows, sigma_start, adapter: gpu.info.sr_adapter_line() }
    })
}

fn print_series(run: &Run) {
    println!("{}", run.adapter);
    let n = run.rows.len();
    for (k, r) in run.rows.iter().enumerate() {
        // Every 10th row and the last: the whole series is in the test's own memory, not the log.
        if k % 10 == 0 || k + 1 == n {
            println!(
                "SR-JEANS step {:>5} t {:>8.4}: Σmax {:>8.3}, J {:.4} at ({}, {}), {} object(s) of mass {:.1}, secondary peak {:.4} of Σmax",
                r.step, r.sim_time, r.sigma_max, r.j, r.j_cell.0, r.j_cell.1, r.objects, r.object_mass, r.peak_share
            );
        }
    }
}

// ---- the scene, the ripple and the checks' own arithmetic ----

#[test]
fn the_scene_is_the_disk_cloud_at_seven_hundredths_of_w_with_a_deterministic_one_percent_ripple() {
    let scene = scene();
    // Only the four K's are overridden — G-COLL's isolation of gravity and gas from the cold pressure, which at Σ ~ 12 stops
    // any collapse (measured: the shipped constants halt this cloud at Σ_max 12.7, M0-T62); every other constant is shipped.
    let mut keys: Vec<&str> = scene.overrides.iter().map(|(k, _)| k.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["k1e", "k1n", "k2e", "k2n"], "the overrides");
    let (physics, shipped) = (scene.physics().unwrap(), Physics::shipped().unwrap());
    for key in Physics::keys() {
        let expected = if ["k1e", "k2e", "k1n", "k2n"].contains(&key) { Some(0.0) } else { shipped.get(key) };
        assert_eq!(physics.get(key), expected, "physics key {key}");
    }
    let s = scene.switches;
    assert!(s.gravity && s.hydro && !s.heat && !s.reactions && !s.neutrinos && !s.sinks, "{s:?}");
    assert!(!scene.slowdown && scene.edge == EdgeMode::Leave);
    let elements = elements();
    let eos = Eos::from_physics(&physics).unwrap();
    let comp = elements.composition(&SUN_MIX);
    let (cells, vac) = (N * N, physics.get("sigma_vac").unwrap());

    // The cloud: a = 40 (it spans 80 cells about (64, 64)), Σc = 1, U_th = 0.07|W| (the ruling; §5.1 says 0.1|W|) with W
    // the grid's own solve — the scene file's T has four decimals, so the ratio is graded to 1e-4 relative.
    let p = cloud_planes();
    let sigma: Vec<f64> = p[..cells].iter().map(|&s| f64::from(s)).collect();
    let (mut lo, mut hi) = (usize::MAX, 0);
    for (i, &s) in sigma.iter().enumerate() {
        if s > 0.0 {
            lo = lo.min(i % N);
            hi = hi.max(i % N);
        }
    }
    assert!((hi - lo + 1).abs_diff(80) <= 2, "the cloud spans {} cells, 2a = 80", hi - lo + 1);
    let top = sigma.iter().cloned().fold(0.0, f64::max);
    assert!((top - 1.0).abs() < 0.01, "central Σ {top}");
    let source: Vec<f64> = sigma.iter().map(|&s| if s >= vac { s } else { 0.0 }).collect();
    let phi = Gravity::new(1.0).potential(&source, N, N).unwrap();
    let w = 0.5 * source.iter().zip(&phi).map(|(m, p)| m * p).sum::<f64>();
    let u_th: f64 = (0..cells)
        .filter(|&i| sigma[i] > 0.0)
        .map(|i| f64::from(p[3 * cells + i]) - sigma[i] * eos.eps_cold(sigma[i], comp.y_e, 0.0))
        .sum();
    println!("SR-JEANS scene: central Σ {top:.4}, W {w:.2}, U_th {u_th:.2}, U_th/|W| {:.6}", u_th / w.abs());
    assert!((u_th / w.abs() / U_OVER_W - 1.0).abs() <= 1e-4, "U_th/|W| = {}, not {U_OVER_W}", u_th / w.abs());

    // The ripple: the same planes with Σ scaled by 1 + 0.01 (cos 3θ + cos 5θ) — within ±2 % each, bit for bit the same on
    // a second call, a zero mean over the cloud (no net mass moved beyond the sampling), the cloud's cells and nothing else.
    let (a, b) = (rippled_planes(), rippled_planes());
    assert!(a == b, "the ripple is deterministic");
    let (mut worst, mut mass_before, mut mass_after) = (0.0f64, 0.0, 0.0);
    for i in 0..cells {
        if sigma[i] <= 0.0 {
            assert_eq!(a[i], 0.0, "the ripple adds no gas at cell {i}");
            continue;
        }
        let factor = f64::from(a[i]) / sigma[i];
        worst = worst.max((factor - ripple(i % N, i / N)).abs());
        assert!((factor - 1.0).abs() <= 2.0 * RIPPLE + 1e-6, "cell {i}: ×{factor}");
        assert_eq!((a[cells + i], a[2 * cells + i]), (0.0, 0.0), "cell {i} is at rest");
        mass_before += sigma[i];
        mass_after += f64::from(a[i]);
    }
    println!("SR-JEANS ripple: worst |Σ'/Σ − (1 + 0.01(cos 3θ + cos 5θ))| = {worst:.2e}, mass {mass_before:.3} → {mass_after:.3}");
    assert!(worst <= 1e-6, "Σ'/Σ departs from the ripple by {worst}");
    assert!((mass_after / mass_before - 1.0).abs() <= 1e-3, "the ripple moved mass: {mass_before} → {mass_after}");
    // It is the ripple, not a flat cloud: the cell at θ = 0 (cos 0 + cos 0 = 2) sits 2 % above, the one near θ = π/3 …
    // (cos π + cos 5π/3 = −0.5) below.
    let factor = |x: usize, y: usize| f64::from(a[y * N + x]) / sigma[y * N + x];
    assert!(factor(100, 64) > 1.015 && factor(100, 63) > 1.015, "θ ≈ 0: {} {}", factor(100, 64), factor(100, 63));
}

#[test]
fn the_jeans_number_the_peak_finder_and_the_object_count_see_what_they_must() {
    // J = GΣ²/(2Π): Σ = 1 at Π = 2 is exactly the bound; a hotter cell below it, a colder one above.
    assert_eq!(jeans_number(1.0, 2.0), J_MAX);
    assert!(jeans_number(1.0, 3.0) < J_MAX && jeans_number(1.0, 1.0) > J_MAX);
    assert_eq!(jeans_number(10.0, 200.0), J_MAX);

    // The peak finder on hand-made planes: one smooth hill has no secondary; a second clump is found at its share of the
    // central Σ, 0.09 under the bar and 0.11 over it; two equal clumps count one each way; a 3-cell-wide shoulder is a
    // peak only when it is higher than every cell within 2.
    let hill = |extra: &[(usize, usize, f64)]| {
        let mut s = vec![0.0; N * N];
        for y in 0..N {
            for x in 0..N {
                let r2 = ((x as f64 - 40.0).powi(2) + (y as f64 - 40.0).powi(2)) / 400.0;
                s[y * N + x] = 100.0 * (-r2).exp();
            }
        }
        for &(x, y, v) in extra {
            s[y * N + x] = v;
        }
        s
    };
    assert_eq!(secondary_peaks(&hill(&[])).1.len(), 0, "a smooth hill");
    let clump = |share: f64| {
        let mut s = vec![0.0; N * N];
        s[40 * N + 40] = 100.0;
        s[90 * N + 90] = 100.0 * share;
        secondary_peaks(&s)
    };
    assert_eq!(clump(0.11).1.len(), 1);
    assert!((clump(0.11).1[0] / clump(0.11).0 - 0.11).abs() < 1e-12);
    assert!(clump(0.09).1[0] / clump(0.09).0 < PEAK_SHARE, "a 9 % clump is below the bar, yet listed so the run can print it");
    assert_eq!(clump(1.0).1.len(), 1, "two equal clumps: one is the global maximum, the other secondary");
    let mut tied = vec![0.0; N * N];
    tied[40 * N + 40] = 5.0;
    tied[40 * N + 41] = 5.0;
    assert_eq!(secondary_peaks(&tied).1.len(), 0, "a 2-cell plateau is one peak, not two");

    // The object count on the engine's own tracker: two separate blocks of gas are two objects (§1.9.2) — a hand-made
    // report, so the count is the tracker's and the check that reads it fails when the cloud splits.
    use sr_physics::observe::{BlockData, Report};
    let block = |mass: f64| BlockData { mass, max_sigma: 1.0, thermal: 1.0, ..BlockData::default() };
    let mut blocks = vec![BlockData::default(); 16];
    blocks[0] = block(3000.0);
    blocks[2] = block(3000.0);
    let two = Report { sim_dt: 1.0, world_width: 64, origin: [0, 0], columns: 4, blocks, sinks: Vec::new() };
    let mut tracker = Tracker::new(TrackerConfig::from_physics(&Physics::shipped().unwrap(), SUN_MASS_NOMINAL));
    assert_eq!(tracker.update(&two).len(), 2, "two blocks with an empty one between: two objects");
    let mut joined = two.clone();
    joined.blocks[1] = block(10.0);
    let mut tracker = Tracker::new(TrackerConfig::from_physics(&Physics::shipped().unwrap(), SUN_MASS_NOMINAL));
    assert_eq!(tracker.update(&joined).len(), 1, "joined by a block of gas: one");
    println!("SR-JEANS checks: J(1, 2) = {J_MAX}, peaks and objects counted on synthetic planes (NOT PROVEN on the star)");
}

// ---- the collapse on the GPU ----

#[test]
fn the_collapse_reaches_a_hundred_times_the_central_sigma() {
    let run = run();
    print_series(run);
    let last = run.last();
    let largest = run.rows.iter().map(|r| r.sigma_max).fold(0.0, f64::max);
    println!(
        "SR-JEANS reach: Σ_max {:.3} at the end (largest {largest:.3}) after {} steps, t = {:.4} (start {:.4}, target {:.1}); {} summaries",
        last.sigma_max,
        last.step,
        last.sim_time,
        run.sigma_start,
        FACTOR * run.sigma_start,
        run.rows.len()
    );
    assert!(
        run.reached(),
        "the collapse never reached {FACTOR} × the central Σ ({:.1}): the largest Σ was {largest:.3} after {} steps (t = {:.4}) — \
         stopped by pressure, or cut by the {MAX_STEPS}-step cap",
        FACTOR * run.sigma_start,
        last.step,
        last.sim_time
    );
}

#[test]
fn the_jeans_number_stays_within_a_quarter_everywhere_in_every_summary() {
    let run = run();
    assert!(run.reached(), "the collapse never reached {FACTOR} ×: its largest Σ was {:.3}", run.rows.iter().map(|r| r.sigma_max).fold(0.0, f64::max));
    let worst = run.rows.iter().fold(&run.rows[0], |w, r| if r.j > w.j { r } else { w });
    println!(
        "SR-JEANS J: worst {:.4} (bound {J_MAX}) at step {}, t = {:.4}, cell ({}, {}), Σ_max then {:.3}; over {} summaries",
        worst.j, worst.step, worst.sim_time, worst.j_cell.0, worst.j_cell.1, worst.sigma_max, run.rows.len()
    );
    assert!(
        worst.j <= J_MAX,
        "J = {:.4} > {J_MAX} at step {} (t = {:.4}) in cell ({}, {}), Σ_max {:.3}",
        worst.j,
        worst.step,
        worst.sim_time,
        worst.j_cell.0,
        worst.j_cell.1,
        worst.sigma_max
    );
}

#[test]
fn at_a_hundred_times_the_sigma_there_is_one_object_and_no_secondary_peak() {
    let run = run();
    let last = run.last();
    assert!(run.reached(), "the collapse never reached {FACTOR} ×: its largest Σ was {:.3}", run.rows.iter().map(|r| r.sigma_max).fold(0.0, f64::max));
    let most = run.rows.iter().map(|r| r.objects).max().unwrap();
    let peak = run.rows.iter().map(|r| r.peak_share).fold(0.0, f64::max);
    println!(
        "SR-JEANS end: step {}, t = {:.4}, Σ_max {:.3}: {} object(s) of mass {:.1} (at most {most} in any summary); \
         largest secondary peak {:.4} of Σ_max at the end, {peak:.4} in any summary",
        last.step, last.sim_time, last.sigma_max, last.objects, last.object_mass, last.peak_share
    );
    assert_eq!(last.objects, 1, "{} objects at Σ_max = {:.3} (step {})", last.objects, last.sigma_max, last.step);
    assert!(
        last.peak_share <= PEAK_SHARE,
        "a secondary peak at {:.4} of the central Σ (> {PEAK_SHARE}) at step {}",
        last.peak_share,
        last.step
    );
}
