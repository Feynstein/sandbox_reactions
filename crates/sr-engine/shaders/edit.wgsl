// P0 edits (m0_contrat.md §1.3.2, §1.12, §2.7, §2.9): the player's queued edits applied at the step boundary, three
// dispatches in this order — `edit_gate`, `edit_cells`, `edit_finish`. Compiled with eos.wgsl prepended (group 1).
//
// The queue (step/edit.rs `Edits`): a header whose first word is the count of edits queued since the last step, then up to
// MAX_EDITS records of REC words, applied in queue order. Every record is a plain value; no pass reads the wall clock.
//
// `edit_gate` (one invocation) sizes `edit_cells` from the count — the whole world in 256-cell workgroups, nothing when
// no edit is queued — and sets the box's edit flag (atomicOr 1) when a paint or a drop lands outside the active box, so
// the box_gate / box_cells / box_fit that follow in P0 re-fit it (§1.3.3). `edit_cells` runs one invocation per cell, which
// walks the queued records in order and applies each to its own cell only: no atomic, the same queue gives the same bits
// (§1.3.4). `edit_finish` empties the queue and, for a Clear, removes the sinks.
//
// What each kind does to a cell (m0_contrat.md §1.12; all at rest, momentum density unchanged):
// - paint: inside the disk, ΔΣ = paint_sigma of the species at T = paint_t, mixed by mass;
// - drop: where the preset cloud's patch holds Σ_p, that mass at the cloud's temperature and composition, mixed likewise;
// - erase, clear: the floor state — Σ_floor, u = 0, T_floor, pure hydrogen (M0-T3's default composition);
// - heat, cool: ε_th × heat_factor or cool_factor, floored at T_floor, on cells that are not vacuum.
// A deposit keeps the gas it lands on exactly as it was: its thermal energy Σ ε_th is added to, the cold energy follows the
// new Σ and composition, and the kinetic energy follows the unchanged momentum on the larger Σ. Nothing branches on the
// cell's density or temperature (B7: a shining star reacts through the same physics as gas).
//
// Booking (state.rs BOOK_GROUPS 0 and 1, one slot per cell and term, written only by the cell's own invocation): the mass
// a cell gained (painted, preset_dropped) or lost (erased, cleared), and the energy its E plane changed by under each
// tool — the stored difference, so the ledger balances to the bit whatever the f32 rounding of the update.

struct Dims {
    width: u32,
    height: u32,
    cells: u32,
    _pad: u32,
}

struct EditParams {
    paint_sigma: f32,
    paint_t: f32,
    heat_factor: f32,
    cool_factor: f32,
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

@group(0) @binding(0) var<uniform> dims: Dims;
@group(0) @binding(1) var<storage, read_write> species_a: array<f32>;
@group(0) @binding(2) var<storage, read_write> species_b: array<f32>;
@group(0) @binding(3) var<storage, read_write> hydro: array<f32>;
// booking.tools_mass: painted, erased, preset_dropped, cleared (one plane of `cells` slots each).
@group(0) @binding(4) var<storage, read_write> book_mass: array<f32>;
// booking.tools_energy: painted, erased, heated, cooled, preset, cleared.
@group(0) @binding(5) var<storage, read_write> book_energy: array<f32>;
@group(0) @binding(6) var<storage, read_write> queue: array<u32>;
// The Σ of the queued preset clouds, one patch after another, each w × h row-major.
@group(0) @binding(7) var<storage, read> cloud_sigma: array<f32>;
@group(0) @binding(8) var<uniform> params: EditParams;
@group(0) @binding(9) var<storage, read_write> gate: array<u32, 6>;
@group(0) @binding(10) var<storage, read_write> box_flag: atomic<u32>;
@group(0) @binding(11) var<uniform> abox: ActiveBox;
// The sinks' state buffer: its first word is the count, its second the formed-cell marker.
@group(0) @binding(12) var<storage, read_write> sinks: array<u32>;

const WG: u32 = 256u;
const HEAD: u32 = 4u;
const REC: u32 = 24u;
const MAX_EDITS: u32 = 8u;

const PAINT: u32 = 1u;
const ERASE: u32 = 2u;
const HEAT: u32 = 3u;
const COOL: u32 = 4u;
const DROP: u32 = 5u;
const CLEAR: u32 = 6u;

// Record words: kind, x, y, radius, species, then a drop's patch x0, y0, w, h and offset (in cells), its temperature and
// its composition from word 11.
const W_KIND: u32 = 0u;
const W_X: u32 = 1u;
const W_Y: u32 = 2u;
const W_RADIUS: u32 = 3u;
const W_SPECIES: u32 = 4u;
const W_PX: u32 = 5u;
const W_PY: u32 = 6u;
const W_PW: u32 = 7u;
const W_PH: u32 = 8u;
const W_POFF: u32 = 9u;
const W_T: u32 = 10u;
const W_X0: u32 = 11u;

fn word(k: u32, w: u32) -> u32 {
    return queue[HEAD + k * REC + w];
}

fn edit_count() -> u32 {
    return min(queue[0], MAX_EDITS);
}

@compute @workgroup_size(1)
fn edit_gate() {
    let count = edit_count();
    gate[0] = select(0u, (dims.cells + WG - 1u) / WG, count > 0u);
    gate[1] = 1u;
    gate[2] = 1u;
    let w = i32(dims.width);
    let h = i32(dims.height);
    let bx0 = i32(abox.x0);
    let by0 = i32(abox.y0);
    var outside = false;
    for (var k = 0u; k < count; k++) {
        let kind = word(k, W_KIND);
        var lo = vec2<i32>(0, 0);
        var hi = vec2<i32>(0, 0);
        if (kind == PAINT) {
            let c = vec2<i32>(i32(word(k, W_X)), i32(word(k, W_Y)));
            let r = i32(word(k, W_RADIUS));
            lo = c - vec2<i32>(r, r);
            hi = c + vec2<i32>(r, r);
        } else if (kind == DROP) {
            lo = vec2<i32>(i32(word(k, W_PX)), i32(word(k, W_PY)));
            hi = lo + vec2<i32>(i32(word(k, W_PW)), i32(word(k, W_PH))) - vec2<i32>(1, 1);
        } else {
            continue;
        }
        lo = max(lo, vec2<i32>(0, 0));
        hi = min(hi, vec2<i32>(w - 1, h - 1));
        if (lo.x < bx0 || lo.y < by0 || hi.x >= bx0 + i32(abox.w) || hi.y >= by0 + i32(abox.h)) {
            outside = true;
        }
    }
    if (outside) {
        atomicOr(&box_flag, 1u);
    }
}

fn floor_hydrogen() -> array<f32, 10> {
    var x: array<f32, 10>;
    for (var i = 0u; i < N_SPECIES; i++) {
        x[i] = 0.0;
    }
    x[ID_H] = 1.0;
    return x;
}

// The floor state's E for pure hydrogen: Σ_floor (T_floor/μ + ε_cold(Σ_floor)) — P8's own (floors.wgsl).
fn floor_energy() -> f32 {
    let x = floor_hydrogen();
    let comp = composition(x);
    let s = eos.sigma_floor;
    return s * (eps_th_floor(comp.inv_mu) + eps_cold(s, comp.y_e, 0.0));
}

// A cell's state while the queue is walked: ΔΣ and ΔE per booked term are added after each edit.
struct Cell {
    sigma: f32,
    mom: vec2<f32>,
    energy: f32,
    x: array<f32, 10>,
}

// Adds `ds` of gas of fractions `xp` at specific thermal energy `eps_p`, at rest, to the cell. Returns (ΔΣ, ΔE) as
// stored. A cell at Σ = 0 — outside every box of a dump made before M0-D101, whose scenes left their vacuum at 0 (P8
// floors only the box's cells) — has no cold or kinetic energy, where ε_cold(0) and ½|m|²/Σ₀ are 0/0.
fn deposit(c: ptr<function, Cell>, ds: f32, xp: array<f32, 10>, eps_p: f32) -> vec2<f32> {
    let s0 = (*c).sigma;
    let e0 = (*c).energy;
    let mom = (*c).mom;
    let gas = s0 > 0.0;
    let comp0 = composition((*c).x);
    let u0 = select(0.0, s0 * eps_cold(s0, comp0.y_e, (*c).x[ID_N]), gas);
    let s1 = s0 + ds;
    var x1: array<f32, 10>;
    for (var i = 0u; i < N_SPECIES; i++) {
        x1[i] = (s0 * (*c).x[i] + ds * xp[i]) / s1;
    }
    let comp1 = composition(x1);
    let u1 = s1 * eps_cold(s1, comp1.y_e, x1[ID_N]);
    // ½|m|² (1/Σ₁ − 1/Σ₀): the same momentum on the larger Σ.
    let kinetic = select(0.0, -0.5 * dot(mom, mom) * ds / (s0 * s1), gas);
    let e1 = e0 + ds * eps_p + (u1 - u0) + kinetic;
    (*c).sigma = s1;
    (*c).energy = e1;
    (*c).x = x1;
    return vec2<f32>(s1 - s0, e1 - e0);
}

// The floor state in place of the cell. Returns (mass lost, ΔE).
fn to_floor(c: ptr<function, Cell>) -> vec2<f32> {
    let s = eos.sigma_floor;
    let e = floor_energy();
    let lost = (*c).sigma - s;
    let de = e - (*c).energy;
    (*c).sigma = s;
    (*c).mom = vec2<f32>(0.0, 0.0);
    (*c).energy = e;
    (*c).x = floor_hydrogen();
    return vec2<f32>(lost, de);
}

// ε_th × `factor`, floored at T_floor and never raised by a cool; the cell's non-vacuum gas only. Returns ΔE.
fn scale_thermal(c: ptr<function, Cell>, factor: f32, cooling: bool) -> f32 {
    if ((*c).sigma < eos.sigma_vac) {
        return 0.0;
    }
    let comp = composition((*c).x);
    let eps = eps_th_raw((*c).sigma, (*c).mom, (*c).energy, comp.y_e, (*c).x[ID_N]);
    var scaled = max(eps * factor, eps_th_floor(comp.inv_mu));
    if (cooling) {
        scaled = min(scaled, eps);
    }
    let e = (*c).energy + (*c).sigma * (scaled - eps);
    let de = e - (*c).energy;
    (*c).energy = e;
    return de;
}

@compute @workgroup_size(256)
fn edit_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let cell = id.x;
    if (cell >= dims.cells) {
        return;
    }
    let n = dims.cells;
    let count = edit_count();
    let px = cell % dims.width;
    let py = cell / dims.width;

    var c: Cell;
    c.sigma = hydro[cell];
    c.mom = vec2<f32>(hydro[n + cell], hydro[2u * n + cell]);
    c.energy = hydro[3u * n + cell];
    for (var i = 0u; i < 5u; i++) {
        c.x[i] = species_a[i * n + cell];
        c.x[5u + i] = species_b[i * n + cell];
    }

    // The booked terms of this cell: mass painted, erased, preset_dropped, cleared; energy painted, erased, heated,
    // cooled, preset, cleared.
    var mass = vec4<f32>(0.0);
    var en: array<f32, 6>;
    var touched = false;
    for (var k = 0u; k < count; k++) {
        let kind = word(k, W_KIND);
        let dx = i32(px) - i32(word(k, W_X));
        let dy = i32(py) - i32(word(k, W_Y));
        let r = i32(word(k, W_RADIUS));
        let inside = dx * dx + dy * dy <= r * r;
        if (kind == PAINT && inside) {
            var xp = floor_hydrogen();
            let sp = word(k, W_SPECIES);
            for (var i = 0u; i < N_SPECIES; i++) {
                xp[i] = select(0.0, 1.0, i == sp);
            }
            let d = deposit(&c, params.paint_sigma, xp, params.paint_t * eos.inv_mu_coef[sp / 4u][sp % 4u]);
            mass.x += d.x;
            en[0] += d.y;
            touched = true;
        } else if (kind == ERASE && inside) {
            let d = to_floor(&c);
            mass.y += d.x;
            en[1] += d.y;
            touched = true;
        } else if (kind == HEAT && inside) {
            en[2] += scale_thermal(&c, params.heat_factor, false);
            touched = true;
        } else if (kind == COOL && inside) {
            en[3] += scale_thermal(&c, params.cool_factor, true);
            touched = true;
        } else if (kind == DROP) {
            let ox = i32(px) - i32(word(k, W_PX));
            let oy = i32(py) - i32(word(k, W_PY));
            if (ox >= 0 && oy >= 0 && ox < i32(word(k, W_PW)) && oy < i32(word(k, W_PH))) {
                let sp = cloud_sigma[word(k, W_POFF) + u32(oy) * word(k, W_PW) + u32(ox)];
                if (sp > 0.0) {
                    var xp: array<f32, 10>;
                    for (var i = 0u; i < N_SPECIES; i++) {
                        xp[i] = bitcast<f32>(word(k, W_X0 + i));
                    }
                    let eps_p = bitcast<f32>(word(k, W_T)) * composition(xp).inv_mu;
                    let d = deposit(&c, sp, xp, eps_p);
                    mass.z += d.x;
                    en[4] += d.y;
                    touched = true;
                }
            }
        } else if (kind == CLEAR) {
            let d = to_floor(&c);
            mass.w += d.x;
            en[5] += d.y;
            touched = true;
        }
    }
    if (!touched) {
        return;
    }

    hydro[cell] = c.sigma;
    hydro[3u * n + cell] = c.energy;
    hydro[n + cell] = c.mom.x;
    hydro[2u * n + cell] = c.mom.y;
    for (var i = 0u; i < 5u; i++) {
        species_a[i * n + cell] = c.x[i];
        species_b[i * n + cell] = c.x[5u + i];
    }
    for (var t = 0u; t < 4u; t++) {
        book_mass[t * n + cell] += mass[t];
    }
    for (var t = 0u; t < 6u; t++) {
        book_energy[t * n + cell] += en[t];
    }
}

@compute @workgroup_size(1)
fn edit_finish() {
    let count = edit_count();
    for (var k = 0u; k < count; k++) {
        if (word(k, W_KIND) == CLEAR) {
            sinks[0] = 0u;
            sinks[1] = 0u;
        }
    }
    queue[0] = 0u;
}
