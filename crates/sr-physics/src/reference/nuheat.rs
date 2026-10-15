//! Neutrino heating on the CPU twin (m0_contrat.md §2.5.4, §1.6.4, §2.7, R13): P7's f64 reference, written before its shader
//! (M0-T54 ports it).
//!
//! The neutrino source S_ν = ε_νΣ (power per area, [`super::react::Pass::s_nu`]) is nonzero only in the newborn neutron star,
//! X_n ≥ [`X_N_STAR`]. P7 hands a share of it to the gas around: D(x) = Σ(x) · (S_ν ∗ K_ν)(x) with K_ν(r) = 1/(r² + 1), r in
//! cells, through [`Convolver`] (the machinery of §1.4.2, zero-padded over the whole world), and D = 0 where X_n ≥ 0.5; the
//! deposited power is q(x) = f_dep · (Σ_y S_ν(y)) · D(x) / Σ_x D(x). Summed over the world that is exactly f_dep of the
//! emitted power whatever D is — it is normalised over ΣD, never per cell — so deposited ≤ emitted holds by construction,
//! f_dep being at most 0.1 while S2 is off (§1.6.4; the registry refuses more). A cell is one unit of area, so q is a power
//! per cell as well as per area; the step adds q Δt to E and books Σ q Δt as `neutrino_deposited` (§2.9).
//!
//! Nothing is emitted when ΣS_ν = 0 (no FFT is run); nothing is deposited when ΣD = 0 (every cell with mass is a neutron
//! star): the energy then stays with the star, which the loss side has already taken from it — the booking says so.
//! Conventions as [`hydro`](super::hydro): channel-major planes in §2.2's order, `index = y * w + x`.

use super::fft::{Convolver, FftError};
use super::hydro::State;
use super::react::X_N_STAR;
use crate::registry::Physics;

/// The species plane of neutron matter (§2.2: channels 4–13, §1.5's slot 9).
const NEUTRON_PLANE: usize = 4 + 9;
/// E's channel (§2.2).
const ENERGY: usize = 3;

/// K_ν(r) = 1/(r² + 1) for an offset of (dx, dy) cells (§2.5.4).
pub fn kernel(dx: isize, dy: isize) -> f64 {
    1.0 / ((dx * dx + dy * dy) as f64 + 1.0)
}

/// What one P7 step did, as powers per area (multiply by Δt for the step's energy).
#[derive(Debug, Clone, PartialEq)]
pub struct Pass {
    /// q(x): the power deposited in each cell, `width × height`, ≥ 0 and exactly 0 in a neutron star.
    pub q: Vec<f64>,
    /// Σ S_ν: the power the neutron stars emitted.
    pub emitted: f64,
    /// Σ q: f_dep × `emitted`, or 0 when no cell outside the stars has mass to take it.
    pub deposited: f64,
}

/// P7's twin: the share f_dep and the convolution's cache, and what has been booked `neutrino_deposited`.
pub struct NuHeat {
    f_dep: f64,
    conv: Convolver,
    /// Booked `neutrino_deposited` (§2.9) by [`NuHeat::apply`] since the twin was made: energy, Σ q Δt.
    pub deposited: f64,
}

impl NuHeat {
    /// P7 for a run's constants (a scene's `overrides` included): f_dep is read from `physics`, whose registry has bounded it.
    pub fn new(physics: &Physics) -> NuHeat {
        NuHeat { f_dep: physics.get("f_dep").expect("physics.json key"), conv: Convolver::new(kernel), deposited: 0.0 }
    }

    /// The share of the emitted power the gas gets.
    pub fn f_dep(&self) -> f64 {
        self.f_dep
    }

    /// q(x) for the state `st` and the source `s_nu` (`width × height`, power per area, ≥ 0).
    pub fn heating(&mut self, st: &State, s_nu: &[f64]) -> Result<Pass, FftError> {
        let n = st.cells();
        assert_eq!(s_nu.len(), n, "S_ν is one plane of {} x {}", st.width, st.height);
        let emitted: f64 = s_nu.iter().sum();
        let mut q = vec![0.0; n];
        if emitted == 0.0 {
            return Ok(Pass { q, emitted, deposited: 0.0 });
        }
        let reach = self.conv.convolve(s_nu, st.width, st.height)?;
        let (sigma, x_n) = (st.plane(0), st.plane(NEUTRON_PLANE));
        // The convolution's round-off can leave −1e-17 where the exact sum is ≥ 0: D is a density of shares, never negative.
        let d: Vec<f64> = (0..n).map(|i| if x_n[i] >= X_N_STAR { 0.0 } else { (sigma[i] * reach[i]).max(0.0) }).collect();
        let total: f64 = d.iter().sum();
        if total > 0.0 {
            let power = self.f_dep * emitted / total;
            for (out, d) in q.iter_mut().zip(&d) {
                *out = power * d;
            }
        }
        let deposited = q.iter().sum();
        Ok(Pass { q, emitted, deposited })
    }

    /// One step of P7: E += q Δt in every cell, the energy Σ q Δt returned and added to [`NuHeat::deposited`].
    pub fn apply(&mut self, st: &mut State, s_nu: &[f64], dt: f64) -> Result<f64, FftError> {
        let pass = self.heating(st, s_nu)?;
        let n = st.cells();
        let mut energy = 0.0;
        for (i, q) in pass.q.iter().enumerate() {
            st.planes[ENERGY * n + i] += q * dt;
            energy += q * dt;
        }
        self.deposited += energy;
        Ok(energy)
    }
}
