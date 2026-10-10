//! M0-T18 — the side fields and the booking layout (contract §2.2's last paragraph, §1.9.1, §2.9, §1.3.4): a test
//! pipeline adds known values to every slot of every term over 16 steps; each term's f64 total from `book_totals`
//! against the closed-form total, two runs bit-identical, the clear empties every term, and the layout names every
//! §2.9 term and §1.9.1 accumulator with its writer.
//!
//! The booked values are small integers times powers of two, so every per-slot f32 sum over the 16 steps is exact and
//! the oracle is exact too: the 10⁻⁹ relative tolerance then grades the layout and the f64 helper (a skipped row,
//! a term read from the wrong plane), never f32 rounding.

use super::device;
use sr_engine::state::{book_totals, Booking, Domain, SideFields, WorldConfig, BOOK_GROUPS, N_TERMS, TERMS};
use sr_engine::step::Pass;

const STEPS: u32 = 16;
const TOLERANCE: f64 = 1e-9;

/// One dispatch per booking buffer per step: every slot of every term in it adds `value(term, slot, step)`, one
/// invocation per slot, no atomic — the same read-modify-write of its own slot a pass does.
const BOOK_WGSL: &str = r#"
struct Step { step: u32, _a: u32, _b: u32, _c: u32 }
struct Group { first_term: u32, terms: u32, slots: u32, _pad: u32 }

@group(0) @binding(0) var<uniform> tick: Step;
@group(0) @binding(1) var<uniform> params: Group;
@group(0) @binding(2) var<storage, read_write> book: array<f32>;

@compute @workgroup_size(64)
fn book_known(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    let i = id.y * n.x * 64u + id.x;
    if (i >= params.terms * params.slots) {
        return;
    }
    let term = params.first_term + i / params.slots;
    let slot = i % params.slots;
    let k = i32((slot * 7u + term * 13u + tick.step * 5u) % 97u) - 40;
    book[i] += ldexp(f32(k), i32(term % 5u) - 6);
}
"#;

/// The value the shader adds, in f64 — the oracle's side of the same formula.
fn value(term: usize, slot: usize, step: u32) -> f64 {
    let k = ((slot * 7 + term * 13 + step as usize * 5) % 97) as i64 - 40;
    k as f64 * 2f64.powi((term % 5) as i32 - 6)
}

/// Each term's exact total after `steps` steps.
fn oracle(world: WorldConfig, steps: u32) -> [f64; N_TERMS] {
    let mut totals = [0f64; N_TERMS];
    for group in &BOOK_GROUPS {
        for t in group.terms.clone() {
            for slot in 0..group.domain.slots(world) {
                for s in 0..steps {
                    totals[t] += value(t, slot, s);
                }
            }
        }
    }
    totals
}

struct Pipeline {
    pipeline: wgpu::ComputePipeline,
    step: wgpu::Buffer,
    bind_groups: Vec<(wgpu::BindGroup, [u32; 3])>,
}

fn uniform(device: &wgpu::Device, label: &str, words: [u32; 4]) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    buffer.get_mapped_range_mut(..).expect("mapped at creation").copy_from_slice(&bytes);
    buffer.unmap();
    buffer
}

impl Pipeline {
    fn new(device: &wgpu::Device, booking: &Booking) -> Pipeline {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("booking test"),
            source: wgpu::ShaderSource::Wgsl(BOOK_WGSL.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("booking test"),
            layout: None,
            module: &module,
            entry_point: Some("book_known"),
            compilation_options: Default::default(),
            cache: None,
        });
        let step = uniform(device, "booking test step", [0; 4]);
        let world = booking.world();
        let bind_groups = BOOK_GROUPS
            .iter()
            .enumerate()
            .map(|(g, group)| {
                let terms = (group.terms.end - group.terms.start) as u32;
                let slots = group.domain.slots(world) as u32;
                let params = uniform(device, group.name, [group.terms.start as u32, terms, slots, 0]);
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(group.name),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: step.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: params.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 2, resource: booking.group(g).as_entire_binding() },
                    ],
                });
                let workgroups = (terms * slots).div_ceil(64);
                let x = workgroups.min(32768);
                (bind_group, [x, workgroups.div_ceil(x), 1])
            })
            .collect();
        Pipeline { pipeline, step, bind_groups }
    }

    /// Books `steps` steps, one submission each (the step index goes through the uniform).
    fn run(&self, device: &wgpu::Device, queue: &wgpu::Queue, steps: u32) {
        for s in 0..steps {
            let words = [s, 0, 0, 0];
            queue.write_buffer(&self.step, 0, &words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("booking") });
            {
                let mut cpass = encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("booking"), timestamp_writes: None });
                cpass.set_pipeline(&self.pipeline);
                for (bind_group, wg) in &self.bind_groups {
                    cpass.set_bind_group(0, bind_group, &[]);
                    cpass.dispatch_workgroups(wg[0], wg[1], wg[2]);
                }
            }
            queue.submit([encoder.finish()]);
        }
    }
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// A booking for the default 600 × 400 world after `STEPS` steps of the test pipeline, read back.
fn booked(gpu: &sr_engine::gpu::Gpu) -> (Booking, Pipeline, Vec<f32>) {
    let booking = Booking::new(&gpu.device, WorldConfig::default()).unwrap();
    let pipeline = Pipeline::new(&gpu.device, &booking);
    pipeline.run(&gpu.device, &gpu.queue, STEPS);
    let planes = booking.readback(&gpu.device, &gpu.queue);
    (booking, pipeline, planes)
}

#[test]
fn every_terms_f64_total_equals_the_expected_total_over_16_steps() {
    let gpu = device();
    let world = WorldConfig::default();
    let (_, _, planes) = booked(&gpu);
    let got = book_totals(world, &planes);
    let want = oracle(world, STEPS);
    let mut worst = 0f64;
    for (t, term) in TERMS.iter().enumerate() {
        assert!(want[t] != 0.0, "{}: the expected total is 0, a relative check means nothing", term.name);
        let rel = (got[t] - want[t]).abs() / want[t].abs();
        let (name, writer) = (term.name, term.writer.label());
        assert!(rel <= TOLERANCE, "{name} ({writer}): total {}, expected {} (relative {rel:e})", got[t], want[t]);
        worst = worst.max(rel);
    }
    let values = planes.len();
    println!("{N_TERMS} terms, {values} values, {STEPS} steps: worst relative error {worst:e} (tolerance {TOLERANCE:e})");
}

#[test]
fn two_runs_are_bit_identical() {
    let gpu = device();
    let (_, _, a) = booked(&gpu);
    let (_, _, b) = booked(&gpu);
    assert!(a.iter().any(|&v| v != 0.0), "nothing was booked: the case would pass with no pipeline at all");
    assert_eq!(bits(&a), bits(&b), "two runs of {STEPS} booking steps differ");
    let (ta, tb) = (book_totals(WorldConfig::default(), &a), book_totals(WorldConfig::default(), &b));
    assert_eq!(ta.map(f64::to_bits), tb.map(f64::to_bits), "the f64 totals of identical planes differ");
}

#[test]
fn the_clear_empties_every_term() {
    let gpu = device();
    let world = WorldConfig::default();
    let (booking, pipeline, before) = booked(&gpu);
    let totals = book_totals(world, &before);
    assert!(totals.iter().all(|&t| t != 0.0), "a term was empty before the clear: {totals:?}");

    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("clear") });
    booking.clear(&mut encoder);
    gpu.queue.submit([encoder.finish()]);
    let after = booking.readback(&gpu.device, &gpu.queue);
    assert_eq!(after.len(), before.len());
    if let Some(i) = after.iter().position(|v| v.to_bits() != 0) {
        panic!("value {i} is {} after the clear", after[i]);
    }
    assert!(book_totals(world, &after).iter().all(|&t| t.to_bits() == 0));

    // Booking resumes from zero: one more run gives the first run's planes again.
    pipeline.run(&gpu.device, &gpu.queue, STEPS);
    let again = booking.readback(&gpu.device, &gpu.queue);
    assert_eq!(bits(&again), bits(&before), "booking after the clear did not start from 0");
}

#[test]
fn the_layout_names_every_term_and_accumulator_with_its_writer() {
    // §2.9's booked terms and §1.9.1's frame accumulators, each with the pass §1.3.2 and §2.4–§2.8 name as its writer.
    let expected: &[(&str, Pass, bool)] = &[
        ("mass.painted", Pass::Edits, false),
        ("mass.erased", Pass::Edits, false),
        ("mass.preset_dropped", Pass::Edits, false),
        ("mass.cleared", Pass::Edits, false),
        ("energy.tools.painted", Pass::Edits, false),
        ("energy.tools.erased", Pass::Edits, false),
        ("energy.tools.heated", Pass::Edits, false),
        ("energy.tools.cooled", Pass::Edits, false),
        ("energy.tools.preset", Pass::Edits, false),
        ("energy.tools.cleared", Pass::Edits, false),
        ("energy.swallowed", Pass::Sinks, false),
        ("escaped.mass", Pass::Hydro, false),
        ("escaped.mom_x", Pass::Hydro, false),
        ("escaped.mom_y", Pass::Hydro, false),
        ("escaped.energy", Pass::Hydro, false),
        ("escaped.mass.x_H", Pass::Hydro, false),
        ("escaped.mass.x_He", Pass::Hydro, false),
        ("escaped.mass.x_C", Pass::Hydro, false),
        ("escaped.mass.x_O", Pass::Hydro, false),
        ("escaped.mass.x_Ne", Pass::Hydro, false),
        ("escaped.mass.x_Mg", Pass::Hydro, false),
        ("escaped.mass.x_Si", Pass::Hydro, false),
        ("escaped.mass.x_S", Pass::Hydro, false),
        ("escaped.mass.x_Fe", Pass::Hydro, false),
        ("escaped.mass.x_n", Pass::Hydro, false),
        ("radiated", Pass::Heat, true),
        ("nuclear.H", Pass::Reactions, true),
        ("nuclear.He", Pass::Reactions, true),
        ("nuclear.C", Pass::Reactions, true),
        ("nuclear.Ne", Pass::Reactions, true),
        ("nuclear.O", Pass::Reactions, true),
        ("nuclear.Si", Pass::Reactions, true),
        ("nuclear.N", Pass::Reactions, true),
        ("neutrino_lost", Pass::Reactions, true),
        ("neutrino_deposited", Pass::NeutrinoHeating, true),
        ("mass.vacuum_reset", Pass::Floors, false),
        ("energy.floor_added", Pass::Floors, false),
    ];
    let got: Vec<(&str, Pass, bool)> = TERMS.iter().map(|t| (t.name, t.writer, t.accumulator)).collect();
    assert_eq!(got, expected);
    assert_eq!(TERMS.iter().filter(|t| t.accumulator).count(), 10, "§1.9.1: radiated, 7 nuclear groups, 2 neutrino");

    // One writer per buffer, escaped on the boundary faces, every buffer within WebGPU's default binding size.
    let world = WorldConfig::default();
    let gpu = device();
    let booking = Booking::new(&gpu.device, world).unwrap();
    for (g, group) in BOOK_GROUPS.iter().enumerate() {
        let writers: Vec<Pass> = TERMS[group.terms.clone()].iter().map(|t| t.writer).collect();
        assert!(writers.windows(2).all(|w| w[0] == w[1]), "{}: several writers {writers:?}", group.name);
        let escaped = TERMS[group.terms.start].name.starts_with("escaped.");
        assert_eq!(group.domain == Domain::EdgeFaces, escaped, "{}", group.name);
        let planes = (group.terms.end - group.terms.start) as u64;
        assert_eq!(booking.group(g).size(), planes * group.domain.slots(world) as u64 * 4, "{}", group.name);
    }
    assert_eq!(Domain::EdgeFaces.slots(world), 2000);
    let rows = Domain::Cells.rows(world);
    assert_eq!((rows.len(), rows[0].clone(), rows[399].clone()), (400, 0..600, 239_400..240_000));

    // The side fields: φ, F (two components) and S_ν, a plane of the world's cells each, zeroed.
    let side = SideFields::new(&gpu.device, world).unwrap();
    let plane = world.cells() as u64 * 4;
    assert_eq!(
        (side.phi().size(), side.flux().size(), side.s_nu().size()),
        (plane, 2 * plane, plane),
        "φ, F, S_ν sizes"
    );
}
