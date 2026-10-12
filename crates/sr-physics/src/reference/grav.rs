//! Gravity on the CPU twin (m0_contrat.md §1.4.1–§1.4.2, §1.7, R13): φ = K ∗ Σ_src over the active box by the
//! zero-padded convolution of [`fft`](super::fft), and g = −∇φ by central differences. The pass's f64 reference,
//! written before its shader.
//!
//! The world is a razor-thin sheet in 3D space (§1.4.1): every cell pulls every other with the 3D law, r measured in the
//! plane. K(i, j) = −G / √(i² + j²) for (i, j) ≠ (0, 0); K(0, 0) = −G · 4 ln(1 + √2), the exact potential at a unit
//! square's centre from its own uniform mass — not Plummer softening (SM-C1: softening ε = 0.5 doubled the force error).
//! Free space in both edge modes: no image masses (SM-E8). Σ_src is Σ in non-vacuum cells and 0 in vacuum.
//!
//! Conventions as [`fft`](super::fft): row-major, `index = y * w + x`, y down (§2.12.4), so `gy > 0` pulls toward larger
//! rows. The box is the field's own extent: the caller crops to the active box (§1.3.3). At the box's outermost ring the
//! central difference has no neighbour outside, so it falls back to the one-sided difference there; the box carries a
//! margin of vacuum (§1.3.3), so no matter sits on that ring.

use super::fft::{Convolver, FftError};

/// −K(0, 0) / G = 4 ln(1 + √2) = 3.525494 (§1.4.2, my arithmetic there): ∫∫ over [−½, ½]² of dA / r.
pub const SELF_CELL: f64 = 3.525_494_348_078_172;

/// K(dx, dy), the potential at a cell `(dx, dy)` away from a unit source of Σ (§1.4.2).
pub fn kernel(g: f64, dx: isize, dy: isize) -> f64 {
    if dx == 0 && dy == 0 {
        return -g * SELF_CELL;
    }
    let r2 = (dx * dx + dy * dy) as f64;
    -g / r2.sqrt()
}

/// Σ_src: Σ where the cell is not vacuum (Σ ≥ `sigma_vac`, the box's own test, §1.3.3), 0 in vacuum.
pub fn source(sigma: &[f64], sigma_vac: f64) -> Vec<f64> {
    sigma.iter().map(|&s| if s >= sigma_vac { s } else { 0.0 }).collect()
}

/// The potential and the acceleration over a `w` × `h` box.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub phi: Vec<f64>,
    pub gx: Vec<f64>,
    pub gy: Vec<f64>,
}

/// g = −∇φ by central differences (one-sided on the box's outermost ring, an axis of one cell has none).
pub fn acceleration(phi: &[f64], w: usize, h: usize) -> Result<(Vec<f64>, Vec<f64>), FftError> {
    if phi.len() != w * h {
        return Err(FftError::LengthMismatch { expected: w * h, found: phi.len() });
    }
    let mut gx = vec![0.0; w * h];
    let mut gy = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let at = |x: usize, y: usize| phi[y * w + x];
            if w > 1 {
                let (lo, hi) = (x.saturating_sub(1), (x + 1).min(w - 1));
                gx[y * w + x] = -(at(hi, y) - at(lo, y)) / (hi - lo) as f64;
            }
            if h > 1 {
                let (lo, hi) = (y.saturating_sub(1), (y + 1).min(h - 1));
                gy[y * w + x] = -(at(x, hi) - at(x, lo)) / (hi - lo) as f64;
            }
        }
    }
    Ok((gx, gy))
}

/// The gravity pass: one convolver, its kernel's transform cached per padded size.
pub struct Gravity {
    conv: Convolver,
}

impl Gravity {
    /// A gravity pass for the constant `g` (G = 1 in the sandbox's units, §2.1).
    pub fn new(g: f64) -> Gravity {
        Gravity { conv: Convolver::new(move |dx, dy| kernel(g, dx, dy)) }
    }

    /// φ = K ∗ Σ_src over the `w` × `h` box; `sigma_src` is already masked ([`source`]).
    pub fn potential(&mut self, sigma_src: &[f64], w: usize, h: usize) -> Result<Vec<f64>, FftError> {
        self.conv.convolve(sigma_src, w, h)
    }

    /// φ and g = −∇φ from Σ over the `w` × `h` box, vacuum (Σ < `sigma_vac`) not a source.
    pub fn field(&mut self, sigma: &[f64], sigma_vac: f64, w: usize, h: usize) -> Result<Field, FftError> {
        let phi = self.potential(&source(sigma, sigma_vac), w, h)?;
        let (gx, gy) = acceleration(&phi, w, h)?;
        Ok(Field { phi, gx, gy })
    }
}
