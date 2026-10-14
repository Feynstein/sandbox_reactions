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
// fires (`stopped` 1): every later indirect size of the frame is zeroed — the slots, the box's sizes (kept, then restored by
// `latch_end`), P0's gate — so no later step runs, the box re-fit included. A set re-arms nothing: `since` restarts
// at 0 on every set, fired or not. The slow-down off, a set never fires. Δt is never read to decide a size (§1.8.7).
//
// P5's carry (§0.5 [M0-TJ4] (6), M0-T121), on a step with P5 — frames and bare runs alike (latch.rs): each step records
// `latch_hold` first and `latch_carry` after the rounds it records. A step whose rounds are still running there is
// carried: `stopped` 2, every later size of the submission zeroed as a fired latch zeroes them, `held` set. The next
// submission's first step is its continuation: `latch_hold` closes its P0–P5 opening (no new step begins) and starts the
// rounds again; its `latch_carry`, once the rounds end, opens the sizes for its commit and P6–P9 at its own index.

struct LatchState {
    flag: atomic<u32>,
    slow_down: u32,
    stopped: u32,
    steps_run: u32,
    fired_at: u32,
    since: f32,
    advanced: f32,
    // A step's P5 carried over into the next submission (§0.5 [M0-TJ4] (6)).
    held: u32,
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
// P5's (step/heat.rs): heat.wgsl's `Machines` as words, and its slots' sizes of parity 0.
@group(0) @binding(7) var<storage, read_write> heat: array<u32>;
@group(0) @binding(8) var<storage, read_write> rounds_even: array<u32>;

// Words of `heat`: the parity-1 record's tiles across and down, and the report's `live`.
const REC_TX: u32 = 22u;
const REC_TY: u32 = 23u;
const LIVE: u32 = 40u;

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
        restore_sizes();
    }
}

// The sizes saved by `zero_sizes` back.
fn restore_sizes() {
    for (var i = 0u; i < 6u; i++) {
        box_args[i] = latch.saved[i];
        gate_args[i] = latch.saved[6u + i];
    }
}

// Before a step's P0: a held step continues — no new step begins (every size zeroed, the box's kept) and its rounds start
// again at slot 0.
@compute @workgroup_size(1)
fn latch_hold() {
    if (latch.stopped != 0u || latch.held == 0u) {
        return;
    }
    zero_sizes();
    rounds_even[0] = heat[REC_TX];
    rounds_even[1] = heat[REC_TY];
    rounds_even[2] = 1u;
}

// After the rounds a step records (and heat.wgsl's `heat_park`, which zeroed their words): still running → carried, the
// rest of the submission zeroed; ended after a hold → the box back and the sizes opened for the commit and P6–P9 at the
// step's index (P1 counted it).
@compute @workgroup_size(1)
fn latch_carry() {
    if (latch.stopped != 0u) {
        return;
    }
    if (heat[LIVE] != 0u) {
        if (latch.held == 0u) {
            zero_sizes();
        }
        latch.held = 1u;
        latch.stopped = 2u;
    } else if (latch.held != 0u) {
        latch.held = 0u;
        restore_sizes();
        open_sizes(dt.steps - 1u);
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
