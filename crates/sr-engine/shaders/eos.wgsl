// The equation of state on the GPU (m0_contrat.md §2.3.1–§2.3.3, §1.5, §2.7) — the helpers every pass shares. A shader
// that needs them is compiled with this file prepended (step/mod.rs, `with_eos`); its own bindings stay in group 0,
// these are group 1, one bind group built once per run (`EosGpu`).
//
// The same formulas as sr_physics::eos, in f32: 1/μ = Σ_i X_i (Z_i + 1)/A_i, Y_e = Σ_i X_i Z_i/A_i and
// Z_met = 1 − X_H − X_He − X_n from the species table (elements.json); the cold pressure
// P(x; K₁, K₂) = 1 / (1/(K₁x²) + 1/(K₂x^{3/2})), written K₁x² / (1 + K₁√x/K₂) so a vanishing x never divides by 0;
// u(x) read from M0-T12's table — ln u at 512 nodes log-spaced over [1e-8, 1e8] and each segment's slope in ln u, taken
// in f64 before the upload (a slope from two f32 ln u near 30 would lose 5 digits), uploaded once, interpolated
// linearly in (ln x, ln u), the power law K₁x² below the table and the last segment's slope above it;
// ε_cold = (u_e + u_n)/Σ;
// Π = Σ ε_th + Π_e + Π_n; c² = 2Π/Σ; T = μ ε_th, floored at T_floor.

struct EosParams {
    // Per species in §1.5's order, four to a vec4 (10 used): (Z + 1)/A and Z/A.
    inv_mu_coef: array<vec4<f32>, 3>,
    y_e_coef: array<vec4<f32>, 3>,
    // The cold pairs; a pair that is off has both at 0 and its `cold_*` flag at 0.
    k1e: f32,
    k2e: f32,
    k1n: f32,
    k2n: f32,
    // The table's geometry: ln X_MIN and 1/Δ(ln x).
    ln_x_min: f32,
    inv_dln_x: f32,
    // The floors (§2.7).
    t_floor: f32,
    sigma_floor: f32,
    sigma_vac: f32,
    cold_e: u32,
    cold_n: u32,
    table_points: u32,
}

@group(1) @binding(0) var<uniform> eos: EosParams;
// Per table — the electrons', then neutron matter's — ln u at the `table_points` nodes, then the slope
// ln u[i + 1] − ln u[i] of each segment (the last entry unused).
@group(1) @binding(1) var<storage, read> eos_table: array<f32>;

const N_SPECIES: u32 = 10u;
const ID_H: u32 = 0u;
const ID_HE: u32 = 1u;
const ID_N: u32 = 9u;
const TABLE_E: u32 = 0u;
const TABLE_N: u32 = 1u;

struct Composition {
    inv_mu: f32,
    y_e: f32,
    z_met: f32,
}

// 1/μ, Y_e and Z_met of mass fractions `x` (§1.5's order, summing to 1).
fn composition(x: array<f32, 10>) -> Composition {
    var inv_mu = 0.0;
    var y_e = 0.0;
    for (var i = 0u; i < N_SPECIES; i++) {
        inv_mu += x[i] * eos.inv_mu_coef[i / 4u][i % 4u];
        y_e += x[i] * eos.y_e_coef[i / 4u][i % 4u];
    }
    return Composition(inv_mu, y_e, 1.0 - x[ID_H] - x[ID_HE] - x[ID_N]);
}

// The cold pressure P(x; K₁, K₂) (§2.3.2); 0 for x ≤ 0.
fn cold_pressure(x: f32, k1: f32, k2: f32) -> f32 {
    if (x <= 0.0) {
        return 0.0;
    }
    return k1 * x * x / (1.0 + k1 * sqrt(x) / k2);
}

// u(x) of table `t` (TABLE_E or TABLE_N), interpolated log-log as ColdTable::u; 0 for x ≤ 0.
fn cold_u(t: u32, x: f32) -> f32 {
    if (x <= 0.0) {
        return 0.0;
    }
    let base = 2u * t * eos.table_points;
    let d = log(x) - eos.ln_x_min;
    let pos = d * eos.inv_dln_x;
    if (pos < 0.0) {
        return exp(eos_table[base] + 2.0 * d);
    }
    let i = min(u32(floor(pos)), eos.table_points - 2u);
    let frac = pos - f32(i);
    return exp(eos_table[base + i] + frac * eos_table[base + eos.table_points + i]);
}

// Π_e = P(Y_eΣ; K₁ₑ, K₂ₑ) and Π_n = P(X_nΣ; K₁ₙ, K₂ₙ), 0 with the pair off.
fn pi_e(sigma: f32, y_e: f32) -> f32 {
    if (eos.cold_e == 0u) {
        return 0.0;
    }
    return cold_pressure(y_e * sigma, eos.k1e, eos.k2e);
}

fn pi_n(sigma: f32, x_n: f32) -> f32 {
    if (eos.cold_n == 0u) {
        return 0.0;
    }
    return cold_pressure(x_n * sigma, eos.k1n, eos.k2n);
}

// ε_cold = (u_e + u_n)/Σ, per unit mass (§2.3.2).
fn eps_cold(sigma: f32, y_e: f32, x_n: f32) -> f32 {
    var u = 0.0;
    if (eos.cold_e != 0u) {
        u += cold_u(TABLE_E, y_e * sigma);
    }
    if (eos.cold_n != 0u) {
        u += cold_u(TABLE_N, x_n * sigma);
    }
    return u / sigma;
}

// Π = Π_th + Π_e + Π_n, Π_th = (γ − 1) Σ ε_th = Σ ε_th (§2.3.1, §2.3.3).
fn pressure(sigma: f32, eps_th: f32, y_e: f32, x_n: f32) -> f32 {
    return sigma * eps_th + pi_e(sigma, y_e) + pi_n(sigma, x_n);
}

// c² = 2Π/Σ, the wave-speed bound for fluxes and Δt (§2.3.3).
fn wave_speed_sq(sigma: f32, pi: f32) -> f32 {
    return 2.0 * pi / sigma;
}

// T = μ ε_th (§2.3.1).
fn temperature(eps_th: f32, inv_mu: f32) -> f32 {
    return eps_th / inv_mu;
}

// The floor of ε_th, T_floor/μ (§2.3.3).
fn eps_th_floor(inv_mu: f32) -> f32 {
    return eos.t_floor * inv_mu;
}

// ε_th = E/Σ − ½|u|² − ε_cold from the state, before any floor (§2.3.3).
fn eps_th_raw(sigma: f32, mom: vec2<f32>, energy: f32, y_e: f32, x_n: f32) -> f32 {
    let vel = mom / sigma;
    return energy / sigma - 0.5 * dot(vel, vel) - eps_cold(sigma, y_e, x_n);
}
