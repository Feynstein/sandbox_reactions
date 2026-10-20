// P4 hydro (m0_contrat.md §1.3.2, §2.3.3–§2.3.5, §2.8): M0-T33b's twin (sr_physics::reference::hydro) in f32, three
// dispatches — `hydro_update` over the active box's 8 × 8 tiles, `hydro_edges` over the world's boundary faces, then
// `hydro_commit` over the box's tiles. Compiled with eos.wgsl prepended (group 1).
//
// `hydro_update`, one cell per invocation: finite volume, unsplit, second order — MUSCL-Hancock with the monotonised-
// central limiter on the primitives (Σ, u, v, Π, X_i), the species slopes sharing one limiter factor so ΣX_i = 1 holds;
// the Hancock predictor advances a cell's four face states by half a step with both axes' flux differences; HLL fluxes
// with Davis speeds (c² = 2Π/Σ) join the predicted states across each face, the species riding the mass flux's upwind
// side. One workgroup per 8 × 8 tile (M0-D30): the tile's 12 × 12 primitives are read once into workgroup memory, each
// predictor runs once — the tile's 64 and the 32 of the ring around it whose faces the tile's faces need — and the
// predicted faces cross between invocations through the same memory, one axis at a time; no face state is stored in a
// buffer, so the dispatch needs no scratch beyond the next state. The update writes the next state, never the state it
// reads — a neighbour's predictor must see the cells as the step found them — and `hydro_commit` copies it back.
//
// Edges (§2.8), the mode a uniform `set_edge_mode` writes between submissions (M0-T35). *bounce*: a cell outside the
// world reads as its mirror inside, its normal velocity negated; the wall face's mass, energy and species fluxes are
// exactly 0 and its normal-momentum flux is the predicted face state's Π. *leave*: a cell outside the world reads as the
// edge cell, its normal velocity clamped outward; the boundary face's flux is HLL between the edge cell's predicted face
// and the ghost's, and a face whose mass flux points inward is a wall instead (inflow exactly 0). `hydro_edges` books
// what crosses each boundary face into `escaped` — the same `boundary` flux on the same state, so the grid's loss is
// the booking; bounce books nothing.
//
// Gravity's source (§1.4.2; M0-T36), when the step runs P2 (`edges.gravity`): `cell_faces`' Hancock predictor kicks each
// face state by half a step of the cell's g — velocity by ½Δt·g, energy by the work Σ(u + ¼Δt·g)·g·½Δt, leaving its
// thermal energy as it was — beside the flux differences; `hydro_update` adds Δt·Σg to the momentum, Σ the cell centre's
// half-step value from that same predictor. The energy source is the work the step's mass fluxes do across φ (§0.5
// [M0-D64]): a face whose mass flux q joins the cell i to its neighbour j does Δt·q·(φ_i − φ_j), i taking
// (½ p_i + w_i (q − ½(p_i + p_j)))·(φ_i − φ_j)·Δt of it (`grav_share`) — p the two cells' half-step momenta on that axis,
// w_i = Σ_i/(Σ_i + Σ_j) their half-step masses' share — so each cell keeps the Δt·(Σu)·g its own momentum pays for and the
// mass HLL's diffusion moves is paid by the two cells' masses; the neighbours' half-step centres cross through `mids`.
// Summed, the energy gained is what the moved mass does to W at the step's φ. A ghost's g and φ are its edge cell's, so
// no work is done across the world's edge. `hydro_edges` recomputes the same predictor, so it binds g too. The twin's
// `Hydro::step_with` is the reference.
//
// The radiation force (§2.4.4; M0-T43), when the step runs P5 (`edges.radiation`): `hydro_update` adds Δt·f to a gas
// cell's momentum, f = κΣF/c_sb from the cell as the step found it and F as P5 left it in `side.flux` at the end of the
// last step — the previous step's flux, since P5 runs after P4 —, after the flux differences and gravity's source; E is
// left as it is (the work comes out of the thermal energy), capped at the heat the cell holds above its floor (§0.5
// [M0-D58], `push_share`). The predictor is not kicked and the boundary fluxes do not see it, so `hydro_edges` binds no F.
// The twin's `Hydro::step_forced` is the reference.
//
// A cell below Σ_vac reads as the floor state (Σ_floor, at rest, T_floor), as P9's rates do (M0-T20) — the box's ghost
// ring included, where an uploaded vacuum cell may hold Σ = 0; a cell and its four neighbours all below Σ_vac are left
// as they are. A face state's Σ is held between its cell's and the neighbour's on that side, as MC keeps it in exact
// arithmetic — in f32 it rounds to 0 beside such a cell (`bracketed`, M0-D44). Δt is the Δt record's first word (P1). No atomic, no cross-invocation order: the same state gives the
// same next state, bit for bit (§1.3.4).

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

@group(0) @binding(0) var<uniform> dims: Dims;
// The state as the step found it: the hydro planes (sigma, mom_x, mom_y, energy), then the species planes (x_H … x_Ne,
// x_Mg … x_n), row-major.
@group(0) @binding(1) var<storage, read> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
// The next state: the state's 14 planes in §2.2's order, the first 7 in `next_lo`, the last 7 in `next_hi` (step/hydro.rs)
// — two buffers, so the update keeps a storage slot for g within WebGPU's 8.
@group(0) @binding(4) var<storage, read_write> next_lo: array<f32>;
@group(0) @binding(5) var<storage, read_write> next_hi: array<f32>;
// P5's flux F (state.rs `side.flux`): the x plane, then the y plane, cell-centred, row-major — read only when
// `edges.radiation` is 1 (else a placeholder is bound). The update's eighth storage buffer.
@group(0) @binding(6) var<storage, read> rad_flux: array<f32>;
@group(0) @binding(7) var<uniform> abox: ActiveBox;
// The Δt record (reduce.wgsl `DtState`), its first four words as a uniform — so the update keeps a storage slot for F
// (M0-T43): word 0 is Δt_n.
@group(0) @binding(8) var<uniform> dt_rec: vec4<f32>;
// `hydro_commit`'s writable view of the state.
@group(0) @binding(9) var<storage, read_write> out_hydro: array<f32>;
@group(0) @binding(10) var<storage, read_write> out_a: array<f32>;
@group(0) @binding(11) var<storage, read_write> out_b: array<f32>;
// The booking's `escaped` buffer (state.rs `booking.escaped`): 14 planes — mass, mom_x, mom_y, energy, each species'
// mass — of 2 (width + height) boundary-face slots each: x = 0 by y, x = width − 1 by y, y = 0 by x, y = height − 1 by x.
@group(0) @binding(12) var<storage, read_write> escaped: array<f32>;

// The edge mode: `leave` 1, bounce 0; `gravity` 1 when the step runs P2 (`grav` is then its g); `radiation` 1 when it
// runs P5 (`rad_flux` is then its F); then the radiation force's constants (physics.json, §2.4.2).
struct Edges {
    leave: u32,
    gravity: u32,
    radiation: u32,
    _c: u32,
    kappa0: f32,
    c_sb: f32,
    kappa_dust: f32,
    t_dust: f32,
}
@group(0) @binding(13) var<uniform> edges: Edges;
// P2's g = −∇φ (step/grav.rs `field`): the x plane, the y plane, then φ's, row-major over the world — read only when
// `edges.gravity` is 1 (else a placeholder is bound).
@group(0) @binding(14) var<storage, read> grav: array<f32>;

const PLANES_A: u32 = 5u;
const PLANES_B: u32 = 5u;
// The planes `next_lo` holds.
const NEXT_LO: u32 = 7u;
// Conserved components per cell: Σ, Σu, Σv, E, then Σ X_i.
const COMPONENTS: u32 = 14u;

// A cell's primitives: Σ, u, v, the total pressure Π and the mass fractions.
struct Prim {
    sigma: f32,
    u: f32,
    v: f32,
    pi: f32,
    x: array<f32, 10>,
}

// A predicted face state: its primitives and its conserved (Σ, Σu, Σv, E).
struct Face {
    p: Prim,
    q: vec4<f32>,
}

// A cell's predictor: its four face states, [x−, x+, y−, y+], and its centre's (Σ, Σu, Σv) half a step on.
struct Predicted {
    faces: array<Face, 4>,
    mid: vec3<f32>,
}

// An Inf or a NaN, told by its exponent bits (floors.wgsl) — never a float comparison, which WGSL may fold away.
fn non_finite(v: f32) -> bool {
    return (bitcast<u32>(v) & 0x7f800000u) == 0x7f800000u;
}

// Cell `cell` as primitives: the floor state below Σ_vac, else ε_th from E floored at T_floor/μ (§2.3.3).
fn read_cell(cell: u32) -> Prim {
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
    let sigma = hydro[cell];
    if (sigma < eos.sigma_vac) {
        let s = eos.sigma_floor;
        return Prim(s, 0.0, 0.0, pressure(s, eps_floor, comp.y_e, x_n), x);
    }
    let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
    let eps_th = max(eps_th_raw(sigma, mom, hydro[3u * n + cell], comp.y_e, x_n), eps_floor);
    return Prim(sigma, mom.x / sigma, mom.y / sigma, pressure(sigma, eps_th, comp.y_e, x_n), x);
}

// The cell at (x, y), or a ghost beyond the edge (§2.8): bounce, its mirror inside with the normal velocity negated;
// leave, the edge cell with the normal velocity clamped outward.
fn read_at(x: i32, y: i32) -> Prim {
    let w = i32(dims.width);
    let h = i32(dims.height);
    if (edges.leave != 0u) {
        var p = read_cell(u32(clamp(y, 0, h - 1)) * dims.width + u32(clamp(x, 0, w - 1)));
        if (x < 0) {
            p.u = min(p.u, 0.0);
        } else if (x >= w) {
            p.u = max(p.u, 0.0);
        }
        if (y < 0) {
            p.v = min(p.v, 0.0);
        } else if (y >= h) {
            p.v = max(p.v, 0.0);
        }
        return p;
    }
    var mx = x;
    var my = y;
    if (x < 0) {
        mx = -1 - x;
    } else if (x >= w) {
        mx = 2 * w - 1 - x;
    }
    if (y < 0) {
        my = -1 - y;
    } else if (y >= h) {
        my = 2 * h - 1 - y;
    }
    var p = read_cell(u32(my) * dims.width + u32(mx));
    if (mx != x) {
        p.u = -p.u;
    }
    if (my != y) {
        p.v = -p.v;
    }
    return p;
}

// Whether the cell at (x, y) holds less than Σ_vac; a cell outside the world counts as vacuum.
fn vacuum_at(x: i32, y: i32) -> bool {
    if (x < 0 || y < 0 || x >= i32(dims.width) || y >= i32(dims.height)) {
        return true;
    }
    return hydro[u32(y) * dims.width + u32(x)] < eos.sigma_vac;
}

// The radiation force per area f = κΣF/c_sb on gas cell `cell` (§2.4.4), κ = κ₀(1 + X_H) plus κ_dust Z_met where
// T < T_dust (§2.4.2, as heat.wgsl's `heat_cells`), from the state as the step found it.
fn radiation_force(cell: u32) -> vec2<f32> {
    let n = dims.cells;
    let sigma = hydro[cell];
    var kappa = edges.kappa0 * (1.0 + species_a[ID_H * n + cell]);
    if (edges.kappa_dust != 0.0) {
        var x: array<f32, 10>;
        for (var i = 0u; i < PLANES_A; i++) {
            x[i] = species_a[i * n + cell];
            x[PLANES_A + i] = species_b[i * n + cell];
        }
        let comp = composition(x);
        let mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
        let eps = max(eps_th_raw(sigma, mom, hydro[3u * n + cell], comp.y_e, x[ID_N]), eps_th_floor(comp.inv_mu));
        if (temperature(eps, comp.inv_mu) < edges.t_dust) {
            kappa += edges.kappa_dust * comp.z_met;
        }
    }
    return kappa * sigma * vec2<f32>(rad_flux[cell], rad_flux[n + cell]) / edges.c_sb;
}

// The share s ∈ [0, 1] of the impulse `d` the updated cell `u` can pay for (§0.5 [M0-D58]): the push's work
// (|p₀ + s d|² − |p₀|²)/2Σ at most the thermal energy above the floor, A = Σ(ε_th − T_floor/μ); 0 when A is not
// positive. The root is taken in the form without cancellation on each sign of p₀·d (the twin's `push_share`).
fn push_share(u: array<f32, 14>, d: vec2<f32>) -> f32 {
    let d2 = dot(d, d);
    if (d2 == 0.0) {
        return 1.0;
    }
    let sigma = u[0];
    var x: array<f32, 10>;
    for (var i = 0u; i < 10u; i++) {
        x[i] = u[4u + i] / sigma;
    }
    let comp = composition(x);
    let p0 = vec2<f32>(u[1], u[2]);
    let a = sigma * (eps_th_raw(sigma, p0, u[3], comp.y_e, x[ID_N]) - eps_th_floor(comp.inv_mu));
    if (!(a > 0.0)) {
        return 0.0;
    }
    let q = dot(p0, d);
    let disc = sqrt(q * q + 2.0 * sigma * a * d2);
    var root = (disc - q) / d2;
    if (q >= 0.0) {
        root = 2.0 * sigma * a / (q + disc);
    }
    return min(root, 1.0);
}

// g at (x, y), a ghost's its edge cell's; 0 when the step runs no P2.
fn accel_at(x: i32, y: i32) -> vec2<f32> {
    if (edges.gravity == 0u) {
        return vec2<f32>(0.0, 0.0);
    }
    let cell = u32(clamp(y, 0, i32(dims.height) - 1)) * dims.width + u32(clamp(x, 0, i32(dims.width) - 1));
    return vec2<f32>(grav[cell], grav[dims.cells + cell]);
}

// φ at (x, y), a ghost's its edge cell's (so no work is done across the world's edge); read only when the step runs P2.
fn phi_at(x: i32, y: i32) -> f32 {
    let cell = u32(clamp(y, 0, i32(dims.height) - 1)) * dims.width + u32(clamp(x, 0, i32(dims.width) - 1));
    return grav[2u * dims.cells + cell];
}

// Gravity's energy source at one face of a cell (the header comment): `q` the face's mass flux toward the neighbour
// (along +x or +y), `pi`, `pj` the cell's and the neighbour's half-step momentum on that axis, `si`, `sj` their half-step
// Σ, `d` = φ_cell − φ_neighbour for the + face (φ_neighbour − φ_cell for the − face). The cell's share of the face's
// work q·d: its own ½ pi·d, plus its mass's share of what the face moved beyond the two cells' own momenta.
fn grav_share(q: f32, pi: f32, pj: f32, si: f32, sj: f32, d: f32) -> f32 {
    let a = max(si, 0.0);
    let b = max(sj, 0.0);
    var w = 0.5;
    if (a + b > 0.0) {
        w = a / (a + b);
    }
    return (0.5 * pi + w * (q - 0.5 * (pi + pj))) * d;
}

// The monotonised-central slope: ½(r − l) limited to twice either one-sided difference, 0 at an extremum.
fn mc(l: f32, c: f32, r: f32) -> f32 {
    let a = c - l;
    let b = r - c;
    if (a * b <= 0.0) {
        return 0.0;
    }
    let m = min(min(abs(0.5 * (a + b)), 2.0 * abs(a)), 2.0 * abs(b));
    return select(-m, m, a > 0.0);
}

// The slope of a cell along one axis from its two neighbours on it; the species' central differences all scaled by the
// one factor that brings the most limited within its MC slope (so they keep summing to zero).
fn slope(l: Prim, c: Prim, r: Prim) -> Prim {
    var central: array<f32, 10>;
    var theta = 1.0;
    for (var i = 0u; i < N_SPECIES; i++) {
        central[i] = 0.5 * (r.x[i] - l.x[i]);
        if (central[i] != 0.0) {
            theta = min(theta, mc(l.x[i], c.x[i], r.x[i]) / central[i]);
        }
    }
    for (var i = 0u; i < N_SPECIES; i++) {
        central[i] = theta * central[i];
    }
    return Prim(mc(l.sigma, c.sigma, r.sigma), mc(l.u, c.u, r.u), mc(l.v, c.v, r.v), mc(l.pi, c.pi, r.pi), central);
}

// `c + f · s`.
fn offset(c: Prim, s: Prim, f: f32) -> Prim {
    var x: array<f32, 10>;
    for (var i = 0u; i < N_SPECIES; i++) {
        x[i] = c.x[i] + f * s.x[i];
    }
    return Prim(c.sigma + f * s.sigma, c.u + f * s.u, c.v + f * s.v, c.pi + f * s.pi, x);
}

// Face state `w` of cell `c` with its Σ held between `c`'s and the neighbour `n`'s on that side (M0-D44). The MC slope keeps
// it there in exact arithmetic; in f32, c − ½·2(c − n) is exactly 0 when n is below half of c's ulp (a dense cell facing a
// floor cell, its far neighbour ≥ 4 × denser), and `cons` would divide by it — the twin's f64 face is n.
fn bracketed(w: Prim, c: Prim, n: Prim) -> Prim {
    var b = w;
    b.sigma = clamp(w.sigma, min(c.sigma, n.sigma), max(c.sigma, n.sigma));
    return b;
}

// The conserved vector of primitives: E = Σ(ε_th + ε_cold) + ½Σ|u|², ε_th = (Π − Π_e − Π_n)/Σ floored at T_floor/μ.
fn cons(p: Prim) -> array<f32, 14> {
    let comp = composition(p.x);
    let x_n = p.x[ID_N];
    let thermal = (p.pi - pi_e(p.sigma, comp.y_e) - pi_n(p.sigma, x_n)) / p.sigma;
    let eps_th = max(thermal, eps_th_floor(comp.inv_mu));
    var u: array<f32, 14>;
    u[0] = p.sigma;
    u[1] = p.sigma * p.u;
    u[2] = p.sigma * p.v;
    u[3] = p.sigma * (eps_th + eps_cold(p.sigma, comp.y_e, x_n)) + 0.5 * p.sigma * (p.u * p.u + p.v * p.v);
    for (var i = 0u; i < N_SPECIES; i++) {
        u[4u + i] = p.sigma * p.x[i];
    }
    return u;
}

// The flux through a face normal to `dir` (0: x, 1: y) of a state whose total energy per area is `energy`.
fn flux(p: Prim, energy: f32, dir: u32) -> array<f32, 14> {
    let un = select(p.v, p.u, dir == 0u);
    let ut = select(p.u, p.v, dir == 0u);
    let mass = p.sigma * un;
    var f: array<f32, 14>;
    f[0] = mass;
    // Constant indices: an index from `dir` would keep the array out of registers.
    if (dir == 0u) {
        f[1] = mass * un + p.pi;
        f[2] = mass * ut;
    } else {
        f[1] = mass * ut;
        f[2] = mass * un + p.pi;
    }
    f[3] = (energy + p.pi) * un;
    for (var i = 0u; i < N_SPECIES; i++) {
        f[4u + i] = mass * p.x[i];
    }
    return f;
}

// A face state from primitives `base` (conserved `u0`) advanced by the Hancock increment `delta`; the unevolved state
// when the advanced one is not a state (Σ ≤ 0, no thermal energy left, not finite).
fn evolved(base: Prim, u0: array<f32, 14>, delta: array<f32, 14>) -> Face {
    let unevolved = Face(base, vec4<f32>(u0[0], u0[1], u0[2], u0[3]));
    var u: array<f32, 14>;
    var bad = false;
    for (var k = 0u; k < COMPONENTS; k++) {
        u[k] = u0[k] + delta[k];
        bad = bad || non_finite(u[k]);
    }
    let sigma = u[0];
    if (bad || sigma <= 0.0) {
        return unevolved;
    }
    var x: array<f32, 10>;
    for (var i = 0u; i < N_SPECIES; i++) {
        x[i] = u[4u + i] / sigma;
    }
    let comp = composition(x);
    let x_n = x[ID_N];
    let raw = u[3] / sigma - 0.5 * (u[1] * u[1] + u[2] * u[2]) / (sigma * sigma) - eps_cold(sigma, comp.y_e, x_n);
    if (non_finite(raw) || raw <= 0.0) {
        return unevolved;
    }
    let eps_th = max(raw, eps_th_floor(comp.inv_mu));
    let p = Prim(sigma, u[1] / sigma, u[2] / sigma, pressure(sigma, eps_th, comp.y_e, x_n), x);
    return Face(p, vec4<f32>(u[0], u[1], u[2], u[3]));
}

// The Hancock delta `delta` of a face state `p` with its half kick under g added (the header comment).
fn kicked(delta: array<f32, 14>, p: Prim, g: vec2<f32>, half: f32) -> array<f32, 14> {
    var d = delta;
    d[1] += half * p.sigma * g.x;
    d[2] += half * p.sigma * g.y;
    d[3] += half * p.sigma * ((p.u + 0.5 * half * g.x) * g.x + (p.v + 0.5 * half * g.y) * g.y);
    return d;
}

// A cell's Hancock step: its four unevolved face states, [x−, x+, y−, y+], their conserved vectors, the flux delta,
// its centre's (Σ, Σu, Σv) half a step on and its g.
struct Hancock {
    w: array<Prim, 4>,
    u: array<array<f32, 14>, 4>,
    delta: array<f32, 14>,
    mid: vec3<f32>,
    g: vec2<f32>,
}

// The Hancock step of the cell at (x, y) from its primitives `c` and its neighbours' on x (`xl`, `xr`) and y (`yl`, `yr`).
fn hancock(c: Prim, xl: Prim, xr: Prim, yl: Prim, yr: Prim, x: i32, y: i32, half: f32) -> Hancock {
    let sx = slope(xl, c, xr);
    let sy = slope(yl, c, yr);
    let w = array<Prim, 4>(bracketed(offset(c, sx, -0.5), c, xl), bracketed(offset(c, sx, 0.5), c, xr),
        bracketed(offset(c, sy, -0.5), c, yl), bracketed(offset(c, sy, 0.5), c, yr));
    let u = array<array<f32, 14>, 4>(cons(w[0]), cons(w[1]), cons(w[2]), cons(w[3]));
    let fxl = flux(w[0], u[0][3], 0u);
    let fxr = flux(w[1], u[1][3], 0u);
    let fyl = flux(w[2], u[2][3], 1u);
    let fyr = flux(w[3], u[3][3], 1u);
    var delta: array<f32, 14>;
    for (var k = 0u; k < COMPONENTS; k++) {
        delta[k] = half * ((fxl[k] - fxr[k]) + (fyl[k] - fyr[k]));
    }
    var mid = vec3<f32>(c.sigma + delta[0], c.sigma * c.u + delta[1], c.sigma * c.v + delta[2]);
    var g = vec2<f32>(0.0, 0.0);
    if (edges.gravity != 0u) {
        g = accel_at(x, y);
        mid.y += half * c.sigma * g.x;
        mid.z += half * c.sigma * g.y;
    }
    return Hancock(w, u, delta, mid, g);
}

// The predicted face state from face `w` (conserved `u0`) of a Hancock step: advanced by its `delta`, kicked by its g
// when the step runs P2.
fn face_state(w: Prim, u0: array<f32, 14>, delta: array<f32, 14>, g: vec2<f32>, half: f32) -> Face {
    if (edges.gravity == 0u) {
        return evolved(w, u0, delta);
    }
    return evolved(w, u0, kicked(delta, w, g, half));
}

// A Hancock step's four predicted face states and its centre half a step on.
fn predict(h: Hancock, half: f32) -> Predicted {
    let faces = array<Face, 4>(face_state(h.w[0], h.u[0], h.delta, h.g, half),
        face_state(h.w[1], h.u[1], h.delta, h.g, half), face_state(h.w[2], h.u[2], h.delta, h.g, half),
        face_state(h.w[3], h.u[3], h.delta, h.g, half));
    return Predicted(faces, h.mid);
}

// The predictor of the cell at (x, y), read from the state: its four face states, [x−, x+, y−, y+], and its centre half
// a step on (`hydro_edges`; `hydro_update` reads its tile's primitives instead).
fn cell_faces(x: i32, y: i32, half: f32) -> Predicted {
    let c = read_at(x, y);
    return predict(hancock(c, read_at(x - 1, y), read_at(x + 1, y), read_at(x, y - 1), read_at(x, y + 1), x, y, half), half);
}

// The HLL flux across a face normal to `dir` between `l` and `r` (Davis speeds), species by the mass flux's upwind side.
fn hll(l: Face, r: Face, dir: u32) -> array<f32, 14> {
    let unl = select(l.p.v, l.p.u, dir == 0u);
    let unr = select(r.p.v, r.p.u, dir == 0u);
    let cl = sqrt(max(wave_speed_sq(l.p.sigma, l.p.pi), 0.0));
    let cr = sqrt(max(wave_speed_sq(r.p.sigma, r.p.pi), 0.0));
    let s_l = min(unl - cl, unr - cr);
    let s_r = max(unl + cl, unr + cr);
    let fl = flux(l.p, l.q.w, dir);
    let fr = flux(r.p, r.q.w, dir);
    var f: array<f32, 14>;
    if (s_l >= 0.0) {
        f = fl;
    } else if (s_r <= 0.0) {
        f = fr;
    } else {
        for (var k = 0u; k < 4u; k++) {
            f[k] = (s_r * fl[k] - s_l * fr[k] + s_l * s_r * (r.q[k] - l.q[k])) / (s_r - s_l);
        }
    }
    var up = r.p;
    if (f[0] > 0.0) {
        up = l.p;
    }
    for (var i = 0u; i < N_SPECIES; i++) {
        f[4u + i] = f[0] * up.x[i];
    }
    return f;
}

// The flux through a wall (§2.8, bounce): mass, energy and species exactly 0, the normal momentum the pressure alone.
fn wall(face: Face, dir: u32) -> array<f32, 14> {
    var f: array<f32, 14>;
    if (dir == 0u) {
        f[1] = face.p.pi;
    } else {
        f[2] = face.p.pi;
    }
    return f;
}

// The flux through the boundary face on `side` (0: x = 0, 1: x = width − 1, 2: y = 0, 3: y = height − 1) of the edge
// cell (x, y), whose own predicted face there is `own` (§2.8). Bounce: the wall. Leave: HLL between `own` and the facing
// predicted face of the ghost beyond it, clamped to outflow — a face whose mass flux points into the world is a wall,
// so inflow is exactly 0 and nothing else enters with it.
fn boundary(x: i32, y: i32, side: u32, own: Face, half: f32) -> array<f32, 14> {
    if (edges.leave == 0u) {
        return wall(own, side / 2u);
    }
    var ghost: Face;
    switch side {
        case 0u: {
            ghost = cell_faces(x - 1, y, half).faces[1];
        }
        case 1u: {
            ghost = cell_faces(x + 1, y, half).faces[0];
        }
        case 2u: {
            ghost = cell_faces(x, y - 1, half).faces[3];
        }
        default: {
            ghost = cell_faces(x, y + 1, half).faces[2];
        }
    }
    return edge_flux(side, own, ghost);
}

// `boundary`'s flux from the ghost's facing predicted face `ghost`, already in hand (`hydro_update`'s tile).
fn edge_flux(side: u32, own: Face, ghost: Face) -> array<f32, 14> {
    let dir = side / 2u;
    if (edges.leave == 0u) {
        return wall(own, dir);
    }
    var f: array<f32, 14>;
    switch side {
        case 0u: {
            f = hll(ghost, own, 0u);
        }
        case 1u: {
            f = hll(own, ghost, 0u);
        }
        case 2u: {
            f = hll(ghost, own, 1u);
        }
        default: {
            f = hll(own, ghost, 1u);
        }
    }
    let high = (side & 1u) == 1u;
    if ((high && f[0] < 0.0) || (!high && f[0] > 0.0)) {
        return wall(own, dir);
    }
    return f;
}

// Whether `hydro_update` leaves the cell at (x, y) as it is: it and its four neighbours all below Σ_vac.
fn left_empty(x: i32, y: i32) -> bool {
    return vacuum_at(x, y) && vacuum_at(x - 1, y) && vacuum_at(x + 1, y) && vacuum_at(x, y - 1) && vacuum_at(x, y + 1);
}

// Plane `k` (§2.2's order) of the next state at cell `cell`: set, and read.
fn put_next(k: u32, cell: u32, v: f32) {
    let n = dims.cells;
    if (k < NEXT_LO) {
        next_lo[k * n + cell] = v;
    } else {
        next_hi[(k - NEXT_LO) * n + cell] = v;
    }
}

fn get_next(k: u32, cell: u32) -> f32 {
    let n = dims.cells;
    if (k < NEXT_LO) {
        return next_lo[k * n + cell];
    }
    return next_hi[(k - NEXT_LO) * n + cell];
}

// Writes conserved `u` as cell `cell` of the next state: Σ, the momenta, E and the mass fractions ΣX_i/Σ.
fn write_next(cell: u32, u: array<f32, 14>) {
    for (var k = 0u; k < 4u; k++) {
        put_next(k, cell, u[k]);
    }
    for (var i = 0u; i < N_SPECIES; i++) {
        put_next(4u + i, cell, u[4u + i] / u[0]);
    }
}

// Copies cell `cell` into the next state unchanged.
fn keep(cell: u32) {
    let n = dims.cells;
    for (var k = 0u; k < 4u; k++) {
        put_next(k, cell, hydro[k * n + cell]);
    }
    for (var i = 0u; i < PLANES_A; i++) {
        put_next(4u + i, cell, species_a[i * n + cell]);
    }
    for (var i = 0u; i < PLANES_B; i++) {
        put_next(4u + PLANES_A + i, cell, species_b[i * n + cell]);
    }
}

// `hydro_update`'s tile (M0-D30): its 8 × 8 cells, the ring of 32 around them whose predicted faces the tile's faces
// need (the ring's corners left out), and the ring beyond, which those predictors read — 12 × 12 primitives, slot
// (b + 2) · SPAN + a + 2 for the tile's own coordinates (a, b) in −2 … 9.
const TILE: u32 = 8u;
const SPAN: u32 = 12u;
const PRIM_SLOTS: u32 = 144u;
// One axis's faces: the + face of the cells at −1 … 6 along it (slot (a + 1) · 8 + b), then the − face of those at 1 … 8
// (slot 64 + (a − 1) · 8 + b), b the coordinate across.
const FACE_SLOTS: u32 = 128u;
// Workgroup memory, component-major: the primitives (144 slots × 14 words), then the same words reused for one axis's
// faces at a time (128 slots × 18 words: the primitives' 14, then Σ, Σu, Σv, E) — 9,216 B of §6.2.2's 16,384.
var<workgroup> tile: array<f32, 2304>;
// Whether any cell of the tile moves (`hydro_update`): the invocations' votes, then the one value all of them read.
var<workgroup> votes: atomic<u32>;
// The half-step centres (Σ, Σu, Σv) of the tile's cells (slot b · 8 + a), then of its ring's (64 + the ring invocation's
// index): gravity's energy source reads each cell's four neighbours'.
var<workgroup> mids: array<vec3<f32>, 96>;
var<workgroup> moving: u32;

fn put_prim(s: u32, p: Prim) {
    tile[s] = p.sigma;
    tile[PRIM_SLOTS + s] = p.u;
    tile[2u * PRIM_SLOTS + s] = p.v;
    tile[3u * PRIM_SLOTS + s] = p.pi;
    for (var i = 0u; i < N_SPECIES; i++) {
        tile[(4u + i) * PRIM_SLOTS + s] = p.x[i];
    }
}

fn get_prim(s: u32) -> Prim {
    var x: array<f32, 10>;
    for (var i = 0u; i < N_SPECIES; i++) {
        x[i] = tile[(4u + i) * PRIM_SLOTS + s];
    }
    return Prim(tile[s], tile[PRIM_SLOTS + s], tile[2u * PRIM_SLOTS + s], tile[3u * PRIM_SLOTS + s], x);
}

fn put_face(s: u32, f: Face) {
    tile[s] = f.p.sigma;
    tile[FACE_SLOTS + s] = f.p.u;
    tile[2u * FACE_SLOTS + s] = f.p.v;
    tile[3u * FACE_SLOTS + s] = f.p.pi;
    for (var i = 0u; i < N_SPECIES; i++) {
        tile[(4u + i) * FACE_SLOTS + s] = f.p.x[i];
    }
    for (var k = 0u; k < 4u; k++) {
        tile[(14u + k) * FACE_SLOTS + s] = f.q[k];
    }
}

fn get_face(s: u32) -> Face {
    var x: array<f32, 10>;
    for (var i = 0u; i < N_SPECIES; i++) {
        x[i] = tile[(4u + i) * FACE_SLOTS + s];
    }
    let p = Prim(tile[s], tile[FACE_SLOTS + s], tile[2u * FACE_SLOTS + s], tile[3u * FACE_SLOTS + s], x);
    let q = vec4<f32>(tile[14u * FACE_SLOTS + s], tile[15u * FACE_SLOTS + s], tile[16u * FACE_SLOTS + s],
        tile[17u * FACE_SLOTS + s]);
    return Face(p, q);
}

// The Hancock step of the tile's cell (a, b), from the tile's primitives; (x0, y0) is the tile's first cell.
fn tile_hancock(a: i32, b: i32, x0: i32, y0: i32, half: f32) -> Hancock {
    let s = u32((b + 2) * i32(SPAN) + a + 2);
    return hancock(get_prim(s), get_prim(s - 1u), get_prim(s + 1u), get_prim(s - SPAN), get_prim(s + SPAN), x0 + a, y0 + b,
        half);
}

// One workgroup per 8 × 8 tile of the box; every barrier is reached by all 64 invocations, those past the box's edge
// included — their cells' predictors are their neighbours' faces.
@compute @workgroup_size(8, 8)
fn hydro_update(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(local_invocation_index) li: u32,
    @builtin(workgroup_id) wid: vec3<u32>) {
    let x0 = i32(abox.x0 + TILE * wid.x);
    let y0 = i32(abox.y0 + TILE * wid.y);
    let a = i32(lid.x);
    let b = i32(lid.y);
    let x = x0 + a;
    let y = y0 + b;
    let inside = TILE * wid.x + lid.x < abox.w && TILE * wid.y + lid.y < abox.h;
    let cell = u32(y) * dims.width + u32(x);
    let n = dims.cells;
    // A cell outside the box is not written; one left empty is copied; a tile none of whose cells moves stops here.
    let moves = inside && !left_empty(x, y);
    if (moves) {
        atomicOr(&votes, 1u);
    }
    workgroupBarrier();
    if (li == 0u) {
        moving = atomicLoad(&votes);
    }
    if (workgroupUniformLoad(&moving) == 0u) {
        if (inside) {
            keep(cell);
        }
        return;
    }

    // The primitives, each read once (`read_at`: ghosts beyond the world); the four 2 × 2 corners but their inner
    // cells are read by no predictor.
    for (var s = li; s < PRIM_SLOTS; s += TILE * TILE) {
        let sa = s % SPAN;
        let sb = s / SPAN;
        let corner = (sa < 2u || sa >= SPAN - 2u) && (sb < 2u || sb >= SPAN - 2u);
        let inner = (sa == 1u || sa == SPAN - 2u) && (sb == 1u || sb == SPAN - 2u);
        if (!corner || inner) {
            put_prim(s, read_at(x0 - 2 + i32(sa), y0 - 2 + i32(sb)));
        }
    }
    workgroupBarrier();

    let dt = dt_rec.x;
    let half = 0.5 * dt;
    // The ring's predictors, invocations 0 … 31, each keeping the one face the tile needs: column −1's x+ (0 … 7),
    // column 8's x− (8 … 15), row −1's y+ (16 … 23), row 8's y− (24 … 31). The face is chosen before it is evolved, so
    // the four groups run one `evolved`.
    var ring: Face;
    if (li < 32u) {
        let k = i32(li % TILE);
        let side = li / TILE;
        var ra = k;
        var rb = k;
        if (side == 0u) {
            ra = -1;
        } else if (side == 1u) {
            ra = i32(TILE);
        } else if (side == 2u) {
            rb = -1;
        } else {
            rb = i32(TILE);
        }
        let h = tile_hancock(ra, rb, x0, y0, half);
        var w = h.w[1];
        var u0 = h.u[1];
        if (side == 1u) {
            w = h.w[0];
            u0 = h.u[0];
        } else if (side == 2u) {
            w = h.w[3];
            u0 = h.u[3];
        } else if (side == 3u) {
            w = h.w[2];
            u0 = h.u[2];
        }
        ring = face_state(w, u0, h.delta, h.g, half);
        mids[64u + li] = h.mid;
    }
    let pred = predict(tile_hancock(a, b, x0, y0, half), half);
    let own = pred.faces;
    mids[lid.y * TILE + lid.x] = pred.mid;
    workgroupBarrier();

    let last_x = i32(dims.width) - 1;
    let last_y = i32(dims.height) - 1;
    // Face fluxes, each between the cell before it on its axis and the cell after: x−, x+, y−, y+ — the x faces through
    // the tile's memory, then the y faces.
    var fxl: array<f32, 14>;
    var fxr: array<f32, 14>;
    var fyl: array<f32, 14>;
    var fyr: array<f32, 14>;
    if (a < i32(TILE) - 1) {
        put_face(u32(a + 1) * TILE + lid.y, own[1]);
    }
    if (a > 0) {
        put_face(64u + u32(a - 1) * TILE + lid.y, own[0]);
    }
    if (li < TILE) {
        put_face(li, ring);
    } else if (li < 2u * TILE) {
        put_face(64u + (TILE - 1u) * TILE + li - TILE, ring);
    }
    workgroupBarrier();
    if (moves) {
        let before = get_face(lid.x * TILE + lid.y);
        let after = get_face(64u + lid.x * TILE + lid.y);
        if (x == 0) {
            fxl = edge_flux(0u, own[0], before);
        } else {
            fxl = hll(before, own[0], 0u);
        }
        if (x == last_x) {
            fxr = edge_flux(1u, own[1], after);
        } else {
            fxr = hll(own[1], after, 0u);
        }
    }
    workgroupBarrier();
    if (b < i32(TILE) - 1) {
        put_face(u32(b + 1) * TILE + lid.x, own[3]);
    }
    if (b > 0) {
        put_face(64u + u32(b - 1) * TILE + lid.x, own[2]);
    }
    if (li >= 2u * TILE && li < 3u * TILE) {
        put_face(li - 2u * TILE, ring);
    } else if (li >= 3u * TILE && li < 4u * TILE) {
        put_face(64u + (TILE - 1u) * TILE + li - 3u * TILE, ring);
    }
    workgroupBarrier();
    if (!moves) {
        if (inside) {
            keep(cell);
        }
        return;
    }
    let below = get_face(lid.y * TILE + lid.x);
    let above = get_face(64u + lid.y * TILE + lid.x);
    if (y == 0) {
        fyl = edge_flux(2u, own[2], below);
    } else {
        fyl = hll(below, own[2], 1u);
    }
    if (y == last_y) {
        fyr = edge_flux(3u, own[3], above);
    } else {
        fyr = hll(own[3], above, 1u);
    }
    let empty_here = vacuum_at(x, y);

    // The stored vector where there is gas (bit for bit what the planes hold), the floor state's below Σ_vac.
    var u: array<f32, 14>;
    if (empty_here) {
        u = cons(read_cell(cell));
    } else {
        for (var k = 0u; k < 4u; k++) {
            u[k] = hydro[k * n + cell];
        }
        for (var i = 0u; i < PLANES_A; i++) {
            u[4u + i] = u[0] * species_a[i * n + cell];
        }
        for (var i = 0u; i < PLANES_B; i++) {
            u[4u + PLANES_A + i] = u[0] * species_b[i * n + cell];
        }
    }
    for (var k = 0u; k < COMPONENTS; k++) {
        u[k] -= dt * ((fxr[k] - fxl[k]) + (fyr[k] - fyl[k]));
    }
    // Gravity's source, time-centred on the predictor's half step (§1.4.2).
    if (edges.gravity != 0u) {
        let g = accel_at(x, y);
        u[1] += dt * pred.mid.x * g.x;
        u[2] += dt * pred.mid.x * g.y;
        let p = phi_at(x, y);
        let m = pred.mid;
        let ml = mids[select(64u + lid.y, lid.y * TILE + lid.x - 1u, lid.x > 0u)];
        let mr = mids[select(72u + lid.y, lid.y * TILE + lid.x + 1u, lid.x + 1u < TILE)];
        let mu = mids[select(80u + lid.x, (lid.y - 1u) * TILE + lid.x, lid.y > 0u)];
        let md = mids[select(88u + lid.x, (lid.y + 1u) * TILE + lid.x, lid.y + 1u < TILE)];
        u[3] += dt * (grav_share(fxr[0], m.y, mr.y, m.x, mr.x, p - phi_at(x + 1, y))
            + grav_share(fxl[0], m.y, ml.y, m.x, ml.x, phi_at(x - 1, y) - p)
            + grav_share(fyr[0], m.z, md.z, m.x, md.x, p - phi_at(x, y + 1))
            + grav_share(fyl[0], m.z, mu.z, m.x, mu.x, phi_at(x, y - 1) - p));
    }
    // The radiation force from the previous step's flux (§2.4.4): momentum only, E as it is.
    if (edges.radiation != 0u && !empty_here) {
        let f = radiation_force(cell);
        let d = dt * f;
        let s = push_share(u, d);
        u[1] += s * d.x;
        u[2] += s * d.y;
    }
    write_next(cell, u);
}

// One invocation per boundary-face slot, leaving only: the face's `boundary` flux, as `hydro_update` applied it to its
// edge cell, added to the slot's escaped terms — the grid loses dt·f through a high face, −dt·f through a low one. A face
// whose cell `hydro_update` did not move (outside the box, or left empty) books nothing. Runs before `hydro_commit`, on
// the state as the step found it.
@compute @workgroup_size(64)
fn hydro_edges(@builtin(global_invocation_id) id: vec3<u32>) {
    let w = dims.width;
    let h = dims.height;
    let slots = 2u * (w + h);
    let slot = id.x;
    if (slot >= slots || edges.leave == 0u) {
        return;
    }
    var cx = 0u;
    var cy = 0u;
    var side = 0u;
    if (slot < h) {
        cy = slot;
    } else if (slot < 2u * h) {
        cx = w - 1u;
        cy = slot - h;
        side = 1u;
    } else if (slot < 2u * h + w) {
        cx = slot - 2u * h;
        side = 2u;
    } else {
        cx = slot - 2u * h - w;
        cy = h - 1u;
        side = 3u;
    }
    if (cx < abox.x0 || cx >= abox.x0 + abox.w || cy < abox.y0 || cy >= abox.y0 + abox.h) {
        return;
    }
    let x = i32(cx);
    let y = i32(cy);
    if (left_empty(x, y)) {
        return;
    }
    let dt = dt_rec.x;
    let half = 0.5 * dt;
    var own = cell_faces(x, y, half).faces;
    let f = boundary(x, y, side, own[side], half);
    let s = select(-dt, dt, (side & 1u) == 1u);
    for (var k = 0u; k < COMPONENTS; k++) {
        escaped[k * slots + slot] += s * f[k];
    }
}

@compute @workgroup_size(8, 8)
fn hydro_commit(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let cell = (abox.y0 + id.y) * dims.width + abox.x0 + id.x;
    let n = dims.cells;
    for (var k = 0u; k < 4u; k++) {
        out_hydro[k * n + cell] = get_next(k, cell);
    }
    for (var i = 0u; i < PLANES_A; i++) {
        out_a[i * n + cell] = get_next(4u + i, cell);
    }
    for (var i = 0u; i < PLANES_B; i++) {
        out_b[i * n + cell] = get_next(4u + PLANES_A + i, cell);
    }
}
