# M0-TM4 — the rating pass (2026-10-12, win-laptop)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt), the
heading's rung, no mismatch. Raw runs: logs/M0-TM4.log, plus the logs verify.py writes. Read-only helpers, in the session
scratchpad's tm4/ (never in the repo, never run on the plan's behalf): blocks.py (each block's heading, `Sizing:`, `Carried
flags:`, `Deliver:`, `Verify:`, `Pass:`, and each open block diffed against an older plan) and checkdiff.py (the diff against
a snapshot, line by line, each line classed).

**Deviation, declared.** On the launch prompt I ran the hook's line as written, `python3 tools/pb/rung_record.py now`, and
then `python3 tools/pb/plan.py show M0-TM4`. On win-laptop R15 reads both as `py -3.12 tools/pb/…`: `python3` is the Store
alias, 3.13.14 (Hazards). Both returned normally (`now` GO, `show` GO, `baseline:` recorded). Every later call used
`py -3.12`.

## What fired it
M0-V3 filed it at Phase 5's close (PLAYBOOK §2.3, §0 rule (4)), in this block's `Carried flags:`. That V's table (the record,
reports/v3.md § Phase 5's close) no longer moves BUILD on Sonnet 5.5, high:
- BUILD on Sonnet 5.5, high: 11 blocks, first try 9/11, $1.89 per DONE block.
- BUILD on Opus 5.5, high (usual): 24 blocks, first try 17/24, $2.92 per DONE block.

M0-TM2 applied rule (4)'s move from M0-V2's table ($1.70 against $1.44, 10 blocks each). M0-TM3 kept the 89 blocks it moved
and left them for this table to decide.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-296_agent`.
  - The agent is still Claude Code, now 2.1.296; the ladder cited 2.1.295.
  - `CLAUDE_EFFORT` is the picker's launch level, a hint. The switch ran this session at max (`now`).
- **The model pages**: the claude-api skill bundled with Claude Code 2.1.296, loaded this session (extracted to
  `%TEMP%\claude\bundled-skills\2.1.296\…\claude-api`). Its "Current Models (cached: 2026-10-06)" table:
  - Fable 5.1: $10 / $50 per MTok.
  - Opus 5.5: $4 / $20.
  - Sonnet 5.5: $2 / $10.
  - Haiku 5.5: $0.10 / $0.50 for prompts ≤ 100K tokens ($0.50 / $2.50 beyond).
  - Effort levels: low…max on all four. The default is medium for Opus 5.5 and Haiku 5.5, high for Sonnet 5.5.
- **shared/models.md**: byte-identical to 2.1.295's (sha256 1601794b74c4…c6f9a2ec, the file M0-TM2 and M0-TM3 read). All four
  models are `Active`; none is deprecated, retired or renamed.
- **The result**: nothing was retired, renamed, added, repriced or relevelled, and every rung's level is one its model
  takes. The `Models:` line is re-dated and re-cited, M0-TM2's precedent: "read 2026-10-11 … (Claude Code 2.1.295, cached
  2026-10-06)" → "read 2026-10-12 … (Claude Code 2.1.296, cached 2026-10-06)". Its rungs and marks are byte-identical.
- No question to the lead: no rung they run changed (§0).
- **The switch**: M0-TH-adopt-a's swap gave `RUNGS` all seven rungs (tasks/M0-TH-adopt-a.md), and M0-TC2 ran on
  `model=claude-haiku-5-5 level=high` (tasks/M0-TC2.md). M0-TM3's doubt about Haiku 5.5, high is closed.

## The record
`py -3.12 tools/pb/rung_record.py report` (2026-10-12, win-laptop):
```
report: 70 DONE blocks · 11 class/rung rows · 1 session folder(s) (C--workspace-sandbox-reactions) · 28 block(s) with no session file readable (rung from the handoff, cost null) · 0 block(s) priced from tokens (a session file with no cost line) · 0 priced from list prices · 3 D block(s) naming no cause in the record · $159.42 read from the sessions' own cost
BUILD on Opus 5.5, high: 24 block(s) · first-try 17/24 · $2.91 per DONE block · usual
BUILD on Sonnet 5.5, high: 11 block(s) · first-try 9/11 · $1.89 per DONE block · cheaper on Sonnet 5.5, high · vs same class on (usual)
BUILD on unknown: 17 block(s) · first-try 7/12 · n/a per DONE block · off the ladder
BUILD·E on Sonnet 5.5, medium: 1 block(s) · first-try 1/1 · $1.15 per DONE block · n<10
BUILD·E on unknown: 2 block(s) · first-try 2/2 · n/a per DONE block · off the ladder
CHECK on Opus 5.5, max: 2 block(s) · first-try 2/2 · $21.10 per DONE block · above (usual)
CHECK on unknown: 1 block(s) · first-try 1/1 · n/a per DONE block · off the ladder
MOVE on Haiku 5.5, high: 1 block(s) · first-try 1/1 · $0.04 per DONE block · n<10
MOVE on unknown: 1 block(s) · first-try n/a · n/a per DONE block · off the ladder
PLAN on Opus 5.5, max: 5 block(s) · first-try 4/5 · $8.41 per DONE block · above (usual)
PLAN on unknown: 5 block(s) · first-try 2/2 · n/a per DONE block · off the ladder
=== GO ===
```
- **Rule (4), read on M0-V3's table** (§0: "the record is the phase-closing V's table"). BUILD on Sonnet 5.5, high has
  ≥ 10 blocks below (usual), at $1.89 per DONE block. That is not above BUILD on (usual), which has ≥ 10 blocks there at
  $2.92. **The class does not move.**
  - The costs come from session files, so the cost comparison applies.
  - The list-price branch agrees, for the record: 9/11 = 0.82 is above 17/24 × ($10 / $20) = 0.35.
- **Today's report reads the same.** $1.89 against $2.91. The cent is drift since V3's read. The DONE count is 70 against
  V3's 68 (M0-V3 and M0-TC2 added).
- **The comparison is like for like now.** The (usual) row holds lot 3's rule-(4) builds (M0-TH-adopt-a, T23, T24, T25, T27)
  next to its rule-(5) builds and the small D fixes.
- **No other class moves.**
  - BUILD·E: Haiku 5.5, high has no record; Sonnet 5.5, medium has 1 block (M0-D13).
  - MOVE on Haiku 5.5, high: 1 block (M0-TC2, $0.04). Under 10 blocks; and a MOVE's class is its type alone.
- **Rule (1)**: no open block's `Carried flags:` holds an escalation trail.

## The rating: every open block, 122 of them
**Method.**
1. I diffed every open block against its text before M0-TM3's pass (ab36299; M0-TM3's own edits there were 7 E headings).
   - Changed since: M0-T35, T37, T39, T84, T85, T111 (new flags), M0-V4 and M0-TZ (flags on a CHECK and a PLAN block), and
     the 7 E headings.
   - New: M0-TR1, D21, D22, D23, D24 and this block.
2. Because the outcome now decides a rung, I read every one of the 103 open BUILD blocks again in full: heading, `Sizing:`,
   `Carried flags:`, `Deliver:`, `Verify:` and `Pass:`.
3. I applied §0's rubric, first match wins.

I read the closed list as M0-TB set it (reports/size_pass.md §9) and M0-TM1–TM3 kept it:
- (a) a FROZEN passage, a schema, a persisted format or a migration changed.
- (b) a D whose cause is not measured.
- (c) a design the spec leaves open: the contract leaves a structure to the builder, and later blocks build on it.
- (d) secrets, money or data loss.
- (e) numeric or precision semantics, or concurrency, as the block's own acceptance: bit-identical results from parallel
  code, GPU-side control state, an asynchronous readback, or f32 precision over many sub-steps. Not an f64 twin graded by an
  analytic oracle.
- (f) a library's internals read to find a mechanism.

A block that reuses a mechanism an earlier block built does not earn the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 15 | V4–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM4, TJ1*, TJ2*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 8 | T30, T31, T34, T42, T46, T51, T56, D21 | 0 |
| Opus 5.5, high (usual) | (4), the record | 0 | — the class no longer moves (M0-V3) | −84 |
| Sonnet 5.5, high | (3), no reason from the list | 86 | the 84 below, TR1, D23 | +84 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung | 9 | T39, T48, T52b, T91, T93a, T94, T111, D22, D24 | 0 |
| Fable 5.1, high · Opus 5.5, xhigh · Sonnet 5.5, medium | — | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`.

### The 84 re-rated: Opus 5.5, high → Sonnet 5.5, high (rule (3), no reason from its list)
These are M0-TM3's 89 rule-(4) blocks, less the five that are now DONE (M0-TH-adopt-a, T23, T24, T25, T27). The move that put
them on (usual) no longer holds, and no reason from (3)'s list holds for any of them:

T28, T29, T32, T33a, T33b, T35, T36, T37, T38, T40, T41, T43, T44a, T44b, T45, T47, T49, T50, T52a, T53, T54, T55, T57, T58,
T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83,
T84, T85, T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107, T98b,
T99b, T100, T101, T108, T109, T110, T112, T113, T114*, T115, TD, T116.

They fall into kinds, each read against the list:
- **f64 twins graded by analytic or exact oracles**: T28, T29, T33a, T33b, T41, T44a, T44b, T45, T50, T53.
  - No (e) by M0-TB's reading: errors there are loud.
  - No (c): each algorithm has its reference (Hockney–Eastwood, Toro's exact solver, MUSCL-Hancock with HLL, RKL2), and the
    contract fixes it. M0-T29's fallback to the force kernels is named by §5.1.
- **GPU passes that add a term or a gate to a mechanism built earlier**: T35, T36, T43, T47, T52a, T54, T76.
  - The edge ghosts and the booking are M0-T18's. The sources sit in M0-T34's scheme. The latch's atomicOr is M0-T22's.
  - M0-T54's third indirect buffer extends M0-T22's latch by its recipe (its flag). M0-T76's box flag is M0-T21's.
- **The twin's driver and the comparisons**: T37, T38, T110.
  - M0-T37's acceptance is CPU against CPU.
  - M0-T38's is G-REF's tolerances on mechanisms built earlier.
  - M0-T110 tests the determinism M0-T18, T20, T22, T26 and M0-D21 build; a red is a D.
- **Observers and readouts on the CPU**, each fixed by the contract: T57 (the fixed-order partials are M0-T56's), T58, T59,
  T60, T61, T69, T72, T73.
- **Calibration**: T65, T66, T68, T70, T95.
  - M0-T65 implements §2.11's schema and changes none; its SHA-256 holds no secret.
  - The calibration file is regenerable, and the tests write a partial file or a copy, so (d) does not hold.
- **Rendering and the app's UI**: T79, T80, T81, T82, T84, T85, T86, T87, T89, T90, T92, T93b.
  - M0-T84's `poll()` never blocks, by M0-T22's design.
  - Its wasm32 wall-clock `step_ms` is a timing choice, which the list excludes.
- **Long whole-life tests and measurement reports** (⏱ and probes): T62, T63, T67, T97, T103–T107, T98b, T99b, T100, T101,
  T109, T112, T113. Timing is never a reason. An UNVERIFIED red is a D or a question, never this block's design.
- **Tuning**: T98a, T99a. "Judgment-heavy" is not on the list; physics.json's values change and no schema does.
- **The cadence**: T114*, which reuses M0-T21's GPU-side sizes.
- **Edits and formats**: T74, T75, T77 (triggers read at step boundaries, synchronous in headless runs).
- **Docs, the UI/UX passes, the demo and the tracker**: T32, T40, T49, T55, T64, T71, T78, T83, T88, T96, T102, T108,
  T115, TV2, TV3, TD, T116.

### The 8 BUILD blocks above the cheapest rung that fits, each with its rule
The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. These 8 sit one rung above it,
on (usual), by rule (5): a reason from (3)'s list, re-derived from the block's own text.

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T30 | (c), (e) | "a multi-pass scheme there", "as few dispatches as the limits allow": the GPU FFT is left to the builder, and P2 (M0-T31) and P7 (M0-T54) convolve through it; "≤ 10⁻⁵ relative L2 in f32" |
| M0-T31 | (c) | M0-T21's flag: the FFT size per axis lives only on the GPU, and "choosing a cached kernel transform per size on the CPU would need a readback — size gravity's dispatches indirectly or hold every power-of-two size's kernel". The contract fixes the cache per size (§1.3.3) and forbids a readback that decides a dispatch; how P2 picks a size's kernel is left open |
| M0-T34 | (c), (e) | "fused into as few dispatches as the limits allow" "within 8 storage buffers per stage": the kernel layout is the builder's; "M0-T33b's scheme in f32" |
| M0-T42 | (e) | "RKL2's s stages as dispatches with coefficients frozen at P5's start", in f32: f32 precision over up to 32 sub-steps a step |
| M0-T46 | (e) | "M0-T44b's sub-cycles per cell in f32", f(T) within 10⁻⁴: stiff sub-cycles in f32 |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)": bit-identical results from parallel code |
| M0-T56 | (e) | every §1.9.1 field reduced in fixed order, "read back asynchronously (≤ 2 frames late)", the accumulators reset |
| M0-D21 | (e), (c) | "within 10⁻⁹ relative of the f64 sum of the steps' f32 Δt in that order", "four byte-identical dumps" across frame groupings. WGSL has no f64, so where each step's Δt is summed in step order is the builder's, and M0-T60's `--until sim_time`, M0-T110's dumps and M0-T111's read it. The filer (M0-V3) named numeric semantics and a persisted format; (e) alone sets the rung |

### The new blocks, re-derived (filed by M0-V3; every rating unchanged)
- **M0-TR1**: Sonnet 5.5, high, rule (3). A measurement, a report, and the fixes filed as blocks; no reason from the list.
- **M0-D21**: (usual), rule (5). See the table above.
- **M0-D22, M0-D24**: Haiku 5.5, high, rule (3), `Sizing: E`. Each cause is measured (from verify.anchor.json; from `git apply
  --check` at three commits), and neither block has a reason.
- **M0-D23**: Sonnet 5.5, high, rule (3).
  - Its `Caused by: unknown` names no causing block, but the cause itself is measured. From PowerShell, `bash` resolves to
    WSL's relay (the log). With Git's usr\bin first on the PATH, the scope reads GO 18/18. So (b) does not hold.
  - How Windows resolves a bare `bash` is documented OS behaviour, not a library's internals, so (f) does not hold.

### The E blocks: Haiku 5.5, high (rule (3))
BUILD·E on Haiku 5.5, high still has no record, so rule (4) moves nothing. Each block was re-read for a reason from the list:
- **M0-T48, T52b, T91, T94**: as M0-TM3 read them (M0-T38's shape copied; SRDUMP01's `sinks` key already in the header from
  M0-T26; a view section; URL parameters and a smoke case).
- **M0-T93a**: the closest call. Its asynchronous one-cell readback reuses M0-T22's map-on-submit `poll()` and M0-T56's
  ≤ 2-frames-late readback (M0-TM2, M0-TM3).
- **M0-T111**: new M0-V3 flag (keep both the k = 1 and the k = 15 latch cases). A test-design instruction on M0-T22's latch;
  no reason.
- **M0-T39**: new M0-V3 flag; see Considered.

### The flags new since M0-TM3, read for the rating
- **M0-T35** `[M0-T25]`: EdgeMode reaches headless; `set_edge_mode` wires it. An interface, built. No reason.
- **M0-T37** `[M0-T25]`: `Scene::planes`, switches and overrides, so the twin starts from the same scene. `[M0-T27]`: the
  USAGE line. No reason.
- **M0-T39** `[M0-V3]`: G-BOX against the vacuum floor; see Considered.
- **M0-T84**:
  - `[M0-T24]`: TimeControl's report and plan, and N_max until a `step_ms` arrives.
  - The wasm32 wall-clock `step_ms` is a timing choice, excluded.
  - The f32::MAX sim-time guard is a sentinel guard, as M0-TM3 read M0-T24's own.
  - `[M0-V3]`: the desktop, capture and web scopes' paths gain sr-physics and the assets, a manifest edit of M0-D22's class.
  - No reason.
- **M0-T85** `[M0-T24]`: speed_text, rungs and the banner key. An interface. No reason.
- **M0-T111** `[M0-V3]`: above.
- The new flags on M0-V4 (`[M0-V3]`) and M0-TZ (`[M0-TH-adopt-a]`, `[M0-D20]`) sit on a CHECK and a PLAN block. Rule (2)
  pins those ratings.

### Considered — observations, never defects (§0, no step up on a guess)
- **M0-T39 on Haiku 5.5, high, against M0-V3's arithmetic.**
  - The arithmetic (source only): under the whole-world switch every vacuum cell floors at step 0, so the raw grid sums differ
    from the box run's. The hydrogen sum is 9.3 × 10⁻⁷ off, inside §5.3's 10⁻⁶ by about 7 %.
  - I weighed (e). The flag names the comparison to make: the conserved sums, vacuum_reset added per §2.9, or a stated sum
    per case. It is a fixed instruction, not a design left open, and the sums and the box are M0-T18's and M0-T21's.
  - No reason. A NO-GO claim run there rates it (usual) by rule (1).
- **Haiku 5.5, high is a large step down for the 9 E blocks** (output $0.50 against Sonnet's $10 per MTok). The rung's only
  record so far is M0-TC2, a MOVE (1 of 1 first try, $0.04), which is another class; BUILD·E has none there. Rule (1) prices a
  miss.
- **The 84 lose the (usual) rung they held since M0-TM2.** That is rule (4) as written: the record that moved the class no
  longer does. Rule (1) prices a miss per block: a NO-GO claim run rates it (usual) and sets it back to TODO. Rule (4) reads
  the class again at each phase-closing V. "No record yet" is never a reason, and here the record exists.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and the
  rating is kept.

## What changed in the plan
- **Ratings: 84 changed**, all Opus 5.5, high → Sonnet 5.5, high. By rung, now (changed):
  - Opus 5.5, max: 19 (0).
  - Opus 5.5, high: 8 (−84).
  - Sonnet 5.5, high: 86 (+84).
  - Haiku 5.5, high: 9 (0).
  - Fable 5.1, high, Opus 5.5, xhigh and Sonnet 5.5, medium: 0 (0).
- **The `Models:` line**: re-dated and re-cited, read 2026-10-11 (Claude Code 2.1.295) → read 2026-10-12 (Claude Code
  2.1.296). The cache date and the rungs are byte-identical.
- **The register**: `Model ratings: 2026-10-12 by M0-TM4`, written through `plan.py close --set`.
- **How the edits were made.** plan.py has no call that rewrites a rating. So, as M0-TM2 and M0-TM3 did, I used the
  file-edit tool, one heading per edit. Each `old_string` was the whole heading line, unique by its id; I checked each was
  unique before editing.
- **The diff**, checked by checkdiff.py against a snapshot taken right after `plan.py status`: 84 rating segments, 1 `Models:`
  line, 0 other lines, before the close.

## Measured and not acted on
- **Repo facts still read "Claude Code 2.1.295"** for win-laptop (M0-TE-V12.3's re-probe); the probe reads 2.1.296 today. The
  switch holds (`plugin=installed`; the session ran on the rung). That line is outside this diff; it belongs to the next block
  that edits Repo facts.
- **The Pipeline state's `Next task: M0-TM4`** goes stale when this block closes; the next task is M0-TR1. This diff is
  bounded to rating segments, the `Models:` line and the register's `Model ratings:` line (M0-TM1–TM3's precedent), so the
  next close sets it.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the picker's level, which the switch
  overrides (§B.3).

## The routine, on win-laptop (box Laser2025-20)
- **`py -3.12 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**, after the 84 edits and the `Models:`
  line: [ALREADY RUN — PASS (GO, 516 checks, 0 failed, 0 warnings, header 454/600, ladder 7 rungs, refresh 0/5) on
  win-laptop].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (84 rating segments, 1 `Models:` line, 0 other) on
  win-laptop].
- **No red-arm.** This task added no scope and changed no scope's check (R9; PLAYBOOK §4 Rule 4, v12.3).
- **The close**: `py -3.12 tools/pb/plan.py close M0-TM4 --status "DONE" --text "<handoff>" --set "Model ratings=2026-10-12
  by M0-TM4"`, then checkdiff.py again.
- **The claim run**, the last step, after the close: `py -3.12 tools/pb/verify.py --changed --task M0-TM4`. `plan.py show`
  printed a `baseline:` line, so no `--base` is named (R10).
- The results are below, appended after the runs.

### Results, appended after the runs
- **The close** (08:19): `close: checks=4 failed=0 warnings=0 · handoff_lines=1 · flow=per-task · set=1 · replaced=1 ·
  appended=0`, GO. Status `DONE (2026-10-12 08:19, started 08:03)`; the register reads `Model ratings: 2026-10-12 by M0-TM4`.
- **checkdiff.py** after the close: [ALREADY RUN — PASS (84 rating segments, 1 `Models:` line, 3 own lines — Status, Handoff,
  Flow row — 1 register line `Model ratings:`, 0 other) on win-laptop].
- **The claim run**, `py -3.12 tools/pb/verify.py --changed --task M0-TM4`, the last step (08:20): [ALREADY RUN — PASS (GO: 515
  passed, 0 failed, plan_lint 1/1 scope, 23 scopes not impacted, 0.30 s; base M0-TM4's start 2026-10-12T08:01:58-04:00; tree
  e01b7b8ef726+df28f9a91253; python 3.12.10; load 3.96 → 1.7; no FLAG) on win-laptop].
  - lint's count reads 515 after the close, against 516 before it. M0-TM2 and M0-TM3 saw the same one-check drop.
- **Files this task wrote**:
  - the plan: 84 rating segments, the `Models:` line, its own Status, Handoff and Flow row, and the register's `Model
    ratings:` line;
  - this file and logs/M0-TM4.log.
  - verify.py wrote logs/M0-TM4.plan_lint.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
  - Only this section was written after the claim run. It is a skip-class path, graded by no scope.
