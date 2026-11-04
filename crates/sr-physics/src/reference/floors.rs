//! P8 on the CPU twin (m0_contrat.md §1.3.2, §2.3.3, §2.7 [M0-T19], §2.9, R13): the floors in f64, written to match
//! shaders/floors.wgsl and moved here from the inline twin of tests/gpu/floors.rs (M0-T37).
//!
//! The dispatch order is the vacuum reset and the temperature floor, then the species renormalised. A cell's fractions
//! are clamped ≥ 0 and divided by their sum, pure hydrogen when none is positive; μ, Y_e and X_n are those of the
//! renormalised fractions (the gas the step ends with). A cell below Σ_vac is reset to the floor state — Σ_floor, u = 0,
//! T = T_floor, its fractions kept — and the mass it loses is booked `vacuum_reset` (signed: Σ − Σ_floor); any other cell
//! whose ε_th = E/Σ − ½|u|² − ε_cold is below T_floor/μ is raised to it. Every energy P8 changes, the reset's E_floor − E
//! and the floor's alike, is booked `floor_added` (§2.7 [M0-T19]); the momentum a reset removes is booked nowhere.
//!
//! Σ_vac and Σ_floor are compared and written as the f32 the GPU holds (the planes are f32 there); T_floor stays f64, as
//! the GPU's EOS table reads it. A cell holding an Inf or a NaN fraction is left as read, nothing booked, for P9's
//! guard (M0-D18).
//!
//! The hold (§2.7 [M0-TG2]; M0-R11's H), P8's last act ([`Floors::hold`]): a cell with Σ ≥ σ_hold whose ε_th — as the
//! floor leaves it — is at most η_hold · ε_cold, which holds no iron (X_Fe = 0, §2.7 [M0-D71f]) or is neutron-rich (X_n ≥
//! 0.5, §2.7 [M0-R13]), and whose disk r ≤ 24 holds less than m_tov_sb (§0.5 [M0-T130]: P3's mass test's disk form and
//! limit, sinks.rs [`disk`] and [`m_tov_sb`], at [`R_HOLD`], read on Σ as the floors leave it; M0-D153) is held, and the next step's P4 leaves it
//! as it is (hydro.rs's held walls). A held cell is released past 2η_hold · ε_cold, under ½σ_hold, once X_Fe > 0 with X_n
//! < 0.5, or once its disk holds m_tov_sb: a collapsing iron core stays free, a remnant's neutron core is held degenerate,
//! and a core heading past m_tov_sb is free for P3's sink. A newly held cell's momentum is set to 0 — its kinetic energy
//! left in E, where it becomes heat — and booked `held`. ε_th and ε_cold are read in f64 of the renormalised fractions.

use super::hydro::State;
use super::sinks::{disk, m_tov_sb};
use crate::eos::Eos;
use crate::registry::{Elements, Physics, RegistryError, N_SPECIES};

const NS: usize = N_SPECIES;
/// The neutron fraction's slot in §1.5's order (the registry's `ID_N`).
const ID_N: usize = 9;
/// Iron's slot in §1.5's order: a cell holding any is held only once neutron-rich (§2.7 [M0-D71f], [M0-R13]).
const ID_FE: usize = 8;
/// X_n from which a cell holding iron is neutron-rich, past N_Fe's gate (§2.7 [M0-R13]).
const NEUTRON_RICH: f64 = 0.5;
/// σ_hold: the least Σ a held cell has (§2.7 [M0-TG2]; a code constant until M0-D71g moves it into physics.json).
pub const SIGMA_HOLD: f64 = 10.0;
/// η_hold: a cell is held at ε_th ≤ η_hold · ε_cold and released past twice that (§2.7 [M0-TG2]; as [`SIGMA_HOLD`]).
pub const ETA_HOLD: f64 = 0.1;
/// The hold's gate radius (§0.5 [M0-T130], M0-D153): a cell is held only while the mass within `R_HOLD` cells of it, centre
/// to centre, is under m_tov_sb — P3's disk form (§0.5 [M0-R6]) at a radius wide enough to take a collapsing clump's infall
/// (r ≤ 4 froze the 20 M₀ life's clumps at birth, M0-T130).
pub const R_HOLD: f64 = 24.0;
/// The state's species planes (§2.2: channels 4–13).
const FIRST_SPECIES: usize = 4;

/// The fractions as P8 leaves them: clamped ≥ 0 and divided by their sum, pure hydrogen when none is positive.
pub fn renormalised(x: &[f64; NS]) -> [f64; NS] {
    let clamped = x.map(|v| v.max(0.0));
    let sum: f64 = clamped.iter().sum();
    let mut out = [0.0; NS];
    if sum > 0.0 {
        for (o, v) in out.iter_mut().zip(&clamped) {
            *o = v / sum;
        }
    } else {
        out[0] = 1.0;
    }
    out
}

/// One cell after P8 and what P8 books for it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub sigma: f64,
    pub mom: [f64; 2],
    pub energy: f64,
    /// The renormalised fractions.
    pub x: [f64; NS],
    /// `vacuum_reset`: Σ before − Σ after, 0 outside a reset.
    pub mass_out: f64,
    /// `floor_added`: E after − E before.
    pub energy_added: f64,
    pub reset: bool,
    pub floored: bool,
}

/// What P8 booked, summed over cells (§2.9's `vacuum_reset` and `floor_added`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Booked {
    pub vacuum_reset: f64,
    pub floor_added: f64,
}

impl std::ops::AddAssign for Booked {
    fn add_assign(&mut self, other: Booked) {
        self.vacuum_reset += other.vacuum_reset;
        self.floor_added += other.floor_added;
    }
}

/// P8's constants for a run (a scene's `overrides` included).
#[derive(Debug, Clone)]
pub struct Floors {
    eos: Eos,
    elements: Elements,
    sigma_vac: f64,
    sigma_floor: f64,
    /// m_tov_sb for the run's physics (§2.11 item 2, sinks.rs [`m_tov_sb`]): a cell whose disk r ≤ 24 holds it is free.
    m_tov: f64,
}

impl Floors {
    pub fn new(physics: &Physics, elements: &Elements) -> Result<Floors, RegistryError> {
        let get = |k: &str| f64::from(physics.get(k).expect("physics.json key") as f32);
        Ok(Floors {
            eos: Eos::from_physics(physics)?,
            elements: elements.clone(),
            sigma_vac: get("sigma_vac"),
            sigma_floor: get("sigma_floor"),
            m_tov: m_tov_sb(physics, elements),
        })
    }

    /// P8 on one cell: `raw` are the fractions as read (not yet renormalised).
    pub fn cell(&self, sigma: f64, mom: [f64; 2], energy: f64, raw: [f64; NS]) -> Cell {
        let x = renormalised(&raw);
        let comp = self.elements.composition(&x);
        let mu = 1.0 / comp.inv_mu;
        if sigma < self.sigma_vac {
            let s = self.sigma_floor;
            let e = s * (Eos::eps_th_of_t(self.eos.t_floor(), mu) + self.eos.eps_cold(s, comp.y_e, x[ID_N]));
            return Cell {
                sigma: s,
                mom: [0.0, 0.0],
                energy: e,
                x,
                mass_out: sigma - s,
                energy_added: e - energy,
                reset: true,
                floored: false,
            };
        }
        let t = self.eos.thermal(energy, sigma, mom, comp.y_e, x[ID_N], mu);
        Cell {
            sigma,
            mom,
            energy: energy + t.floor_added,
            x,
            mass_out: 0.0,
            energy_added: t.floor_added,
            reset: false,
            floored: t.floor_added > 0.0,
        }
    }

    /// P8 over the whole world, in place; returns what it booked.
    pub fn apply(&self, st: &mut State) -> Booked {
        let n = st.cells();
        let mut booked = Booked::default();
        for i in 0..n {
            let raw: [f64; NS] = std::array::from_fn(|k| st.planes[(FIRST_SPECIES + k) * n + i]);
            if raw.iter().any(|v| !v.is_finite()) {
                continue;
            }
            let c = self.cell(st.planes[i], [st.planes[n + i], st.planes[2 * n + i]], st.planes[3 * n + i], raw);
            st.planes[i] = c.sigma;
            st.planes[n + i] = c.mom[0];
            st.planes[2 * n + i] = c.mom[1];
            st.planes[3 * n + i] = c.energy;
            for (k, v) in c.x.iter().enumerate() {
                st.planes[(FIRST_SPECIES + k) * n + i] = *v;
            }
            booked += Booked { vacuum_reset: c.mass_out, floor_added: c.energy_added };
        }
        booked
    }

    /// m_tov_sb as the hold reads it.
    pub fn m_tov(&self) -> f64 {
        self.m_tov
    }

    /// The mass of cell `i`'s disk r ≤ [`R_HOLD`] (sinks.rs [`disk`], row-major, P3's mass test's form), the hold's gate.
    pub fn disk_mass(st: &State, i: usize) -> f64 {
        disk(i % st.width, i / st.width, R_HOLD, st.width, st.height).map(|c| st.planes[c]).sum()
    }

    /// Whether a cell of Σ `sigma`, momentum `mom`, energy `energy` and fractions `x`, whose disk r ≤ 24 holds `m_disk()`,
    /// is held after P8, given whether it was held before (§2.7 [M0-TG2], [M0-D71f], [M0-R13]; §0.5 [M0-T130]): the
    /// hold's test, or its release with the hysteresis. The disk gate reads on both kinds alike, so no cell flickers at it;
    /// `m_disk` is called only where the rest of the test holds or keeps the cell (so only at Σ ≥ ½σ_hold) — the answer
    /// is the same, the 1,809-cell sum spared everywhere else.
    pub fn holds(&self, sigma: f64, mom: [f64; 2], energy: f64, x: &[f64; NS], m_disk: impl FnOnce() -> f64, was_held: bool) -> bool {
        let x = renormalised(x);
        let comp = self.elements.composition(&x);
        let cold = self.eos.eps_cold(sigma, comp.y_e, x[ID_N]);
        let eps_th = energy / sigma - 0.5 * (mom[0] * mom[0] + mom[1] * mom[1]) / (sigma * sigma) - cold;
        let kind = x[ID_FE] == 0.0 || x[ID_N] >= NEUTRON_RICH;
        let rest = if was_held {
            !(eps_th > 2.0 * ETA_HOLD * cold || sigma < 0.5 * SIGMA_HOLD || !kind)
        } else {
            sigma >= SIGMA_HOLD && eps_th <= ETA_HOLD * cold && kind
        };
        rest && m_disk() < self.m_tov
    }

    /// The hold over the whole world, in place, after [`Floors::apply`]: `held` (one flag per cell) becomes the mask the
    /// next step's P4 reads; a newly held cell's momentum is zeroed and returned summed — the momentum booked `held`.
    pub fn hold(&self, st: &mut State, held: &mut [bool]) -> [f64; 2] {
        let n = st.cells();
        assert_eq!(held.len(), n, "one held flag per cell");
        let mut taken = [0.0; 2];
        for (i, h) in held.iter_mut().enumerate() {
            let raw: [f64; NS] = std::array::from_fn(|k| st.planes[(FIRST_SPECIES + k) * n + i]);
            if raw.iter().any(|v| !v.is_finite()) {
                continue;
            }
            let mom = [st.planes[n + i], st.planes[2 * n + i]];
            let now = self.holds(st.planes[i], mom, st.planes[3 * n + i], &raw, || Self::disk_mass(st, i), *h);
            if now && !*h {
                taken[0] += mom[0];
                taken[1] += mom[1];
                st.planes[n + i] = 0.0;
                st.planes[2 * n + i] = 0.0;
            }
            *h = now;
        }
        taken
    }
}
