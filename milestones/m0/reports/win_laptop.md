# win-laptop — the box, what was installed, the toolkit's verdicts, lot 1 mapped (M0-TE-win, 2026-10-08)

Written by M0-TE-win on win-laptop (hostname Laser2025-20), Claude Code 2.1.294 (`AI_AGENT=claude-code_2-1-294_agent`),
the rung read by `rung_record.py now`: model=claude-sonnet-5-5 level=high plugin=installed. Raw trace:
milestones/m0/logs/M0-TE-win.log (+ `.tools_selftest.log`, `.toolchain.log`, `.toolchain.redarm.log`).
M0-TJ3 reads this file to rule what runs where; its answers land in a `## Ruling` section below.

## Box
Probed first, read-only (R5, §4 Rule 5), before any command was phrased.

| Item | Probed value |
|---|---|
| Hostname · user | Laser2025-20 · YohanBelanger (AzureAD) |
| Windows | 11 "Affaires" (Pro), version 10.0, build 26300, 64-bit; Developer Mode on (`AllowDevelopmentWithoutDevLicense` = 1) |
| CPU · RAM | Intel Core Ultra 9 185H, 16 cores / 22 threads · 31.5 GiB |
| Disk | C: holds the repo; **31.3 GiB free of 953 GiB (3 %)** — see Hazards |
| GPUs (Win32_VideoController) | Intel Arc Graphics, driver 32.0.101.6790 (2025-04-27) · NVIDIA GeForce RTX 4080 Laptop GPU, driver 32.0.15.8129 (2025-09-04). `vulkan-1.dll`, `d3d12.dll` and WARP (`d3d10warp.dll`) are present in System32; no Vulkan SDK / `vulkaninfo` |
| Shells | PowerShell 5.1.26100.9444 (Desktop edition) for the PowerShell tool; **Git Bash 5.3.15 (msys, MINGW64_NT-10.0-26300)** is the shell Claude Code's Bash tool runs commands in; `git` 2.55.0.windows.5 |
| Agent | Claude Code 2.1.294, VS Code extension; the switch plugin `pb-switch@pb` 12.2.1 is installed (`installed_plugins.json`) and `rung_record.py now` reads `plugin=installed` |
| Clone | `C:\workspace\sandbox_reactions`, branch main at 3be0d03 "M0 Phase 2: contract, harness and build pipeline"; `git ls-remote origin main` = 3be0d03f77c2… (the lead's pushed Phase 2 gate) — equal; working tree clean at the start |
| Line endings | `core.autocrlf=true` (Git for Windows' system config) but `.gitattributes` (`* text=auto eol=lf`, `*.sh`, `*.py`, `*.mjs`, … `eol=lf`) wins: `git ls-files --eol` shows 79 tracked files `w/lf`, 0 `w/crlf`, every .sh / .py / .mjs `i/lf w/lf` |

## Installed
Table = contract §1.2 / contract_rulings.md §2 (the approved table). R3: user-level, the agent's own setup. Versions as measured.

| Tool | Table | Found before | Now | How |
|---|---|---|---|---|
| Python | 3.12 | `python3` → the Store package's alias, **real Python 3.13.14** (`py -0p`: 3.13 Store, 3.12.10 under `python`) | unchanged — `python3` = 3.13.14 (≥ 3.12 passes Verify 1) | nothing installed; declared, see Hazards |
| Node + npm | Node 20 | Node 26.8.2, npm 11.19.1 | unchanged | playwright-core 1.64.0 needs Node ≥ 20, so 26 satisfies it; **declared default** (not downgraded — R3 forbids it); M0-T15 re-checks when Playwright first runs here |
| Rust | 1.99.0 | rustc/cargo **1.97.1** (rustup 1.29.0, target msvc only) | rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0; clippy, rustfmt; targets `x86_64-pc-windows-msvc` + `wasm32-unknown-unknown`; rustup 1.29.1 | `rustup update stable` (stable == 1.99.0 today), `rustup target add wasm32-unknown-unknown` |
| MSVC C++ build tools | not in the table | Visual Studio 2026 Community 18.7.3 with MSVC 14.44.35207 and 14.51.36231 and Windows SDKs 10.0.22621 / 26100 / 28000 | unchanged | nothing installed — the msvc target links with it already; no ask needed |
| trunk | 0.21.14 | missing | 0.21.14 in `~/.cargo/bin/trunk.exe` | the `trunk-x86_64-pc-windows-msvc.zip` release of github.com/trunk-rs/trunk v0.21.14, sha256 cd6ac15b9daff0365e5695036791ef2ce3c63f61c014f5a8c532363266e4569c = the `.sha256` file's |
| Google Chrome | (155 on linux-pc) | 154.0.8037.98 at `C:\Program Files\Google\Chrome\Application\chrome.exe` | unchanged | present; a test tool only |
| rsync (not in the table) | — | missing | 3.5.1, protocol 33, in `C:\Users\YohanBelanger\bin` | **asked of the lead** (§4 Rule 2; GPL-3.0-or-later, a dev tool never shipped or copied into the game, $0): « Install rsync, user-level (Recommended) ». MSYS2 packages rsync-3.5.1-2, libxxhash-0.8.4-1, popt-1.19-1 from repo.msys2.org, each sha256 equal to msys.db's |
| `MSYS` user variable | — | unset | `winsymlinks:nativestrict` | `scratch_copy.sh` links `.venv` and `node_modules` as symlinks; Git Bash otherwise copies them. Needs Developer Mode (on). New shells and a restarted VS Code see it |

playwright-core 1.64.0 is not installed (it arrives with `web/package.json`, M0-T15; `npm install` in the repo then).
The switch: `rung_record.py now` → `plugin=installed` — no install line owed.
Sizes: the downloads were trunk's zip (7.3 MB) and the three MSYS2 packages (~0.5 MB); the Rust update's size was not measured.

## Toolkit
`python3 tools/pb/verify.py --all --task M0-TE-win` in Git Bash with `MSYS=winsymlinks:nativestrict`
(second run; the first, before rsync and the MSYS variable, read the same four reds with 9 checks red in scratch_copy):

| Scope | win-laptop | linux-pc (plan: Rules, testing.md) |
|---|---|---|
| `plan_lint` | GO, 479 checks (487 at the re-run) | GO |
| `toolchain` | **GO, 3 cases** (desktop, wasm, trunk) — cold 113 s, warm 4.4 s | GO, 3 |
| `toolchain` red-armed | **GO: clean GO, 3 plants / 3 red** (65 s) | same |
| `tools_selftest` | **NO-GO, 10 cases: 6 GO, 4 red** — after M0-D2..D5: **GO, 10/10** (215 s) | 10 cases, GO |
| loop | 75–97 s wall (jobs 8; `tools_selftest[launch]` alone is 97 s) | 3.2 s |

Case counts equal linux-pc's (plan_lint 479, tools_selftest 10, toolchain 3); no case skips — the one SKIP printed inside
`launch.py selftest` («private X display: an X server is POSIX only — NOT RUN on os nt») is that tool's own, inside a GO.

Re-run after M0-D2..D5 (2026-10-08, win-laptop, Git Bash, `--task M0-TE-win-r2` because the first run's logs are committed and the harness never appends to committed evidence): `verify.py --all` → **500 passed, 0 failed, 0 skipped, 3/3 scopes GO**, 97.7 s wall (tools_selftest 10/10, 215 s of case time; toolchain 3/3; plan_lint 487); `--redarm toolchain` → clean GO, 3 plants / 3 red (107 s). Logs: `logs/M0-TE-win-r2.*`.

Red cases of `tools_selftest` at the first run, each reproduced alone and routed to a D (all four since fixed, DONE):

| Case | Checks red | Routed |
|---|---|---|
| `rung_record` | 4 of 62: the `now:` group (the POSIX install line, another project's scope, a stale install, no `$CLAUDE_CODE_EXECPATH`) | M0-D2 |
| `content_gate` | 2 of 37: `--denylist` default, then `PermissionError: [WinError 32]` on a temp file (OPT-D off, tool unused) | M0-D3 |
| `plan` | 1 of 246: the over-500-line `Read:` WARN check gets an extra WARN | M0-D4 |
| `scratch_copy` | 1 of 12: «a dest inside the source → NO-GO, nothing created» | M0-D5 |

Green on this box: `launch` (46 checks), `verify` (166, 98/98 plants red), `review_page`, `status_page`, `capture_web` (85; its live tier NOT RUN — no
Playwright yet), `switch` (29, 8/8 plants red). `tests/toolchain/check.sh` ran on Windows **unchanged**; its three cases and
three plants are untouched and re-proven red here.

`bash` as the toolkit's subprocesses resolve it:
- from the **Bash tool (Git Bash)**: `C:\Program Files\Git\usr\bin\bash.EXE`, GNU bash 5.3.15 — correct;
- from the **PowerShell tool / a PowerShell terminal**: `C:\WINDOWS\system32\bash.exe`, **WSL's** relay (its default distro is docker-desktop, which has no `/bin/bash`) —
  `verify.py toolchain` run from PowerShell read NO-GO 0/3 (« no `=== GO ===` line (exit 1) »). Declared: **every toolkit call on this box goes through the Bash tool**, never PowerShell; no PATH edit made (putting Git's `usr\bin` ahead of System32 would shadow Windows' `find` and `sort` for every other program — the lead's call, not an install).

One intermittent crash, not filed (not reproducible): in the claim run (`verify.py --changed`, 8 jobs) the `verify` selftest case exited
`3221225477` (0xC0000005, an access violation) after 5 s with no output; the same selftest was GO in the two `--all` runs and in
three runs alone (166 checks, 98/98 plants red). Seen once in about eight runs; a D waits for a second sighting.

Status: **Verify 2 is NO-GO** (4 red cases), so M0-TE-win is BLOCKED by its own Fail arm — each red routed above, no check loosened.

## Lot 1 on win-laptop
Read against this box: Intel Arc + RTX 4080 Laptop (Vulkan and DX12 through wgpu), no Xvfb, no Wayland, Git Bash, the Bash tool.
Three classes — **as written** (runs here unchanged), **Windows variant** (named), **linux-pc only**.
M0-TJ3 rules which become box-bound (`box: laserax-ai`, read `owed on laserax-ai`) and which get the variant.

| Block | Class | What changes here |
|---|---|---|
| M0-T1 workspace, `build` scope (`tests/cargo.sh`) | **as written** — NOT PROVEN until it exists | `cargo` is already on the PATH here (`~/.cargo/bin` first is harmless); `rust-toolchain.toml` pins 1.99.0 and rustup fetches it if absent; red-arm's shared `CARGO_TARGET_DIR` under `~/.cache/sandbox-reactions/redarm-target/` is a MSYS path to the user's home — works; **disk: 31 GiB free** (wgpu + eframe `build/target` may take 5–15 GB, plus the shared red-arm cache) |
| M0-T2 `adapter` scope | **Windows variant** | the adapter list is Intel Arc + RTX 4080 Laptop (+ DX12 and WARP backends, no `llvmpipe`/Quadro/RTX 5090); the Verify 1 Pass line names «Quadro RTX 4000, RTX 5090 and llvmpipe» → on this box it is **linux-pc only** as written; a Windows variant names the RTX 4080 Laptop (`--adapter "RTX 4080"`) and the Intel Arc, accepts the DX12 and Vulkan backends, and §6.1's «Vulkan preferred where a name matches on two backends» is the rule under test |
| M0-T3 `state` scope | **as written**, adapter-named | GPU test; runs on any adapter — the log names the adapter (R13); bit-identity is per adapter (§1.3.4, §6.7), so numbers from here never stand for linux-pc's |
| M0-T4 `boot` scope (`headless-boot`, `tests/smoke.sh`) | **as written**, plus `launch.py` on Windows | `launch.py` selftest is GO here (its X-display check is POSIX-only); `tests/smoke.sh` builds in a scratch copy — rsync is installed, MSYS symlinks set; the binary is `sandbox-reactions.exe` (a `.exe` suffix the script must not assume away) |
| M0-T5 `desktop` scope (`game-xvfb`) | **linux-pc only** (Xvfb, lavapipe) | Windows variant = a window check without Xvfb: a visible window (asks, R4) or an offscreen wgpu frame; no private display exists here — a headless/offscreen route is the only R4-compatible one. `game` (visible) is the lead's, on this laptop's GPU |
| M0-T6 launcher `start.sh` + case `launcher` | **Windows variant** | `start.sh` double-clicks only on Linux; a `start.bat` beside it (§8) calls `python3 tools/pb/launch.py start game` (or `py -3`); the case `printf '\n' \| bash start.sh game-xvfb` is **linux-pc only** (game-xvfb) |
| M0-T7 `web` scope (`trunk build --release`) | **as written** | trunk 0.21.14 is installed and the `toolchain` case `trunk` is GO here (it fetched wasm-bindgen-cli 0.2.129 and wasm-opt itself); `web/dist` etc. |
| M0-T8 `capture` scope | **linux-pc only** (`xvfb-run`, llvmpipe) | the Windows variant is eframe's viewport screenshot on a real window (visible → asks) or an offscreen frame |
| M0-T9 docs | **as written** | `grep` checks run in Git Bash |
| M0-TV1 UI/UX pass | **mostly as written** | `review_page.py serve`, `capture_web` work (selftests GO); the no-WebGPU capture uses Chrome's headless `--screenshot` against `C:\Program Files\Google\Chrome\Application\chrome.exe` (contract §6.2.3 hard-codes `/usr/bin/google-chrome`) |
| M0-V1 lot 1 validation | **Windows variant** for the parts above; its `--all` is red here until M0-D2–D5 are fixed or TJ3 box-binds `tools_selftest`; the human-run tier (double-click `start.sh`) becomes the `.bat` on this laptop |

The harness's own scopes: `plan_lint` as written · `toolchain` as written (GO here) · `tools_selftest` 6 GO / 4 red (above).
Scopes lots 1–N will add, by the same patterns (the plan's scopes, `verify.json`'s `box:` field = hostname `laserax-ai` for linux-pc):
- **GPU physics scopes** (`grav-force`, `eos`, `gas`, …, `--test gpu <module>::`, R11–R13): runnable on this box's two adapters as a *smoke*, but their `Pass:` lines name RTX 5090 / Quadro RTX 4000 → linux-pc only for the gate; a result here names its adapter (R13) and is not the gate.
- **Timing scopes** (`G-FPS`, `G-TOP`, `⏱`, R12: «Quadro only»): linux-pc only.
- **CPU-twin scopes** (`cargo test --lib`, the f64 oracle): as written — no GPU needed.
- **Web smoke** (`web/smoke.mjs`, contract §6.2.3): Windows variant — the Chrome path and the ANGLE/D3D12 WebGPU flags replace `--use-angle=vulkan --disable-vulkan-surface`; UNVERIFIED until the first web smoke runs.
- **Anything `bash tests/*.sh` that names `xvfb-run`, `/usr/bin`, `~/.cache` by Unix path**: Git Bash maps `~` and `/tmp`; `xvfb-run`, `/usr/bin/google-chrome` and Wayland do not exist.

## Hazards
Each Windows difference that bit — dated 2026-10-08, root-caused; also copied into the plan's Repo facts → Hazards.

1. **`bash` is two programs.** Root cause: Windows' PATH puts `C:\WINDOWS\system32\bash.exe` (the WSL relay; the default distro is docker-desktop, no `/bin/bash`) ahead of Git's. Effect: every `bash …` the toolkit spawns fails from PowerShell (`verify.py toolchain` → 0 of 3 cases); in the Bash tool it resolves to Git's `usr\bin\bash.EXE` and works. Rule: toolkit calls go through the Bash tool.
2. **No `rsync`.** Root cause: neither Windows nor Git for Windows ships it. Effect: `scratch_copy.sh` failed 9 of 12 selftest checks (`find: '/tmp/scratch_copy_selftest…': No such file`). Fixed by the lead-approved install (Installed).
3. **Symlinks.** Root cause: Git Bash copies instead of linking unless `MSYS=winsymlinks:nativestrict`, which needs Developer Mode (on). Effect: scratch copies' `.venv` / `node_modules` links; 3 more checks red. Fixed by the user variable; a fresh shell/VS Code restart picks it up, a shell opened before does not.
4. **`python3` is a Store alias.** Root cause: `C:\Users\YohanBelanger\AppData\Local\Microsoft\WindowsApps\python3.exe` runs the Store's Python 3.13.14, while `python` is 3.12.10 (python.org) and `py` defaults to 3.13. Effect: none today (real interpreter, ≥ 3.12; `verify.py` reports python 3.13.14); the Microsoft Store's *installer stub* (the failing case §B.2 warns of) is not what answers here. If a Store update ever removes the package, `python3` reverts to the stub: the fix is `py -3`.
5. **Node 26.8.2, not Node 20; Chrome 154, not 155.** Declared, not changed (R3 forbids downgrading); both satisfy what the approved table's tools need.
6. **Disk is 97 % full.** Root cause: 953 GiB drive, 922 GiB used. Effect: build caches (`build/target`, `~/.cache/sandbox-reactions/redarm-target/`, trunk's `%LOCALAPPDATA%\trunk`, `node_modules`) can fill the 31 GiB left; each lot's first big build should read the free space first.
7. **Windows file locks.** Root cause: Windows refuses to delete or replace an open file. Effect: `content_gate.py selftest` stops on WinError 32 (M0-D3); `plan.py` retries its own replace («needed 2 attempts - transient lock on this box»), harmless. 
8. **`tools_selftest` is slow here.** 170–254 s against linux-pc's ~10 s (process starts and temp-repo creation are slower on Windows); the complete loop is 75–97 s warm (Rules budget: 600 s). Not a defect; a TR counts it.
9. **Git line-ending warnings** («LF will be replaced by CRLF») appear in selftests' temporary repos, which have no `.gitattributes`; the real tree is LF throughout.

## Ruling
M0-TJ3, 2026-10-08 ~17:40 EDT, on win-laptop (Laser2025-20, Git Bash), the rung read by `rung_record.py now`:
model=claude-opus-5-5 level=max plugin=installed. One call of the question tool, four questions, the recommendation
first, each option's description its consequence as the lead read it. The lead's picks are the record (PLAYBOOK §12):
quoted below, never restated. Log: milestones/m0/logs/M0-TJ3.log; detail: milestones/m0/tasks/M0-TJ3.md.

### The questions as asked, and the answers (picked: **bold**, ← picked)
**Q1 · Where work** — «Your laptop is set up and the toolkit passes there (500 checks green). R5 says "linux-pc for every
M0 gate, win-laptop later". Where should M0's work run from now on?»
- «Build anywhere, close lots on PC (Recommended)» — «Coding tasks and their own checks run on whichever machine you're
  on; each lot's validation task (M0-V1, V2, …) runs on the Linux PC, where the PC-only checks run. Consequence: you can
  build on the laptop, but a lot only closes when you're at the PC.»
- **«Build and close lots anywhere»** — «A lot can also be validated on the laptop: PC-only checks are marked owed, the
  next validation on the PC runs them, and the final one (M0-V17) runs them all. Consequence: nothing waits for the PC,
  but a PC-only problem can surface a lot or more later.» ← picked
- «Keep the laptop off M0 builds» — «R5 stays as written: only plans, docs and toolkit work on the laptop. Consequence:
  coding tasks wait until you're at the PC.»

**Q2 · Win versions** — «Some checks name hardware or paths only the PC has. Should the laptop get its own version of
each where one is cheap? (Timing stays on the PC's Quadro RTX 4000 either way: it stands in for the 'mid-range gaming
PC'.)»
- **«Laptop versions (Recommended)»** — «The adapter test lists the laptop's own graphics cards (RTX 4080 Laptop, Intel
  Arc); physics checks run on the RTX 4080 where the PC uses the RTX 5090 (the contract has physics agree across cards
  within its tolerances); the web test uses Chrome's Windows install; a double-click start.bat sits beside start.sh.
  Consequence: most tasks can be built and checked on the laptop.» ← picked
- «PC only» — «Those checks run only on the PC and show 'owed on laserax-ai' on the laptop; a task whose own check is
  PC-only waits for the PC. Consequence: no extra work, but most physics tasks can't be finished on the laptop.»

**Q3 · Window** — «On the PC, agents test the game window on a private virtual screen (Xvfb) that you never see.
Windows has no such screen. How should the laptop test the window (and the screenshot and UI-cost checks that need
it)?»
- **«Off-screen window (Recommended)»** — «Agents open the game window outside the visible screen area (never focused,
  not in the taskbar) on the laptop's own GPU, and R4 (the debug-run grant) is extended to count that as off-screen.
  Consequence: window work can be built on the laptop with nothing popping up; untested yet, so if M0-T5 finds it can't
  work, window tasks go back to the PC.» ← picked
- «Window work at the PC» — «On the laptop the window, screenshot and UI-cost checks show 'owed on laserax-ai'.
  Consequence: the tasks that build them (M0-T5, M0-T6's check, M0-T8, the UI passes, lots 12–13) run at the PC.»

**Q4 · Crash check** — «M0-D7 left one proof owed before the laptop joins: the toolkit's selftests 20 times in a row
under Python 3.12 with no crash (the 0xC0000005 crash seen twice). About 30 minutes, over the 10-minute line, so it's
yours to run. When?»
- «Run it now (Recommended)» — «You paste one loop into a Git Bash terminal (not PowerShell) and let it run while I write
  the ruling; I check the 20 results before closing. Consequence: the laptop joins with the crash ruled out, or with a bug
  task holding the crash's stack.»
- «Before lot 1 closes» — «The laptop joins now; you run the loop any time before M0-V1, which reads its 20 logs.
  Consequence: until then, a crash in a task's final check run can send that task back to TODO.»
- **«Skip it»** — «The laptop joins without it; tracing is on, so any later crash is caught with its stack and filed as
  a bug. Consequence: no 30 minutes now, but crashes may still interrupt tasks.» ← picked

### What the ruling changed — each surface cited to its answer
| Surface | Change | Answer |
|---|---|---|
| R5 (Rules row, m0_rules.md) | both boxes in play; any block, lot V's included, runs on the box the lead is on; the old text mirrored in Superseded | Q1 |
| R15 (new: Rules row, m0_rules.md) | the box pattern — what a block written for linux-pc reads on win-laptop, the owed-check mechanism, switching boxes | Q1, Q2, Q3 |
| R4 (Rules row, m0_rules.md) | the DEBUG lane counts the off-screen window on win-laptop as off-screen | Q3 |
| R12 (Rules row, m0_rules.md) | physics ⏱ on the RTX 4080 Laptop on win-laptop; timing on the Quadro only, unchanged | Q2 |
| R9 (m0_rules.md) · the Rules' budget line | the loop budget stated per box (PLAYBOOK §8), win-laptop's measured 97.7 s | Q1 |
| Contract §6.6 | rewritten in place [M0-TJ3]: win-laptop in play — adapters, the off-screen route, physics, timing never, Chrome, launchers | Q1–Q3 |
| Contract §3.3 · §3.6 | `--offscreen-window` · the `game-offscreen` service · start.bat beside start.sh [M0-TJ3] | Q3 · Q2 |
| Contract §6.2.3 · §6.4 · §5.4 (G-DESK, G-WEB) | Windows Chrome for the web smoke · the RTX 4080 Laptop for physics · each guarantee's win-laptop route [M0-TJ3] | Q2, Q3 |
| M0-T2 | the adapter test names each box's adapters | Q2 |
| M0-T5 | the off-screen route built with the window; the desktop scope's case `xvfb` renamed `window` (the box picks its route) | Q3 |
| M0-T6 | start.bat beside start.sh; the `launcher` case through the box's route; no longer an edit job (re-rated, §0 rule (3)) | Q2, Q3 |
| M0-T8 | the capture check's win-laptop route | Q3 |
| M0-T9 | running.md and testing.md name both boxes | Q1, Q2, Q3 |
| M0-T84 | its Pass line's case names (`window` for `xvfb`) | Q3 |
| M0-V1 | runs on the box the lead is on; the human-run tier double-clicks that box's launcher | Q1, Q2 |
| M0-V17 | runs on linux-pc and runs every check still owed there; names any owed on win-laptop with its last GO | Q1 |
| Goal paragraph · Repo facts (Targets, Boxes, launchers) | "the Windows laptop" no longer routed past M0; win-laptop in play | Q1 |
| M0-D7's 20-of-20 loop | retired, N/A — Superseded | Q4 |
| M0-D8 (filed) | `tools_selftest[verify]` crashed in M0-TJ3's claim run (an IndexError in a three-field parse, once in five runs) — a D placed before M0-T1 and the Phase 2b gate | Q4: «any later crash is caught with its stack and filed as a bug» |

### Where each owed check runs
- **M0-D2..D7's toolkit fixes, owed on linux-pc** (each D's handoff; M0-D3's and M0-D7's flags on this block):
  `python3 tools/pb/verify.py tools_selftest` (10/10, content_gate 36/36) and `python3 tools/pb/verify.py --redarm
  tools_selftest` on linux-pc — flagged into M0-V1's `Carried flags:`; a V on win-laptop carries the flag to the next V
  (R15); M0-V17, on linux-pc, at the latest.
- **M0-D7's 20-of-20 loop on win-laptop** — not run, retired by the lead (Q4 «Skip it»); a later 0xC0000005 is a D with
  the faulthandler stack (Hazards).
- **A check only one box can run, from lot 1 on** (R15): the block that meets it tags it `[NOT RUN — owed on <box>]`; the
  lot's V lists it and flags it into the next V; the next V run on that box runs it first; M0-V17 runs every one owed on
  linux-pc (Q1's «the final one (M0-V17) runs them all») and names, with its last GO, any owed on win-laptop (PLAYBOOK §8).
- **A block whose `Deliver:` is a Quadro measurement** — M0-T63, M0-T112, M0-T113 — runs on linux-pc (Q2's «Timing stays on
  the PC's Quadro RTX 4000 either way»); its scope is `box: laserax-ai`.
- **The two laptop routes' first proofs:** the off-screen window — M0-T5 on win-laptop (UNVERIFIED until then; a red
  sends the window tasks to linux-pc, Q3); the Windows web smoke — M0-T15 on win-laptop, else owed there to the next V on
  win-laptop (UNVERIFIED, §6.2.3).

### The later blocks R15 covers, with no per-block edit
- **The RTX 5090 → the RTX 4080 Laptop** (physics, calibrate): M0-T30 (Adversarial), M0-T66 (Verify 3), M0-T70, M0-V9,
  M0-T97, M0-T98b, M0-T99b, M0-T100, M0-T101, M0-T103, M0-T104, M0-T105, M0-T106, M0-T107.
- **The Quadro RTX 4000 → linux-pc only, owed elsewhere** (timing): M0-T31 (Verify 2–3), M0-T34, M0-T42, M0-T46 (their
  timing steps), M0-V6 (the costs gathered), M0-T63, M0-T95 (the real `--measure-ui` run), M0-V13 (the lead's
  `--measure-ui 600`), M0-T112, M0-T113, M0-V16 (the lead's fps and top-speed runs), M0-TJ2's wake.
- **Xvfb, `xvfb-run`, `game-xvfb`, llvmpipe and lavapipe → the off-screen window**: M0-T95 (tests/measure/check.sh),
  M0-TV1, M0-TV2, M0-TV3 (the captures; a state lavapipe cannot reach is reached by the laptop's own GPU), M0-V12
  (Adversarial), M0-TD (the captures).
- **start.sh → start.bat** (the lead's double-click): M0-V12, M0-V13, M0-TW.
- **/usr/bin/google-chrome and its Linux flags → §6.2.3's win-laptop line**: M0-T15, M0-T94, M0-TV1 (the no-WebGPU
  capture), M0-TV3 (the web capture), M0-V13 (G-WEB).
- **`python3 tools/pb/<tool>` → `py -3.12 tools/pb/<tool>`** in Git Bash: every block's commands (M0-D7's switch, kept).
- **"on linux-pc" in a lot V → the box it runs on**: M0-V2–M0-V16.

### Not retired, explicitly
- linux-pc's routes and checks — Xvfb with lavapipe (`game-xvfb`), start.sh, /usr/bin/google-chrome with its flags,
  llvmpipe in the adapter list, the RTX 5090 for physics there: each stays; the laptop's versions sit beside them.
- Timing on the Quadro RTX 4000 — G-FPS, G-TOP, the per-pass costs, step_cost.md, render_reserve_ms (R12, §5.3, §6.1):
  linux-pc only; no timing gate on the laptop's GPUs.
- R4's exclusions — a visible window on the lead's screen is asked each time, on both boxes; a DEBUG run is never
  acceptance; the 10-minute line.
- The FROZEN clauses (Q1–Q5 of M0-TC), every physics oracle and tolerance (§0.4: never a looser number), bit-identity per
  adapter (§1.3.4) — never compared across boxes.
- R2 and R3 as written (R2 reaches win-laptop by its own words, «once it is in play»); R10's claim-run base; §10 — agents
  never write git.
- M0-D7's switch (`py -3.12`) and trace (`PYTHONFAULTHANDLER=1`) — only its 20-run loop is retired.
- The owed linux-pc toolkit checks of M0-D2..D7 — routed above, not retired.
- Steam, a Windows release, installer or store build, and the goal paragraph's other routed items — still past M0.
