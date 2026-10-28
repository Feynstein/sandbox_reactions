//! M0-T76 — player edits on the GPU, P0 (contract §1.3.2 P0, §1.12, §2.7, §2.9, §3.1): `Step::queue_edit` and
//! shaders/edit.wgsl, on 64 × 64 to 256 × 256 worlds run through P0, P1, P8 and P9 alone (the other passes would move the
//! gas and blur what an edit did), one step after the edits are queued.
//!
//! Oracles, from the same f32 inputs the GPU read, in f64 (`deposit_ref`, `eps_th`): a deposit keeps the gas it lands on —
//! Σ ε_th is added to, the cold energy follows the new Σ and composition, the momentum stays — and heat and cool scale ε_th
//! and floor it. The booked terms are graded against what the E and Σ planes changed by.
//!
//! Tolerances, fixed before the first run (R8): booked terms against the change in the state, summed in f64, within 10⁻⁶ of
//! the field's total (BOOK_TOL; the pass books the stored difference, so the balance is exact up to P8's own terms, which
//! are in the balance); a value read through the u(x) table — E of a deposit, a drop's cells — within 2 × 10⁻⁵ relative
//! of the f64 twin (TABLE_TOL, tests/gpu/floors.rs's reasoning); a temperature read back from the state, E/Σ − ½|u|² −
//! ε_cold, within 5 × 10⁻⁵ of the cell's stored specific energy E/Σ (THERMAL_TOL: what f32 holds of it — the subtraction
//! cancels the cold share, which is most of E/Σ in cold hydrogen at Σ = 0.5). What an edit does not touch: bit-identical.

use super::device;
use sr_engine::gpu::Gpu;
use sr_engine::scene::{Object, Scene, UNIT_MASS};
use sr_engine::state::{book_totals, Booking, SideFields, State, WorldConfig, N_CHANNELS, TERMS};
use sr_engine::step::boxfit::{BoxMode, BoxReport};
use sr_engine::step::edit::{Edit, EditError, MAX_EDITS};
use sr_engine::step::latch::FramePlan;
use sr_engine::step::sinks::SinkSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_engine::dump::Sink;
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const BOOK_TOL: f64 = 1e-6;
const TABLE_TOL: f64 = 2e-5;
const THERMAL_TOL: f64 = 5e-5;

const H: [f64; N_SPECIES] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const HE_MIX: [f64; N_SPECIES] = [0.7, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const C_O: [f64; N_SPECIES] = [0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const SP_H: usize = 0;
const SP_C: usize = 2;

struct Constants {
    physics: Physics,
    elements: Elements,
    eos: Eos,
}

fn constants() -> Constants {
    let physics = Physics::shipped().unwrap();
    let eos = Eos::from_physics(&physics).unwrap();
    Constants { physics, elements: Elements::shipped().unwrap(), eos }
}

impl Constants {
    fn get(&self, key: &str) -> f64 {
        self.physics.get(key).unwrap()
    }

    fn inv_mu(&self, x: &[f64; N_SPECIES]) -> f64 {
        self.elements.composition(x).inv_mu
    }
}

#[derive(Clone, Copy, Debug)]
struct Cell {
    sigma: f64,
    mom: [f64; 2],
    energy: f64,
    x: [f64; N_SPECIES],
}

fn index(world: WorldConfig, x: u32, y: u32) -> usize {
    (y * world.width + x) as usize
}

fn read(p: &[f32], world: WorldConfig, x: u32, y: u32) -> Cell {
    let (n, i) = (world.cells(), index(world, x, y));
    let f = |ch: usize| f64::from(p[ch * n + i]);
    Cell { sigma: f(0), mom: [f(1), f(2)], energy: f(3), x: std::array::from_fn(|s| f(4 + s)) }
}

fn bits(p: &[f32], world: WorldConfig, x: u32, y: u32, ch: usize) -> u32 {
    p[ch * world.cells() + index(world, x, y)].to_bits()
}

/// Gas of `sigma`, temperature `t`, fractions `comp` and velocity `v` on one cell, as the scene writes it (f64 → f32).
#[allow(clippy::too_many_arguments)]
fn put(p: &mut [f32], world: WorldConfig, c: &Constants, (x, y): (u32, u32), sigma: f64, t: f64, comp: &[f64; N_SPECIES], v: [f64; 2]) {
    let (n, i) = (world.cells(), index(world, x, y));
    let k = c.elements.composition(comp);
    let eps = Eos::eps_th_of_t(t, 1.0 / k.inv_mu) + c.eos.eps_cold(sigma, k.y_e, comp[N_SPECIES - 1]) + 0.5 * (v[0] * v[0] + v[1] * v[1]);
    p[i] = sigma as f32;
    p[n + i] = (sigma * v[0]) as f32;
    p[2 * n + i] = (sigma * v[1]) as f32;
    p[3 * n + i] = (sigma * eps) as f32;
    for (s, &xs) in comp.iter().enumerate() {
        p[(4 + s) * n + i] = xs as f32;
    }
}

/// Every cell in the floor state: Σ_floor of hydrogen at T_floor, at rest.
fn floor_field(world: WorldConfig, c: &Constants) -> Vec<f32> {
    let mut p = vec![0f32; N_CHANNELS * world.cells()];
    for y in 0..world.height {
        for x in 0..world.width {
            put(&mut p, world, c, (x, y), c.get("sigma_floor"), c.get("t_floor"), &H, [0.0, 0.0]);
        }
    }
    p
}

#[allow(clippy::too_many_arguments)]
fn disk(p: &mut [f32], world: WorldConfig, c: &Constants, (cx, cy): (i32, i32), r: i32, sigma: f64, t: f64, comp: &[f64; N_SPECIES], v: [f64; 2]) {
    for y in 0..world.height as i32 {
        for x in 0..world.width as i32 {
            if (x - cx).pow(2) + (y - cy).pow(2) <= r * r {
                put(p, world, c, (x as u32, y as u32), sigma, t, comp, v);
            }
        }
    }
}

fn in_disk(x: u32, y: u32, (cx, cy): (u32, u32), r: u32) -> bool {
    let (dx, dy) = (i64::from(x) - i64::from(cx), i64::from(y) - i64::from(cy));
    dx * dx + dy * dy <= i64::from(r * r)
}

/// ε_th = E/Σ − ½|u|² − ε_cold of the cell's state, in f64.
fn eps_th(c: &Constants, cell: &Cell) -> f64 {
    let k = c.elements.composition(&cell.x);
    cell.energy / cell.sigma - 0.5 * (cell.mom[0].powi(2) + cell.mom[1].powi(2)) / cell.sigma.powi(2) - c.eos.eps_cold(cell.sigma, k.y_e, cell.x[N_SPECIES - 1])
}

/// The deposit of `ds` of fractions `xp` at specific thermal energy `eps_p`, at rest, on `cell` (§1.12).
fn deposit_ref(c: &Constants, cell: &Cell, ds: f64, xp: &[f64; N_SPECIES], eps_p: f64) -> Cell {
    let eth = cell.sigma * eps_th(c, cell) + ds * eps_p;
    let s1 = cell.sigma + ds;
    let x1: [f64; N_SPECIES] = std::array::from_fn(|i| (cell.sigma * cell.x[i] + ds * xp[i]) / s1);
    let k = c.elements.composition(&x1);
    let kinetic = 0.5 * (cell.mom[0].powi(2) + cell.mom[1].powi(2)) / s1;
    Cell { sigma: s1, mom: cell.mom, energy: eth + s1 * c.eos.eps_cold(s1, k.y_e, x1[N_SPECIES - 1]) + kinetic, x: x1 }
}

fn one_hot(species: usize) -> [f64; N_SPECIES] {
    std::array::from_fn(|i| f64::from(u8::from(i == species)))
}

fn total(p: &[f32], world: WorldConfig, ch: usize) -> f64 {
    let n = world.cells();
    p[ch * n..(ch + 1) * n].iter().map(|&v| f64::from(v)).sum()
}

fn close(got: f64, want: f64, tol: f64) -> bool {
    (got - want).abs() <= tol * want.abs()
}

/// ε_th of `got` within THERMAL_TOL of `want`, the scale the cell's stored specific energy E/Σ.
fn thermal_close(got: &Cell, want: f64, c: &Constants) -> bool {
    (eps_th(c, got) - want).abs() <= THERMAL_TOL * (got.energy / got.sigma).abs()
}

/// A world run through P0, P1, P8 and P9 with the edit pass on (or off).
struct Rig {
    gpu: Gpu,
    world: WorldConfig,
    state: State,
    booking: Booking,
    step: Step,
}

impl Rig {
    fn new(c: &Constants, world: WorldConfig, planes: &[f32], edits: bool, sinks: bool) -> Rig {
        let gpu = device();
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, &c.physics, &c.elements);
        let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: None };
        let side = SideFields::new(&gpu.device, world).unwrap();
        let setup = SinkSetup { side: &side };
        let step = Step::with_sinks(&gpu.device, &state, &booking, &eos, &c.physics, &passes, None, None, sinks.then_some(&setup)).unwrap();
        let step = if edits { step.with_edits(&gpu.device, &state, &booking, &eos, &c.physics) } else { step };
        Rig { gpu, world, state, booking, step }
    }

    fn queue(&self, edit: Edit) {
        self.step.queue_edit(&self.gpu.queue, &edit).unwrap();
    }

    fn run(&self, steps: u32) {
        self.step.run(&self.gpu.device, &self.gpu.queue, steps);
    }

    fn planes(&self) -> Vec<f32> {
        self.state.readback(&self.gpu.device, &self.gpu.queue)
    }

    fn booked(&self, name: &str) -> f64 {
        let totals = book_totals(self.world, &self.booking.readback(&self.gpu.device, &self.gpu.queue));
        totals[TERMS.iter().position(|t| t.name == name).unwrap()]
    }

    fn the_box(&self) -> BoxReport {
        self.step.read_box(&self.gpu.device, &self.gpu.queue)
    }
}

/// The mass and energy balances of one step with only P0 and P8 moving matter: the state's change equals the booked terms.
/// Returns (mass change, energy change).
fn assert_balanced(rig: &Rig, before: &[f32], after: &[f32], mass_terms: f64, energy_terms: &[&str], what: &str) -> (f64, f64) {
    let world = rig.world;
    let d_mass = total(after, world, 0) - total(before, world, 0);
    let d_energy = total(after, world, 3) - total(before, world, 3);
    let booked_energy: f64 = energy_terms.iter().map(|t| rig.booked(t)).sum::<f64>() + rig.booked("energy.floor_added");
    let booked_mass = mass_terms - rig.booked("mass.vacuum_reset");
    let (mass_scale, energy_scale) = (total(after, world, 0).abs().max(total(before, world, 0).abs()), total(after, world, 3).abs().max(total(before, world, 3).abs()));
    assert!((d_mass - booked_mass).abs() <= BOOK_TOL * mass_scale, "{what}: Δmass {d_mass} against the booked {booked_mass}");
    assert!((d_energy - booked_energy).abs() <= BOOK_TOL * energy_scale, "{what}: ΔE {d_energy} against the booked {booked_energy}");
    (d_mass, d_energy)
}

const W64: WorldConfig = WorldConfig { width: 64, height: 64 };

/// A warm moving cloud: Σ = 0.5 of H/He at T = 2, velocity (0.3, −0.2), a disk of 20 cells about (32, 32).
fn warm_gas(c: &Constants) -> Vec<f32> {
    let mut p = floor_field(W64, c);
    disk(&mut p, W64, c, (32, 32), 20, 0.5, 2.0, &HE_MIX, [0.3, -0.2]);
    p
}

#[test]
fn paint_adds_paint_sigma_at_rest_mixed_by_mass_and_books_it() {
    let c = constants();
    let before = warm_gas(&c);
    let rig = Rig::new(&c, W64, &before, true, false);
    rig.queue(Edit::Paint { cell: [32, 32], radius: 6, species: SP_C });
    rig.run(1);
    let after = rig.planes();
    let (ds, t_paint) = (c.get("paint_sigma"), c.get("paint_t"));
    let eps_p = t_paint * c.inv_mu(&one_hot(SP_C));
    let mut painted = 0;
    for y in 0..W64.height {
        for x in 0..W64.width {
            let (b, a) = (read(&before, W64, x, y), read(&after, W64, x, y));
            if !in_disk(x, y, (32, 32), 6) {
                for ch in [0, 1, 2] {
                    assert_eq!(bits(&before, W64, x, y, ch), bits(&after, W64, x, y, ch), "({x}, {y}) channel {ch} outside the brush");
                }
                continue;
            }
            painted += 1;
            // The adversarial case: the momentum density is the cell's old one, bit for bit — not Σ' times the old velocity.
            for ch in [1, 2] {
                assert_eq!(bits(&before, W64, x, y, ch), bits(&after, W64, x, y, ch), "({x}, {y}) momentum channel {ch}");
            }
            assert!(close(a.sigma, b.sigma + ds, 1e-6), "({x}, {y}) Σ {} after {}", b.sigma, a.sigma);
            let want = deposit_ref(&c, &b, ds, &one_hot(SP_C), eps_p);
            assert!(close(a.energy, want.energy, TABLE_TOL), "({x}, {y}) E {} against {}", a.energy, want.energy);
            assert!(close(a.x[SP_C], ds / want.sigma, 1e-5), "({x}, {y}) x_C {} against {}", a.x[SP_C], ds / want.sigma);
            assert!((a.x.iter().sum::<f64>() - 1.0).abs() < 1e-6, "({x}, {y}) fractions sum to 1");
        }
    }
    assert_eq!(painted, 113, "a radius-6 brush covers 113 cells");
    let (d_mass, _) = assert_balanced(&rig, &before, &after, rig.booked("mass.painted"), &["energy.tools.painted"], "paint");
    assert!(close(rig.booked("mass.painted"), 113.0 * ds, 1e-4), "painted {} against 113 × paint_sigma", rig.booked("mass.painted"));
    assert!(d_mass > 0.0);
    let momentum_change = (total(&after, W64, 1) - total(&before, W64, 1)).abs() + (total(&after, W64, 2) - total(&before, W64, 2)).abs();
    assert!(momentum_change < 1e-9, "momentum changed by {momentum_change}");
}

#[test]
fn erase_writes_the_floor_state_and_books_what_left() {
    let c = constants();
    let before = warm_gas(&c);
    let rig = Rig::new(&c, W64, &before, true, false);
    rig.queue(Edit::Erase { cell: [32, 32], radius: 6 });
    rig.run(1);
    let after = rig.planes();
    let floor = floor_field(W64, &c);
    let mut erased = 0.0;
    for y in 0..W64.height {
        for x in 0..W64.width {
            if in_disk(x, y, (32, 32), 6) {
                let (a, f) = (read(&after, W64, x, y), read(&floor, W64, x, y));
                assert!(close(a.sigma, f.sigma, 1e-6) && a.mom == [0.0, 0.0] && close(a.energy, f.energy, 1e-5), "({x}, {y}) {a:?} against the floor state {f:?}");
                assert!(a.x[0] == 1.0 && a.x[1..].iter().all(|&v| v == 0.0), "({x}, {y}) pure hydrogen");
                erased += read(&before, W64, x, y).sigma - f.sigma;
            } else {
                for ch in (0..N_CHANNELS).filter(|&ch| ch != 3) {
                    assert_eq!(bits(&before, W64, x, y, ch), bits(&after, W64, x, y, ch), "({x}, {y}) channel {ch} outside the brush");
                }
                // E: P8 re-evaluates a floor cell's energy in f32, a unit in the last place from the upload's f64.
                assert!(close(read(&after, W64, x, y).energy, read(&before, W64, x, y).energy, 1e-6), "({x}, {y}) E outside the brush");
            }
        }
    }
    assert!(close(rig.booked("mass.erased"), erased, BOOK_TOL), "erased {} against {erased}", rig.booked("mass.erased"));
    assert_balanced(&rig, &before, &after, -rig.booked("mass.erased"), &["energy.tools.erased"], "erase");
    assert!(rig.booked("energy.tools.erased") < 0.0, "erasing hot gas books a negative tool energy");
}

#[test]
fn heat_and_cool_scale_the_thermal_energy_floor_it_and_leave_vacuum_alone() {
    let c = constants();
    let mut before = warm_gas(&c);
    put(&mut before, W64, &c, (60, 4), 0.5, 0.051, &H, [0.0, 0.0]);
    let rig = Rig::new(&c, W64, &before, true, false);
    // The heat brush reaches past the cloud's edge into vacuum (the cloud ends 20 cells from (32, 32)).
    rig.queue(Edit::Heat { cell: [50, 32], radius: 6 });
    rig.queue(Edit::Cool { cell: [20, 32], radius: 4 });
    rig.queue(Edit::Cool { cell: [60, 4], radius: 1 });
    rig.run(1);
    let after = rig.planes();
    let (hot, cold) = (c.get("heat_factor"), c.get("cool_factor"));
    for y in 0..W64.height {
        for x in 0..W64.width {
            let (b, a) = (read(&before, W64, x, y), read(&after, W64, x, y));
            let factor = if in_disk(x, y, (50, 32), 6) && b.sigma >= c.get("sigma_vac") {
                hot
            } else if in_disk(x, y, (20, 32), 4) {
                cold
            } else {
                1.0
            };
            for ch in [0, 1, 2] {
                assert_eq!(bits(&before, W64, x, y, ch), bits(&after, W64, x, y, ch), "({x}, {y}) channel {ch}: heat and cool leave Σ and the momentum");
            }
            if factor == 1.0 && !((x, y) == (60, 4)) {
                assert!(close(a.energy, b.energy, 1e-6), "({x}, {y}) untouched E {} became {}", b.energy, a.energy);
                continue;
            }
            let floor = c.get("t_floor") * c.inv_mu(&b.x);
            let want = if (x, y) == (60, 4) { floor } else { (eps_th(&c, &b) * factor).max(floor) };
            assert!(thermal_close(&a, want, &c), "({x}, {y}) ε_th {} against {want}", eps_th(&c, &a));
        }
    }
    assert!(rig.booked("energy.tools.heated") > 0.0 && rig.booked("energy.tools.cooled") < 0.0, "heat adds energy, cool takes it");
    assert_balanced(&rig, &before, &after, 0.0, &["energy.tools.heated", "energy.tools.cooled"], "heat and cool");
}

/// B7: the pass has no branch on the cell's density or temperature. A hot dense cloud (Σ = 300 of carbon and oxygen at
/// T = 3 × 10⁴, the cold pressure on) takes the same paint and heat as the warm gas does.
#[test]
fn an_edit_on_a_hot_dense_field_acts_exactly_as_on_gas() {
    let c = constants();
    let mut before = floor_field(W64, &c);
    disk(&mut before, W64, &c, (32, 32), 12, 300.0, 3.0e4, &C_O, [1.5, 0.5]);
    disk(&mut before, W64, &c, (56, 56), 4, 0.5, 2.0, &HE_MIX, [0.3, -0.2]);
    let rig = Rig::new(&c, W64, &before, true, false);
    rig.queue(Edit::Paint { cell: [32, 32], radius: 4, species: SP_H });
    rig.queue(Edit::Heat { cell: [32, 32], radius: 2 });
    rig.queue(Edit::Paint { cell: [56, 56], radius: 2, species: SP_H });
    rig.run(1);
    let after = rig.planes();
    let ds = c.get("paint_sigma");
    let eps_p = c.get("paint_t") * c.inv_mu(&one_hot(SP_H));
    for (cell, label) in [((32u32, 32u32), "hot dense, painted and heated"), ((36, 32), "hot dense, painted"), ((57, 56), "warm gas, painted")] {
        let (b, a) = (read(&before, W64, cell.0, cell.1), read(&after, W64, cell.0, cell.1));
        let mut want = deposit_ref(&c, &b, ds, &one_hot(SP_H), eps_p);
        if label.ends_with("heated") {
            want.energy += want.sigma * eps_th(&c, &want) * (c.get("heat_factor") - 1.0);
        }
        // f32 holds Σ = 300 to 3 × 10⁻⁵: the mass a paint adds is Σ' − Σ as stored.
        assert!((a.sigma - b.sigma - ds).abs() <= f64::from(f32::EPSILON) * a.sigma, "{label}: ΔΣ {}", a.sigma - b.sigma);
        assert_eq!(a.mom.map(f64::to_bits), b.mom.map(f64::to_bits), "{label}: momentum density unchanged");
        assert!(thermal_close(&a, eps_th(&c, &want), &c), "{label}: ε_th {} against {}", eps_th(&c, &a), eps_th(&c, &want));
        assert!(close(a.energy, want.energy, TABLE_TOL), "{label}: E {} against {}", a.energy, want.energy);
    }
    assert_balanced(&rig, &before, &after, rig.booked("mass.painted"), &["energy.tools.painted", "energy.tools.heated"], "hot dense field");
}

#[test]
fn a_preset_drop_writes_the_cloud_centred_on_the_cell_and_books_it() {
    let c = constants();
    let world = WorldConfig::new(256, 256).unwrap();
    let before = floor_field(world, &c);
    let rig = Rig::new(&c, world, &before, true, false);
    let drop = Edit::drop_preset("sun", UNIT_MASS, [128, 128], world, &c.physics, &c.elements).unwrap();
    rig.queue(drop);
    rig.run(1);
    let after = rig.planes();

    // The scene's own cloud at the same centre (the middle of cell (128, 128)).
    let mut scene = Scene::preset_mass("sun", UNIT_MASS, world).unwrap();
    scene.objects = vec![Object::Preset { preset: "sun".into(), x: 128.5, y: 128.5, mass: UNIT_MASS }];
    let (want, _) = scene.planes(world, &c.physics, &c.elements).unwrap();
    let n = world.cells();
    let mut cells = 0;
    for i in 0..n {
        let (s, w) = (f64::from(after[i]) - c.get("sigma_floor"), f64::from(want[i]));
        if w > 0.0 {
            cells += 1;
            assert!((s - w).abs() <= 1e-6 * w + 2e-7, "cell {i}: Σ {s} against the scene's {w}");
        }
        if w >= 1e-3 {
            let (a, b) = (f64::from(after[3 * n + i]), f64::from(want[3 * n + i]));
            assert!(close(a, b, TABLE_TOL), "cell {i}: E {a} against the scene's {b}");
            assert_eq!((after[n + i], after[2 * n + i]), (0.0, 0.0), "cell {i}: at rest");
            assert!((f64::from(after[4 * n + i]) - 0.7346).abs() < 1e-5, "cell {i}: the Sun's hydrogen");
        }
    }
    assert!(cells > 4000, "the cloud covers {cells} cells");
    let dropped = rig.booked("mass.preset_dropped");
    assert!(close(dropped, UNIT_MASS, 1e-5), "preset_dropped {dropped} against the cloud's mass {UNIT_MASS}");
    assert_balanced(&rig, &before, &after, dropped, &["energy.tools.preset"], "preset drop");
    let b = rig.the_box();
    assert!(b.w >= 80 && b.h >= 80 && b.x0 <= 88 && b.y0 <= 88 && b.x0 + b.w >= 168, "the box grew to the cloud: {b:?}");
}

#[test]
fn a_drop_that_would_leave_the_world_is_refused_with_no_room() {
    let c = constants();
    let world = WorldConfig::new(256, 256).unwrap();
    let sun = |cell: [u32; 2]| Edit::drop_preset("sun", UNIT_MASS, cell, world, &c.physics, &c.elements);
    for cell in [[30, 128], [128, 30], [226, 128], [128, 226], [5, 5], [250, 250]] {
        let err = sun(cell).expect_err("the cloud of radius 40 does not fit");
        assert_eq!(err, EditError::NoRoom, "{cell:?}");
        assert_eq!(err.message_key(), Some("presets.no_room"));
    }
    for cell in [[41, 128], [128, 41], [214, 128], [128, 214]] {
        assert!(sun(cell).is_ok(), "{cell:?} leaves the cloud room");
    }
    // A refused drop queues nothing: the world is as it was after a step.
    let before = floor_field(world, &c);
    let rig = Rig::new(&c, world, &before, true, false);
    let _ = sun([30, 128]);
    rig.run(1);
    let after = rig.planes();
    assert_eq!(rig.booked("mass.preset_dropped"), 0.0);
    assert_eq!(total(&before, world, 0), total(&after, world, 0));
    assert_eq!(Edit::drop_preset("sun", f64::NAN, [128, 128], world, &c.physics, &c.elements).map(|_| ()).unwrap_err().message_key(), None);
}

fn box_oracle(p: &[f32], world: WorldConfig, sigma_vac: f32) -> [u32; 4] {
    let (w, h) = (i64::from(world.width), i64::from(world.height));
    let mut b: Option<[i64; 4]> = None;
    for (i, _) in p[..world.cells()].iter().enumerate().filter(|(_, s)| **s >= sigma_vac) {
        let (x, y) = (i as i64 % w, i as i64 / w);
        let o = b.unwrap_or([x, y, x + 1, y + 1]);
        b = Some([o[0].min(x), o[1].min(y), o[2].max(x + 1), o[3].max(y + 1)]);
    }
    let Some(b) = b else { return [0; 4] };
    let lo = |v: i64| ((v - 8).div_euclid(8) * 8).max(0);
    let hi = |v: i64, n: i64| ((v + 8 + 7).div_euclid(8) * 8).min(n);
    let (x0, y0) = (lo(b[0]), lo(b[1]));
    [x0, y0, hi(b[2], w) - x0, hi(b[3], h) - y0].map(|v| v as u32)
}

#[test]
fn the_box_grows_to_an_edit_outside_it_and_only_then() {
    let c = constants();
    let world = WorldConfig::new(128, 128).unwrap();
    let mut before = floor_field(world, &c);
    disk(&mut before, world, &c, (20, 20), 6, 1.0, 1.0, &HE_MIX, [0.0, 0.0]);
    let vac = c.get("sigma_vac") as f32;
    let rig = Rig::new(&c, world, &before, true, false);
    rig.run(1);
    let settled = rig.the_box();
    assert_eq!([settled.x0, settled.y0, settled.w, settled.h], box_oracle(&before, world, vac));
    // An edit inside the box: the heat, and an erase, leave it and the fit count alone.
    rig.queue(Edit::Heat { cell: [20, 20], radius: 3 });
    rig.queue(Edit::Erase { cell: [22, 20], radius: 1 });
    rig.run(1);
    let inside = rig.the_box();
    assert_eq!((inside.x0, inside.y0, inside.w, inside.h, inside.fits), (settled.x0, settled.y0, settled.w, settled.h, settled.fits), "an edit inside the box");
    // A paint far from it: one fit more, over the painted gas.
    rig.queue(Edit::Paint { cell: [100, 100], radius: 5, species: SP_H });
    rig.run(1);
    let grown = rig.the_box();
    assert_eq!(grown.fits, settled.fits + 1, "the edit outside the box fit it once");
    assert_eq!([grown.x0, grown.y0, grown.w, grown.h], box_oracle(&rig.planes(), world, vac));
    assert!(grown.x0 + grown.w >= 106 && grown.y0 + grown.h >= 106, "the box reaches the painted gas: {grown:?}");
}

#[test]
fn clear_restores_the_floor_state_removes_the_sinks_and_books_cleared() {
    let c = constants();
    let before = warm_gas(&c);
    let rig = Rig::new(&c, W64, &before, true, true);
    let sinks = [Sink { x: 30.5, y: 30.5, vx: 0.0, vy: 0.0, m: 5.0 }, Sink { x: 40.5, y: 20.5, vx: 0.1, vy: 0.0, m: 7.0 }];
    rig.step.set_sinks(&rig.gpu.queue, &sinks).unwrap();
    assert_eq!(rig.step.read_sinks(&rig.gpu.device, &rig.gpu.queue).unwrap().sinks.len(), 2);
    assert!(Edit::Clear.restarts_ledger() && !Edit::Erase { cell: [0, 0], radius: 1 }.restarts_ledger());
    rig.queue(Edit::Clear);
    rig.run(1);
    let after = rig.planes();
    let floor = floor_field(W64, &c);
    for y in 0..W64.height {
        for x in 0..W64.width {
            let (a, f) = (read(&after, W64, x, y), read(&floor, W64, x, y));
            assert!(close(a.sigma, f.sigma, 1e-6) && a.mom == [0.0, 0.0] && close(a.energy, f.energy, 1e-5) && a.x[0] == 1.0, "({x}, {y}) {a:?}");
        }
    }
    assert_eq!(rig.step.read_sinks(&rig.gpu.device, &rig.gpu.queue).unwrap().sinks.len(), 0, "the sinks are removed");
    let cleared = total(&before, W64, 0) - total(&floor, W64, 0);
    assert!(close(rig.booked("mass.cleared"), cleared, BOOK_TOL), "cleared {} against {cleared}", rig.booked("mass.cleared"));
    assert_balanced(&rig, &before, &after, -rig.booked("mass.cleared"), &["energy.tools.cleared"], "clear");
}

#[test]
fn queued_edits_apply_once_in_order_and_the_queue_is_bounded() {
    let c = constants();
    let before = floor_field(W64, &c);
    let rig = Rig::new(&c, W64, &before, true, false);
    let ds = c.get("paint_sigma");
    let sigma_at = |p: &[f32], x, y| read(p, W64, x, y).sigma;
    // Paint then erase leaves the floor; erase then paint leaves the paint; two paints add twice.
    rig.queue(Edit::Paint { cell: [10, 10], radius: 1, species: SP_H });
    rig.queue(Edit::Erase { cell: [10, 10], radius: 1 });
    rig.queue(Edit::Erase { cell: [20, 10], radius: 1 });
    rig.queue(Edit::Paint { cell: [20, 10], radius: 1, species: SP_H });
    rig.queue(Edit::Paint { cell: [30, 10], radius: 1, species: SP_H });
    rig.queue(Edit::Paint { cell: [30, 10], radius: 1, species: SP_H });
    rig.run(1);
    let first = rig.planes();
    assert!(sigma_at(&first, 10, 10) < 1e-6, "paint, then erase: {}", sigma_at(&first, 10, 10));
    assert!(close(sigma_at(&first, 20, 10), ds, 1e-4), "erase, then paint: {}", sigma_at(&first, 20, 10));
    assert!(close(sigma_at(&first, 30, 10), 2.0 * ds, 1e-4), "two paints: {}", sigma_at(&first, 30, 10));
    // The next steps apply nothing again.
    rig.run(3);
    let later = rig.planes();
    assert_eq!(sigma_at(&later, 30, 10).to_bits(), sigma_at(&first, 30, 10).to_bits(), "the queue was emptied");
    let painted = rig.booked("mass.painted");
    assert!(close(painted, 4.0 * 5.0 * ds, 1e-3), "painted {painted}: four paints of five cells each");

    // A step has gone by: a fresh queue of MAX_EDITS takes them all, and one more is refused.
    for k in 0..MAX_EDITS {
        rig.step.queue_edit(&rig.gpu.queue, &Edit::Paint { cell: [50, 4 + 6 * k as u32], radius: 1, species: SP_H }).unwrap();
    }
    let full = rig.step.queue_edit(&rig.gpu.queue, &Edit::Paint { cell: [56, 4], radius: 1, species: SP_H });
    assert_eq!(full, Err(EditError::QueueFull));
    rig.run(1);
    assert!(close(sigma_at(&rig.planes(), 50, 46), ds, 1e-4), "the eighth edit applied");
    // Refusals that never reach the GPU.
    let bad = |e: Edit| matches!(rig.step.queue_edit(&rig.gpu.queue, &e), Err(EditError::Invalid(_)));
    assert!(bad(Edit::Paint { cell: [10, 10], radius: 0, species: SP_H }));
    assert!(bad(Edit::Erase { cell: [10, 10], radius: 41 }));
    assert!(bad(Edit::Heat { cell: [64, 10], radius: 3 }));
    assert!(bad(Edit::Paint { cell: [10, 10], radius: 3, species: N_SPECIES }));
    let plain = Rig::new(&c, W64, &before, false, false);
    assert_eq!(plain.step.queue_edit(&plain.gpu.queue, &Edit::Clear), Err(EditError::NotBuilt));
}

/// A frame of steps under the latch (§1.8.3, §1.8.5): the edit queued before it lands in its first step, once — the later
/// steps of the frame and the frames after it apply nothing again — and the box is re-fit by the paint outside it.
#[test]
fn a_frame_applies_the_queued_edit_once_in_its_first_step() {
    let c = constants();
    let world = WorldConfig::new(128, 128).unwrap();
    let mut before = floor_field(world, &c);
    disk(&mut before, world, &c, (20, 20), 6, 1.0, 1.0, &HE_MIX, [0.0, 0.0]);
    let rig = Rig::new(&c, world, &before, true, false);
    rig.run(1);
    rig.queue(Edit::Paint { cell: [100, 100], radius: 3, species: SP_H });
    for _ in 0..2 {
        let mut encoder = rig.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        rig.step.encode_frame(&FramePlan { steps: 5, slow_down: false }, &mut encoder);
        rig.gpu.queue.submit([encoder.finish()]);
        rig.gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        assert_eq!(rig.step.poll(&rig.gpu.device).len(), 1);
    }
    let after = rig.planes();
    let ds = c.get("paint_sigma");
    assert!(close(read(&after, world, 100, 100).sigma, ds, 1e-4), "painted once: {}", read(&after, world, 100, 100).sigma);
    let cells = (0..128u32).flat_map(|y| (0..128u32).map(move |x| (x, y))).filter(|&(x, y)| in_disk(x, y, (100, 100), 3)).count();
    assert!(close(rig.booked("mass.painted"), cells as f64 * ds, 1e-3), "painted {} for {cells} cells", rig.booked("mass.painted"));
    let b = rig.the_box();
    assert!(b.x0 + b.w >= 103 && b.y0 + b.h >= 103, "the box reaches the painted gas: {b:?}");
}
