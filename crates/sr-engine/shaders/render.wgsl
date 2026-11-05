// The cell image (m0_contrat.md §1.11, §1.10.2, §1.8.3, §4.11; M0-T80): one compute pass writing the W × H RGBA8 image of the
// active view from the state, blended with the previous state by α (slow motion, §1.8.3). Compiled with eos.wgsl prepended
// (group 1): T is read as the block summary reads it — μ ε_th, ε_th = E/Σ − ½|u|² − ε_cold floored at T_floor/μ — then
// translated to kelvins through §1.10.2's anchors (piecewise log-linear, 10 K at and below T_floor, the last segment
// extended), as the readouts and the inspector do.
//
// The blend is of the state (Σ, Σu, Σv, E and the ten mass fractions, channel by channel), before anything is derived: α = 0
// is the previous state exactly and α = 1 the current one (no mix is evaluated at either). A cell whose Σ is not positive —
// a state never written — reads as the floor state (Σ_floor, at rest, T_floor, hydrogen).
//
// Views, each one a function of the (blended) cell:
//   Glow    blackbody(T_K) · b + haze · (1 − b), b = (1 − exp(−(T_K / min(S, 3000))⁴)) · D(T_K) · ε, haze = #2A3550 · clamp((log₁₀ Σ + 8) / 8, 0, 1) ·
//           0.6 — the formula's colours as written (the table is sRGB-encoded, §1.11), the table read linearly in log T between
//           its 512 entries and clamped to its ends; ε = 1 − exp(−10 κ₀(1 + X_H) Σ) the cell's emissivity, from its optical
//           depth across the plane (§2.4.2's κ, scaled by render.rs `EMISSIVITY_SCALE`; [M0-D157]: thin gas glows as faintly as
//           it emits). (The glow added over this image is glow.wgsl's, M0-T81.)
//   Heat    §4.11's map over the Heat range's log₁₀ T_K (1 … 10 by default).
//   Element the species colours (§1.5) weighted by mass fraction, in linear RGB, times clamp((log₁₀ Σ + 6) / 8, 0.15, 1); a
//           cell under Σ_vac is black.
//   Density §4.11's map over the Density range's log₁₀ Σ (−6 … 4 by default, sandbox units).
// S, D and the ranges are the display settings ([M0-R12], M0-T126); at the defaults (3,000 K, D = 1, the whole spans) the
// formulas are the ones above with 3000, 1 … 10 and −6 … 4.
// The maps interpolate linearly in linear RGB between their stops and clamp outside (§4.11); the stops are sRGB hex.

struct Dims {
    width: u32,
    height: u32,
    cells: u32,
    _pad: u32,
}

// render.rs `View` as a word, and α; `lo` and `span` (log₁₀ units) are `extent_bins`' histogram span, unused by `render_cells`.
struct RenderParams {
    view: u32,
    alpha: f32,
    lo: f32,
    span: f32,
}

// The constants of one run, uploaded once (render.rs `Renderer::new`): §1.10.2's eight anchors as ln T_sb, ln T_K and the
// slope of ln T_K in ln T_sb of each segment (the eighth unused); the species colours in linear RGB; the blackbody table
// (sRGB-encoded, one vec4 per entry, w unused).
struct Palette {
    ln_t: array<vec4<f32>, 2>,
    ln_k: array<vec4<f32>, 2>,
    slope: array<vec4<f32>, 2>,
    species: array<vec4<f32>, 10>,
    blackbody: array<vec4<f32>, 512>,
}

// The display settings (render.rs `Display`, M0-T126; §1.11 [M0-R12]): glow.x = min(S, 3000) K, the Heat range ends
// (log₁₀ T_K) and the Density range ends (log₁₀ Σ) in `ranges`, and the Glow dimming D over log₁₀ T_K from log₁₀ 3000 to 11
// (1,024 entries, four to a vec4), built on the CPU when S changes; glow.y = EMISSIVITY_SCALE · κ₀, the run's κ₀ scaled
// (the emissivity, [M0-D157]); glow.zw unused.
struct DisplayParams {
    glow: vec4<f32>,
    ranges: vec4<f32>,
    dimming: array<vec4<f32>, 256>,
}

@group(0) @binding(0) var<uniform> dims: Dims;
// The current state, then the previous: hydro planes (sigma, mom_x, mom_y, energy), species planes x_H … x_Ne, x_Mg … x_n.
@group(0) @binding(1) var<storage, read> hydro: array<f32>;
@group(0) @binding(2) var<storage, read> species_a: array<f32>;
@group(0) @binding(3) var<storage, read> species_b: array<f32>;
@group(0) @binding(4) var<storage, read> prev_hydro: array<f32>;
@group(0) @binding(5) var<storage, read> prev_species_a: array<f32>;
@group(0) @binding(6) var<storage, read> prev_species_b: array<f32>;
@group(0) @binding(7) var<uniform> params: RenderParams;
@group(0) @binding(8) var<uniform> pal: Palette;
@group(0) @binding(9) var image: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(10) var<uniform> display: DisplayParams;
// `extent_bins`' histogram: 1,024 counters, zero on entry (render.rs `Renderer::fit_range`).
@group(0) @binding(11) var<storage, read_write> bins: array<atomic<u32>, 1024>;

const VIEW_GLOW: u32 = 0u;
const VIEW_HEAT: u32 = 1u;
const VIEW_ELEMENT: u32 = 2u;
const VIEW_DENSITY: u32 = 3u;

const N_ANCHORS: u32 = 8u;
const TABLE_LEN: u32 = 512u;
const TABLE_T_MIN: f32 = 1000.0;
const TABLE_LN_RANGE: f32 = 3.688879454113936; // ln(40,000 / 1,000)
const EXTENT_BINS: u32 = 1024u;
const DIMMING_LEN: u32 = 1024u;
const DIMMING_L10_LO: f32 = 3.4771213; // log₁₀ 3,000
const DIMMING_L10_SPAN: f32 = 7.5228787; // 11 − log₁₀ 3,000

// §4.11 as sRGB bytes / 255, stops evenly spaced over log₁₀ T_K 1 … 10 and log₁₀ Σ −6 … 4.
var<private> HEAT: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3<f32>(0.000000, 0.000000, 0.015686), // #000004
    vec3<f32>(0.231373, 0.058824, 0.439216), // #3B0F70
    vec3<f32>(0.549020, 0.160784, 0.505882), // #8C2981
    vec3<f32>(0.870588, 0.286275, 0.407843), // #DE4968
    vec3<f32>(0.996078, 0.623529, 0.427451), // #FE9F6D
    vec3<f32>(0.988235, 0.992157, 0.749020), // #FCFDBF
);
var<private> DENSITY: array<vec3<f32>, 5> = array<vec3<f32>, 5>(
    vec3<f32>(0.266667, 0.003922, 0.329412), // #440154
    vec3<f32>(0.231373, 0.321569, 0.545098), // #3B528B
    vec3<f32>(0.129412, 0.568627, 0.549020), // #21918C
    vec3<f32>(0.368627, 0.788235, 0.384314), // #5EC962
    vec3<f32>(0.992157, 0.905882, 0.145098), // #FDE725
);
const HAZE: vec3<f32> = vec3<f32>(0.164706, 0.207843, 0.313725); // #2A3550
const HAZE_GAIN: f32 = 0.6;
const BG_SPACE: vec3<f32> = vec3<f32>(0.019608, 0.027451, 0.050980); // #05070D

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let k = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(1.055 * pow(k, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055), 12.92 * k, k <= vec3<f32>(0.0031308));
}

struct Cell {
    sigma: f32,
    mom: vec2<f32>,
    energy: f32,
    x: array<f32, 10>,
}

fn floor_cell() -> Cell {
    var c: Cell;
    c.sigma = eos.sigma_floor;
    c.mom = vec2<f32>(0.0);
    c.energy = 0.0;
    c.x[ID_H] = 1.0;
    return c;
}

fn load(i: u32, previous: bool) -> Cell {
    var c: Cell;
    let n = dims.cells;
    if (previous) {
        c.sigma = prev_hydro[i];
        c.mom = vec2<f32>(prev_hydro[n + i], prev_hydro[2u * n + i]);
        c.energy = prev_hydro[3u * n + i];
        for (var k = 0u; k < 5u; k++) {
            c.x[k] = prev_species_a[k * n + i];
            c.x[5u + k] = prev_species_b[k * n + i];
        }
    } else {
        c.sigma = hydro[i];
        c.mom = vec2<f32>(hydro[n + i], hydro[2u * n + i]);
        c.energy = hydro[3u * n + i];
        for (var k = 0u; k < 5u; k++) {
            c.x[k] = species_a[k * n + i];
            c.x[5u + k] = species_b[k * n + i];
        }
    }
    if (!(c.sigma > 0.0)) {
        return floor_cell();
    }
    return c;
}

// The state at α of the way from the previous to the current one; neither end evaluates a mix.
fn blended(i: u32, alpha: f32) -> Cell {
    if (alpha <= 0.0) {
        return load(i, true);
    }
    if (alpha >= 1.0) {
        return load(i, false);
    }
    let a = load(i, true);
    let b = load(i, false);
    var c: Cell;
    c.sigma = mix(a.sigma, b.sigma, alpha);
    c.mom = mix(a.mom, b.mom, alpha);
    c.energy = mix(a.energy, b.energy, alpha);
    for (var k = 0u; k < N_SPECIES; k++) {
        c.x[k] = mix(a.x[k], b.x[k], alpha);
    }
    return c;
}

// T in sandbox units as the summary reads it: μ ε_th, ε_th floored at T_floor/μ.
fn cell_temperature(c: Cell) -> f32 {
    let comp = composition(c.x);
    let eps = max(eps_th_raw(c.sigma, c.mom, c.energy, comp.y_e, c.x[ID_N]), eps_th_floor(comp.inv_mu));
    return temperature(eps, comp.inv_mu);
}

// §1.10.2: the piecewise log-linear translation to kelvins; 10 K at and below T_floor, the last segment extended.
fn to_kelvin(t_sb: f32) -> f32 {
    if (!(t_sb > exp(pal.ln_t[0].x))) {
        return exp(pal.ln_k[0].x);
    }
    let l = log(t_sb);
    var seg = 0u;
    for (var k = 1u; k < N_ANCHORS - 1u; k++) {
        if (l >= pal.ln_t[k / 4u][k % 4u]) {
            seg = k;
        }
    }
    let lt = pal.ln_t[seg / 4u][seg % 4u];
    let lk = pal.ln_k[seg / 4u][seg % 4u];
    return exp(lk + pal.slope[seg / 4u][seg % 4u] * (l - lt));
}

// The 512-entry table, linear in log T between entries, the end colours outside (render.rs `blackbody_lookup`).
fn blackbody(t_k: f32) -> vec3<f32> {
    let u = clamp(log(t_k / TABLE_T_MIN) / TABLE_LN_RANGE, 0.0, 1.0) * f32(TABLE_LEN - 1u);
    let i = min(u32(floor(u)), TABLE_LEN - 2u);
    return mix(pal.blackbody[i].xyz, pal.blackbody[i + 1u].xyz, u - f32(i));
}

// The dimming table D(T), linear in log T between entries, clamped to its ends (render.rs `dimming_lookup`); a + (b − a) f is
// exactly a where the entries are equal — every entry is 1 at S ≤ 3,000 K.
fn dimming(t_k: f32) -> f32 {
    let u = clamp((log2(t_k) * 0.30102999566 - DIMMING_L10_LO) / DIMMING_L10_SPAN, 0.0, 1.0) * f32(DIMMING_LEN - 1u);
    let i = min(u32(floor(u)), DIMMING_LEN - 2u);
    let a = display.dimming[i / 4u][i % 4u];
    let b = display.dimming[(i + 1u) / 4u][(i + 1u) % 4u];
    return a + (b - a) * (u - f32(i));
}

// A map of stops (sRGB) at position p ∈ [0, n − 1], interpolated in linear RGB, returned sRGB.
fn heat_map(l10_t: f32) -> vec3<f32> {
    let p = clamp((l10_t - display.ranges.x) / (display.ranges.y - display.ranges.x), 0.0, 1.0) * 5.0;
    let i = min(u32(floor(p)), 4u);
    return linear_to_srgb(mix(srgb_to_linear(HEAT[i]), srgb_to_linear(HEAT[i + 1u]), p - f32(i)));
}

fn density_map(l10_sigma: f32) -> vec3<f32> {
    let p = clamp((l10_sigma - display.ranges.z) / (display.ranges.w - display.ranges.z), 0.0, 1.0) * 4.0;
    let i = min(u32(floor(p)), 3u);
    return linear_to_srgb(mix(srgb_to_linear(DENSITY[i]), srgb_to_linear(DENSITY[i + 1u]), p - f32(i)));
}

// ε = 1 − exp(−τ), τ = 10 κ₀(1 + X_H) Σ: the cell's optical depth across the plane (§2.4.2), × EMISSIVITY_SCALE ([M0-D157]).
fn emissivity(c: Cell) -> f32 {
    return 1.0 - exp(-display.glow.y * (1.0 + c.x[ID_H]) * c.sigma);
}

fn glow_colour(c: Cell, t_k: f32) -> vec3<f32> {
    let b = (1.0 - exp(-pow(t_k / display.glow.x, 4.0))) * dimming(t_k) * emissivity(c);
    let haze = HAZE * clamp((log2(c.sigma) * 0.30102999566 + 8.0) / 8.0, 0.0, 1.0) * HAZE_GAIN;
    return blackbody(t_k) * b + haze * (1.0 - b);
}

fn element_colour(c: Cell) -> vec3<f32> {
    var lin = vec3<f32>(0.0);
    for (var k = 0u; k < N_SPECIES; k++) {
        lin += c.x[k] * pal.species[k].xyz;
    }
    let visible = clamp((log2(c.sigma) * 0.30102999566 + 6.0) / 8.0, 0.15, 1.0);
    return linear_to_srgb(lin * visible);
}

@compute @workgroup_size(8, 8)
fn render_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }
    let c = blended(id.y * dims.width + id.x, params.alpha);
    var rgb: vec3<f32>;
    switch (params.view) {
        case VIEW_HEAT: {
            rgb = heat_map(log2(to_kelvin(cell_temperature(c))) * 0.30102999566);
        }
        case VIEW_ELEMENT: {
            rgb = element_colour(c);
        }
        case VIEW_DENSITY: {
            rgb = density_map(log2(c.sigma) * 0.30102999566);
        }
        default: {
            rgb = glow_colour(c, to_kelvin(cell_temperature(c)));
        }
    }
    if (c.sigma < eos.sigma_vac) {
        rgb = BG_SPACE;
    }
    textureStore(image, vec2<i32>(i32(id.x), i32(id.y)), vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}

// «Fit to view»'s histogram (M0-T127a, §1.11 [M0-R12]): one pass over the current state counting the cells with Σ ≥ Σ_vac into
// 1,024 bins of the Heat view's log₁₀ T_K or the Density view's log₁₀ Σ (params.view) over [params.lo, params.lo + params.span];
// a cell past the span lands in the end bin. No alpha: the query reads the current state.
@compute @workgroup_size(8, 8)
fn extent_bins(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }
    let c = load(id.y * dims.width + id.x, false);
    if (c.sigma < eos.sigma_vac) {
        return;
    }
    var q: f32;
    if (params.view == VIEW_HEAT) {
        q = log2(to_kelvin(cell_temperature(c))) * 0.30102999566;
    } else {
        q = log2(c.sigma) * 0.30102999566;
    }
    let u = clamp(floor((q - params.lo) / params.span * f32(EXTENT_BINS)), 0.0, f32(EXTENT_BINS - 1u));
    atomicAdd(&bins[u32(u)], 1u);
}
