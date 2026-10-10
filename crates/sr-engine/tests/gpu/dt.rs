//! M0-T20 — P1 Δt and P9's reduction and non-finite guard (contract §1.3.2 P1 and P9, §1.3.4, §2.3.3, §2.3.5, §3.3).
//!
//! The oracle is §2.3.5's formula in f64 from the same f32 state: Δt = min(C · min_cells [(|u| + c)/Δx + (|v| + c)/Δy]⁻¹,
//! η_g · min_cells √(Δx/|g|)) with C = 0.4 and η_g = 0.3 (§2.7's table, written here, not read from physics.json),
//! Δx = Δy = 1 (§2.1), the gravity term infinite until P2 writes g. The fields are "of known u and c": the cold pairs
//! off, so Π = Σ ε_th and c² = 2ε_th (§2.3.1, §2.3.3), each cell built from a chosen (u, v, c); a cell below Σ_vac counts
//! as P8's floor state (u = 0, c² = 2 T_floor/μ).
//!
//! Tolerance, fixed before the first run (R8; the block's figure): Δt within 10⁻⁶ relative of the f64 oracle. The f32
//! path is ε_th = E/Σ − ½|u|², E/Σ ≤ 3 ε_th here (≈ 2⁻²⁴ × 3 × a few operations ≈ 2 × 10⁻⁷), then a square root and two
//! sums (≈ 10⁻⁷) — the max itself is exact. Bit-identity across runs: exact.

use super::device;
use sr_engine::gpu::Gpu;
use sr_engine::state::{Booking, State, WorldConfig, CHANNELS, N_CHANNELS, SPECIES};
use sr_engine::step::dt::{DtReport, NonFinite, GUARD_EVERY};
use sr_engine::step::{EosGpu, Pass, Step};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// §2.3.5 / §2.7: C and η_g as the contract fixes them.
const C: f64 = 0.4;
const ETA_G: f64 = 0.3;
const DT_TOL: f64 = 1e-6;

/// A fixed sequence of values in [0, 1) — the same on every run (xorshift64*).
struct Seq(u64);

impl Seq {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next()
    }
}

struct Constants {
    physics: Physics,
    elements: Elements,
}

/// The shipped constants with both cold pairs off (a cold pair's pressure would make c depend on the u(x) table).
fn constants() -> Constants {
    let shipped = Physics::shipped().expect("shipped physics.json");
    assert_eq!(shipped.get("cfl"), Some(C), "physics.json's cfl is §2.3.5's C");
    assert_eq!(shipped.get("eta_g"), Some(ETA_G), "physics.json's eta_g is §2.3.5's η_g");
    let physics = shipped.with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).expect("pairs off");
    Constants { physics, elements: Elements::shipped().expect("shipped elements.json") }
}

/// A state, its step and nothing else.
struct Run {
    state: State,
    step: Step,
}

impl Run {
    fn new(gpu: &Gpu, c: &Constants, world: WorldConfig, planes: &[f32]) -> Run {
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let step = Step::new(&gpu.device, &state, &booking, &eos, &c.physics);
        Run { state, step }
    }

    fn run(&self, gpu: &Gpu, steps: u32) -> DtReport {
        self.step.run(&gpu.device, &gpu.queue, steps);
        self.step.read_dt(&gpu.device, &gpu.queue)
    }

    fn planes(&self, gpu: &Gpu) -> Vec<f32> {
        self.state.readback(&gpu.device, &gpu.queue)
    }
}

/// A field of known u and c: Σ in [0.5, 2), (u, v) in [−2, 2)², c in [0.5, 3), mass fractions drawn and normalised in
/// f64; every 97th cell vacuum (Σ = 0, x_H = 1); the fastest cell — u = 8, v = −5, c = 6 — at `peak`, if any.
fn field(world: WorldConfig, seed: u64, peak: Option<usize>) -> Vec<f32> {
    let cells = world.cells();
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut seq = Seq(seed);
    for i in 0..cells {
        if i % 97 == 13 && Some(i) != peak {
            planes[SPECIES.start * cells + i] = 1.0;
            continue;
        }
        let sigma = seq.range(0.5, 2.0);
        let (u, v, c) = if Some(i) == peak { (8.0, -5.0, 6.0) } else { (seq.range(-2.0, 2.0), seq.range(-2.0, 2.0), seq.range(0.5, 3.0)) };
        let eps_th = 0.5 * c * c;
        planes[i] = sigma as f32;
        planes[cells + i] = (sigma * u) as f32;
        planes[2 * cells + i] = (sigma * v) as f32;
        planes[3 * cells + i] = (sigma * (eps_th + 0.5 * (u * u + v * v))) as f32;
        let x: Vec<f64> = (0..N_SPECIES).map(|_| seq.next()).collect();
        let sum: f64 = x.iter().sum();
        for (k, xk) in x.iter().enumerate() {
            planes[(SPECIES.start + k) * cells + i] = (xk / sum) as f32;
        }
    }
    planes
}

/// §2.3.5's Δt in f64 from the f32 state, the cold pairs off and no gravity.
fn oracle(c: &Constants, world: WorldConfig, planes: &[f32]) -> f64 {
    let cells = world.cells();
    let get = |ch: usize, i: usize| planes[ch * cells + i] as f64;
    let key = |k: &str| c.physics.get(k).unwrap() as f32 as f64;
    let mut top = 0f64;
    for i in 0..cells {
        let x: [f64; N_SPECIES] = std::array::from_fn(|k| get(SPECIES.start + k, i));
        let eps_floor = key("t_floor") * c.elements.composition(&x).inv_mu;
        let (u, v, eps_th) = if get(0, i) < key("sigma_vac") {
            (0.0, 0.0, eps_floor)
        } else {
            let sigma = get(0, i);
            let (u, v) = (get(1, i) / sigma, get(2, i) / sigma);
            (u, v, (get(3, i) / sigma - 0.5 * (u * u + v * v)).max(eps_floor))
        };
        let cs = (2.0 * eps_th).sqrt();
        top = top.max((u.abs() + cs) + (v.abs() + cs));
    }
    let gravity = f64::INFINITY; // until P2 writes g
    (C / top).min(gravity * ETA_G)
}

fn close(got: f32, want: f64) -> bool {
    (got as f64 - want).abs() <= DT_TOL * want.abs()
}

#[test]
fn p1_advances_dt_and_p9_reduces_it_and_guards_every_64th_step() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let run = Run::new(&gpu, &c, world, &field(world, 1, None));
    assert_eq!(run.step.dispatch_labels(Pass::Dt), ["dt_advance"]);
    assert_eq!(run.step.dispatch_labels(Pass::Reductions), ["dt_cells", "dt_partials"]);
    assert_eq!(run.step.guard_labels(), ["guard_scan", "guard_stamp"]);
    assert_eq!(GUARD_EVERY, 64);
}

#[test]
fn dt_equals_the_cfl_formula_on_fields_of_known_u_and_c() {
    let gpu = device();
    let c = constants();
    // The fastest cell first, last, and inside a later workgroup; 72 × 88 = 6,336 cells leaves a part-filled last tile.
    // With no planted peak, whichever drawn cell is fastest sets Δt, so every cell's u and c are graded.
    let fields = [
        ((64, 64), Some(0usize)),
        ((72, 88), Some(72 * 88 - 1)),
        ((600, 400), Some(123_457)),
        ((600, 400), Some(0)),
        ((64, 64), None),
        ((72, 88), None),
        ((600, 400), None),
    ];
    for (seed, &((w, h), peak)) in fields.iter().enumerate() {
        let world = WorldConfig::new(w, h).unwrap();
        let start = field(world, 0x5eed_0020 + seed as u64, peak);
        let run = Run::new(&gpu, &c, world, &start);
        let report = run.run(&gpu, 1);
        let after = run.planes(&gpu);
        let (want0, want1) = (oracle(&c, world, &start), oracle(&c, world, &after));
        println!(
            "{w} × {h}, peak at {peak:?}: Δt_0 = {:e} (oracle {want0:e}), Δt_1 = {:e} (oracle {want1:e}), tolerance {DT_TOL:e}",
            report.dt, report.dt_next
        );
        assert!(close(report.dt, want0), "{w} × {h}: Δt_0 {} against the f64 formula {want0}", report.dt);
        assert!(close(report.dt_next, want1), "{w} × {h}: Δt_1 {} against the f64 formula {want1}", report.dt_next);
        // A planted peak bounds Δt: C / (|u| + |v| + 2c) = 0.4 / 25; the drawn cells stay below |u| + |v| + 2c = 10.
        let peak_dt = C / 25.0;
        assert_eq!(close(report.dt, peak_dt), peak.is_some(), "{w} × {h}: Δt_0 {} against the peak's {peak_dt}", report.dt);
        assert_eq!(report.steps, 1);
        assert_eq!(report.non_finite, None);
    }
}

#[test]
fn step_n_runs_on_the_dt_p9_made_at_the_end_of_step_n_minus_1() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let a = field(world, 7, Some(100));
    let run = Run::new(&gpu, &c, world, &a);
    let first = run.run(&gpu, 1);
    assert!(close(first.dt, oracle(&c, world, &a)), "step 0 runs on the initial state's reduction: {}", first.dt);

    // An edit between steps (the state replaced, its fastest cell slower): step 1 must still run on step 0's P9 value.
    let mut b = field(world, 8, Some(2000));
    let cells = world.cells();
    for ch in [1, 2] {
        b[ch * cells + 2000] *= 0.25;
    }
    run.state.upload(&gpu.queue, &b).unwrap();
    let second = run.run(&gpu, 1);
    let after = run.planes(&gpu);
    println!("Δt_0 = {:e}, Δt_1 = {:e} (P9 of step 0), Δt_2 = {:e} (P9 of step 1)", first.dt, second.dt, second.dt_next);
    assert_eq!(second.dt.to_bits(), first.dt_next.to_bits(), "step 1's Δt is P9's value from the end of step 0");
    assert!(close(second.dt_next, oracle(&c, world, &after)), "Δt_2 against the edited state: {}", second.dt_next);
    assert_ne!(second.dt_next.to_bits(), second.dt.to_bits(), "the edit changed nothing: the case cannot tell the lag");
    assert_eq!(second.steps, 2);
}

#[test]
fn dt_and_the_state_are_bit_identical_over_two_runs_on_a_600_by_400_field() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(600, 400).unwrap();
    let start = field(world, 0x5eed_0b17, None);
    let trace = || {
        let run = Run::new(&gpu, &c, world, &start);
        let reports: Vec<DtReport> = [1, 63, 1, 65].iter().map(|&n| run.run(&gpu, n)).collect();
        let bits: Vec<[u32; 2]> = reports.iter().map(|r| [r.dt.to_bits(), r.dt_next.to_bits()]).collect();
        (bits, run.planes(&gpu).iter().map(|v| v.to_bits()).collect::<Vec<u32>>())
    };
    let (a, b) = (trace(), trace());
    println!("Δt over 130 steps, run 1: {:?}", a.0.iter().map(|w| f32::from_bits(w[0])).collect::<Vec<_>>());
    assert_eq!(a.0, b.0, "Δt differs between two runs");
    assert!(a.1 == b.1, "the state differs between two runs");
}

#[test]
fn the_guard_names_the_step_cell_and_channel_of_a_planted_non_finite_value() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let at = |x: usize, y: usize| y * 64 + x;
    let energy = CHANNELS.iter().position(|&n| n == "energy").unwrap();
    let mom_y = CHANNELS.iter().position(|&n| n == "mom_y").unwrap();

    // Before step 0: the guard's first scan (step 0) finds it.
    let mut start = field(world, 3, None);
    start[energy * cells + at(37, 21)] = f32::NAN;
    let run = Run::new(&gpu, &c, world, &start);
    let report = run.run(&gpu, 5);
    assert_eq!(report.non_finite, Some(NonFinite { step: 0, x: 37, y: 21, channel: "energy" }));

    // A clean run: 70 steps, no finding. Then two values planted before step 70 — an Inf momentum at (5, 60) and a NaN
    // energy at (50, 2), the lower cell index — found by the scan at step 128, never 64, and kept past the next scan.
    let run = Run::new(&gpu, &c, world, &field(world, 3, None));
    assert_eq!(run.run(&gpu, 70).non_finite, None, "a clean run tripped the guard");
    let mut planes = run.planes(&gpu);
    planes[mom_y * cells + at(5, 60)] = f32::INFINITY;
    planes[energy * cells + at(50, 2)] = f32::NAN;
    run.state.upload(&gpu.queue, &planes).unwrap();
    assert_eq!(run.run(&gpu, 57).non_finite, None, "steps 70–126: no scan between 64 and 128");
    let found = Some(NonFinite { step: 128, x: 50, y: 2, channel: "energy" });
    assert_eq!(run.run(&gpu, 2).non_finite, found, "the scan at step 128");
    assert_eq!(run.run(&gpu, 80).non_finite, found, "the first finding is kept past the scan at step 192");
}

#[test]
fn the_guard_names_a_nan_fraction_that_p8_would_renormalise_away() {
    // M0-D18: P8's clamp may read max(NaN, 0) as 0 — a NaN x_C planted before step 0 must still be there for the scan
    // at the end of step 0, which names its step, cell and channel; the cell's fractions stay as planted.
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let i = 10 * 64 + 20;
    let x_c = CHANNELS.iter().position(|&n| n == "x_C").unwrap();
    let mut start = field(world, 3, None);
    start[x_c * cells + i] = f32::NAN;
    let run = Run::new(&gpu, &c, world, &start);
    let report = run.run(&gpu, 1);
    assert_eq!(report.non_finite, Some(NonFinite { step: 0, x: 20, y: 10, channel: "x_C" }));
    let planes = run.planes(&gpu);
    for ch in SPECIES {
        let (b, a) = (start[ch * cells + i], planes[ch * cells + i]);
        assert_eq!(a.to_bits(), b.to_bits(), "(20, 10) {}: {b} → {a}, P8 renormalised a NaN cell", CHANNELS[ch]);
    }
}
