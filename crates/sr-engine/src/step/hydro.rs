//! P4 hydro (m0_contrat.md §1.3.2, §2.3, §2.8; M0-T34), on shaders/hydro.wgsl: M0-T33b's twin
//! (sr_physics::reference::hydro) in f32 — unsplit MUSCL-Hancock, the monotonised-central limiter, HLL with Davis
//! speeds, species by the mass flux, eos.wgsl's pressure, two ghosts a side — bounce or leave (§2.8; M0-T35).
//!
//! Three dispatches: `hydro_update` over the box's 8 × 8 tiles ([`Size::Indirect`] at [`ARGS_TILES`]) writes the next
//! state into P4's own planes — two buffers of [`NEXT_PLANES`] planes in §2.2's order — from the state as the step found it, each
//! tile's predicted faces exchanged through 9,216 B of workgroup memory instead of stored (a stored face set is 60 words a
//! cell, past WebGPU's 128 MiB binding at §2.8's largest worlds; M0-D30); `hydro_edges`, one invocation per boundary face (a fixed size),
//! books what a leave edge lets out into the booking's `escaped` buffer from the same face flux the update applied;
//! `hydro_commit` copies the next state back over the box. The update binds the state's three buffers, the next
//! state's two, P2's g, P5's F and eos.wgsl's table: WebGPU's 8 storage buffers per stage (§2.2, §6.2.2), all of
//! them — hence the booking's own dispatch, the next state in two buffers of seven planes, not three as the state (M0-T36:
//! a slot for g; 7 planes at 2048 × 2048 still fit the 128 MiB binding), and the Δt record bound as a uniform (M0-T43: a
//! slot for F; P9 writes it in another dispatch, as the box fit writes the box).
//!
//! The edge mode is a uniform of P4's ([`Hydro::set_edge_mode`]): a write lands between two submissions, so a frame or a
//! batch of steps runs whole under one mode and the next under the new one (§3.1: at the next step boundary). The same
//! uniform says whether the step runs P2: gravity's source (§1.4.2; M0-T36) — the predictor's half kick and the update's
//! time-centred Σg and Σu·g, hydro.wgsl's header — reads P2's g when it does, and a placeholder is bound when it does
//! not. Likewise whether it runs P5: the radiation force f = κΣF/c_sb (§2.4.4; M0-T43) added to the update's momentum
//! from the F P5 left in `side.flux` at the end of the last step, with physics.json's κ₀, c_sb, κ_dust and T_dust in the
//! same uniform.

use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size};
use crate::scene::EdgeMode;
use crate::state::{Booking, State, WorldConfig, BOOK_GROUPS, N_CHANNELS, TERMS};

/// The boundary-face slots one `hydro_edges` workgroup covers.
const EDGE_GROUP: u32 = 64;
/// The planes in each of the next state's two buffers (hydro.wgsl `NEXT_LO`).
pub const NEXT_PLANES: usize = 7;

const _: () = {
    assert!(2 * NEXT_PLANES == N_CHANNELS);
    assert!(NEXT_PLANES as u64 * (WorldConfig::MAX as u64) * (WorldConfig::MAX as u64) * 4 <= 128 << 20);
};

/// P4: its dispatches, the edge mode's uniform and the rest of its words (whether the step runs P2 and P5, the radiation
/// force's constants).
pub struct Hydro {
    pub(super) dispatches: Vec<Dispatch>,
    edge: wgpu::Buffer,
    sources: [u32; 7],
}

impl Hydro {
    /// P4's three dispatches for `state`'s world over `boxfit`'s box under `edge`, reading Δt_n from the Δt record
    /// `dt_record` (its first word), gravity's source from P2's g `gravity` when the step runs P2, the radiation force from
    /// P5's F `flux` (side.flux) when it runs P5 with `physics`' constants, and booking escaped into `booking`; the
    /// next-state planes live as long as the bind groups that hold them.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        boxfit: &BoxFit,
        dt_record: &wgpu::Buffer,
        gravity: Option<&wgpu::Buffer>,
        flux: Option<&wgpu::Buffer>,
        edge: EdgeMode,
    ) -> Hydro {
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let sources = [
            u32::from(gravity.is_some()),
            u32::from(flux.is_some()),
            0,
            get("kappa0").to_bits(),
            get("c_sb").to_bits(),
            get("kappa_dust").to_bits(),
            get("t_dust").to_bits(),
        ];
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let mode = init_buffer(device, "hydro.edge", usage, &words(edge, &sources));
        let (no_gravity, no_flux);
        let g = match gravity {
            Some(g) => g,
            None => {
                no_gravity = init_buffer(device, "hydro.no_gravity", wgpu::BufferUsages::STORAGE, &[0, 0]);
                &no_gravity
            }
        };
        let flux = match flux {
            Some(f) => f,
            None => {
                no_flux = init_buffer(device, "hydro.no_flux", wgpu::BufferUsages::STORAGE, &[0, 0]);
                &no_flux
            }
        };
        let dispatches = dispatches(device, state, booking, eos, boxfit, dt_record, g, flux, &mode);
        Hydro { dispatches, edge: mode, sources }
    }

    /// Sets the edge mode for every step submitted after this call (`queue.write_buffer`: ahead of the next submission).
    pub fn set_edge_mode(&self, queue: &wgpu::Queue, edge: EdgeMode) {
        let bytes: Vec<u8> = words(edge, &self.sources).iter().flat_map(|w| w.to_le_bytes()).collect();
        queue.write_buffer(&self.edge, 0, &bytes);
    }
}

/// The edge uniform's words (hydro.wgsl `Edges`): leave 1, bounce 0; then the sources' words — 1 when the step runs P2,
/// 1 when it runs P5, a pad, κ₀, c_sb, κ_dust and T_dust as f32 bits.
fn words(edge: EdgeMode, sources: &[u32; 7]) -> [u32; 8] {
    let mut w = [0; 8];
    w[0] = u32::from(edge == EdgeMode::Leave);
    w[1..].copy_from_slice(sources);
    w
}

#[allow(clippy::too_many_arguments)]
fn dispatches(
    device: &wgpu::Device,
    state: &State,
    booking: &Booking,
    eos: &EosGpu,
    boxfit: &BoxFit,
    dt_record: &wgpu::Buffer,
    gravity: &wgpu::Buffer,
    flux: &wgpu::Buffer,
    edge: &wgpu::Buffer,
) -> Vec<Dispatch> {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hydro.wgsl"),
        source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/hydro.wgsl")).into()),
    });
    let cells = state.world().cells() as u64;
    let next: [wgpu::Buffer; 2] = std::array::from_fn(|g| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(["hydro.next.lo", "hydro.next.hi"][g]),
            size: NEXT_PLANES as u64 * cells * 4,
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
            entry(6, flux),
            entry(7, boxfit.active()),
            entry(8, dt_record),
            entry(13, edge),
            entry(14, gravity),
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
            entry(14, gravity),
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
