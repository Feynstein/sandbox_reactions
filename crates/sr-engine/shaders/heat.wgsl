// P5 heat (m0_contrat.md §1.3.2, §2.4, §2.9's radiated, §1.9.1; M0-T42): M0-T41's twin (sr_physics::reference::heat) in
// f32 — one-temperature flux-limited diffusion written as face conductances, stepped by RKL2 with every coefficient frozen
// at the pass's start. Compiled with eos.wgsl prepended (group 1).
//
// A face between cells a and b carries F = −K (T_b − T_a), positive toward +x or +y, with
// K = c_sb λ(R) a_r (T_a + T_b)(T_a² + T_b²)/(κΣ)_f + k_cond, κΣ the mean of the two cells' and R = 2|T_b⁴ − T_a⁴|/((κΣ)_f (T_a⁴ + T_b⁴))
// (a_r cancels; the fourth powers are taken relative to the hotter side, so a hot cell cannot overflow them). A vacuum cell
// (Σ < Σ_vac), the world's edge, and any cell outside the active box (§1.3.3: vacuum) are the floor state beyond the gas
// side — Σ_floor, T_floor, the gas side's κ and 1/μ — and are never updated; between two such, K = 0.
//
// The dispatches, in order (step/heat.rs): `heat_begin` clears the rate words; `heat_cells` freezes each box cell's T₀ (ε_th
// floored at T_floor/μ, as the twin's Eos::thermal), C = Σ/μ (0 marks a vacuum cell), κ, 1/μ and κΣ; `heat_stiff` takes each
// gas cell's own Σ_f K_f / C (its rate before any cap) into a plane and the raw word; `heat_faces` writes K and F of each box
// cell's −x and −y faces, and of the +x, +y faces of the box's last column and row, capped where the cell is stiff (below);
// `heat_rate` takes each gas cell's Σ_f K_f / C into the rate word — atomicMax on the bits of a non-negative f32, exact and
// order-free (§1.3.4) — with its M(Y₀) = dT/dt and its rate of loss through vacuum faces, and writes the cell-centred F the
// next step's P4 reads; `heat_plan` turns the rate and Δt_n into Δt_p = 1/rate and s, the smallest s ≥ 2 with
// Δt_p (s² + s − 2)/4 ≥ Δt (§2.4.3); `heat_stage` j = 1 … 32, one dispatch each, is RKL2's stage j where j ≤ s (Meyer et al.
// 2012: w₁ = 4/(s² + s − 2)); the booked loss R_j rides the same recursion; `heat_commit` adds C (Y_s − Y₀) to E and R_s to
// the cell's `radiated` slot.
//
// Thirty-two stage dispatches are recorded. A cell whose own rate would need more than 32 stages over Δt_n (thin hot gas —
// sun_disk's rim needs thousands) is stiff: each face it touches is capped at K ≤ 0.999 (32² + 32 − 2)/(4Δt) · C/4, C the
// smaller gas side's, so every cell fits 32 stages over the full Δt and only the stiff cells' heat lags (§2.4.3 [M0-T42], the
// lead's ruling). A step with no stiff cell is the twin's scheme untouched; the raw word keeps the s Δt needed, for the
// contract's `SR-WARN heat stages <s>`. Should rounding still ask 33, the pass runs 32 over the time they cover.

struct Dims {
    width: u32,
    height: u32,
    cells: u32,
    _pad: u32,
}

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

// physics.json's constants and the scene's test flags (§2.12.1).
struct HeatParams {
    a_r: f32,
    c_sb: f32,
    kappa0: f32,
    k_cond: f32,
    kappa_dust: f32,
    t_dust: f32,
    chi: f32,
    flags: u32,
    // eos.wgsl's floors, here so only `heat_cells` binds the EOS.
    t_floor: f32,
    sigma_floor: f32,
    _a: u32,
    _b: u32,
}

// The pass's record (step/heat.rs `HeatReport`): the rate words (capped, raw), the stages run, the stages Δt needed
// before any cap, the most ever needed, the time the stages cover and Δt_p (0 when nothing flows).
struct HeatRecord {
    rate: atomic<u32>,
    raw: atomic<u32>,
    s: u32,
    need: u32,
    worst: u32,
    dt: f32,
    dt_p: f32,
    _a: u32,
}

struct StageIndex {
    j: u32,
    _a: u32,
    _b: u32,
    _c: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The hydro planes: sigma, mom_x, mom_y, energy (E written by `heat_commit` alone).
@group(0) @binding(1) var<storage, read_write> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
// The frozen planes, T0 … OUT0.
@group(0) @binding(4) var<storage, read_write> frozen: array<f32>;
// K on the x faces ((width + 1) × height, face (x, y) between cells x − 1 and x), K on the y faces (width × (height + 1)),
// then F on each, in the same layout.
@group(0) @binding(5) var<storage, read_write> faces: array<f32>;
// Y's three slots, then R's: stage j writes slot j % 3.
@group(0) @binding(6) var<storage, read_write> yr: array<f32>;
@group(0) @binding(7) var<storage, read_write> rec: HeatRecord;
@group(0) @binding(8) var<uniform> abox: ActiveBox;
// The Δt record (reduce.wgsl `DtState`): Δt_n is its first word.
@group(0) @binding(9) var<storage, read> dt_rec: array<f32>;
@group(0) @binding(10) var<uniform> params: HeatParams;
// side.flux (state.rs SideFields): F_x, then F_y, cell-centred.
@group(0) @binding(11) var<storage, read_write> flux: array<f32>;
// booking.heat: `radiated`, one slot per cell.
@group(0) @binding(12) var<storage, read_write> book: array<f32>;
@group(0) @binding(13) var<uniform> stage: StageIndex;

const MAX_STAGES: u32 = 32u;
const CONSTANT_DIFFUSIVITY: u32 = 1u;
const FLUX_LIMITER: u32 = 2u;
const PIN_TEMPERATURE: u32 = 4u;
const T0: u32 = 0u;
const CAP: u32 = 1u;
const KAPPA: u32 = 2u;
const INV_MU: u32 = 3u;
const KS: u32 = 4u;
const M0: u32 = 5u;
const OUT0: u32 = 6u;
// `heat_stiff`'s rate lives in the M(Y₀) plane until `heat_rate` writes M(Y₀) there.
const RAW: u32 = 5u;
// The capped faces' margin under 32 stages.
const THROTTLE: f32 = 0.999;
// `flow`'s temperatures: the frozen T₀ instead of a Y slot.
const FROM_T0: u32 = 3u;

struct Side {
    gas: bool,
    t: f32,
    c: f32,
    kappa: f32,
    inv_mu: f32,
    ks: f32,
}

fn finite(v: f32) -> bool {
    return (bitcast<u32>(v) & 0x7f800000u) != 0x7f800000u;
}

fn nx() -> u32 {
    return (dims.width + 1u) * dims.height;
}

fn ny() -> u32 {
    return dims.width * (dims.height + 1u);
}

// Whether (x, y) is a cell of the box; a coordinate below the box wraps past it.
fn in_box(x: u32, y: u32) -> bool {
    return x - abox.x0 < abox.w && y - abox.y0 < abox.h && x < dims.width && y < dims.height;
}

// The far side of a face with no cell of the box there.
fn no_side() -> Side {
    return Side(false, params.t_floor, 0.0, 0.0, 0.0, 0.0);
}

fn side_at(x: u32, y: u32) -> Side {
    if (!in_box(x, y)) {
        return no_side();
    }
    let n = dims.cells;
    let cell = y * dims.width + x;
    let c = frozen[CAP * n + cell];
    return Side(c > 0.0, frozen[T0 * n + cell], c, frozen[KAPPA * n + cell], frozen[INV_MU * n + cell], frozen[KS * n + cell]);
}

// The vacuum beyond gas side `g`: Σ_floor at T_floor with g's κ and 1/μ.
fn vacuum_beyond(g: Side) -> Side {
    return Side(false, params.t_floor, params.sigma_floor * g.inv_mu, g.kappa, g.inv_mu, g.kappa * params.sigma_floor);
}

// Levermore–Pomraning's λ(R), or 1/3 with the scene's flux_limiter off.
fn limiter(r: f32) -> f32 {
    if ((params.flags & FLUX_LIMITER) == 0u) {
        return 1.0 / 3.0;
    }
    return (2.0 + r) / (6.0 + 3.0 * r + r * r);
}

// (K, F) of the face from `a_in` to `b_in`.
fn face(a_in: Side, b_in: Side) -> vec2<f32> {
    if (!a_in.gas && !b_in.gas) {
        return vec2<f32>(0.0);
    }
    var a = a_in;
    var b = b_in;
    if (!a.gas) {
        a = vacuum_beyond(b);
    }
    if (!b.gas) {
        b = vacuum_beyond(a);
    }
    var k: f32;
    if ((params.flags & CONSTANT_DIFFUSIVITY) != 0u) {
        k = params.chi * (0.5 * (a.c + b.c));
    } else {
        let tm = max(a.t, b.t);
        let ra = a.t / tm;
        let rb = b.t / tm;
        let qa = ra * ra * ra * ra;
        let qb = rb * rb * rb * rb;
        let ks = 0.5 * (a.ks + b.ks);
        let r = 2.0 * abs(qb - qa) / (ks * (qa + qb));
        k = params.c_sb * limiter(r) * params.a_r * (a.t + b.t) * (a.t * a.t + b.t * b.t) / ks + params.k_cond;
    }
    return vec2<f32>(k, -k * (b.t - a.t));
}

// A cell's rate before any cap; 0 outside the box or for vacuum.
fn raw_at(x: u32, y: u32) -> f32 {
    if (!in_box(x, y)) {
        return 0.0;
    }
    return frozen[RAW * dims.cells + y * dims.width + x];
}

// (K, F) of the face from cell (xa, ya) to (xb, yb), capped where either side is stiff (the header).
fn throttled(xa: u32, ya: u32, xb: u32, yb: u32) -> vec2<f32> {
    let a = side_at(xa, ya);
    let b = side_at(xb, yb);
    let kf = face(a, b);
    let dt = dt_rec[0];
    if (kf.x <= 0.0 || !(dt > 0.0) || !finite(dt)) {
        return kf;
    }
    let cap = covered(MAX_STAGES) / (4.0 * dt);
    if (raw_at(xa, ya) <= cap && raw_at(xb, yb) <= cap) {
        return kf;
    }
    var c = 3.0e38;
    if (a.gas) {
        c = min(c, a.c);
    }
    if (b.gas) {
        c = min(c, b.c);
    }
    let k_max = THROTTLE * cap * c / 4.0;
    if (kf.x <= k_max) {
        return kf;
    }
    return kf * (k_max / kf.x);
}

fn put_x(i: u32, kf: vec2<f32>) {
    faces[i] = kf.x;
    faces[nx() + ny() + i] = kf.y;
}

fn put_y(i: u32, kf: vec2<f32>) {
    faces[nx() + i] = kf.x;
    faces[2u * nx() + ny() + i] = kf.y;
}

// T of `cell` at `slot` (a Y slot, or FROM_T0).
fn temp_at(cell: u32, slot: u32) -> f32 {
    if (slot == FROM_T0) {
        return frozen[T0 * dims.cells + cell];
    }
    return yr[slot * dims.cells + cell];
}

// For gas cell (x, y) at the temperatures of `slot`: dT/dt, and the rate it loses heat through faces whose far side is
// not gas, Σ K (T − T_floor).
fn flow(x: u32, y: u32, slot: u32) -> vec2<f32> {
    let w = dims.width;
    let n = dims.cells;
    let cell = y * w + x;
    let ti = temp_at(cell, slot);
    var k = array<f32, 4>(faces[y * (w + 1u) + x], faces[y * (w + 1u) + x + 1u], faces[nx() + y * w + x], faces[nx() + (y + 1u) * w + x]);
    var nb = array<vec2<u32>, 4>(vec2<u32>(x - 1u, y), vec2<u32>(x + 1u, y), vec2<u32>(x, y - 1u), vec2<u32>(x, y + 1u));
    var dtdt = 0.0;
    var out = 0.0;
    for (var f = 0u; f < 4u; f++) {
        let o = nb[f];
        let j = o.y * w + o.x;
        if (in_box(o.x, o.y) && frozen[CAP * n + j] > 0.0) {
            dtdt += k[f] * (temp_at(j, slot) - ti);
        } else {
            dtdt += k[f] * (params.t_floor - ti);
            out += k[f] * (ti - params.t_floor);
        }
    }
    return vec2<f32>(dtdt / frozen[CAP * n + cell], out);
}

// RKL2's b_j: 1/3 for j ≤ 2, (j² + j − 2)/(2j(j + 1)) above.
fn b(j: u32) -> f32 {
    if (j <= 2u) {
        return 1.0 / 3.0;
    }
    let f = f32(j);
    return (f * f + f - 2.0) / (2.0 * f * (f + 1.0));
}

// s² + s − 2: the step s stages cover, in units of Δt_p/4.
fn covered(s: u32) -> f32 {
    return f32(s * s + s - 2u);
}

// The smallest s ≥ 2 with s² + s − 2 ≥ q = 4Δt/Δt_p; past q = 4 × 10⁶ (s ≈ 2000, only ever read by the warning) the closed
// form's ceiling alone, held under u32's range.
fn stages_for(q: f32) -> u32 {
    if (q > 4.0e6) {
        return u32(min(ceil((sqrt(9.0 + 4.0 * q) - 1.0) / 2.0), 4.0e9));
    }
    var s = max(2u, u32(floor((sqrt(9.0 + 4.0 * q) - 1.0) / 2.0)));
    while (s > 2u && covered(s - 1u) >= q) {
        s -= 1u;
    }
    while (covered(s) < q) {
        s += 1u;
    }
    return s;
}

@compute @workgroup_size(1)
fn heat_begin() {
    atomicStore(&rec.rate, 0u);
    atomicStore(&rec.raw, 0u);
}

@compute @workgroup_size(8, 8)
fn heat_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let n = dims.cells;
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
    var x: array<f32, 10>;
    for (var i = 0u; i < 5u; i++) {
        x[i] = species_a[i * n + cell];
        x[5u + i] = species_b[i * n + cell];
    }
    let comp = composition(x);
    let sigma = hydro[cell];
    var t = eos.t_floor;
    var c = 0.0;
    if (sigma >= eos.sigma_vac) {
        let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
        let eps = max(eps_th_raw(sigma, mom, hydro[3u * n + cell], comp.y_e, x[ID_N]), eps_th_floor(comp.inv_mu));
        t = temperature(eps, comp.inv_mu);
        c = sigma * comp.inv_mu;
    }
    var kappa = params.kappa0 * (1.0 + x[ID_H]);
    if (t < params.t_dust) {
        kappa += params.kappa_dust * comp.z_met;
    }
    frozen[T0 * n + cell] = t;
    frozen[CAP * n + cell] = c;
    frozen[KAPPA * n + cell] = kappa;
    frozen[INV_MU * n + cell] = comp.inv_mu;
    frozen[KS * n + cell] = kappa * sigma;
}

@compute @workgroup_size(8, 8)
fn heat_stiff(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let me = side_at(x, y);
    var raw = 0.0;
    if (me.gas) {
        let k = face(side_at(x - 1u, y), me).x + face(me, side_at(x + 1u, y)).x + face(side_at(x, y - 1u), me).x
            + face(me, side_at(x, y + 1u)).x;
        raw = max(k / me.c, 0.0);
        atomicMax(&rec.raw, bitcast<u32>(raw));
    }
    frozen[RAW * dims.cells + y * dims.width + x] = raw;
}

@compute @workgroup_size(8, 8)
fn heat_faces(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let w = dims.width;
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    put_x(y * (w + 1u) + x, throttled(x - 1u, y, x, y));
    put_y(y * w + x, throttled(x, y - 1u, x, y));
    if (id.x + 1u == abox.w) {
        put_x(y * (w + 1u) + x + 1u, throttled(x, y, x + 1u, y));
    }
    if (id.y + 1u == abox.h) {
        put_y((y + 1u) * w + x, throttled(x, y, x, y + 1u));
    }
}

@compute @workgroup_size(8, 8)
fn heat_rate(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let w = dims.width;
    let n = dims.cells;
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let cell = y * w + x;
    let fx = nx() + ny();
    let fy = 2u * nx() + ny();
    flux[cell] = 0.5 * (faces[fx + y * (w + 1u) + x] + faces[fx + y * (w + 1u) + x + 1u]);
    flux[n + cell] = 0.5 * (faces[fy + y * w + x] + faces[fy + (y + 1u) * w + x]);
    let c = frozen[CAP * n + cell];
    if (c <= 0.0) {
        return;
    }
    let k = faces[y * (w + 1u) + x] + faces[y * (w + 1u) + x + 1u] + faces[nx() + y * w + x] + faces[nx() + (y + 1u) * w + x];
    atomicMax(&rec.rate, bitcast<u32>(max(k / c, 0.0)));
    let m = flow(x, y, FROM_T0);
    frozen[M0 * n + cell] = m.x;
    frozen[OUT0 * n + cell] = m.y;
}

@compute @workgroup_size(1)
fn heat_plan() {
    let bits = atomicLoad(&rec.rate);
    let rate = bitcast<f32>(bits);
    let dt = dt_rec[0];
    rec.s = 0u;
    rec.need = 0u;
    rec.dt = 0.0;
    rec.dt_p = 0.0;
    if (bits == 0u || !finite(rate)) {
        return;
    }
    rec.dt_p = 1.0 / rate;
    if (!(dt > 0.0) || !finite(dt) || (params.flags & PIN_TEMPERATURE) != 0u) {
        return;
    }
    let need = stages_for(4.0 * dt * max(bitcast<f32>(atomicLoad(&rec.raw)), rate));
    let capped = stages_for(4.0 * dt * rate);
    let s = min(capped, MAX_STAGES);
    rec.need = need;
    rec.worst = max(rec.worst, need);
    rec.s = s;
    rec.dt = select(dt, rec.dt_p * covered(s) / 4.0, capped > MAX_STAGES);
}

@compute @workgroup_size(8, 8)
fn heat_stage(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = stage.j;
    if (j > rec.s || id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let n = dims.cells;
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let cell = y * dims.width + x;
    if (frozen[CAP * n + cell] <= 0.0) {
        return;
    }
    let s = rec.s;
    let dt = rec.dt;
    let w1 = 4.0 / covered(s);
    let y0 = frozen[T0 * n + cell];
    let m0 = frozen[M0 * n + cell];
    let o0 = frozen[OUT0 * n + cell];
    if (j == 1u) {
        // Y₁ = Y₀ + μ̃₁ Δt M(Y₀), μ̃₁ = b₁ w₁; slot 0 holds Y₀ (and R₀ = 0) for stage 2.
        let mt1 = b(1u) * w1;
        yr[cell] = y0;
        yr[3u * n + cell] = 0.0;
        yr[n + cell] = y0 + mt1 * dt * m0;
        yr[4u * n + cell] = mt1 * dt * o0;
        return;
    }
    let jf = f32(j);
    let mu = (2.0 * jf - 1.0) / jf * b(j) / b(j - 1u);
    let nu = -(jf - 1.0) / jf * b(j) / b(j - 2u);
    let mt = mu * w1;
    let gt = -(1.0 - b(j - 1u)) * mt;
    let p1 = (j - 1u) % 3u;
    let p2 = (j - 2u) % 3u;
    let me = j % 3u;
    let m = flow(x, y, p1);
    yr[me * n + cell] = mu * yr[p1 * n + cell] + nu * yr[p2 * n + cell] + (1.0 - mu - nu) * y0 + mt * dt * m.x + gt * dt * m0;
    yr[(3u + me) * n + cell] = mu * yr[(3u + p1) * n + cell] + nu * yr[(3u + p2) * n + cell] + mt * dt * m.y + gt * dt * o0;
}

@compute @workgroup_size(8, 8)
fn heat_commit(@builtin(global_invocation_id) id: vec3<u32>) {
    let s = rec.s;
    if (s == 0u || id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let n = dims.cells;
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
    let c = frozen[CAP * n + cell];
    if (c <= 0.0) {
        return;
    }
    let slot = s % 3u;
    hydro[3u * n + cell] += c * (yr[slot * n + cell] - frozen[T0 * n + cell]);
    book[cell] += yr[(3u + slot) * n + cell];
}
