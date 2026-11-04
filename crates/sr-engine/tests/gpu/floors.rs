//! M0-T19 — P8 complete and the equation of state on the GPU (contract §1.3.2 P8, §1.5, §2.3.2–§2.3.3, §2.7, §2.9):
//! eos.wgsl's helpers against M0-T12's CPU EOS (sr_physics::eos) on a sweep of densities and compositions, with the cold
//! pairs on and off; then a 64 × 64 field of vacuum, cold, dense cold and hot cells, their fractions normalised,
//! unnormalised, of mixed signs or all ≤ 0 → one step → each rule against the f64 twin of P8 below, and the booked
//! `vacuum_reset` and `floor_added` against the mass and energy the step changed.
//!
//! The twin (`p8_twin`) is P8 in f64 from the same f32 inputs: the fractions renormalised (pure hydrogen when none is
//! positive, §2.7 [M0-T19]); μ, Y_e and X_n of those; below Σ_vac the floor state Σ_floor, u = 0,
//! E = Σ_floor (T_floor/μ + ε_cold(Σ_floor)); else Eos::thermal's floor. M0-T37 moved it into sr_physics::reference::floors.
//!
//! Tolerances, fixed before the first run (R8): a value read through the u(x) table — ε_cold, and every E or Π built
//! on it — within 2 × 10⁻⁵ relative: the f32 position in the table carries |ln x − ln X_MIN| ≲ 37 at 2⁻²⁴ each, × 13.9
//! nodes per unit of ln x, ≈ 3 × 10⁻⁵ of a node, × Δ ln u ≤ 0.15 per node ≈ 5 × 10⁻⁶; WGSL's exp of |ln u| ≲ 40 is
//! good to 3 + 2·40 ULP ≈ 5 × 10⁻⁶; ln u stored in f32, 40 × 2⁻²⁴ ≈ 2.4 × 10⁻⁶ — the sum doubled. A value of f32
//! arithmetic alone (1/μ, Y_e, the closed-form P) within 2 × 10⁻⁶ relative; Z_met within 10⁻⁶ absolute (it is a
//! difference). What the floors leave untouched: bit-identical. The booked terms equal the change within 10⁻⁶
//! relative (the block's figure), cell by cell and in total.
//!
//! M0-D71b — P8's hold on the GPU (§2.7 [M0-TG2]; M0-D71's twin, `Floors::hold`): a 64 × 64 field built for it — Σ
//! across σ_hold and ½σ_hold (each edge's f32 neighbours, exact in both), ε_th at multiples of η_hold · ε_cold across
//! the η and 2η edges, iron about Σ_N's gate, half the cells uploaded already held, every cell moving — → one step (the
//! initial state's hold, then step 0's P8) → the mask, the momenta and `held`'s booking against the twin's same sequence
//! from the same f32 start (hold, floors, hold): the mask equal cell for cell, a newly held cell's momentum exactly 0
//! and its E as it was (the kinetic energy left as heat), every other momentum bit-identical, the booked momentum the
//! twin's within 10⁻⁶. A cell whose ε_th lies within 10⁻⁴ of max(E/Σ, ε_cold) of an η or 2η edge, or whose Y_eΣ lies
//! within 10⁻⁵ of Σ_N, at either hold, is within f32's rounding of an edge (ε_cold alone carries 2 × 10⁻⁵, TABLE_TOL):
//! counted, printed and left out. Then a second step over the held state moves nothing; the initial state's hold
//! precedes its Δt (a held cell's speed bounds no Δt_0); Clear leaves no cell held. The legacy field above keeps every
//! cell of Σ ≥ σ_hold at rest, so the hold, whatever it decides there, moves none of the floors' cases. The field's
//! cases run on [`field_constants`]: the shipped ones with the neutrons' cold pressure off (K₁ₙ = K₂ₙ = 0), so m_tov_sb
//! is infinite (f32's largest on the GPU), no disk r ≤ 24 the field makes (≤ 1,809 × 1,030) reaches it and §0.5 [M0-T130]'s
//! gate frees none of its cells — no field cell holds neutrons, so the K's reach nothing else; the gate is held_core's
//! (M0-T130), on the twin and the GPU. (K₂ₙ × 10 is refused by physics.json's K9 bound, m_tov/m_ch ≤ 2.)

use super::device;
use sr_engine::state::{book_totals, Booking, State, WorldConfig, BOOK_GROUPS, CHANNELS, N_CHANNELS, SPECIES, TERMS};
use sr_engine::step::edit::Edit;
use sr_engine::step::{with_eos, EosGpu, Pass, Step};
use sr_physics::eos::Eos;
use sr_physics::reference::floors::{self, Cell, Floors, ETA_HOLD, SIGMA_HOLD};
use sr_physics::reference::hydro::State as TwinState;
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
    floors: Floors,
}

fn shipped() -> Constants {
    constants(Physics::shipped().expect("shipped physics.json"))
}

/// The hold field's constants: the shipped ones with the neutrons' cold pressure off, m_tov_sb infinite — so the disk gate
/// (§0.5 [M0-T130]) frees none of the field's cells, which hold no neutrons (the K's reach nothing else of theirs).
fn field_constants() -> Constants {
    let physics = Physics::shipped().expect("shipped physics.json").with(&[("k1n", 0.0), ("k2n", 0.0)]).expect("K₁ₙ = K₂ₙ = 0");
    let c = constants(physics);
    assert!(c.floors.m_tov().is_infinite(), "m_tov_sb {} with the neutrons' cold pressure off", c.floors.m_tov());
    assert!(HOLD_X.iter().all(|x| x[9] == 0.0), "a hold field mix holds neutrons: the K's would reach its ε_cold");
    c
}

fn constants(physics: Physics) -> Constants {
    let eos = Eos::from_physics(&physics).expect("shipped constants");
    let elements = Elements::shipped().expect("shipped elements.json");
    let floors = Floors::new(&physics, &elements).expect("shipped constants");
    Constants { physics, elements, eos, floors }
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
    let floors = Floors::new(&physics, &on.elements).unwrap();
    let off = Constants { eos: Eos::from_physics(&physics).unwrap(), physics, elements: on.elements, floors };
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

/// The fractions as P8 leaves them, in f64 (reference::floors): clamped ≥ 0 and divided by their sum; pure hydrogen when
/// none is positive.
fn renormalised(x: &[f32]) -> [f64; N_SPECIES] {
    floors::renormalised(&std::array::from_fn(|k| f64::from(x[k])))
}

/// One cell after P8, in f64, and what P8 books for it.
type Twin = Cell;

/// P8's f64 twin (sr_physics::reference::floors, moved there by M0-T37) for one cell of the canonical planes.
fn p8_twin(c: &Constants, planes: &[f32], cells: usize, i: usize) -> Twin {
    let at = |ch: usize| f64::from(planes[ch * cells + i]);
    let raw: [f64; N_SPECIES] = std::array::from_fn(|k| at(SPECIES.start + k));
    c.floors.cell(at(0), [at(1), at(2)], at(3), raw)
}

/// 64 × 64 cells: the kinds by `Kind::of`, the fractions by (cell / 8) mod 4 — normalised (rich in H and n),
/// unnormalised, mixed signs, all ≤ 0. Cold cells sit at ε_th = f · T_floor/μ with f ∈ [−3, 0.3] (dense: [−4, −1], so
/// the f32 ε_cold of Σ up to 60 cannot flip the floor's decision), hot ones at f ∈ [10, 10⁴]. A cell of Σ ≥ σ_hold is
/// at rest (M0-D71b): the hold may take it, and then zeroes no momentum the floors' cases read.
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
        let mut v = [4.0 * seq.next() - 2.0, 4.0 * seq.next() - 2.0];
        let (sigma, f) = match Kind::of(i) {
            Kind::Vacuum => (VACUUM_SIGMA[(i / 32) % 8], 0.0),
            Kind::Cold => (0.1 * 100f64.powf(seq.next()), -3.0 + 3.3 * seq.next()),
            Kind::ColdDense => (20.0 + 40.0 * seq.next(), -4.0 + 3.0 * seq.next()),
            Kind::Hot => (1e-5 * 1e7f64.powf(seq.next()), 10.0 + 1e4 * seq.next()),
        };
        let sigma = sigma as f32 as f64;
        if sigma >= SIGMA_HOLD {
            v = [0.0, 0.0];
        }
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
    let world = WorldConfig::new(64, 64).unwrap();
    run_p8_on(c, world, field(c, world))
}

fn run_p8_on(c: &Constants, world: WorldConfig, before: Vec<f32>) -> Run {
    let gpu = device();
    let state = State::new(&gpu.device, world).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &before).unwrap();
    let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
    Step::new(&gpu.device, &state, &booking, &eos, &c.physics).run(&gpu.device, &gpu.queue, 1);
    let after = state.readback(&gpu.device, &gpu.queue);
    let book = booking.readback(&gpu.device, &gpu.queue);
    let totals = book_totals(world, &book);
    let plane = |name: &str| term_plane(world, &book, name);
    Run {
        world,
        vacuum_reset: plane("mass.vacuum_reset"),
        floor_added: plane("energy.floor_added"),
        totals: TERMS.iter().map(|t| t.name).zip(totals).collect(),
        before,
        after,
    }
}

/// The plane of a booked term in a [`Booking::readback`]: its group's offset in the readback, then its index in the group.
fn term_plane(world: WorldConfig, book: &[f32], name: &str) -> Vec<f32> {
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
    let step = Step::new(&gpu.device, &state, &booking, &EosGpu::new(&gpu.device, &c.physics, &c.elements), &c.physics);
    assert_eq!(step.dispatch_labels(Pass::Floors), ["floor_cells", "renormalise_species"]);
    // P1 and P9 hold the Δt and the guard (M0-T20, tests/gpu/dt.rs); every other pass is still empty.
    for pass in Pass::ORDER.into_iter().filter(|&p| ![Pass::Dt, Pass::Floors, Pass::Reductions].contains(&p)) {
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
fn a_cell_with_a_non_finite_fraction_is_left_for_the_guard() {
    // M0-D18: a NaN x_C in a vacuum, a cold and a hot cell, an Inf x_Fe and a −Inf x_n elsewhere → after one step each
    // such cell is bit-identical — its fractions still non-finite for P9's guard, no floor, nothing booked — and every
    // other cell is what the clean field's run leaves.
    let c = shipped();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let clean = run_p8(&c);
    let species = |name: &str| CHANNELS.iter().position(|&n| n == name).unwrap();
    let (x_c, x_fe, x_n) = (species("x_C"), species("x_Fe"), species("x_n"));
    let planted = [
        (8 * 40, x_c, f32::NAN),
        (8 * 41 + 2, x_c, f32::NAN),
        (8 * 42 + 5, x_c, f32::NAN),
        (8 * 43 + 4, x_fe, f32::INFINITY),
        (8 * 44 + 7, x_n, f32::NEG_INFINITY),
    ];
    let kinds: Vec<Kind> = planted.iter().map(|&(i, _, _)| Kind::of(i)).collect();
    assert_eq!(kinds, [Kind::Vacuum, Kind::Cold, Kind::Hot, Kind::ColdDense, Kind::Cold], "the planted cells' kinds");
    let mut before = field(&c, world);
    for &(i, ch, v) in &planted {
        before[ch * cells + i] = v;
    }
    let run = run_p8_on(&c, world, before);
    for &(i, ch, _) in &planted {
        let x = run.get(&run.after, ch, i);
        assert!(!x.is_finite(), "cell {i} ({:?}): {} {x} after P8, the guard would never see it", Kind::of(i), CHANNELS[ch]);
        for k in 0..N_CHANNELS {
            let (b, a) = (run.get(&run.before, k, i), run.get(&run.after, k, i));
            assert_eq!(a.to_bits(), b.to_bits(), "cell {i}: {} {b} → {a}, P8 touched a non-finite cell", CHANNELS[k]);
        }
        assert_eq!((run.vacuum_reset[i], run.floor_added[i]), (0.0, 0.0), "cell {i}: P8 booked a non-finite cell");
    }
    let mut others = 0;
    for i in (0..cells).filter(|i| !planted.iter().any(|p| p.0 == *i)) {
        for k in 0..N_CHANNELS {
            let (want, got) = (clean.get(&clean.after, k, i), run.get(&run.after, k, i));
            assert_eq!(got.to_bits(), want.to_bits(), "cell {i}: {} {got}, the clean run's {want}", CHANNELS[k]);
        }
        others += 1;
    }
    println!("{} planted cells left as read (non-finite fractions kept, nothing booked); {others} other cells as the clean run", planted.len());
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

// ---------------------------------------------------------------------------------------------------------------
// P8's hold (M0-D71b), against the twin's `Floors::hold`

/// The edge band for ε_th at η and 2η, relative to max(E/Σ, ε_cold): ε_cold's table error (TABLE_TOL) and f32's E/Σ
/// and subtraction, five times over.
const EDGE_EPS: f64 = 1e-4;
/// The edge band for Y_eΣ about Σ_N, relative: Y_e's f32 arithmetic (ARITH_TOL) and Σ_N rounded to f32, five times over.
const EDGE_GATE: f64 = 1e-5;

/// The hold field's compositions, by cell mod 4: a solar-like mix, carbon–oxygen, iron with silicon and sulphur, iron.
const HOLD_X: [[f64; N_SPECIES]; 4] = [
    [0.7346, 0.2485, 0.0077, 0.0092, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.02, 0.48, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.3, 0.2, 0.5, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
];
/// ε_th / (η_hold · ε_cold) by row mod 16: across the η edge and the 2η edge, exactly on each (left out as within
/// rounding), and at or below 0 (the floor raises it, then the hold reads it).
const HOLD_F: [f64; 16] = [-1.0, 0.0, 0.3, 0.9, 0.99, 1.0, 1.01, 1.1, 1.5, 1.9, 1.99, 2.0, 2.01, 2.1, 3.0, 10.0];

fn f32_below(v: f32) -> f32 {
    f32::from_bits(v.to_bits() - 1)
}

fn f32_above(v: f32) -> f32 {
    f32::from_bits(v.to_bits() + 1)
}

/// One cell of the hold field: its composition, Σ, ε_th / (η_hold ε_cold), whether it is uploaded held, and whether it is
/// built on an edge (ε_th at η or 2η exactly, or Y_eΣ at Σ_N).
#[derive(Clone, Copy)]
struct HoldCell {
    class: usize,
    sigma: f32,
    f: f64,
    held: bool,
    on_edge: bool,
}

impl HoldCell {
    /// Whether N_Fe's gate is open: iron present and Y_eΣ > Σ_N.
    fn gated(&self, c: &Constants) -> bool {
        let x = HOLD_X[self.class];
        x[8] > 0.0 && c.elements.composition(&x).y_e * f64::from(self.sigma) > c.physics.get("sigma_n").unwrap()
    }
}

/// 64 × 64 cells: the composition by x mod 4, Σ by (x / 4) mod 16 — for the iron-free mixes across ½σ_hold and σ_hold
/// (each edge and its f32 neighbours) up to 1,000, for the iron mixes Σ_N / Y_e times 0.5 to 3 (0.9999, 1 and 1.0001 of
/// it among them) beside a few below σ_hold —, ε_th by row mod 16 ([`HOLD_F`]), held as uploaded in rows 16–31 and 48–63;
/// every cell moving at up to 0.5 on each axis, its kinetic energy in E.
fn hold_field(c: &Constants, world: WorldConfig) -> (Vec<f32>, Vec<u32>, Vec<HoldCell>) {
    let cells = world.cells();
    let w = world.width as usize;
    let sigma_n = c.physics.get("sigma_n").unwrap();
    let free: [f32; 16] = [
        4.0,
        f32_below(5.0),
        5.0,
        f32_above(5.0),
        7.5,
        f32_below(10.0),
        10.0,
        f32_above(10.0),
        12.0,
        20.0,
        40.0,
        80.0,
        150.0,
        300.0,
        600.0,
        1000.0,
    ];
    const IRON: [f64; 16] = [0.0, 0.0, 0.0, 0.0, 0.5, 0.9, 0.99, 0.9999, 1.0, 1.0001, 1.01, 1.1, 1.5, 2.0, 3.0, 0.0];
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut mask = vec![0u32; cells];
    let mut params = Vec::with_capacity(cells);
    let mut seq = Seq(0x5eed_d71b);
    for i in 0..cells {
        let (x, y) = (i % w, i / w);
        let class = x % 4;
        let k = (x / 4) % 16;
        let comp_x = HOLD_X[class];
        let comp = c.elements.composition(&comp_x);
        let sigma = match (class, k) {
            (0 | 1, _) => free[k],
            (_, 0) => f32_below(5.0),
            (_, 1) => 10.0,
            (_, 2) => 20.0,
            (_, 3) => 100.0,
            (_, 15) => 4.0,
            _ => (IRON[k] * sigma_n / comp.y_e) as f32,
        };
        let f = HOLD_F[y % 16];
        let held = (y / 16) % 2 == 1;
        let s = f64::from(sigma);
        let cold = c.eos.eps_cold(s, comp.y_e, comp_x[9]);
        let v = [seq.next() - 0.5, seq.next() - 0.5];
        let energy = s * (cold + f * ETA_HOLD * cold) + 0.5 * s * (v[0] * v[0] + v[1] * v[1]);
        planes[i] = sigma;
        planes[cells + i] = (s * v[0]) as f32;
        planes[2 * cells + i] = (s * v[1]) as f32;
        planes[3 * cells + i] = energy as f32;
        for (k, xk) in comp_x.iter().enumerate() {
            planes[(SPECIES.start + k) * cells + i] = *xk as f32;
        }
        mask[i] = u32::from(held);
        let on_edge = f == 1.0 || f == 2.0 || (class >= 2 && IRON[k] == 1.0);
        params.push(HoldCell { class, sigma, f, held, on_edge });
    }
    (planes, mask, params)
}

/// The GPU's side of a hold case: a state, its booking and a step (P1, P8, P9; P0 too when built with edits).
struct HoldGpu {
    gpu: sr_engine::gpu::Gpu,
    world: WorldConfig,
    state: State,
    booking: Booking,
    step: Step,
}

impl HoldGpu {
    fn new(c: &Constants, world: WorldConfig, planes: &[f32], mask: &[u32], edits: bool) -> HoldGpu {
        let gpu = device();
        let state = State::new(&gpu.device, world).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, planes).unwrap();
        state.upload_held(&gpu.queue, mask).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let mut step = Step::new(&gpu.device, &state, &booking, &eos, &c.physics);
        if edits {
            step = step.with_edits(&gpu.device, &state, &booking, &eos, &c.physics);
        }
        HoldGpu { gpu, world, state, booking, step }
    }

    fn run(&self, steps: u32) {
        self.step.run(&self.gpu.device, &self.gpu.queue, steps);
    }

    fn planes(&self) -> Vec<f32> {
        self.state.readback(&self.gpu.device, &self.gpu.queue)
    }

    fn mask(&self) -> Vec<u32> {
        self.state.read_held(&self.gpu.device, &self.gpu.queue)
    }

    /// `held.mom_x` and `held.mom_y`, one slot per cell.
    fn held_booked(&self) -> [Vec<f32>; 2] {
        let book = self.booking.readback(&self.gpu.device, &self.gpu.queue);
        ["held.mom_x", "held.mom_y"].map(|name| term_plane(self.world, &book, name))
    }

    fn clear_booking(&self) {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("clear") });
        self.booking.clear(&mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
    }
}

/// The twin's side: the initial state's hold, then step 0's P8 (`Floors::apply`, then `Floors::hold`), in f64 from the
/// same f32 start — the mask after, the momentum each cell had when newly held (summed over both holds), and whether a
/// cell sat within f32's rounding of an edge at either hold.
struct TwinHold {
    mask: Vec<bool>,
    taken: Vec<[f64; 2]>,
    edge: Vec<bool>,
    /// The two holds' sums, as `Floors::hold` returns them.
    sums: [f64; 2],
}

fn twin_hold(c: &Constants, world: WorldConfig, before: &[f32], mask0: &[u32]) -> TwinHold {
    let cells = world.cells();
    let mut st = TwinState::new(world.width as usize, world.height as usize, before.iter().map(|&v| f64::from(v)).collect());
    let mut mask: Vec<bool> = mask0.iter().map(|&m| m != 0).collect();
    let mut taken = vec![[0.0; 2]; cells];
    let mut edge = vec![false; cells];
    let mut sums = [0.0; 2];
    let sigma_n = c.physics.get("sigma_n").unwrap();
    for pass in 0..2 {
        if pass == 1 {
            c.floors.apply(&mut st);
        }
        let p = &st.planes;
        for (i, e) in edge.iter_mut().enumerate() {
            let x = floors::renormalised(&std::array::from_fn(|k| p[(SPECIES.start + k) * cells + i]));
            let comp = c.elements.composition(&x);
            let (sigma, energy) = (p[i], p[3 * cells + i]);
            let cold = c.eos.eps_cold(sigma, comp.y_e, x[9]);
            let (mx, my) = (p[cells + i], p[2 * cells + i]);
            let eps_th = energy / sigma - 0.5 * (mx * mx + my * my) / (sigma * sigma) - cold;
            let band = EDGE_EPS * (energy / sigma).abs().max(cold);
            let at_eta = [1.0, 2.0].iter().any(|k| (eps_th - k * ETA_HOLD * cold).abs() <= band);
            let at_gate = x[8] > 0.0 && (comp.y_e * sigma - sigma_n).abs() <= EDGE_GATE * sigma_n;
            *e |= at_eta || at_gate;
        }
        let mom: Vec<[f64; 2]> = (0..cells).map(|i| [st.planes[cells + i], st.planes[2 * cells + i]]).collect();
        let was = mask.clone();
        let sum = c.floors.hold(&mut st, &mut mask);
        sums = [sums[0] + sum[0], sums[1] + sum[1]];
        for i in (0..cells).filter(|&i| mask[i] && !was[i]) {
            taken[i] = [taken[i][0] + mom[i][0], taken[i][1] + mom[i][1]];
        }
    }
    TwinHold { mask, taken, edge, sums }
}

#[test]
fn the_hold_mask_and_momenta_equal_the_twins_cell_for_cell() {
    let c = field_constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let (before, mask0, params) = hold_field(&c, world);
    let gpu = HoldGpu::new(&c, world, &before, &mask0, false);
    gpu.run(1);
    let (after, mask) = (gpu.planes(), gpu.mask());
    let twin = twin_hold(&c, world, &before, &mask0);
    let edges = twin.edge.iter().filter(|&&e| e).count();
    // Outcomes among the graded cells: [newly held, kept in the hysteresis band, released past 2η, released under
    // ½σ_hold, released or refused at N_Fe's gate, refused under σ_hold, refused in the band].
    let mut seen = [0usize; 7];
    for i in (0..cells).filter(|&i| !twin.edge[i]) {
        let p = params[i];
        assert_eq!(mask[i], u32::from(twin.mask[i]), "cell {i} (class {}, Σ {}, f {}, held {}): the mask", p.class, p.sigma, p.f, p.held);
        let newly = twin.mask[i] && !p.held;
        for ch in [1, 2] {
            let (b, a) = (before[ch * cells + i], after[ch * cells + i]);
            if newly {
                assert_eq!(a.to_bits(), 0f32.to_bits(), "cell {i}: newly held, {} {a}", CHANNELS[ch]);
            } else {
                assert_eq!(a.to_bits(), b.to_bits(), "cell {i}: not newly held, {} {b} → {a}", CHANNELS[ch]);
            }
        }
        if p.f > 0.0 {
            let (b, a) = (before[3 * cells + i], after[3 * cells + i]);
            assert_eq!(a.to_bits(), b.to_bits(), "cell {i}: E {b} → {a} (above the floor; the hold leaves E)");
        }
        let category = match (p.held, twin.mask[i]) {
            (false, true) => Some(0),
            (true, true) if p.f > 1.01 && p.f < 1.99 => Some(1),
            (true, false) if p.gated(&c) => Some(4),
            (true, false) if p.sigma < 5.0 => Some(3),
            (true, false) if p.f > 2.0 => Some(2),
            (false, false) if p.gated(&c) && p.f < 1.0 && p.sigma >= 10.0 => Some(4),
            (false, false) if p.sigma < 10.0 && p.f < 1.0 => Some(5),
            (false, false) if p.f > 1.01 && p.f < 1.99 => Some(6),
            _ => None,
        };
        if let Some(k) = category {
            seen[k] += 1;
        }
    }
    let names = ["newly held", "kept (η < ε_th/ε_cold ≤ 2η)", "released past 2η", "released under ½σ_hold", "at N_Fe's gate", "refused under σ_hold", "refused in the band"];
    let line: Vec<String> = names.iter().zip(seen).map(|(n, k)| format!("{n} {k}")).collect();
    let built = params.iter().filter(|p| p.on_edge).count();
    println!("{} cells graded, {edges} within f32's rounding of an edge left out ({built} built on one); {}", cells - edges, line.join(", "));
    for (n, k) in names.iter().zip(seen) {
        assert!(k > 0, "no graded cell {n}: the case grades nothing there");
    }
    // Every cell built on an edge is within rounding of it; any other is a rare f32 coincidence.
    assert!((0..cells).all(|i| !params[i].on_edge || twin.edge[i]), "a cell built on an edge was graded");
    let built = params.iter().filter(|p| p.on_edge).count();
    assert!(edges - built <= cells / 100, "{edges} cells at an edge, {built} built there");
}

#[test]
fn the_held_momentum_is_booked_as_the_twin_books_it() {
    let c = field_constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let (before, mask0, _) = hold_field(&c, world);
    let gpu = HoldGpu::new(&c, world, &before, &mask0, false);
    gpu.run(1);
    let booked = gpu.held_booked();
    let twin = twin_hold(&c, world, &before, &mask0);
    let (mut got, mut want, mut scale) = ([0f64; 2], [0f64; 2], 0f64);
    for i in (0..cells).filter(|&i| !twin.edge[i]) {
        for a in 0..2 {
            let (g, w) = (f64::from(booked[a][i]), twin.taken[i][a]);
            assert!((g - w).abs() <= BOOK_TOL * w.abs(), "cell {i}: held.mom_{} booked {g:e}, the twin's {w:e}", ["x", "y"][a]);
            got[a] += g;
            want[a] += w;
            scale += w.abs();
        }
    }
    assert!(scale > 0.0, "no momentum taken: the case grades nothing");
    for a in 0..2 {
        assert!((got[a] - want[a]).abs() <= BOOK_TOL * scale, "held.mom_{}: {:e}, the twin's {:e}", ["x", "y"][a], got[a], want[a]);
    }
    let all = [0, 1].map(|a| twin.taken.iter().map(|t| t[a]).sum::<f64>());
    assert!((0..2).all(|a| (all[a] - twin.sums[a]).abs() <= 1e-12 * scale), "the per-cell taken sum to Floors::hold's");
    println!("held booked ({:e}, {:e}) over the graded cells, the twin's ({:e}, {:e}); Σ|taken| {scale:e}", got[0], got[1], want[0], want[1]);
}

#[test]
fn a_second_p8_over_the_held_state_moves_nothing() {
    let c = field_constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let (before, mask0, _) = hold_field(&c, world);
    let gpu = HoldGpu::new(&c, world, &before, &mask0, false);
    gpu.run(1);
    let (first, mask1) = (gpu.planes(), gpu.mask());
    gpu.clear_booking();
    gpu.run(1);
    let (second, mask2) = (gpu.planes(), gpu.mask());
    let booked = gpu.held_booked();
    let held = mask1.iter().filter(|&&m| m != 0).count();
    assert!(held > 0 && held < cells, "{held} cells held after the first step");
    for i in 0..cells {
        assert_eq!(mask2[i], mask1[i], "cell {i}: the second P8 changed the mask");
        for ch in [1, 2] {
            assert_eq!(second[ch * cells + i].to_bits(), first[ch * cells + i].to_bits(), "cell {i}: {} moved", CHANNELS[ch]);
        }
        assert_eq!((booked[0][i], booked[1][i]), (0.0, 0.0), "cell {i}: the second P8 booked held momentum");
    }
    println!("{held} cells held; the second step's P8 kept the mask, the momenta and booked 0 on all {cells} cells");
}

#[test]
fn the_initial_states_hold_comes_before_its_dt() {
    // Warm gas at rest, and a 4 × 4 block of dense cold gas moving at 200, its own sound speed ≈ 25: the initial state's
    // hold zeroes the block's momentum (its kinetic energy left as heat) before P9's reduction, so Δt_0 is the Δt of the
    // same state with those momenta zeroed by hand, bit for bit — and not the Δt of the block uploaded already held,
    // whose momentum no hold zeroes (r_h = |u| + |v| + 2c: ≈ 250 moving and cold, ≈ 400 at rest with its heat).
    let c = shipped();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let x = HOLD_X[0];
    let comp = c.elements.composition(&x);
    let mut moving = vec![0f32; N_CHANNELS * cells];
    let block = |i: usize| (30..34).contains(&(i % 64)) && (30..34).contains(&(i / 64));
    for i in 0..cells {
        let (sigma, eps_th, v) = if block(i) { (50.0, None, 200.0) } else { (1.0, Some(1.0), 0.0) };
        let cold = c.eos.eps_cold(sigma, comp.y_e, x[9]);
        let eps_th = eps_th.unwrap_or(0.5 * ETA_HOLD * cold);
        moving[i] = sigma as f32;
        moving[cells + i] = (sigma * v) as f32;
        moving[3 * cells + i] = (sigma * (cold + eps_th) + 0.5 * sigma * v * v) as f32;
        for (k, xk) in x.iter().enumerate() {
            moving[(SPECIES.start + k) * cells + i] = *xk as f32;
        }
    }
    let mut zeroed = moving.clone();
    zeroed[cells..2 * cells].fill(0.0);
    let none = vec![0u32; cells];
    let pre_held: Vec<u32> = (0..cells).map(|i| u32::from(block(i))).collect();
    let dt = |planes: &[f32], mask: &[u32]| {
        let gpu = HoldGpu::new(&c, world, planes, mask, false);
        gpu.run(1);
        (gpu.step.read_dt(&gpu.gpu.device, &gpu.gpu.queue).dt, gpu.mask())
    };
    let (dt_moving, mask) = dt(&moving, &none);
    let (dt_zeroed, _) = dt(&zeroed, &none);
    let (dt_kept, _) = dt(&moving, &pre_held);
    println!("Δt_0: moving block {dt_moving:e}, its momenta zeroed by hand {dt_zeroed:e}, uploaded held (kept moving) {dt_kept:e}");
    let twin = twin_hold(&c, world, &moving, &none);
    assert!((0..cells).filter(|&i| block(i)).all(|i| twin.taken[i][0] > 0.0 && !twin.edge[i]), "the twin holds the block at once");
    assert_eq!(mask, twin.mask.iter().map(|&m| u32::from(m)).collect::<Vec<_>>(), "the mask after step 0");
    assert_eq!(dt_moving.to_bits(), dt_zeroed.to_bits(), "Δt_0 read the block's momentum: the hold came after the reduction");
    assert!((dt_kept / dt_moving - 1.0).abs() > 0.05, "the block's momentum moves no Δt: the case cannot tell the order");
}

#[test]
fn clear_leaves_no_cell_held() {
    let c = field_constants();
    let world = WorldConfig::new(64, 64).unwrap();
    let (before, mask0, _) = hold_field(&c, world);
    let gpu = HoldGpu::new(&c, world, &before, &mask0, true);
    gpu.run(1);
    let held = gpu.mask().iter().filter(|&&m| m != 0).count();
    assert!(held > 0, "no cell held before the Clear: the case grades nothing");
    gpu.step.queue_edit(&gpu.gpu.queue, &Edit::Clear).unwrap();
    gpu.run(1);
    let left = gpu.mask().iter().filter(|&&m| m != 0).count();
    assert_eq!(left, 0, "{left} cells still held after the Clear");
    println!("{held} cells held, then a Clear: none held after its step");
}
