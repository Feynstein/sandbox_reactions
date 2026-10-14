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

use sr_engine::gpu::Gpu;
use sr_engine::scene::{EdgeMode, TestSettings};
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::HeatSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::heat::{Heat, TestFlags};
use sr_physics::reference::hydro::{Edge, Hydro, State};
use sr_physics::registry::{Elements, Physics};

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
    let n = W * H;
    let mut planes = vec![0.0; 14 * n];
    for i in 0..n {
        let (sigma, t) = if SLAB.contains(&(i % W)) { HOT } else { MEDIUM };
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
