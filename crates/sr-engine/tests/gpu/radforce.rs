//! M0-T43 — the radiation force in the flow (contract §2.4.4, §1.3.2's P4, §2.3.4, R8, R13): f = κΣF/c_sb (SM-E22), from the
//! previous step's flux F, is P4's momentum source; E is unchanged by it (the work comes out of the thermal energy). The
//! twin (`sr_physics::reference::hydro`'s `step_forced`) and the GPU's P4 (shaders/hydro.wgsl's update, F bound from
//! `side.flux`) on the physics adapter (R12). Scope `radiation_force` (R17; the contract's `radiation-force`).
//!
//! The scene: a hot slab in a uniform medium — 64 × 64, bounce, hydrogen, at rest, the cold pressure off; the medium Σ = 1 at
//! T = 1, the slab (columns 24 … 39, every row) Σ = 0.1 at T = 10, so Π = 2 on both sides and nothing moves but what the
//! force moves and HLL's smearing of the contact. P5 runs with `pin_temperature` (§2.12.1): it computes F every step and
//! leaves E alone, so the only E a step changes is P4's. The light pushes the medium away from the slab on both sides and
//! the slab's and the medium's edge cells toward the world's edges.
//!
//! The oracle, written here from the contract and never read from the scheme: f = κΣF/c_sb with κ = κ₀(1 + X_H) (κ_dust = 0
//! in physics.json, asserted), Σ and X_H from the state the step starts from and F as it stands at that moment — the
//! previous step's P5's. "The momentum gained" is the step's momentum against the same step without the force, both from
//! the same state at the same Δt: the gas flow is the same in both, so what the force adds is Δt·f per cell (in exact
//! arithmetic, to the bit in the fluxes). Over `STEPS` steps, each step from the forced run's own state, the momentum
//! gained in each half of the world (x left/right for the x momentum, y top/bottom for the y momentum — the force has a
//! sign in each) equals Σ Δt·f there within 1e-6 relative (the plan's tolerance, fixed before the first run, for the twin
//! and for the GPU's f32); Σ, E and every species are the force-free step's bit for bit. The adversarial case: the force
//! taken from this step's F (P5's, after P4) instead of the previous one would change the scheme's order silently, so one
//! step runs on an F written by hand — the previous step's negated — and the momentum gained must be that F's force, not
//! the force of the F P5 leaves at the step's end.
//!
//! M0-D58 — the work is capped at the heat above the floor (§0.5 [M0-D58]). The oracle, from the contract's words: the cell
//! gains s·Δt·f with s the largest share ≤ 1 whose work (|p₀ + s d|² − |p₀|²)/2Σ is at most Σ(ε_th − T_floor/μ), p₀ and ε_th
//! the force-free step's (0 when that is not positive), so P8 adds no energy for the push. On the slab made hot enough
//! (T 1,000 in a medium at T 0.06: the impulse's work ~10⁸ against a heat of 0.02 a cell) the cap binds at the slab's edges
//! and the momentum gained is s·Δt·f, E the free step's. Then D58's block itself — a 2 × 2 C/O block of Σ 1,250 at T_floor,
//! heat on, 64 × 64 bounce, gravity and hydro, the shipped constants — 50 steps on the twin and through `headless` on the
//! GPU: every value finite and the mass invariant within 10⁻⁶ (uncapped, E runs to NaN by step 12 and the world empties).

use std::path::PathBuf;

use serde_json::Value;
use sr_engine::dump::Dump;
use sr_engine::gpu::Gpu;
use sr_engine::scene::{EdgeMode, Scene, TestSettings};
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::HeatSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::heat::{Heat, TestFlags};
use sr_physics::reference::hydro::{Edge, Hydro, State};
use sr_physics::reference::{react, Burning, Switches, Twin};
use sr_physics::registry::{Elements, Physics, Reactions};

const W: usize = 64;
const H: usize = 64;
/// The slab's columns.
const SLAB: std::ops::Range<usize> = 24..40;
/// Σ and T of the medium and of the slab: Π = Σ ε_th = Σ T / μ is 2 on both sides (hydrogen, 1/μ = 2).
const MEDIUM: (f64, f64) = (1.0, 1.0);
const HOT: (f64, f64) = (0.1, 10.0);
/// Hydrogen's 1/μ (§1.5).
const INV_MU: f64 = 2.0;
/// Every step's Δt (CFL 0.4 allows 0.032 in the slab, where c² = 2Π/Σ = 40).
const DT: f64 = 0.02;
const STEPS: usize = 8;
/// The plan's tolerance on the momentum gained, relative, per half and component.
const TOL: f64 = 1e-6;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

/// The shipped constants with the cold pressure off (Π = Σ ε_th).
fn physics() -> Physics {
    let physics = Physics::shipped().unwrap().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).unwrap();
    assert_eq!(physics.get("kappa_dust"), Some(0.0), "the oracle's κ has no dust term (§2.4.2: 0 until S1)");
    physics
}

fn pinned() -> TestFlags {
    TestFlags { pin_temperature: true, constant_diffusivity: None, flux_limiter: true }
}

/// The hot slab in its medium, every plane rounded to f32 (the GPU's start, so the twin starts where it does).
fn slab() -> State {
    slab_of(MEDIUM, HOT)
}

/// A slab of (Σ, T) `hot` in a medium of (Σ, T) `medium`, as [`slab`].
fn slab_of(medium: (f64, f64), hot: (f64, f64)) -> State {
    let n = W * H;
    let mut planes = vec![0.0; 14 * n];
    for i in 0..n {
        let (sigma, t) = if SLAB.contains(&(i % W)) { hot } else { medium };
        planes[i] = sigma;
        planes[3 * n + i] = sigma * INV_MU * t;
        planes[4 * n + i] = 1.0;
    }
    State::new(W, H, planes.iter().map(|&v| f64::from(v as f32)).collect())
}

/// f = κΣF/c_sb (§2.4.4) on every cell of `st` under the cell-centred flux (`fx`, `fy`): κ = κ₀(1 + X_H), 0 below Σ_vac.
fn force(physics: &Physics, st: &State, fx: &[f64], fy: &[f64]) -> [Vec<f64>; 2] {
    let get = |k: &str| physics.get(k).unwrap();
    let (kappa0, c_sb, sigma_vac) = (get("kappa0"), get("c_sb"), get("sigma_vac"));
    let n = st.cells();
    let f = |c: &[f64]| -> Vec<f64> {
        (0..n)
            .map(|i| {
                let sigma = st.planes[i];
                if sigma < sigma_vac {
                    return 0.0;
                }
                kappa0 * (1.0 + st.planes[4 * n + i]) * sigma * c[i] / c_sb
            })
            .collect()
    };
    [f(fx), f(fy)]
}

/// The momentum one step gained against the same step without the force, by half of the world, and Σ Δt·f there.
#[derive(Default)]
struct Tally {
    /// [component][half]: x by left/right, y by top/bottom.
    gained: [[f64; 2]; 2],
    expected: [[f64; 2]; 2],
    /// The largest |gained − Δt·f| of a cell, and the largest |Δt·f|.
    cell_gap: f64,
    cell_scale: f64,
    /// The largest difference in any other channel (Σ, E, the species).
    other: f64,
}

impl Tally {
    /// One step: `forced` and `free` ran from the same state at `dt`, the first under the force `f`.
    fn add(&mut self, forced: &State, free: &State, f: &[Vec<f64>; 2], dt: f64) {
        let n = forced.cells();
        for i in 0..n {
            let half = [usize::from(i % W >= W / 2), usize::from(i / W >= H / 2)];
            for c in 0..2 {
                let gained = forced.planes[(1 + c) * n + i] - free.planes[(1 + c) * n + i];
                let expected = dt * f[c][i];
                self.gained[c][half[c]] += gained;
                self.expected[c][half[c]] += expected;
                self.cell_gap = self.cell_gap.max((gained - expected).abs());
                self.cell_scale = self.cell_scale.max(expected.abs());
            }
            for ch in (0..14).filter(|&ch| ch != 1 && ch != 2) {
                self.other = self.other.max((forced.planes[ch * n + i] - free.planes[ch * n + i]).abs());
            }
        }
    }

    fn check(&self, label: &str) {
        let names = [["x, left", "x, right"], ["y, top", "y, bottom"]];
        for c in 0..2 {
            for h in 0..2 {
                let (g, e) = (self.gained[c][h], self.expected[c][h]);
                let rel = (g - e).abs() / e.abs();
                println!("SR-RADFORCE {label} {}: gained {g:.9e}, Σ Δt·f {e:.9e}, relative {rel:.2e}", names[c][h]);
                // The force is real and has its sign: away from the slab in x, toward the edges in y.
                let sign = if h == 0 { -1.0 } else { 1.0 };
                assert!(sign * e > 1e-6, "{label} {}: the force must push (Σ Δt·f = {e:e})", names[c][h]);
                assert!(rel <= TOL, "{label} {}: gained {g:e} against Σ Δt·f {e:e} ({rel:e} > {TOL:e})", names[c][h]);
            }
        }
        println!(
            "SR-RADFORCE {label} cells: worst |gained − Δt·f| {:.2e} of the largest Δt·f {:.2e}; Σ, E, species {:e} from the free step",
            self.cell_gap, self.cell_scale, self.other
        );
        assert_eq!(self.other, 0.0, "{label}: the force changes the momentum only — Σ, E and the species bit for bit");
    }
}

/// The twin: P4 with the previous step's F, against P4 without it from the same state; P5 pinned between steps.
#[test]
fn twin_momentum_gained_is_the_force_impulse_and_e_is_unchanged() {
    let physics = physics();
    let hydro = Hydro::new(&physics, &elements(), Edge::Bounce).unwrap();
    let heat = Heat::new(&physics, &elements(), pinned()).unwrap();
    let mut st = slab();
    let mut flux = heat.step(&mut st.clone(), DT).flux.cells();
    let mut tally = Tally::default();
    for _ in 0..STEPS {
        let f = force(&physics, &st, &flux.0, &flux.1);
        let mut free = st.clone();
        hydro.step(&mut free, DT);
        hydro.step_forced(&mut st, DT, None, Some((&flux.0, &flux.1)));
        tally.add(&st, &free, &f, DT);
        let before = st.clone();
        flux = heat.step(&mut st, DT).flux.cells();
        assert_eq!(st, before, "pinned P5 leaves the state alone");
    }
    tally.check("twin");
}

/// One step on the GPU: the state, its booking and the step built on them.
struct Run {
    state: GpuState,
    booking: Booking,
    side: SideFields,
    step: Step,
}

/// The forced run (P4 with P5 pinned: the force on) and the free run (P4 alone) on one device, P1, P8 and P9 around them.
struct Pair {
    gpu: Gpu,
    forced: Run,
    free: Run,
}

impl Pair {
    fn new(physics: &Physics, first: &State) -> Pair {
        let gpu = super::device();
        let world = WorldConfig::new(W as u32, H as u32).unwrap();
        let eos = EosGpu::new(&gpu.device, physics, &elements());
        let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: Some(EdgeMode::Bounce) };
        let test = TestSettings { pin_temperature: true, constant_diffusivity: None, flux_limiter: true };
        let run = |heat: bool| {
            let state = GpuState::new(&gpu.device, world).unwrap();
            let booking = Booking::new(&gpu.device, world).unwrap();
            let side = SideFields::new(&gpu.device, world).unwrap();
            let setup = HeatSetup { side: &side, test };
            let step = Step::with_heat(&gpu.device, &state, &booking, &eos, physics, &passes, heat.then_some(&setup)).unwrap();
            Run { state, booking, side, step }
        };
        let pair = Pair { forced: run(true), free: run(false), gpu };
        // Step 0 runs on the initial reduction's Δt: run it, then start again from `first` (F stays as P5 left it).
        for r in [&pair.forced, &pair.free] {
            pair.upload(r, first);
            r.step.run(&pair.gpu.device, &pair.gpu.queue, 1);
        }
        pair.upload(&pair.forced, first);
        pair
    }

    fn upload(&self, run: &Run, st: &State) {
        let planes: Vec<f32> = st.planes.iter().map(|&v| v as f32).collect();
        run.state.upload(&self.gpu.queue, &planes).unwrap();
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        run.booking.clear(&mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
    }

    fn planes(&self, run: &Run) -> State {
        State::new(W, H, run.state.readback(&self.gpu.device, &self.gpu.queue).iter().map(|&v| f64::from(v)).collect())
    }

    /// The forced run's side.flux — the F its next P4 reads — as two f64 planes.
    fn flux(&self) -> (Vec<f64>, Vec<f64>) {
        let all: Vec<f64> = read(&self.gpu, self.forced.side.flux()).iter().map(|&v| f64::from(v)).collect();
        let n = W * H;
        (all[..n].to_vec(), all[n..2 * n].to_vec())
    }

    fn write_flux(&self, fx: &[f64], fy: &[f64]) {
        let bytes: Vec<u8> = fx.iter().chain(fy).flat_map(|&v| (v as f32).to_le_bytes()).collect();
        self.gpu.queue.write_buffer(self.forced.side.flux(), 0, &bytes);
    }

    /// One step of each run from the forced run's state at `dt` (written as the Δt record's Δt_{n+1}, reduce.wgsl
    /// `DtState`'s second word, which P1 takes); the state both started from, and both results.
    fn step(&self, dt: f64) -> (State, State, State) {
        let start = self.planes(&self.forced);
        self.upload(&self.free, &start);
        for r in [&self.forced, &self.free] {
            self.gpu.queue.write_buffer(r.step.dt_record(), 4, &(dt as f32).to_le_bytes());
            r.step.run(&self.gpu.device, &self.gpu.queue, 1);
            assert_eq!(r.step.read_dt(&self.gpu.device, &self.gpu.queue).dt, dt as f32, "P1 took the Δt written");
        }
        (start, self.planes(&self.forced), self.planes(&self.free))
    }
}

fn read(gpu: &Gpu, buffer: &wgpu::Buffer) -> Vec<f32> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("radforce.test.read"),
        size: buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, buffer.size());
    gpu.queue.submit([encoder.finish()]);
    staging.map_async(wgpu::MapMode::Read, .., |r| r.expect("mapped"));
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let view = staging.get_mapped_range(..).unwrap();
    view.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect()
}

/// The GPU's P4: each step's momentum gained against the free step is Σ Δt·f of the F side.flux held at the step's start.
#[test]
fn gpu_momentum_gained_is_the_force_impulse_and_e_is_unchanged() {
    let physics = physics();
    let first = slab();
    let pair = Pair::new(&physics, &first);
    let dt = f64::from(DT as f32);
    let mut tally = Tally::default();
    for _ in 0..STEPS {
        let (fx, fy) = pair.flux();
        let start = pair.planes(&pair.forced);
        let (from, forced, free) = pair.step(dt);
        assert_eq!(from, start);
        tally.add(&forced, &free, &force(&physics, &start, &fx, &fy), dt);
    }
    tally.check("gpu");
}

/// The adversarial case: P4 reads the F the last step left, not the one this step's P5 writes. One step runs on an F
/// written by hand — the previous step's negated —: the momentum gained is that F's force within 1e-6, and differs from
/// the force of the F P5 leaves at the step's end by more than the force itself.
#[test]
fn gpu_p4_reads_the_previous_steps_flux() {
    let physics = physics();
    let pair = Pair::new(&physics, &slab());
    let dt = f64::from(DT as f32);
    for _ in 0..2 {
        pair.step(dt);
    }
    let (fx, fy) = pair.flux();
    let (hx, hy): (Vec<f64>, Vec<f64>) = (fx.iter().map(|v| -v).collect(), fy.iter().map(|v| -v).collect());
    pair.write_flux(&hx, &hy);
    let (start, forced, free) = pair.step(dt);
    let (ax, ay) = pair.flux();
    let mut by_hand = Tally::default();
    by_hand.add(&forced, &free, &force(&physics, &start, &hx, &hy), dt);
    let mut after = Tally::default();
    after.add(&forced, &free, &force(&physics, &start, &ax, &ay), dt);
    for c in 0..2 {
        for h in 0..2 {
            let (g, e, e_after) = (by_hand.gained[c][h], by_hand.expected[c][h], after.expected[c][h]);
            let rel = (g - e).abs() / e.abs();
            let rel_after = (g - e_after).abs() / e_after.abs();
            println!("SR-RADFORCE by-hand [{c}][{h}]: gained {g:.9e}, the hand F's {e:.9e} ({rel:.2e}), P5's F after {e_after:.9e} ({rel_after:.2e})");
            assert!(e.abs() > 1e-6 && rel <= TOL, "P4 must push with the F side.flux held at the step's start ({rel:e})");
            assert!(rel_after > 1.0, "the F P5 wrote this step must not be the one P4 read ({rel_after:e})");
        }
    }
    assert_eq!(by_hand.other, 0.0, "Σ, E and the species as the free step's");
}

/// The cap's slab: the medium just above the floor (Σ 1, T 0.06: 0.02 of heat above it a cell), the slab at T 1,000 (Σ 0.1).
const CAP_MEDIUM: (f64, f64) = (1.0, 0.06);
const CAP_HOT: (f64, f64) = (0.1, 1000.0);
/// One step's Δt there (CFL 0.4 allows 6.3e-3 in the slab, c² = 2Π/Σ = 4,000).
const CAP_DT: f64 = 2e-3;
/// The cap's tolerances, fixed before the first run: the momentum gained per half of the world against Σ s·Δt·f, and E
/// per cell against the free step's (P8's rounding where the push spends the heat to the floor) — twin, GPU.
const CAP_TOL: (f64, f64) = (1e-9, 1e-5);
const CAP_E_TOL: (f64, f64) = (1e-12, 1e-5);

/// The oracle's share s of the impulse `d` in a cell whose force-free state is (Σ, p₀, E): the largest s ≤ 1 with
/// (|p₀ + s d|² − |p₀|²)/2Σ ≤ A = Σ(E/Σ − |p₀|²/2Σ² − T_floor/μ) — hydrogen, the cold pressure off (ε_cold = 0) — the
/// quadratic's positive root; 0 when A is not positive.
fn share(physics: &Physics, sigma: f64, p0: [f64; 2], energy: f64, d: [f64; 2]) -> f64 {
    let d2 = d[0] * d[0] + d[1] * d[1];
    if d2 == 0.0 {
        return 1.0;
    }
    let p2 = p0[0] * p0[0] + p0[1] * p0[1];
    let a = energy - 0.5 * p2 / sigma - sigma * physics.get("t_floor").unwrap() * INV_MU;
    if a <= 0.0 {
        return 0.0;
    }
    let q = p0[0] * d[0] + p0[1] * d[1];
    ((-q + (q * q + 2.0 * sigma * a * d2).sqrt()) / d2).min(1.0)
}

/// The cap graded on one step: `forced` and `free` ran from the same state at `dt`, the first under the force `f`. Per half
/// of the world (x left/right), the x momentum gained equals Σ s·Δt·f within `tol` relative, s from the free step's cell;
/// E per cell is the free step's within `e_tol`; the cap binds (s < 1/2) in at least two columns of cells on each side.
fn check_cap(label: &str, physics: &Physics, forced: &State, free: &State, f: &[Vec<f64>; 2], dt: f64, (tol, e_tol): (f64, f64)) {
    let n = forced.cells();
    let (mut gained, mut expected, mut uncapped) = ([0.0; 2], [0.0; 2], [0.0; 2]);
    let (mut bound, mut e_gap) = (0, 0.0f64);
    for i in 0..n {
        let d = [dt * f[0][i], dt * f[1][i]];
        let s = share(physics, free.planes[i], [free.planes[n + i], free.planes[2 * n + i]], free.planes[3 * n + i], d);
        let half = usize::from(i % W >= W / 2);
        gained[half] += forced.planes[n + i] - free.planes[n + i];
        expected[half] += s * d[0];
        uncapped[half] += d[0];
        bound += usize::from(s < 0.5);
        e_gap = e_gap.max((forced.planes[3 * n + i] - free.planes[3 * n + i]).abs() / free.planes[3 * n + i].abs());
    }
    for h in 0..2 {
        let rel = (gained[h] - expected[h]).abs() / expected[h].abs();
        println!(
            "SR-RADFORCE {label} cap, x {}: gained {:.9e}, Σ s·Δt·f {:.9e} ({rel:.2e}), uncapped Σ Δt·f {:.3e}",
            ["left", "right"][h],
            gained[h],
            expected[h],
            uncapped[h]
        );
        assert!(rel <= tol, "{label}: the momentum gained {} against Σ s·Δt·f {} ({rel:e} > {tol:e})", gained[h], expected[h]);
        // The set-up guard: the uncapped impulse is ≥ 10× the capped one, ≥ 100× its work (measured 425× and 1,720×).
        assert!(uncapped[h].abs() > 10.0 * expected[h].abs(), "{label}: the slab must ask far more work than the heat holds");
    }
    println!("SR-RADFORCE {label} cap: {bound} cells capped below half the impulse; E's largest relative gap from the free step {e_gap:.2e}");
    assert!(bound >= 4 * H, "{label}: the cap binds in {bound} cells, expected at least 4 columns");
    assert!(e_gap <= e_tol, "{label}: E moved {e_gap:e} from the free step (P8 refilled a heat the push spent)");
}

/// The twin's P4: the push's work capped at the heat above the floor (§0.5 [M0-D58]).
#[test]
fn twin_the_push_spends_no_more_than_the_heat_above_the_floor() {
    let physics = physics();
    let hydro = Hydro::new(&physics, &elements(), Edge::Bounce).unwrap();
    let heat = Heat::new(&physics, &elements(), pinned()).unwrap();
    let mut st = slab_of(CAP_MEDIUM, CAP_HOT);
    let flux = heat.step(&mut st.clone(), CAP_DT).flux.cells();
    let f = force(&physics, &st, &flux.0, &flux.1);
    let mut free = st.clone();
    hydro.step(&mut free, CAP_DT);
    hydro.step_forced(&mut st, CAP_DT, None, Some((&flux.0, &flux.1)));
    check_cap("twin", &physics, &st, &free, &f, CAP_DT, (CAP_TOL.0, CAP_E_TOL.0));
}

/// The GPU's P4: the same, F as side.flux holds it at the step's start.
#[test]
fn gpu_the_push_spends_no_more_than_the_heat_above_the_floor() {
    let physics = physics();
    let pair = Pair::new(&physics, &slab_of(CAP_MEDIUM, CAP_HOT));
    let dt = f64::from(CAP_DT as f32);
    let (fx, fy) = pair.flux();
    let (start, forced, free) = pair.step(dt);
    check_cap("gpu", &physics, &forced, &free, &force(&physics, &start, &fx, &fy), dt, (CAP_TOL.1, CAP_E_TOL.1));
}

/// M0-D58's block: 50 steps, the shipped constants.
const BLOCK_STEPS: u64 = 50;
/// The mass invariant's band over them (G-CONS, §5.3), relative.
const BLOCK_BAND: f64 = 1e-6;

/// The block of M0-D58 (M0-R4 M9's C/O 5,000): a 2 × 2 rect of Σ 1,250 at T_floor at the centre of a 64 × 64 bounce world,
/// gravity, hydro and heat on.
fn block_scene() -> String {
    r#"{"version": 1, "world": {"width": 64, "height": 64, "edge": "bounce"}, "slowdown": false,
 "switches": {"gravity": true, "hydro": true, "heat": true, "reactions": false, "neutrinos": false, "sinks": false},
 "objects": [{"kind": "rect", "x0": 31, "y0": 31, "w": 2, "h": 2, "sigma": 1250, "t": 0.05, "v": [0, 0],
              "composition": {"C": 0.5, "O": 0.5}}]}"#
        .to_string()
}

/// The twin on the block: 50 steps with every pass the scene runs, every value finite, the grid's mass plus what P8's
/// vacuum reset removed constant within 10⁻⁶ (bounce: nothing escapes).
#[test]
fn twin_a_dense_hot_block_stays_finite_and_keeps_its_mass() {
    let elements = elements();
    let scene = Scene::parse(&block_scene(), &elements).unwrap();
    let physics = scene.physics().unwrap();
    let (planes, _) = scene.planes(scene.world, &physics, &elements).unwrap();
    let records = Reactions::shipped(&elements, &physics).unwrap();
    let burning = Burning { heat: TestFlags::default(), react: react::TestFlags { pin_temperature: false }, reactions: &records, neutrinos: false };
    let switches = Switches { gravity: true, hydro: true, heat: true, reactions: false };
    let mut twin = Twin::new(&physics, &elements, Edge::Bounce, switches, Some(burning), W, H, planes.iter().map(|&v| f64::from(v)).collect()).unwrap();
    let n = W * H;
    let mass = |t: &Twin| t.state.planes[..n].iter().sum::<f64>() + t.booked.vacuum_reset;
    let before = mass(&twin);
    for _ in 0..BLOCK_STEPS {
        twin.step();
    }
    let after = mass(&twin);
    let energy: f64 = twin.state.planes[3 * n..4 * n].iter().sum();
    let drift = (after - before).abs() / before;
    println!(
        "SR-RADFORCE twin block: {BLOCK_STEPS} steps to sim time {:.5}, E {energy:.4e}, floor_added {:.4e}, mass invariant {before:.9} → {after:.9} ({drift:.2e})",
        twin.sim_time(),
        twin.booked.floor_added
    );
    assert_eq!(twin.first_non_finite(), None, "the block's state went non-finite");
    assert!(drift <= BLOCK_BAND, "the mass invariant drifted {drift:e}");
}

/// The GPU on the block, through `headless` in-process on the physics adapter: 50 steps, exit 0, every value of the final
/// dump finite (F's planes included), summary.json's mass invariant within 10⁻⁶ from the ledger's start to its end.
#[test]
fn gpu_a_dense_hot_block_stays_finite_and_keeps_its_mass() {
    let dir: PathBuf = std::env::temp_dir().join(format!("sr-radforce-{}-block", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("scene.json");
    std::fs::write(&file, block_scene()).unwrap();
    let out = dir.join("out");
    let mut args: Vec<String> =
        ["--scene", &file.display().to_string(), "--steps", &BLOCK_STEPS.to_string(), "--out", &out.display().to_string()].map(String::from).to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    assert_eq!(sr_engine::headless::run(&args), 0, "headless");
    let summary: Value = serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
    let dump = Dump::read(&out.join("dumps").join(format!("s_{BLOCK_STEPS}.bin"))).unwrap();
    let mass = |at: &str| summary["ledger"][at]["mass"]["invariant"].as_f64().expect("summary.json's mass invariant");
    let (start, end) = (mass("start"), mass("end"));
    let drift = (end - start).abs() / start;
    let non_finite = dump.planes.iter().chain(dump.flux.iter().flatten()).filter(|v| !v.is_finite()).count();
    println!(
        "SR-RADFORCE gpu block: {} steps, sim time {}, mass invariant {start:.9} → {end:.9} ({drift:.2e}), floor_added {}, {non_finite} non-finite values in the final dump",
        summary["steps"], summary["sim_time"], summary["ledger"]["end"]["energy"]["floor_added"]
    );
    assert_eq!(non_finite, 0, "the block's final state holds non-finite values");
    assert!(drift <= BLOCK_BAND, "the mass invariant drifted {drift:e}");
    let _ = std::fs::remove_dir_all(&dir);
}
