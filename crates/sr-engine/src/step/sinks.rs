//! P3 sinks (m0_contrat.md §1.3.2, §1.3.4, §2.6, §2.9; M0-T51), on shaders/sinks.wgsl: M0-T50's twin
//! (sr_physics::reference::sinks, `Sinks::step`) in f32, every reading it declared kept — a sink's position in cells with
//! cell (i, j) centred at (i + ½, j + ½); the share 1 − Σ_floor/Σ of a cell's Σ, momentum and E taken; accretion sink by
//! sink in index order, each sink's M and v frozen for its own pass; the world's outer ring never forming; formation's φ
//! with the sinks' own term; merging the lowest pair first, strictly inside the larger r_acc, into the lower slot.
//!
//! The sinks live on the GPU only, in one small buffer ([`Sinks`]'s state: the header, 16 sinks, then a candidate slot per
//! 8 × 8 tile of the world), so a frame runs P3 with no CPU round trip. Six dispatches, each box-sized ([`Size::Indirect`]
//! at [`ARGS_TILES`]) or fixed ([`Size::Fixed`]), so a frame's latch reaches them: `sink_candidates` over the box's tiles,
//! each tile's best candidate by a fixed tree; `sink_update`, one workgroup, the tiles' best by a fixed-order scan and
//! tree, the formation, and accretion per sink with the taken mass and momentum summed by a fixed tree; then the
//! kick-drift-kick of §0.5 [M0-V7] (M0-D46) — `sink_pull` over the box's tiles (each tile's pull on each sink from its
//! cells with Σ ≥ Σ_vac, the reaction of the sink's softened pull, by a fixed tree), `sink_kick_drift` (the tiles summed
//! in a fixed order, the pair's pull softened by the larger r_acc, the half-kick and the drift), `sink_pull` again at the
//! drifted positions, `sink_kick_merge` (the closing half-kick, the merging). No atomic: the same state gives the same
//! bits (§1.3.4). The gas's thermal and cold energy a sink takes is booked `energy.swallowed` at the giving cell
//! (state.rs's booking.sinks, M0-T18).
//!
//! The motion reads no grid g: the box holds every cell with Σ ≥ Σ_vac (§1.3.3), so the sum reaches a sink outside it.
//! φ is P2's plane, which carries the sinks' potentials (`grav_sinks`, §1.4.3, M0-T52a) for formation; a step without P2
//! has no sinks' term in it (the tests' synthetic worlds upload φ by hand). A formation sets the collapse
//! latch (§1.8.5): `sink_latch`, built apart ([`Sinks::latch_dispatch`]) because the latch is made from the finished pass
//! lists. A dump carries the sinks in its header (dump.rs, [`Step::read_sinks`] and [`Step::set_sinks`], M0-T52b).

use sr_physics::registry::Physics;

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size, Step};
use crate::dump::{Sink, MAX_SINKS};
use crate::state::{read_buffers, Booking, SideFields, State, BOOK_GROUPS, TERMS};

/// The state buffer's words: an 8-word header (count, formed + 1, merged, a pad; accreted, three pads), 16 sinks of 8
/// (x, y, vx, vy, m, m_lo, two pads), then two per tile.
const HEADER_WORDS: usize = 8;
const SINK_WORDS: usize = 8;
const SINKS_WORDS: usize = HEADER_WORDS + MAX_SINKS * SINK_WORDS;
/// The pull buffer's head (sinks.wgsl `PULL_TILES`): each slot's acceleration at the step's start and at its drifted
/// position, two words each; then two words per sink per tile.
const PULL_HEAD_WORDS: usize = 2 * 2 * MAX_SINKS;

/// What P3 needs beyond the state: the side fields whose φ it reads (P2's, or a test's).
#[derive(Clone, Copy)]
pub struct SinkSetup<'a> {
    pub side: &'a SideFields,
}

/// P3's dispatches and the sinks' buffer.
pub struct Sinks {
    pub(super) dispatches: Vec<Dispatch>,
    state: wgpu::Buffer,
    pull: wgpu::Buffer,
    module: wgpu::ShaderModule,
    width: u32,
}

/// The sinks after the last step run, and what that step did.
#[derive(Clone, Debug, PartialEq)]
pub struct SinkReport {
    /// In slot order; m is the f32 pair's sum.
    pub sinks: Vec<Sink>,
    /// The cell a sink formed at, (x, y).
    pub formed: Option<(u32, u32)>,
    /// The mass accretion gave the sinks.
    pub accreted: f64,
    /// Sinks lost to merging.
    pub merged: u32,
    /// Each sink's acceleration in the step's kick-drift-kick (§0.5 [M0-V7]: the gas's pull and the other sinks'), at its
    /// start position and at its drifted one, by the slot it held before merging; zeros after [`Step::set_sinks`].
    pub pulls: Vec<[[f64; 2]; 2]>,
}

/// The sinks' state buffer for `state`'s world, empty: made before P2 and P3 so both bind it (P2 adds the sinks' potential
/// and pull from it, §1.4.3).
pub(super) fn state_buffer(device: &wgpu::Device, state: &State) -> wgpu::Buffer {
    let world = state.world();
    let tiles = world.width.div_ceil(8) * world.height.div_ceil(8);
    let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
    init_buffer(device, "sinks.state", storage, &vec![0; SINKS_WORDS + 2 * tiles as usize])
}

impl Sinks {
    /// P3 for `state`'s world over `boxfit`'s box, the sinks living in `sinks` ([`state_buffer`]), reading Δt_n from
    /// `dt_record`, φ from `setup.side`, booking into `booking`, with Σ_BH, c_sb and Σ_vac from `physics` and G = 1.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        boxfit: &BoxFit,
        dt_record: &wgpu::Buffer,
        sinks: wgpu::Buffer,
        setup: &SinkSetup<'_>,
    ) -> Sinks {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sinks.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/sinks.wgsl")).into()),
        });
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let world = state.world();
        let tiles = world.width.div_ceil(8) * world.height.div_ceil(8);
        let c_sb = get("c_sb");
        let params = [(super::grav::G as f32).to_bits(), get("sigma_bh").to_bits(), (c_sb * c_sb).to_bits(), tiles, get("sigma_vac").to_bits(), 0, 0, 0];
        let params = init_buffer(device, "sinks.params", wgpu::BufferUsages::UNIFORM, &params);
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let pull = init_buffer(device, "sinks.pull", storage, &vec![0; PULL_HEAD_WORDS + 2 * MAX_SINKS * tiles as usize]);
        let book = BOOK_GROUPS.iter().position(|b| TERMS[b.terms.start].writer == Pass::Sinks).expect("a booking buffer written by P3");
        let resource = |binding: u32| -> &wgpu::Buffer {
            match binding {
                0 => state.dims(),
                1 => state.group(0),
                2 => state.group(1),
                3 => state.group(2),
                4 => booking.group(book),
                5 => boxfit.active(),
                6 => dt_record,
                7 => &params,
                8 => &sinks,
                9 => setup.side.phi(),
                _ => &pull,
            }
        };
        // `with_eos`: the entry point reads the EOS (group 1); the motion's read Σ_vac from the params instead.
        let dispatch = |label: &'static str, bindings: &[u32], with_eos: bool, size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: resource(b).as_entire_binding() }).collect();
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some(label), layout: &pipeline.get_bind_group_layout(0), entries: &entries });
            let mut bind_groups = vec![group];
            if with_eos {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size }
        };
        let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);
        let dispatches = vec![
            dispatch("sink_candidates", &[0, 1, 2, 3, 5, 7, 8, 9], true, tiles()),
            dispatch("sink_update", &[0, 1, 2, 3, 4, 5, 7, 8], true, Size::Fixed([1, 1, 1])),
            dispatch("sink_pull", &[0, 1, 5, 7, 8, 10], false, tiles()),
            dispatch("sink_kick_drift", &[5, 6, 7, 8, 10], false, Size::Fixed([1, 1, 1])),
            dispatch("sink_pull", &[0, 1, 5, 7, 8, 10], false, tiles()),
            dispatch("sink_kick_merge", &[5, 6, 7, 8, 10], false, Size::Fixed([1, 1, 1])),
        ];
        Sinks { dispatches, state: sinks, pull, module, width: world.width }
    }

    /// `sink_latch`: after `sink_update`, ORs a formation's latch bit into `flag`'s first word (the latch record, §1.8.5).
    /// Built apart from [`Sinks::new`] because the latch is made from the step's finished pass lists; the caller appends it
    /// to P3's.
    pub(super) fn latch_dispatch(&self, device: &wgpu::Device, flag: &wgpu::Buffer) -> Dispatch {
        let pipeline = compute_pipeline(device, &self.module, "sink_latch");
        let entries = [
            wgpu::BindGroupEntry { binding: 8, resource: self.state.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 11, resource: flag.as_entire_binding() },
        ];
        // group 0 only: the EOS table's group 1 is not used by this entry point, so its auto layout leaves it out
        let bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sink_latch"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        })];
        Dispatch { label: "sink_latch", pipeline, bind_groups, size: Size::Fixed([1, 1, 1]) }
    }
}

impl Step {
    /// Whether the step runs P3, and so holds sinks ([`Step::set_sinks`], [`Step::read_sinks`]).
    pub fn runs_sinks(&self) -> bool {
        self.sinks.is_some()
    }

    /// Replaces the sinks with `sinks`, in slot order, at the next step boundary (a write lands between submissions);
    /// refused past 16 or when the step runs no P3. The step's report is cleared.
    pub fn set_sinks(&self, queue: &wgpu::Queue, sinks: &[Sink]) -> Result<(), String> {
        let p3 = self.sinks.as_ref().ok_or("the step runs no P3")?;
        if sinks.len() > MAX_SINKS {
            return Err(format!("{} sinks, at most {MAX_SINKS}", sinks.len()));
        }
        let mut words = vec![0u32; SINKS_WORDS];
        words[0] = sinks.len() as u32;
        for (k, s) in sinks.iter().enumerate() {
            let m = s.m as f32;
            let rec = [s.x as f32, s.y as f32, s.vx as f32, s.vy as f32, m, (s.m - f64::from(m)) as f32, 0.0, 0.0];
            words[HEADER_WORDS + k * SINK_WORDS..][..SINK_WORDS].copy_from_slice(&rec.map(f32::to_bits));
        }
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        queue.write_buffer(&p3.state, 0, &bytes);
        queue.write_buffer(&p3.pull, 0, &[0; 4 * PULL_HEAD_WORDS]);
        Ok(())
    }

    /// The sinks after every submitted step and what the last step's P3 did; none when the step runs no P3. Blocks on
    /// the device (tests and headless).
    pub fn read_sinks(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<SinkReport> {
        let p3 = self.sinks.as_ref()?;
        let v = read_buffers(device, queue, "sinks.read", [&p3.state, &p3.pull].into_iter());
        let pull = &v[(p3.state.size() / 4) as usize..];
        let word = |i: usize| v[i].to_bits();
        let count = (word(0) as usize).min(MAX_SINKS);
        let sinks = (0..count)
            .map(|k| {
                let r = |i: usize| f64::from(v[HEADER_WORDS + k * SINK_WORDS + i]);
                Sink { x: r(0), y: r(1), vx: r(2), vy: r(3), m: r(4) + r(5) }
            })
            .collect();
        let formed = word(1).checked_sub(1).map(|c| (c % p3.width, c / p3.width));
        let at = |i: usize| [f64::from(pull[2 * i]), f64::from(pull[2 * i + 1])];
        let pulls = (0..count).map(|k| [at(k), at(MAX_SINKS + k)]).collect();
        Some(SinkReport { sinks, formed, accreted: f64::from(v[4]), merged: word(2), pulls })
    }
}
