//! One simulation step (m0_contrat.md §1.3.2): the passes P0–P9, in that order, each a fixed list of compute
//! dispatches built once for the state's world. A step records the lists as they are — it reads no wall clock, no
//! rung, no frame and no label — so the same state and step count give the same state, bit for bit (§1.3.4).
//!
//! Today P1, P8 and P9 hold dispatches: P1 and P9 the Δt and the non-finite guard (shaders/reduce.wgsl, [`dt`],
//! M0-T20), P8 the floors, then the species renormalisation (shaders/floors.wgsl, M0-T19); every other pass is an empty
//! slot its lot fills, in place, without reordering the others. A step knows its index — the guard runs on every 64th
//! (index ≡ 0 mod 64), and step 0 is preceded by the initial state's Δt reduction.
//!
//! Every pass runs inside the active box ([`boxfit`], §1.3.3, M0-T21): a box-sized dispatch takes its workgroup count
//! from the box's indirect sizes on the GPU ([`Size::Indirect`]), never from a readback. The box is fit before step 0
//! (ahead of the initial Δt reduction), after P9 at step indices ≡ 0 (mod 16), and in P0 after an edit outside it.
//!
//! A frame ([`latch`], §1.8.3, §1.8.5, M0-T22) records N steps under the latch: every dispatch of a step then takes its
//! workgroup count from a buffer the latch's controller can zero, and the guard's and the re-fit's schedule is the GPU's
//! step index, not this step's CPU count.
//!
//! The equation of state every pass shares (shaders/eos.wgsl) is prepended to a pass's shader ([`with_eos`]) and bound
//! as group 1 from [`EosGpu`], built once per run from physics.json and elements.json: the species table, the cold
//! pairs, M0-T12's u(x) table (sr_physics::eos::ColdTable) and the floors.

pub mod boxfit;
pub mod dt;
pub mod latch;

use std::cell::Cell;

use sr_physics::eos::{ColdTable, Eos, TABLE_POINTS, X_MIN};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

use crate::state::{Booking, State, BOOK_GROUPS, TERMS};
use boxfit::{BoxFit, BoxMode, BoxReport, ARGS_TILES, REFIT_EVERY};
use dt::{Dt, DtReport, GUARD_EVERY};
use latch::Latch;

/// The passes of one step, in §1.3.2's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pass {
    Edits,
    Dt,
    Gravity,
    Sinks,
    Hydro,
    Heat,
    Reactions,
    NeutrinoHeating,
    Floors,
    Reductions,
}

impl Pass {
    pub const ORDER: [Pass; 10] = [
        Pass::Edits,
        Pass::Dt,
        Pass::Gravity,
        Pass::Sinks,
        Pass::Hydro,
        Pass::Heat,
        Pass::Reactions,
        Pass::NeutrinoHeating,
        Pass::Floors,
        Pass::Reductions,
    ];

    /// The debugger's name for the pass, "P0 edits" … "P9 reductions".
    pub fn label(self) -> &'static str {
        match self {
            Pass::Edits => "P0 edits",
            Pass::Dt => "P1 dt",
            Pass::Gravity => "P2 gravity",
            Pass::Sinks => "P3 sinks",
            Pass::Hydro => "P4 hydro",
            Pass::Heat => "P5 heat",
            Pass::Reactions => "P6 reactions",
            Pass::NeutrinoHeating => "P7 neutrino heating",
            Pass::Floors => "P8 floors",
            Pass::Reductions => "P9 reductions",
        }
    }
}

/// One compute dispatch of a pass: its entry point, a pipeline, its bind groups (group g at index g) and its workgroup
/// count.
pub struct Dispatch {
    pub label: &'static str,
    pub pipeline: wgpu::ComputePipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub size: Size,
}

/// A dispatch's workgroup count: fixed when built, or read by the GPU from a buffer at a byte offset (the box's sizes).
#[derive(Clone, Debug)]
pub enum Size {
    Fixed([u32; 3]),
    Indirect(wgpu::Buffer, u64),
}

/// The step: one dispatch list per pass, indexed as `Pass::ORDER`, the Δt record, the active box, the latch and the
/// index of the next step.
pub struct Step {
    passes: [Vec<Dispatch>; 10],
    dt: Dt,
    boxfit: BoxFit,
    latch: Latch,
    next: Cell<u64>,
}

impl Step {
    /// Builds every pass's dispatch list for `state`'s buffers and world, booking into `booking`, with the equation of
    /// state `eos` and the run's constants `physics` (C, η_g, Σ_vac), the box fit to the state. The next step it
    /// records is step 0.
    pub fn new(device: &wgpu::Device, state: &State, booking: &Booking, eos: &EosGpu, physics: &Physics) -> Step {
        Step::with_box(device, state, booking, eos, physics, BoxMode::Fit)
    }

    /// As [`Step::new`], the box fit to the state or held at the world (the switch for tests, G-BOX).
    pub fn with_box(
        device: &wgpu::Device,
        state: &State,
        booking: &Booking,
        eos: &EosGpu,
        physics: &Physics,
        mode: BoxMode,
    ) -> Step {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("reduce.wgsl"),
            source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/reduce.wgsl")).into()),
        });
        let mut boxfit = BoxFit::new(device, state, physics, mode);
        let mut dt = Dt::new(device, &module, state, eos, physics, &boxfit);
        boxfit.build(device, &module, state, dt.record());
        let mut passes: [Vec<Dispatch>; 10] = Default::default();
        passes[Pass::Dt as usize] = std::mem::take(&mut dt.advance);
        passes[Pass::Floors as usize] = floors(device, state, booking, eos, &boxfit);
        passes[Pass::Reductions as usize] = std::mem::take(&mut dt.reduce);
        let latch = Latch::new(device, &passes, &dt, &boxfit);
        Step { passes, dt, boxfit, latch, next: Cell::new(0) }
    }

    /// How many dispatches each pass records on a step the guard skips, in `Pass::ORDER`.
    pub fn dispatch_counts(&self) -> [usize; 10] {
        std::array::from_fn(|p| self.passes[p].len())
    }

    /// The entry points a pass dispatches on a step the guard skips, in its order.
    pub fn dispatch_labels(&self, pass: Pass) -> Vec<&'static str> {
        self.passes[pass as usize].iter().map(|d| d.label).collect()
    }

    /// The entry points P9 adds after the Δt reduction on every 64th step: the non-finite guard.
    pub fn guard_labels(&self) -> Vec<&'static str> {
        self.dt.guard.iter().map(|d| d.label).collect()
    }

    /// The box's entry points: its re-fit (before step 0, and after P9 every 16th step), then P0's, after the edits.
    pub fn box_labels(&self) -> [Vec<&'static str>; 2] {
        [&self.boxfit.refit, &self.boxfit.on_edit].map(|list| list.iter().map(|d| d.label).collect())
    }

    /// Every dispatch of a step whose workgroup count the GPU reads from the box's sizes: (entry point, byte offset in
    /// [`BoxFit::args`]).
    pub fn box_sized(&self) -> Vec<(&'static str, u64)> {
        let all = self.passes.iter().flatten().chain(&self.dt.guard);
        all.filter_map(|d| match &d.size {
            Size::Indirect(b, offset) if b == self.boxfit.args() => Some((d.label, *offset)),
            _ => None,
        })
        .collect()
    }

    /// The box as the GPU last wrote it. Blocks on the device (tests and headless only) — never sizes a dispatch.
    pub fn read_box(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> BoxReport {
        self.boxfit.read(device, queue)
    }

    /// The box's edit flag, set by P0's edit pass when an edit lands outside the box ([`BoxFit::edit_flag`]).
    pub fn box_edit_flag(&self) -> &wgpu::Buffer {
        self.boxfit.edit_flag()
    }

    /// Records the next step: a compute pass per non-empty slot, P0 to P9 — before step 0, the initial state's Δt. The
    /// guard and the re-fit follow this step's CPU count; after a frame the latch stopped, that count runs ahead of the
    /// GPU's, so a run uses frames ([`Step::encode_frame`]) or bare steps, not both.
    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder) {
        self.encode_step(encoder, None);
    }

    /// One step, bare (`latch` none) or under the latch: then every fixed-size dispatch reads its workgroup count from
    /// the latch's sizes, the guard and the re-fit are recorded on every step and opened by the GPU's step index, and the
    /// controller closes P9.
    fn encode_step(&self, encoder: &mut wgpu::CommandEncoder, latch: Option<&Latch>) {
        let n = self.next.get();
        let record = |cpass: &mut wgpu::ComputePass<'_>, list: &[Dispatch], every: u64| match latch {
            Some(l) => l.record(cpass, list, every),
            None => record_dispatches(cpass, list),
        };
        if n == 0 {
            // The box fit to the initial state, then P9's Δt reduction over it; the reduce dispatches are P9's own list.
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("P9 reductions (initial state)"),
                timestamp_writes: None,
            });
            record(&mut cpass, &self.boxfit.refit, REFIT_EVERY);
            record(&mut cpass, &self.passes[Pass::Reductions as usize], 1);
        }
        for (pass, dispatches) in Pass::ORDER.iter().zip(&self.passes) {
            let hooked = latch.is_some_and(|l| l.hooked()) && *pass == Pass::Reactions;
            if dispatches.is_empty() && *pass != Pass::Edits && !hooked {
                continue;
            }
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(pass.label()), timestamp_writes: None });
            record(&mut cpass, dispatches, 1);
            if *pass == Pass::Edits {
                record(&mut cpass, &self.boxfit.on_edit, 1);
            }
            if let (true, Some(l)) = (hooked, latch) {
                l.record(&mut cpass, l.hook_dispatches(), 1);
            }
            if *pass == Pass::Reductions && (latch.is_some() || n.is_multiple_of(GUARD_EVERY)) {
                record(&mut cpass, &self.dt.guard, GUARD_EVERY);
            }
            if *pass == Pass::Reductions && (latch.is_some() || n.is_multiple_of(REFIT_EVERY)) {
                record(&mut cpass, &self.boxfit.refit, REFIT_EVERY);
            }
            if let (Pass::Reductions, Some(l)) = (*pass, latch) {
                record_dispatches(&mut cpass, std::slice::from_ref(l.controller()));
            }
        }
        self.next.set(n + 1);
    }

    /// The Δt record after every submitted step: Δt_n, Δt_{n+1}, the steps run and the guard's finding. Blocks on the
    /// device (native and headless only, §6.3).
    pub fn read_dt(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> DtReport {
        self.dt.read(device, queue)
    }

    /// The Δt record buffer, which later passes bind to read Δt_n (its first word, f32).
    pub fn dt_record(&self) -> &wgpu::Buffer {
        self.dt.record()
    }

    /// Records and submits `steps` steps, in order, in one submission.
    pub fn run(&self, device: &wgpu::Device, queue: &wgpu::Queue, steps: u32) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("step") });
        for _ in 0..steps {
            self.encode(&mut encoder);
        }
        queue.submit([encoder.finish()]);
    }
}

/// shaders/eos.wgsl, the helpers every pass shares.
pub const EOS_WGSL: &str = include_str!("../../shaders/eos.wgsl");

/// A pass's shader source with eos.wgsl prepended: its own bindings in group 0, the EOS's in group 1.
pub fn with_eos(source: &str) -> String {
    format!("{EOS_WGSL}\n{source}")
}

/// The size of eos.wgsl's `EosParams` uniform: two species arrays of three vec4, then three rows of four words.
const EOS_PARAMS_BYTES: usize = 2 * 48 + 3 * 16;

/// The equation of state on the GPU, uploaded once per run: `EosParams` (the species table's (Z + 1)/A and Z/A, the
/// cold pairs, the table's geometry, T_floor, Σ_floor, Σ_vac) and both cold tables, electrons then neutron matter, each
/// [`TABLE_POINTS`] ln u then as many segment slopes taken in f64 (zeros, and its flag 0, for a pair that is off).
pub struct EosGpu {
    params: wgpu::Buffer,
    table: wgpu::Buffer,
}

impl EosGpu {
    /// From a run's constants (a scene's `overrides` included) and the element registry. `physics` is validated, so
    /// each cold pair is on or off as a whole.
    pub fn new(device: &wgpu::Device, physics: &Physics, elements: &Elements) -> EosGpu {
        let eos = Eos::from_physics(physics).expect("validated physics.json: each cold pair is on or off");
        let get = |k: &str| physics.get(k).unwrap_or_else(|| panic!("physics.json key {k}")) as f32;
        let (mut inv_mu, mut y_e) = ([0f32; 12], [0f32; 12]);
        for (i, s) in elements.species().iter().enumerate().take(N_SPECIES) {
            let (a, z) = (f64::from(s.a), f64::from(s.z));
            inv_mu[i] = ((z + 1.0) / a) as f32;
            y_e[i] = (z / a) as f32;
        }
        let pair = |t: Option<&ColdTable>| t.map_or((0.0, 0.0, 0u32), |t| (t.k1() as f32, t.k2() as f32, 1));
        let (k1e, k2e, cold_e) = pair(eos.electron_table());
        let (k1n, k2n, cold_n) = pair(eos.neutron_table());
        let mut words: Vec<u32> = inv_mu.iter().chain(&y_e).map(|v| v.to_bits()).collect();
        words.extend([k1e, k2e, k1n, k2n].map(f32::to_bits));
        let geometry = [X_MIN.ln() as f32, (1.0 / ColdTable::step()) as f32, get("t_floor"), get("sigma_floor")];
        words.extend(geometry.map(f32::to_bits));
        words.extend([get("sigma_vac").to_bits(), cold_e, cold_n, TABLE_POINTS as u32]);
        assert_eq!(words.len() * 4, EOS_PARAMS_BYTES);

        let mut table: Vec<f32> = Vec::with_capacity(4 * TABLE_POINTS);
        for t in [eos.electron_table(), eos.neutron_table()] {
            match t {
                Some(t) => {
                    let ln_u = t.ln_u();
                    table.extend(ln_u.iter().map(|&v| v as f32));
                    table.extend(ln_u.windows(2).map(|w| (w[1] - w[0]) as f32));
                    table.push(0.0);
                }
                None => table.extend(std::iter::repeat_n(0.0, 2 * TABLE_POINTS)),
            }
        }
        let table: Vec<u32> = table.iter().map(|v| v.to_bits()).collect();
        EosGpu {
            params: init_buffer(device, "eos.params", wgpu::BufferUsages::UNIFORM, &words),
            table: init_buffer(device, "eos.table", wgpu::BufferUsages::STORAGE, &table),
        }
    }

    /// The group-1 bind group of a pipeline compiled with [`with_eos`].
    pub fn bind_group(&self, device: &wgpu::Device, pipeline: &wgpu::ComputePipeline) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("eos"),
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.params.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.table.as_entire_binding() },
            ],
        })
    }
}

/// A buffer created holding `words`, little-endian.
fn init_buffer(device: &wgpu::Device, label: &str, usage: wgpu::BufferUsages, words: &[u32]) -> wgpu::Buffer {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage,
        mapped_at_creation: true,
    });
    buffer.get_mapped_range_mut(..).expect("mapped at creation").copy_from_slice(&bytes);
    buffer.unmap();
    buffer
}

/// Sets each dispatch's pipeline and bind groups and dispatches it, in order.
fn record_dispatches(cpass: &mut wgpu::ComputePass<'_>, dispatches: &[Dispatch]) {
    for d in dispatches {
        record_sized(cpass, d, &d.size);
    }
}

/// Sets `d`'s pipeline and bind groups and dispatches it with `size` (its own, or the latch's in its place).
fn record_sized(cpass: &mut wgpu::ComputePass<'_>, d: &Dispatch, size: &Size) {
    cpass.set_pipeline(&d.pipeline);
    for (g, bind_group) in d.bind_groups.iter().enumerate() {
        cpass.set_bind_group(g as u32, bind_group, &[]);
    }
    match size {
        Size::Fixed([x, y, z]) => cpass.dispatch_workgroups(*x, *y, *z),
        Size::Indirect(buffer, offset) => cpass.dispatch_workgroups_indirect(buffer, *offset),
    }
}

fn compute_pipeline(device: &wgpu::Device, module: &wgpu::ShaderModule, entry: &'static str) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry),
        layout: None,
        module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
    })
}

/// P8 floors (§1.3.2): `floor_cells` — the vacuum reset and the temperature floor, booked into P8's booking buffer —
/// then `renormalise_species`, both over the box's 8 × 8 tiles.
fn floors(device: &wgpu::Device, state: &State, booking: &Booking, eos: &EosGpu, boxfit: &BoxFit) -> Vec<Dispatch> {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("floors.wgsl"),
        source: wgpu::ShaderSource::Wgsl(with_eos(include_str!("../../shaders/floors.wgsl")).into()),
    });
    let book = BOOK_GROUPS
        .iter()
        .position(|g| TERMS[g.terms.start].writer == Pass::Floors)
        .expect("a booking buffer written by P8");

    let floor_cells = compute_pipeline(device, &module, "floor_cells");
    let cells_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P8 floor_cells"),
        layout: &floor_cells.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: state.dims().as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: state.group(1).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: state.group(2).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: state.group(0).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: booking.group(book).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: boxfit.active().as_entire_binding() },
        ],
    });
    let eos_group = eos.bind_group(device, &floor_cells);

    let renormalise = compute_pipeline(device, &module, "renormalise_species");
    let species_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P8 renormalise_species"),
        layout: &renormalise.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: state.dims().as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: state.group(1).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: state.group(2).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: boxfit.active().as_entire_binding() },
        ],
    });
    let tiles = || Size::Indirect(boxfit.args().clone(), ARGS_TILES);

    vec![
        Dispatch { label: "floor_cells", pipeline: floor_cells, bind_groups: vec![cells_group, eos_group], size: tiles() },
        Dispatch { label: "renormalise_species", pipeline: renormalise, bind_groups: vec![species_group], size: tiles() },
    ]
}
