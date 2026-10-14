//! P6 reactions (m0_contrat.md §1.3.2, §1.6, §2.5.1–§2.5.3, §1.8.5, §1.9.1; M0-T46, M0-T47), on shaders/react.wgsl: M0-T44b's
//! and M0-T45's twin (sr_physics::reference::react) in f32 — each box cell burns alone over Δt_n by linearised backward-Euler sub-cycles under
//! `burn_dx` and `burn_dt`, the heat of every sub-step into the cell's energy and into its group's `nuclear.<group>` slot.
//!
//! The registry's enabled records ride one uniform with the run's constants: each record's law (inputs and orders, A, a, q,
//! T_thr, the gate, the terms, ν by species) and M0-T44a's 256-node ln f table ([`Omega::ln_f_nodes`], f64 rounded once to
//! f32) — so P6 binds seven storage buffers (S_ν's plane the last) and the EOS's table, WebGPU's eight (§6.2.2). Three
//! dispatches: `react_begin` (one invocation) clears the step's counters, `react_cells` runs over the box's 8 × 8 tiles
//! ([`Size::Indirect`] at [`ARGS_TILES`]), `react_latch` (one invocation) hands the latch bit on. The counters
//! ([`Step::read_react`]) say how many sub-steps the step took and whether a cell ran out of attempts — the shader's header
//! has its f32 guards.
//!
//! With the scene's `neutrinos` switch on ([`ReactSetup::neutrinos`]) each sub-step also takes ε_ν from the cell, books
//! `neutrino_lost` and leaves S_ν in `side.s_nu`; a cell whose N_Fe rate is > 0 sets the collapse latch (§1.8.5): the cells
//! pass ORs a bit into the report and `react_latch`, built once the step's latch exists ([`React::latch_dispatch`]), ORs it
//! into the latch record's flag word — `react_cells` has no binding left for the flag itself.

use sr_physics::rates::{Omega, TABLE_POINTS};
use sr_physics::registry::{Physics, Reactions, GROUPS, N_SPECIES};

use super::boxfit::{BoxFit, ARGS_TILES};
use super::{compute_pipeline, init_buffer, with_eos, Dispatch, EosGpu, Pass, Size, Step};
use crate::scene::TestSettings;
use crate::state::{read_buffers, Booking, SideFields, State, BOOK_GROUPS, TERMS};

/// react.wgsl's sizes: records, inputs and terms per record.
pub const MAX_RECORDS: usize = 16;
const MAX_INPUTS: usize = 4;
const MAX_TERMS: usize = 4;
/// `ReactParams`: its header, then the records (12 vec4 each), then the tables.
const HEADER_WORDS: usize = 12;
const RECORD_WORDS: usize = 48;
const REPORT_WORDS: usize = 8;
const PIN_TEMPERATURE: u32 = 1;
const NEUTRINOS: u32 = 2;
const HAS_TERMS: u32 = 1;
const GATED: u32 = 2;
const LATCHES: u32 = 4;
/// §1.6.2's record whose rate > 0 sets the latch (the twin's own constant).
const NFE: &str = "N_Fe";

/// What P6 needs beyond the state: the registry parsed against the run's constants, the scene's test flags, its
/// `neutrinos` switch and the side fields S_ν goes to (written only with the switch on).
#[derive(Clone, Copy)]
pub struct ReactSetup<'a> {
    pub reactions: &'a Reactions,
    pub test: TestSettings,
    pub neutrinos: bool,
    pub side: &'a SideFields,
}

/// P6's dispatches and its counters.
pub struct React {
    pub(super) dispatches: Vec<Dispatch>,
    report: wgpu::Buffer,
    module: wgpu::ShaderModule,
}

/// P6's counters after the last step run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReactReport {
    /// Sub-steps accepted over every cell, and retried at half size.
    pub substeps: u32,
    pub retried: u32,
    /// The most sub-steps one cell took, in the step and in any step since the step was built.
    pub max_substeps: u32,
    pub worst: u32,
    /// Cells that ran out of attempts with time left, in the step and since the step was built.
    pub unfinished: u32,
    pub unfinished_run: u32,
}

/// The uniform's words: the header, then `MAX_RECORDS` records, then each record's table.
fn params_words(physics: &Physics, reactions: &Reactions, test: TestSettings, neutrinos: bool) -> Vec<u32> {
    let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
    let recs: Vec<_> = reactions.records().iter().filter(|r| r.enabled).collect();
    assert!(recs.len() <= MAX_RECORDS, "react.wgsl holds {MAX_RECORDS} enabled records, the registry {}", recs.len());
    let f = |v: f64| (v as f32).to_bits();
    let flags = u32::from(test.pin_temperature) * PIN_TEMPERATURE + u32::from(neutrinos) * NEUTRINOS;
    let mut words = vec![recs.len() as u32, flags, get("burn_dx").to_bits(), get("burn_dt").to_bits()];
    words.extend([f(1.0 / Omega::step()), f((TABLE_POINTS - 1) as f64), 0, 0]);
    words.extend([get("a_nu").to_bits(), get("t_nu").to_bits(), get("m_nu").to_bits(), 0]);
    assert_eq!(words.len(), HEADER_WORDS);
    let mut tables = vec![0u32; MAX_RECORDS * TABLE_POINTS];
    for (r, rec) in recs.iter().enumerate() {
        let rate = &rec.rate;
        assert!(rec.inputs.len() <= MAX_INPUTS && rate.terms.len() <= MAX_TERMS, "{}: react.wgsl's record is too small", rec.id);
        let group = GROUPS.iter().position(|g| *g == rec.group).expect("a record's group is one of GROUPS (registry load)");
        let flags = u32::from(rate.t_thr.is_some()) * HAS_TERMS + u32::from(rate.gate.is_some()) * GATED + u32::from(rec.id == NFE) * LATCHES;
        let mut w = vec![group as u32, rec.inputs.len() as u32, rate.terms.len() as u32, flags];
        let (mut species, mut orders) = ([0u32; MAX_INPUTS], [0u32; MAX_INPUTS]);
        for (k, (s, &n)) in rec.inputs.iter().zip(&rate.orders).enumerate() {
            (species[k], orders[k]) = (s.species as u32, f(n));
        }
        w.extend(species.into_iter().chain(orders));
        w.extend([f(rate.a_coef), f(rate.a), f(rec.q(physics)), f(rate.t_thr.unwrap_or(0.0))]);
        let (sigma_g, p) = rate.gate.as_ref().map_or((0.0, 0.0), |g| (g.sigma_g, g.p));
        w.extend([f(sigma_g), f(p), 0, 0]);
        for k in 0..MAX_TERMS {
            w.extend(rate.terms.get(k).map_or([0; 4], |t| [f(t.t_k), f(t.nu), f(t.w), 0]));
        }
        let mut nu = [0.0f64; 12];
        rec.outputs.iter().for_each(|s| nu[s.species] += s.share);
        rec.inputs.iter().for_each(|s| nu[s.species] -= s.share);
        w.extend(nu.map(f));
        assert_eq!(w.len(), RECORD_WORDS);
        words.extend(w);
        for (slot, v) in tables[r * TABLE_POINTS..].iter_mut().zip(Omega::new(rec).ln_f_nodes()) {
            *slot = f(*v);
        }
    }
    words.resize(HEADER_WORDS + MAX_RECORDS * RECORD_WORDS, 0);
    words.extend(tables);
    const _: () = assert!(N_SPECIES <= 12);
    words
}

impl React {
    /// P6 for `state`'s world over `boxfit`'s box, reading Δt_n from `dt_record`, booking `nuclear.<group>` into
    /// `booking`, with the constants of `physics` and the enabled records of `setup.reactions`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        boxfit: &BoxFit,
        dt_record: &wgpu::Buffer,
        setup: &ReactSetup<'_>,
    ) -> React {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("react.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/react.wgsl")).into()),
        });
        let params = init_buffer(device, "react.params", wgpu::BufferUsages::UNIFORM, &params_words(physics, setup.reactions, setup.test, setup.neutrinos));
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        let report = init_buffer(device, "react.report", storage, &[0; REPORT_WORDS]);
        let book = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].writer == Pass::Reactions).expect("a booking buffer written by P6");
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
                8 => &report,
                _ => setup.side.s_nu(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: resource(b).as_entire_binding() }).collect();
            let mut bind_groups =
                vec![device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some(label), layout: &pipeline.get_bind_group_layout(0), entries: &entries })];
            if label == "react_cells" {
                bind_groups.push(eos.bind_group(device, &pipeline));
            }
            Dispatch { label, pipeline, bind_groups, size }
        };
        let dispatches = vec![
            dispatch("react_begin", &[8], Size::Fixed([1, 1, 1])),
            dispatch("react_cells", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 10], Size::Indirect(boxfit.args().clone(), ARGS_TILES)),
        ];
        React { dispatches, report, module }
    }

    /// `react_latch`: after the cells, ORs the step's latch bit into `flag`'s first word (the latch record, §1.8.5). Built
    /// apart from [`React::new`] because the latch is made from the step's finished pass lists; the caller appends it to
    /// P6's.
    pub(super) fn latch_dispatch(&self, device: &wgpu::Device, flag: &wgpu::Buffer) -> Dispatch {
        let pipeline = compute_pipeline(device, &self.module, "react_latch");
        let entries = [
            wgpu::BindGroupEntry { binding: 8, resource: self.report.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 9, resource: flag.as_entire_binding() },
        ];
        let bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("react_latch"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        })];
        Dispatch { label: "react_latch", pipeline, bind_groups, size: Size::Fixed([1, 1, 1]) }
    }
}

impl Step {
    /// P6's counters after every submitted step; none when the step runs no P6. Blocks on the device (tests and headless).
    pub fn read_react(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<ReactReport> {
        let react = self.react.as_ref()?;
        let w: Vec<u32> = read_buffers(device, queue, "react.report", std::iter::once(&react.report)).iter().map(|v| v.to_bits()).collect();
        Some(ReactReport { substeps: w[0], retried: w[1], max_substeps: w[2], unfinished: w[3], worst: w[4], unfinished_run: w[5] })
    }
}
