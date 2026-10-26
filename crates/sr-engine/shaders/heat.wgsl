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
// one that moves Σ C|ΔT| by at most 10⁻⁶ Σ C T⁰ and every cell by at most 10⁻³ of its T. Where Σ_f D_f is 10⁵ c_i (thin gas
// at the shipped a_r; heat_spot) f32 must not lose what the measures weigh (M0-D56): every product with A and the residual
// are taken face by face, c_i v_i + Σ_f D_f (v_i − v_f), and the iterate's u lives in two words (x + lo) from pass to pass —
// f32's ε of u alone leaves b − A x ~20× the u-measure's bound, so every pass ran a round and the passes never ended (64 on
// heat_spot's first step, the twin 8). On the linear law the second pass
// is the check: its residual is recomputed from the solved u, so a recursive residual that drifted from the true one is
// solved again, not committed. The ceiling (64 passes, 4,096 rounds; §0.5 [M0-D37]) commits the last iterate's F by limited
// fluxes (§0.5 [M0-D89], below), and so does a step whose sums leave f32's range (§0.5 [M0-D93]): a thin cell beside
// vacuum at T ~ 10⁷ holds u ~ 10²⁶, so Σ c u² and the rounds' γ and η overflow, the method would stop as solved at round 0
// pass after pass and the Newton test read the unmoved iterate as converged — a pass bound, γ, η or a measure that is not
// finite ends the step there, unconverged.
//
// One dispatch a round (Chronopoulos–Gear's form of the method: one reduction point a round): `heat_round` is recorded
// MAX_ROUNDS + 3 MAX_PASSES + 2 times, each slot reading its workgroup count from a word the slot before it wrote — the
// args buffer of its parity, so no dispatch writes the buffer it is sized by — and each slot's workgroups first sum the
// previous slot's partial sums, every workgroup in the same fixed order (§1.3.4), so all of them take the same decision.
// What a slot does is the record's mode: INIT builds the first pass's system at the first iterate (T⁰; on the V-cycle
// T⁰ + the last step's P5 rate × Δt, §0.5 [M0-D85]); RESID ends the passes or starts the
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
// balance, δ = −(Σ C (T − T⁰) + R)/Σ C T; `heat_apply` adds C (T − T⁰ + δ T) to E and keeps the step's rate (T − T⁰)/Δt
// for the next step's first iterate (§0.5 [M0-D85]). Past the ceiling (§0.5 [M0-D89]) the last iterate's T is no answer —
// a hot cell beside vacuum, still within a factor 2 of its T⁰, emits ~10¹² times the heat it holds and δ took that from
// every cell — so the heat moves by limited fluxes instead: `heat_close` keeps each gas cell's scale s = min(1, held/outflow)
// (the heat above its floor over what its faces would carry out), `heat_limit` takes every face's F at the scale of the cell
// it leaves and books the cell's change −Δt Σ F and its light, δ = 0, and `heat_apply` adds the change and rewrites its
// faces' F.
//
// The V-cycle as the preconditioner (§0.5 [M0-D82]; M0-D83; `parity.mg`): M⁻¹r is no longer cell-local, so a slot is a
// group — its `heat_round`, then the levels (`mg_level0`, `mg_coarsen`), the V-cycle (the last section) and `heat_matvec`,
// each sized by the slot's `heat_round` in the multigrid's words after the slots' words of its own args buffer (`mg_size`),
// none when the slot has no use for them. RESID builds the pass's levels at its iterate; START and a round put their r
// (the shifted start's, r_{k+1}) as level 0's right side, the V-cycle makes z = M⁻¹r, and `heat_matvec` forms w = A z
// and the next slot's sums (zᵀr, zᵀw, the heat measure, and rᵀD⁻¹r: (4)'s u-measure is Jacobi's whatever M is); a round
// takes the last V-cycle's z where Jacobi takes D⁻¹r. Every slot in the mode writes all its words, and the slots on both
// sides of a step's end (an UPDATE or INIT, then the ending RESID) write none on, so a slot that does not run finds its
// group's words zero; `heat_park` zeroes both parities' at a carry, `heat_sum` at the step's end. The levels, z and the
// Krylov planes live in their buffers between slots, so a carried pass goes on from what its last slot left.

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

// The slot's parity, and whether its round runs on the V-cycle (step/heat.rs `HeatPrecond`).
struct Parity {
    p: u32,
    mg: u32,
    _b: u32,
    _c: u32,
}

// The iterate's u (x), its T, CG's p, the diagonal and u's low word (x + lo is the iterate's u), one plane each.
@group(0) @binding(14) var<storage, read_write> solve: array<f32>;
// r, w and s, each in two parities: a slot writes its own and reads the other, which neighbours do not change under it.
@group(0) @binding(15) var<storage, read_write> krylov: array<f32>;
@group(0) @binding(16) var<storage, read_write> machine: Machines;
// The rounds' partial sums, one 16 × 16 tile each, per parity (params.tiles a parity), then `heat_close`'s, one 8 × 8 tile each.
@group(0) @binding(17) var<storage, read_write> partials: array<vec4<f32>>;
// The next slot's workgroup counts (three words a slot, the slots of one parity), then the multigrid's MGA_COUNT for the
// slot's own group; `heat_sum` zeroes this one and the other.
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
const SV_L: u32 = 4u;
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
    return heat_term(y * dims.width + x, dt) <= FREE_SHARE * d_sums(x, y).x;
}

// c_i of gas cell i, this pass's heat term (C_i/Δt) dT/du at the iterate's T: the diagonal less Σ_f D_f, taken from C_i and
// T, never as that difference (f32 holds the diagonal to ε of Σ_f D_f, a few % of c_i in thin gas at the shipped a_r).
fn heat_term(i: u32, dt: f32) -> f32 {
    let n = dims.cells;
    return frozen[CAP * n + i] / dt * slope(solve[SV_T * n + i]);
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

// v_j of a neighbour: 1 D⁻¹r of parity `old` once the start's shift `a` moved it (r − a A z, A z in p's plane), 2 D⁻¹r_{k+1}
// of the round (α, β).
fn value(j: u32, which: u32, old: u32, a: f32, b: f32, first: bool) -> f32 {
    let n = dims.cells;
    if (which == 1u) {
        return (krylov[(KR_R + old) * n + j] - a * solve[SV_P * n + j]) / solve[SV_D * n + j];
    }
    return advance(j, old, a, b, first).y / solve[SV_D * n + j];
}

// (A v)_i of gas cell (x, y), v_i = `own`, c_i = `c`, the gas neighbours' v from `value`: c_i v_i + Σ_f D_f (v_i − v_f), v_f = 0
// beyond a vacuum or edge face — face by face (M0-D56): on a smooth v, where A v is ~c_i v_i and Σ_f D_f is 10⁵ c_i (thin gas
// at the shipped a_r), d v_i − Σ_f D_f v_j cancels away f32's digits and the rounds' r drifts 10³ from b − A x.
fn applied(x: u32, y: u32, c: f32, own: f32, which: u32, old: u32, a: f32, b: f32, first: bool) -> f32 {
    var d = face_d(x, y);
    var nb = neighbours(x, y);
    var sum = c * own;
    for (var f = 0u; f < 4u; f++) {
        let o = nb[f];
        var far = 0.0;
        if (gas_at(o.x, o.y)) {
            far = value(o.y * dims.width + o.x, which, old, a, b, first);
        }
        sum += d[f] * (own - far);
    }
    return sum;
}

// x_i += v in its two words, hi + lo (Fast2Sum: |v| and lo far under hi in a solve's rounds), so the iterate keeps u to
// ~ε² — f32's ε of u alone leaves b − A x ~20× (4)'s bound where Σ_f D_f is 10⁵ c_i (M0-D56).
fn raise(i: u32, v: f32) {
    let n = dims.cells;
    let hi = solve[SV_X * n + i];
    let lo = solve[SV_L * n + i] + v;
    let s = hi + lo;
    solve[SV_L * n + i] = lo - (s - hi);
    solve[SV_X * n + i] = s;
}

// Gas cell (x, y)'s pass system at the iterate T = t into `solve` (the caller writes its u); returns (c u², (C/Δt) T²), the
// bounds' terms.
fn build(x: u32, y: u32, t: f32, dt: f32) -> vec2<f32> {
    let n = dims.cells;
    let i = y * dims.width + x;
    let heat = frozen[CAP * n + i] / dt;
    let c = heat * slope(t);
    let u = u_of(t);
    solve[SV_T * n + i] = t;
    solve[SV_D * n + i] = c + d_sums(x, y).x;
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
    let mg = parity.mg != 0u;
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
            // The next pass's bounds out of f32's range: no round can meet them (§0.5 [M0-D93]).
            let unmeasured = !(finite(total.z) && finite(total.w));
            converged = !unmeasured && (empty || (rec.passes > 0u && total.x <= NEWTON_HEAT * rec.heat0 && total.y == 0.0));
            finish = converged || unmeasured || rec.passes == MAX_PASSES || rec.rounds >= MAX_ROUNDS;
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
            // (4)'s u-measure is (r, D⁻¹r) whatever the preconditioner: Jacobi's γ, the V-cycle's fourth sum.
            let measure = select(gamma, total.w, mg);
            var stop = (measure <= rec.u_bound && total.z <= rec.t_bound) || !(eta > 0.0);
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
            // A round's sums out of f32's range: γ or η NaN would stop the method as solved, so the step ends unconverged
            // instead, its cells left at the iterate (§0.5 [M0-D93]).
            if (!(finite(gamma) && finite(eta) && finite(measure) && finite(total.z))) {
                action = CELL_NONE;
                finish = true;
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
                // The first iterate (§0.5 [M0-D85]): on the V-cycle T⁰ + the cell's P5 rate in the last step × Δt_n (the
                // rate `heat_apply` left in krylov's first s plane), T⁰ where that is off by more than a factor 2 or not
                // finite, floored at T_floor; on Jacobi T⁰.
                let t0 = frozen[T0 * n + i];
                var tg = t0;
                if (mg) {
                    tg = t0 + krylov[KR_S * n + i] * rec.dt;
                    if (!(tg >= 0.5 * t0 && tg <= 2.0 * t0)) {
                        tg = t0;
                    }
                    tg = max(tg, params.t_floor);
                }
                solve[SV_X * n + i] = u_of(tg);
                solve[SV_L * n + i] = 0.0;
                let q = build(x, y, tg, rec.dt);
                part = vec4<f32>(c * t0, 0.0, q.x, q.y);
            }
            case CELL_RESID: {
                // r = b − A x at the iterate (x = u, its two words): −(C_i/Δt)(T_i − T_i⁰) − Σ_f D_f (u_i − u_f), u_f = u_vac
                // beyond a vacuum or edge face — the step's own residual, face by face (M0-D56).
                // The start's Galerkin shift along z (`free_at`): (A z)_i into p's plane, which the solve's first round
                // overwrites, and this cell's share of zᵀr and zᵀA z — taken face by face over the faces leaving the
                // heat-free cells, so the huge D_f between two of them cancel exactly instead of in f32's sum of r.
                let xi = solve[SV_X * n + i];
                let lo = solve[SV_L * n + i];
                var fd = face_d(x, y);
                var nb = neighbours(x, y);
                let free = free_at(x, y, rec.dt);
                let kept = -heat * (solve[SV_T * n + i] - frozen[T0 * n + i]);
                var r = kept;
                var zr = kept;
                var az = 0.0;
                if (free) {
                    az = heat_term(i, rec.dt);
                }
                for (var f = 0u; f < 4u; f++) {
                    let o = nb[f];
                    let other = free_at(o.x, o.y, rec.dt);
                    var far = u_of(params.t_floor);
                    var far_lo = 0.0;
                    if (gas_at(o.x, o.y)) {
                        far = solve[SV_X * n + o.y * dims.width + o.x];
                        far_lo = solve[SV_L * n + o.y * dims.width + o.x];
                    }
                    let out = fd[f] * ((xi - far) + (lo - far_lo));
                    r -= out;
                    if (free && !other) {
                        az += fd[f];
                        zr -= out;
                    } else if (!free && other) {
                        az -= fd[f];
                    }
                }
                krylov[(KR_R + me) * n + i] = r;
                solve[SV_P * n + i] = az;
                if (free) {
                    part = vec4<f32>(zr, az, 0.0, 0.0);
                }
            }
            case CELL_START: {
                let d = solve[SV_D * n + i];
                let r = krylov[(KR_R + old) * n + i] - shift * solve[SV_P * n + i];
                if (free_at(x, y, rec.dt)) {
                    raise(i, shift);
                }
                krylov[(KR_R + me) * n + i] = r;
                if (mg) {
                    // z = M⁻¹r and w = A z follow, by the V-cycle and `heat_matvec`.
                    mg_vec[mg_layout.level[0].vec + mg_at(x, y)] = r;
                } else {
                    let u = r / d;
                    let w = applied(x, y, heat_term(i, rec.dt), u, 1u, old, shift, 0.0, true);
                    krylov[(KR_W + me) * n + i] = w;
                    let e = r * slope(solve[SV_T * n + i]) / d;
                    part = vec4<f32>(r * u, w * u, heat * e * e, 0.0);
                }
            }
            case CELL_ROUND: {
                let d = solve[SV_D * n + i];
                // p_k = z_k + β p_{k−1}: z_k is D⁻¹r_k, or the V-cycle's of the slot before.
                var p = krylov[(KR_R + old) * n + i] / d;
                if (mg) {
                    p = mg_vec[mg_z(0u, 1u) + mg_at(x, y)];
                }
                if (!first) {
                    p += beta * solve[SV_P * n + i];
                }
                let sr = advance(i, old, alpha, beta, first);
                solve[SV_P * n + i] = p;
                raise(i, alpha * p);
                let r = sr.y;
                krylov[(KR_S + me) * n + i] = sr.x;
                krylov[(KR_R + me) * n + i] = r;
                if (mg) {
                    mg_vec[mg_layout.level[0].vec + mg_at(x, y)] = r;
                } else {
                    let u = r / d;
                    let w = applied(x, y, heat_term(i, rec.dt), u, 2u, old, alpha, beta, first);
                    krylov[(KR_W + me) * n + i] = w;
                    let e = r * slope(solve[SV_T * n + i]) / d;
                    part = vec4<f32>(r * u, w * u, heat * e * e, 0.0);
                }
            }
            case CELL_UPDATE: {
                let t = solve[SV_T * n + i];
                let u = solve[SV_X * n + i] + solve[SV_L * n + i];
                let u_vac = u_of(params.t_floor);
                let free_t = t_of(max(u, u_vac));
                let moved = clamp(free_t, 0.5 * t, 2.0 * t);
                // The next pass starts from the solved u in its two words, unless the floor or the factor 2 moved it.
                if (moved != free_t || !(u >= u_vac)) {
                    solve[SV_X * n + i] = u_of(moved);
                    solve[SV_L * n + i] = 0.0;
                }
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
    if (mg && wid.x == 0u && wid.y == 0u) {
        // The slot's group: the levels after a RESID that goes on, the V-cycle and the matvec after a START or a round.
        let base = arrayLength(&args_next) - 3u * MGA_COUNT;
        for (var k = li; k < MGA_COUNT; k += 256u) {
            let s = mg_size(k, action == CELL_RESID, action == CELL_START || action == CELL_ROUND);
            args_next[base + 3u * k] = s.x;
            args_next[base + 3u * k + 1u] = s.y;
            args_next[base + 3u * k + 2u] = s.z;
        }
    }
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

// The end of a V-cycle slot (sized by its `heat_round`, MGA_MATVEC): w = A z at every gas cell, z the V-cycle's level-0 z up
// (0 beyond a vacuum or edge face), face by face as `applied`, and the slot's sums for the next — zᵀr, zᵀw, the heat
// measure and rᵀD⁻¹r — in the tiles `heat_round` wrote.
@compute @workgroup_size(16, 16)
fn heat_matvec(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    let me = parity.p;
    let rec = machine.rec[me];
    let x = abox.x0 + wid.x * TILE + lid.x;
    let y = abox.y0 + wid.y * TILE + lid.y;
    var part = vec4<f32>(0.0);
    if (gas_at(x, y)) {
        let n = dims.cells;
        let i = y * dims.width + x;
        let z = mg_z(0u, 1u);
        let own = mg_vec[z + mg_at(x, y)];
        var d = face_d(x, y);
        var nb = neighbours(x, y);
        var w = heat_term(i, rec.dt) * own;
        for (var f = 0u; f < 4u; f++) {
            var far = 0.0;
            if (gas_at(nb[f].x, nb[f].y)) {
                far = mg_vec[z + mg_at(nb[f].x, nb[f].y)];
            }
            w += d[f] * (own - far);
        }
        krylov[(KR_W + me) * n + i] = w;
        let r = krylov[(KR_R + me) * n + i];
        let diag = solve[SV_D * n + i];
        let e = r * slope(solve[SV_T * n + i]) / diag;
        part = vec4<f32>(r * own, w * own, frozen[CAP * n + i] / rec.dt * e * e, r * r / diag);
    }
    wsum[li] = part;
    workgroupBarrier();
    tree(li, 128u);
    if (li == 0u) {
        partials[me * params.tiles + wid.y * rec.tx + wid.x] = wsum[0];
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

// Whether the step commits by limited fluxes: it ran and reached the ceiling (§0.5 [M0-D89]).
fn limited() -> bool {
    return machine.report.stepped != 0u && machine.report.converged == 0u;
}

// A cell's outflow scale past the ceiling, `heat_close`'s in p's plane for a gas cell; 1 for vacuum.
fn scale_at(x: u32, y: u32) -> f32 {
    if (gas_at(x, y)) {
        return solve[SV_P * dims.cells + y * dims.width + x];
    }
    return 1.0;
}

// F of a face at the scale of the cell it leaves: `minus` is the −x (−y) side's scale, `plus` the other's.
fn limit(f: f32, minus: f32, plus: f32) -> f32 {
    return f * select(plus, minus, f > 0.0);
}

// The limited F of cell (x, y)'s −x, +x, −y and +y faces, from the F `heat_close` wrote.
fn limited_faces(x: u32, y: u32) -> array<f32, 4> {
    let w = dims.width;
    let fx = nx() + ny();
    let fy = 2u * nx() + ny();
    let me = scale_at(x, y);
    return array<f32, 4>(
        limit(faces[fx + y * (w + 1u) + x], scale_at(x - 1u, y), me),
        limit(faces[fx + y * (w + 1u) + x + 1u], me, scale_at(x + 1u, y)),
        limit(faces[fy + y * w + x], scale_at(x, y - 1u), me),
        limit(faces[fy + (y + 1u) * w + x], me, scale_at(x, y + 1u)),
    );
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
        if (limited() && gas_at(x, y)) {
            // Past the ceiling the light and the sums are `heat_limit`'s; here the cell's scale, s = min(1, held/outflow),
            // the heat above its floor over what its faces would carry out.
            let c = frozen[CAP * n + cell];
            let out = machine.rec[1].dt * (max(-fl, 0.0) + max(fr, 0.0) + max(-ft, 0.0) + max(fb, 0.0));
            let held = c * max(frozen[T0 * n + cell] - params.t_floor, 0.0);
            solve[SV_P * n + cell] = select(1.0, held / out, out > held);
        } else if (machine.report.stepped != 0u && gas_at(x, y)) {
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

// The commit past the ceiling (§0.5 [M0-D89]; the twin's `commit_limited`): each face's F from `heat_close` at the scale of
// the cell it leaves, side.flux from those, the cell's change of heat −Δt Σ_f F_f (outflow positive) in the iterate's low-word
// plane for `heat_apply` (the next step's first slot zeroes it), what leaves through vacuum and edge faces booked
// `radiated`, and the sums `heat_sum` reads in `heat_close`'s place. A converged step's commit is `heat_close`'s alone.
@compute @workgroup_size(8, 8)
fn heat_limit(@builtin(global_invocation_id) id: vec3<u32>, @builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    let on = limited();
    var part = vec4<f32>(0.0);
    if (on && id.x < abox.w && id.y < abox.h) {
        let n = dims.cells;
        let x = abox.x0 + id.x;
        let y = abox.y0 + id.y;
        let cell = y * dims.width + x;
        let f = limited_faces(x, y);
        flux[cell] = 0.5 * (f[0] + f[1]);
        flux[n + cell] = 0.5 * (f[2] + f[3]);
        if (gas_at(x, y)) {
            let out = array<f32, 4>(-f[0], f[1], -f[2], f[3]);
            var nb = neighbours(x, y);
            var leaving = 0.0;
            var light = 0.0;
            for (var k = 0u; k < 4u; k++) {
                leaving += out[k];
                if (!gas_at(nb[k].x, nb[k].y)) {
                    light += out[k];
                }
            }
            let dt = machine.rec[1].dt;
            let change = -dt * leaving;
            solve[SV_L * n + cell] = change;
            book[cell] += dt * light;
            part = vec4<f32>(change, dt * light, frozen[CAP * n + cell] * frozen[T0 * n + cell] + change, 0.0);
        }
    }
    wsum[li] = part;
    workgroupBarrier();
    tree(li, 32u);
    if (on && li == 0u) {
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
        // Past the ceiling the limited fluxes book each face on both sides: no closure (§0.5 [M0-D89]).
        var delta = 0.0;
        if (machine.report.stepped != 0u && machine.report.converged != 0u && s.z > 0.0) {
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
// run having written slot `rec[1].slot`'s), the multigrid's words of both parities too (a later step of the submission
// records slot groups that would read them), and its next slot made 0, so the next submission's `latch_hold` starts it
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
    for (var i = arrayLength(&args_next) - 3u * MGA_COUNT; i < arrayLength(&args_next); i++) {
        args_next[i] = 0u;
        args_other[i] = 0u;
    }
    machine.rec[1].slot = 0u;
}

@compute @workgroup_size(8, 8)
fn heat_apply(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let n = dims.cells;
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let cell = y * dims.width + x;
    let c = frozen[CAP * n + cell];
    let past = limited();
    if (past) {
        // Past the ceiling (§0.5 [M0-D89]) the faces this cell writes in `heat_close` — its −x and −y, and its +x and +y at
        // the box's edge — take their limited F, read and written by this cell alone.
        let w = dims.width;
        let me = scale_at(x, y);
        let left = nx() + ny() + y * (w + 1u) + x;
        let top = 2u * nx() + ny() + y * w + x;
        faces[left] = limit(faces[left], scale_at(x - 1u, y), me);
        faces[top] = limit(faces[top], scale_at(x, y - 1u), me);
        if (id.x + 1u == abox.w) {
            faces[left + 1u] = limit(faces[left + 1u], me, scale_at(x + 1u, y));
        }
        if (id.y + 1u == abox.h) {
            faces[top + w] = limit(faces[top + w], me, scale_at(x, y + 1u));
        }
    }
    // The cell's P5 rate in this step, (T − T⁰)/Δt_n, for the next step's first iterate (§0.5 [M0-D85]): krylov's first s
    // plane, which no round reads before writing it; 0 off the gas and when the step did not run. A dump carries it.
    if (machine.report.stepped == 0u || c <= 0.0) {
        krylov[KR_S * n + cell] = 0.0;
        return;
    }
    if (past) {
        let change = solve[SV_L * n + cell];
        hydro[3u * n + cell] += change;
        krylov[KR_S * n + cell] = change / (c * machine.rec[1].dt);
        return;
    }
    let t = solve[SV_T * n + cell];
    let t0 = frozen[T0 * n + cell];
    hydro[3u * n + cell] += c * (t - t0 + machine.report.delta * t);
    krylov[KR_S * n + cell] = (t - t0) / machine.rec[1].dt;
}

// ---- P5's multigrid preconditioner (§0.5 [M0-D82]; reports/heat_multigrid.md) ----
//
// The V-cycle M⁻¹ the PCG may take in Jacobi's place (step/heat.rs `HeatPrecond`; in the round machine since M0-D83, sized by
// `mg_size`, the levels built at the iterate there — `mg_step.start` 0; the tests' window builds at T⁰). Level 0
// is a pass's system over the box, cell (i, j) the box's (x0 + i, y0 + j): a gas cell's own term c_i + Σ_vac D_f, D_f between
// two gas cells, A v = own v + Σ_f D_f (v_i − v_f) face by face (M0-D56). Level l + 1 is Pᵀ A_l P over 2 × 2 aggregates
// (P piecewise constant): own terms and the faces between two aggregates summed, an aggregate with no active cell inactive
// (own −1); the levels halve until both sides are ≤ MG_LAST (9 at 600 × 400). Down each level of > MG_COARSE cells,
// `mg_smooth` (a 16 × 16 tile and its halo in 1.3 KB of workgroup memory, the halo frozen at the dispatch's start, red first)
// and `mg_restrict`; `mg_coarse`, one workgroup, the rest (from 19 × 13 at 600 × 400; the coarsest by MG_SOLVE symmetric
// pairs); up, `mg_smooth` on z + P z_coarse, black first — each the adjoint of its way down, so M⁻¹ is symmetric. z down and
// z up are two planes, so no tile reads a halo another workgroup of its dispatch writes.

const MG_LEVELS: u32 = 12u;
const MG_TILE: u32 = 16u;
const MG_HALO: u32 = 18u;
const MG_SWEEPS: u32 = 4u;
const MG_COARSE: u32 = 256u;
const MG_LAST: u32 = 4u;
const MG_SOLVE: u32 = 50u;

// Each level's planes, sized for the world's level (w × h) and indexed by the box's: the coefficients from `coef` (own,
// then the x faces, (w + 1) × h, face (x, y) between x − 1 and x, then the y faces, w × (h + 1)), the vectors from `vec`
// (the right side b, z down, z up).
struct MgLevel {
    coef: u32,
    vec: u32,
    w: u32,
    h: u32,
}

struct MgLayout {
    level: array<MgLevel, MG_LEVELS>,
}

// The level a dispatch works on, whether it goes up, and whether `mg_level0` linearises at T⁰ (the first pass's system).
struct MgStep {
    level: u32,
    up: u32,
    start: u32,
    _a: u32,
}

@group(0) @binding(21) var<storage, read_write> mg_coef: array<f32>;
@group(0) @binding(22) var<storage, read_write> mg_vec: array<f32>;
@group(0) @binding(23) var<uniform> mg_layout: MgLayout;
@group(0) @binding(24) var<uniform> mg_step: MgStep;

var<workgroup> mg_tile: array<f32, 324>;

// Level l's sides over the box.
fn mg_dims(l: u32) -> vec2<u32> {
    let s = (1u << l) - 1u;
    return vec2<u32>((abox.w + s) >> l, (abox.h + s) >> l);
}

// (the first level of at most MG_COARSE cells — it and the coarser ones run in `mg_coarse` —, the coarsest: both sides
// ≤ MG_LAST).
fn mg_span() -> vec2<u32> {
    var s = vec2<u32>(0u);
    while (mg_dims(s.x).x * mg_dims(s.x).y > MG_COARSE) {
        s.x++;
    }
    while (s.y + 1u < MG_LEVELS && max(mg_dims(s.y).x, mg_dims(s.y).y) > MG_LAST) {
        s.y++;
    }
    return s;
}

// Level 0's index of box cell (x, y).
fn mg_at(x: u32, y: u32) -> u32 {
    return (y - abox.y0) * abox.w + x - abox.x0;
}

// The multigrid's entries in a round machine's args buffer (after the slots' words): `mg_level0`, `mg_coarsen` a level,
// `mg_smooth` a level (down and up), `mg_restrict` a level, `mg_coarse`, `heat_matvec` (step/heat.rs records them so).
const MGA_LEVEL0: u32 = 0u;
const MGA_COARSEN: u32 = 1u;
const MGA_SMOOTH: u32 = 1u + MG_LEVELS;
const MGA_RESTRICT: u32 = 1u + 2u * MG_LEVELS;
const MGA_COARSE: u32 = 1u + 3u * MG_LEVELS;
const MGA_MATVEC: u32 = 2u + 3u * MG_LEVELS;
const MGA_COUNT: u32 = 3u + 3u * MG_LEVELS;

// Entry k's workgroups for a slot that builds the levels (`build`) or applies the V-cycle and the matvec (`cycle`): over
// the box's level, none for a level the box has not.
fn mg_size(k: u32, build: bool, cycle: bool) -> vec3<u32> {
    let span = mg_span();
    var on = cycle;
    var n = vec2<u32>(1u);
    if (k == MGA_LEVEL0) {
        on = build;
        n = (mg_dims(0u) + 7u) / 8u;
    } else if (k < MGA_SMOOTH) {
        on = build && k - MGA_COARSEN < span.y;
        n = (mg_dims(k - MGA_COARSEN + 1u) + 7u) / 8u;
    } else if (k < MGA_RESTRICT) {
        on = cycle && k - MGA_SMOOTH < span.x;
        n = (mg_dims(k - MGA_SMOOTH) + 15u) / 16u;
    } else if (k < MGA_COARSE) {
        on = cycle && k - MGA_RESTRICT < span.x;
        n = (mg_dims(k - MGA_RESTRICT + 1u) + 7u) / 8u;
    } else if (k == MGA_MATVEC) {
        n = (mg_dims(0u) + 15u) / 16u;
    }
    return select(vec3<u32>(0u), vec3<u32>(n, 1u), on);
}

fn mg_fx(l: u32) -> u32 {
    let v = mg_layout.level[l];
    return v.coef + v.w * v.h;
}

fn mg_fy(l: u32) -> u32 {
    let v = mg_layout.level[l];
    return mg_fx(l) + (v.w + 1u) * v.h;
}

// Level l's plane of z: down (0) or up (1).
fn mg_z(l: u32, up: u32) -> u32 {
    let v = mg_layout.level[l];
    return v.vec + (1u + up) * v.w * v.h;
}

// (the four faces' D_f, −x, +x, −y, +y) of cell (x, y) of level l, sides d.
fn mg_faces(l: u32, d: vec2<u32>, x: u32, y: u32) -> vec4<f32> {
    let fx = mg_fx(l) + y * (d.x + 1u) + x;
    let fy = mg_fy(l) + y * d.x + x;
    return vec4<f32>(mg_coef[fx], mg_coef[fx + 1u], mg_coef[fy], mg_coef[fy + d.x]);
}

// z at the four neighbours of cell i of level l (plane z); a face of D_f 0 reads nothing.
fn mg_near(z: u32, d: vec2<u32>, i: u32, f: vec4<f32>) -> vec4<f32> {
    var v = vec4<f32>(0.0);
    if (f.x != 0.0) {
        v.x = mg_vec[z + i - 1u];
    }
    if (f.y != 0.0) {
        v.y = mg_vec[z + i + 1u];
    }
    if (f.z != 0.0) {
        v.z = mg_vec[z + i - d.x];
    }
    if (f.w != 0.0) {
        v.w = mg_vec[z + i + d.x];
    }
    return v;
}

// b − A z at active cell (x, y) of level l, plane z, face by face.
fn mg_residual(l: u32, d: vec2<u32>, x: u32, y: u32, z: u32) -> f32 {
    let i = y * d.x + x;
    let f = mg_faces(l, d, x, y);
    let zi = mg_vec[z + i];
    let near = mg_near(z, d, i, f);
    return mg_vec[mg_layout.level[l].vec + i] - mg_coef[mg_layout.level[l].coef + i] * zi - dot(f, vec4<f32>(zi) - near);
}

@compute @workgroup_size(8, 8)
fn mg_level0(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let n = dims.cells;
    let cell = y * dims.width + x;
    let i = id.y * abox.w + id.x;
    let gas = gas_at(x, y);
    var d = face_d(x, y);
    var nb = neighbours(x, y);
    var own = -1.0;
    var near = vec4<f32>(0.0);
    if (gas) {
        let t = select(solve[SV_T * n + cell], frozen[T0 * n + cell], mg_step.start != 0u);
        own = frozen[CAP * n + cell] / machine.rec[1].dt * slope(t);
        for (var f = 0u; f < 4u; f++) {
            if (gas_at(nb[f].x, nb[f].y)) {
                near[f] = d[f];
            } else {
                own += d[f];
            }
        }
    }
    let fx = mg_fx(0u) + id.y * (abox.w + 1u) + id.x;
    let fy = mg_fy(0u) + id.y * abox.w + id.x;
    mg_coef[mg_layout.level[0].coef + i] = own;
    // A face beyond the box has no gas on its far side: 0, as at a vacuum face.
    mg_coef[fx] = select(0.0, near.x, gas);
    mg_coef[fy] = select(0.0, near.z, gas);
    if (id.x + 1u == abox.w) {
        mg_coef[fx + 1u] = 0.0;
    }
    if (id.y + 1u == abox.h) {
        mg_coef[fy + abox.w] = 0.0;
    }
}

@compute @workgroup_size(8, 8)
fn mg_coarsen(@builtin(global_invocation_id) id: vec3<u32>) {
    let l = mg_step.level;
    let fine = mg_dims(l);
    let d = mg_dims(l + 1u);
    if (l >= mg_span().y || id.x >= d.x || id.y >= d.y) {
        return;
    }
    var own = 0.0;
    var live = false;
    var fx = 0.0;
    var fy = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let x = 2u * id.x + (k & 1u);
        let y = 2u * id.y + (k >> 1u);
        if (x < fine.x && y < fine.y) {
            let o = mg_coef[mg_layout.level[l].coef + y * fine.x + x];
            if (o >= 0.0) {
                own += o;
                live = true;
            }
        }
    }
    // The −x face sums the fine faces at x = 2 id.x of both rows (0 at the level's edge), the −y face likewise.
    for (var k = 0u; k < 2u; k++) {
        if (2u * id.y + k < fine.y) {
            fx += mg_coef[mg_fx(l) + (2u * id.y + k) * (fine.x + 1u) + 2u * id.x];
        }
        if (2u * id.x + k < fine.x) {
            fy += mg_coef[mg_fy(l) + 2u * id.y * fine.x + 2u * id.x + k];
        }
    }
    let cx = mg_fx(l + 1u) + id.y * (d.x + 1u) + id.x;
    let cy = mg_fy(l + 1u) + id.y * d.x + id.x;
    mg_coef[mg_layout.level[l + 1u].coef + id.y * d.x + id.x] = select(-1.0, own, live);
    mg_coef[cx] = fx;
    mg_coef[cy] = fy;
    if (id.x + 1u == d.x) {
        mg_coef[cx + 1u] = 0.0;
    }
    if (id.y + 1u == d.y) {
        mg_coef[cy + d.x] = 0.0;
    }
}

@compute @workgroup_size(16, 16)
fn mg_smooth(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    let l = mg_step.level;
    let up = mg_step.up;
    let d = mg_dims(l);
    let cd = mg_dims(l + 1u);
    // Every invocation reaches every barrier: a tile past the level only does nothing.
    let live = l < mg_span().x && wid.x * MG_TILE < d.x && wid.y * MG_TILE < d.y;
    let coef = mg_layout.level[l].coef;
    // The tile and its halo at the dispatch's start: 0 going down, z + P z_coarse going up.
    for (var k = li; k < MG_HALO * MG_HALO; k += 256u) {
        let x = wid.x * MG_TILE + k % MG_HALO - 1u;
        let y = wid.y * MG_TILE + k / MG_HALO - 1u;
        var v = 0.0;
        if (live && up != 0u && x < d.x && y < d.y && mg_coef[coef + y * d.x + x] >= 0.0) {
            v = mg_vec[mg_z(l, 0u) + y * d.x + x] + mg_vec[mg_z(l + 1u, 1u) + (y / 2u) * cd.x + x / 2u];
        }
        mg_tile[k] = v;
    }
    workgroupBarrier();
    let x = wid.x * MG_TILE + lid.x;
    let y = wid.y * MG_TILE + lid.y;
    let i = y * d.x + x;
    let mine = live && x < d.x && y < d.y && mg_coef[coef + i] >= 0.0;
    var f = vec4<f32>(0.0);
    var b = 0.0;
    var diag = 1.0;
    if (mine) {
        f = mg_faces(l, d, x, y);
        b = mg_vec[mg_layout.level[l].vec + i];
        diag = mg_coef[coef + i] + f.x + f.y + f.z + f.w;
    }
    let t = (lid.y + 1u) * MG_HALO + lid.x + 1u;
    for (var s = 0u; s < 2u * MG_SWEEPS; s++) {
        if (mine && (x + y) % 2u == (s + up) % 2u) {
            let near = vec4<f32>(mg_tile[t - 1u], mg_tile[t + 1u], mg_tile[t - MG_HALO], mg_tile[t + MG_HALO]);
            mg_tile[t] = (b + dot(f, near)) / diag;
        }
        workgroupBarrier();
    }
    if (mine) {
        mg_vec[mg_z(l, up) + i] = mg_tile[t];
    }
}

@compute @workgroup_size(8, 8)
fn mg_restrict(@builtin(global_invocation_id) id: vec3<u32>) {
    let l = mg_step.level;
    let fine = mg_dims(l);
    let d = mg_dims(l + 1u);
    if (l >= mg_span().x || id.x >= d.x || id.y >= d.y) {
        return;
    }
    mg_vec[mg_layout.level[l + 1u].vec + id.y * d.x + id.x] = mg_gather(l, fine, id.x, id.y, mg_z(l, 0u));
}

// The coarse right side at aggregate (cx, cy): the residual b − A z of its active cells of level l, summed.
fn mg_gather(l: u32, fine: vec2<u32>, cx: u32, cy: u32, z: u32) -> f32 {
    var sum = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let x = 2u * cx + (k & 1u);
        let y = 2u * cy + (k >> 1u);
        if (x < fine.x && y < fine.y && mg_coef[mg_layout.level[l].coef + y * fine.x + x] >= 0.0) {
            sum += mg_residual(l, fine, x, y, z);
        }
    }
    return sum;
}

// One red-black half-sweep of colour `colour` over level l in z's up plane, by `mg_coarse`'s one workgroup.
fn mg_half(l: u32, li: u32, colour: u32) {
    let d = mg_dims(l);
    let z = mg_z(l, 1u);
    for (var i = li; i < d.x * d.y; i += 256u) {
        let x = i % d.x;
        let y = i / d.x;
        let own = mg_coef[mg_layout.level[l].coef + i];
        if ((x + y) % 2u == colour && own >= 0.0) {
            let f = mg_faces(l, d, x, y);
            mg_vec[z + i] = (mg_vec[mg_layout.level[l].vec + i] + dot(f, mg_near(z, d, i, f))) / (own + f.x + f.y + f.z + f.w);
        }
    }
    storageBarrier();
}

@compute @workgroup_size(256)
fn mg_coarse(@builtin(local_invocation_index) li: u32) {
    let first = mg_span().x;
    let last = mg_span().y;
    for (var l = first; l <= last; l++) {
        let d = mg_dims(l);
        for (var i = li; i < d.x * d.y; i += 256u) {
            mg_vec[mg_z(l, 1u) + i] = 0.0;
        }
        storageBarrier();
        if (l == last) {
            // The coarsest, a few cells: MG_SOLVE symmetric pairs, red, black, black, red.
            for (var s = 0u; s < 4u * MG_SOLVE; s++) {
                mg_half(l, li, ((s + 1u) / 2u) % 2u);
            }
            break;
        }
        for (var s = 0u; s < 2u * MG_SWEEPS; s++) {
            mg_half(l, li, s % 2u);
        }
        let cd = mg_dims(l + 1u);
        for (var i = li; i < cd.x * cd.y; i += 256u) {
            mg_vec[mg_layout.level[l + 1u].vec + i] = mg_gather(l, d, i % cd.x, i / cd.x, mg_z(l, 1u));
        }
        storageBarrier();
    }
    for (var l = last; l > first; l--) {
        let d = mg_dims(l - 1u);
        let cd = mg_dims(l);
        for (var i = li; i < d.x * d.y; i += 256u) {
            let x = i % d.x;
            let y = i / d.x;
            if (mg_coef[mg_layout.level[l - 1u].coef + i] >= 0.0) {
                mg_vec[mg_z(l - 1u, 1u) + i] += mg_vec[mg_z(l, 1u) + (y / 2u) * cd.x + x / 2u];
            }
        }
        storageBarrier();
        for (var s = 0u; s < 2u * MG_SWEEPS; s++) {
            mg_half(l - 1u, li, (s + 1u) % 2u);
        }
    }
}
