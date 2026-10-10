//! Sandbox units (m0_contrat.md §2.1). Length is one cell (Δx = 1), G = 1, and the mass unit is the one that gives the
//! Sun-like preset's cloud a central surface density Σc = 1 at radius a = 40 cells. One time unit is
//! √(cell³ / (G · mass unit)); with G = 1 and Δx = 1 it needs no constant of its own. Temperature is a specific
//! energy (cell² / t.u.²). Real values never enter the physics — only §1.10's display translation.

use std::f64::consts::PI;

/// The gravitational constant, 1 by the choice of the mass unit.
pub const G: f64 = 1.0;
/// Cell edge (Δx = Δy), the length unit.
pub const DX: f64 = 1.0;
/// Central surface density of the Sun-like cloud, the mass unit's definition.
pub const SIGMA_C: f64 = 1.0;
/// Radius of the Sun-like cloud, in cells.
pub const A_SUN: f64 = 40.0;

/// Mass of a Maclaurin disk Σ(r) = Σc √(1 − r²/a²): M = 2πΣc a²/3 (§2.1, §2.10).
pub const fn disk_mass(sigma_c: f64, a: f64) -> f64 {
    2.0 * PI * sigma_c * a * a / 3.0
}

/// The nominal Sun-like mass M₀ = 2πΣc a²/3 at Σc = 1, a = 40, ≈ 3,351. The calibrated `preset_mass_sb.sun` (§2.10)
/// sets the exact figure the presets use; this is the unit's definition.
pub const M0: f64 = disk_mass(SIGMA_C, A_SUN);
