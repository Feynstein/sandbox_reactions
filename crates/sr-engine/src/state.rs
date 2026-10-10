//! The cell state (m0_contrat.md §2.2) and the world's size (§2.8).
//!
//! Fourteen f32 channels per cell. On the CPU, the canonical form: one row-major plane per channel, in §2.2's order
//! (the state dump's, §2.12.3) — channel `c` of cell (x, y) is `planes[c·cells + y·width + x]`. On the GPU, the same
//! planes packed into three storage buffers, each a run of whole planes in canonical order ([`GROUPS`]): WebGPU's
//! default binding size (128 MiB) holds at most eight 2048 × 2048 planes, and three buffers leave five of a stage's
//! eight storage slots (TW-E15, §6.2.2) to the passes. Upload and readback go through the canonical form only.
//!
//! Beside the state, and not part of it (§2.2's last paragraph): the per-cell side fields φ, F and S_ν
//! ([`SideFields`]), and the booking layout ([`Booking`]) every pass adds its §2.9 ledger terms and §1.9.1 frame
//! accumulators into — f32 per slot, read back and summed in f64 in a fixed order by [`book_totals`].

use std::fmt;
use std::ops::Range;

use crate::step::Pass;

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
        read_buffers(device, queue, "state.readback", self.groups.iter())
    }
}

/// Copies `buffers`, whole and in order, into one staging buffer and reads it back as f32. Blocks on the device
/// (native and headless only, §6.3).
fn read_buffers<'a>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    buffers: impl Iterator<Item = &'a wgpu::Buffer> + Clone,
) -> Vec<f32> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: buffers.clone().map(|b| b.size()).sum(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
    let mut offset = 0;
    for b in buffers {
        encoder.copy_buffer_to_buffer(b, 0, &staging, offset, b.size());
        offset += b.size();
    }
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap_or_else(|e| panic!("{label}: device lost: {e}"));
    rx.recv().expect("map callback dropped").unwrap_or_else(|e| panic!("{label}: map failed: {e}"));
    let values = {
        let view = staging.get_mapped_range(..).expect("mapped for reading");
        view.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect()
    };
    staging.unmap();
    values
}

fn storage_buffer(device: &wgpu::Device, label: &str, bytes: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

/// The per-cell side fields (§2.2: "not state, per cell"), row-major planes like the state's, f32, zeroed at
/// creation. Each has one writer and is read by the passes after it:
/// - `side.phi` — φ, 1 plane: written by P2 gravity; read by P3 sinks (formation, boundness), P4 hydro (g = −∇φ)
///   and the block summary (Σm·φ, unbound mass, §1.9.1).
/// - `side.flux` — the heat flux F, 2 planes (F_x, F_y): written by P5 heat; read by the next step's P4 (the
///   radiation force, §2.4.4).
/// - `side.s_nu` — the neutrino source S_ν, 1 plane: written by P6 reactions; read by P7 neutrino heating (§2.5.4).
pub struct SideFields {
    phi: wgpu::Buffer,
    flux: wgpu::Buffer,
    s_nu: wgpu::Buffer,
}

impl SideFields {
    pub const PHI_PLANES: usize = 1;
    pub const FLUX_PLANES: usize = 2;
    pub const S_NU_PLANES: usize = 1;

    pub fn new(device: &wgpu::Device, world: WorldConfig) -> Result<SideFields, StateError> {
        world.validate()?;
        let plane = world.cells() as u64 * 4;
        Ok(SideFields {
            phi: storage_buffer(device, "side.phi", Self::PHI_PLANES as u64 * plane),
            flux: storage_buffer(device, "side.flux", Self::FLUX_PLANES as u64 * plane),
            s_nu: storage_buffer(device, "side.s_nu", Self::S_NU_PLANES as u64 * plane),
        })
    }

    pub fn phi(&self) -> &wgpu::Buffer {
        &self.phi
    }

    pub fn flux(&self) -> &wgpu::Buffer {
        &self.flux
    }

    pub fn s_nu(&self) -> &wgpu::Buffer {
        &self.s_nu
    }
}

/// Where a booked term's slots lie. Each slot has one writer invocation per dispatch, which adds its value to it
/// (a read-modify-write of its own slot, never an atomic, §1.3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    /// One slot per cell, row-major (y · width + x).
    Cells,
    /// One slot per boundary face of the world (§2.8): the x = 0 faces by y, then x = width − 1 by y, then y = 0 by x,
    /// then y = height − 1 by x — 2 (width + height) slots; a corner cell owns one face on each of its two sides.
    EdgeFaces,
}

impl Domain {
    pub fn slots(self, world: WorldConfig) -> usize {
        match self {
            Domain::Cells => world.cells(),
            Domain::EdgeFaces => 2 * (world.width as usize + world.height as usize),
        }
    }

    /// The slots in summation order, as rows: a world row each for `Cells`, a side each for `EdgeFaces`.
    pub fn rows(self, world: WorldConfig) -> Vec<Range<usize>> {
        let (w, h) = (world.width as usize, world.height as usize);
        match self {
            Domain::Cells => (0..h).map(|y| y * w..(y + 1) * w).collect(),
            Domain::EdgeFaces => vec![0..h, h..2 * h, 2 * h..2 * h + w, 2 * h + w..2 * (h + w)],
        }
    }
}

/// One booked term: its name (the ledger's, §2.9, or the frame accumulator's, §1.9.1), the one pass that adds into
/// it, and whether the block summary sums it as a frame accumulator.
#[derive(Clone, Copy, Debug)]
pub struct Term {
    pub name: &'static str,
    pub writer: Pass,
    pub accumulator: bool,
}

const fn term(name: &'static str, writer: Pass) -> Term {
    Term { name, writer, accumulator: false }
}

const fn accumulator(name: &'static str, writer: Pass) -> Term {
    Term { name, writer, accumulator: true }
}

/// Every §2.9 booked term and §1.9.1 frame accumulator, with its writer, in booking order. Masses, energies and
/// momenta in sandbox units (§2.1) per slot; energies added are signed (floor_added, tools, nuclear). The ledger's
/// state-derived terms (grid mass, kinetic, thermal, cold, W, the sinks) come from the state and the block summary,
/// not from here; nuclear_released is the sum of the seven nuclear groups.
pub const TERMS: [Term; 37] = [
    // P0 edits (§1.12): mass entered or left by the tools, and the tools' net energy.
    term("mass.painted", Pass::Edits),
    term("mass.erased", Pass::Edits),
    term("mass.preset_dropped", Pass::Edits),
    term("mass.cleared", Pass::Edits),
    term("energy.tools.painted", Pass::Edits),
    term("energy.tools.erased", Pass::Edits),
    term("energy.tools.heated", Pass::Edits),
    term("energy.tools.cooled", Pass::Edits),
    term("energy.tools.preset", Pass::Edits),
    term("energy.tools.cleared", Pass::Edits),
    // P3 sinks (§2.6): the thermal and cold energy of gas a sink forms from or accretes, at the giving cell.
    term("energy.swallowed", Pass::Sinks),
    // P4 hydro (§2.8, leave mode): whatever crosses a boundary face — mass, momentum, energy, each species' mass.
    term("escaped.mass", Pass::Hydro),
    term("escaped.mom_x", Pass::Hydro),
    term("escaped.mom_y", Pass::Hydro),
    term("escaped.energy", Pass::Hydro),
    term("escaped.mass.x_H", Pass::Hydro),
    term("escaped.mass.x_He", Pass::Hydro),
    term("escaped.mass.x_C", Pass::Hydro),
    term("escaped.mass.x_O", Pass::Hydro),
    term("escaped.mass.x_Ne", Pass::Hydro),
    term("escaped.mass.x_Mg", Pass::Hydro),
    term("escaped.mass.x_Si", Pass::Hydro),
    term("escaped.mass.x_S", Pass::Hydro),
    term("escaped.mass.x_Fe", Pass::Hydro),
    term("escaped.mass.x_n", Pass::Hydro),
    // P5 heat (§2.4.4): light leaving into vacuum or through the edge, at the emitting cell.
    accumulator("radiated", Pass::Heat),
    // P6 reactions (§2.5.2, §2.5.3): nuclear energy by group (signed; N is neutronization, ≤ 0), neutrino cooling.
    accumulator("nuclear.H", Pass::Reactions),
    accumulator("nuclear.He", Pass::Reactions),
    accumulator("nuclear.C", Pass::Reactions),
    accumulator("nuclear.Ne", Pass::Reactions),
    accumulator("nuclear.O", Pass::Reactions),
    accumulator("nuclear.Si", Pass::Reactions),
    accumulator("nuclear.N", Pass::Reactions),
    accumulator("neutrino_lost", Pass::Reactions),
    // P7 neutrino heating (§2.5.4).
    accumulator("neutrino_deposited", Pass::NeutrinoHeating),
    // P8 floors (§2.7, §2.3.3): mass reset to vacuum, and the energy the floors add (signed).
    term("mass.vacuum_reset", Pass::Floors),
    term("energy.floor_added", Pass::Floors),
];
pub const N_TERMS: usize = TERMS.len();

/// The booking buffers: buffer `g` holds the planes of `TERMS[BOOK_GROUPS[g].terms]`, in order, each plane
/// `domain.slots(world)` f32. One writer pass per buffer, so a pass binds only its own (P0 binds two).
pub struct BookGroup {
    pub name: &'static str,
    pub terms: Range<usize>,
    pub domain: Domain,
}

pub const BOOK_GROUPS: [BookGroup; 8] = [
    BookGroup { name: "booking.tools_mass", terms: 0..4, domain: Domain::Cells },
    BookGroup { name: "booking.tools_energy", terms: 4..10, domain: Domain::Cells },
    BookGroup { name: "booking.sinks", terms: 10..11, domain: Domain::Cells },
    BookGroup { name: "booking.escaped", terms: 11..25, domain: Domain::EdgeFaces },
    BookGroup { name: "booking.heat", terms: 25..26, domain: Domain::Cells },
    BookGroup { name: "booking.reactions", terms: 26..34, domain: Domain::Cells },
    BookGroup { name: "booking.neutrino_heating", terms: 34..35, domain: Domain::Cells },
    BookGroup { name: "booking.floors", terms: 35..37, domain: Domain::Cells },
];

/// The groups tile `TERMS` in order, each group has one writer, and each fits WebGPU's default binding size at the
/// largest world.
const _: () = {
    let mut next = 0;
    let mut g = 0;
    while g < BOOK_GROUPS.len() {
        let group = &BOOK_GROUPS[g];
        assert!(group.terms.start == next && group.terms.end > group.terms.start);
        let mut t = group.terms.start;
        while t < group.terms.end {
            assert!(TERMS[t].writer as usize == TERMS[group.terms.start].writer as usize);
            t += 1;
        }
        let max = WorldConfig::MAX as u64;
        let slots = match group.domain {
            Domain::Cells => max * max,
            Domain::EdgeFaces => 4 * max,
        };
        assert!((group.terms.end - group.terms.start) as u64 * slots * 4 <= 128 << 20);
        next = group.terms.end;
        g += 1;
    }
    assert!(next == N_TERMS);
};

/// The booking layout on the GPU: one storage buffer per [`BOOK_GROUPS`] entry, zeroed at creation and by
/// [`Booking::clear`]. Passes add into it within a frame; the frame's reader (the block summary, the ledger) reads it
/// and then clears it.
pub struct Booking {
    world: WorldConfig,
    groups: [wgpu::Buffer; 8],
}

impl Booking {
    pub fn new(device: &wgpu::Device, world: WorldConfig) -> Result<Booking, StateError> {
        world.validate()?;
        let groups = std::array::from_fn(|g| {
            let group = &BOOK_GROUPS[g];
            let planes = (group.terms.end - group.terms.start) as u64;
            storage_buffer(device, group.name, planes * group.domain.slots(world) as u64 * 4)
        });
        Ok(Booking { world, groups })
    }

    pub fn world(&self) -> WorldConfig {
        self.world
    }

    /// The storage buffer holding the planes of `BOOK_GROUPS[g]`.
    pub fn group(&self, g: usize) -> &wgpu::Buffer {
        &self.groups[g]
    }

    /// Records the clear of every term, in the encoder's order with the passes around it.
    pub fn clear(&self, encoder: &mut wgpu::CommandEncoder) {
        for b in &self.groups {
            encoder.clear_buffer(b, 0, None);
        }
    }

    /// Reads every term back: the planes of `TERMS` in order, each `domain.slots(world)` long. Blocks on the device
    /// (native and headless only, §6.3).
    pub fn readback(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<f32> {
        read_buffers(device, queue, "booking.readback", self.groups.iter())
    }
}

/// Each term's total, in f64, from a [`Booking::readback`]: term by term, row by row ([`Domain::rows`]), slot by slot,
/// always in that order, so the same planes give the same bits. The ledger (§2.9) sums its booked terms with this.
pub fn book_totals(world: WorldConfig, planes: &[f32]) -> [f64; N_TERMS] {
    let mut totals = [0f64; N_TERMS];
    let mut base = 0;
    for group in &BOOK_GROUPS {
        let slots = group.domain.slots(world);
        let rows = group.domain.rows(world);
        for total in &mut totals[group.terms.clone()] {
            let plane = &planes[base..base + slots];
            for row in &rows {
                *total += plane[row.clone()].iter().map(|&v| v as f64).sum::<f64>();
            }
            base += slots;
        }
    }
    assert_eq!(base, planes.len(), "book_totals: {} values, the layout holds {base}", planes.len());
    totals
}
