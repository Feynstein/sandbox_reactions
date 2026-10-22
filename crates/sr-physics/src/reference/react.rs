//! Burning on the CPU twin (m0_contrat.md §1.6, §1.7, §2.5.1, §2.5.2, §5.1's G-BURN and G-ORDER, R13): P6's f64
//! reference, written before its shader (M0-T46 ports it).
//!
//! Per cell, after the flow step, with the flow and the neighbours out of it (SM M5): each enabled record of the registry
//! converts mass at the specific rate ω of [`Omega`] — its inputs lose `share · ω`, its outputs gain `share · ω`, the cell's
//! thermal energy gains `q · ω` per unit mass (q = q_ratio · Q_H, q < 0 absorbs). The step of Δt is sub-cycled by
//! linearised backward Euler: with S(X) = Σ_r ν_r ω_r(X), ν_r the record's output shares minus its input shares, and J = ∂S/∂X
//! (T and Σ held for the sub-step), a sub-step of h solves `(I − hJ) ΔX = h S`, the stiff-stable step of a 10-species chain
//! that needs no Bader–Deuflhard (SM M5). The conversion each record made is the one the linearisation implies,
//! Δξ_r = h (ω_r + Σ_j ∂ω_r/∂X_j ΔX_j), so ΔX = Σ_r ν_r Δξ_r holds to the solve's rounding and the heat of the sub-step,
//! Σ_r q_r Δξ_r, is the heat of exactly the mass that moved: it enters the specific energy and the group's accumulator at
//! every sub-step, never once per step from the step's first rate.
//!
//! The sub-step h is the largest of: the time left; `burn_dx` over the largest |S_i| (no fraction changes by more than
//! `burn_dx`, absolute); `burn_dt` T over the |dT/dt| the sub-step's own heating and mean-weight change imply (T changes by
//! no more than `burn_dt`, relative). The step is then taken and *checked*: a sub-step that changed a fraction by more than
//! `burn_dx`, T by more than `burn_dt` T, or drove a fraction below zero is retried at half the size (the predictor is a
//! rate, the implicit step is not bound by it). Fractions are clamped ≥ 0 and renormalised to Σ X = 1 after each accepted
//! sub-step. A cell where no record converts anything (every ω is 0 — below T_thr, or its inputs absent) is left bit for bit.
//!
//! Energy: E changes by Σ · Σ_r q_r Δξ_r, exactly the nuclear energy booked; the thermal part is whatever E leaves once the
//! kinetic energy and the cold energy of the *new* composition (Y_e and X_n change as species convert) are taken out, so the
//! cold energy a changed composition shifts comes out of the thermal energy, never from nowhere (§2.9's invariant). T for the
//! rates is μ ε_th with ε_th floored at T_floor/μ (P8 books the floor, not this pass).
//!
//! `pin_temperature` (§2.12.1) holds T at the step's initial value: the rates read it, the sub-steps' ΔT limit is off, and
//! the energy follows the mean weight — ε_th = T/μ of the new composition — so a pinned cell keeps its temperature across
//! steps while the heat is still booked in full (G-BURN's one-cell oracles).
//!
//! Neutrino cooling (§2.5.3, M0-T45), when the run's `neutrinos` switch is on ([`React::with_neutrinos`]): every sub-step also
//! loses ε_ν = A_ν (T/T_ν)^m per unit mass for T ≥ T_ν (0 below), from the T the sub-step starts at, so the energy leaves as
//! `E −= Σ ε_ν h` with the sub-step's heat and the same T limit binds it (`dT/dt` carries `−μ ε_ν`); the loss is booked
//! `neutrino_lost` ([`Pass::neutrino_lost`], energy per area, ≥ 0) and, where the cell leaves the pass with X_n ≥ 0.5 (the
//! newborn neutron star), kept as the source S_ν = ε_νΣ ([`Pass::s_nu`], the pass's mean power per area) for P7. With T pinned
//! the loss is booked and the cell's T held. Cooling moves no mass, so no fraction changes by it.
//!
//! N_Fe in closed form (§0.5 [M0-R10]): N_Fe (f ≡ 1, one input of order 1 into a species of Z = 0, the gate p = 1) is stepped
//! exactly over the pass's Δt *before* the other records sub-cycle, which then skip it — split order N_Fe first, so iron another
//! record makes within the pass waits for the next pass. Read for Y_e with Σ fixed, its law is a constant-coefficient Riccati
//! equation, dY_e/dt = −k (Y_e − a_Y)(Y_e − b), k = A Σ^a Σ/Σ_N, b = Σ_N/Σ, a_Y the Y_e of every species but iron; with
//! d = Y_e − b and E = e^{−k(b − a_Y)Δt}, Y_e − b = d E/(1 + d (1 − E)/(b − a_Y)) ((1 − E)/(b − a_Y) → kΔt as b → a_Y). It is
//! evaluated as the drop ΔY_e = u v φ/(1 + w φ) — u = Y_e − b, v = Y_e − a_Y, w the one of them that decays (u where b ≥ a_Y,
//! the gate closes; v where b < a_Y, the iron runs out), φ = (1 − e^{−k|b − a_Y|Δt})/|b − a_Y| — so no exponential grows and
//! no difference cancels. X_Fe falls by ΔY_e/y_Fe and X_n gains it; its heat q ΔX_Fe enters the energy and `nuclear.N` at once.
//! A gap left under f64's resolution of Y_e lands the cell on its limit: at Y_eΣ ≤ Σ_N, the gate shut, or with no iron. No
//! sub-step is spent on it, so burn_dt's T limit (which at T_floor predicted h ≈ 7.6 × 10⁻⁹ against a T-free rate, M0-R10 M1)
//! never reads it.
//!
//! The collapse latch's P6 condition (§1.8.5): any cell whose N_Fe rate is > 0 at the start of the pass ([`Pass::latch`]).
//! N_Fe's gate (Y_eΣ against Σ_N, p = 1) is in [`Omega`], read here with the cell's own Y_e; q < 0 absorbs.
//!
//! Conventions as [`hydro`](super::hydro): channel-major planes in §2.2's order, `index = y * w + x`. A cell below Σ_vac is not
//! touched.

use super::hydro::State;
use crate::eos::Eos;
use crate::rates::Omega;
use crate::registry::{Elements, Physics, Reactions, RegistryError, GROUPS, N_SPECIES};

const NS: usize = N_SPECIES;
const ID_N: usize = 9;
/// §2.5.3: a cell is a newborn neutron star, the source of S_ν, from this neutron fraction.
pub const X_N_STAR: f64 = 0.5;
/// §1.6.2's id of the record whose rate > 0 sets the latch.
const NFE: &str = "N_Fe";
/// Nuclear groups, the accumulators of §1.9.1 (`nuclear.H` …): [`GROUPS`]' order.
pub const N_GROUPS: usize = GROUPS.len();
/// Attempts (accepted and retried) one cell may spend on one pass before it is left unfinished and reported.
pub const MAX_ATTEMPTS: u64 = 1_000_000;
/// A retry below this fraction of the pass's Δt takes its step as it is, clamped (the half-size loop's floor).
const MIN_H: f64 = 1e-14;
/// N_Fe's closed form leaves a gap under this many ulps of b = Σ_N/Σ: the cell is landed on its limit (module doc).
const LAND_ULPS: f64 = 4.0;

/// §2.12.1's `test`, as the reactions pass reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TestFlags {
    pub pin_temperature: bool,
}

/// One enabled record, ready to evaluate.
#[derive(Debug, Clone)]
struct Rec {
    omega: Omega,
    /// Index into [`GROUPS`].
    group: usize,
    /// q = q_ratio · Q_H, energy per unit mass converted.
    q: f64,
    /// Output shares minus input shares, per species.
    nu: [f64; NS],
    /// (species id, order) of each input.
    inputs: Vec<(usize, f64)>,
}

/// N_Fe's law, the shape the closed form needs (module doc).
#[derive(Debug, Clone, Copy)]
struct Nfe {
    /// Index into `recs`.
    rec: usize,
    /// The input (iron) and the output (neutrons, Z = 0).
    fe: usize,
    n: usize,
    /// The input's Z/A.
    y_fe: f64,
    /// A, a and the gate's Σ_g.
    a_coef: f64,
    a_sigma: f64,
    sigma_g: f64,
}

/// What one pass of P6 did.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pass {
    /// Booked `nuclear.<group>` (§1.9.1, §2.9): energy per area released in each cell, `N_GROUPS` planes of `width × height`
    /// (plane `g` at `g * n`), signed — N_Fe's is ≤ 0.
    pub nuclear: Vec<f64>,
    /// Booked `neutrino_lost` (§2.5.3, §2.9): energy per area the cooling took from each cell, `width × height`, ≥ 0.
    pub neutrino_lost: Vec<f64>,
    /// The neutrino source S_ν (§2.5.3, `side.s_nu`): in a cell that leaves the pass with X_n ≥ [`X_N_STAR`], its
    /// `neutrino_lost` over the pass's Δt (ε_νΣ, the mean power per area), else 0; `width × height`.
    pub s_nu: Vec<f64>,
    /// §1.8.5's P6 condition: some cell's N_Fe rate was > 0 at the start of the pass.
    pub latch: bool,
    /// Accepted sub-steps over every cell.
    pub substeps: u64,
    /// Sub-steps retried at half size (the check refused them).
    pub retried: u64,
    /// The most sub-steps any one cell took.
    pub max_substeps: u64,
    /// The largest change of any fraction in one accepted sub-step (absolute), and of T (relative, 0 with T pinned).
    pub max_dx: f64,
    pub max_dt_rel: f64,
    /// Cells that hit [`MAX_ATTEMPTS`] with time left.
    pub unfinished: usize,
}

impl Pass {
    /// Energy per area released into `group`'s accumulator over the whole world.
    pub fn nuclear_total(&self, group: usize) -> f64 {
        let n = self.nuclear.len() / N_GROUPS;
        self.nuclear[group * n..(group + 1) * n].iter().sum()
    }

    /// Energy per area the cooling took over the whole world.
    pub fn neutrino_lost_total(&self) -> f64 {
        self.neutrino_lost.iter().sum()
    }
}

/// P6's CPU f64 twin: the constants of a run and the registry's enabled records.
#[derive(Debug, Clone)]
pub struct React {
    eos: Eos,
    elements: Elements,
    recs: Vec<Rec>,
    /// Per record of `recs`: it is N_Fe, whose rate > 0 sets the latch.
    latch: Vec<bool>,
    /// N_Fe, stepped in closed form; `None` when it is disabled or its shape is not the closed form's (it then sub-cycles).
    nfe: Option<Nfe>,
    sigma_vac: f64,
    burn_dx: f64,
    burn_dt: f64,
    a_nu: f64,
    t_nu: f64,
    m_nu: f64,
    neutrinos: bool,
    test: TestFlags,
}

/// One cell's result.
struct Cell {
    x: [f64; NS],
    /// Σ q Δξ − Σ ε_ν h over the pass, per unit mass.
    de: f64,
    /// Σ ε_ν h over the pass, per unit mass (booked `neutrino_lost`).
    lost: f64,
    /// N_Fe's rate was > 0 at the start of the pass.
    nfe: bool,
    /// Heat per unit mass released, by group.
    booked: [f64; N_GROUPS],
    /// The pinned T's energy: ε_th + ε_cold of the final composition at T0 (pin mode only).
    pinned_e: Option<f64>,
    substeps: u64,
    retried: u64,
    max_dx: f64,
    max_dt_rel: f64,
    unfinished: bool,
    changed: bool,
}

impl React {
    /// The twin of a run's constants (a scene's `overrides` included — `reactions` was parsed against the same `physics`).
    pub fn new(physics: &Physics, elements: &Elements, reactions: &Reactions, test: TestFlags) -> Result<React, RegistryError> {
        let get = |k: &str| physics.get(k).expect("physics.json key");
        let recs = reactions
            .records()
            .iter()
            .filter(|r| r.enabled)
            .map(|r| {
                let mut nu = [0.0; NS];
                for s in &r.outputs {
                    nu[s.species] += s.share;
                }
                for s in &r.inputs {
                    nu[s.species] -= s.share;
                }
                Rec {
                    omega: Omega::new(r),
                    group: GROUPS.iter().position(|g| *g == r.group).expect("a record's group is one of GROUPS (registry load)"),
                    q: r.q(physics),
                    nu,
                    inputs: r.inputs.iter().zip(&r.rate.orders).map(|(s, &n)| (s.species, n)).collect(),
                }
            })
            .collect();
        let latch = reactions.records().iter().filter(|r| r.enabled).map(|r| r.id == NFE).collect();
        let species = elements.species();
        let nfe = reactions.records().iter().filter(|r| r.enabled).enumerate().find(|(_, r)| r.id == NFE).and_then(|(rec, r)| {
            let rate = &r.rate;
            let gate = rate.gate.as_ref()?;
            let shaped = rate.t_thr.is_none()
                && rate.terms.is_empty()
                && rate.orders == [1.0]
                && r.inputs.len() == 1
                && r.outputs.len() == 1
                && species[r.outputs[0].species].z == 0
                && gate.p == 1.0;
            let fe = &species[r.inputs[0].species];
            shaped.then(|| Nfe {
                rec,
                fe: r.inputs[0].species,
                n: r.outputs[0].species,
                y_fe: f64::from(fe.z) / f64::from(fe.a),
                a_coef: rate.a_coef,
                a_sigma: rate.a,
                sigma_g: gate.sigma_g,
            })
        });
        Ok(React {
            eos: Eos::from_physics(physics)?,
            elements: elements.clone(),
            recs,
            latch,
            nfe,
            sigma_vac: get("sigma_vac"),
            burn_dx: get("burn_dx"),
            burn_dt: get("burn_dt"),
            a_nu: get("a_nu"),
            t_nu: get("t_nu"),
            m_nu: get("m_nu"),
            neutrinos: false,
            test,
        })
    }

    /// The scene's `neutrinos` switch (§2.12.1): cooling on or off. A twin from [`React::new`] has it off, so the passes
    /// that graded burning alone keep their cases; a run that follows the scene's switches (default on) calls this.
    pub fn with_neutrinos(mut self, on: bool) -> React {
        self.neutrinos = on;
        self
    }

    /// ε_ν = A_ν (T/T_ν)^m per unit mass for T ≥ T_ν, else 0 — 0 too with the switch off (§2.5.3).
    pub fn eps_nu(&self, t: f64) -> f64 {
        if self.neutrinos && t >= self.t_nu {
            self.a_nu * (t / self.t_nu).powf(self.m_nu)
        } else {
            0.0
        }
    }

    /// A composition's (μ, Y_e).
    fn comp(&self, x: &[f64; NS]) -> (f64, f64) {
        let c = self.elements.composition(x);
        (1.0 / c.inv_mu, c.y_e)
    }

    /// Burns one cell over `dt`: `x` its fractions, `e_tot` its thermal plus cold energy per unit mass.
    fn burn(&self, sigma: f64, x0: [f64; NS], e_tot0: f64, dt: f64) -> Cell {
        let mut x = x0;
        let mut e_tot = e_tot0;
        let mut cell = Cell { x, de: 0.0, lost: 0.0, nfe: false, booked: [0.0; N_GROUPS], pinned_e: None, substeps: 0, retried: 0, max_dx: 0.0, max_dt_rel: 0.0, unfinished: false, changed: false };
        let (mu0, y_e0) = self.comp(&x);
        let floor = |mu: f64| self.eos.t_floor() / mu;
        let eps_th0 = (e_tot0 - self.eos.eps_cold(sigma, y_e0, x0[ID_N])).max(floor(mu0));
        let t0 = Eos::temperature(eps_th0, mu0);
        // N_Fe first, in closed form over the whole Δt; the latch reads its rate here, at the pass's start.
        if let Some(nfe) = &self.nfe {
            let w_nfe = self.recs[nfe.rec].omega.omega(sigma, &x, t0, y_e0);
            cell.nfe = w_nfe > 0.0;
            if cell.nfe {
                let dm = self.nfe_exact(nfe, sigma, &mut x, dt);
                let heat = self.recs[nfe.rec].q * dm;
                if !self.test.pin_temperature {
                    e_tot += heat;
                }
                cell.de += heat;
                cell.booked[self.recs[nfe.rec].group] += heat;
                cell.changed |= dm > 0.0;
            }
        }
        let nrec = self.recs.len();
        let mut t_done = 0.0;
        let mut h_try = f64::INFINITY;
        let mut attempts = 0u64;
        while t_done < dt {
            let t_rem = dt - t_done;
            let (mu, y_e) = self.comp(&x);
            let t = if self.test.pin_temperature {
                t0
            } else {
                Eos::temperature((e_tot - self.eos.eps_cold(sigma, y_e, x[ID_N])).max(floor(mu)), mu)
            };
            // ω_r and ∂ω_r/∂X_j at the sub-step's start (T, Σ and Y_e held).
            let mut omegas = vec![0.0; nrec];
            let mut grad = vec![[0.0; NS]; nrec];
            let mut s = [0.0; NS];
            let mut qdot = 0.0;
            let eps_nu = self.eps_nu(t);
            for (r, rec) in self.recs.iter().enumerate() {
                if self.nfe.is_some_and(|nfe| nfe.rec == r) {
                    continue;
                }
                let w = rec.omega.omega(sigma, &x, t, y_e);
                omegas[r] = w;
                cell.nfe |= self.latch[r] && w > 0.0;
                // ω = 0 may still have a nonzero derivative (an absent input of order 1): the gradient is always built.
                for &(j, n) in &rec.inputs {
                    if n == 0.0 {
                        continue;
                    }
                    let mut xj = x;
                    xj[j] = 1.0;
                    grad[r][j] += n * x[j].powf(n - 1.0) * rec.omega.omega(sigma, &xj, t, y_e);
                }
                for (si, nu) in s.iter_mut().zip(&rec.nu) {
                    *si += nu * w;
                }
                qdot += rec.q * w;
            }
            if eps_nu == 0.0 && omegas.iter().all(|&w| w == 0.0) {
                break;
            }
            // The sub-step's size.
            let s_max = s.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            let mut h = t_rem;
            if s_max > 0.0 {
                h = h.min(self.burn_dx / s_max);
            }
            if !self.test.pin_temperature {
                let dinv_mu: f64 = (0..NS).map(|i| s[i] * self.species_inv_mu(i)).sum();
                let dt_dt = mu * (qdot - eps_nu) - t * mu * dinv_mu;
                if dt_dt != 0.0 {
                    h = h.min(self.burn_dt * t / dt_dt.abs());
                }
            }
            h = h.min(h_try);
            // Take it, check it, halve until it holds.
            loop {
                attempts += 1;
                if attempts > MAX_ATTEMPTS {
                    cell.unfinished = true;
                    break;
                }
                let last = h >= t_rem;
                let h_use = h.min(t_rem);
                let dx = self.solve(&grad, &s, h_use);
                let mut x_new = x;
                let mut worst = 0.0f64;
                let mut negative = false;
                for i in 0..NS {
                    x_new[i] += dx[i];
                    worst = worst.max(dx[i].abs());
                    if x_new[i] < -1e-12 {
                        negative = true;
                    }
                }
                // Heat of the mass the linearisation moved, by record.
                let mut de = 0.0;
                let mut by_group = [0.0; N_GROUPS];
                for (r, rec) in self.recs.iter().enumerate() {
                    let xi = h_use * (omegas[r] + (0..NS).map(|j| grad[r][j] * dx[j]).sum::<f64>());
                    de += rec.q * xi;
                    by_group[rec.group] += rec.q * xi;
                }
                let lost = eps_nu * h_use;
                for v in x_new.iter_mut() {
                    *v = v.max(0.0);
                }
                let sum: f64 = x_new.iter().sum();
                for v in x_new.iter_mut() {
                    *v /= sum;
                }
                let (mu_new, y_e_new) = self.comp(&x_new);
                let (e_new, t_new) = if self.test.pin_temperature {
                    (e_tot, t0)
                } else {
                    let e = e_tot + de - lost;
                    (e, Eos::temperature((e - self.eos.eps_cold(sigma, y_e_new, x_new[ID_N])).max(floor(mu_new)), mu_new))
                };
                let dt_rel = if self.test.pin_temperature { 0.0 } else { (t_new - t).abs() / t };
                let bad = worst > self.burn_dx * (1.0 + 1e-12) || dt_rel > self.burn_dt * (1.0 + 1e-12) || negative;
                if bad && h_use > MIN_H * dt {
                    h = 0.5 * h_use;
                    cell.retried += 1;
                    continue;
                }
                x = x_new;
                e_tot = e_new;
                cell.de += de - lost;
                cell.lost += lost;
                for (booked, add) in cell.booked.iter_mut().zip(&by_group) {
                    *booked += add;
                }
                cell.substeps += 1;
                cell.changed = true;
                cell.max_dx = cell.max_dx.max(worst);
                cell.max_dt_rel = cell.max_dt_rel.max(dt_rel);
                t_done = if last { dt } else { t_done + h_use };
                h_try = (2.0 * h_use).max(f64::MIN_POSITIVE);
                break;
            }
            if cell.unfinished {
                break;
            }
        }
        cell.x = x;
        if self.test.pin_temperature && cell.changed {
            let (mu, y_e) = self.comp(&x);
            cell.pinned_e = Some(t0 / mu + self.eos.eps_cold(sigma, y_e, x[ID_N]));
        }
        cell
    }

    /// (1/μ)'s derivative with respect to X_i: (Z_i + 1)/A_i.
    fn species_inv_mu(&self, i: usize) -> f64 {
        let s = &self.elements.species()[i];
        (f64::from(s.z) + 1.0) / f64::from(s.a)
    }

    /// N_Fe over `dt` in closed form (module doc, §0.5 [M0-R10]), Σ fixed: moves the mass converted from iron to neutrons in
    /// `x` and returns it (≥ 0, at most the iron there was). A cell under its gate or without iron is left as it is.
    fn nfe_exact(&self, nfe: &Nfe, sigma: f64, x: &mut [f64; NS], dt: f64) -> f64 {
        let b = nfe.sigma_g / sigma;
        let mut others = *x;
        others[nfe.fe] = 0.0;
        let a_y = self.comp(&others).1;
        let c = b - a_y;
        // v = Y_e − a_Y (the iron's electrons), u = Y_e − b (the gate's gap).
        let v = nfe.y_fe * x[nfe.fe];
        let u = v - c;
        if !(u > 0.0 && v > 0.0) || dt <= 0.0 {
            return 0.0;
        }
        let k = nfe.a_coef * sigma.powf(nfe.a_sigma) / b;
        let (w, s) = if c >= 0.0 { (u, c) } else { (v, -c) };
        let ks = k * s * dt;
        let phi = if ks > 0.0 { -(-ks).exp_m1() / s } else { k * dt };
        let dy = u * v * phi / (1.0 + w * phi);
        let mut dm = (dy / nfe.y_fe).min(x[nfe.fe]);
        // What the closed form leaves of the decaying gap: under f64's resolution the cell is on its limit.
        let left = w * (-ks).exp() / (1.0 + w * phi);
        if left <= LAND_ULPS * f64::EPSILON * b {
            if c >= 0.0 {
                x[nfe.fe] -= dm;
                x[nfe.n] += dm;
                let mut nudge = f64::EPSILON * x[nfe.fe];
                for _ in 0..16 {
                    let y_e = self.comp(x).1;
                    if y_e * sigma <= nfe.sigma_g || x[nfe.fe] == 0.0 {
                        break;
                    }
                    let more = ((y_e * sigma - nfe.sigma_g) / (sigma * nfe.y_fe)).max(nudge).min(x[nfe.fe]);
                    x[nfe.fe] -= more;
                    x[nfe.n] += more;
                    dm += more;
                    nudge *= 2.0;
                }
                return dm;
            }
            dm = x[nfe.fe];
        }
        x[nfe.fe] -= dm;
        x[nfe.n] += dm;
        dm
    }

    /// The linearised backward-Euler step: ΔX with (I − hJ) ΔX = h S, J = Σ_r ν_r ∇ω_r.
    fn solve(&self, grad: &[[f64; NS]], s: &[f64; NS], h: f64) -> [f64; NS] {
        // A = I − h J, row i: A[i][j] = δ_ij − h Σ_r ν_ri g_rj; the right side h S.
        let mut a = [[0.0; NS + 1]; NS];
        for i in 0..NS {
            a[i][i] = 1.0;
            a[i][NS] = h * s[i];
            for (r, rec) in self.recs.iter().enumerate() {
                if rec.nu[i] == 0.0 {
                    continue;
                }
                for j in 0..NS {
                    a[i][j] -= h * rec.nu[i] * grad[r][j];
                }
            }
        }
        // Gaussian elimination with partial pivoting.
        for col in 0..NS {
            let piv = (col..NS).max_by(|&p, &q| a[p][col].abs().total_cmp(&a[q][col].abs())).unwrap();
            a.swap(col, piv);
            let d = a[col][col];
            if d == 0.0 {
                continue;
            }
            for row in col + 1..NS {
                let f = a[row][col] / d;
                if f != 0.0 {
                    let pivot_row = a[col];
                    for (v, p) in a[row][col..].iter_mut().zip(&pivot_row[col..]) {
                        *v -= f * p;
                    }
                }
            }
        }
        let mut dx = [0.0; NS];
        for i in (0..NS).rev() {
            let mut v = a[i][NS];
            for j in i + 1..NS {
                v -= a[i][j] * dx[j];
            }
            dx[i] = if a[i][i] == 0.0 { 0.0 } else { v / a[i][i] };
        }
        dx
    }

    /// P6 on `st`, in place, over `dt`: each cell burns alone (module doc); returns what was booked and the sub-cycle counts.
    pub fn step(&self, st: &mut State, dt: f64) -> Pass {
        let n = st.width * st.height;
        let mut pass = Pass { nuclear: vec![0.0; N_GROUPS * n], neutrino_lost: vec![0.0; n], s_nu: vec![0.0; n], ..Pass::default() };
        for i in 0..n {
            let sigma = st.get(0, i);
            if sigma < self.sigma_vac {
                continue;
            }
            let x: [f64; NS] = std::array::from_fn(|s| st.get(4 + s, i));
            let kinetic = 0.5 * (st.get(1, i).powi(2) + st.get(2, i).powi(2)) / (sigma * sigma);
            let e_tot = st.get(3, i) / sigma - kinetic;
            let cell = self.burn(sigma, x, e_tot, dt);
            pass.substeps += cell.substeps;
            pass.retried += cell.retried;
            pass.max_substeps = pass.max_substeps.max(cell.substeps);
            pass.max_dx = pass.max_dx.max(cell.max_dx);
            pass.max_dt_rel = pass.max_dt_rel.max(cell.max_dt_rel);
            pass.unfinished += usize::from(cell.unfinished);
            pass.latch |= cell.nfe;
            if !cell.changed {
                continue;
            }
            for (s, v) in cell.x.iter().enumerate() {
                st.planes[(4 + s) * n + i] = *v;
            }
            let e = match cell.pinned_e {
                Some(e_pinned) => sigma * (e_pinned + kinetic),
                None => st.get(3, i) + sigma * cell.de,
            };
            st.planes[3 * n + i] = e;
            for g in 0..N_GROUPS {
                pass.nuclear[g * n + i] = sigma * cell.booked[g];
            }
            pass.neutrino_lost[i] = sigma * cell.lost;
            if cell.x[ID_N] >= X_N_STAR && dt > 0.0 {
                pass.s_nu[i] = sigma * cell.lost / dt;
            }
        }
        pass
    }
}
