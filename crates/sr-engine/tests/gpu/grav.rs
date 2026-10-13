//! M0-T29 — gravity on the CPU twin (contract §1.4.1–§1.4.3, §1.7, §5.1's G-GRAV1 and G-GRAV2): K(i, j) = −G/√(i² + j²)
//! with the exact self cell −G·4 ln(1 + √2), φ = K ∗ Σ_src by the zero-padded FFT convolution, g = −∇φ by central
//! differences. The twin is sr_physics::reference::grav.
//!
//! M0-T31 — P2 on the GPU (step/grav.rs, shaders/grav.wgsl) joins each scope with its own cases (`gpu_…`): the same
//! oracles on one step of a state with gravity on, over the box the step fits, and φ and g within G-REF's tolerance of
//! the twin over that box (relative L1 ≤ 1e-3, §5.3; measured values printed as `SR-GRAV-REF`); P9's gravity term from
//! the GPU's g; a frame's P2 the bare step's, bit for bit. Every GPU number names its adapter (R13).
//!
//! M0-D25 — what M0-V4's mutants of §1.4.2 found ungraded: K asserted at every offset of a 64 × 64 box, not seven;
//! P2's vacuum mask on a box with sub-Σ_vac cells in it; P2 on a wide and a tall box (FFT 1024 × 128, 128 × 1024) and
//! on the whole 600 × 400 world (2048 × 1024), against the twin and against a direct sum of the kernel over the sources
//! (no FFT, no table), and G-GRAV2's ratio on the two boxes whose outermost ring is empty.

use sr_physics::reference::grav::{self, Gravity, SELF_CELL};

/// G = 1 in the sandbox's units (§2.1).
const G: f64 = 1.0;

/// §1.4.2's self-cell constant, −3.525494 G, from the closed form 4 ln(1 + √2).
const SELF_CELL_6: f64 = 3.525494;

/// ∫∫ over the unit square [−½, ½]² of dA / r, by Simpson's rule on the polar form — independent of the closed form:
/// 8 ∫₀^{π/4} ∫₀^{½/cos θ} dr dθ = 8 ∫₀^{π/4} (½ / cos θ) dθ.
fn square_potential_by_quadrature() -> f64 {
    let n = 4096; // even
    let (a, b) = (0.0, std::f64::consts::FRAC_PI_4);
    let f = |t: f64| 0.5 / t.cos();
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for k in 1..n {
        sum += f(a + k as f64 * h) * if k % 2 == 1 { 4.0 } else { 2.0 };
    }
    8.0 * sum * h / 3.0
}

mod force {
    use super::*;
    use sr_physics::registry::Physics;
    use std::f64::consts::PI;

    /// The Maclaurin disk Σ(r) = Σc √(1 − r²/a²) for r < a, centred at `(cx, cy)` in cell-centre coordinates (cell
    /// (i, j) sits at (i, j)); Σ sampled at the cell centres, cells as point masses (SM-C1).
    fn disk(a: f64, sigma_c: f64, (cx, cy): (f64, f64), w: usize, h: usize) -> Vec<f64> {
        let mut sigma = vec![0.0; w * h];
        for j in 0..h {
            for i in 0..w {
                let r2 = ((i as f64 - cx).powi(2) + (j as f64 - cy).powi(2)) / (a * a);
                if r2 < 1.0 {
                    sigma[j * w + i] = sigma_c * (1.0 - r2).sqrt();
                }
            }
        }
        sigma
    }

    /// The largest |Δg|/|g| over every cell with 0 < r ≤ 0.75a, where g(r) = −(π²GΣc/2a)·r (E6) — the cell(s) at
    /// r = 0 have g = 0 exactly and are checked absolutely. Returns (worst error, its cell, cells checked, the
    /// largest |g| found at r = 0 over k·a).
    fn worst_error(a: f64, centre: (f64, f64)) -> (f64, (usize, usize), usize, f64) {
        let side = (2.0 * a).ceil() as usize + 17;
        let (cx, cy) = centre;
        let sigma = disk(a, 1.0, (cx + (side as f64 - 1.0) / 2.0, cy + (side as f64 - 1.0) / 2.0), side, side);
        let origin = ((side as f64 - 1.0) / 2.0, (side as f64 - 1.0) / 2.0);
        let field = Gravity::new(G).field(&sigma, 0.0, side, side).unwrap();
        disk_error(a, 1.0, (origin.0 + cx, origin.1 + cy), &field.gx, &field.gy, side)
    }

    /// [`worst_error`]'s measure of a field `gx`, `gy` (`w` cells wide) against the disk of radius `a` and central Σc
    /// `sigma_c` centred at `(ox, oy)` in its cells.
    fn disk_error(
        a: f64,
        sigma_c: f64,
        (ox, oy): (f64, f64),
        gx: &[f64],
        gy: &[f64],
        w: usize,
    ) -> (f64, (usize, usize), usize, f64) {
        let k = PI * PI * G * sigma_c / (2.0 * a);
        let (mut worst, mut at, mut count, mut centre_g) = (0.0f64, (0, 0), 0, 0.0f64);
        for j in 0..gx.len() / w {
            for i in 0..w {
                let (dx, dy) = (i as f64 - ox, j as f64 - oy);
                let r = dx.hypot(dy);
                if r > 0.75 * a {
                    continue;
                }
                let (gx, gy) = (gx[j * w + i], gy[j * w + i]);
                if r == 0.0 {
                    centre_g = centre_g.max(gx.hypot(gy) / (k * a));
                    continue;
                }
                let exact = k * r;
                let err = ((gx + k * dx).powi(2) + (gy + k * dy).powi(2)).sqrt() / exact;
                count += 1;
                if err > worst {
                    (worst, at) = (err, (i, j));
                }
            }
        }
        (worst, at, count, centre_g)
    }

    #[test]
    fn kernel_is_the_section_1_4_2_kernel() {
        // The self cell: the closed form, six places as the contract prints it, and an independent quadrature.
        assert!((SELF_CELL - 4.0 * (1.0 + 2f64.sqrt()).ln()).abs() < 1e-15, "{SELF_CELL}");
        assert!((SELF_CELL - SELF_CELL_6).abs() < 5e-7, "{SELF_CELL}");
        let quad = square_potential_by_quadrature();
        assert!((SELF_CELL - quad).abs() < 1e-12, "closed form {SELF_CELL} vs quadrature {quad}");
        assert_eq!(grav::kernel(G, 0, 0), -SELF_CELL, "K(0, 0) is the exact constant, not a softened 1/ε");
        assert_eq!(grav::kernel(2.5, 0, 0), -2.5 * SELF_CELL);
        // Everywhere else the plain 3D law in the plane, symmetric in both axes and in their swap, linear in G.
        for (i, j, r) in [(1, 0, 1.0), (0, 1, 1.0), (3, 4, 5.0), (-5, 12, 13.0), (8, -15, 17.0), (-7, -24, 25.0)] {
            assert!((grav::kernel(G, i, j) + 1.0 / r).abs() < 1e-15, "K({i}, {j})");
            assert_eq!(grav::kernel(G, i, j), grav::kernel(G, -i, j));
            assert_eq!(grav::kernel(G, i, j), grav::kernel(G, i, -j));
            assert_eq!(grav::kernel(G, i, j), grav::kernel(G, j, i));
            assert!((grav::kernel(3.0, i, j) - 3.0 * grav::kernel(G, i, j)).abs() < 1e-15);
        }
        // Through the convolution: one unit of Σ makes −G·4 ln(1 + √2) at its own cell and −G/r around it.
        let (w, h) = (33, 33);
        let mut sigma = vec![0.0; w * h];
        sigma[16 * w + 16] = 1.0;
        let phi = Gravity::new(G).potential(&sigma, w, h).unwrap();
        assert!((phi[16 * w + 16] + SELF_CELL).abs() < 1e-12, "own cell {}", phi[16 * w + 16]);
        assert!((phi[16 * w + 19] + 1.0 / 3.0).abs() < 1e-12 && (phi[20 * w + 19] + 1.0 / 5.0).abs() < 1e-12);
        // Vacuum is no source: a cell under Σ_vac adds nothing, one at Σ_vac does.
        assert_eq!(grav::source(&[0.0, 9e-7, 1e-6, 2.0], 1e-6), vec![0.0, 0.0, 1e-6, 2.0]);
    }

    #[test]
    fn kernel_is_minus_g_over_r_at_every_offset_of_a_64_box() {
        // §1.4.2 at all of a 64 × 64 box's offsets (|i|, |j| ≤ 63): the table the GPU samples is this function, so a
        // wrong value at a few offsets (the diagonal neighbours, say) is the same mistake twice and the GPU cases see none.
        let mut checked = 0;
        for j in -63isize..=63 {
            for i in -63isize..=63 {
                if (i, j) == (0, 0) {
                    assert_eq!(grav::kernel(G, 0, 0), -SELF_CELL);
                    continue;
                }
                let want = -G / (i as f64).hypot(j as f64);
                let got = grav::kernel(G, i, j);
                assert!((got - want).abs() <= 4e-16 * want.abs(), "K({i}, {j}) = {got}, −G/√(i² + j²) = {want}");
                checked += 1;
            }
        }
        assert_eq!(checked, 127 * 127 - 1);
    }

    #[test]
    fn maclaurin_disk_a40_force_is_within_2_percent_everywhere() {
        let (worst, at, count, centre_g) = worst_error(40.0, (0.0, 0.0));
        println!("SR-GRAV1 a=40 centred on a cell: largest |Δg|/|g| = {:.3e} ({:.2} %) at cell {at:?}, {count} cells with 0 < r ≤ 0.75a", worst, worst * 100.0);
        assert!(count > 2700, "{count} cells checked");
        assert!(worst <= 0.02, "G-GRAV1: {:.3} % at cell {at:?} over 2 %", worst * 100.0);
        assert!(centre_g < 1e-12, "the centre cell has g = 0 by symmetry, found {centre_g:e}·k·a");
    }

    #[test]
    fn force_error_halves_when_the_disk_doubles() {
        let (e40, at40, _, _) = worst_error(40.0, (0.0, 0.0));
        let (e20, at20, _, _) = worst_error(20.0, (0.0, 0.0));
        let ratio = e20 / e40;
        println!("SR-GRAV1 largest error a=20: {:.3e} ({:.2} %) at {at20:?}; a=40: {:.3e} ({:.2} %) at {at40:?}; ratio {ratio:.3}", e20, e20 * 100.0, e40, e40 * 100.0);
        assert!((1.6..=2.4).contains(&ratio), "G-GRAV1: a=20 over a=40 = {ratio:.3}, first order wants 1.6 … 2.4");
        assert!(e20 > e40, "a smaller disk is the worse resolved");
    }

    #[test]
    fn maclaurin_disk_centred_on_a_cell_corner_holds_too() {
        // The scene files' convention (§2.12.4: an object's x, y are points, cells sampled at their centres) puts a
        // disk's centre on a corner, not a cell centre.
        let (worst, at, count, _) = worst_error(40.0, (0.5, 0.5));
        println!("SR-GRAV1 a=40 centred on a corner: largest |Δg|/|g| = {:.3e} ({:.2} %) at cell {at:?}, {count} cells", worst, worst * 100.0);
        assert!(worst <= 0.02, "G-GRAV1 (corner-centred): {:.3} % at cell {at:?}", worst * 100.0);
    }

    /// One step of a cold Maclaurin disk (a, Σc = 1) centred on cell (c, c) of a `side` × `side` world, gravity on: the
    /// GPU's g over the box the step fit against E6, and φ and g against the twin.
    fn gpu_disk(gpu: &sr_engine::gpu::Gpu, a: f64, side: u32) -> (f64, (usize, usize), usize, f64) {
        let c = f64::from(side / 2);
        let sigma = disk(a, 1.0, (c, c), side as usize, side as usize);
        let solved = super::gpu::solve(gpu, &sigma, side, side, 1, true);
        let field = solved.cropped();
        let (x0, y0) = (f64::from(solved.area.x0), f64::from(solved.area.y0));
        let measured = disk_error(a, 1.0, (c - x0, c - y0), &field.gx, &field.gy, solved.area.w as usize);
        solved.against_twin(&format!("disk a={a}"));
        measured
    }

    #[test]
    fn gpu_maclaurin_disk_a40_force_is_within_2_percent_and_on_the_twin() {
        let gpu = crate::device();
        let (worst, at, count, centre_g) = gpu_disk(&gpu, 40.0, 128);
        println!("SR-GRAV1 GPU a=40 centred on a cell: largest |Δg|/|g| = {:.3e} ({:.2} %) at box cell {at:?}, {count} cells with 0 < r ≤ 0.75a", worst, worst * 100.0);
        assert!(count > 2700, "{count} cells checked");
        assert!(worst <= 0.02, "G-GRAV1 (GPU): {:.3} % at box cell {at:?} over 2 %", worst * 100.0);
        assert!(centre_g < 1e-5, "the centre cell has g = 0 by symmetry, found {centre_g:e}·k·a");
    }

    #[test]
    fn gpu_force_error_halves_when_the_disk_doubles() {
        let gpu = crate::device();
        let (e40, _, _, _) = gpu_disk(&gpu, 40.0, 128);
        let (e20, _, _, _) = gpu_disk(&gpu, 20.0, 64);
        let ratio = e20 / e40;
        println!("SR-GRAV1 GPU largest error a=20: {e20:.3e}; a=40: {e40:.3e}; ratio {ratio:.3}");
        assert!((1.6..=2.4).contains(&ratio), "G-GRAV1 (GPU): a=20 over a=40 = {ratio:.3}, first order wants 1.6 … 2.4");
    }

    #[test]
    fn gpu_sub_vacuum_cells_are_no_source() {
        // §1.4.2: Σ_src = 0 below Σ_vac. A faint disk (Σc = 1e-3, a = 20) in a world of 0.9 Σ_vac: the box is the disk's,
        // and the ~2800 sub-vacuum cells inside it, masked, add nothing — unmasked they shift φ by 2.5e-3 relative L1.
        let gpu = crate::device();
        let (w, h, sigma) = super::gpu::faint_disk_in_sub_vacuum();
        let solved = super::gpu::solve(&gpu, &sigma, w, h, 1, true);
        let sigma_vac = Physics::shipped().unwrap().get("sigma_vac").unwrap();
        let sub = solved.cropped().sigma.iter().filter(|&&s| s < sigma_vac).count();
        assert!(sub > 1000, "{sub} sub-vacuum cells in the box: the case must have some to say anything");
        solved.against_twin("sub-vacuum disk");
        solved.against_direct_sum("sub-vacuum disk", 1e-4);
    }

    #[test]
    fn gpu_wide_box_has_the_twins_and_the_kernels_potential() {
        let gpu = crate::device();
        let (w, h, sigma) = super::gpu::wide_scene();
        let solved = super::gpu::solve(&gpu, &sigma, w, h, 1, true);
        assert_eq!((solved.area.w, solved.area.h, solved.area.fft), (392, 64, [1024, 128]), "{:?}", solved.area);
        solved.against_twin("wide box");
        solved.against_direct_sum("wide box", 1e-4);
    }

    #[test]
    fn gpu_tall_box_has_the_twins_and_the_kernels_potential() {
        let gpu = crate::device();
        let (w, h, sigma) = super::gpu::tall_scene();
        let solved = super::gpu::solve(&gpu, &sigma, w, h, 1, true);
        assert_eq!((solved.area.w, solved.area.h, solved.area.fft), (40, 344, [128, 1024]), "{:?}", solved.area);
        solved.against_twin("tall box");
        solved.against_direct_sum("tall box", 1e-4);
    }

    #[test]
    fn gpu_whole_default_world_has_the_twins_and_the_kernels_potential() {
        // 600 × 400, matter to every edge: the box is the world and the FFT the largest one a world takes, 2048 × 1024.
        let gpu = crate::device();
        let (w, h, sigma) = super::gpu::whole_world_scene();
        let solved = super::gpu::solve(&gpu, &sigma, w, h, 1, true);
        assert_eq!((solved.area.w, solved.area.h, solved.area.fft), (600, 400, [2048, 1024]), "{:?}", solved.area);
        solved.against_twin("whole 600x400 world");
        solved.against_direct_sum("whole 600x400 world", 1e-4);
    }

    #[test]
    fn gpu_gravity_bounds_dt_with_the_section_2_3_5_term() {
        // §2.3.5: Δt = min(C · …, η_g · min_cells √(Δx/|g|)). The same cold disk with gravity off gives the hydro term
        // alone; with it on, Δt_0 (the initial state's reduction, after P2 on it) and Δt_1 (step 0's P9) are that term
        // or η_g / √(max |g|) over the box, whichever is smaller — here the gravity term, or the case says nothing: the
        // cold pairs off (as the dt scope's cases), so c is T_floor's.
        let gpu = crate::device();
        let sigma = disk(40.0, 1.0, (64.0, 64.0), 128, 128);
        let physics = Physics::shipped().unwrap().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]).unwrap();
        let on = super::gpu::solve_with(&gpu, &physics, &sigma, 128, 128, true);
        let off = super::gpu::solve_with(&gpu, &physics, &sigma, 128, 128, false);
        let field = on.cropped();
        let max_g = field.gx.iter().zip(&field.gy).map(|(x, y)| x.hypot(*y)).fold(0.0f64, f64::max);
        let eta_g = physics.get("eta_g").unwrap();
        let dt_g = eta_g / max_g.sqrt();
        println!("SR-GRAV-DT max |g| {max_g:.6e}: η_g term {dt_g:.6e}; gravity off Δt_0 {:e}, Δt_1 {:e}; on Δt_0 {:e}, Δt_1 {:e}", off.dt.dt, off.dt.dt_next, on.dt.dt, on.dt.dt_next);
        assert!(dt_g < 0.5 * f64::from(off.dt.dt.min(off.dt.dt_next)), "the gravity term must bind for the case to say anything");
        for (what, got, hydro) in [("Δt_0", on.dt.dt, off.dt.dt), ("Δt_1", on.dt.dt_next, off.dt.dt_next)] {
            let want = dt_g.min(f64::from(hydro));
            let rel = (f64::from(got) - want).abs() / want;
            assert!(rel <= 1e-6, "{what}: {got} against min(η_g/√max|g|, hydro) = {want} ({rel:e} off)");
        }
    }

    #[test]
    fn gpu_a_frames_gravity_is_the_bare_steps_bit_for_bit() {
        // Under a frame every dispatch reads its size from the latch's slots (latch.rs): P2's fixed sizes and its box
        // tiles must come out as the bare step's.
        let gpu = crate::device();
        let sigma = disk(30.0, 1.0, (50.0, 70.0), 128, 128);
        let bare = super::gpu::solve(&gpu, &sigma, 128, 128, 3, true);
        let framed = super::gpu::solve_framed(&gpu, &sigma, 128, 128, 3);
        assert!(bare.g.iter().any(|&v| v != 0.0), "gravity ran");
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!((framed.dt.dt.to_bits(), framed.dt.dt_next.to_bits()), (bare.dt.dt.to_bits(), bare.dt.dt_next.to_bits()));
        assert!(bits(&framed.phi) == bits(&bare.phi) && bits(&framed.g) == bits(&bare.g), "φ and g as the bare steps'");
    }
}

mod selfforce {
    use super::*;
    use sr_engine::scene::Scene;
    use sr_physics::registry::{Elements, Physics};
    use std::path::Path;

    /// (Σ m·g, Σ m|g|) over a field: the net force vector and the sum of the force magnitudes.
    fn net_and_scale(sigma: &[f64], gx: &[f64], gy: &[f64]) -> ([f64; 2], f64) {
        let (mut net, mut scale) = ([0.0, 0.0], 0.0);
        for ((&m, &x), &y) in sigma.iter().zip(gx).zip(gy) {
            net[0] += m * x;
            net[1] += m * y;
            scale += m * x.hypot(y);
        }
        (net, scale)
    }

    #[test]
    fn asym_blob_has_no_net_self_force() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes/asym_blob.json");
        let scene = Scene::load(&root).unwrap();
        let physics = Physics::shipped().unwrap();
        let (planes, warnings) = scene.planes(scene.world, &physics, &Elements::shipped().unwrap()).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        let (w, h) = (scene.world.width as usize, scene.world.height as usize);
        let sigma: Vec<f64> = planes[..w * h].iter().map(|&s| f64::from(s)).collect();
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let field = Gravity::new(G).field(&sigma, sigma_vac, w, h).unwrap();
        let src = grav::source(&sigma, sigma_vac);
        let (net, scale) = net_and_scale(&src, &field.gx, &field.gy);
        let ratio = net[0].hypot(net[1]) / scale;
        let mass: f64 = src.iter().sum();
        println!("SR-GRAV2 asym_blob {w}x{h}: mass {mass:.3}, |Σ m·g| = {:.3e}, Σ m|g| = {scale:.3e}, ratio {ratio:.3e}", net[0].hypot(net[1]));
        // The blob is a real test: two bodies, centre of mass between them, a pull on every part of it.
        let cells = src.iter().filter(|&&s| s > 0.0).count();
        assert!(cells > 1500 && mass > 500.0, "{cells} cells, mass {mass}");
        assert!(scale > 100.0, "Σ m|g| = {scale}: the blob must pull itself for the test to say anything");
        assert!(ratio <= 1e-5, "G-GRAV2: |Σ m·g| / Σ m|g| = {ratio:.3e}");
    }

    #[test]
    fn random_asymmetric_masses_have_none_either() {
        // A seeded scatter of masses (xorshift) in a 50 × 37 box, a 3-cell rim left empty: the cancellation is the kernel's
        // symmetry, not the blob's shape.
        let (w, h) = (50usize, 37usize);
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut sigma = vec![0.0; w * h];
        for y in 3..h - 3 {
            for x in 3..w - 3 {
                if next() < 0.4 {
                    sigma[y * w + x] = 0.01 + 2.0 * next() * next();
                }
            }
        }
        let field = Gravity::new(G).field(&sigma, 1e-6, w, h).unwrap();
        let (net, scale) = net_and_scale(&sigma, &field.gx, &field.gy);
        let ratio = net[0].hypot(net[1]) / scale;
        println!("SR-GRAV2 random {w}x{h}: ratio {ratio:.3e}");
        assert!(scale > 1.0 && ratio <= 1e-5, "|Σ m·g| / Σ m|g| = {ratio:.3e}");
    }

    #[test]
    fn gpu_asym_blob_has_no_net_self_force() {
        let gpu = crate::device();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes/asym_blob.json");
        let scene = Scene::load(&root).unwrap();
        let physics = Physics::shipped().unwrap();
        let (planes, _) = scene.planes(scene.world, &physics, &Elements::shipped().unwrap()).unwrap();
        let (w, h) = (scene.world.width, scene.world.height);
        let solved = super::gpu::solve_planes(&gpu, &planes, w, h, 1, true);
        let field = solved.cropped();
        let src = grav::source(&field.sigma, physics.get("sigma_vac").unwrap());
        let (net, scale) = net_and_scale(&src, &field.gx, &field.gy);
        let ratio = net[0].hypot(net[1]) / scale;
        println!("SR-GRAV2 GPU asym_blob {w}x{h}, box {:?}: |Σ m·g| = {:.3e}, Σ m|g| = {scale:.3e}, ratio {ratio:.3e}", solved.area, net[0].hypot(net[1]));
        assert!(src.iter().filter(|&&s| s > 0.0).count() > 1500 && scale > 100.0, "the whole blob in the box, pulling itself");
        assert!(ratio <= 1e-5, "G-GRAV2 (GPU): |Σ m·g| / Σ m|g| = {ratio:.3e}");
        solved.against_twin("asym_blob");
    }

    /// G-GRAV2 on a non-square box whose outermost ring holds no matter: `scene` through one GPU step, the net force
    /// against the pull, the box the shape and FFT the case is written for.
    fn gpu_no_net_force_on(what: &str, (w, h, sigma): (u32, u32, Vec<f64>), box_wh: (u32, u32), fft: [u32; 2]) {
        let gpu = crate::device();
        let solved = super::gpu::solve(&gpu, &sigma, w, h, 1, true);
        assert_eq!((solved.area.w, solved.area.h, solved.area.fft), (box_wh.0, box_wh.1, fft), "{:?}", solved.area);
        let field = solved.cropped();
        let src = grav::source(&field.sigma, Physics::shipped().unwrap().get("sigma_vac").unwrap());
        let (net, scale) = net_and_scale(&src, &field.gx, &field.gy);
        let ratio = net[0].hypot(net[1]) / scale;
        println!("SR-GRAV2 GPU {what} {w}x{h}, box {:?}: |Σ m·g| = {:.3e}, Σ m|g| = {scale:.3e}, ratio {ratio:.3e}", solved.area, net[0].hypot(net[1]));
        assert!(scale > 100.0, "Σ m|g| = {scale}: the matter must pull itself for the case to say anything");
        assert!(ratio <= 1e-5, "G-GRAV2 (GPU, {what}): |Σ m·g| / Σ m|g| = {ratio:.3e}");
        solved.against_twin(what);
    }

    #[test]
    fn gpu_wide_box_has_no_net_self_force() {
        gpu_no_net_force_on("wide box", super::gpu::wide_scene(), (392, 64), [1024, 128]);
    }

    #[test]
    fn gpu_tall_box_has_no_net_self_force() {
        gpu_no_net_force_on("tall box", super::gpu::tall_scene(), (40, 344), [128, 1024]);
    }
}

/// P2 on the GPU, the cases' common part: a state uploaded, steps run with gravity on (or off), then the box, φ, g and
/// the Δt record read back.
mod gpu {
    use super::*;
    use sr_engine::gpu::Gpu;
    use sr_engine::state::{Booking, SideFields, State, WorldConfig, N_CHANNELS, SPECIES};
    use sr_engine::step::boxfit::{BoxMode, BoxReport};
    use sr_engine::step::dt::DtReport;
    use sr_engine::step::fft::read_back;
    use sr_engine::step::latch::FramePlan;
    use sr_engine::step::{EosGpu, Step};
    use sr_physics::registry::{Elements, Physics};

    /// G-REF's tolerance (§5.3): relative L1 against the twin.
    const REF_L1: f64 = 1e-3;

    /// A run as read back: Σ as uploaded, the box, φ, g (x plane, then y) and the Δt record.
    pub struct Solved {
        pub world: WorldConfig,
        pub sigma: Vec<f32>,
        pub area: BoxReport,
        pub phi: Vec<f32>,
        pub g: Vec<f32>,
        pub dt: DtReport,
    }

    /// The box's cells of a run, in f64, `area.w` wide.
    pub struct Cropped {
        pub sigma: Vec<f64>,
        pub phi: Vec<f64>,
        pub gx: Vec<f64>,
        pub gy: Vec<f64>,
    }

    impl Solved {
        pub fn cropped(&self) -> Cropped {
            let (b, w, cells) = (&self.area, self.world.width as usize, self.world.cells());
            let crop = |plane: &[f32]| -> Vec<f64> {
                let mut out = Vec::with_capacity((b.w * b.h) as usize);
                for y in b.y0..b.y0 + b.h {
                    let row = y as usize * w;
                    out.extend(plane[row + b.x0 as usize..row + (b.x0 + b.w) as usize].iter().map(|&v| f64::from(v)));
                }
                out
            };
            Cropped { sigma: crop(&self.sigma), phi: crop(&self.phi), gx: crop(&self.g[..cells]), gy: crop(&self.g[cells..]) }
        }

        /// φ, gx and gy over the box within G-REF's relative L1 of the twin run on the same box (Σ as uploaded).
        pub fn against_twin(&self, what: &str) {
            let c = self.cropped();
            let sigma_vac = Physics::shipped().unwrap().get("sigma_vac").unwrap();
            let twin = Gravity::new(G).field(&c.sigma, sigma_vac, self.area.w as usize, self.area.h as usize).unwrap();
            let l1 = |gpu: &[f64], twin: &[f64]| {
                gpu.iter().zip(twin).map(|(a, b)| (a - b).abs()).sum::<f64>() / twin.iter().map(|b| b.abs()).sum::<f64>()
            };
            let (phi, gx, gy) = (l1(&c.phi, &twin.phi), l1(&c.gx, &twin.gx), l1(&c.gy, &twin.gy));
            let b = &self.area;
            println!("SR-GRAV-REF {what}, box {}x{} at ({}, {}), FFT {:?}: relative L1 against the twin φ {phi:.2e}, gx {gx:.2e}, gy {gy:.2e}", b.w, b.h, b.x0, b.y0, b.fft);
            assert!(phi <= REF_L1 && gx <= REF_L1 && gy <= REF_L1, "{what}: the GPU against the twin over G-REF's {REF_L1}");
        }

        /// φ at nine cells of the box against the kernel summed directly over the masked sources — φ(x) = Σ_y K(x − y)
        /// Σ_src(y), no FFT, no table, no twin — each within `tol` relative. Printed as `SR-GRAV-DIRECT`.
        pub fn against_direct_sum(&self, what: &str, tol: f64) {
            let c = self.cropped();
            let (w, h) = (self.area.w as usize, self.area.h as usize);
            let src = grav::source(&c.sigma, Physics::shipped().unwrap().get("sigma_vac").unwrap());
            let sources: Vec<(isize, isize, f64)> = src
                .iter()
                .enumerate()
                .filter(|&(_, &m)| m > 0.0)
                .map(|(k, &m)| ((k % w) as isize, (k / w) as isize, m))
                .collect();
            let cells = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1), (w / 2, h / 2), (w / 3, h / 5), (2 * w / 3, 4 * h / 5), (w / 7, h / 2), (5 * w / 6, h / 3)];
            let mut worst = (0.0f64, (0, 0));
            for &(x, y) in &cells {
                let direct: f64 = sources.iter().map(|&(sx, sy, m)| m * grav::kernel(G, x as isize - sx, y as isize - sy)).sum();
                let err = (c.phi[y * w + x] - direct).abs() / direct.abs();
                if err > worst.0 {
                    worst = (err, (x, y));
                }
            }
            println!("SR-GRAV-DIRECT {what}, {} sources: φ against the direct sum at {} cells, worst relative {:.2e} at box cell {:?}", sources.len(), cells.len(), worst.0, worst.1);
            assert!(worst.0 <= tol, "{what}: φ against the direct sum of K over the sources, {:.2e} at box cell {:?} over {tol:e}", worst.0, worst.1);
        }
    }

    /// The scenes of M0-V4's probes, each with a box of a shape no square case has. Σ in the world's cells, row-major.
    ///
    /// A Maclaurin disk (a = 20, Σc = 1) at (150, 200) and a block of Σ 0.5 at x 400–499, y 185–214, in 600 × 400: the
    /// box 392 × 64 at an 8-cell margin, FFT 1024 × 128.
    pub fn wide_scene() -> (u32, u32, Vec<f64>) {
        let (w, h) = (600usize, 400usize);
        let mut sigma = vec![0.0; w * h];
        for j in 0..h {
            for i in 0..w {
                let r2 = ((i as f64 - 150.0).powi(2) + (j as f64 - 200.0).powi(2)) / 400.0;
                if r2 < 1.0 {
                    sigma[j * w + i] = (1.0 - r2).sqrt();
                }
                if (400..500).contains(&i) && (185..215).contains(&j) {
                    sigma[j * w + i] += 0.5;
                }
            }
        }
        (w as u32, h as u32, sigma)
    }

    /// A column of Σ 0.3 + 0.001·y, x 50–69, y 100–419, in 128 × 512: the box 40 × 344, FFT 128 × 1024.
    pub fn tall_scene() -> (u32, u32, Vec<f64>) {
        let (w, h) = (128usize, 512usize);
        let mut sigma = vec![0.0; w * h];
        for j in 100..420 {
            for i in 50..70 {
                sigma[j * w + i] = 0.3 + 0.001 * j as f64;
            }
        }
        (w as u32, h as u32, sigma)
    }

    /// Σ = 0.5 + 0.25·sin(x/37)·cos(y/23) in every cell of 600 × 400: the box is the world, FFT 2048 × 1024.
    pub fn whole_world_scene() -> (u32, u32, Vec<f64>) {
        let (w, h) = (600usize, 400usize);
        let sigma = (0..w * h)
            .map(|k| {
                let (i, j) = ((k % w) as f64, (k / w) as f64);
                0.5 + 0.25 * (i / 37.0).sin() * (j / 23.0).cos()
            })
            .collect();
        (w as u32, h as u32, sigma)
    }

    /// A faint Maclaurin disk (Σc = 1e-3, a = 20, floored at 1 % of Σc) at (64, 64) of a 128 × 128 world of 0.9 Σ_vac.
    pub fn faint_disk_in_sub_vacuum() -> (u32, u32, Vec<f64>) {
        let (w, h) = (128usize, 128usize);
        let mut sigma = vec![0.9 * Physics::shipped().unwrap().get("sigma_vac").unwrap(); w * h];
        for j in 0..h {
            for i in 0..w {
                let r2 = ((i as f64 - 64.0).powi(2) + (j as f64 - 64.0).powi(2)) / 400.0;
                if r2 < 1.0 {
                    sigma[j * w + i] = 1e-3 * (1.0 - r2).sqrt().max(0.01);
                }
            }
        }
        (w as u32, h as u32, sigma)
    }

    /// 14 planes: Σ as given, x_H = 1, every other channel 0 (cold: the energy floors at T_floor).
    fn planes(sigma: &[f64]) -> Vec<f32> {
        let cells = sigma.len();
        let mut planes = vec![0f32; N_CHANNELS * cells];
        for (p, &s) in planes.iter_mut().zip(sigma) {
            *p = s as f32;
        }
        planes[SPECIES.start * cells..(SPECIES.start + 1) * cells].fill(1.0);
        planes
    }

    /// `steps` bare steps of Σ (x_H = 1, cold) in a `w` × `h` world, gravity on or off.
    pub fn solve(gpu: &Gpu, sigma: &[f64], w: u32, h: u32, steps: u32, gravity: bool) -> Solved {
        solve_planes(gpu, &planes(sigma), w, h, steps, gravity)
    }

    pub fn solve_planes(gpu: &Gpu, planes: &[f32], w: u32, h: u32, steps: u32, gravity: bool) -> Solved {
        run(gpu, &Physics::shipped().unwrap(), planes, w, h, gravity, |step| step.run(&gpu.device, &gpu.queue, steps))
    }

    /// As [`solve`], one step under the constants `physics`.
    pub fn solve_with(gpu: &Gpu, physics: &Physics, sigma: &[f64], w: u32, h: u32, gravity: bool) -> Solved {
        run(gpu, physics, &planes(sigma), w, h, gravity, |step| step.run(&gpu.device, &gpu.queue, 1))
    }

    /// As [`solve`] with gravity on, the steps as one frame under the latch (the slow-down off).
    pub fn solve_framed(gpu: &Gpu, sigma: &[f64], w: u32, h: u32, steps: u32) -> Solved {
        run(gpu, &Physics::shipped().unwrap(), &planes(sigma), w, h, true, |step| {
            let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
            step.encode_frame(&FramePlan { steps, slow_down: false }, &mut encoder);
            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            let reports = step.poll(&gpu.device);
            assert_eq!(reports.len(), 1, "{reports:?}");
            assert_eq!(reports[0].steps_run, steps);
        })
    }

    fn run(gpu: &Gpu, physics: &Physics, planes: &[f32], w: u32, h: u32, gravity: bool, steps: impl FnOnce(&Step)) -> Solved {
        let world = WorldConfig::new(w, h).unwrap();
        let elements = Elements::shipped().unwrap();
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, planes).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, physics, &elements);
        let side = SideFields::new(&gpu.device, world).unwrap();
        let step = if gravity {
            Step::with_gravity(&gpu.device, &state, &booking, &eos, physics, BoxMode::Fit, &side).unwrap()
        } else {
            Step::new(&gpu.device, &state, &booking, &eos, physics)
        };
        steps(&step);
        let g = match step.gravity_field() {
            Some(b) => read_back(&gpu.device, &gpu.queue, b),
            None => vec![0.0; 2 * world.cells()],
        };
        Solved {
            world,
            sigma: planes[..world.cells()].to_vec(),
            area: step.read_box(&gpu.device, &gpu.queue),
            phi: read_back(&gpu.device, &gpu.queue, side.phi()),
            g,
            dt: step.read_dt(&gpu.device, &gpu.queue),
        }
    }
}
