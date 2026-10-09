# Architecture — the crates, the step, the data flow, the observer boundary

Written by M0-T9 (2026-10-09) from `milestones/m0/m0_contrat.md` §1.1–§1.3 and §1.9. The contract owns every
number, formula and string; this page names the parts and points to the section that fixes them. Where a part is not
built yet it says so (« lands with »), so the page can be read against the tree. Each lot's V checks the code against
this page (§7).

## The crates — dependencies point one way (§1.1)
```
sr-app  ──►  sr-engine  ──►  sr-physics
```
| Crate | Holds | GPU? |
|---|---|---|
| `sr-physics` | units, registries (load + validate), EOS, rate laws, time control (§1.8), `reference` (the CPU f64 twin of every pass — §5's oracle, never shipped), `observe` (tracker, stage classifier, readout translation, age clock — §1.9, §1.10) | no |
| `sr-engine` | wgpu: the device and adapter choice (`gpu`), the cell state (`state`), the step and its passes (`step`, `shaders/*.wgsl`), the box, the latch, block summaries, rendering, and the headless / bench / calibrate commands (§3.3) | yes |
| `sr-app` | the game: the eframe app (`desktop.rs`, shared by the desktop and the web entry), the status endpoint (`status.rs`, native only, §3.4), the scripted captures (`capture.rs`, native only, §3.3), the web entry (`web.rs`, wasm32 only, §3.5), the string table (`strings.rs`, §4.1); the binary `sandbox-reactions` | through sr-engine |

Outside `crates/`: `assets/` (the registries and constants, embedded at compile time, §1.5–§1.7, §2.7, §2.11) ·
`web/` (the trunk entry, §3.5) · `scenes/` (the test scenes, schema §2.12) · `tests/` (the scopes' runners and
`tests/plants/`) · `build/` (cargo's target and every build output, git-ignored, §6.5).

Two rules the layout enforces:
- **No upward import.** sr-physics knows nothing of wgpu; sr-engine knows nothing of eframe or egui. The CPU twin and
  the readout translations therefore run without a GPU (§6.3).
- **One device, default limits.** sr-engine creates its device at `Limits::default()` with no native-only feature, so
  the same shaders run in the browser (§6.2.2); a feature that only a native adapter has is a defect, not a shortcut.

## sr-engine's modules (§1.3.1)
`state` (§2.2) · `grav` (§1.4) · `hydro` (§2.3) · `heat` (§2.4) · `react` (§1.6, §2.5) · `sinks` (§2.6) ·
`edges` (§2.8) · `boxfit` (§1.3.3) · `latch` (§1.8.5) · `summary` (§1.9.1) · `render` (§1.11) · `edit` (§1.12) ·
`ledger` (§2.9) · `headless`, `bench`, `calibrate` (§3). Built so far (M0-T2–M0-T4): `gpu`, `state`, `step` (only P8
holds a dispatch), `headless`; each other module lands with the lot the plan names for its pass, and fills the slot
that is already there — no reordering.

## One step — the pass order (§1.3.2)
`step::Pass` is the contract's P0–P9, recorded as fixed dispatch lists in this order, built once for the state's world:

| Pass | Role (the contract's row has the formulas) |
|---|---|
| P0 edits | the player's queued edits at the step boundary; the box re-fit; ledger entries (§1.12) |
| P1 Δt | the step's Δt is the reduction P9 made at the end of the step before (§2.3.3) |
| P2 gravity | the potential by FFT convolution over the box, on a cadence; sinks' potentials added (§1.4) |
| P3 sinks | formation and accretion, then the sinks' kick-drift (§2.6) |
| P4 hydro | the flux scheme with the gravity source, the radiation force and the edge ghosts (§2.3, §2.8) |
| P5 heat | flux-limited diffusion; radiated energy counted; the flux kept for the next step's P4 (§2.4) |
| P6 reactions | every enabled registry record and neutrino cooling, sub-cycled per cell; the latch flag (§2.5, §1.8.5) |
| P7 neutrino heating | the neutrino share deposited through a kernel (§2.5.4) |
| P8 floors | vacuum reset, temperature floor, species renormalised; ledger floor terms (§2.7) |
| P9 reductions | the next Δt, the non-finite guard, the per-cell frame accumulators (§2.3.3, §3.3, §1.9.1) |

After P9, at the step indices the contract names, the box is re-fitted (§1.3.3) and gravity's cadence decided
(§1.4.4), both from the state alone.

**What no pass may read:** the wall clock, the rung, the frame count, a label (§1.3.2). The step is a function of the
state and the step count — that is what makes a run bit-identical on one adapter, driver and binary (§1.3.4, §6.7).
**What breaks that:** a floating-point atomic in a pass that writes the state or a value it depends on; a reduction that
feeds the state and is not a fixed-order tree. Flags use `atomicOr` / `atomicMax` (§1.3.4).

## The data flow
```
 scene / preset / edits ─► State (CPU canonical planes, §2.2) ─upload─► GPU storage buffers
                                                                          │
   frame loop (time control, §1.8) ── asks for k steps ──► Step: P0 … P9 ─┤   the box (§1.3.3) bounds every pass
                                                                          ▼
                                    block summaries (§1.9.1, read back async.) ─► observe (sr-physics)
                                    ledger (§2.9)                                              │
                                    render (§1.11) ◄── state, views, glow                     ▼
                                                                  objects · stages · events · readouts (§1.9, §1.10)
```
- **State.** The CPU holds the canonical planes in §2.2's order (the state dump's order, §2.12.3); the GPU holds the
  same planes packed into a few storage buffers of whole planes — packing is the engine's business, upload and
  readback go through the canonical form only.
- **The frame loop** (`sr_physics::time`, pure logic, §1.8) decides how many steps a frame runs; the step never learns why.
- **Headless** (`sr_engine::headless`, §3.3) runs the same `Step` with no surface: scene in, `summary.json` out
  (§2.12.2). The desktop and web entries run it inside eframe's frame loop. Nothing in the step differs between them.
- **The web entry** is the same `SandboxApp` as the desktop (§3.5); only the runner, the canvas and the readiness signal
  differ (`web.rs`, `srState`).

## The observer boundary (§1.9, I2) — labels watch, never drive
Everything that names or explains the star — blocks, objects, stages, events, the predicted ending, the readouts, the
age clock (§1.9.1–§1.9.5, §1.10) — lives in `sr_physics::observe` and reads the **block summaries** and **ledger** the
step produced. The arrow goes one way:

```
 step (sr_engine::step, shaders) ──produces──► summaries, ledger ──read by──► observe ──► UI, events, readouts
        ▲                                                                         │
        └──────────────────────── never ◄───────────────────────────────────────┘
```
- `sr_engine::step` and every shader never import `sr_physics::observe` (§1.1). A stage change, an object id or a
  readout can never alter a state value: the star's life comes out of the physics, never a script.
- The one sanctioned feedback is **time control**: the frame loop may slow down at an event (the latch, the
  automatic slow-down — §1.8), but it changes how many steps run, never what a step does.
- **G-WATCH** (the static scan, M0-T16; contract §5) is the check that grades this boundary; M0-T9 only names it.

## Where a number lives
Not here. Tunable constants → `assets/physics.json` (§2.7); measured ones (ending thresholds, the top rung, the render
reserve, the readout anchors) → `assets/calibration.json` (§2.11, written by `sandbox-reactions calibrate`); strings →
the string table (§4.1); tolerances → §5 (set before the first run, never loosened in a build block, §0.4).
