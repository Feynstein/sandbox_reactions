// P8 floors (m0_contrat.md §1.3.2, §2.7, §2.3.3, §2.9): two dispatches, in this order — `floor_cells` (the vacuum reset,
// then the temperature floor) and `renormalise_species`. Compiled with eos.wgsl prepended (group 1).
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
// Both run over the active box only (§1.3.3), as many 8 × 8 tiles as its indirect size says, from its origin: outside
// it every cell is vacuum and static.

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
// booking.floors (state.rs BOOK_GROUPS): mass.vacuum_reset's plane, then energy.floor_added's, one slot per cell.
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

const PLANES_A: u32 = 5u;
const PLANES_B: u32 = 5u;

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

@compute @workgroup_size(8, 8)
fn renormalise_species(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
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
}
