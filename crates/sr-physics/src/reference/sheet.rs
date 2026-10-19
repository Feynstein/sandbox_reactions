//! The sheet's cold equilibria (m0_contrat.md §2.3.2, §2.11 items 1–2 [M0-R4], §5.1 G-CORE): an axisymmetric razor-thin
//! disk of cold gas in hydrostatic equilibrium under §1.4.1's gravity, in the continuum — no grid, no GPU. The ceiling
//! mass of the sheet is a limit (R → 0 as Σ_c → ∞), so only a computation off the grid can show it (reports/cold_ceiling.md).
//!
//! Hydrostatic equilibrium of a barotropic gas is h(Σ) + φ = const, h(Σ) = ∫ Π′(s)/s ds. The potential of an annulus
//! carries the ring kernel: φ(r) = −G ∫ Σ(r′) 4r′/(r + r′) K(k) dr′, k² = 4rr′/(r + r′)², K by the arithmetic–geometric
//! mean, each annulus by 8-point Gauss–Legendre and split at the field point by r′ = r ± t², which takes the logarithmic
//! singularity out of the integrand. The solver is Hachisu's self-consistent field (SM-E6, S8): φ from Σ, Σ from h⁻¹, mixed,
//! until no Σ moves. All lengths are in units of the disk's radius until [`Sheet::fixed_central`] scales them back
//! (φ ∝ R, M ∝ R², W ∝ R³, ∫Π dA ∝ R²).

use std::f64::consts::PI;
use std::fmt;

use crate::eos::Eos;

/// G = 1 (§2.1).
const G: f64 = 1.0;
/// The annuli of §2.11 item 1.
pub const ANNULI: usize = 200;
/// The first and last decades of the sequence M_k = M(Σ_c = 10^k) (§2.11 item 1).
const DECADES: usize = 8;
/// A round settles when no Σ moves by more than this share of Σ_c (§2.11 item 1).
const TOLERANCE: f64 = 1e-11;
/// Rounds before the iteration gives up.
const MAX_ROUNDS: usize = 20_000;
/// The virial |2∫Π dA/W + 1| every equilibrium of the sequence must hold (§2.11 item 1).
const VIRIAL_BOUND: f64 = 1e-3;
/// The limits from decades (6, 7) and (7, 8) must agree within this share (§2.11 item 1, SM-O9).
const LIMIT_BAND: f64 = 5e-3;
/// Steps of the enthalpy table in ln Σ.
const TABLE_STEP: f64 = 2e-3;
/// The table's top, over every Σ_c of the sequence.
const TABLE_TOP: f64 = 1e10;

const GL: [(f64, f64); 8] = [
    (-0.960_289_856_497_536_3, 0.101_228_536_290_376_3),
    (-0.796_666_477_413_626_7, 0.222_381_034_453_374_5),
    (-0.525_532_409_916_329, 0.313_706_645_877_887_3),
    (-0.183_434_642_495_649_8, 0.362_683_783_378_362),
    (0.183_434_642_495_649_8, 0.362_683_783_378_362),
    (0.525_532_409_916_329, 0.313_706_645_877_887_3),
    (0.796_666_477_413_626_7, 0.222_381_034_453_374_5),
    (0.960_289_856_497_536_3, 0.101_228_536_290_376_3),
];

#[derive(Debug, Clone, PartialEq)]
pub enum SheetError {
    /// The self-consistent field did not settle at this central Σ.
    NotConverged { sigma_c: f64 },
    /// The equation of state has no cold pressure to hold a disk (a K pair is off).
    NoColdPressure,
    /// M_k did not rise from decade k − 1 to k.
    NotRising { k: usize },
    /// An equilibrium's |2∫Π dA/W + 1| is over the bound.
    Virial { k: usize, value: f64 },
    /// The limits from decades (6, 7) and (7, 8) differ by more than the band: M does not approach a ceiling as Σ_c^{−1/2}.
    NoCeiling { from_6_7: f64, from_7_8: f64 },
}

impl fmt::Display for SheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SheetError::NotConverged { sigma_c } => write!(f, "the self-consistent field did not settle at Σ_c = {sigma_c:e}"),
            SheetError::NoColdPressure => write!(f, "no cold pressure: a disk of this matter has no ceiling to compute"),
            SheetError::NotRising { k } => write!(f, "M(Σ_c) does not rise from decade {} to {k}", k - 1),
            SheetError::Virial { k, value } => write!(f, "decade {k}: |2∫Π dA/W + 1| = {value:e} is over {VIRIAL_BOUND:e}"),
            SheetError::NoCeiling { from_6_7, from_7_8 } => {
                write!(f, "no ceiling: the limit from decades 6, 7 is {from_6_7:.6e} and from 7, 8 is {from_7_8:.6e}")
            }
        }
    }
}

/// K(k) by the arithmetic–geometric mean, from the complementary modulus k′ = √(1 − k²).
fn ellk(kp: f64) -> f64 {
    let (mut a, mut b) = (1.0, kp.max(1e-300));
    for _ in 0..80 {
        if (a - b).abs() < 1e-15 * a {
            break;
        }
        (a, b) = (0.5 * (a + b), (a * b).sqrt());
    }
    PI / (2.0 * a)
}

/// 4r′/(r + r′) K(k): the potential at r per unit Σ per unit r′ of a ring at r′, times −G.
fn ring(r: f64, rp: f64) -> f64 {
    let s = r + rp;
    if s == 0.0 {
        return 0.0;
    }
    4.0 * rp / s * ellk((r - rp).abs() / s)
}

fn gauss(f: impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let (m, h) = (0.5 * (a + b), 0.5 * (b - a));
    GL.iter().map(|&(x, w)| w * f(m + h * x)).sum::<f64>() * h
}

/// A barotropic fluid Π(Σ): its enthalpy h(Σ) = ∫ Π′(s)/s ds tabulated in ln Σ from h(Σ_vac) = 0 (the disk's edge is where Σ
/// falls to Σ_vac), and the inverse. The sheet is solved for whatever Π it is given.
pub struct Fluid {
    pi: Box<dyn Fn(f64) -> f64>,
    ln_vac: f64,
    h: Vec<f64>,
}

impl Fluid {
    pub fn new(pi: impl Fn(f64) -> f64 + 'static, sigma_vac: f64) -> Fluid {
        let slope = |s: f64| {
            let d = 1e-6;
            (pi(s * (1.0 + d)) - pi(s * (1.0 - d))) / (2.0 * d * s)
        };
        let n = ((TABLE_TOP / sigma_vac).ln() / TABLE_STEP).ceil() as usize;
        let mut h = vec![0.0];
        for i in 0..n {
            // dh = Π′(Σ) d ln Σ, Π′ read at the segment's geometric midpoint.
            let mid = sigma_vac * (TABLE_STEP * (i as f64 + 0.5)).exp();
            h.push(h[i] + slope(mid) * TABLE_STEP);
        }
        Fluid { pi: Box::new(pi), ln_vac: sigma_vac.ln(), h }
    }

    pub fn pi(&self, sigma: f64) -> f64 {
        (self.pi)(sigma)
    }

    /// h(Σ), 0 at and under Σ_vac.
    pub fn enthalpy(&self, sigma: f64) -> f64 {
        let x = ((sigma.ln() - self.ln_vac) / TABLE_STEP).max(0.0);
        let i = (x as usize).min(self.h.len() - 2);
        self.h[i] + (self.h[i + 1] - self.h[i]) * (x - i as f64)
    }

    /// Σ with h(Σ) = `value`; 0 at and under h(Σ_vac) (vacuum).
    pub fn sigma_at(&self, value: f64) -> f64 {
        if value <= 0.0 {
            return 0.0;
        }
        let i = self.h.partition_point(|&h| h <= value);
        if i >= self.h.len() {
            return (self.ln_vac + TABLE_STEP * (self.h.len() - 1) as f64).exp();
        }
        let f = (value - self.h[i - 1]) / (self.h[i] - self.h[i - 1]);
        (self.ln_vac + TABLE_STEP * ((i - 1) as f64 + f)).exp()
    }
}

/// A hydrostatic disk of unit-free scale, as [`Sheet::fixed_central`] returns it (physical units, G = 1).
#[derive(Debug, Clone)]
pub struct Equilibrium {
    pub sigma_c: f64,
    /// The disk's radius, solved for.
    pub radius: f64,
    pub mass: f64,
    /// The radius holding 90 % of the mass.
    pub r90: f64,
    /// W = ½∫Σφ dA (negative).
    pub w: f64,
    /// ∫Π dA.
    pub pi_integral: f64,
    pub rounds: usize,
}

impl Equilibrium {
    /// 2∫Π dA/W + 1, zero for an equilibrium in a 1/r potential (the virial theorem of the sheet).
    pub fn virial(&self) -> f64 {
        2.0 * self.pi_integral / self.w + 1.0
    }
}

/// The pure-polytrope sheet Π = KΣ^γ of Hachisu's classic form: Σ_max = 1 and the edge at R = 1 fixed, K solved for.
#[derive(Debug, Clone)]
pub struct Polytrope {
    pub gamma: f64,
    pub k: f64,
    pub mass: f64,
    pub w: f64,
    pub pi_integral: f64,
    /// The profile on the unit grid's nodes.
    pub sigma: Vec<f64>,
    pub rounds: usize,
}

impl Polytrope {
    /// M G²/K²: for γ = 3/2 the ceiling constant c_sheet (M_ch = c_sheet K₂′²/G²).
    pub fn mass_constant(&self) -> f64 {
        self.mass * G * G / (self.k * self.k)
    }

    pub fn virial(&self) -> f64 {
        2.0 * self.pi_integral / self.w + 1.0
    }
}

/// The unit-radius grid — equal annuli out to R = 1 — with the ring kernel's weights W[i][j] = ∫ over annulus j of the
/// kernel at node i.
pub struct Sheet {
    n: usize,
    edges: Vec<f64>,
    nodes: Vec<f64>,
    weights: Vec<f64>,
}

impl Sheet {
    pub fn new(n: usize) -> Sheet {
        let edges: Vec<f64> = (0..=n).map(|j| j as f64 / n as f64).collect();
        let nodes: Vec<f64> = (0..n).map(|j| 0.5 * (edges[j] + edges[j + 1])).collect();
        let mut weights = vec![0.0; n * n];
        for (i, &r) in nodes.iter().enumerate() {
            for j in 0..n {
                let (a, b) = (edges[j], edges[j + 1]);
                weights[i * n + j] = if i == j {
                    // The singularity: each side of the field point by r′ = r ± t², four pieces in t.
                    let mut total = 0.0;
                    for (lo, hi, sign) in [(a, r, -1.0), (r, b, 1.0)] {
                        let l = (hi - lo).sqrt();
                        for q in 0..4 {
                            let (t0, t1) = (l * q as f64 / 4.0, l * (q + 1) as f64 / 4.0);
                            total += gauss(|t| ring(r, r + sign * t * t) * 2.0 * t, t0, t1);
                        }
                    }
                    total
                } else {
                    let near = (a - r).abs().min((b - r).abs());
                    let pieces = if near > 2.0 * (b - a) { 1 } else { 6 };
                    (0..pieces).map(|q| gauss(|x| ring(r, x), a + (b - a) * q as f64 / pieces as f64, a + (b - a) * (q + 1) as f64 / pieces as f64)).sum()
                };
            }
        }
        Sheet { n, edges, nodes, weights }
    }

    pub fn annuli(&self) -> usize {
        self.n
    }

    pub fn nodes(&self) -> &[f64] {
        &self.nodes
    }

    fn area(&self, j: usize) -> f64 {
        PI * (self.edges[j + 1] * self.edges[j + 1] - self.edges[j] * self.edges[j])
    }

    /// φ at each node for the profile `sigma` (unit radius, G = 1).
    pub fn potential(&self, sigma: &[f64]) -> Vec<f64> {
        (0..self.n).map(|i| -G * self.weights[i * self.n..(i + 1) * self.n].iter().zip(sigma).map(|(w, s)| w * s).sum::<f64>()).collect()
    }

    pub fn mass(&self, sigma: &[f64]) -> f64 {
        sigma.iter().enumerate().map(|(j, s)| self.area(j) * s).sum()
    }

    /// The radius holding `share` of the mass, Σ uniform within an annulus.
    pub fn radius_holding(&self, sigma: &[f64], share: f64) -> f64 {
        let (target, mut held) = (share * self.mass(sigma), 0.0);
        for (j, &s) in sigma.iter().enumerate() {
            let dm = self.area(j) * s;
            if held + dm >= target {
                return (self.edges[j] * self.edges[j] + (target - held) / (PI * s)).sqrt();
            }
            held += dm;
        }
        *self.edges.last().expect("a grid has an edge")
    }

    /// ½∫Σφ dA of the unit profile with its potential.
    fn grav_energy(&self, sigma: &[f64], phi: &[f64]) -> f64 {
        0.5 * (0..self.n).map(|j| self.area(j) * sigma[j] * phi[j]).sum::<f64>()
    }

    /// φ extrapolated to the edge R = 1 from the last two nodes.
    fn edge_potential(&self, phi: &[f64]) -> f64 {
        let (a, b) = (self.n - 2, self.n - 1);
        phi[b] + (phi[b] - phi[a]) * (1.0 - self.nodes[b]) / (self.nodes[b] - self.nodes[a])
    }

    /// The equilibrium of `fluid` with the central Σ fixed at `sigma_c` and the radius solved for: with R the radius, the
    /// unit profile obeys h(Σ(x)) = R (φ₁ − φ(x)), so R = h(Σ_c)/(φ₁ − φ(0)) each round.
    pub fn fixed_central(&self, fluid: &Fluid, sigma_c: f64) -> Result<Equilibrium, SheetError> {
        let hc = fluid.enthalpy(sigma_c);
        let mut sigma: Vec<f64> = self.nodes.iter().map(|r| sigma_c * (1.0 - r * r)).collect();
        for rounds in 1..=MAX_ROUNDS {
            let phi = self.potential(&sigma);
            let p1 = self.edge_potential(&phi);
            let radius = hc / (p1 - phi[0]);
            let next: Vec<f64> = phi.iter().map(|p| fluid.sigma_at(radius * (p1 - p))).collect();
            let err = sigma.iter().zip(&next).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max) / sigma_c;
            for (s, n) in sigma.iter_mut().zip(&next) {
                *s = 0.5 * (*s + n);
            }
            if err < TOLERANCE {
                let phi = self.potential(&sigma);
                let pi_integral: f64 = (0..self.n).map(|j| self.area(j) * fluid.pi(sigma[j])).sum();
                return Ok(Equilibrium {
                    sigma_c,
                    radius,
                    mass: radius * radius * self.mass(&sigma),
                    r90: radius * self.radius_holding(&sigma, 0.9),
                    w: radius.powi(3) * self.grav_energy(&sigma, &phi),
                    pi_integral: radius * radius * pi_integral,
                    rounds,
                });
            }
        }
        Err(SheetError::NotConverged { sigma_c })
    }

    /// Hachisu's SCF for Π = KΣ^γ with Σ_max = 1 and the edge at R = 1 fixed: K comes out of the central enthalpy,
    /// K γ/(γ − 1) = φ₁ − φ(0).
    pub fn polytrope(&self, gamma: f64) -> Result<Polytrope, SheetError> {
        let mut sigma: Vec<f64> = self.nodes.iter().map(|r| 1.0 - r * r).collect();
        for rounds in 1..=MAX_ROUNDS {
            let phi = self.potential(&sigma);
            let p1 = self.edge_potential(&phi);
            let hc = p1 - phi[0];
            let next: Vec<f64> = phi.iter().map(|p| ((p1 - p) / hc).max(0.0).powf(1.0 / (gamma - 1.0))).collect();
            let err = sigma.iter().zip(&next).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
            for (s, n) in sigma.iter_mut().zip(&next) {
                *s = 0.5 * (*s + n);
            }
            if err < TOLERANCE {
                let phi = self.potential(&sigma);
                let k = (p1 - phi[0]) * (gamma - 1.0) / gamma;
                let pi_integral = (0..self.n).map(|j| self.area(j) * k * sigma[j].powf(gamma)).sum();
                return Ok(Polytrope { gamma, k, mass: self.mass(&sigma), w: self.grav_energy(&sigma, &phi), pi_integral, sigma, rounds });
            }
        }
        Err(SheetError::NotConverged { sigma_c: 1.0 })
    }
}

/// The matter of a cold disk: electrons per nucleon, neutron fraction and 1/μ (§1.5), at T_floor (§2.11 item 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColdMatter {
    pub y_e: f64,
    pub x_n: f64,
    pub inv_mu: f64,
}

/// §2.11 items 1–2: the sequence M_k = M(Σ_c = 10^k), k = 0 … 8, and the ceiling it approaches.
#[derive(Debug, Clone)]
pub struct Ceiling {
    pub sequence: Vec<Equilibrium>,
    /// m_ch_sb (or m_tov_sb): M₈ + (M₈ − M₇)/(√10 − 1), the deficit falling as Σ_c^{−1/2}.
    pub limit: f64,
}

fn limit_of(lower: f64, upper: f64) -> f64 {
    upper + (upper - lower) / (10f64.sqrt() - 1.0)
}

/// The sequence and the limit of a fluid Π(Σ), with §2.11 item 1's refusals: M_k rising at every k, every equilibrium's
/// |2∫Π dA/W + 1| ≤ 10⁻³, and the limit from decades (6, 7) within 0.5 % of the one from (7, 8).
pub fn sequence(sheet: &Sheet, fluid: &Fluid) -> Result<Ceiling, SheetError> {
    let mut sequence: Vec<Equilibrium> = Vec::new();
    for k in 0..=DECADES {
        let eq = sheet.fixed_central(fluid, 10f64.powi(k as i32))?;
        if k > 0 && eq.mass <= sequence[k - 1].mass {
            return Err(SheetError::NotRising { k });
        }
        if eq.virial().abs() > VIRIAL_BOUND {
            return Err(SheetError::Virial { k, value: eq.virial() });
        }
        sequence.push(eq);
    }
    let m = |k: usize| sequence[k].mass;
    let (from_6_7, limit) = (limit_of(m(6), m(7)), limit_of(m(7), m(8)));
    if (from_6_7 - limit).abs() > LIMIT_BAND * limit {
        return Err(SheetError::NoCeiling { from_6_7, from_7_8: limit });
    }
    Ok(Ceiling { sequence, limit })
}

/// m_ch_sb for a `matter` (item 1: C 0.5 + O 0.5), m_tov_sb for neutron matter (item 2), through [`Eos`]'s own Π at T_floor.
pub fn ceiling(eos: &Eos, matter: ColdMatter, sigma_vac: f64) -> Result<Ceiling, SheetError> {
    if eos.electron_table().is_none() && eos.neutron_table().is_none() {
        return Err(SheetError::NoColdPressure);
    }
    let (eos, eps_th) = (eos.clone(), eos.t_floor() * matter.inv_mu);
    let fluid = Fluid::new(move |s| eos.pi(s, eps_th, matter.y_e, matter.x_n), sigma_vac);
    sequence(&Sheet::new(ANNULI), &fluid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{Elements, Physics};

    const C_SHEET: f64 = 1.4331;

    fn shipped() -> (Eos, Elements, f64) {
        let physics = Physics::shipped().expect("shipped physics");
        (Eos::from_physics(&physics).expect("shipped constants"), Elements::shipped().expect("shipped elements"), physics.get("sigma_vac").expect("sigma_vac"))
    }

    /// C 0.5 + O 0.5 (item 1) and neutron matter (item 2), by the registry's own composition.
    fn matters(elements: &Elements) -> [ColdMatter; 2] {
        let mut co = [0.0; crate::registry::N_SPECIES];
        co[elements.index_of("C").expect("carbon")] = 0.5;
        co[elements.index_of("O").expect("oxygen")] = 0.5;
        let mut n = [0.0; crate::registry::N_SPECIES];
        n[elements.index_of("n").expect("neutron matter")] = 1.0;
        let (c, nc) = (elements.composition(&co), elements.composition(&n));
        [ColdMatter { y_e: c.y_e, x_n: 0.0, inv_mu: c.inv_mu }, ColdMatter { y_e: nc.y_e, x_n: 1.0, inv_mu: nc.inv_mu }]
    }

    /// The Maclaurin disk Σ = √(1 − r²) (a = 1, Σ_c = 1, G = 1) has φ(r) = −(π²/2)(1 − r²/2) for r < 1 (SM-E6): the ring
    /// kernel, an annulus at a time, against it.
    #[test]
    fn ring_kernel_gives_the_maclaurin_potential() {
        let sheet = Sheet::new(ANNULI);
        let sigma: Vec<f64> = sheet.nodes().iter().map(|r| (1.0 - r * r).sqrt()).collect();
        let phi = sheet.potential(&sigma);
        let phi0 = PI * PI / 2.0;
        let worst = sheet.nodes().iter().zip(&phi).filter(|(r, _)| **r <= 0.95).map(|(r, p)| (p + phi0 * (1.0 - r * r / 2.0)).abs() / phi0).fold(0.0, f64::max);
        eprintln!("SR-SHEET-MACLAURIN worst potential error {worst:.2e} of |φ_c|");
        assert!(worst < 1e-4, "the Maclaurin disk's potential is off by {worst:e}");
    }

    /// γ = 3 is the sheet's critical polytrope family member whose equilibrium is the Maclaurin disk, with K = π²G a/(6 Σ_c)
    /// (h = 3KΣ²/2 against φ's r² term): the solver's profile and K against the closed form.
    #[test]
    fn gamma_three_equilibrium_is_the_maclaurin_profile() {
        let sheet = Sheet::new(ANNULI);
        let p = sheet.polytrope(3.0).expect("γ = 3 settles");
        let worst = sheet.nodes().iter().zip(&p.sigma).map(|(r, s)| (s - (1.0 - r * r).sqrt()).abs()).fold(0.0, f64::max);
        eprintln!("SR-SHEET-GAMMA3 profile error {worst:.2e}, K {:.6} (π²/6 = {:.6})", p.k, PI * PI / 6.0);
        assert!(worst < 1e-3, "γ = 3's profile is off the Maclaurin disk by {worst:e}");
        assert!((p.k / (PI * PI / 6.0) - 1.0).abs() < 1e-3, "K {} against π²/6", p.k);
    }

    /// The critical sheet Π = KΣ^{3/2}: M G²/K² = c_sheet = 1.4331 at 100 and 200 annuli, the virial 2∫Π dA/|W| = 1. A
    /// uniform disk's 9π/16 = 1.767 is no equilibrium and sits 23 % above.
    #[test]
    fn critical_sheet_mass_constant() {
        for n in [100, 200] {
            let p = Sheet::new(n).polytrope(1.5).expect("the critical sheet settles");
            let c = p.mass_constant();
            eprintln!("SR-SHEET-CSHEET n={n} c_sheet {c:.5} virial {:.2e}", p.virial());
            assert!((c / C_SHEET - 1.0).abs() < 1e-3, "{n} annuli: c_sheet {c}");
            assert!(p.virial().abs() < 1e-4, "{n} annuli: virial {}", p.virial());
            assert!(c < 9.0 * PI / 16.0 / 1.2, "the equilibrium is not the uniform disk's 1.767");
        }
    }

    /// Item 1 and item 2 on the shipped constants: M(Σ_c) rises at every decade, every virial inside 10⁻³, and the limits
    /// within 0.5 % of c_sheet K₂′² (3,216.4 and 4,853.2) — a second iteration (the critical sheet, Σ_max fixed) the
    /// sequence never uses supplies c_sheet's K₂′ scaling.
    #[test]
    fn shipped_limits_are_the_critical_sheets_mass() {
        let (eos, elements, vac) = shipped();
        let [co, neutron] = matters(&elements);
        let c_sheet = Sheet::new(ANNULI).polytrope(1.5).expect("the critical sheet settles").mass_constant();
        for (name, matter, k2, want) in [("C/O", co, 134.0 * 0.5f64.powf(1.5), 3216.4), ("neutron", neutron, 58.2, 4853.2)] {
            let c = ceiling(&eos, matter, vac).expect("the shipped EoS has a ceiling");
            eprintln!("SR-SHEET-LIMIT {name} limit {:.2} (c_sheet K₂′² {:.2}), M_8 {:.2}", c.limit, c_sheet * k2 * k2, c.sequence[8].mass);
            assert!((c.limit / want - 1.0).abs() < 5e-3, "{name}: limit {} against {want}", c.limit);
            assert!((c.limit / (c_sheet * k2 * k2) - 1.0).abs() < 5e-3, "{name}: limit {} against c_sheet K₂′²", c.limit);
            assert!(c.sequence.windows(2).all(|w| w[1].mass > w[0].mass), "{name}: M rises at every decade");
            assert!(c.sequence.iter().all(|e| e.virial().abs() <= 1e-3), "{name}: virials");
        }
        let (neutron_limit, co_limit) = (ceiling(&eos, neutron, vac).unwrap().limit, ceiling(&eos, co, vac).unwrap().limit);
        assert!((1.4..=2.0).contains(&(neutron_limit / co_limit)), "K9's band: m_tov/m_ch {}", neutron_limit / co_limit);
    }

    /// A relativistic exponent of 1.6 has no ceiling (M grows as Σ_c^0.1 without bound): the sequence refuses it, naming the
    /// two limits that disagree.
    #[test]
    fn exponent_one_point_six_has_no_ceiling() {
        let (_, _, vac) = shipped();
        let (k1, k2) = (5.325, 47.376);
        let fluid = Fluid::new(move |x| 1.0 / (1.0 / (k1 * x * x) + 1.0 / (k2 * x.powf(1.6))), vac);
        match sequence(&Sheet::new(ANNULI), &fluid) {
            Err(SheetError::NoCeiling { from_6_7, from_7_8 }) => assert!((from_6_7 / from_7_8 - 1.0).abs() > 0.05, "{from_6_7} vs {from_7_8}"),
            other => panic!("expected a refusal naming the missing ceiling, got {:?}", other.map(|c| c.limit)),
        }
    }

    /// The enthalpy table against closed forms: Π = KΣ² has h = 2K(Σ − Σ_vac) from h(Σ_vac) = 0, and σ_at inverts it.
    #[test]
    fn enthalpy_table_matches_the_closed_form_and_inverts() {
        let (k, vac) = (3.0, 1e-6);
        let fluid = Fluid::new(move |s| k * s * s, vac);
        for s in [1e-5, 1e-2, 1.0, 40.0, 1e5, 1e8] {
            let want = 2.0 * k * (s - vac);
            assert!((fluid.enthalpy(s) / want - 1.0).abs() < 1e-5, "h({s}) = {} against {want}", fluid.enthalpy(s));
            assert!((fluid.sigma_at(fluid.enthalpy(s)) / s - 1.0).abs() < 1e-9, "σ_at(h({s}))");
        }
        assert_eq!(fluid.sigma_at(0.0), 0.0);
    }

    /// The pure Σ² law (γ = 2): R is independent of M — SM-E4's « for D = 2 the radius is independent of mass », the sheet's
    /// analogue, which G-CORE (c) grades: R₉₀ flat within 3 % over a factor 4 in Σ_c.
    #[test]
    fn gamma_two_radius_is_independent_of_mass() {
        let sheet = Sheet::new(ANNULI);
        let fluid = Fluid::new(|s| 2.0 * s * s, 1e-6);
        let r90: Vec<f64> = [0.25, 0.5, 1.0].iter().map(|&c| sheet.fixed_central(&fluid, c).expect("settles").r90).collect();
        eprintln!("SR-SHEET-GAMMA2 R90 {r90:?}");
        assert!(r90.iter().all(|r| (r / r90[2] - 1.0).abs() < 0.03), "R90 {r90:?}");
    }
}
