//! P0 edits (m0_contrat.md §1.3.2, §1.12, §2.9, §3.1; M0-T76), on shaders/edit.wgsl: the player's tools — paint, erase,
//! heat, cool, drop a preset, clear the world — queued by [`Step::queue_edit`] and applied at the next step's P0.
//!
//! Three dispatches, all before the box's own re-fit ([`super::boxfit`]) in P0: `edit_gate` (one invocation) sizes
//! `edit_cells` from the queue's count through the box's gate buffer — nothing when no edit is queued — and sets the box's
//! edit flag when a paint or a drop lands outside the active box; `edit_cells` walks the queue in order, one invocation
//! per cell; `edit_finish` empties it and, for a Clear, removes the sinks. The two single-invocation dispatches take the
//! latch's fixed size, `edit_cells` the gate the latch already zeroes, so a frame's latch reaches P0 as it reaches every
//! pass (§1.8.5). Each edit is booked in §2.9's terms into the booking's P0 buffers (state.rs): mass painted, erased,
//! preset_dropped, cleared; energy `tools.*` — the E plane's stored change, so the ledger balances to the bit. A step whose
//! queue holds a paint or a drop records Δt's re-reduction after the box's re-fit ([`super::dt`], §0.5 [M0-D110]), so it
//! steps on the bound of the gas it added.
//!
//! The queue lives on the GPU; the CPU keeps the records queued since the last step so that a second edit before a step
//! joins the first ([`Step::queue_edit`]). At most [`MAX_EDITS`] records, and preset clouds of at most [`PATCH_CELLS`] cells
//! between them, wait for one step; one more is [`EditError::QueueFull`].
//!
//! Not built by [`Step::new`] and its siblings: [`Step::with_edits`] adds P0's dispatches to a built step, so the steps of
//! the other scopes keep their pass lists.

use sr_physics::registry::{Elements, Physics, N_SPECIES};

use super::boxfit::BoxFit;
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size, Step};
use crate::scene::{self, CloudError};
use crate::state::{Booking, State, WorldConfig, BOOK_GROUPS, TERMS};

/// The edits one step applies at most.
pub const MAX_EDITS: usize = 8;
/// Words of one queued record (edit.wgsl `REC`) and of the queue's head.
const RECORD_WORDS: usize = 24;
const HEAD_WORDS: usize = 4;
/// The brush radii the tools take (§4.4).
pub const RADIUS: std::ops::RangeInclusive<u32> = 1..=40;
/// The patch arena: the cells of the preset clouds queued for one step — one cloud at the largest radius (§2.10).
pub const PATCH_CELLS: u32 = 300 * 300;

/// One player edit (§3.1). Cells are (x, y) in the world (§2.12.4).
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// ΔΣ = `paint_sigma` of species `species` (§1.5's id) at `paint_t`, at rest, in the disk of `radius` cells.
    Paint { cell: [u32; 2], radius: u32, species: usize },
    Erase { cell: [u32; 2], radius: u32 },
    Heat { cell: [u32; 2], radius: u32 },
    Cool { cell: [u32; 2], radius: u32 },
    /// A preset cloud ready to write ([`Edit::drop_preset`]).
    DropPreset(scene::Cloud),
    /// Every cell to the floor state, the sinks removed; the caller restarts its ledger ([`Edit::restarts_ledger`]).
    Clear,
}

/// Why an edit was not queued.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// A preset whose cloud would reach past the world's edge: the app shows `presets.no_room` (§4.5).
    NoRoom,
    /// A cell outside the world, a radius outside 1–40, a species outside the registry, an unknown preset or a bad mass.
    Invalid(String),
    /// [`MAX_EDITS`] edits (or the patch arena) already wait for the next step.
    QueueFull,
    /// The step was built without [`Step::with_edits`].
    NotBuilt,
}

impl EditError {
    /// The §4.5 string key the app shows for this refusal, where it has one.
    pub fn message_key(&self) -> Option<&'static str> {
        matches!(self, EditError::NoRoom).then_some("presets.no_room")
    }
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::NoRoom => write!(f, "not enough room for this star"),
            EditError::Invalid(why) => write!(f, "edit refused: {why}"),
            EditError::QueueFull => write!(f, "edit refused: {MAX_EDITS} edits already wait for the next step"),
            EditError::NotBuilt => write!(f, "edit refused: the step was built without edits"),
        }
    }
}

impl std::error::Error for EditError {}

impl Edit {
    /// The preset `preset` at mass `mass` (sandbox units, §2.10: the app's choice, not this module's) centred on `cell`, or
    /// [`EditError::NoRoom`] when any of its cells would fall outside the world (§1.12).
    pub fn drop_preset(
        preset: &str,
        mass: f64,
        cell: [u32; 2],
        world: WorldConfig,
        physics: &Physics,
        elements: &Elements,
    ) -> Result<Edit, EditError> {
        let centre = [f64::from(cell[0]) + 0.5, f64::from(cell[1]) + 0.5];
        match scene::preset_cloud(preset, mass, centre, world, physics, elements) {
            Ok(cloud) => Ok(Edit::DropPreset(cloud)),
            Err(CloudError::NoRoom) => Err(EditError::NoRoom),
            Err(CloudError::Invalid(why)) => Err(EditError::Invalid(why)),
        }
    }

    /// Whether the ledger restarts with this edit (§1.12: Clear world).
    pub fn restarts_ledger(&self) -> bool {
        matches!(self, Edit::Clear)
    }
}

/// The records queued since the step that last ran, and where the patches stand.
#[derive(Default)]
struct Pending {
    /// The CPU step count the records were queued at: a step recorded since has applied them.
    at_step: u64,
    records: Vec<[u32; RECORD_WORDS]>,
    patch_cells: u32,
}

/// P0's queue, patch arena and dispatches.
pub struct Edits {
    world: WorldConfig,
    queue: wgpu::Buffer,
    patch: wgpu::Buffer,
    pending: std::cell::RefCell<Pending>,
    pub(super) dispatches: Vec<Dispatch>,
}

impl Edits {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        boxfit: &BoxFit,
        sinks: Option<&wgpu::Buffer>,
    ) -> Edits {
        let world = state.world();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edit.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/edit.wgsl")).into()),
        });
        let storage = wgpu::BufferUsages::STORAGE;
        let queue = init_buffer(
            device,
            "edit.queue",
            storage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            &[0; HEAD_WORDS + MAX_EDITS * RECORD_WORDS],
        );
        let patch_cells = PATCH_CELLS.min(world.cells() as u32);
        let patch = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edit.patch"),
            size: u64::from(patch_cells) * 4,
            usage: storage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let params = init_buffer(
            device,
            "edit.params",
            wgpu::BufferUsages::UNIFORM,
            &["paint_sigma", "paint_t", "heat_factor", "cool_factor"].map(|k| get(k).to_bits()),
        );
        let no_sinks = sinks.is_none().then(|| init_buffer(device, "edit.no_sinks", storage, &[0; 2]));
        let sinks = sinks.or(no_sinks.as_ref()).expect("the sinks' buffer or its stand-in");
        let book = |writer_term: &str| {
            let g = BOOK_GROUPS
                .iter()
                .position(|g| TERMS[g.terms.start].name == writer_term && TERMS[g.terms.start].writer == Pass::Edits)
                .expect("a booking buffer written by P0");
            booking.group(g)
        };
        let (book_mass, book_energy) = (book("mass.painted"), book("energy.tools.painted"));

        let entry = |binding: u32| -> wgpu::BindingResource<'_> {
            match binding {
                0 => state.dims().as_entire_binding(),
                1 => state.group(1).as_entire_binding(),
                2 => state.group(2).as_entire_binding(),
                3 => state.group(0).as_entire_binding(),
                4 => book_mass.as_entire_binding(),
                5 => book_energy.as_entire_binding(),
                6 => queue.as_entire_binding(),
                7 => patch.as_entire_binding(),
                8 => params.as_entire_binding(),
                9 => boxfit.gate().as_entire_binding(),
                10 => boxfit.edit_flag().as_entire_binding(),
                11 => boxfit.active().as_entire_binding(),
                _ => sinks.as_entire_binding(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size, with_eos_group: bool| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: entry(b) }).collect();
            let mut bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            if with_eos_group {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size }
        };
        let one = || Size::Fixed([1, 1, 1]);
        let dispatches = vec![
            dispatch("edit_gate", &[0, 6, 9, 10, 11], one(), false),
            dispatch("edit_cells", &[0, 1, 2, 3, 4, 5, 6, 7, 8], Size::Indirect(boxfit.gate().clone(), 0), true),
            dispatch("edit_finish", &[6, 12], one(), false),
        ];
        Edits { world, queue, patch, pending: Default::default(), dispatches }
    }
}

impl Edits {
    /// Whether step `step`'s P0 applies a paint or a preset drop (kinds 1 and 5): that step re-makes its Δt after them
    /// (§0.5 [M0-D110]).
    pub(super) fn deposits_at(&self, step: u64) -> bool {
        let pending = self.pending.borrow();
        pending.at_step == step && pending.records.iter().any(|r| matches!(r[0], 1 | 5))
    }
}

impl Step {
    /// This step with P0's edit pass: [`Step::queue_edit`] works from here on. Built on the step's own box, booking and
    /// sinks (the Clear removes them), so the other passes' dispatch lists are untouched.
    pub fn with_edits(mut self, device: &wgpu::Device, state: &State, booking: &Booking, eos: &EosGpu, physics: &Physics) -> Step {
        let mut edits = Edits::new(device, state, booking, eos, physics, &self.boxfit, self.sinks.as_ref().map(|s| s.state_buffer()));
        self.passes[Pass::Edits as usize] = std::mem::take(&mut edits.dispatches);
        self.edits = Some(edits);
        self
    }

    /// Queues `edit` for the next step's P0 (§3.1). A second edit before that step joins the first and they apply in
    /// order, once; a step recorded since the last call has applied the earlier ones. Clear restarts the caller's ledger
    /// ([`Edit::restarts_ledger`]).
    pub fn queue_edit(&self, queue: &wgpu::Queue, edit: &Edit) -> Result<(), EditError> {
        let edits = self.edits.as_ref().ok_or(EditError::NotBuilt)?;
        let world = edits.world;
        let in_world = |c: &[u32; 2]| c[0] < world.width && c[1] < world.height;
        let brush = |kind: u32, cell: &[u32; 2], radius: u32, species: usize| {
            if !in_world(cell) {
                return Err(EditError::Invalid(format!("cell ({}, {}) is outside the {}×{} world", cell[0], cell[1], world.width, world.height)));
            }
            if !RADIUS.contains(&radius) {
                return Err(EditError::Invalid(format!("radius {radius} is outside {}–{}", RADIUS.start(), RADIUS.end())));
            }
            if species >= N_SPECIES {
                return Err(EditError::Invalid(format!("species {species} is not in the registry")));
            }
            let mut r = [0u32; RECORD_WORDS];
            r[..5].copy_from_slice(&[kind, cell[0], cell[1], radius, species as u32]);
            Ok(r)
        };
        let mut patch_cells = 0;
        let record = match edit {
            Edit::Paint { cell, radius, species } => brush(1, cell, *radius, *species)?,
            Edit::Erase { cell, radius } => brush(2, cell, *radius, 0)?,
            Edit::Heat { cell, radius } => brush(3, cell, *radius, 0)?,
            Edit::Cool { cell, radius } => brush(4, cell, *radius, 0)?,
            Edit::DropPreset(cloud) => {
                patch_cells = cloud.sigma.len() as u32;
                let mut r = [0u32; RECORD_WORDS];
                r[0] = 5;
                r[5..9].copy_from_slice(&[cloud.x0, cloud.y0, cloud.w, cloud.h]);
                r[10] = cloud.t.to_bits();
                for (word, x) in r[11..].iter_mut().zip(cloud.composition) {
                    *word = x.to_bits();
                }
                r
            }
            Edit::Clear => {
                let mut r = [0u32; RECORD_WORDS];
                r[0] = 6;
                r
            }
        };
        let mut pending = edits.pending.borrow_mut();
        if pending.at_step != self.next.get() {
            *pending = Pending { at_step: self.next.get(), ..Pending::default() };
        }
        if pending.records.len() == MAX_EDITS || pending.patch_cells + patch_cells > PATCH_CELLS.min(world.cells() as u32) {
            return Err(EditError::QueueFull);
        }
        let mut record = record;
        if let Edit::DropPreset(cloud) = edit {
            record[9] = pending.patch_cells;
            let bytes: Vec<u8> = cloud.sigma.iter().flat_map(|v| v.to_le_bytes()).collect();
            queue.write_buffer(&edits.patch, u64::from(pending.patch_cells) * 4, &bytes);
            pending.patch_cells += patch_cells;
        }
        pending.records.push(record);
        let mut words = vec![pending.records.len() as u32, 0, 0, 0];
        words.extend(pending.records.iter().flatten());
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        queue.write_buffer(&edits.queue, 0, &bytes);
        Ok(())
    }
}
