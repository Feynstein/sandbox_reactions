//! The cell state (m0_contrat.md §2.2) and the world's size (§2.8).
//!
//! Fourteen f32 channels per cell. On the CPU, the canonical form: one row-major plane per channel, in §2.2's order
//! (the state dump's, §2.12.3) — channel `c` of cell (x, y) is `planes[c·cells + y·width + x]`. On the GPU, the same
//! planes packed into three storage buffers, each a run of whole planes in canonical order ([`GROUPS`]): WebGPU's
//! default binding size (128 MiB) holds at most eight 2048 × 2048 planes, and three buffers leave five of a stage's
//! eight storage slots (TW-E15, §6.2.2) to the passes. Upload and readback go through the canonical form only.

use std::fmt;
use std::ops::Range;

/// §2.2's channels in canonical order — the dump's names (§2.12.3).
pub const CHANNELS: [&str; 14] = [
    "sigma", "mom_x", "mom_y", "energy", "x_H", "x_He", "x_C", "x_O", "x_Ne", "x_Mg", "x_Si", "x_S", "x_Fe", "x_n",
];
pub const N_CHANNELS: usize = CHANNELS.len();
/// The mass fractions are channels `SPECIES` (§1.5's order), Σ_i X_i = 1.
pub const SPECIES: Range<usize> = 4..14;

/// The GPU packing: buffer `g` holds the planes `GROUPS[g]`, in canonical order. 0 — sigma, mom_x, mom_y, energy;
/// 1 — x_H … x_Ne; 2 — x_Mg … x_n.
pub const GROUPS: [Range<usize>; 3] = [0..4, 4..9, 9..14];
pub const GROUP_NAMES: [&str; 3] = ["state.hydro", "state.species_a", "state.species_b"];

/// The largest group, at the largest world, fits WebGPU's default `max_storage_buffer_binding_size` (128 MiB).
const _: () = {
    let mut g = 0;
    while g < GROUPS.len() {
        let planes = (GROUPS[g].end - GROUPS[g].start) as u64;
        assert!(planes * (WorldConfig::MAX as u64) * (WorldConfig::MAX as u64) * 4 <= 128 << 20);
        g += 1;
    }
};

/// The world's size in cells (§2.8): a setting, never built in — default 600 × 400, any width and height that are
/// multiples of 8 within [64, 2048].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { width: 600, height: 400 }
    }
}

impl WorldConfig {
    pub const MIN: u32 = 64;
    pub const MAX: u32 = 2048;
    pub const MULTIPLE: u32 = 8;

    pub fn new(width: u32, height: u32) -> Result<Self, StateError> {
        let world = Self { width, height };
        world.validate()?;
        Ok(world)
    }

    pub fn validate(&self) -> Result<(), StateError> {
        let ok = |n: u32| (Self::MIN..=Self::MAX).contains(&n) && n.is_multiple_of(Self::MULTIPLE);
        if ok(self.width) && ok(self.height) {
            Ok(())
        } else {
            Err(StateError::WorldSize { width: self.width, height: self.height })
        }
    }

    pub fn cells(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StateError {
    /// Outside §2.8's sizes: a multiple of 8 within [64, 2048] on each axis.
    WorldSize { width: u32, height: u32 },
    /// An upload of other than 14 planes of this world's cells.
    PlanesLength { expected: usize, got: usize },
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::WorldSize { width, height } => write!(
                f,
                "world {width} × {height}: width and height must be multiples of {} within [{}, {}]",
                WorldConfig::MULTIPLE,
                WorldConfig::MIN,
                WorldConfig::MAX
            ),
            StateError::PlanesLength { expected, got } => {
                write!(f, "state upload: {got} values, expected {expected} ({N_CHANNELS} planes of the world's cells)")
            }
        }
    }
}

impl std::error::Error for StateError {}

/// The cell state on the GPU, and the world's dimensions every pass reads (a uniform:
/// `struct Dims { width: u32, height: u32, cells: u32, _pad: u32 }`).
pub struct State {
    world: WorldConfig,
    groups: [wgpu::Buffer; 3],
    dims: wgpu::Buffer,
}

impl State {
    /// A zeroed state for `world` (wgpu zero-initialises new buffers).
    pub fn new(device: &wgpu::Device, world: WorldConfig) -> Result<State, StateError> {
        world.validate()?;
        let cells = world.cells() as u64;
        let groups = std::array::from_fn(|g| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(GROUP_NAMES[g]),
                size: (GROUPS[g].end - GROUPS[g].start) as u64 * cells * 4,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        });
        let dims = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("state.dims"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });
        {
            let words = [world.width, world.height, world.cells() as u32, 0];
            let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
            dims.get_mapped_range_mut(..).expect("mapped at creation").copy_from_slice(&bytes);
        }
        dims.unmap();
        Ok(State { world, groups, dims })
    }

    pub fn world(&self) -> WorldConfig {
        self.world
    }

    /// The storage buffer holding the planes `GROUPS[g]`.
    pub fn group(&self, g: usize) -> &wgpu::Buffer {
        &self.groups[g]
    }

    pub fn dims(&self) -> &wgpu::Buffer {
        &self.dims
    }

    /// Writes the whole state from its canonical form: 14 row-major planes in §2.2's order.
    pub fn upload(&self, queue: &wgpu::Queue, planes: &[f32]) -> Result<(), StateError> {
        let cells = self.world.cells();
        if planes.len() != N_CHANNELS * cells {
            return Err(StateError::PlanesLength { expected: N_CHANNELS * cells, got: planes.len() });
        }
        for (g, range) in GROUPS.iter().enumerate() {
            let bytes: Vec<u8> =
                planes[range.start * cells..range.end * cells].iter().flat_map(|v| v.to_le_bytes()).collect();
            queue.write_buffer(&self.groups[g], 0, &bytes);
        }
        Ok(())
    }

    /// Reads the whole state back in its canonical form, after every submitted step. Blocks on the device: native
    /// and headless only (§6.3) — a browser maps asynchronously.
    pub fn readback(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<f32> {
        let plane_bytes = self.world.cells() as u64 * 4;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("state.readback"),
            size: N_CHANNELS as u64 * plane_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("state.readback") });
        for (g, range) in GROUPS.iter().enumerate() {
            let size = (range.end - range.start) as u64 * plane_bytes;
            encoder.copy_buffer_to_buffer(&self.groups[g], 0, &staging, range.start as u64 * plane_bytes, size);
        }
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        staging.map_async(wgpu::MapMode::Read, .., move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).expect("device lost while reading the state back");
        rx.recv().expect("map callback dropped").expect("state readback: map failed");
        let planes = {
            let view = staging.get_mapped_range(..).expect("mapped for reading");
            view.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect()
        };
        staging.unmap();
        planes
    }
}
