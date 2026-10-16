# M0-TM8 — the rating pass (2026-10-16, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt right after
the heading's grep). That is the heading's rung, so no mismatch. Box: linux-pc (laserax-ai, Linux 7.0.0-38-generic). Raw
runs: logs/M0-TM8.log, plus the logs verify.py writes. Read-only helpers sit in the session scratchpad
(`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/9b0ad6df-44ea-42d0-805e-407b65c127bc/scratchpad/tm8/`). They
are never in the repo and never write the plan:
- blocks.py: each block's heading, type, rating, switch, status, `Sizing:` and flag stamps; `diff <old>` compares the open
  blocks against an older plan, `full` prints a block;
- checkdiff.py: the diff against a snapshot, each changed line classed rating · models · register · own · other (GO when
  other = 0). Red-tested on a mutant (a rating changed, a ` · switch` dropped, a `Read:` line edited): rating=1, other=2,
  NO-GO, exit 1. Its control (the `Models:` line alone): models=1, GO;
- scan.py (rule (1)'s trail words, the count by rung) and newflags.py (a block's flags appended since an older plan).

## What fired it
M0-T61 (try 2) filed it when `plan_lint` read the refresh at 7 of 5 (Rules: `Rating refresh: 5 flags`), M0-T61's own try-1
trail among the seven (tasks/M0-T61.md § Try 2). That trail was retired at M0-T61's close. Try 2's flags to M0-T63 and M0-T64
were written after the filing, and M0-T62's to M0-T64 after that, so the lint read **9 of 5** at this block's start. These are
the carried flags on open BUILD blocks stamped after `Model ratings: 2026-10-15 by M0-TM7`:
- M0-T63 `[M0-T60]`: `headless --until` frames at 16 steps (headless::FRAME, untuned), and the per-frame summary costs fill_world
  2.5 s / 3.1 s / 4.6 s over 800 steps at 256 / 16 / 4 on the RTX 5090; stages need 3 frames to show.
- M0-T63 `[M0-T61]` × 2: the nominal cloud is `Scene::preset_mass("sun", UNIT_MASS, world)`, scaled so its cells hold M₀
  exactly. Every Σ is ×0.9998163 against numbers measured before (centre cell 0.99966), so baselines need a re-read.
- M0-T64 `[M0-T60]`: readouts.md's content — summary.json's events and objects as built.
- M0-T64 `[M0-T61]` × 2: the scope is `preset_cloud`, not `preset-cloud`.
- M0-T64 `[M0-T62]`: testing.md has the `jeans` row, and the pages' G-JEANS line reads U_th = 0.05|W| (contract §0.5 [M0-T62]).
- M0-T74 `[M0-T61]`: `Object::Preset` carries mass. A preset takes its calibrated mass through `Scene::preset_mass`, with
  `cloud_shape(mass)` and MAX_RADIUS 150.
- M0-T106 `[M0-T57]`: the ledger starts after a run's first step, and momentum holds to 1e-6 only on a symmetric collapse:
  - a drifting cloud reads 3e-3 of Σm|u| with gravity (source-term gravity) and 2.9e-4 without it (a vacuum reset's momentum,
    booked nowhere, §2.7 [M0-T19]);
  - the energy invariant drifted 0.4–0.9 of its scale on pressureless collapses, cause not measured;
  - the sinks' mφ is not in W.

Also read, though the lint does not count them (flags on gates; rule (2) pins those ratings):
- M0-V8 `[M0-T62]`: G-JEANS is graded at U_th = 0.05|W| with the cold pressure and heat off. The preset's own collapse is
  G-END's and G-STAGES's.
- M0-TZ `[M3-TB1, 2026-10-16]`: from `../space_tykun`, on the lead's word, this game's slot on the shared website
  (`https://tykungames.com/sandbox-reactions/`).

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-301_agent`.
  - Claude Code moved from 2.1.300, the version the `Models:` line cited, to 2.1.301.
  - `CLAUDE_EFFORT` is the process's launch level, a hint. The switch ran this session at max (`now`).
- **The model pages: read.** The Skill tool loaded the claude-api skill from 2.1.301's bundle. Its "Current Models (cached:
  2026-10-06)" table:
  - Prices per MTok: Fable 5.1 $10 / $50 · Opus 5.5 $4 / $20 · Sonnet 5.5 $2 / $10 · Haiku 5.5 $0.10 / $0.50 for prompts
    ≤ 100K tokens ($0.50 / $2.50 beyond).
  - Effort levels: low…max on all four. The default is medium for Opus 5.5 and Haiku 5.5, high for Sonnet 5.5.
- **shared/models.md.** In 2.1.301's bundle (`/tmp/claude-1000/bundled-skills/2.1.301/…`) it hashes (LF) to sha256
  c1af1f093d46fd58e50ef3e8e28d339650f333059bfc49a63d01d0175675161b, the c1af1f09…5675161b that M0-TM7 and M0-TM6 read. It is
  byte for byte the same page.
  - The other bundles are no longer on disk, so there was no file to diff against; the hash stands in for the diff.
  - All four ladder models are `Active`, and no model was added to the page.
- **The result.** Nothing was retired, renamed, added, repriced or relevelled, and every rung's level is one its model takes.
  No rung the lead runs changed, so no question was owed (§0).
- **The `Models:` line**, re-dated and re-cited in M0-TM7's form:
  - Before: "read 2026-10-15 from the claude-api skill's model table (Claude Code 2.1.300, cached 2026-10-06), the probe and
    the lead's answer (…)".
  - After: "read 2026-10-16 from the claude-api skill's model table (Claude Code 2.1.301, cached 2026-10-06), the probe and
    the lead's answer (…)".
  - The rungs and marks are byte-identical.
- **Not read: the live pricing pages** (the skill's shared/live-sources.md). The ladder cites the skill's table, cached
  2026-10-06. That list prices have not moved since is NOT PROVEN (source: the bundle's cache). No plausible move changes a
  rating:
  - rule (4)'s session-cost branch reads $1.98 against $6.50;
  - its list-price branch moves the class only if Sonnet 5.5's output price exceeds 1.21 × Opus 5.5's (36/49 = 0.735 against
    31/51 = 0.608).

## The record
- **The binding record** (§0: the phase-closing V's table) is still M0-V7's: reports/v7.md § Phase 9's close (linux-pc,
  2026-10-15). No V has closed a phase since.
  - BUILD on Sonnet 5.5, high: 41 blocks, 30/39 first try, $2.02 per DONE block.
  - BUILD on Opus 5.5, high (usual): 53 blocks, 31/49, $6.26.
  - Rule (4): ≥ 10 blocks below (usual), at a cost below (usual)'s → **the class does not move**. Every other class below
    (usual) has n < 10. M0-V7: "No TM".
- **Today's read on linux-pc** (`python3 tools/pb/rung_record.py report`, logs/M0-TM8.log): 159 DONE blocks, 12 class/rung
  rows, 68 blocks with no session file readable here, 4 D blocks naming no cause, $401.66 read from the sessions' own cost.
  - BUILD on Sonnet 5.5, high: 51 blocks, 36/49, $1.98. BUILD on Opus 5.5, high: 55 blocks, 31/51, $6.50. They agree: no move.
  - The escalations since M0-TM7 are in these rows: M0-D47 (DONE on Opus 5.5, high, its own trail) and M0-T61 (try 1 red on
    Sonnet 5.5, high, try 2 DONE on (usual)).
  - BUILD·E on Haiku 5.5, high: 8 blocks, 6/8, $1.06. BUILD·E on Sonnet 5.5, medium: 2. MOVE on Haiku 5.5, high: 3, $0.17.
    MOVE on Sonnet 5.5, medium: 1. Each has n < 10.
- **Rule (1)**: no open block's `Carried flags:` holds an escalation trail. scan.py searched the 75 open blocks for the trail's
  words (escalat, trail, NO-GO claim, claim is red, back to TODO, rated (usual), failed attempt, try 1/2, second attempt). The
  only hits are this block's own text and a test file named `trail ` in M0-TZ's flags.

## The rating: every open block, 75 of them
**Method.**
1. blocks.py diffed every open block against the plan at 60dc024, 27bc0aa and a2bed96 — the commits on either side of
   M0-TM7's pass. M0-TM7 rated a2bed96 plus its working tree; 27bc0aa holds M0-TM7's close.
   - 27bc0aa against a2bed96: 81 open blocks, all byte-identical.
   - 60dc024 against 27bc0aa: 80, all byte-identical.
   - Now against 60dc024:
     - new: this block;
     - changed: M0-T63, T64, V8, T74, T106 and TZ, each in `Carried flags:` alone (appended);
     - the other 68 open blocks are byte-identical to that text, headings included.
2. The ladder is unchanged and the record moves no class, so an unchanged block keeps M0-TM7's derivation (tasks/M0-TM7.md §
   The rating). That derivation is M0-TM4's kinds (tasks/M0-TM4.md § The rating), re-checked here by type, `Sizing:` and flags.
   - Read in full: the six changed blocks — heading, `Sizing:`, `Carried flags:`, `Deliver:`, `Verify:` and `Pass:`.
3. §0's rubric, first match wins. The closed list is read as M0-TB set it (reports/size_pass.md §9) and M0-TM1–TM7 kept it:
   (a) a FROZEN passage, a schema, a persisted format or a migration changed · (b) a D whose cause is not measured · (c) a
   design the spec leaves open · (d) secrets, money or data loss · (e) numeric or precision semantics, or concurrency, as the
   block's own acceptance · (f) a library's internals. A block that reuses a mechanism an earlier block built does not earn
   the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 11 | V8–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM8, TJ1*, TJ2*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 0 | — (M0-T56, the last one, is DONE) | 0 |
| Sonnet 5.5, high | (3), no reason from the list | 56 | below | 0 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung | 4 | T91, T93a, T94, T111 | 0 |
| Fable 5.1, high · Opus 5.5, xhigh · Sonnet 5.5, medium | — | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`
(75 of 75).

The 56 on Sonnet 5.5, high: T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82,
T83, T84, T85, T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107,
T98b, T99b, T100, T101, T108, T109, T110, T112, T113, T114*, T115, TD, T116. These are M0-TM7's 64 less the eight now DONE
(D47, D48, T57, T58, T59, T60, T61, T62). Their kinds, each read against the list, are M0-TM4's.

### The blocks above the cheapest rung that fits, with their rule
- **BUILD: none.** The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. M0-T56, the
  one open BUILD block on (usual) at M0-TM7 (by (e)), is DONE.
- **The 15 gates on Opus 5.5, max (gate), by rule (2), by type:**
  - CHECK: M0-V8, V9, V10, V11, V12, V13, V14, V15, V16, V17 and M0-TW (the CHECK scribe of `LEAD walk + CHECK`).
  - PLAN: M0-TM8, M0-TZ, and M0-TJ1* and M0-TJ2* (`PLAN + LEAD answers`, DEFERRED).

### The E blocks: Haiku 5.5, high (rule (3))
M0-T91, T93a, T94 and T111 are byte-identical to the text M0-TM7 read, and each still carries `Sizing: E`. M0-TM4's readings
stand: M0-T93a is the closest call, and its asynchronous one-cell readback reuses M0-T22's `poll()` and M0-T56's ≤ 2-frames-late
readback. BUILD·E on Haiku 5.5, high has 8 blocks in the record (n < 10), so rule (4) moves nothing.

### The changed blocks, re-derived (every rating unchanged)
- **M0-T63** `[M0-T60]`, `[M0-T61]` × 2: **Sonnet 5.5, high**, rule (3).
  - M0-TM4's kind: measurement reports. The flags are measured inputs to the probe: the `--until` frame cadence and its cost,
    and the preset's exact mass rescaling every Σ by 0.9998163.
  - Timing is never a reason. The Σ scale is a number M0-T61's built mechanism sets, which the report re-reads. It is not
    precision semantics the block's own acceptance grades: its Verify is four report sections, each number with its command
    and adapter.
  - No reason.
- **M0-T64** `[M0-T60]`, `[M0-T61]` × 2, `[M0-T62]`: **Sonnet 5.5, high**, rule (3).
  - A docs block. The flags give the content: summary.json's events and objects as M0-T58–T60 built them, the scope's name,
    and G-JEANS at U_th = 0.05|W|.
  - Documenting a schema changes none, so no (a). A scope's name is a fixed spec's wording (R17), which the list excludes.
  - No reason.
- **M0-T74** `[M0-T61]`: **Sonnet 5.5, high**, rule (3).
  - M0-TM4's kind: edits and formats. The flag names the interface M0-T61 built (`Object::Preset`'s mass,
    `Scene::preset_mass`, `cloud_shape`, MAX_RADIUS 150).
  - Reusing a built mechanism earns no (c).
  - No reason.
- **M0-T106** `[M0-T57]`: **Sonnet 5.5, high**, rule (3). M0-TM4's kind: a whole-life test whose UNVERIFIED red is a D or the
  lead's question. Weighed against the list:
  - (b): M0-T106 is a T, not a D. The energy drift's unmeasured cause belongs to the D its own Fail line files ("more drift is a
    D for a conservative gravity formulation, never a looser bound"), and that D's filer rates it by (b).
  - (e): the bands are contract §5.1's, fixed. The sums are M0-T57's ledger, built on M0-T56's fixed-order partials. Both are
    reused, so neither is the block's own semantics.
  - (c): the flag's "G-CONS's energy band needs a scene that resolves its collapse" is guidance on scenes the `Deliver:`
    already fixes: a whole Sun-like life in bounce, and a supernova in leave from the Massive preset's iron-core dump.
  - No reason. See § Considered.
- **M0-V8** `[M0-T62]` (CHECK) and **M0-TZ** `[M3-TB1]` (PLAN): rule (2) pins the gate.
- **M0-TM8**: PLAN, rule (2).

### Considered — observations, never defects (§0, no step up on a guess)
- **M0-T106's claim run is measured red-prone on the shipped scheme.** At M0-TM7 there was one measurement; there are three
  now:
  - momentum: 3.9e-3 of Σm|u| in 40 steps from source-term gravity (M0-V7 § Observations 1);
  - momentum: 3e-3 on a drifting cloud with gravity, and 2.9e-4 without it, a vacuum reset's momentum booked nowhere (§2.7
    [M0-T19]; M0-T57);
  - energy: the invariant drifted 0.4–0.9 of its scale on pressureless collapses, cause not measured (M0-T57).

  On Sonnet 5.5, high a red claim ends the try and rates the block (usual) by rule (1), yet no rung makes the scheme conserve.
  The route is the lead's ruling, or the D its Fail line names (§0.4, §5.1: a conservative gravity formulation, never a looser
  bound). A rung prices a builder, not a contract ruling, so there is no step up. The lead may want to rule on G-CONS's momentum
  clause, and on the vacuum reset's unbooked momentum, before lot 15 reaches M0-T106.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and the
  rating is kept.

## What changed in the plan
- **Ratings: 0 changed.** By rung, now (changed):
  - Opus 5.5, max: 15 (0).
  - Opus 5.5, high: 0 (0; M0-T56 DONE).
  - Sonnet 5.5, high: 56 (0).
  - Haiku 5.5, high: 4 (0).
  - Fable 5.1, high, Opus 5.5, xhigh and Sonnet 5.5, medium: 0 (0).
- **The `Models:` line**: re-dated and re-cited (above).
- **The register**: `Model ratings: 2026-10-16 by M0-TM8`, written through `plan.py register --set` before the claim run, so
  the claim covers it (§4 Rule 4: step 4, the structural edits, then step 5, the claim). The lint's refresh went from 9/5 to 0/5.
- **How the `Models:` line was edited.** plan.py has no call that rewrites the `Models:` line or a rating. So, as M0-TM2–TM7
  did, I used the file-edit tool, one edit. `old_string` was the line's tail, grep-counted to 1 first.
- **The diff**, checked by checkdiff.py before the close:
  - against the snapshot taken right after `plan.py status`: 0 rating segments, 1 `Models:` line, 1 register line, 0 other;
  - against the start's snapshot: the same, plus 2 own lines (this block's Status and its Flow row).

## Measured and not acted on
This block's diff is bounded to rating segments, the `Models:` line and the register's line (its `Pass:` and `Fail:`), so
nothing below is filed or flagged from here (M0-TM5–TM7's precedent).
- **M0-T106's G-CONS clauses**: see § Considered. M0-V7 and M0-T57 already flagged them into M0-T106. They are for the lead.
- **M0-T64's Verify 2 regex** names `observe-run` and `preset-cloud`, while verify.json's scopes are `observe_run` and
  `preset_cloud`.
  - The Superseded entry for contract §0.5 [M0-V2] and R17 reads every block's hyphenated scope name with `_`.
  - M0-T61's flags say so for `preset_cloud`; none names `observe_run`.
- **The Pipeline state's stale lines**:
  - `Next task: M0-TM8` goes stale at this close, and the close does not set it. M0-T63 runs next.
  - `Open D/BLOCKED register: M0-D48 … — TODO, after M0-TM7`: M0-D48 is DONE (committed in 27bc0aa), and no D is open (D47,
    D48 and D49 are DONE).
  - `Outstanding commit gates` reads Phase 9's as uncommitted, but git log shows a2bed96, "M0 Phase 9: black holes and
    neutrino heating" (2026-10-15 13:57). The push is not read here.
- **The lint's six WARNs**: M0-TC6, D45, D46, TM7, D47 and D48 are DONE outside the current phase and want stubs (§2.6). Stubbing
  them is a compaction's job, or the next V's close.
- **Repo facts' agent line** names Claude Code 2.1.292 at the 2026-10-08 probe; today's probe reads 2.1.301. It is a dated
  probe, outside this diff.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the process's launch level, which the
  switch overrides (§B.3).
- **The box's load** read 43.16 at the claim run (32 threads): another job was on the box. The claim is 0.12 s, so its verdict
  does not depend on the load.

## The routine, on linux-pc (laserax-ai)
- **`python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**:
  - after the `Models:` line: [ALREADY RUN — PASS (GO, 553 checks, 0 failed, 6 warnings — the stubs above; header 558/600,
    ladder 7 rungs, refresh 9/5) on linux-pc];
  - after the register: [ALREADY RUN — PASS (GO, 553 checks, 0 failed, 6 warnings, refresh 0/5) on linux-pc].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (0 rating segments, 1 `Models:` line, 1 register
  line, 0 other) on linux-pc]. On the mutant: [ALREADY RUN — PASS (rating=1, other=2, NO-GO, exit 1 — the red it must show)
  on linux-pc].
- **No red-arm.** This task added no scope and changed no scope's check (R9; PLAYBOOK §4 Rule 4, v12.3).
- **The claim run**, after the structural edits: `python3 tools/pb/verify.py --changed --task M0-TM8`. `plan.py show` printed
  a `baseline:` line, so no `--base` is named (R10). [ALREADY RUN — PASS (GO: plan_lint 553 passed, 0 failed; 1/1 scopes GO,
  51 not impacted, 0 never checked, 0 stale, 0 unclassified, 0 owed; 0.12 s; base M0-TM8's start 2026-10-16T10:48:13-04:00;
  tree 60dc024b5d2e+a700e9c00625; python 3.12.3; load 43.16) on linux-pc].
- **The close**: `python3 tools/pb/plan.py close M0-TM8 --status "DONE" --text "<handoff>"`, written from the claim run.
  Then checkdiff.py and the lint run again, and `plan_lint` is re-verified scoped, since the close's edit lands after the claim
  (§4 Rule 4 step 7).
- The results are below, appended after the runs.

### Results, appended after the runs
- **The close** (10:58): `close: checks=3 failed=0 warnings=0 · handoff_lines=1 · flow=per-task`, GO. Status `DONE
  (2026-10-16 10:58, started 10:49)`. The close does not set the register's `Next task:`, which still reads M0-TM8, left as
  found; M0-T63 runs next.
- **checkdiff.py** after the close, against the post-status snapshot and the start's: [ALREADY RUN — PASS (0 rating segments,
  1 `Models:` line, 3 own lines — Status, Handoff, Flow row — 1 register line `Model ratings:`, 0 other) on linux-pc].
- **The lint** after the close: [ALREADY RUN — PASS (GO, 552 checks, 0 failed, 6 warnings — the stubs above; header 558/600,
  ladder 7 rungs, refresh 0/5) on linux-pc]. The count drops by one at a close, as M0-TM2–TM7 saw.
- **`plan_lint`, re-verified scoped** after the close (§4 Rule 4 step 7), `python3 tools/pb/verify.py plan_lint --task
  M0-TM8`: [ALREADY RUN — PASS (GO, 552 passed, 0 failed, 1/1 scopes, 0.12 s, load 31.05) on linux-pc].
- **Files this task wrote**:
  - the plan: the `Models:` line, the register's `Model ratings:` line, and its own Status, Handoff and Flow row;
  - this file and logs/M0-TM8.log.
  - verify.py wrote logs/M0-TM8.plan_lint.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
  - Only this section was written after the claim run. It is a skip-class path (tasks), graded by no scope.
