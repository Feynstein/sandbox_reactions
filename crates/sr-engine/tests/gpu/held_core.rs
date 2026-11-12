//! M0-D71 — the held core on the CPU twin (contract §2.7 [M0-TG2], §2.3.4 [M0-TG2], §1.4.2 [M0-TG2], §2.9's `held`, §5.1's
//! G-REST; M0-R11's H, reports/unresolved_remnant.md). P8's last act holds a cell with Σ ≥ σ_hold and ε_th ≤ η_hold ε_cold
//! which holds no iron past a trace (X_Fe < 10⁻³, §2.7 [M0-D71f], [M0-D170]) or is neutron-rich (X_n ≥ 0.5, [M0-R13]) with its disk r ≤ 24 under
//! m_tov_sb (§0.5 [M0-T130], M0-D153), and releases it past 2η_hold, under ½σ_hold, once X_Fe ≥ 10⁻³ with X_n < 0.5 or its disk
//! reaches m_tov_sb; P4
//! leaves a held cell as it is behind reflecting walls (HLL against the mirror, no mass, energy or species across, no
//! work); the momentum the hold takes from the grid is booked `held`. One-way inflow (§2.3.4's item 2) is not built: it
//! took the M 900 disk's floor to 392.6 (2.26 without) and moved to M0-D71f (§0.5 [M0-D71], the lead at M0-D71). The
//! twin's cases touch no adapter (R12); the GPU's, below the twin's, are M0-D71c's.
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
//! - **Item 1** (M0-D152, §2.7 [M0-D71f]: no iron is held; [M0-D170]: a trace under 10⁻³ is none): cold iron under and
//!   past N_Fe's gate never held; C/O beside it held, and C/O with X_Fe 9 × 10⁻⁴; the C/O released once its iron appears
//!   past the trace (X_Fe 1.1 × 10⁻³).
//! - **The neutron core** (M0-T130, M0-D153, §2.7 [M0-R13], §0.5 [M0-T130]): a cold neutron-rich cell (X_n 0.6, X_Fe 0.4)
//!   held while its disk r ≤ 24 holds 0.95 m_tov_sb (ballast spread over the disk, probes 64 apart), never at 1.05,
//!   released once its disk crosses to 1.05; iron at X_n 0.4 never held.
//!
//! Plant `held-wall-pressure-only`: the held wall takes §2.8's bounce flux, Π alone (M0-R11: the M 900 disk ran to KE 5 × 10⁶).
//!
//! **The GPU** (M0-D71c; hydro.wgsl's held cells, the adapter `SR_TEST_ADAPTER` names, R12): every GPU case starts from
//! f32 planes, its mask the initial state's hold (step 0's, as `Twin::new`'s).
//! - **G-REST's GPU half**: both disks 10,000 steps on G-REST's bounds, held cells' momentum 0 at every 50th step, grid +
//!   held momentum (P8's `held.mom_*` and P4's `held.spared.mom_*`) printed, not graded (§0.5 [M0-D71]); each disk against
//!   the twin at step 200 (G-REF's span): Σ and E relative L1 ≤ 10⁻³, the held set the same.
//! - **The strips** (64 × 64, every row alike, a cold C/O column held by the rule among warm hydrogen), one step against
//!   the twin's: the wall turns a free cell back; nothing crosses either way; the column's Σ, E and X bit for bit and its
//!   momentum 0; what the column was spared the twin's within 10⁻³; grid + held momentum kept to 10⁻⁶ of Σm|u| (no
//!   gravity). With P2: no work across the wall — the wall's free neighbours' E the twin's within 10⁻⁵, where the work
//!   [M0-D64] would do there is larger — and the column's Σg kick booked as the twin books it.
//! - **A held edge cell** in a leave world books nothing into `escaped`, its face carrying nothing.
//! - **Item 1 on the GPU** (M0-D152): the twin's item-1 cells in a world of thin hydrogen — the masks the twin's, f32 alike.
//! - **The neutron core on the GPU** (M0-T130): the twin's probes, one step each side of the crossing — the masks the twin's.
//! - **Clear in P0** (M0-D71b's flag): a Clear and a paint over a held core in one step give the planes the same edits give
//!   with the mask zeroed by hand — P4 sees no stale wall.
//! - **The pull** (M0-D151, §1.4.2 [M0-D149]: P2's g_pull, the field of all but the held cells, what hydro.wgsl's held
//!   cells read): the twin's two pull cases on the GPU — the world of held cells books `held` exactly 0 (P2's f32 field
//!   summed over it is not), the held block beside warm gas books the gas's pull within [`PULL_BAND`] of the twin's.
//! - **G-REST's window** (M0-D71f): scenes/remnant_window.bin — M0-D67's box of the Sun-like life's step 450,000 in a
//!   128 × 128 leave world ([`cut_window`]; `SR_WRITE_SCENES=1` re-cuts it from the git-ignored source) — 10,000 steps of
//!   every pass through `headless`: floor_added ≤ 15 % of radiated, both halves printed (the twin's 1.2 % and 46 %).
//! - **The hold's keys** (M0-D71g, §2.7 [M0-D71g]): σ_hold, η_hold and r_hold are physics.json's `sigma_hold`,
//!   `eta_hold` and `r_hold`, read through `Physics` by the twin and the GPU's P8 — the M 900 disk, one step: the shipped
//!   σ_hold holds 16 on each side, the same set; `sigma_hold` overridden past every Σ holds none on either side.
//!
//! Plant `held-wall-gpu-pressure-only`: hydro.wgsl's held wall the bounce flux, Π alone. Plant `held-core-gpu-whole-g`:
//! hydro.wgsl's held cells read P2's whole g again (the world of held cells books its f32 rounding). Plant
//! `hold-iron-gate`: the gate form back on the twin and in floors.wgsl — iron held while Y_eΣ ≤ Σ_N (M0-D71f). Plant
//! `neutron-hold-ungated`: the m_tov_sb gate dropped on the twin and in floors.wgsl — M0-R13's F, the neutron-rich core
//! held at any mass. Plant `hold-constants-ignored`: the old literals (10, 0.1, 24) back in the twin's `Floors::new` and
//! P8's uniform — the override case holds 16 on both sides.

use std::path::{Path, PathBuf};

use serde_json::Value;
use sr_engine::calibrate::{cold_disk_scene, Matter};
use sr_engine::dump::Dump;
use sr_engine::headless;
use sr_engine::scene::EdgeMode;
use sr_engine::state::{book_totals, Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, N_TERMS, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::edit::Edit;
use sr_engine::step::fft::read_back;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::eos::Eos;
use sr_physics::reference::floors::Floors;
use sr_physics::reference::grav::Gravity;
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

/// One of the shipped hold's keys — `sigma_hold`, `eta_hold` or `r_hold` — as physics.json holds it (§2.7 [M0-D71g]).
fn hold_key(key: &str) -> f64 {
    physics().get(key).unwrap_or_else(|| panic!("physics.json key {key}"))
}

/// The shipped physics with the neutrons' cold pressure off (K₁ₙ = K₂ₙ = 0), m_tov_sb infinite: for the fixtures that pack
/// cold C/O past m_tov_sb within r ≤ 24 (a world of held cells, the dense strip) to grade the walls and the pull, never the
/// gate (§0.5 [M0-T130], M0-D153; floors' `field_constants`, M0-T130). They hold no neutrons, so the K's reach nothing else.
fn gate_off() -> Physics {
    let physics = physics().with(&[("k1n", 0.0), ("k2n", 0.0)]).expect("K₁ₙ = K₂ₙ = 0");
    assert!(Floors::new(&physics, &elements()).unwrap().m_tov().is_infinite(), "m_tov_sb with the neutrons' cold pressure off");
    physics
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
    let spared = hydro.step_held(&mut st, dt, None, None, None, &held);
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
    let spared = hydro.step_held(&mut st, dt, None, None, None, &held);
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
    let spared = hydro.step_held(&mut st, dt, Some(&field), None, None, &held);
    let mass = total(&before, 0);
    let held_mass: f64 = (0..H).map(|y| before.get(0, y * W + COLUMN)).sum();
    let kick = dt * g * mass;
    let drift = momentum_drift(&before, &st, spared, [kick, 0.0], kick);
    println!("SR-HELD-GRAVITY Δt g M {kick:.6e} (the held column's share {:.6e}); held {spared:?}; drift {drift:?}", dt * g * held_mass);
    assert_column_kept(&before, &st);
    assert!(drift[0] <= STRIP_BAND && drift[1] <= STRIP_BAND, "grid + held momentum gained other than Δt·g·M: {drift:?}");
}

// ---- the sticky wall (§2.3.4 [M0-D172]) ----

/// The falls' speed: fast enough that the wall's own flux leaves the infall moving in after one step (so the stop shows).
const FALL: f64 = 1.0;

/// A `size` × `size` world of warm hydrogen (Σ 1, ε_th 1 + ε_cold) round a held block of cold C/O (Σ 20, ε_th ½η_hold
/// ε_cold: held by the rule) at x, y ∈ [b, b + 4): the three cells left of it, above it and below it fall into it at
/// [`FALL`], the three right of it move away at [`FALL`], the rest at rest (the edges' impulses cancel). The state and
/// the block's mask.
fn fall_world(size: usize, b: usize) -> (State, Vec<bool>) {
    let eos = Eos::from_physics(&physics()).unwrap();
    let hy = fractions(&[("H", 1.0)]);
    let cold_h = eos.eps_cold(1.0, elements().composition(&hy).y_e, 0.0);
    let inside = |v: usize| (b..b + 4).contains(&v);
    let near = |v: usize, lo: usize| (lo..lo + 3).contains(&v);
    let cells: Vec<Cell> = (0..size * size)
        .map(|i| {
            let (x, y) = (i % size, i / size);
            if inside(x) && inside(y) {
                return cold_cell(&eos, 20.0, co(), 0.5 * hold_key("eta_hold"), [0.0, 0.0]);
            }
            let u = if inside(y) && (near(x, b - 3) || near(x, b + 4)) { FALL } else { 0.0 };
            let v = if inside(x) && near(y, b - 3) {
                FALL
            } else if inside(x) && near(y, b + 4) {
                -FALL
            } else {
                0.0
            };
            Cell { sigma: 1.0, mom: [u, v], energy: 1.0 + 2.0 * cold_h + 0.5 * (u * u + v * v), x: hy }
        })
        .collect();
    let held = (0..size * size).map(|i| inside(i % size) && inside(i / size)).collect();
    (state(size, size, &cells), held)
}

/// The block's free face-neighbours in a `size` world: (cell, axis, whether its fall points into the block) — left, top,
/// bottom fall in; right moves away.
fn wall_neighbours(size: usize, b: usize) -> Vec<(usize, usize, bool)> {
    (b..b + 4)
        .flat_map(|k| [(k * size + b - 1, 0, true), (k * size + b + 4, 0, false), ((b - 1) * size + k, 1, true), ((b + 4) * size + k, 1, true)])
        .collect()
}

/// §2.3.4 [M0-D172] on the twin: a free cell falling into a held block stops at its wall — its momentum into the block
/// exactly 0 after the step, its kinetic energy left in E (the grid's E unchanged: bounce, no gravity), that momentum
/// booked `held` (grid + held momentum kept) — and a cell moving away from the block keeps its momentum.
#[test]
fn a_free_cell_stops_at_a_held_wall() {
    const S: usize = 16;
    const B: usize = 6;
    let hydro = Hydro::new(&physics(), &elements(), Edge::Bounce).unwrap();
    let (mut st, held) = fall_world(S, B);
    let before = st.clone();
    let dt = hydro.dt(&st, None);
    let spared = hydro.step_held(&mut st, dt, None, None, None, &held);
    let scale = sum_abs_momentum(&before) + spared[0].hypot(spared[1]);
    let drift = momentum_drift(&before, &st, spared, [0.0, 0.0], scale);
    let de = (total(&st, 3) - total(&before, 3)).abs() / total(&before, 3);
    let into: Vec<f64> = wall_neighbours(S, B).iter().filter(|n| n.2).map(|&(i, k, _)| st.get(1 + k, i)).collect();
    let away: Vec<f64> = wall_neighbours(S, B).iter().filter(|n| !n.2).map(|&(i, k, _)| st.get(1 + k, i)).collect();
    println!("SR-HELD-STICKY into the wall {into:?}; away {away:?}; held {spared:?}; momentum drift {drift:?}; E moved {de:.3e}");
    assert!(into.iter().all(|&p| p == 0.0), "a free cell's momentum into the held block after the step: {into:?}");
    assert!(away.iter().all(|&p| p > 0.0), "a free cell moving away from the block lost its momentum: {away:?}");
    assert!(drift[0] <= STRIP_BAND && drift[1] <= STRIP_BAND, "grid + held momentum drifted {drift:?}");
    assert!(de <= STRIP_BAND, "the grid's E moved by {de:e}: the stop took the infall's kinetic energy out of E");
}

/// The pull cases' world: 32 × 32, bounce, gravity and hydro on the twin.
const PW: usize = 32;

/// The pull cases' state: held C/O cells (Σ 20 to 30, uneven, ε_th ½η_hold ε_cold) at `core`, warm hydrogen (Σ 1) at
/// `gas`, vacuum elsewhere, all at rest — so each cell's half-step Σ is its Σ — and the twin of it, on [`gate_off`]'s physics
/// (a world of such cells holds ≈ 4 × 10⁴ within r ≤ 24).
fn pull_world(core: &[(usize, usize)], gas: &[(usize, usize)]) -> (Twin, State) {
    pull_world_of(PW, core, gas)
}

/// [`pull_world`] in a `size` × `size` world.
fn pull_world_of(size: usize, core: &[(usize, usize)], gas: &[(usize, usize)]) -> (Twin, State) {
    let physics = gate_off();
    let eos = Eos::from_physics(&physics).unwrap();
    let hy = fractions(&[("H", 1.0)]);
    let cold_h = eos.eps_cold(1.0, elements().composition(&hy).y_e, 0.0);
    let cells: Vec<Cell> = (0..size * size)
        .map(|i| {
            let p = (i % size, i / size);
            if core.contains(&p) {
                cold_cell(&eos, 20.0 + ((7 * p.0 + 3 * p.1) % 11) as f64, co(), 0.5 * hold_key("eta_hold"), [0.0, 0.0])
            } else if gas.contains(&p) {
                Cell { sigma: 1.0, mom: [0.0, 0.0], energy: 1.0 + 2.0 * cold_h, x: hy }
            } else {
                Cell { sigma: 1e-8, mom: [0.0, 0.0], energy: 1e-8, x: hy }
            }
        })
        .collect();
    let st = state(size, size, &cells);
    let switches = Switches { gravity: true, hydro: true, heat: false, reactions: false };
    let twin = Twin::new(&physics, &elements(), Edge::Bounce, switches, None, size, size, st.planes.clone()).unwrap();
    (twin, st)
}

/// Σ Δt·Σ_i·g_i over the cells `on` picks, (x, y): g the field P2 solves from the Σ of the cells `sources` picks.
fn pull_on(st: &State, dt: f64, sources: impl Fn(usize) -> bool, on: impl Fn(usize) -> bool) -> [f64; 2] {
    let n = st.cells();
    let sigma: Vec<f64> = (0..n).map(|i| if sources(i) { st.get(0, i) } else { 0.0 }).collect();
    let f = Gravity::new(1.0).field(&sigma, physics().get("sigma_vac").unwrap(), PW, PW).unwrap();
    (0..n).filter(|&i| on(i)).fold([0.0, 0.0], |a, i| [a[0] + dt * st.get(0, i) * f.gx[i], a[1] + dt * st.get(0, i) * f.gy[i]])
}

/// §1.4.2 [M0-D149]: a held core books none of its own pull — a world of held C/O cells, its Σ uneven, one step: every
/// face held–held or on the edge, so nothing but gravity could be booked, and `held` is exactly 0, where P2's whole field
/// summed over the core (the booking before M0-D149) is not: f64's rounding of a sum that cancels, which in the GPU's f32
/// was its whole `held` on G-REST's disks. (A core in vacuum books its walls' floor pressure, 1e-10.)
#[test]
fn a_held_core_books_none_of_its_own_pull() {
    let core: Vec<(usize, usize)> = (0..PW).flat_map(|y| (0..PW).map(move |x| (x, y))).collect();
    let (mut twin, st) = pull_world(&core, &[]);
    twin.step();
    let in_core = |i: usize| core.contains(&(i % PW, i / PW));
    let own = pull_on(&st, twin.dt(), |_| true, in_core);
    println!("SR-HELD-OWN-PULL held {:?}; P2's whole field summed over the core {own:?}", twin.held);
    assert_eq!(twin.held_mask().iter().filter(|&&m| m).count(), core.len(), "the world is not held as built: the case grades no hold");
    assert!(own != [0.0, 0.0], "the whole field sums to 0 over the core: the case cannot tell its own pull from none");
    assert_eq!(twin.held, [0.0, 0.0], "the held core booked its own pull");
}

/// §1.4.2 [M0-D149]: what a held core books is the free gas's pull on it, the reaction of the core's pull on that gas
/// (Newton's third law, which P2's even K keeps) — a 4 × 4 held block, a 6 × 6 square of warm hydrogen below and to its
/// right, one step: `held` toward the gas, within 10⁻⁹ of minus Σ Δt·Σ·g_core over the gas (g_core the block's field
/// alone). A booking that drops the pull fails here; one that keeps the core's own, the case above.
#[test]
fn a_held_core_books_the_free_gas_pull_as_its_reaction() {
    let core: Vec<(usize, usize)> = (8..12).flat_map(|y| (6..10).map(move |x| (x, y))).collect();
    let gas: Vec<(usize, usize)> = (12..18).flat_map(|y| (18..24).map(move |x| (x, y))).collect();
    let (mut twin, st) = pull_world(&core, &gas);
    twin.step();
    let in_core = |i: usize| core.contains(&(i % PW, i / PW));
    let in_gas = |i: usize| gas.contains(&(i % PW, i / PW));
    let reaction = pull_on(&st, twin.dt(), in_core, in_gas);
    let off = (twin.held[0] + reaction[0]).hypot(twin.held[1] + reaction[1]) / reaction[0].hypot(reaction[1]);
    println!("SR-HELD-GAS-PULL held {:?}; minus the core's pull on the gas {:?} (off {off:.3e})", twin.held, reaction.map(|r| -r));
    assert_eq!(twin.held_mask().iter().filter(|&&m| m).count(), core.len(), "the block is not held as built: the case grades no hold");
    assert!(twin.held[0] > 0.0 && twin.held[1] > 0.0, "the held block booked {:?}: the gas lies toward +x and +y", twin.held);
    assert!(off <= 1e-9, "held {:?} against the reaction {:?}: off by {off:e}", twin.held, reaction.map(|r| -r));
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
    assert_eq!((floors.sigma_hold(), floors.eta_hold(), floors.r_hold()), (10.0, 0.1, 24.0), "§2.7 [M0-D71g]'s σ_hold, η_hold and r_hold");
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

/// Item 1's cells (§2.7 [M0-D71f], [M0-D170]): [cold iron under N_Fe's gate, cold iron past it, C/O as dense, C/O as
/// dense, C/O with a trace of iron 9 × 10⁻⁴ — under the trace 10⁻³, held as iron-free], all at ½η_hold; with
/// `iron_appears` the fourth gains X_Fe 1.1 × 10⁻³ (from its O), past the trace. Σ_N / Y_e of iron times 0.9 and 1.1 —
/// the gate form held the first and refused the second.
fn item1_cells(eos: &Eos, physics: &Physics, iron_appears: bool) -> ([Cell; 5], f64, f64) {
    let iron = fractions(&[("Fe", 1.0)]);
    let y_e = elements().composition(&iron).y_e;
    let sigma_n = physics.get("sigma_n").unwrap();
    let (under, past) = (0.9 * sigma_n / y_e, 1.1 * sigma_n / y_e);
    let c = |sigma: f64, x: [f64; N_SPECIES]| cold_cell(eos, sigma, x, 0.5 * hold_key("eta_hold"), [0.0, 0.0]);
    let fourth = if iron_appears { fractions(&[("C", 0.5), ("O", 0.4989), ("Fe", 1.1e-3)]) } else { co() };
    let trace = fractions(&[("C", 0.5), ("O", 0.4991), ("Fe", 9e-4)]);
    ([c(under, iron), c(past, iron), c(past, co()), c(past, fourth), c(past, trace)], under, past)
}

/// Item 1's masks: after the first P8, and after the second, once the fourth cell's iron appears.
const ITEM1: [[bool; 5]; 2] = [[false, false, true, true, true], [false, false, true, false, true]];

#[test]
fn no_iron_is_held() {
    let physics = physics();
    let (eos, floors) = (Eos::from_physics(&physics).unwrap(), Floors::new(&physics, &elements()).unwrap());
    let mut held = vec![false; 5];
    let (cells, under, past) = item1_cells(&eos, &physics, false);
    p8(&floors, &cells, &mut held);
    assert_eq!(held, ITEM1[0], "iron at Σ {under:.1} and {past:.1}, C/O beside it, C/O with X_Fe 9e-4");
    // The held C/O cell gains iron past the trace: released, though as cold as before; the trace cell stays held.
    p8(&floors, &item1_cells(&eos, &physics, true).0, &mut held);
    assert_eq!(held, ITEM1[1], "the held C/O cell with X_Fe 1.1e-3");
    println!("SR-HELD-NO-IRON twin: iron never held at Σ {under:.1} or {past:.1}; C/O and C/O with X_Fe 9e-4 held, released at X_Fe 1.1e-3");
}

/// The neutron probes' world edge: four disks r ≤ 24, `NP` cells apart, inside it.
const NW: usize = 128;
/// The probes' spacing, ≥ 50 so no probe's disk r ≤ 24 reaches another's ballast.
const NP: usize = 64;

/// The neutron hold's probes (§2.7 [M0-R13], §0.5 [M0-T130], M0-D153): in an `NW` × `NW` world of thin warm hydrogen
/// (Σ 1, never held), four cold centres at ½η_hold at (32, 32), (96, 32), (32, 96) and (96, 96), `NP` apart —
/// [neutron-rich (X_n 0.6, X_Fe 0.4) with its disk r ≤ 24 at 0.95 m_tov_sb, the same at 1.05 m_tov_sb, iron under X_n 0.5
/// (X_n 0.4, X_Fe 0.6) at 0.95, neutron-rich at 0.95 — or at 1.05 once `crossed`]: every other cell of each centre's disk
/// r ≤ r_hold warm hydrogen ballast (Σ ≈ 2, under σ_hold; ε_th = 1 + 10 ε_cold), spread so the disk (centre + 1,808
/// ballast cells) holds the share asked. Returns the cells and the centres' indices.
fn neutron_probes(eos: &Eos, m_tov: f64, crossed: bool) -> (Vec<Cell>, [usize; 4]) {
    const CENTRE: f64 = 1_000.0;
    let at: [usize; 4] = std::array::from_fn(|k| (32 + NP * (k / 2)) * NW + 32 + NP * (k % 2));
    let r = hold_key("r_hold") as i64;
    let ring: Vec<(i64, i64)> =
        (-r..=r).flat_map(|dy| (-r..=r).map(move |dx| (dx, dy))).filter(|&(dx, dy)| (dx, dy) != (0, 0) && dx * dx + dy * dy <= r * r).collect();
    let hy = fractions(&[("H", 1.0)]);
    let rich = fractions(&[("n", 0.6), ("Fe", 0.4)]);
    let iron = fractions(&[("n", 0.4), ("Fe", 0.6)]);
    let ballast = |share: f64| {
        let b = (share * m_tov - CENTRE) / ring.len() as f64;
        assert!(b < 0.5 * hold_key("sigma_hold"), "ballast Σ {b}: never held, never summed");
        let cold = eos.eps_cold(b, elements().composition(&hy).y_e, 0.0);
        Cell { sigma: b, mom: [0.0; 2], energy: b * (cold + 1.0 + 10.0 * cold), x: hy }
    };
    let probes = [(rich, 0.95), (rich, 1.05), (iron, 0.95), (rich, if crossed { 1.05 } else { 0.95 })];
    let mut cells = vec![Cell { sigma: 1.0, mom: [0.0; 2], energy: 1.0, x: hy }; NW * NW];
    for (&c, &(x, share)) in at.iter().zip(&probes) {
        cells[c] = cold_cell(eos, CENTRE, x, 0.5 * hold_key("eta_hold"), [0.0, 0.0]);
        for &(dx, dy) in &ring {
            cells[(c as i64 + dy * NW as i64 + dx) as usize] = ballast(share);
        }
    }
    (cells, at)
}

/// The probes' masks: after the first P8, and after the second, once the fourth's disk has crossed m_tov_sb.
const NEUTRON: [[bool; 4]; 2] = [[true, false, false, true], [true, false, false, false]];

#[test]
fn a_neutron_core_is_held_under_m_tov_sb() {
    // §2.7 [M0-R13] as §0.5 [M0-T130] gates it (M0-D153): a cold neutron-rich cell is held while its disk r ≤ 24 is under
    // m_tov_sb, never over it, released once its disk crosses it; iron under X_n 0.5 is never held (M0-D152's rule kept).
    let physics = physics();
    let (eos, floors) = (Eos::from_physics(&physics).unwrap(), Floors::new(&physics, &elements()).unwrap());
    let m_tov = floors.m_tov();
    assert!((4_800.0..4_900.0).contains(&m_tov), "m_tov_sb {m_tov} (§2.11's shipped 4,853.17)");
    let mut held = vec![false; NW * NW];
    let mut masks = Vec::new();
    for crossed in [false, true] {
        let (cells, at) = neutron_probes(&eos, m_tov, crossed);
        let mut st = state(NW, NW, &cells);
        floors.apply(&mut st);
        let disks = at.map(|c| floors.disk_mass(&st, c));
        floors.hold(&mut st, &mut held);
        let mask = at.map(|c| held[c]);
        println!(
            "SR-HELD-NEUTRON twin: disks {:?} of m_tov_sb {m_tov:.2} → mask {mask:?} ({} held)",
            disks.map(|d| (d / m_tov * 1e3).round() / 1e3),
            held.iter().filter(|&&h| h).count()
        );
        masks.push((mask, held.iter().filter(|&&h| h).count()));
    }
    assert_eq!(masks[0].0, NEUTRON[0], "neutron-rich at 0.95 and 1.05 m_tov_sb, iron at X_n 0.4, neutron-rich at 0.95");
    assert_eq!(masks[1].0, NEUTRON[1], "the fourth's disk crossed to 1.05 m_tov_sb");
    assert_eq!((masks[0].1, masks[1].1), (2, 1), "cells held off the probes'");
}

// ---- the GPU (M0-D71c) ----

/// G-REF's band (§5.1): relative L1 in Σ and E against the twin.
const L1_BAND: f64 = 1e-3;
/// What the held cells were spared, against the twin's: relative.
const SPARED_BAND: f64 = 1e-3;
/// The GPU strips' momentum band, relative to Σm|u|: f32 cells, a step.
const GPU_STRIP_BAND: f64 = 1e-6;
/// The wall's free neighbours' E against the twin's, relative, with P2.
const WORK_BAND: f64 = 1e-5;
/// Steps per submission on the GPU, and so how often the disks' held cells are read.
const CHUNK: u64 = 50;
/// The GPU's pull on the held block against the twin's, relative: f32 fields and an f32 product per cell, a step.
const PULL_BAND: f64 = 1e-4;
/// The GPU strips: 64 × 64, the held column at x = 40 (off the centre, so P2's g there is not 0).
const GW: usize = 64;
const GCOL: usize = 40;

/// A GPU world from f32 planes: P4 under `edge`, P2 when `gravity`, P0 when `edits`; the box fit to the state.
struct Gpu {
    gpu: sr_engine::gpu::Gpu,
    world: WorldConfig,
    state: GpuState,
    booking: Booking,
    _side: SideFields,
    step: Step,
}

impl Gpu {
    fn new(physics: &Physics, world: WorldConfig, planes: &[f32], edge: EdgeMode, gravity: bool, edits: bool) -> Gpu {
        let gpu = super::device();
        let state = GpuState::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let side = SideFields::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, physics, &elements());
        let passes = Passes { mode: BoxMode::Fit, gravity: gravity.then_some(&side), hydro: Some(edge) };
        let mut step = Step::with_passes(&gpu.device, &state, &booking, &eos, physics, &passes).unwrap();
        if edits {
            step = step.with_edits(&gpu.device, &state, &booking, &eos, physics);
        }
        Gpu { gpu, world, state, booking, _side: side, step }
    }

    fn run(&self, steps: u64) {
        self.step.run(&self.gpu.device, &self.gpu.queue, steps as u32);
    }

    fn planes(&self) -> Vec<f32> {
        self.state.readback(&self.gpu.device, &self.gpu.queue)
    }

    fn mask(&self) -> Vec<bool> {
        self.state.read_held(&self.gpu.device, &self.gpu.queue).iter().map(|&m| m != 0).collect()
    }

    fn book(&self) -> Vec<f32> {
        self.booking.readback(&self.gpu.device, &self.gpu.queue)
    }

    fn booked(&self, name: &str) -> f64 {
        book_totals(self.world, &self.book())[TERMS.iter().position(|t| t.name == name).unwrap()]
    }

    /// The momentum the hold took from the grid: P8's newly held cells' and what P4 spared the held ones, (x, y).
    fn held(&self) -> [f64; 2] {
        let totals: [f64; N_TERMS] = book_totals(self.world, &self.book());
        let t = |name: &str| totals[TERMS.iter().position(|t| t.name == name).unwrap()];
        [t("held.mom_x") + t("held.spared.mom_x"), t("held.mom_y") + t("held.spared.mom_y")]
    }
}

/// The slots of booked term `name` in a [`Booking`] readback.
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

/// f32 planes of a state's f64 ones, and back: the GPU and the twin start from the same numbers.
fn to_f32(st: &State) -> Vec<f32> {
    st.planes.iter().map(|&v| v as f32).collect()
}

fn twin_of(physics: &Physics, planes: &[f32], world: WorldConfig, edge: EdgeMode, gravity: bool) -> Twin {
    let edge = if edge == EdgeMode::Leave { Edge::Leave } else { Edge::Bounce };
    let switches = Switches { gravity, hydro: true, heat: false, reactions: false };
    let (w, h) = (world.width as usize, world.height as usize);
    Twin::new(physics, &elements(), edge, switches, None, w, h, planes.iter().map(|&v| f64::from(v)).collect()).unwrap()
}

/// Grid momentum (x, y) of f32 planes in f64, and Σm|u|.
fn gpu_momentum(planes: &[f32], n: usize) -> ([f64; 2], f64) {
    let p = |i: usize| (f64::from(planes[n + i]), f64::from(planes[2 * n + i]));
    let sum = (0..n).fold([0.0; 2], |s, i| [s[0] + p(i).0, s[1] + p(i).1]);
    (sum, (0..n).map(|i| p(i).0.hypot(p(i).1)).sum())
}

fn gpu_kinetic(planes: &[f32], n: usize) -> f64 {
    (0..n)
        .filter(|&i| planes[i] > 0.0)
        .map(|i| 0.5 * (f64::from(planes[n + i]).powi(2) + f64::from(planes[2 * n + i]).powi(2)) / f64::from(planes[i]))
        .sum()
}

/// Relative L1 difference of plane `c`: Σ|a − b| / Σ|b| (b the twin's).
fn l1(gpu: &[f32], twin: &[f64], c: usize, n: usize) -> f64 {
    let r = c * n..(c + 1) * n;
    let diff: f64 = gpu[r.clone()].iter().zip(&twin[r.clone()]).map(|(&a, &b)| (f64::from(a) - b).abs()).sum();
    diff / twin[r].iter().map(|b| b.abs()).sum::<f64>()
}

/// The disk of `mass` as `calibrate` builds it: its f32 planes and world.
fn disk(physics: &Physics, mass: f64) -> (Vec<f32>, WorldConfig, EdgeMode) {
    let (scene, planes) = cold_disk_scene(mass, Matter::CarbonOxygen, physics, &elements(), &[]).unwrap().expect("an equilibrium");
    (planes, scene.world, scene.edge)
}

/// The disk of `mass` stepped 10,000 times on the GPU, graded on G-REST's bounds; `r11` is M0-R11's held count.
fn disk_rests_on_the_gpu(mass: f64, r11: usize) {
    let physics = physics();
    let (planes, world, edge) = disk(&physics, mass);
    let gpu = Gpu::new(&physics, world, &planes, edge, true, false);
    let n = world.cells();
    let (p0, _) = gpu_momentum(&planes, n);
    let (mut ke_from, mut floor_from, mut held_moving, mut held_first) = (f64::NAN, f64::NAN, 0usize, 0usize);
    for chunk in 1..=STEPS / CHUNK {
        gpu.run(CHUNK);
        let (out, mask) = (gpu.planes(), gpu.mask());
        held_moving += (0..n).filter(|&i| mask[i] && (out[n + i] != 0.0 || out[2 * n + i] != 0.0)).count();
        if chunk == 1 {
            held_first = mask.iter().filter(|&&m| m).count();
        }
        if chunk * CHUNK == FROM {
            ke_from = gpu_kinetic(&out, n);
            floor_from = gpu.booked("energy.floor_added");
        }
    }
    let report = gpu.step.read_dt(&gpu.gpu.device, &gpu.gpu.queue);
    assert_eq!(report.steps, STEPS, "the GPU ran {} steps", report.steps);
    assert!(report.non_finite.is_none(), "non-finite state: {:?}", report.non_finite);
    let out = gpu.planes();
    let held = gpu.mask().iter().filter(|&&m| m).count();
    let (ke_end, floor) = (gpu_kinetic(&out, n), gpu.booked("energy.floor_added") - floor_from);
    let ((p1, scale), booked) = (gpu_momentum(&out, n), gpu.held());
    let drift = (p1[0] + booked[0] - p0[0]).hypot(p1[1] + booked[1] - p0[1]);
    println!(
        "SR-HELD-CORE-GPU disk M {mass}: held {held_first} at step {CHUNK}, {held} at {STEPS} (M0-R11: {r11}); floor_added \
         {floor:.4e} over {FROM}–{STEPS}; KE {ke_from:.4e} → {ke_end:.4e}; held momentum ({:.4e}, {:.4e}); momentum drift \
         {drift:.3e} of Σm|u| {scale:.4e}; held cells moving {held_moving}",
        booked[0],
        booked[1]
    );
    assert!(held > 0, "the disk holds no cell: the hold never fired");
    assert_eq!(held_moving, 0, "held cells with a momentum at a read");
    assert!(floor <= FLOOR_BOUND, "floor_added {floor} over steps {FROM}–{STEPS} (bound {FLOOR_BOUND})");
    assert!(ke_end <= KE_GROWTH * ke_from, "KE {ke_from} at {FROM} → {ke_end} at {STEPS} (bound × {KE_GROWTH})");
}

#[test]
fn gpu_disk_m900_rests_held() {
    disk_rests_on_the_gpu(900.0, 16);
}

#[test]
fn gpu_disk_m1800_rests_held() {
    disk_rests_on_the_gpu(1_800.0, 12);
}

/// The disk of `mass` on the GPU and the twin from the same f32 planes, 200 steps (G-REF's span): Σ and E within G-REF's
/// band, the held set the same.
fn disk_agrees_with_the_twin(mass: f64) {
    const SPAN: u64 = 200;
    let physics = physics();
    let (planes, world, edge) = disk(&physics, mass);
    let n = world.cells();
    let mut twin = twin_of(&physics, &planes, world, edge, true);
    for _ in 0..SPAN {
        twin.step();
    }
    let gpu = Gpu::new(&physics, world, &planes, edge, true, false);
    gpu.run(SPAN);
    let (out, mask) = (gpu.planes(), gpu.mask());
    let (sigma, energy) = (l1(&out, &twin.state.planes, 0, n), l1(&out, &twin.state.planes, 3, n));
    let differ = (0..n).filter(|&i| mask[i] != twin.held_mask()[i]).count();
    let held = mask.iter().filter(|&&m| m).count();
    // The f32 slots' scale: what P4 spared each held cell over the span, summed in magnitude.
    let book = gpu.book();
    let spared: f64 = ["held.spared.mom_x", "held.spared.mom_y"].iter().flat_map(|t| term_plane(world, &book, t)).map(|v| f64::from(v).abs()).sum();
    println!(
        "SR-HELD-REF disk M {mass}, step {SPAN}: relative L1 Σ {sigma:.3e}, E {energy:.3e} (band {L1_BAND:e}); held {held}, {differ} \
         cells held on one side only; held momentum GPU {:?} (Σ|slot| {spared:.4e}), twin {:?}",
        gpu.held(),
        twin.held
    );
    assert!(held > 0, "no cell held: the comparison grades no hold");
    assert_eq!(differ, 0, "the held sets differ in {differ} cells");
    assert!(sigma <= L1_BAND && energy <= L1_BAND, "relative L1 Σ {sigma:e}, E {energy:e} > {L1_BAND:e}");
}

#[test]
fn gpu_disk_m900_agrees_with_the_twin_at_step_200() {
    disk_agrees_with_the_twin(900.0);
}

#[test]
fn gpu_disk_m1800_agrees_with_the_twin_at_step_200() {
    disk_agrees_with_the_twin(1_800.0);
}

// ---- the hold's keys read from the physics (M0-D71g) ----

/// The M 900 disk's held count after one step, twin and GPU, under `physics` (the disk built from the shipped physics).
fn disk_m900_held(physics: &Physics, planes: &[f32], world: WorldConfig, edge: EdgeMode) -> (usize, usize, usize) {
    let mut twin = twin_of(physics, planes, world, edge, true);
    twin.step();
    let gpu = Gpu::new(physics, world, planes, edge, true, false);
    gpu.run(1);
    let mask = gpu.mask();
    let differ = (0..world.cells()).filter(|&i| mask[i] != twin.held_mask()[i]).count();
    (twin.held_mask().iter().filter(|&&b| b).count(), mask.iter().filter(|&&m| m).count(), differ)
}

/// §2.7 [M0-D71g]: σ_hold, η_hold and r_hold are physics.json's keys, read through `Physics` by the twin (`Floors`) and
/// the GPU's P8 (its uniform) — a scene's `overrides` reaches both. The M 900 disk, one step: the shipped σ_hold holds 16
/// cells on each side, the same set; `sigma_hold` overridden to ten times the disk's densest Σ holds none on either side.
/// Adversarial: a side reading a literal holds its 16 under the override, which no literal holds.
#[test]
fn gpu_an_overridden_sigma_hold_reaches_the_twin_and_the_gpu() {
    let shipped = physics();
    let (planes, world, edge) = disk(&shipped, 900.0);
    let densest = planes[..world.cells()].iter().fold(0f32, |m, &v| m.max(v)) as f64;
    let over = shipped.with(&[("sigma_hold", 10.0 * densest)]).expect("sigma_hold overridden");
    let (twin, gpu, differ) = disk_m900_held(&shipped, &planes, world, edge);
    let (twin_over, gpu_over, differ_over) = disk_m900_held(&over, &planes, world, edge);
    println!(
        "SR-HELD-KEYS disk M 900, step 1: shipped σ_hold {} → held twin {twin}, GPU {gpu} ({differ} on one side only); \
         sigma_hold {:.4e} (10 × the densest Σ {densest:.4e}) → twin {twin_over}, GPU {gpu_over} ({differ_over})",
        hold_key("sigma_hold"),
        10.0 * densest
    );
    assert_eq!((twin, gpu, differ), (16, 16, 0), "the shipped σ_hold holds the M 900 disk's 16, the same set on each side");
    assert_eq!((twin_over, gpu_over), (0, 0), "sigma_hold past every Σ: a side still holds — it read a literal");
}

// ---- the pull on the GPU (M0-D151) ----

/// [`pull_world`]'s state at the GPU's 64 × 64 on the GPU and the twin, one step: the GPU, its held total and the twin.
fn gpu_pull_step(core: &[(usize, usize)], gas: &[(usize, usize)]) -> (Gpu, [f64; 2], Twin) {
    let physics = gate_off();
    let (mut twin, st) = pull_world_of(GW, core, gas);
    twin.step();
    let world = WorldConfig::new(GW as u32, GW as u32).unwrap();
    let gpu = Gpu::new(&physics, world, &to_f32(&st), EdgeMode::Bounce, true, false);
    gpu.run(1);
    let held = gpu.held();
    (gpu, held, twin)
}

/// §1.4.2 [M0-D149] on the GPU: a world of held cells (`a_held_core_books_none_of_its_own_pull`'s, at 64 × 64) books
/// `held` exactly 0 — its g_pull is the field of nothing — where P2's whole f32 field summed over it (the booking before
/// M0-D151) is not.
#[test]
fn gpu_a_held_core_books_none_of_its_own_pull() {
    let core: Vec<(usize, usize)> = (0..GW).flat_map(|y| (0..GW).map(move |x| (x, y))).collect();
    let (gpu, held, _) = gpu_pull_step(&core, &[]);
    let g = read_back(&gpu.gpu.device, &gpu.gpu.queue, gpu.step.gravity_field().expect("a step with P2"));
    let (n, planes) = (GW * GW, gpu.planes());
    let own = (0..n).fold([0.0, 0.0], |a, i| [a[0] + f64::from(planes[i]) * f64::from(g[i]), a[1] + f64::from(planes[i]) * f64::from(g[n + i])]);
    println!("SR-HELD-OWN-PULL-GPU held {held:?}; Σ Σ·g over the core, P2's whole field {own:?}");
    assert_eq!(gpu.mask().iter().filter(|&&m| m).count(), n, "the world is not held as built: the case grades no hold");
    assert!(own != [0.0, 0.0], "P2's whole field sums to 0 over the core: the case cannot tell its own pull from none");
    assert_eq!(held, [0.0, 0.0], "the held core booked its own pull");
}

/// §1.4.2 [M0-D149] on the GPU: the held block beside warm gas
/// (`a_held_core_books_the_free_gas_pull_as_its_reaction`'s) books the gas's pull, the twin's within [`PULL_BAND`].
#[test]
fn gpu_a_held_core_books_the_free_gas_pull_as_the_twins() {
    let core: Vec<(usize, usize)> = (8..12).flat_map(|y| (6..10).map(move |x| (x, y))).collect();
    let gas: Vec<(usize, usize)> = (12..18).flat_map(|y| (18..24).map(move |x| (x, y))).collect();
    let (gpu, held, twin) = gpu_pull_step(&core, &gas);
    let off = (held[0] - twin.held[0]).hypot(held[1] - twin.held[1]) / twin.held[0].hypot(twin.held[1]);
    println!("SR-HELD-GAS-PULL-GPU held {held:?}; twin {:?} (off {off:.3e}, band {PULL_BAND:e})", twin.held);
    assert_eq!(gpu.mask().iter().filter(|&&m| m).count(), core.len(), "the block is not held as built: the case grades no hold");
    assert!(held[0] > 0.0 && held[1] > 0.0, "the held block booked {held:?}: the gas lies toward +x and +y");
    assert!(off <= PULL_BAND, "held {held:?} against the twin's {:?}: off by {off:e}", twin.held);
}

// ---- the GPU strips ----

/// A 64 × 64 world, every row alike: the columns `held` cold C/O at Σ 20 (ε_th ½η_hold ε_cold: held by the rule), the
/// rest hydrogen of Σ `sigma`, warm (ε_th 1 + ε_cold), at x velocity `u(x)`.
fn gpu_strip(eos: &Eos, held: &[usize], sigma: f64, u: impl Fn(usize) -> f64) -> Vec<f32> {
    let hy = fractions(&[("H", 1.0)]);
    let cold_h = eos.eps_cold(sigma, elements().composition(&hy).y_e, 0.0);
    let cells: Vec<Cell> = (0..GW * GW)
        .map(|i| {
            let x = i % GW;
            if held.contains(&x) {
                return cold_cell(eos, 20.0, co(), 0.5 * hold_key("eta_hold"), [0.0, 0.0]);
            }
            let v = u(x);
            Cell { sigma, mom: [sigma * v, 0.0], energy: sigma * (1.0 + 2.0 * cold_h + 0.5 * v * v), x: hy }
        })
        .collect();
    to_f32(&state(GW, GW, &cells))
}

/// One step of `planes` on the GPU and the twin: the GPU's state after it, and both.
fn strip_step(physics: &Physics, planes: &[f32], edge: EdgeMode, gravity: bool) -> (Vec<f32>, Gpu, Twin) {
    let world = WorldConfig::new(GW as u32, GW as u32).unwrap();
    let mut twin = twin_of(physics, planes, world, edge, gravity);
    twin.step();
    let gpu = Gpu::new(physics, world, planes, edge, gravity, false);
    gpu.run(1);
    (gpu.planes(), gpu, twin)
}

/// The held columns `cols` of the GPU's state: held, their Σ, E and X bit for bit, their momentum 0.
fn assert_gpu_columns_kept(gpu: &Gpu, before: &[f32], after: &[f32], cols: &[usize]) {
    let n = GW * GW;
    let mask = gpu.mask();
    for i in (0..n).filter(|i| cols.contains(&(i % GW))) {
        assert!(mask[i], "cell {i} of a held column is not held: the case grades no hold");
        for c in (0..14).filter(|&c| c != 1 && c != 2) {
            assert_eq!(after[c * n + i].to_bits(), before[c * n + i].to_bits(), "cell {i}: the held cell's channel {c} moved");
        }
        assert!(after[n + i] == 0.0 && after[2 * n + i] == 0.0, "cell {i}: the held cell's momentum");
    }
    assert_eq!(mask.iter().filter(|&&m| m).count(), cols.len() * GW, "cells held off the columns");
}

/// What the GPU's held cells were spared against the twin's, relative.
fn spared_off(gpu: &Gpu, twin: &Twin) -> f64 {
    let g = gpu.held();
    (g[0] - twin.held[0]).hypot(g[1] - twin.held[1]) / twin.held[0].hypot(twin.held[1])
}

#[test]
fn gpu_a_held_wall_reflects_as_the_twins() {
    let physics = physics();
    let eos = Eos::from_physics(&physics).unwrap();
    // Gas at rest at both edges, cells 37–39 moving into the held column at 0.05.
    let before = gpu_strip(&eos, &[GCOL], 1.0, |x| if (GCOL - 3..GCOL).contains(&x) { 0.05 } else { 0.0 });
    let (after, gpu, twin) = strip_step(&physics, &before, EdgeMode::Bounce, false);
    let n = GW * GW;
    assert_gpu_columns_kept(&gpu, &before, &after, &[GCOL]);
    let dp: f64 = (0..GW).map(|y| y * GW + GCOL - 1).map(|i| f64::from(after[n + i]) - f64::from(before[n + i])).sum();
    let (p0, scale) = gpu_momentum(&before, n);
    let (p1, held) = (gpu_momentum(&after, n).0, gpu.held());
    let drift = [0, 1].map(|k| (p1[k] + held[k] - p0[k]).abs() / scale);
    let off = spared_off(&gpu, &twin);
    println!("SR-HELD-WALL-GPU Δp of the free cells {dp:.6e}; held GPU {held:?}, twin {:?} (off {off:.3e}); drift {drift:?} of {scale:.4e}", twin.held);
    assert!(dp < 0.0, "the free cells' momentum into the wall rose by {dp}: the wall does not turn them back");
    assert!(held[0] > 0.0, "the wall's impulse on the held column {held:?}: the gas pushes it toward +x");
    assert!(off <= SPARED_BAND, "the held column was spared {held:?}, the twin's {:?}", twin.held);
    assert!(drift[0] <= GPU_STRIP_BAND && drift[1] <= GPU_STRIP_BAND, "grid + held momentum drifted {drift:?}");
}

#[test]
fn gpu_nothing_crosses_a_held_cell_either_way() {
    // Two columns of Σ 20 beside Σ 30 gas put ≈ 4 × 10⁴ within r ≤ 24 of a column cell: the gate off (`gate_off`).
    let physics = gate_off();
    let eos = Eos::from_physics(&physics).unwrap();
    // Dense warm gas (Σ 30 against the column's 20), cells 37–39 driven into the column at 0.5 — its ordinary HLL flux
    // carries mass in — and 41–42 drawn away from it at 0.5; two held columns side by side at 40 and 41 would also hold
    // the held–held face, so 41 is free here and a second strip holds 40 and 41.
    let mover = |x: usize| if (GCOL - 3..GCOL).contains(&x) || (GCOL + 1..GCOL + 3).contains(&x) { 0.5 } else { 0.0 };
    for cols in [vec![GCOL], vec![GCOL, GCOL + 1]] {
        let before = gpu_strip(&eos, &cols, 30.0, mover);
        let (after, gpu, twin) = strip_step(&physics, &before, EdgeMode::Bounce, false);
        let n = GW * GW;
        assert_gpu_columns_kept(&gpu, &before, &after, &cols);
        let (p0, scale) = gpu_momentum(&before, n);
        let (p1, held) = (gpu_momentum(&after, n).0, gpu.held());
        let drift = [0, 1].map(|k| (p1[k] + held[k] - p0[k]).abs() / scale);
        let off = spared_off(&gpu, &twin);
        println!("SR-HELD-NO-INFLOW-GPU columns {cols:?}: held GPU {held:?}, twin {:?} (off {off:.3e}); drift {drift:?} of {scale:.4e}", twin.held);
        assert!(off <= SPARED_BAND, "columns {cols:?}: spared {held:?}, the twin's {:?}", twin.held);
        assert!(drift[0] <= GPU_STRIP_BAND && drift[1] <= GPU_STRIP_BAND, "columns {cols:?}: grid + held momentum drifted {drift:?}");
    }
}

#[test]
fn gpu_no_work_crosses_a_held_wall_and_the_kick_is_booked() {
    let physics = physics();
    let eos = Eos::from_physics(&physics).unwrap();
    // The wall strip under its own gravity (P2): the column's Σg kick booked, the wall's neighbours doing no work across it.
    let before = gpu_strip(&eos, &[GCOL], 1.0, |x| if (GCOL - 3..GCOL).contains(&x) { 0.05 } else { 0.0 });
    let (after, gpu, twin) = strip_step(&physics, &before, EdgeMode::Bounce, true);
    let n = GW * GW;
    assert_gpu_columns_kept(&gpu, &before, &after, &[GCOL]);
    let off = spared_off(&gpu, &twin);
    // The neighbours' E against the twin's, and how far E moved there over the step (what a face's work would be part of).
    let near: Vec<usize> = (0..GW).flat_map(|y| [y * GW + GCOL - 1, y * GW + GCOL + 1]).collect();
    let worst = near
        .iter()
        .map(|&i| (f64::from(after[3 * n + i]) - twin.state.planes[3 * n + i]).abs() / twin.state.planes[3 * n + i].abs())
        .fold(0.0, f64::max);
    let moved = near.iter().map(|&i| (f64::from(after[3 * n + i]) / f64::from(before[3 * n + i]) - 1.0).abs()).fold(0.0, f64::max);
    println!(
        "SR-HELD-WORK-GPU the wall's neighbours' E off the twin's by {worst:.3e} (band {WORK_BAND:e}; E moved by up to {moved:.3e}); \
         held GPU {:?}, twin {:?} (off {off:.3e})",
        gpu.held(),
        twin.held
    );
    assert!(twin.held[1].abs() > 0.0 || twin.held[0].abs() > 0.0, "the twin booked no kick: the case grades nothing");
    assert!(off <= SPARED_BAND, "the held column was spared {:?}, the twin's {:?}", gpu.held(), twin.held);
    assert!(worst <= WORK_BAND, "the wall's neighbours' E off the twin's by {worst:e}: work done across the wall");
}

/// §2.3.4 [M0-D172] on the GPU: [`a_free_cell_stops_at_a_held_wall`]'s fall at 64 × 64, one step against the twin — the
/// block held as built, every free cell falling into it stopped at exactly 0 (`hydro_commit`), the cells moving away
/// still moving, the stopped momentum booked `held` as the twin's, grid + held momentum kept, and every cell's E the twin's.
#[test]
fn gpu_a_free_cell_stops_at_a_held_wall_as_the_twins() {
    const B: usize = 30;
    let physics = physics();
    let (st, block) = fall_world(GW, B);
    let before = to_f32(&st);
    let (after, gpu, twin) = strip_step(&physics, &before, EdgeMode::Bounce, false);
    let n = GW * GW;
    assert_eq!(gpu.mask(), block, "the block is not held as built: the case grades no wall");
    let into: Vec<f32> = wall_neighbours(GW, B).iter().filter(|w| w.2).map(|&(i, k, _)| after[(1 + k) * n + i]).collect();
    let away: Vec<f32> = wall_neighbours(GW, B).iter().filter(|w| !w.2).map(|&(i, k, _)| after[(1 + k) * n + i]).collect();
    let (p0, scale) = gpu_momentum(&before, n);
    let (p1, held) = (gpu_momentum(&after, n).0, gpu.held());
    let drift = [0, 1].map(|k| (p1[k] + held[k] - p0[k]).abs() / scale);
    let off = spared_off(&gpu, &twin);
    let worst = (0..n)
        .filter(|&i| !block[i])
        .map(|i| (f64::from(after[3 * n + i]) - twin.state.planes[3 * n + i]).abs() / twin.state.planes[3 * n + i].abs())
        .fold(0.0, f64::max);
    println!(
        "SR-HELD-STICKY-GPU into the wall {into:?}; away {away:?}; held GPU {held:?}, twin {:?} (off {off:.3e}); drift {drift:?}; \
         free E off the twin's by {worst:.3e}",
        twin.held
    );
    assert!(into.iter().all(|&p| p == 0.0), "a free cell's momentum into the held block after the step: {into:?}");
    assert!(away.iter().all(|&p| p > 0.0), "a free cell moving away from the block lost its momentum: {away:?}");
    assert!(off <= SPARED_BAND, "the held block booked {held:?}, the twin's {:?}", twin.held);
    assert!(drift[0] <= GPU_STRIP_BAND && drift[1] <= GPU_STRIP_BAND, "grid + held momentum drifted {drift:?}");
    assert!(worst <= WORK_BAND, "a free cell's E off the twin's by {worst:e}");
}

#[test]
fn gpu_a_held_edge_cell_lets_nothing_escape() {
    let physics = physics();
    let eos = Eos::from_physics(&physics).unwrap();
    let before = gpu_strip(&eos, &[0], 1.0, |_| 0.0);
    let (after, gpu, _) = strip_step(&physics, &before, EdgeMode::Leave, false);
    assert_gpu_columns_kept(&gpu, &before, &after, &[0]);
    // Slots: x = 0 by y, x = 63 by y, y = 0 by x, y = 63 by x. The held column's: the x = 0 side's and x = 0 on y's sides.
    let book = gpu.book();
    let held_slot = |s: usize| s < GW || s == 2 * GW || s == 3 * GW;
    let mut leaked = Vec::new();
    for t in TERMS.iter().filter(|t| t.name.starts_with("escaped.")) {
        let plane = term_plane(gpu.world, &book, t.name);
        leaked.extend((0..plane.len()).filter(|&s| held_slot(s) && plane[s] != 0.0).map(|s| (t.name, s, plane[s])));
    }
    let free = term_plane(gpu.world, &book, "escaped.mom_y");
    let free_booked = (2 * GW + 1..3 * GW).filter(|&s| free[s] != 0.0).count();
    println!("SR-HELD-EDGE-GPU {} held slots booked, {free_booked} of the free cells' y = 0 slots booked", leaked.len());
    assert!(free_booked > 0, "no free edge cell booked escaped: the case grades no booking");
    assert!(leaked.is_empty(), "the held edge cells booked escaped: {:?}", &leaked[..leaked.len().min(4)]);
}

#[test]
fn gpu_clear_leaves_p4_no_stale_wall() {
    // M0-D71b's flag: P0's Clear zeroes the mask, which only P4 — between P0 and P8 — can tell. A cold C/O core held by
    // the rule, then a Clear and a paint across the core's place in one step: the planes the same edits give with the
    // mask zeroed by hand, bit for bit; the paint's edge lies inside the core's place, so a stale wall would hold gas there.
    let physics = physics();
    let eos = Eos::from_physics(&physics).unwrap();
    let block = |i: usize| (30..34).contains(&(i % GW)) && (30..34).contains(&(i / GW));
    let hy = fractions(&[("H", 1.0)]);
    let cells: Vec<Cell> = (0..GW * GW)
        .map(|i| if block(i) { cold_cell(&eos, 50.0, co(), 0.5 * hold_key("eta_hold"), [0.0, 0.0]) } else { Cell { sigma: 1.0, mom: [0.0; 2], energy: 1.0, x: hy } })
        .collect();
    let planes = to_f32(&state(GW, GW, &cells));
    let world = WorldConfig::new(GW as u32, GW as u32).unwrap();
    let h = elements().index_of("H").unwrap();
    let run = |zeroed: bool| {
        let gpu = Gpu::new(&physics, world, &planes, EdgeMode::Bounce, false, true);
        gpu.run(1);
        let held = gpu.mask().iter().filter(|&&m| m).count();
        if zeroed {
            gpu.state.upload_held(&gpu.gpu.queue, &vec![0; GW * GW]).unwrap();
        }
        for e in [Edit::Clear, Edit::Paint { cell: [28, 31], radius: 3, species: h }] {
            gpu.step.queue_edit(&gpu.gpu.queue, &e).unwrap();
        }
        gpu.run(1);
        (held, gpu.planes())
    };
    let (held, cleared) = run(false);
    let (_, reference) = run(true);
    let n = GW * GW;
    let moved = (0..n).filter(|&i| block(i) && (reference[n + i] != 0.0 || reference[2 * n + i] != 0.0)).count();
    let differ = (0..cleared.len()).filter(|&k| cleared[k].to_bits() != reference[k].to_bits()).count();
    println!("SR-HELD-CLEAR-GPU {held} held before the Clear; {moved} of the core's cells moving after it; {differ} values differ");
    assert_eq!(held, 16, "the core is not held before the Clear: the case grades nothing");
    assert!(moved > 0, "no cell of the core's place moved: a stale wall would show nothing");
    assert_eq!(differ, 0, "the Clear's step differs from the same step with the mask zeroed: P4 read a stale mask");
}

#[test]
fn gpu_no_iron_is_held() {
    // Item 1 on the GPU (§2.7 [M0-D71f], floors.wgsl's `hold`, f32 alike): item 1's five cells in a row of a 64 × 64
    // world of thin warm hydrogen (Σ 1, never held), 12 cells apart; the mask after the initial hold and one step, then
    // the same planes uploaded with the fourth cell's iron appeared (the mask kept) and one step more.
    let physics = physics();
    let eos = Eos::from_physics(&physics).unwrap();
    let world = WorldConfig::new(GW as u32, GW as u32).unwrap();
    let at: [usize; 5] = std::array::from_fn(|k| 32 * GW + 8 + 12 * k);
    let hy = fractions(&[("H", 1.0)]);
    let planes = |iron_appears: bool| {
        let (five, _, _) = item1_cells(&eos, &physics, iron_appears);
        let cells: Vec<Cell> = (0..GW * GW)
            .map(|i| at.iter().position(|&a| a == i).map_or(Cell { sigma: 1.0, mom: [0.0; 2], energy: 1.0, x: hy }, |k| five[k]))
            .collect();
        to_f32(&state(GW, GW, &cells))
    };
    let gpu = Gpu::new(&physics, world, &planes(false), EdgeMode::Bounce, false, false);
    let read = |gpu: &Gpu| {
        let mask = gpu.mask();
        (at.map(|i| mask[i]), mask.iter().filter(|&&m| m).count())
    };
    gpu.run(1);
    let (first, n_first) = read(&gpu);
    gpu.state.upload(&gpu.gpu.queue, &planes(true)).unwrap();
    gpu.run(1);
    let (second, n_second) = read(&gpu);
    println!("SR-HELD-NO-IRON GPU: masks {first:?} ({n_first} held) then {second:?} ({n_second} held)");
    assert_eq!(first, ITEM1[0], "iron under and past N_Fe's gate, C/O beside it, C/O with X_Fe 9e-4");
    assert_eq!(second, ITEM1[1], "the held C/O cell with X_Fe 1.1e-3");
    assert_eq!((n_first, n_second), (3, 2), "cells held off item 1's");
}

#[test]
fn gpu_a_neutron_core_is_held_under_m_tov_sb() {
    // The neutron hold on the GPU (§2.7 [M0-R13], §0.5 [M0-T130]; floors.wgsl's `hold` and `disk_mass`, f32 alike): the
    // twin's probes, the mask after one step, then the planes uploaded with the fourth's disk crossed (the mask kept) and
    // one step more. One hydro step moves the ballast by less than a cell, its disk's mass within the 5 % margin.
    let physics = physics();
    let eos = Eos::from_physics(&physics).unwrap();
    let m_tov = Floors::new(&physics, &elements()).unwrap().m_tov();
    let world = WorldConfig::new(NW as u32, NW as u32).unwrap();
    let planes = |crossed: bool| to_f32(&state(NW, NW, &neutron_probes(&eos, m_tov, crossed).0));
    let at = neutron_probes(&eos, m_tov, false).1;
    let gpu = Gpu::new(&physics, world, &planes(false), EdgeMode::Bounce, false, false);
    let read = |gpu: &Gpu| {
        let mask = gpu.mask();
        (at.map(|i| mask[i]), mask.iter().filter(|&&m| m).count())
    };
    gpu.run(1);
    let (first, n_first) = read(&gpu);
    gpu.state.upload(&gpu.gpu.queue, &planes(true)).unwrap();
    gpu.run(1);
    let (second, n_second) = read(&gpu);
    println!("SR-HELD-NEUTRON GPU: masks {first:?} ({n_first} held) then {second:?} ({n_second} held), m_tov_sb {m_tov:.2}");
    assert_eq!(first, NEUTRON[0], "neutron-rich at 0.95 and 1.05 m_tov_sb, iron at X_n 0.4, neutron-rich at 0.95");
    assert_eq!(second, NEUTRON[1], "the fourth's disk crossed to 1.05 m_tov_sb");
    assert_eq!((n_first, n_second), (2, 1), "cells held off the probes'");
}

// ---- G-REST's window on the GPU (M0-D71f) ----

/// The window's source, the Sun-like life's step 450,000 (git-ignored: on linux-pc and laserax-ai-server, not in a clone).
const WINDOW_SOURCE: &str = "build/v9-sun-life/dumps/s_450000.bin";
/// M0-D67's box of the source, x, y, w, h, set at (`WINDOW_AT`, `WINDOW_AT`) of a `WINDOW` × `WINDOW` world.
const WINDOW_BOX: [usize; 4] = [248, 144, 104, 104];
const WINDOW: usize = 128;
const WINDOW_AT: usize = 12;
/// G-REST's ruled bound on the window: floor_added over its 10,000 steps against radiated.
const WINDOW_SHARE: f64 = 0.15;

fn repo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenes_dir() -> PathBuf {
    repo_dir().join("scenes")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-held-core-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

/// The window cut from the source dump: every plane's box (Σ … X, F, the rate and the mask when the dump holds them) at
/// (`WINDOW_AT`, `WINDOW_AT`), the source's corner cell (vacuum) outside — M0-R11's crop — and the active box moved with it,
/// so the window's box, its FFT size and its records are the life's: outside the box is vacuum, and gravity free-space.
fn cut_window(src: &Dump) -> Dump {
    let (w0, n0, n) = (src.world.width as usize, src.world.cells(), WINDOW * WINDOW);
    assert_eq!(src.resume.box_record[..4].iter().map(|&v| v as usize).collect::<Vec<_>>(), WINDOW_BOX, "the source's active box");
    assert!(src.sinks.is_empty(), "the source holds sinks");
    fn crop<T: Copy + Default>(planes: &[T], w0: usize, n0: usize, n: usize) -> Vec<T> {
        let mut out = vec![T::default(); planes.len() / n0 * n];
        for (c, plane) in planes.chunks(n0).enumerate() {
            for y in 0..WINDOW {
                for x in 0..WINDOW {
                    let inside = (WINDOW_AT..WINDOW_AT + WINDOW_BOX[2]).contains(&x) && (WINDOW_AT..WINDOW_AT + WINDOW_BOX[3]).contains(&y);
                    let from = if inside { (y - WINDOW_AT + WINDOW_BOX[1]) * w0 + x - WINDOW_AT + WINDOW_BOX[0] } else { 0 };
                    out[c * n + y * WINDOW + x] = plane[from];
                }
            }
        }
        out
    }
    let mut resume = src.resume;
    resume.box_record[0] = WINDOW_AT as u32;
    resume.box_record[1] = WINDOW_AT as u32;
    Dump {
        world: WorldConfig::new(WINDOW as u32, WINDOW as u32).unwrap(),
        step: src.step,
        sim_time: src.sim_time,
        sinks: Vec::new(),
        resume,
        planes: crop(&src.planes, w0, n0, n),
        flux: src.flux.as_deref().map(|f| crop(f, w0, n0, n)),
        rate: src.rate.as_deref().map(|r| crop(r, w0, n0, n)),
        held: src.held.as_deref().map(|h| crop(h, w0, n0, n)),
    }
}

/// scenes/remnant_window.bin, checked against the cut where the source is on disk: stale is a failure, never a silent
/// re-cut (`SR_WRITE_SCENES=1` writes it first, the one way to refresh it); where the source is absent it is read as it is.
fn window_file() -> PathBuf {
    let path = scenes_dir().join("remnant_window.bin");
    let source = repo_dir().join(WINDOW_SOURCE);
    match Dump::read(&source) {
        Ok(src) => {
            let cut = cut_window(&src);
            if std::env::var_os("SR_WRITE_SCENES").is_some() {
                cut.write(&path).expect("write scenes/remnant_window.bin");
            }
            let file = Dump::read(&path).unwrap_or_else(|e| panic!("{e} — SR_WRITE_SCENES=1 bash tests/gpu.sh held_core physics writes it"));
            assert!(file == cut, "scenes/remnant_window.bin is not the cut of {WINDOW_SOURCE} — SR_WRITE_SCENES=1 bash tests/gpu.sh held_core physics rewrites it");
            println!("SR-HELD-WINDOW scenes/remnant_window.bin is the cut of {WINDOW_SOURCE}");
        }
        Err(e) => {
            assert!(std::env::var_os("SR_WRITE_SCENES").is_none(), "SR_WRITE_SCENES=1 needs {WINDOW_SOURCE}: {e}");
            println!("SR-HELD-WINDOW {WINDOW_SOURCE} is not here ({e}): scenes/remnant_window.bin read as it is, not re-cut");
        }
    }
    path
}

/// A ledger number: summary.json writes f64s, an observers' snapshot their exact text.
fn number(v: &Value) -> f64 {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())).unwrap_or_else(|| panic!("not a number: {v}"))
}

#[test]
fn gpu_the_remnant_window_shines_on_its_own_heat() {
    // G-REST's window: the cut 10,000 steps on the GPU, every pass of the Sun preset's scene in a leave world; P8's
    // floor_added ≤ 15 % of the light radiated. Both halves printed: the twin's crop under H read 1.2 % and 46 %, 3.8 % in
    // all (M0-R11), so a whole-run share can hide a second half fed by the floor.
    let file = window_file();
    let from = Dump::read(&file).unwrap().step;
    let dir = scratch("window");
    let mut args: Vec<String> = ["--scene", "preset:sun", "--load-dump", &file.display().to_string(), "--edge", "leave", "--steps", &STEPS.to_string(), "--dump-every", &(STEPS / 2).to_string(), "--out", &dir.display().to_string()]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(adapter) = std::env::var("SR_TEST_ADAPTER").ok().filter(|s| !s.trim().is_empty()) {
        args.extend(["--adapter".to_string(), adapter]);
    }
    assert_eq!(headless::run(&args), 0, "headless {args:?}");
    let summary: Value = serde_json::from_slice(&std::fs::read(dir.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["steps"].as_u64(), Some(STEPS), "the window's steps");
    let mid_step = from + STEPS / 2;
    let observers: Value = serde_json::from_slice(&std::fs::read(dir.join(format!("dumps/s_{mid_step}.observers.json"))).unwrap()).unwrap();
    let (start, mid, end) = (&summary["ledger"]["start"], &observers["ledger"]["last"], &summary["ledger"]["end"]);
    assert_eq!(mid["step"].as_u64().or_else(|| mid["step"].as_str().and_then(|s| s.parse().ok())), Some(mid_step), "the mid record's step");
    let e = |r: &Value, k: &str| number(&r["energy"][k]);
    let span = |a: &Value, b: &Value| (e(b, "floor_added") - e(a, "floor_added"), e(b, "radiated") - e(a, "radiated"));
    let (halves, (floor, radiated)) = ([span(start, mid), span(mid, end)], span(start, end));
    let last = Dump::read(&dir.join(format!("dumps/s_{}.bin", from + STEPS))).unwrap();
    let held = last.held.as_ref().map_or(0, |h| h.iter().filter(|&&m| m != 0).count());
    let drift = e(end, "invariant") - e(start, "invariant");
    println!(
        "SR-HELD-WINDOW GPU {}: floor_added {floor:.4e} of radiated {radiated:.4e} = {:.2} % over {STEPS} steps (bound {:.0} %; \
         M0-R11's twin 3.8 %); halves {:.2} % and {:.2} % ({:.3e}/{:.3e}, {:.3e}/{:.3e}); held {held} at step {}; the energy \
         invariant {:+.4e} of |W| {:.4e}",
        summary["adapter"]["name"],
        100.0 * floor / radiated,
        100.0 * WINDOW_SHARE,
        100.0 * halves[0].0 / halves[0].1,
        100.0 * halves[1].0 / halves[1].1,
        halves[0].0,
        halves[0].1,
        halves[1].0,
        halves[1].1,
        from + STEPS,
        drift,
        e(start, "potential").abs()
    );
    assert!(held > 0, "no cell held at the window's end: the hold never fired");
    assert!(radiated > 0.0, "the window radiated nothing");
    assert!(floor <= WINDOW_SHARE * radiated, "floor_added {floor} over the window's {STEPS} steps against radiated {radiated} (bound {WINDOW_SHARE})");
    let _ = std::fs::remove_dir_all(dir);
}
