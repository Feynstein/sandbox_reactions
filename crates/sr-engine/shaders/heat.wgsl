// P5 heat (m0_contrat.md §1.3.2, §2.4, §0.5 [M0-TJ4], §2.9's radiated, §1.9.1; M0-T42, M0-T118, M0-T119): the twin's
// implicit step (sr_physics::reference::heat `Heat::step`) in f32 — one-temperature flux-limited diffusion as one
// backward-Euler step in the heat potential u = a_r T⁴ (u = T under the scene's test.constant_diffusivity), the light law
// frozen at the pass's start and the heat solved by Newton passes. RKL2 and its 32-stage face cap are retired (§0.5
// [M0-TJ4]). Compiled with eos.wgsl prepended (group 1).
//
// A face between cells a and b carries F = −D_f (u_b − u_a), positive toward +x or +y, with
// D_f = c_sb λ(R)/(κΣ)_f + k_cond/(a_r (T_a + T_b)(T_a² + T_b²)), κΣ the mean of the two cells' and
// R = 2|T_b⁴ − T_a⁴|/((κΣ)_f (T_a⁴ + T_b⁴)) (a_r cancels; the fourth powers are taken relative to the hotter side, so a hot
// cell cannot overflow them) — or D_f = χ C_f, the two heat capacities' mean, under constant_diffusivity. A vacuum cell
// (Σ < Σ_vac), the world's edge, and any cell outside the active box (§1.3.3: vacuum) are the floor state beyond the gas
// side — Σ_floor, T_floor, the gas side's κ and 1/μ, u_vac = a_r T_floor⁴ — and are never updated; between two such, D_f = 0.
//
// `heat_cells` freezes each box cell's T⁰ (ε_th floored at T_floor/μ, as the twin's Eos::thermal), C = Σ/μ (0 marks a vacuum
// cell), κ, 1/μ and κΣ; the section "The implicit step" below does the rest.

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
    // The implicit step's partial-sum slots per parity: the world's 16 × 16 tiles.
    tiles: u32,
    _b: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The hydro planes: sigma, mom_x, mom_y, energy (E written by `heat_apply` alone).
@group(0) @binding(1) var<storage, read_write> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
// The frozen planes, T0 … KS.
@group(0) @binding(4) var<storage, read_write> frozen: array<f32>;
// D_f on the x faces ((width + 1) × height, face (x, y) between cells x − 1 and x), D_f on the y faces (width × (height + 1)),
// then F on each, in the same layout.
@group(0) @binding(5) var<storage, read_write> faces: array<f32>;
@group(0) @binding(8) var<uniform> abox: ActiveBox;
// The Δt record (reduce.wgsl `DtState`): Δt_n is its first word.
@group(0) @binding(9) var<storage, read> dt_rec: array<f32>;
@group(0) @binding(10) var<uniform> params: HeatParams;
// side.flux (state.rs SideFields): F_x, then F_y, cell-centred.
@group(0) @binding(11) var<storage, read_write> flux: array<f32>;
// booking.heat: `radiated`, one slot per cell.
@group(0) @binding(12) var<storage, read_write> book: array<f32>;

const CONSTANT_DIFFUSIVITY: u32 = 1u;
const FLUX_LIMITER: u32 = 2u;
const PIN_TEMPERATURE: u32 = 4u;
const T0: u32 = 0u;
const CAP: u32 = 1u;
const KAPPA: u32 = 2u;
const INV_MU: u32 = 3u;
const KS: u32 = 4u;

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

// (D_f, F) of the face from `a_in` to `b_in`, the law on the two sides' start state.
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
    var d: f32;
    if ((params.flags & CONSTANT_DIFFUSIVITY) != 0u) {
        d = params.chi * (0.5 * (a.c + b.c));
    } else {
        let tm = max(a.t, b.t);
        let ra = a.t / tm;
        let rb = b.t / tm;
        let qa = ra * ra * ra * ra;
        let qb = rb * rb * rb * rb;
        let ks = 0.5 * (a.ks + b.ks);
        let r = 2.0 * abs(qb - qa) / (ks * (qa + qb));
        d = params.c_sb * limiter(r) / ks;
        if (params.k_cond != 0.0) {
            d += params.k_cond / (params.a_r * (a.t + b.t) * (a.t * a.t + b.t * b.t));
        }
    }
    return vec2<f32>(d, -d * (u_of(b.t) - u_of(a.t)));
}

fn put_x(i: u32, kf: vec2<f32>) {
    faces[i] = kf.x;
    faces[nx() + ny() + i] = kf.y;
}

fn put_y(i: u32, kf: vec2<f32>) {
    faces[nx() + i] = kf.x;
    faces[2u * nx() + ny() + i] = kf.y;
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

// ---- The implicit step (§0.5 [M0-TJ4] (2), (4), (5), (6); M0-T118, M0-T119) — every scene ----
//
// One backward-Euler step over Δt_n, the law frozen at the pass's start: every gas cell solves C_i (T(u_i) − T_i⁰)
// + Δt Σ_f F_f(u) = 0, F_f = −D_f (u_b − u_a), D_f the face's of `heat_law` (the header), T(u) = (u/a_r)^{1/4} (u = T under
// constant_diffusivity: the system is linear). Newton passes from the start state linearise T(u) at the latest iterate, so
// each pass is one symmetric positive-definite system — diagonal c_i + Σ_f D_f with c_i = (C_i/Δt) dT/du,
// dT/du = 1/(4 a_r T³) at the iterate, −D_f between gas cells, right side c_i u_i − (C_i/Δt)(T_i − T_i⁰) + Σ_vac D_f u_vac —
// solved by
// Jacobi-preconditioned conjugate gradients in f32 until both of (4)'s measures hold at 10⁻⁵ (checked before every round,
// the start included), each pass's solve started from the iterate shifted along z, the indicator of the pass's heat-free
// cells (`free_at`), by the Galerkin step zᵀr/zᵀA z (M0-D36: a thin ring's common level, which the rounds barely move and
// the measures do not see, set once a pass — its emission 1 % high and the closure δ −1.4 × 10⁻⁴ on heat_rim without it);
// the update keeps u ≥ u_vac and moves no T by more than a factor 2; the passes end after the first
// one that moves Σ C|ΔT| by at most 10⁻⁶ Σ C T⁰ and every cell by at most 10⁻³ of its T. On the linear law the second pass
// is the check: its residual is recomputed from the solved u, so a recursive residual that drifted from the true one is
// solved again, not committed. The ceiling (64 passes, 4,096 rounds; §0.5 [M0-D37]) commits the last iterate unconverged.
//
// One dispatch a round (Chronopoulos–Gear's form of the method: one reduction point a round): `heat_round` is recorded
// MAX_ROUNDS + 3 MAX_PASSES + 2 times, each slot reading its workgroup count from a word the slot before it wrote — the
// args buffer of its parity, so no dispatch writes the buffer it is sized by — and each slot's workgroups first sum the
// previous slot's partial sums, every workgroup in the same fixed order (§1.3.4), so all of them take the same decision.
// What a slot does is the record's mode: INIT builds the first pass's system at T⁰; RESID ends the passes or starts the
// next one's solve (r = b − A x, from the iterate, with A z and zᵀr, zᵀA z); START shifts x and r along z and forms
// w = A D⁻¹r; SOLVE is a round, or — once the measures hold,
// the rounds run out or the method breaks down — the Newton update and the next pass's system (the iterate's T, x = u, the
// diagonal, the right side), the pass's end test left to the RESID after it. A slot that ends the step enables none
// after it; `heat_sum` zeroes every slot's word, so the next step's rounds run only when its own `heat_open` starts them,
// and a frame the latch stopped (§1.8.5: `heat_open` is one of its fixed sizes) runs none. A submission records the rounds
// the engine chooses, an even count (§0.5 [M0-TJ4] (6), M0-T121): a step still `live` after them is carried — latch.wgsl's
// `heat_park` zeroes the slots' words, `latch_carry` the rest of the submission, and the next one's `latch_hold` starts its
// rounds again from slot 0 of parity 0 (the record of parity 1 is the last one written), its commit and P6–P9 after them.
// The commit: `heat_close` writes F = −D_f (u_b − u_a) at the solved u (the start state's when the step is pinned or
// not run), side.flux and each cell's `radiated` Δt Σ_vac D_f (u_i − u_vac), with the heat sums; `heat_sum` closes the
// balance, δ = −(Σ C (T − T⁰) + R)/Σ C T; `heat_apply` adds C (T − T⁰ + δ T) to E.

// The machine's record, one per parity: the slot that reads it, its mode, the passes and rounds so far, this pass's
// rounds, how many partial sums the slot reads, the rounds' tiles, Δt, Σ C T⁰, this pass's two bounds and the last
// round's γ and η.
struct Machine {
    slot: u32,
    mode: u32,
    passes: u32,
    rounds: u32,
    k: u32,
    parts: u32,
    tx: u32,
    ty: u32,
    dt: f32,
    heat0: f32,
    u_bound: f32,
    t_bound: f32,
    gamma: f32,
    eta: f32,
    _a: u32,
    _b: u32,
}

// What the step did (step/heat.rs `HeatReport`): its passes, rounds, whether it converged and whether it ran, then the
// closure δ and the sums it came from; whether its rounds are still running (latch.wgsl's `latch_carry` reads it), its
// step index, how many unconverged steps `log` holds since headless last read it (step/heat.rs `heat_warnings`), and the
// most slots a step ended since the submission began (2 + 3 passes + rounds; the engine's choice of rounds learns it).
struct Report {
    passes: u32,
    rounds: u32,
    converged: u32,
    stepped: u32,
    delta: f32,
    gained: f32,
    radiated: f32,
    held: f32,
    live: u32,
    step: u32,
    logged: u32,
    most: u32,
}

// The steps that reached the ceiling (§0.5 [M0-TJ4] (6)): (step, passes, rounds, 0), the first LOG since the last read.
const LOG: u32 = 256u;

struct Machines {
    rec: array<Machine, 2>,
    report: Report,
    log: array<vec4<u32>, LOG>,
}

struct Parity {
    p: u32,
    _a: u32,
    _b: u32,
    _c: u32,
}

// The iterate's u (x), its T, CG's p, the diagonal and the right side, one plane each.
@group(0) @binding(14) var<storage, read_write> solve: array<f32>;
// r, w and s, each in two parities: a slot writes its own and reads the other, which neighbours do not change under it.
@group(0) @binding(15) var<storage, read_write> krylov: array<f32>;
@group(0) @binding(16) var<storage, read_write> machine: Machines;
// The rounds' partial sums, one 16 × 16 tile each, per parity (params.tiles a parity), then `heat_close`'s, one 8 × 8 tile each.
@group(0) @binding(17) var<storage, read_write> partials: array<vec4<f32>>;
// The next slot's workgroup counts (three words a slot, the slots of one parity); `heat_sum` zeroes this one and the other.
@group(0) @binding(18) var<storage, read_write> args_next: array<u32>;
@group(0) @binding(19) var<storage, read_write> args_other: array<u32>;
@group(0) @binding(20) var<uniform> parity: Parity;

const SOLVE_TOL: f32 = 1.0e-5;
const NEWTON_HEAT: f32 = 1.0e-6;
const NEWTON_CELL: f32 = 1.0e-3;
const MAX_PASSES: u32 = 64u;
const MAX_ROUNDS: u32 = 4096u;
const TILE: u32 = 16u;
// A gas cell whose heat term c_i is at most this share of its faces' Σ_f D_f is heat-free (`free_at`).
const FREE_SHARE: f32 = 1.0e-3;
const DO_INIT: u32 = 0u;
const DO_RESID: u32 = 1u;
const DO_START: u32 = 2u;
const DO_SOLVE: u32 = 3u;
const DO_NOTHING: u32 = 4u;
const SV_X: u32 = 0u;
const SV_T: u32 = 1u;
const SV_P: u32 = 2u;
const SV_D: u32 = 3u;
const SV_B: u32 = 4u;
const KR_R: u32 = 0u;
const KR_W: u32 = 2u;
const KR_S: u32 = 4u;
// What a slot does to its cells.
const CELL_NONE: u32 = 0u;
const CELL_BUILD: u32 = 1u;
const CELL_RESID: u32 = 2u;
const CELL_START: u32 = 3u;
const CELL_ROUND: u32 = 4u;
const CELL_UPDATE: u32 = 5u;

var<workgroup> wsum: array<vec4<f32>, 256>;

// wsum[0] ← the sum of wsum[0 .. 2 half), halved in a fixed order; every invocation of the workgroup calls it.
fn tree(li: u32, half: u32) {
    for (var s = half; s > 0u; s >>= 1u) {
        if (li < s) {
            wsum[li] += wsum[li + s];
        }
        workgroupBarrier();
    }
}

// u of a temperature, its inverse and dT/du: u = a_r T⁴ (a_r T² first, so T⁴ never overflows alone), or u = T under
// constant_diffusivity.
fn u_of(t: f32) -> f32 {
    if ((params.flags & CONSTANT_DIFFUSIVITY) != 0u) {
        return t;
    }
    let q = t * t;
    return params.a_r * q * q;
}

fn t_of(u: f32) -> f32 {
    if ((params.flags & CONSTANT_DIFFUSIVITY) != 0u) {
        return u;
    }
    return sqrt(sqrt(u / params.a_r));
}

fn slope(t: f32) -> f32 {
    if ((params.flags & CONSTANT_DIFFUSIVITY) != 0u) {
        return 1.0;
    }
    return 1.0 / (4.0 * params.a_r * t * t * t);
}

fn gas_at(x: u32, y: u32) -> bool {
    return in_box(x, y) && frozen[CAP * dims.cells + y * dims.width + x] > 0.0;
}

// D_f of cell (x, y)'s −x, +x, −y and +y faces.
fn face_d(x: u32, y: u32) -> array<f32, 4> {
    let w = dims.width;
    return array<f32, 4>(faces[y * (w + 1u) + x], faces[y * (w + 1u) + x + 1u], faces[nx() + y * w + x], faces[nx() + (y + 1u) * w + x]);
}

fn neighbours(x: u32, y: u32) -> array<vec2<u32>, 4> {
    return array<vec2<u32>, 4>(vec2<u32>(x - 1u, y), vec2<u32>(x + 1u, y), vec2<u32>(x, y - 1u), vec2<u32>(x, y + 1u));
}

// (Σ_f D_f, Σ_f D_f over the faces whose far side is not gas) of cell (x, y).
fn d_sums(x: u32, y: u32) -> vec2<f32> {
    var d = face_d(x, y);
    var nb = neighbours(x, y);
    var all = 0.0;
    var vac = 0.0;
    for (var f = 0u; f < 4u; f++) {
        all += d[f];
        if (!gas_at(nb[f].x, nb[f].y)) {
            vac += d[f];
        }
    }
    return vec2<f32>(all, vac);
}

// Whether (x, y) is a heat-free gas cell of this pass: c_i ≤ FREE_SHARE Σ_f D_f, c_i taken from C_i and the pass's T (never
// as the diagonal less Σ_f D_f, which f32 loses where Σ_f D_f is 10⁵ c_i). Such a cell only passes light on: a thin ring's
// cells, coupled to each other ~10⁵ times more strongly than to the disk and the vacuum, have their common level as the
// system's slowest mode — Jacobi-PCG barely moves it, and (4)'s measures, weighing heat, do not see it.
fn free_at(x: u32, y: u32, dt: f32) -> bool {
    if (!gas_at(x, y)) {
        return false;
    }
    let n = dims.cells;
    let i = y * dims.width + x;
    let c = frozen[CAP * n + i] / dt * slope(solve[SV_T * n + i]);
    return c <= FREE_SHARE * d_sums(x, y).x;
}

// (s_k, r_{k+1}) of cell j in a round: s_k = w_k + β s_{k−1} (w_k at a solve's first round), r_{k+1} = r_k − α s_k.
fn advance(j: u32, old: u32, a: f32, b: f32, first: bool) -> vec2<f32> {
    let n = dims.cells;
    var s = krylov[(KR_W + old) * n + j];
    if (!first) {
        s += b * krylov[(KR_S + old) * n + j];
    }
    return vec2<f32>(s, krylov[(KR_R + old) * n + j] - a * s);
}

// v_j of a neighbour: 0 the iterate's x, 1 D⁻¹r of parity `old` once the start's shift `a` moved it (r − a A z, A z in
// p's plane), 2 D⁻¹r_{k+1} of the round (α, β).
fn value(j: u32, which: u32, old: u32, a: f32, b: f32, first: bool) -> f32 {
    let n = dims.cells;
    if (which == 0u) {
        return solve[SV_X * n + j];
    }
    if (which == 1u) {
        return (krylov[(KR_R + old) * n + j] - a * solve[SV_P * n + j]) / solve[SV_D * n + j];
    }
    return advance(j, old, a, b, first).y / solve[SV_D * n + j];
}

// Σ_f D_f v_j over cell (x, y)'s gas neighbours: the off-diagonal part of A v.
fn coupled(x: u32, y: u32, which: u32, old: u32, a: f32, b: f32, first: bool) -> f32 {
    var d = face_d(x, y);
    var nb = neighbours(x, y);
    var sum = 0.0;
    for (var f = 0u; f < 4u; f++) {
        let o = nb[f];
        if (gas_at(o.x, o.y)) {
            sum += d[f] * value(o.y * dims.width + o.x, which, old, a, b, first);
        }
    }
    return sum;
}

// Gas cell (x, y)'s pass system at the iterate T = t into `solve`; returns (c u², (C/Δt) T²), the bounds' terms.
fn build(x: u32, y: u32, t: f32, dt: f32) -> vec2<f32> {
    let n = dims.cells;
    let i = y * dims.width + x;
    let heat = frozen[CAP * n + i] / dt;
    let c = heat * slope(t);
    let ds = d_sums(x, y);
    let u = u_of(t);
    solve[SV_T * n + i] = t;
    solve[SV_X * n + i] = u;
    solve[SV_D * n + i] = c + ds.x;
    solve[SV_B * n + i] = c * u - heat * (t - frozen[T0 * n + i]) + ds.y * u_of(params.t_floor);
    return vec2<f32>(c * u * u, heat * t * t);
}

@compute @workgroup_size(8, 8)
fn heat_law(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let w = dims.width;
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    put_x(y * (w + 1u) + x, face(side_at(x - 1u, y), side_at(x, y)));
    put_y(y * w + x, face(side_at(x, y - 1u), side_at(x, y)));
    if (id.x + 1u == abox.w) {
        put_x(y * (w + 1u) + x + 1u, face(side_at(x, y), side_at(x + 1u, y)));
    }
    if (id.y + 1u == abox.h) {
        put_y((y + 1u) * w + x, face(side_at(x, y), side_at(x, y + 1u)));
    }
}

@compute @workgroup_size(1)
fn heat_open() {
    let dt = dt_rec[0];
    let tx = (abox.w + TILE - 1u) / TILE;
    let ty = (abox.h + TILE - 1u) / TILE;
    let stepped = (params.flags & PIN_TEMPERATURE) == 0u && dt > 0.0 && finite(dt) && tx * ty > 0u;
    let on = select(0u, 1u, stepped);
    machine.rec[1] = Machine(0u, DO_INIT, 0u, 0u, 0u, 0u, tx, ty, dt, 0.0, 0.0, 0.0, 0.0, 0.0, 0u, 0u);
    // reduce.wgsl's DtState: its third word is the step in flight.
    let step = bitcast<u32>(dt_rec[2]);
    let kept = machine.report;
    machine.report = Report(0u, 0u, 1u - on, on, 0.0, 0.0, 0.0, 0.0, on, step, kept.logged, kept.most);
    if (stepped) {
        args_next[0] = tx;
        args_next[1] = ty;
        args_next[2] = 1u;
    }
}

@compute @workgroup_size(16, 16)
fn heat_round(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    let me = parity.p;
    let old = 1u - me;
    let rec = machine.rec[old];
    var acc = vec4<f32>(0.0);
    for (var i = li; i < rec.parts; i += 256u) {
        acc += partials[old * params.tiles + i];
    }
    wsum[li] = acc;
    workgroupBarrier();
    tree(li, 128u);
    let total = wsum[0];
    workgroupBarrier();

    // The slot's decision, the same in every workgroup.
    var next = rec;
    next.slot = rec.slot + 1u;
    next.parts = rec.tx * rec.ty;
    var action = CELL_NONE;
    var go_on = true;
    var finish = false;
    var converged = false;
    let first = rec.k == 0u;
    var alpha = 0.0;
    var beta = 0.0;
    var shift = 0.0;
    switch rec.mode {
        case DO_INIT: {
            action = CELL_BUILD;
            next.mode = DO_RESID;
        }
        case DO_RESID: {
            if (rec.passes == 0u) {
                next.heat0 = total.x;
            }
            let empty = rec.passes == 0u && !(total.x > 0.0);
            converged = empty || (rec.passes > 0u && total.x <= NEWTON_HEAT * rec.heat0 && total.y == 0.0);
            finish = converged || rec.passes == MAX_PASSES || rec.rounds >= MAX_ROUNDS;
            if (!finish) {
                action = CELL_RESID;
                next.mode = DO_START;
                next.passes = rec.passes + 1u;
                next.k = 0u;
                next.u_bound = SOLVE_TOL * SOLVE_TOL * total.z;
                next.t_bound = SOLVE_TOL * SOLVE_TOL * total.w;
            }
        }
        case DO_START: {
            // The start's shift along z, the heat-free cells' indicator: zᵀr / zᵀA z (none when no cell is heat-free).
            shift = select(0.0, total.x / total.y, total.y > 0.0);
            action = CELL_START;
            next.mode = DO_SOLVE;
        }
        case DO_SOLVE: {
            let gamma = total.x;
            var eta = total.y;
            if (!first) {
                beta = gamma / rec.gamma;
                eta = total.y - beta * beta * rec.eta;
            }
            var stop = (gamma <= rec.u_bound && total.z <= rec.t_bound) || !(eta > 0.0);
            stop = stop || rec.rounds >= MAX_ROUNDS;
            if (stop) {
                action = CELL_UPDATE;
                next.mode = DO_RESID;
            } else {
                action = CELL_ROUND;
                alpha = gamma / eta;
                next.gamma = gamma;
                next.eta = eta;
                next.k = rec.k + 1u;
                next.rounds = rec.rounds + 1u;
            }
        }
        default: {
            go_on = false;
        }
    }
    if (finish) {
        next.mode = DO_NOTHING;
        go_on = false;
    }

    let x = abox.x0 + wid.x * TILE + lid.x;
    let y = abox.y0 + wid.y * TILE + lid.y;
    var part = vec4<f32>(0.0);
    if (action != CELL_NONE && gas_at(x, y)) {
        let n = dims.cells;
        let i = y * dims.width + x;
        let c = frozen[CAP * n + i];
        let heat = c / rec.dt;
        switch action {
            case CELL_BUILD: {
                let t0 = frozen[T0 * n + i];
                let q = build(x, y, t0, rec.dt);
                part = vec4<f32>(c * t0, 0.0, q.x, q.y);
            }
            case CELL_RESID: {
                let d = solve[SV_D * n + i];
                let xi = solve[SV_X * n + i];
                krylov[(KR_R + me) * n + i] = solve[SV_B * n + i] - (d * xi - coupled(x, y, 0u, old, 0.0, 0.0, true));
                // The start's Galerkin shift along z (`free_at`): (A z)_i into p's plane, which the solve's first round
                // overwrites, and this cell's share of zᵀr and zᵀA z — taken face by face over the faces leaving the
                // heat-free cells, so the huge D_f between two of them cancel exactly instead of in f32's sum of r.
                var fd = face_d(x, y);
                var nb = neighbours(x, y);
                let free = free_at(x, y, rec.dt);
                var az = 0.0;
                var zax = 0.0;
                if (free) {
                    az = heat * slope(solve[SV_T * n + i]);
                    zax = az * xi;
                }
                for (var f = 0u; f < 4u; f++) {
                    let o = nb[f];
                    let other = free_at(o.x, o.y, rec.dt);
                    if (free && !other) {
                        az += fd[f];
                        zax += fd[f] * xi;
                        if (gas_at(o.x, o.y)) {
                            zax -= fd[f] * solve[SV_X * n + o.y * dims.width + o.x];
                        }
                    } else if (!free && other) {
                        az -= fd[f];
                    }
                }
                solve[SV_P * n + i] = az;
                if (free) {
                    part = vec4<f32>(solve[SV_B * n + i] - zax, az, 0.0, 0.0);
                }
            }
            case CELL_START: {
                let d = solve[SV_D * n + i];
                let r = krylov[(KR_R + old) * n + i] - shift * solve[SV_P * n + i];
                if (free_at(x, y, rec.dt)) {
                    solve[SV_X * n + i] += shift;
                }
                let u = r / d;
                let w = d * u - coupled(x, y, 1u, old, shift, 0.0, true);
                krylov[(KR_R + me) * n + i] = r;
                krylov[(KR_W + me) * n + i] = w;
                let e = r * slope(solve[SV_T * n + i]) / d;
                part = vec4<f32>(r * u, w * u, heat * e * e, 0.0);
            }
            case CELL_ROUND: {
                let d = solve[SV_D * n + i];
                var p = krylov[(KR_R + old) * n + i] / d;
                if (!first) {
                    p += beta * solve[SV_P * n + i];
                }
                let sr = advance(i, old, alpha, beta, first);
                solve[SV_P * n + i] = p;
                solve[SV_X * n + i] += alpha * p;
                let r = sr.y;
                let u = r / d;
                let w = d * u - coupled(x, y, 2u, old, alpha, beta, first);
                krylov[(KR_S + me) * n + i] = sr.x;
                krylov[(KR_R + me) * n + i] = r;
                krylov[(KR_W + me) * n + i] = w;
                let e = r * slope(solve[SV_T * n + i]) / d;
                part = vec4<f32>(r * u, w * u, heat * e * e, 0.0);
            }
            case CELL_UPDATE: {
                let t = solve[SV_T * n + i];
                let moved = clamp(t_of(max(solve[SV_X * n + i], u_of(params.t_floor))), 0.5 * t, 2.0 * t);
                let dif = abs(moved - t);
                let q = build(x, y, moved, rec.dt);
                part = vec4<f32>(c * dif, select(0.0, 1.0, dif > NEWTON_CELL * t), q.x, q.y);
            }
            default: {}
        }
    }
    wsum[li] = part;
    workgroupBarrier();
    tree(li, 128u);
    if (li == 0u) {
        partials[me * params.tiles + wid.y * rec.tx + wid.x] = wsum[0];
        if (wid.x == 0u && wid.y == 0u) {
            machine.rec[me] = next;
            if (go_on) {
                let at = (next.slot / 2u) * 3u;
                args_next[at] = rec.tx;
                args_next[at + 1u] = rec.ty;
                args_next[at + 2u] = 1u;
            }
            if (finish) {
                machine.report.passes = rec.passes;
                machine.report.rounds = rec.rounds;
                machine.report.converged = select(0u, 1u, converged);
                machine.report.live = 0u;
                machine.report.most = max(machine.report.most, 2u + 3u * rec.passes + rec.rounds);
                if (!converged) {
                    let i = machine.report.logged;
                    if (i < LOG) {
                        machine.log[i] = vec4<u32>(machine.report.step, rec.passes, rec.rounds, 0u);
                    }
                    machine.report.logged = i + 1u;
                }
            }
        }
    }
}

// T of gas cell `cell` after the step: the iterate when the step ran, T⁰ when it did not.
fn final_t(cell: u32) -> f32 {
    if (machine.report.stepped != 0u) {
        return solve[SV_T * dims.cells + cell];
    }
    return frozen[T0 * dims.cells + cell];
}

// u on one side of a face: the gas cell's after the step, else the vacuum's.
fn u_side(x: u32, y: u32) -> f32 {
    if (gas_at(x, y)) {
        return u_of(final_t(y * dims.width + x));
    }
    return u_of(params.t_floor);
}

// F = −D_f (u_b − u_a) of the face from (xa, ya) to (xb, yb); 0 where D_f is (two vacuums).
fn solved(xa: u32, ya: u32, xb: u32, yb: u32, d: f32) -> f32 {
    if (d == 0.0) {
        return 0.0;
    }
    return -d * (u_side(xb, yb) - u_side(xa, ya));
}

@compute @workgroup_size(8, 8)
fn heat_close(@builtin(global_invocation_id) id: vec3<u32>, @builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    var part = vec4<f32>(0.0);
    if (id.x < abox.w && id.y < abox.h) {
        let w = dims.width;
        let n = dims.cells;
        let x = abox.x0 + id.x;
        let y = abox.y0 + id.y;
        let cell = y * w + x;
        var d = face_d(x, y);
        let fl = solved(x - 1u, y, x, y, d[0]);
        let fr = solved(x, y, x + 1u, y, d[1]);
        let ft = solved(x, y - 1u, x, y, d[2]);
        let fb = solved(x, y, x, y + 1u, d[3]);
        let fx = nx() + ny();
        let fy = 2u * nx() + ny();
        faces[fx + y * (w + 1u) + x] = fl;
        faces[fy + y * w + x] = ft;
        if (id.x + 1u == abox.w) {
            faces[fx + y * (w + 1u) + x + 1u] = fr;
        }
        if (id.y + 1u == abox.h) {
            faces[fy + (y + 1u) * w + x] = fb;
        }
        flux[cell] = 0.5 * (fl + fr);
        flux[n + cell] = 0.5 * (ft + fb);
        if (machine.report.stepped != 0u && gas_at(x, y)) {
            let t = final_t(cell);
            let c = frozen[CAP * n + cell];
            let emitted = machine.rec[1].dt * d_sums(x, y).y * (u_of(t) - u_of(params.t_floor));
            book[cell] += emitted;
            part = vec4<f32>(c * (t - frozen[T0 * n + cell]), emitted, c * t, 0.0);
        }
    }
    wsum[li] = part;
    workgroupBarrier();
    tree(li, 32u);
    if (li == 0u) {
        partials[2u * params.tiles + wid.y * ((abox.w + 7u) / 8u) + wid.x] = wsum[0];
    }
}

@compute @workgroup_size(256)
fn heat_sum(@builtin(local_invocation_index) li: u32) {
    let count = ((abox.w + 7u) / 8u) * ((abox.h + 7u) / 8u);
    var acc = vec4<f32>(0.0);
    for (var i = li; i < count; i += 256u) {
        acc += partials[2u * params.tiles + i];
    }
    wsum[li] = acc;
    workgroupBarrier();
    tree(li, 128u);
    if (li == 0u) {
        let s = wsum[0];
        var delta = 0.0;
        if (machine.report.stepped != 0u && s.z > 0.0) {
            delta = -(s.x + s.y) / s.z;
        }
        machine.report.delta = delta;
        machine.report.gained = s.x;
        machine.report.radiated = s.y;
        machine.report.held = s.z;
    }
    for (var i = li; i < arrayLength(&args_next); i += 256u) {
        args_next[i] = 0u;
    }
    for (var i = li; i < arrayLength(&args_other); i += 256u) {
        args_other[i] = 0u;
    }
}

// After the rounds a submission records (recorded bare, so a held step's continuation runs it): a step still `live` is
// carried (§0.5 [M0-TJ4] (6)) — the words its slots wrote zeroed (slot j's at j / 2 in parity j % 2's buffer, the last
// run having written slot `rec[1].slot`'s) and its next slot made 0, so the next submission's `latch_hold` starts it
// there. Running again where nothing is live, or after the zeroing, changes nothing.
@compute @workgroup_size(1)
fn heat_park() {
    if (machine.report.live == 0u) {
        return;
    }
    let words = min((machine.rec[1].slot / 2u + 1u) * 3u, arrayLength(&args_next));
    for (var i = 0u; i < words; i++) {
        args_next[i] = 0u;
        args_other[i] = 0u;
    }
    machine.rec[1].slot = 0u;
}

@compute @workgroup_size(8, 8)
fn heat_apply(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h || machine.report.stepped == 0u) {
        return;
    }
    let n = dims.cells;
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
    let c = frozen[CAP * n + cell];
    if (c <= 0.0) {
        return;
    }
    let t = solve[SV_T * n + cell];
    hydro[3u * n + cell] += c * (t - frozen[T0 * n + cell] + machine.report.delta * t);
}
