//! The equation of state (m0_contrat.md §2.3.1–§2.3.3): the ideal gas in the plane (γ = 2), the cold pressure of a 2D
//! Fermi gas for electrons and neutron matter, and the tabulated energy per area u(x) the GPU samples the same way
//! (M0-T19). Sandbox units (§2.1): Π_th = Σ ε_th, T = μ ε_th.
//!
//! The cold pressure blends the two limits of a degenerate gas in two dimensions (SM-E4, SM-E5): P ~ K₁x² when
//! non-relativistic, P ~ K₂x^{3/2} when ultra-relativistic. Its energy per area is u(x) = x ∫₀^x P(s)/s² ds, which tends
//! to K₁x² and 2K₂x^{3/2} in the two limits. A K pair at 0 means no cold pressure (the scenes' `overrides` for Sod and
//! the cold collapse, §2.12.1); a pair is on or off as a whole (physics.json enforces it too).

use crate::registry::{Physics, RegistryError};

/// The ultra-relativistic exponent of the cold pressure, P ~ K₂ x^{3/2} (SM-E5).
const P_UR_EXPONENT: f64 = 1.5;
/// Points of the u(x) table (§2.3.2).
pub const TABLE_POINTS: usize = 512;
/// The table's range in x = Y_eΣ or X_nΣ (§2.3.2).
pub const X_MIN: f64 = 1e-8;
pub const X_MAX: f64 = 1e8;
/// Simpson sub-intervals per table segment when the table is built (the integrand is smooth in t = √s; the table is
/// built once).
const QUAD_STEPS: usize = 64;

/// The cold pressure P(x; K₁, K₂) = 1 / (1/(K₁x²) + 1/(K₂x^{3/2})) (§2.3.2). Zero for x ≤ 0 or a pair that is off.
pub fn cold_pressure(x: f64, k1: f64, k2: f64) -> f64 {
    if x <= 0.0 || k1 <= 0.0 || k2 <= 0.0 {
        return 0.0;
    }
    1.0 / (1.0 / (k1 * x * x) + 1.0 / (k2 * x.powf(P_UR_EXPONENT)))
}

/// Simpson's rule for ∫₀^t of P(s)/s² · 2t dt (s = t²), over `[t0, t1]` with `n` (even) sub-intervals. The integrand is
/// 1/(1/K₁ + s^{1/2}/K₂) · 2t when the exponent is 3/2: smooth in t, with no singularity at 0.
fn quad_segment(k1: f64, k2: f64, t0: f64, t1: f64, n: usize) -> f64 {
    let f = |t: f64| {
        if t <= 0.0 {
            0.0
        } else {
            let s = t * t;
            cold_pressure(s, k1, k2) / (s * s) * 2.0 * t
        }
    };
    let h = (t1 - t0) / n as f64;
    let mut sum = f(t0) + f(t1);
    for i in 1..n {
        sum += f(t0 + h * i as f64) * if i % 2 == 1 { 4.0 } else { 2.0 };
    }
    sum * h / 3.0
}

/// u(x) = x ∫₀^x P(s)/s² ds in closed form, for the P with exponent 3/2 (the blend integrates exactly: with
/// y = (K₁/K₂)√x, u = x · 2(K₂²/K₁) (y − ln(1 + y))). The table's check, never its source. 0 when the pair is off.
pub fn cold_energy_exact(x: f64, k1: f64, k2: f64) -> f64 {
    if x <= 0.0 || k1 <= 0.0 || k2 <= 0.0 {
        return 0.0;
    }
    let y = k1 / k2 * x.sqrt();
    // y − ln(1 + y) cancels for small y: use its series there.
    let g = if y < 1e-3 {
        // Σ_{k≥2} (−1)^k y^k / k, to y⁹ (the next term is below 1e-27 of the first)
        (2..10).map(|k| (if k % 2 == 0 { 1.0 } else { -1.0 }) * y.powi(k) / k as f64).sum::<f64>()
    } else {
        y - y.ln_1p()
    };
    x * 2.0 * k2 * k2 / k1 * g
}

/// u(x) tabulated on [`TABLE_POINTS`] points log-spaced over [[`X_MIN`], [`X_MAX`]] and interpolated linearly in
/// (ln x, ln u) — the table the GPU uploads and reads the same way (§2.3.2). Below [`X_MIN`] u follows the
/// non-relativistic limit's power law K₁x² (exponent 2); above [`X_MAX`] the last segment's log-slope continues.
#[derive(Debug, Clone, PartialEq)]
pub struct ColdTable {
    k1: f64,
    k2: f64,
    /// ln u at the nodes.
    ln_u: Vec<f64>,
}

impl ColdTable {
    /// The table of one K pair, which must be on (both > 0).
    pub fn build(k1: f64, k2: f64) -> ColdTable {
        assert!(k1 > 0.0 && k2 > 0.0, "a K pair that is off has no table");
        let n = TABLE_POINTS;
        let mut ln_u = Vec::with_capacity(n);
        // J(x_i) = ∫₀^{x_i} P(s)/s² ds, accumulated segment by segment in t = √s.
        let mut t_prev = 0.0;
        let mut j = 0.0;
        for i in 0..n {
            let t = Self::node_x(i).sqrt();
            let steps = if i == 0 { QUAD_STEPS * 8 } else { QUAD_STEPS };
            j += quad_segment(k1, k2, t_prev, t, steps);
            ln_u.push((Self::node_x(i) * j).ln());
            t_prev = t;
        }
        ColdTable { k1, k2, ln_u }
    }

    /// Node i's x, log-spaced: X_MIN at 0, X_MAX at the last.
    pub fn node_x(i: usize) -> f64 {
        let frac = i as f64 / (TABLE_POINTS - 1) as f64;
        (X_MIN.ln() + frac * (X_MAX.ln() - X_MIN.ln())).exp()
    }

    /// The log-spacing between nodes, Δ ln x.
    pub fn step() -> f64 {
        (X_MAX.ln() - X_MIN.ln()) / (TABLE_POINTS - 1) as f64
    }

    /// The table's u at node i (what the GPU uploads).
    pub fn node_u(&self, i: usize) -> f64 {
        self.ln_u[i].exp()
    }

    /// ln u at every node.
    pub fn ln_u(&self) -> &[f64] {
        &self.ln_u
    }

    /// u(x), interpolated log-log; 0 for x ≤ 0.
    pub fn u(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let pos = (x.ln() - X_MIN.ln()) / Self::step();
        if pos < 0.0 {
            return self.ln_u[0].exp() * (x / X_MIN).powi(2);
        }
        let i = (pos.floor().max(0.0) as usize).min(TABLE_POINTS - 2);
        let frac = pos - i as f64;
        (self.ln_u[i] + frac * (self.ln_u[i + 1] - self.ln_u[i])).exp()
    }

    pub fn k1(&self) -> f64 {
        self.k1
    }

    pub fn k2(&self) -> f64 {
        self.k2
    }
}

/// The thermal energy recovered from E, and what the floor added (§2.3.3, §2.9 `floor_added`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thermal {
    /// ε_th per unit mass, at least T_floor/μ.
    pub eps_th: f64,
    /// Energy per area the floor put back, Σ · (floor − raw), 0 when the cell was above it.
    pub floor_added: f64,
}

/// The equation of state of one run: the two cold pairs' tables and T_floor.
#[derive(Debug, Clone, PartialEq)]
pub struct Eos {
    electrons: Option<ColdTable>,
    neutrons: Option<ColdTable>,
    t_floor: f64,
}

fn eos_err<T>(key: &str, message: String) -> Result<T, RegistryError> {
    Err(RegistryError { file: "physics.json", key: key.to_string(), message })
}

fn pair(k1_key: &str, k1: f64, k2_key: &str, k2: f64) -> Result<Option<ColdTable>, RegistryError> {
    for (key, k) in [(k1_key, k1), (k2_key, k2)] {
        if !k.is_finite() || k < 0.0 {
            return eos_err(key, format!("must be finite and ≥ 0, got {k}"));
        }
    }
    match (k1 > 0.0, k2 > 0.0) {
        (false, false) => Ok(None),
        (true, true) => Ok(Some(ColdTable::build(k1, k2))),
        (false, true) => eos_err(k1_key, format!("must be > 0 while {k2_key} is (a pair is on or off)")),
        (true, false) => eos_err(k2_key, format!("must be > 0 while {k1_key} is (a pair is on or off)")),
    }
}

impl Eos {
    /// An equation of state from raw constants; a K of 0 (both of a pair) turns that cold pressure off.
    pub fn new(k1e: f64, k2e: f64, k1n: f64, k2n: f64, t_floor: f64) -> Result<Eos, RegistryError> {
        if !t_floor.is_finite() || t_floor <= 0.0 {
            return eos_err("t_floor", format!("must be > 0, got {t_floor}"));
        }
        Ok(Eos {
            electrons: pair("k1e", k1e, "k2e", k2e)?,
            neutrons: pair("k1n", k1n, "k2n", k2n)?,
            t_floor,
        })
    }

    /// The equation of state of a run's constants (a scene's `overrides` included).
    pub fn from_physics(physics: &Physics) -> Result<Eos, RegistryError> {
        let get = |k: &str| physics.get(k).expect("physics.json key");
        Eos::new(get("k1e"), get("k2e"), get("k1n"), get("k2n"), get("t_floor"))
    }

    pub fn t_floor(&self) -> f64 {
        self.t_floor
    }

    pub fn electron_table(&self) -> Option<&ColdTable> {
        self.electrons.as_ref()
    }

    pub fn neutron_table(&self) -> Option<&ColdTable> {
        self.neutrons.as_ref()
    }

    /// Π_th = (γ − 1) Σ ε_th = Σ ε_th (§2.3.1).
    pub fn pi_th(sigma: f64, eps_th: f64) -> f64 {
        sigma * eps_th
    }

    /// T = μ ε_th (§2.3.1).
    pub fn temperature(eps_th: f64, mu: f64) -> f64 {
        mu * eps_th
    }

    /// ε_th = T/μ, the inverse of [`Eos::temperature`].
    pub fn eps_th_of_t(t: f64, mu: f64) -> f64 {
        t / mu
    }

    /// Π_e = P(Y_eΣ; K₁ₑ, K₂ₑ), 0 with the pair off.
    pub fn pi_e(&self, sigma: f64, y_e: f64) -> f64 {
        self.electrons.as_ref().map_or(0.0, |t| cold_pressure(y_e * sigma, t.k1, t.k2))
    }

    /// Π_n = P(X_nΣ; K₁ₙ, K₂ₙ), 0 with the pair off.
    pub fn pi_n(&self, sigma: f64, x_n: f64) -> f64 {
        self.neutrons.as_ref().map_or(0.0, |t| cold_pressure(x_n * sigma, t.k1, t.k2))
    }

    /// Π = Π_th + Π_e + Π_n (§2.3.3).
    pub fn pi(&self, sigma: f64, eps_th: f64, y_e: f64, x_n: f64) -> f64 {
        Self::pi_th(sigma, eps_th) + self.pi_e(sigma, y_e) + self.pi_n(sigma, x_n)
    }

    /// ε_cold = (u_e + u_n)/Σ, the cold energy per unit mass (§2.3.2).
    pub fn eps_cold(&self, sigma: f64, y_e: f64, x_n: f64) -> f64 {
        let u_e = self.electrons.as_ref().map_or(0.0, |t| t.u(y_e * sigma));
        let u_n = self.neutrons.as_ref().map_or(0.0, |t| t.u(x_n * sigma));
        (u_e + u_n) / sigma
    }

    /// c² = 2Π/Σ, the wave-speed bound for fluxes and Δt (§2.3.3).
    pub fn wave_speed_sq(sigma: f64, pi: f64) -> f64 {
        2.0 * pi / sigma
    }

    /// ε_th = E/Σ − ½|u|² − ε_cold from the state's energy and momentum, floored at T_floor/μ; the floor's energy is
    /// returned to be booked (§2.3.3, §2.9).
    pub fn thermal(&self, energy: f64, sigma: f64, mom: [f64; 2], y_e: f64, x_n: f64, mu: f64) -> Thermal {
        let kinetic = 0.5 * (mom[0] * mom[0] + mom[1] * mom[1]) / (sigma * sigma);
        let raw = energy / sigma - kinetic - self.eps_cold(sigma, y_e, x_n);
        let floor = self.t_floor / mu;
        if raw < floor {
            Thermal { eps_th: floor, floor_added: sigma * (floor - raw) }
        } else {
            Thermal { eps_th: raw, floor_added: 0.0 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped constants (§2.7): electrons 21.3 / 134, neutron matter 2.67 / 58.2, T_floor 0.05.
    const SHIPPED_PAIRS: [(f64, f64); 2] = [(21.3, 134.0), (2.67, 58.2)];

    fn rel(a: f64, b: f64) -> f64 {
        ((a - b) / b).abs()
    }

    fn shipped() -> Eos {
        Eos::from_physics(&Physics::shipped().expect("shipped physics.json")).expect("shipped constants")
    }

    /// The table's constants are the shipped ones, so a drift in physics.json shows here.
    #[test]
    fn shipped_pairs_are_the_contracts() {
        let physics = Physics::shipped().unwrap();
        for (key, v) in [("k1e", 21.3), ("k2e", 134.0), ("k1n", 2.67), ("k2n", 58.2), ("t_floor", 0.05)] {
            assert_eq!(physics.get(key), Some(v), "{key}");
        }
        let eos = shipped();
        assert_eq!((eos.electron_table().unwrap().k1(), eos.electron_table().unwrap().k2()), (21.3, 134.0));
        assert_eq!((eos.neutron_table().unwrap().k1(), eos.neutron_table().unwrap().k2()), (2.67, 58.2));
        assert_eq!(eos.t_floor(), 0.05);
    }

    /// P's two limits (SM-E4: K₁x², SM-E5: K₂x^{3/2}) within 1e-6 relative, far enough out that the blend has
    /// converged: P/(K₁x²) = 1/(1 + K₁√x/K₂) exactly, so the deviation at x is K₁√x/K₂ and at the far end K₂/(K₁√x).
    #[test]
    fn cold_pressure_limits() {
        for (k1, k2) in SHIPPED_PAIRS {
            let lo = 1e-12;
            assert!(rel(cold_pressure(lo, k1, k2), k1 * lo * lo) < 1e-6, "x² limit, K = {k1}/{k2}");
            let hi = 1e16;
            assert!(rel(cold_pressure(hi, k1, k2), k2 * hi.powf(1.5)) < 1e-6, "x^3/2 limit, K = {k1}/{k2}");
            // The blend itself, between the limits: P = K₁x² / (1 + K₁√x/K₂), exactly.
            for x in [1e-6, 1e-2, 1.0, 30.0, 1e4] {
                let want = k1 * x * x / (1.0 + k1 * x.sqrt() / k2);
                assert!(rel(cold_pressure(x, k1, k2), want) < 1e-12, "blend at x = {x}");
            }
        }
    }

    /// The table's source (quadrature of P) against the integral in closed form, at every node: the table is the
    /// integral to ~1e-10, not just a curve that looks right.
    #[test]
    fn table_nodes_equal_the_closed_form_integral() {
        for (k1, k2) in SHIPPED_PAIRS.into_iter().chain([(1.0, 100.0), (100.0, 1.0), (7.0, 7.0)]) {
            let table = ColdTable::build(k1, k2);
            let worst = (0..TABLE_POINTS)
                .map(|i| rel(table.node_u(i), cold_energy_exact(ColdTable::node_x(i), k1, k2)))
                .fold(0.0, f64::max);
            assert!(worst < 1e-9, "K = {k1}/{k2}: worst node error {worst:e}");
        }
    }

    /// u(x) against both limits' closed forms: K₁x² (SM-E4: energy = pressure in 2D) and 2K₂x^{3/2} (SM-E5: energy =
    /// twice the pressure), within 1e-4. The blend converges to the ultra-relativistic form like ln(x)/√x, so with
    /// the shipped K's it does so only far beyond the table; the table's own ends are checked with pairs whose
    /// transition lies inside it, and the shipped pairs with the exact integral beyond the table.
    #[test]
    fn energy_limits() {
        // K₂ ≪ K₁ puts the transition at the low end, so the high end is ultra-relativistic.
        let ur = ColdTable::build(100.0, 1.0);
        for x in [1e8, 1e7] {
            assert!(rel(ur.u(x), 2.0 * 1.0 * x.powf(1.5)) < 1e-4, "x^3/2 limit at {x}");
        }
        // K₂ ≫ K₁ puts it at the high end, so the low end is non-relativistic.
        let nr = ColdTable::build(1.0, 100.0);
        for x in [1e-8, 1e-6] {
            assert!(rel(nr.u(x), 1.0 * x * x) < 1e-4, "x² limit at {x}");
        }
        for (k1, k2) in SHIPPED_PAIRS {
            assert!(rel(cold_energy_exact(1e-12, k1, k2), k1 * 1e-24) < 1e-6, "x² limit, K = {k1}/{k2}");
            assert!(rel(cold_energy_exact(1e20, k1, k2), 2.0 * k2 * 1e30) < 1e-6, "x^3/2 limit, K = {k1}/{k2}");
            // and the shipped table's low end, which is inside its range
            let table = ColdTable::build(k1, k2);
            assert!(rel(table.u(1e-8), k1 * 1e-16) < 1e-4, "table x² limit, K = {k1}/{k2}");
        }
    }

    /// The closed form against an independent quadrature of P (the one that is not the table's own).
    #[test]
    fn closed_form_matches_quadrature() {
        for (k1, k2) in SHIPPED_PAIRS {
            for x in [1e-7_f64, 1e-3, 0.5, 20.0, 5e3, 3e7] {
                let t = x.sqrt();
                let j = quad_segment(k1, k2, 0.0, t, 20_000);
                assert!(rel(x * j, cold_energy_exact(x, k1, k2)) < 1e-9, "x = {x}, K = {k1}/{k2}");
            }
        }
    }

    /// A table accurate at its nodes and wrong between them: every midpoint (and the third-points) of every segment,
    /// for the shipped pairs and a sweep of transition positions, within 1e-4 of the exact integral.
    #[test]
    fn interpolation_between_nodes() {
        let mut pairs = SHIPPED_PAIRS.to_vec();
        pairs.extend([(1.0, 1e-2), (1.0, 1.0), (1.0, 1e2), (1e3, 1.0), (1.0, 1e3)]);
        for (k1, k2) in pairs {
            let table = ColdTable::build(k1, k2);
            let mut worst: f64 = 0.0;
            for i in 0..TABLE_POINTS - 1 {
                for frac in [1.0 / 3.0, 0.5, 2.0 / 3.0] {
                    let x = (ColdTable::node_x(i).ln() + frac * ColdTable::step()).exp();
                    worst = worst.max(rel(table.u(x), cold_energy_exact(x, k1, k2)));
                }
            }
            assert!(worst <= 1e-4, "K = {k1}/{k2}: worst interpolation error {worst:e}");
        }
    }

    /// Nodes are log-spaced over [1e-8, 1e8] on exactly 512 points; the table returns its node values at the nodes.
    #[test]
    fn table_geometry() {
        assert_eq!(TABLE_POINTS, 512);
        assert!(rel(ColdTable::node_x(0), 1e-8) < 1e-12);
        assert!(rel(ColdTable::node_x(TABLE_POINTS - 1), 1e8) < 1e-12);
        let ratio = ColdTable::node_x(1) / ColdTable::node_x(0);
        for i in 1..TABLE_POINTS {
            assert!(rel(ColdTable::node_x(i) / ColdTable::node_x(i - 1), ratio) < 1e-9, "uniform in ln x at {i}");
        }
        let table = ColdTable::build(21.3, 134.0);
        assert_eq!(table.ln_u().len(), TABLE_POINTS);
        for i in [0, 100, 255, 511] {
            assert!(rel(table.u(ColdTable::node_x(i)), table.node_u(i)) < 1e-9, "node {i}");
        }
    }

    /// Monotonicity: P and u rise with x everywhere, the table's nodes and a fine sweep between them; u stays positive
    /// and finite; P is below both of its limits (the blend is their harmonic combination).
    #[test]
    fn monotone() {
        for (k1, k2) in SHIPPED_PAIRS {
            let table = ColdTable::build(k1, k2);
            for i in 1..TABLE_POINTS {
                assert!(table.ln_u()[i] > table.ln_u()[i - 1], "node {i} (K = {k1}/{k2})");
            }
            let (mut p_prev, mut u_prev) = (0.0, 0.0);
            for j in 0..20_000 {
                let x = X_MIN * (X_MAX / X_MIN).powf(j as f64 / 19_999.0);
                let (p, u) = (cold_pressure(x, k1, k2), table.u(x));
                assert!(p > p_prev && u > u_prev && u.is_finite(), "x = {x:e} (K = {k1}/{k2})");
                assert!(p < k1 * x * x && p < k2 * x.powf(1.5), "P below both limits at {x:e}");
                (p_prev, u_prev) = (p, u);
            }
        }
    }

    /// The low end: x = 0 and below give 0, and u(x) ~ K₁x² (a power law with exponent 2) continues below the table
    /// rather than the integral diverging.
    #[test]
    fn low_end_is_finite() {
        let table = ColdTable::build(21.3, 134.0);
        assert_eq!(table.u(0.0), 0.0);
        assert_eq!(table.u(-1.0), 0.0);
        assert_eq!(cold_pressure(0.0, 21.3, 134.0), 0.0);
        assert_eq!(cold_energy_exact(0.0, 21.3, 134.0), 0.0);
        for x in [1e-9, 1e-12, 1e-20] {
            let u = table.u(x);
            assert!(u.is_finite() && u > 0.0, "x = {x:e}");
            assert!(rel(u, 21.3 * x * x) < 1e-4, "x² below the table at {x:e}");
        }
        // the closed form's series branch (y < 1e-3) and its direct one agree with the quadrature across the join
        for y in [9.99e-4_f64, 1.001e-3] {
            let x = y * y; // K₁ = K₂ = 1, so y = √x
            let j = quad_segment(1.0, 1.0, 0.0, x.sqrt(), 2_000);
            assert!(rel(x * j, cold_energy_exact(x, 1.0, 1.0)) < 1e-9, "y = {y}");
        }
    }

    /// A K of 0 is no cold pressure: no table, zero Π and ε_cold; a half-on pair or a negative K is refused.
    #[test]
    fn zero_k_means_no_cold_pressure() {
        let eos = Eos::new(0.0, 0.0, 0.0, 0.0, 0.05).unwrap();
        assert!(eos.electron_table().is_none() && eos.neutron_table().is_none());
        assert_eq!(eos.pi_e(10.0, 0.5), 0.0);
        assert_eq!(eos.pi_n(10.0, 1.0), 0.0);
        assert_eq!(eos.eps_cold(10.0, 0.5, 1.0), 0.0);
        assert_eq!(eos.pi(10.0, 0.3, 0.5, 1.0), 3.0, "only the thermal pressure is left");
        assert_eq!(cold_pressure(5.0, 0.0, 134.0), 0.0);
        // electrons alone (the cold-collapse overrides turn neutron matter off, or the reverse)
        let e_only = Eos::new(21.3, 134.0, 0.0, 0.0, 0.05).unwrap();
        assert!(e_only.pi_e(1.0, 0.5) > 0.0 && e_only.pi_n(1.0, 1.0) == 0.0);
        assert_eq!(Eos::new(21.3, 0.0, 0.0, 0.0, 0.05).unwrap_err().key, "k2e");
        assert_eq!(Eos::new(0.0, 0.0, 0.0, 58.2, 0.05).unwrap_err().key, "k1n");
        assert_eq!(Eos::new(-1.0, 1.0, 0.0, 0.0, 0.05).unwrap_err().key, "k1e");
        assert_eq!(Eos::new(0.0, 0.0, 0.0, 0.0, 0.0).unwrap_err().key, "t_floor");
    }

    /// §2.3.1 and §2.3.3: Π_th = Σε_th, T = με_th (and back), Π = Π_th + Π_e + Π_n, c² = 2Π/Σ.
    #[test]
    fn gas_laws() {
        let eos = shipped();
        let (sigma, eps, mu, y_e, x_n) = (3.5, 0.2, 0.62, 0.5, 0.25);
        assert_eq!(Eos::pi_th(sigma, eps), sigma * eps);
        assert_eq!(Eos::temperature(eps, mu), mu * eps);
        assert!(rel(Eos::eps_th_of_t(Eos::temperature(eps, mu), mu), eps) < 1e-15);
        let pi_e = cold_pressure(y_e * sigma, 21.3, 134.0);
        let pi_n = cold_pressure(x_n * sigma, 2.67, 58.2);
        assert!(rel(eos.pi_e(sigma, y_e), pi_e) < 1e-15 && rel(eos.pi_n(sigma, x_n), pi_n) < 1e-15);
        let pi = eos.pi(sigma, eps, y_e, x_n);
        assert!(rel(pi, sigma * eps + pi_e + pi_n) < 1e-15);
        assert!(rel(Eos::wave_speed_sq(sigma, pi), 2.0 * pi / sigma) < 1e-15);
    }

    /// c² = 2Π/Σ bounds the flux's wave speed because every term's log-slope in Σ is ≤ 2 (§2.3.3): checked on the
    /// pressure terms by a centred difference of ln Π against ln Σ.
    #[test]
    fn pressure_log_slope_is_at_most_two() {
        let eos = shipped();
        let slope = |f: &dyn Fn(f64) -> f64, s: f64| {
            let h = 1e-4;
            (f(s * (1.0 + h)).ln() - f(s * (1.0 - h)).ln()) / ((1.0 + h).ln() - (1.0 - h).ln())
        };
        for s in [1e-4, 0.01, 1.0, 25.0, 1e3, 1e5] {
            assert!((slope(&|s| eos.pi_e(s, 0.5), s) - 2.0).abs() <= 0.5 + 1e-6, "electrons at {s}");
            assert!(slope(&|s| eos.pi_e(s, 0.5), s) <= 2.0 + 1e-9);
            assert!(slope(&|s| eos.pi_n(s, 1.0), s) <= 2.0 + 1e-9, "neutrons at {s}");
            assert!((slope(&|s| Eos::pi_th(s, 0.3), s) - 1.0).abs() < 1e-6, "thermal at fixed ε_th");
        }
    }

    /// ε_th = E/Σ − ½|u|² − ε_cold: E built from a known ε_th comes back; below the floor the floor is returned and
    /// its energy booked as Σ·(floor − raw); exactly at the floor nothing is booked.
    #[test]
    fn thermal_from_energy_and_the_floor() {
        let eos = shipped();
        let (sigma, mom, y_e, x_n, mu) = (4.0, [2.0, -3.0], 0.5, 0.1, 0.6);
        let eps_cold = eos.eps_cold(sigma, y_e, x_n);
        assert!(eps_cold > 0.0);
        let energy = |eps_th: f64| sigma * (eps_th + eps_cold) + 0.5 * (4.0 + 9.0) / sigma;
        let t = eos.thermal(energy(0.7), sigma, mom, y_e, x_n, mu);
        assert!(rel(t.eps_th, 0.7) < 1e-12 && t.floor_added == 0.0);
        let floor = 0.05 / mu;
        let t = eos.thermal(energy(floor), sigma, mom, y_e, x_n, mu);
        assert!(rel(t.eps_th, floor) < 1e-12 && t.floor_added.abs() < 1e-9, "exactly at the floor");
        let t = eos.thermal(energy(-0.4), sigma, mom, y_e, x_n, mu);
        assert_eq!(t.eps_th, floor);
        assert!(rel(t.floor_added, sigma * (floor + 0.4)) < 1e-9, "the floor's energy is booked");
        // the recovered temperature never reads below T_floor
        assert!(Eos::temperature(t.eps_th, mu) >= 0.05 - 1e-15);
    }
}
