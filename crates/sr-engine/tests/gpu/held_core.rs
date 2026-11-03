//! M0-D71 — the held core on the CPU twin (contract §2.7 [M0-TG2], §2.3.4 [M0-TG2], §1.4.2 [M0-TG2], §2.9's `held`, §5.1's
//! G-REST; M0-R11's H, reports/unresolved_remnant.md). P8's last act holds a cell with Σ ≥ σ_hold and ε_th ≤ η_hold ε_cold
//! whose N_Fe rate is 0, and releases it past 2η_hold, under ½σ_hold or at N_Fe's gate; P4 leaves a held cell as it is
//! behind reflecting walls (HLL against the mirror, no mass, energy or species across, no work); the momentum the hold
//! takes from the grid is booked `held`. One-way inflow (§2.3.4's item 2) is not built: it took the M 900 disk's floor to
//! 392.6 (2.26 without) and moved to M0-D71f (§0.5 [M0-D71], the lead at M0-D71). The twin's cases touch no adapter (R12).
//!
//! - **The SCF disks** (G-REST's twin half): `calibrate::cold_disk_scene`'s cold C/O disk at M 900 and 1,800 (the shipped
//!   K's, T_floor, 64 × 64, bounce, gravity and hydro), 10,000 steps: every held cell's momentum exactly 0, floor_added over
//!   steps 1,000–10,000 ≤ 10, KE(10,000) ≤ 2 × KE(1,000) (the ruled bounds; M0-R11 measured floor 2.1 and 0, KE 155 → 154
//!   and 0 → 0). The held count is printed against M0-R11's 16 and 12 — a rule that holds too much also reads u = 0 (the
//!   adversarial line). Grid + held momentum is printed against Σm|u|, not graded (§0.5 [M0-D71]: source-term gravity
//!   does not conserve momentum — today's tree drifts 1.4 × 10⁻² and 2.2 × 10⁻⁵ of Σm|u| on these disks, and at M 1,800
//!   every gas cell is held or vacuum, Σm|u| = 0); the booking is graded where it holds by construction, the strip cases.
//! - **The wall**: a free cell moving into a held column — the held cells' Σ, E and species bit for bit, their momentum 0,
//!   the free cell's normal momentum turned back (§2.8's Π-alone flux pushes it on), grid + held momentum kept to 10⁻¹².
//! - **Nothing crosses either way**: dense gas driven into a held cell, whose ordinary HLL flux would carry mass in, and
//!   gas drawn away from it — the held cells bit for bit, grid + held momentum kept to 10⁻¹².
//! - **Gravity's kick booked**: the strip at rest in a uniform g — grid + held momentum gains Δt·g·M to 10⁻¹² (a held
//!   cell's missed or reversed Σg·Δt is its share of M), the held cells' Σ, E and species bit for bit.
//! - **The hysteresis**: held at η, kept to 2η, released past it and under ½σ_hold; never held under σ_hold or past η; a
//!   newly held cell's momentum zeroed and booked, its E as it was.
//! - **Item 1**: a held iron cell released once Y_eΣ passes Σ_N; cold iron past the gate never held, C/O there held.
//!
//! Plant `held-wall-pressure-only`: the held wall takes §2.8's bounce flux, Π alone (M0-R11: the M 900 disk ran to KE 5 × 10⁶).

use sr_engine::calibrate::{cold_disk_scene, Matter};
use sr_physics::eos::Eos;
use sr_physics::reference::floors::{Floors, ETA_HOLD, SIGMA_HOLD};
use sr_physics::reference::hydro::{Edge, Hydro, State, COMPONENTS};
use sr_physics::reference::{Switches, Twin};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// G-REST's run and its window's start (§5.1 [M0-TG2]).
const STEPS: u64 = 10_000;
const FROM: u64 = 1_000;
/// G-REST's ruled bounds: floor_added over the window, KE's growth over it.
const FLOOR_BOUND: f64 = 10.0;
const KE_GROWTH: f64 = 2.0;
/// The strip cases' momentum band, relative: f64 sums of a dozen terms.
const STRIP_BAND: f64 = 1e-12;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn physics() -> Physics {
    Physics::shipped().unwrap()
}

/// Mass fractions of the named species.
fn fractions(parts: &[(&str, f64)]) -> [f64; N_SPECIES] {
    let elements = elements();
    let mut x = [0.0; N_SPECIES];
    for &(key, share) in parts {
        x[elements.index_of(key).unwrap_or_else(|| panic!("elements.json has no {key}"))] = share;
    }
    x
}

/// One cell's channels: Σ, momentum, E, fractions.
#[derive(Clone, Copy)]
struct Cell {
    sigma: f64,
    mom: [f64; 2],
    energy: f64,
    x: [f64; N_SPECIES],
}

/// A `width` × `height` state of `cells` (row-major).
fn state(width: usize, height: usize, cells: &[Cell]) -> State {
    let n = width * height;
    assert_eq!(cells.len(), n);
    let mut planes = vec![0.0; COMPONENTS * n];
    for (i, c) in cells.iter().enumerate() {
        planes[i] = c.sigma;
        planes[n + i] = c.mom[0];
        planes[2 * n + i] = c.mom[1];
        planes[3 * n + i] = c.energy;
        for k in 0..N_SPECIES {
            planes[(4 + k) * n + i] = c.x[k];
        }
    }
    State::new(width, height, planes)
}

fn total(st: &State, c: usize) -> f64 {
    st.plane(c).iter().sum()
}

/// Σ_i |p_i| of the grid: G-CONS's Σm|u|.
fn sum_abs_momentum(st: &State) -> f64 {
    let n = st.cells();
    (0..n).map(|i| st.get(1, i).hypot(st.get(2, i))).sum()
}

fn kinetic(st: &State) -> f64 {
    let n = st.cells();
    (0..n).filter(|&i| st.get(0, i) > 0.0).map(|i| 0.5 * (st.get(1, i).powi(2) + st.get(2, i).powi(2)) / st.get(0, i)).sum()
}

// ---- the SCF disks ----

/// The disk of `mass` stepped 10,000 times on the twin, graded on G-REST's bounds; `r11` is M0-R11's held count.
fn disk_rests(mass: f64, r11: usize) {
    let (physics, elements) = (physics(), elements());
    let (scene, planes) = cold_disk_scene(mass, Matter::CarbonOxygen, &physics, &elements, &[]).unwrap().expect("an equilibrium");
    let (w, h) = (scene.world.width as usize, scene.world.height as usize);
    let switches = Switches { gravity: true, hydro: true, heat: false, reactions: false };
    let mut twin =
        Twin::new(&physics, &elements, Edge::Bounce, switches, None, w, h, planes.iter().map(|&v| f64::from(v)).collect()).unwrap();
    let n = w * h;
    let momentum = |t: &Twin| [total(&t.state, 1) + t.held[0], total(&t.state, 2) + t.held[1]];
    let p0 = momentum(&twin);
    let held0 = twin.held_mask().iter().filter(|&&b| b).count();
    let (mut ke_from, mut floor_from) = (f64::NAN, f64::NAN);
    let mut held_moving = 0usize;
    for step in 1..=STEPS {
        twin.step();
        let st = &twin.state;
        held_moving += (0..n).filter(|&i| twin.held_mask()[i] && (st.get(1, i) != 0.0 || st.get(2, i) != 0.0)).count();
        if step == FROM {
            ke_from = kinetic(st);
            floor_from = twin.booked.floor_added;
        }
    }
    let st = &twin.state;
    let held = twin.held_mask().iter().filter(|&&b| b).count();
    let (ke_end, floor) = (kinetic(st), twin.booked.floor_added - floor_from);
    let p1 = momentum(&twin);
    let drift = (p1[0] - p0[0]).hypot(p1[1] - p0[1]);
    let scale = sum_abs_momentum(st);
    println!(
        "SR-HELD-CORE disk M {mass}: held {held0} at step 0, {held} at {STEPS} (M0-R11: {r11}); floor_added {floor:.4e} over \
         {FROM}–{STEPS}; KE {ke_from:.4e} → {ke_end:.4e}; held momentum ({:.4e}, {:.4e}); momentum drift {drift:.3e} of Σm|u| \
         {scale:.4e}; held cells moving {held_moving}",
        twin.held[0], twin.held[1]
    );
    assert!(held > 0, "the disk holds no cell: the hold never fired");
    assert_eq!(held_moving, 0, "held cells with a momentum after a step");
    assert!(floor <= FLOOR_BOUND, "floor_added {floor} over steps {FROM}–{STEPS} (bound {FLOOR_BOUND})");
    assert!(ke_end <= KE_GROWTH * ke_from, "KE {ke_from} at {FROM} → {ke_end} at {STEPS} (bound × {KE_GROWTH})");
}

#[test]
fn disk_m900_rests_held() {
    disk_rests(900.0, 16);
}

#[test]
fn disk_m1800_rests_held() {
    disk_rests(1_800.0, 12);
}

// ---- P4's walls and inflow, on a 12 × 4 bounce strip, a held column at x = 6 ----

const W: usize = 12;
const H: usize = 4;
const COLUMN: usize = 6;

/// The cold pressure off (Π = Σε_th), so the strip's gas is the γ = 2 ideal gas alone.
fn warm_hydro() -> Hydro {
    let physics = physics().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).unwrap();
    Hydro::new(&physics, &elements(), Edge::Bounce).unwrap()
}

/// The strip: column x has `sigma(x)`, specific ε_th 1, x velocity `u(x)`; the held column's gas is helium, the rest
/// hydrogen, so a species crossing shows.
fn strip(sigma: impl Fn(usize) -> f64, u: impl Fn(usize) -> f64) -> (State, Vec<bool>) {
    let (hy, he) = (fractions(&[("H", 1.0)]), fractions(&[("He", 1.0)]));
    let cells: Vec<Cell> = (0..W * H)
        .map(|i| {
            let x = i % W;
            let (s, v) = (sigma(x), u(x));
            Cell { sigma: s, mom: [s * v, 0.0], energy: s * (1.0 + 0.5 * v * v), x: if x == COLUMN { he } else { hy } }
        })
        .collect();
    let held = (0..W * H).map(|i| i % W == COLUMN).collect();
    (state(W, H, &cells), held)
}

/// The held column's channels, every row: (Σ, Σu, Σv, E, X_He).
fn column(st: &State) -> Vec<[f64; 5]> {
    (0..H).map(|y| y * W + COLUMN).map(|i| [st.get(0, i), st.get(1, i), st.get(2, i), st.get(3, i), st.get(5, i)]).collect()
}

#[test]
fn a_held_wall_reflects_and_passes_nothing() {
    let hydro = warm_hydro();
    // Gas at rest at both edges (so the edges' fluxes cancel), cells 3–5 moving into the held column at 0.05.
    let (mut st, held) = strip(|x| if x == COLUMN { 20.0 } else { 1.0 }, |x| if (3..COLUMN).contains(&x) { 0.05 } else { 0.0 });
    let before = st.clone();
    let dt = hydro.dt(&st, None);
    let spared = hydro.step_held(&mut st, dt, None, None, &held);
    assert_column_kept(&before, &st);
    // The free cell against the wall: its momentum toward the wall falls (Π alone would push it on, +Δt Σu²).
    let dp: f64 = (0..H).map(|y| y * W + COLUMN - 1).map(|i| st.get(1, i) - before.get(1, i)).sum();
    // Grid + held momentum: the walls' impulses go to `held`, nothing else is lost.
    let scale = sum_abs_momentum(&before) + spared[0].hypot(spared[1]);
    let drift = momentum_drift(&before, &st, spared, [0.0, 0.0], scale);
    println!("SR-HELD-WALL Δp of the free cell {dp:.6e} (Δt {dt:.4e}); held {spared:?}; momentum drift {drift:?} of {scale:.4e}");
    assert!(dp < 0.0, "the free cell's momentum into the wall rose by {dp}: the wall does not turn it back");
    assert!(spared[0] > 0.0, "the wall's impulse on the held column {spared:?}: the gas pushes it toward +x");
    assert!(drift[0] <= STRIP_BAND && drift[1] <= STRIP_BAND, "grid + held momentum drifted {drift:?}");
}

/// Grid + held momentum's change over a step, relative to `scale`: (x, y).
fn momentum_drift(before: &State, after: &State, spared: [f64; 2], expected: [f64; 2], scale: f64) -> [f64; 2] {
    [0, 1].map(|k| (total(after, 1 + k) + spared[k] - total(before, 1 + k) - expected[k]).abs() / scale)
}

/// The held column bit for bit and at rest.
fn assert_column_kept(before: &State, after: &State) {
    for (y, (c, b)) in column(after).iter().zip(column(before)).enumerate() {
        assert!(c[0] == b[0] && c[3] == b[3] && c[4] == b[4], "row {y}: the held cell's Σ, E or X_He moved: {b:?} → {c:?}");
        assert!(c[1] == 0.0 && c[2] == 0.0, "row {y}: the held cell's momentum {c:?}");
    }
}

#[test]
fn nothing_crosses_a_held_cell_either_way() {
    let hydro = warm_hydro();
    // Dense gas (Σ 30 against the column's 20) everywhere, at rest at both edges; cells 3–5 driven into the column at 0.5
    // — its ordinary HLL flux carries mass in — and cells 7–8 drawn away from it at 0.5.
    let (mut st, held) = strip(
        |x| if x == COLUMN { 20.0 } else { 30.0 },
        |x| if (3..COLUMN).contains(&x) || (COLUMN + 1..COLUMN + 3).contains(&x) { 0.5 } else { 0.0 },
    );
    let before = st.clone();
    let dt = hydro.dt(&st, None);
    let spared = hydro.step_held(&mut st, dt, None, None, &held);
    let scale = sum_abs_momentum(&before) + spared[0].hypot(spared[1]);
    let drift = momentum_drift(&before, &st, spared, [0.0, 0.0], scale);
    println!("SR-HELD-NO-INFLOW held {spared:?}; momentum drift {drift:?} of {scale:.4e}");
    assert_column_kept(&before, &st);
    assert!(drift[0] <= STRIP_BAND && drift[1] <= STRIP_BAND, "grid + held momentum drifted {drift:?}");
}

#[test]
fn gravity_kick_on_a_held_cell_is_booked() {
    let hydro = warm_hydro();
    let (mut st, held) = strip(|x| if x == COLUMN { 20.0 } else { 1.0 }, |_| 0.0);
    let before = st.clone();
    // A uniform g along +x: φ = −g x.
    let g = 0.01;
    let field = sr_physics::reference::grav::Field {
        phi: (0..W * H).map(|i| -g * (i % W) as f64).collect(),
        gx: vec![g; W * H],
        gy: vec![0.0; W * H],
    };
    let dt = hydro.dt(&st, Some((&field.gx, &field.gy)));
    let spared = hydro.step_held(&mut st, dt, Some(&field), None, &held);
    let mass = total(&before, 0);
    let held_mass: f64 = (0..H).map(|y| before.get(0, y * W + COLUMN)).sum();
    let kick = dt * g * mass;
    let drift = momentum_drift(&before, &st, spared, [kick, 0.0], kick);
    println!("SR-HELD-GRAVITY Δt g M {kick:.6e} (the held column's share {:.6e}); held {spared:?}; drift {drift:?}", dt * g * held_mass);
    assert_column_kept(&before, &st);
    assert!(drift[0] <= STRIP_BAND && drift[1] <= STRIP_BAND, "grid + held momentum gained other than Δt·g·M: {drift:?}");
}

// ---- P8's hold ----

/// C 0.5 + O 0.5, the disks' matter.
fn co() -> [f64; N_SPECIES] {
    fractions(&[("C", 0.5), ("O", 0.5)])
}

/// A cell at rest of Σ `sigma` and fractions `x` whose ε_th is `ratio` × its ε_cold, with momentum `mom` on top.
fn cold_cell(eos: &Eos, sigma: f64, x: [f64; N_SPECIES], ratio: f64, mom: [f64; 2]) -> Cell {
    let comp = elements().composition(&x);
    let cold = eos.eps_cold(sigma, comp.y_e, x[9]);
    let ke = 0.5 * (mom[0] * mom[0] + mom[1] * mom[1]) / sigma;
    Cell { sigma, mom, energy: sigma * (1.0 + ratio) * cold + ke, x }
}

/// P8 as the twin runs it — the floors, then the hold — on `cells`, a 1-row world; the mask in and out.
fn p8(floors: &Floors, cells: &[Cell], held: &mut [bool]) -> (State, [f64; 2]) {
    let mut st = state(cells.len(), 1, cells);
    floors.apply(&mut st);
    let taken = floors.hold(&mut st, held);
    (st, taken)
}

#[test]
fn the_hold_has_its_hysteresis() {
    let physics = physics();
    let (eos, floors) = (Eos::from_physics(&physics).unwrap(), Floors::new(&physics, &elements()).unwrap());
    assert_eq!((SIGMA_HOLD, ETA_HOLD), (10.0, 0.1), "§2.7 [M0-TG2]'s σ_hold and η_hold");
    let c = |sigma: f64, ratio: f64| cold_cell(&eos, sigma, co(), ratio, [0.0, 0.0]);
    // Column by column: [a held at η then warmed, never held past η, never held under σ_hold, held then thinned, newly held moving].
    let steps: [[Cell; 5]; 5] = [
        [c(20.0, 0.099), c(20.0, 0.101), c(9.99, 0.01), c(20.0, 0.05), cold_cell(&eos, 20.0, co(), 0.05, [3.0, -4.0])],
        [c(20.0, 0.15), c(20.0, 0.101), c(9.99, 0.01), c(5.5, 0.05), c(20.0, 0.05)],
        [c(20.0, 0.199), c(20.0, 0.15), c(9.99, 0.001), c(5.01, 0.05), c(20.0, 0.05)],
        [c(20.0, 0.201), c(20.0, 0.3), c(9.99, 0.0), c(4.99, 0.05), c(20.0, 0.05)],
        [c(20.0, 0.15), c(20.0, 0.101), c(9.99, 0.01), c(20.0, 0.05), c(20.0, 0.05)],
    ];
    let want = [
        [true, false, false, true, true],
        [true, false, false, true, true],
        [true, false, false, true, true],
        [false, false, false, false, true],
        [false, false, false, true, true],
    ];
    let mut held = vec![false; 5];
    for (k, (cells, want)) in steps.iter().zip(&want).enumerate() {
        let (st, taken) = p8(&floors, cells, &mut held);
        assert_eq!(&held[..], want, "step {k}: the mask");
        if k == 0 {
            // The newly held moving cell: its momentum zeroed and booked, its E (the kinetic energy in it) as it was.
            assert_eq!(taken, [3.0, -4.0], "the momentum booked `held`");
            assert_eq!((st.get(1, 4), st.get(2, 4)), (0.0, 0.0));
            assert_eq!(st.get(3, 4), cells[4].energy, "a newly held cell's E changed");
        } else {
            assert_eq!(taken, [0.0, 0.0], "step {k}: only a newly held cell's momentum is booked");
        }
    }
}

#[test]
fn a_held_iron_cell_is_released_at_n_fe_gate() {
    let physics = physics();
    let (eos, floors) = (Eos::from_physics(&physics).unwrap(), Floors::new(&physics, &elements()).unwrap());
    let sigma_n = physics.get("sigma_n").unwrap();
    let iron = fractions(&[("Fe", 1.0)]);
    let y_e = elements().composition(&iron).y_e;
    let (under, past) = (0.9 * sigma_n / y_e, 1.1 * sigma_n / y_e);
    let c = |sigma: f64, x: [f64; N_SPECIES]| cold_cell(&eos, sigma, x, 0.05, [0.0, 0.0]);
    let half = fractions(&[("He", 0.5), ("Fe", 0.5)]);
    let mut held = vec![false; 4];
    // [iron under its gate, iron past it, C/O as dense, He | Fe past the gate]
    p8(&floors, &[c(under, iron), c(past, iron), c(past, co()), c(past, half)], &mut held);
    assert_eq!(held, [true, false, true, false], "Y_eΣ against Σ_N {sigma_n} (iron's Y_e {y_e:.4})");
    // The held iron cell compressed past the gate: released, though as cold as before.
    p8(&floors, &[c(past, iron), c(past, iron), c(past, co()), c(past, half)], &mut held);
    assert_eq!(held, [false, false, true, false], "the held iron cell past Y_eΣ = Σ_N");
    println!("SR-HELD-NFE Σ_N {sigma_n}, iron Y_e {y_e:.4}: held at Σ {under:.1}, released at {past:.1}");
}
