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
//! The glow: for the Glow view, [`Renderer::render`] draws the cell image into an image of its own, then runs shaders/glow.wgsl's
//! passes over it — a bright pass on luminance > 0.8 at half resolution, a separable 9-tap Gaussian (σ = 3 texels), the result
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
    glow: Glow,
    width: u32,
    height: u32,
}

impl Renderer {
    /// For `state`'s world, blending with `previous` (the same world; the last state before `state`'s). `anchors_sb` are
    /// §1.10.2's eight sandbox temperatures — T_floor, the Sun-like T_eff, T_H, T_He, T_C, T_Ne, T_O, T_Si — paired with
    /// [`TEMP_KELVIN`]; the species colours come from `elements`.
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
        let group = |s: &State| std::array::from_fn(|g| s.group(g).clone());
        Renderer {
            device: device.clone(),
            pipeline,
            eos,
            state: group(state),
            previous: group(previous),
            dims: state.dims().clone(),
            palette,
            glow: Glow::new(device, world.width, world.height),
            width: world.width,
            height: world.height,
        }
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
}
