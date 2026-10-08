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
  evidence for your handoff.
- Exclusions: a window on the lead's desktop (asked each time) · acceptance: a DEBUG run never
  closes a `Pass:` the lead or a V owns · a run over 10 minutes.
- Ceiling: $0.

## R5 · Boxes
- The lead, 2026-10-08: "this pc only for now, but I also work on a windows laptop".
- linux-pc is the box for every M0 gate; win-laptop joins when the lead first runs a task there.
  An agent probes its box before phrasing any command (§4 Rule 5): `uname -a` on POSIX,
  `$PSVersionTable` on Windows. A scope only one box can run declares `box:` in the manifest and
  reads `owed on <box>` elsewhere, never NO-GO (§8). Every verdict names its box.

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

## Considered and REJECTED (false economy)
- none yet — M0-TB starts this list (§2.1).
