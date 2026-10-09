// P8 floors (m0_contrat.md §1.3.2, §2.7): vacuum reset, temperature floor, then the species renormalised. Today only
// the last — the vacuum reset and the temperature floor arrive with M0-T19, ahead of it in P8's dispatch list.
//
// Species renormalisation, one cell per invocation: every mass fraction clamped ≥ 0, then divided by their sum, so
// Σ_i X_i = 1. A cell whose fractions are all ≤ 0 becomes pure hydrogen (M0-T3's declared default; the contract is
// silent). The sum runs in §1.5's order, cell by cell, with no atomic: the result is the same on every run (§1.3.4).
// The state's other channels are not bound.

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

const PLANES_A: u32 = 5u;
const PLANES_B: u32 = 5u;

@compute @workgroup_size(8, 8)
fn renormalise_species(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }
    let cell = id.y * dims.width + id.x;
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

    for (var i = 0u; i < PLANES_A; i++) {
        species_a[i * n + cell] = x[i];
    }
    for (var i = 0u; i < PLANES_B; i++) {
        species_b[i * n + cell] = x[PLANES_A + i];
    }
}
