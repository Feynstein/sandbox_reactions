//! P4 hydro (m0_contrat.md §1.3.2, §2.3, §2.8; M0-T34), on shaders/hydro.wgsl: M0-T33b's twin
//! (sr_physics::reference::hydro) in f32 — unsplit MUSCL-Hancock, the monotonised-central limiter, HLL with Davis
//! speeds, species by the mass flux, eos.wgsl's pressure, two ghosts a side — bounce or leave (§2.8; M0-T35).
//!
//! Three dispatches, `hydro_push_sum` between the last two (below): `hydro_update` over the box's 8 × 8 tiles ([`Size::Indirect`] at [`ARGS_TILES`]) writes the next
//! state into P4's own planes — two buffers of [`NEXT_PLANES`] planes in §2.2's order — from the state as the step found it, each
//! tile's predicted faces exchanged through 9,216 B of workgroup memory instead of stored (a stored face set is 60 words a
//! cell, past WebGPU's 128 MiB binding at §2.8's largest worlds; M0-D30); `hydro_edges`, one invocation per boundary face (a fixed size),
//! books what a leave edge lets out — and [M0-D194] a bounce wall's pressure impulse — into the booking's `escaped` buffer
//! from the same face flux the update applied;
//! `hydro_commit` copies the next state back over the box — after, when the step runs P5, `hydro_push_sum` (one workgroup)
//! has summed the radiation force's pushes the update wrote over F, and the commit takes the net back (§2.4.4 [M0-D184]);
//! [M0-D194] when it runs P2, the same workgroup sums gravity's kicks exactly (fixed point) and the commit takes their net
//! back from the densest free cell (§1.4.2 [M0-D194]); [M0-D195] on every step it sums the mass update's rounding exactly
//! — Σⁿ⁺¹ − Σⁿ over the box and what the faces let out, plus the last step's carry — and the commit gives it back to every
//! free cell in proportion to its Σ; `hydro_settle` (one workgroup, last) sums exactly what f32 kept of the shares and
//! carries the rest to the next step ([`Hydro::mass_carry`], a dump's `resume.mass_carry`). The update binds the state's three buffers, the next
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
//!
//! The half step (§1.4.2 [M0-D183], [M0-D192]): with P2, two dispatches come first — `hydro_update` under the mid pass's
//! own uniform (`edge_mid`: the edge words with `mid` 1 and `radiation` 0), which writes the midpoint of the step's own
//! mass update, ½(Σⁿ + Σⁿ⁺¹), into the next state's Σ plane and takes nothing else, then `hydro_mid`, which writes it, 0
//! below Σ_vac, into P2's field (its plane 10, [`super::grav::FIELD_PLANES`]) — the source P2's half-step chains solve
//! ([`super::grav::Gravity::take_mid`], joined after `hydro_mid` by step/mod.rs) and the update's kick multiplies. One more
//! flux pass a step: the midpoint of the mass HLL moves is what makes the faces' work move W exactly (hydro.wgsl's header).
//!
//! The hold (§2.3.4 [M0-TG2]; M0-D71c): the three dispatches read the mask the last P8 wrote (`State::held`) as a
//! sampled texture, outside the storage limit; `hydro_commit` books what a held cell was spared into the booking's `held`
//! buffer (`booking.held`), P4's second, and stops a free cell's momentum into a held face-neighbour, booked there too
//! (the sticky wall, §2.3.4 [M0-D172]).

use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size};
use crate::scene::EdgeMode;
use crate::state::{read_buffers, Booking, State, WorldConfig, BOOK_GROUPS, N_CHANNELS, TERMS};

/// The boundary-face slots one `hydro_edges` workgroup covers.
const EDGE_GROUP: u32 = 64;
/// [M0-D195] `push_net`'s mass carry (hydro.wgsl MASS_CARRY) and its first boundary-face word (FACE_MASS).
const MASS_CARRY_WORD: usize = 10;
const FACE_MASS_WORD: usize = 12;
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
    /// The mid pass's edge uniform (§1.4.2 [M0-D192]): the same edge mode, `mid` 1, `radiation` 0.
    edge_mid: wgpu::Buffer,
    sources: [u32; 7],
    /// `hydro_push_sum`'s and the commit's words (hydro.wgsl `push_net`), [M0-D195] the mass carry among them.
    push_net: wgpu::Buffer,
}

impl Hydro {
    /// P4's three dispatches for `state`'s world over `boxfit`'s box under `edge`, reading Δt_n from the Δt record
    /// `dt_record` (its first word), gravity's source from P2's g `gravity` when the step runs P2 — [M0-D194] its net
    /// summed on the gas's own field `gas` ([`Gravity::gas_field`](super::grav::Gravity::gas_field)) —, the radiation force from
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
        gravity: Option<(&wgpu::Buffer, &wgpu::Buffer)>,
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
        let mode_mid = init_buffer(device, "hydro.edge_mid", usage, &mid_words(edge, &sources));
        let (no_gravity, no_flux);
        let g = match gravity {
            Some(g) => g,
            None => {
                no_gravity = init_buffer(device, "hydro.no_gravity", wgpu::BufferUsages::STORAGE, &[0, 0]);
                (&no_gravity, &no_gravity)
            }
        };
        let flux = match flux {
            Some(f) => f,
            None => {
                no_flux = init_buffer(device, "hydro.no_flux", wgpu::BufferUsages::STORAGE, &[0, 0]);
                &no_flux
            }
        };
        let (dispatches, push_net) =
            dispatches(device, state, booking, eos, boxfit, dt_record, g, flux, [&mode, &mode_mid], gravity.is_some());
        Hydro { dispatches, edge: mode, edge_mid: mode_mid, sources, push_net }
    }

    /// [M0-D195] The mass the last step's give-back left to carry (hydro.wgsl MASS_CARRY): a dump's `resume.mass_carry`.
    /// Blocks on the device (tests and headless runs only).
    pub fn mass_carry(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> f32 {
        let words = read_buffers(device, queue, "hydro.push_net.readback", std::iter::once(&self.push_net));
        words[MASS_CARRY_WORD]
    }

    /// [M0-D195] Restores the carry a dump holds, for the next step's give-back (`queue.write_buffer`: ahead of the next
    /// submission).
    pub fn set_mass_carry(&self, queue: &wgpu::Queue, carry: f32) {
        queue.write_buffer(&self.push_net, 4 * MASS_CARRY_WORD as u64, &carry.to_le_bytes());
    }

    /// Sets the edge mode for every step submitted after this call (`queue.write_buffer`: ahead of the next submission).
    pub fn set_edge_mode(&self, queue: &wgpu::Queue, edge: EdgeMode) {
        let bytes = |w: [u32; 8]| w.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>();
        queue.write_buffer(&self.edge, 0, &bytes(words(edge, &self.sources)));
        queue.write_buffer(&self.edge_mid, 0, &bytes(mid_words(edge, &self.sources)));
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

/// The mid pass's words (§1.4.2 [M0-D192]): [`words`] with `radiation` 0 (no force, F untouched) and `mid` 1.
fn mid_words(edge: EdgeMode, sources: &[u32; 7]) -> [u32; 8] {
    let mut w = words(edge, sources);
    w[2] = 0;
    w[3] = 1;
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
    (gravity, gas): (&wgpu::Buffer, &wgpu::Buffer),
    flux: &wgpu::Buffer,
    [edge, edge_mid]: [&wgpu::Buffer; 2],
    half_step: bool,
) -> (Vec<Dispatch>, wgpu::Buffer) {
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
    let update_group_with = |label: &str, edge: &wgpu::Buffer| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
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
                held(15, state),
            ],
        })
    };
    let update_group = update_group_with("P4 hydro_update", edge);
    let eos_group = eos.bind_group(device, &update);

    let book = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == Pass::Hydro).expect("a booking buffer written by P4");
    let world = state.world();
    let faces = 2 * (world.width + world.height);
    // [M0-D184] The radiation force's net push and gross over the box, (N_x, N_y, S, 0), then [M0-D194] gravity's net kick
    // and the densest free cell, (K_x, K_y, its index or −1, 0): `hydro_push_sum`'s, the commit's. [M0-D199] K less what f32
    // drops of the push's give-back, which `hydro_push_sum` reads off the next state's momentum (`next[0]`). [M0-D195] Then
    // the mass given back, its cell, the carry and a pad, and one word per boundary face, the mass `hydro_edges` let out:
    // the carry, [`MASS_CARRY_WORD`], is read and restored with a dump's records.
    let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
    let push_net = init_buffer(device, "hydro.push_net", usage, &vec![0; FACE_MASS_WORD + faces as usize]);
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
            held(15, state),
            entry(18, &push_net),
        ],
    });
    let edges_eos = eos.bind_group(device, &edges);

    let push_sum = compute_pipeline(device, &module, "hydro_push_sum");
    let push_sum_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P4 hydro_push_sum"),
        layout: &push_sum.get_bind_group_layout(0),
        entries: &[
            entry(0, state.dims()),
            entry(1, state.group(0)),
            entry(4, &next[0]),
            entry(6, flux),
            entry(7, boxfit.active()),
            entry(8, dt_record),
            entry(13, edge),
            entry(14, gravity),
            held(15, state),
            entry(18, &push_net),
            entry(19, gas),
        ],
    });

    let held_book = BOOK_GROUPS.iter().position(|g| g.name == "booking.held").expect("the booking buffer of P4's held cells");
    let commit = compute_pipeline(device, &module, "hydro_commit");
    let commit_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P4 hydro_commit"),
        layout: &commit.get_bind_group_layout(0),
        entries: &[
            entry(0, state.dims()),
            entry(4, &next[0]),
            entry(5, &next[1]),
            entry(6, flux),
            entry(7, boxfit.active()),
            entry(9, state.group(0)),
            entry(10, state.group(1)),
            entry(11, state.group(2)),
            entry(13, edge),
            held(15, state),
            entry(16, booking.group(held_book)),
            entry(18, &push_net),
        ],
    });
    // The sticky wall reads Σ_vac (§2.3.4 [M0-D172]): the parameters alone, no cold table.
    let commit_eos = eos.params_bind_group(device, &commit);
    // [M0-D195] What f32 kept of the give-back's shares, after the commit: the carry into `push_net`.
    let settle = compute_pipeline(device, &module, "hydro_settle");
    let settle_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P4 hydro_settle"),
        layout: &settle.get_bind_group_layout(0),
        entries: &[
            entry(0, state.dims()),
            entry(4, &next[0]),
            entry(7, boxfit.active()),
            entry(9, state.group(0)),
            held(15, state),
            entry(18, &push_net),
        ],
    });
    let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);
    // Σ_src at the half step into P2's field (§1.4.2 [M0-D183], [M0-D192]), with P2 only: the mid pass — the update's own
    // fluxes, ½(Σⁿ + Σⁿ⁺¹) into the next state's Σ plane —, then `hydro_mid` from that plane.
    let mid = half_step.then(|| {
        let pass = Dispatch {
            label: "hydro_update_mid",
            pipeline: update.clone(),
            bind_groups: vec![update_group_with("P4 hydro_update (mid pass)", edge_mid), eos_group.clone()],
            size: tiles(),
        };
        let pipeline = compute_pipeline(device, &module, "hydro_mid");
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("P4 hydro_mid"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[entry(0, state.dims()), entry(4, &next[0]), entry(7, boxfit.active()), entry(17, gravity)],
        });
        let eos_group = eos.params_bind_group(device, &pipeline);
        [pass, Dispatch { label: "hydro_mid", pipeline, bind_groups: vec![group, eos_group], size: tiles() }]
    });
    let dispatches = mid.into_iter().flatten().chain([
        Dispatch { label: "hydro_update", pipeline: update, bind_groups: vec![update_group, eos_group], size: tiles() },
        Dispatch {
            label: "hydro_edges",
            pipeline: edges,
            bind_groups: vec![edges_group, edges_eos],
            size: Size::Fixed([faces.div_ceil(EDGE_GROUP), 1, 1]),
        },
        Dispatch { label: "hydro_push_sum", pipeline: push_sum, bind_groups: vec![push_sum_group], size: Size::Fixed([1, 1, 1]) },
        Dispatch { label: "hydro_commit", pipeline: commit, bind_groups: vec![commit_group, commit_eos], size: tiles() },
        Dispatch { label: "hydro_settle", pipeline: settle, bind_groups: vec![settle_group], size: Size::Fixed([1, 1, 1]) },
    ])
    .collect();
    (dispatches, push_net)
}

/// A bind group entry binding the whole of `buffer`.
fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource: buffer.as_entire_binding() }
}

/// A bind group entry binding the hold's mask (`State::held`), read as a sampled texture.
fn held(binding: u32, state: &State) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource: wgpu::BindingResource::TextureView(state.held()) }
}
