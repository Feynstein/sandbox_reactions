//! Sinks — black holes — on the CPU twin (m0_contrat.md §1.4.3, §2.6, §2.9, SM-E28, SM-E29, R13): P3's f64 reference,
//! written before its shader (M0-T51 ports it).
//!
//! A sink is a point mass: position and velocity in cells and cells per t.u., mass M_s; up to [`MAX_SINKS`] (16). Position
//! convention as [`grav::SinkSource`]: cell (i, j) spans [i, i + 1] × [j, j + 1], its centre at (i + ½, j + ½); a sink formed
//! at a cell sits at its centre. P3 runs, in the step's order, [`Sinks::form`], [`Sinks::accrete`], [`Sinks::advance`] and
//! [`Sinks::merge`] (§1.3.2: formation and accretion, then the kick-drift; merging closes the motion, §2.6).
//!
//! *Formation* — all four criteria at one cell (SM-E28: "a sole density threshold … is insufficient"): Σ ≥ Σ_BH; ∇·u < 0
//! (central differences of the velocity, the cell's four neighbours; a vacuum neighbour at rest); φ no greater than at any
//! of its 8 neighbours; bound, ½|u|² + ε_th + φ < 0. A cell on the world's outer ring has no 8 neighbours and does not
//! form. The densest candidate forms (ties: the lowest y, then the lowest x), at most one per step and never past 16.
//! *A sink from mass* (§0.5 [M0-R6]): a cell under Σ_BH is a candidate too when it is a local Σ maximum (no less than its 8
//! neighbours) whose disk of cells with centres r ≤ [`R_CORE`] = 4 from its own holds M_core ≥ m_tov_sb (§2.11 item 2, the
//! sheet's neutron-matter ceiling for the run's physics, [`m_tov_sb`]), the mass-weighted Y_e over r ≤ [`R_Y_E`] = 3 at most
//! [`CORE_Y_E`] = 0.35, and the other three criteria at the cell; such a sink takes the disk's mass above the floor (cell by
//! cell, row-major) and its momentum. A cell at Σ ≥ Σ_BH forms as before, taking its cell alone, unless its disk passes the
//! mass test too.
//!
//! *Accretion* — every step, per sink in index order: r_acc = max(2, 2GM_s/c_sb²) of the sink's mass at the pass's start; a
//! non-vacuum cell whose centre lies within r_acc and whose gas is bound to the sink, ½|u − v_s|² + ε_th − GM_s/max(r, ½)
//! < 0, with M_s and v_s as the pass found them, gives the sink its mass above Σ_floor and that mass's momentum. Both
//! formation and accretion take the share 1 − f of the cell, f = Σ_floor/Σ: Σ, the momentum and E scale by f, the species
//! fractions stay, so the remnant keeps its velocity and its energy per mass. What leaves the cell is the sink's mass and
//! momentum; the gas's thermal and cold energy, (1 − f)(E − ½|Σu|²/Σ), is booked `swallowed` (§2.9) — the kinetic part
//! goes with the momentum. The cells of a sink are read and written in row-major order, one sink after the other (§1.3.4:
//! a fixed order, so a cell two sinks reach goes to the first).
//!
//! *Motion* (§0.5 [M0-V7]) — kick-drift-kick over all sinks together: ½Δt of a, Δt of v, ½Δt of a again at the new positions,
//! a the exact reaction of the sink's own softened pull (§1.4.3) on the gas: Σ_cells G m d / (|d|² + r_acc²)^{3/2}, m a cell's
//! mass (Σ; a cell is one unit of area), d from the sink to the cell's centre, over every cell with Σ ≥ Σ_vac, row-major, on
//! the state after accretion (a cell P3 took this step is left at Σ_floor, below Σ_vac), plus every other sink's pull
//! G M_j d / (d² + r²)^{3/2}, r the pair's larger r_acc — so each pull between a sink and a cell, or two sinks, is equal and
//! opposite and no grid g is read for the motion (formation keeps φ); P3's shader has the same law since M0-D46. *Merging*: two sinks closer than
//! the larger of their r_acc merge — the lowest pair first, again until none is — into the lower slot with the masses and the
//! momenta summed, at the mass-weighted centre. The latch flag P3 sets on a formation is the engine's (M0-T52a).

use super::floors::renormalised;
use super::grav::{Field, SinkSource};
use super::hydro::State;
use super::sheet::{self, ColdMatter};
use crate::eos::Eos;
use crate::registry::{Elements, Physics, RegistryError, N_SPECIES};

/// §2.6: up to 16 sinks.
pub const MAX_SINKS: usize = 16;
/// The smallest r in the bound test's −GM/max(r, ½) (§2.6).
const R_MIN: f64 = 0.5;
/// The species planes of the state (§2.2: channels 4–13) and neutron matter's slot (§1.5).
const FIRST_SPECIES: usize = 4;
const ID_N: usize = 9;
/// §0.5 [M0-R6]: the mass test's disk, its Y_e disk (cells, centre to centre) and the neutronized core's Y_e bound (§0.5
/// [M0-D79]'s).
pub const R_CORE: f64 = 4.0;
pub const R_Y_E: f64 = 3.0;
pub const CORE_Y_E: f64 = 0.35;

/// A sink: position (cells), velocity (cells per t.u.) and mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sink {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub m: f64,
}

/// What one P3 step did.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Report {
    /// The cell a sink formed at, (x, y).
    pub formed: Option<(usize, usize)>,
    /// The mass accretion gave the sinks this step.
    pub accreted: f64,
    /// Sinks lost to merging this step.
    pub merged: usize,
}

/// One cell's gas as the criteria read it.
struct Gas {
    u: [f64; 2],
    eps_th: f64,
}

/// P3's twin: the sinks, the constants and what has been booked `swallowed`.
pub struct Sinks {
    pub sinks: Vec<Sink>,
    /// Booked `swallowed` (§2.9) since the twin was made: the thermal and cold energy of the gas the sinks took.
    pub swallowed: f64,
    eos: Eos,
    elements: Elements,
    g: f64,
    sigma_bh: f64,
    sigma_vac: f64,
    sigma_floor: f64,
    c_sb: f64,
    m_tov: f64,
}

/// m_tov_sb for `physics` (§2.11 item 2): the sheet's neutron-matter ceiling, a few hundredths of a second, never read from
/// assets/calibration.json (as §0.5 [M0-D81]'s m_ch_sb); infinite when the neutrons' cold pressure is off, so no sink forms
/// from mass.
pub fn m_tov_sb(physics: &Physics, elements: &Elements) -> f64 {
    let eos = Eos::from_physics(physics).expect("the run's physics");
    let (Some(n), true) = (elements.index_of("n"), eos.neutron_table().is_some()) else {
        return f64::INFINITY;
    };
    let mut x = [0.0; N_SPECIES];
    x[n] = 1.0;
    let comp = elements.composition(&x);
    let matter = ColdMatter { y_e: comp.y_e, x_n: 1.0, inv_mu: comp.inv_mu };
    let vac = physics.get("sigma_vac").expect("physics.json key");
    sheet::ceiling(&eos, matter, vac).map(|c| c.limit).unwrap_or_else(|e| panic!("the run's physics has the neutrons' cold pressure and no ceiling: {e}"))
}

/// The cells of the disk of radius `r` about cell (x, y), centre to centre, clipped to the world, row-major.
pub fn disk(x: usize, y: usize, r: f64, w: usize, h: usize) -> impl Iterator<Item = usize> {
    let k = r as i64;
    (-k..=k).flat_map(move |dy| (-k..=k).map(move |dx| (dx, dy))).filter_map(move |(dx, dy)| {
        let (cx, cy) = (x as i64 + dx, y as i64 + dy);
        let inside = cx >= 0 && cy >= 0 && cx < w as i64 && cy < h as i64 && ((dx * dx + dy * dy) as f64) <= r * r;
        inside.then_some(cy as usize * w + cx as usize)
    })
}

/// r_acc = max(2 cells, 2GM_s/c_sb²) (§2.6, SM-E29's form).
pub fn r_acc(g: f64, m: f64, c_sb: f64) -> f64 {
    (2.0 * g * m / (c_sb * c_sb)).max(2.0)
}

impl Sinks {
    /// P3 for a run's constants (a scene's `overrides` included), no sink yet.
    pub fn new(physics: &Physics, elements: &Elements, g: f64) -> Result<Sinks, RegistryError> {
        let get = |k: &str| physics.get(k).expect("physics.json key");
        Ok(Sinks {
            sinks: Vec::new(),
            swallowed: 0.0,
            eos: Eos::from_physics(physics)?,
            elements: elements.clone(),
            g,
            sigma_bh: get("sigma_bh"),
            sigma_vac: get("sigma_vac"),
            sigma_floor: get("sigma_floor"),
            c_sb: get("c_sb"),
            m_tov: m_tov_sb(physics, elements),
        })
    }

    /// The m_tov_sb the mass test reads.
    pub fn m_tov(&self) -> f64 {
        self.m_tov
    }

    /// Sink `i`'s r_acc at its current mass.
    pub fn r_acc(&self, i: usize) -> f64 {
        r_acc(self.g, self.sinks[i].m, self.c_sb)
    }

    /// The sinks as P2 adds them to φ and g ([`super::grav::add_sinks`], §1.4.3).
    pub fn sources(&self) -> Vec<SinkSource> {
        (0..self.sinks.len()).map(|i| SinkSource { x: self.sinks[i].x, y: self.sinks[i].y, m: self.sinks[i].m, r_acc: self.r_acc(i) }).collect()
    }

    /// The sinks' mass.
    pub fn mass(&self) -> f64 {
        self.sinks.iter().map(|s| s.m).sum()
    }

    /// The sinks' momentum (Σ M_s v_s), x and y.
    pub fn momentum(&self) -> [f64; 2] {
        self.sinks.iter().fold([0.0; 2], |p, s| [p[0] + s.m * s.vx, p[1] + s.m * s.vy])
    }

    fn gas(&self, st: &State, i: usize) -> Gas {
        let sigma = st.get(0, i);
        let mom = [st.get(1, i), st.get(2, i)];
        let x: [f64; N_SPECIES] = std::array::from_fn(|k| st.get(FIRST_SPECIES + k, i));
        let x = renormalised(&x);
        let comp = self.elements.composition(&x);
        let eps_th = self.eos.thermal(st.get(3, i), sigma, mom, comp.y_e, x[ID_N], 1.0 / comp.inv_mu).eps_th;
        Gas { u: [mom[0] / sigma, mom[1] / sigma], eps_th }
    }

    /// The velocity of cell `i` as ∇·u reads it: at rest below Σ_vac (the floor state).
    fn velocity(&self, st: &State, i: usize) -> [f64; 2] {
        let sigma = st.get(0, i);
        if sigma < self.sigma_vac {
            return [0.0; 2];
        }
        [st.get(1, i) / sigma, st.get(2, i) / sigma]
    }

    /// Takes the share 1 − f of cell `i` (f = Σ_floor/Σ): returns (mass, momentum) given to a sink and books the cell's
    /// thermal and cold energy `swallowed`. The cell keeps f of Σ, the momentum and E, its fractions as they were.
    fn take(&mut self, st: &mut State, i: usize) -> (f64, [f64; 2]) {
        let n = st.cells();
        let (sigma, mom, energy) = (st.planes[i], [st.planes[n + i], st.planes[2 * n + i]], st.planes[3 * n + i]);
        let (f, taken) = (self.sigma_floor / sigma, 1.0 - self.sigma_floor / sigma);
        let thermal_cold = energy - 0.5 * (mom[0] * mom[0] + mom[1] * mom[1]) / sigma;
        self.swallowed += taken * thermal_cold;
        for c in 0..4 {
            st.planes[c * n + i] *= f;
        }
        (taken * sigma, [taken * mom[0], taken * mom[1]])
    }

    /// The mass test (§0.5 [M0-R6]) at cell (x, y): its disk r ≤ 4 holds m_tov_sb or more and the disk r ≤ 3 is neutronized
    /// (mass-weighted Y_e ≤ 0.35).
    fn core(&self, st: &State, x: usize, y: usize) -> bool {
        let (w, h) = (st.width, st.height);
        let mass: f64 = disk(x, y, R_CORE, w, h).map(|i| st.get(0, i)).sum();
        if mass < self.m_tov {
            return false;
        }
        let (mut m, mut ye) = (0.0, 0.0);
        for i in disk(x, y, R_Y_E, w, h) {
            let xs: [f64; N_SPECIES] = std::array::from_fn(|k| st.get(FIRST_SPECIES + k, i));
            m += st.get(0, i);
            ye += st.get(0, i) * self.elements.composition(&renormalised(&xs)).y_e;
        }
        ye <= CORE_Y_E * m
    }

    /// Formation (§2.6, §0.5 [M0-R6]): at most one sink, at the densest cell passing Σ ≥ Σ_BH or the mass test, and the other
    /// three criteria, against the potential `phi` (the gas's and the sinks', §1.4.3); `None` when there is none or 16 sinks
    /// already.
    pub fn form(&mut self, st: &mut State, phi: &[f64]) -> Option<(usize, usize)> {
        let (w, h) = (st.width, st.height);
        if self.sinks.len() >= MAX_SINKS || w < 3 || h < 3 {
            return None;
        }
        let mut best: Option<(f64, usize, usize)> = None;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                let sigma = st.get(0, i);
                if best.is_some_and(|(s, _, _)| sigma <= s) {
                    continue;
                }
                if sigma < self.sigma_bh && !self.peak(st, x, y) {
                    continue;
                }
                let div = (self.velocity(st, i + 1)[0] - self.velocity(st, i - 1)[0]) / 2.0 + (self.velocity(st, i + w)[1] - self.velocity(st, i - w)[1]) / 2.0;
                if div >= 0.0 {
                    continue;
                }
                let lowest = (y - 1..=y + 1).all(|ny| (x - 1..=x + 1).all(|nx| phi[i] <= phi[ny * w + nx]));
                if !lowest {
                    continue;
                }
                let gas = self.gas(st, i);
                if 0.5 * (gas.u[0] * gas.u[0] + gas.u[1] * gas.u[1]) + gas.eps_th + phi[i] >= 0.0 {
                    continue;
                }
                if sigma < self.sigma_bh && !self.core(st, x, y) {
                    continue;
                }
                // Strictly denser wins; rows then columns in order, so a tie keeps the lowest y then the lowest x.
                best = Some((sigma, x, y));
            }
        }
        let (_, x, y) = best?;
        // A sink from mass takes its disk, cell by cell in row-major order (the cells accretion would take: Σ ≥ Σ_vac and
        // above the floor); one from Σ_BH alone its cell.
        let cells: Vec<usize> = if self.peak(st, x, y) && self.core(st, x, y) {
            disk(x, y, R_CORE, w, h).filter(|&i| st.get(0, i) >= self.sigma_vac && st.get(0, i) > self.sigma_floor).collect()
        } else {
            vec![y * w + x]
        };
        let (mut m, mut p) = (0.0, [0.0; 2]);
        for i in cells {
            let (dm, dp) = self.take(st, i);
            m += dm;
            p = [p[0] + dp[0], p[1] + dp[1]];
        }
        self.sinks.push(Sink { x: x as f64 + 0.5, y: y as f64 + 0.5, vx: p[0] / m, vy: p[1] / m, m });
        Some((x, y))
    }

    /// Whether cell (x, y) is a local Σ maximum, no less than at any of its 8 neighbours.
    fn peak(&self, st: &State, x: usize, y: usize) -> bool {
        let (w, sigma) = (st.width, st.get(0, y * st.width + x));
        (y - 1..=y + 1).all(|ny| (x - 1..=x + 1).all(|nx| sigma >= st.get(0, ny * w + nx)))
    }

    /// Accretion (§2.6): every sink in index order takes the bound gas within its r_acc; returns the mass taken.
    pub fn accrete(&mut self, st: &mut State) -> f64 {
        let (w, h) = (st.width, st.height);
        let mut total = 0.0;
        for k in 0..self.sinks.len() {
            let s = self.sinks[k];
            let ra = self.r_acc(k);
            let (x0, x1) = ((s.x - ra).floor().max(0.0) as usize, ((s.x + ra).ceil().max(0.0) as usize).min(w - 1));
            let (y0, y1) = ((s.y - ra).floor().max(0.0) as usize, ((s.y + ra).ceil().max(0.0) as usize).min(h - 1));
            let (mut dm, mut dp) = (0.0, [0.0; 2]);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let i = y * w + x;
                    let (cx, cy) = (x as f64 + 0.5 - s.x, y as f64 + 0.5 - s.y);
                    let r = cx.hypot(cy);
                    if r > ra || st.get(0, i) < self.sigma_vac || st.get(0, i) <= self.sigma_floor {
                        continue;
                    }
                    let gas = self.gas(st, i);
                    let (rx, ry) = (gas.u[0] - s.vx, gas.u[1] - s.vy);
                    if 0.5 * (rx * rx + ry * ry) + gas.eps_th - self.g * s.m / r.max(R_MIN) >= 0.0 {
                        continue;
                    }
                    let (m, p) = self.take(st, i);
                    dm += m;
                    dp[0] += p[0];
                    dp[1] += p[1];
                }
            }
            let m = s.m + dm;
            self.sinks[k] = Sink { m, vx: (s.m * s.vx + dp[0]) / m, vy: (s.m * s.vy + dp[1]) / m, ..s };
            total += dm;
        }
        total
    }

    /// The pull on sink `i` at the positions `at` ([M0-V7]): the sum over the gas of its softened pull's reaction, plus every
    /// other sink's pull softened by the pair's larger r_acc.
    fn acceleration(&self, i: usize, at: &[[f64; 2]], st: &State) -> [f64; 2] {
        let (w, ra) = (st.width, self.r_acc(i));
        let mut a = [0.0; 2];
        for c in 0..st.cells() {
            let sigma = st.get(0, c);
            if sigma < self.sigma_vac {
                continue;
            }
            let (dx, dy) = ((c % w) as f64 + 0.5 - at[i][0], (c / w) as f64 + 0.5 - at[i][1]);
            let soft = dx * dx + dy * dy + ra * ra;
            let pull = self.g * sigma / (soft * soft.sqrt());
            a[0] += pull * dx;
            a[1] += pull * dy;
        }
        let [bx, by] = self.pair_pull(i, at);
        [a[0] + bx, a[1] + by]
    }

    /// The other sinks' pull on sink `i` at the positions `at`, a pair softened by its larger r_acc.
    fn pair_pull(&self, i: usize, at: &[[f64; 2]]) -> [f64; 2] {
        let mut a = [0.0; 2];
        for j in (0..self.sinks.len()).filter(|&j| j != i) {
            let (dx, dy) = (at[j][0] - at[i][0], at[j][1] - at[i][1]);
            let ra = self.r_acc(i).max(self.r_acc(j));
            let soft = dx * dx + dy * dy + ra * ra;
            let pull = self.g * self.sinks[j].m / (soft * soft.sqrt());
            a[0] += pull * dx;
            a[1] += pull * dy;
        }
        a
    }

    /// Each sink's acceleration at its position in the gas `st` (the state after accretion): what [`Sinks::advance`] kicks with.
    pub fn pull(&self, st: &State) -> Vec<[f64; 2]> {
        let at: Vec<[f64; 2]> = self.sinks.iter().map(|s| [s.x, s.y]).collect();
        (0..self.sinks.len()).map(|i| self.acceleration(i, &at, st)).collect()
    }

    /// Kick-drift-kick over all sinks by Δt in the gas `st` (§0.5 [M0-V7]): the pull is the sum over its cells, read at the
    /// start positions and again at the drifted ones.
    pub fn advance(&mut self, st: &State, dt: f64) {
        self.kick_drift_kick(dt, |s, i, at| s.acceleration(i, at, st));
    }

    fn kick_drift_kick(&mut self, dt: f64, pull: impl Fn(&Sinks, usize, &[[f64; 2]]) -> [f64; 2]) {
        let n = self.sinks.len();
        let at: Vec<[f64; 2]> = self.sinks.iter().map(|s| [s.x, s.y]).collect();
        let a: Vec<[f64; 2]> = (0..n).map(|i| pull(self, i, &at)).collect();
        for (s, a) in self.sinks.iter_mut().zip(&a) {
            s.vx += 0.5 * dt * a[0];
            s.vy += 0.5 * dt * a[1];
            s.x += dt * s.vx;
            s.y += dt * s.vy;
        }
        let at: Vec<[f64; 2]> = self.sinks.iter().map(|s| [s.x, s.y]).collect();
        let a: Vec<[f64; 2]> = (0..n).map(|i| pull(self, i, &at)).collect();
        for (s, a) in self.sinks.iter_mut().zip(&a) {
            s.vx += 0.5 * dt * a[0];
            s.vy += 0.5 * dt * a[1];
        }
    }

    /// Merging (§2.6): the lowest pair closer than the larger r_acc becomes one, until none is; returns the sinks lost.
    pub fn merge(&mut self) -> usize {
        let mut lost = 0;
        'again: loop {
            for i in 0..self.sinks.len() {
                for j in i + 1..self.sinks.len() {
                    let (a, b) = (self.sinks[i], self.sinks[j]);
                    if (a.x - b.x).hypot(a.y - b.y) < self.r_acc(i).max(self.r_acc(j)) {
                        let m = a.m + b.m;
                        self.sinks[i] = Sink {
                            x: (a.m * a.x + b.m * b.x) / m,
                            y: (a.m * a.y + b.m * b.y) / m,
                            vx: (a.m * a.vx + b.m * b.vx) / m,
                            vy: (a.m * a.vy + b.m * b.vy) / m,
                            m,
                        };
                        self.sinks.remove(j);
                        lost += 1;
                        continue 'again;
                    }
                }
            }
            return lost;
        }
    }

    /// One P3 step ([M0-V7]): formation against φ of the gas's `grid` with the sinks' potential added, accretion, then the
    /// kick-drift-kick in the gas that accretion left, and merging.
    pub fn step(&mut self, st: &mut State, grid: &Field, dt: f64) -> Report {
        let (formed, accreted) = self.form_and_accrete(st, grid);
        self.advance(st, dt);
        let merged = self.merge();
        Report { formed, accreted, merged }
    }

    fn form_and_accrete(&mut self, st: &mut State, grid: &Field) -> (Option<(usize, usize)>, f64) {
        let (w, h) = (st.width, st.height);
        let mut full = grid.clone();
        super::grav::add_sinks(&mut full, w, h, self.g, &self.sources());
        let formed = self.form(st, &full.phi);
        (formed, self.accrete(st))
    }
}
