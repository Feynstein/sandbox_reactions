//! The block summary (m0_contrat.md §1.9.1, §1.3.4, §3.1; M0-T56), on shaders/summary.wgsl.
//!
//! [`Step::encode_frame`](crate::step::Step::encode_frame) records it after the frame's steps: over the active box, one
//! [`Block`] per 4 × 4 cells, the box's blocks row-major from its origin — mass, Σx·m and Σy·m, momentum, kinetic,
//! thermal and cold energy, ∫Π dA, the ten species masses, Σm·φ, the max-Σ cell (its index, Σ, T, X, Π_cold / Π), the
//! frame accumulators summed (state.rs's `TERMS` marked `accumulator`, M0-T18), the unbound mass and the sinks inside.
//! Every value is an f32 sum of the block's 16 cells in row-major order, so the same state gives the same bits (§1.3.4);
//! the summary feeds observers only.
//!
//! It is read back asynchronously in the frame's own readback, after the latch's records (step/latch.rs): with it, the
//! box record and the sinks' header and slots, so a frame's [`Frame`] arrives in the same `poll` as its report — never
//! later than its frame's submission's end, within §1.9.1's 2 frames. The whole blocks buffer is copied, the world's
//! worth: the CPU learns the box only from the copy.
//!
//! The accumulators reset after each summary: the next frame clears their booking buffers before its first step, so
//! between frames the booking still holds what the last summary summed (tests and the ledger read it there). A frame
//! whose last step's P5 carried over (§0.5 [M0-TJ4] (6)) is summarised as the state stands, the held step's P0–P4 in it.

use std::cell::Cell;

use crate::dump::{Sink, MAX_SINKS};
use crate::state::{Booking, State, BOOK_GROUPS, TERMS};
use crate::step::boxfit::{BoxFit, ARGS_TILES};
use crate::step::latch::FrameReport;
use crate::step::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size};

/// A block's edge in cells (§2.7's `block`).
pub const BLOCK: u32 = 4;
/// Words per block (summary.wgsl `FIELDS`): the 46 of [`Block`] and two pad words.
pub const FIELDS: usize = 48;
/// The nuclear groups' accumulators, in [`TERMS`]' order: H, He, C, Ne, O, Si, N (§1.9.1).
pub const NUCLEAR: [&str; 7] = ["nuclear.H", "nuclear.He", "nuclear.C", "nuclear.Ne", "nuclear.O", "nuclear.Si", "nuclear.N"];
/// The box record's words (reduce.wgsl `ActiveBox`) and the sinks' header and slots (step/sinks.rs).
const BOX_WORDS: usize = 8;
const SINK_HEADER_WORDS: usize = 8;
const SINK_WORDS: usize = 8;
const SINKS_WORDS: usize = SINK_HEADER_WORDS + MAX_SINKS * SINK_WORDS;
/// The passes whose booking buffers hold the accumulators (summary.wgsl bindings 8–10).
const ACCUMULATOR_PASSES: [Pass; 3] = [Pass::Heat, Pass::Reactions, Pass::NeutrinoHeating];

/// One block of 4 × 4 cells: sums over its cells in sandbox units (§2.1; dA = 1), f32 as the GPU added them.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub mass: f32,
    /// Σx·m and Σy·m, x and y a cell's centre (i + ½, j + ½) in world cells.
    pub mx: f32,
    pub my: f32,
    pub momentum: [f32; 2],
    pub kinetic: f32,
    pub thermal: f32,
    pub cold: f32,
    /// ∫Π dA.
    pub pressure: f32,
    /// Σ·X_i, §1.5's order.
    pub species: [f32; 10],
    /// Σm·φ (0 in a step with no φ).
    pub phi_m: f32,
    /// The max-Σ cell (the lowest y, then the lowest x, among equals): its index y·width + x, Σ, T, X and Π_cold / Π.
    pub max_cell: u32,
    pub max_sigma: f32,
    pub max_t: f32,
    pub max_x: [f32; 10],
    pub cold_ratio: f32,
    /// The frame accumulators summed: radiated, nuclear by group ([`NUCLEAR`]), neutrino energy lost and deposited.
    pub radiated: f32,
    pub nuclear: [f32; 7],
    pub neutrino_lost: f32,
    pub neutrino_deposited: f32,
    /// The mass of the cells with ½|u|² + ε_th + φ > 0.
    pub unbound: f32,
    /// The sinks whose position lies in the block's cells: bit k for [`BlockSummary::sinks`]' k.
    pub sinks: u32,
}

impl Block {
    fn from_words(w: &[u32]) -> Block {
        let f = |i: usize| f32::from_bits(w[i]);
        let run = |from: usize| -> [f32; 10] { std::array::from_fn(|i| f(from + i)) };
        Block {
            mass: f(0),
            mx: f(1),
            my: f(2),
            momentum: [f(3), f(4)],
            kinetic: f(5),
            thermal: f(6),
            cold: f(7),
            pressure: f(8),
            species: run(9),
            phi_m: f(19),
            max_cell: w[20],
            max_sigma: f(21),
            max_t: f(22),
            max_x: run(23),
            cold_ratio: f(33),
            radiated: f(34),
            nuclear: std::array::from_fn(|i| f(35 + i)),
            neutrino_lost: f(42),
            neutrino_deposited: f(43),
            unbound: f(44),
            sinks: w[45],
        }
    }
}

/// A frame's block summary: the box it covers and its blocks, `columns` to a row, and the sinks the masks index.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockSummary {
    /// The box's origin and size in cells (multiples of 8; all 0 when the world is empty).
    pub origin: [u32; 2],
    pub size: [u32; 2],
    /// Blocks per row: the box's width / 4.
    pub columns: u32,
    pub blocks: Vec<Block>,
    /// The sinks after the frame, in slot order (none in a step without P3).
    pub sinks: Vec<Sink>,
}

impl BlockSummary {
    /// The block at (column, row) of the box's blocks.
    pub fn block(&self, column: u32, row: u32) -> &Block {
        &self.blocks[(row * self.columns + column) as usize]
    }
}

/// A frame as [`Step::poll_frames`](crate::step::Step::poll_frames) returns it: the report and the block summary.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub report: FrameReport,
    pub summary: BlockSummary,
}

/// The summary's dispatches and buffers, built once for a step.
pub(crate) struct Summary {
    dispatches: Vec<Dispatch>,
    blocks: wgpu::Buffer,
    active: wgpu::Buffer,
    sinks: Option<wgpu::Buffer>,
    /// The booking buffers that hold only accumulators, cleared after each summary.
    accumulators: Vec<wgpu::Buffer>,
    /// A summary was recorded and its accumulators are not cleared yet.
    pending: Cell<bool>,
}

impl Summary {
    /// The summary for `state`'s world over `boxfit`'s box: φ from `phi` (none: read as 0), the sinks from P3's state
    /// buffer when the step has one.
    pub(crate) fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        boxfit: &BoxFit,
        phi: Option<&wgpu::Buffer>,
        sinks: Option<&wgpu::Buffer>,
    ) -> Summary {
        let world = state.world();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("summary.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../shaders/summary.wgsl")).into()),
        });
        let blocks_count = (world.width / BLOCK) as u64 * (world.height / BLOCK) as u64;
        let blocks = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("summary.blocks"),
            size: blocks_count * FIELDS as u64 * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let uniform = wgpu::BufferUsages::UNIFORM;
        let params = init_buffer(device, "summary.params", uniform, &[u32::from(phi.is_some()), 0, 0, 0]);
        let placeholder = init_buffer(device, "summary.placeholder", wgpu::BufferUsages::STORAGE, &[0; 4]);
        let group = |pass: Pass| BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == pass).expect("a booking buffer per accumulator pass");
        let book = |pass: Pass| booking.group(group(pass));
        // Exactly the accumulators: these three buffers hold accumulators only, and every accumulator.
        let terms: Vec<usize> = ACCUMULATOR_PASSES.iter().flat_map(|&p| BOOK_GROUPS[group(p)].terms.clone()).collect();
        assert!(terms.iter().all(|&t| TERMS[t].accumulator) && terms.len() == TERMS.iter().filter(|t| t.accumulator).count());
        let resource = |binding: u32| -> wgpu::BindingResource<'_> {
            match binding {
                0 => state.dims().as_entire_binding(),
                1..=3 => state.group(binding as usize - 1).as_entire_binding(),
                4 => phi.unwrap_or(&placeholder).as_entire_binding(),
                5 => blocks.as_entire_binding(),
                6 => boxfit.active().as_entire_binding(),
                7 => params.as_entire_binding(),
                8 => book(Pass::Heat).as_entire_binding(),
                9 => book(Pass::Reactions).as_entire_binding(),
                10 => book(Pass::NeutrinoHeating).as_entire_binding(),
                _ => sinks.unwrap_or(&placeholder).as_entire_binding(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], with_eos_group: bool| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> = bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: resource(b) }).collect();
            let mut bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            if with_eos_group {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size: Size::Indirect(boxfit.args().clone(), ARGS_TILES) }
        };
        Summary {
            dispatches: vec![dispatch("summary_cells", &[0, 1, 2, 3, 4, 5, 6, 7], true), dispatch("summary_book", &[0, 5, 6, 8, 9, 10, 11], false)],
            blocks,
            active: boxfit.active().clone(),
            sinks: sinks.cloned(),
            accumulators: ACCUMULATOR_PASSES.iter().map(|&p| book(p).clone()).collect(),
            pending: Cell::new(false),
        }
    }

    /// The bytes a frame's readback holds for the summary: the box record, the sinks' words, the blocks.
    pub(crate) fn bytes(&self) -> u64 {
        ((BOX_WORDS + SINKS_WORDS) * 4) as u64 + self.blocks.size()
    }

    /// At a frame's start: the accumulators the last summary summed cleared, before the frame's first step.
    pub(crate) fn encode_reset(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.pending.replace(false) {
            for b in &self.accumulators {
                encoder.clear_buffer(b, 0, None);
            }
        }
    }

    /// After a frame's steps: the summary over the box, then its copy into `readback` from byte `at`.
    pub(crate) fn encode(&self, encoder: &mut wgpu::CommandEncoder, readback: &wgpu::Buffer, at: u64) {
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("block summary"), timestamp_writes: None });
            crate::step::record_dispatches(&mut cpass, &self.dispatches);
        }
        encoder.copy_buffer_to_buffer(&self.active, 0, readback, at, (BOX_WORDS * 4) as u64);
        let sinks_at = at + (BOX_WORDS * 4) as u64;
        match &self.sinks {
            Some(s) => encoder.copy_buffer_to_buffer(s, 0, readback, sinks_at, (SINKS_WORDS * 4) as u64),
            None => encoder.clear_buffer(readback, sinks_at, Some((SINKS_WORDS * 4) as u64)),
        }
        encoder.copy_buffer_to_buffer(&self.blocks, 0, readback, sinks_at + (SINKS_WORDS * 4) as u64, self.blocks.size());
        self.pending.set(true);
    }

    /// The summary from a readback's summary bytes ([`Summary::bytes`]).
    pub(crate) fn decode(bytes: &[u8]) -> BlockSummary {
        let words: Vec<u32> = bytes.as_chunks::<4>().0.iter().map(|b| u32::from_le_bytes(*b)).collect();
        let (abox, rest) = words.split_at(BOX_WORDS);
        let (sinks, blocks) = rest.split_at(SINKS_WORDS);
        let (columns, rows) = (abox[2] / BLOCK, abox[3] / BLOCK);
        let count = (sinks[0] as usize).min(MAX_SINKS);
        let sinks = (0..count)
            .map(|k| {
                let r = |i: usize| f64::from(f32::from_bits(sinks[SINK_HEADER_WORDS + k * SINK_WORDS + i]));
                Sink { x: r(0), y: r(1), vx: r(2), vy: r(3), m: r(4) + r(5) }
            })
            .collect();
        BlockSummary {
            origin: [abox[0], abox[1]],
            size: [abox[2], abox[3]],
            columns,
            blocks: blocks.as_chunks::<FIELDS>().0.iter().take((columns * rows) as usize).map(|w| Block::from_words(w)).collect(),
            sinks,
        }
    }
}
