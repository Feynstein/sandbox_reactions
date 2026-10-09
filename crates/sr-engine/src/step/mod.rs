//! One simulation step (m0_contrat.md §1.3.2): the passes P0–P9, in that order, each a fixed list of compute
//! dispatches built once for the state's world. A step records the lists as they are — it reads no wall clock, no
//! rung, no frame and no label — so the same state and step count give the same state, bit for bit (§1.3.4).
//!
//! Today only P8 holds a dispatch, the species renormalisation (shaders/floors.wgsl); every other pass is an empty
//! slot its lot fills, in place, without reordering the others.

use crate::state::State;

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

/// One compute dispatch of a pass: a pipeline, its bindings and a fixed workgroup count.
pub struct Dispatch {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub workgroups: [u32; 3],
}

/// The step: one dispatch list per pass, indexed as `Pass::ORDER`.
pub struct Step {
    passes: [Vec<Dispatch>; 10],
}

impl Step {
    /// Builds every pass's dispatch list for `state`'s buffers and world.
    pub fn new(device: &wgpu::Device, state: &State) -> Step {
        let mut passes: [Vec<Dispatch>; 10] = Default::default();
        passes[Pass::Floors as usize] = floors(device, state);
        Step { passes }
    }

    /// How many dispatches each pass records, in `Pass::ORDER`.
    pub fn dispatch_counts(&self) -> [usize; 10] {
        std::array::from_fn(|p| self.passes[p].len())
    }

    /// Records one step: a compute pass per non-empty slot, P0 to P9.
    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder) {
        for (pass, dispatches) in Pass::ORDER.iter().zip(&self.passes) {
            if dispatches.is_empty() {
                continue;
            }
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(pass.label()), timestamp_writes: None });
            for d in dispatches {
                cpass.set_pipeline(&d.pipeline);
                cpass.set_bind_group(0, &d.bind_group, &[]);
                cpass.dispatch_workgroups(d.workgroups[0], d.workgroups[1], d.workgroups[2]);
            }
        }
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

/// Workgroups covering the world with the shaders' 8 × 8 tiles (the world is a multiple of 8, §2.8).
fn tiles(state: &State) -> [u32; 3] {
    let world = state.world();
    [world.width.div_ceil(8), world.height.div_ceil(8), 1]
}

/// P8 floors: the species renormalisation (M0-T19 puts the vacuum reset and the temperature floor ahead of it).
fn floors(device: &wgpu::Device, state: &State) -> Vec<Dispatch> {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("floors.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/floors.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("P8 renormalise_species"),
        layout: None,
        module: &module,
        entry_point: Some("renormalise_species"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("P8 renormalise_species"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: state.dims().as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: state.group(1).as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: state.group(2).as_entire_binding() },
        ],
    });
    vec![Dispatch { pipeline, bind_group, workgroups: tiles(state) }]
}
