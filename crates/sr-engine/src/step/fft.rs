//! The GPU FFT (m0_contrat.md §1.3.3, §1.4.2, §6.2.2), on shaders/fft.wgsl: 2D complex transforms of 32 … 2048 points
//! per axis, and the zero-padded convolution φ = K ∗ s over the active box with the kernel's transform cached per
//! padded size. The pass that uses it (P2 gravity, P7's kernel) is its lot's; the f64 reference is
//! sr_physics::reference::fft, whose conventions this follows (row-major, forward unnormalised, inverse ÷ N).
//!
//! One row or one column per workgroup, in workgroup memory: a 2048-point row of vec2<f32> is exactly WebGPU's 16,384 B,
//! so the shader keeps nothing else there — the twiddles are one table of e^{−2πi k/2048} in a storage buffer, taken in
//! f64 here, a length n reading every (2048/n)-th entry. Dispatches, each a fixed size for its box:
//! - a 2D transform ([`Fft::transform`]): rows, then columns — [`TRANSFORM_DISPATCHES`];
//! - a convolution ([`Convolution::dispatches`]): the box's h rows zero-padded and forward; each column forward, times
//!   the kernel's transform, inverse (decimation in frequency, so no reordering pass); the h rows inverse, the box's
//!   cells written — [`CONVOLUTION_DISPATCHES`]. The padded rows h … n − 1 are zero and never transformed.
//!
//! The shader's geometry uniform has the active box's layout (reduce.wgsl's ActiveBox: x0, y0, w, h, fft_w, fft_h, …)
//! and every entry point returns early past the box's rows or columns, so a caller may bind the box's own record and
//! over-dispatch; here each call writes its own.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;

use super::{compute_pipeline, init_buffer, record_dispatches, Dispatch, Size};
use crate::state::read_buffers;

/// The smallest and largest transform length per axis (§1.3.3).
pub const MIN_N: u32 = 32;
pub const MAX_N: u32 = 2048;
/// Dispatches per 2D transform and per convolution.
pub const TRANSFORM_DISPATCHES: usize = 2;
pub const CONVOLUTION_DISPATCHES: usize = 3;

/// What a transform or a convolution refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FftError {
    /// A transform length that is not a power of two in [`MIN_N`] ..= [`MAX_N`].
    Length(u32),
    /// A box axis that is empty or pads past [`MAX_N`].
    BoxAxis(u32),
    /// A box that leaves its plane's rows.
    Outside { x0: u32, w: u32, stride: u32 },
}

impl fmt::Display for FftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FftError::Length(n) => write!(f, "FFT length {n} is not a power of two in {MIN_N}..={MAX_N}"),
            FftError::BoxAxis(n) => write!(f, "box axis of {n} cells pads outside {MIN_N}..={MAX_N}"),
            FftError::Outside { x0, w, stride } => write!(f, "box columns {x0}..{} leave a row of {stride}", x0 + w),
        }
    }
}

impl std::error::Error for FftError {}

/// The padded size of a box axis of `len` cells: the smallest power of two ≥ 2 × `len`, at least [`MIN_N`] (§1.3.3).
pub fn padded(len: u32) -> Result<u32, FftError> {
    let n = (2 * len).next_power_of_two().max(MIN_N);
    if len == 0 || n > MAX_N {
        return Err(FftError::BoxAxis(len));
    }
    Ok(n)
}

fn check_length(n: u32) -> Result<u32, FftError> {
    if !n.is_power_of_two() || !(MIN_N..=MAX_N).contains(&n) {
        return Err(FftError::Length(n));
    }
    Ok(n)
}

/// The active box a convolution runs over, in cells of its planes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x0: u32,
    pub y0: u32,
    pub w: u32,
    pub h: u32,
}

/// A real plane inside a storage buffer of f32: the buffer and where the plane starts, in f32 elements.
#[derive(Clone, Copy, Debug)]
pub struct Plane<'a> {
    pub buffer: &'a wgpu::Buffer,
    pub offset: u32,
}

/// The pipelines and the twiddle table, built once per device.
pub struct Fft {
    rows_c2c: wgpu::ComputePipeline,
    columns_c2c: wgpu::ComputePipeline,
    conv_rows: wgpu::ComputePipeline,
    conv_columns: wgpu::ComputePipeline,
    conv_rows_inverse: wgpu::ComputePipeline,
    twiddles: wgpu::Buffer,
}

impl Fft {
    pub fn new(device: &wgpu::Device) -> Fft {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fft.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/fft.wgsl").into()),
        });
        Fft {
            rows_c2c: compute_pipeline(device, &module, "rows_c2c"),
            columns_c2c: compute_pipeline(device, &module, "columns_c2c"),
            conv_rows: compute_pipeline(device, &module, "conv_rows"),
            conv_columns: compute_pipeline(device, &module, "conv_columns"),
            conv_rows_inverse: compute_pipeline(device, &module, "conv_rows_inverse"),
            twiddles: twiddles(device),
        }
    }

    /// The dispatches of a 2D transform in place of `data`, `nx` × `ny` complex values (vec2<f32>, row-major): the
    /// rows, then the columns; with `inverse` the conjugate transform divided by nx · ny.
    pub fn transform(
        &self,
        device: &wgpu::Device,
        data: &wgpu::Buffer,
        nx: u32,
        ny: u32,
        inverse: bool,
    ) -> Result<Vec<Dispatch>, FftError> {
        let (nx, ny) = (check_length(nx)?, check_length(ny)?);
        let geometry = geometry(device, Region { x0: 0, y0: 0, w: nx, h: ny }, nx, ny);
        let params = params(device, 0, 0, 0, inverse);
        let entries = [(0, &geometry), (1, &params), (2, &self.twiddles), (3, data)];
        Ok(vec![
            dispatch(device, "fft rows_c2c", &self.rows_c2c, &entries, ny),
            dispatch(device, "fft columns_c2c", &self.columns_c2c, &entries, nx),
        ])
    }
}

/// The zero-padded convolution φ(x) = Σ_y K(x − y) s(y) over a box, the kernel's transform cached per padded size.
pub struct Convolution {
    fft: Fft,
    kernel: Box<dyn Fn(isize, isize) -> f64>,
    spectra: RefCell<HashMap<(u32, u32), wgpu::Buffer>>,
}

impl Convolution {
    /// A convolution with `kernel(dx, dy)`, the weight of a source `dx` cells left of and `dy` above the target
    /// (the twin's sign convention: dx = x_target − x_source).
    pub fn new(device: &wgpu::Device, kernel: impl Fn(isize, isize) -> f64 + 'static) -> Convolution {
        Convolution { fft: Fft::new(device), kernel: Box::new(kernel), spectra: RefCell::new(HashMap::new()) }
    }

    /// The padded sizes whose kernel transform is cached, sorted.
    pub fn cached_sizes(&self) -> Vec<(u32, u32)> {
        let mut sizes: Vec<_> = self.spectra.borrow().keys().copied().collect();
        sizes.sort_unstable();
        sizes
    }

    /// The kernel's transform at `nx` × `ny`: sampled at signed offsets (0 … n/2 − 1, then −n/2 … −1) in f64, stored
    /// in f32 and transformed on the GPU (submitted here) the first time a size is asked for, kept after.
    pub fn spectrum(&self, device: &wgpu::Device, queue: &wgpu::Queue, nx: u32, ny: u32) -> Result<wgpu::Buffer, FftError> {
        if let Some(b) = self.spectra.borrow().get(&(nx, ny)) {
            return Ok(b.clone());
        }
        let (nx, ny) = (check_length(nx)?, check_length(ny)?);
        let signed = |i: u32, n: u32| if i < n / 2 { i as isize } else { i as isize - n as isize };
        let mut words = Vec::with_capacity(2 * (nx * ny) as usize);
        for j in 0..ny {
            for i in 0..nx {
                words.push(((self.kernel)(signed(i, nx), signed(j, ny)) as f32).to_bits());
                words.push(0);
            }
        }
        let buffer = init_buffer(device, "fft.kernel_spectrum", wgpu::BufferUsages::STORAGE, &words);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fft.kernel") });
        encode(&mut encoder, &self.fft.transform(device, &buffer, nx, ny, false)?);
        queue.submit([encoder.finish()]);
        self.spectra.borrow_mut().insert((nx, ny), buffer.clone());
        Ok(buffer)
    }

    /// The dispatches of φ = K ∗ s over `region`: `src`'s cells in the box (every other cell ignored) into the same
    /// cells of `dst` (every other cell left as it was); both planes `stride` cells wide. The kernel's transform for
    /// the box's padded size is made now if it is not cached.
    pub fn dispatches(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: Plane<'_>,
        dst: Plane<'_>,
        stride: u32,
        region: Region,
    ) -> Result<Vec<Dispatch>, FftError> {
        let (nx, ny) = (padded(region.w)?, padded(region.h)?);
        if region.x0 + region.w > stride {
            return Err(FftError::Outside { x0: region.x0, w: region.w, stride });
        }
        let kspec = self.spectrum(device, queue, nx, ny)?;
        let work = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fft.work"),
            size: 8 * u64::from(nx) * u64::from(region.h),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let geometry = geometry(device, region, nx, ny);
        let params = params(device, stride, src.offset, dst.offset, false);
        let f = &self.fft;
        let (g, t, w) = ((0, &geometry), (2, &f.twiddles), (3, &work));
        Ok(vec![
            dispatch(device, "fft conv_rows", &f.conv_rows, &[g, (1, &params), t, w, (4, src.buffer)], region.h),
            dispatch(device, "fft conv_columns", &f.conv_columns, &[g, t, w, (6, &kspec)], nx),
            dispatch(device, "fft conv_rows_inverse", &f.conv_rows_inverse, &[g, (1, &params), t, w, (5, dst.buffer)], region.h),
        ])
    }
}

/// Records `dispatches` in one compute pass.
pub fn encode(encoder: &mut wgpu::CommandEncoder, dispatches: &[Dispatch]) {
    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("fft"), timestamp_writes: None });
    record_dispatches(&mut cpass, dispatches);
}

/// A storage buffer read back as f32 (tests and headless runs only; blocks on the device).
pub fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> Vec<f32> {
    read_buffers(device, queue, "fft.readback", std::iter::once(buffer))
}

/// fft.wgsl's twiddle table: e^{−2πi k/2048} for k < 1024, taken in f64, stored as vec2<f32>.
pub(super) fn twiddles(device: &wgpu::Device) -> wgpu::Buffer {
    let words: Vec<u32> = (0..MAX_N / 2)
        .flat_map(|k| {
            let (s, c) = (-2.0 * PI * f64::from(k) / f64::from(MAX_N)).sin_cos();
            [(c as f32).to_bits(), (s as f32).to_bits()]
        })
        .collect();
    init_buffer(device, "fft.twiddles", wgpu::BufferUsages::STORAGE, &words)
}

/// The geometry uniform, in reduce.wgsl's ActiveBox layout.
fn geometry(device: &wgpu::Device, r: Region, nx: u32, ny: u32) -> wgpu::Buffer {
    init_buffer(device, "fft.geometry", wgpu::BufferUsages::UNIFORM, &[r.x0, r.y0, r.w, r.h, nx, ny, 0, 0])
}

fn params(device: &wgpu::Device, stride: u32, src_offset: u32, dst_offset: u32, inverse: bool) -> wgpu::Buffer {
    let words = [stride, src_offset, dst_offset, u32::from(inverse)];
    init_buffer(device, "fft.params", wgpu::BufferUsages::UNIFORM, &words)
}

/// One workgroup per row or column; the group holds exactly the bindings the entry point uses (automatic layout).
fn dispatch(
    device: &wgpu::Device,
    label: &'static str,
    pipeline: &wgpu::ComputePipeline,
    entries: &[(u32, &wgpu::Buffer)],
    workgroups: u32,
) -> Dispatch {
    let entries: Vec<_> = entries
        .iter()
        .map(|&(binding, b)| wgpu::BindGroupEntry { binding, resource: b.as_entire_binding() })
        .collect();
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });
    Dispatch { label, pipeline: pipeline.clone(), bind_groups: vec![group], size: Size::Fixed([workgroups, 1, 1]) }
}
