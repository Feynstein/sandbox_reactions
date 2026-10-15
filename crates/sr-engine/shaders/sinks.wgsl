// P3 sinks (m0_contrat.md §1.3.2, §1.3.4, §2.6, §2.9, §0.5 [M0-V7]): M0-T50's twin (sr_physics::reference::sinks) in f32, six
// dispatches, compiled with eos.wgsl prepended (group 1); step/sinks.rs builds them.
//
// - `sink_candidates`, over the box's 8 × 8 tiles: each cell tests §2.6's four criteria — Σ ≥ Σ_BH; ∇·u < 0 by central
//   differences of the velocity (a neighbour below Σ_vac at rest); φ no greater than at its 8 neighbours, φ the plane P2
//   left, which holds the sinks' own −GM/√(r² + r_acc²) once P2 has added it (§1.4.3, M0-T52a); bound, ½|u|² + ε_th + φ < 0 — cells on the world's outer ring
//   never; the tile's best (the densest, ties the lowest row-major index, so the lowest y then x) by a fixed tree in
//   workgroup memory, into its slot of `st.cand`.
// - `sink_update`, one workgroup: the tiles' best by a strided scan and a fixed tree, then the formation (at most one, none
//   past 16); accretion sink by sink in index order — each sink's square of cells split over the invocations in a fixed
//   stride, the taken mass and momentum summed by a fixed tree, the sink's M and v frozen for its pass, a storage barrier
//   before the next sink so a cell two sinks reach goes to the first (left at Σ_floor, the next refuses it).
// - Kick-drift-kick over all sinks together (§0.5 [M0-V7]), on the gas accretion left: `sink_pull` over the box's tiles —
//   each cell with Σ ≥ Σ_vac pulls each sink by the reaction of the sink's softened pull on it, G m d/(|d|² + r_acc²)^{3/2},
//   each tile's partial per sink by a fixed tree into `pull`; `sink_kick_drift`, one workgroup — the tiles' partials summed
//   per sink by a fixed-order stride and tree, plus the other sinks' pull softened by the pair's larger r_acc, the first
//   half-kick and the drift; `sink_pull` again at the drifted positions; `sink_kick_merge` — the closing half-kick, then
//   merging, the lowest pair closer than the larger r_acc first, into the lower slot, at the mass-weighted centre. The box
//   holds every cell with Σ ≥ Σ_vac (§1.3.3), so a sink anywhere — outside the box too — feels all the gas, and no grid g
//   is read. No atomic anywhere: the same state gives the same bits (§1.3.4).
//
// A cell gives the share 1 − f, f = Σ_floor/Σ, of its Σ, momentum and E: it keeps Σ_floor exactly, f of its momentum and
// E, its fractions — so its velocity and its energy per mass. Its thermal and cold energy, the share's E − ½|mom|²/Σ, is
// booked `swallowed` at the cell (state.rs's booking.sinks; each cell is taken at most once a step, by one invocation).
// A sink's mass is a pair of f32, `m` + `m_lo` (a compensated sum): thin gas accreting onto a heavy sink is not lost below
// m's last bit.

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

struct SinkParams {
    grav: f32,
    sigma_bh: f32,
    // c_sb², for r_acc = max(2, 2GM/c_sb²)
    c2: f32,
    // the slots of `st.cand`: the world's 8 × 8 tiles
    tiles: u32,
    // Σ_vac: the cells that pull a sink (eos.wgsl's, here so the motion's dispatches need no EOS group)
    sigma_vac: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

struct Sink {
    pos: vec2<f32>,
    vel: vec2<f32>,
    m: f32,
    m_lo: f32,
    _pad: vec2<f32>,
}

// step/sinks.rs's layout: the header, the 16 sinks, then a (Σ bits, cell) pair per tile.
struct SinkState {
    count: u32,
    // the cell a sink formed at this step + 1; 0: none
    formed: u32,
    // sinks lost to merging this step
    merged: u32,
    _pad0: u32,
    // the mass accretion gave the sinks this step
    accreted: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    sinks: array<Sink, 16>,
    cand: array<vec2<u32>>,
}

@group(0) @binding(0) var<uniform> dims: Dims;
@group(0) @binding(1) var<storage, read_write> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
// booking.sinks: energy.swallowed's plane, one slot per cell
@group(0) @binding(4) var<storage, read_write> book: array<f32>;
@group(0) @binding(5) var<uniform> abox: ActiveBox;
// Δt_n, the record's first word
@group(0) @binding(6) var<uniform> dt_rec: vec4<f32>;
@group(0) @binding(7) var<uniform> sp: SinkParams;
@group(0) @binding(8) var<storage, read_write> st: SinkState;
// φ as P2 left it (side.phi): the gas's and the sinks' (`grav_sinks`)
@group(0) @binding(9) var<storage, read> phi: array<f32>;
// The motion's pull: the head, each slot's acceleration at the step's start and at its drifted position ([PULL_START + k],
// [PULL_END + k]: the gas's and the other sinks'); then each box tile's partials, MAX_SINKS a tile (`sink_pull`).
@group(0) @binding(10) var<storage, read_write> pull: array<vec2<f32>>;
// The latch record's first word, the flag (latch.wgsl `LatchState`), used by `sink_latch` alone
@group(0) @binding(11) var<storage, read_write> latch_flag: array<atomic<u32>>;

const MAX_SINKS: u32 = 16u;
const NONE: u32 = 0xffffffffu;
// The smallest r in the bound test's −GM/max(r, ½) (§2.6).
const R_MIN: f32 = 0.5;
const WG: u32 = 256u;
// `pull`'s head: the start's accelerations, the drifted ones, then the tiles' partials.
const PULL_START: u32 = 0u;
const PULL_END: u32 = MAX_SINKS;
const PULL_TILES: u32 = 2u * MAX_SINKS;

fn sink_mass(k: u32) -> f32 {
    return st.sinks[k].m + st.sinks[k].m_lo;
}

fn r_acc(m: f32) -> f32 {
    return max(2.0, 2.0 * sp.grav * m / sp.c2);
}

// The cell's fractions clamped ≥ 0 and divided by their sum, pure hydrogen when none is positive (floors.wgsl's).
fn fractions(cell: u32) -> array<f32, 10> {
    let n = dims.cells;
    var x: array<f32, 10>;
    var sum = 0.0;
    for (var i = 0u; i < 5u; i++) {
        x[i] = max(species_a[i * n + cell], 0.0);
        x[5u + i] = max(species_b[i * n + cell], 0.0);
    }
    for (var i = 0u; i < N_SPECIES; i++) {
        sum += x[i];
    }
    for (var i = 0u; i < N_SPECIES; i++) {
        if (sum > 0.0) {
            x[i] = x[i] / sum;
        } else {
            x[i] = select(0.0, 1.0, i == ID_H);
        }
    }
    return x;
}

// ε_th of a cell, floored at T_floor/μ as the twin's Eos::thermal.
fn eps_th(cell: u32) -> f32 {
    let n = dims.cells;
    let x = fractions(cell);
    let comp = composition(x);
    let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
    let raw = eps_th_raw(hydro[cell], mom, hydro[3u * n + cell], comp.y_e, x[ID_N]);
    return max(raw, eps_th_floor(comp.inv_mu));
}

// A cell's velocity as ∇·u reads it: at rest below Σ_vac.
fn velocity(cell: u32) -> vec2<f32> {
    let n = dims.cells;
    let sigma = hydro[cell];
    if (sigma < eos.sigma_vac) {
        return vec2<f32>(0.0, 0.0);
    }
    return vec2<f32>(hydro[n + cell], hydro[2u * n + cell]) / sigma;
}

// §2.6's four criteria at cell (x, y), inside the world's outer ring.
fn forms(x: u32, y: u32) -> bool {
    let w = dims.width;
    let cell = y * w + x;
    let sigma = hydro[cell];
    if (sigma < sp.sigma_bh) {
        return false;
    }
    let div = 0.5 * (velocity(cell + 1u).x - velocity(cell - 1u).x) + 0.5 * (velocity(cell + w).y - velocity(cell - w).y);
    if (div >= 0.0) {
        return false;
    }
    let p = phi[y * dims.width + x];
    for (var ny = y - 1u; ny <= y + 1u; ny++) {
        for (var nx = x - 1u; nx <= x + 1u; nx++) {
            if (p > phi[ny * dims.width + nx]) {
                return false;
            }
        }
    }
    let n = dims.cells;
    let u = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]) / sigma;
    return 0.5 * dot(u, u) + eps_th(cell) + p < 0.0;
}

// Whether (s1, c1) beats (s0, c0): denser, or as dense at a lower row-major index.
fn beats(s1: f32, c1: u32, s0: f32, c0: u32) -> bool {
    return s1 > s0 || (s1 == s0 && c1 < c0);
}

var<workgroup> best_s: array<f32, 256>;
var<workgroup> best_c: array<u32, 256>;

// The tree over the first `n` entries of best_s/best_c (a power of two): the winner at 0.
fn tree_best(li: u32, n: u32) {
    for (var k = n / 2u; k > 0u; k = k / 2u) {
        if (li < k && beats(best_s[li + k], best_c[li + k], best_s[li], best_c[li])) {
            best_s[li] = best_s[li + k];
            best_c[li] = best_c[li + k];
        }
        workgroupBarrier();
    }
}

// 1 of 6: each box tile's best candidate into st.cand[tile].
@compute @workgroup_size(8, 8)
fn sink_candidates(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    let bx = wg.x * 8u + lid.x;
    let by = wg.y * 8u + lid.y;
    let x = abox.x0 + bx;
    let y = abox.y0 + by;
    var s = -1.0;
    var c = NONE;
    if (bx < abox.w && by < abox.h && x >= 1u && y >= 1u && x + 1u < dims.width && y + 1u < dims.height) {
        if (forms(x, y)) {
            s = hydro[y * dims.width + x];
            c = y * dims.width + x;
        }
    }
    best_s[li] = s;
    best_c[li] = c;
    workgroupBarrier();
    tree_best(li, 64u);
    if (li == 0u) {
        st.cand[wg.y * ((abox.w + 7u) / 8u) + wg.x] = vec2<u32>(bitcast<u32>(best_s[0]), best_c[0]);
    }
}

// Takes the share 1 − Σ_floor/Σ of `cell`: books its thermal and cold energy `swallowed`, returns (mass, momentum).
fn take(cell: u32) -> vec3<f32> {
    let n = dims.cells;
    let sigma = hydro[cell];
    let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
    let energy = hydro[3u * n + cell];
    let f = eos.sigma_floor / sigma;
    book[cell] += (1.0 - f) * (energy - 0.5 * dot(mom, mom) / sigma);
    hydro[cell] = eos.sigma_floor;
    hydro[n + cell] = mom.x * f;
    hydro[2u * n + cell] = mom.y * f;
    hydro[3u * n + cell] = energy * f;
    return vec3<f32>(sigma - eos.sigma_floor, mom - mom * f);
}

// Adds `dm` to sink k's mass pair (Fast2Sum on the larger first), then renormalises the pair so m = fl(m + m_lo): its f64
// sum splits back into the same two f32 (step/sinks.rs's `set_sinks`, a dump's restore).
fn add_mass(k: u32, dm: f32) {
    let a = st.sinks[k].m;
    let s = a + dm;
    var e: f32;
    if (abs(a) >= abs(dm)) {
        e = (a - s) + dm;
    } else {
        e = (dm - s) + a;
    }
    let lo = st.sinks[k].m_lo + e;
    let hi = s + lo;
    st.sinks[k].m = hi;
    st.sinks[k].m_lo = lo - (hi - s);
}

var<workgroup> part: array<vec3<f32>, 256>;

// 2 of 6: formation and accretion, one workgroup.
@compute @workgroup_size(256)
fn sink_update(@builtin(local_invocation_index) li: u32) {
    let n = dims.cells;
    let w = dims.width;
    let h = dims.height;

    // The box's tiles' best: each invocation its stride, in order, then the tree.
    let tiles = ((abox.w + 7u) / 8u) * ((abox.h + 7u) / 8u);
    var s = -1.0;
    var c = NONE;
    for (var t = li; t < tiles; t += WG) {
        let v = st.cand[t];
        if (beats(bitcast<f32>(v.x), v.y, s, c)) {
            s = bitcast<f32>(v.x);
            c = v.y;
        }
    }
    best_s[li] = s;
    best_c[li] = c;
    workgroupBarrier();
    tree_best(li, WG);
    if (li == 0u) {
        st.formed = 0u;
        st.merged = 0u;
        st.accreted = 0.0;
        let k = st.count;
        if (best_c[0] != NONE && k < MAX_SINKS) {
            let cell = best_c[0];
            let got = take(cell);
            st.sinks[k] = Sink(vec2<f32>(f32(cell % w) + 0.5, f32(cell / w) + 0.5), got.yz / got.x, got.x, 0.0, vec2<f32>(0.0, 0.0));
            st.count = k + 1u;
            st.formed = cell + 1u;
        }
    }
    storageBarrier();
    workgroupBarrier();

    // Accretion, sink by sink in index order; the loop runs all 16 slots so every barrier is in uniform control flow.
    for (var k = 0u; k < MAX_SINKS; k++) {
        var got = vec3<f32>(0.0, 0.0, 0.0);
        if (k < st.count) {
            let pos = st.sinks[k].pos;
            let vel = st.sinks[k].vel;
            let m = sink_mass(k);
            let ra = r_acc(m);
            let x0 = u32(max(floor(pos.x - ra), 0.0));
            let x1 = min(u32(max(ceil(pos.x + ra), 0.0)), w - 1u);
            let y0 = u32(max(floor(pos.y - ra), 0.0));
            let y1 = min(u32(max(ceil(pos.y + ra), 0.0)), h - 1u);
            if (x0 <= x1 && y0 <= y1) {
                let side = x1 - x0 + 1u;
                let cells = side * (y1 - y0 + 1u);
                for (var i = li; i < cells; i += WG) {
                    let x = x0 + i % side;
                    let y = y0 + i / side;
                    let cell = y * w + x;
                    let d = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5) - pos;
                    let r = length(d);
                    let sigma = hydro[cell];
                    if (r > ra || sigma < eos.sigma_vac || sigma <= eos.sigma_floor) {
                        continue;
                    }
                    let rel = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]) / sigma - vel;
                    if (0.5 * dot(rel, rel) + eps_th(cell) - sp.grav * m / max(r, R_MIN) >= 0.0) {
                        continue;
                    }
                    got += take(cell);
                }
            }
        }
        part[li] = got;
        workgroupBarrier();
        for (var q = WG / 2u; q > 0u; q = q / 2u) {
            if (li < q) {
                part[li] += part[li + q];
            }
            workgroupBarrier();
        }
        if (li == 0u && k < st.count) {
            let dm = part[0].x;
            let m = sink_mass(k);
            let p = m * st.sinks[k].vel + part[0].yz;
            add_mass(k, dm);
            st.sinks[k].vel = p / sink_mass(k);
            st.accreted += dm;
        }
        storageBarrier();
        workgroupBarrier();
    }
}

var<workgroup> tile_pull: array<vec2<f32>, 64>;

// 3 and 5 of 6: over the box's 8 × 8 tiles, each tile's pull on each sink at its position now — the reaction of the sink's
// softened pull on every cell with Σ ≥ Σ_vac (§0.5 [M0-V7], §1.4.3), G m d/(|d|² + r_acc²)^{3/2}, d from the sink to the
// cell's centre, m the cell's Σ — summed over the tile by a fixed tree into pull[PULL_TILES + tile × MAX_SINKS + k]. The
// slots run to MAX_SINKS so every barrier is in uniform control flow.
@compute @workgroup_size(8, 8)
fn sink_pull(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    let bx = wg.x * 8u + lid.x;
    let by = wg.y * 8u + lid.y;
    let x = abox.x0 + bx;
    let y = abox.y0 + by;
    var m = 0.0;
    if (bx < abox.w && by < abox.h) {
        let sigma = hydro[y * dims.width + x];
        if (sigma >= sp.sigma_vac) {
            m = sigma;
        }
    }
    let centre = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
    let tile = wg.y * ((abox.w + 7u) / 8u) + wg.x;
    for (var k = 0u; k < MAX_SINKS; k++) {
        var a = vec2<f32>(0.0, 0.0);
        if (m > 0.0 && k < st.count) {
            let d = centre - st.sinks[k].pos;
            let ra = r_acc(sink_mass(k));
            let soft = dot(d, d) + ra * ra;
            a = sp.grav * m / (soft * sqrt(soft)) * d;
        }
        tile_pull[li] = a;
        workgroupBarrier();
        for (var q = 32u; q > 0u; q = q / 2u) {
            if (li < q) {
                tile_pull[li] += tile_pull[li + q];
            }
            workgroupBarrier();
        }
        if (li == 0u) {
            pull[PULL_TILES + tile * MAX_SINKS + k] = tile_pull[0];
        }
        workgroupBarrier();
    }
}

var<workgroup> wide: array<vec2<f32>, 256>;
var<workgroup> at: array<vec2<f32>, 16>;
var<workgroup> acc: array<vec2<f32>, 16>;

// The gas's pull on each sink into `acc`: the box's tiles' partials, each invocation its stride in order, then a fixed tree.
fn gas_pull(li: u32) {
    let tiles = ((abox.w + 7u) / 8u) * ((abox.h + 7u) / 8u);
    for (var k = 0u; k < MAX_SINKS; k++) {
        var a = vec2<f32>(0.0, 0.0);
        for (var t = li; t < tiles; t += WG) {
            a += pull[PULL_TILES + t * MAX_SINKS + k];
        }
        wide[li] = a;
        workgroupBarrier();
        for (var q = WG / 2u; q > 0u; q = q / 2u) {
            if (li < q) {
                wide[li] += wide[li + q];
            }
            workgroupBarrier();
        }
        if (li == 0u) {
            acc[k] = wide[0];
        }
        workgroupBarrier();
    }
}

// The other sinks' pull on sink i at the positions `at`, each pair softened by its larger r_acc (§0.5 [M0-V7]).
fn pair_pull(i: u32, count: u32) -> vec2<f32> {
    var a = vec2<f32>(0.0, 0.0);
    for (var j = 0u; j < count; j++) {
        if (j != i) {
            let d = at[j] - at[i];
            let ra = max(r_acc(sink_mass(i)), r_acc(sink_mass(j)));
            let soft = dot(d, d) + ra * ra;
            a += sp.grav * sink_mass(j) / (soft * sqrt(soft)) * d;
        }
    }
    return a;
}

// Each sink's acceleration at the positions `at` (the gas's, from `gas_pull`, and the other sinks'), kept at pull[slot + i].
fn kick_pull(li: u32, count: u32, slot: u32) -> vec2<f32> {
    let a = acc[li] + pair_pull(li, count);
    pull[slot + li] = a;
    return a;
}

// 4 of 6: the first half-kick and the drift, one workgroup.
@compute @workgroup_size(256)
fn sink_kick_drift(@builtin(local_invocation_index) li: u32) {
    gas_pull(li);
    let dt = dt_rec.x;
    let count = min(st.count, MAX_SINKS);
    if (li < count) {
        at[li] = st.sinks[li].pos;
    }
    workgroupBarrier();
    if (li < count) {
        let v = st.sinks[li].vel + 0.5 * dt * kick_pull(li, count, PULL_START);
        st.sinks[li].vel = v;
        st.sinks[li].pos = at[li] + dt * v;
    }
}

// 6 of 6: the closing half-kick at the drifted positions, then merging, one workgroup.
@compute @workgroup_size(256)
fn sink_kick_merge(@builtin(local_invocation_index) li: u32) {
    gas_pull(li);
    let dt = dt_rec.x;
    let count = min(st.count, MAX_SINKS);
    if (li < count) {
        at[li] = st.sinks[li].pos;
    }
    workgroupBarrier();
    if (li < count) {
        st.sinks[li].vel += 0.5 * dt * kick_pull(li, count, PULL_END);
    }
    storageBarrier();

    // Merging: the lowest pair closer than the larger r_acc, again until none is.
    if (li == 0u) {
        var live = count;
        var again = true;
        while (again) {
            again = false;
            for (var i = 0u; i < live && !again; i++) {
                for (var j = i + 1u; j < live && !again; j++) {
                    let a = st.sinks[i];
                    let b = st.sinks[j];
                    let ma = sink_mass(i);
                    let mb = sink_mass(j);
                    if (distance(a.pos, b.pos) < max(r_acc(ma), r_acc(mb))) {
                        let m = ma + mb;
                        st.sinks[i].pos = (ma * a.pos + mb * b.pos) / m;
                        st.sinks[i].vel = (ma * a.vel + mb * b.vel) / m;
                        add_mass(i, b.m);
                        add_mass(i, b.m_lo);
                        for (var q = j; q + 1u < live; q++) {
                            st.sinks[q] = st.sinks[q + 1u];
                        }
                        live -= 1u;
                        st.merged += 1u;
                        again = true;
                    }
                }
            }
        }
        st.count = live;
    }
}

// The collapse latch (§1.8.5): after P3's six dispatches, one invocation ORs 1 into word 0 of the latch record when a sink
// formed this step — built apart (step/sinks.rs `latch_dispatch`) because the latch is made from the finished pass lists.
@compute @workgroup_size(1)
fn sink_latch() {
    if (st.formed != 0u) {
        atomicOr(&latch_flag[0], 1u);
    }
}
