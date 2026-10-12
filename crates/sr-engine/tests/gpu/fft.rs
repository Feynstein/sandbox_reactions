//! M0-T30 — the GPU FFT and the convolution over the box (contract §1.3.3, §1.4.2, §6.2.2), on the physics adapter
//! (R12, R15).
//!
//! The oracle is M0-T28's twin, sr_physics::reference::fft (itself graded against the direct DFT and direct summation
//! by the `fft_twin` scope), run in f64 on the same f32 inputs. Tolerance, fixed before the first run (the block's):
//! ‖gpu − twin‖₂ / ‖twin‖₂ ≤ 1e-5 — f32 arithmetic over log₂ N ≤ 21 butterfly levels. The device is the helper's,
//! at WebGPU's `Limits::default()` (§6.2.2); a shader over those limits fails its pipeline's validation, so every
//! case is also the limits' check. Each case prints its dispatch count (`SR-FFT`), the cost case its ms (`SR-FFT-COST`).

use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use super::device;
use sr_engine::gpu::Gpu;
use sr_engine::step::fft::{self, Convolution, Fft, Plane, Region, CONVOLUTION_DISPATCHES, TRANSFORM_DISPATCHES};
use sr_engine::step::Dispatch;
use sr_physics::reference::fft::{fft2, Complex, Convolver, FftPlan};
use sr_physics::reference::grav;
use wgpu::util::DeviceExt;

const TOL: f64 = 1e-5;

/// A deterministic field in [−1, 1): the twin's tests' 64-bit LCG, so a failure reproduces.
fn pseudo_random(count: usize, seed: u64) -> Vec<f32> {
    let mut s = seed;
    (0..count)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (((s >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0) as f32
        })
        .collect()
}

/// ‖a − b‖₂ / ‖b‖₂.
fn rel_l2(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let diff: f64 = a.iter().zip(b).map(|(p, q)| (p - q) * (p - q)).sum();
    let norm: f64 = b.iter().map(|q| q * q).sum();
    (diff / norm).sqrt()
}

fn storage(gpu: &Gpu, label: &str, values: &[f32]) -> wgpu::Buffer {
    gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &values.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    })
}

fn run(gpu: &Gpu, dispatches: &[Dispatch]) {
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    fft::encode(&mut encoder, dispatches);
    gpu.queue.submit([encoder.finish()]);
}

/// Interleaved re, im (the GPU's vec2<f32> layout) to the twin's complex values, and back to f64 pairs.
fn to_complex(v: &[f32]) -> Vec<Complex> {
    v.chunks_exact(2).map(|c| Complex::new(f64::from(c[0]), f64::from(c[1]))).collect()
}

fn pairs(v: &[Complex]) -> Vec<f64> {
    v.iter().flat_map(|z| [z.re, z.im]).collect()
}

fn widen(v: &[f32]) -> Vec<f64> {
    v.iter().map(|&x| f64::from(x)).collect()
}

/// The forward transform of a random field against the twin's; then the inverse, on the GPU, of the twin's spectrum
/// (rounded to f32) against the twin's inverse of the same values — each direction graded on its own.
fn check_transform(gpu: &Gpu, f: &Fft, nx: usize, ny: usize, seed: u64) {
    let (plan_x, plan_y) = (FftPlan::new(nx).unwrap(), FftPlan::new(ny).unwrap());
    let input = pseudo_random(2 * nx * ny, seed);
    let data = storage(gpu, "fft.test.data", &input);
    let forward = f.transform(&gpu.device, &data, nx as u32, ny as u32, false).unwrap();
    assert_eq!(forward.len(), TRANSFORM_DISPATCHES);
    run(gpu, &forward);
    let gpu_forward = widen(&fft::read_back(&gpu.device, &gpu.queue, &data));
    let mut twin = to_complex(&input);
    fft2(&mut twin, &plan_x, &plan_y, false).unwrap();
    let e_forward = rel_l2(&gpu_forward, &pairs(&twin));

    let spectrum: Vec<f32> = pairs(&twin).iter().map(|&v| v as f32).collect();
    let data = storage(gpu, "fft.test.spectrum", &spectrum);
    let inverse = f.transform(&gpu.device, &data, nx as u32, ny as u32, true).unwrap();
    assert_eq!(inverse.len(), TRANSFORM_DISPATCHES);
    run(gpu, &inverse);
    let gpu_inverse = widen(&fft::read_back(&gpu.device, &gpu.queue, &data));
    let mut twin_inverse = to_complex(&spectrum);
    fft2(&mut twin_inverse, &plan_x, &plan_y, true).unwrap();
    let e_inverse = rel_l2(&gpu_inverse, &pairs(&twin_inverse));
    println!(
        "SR-FFT {nx}x{ny} transform: {} dispatches each way; forward rel L2 {e_forward:.2e}, inverse {e_inverse:.2e} \
         (limit {TOL:e})",
        forward.len()
    );
    assert!(e_forward <= TOL, "{nx} × {ny} forward: relative L2 {e_forward:e} over {TOL:e}");
    assert!(e_inverse <= TOL, "{nx} × {ny} inverse: relative L2 {e_inverse:e} over {TOL:e}");
}

#[test]
fn transform_64_by_64_matches_the_twin() {
    let gpu = device();
    check_transform(&gpu, &Fft::new(&gpu.device), 64, 64, 1);
}

#[test]
fn transform_256_by_128_matches_the_twin() {
    let gpu = device();
    check_transform(&gpu, &Fft::new(&gpu.device), 256, 128, 2);
}

#[test]
fn transform_2048_by_1024_matches_the_twin() {
    let gpu = device();
    check_transform(&gpu, &Fft::new(&gpu.device), 2048, 1024, 3);
}

#[test]
fn transform_every_length_32_to_2048_on_both_axes_and_the_refusals() {
    let gpu = device();
    let f = Fft::new(&gpu.device);
    // 32 … 2048 each as rows and as columns, the extremes paired
    for (i, (nx, ny)) in [(32, 2048), (2048, 32), (64, 512), (512, 64), (128, 1024), (1024, 128), (256, 256)]
        .into_iter()
        .enumerate()
    {
        check_transform(&gpu, &f, nx, ny, 10 + i as u64);
    }
    let data = storage(&gpu, "fft.test.refuse", &[0.0; 2 * 64 * 64]);
    for (nx, ny, bad) in [(16, 64, 16), (64, 4096, 4096), (48, 64, 48), (64, 0, 0)] {
        assert_eq!(f.transform(&gpu.device, &data, nx, ny, false).err(), Some(fft::FftError::Length(bad)));
    }
    assert_eq!(fft::padded(1024), Ok(2048));
    assert_eq!(fft::padded(1025), Err(fft::FftError::BoxAxis(1025)));
    assert_eq!(fft::padded(0), Err(fft::FftError::BoxAxis(0)));
    assert_eq!(fft::padded(1), Ok(32));
    assert_eq!(fft::padded(300), Ok(1024));
}

/// The twin's asymmetric kernel (its fft.rs tests): K(dx, dy) ≠ K(−dx, dy) and ≠ K(dx, −dy), long-ranged, so a flipped
/// axis, a shifted offset or a wrap-around shows.
fn asymmetric_kernel(dx: isize, dy: isize) -> f64 {
    let (x, y) = (dx as f64, dy as f64);
    (1.0 + 0.5 * (0.2 * x).tanh() + 0.3 * (0.1 * y).tanh()) / (1.0 + x * x + 2.0 * y * y).sqrt()
}

/// A world of `world_w` × `world_h` holding `field` (random everywhere, the box's cells and the rest), convolved over
/// `region` on the GPU, against the twin's convolution of the box's cells alone. The destination is the second plane
/// of a two-plane buffer, filled with a sentinel: cells outside the box must keep it.
fn check_convolution(
    gpu: &Gpu,
    conv: &Convolution,
    twin: &mut Convolver,
    (world_w, world_h): (u32, u32),
    region: Region,
    field: &[f32],
) -> f64 {
    let cells = (world_w * world_h) as usize;
    const SENTINEL: f32 = -12345.0;
    let src = storage(gpu, "fft.test.src", field);
    let dst = storage(gpu, "fft.test.dst", &vec![SENTINEL; 2 * cells]);
    let d = conv
        .dispatches(&gpu.device, &gpu.queue, Plane { buffer: &src, offset: 0 }, Plane { buffer: &dst, offset: cells as u32 }, world_w, region)
        .unwrap();
    assert_eq!(d.len(), CONVOLUTION_DISPATCHES);
    run(gpu, &d);
    let out = fft::read_back(&gpu.device, &gpu.queue, &dst);
    assert!(out[..cells].iter().all(|&v| v == SENTINEL), "the first plane was written");
    let (w, h) = (region.w as usize, region.h as usize);
    let mut boxed = vec![0.0; w * h];
    let mut got = vec![0.0; w * h];
    for y in 0..world_h as usize {
        for x in 0..world_w as usize {
            let v = out[cells + y * world_w as usize + x];
            let (bx, by) = (x.wrapping_sub(region.x0 as usize), y.wrapping_sub(region.y0 as usize));
            if bx < w && by < h {
                boxed[by * w + bx] = f64::from(field[y * world_w as usize + x]);
                got[by * w + bx] = f64::from(v);
            } else {
                assert_eq!(v, SENTINEL, "cell ({x}, {y}) outside the box was written");
            }
        }
    }
    let want = twin.convolve(&boxed, w, h).unwrap();
    let e = rel_l2(&got, &want);
    let (nx, ny) = (fft::padded(region.w).unwrap(), fft::padded(region.h).unwrap());
    println!(
        "SR-FFT convolution box {}x{} at ({}, {}) padded {nx}x{ny}: {} dispatches (+{TRANSFORM_DISPATCHES} once per \
         size for the kernel); rel L2 {e:.2e} (limit {TOL:e})",
        region.w,
        region.h,
        region.x0,
        region.y0,
        d.len()
    );
    assert!(e <= TOL, "convolution over {region:?}: relative L2 {e:e} over {TOL:e}");
    e
}

#[test]
fn convolution_matches_the_twin_with_an_asymmetric_kernel_on_boxes_inside_a_world() {
    let gpu = device();
    let conv = Convolution::new(&gpu.device, asymmetric_kernel);
    let mut twin = Convolver::new(asymmetric_kernel);
    let world = (160u32, 120u32);
    // positive masses, so the box's rim carries weight; cells outside the box hold mass too, which must be ignored
    let field: Vec<f32> = pseudo_random(160 * 120, 29).iter().map(|v| v + 1.0).collect();
    // off the origin (128 × 128), at the world's corner, at its far corner, and a box of one row
    for region in [
        Region { x0: 24, y0: 16, w: 48, h: 40 },
        Region { x0: 0, y0: 0, w: 40, h: 33 },
        Region { x0: 96, y0: 72, w: 64, h: 48 },
        Region { x0: 8, y0: 100, w: 150, h: 1 },
    ] {
        check_convolution(&gpu, &conv, &mut twin, world, region, &field);
    }
}

#[test]
fn convolution_of_gravity_over_the_whole_600_by_400_world_matches_the_twin() {
    let gpu = device();
    let kernel = |dx, dy| grav::kernel(1.0, dx, dy);
    let conv = Convolution::new(&gpu.device, kernel);
    let mut twin = Convolver::new(kernel);
    let field: Vec<f32> = pseudo_random(600 * 400, 31).iter().map(|v| 0.5 * (v + 1.0)).collect();
    check_convolution(&gpu, &conv, &mut twin, (600, 400), Region { x0: 0, y0: 0, w: 600, h: 400 }, &field);
    assert_eq!(conv.cached_sizes(), vec![(2048, 1024)]);
}

#[test]
fn the_kernel_transform_is_cached_per_padded_size() {
    let gpu = device();
    let evals = Rc::new(Cell::new(0usize));
    let counter = Rc::clone(&evals);
    let conv = Convolution::new(&gpu.device, move |dx, dy| {
        counter.set(counter.get() + 1);
        asymmetric_kernel(dx, dy)
    });
    let mut twin = Convolver::new(asymmetric_kernel);
    let field: Vec<f32> = pseudo_random(64 * 64, 41).iter().map(|v| v + 1.0).collect();
    // 48 × 40 and 50 × 41 both pad to 128 × 128: sampled once; 20 × 10 pads to 64 × 32, a second entry
    check_convolution(&gpu, &conv, &mut twin, (64, 64), Region { x0: 8, y0: 8, w: 48, h: 40 }, &field);
    assert_eq!(conv.cached_sizes(), vec![(128, 128)]);
    assert_eq!(evals.get(), 128 * 128);
    check_convolution(&gpu, &conv, &mut twin, (64, 64), Region { x0: 0, y0: 16, w: 50, h: 41 }, &field);
    assert_eq!(evals.get(), 128 * 128, "the same padded size must not resample the kernel");
    check_convolution(&gpu, &conv, &mut twin, (64, 64), Region { x0: 40, y0: 50, w: 20, h: 10 }, &field);
    assert_eq!(conv.cached_sizes(), vec![(64, 32), (128, 128)]);
    assert_eq!(evals.get(), 128 * 128 + 64 * 32);
    // a box that leaves its plane's rows, or pads past 2048, is refused before anything is made
    let buf = storage(&gpu, "fft.test.refuse", &[0.0; 64 * 64]);
    let plane = Plane { buffer: &buf, offset: 0 };
    let refused = |r: Region| conv.dispatches(&gpu.device, &gpu.queue, plane, plane, 64, r).err();
    assert_eq!(refused(Region { x0: 40, y0: 0, w: 32, h: 8 }), Some(fft::FftError::Outside { x0: 40, w: 32, stride: 64 }));
    assert_eq!(refused(Region { x0: 0, y0: 0, w: 1025, h: 8 }), Some(fft::FftError::BoxAxis(1025)));
    assert_eq!(evals.get(), 128 * 128 + 64 * 32);
}

/// Not graded (no ⏱ scope, R12): the GPU time per convolution at TW's three box sizes, the twiddles tabulated, beside
/// the proxy's figures (reports/time_warp.md M2–M4, the Quadro RTX 4000: gravity 1.24 ms at 2048 × 1024, 0.17 ms at
/// 512 × 512, a whole step 0.11 ms at 128 × 128). Wall clock over batches of 20 convolutions in one submit, the median
/// of 5 batches after a warm-up, so it includes one submit's overhead shared by 20.
#[test]
fn cost_per_convolution_in_the_log() {
    let gpu = device();
    let conv = Convolution::new(&gpu.device, |dx, dy| grav::kernel(1.0, dx, dy));
    let (world_w, world_h) = (600u32, 400u32);
    let cells = (world_w * world_h) as usize;
    let src = storage(&gpu, "fft.cost.src", &pseudo_random(cells, 51));
    let dst = storage(&gpu, "fft.cost.dst", &vec![0.0; cells]);
    for (region, proxy) in [
        (Region { x0: 268, y0: 168, w: 64, h: 64 }, "TW M4: whole step 0.11 ms at 128x128"),
        (Region { x0: 220, y0: 120, w: 160, h: 160 }, "TW M3: gravity 0.17 ms at 512x512"),
        (Region { x0: 0, y0: 0, w: 600, h: 400 }, "TW M2: gravity 1.24 ms at 2048x1024"),
    ] {
        let d = conv
            .dispatches(&gpu.device, &gpu.queue, Plane { buffer: &src, offset: 0 }, Plane { buffer: &dst, offset: 0 }, world_w, region)
            .unwrap();
        let batch = |n: usize| {
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            for _ in 0..n {
                fft::encode(&mut encoder, &d);
            }
            let t = Instant::now();
            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            t.elapsed().as_secs_f64() * 1e3 / n as f64
        };
        batch(5);
        let mut ms: Vec<f64> = (0..5).map(|_| batch(20)).collect();
        ms.sort_by(f64::total_cmp);
        let (nx, ny) = (fft::padded(region.w).unwrap(), fft::padded(region.h).unwrap());
        println!(
            "SR-FFT-COST box {}x{} padded {nx}x{ny}: {:.3} ms per convolution ({} dispatches, twiddles tabulated; \
             median of 5 batches of 20, wall); {proxy}",
            region.w,
            region.h,
            ms[2],
            d.len()
        );
        assert!(ms[2].is_finite() && ms[2] > 0.0);
    }
}
