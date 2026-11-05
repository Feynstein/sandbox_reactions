//! Rendering (m0_contrat.md §1.11): the blackbody table the Glow view's colour is drawn from (M0-T79), the cell
//! image (M0-T80) and the glow (M0-T81).
//!
//! The table: Planck's B_λ integrated over 380–780 nm in 5 nm steps against Wyman, Sloan & Shirley's multi-lobe fit of the
//! CIE 1931 colour-matching functions, converted by the IEC 61966-2-1 XYZ → linear-sRGB matrix, clamped ≥ 0,
//! normalised so the largest channel is 1, then sRGB-encoded; 512 entries over log T from 1,000 K to 40,000 K.
//!
//! The image: [`Renderer::render`] records one compute pass (shaders/render.wgsl) writing the world's W × H RGBA8 image of
//! a [`View`] — Glow, Heat, Element or Density — into a storage texture, from the state blended with the previous state by
//! α (§1.8.3). Temperatures go through §1.10.2's translation to kelvins, the same eight anchors that build the readouts'
//! `TempScale`.
//!
//! The display settings ([`Display`], M0-T126; §1.11 [M0-R12]): the Glow setting S — a temperature the Glow view's brightness is
//! dimmed against, a 1,024-entry table over log₁₀ T from log₁₀ 3,000 to 11 built on the CPU when S changes — and the Heat and
//! Density views' ranges, two uniforms. At the default they are today's formulas, byte for byte.
//!
//! Fit to view ([`Renderer::fit_range`], M0-T127a): one compute pass bins the Heat view's log₁₀ T_K or the Density view's log₁₀ Σ of the
//! cells with Σ ≥ Σ_vac into 1,024 bins by atomics, read back once; [`extent_from_bins`] turns the bins into the range.
//!
//! The glow: for the Glow view, [`Renderer::render`] draws the cell image into an image of its own, then runs shaders/glow.wgsl's
//! passes over it — a bright pass on the brightest channel > 0.8 at half resolution, a separable 9-tap Gaussian (σ = 3 texels), the result
//! upsampled and added at 0.6 — into the caller's target. The other views never go through them.

use sr_physics::observe::translate::TEMP_KELVIN;
use sr_physics::registry::Elements;

use crate::state::State;
use crate::step::{compute_pipeline, init_buffer, with_eos, EosGpu};

/// Planck's constant, the speed of light and Boltzmann's constant — the exact SI values (2019).
const H: f64 = 6.626_070_15e-34;
const C: f64 = 299_792_458.0;
const K_B: f64 = 1.380_649e-23;

/// c₂ = hc/k_B in nm·K (§1.11: 1.438777 × 10⁷, from the exact SI constants).
pub const C2_NM_K: f64 = H * C / K_B * 1.0e9;

pub const TABLE_LEN: usize = 512;
pub const T_MIN: f64 = 1_000.0;
pub const T_MAX: f64 = 40_000.0;

const LAMBDA_MIN: f64 = 380.0;
const LAMBDA_MAX: f64 = 780.0;
const LAMBDA_STEP: f64 = 5.0;

/// IEC 61966-2-1 XYZ → linear sRGB (§1.11).
const XYZ_TO_RGB: [[f64; 3]; 3] = [
    [3.2406, -1.5372, -0.4986],
    [-0.9689, 1.8758, 0.0415],
    [0.0557, -0.2040, 1.0570],
];

/// One lobe of Wyman's fit: α · exp(−½[(λ − β) · (λ < β ? γ : δ)]²), Table 1's (α, β, γ, δ).
type Lobe = (f64, f64, f64, f64);

const X_LOBES: [Lobe; 3] = [
    (0.362, 442.0, 0.0624, 0.0374),
    (1.056, 599.8, 0.0264, 0.0323),
    (-0.065, 501.1, 0.0490, 0.0382),
];
const Y_LOBES: [Lobe; 2] = [(0.821, 568.8, 0.0213, 0.0247), (0.286, 530.9, 0.0613, 0.0322)];
const Z_LOBES: [Lobe; 2] = [(1.217, 437.0, 0.0845, 0.0278), (0.681, 459.0, 0.0385, 0.0725)];

fn lobes(set: &[Lobe], lambda: f64) -> f64 {
    set.iter()
        .map(|&(a, b, g, d)| {
            let t = (lambda - b) * if lambda < b { g } else { d };
            a * (-0.5 * t * t).exp()
        })
        .sum()
}

/// The CIE 1931 colour-matching functions (x̄, ȳ, z̄) at `lambda` nm, Wyman's multi-lobe fit.
pub fn cie_fit(lambda: f64) -> [f64; 3] {
    [lobes(&X_LOBES, lambda), lobes(&Y_LOBES, lambda), lobes(&Z_LOBES, lambda)]
}

/// Planck's B_λ up to the constant 2hc², which the normalisation cancels: λ⁻⁵ / (e^{c₂/(λT)} − 1), λ in nm.
fn planck(lambda: f64, t: f64) -> f64 {
    lambda.powi(-5) / (C2_NM_K / (lambda * t)).exp_m1()
}

fn srgb_encode(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// The blackbody's colour at `t` kelvin: sRGB-encoded, largest channel 1 (computed, not looked up).
pub fn blackbody_srgb(t: f64) -> [f64; 3] {
    let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
    let steps = ((LAMBDA_MAX - LAMBDA_MIN) / LAMBDA_STEP).round() as usize;
    for i in 0..=steps {
        let lambda = LAMBDA_MIN + LAMBDA_STEP * i as f64;
        let b = planck(lambda, t);
        let [xb, yb, zb] = cie_fit(lambda);
        x += b * xb;
        y += b * yb;
        z += b * zb;
    }
    let lin = XYZ_TO_RGB.map(|r| (r[0] * x + r[1] * y + r[2] * z).max(0.0));
    let m = lin[0].max(lin[1]).max(lin[2]);
    lin.map(|c| srgb_encode(c / m))
}

/// The temperature of table entry `i`: log-spaced from `T_MIN` to `T_MAX`.
pub fn table_temperature(i: usize) -> f64 {
    T_MIN * (T_MAX / T_MIN).powf(i as f64 / (TABLE_LEN - 1) as f64)
}

/// The 512-entry table, built once on the CPU.
pub fn blackbody_table() -> Vec<[f32; 3]> {
    (0..TABLE_LEN).map(|i| blackbody_srgb(table_temperature(i)).map(|c| c as f32)).collect()
}

/// A table lookup: linear in log T between entries; outside 1,000–40,000 K, the end colours.
pub fn blackbody_lookup(table: &[[f32; 3]], t: f64) -> [f32; 3] {
    let n = table.len();
    let u = ((t.clamp(T_MIN, T_MAX) / T_MIN).ln() / (T_MAX / T_MIN).ln()) * (n - 1) as f64;
    let i = (u.floor() as usize).min(n - 2);
    let f = (u - i as f64) as f32;
    [0, 1, 2].map(|k| table[i][k] * (1.0 - f) + table[i + 1][k] * f)
}

/// The Glow setting S's domain (§1.11 [M0-R12]) and its default: today's 3,000 K.
pub const GLOW_S_MIN: f64 = 10.0;
pub const GLOW_S_MAX: f64 = 1.0e10;
pub const GLOW_S_DEFAULT: f64 = 3_000.0;
/// The Heat view's span, log₁₀ T_K, and the Density view's, log₁₀ Σ: the ranges' bounds and their defaults.
pub const HEAT_SPAN: [f64; 2] = [1.0, 10.0];
pub const DENSITY_SPAN: [f64; 2] = [-6.0, 4.0];

/// «Fit to view»'s histogram: its bin count, the percentile its low end reads and the narrowest range it hands back (§1.11 [M0-R12]).
pub const EXTENT_BINS: usize = 1024;
const EXTENT_PERCENTILE_PER_MILLE: u64 = 5;
const EXTENT_MIN_WIDTH: f64 = 0.1;

/// The range «Fit to view» sets from a histogram of the quantity's `EXTENT_BINS` equal bins over `span` (§1.11 [M0-R12]): L the
/// lower edge of the bin holding the 0.5th percentile (the ⌈0.005 n⌉-th cell counted from below), H the upper edge of the highest
/// non-empty bin, widened about their middle to 0.1 if narrower, and kept inside the span — the 0.1 window slid in, never
/// shortened, so H − L ≥ 0.1 holds at the span's ends. `None` when the bins hold no cell.
pub fn extent_from_bins(bins: &[u32], span: [f64; 2]) -> Option<[f64; 2]> {
    let n: u64 = bins.iter().map(|&b| u64::from(b)).sum();
    if n == 0 {
        return None;
    }
    let rank = (n * EXTENT_PERCENTILE_PER_MILLE).div_ceil(1000).max(1);
    let mut cumulative = 0;
    let low = bins
        .iter()
        .position(|&b| {
            cumulative += u64::from(b);
            cumulative >= rank
        })
        .expect("the bins hold n cells");
    let high = bins.iter().rposition(|&b| b > 0).expect("the bins hold n cells");
    let width = (span[1] - span[0]) / bins.len() as f64;
    let (mut lo, mut hi) = (span[0] + width * low as f64, span[0] + width * (high + 1) as f64);
    if hi - lo < EXTENT_MIN_WIDTH {
        let mid = 0.5 * (lo + hi);
        (lo, hi) = (mid - 0.5 * EXTENT_MIN_WIDTH, mid + 0.5 * EXTENT_MIN_WIDTH);
    }
    if lo < span[0] {
        (lo, hi) = (span[0], span[0] + EXTENT_MIN_WIDTH.max(hi - lo));
    }
    if hi > span[1] {
        (lo, hi) = (span[1] - EXTENT_MIN_WIDTH.max(hi - lo), span[1]);
    }
    Some([lo.max(span[0]), hi.min(span[1])])
}

pub const DIMMING_LEN: usize = 1024;
/// The dimming table's span, log₁₀ T_K: from log₁₀ 3,000 to 11.
const DIMMING_L10_LO: f64 = 3.477_121_254_719_662;
const DIMMING_L10_HI: f64 = 11.0;
/// β of §1.11 [M0-R12]'s asinh stretch: about four decades of luminance.
const DIMMING_BETA: f64 = 1.0e-4;

/// The display settings (§1.11 [M0-R12]): the Glow setting S in kelvins, and the Heat and Density views' ranges as log₁₀ ends
/// (log₁₀ T_K and log₁₀ Σ), each inside its span, `[lo, hi]` with `hi − lo ≥ 0.1` — what the caller keeps in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Display {
    pub glow_s: f64,
    pub heat: [f64; 2],
    pub density: [f64; 2],
}

impl Default for Display {
    /// Today's look: 3,000 K, the whole spans.
    fn default() -> Display {
        Display { glow_s: GLOW_S_DEFAULT, heat: HEAT_SPAN, density: DENSITY_SPAN }
    }
}

/// The blackbody's visible luminance Y(T): Planck × ȳ summed over 380–780 nm in 5 nm steps — the table's own sum, unnormalised.
pub fn visible_luminance(t: f64) -> f64 {
    let steps = ((LAMBDA_MAX - LAMBDA_MIN) / LAMBDA_STEP).round() as usize;
    (0..=steps)
        .map(|i| {
            let lambda = LAMBDA_MIN + LAMBDA_STEP * i as f64;
            planck(lambda, t) * lobes(&Y_LOBES, lambda)
        })
        .sum()
}

/// The Glow view's dimming D(T; S) (§1.11 [M0-R12]): 1 for S ≤ 3,000 K, else clamp(asinh(x/β)/asinh(1/β), 0, 1) with
/// x = Y(max(T, 3,000 K)) / Y(S).
pub fn dimming(t: f64, s: f64) -> f64 {
    if s <= GLOW_S_DEFAULT {
        return 1.0;
    }
    let x = visible_luminance(t.max(GLOW_S_DEFAULT)) / visible_luminance(s);
    ((x / DIMMING_BETA).asinh() / (1.0 / DIMMING_BETA).asinh()).clamp(0.0, 1.0)
}

/// The temperature of dimming-table entry `i`: log₁₀ T linear from log₁₀ 3,000 to 11.
pub fn dimming_temperature(i: usize) -> f64 {
    10f64.powf(DIMMING_L10_LO + (DIMMING_L10_HI - DIMMING_L10_LO) * i as f64 / (DIMMING_LEN - 1) as f64)
}

/// The 1,024-entry dimming table for the setting `s`: every entry 1 at S ≤ 3,000 K.
pub fn dimming_table(s: f64) -> Vec<f32> {
    (0..DIMMING_LEN).map(|i| dimming(dimming_temperature(i), s) as f32).collect()
}

/// A dimming-table lookup: linear in log T between entries, clamped to the ends (render.wgsl `dimming`).
pub fn dimming_lookup(table: &[f32], t: f64) -> f32 {
    let n = table.len();
    let u = ((t.log10() - DIMMING_L10_LO) / (DIMMING_L10_HI - DIMMING_L10_LO)).clamp(0.0, 1.0) * (n - 1) as f64;
    let i = (u.floor() as usize).min(n - 2);
    table[i] + (table[i + 1] - table[i]) * (u - i as f64) as f32
}

/// The Glow view's emissivity scale (§1.11 [M0-D157]): ε = 1 − exp(−τ), τ = EMISSIVITY_SCALE · κ₀(1 + X_H) Σ. Measured
/// against 1, 3, 30 and 100 on the Sun-like and Massive presets (RTX 4090; tasks/M0-D157.md): 10 draws the body as round as 3
/// (axis/diagonal 0.980 against 0.982 at the Sun-like step 300) and keeps a painted stroke visible — the lead's pick.
pub const EMISSIVITY_SCALE: f64 = 10.0;

/// render.wgsl's `DisplayParams` as words: S clamped to its domain and to 3,000 K above (the exposure curve's own scale) and
/// the emissivity's coefficient EMISSIVITY_SCALE · κ₀ (§1.11 [M0-D157]), the two ranges, then the dimming table.
fn display_words(display: &Display, kappa0: f64, dimming: &[f32]) -> Vec<u32> {
    let s_eff = display.glow_s.min(GLOW_S_DEFAULT);
    let mut words: Vec<u32> = [s_eff, EMISSIVITY_SCALE * kappa0, 0.0, 0.0].map(|v| (v as f32).to_bits()).to_vec();
    words.extend([display.heat[0], display.heat[1], display.density[0], display.density[1]].map(|v| (v as f32).to_bits()));
    words.extend(dimming.iter().map(|v| v.to_bits()));
    words
}

/// The four views of the cell image (§1.11); the discriminant is the shader's `view` word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Glow = 0,
    Heat = 1,
    Element = 2,
    Density = 3,
}

/// shaders/render.wgsl, the cell image's pass.
const RENDER_WGSL: &str = include_str!("../shaders/render.wgsl");
/// The pass's tile edge (render.wgsl `@workgroup_size(8, 8)`).
const TILE: u32 = 8;
const N_ANCHORS: usize = 8;
const N_SPECIES: usize = 10;

/// shaders/glow.wgsl, the glow's four passes.
const GLOW_WGSL: &str = include_str!("../shaders/glow.wgsl");
/// The format of the glow's half-resolution textures: a lone cell's halo is under one level of an 8-bit image.
const HALF_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The format of the cell image's target: a storage texture of RGBA8, sRGB-encoded values (the display draws it as is).
pub const IMAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn entry(binding: u32, resource: wgpu::BindingResource<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource }
}

fn tex(v: &wgpu::TextureView) -> wgpu::BindingResource<'_> {
    wgpu::BindingResource::TextureView(v)
}

fn image_texture(device: &wgpu::Device, label: &str, format: wgpu::TextureFormat, w: u32, h: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

/// The glow's passes (shaders/glow.wgsl) and the textures between them: the Glow view's cell image, and two half-resolution
/// textures the bright pass and the two blurs ping-pong between (`half_b` lives in the bind groups).
struct Glow {
    cells: wgpu::TextureView,
    half_a: wgpu::TextureView,
    bright: (wgpu::ComputePipeline, wgpu::BindGroup),
    blur_h: (wgpu::ComputePipeline, wgpu::BindGroup),
    blur_v: (wgpu::ComputePipeline, wgpu::BindGroup),
    composite: wgpu::ComputePipeline,
}

impl Glow {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Glow {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("glow.wgsl"),
            source: wgpu::ShaderSource::Wgsl(GLOW_WGSL.into()),
        });
        let (hw, hh) = (width.div_ceil(2), height.div_ceil(2));
        let cells = image_texture(device, "glow.cells", IMAGE_FORMAT, width, height);
        let half_a = image_texture(device, "glow.half_a", HALF_FORMAT, hw, hh);
        let half_b = image_texture(device, "glow.half_b", HALF_FORMAT, hw, hh);
        let pass = |entry_point: &'static str, entries: &[wgpu::BindGroupEntry<'_>]| {
            let pipeline = compute_pipeline(device, &module, entry_point);
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(entry_point),
                layout: &pipeline.get_bind_group_layout(0),
                entries,
            });
            (pipeline, group)
        };
        Glow {
            bright: pass("glow_bright", &[entry(0, tex(&cells)), entry(2, tex(&half_a))]),
            blur_h: pass("glow_blur_h", &[entry(1, tex(&half_a)), entry(2, tex(&half_b))]),
            blur_v: pass("glow_blur_v", &[entry(1, tex(&half_b)), entry(2, tex(&half_a))]),
            composite: compute_pipeline(device, &module, "glow_composite"),
            cells,
            half_a,
        }
    }

    /// Records the four passes: the cell image in `self.cells`, the glow added into `target`.
    fn encode(&self, device: &wgpu::Device, target: &wgpu::TextureView, width: u32, height: u32, encoder: &mut wgpu::CommandEncoder) {
        let (hw, hh) = (width.div_ceil(2), height.div_ceil(2));
        for (label, (pipeline, group)) in [("glow.bright", &self.bright), ("glow.blur_h", &self.blur_h), ("glow.blur_v", &self.blur_v)] {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(label), timestamp_writes: None });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(hw.div_ceil(TILE), hh.div_ceil(TILE), 1);
        }
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glow.composite"),
            layout: &self.composite.get_bind_group_layout(0),
            entries: &[
                entry(0, wgpu::BindingResource::TextureView(&self.cells)),
                entry(1, wgpu::BindingResource::TextureView(&self.half_a)),
                entry(3, wgpu::BindingResource::TextureView(target)),
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("glow.composite"), timestamp_writes: None });
        pass.set_pipeline(&self.composite);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(width.div_ceil(TILE), height.div_ceil(TILE), 1);
    }
}

/// The cell image's pass, built once for a world: the pipeline, the constants of the run (§1.10.2's anchors, the species
/// colours, the blackbody table) and the two states it reads.
pub struct Renderer {
    device: wgpu::Device,
    pipeline: wgpu::ComputePipeline,
    eos: wgpu::BindGroup,
    state: [wgpu::Buffer; 3],
    previous: [wgpu::Buffer; 3],
    dims: wgpu::Buffer,
    palette: wgpu::Buffer,
    display: Display,
    kappa0: f64,
    dimming: Vec<f32>,
    display_buf: wgpu::Buffer,
    extent: (wgpu::ComputePipeline, wgpu::BindGroup),
    glow: Glow,
    width: u32,
    height: u32,
}

impl Renderer {
    /// For `state`'s world, blending with `previous` (the same world; the last state before `state`'s). `anchors_sb` are
    /// §1.10.2's eight sandbox temperatures — T_floor, the Sun-like T_eff, T_H, T_He, T_C, T_Ne, T_O, T_Si — paired with
    /// [`TEMP_KELVIN`]; the species colours come from `elements`; the Glow view's emissivity reads `eos`'s κ₀ (§1.11 [M0-D157]).
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        previous: &State,
        eos: &EosGpu,
        anchors_sb: [f64; N_ANCHORS],
        elements: &Elements,
    ) -> Renderer {
        assert_eq!(state.world(), previous.world(), "the renderer blends two states of one world");
        let world = state.world();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(RENDER_WGSL).into()),
        });
        let pipeline = compute_pipeline(device, &module, "render_cells");
        let extent_pipeline = compute_pipeline(device, &module, "extent_bins");
        let kappa0 = eos.kappa0();
        let extent_eos = eos.bind_group(device, &extent_pipeline);
        let eos = eos.bind_group(device, &pipeline);

        // The palette uniform (render.wgsl `Palette`): ln T_sb, ln T_K, slopes, the species in linear RGB, the table.
        let ln_t: Vec<f32> = anchors_sb.iter().map(|t| t.ln() as f32).collect();
        let ln_k: Vec<f32> = TEMP_KELVIN.iter().map(|k| k.ln() as f32).collect();
        let slope: Vec<f32> = (0..N_ANCHORS)
            .map(|i| {
                let j = i.min(N_ANCHORS - 2);
                ((TEMP_KELVIN[j + 1].ln() - TEMP_KELVIN[j].ln()) / (anchors_sb[j + 1].ln() - anchors_sb[j].ln())) as f32
            })
            .collect();
        let mut words: Vec<u32> = [ln_t, ln_k, slope].iter().flatten().map(|v| v.to_bits()).collect();
        let species = elements.species();
        assert!(species.len() >= N_SPECIES, "render.wgsl holds {N_SPECIES} species colours");
        for s in species.iter().take(N_SPECIES) {
            words.extend(s.color.map(|c| (srgb_to_linear(f64::from(c) / 255.0) as f32).to_bits()));
            words.push(0);
        }
        for c in blackbody_table() {
            words.extend(c.map(f32::to_bits));
            words.push(0);
        }
        let palette = init_buffer(device, "render.palette", wgpu::BufferUsages::UNIFORM, &words);
        let display = Display::default();
        let dimming = dimming_table(display.glow_s);
        let display_buf =
            init_buffer(device, "render.display", wgpu::BufferUsages::UNIFORM, &display_words(&display, kappa0, &dimming));
        let group = |s: &State| std::array::from_fn(|g| s.group(g).clone());
        Renderer {
            device: device.clone(),
            pipeline,
            eos,
            state: group(state),
            previous: group(previous),
            dims: state.dims().clone(),
            palette,
            display,
            kappa0,
            dimming,
            display_buf,
            extent: (extent_pipeline, extent_eos),
            glow: Glow::new(device, world.width, world.height),
            width: world.width,
            height: world.height,
        }
    }

    /// Sets the display settings (§1.11 [M0-R12]); drawn from the next [`Renderer::render`] on. S is clamped to its domain; the
    /// dimming table is rebuilt only when S changed, the ranges being two uniforms.
    pub fn set_display(&mut self, display: Display) {
        let display = Display { glow_s: display.glow_s.clamp(GLOW_S_MIN, GLOW_S_MAX), ..display };
        if display.glow_s != self.display.glow_s {
            self.dimming = dimming_table(display.glow_s);
        }
        self.display_buf = init_buffer(&self.device, "render.display", wgpu::BufferUsages::UNIFORM, &display_words(&display, self.kappa0, &self.dimming));
        self.display = display;
    }

    /// Records the image of `view` into `target` (a view of an [`IMAGE_FORMAT`] storage texture of the world's size): the
    /// state at `alpha` of the way from the previous state to the current one — 0 the previous exactly, 1 the current
    /// exactly, in between the channels mixed (§1.8.3's slow motion draws α = owed / Δt_est; outside [0, 1] it is clamped).
    /// The Glow view carries the glow (§1.11, shaders/glow.wgsl); the other views are the cell image alone.
    pub fn render(&self, view: View, alpha: f32, target: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        if view == View::Glow {
            self.cell_image(view, alpha, &self.glow.cells, encoder);
            self.glow.encode(&self.device, target, self.width, self.height, encoder);
        } else {
            self.cell_image(view, alpha, target, encoder);
        }
    }

    /// The cell image of `view` alone, glow-less even for [`View::Glow`] — what [`Renderer::render`] draws for the other
    /// views, and what the glow is added to (`target` as for `render`).
    pub fn render_cells(&self, view: View, alpha: f32, target: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        self.cell_image(view, alpha, target, encoder);
    }

    /// The range «Fit to view» sets for `view` (§1.11 [M0-R12]): log₁₀ T_K ends for Heat, log₁₀ Σ ends for Density, from the
    /// histogram of the current state's cells with Σ ≥ Σ_vac (a cell past the span counts in its end bin) — one pass, read back
    /// once and waited for. `None` for Glow and Element, which have no range, and for a world with no matter.
    pub fn fit_range(&self, queue: &wgpu::Queue, view: View) -> Option<[f64; 2]> {
        let span = match view {
            View::Heat => HEAT_SPAN,
            View::Density => DENSITY_SPAN,
            View::Glow | View::Element => return None,
        };
        let params = init_buffer(
            &self.device,
            "render.extent_params",
            wgpu::BufferUsages::UNIFORM,
            &[view as u32, 0, (span[0] as f32).to_bits(), ((span[1] - span[0]) as f32).to_bits()],
        );
        let bins = init_buffer(
            &self.device,
            "render.extent_bins",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            &[0; EXTENT_BINS],
        );
        let bytes = (EXTENT_BINS * 4) as u64;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render.extent_staging"),
            size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (pipeline, eos) = &self.extent;
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render.extent"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                entry(0, self.dims.as_entire_binding()),
                entry(1, self.state[0].as_entire_binding()),
                entry(2, self.state[1].as_entire_binding()),
                entry(3, self.state[2].as_entire_binding()),
                entry(4, self.previous[0].as_entire_binding()),
                entry(5, self.previous[1].as_entire_binding()),
                entry(6, self.previous[2].as_entire_binding()),
                entry(7, params.as_entire_binding()),
                entry(8, self.palette.as_entire_binding()),
                entry(11, bins.as_entire_binding()),
            ],
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render.extent") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("render.extent"), timestamp_writes: None });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_bind_group(1, eos, &[]);
            pass.dispatch_workgroups(self.width.div_ceil(TILE), self.height.div_ceil(TILE), 1);
        }
        encoder.copy_buffer_to_buffer(&bins, 0, &staging, 0, bytes);
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.map_async(wgpu::MapMode::Read, .., move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::wait_indefinitely()).expect("the device is lost");
        rx.recv().expect("the map callback ran").expect("the histogram maps");
        let counts: Vec<u32> = {
            let mapped = staging.get_mapped_range(..).expect("mapped");
            mapped.as_chunks::<4>().0.iter().map(|b| u32::from_le_bytes(*b)).collect()
        };
        staging.unmap();
        extent_from_bins(&counts, span)
    }

    fn cell_image(&self, view: View, alpha: f32, target: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let params = init_buffer(
            &self.device,
            "render.params",
            wgpu::BufferUsages::UNIFORM,
            &[view as u32, alpha.clamp(0.0, 1.0).to_bits(), 0, 0],
        );
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                entry(0, self.dims.as_entire_binding()),
                entry(1, self.state[0].as_entire_binding()),
                entry(2, self.state[1].as_entire_binding()),
                entry(3, self.state[2].as_entire_binding()),
                entry(4, self.previous[0].as_entire_binding()),
                entry(5, self.previous[1].as_entire_binding()),
                entry(6, self.previous[2].as_entire_binding()),
                entry(7, params.as_entire_binding()),
                entry(8, self.palette.as_entire_binding()),
                entry(9, wgpu::BindingResource::TextureView(target)),
                entry(10, self.display_buf.as_entire_binding()),
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("render"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_bind_group(1, &self.eos, &[]);
        pass.dispatch_workgroups(self.width.div_ceil(TILE), self.height.div_ceil(TILE), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs()
    }

    #[test]
    fn c2_from_the_exact_si_constants() {
        assert!(rel(C2_NM_K, 1.438777e7) < 1e-6, "c2 = {C2_NM_K}");
    }

    #[test]
    fn table_has_512_entries_in_range_with_the_largest_channel_one() {
        let t = blackbody_table();
        assert_eq!(t.len(), 512);
        assert!((table_temperature(0) - 1_000.0).abs() < 1e-9);
        assert!((table_temperature(511) - 40_000.0).abs() < 1e-6);
        for (i, c) in t.iter().enumerate() {
            assert!(c.iter().all(|v| (0.0..=1.0).contains(v)), "entry {i}: {c:?}");
            let m = c[0].max(c[1]).max(c[2]);
            assert!((m - 1.0).abs() < 1e-6, "entry {i}: largest channel {m}");
        }
    }

    #[test]
    fn blue_to_red_ratio_rises_with_temperature() {
        let t = blackbody_table();
        let ratio = |c: &[f32; 3]| c[2] as f64 / (c[0] as f64).max(1e-12);
        for w in t.windows(2) {
            assert!(ratio(&w[1]) >= ratio(&w[0]), "{:?} then {:?}", w[0], w[1]);
        }
        assert!(ratio(&t[511]) > 10.0 * ratio(&t[0]).max(1e-3));
    }

    #[test]
    fn six_thousand_five_hundred_kelvin_is_near_white() {
        let c = blackbody_srgb(6_500.0);
        assert!(c.iter().all(|&v| v >= 0.9), "6,500 K = {c:?}");
    }

    #[test]
    fn one_thousand_kelvin_is_red_dominant() {
        let c = blackbody_srgb(1_000.0);
        assert!(c[0] > c[1] && c[1] > c[2], "1,000 K = {c:?}");
        assert!(c[2] < 0.5 * c[0], "1,000 K = {c:?}");
    }

    #[test]
    fn forty_thousand_kelvin_is_blue_dominant() {
        let c = blackbody_srgb(40_000.0);
        assert!(c[2] > c[1] && c[1] > c[0], "40,000 K = {c:?}");
        assert!(c[2] > 1.2 * c[0], "40,000 K = {c:?}");
    }

    /// The fit against the CIE 1931 2° observer's table (cvrl.ioo.ucl.ac.uk's ciexyz31_1 values, read through
    /// colour-science's copy of it, 2026-10-29) within 5 % of each value of 0.05 or more. Below that the fit's tails
    /// (ȳ at 440 nm: 0.0191 against 0.0230, z̄ at 555 and 600 nm: 0.0057 and 0.0008) are held to 0.005 absolute —
    /// half a percent of the peak: Wyman's lobes, verbatim, do not claim 5 % of a tail's value.
    #[test]
    fn fit_matches_the_cie_1931_values() {
        let cie = [
            (440.0, [0.348_28, 0.023_0, 1.747_06]),
            (555.0, [0.512_050_1, 1.0, 0.005_749_999]),
            (600.0, [1.062_2, 0.631_0, 0.000_8]),
        ];
        for (lambda, want) in cie {
            let got = cie_fit(lambda);
            for k in 0..3 {
                let ok = if want[k] < 0.05 { (got[k] - want[k]).abs() < 0.005 } else { rel(got[k], want[k]) < 0.05 };
                assert!(ok, "λ = {lambda} nm, channel {k}: fit {} vs CIE {}", got[k], want[k]);
            }
        }
    }

    #[test]
    fn lookup_clamps_to_the_end_colours_and_follows_the_table() {
        let t = blackbody_table();
        assert_eq!(blackbody_lookup(&t, 10.0), t[0]);
        assert_eq!(blackbody_lookup(&t, 1.0e6), t[511]);
        let mid = blackbody_lookup(&t, table_temperature(200));
        for k in 0..3 {
            assert!((mid[k] - t[200][k]).abs() < 1e-5);
        }
        let direct = blackbody_srgb(5_770.0);
        let looked = blackbody_lookup(&t, 5_770.0);
        for k in 0..3 {
            assert!((looked[k] as f64 - direct[k]).abs() < 0.01, "{looked:?} vs {direct:?}");
        }
    }

    #[test]
    fn dimming_is_one_everywhere_at_or_under_3000_kelvin() {
        for s in [10.0, 1_000.0, 3_000.0] {
            assert!(dimming_table(s).iter().all(|&d| d == 1.0), "S = {s}");
        }
    }

    #[test]
    fn dimming_rises_with_temperature_and_reaches_one_at_s() {
        for s in [1.0e5, 1.0e7, 1.0e9] {
            let t = dimming_table(s);
            assert!(t.windows(2).all(|w| w[1] >= w[0]), "S = {s}: not monotone");
            assert!(t[0] < 0.5 && t[t.len() - 1] == 1.0, "S = {s}: ends {} … {}", t[0], t[t.len() - 1]);
            assert!(dimming(s, s) > 0.99 && dimming(s / 1.0e4, s) < 0.5, "S = {s}");
        }
    }

    #[test]
    fn dimming_lookup_follows_the_table_and_clamps() {
        let t = dimming_table(1.0e7);
        assert_eq!(dimming_lookup(&t, 10.0), t[0]);
        assert_eq!(dimming_lookup(&t, 1.0e12), t[DIMMING_LEN - 1]);
        assert!((dimming_lookup(&t, dimming_temperature(500)) - t[500]).abs() < 1e-5);
        for k in [3.0e3, 2.0e5, 4.0e6, 8.0e6, 3.0e7] {
            assert!((f64::from(dimming_lookup(&t, k)) - dimming(k, 1.0e7)).abs() < 2e-3, "T = {k}");
        }
    }
}
