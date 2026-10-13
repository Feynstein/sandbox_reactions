# M0-TM5 — the rating pass (2026-10-13, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt right after
the heading's grep), the heading's rung, no mismatch. Box: linux-pc (laserax-ai). Raw runs: logs/M0-TM5.log, plus the logs
verify.py writes. Read-only helpers, in the session scratchpad's tm5/ (never in the repo, never run on the plan's behalf):
blocks.py (each open block's heading, rating, `Sizing:` and flag stamps, diffed against an older plan; `--full` prints a
block) and checkdiff.py (the diff against a snapshot, each changed line classed rating · models · own · register · other;
red-tested on a mutated copy — a `Read:` line edited and a ` · switch` dropped: other=2, NO-GO).

## What fired it
`plan.py lint` at M0-T34's close read refresh 5/5: five carried flags stamped 2026-10-13 on open BUILD blocks since
`Model ratings: 2026-10-12 by M0-TM4`.
- `[M0-T33b]` → M0-T35 (the twin's `Edge` has `Bounce` only; Leave and the escaped booking go in `step()`'s match) and
  M0-T36 (the twin's Hancock predictor, where Σg and Σu·g go).
- `[M0-T34]` → M0-T35, M0-T36, M0-T43: `hydro_update` binds WebGPU's 8 storage buffers (state ×3, next ×3, the Δt
  record, the eos table); the escaped booking, g and F each need a slot.
- M0-T34's flag to M0-V5 (P4's cost on the Quadro) sits on a CHECK block and does not count.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-296_agent`.
  - Still Claude Code 2.1.296, the version the ladder cites.
  - `CLAUDE_EFFORT` is the picker's launch level, a hint. The switch ran this session at max (`now`).
- **The model pages: refused.** Claude Code's auto-mode classifier refused both routes to the claude-api skill's model
  table: a `find` over the bundled-skills folder ("Auto-Mode Bypass") and the Skill tool's load of `claude-api`
  ("Credential Exploration"). Neither was retried or worked around. The plan's first Hazard (2026-10-08) sends such a
  refusal to the lead as a question.
- **The lead's answer** (the question tool, 2026-10-13): «Re-date on the probe (Recommended)». Claude Code 2.1.296 bundles
  the table M0-TM4 read on 2026-10-12 (cached 2026-10-06): Fable 5.1 $10 / $50, Opus 5.5 $4 / $20, Sonnet 5.5 $2 / $10,
  Haiku 5.5 $0.10 / $0.50 per MTok (prompts ≤ 100K tokens); effort low…max on all four; all Active (tasks/M0-TM4.md).
- **The result.** The `Models:` line is re-dated and re-cited. "read 2026-10-12 from the claude-api skill's model table
  (Claude Code 2.1.296, cached 2026-10-06), the probe and the lead's answer (…)" → "read 2026-10-13 from the probe (Claude
  Code 2.1.296: the claude-api skill's model table M0-TM4 read 2026-10-12, cached 2026-10-06; its load refused by auto mode
  at M0-TM5) and the lead's answer (…; 2026-10-13, M0-TM5: «Re-date on the probe»)". Its rungs and marks are byte-identical.
  No rung the lead runs changed, so there is no further question (§0).
- **NOT PROVEN (source only)**: that today's bundle matches the file M0-TM4 hashed (models.md, sha256 1601794b74c4…). The page
  was not read today; the probe's unchanged version and the lead's answer stand in for it.

## The record
- **The binding record** (§0: "the record is the phase-closing V's table"): M0-V4's, reports/v4.md § Phase 6's close
  (win-laptop, 2026-10-12).
  - BUILD on Sonnet 5.5, high: 16 blocks, 13/16 first try, $1.64 per DONE block.
  - BUILD on Opus 5.5, high (usual): 29 blocks, 22/29, $3.11.
  - Rule (4): ≥ 10 blocks below (usual), at a cost not above (usual)'s → **the class does not move**. BUILD·E on Haiku 5.5,
    high has 2 blocks (n < 10). A MOVE's class is its type alone. No class moves (M0-V4: "no TM").
- **Today's read on linux-pc** (`python3 tools/pb/rung_record.py report`, 09:28): 91 DONE blocks, 16 class/rung rows, 60
  blocks with no session file readable here (win-laptop's), $152.51 read from this box's sessions.
  - BUILD on Sonnet 5.5, high: 11 blocks, 5/11, $1.78. BUILD on Opus 5.5, high: 10 blocks, 4/6, $4.05. It agrees: no move.
  - Lot 5 since M0-V4, all on linux-pc: M0-T33a (Sonnet 5.5, high; GO first try; $0.59), M0-T33b (Sonnet 5.5, high; first
    claim NO-GO on clippy and `desktop[launcher]`; $2.58), M0-T34 (Opus 5.5, high; first claim NO-GO on the timing and dump
    oracles; $6.98).
  - Six of the seven rows read from a handoff are misread (§ Measured and not acted on). The verdict does not change.
- **Rule (1)**: no open block's `Carried flags:` holds an escalation trail.

## The rating: every open block, 109 of them
**Method.**
1. blocks.py diffed every open block against the plan before M0-TM4's pass (e01b7b8).
   - Changed since, apart from M0-TM4's 84 rating segments: new flags on M0-T35, T36, T43 (the five that fired), M0-T60
     (`[M0-D21]`), M0-T84 (`[M0-D26]`), M0-V5 (a CHECK block) and M0-TZ (a PLAN block).
   - New: M0-D28 (filed by M0-T34) and this block.
   - Every other open block is byte-identical to the text M0-TM4 read in full.
2. The record moves no class and the ladder is unchanged, so an unchanged block's derivation is M0-TM4's (tasks/M0-TM4.md §
   The rating), re-checked here by type, `Sizing:` and flags. I read the changed and new blocks in full: heading, `Sizing:`,
   `Carried flags:`, `Deliver:`, `Verify:` and `Pass:`.
3. §0's rubric, first match wins. The closed list is read as M0-TB set it (reports/size_pass.md §9) and M0-TM1–TM4 kept it:
   (c) is "the contract leaves a structure to the builder, and later blocks build on it".

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 14 | V5–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM5, TJ1*, TJ2*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 8 | T35, T36, T42, T43, T46, T51, T56, D28 | +3 |
| Sonnet 5.5, high | (3), no reason from the list | 76 | below | −3 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung | 7 | T39, T48, T52b, T91, T93a, T94, T111 | 0 |
| Fable 5.1, high · Opus 5.5, xhigh · Sonnet 5.5, medium | — | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`
(109 of 109).

The 76 on Sonnet 5.5, high: T37, T38, T40, T41, T44a, T44b, T45, T47, T49, T50, T52a, T53, T54, T55, T57, T58, T59, T60,
T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85,
T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107, T98b, T99b, T100,
T101, T108, T109, T110, T112, T113, T114*, T115, TD, T116. Their kinds, each read against the list, are M0-TM4's.

### The 3 re-rated: Sonnet 5.5, high → Opus 5.5, high (rule (5), reason (c) from (3)'s list)
- **The contract leaves the packing open.** §2.2: "the GPU may pack channels as it likes within the WebGPU default of 8
  storage buffers per shader stage (TW-E15), which binds native builds too (§6.2.2)".
- **M0-T34 measured the kernel full.** Its flags: `hydro_update` binds all 8 (state ×3, next ×3, the Δt record, the eos
  table). M0-T35's escaped booking (M0-T18's BOOK_GROUPS: one buffer per writer), M0-T36's g and M0-T43's F each need one
  more: 11 bindings against 8.
  - One recipe is named: the next state in two buffers of 7 planes (≤ 117 MB each at 2048², under the 128 MiB binding)
    frees one slot.
  - The other option is a third dispatch over the edge cells, which adds the fixed cost per pass that M0-D26 measured
    (70 % of the Sun-sized box's step).
- **Each layout is built on by the next.** M0-T36's flag says "M0-T35 may take it first …: read its handoff". M0-T43's says
  "M0-T35/T36 may each have taken a freed slot … read their handoffs before binding". This is the (c) that M0-TM4 named for
  M0-T34 itself ("within 8 storage buffers per stage: the kernel layout is the builder's").
- **What M0-TM4 read, and what changed.** M0-TM4 read the three as "GPU passes that add a term … to a mechanism built
  earlier: the sources sit in M0-T34's scheme". M0-T34's flags show the scheme has no room for them. The rest of each
  `Deliver:` is fixed wording and earns nothing: the leave ghosts, the booking per M0-T18, §1.4.2's time-centred sources, and
  f = κΣF/c_sb with E unchanged.

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T35 | (c) | `[M0-T34]`: "hydro_update already binds WebGPU's 8 storage buffers …: the edge mode fits a uniform, but the escaped booking needs a slot — e.g. the next state in two buffers of 7 planes …, or a small third dispatch over the edge cells only" |
| M0-T36 | (c) | `[M0-T34]`: "hydro_update binds WebGPU's 8 storage buffers already: g needs a slot — the next state in two buffers of 7 planes frees one (M0-T35 may take it first for the escaped booking: read its handoff)" |
| M0-T43 | (c) | `[M0-T34]`: "hydro_update … is at WebGPU's 8 storage buffers since M0-T34 … and M0-T35/T36 may each have taken a freed slot: the flux F needs one — read their handoffs before binding" |

### The 8 BUILD blocks above the cheapest rung that fits, each with its rule
The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. These 8 sit one rung above it,
on (usual), by rule (5): a reason from (3)'s list, re-derived from the block's own text.

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T35, T36, T43 | (c) | above |
| M0-T42 | (e) | "RKL2's s stages as dispatches with coefficients frozen at P5's start", in f32: f32 precision over up to 32 sub-steps a step (text unchanged since M0-TM4) |
| M0-T46 | (e) | "M0-T44b's sub-cycles per cell in f32", f(T) within 10⁻⁴: stiff sub-cycles in f32 (unchanged) |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)": bit-identical results from parallel code (unchanged) |
| M0-T56 | (e) | every §1.9.1 field reduced in fixed order, "read back asynchronously (≤ 2 frames late)" (unchanged) |
| M0-D28 | (b), (e) | "Caused by: unknown", "Repro: not yet deterministic", "Suspected cause: …": a D whose cause is not measured; two sibling cases racing over one port lock and one server: concurrency as shared state. Filed by M0-T34 on (usual); unchanged |

### The rule (2) blocks: 18, on Opus 5.5, max (gate)
V5–V17 and TW's CHECK scribe (CHECK); TM5, TJ1*, TJ2*, TZ (PLAN). The new flags on M0-V5 (`[M0-V4]`, `[M0-D26]`,
`[M0-T34]`) and M0-TZ (`[M0-D23]`) move nothing: rule (2) pins the gate.

### The other flags new since M0-TM4, read for the rating
- **M0-T35** `[M0-T33b]`: where Leave goes in the twin (`Edge`, `Hydro::wall`). The twin's `State` has no escaped booking
  yet; `State::totals` is the sum to check one against. Fixed by the `Deliver:`; no reason beyond (c).
- **M0-T36** `[M0-T33b]`: where the predictor's sources go in the twin; `Hydro::dt` already takes g; the four K's
  override. This is placement in a fixed scheme (§1.4.2); no reason beyond (c).
- **M0-T60** `[M0-D21]`: sim time exists (`DtReport.sim_time` / `FrameReport.sim_time`, f64); read it for `--until
  sim_time`. A mechanism already built; no reason.
- **M0-T84** `[M0-D26]`: time the step as one compute pass when no per-pass cost is asked for. A timing choice, which the
  list excludes; the timestamps are M0-D26's. No reason.

### Considered — observations, never defects (§0, no step up on a guess)
- **One layout decision ahead of M0-T35 would make the three a reuse.** The 11 bindings against 8 are one design spread
  over three blocks. A block that repacked P4's bindings once, freeing three slots, would leave M0-T35, T36 and T43 each
  adding a binding to a slot that exists: rule (3), Sonnet 5.5, high. That is a re-scope (§2.1), which a rating pass may not
  write. It is named here for the lead and the next PLAN block.
- **M0-T43 is the weakest of the three (c)s.** No open block writes hydro.wgsl after it, so "later blocks build on it" holds
  only past M0. It stays (c) because its design is the most constrained one (the slots that M0-T35 and T36 take are gone),
  and its own flag defers the binding to their handoffs.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and the
  rating is kept.

## What changed in the plan
- **Ratings: 3 changed**, all Sonnet 5.5, high → Opus 5.5, high (M0-T35, T36, T43). By rung, now (changed):
  - Opus 5.5, max: 18 (0).
  - Opus 5.5, high: 8 (+3).
  - Sonnet 5.5, high: 76 (−3).
  - Haiku 5.5, high: 7 (0).
  - Fable 5.1, high, Opus 5.5, xhigh and Sonnet 5.5, medium: 0 (0).
- **The `Models:` line**: re-dated and re-cited (above).
- **The register**: `Model ratings: 2026-10-13 by M0-TM5`, written through `plan.py close --set`.
- **How the edits were made.** plan.py has no call that rewrites a rating. So, as M0-TM2–TM4 did, I used the file-edit tool,
  one heading per edit. Each `old_string` was the whole heading line, unique by its id (grep-counted to 1 each before the
  edits).
- **The diff**, checked by checkdiff.py against a snapshot taken right after `plan.py status`: 3 rating segments, 1 `Models:`
  line, 0 other lines, before the close.

## Measured and not acted on
This block's diff is bounded to rating segments, the `Models:` line and the register's line (its `Pass:` and `Fail:`), so
nothing below is filed or flagged from here.
- **rung_record.py's handoff fallback misreads the rung.** On a box without a block's session file, `report` reads the rung
  from the handoff and ignores its quoted `model=… level=…`.
  - Measured on linux-pc today (`report --json`, the rows `via: handoff`; 6 of 7 misread):
    - M0-TJ3, V2, TM2 and TM4 quote `model=claude-opus-5-5 level=max` and are read as Sonnet 5.5, high.
    - M0-TM3 quotes the same and is read as Sonnet 5.5, medium.
    - M0-TR1 quotes `model=claude-sonnet-5-5 level=high` and is read as Opus 5.5, high (its prose: "both Opus 5.5 high").
    - M0-D13 is read right (Opus 5.5, high, as quoted).
  - In 5 of the 6, the rung read is the first `<model>, <level>` string in the handoff's prose. Suspected cause: the
    fallback's first ladder-rung match in the text.
  - Effect: a V reading the record on the other box counts blocks on rungs they never ran on. M0-TR1 lands in BUILD on
    (usual), the row rule (4) compares against. Cost per DONE block is the mean over sessions' own costs (BUILD on Opus 5.5,
    high: $36.41 / 9 = $4.05), so the misread moves the counts and the first-try rate, not the cost. Rule (4)'s verdict
    here does not change.
  - Filing it as a D on tools/pb/rung_record.py is for the next filer or the lead: a `plan.py append` adds bytes that this
    block's `Fail:` line forbids. M0-V5, which reads the record next, is the block it bites.
- **The register's open-D line** still names M0-D25 ("— after M0-D27"); M0-D25 is DONE (2026-10-12 16:15).
- **lint's 4 WARNs**: M0-TC3, D25, D26 and D27 are DONE outside the current phase and wait to be stubbed (a TC's work, or
  the next V's).
- **M0-T33b's first claim** was NO-GO on Sonnet 5.5, high (clippy; `desktop[launcher]`). A GO re-run followed under
  `M0-T33b-r2` (base M0-T34's start; "no start for M0-T33b-r2"), and the block closed DONE. Rule (1)'s trail (rate it
  (usual), set it back to TODO) was not written. The record prices it as a first-try miss in Sonnet's row; no open block
  carries a trail.
- **The Pipeline state's `Next task: M0-TM5`** goes stale at this close; the next task is M0-D28 (AFTER M0-TM5). This diff is
  bounded, so the next close sets it (M0-TM1–TM4's precedent).
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the picker's level, which the switch
  overrides (§B.3).

## The routine, on linux-pc (laserax-ai)
- **`python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**, after the 3 edits and the `Models:`
  line: [ALREADY RUN — PASS (GO, 519 checks, 0 failed, 4 warnings — the stubs above; header 467/600, ladder 7 rungs, refresh
  5/5) on linux-pc].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (3 rating segments, 1 `Models:` line, 0 other) on
  linux-pc].
- **No red-arm.** This task added no scope and changed no scope's check (R9; PLAYBOOK §4 Rule 4, v12.3).
- **The close**: `python3 tools/pb/plan.py close M0-TM5 --status "DONE" --text "<handoff>" --set "Model ratings=2026-10-13
  by M0-TM5"`, then checkdiff.py again.
- **The claim run**, the last step, after the close: `python3 tools/pb/verify.py --changed --task M0-TM5`. `plan.py show`
  printed a `baseline:` line, so no `--base` is named (R10).
- The results are below, appended after the runs.

### Results, appended after the runs
- **The close** (09:40): `close: checks=4 failed=0 warnings=0 · handoff_lines=1 · flow=per-task · set=1 · replaced=1 ·
  appended=0`, GO. Status `DONE (2026-10-13 09:40, started 09:19)`; the register reads `Model ratings: 2026-10-13 by M0-TM5`.
- **checkdiff.py** after the close: [ALREADY RUN — PASS (3 rating segments, 1 `Models:` line, 3 own lines — Status, Handoff,
  Flow row — 1 register line `Model ratings:`, 0 other) on linux-pc].
- **The claim run**, `python3 tools/pb/verify.py --changed --task M0-TM5`, the last step: [ALREADY RUN — PASS (GO: 518
  passed, 0 failed, plan_lint 1/1 scope, 29 scopes not impacted, 0.12 s; base M0-TM5's start 2026-10-13T09:18:56-04:00;
  tree cf0f7452dd4b+e70944bc0e28; python 3.12.3; load 13.52; anchor 0 stale, 0 unclassified) on linux-pc].
  - lint's count reads 518 after the close, against 519 before it: the same one-check drop M0-TM2–TM4 saw.
- **Files this task wrote**:
  - the plan: 3 rating segments, the `Models:` line, its own Status, Handoff and Flow row, and the register's `Model
    ratings:` line;
  - this file and logs/M0-TM5.log.
  - verify.py wrote logs/M0-TM5.plan_lint.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
  - Only this section was written after the claim run. It is a skip-class path, graded by no scope.
