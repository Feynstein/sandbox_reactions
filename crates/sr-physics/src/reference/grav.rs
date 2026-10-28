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
//! central difference reads φ one cell past the box, which the zero-padded convolution holds exactly (free space, no
//! source there: [`Convolver::convolve_ringed`]). So Σ m·g cancels by the kernel's symmetry even with matter on that
//! ring — a box clamped to a world edge (M0-D69: the one-sided difference there leaked a collapse's momentum).

use super::fft::{Convolver, FftError};

/// The convolution's ring past the box that g's central difference reads (M0-D69).
pub const RING: usize = 1;

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

/// g = −∇φ by central differences over the `w` × `h` box, from φ on the box grown by [`RING`] cell per side
/// ([`Convolver::convolve_ringed`]'s layout), so the outermost ring's difference reads free space's φ past the box.
pub fn acceleration(phi_ringed: &[f64], w: usize, h: usize) -> Result<(Vec<f64>, Vec<f64>), FftError> {
    let rw = w + 2 * RING;
    if phi_ringed.len() != rw * (h + 2 * RING) {
        return Err(FftError::LengthMismatch { expected: rw * (h + 2 * RING), found: phi_ringed.len() });
    }
    let mut gx = vec![0.0; w * h];
    let mut gy = vec![0.0; w * h];
    let at = |x: usize, y: usize| phi_ringed[y * rw + x];
    for y in 0..h {
        for x in 0..w {
            // the box's (x, y) sits at (x + 1, y + 1) of the ringed field
            gx[y * w + x] = -(at(x + 2, y + 1) - at(x, y + 1)) / 2.0;
            gy[y * w + x] = -(at(x + 1, y + 2) - at(x + 1, y)) / 2.0;
        }
    }
    Ok((gx, gy))
}

/// A sink as P2 sees it (§1.4.3): its position in cells (cell (i, j) spans [i, i + 1] × [j, j + 1], its centre at
/// (i + ½, j + ½)), its mass and its softening length r_acc (§2.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SinkSource {
    pub x: f64,
    pub y: f64,
    pub m: f64,
    pub r_acc: f64,
}

/// Adds the sinks' pull to a field (§1.4.3): φ at each cell's centre gains −G M_s / √(r² + r_acc²), r from the sink's
/// position, and g gains its exact gradient −G M_s d / (r² + r_acc²)^{3/2}, d the vector from the sink to the cell
/// centre (toward the sink). The sink's term is analytic, not convolved: the gas feels it in full inside r_acc.
pub fn add_sinks(field: &mut Field, w: usize, h: usize, g: f64, sinks: &[SinkSource]) {
    for s in sinks {
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = (x as f64 + 0.5 - s.x, y as f64 + 0.5 - s.y);
                let soft = dx * dx + dy * dy + s.r_acc * s.r_acc;
                let i = y * w + x;
                field.phi[i] -= g * s.m / soft.sqrt();
                let pull = g * s.m / (soft * soft.sqrt());
                field.gx[i] -= pull * dx;
                field.gy[i] -= pull * dy;
            }
        }
    }
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
        let ringed = self.conv.convolve_ringed(&source(sigma, sigma_vac), w, h, RING)?;
        let (gx, gy) = acceleration(&ringed, w, h)?;
        let rw = w + 2 * RING;
        let phi = (0..h).flat_map(|y| ringed[(y + RING) * rw + RING..][..w].to_vec()).collect();
        Ok(Field { phi, gx, gy })
    }

    /// [`Gravity::field`] with the sinks' potential and pull added ([`add_sinks`], §1.4.3).
    pub fn field_with_sinks(&mut self, sigma: &[f64], sigma_vac: f64, w: usize, h: usize, g: f64, sinks: &[SinkSource]) -> Result<Field, FftError> {
        let mut field = self.field(sigma, sigma_vac, w, h)?;
        add_sinks(&mut field, w, h, g, sinks);
        Ok(field)
    }
}
