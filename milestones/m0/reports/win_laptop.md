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
| `plan_lint` | GO, 479 checks | GO |
| `toolchain` | **GO, 3 cases** (desktop, wasm, trunk) — cold 113 s, warm 4.4 s | GO, 3 |
| `toolchain` red-armed | **GO: clean GO, 3 plants / 3 red** (65 s) | same |
| `tools_selftest` | **NO-GO, 10 cases: 6 GO, 4 red** | 10 cases, GO |
| loop | 75–97 s wall (jobs 8; `tools_selftest[launch]` alone is 97 s) | 3.2 s |

Case counts equal linux-pc's (plan_lint 479, tools_selftest 10, toolchain 3); no case skips — the one SKIP printed inside
`launch.py selftest` («private X display: an X server is POSIX only — NOT RUN on os nt») is that tool's own, inside a GO.

Red cases of `tools_selftest`, each reproduced alone and routed to a D (filed, specs in the plan):

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
