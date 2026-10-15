// The block summary (m0_contrat.md §1.9.1, §1.3.4, §3.1; M0-T56): after a frame's last step, over the active box, one
// record of FIELDS words per block of 4 × 4 cells, the box's blocks row-major from its origin. Compiled with eos.wgsl
// prepended (group 1, `summary_cells` only). Two dispatches, each one invocation per block and none past the box's blocks,
// so the eight storage buffers a stage may bind (§6.2.2) hold each: `summary_cells` sums the state's fields and finds the
// max-Σ cell, `summary_book` sums the frame accumulators (state.rs's booking.heat, booking.reactions,
// booking.neutrino_heating) and marks the sinks inside. Each invocation reads its 16 cells in row-major order and adds them
// in f32 in that order — no atomic, no other invocation's words: the same state gives the same bits (§1.3.4). The summary
// feeds observers only; nothing here writes the state.
//
// Per cell, Σ is its mass (dA = 1); its position the cell's centre (i + ½, j + ½), as the sinks read it; ε_th = E/Σ −
// ½|u|² − ε_cold as the state holds it (P8 floored it); unbound when ½|u|² + ε_th + φ > 0, φ 0 when the step has no φ.
// The max-Σ cell is the first of the block's largest Σ in row-major order (the lowest y, then the lowest x); its T is
// μ ε_th floored at T_floor, as eos.wgsl reads it.

struct Dims {
    width: u32,
    height: u32,
    cells: u32,
    _pad: u32,
}

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

// summary.rs: whether `phi` is the step's φ (0: a placeholder, φ read as 0).
struct SummaryParams {
    has_phi: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The hydro planes: sigma, mom_x, mom_y, energy; the species planes x_H … x_Ne, then x_Mg … x_n.
@group(0) @binding(1) var<storage, read> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
@group(0) @binding(4) var<storage, read> phi: array<f32>;
// The blocks, FIELDS words each, f32 by their bits but the max-Σ cell's index and the sinks' mask.
@group(0) @binding(5) var<storage, read_write> blocks: array<u32>;
@group(0) @binding(6) var<uniform> abox: ActiveBox;
@group(0) @binding(7) var<uniform> params: SummaryParams;
// The accumulators, one slot per cell: radiated; nuclear H, He, C, Ne, O, Si, N, then neutrino_lost; neutrino_deposited.
@group(0) @binding(8) var<storage, read> book_heat: array<f32>;
@group(0) @binding(9) var<storage, read> book_react: array<f32>;
@group(0) @binding(10) var<storage, read> book_nuheat: array<f32>;
// step/sinks.rs's state buffer, as words: the count first, the sinks from word 8, 8 words each (x, y first).
@group(0) @binding(11) var<storage, read> sinks: array<u32>;

const BLOCK: u32 = 4u;
const FIELDS: u32 = 48u;
// summary.rs `Block`'s order.
const F_MASS: u32 = 0u;
const F_MX: u32 = 1u;
const F_MY: u32 = 2u;
const F_PX: u32 = 3u;
const F_PY: u32 = 4u;
const F_KINETIC: u32 = 5u;
const F_THERMAL: u32 = 6u;
const F_COLD: u32 = 7u;
const F_PRESSURE: u32 = 8u;
const F_SPECIES: u32 = 9u;
const F_PHI_M: u32 = 19u;
const F_MAX_CELL: u32 = 20u;
const F_MAX_SIGMA: u32 = 21u;
const F_MAX_T: u32 = 22u;
const F_MAX_X: u32 = 23u;
const F_COLD_RATIO: u32 = 33u;
const F_RADIATED: u32 = 34u;
const F_NUCLEAR: u32 = 35u;
const F_NU_LOST: u32 = 42u;
const F_NU_DEPOSITED: u32 = 43u;
const F_UNBOUND: u32 = 44u;
const F_SINKS: u32 = 45u;
const NUCLEAR_GROUPS: u32 = 7u;
const SINK_WORDS: u32 = 8u;
const SINKS_FROM: u32 = 8u;
const MAX_SINKS: u32 = 16u;

// The block of invocation `gid` within the box, and its first word; none past the box's blocks.
fn block_base(gid: vec3<u32>) -> u32 {
    let bw = abox.w / BLOCK;
    let bh = abox.h / BLOCK;
    if (gid.x >= bw || gid.y >= bh) {
        return 0xffffffffu;
    }
    return (gid.y * bw + gid.x) * FIELDS;
}

// The world cell of the block's cell k (row-major, 0..16).
fn cell_of(gid: vec3<u32>, k: u32) -> u32 {
    let x = abox.x0 + gid.x * BLOCK + k % BLOCK;
    let y = abox.y0 + gid.y * BLOCK + k / BLOCK;
    return y * dims.width + x;
}

fn fractions(cell: u32) -> array<f32, 10> {
    let n = dims.cells;
    var x: array<f32, 10>;
    for (var i = 0u; i < 5u; i++) {
        x[i] = species_a[i * n + cell];
        x[5u + i] = species_b[i * n + cell];
    }
    return x;
}

fn put(base: u32, field: u32, v: f32) {
    blocks[base + field] = bitcast<u32>(v);
}

@compute @workgroup_size(2, 2, 1)
fn summary_cells(@builtin(global_invocation_id) gid: vec3<u32>) {
    let base = block_base(gid);
    if (base == 0xffffffffu) {
        return;
    }
    let n = dims.cells;
    var mass = 0.0;
    var mx = 0.0;
    var my = 0.0;
    var p = vec2<f32>(0.0);
    var kinetic = 0.0;
    var thermal = 0.0;
    var cold = 0.0;
    var pi_dA = 0.0;
    var species: array<f32, 10>;
    var phi_m = 0.0;
    var unbound = 0.0;
    var best = cell_of(gid, 0u);
    var best_sigma = -1.0;
    for (var k = 0u; k < BLOCK * BLOCK; k++) {
        let cell = cell_of(gid, k);
        let s = hydro[cell];
        let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
        let x = fractions(cell);
        let at = vec2<f32>(f32(cell % dims.width), f32(cell / dims.width)) + 0.5;
        mass += s;
        mx += at.x * s;
        my += at.y * s;
        p += mom;
        for (var i = 0u; i < N_SPECIES; i++) {
            species[i] += s * x[i];
        }
        if (s > best_sigma) {
            best_sigma = s;
            best = cell;
        }
        if (s > 0.0) {
            let comp = composition(x);
            let vel = mom / s;
            let ke = 0.5 * dot(vel, vel);
            let ec = eps_cold(s, comp.y_e, x[ID_N]);
            let eth = hydro[3u * n + cell] / s - ke - ec;
            var phi_v = 0.0;
            if (params.has_phi != 0u) {
                phi_v = phi[cell];
            }
            kinetic += 0.5 * dot(mom, mom) / s;
            thermal += s * eth;
            cold += s * ec;
            pi_dA += pressure(s, eth, comp.y_e, x[ID_N]);
            phi_m += s * phi_v;
            if (ke + eth + phi_v > 0.0) {
                unbound += s;
            }
        }
    }
    put(base, F_MASS, mass);
    put(base, F_MX, mx);
    put(base, F_MY, my);
    put(base, F_PX, p.x);
    put(base, F_PY, p.y);
    put(base, F_KINETIC, kinetic);
    put(base, F_THERMAL, thermal);
    put(base, F_COLD, cold);
    put(base, F_PRESSURE, pi_dA);
    for (var i = 0u; i < N_SPECIES; i++) {
        put(base, F_SPECIES + i, species[i]);
    }
    put(base, F_PHI_M, phi_m);
    put(base, F_UNBOUND, unbound);

    // The max-Σ cell: Σ, T, X and Π_cold / Π.
    let s = hydro[best];
    let x = fractions(best);
    var t = 0.0;
    var ratio = 0.0;
    if (s > 0.0) {
        let comp = composition(x);
        let vel = vec2<f32>(hydro[n + best], hydro[2u * n + best]) / s;
        let eth = hydro[3u * n + best] / s - 0.5 * dot(vel, vel) - eps_cold(s, comp.y_e, x[ID_N]);
        t = temperature(max(eth, eps_th_floor(comp.inv_mu)), comp.inv_mu);
        let pi_cold = pi_e(s, comp.y_e) + pi_n(s, x[ID_N]);
        let pi_all = pressure(s, eth, comp.y_e, x[ID_N]);
        if (pi_all != 0.0) {
            ratio = pi_cold / pi_all;
        }
    }
    blocks[base + F_MAX_CELL] = best;
    put(base, F_MAX_SIGMA, s);
    put(base, F_MAX_T, t);
    for (var i = 0u; i < N_SPECIES; i++) {
        put(base, F_MAX_X + i, x[i]);
    }
    put(base, F_COLD_RATIO, ratio);
}

@compute @workgroup_size(2, 2, 1)
fn summary_book(@builtin(global_invocation_id) gid: vec3<u32>) {
    let base = block_base(gid);
    if (base == 0xffffffffu) {
        return;
    }
    let n = dims.cells;
    var radiated = 0.0;
    var react: array<f32, 8>;
    var deposited = 0.0;
    for (var k = 0u; k < BLOCK * BLOCK; k++) {
        let cell = cell_of(gid, k);
        radiated += book_heat[cell];
        for (var g = 0u; g <= NUCLEAR_GROUPS; g++) {
            react[g] += book_react[g * n + cell];
        }
        deposited += book_nuheat[cell];
    }
    put(base, F_RADIATED, radiated);
    for (var g = 0u; g < NUCLEAR_GROUPS; g++) {
        put(base, F_NUCLEAR + g, react[g]);
    }
    put(base, F_NU_LOST, react[NUCLEAR_GROUPS]);
    put(base, F_NU_DEPOSITED, deposited);

    // The sinks whose position lies in the block's cells, by slot.
    let lo = vec2<f32>(f32(abox.x0 + gid.x * BLOCK), f32(abox.y0 + gid.y * BLOCK));
    let hi = lo + f32(BLOCK);
    var mask = 0u;
    for (var k = 0u; k < min(sinks[0], MAX_SINKS); k++) {
        let at = vec2<f32>(bitcast<f32>(sinks[SINKS_FROM + k * SINK_WORDS]), bitcast<f32>(sinks[SINKS_FROM + k * SINK_WORDS + 1u]));
        if (all(at >= lo) && all(at < hi)) {
            mask |= 1u << k;
        }
    }
    blocks[base + F_SINKS] = mask;
}
