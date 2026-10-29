//! M0-T80 — the cell image and the four views (contract §1.11, §1.10.2, §1.8.3, §4.11; crates/sr-engine/src/render.rs,
//! shaders/render.wgsl) on the physics adapter (R12), graded against a CPU formula written out here from the contract's text
//! — the stops are §4.11's hex strings, the colours §1.5's, the temperature path the cell's f32 values through the EOS and
//! `TempScale` in f64 — never the shader's constants. A cell's channel must land within 1/255 of the formula (one level of
//! the RGBA8 image). The worlds are hand-made, 64 × 64 (`NOT PROVEN (synthetic)` for a star's image): the formula is proven.
//!
//! Gentle worlds for the views that read T (Σ ≤ 0.25, T ≥ 0.08): there ε_cold is a few units and the table's ~2 × 10⁻⁵
//! relative error (eos.wgsl) stays under a tenth of a level in the colour; the wide world (Σ to 10⁴, every species) serves the
//! views that read only Σ and X.

use sr_engine::gpu::Gpu;
use sr_engine::render::{blackbody_lookup, blackbody_table, Renderer, View, IMAGE_FORMAT};
use sr_engine::state::{State as GpuState, WorldConfig};
use sr_engine::step::EosGpu;
use sr_physics::calibration::{physics_hash, Calibration};
use sr_physics::eos::Eos;
use sr_physics::observe::translate::TempScale;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const N: usize = 64;
const CELLS: usize = N * N;
/// One level of the RGBA8 image, in the formula's 0…1 scale ×255.
const LEVEL: f64 = 1.0;

const HEAT_STOPS: [&str; 6] = ["#000004", "#3B0F70", "#8C2981", "#DE4968", "#FE9F6D", "#FCFDBF"];
const DENSITY_STOPS: [&str; 5] = ["#440154", "#3B528B", "#21918C", "#5EC962", "#FDE725"];
const HAZE: &str = "#2A3550";
const BG_SPACE: &str = "#05070D";

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn hex(s: &str) -> [f64; 3] {
    let v = |i: usize| f64::from(u8::from_str_radix(&s[1 + 2 * i..3 + 2 * i], 16).unwrap()) / 255.0;
    [v(0), v(1), v(2)]
}

fn lin(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn enc(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// A map of `stops` (hex) at p ∈ [0, 1] along them, interpolated in linear RGB and sRGB-encoded (§4.11).
fn map(stops: &[&str], p: f64) -> [f64; 3] {
    let s = p.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let i = (s.floor() as usize).min(stops.len() - 2);
    let (a, b, f) = (hex(stops[i]), hex(stops[i + 1]), s - i as f64);
    std::array::from_fn(|k| enc(lin(a[k]) * (1.0 - f) + lin(b[k]) * f))
}

/// A cell of the CPU formula: the 14 channels, f64 from the f32 values the GPU holds.
#[derive(Clone, Debug)]
struct Cell {
    sigma: f64,
    mom: [f64; 2],
    energy: f64,
    x: [f64; N_SPECIES],
}

/// What a test puts in a cell: Σ, the velocity, T in sandbox units and the mass fractions (normalised on use).
#[derive(Clone)]
struct Spec {
    sigma: f64,
    vel: [f64; 2],
    t: f64,
    x: [f64; N_SPECIES],
}

fn pure(i: usize) -> [f64; N_SPECIES] {
    let mut x = [0.0; N_SPECIES];
    x[i] = 1.0;
    x
}

fn gas(sigma: f64, t: f64) -> Spec {
    Spec { sigma, vel: [0.0; 2], t, x: pure(0) }
}

struct Rig {
    gpu: Gpu,
    renderer: Renderer,
    eos: Eos,
    elements: Elements,
    scale: TempScale,
    anchors: [f64; 8],
    physics: Physics,
    texture: wgpu::Texture,
    cur_planes: Vec<f32>,
    prev_planes: Vec<f32>,
}

fn anchors(c: &Calibration, p: &Physics) -> [f64; 8] {
    let k = |key: &str| p.get(key).unwrap();
    [k("t_floor"), c.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")]
}

/// The planes of a world holding `cells` (by index) over the floor state.
fn planes(physics: &Physics, elements: &Elements, cells: &[(usize, Spec)]) -> Vec<f32> {
    let eos = Eos::from_physics(physics).unwrap();
    let floor = gas(physics.get("sigma_floor").unwrap(), physics.get("t_floor").unwrap());
    let mut p = vec![0f32; 14 * CELLS];
    let mut put = |c: usize, s: &Spec| {
        let total: f64 = s.x.iter().sum();
        let x: [f64; N_SPECIES] = std::array::from_fn(|i| s.x[i] / total);
        let comp = elements.composition(&x);
        let e = s.sigma * (s.t * comp.inv_mu + eos.eps_cold(s.sigma, comp.y_e, x[9]))
            + 0.5 * s.sigma * (s.vel[0] * s.vel[0] + s.vel[1] * s.vel[1]);
        for (k, v) in [s.sigma, s.sigma * s.vel[0], s.sigma * s.vel[1], e].into_iter().chain(x).enumerate() {
            p[k * CELLS + c] = v as f32;
        }
    };
    for c in 0..CELLS {
        put(c, &floor);
    }
    for (c, s) in cells {
        put(*c, s);
    }
    p
}

fn rig(cur: Vec<f32>, prev: Vec<f32>, physics: Physics, elements: Elements) -> Rig {
    let gpu = super::device();
    let world = WorldConfig::new(N as u32, N as u32).unwrap();
    let (cs, ps) = (GpuState::new(&gpu.device, world).unwrap(), GpuState::new(&gpu.device, world).unwrap());
    cs.upload(&gpu.queue, &cur).unwrap();
    ps.upload(&gpu.queue, &prev).unwrap();
    let eos_gpu = EosGpu::new(&gpu.device, &physics, &elements);
    let cal = Calibration::test_calibration(&physics_hash().expect("the shipped assets are JSON"));
    let a = anchors(&cal, &physics);
    let renderer = Renderer::new(&gpu.device, &cs, &ps, &eos_gpu, a, &elements);
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("views.image"),
        size: wgpu::Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: IMAGE_FORMAT,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    Rig {
        eos: Eos::from_physics(&physics).unwrap(),
        scale: TempScale::new(a).unwrap(),
        anchors: a,
        gpu,
        renderer,
        elements,
        physics,
        texture,
        cur_planes: cur,
        prev_planes: prev,
    }
}

fn setup(cur: &[(usize, Spec)], prev: &[(usize, Spec)]) -> Rig {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let (c, p) = (planes(&physics, &elements, cur), planes(&physics, &elements, prev));
    rig(c, p, physics, elements)
}

impl Rig {
    /// The image of `view` at `alpha`, one RGBA per cell, row-major; waited for.
    fn image(&self, view: View, alpha: f32) -> Vec<[u8; 4]> {
        let (device, queue) = (&self.gpu.device, &self.gpu.queue);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("views.staging"),
            size: (N * N * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("views") });
        let target = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer.render_cells(view, alpha, &target, &mut encoder);
        // 64 texels × 4 B = 256 B a row: already WebGPU's row alignment.
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
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

    fn cell(planes: &[f32], c: usize) -> Cell {
        let v = |k: usize| f64::from(planes[k * CELLS + c]);
        Cell { sigma: v(0), mom: [v(1), v(2)], energy: v(3), x: std::array::from_fn(|i| v(4 + i)) }
    }

    /// The state at `alpha` between the previous and the current one, channel by channel.
    fn blended(&self, c: usize, alpha: f64) -> Cell {
        let (a, b) = (Rig::cell(&self.prev_planes, c), Rig::cell(&self.cur_planes, c));
        let m = |p: f64, q: f64| p * (1.0 - alpha) + q * alpha;
        Cell { sigma: m(a.sigma, b.sigma), mom: [m(a.mom[0], b.mom[0]), m(a.mom[1], b.mom[1])], energy: m(a.energy, b.energy), x: std::array::from_fn(|i| m(a.x[i], b.x[i])) }
    }

    /// T in kelvins: μ ε_th floored at T_floor (§2.3), translated (§1.10.2).
    fn kelvin(&self, c: &Cell) -> f64 {
        let comp = self.elements.composition(&c.x);
        let u2 = (c.mom[0] * c.mom[0] + c.mom[1] * c.mom[1]) / (c.sigma * c.sigma);
        let eth = c.energy / c.sigma - 0.5 * u2 - self.eos.eps_cold(c.sigma, comp.y_e, c.x[9]);
        let t = (eth / comp.inv_mu).max(self.eos.t_floor());
        self.scale.to_kelvin(t)
    }

    /// §1.11's colour of a cell in `view`, 0…1; below Σ_vac every view draws bg.space (§4.11, M0-D113).
    fn colour(&self, view: View, c: &Cell) -> [f64; 3] {
        if c.sigma < self.physics.get("sigma_vac").unwrap() {
            return hex(BG_SPACE);
        }
        let l10_sigma = c.sigma.log10();
        match view {
            View::Glow => {
                let t_k = self.kelvin(c);
                let b = 1.0 - (-(t_k / 3000.0).powi(4)).exp();
                let table = blackbody_table();
                let bb = blackbody_lookup(&table, t_k);
                let haze = hex(HAZE);
                let h = ((l10_sigma + 8.0) / 8.0).clamp(0.0, 1.0) * 0.6;
                std::array::from_fn(|k| f64::from(bb[k]) * b + haze[k] * h * (1.0 - b))
            }
            View::Heat => map(&HEAT_STOPS, (self.kelvin(c).log10() - 1.0) / 9.0),
            View::Element => {
                let visible = ((l10_sigma + 6.0) / 8.0).clamp(0.15, 1.0);
                let mut sum = [0.0; 3];
                for (i, s) in self.elements.species().iter().enumerate() {
                    for k in 0..3 {
                        sum[k] += c.x[i] * lin(f64::from(s.color[k]) / 255.0);
                    }
                }
                sum.map(|v| enc(v * visible))
            }
            View::Density => map(&DENSITY_STOPS, (l10_sigma + 6.0) / 10.0),
        }
    }

    /// The largest channel deviation, in levels, of `image` from the formula over the cells `which`, and the cell.
    fn worst(&self, view: View, alpha: f64, image: &[[u8; 4]], which: impl Iterator<Item = usize>) -> (f64, usize) {
        let mut worst = (0.0, 0);
        for c in which {
            let want = self.colour(view, &self.blended(c, alpha));
            for k in 0..3 {
                let d = (f64::from(image[c][k]) - 255.0 * want[k]).abs();
                if d > worst.0 {
                    worst = (d, c);
                }
            }
        }
        worst
    }

    fn assert_matches(&self, view: View, alpha: f64, image: &[[u8; 4]], which: impl Iterator<Item = usize>) {
        let (d, c) = self.worst(view, alpha, image, which);
        let want = self.colour(view, &self.blended(c, alpha)).map(|v| 255.0 * v);
        assert!(d <= LEVEL, "{view:?} α={alpha}: cell {c} is off by {d:.3} levels: got {:?}, formula {want:.2?}", image[c]);
    }
}

fn idx(x: usize, y: usize) -> usize {
    y * N + x
}

/// A world of `CELLS` cells: Σ log-uniform in [1e-8, `sigma_hi`], T log-uniform in [`t_lo`, `t_hi`], |u| ≤ 0.3, mass fractions
/// over `species` random.
fn random_world(seed: u64, sigma_hi: f64, t_lo: f64, t_hi: f64, species: &[usize]) -> Vec<(usize, Spec)> {
    let mut rng = Lcg(seed);
    (0..CELLS)
        .map(|c| {
            let sigma = 1e-8 * (sigma_hi / 1e-8).powf(rng.next());
            let t = t_lo * (t_hi / t_lo).powf(rng.next());
            let mut x = [0.0; N_SPECIES];
            for &i in species {
                x[i] = rng.next();
            }
            if x.iter().sum::<f64>() == 0.0 {
                x[0] = 1.0;
            }
            (c, Spec { sigma, vel: [0.6 * rng.next() - 0.3, 0.6 * rng.next() - 0.3], t, x })
        })
        .collect()
}

/// T_sb that translates to `kelvin` (§1.10.2's inverse).
fn sb_of(rig: &Rig, kelvin: f64) -> f64 {
    rig.scale.to_sb(kelvin)
}

fn gentle() -> Vec<(usize, Spec)> {
    random_world(80, 0.25, 0.08, 30.0, &[0, 1, 2, 3, 8])
}

fn wide() -> Vec<(usize, Spec)> {
    random_world(81, 1e4, 0.1, 10.0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
}

#[test]
fn glow_matches_the_formula_across_a_world() {
    let rig = setup(&gentle(), &gentle());
    let image = rig.image(View::Glow, 1.0);
    assert!(image.iter().all(|p| p[3] == 255), "alpha is 1 everywhere");
    rig.assert_matches(View::Glow, 1.0, &image, 0..CELLS);
}

#[test]
fn heat_matches_the_formula_across_a_world() {
    let rig = setup(&gentle(), &gentle());
    rig.assert_matches(View::Heat, 1.0, &rig.image(View::Heat, 1.0), 0..CELLS);
}

#[test]
fn element_matches_the_formula_across_a_wide_world() {
    let rig = setup(&wide(), &wide());
    rig.assert_matches(View::Element, 1.0, &rig.image(View::Element, 1.0), 0..CELLS);
}

#[test]
fn density_matches_the_formula_across_a_wide_world() {
    let rig = setup(&wide(), &wide());
    rig.assert_matches(View::Density, 1.0, &rig.image(View::Density, 1.0), 0..CELLS);
}

/// Cells at the hex stops of §4.11: the heat map at T_K = 10^1, 10^2.8, … 10^10 reads the stops' own colours.
#[test]
fn heat_reads_the_stops_colours() {
    let probe = setup(&[], &[]);
    let cells: Vec<(usize, Spec)> = (0..6)
        .map(|i| {
            let kelvin = 10f64.powf(1.0 + 1.8 * i as f64);
            // T_K = 10 K is T_floor; the others carry the gas hot enough that Σ = 0.01's cold energy is negligible.
            (idx(3 + 2 * i, 5), gas(0.01, sb_of(&probe, kelvin)))
        })
        .collect();
    let rig = setup(&cells, &cells);
    let image = rig.image(View::Heat, 1.0);
    for (i, (c, _)) in cells.iter().enumerate() {
        let want = hex(HEAT_STOPS[i]);
        for k in 0..3 {
            assert!((f64::from(image[*c][k]) - 255.0 * want[k]).abs() <= LEVEL, "stop {i} ({}): got {:?}, stop {}", HEAT_STOPS[i], image[*c], HEAT_STOPS[i]);
        }
    }
}

/// Cells at the density stops, Σ = 10^−6 … 10^4: the stops' own colours.
#[test]
fn density_reads_the_stops_colours() {
    let cells: Vec<(usize, Spec)> = [-6.0, -3.5, -1.0, 1.5, 4.0].iter().enumerate().map(|(i, l)| (idx(4 + 2 * i, 9), gas(10f64.powf(*l), 1.0))).collect();
    let rig = setup(&cells, &cells);
    let image = rig.image(View::Density, 1.0);
    for (i, (c, _)) in cells.iter().enumerate() {
        let want = hex(DENSITY_STOPS[i]);
        for k in 0..3 {
            assert!((f64::from(image[*c][k]) - 255.0 * want[k]).abs() <= LEVEL, "stop {i} ({}): got {:?}", DENSITY_STOPS[i], image[*c]);
        }
    }
}

/// Between two stops the maps blend in linear RGB: the midpoint of each map is the linear-RGB mean of its two stops, and
/// that differs from the sRGB mean by more than a level — a map blended in sRGB passes at the stops and fails here.
#[test]
fn the_maps_midpoints_are_linear_rgb_means() {
    let probe = setup(&[], &[]);
    // Heat: log₁₀ T_K = 3.7, between stops 1 (2.8) and 2 (4.6); density: log₁₀ Σ = −4.75, between stops 0 and 1.
    let cells = [(idx(10, 10), gas(0.01, sb_of(&probe, 10f64.powf(3.7)))), (idx(12, 10), gas(10f64.powf(-4.75), 1.0))];
    let rig = setup(&cells, &cells);
    for (view, c, stops) in [(View::Heat, cells[0].0, &HEAT_STOPS[1..3]), (View::Density, cells[1].0, &DENSITY_STOPS[0..2])] {
        let image = rig.image(view, 1.0);
        let (a, b) = (hex(stops[0]), hex(stops[1]));
        let linear_mean: [f64; 3] = std::array::from_fn(|k| enc((lin(a[k]) + lin(b[k])) / 2.0));
        let srgb_mean: [f64; 3] = std::array::from_fn(|k| (a[k] + b[k]) / 2.0);
        let apart = (0..3).map(|k| 255.0 * (linear_mean[k] - srgb_mean[k]).abs()).fold(0.0, f64::max);
        assert!(apart > 2.0 * LEVEL, "{view:?}: the stops {stops:?} do not separate the two blends ({apart:.2} levels)");
        for k in 0..3 {
            assert!((f64::from(image[c][k]) - 255.0 * linear_mean[k]).abs() <= LEVEL, "{view:?} midpoint: got {:?}, linear mean {linear_mean:.4?}", image[c]);
        }
    }
    // Element: half hydrogen, half helium, Σ = 100 (visibility 1) — the linear mean of #7FB2FF and #FFE38A.
    let mix = Spec { sigma: 100.0, vel: [0.0; 2], t: 1.0, x: { let mut x = [0.0; N_SPECIES]; x[0] = 0.5; x[1] = 0.5; x } };
    let c = idx(14, 10);
    let rig = setup(&[(c, mix.clone())], &[(c, mix)]);
    let (a, b) = (hex("#7FB2FF"), hex("#FFE38A"));
    let linear_mean: [f64; 3] = std::array::from_fn(|k| enc((lin(a[k]) + lin(b[k])) / 2.0));
    let srgb_mean: [f64; 3] = std::array::from_fn(|k| (a[k] + b[k]) / 2.0);
    assert!((0..3).any(|k| 255.0 * (linear_mean[k] - srgb_mean[k]).abs() > 2.0 * LEVEL));
    let image = rig.image(View::Element, 1.0);
    for k in 0..3 {
        assert!((f64::from(image[c][k]) - 255.0 * linear_mean[k]).abs() <= LEVEL, "element midpoint: got {:?}, linear mean {linear_mean:.4?}", image[c]);
    }
}

/// Below Σ_vac (10⁻⁶) every view draws bg.space #05070D (§4.11, M0-D113): the floor state (Σ = 10⁻⁸), a cell between Σ_floor and
/// Σ_vac (10⁻⁷), and the cell just above Σ_vac reading its view's formula, not bg.space.
#[test]
fn vacuum_is_bg_space_in_every_view() {
    let bg = hex(BG_SPACE);
    let (floor, between, above) = (idx(7, 7), idx(8, 7), idx(9, 7));
    let rig = setup(&[(between, gas(1e-7, 1.0)), (above, gas(1.2e-6, 10.0))], &[]);
    let sigma_vac = rig.physics.get("sigma_vac").unwrap();
    assert!(1e-7 < sigma_vac && sigma_vac < 1.2e-6, "the cells straddle Σ_vac = {sigma_vac}");
    for view in [View::Glow, View::Heat, View::Element, View::Density] {
        let image = rig.image(view, 1.0);
        for (c, what) in [(floor, "floor"), (between, "Σ 10⁻⁷")] {
            for k in 0..3 {
                assert!((f64::from(image[c][k]) - 255.0 * bg[k]).abs() <= LEVEL, "{view:?} {what}: got {:?}, want bg.space {bg:.4?}", image[c]);
            }
            assert_eq!(image[c][3], 255);
        }
        rig.assert_matches(view, 1.0, &image, std::iter::once(above));
        let off = (0..3).map(|k| (f64::from(image[above][k]) - 255.0 * bg[k]).abs()).fold(0.0, f64::max);
        assert!(off > 3.0, "{view:?}: the cell just above Σ_vac reads {:?}, bg.space within {off:.1} levels", image[above]);
    }
}

/// Beyond the last anchor (T_Si ↔ 3.3 × 10⁹ K) the O–Si segment is extended: 10¹⁰ K reads the Heat map's last stop; between
/// the floor and the Sun's T_eff the first segment is the translation, not a clamp.
#[test]
fn temperatures_translate_past_the_anchors() {
    let probe = setup(&[], &[]);
    let top = sb_of(&probe, 1.0e10);
    assert!(top > probe.anchors[7], "1e10 K lies past T_Si");
    let cells = [(idx(20, 20), gas(0.01, top * 1.3)), (idx(22, 20), gas(0.01, 0.5 * (probe.anchors[0] + probe.anchors[1])))];
    let rig = setup(&cells, &cells);
    let image = rig.image(View::Heat, 1.0);
    // 1.3 × T past 10¹⁰ K clamps to the last stop; the mid-first-segment cell is against the formula.
    for k in 0..3 {
        assert!((f64::from(image[cells[0].0][k]) - 255.0 * hex(HEAT_STOPS[5])[k]).abs() <= LEVEL, "past 1e10 K: {:?}", image[cells[0].0]);
    }
    rig.assert_matches(View::Heat, 1.0, &image, [cells[1].0].into_iter());
    let kelvin = rig.kelvin(&Rig::cell(&rig.cur_planes, cells[1].0));
    assert!(kelvin > 10.0 && kelvin < 5772.0, "inside the first segment: {kelvin} K");
}

/// α = 1 is the current state and α = 0 the previous one, exactly: the images equal those of rigs holding that state as both.
#[test]
fn alpha_zero_and_one_give_the_two_states_exactly() {
    let (a, b) = (gentle(), random_world(82, 0.25, 0.08, 30.0, &[0, 1, 2, 3, 8]));
    let blend = setup(&a, &b);
    let only_current = setup(&a, &a);
    let only_previous = setup(&b, &b);
    for view in [View::Glow, View::Heat, View::Element, View::Density] {
        assert_eq!(blend.image(view, 1.0), only_current.image(view, 1.0), "{view:?} at α = 1 is the current state");
        assert_eq!(blend.image(view, 0.0), only_previous.image(view, 0.0), "{view:?} at α = 0 is the previous state");
        assert_ne!(blend.image(view, 0.0), blend.image(view, 1.0), "{view:?}: the two states differ");
    }
}

/// In between, the state is mixed channel by channel and the colour is the formula's of that state.
#[test]
fn alpha_between_blends_the_states() {
    // Previous: the current world with Σ 0.8–1.2 × and T 0.7–1.3 ×, so the mixed gas stays inside the gentle range.
    let cur = gentle();
    let mut rng = Lcg(83);
    let prev: Vec<(usize, Spec)> = cur
        .iter()
        .map(|(c, s)| (*c, Spec { sigma: s.sigma * (0.8 + 0.4 * rng.next()), t: s.t * (0.7 + 0.6 * rng.next()), ..s.clone() }))
        .collect();
    let rig = setup(&cur, &prev);
    for alpha in [0.25f32, 0.5] {
        for view in [View::Density, View::Element, View::Heat] {
            let image = rig.image(view, alpha);
            rig.assert_matches(view, f64::from(alpha), &image, 0..CELLS);
        }
    }
    assert_ne!(rig.image(View::Density, 0.5), rig.image(View::Density, 1.0), "α = ½ is not the current state");
    assert_ne!(rig.image(View::Density, 0.5), rig.image(View::Density, 0.0), "α = ½ is not the previous state");
}
