# M0-TM6 — the rating pass (2026-10-14, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt right after
the heading's grep), the heading's rung, no mismatch. Box: linux-pc (laserax-ai). Raw runs: logs/M0-TM6.log, plus the logs
verify.py writes. Read-only helpers, in the session scratchpad's tm6/ (never in the repo, never run on the plan's behalf):
blocks.py (each block's heading, rating, `Sizing:` and flag stamps; `--diff` against an older plan, `--full` prints a block)
and checkdiff.py (the diff against a snapshot, each changed line classed rating · models · own · register · other;
red-tested on a mutated copy — a rating changed, a ` · switch` dropped, a `Read:` line edited: rating=1, other=2, NO-GO,
exit 1).

## What fired it
M0-T43's `plan.py flag` read refresh 6/5: six carried flags stamped 2026-10-14 on open BUILD blocks since `Model ratings:
2026-10-13 by M0-TM5`.
- `[M0-T42]` → M0-T48 (the GPU's P5 caps a face past 32 stages; the twin has no cap), M0-T49 (lot 6's heat rows), M0-T63
  (P5's cost on the Quadro).
- `[M0-T43]` → M0-T48 (the twin runs no P5 and passes no F), M0-T49 (the radiation_force row, physics.md's layout), M0-T63
  (P4's extra reads when P5 runs).
- Stamped 2026-10-13 but filed after M0-TM5 ran (09:19–09:40), so not counted by the lint (the pass's own day), and read here
  all the same: M0-T48 (`[M0-T38]`, `[M0-V5]`, `[M0-T41]`), M0-T49 (`[M0-T41]`), M0-T63 (`[M0-V5]`, `[M0-D30]`), M0-T106
  (`[M0-T35]`); on gates, M0-V6 (`[M0-V5]`, `[M0-D32]`) and M0-TZ (`[M0-D32]`).

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-296_agent`.
  - Still Claude Code 2.1.296, the version the ladder cites.
  - `CLAUDE_EFFORT` is the process's launch level, a hint. The switch ran this session at max (`now`).
- **The model pages: read.** The Skill tool loaded the claude-api skill this session (auto mode refused it at M0-TM5, not
  today). Its "Current Models (cached: 2026-10-06)" table:
  - Fable 5.1: $10 / $50 per MTok. Opus 5.5: $4 / $20. Sonnet 5.5: $2 / $10. Haiku 5.5: $0.10 / $0.50 for prompts ≤ 100K
    tokens ($0.50 / $2.50 beyond).
  - Effort levels: low…max on all four. The default is medium for Opus 5.5 and Haiku 5.5, high for Sonnet 5.5.
- **shared/models.md, the file M0-TM2–TM4 read.** Hashed as extracted on linux-pc (LF), sha256 c1af1f09…5675161b. With CRLF
  line endings (win-laptop's extraction, where M0-TM4 hashed it), 1601794b74c4a9415eb272d5ad459ab2776f224d731285c4b492a4c5c6f9a2ec:
  M0-TM4's 1601794b74c4…c6f9a2ec. So the page is byte for byte the one M0-TM4 read, up to line endings, and M0-TM5's NOT PROVEN
  on that point is now proven. All four ladder models are `Active`; none is deprecated, retired or renamed.
- **The result.** Nothing was retired, renamed, added, repriced or relevelled, and every rung's level is one its model takes.
  No question to the lead: no rung they run changed (§0).
- **The `Models:` line**, re-dated and re-cited to the page (the form M0-TM2 and M0-TM4 used).
  - Before: "read 2026-10-13 from the probe (Claude Code 2.1.296: the claude-api skill's model table M0-TM4 read 2026-10-12,
    cached 2026-10-06; its load refused by auto mode at M0-TM5) and the lead's answer (…; 2026-10-13, M0-TM5: «Re-date on the
    probe»)".
  - After: "read 2026-10-14 from the claude-api skill's model table (Claude Code 2.1.296, cached 2026-10-06), the probe and the
    lead's answer (…)".
  - The rungs and marks are byte-identical. The lead's «Re-date on the probe» answered 2026-10-13's refused read. Today's
    date rests on the page, so that clause left the line; the lead's words stay verbatim in tasks/M0-TM5.md.
- **Not read: the live pricing pages** (the skill's shared/live-sources.md). The ladder cites the skill's table, cached
  2026-10-06. That list prices have not moved since then is NOT PROVEN (source: the bundle's cache).

## The record
- **The binding record** (§0: the phase-closing V's table) is M0-V5's: reports/v5.md § Phase 7's close (linux-pc,
  2026-10-13).
  - BUILD on Sonnet 5.5, high: 14 blocks, 8/14 first try, $1.68 per DONE block.
  - BUILD on Opus 5.5, high (usual): 14 blocks, 7/10, $4.03.
  - Rule (4): ≥ 10 blocks below (usual), at a cost not above (usual)'s → **the class does not move**. BUILD·E on Haiku 5.5,
    high has 1 block (n < 10). A MOVE's class is its type alone. No class moves (M0-V5: "No TM").
- **Today's read on linux-pc** (`python3 tools/pb/rung_record.py report`, after M0-D32's fix): 108 DONE blocks, 12 class/rung
  rows, 65 blocks with no session file readable here, $218.20 read from this box's sessions.
  - BUILD on Sonnet 5.5, high: 28 blocks, 20/27, $1.68. BUILD on Opus 5.5, high: 39 blocks, 22/35, $4.66. It agrees: no move.
  - BUILD·E on Haiku 5.5, high: 3 blocks, 2/3, $2.95. BUILD·E on Sonnet 5.5, medium: 2 blocks, $0.34. Both n < 10, so rule (4)
    reads nothing there.
- **Rule (1)**: no open block's `Carried flags:` holds an escalation trail. The one trail in the plan sits on M0-T41, which is
  DONE.

## The rating: every open block, 99 of them
**Method.**
1. blocks.py diffed every open block against the plan at 19e0d8e (M0-T35's commit, after M0-TM5's close) and at cf0f745
   (the commit M0-TM5's tree stood on).
   - Changed since M0-TM5's read: M0-T48, T49 and T63 (the six flags that fired, plus ones stamped 2026-10-13), M0-T106
     (`[M0-T35]`), M0-V6 and M0-TZ (flags on a CHECK and a PLAN block).
   - New: M0-D33 (filed by M0-T43) and this block.
   - The other 91 open blocks are byte-identical, headings included, to the text M0-TM5 re-checked and M0-TM4 read in full.
2. The ladder is unchanged and the record moves no class. So an unchanged block's derivation is M0-TM4's (tasks/M0-TM4.md §
   The rating) as M0-TM5 carried it, re-checked here by type, `Sizing:` and flags.
   - Read in full: the changed and new blocks, and the four on (usual), which are M0-T46, T51, T56 and D33. For each, the
     heading, `Sizing:`, `Carried flags:`, `Deliver:`, `Verify:` and `Pass:`.
   - M0-T44a … T47 were read in full as well, as context for M0-T48.
3. §0's rubric, first match wins. The closed list is read as M0-TB set it (reports/size_pass.md §9) and M0-TM1–TM5 kept it:
   - (a) a FROZEN passage, a schema, a persisted format or a migration changed.
   - (b) a D whose cause is not measured.
   - (c) a design the spec leaves open: the contract leaves a structure to the builder, and later blocks build on it.
   - (d) secrets, money or data loss.
   - (e) numeric or precision semantics, or concurrency, as the block's own acceptance.
   - (f) a library's internals.

   A block that reuses a mechanism an earlier block built does not earn the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 13 | V6–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM6, TJ1*, TJ2*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 4 | T46, T51, T56, D33 | 0 |
| Sonnet 5.5, high | (3), no reason from the list | 72 | below | 0 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung | 6 | T48, T52b, T91, T93a, T94, T111 | 0 |
| Fable 5.1, high · Opus 5.5, xhigh · Sonnet 5.5, medium | — | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`
(99 of 99).

The 72 on Sonnet 5.5, high: T44a, T44b, T45, T47, T49, T50, T52a, T53, T54, T55, T57, T58, T59, T60, T61, T62, T63, T64, T65,
T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, TV2, T89,
T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107, T98b, T99b, T100, T101, T108, T109, T110,
T112, T113, T114*, T115, TD, T116. These are M0-TM5's 76, less M0-T37, T38, T40 and T41, which are now DONE. Their kinds, each
read against the list, are M0-TM4's.

### The 4 BUILD blocks above the cheapest rung that fits, each with its rule
The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. These 4 sit one rung above it,
on (usual), by rule (5): a reason from (3)'s list, re-derived from the block's own text.

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T46 | (e) | "M0-T44b's sub-cycles per cell in f32", f(T) within 10⁻⁴; Adversarial: "a per-cell loop of up to hundreds of sub-cycles diverging across a workgroup". Stiff sub-cycles in f32 (unchanged) |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)"; Adversarial: "two sinks reducing into each other's accretion in parallel take the same cell twice". Bit-identical results from parallel code (unchanged) |
| M0-T56 | (e) | each §1.9.1 field "within the f32 rounding it states per field", "read back asynchronously (≤ 2 frames late)". An asynchronous readback (unchanged) |
| M0-D33 | (a), (b) | `Ruling:` "the dump appends F's two f32 planes … after §2.2's 14 channels, flagged in `resume` …; a header whose flag and plane count disagree is refused", declared as contract §0.5 [M0-D33]: §2.12.3's dump, a persisted format, changed. Its "Suspected cause:" is read from the code, not measured. Filed by M0-T43 on (usual); re-derived the same |

### The changed blocks, re-derived (every rating unchanged)
- **M0-T48**: Haiku 5.5, high, rule (3), `Sizing: E`. Its new flags:
  - `[M0-T38]`: grade() takes a scene file, and heat and reactions run on both sides.
  - `[M0-V5]`: G-REF holds under the shipped EOS, so no cold-pressure override is needed. §0.5 [M0-V5] sets the cell set.
  - `[M0-T41]`: the twin's driver runs no P5, and no block's `Deliver:` adds it. The heat switch, P5 between P4 and P8, its
    pass label and `Heat::step` fall to whoever builds the burning field.
  - `[M0-T42]`: the GPU caps a face past 32 stages and the twin does not. Either cap the twin the same way, or pick a scene
    whose s stays ≤ 32.
  - `[M0-T43]`: the twin passes no F. Once it gains P5, the twin calls `step_forced(…, Some(F_prev))` and keeps Heat::step's
    flux for the next step.

  Read against the list:
  - No (a): a new scene in §2.12.1's format, and no schema changes.
  - No (c): every step is named (§1.3.2's order, `Heat::step`, `step_forced`, §0.5 [M0-T42]'s cap or a scene inside it), and
    the driver is M0-T37's structure, extended.
  - No (e): this is G-REF's third case, graded on mechanisms that M0-T42, T43, T44b and T46 build. M0-TM4 read M0-T38's two
    cases the same way, and the f32 sub-steps are already M0-T42's and M0-T46's own (e).

  No reason holds, so rule (3) and `Sizing: E` put it on the ladder's lowest rung. Its size: § Considered.
- **M0-T49**: Sonnet 5.5, high. The flags are row counts, timings and layout notes for docs pages. No reason.
- **M0-T63**: Sonnet 5.5, high. The flags carry costs measured on the Quadro (M0-V5, M0-D30, M0-T42) and one not measured
  (P4's F reads, M0-T43). It is a timing report, and the list excludes a timing choice. No reason.
- **M0-T106**: Sonnet 5.5, high. `[M0-T35]`: bounce books no escaped momentum. The block's own text is unchanged, a whole-life
  test whose UNVERIFIED red is a D or a question (M0-TM4's kind). No reason. Its momentum clause: § Considered.
- **M0-V6** (`[M0-V5]`, `[M0-D32]`) and **M0-TZ** (`[M0-D32]`): a CHECK and a PLAN block, so rule (2) pins the gate.
- **M0-D33**: above. **M0-TM6**: PLAN, rule (2).

### Considered — observations, never defects (§0, no step up on a guess)
- **M0-T48 has outgrown an edit job.**
  - §0's E is "≤ 2 files, ≤ 150 new lines · no new module, dependency, schema or string · verified by an existing scope".
  - Its `Deliver:` names three files: cpu_gpu.rs, a new scenes/ref_burn.json and verify.json's count. Its flags add the twin's
    driver: reference/mod.rs gets a heat switch, P5 with F carried, P5's pass label, and P6 too, since "heat and reactions on"
    burns on the twin. M0-T44b builds reference/react.rs, and no open block's `Deliver:` wires it, or P5, into the driver (a
    grep of the plan for `Switches`, `reference::Twin`, the twin's driver and reference/mod.rs). One of `[M0-T42]`'s two routes
    adds reference/heat.rs's cap as well.
  - Size is §2.1's: "a block that looks too big for its rung is split, never stepped up". So the rating stays.
  - There are two remedies, and both are PLAN edits this block may not write (its Fail: "any other byte changed"):
    - a split: the twin driver's P5, F and P6 as their own block before M0-T48, leaving M0-T48 the E case it was;
    - or `Sizing: E` removed, M0-TJ3's precedent with M0-T6 (Rules' E line). Rule (3) would then put it on Sonnet 5.5, high:
      no reason from the list holds.
  - That choice is for the lead and the next PLAN block. Until then, rule (1) prices a miss: a NO-GO claim on Haiku 5.5, high
    rates it (usual) and sends it back to TODO.
- **M0-T106's momentum bound in bounce.** Source only, not measured.
  - Its `Deliver:` bounds momentum to 10⁻⁶ of Σm|u| over a whole Sun-like life in bounce. §5.1's G-CONS reads "momentum (grid
    + sinks + escaped) to 10⁻⁶ of Σm|u| … Mass and momentum *by construction*".
  - `[M0-T35]`: "Bounce books no escaped: the walls' pressure impulse changes the grid's momentum unbooked".
  - Contract §2.7 (P8's floors) reads "G-CONS grades mass and energy only".
  - M0-V5's observation 4 (the Sun-like preset's rim reaches the edges within 2,000 steps) makes a wall contact likely in a
    whole life.
  - §5.1 is not FROZEN, so (a) does not hold. Whether G-CONS grades momentum in bounce is a contract question: its builder
    asks («Wrong or ambiguous → ask»), or a PLAN block settles it first.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and the
  rating is kept.

## What changed in the plan
- **Ratings: 0 changed.** By rung, now (changed):
  - Opus 5.5, max: 17 (0).
  - Opus 5.5, high: 4 (0).
  - Sonnet 5.5, high: 72 (0).
  - Haiku 5.5, high: 6 (0).
  - Fable 5.1, high, Opus 5.5, xhigh and Sonnet 5.5, medium: 0 (0).
- **The `Models:` line**: re-dated and re-cited (above).
- **The register**: `Model ratings: 2026-10-14 by M0-TM6`, written through `plan.py close --set`.
- **How the edit was made.** plan.py has no call that rewrites the `Models:` line or a rating. So, as M0-TM2–TM5 did, I used
  the file-edit tool, one edit; `old_string` was the line's tail, the line grep-counted to 1 first.
- **The diff**, checked by checkdiff.py against a snapshot taken right after `plan.py status`: 0 rating segments, 1 `Models:`
  line, 0 other lines, before the close.

## Measured and not acted on
This block's diff is bounded to rating segments, the `Models:` line and the register's line (its `Pass:` and `Fail:`), so
nothing below is filed or flagged from here (M0-TM5's precedent).
- **M0-T48's size and M0-T106's momentum clause**: § Considered. For the lead and the next PLAN block.
- **lint's 4 WARNs**: M0-TC4, D30, D31 and D32 are DONE outside the current phase and wait to be stubbed (a TC's work, or the
  next V's).
- **The Pipeline state's `Next task: M0-TM6`** goes stale at this close; the next task is M0-D33 (AFTER M0-TM6). This diff is
  bounded, so the next close sets it (M0-TM1–TM5's precedent).
- **models.md's hash differs by line endings between the two boxes' extractions** (LF here, CRLF on win-laptop). A later pass
  comparing hashes across boxes normalises line endings first.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the process's launch level, which the
  switch overrides (§B.3).

## The routine, on linux-pc (laserax-ai)
- **`python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**, after the `Models:` line: [ALREADY RUN —
  PASS (GO, 523 checks, 0 failed, 4 warnings — the stubs above; header 491/600, ladder 7 rungs, refresh 6/5) on linux-pc].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (0 rating segments, 1 `Models:` line, 0 other) on
  linux-pc]; on the mutant: [ALREADY RUN — PASS (rating=1, other=2, NO-GO, exit 1 — the red it must show) on linux-pc].
- **No red-arm.** This task added no scope and changed no scope's check (R9; PLAYBOOK §4 Rule 4, v12.3).
- **The close**: `python3 tools/pb/plan.py close M0-TM6 --status "DONE" --text "<handoff>" --set "Model ratings=2026-10-14
  by M0-TM6"`, then checkdiff.py and the lint again.
- **The claim run**, the last step, after the close: `python3 tools/pb/verify.py --changed --task M0-TM6`. `plan.py show`
  printed a `baseline:` line, so no `--base` is named (R10).
- The results are below, appended after the runs.

### Results, appended after the runs
- **The close** (08:08): `close: checks=4 failed=0 warnings=0 · handoff_lines=1 · flow=per-task · set=1 · replaced=1 ·
  appended=0`, GO. Status `DONE (2026-10-14 08:08, started 08:01)`; the register reads `Model ratings: 2026-10-14 by M0-TM6`.
- **checkdiff.py** after the close: [ALREADY RUN — PASS (0 rating segments, 1 `Models:` line, 3 own lines — Status, Handoff,
  Flow row — 1 register line `Model ratings:`, 0 other) on linux-pc].
- **The lint** after the close: [ALREADY RUN — PASS (GO, 522 checks, 0 failed, 4 warnings — the stubs above; refresh 0/5)
  on linux-pc]. The count drops by one at a close, as M0-TM2–TM5 saw.
- **The claim run**, `python3 tools/pb/verify.py --changed --task M0-TM6`, the last step: [ALREADY RUN — FAIL (NO-GO:
  plan_lint 522/0 GO; dump 7 passed, 1 failed; 529 passed, 1 failed in all, 1/2 scopes GO, 36 not impacted, 3.02 s; base
  M0-TM6's start 2026-10-14T07:58:41-04:00; tree 4eb47e04d92c+1fd6c45ce99c; python 3.12.3; load 9.27→9.41; anchor 76 never
  checked, 0 stale, 0 unclassified) on linux-pc].
  - **The red is M0-D33's.** `dump::headless_two_runs_of_100_through_a_dump_write_the_200_step_runs_dumps_byte_for_byte`
    panics at tests/gpu/dump.rs:304, "s_150.bin: the resumed run's dump differs from the straight run's"
    (logs/M0-TM6.dump.log). That is the case and the line of M0-D33's `Symptom:`, reproduced.
  - **Why dump ran.** The run's reason was "never: .cargo/config.toml (+75)": since M0-T43's red, none of dump's 76 paths
    holds a green anchor entry. This task wrote none of them; it wrote the plan, this file and logs/M0-TM6*. The 76 "never
    checked" lines are those same paths.
  - **The register already says so:** "M0-D33 (TODO, after M0-TM6): … dump NO-GO in claims until it lands (Caused by:
    M0-T43)". M0-T43 closed DONE on the same red.
  - **This block stays DONE.** It is a PLAN block rated (gate), so the rule that a red claim below (usual) ends the try does
    not apply. Its `Pass:` holds: the lint is GO, every open block is on a rung of the ladder, and the diff is bounded. The
    claim stays NO-GO until M0-D33, the next task, lands.
- **Files this task wrote**:
  - the plan: the `Models:` line, its own Status, Handoff and Flow row, and the register's `Model ratings:` line;
  - this file and logs/M0-TM6.log.
  - verify.py wrote logs/M0-TM6.plan_lint.log, logs/M0-TM6.dump.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json
    ("1 path×scope entries moved by this run").
  - Only this section was written after the claim run. It is a skip-class path, graded by no scope.
