//! P4 hydro (m0_contrat.md §1.3.2, §2.3, §2.8; M0-T34), on shaders/hydro.wgsl: M0-T33b's twin
//! (sr_physics::reference::hydro) in f32 — unsplit MUSCL-Hancock, the monotonised-central limiter, HLL with Davis
//! speeds, species by the mass flux, eos.wgsl's pressure, two ghosts a side — bounce or leave (§2.8; M0-T35).
//!
//! Three dispatches: `hydro_update` over the box's 8 × 8 tiles ([`Size::Indirect`] at [`ARGS_TILES`]) writes the next
//! state into P4's own planes — packed as the state's three buffers — from the state as the step found it, each
//! cell recomputing its neighbours' predicted faces instead of storing them (a stored face set is 60 words a cell, past
//! WebGPU's 128 MiB binding at §2.8's largest worlds); `hydro_edges`, one invocation per boundary face (a fixed size),
//! books what a leave edge lets out into the booking's `escaped` buffer from the same face flux the update applied;
//! `hydro_commit` copies the next state back over the box. The update binds the state's three buffers, the next
//! state's three, the Δt record and eos.wgsl's table: WebGPU's 8 storage buffers per stage (§2.2, §6.2.2), all of them —
//! hence the booking's own dispatch, which binds six.
//!
//! The edge mode is a uniform of P4's ([`Hydro::set_edge_mode`]): a write lands between two submissions, so a frame or a
//! batch of steps runs whole under one mode and the next under the new one (§3.1: at the next step boundary).
//!
//! Gravity's source (§1.4.2) is M0-T36's, the radiation force (§2.4.4) M0-T43's.

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size};
use crate::scene::EdgeMode;
use crate::state::{Booking, State, BOOK_GROUPS, GROUPS, GROUP_NAMES, TERMS};

/// The boundary-face slots one `hydro_edges` workgroup covers.
const EDGE_GROUP: u32 = 64;

/// P4: its dispatches and the edge mode's uniform.
pub struct Hydro {
    pub(super) dispatches: Vec<Dispatch>,
    edge: wgpu::Buffer,
}

impl Hydro {
    /// P4's three dispatches for `state`'s world over `boxfit`'s box under `edge`, reading Δt_n from the Δt record
    /// `dt_record` (its first word) and booking escaped into `booking`; the next-state planes live as long as the bind
    /// groups that hold them.
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        boxfit: &BoxFit,
        dt_record: &wgpu::Buffer,
        edge: EdgeMode,
    ) -> Hydro {
        let mode = init_buffer(device, "hydro.edge", wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, &words(edge));
        Hydro { dispatches: dispatches(device, state, booking, eos, boxfit, dt_record, &mode), edge: mode }
    }

    /// Sets the edge mode for every step submitted after this call (`queue.write_buffer`: ahead of the next submission).
    pub fn set_edge_mode(&self, queue: &wgpu::Queue, edge: EdgeMode) {
        let bytes: Vec<u8> = words(edge).iter().flat_map(|w| w.to_le_bytes()).collect();
        queue.write_buffer(&self.edge, 0, &bytes);
    }
}

/// The edge uniform's words (hydro.wgsl `Edges`): leave 1, bounce 0.
fn words(edge: EdgeMode) -> [u32; 4] {
    [u32::from(edge == EdgeMode::Leave), 0, 0, 0]
}

fn dispatches(
    device: &wgpu::Device,
    state: &State,
    booking: &Booking,
    eos: &EosGpu,
    boxfit: &BoxFit,
    dt_record: &wgpu::Buffer,
    edge: &wgpu::Buffer,
) -> Vec<Dispatch> {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hydro.wgsl"),
        source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/hydro.wgsl")).into()),
    });
    let cells = state.world().cells() as u64;
    let next: [wgpu::Buffer; 3] = std::array::from_fn(|g| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("hydro.next.{}", GROUP_NAMES[g].trim_start_matches("state."))),
            size: (GROUPS[g].end - GROUPS[g].start) as u64 * cells * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        })
    });

    let update = compute_pipeline(device, &module, "hydro_update");
    let update_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P4 hydro_update"),
        layout: &update.get_bind_group_layout(0),
        entries: &[
            entry(0, state.dims()),
            entry(1, state.group(0)),
            entry(2, state.group(1)),
            entry(3, state.group(2)),
            entry(4, &next[0]),
            entry(5, &next[1]),
            entry(6, &next[2]),
            entry(7, boxfit.active()),
            entry(8, dt_record),
            entry(13, edge),
        ],
    });
    let eos_group = eos.bind_group(device, &update);

    let book = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == Pass::Hydro).expect("a booking buffer written by P4");
    let edges = compute_pipeline(device, &module, "hydro_edges");
    let edges_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P4 hydro_edges"),
        layout: &edges.get_bind_group_layout(0),
        entries: &[
            entry(0, state.dims()),
            entry(1, state.group(0)),
            entry(2, state.group(1)),
            entry(3, state.group(2)),
            entry(7, boxfit.active()),
            entry(8, dt_record),
            entry(12, booking.group(book)),
            entry(13, edge),
        ],
    });
    let edges_eos = eos.bind_group(device, &edges);
    let world = state.world();
    let faces = 2 * (world.width + world.height);

    let commit = compute_pipeline(device, &module, "hydro_commit");
    let commit_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P4 hydro_commit"),
        layout: &commit.get_bind_group_layout(0),
        entries: &[
            entry(0, state.dims()),
            entry(4, &next[0]),
            entry(5, &next[1]),
            entry(6, &next[2]),
            entry(7, boxfit.active()),
            entry(9, state.group(0)),
            entry(10, state.group(1)),
            entry(11, state.group(2)),
        ],
    });
    let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);
    vec![
        Dispatch { label: "hydro_update", pipeline: update, bind_groups: vec![update_group, eos_group], size: tiles() },
        Dispatch {
            label: "hydro_edges",
            pipeline: edges,
            bind_groups: vec![edges_group, edges_eos],
            size: Size::Fixed([faces.div_ceil(EDGE_GROUP), 1, 1]),
        },
        Dispatch { label: "hydro_commit", pipeline: commit, bind_groups: vec![commit_group], size: tiles() },
    ]
}

/// A bind group entry binding the whole of `buffer`.
fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource: buffer.as_entire_binding() }
}
