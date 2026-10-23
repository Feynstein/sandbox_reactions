//! Heat and light on the CPU twin (m0_contrat.md §1.7, §2.4, §0.5 [M0-TJ4], §1.6.4's S1, §5.1's G-HEAT1, G-HEAT2 and G-FLD,
//! R13): P5's f64 reference, written before its shader (M0-T118 and M0-T119 ported it to the GPU).
//!
//! One temperature (SM M4): ∂(Σε_th)/∂t = −∇·F, ε_th = T/μ, so a cell holds the heat capacity C = Σ/μ per area. The
//! radiative flux is F = −(c_sb λ(R)/(κΣ)) ∇(a_r T⁴) with Levermore–Pomraning's λ(R) = (2 + R)/(6 + 3R + R²) and
//! R = |∇(a_r T⁴)|/(κΣ a_r T⁴) (§2.4.1, SM-E19), plus the conduction k_cond ∇T. Across a face the gradient is the
//! difference of the two cells (Δx = 1), κΣ is the mean of the two cells' κΣ and a_r T⁴ in R the mean of theirs, so the
//! law's |F| = c_sb λR · a_r T⁴_face ≤ c_sb a_r T⁴ of the hotter side by construction (λR < 1); thick gas gives λ = 1/3,
//! the radiative conductivity (4/3) a_r c_sb T³/(κΣ) (SM-E20). κ = κ₀(1 + X_H) plus κ_dust Z_met where T < T_dust (S1,
//! `kappa_dust` = 0 until the stand-in is enabled, §2.4.2).
//!
//! The step (§0.5 [M0-TJ4] (1)–(5)) is one backward-Euler step in the heat potential u = a_r T⁴: a face carries
//! F = −D_f (u_b − u_a), D_f = c_sb λ(R_f)/(κΣ)_f + k_cond/(a_r (T_a + T_b)(T_a² + T_b²)), taken once from the pass's start
//! state (the light law frozen); every gas cell then solves C_i (T(u_i) − T_i⁰) + Δt Σ_f F_f(u) = 0 (outflow positive),
//! T(u) = (u/a_r)^{1/4}. Newton passes linearise T(u) at the latest iterate, so each pass is one symmetric positive-definite
//! system — diagonal c_i + Σ_f D_f with c_i = C_i/(4 a_r T_i³ Δt), −D_f between gas cells — solved by Jacobi-preconditioned
//! conjugate gradients to [`SOLVE_TOL`] in both of (4)'s measures; a pass moves no cell's T by more than a factor 2 and keeps
//! u ≥ u_vac, and the passes end after the first one that moves Σ C |ΔT| by at most [`NEWTON_HEAT`] of Σ C T⁰ and every
//! cell by at most [`NEWTON_CELL`] of its T. The first iterate is the start state; nothing is kept between steps. Past
//! [`MAX_PASSES`] passes or [`MAX_ROUNDS`] rounds the step commits by limited fluxes and says so ([`Pass::warning`]).
//!
//! The commit: F is the solved flux, the energy the step moved (step n + 1's P4 reads it, M0-T43); what the gas cells lose
//! through their vacuum and edge faces is `radiated` R; E_i += C_i ((1 + δ) T(u_i) − T_i⁰) with δ = −(Σ C (T(u) − T⁰) + R)/
//! Σ C T(u), so the grid loses exactly the heat it books. RKL2 (§2.4.3 before [M0-TJ4]) is retired (M0-T119). Past the
//! ceiling (§0.5 [M0-D89]) the last iterate's T(u) is no answer — a hot cell beside vacuum still at a factor 2 of its T⁰
//! emits ~10¹² times the heat it holds, and δ took that from every cell — so the heat moves by the faces' F at the last
//! iterate instead, each gas cell's outflow scaled to the heat it holds above its floor ([`Heat::commit_limited`]).
//!
//! Light leaving (§2.4.4): a face between a gas cell and a vacuum cell (Σ < Σ_vac), or the world edge in either edge mode
//! (D1), has the vacuum state — Σ_floor, T_floor, the gas side's κ and μ, u_vac = a_r T_floor⁴ — on its far side. The
//! vacuum cell is not updated; what the gas cell loses through such faces is booked per emitting cell ([`Pass::emitted`],
//! the frame accumulator of §1.9.1) and in all ([`Pass::radiated`]).
//!
//! The scene's `test` flags (§2.12.1): `constant_diffusivity` = χ makes u = T and D_f = χ · C_f (the mean of the two heat
//! capacities — a uniform medium diffuses T with the diffusivity χ exactly, G-HEAT1), the step linear; `flux_limiter`
//! false makes λ ≡ 1/3 (the limiter off, G-FLD's plant); `pin_temperature` leaves the temperatures alone — the pass keeps
//! the law's F on the start state ([`Heat::law`]), and books nothing.
//!
//! Conventions as [`hydro`](super::hydro): channel-major planes in §2.2's order, `index = y * w + x`, y down; only E
//! changes — by C · ΔT, so the cold energy and the kinetic energy stay as they were.

use super::hydro::State;
use crate::eos::Eos;
use crate::registry::{Elements, Physics, RegistryError, N_SPECIES};

const NS: usize = N_SPECIES;
/// Hydrogen's slot in §1.5's order, and the neutron fraction's.
const ID_H: usize = 0;
const ID_N: usize = 9;
/// The step's ceiling (§0.5 [M0-TJ4] (6), [M0-D37]): Newton passes, and CG rounds over all of them. A heating front
/// through cold gas moves about a cell a pass: sun_disk's steps 340–347 need 17–37 passes on the GPU, 20–41
/// at this module's tolerances (an f64 model on the GPU's own systems, M0-D37).
pub const MAX_PASSES: usize = 64;
pub const MAX_ROUNDS: usize = 4096;
/// The twin's stopping rules (§0.5 [M0-TJ4] (3), (4)): the passes' heat and per-cell change, each solve's two measures.
pub const NEWTON_HEAT: f64 = 1e-12;
pub const NEWTON_CELL: f64 = 1e-10;
pub const SOLVE_TOL: f64 = 1e-12;

/// §2.12.1's `test`, as the heat pass reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestFlags {
    pub pin_temperature: bool,
    pub constant_diffusivity: Option<f64>,
    pub flux_limiter: bool,
}

impl Default for TestFlags {
    fn default() -> Self {
        Self { pin_temperature: false, constant_diffusivity: None, flux_limiter: true }
    }
}

/// F on every face: `x` has `(width + 1) × height` faces, face `(x, y)` between cells `(x − 1, y)` and `(x, y)` (x = 0 and
/// x = width are the world's edges); `y` has `width × (height + 1)`, face `(x, y)` between `(x, y − 1)` and `(x, y)`. A flux
/// is positive toward +x or +y.
#[derive(Debug, Clone, PartialEq)]
pub struct Flux {
    pub width: usize,
    pub height: usize,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl Flux {
    /// The cell-centred field P4 reads (§2.2's two planes of F): the mean of a cell's two faces on each axis.
    pub fn cells(&self) -> (Vec<f64>, Vec<f64>) {
        let (w, h) = (self.width, self.height);
        let mut fx = vec![0.0; w * h];
        let mut fy = vec![0.0; w * h];
        for y in 0..h {
            for x in 0..w {
                fx[y * w + x] = 0.5 * (self.x[y * (w + 1) + x] + self.x[y * (w + 1) + x + 1]);
                fy[y * w + x] = 0.5 * (self.y[y * w + x] + self.y[(y + 1) * w + x]);
            }
        }
        (fx, fy)
    }
}

/// What one pass of P5 did.
#[derive(Debug, Clone, PartialEq)]
pub struct Pass {
    /// Newton passes (0 when the temperatures are pinned, nothing is gas or Δt is 0).
    pub passes: usize,
    /// CG rounds over all its passes.
    pub rounds: usize,
    /// The heat balance's closure δ (§0.5 [M0-TJ4] (5)): of the order of the solve's tolerance when the step converged, 0
    /// past the ceiling.
    pub delta: f64,
    /// False when the step reached [`MAX_PASSES`] or [`MAX_ROUNDS`] and committed by limited fluxes (§0.5 [M0-D89]).
    pub converged: bool,
    /// Booked `radiated` (§2.9): the heat the gas lost into vacuum cells and through the world's edge, energy per area summed.
    pub radiated: f64,
    /// `radiated` by emitting cell (§2.4.4, §1.9.1's frame accumulator): `width × height`.
    pub emitted: Vec<f64>,
    /// F at every face: the solved flux, the start state's when pinned (module doc).
    pub flux: Flux,
}

impl Pass {
    /// `SR-WARN heat unconverged step <n> passes <k> rounds <r>` when the step hit its ceiling (§0.5 [M0-TJ4] (6)).
    pub fn warning(&self, step: u64) -> Option<String> {
        (!self.converged).then(|| format!("SR-WARN heat unconverged step {step} passes {} rounds {}", self.passes, self.rounds))
    }
}

/// Levermore–Pomraning's flux limiter (SM-E19): 1/3 for R → 0, ≈ 1/R for R → ∞.
pub fn lambda(r: f64) -> f64 {
    (2.0 + r) / (6.0 + 3.0 * r + r * r)
}

/// One side of a face: a gas cell, a vacuum cell or the ghost beyond the edge.
#[derive(Debug, Clone, Copy)]
struct Side {
    gas: bool,
    t: f64,
    sigma: f64,
    kappa: f64,
    inv_mu: f64,
}

impl Side {
    /// κΣ, the optical depth of a cell.
    fn ks(&self) -> f64 {
        self.kappa * self.sigma
    }

    /// C = Σ/μ, the heat capacity per area.
    fn c(&self) -> f64 {
        self.sigma * self.inv_mu
    }
}

/// P5's CPU f64 twin: the constants of a run and the scheme.
#[derive(Debug, Clone)]
pub struct Heat {
    eos: Eos,
    elements: Elements,
    sigma_vac: f64,
    sigma_floor: f64,
    a_r: f64,
    c_sb: f64,
    kappa0: f64,
    k_cond: f64,
    kappa_dust: f64,
    t_dust: f64,
    test: TestFlags,
}

impl Heat {
    /// The twin of a run's constants (a scene's `overrides` included) and its `test` flags.
    pub fn new(physics: &Physics, elements: &Elements, test: TestFlags) -> Result<Heat, RegistryError> {
        let get = |k: &str| physics.get(k).expect("physics.json key");
        Ok(Heat {
            eos: Eos::from_physics(physics)?,
            elements: elements.clone(),
            sigma_vac: get("sigma_vac"),
            sigma_floor: get("sigma_floor"),
            a_r: get("a_r"),
            c_sb: get("c_sb"),
            kappa0: get("kappa0"),
            k_cond: get("k_cond"),
            kappa_dust: get("kappa_dust"),
            t_dust: get("t_dust"),
            test,
        })
    }

    /// λ as this run applies it: Levermore–Pomraning's, or 1/3 with the scene's `flux_limiter` off.
    fn limiter(&self, r: f64) -> f64 {
        if !self.test.flux_limiter {
            return 1.0 / 3.0;
        }
        lambda(r)
    }

    /// Cell `i` of `st` as one side of its faces: its temperature (the floor state below Σ_vac), κ and 1/μ.
    fn side(&self, st: &State, i: usize) -> Side {
        let x: [f64; NS] = std::array::from_fn(|s| st.get(4 + s, i));
        let comp = self.elements.composition(&x);
        let mu = 1.0 / comp.inv_mu;
        let sigma = st.get(0, i);
        if sigma < self.sigma_vac {
            return Side { gas: false, t: self.eos.t_floor(), sigma: self.sigma_floor, kappa: self.kappa(&x, comp.z_met, self.eos.t_floor()), inv_mu: comp.inv_mu };
        }
        let th = self.eos.thermal(st.get(3, i), sigma, [st.get(1, i), st.get(2, i)], comp.y_e, x[ID_N], mu);
        let t = Eos::temperature(th.eps_th, mu);
        Side { gas: true, t, sigma, kappa: self.kappa(&x, comp.z_met, t), inv_mu: comp.inv_mu }
    }

    /// κ = κ₀ (1 + X_H) + κ_dust Z_met where T < T_dust (§2.4.2, S1).
    fn kappa(&self, x: &[f64; NS], z_met: f64, t: f64) -> f64 {
        let dust = if t < self.t_dust { self.kappa_dust * z_met } else { 0.0 };
        self.kappa0 * (1.0 + x[ID_H]) + dust
    }

    /// The vacuum beyond a gas side: Σ_floor at T_floor with the gas side's κ and μ.
    fn vacuum_beyond(&self, gas: &Side) -> Side {
        Side { gas: false, t: self.eos.t_floor(), sigma: self.sigma_floor, kappa: gas.kappa, inv_mu: gas.inv_mu }
    }

    /// The heat potential u of a temperature: a_r T⁴, or T under `constant_diffusivity`.
    fn u_of(&self, t: f64) -> f64 {
        if self.test.constant_diffusivity.is_some() {
            return t;
        }
        self.a_r * t.powi(4)
    }

    /// T(u), the inverse of [`Heat::u_of`].
    fn t_of(&self, u: f64) -> f64 {
        if self.test.constant_diffusivity.is_some() {
            return u;
        }
        (u / self.a_r).sqrt().sqrt()
    }

    /// dT/du at T: 1/(4 a_r T³), or 1 under `constant_diffusivity`.
    fn dt_du(&self, t: f64) -> f64 {
        if self.test.constant_diffusivity.is_some() {
            return 1.0;
        }
        1.0 / (4.0 * self.a_r * t.powi(3))
    }

    /// D_f of the face between `a` and `b` (module doc); 0 between two vacuums.
    fn diffusivity(&self, a: &Side, b: &Side) -> f64 {
        if !a.gas && !b.gas {
            return 0.0;
        }
        if let Some(chi) = self.test.constant_diffusivity {
            return chi * (0.5 * (a.c() + b.c()));
        }
        let (era, erb) = (self.a_r * a.t.powi(4), self.a_r * b.t.powi(4));
        let face_er = 0.5 * (era + erb);
        let ks = 0.5 * (a.ks() + b.ks());
        let r = if face_er > 0.0 { (erb - era).abs() / (ks * face_er) } else { 0.0 };
        let radiative = self.c_sb * self.limiter(r) / ks;
        let conduction = if self.k_cond != 0.0 { self.k_cond / (self.a_r * (a.t + b.t) * (a.t * a.t + b.t * b.t)) } else { 0.0 };
        radiative + conduction
    }

    /// The gas cells' sides and the face network of `st`, each face's coefficient by `coef` (D_f) and the far-side
    /// value `vac` beyond a gas cell's vacuum and edge faces.
    fn network(&self, st: &State, coef: impl Fn(&Side, &Side) -> f64, vac: f64) -> (Vec<Side>, Net) {
        let (w, h) = (st.width, st.height);
        let sides: Vec<Side> = (0..w * h).map(|i| self.side(st, i)).collect();
        // One face: the two sides (a vacuum beyond the edge, or beyond a gas cell next to vacuum, is the far side).
        let face = |a: Option<&Side>, b: Option<&Side>| -> f64 {
            let (a, b) = match (a, b) {
                (Some(a), Some(b)) if a.gas || b.gas => (if a.gas { *a } else { self.vacuum_beyond(b) }, if b.gas { *b } else { self.vacuum_beyond(a) }),
                (Some(a), Some(_)) => (*a, *a),
                (Some(a), None) => (*a, self.vacuum_beyond(a)),
                (None, Some(b)) => (self.vacuum_beyond(b), *b),
                (None, None) => unreachable!("a face has a cell on one side at least"),
            };
            coef(&a, &b)
        };
        let mut net = Net {
            w,
            h,
            gas: sides.iter().map(|s| s.gas).collect(),
            c: sides.iter().map(|s| s.c()).collect(),
            kx: vec![0.0; (w + 1) * h],
            ky: vec![0.0; w * (h + 1)],
            vac,
        };
        for y in 0..h {
            for x in 0..=w {
                net.kx[y * (w + 1) + x] = face(if x > 0 { Some(&sides[y * w + x - 1]) } else { None }, if x < w { Some(&sides[y * w + x]) } else { None });
            }
        }
        for y in 0..=h {
            for x in 0..w {
                net.ky[y * w + x] = face(if y > 0 { Some(&sides[(y - 1) * w + x]) } else { None }, if y < h { Some(&sides[y * w + x]) } else { None });
            }
        }
        (sides, net)
    }

    /// The law the pass freezes, on `st`: F = −D_f (u_b − u_a) at every face from the start state (§0.5 [M0-TJ4] (1)) —
    /// what a pinned pass keeps, and what G-FLD bounds.
    pub fn law(&self, st: &State) -> Flux {
        let (sides, net) = self.network(st, |a, b| self.diffusivity(a, b), self.u_of(self.eos.t_floor()));
        let u: Vec<f64> = sides.iter().map(|s| self.u_of(s.t)).collect();
        net.flux(&u)
    }
}

/// The frozen network of a pass: which cells are gas, their heat capacities, the coefficient of every face and the value
/// beyond a gas cell's vacuum and edge faces.
struct Net {
    w: usize,
    h: usize,
    gas: Vec<bool>,
    c: Vec<f64>,
    kx: Vec<f64>,
    ky: Vec<f64>,
    vac: f64,
}

impl Net {
    /// The four faces of cell (x, y): coefficient and the cell index beyond (None: the ghost past the world's edge).
    fn faces(&self, x: usize, y: usize) -> [(f64, Option<usize>); 4] {
        let (w, h) = (self.w, self.h);
        let i = y * w + x;
        [
            (self.kx[y * (w + 1) + x], (x > 0).then(|| i - 1)),
            (self.kx[y * (w + 1) + x + 1], (x + 1 < w).then(|| i + 1)),
            (self.ky[y * w + x], (y > 0).then(|| i - w)),
            (self.ky[(y + 1) * w + x], (y + 1 < h).then(|| i + w)),
        ]
    }

    /// F = −k (v_b − v_a) at every face for the cell values `v` (the far side `vac` where a cell is not gas or is the edge).
    fn flux(&self, v: &[f64]) -> Flux {
        let (w, h) = (self.w, self.h);
        let at = |i: Option<usize>| match i {
            Some(i) if self.gas[i] => v[i],
            _ => self.vac,
        };
        let mut flux = Flux { width: w, height: h, x: vec![0.0; (w + 1) * h], y: vec![0.0; w * (h + 1)] };
        for y in 0..h {
            for x in 0..=w {
                let (a, b) = ((x > 0).then(|| y * w + x - 1), (x < w).then(|| y * w + x));
                let k = self.kx[y * (w + 1) + x];
                flux.x[y * (w + 1) + x] = if k == 0.0 { 0.0 } else { -k * (at(b) - at(a)) };
            }
        }
        for y in 0..=h {
            for x in 0..w {
                let (a, b) = ((y > 0).then(|| (y - 1) * w + x), (y < h).then(|| y * w + x));
                let k = self.ky[y * w + x];
                flux.y[y * w + x] = if k == 0.0 { 0.0 } else { -k * (at(b) - at(a)) };
            }
        }
        flux
    }

    /// Σ_f k over a gas cell's four faces.
    fn k_sum(&self, i: usize) -> f64 {
        self.faces(i % self.w, i / self.w).iter().map(|(k, _)| k).sum()
    }

    /// Σ_f k over a gas cell's vacuum and edge faces.
    fn k_vac(&self, i: usize) -> f64 {
        self.faces(i % self.w, i / self.w).iter().filter(|(_, nb)| !matches!(nb, Some(j) if self.gas[*j])).map(|(k, _)| k).sum()
    }

    /// A v = (diag) v − Σ_f k v_nb over the gas cells (0 elsewhere).
    fn matvec(&self, diag: &[f64], v: &[f64], out: &mut [f64]) {
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                if !self.gas[i] {
                    out[i] = 0.0;
                    continue;
                }
                let mut s = diag[i] * v[i];
                for (k, nb) in self.faces(x, y) {
                    if let Some(j) = nb.filter(|&j| self.gas[j]) {
                        s -= k * v[j];
                    }
                }
                out[i] = s;
            }
        }
    }
}

/// One Newton pass's system and its stopping measures (§0.5 [M0-TJ4] (3), (4)).
struct System<'a> {
    net: &'a Net,
    /// The diagonal c_i + Σ_f D_f.
    diag: Vec<f64>,
    rhs: Vec<f64>,
    /// C_i/Δt and dT/du at the iterate, for the heat measure's ê_i = r_i (dT/du)_i / D_ii.
    heat: Vec<f64>,
    slope: Vec<f64>,
    /// ε² Σ c_i (u_i^{k−1})² and ε² Σ (C_i/Δt) (T_i^{k−1})².
    u_bound: f64,
    t_bound: f64,
}

impl System<'_> {
    /// Jacobi-preconditioned CG from `x`, at most `max_rounds` rounds, ending at the first round where both measures hold
    /// (checked before every round, the start included); returns the rounds it ran and whether the measures hold.
    fn solve(&self, x: &mut [f64], max_rounds: usize) -> (usize, bool) {
        let n = x.len();
        let gas = &self.net.gas;
        let mut q = vec![0.0; n];
        self.net.matvec(&self.diag, x, &mut q);
        let mut r: Vec<f64> = (0..n).map(|i| if gas[i] { self.rhs[i] - q[i] } else { 0.0 }).collect();
        let mut z: Vec<f64> = (0..n).map(|i| if gas[i] { r[i] / self.diag[i] } else { 0.0 }).collect();
        let mut p = z.clone();
        let mut rz: f64 = (0..n).map(|i| r[i] * z[i]).sum();
        let mut rounds = 0;
        loop {
            let e_heat: f64 = (0..n).filter(|&i| gas[i]).map(|i| self.heat[i] * (r[i] * self.slope[i] / self.diag[i]).powi(2)).sum();
            if rz <= self.u_bound && e_heat <= self.t_bound {
                return (rounds, true);
            }
            if rounds == max_rounds {
                return (rounds, false);
            }
            self.net.matvec(&self.diag, &p, &mut q);
            let pq: f64 = (0..n).map(|i| p[i] * q[i]).sum();
            let alpha = rz / pq;
            for i in 0..n {
                x[i] += alpha * p[i];
                r[i] -= alpha * q[i];
                z[i] = if gas[i] { r[i] / self.diag[i] } else { 0.0 };
            }
            let rz_next: f64 = (0..n).map(|i| r[i] * z[i]).sum();
            let beta = rz_next / rz;
            for i in 0..n {
                p[i] = z[i] + beta * p[i];
            }
            rz = rz_next;
            rounds += 1;
        }
    }
}

impl Heat {
    /// P5 on `st`, in place, over `dt`: the implicit step of §0.5 [M0-TJ4] (module doc); E changes by
    /// C ((1 + δ) T − T⁰) in every gas cell; returns its passes, rounds, δ, the booking and the solved F.
    pub fn step(&self, st: &mut State, dt: f64) -> Pass {
        let (w, h) = (st.width, st.height);
        let n = w * h;
        let t_floor = self.eos.t_floor();
        let u_vac = self.u_of(t_floor);
        let (sides, net) = self.network(st, |a, b| self.diffusivity(a, b), u_vac);
        let t0: Vec<f64> = sides.iter().map(|s| s.t).collect();
        let u0: Vec<f64> = t0.iter().map(|&t| self.u_of(t)).collect();
        let gas: Vec<usize> = (0..n).filter(|&i| net.gas[i]).collect();
        let none = Pass { passes: 0, rounds: 0, delta: 0.0, converged: true, radiated: 0.0, emitted: vec![0.0; n], flux: net.flux(&u0) };
        if self.test.pin_temperature || gas.is_empty() || dt.is_nan() || dt <= 0.0 {
            return none;
        }

        let heat0: f64 = gas.iter().map(|&i| net.c[i] * t0[i]).sum();
        let k_sum: Vec<f64> = (0..n).map(|i| if net.gas[i] { net.k_sum(i) } else { 0.0 }).collect();
        let k_vac: Vec<f64> = (0..n).map(|i| if net.gas[i] { net.k_vac(i) } else { 0.0 }).collect();
        let (mut t, mut u) = (t0.clone(), u0.clone());
        let (mut passes, mut rounds, mut converged) = (0, 0, false);
        while passes < MAX_PASSES && rounds < MAX_ROUNDS {
            passes += 1;
            // The pass's system at the iterate (t, u): (c_i + Σ D_f) u_i − Σ_gas D_f u_j = c_i u_i − (C_i/Δt)(T_i − T_i⁰) +
            // Σ_vac D_f u_vac, c_i = (C_i/Δt) dT/du.
            let heat: Vec<f64> = (0..n).map(|i| if net.gas[i] { net.c[i] / dt } else { 0.0 }).collect();
            let slope: Vec<f64> = (0..n).map(|i| if net.gas[i] { self.dt_du(t[i]) } else { 0.0 }).collect();
            let ci: Vec<f64> = (0..n).map(|i| heat[i] * slope[i]).collect();
            let eps2 = SOLVE_TOL * SOLVE_TOL;
            let sys = System {
                net: &net,
                diag: (0..n).map(|i| if net.gas[i] { ci[i] + k_sum[i] } else { 1.0 }).collect(),
                rhs: (0..n).map(|i| if net.gas[i] { ci[i] * u[i] - heat[i] * (t[i] - t0[i]) + k_vac[i] * u_vac } else { 0.0 }).collect(),
                u_bound: eps2 * gas.iter().map(|&i| ci[i] * u[i] * u[i]).sum::<f64>(),
                t_bound: eps2 * gas.iter().map(|&i| heat[i] * t[i] * t[i]).sum::<f64>(),
                heat,
                slope,
            };
            let mut x: Vec<f64> = (0..n).map(|i| if net.gas[i] { u[i] } else { 0.0 }).collect();
            let (r, _) = sys.solve(&mut x, MAX_ROUNDS - rounds);
            rounds += r;
            // The pass's update: u ≥ u_vac, T moved by no more than a factor 2.
            let (mut moved, mut every) = (0.0, true);
            for &i in &gas {
                let next = self.t_of(x[i].max(u_vac)).clamp(0.5 * t[i], 2.0 * t[i]);
                let d = (next - t[i]).abs();
                moved += net.c[i] * d;
                every &= d <= NEWTON_CELL * t[i];
                t[i] = next;
                u[i] = self.u_of(next);
            }
            if moved <= NEWTON_HEAT * heat0 && every {
                converged = true;
                break;
            }
        }

        if !converged {
            return self.commit_limited(st, &net, &u, &t0, dt, passes, rounds);
        }
        // The commit: the solved flux, the light leaving, E closed by δ.
        let flux = net.flux(&u);
        let mut emitted = vec![0.0; n];
        for &i in &gas {
            emitted[i] = dt * k_vac[i] * (u[i] - u_vac);
        }
        let radiated: f64 = emitted.iter().sum();
        let gained: f64 = gas.iter().map(|&i| net.c[i] * (t[i] - t0[i])).sum();
        let held: f64 = gas.iter().map(|&i| net.c[i] * t[i]).sum();
        let delta = -(gained + radiated) / held;
        let energy = 3 * n;
        for &i in &gas {
            st.planes[energy + i] += net.c[i] * ((1.0 + delta) * t[i] - t0[i]);
        }
        Pass { passes, rounds, delta, converged, radiated, emitted, flux }
    }

    /// The commit past the ceiling (§0.5 [M0-D89]): F = −D_f (u_b − u_a) at the last iterate `u`, each gas cell's outflow
    /// Δt Σ_f max(F, 0) over its four faces scaled by s_i = min(1, C_i (T_i⁰ − T_floor)/outflow) — every face's F taken at
    /// the scale of the cell it leaves (1 from vacuum) — then E_i −= Δt Σ_f F_f (outflow positive), and what leaves through
    /// the vacuum and edge faces is `radiated`. No cell gives more heat than it holds above its floor, the two sides of a face
    /// book the same F, and δ = 0: the grid loses exactly the heat it books, face by face.
    #[allow(clippy::too_many_arguments)]
    fn commit_limited(&self, st: &mut State, net: &Net, u: &[f64], t0: &[f64], dt: f64, passes: usize, rounds: usize) -> Pass {
        let (w, h) = (net.w, net.h);
        let n = w * h;
        let t_floor = self.eos.t_floor();
        let mut flux = net.flux(u);
        // F leaving cell i through its −x, +x, −y and +y faces (`Net::faces`' order).
        let leaving = |f: &Flux, i: usize| {
            let (x, y) = (i % w, i / w);
            [-f.x[y * (w + 1) + x], f.x[y * (w + 1) + x + 1], -f.y[y * w + x], f.y[(y + 1) * w + x]]
        };
        let scale: Vec<f64> = (0..n)
            .map(|i| {
                if !net.gas[i] {
                    return 1.0;
                }
                let out = dt * leaving(&flux, i).iter().map(|f| f.max(0.0)).sum::<f64>();
                let held = net.c[i] * (t0[i] - t_floor).max(0.0);
                if out > held { held / out } else { 1.0 }
            })
            .collect();
        let of = |cell: Option<usize>| cell.map_or(1.0, |j| scale[j]);
        for y in 0..h {
            for x in 0..=w {
                let k = y * (w + 1) + x;
                let from = if flux.x[k] > 0.0 { (x > 0).then(|| y * w + x - 1) } else { (x < w).then(|| y * w + x) };
                flux.x[k] *= of(from);
            }
        }
        for y in 0..=h {
            for x in 0..w {
                let k = y * w + x;
                let from = if flux.y[k] > 0.0 { (y > 0).then(|| (y - 1) * w + x) } else { (y < h).then(|| y * w + x) };
                flux.y[k] *= of(from);
            }
        }
        let mut emitted = vec![0.0; n];
        let energy = 3 * n;
        for i in (0..n).filter(|&i| net.gas[i]) {
            let out = leaving(&flux, i);
            for (f, (_, beyond)) in net.faces(i % w, i / w).iter().enumerate() {
                if !matches!(beyond, Some(j) if net.gas[*j]) {
                    emitted[i] += dt * out[f];
                }
            }
            st.planes[energy + i] -= dt * out.iter().sum::<f64>();
        }
        let radiated = emitted.iter().sum();
        Pass { passes, rounds, delta: 0.0, converged: false, radiated, emitted, flux }
    }
}
