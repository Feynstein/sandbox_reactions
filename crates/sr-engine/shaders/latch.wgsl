// The latch and the frame's sizes (m0_contrat.md §1.8.3, §1.8.5, §1.8.7, §3.1; TW-R4, TW-E16).
//
// A frame records N steps; under the latch every dispatch of a step takes its workgroup count from a buffer the
// controller can zero: a fixed-size dispatch from `sizes`' slot in `args` (one slot per size and schedule), a box-sized
// one from the box's own sizes, P0's re-fit from its gate (latch.rs `Latch`). `latch_begin` opens the frame: each slot
// gets its size when it runs on every step, or when its period divides the index of the next step (the guard's 64, the
// re-fit's 16, §1.3.2) — the GPU's step index, never the CPU's count — else 0.
//
// The flag (word 0, a u32): a pass sets it with atomicOr — P6 when a cell's N_Fe rate is > 0, P3 when a sink forms
// (§1.8.5; their lots) — and the controller, `latch_check` after each step's P9, takes it (atomicExchange). The latch is
// armed when the flag was not set during the last WINDOW of sim time: `since` is the sim time since the last set, the
// step's Δt added before the test, held at WINDOW (armed from the start). An armed latch set with the slow-down on
// fires: every later indirect size of the frame is zeroed — the slots, the box's sizes (kept, then restored by
// `latch_end`), P0's gate — so no later step runs, the box re-fit included. A set re-arms nothing: `since` restarts
// at 0 on every set, fired or not. The slow-down off, a set never fires. Δt is never read to decide a size (§1.8.7).

struct LatchState {
    flag: atomic<u32>,
    slow_down: u32,
    stopped: u32,
    steps_run: u32,
    fired_at: u32,
    since: f32,
    advanced: f32,
    _pad: u32,
    // The two indirect buffers' sizes as they stood when the latch fired (box args, then P0's gate).
    saved: array<u32, 12>,
    _pad1: vec4<u32>,
}

// reduce.wgsl's DtState, read only here.
struct DtState {
    dt: f32,
    dt_next: f32,
    step: u32,
    steps: u32,
    found: u32,
    bad_step: u32,
    bad_code: u32,
    flagged: u32,
}

// The test hook (latch.rs `set_latch_hook`): the flag set in P6 of the steps lo, lo + stride, … below hi; `probe`
// counts the box-sized dispatches that ran.
struct Hook {
    lo: u32,
    hi: u32,
    stride: u32,
    probe: atomic<u32>,
}

@group(0) @binding(0) var<storage, read_write> latch: LatchState;
@group(0) @binding(1) var<storage, read_write> args: array<u32>;
// One slot per row: (x, y, z, period); period 0 runs on every step.
@group(0) @binding(2) var<storage, read> sizes: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read> dt: DtState;
@group(0) @binding(4) var<storage, read_write> box_args: array<u32, 6>;
@group(0) @binding(5) var<storage, read_write> gate_args: array<u32, 6>;
@group(0) @binding(6) var<storage, read_write> hook: Hook;

// §1.8.5: armed when not set during the last 1.0 t.u.
const WINDOW: f32 = 1.0;

// The slots for the step whose index is `next`.
fn open_sizes(next: u32) {
    for (var i = 0u; i < arrayLength(&sizes); i++) {
        let s = sizes[i];
        let on = s.w == 0u || next % s.w == 0u;
        args[3u * i] = select(0u, s.x, on);
        args[3u * i + 1u] = select(0u, s.y, on);
        args[3u * i + 2u] = select(0u, s.z, on);
    }
}

// Every later indirect size of the frame zeroed; the box's and the gate's kept for `latch_end`.
fn zero_sizes() {
    for (var i = 0u; i < 3u * arrayLength(&sizes); i++) {
        args[i] = 0u;
    }
    for (var i = 0u; i < 6u; i++) {
        latch.saved[i] = box_args[i];
        latch.saved[6u + i] = gate_args[i];
        box_args[i] = 0u;
        gate_args[i] = 0u;
    }
}

@compute @workgroup_size(1)
fn latch_begin() {
    latch.stopped = 0u;
    latch.steps_run = 0u;
    latch.fired_at = 0u;
    latch.advanced = 0.0;
    open_sizes(dt.steps);
}

@compute @workgroup_size(1)
fn latch_check() {
    if (latch.stopped != 0u) {
        return;
    }
    latch.steps_run = latch.steps_run + 1u;
    latch.advanced = latch.advanced + dt.dt;
    latch.since = min(latch.since + dt.dt, WINDOW);
    if (atomicExchange(&latch.flag, 0u) != 0u) {
        if (latch.since >= WINDOW && latch.slow_down != 0u) {
            latch.stopped = 1u;
            latch.fired_at = latch.steps_run;
        }
        latch.since = 0.0;
    }
    if (latch.stopped != 0u) {
        zero_sizes();
    } else {
        open_sizes(dt.steps);
    }
}

// The box's and the gate's sizes back, so the next frame — or a bare step — runs inside the box.
@compute @workgroup_size(1)
fn latch_end() {
    if (latch.stopped != 0u) {
        for (var i = 0u; i < 6u; i++) {
            box_args[i] = latch.saved[i];
            gate_args[i] = latch.saved[6u + i];
        }
    }
}

@compute @workgroup_size(1)
fn latch_hook() {
    let s = dt.step;
    if (s >= hook.lo && s < hook.hi && (s - hook.lo) % max(hook.stride, 1u) == 0u) {
        atomicOr(&latch.flag, 1u);
    }
}

@compute @workgroup_size(8, 8)
fn latch_probe(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    if (lid == 0u && wg.x == 0u && wg.y == 0u) {
        atomicAdd(&hook.probe, 1u);
    }
}
