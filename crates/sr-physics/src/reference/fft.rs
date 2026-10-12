//! The twin's FFT and convolution (m0_contrat.md §1.3.3, §1.4.2, §2.5.4): a radix-2 complex FFT in f64, 2D transforms,
//! and the zero-padded convolution of a field with a kernel over the active box (Hockney–Eastwood). Written here:
//! §1.2's table holds no FFT crate.
//!
//! Conventions: row-major, `index = y * nx + x`; the forward transform is X_k = Σ x_n e^{−2πi kn/N}, unnormalised; the
//! inverse is conjugated and divided by N, so inverse(forward(x)) = x. A convolution over a w × h box pads each axis to
//! the smallest power of two ≥ 2 × its length, at least 32 (§1.3.3), so that every offset x − y in (−w, w) stays
//! distinct in the periodic kernel and nothing wraps around. The kernel is sampled at signed offsets, K(dx, dy) with
//! dx = x − x_source, and is not assumed symmetric.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::ops::{Add, Mul, Sub};

/// The smallest padded size per axis (§1.3.3).
pub const MIN_PADDED: usize = 32;
/// The largest padded size per axis (§1.3.3).
pub const MAX_PADDED: usize = 2048;

/// What a transform or a convolution refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FftError {
    /// A transform length that is not a power of two.
    NotPowerOfTwo(usize),
    /// A box axis that is empty or whose padded size would exceed [`MAX_PADDED`].
    BoxOutOfRange(usize),
    /// A buffer whose length is not the product of its axes.
    LengthMismatch { expected: usize, found: usize },
}

impl fmt::Display for FftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FftError::NotPowerOfTwo(n) => write!(f, "FFT length {n} is not a power of two"),
            FftError::BoxOutOfRange(n) => write!(f, "box axis of {n} cells pads outside {MIN_PADDED}..={MAX_PADDED}"),
            FftError::LengthMismatch { expected, found } => write!(f, "buffer holds {found} values, expected {expected}"),
        }
    }
}

impl std::error::Error for FftError {}

/// A complex number in f64.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Complex {
        Complex { re, im }
    }

    pub fn conj(self) -> Complex {
        Complex { re: self.re, im: -self.im }
    }

    /// |z|².
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

impl Add for Complex {
    type Output = Complex;
    fn add(self, o: Complex) -> Complex {
        Complex { re: self.re + o.re, im: self.im + o.im }
    }
}

impl Sub for Complex {
    type Output = Complex;
    fn sub(self, o: Complex) -> Complex {
        Complex { re: self.re - o.re, im: self.im - o.im }
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, o: Complex) -> Complex {
        Complex { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
}

/// The padded size of a box axis of `len` cells: the smallest power of two ≥ 2 × `len`, at least [`MIN_PADDED`]
/// (§1.3.3). Refuses an empty axis and one that would pad past [`MAX_PADDED`].
pub fn padded_size(len: usize) -> Result<usize, FftError> {
    if len == 0 {
        return Err(FftError::BoxOutOfRange(len));
    }
    let n = (2 * len).next_power_of_two().max(MIN_PADDED);
    if n > MAX_PADDED {
        return Err(FftError::BoxOutOfRange(len));
    }
    Ok(n)
}

/// One length's radix-2 plan: the twiddle factors e^{−2πi k/N} (each from its own sin/cos, not a recurrence) and the
/// bit-reversal permutation. Built once per length.
#[derive(Debug, Clone)]
pub struct FftPlan {
    n: usize,
    twiddles: Vec<Complex>,
    reversed: Vec<usize>,
}

impl FftPlan {
    /// A plan for any power-of-two length ≥ 1 (the convolution only asks for 32…2048).
    pub fn new(n: usize) -> Result<FftPlan, FftError> {
        if n == 0 || !n.is_power_of_two() {
            return Err(FftError::NotPowerOfTwo(n));
        }
        let twiddles = (0..n / 2)
            .map(|k| {
                let (s, c) = (-2.0 * PI * k as f64 / n as f64).sin_cos();
                Complex::new(c, s)
            })
            .collect();
        let bits = n.trailing_zeros();
        let reversed = (0..n).map(|i| if bits == 0 { 0 } else { i.reverse_bits() >> (usize::BITS - bits) }).collect();
        Ok(FftPlan { n, twiddles, reversed })
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// In place: the forward transform, or with `inverse` the conjugate transform divided by N.
    pub fn transform(&self, data: &mut [Complex], inverse: bool) -> Result<(), FftError> {
        let n = self.n;
        if data.len() != n {
            return Err(FftError::LengthMismatch { expected: n, found: data.len() });
        }
        for i in 0..n {
            let j = self.reversed[i];
            if i < j {
                data.swap(i, j);
            }
        }
        let mut half = 1;
        while half < n {
            let step = n / (2 * half);
            for start in (0..n).step_by(2 * half) {
                for k in 0..half {
                    let t = self.twiddles[k * step];
                    let w = if inverse { t.conj() } else { t };
                    let a = data[start + k];
                    let b = data[start + k + half] * w;
                    data[start + k] = a + b;
                    data[start + k + half] = a - b;
                }
            }
            half *= 2;
        }
        if inverse {
            let scale = 1.0 / n as f64;
            for z in data.iter_mut() {
                z.re *= scale;
                z.im *= scale;
            }
        }
        Ok(())
    }
}

/// A 2D transform in place: the rows with `plan_x`, then the columns with `plan_y`. `data` is `plan_x.len()` wide and
/// `plan_y.len()` tall.
pub fn fft2(data: &mut [Complex], plan_x: &FftPlan, plan_y: &FftPlan, inverse: bool) -> Result<(), FftError> {
    let (nx, ny) = (plan_x.len(), plan_y.len());
    if data.len() != nx * ny {
        return Err(FftError::LengthMismatch { expected: nx * ny, found: data.len() });
    }
    for row in data.chunks_exact_mut(nx) {
        plan_x.transform(row, inverse)?;
    }
    let mut column = vec![Complex::ZERO; ny];
    for x in 0..nx {
        for y in 0..ny {
            column[y] = data[y * nx + x];
        }
        plan_y.transform(&mut column, inverse)?;
        for y in 0..ny {
            data[y * nx + x] = column[y];
        }
    }
    Ok(())
}

/// A kernel's signed offset for padded index `i` of an axis of `n`: 0…n/2−1 as is, n/2…n−1 as i − n. The entry at n/2
/// is never reached by a box (offsets stay inside (−n/2, n/2)).
fn signed_offset(i: usize, n: usize) -> isize {
    if i < n / 2 {
        i as isize
    } else {
        i as isize - n as isize
    }
}

/// The zero-padded convolution φ(x) = Σ_y K(x − y) s(y) of a field with one kernel, over any box up to 1024 per axis.
/// The kernel's transform is cached per padded size (§1.3.3), as are the plans.
pub struct Convolver {
    kernel: Box<dyn Fn(isize, isize) -> f64>,
    plans: HashMap<usize, FftPlan>,
    spectra: HashMap<(usize, usize), Vec<Complex>>,
}

impl Convolver {
    /// A convolver for `kernel(dx, dy)`, the weight of a source `dx` cells to the left and `dy` below the target.
    pub fn new(kernel: impl Fn(isize, isize) -> f64 + 'static) -> Convolver {
        Convolver { kernel: Box::new(kernel), plans: HashMap::new(), spectra: HashMap::new() }
    }

    /// The padded sizes whose kernel transform is cached, sorted.
    pub fn cached_sizes(&self) -> Vec<(usize, usize)> {
        let mut sizes: Vec<_> = self.spectra.keys().copied().collect();
        sizes.sort_unstable();
        sizes
    }

    /// The convolution of the `w` × `h` row-major `field` with the kernel, on the same box.
    pub fn convolve(&mut self, field: &[f64], w: usize, h: usize) -> Result<Vec<f64>, FftError> {
        if field.len() != w * h {
            return Err(FftError::LengthMismatch { expected: w * h, found: field.len() });
        }
        let (px, py) = (padded_size(w)?, padded_size(h)?);
        for n in [px, py] {
            if let Entry::Vacant(slot) = self.plans.entry(n) {
                slot.insert(FftPlan::new(n)?);
            }
        }
        let (plan_x, plan_y) = (&self.plans[&px], &self.plans[&py]);
        if !self.spectra.contains_key(&(px, py)) {
            let mut k = vec![Complex::ZERO; px * py];
            for j in 0..py {
                for i in 0..px {
                    k[j * px + i].re = (self.kernel)(signed_offset(i, px), signed_offset(j, py));
                }
            }
            fft2(&mut k, plan_x, plan_y, false)?;
            self.spectra.insert((px, py), k);
        }
        let spectrum = &self.spectra[&(px, py)];

        let mut padded = vec![Complex::ZERO; px * py];
        for y in 0..h {
            for x in 0..w {
                padded[y * px + x].re = field[y * w + x];
            }
        }
        fft2(&mut padded, plan_x, plan_y, false)?;
        for (z, k) in padded.iter_mut().zip(spectrum) {
            *z = *z * *k;
        }
        fft2(&mut padded, plan_x, plan_y, true)?;

        let mut out = vec![0.0; w * h];
        for y in 0..h {
            for x in 0..w {
                out[y * w + x] = padded[y * px + x].re;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    /// A deterministic field in [−1, 1): a 64-bit LCG, so a failure reproduces.
    fn pseudo_random(count: usize, seed: u64) -> Vec<f64> {
        let mut s = seed;
        (0..count)
            .map(|_| {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((s >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
            })
            .collect()
    }

    fn pseudo_complex(count: usize, seed: u64) -> Vec<Complex> {
        let v = pseudo_random(2 * count, seed);
        (0..count).map(|i| Complex::new(v[2 * i], v[2 * i + 1])).collect()
    }

    /// The direct DFT, O(N²): X_k = Σ x_n e^{∓2πi kn/N}, the inverse divided by N.
    fn direct_dft(x: &[Complex], inverse: bool) -> Vec<Complex> {
        let n = x.len();
        let sign = if inverse { 1.0 } else { -1.0 };
        (0..n)
            .map(|k| {
                let mut sum = Complex::ZERO;
                for (j, v) in x.iter().enumerate() {
                    // (k·j mod N) keeps the angle small, so the reference itself stays accurate
                    let (s, c) = (sign * 2.0 * PI * ((k * j) % n) as f64 / n as f64).sin_cos();
                    sum = sum + *v * Complex::new(c, s);
                }
                if inverse {
                    sum.re /= n as f64;
                    sum.im /= n as f64;
                }
                sum
            })
            .collect()
    }

    /// The largest |a − b| over the largest |b|.
    fn rel_err(a: &[Complex], b: &[Complex]) -> f64 {
        let scale = b.iter().map(|z| z.norm_sqr().sqrt()).fold(0.0, f64::max);
        let worst = a.iter().zip(b).map(|(p, q)| (*p - *q).norm_sqr().sqrt()).fold(0.0, f64::max);
        worst / scale
    }

    fn rel_err_real(a: &[f64], b: &[f64]) -> f64 {
        let scale = b.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        a.iter().zip(b).fold(0.0f64, |m, (p, q)| m.max((p - q).abs())) / scale
    }

    #[test]
    fn fft_matches_the_direct_dft() {
        for n in [1usize, 2, 4, 8, 16, 32, 64, 128] {
            let x = pseudo_complex(n, 7 + n as u64);
            let plan = FftPlan::new(n).unwrap();
            for inverse in [false, true] {
                let mut y = x.clone();
                plan.transform(&mut y, inverse).unwrap();
                let e = rel_err(&y, &direct_dft(&x, inverse));
                assert!(e < 1e-12, "n = {n}, inverse = {inverse}: relative error {e:e}");
            }
        }
    }

    #[test]
    fn fft_of_a_delta_and_a_wave_are_exact() {
        let plan = FftPlan::new(16).unwrap();
        let mut delta = vec![Complex::ZERO; 16];
        delta[0].re = 1.0;
        plan.transform(&mut delta, false).unwrap();
        assert!(delta.iter().all(|z| (z.re - 1.0).abs() < 1e-15 && z.im.abs() < 1e-15));
        // x_n = e^{+2πi 3n/16} puts all of its weight, N, in bin 3: the forward kernel is e^{−…}
        let mut wave: Vec<Complex> = (0..16)
            .map(|n| {
                let (s, c) = (2.0 * PI * 3.0 * n as f64 / 16.0).sin_cos();
                Complex::new(c, s)
            })
            .collect();
        plan.transform(&mut wave, false).unwrap();
        for (k, z) in wave.iter().enumerate() {
            let want = if k == 3 { 16.0 } else { 0.0 };
            assert!((z.re - want).abs() < 1e-12 && z.im.abs() < 1e-12, "bin {k}: {z:?}");
        }
    }

    #[test]
    fn inverse_undoes_forward_in_1d_and_2d() {
        for n in [32usize, 256, 2048] {
            let x = pseudo_complex(n, 3);
            let plan = FftPlan::new(n).unwrap();
            let mut y = x.clone();
            plan.transform(&mut y, false).unwrap();
            plan.transform(&mut y, true).unwrap();
            assert!(rel_err(&y, &x) < 1e-13, "1D n = {n}");
        }
        let (px, py) = (FftPlan::new(64).unwrap(), FftPlan::new(32).unwrap());
        let x = pseudo_complex(64 * 32, 5);
        let mut y = x.clone();
        fft2(&mut y, &px, &py, false).unwrap();
        fft2(&mut y, &px, &py, true).unwrap();
        assert!(rel_err(&y, &x) < 1e-13, "2D 64 × 32");
    }

    #[test]
    fn fft2_matches_the_direct_2d_dft_and_obeys_parseval() {
        let (nx, ny) = (16usize, 8usize);
        let x = pseudo_complex(nx * ny, 11);
        let (plan_x, plan_y) = (FftPlan::new(nx).unwrap(), FftPlan::new(ny).unwrap());
        let mut y = x.clone();
        fft2(&mut y, &plan_x, &plan_y, false).unwrap();
        let mut direct = vec![Complex::ZERO; nx * ny];
        for ky in 0..ny {
            for kx in 0..nx {
                let mut sum = Complex::ZERO;
                for jy in 0..ny {
                    for jx in 0..nx {
                        let phase = ((kx * jx) % nx) as f64 / nx as f64 + ((ky * jy) % ny) as f64 / ny as f64;
                        let (s, c) = (-2.0 * PI * phase).sin_cos();
                        sum = sum + x[jy * nx + jx] * Complex::new(c, s);
                    }
                }
                direct[ky * nx + kx] = sum;
            }
        }
        let e = rel_err(&y, &direct);
        assert!(e < 1e-12, "2D relative error {e:e}");
        // Parseval: Σ|x|² = (1/N) Σ|X|²
        let space: f64 = x.iter().map(|z| z.norm_sqr()).sum();
        let freq: f64 = y.iter().map(|z| z.norm_sqr()).sum::<f64>() / (nx * ny) as f64;
        assert!(((space - freq) / space).abs() < 1e-13, "Parseval: {space} against {freq}");
    }

    #[test]
    fn parseval_holds_at_the_largest_size() {
        let x = pseudo_complex(2048, 17);
        let mut y = x.clone();
        FftPlan::new(2048).unwrap().transform(&mut y, false).unwrap();
        let space: f64 = x.iter().map(|z| z.norm_sqr()).sum();
        let freq: f64 = y.iter().map(|z| z.norm_sqr()).sum::<f64>() / 2048.0;
        assert!(((space - freq) / space).abs() < 1e-13, "Parseval: {space} against {freq}");
    }

    /// A kernel with no symmetry in either axis, long-ranged so that wrap-around would show: K(dx, dy) ≠ K(−dx, dy)
    /// and ≠ K(dx, −dy), always positive (the tanh terms stay within ±0.8).
    fn asymmetric_kernel(dx: isize, dy: isize) -> f64 {
        let (x, y) = (dx as f64, dy as f64);
        (1.0 + 0.5 * (0.2 * x).tanh() + 0.3 * (0.1 * y).tanh()) / (1.0 + x * x + 2.0 * y * y).sqrt()
    }

    /// Σ_y K(x − y) s(y) by direct summation over the whole box.
    fn direct_convolution(field: &[f64], w: usize, h: usize, kernel: impl Fn(isize, isize) -> f64) -> Vec<f64> {
        let mut out = vec![0.0; w * h];
        for ty in 0..h {
            for tx in 0..w {
                let mut sum = 0.0;
                for sy in 0..h {
                    for sx in 0..w {
                        sum += kernel(tx as isize - sx as isize, ty as isize - sy as isize) * field[sy * w + sx];
                    }
                }
                out[ty * w + tx] = sum;
            }
        }
        out
    }

    #[test]
    fn convolution_matches_direct_summation_on_48_by_40_with_an_asymmetric_kernel() {
        let (w, h) = (48usize, 40usize);
        assert_ne!(asymmetric_kernel(5, 3), asymmetric_kernel(-5, 3));
        assert_ne!(asymmetric_kernel(5, 3), asymmetric_kernel(5, -3));
        // positive masses everywhere, so the edge rows and columns carry weight into the comparison
        let field: Vec<f64> = pseudo_random(w * h, 29).iter().map(|v| v + 1.0).collect();
        let fast = Convolver::new(asymmetric_kernel).convolve(&field, w, h).unwrap();
        let direct = direct_convolution(&field, w, h, asymmetric_kernel);
        let e = rel_err_real(&fast, &direct);
        assert!(e < 1e-10, "relative error {e:e}");
        // the edge cells themselves, rim by rim, not only the worst case over the box
        for x in 0..w {
            for y in [0, h - 1] {
                let (a, b) = (fast[y * w + x], direct[y * w + x]);
                assert!(((a - b) / b).abs() < 1e-10, "edge cell ({x}, {y}): {a} against {b}");
            }
        }
        for y in 0..h {
            for x in [0, w - 1] {
                let (a, b) = (fast[y * w + x], direct[y * w + x]);
                assert!(((a - b) / b).abs() < 1e-10, "edge cell ({x}, {y}): {a} against {b}");
            }
        }
    }

    #[test]
    fn mass_in_the_corners_does_not_wrap_around() {
        // one source at each corner of a 40 × 33 box: with the padding one cell short, the far corner's pull would
        // wrap onto the near one; the direct sum has none.
        let (w, h) = (40usize, 33usize);
        let mut field = vec![0.0; w * h];
        for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            field[y * w + x] = 1.0;
        }
        let fast = Convolver::new(asymmetric_kernel).convolve(&field, w, h).unwrap();
        let direct = direct_convolution(&field, w, h, asymmetric_kernel);
        let e = rel_err_real(&fast, &direct);
        assert!(e < 1e-10, "relative error {e:e}");
    }

    #[test]
    fn a_single_source_reproduces_the_kernel_shifted() {
        let (w, h) = (20usize, 17usize);
        let (sx, sy) = (6usize, 11usize);
        let mut field = vec![0.0; w * h];
        field[sy * w + sx] = 2.5;
        let out = Convolver::new(asymmetric_kernel).convolve(&field, w, h).unwrap();
        for y in 0..h {
            for x in 0..w {
                let want = 2.5 * asymmetric_kernel(x as isize - sx as isize, y as isize - sy as isize);
                assert!((out[y * w + x] - want).abs() < 1e-12 * want.abs().max(1.0), "cell ({x}, {y})");
            }
        }
    }

    #[test]
    fn the_kernel_transform_is_cached_per_padded_size() {
        let evals = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&evals);
        let mut conv = Convolver::new(move |dx, dy| {
            counter.set(counter.get() + 1);
            asymmetric_kernel(dx, dy)
        });
        // 48 × 40 pads to 128 × 128 (96 → 128, 80 → 128); 50 × 41 pads to the same size
        conv.convolve(&vec![1.0; 48 * 40], 48, 40).unwrap();
        assert_eq!(conv.cached_sizes(), vec![(128, 128)]);
        assert_eq!(evals.get(), 128 * 128);
        conv.convolve(&vec![1.0; 50 * 41], 50, 41).unwrap();
        assert_eq!(evals.get(), 128 * 128, "the same padded size must not resample the kernel");
        // 20 × 10 pads to 64 × 32: a second entry
        conv.convolve(&vec![1.0; 20 * 10], 20, 10).unwrap();
        assert_eq!(conv.cached_sizes(), vec![(64, 32), (128, 128)]);
        assert_eq!(evals.get(), 128 * 128 + 64 * 32);
    }

    #[test]
    fn padded_sizes_follow_the_box_rule_and_refuse_out_of_range_boxes() {
        for (len, want) in [(1, 32), (16, 32), (17, 64), (32, 64), (48, 128), (64, 128), (300, 1024), (1024, 2048)] {
            assert_eq!(padded_size(len), Ok(want), "box axis {len}");
        }
        assert_eq!(padded_size(0), Err(FftError::BoxOutOfRange(0)));
        assert_eq!(padded_size(1025), Err(FftError::BoxOutOfRange(1025)));
        assert_eq!(FftPlan::new(0).unwrap_err(), FftError::NotPowerOfTwo(0));
        assert_eq!(FftPlan::new(48).unwrap_err(), FftError::NotPowerOfTwo(48));
        let plan = FftPlan::new(32).unwrap();
        assert_eq!(
            plan.transform(&mut [Complex::ZERO; 16], false),
            Err(FftError::LengthMismatch { expected: 32, found: 16 })
        );
        let mut conv = Convolver::new(asymmetric_kernel);
        assert_eq!(conv.convolve(&[0.0; 5], 2, 3), Err(FftError::LengthMismatch { expected: 6, found: 5 }));
    }
}
