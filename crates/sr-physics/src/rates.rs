//! The rate law (m0_contrat.md §1.6.1 as amended by §0.5 [M0-D34]): the specific conversion rate of one registry record,
//!
//!   ω = A · Σ^a · Π_i X_i^{n_i} · f(T) · g(Σ),
//!   f(T) = Σ_k w_k (T/T_k)^{−2/3} exp(τ_k (1 − (T_k/T)^{1/3})), τ_k = 3ν_k + 2, for T ≥ T_thr, else 0,
//!   g(Σ) = 1, or for a gated record ((Y_e Σ)/Σ_g − 1)^p when Y_e Σ > Σ_g, else 0.
//!
//! Each term is Gamow-shaped, the non-resonant charged-particle rate's form: equal to the power law w_k (T/T_k)^ν_k at T_k
//! in value and in slope (d ln f/d ln T = −2/3 + (τ_k/3)(T_k/T)^{1/3}, ν_k there), its exponent falling above T_k and
//! bounded (ln f ≤ τ_k − 2 ln(τ_k/2) − 2, at T = T_k (τ_k/2)³: ≈ 112 for ν = 40) — the power law it replaces reached e^194
//! in G-REF's runaway (M0-D34). [`Omega`] is one record's law, compiled from a [`Record`]: M0-T44b's sub-cycles evaluate it
//! in f64 and M0-T46's shader evaluates the same law from its tables ([`Omega::ln_f_nodes`]) and, above them, by the same
//! direct form. f(T) is tabulated on [`TABLE_POINTS`] = 256 log-spaced temperatures from T_thr up to [`TABLE_SPAN`] · T_thr
//! and interpolated log-log (§1.6.1: "if the interpolation error stays ≤ 10⁻⁴ relative"). Log-log interpolation errs by the
//! curvature of ln f in ln T, −(τ_k/9)(T_k/T)^{1/3} for one term (at most h²/8 times it, h the node spacing: 2.8e-5 for
//! He_burn's ν 40 at T_thr) and more where two terms cross; the span is the widest whose worst midpoint error for the
//! shipped H_burn (ν 4 and 18, the steepest bend of any record) stays inside the budget (7.4e-5 at T ≈ 118). Above the span
//! the direct form is evaluated; below T_thr f is exactly 0, the table never starts below it, so the step at T_thr is never
//! smeared.

use crate::registry::{Gate, Record, N_SPECIES};

/// Points of the f(T) table (§1.6.1).
pub const TABLE_POINTS: usize = 256;
/// The table covers [T_thr, `TABLE_SPAN` · T_thr]; the rest is the direct form (see the module doc).
pub const TABLE_SPAN: f64 = 2.5;
/// The interpolation budget (§1.6.1, G-REF), relative.
pub const TABLE_TOLERANCE: f64 = 1e-4;

/// One record's rate law, ready to evaluate.
#[derive(Debug, Clone, PartialEq)]
pub struct Omega {
    /// The species ids of the inputs and the order n_i of each, in the record's input order.
    inputs: Vec<(usize, f64)>,
    a_sigma: f64,
    a_coef: f64,
    /// (T_k, ν_k, w_k); empty means f ≡ 1.
    terms: Vec<(f64, f64, f64)>,
    t_thr: Option<f64>,
    gate: Option<Gate>,
    /// ln f at the nodes; empty without terms.
    ln_f: Vec<f64>,
}

/// ln of one Gamow-shaped term, ln w − (2/3) u + τ (1 − e^{−u/3}) with u = ln(T/T_k), τ = 3ν + 2 (module doc).
fn term_ln(t_k: f64, nu: f64, w: f64, t: f64) -> f64 {
    let u = (t / t_k).ln();
    w.ln() - 2.0 / 3.0 * u - (3.0 * nu + 2.0) * (-u / 3.0).exp_m1()
}

/// f(T) = Σ_k w_k (T/T_k)^{−2/3} exp(τ_k (1 − (T_k/T)^{1/3})), the direct form, without the threshold.
fn f_sum(terms: &[(f64, f64, f64)], t: f64) -> f64 {
    terms.iter().map(|&(t_k, nu, w)| term_ln(t_k, nu, w, t).exp()).sum()
}

impl Omega {
    /// The law of `record`. A record with terms must carry a threshold (the registry parse enforces both together).
    pub fn new(record: &Record) -> Omega {
        let rate = &record.rate;
        let inputs = record.inputs.iter().zip(&rate.orders).map(|(s, &n)| (s.species, n)).collect();
        let terms: Vec<(f64, f64, f64)> = rate.terms.iter().map(|t| (t.t_k, t.nu, t.w)).collect();
        let mut omega = Omega { inputs, a_sigma: rate.a, a_coef: rate.a_coef, terms, t_thr: rate.t_thr, gate: rate.gate.clone(), ln_f: Vec::new() };
        if let Some(t_thr) = omega.t_thr {
            omega.ln_f = (0..TABLE_POINTS).map(|i| f_sum(&omega.terms, Self::node_t(t_thr, i)).ln()).collect();
        }
        omega
    }

    /// Node i's temperature, log-spaced: T_thr at 0, `TABLE_SPAN` · T_thr at the last.
    pub fn node_t(t_thr: f64, i: usize) -> f64 {
        t_thr * (TABLE_SPAN.ln() * i as f64 / (TABLE_POINTS - 1) as f64).exp()
    }

    /// The log-spacing between nodes, Δ ln T.
    pub fn step() -> f64 {
        TABLE_SPAN.ln() / (TABLE_POINTS - 1) as f64
    }

    /// T_thr, or `None` for a record with f ≡ 1.
    pub fn t_thr(&self) -> Option<f64> {
        self.t_thr
    }

    /// ln f at every node (what the GPU uploads); empty for f ≡ 1.
    pub fn ln_f_nodes(&self) -> &[f64] {
        &self.ln_f
    }

    /// f(T) by the direct form: 0 below T_thr, 1 without terms.
    pub fn f_direct(&self, t: f64) -> f64 {
        match self.t_thr {
            None => 1.0,
            Some(t_thr) if t < t_thr => 0.0,
            Some(_) => f_sum(&self.terms, t),
        }
    }

    /// f(T) through the table: 0 below T_thr, interpolated log-log inside the span, the direct form above it.
    pub fn f(&self, t: f64) -> f64 {
        let Some(t_thr) = self.t_thr else { return 1.0 };
        if t < t_thr {
            return 0.0;
        }
        let pos = (t / t_thr).ln() / Self::step();
        if pos >= (TABLE_POINTS - 1) as f64 {
            return f_sum(&self.terms, t);
        }
        let i = pos.floor() as usize;
        let frac = pos - i as f64;
        (self.ln_f[i] + frac * (self.ln_f[i + 1] - self.ln_f[i])).exp()
    }

    /// g at `y_e_sigma` = Y_e Σ: 1 for an ungated record.
    pub fn g(&self, y_e_sigma: f64) -> f64 {
        match &self.gate {
            None => 1.0,
            Some(g) if y_e_sigma > g.sigma_g => (y_e_sigma / g.sigma_g - 1.0).powf(g.p),
            Some(_) => 0.0,
        }
    }

    fn omega_with(&self, sigma: f64, x: &[f64; N_SPECIES], y_e: f64, f: f64) -> f64 {
        let orders: f64 = self.inputs.iter().map(|&(s, n)| x[s].powf(n)).product();
        self.a_coef * sigma.powf(self.a_sigma) * orders * f * self.g(y_e * sigma)
    }

    /// ω at density `sigma`, fractions `x`, temperature `t`, electrons per nucleon `y_e` (the gate reads Y_e Σ), f from
    /// the table.
    pub fn omega(&self, sigma: f64, x: &[f64; N_SPECIES], t: f64, y_e: f64) -> f64 {
        self.omega_with(sigma, x, y_e, self.f(t))
    }

    /// ω with f by the direct form: the oracle the table is held against.
    pub fn omega_direct(&self, sigma: f64, x: &[f64; N_SPECIES], t: f64, y_e: f64) -> f64 {
        self.omega_with(sigma, x, y_e, self.f_direct(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{Elements, Physics, Reactions, Registry};

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs().max(f64::MIN_POSITIVE)
    }

    fn registry() -> Registry {
        Registry::load().expect("shipped registries load")
    }

    /// M0-TC's ladder (§2.5.1 before §0.5 [M0-D50]), the temperatures the hand values below were written at: T_thr 50, the
    /// crossing 119.5, He_burn's slope at 140, ω at T_k 100 … 420. f reads T/T_k alone, so the law graded at it is the
    /// shipped law; the table's cases (its budget at every node and past the span) read the shipped ladder.
    const TC_LADDER: [(&str, f64); 6] = [("t_h", 100.0), ("t_he", 140.0), ("t_c", 196.0), ("t_ne", 274.0), ("t_o", 300.0), ("t_si", 420.0)];

    /// The registries with [`TC_LADDER`] pinned through `Physics::with` (G-COLL's pattern), every other constant shipped.
    fn pinned() -> Registry {
        let elements = Elements::shipped().unwrap();
        let physics = Physics::shipped().unwrap().with(&TC_LADDER).expect("M0-TC's ladder passes K6");
        let reactions = Reactions::shipped(&elements, &physics).unwrap();
        Registry { elements, physics, reactions }
    }

    fn law(r: &Registry, id: &str) -> Omega {
        Omega::new(r.reactions.get(id).unwrap_or_else(|| panic!("record {id}")))
    }

    /// Fractions with `species` at `value` and nothing else (the rest is not read by a law that does not name it).
    fn x_of(r: &Registry, parts: &[(&str, f64)]) -> [f64; N_SPECIES] {
        let mut x = [0.0; N_SPECIES];
        for &(key, v) in parts {
            x[r.elements.index_of(key).unwrap_or_else(|| panic!("species {key}"))] = v;
        }
        x
    }

    /// One Gamow-shaped term written out from §0.5 [M0-D34]: (T/T_k)^{−2/3} exp(τ (1 − (T_k/T)^{1/3})), τ = 3ν + 2.
    fn gamow(t: f64, t_k: f64, nu: f64) -> f64 {
        (t / t_k).powf(-2.0 / 3.0) * ((3.0 * nu + 2.0) * (1.0 - (t_k / t).cbrt())).exp()
    }

    /// Records with a temperature dependence.
    fn burning(r: &Registry) -> Vec<&Record> {
        r.reactions.records().iter().filter(|rec| !rec.rate.terms.is_empty()).collect()
    }

    /// 256 nodes, log-spaced from T_thr over the span, the table's own values at the nodes.
    #[test]
    fn table_geometry() {
        assert_eq!(TABLE_POINTS, 256);
        let r = pinned();
        let h = law(&r, "H_burn");
        let t_thr = h.t_thr().expect("H_burn has a threshold");
        assert!(rel(t_thr, 50.0) < 1e-12, "T_thr = 0.5 T_H");
        assert_eq!(h.ln_f_nodes().len(), TABLE_POINTS);
        assert!(rel(Omega::node_t(t_thr, 0), t_thr) < 1e-12);
        assert!(rel(Omega::node_t(t_thr, TABLE_POINTS - 1), TABLE_SPAN * t_thr) < 1e-12);
        let ratio = Omega::node_t(t_thr, 1) / Omega::node_t(t_thr, 0);
        for i in 1..TABLE_POINTS {
            assert!(rel(Omega::node_t(t_thr, i) / Omega::node_t(t_thr, i - 1), ratio) < 1e-9, "uniform in ln T at {i}");
        }
        let n = law(&r, "N_Fe");
        assert!(n.ln_f_nodes().is_empty() && n.t_thr().is_none(), "f ≡ 1 has no table");
    }

    /// The table against the direct form at every node and at every segment's midpoint and third-points, for every
    /// record with terms. H_burn's two terms make the midpoints bend; the budget is 1e-4 and the worst is recorded. One
    /// Gamow term errs by its curvature alone: ln f's second derivative in ln T is −(τ/9)(T_k/T)^{1/3}, largest at T_thr =
    /// T_k/2, so the worst midpoint reads h²/8 · (τ/9) · 2^{1/3} (h the node spacing; 2.76e-5 for ν 40) — asserted from
    /// both sides, so the bound is the error's, not a loose ceiling.
    #[test]
    fn table_against_direct_form_at_nodes_and_midpoints() {
        let r = registry();
        for rec in burning(&r) {
            let omega = Omega::new(rec);
            let t_thr = omega.t_thr().expect("threshold");
            let (mut worst_node, mut worst_between): (f64, f64) = (0.0, 0.0);
            for i in 0..TABLE_POINTS {
                let t = Omega::node_t(t_thr, i);
                worst_node = worst_node.max(rel(omega.f(t), omega.f_direct(t)));
            }
            for i in 0..TABLE_POINTS - 1 {
                for frac in [1.0 / 3.0, 0.5, 2.0 / 3.0] {
                    let t = t_thr * ((i as f64 + frac) * Omega::step()).exp();
                    worst_between = worst_between.max(rel(omega.f(t), omega.f_direct(t)));
                }
            }
            assert!(worst_node < 1e-12, "{}: at the nodes {worst_node:e}", rec.id);
            assert!(worst_between <= TABLE_TOLERANCE, "{}: between nodes {worst_between:e}", rec.id);
            if rec.rate.terms.len() == 1 {
                let tau = 3.0 * rec.rate.terms[0].nu + 2.0;
                let bound = Omega::step().powi(2) / 8.0 * tau / 9.0 * 2f64.cbrt();
                assert!(worst_between <= bound * 1.01 && worst_between >= bound * 0.9, "{}: one term's worst {worst_between:e}, curvature {bound:e}", rec.id);
            }
        }
    }

    /// The check is not vacuous: H_burn's interpolation does err between its nodes (two power laws, ν 4 and 18), and the
    /// error sits near its bend, where the terms cross — measured, not assumed.
    #[test]
    fn two_terms_are_not_exact() {
        let r = pinned();
        let h = law(&r, "H_burn");
        let t_thr = h.t_thr().unwrap();
        let mut worst = (0.0_f64, 0.0_f64);
        for i in 0..TABLE_POINTS - 1 {
            let t = t_thr * ((i as f64 + 0.5) * Omega::step()).exp();
            let e = rel(h.f(t), h.f_direct(t));
            if e > worst.0 {
                worst = (e, t);
            }
        }
        assert!(worst.0 > 1e-6, "the midpoints should show the bend, worst {:e}", worst.0);
        assert!(worst.0 <= TABLE_TOLERANCE, "worst {:e} at T = {}", worst.0, worst.1);
        // The terms cross where their logs agree: −(2/3) ln(T/100) + 14 (1 − (100/T)^{1/3}) = −(2/3) ln(T/115) + 56 (1 −
        // (115/T)^{1/3}), so T^{−1/3} (56 · 115^{1/3} − 14 · 100^{1/3}) = 42 + (2/3) ln 1.15 (hand-solved): T ≈ 119.5.
        let y = (42.0 + 2.0 / 3.0 * 1.15_f64.ln()) / (56.0 * 115_f64.cbrt() - 14.0 * 100_f64.cbrt());
        let crossing = y.powi(-3);
        assert!((crossing - 119.5).abs() < 0.05 && rel(law(&r, "H_burn").f_direct(crossing), 2.0 * gamow(crossing, 100.0, 4.0)) < 1e-9);
        assert!((worst.1 - crossing).abs() < 5.0, "the worst midpoint {} is not at the crossing {crossing}", worst.1);
        eprintln!("H_burn worst midpoint error {:e} at T = {:.3}", worst.0, worst.1);
    }

    /// f = 0 below T_thr, the direct form at and above it; the table is not smeared across the step.
    #[test]
    fn f_is_zero_below_the_threshold() {
        let r = pinned();
        for rec in burning(&r) {
            let omega = Omega::new(rec);
            let t_thr = omega.t_thr().unwrap();
            for t in [0.0, 0.05, 0.5 * t_thr, t_thr * (1.0 - 1e-9), t_thr.next_down()] {
                assert_eq!(omega.f(t), 0.0, "{} at T = {t}", rec.id);
                assert_eq!(omega.f_direct(t), 0.0, "{} direct at T = {t}", rec.id);
            }
            assert!(omega.f(t_thr) > 0.0, "{}: f(T_thr) is the step's top, not 0", rec.id);
            assert!(rel(omega.f(t_thr), omega.f_direct(t_thr)) < 1e-12, "{} at T_thr", rec.id);
            let just_above = t_thr * (1.0 + 1e-9);
            assert!(rel(omega.f(just_above), omega.f_direct(just_above)) < TABLE_TOLERANCE, "{} just above T_thr", rec.id);
        }
        // H_burn's top of the step is 0.5^{−2/3} e^{14 (1 − 2^{1/3})} + (50/115)^{−2/3} e^{56 (1 − 2.3^{1/3})} ≈ 0.0417 by hand.
        let h = law(&r, "H_burn");
        let hand = 0.5_f64.powf(-2.0 / 3.0) * (14.0 * (1.0 - 2_f64.cbrt())).exp() + (50.0_f64 / 115.0).powf(-2.0 / 3.0) * (56.0 * (1.0 - 2.3_f64.cbrt())).exp();
        assert!((hand - 0.041719).abs() < 1e-6, "{hand}");
        assert!(rel(h.f(50.0), hand) < 1e-12);
    }

    /// Above the table's span the direct form answers: continuous at the edge, exact beyond, no extrapolated slope.
    #[test]
    fn above_the_span_is_the_direct_form() {
        let r = registry();
        let h = law(&r, "H_burn");
        let top = TABLE_SPAN * h.t_thr().unwrap();
        for t in [top * (1.0 - 1e-9), top, top * (1.0 + 1e-9), 1.5 * top, 4.0 * top, 1e3] {
            assert!(rel(h.f(t), h.f_direct(t)) <= TABLE_TOLERANCE, "T = {t}");
        }
        for t in [1.5 * top, 4.0 * top, 1e3] {
            assert_eq!(h.f(t), h.f_direct(t), "beyond the span, T = {t}");
        }
    }

    /// g on both sides of Σ_g for N_Fe (gate on Y_eΣ, Σ_g = Σ_N = 160, p = 1): 0 at and below, (Y_eΣ/Σ_g − 1)^p above;
    /// p = 2 from a record edited to say so; an ungated record's g is 1.
    #[test]
    fn gate_on_both_sides_of_sigma_g() {
        let r = registry();
        let n = law(&r, "N_Fe");
        assert_eq!(n.g(0.0), 0.0);
        assert_eq!(n.g(159.0), 0.0);
        assert_eq!(n.g(160.0), 0.0, "the gate opens strictly above Σ_g");
        assert_eq!(n.g(160.0_f64.next_down()), 0.0);
        assert!(n.g(160.0_f64.next_up()) > 0.0 && n.g(160.0_f64.next_up()) < 1e-12, "opens from 0");
        assert!(rel(n.g(161.6), 0.01) < 1e-9);
        assert!(rel(n.g(240.0), 0.5) < 1e-12);
        assert!(rel(n.g(320.0), 1.0) < 1e-12);
        assert!(rel(n.g(800.0), 4.0) < 1e-12);
        assert_eq!(law(&r, "H_burn").g(1e9), 1.0);
        assert_eq!(law(&r, "H_burn").g(0.0), 1.0);

        let text = include_str!("../../../assets/reactions.json").replace("\"p\": 1", "\"p\": 2");
        let (elements, physics) = (Elements::shipped().unwrap(), Physics::shipped().unwrap());
        let two = Reactions::parse(&text, &elements, &physics).expect("p = 2 loads");
        let n2 = Omega::new(two.get("N_Fe").unwrap());
        assert_eq!(n2.g(160.0), 0.0);
        assert!(rel(n2.g(480.0), 4.0) < 1e-12, "(3 − 1)² = 4");
    }

    /// The formula worked by hand: ω for every record of reactions.json at a hot, dense cell and at a cold one, against its
    /// own line written out below (A and ν copied from physics.json, T_k from [`TC_LADDER`]).
    #[test]
    fn omega_of_each_record_by_hand() {
        let r = pinned();
        let (sigma, y_e) = (3.0_f64, 0.5);
        let x = x_of(&r, &[("H", 0.6), ("He", 0.3), ("C", 0.2), ("Ne", 0.15), ("O", 0.25), ("Mg", 0.1), ("Si", 0.4), ("S", 0.05), ("Fe", 0.7)]);
        // (id, T, ω by hand)
        let cases: Vec<(&str, f64, f64)> = vec![
            ("H_burn", 114.0, 5e-4 * sigma * 0.6_f64.powi(2) * (gamow(114.0, 100.0, 4.0) + gamow(114.0, 115.0, 18.0))),
            ("He_burn", 150.0, 1e-3 * sigma.powi(2) * 0.3_f64.powi(3) * gamow(150.0, 140.0, 40.0)),
            ("C_alpha", 150.0, 1e-3 * sigma * 0.2 * 0.3 * gamow(150.0, 140.0, 40.0)),
            ("C_burn", 210.0, 1e-3 * sigma * 0.2_f64.powi(2) * gamow(210.0, 196.0, 30.0)),
            ("Ne_burn", 290.0, 1e-3 * sigma * 0.15 * gamow(290.0, 274.0, 30.0)),
            ("O_burn", 320.0, 1e-3 * sigma * 0.25_f64.powi(2) * gamow(320.0, 300.0, 30.0)),
            ("Mg_burn", 320.0, 1e-3 * sigma * 0.1 * gamow(320.0, 300.0, 30.0)),
            ("Si_burn", 450.0, 1e-3 * sigma * 0.4 * gamow(450.0, 420.0, 30.0)),
            ("S_burn", 450.0, 1e-3 * sigma * 0.05 * gamow(450.0, 420.0, 30.0)),
            // N_Fe: a = 0, f ≡ 1, g = Y_eΣ/160 − 1 with Y_eΣ = 0.5 × 400 = 200 → 0.25; ω = 1e-3 × 0.7 × 0.25.
            ("N_Fe", 1.0, 1e-3 * 0.7 * 0.25),
        ];
        assert_eq!(cases.len(), r.reactions.records().len(), "every record has a hand value");
        for (id, t, hand) in cases {
            let omega = law(&r, id);
            let s = if id == "N_Fe" { 400.0 } else { sigma };
            let got = omega.omega_direct(s, &x, t, y_e);
            assert!(hand > 0.0 && rel(got, hand) < 1e-12, "{id}: {got:e} against {hand:e}");
            assert!(rel(omega.omega(s, &x, t, y_e), hand) <= TABLE_TOLERANCE, "{id}: table {:e} against {hand:e}", omega.omega(s, &x, t, y_e));
        }
        // Cold: below T_thr every burning record is 0; N_Fe below its gate is 0 at any T.
        for rec in burning(&r) {
            let omega = Omega::new(rec);
            assert_eq!(omega.omega(sigma, &x, 0.9 * omega.t_thr().unwrap(), y_e), 0.0, "{} cold", rec.id);
        }
        let n = law(&r, "N_Fe");
        assert_eq!(n.omega(100.0, &x, 1e3, y_e), 0.0, "Y_eΣ = 50 < Σ_N");
    }

    /// The Gamow shape (§0.5 [M0-D34]): each term equals the power law it replaced at T_k in value and slope; its exponent
    /// falls as (τ/3)(T_k/T)^{1/3} − 2/3 — He_burn's 40 at T_He, ≈ 19.7 at 8 T_He; and ln f is bounded by τ − 2 ln(τ/2) − 2,
    /// reached at T = T_k (τ/2)³ (≈ 111.8 for ν 40 at 3.2 × 10⁷), so no record's f passes e^112 at any temperature.
    #[test]
    fn gamow_shape_and_bound() {
        let r = pinned();
        let slope = |o: &Omega, t: f64| {
            let d = 1e-5;
            (o.f_direct(t * (1.0 + d)).ln() - o.f_direct(t * (1.0 - d)).ln()) / ((1.0 + d) / (1.0 - d)).ln()
        };
        let mut highest: f64 = 0.0;
        for rec in burning(&r) {
            let omega = Omega::new(rec);
            for term in rec.rate.terms.iter().filter(|_| rec.rate.terms.len() == 1) {
                let (t_k, nu) = (term.t_k, term.nu);
                assert!(rel(omega.f_direct(t_k), term.w) < 1e-14, "{}: f(T_k) = w", rec.id);
                assert!((slope(&omega, t_k) - nu).abs() < 1e-6, "{}: slope {} at T_k, ν {nu}", rec.id, slope(&omega, t_k));
                assert!((slope(&omega, 8.0 * t_k) - ((3.0 * nu + 2.0) / 6.0 - 2.0 / 3.0)).abs() < 1e-6, "{}: slope at 8 T_k", rec.id);
                let tau = 3.0 * nu + 2.0;
                let (bound, at) = (tau - 2.0 * (tau / 2.0).ln() - 2.0, t_k * (tau / 2.0).powi(3));
                assert!((omega.f_direct(at).ln() - bound).abs() < 1e-9, "{}: ln f {} at its top, bound {bound}", rec.id, omega.f_direct(at).ln());
                for i in 0..=400 {
                    let t = t_k * 10f64.powf(i as f64 / 40.0);
                    assert!(omega.f_direct(t).ln() <= bound + 1e-9, "{}: ln f {} > {bound} at T = {t}", rec.id, omega.f_direct(t).ln());
                }
            }
            for i in 0..=400 {
                highest = highest.max(omega.f(omega.t_thr().unwrap() * 10f64.powf(i as f64 / 40.0)).ln());
            }
        }
        let he = law(&r, "He_burn");
        assert!((slope(&he, 140.0) - 40.0).abs() < 1e-6 && (slope(&he, 8.0 * 140.0) - 19.666_666_666_666_668).abs() < 1e-6);
        assert!(highest < 112.0 && highest > 111.0, "the largest ln f over the shipped records {highest}");
    }
}
