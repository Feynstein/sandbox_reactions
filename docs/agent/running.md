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

## The binary `sandbox-reactions` (§3.3)
`sandbox-reactions --help` prints the usage line. What exists today (M0-T4–M0-T8); the rest of §3.3's flags arrive with the
block that builds their feature and are refused with exit 4 until then:
- **Desktop** (no subcommand): `[--adapter S] [--status-port P] [--offscreen-window] [--capture SCRIPT --out DIR]`.
  - `--status-port` serves `GET /status` on 127.0.0.1 only (§3.4); off without the flag, never in the web build.
  - `--offscreen-window` places the window outside every monitor, never activated, out of the taskbar (§3.3 [M0-TJ3]).
  - `--capture` runs a scripted session, one PNG per step plus `captions.json`, then exits 0 (§3.3; the harness is
    `tests/capture/`). An action not built yet is refused with `SR-ERROR`, exit 4.
- **Headless**: `headless --scene <scenes/FILE.json | preset:<sun|massive|giant>> --steps N [--world WxH]
  [--edge leave|bounce] [--max-steps N] [--out DIR] [--timing] [--dump-every N] [--load-dump FILE]
  [--adapter S | --cpu-reference]` — the
  same step with no surface and no display (§6.3); `DIR/summary.json` per §2.12.2 and, with `--out`, the final state dump
  `DIR/dumps/s_<step>.bin` (§2.12.3; `--dump-every` adds one at every step index N divides, counted from the scene;
  `--load-dump` starts from a dump, M0-T26). `--world` and `--edge` stand over the scene's `world` (M0-T25); `--timing`
  adds `timing` to the summary (M0-T23). The scene's `switches` (§2.12.1, each on when left out) build the step's passes:
  `gravity` P2, `hydro` P4, `heat` P5, `reactions` P6, `sinks` P3 and `neutrinos` P7 — P7 only with `reactions` on, S_ν being
  P6's (M0-D47); `--timing` lists the passes built. `--cpu-reference` (M0-T37) runs the CPU f64 twin from the same scene instead of
  the GPU — worlds up to 128 × 128 (exit 4 above), the same summary.json and dumps (the box record the whole world's);
  `--adapter`, `--timing` and `--load-dump` are the GPU's and refused with it. stdout: first line `SR-ADAPTER …`, last `SR-HEADLESS DONE …`; errors `SR-ERROR`,
  warnings `SR-WARN`. Exit codes: §3.3. The file formats: `docs/agent/architecture.md` § The files. sr-app's usage line
  (`--help`) lists these flags (M0-T37).
- **`calibrate`** (M0-T66b–M0-T70): `calibrate --out FILE [--adapter S] [--only a,b] [--max-steps N] [--bracket LO,HI]` —
  § Calibrating below. (sr-app's usage line lists `--only` only; the other two flags are refused with exit 4 where they
  do not apply.)
- Later blocks add `bench`, `--measure-ui` (a visible window — asked of the lead each time, R4) and the headless run's
  `--until`, `--frames-every`, `--edits` (§3.3).

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
| linux-pc | Quadro RTX 4000 (drives the display) · RTX 5090 · "Intel(R) Graphics (RPL-S)" (the Intel iGPU — match it with `--adapter "RPL-S"`; `"UHD"` finds nothing) · llvmpipe (Mesa software Vulkan) | physics scopes `--adapter "RTX 5090"` (`SR_TEST_ADAPTER`, `tests/gpu.sh … physics`); timing scopes the Quadro only (`box: laserax-ai`); private-display windows `--adapter llvmpipe` |
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

## Long runs (§6.4)
`calibrate`, the ⏱ scopes and benches at low rungs can pass 10 minutes on the Quadro: **the lead's**, in one visible
terminal, placed in the V windows. A run over 10 minutes is never an agent's (PLAYBOOK §4 Rule 4).

## Calibrating (§2.11, §3.3, §6.4; `crates/sr-engine/src/calibrate.rs`)
`calibrate` measures the sandbox's own scales — the cold ceilings, the ignition and ending masses, the presets' masses and
lives — and writes `assets/calibration.json`. The file's schema and every bound are §2.11's; its values are never copied
into a page — read the file (`docs/agent/physics.md` § Calibration says what each key is). Commands run from the repo root,
the binary built first (top of this page).
```
build/target/release/sandbox-reactions calibrate --adapter "RTX 5090" --out assets/calibration.json
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
- **What a run prints.** One `SR-CALIBRATE run …` line per life (mass, side, ending, steps, wall time) so a crash or a kill
  leaves a trace; the bisection stops at 2 % (§2.11). `measured.adapter` in the file names the adapter the lives ran on
  (`cpu` while only items 1–2 are in it). Item 5 (K9, `check_order`) runs **before** the preset lives, so a broken order
  costs seconds, not hours. Exit codes: §3.3 — 4 bad arguments, 6 a failed check or a life that proves nothing.
- **Who runs it.** Items 1–2 are the agents' (CPU, seconds). `m_ign_sb` is minutes on the RTX 5090; `m_up_sb`, `m_bh_sb` and the
  preset lives are **whole stellar lives** — hours at the defaults — so the whole calibration is **the lead's**, in one visible
  terminal, in M0-V9's window (§6.4; PLAYBOOK §4 Rule 4). Item by item (`--only`) is how it splits. An agent runs no life
  here; the scopes `calibrate_cold`, `calibrate_bisect`, `calibrate_order`, `cold_core` and `calibration` (`testing.md`) grade
  the code on stubs and the CPU.
- **Which adapter.** Physics numbers are measured on the RTX 5090 (`--adapter "RTX 5090"`, §6.4) and named in the file;
  they are used on every other adapter by leaning on G-REF's cross-adapter tolerance (§6.7), not on identity. Timing is
  never part of it.
- **The hash rule: change a constant, calibrate again.** The file carries a `physics_hash` of the canonical JSON of
  elements.json + reactions.json + physics.json (whitespace and key order do not move it). A binary whose embedded
  calibration holds another hash refuses a preset scene with `SR-ERROR calibration out of date`, exit 6 (G-CAL); a headless,
  test or calibrate run whose scene names no preset runs without one. Editing any constant — `sigma_bh` moved it at
  M0-T66b — means re-running `calibrate`, and `PINNED_HASH` in `crates/sr-engine/tests/gpu/calibration.rs` re-pinned until
  the file is embedded (the `calibration` scope's message names the new hash).
- **Embedding.** After the first write, `EMBEDDED` in `crates/sr-physics/src/calibration.rs` is set to
  `Some(include_str!("../../../assets/calibration.json"))` (M0-V9's step; until then it is `None` and exit 6 from the real
  binary is NOT PROVEN). `Calibration::parse` refuses a file without `render_reserve_ms`, so a calibration is complete only
  after the drawing lot's `--measure-ui`.
- **If a run exits 6.** K9's order or band, Σ_BH against ns_sigma_max_sb, c_sb against twice `max_gas_speed`, a preset not ending
  as a white dwarf, neutron star or black hole, or the Sun-like life never reaching X_H,c 0.35 (`t_eff_sun_sb`): each names what
  failed. It is a defect to file or a question to the lead, never a looser number (§0.4), and never re-tuned inside the run (§2.11
  item 5).
