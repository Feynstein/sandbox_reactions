# M0-TM7 — the rating pass (2026-10-15, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt right after
the heading's grep). That is the heading's rung, so no mismatch. Box: linux-pc (laserax-ai, Linux 7.0.0-38-generic). Raw
runs: logs/M0-TM7.log, plus the logs verify.py writes. Read-only helpers sit in the session scratchpad
(`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/06f81b74-cb58-40eb-922e-09cf926994ca/scratchpad/tm7/`). They
are never in the repo and never write the plan:
- blocks.py: each block's heading, type, rating, switch, status, `Sizing:` and flag stamps; `diff <rev>` compares against an
  older plan, `full` prints a block;
- checkdiff.py: the diff against a snapshot, each changed line classed rating · models · own · register · other. Red-tested on
  a mutant (a rating changed, a ` · switch` dropped, a `Read:` line edited): rating=1, other=2, NO-GO, exit 1. Its control
  (the `Models:` line alone): models=1, GO.

## What fired it
M0-D46's claim run: `plan_lint` read the refresh at 5 of 5 (Rules: `Rating refresh: 5 flags`). These are the carried flags on
open BUILD blocks stamped after `Model ratings: 2026-10-14 by M0-TM6`. The lint counts five today:
- M0-D48 `[M0-D46]`: testing.md's sink row (30 cases now; P3's motion over the box's tiles).
- M0-T57 `[M0-V7]`: source-term gravity's momentum, and the sinks' energy terms (reports/v7.md § Observations 1–2).
- M0-T64 `[M0-V7]`: architecture.md's pass table and frame sections have no owner (§ Observations 6).
- M0-T106 `[M0-V7]`: G-CONS's momentum clause against source-term gravity (§ Observations 1).
- M0-T112 `[M0-V6]`: hard steps on the Quadro (P6 ≈ 43 ms over a runaway; a burnt iron field's P5 ≈ 3.6 ms a step).

The heading's flag also counted M0-D45's flag to M0-D46, which M0-D46's close retired.

Also read, though the lint does not count them:
- M0-T63 `[M0-TJ4, 2026-10-14]`: P5 is implicit, and its Quadro cost is M0-T122's reports/p5_cost.md. It is stamped M0-TM6's
  day but was filed after M0-TM6 closed (08:08), so no pass had read it.
- Flags on gates: M0-V8 `[M0-V7]` (owed on win-laptop), M0-TZ `[M0-T52a]` × 2, and this block's own `[M0-D46]`.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-300_agent`.
  - Claude Code moved from 2.1.296, the version the `Models:` line cited, to 2.1.300.
  - `CLAUDE_EFFORT` is the process's launch level, a hint. The switch ran this session at max (`now`).
- **The model pages: read.** The Skill tool loaded the claude-api skill from 2.1.300's bundle. Its "Current Models (cached:
  2026-10-06)" table:
  - Prices per MTok: Fable 5.1 $10 / $50 · Opus 5.5 $4 / $20 · Sonnet 5.5 $2 / $10 · Haiku 5.5 $0.10 / $0.50 for prompts
    ≤ 100K tokens ($0.50 / $2.50 beyond).
  - Effort levels: low…max on all four. The default is medium for Opus 5.5 and Haiku 5.5, high for Sonnet 5.5.
- **shared/models.md.** Both bundles' extractions are on disk (`/tmp/claude-1000/bundled-skills/2.1.296/…` and `…/2.1.300/…`).
  - The diff between them is empty. Both hash (LF) to sha256 c1af1f093d46…0175675161b, M0-TM6's c1af1f09…5675161b. So the page
    is byte for byte the one M0-TM6 read (and M0-TM2–TM4, up to line endings).
  - All four ladder models are `Active`, and no model was added to the pages.
- **The result.** Nothing was retired, renamed, added, repriced or relevelled, and every rung's level is one its model takes.
  No rung the lead runs changed, so no question was owed (§0).
- **The `Models:` line**, re-dated and re-cited in the form M0-TM2, TM4 and TM6 used:
  - Before: "read 2026-10-14 from the claude-api skill's model table (Claude Code 2.1.296, cached 2026-10-06), the probe and
    the lead's answer (…)".
  - After: "read 2026-10-15 from the claude-api skill's model table (Claude Code 2.1.300, cached 2026-10-06), the probe and
    the lead's answer (…)".
  - The rungs and marks are byte-identical.
- **Not read: the live pricing pages** (the skill's shared/live-sources.md). The ladder cites the skill's table, cached
  2026-10-06. That list prices have not moved since then is NOT PROVEN (source: the bundle's cache).

## The record
- **The binding record** (§0: the phase-closing V's table) is M0-V7's: reports/v7.md § Phase 9's close (linux-pc,
  2026-10-15).
  - BUILD on Sonnet 5.5, high: 41 blocks, 30/39 first try, $2.02 per DONE block.
  - BUILD on Opus 5.5, high (usual): 53 blocks, 31/49, $6.26.
  - Rule (4): ≥ 10 blocks below (usual), at a cost below (usual)'s → **the class does not move**.
  - The other classes below (usual) are BUILD·E on Haiku 5.5, high (8), BUILD·E on Sonnet 5.5, medium (2), MOVE on Haiku
    5.5, high (2) and MOVE on Sonnet 5.5, medium (1). Each has n < 10, so rule (4) reads nothing there. M0-V7: "No TM".
- **Today's read on linux-pc** (`python3 tools/pb/rung_record.py report`): 148 DONE blocks, 12 class/rung rows, 68 blocks
  with no session file readable here, $377.69 read from the sessions' own cost.
  - BUILD on Sonnet 5.5, high: 42 blocks, 31/40, $2.01. BUILD on Opus 5.5, high: 54 blocks, 31/50, $6.44. It agrees: no move.
  - BUILD·E on Haiku 5.5, high: 8 blocks, 6/8, $0.95. MOVE on Haiku 5.5, high: 3 blocks, $0.17. Both n < 10.
- **Rule (1)**: no open block's `Carried flags:` holds an escalation trail. I scanned the 84 blocks for the trail's words
  (escalat, trail, NO-GO claim, back to TODO, rated (usual), failed attempt). The only hits are a test file named `trail ` in
  M0-TZ's flags.

## The rating: every open block, 84 of them
**Method.**
1. blocks.py diffed every open block against the plan at 4eb47e0. That is the tree M0-TM6 rated on (its claim read "tree
   4eb47e04d92c+…").
   - New: M0-D47 and M0-D48 (filed by M0-V7), and this block.
   - Changed:
     - flags only: M0-T57, T63, T64, T106 and T112;
     - one flags line added: M0-V8;
     - flags: M0-TZ.
   - The other 74 open blocks are byte-identical to that text, headings included.
2. The ladder is unchanged and the record moves no class, so an unchanged block keeps M0-TM6's derivation (tasks/M0-TM6.md §
   The rating). That derivation is M0-TM4's kinds (tasks/M0-TM4.md § The rating), re-checked here by type, `Sizing:` and flags.
   - Read in full: the new and changed blocks, and M0-T56, the one BUILD block above the cheapest rung. For each, the heading,
     `Sizing:`, `Carried flags:`, `Deliver:`, `Verify:` and `Pass:`.
3. §0's rubric, first match wins. The closed list is read as M0-TB set it (reports/size_pass.md §9) and M0-TM1–TM6 kept it:
   (a) a FROZEN passage, a schema, a persisted format or a migration changed · (b) a D whose cause is not measured · (c) a
   design the spec leaves open · (d) secrets, money or data loss · (e) numeric or precision semantics, or concurrency, as the
   block's own acceptance · (f) a library's internals. A block that reuses a mechanism an earlier block built does not earn
   the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 11 | V8–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM7, TJ1*, TJ2*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 1 | T56 | 0 |
| Sonnet 5.5, high | (3), no reason from the list | 64 | below | 0 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung | 4 | T91, T93a, T94, T111 | 0 |
| Fable 5.1, high · Opus 5.5, xhigh · Sonnet 5.5, medium | — | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`
(84 of 84).

The 64 on Sonnet 5.5, high: D47, D48, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73,
T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a,
T99a, T102, T103, T104, T105, T106, T107, T98b, T99b, T100, T101, T108, T109, T110, T112, T113, T114*, T115, TD, T116. These
are M0-TM6's 72 plus D47 and D48, less the ten now DONE (T44a, T44b, T45, T47, T49, T50, T52a, T53, T54, T55). Their kinds,
each read against the list, are M0-TM4's.

### The 1 BUILD block above the cheapest rung that fits, with its rule
The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. One block sits one rung above
it, on (usual), by rule (5): a reason from (3)'s list, re-derived from the block's own text.

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T56 | (e) | "each field against an f64 CPU sum of the same dump within the f32 rounding it states per field": per-field f32 precision is the block's own acceptance. "read back asynchronously (≤ 2 frames late), the accumulators reset after each summary": state shared across frames in flight. Unchanged since M0-TM4 |

M0-T46, T51 and D33, M0-TM6's other three on (usual), are DONE.

### The E blocks: Haiku 5.5, high (rule (3))
M0-T91, T93a, T94 and T111 are unchanged since 4eb47e0, and each still carries `Sizing: E`. M0-TM4's readings stand:
- M0-T93a is the closest call. Its asynchronous one-cell readback reuses M0-T22's `poll()` and M0-T56's ≤ 2-frames-late
  readback.
- M0-T111 keeps the k = 1 and k = 15 latch cases.

BUILD·E on Haiku 5.5, high has 8 blocks in the record (n < 10), so rule (4) moves nothing.

### The new and changed blocks, re-derived (every rating unchanged)
- **M0-D47** (new, filed on Sonnet 5.5, high, "rule (3) — no reason from §0's list"): **Sonnet 5.5, high**.
  - No (b): its cause is measured. "Suspected cause: measured — the step's constructor": sun_disk times no P3 and no P7, and
    headless.rs reads `switches.sinks` nowhere.
  - No (a): the step is built from §2.12.1's existing switches. The dump's format does not change; a dump holding sinks is read
    as M0-T52b writes it.
  - No (c): `Step::with_neutrino_heating` (M0-T54) already takes P3's and P7's setups, so this is wiring. Its one open choice
    (P7 with reactions off, "kept or refused, declared") is a declared default, not a module's interface.
  - No (e) and no (f).
- **M0-D48** (new): **Sonnet 5.5, high**. It is a docs block, and not E: it has three pages, graded by greps, and no scope
  (M0-V5's reading for M0-D31). Its flag `[M0-D46]` adds lines of the same kind (testing.md's sink row).
- **M0-T57** `[M0-V7]`: **Sonnet 5.5, high**. M0-TM4's kind: an observer on the CPU, fixed by the contract (§2.9, §2.12.2),
  whose fixed-order partials are M0-T56's.
  - Flag (1) is a measured fact about P4's scheme: 3.9 × 10⁻³ of Σm|u| in 40 steps on sink_cloud.json, on the twin and the RTX
    5090. The block's own disk case passes it only by symmetry.
  - Flag (2): §2.9 names no sinks' kinetic term, and accretion's inelastic loss and the swallowed mass's potential are booked
    nowhere. That is a gap in a fixed spec: a question for the lead (Changes by asking), "never a fixed spec's wording".
  - The ledger reports energy drift; G-CONS (M0-T106) grades it.
  - No reason.
- **M0-T63** `[M0-TJ4]`: **Sonnet 5.5, high**. A timing report, and the list excludes a timing choice.
- **M0-T64** `[M0-V7]`: **Sonnet 5.5, high**. A docs block; the flag proposes, by asking, adding architecture.md's frame lines.
- **M0-T106** `[M0-V7]`: **Sonnet 5.5, high**. M0-TM4's kind: a whole-life test whose UNVERIFIED red is a D or the lead's
  question. The flag says so itself: "a red here is the lead's (§0.4, §5.1: a conservative gravity formulation, never a looser
  bound)".
- **M0-T112** `[M0-V6]`: **Sonnet 5.5, high**. A timing test on the Quadro; the hard steps the flag measures are numbers for
  the bench to meet or report. Timing is never a reason.
- **M0-V8** `[M0-V7]` and **M0-TZ** `[M0-T52a]` × 2: a CHECK and a PLAN block, so rule (2) pins the gate.
- **M0-TM7**: PLAN, rule (2).

### Considered — observations, never defects (§0, no step up on a guess)
- **G-CONS's momentum clause is now measured.** At M0-TM6 it was source only.
  - Contract §5.1 marks momentum "by construction". M0-T57 (500 steps) and M0-T106 (whole lives) both grade it to 10⁻⁶ of
    Σm|u|.
  - M0-V7 measured P4's source-term gravity (Σ_½·g, §1.4.2) moving it by 3.9 × 10⁻³ in 40 steps; with gravity off, 1.1 × 10⁻⁶.
  - M0-TM6's bounce observation (M0-T35's flag; §2.7's « G-CONS grades mass and energy only ») stands beside it.
  - Both blocks carry the flag, and the route is set: the lead's ruling, never a looser bound. A rung prices a builder, not a
    contract ruling, so no step up.
  - The lead may want to rule on the clause before lot 8 reaches M0-T57: a conservative gravity formulation, or §5.1 amended.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and the
  rating is kept.

## What changed in the plan
- **Ratings: 0 changed.** By rung, now (changed):
  - Opus 5.5, max: 15 (0).
  - Opus 5.5, high: 1 (0).
  - Sonnet 5.5, high: 64 (0).
  - Haiku 5.5, high: 4 (0).
  - Fable 5.1, high, Opus 5.5, xhigh and Sonnet 5.5, medium: 0 (0).
- **The `Models:` line**: re-dated and re-cited (above).
- **The register**: `Model ratings: 2026-10-15 by M0-TM7`, written through `plan.py register --set` before the claim run, so
  the claim covers it (§4 Rule 4: step 4, the structural edits, then step 5, the claim).
- **How the `Models:` line was edited.** plan.py has no call that rewrites the `Models:` line or a rating. So, as M0-TM2–TM6
  did, I used the file-edit tool, one edit; `old_string` was the line's tail, the line grep-counted to 1 first.
- **The diff**, checked by checkdiff.py against a snapshot taken right after `plan.py status`, before the close: 0 rating
  segments, 1 `Models:` line, 1 register line, 0 other.

## Measured and not acted on
This block's diff is bounded to rating segments, the `Models:` line and the register's line (its `Pass:` and `Fail:`), so
nothing below is filed or flagged from here (M0-TM5's and M0-TM6's precedent).
- **G-CONS's momentum clause and §2.9's sinks' energy terms**: § Considered. M0-V7 already flagged both into M0-T57 and
  M0-T106. They are for the lead.
- **The Pipeline state's `Next task: M0-TM7`** goes stale at this close unless the close sets it. The next task is M0-D47 (the
  register: "M0-D47 … M0-D48 … — TODO, after M0-TM7").
- **Repo facts' agent line** names Claude Code 2.1.292 at the 2026-10-08 probe; today's probe reads 2.1.300. It is a dated
  probe, outside this diff.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the process's launch level, which the
  switch overrides (§B.3).

## The routine, on linux-pc (laserax-ai)
- **`python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**:
  - after the `Models:` line: [ALREADY RUN — PASS (GO, 558 checks, 0 failed, 0 warnings; header 552/600, ladder 7 rungs,
    refresh 5/5) on linux-pc];
  - after the register: [ALREADY RUN — PASS (GO, 558 checks, 0 failed, 0 warnings, refresh 0/5) on linux-pc].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (0 rating segments, 1 `Models:` line, 1 register
  line, 0 other) on linux-pc]. On the mutant: [ALREADY RUN — PASS (rating=1, other=2, NO-GO, exit 1 — the red it must show)
  on linux-pc].
- **No red-arm.** This task added no scope and changed no scope's check (R9; PLAYBOOK §4 Rule 4, v12.3).
- **The claim run**, after the structural edits: `python3 tools/pb/verify.py --changed --task M0-TM7`. `plan.py show` printed
  a `baseline:` line, so no `--base` is named (R10).
- **The close**: `python3 tools/pb/plan.py close M0-TM7 --status "DONE" --text "<handoff>"`, written from the claim run.
  Then checkdiff.py and the lint run again, and `plan_lint` is re-verified scoped, since the close's edit lands after the claim
  (§4 Rule 4 step 7).
- The results are below, appended after the runs.

### Results, appended after the runs
- **The claim run** (14:34), `python3 tools/pb/verify.py --changed --task M0-TM7`: [ALREADY RUN — PASS (GO: plan_lint 558
  passed, 0 failed; 1/1 scopes GO, 44 not impacted, 0 never checked, 0 stale, 0 unclassified, 0 owed; 0.12 s; base M0-TM7's
  start 2026-10-15T14:25:31-04:00; tree a2bed96f1172+7294b4c529b5; python 3.12.3; load 10.71) on linux-pc].
- **The close** (14:34): `close: checks=3 failed=0 warnings=0 · handoff_lines=1 · flow=per-task`, GO. Status `DONE
  (2026-10-15 14:34, started 14:30)`. The close does not set the register's `Next task:`, which still reads M0-TM7, left as
  found; M0-D47 runs next.
- **checkdiff.py** after the close: [ALREADY RUN — PASS (0 rating segments, 1 `Models:` line, 3 own lines — Status, Handoff,
  Flow row — 1 register line `Model ratings:`, 0 other) on linux-pc].
- **The lint** after the close: [ALREADY RUN — PASS (GO, 557 checks, 0 failed, 0 warnings; header 552/600, ladder 7 rungs,
  refresh 0/5) on linux-pc]. The count drops by one at a close, as M0-TM2–TM6 saw.
- **`plan_lint`, re-verified scoped** after the close (§4 Rule 4 step 7), `python3 tools/pb/verify.py plan_lint --task
  M0-TM7`: [ALREADY RUN — PASS (GO, 557 passed, 0 failed, 1/1 scopes, 0.12 s, load 10.01) on linux-pc].
- **Files this task wrote**:
  - the plan: the `Models:` line, the register's `Model ratings:` line, and its own Status, Handoff and Flow row;
  - this file and logs/M0-TM7.log.
  - verify.py wrote logs/M0-TM7.plan_lint.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
  - Only this section was written after the claim run. It is a skip-class path (tasks), graded by no scope.
