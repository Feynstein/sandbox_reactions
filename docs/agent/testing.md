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

## Scopes today (linux-pc, 2026-10-08, warm caches)
| Scope | Checks | Cases | Cost | Plants |
|---|---|---|---|---|
| `plan_lint` | the plan's structure (`plan.py lint`) | ≥ 30 (49 now) | 0.1 s | `plan-bad-status` (a command: the plan moves, a patch would not apply) |
| `tools_selftest` | each annex tool's own red-armed selftest — proves the tools, never the game | 10 | 10 s | `tools-selftest-scratch` |
| `toolchain` | the stack builds a one-file program: `desktop` (rustc 1.99), `wasm` (wasm32-unknown-unknown), `trunk` (trunk 0.21.14 + wasm-bindgen 0.2.129) | 3 | 0.3 s warm · ≈ 9 s cold (first trunk run fetches wasm-bindgen-cli into `~/.cache/trunk`) | `toolchain-compiler`, `toolchain-web-target`, `toolchain-trunk` |
| `build` | the game's workspace (R11): `native` — `cargo build --workspace --release --locked` then `cargo clippy ... -- -D warnings`; `wasm` — `cargo check --workspace --target wasm32-unknown-unknown --locked` (M0-T1) | 2 | ≈ 5 s on win-laptop (the empty `sr-physics`; grows with the crates) | `build-wasm-only` |

The complete loop: 3.2 s wall on linux-pc (62 cases, `jobs` 8). `tools_selftest` GO is the tools' GO, never the
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
- **`capture_web.mjs`'s live tier** needs Playwright (arrives with `web/package.json`): until then its selftest
  reports `live=0` — `NOT RUN (no Playwright)`.

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
- Every build, check and test is `--locked`: `Cargo.lock` is committed, and adding a dependency means regenerating it.
- One build profile: `[profile.release]` in the root `Cargo.toml`; `[profile.test]` inherits it (the CPU twin needs optimised code).
- `rust-toolchain.toml` pins 1.99.0 (+ clippy, rustfmt, targets linux-gnu and wasm32); the first cargo call in a tree auto-installs that
  toolchain user-level (win-laptop: 2026-10-08, ≈ 25 s, beside `stable`).
- A new member crate lands inside `crates/*` and is covered by the `build` scope's `crates/**` path; a cargo test scope is
  `cargo test --release --locked -p <crate> --lib` (R11) and sources `tests/cargo.sh` for the environment.

## Long runs
`calibrate`, the ⏱ scopes and benches at low rungs can pass 10 minutes on the Quadro: the lead's, in one
visible terminal, placed in V windows (contract §6.4). A scope the lead runs sets its own `timeout_s`.
