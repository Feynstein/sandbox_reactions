//! Heat and light on the CPU twin (m0_contrat.md §1.7, §2.4, §1.6.4's S1, §5.1's G-HEAT1, G-HEAT2 and G-FLD, R13): P5's f64
//! reference, written before its shader (M0-T42 ports it).
//!
//! One temperature (SM M4): ∂(Σε_th)/∂t = −∇·F, ε_th = T/μ, so a cell holds the heat capacity C = Σ/μ per area. The
//! radiative flux is F = −(c_sb λ(R)/(κΣ)) ∇(a_r T⁴) with Levermore–Pomraning's λ(R) = (2 + R)/(6 + 3R + R²) and
//! R = |∇(a_r T⁴)|/(κΣ a_r T⁴) (§2.4.1, SM-E19), plus the conduction k_cond ∇T. Across a face the gradient is the
//! difference of the two cells (Δx = 1), κΣ is the mean of the two cells' κΣ and a_r T⁴ in R the mean of theirs, so
//! |F| = c_sb λR · a_r T⁴_face ≤ c_sb a_r T⁴ of the hotter side by construction (λR < 1); thick gas gives λ = 1/3, the
//! radiative conductivity (4/3) a_r c_sb T³/(κΣ) (SM-E20). κ = κ₀(1 + X_H) plus κ_dust Z_met where T < T_dust (S1,
//! `kappa_dust` = 0 until the stand-in is enabled, §2.4.2).
//!
//! A face's flux is written as a conductance: F = −K_f (T_b − T_a), K_f = c_sb λ a_r (T_a + T_b)(T_a² + T_b²)/(κΣ)_f +
//! k_cond — a_r (T_b⁴ − T_a⁴)/(T_b − T_a) taken as the product, so K_f needs no division by a temperature difference.
//! The coefficients K_f, and with them the pass's step, are frozen at the start of P5 (§2.4.3); the stages then advance
//! the linear system C dT/dt = Σ_f K_f (T_nb − T) with RKL2 super-time-stepping (Meyer et al. 2012, SM-E18):
//! Δt_p = 1/max_cells (Σ_f K_f / C) — the contract's 0.5/max(χ/Δx² + χ/Δy²) where χ = K/C is uniform, and Gershgorin's
//! bound where it is not — and s the smallest integer ≥ 2 with Δt_p (s² + s − 2)/4 ≥ Δt. Above [`MAX_STAGES`] stages the
//! pass says so ([`Pass::warning`], `SR-WARN heat stages <s>`); an implicit solver is a PLAN decision, never a silent
//! switch.
//!
//! Light leaving (§2.4.4): a face between a gas cell and a vacuum cell (Σ < Σ_vac), or the world edge in either edge mode
//! (D1), has the vacuum state — Σ_floor, T_floor, the gas side's κ and μ — on its far side. The vacuum cell is not
//! updated; what the gas cell loses through such faces is `radiated`, booked per emitting cell ([`Pass::emitted`], the
//! frame accumulator of §1.9.1) and in all ([`Pass::radiated`]). RKL2's recursion is linear, so the same coefficients
//! carry the booking: the energy the stages took out of the grid is exactly the energy booked, to rounding.
//!
//! F is kept (§1.3.2): [`Pass::flux`] holds each face's flux from the start-of-pass temperatures (positive toward +x, +y),
//! the field step n + 1's P4 reads for the radiation force (M0-T43).
//!
//! The scene's `test` flags (§2.12.1): `constant_diffusivity` = χ makes K_f = χ · C_f (the mean of the two heat
//! capacities — a uniform medium diffuses T with the diffusivity χ exactly, G-HEAT1); `flux_limiter` false makes λ ≡ 1/3
//! (the limiter off, G-FLD's plant); `pin_temperature` leaves the temperatures alone — the pass still computes F, and books
//! nothing.
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
/// SR-WARN above this many stages (§2.4.3).
pub const MAX_STAGES: usize = 32;

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

/// F on every face, at the temperatures the pass started from: `x` has `(width + 1) × height` faces, face `(x, y)`
/// between cells `(x − 1, y)` and `(x, y)` (x = 0 and x = width are the world's edges); `y` has `width × (height + 1)`,
/// face `(x, y)` between `(x, y − 1)` and `(x, y)`. A flux is positive toward +x or +y.
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
    /// RKL2's stages (0 when nothing could flow, or the temperatures are pinned).
    pub stages: usize,
    /// The explicit parabolic step Δt_p of the start-of-pass coefficients (∞ when nothing could flow).
    pub dt_p: f64,
    /// Booked `radiated` (§2.9): the heat the gas lost into vacuum cells and through the world's edge, energy per area summed.
    pub radiated: f64,
    /// `radiated` by emitting cell (§2.4.4, §1.9.1's frame accumulator): `width × height`.
    pub emitted: Vec<f64>,
    /// F at every face (module doc).
    pub flux: Flux,
}

impl Pass {
    /// `SR-WARN heat stages <s>` when the pass needed more than [`MAX_STAGES`] (§2.4.3).
    pub fn warning(&self) -> Option<String> {
        (self.stages > MAX_STAGES).then(|| format!("SR-WARN heat stages {}", self.stages))
    }
}

/// Levermore–Pomraning's flux limiter (SM-E19): 1/3 for R → 0, ≈ 1/R for R → ∞.
pub fn lambda(r: f64) -> f64 {
    (2.0 + r) / (6.0 + 3.0 * r + r * r)
}

/// The smallest s ≥ 2 with Δt_p (s² + s − 2)/4 ≥ Δt (§2.4.3).
pub fn stages_for(dt_p: f64, dt: f64) -> usize {
    let need = 4.0 * dt / dt_p;
    let mut s = (((9.0 + 4.0 * need).sqrt() - 1.0) / 2.0).floor().max(2.0) as usize;
    let covers = |s: usize| (s * s + s) as f64 - 2.0 >= need;
    while s > 2 && covers(s - 1) {
        s -= 1;
    }
    while !covers(s) {
        s += 1;
    }
    s
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

    /// The conductance K_f of the face between `a` and `b` (module doc); 0 between two vacuums.
    fn conductance(&self, a: &Side, b: &Side) -> f64 {
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
        let radiative = self.c_sb * self.limiter(r) * self.a_r * (a.t + b.t) * (a.t * a.t + b.t * b.t) / ks;
        radiative + self.k_cond
    }
}

/// The frozen network of a pass: which cells are gas, their heat capacities and the conductance of every face.
struct Net {
    w: usize,
    h: usize,
    gas: Vec<bool>,
    c: Vec<f64>,
    kx: Vec<f64>,
    ky: Vec<f64>,
    t_vac: f64,
}

impl Net {
    /// The four faces of cell (x, y): conductance and the cell index beyond (None: the ghost past the world's edge).
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

    /// dT/dt of every gas cell for temperatures `t`, and the rate each cell loses heat to vacuum faces (energy per area,
    /// per time) — `rate[i] = Σ K_f (T_i − T_vac)` over its faces whose far side is not gas.
    fn apply(&self, t: &[f64], dt_dt: &mut [f64], rate: &mut [f64]) {
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                if !self.gas[i] {
                    dt_dt[i] = 0.0;
                    rate[i] = 0.0;
                    continue;
                }
                let (mut flow, mut out) = (0.0, 0.0);
                for (k, nb) in self.faces(x, y) {
                    match nb {
                        Some(j) if self.gas[j] => flow += k * (t[j] - t[i]),
                        _ => {
                            flow += k * (self.t_vac - t[i]);
                            out += k * (t[i] - self.t_vac);
                        }
                    }
                }
                dt_dt[i] = flow / self.c[i];
                rate[i] = out;
            }
        }
    }
}

/// RKL2's b_j (Meyer et al. 2012): 1/3 for j ≤ 2, (j² + j − 2)/(2j(j + 1)) above.
fn b(j: usize) -> f64 {
    if j <= 2 {
        1.0 / 3.0
    } else {
        let j = j as f64;
        (j * j + j - 2.0) / (2.0 * j * (j + 1.0))
    }
}

impl Heat {
    /// P5 on `st`, in place, over `dt`: E changes by C · ΔT in every gas cell; returns the stages, the booking and F.
    pub fn step(&self, st: &mut State, dt: f64) -> Pass {
        let (w, h) = (st.width, st.height);
        let n = w * h;
        let sides: Vec<Side> = (0..n).map(|i| self.side(st, i)).collect();
        let mut net = Net {
            w,
            h,
            gas: sides.iter().map(|s| s.gas).collect(),
            c: sides.iter().map(|s| s.c()).collect(),
            kx: vec![0.0; (w + 1) * h],
            ky: vec![0.0; w * (h + 1)],
            t_vac: self.eos.t_floor(),
        };
        let mut flux = Flux { width: w, height: h, x: vec![0.0; (w + 1) * h], y: vec![0.0; w * (h + 1)] };
        // One face: the two sides (a vacuum beyond the edge, or beyond a gas cell next to vacuum, is the far side).
        let face = |a: Option<&Side>, b: Option<&Side>| -> (f64, f64) {
            let (a, b) = match (a, b) {
                (Some(a), Some(b)) if a.gas || b.gas => (if a.gas { *a } else { self.vacuum_beyond(b) }, if b.gas { *b } else { self.vacuum_beyond(a) }),
                (Some(a), Some(_)) => (*a, *a),
                (Some(a), None) => (*a, self.vacuum_beyond(a)),
                (None, Some(b)) => (self.vacuum_beyond(b), *b),
                (None, None) => unreachable!("a face has a cell on one side at least"),
            };
            let k = self.conductance(&a, &b);
            (k, -k * (b.t - a.t))
        };
        for y in 0..h {
            for x in 0..=w {
                let (a, b) = (if x > 0 { Some(&sides[y * w + x - 1]) } else { None }, if x < w { Some(&sides[y * w + x]) } else { None });
                let (k, f) = face(a, b);
                net.kx[y * (w + 1) + x] = k;
                flux.x[y * (w + 1) + x] = f;
            }
        }
        for y in 0..=h {
            for x in 0..w {
                let (a, b) = (if y > 0 { Some(&sides[(y - 1) * w + x]) } else { None }, if y < h { Some(&sides[y * w + x]) } else { None });
                let (k, f) = face(a, b);
                net.ky[y * w + x] = k;
                flux.y[y * w + x] = f;
            }
        }

        let max_rate = (0..n)
            .filter(|&i| net.gas[i])
            .map(|i| net.faces(i % w, i / w).iter().map(|(k, _)| k).sum::<f64>() / net.c[i])
            .fold(0.0, f64::max);
        let none = Pass { stages: 0, dt_p: f64::INFINITY, radiated: 0.0, emitted: vec![0.0; n], flux: flux.clone() };
        if self.test.pin_temperature || !(max_rate > 0.0 && dt > 0.0) {
            return Pass { dt_p: if max_rate > 0.0 { 1.0 / max_rate } else { f64::INFINITY }, ..none };
        }
        let dt_p = 1.0 / max_rate;
        let s = stages_for(dt_p, dt);

        let y0: Vec<f64> = sides.iter().map(|s| s.t).collect();
        let (mut m0, mut out0) = (vec![0.0; n], vec![0.0; n]);
        net.apply(&y0, &mut m0, &mut out0);
        let w1 = 4.0 / ((s * s + s) as f64 - 2.0);
        let (mut m, mut out) = (vec![0.0; n], vec![0.0; n]);
        // Y_1 = Y_0 + μ̃_1 Δt M(Y_0); the booking R_j follows the same recursion with the rate in place of M.
        let mt1 = b(1) * w1;
        let mut y_prev2 = y0.clone();
        let mut y_prev: Vec<f64> = (0..n).map(|i| y0[i] + mt1 * dt * m0[i]).collect();
        let mut r_prev2 = vec![0.0; n];
        let mut r_prev: Vec<f64> = (0..n).map(|i| mt1 * dt * out0[i]).collect();
        for j in 2..=s {
            net.apply(&y_prev, &mut m, &mut out);
            let jf = j as f64;
            let mu = (2.0 * jf - 1.0) / jf * b(j) / b(j - 1);
            let nu = -(jf - 1.0) / jf * b(j) / b(j - 2);
            let mt = mu * w1;
            let gt = -(1.0 - b(j - 1)) * mt;
            let y_next: Vec<f64> = (0..n).map(|i| mu * y_prev[i] + nu * y_prev2[i] + (1.0 - mu - nu) * y0[i] + mt * dt * m[i] + gt * dt * m0[i]).collect();
            let r_next: Vec<f64> = (0..n).map(|i| mu * r_prev[i] + nu * r_prev2[i] + mt * dt * out[i] + gt * dt * out0[i]).collect();
            y_prev2 = std::mem::replace(&mut y_prev, y_next);
            r_prev2 = std::mem::replace(&mut r_prev, r_next);
        }
        let energy = 3 * n;
        for i in 0..n {
            if net.gas[i] {
                st.planes[energy + i] += net.c[i] * (y_prev[i] - y0[i]);
            }
        }
        let radiated = r_prev.iter().sum();
        Pass { stages: s, dt_p, radiated, emitted: r_prev, flux }
    }
}
