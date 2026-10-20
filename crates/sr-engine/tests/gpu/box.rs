//! M0-T21 — the active box (contract §1.3.3, §1.3.2, §1.3.4; TW-R1 A2; G-BOX's box half). M0-T39 adds G-BOX's
//! conservation cases at the end: the box against the whole world on scenes/sun_disk.json (every sum at step 100, the
//! masses at 2,000 — contract §0.5 [M0-T39]), and a blob at 0.35 cells per step against the box's margin in bounce mode
//! (scenes/box_blob.json).
//!
//! The oracle is §1.3.3 written here, in i64 from the uploaded f32 state, never the engine's constants: the bounding box
//! of the cells with Σ ≥ Σ_vac, grown by 8 cells, rounded out to multiples of 8, clamped to the world (in that order);
//! the FFT size per axis the smallest power of two ≥ 2 × the box, at least 32; the re-fit after P9 at step indices
//! ≡ 0 (mod 16) and in P0 after an edit outside the box. Every number is exact (integers); the one float check, Δt over
//! the box, is dt.rs's 1e-6 relative.

use super::device;
use std::path::{Path, PathBuf};

use sr_engine::gpu::Gpu;
use sr_engine::scene::Scene;
use sr_engine::state::{book_totals, Booking, SideFields, State, WorldConfig, CHANNELS, N_CHANNELS, N_TERMS, TERMS};
use sr_engine::step::boxfit::{BoxMode, BoxReport, ARGS_CELLS, ARGS_TILES};
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// §1.3.3 and §1.3.2, as the contract fixes them.
const MARGIN: i64 = 8;
const ROUND: i64 = 8;
const FFT_MIN: u32 = 32;
const REFIT_EVERY: u32 = 16;
const C: f64 = 0.4;
const DT_TOL: f64 = 1e-6;

struct Constants {
    physics: Physics,
    elements: Elements,
}

impl Constants {
    fn f32_of(&self, key: &str) -> f32 {
        self.physics.get(key).unwrap() as f32
    }
}

/// The shipped constants, the cold pairs off (so c² = 2ε_th, as in tests/gpu/dt.rs).
fn constants() -> Constants {
    let physics = Physics::shipped().unwrap().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).unwrap();
    Constants { physics, elements: Elements::shipped().unwrap() }
}

fn channel(name: &str) -> usize {
    CHANNELS.iter().position(|&n| n == name).unwrap()
}

/// A world of vacuum — Σ = 0, E = 0, at rest, pure hydrogen.
fn vacuum(world: WorldConfig) -> Vec<f32> {
    let mut p = vec![0f32; N_CHANNELS * world.cells()];
    p[channel("x_H") * world.cells()..channel("x_He") * world.cells()].fill(1.0);
    p
}

fn set(p: &mut [f32], world: WorldConfig, (x, y): (u32, u32), ch: usize, v: f32) {
    p[ch * world.cells() + (y * world.width + x) as usize] = v;
}

/// Gas at rest (Σ = 1, E = 0) on each cell.
fn with_gas(world: WorldConfig, cells: &[(u32, u32)]) -> Vec<f32> {
    let mut p = vacuum(world);
    for &c in cells {
        set(&mut p, world, c, 0, 1.0);
    }
    p
}

/// §1.3.3's box of `p`: [x0, y0, w, h]; all zero when no cell holds Σ ≥ Σ_vac.
fn oracle(world: WorldConfig, p: &[f32], sigma_vac: f32) -> [u32; 4] {
    let (w, h) = (i64::from(world.width), i64::from(world.height));
    let mut b: Option<[i64; 4]> = None;
    for (i, _) in p[..world.cells()].iter().enumerate().filter(|(_, s)| **s >= sigma_vac) {
        let (x, y) = (i as i64 % w, i as i64 / w);
        let o = b.unwrap_or([x, y, x + 1, y + 1]);
        b = Some([o[0].min(x), o[1].min(y), o[2].max(x + 1), o[3].max(y + 1)]);
    }
    let Some(b) = b else { return [0; 4] };
    let lo = |v: i64| ((v - MARGIN).div_euclid(ROUND) * ROUND).max(0);
    let hi = |v: i64, n: i64| ((v + MARGIN + ROUND - 1).div_euclid(ROUND) * ROUND).min(n);
    let (x0, y0) = (lo(b[0]), lo(b[1]));
    [x0, y0, hi(b[2], w) - x0, hi(b[3], h) - y0].map(|v| v as u32)
}

fn fft(n: u32) -> u32 {
    (2 * n).next_power_of_two().max(FFT_MIN)
}

fn rect(r: &BoxReport) -> [u32; 4] {
    [r.x0, r.y0, r.w, r.h]
}

struct Run {
    state: State,
    step: Step,
}

impl Run {
    fn new(gpu: &Gpu, c: &Constants, world: WorldConfig, planes: &[f32], mode: BoxMode) -> Run {
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let step = Step::with_box(&gpu.device, &state, &booking, &eos, &c.physics, mode);
        Run { state, step }
    }

    fn run(&self, gpu: &Gpu, steps: u32) -> BoxReport {
        self.step.run(&gpu.device, &gpu.queue, steps);
        self.step.read_box(&gpu.device, &gpu.queue)
    }

    /// The state with the gas moved: Σ = 0 on `from`, 1 on `to` (an edit in all but name).
    fn move_gas(&self, gpu: &Gpu, from: (u32, u32), to: (u32, u32)) {
        let world = self.state.world();
        let mut p = self.state.readback(&gpu.device, &gpu.queue);
        set(&mut p, world, from, 0, 0.0);
        set(&mut p, world, to, 0, 1.0);
        self.state.upload(&gpu.queue, &p).unwrap();
    }
}

#[test]
fn the_box_is_the_non_vacuum_cells_grown_by_8_rounded_to_8_and_clamped_at_known_places_edges_and_corners() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::default();
    let sigma_vac = c.f32_of("sigma_vac");
    let below = f32::from_bits(sigma_vac.to_bits() - 1);
    // (gas cells, the box [x0, y0, w, h] worked by hand from §1.3.3)
    let rect_cells: Vec<(u32, u32)> = (50..60).flat_map(|y| (100..200).map(move |x| (x, y))).collect();
    let places: Vec<(&str, Vec<(u32, u32)>, [u32; 4])> = vec![
        ("empty world", vec![], [0, 0, 0, 0]),
        ("one cell, middle", vec![(300, 200)], [288, 192, 24, 24]),
        ("on the 8-grid", vec![(8, 8)], [0, 0, 24, 24]),
        ("on the 8-grid, inside", vec![(16, 16)], [8, 8, 24, 24]),
        ("one off the grid", vec![(7, 7)], [0, 0, 16, 16]),
        ("corner (0, 0)", vec![(0, 0)], [0, 0, 16, 16]),
        ("corner (599, 399)", vec![(599, 399)], [584, 384, 16, 16]),
        ("left edge", vec![(0, 200)], [0, 192, 16, 24]),
        ("right edge", vec![(599, 200)], [584, 192, 16, 24]),
        ("top edge", vec![(300, 0)], [288, 0, 24, 16]),
        ("a 100 × 10 block", rect_cells, [88, 40, 120, 32]),
        ("two far corners", vec![(10, 390), (590, 5)], [0, 0, 600, 400]),
    ];
    for (name, cells, want) in &places {
        let start = with_gas(world, cells);
        let got = rect(&Run::new(&gpu, &c, world, &start, BoxMode::Fit).run(&gpu, 1));
        println!("{name}: box {got:?}");
        assert_eq!(oracle(world, &start, sigma_vac), *want, "{name}: the oracle against the hand-worked box");
        assert_eq!(got, *want, "{name}");
    }
    // Σ = Σ_vac counts as matter, the next f32 below does not.
    let mut p = with_gas(world, &[]);
    set(&mut p, world, (40, 40), 0, sigma_vac);
    set(&mut p, world, (500, 300), 0, below);
    let got = rect(&Run::new(&gpu, &c, world, &p, BoxMode::Fit).run(&gpu, 1));
    assert_eq!((got, oracle(world, &p, sigma_vac)), ([32, 32, 24, 24], [32, 32, 24, 24]), "Σ_vac at (40, 40), below at (500, 300)");
}

#[test]
fn the_fft_size_per_axis_is_the_smallest_power_of_two_at_least_twice_the_box_and_32() {
    let gpu = device();
    let c = constants();
    let cases: [((u32, u32), Vec<(u32, u32)>, [u32; 2]); 5] = [
        ((600, 400), vec![], [32, 32]),
        ((600, 400), vec![(4, 4)], [32, 32]),                       // 16 × 16
        ((600, 400), vec![(300, 200)], [64, 64]),                   // 24 × 24
        ((600, 400), vec![(0, 0), (599, 399)], [2048, 1024]),       // the world
        ((2048, 64), vec![(0, 0), (2047, 63)], [4096, 128]),        // the widest world
    ];
    for ((w, h), cells, want) in &cases {
        let world = WorldConfig::new(*w, *h).unwrap();
        let r = Run::new(&gpu, &c, world, &with_gas(world, cells), BoxMode::Fit).run(&gpu, 1);
        println!("{w} × {h}, gas on {cells:?}: box {:?}, FFT {:?}", rect(&r), r.fft);
        assert_eq!([fft(r.w), fft(r.h)], *want, "the formula against the hand-worked sizes");
        assert_eq!(r.fft, *want, "{w} × {h}, box {:?}", rect(&r));
    }
}

#[test]
fn the_box_is_refit_after_p9_at_step_indices_0_mod_16_and_never_between() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(256, 128).unwrap();
    let (a, b) = ((40, 40), (200, 100));
    let (box_a, box_b) = ([32, 32, 24, 24], [192, 88, 24, 24]);
    let run = Run::new(&gpu, &c, world, &with_gas(world, &[a]), BoxMode::Fit);
    assert_eq!(run.step.box_labels()[0], ["box_cells", "box_fit"]);
    // (gas moved before the batch, steps run, the box, the first step it holds for, the fits written)
    let script = [
        (None, 1, box_a, 1, 2), // before step 0, then after step 0's P9
        (Some((a, b)), 15, box_a, 1, 2),
        (None, 1, box_b, 17, 3), // after step 16's P9
        (Some((b, a)), 15, box_b, 17, 3),
        (None, 1, box_a, 33, 4), // after step 32's P9
    ];
    let mut steps = 0;
    for (moved, n, want, from, fits) in script {
        if let Some((from, to)) = moved {
            run.move_gas(&gpu, from, to);
        }
        let r = run.run(&gpu, n);
        steps += n;
        println!("after {steps} steps: box {:?}, from step {}, {} fits", rect(&r), r.from_step, r.fits);
        assert_eq!((rect(&r), r.from_step, r.fits), (want, from, fits), "after {steps} steps");
        assert_eq!(r.from_step, (steps - 1) / REFIT_EVERY * REFIT_EVERY + 1, "the last re-fit's step index");
    }
}

#[test]
fn an_edit_outside_the_box_refits_it_in_p0_and_the_flag_clears() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(256, 128).unwrap();
    let (a, b) = ((40, 40), (200, 100));
    let (box_a, box_b) = ([32, 32, 24, 24], [192, 88, 24, 24]);
    let run = Run::new(&gpu, &c, world, &with_gas(world, &[a]), BoxMode::Fit);
    assert_eq!(run.step.box_labels()[1], ["box_gate", "box_cells", "box_fit"]);
    let r = run.run(&gpu, 3);
    assert_eq!((rect(&r), r.fits), (box_a, 2));
    run.move_gas(&gpu, a, b);
    let r = run.run(&gpu, 1);
    assert_eq!((rect(&r), r.fits), (box_a, 2), "step 3, no edit flagged: no re-fit");
    // P0's edit pass ORs 1 into the flag when an edit lands outside the box; the box holds from this step's P1.
    gpu.queue.write_buffer(run.step.box_edit_flag(), 0, &1u32.to_le_bytes());
    let r = run.run(&gpu, 1);
    println!("step 4, edit flagged: box {:?}, from step {}, {} fits", rect(&r), r.from_step, r.fits);
    assert_eq!((rect(&r), r.from_step, r.fits), (box_b, 4, 3), "step 4's P0 re-fit");
    run.move_gas(&gpu, b, a);
    let r = run.run(&gpu, 1);
    assert_eq!((rect(&r), r.fits), (box_b, 3), "step 5: the flag was cleared by step 4's fit");
}

#[test]
fn the_passes_read_their_dispatch_sizes_from_the_box() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::new(256, 128).unwrap();
    let cells = world.cells();
    let (sigma_vac, sigma_floor) = (c.f32_of("sigma_vac"), c.f32_of("sigma_floor"));
    // Iron gas at rest and cold on x 100..120, y 50..60; inside its box every cell iron, outside vacuum hydrogen — so a
    // Δt over the box and one over the world differ (c² = 2 T_floor/μ, §2.3.3).
    let mut start = with_gas(world, &(50..60).flat_map(|y| (100..120).map(move |x| (x, y))).collect::<Vec<_>>());
    let [x0, y0, w, h] = oracle(world, &start, sigma_vac);
    assert_eq!([x0, y0, w, h], [88, 40, 40, 32]);
    let inside = |i: usize| (x0..x0 + w).contains(&(i as u32 % 256)) && (y0..y0 + h).contains(&(i as u32 / 256));
    for i in (0..cells).filter(|&i| inside(i)) {
        start[channel("x_H") * cells + i] = 0.0;
        start[channel("x_Fe") * cells + i] = 1.0;
    }
    let dt_of = |x_fe: f64| {
        let x: [f64; N_SPECIES] = std::array::from_fn(|k| if k == 8 { x_fe } else if k == 0 { 1.0 - x_fe } else { 0.0 });
        C / (2.0 * (2.0 * f64::from(c.f32_of("t_floor")) * c.elements.composition(&x).inv_mu).sqrt())
    };
    for (mode, want_rect, want_dt) in [(BoxMode::Fit, [x0, y0, w, h], dt_of(1.0)), (BoxMode::WholeWorld, [0, 0, 256, 128], dt_of(0.0))] {
        let run = Run::new(&gpu, &c, world, &start, mode);
        let r = run.run(&gpu, 1);
        let dt = run.step.read_dt(&gpu.device, &gpu.queue).dt as f64;
        let after = run.state.readback(&gpu.device, &gpu.queue);
        println!("{mode:?}: box {:?}, tiles {:?}, cells {:?}, Δt_0 {dt:e} (want {want_dt:e})", rect(&r), r.tiles, r.cells);
        assert_eq!(rect(&r), want_rect, "{mode:?}");
        assert_eq!(r.tiles, [r.w / 8, r.h / 8, 1], "{mode:?}: 8 × 8 tiles over the box");
        assert_eq!(r.cells, [(r.w * r.h).div_ceil(256), 1, 1], "{mode:?}: 256-cell workgroups over the box");
        assert_eq!(
            run.step.box_sized(),
            [("floor_cells", ARGS_TILES), ("renormalise_species", ARGS_TILES), ("dt_cells", ARGS_CELLS)],
            "the dispatches sized by the box"
        );
        assert!((dt - want_dt).abs() <= DT_TOL * want_dt, "{mode:?}: Δt_0 {dt:e}, the f64 formula over the box {want_dt:e}");
        // P8 floors the vacuum inside the box; outside it, the cells are as uploaded (Σ = 0, not Σ_floor).
        let held = |i: usize| mode == BoxMode::Fit && !inside(i);
        for i in (0..cells).filter(|&i| start[i] < sigma_vac) {
            let want = if held(i) { 0.0 } else { sigma_floor };
            assert_eq!(after[i].to_bits(), want.to_bits(), "{mode:?}: cell ({}, {}) Σ {}", i % 256, i / 256, after[i]);
        }
    }
}

#[test]
fn the_whole_world_switch_holds_the_box_at_the_world() {
    let gpu = device();
    let c = constants();
    let world = WorldConfig::default();
    for cells in [vec![], vec![(0, 0)], vec![(300, 200)]] {
        let run = Run::new(&gpu, &c, world, &with_gas(world, &cells), BoxMode::WholeWorld);
        for (n, from, fits) in [(1, 1, 2), (16, 17, 3)] {
            let r = run.run(&gpu, n);
            assert_eq!((rect(&r), r.fft, r.tiles), ([0, 0, 600, 400], [2048, 1024], [75, 50, 1]), "gas on {cells:?}");
            assert_eq!((r.from_step, r.fits), (from, fits), "gas on {cells:?}: the switch keeps the re-fit's schedule");
        }
    }
}

// ---- M0-T39: G-BOX's conservation cases (contract §5.3, TW-R5 T2) ----

/// §5.3's band: each conserved sum equal across the two runs to 10⁻⁶ of the sum's own scale (its cells' magnitudes summed,
/// so a momentum that sums to ~0 is graded against the motion it is made of, not against itself).
const CONSERVED_TOL: f64 = 1e-6;

/// Contract §0.5 [M0-T39], [M0-D66]: every sum compared at step 100, while the box is still inside the world; mass and each
/// species' mass again at 2,000. Gravity's FFT sizes differ in round-off (1.7e-8 of momentum at step 1) and the collapse
/// grows it (3.1e-6 of momentum and 4.4e-6 of energy at 2,000, RTX 5090); with gravity off the two runs are bit-identical.
/// M0-T39 graded step 200; on §0.5 [M0-D64]'s face work the core heats fast there while the runs' clocks drift apart by Δt's
/// round-off (1.7e-6 of sim time at 200), so energy reads 5.4e-6 at 200 and 1.6e-7 at 100 (the lead, 2026-10-20, M0-D66).
const EARLY: u32 = 100;
const STEPS: u32 = 2000;

/// The mass sums of `s` — Σ and each species' Σ·X_i — the ones a vacuum reset books and the gravity round-off leaves
/// alone; momentum (a vacuum reset removes it unbooked, §2.7 [M0-T19]) and energy dropped.
fn masses(s: Vec<Sum>) -> Vec<Sum> {
    s.into_iter().enumerate().filter(|(k, _)| !(1..=3).contains(k)).map(|(_, x)| x).collect()
}

/// One conserved sum: the total in f64 and the scale it is graded against.
struct Sum {
    name: &'static str,
    total: f64,
    scale: f64,
}

/// The conserved sums of a planes readback, each over the cells in row order: mass, momentum x and y, energy, then each
/// species' mass Σ·X_i in §1.5's order.
fn sums(world: WorldConfig, p: &[f32]) -> Vec<Sum> {
    let n = world.cells();
    let ch = |c: usize, i: usize| f64::from(p[c * n + i]);
    let quantity = |name: &'static str, f: &dyn Fn(usize) -> f64| {
        let v: Vec<f64> = (0..n).map(f).collect();
        Sum { name, total: v.iter().sum(), scale: v.iter().map(|x| x.abs()).sum() }
    };
    let mut out = vec![
        quantity(CHANNELS[0], &|i| ch(0, i)),
        quantity(CHANNELS[1], &|i| ch(1, i)),
        quantity(CHANNELS[2], &|i| ch(2, i)),
        quantity(CHANNELS[3], &|i| ch(3, i)),
    ];
    for c in 4..N_CHANNELS {
        out.push(quantity(CHANNELS[c], &|i| ch(0, i) * ch(c, i)));
    }
    out
}

/// The booked total of the ledger term `name`.
fn term(booked: &[f64; N_TERMS], name: &str) -> f64 {
    booked[TERMS.iter().position(|t| t.name == name).unwrap()]
}

/// A run's sums as the physics conserves them: the state's, plus what left through the edges (`escaped.*`, booked
/// positive), with P8's floor taken back out. The floor's booked rise leaves the state: vacuum_reset is −(the mass the
/// floors added) and floor_added is +(the energy they added); the floor is pure hydrogen, so x_H gives back the same
/// mass. Without this the whole-world run carries the vacuum's floor (M0-V3's arithmetic: ≈ 2.3 × 10⁻³ of mass on
/// sun_disk), which the box run does not.
fn physical(world: WorldConfig, planes: &[f32], booked: &[f64; N_TERMS]) -> Vec<Sum> {
    let mut s = sums(world, planes);
    let (mass, energy) = (term(booked, "mass.vacuum_reset"), term(booked, "energy.floor_added"));
    s[0].total += mass + term(booked, "escaped.mass");
    s[1].total += term(booked, "escaped.mom_x");
    s[2].total += term(booked, "escaped.mom_y");
    s[3].total += term(booked, "escaped.energy") - energy;
    for k in 4..s.len() {
        s[k].total += term(booked, &format!("escaped.mass.{}", s[k].name));
    }
    s[4].total += mass;
    s
}

/// Every sum of `a` equal to `b`'s within the band: all printed first (SR-BOX), then asserted, so one run shows them all.
fn compare(what: &str, a: &[Sum], b: &[Sum]) {
    let mut red = vec![];
    for (x, y) in a.iter().zip(b) {
        let diff = (x.total - y.total).abs();
        let scale = x.scale.max(y.scale);
        println!("SR-BOX {what} {}: {:.12e} vs {:.12e}, |Δ| {diff:.3e} = {:.2e} of scale {scale:.4e}", x.name, x.total, y.total, diff / scale);
        if diff > CONSERVED_TOL * scale {
            red.push(format!("{} ({:.2e} of scale)", x.name, diff / scale));
        }
    }
    assert!(red.is_empty(), "{what}: outside the 1e-6 band: {}", red.join(", "));
}

/// A scene run on the GPU with its switches, edge and physics as the file sets them, under `mode`.
struct SceneRun {
    world: WorldConfig,
    state: State,
    booking: Booking,
    _side: SideFields,
    step: Step,
}

impl SceneRun {
    fn new(gpu: &Gpu, file: &str, mode: BoxMode) -> SceneRun {
        let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes");
        let scene = Scene::load(&dir.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        let (world, physics) = (scene.world, scene.physics().unwrap());
        let elements = Elements::shipped().unwrap();
        let (planes, warnings) = scene.planes(world, &physics, &elements).unwrap();
        assert!(warnings.is_empty(), "{file}: {warnings:?}");
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let side = SideFields::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &physics, &elements);
        let passes = Passes {
            mode,
            gravity: scene.switches.gravity.then_some(&side),
            hydro: scene.switches.hydro.then_some(scene.edge),
        };
        let step = Step::with_passes(&gpu.device, &state, &booking, &eos, &physics, &passes).unwrap();
        SceneRun { world, state, booking, _side: side, step }
    }

    fn run(&self, gpu: &Gpu, steps: u32) {
        self.step.run(&gpu.device, &gpu.queue, steps);
    }

    fn planes(&self, gpu: &Gpu) -> Vec<f32> {
        self.state.readback(&gpu.device, &gpu.queue)
    }

    /// Every term's total over the run so far: the booking is not cleared between steps (the ledger, §2.9).
    fn booked(&self, gpu: &Gpu) -> [f64; N_TERMS] {
        book_totals(self.world, &self.booking.readback(&gpu.device, &gpu.queue))
    }
}

/// The Σ-weighted centre of mass along x, in cells.
fn centroid_x(world: WorldConfig, p: &[f32]) -> f64 {
    let w = world.width as usize;
    let cells = world.cells();
    let (mut m, mut mx) = (0f64, 0f64);
    for (i, &s) in p[..cells].iter().enumerate() {
        m += f64::from(s);
        mx += f64::from(s) * (i % w) as f64;
    }
    mx / m
}

#[test]
fn sun_disk_in_the_box_matches_the_whole_world_at_step_100_and_its_masses_at_2000() {
    let gpu = device();
    let boxed = SceneRun::new(&gpu, "sun_disk.json", BoxMode::Fit);
    let whole = SceneRun::new(&gpu, "sun_disk.json", BoxMode::WholeWorld);
    let world = boxed.world;
    let mut done = 0;
    for at in [EARLY, STEPS] {
        boxed.run(&gpu, at - done);
        whole.run(&gpu, at - done);
        done = at;
        let r = boxed.step.read_box(&gpu.device, &gpu.queue);
        let (bb, bw) = (boxed.booked(&gpu), whole.booked(&gpu));
        println!(
            "SR-BOX sun_disk step {at}: box {:?} FFT {:?}; vacuum_reset {:e} vs {:e}, escaped.mass {:e} vs {:e}",
            rect(&r),
            r.fft,
            term(&bb, "mass.vacuum_reset"),
            term(&bw, "mass.vacuum_reset"),
            term(&bb, "escaped.mass"),
            term(&bw, "escaped.mass"),
        );
        let (a, b) = (physical(world, &boxed.planes(&gpu), &bb), physical(world, &whole.planes(&gpu), &bw));
        if at == EARLY {
            assert!(r.w < world.width && r.h < world.height, "step {at}: the box {:?} is the world's width or height", rect(&r));
            compare(&format!("sun_disk step {at}"), &a, &b);
        } else {
            compare(&format!("sun_disk step {at}, masses"), &masses(a), &masses(b));
        }
    }
}

#[test]
fn a_blob_at_the_box_edge_loses_no_mass_in_bounce_mode() {
    let gpu = device();
    let run = SceneRun::new(&gpu, "box_blob.json", BoxMode::Fit);
    let start = physical(run.world, &run.planes(&gpu), &run.booked(&gpu));
    // The blob moves 0.35 cells per step (the CFL bound is 0.4, §2.3.3): the margin of 8 holds 16 steps of it (5.6 cells)
    // between re-fits; a margin of 0 would let it leave the box inside the first 16. Each 16-step chunk's speed is checked
    // after the masses, so a box that holds the blob back shows its mass first.
    let mut x = centroid_x(run.world, &run.planes(&gpu));
    let mut speeds = vec![];
    for chunk in 0..6 {
        run.run(&gpu, 16);
        let x_now = centroid_x(run.world, &run.planes(&gpu));
        let speed = (x - x_now) / 16.0;
        println!("SR-BOX box_blob chunk {chunk}: centroid at {x_now:.3}, {speed:.4} cells per step leftward");
        speeds.push(speed);
        x = x_now;
    }
    // §5.3 grades the blob's mass: a vacuum reset in its wake takes momentum out unbooked (§2.7 [M0-T19]).
    let end = physical(run.world, &run.planes(&gpu), &run.booked(&gpu));
    compare("box_blob, masses", &masses(end), &masses(start));
    for speed in speeds {
        assert!((0.30..=0.40).contains(&speed), "the blob moves {speed:.4} cells per step, not the 0.35 the margin is graded at");
    }
}
