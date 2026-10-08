# M0 rules — full text (the plan's `## Rules` is the index)

Binding as if it sat in the plan (PLAYBOOK §11). Numbers are never reused across milestones. A
grant states scope · exclusions · ceiling, and is a lane, never a licence: anything paid, a run
over 10 minutes, a visible or foreground-stealing window, or anything irreversible is still asked
every time (§13 OPT-C). Grants buy execution, never acceptance.

## R1 · The Python venv
- The lead, 2026-10-08: "we can create a .venv here in this folder for python if we need it".
- Scope: one `.venv/` at the repo root, git-ignored, created when a task's `Deliver:` needs Python
  packages. The playbook's own tools need none (standard library only, annex §A.0).
- Exclusions: a package outside the approved dependency table still asks (§4 Rule 2); the venv is
  never copied into a scratch copy — `tools/pb/scratch_copy.sh` symlinks it (§11).

## R2 · Grant — BUILD lane (OPT-C)
- The lead, 2026-10-08, picked "Build and test (Recommended)", shown as: "Compile, run the test
  suites and fetch the approved libraries on their own."
- Scope: on linux-pc, and on win-laptop once it is in play: compile the game and its tools; run
  every deterministic suite, the headless smoke and the web build's smoke; fetch the libraries the
  approved dependency table names, through the stack's own package manager. Granted means run —
  handing a granted run back to the lead is the failure this lane closes.
- Exclusions: a library outside the approved table (ask, §4 Rule 2) · a system-wide install (R3
  says which are the agent's) · a run expected to take over 10 minutes (§4 Rule 4: the lead's, in
  one visible terminal) · anything paid.
- Ceiling: $0.

## R3 · Grant — installs, on both PCs
- The lead, 2026-10-08: "this pc only for now, but I also work on a windows laptop, so when an
  agent finds out hes on windows, he drains the installs and the setup" — my reading: on the
  Windows laptop, the agent does the installs and the setup itself — then picked "Install
  approved tools, both PCs (Recommended)", shown as: "Like your Windows rule: an agent installs
  what the approved list names (e.g. the Rust toolchain in your home folder). Anything paid still
  asks."
- Scope: the tools the approved dependency table names — a compiler toolchain, a build tool, the
  web target's toolchain, an SDK the table lists — installed user-level (the home folder or a
  project-local folder) on linux-pc and win-laptop; on win-laptop, the whole environment setup the
  table implies, the box probed first (§4 Rule 5). Each install is logged in the task's log with
  its version, and the version lands in Repo facts.
- Exclusions: anything paid · anything outside the approved table · a system-wide install that
  needs sudo on linux-pc or an elevation prompt on win-laptop — handed to the lead as one command
  tagged `[NOT RUN — for you]` · removing or downgrading anything the lead installed.
- Ceiling: $0.

## R4 · Grant — DEBUG lane (OPT-C)
- The lead, 2026-10-08, picked "Run the game off-screen (Recommended)", shown as: "Agents run their
  own build headless to check their work — never as your acceptance. Opening a visible window
  still asks you each time."
- Scope: run your own build headless — offscreen rendering, or a private virtual display (Xvfb)
  that the run starts and stops itself — to check your own work before handoff. Its frames are
  evidence for your handoff. [M0-TJ3] On win-laptop, which has no private display, the game's window
  placed outside every monitor's area, never activated and kept out of the taskbar (`--offscreen-window`,
  contract §6.6) counts as off-screen: it never shows on the lead's screen. The lead, 2026-10-08, at
  M0-TJ3, picked "Off-screen window (Recommended)", shown as: "Agents open the game window outside the
  visible screen area (never focused, not in the taskbar) on the laptop's own GPU, and R4 (the debug-run
  grant) is extended to count that as off-screen." (reports/win_laptop.md § Ruling, Q3).
- Exclusions: a window on the lead's desktop (asked each time) · acceptance: a DEBUG run never
  closes a `Pass:` the lead or a V owns · a run over 10 minutes.
- Ceiling: $0.

## R5 · Boxes [amended by M0-TJ3, 2026-10-08 — the old text in the plan's Superseded section]
- The lead, 2026-10-08: "this pc only for now, but I also work on a windows laptop"; then, at M0-TJ3,
  "Build and close lots anywhere", shown as: "A lot can also be validated on the laptop: PC-only checks
  are marked owed, the next validation on the PC runs them, and the final one (M0-V17) runs them all.
  Consequence: nothing waits for the PC, but a PC-only problem can surface a lot or more later."
  (reports/win_laptop.md § Ruling, Q1).
- Two boxes are in play for M0: linux-pc (hostname laserax-ai) and win-laptop (Laser2025-20). Every
  block — BUILD, CHECK or PLAN, the lot V's included — runs on the box the lead is on; how a block
  written for one box reads on the other is R15's. An agent probes its box before phrasing any command
  (§4 Rule 5): `uname -a` on POSIX, `$PSVersionTable` on Windows. A scope only one box can run declares
  `box: <hostname>` in the manifest and reads `owed on <box>` elsewhere, never NO-GO (§8); the next V
  run on that box runs it, and M0-V17 runs every check still owed on linux-pc (R15). Every verdict
  names its box.

## R6 · Modules assumed at bootstrap
- OPT-B (show-off demo) on: the lead asked for the star's life "As a first demo"; M0-TG places a TD
  after the final V.
- Off: OPT-A (no measurement campaign yet; a later plan may turn it on for performance work) ·
  OPT-D (no real user data) · OPT-E and OPT-F (one product: story mode and sandbox mode are modes of
  one game) · OPT-G (the lead's own game: "be able to sell it on steam").
- Declared in reports/bootstrap.md §4; confirmed by the lead at M0-TP, 2026-10-08: "Yes, as
  assumed (Recommended)" (reports/plan_redteam.md §8).

## R7 · Copyleft
- The lead, 2026-10-08, at M0-TP: "No copyleft code (Recommended)", shown as: "The game stays
  closed-source and sellable on Steam with no duty to publish its code. GPL projects like The
  Powder Toy are studied for ideas only, never copied or translated; an LGPL library only as a
  separately linked file (rule R7 as written)."
- No copyleft source (GPL, AGPL, or an LGPL library linked statically) enters the tree. Copyleft
  projects — The Powder Toy among them, its licence to be read by M0-R1 (UNVERIFIED until then) —
  are studied for ideas and behaviour, never copied, translated or paraphrased line by line.
  Every dependency's licence sits in the approved table, copyleft flagged (§4 Rule 2).
- Why: the lead's "be able to sell it on steam" and the ruling above — a closed-source commercial
  game, which copied copyleft code would oblige to publish its source.

## R8 · Physics oracles
- Every physics behaviour the contract fixes names its oracle — an analytic solution, a published
  relation or a measured value, with its source — and a tolerance; a harness scope checks the
  engine against it and is red-armed by a planted bug (§8). A capture that looks right is
  evidence of looks only: for physics it is NOT PROVEN (synthetic).
- Why: §8's "Real data, or it is not proven", read for a simulation under the lead's "Real laws,
  squeezed scale" (reports/sandbox_interview.md, answer 1, shown as "Tests check each law's exact
  answers, not the real Sun's numbers"): its real data are each law's exact answers in sandbox
  units, and nature's measured numbers anchor only the readouts' translation to real units
  (re-worded at M0-TP, 2026-10-08, on the lead's "Rewrite them (Recommended)"). The lead wants "as
  much physics as we can"; without oracles, plausibility drifts unnoticed.

## R9 · The routine — one harness, one order (PLAYBOOK §4 Rule 4)
- Written by M0-TH, 2026-10-08. The runner is `tools/pb/verify.py` over `tools/pb/verify.json`; how to add a
  scope, its count parser for this stack and what each scope costs: `docs/agent/testing.md`.
- The order, for every task that builds or changes something checkable: (1) author the artifact and its scope,
  with the scope's planted bugs (`tests/plants/<plant>.patch`, or a command for a file that moves); (2) iterate on
  **your scope alone** until green (`verify.py <scope> --task <ID>`); (3) red-arm it while you iterate
  (`verify.py --redarm <scope> --task <ID>`: the clean copy GO, each plant NO-GO); (4) make every plan edit through
  `plan.py`; (5) run `verify.py --changed --base <named commit> --task <ID>` **once, last** — the claim about the
  tree you hand over; (6) write the handoff from that run; (7) a fix after step 5 is re-verified scoped, one block
  per scope it could have moved. The complete loop (`--all`) runs at phase-closing V and TR blocks, never per task;
  a second green loop on an unchanged tree is waste, named in the handoff.
- Scope: a scope is a manifest entry, never a second runner. Every task that builds a contract surface adds its
  scope or case; a scope only a plant can turn red is unfinished. A scope without paths is a placeholder.
- Budget: the complete loop's wall clock on linux-pc is held under 600 s (Rule 4's 10-minute line; measured 3.2 s
  warm at M0-TH). Over it, or on the lead's word, a TR measures and files the fixes — no other trigger.
  [M0-TJ3] The same 600 s on win-laptop, in play since M0-TJ3 (R5) — PLAYBOOK §8 states the budget per box;
  measured 97.7 s wall there (3 scopes, 500 passed, M0-TE-win-r2, 2026-10-08). Each V restates its own box's.
- Why: PLAYBOOK §4 Rule 4 — iterating on a moving artifact under a wide loop costs the lead's time, and a check
  nobody has seen red proves nothing.

## R10–R14 · The Build preamble — every build lot's shared rules
- Written by M0-TG (2026-10-08) once above the build lots; hoisted here by M0-TB (2026-10-08), each bullet
  moved byte for byte (logs/M0-TB.log), because a task's start read (`plan.py show`) prints the header and
  its own block, never that preamble. "Below" in R14 reads "in the build lots".

### R10 · The claim run
- **The claim run.** After its scope's GO and its red-arm, every BUILD block's last step is R9's claim
  run, `python3 tools/pb/verify.py --changed --base <the last commit gate's SHA> --task <its ID>` · Pass:
  GO · Fail: NO-GO — the base a named commit, never `HEAD` (PLAYBOOK §A.2).

### R11 · Scopes
- **Scopes.** GPU scopes are modules of one test binary, crates/sr-engine/tests/gpu/ (M0-T2), each run
  as `cargo test -p sr-engine --release --test gpu <module>::`; pure-CPU scopes as `cargo test -p <crate>
  --release --lib <module>::` (sr-app's with `--bin sandbox-reactions`); both read with
  docs/agent/testing.md's regex parse; scripts source tests/cargo.sh (M0-T1). Each lot's docs block
  writes its scopes' rows into docs/agent/testing.md (contract §7). A path marked (new) did not exist on
  2026-10-08: the tree held no game code (`git ls-files`).

### R12 · ⏱ scopes
- **⏱ scopes** (contract §5.0, §6.4): their `paths` name only their own test and scene files, so
  `--changed` runs them only when those change; physics ⏱ scopes run on the RTX 5090 [M0-TJ3: on
  win-laptop, the RTX 4080 Laptop — R15; the lead's Q2, "Laptop versions (Recommended)"]; a run over 10
  minutes is the lead's, in the lot V's window, one visible terminal [NOT RUN — for you]; the timing
  scopes (fps, top-speed) run on the Quadro RTX 4000 only.

### R13 · Physics
- **Physics.** Every GPU number names its adapter (Hazards). A physics row passes only against its
  oracle within its tolerance (R8); an UNVERIFIED row (§5.5) that goes red is a D, or a question to the
  lead when a tolerance or a FROZEN clause is at stake — never a looser number (§0.4). Each pass's CPU
  f64 twin comes before its GPU shader (D11, G-REF); the box and the latch are first-lot architecture
  (M0-R3's and M0-TC's flags).

### R14 · Every V
- **Every V below** is done when its verdict is recorded, its D's created and the register updated
  (PLAYBOOK §14.3's V template); a V that adds to that says so on its own `Done when:` line.

## R15 · The box pattern — what a block reads on each box (M0-TJ3)
- The lead, 2026-10-08, at M0-TJ3 (reports/win_laptop.md § Ruling): Q1 "Build and close lots anywhere", Q2 "Laptop
  versions (Recommended)", Q3 "Off-screen window (Recommended)". Every block from M0-T1 on reads this rule; a block
  written for linux-pc is never edited one by one for win-laptop where this table suffices.
- **Which box.** A block — BUILD, CHECK or PLAN, the lot V's included — runs on the box the lead is on (R5). On
  win-laptop, a block that names linux-pc's hardware or paths reads them so:

| A block names (linux-pc) | On win-laptop it reads |
|---|---|
| the RTX 5090 — physics scopes, `calibrate`, probes (R12, contract §6.4) | the RTX 4080 Laptop (`--adapter "RTX 4080"`); every number names its adapter (R13), cross-adapter agreement within G-REF's tolerances (contract §6.7) |
| the Quadro RTX 4000 — timing: G-FPS, G-TOP, per-pass costs, step_cost.md, `--measure-ui`'s render_reserve_ms | nothing: linux-pc only, `box: laserax-ai`, read `owed on laserax-ai` here |
| Xvfb, `xvfb-run`, `game-xvfb`, llvmpipe or lavapipe — the window, capture and UI-cost checks | the off-screen window: `--offscreen-window`, the `game-offscreen` service, the box's own adapter (contract §3.3, §3.6, §6.6; R4) |
| start.sh — the lead's double-click, the `launcher` case | start.bat (contract §3.6) |
| /usr/bin/google-chrome and contract §6.2.3's Linux flags | contract §6.2.3's win-laptop line (Chrome under `C:\Program Files\Google\Chrome\Application\`) |
| linux-pc's adapters (M0-T2) | the RTX 4080 Laptop and the Intel Arc (contract §6.6) |
| `python3 tools/pb/<tool> …` | `py -3.12 tools/pb/<tool> …`, in Git Bash — never PowerShell (Hazards, M0-D7) |
| `build/target/release/sandbox-reactions` | the same; the file is `sandbox-reactions.exe` — a script never assumes the suffix away |
| "on linux-pc" in a V's `Deliver:` | the box the V runs on (M0-V17 excepted: linux-pc) |

- **One check, one route per box.** A check that differs by box keeps one scope name and its case names; its script
  picks the box's route from the table above (by `uname -s`), never a second scope. A check only one box can run
  declares `box: <hostname>` in verify.json (`laserax-ai` = linux-pc, `Laser2025-20` = win-laptop) and reads
  `owed on <box>` elsewhere, never NO-GO (PLAYBOOK §8).
- **Owed checks.** A block's Verify step that only the other box can run is tagged `[NOT RUN — owed on <box>]` in its
  handoff; a block whose `Deliver:` is a Quadro measurement (M0-T63, M0-T112, M0-T113) runs on linux-pc. A V names its
  box and lists every check owed on the other box — the route it did not run included — and flags them into the next
  V (`plan.py flag`); the next V run on that box runs them first; M0-V17 runs on linux-pc and runs every check still
  owed there (Q1: "the final one (M0-V17) runs them all"), and names any owed on win-laptop with its last GO.
- **Switching boxes.** Agents never write git (PLAYBOOK §10): the lead commits and pushes on the box they leave — at a
  commit gate where they can; mid-phase `git add -A`, `git commit -m "M0 WIP: switching boxes after <id>"`, `git push`
  — and runs `git pull` on the box they join. The claim run's base stays the last commit gate's SHA (R10).
- **The lead's own runs** (a visible window, a run over 10 minutes) run on the box the block names, else on the box the
  lead is on; the human-run tier double-clicks that box's launcher.

## Considered and REJECTED (false economy)
Binding (PLAYBOOK §2.1, §11): a later agent proposing an item below re-opens a closed decision — it asks
the lead with the new evidence, never applies it. Started by M0-TB, 2026-10-08.
- **Merging each lot's docs block into its code blocks** (fewer sessions) — two artifact kinds in one
  block (§2.1 (i)); contract §7 gives each lot its page, and the docs block reads the lot's handoffs whole.
- **Merging a pass's CPU twin and GPU blocks** — the twin before the shader is first-lot architecture
  (R13; D11, G-REF); merged, one block holds two artifacts and the twin's oracle loses its independence.
- **Dropping each block's red-arm Verify step for a citation of R9** — the plant is a `Deliver:` item;
  without its Verify step it becomes an item no check names (§3.2), which the lint flags.
- **Replacing each lot V's phase-close checklist by a citation of PLAYBOOK §14.3** — the inline line
  and a half is cheaper than every V opening the playbook to find it.
- **One TV for the whole milestone** — §8 asks one per phase that adds or changes screens; a late TV
  turns early UI misfits into a wall of D's.
- **The complete loop only at the final V** — it costs seconds today (3.2 s at M0-TH); each phase-closing
  V's loop time is the measurement that fires a TR (R9).
- **Dropping `Adversarial:` lines from build blocks** — mandatory on implementation blocks of an M or L
  milestone (§3.2).
- **One "whole-life" scope for the ⏱ guarantees** — each §5 row is its own scope with its own plant
  (contract §5.0); a merged scope's red cannot name the guarantee that failed.
- **Hoisting the shared tail of `Read:` lines** (`+ docs/agent/testing.md`) — three words a block; a
  `Read:` line lists exactly what its block reads (§3.2).
- **Stepping a near-ceiling block up a rung instead of splitting it** — size is §2.1's: a block too big
  for its rung splits, never steps up (§0, no step up on a guess).
