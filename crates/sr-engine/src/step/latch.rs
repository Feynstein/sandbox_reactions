//! A frame of steps and the collapse latch (m0_contrat.md §1.8.3, §1.8.5, §1.8.7, §3.1; TW-R4, TW-E16), on
//! shaders/latch.wgsl.
//!
//! [`Step::encode_frame`] records a plan's N steps in one encoder — one submission — between `latch_begin` and
//! `latch_end`, the controller (`latch_check`) closing each step's P9. Every dispatch of a step reads its workgroup count
//! from a buffer the controller can zero: a fixed-size one from its slot in [`Latch::args`] (one slot per size and
//! schedule — the guard's every 64 steps and the re-fit's every 16 open on the GPU's step index), a box-sized one from
//! the box's sizes, P0's re-fit from its gate. [`Latch::new`] refuses a step holding any other indirect buffer: a pass
//! that adds one adds it to latch.wgsl first, or the latch would not reach it.
//!
//! The flag is the latch record's first word ([`Step::latch_flag`]): P6 and P3 set it with atomicOr (their lots). When an
//! armed latch is set with the slow-down on, no later step of the frame runs — the box re-fit's dispatches included — and
//! the frame's report says so; the CPU then drops the rung to ×0.1 (sr_physics::time). [`Step::poll`] returns each
//! submitted frame's report once, from a readback mapped on submit, and never waits for the GPU.

use std::cell::{Cell, RefCell};
use std::sync::mpsc::{channel, Receiver, Sender};

use super::boxfit::{BoxFit, REFIT_EVERY};
use super::dt::{Dt, GUARD_EVERY};
use super::{compute_pipeline, init_buffer, record_sized, Dispatch, Size, Step};
use crate::state::read_buffers;

/// latch.wgsl's `LatchState`: flag, slow_down, stopped, steps_run, fired_at, since, advanced, a pad word, the 12 saved
/// sizes, 4 pad words.
const RECORD_WORDS: usize = 24;
const SLOW_DOWN_BYTE: u64 = 4;
/// The window the latch re-arms after (§1.8.5), in t.u.: `since` starts there, so a new latch is armed.
const WINDOW: f32 = 1.0;
/// reduce.wgsl's `DtState`, copied after the latch record into each readback.
const DT_WORDS: usize = 8;
const READBACK_BYTES: u64 = ((RECORD_WORDS + DT_WORDS) * 4) as u64;

/// What a frame is asked to run: its steps (§1.8.3's N) and whether the slow-down is on (§1.8.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FramePlan {
    pub steps: u32,
    pub slow_down: bool,
}

/// A submitted frame, as read back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameReport {
    /// Steps the frame ran: its plan's, or fewer when the latch fired.
    pub steps_run: u32,
    /// The sim time the frame advanced: its steps' Δt summed in order, f32.
    pub advanced: f32,
    /// The sim time of every frame reported so far, summed in f64.
    pub sim_time: f64,
    /// Δt of the frame's last step run, and the next step's (§1.3.2 P1, P9).
    pub dt: f32,
    pub dt_next: f32,
    /// Steps begun since the state was made (the Δt record's count).
    pub steps: u64,
    /// The frame's step (1 for its first) after which the latch fired; none when it did not.
    pub latch_fired: Option<u32>,
}

/// The latch's buffers and dispatches, built once for a step.
pub struct Latch {
    record: wgpu::Buffer,
    args: wgpu::Buffer,
    /// (size, period) per slot of `args`, in slot order; period 0 runs on every step.
    slots: Vec<([u32; 3], u64)>,
    /// The slow-down word, off then on, copied into the record by each frame.
    switch: [wgpu::Buffer; 2],
    hook: wgpu::Buffer,
    hooked: Cell<bool>,
    begin: Dispatch,
    check: Dispatch,
    end: Dispatch,
    /// P6 under the hook: `latch_hook`, `latch_probe` (sized by the box).
    hook_dispatches: Vec<Dispatch>,
    dt_record: wgpu::Buffer,
    /// For the readback buffers: one per frame in flight, made when none is free.
    device: wgpu::Device,
    readbacks: RefCell<Readbacks>,
}

struct Readbacks {
    slots: Vec<(wgpu::Buffer, Option<u64>)>,
    sent: Sender<(usize, Result<(), wgpu::BufferAsyncError>)>,
    ready: Receiver<(usize, Result<(), wgpu::BufferAsyncError>)>,
    frames: u64,
    sim_time: f64,
}

impl Latch {
    /// The latch for a step made of `passes`, `dt`'s guard and `boxfit`'s re-fits: one slot per fixed size and schedule,
    /// every indirect buffer the step reads checked to be one the controller zeroes.
    pub(super) fn new(device: &wgpu::Device, passes: &[Vec<Dispatch>; 10], dt: &Dt, boxfit: &BoxFit) -> Latch {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("latch.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/latch.wgsl").into()),
        });
        let every_step = passes.iter().flatten().chain(&boxfit.on_edit);
        let lists = every_step.map(|d| (d, 0)).chain(dt.guard.iter().map(|d| (d, GUARD_EVERY))).chain(boxfit.refit.iter().map(|d| (d, REFIT_EVERY)));
        let mut slots: Vec<([u32; 3], u64)> = vec![([1, 1, 1], 0)];
        let mut indirect: Vec<wgpu::Buffer> = Vec::new();
        for (d, every) in lists {
            match &d.size {
                Size::Fixed(s) if !slots.contains(&(*s, every)) => slots.push((*s, every)),
                Size::Indirect(b, _) if !indirect.contains(b) => indirect.push(b.clone()),
                _ => {}
            }
        }
        assert!(
            indirect.len() == 2 && indirect.contains(boxfit.args()) && indirect.iter().all(|b| b.size() == 24),
            "the latch zeroes exactly the box's sizes and P0's gate (latch.wgsl bindings 4, 5); this step reads {indirect:?}"
        );
        let gate = indirect.iter().find(|b| *b != boxfit.args()).expect("P0's gate").clone();

        let storage = wgpu::BufferUsages::STORAGE;
        let mut record_words = [0u32; RECORD_WORDS];
        record_words[5] = WINDOW.to_bits();
        let record = init_buffer(device, "latch.record", storage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, &record_words);
        let size_words: Vec<u32> = slots.iter().flat_map(|(s, every)| [s[0], s[1], s[2], *every as u32]).collect();
        let sizes = init_buffer(device, "latch.sizes", storage, &size_words);
        let open: Vec<u32> = slots.iter().flat_map(|(s, _)| *s).collect();
        let args = init_buffer(device, "latch.args", storage | wgpu::BufferUsages::INDIRECT, &open);
        let switch = [0, 1].map(|on| init_buffer(device, "latch.switch", wgpu::BufferUsages::COPY_SRC, &[on]));
        let hook = init_buffer(device, "latch.hook", storage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, &[1, 0, 1, 0]);

        let entry = |binding: u32| -> wgpu::BindingResource<'_> {
            match binding {
                0 => record.as_entire_binding(),
                1 => args.as_entire_binding(),
                2 => sizes.as_entire_binding(),
                3 => dt.record().as_entire_binding(),
                4 => boxfit.args().as_entire_binding(),
                5 => gate.as_entire_binding(),
                _ => hook.as_entire_binding(),
            }
        };
        let dispatch = |label: &'static str, bindings: &[u32], size: Size| {
            let pipeline = compute_pipeline(device, &module, label);
            let entries: Vec<wgpu::BindGroupEntry> =
                bindings.iter().map(|&b| wgpu::BindGroupEntry { binding: b, resource: entry(b) }).collect();
            let bind_groups = vec![device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &entries,
            })];
            Dispatch { label, pipeline, bind_groups, size }
        };
        let one = || Size::Fixed([1, 1, 1]);
        let (sent, ready) = channel();
        Latch {
            begin: dispatch("latch_begin", &[0, 1, 2, 3], one()),
            check: dispatch("latch_check", &[0, 1, 2, 3, 4, 5], one()),
            end: dispatch("latch_end", &[0, 4, 5], one()),
            hook_dispatches: vec![
                dispatch("latch_hook", &[0, 3, 6], one()),
                dispatch("latch_probe", &[6], Size::Indirect(boxfit.args().clone(), super::boxfit::ARGS_TILES)),
            ],
            dt_record: dt.record().clone(),
            device: device.clone(),
            readbacks: RefCell::new(Readbacks { slots: Vec::new(), sent, ready, frames: 0, sim_time: 0.0 }),
            hooked: Cell::new(false),
            record,
            args,
            slots,
            switch,
            hook,
        }
    }

    /// Records `list` with each fixed size read from its slot (`every` its schedule: 1 or 0 for every step, else the
    /// period); an indirect size is recorded as it is — its buffer is one the controller zeroes.
    pub(super) fn record(&self, cpass: &mut wgpu::ComputePass<'_>, list: &[Dispatch], every: u64) {
        let every = if every == 1 { 0 } else { every };
        for d in list {
            match &d.size {
                Size::Fixed(s) => {
                    let slot = self.slots.iter().position(|x| *x == (*s, every)).expect("a slot for every fixed size");
                    record_sized(cpass, d, &Size::Indirect(self.args.clone(), slot as u64 * 12));
                }
                Size::Indirect(..) => record_sized(cpass, d, &d.size),
            }
        }
    }

    /// The controller, recorded with its own fixed size after each step's P9: it runs whether or not the latch fired.
    pub(super) fn controller(&self) -> &Dispatch {
        &self.check
    }

    pub(super) fn hooked(&self) -> bool {
        self.hooked.get()
    }

    pub(super) fn hook_dispatches(&self) -> &[Dispatch] {
        &self.hook_dispatches
    }
}

impl Step {
    /// Records a frame (§1.8.3, §1.8.5, §3.1): `plan.steps` steps under the latch, in `encoder` — one submission — then
    /// the frame's report, read back when the submission completes ([`Step::poll`]).
    pub fn encode_frame(&self, plan: &FramePlan, encoder: &mut wgpu::CommandEncoder) {
        let latch = &self.latch;
        encoder.copy_buffer_to_buffer(&latch.switch[usize::from(plan.slow_down)], 0, &latch.record, SLOW_DOWN_BYTE, 4);
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("frame begin"), timestamp_writes: None });
            super::record_dispatches(&mut cpass, std::slice::from_ref(&latch.begin));
        }
        for _ in 0..plan.steps {
            self.encode_step(encoder, Some(latch));
        }
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("frame end"), timestamp_writes: None });
            super::record_dispatches(&mut cpass, std::slice::from_ref(&latch.end));
        }

        let mut rb = latch.readbacks.borrow_mut();
        let frame = rb.frames;
        rb.frames += 1;
        let i = match rb.slots.iter().position(|(_, f)| f.is_none()) {
            Some(i) => i,
            None => {
                let buffer = latch.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("latch.readback"),
                    size: READBACK_BYTES,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                rb.slots.push((buffer, None));
                rb.slots.len() - 1
            }
        };
        let (buffer, busy) = &mut rb.slots[i];
        *busy = Some(frame);
        let buffer = buffer.clone();
        encoder.copy_buffer_to_buffer(&latch.record, 0, &buffer, 0, (RECORD_WORDS * 4) as u64);
        encoder.copy_buffer_to_buffer(&latch.dt_record, 0, &buffer, (RECORD_WORDS * 4) as u64, (DT_WORDS * 4) as u64);
        let sent = rb.sent.clone();
        encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., move |r| {
            let _ = sent.send((i, r));
        });
    }

    /// The reports of the frames whose readback has arrived since the last call, in submission order. Never blocks:
    /// it polls the device without waiting (§3.1).
    pub fn poll(&self, device: &wgpu::Device) -> Vec<FrameReport> {
        let _ = device.poll(wgpu::PollType::Poll);
        let mut rb = self.latch.readbacks.borrow_mut();
        let mut got: Vec<(u64, [u32; RECORD_WORDS + DT_WORDS])> = Vec::new();
        while let Ok((i, result)) = rb.ready.try_recv() {
            let (buffer, busy) = &mut rb.slots[i];
            let frame = busy.take().expect("a readback in flight");
            result.unwrap_or_else(|e| panic!("latch readback: map failed: {e}"));
            let mut w = [0u32; RECORD_WORDS + DT_WORDS];
            {
                let view = buffer.get_mapped_range(..).expect("mapped for reading");
                for (k, b) in view.as_chunks::<4>().0.iter().enumerate() {
                    w[k] = u32::from_le_bytes(*b);
                }
            }
            buffer.unmap();
            got.push((frame, w));
        }
        got.sort_by_key(|(frame, _)| *frame);
        got.into_iter()
            .map(|(_, w)| {
                let advanced = f32::from_bits(w[6]);
                rb.sim_time += f64::from(advanced);
                let d = &w[RECORD_WORDS..];
                FrameReport {
                    steps_run: w[3],
                    advanced,
                    sim_time: rb.sim_time,
                    dt: f32::from_bits(d[0]),
                    dt_next: f32::from_bits(d[1]),
                    steps: u64::from(d[3]),
                    latch_fired: (w[2] != 0).then_some(w[4]),
                }
            })
            .collect()
    }

    /// The latch record: its first word is the flag a pass sets with atomicOr (P6, P3 — §1.8.5).
    pub fn latch_flag(&self) -> &wgpu::Buffer {
        &self.latch.record
    }

    /// The test hook: from the next frame on, P6 sets the flag on the steps `lo`, `lo + stride`, … below `hi` (GPU step
    /// indices; step 0 is the first ever), and counts the box-sized dispatches that run ([`Step::latch_probe`]); none
    /// takes the hook out of the frame.
    pub fn set_latch_hook(&self, queue: &wgpu::Queue, hook: Option<(u32, u32, u32)>) {
        let (lo, hi, stride) = hook.unwrap_or((1, 0, 1));
        let bytes: Vec<u8> = [lo, hi, stride].iter().flat_map(|w| w.to_le_bytes()).collect();
        queue.write_buffer(&self.latch.hook, 0, &bytes);
        self.latch.hooked.set(hook.is_some());
    }

    /// The box-sized hook dispatches run since the step was made. Blocks on the device (tests only).
    pub fn latch_probe(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> u32 {
        read_buffers(device, queue, "latch.probe", std::iter::once(&self.latch.hook))[3].to_bits()
    }
}
