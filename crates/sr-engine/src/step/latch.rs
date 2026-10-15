//! A frame of steps and the collapse latch (m0_contrat.md §1.8.3, §1.8.5, §1.8.7, §3.1; TW-R4, TW-E16), on
//! shaders/latch.wgsl.
//!
//! [`Step::encode_frame`] records a plan's N steps in one encoder — one submission — between `latch_begin` and
//! `latch_end`, the controller (`latch_check`) closing each step's P9. Every dispatch of a step reads its workgroup count
//! from a buffer the controller can zero: a fixed-size one from its slot in [`Latch::args`] (one slot per size and
//! schedule — the guard's every 64 steps and the re-fit's every 16 open on the GPU's step index), a box-sized one from
//! the box's sizes, P0's re-fit from its gate. [`Latch::new`] refuses a step holding any other indirect buffer: a pass
//! that adds one adds it to latch.wgsl first, or the latch would not reach it — a [`Size::Chained`] chain excepted, which
//! runs only when its pass's own fixed-size opener starts it.
//!
//! The flag is the latch record's first word ([`Step::latch_flag`]): P6 and P3 set it with atomicOr (their lots). When an
//! armed latch is set with the slow-down on, no later step of the frame runs — the box re-fit's dispatches included — and
//! the frame's report says so; the CPU then drops the rung to ×0.1 (sr_physics::time). [`Step::poll`] returns each
//! submitted frame's report once, from a readback mapped on submit, and never waits for the GPU. With timing on
//! ([`Step::set_timing`], M0-T23) a frame is one timing batch: its resolved timestamps ride in the same readback, after
//! the latch record, the Δt record and P5's most slots a step used, and the report carries the frame's cost per step and
//! per pass.
//!
//! P5's carry (§0.5 [M0-TJ4] (6), M0-T121): a step with P5 records `latch_hold` before P0 and `latch_carry` after the
//! slots P5 records, frames and bare runs alike ([`Step::run`] records its submissions under the latch, the slow-down
//! off). A step still in its rounds there is carried — the rest of the submission zeroed, the record's `held` set — and
//! the next submission's first step continues it; a frame's report says so, and a dump is refused until it has ended.
//!
//! The block summary (§1.9.1, [`crate::summary`], M0-T56) closes every frame, after `latch_end` and outside the frame's
//! timing: its accumulators cleared at the next frame's start, its bytes in the frame's readback after the timestamps'
//! room, so [`Step::poll_frames`] returns it with the frame's report.

use std::cell::{Cell, RefCell};
use std::sync::mpsc::{channel, Receiver, Sender};

use super::boxfit::{BoxFit, REFIT_EVERY};
use super::dt::{self, Dt, GUARD_EVERY};
use super::{compute_pipeline, init_buffer, record_sized, Batch, Dispatch, Size, Step, StepCost, FRAME_SLOT, QUERIES};
use crate::dump::Resume;
use crate::state::read_buffers;
use crate::summary::{Frame, Summary};

/// latch.wgsl's `LatchState`: flag, slow_down, stopped, steps_run, fired_at, since, advanced, a pad word, the 12 saved
/// sizes, 4 pad words.
const RECORD_WORDS: usize = 24;
const SLOW_DOWN_BYTE: u64 = 4;
const SINCE_BYTE: u64 = 20;
/// The window the latch re-arms after (§1.8.5), in t.u.: `since` starts there, so a new latch is armed.
const WINDOW: f32 = 1.0;
/// The record's `held` word: a step's P5 carried over (latch.wgsl).
const HELD_WORD: usize = 7;
/// reduce.wgsl's `DtState`, copied after the latch record into each readback (the sim time with it), then P5's `most`
/// (one word) for the engine's choice of rounds.
const DT_WORDS: usize = dt::RECORD_WORDS;
const READ_WORDS: usize = RECORD_WORDS + DT_WORDS + 1;
const READBACK_BYTES: u64 = (READ_WORDS * 4) as u64;
/// A bare run's submissions, at most, per step and slot ([`Step::run`]): far past what the ceiling lets a step use.
const SUBMISSIONS_PER_SLOT: u32 = 2;
/// The two records, then room for a whole query set's timestamps.
const READBACK_TIMED_BYTES: u64 = READBACK_BYTES + 8 * QUERIES as u64;

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
    /// The sim time after the frame: every step's Δt since the state was made, added in step order in f64 on the GPU
    /// (P1) — the same whatever the steps' grouping into frames (M0-D21).
    pub sim_time: f64,
    /// Δt of the frame's last step run, and the next step's (§1.3.2 P1, P9).
    pub dt: f32,
    pub dt_next: f32,
    /// Steps begun since the state was made (the Δt record's count).
    pub steps: u64,
    /// The frame's step (1 for its first) after which the latch fired; none when it did not.
    pub latch_fired: Option<u32>,
    /// A step's P5 carried over into the next frame (§0.5 [M0-TJ4] (6)): `steps` counts it begun, `steps_run` will count
    /// it in the frame it ends in.
    pub carried: bool,
    /// The frame's cost per step and per pass (§1.8.3's c_step input); none while timing is off.
    pub cost: Option<StepCost>,
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
    /// P5's gate when the step runs P5: `latch_hold` and `latch_carry`, and the buffer P5's report is read from.
    gate: Option<(Dispatch, Dispatch, wgpu::Buffer)>,
    dt_record: wgpu::Buffer,
    /// For the readback buffers: one per frame in flight, made when none is free.
    device: wgpu::Device,
    readbacks: RefCell<Readbacks>,
}

/// A frame in flight: its number and, with timing on, its batch and whether its timestamps were resolved.
type InFlight = (u64, Option<(Batch, bool)>);

struct Readbacks {
    /// A readback buffer and, while in flight, its frame.
    slots: Vec<(wgpu::Buffer, Option<InFlight>)>,
    sent: Sender<(usize, Result<(), wgpu::BufferAsyncError>)>,
    ready: Receiver<(usize, Result<(), wgpu::BufferAsyncError>)>,
    frames: u64,
}

impl Latch {
    /// The latch for a step made of `passes`, `dt`'s guard and `boxfit`'s re-fits: one slot per fixed size and schedule,
    /// every indirect buffer the step reads checked to be one the controller zeroes; with P5's gate on `heat` (its
    /// `Machines`, then its slots' sizes of parity 0) when the step runs P5.
    pub(super) fn new(device: &wgpu::Device, passes: &[Vec<Dispatch>; 10], dt: &Dt, boxfit: &BoxFit, heat: Option<[&wgpu::Buffer; 2]>) -> Latch {
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
                6 => hook.as_entire_binding(),
                b => heat.expect("P5's gate binds P5's buffers")[b as usize - 7].as_entire_binding(),
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
            gate: heat.map(|h| {
                (dispatch("latch_hold", &[0, 1, 2, 4, 5, 7, 8], one()), dispatch("latch_carry", &[0, 1, 2, 3, 4, 5, 7], one()), h[0].clone())
            }),
            dt_record: dt.record().clone(),
            device: device.clone(),
            readbacks: RefCell::new(Readbacks { slots: Vec::new(), sent, ready, frames: 0 }),
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
                Size::Indirect(..) | Size::Chained(..) => record_sized(cpass, d, &d.size),
            }
        }
    }

    /// The controller, recorded with its own fixed size after each step's P9: it runs whether or not the latch fired.
    pub(super) fn controller(&self) -> &Dispatch {
        &self.check
    }

    /// The latch record, whose first word is the flag P6 and P3 OR into.
    pub(super) fn flag(&self) -> &wgpu::Buffer {
        &self.record
    }

    pub(super) fn hooked(&self) -> bool {
        self.hooked.get()
    }

    pub(super) fn hook_dispatches(&self) -> &[Dispatch] {
        &self.hook_dispatches
    }

    /// P5's gate, `latch_hold` and `latch_carry`, recorded bare; none without P5.
    pub(super) fn gate(&self) -> Option<(&Dispatch, &Dispatch)> {
        self.gate.as_ref().map(|(hold, carry, _)| (hold, carry))
    }
}

impl Step {
    /// Records a frame (§1.8.3, §1.8.5, §3.1): `plan.steps` steps under the latch, in `encoder` — one submission — then
    /// the frame's report, read back when the submission completes ([`Step::poll`]).
    pub fn encode_frame(&self, plan: &FramePlan, encoder: &mut wgpu::CommandEncoder) {
        let latch = &self.latch;
        self.summary.encode_reset(encoder);
        self.timer.start();
        self.encode_latched(encoder, plan, true);
        let timed = self.timer.finish(encoder);

        let mut rb = latch.readbacks.borrow_mut();
        let frame = rb.frames;
        rb.frames += 1;
        let i = match rb.slots.iter().position(|(_, f)| f.is_none()) {
            Some(i) => i,
            None => {
                let buffer = latch.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("latch.readback"),
                    size: READBACK_TIMED_BYTES + self.summary.bytes(),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                rb.slots.push((buffer, None));
                rb.slots.len() - 1
            }
        };
        let (buffer, busy) = &mut rb.slots[i];
        let buffer = buffer.clone();
        encoder.copy_buffer_to_buffer(&latch.record, 0, &buffer, 0, (RECORD_WORDS * 4) as u64);
        encoder.copy_buffer_to_buffer(&latch.dt_record, 0, &buffer, (RECORD_WORDS * 4) as u64, (DT_WORDS * 4) as u64);
        if let Some((_, _, machine)) = &latch.gate {
            encoder.copy_buffer_to_buffer(machine, super::heat::MOST_BYTE, &buffer, ((RECORD_WORDS + DT_WORDS) * 4) as u64, 4);
        }
        *busy = Some((
            frame,
            timed.map(|(batch, resolved)| {
                if let Some((resolve, bytes)) = &resolved {
                    encoder.copy_buffer_to_buffer(resolve, 0, &buffer, READBACK_BYTES, *bytes);
                }
                (batch, resolved.is_some())
            }),
        ));
        self.summary.encode(encoder, &buffer, READBACK_TIMED_BYTES);
        let sent = rb.sent.clone();
        encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., move |r| {
            let _ = sent.send((i, r));
        });
    }

    /// The reports of the frames whose readback has arrived since the last call, in submission order. Never blocks:
    /// it polls the device without waiting (§3.1). Their block summaries are dropped ([`Step::poll_frames`] keeps them).
    pub fn poll(&self, device: &wgpu::Device) -> Vec<FrameReport> {
        self.drain(device, false).into_iter().map(|(report, _)| report).collect()
    }

    /// [`Step::poll`] with each frame's block summary (§1.9.1, §3.1).
    pub fn poll_frames(&self, device: &wgpu::Device) -> Vec<Frame> {
        let frames = self.drain(device, true).into_iter();
        frames.map(|(report, summary)| Frame { report, summary: summary.expect("decoded") }).collect()
    }

    /// The frames whose readback has arrived, in submission order, each summary decoded when `summaries`.
    fn drain(&self, device: &wgpu::Device, summaries: bool) -> Vec<(FrameReport, Option<crate::summary::BlockSummary>)> {
        let _ = device.poll(wgpu::PollType::Poll);
        let mut rb = self.latch.readbacks.borrow_mut();
        type Got = (u64, [u32; READ_WORDS], Option<(Batch, Option<Vec<u64>>)>, Option<crate::summary::BlockSummary>);
        let mut got: Vec<Got> = Vec::new();
        while let Ok((i, result)) = rb.ready.try_recv() {
            let (buffer, busy) = &mut rb.slots[i];
            let (frame, timed) = busy.take().expect("a readback in flight");
            result.unwrap_or_else(|e| panic!("latch readback: map failed: {e}"));
            let mut w = [0u32; READ_WORDS];
            let (timed, summary) = {
                let view = buffer.get_mapped_range(..).expect("mapped for reading");
                let (latched, summary) = view.split_at(READBACK_TIMED_BYTES as usize);
                let summary = summaries.then(|| Summary::decode(summary));
                let (records, stamps) = latched.split_at(READBACK_BYTES as usize);
                for (k, b) in records.as_chunks::<4>().0.iter().enumerate() {
                    w[k] = u32::from_le_bytes(*b);
                }
                let timed = timed.map(|(batch, stamped)| {
                    let n = 2 * batch.passes.len();
                    let ticks = stamped.then(|| stamps.as_chunks::<8>().0[..n].iter().map(|b| u64::from_le_bytes(*b)).collect());
                    (batch, ticks)
                });
                (timed, summary)
            };
            buffer.unmap();
            got.push((frame, w, timed, summary));
        }
        got.sort_by_key(|(frame, ..)| *frame);
        drop(rb);
        got.into_iter()
            .map(|(_, w, timed, summary)| {
                let d = &w[RECORD_WORDS..];
                if let Some(heat) = &self.heat {
                    heat.learn(w[READ_WORDS - 1]);
                }
                let report = FrameReport {
                    steps_run: w[3],
                    advanced: f32::from_bits(w[6]),
                    sim_time: dt::sim_time(d),
                    dt: f32::from_bits(d[0]),
                    dt_next: f32::from_bits(d[1]),
                    steps: u64::from(d[3]),
                    latch_fired: (w[2] == 1).then_some(w[4]),
                    carried: w[HELD_WORD] != 0,
                    cost: timed.and_then(|(batch, ticks)| self.timer.cost(&batch, ticks.as_deref(), w[3])),
                };
                (report, summary)
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

    /// What a state dump carries beyond the cells (§2.12.3, contract §0.5 [M0-T26]): the GPU's step count (the next
    /// step's index), the sim time of those steps (the Δt record's, M0-D21), and the records no cell holds — Δt_n and Δt_{n+1}, the
    /// latch's sim time since its last set, the box's record and indirect sizes. Read at a step boundary, after every
    /// submitted step; blocks on the device (tests and headless runs only).
    pub fn resume_records(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (u64, f64, Resume) {
        let buffers = [self.dt.record(), self.boxfit.active(), self.boxfit.args(), &self.latch.record];
        let w: Vec<u32> = read_buffers(device, queue, "dump.records", buffers.into_iter()).iter().map(|v| v.to_bits()).collect();
        assert_eq!(w[DT_WORDS + 8 + 6 + HELD_WORD], 0, "a dump at a step's end only: a step's P5 is carried over (§0.5 [M0-TJ4] (6))");
        let resume = Resume {
            dt: f32::from_bits(w[0]),
            dt_next: f32::from_bits(w[1]),
            latch_since: f32::from_bits(w[DT_WORDS + 8 + 6 + 5]),
            box_record: std::array::from_fn(|i| w[DT_WORDS + i]),
            box_args: std::array::from_fn(|i| w[DT_WORDS + 8 + i]),
        };
        (u64::from(w[3]), dt::sim_time(&w), resume)
    }

    /// Records a submission's steps under the latch (`plan`): the slow-down word, `latch_begin`, the steps, `latch_end` —
    /// the frame's own passes timed when `framed`, else untimed (a bare run's); P5's `most` cleared first.
    fn encode_latched(&self, encoder: &mut wgpu::CommandEncoder, plan: &FramePlan, framed: bool) {
        let latch = &self.latch;
        encoder.copy_buffer_to_buffer(&latch.switch[usize::from(plan.slow_down)], 0, &latch.record, SLOW_DOWN_BYTE, 4);
        if let Some((_, _, machine)) = &latch.gate {
            encoder.clear_buffer(machine, super::heat::MOST_BYTE, Some(4));
        }
        let own = |encoder: &mut wgpu::CommandEncoder, label: &str, d: &Dispatch| {
            let mut cpass = if framed {
                self.timer.begin(encoder, label, FRAME_SLOT)
            } else {
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(label), timestamp_writes: None })
            };
            super::record_dispatches(&mut cpass, std::slice::from_ref(d));
        };
        own(encoder, "frame begin", &latch.begin);
        for _ in 0..plan.steps {
            self.encode_step(encoder, Some(latch));
        }
        own(encoder, "frame end", &latch.end);
    }

    /// [`Step::run`] for a step with P5: submissions under the latch, the slow-down off, until `steps` steps have ended —
    /// each waited for, its record read back (the steps it ended, whether one is held) with P5's most slots a step used
    /// (the engine's choice of rounds); a submission after a carry holds the carried step alone. Returns the submissions.
    pub(super) fn run_gated(&self, device: &wgpu::Device, queue: &wgpu::Queue, steps: u32) -> u32 {
        let (_, _, machine) = self.latch.gate.as_ref().expect("P5's gate");
        let heat = self.heat.as_ref().expect("the step runs P5");
        if !self.timer.batch.borrow().live {
            self.timer.start();
        }
        let ceiling = steps.saturating_mul(SUBMISSIONS_PER_SLOT * super::heat::SLOTS as u32);
        let (mut left, mut held, mut submissions) = (steps, false, 0u32);
        while left > 0 {
            assert!(submissions < ceiling, "{steps} steps still {left} short after {submissions} submissions: P5's carry never ends");
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("step") });
            self.encode_latched(&mut encoder, &FramePlan { steps: if held { 1 } else { left }, slow_down: false }, false);
            queue.submit([encoder.finish()]);
            submissions += 1;
            let w: Vec<u32> = read_buffers(device, queue, "latch.run", [&self.latch.record, machine].into_iter()).iter().map(|v| v.to_bits()).collect();
            let most = w[RECORD_WORDS + super::heat::MOST_BYTE as usize / 4];
            heat.learn(most);
            let ended = w[3];
            held = w[HELD_WORD] != 0;
            left = left.saturating_sub(ended);
            let mut b = self.timer.batch.borrow_mut();
            b.ended = Some(b.ended.unwrap_or(0) + ended);
        }
        submissions
    }

    /// Restores [`Step::resume_records`] into a step that has recorded nothing yet: its next step is `step` (step 0 is
    /// still preceded by the initial fit and reduction), the sim time goes on from `sim_time`, the guard's finding
    /// is clear and the latch's flag unset. Every later step then runs as it would have in the run the records came from.
    pub fn restore(&self, queue: &wgpu::Queue, step: u64, sim_time: f64, r: &Resume) {
        let steps = u32::try_from(step).expect("a step index within the GPU's u32 count");
        let bytes = |words: &[u32]| words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>();
        // reduce.wgsl's DtState: dt, dt_next, the step in flight (the last one run), the steps begun, a clear guard, the
        // sim time's bits.
        let t = sim_time.to_bits();
        let dt = [r.dt.to_bits(), r.dt_next.to_bits(), steps.saturating_sub(1), steps, 0, 0, 0, 0, t as u32, (t >> 32) as u32];
        queue.write_buffer(self.dt.record(), 0, &bytes(&dt));
        queue.write_buffer(self.boxfit.active(), 0, &bytes(&r.box_record));
        queue.write_buffer(self.boxfit.args(), 0, &bytes(&r.box_args));
        queue.write_buffer(&self.latch.record, 0, &bytes(&[0]));
        queue.write_buffer(&self.latch.record, SINCE_BYTE, &bytes(&[r.latch_since.to_bits()]));
        self.next.set(step);
    }
}
