//! M0-T3 — the cell state and the step loop (contract §2.2, §2.8, §1.3.2, §1.3.4): the canonical upload/readback
//! round trip, P0–P9's order, and P8's species renormalisation as the first pass (P8's floors: tests/gpu/floors.rs).

use super::device;
use sr_engine::gpu::Gpu;
use sr_engine::state::{Booking, InspectReply, Inspector, State, StateError, WorldConfig, CHANNELS, N_CHANNELS, SPECIES};
use sr_engine::step::{EosGpu, Pass, Step};
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

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

/// The step of `state` with the shipped constants, booking into a booking layout of its own.
fn step(gpu: &Gpu, state: &State) -> Step {
    let booking = Booking::new(&gpu.device, state.world()).unwrap();
    let physics = Physics::shipped().unwrap();
    let eos = EosGpu::new(&gpu.device, &physics, &Elements::shipped().unwrap());
    Step::new(&gpu.device, state, &booking, &eos, &physics)
}

/// 64 × 64 cells: channels 0–3 a distinct, exact value per cell — hot gas, Σ in [1, 2) and E/Σ ≥ 500, far above P8's
/// floors (M0-T19), so they leave it; the fractions unnormalised, some negative, and the edge cases on a fixed
/// pattern — all ≤ 0, all zero, only x_n positive.
fn unnormalised_field(world: WorldConfig) -> Vec<f32> {
    let cells = world.cells();
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut seq = Seq(0x5eed_0003);
    for i in 0..cells {
        let k = i as f32;
        planes[i] = 1.0 + k / 4096.0;
        planes[cells + i] = k / 1024.0 - 2.0;
        planes[2 * cells + i] = 2.0 - k / 2048.0;
        planes[3 * cells + i] = 1000.0 + k * 0.25;
    }
    for i in 0..cells {
        for (k, c) in SPECIES.enumerate() {
            let v = match i % 8 {
                0 => 3.0 * seq.next(),               // positive, unnormalised
                1 => 3.0 * seq.next() - 1.0,         // mixed signs
                2 => -seq.next() - 1e-3,             // all negative → pure hydrogen
                3 => 0.0,                            // all zero → pure hydrogen
                4 if k == SPECIES.len() - 1 => 0.7,  // only x_n positive → x_n = 1
                4 => -0.25,
                5 => 1e-3 * seq.next(),              // tiny, sum ≪ 1
                _ => 2.5 * seq.next() - 0.5,
            };
            planes[c * cells + i] = v as f32;
        }
    }
    planes
}

/// The renormalisation in f64, the oracle for one cell's fractions: clamp ≥ 0, divide by the sum; pure hydrogen
/// when every fraction is ≤ 0 (M0-T3's declared default).
fn renormalised(x: &[f32]) -> Vec<f64> {
    let clamped: Vec<f64> = x.iter().map(|&v| (v as f64).max(0.0)).collect();
    let sum: f64 = clamped.iter().sum();
    if sum > 0.0 {
        clamped.iter().map(|v| v / sum).collect()
    } else {
        let mut h = vec![0.0; x.len()];
        h[0] = 1.0;
        h
    }
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn world_config_takes_multiples_of_8_within_64_to_2048() {
    assert_eq!(WorldConfig::default(), WorldConfig { width: 600, height: 400 });
    assert!(WorldConfig::default().validate().is_ok());
    for (w, h) in [(64, 64), (2048, 2048), (600, 400), (72, 2040)] {
        assert_eq!(WorldConfig::new(w, h).map(|c| c.cells()), Ok(w as usize * h as usize), "{w} × {h}");
    }
    for (w, h) in [(56, 64), (64, 2056), (601, 400), (600, 402), (0, 0), (4096, 64)] {
        assert_eq!(WorldConfig::new(w, h), Err(StateError::WorldSize { width: w, height: h }), "{w} × {h}");
    }
    assert_eq!(CHANNELS.len(), 14);
    assert_eq!(&CHANNELS[SPECIES], &["x_H", "x_He", "x_C", "x_O", "x_Ne", "x_Mg", "x_Si", "x_S", "x_Fe", "x_n"]);
}

#[test]
fn round_trip_600x400_is_bit_exact_with_a_distinct_value_per_channel_and_cell() {
    let gpu = device();
    let world = WorldConfig::default();
    let cells = world.cells();
    // Every value an exact, distinct f32 (|k| < 2^24), signs mixed: a channel or cell moved anywhere shows.
    let planes: Vec<f32> = (0..N_CHANNELS * cells)
        .map(|k| if k % 3 == 0 { -((k + 1) as f32) } else { (k + 1) as f32 })
        .collect();
    let state = State::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &planes).unwrap();
    let back = state.readback(&gpu.device, &gpu.queue);
    assert_eq!(back.len(), planes.len());
    if let Some(k) = (0..planes.len()).find(|&k| back[k].to_bits() != planes[k].to_bits()) {
        panic!(
            "first difference: channel {} cell {}: uploaded {}, read back {}",
            CHANNELS[k / cells],
            k % cells,
            planes[k],
            back[k]
        );
    }
    println!("round trip {} × {}: {} values bit-exact", world.width, world.height, planes.len());
}

#[test]
fn upload_refuses_anything_but_14_planes_of_the_world() {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let state = State::new(&gpu.device, world).unwrap();
    let expected = N_CHANNELS * world.cells();
    for got in [0, expected - 1, expected + 1, 13 * world.cells()] {
        assert_eq!(
            state.upload(&gpu.queue, &vec![0.0; got]),
            Err(StateError::PlanesLength { expected, got }),
            "{got} values"
        );
    }
    assert!(State::new(&gpu.device, WorldConfig { width: 60, height: 64 }).is_err(), "a world off §2.8's sizes");
}

#[test]
fn a_step_is_p0_to_p9_with_p1s_dt_p8s_floors_and_renormalisation_and_p9s_dt_reduction_its_only_dispatches_today() {
    let labels: Vec<&str> = Pass::ORDER.iter().map(|p| p.label()).collect();
    for (i, label) in labels.iter().enumerate() {
        assert!(label.starts_with(&format!("P{i} ")), "{labels:?}");
    }
    let gpu = device();
    let state = State::new(&gpu.device, WorldConfig::new(64, 64).unwrap()).unwrap();
    let step = step(&gpu, &state);
    let mut expected = [0usize; 10];
    expected[Pass::Dt as usize] = 1;
    expected[Pass::Floors as usize] = 2;
    expected[Pass::Reductions as usize] = 2;
    assert_eq!(step.dispatch_counts(), expected);
}

#[test]
fn one_step_renormalises_the_species_and_leaves_the_other_channels_bit_identical() {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let before = unnormalised_field(world);
    let state = State::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &before).unwrap();
    step(&gpu, &state).run(&gpu.device, &gpu.queue, 1);
    let after = state.readback(&gpu.device, &gpu.queue);

    let other = 0..SPECIES.start * cells;
    assert_eq!(bits(&after[other.clone()]), bits(&before[other]), "channels sigma … energy changed");

    let (mut worst_sum, mut worst_x) = (0f64, 0f64);
    for i in 0..cells {
        let x_before: Vec<f32> = SPECIES.map(|c| before[c * cells + i]).collect();
        let x_after: Vec<f32> = SPECIES.map(|c| after[c * cells + i]).collect();
        let oracle = renormalised(&x_before);
        let sum: f64 = x_after.iter().map(|&v| v as f64).sum();
        assert!(x_after.iter().all(|&v| v >= 0.0), "cell {i}: a negative fraction {x_after:?}");
        assert!((sum - 1.0).abs() <= 1e-6, "cell {i}: Σ X = {sum} (from {x_before:?}, now {x_after:?})");
        for (k, (&got, want)) in x_after.iter().zip(&oracle).enumerate() {
            let err = (got as f64 - want).abs();
            assert!(err <= 1e-6, "cell {i}, {}: {got}, the f64 oracle {want}", CHANNELS[SPECIES.start + k]);
            worst_x = worst_x.max(err);
        }
        worst_sum = worst_sum.max((sum - 1.0).abs());
    }
    println!("{cells} cells: worst |Σ X − 1| = {worst_sum:e}, worst |X − oracle| = {worst_x:e} (tolerance 1e-6)");
}

#[test]
fn two_runs_from_the_same_state_are_bit_identical() {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let start = unnormalised_field(world);
    let run = || {
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &start).unwrap();
        step(&gpu, &state).run(&gpu.device, &gpu.queue, 3);
        state.readback(&gpu.device, &gpu.queue)
    };
    let (a, b) = (run(), run());
    assert_eq!(bits(&a), bits(&b), "two runs of 3 steps differ");
    assert_ne!(bits(&a), bits(&start), "the step changed nothing: the case would pass with no pass at all");
}

/// A 64 × 64 state of distinct, valid cells: Σ in [1, 2), a velocity per cell, E/Σ well above the floor, and a
/// two-species mix (hydrogen / helium) that changes from cell to cell.
fn inspectable(world: WorldConfig) -> Vec<f32> {
    let cells = world.cells();
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut seq = Seq(0x1234_5678_9abc_def1);
    for i in 0..cells {
        let sigma = 1.0 + seq.next();
        let (ux, uy) = (seq.next() - 0.5, seq.next() - 0.5);
        let h = 0.2 + 0.6 * seq.next();
        planes[i] = sigma as f32;
        planes[cells + i] = (sigma * ux) as f32;
        planes[2 * cells + i] = (sigma * uy) as f32;
        planes[3 * cells + i] = (sigma * (600.0 + 400.0 * seq.next() + 0.5 * (ux * ux + uy * uy))) as f32;
        planes[SPECIES.start * cells + i] = h as f32;
        planes[(SPECIES.start + 1) * cells + i] = (1.0 - h) as f32;
    }
    planes
}

/// What the dump's cell says, graded in f64 from the canonical planes: Σ, u, T = μ ε_th and X.
fn from_dump(planes: &[f32], cells: usize, i: usize, eos: &Eos, elements: &Elements) -> (f32, [f32; 2], f64, [f32; N_SPECIES]) {
    let at = |c: usize| planes[c * cells + i];
    let x: [f32; N_SPECIES] = std::array::from_fn(|k| at(SPECIES.start + k));
    let xf: [f64; N_SPECIES] = std::array::from_fn(|k| f64::from(x[k]));
    let comp = elements.composition(&xf);
    let mu = 1.0 / comp.inv_mu;
    let (s, mom) = (f64::from(at(0)), [f64::from(at(1)), f64::from(at(2))]);
    let th = eos.thermal(f64::from(at(3)), s, mom, comp.y_e, xf[N_SPECIES - 1], mu);
    (at(0), [(mom[0] / s) as f32, (mom[1] / s) as f32], Eos::temperature(th.eps_th, mu), x)
}

struct Probe {
    gpu: Gpu,
    world: WorldConfig,
    state: State,
    inspector: Inspector,
    planes: Vec<f32>,
    eos: Eos,
    elements: Elements,
}

fn probe(planes: Option<Vec<f32>>) -> Probe {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let state = State::new(&gpu.device, world).unwrap();
    let planes = planes.unwrap_or_else(|| inspectable(world));
    state.upload(&gpu.queue, &planes).unwrap();
    let inspector = Inspector::new(world, &physics, &elements);
    Probe { gpu, world, state, inspector, planes, eos: Eos::from_physics(&physics).unwrap(), elements }
}

impl Probe {
    fn ask(&mut self, cell: (u32, u32)) {
        self.inspector.inspect(&self.gpu.device, &self.gpu.queue, &self.state, cell).unwrap();
    }

    /// Frames of 16 ms, each a `poll()` that never waits; the replies and the frame that delivered them (1-based).
    fn frames_until(&mut self, count: usize, limit: usize) -> (Vec<InspectReply>, usize) {
        let mut got = Vec::new();
        for frame in 1..=limit {
            got.extend(self.inspector.poll(&self.gpu.device));
            if got.len() >= count {
                return (got, frame);
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        (got, limit + 1)
    }
}

#[test]
fn inspect_replies_with_the_dumps_cell_sigma_u_t_and_x() {
    let mut p = probe(None);
    let cells = p.world.cells();
    for cell in [(0, 0), (63, 63), (17, 40), (40, 17)] {
        p.ask(cell);
        let (replies, _) = p.frames_until(1, 5);
        let r = replies.first().unwrap_or_else(|| panic!("no reply for {cell:?}"));
        let i = (cell.1 * 64 + cell.0) as usize;
        let (sigma, u, t, x) = from_dump(&p.planes, cells, i, &p.eos, &p.elements);
        assert_eq!(r.cell, cell);
        assert_eq!(r.sigma.to_bits(), sigma.to_bits(), "{cell:?} Σ");
        assert_eq!([r.u[0].to_bits(), r.u[1].to_bits()], [u[0].to_bits(), u[1].to_bits()], "{cell:?} u");
        assert_eq!(r.x.map(f32::to_bits), x.map(f32::to_bits), "{cell:?} X");
        assert!((f64::from(r.temperature) - t).abs() <= 1e-6 * t, "{cell:?} T {} vs {t}", r.temperature);
        assert!(r.temperature > 100.0, "{cell:?}: a hot cell reads hot, not the floor");
    }
}

#[test]
fn the_reply_arrives_within_2_frames_and_poll_never_waits() {
    let mut p = probe(None);
    p.ask((5, 9));
    let (replies, frame) = p.frames_until(1, 10);
    println!("inspect: reply in frame {frame}");
    assert_eq!(replies.len(), 1, "no reply in 10 frames");
    assert!(frame <= 2, "the reply took {frame} frames");
    // The adversarial case: a poll with nothing pending is instant, and one made at once after a request returns
    // whatever has arrived — never waits for the rest.
    let t0 = std::time::Instant::now();
    assert!(p.inspector.poll(&p.gpu.device).is_empty());
    assert!(t0.elapsed() < std::time::Duration::from_millis(5), "an empty poll took {:?}", t0.elapsed());
}

#[test]
fn the_reply_is_the_cell_as_it_stood_at_the_request_not_at_the_poll() {
    let mut p = probe(None);
    p.ask((3, 3));
    let cells = p.world.cells();
    let mut changed = p.planes.clone();
    changed[3 * 64 + 3] = 99.0;
    p.state.upload(&p.gpu.queue, &changed).unwrap();
    let (replies, _) = p.frames_until(1, 5);
    assert_eq!(replies[0].sigma.to_bits(), p.planes[3 * 64 + 3].to_bits());
    assert_ne!(replies[0].sigma, changed[3 * 64 + 3]);
    assert_eq!(cells, 4096);
}

#[test]
fn several_requests_answer_in_order_each_with_its_own_cell() {
    let mut p = probe(None);
    let cells = [(1, 1), (62, 2), (30, 31), (1, 1)];
    for c in cells {
        p.ask(c);
    }
    let (replies, frame) = p.frames_until(cells.len(), 6);
    assert!(frame <= 2, "{frame} frames for four requests");
    assert_eq!(replies.iter().map(|r| r.cell).collect::<Vec<_>>(), cells);
    assert_eq!(replies[0], replies[3]);
    assert_ne!(replies[0].sigma, replies[1].sigma);
}

#[test]
fn a_cell_outside_the_world_is_refused_and_asks_nothing() {
    let mut p = probe(None);
    for cell in [(64, 0), (0, 64), (u32::MAX, 3)] {
        let e = p.inspector.inspect(&p.gpu.device, &p.gpu.queue, &p.state, cell).unwrap_err();
        assert_eq!(e, StateError::CellOutside { cell, world: (64, 64) });
    }
    let (replies, _) = p.frames_until(1, 3);
    assert!(replies.is_empty());
}

#[test]
fn an_empty_cell_reads_zero_sigma_and_the_floor_temperature_and_a_cold_cell_the_floor() {
    let world = WorldConfig::new(64, 64).unwrap();
    let mut planes = inspectable(world);
    let cells = world.cells();
    // Cell 0: no mass at all. Cell 1: energy under the floor's — P8 would raise it, the inspector reads the floor.
    for c in 0..N_CHANNELS {
        planes[c * cells] = 0.0;
    }
    planes[3 * cells + 1] = 0.0;
    let mut p = probe(Some(planes));
    p.ask((0, 0));
    p.ask((1, 0));
    let (r, _) = p.frames_until(2, 5);
    let floor = p.eos.t_floor() as f32;
    assert_eq!((r[0].sigma, r[0].u, r[0].temperature), (0.0, [0.0, 0.0], floor));
    assert!(r[1].sigma > 0.0 && r[1].temperature > 0.0);
    let mu = 1.0 / p.elements.composition(&r[1].x.map(f64::from)).inv_mu;
    assert!((f64::from(r[1].temperature) - p.eos.t_floor()).abs() < 1e-5, "T {} mu {mu}", r[1].temperature);
}
