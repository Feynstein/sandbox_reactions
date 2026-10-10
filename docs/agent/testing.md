# Testing — the scopes, how to add one, what each costs

Written by M0-TH (2026-10-08); each lot that adds a scope adds its row here. The routine itself is
`m0_rules.md` R9 (PLAYBOOK §4 Rule 4). Contract surfaces and their guarantees: `milestones/m0/m0_contrat.md` §5.
One runner: `tools/pb/verify.py` over `tools/pb/verify.json`. Never a second runner, never a one-off
probe that repeats a scope.

## The calls (repo root; every run takes `--task <ID>`)
| Call | Use |
|---|---|
| `python3 tools/pb/verify.py <scope> --task <ID>` | iterate on your scope alone |
| `python3 tools/pb/verify.py <scope> --case <id> --task <ID>` | one case of a scope |
| `python3 tools/pb/verify.py --redarm <scope> --task <ID>` | the clean copy GO, each plant NO-GO |
| `python3 tools/pb/verify.py --changed --base <commit> --task <ID>` | once, last: every scope your changed paths touch |
| `python3 tools/pb/verify.py --all --task <ID>` | the complete loop — phase-closing V and TR blocks only |
| `python3 tools/pb/verify.py list` · `report` | the scopes · loops, redundant loops, slowest scopes |

A run's log opens first at `milestones/m0/logs/<ID>.<scope>.log`; every run appends to
`loop_times.jsonl` there. Verdict lines: one counts line, then `=== GO ===` or `=== NO-GO: <reason> ===`.

## Scopes today (linux-pc, 10 scopes, costs as measured by M0-V1 on 2026-10-09, warm caches)
| Scope | Checks | Cases | Cost | Plants |
|---|---|---|---|---|
| `plan_lint` | the plan's structure (`plan.py lint`) | ≥ 30 (49 now) | 0.1 s | `plan-bad-status` (a command: the plan moves, a patch would not apply) |
| `tools_selftest` | each annex tool's own red-armed selftest — proves the tools, never the game | 10 | 10 s | `tools-selftest-scratch` |
| `toolchain` | the stack builds a one-file program: `desktop` (rustc 1.99), `wasm` (wasm32-unknown-unknown), `trunk` (trunk 0.21.14 + wasm-bindgen 0.2.129) | 3 | 0.3 s warm · ≈ 9 s cold (first trunk run fetches wasm-bindgen-cli into `~/.cache/trunk`) | `toolchain-compiler`, `toolchain-web-target`, `toolchain-trunk` |
| `build` | the game's workspace (R11): `native` — `cargo build --workspace --release --locked` then `cargo clippy ... -- -D warnings`; `wasm` — `cargo check --workspace --target wasm32-unknown-unknown --locked` (M0-T1) | 2 | ≈ 5 s on win-laptop (the empty `sr-physics`; grows with the crates) · ≈ 0.6 s warm · ≈ 57 s `--redarm`, linux-pc, M0-V1 2026-10-09 | `build-wasm-only` |
| `adapter` | the GPU adapter choice (§6.1–§6.3, §6.2.2): the matching rule on a fixed list (case, substring, two backends, no match) and a live headless device per adapter the box has, at `Limits::default()`; the log names every adapter as `SR-ADAPTER …` (M0-T2) | 10 | ≈ 5 s warm on linux-pc (M0-V1; 4 adapters: Quadro, RTX 5090, "Intel(R) Graphics (RPL-S)", llvmpipe) | `adapter-case` |
| `state` | the cell state and one step (§2.2, §1.3.2, §1.3.4): a world-size upload/readback round trip with a distinct value per channel and cell, bit-exact; unnormalised and negative species fractions → one step → ΣX = 1 within the tolerance and the other channels bit-identical; two runs bit-identical (M0-T3). `bash tests/gpu.sh state physics` | 6 | ≈ 3 s on linux-pc (RTX 5090, Vulkan; M0-V1, 2026-10-09) | `state-renorm-skip` |
| `boot` | G-BOOT (§5.4): `bash tests/smoke.sh headless-boot` builds sr-app into the tree's own `build/target`, runs `launch.py smoke headless-boot`, then the same command once more into a kept folder and checks `summary.json` carries every §2.12.2 key with `steps` = 200 and stdout's first/last lines (M0-T4) | 1 | ≈ 2 s warm · ≈ 65 s for `--redarm` (cold build per arm), linux-pc, M0-V1 2026-10-09 | `scene-refuse` |
| `desktop` | G-DESK (§5.4, §3.4): case `window` — `bash tests/smoke.sh window`, the box's route (`game-xvfb` on linux-pc, `game-offscreen` on win-laptop, R15): `/status` reads `ready` (the first presented frame) within the deadline; case `launcher` — the double-click launcher fed an Enter (`printf '\n' \| bash start.sh game-xvfb`; win-laptop `start.bat game-offscreen`) starts, reaches ready, stops and leaves `launch.py status` at `running: 0` (M0-T5, M0-T6) | 2 | ≈ 6 s for the scope's two cases warm · `--redarm` ≈ 277 s (the `status-never-ready` arm waits out the 90 s deadline), linux-pc, M0-V1 2026-10-09; win-laptop route owed there | `status-never-ready`, `launcher-no-stop` |
| `web` | G-WEB's build (§3.5, §6.2.1, §6.2.4): case `build` — `trunk build --release` into `web/dist`: index.html, the `.wasm` and the glue exist; the canvas, the `web.*` texts and the inline WebGPU check stand ahead of any use of the wasm; trunk's own auto-loader is gone (M0-T7); cases `ready` and `no-webgpu` — `web/smoke.mjs` (playwright-core 1.64.0, the box's installed Chrome, headless, §6.2.3's flags) against launch.json's `web` service (the static server on 47812 over `web/dist`): `/` reaches srState "ready" within 30 s, `SR-ADAPTER` and `SR-CHROME-ADAPTER` (Chrome's own answer; a software adapter is a NOTE) in the log; `/?no-webgpu=1` shows srState "no-webgpu" and the three `web.no_webgpu_*` texts exactly, no wasm fetched (M0-T15). Both launch Chrome with `chromiumSandbox: true` (playwright-core appends `--no-sandbox` otherwise) and read its command line from chrome://version: `sandbox: on` in the log, or NO-GO naming `--no-sandbox` (§6.2.3, M0-D17). The three cases share one lock (the port) and a dist built fresh against its inputs (`build/web-built.stamp`); the preset case waits for M0-T94 | 3 | ≈ 73 s wall on win-laptop (M0-T15, 2026-10-10: the build case rebuilds, the other two reuse a fresh dist) · `--redarm` ≈ 515 s raw, ≈ 480 s load-normalised (four plants; M0-D17, 2026-10-10), win-laptop; linux-pc owed | `web-no-gpu-check`, `web-autoload`, `never-ready`, `smoke-no-sandbox` |
| `capture` | the scripted captures (§3.3): case `smoke` — `tests/capture/smoke.json` (paused, then running) on the box's window route → two PNGs of the window's size whose pixels differ, and a valid `captions.json` (review_page.py's shape); case `refuse` — an action not built yet, an unknown action and a bad id each → exit 4 with an `SR-ERROR` line, before any window opens (M0-T8) | 2 | ≈ 1–2 s warm · ≈ 52 s `--redarm`, linux-pc, M0-V1 2026-10-09; win-laptop route owed there | `capture-no-captions` |

## Scopes added by lot 2 (M0-T10 – M0-T16; win-laptop `Laser2025-20`, costs from the tasks' own runs 2026-10-10; none run on linux-pc yet — `owed on laserax-ai`, R15)
| Scope | Checks | Cases | Cost | Plants |
|---|---|---|---|---|
| `registry` | the element, constant and reaction registries (§1.5, §1.6, §2.5.1, §2.7): `bash tests/cpu.sh sr-physics registry`. Shipped files parse and equal §1.5's table; physics.json's every bound (ignition ladder, K7 floors, m < ν, Σ_N, m_tov/m_ch, f_dep cap, K pairs on or off) each refused naming its key; reactions.json's ten records resolve, shares sum to 1 on both sides, every refusal names `records[<id>].<field>`; stand-ins ship off (M0-T10, M0-T11) | 29 | ≈ 2–15 s (warm / cold), `--redarm` ≈ 25 s | `registry-order-unchecked`, `registry-shares-unchecked` |
| `eos` | the equation of state (§2.3.1–§2.3.3): `bash tests/cpu.sh sr-physics eos`. P's limits; the table's nodes against the closed-form integral; log-log interpolation between nodes inside the contract's error budget (worst measured recorded in M0-T12's detail file); extrapolation either side; Π, c², T and the floor's `floor_added`; a half-on K pair refused (M0-T12). The GPU side reads the same table (M0-T19) | 13 | ≈ 3 s, `--redarm` ≈ 12 s | `eos-exponent` |
| `strings` | G-STR (§4.1): `bash tests/cpu.sh sr-app --test strings` — sr-app is bin-only, so the test includes `strings.rs` by `#[path]`. Cases `same_keys_both_ways` (also refuses a block read as < 100 rows), `texts_byte_identical`, `web_texts_verbatim_in_index_html`; the contract's `strings` block is read at run time, so editing §4.1 moves the scope (M0-T13) | 3 | ≈ 5 s warm, `--redarm` ≈ 170 s (cold build) | `string-typo` |
| `licences` | G-LIC (§5.4, R7): `python3 tests/licences/check.py <case>` — `cargo-native` and `cargo-wasm32` judge the packages in `cargo metadata --locked --filter-platform`'s resolve graph (not the whole lockfile), `npm` the licences in `web/node_modules/**/package.json` (`npm ci` first when absent). Expression grammar: ids, parentheses, AND, OR, `/` as OR, WITH only for Apache-2.0 WITH LLVM-exception. One named exception, by exact (package, expression): `epaint_default_fonts` (contract §0.5 [M0-T14]). The `npm` case is graded on the web tree as it stands (M0-T14, M0-T15) | 3 | ≈ 2 s (cargo) · ≈ 45 s with `npm ci`; `--redarm` ≈ 6 s | `gpl-dep` |
| `watch_only` | G-WATCH's static scan (§5.4, I2): `python3 tests/watch/check.py` — no whole identifier `observe`, `Stage`, `Tracker` or calibration key (read from §2.11's `**Schema:**` line at run time, less `version`, `measured`, `physics_hash`) in `crates/sr-engine/src/step/` or `crates/sr-engine/shaders/`, comments and strings included. Fewer than 2 files scanned or 10 keys read is NO-GO. **Named `watch_only`, not the contract's `watch-only`:** verify.py scope names are `[a-z0-9_]` (M0-T16) | 1 | < 1 s, `--redarm` ≈ 4 s | `stage-in-step` |

The `web` row above also holds the cases M0-T15 added (`ready`, `no-webgpu`); the `ready` case is smoke only — it checks srState and logs both adapters, no pixels —
and the preset case waits for M0-T94.

The complete loop (M0-V1, `verify.py --all`, linux-pc `laserax-ai`, 2026-10-09): 537 cases in 6.1 s (`jobs` 8), 449.7 s wall in all with
the six scopes new since HEAD red-armed first (`desktop` alone 287 s); median loop 3.3 s (`verify.py report`). Budget: `m0_implementation_plan.md` Rules. `tools_selftest` GO is the tools' GO, never the
game's: the game's scopes arrive with the lots that build their passes (§5 names 38 — `grav-force` … `web`;
`licences` and `strings` need no physics and can be armed as soon as code exists).

## Adding a scope — the block that builds a contract surface does this
1. **The artifact first** (a pass, a command, a check), then its entry in `tools/pb/verify.json`:
   `{"cmd" | "cases" + "case_cmd" | "cases_cmd", "parse", "expected", "paths", "timeout_s", "plants"}`.
   `expected` is the number of cases the contract row promises — fewer ran is NO-GO, zero is NO-GO. `paths`
   are the globs the scope grades (`**` crosses folders, `*` does not); they drive `--changed` and the
   coverage anchor.
2. **The count parser — this stack.**
   - A binary that reports `=== GO ===` / `=== NO-GO: … ===` (the game's own commands: `headless`, `bench`,
     `calibrate`, a check subcommand): the default `gonogo` parse; `count` names a regex with an `n` group
     when one command is many cases.
   - `cargo test`: **one scope per test binary** — `cargo test -p <crate> --lib` or `--test <file>` — with
     `"parse": "regex", "regex": "test result: \\w+\\. (?P<passed>\\d+) passed; (?P<failed>\\d+) failed; (?P<skipped>\\d+) ignored"`.
     Measured 2026-10-08: cargo prints one `test result:` line per test binary, and `regex` reads only the last
     match, so a whole-workspace `cargo test` would count one binary (and stop at the first failing one).
   - Many cases from one command, run in parallel: `cases` (list or id → command) with `case_cmd` holding
     `{case}`; a case id has no space or shell character.
3. **The planted bug.** Each scope names ≥ 1 plant: `tests/plants/<plant>.patch`, a unified diff applied only
   in a scratch copy by `--redarm`, which must turn the scope NO-GO. A bug planted in a file that moves (the
   plan) is a command instead: `{"id": …, "patch": "{python} -S -c \"…\""}`. A plant that stays GO is an unfailable
   scope — fix the scope, never the plant. Physics rows: R8 — an oracle with a cited source and a tolerance
   fixed before the first run (§5.0); never a looser number.
4. **Paths classified.** Every file git can see is covered by a scope's `paths` or named by a `skip_classes`
   entry, else `--changed` is NO-GO (`UNCLASSIFIED`). A new folder → its scope's `paths`, or a skip class with a
   reason. Today's skip classes: the harness's manifest and anchor · instruction and config files · reports, tasks,
   the contract and docs.
5. **Boxes.** A scope only linux-pc can run declares `"box": "<hostname>"` (linux-pc is `laserax-ai`) and reads
   `owed on <box>` elsewhere, never NO-GO.
6. **Add the row above** and the contract section the scope grades.

## What every GPU or timing scope names
The adapter it ran on (`SR-ADAPTER …`): linux-pc has three GPUs and its display is on the Quadro RTX 4000
(Hazards). Physics-only scopes may use `--adapter "RTX 5090"`; timing scopes (G-FPS, G-TOP) never do. Results
across adapters agree within G-REF's tolerances only (contract §6.7).

## Hazards measured by M0-TH
- **Cargo is off the PATH** (`~/.cargo/bin`): scope commands and scripts put it first themselves
  (`tests/toolchain/check.sh` shows how) or call `~/.cargo/bin/cargo`.
- **Scratch copies carry a repo-root `target/` and `.cache/`.** `verify.py --redarm` copies the tree leaving out by
  name only `.git .venv venv node_modules models data var output __pycache__ .pytest_cache dist build` (annex
  §A.2/§A.7). `target/` and `.cache/` are not on that list, so each red-arm arm would copy them — the disk is
  82 % full. Settled by contract §6.5 as amended [M0-TG] (the lead approved, 2026-10-08): cargo's target is
  `build/target/` (`.cargo/config.toml`), left out of copies by name; red-arm copies may share
  `~/.cache/sandbox-reactions/redarm-target/` outside the tree for the dependencies' artifacts — never for a
  binary a scope launches. The mechanism is below (§ Cargo scopes).
- **A scope's script must create its own `build/`**: a scratch copy has none (the toolchain scope's first red-arm
  failed on exactly this).
- **`rustup`-installed targets**: the `wasm` case needs `wasm32-unknown-unknown` (`rustup target add`, R3) and the
  `trunk` case needs `trunk` 0.21.14 on `~/.cargo/bin` — a box without them reads NO-GO there, naming the missing tool.
- **`capture_web.mjs`'s live tier** needs Playwright: `web/package.json` (M0-T15) pins playwright-core 1.64.0; `npm ci` in `web/` installs it (the web smoke's case and the licences `npm` case do it themselves when `node_modules` is absent).

## Hazards measured on win-laptop (M0-TE-win, M0-D5)
- **Run the toolkit from Git Bash with `MSYS=winsymlinks:nativestrict`** (a user variable there; Developer Mode
  on), never from PowerShell, whose `bash` is WSL's relay. A Git Bash **without** the setting copies instead of
  linking: `scratch_copy.sh --selftest` reads NO-GO 9/12 (the leak, symlink and `models` plant checks) and
  `tools_selftest[scratch_copy]` is red — not a defect, the shell; set the variable and rerun.
- **8.3 short names** (`TMPDIR=/c/Users/YOHANB~1/…`): `pwd -P` expands them and normalises case and `/tmp`,
  `realpath -m` does not — two paths are compared only after one resolution (scratch_copy.sh's `canon`, M0-D5).

## Cargo scopes — `tests/cargo.sh` (M0-T1)
Every cargo scope's script sources `tests/cargo.sh`; it is also the `build` scope's case runner (`bash tests/cargo.sh build native|wasm`).
- It puts `~/.cargo/bin` first on the PATH (linux-pc has it off; harmless on win-laptop).
- Real tree: `CARGO_TARGET_DIR` is unset, so cargo builds into `build/target/` (`.cargo/config.toml`). In a red-arm scratch
  copy (no `.git`) it is `~/.cache/sandbox-reactions/redarm-target/`, shared by both arms so a plant rebuilds only the workspace's
  crates — for test builds only, never a binary a scope launches (§6.5). The cache is safe to delete.
- In a copy, `cargo` is a wrapper (M0-D9): cargo's freshness is mtime-based and the copy keeps each file's mtime, so a build
  another arm left in the shared target looked fresh (a second `--redarm` ran the plant's binary in its clean arm). Each call
  takes the target's lock (`redarm-target.lock`; `flock`, else a `.lock.d` mkdir lock broken when its holder is gone),
  touches `crates/ assets/ scenes/ Cargo.toml Cargo.lock .cargo/`, then runs cargo inside the lock: every arm rebuilds the
  workspace's crates (≈ 1–3 s), never the dependencies, and arms building in parallel are serialised. A scope calls `cargo`
  after sourcing `tests/cargo.sh`, never `command cargo` or a path to it, or it bypasses the wrapper. A tool that runs the
  cargo binary itself (trunk) runs as `sr_in_shared_target <tool> ...` (the same lock and touch; a plain call in the real
  tree) — bare, trunk built from the plant's artifacts in a clean arm (M0-T15). An input cargo reads from
  outside those paths (an `include_str!` elsewhere) goes on the touch list.
- Every build, check and test is `--locked`: `Cargo.lock` is committed, and adding a dependency means regenerating it.
- One build profile: `[profile.release]` in the root `Cargo.toml`; `[profile.test]` inherits it (the CPU twin needs optimised code).
- `rust-toolchain.toml` pins 1.99.0 (+ clippy, rustfmt, targets linux-gnu and wasm32); the first cargo call in a tree auto-installs that
  toolchain user-level (win-laptop: 2026-10-08, ≈ 25 s, beside `stable`).
- A GPU scope is `bash tests/gpu.sh <module>` (M0-T2): `cargo test` of sr-engine's one GPU binary, `<module>::`, `--nocapture --test-threads=1`, so the log names the adapters; a GPU test gets its device from `device()` in `crates/sr-engine/tests/gpu/main.rs` (adapter from `SR_TEST_ADAPTER`, else the high-performance pick — on linux-pc that is the Quadro RTX 4000, never rely on it).
- `bash tests/gpu.sh <module> [physics]`: the optional `physics` argument points the module at the box's physics adapter (R12) — `SR_TEST_ADAPTER` becomes `RTX 5090` on linux-pc, `RTX 4080` under Git Bash — unless the caller already set `SR_TEST_ADAPTER` (M0-T3). The `state` scope uses it; `adapter` does not (it walks every adapter the box has).
- A window or boot scope is `bash tests/smoke.sh <service|window|launcher>` (M0-T4–M0-T6): it builds sr-app into **the tree's own `build/target`**, never the shared `redarm-target` (two arms must not launch one binary, §6.5), takes a lock on launch.json's port so the desktop scope's parallel cases boot one at a time, and always stops what it started. `tools/pb/launch.json` holds the services (`headless-boot`, `game`, `game-xvfb`, `game-offscreen`, `web`, §3.6); the binary's flags and the routes are in `docs/agent/running.md`.
- `bash tests/capture/check.sh <smoke|refuse>` (M0-T8) and `bash tests/web/check.sh build` (M0-T7) follow the same pattern: build or `trunk build` in the tree's own `build/`, check, end in `=== GO ===` / `=== NO-GO: … ===`. The capture's windowed arm runs under `xvfb-run … env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE` on linux-pc (winit prefers Wayland whenever it is set, so without the `env -u` the window opens on the lead's desktop) and with `--offscreen-window` on win-laptop.
- A new member crate lands inside `crates/*` and is covered by the `build` scope's `crates/**` path; a cargo test scope is
  `cargo test --release --locked -p <crate> --lib` (R11) and sources `tests/cargo.sh` for the environment.

## Boxes — two hostnames, one rule (R5, R15; contract §6.1, §6.6)
Both boxes are in play for every M0 block, lot V's included; a block runs on the box the lead is on, after probing it,
and every verdict names its box.
| | linux-pc | win-laptop |
|---|---|---|
| hostname (`box:` in verify.json) | `laserax-ai` | `Laser2025-20` |
| shell for the toolkit | bash | Git Bash through the Bash tool only (`py -3.12 tools/pb/…`, never `python3`; `MSYS=winsymlinks:nativestrict`) |
| physics adapter (R12) | RTX 5090 | RTX 4080 Laptop |
| window checks (G-DESK, captures) | `game-xvfb` / `xvfb-run` on a private Xvfb with lavapipe | `game-offscreen` — `--offscreen-window`, never focused, out of the taskbar |
| launcher | `start.sh` | `start.bat` (and `start.ps1`) |
| web smoke's Chrome | `/usr/bin/google-chrome` | `C:\Program Files\Google\Chrome\Application\chrome.exe` |
| timing (G-FPS, G-TOP, per-pass costs, `--measure-ui`) | the Quadro RTX 4000 — `box: laserax-ai` | never here: reads `owed on laserax-ai` |
A check only one box can run reads `NOT RUN — owed on <other box>`, flagged V to V and run at the next V on its box (R15).
A scope's `box` key is the hostname, never `linux-pc`. Details: `docs/agent/running.md`.

## Long runs
`calibrate`, the ⏱ scopes and benches at low rungs can pass 10 minutes on the Quadro: the lead's, in one
visible terminal, placed in V windows (contract §6.4). A scope the lead runs sets its own `timeout_s`.
