//! M0-T81 — the glow (contract §1.11, §4.11; crates/sr-engine/src/render.rs, shaders/glow.wgsl) on the physics adapter (R12): the
//! Glow view's cell image (`Renderer::render_cells`, proven against the formulas by `views`) is read back, and the glow
//! (`Renderer::render`) is graded against a CPU model of §1.11 written out here from its text, in f64, from that image: the
//! bright pass on the brightest channel > 0.8 (M0-D112) at half resolution (the mean of a 2 × 2 block of cells, each kept or zeroed), a separable
//! 9-tap Gaussian of σ = 3 half-resolution texels (weights exp(−k²/2σ²) normalised over k = −4…4, zero outside), the halo
//! upsampled bilinearly (clamped at the edges) and added at 0.6, clamped to 1. A channel must land within one level of the 8-bit
//! image of the model. The worlds are hand-made, 64 × 64 (`NOT PROVEN (synthetic)` for a star's glow): the passes are proven.

use sr_engine::gpu::Gpu;
use sr_engine::render::{Renderer, View, IMAGE_FORMAT};
use sr_engine::state::{State as GpuState, WorldConfig};
use sr_engine::step::EosGpu;
use sr_physics::calibration::{physics_hash, Calibration};
use sr_physics::eos::Eos;
use sr_physics::observe::translate::TempScale;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const N: usize = 64;
const CELLS: usize = N * N;
const HALF: usize = N / 2;
/// One level of the RGBA8 image.
const LEVEL: f64 = 1.0;
const THRESHOLD: f64 = 0.8;
const SIGMA: f64 = 3.0;
const INTENSITY: f64 = 0.6;
const RADIUS: i64 = 4;

type Image = Vec<[u8; 4]>;

struct Rig {
    gpu: Gpu,
    renderer: Renderer,
    texture: wgpu::Texture,
}

fn idx(x: usize, y: usize) -> usize {
    y * N + x
}

/// Planes of a 64 × 64 world of hydrogen at the floor state with `cells` — (cell, Σ, T in kelvins) — written over it.
fn planes(physics: &Physics, elements: &Elements, scale: &TempScale, cells: &[(usize, f64, f64)]) -> Vec<f32> {
    let eos = Eos::from_physics(physics).unwrap();
    let mut x = [0.0; N_SPECIES];
    x[0] = 1.0;
    let comp = elements.composition(&x);
    let mut p = vec![0f32; 14 * CELLS];
    let mut put = |c: usize, sigma: f64, t: f64| {
        let e = sigma * (t * comp.inv_mu + eos.eps_cold(sigma, comp.y_e, x[9]));
        for (k, v) in [sigma, 0.0, 0.0, e].into_iter().chain(x).enumerate() {
            p[k * CELLS + c] = v as f32;
        }
    };
    let (floor_sigma, floor_t) = (physics.get("sigma_floor").unwrap(), physics.get("t_floor").unwrap());
    for c in 0..CELLS {
        put(c, floor_sigma, floor_t);
    }
    for &(c, sigma, kelvin) in cells {
        put(c, sigma, scale.to_sb(kelvin));
    }
    p
}

fn setup(cells: &[(usize, f64, f64)]) -> Rig {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let gpu = super::device();
    let cal = Calibration::test_calibration(&physics_hash().expect("the shipped assets are JSON"));
    let k = |key: &str| physics.get(key).unwrap();
    let anchors = [k("t_floor"), cal.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")];
    let scale = TempScale::new(anchors).unwrap();
    let p = planes(&physics, &elements, &scale, cells);
    let world = WorldConfig::new(N as u32, N as u32).unwrap();
    let (cs, ps) = (GpuState::new(&gpu.device, world).unwrap(), GpuState::new(&gpu.device, world).unwrap());
    cs.upload(&gpu.queue, &p).unwrap();
    ps.upload(&gpu.queue, &p).unwrap();
    let eos_gpu = EosGpu::new(&gpu.device, &physics, &elements);
    let renderer = Renderer::new(&gpu.device, &cs, &ps, &eos_gpu, anchors, &elements);
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glow.image"),
        size: wgpu::Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: IMAGE_FORMAT,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    Rig { gpu, renderer, texture }
}

impl Rig {
    /// The image `view` draws — `glow` for `Renderer::render`, else the cell image alone (`Renderer::render_cells`) — waited for.
    fn image(&self, view: View, glow: bool) -> Image {
        let (device, queue) = (&self.gpu.device, &self.gpu.queue);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glow.staging"),
            size: (N * N * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("glow") });
        let target = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
        if glow {
            self.renderer.render(view, 1.0, &target, &mut encoder);
        } else {
            self.renderer.render_cells(view, 1.0, &target, &mut encoder);
        }
        // 64 texels × 4 B = 256 B a row: already WebGPU's row alignment.
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &self.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(N as u32 * 4), rows_per_image: Some(N as u32) },
            },
            wgpu::Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 },
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.map_async(wgpu::MapMode::Read, .., move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let out = {
            let view = staging.get_mapped_range(..).unwrap();
            view.as_chunks::<4>().0.to_vec()
        };
        staging.unmap();
        out
    }
}

fn rgb(image: &Image, c: usize) -> [f64; 3] {
    std::array::from_fn(|k| f64::from(image[c][k]) / 255.0)
}

/// The largest of the three channels: what the bright pass tests (M0-D112).
fn brightest(c: [f64; 3]) -> f64 {
    c[0].max(c[1]).max(c[2])
}

/// The luminance the bright pass used to test (0.2126 R + 0.7152 G + 0.0722 B): the premise of the hot cases, whose luminance is below the threshold.
fn luminance(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// §1.11's halo of `cells` (the Glow view's cell image): the bright pass, the two Gaussian passes, the upsample — per cell, before
/// the intensity. Written from the contract's text, in f64.
fn halo(cells: &Image) -> Vec<[f64; 3]> {
    // The bright pass at half resolution: the mean of a 2 × 2 block, each cell kept when its brightest channel is above the threshold.
    let mut half = vec![[0.0; 3]; HALF * HALF];
    for hy in 0..HALF {
        for hx in 0..HALF {
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let c = rgb(cells, idx(2 * hx + dx, 2 * hy + dy));
                if brightest(c) > THRESHOLD {
                    for k in 0..3 {
                        half[hy * HALF + hx][k] += c[k] / 4.0;
                    }
                }
            }
        }
    }
    // The 9-tap Gaussian, normalised to sum 1.
    let tap = |k: i64| (-((k * k) as f64) / (2.0 * SIGMA * SIGMA)).exp();
    let norm: f64 = (-RADIUS..=RADIUS).map(tap).sum();
    let blur = |src: &[[f64; 3]], horizontal: bool| -> Vec<[f64; 3]> {
        let mut out = vec![[0.0; 3]; HALF * HALF];
        for y in 0..HALF as i64 {
            for x in 0..HALF as i64 {
                for k in -RADIUS..=RADIUS {
                    let (qx, qy) = if horizontal { (x + k, y) } else { (x, y + k) };
                    if (0..HALF as i64).contains(&qx) && (0..HALF as i64).contains(&qy) {
                        for ch in 0..3 {
                            out[(y * HALF as i64 + x) as usize][ch] += tap(k) / norm * src[(qy * HALF as i64 + qx) as usize][ch];
                        }
                    }
                }
            }
        }
        out
    };
    let half = blur(&blur(&half, true), false);
    // The upsample: bilinear, texel i centred on cell 2i + 0.5, clamped at the edges.
    let at = |x: i64, y: i64| half[(y.clamp(0, HALF as i64 - 1) * HALF as i64 + x.clamp(0, HALF as i64 - 1)) as usize];
    let mut out = vec![[0.0; 3]; CELLS];
    for y in 0..N {
        for x in 0..N {
            let (fx, fy) = ((x as f64 + 0.5) / 2.0 - 0.5, (y as f64 + 0.5) / 2.0 - 0.5);
            let (x0, y0) = (fx.floor(), fy.floor());
            let (tx, ty) = (fx - x0, fy - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            for ch in 0..3 {
                let top = at(x0, y0)[ch] * (1.0 - tx) + at(x0 + 1, y0)[ch] * tx;
                let bottom = at(x0, y0 + 1)[ch] * (1.0 - tx) + at(x0 + 1, y0 + 1)[ch] * tx;
                out[idx(x, y)][ch] = top * (1.0 - ty) + bottom * ty;
            }
        }
    }
    out
}

/// The model's glow image: the cell image plus 0.6 of its halo, clamped, in levels (0…255, unrounded).
fn model(cells: &Image) -> Vec<[f64; 3]> {
    let h = halo(cells);
    (0..CELLS).map(|c| std::array::from_fn(|k| 255.0 * (rgb(cells, c)[k] + INTENSITY * h[c][k]).clamp(0.0, 1.0))).collect()
}

/// The worst deviation, in levels, of `got` from the model of `cells`, and the cell.
fn worst(got: &Image, want: &[[f64; 3]]) -> (f64, usize) {
    let mut w = (0.0, 0);
    for c in 0..CELLS {
        for k in 0..3 {
            let d = (f64::from(got[c][k]) - want[c][k]).abs();
            if d > w.0 {
                w = (d, c);
            }
        }
    }
    w
}

fn assert_glow_matches(rig: &Rig) -> (Image, Image) {
    let cells = rig.image(View::Glow, false);
    let got = rig.image(View::Glow, true);
    let want = model(&cells);
    let (d, c) = worst(&got, &want);
    assert!(d <= LEVEL, "cell {c} is off by {d:.3} levels: got {:?}, model {:.2?}, cell image {:?}", got[c], want[c], cells[c]);
    (cells, got)
}

/// 6,500 K at b ≈ 1: near-white, whose largest channel (1.0 in the image) is past 0.8 — checked on the image by each test. Hotter
/// matter is too (M0-D112): `hotter_than_the_luminance_band_still_blooms` draws the table's end and past it.
const HOT_K: f64 = 6_500.0;
const HOT_SIGMA: f64 = 0.2;

fn hot(cells: impl IntoIterator<Item = usize>) -> Vec<(usize, f64, f64)> {
    cells.into_iter().map(|c| (c, HOT_SIGMA, HOT_K)).collect()
}

fn block(x0: usize, y0: usize, w: usize, h: usize) -> Vec<usize> {
    (y0..y0 + h).flat_map(|y| (x0..x0 + w).map(move |x| idx(x, y))).collect()
}

fn bright_cells(image: &Image) -> Vec<usize> {
    (0..CELLS).filter(|&c| brightest(rgb(image, c)) > THRESHOLD).collect()
}

#[test]
fn a_lone_bright_cell_halo_follows_the_gaussian() {
    let rig = setup(&hot([idx(32, 32)]));
    let (cells, got) = assert_glow_matches(&rig);
    assert_eq!(bright_cells(&cells), vec![idx(32, 32)], "the premise: exactly one cell is past the threshold");
    // The halo reaches 4 half-resolution texels either side of the cell's texel and the upsample one more: 11 cells at most.
    for y in 0..N {
        for x in 0..N {
            if x.abs_diff(32).max(y.abs_diff(32)) > 11 {
                assert_eq!(got[idx(x, y)], cells[idx(x, y)], "cell ({x}, {y}) is past the halo's reach and must not change");
            }
        }
    }
    // The cell's own texel is the halo's peak: the weights fall away from it.
    let level = |x: usize, y: usize| rgb(&got, idx(x, y))[2] - rgb(&cells, idx(x, y))[2];
    assert!(level(33, 33) >= level(38, 33) && level(33, 33) >= level(33, 38), "the halo falls away from the cell");
}

#[test]
fn a_bright_block_halo_follows_the_gaussian_and_is_visible() {
    let rig = setup(&hot(block(24, 24, 16, 16)));
    let (cells, got) = assert_glow_matches(&rig);
    assert_eq!(bright_cells(&cells).len(), 16 * 16, "the premise: the block is the bright cells");
    // Next to the block the halo is well above one level, and it falls away outside: 4 cells out is brighter than 12 out.
    let blue = |image: &Image, x: usize, y: usize| f64::from(image[idx(x, y)][2]);
    let (near, far) = (blue(&got, 22, 32) - blue(&cells, 22, 32), blue(&got, 14, 32) - blue(&cells, 14, 32));
    assert!(near > 20.0, "the halo at the block's edge is {near} levels of blue");
    assert!(near > far, "the halo falls away: {near} levels at the edge, {far} ten cells out");
}

#[test]
fn nothing_is_added_below_the_threshold() {
    // A gradient of warm cells, cool and moderately hot, with the dimmer half of the blackbody table: no channel past 0.8.
    let mut cells = Vec::new();
    for x in 0..N {
        for y in 0..N {
            let kelvin = 700.0 * (2_600.0_f64 / 700.0).powf(x as f64 / (N - 1) as f64);
            cells.push((idx(x, y), 0.01 + 0.2 * y as f64 / N as f64, kelvin));
        }
    }
    let rig = setup(&cells);
    let base = rig.image(View::Glow, false);
    let got = rig.image(View::Glow, true);
    let top = (0..CELLS).map(|c| brightest(rgb(&base, c))).fold(0.0, f64::max);
    assert!(top > 0.2 && top <= THRESHOLD, "the premise: the world is lit but nowhere past the threshold (peak channel {top:.3})");
    assert_eq!(got, base, "no cell is past the threshold: the glow image is the cell image, byte for byte");
}

#[test]
fn a_bright_cell_at_the_corner_and_one_on_the_edge() {
    let rig = setup(&hot([idx(0, 0), idx(63, 20), idx(40, 63)]));
    let (cells, _) = assert_glow_matches(&rig);
    assert_eq!(bright_cells(&cells).len(), 3, "the premise: three cells are past the threshold");
}

#[test]
fn the_other_views_are_untouched() {
    let mut cells = hot(block(24, 24, 16, 16));
    cells.extend((0..CELLS).filter(|c| c % 7 == 0).map(|c| (c, 0.05, 3_000.0)));
    let rig = setup(&cells);
    for view in [View::Heat, View::Element, View::Density] {
        let alone = rig.image(view, false);
        let drawn = rig.image(view, true);
        assert_eq!(drawn, alone, "{view:?} is the cell image alone: no glow reaches it");
    }
    // And the premise: the same world's Glow view does change.
    assert_ne!(rig.image(View::Glow, true), rig.image(View::Glow, false), "the Glow view carries the glow");
}

#[test]
fn hotter_than_the_luminance_band_still_blooms() {
    // 40,000 K is the table's end colour (158, 184, 255) at luminance 0.72, and 10⁷ K is past the table (the end colour again):
    // a luminance test missed both (M0-D112). Each fills a 16 × 16 block, so its halo is well above one level; the model is the
    // Gaussian's weights in f64 over the cell image, as the lone-cell case is.
    let mut cells = Vec::new();
    for (x0, kelvin) in [(8, 40_000.0), (40, 1.0e7)] {
        cells.extend(block(x0, 24, 16, 16).into_iter().map(|c| (c, HOT_SIGMA, kelvin)));
    }
    let rig = setup(&cells);
    let (image, got) = assert_glow_matches(&rig);
    assert_eq!(bright_cells(&image).len(), 2 * 16 * 16, "the premise: both blocks are past the threshold");
    for (x, label) in [(12, "40,000 K"), (44, "10⁷ K")] {
        let c = rgb(&image, idx(x, 32));
        assert!(luminance(c) <= THRESHOLD, "the premise: the {label} cell's luminance {:.3} is under the threshold", luminance(c));
        assert!(brightest(c) > THRESHOLD, "the premise: its brightest channel {:.3} is over it", brightest(c));
    }
    // Beside each block the empty world is dark and the halo is the glow: 2 cells off the 40,000 K block's left edge, and off the
    // 10⁷ K block's right edge.
    let blue = |image: &Image, x: usize, y: usize| f64::from(image[idx(x, y)][2]);
    for x in [6, 57] {
        let added = blue(&got, x, 32) - blue(&image, x, 32);
        assert!(added > 10.0, "the halo at ({x}, 32) is {added} levels of blue");
    }
}

#[test]
fn dense_cold_gas_does_not_bloom() {
    // The haze is at most 0.6 × #2A3550, whose largest channel is 0.314: the densest, coldest gas stays the cell image.
    let cells: Vec<_> = (0..CELLS).map(|c| (c, 1.0e3, 100.0)).collect();
    let rig = setup(&cells);
    let base = rig.image(View::Glow, false);
    let top = (0..CELLS).map(|c| brightest(rgb(&base, c))).fold(0.0, f64::max);
    assert!(top > 0.1 && top < 0.4, "the premise: the world is the haze, lit but dim (peak channel {top:.3})");
    assert_eq!(rig.image(View::Glow, true), base, "the haze does not bloom: the glow image is the cell image, byte for byte");
}
