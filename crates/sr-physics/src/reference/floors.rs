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

use super::hydro::State;
use crate::eos::Eos;
use crate::registry::{Elements, Physics, RegistryError, N_SPECIES};

const NS: usize = N_SPECIES;
/// The neutron fraction's slot in §1.5's order (the registry's `ID_N`).
const ID_N: usize = 9;
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
}

impl Floors {
    pub fn new(physics: &Physics, elements: &Elements) -> Result<Floors, RegistryError> {
        let get = |k: &str| f64::from(physics.get(k).expect("physics.json key") as f32);
        Ok(Floors { eos: Eos::from_physics(physics)?, elements: elements.clone(), sigma_vac: get("sigma_vac"), sigma_floor: get("sigma_floor") })
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
}
