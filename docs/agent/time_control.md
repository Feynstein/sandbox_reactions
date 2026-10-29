# Time control — the rungs and Max, the frame plan, the cap, pause and step, the slow-down

Written by M0-T27 (2026-10-11), Max by M0-T123 (2026-10-18), from `milestones/m0/m0_contrat.md` §1.8, §3.2, §4.3 and the handoffs of M0-T22–M0-T24.
The contract owns every number, formula and string; this page names the parts and points to the section that fixes
them, never a value §1.8 or `assets/calibration.json` holds. The app's frame loop is wired by M0-T84 (the frame loop), M0-T85 (the top bar), M0-T86 (the banner)
and M0-T87 (the world's actions); § The frame loop in the app says how. The step side of a frame — `encode_frame`, the
latch, `poll` — is in `docs/agent/architecture.md` § A frame and the latch.

## Where it lives — `sr_physics::time`, pure logic (§1.8, §3.2)
`crates/sr-physics/src/time.rs` (M0-T24): no GPU, no wall clock of its own, no engine type. The app hands it each frame's
wall time and each read-back frame; it hands back how many steps the next frame runs. Time never touches the physics
(§1.8.7): nothing here reaches Δt, a pass or a label — only the count of steps a frame runs (G-WARP, M0-T110).

| §3.2 | `TimeControl` |
|---|---|
| the rungs, from calibration's top rung | `TimeControl::new(top_rung, render_reserve_ms)` — both `Option`s from calibration (§2.11); `rungs()`, `rung()` |
| `select(k)`, `faster()`, `slower()` | the same; keys 1…n are `select(0…n−1)`, out of range ignored; the ends hold |
| `toggle_pause()`, `step_once()` | the same; `paused()` |
| `plan_frame(estimates) → {steps, f_target}` | `plan_frame(Estimates { wall_s }) → Plan { steps, f_target, slow_down }` |
| `on_report(report)` | `on_report(&Report { steps_run, advanced, dt_next, step_ms, latch_fired })` |
| `on_event(event)` | `on_event(Event::{Ignition, CoreCollapse, Supernova})`; `banner()` |
| `speed_text()` | `speed_text() → SpeedText { rung, running, paused }` |
| the slow-down setting | `slow_down()`, `set_slow_down(on)` — on at start (§1.8.5) |

Helpers: `top_rung_from_life(life_sun_sb)` (§1.8.2's TOP, which caps the numbered rungs) and `sig2(x)` (two significant
digits, §4.12 — `{actual}`). `Plan` carries `slow_down` so the engine's `FramePlan { steps, slow_down }` is built from it alone.

## The rungs (§1.8.1, §1.8.2)
- ×1 is one sandbox time unit per wall second (§1.8.1). The ladder is `time::LADDER` (value, label), §1.8.2's list.
- **Before calibration** (`top_rung` none): the whole ladder, then **Max**. **After**: the ladder rungs below TOP / 1.5
  (TOP from `life_sun_sb`, §1.8.2, §2.11 — it caps the numbered rungs and is no rung), then Max. A top rung that is not
  a positive finite number is refused (`new` returns `Err`), as is a negative or non-finite render reserve.
- **Max** (M0-TJ2, §1.8.2) is the last rung of `rungs()`: label `MAX_LABEL` ("Max"), value `f64::INFINITY`, so it sorts
  above every numbered rung for `select`, `faster`, `slower` and the slow-down's "only ever slower". Key 0 is the app's:
  `select(rungs().len() − 1)`. `faster` at Max stays; `slower` leaves it for the numbered rung below.
- The game starts at ×1, running, the slow-down on. The labels are what `time.rung` shows (§4.3), not thousands-grouped
  (§4.12's "ladder rungs as written").

## A frame — the plan (§1.8.3)
Each frame, running: `owed += rung / f_target`; `N = min(N_max, ⌈owed / Δt_est⌉)`; the plan's steps go to the engine,
and the frame's report, when it comes back (≤ 2 frames late, `Step::poll`), books what was actually advanced. **At Max
none of that is owed**: N = N_max in every frame, `owed` is 0 and stays 0 (set on selecting Max, never added to, its
reports book nothing), whatever sim time the steps advance. Max's infinite value is never put through the arithmetic
(⌈∞ / Δt⌉ would plan `u32::MAX` steps, ∞ − ∞ a NaN).
- **Δt_est** is the last `dt_next` read back. None before the first report and while the world is empty (an empty box's
  Δt is the largest f32, M0-T21): then one step a frame while anything is owed, and an empty world's frame resets owed
  instead of booking its sim time.
- **owed is booked latency-safe** (M0-T24's default 1, `milestones/m0/tasks/M0-T24.md`): `plan_frame` takes N·Δt_est
  off owed when it plans; `on_report` gives that back and takes the advanced sim time — over time exactly §1.8.3's
  `owed −= advanced`, but a frame in flight is never planned twice. Reports arrive once per frame that ran steps, in
  order.
- **owed is bounded** to [−Δt_est, one frame's sim time]: the lower bound is §1.8.3's; the upper is M0-T24's default 2 —
  what the cap or the latch left unrun is shown by the readout, never repaid later at another rung.
- **N_max** = max(1, ⌊(1000 / f_target − R_reserve) / c_step⌋), c_step the mean cost per step over the last frames
  §1.8.3 names (`COST_FRAMES`), R_reserve calibration's `render_reserve_ms` or `RENDER_RESERVE_MS` until it exists. The
  cost comes from `FrameReport.cost` (M0-T23, `Step::set_timing`); a latch-stopped frame's cost is skipped. **N_max is 1
  until a cost is read back** (M0-T24's default 3) — so the app must turn timing on, or feed its own `step_ms`: the web
  build without `timestamp-query` has no cost (M0-T23; flagged to M0-T84).
- **Slow motion** (less than one step a frame owed): §1.8.3's interpolation α is the renderer's, computed by the app —
  § The frame loop in the app.

## The cap is never silent (§1.8.4)
A frame **binds** when N_max < ⌈owed / Δt_est⌉. When it bound in at least `CAP_BINDS` of the last `CAP_WINDOW` frames
(§1.8.4's counts), `speed_text().running` is `Some(actual)`: the sim time the last reports advanced ÷ the wall time of the
last running plans, through `sig2` — the readout "Speed ×{rung} (running ×{actual})" (`time.speed_capped`, §4.3).
**At Max the readout always carries it** (`running` is `Some`, "0" before any frame advanced), the string being
`time.speed_max` (§4.3, M0-T125). A rung change, by the player or the slow-down, clears the cap's window and
`{actual}`'s and sets f_target (60 at a numbered rung, 30 at Max, from Max's first frame); the cost window is kept (cost
per step is the physics', not the rung's).

## Pause and step (§1.8.5)
- `toggle_pause()` stops stepping: a paused `plan_frame` plans 0 steps and records nothing in any window.
- `step_once()` while paused queues exactly one step, which the next plan runs (one per frame, one per press); while
  running, it pauses. A single step owes nothing.

## The slow-down and the latch (§1.8.5)
- `on_event` with the setting on: ignition → ×1, core collapse → ×0.1, supernova → ×0.1 — only ever slower (an event
  whose rung is not below the current one does nothing), never an automatic return; `banner()` names the event until the
  rung actually changes (`slower` at the lowest rung keeps it). With the setting off, `on_event` does nothing.
- **The latch** is the engine's (`step::latch`): an armed set with `Plan.slow_down` on stops the frame on the GPU, and the
  report says `latch_fired`. Time control then clamps owed to ≤ 0 — the stopped steps are never owed — and the app sends
  `on_event(Event::CoreCollapse)` (the drop to ×0.1; wired by M0-T86 — § The frame loop in the app). With the setting off the frame runs
  through, and any shortfall shows in `running ×`.
- Ignition and supernova are the observers' events (§1.9.4), not the latch's: the app feeds the `Watch`'s events to
  `on_event` (M0-T86).
- **G-LATCH is NOT PROVEN (synthetic)** until M0-T111: the `latch` scope proves the mechanism on a test hook; the
  contract's scenario (the Massive preset from an iron-core dump) is M0-T111's.

## The 30 frames at Max (§1.8.6 — FROZEN, Q2; M0-TJ2)
f_target is 60 at every numbered rung (`F_TARGET`) and 30 at Max (`F_TARGET_MAX`) — no switch, no window: selecting Max
sets it, leaving Max sets it back, so N_max = ⌊(1000 / 30 − R_reserve) / c_step⌋ from Max's first frame. The sim time per
wall second at Max is what the card advances, shown by the readout; no performance number stops the build or holds a
lot (the lead, 2026-10-18, M0-TJ2: «we do not block anything on performance»). The earlier 30-frames switch at TOP
(`F_TARGET_TOP`, `QUIET_FRAMES`, the 30-of-60 and 120-quiet counts) is removed.

## The frame loop in the app (§1.8.3, §1.8.5, §3.1, §4.2; `crates/sr-app/src/desktop.rs` `mod sim`, `ui/banner.rs`)
M0-T84–M0-T87. `sr_physics::time` stays pure; the app owns the wall clock, the engine and the texture. `Sim` holds the
`TimeControl`, the engine's `Step` on **eframe's own device** (one `RenderState`), a second `State` for α, the cell image
registered as a native egui texture (nearest filtering), the `Watch` (the observers the headless run uses) and the banner's `Feed`.
- **One frame, in order** (`Sim::frame(now, frozen)`, called once per egui pass after the top bar's commands ran):
  1. `poll()` — never blocks: `Step::poll_frames` hands back the frames read back, in order; each report goes to
     `TimeControl::on_report` (a **forced** frame's — a capture's `steps` — with `steps_run` 0: its Δt only, it owes nothing);
     a planned frame the latch cut goes to `Feed::latch`; the block summary goes to `Watch::update` and each fresh event to
     `Feed::event`.
  2. The plan: `plan_frame(Estimates { wall_s })` with the wall time since the last pass (1/60 s on the first). A capture's
     forced steps replace it (chunks of `FORCED_CHUNK`, slow-down off so the latch never cuts them).
  3. α (below), then, when steps > 0, the state is copied to the previous-state buffers **before** `encode_frame(&FramePlan
     { steps, slow_down })`; the frame is queued as in flight (forced or not) so its report is told apart.
  4. `render` draws the active view into the texture with α, one submit. The image is drawn by the central panel; the
     status bar's steps per frame is the **mean of the frames that ran any steps in the last second** (a mean over every frame
     reads 0 at ×1, where a Sun-like star is owed less than one step a frame).
- **Interpolation α (§1.8.3, TW-E4)**: when the rung owes less than one step a frame (per-frame sim time `rung / f_target` below
  Δt_est) and the clock runs, α = 1 + owed / Δt_est, clamped to [0, 1] — the previous state just after a step, the current one
  when the next is due; otherwise α = 1 (any rung owing a step or more a frame, Max, paused, a forced frame, no Δt yet). The app
  **mirrors** `TimeControl`'s owed (no accessor) with the same bounds: [−Δt_est, one frame's sim time]. The contract's
  owed / Δt_est is ≤ 0 under its own bound; the `1 +` is the app's reading, declared by M0-T84.
- **Empty world**: Δt is the floor gas's (≈ 0.45 t.u., M0-D96), so the clock steps once per ~27 frames at ×1 and an edit
  (a drop) lands at the next step, ≤ ~0.45 s later. A new world starts empty, at ×1, running, the slow-down on.
- **The commands** reach the clock before the frame: the top bar's buttons and the keys (`ui::topbar::apply`, running.md
  § The window) — Pause/Play, Step (`step_once`: one step the next plan runs; while running it pauses), a rung, Max, ±, the
  slow-down. The world's two (`WorldBar::apply`) do not touch the clock: the edge choice goes to `Step::set_edge_mode`, Clear to
  an engine `Edit::Clear` for the next P0.
- **The banner (§1.8.5)**: `ui/banner.rs` is a view model — `view(&TimeControl) → Option<Banner>` is the text of
  `TimeControl::banner()`'s event (`banner.ignition`, `banner.core_collapse`, `banner.supernova`, byte-exact) — and `Feed`, the
  door of the slow-down's two sources. `Feed::latch` is a frame the engine's latch cut (a core collapse, ×0.1);
  `Feed::event(time, object, kind)` takes the observers' `ignition`, `core_collapse` and `supernova` (`sink_formed` and `ending`
  are ignored). Each (object, event) is booked once (§1.9.4), and a Tracker `core_collapse` that follows a latch cut is the
  same collapse and is **swallowed once**, so no second banner. The clock stays the one place that says only-slower, the
  rung and the setting off; the banner shows until the player's own rung change — **no automatic return**. `desktop.rs` reads
  `banner::view` *after* the frame, so an event read back in this pass has slowed the clock already, and draws it over the
  world's top centre.
- **Clear starts the watch over**: `Sim::clear` makes a new `Watch` and `Feed` (the old objects are gone; a reused object id
  would otherwise never banner again). Frames in flight at the click still carry the old world's summary — unmeasured.
- **Not proven**: the banner on screen (a collapse or supernova is not reachable on lavapipe within a capture); `Watch::update`'s
  per-frame cost on a real GPU (it decodes the whole block summary every frame; nothing blocks on it, M0-TJ2); fps at ×1 off
  lavapipe (25 there, linux-pc, M0-T84). The readout's `{actual}` is written as `sig2` writes it, **ungrouped** (×4200) while
  `format::actual_number` groups it (4,200) — an unsettled contract point (§4.12 against M0-T24's flag), the lead's ruling.

## What grades it
The `time_control` scope (`bash tests/cpu.sh sr-physics time`, no GPU; R17 spells §5's `time-control` so) — 23 cases
over §1.8 and §3.2 with a zero-latency driver, plants `slowdown-auto-return` and `time-max-paced` (Max paced like a
numbered rung, `docs/agent/testing.md`). The app side is graded by
`ui_topbar` (14 cases, the bar and the keys), `ui_banner` (12) and `ui_world` (7), all without a window, and the `capture` scope's
`actions` case (the bar on screen) and the `desktop` scope's `world` case (the loop drawing) — `docs/agent/testing.md`. G-WARP (M0-T110),
G-TOP and G-FPS (⏱, the Quadro only, R12) grade it in the running game later.
