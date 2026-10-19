//! P2 gravity (m0_contrat.md §1.3.2, §1.4.1–§1.4.2), on shaders/grav.wgsl appended to shaders/fft.wgsl: φ = K ∗ Σ_src
//! over the active box by M0-T30's zero-padded convolution, then g = −∇φ by central differences — the scheme of the CPU
//! twin, sr_physics::reference::grav (M0-T29), whose kernel this samples. Central differences, not force kernels: the
//! twin measured G-GRAV1 at 0.30 % (limit 2 %).
//!
//! The box's FFT size lives only on the GPU (§1.3.3, M0-T21), so the kernel's transform is held for every padded size the
//! world's boxes can take — 32 … padded(width) by 32 … padded(height), 42 sizes at 600 × 400 — made once per step, on
//! the CPU in f64 (sr_physics::reference::fft) and stored in f32 ([`spectra`]). K is real and even in both axes, so K̂ is
//! too: each size keeps its real quarter, (nx/2 + 1) × (ny/2 + 1) values — 8.3 MB at 600 × 400 where whole complex
//! spectra would take 65 MB — and `grav_columns` reads the quarter of the size the box names.
//!
//! Four dispatches — five in a step that runs P3 — each a fixed size or the box's tiles, so a frame's latch reaches them
//! (latch.rs):
//! - `grav_rows`: the box's rows of Σ, vacuum (Σ < Σ_vac) as 0, zero-padded and forward — one workgroup per world row;
//! - `grav_columns`: each column forward, times K̂ ÷ (nx · ny), back — one per column of the world's largest padded size;
//! - `conv_rows_inverse` (fft.wgsl's): the rows back, φ into the box's cells of `side.phi`;
//! - `grav_gradient`: g = −∇φ over the box's 8 × 8 tiles, one-sided on the box's outermost ring as the twin;
//! - `grav_sinks` (a step with P3 only, M0-T52a): the sinks' softened potentials (§1.4.3) added to φ in place and their
//!   exact pull added to g — into a second field, so [`Gravity::gas_field`] stays the gas's alone for P3's kick-drift-kick
//!   (a g carrying a sink's own pull would pull the sink on itself through the bilinear read) while [`Gravity::field`],
//!   what P4 and Δt read, carries every pull. φ is the gas's plus the sinks'.
//!
//! Workgroups past the box return at once. φ and g outside the box keep their last values: no pass reads them there.
//! P9's Δt reads |g| ([`super::dt`], §2.3.5's gravity term); P4 and the block summary read φ and g (their lots). The
//! cadence (§1.4.4) is not built.

use sr_physics::reference::fft::{Complex, FftPlan};
use sr_physics::reference::grav::kernel;
use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::fft::{self, padded, FftError, MIN_N};
use super::{compute_pipeline, init_buffer, Dispatch, Size};
use crate::state::{SideFields, State};

/// G = 1 in the sandbox's units (§2.1).
pub const G: f64 = 1.0;
/// Dispatches per step (one more, `grav_sinks`, in a step that runs P3).
pub const DISPATCHES: usize = 4;

/// The kernel's transform for a `width` × `height` world: every padded size's real quarter, in f32, and where each size
/// starts — at index (log2 nx − 5) · `ny_sizes` + log2 ny − 5 — with `ny_sizes`. Refused when an axis pads past
/// [`fft::MAX_N`] (a world over 1024 cells on that axis).
pub fn spectra(width: u32, height: u32) -> Result<(Vec<f32>, Vec<u32>, u32), FftError> {
    spectra_of(width, height, &|dx, dy| kernel(G, dx, dy))
}

/// [`spectra`] for any kernel real and even in both axes, `kernel(dx, dy)` the weight at a signed offset in cells: P7's
/// K_ν (step/nuheat.rs) takes its transforms the same way.
pub fn spectra_of(width: u32, height: u32, kernel: &dyn Fn(isize, isize) -> f64) -> Result<(Vec<f32>, Vec<u32>, u32), FftError> {
    let lengths = |len: u32| -> Result<Vec<u32>, FftError> {
        let top = padded(len)?;
        Ok(std::iter::successors(Some(MIN_N), |n| Some(n * 2)).take_while(|&n| n <= top).collect())
    };
    let (xs, ys) = (lengths(width)?, lengths(height)?);
    let (mut table, mut offsets) = (Vec::new(), Vec::with_capacity(xs.len() * ys.len()));
    for &nx in &xs {
        for &ny in &ys {
            offsets.push(table.len() as u32);
            quarter(nx as usize, ny as usize, kernel, &mut table);
        }
    }
    Ok((table, offsets, ys.len() as u32))
}

/// Appends K̂'s real quarter at `nx` × `ny`, row-major in ky: K sampled at signed offsets (0 … n/2 − 1, then
/// −n/2 … −1, as fft.rs's `Convolution::spectrum`), the rows j ≤ ny/2 transformed (K(i, −j) = K(i, j)), each row's
/// transform real and even, then the columns kx ≤ nx/2.
fn quarter(nx: usize, ny: usize, kernel: &dyn Fn(isize, isize) -> f64, out: &mut Vec<f32>) {
    let plan = |n| FftPlan::new(n).expect("a power-of-two length");
    let transform = |p: &FftPlan, line: &mut [Complex]| p.transform(line, false).expect("a line of the plan's length");
    let (px, py) = (plan(nx), plan(ny));
    let (qx, qy) = (nx / 2 + 1, ny / 2 + 1);
    let signed = |i: usize, n: usize| if i < n / 2 { i as isize } else { i as isize - n as isize };
    let mut rows = vec![0.0; qy * qx];
    let mut line = vec![Complex::ZERO; nx];
    for j in 0..qy {
        for (i, z) in line.iter_mut().enumerate() {
            *z = Complex::new(kernel(signed(i, nx), j as isize), 0.0);
        }
        transform(&px, &mut line);
        for (kx, z) in line[..qx].iter().enumerate() {
            rows[j * qx + kx] = z.re;
        }
    }
    let start = out.len();
    out.resize(start + qx * qy, 0.0);
    let mut line = vec![Complex::ZERO; ny];
    for kx in 0..qx {
        for (j, z) in line.iter_mut().enumerate() {
            *z = Complex::new(rows[j.min(ny - j) * qx + kx], 0.0);
        }
        transform(&py, &mut line);
        for (ky, z) in line[..qy].iter().enumerate() {
            out[start + ky * qx + kx] = z.re as f32;
        }
    }
}

/// P2's buffers and dispatches, built once for a state.
pub struct Gravity {
    field: wgpu::Buffer,
    /// With P3: the gas's own g, which `field` holds plus the sinks' pull.
    gas: Option<wgpu::Buffer>,
    pub(super) dispatches: Vec<Dispatch>,
}

impl Gravity {
    /// P2 over `boxfit`'s box: Σ from `state`, φ into `side`'s φ plane, g into its own two planes, Σ_vac from `physics`.
    /// With `sinks` — P3's state buffer — the sinks' potential joins φ and their pull joins g (§1.4.3), c_sb from `physics`.
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        side: &SideFields,
        physics: &Physics,
        boxfit: &BoxFit,
        sinks: Option<&wgpu::Buffer>,
    ) -> Result<Gravity, FftError> {
        let world = state.world();
        let (table, offsets, ny_sizes) = spectra(world.width, world.height)?;
        let max_nx = padded(world.width)?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grav.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{}\n{}", include_str!("../../shaders/fft.wgsl"), include_str!("../../shaders/grav.wgsl")).into(),
            ),
        });
        let storage = wgpu::BufferUsages::STORAGE;
        let sigma_vac = physics.get("sigma_vac").expect("physics.json key sigma_vac") as f32;
        let cells = world.cells() as u32;
        let fft_params = init_buffer(device, "grav.fft_params", wgpu::BufferUsages::UNIFORM, &[world.width, 0, 0, 0]);
        let c_sb = physics.get("c_sb").expect("physics.json key c_sb") as f32;
        let grav_params = init_buffer(
            device,
            "grav.params",
            wgpu::BufferUsages::UNIFORM,
            &[sigma_vac.to_bits(), ny_sizes, cells, 0, (G as f32).to_bits(), (c_sb * c_sb).to_bits(), 0, 0],
        );
        let twiddles = fft::twiddles(device);
        let kquarter = init_buffer(device, "grav.kernel_quarters", storage, &table.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
        let koffsets = init_buffer(device, "grav.kernel_offsets", storage, &offsets);
        let work = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grav.work"),
            size: 8 * u64::from(max_nx) * u64::from(world.height),
            usage: storage,
            mapped_at_creation: false,
        });
        let plane = |label| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 3 * u64::from(cells) * 4,
                usage: storage | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        };
        let field = plane("grav.g");
        // With sinks, grav_gradient writes the gas's g here and grav_sinks adds the sinks' pull into `field`.
        let gas = sinks.map(|_| plane("grav.g_gas"));

        // Every entry point binds the subset of group 0 it uses (auto layouts); `entry` maps a binding to its buffer.
        let entry = |binding: u32| -> wgpu::BindingResource<'_> {
            match binding {
                0 => boxfit.active().as_entire_binding(),
                1 => fft_params.as_entire_binding(),
                2 => twiddles.as_entire_binding(),
                3 => work.as_entire_binding(),
                4 => state.group(0).as_entire_binding(),
                5 | 10 => side.phi().as_entire_binding(),
                7 => grav_params.as_entire_binding(),
                8 => kquarter.as_entire_binding(),
                9 => koffsets.as_entire_binding(),
                11 => gas.as_ref().unwrap_or(&field).as_entire_binding(),
                12 => sinks.expect("a step with P3").as_entire_binding(),
                _ => field.as_entire_binding(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: entry(b) }).collect();
            let bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            Dispatch { label, pipeline, bind_groups, size }
        };
        let rows = || Size::Fixed([world.height, 1, 1]);
        let mut dispatches = vec![
            dispatch("grav_rows", &[0, 1, 2, 3, 4, 7], rows()),
            dispatch("grav_columns", &[0, 2, 3, 7, 8, 9], Size::Fixed([max_nx, 1, 1])),
            dispatch("conv_rows_inverse", &[0, 1, 2, 3, 5], rows()),
            dispatch("grav_gradient", &[0, 1, 7, 10, 11], Size::Indirect(boxfit.args().clone(), ARGS_TILES)),
        ];
        if sinks.is_some() {
            dispatches.push(dispatch("grav_sinks", &[0, 1, 5, 7, 11, 12, 13], Size::Indirect(boxfit.args().clone(), ARGS_TILES)));
        }
        Ok(Gravity { field, gas, dispatches })
    }

    /// g = −∇φ, f32: the x plane, then the y plane, row-major over the world (y down, §2.12.4); with sinks, their pull
    /// (§1.4.3) included — the field P4 and Δt read.
    pub fn field(&self) -> &wgpu::Buffer {
        &self.field
    }

    /// The gas's own g = −∇φ: [`Gravity::field`] without the sinks' pull — P3's kick-drift-kick reads this one.
    pub fn gas_field(&self) -> &wgpu::Buffer {
        self.gas.as_ref().unwrap_or(&self.field)
    }
}
