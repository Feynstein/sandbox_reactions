// P6 reactions (m0_contrat.md §1.3.2, §1.6, §2.5.1–§2.5.3, §1.8.5, §1.9.1; M0-T46, M0-T47): M0-T44b's and M0-T45's twin
// (sr_physics::reference::react) in f32. Compiled with eos.wgsl prepended (group 1).
//
// One invocation per box cell, the cell alone: each enabled registry record converts mass at ω = A Σ^a Π X_i^{n_i} f(T) g(Σ)
// (rates.rs — f from the record's 256-node ln f table, log-log, the Gamow-shaped direct form above its span, 0 below T_thr). The pass's
// Δt_n is sub-cycled by linearised backward Euler: S = Σ_r ν_r ω_r, J = Σ_r ν_r ∇ω_r (∇ω_r with X_j := 1, as the twin), a
// sub-step of h solves (I − hJ) ΔX = hS by Gauss with partial pivoting, the conversion each record made is
// Δξ_r = h (ω_r + ∇ω_r · ΔX) and its heat q_r Δξ_r enters the cell's energy and its group's accumulator every sub-step.
// h = min(time left, burn_dx / max|S_i|, burn_dt T / |dT/dt|, 2 × the last h); the step is checked (ΔX ≤ burn_dx, ΔT/T ≤
// burn_dt, no fraction below zero) and halved on failure; fractions clamped ≥ 0 and renormalised. A cell where every ω is 0
// is left bit for bit; a cell below Σ_vac, or a NaN Σ, is not touched. With `pin_temperature` the rates read the step's
// initial T and E is rewritten to that T with the new composition's μ and cold energy.
//
// f32 where the twin is f64. Four sums are compensated (Knuth's two-sum): the time done, so a sub-step far under f32's
// resolution of Δt_n still counts (a runaway's sub-steps reach 1e-9 Δt_n); each fraction, rounded to f32 once when it is
// written back, and its renormalisation, by the compensated excess of the sum over 1 (a fraction rounded every sub-step
// drifts 1.4e-5 over G-BURN's 9,000, measured); and the heat — the cell's energy and each group's accumulator. The rates
// read the fractions' rounded parts. Two guards the twin does not need: the rate scale (He_burn's Gamow-shaped f reaches
// e^95 in G-REF's runaway at T ≈ 18,800 and is bounded by e^112, past f32's e^88), and a cell stops after MAX_ATTEMPTS
// attempts with its time left unburned, counted `unfinished`. The scale (M0-D34, M0-D35): where a burning record's ln f
// (every input present) passes LN_F_MAX, every ω, ∇ω, S, Σ qω and ε_ν of the sub-step is taken e^−shift times the true one,
// shift the least that brings the largest back to LN_F_MAX (at most 112 − 60 = 52 under the Gamow law, so e^shift and the
// scaled time stay inside f32), and h runs in that scaled time (the true h is h e^−shift, the time left t_rem e^shift) —
// the ratios between records, so the branching and the heat, stay the twin's; capping each f at LN_F_MAX instead booked
// G-REF's runaway 0.9 % hot under the power law (M0-D34). A record live by its gradient alone (one input of order 1 absent)
// is held to e^LN_F_LIVE in that scale: past it h|∂ω/∂X| ≫ 1, backward Euler's limit (it takes all of the absent input the
// sub-step makes), so its exact size no longer moves ΔX. Where h|J| would pass HJ_MAX the system is solved divided by h
// (`solve`); the h carried from the last sub-step is moved to this one's scale and floored at f32's smallest normal, as the
// twin floors at f64's. The checks' margins are f32's: a fraction below −max(1e-12, 2.4e-7 |ΔX|) is negative (a flat −1e-7
// retried less than the twin near a species' end, M0-D34), a limit is passed beyond 1e-5 of itself; the half-size retry
// stops at the twin's MIN_H · Δt_n.
//
// Neutrinos (M0-T47, the scene's `neutrinos` switch): ε_ν = A_ν (T/T_ν)^m per unit mass for T ≥ T_ν is read at each
// sub-step's start T, taken from the cell's energy (E −= ε_ν h, with the nuclear heat), carried in dT/dt (−μ ε_ν) and added
// to the booking's `neutrino_lost` as Σ Σ ε_ν h (a compensated sum); a cell with every ω = 0 and ε_ν = 0 is still left
// bit for bit. Where the cell leaves the pass with X_n ≥ X_N_STAR its S_ν = Σ ε_ν h summed over Δt_n (the pass's mean
// power per area) goes to side.s_nu, 0 elsewhere (the vacuum and the cells P6 skips included) — written only with the
// switch on, the plane being zero from creation. The collapse latch (§1.8.5): a cell whose N_Fe rate (a record the host
// flags LATCHES) is > 0 at the start of any sub-step ORs 1 into report.latch; `react_latch`, one invocation after the
// cells, ORs it into word 0 of the latch record (Step::latch_flag) — a ninth binding cannot be added to `react_cells`
// (7 storage buffers here + the EOS table = WebGPU's 8).
//
// `react_begin` clears the step's counters; `react_cells` burns and adds to the counters with u32 atomics (order-free, never
// a float, §1.3.4) — the state and the booking each have one writer invocation per slot.

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

// One enabled record (step/react.rs packs it).
struct Record {
    // Its group (GROUPS' index), how many inputs and terms, its flags (HAS_TERMS, GATED).
    head: vec4<u32>,
    // Each input's species id and order n_i.
    species: vec4<u32>,
    orders: vec4<f32>,
    // A (a_coef), a (Σ's exponent), q = q_ratio Q_H, T_thr.
    coef: vec4<f32>,
    // Σ_g and p of the gate.
    gate: vec4<f32>,
    // (T_k, ν_k, w_k, 0).
    terms: array<vec4<f32>, 4>,
    // Output shares minus input shares, by species.
    nu: array<vec4<f32>, 3>,
}

struct ReactParams {
    n_recs: u32,
    flags: u32,
    burn_dx: f32,
    burn_dt: f32,
    // 1 / Δ ln T of the tables, and the last node's index.
    inv_step: f32,
    last_node: f32,
    // 0, unknown to the compiler: `opaque` XORs a value's bits with it, so `add2`'s error terms are not folded away
    // (measured: on the RTX 5090 they were, even multiplied by a uniform 1.0).
    zero: u32,
    _a: u32,
    // ε_ν = a_nu (T / t_nu)^m_nu for T ≥ t_nu (physics.json's A_ν, T_ν, m).
    a_nu: f32,
    t_nu: f32,
    m_nu: f32,
    _b: u32,
    recs: array<Record, 16>,
    // ln f of record r at node i: entry 256 r + i, four to a vec4.
    ln_f: array<vec4<f32>, 1024>,
}

// The step's counters (the first four, cleared by `react_begin`) and the run's (since the step was built).
struct ReactReport {
    substeps: atomic<u32>,
    retried: atomic<u32>,
    max_substeps: atomic<u32>,
    unfinished: atomic<u32>,
    worst: atomic<u32>,
    unfinished_run: atomic<u32>,
    // 1 when any cell's N_Fe rate was > 0 at a sub-step's start this step (cleared by `react_begin`).
    latch: atomic<u32>,
    _b: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The hydro planes (sigma, mom_x, mom_y, energy), then the species planes x_H … x_Ne and x_Mg … x_n, row-major.
@group(0) @binding(1) var<storage, read_write> hydro: array<f32>;
@group(0) @binding(2) var<storage, read_write> species_a: array<f32>;
@group(0) @binding(3) var<storage, read_write> species_b: array<f32>;
// booking.reactions (state.rs BOOK_GROUPS): nuclear.H … nuclear.N, then neutrino_lost, one slot per cell.
@group(0) @binding(4) var<storage, read_write> book: array<f32>;
@group(0) @binding(5) var<uniform> abox: ActiveBox;
// The Δt record (reduce.wgsl `DtState`): Δt_n is its first word.
@group(0) @binding(6) var<storage, read> dt_rec: array<f32>;
@group(0) @binding(7) var<uniform> params: ReactParams;
@group(0) @binding(8) var<storage, read_write> report: ReactReport;
// The latch record's first word, the flag (latch.wgsl `LatchState`), used by `react_latch` alone.
@group(0) @binding(9) var<storage, read_write> latch_flag: array<atomic<u32>>;
// side.s_nu: the neutrino source S_ν, one plane (§2.2).
@group(0) @binding(10) var<storage, read_write> s_nu: array<f32>;

const MAX_INPUTS: u32 = 4u;
const N_GROUPS: u32 = 7u;
const TABLE_POINTS: u32 = 256u;
const PIN_TEMPERATURE: u32 = 1u;
const HAS_TERMS: u32 = 1u;
const GATED: u32 = 2u;
const LATCHES: u32 = 4u;
const NEUTRINOS: u32 = 2u;
// §2.5.3: where a cell keeps S_ν — the newborn neutron star.
const X_N_STAR: f32 = 0.5;
const MAX_ATTEMPTS: u32 = 65536u;
const MIN_H: f32 = 1e-14;
const LN_F_MAX: f32 = 60.0;
const MIN_POSITIVE: f32 = 1.17549435e-38;
const HJ_MAX: f32 = 1e30;
const LN_F_LIVE: f32 = 80.0;
// A fraction is negative below −max(1e-12, 4 f32 ulps of its ΔX): the twin's 1e-12 where the solve's own rounding allows.
const NEGATIVE: f32 = 1e-12;
const NEGATIVE_DX: f32 = 2.4e-7;
const MARGIN: f32 = 1e-5;

// The cell's fractions, the rates and their gradients at the sub-step's start (∂ω_r/∂X of input k at 4r + k), S, the
// system (I − hJ | hS) and its solution.
var<private> cx: array<f32, 10>;
var<private> cx_lo: array<f32, 10>;
var<private> omega: array<f32, 16>;
var<private> grad: array<f32, 64>;
var<private> src: array<f32, 10>;
var<private> mat: array<array<f32, 11>, 10>;
var<private> dx: array<f32, 10>;
// Whether a latching record's rate was > 0 at any sub-step's start in this cell.
var<private> latched: bool;
// The sub-step's rate scale (header): ≥ 0, and 0 unless a live record's ln f passes LN_F_MAX.
var<private> shift: f32;
// Per record, at the sub-step's start: ln f (the largest term's above the table's span), how f is read (F_* below),
// A Σ^a, g, and whether its ω or its gradient can be nonzero (`live`, the records the scale is taken over).
var<private> rec_ln_f: array<f32, 16>;
var<private> rec_kind: array<u32, 16>;
var<private> rec_a_sigma: array<f32, 16>;
var<private> rec_g: array<f32, 16>;
var<private> rec_live: array<bool, 16>;
var<private> rec_whole: array<bool, 16>;

// x^n for x ≥ 0, as f64's powf: 1 at n = 0 (0⁰ included), 0 at x = 0 otherwise; a small whole n by products.
fn powr(x: f32, n: f32) -> f32 {
    if (n == 0.0) {
        return 1.0;
    }
    if (x <= 0.0) {
        return 0.0;
    }
    if (n == floor(n) && n > 0.0 && n <= 8.0) {
        var v = x;
        for (var k = 1u; k < u32(n); k++) {
            v *= x;
        }
        return v;
    }
    return exp2(n * log2(x));
}

fn nu_of(r: u32, i: u32) -> f32 {
    return params.recs[r].nu[i / 4u][i % 4u];
}

fn ln_f_at(at: u32) -> f32 {
    return params.ln_f[at / 4u][at % 4u];
}

// How record r's f(T) is read (rates.rs `Omega::f`): 0 below T_thr, 1 without terms, the table inside its span, the direct
// form above.
const F_ZERO: u32 = 0u;
const F_ONE: u32 = 1u;
const F_TABLE: u32 = 2u;
const F_DIRECT: u32 = 3u;

// One Gamow-shaped term of the direct form (rates.rs `term_ln`, §0.5 [M0-D34]): ln(w (T/T_k)^{−2/3} exp(τ (1 −
// (T_k/T)^{1/3}))) = ln w − (2/3) u + τ (1 − e^{−u/3}), u = ln(T/T_k), τ = 3ν + 2.
fn term_ln(r: u32, k: u32, t: f32) -> f32 {
    let term = params.recs[r].terms[k];
    let u = log(t / term.x);
    return log(term.z) - u * (2.0 / 3.0) + (3.0 * term.y + 2.0) * (1.0 - exp(-u / 3.0));
}

// Record r's f(T) read, as (ln f, kind): ln f is the table's, the largest term's in the direct form, 0 for F_ONE and F_ZERO.
fn rate_ln(r: u32, t: f32) -> vec2<f32> {
    let head = params.recs[r].head;
    if ((head.w & HAS_TERMS) == 0u) {
        return vec2<f32>(0.0, f32(F_ONE));
    }
    let t_thr = params.recs[r].coef.w;
    if (t < t_thr) {
        return vec2<f32>(0.0, f32(F_ZERO));
    }
    let pos = log(t / t_thr) * params.inv_step;
    if (pos >= params.last_node) {
        var top = term_ln(r, 0u, t);
        for (var k = 1u; k < head.z; k++) {
            top = max(top, term_ln(r, k, t));
        }
        return vec2<f32>(top, f32(F_DIRECT));
    }
    let i = min(u32(pos), TABLE_POINTS - 2u);
    let a = ln_f_at(r * TABLE_POINTS + i);
    let b = ln_f_at(r * TABLE_POINTS + i + 1u);
    return vec2<f32>(a + (pos - f32(i)) * (b - a), f32(F_TABLE));
}

// Record r's f(T) e^−shift, from what `rates` read of it, at most e^LN_F_LIVE (a record live by its gradient alone, header).
fn rate_f_scaled(r: u32, t: f32) -> f32 {
    switch (rec_kind[r]) {
        case F_ONE: {
            return select(exp(-shift), 1.0, shift == 0.0);
        }
        case F_TABLE: {
            return exp(min(rec_ln_f[r] - shift, LN_F_LIVE));
        }
        case F_DIRECT: {
            var f = 0.0;
            for (var k = 0u; k < params.recs[r].head.z; k++) {
                f += exp(min(term_ln(r, k, t) - shift, LN_F_LIVE));
            }
            return f;
        }
        default: {
            return 0.0;
        }
    }
}

// g of record r at Y_e Σ: 1 ungated, ((Y_e Σ)/Σ_g − 1)^p above Σ_g, else 0.
fn rate_g(r: u32, y_e_sigma: f32) -> f32 {
    if ((params.recs[r].head.w & GATED) == 0u) {
        return 1.0;
    }
    let gate = params.recs[r].gate;
    if (y_e_sigma > gate.x) {
        return powr(y_e_sigma / gate.x - 1.0, gate.y);
    }
    return 0.0;
}

// ε_ν at T (§2.5.3): A_ν (T/T_ν)^m per unit mass from T_ν up, 0 below it and with the switch off.
fn eps_nu(t: f32) -> f32 {
    if ((params.flags & NEUTRINOS) == 0u || t < params.t_nu) {
        return 0.0;
    }
    return params.a_nu * powr(t / params.t_nu, params.m_nu);
}

// ω_r, ∇ω_r and S at (Σ, `cx`, T, Y_e), each e^−shift times the true one (`shift` set here, header); returns Σ_r q_r ω_r
// so scaled, and whether any true ω is nonzero.
fn rates(sigma: f32, t: f32, y_e: f32) -> vec2<f32> {
    for (var i = 0u; i < N_SPECIES; i++) {
        src[i] = 0.0;
    }
    // The scale, over the burning records (every input present). Live: ω, or its derivative in one absent input of order 1,
    // can be nonzero — a record live by its gradient alone is not in the scale, and is held to e^LN_F_LIVE (header).
    var top = LN_F_MAX;
    for (var r = 0u; r < params.n_recs; r++) {
        let rec = params.recs[r];
        let read = rate_ln(r, t);
        rec_ln_f[r] = read.x;
        rec_kind[r] = u32(read.y);
        rec_a_sigma[r] = rec.coef.x * powr(sigma, rec.coef.y);
        rec_g[r] = rate_g(r, y_e * sigma);
        var absent = 0u;
        var stuck = false;
        for (var k = 0u; k < rec.head.y; k++) {
            if (rec.orders[k] != 0.0 && !(cx[rec.species[k]] > 0.0)) {
                absent++;
                stuck = stuck || rec.orders[k] != 1.0;
            }
        }
        rec_whole[r] = absent == 0u;
        rec_live[r] = rec_kind[r] != F_ZERO && rec_a_sigma[r] * rec_g[r] != 0.0 && (absent == 0u || (absent == 1u && !stuck));
        if (rec_live[r] && rec_whole[r]) {
            top = max(top, read.x);
        }
    }
    shift = top - LN_F_MAX;
    var qdot = 0.0;
    var lit = 0.0;
    for (var r = 0u; r < params.n_recs; r++) {
        let rec = params.recs[r];
        let base = rec_a_sigma[r] * select(0.0, rate_f_scaled(r, t), rec_live[r]) * rec_g[r];
        var prod = base;
        for (var k = 0u; k < rec.head.y; k++) {
            prod *= powr(cx[rec.species[k]], rec.orders[k]);
        }
        omega[r] = prod;
        if (prod != 0.0) {
            lit = 1.0;
        }
        // A latching ω the scale took under f32's range is still > 0.
        if ((prod > 0.0 || (prod == 0.0 && shift > 0.0 && rec_live[r] && rec_whole[r])) && (rec.head.w & LATCHES) != 0u) {
            latched = true;
        }
        for (var k = 0u; k < MAX_INPUTS; k++) {
            grad[4u * r + k] = 0.0;
            if (k >= rec.head.y || rec.orders[k] == 0.0) {
                continue;
            }
            // n x_j^(n−1) ω(X with X_j := 1).
            let j = rec.species[k];
            var g = rec.orders[k] * powr(cx[j], rec.orders[k] - 1.0) * base;
            for (var l = 0u; l < rec.head.y; l++) {
                if (rec.species[l] != j) {
                    g *= powr(cx[rec.species[l]], rec.orders[l]);
                }
            }
            grad[4u * r + k] = g;
        }
        for (var i = 0u; i < N_SPECIES; i++) {
            src[i] += nu_of(r, i) * prod;
        }
        qdot += rec.coef.z * prod;
    }
    return vec2<f32>(qdot, lit);
}

// ΔX into `dx` with (I − hJ) ΔX = hS, J = Σ_r ν_r ∇ω_r: Gauss with partial pivoting (the twin's `solve`). Where h|J| would
// pass HJ_MAX — a scaled sub-step whose h is set by a small S beside a record live by its gradient alone (an input absent),
// h|J| reaching 10⁵³ in G-REF's runaway, which f64 holds and f32 does not — the same system divided by h,
// (I/h − J) ΔX = S, whose entries all stay in range (M0-D34).
fn solve(h: f32) {
    var j_max = 0.0;
    for (var r = 0u; r < params.n_recs; r++) {
        for (var k = 0u; k < params.recs[r].head.y; k++) {
            j_max = max(j_max, abs(grad[4u * r + k]));
        }
    }
    let divided = !(h * j_max <= HJ_MAX);
    let diag = select(1.0, 1.0 / h, divided);
    let hs = select(h, 1.0, divided);
    for (var i = 0u; i < N_SPECIES; i++) {
        for (var j = 0u; j < N_SPECIES; j++) {
            mat[i][j] = select(0.0, diag, i == j);
        }
        mat[i][N_SPECIES] = hs * src[i];
    }
    for (var r = 0u; r < params.n_recs; r++) {
        for (var k = 0u; k < params.recs[r].head.y; k++) {
            let g = grad[4u * r + k];
            if (g == 0.0) {
                continue;
            }
            let j = params.recs[r].species[k];
            for (var i = 0u; i < N_SPECIES; i++) {
                mat[i][j] -= hs * nu_of(r, i) * g;
            }
        }
    }
    for (var col = 0u; col < N_SPECIES; col++) {
        // The last of equal pivots, as Rust's max_by.
        var piv = col;
        for (var p = col + 1u; p < N_SPECIES; p++) {
            if (abs(mat[p][col]) >= abs(mat[piv][col])) {
                piv = p;
            }
        }
        if (piv != col) {
            let row = mat[col];
            mat[col] = mat[piv];
            mat[piv] = row;
        }
        let d = mat[col][col];
        if (d == 0.0) {
            continue;
        }
        for (var row = col + 1u; row < N_SPECIES; row++) {
            let f = mat[row][col] / d;
            if (f != 0.0) {
                for (var c = col; c <= N_SPECIES; c++) {
                    mat[row][c] -= f * mat[col][c];
                }
            }
        }
    }
    for (var ii = 0u; ii < N_SPECIES; ii++) {
        let i = N_SPECIES - 1u - ii;
        var v = mat[i][N_SPECIES];
        for (var j = i + 1u; j < N_SPECIES; j++) {
            v -= mat[i][j] * dx[j];
        }
        dx[i] = select(v / mat[i][i], 0.0, mat[i][i] == 0.0);
    }
}

// (sum, error) + v, Knuth's two-sum: the error term carries what f32 rounded off.
fn add2(acc: vec2<f32>, v: f32) -> vec2<f32> {
    let s = opaque(acc.x + v);
    let bb = opaque(s - acc.x);
    let err = (acc.x - (s - bb)) + (v - bb);
    let t = acc.y + err;
    let hi = opaque(s + t);
    return vec2<f32>(hi, t - (hi - s));
}

// `v` itself, through bits the compiler cannot see.
fn opaque(v: f32) -> f32 {
    return bitcast<f32>(bitcast<u32>(v) ^ params.zero);
}

// T from the specific thermal-plus-cold energy, ε_th floored at T_floor/μ for the rates.
fn temp_of(sigma: f32, e_tot: f32, comp: Composition, x_n: f32) -> f32 {
    return temperature(max(e_tot - eps_cold(sigma, comp.y_e, x_n), eps_th_floor(comp.inv_mu)), comp.inv_mu);
}

@compute @workgroup_size(1)
fn react_begin() {
    atomicStore(&report.substeps, 0u);
    atomicStore(&report.retried, 0u);
    atomicStore(&report.max_substeps, 0u);
    atomicStore(&report.unfinished, 0u);
    atomicStore(&report.latch, 0u);
}

@compute @workgroup_size(1)
fn react_latch() {
    if (atomicLoad(&report.latch) != 0u) {
        atomicOr(&latch_flag[0], 1u);
    }
}

@compute @workgroup_size(8, 8)
fn react_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
    let n = dims.cells;
    let sigma = hydro[cell];
    let nu_on = (params.flags & NEUTRINOS) != 0u;
    if (!(sigma >= eos.sigma_vac)) {
        if (nu_on) {
            s_nu[cell] = 0.0;
        }
        return;
    }
    latched = false;
    for (var i = 0u; i < 5u; i++) {
        cx[i] = species_a[i * n + cell];
        cx[5u + i] = species_b[i * n + cell];
        cx_lo[i] = 0.0;
        cx_lo[5u + i] = 0.0;
    }
    let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
    let energy = hydro[3u * n + cell];
    let kinetic = 0.5 * dot(mom, mom) / (sigma * sigma);
    var e_tot = energy / sigma - kinetic;
    let dt = dt_rec[0];
    let pin = (params.flags & PIN_TEMPERATURE) != 0u;
    let t0 = temp_of(sigma, e_tot, composition(cx), cx[ID_N]);

    var booked = array<vec2<f32>, 7>();
    var de_sum = vec2<f32>(0.0, 0.0);
    var lost_sum = vec2<f32>(0.0, 0.0);
    var t_done = vec2<f32>(0.0, 0.0);
    // The last sub-step's 2h, in the scaled time of its own shift.
    var h_try = dt;
    var shift_try = 0.0;
    var attempts = 0u;
    var substeps = 0u;
    var retried = 0u;
    var unfinished = false;
    loop {
        let t_rem = (dt - t_done.x) - t_done.y;
        if (!(t_rem > 0.0)) {
            break;
        }
        let comp = composition(cx);
        var t = t0;
        if (!pin) {
            t = temp_of(sigma, e_tot, comp, cx[ID_N]);
        }
        let rate = rates(sigma, t, comp.y_e);
        let eps_true = eps_nu(t);
        if (rate.y == 0.0 && eps_true == 0.0) {
            break;
        }
        // e^±shift: 1 unscaled, e^shift +inf past f32's range (the time left then bounds nothing, as in the twin's f64).
        let up = select(exp(shift), 1.0, shift == 0.0);
        let down = select(exp(-shift), 1.0, shift == 0.0);
        let eps = eps_true * down;
        // The sub-step's size, in scaled time.
        var s_max = 0.0;
        for (var i = 0u; i < N_SPECIES; i++) {
            s_max = max(s_max, abs(src[i]));
        }
        var h = t_rem * up;
        if (s_max > 0.0) {
            h = min(h, params.burn_dx / s_max);
        }
        if (!pin) {
            var dinv_mu = 0.0;
            for (var i = 0u; i < N_SPECIES; i++) {
                dinv_mu += src[i] * eos.inv_mu_coef[i / 4u][i % 4u];
            }
            let mu = 1.0 / comp.inv_mu;
            let dt_dt = mu * (rate.x - eps) - t * mu * dinv_mu;
            if (dt_dt != 0.0) {
                h = min(h, params.burn_dt * t / abs(dt_dt));
            }
        }
        // The last 2h carried to this sub-step's scale, floored as the twin's (f64::MIN_POSITIVE there, f32's here).
        h = min(h, max(h_try * select(exp(shift - shift_try), 1.0, shift == shift_try), MIN_POSITIVE));
        let t_rem_scaled = t_rem * up;
        // Take it, check it, halve until it holds.
        loop {
            attempts++;
            if (attempts > MAX_ATTEMPTS) {
                unfinished = true;
                break;
            }
            let last = h >= t_rem_scaled;
            let h_use = min(h, t_rem_scaled);
            solve(h_use);
            var x_new = array<f32, 10>();
            var x_lo = array<f32, 10>();
            var worst = 0.0;
            var negative = false;
            for (var i = 0u; i < N_SPECIES; i++) {
                let v = add2(vec2<f32>(cx[i], cx_lo[i]), dx[i]);
                x_new[i] = v.x;
                x_lo[i] = v.y;
                worst = max(worst, abs(dx[i]));
                negative = negative || v.x + v.y < -max(NEGATIVE, NEGATIVE_DX * abs(dx[i]));
            }
            // Heat of the mass the linearisation moved, by record.
            var de = 0.0;
            var by_group = array<f32, 7>();
            for (var r = 0u; r < params.n_recs; r++) {
                var lin = omega[r];
                for (var k = 0u; k < params.recs[r].head.y; k++) {
                    lin += grad[4u * r + k] * dx[params.recs[r].species[k]];
                }
                let heat = params.recs[r].coef.z * h_use * lin;
                de += heat;
                by_group[params.recs[r].head.x] += heat;
            }
            let lost = eps * h_use;
            // Clamped ≥ 0, then x / Σx = x − x e/(1 + e), e = Σx − 1 from the compensated sum.
            var total = vec2<f32>(0.0, 0.0);
            for (var i = 0u; i < N_SPECIES; i++) {
                if (x_new[i] + x_lo[i] < 0.0) {
                    x_new[i] = 0.0;
                    x_lo[i] = 0.0;
                }
                total = add2(add2(total, x_new[i]), x_lo[i]);
            }
            let excess = (total.x - 1.0) + total.y;
            for (var i = 0u; i < N_SPECIES; i++) {
                let v = add2(vec2<f32>(x_new[i], x_lo[i]), -(x_new[i] + x_lo[i]) * excess / (1.0 + excess));
                x_new[i] = v.x;
                x_lo[i] = v.y;
            }
            var e_new = e_tot;
            var dt_rel = 0.0;
            if (!pin) {
                e_new = e_tot + de - lost;
                dt_rel = abs(temp_of(sigma, e_new, composition(x_new), x_new[ID_N]) - t) / t;
            }
            let bad = worst > params.burn_dx * (1.0 + MARGIN) || dt_rel > params.burn_dt * (1.0 + MARGIN) || negative;
            if (bad && h_use > MIN_H * dt * up) {
                h = 0.5 * h_use;
                retried++;
                continue;
            }
            cx = x_new;
            cx_lo = x_lo;
            e_tot = e_new;
            de_sum = add2(de_sum, de);
            lost_sum = add2(lost_sum, lost);
            for (var g = 0u; g < N_GROUPS; g++) {
                booked[g] = add2(booked[g], by_group[g]);
            }
            substeps++;
            if (last) {
                t_done = vec2<f32>(dt, 0.0);
            } else {
                t_done = add2(t_done, h_use * down);
            }
            h_try = 2.0 * h_use;
            shift_try = shift;
            break;
        }
        if (unfinished) {
            break;
        }
    }
    if (attempts > 0u) {
        atomicAdd(&report.substeps, substeps);
        atomicAdd(&report.retried, retried);
        atomicMax(&report.max_substeps, substeps);
        atomicMax(&report.worst, substeps);
        if (unfinished) {
            atomicAdd(&report.unfinished, 1u);
            atomicAdd(&report.unfinished_run, 1u);
        }
    }
    if (latched) {
        atomicOr(&report.latch, 1u);
    }
    if (substeps == 0u) {
        if (nu_on) {
            s_nu[cell] = 0.0;
        }
        return;
    }
    for (var i = 0u; i < N_SPECIES; i++) {
        cx[i] = cx[i] + cx_lo[i];
    }
    for (var i = 0u; i < 5u; i++) {
        species_a[i * n + cell] = cx[i];
        species_b[i * n + cell] = cx[5u + i];
    }
    if (pin) {
        let comp = composition(cx);
        hydro[3u * n + cell] = sigma * (t0 * comp.inv_mu + eps_cold(sigma, comp.y_e, cx[ID_N]) + kinetic);
    } else {
        hydro[3u * n + cell] = energy + sigma * ((de_sum.x + de_sum.y) - (lost_sum.x + lost_sum.y));
    }
    for (var g = 0u; g < N_GROUPS; g++) {
        book[g * n + cell] += sigma * (booked[g].x + booked[g].y);
    }
    let lost_all = lost_sum.x + lost_sum.y;
    book[N_GROUPS * n + cell] += sigma * lost_all;
    if (nu_on) {
        s_nu[cell] = select(0.0, sigma * lost_all / dt, cx[ID_N] >= X_N_STAR && dt > 0.0);
    }
}
