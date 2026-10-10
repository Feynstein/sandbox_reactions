// P1 Δt and P9's reductions (m0_contrat.md §1.3.2, §1.3.4, §2.3.3, §2.3.5, §3.3). Compiled with eos.wgsl prepended
// (group 1).
//
// Δt_{n+1} = min(C · min_cells [(|u| + c)/Δx + (|v| + c)/Δy]⁻¹, η_g · min_cells √(Δx/|g|)), Δx = Δy = 1 cell (§2.1),
// reduced as the largest rate of each term — r_h = (|u| + c)/Δx + (|v| + c)/Δy and r_g = |g|/Δx — so Δt_{n+1} =
// min(C / max r_h, η_g / √(max r_g)); a term whose largest rate is 0 does not bound Δt. Until P2 writes g, r_g is 0
// and the gravity term infinite. c² = 2Π/Σ with ε_th floored at T_floor/μ (§2.3.3); a cell below Σ_vac counts as the
// floor state P8 leaves it in (Σ_floor, u = 0, T = T_floor), so the reduction of an initial state and of a floored
// one agree. The fractions are taken as stored: P9 runs after P8's renormalisation.
//
// The reduction is a fixed-order tree (§1.3.4): `dt_cells` takes 256 consecutive cells (row-major) per workgroup and
// halves them in shared memory, pairing slot i with slot i + s for s = 128, 64, …, 1; `dt_partials`, one workgroup,
// walks the partials in strides of 256 in index order, then halves the same way and writes Δt_{n+1}. No atomic, no
// scheduling-dependent order. P1's `dt_advance` makes it step n + 1's Δt.
//
// The non-finite guard (every 64 steps, §1.3.2 P9): `guard_scan` reads all 14 channels of every cell; a NaN or Inf is
// told by its bits (exponent all ones), never by a float comparison WGSL may fold away. Each finding is atomicMax'd as
// 0xFFFFFFFF − (cell · 16 + channel), order-independent (§1.3.4), so the lowest cell, then its lowest channel, wins.
// `guard_stamp` then records the first finding with the step it was made at; once recorded, later scans stop.

struct Dims {
    width: u32,
    height: u32,
    cells: u32,
    _pad: u32,
}

struct DtParams {
    cfl: f32,
    eta_g: f32,
    partials: u32,
    _pad: u32,
}

// The Δt record, one per run (dt.rs `DtRecord`): Δt_n of the step in flight, Δt_{n+1} as P9 left it, the index of the
// step in flight and the steps begun; the guard's running finding, then the recorded one (step, code) and its flag.
struct DtState {
    dt: f32,
    dt_next: f32,
    step: u32,
    steps: u32,
    found: atomic<u32>,
    bad_step: u32,
    bad_code: u32,
    flagged: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The hydro planes (sigma, mom_x, mom_y, energy), then the species planes (x_H … x_Ne, x_Mg … x_n), row-major.
@group(0) @binding(1) var<storage, read> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
// One (max r_h, max r_g) per `dt_cells` workgroup.
@group(0) @binding(4) var<storage, read_write> partials: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> rec: DtState;
@group(0) @binding(6) var<uniform> params: DtParams;

const WG: u32 = 256u;
const DX: f32 = 1.0;
const DY: f32 = 1.0;
const PLANES_A: u32 = 5u;
const PLANES_B: u32 = 5u;
// The largest finite f32: Δt when no term bounds it.
const DT_UNBOUNDED: f32 = 3.40282347e38;
const NONE: u32 = 0u;

var<workgroup> tree: array<vec2<f32>, 256>;

// The cell's two rates, (r_h, r_g).
fn cell_rates(cell: u32) -> vec2<f32> {
    let n = dims.cells;
    var x: array<f32, 10>;
    for (var i = 0u; i < PLANES_A; i++) {
        x[i] = species_a[i * n + cell];
    }
    for (var i = 0u; i < PLANES_B; i++) {
        x[PLANES_A + i] = species_b[i * n + cell];
    }
    let comp = composition(x);
    let x_n = x[ID_N];
    let eps_floor = eps_th_floor(comp.inv_mu);

    var sigma = hydro[cell];
    var vel = vec2<f32>(0.0, 0.0);
    var eps_th = eps_floor;
    if (sigma < eos.sigma_vac) {
        sigma = eos.sigma_floor;
    } else {
        let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
        vel = mom / sigma;
        eps_th = max(eps_th_raw(sigma, mom, hydro[3u * n + cell], comp.y_e, x_n), eps_floor);
    }
    let c = sqrt(max(wave_speed_sq(sigma, pressure(sigma, eps_th, comp.y_e, x_n)), 0.0));
    let r_h = (abs(vel.x) + c) / DX + (abs(vel.y) + c) / DY;
    let r_g = 0.0; // |g|/Δx once P2 writes g (§1.4, §2.3.5)
    return vec2<f32>(r_h, r_g);
}

// Halves `tree` in place, slot i taking max(slot i, slot i + s) for s = 128 … 1; slot 0 holds the workgroup's max.
fn halve(lid: u32) {
    for (var s = WG / 2u; s > 0u; s = s / 2u) {
        workgroupBarrier();
        if (lid < s) {
            tree[lid] = max(tree[lid], tree[lid + s]);
        }
    }
    workgroupBarrier();
}

@compute @workgroup_size(256)
fn dt_cells(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    let cell = wg.x * WG + lid;
    var r = vec2<f32>(0.0, 0.0);
    if (cell < dims.cells) {
        r = cell_rates(cell);
    }
    tree[lid] = r;
    halve(lid);
    if (lid == 0u) {
        partials[wg.x] = tree[0];
    }
}

@compute @workgroup_size(256)
fn dt_partials(@builtin(local_invocation_index) lid: u32) {
    var r = vec2<f32>(0.0, 0.0);
    for (var k = lid; k < params.partials; k += WG) {
        r = max(r, partials[k]);
    }
    tree[lid] = r;
    halve(lid);
    if (lid == 0u) {
        let top = tree[0];
        var dt = DT_UNBOUNDED;
        if (top.x > 0.0) {
            dt = min(dt, params.cfl / top.x);
        }
        if (top.y > 0.0) {
            dt = min(dt, params.eta_g / sqrt(top.y));
        }
        rec.dt_next = dt;
    }
}

// P1: step n's Δt is the one P9 made at the end of step n − 1 (or the initial state's reduction, before step 0).
@compute @workgroup_size(1)
fn dt_advance() {
    rec.dt = rec.dt_next;
    rec.step = rec.steps;
    rec.steps = rec.steps + 1u;
}

fn non_finite(v: f32) -> bool {
    return (bitcast<u32>(v) & 0x7f800000u) == 0x7f800000u;
}

@compute @workgroup_size(8, 8)
fn guard_scan(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height || rec.flagged != 0u) {
        return;
    }
    let cell = id.y * dims.width + id.x;
    let n = dims.cells;
    var channel = 0xffffffffu;
    for (var c = 0u; c < 4u && channel == 0xffffffffu; c++) {
        if (non_finite(hydro[c * n + cell])) {
            channel = c;
        }
    }
    for (var c = 0u; c < PLANES_A && channel == 0xffffffffu; c++) {
        if (non_finite(species_a[c * n + cell])) {
            channel = 4u + c;
        }
    }
    for (var c = 0u; c < PLANES_B && channel == 0xffffffffu; c++) {
        if (non_finite(species_b[c * n + cell])) {
            channel = 4u + PLANES_A + c;
        }
    }
    if (channel != 0xffffffffu) {
        atomicMax(&rec.found, 0xffffffffu - (cell * 16u + channel));
    }
}

@compute @workgroup_size(1)
fn guard_stamp() {
    let found = atomicLoad(&rec.found);
    if (found != NONE && rec.flagged == 0u) {
        rec.flagged = 1u;
        rec.bad_step = rec.step;
        rec.bad_code = 0xffffffffu - found;
    }
}
