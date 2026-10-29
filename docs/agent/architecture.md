# Architecture — the crates, the step, the data flow, the observer boundary

Written by M0-T9 (2026-10-09) from `milestones/m0/m0_contrat.md` §1.1–§1.3 and §1.9; lot 3's parts (the dispatch lists,
the booking layout, the box, the frame and the latch, the files) added by M0-T27 (2026-10-11) from §1.3, §1.8, §2.9,
§2.12 and the handoffs of M0-T18–M0-T26. The contract owns every number, formula and string; this page names the parts
and points to the section that fixes them. Where a part is not built yet it says so (« lands with »), so the page can be
read against the tree. Each lot's V checks the code against this page (§7). Rendering (§1.11; M0-T79–M0-T82) added by M0-T83. Time control's own page:
`docs/agent/time_control.md`.

## The crates — dependencies point one way (§1.1)
```
sr-app  ──►  sr-engine  ──►  sr-physics
```
| Crate | Holds | GPU? |
|---|---|---|
| `sr-physics` | units, registries (load + validate), EOS, rate laws, time control (§1.8), `reference` (the CPU f64 twin of every pass — §5's oracle, never shipped), `observe` (tracker, stage classifier, readout translation, age clock — §1.9, §1.10) | no |
| `sr-engine` | wgpu: the device and adapter choice (`gpu`), the cell state (`state`), the step and its passes (`step`, `shaders/*.wgsl`), the box, the latch, block summaries, rendering, scene files and state dumps (`scene`, `dump`), and the headless / bench / calibrate commands (§3.3) | yes |
| `sr-app` | the game: the eframe app (`desktop.rs`, shared by the desktop and the web entry), the status endpoint (`status.rs`, native only, §3.4), the scripted captures (`capture.rs`, native only, §3.3), the web entry (`web.rs`, wasm32 only, §3.5), the string table (`strings.rs`, §4.1); the binary `sandbox-reactions` | through sr-engine |

Outside `crates/`: `assets/` (the registries and constants, embedded at compile time, §1.5–§1.7, §2.7, §2.11) ·
`web/` (the trunk entry, §3.5) · `scenes/` (the test scenes, schema §2.12.1) · `tests/` (the scopes' runners and
`tests/plants/`) · `build/` (cargo's target and every build output, git-ignored, §6.5).

Two rules the layout enforces:
- **No upward import.** sr-physics knows nothing of wgpu; sr-engine knows nothing of eframe or egui. The CPU twin, time
  control and the readout translations therefore run without a GPU (§6.3).
- **One device, default limits.** sr-engine creates its device at `Limits::default()` with no native-only feature —
  WebGPU's `timestamp-query` is requested only where the adapter offers it (M0-T23) — so the same shaders run in the
  browser (§6.2.2); a feature that only a native adapter has is a defect, not a shortcut.

## The modules (§1.3.1) — what is built
sr-engine: `state` (§2.2) · `grav` (§1.4) · `hydro` (§2.3) · `heat` (§2.4) · `react` (§1.6, §2.5) · `sinks` (§2.6) ·
`edges` (§2.8) · `boxfit` (§1.3.3) · `latch` (§1.8.5) · `summary` (§1.9.1) · `render` (§1.11) · `edit` (§1.12) ·
`ledger` (§2.9) · `headless`, `bench`, `calibrate` (§3). sr-physics: `units`, `registry`, `eos`, `rates`, `time` (§1.8),
`reference`, `observe`.

Built through M0-T26 (2026-10-11):
| Module | File | Built by |
|---|---|---|
| `gpu` — adapter choice, the device | `crates/sr-engine/src/gpu.rs` | M0-T2, M0-T23 (timestamps where offered) |
| `state` — the cell state, the side fields, the booking layout | `crates/sr-engine/src/state.rs` | M0-T3, M0-T18 |
| `step` — the pass slots, the dispatch lists, the EOS on the GPU, the cost per step | `crates/sr-engine/src/step/mod.rs`, `shaders/floors.wgsl`, `shaders/eos.wgsl` | M0-T3, M0-T19, M0-T23 |
| `step::dt` — P1, P9's Δt and the non-finite guard | `crates/sr-engine/src/step/dt.rs`, `shaders/reduce.wgsl` | M0-T20, M0-D18 |
| `step::boxfit` — the active box (the contract's `boxfit`) | `crates/sr-engine/src/step/boxfit.rs`, `shaders/reduce.wgsl` | M0-T21 |
| `step::latch` — a frame of steps, the latch (the contract's `latch`) | `crates/sr-engine/src/step/latch.rs`, `shaders/latch.wgsl` | M0-T22, M0-T23, M0-T26 |
| `scene` — scene files | `crates/sr-engine/src/scene.rs` | M0-T25 |
| `dump` — state dumps | `crates/sr-engine/src/dump.rs` | M0-T26 |
| `headless` | `crates/sr-engine/src/headless.rs` | M0-T4, M0-T20, M0-T23, M0-T25, M0-T26, M0-T82 (frames) |
| `render` — the blackbody table, the cell image, the glow (§ Rendering) | `crates/sr-engine/src/render.rs`, `shaders/render.wgsl`, `shaders/glow.wgsl` | M0-T79, M0-T80, M0-T81 |
| sr-physics `units`, `registry`, `eos`, `time` | `crates/sr-physics/src/*.rs` | M0-T10–M0-T12, M0-T24 |

`boxfit` and `latch` sit under `step::` because both rewrite how a step's dispatches are sized; the contract names them as
modules and the names are kept. Every other module lands with the lot the plan names for its pass, and fills the slot
that is already there — no reordering. **No `Engine` type yet** (§3.1's API): today a caller drives `Step`'s methods
directly (M0-T22's deviation); the app wires none of this until M0-T84.

## One step — the pass order and the dispatch lists (§1.3.2)
`step::Pass` is the contract's P0–P9 (`Pass::ORDER`; `Pass::label` gives "P0 edits" … "P9 reductions"). A `Step` holds one
`Vec<Dispatch>` per pass, built once for the state's world by `Step::new` (or `Step::with_box`, below); a `Dispatch` is an
entry point, its pipeline, its bind groups (group g at index g) and its `Size` — `Fixed` workgroups or `Indirect` (a
buffer and a byte offset the GPU reads). `Step::encode` records one step: every non-empty slot in order (P0 is always
recorded, for the box's edit re-fit) — a compute pass each, labelled with its pass, when the step's passes are timed;
otherwise the whole step is one compute pass, the same dispatches in the same order (M0-D26: the pass boundaries cost
~0.06 ms a step on a Sun-sized box).

| Pass | Role (the contract's row has the formulas) | Dispatches today |
|---|---|---|
| P0 edits | the player's queued edits at the step boundary; the box re-fit; ledger entries (§1.12) | the box's `box_gate`, `box_cells`, `box_fit` (sized by the edit gate — zero when no edit flagged); the edit pass itself lands with its lot (§1.12) |
| P1 Δt | the step's Δt is the reduction P9 made at the end of the step before (§2.3.3) | `dt_advance` |
| P2 gravity | the potential by FFT convolution over the box, on a cadence; sinks' potentials added (§1.4) | when the step runs gravity (`Passes`): `grav_rows` (one workgroup a world row), `grav_columns` (fixed, the padded width), `conv_rows_inverse` (a workgroup a world row), `grav_gradient` (box tiles, indirect), and in a step that runs P3 a fifth, `grav_sinks` (box tiles, indirect: the sinks' potentials and pulls, M0-T52a) — every step, the cadence is not built |
| P3 sinks | formation and accretion, then the sinks' kick-drift (§2.6) | when the step runs sinks (`Step::with_sinks`; headless builds it from the scene's `sinks` switch, M0-D47): `sink_candidates` (box tiles, indirect), `sink_update` (fixed, one workgroup), `sink_pull` (box tiles, indirect), `sink_kick_drift` (fixed, one workgroup), `sink_pull` again (box tiles, indirect) and `sink_kick_merge` (fixed, one workgroup) — the motion of §0.5 [M0-V7], M0-D46; `sink_latch` last (fixed, one invocation — built once the step's latch exists, a formation sets the flag there) |
| P4 hydro | the flux scheme with the gravity source, the radiation force and the edge ghosts (§2.3, §2.8) | when the step runs hydro (`Passes`): `hydro_update` (box tiles, indirect: one workgroup an 8 × 8 tile, predicted faces exchanged in workgroup memory; binds WebGPU's 8 storage buffers), `hydro_edges` (fixed, one invocation a boundary face; binds 7), `hydro_commit` (box tiles, indirect) |
| P5 heat | flux-limited diffusion; radiated energy counted; the flux kept for the next step's P4 (§2.4) | when the step runs heat (`Step::with_heat`): `heat_cells` (box tiles, indirect), `heat_law` (box tiles), `heat_open` (fixed, one invocation — the latch reaches the chain through it), `heat_round` × `SLOTS` (chained: each sized by a word the round before wrote, so a converged step's later rounds have no workgroups; a step records only the first `Step::heat_rounds` of them), `heat_close` (box tiles), `heat_sum` (fixed, one workgroup), `heat_apply` (box tiles); around them `latch_hold` before P0 and `heat_park`, `latch_carry` after the rounds (fixed, one invocation each — a step still in its rounds is carried into the next submission, M0-T121) |
| P6 reactions | every enabled registry record and neutrino cooling, sub-cycled per cell; the latch flag (§2.5, §1.8.5) | when the step runs reactions (`Step::with_reactions`): `react_begin` (fixed, one invocation), `react_cells` (box tiles, indirect), `react_latch` (fixed, one invocation — built once the step's latch exists; the test hook `set_latch_hook` sets the flag there too) |
| P7 neutrino heating | the neutrino share deposited through a kernel (§2.5.4) | when the step runs neutrino heating (`Step::with_neutrino_heating`; headless builds it with `reactions` and `neutrinos` both on, S_ν being P6's, M0-D47): `nu_tiles` (box tiles, indirect), `nu_gate` (fixed, one workgroup — the sum of S_ν over the tiles opens or closes the chain), then chained on it: `conv_rows`, `nu_columns`, `conv_rows_inverse` (a workgroup a world row, a column, a world row), `nu_deposit` (box tiles), `nu_share` (one workgroup), `nu_apply` (box tiles); `nu_close` last (fixed, one invocation) — zero chained dispatches when ΣS_ν = 0 |
| P8 floors | vacuum reset, temperature floor, species renormalised; ledger floor terms (§2.7) | `floor_cells`, then `renormalise_species` (both box-sized) |
| P9 reductions | the next Δt, the non-finite guard, the per-cell frame accumulators (§2.3.3, §3.3, §1.9.1) | `dt_cells` (box-sized), `dt_partials`; `guard_scan`, `guard_stamp` on the guard's steps; the re-fit `box_cells`, `box_fit` on the re-fit's steps; under a frame, or on a step with P5, `latch_check` last |

- **Before step 0**: the box is fit to the initial state, then P9's Δt reduction runs over it, recorded once as a compute
  pass labelled "P9 reductions (initial state)" — so step 0 runs on the initial state's Δt and step n on the value P9
  made at the end of step n − 1 (§1.3.2 P1).
- **On a schedule, after P9**: the non-finite guard on step indices that `dt::GUARD_EVERY` divides (§1.3.2 P9, §3.3's
  exit 5 — the first NaN or Inf by step, cell and channel, read from the bits, never a float comparison; P8 leaves a cell
  holding a non-finite fraction as read so the guard still sees it, M0-D18) and the box re-fit on those that
  `boxfit::REFIT_EVERY` divides (§1.3.2's last line). Bare steps follow the CPU's count; a frame opens them on the GPU's
  step index (§ A frame).
- **The EOS every pass shares** (`shaders/eos.wgsl`) is prepended to a pass's shader (`step::with_eos`) and bound as group
  1 from `EosGpu`, built once per run from physics.json and elements.json (§2.3, §2.7 [M0-T19]).
- **Introspection for tests**: `dispatch_counts`, `dispatch_labels(pass)`, `guard_labels`, `box_labels`, `box_sized`
  (which dispatches read the box's sizes, and at which offset). The `state`, `floors`, `dt`, `box` and `latch` scopes
  grade the lists through these (`docs/agent/testing.md`).
- **The cost of a step** (§1.8.3's c_step, §2.12.2's `timing`): off until `Step::set_timing(…, Timing::Auto)`. Each
  pass of a timed step is a compute pass that writes a timestamp at its beginning and end — pass boundaries only, never
  inside a pass (native-only, §6.2.2) — where the device has `timestamp-query`, else the batch is timed by the wall clock and says so (`CostSource`;
  wasm32 has no wall clock, so there the cost is none). `StepCost` carries the source, the steps timed, the span per step
  (first pass's beginning → last pass's end) and the time per pass (M0-T23).

- **wgpu's indirect-call validation is off** (`gpu::instance_flags`, the headless instance's and the app's, M0-D26): it
  puts a hidden dispatch before every indirect one (~11–14 µs each on the RTX 4080 Laptop). Every indirect size the
  step reads is bounded by the world — at most 2048² / 256 = 16,384 workgroups, a compile-time assertion — so a pass
  that adds one keeps it so; `WGPU_VALIDATION_INDIRECT_CALL=1` turns the validation back on.

**What no pass may read:** the wall clock, the rung, the frame count, a label (§1.3.2). The step is a function of the
state and the step count — that is what makes a run bit-identical on one adapter, driver and binary (§1.3.4, §6.7).
**What breaks that:** a floating-point atomic in a pass that writes the state or a value it depends on; a reduction that
feeds the state and is not a fixed-order tree. Flags use `atomicOr` / `atomicMax` (§1.3.4). Δt's reduction and the box's
are both fixed-order trees: a workgroup halves its cells in shared memory, one workgroup then walks the partials in index
order.

## The booking layout (§2.2's last paragraph, §2.9, §1.9.1)
Beside the state, never part of it (`state.rs`):
- **The state** — §2.2's channels as `CHANNELS`, canonical on the CPU as one row-major plane per channel in §2.2's order
  (the dump's order), packed on the GPU into three storage buffers of whole planes (`GROUPS`, `GROUP_NAMES`: hydro,
  species a, species b), which leaves a stage's other storage slots to the passes (§6.2.2). Upload and readback go
  through the canonical form only.
- **The side fields** (`SideFields`) — φ (1 plane, P2's), the heat flux F (2 planes, F_x and F_y, P5's, read by the next
  step's P4) and S_ν (1 plane, P6's, read by P7). φ is written by P2, F by P5 (M0-T42) and S_ν by P6 (M0-T47, with the scene's `neutrinos` switch); P7 reads it (`nu_tiles`, `nu_deposit`, M0-T54).
- **The booking** (`Booking`, `TERMS`, `BOOK_GROUPS`) — every §2.9 booked term and §1.9.1 frame accumulator, a `Term`
  each, with the one pass that writes it (`Term::writer`) and whether the block summary sums it as an accumulator. The
  state-derived ledger terms (grid mass, kinetic, thermal, cold, W, the sinks) are not here: they come from the state and
  the block summary. `BOOK_GROUPS` tiles `TERMS` in order into one buffer per writer pass, so a pass binds only its own
  (P0 binds two: the tools' mass and energy); a compile-time assertion holds the tiling, the one writer per buffer and the
  largest world inside WebGPU's default binding size.

```
 buffer (BOOK_GROUPS)       writer  slots (Domain)        terms (TERMS, in order)
 booking.tools_mass         P0      one per cell          mass.painted · erased · preset_dropped · cleared
 booking.tools_energy       P0      one per cell          energy.tools.painted · erased · heated · cooled · preset · cleared
 booking.sinks              P3      one per cell          energy.swallowed
 booking.escaped            P4      one per boundary face escaped.mass · mom_x · mom_y · energy · mass.x_<each species>
 booking.heat               P5      one per cell          radiated (accumulator)
 booking.reactions          P6      one per cell          nuclear.<each group> · neutrino_lost (accumulators)
 booking.neutrino_heating   P7      one per cell          neutrino_deposited (accumulator)
 booking.floors             P8      one per cell          mass.vacuum_reset · energy.floor_added
```
(`TERMS` is the list; §2.9's `nuclear_released` is the sum of the nuclear groups.)
- **A slot has one writer invocation per dispatch**, which adds its value to it — a read-modify-write of its own slot,
  never an atomic (§1.3.4). `Domain::Cells` is a slot per cell, row-major; `Domain::EdgeFaces` a slot per boundary face
  of the world (§2.8), side by side: x = 0 by y, x = width − 1 by y, y = 0 by x, y = height − 1 by x.
- **Summed in f64 in one order**: `Booking::readback`, then `book_totals` — term by term, row by row (`Domain::rows`: a
  world row each for cells, a side each for faces), slot by slot — so the same planes give the same bits. The ledger
  (§2.9, its lot) and the block summary (§1.9.1) read the totals; nothing in the step reads them back.
- `Booking::clear` empties every term (the ledger restarts, §2.9's Clear world). Today P8 is the only pass that books
  (`vacuum_reset`, `floor_added`, M0-T19); the other writers land with their passes.

## The active box (§1.3.3, `step::boxfit`)
The bounding box of non-vacuum cells, grown, rounded out and clamped to the world as §1.3.3 says — empty when no cell is
non-vacuum. Built on the GPU and never read back to size a dispatch:
- **The fit** (`shaders/reduce.wgsl`): `box_cells` takes the whole world's cells a workgroup at a time and halves their
  integer (min x, min y, max x + 1, max y + 1) in shared memory; `box_fit`, one workgroup, walks the partials in index
  order and writes three things — **the box record** (x0, y0, w, h, the FFT size per axis, the step it was fit on, and
  whether it fits; bound as a uniform, `BoxFit::active`), **the indirect sizes** of every box-sized dispatch
  (`BoxFit::args`: 8 × 8 tiles at `ARGS_TILES`, 256-cell workgroups at `ARGS_CELLS`) and **the FFT size per axis**
  gravity's lot will read (§1.3.3's rule).
- **When**: before step 0, after P9 on `REFIT_EVERY`'s steps, and in P0 when an edit lands outside the box — P0's edit
  pass sets `BoxFit::edit_flag`'s word (`atomicOr`), `box_gate` turns it into the scan's and the fit's sizes (zero when
  unset), and `box_fit` clears it.
- **Inside the box**: P8's two dispatches and `dt_cells` today; every later pass runs inside it too (plus its ghost
  cells). Outside it cells are vacuum and static — never floored, never reduced. The re-fit's scan and the guard read the
  whole world. An empty box gives zero-sized dispatches and Δt = the largest f32 (M0-T21's default).
- **`BoxMode::WholeWorld`** holds the box at the world: the switch tests use to compare the two (G-BOX's conservation
  half lands with M0-T39). `Step::read_box` reads the record for tests and headless only.

## A frame and the latch (§1.8.3, §1.8.5, §1.8.7, `step::latch`)
- **A frame** is `Step::encode_frame(&FramePlan { steps, slow_down }, encoder)`: N steps in one encoder, one submission,
  between `latch_begin` and `latch_end`, the controller `latch_check` closing each step's P9. After `latch_end`,
  outside the frame's timing, `encode_frame` records the frame's block summary (`summary.encode`) into the frame's own
  readback; the accumulators it summed are cleared at the next frame's start. How many steps, and why, is
  `sr_physics::time`'s business (`docs/agent/time_control.md`) — the frame never learns why.
- **Every dispatch of a frame's step is zeroable**: it reads its workgroup count from a buffer the controller can zero —
  a fixed-size dispatch from its slot in `Latch::args` (one slot per size and schedule; the guard's and the re-fit's open
  on the GPU's step index, so they hold after a stopped frame), a box-sized one from the box's sizes, P0's re-fit from its
  gate (`latch_end` restores what it zeroed). `Latch::new` refuses a step holding any other indirect buffer: a pass that
  adds one adds it to `latch.wgsl` first, or the latch would not reach it.
- **The flag** is the latch record's first word (`Step::latch_flag`): P6 sets it when a cell's iron-group rate is
  positive, P3 when a sink forms — both with `atomicOr`, P6 in `react_latch` and P3 in `sink_latch` (the test hook sets it too). **Armed**
  when no set happened in the last sim-time span §1.8.5 names (every set restarts the span). An armed set with the
  slow-down on → no later step of the frame runs, the box re-fit's dispatches included, and the frame's report names the
  step it fired after; the CPU then drops the rung (time control). The slow-down off → the frame runs through.
- **`Step::poll(device)`** returns each submitted frame's `FrameReport` once, in order, from a readback mapped on submit
  — never waiting on the GPU: `steps_run`, `advanced` (its steps' Δt summed in order, f32 — the frame's own), `sim_time`
  (the Δt record's clock after the frame), `dt`, `dt_next`, `steps` (the Δt record's count), `latch_fired`, and `cost` with timing on (the frame is one
  timing batch; its timestamps ride the same readback). **`Step::poll_frames(device)`** is `poll` with each frame's block
  summary: `Frame { report, summary }` (§1.9.1); `poll` drops the summary.
- **The sim time** is a function of the steps run (M0-D21): P1 (`dt_advance`) adds each step's Δt to the Δt record's
  clock, an f64 kept as its bits — WGSL has no f64 — by an exact add on u32 words rounded to nearest even, the CPU's
  `t += f64::from(dt)` bit for bit. Frames, however grouped, and bare steps give one value; a step the latch stopped runs
  no P1, and an empty box's Δt (the largest f32) or a Δt of 0 adds nothing. `DtReport`, `FrameReport`, the dump and the
  summary all read that clock.
- **Frames or bare steps, not both**: after a stopped frame the CPU's step count runs ahead of the GPU's, so a run uses
  one or the other (M0-T22). Headless runs bare steps (`Step::run`) today.
- **Proof today**: the `latch` scope grades the mechanism on a test hook (`set_latch_hook`, a box-sized probe). **G-LATCH
  is NOT PROVEN (synthetic)** — its contract scenario (the Massive preset from an iron-core dump, §5.3) arrives with
  M0-T111; until then no page or verdict calls the latch's collapse catch proven.

## The files — scene, dump, summary (§2.12; tests and headless runs only, never a save format, §2.13)
| File | Schema | Code | Written · read by |
|---|---|---|---|
| scene `scenes/*.json` | §2.12.1; coordinates §2.12.4 | `sr_engine::scene` (`Scene`) | written by hand · read by `headless --scene <file\|preset:<name>>` and the tests |
| state dump `dumps/s_<step>.bin` | §2.12.3 plus §0.5 [M0-T26]'s `resume` | `sr_engine::dump` (`Dump`, `dump_state`, `load_dump`) | written by `headless --out DIR` (always at the end; `--dump-every N` too) · read by `headless --load-dump FILE`, M0-T111's scenario and the tests |
| run summary `summary.json` | §2.12.2 (`timing` only with `--timing`) | `sr_engine::headless` | written by `headless --out DIR` · read by the `boot`, `scene` and `timing` scopes, later every ⏱ and physics scope |

- **Scene.** Unknown keys are refused at every level, each refusal naming its path (`switches.hydor: unknown key`),
  `headless` exit 4 with no summary. A `disk` or `rect` takes `t` or `pressure`, exactly one. `overrides` are physics.json
  keys only, through `Physics::with` (the shipped file's bounds); a scene with overrides never writes calibration
  (`Scene::writes_calibration`). An object's `x`, `y` are points in cell units, cells sampled at their centres; objects add
  where they overlap; a `--world` resize keeps them about the world's centre; an object off the world is an `SR-WARN`
  (M0-T25's defaults, `milestones/m0/tasks/M0-T25.md`). Presets are scenes too (`Scene::preset`; every preset is the
  Sun-like disk until M0-T74). The scene's edge and switches are recorded in the summary; the edge mode is P4's since M0-T35
  (`Step::set_edge_mode`), the `gravity` and `hydro` switches build P2 and P4, the others wait for their passes (M0-T37).
- **Dump.** The magic, a little-endian u32 header length, the JSON header, then §2.2's channels as little-endian f32
  planes in §2.2's order, row-major — the state's canonical form, so `load_dump` → `dump_state` is bit for bit. The
  header's `step` is the steps run since the scene (the next step's index); `resume` carries what no cell holds and a
  resumed run needs to stay bit-identical (§1.3.4): Δt_n and Δt_{n+1}, the latch's sim time since its last set, the box's
  record and indirect sizes — through `Step::resume_records` / `Step::restore` (`latch.rs`), with the GPU step count and
  the sim time (the Δt record's clock). Not dumped: the booking (the ledger restarts). `sinks` is P3's, read from `Step::read_sinks` (empty when the step runs no P3, M0-T52b); `load_dump`
  restores them through `Step::set_sinks` and refuses a dump holding any into a step that runs no P3 (the format reads up to §2.6's count). `--load-dump` takes the world and cells from the dump, the constants, edge and switches from the scene;
  `--dump-every` counts step indices from the scene, so a resumed run's dumps fall where a straight run's do.
  `Dump::from_bytes` refuses by name: a wrong magic, a file cut anywhere, bytes past the last plane, a non-UTF-8 or
  non-JSON header, an unknown or missing key, another version, a world §2.8 refuses, another channel list, more sinks than
  §2.6 allows. serde_json parses floats best-effort, so `sim_time` is re-read from its own token (M0-T26).
- **Summary.** Every §2.12.2 key, filled as the blocks that compute it land: today `tool`, `git`, `adapter`, `scene`,
  `world` {width, height, edge}, `steps`, `wall_s` and `until` (`steps:N`); `sim_time` is the Δt record's clock at
  the run's end (a dump's start included), as the final dump and the DONE line write it (M0-D21); `ledger` (M0-T57), `events`, `objects` and `until` (M0-T60) are filled by the observers (`docs/agent/readouts.md`), with `DIR/events.jsonl` beside; `mass_suns`, `predicted_ending`, `ending` and `age_years` are left out of an object until M0-T69, M0-T72 and M0-T73.
  `timing` holds {per_pass_ms, step_ms_mean, source} — `source` added beyond §2.12.2's object by M0-T23 (additive; a PLAN
  task may fold it in).

## The data flow
```
 scene / preset / dump / edits ─► State (CPU canonical planes, §2.2) ─upload─► GPU storage buffers
                                                                                │
   frame loop (time control, §1.8) ── FramePlan {steps} ──► Step: P0 … P9 ─────┤   the box (§1.3.3) bounds every pass
                    ▲                                         + latch_check     │
                    └──── FrameReport (poll, ≤ 2 frames late) ◄─────────────────┤
                                                                                ▼
                                    booking (§2.9, §1.9.1, read back, f64) ─► ledger, block summaries ─► observe
                                    render (§1.11) ◄── state, views, glow                                  │
                                                                  objects · stages · events · readouts (§1.9, §1.10)
```
- **State.** The CPU holds the canonical planes; the GPU the same planes packed (§ The booking layout). Packing is the
  engine's business; upload, readback and dumps go through the canonical form only.
- **The frame loop** (`sr_physics::time`, pure logic, §1.8) decides how many steps a frame runs; the step never learns why.
- **Headless** (`sr_engine::headless`, §3.3) runs the same `Step` with no surface: a scene (or a dump) in, `summary.json`
  and dumps out (§2.12). The desktop and web entries will run it inside eframe's frame loop (M0-T84). Nothing in the step
  differs between them.
- **The web entry** is the same `SandboxApp` as the desktop (§3.5); only the runner, the canvas and the readiness signal
  differ (`web.rs`, `srState`).

## Rendering (§1.11, `sr_engine::render`, shaders/render.wgsl and glow.wgsl)
The picture of the world is a W × H RGBA8 **cell image** — one texel per cell, so cells are visible squares (B12) — written
by a compute pass from the state, never from a copy of it on the CPU. `Renderer::new` is built once for a world (the state,
the previous state, the EOS tables, §1.10.2's eight temperature anchors, the species colours); `render(view, alpha, target,
encoder)` records the passes into the caller's encoder and `target` (a storage texture of the world's size, `IMAGE_FORMAT`).
The quad that draws the texture to the window at an integer scale is the app's (M0-T84), not this page's yet.
- **The four views** (`View`: Glow, Heat, Element, Density) are one entry point and a `view` word. Glow is the default and the
  only one with the glow passes; Heat and Density are the §4.11 sequential maps over log₁₀ T_K and log₁₀ Σ; Element blends
  the species colours (§1.5) by mass fraction. Maps interpolate in linear RGB. A cell below Σ_vac is bg.space in every view
  (M0-D113), ahead of the view's formula. Every colour, stop and threshold is §1.11's and
  §4.11's — this page never restates a design token; `render_cells` draws the cell image alone, glow-less even for Glow.
- **Temperature** is the translated one: the sandbox T goes through §1.10.2's piecewise log-log translation in f32 on the GPU
  (the same eight anchors that build the readouts' `TempScale`), so the view and the status line read one temperature.
- **The interpolation.** The image blends the current state with the previous one by α, channel by channel, before it reads
  any of them: α = 1 is the current state exactly, α = 0 the previous exactly, between them the slow-motion mix of §1.8.3
  (α = owed / Δt_est; clamped to [0, 1]). The caller owes the previous state — a copy of the State taken each step; the
  renderer holds both and never advances either.
- **The blackbody table** (CPU, built once in `Renderer::new`, uploaded in the palette uniform): Planck's law integrated
  against the CIE 1931 fit, converted to sRGB, tabulated over log T, end colours outside (the formulas and constants are
  §1.11's; `blackbody_table`, `blackbody_lookup`). The Glow view reads it at T_K and mixes in the haze for cold gas.
- **The glow** (Glow view only): the cell image is drawn into an image of the renderer's own, then four dispatches — a bright
  pass to a half-resolution texture, a horizontal and a vertical Gaussian blur, a composite that upsamples and adds into
  `target`. The other views write straight into `target`. Numbers: §1.11, §4.11. The bright pass keeps a cell whose brightest
  channel is above 0.8 (§0.5 [M0-D112]), so every cell hotter than ~3,400 K blooms, the table's end colour and matter past the table included.
- **Headless frames** (`headless --frames-every N [--view …]`, §3.3, `headless::FrameOut`): a `Renderer` at α = 1 draws into
  an offscreen RGBA8 texture, the read-back is padded to the row alignment and waited for before the PNG is written
  (`png`), to `DIR/frames/f_<step>.png` at every step index N divides — the chunks are cut there, so a frame is the state of
  the step in its name, byte for byte the image of the dump of that step. A run with frames needs a calibration (exit 6).
- **What is proven** (testing.md: `blackbody`, `views`, `glow`, `frames`): the table against external anchors, the views
  against a CPU formula written from the contract, the glow against a CPU model of its passes, the frames against the dumps.
  All on synthetic worlds — a real star's image and the bloom's cost at the world's size are NOT PROVEN.

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
  automatic slow-down — §1.8), but it changes how many steps run, never what a step does (§1.8.7).
- **In a headless run** the observers sit in `sr_engine::watch`, outside `step/`: `Watch::update` takes a block summary,
  widens it to f64 and feeds `Tracker` and `StageTracker`. A frame loop gets it from `Step::poll_frames`; headless runs
  bare steps (`Step::run`) and takes `Step::summarise` after each chunk of them (one submission, waited for). `headless`
  writes what the observers keep and stops on `--until`. The frame's chunk size (`headless::FRAME`) is the one thing the observers' condition changes — it sets how
  many steps a frame runs, never what a step does. Details: `docs/agent/readouts.md`.
- **G-WATCH** (the `watch_only` scope's static scan, M0-T16; contract §5) is the check that grades this boundary.

## Where a number lives
Not here. Tunable constants → `assets/physics.json` (§2.7); measured ones (ending thresholds, the top rung, the render
reserve, the readout anchors) → `assets/calibration.json` (§2.11, written by `sandbox-reactions calibrate`); strings →
the string table (§4.1); tolerances → §5 (set before the first run, never loosened in a build block, §0.4). Code
constants that carry a contract number (`dt::GUARD_EVERY`, `boxfit::REFIT_EVERY`, `time::LADDER`) are named here, never
their values.
