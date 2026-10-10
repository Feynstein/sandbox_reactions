//! M0-T19 — P8 complete and the equation of state on the GPU (contract §1.3.2 P8, §1.5, §2.3.2–§2.3.3, §2.7, §2.9):
//! eos.wgsl's helpers against M0-T12's CPU EOS (sr_physics::eos) on a sweep of densities and compositions, with the cold
//! pairs on and off; then a 64 × 64 field of vacuum, cold, dense cold and hot cells, their fractions normalised,
//! unnormalised, of mixed signs or all ≤ 0 → one step → each rule against the f64 twin of P8 below, and the booked
//! `vacuum_reset` and `floor_added` against the mass and energy the step changed.
//!
//! The twin (`p8_twin`) is P8 in f64 from the same f32 inputs: the fractions renormalised (pure hydrogen when none is
//! positive, §2.7 [M0-T19]); μ, Y_e and X_n of those; below Σ_vac the floor state Σ_floor, u = 0,
//! E = Σ_floor (T_floor/μ + ε_cold(Σ_floor)); else Eos::thermal's floor. M0-T37 moves it into sr_physics::reference.
//!
//! Tolerances, fixed before the first run (R8): a value read through the u(x) table — ε_cold, and every E or Π built
//! on it — within 2 × 10⁻⁵ relative: the f32 position in the table carries |ln x − ln X_MIN| ≲ 37 at 2⁻²⁴ each, × 13.9
//! nodes per unit of ln x, ≈ 3 × 10⁻⁵ of a node, × Δ ln u ≤ 0.15 per node ≈ 5 × 10⁻⁶; WGSL's exp of |ln u| ≲ 40 is
//! good to 3 + 2·40 ULP ≈ 5 × 10⁻⁶; ln u stored in f32, 40 × 2⁻²⁴ ≈ 2.4 × 10⁻⁶ — the sum doubled. A value of f32
//! arithmetic alone (1/μ, Y_e, the closed-form P) within 2 × 10⁻⁶ relative; Z_met within 10⁻⁶ absolute (it is a
//! difference). What the floors leave untouched: bit-identical. The booked terms equal the change within 10⁻⁶
//! relative (the block's figure), cell by cell and in total.

use super::device;
use sr_engine::state::{book_totals, Booking, State, WorldConfig, BOOK_GROUPS, CHANNELS, N_CHANNELS, SPECIES, TERMS};
use sr_engine::step::{with_eos, EosGpu, Pass, Step};
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const TABLE_TOL: f64 = 2e-5;
const ARITH_TOL: f64 = 2e-6;
const Z_MET_TOL: f64 = 1e-6;
const BOOK_TOL: f64 = 1e-6;

/// A fixed sequence of values in [0, 1) — the same on every run (xorshift64*).
struct Seq(u64);

impl Seq {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// |got − want| within `tol` of |want|; a want of 0 asks for exactly 0.
fn close(got: f64, want: f64, tol: f64) -> bool {
    (got - want).abs() <= tol * want.abs()
}

struct Constants {
    physics: Physics,
    elements: Elements,
    eos: Eos,
}

fn shipped() -> Constants {
    let physics = Physics::shipped().expect("shipped physics.json");
    let eos = Eos::from_physics(&physics).expect("shipped constants");
    Constants { physics, elements: Elements::shipped().expect("shipped elements.json"), eos }
}

impl Constants {
    /// A physics.json value as the GPU holds it (f32), back in f64.
    fn f32_of(&self, key: &str) -> f64 {
        self.physics.get(key).unwrap() as f32 as f64
    }
}

// ---------------------------------------------------------------------------------------------------------------
// eos.wgsl's helpers, evaluated one input per invocation

const HELPERS_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> inputs: array<f32>;
@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;

const IN: u32 = 12u;   // sigma, eps_th, x[10]
const OUT: u32 = 9u;   // 1/mu, Y_e, Z_met, Pi_e, Pi_n, eps_cold, Pi, c^2, T

@compute @workgroup_size(64)
fn evaluate(@builtin(global_invocation_id) id: vec3<u32>) {
    let k = id.x;
    if (k >= arrayLength(&inputs) / IN) {
        return;
    }
    let sigma = inputs[k * IN];
    let eps_th = inputs[k * IN + 1u];
    var x: array<f32, 10>;
    for (var i = 0u; i < 10u; i++) {
        x[i] = inputs[k * IN + 2u + i];
    }
    let c = composition(x);
    let pi = pressure(sigma, eps_th, c.y_e, x[ID_N]);
    let o = k * OUT;
    outputs[o] = c.inv_mu;
    outputs[o + 1u] = c.y_e;
    outputs[o + 2u] = c.z_met;
    outputs[o + 3u] = pi_e(sigma, c.y_e);
    outputs[o + 4u] = pi_n(sigma, x[ID_N]);
    outputs[o + 5u] = eps_cold(sigma, c.y_e, x[ID_N]);
    outputs[o + 6u] = pi;
    outputs[o + 7u] = wave_speed_sq(sigma, pi);
    outputs[o + 8u] = temperature(eps_th, c.inv_mu);
}
"#;

/// Runs `evaluate` over `inputs` (12 f32 per case) with the EOS of `physics`; 9 f32 per case back.
fn evaluate_on_gpu(physics: &Physics, elements: &Elements, inputs: &[f32]) -> Vec<f32> {
    let gpu = device();
    let (device, queue) = (&gpu.device, &gpu.queue);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("eos helpers"),
        source: wgpu::ShaderSource::Wgsl(with_eos(HELPERS_WGSL).into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("eos helpers"),
        layout: None,
        module: &module,
        entry_point: Some("evaluate"),
        compilation_options: Default::default(),
        cache: None,
    });
    let cases = inputs.len() / 12;
    let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
    let buffer = |label, size| {
        device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage: storage, mapped_at_creation: false })
    };
    let input = buffer("inputs", (inputs.len() * 4) as u64);
    let output = buffer("outputs", (cases * 9 * 4) as u64);
    queue.write_buffer(&input, 0, &inputs.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    let own = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("eos helpers"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: input.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: output.as_entire_binding() },
        ],
    });
    let eos = EosGpu::new(device, physics, elements).bind_group(device, &pipeline);
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("outputs.readback"),
        size: output.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("eos helpers") });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &own, &[]);
        cpass.set_bind_group(1, &eos, &[]);
        cpass.dispatch_workgroups((cases as u32).div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &staging, 0, output.size());
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).expect("device lost");
    rx.recv().unwrap().expect("map failed");
    let values = staging.get_mapped_range(..).unwrap().as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect();
    staging.unmap();
    values
}

/// Fractions in §1.5's order: the pure species, a solar-like mix, a carbon–oxygen core, iron with neutron matter.
fn compositions() -> Vec<[f64; N_SPECIES]> {
    let mut list = Vec::new();
    for s in 0..N_SPECIES {
        let mut x = [0.0; N_SPECIES];
        x[s] = 1.0;
        list.push(x);
    }
    list.push([0.7346, 0.2485, 0.0077, 0.0092, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    list.push([0.0, 0.02, 0.48, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    list.push([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.05, 0.45, 0.4]);
    list
}

fn check_helpers(c: &Constants, label: &str) -> usize {
    // Σ log-spaced over [1e-9, 1e9] (x = Y_eΣ crosses the table's ends, 1e-8 and 1e8), ε_th over [0.01, 100].
    let sigmas: Vec<f64> = (0..37).map(|j| 10f64.powf(-9.0 + 0.5 * j as f64)).collect();
    let mut inputs = Vec::new();
    let mut want = Vec::new();
    for x in compositions() {
        let x32 = x.map(|v| v as f32);
        let x = x32.map(f64::from);
        let comp = c.elements.composition(&x);
        for (j, &s) in sigmas.iter().enumerate() {
            let (sigma, eps_th) = (s as f32 as f64, 10f64.powf(-2.0 + 4.0 * (j % 9) as f64 / 8.0) as f32 as f64);
            inputs.push(sigma as f32);
            inputs.push(eps_th as f32);
            inputs.extend(x32);
            let pi = c.eos.pi(sigma, eps_th, comp.y_e, x[9]);
            want.push([
                comp.inv_mu,
                comp.y_e,
                comp.z_met,
                c.eos.pi_e(sigma, comp.y_e),
                c.eos.pi_n(sigma, x[9]),
                c.eos.eps_cold(sigma, comp.y_e, x[9]),
                pi,
                Eos::wave_speed_sq(sigma, pi),
                Eos::temperature(eps_th, 1.0 / comp.inv_mu),
            ]);
        }
    }
    let got = evaluate_on_gpu(&c.physics, &c.elements, &inputs);
    const NAMES: [&str; 9] = ["1/mu", "Y_e", "Z_met", "Pi_e", "Pi_n", "eps_cold", "Pi", "c^2", "T"];
    let mut worst = [0f64; 9];
    for (k, w) in want.iter().enumerate() {
        for q in 0..9 {
            let g = got[k * 9 + q] as f64;
            let ok = match q {
                2 => (g - w[q]).abs() <= Z_MET_TOL,
                0 | 1 | 3 | 4 | 8 => close(g, w[q], ARITH_TOL),
                _ => close(g, w[q], TABLE_TOL),
            };
            let (sigma, x) = (inputs[k * 12], &inputs[k * 12 + 2..k * 12 + 12]);
            assert!(ok, "{label}: {} at Σ = {sigma:e}, X = {x:?}: GPU {g:e}, CPU EOS {:e}", NAMES[q], w[q]);
            if w[q] != 0.0 {
                worst[q] = worst[q].max(((g - w[q]) / w[q]).abs());
            }
        }
    }
    let worst: Vec<String> = NAMES.iter().zip(worst).map(|(n, w)| format!("{n} {w:.1e}")).collect();
    println!("{label}: {} cases; worst relative error: {}", want.len(), worst.join(", "));
    want.len()
}

#[test]
fn eos_helpers_equal_the_cpu_eos_with_the_cold_pairs_on_and_off() {
    let on = shipped();
    assert!(on.eos.electron_table().is_some() && on.eos.neutron_table().is_some());
    let n = check_helpers(&on, "cold pairs on");
    let physics = on.physics.with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).expect("pairs off");
    let off = Constants { eos: Eos::from_physics(&physics).unwrap(), physics, elements: on.elements };
    assert!(off.eos.electron_table().is_none() && off.eos.neutron_table().is_none());
    check_helpers(&off, "cold pairs off");
    assert!(n >= 13 * 37, "{n} cases");
}

// ---------------------------------------------------------------------------------------------------------------
// P8 on a field, against its f64 twin

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Vacuum,
    Cold,
    ColdDense,
    Hot,
}

impl Kind {
    fn of(cell: usize) -> Kind {
        match cell % 8 {
            0 | 1 => Kind::Vacuum,
            2 | 3 | 7 => Kind::Cold,
            4 => Kind::ColdDense,
            _ => Kind::Hot,
        }
    }
}

/// The fractions as P8 leaves them, in f64: clamped ≥ 0 and divided by their sum; pure hydrogen when none is positive.
fn renormalised(x: &[f32]) -> [f64; N_SPECIES] {
    let clamped: Vec<f64> = x.iter().map(|&v| (v as f64).max(0.0)).collect();
    let sum: f64 = clamped.iter().sum();
    let mut out = [0.0; N_SPECIES];
    if sum > 0.0 {
        for (o, v) in out.iter_mut().zip(&clamped) {
            *o = v / sum;
        }
    } else {
        out[0] = 1.0;
    }
    out
}

/// One cell after P8, in f64, and what P8 books for it.
#[derive(Debug)]
struct Twin {
    sigma: f64,
    mom: [f64; 2],
    energy: f64,
    x: [f64; N_SPECIES],
    mass_out: f64,
    energy_added: f64,
    reset: bool,
    floored: bool,
}

/// P8's f64 twin for one cell of the canonical planes.
fn p8_twin(c: &Constants, planes: &[f32], cells: usize, i: usize) -> Twin {
    let at = |ch: usize| planes[ch * cells + i] as f64;
    let raw: Vec<f32> = SPECIES.map(|ch| planes[ch * cells + i]).collect();
    let x = renormalised(&raw);
    let comp = c.elements.composition(&x);
    let mu = 1.0 / comp.inv_mu;
    let (sigma, mom, energy) = (at(0), [at(1), at(2)], at(3));
    if sigma < c.f32_of("sigma_vac") {
        let s = c.f32_of("sigma_floor");
        let e = s * (Eos::eps_th_of_t(c.eos.t_floor(), mu) + c.eos.eps_cold(s, comp.y_e, x[9]));
        return Twin {
            sigma: s,
            mom: [0.0, 0.0],
            energy: e,
            x,
            mass_out: sigma - s,
            energy_added: e - energy,
            reset: true,
            floored: false,
        };
    }
    let t = c.eos.thermal(energy, sigma, mom, comp.y_e, x[9], mu);
    Twin {
        sigma,
        mom,
        energy: energy + t.floor_added,
        x,
        mass_out: 0.0,
        energy_added: t.floor_added,
        reset: false,
        floored: t.floor_added > 0.0,
    }
}

/// 64 × 64 cells: the kinds by `Kind::of`, the fractions by (cell / 8) mod 4 — normalised (rich in H and n),
/// unnormalised, mixed signs, all ≤ 0. Cold cells sit at ε_th = f · T_floor/μ with f ∈ [−3, 0.3] (dense: [−4, −1], so
/// the f32 ε_cold of Σ up to 60 cannot flip the floor's decision), hot ones at f ∈ [10, 10⁴].
fn field(c: &Constants, world: WorldConfig) -> Vec<f32> {
    let cells = world.cells();
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut seq = Seq(0x5eed_0019);
    const VACUUM_SIGMA: [f64; 8] = [0.0, -1e-3, 5e-7, 1e-8, 9.9e-7, -0.0, 2e-7, 1e-9];
    for i in 0..cells {
        let mut raw = [0f64; N_SPECIES];
        let pattern = (i / 8) % 4;
        for (k, r) in raw.iter_mut().enumerate() {
            *r = match pattern {
                0 => seq.next() * if k == 0 || k == 9 { 4.0 } else { 1.0 },
                1 => 3.0 * seq.next(),
                2 => 3.0 * seq.next() - 1.0,
                _ => -seq.next(),
            };
        }
        if pattern == 0 {
            let sum: f64 = raw.iter().sum();
            raw.iter_mut().for_each(|r| *r /= sum);
        }
        for (k, r) in raw.iter().enumerate() {
            planes[(SPECIES.start + k) * cells + i] = *r as f32;
        }
        let x = renormalised(&raw.map(|r| r as f32));
        let comp = c.elements.composition(&x);
        let floor = c.eos.t_floor() * comp.inv_mu;
        let v = [4.0 * seq.next() - 2.0, 4.0 * seq.next() - 2.0];
        let (sigma, f) = match Kind::of(i) {
            Kind::Vacuum => (VACUUM_SIGMA[(i / 32) % 8], 0.0),
            Kind::Cold => (0.1 * 100f64.powf(seq.next()), -3.0 + 3.3 * seq.next()),
            Kind::ColdDense => (20.0 + 40.0 * seq.next(), -4.0 + 3.0 * seq.next()),
            Kind::Hot => (1e-5 * 1e7f64.powf(seq.next()), 10.0 + 1e4 * seq.next()),
        };
        let sigma = sigma as f32 as f64;
        let energy = if Kind::of(i) == Kind::Vacuum {
            2.0 * seq.next() - 1.0
        } else {
            sigma * (c.eos.eps_cold(sigma, comp.y_e, x[9]) + f * floor) + 0.5 * sigma * (v[0] * v[0] + v[1] * v[1])
        };
        planes[i] = sigma as f32;
        planes[cells + i] = (sigma * v[0]) as f32;
        planes[2 * cells + i] = (sigma * v[1]) as f32;
        planes[3 * cells + i] = energy as f32;
    }
    planes
}

/// One step of the field: the state before and after, and P8's two booking planes (vacuum_reset, floor_added).
struct Run {
    world: WorldConfig,
    before: Vec<f32>,
    after: Vec<f32>,
    vacuum_reset: Vec<f32>,
    floor_added: Vec<f32>,
    totals: Vec<(&'static str, f64)>,
}

fn run_p8(c: &Constants) -> Run {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let before = field(c, world);
    let state = State::new(&gpu.device, world).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &before).unwrap();
    let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
    Step::new(&gpu.device, &state, &booking, &eos).run(&gpu.device, &gpu.queue, 1);
    let after = state.readback(&gpu.device, &gpu.queue);
    let book = booking.readback(&gpu.device, &gpu.queue);
    let totals = book_totals(world, &book);
    // The planes of a term: its group's offset in the readback, then its index in the group.
    let plane = |name: &str| -> Vec<f32> {
        let mut base = 0;
        for g in &BOOK_GROUPS {
            let slots = g.domain.slots(world);
            for t in g.terms.clone() {
                if TERMS[t].name == name {
                    return book[base..base + slots].to_vec();
                }
                base += slots;
            }
        }
        panic!("no booked term {name}");
    };
    Run {
        world,
        vacuum_reset: plane("mass.vacuum_reset"),
        floor_added: plane("energy.floor_added"),
        totals: TERMS.iter().map(|t| t.name).zip(totals).collect(),
        before,
        after,
    }
}

impl Run {
    fn cells(&self) -> usize {
        self.world.cells()
    }

    fn get(&self, planes: &[f32], ch: usize, i: usize) -> f32 {
        planes[ch * self.cells() + i]
    }
}

#[test]
fn p8_dispatches_the_floors_ahead_of_the_renormalisation() {
    let gpu = device();
    let c = shipped();
    let world = WorldConfig::new(64, 64).unwrap();
    let state = State::new(&gpu.device, world).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let step = Step::new(&gpu.device, &state, &booking, &EosGpu::new(&gpu.device, &c.physics, &c.elements));
    assert_eq!(step.dispatch_labels(Pass::Floors), ["floor_cells", "renormalise_species"]);
    for pass in Pass::ORDER.into_iter().filter(|&p| p != Pass::Floors) {
        assert!(step.dispatch_labels(pass).is_empty(), "{} holds a dispatch", pass.label());
    }
}

#[test]
fn vacuum_cells_reset_to_the_floor_state() {
    let c = shipped();
    let run = run_p8(&c);
    let sigma_floor = c.physics.get("sigma_floor").unwrap() as f32;
    let (mut n, mut worst) = (0, 0f64);
    for i in 0..run.cells() {
        let twin = p8_twin(&c, &run.before, run.cells(), i);
        assert_eq!(twin.reset, Kind::of(i) == Kind::Vacuum, "cell {i}: the field's vacuum cells are the twin's");
        if !twin.reset {
            continue;
        }
        n += 1;
        let sigma = run.get(&run.after, 0, i);
        assert_eq!(sigma.to_bits(), sigma_floor.to_bits(), "cell {i}: Σ {sigma:e}, Σ_floor {sigma_floor:e}");
        assert_eq!((run.get(&run.after, 1, i), run.get(&run.after, 2, i)), (0.0, 0.0), "cell {i}: u ≠ 0");
        let e = run.get(&run.after, 3, i) as f64;
        assert!(close(e, twin.energy, TABLE_TOL), "cell {i}: E {e:e}, the twin's {:e} (T = T_floor)", twin.energy);
        worst = worst.max(((e - twin.energy) / twin.energy).abs());
        // and the temperature the state now holds is T_floor (ε_th = E/Σ − ε_cold, T = μ ε_th)
        let comp = c.elements.composition(&twin.x);
        let t = (e / sigma as f64 - c.eos.eps_cold(sigma as f64, comp.y_e, twin.x[9])) / comp.inv_mu;
        assert!(close(t, c.eos.t_floor(), 1e-4), "cell {i}: T {t}, T_floor {}", c.eos.t_floor());
    }
    assert!(n >= 1000, "{n} vacuum cells");
    println!("{n} vacuum cells reset: Σ = Σ_floor bit-exact, u = 0, worst |E − twin|/E = {worst:e} (tol {TABLE_TOL:e})");
}

#[test]
fn cold_cells_rise_to_t_floor_and_hot_cells_stay_bit_identical() {
    let c = shipped();
    let run = run_p8(&c);
    let (mut floored, mut untouched, mut worst) = (0, 0, 0f64);
    for i in 0..run.cells() {
        let twin = p8_twin(&c, &run.before, run.cells(), i);
        match Kind::of(i) {
            Kind::Vacuum => continue,
            Kind::Cold | Kind::ColdDense => assert!(twin.floored, "cell {i}: a cold cell the twin does not floor"),
            Kind::Hot => assert!(!twin.floored, "cell {i}: a hot cell the twin floors"),
        }
        for ch in 0..3 {
            let (b, a) = (run.get(&run.before, ch, i), run.get(&run.after, ch, i));
            assert_eq!(a.to_bits(), b.to_bits(), "cell {i}: {} {b} → {a}, the floor moves only E", CHANNELS[ch]);
        }
        let (e_before, e) = (run.get(&run.before, 3, i), run.get(&run.after, 3, i));
        if twin.floored {
            floored += 1;
            assert!(close(e as f64, twin.energy, TABLE_TOL), "cell {i} ({:?}): E {e:e}, the twin's {:e}", Kind::of(i), twin.energy);
            worst = worst.max(((e as f64 - twin.energy) / twin.energy).abs());
        } else {
            untouched += 1;
            assert_eq!(e.to_bits(), e_before.to_bits(), "cell {i}: hot, E {e_before:e} → {e:e}");
        }
    }
    assert!(floored >= 1500 && untouched >= 1000, "{floored} floored, {untouched} untouched");
    println!("{floored} cold cells floored: worst |E − twin|/E = {worst:e} (tol {TABLE_TOL:e}); {untouched} hot cells bit-identical");
}

#[test]
fn species_renormalised_after_the_floors() {
    let c = shipped();
    let run = run_p8(&c);
    let mut worst = 0f64;
    for i in 0..run.cells() {
        let twin = p8_twin(&c, &run.before, run.cells(), i);
        let x: Vec<f32> = SPECIES.map(|ch| run.get(&run.after, ch, i)).collect();
        let sum: f64 = x.iter().map(|&v| v as f64).sum();
        assert!(x.iter().all(|&v| v >= 0.0), "cell {i}: a negative fraction {x:?}");
        assert!((sum - 1.0).abs() <= 1e-6, "cell {i}: Σ X = {sum}");
        for (k, (&got, want)) in x.iter().zip(&twin.x).enumerate() {
            let err = (got as f64 - want).abs();
            assert!(err <= 1e-6, "cell {i}, {}: {got}, the twin's {want}", CHANNELS[SPECIES.start + k]);
            worst = worst.max(err);
        }
    }
    let all_le_0 = (0..run.cells()).filter(|i| (i / 8) % 4 == 3).count();
    assert!(all_le_0 > 0 && (0..run.cells()).filter(|i| (i / 8) % 4 == 3).all(|i| run.get(&run.after, 4, i) == 1.0));
    println!("{} cells: worst |X − twin| = {worst:e}; {all_le_0} all-≤ 0 cells now pure hydrogen", run.cells());
}

#[test]
fn booked_terms_equal_the_mass_and_energy_changed() {
    let c = shipped();
    let run = run_p8(&c);
    let (mut mass, mut mass_abs, mut energy, mut energy_abs) = (0f64, 0f64, 0f64, 0f64);
    for i in 0..run.cells() {
        // what the step changed, exactly, from the f32 states
        let dm = run.get(&run.before, 0, i) as f64 - run.get(&run.after, 0, i) as f64;
        let de = run.get(&run.after, 3, i) as f64 - run.get(&run.before, 3, i) as f64;
        let (bm, be) = (run.vacuum_reset[i] as f64, run.floor_added[i] as f64);
        assert!((bm - dm).abs() <= BOOK_TOL * dm.abs(), "cell {i}: vacuum_reset booked {bm:e}, Σ fell by {dm:e}");
        assert!((be - de).abs() <= BOOK_TOL * de.abs(), "cell {i}: floor_added booked {be:e}, E rose by {de:e}");
        (mass, mass_abs, energy, energy_abs) = (mass + dm, mass_abs + dm.abs(), energy + de, energy_abs + de.abs());
    }
    let total = |name: &str| run.totals.iter().find(|(n, _)| *n == name).unwrap().1;
    let (booked_mass, booked_energy) = (total("mass.vacuum_reset"), total("energy.floor_added"));
    assert!(mass_abs > 0.0 && energy_abs > 0.0, "the field moved no mass or no energy: the case grades nothing");
    assert!((booked_mass - mass).abs() <= BOOK_TOL * mass_abs, "vacuum_reset {booked_mass:e}, Σ fell by {mass:e}");
    assert!((booked_energy - energy).abs() <= BOOK_TOL * energy_abs, "floor_added {booked_energy:e}, E rose by {energy:e}");
    for (name, t) in &run.totals {
        if *name != "mass.vacuum_reset" && *name != "energy.floor_added" {
            assert_eq!(*t, 0.0, "{name}: P8 books only its two terms");
        }
    }
    println!(
        "vacuum_reset {booked_mass:e} (Σ fell by {mass:e}, Σ|ΔΣ| {mass_abs:e}); floor_added {booked_energy:e} (E rose by {energy:e}, Σ|ΔE| {energy_abs:e}); tol {BOOK_TOL:e}"
    );
}
