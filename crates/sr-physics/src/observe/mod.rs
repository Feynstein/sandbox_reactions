//! Observers — they watch, never drive (m0_contrat.md §1.9, I2; M0-T58).
//!
//! The engine hands a frame's block summary over as a [`Report`] in f64 (the values f32 sums made, widened exactly);
//! [`Tracker::update`] turns it into objects with every §1.9.2 quantity. Nothing here reaches back into a step: G-WATCH
//! keeps this module out of `crates/sr-engine/src/step/` and the shaders.

pub mod age;
pub mod ending;
pub mod stages;
pub mod tracker;
pub mod translate;

pub use stages::{classify, Event, EventKind, History, Stage, StageTracker};
pub use tracker::{Object, Tracker, TrackerConfig};

/// The nuclear groups' order in [`BlockData::nuclear`] and [`Object::l_group`]: H, He, C, Ne, O, Si, N (§1.9.1).
pub const GROUPS: [&str; 7] = ["H", "He", "C", "Ne", "O", "Si", "N"];
/// N_Fe's index in [`GROUPS`]: its power is the neutronization, an absorbed power ≤ 0 (§1.9.2).
pub const GROUP_N: usize = 6;

/// One block of the summary, the fields an observer reads (§1.9.1), in the engine's units (§2.1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockData {
    pub mass: f64,
    /// Σx·m and Σy·m, x and y a cell's centre (i + ½, j + ½) in world cells.
    pub mx: f64,
    pub my: f64,
    pub momentum: [f64; 2],
    pub kinetic: f64,
    pub thermal: f64,
    pub cold: f64,
    /// ∫Π dA.
    pub pressure: f64,
    pub species: [f64; 10],
    /// Σm·φ.
    pub phi_m: f64,
    /// The max-Σ cell's index y·width + x, its Σ, T, X (§1.5's order) and Π_cold / Π.
    pub max_cell: u32,
    pub max_sigma: f64,
    pub max_t: f64,
    pub max_x: [f64; 10],
    pub cold_ratio: f64,
    /// The frame's accumulators: radiated, nuclear by group ([`GROUPS`]), neutrino energy lost and deposited.
    pub radiated: f64,
    pub nuclear: [f64; 7],
    pub neutrino_lost: f64,
    pub neutrino_deposited: f64,
    /// The mass of the cells with ½|u|² + ε_th + φ > 0.
    pub unbound: f64,
    /// The sinks whose position lies in the block: bit k for [`Report::sinks`]' k.
    pub sinks: u32,
}

/// A sink after the frame (§2.6): position in world cells, velocity, mass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SinkData {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub m: f64,
}

/// A frame as the observers take it: the summary's blocks over the box and the frame's sim time (§1.9.1, §3.1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// The sim time the frame covered: L is the frame's accumulator ÷ this.
    pub sim_dt: f64,
    /// The world's width in cells, to read a `max_cell` index.
    pub world_width: u32,
    /// The box's origin in cells (a multiple of the block edge) and its blocks per row.
    pub origin: [u32; 2],
    pub columns: u32,
    /// The box's blocks, row-major from its origin.
    pub blocks: Vec<BlockData>,
    pub sinks: Vec<SinkData>,
}
