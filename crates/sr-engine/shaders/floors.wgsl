// P8 floors (m0_contrat.md §1.3.2, §2.7, §2.3.3, §2.9): two dispatches, in this order — `floor_cells` (the vacuum reset,
// then the temperature floor) and `renormalise_species`, which ends with the hold. Compiled with eos.wgsl prepended
// (group 1).
//
// Species renormalisation, one cell per invocation: every mass fraction clamped ≥ 0, then divided by their sum, so
// Σ_i X_i = 1. A cell whose fractions are all ≤ 0 becomes pure hydrogen (§2.7 [M0-T19]). The sum runs in §1.5's order,
// cell by cell, with no atomic: the result is the same on every run (§1.3.4).
//
// The floors read μ, Y_e and X_n of the fractions as the renormalisation will leave them (`renormalised`, the one
// helper both kernels call), so the gas they floor is the gas the step ends with. A cell below Σ_vac is reset to the
// floor state — Σ_floor, u = 0, T = T_floor, its fractions kept — and the mass it loses is booked `vacuum_reset`;
// any other cell whose ε_th = E/Σ − ½|u|² − ε_cold is below T_floor/μ is raised to it. Every energy P8 changes, the
// reset's and the floor's, is booked `floor_added` (signed; §2.9 has no other term for it — §2.7 [M0-T19]). Each
// invocation adds into its own booking slot, no atomic. A NaN Σ or E is left to P9's guard, and so is a cell holding
// an Inf or a NaN fraction: neither kernel touches it — no floor, no booking, its fractions as read — since the clamp's
// max(NaN, 0) may return 0 and renormalise the NaN away before the guard reads it (M0-D18).
//
// The hold (§2.7 [M0-TG2]; M0-D71's twin, sr_physics::reference::floors `Floors::hold`), P8's last act, in the
// renormalisation's invocation once it has written the fractions: a cell with Σ ≥ σ_hold whose ε_th — as the floor left
// it — is at most η_hold · ε_cold of the renormalised fractions, which holds no iron (X_Fe = 0 exactly in f32, §2.7
// [M0-D71f]) or is neutron-rich (X_n ≥ 0.5, [M0-R13]), and whose disk r ≤ r_hold holds less than m_tov_sb (§0.5 [M0-T130],
// M0-D153: sinks.wgsl's mass test's disk form at the twin's r_hold, summed row-major in f32 as `core` sums it, of Σ as
// `floor_cells` left it — no invocation here writes a Σ — and only where the rest of the test holds or keeps the cell,
// as the twin calls it) is held; a held cell is released past 2η_hold · ε_cold, under ½σ_hold, once X_Fe > 0 with
// X_n < 0.5, or once its disk holds m_tov_sb. The mask (state.rs `state.held`, one word per cell) is the next step's P4's. A newly held cell's
// momentum — read by the test before — is set to 0, its kinetic energy left in E, and added into `held`'s two slots.
// Every comparison is f32 of f32 inputs: a cell within f32's rounding of an edge may fall either side of the twin's f64.
// `hold_cells` is the same hold alone, on the initial state before step 0 (step/mod.rs), so its zeroing is booked too.
//
// All three run over the active box only (§1.3.3), as many 8 × 8 tiles as its indirect size says, from its origin:
// outside it every cell is vacuum and static, and a held cell (Σ ≥ ½σ_hold) is never vacuum.

struct Dims {
    width: u32,
    height: u32,
    cells: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The species planes, row-major, in §2.2's order: x_H, x_He, x_C, x_O, x_Ne, then x_Mg, x_Si, x_S, x_Fe, x_n.
@group(0) @binding(1) var<storage, read_write> species_a: array<f32>;
@group(0) @binding(2) var<storage, read_write> species_b: array<f32>;
// The hydro planes: sigma, mom_x, mom_y, energy.
@group(0) @binding(3) var<storage, read_write> hydro: array<f32>;
// booking.floors (state.rs BOOK_GROUPS): mass.vacuum_reset's plane, then energy.floor_added's, held.mom_x's and
// held.mom_y's, one slot per cell.
@group(0) @binding(4) var<storage, read_write> book: array<f32>;

// The active box (reduce.wgsl `ActiveBox`, boxfit.rs).
struct ActiveBox {
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    fft_w: u32,
    fft_h: u32,
    from_step: u32,
    fits: u32,
}

@group(0) @binding(5) var<uniform> abox: ActiveBox;

// The hold's mask: 1 where a cell is held.
@group(0) @binding(6) var held: texture_storage_2d<r32uint, read_write>;

// σ_hold, η_hold and the gate's radius r_hold (physics.json's sigma_hold, eta_hold and r_hold, §2.7 [M0-D71g]) and
// m_tov_sb (f32's largest when the neutrons' cold pressure is off: no disk reaches it).
struct Hold {
    sigma_hold: f32,
    eta_hold: f32,
    m_tov: f32,
    r_hold: f32,
}

@group(0) @binding(7) var<uniform> hold_params: Hold;

const PLANES_A: u32 = 5u;
const PLANES_B: u32 = 5u;
// Iron's slot in §1.5's order: a cell holding any is held only once neutron-rich (§2.7 [M0-D71f], [M0-R13]).
const ID_FE: u32 = 8u;
// X_n from which a cell holding iron is neutron-rich (§2.7 [M0-R13]).
const NEUTRON_RICH: f32 = 0.5;

// An Inf or a NaN, told by its exponent bits as reduce.wgsl's guard tells it — never a float comparison, which WGSL may
// fold away.
fn non_finite(v: f32) -> bool {
    return (bitcast<u32>(v) & 0x7f800000u) == 0x7f800000u;
}

// Whether any of the cell's fractions is an Inf or a NaN: such a cell is left for P9's guard.
fn fractions_non_finite(cell: u32) -> bool {
    let n = dims.cells;
    for (var i = 0u; i < PLANES_A; i++) {
        if (non_finite(species_a[i * n + cell])) {
            return true;
        }
    }
    for (var i = 0u; i < PLANES_B; i++) {
        if (non_finite(species_b[i * n + cell])) {
            return true;
        }
    }
    return false;
}

// The cell's fractions clamped ≥ 0 and divided by their sum; pure hydrogen when none is positive.
fn renormalised(cell: u32) -> array<f32, 10> {
    let n = dims.cells;
    var x: array<f32, 10>;
    var sum = 0.0;
    for (var i = 0u; i < PLANES_A; i++) {
        x[i] = max(species_a[i * n + cell], 0.0);
        sum += x[i];
    }
    for (var i = 0u; i < PLANES_B; i++) {
        x[PLANES_A + i] = max(species_b[i * n + cell], 0.0);
        sum += x[PLANES_A + i];
    }

    if (sum > 0.0) {
        for (var i = 0u; i < PLANES_A + PLANES_B; i++) {
            x[i] = x[i] / sum;
        }
    } else {
        for (var i = 0u; i < PLANES_A + PLANES_B; i++) {
            x[i] = 0.0;
        }
        x[0] = 1.0;
    }
    return x;
}

@compute @workgroup_size(8, 8)
fn floor_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
    let n = dims.cells;
    if (fractions_non_finite(cell)) {
        return;
    }

    let x = renormalised(cell);
    let comp = composition(x);
    let x_n = x[ID_N];
    let eps_floor = eps_th_floor(comp.inv_mu);

    let sigma = hydro[cell];
    let energy = hydro[3u * n + cell];
    var mass_out = 0.0;
    var energy_added = 0.0;
    if (sigma < eos.sigma_vac) {
        let s = eos.sigma_floor;
        let e = s * (eps_floor + eps_cold(s, comp.y_e, x_n));
        hydro[cell] = s;
        hydro[n + cell] = 0.0;
        hydro[2u * n + cell] = 0.0;
        hydro[3u * n + cell] = e;
        mass_out = sigma - s;
        energy_added = e - energy;
    } else {
        let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
        let raw = eps_th_raw(sigma, mom, energy, comp.y_e, x_n);
        if (raw < eps_floor) {
            let e = sigma * (eps_floor + eps_cold(sigma, comp.y_e, x_n)) + 0.5 * dot(mom, mom) / sigma;
            hydro[3u * n + cell] = e;
            energy_added = e - energy;
        }
    }
    book[cell] += mass_out;
    book[n + cell] += energy_added;
}

// The mass of the disk r ≤ r_hold about (x, y), centre to centre, clipped to the world: row-major, as sinks.wgsl's `core`.
fn disk_mass(xy: vec2<u32>) -> f32 {
    let r = i32(hold_params.r_hold);
    var mass = 0.0;
    for (var dy = -r; dy <= r; dy++) {
        for (var dx = -r; dx <= r; dx++) {
            let cx = i32(xy.x) + dx;
            let cy = i32(xy.y) + dy;
            if (dx * dx + dy * dy <= r * r && cx >= 0 && cy >= 0 && cx < i32(dims.width) && cy < i32(dims.height)) {
                mass += hydro[u32(cy) * dims.width + u32(cx)];
            }
        }
    }
    return mass;
}

// The hold at `cell` (x, y), whose renormalised fractions are `x`: the mask's word written, a newly held cell's momentum
// zeroed and booked.
fn hold(cell: u32, xy: vec2<u32>, x: array<f32, 10>) {
    let n = dims.cells;
    let comp = composition(x);
    let sigma = hydro[cell];
    let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
    let cold = eps_cold(sigma, comp.y_e, x[ID_N]);
    let eps_th = hydro[3u * n + cell] / sigma - 0.5 * dot(mom / sigma, mom / sigma) - cold;
    let kind = x[ID_FE] == 0.0 || x[ID_N] >= NEUTRON_RICH;
    let was = textureLoad(held, xy).x != 0u;
    var now: bool;
    if (was) {
        now = !(eps_th > 2.0 * hold_params.eta_hold * cold || sigma < 0.5 * hold_params.sigma_hold || !kind);
    } else {
        now = sigma >= hold_params.sigma_hold && eps_th <= hold_params.eta_hold * cold && kind;
    }
    // The gate (§0.5 [M0-T130]): the disk summed only where the rest holds or keeps the cell, as the twin's `holds`.
    if (now) {
        now = disk_mass(xy) < hold_params.m_tov;
    }
    if (now && !was) {
        book[2u * n + cell] += mom.x;
        book[3u * n + cell] += mom.y;
        hydro[n + cell] = 0.0;
        hydro[2u * n + cell] = 0.0;
    }
    textureStore(held, xy, vec4<u32>(select(0u, 1u, now), 0u, 0u, 0u));
}

@compute @workgroup_size(8, 8)
fn renormalise_species(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let xy = vec2<u32>(abox.x0 + id.x, abox.y0 + id.y);
    let cell = xy.y * dims.width + xy.x;
    let n = dims.cells;
    if (fractions_non_finite(cell)) {
        return;
    }

    let x = renormalised(cell);
    for (var i = 0u; i < PLANES_A; i++) {
        species_a[i * n + cell] = x[i];
    }
    for (var i = 0u; i < PLANES_B; i++) {
        species_b[i * n + cell] = x[PLANES_A + i];
    }
    hold(cell, xy, x);
}

@compute @workgroup_size(8, 8)
fn hold_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let xy = vec2<u32>(abox.x0 + id.x, abox.y0 + id.y);
    let cell = xy.y * dims.width + xy.x;
    if (fractions_non_finite(cell)) {
        return;
    }
    hold(cell, xy, renormalised(cell));
}
