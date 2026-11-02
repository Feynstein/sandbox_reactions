# Running — the launchers, the binary, adapters, the web build, the private-display rule

Written by M0-T9 (2026-10-09) from `milestones/m0/m0_contrat.md` §3.3–§3.6 and §6. Commands run from the repo root.
Ports, deadlines and flags live in `tools/pb/launch.json` (§3.6) and the binary's usage line — this page points to them
and never restates a value they hold. Build first: `~/.cargo/bin/cargo build --release --locked -p sr-app` (cargo is off
the PATH on linux-pc, `~/.cargo/bin` is on it on win-laptop — `docs/agent/testing.md` § Cargo scopes); the binary lands in
`build/target/release/` (§6.5), `sandbox-reactions.exe` on Windows (§6.6) — a script never assumes the suffix away.

## The two ways a person starts the game (the lead, double-clicking)
| Box | Launcher | What it does |
|---|---|---|
| linux-pc (`laserax-ai`) | `start.sh` (Files → Run as a Program) | `launch.py start game --task launcher`, waits for Enter, `launch.py stop` |
| win-laptop (`Laser2025-20`) | `start.bat` (the double-click) | the same, through `py -3.12 tools\pb\launch.py …` |

On win-laptop `start.ps1` is not a double-click: Explorer opens a `.ps1` in Notepad (the lead, M0-V2). It runs from
PowerShell: `powershell -ExecutionPolicy Bypass -File start.ps1`. All three take an optional service name (`bash start.sh game-xvfb`) so a check can run the launcher on a route that
opens no window on the lead's screen. `game` is the only service that asks (`--asked`): the double-click is the ask
(R4, §3.6). Not for agents: a visible window is the lead's to run and to watch.

A launcher reads `launch.py start`'s exit status (M0-D129). On a failed start it says the game **did NOT start**, names the log
(`milestones/m0/logs/launcher.launch.<service>.log`), asks Enter only to close the window, exits non-zero and does **not** call `stop`
(nothing started); it never says «Sandbox Reactions is running» then. The success path is unchanged. `start.bat` and `start.ps1`
are NOT RUN on win-laptop yet (R15).

## The binary `sandbox-reactions` (§3.3)
`sandbox-reactions --help` prints the usage line. What exists today (M0-T4–M0-T8); the rest of §3.3's flags arrive with the
block that builds their feature and are refused with exit 4 until then:
- **Desktop** (no subcommand): `[--adapter S] [--status-port P] [--offscreen-window] [--capture SCRIPT --out DIR]`.
  - `--status-port` serves `GET /status` on 127.0.0.1 only (§3.4); off without the flag, never in the web build.
  - `--offscreen-window` places the window outside every monitor, never activated, out of the taskbar (§3.3 [M0-TJ3]).
  - `--capture` runs a scripted session, one PNG per step plus `captions.json`, then exits 0 (§3.3; the harness is
    `tests/capture/`). An action not built yet is refused with `SR-ERROR`, exit 4.
    The actions, their arguments and what each refuses are in § Captures below; each step also prints `SR-CAPTURE-STATUS <id> <status.line as the
    step's last frame drew it>` (M0-T84).
- **Headless**: `headless --scene <scenes/FILE.json | preset:<sun|massive|giant>> --steps N [--world WxH]
  [--edge leave|bounce] [--max-steps N] [--out DIR] [--timing] [--dump-every N] [--load-dump FILE [--continue]]
  [--adapter S | --cpu-reference]` — the
  same step with no surface and no display (§6.3); `DIR/summary.json` per §2.12.2 and, with `--out`, the final state dump
  `DIR/dumps/s_<step>.bin` (§2.12.3; `--dump-every` adds one at every step index N divides, counted from the scene;
  `--load-dump` starts from a dump, M0-T26, the observers starting over). Beside each dump of a GPU run sits its observers'
  snapshot `s_<step>.observers.json` (M0-D76, §0.5 [M0-D76]); `--load-dump FILE --continue` restores it and goes on as the
  cut run would have — give the cut run's own command plus those two flags: `--steps` and `--max-steps` count from the life's
  start, summary.json's `steps` and `wall_s` are the whole life's, and a snapshot missing or of another step, scene label or
  world is exit 4. `--world` and `--edge` stand over the scene's `world` (M0-T25); `--timing`
  adds `timing` to the summary (M0-T23). The scene's `switches` (§2.12.1, each on when left out) build the step's passes:
  `gravity` P2, `hydro` P4, `heat` P5, `reactions` P6, `sinks` P3 and `neutrinos` P7 — P7 only with `reactions` on, S_ν being
  P6's (M0-D47); `--timing` lists the passes built. `--cpu-reference` (M0-T37) runs the CPU f64 twin from the same scene instead of
  the GPU — worlds up to 128 × 128 (exit 4 above), the same summary.json and dumps (the box record the whole world's);
  `--adapter`, `--timing` and `--load-dump` are the GPU's and refused with it. stdout: first line `SR-ADAPTER …`, last `SR-HEADLESS DONE …`; errors `SR-ERROR`,
  warnings `SR-WARN`. Exit codes: §3.3. The file formats: `docs/agent/architecture.md` § The files. sr-app's usage line
  (`--help`) lists these flags (M0-T37).
- **`calibrate`** (M0-T66b–M0-T70): `calibrate --out FILE [--adapter S] [--only a,b] [--max-steps N] [--bracket LO,HI] [--resume DIR] [--progress-every N] [--checkpoint-every M]` —
  § Calibrating below. (sr-app's usage line lists `--only`, `--resume`, `--progress-every` and `--checkpoint-every`; `--max-steps` and `--bracket` are refused with exit 4 where they
  do not apply.)
- **`--measure-ui <frames> [--calibration FILE]`** (M0-T95): § Measuring the UI below — a visible window, asked of the lead each time (R4).
- Later blocks add `bench` and the headless run's `--until`, `--frames-every`, `--edits` (§3.3).

## The window (§4.1–§4.3, §4.10, §1.11; `crates/sr-app/src/desktop.rs`, `ui/`)
What the window holds (M0-T84–M0-T93b, M0-T126–M0-T129, M0-D119–M0-D123): the top bar, the world, the banner and the status bar here; the left
and right panels, the legend and the zoom in § Tools and presets below. Sizes and colours are §4.11's tokens, named in the code, never restated here.
  **The theme** (`ui/theme.rs`, M0-D119, M0-V12): `theme::install(&ctx)` runs once at start-up, from both entries (`desktop.rs`'s
  creator and `web.rs`), and puts the tokens' `Visuals` on the context for good — egui's dark preference pinned, both of its themes
  dressed — so the window is dark and drawn from the tokens whatever the system's light or dark setting says (a light browser had
  drawn egui's light fills under the light text, the bar's buttons blank). No other file writes a colour literal; graded by
  `ui_theme`, the pixels on screen NOT PROVEN.
- **Layout**, top to bottom: the **top bar** (44 px, `bg.panel` with `border.panel`), the **world** (the centre between the left and the right panel, `bg.space`; the legend and the place hint under it),
  the **status bar** (24 px) carrying `status.line` live at §4.11's small 12 px (`STATUS_FONT`; egui's body text is 13, M0-D123). The world is a 600 × 400 cell image drawn at the integer scale
  s = max(1, ⌊min(w/W, h/H)⌋) physical px, centred with its corner on a pixel and nearest filtering (§1.11), so every cell shows
  whole; a new world is empty (`bg.space` exactly, pinned by the `desktop` scope's `world` case). The banner sits over the world's
  top centre (`banner.bg` / `banner.text`, 20 px) — `docs/agent/time_control.md` § The frame loop in the app.
- **The top bar**, left to right: Pause/Play, Step · the rung buttons (the ladder below TOP / 1.5, Max last; the selected one in
  `accent`) · the speed readout (`time.speed`, `time.speed_capped`, `time.speed_max`, `time.paused`; **`accent` while the clock is
  paused**, so a still world says so, §0.5 [M0-V12]) · the slow-down checkbox ·
  `edge.label` with two radios (Leave for good, Bounce back — not `accent`: §4.2 lists tool, rung and view only) · `world.clear`.
  Every text is a `strings.rs` key (byte-exact, §4.3). The bar is **drawn from a view model**: `ui/topbar.rs`'s `view(&TimeControl,
  &WorldBar)` computes it, `desktop.rs` only draws it and hands back `Command`s; `Command::apply` does the clock's part and
  `WorldBar::apply` the world's, returning an `Effect` (`Edge` → `Step::set_edge_mode` at once, in any state; `Clear` → an engine
  `Edit::Clear` queued for the next step's P0). That is what the `ui_topbar` and `ui_world` scopes grade without a window.
  **Nothing in the bar moves when the clock changes** (the lead at M0-V12: «I dont like when an interaction in a bar moves the whole
  bar»): the Pause/Play button is as wide as the wider of `time.pause` and `time.play` (plus the button padding), and the speed readout
  sits in a slot as wide as its longest usual text — a capped ×1000 or Max's six-digit speed — so a pause, a rung or the cap resizes
  nothing after them. The widths are measured at the bar's 14 px text in `draw_top_bar`; the view model carries the paused state, and
  `ui_topbar` pins that and the single width.
- **Clear world** opens an `egui::Modal` (`world.clear_confirm`, `world.clear_yes`, `world.clear_no`); a Clear is queued by yes
  alone, once. It lands at the next step's P0, so **paused, the old star stays on screen until a step runs**. The rung, the
  slow-down and the edge mode are kept; the observers (`Watch`) and the banner's feed start over.
- **Keys** (§4.10) — what is bound **today** (`ui::topbar::key_command`); a key with Ctrl, Alt or Command held is not the
  bar's, and none is while a text field has the keyboard (`key_commands` takes the presses it maps out of the pass's input, so a
  focused button is not pressed a second time by the Space that paused the clock):

| Key | Action | Command |
|---|---|---|
| Space | Pause / Play | `TogglePause` |
| . (period) | Step (one step while paused; pauses while running) | `Step` |
| 1 … 9 | numbered rung n; past the last numbered rung (the ladder is cut at TOP / 1.5) it does nothing — never Max | `Numbered(n)` |
| 0 | Max | `Max` |
| + or = / − | faster / slower | `Faster` / `Slower` |
| Delete | opens the Clear dialog | `OpenClear` |

  The dialog's buttons are not key-bound (the buttons are clicked). B · E · H · C, `[` `]`, F1–F4, Esc, Home and the mouse are the panels' —
  § Tools and presets, with their own tables; a key in neither table does nothing.
- **Scripting it** (`--capture`, § Captures below): `rung` (the key), `slowdown`, `edge`, `pause`, `steps`, `preset` drive the bar. `check.sh actions`
  compares each step's top bar (44 px) with the step before and the accent's column (×1 → ×10 → Max); `edge-leave` is compared with `edge` (both
  paused, so only the radios differ). **No capture action opens the Clear dialog** and no key is scriptable — the dialog is seen in a visible
  window with the lead's yes (R4) or is NOT PROVEN (M0-T87's flag, M0-TV2's).
- **Not proven by the scopes**: the bar, the banner and the dialog *on screen* — the view models are graded, the drawing by a
  capture's pixels (lavapipe) and the lead's launcher run (the real GPU).

## Tools and presets — the left and right panels, the legend and the zoom (§4.1, §4.2, §4.4–§4.6, §4.10, §1.11; `ui/left.rs`, `display.rs`, `legend.rs`, `zoom.rs`, `right.rs`)
Written by M0-T96 (2026-11-02) from the handoffs of M0-T89–M0-T95 and M0-T126–M0-T129. Every panel is **a view model first**: the file computes
what is shown and what a click or key means, `desktop.rs` only draws it and hands the commands back, and the scope named on each line grades it
with no GPU and no window. The pixels on screen are NOT PROVEN by those scopes (a capture is lavapipe's, § Captures); the pointer is not scripted
but for `hover` and `select`.
- **Layout.** The left panel (220 px) holds, top to bottom: tools, the brush size, the elements, the presets, the views with their display
  controls. The right panel (300 px): the star readout, then the cell inspector. The world sits between them; under it the legend (8 px below,
  as wide as the world), under that the preset's place hint. The scale is `s · z` (§1.11, the zoom's `z`), clipped to the world's `z = 1` rectangle
  (`legend::layout`); `s` gives way (stays 2 at 1240 × 872) when the world, legend and hint would not fit.
- **Tools** (`ui/left.rs`, `Tools`; scope `ui_tools`, M0-T89): Brush, Eraser, Heat, Cool — one selected, the selected one in `accent`. The nine
  paintable elements in the registry's order with their §1.5 swatches, Hydrogen selected at first; neutron matter is never listed and never
  selectable. The brush size slider and `[` `]` (by 2, stopping at both ends); the range and the default are §4.4's. A change of tool keeps the
  element, the size and the view, and a change of element or view keeps the tool. **A held left button queues one edit per frame**
  (`left::frame_edit`): the radius and the species of *that* frame, each tool its own engine `Edit`; the brush outline follows the cell under
  the pointer (`left::outline`). Source read only, not measured: a held brush while paused or at ×0.1–×1 queues at most 8 edits per step (the
  queue's `QueueFull` is ignored), and M0-D110's one-step-late paint (M0-T89's handoff).
- **Presets** (`Presets`; scope `ui_presets`, M0-T90): three buttons, each with its note (the mass and the ending, `presets.*_note`). A click on a
  button arms it (one armed at a time, the armed one `accent`, the tools untouched); `presets.place_hint` shows while one is armed; **the next
  left click in the world drops it there** through `Sim::place` (the engine's `Edit::drop_preset`), and the click is spent — it reaches the
  preset or the brush, never both. A drop that does not fit shows `presets.no_room` for 3 s and **keeps the preset armed**; Esc (only Esc)
  disarms. The cloud's outline follows the cursor at `cloud_radius`'s radius, the engine's own (`scene::cloud_shape`, capped at 150 cells),
  and agrees with the refusal. A drop while the clock is paused asks for one Step to show (M0-T84's flag; the engine takes the edit at the next
  step's P0, as Clear does).
- **Views** (`Views`; M0-T91, in `ui_tools`): Glow (active at first), Heat, Element, Density — the four buttons, the active one `accent`, and F1–F4.
  A switch leaves the brush, the element and the size alone. The renderer draws `views.active()`.
- **Display controls** (`ui/display.rs`; scope `ui_display`, M0-T127b; the renderer's side `Renderer::set_display` and `Display`, scopes `views`
  and `extents`, M0-T126, M0-T127a; why, `reports/display_range.md`), under the four buttons, for the **active** view:
  Glow — `views.glow_white` («Full brightness at»), a slider logarithmic in T over §4.2's span with the value typed in place, and `views.reset`
  (S back to 3,000 K, where the image is today's, bit for bit); Heat — `views.range_from`, `views.range_to`, `views.fit`, `views.reset`;
  Density — the same over Σ; Element — none. A typed value takes digits with `,` in 3-digit groups only («1,5» is junk, not 15), a `.`, an
  e-notation and, for temperatures, an optional `K`; out of range is clamped, anything else restores the shown value. From and To stay 0.1 decade
  apart (the moved end stops at the other). The settings are kept by a switch of view, and the renderer is asked only when one changed (a drag
  over 20 places is 20 asks, 120 frames at one value one ask). **Fit to view** reads the matter's extent from a 1,024-bin histogram of log T or
  log Σ (cells with Σ ≥ Σ_vac only; `Renderer::fit_range`): L is the lower edge of the 0.5th percentile's bin, H the upper edge of the highest
  non-empty bin, widened to 0.1 about the middle and slid inside the span; with no matter it does nothing. It is one **blocking** read-back,
  so on the web build it is a no-op. NOT PROVEN: Clear keeping the settings (by construction — `Sim::clear` queues `Edit::Clear` alone — no case).
- **The legend** (`ui/legend.rs`; scope `ui_legend`, M0-T128): the active view's title, a 12 px bar and its ticks at every power of ten inside the
  shown range, **placed by log T, never linearly** (a label that would overlap its left neighbour is dropped, its tick kept; the first and last
  labels stay inside the bar), then the marks: S in Glow only (`legend.white`, an `accent` triangle and line) and the hovered cell's value
  (`legend.cursor`; none over vacuum, off the world or in Element). The Glow bar is the cell image's colour at S, the Heat and Density bars the maps
  over the range, Element's ten swatches carry their names. A cursor past the shown range stands at the bar's end. Not proven: the drawing (a
  capture's shot, read by eye, M0-TV3); the marks' labels may overlap when S and the cursor are within a label's width (the cursor's flips left).
- **The zoom** (`ui/zoom.rs`; scope `ui_zoom`, M0-T129): the wheel zooms in and out **about the cell under the cursor** (z from 1 to 8, held at the
  ends), the middle-button drag moves the world with the pointer, clamped so the zoomed image always covers the viewport (nothing to pan at
  z = 1), **Home** returns to z = 1. One `cell_at` mapping serves every reader of the pointer — the brush, the preset drop, the right click and the
  inspector — equal to the z = 1 mapping at z = 1, so a brush at z = 8 edits the cell the zoom names. The zoomed image's corner sits on a physical
  pixel at 1, 1.5 and 2 points per pixel. NOT PROVEN by a case: the egui path (wheel events, drag, Home, the painter's clip — no pointer in a capture);
  the remnant at z = 8 is M0-TV3's on a GPU.
- **The star readout** (`ui/right.rs`, `Stars`; scope `ui_star`, M0-T92): the selected object's stage (the Supergiant rows at M ≥ M_up), `star.mass`,
  `star.fate`, or `star.ending` once a remnant holds, `star.age`, `star.rate` (**hidden while paused**), `star.surface`, `star.core`; `star.none`
  with no object, `star.select_hint` with several. The app runs its own tracker, stage tracker, age clock and predictor (beside the banner's
  `Watch`, so the GPU scopes stay out of the UI's claim); `banner::Frames` owns `Stars` and restarts it with the world. **The selection is by the
  tracker's id**: the heaviest by default; a right click selects the nearest centre within max(R, 3 cells); a click on empty space keeps the
  selection; a list that shifts under it keeps the id; a new world empties it.
- **The cell inspector** (`right::Reader`, `Asker`; scope `ui_inspect`, M0-T93b; the engine's side `Inspector::inspect`/`poll`, scope `state`,
  M0-T93a): the hovered cell's mix (up to three species of at least 0.5 % by mass, the largest first, equal shares in species order),
  `inspect.temperature` (in kelvins as the star readout translates it), `inspect.density`, `inspect.speed` with Mach = |u| / √(2Π/Σ) from the
  engine's EOS; `inspect.vacuum` below Σ_vac, `inspect.empty` off the world (D6). **One request is in flight**: the engine's reply comes within
  2 frames and an empty poll costs under 5 ms; the latest hovered cell is asked after each reply and nothing is asked off the world. NOT PROVEN
  (synthetic): the Asker over 120 hovered frames ran with simulated replies, not the real read-back queue; the real `Inspector` ran only in a
  lavapipe capture's 30 hovered frames.

| Key | Action | Where |
|---|---|---|
| B · E · H · C | Brush · Eraser · Heat · Cool | `left::key_command` |
| `[` · `]` | brush size −2 / +2 | `left::key_command` |
| F1 · F2 · F3 · F4 | Glow · Heat · Element · Density view | `left::view_key_command` |
| Esc | disarm the armed preset (read only while one is armed) | `left::preset_key_command` |
| Home | the zoom back to z = 1 | `desktop.rs` key pass → `Zoom::home` |

| Mouse | Action |
|---|---|
| left button | the tool's edit once per held frame, or the armed preset's drop (never both) |
| right click | select the star under the cursor |
| hover | the cell inspector (and the legend's cursor mark) |
| wheel · middle drag | zoom about the cursor · pan |

The §3.3 capture actions for all of it — `tool`, `element`, `paint`, `view`, `preset`, `select`, `hover`, `glow_white`, `range`, `fit`,
`display_reset`, `zoom` — are in § Captures.

## Captures — `--capture SCRIPT --out DIR`, the full action list (§3.3; `crates/sr-app/src/capture.rs`, `tests/capture/`)
A script is a JSON list of steps `{"id", "caption", "actions": [ … ], "frames": n}`: `id` matches `[A-Za-z0-9][A-Za-z0-9.-]*` and is unique;
`caption` is required; `actions` may be left out; `frames` is a whole number from 1 to 100,000 (1 when left out); any other key in a step is
refused. Each action is an object with **exactly one key** — the one exception is `zoom`, which takes two. The run applies a step's actions in
order, lets `frames` frames run, takes one screenshot and writes `DIR/<id>.png` and, at the end, `DIR/captions.json` (review_page.py's shape);
exit 0. The app stands still until the first step begins and from the pass that asked for a screenshot until the PNG is written, so a shot shows
the state after exactly `frames` passes of its step, whatever the machine's speed. A new world is empty, at ×1, running; at ×1 a Sun-like star
is owed less than one step a frame and an empty world steps once per ~0.45 s (Δt is the floor gas's, M0-D96), so a capture that needs a state asks
for `steps`.

| Action | Argument | What it does (the door it goes through) | Refused at parse (exit 4, before any window) |
|---|---|---|---|
| `pause` | `true` / `false` | the clock paused or running (the bar's Pause/Play) | not a boolean |
| `steps` | whole number n | n simulation steps through the real frame loop, paused or not | not a whole number |
| `preset` | `"sun"` / `"massive"` / `"giant"` | the panel's path: arms the preset and drops it **at the world's centre** (`Sim::place`) | any other name; **any second key — §3.3's `"at": [x, y]` is not built** |
| `rung` | key 0–9 | the bar's key: 1–9 the numbered rungs, 0 Max; a key past the ladder does nothing | past 9, not a whole number |
| `slowdown` | `true` / `false` | the bar's checkbox | not a boolean |
| `edge` | `"leave"` / `"bounce"` | the bar's edge choice | any other word |
| `tool` | `"brush"` / `"eraser"` / `"heat"` / `"cool"` | the panel's tool button | any other word |
| `element` | a paintable species' key (`"Fe"`) | a row of the element list; neutron matter is not in it | a key not in the list |
| `view` | `"glow"` / `"heat"` / `"element"` / `"density"` | the panel's view button (F1–F4) | any other word |
| `paint` | `[[x, y], …]` | one frame of the held left button per cell, with the tool and element the panel holds | an empty list, a pair that is not two whole numbers |
| `select` | `[x, y]` | a right click on world cell (x, y) | not a cell |
| `hover` | `[x, y]` | the pointer rests on cell (x, y) as the central panel reads it, **until another `hover` moves it** — it wins over the real pointer; a cell off the world is the pointer off the world | not a cell |
| `glow_white` | kelvins > 0 | the Glow view's «Full brightness at», snapped and clamped as a typed value is | not a positive finite number |
| `range` | `[from, to]` | the active view's range, kelvins for Heat, sandbox Σ for Density, typed in place | not two positive numbers with from < to |
| `fit` | `true` | the active view's «Fit to view» | anything but `true` |
| `display_reset` | `true` | the active view's Reset | anything but `true` |
| `zoom` | `z` 1–8, with `"at": [x, y]` | the view at z centred on cell (x, y) as far as the clamp lets it | z outside 1–8, a missing or extra key, `at` not a cell |

An unknown action is refused by name (`unknown action …`); `NOT_BUILT` in capture.rs is empty today. **Refused at run time** (the capture stops with
the line, a window already open): `glow_white` outside the Glow view, `range` or `fit` outside Heat and Density, `display_reset` in Element
(«the … view has no such control», never a silent no-op), `zoom` at a cell off the world, a `preset` that does not fit, a `paint` the engine
refuses. A pointer click, a key and the Clear dialog are not scriptable. A capture shows the brush's outline at the Xvfb pointer (pre-existing,
M0-T93b).
- **The harness**: `bash tests/capture/check.sh smoke|refuse|actions` (scope `capture`, `docs/agent/testing.md`). `actions` runs
  `tests/capture/actions.json` (21 steps: `start` … `star-zoom`; the Massive preset paused at step 768 for the star's shots) and compares the
  regions each action must change — the top bar, the left panel (220 px), the right panel (300 px) with its lit pixels, the world — and that
  `display_reset` equals the default pixel for pixel and a `zoom` shot's six probed cells keep their colours; `refuse` runs the malformed ones.
  Seven plants hold it red (`capture-*.patch`). Shots are lavapipe's on linux-pc (`xvfb-run`) and `--offscreen-window` on win-laptop; a real
  GPU's picture is M0-TW's.
- **Checked against the code** (M0-T96's adversarial): the table's seventeen actions, all in one script, parse in the built binary (the script's last
  step carried a bad id, so the run stops there with exit 4 naming the id — never an action; logs/M0-T96.capture_actions.log); `preset` with
  `at`, `zoom` out of 1–8 or with one key, `range` with from ≥ to, `fit` false and an unknown name are each refused.

## Measuring the UI — `--measure-ui` (§3.3, §1.8.3's R_reserve; `crates/sr-app/src/measure.rs`, M0-T95)
`sandbox-reactions [--adapter S] --measure-ui <frames> [--calibration FILE]` runs the desktop app with the Sun-like preset dropped at ×1 and measures
**the UI's CPU cost per frame**: eframe's own figure for the frame (the egui pass, the tessellation, the paint; the vsync wait taken out) less what
the app spent on the world itself (its read-back and its frame), after 30 frames of warm-up, the nearest-rank 95th percentile of `<frames>`
samples. It prints `SR-UI p95_ms=<x>` (and names the adapter) and writes `render_reserve_ms` = x + 0.5 into the calibration file; the key is outside
`physics_hash`, so G-CAL holds, and the file is otherwise rewritten as `calibrate` writes it (keys sorted). The time control uses §1.8.3's 3.0 ms
until the key is there.
- **What it is not**: the GPU's time of the presentation (a CPU figure only), and not a number from lavapipe — a **software adapter with no
  `--calibration` is refused** (exit 3, the real file untouched), because a reserve measured there would be in every frame budget.
- **Exits.** 0 done · 3 a software adapter without `--calibration`, or a `--calibration` file that is no calibration (untouched) · 4 `<frames>` of 0 or
  not a number, none, `--capture` beside it, `--calibration` alone — all before any window.
- **Who runs what.** The measurement needs a visible window on the real GPU: **the lead's, asked each time** (R4). The agents' check is
  `bash tests/measure/check.sh run|refuse` (scope `measure_ui`, plant `measure-no-write`): 20 frames on the box's window route into a **copy** of
  `assets/calibration.json` — one `SR-UI` line, x > 0, the copy holding x + 0.5 and every other key as it was, the real file's hash unchanged — and
  the refusals above. It grades the flag, never the number.
- **Owed** (M0-T95): the real run on the Quadro, in M0-V13's window, from the repo root — then a rebuild embeds the file (M0-D62):
  ```
  build/target/release/sandbox-reactions --adapter "Quadro RTX 4000" --measure-ui 600
  ```
  win-laptop: NOT RUN (owed on `Laser2025-20`, R15).

## launch.py — how a service is started, polled, stopped (`tools/pb/launch.py`, §3.4, §3.6)
`python3 tools/pb/launch.py start|stop|status|smoke|list <service> --task <ID>` (`py -3.12 …` on win-laptop). A service's
ready signal is one of three kinds (§3.4): `exit` (headless-boot), `url` (the desktop's `/status` reads `ready` once the
first frame is **presented** — an egui screenshot round trip, never the window's creation), `port` (the web's static
server). `smoke` starts, waits for ready, stops. The services in `launch.json` today: `headless-boot`, `game`,
`game-xvfb`, `game-offscreen` (`web` arrives with M0-T15). **Never reuse or stop a stack you did not start**; `smoke`
probes the port first.

## Adapters (§6.1, §6.6)
`--adapter <substring>` matches the adapter's name case-insensitively; where a name matches on two backends Vulkan wins;
no match is exit 3 naming every adapter seen. Without the flag, wgpu's high-performance preference among adapters that
can present to the window. Every run prints `SR-ADAPTER name=… backend=… driver=…`, and **every performance or physics
number names its adapter** (R13).

| Box | Adapters wgpu lists | Use |
|---|---|---|
| linux-pc | Quadro RTX 4000 (drives the display) · RTX 5090 · "Intel(R) Graphics (RPL-S)" (the Intel iGPU — match it with `--adapter "RPL-S"`; `"UHD"` finds nothing) · llvmpipe (Mesa software Vulkan) | physics scopes `--adapter "RTX 5090"` (`SR_TEST_ADAPTER`, `tests/gpu.sh … physics`) — but a block executed on this box runs its GPU runs on laserax-ai-server's RTX 4090 (R18); timing scopes the Quadro only (`box: laserax-ai`); private-display windows `--adapter llvmpipe` |
| win-laptop | RTX 4080 Laptop · Intel Arc Graphics · WARP where wgpu lists it (UNVERIFIED) | physics reads the 4080 where linux-pc reads the 5090 (§6.4, R15); timing is never measured here (`owed on laserax-ai`) |

The default pick on linux-pc is **not** the 5090 (the display sits on the Quadro): never rely on it — name the adapter.
Results agree across adapters within G-REF's tolerances, never bit for bit (§6.7).

## The private-display rule (R4) — no agent window on the lead's desktop
- **linux-pc.** The lead's session is Wayland with XWayland. Windowed checks run on a **private Xvfb** with Mesa's
  lavapipe: the service `game-xvfb` (launch.py starts the Xvfb, `--adapter llvmpipe`), or `xvfb-run` around a command.
  winit prefers `WAYLAND_DISPLAY` over `DISPLAY` — a command under `xvfb-run` also needs `env -u WAYLAND_DISPLAY -u
  XDG_SESSION_TYPE`, or the window opens on the lead's desktop (found at M0-T8; `tests/capture/check.sh` shows it).
  Wayland ignores a window position, so `--offscreen-window` alone does not hide a window there: `game-offscreen` also
  takes the private Xvfb on Linux.
- **win-laptop.** No Xvfb, Wayland or X11 (§6.6). The window checks use `--offscreen-window` — the service
  `game-offscreen` — on the box's high-performance adapter; the same `/status` ready signal. UNVERIFIED there until the
  lead's box runs it; if the off-screen window cannot present, the window tasks run on linux-pc (§6.6, Q3).
- `game` (a visible window), `--measure-ui` and any other windowed launch ask the lead each time.
- Every toolkit call on win-laptop goes through Git Bash with `MSYS=winsymlinks:nativestrict` (`docs/agent/testing.md`
  § Hazards measured on win-laptop).

## The web build (§3.5, §6.2)
- `cd web && trunk build --release --locked` (trunk 0.21.14, §6.2.1) writes `web/dist/` (git-ignored). `web/Trunk.toml`
  runs a post-build hook, `web/strip-autoload.mjs`, that removes trunk's own wasm auto-loader — the page's inline script
  must run the WebGPU check **first** and load the wasm only on success (§3.5, §6.2.4, Q4). The hook fails loudly if the
  shape it strips is not found. The `web` scope's `build` case grades it (`bash tests/web/check.sh build`).
- Serve `web/dist/` with the standard library only: `python3 -m http.server <port> --bind 127.0.0.1 --directory web/dist`
  (the port is §3.4's; the `web` service of launch.json wraps it from M0-T15).
- The page signals through `document.body.dataset.srState`: `loading` → `ready` after the first presented frame;
  `no-webgpu` (the §4.9 page — `navigator.gpu` or `requestAdapter` missing, or `?no-webgpu=1`, the test hook; the wasm is
  never loaded); `error` (§4.8's message, detail in `data-sr-error-detail`). `srAdapter` carries the adapter description —
  browsers hide adapter names, so it reads sparse. URL parameters for smoke and captures: §3.5.
- No hosting in M0 (§6.2.5). A browser without WebGPU gets the §4.9 page, never a WebGL path (Q4, FROZEN).
- **The web smoke** (`web/smoke.mjs`, M0-T15, §6.2.3): the installed Chrome through playwright-core with the Linux Vulkan
  flags on linux-pc and Chrome's own D3D12 backend on win-laptop — UNVERIFIED until it runs. Chrome is
  `/usr/bin/google-chrome` on linux-pc, `C:\Program Files\Google\Chrome\Application\chrome.exe` on win-laptop.
  `tools/pb/capture_web.mjs` launches Chromium without flags, so it captures only the no-WebGPU page.

### The web build's parameters (§3.5; `crates/sr-app/src/web.rs` `Query`, `SandboxApp::open_with`, M0-T94)
`web/dist/index.html?<key>=<value>&…` — for smoke and captures; each goes through the door the matching capture action uses (§ Captures), applied
before the first pass in this order:

| Parameter | Value | Effect |
|---|---|---|
| `view` | `glow` · `heat` · `element` · `density` | the view's button |
| `rung` | key 0–9 | the bar's key — 1–9 the numbered rungs, 0 Max |
| `preset` | `sun` · `massive` · `giant` | armed and dropped at the world's centre by the panel's path |
| `steps` | whole number n | n steps queued, **then the clock paused** (a page opened with `steps` ends paused) |
| `no-webgpu` | `1` | index.html's test hook (§3.5): the §4.9 page, the wasm never loaded; not the app's |

A key the page does not know is ignored; a known key with a value that is none of its own is an error naming it (`srState` → `error`, the detail in
`data-sr-error-detail`). **On the web the forced steps run one a frame** (`FORCED_CHUNK` is 1; 256 on the desktop): several steps in one submit closed
Chrome's GPU process (8 a frame died at steps=24, 1 a frame ran 200), so `steps=200` is 200 frames. The `web` scope's `preset` case (`node web/smoke.mjs
preset`: `/?preset=sun&steps=200` reaches `ready` within 30 s and the canvas's central 100 × 100 px are not all black) grades the page's loading
with the parameters; **it cannot tell a star from an empty canvas** (`page.screenshot` and `drawImage` of this WebGPU canvas read all zeros) —
M0-V13 names the gap. `--redarm web` is ≈ 10–11 min, the lead's (Rule 4).

## Long runs (§6.4)
`calibrate`, the ⏱ scopes and benches at low rungs can pass 10 minutes on the Quadro: **the lead's**, in one visible
terminal, placed in the V windows. A run over 10 minutes is never an agent's (PLAYBOOK §4 Rule 4).

## Calibrating (§2.11, §3.3, §6.4; `crates/sr-engine/src/calibrate.rs`)
`calibrate` measures the sandbox's own scales — the cold ceilings, the ignition and ending masses, the presets' masses and
lives — and writes `assets/calibration.json`. The file's schema and every bound are §2.11's; its values are never copied
into a page — read the file (`docs/agent/physics.md` § Calibration says what each key is). Commands run from the repo root,
the binary built first (top of this page).
```
build/target/release/sandbox-reactions calibrate --adapter "RTX 4090" --out assets/calibration.json
```
- **`--out FILE`** is merged, never rewritten blind: the file's other items are kept, so one item re-run leaves the rest.
  A file under another `physics_hash` (or not JSON) is dropped with `SR-WARN` and its items are measured again. K9's order
  and Σ_BH ≥ ns_sigma_max_sb are checked on the merged file **before anything is written**; a failure is exit 6 and the file
  is untouched. A run with `overrides` (a test's) measures and writes nothing.
- **`--only a,b`** takes a comma list of items: `m_ch_sb`, `m_tov_sb`, `ns_sigma_max_sb` (items 1–2, the sheet on the CPU, no
  adapter — a second or less), `m_ign_sb` (item 3), `m_up_sb`, `m_bh_sb` (item 4), and the six item-6 names `preset_mass_sb`,
  `t_eff_sun_sb`, `life_sun_sb`, `top_rung`, `stage_durations_sb`, `max_gas_speed` — **one job**: the three preset lives
  measure all six. `render_reserve_ms` is refused with exit 4: the drawing lot's `--measure-ui` writes it (a visible window,
  the lead's each time, R4) and `calibrate` keeps it across a re-run. A bare `calibrate` runs every item.
- **`--max-steps N`** raises the step ceiling of each life (a run that reaches no ending within it is exit 6, never a guess).
  **`--bracket LO,HI`** replaces the default bracket of one bisected item (items 3–4 only, so `--only` names exactly one of
  them); the defaults and what a bracket whose ends agree means are in `calibrate.rs` and §2.11 — a bracket whose ends agree
  is refused after two runs, widen it.
- **`--resume DIR`** (M0-D72) picks an item up where it stopped. DIR is an earlier invocation's
  `build/calibrate_runs/<stamp>/` (the newest: `ls -t build/calibrate_runs | head -1`); run again the same command — the same
  `--only`, `--bracket`, `--adapter`, `--out` — with `--resume DIR` added. Run NN reads `DIR/<item>-<NN>/summary.json` instead of
  running a life when it parses, its `until.met` is true and its `scene` names this run's item and mass (to 3 decimals); its run
  line says `replayed`. The first run with no finished record continues its life from its last checkpoint when there is one and it
  matches (`--checkpoint-every`, below; M0-D77), else runs fresh in DIR, and every run after it runs fresh (a life cut part-way
  loses at most one save interval). A record of another item or mass, an adapter `--adapter` does not match, or a DIR without this item's
  first life is exit 4 (`DIR is not this calibration's: start without --resume`) before any life runs, a DIR that is not a folder
  too; items 1–2 run no life and ignore it. Without it nothing changes.
- **When a life stops** (M0-D75). Every life runs until its condition holds. **m_up_sb**'s lives stop at the first frame where the `core_collapse` event fires or an
  object shows a `neutron_star` or a `black_hole` — their side is heavy from there, whatever they would have become — else at a white dwarf's
  ending: `until.condition` `collapse|ending` in summary.json, `until.met` true, often no `ending` event. **m_ign_sb** stops at the
  ignition or the ending; **m_bh_sb** and the item-6 presets run to their ending (a neutron star shown can still become a black hole). A record
  made before M0-D75 (run to its ending) replays to the same side. Cost: `max_gas_speed` no longer sees an m_up life's gas after its collapse.
- **The ending and ignition** (M0-D78, §0.5 [M0-D78]). A remnant stage is the life's ending once shown for **max(10 t.u., 20 τ_dyn)**, τ_dyn
  the object's at that frame (the `ending` event's `hold_t`): a compact remnant keeps 10 t.u., an extended one (a white dwarf shown while
  the star is still a huge cloud) waits out its dynamics. **Ignition** is the light window (0.5 L ≤ L_H ≤ 2 L) **or** a centre that has
  burned a tenth of its hydrogen (X_H,c ≤ 0.9 × the most it showed, L_H > 0) — the `ignition` event's `by`, `light` or `fuel` — so a cloud
  whose core flashes its hydrogen in one go is ignited and its burnt core is no failed star. A checkpoint of M0-D77's binary (snapshot
  version 1) is not used (`SR-WARN checkpoint not used (… is version 1, this build reads 2)`, step 0); a record of the old rules replays
  as written — move a life's folder aside to run it again (a missing folder runs fresh).
- **`--progress-every N`** (M0-D75; steps, a whole number ≥ 1, default 10,000; 0, a negative or a non-number is exit 4) is the tick of every
  life's progress line and `progress.json` for the invocation; items 1–2 ignore it, and nothing a life computes changes. A 14 M₀ star runs ~10
  steps/s, so the default is a line every ~16 minutes: give it fewer steps for heavy stars (`--progress-every 1000`, the lead's first try) and
  more for light ones (a 1 M₀ life runs ~280 steps/s: 10,000 is a line every ~35 s, 50,000 every ~3 min). `calibrate_status.py` calls a
  `progress.json` stale only past max(15 min, 3 × the tick's expected interval).
- **One step at a time while it is hard** (M0-D80, §0.5 [M0-D80]). A life's next frame is **one step** while the frame just run had a
  step at P5's ceiling (the heat solve hit its 64 passes or 4,096 rounds) or the heaviest object's raw stage is `core_collapse` — the
  physics, never the clock, so a life frames the same whatever else loads the GPU — and **each one-step frame prints a progress line**
  besides the `--progress-every` tick. A stretch reads `step=1400`, `1401`, `1402` … each with `frame=1`; when the trigger clears, the life
  goes back to frames (16 steps while it watches its `until`) and to one line every N steps. A fast life prints no extra line. A checkpoint
  carries whether the next frame is one step (the snapshot's `one_step`), so `--resume` frames as the unbroken life would have. Plain
  `headless` and the game frame as before.
- **The line's numbers** (M0-D80; sandbox units, §2.1; `progress.json` carries the same keys). The progress line ends `stage=<shown> |
  raw=<the heaviest object's stage before the 3-frame hold>`, then six indented rows, the same numbers in the same columns every step:
  ```
  SR-CALIBRATE progress item=m_bh_sb mass=6702.064 step=1010 sim_time=10.469 heat_unconverged=6 stage=white_dwarf | raw=white_dwarf
    frame  steps          16 dt       8.714e-3 s/step    0.05189 P5_ceil         0
    star   M            6690 R           108.4 tau_dyn     13.80 M_unb       12.78
    core   sigma_c     33.76 T_c         33.48 d_c        0.8549 Ye_c       0.4643
    centre X_Fe        1.000 X_n             0 v_r        0.2974 vr/vdyn   0.03786
    light  L           42033 L_nuc   1.802e-11 L_N             0 L_nu            0
    world  escaped   0.09486 objects         1 sinks           0 sink_M          0
  ```
  - `frame` — steps in the frame (1 in a one-step stretch), `dt` its mean Δt (t.u.), `s/step` wall seconds per step (display only),
    `P5_ceil` its steps at P5's ceiling (the running total is `heat_unconverged`);
  - `star` (the heaviest object, §1.9.2) — `M` mass, `R` radius holding 90 % of M (cells), `tau_dyn` √(R³/GM) (t.u.), `M_unb` unbound mass;
  - `core` — `sigma_c` and `T_c` (Σ and T at its centre), `d_c` degeneracy Π_cold/Π (≥ 0.5 a degenerate core), `Ye_c` electrons per nucleon
    Σ X Z/A (iron 0.464, the neutron_star row ≤ 0.35);
  - `centre` — `X_Fe`, `X_n` (iron's and neutrons' mass fractions there), `v_r` mass-weighted radial velocity (< 0 falling in), `vr/vdyn`
    v_r/√(GM/R) (−1 is free fall);
  - `light` — `L`, `L_nuc`, `L_N` (< 0 while iron neutronizes: the core_collapse row), `L_nu` (powers, energy per t.u.);
  - `world` — `escaped` (mass gone through the edge so far, the ledger's), `objects` (how many the tracker found), `sinks` and `sink_M`
    (black holes in the star and their mass).
  Numbers show 4 significant digits, very large or small ones as `1.802e-11`. **Colour** on a terminal: the stage green, the raw stage
  magenta (red while `core_collapse`), a one-step frame's `steps` yellow, `P5_ceil` above 0 and `L_N` below 0 red; a log kept with `| tee`
  or a redirect stays plain, and `NO_COLOR=1` turns it off. `calibrate_status.py --log` reads the stage before ` | `.
- **`--checkpoint-every M`** (M0-D77; steps, a whole number ≥ 1, default 10,000; 0, a negative or a non-number is exit 4; items 1–2 ignore
  it) is the save interval of a running life, **apart from `--progress-every`**: either may be larger than the other and neither moves anything
  a life computes. At the first frame boundary at or past each multiple of M the life writes its checkpoint — a dump and the observers' snapshot
  (M0-D76; ~15 MB) — into `<life>/checkpoint/`, replacing the last pair (a reader finds a whole pair or the previous one); the folder goes when
  the life finishes and stays when it fails or is cut (Ctrl+C, a logout, `--max-steps`). **A stop loses at most one save interval, not the life:**
  run the same command again with `--resume DIR` and the life continues from its checkpoint — `SR-CALIBRATE continue item=… from step <k>`, its run
  line says `continued from step <k>`, its record is the unbroken life's and its `wall_s` the segments' sum. The checkpoint is used only when it
  is this life's: its scene label and mass, the adapter `--adapter` names, the physics_hash, the world, a snapshot of the dump's own step and a
  dump that reads; any other is not used — `SR-WARN checkpoint not used (<why>)` and the life runs from step 0. **Sizing the two flags to a life's
  pace:** a checkpoint costs one dump (~15 MB, a fraction of a second) and a stop loses up to M steps, so choose M as the wall time you would
  accept losing times the life's steps/s — a 14 M₀ life at ~10 steps/s with M = 3,000 loses at most ~5 min, a 1 M₀ life at ~280 steps/s
  with M = 50,000 ~3 min; N (`--progress-every`) is only how often you hear from it, and may be smaller (the status script shows both
  `checkpoint_step` and what a stop would lose). A life whose stop came before its first checkpoint has none: it runs from step 0.
- **What a run prints.** One `SR-CALIBRATE run …` line per life (mass, side, ending, steps, wall time, `record=<dir>`) so a
  crash or a kill leaves a trace, and while a life runs a `SR-CALIBRATE progress item= mass= step= sim_time= heat_unconverged= stage= | …` line every
  `--progress-every` steps, 10,000 by default, and after each one-step frame (M0-D80, above) (the same fields, the stages shown so far and `progress_every` land in the life's `progress.json`, M0-D74, M0-D75; a life's `SR-WARN heat
  unconverged` lines are counted into `heat_unconverged`, not printed — they buried everything else). Each life's summary.json and events.jsonl are kept in `build/calibrate_runs/<UTC stamp>/<item>-<NN>/`
  (git-ignored; the final dump goes; a failed life's record stays and its error names it) — read them to trace a value to its life. The bisection stops at 2 % (§2.11). `measured.adapter` in the file names the adapter the lives ran on
  (`cpu` while only items 1–2 are in it). Item 5 (K9, `check_order`) runs **before** the preset lives, so a broken order
  costs seconds, not hours. Exit codes: §3.3 — 4 bad arguments, 6 a failed check or a life that proves nothing.
- **Where a running item stands** (M0-D73, M0-D74). From a second terminal, beside the live run: `python3 tools/calibrate_status.py [DIR]
  [--item <label>] [--bracket LO,HI] [--log FILE]` (DIR defaults to the newest `build/calibrate_runs/<stamp>/`). **Read-only** — it reads the
  lives' `summary.json`, the running life's `progress.json` and the folders' times, writes nothing, opens no GPU — and prints ≤ 8 lines
  (then the verdict): the finished lives (mass, ending, steps, wall time); the life running — its stage, step and sim_time «from
  progress.json», how old that file is (marked stale past 15 minutes or 3 ticks at the life's pace, whichever is longer: a stopped item) and how long the life has run, from the file's
  `started` (a life re-run by `--resume` keeps its old folder's time, so the folder is not asked); the stage trail
  (`cloud › collapsing › [main_sequence] since step 30,000`, the heat-unconverged count beside it); the life's progress bar, its pace in
  steps/s and the time left, sized by the finished life nearest in mass (a guess: lives differ; no ETA once it runs past that life); lives
  left and an item ETA as a range and a mean, built on the running life's own ETA. `--log FILE` (the run's output kept with `| tee FILE`)
  still wins for the stage when it holds a progress line of the running life. A life continued from a checkpoint (M0-D77) is paced on this process's own steps (those past `continued_from_step`), goes stale on its own tick, and its life line says `checkpoint at step K: a stop now loses N steps` (or `no checkpoint yet`). The life count of a bisected item is exact (`2 +` one per
  halving of the bracket to 2 %: life 01 is LO, life 02 is HI; before life 02 ends it uses `--bracket`, else the item's default, and says
  so); the ETA is marked a guess until 3 lives have finished. A progress.json that is missing, torn or another item's is not read: the
  line says the folder's age instead.
- **Who runs it.** Items 1–2 are the agents' (CPU, seconds). `m_ign_sb` is ~2 minutes on the RTX 5090 (measured there before R18; 10 lives; 126 s with M0-D78's
  hold where it was ~30 s at 10 t.u.); `m_up_sb`, `m_bh_sb` and the
  preset lives are **whole stellar lives** — hours at the defaults — so the whole calibration is **the lead's**, in one visible
  terminal, in M0-V9's window (§6.4; PLAYBOOK §4 Rule 4). Item by item (`--only`) is how it splits. An agent runs no life
  here; the scopes `calibrate_cold`, `calibrate_bisect`, `calibrate_order`, `cold_core` and `calibration` (`testing.md`) grade
  the code on stubs and the CPU.
- **Which adapter.** Physics numbers are measured on the adapter named in the file — the shipped calibration is the RTX 4090's on laserax-ai-server
  (`measured.adapter`, 2026-10-26; R18 makes that box the GPU's for a block run on laserax-ai, §6.4's 5090 predating it);
  they are used on every other adapter by leaning on G-REF's cross-adapter tolerance (§6.7), not on identity. Timing is
  never part of it.
- **The hash rule: change a constant, calibrate again.** The file carries a `physics_hash` of the canonical JSON of
  elements.json + reactions.json + physics.json (whitespace and key order do not move it). A binary whose embedded
  calibration holds another hash refuses a preset scene with `SR-ERROR calibration out of date`, exit 6 (G-CAL); a headless,
  test or calibrate run whose scene names no preset runs without one. Editing any constant — `sigma_bh` moved it at
  M0-T66b — means re-running `calibrate`, and `PINNED_HASH` in `crates/sr-engine/tests/gpu/calibration.rs` re-pinned until
  the file is embedded (the `calibration` scope's message names the new hash).
- **Embedding.** The next build after `calibrate` writes `assets/calibration.json` embeds it — no hand edit (M0-D62):
  `crates/sr-physics/build.rs` sets `calibration::EMBEDDED` to the file whenever it exists, `None` without it (a tree
  without the file still builds; exit 6 from the real binary is NOT PROVEN until a file is there). The file is a
  calibration without `render_reserve_ms`: that key stays absent until the drawing lot's `--measure-ui` writes it, and the
  time control uses §1.8.3's 3.0 ms meanwhile; a reserve that is there must be finite and ≥ 0.
- **If a run exits 6.** K9's order or band, Σ_BH against ns_sigma_max_sb, c_sb against twice `max_gas_speed`, a preset not ending
  as a white dwarf, neutron star or black hole, or the Sun-like life never reaching X_H,c 0.35 (`t_eff_sun_sb`): each names what
  failed. It is a defect to file or a question to the lead, never a looser number (§0.4), and never re-tuned inside the run (§2.11
  item 5).
