//! M0-T56 — the block summary (contract §1.9.1, §1.3.4, §3.1; crates/sr-engine/src/summary.rs, shaders/summary.wgsl) on the
//! physics adapter (R12), graded against an f64 sum of the same dump: the state, φ, the booking and the sinks read back after
//! the summarised frame, each block's 16 cells summed in f64 from their f32 values. Each field's band is K · 2⁻²⁴ · S, S the
//! block's sum of the field's per-cell magnitudes (below, `fields_f64`): K = 16 for sums of stored values (mass, momentum,
//! the accumulators, the unbound mass), 17 for one product a cell (Σx·m, the species, Σm·φ), 20 for the kinetic energy, 288
//! for what goes through the cold table (ε_cold, ε_th, Π, T, Π_cold / Π: ln u stored in f32 and WGSL's log and exp, ~2 × 10⁻⁵
//! relative at worst); the max-Σ cell's index, Σ, X and the sinks' masks exact. The world is hand-made — 64 × 64, a block of
//! gas off the world's centre so the box (0, 8, 56, 48) has an origin — the formula is proven, a star's summary is
//! NOT PROVEN (synthetic).

use sr_engine::dump::Sink;
use sr_engine::gpu::Gpu;
use sr_engine::scene::EdgeMode;
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::latch::FramePlan;
use sr_engine::step::sinks::SinkSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_engine::summary::{Block, BlockSummary, Frame, NUCLEAR};
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const N: usize = 64;
const CELLS: usize = N * N;
const U: f64 = 1.0 / (1u64 << 24) as f64;
/// The accumulators in TERMS' order, as Block lists them.
const ACCUMULATORS: [&str; 10] = ["radiated", NUCLEAR[0], NUCLEAR[1], NUCLEAR[2], NUCLEAR[3], NUCLEAR[4], NUCLEAR[5], NUCLEAR[6], "neutrino_lost", "neutrino_deposited"];

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// The world: gas on x ∈ [14, 42), y ∈ [18, 47) — Σ in [0.5, 1.5), every 7th cell 50–150, two max-Σ ties of 300 (cells
/// (19, 21) and (16, 22); (25, 29) and (27, 29)) — H, He, C and neutrons mixed, |u| ≤ 0.3, T in [1, 5], E holding the
/// cold energy; the rest the floor state.
fn world(eos: &Eos, elements: &Elements) -> Vec<f32> {
    let mut rng = Lcg(56);
    let mut p = vec![0f64; 14 * CELLS];
    for c in 0..CELLS {
        let (x, y) = (c % N, c / N);
        let gas = (14..42).contains(&x) && (18..47).contains(&y);
        let mut f = [0f64; N_SPECIES];
        let (sigma, vel, t) = if gas {
            let s = if [(19, 21), (16, 22), (25, 29), (27, 29)].contains(&(x, y)) {
                300.0
            } else if c % 7 == 0 {
                50.0 + 100.0 * rng.next()
            } else {
                0.5 + rng.next()
            };
            for i in [0, 1, 2, 9] {
                f[i] = rng.next() * if i == 9 { 0.05 } else { 1.0 };
            }
            (s, [0.6 * rng.next() - 0.3, 0.6 * rng.next() - 0.3], 1.0 + 4.0 * rng.next())
        } else {
            f[0] = 1.0;
            (1e-8, [0.0; 2], 0.05)
        };
        let total: f64 = f.iter().sum();
        f.iter_mut().for_each(|v| *v /= total);
        let comp = elements.composition(&f);
        let e = sigma * (t * comp.inv_mu + eos.eps_cold(sigma, comp.y_e, f[9])) + 0.5 * sigma * (vel[0] * vel[0] + vel[1] * vel[1]);
        for (k, v) in [sigma, sigma * vel[0], sigma * vel[1], e].into_iter().chain(f).enumerate() {
            p[k * CELLS + c] = v;
        }
    }
    p.iter().map(|&v| v as f32).collect()
}

fn le_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

struct Rig {
    gpu: Gpu,
    state: GpuState,
    booking: Booking,
    step: Step,
    physics: Physics,
    elements: Elements,
    /// φ as uploaded (zeros in a step without it).
    phi: Vec<f32>,
}

/// The step over the box: P3 with three sinks and φ from its side fields when `sinks`, else P4 in bounce when `hydro`.
fn rig(sinks: bool, hydro: bool) -> Rig {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let gpu = super::device();
    let world = WorldConfig::new(N as u32, N as u32).unwrap();
    let state = GpuState::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &world_planes(&physics, &elements)).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let side = SideFields::new(&gpu.device, world).unwrap();
    let eos = EosGpu::new(&gpu.device, &physics, &elements);
    let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: hydro.then_some(EdgeMode::Bounce) };
    let setup = SinkSetup { side: &side };
    let step = Step::with_sinks(&gpu.device, &state, &booking, &eos, &physics, &passes, None, None, sinks.then_some(&setup)).unwrap();
    let mut rig = Rig { gpu, state, booking, step, physics, elements, phi: vec![0.0; CELLS] };
    if sinks {
        let s = |x: f64, y: f64| Sink { x, y, vx: 0.0, vy: 0.0, m: 10.0 };
        rig.step.set_sinks(&rig.gpu.queue, &[s(33.5, 38.5), s(37.5, 38.5), s(6.5, 12.5)]).unwrap();
        // One bare step fits the box; then φ, ±50 % past each cell's ½|u|² + ε_th (no cell near the unbound line), and every
        // accumulator a signed value from 10⁻³ to 10.
        rig.step.run(&rig.gpu.device, &rig.gpu.queue, 1);
        let cells = rig.state.readback(&rig.gpu.device, &rig.gpu.queue);
        let eos = Eos::from_physics(&rig.physics).unwrap();
        let mut rng = Lcg(7);
        rig.phi = (0..CELLS).map(|c| (-(rig.cell(&eos, &cells, c).specific) * if rng.next() < 0.5 { 0.5 } else { 1.5 }) as f32).collect();
        rig.gpu.queue.write_buffer(side.phi(), 0, &le_bytes(&rig.phi));
        for g in BOOK_GROUPS.iter().enumerate().filter(|(_, g)| TERMS[g.terms.start].accumulator) {
            let n = g.1.terms.len() * CELLS;
            let v: Vec<f32> = (0..n).map(|_| ((2.0 * rng.next() - 1.0) * 10f64.powf(4.0 * rng.next() - 3.0)) as f32).collect();
            rig.gpu.queue.write_buffer(rig.booking.group(g.0), 0, &le_bytes(&v));
        }
    } else {
        rig.step.run(&rig.gpu.device, &rig.gpu.queue, 1);
    }
    rig
}

fn world_planes(physics: &Physics, elements: &Elements) -> Vec<f32> {
    world(&Eos::from_physics(physics).unwrap(), elements)
}

/// A cell's EOS quantities in f64 from its f32 values.
struct CellF64 {
    sigma: f64,
    mom: [f64; 2],
    x: [f64; N_SPECIES],
    ke: f64,
    ec: f64,
    eth: f64,
    /// ½|u|² + ε_th.
    specific: f64,
    abs_e: f64,
    pi: f64,
    pi_cold: f64,
    inv_mu: f64,
}

impl Rig {
    fn cell(&self, eos: &Eos, cells: &[f32], c: usize) -> CellF64 {
        let v = |k: usize| f64::from(cells[k * CELLS + c]);
        let (sigma, mom, e) = (v(0), [v(1), v(2)], v(3));
        let x: [f64; N_SPECIES] = std::array::from_fn(|i| v(4 + i));
        let comp = self.elements.composition(&x);
        let u2 = (mom[0] * mom[0] + mom[1] * mom[1]) / (sigma * sigma);
        let ec = eos.eps_cold(sigma, comp.y_e, x[9]);
        let eth = e / sigma - 0.5 * u2 - ec;
        let pi_cold = eos.pi_e(sigma, comp.y_e) + eos.pi_n(sigma, x[9]);
        CellF64 { sigma, mom, x, ke: 0.5 * u2, ec, eth, specific: 0.5 * u2 + eth, abs_e: (e / sigma).abs(), pi: sigma * eth + pi_cold, pi_cold, inv_mu: comp.inv_mu }
    }

    /// One frame of `steps`, waited for, and its summary.
    fn frame(&self, steps: u32) -> Frame {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.step.encode_frame(&FramePlan { steps, slow_down: false }, &mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
        self.gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let mut frames = self.step.poll_frames(&self.gpu.device);
        assert_eq!(frames.len(), 1, "one frame per frame: {:?}", frames.iter().map(|f| f.report).collect::<Vec<_>>());
        frames.remove(0)
    }

    fn book(&self) -> Vec<f32> {
        self.booking.readback(&self.gpu.device, &self.gpu.queue)
    }
}

/// The accumulator planes of a booking readback, in [`ACCUMULATORS`]' order.
fn accumulators(book: &[f32]) -> Vec<&[f32]> {
    let (mut base, mut planes) = (0, Vec::new());
    for g in &BOOK_GROUPS {
        for t in g.terms.clone() {
            let slots = g.domain.slots(WorldConfig::new(N as u32, N as u32).unwrap());
            if TERMS[t].accumulator {
                planes.push(&book[base..base + slots]);
            }
            base += slots;
        }
    }
    assert_eq!(TERMS.iter().filter(|t| t.accumulator).map(|t| t.name).collect::<Vec<_>>(), ACCUMULATORS);
    planes
}

/// A block's f32 fields by name, then its exact words (the max-Σ cell's index and Σ, its X, the sinks' mask).
fn fields(b: &Block) -> (Vec<(String, f32)>, Vec<u32>) {
    let mut f: Vec<(String, f32)> = [("mass", b.mass), ("Σx·m", b.mx), ("Σy·m", b.my), ("p_x", b.momentum[0]), ("p_y", b.momentum[1])]
        .into_iter()
        .chain([("kinetic", b.kinetic), ("thermal", b.thermal), ("cold", b.cold), ("∫Π dA", b.pressure), ("Σm·φ", b.phi_m)])
        .chain([("unbound", b.unbound), ("T max", b.max_t), ("Π_cold/Π max", b.cold_ratio)])
        .map(|(n, v)| (n.to_string(), v))
        .collect();
    f.extend(b.species.iter().enumerate().map(|(i, &v)| (format!("species {i}"), v)));
    let acc = [b.radiated].into_iter().chain(b.nuclear).chain([b.neutrino_lost, b.neutrino_deposited]);
    f.extend(ACCUMULATORS.iter().zip(acc).map(|(n, v)| (n.to_string(), v)));
    let exact = [b.max_cell, b.max_sigma.to_bits(), b.sinks].into_iter().chain(b.max_x.map(f32::to_bits)).collect();
    (f, exact)
}

/// The f64 sum of a block from the dump: (field, want, S, K) in [`fields`]' order, then the exact words.
fn fields_f64(r: &Rig, cells: &[f32], book: &[f32], s: &BlockSummary, col: u32, row: u32) -> (Vec<(String, f64, f64, f64)>, Vec<u32>) {
    let eos = Eos::from_physics(&r.physics).unwrap();
    let acc = accumulators(book);
    let (x0, y0) = (s.origin[0] as usize + 4 * col as usize, s.origin[1] as usize + 4 * row as usize);
    let ids: Vec<usize> = (0..16).map(|k| (y0 + k / 4) * N + x0 + k % 4).collect();
    let all: Vec<CellF64> = ids.iter().map(|&c| r.cell(&eos, cells, c)).collect();
    let sum = |f: &dyn Fn(usize, &CellF64) -> (f64, f64)| ids.iter().zip(&all).fold((0.0, 0.0), |(a, m), (&c, q)| (a + f(c, q).0, m + f(c, q).1.abs()));
    let at = |c: usize| [(c % N) as f64 + 0.5, (c / N) as f64 + 0.5];
    let thermal_scale = |q: &CellF64| q.sigma * (q.abs_e + q.ke + q.ec);
    let best = (0..16).fold(0, |b, k| if all[k].sigma > all[b].sigma { k } else { b });
    let q = &all[best];
    let t = q.eth.max(eos.t_floor() * q.inv_mu) / q.inv_mu;
    let tol_t = (q.abs_e + q.ke + q.ec) / q.inv_mu;
    let mut f: Vec<(String, f64, f64, f64)> = vec![
        ("mass", sum(&|_, q| (q.sigma, q.sigma)), 16.0),
        ("Σx·m", sum(&|c, q| (at(c)[0] * q.sigma, at(c)[0] * q.sigma)), 17.0),
        ("Σy·m", sum(&|c, q| (at(c)[1] * q.sigma, at(c)[1] * q.sigma)), 17.0),
        ("p_x", sum(&|_, q| (q.mom[0], q.mom[0])), 16.0),
        ("p_y", sum(&|_, q| (q.mom[1], q.mom[1])), 16.0),
        ("kinetic", sum(&|_, q| (q.sigma * q.ke, q.sigma * q.ke)), 20.0),
        ("thermal", sum(&|_, q| (q.sigma * q.eth, thermal_scale(q))), 288.0),
        ("cold", sum(&|_, q| (q.sigma * q.ec, q.sigma * q.ec)), 288.0),
        ("∫Π dA", sum(&|_, q| (q.pi, thermal_scale(q) + q.pi_cold)), 288.0),
        ("Σm·φ", sum(&|c, q| (q.sigma * f64::from(r.phi[c]), q.sigma * f64::from(r.phi[c]))), 17.0),
        ("unbound", sum(&|c, q| (if q.specific + f64::from(r.phi[c]) > 0.0 { q.sigma } else { 0.0 }, q.sigma)), 16.0),
        ("T max", (t, tol_t), 288.0),
        ("Π_cold/Π max", (q.pi_cold / q.pi, (thermal_scale(q) + 2.0 * q.pi_cold) / q.pi.abs()), 288.0),
    ]
    .into_iter()
    .map(|(n, (want, scale), k)| (n.to_string(), want, scale, k))
    .collect();
    for i in 0..N_SPECIES {
        let (want, scale) = sum(&|_, q| (q.sigma * q.x[i], q.sigma * q.x[i]));
        f.push((format!("species {i}"), want, scale, 17.0));
    }
    for (name, plane) in ACCUMULATORS.iter().zip(&acc) {
        let (want, scale) = sum(&|c, _| (f64::from(plane[c]), f64::from(plane[c])));
        f.push((name.to_string(), want, scale, 16.0));
    }
    let inside = |sk: &Sink| sk.x >= 0.0 && sk.y >= 0.0 && (x0..x0 + 4).contains(&(sk.x as usize)) && (y0..y0 + 4).contains(&(sk.y as usize));
    let mask = s.sinks.iter().enumerate().filter(|(_, sk)| inside(sk)).fold(0u32, |m, (k, _)| m | 1 << k);
    let exact = [ids[best] as u32, cells[ids[best]].to_bits(), mask].into_iter().chain((0..N_SPECIES).map(|i| cells[(4 + i) * CELLS + ids[best]].to_bits())).collect();
    (f, exact)
}

/// Every block of `s` against the f64 sum of the dump; returns the worst |gpu − want| / (U · S) per field.
fn check(r: &Rig, s: &BlockSummary, cells: &[f32], book: &[f32]) -> Vec<(String, f64)> {
    let rows = s.size[1] / 4;
    assert_eq!((s.columns, s.blocks.len() as u32), (s.size[0] / 4, s.columns * rows), "the box's blocks, row-major");
    let mut worst: Vec<(String, f64)> = Vec::new();
    for row in 0..rows {
        for col in 0..s.columns {
            let (got, got_exact) = fields(s.block(col, row));
            let (want, want_exact) = fields_f64(r, cells, book, s, col, row);
            assert_eq!(got_exact, want_exact, "block ({col}, {row}): the max-Σ cell's index, Σ, mask and X");
            assert_eq!(got.iter().map(|f| &f.0).collect::<Vec<_>>(), want.iter().map(|f| &f.0).collect::<Vec<_>>());
            for ((name, g), (_, w, scale, k)) in got.iter().zip(&want) {
                let err = (f64::from(*g) - w).abs();
                assert!(err <= k * U * scale, "block ({col}, {row}) {name}: {g} against {w}, |Δ| {err:e} > {k} · 2⁻²⁴ · {scale:e}");
                let ratio = if scale > &0.0 { err / (U * scale) } else { 0.0 };
                match worst.iter_mut().find(|(n, _)| n == name) {
                    Some((_, m)) => *m = m.max(ratio),
                    None => worst.push((name.clone(), ratio)),
                }
            }
        }
    }
    worst
}

/// The cells of the summarised frame and its booking.
fn dump(r: &Rig) -> (Vec<f32>, Vec<f32>) {
    (r.state.readback(&r.gpu.device, &r.gpu.queue), r.book())
}

#[test]
fn gpu_every_field_is_an_f64_sum_of_the_dump() {
    let r = rig(true, false);
    let frame = r.frame(0);
    let (cells, book) = dump(&r);
    let worst = check(&r, &frame.summary, &cells, &book);
    println!("box {:?} + {:?}, {} blocks; worst |Δ| / (2⁻²⁴ S): {worst:?}", frame.summary.origin, frame.summary.size, frame.summary.blocks.len());
    let unbound: f32 = frame.summary.blocks.iter().map(|b| b.unbound).sum();
    let mass: f32 = frame.summary.blocks.iter().map(|b| b.mass).sum();
    assert!(unbound > 0.1 * mass && unbound < 0.9 * mass, "φ splits the gas into bound and unbound: {unbound} of {mass}");
}

#[test]
fn gpu_the_blocks_tile_the_box() {
    let r = rig(true, false);
    let s = r.frame(0).summary;
    let b = r.step.read_box(&r.gpu.device, &r.gpu.queue);
    assert_eq!((s.origin, s.size), ([b.x0, b.y0], [b.w, b.h]), "the box the GPU holds");
    assert_eq!((s.origin, s.size), ([0, 8], [56, 48]), "gas on [14, 42) × [18, 47), 8 cells of margin, multiples of 8");
    let cells = r.state.readback(&r.gpu.device, &r.gpu.queue);
    let inside = |c: usize| (c % N) < 56 && (8..56).contains(&(c / N));
    let mass: f64 = (0..CELLS).filter(|&c| inside(c)).map(|c| f64::from(cells[c])).sum();
    let sigma_vac = r.physics.get("sigma_vac").unwrap();
    assert!((0..CELLS).all(|c| inside(c) || f64::from(cells[c]) < sigma_vac), "every cell outside the box is vacuum");
    let got: f64 = s.blocks.iter().map(|b| f64::from(b.mass)).sum();
    assert!((got - mass).abs() <= 16.0 * U * mass, "the blocks' mass {got} against the box's {mass}");
}

#[test]
fn gpu_the_max_cell_is_the_densest_the_lowest_y_then_x() {
    let r = rig(true, false);
    let s = r.frame(0).summary;
    // (19, 21) and (16, 22) in block (4, 3); (25, 29) and (27, 29) in block (6, 5): Σ = 300 each.
    for ((col, row), (x, y)) in [((4, 3), (19, 21)), ((6, 5), (25, 29))] {
        let b = s.block(col, row);
        assert_eq!((b.max_cell, b.max_sigma), ((y * N + x) as u32, 300.0), "block ({col}, {row})");
    }
}

#[test]
fn gpu_the_sinks_inside_are_marked() {
    let r = rig(true, false);
    let s = r.frame(0).summary;
    assert_eq!(s.sinks.len(), 3);
    let marked: Vec<(u32, u32, u32)> = (0..s.blocks.len() as u32)
        .map(|i| (i % s.columns, i / s.columns, s.blocks[i as usize].sinks))
        .filter(|&(_, _, m)| m != 0)
        .collect();
    println!("sinks {:?}; marked {marked:?}", s.sinks);
    // (33.5, 38.5) and (37.5, 38.5) in blocks (8, 7) and (9, 7); (6.5, 12.5) in block (1, 1).
    assert_eq!(marked, vec![(1, 1, 0b100), (8, 7, 0b001), (9, 7, 0b010)]);
}

#[test]
fn gpu_the_accumulators_reset_after_each_summary() {
    let r = rig(true, false);
    let before = r.book();
    let first = r.frame(0).summary;
    assert_eq!(r.book().iter().map(|v| v.to_bits()).collect::<Vec<_>>(), before.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), "held until the next frame");
    assert!(first.blocks.iter().any(|b| b.radiated != 0.0 && b.neutrino_deposited != 0.0 && b.nuclear.iter().all(|&v| v != 0.0)));
    let second = r.frame(0).summary;
    let after = r.book();
    for b in &second.blocks {
        let (f, _) = fields(b);
        assert!(f.iter().filter(|(n, _)| ACCUMULATORS.contains(&n.as_str())).all(|(_, v)| *v == 0.0), "the next summary's accumulators: {f:?}");
    }
    assert!(accumulators(&after).iter().all(|p| p.iter().all(|&v| v == 0.0)), "the accumulator planes cleared");
    let others = |book: &[f32]| -> Vec<u32> {
        let (mut base, mut kept) = (0, Vec::new());
        for g in &BOOK_GROUPS {
            let slots = g.domain.slots(WorldConfig::new(N as u32, N as u32).unwrap());
            for t in g.terms.clone() {
                if !TERMS[t].accumulator {
                    kept.extend(book[base..base + slots].iter().map(|v| v.to_bits()));
                }
                base += slots;
            }
        }
        kept
    };
    assert_eq!(others(&after), others(&before), "no other booked term touched");
    assert!(others(&before).iter().any(|&v| v != 0), "P8's vacuum reset booked something to keep");
}

#[test]
fn gpu_the_summary_arrives_with_its_frame_at_most_two_frames_late() {
    // Run A: frames one at a time, each summary against the dump after it.
    let a = rig(false, true);
    let mut want = Vec::new();
    for _ in 0..3 {
        let f = a.frame(3);
        let (cells, book) = dump(&a);
        check(&a, &f.summary, &cells, &book);
        assert_eq!(f.report.steps_run, 3);
        want.push(f);
    }
    let momenta = |f: &Frame| f.summary.blocks.iter().map(|b| b.momentum).collect::<Vec<_>>();
    assert_ne!(momenta(&want[0]), momenta(&want[2]), "the gas moved between frames");
    // Run B: the same three frames submitted back to back; waiting for the first submission alone hands over its summary.
    let b = rig(false, true);
    let submissions: Vec<wgpu::SubmissionIndex> = (0..3)
        .map(|_| {
            let mut encoder = b.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
            b.step.encode_frame(&FramePlan { steps: 3, slow_down: false }, &mut encoder);
            b.gpu.queue.submit([encoder.finish()])
        })
        .collect();
    b.gpu.device.poll(wgpu::PollType::Wait { submission_index: Some(submissions[0].clone()), timeout: None }).unwrap();
    let mut got = b.step.poll_frames(&b.gpu.device);
    assert!(!got.is_empty(), "frame 0's summary in hand once its own submission has ended, two frames still queued behind it");
    b.gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    got.extend(b.step.poll_frames(&b.gpu.device));
    assert_eq!(got, want, "each frame's own summary, in order, the same bits as run A's");
}

#[test]
fn gpu_two_runs_give_the_same_bits() {
    let words = |s: &BlockSummary| -> Vec<u32> {
        s.blocks.iter().flat_map(|b| {
            let (f, exact) = fields(b);
            f.into_iter().map(|(_, v)| v.to_bits()).chain(exact).collect::<Vec<_>>()
        }).collect()
    };
    let (one, two) = (rig(true, false).frame(0).summary, rig(true, false).frame(0).summary);
    assert_eq!(words(&one), words(&two));
}
