# M0-TM3 — the rating pass (2026-10-11, win-laptop)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt), the
heading's rung, no mismatch. Raw runs: logs/M0-TM3.log, plus the logs verify.py writes. Read-only helpers (never run on
the plan's behalf, never in the repo), in the session scratchpad's tm3/: blocks.py (each block's heading, Sizing, Carried
flags, Deliver, Verify), blockdiff.py (each open block against its text at M0-TM2's close, 2f176dd), classify.py (the
rating below, checked against the open blocks) and checkdiff.py (the diff against a snapshot, line by line).

## What fired it
M0-TE-V12.3 filed it in place of its own rating (PLAYBOOK §0: an adoption that would rate past ~15 open blocks). The
ladder it wrote from the lead's answer adds three rungs — Fable 5.1, high above `(gate)`, Opus 5.5, xhigh between `(gate)`
and `(usual)`, Haiku 5.5, high the lowest — so rule (3)'s rung for an E block or a MOVE is now Haiku 5.5, high.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-295_agent`. The agent is still
  Claude Code 2.1.295. `CLAUDE_EFFORT` is the picker's launch level, a hint; the switch ran this session at max (`now`).
- **The model pages**: the claude-api skill bundled with Claude Code 2.1.295 on this box, "Current Models (cached:
  2026-10-06)", and its `shared/models.md` (sha256 1601794b…a2ec, the file M0-TM2 read):
  - Fable 5.1 $10 / $50 per MTok, effort low…max · Opus 5.5 $4 / $20, low…max (default medium) · Sonnet 5.5 $2 / $10,
    low…max (default high) · Haiku 5.5 $0.10 / $0.50 for prompts ≤ 100K tokens ($0.50 / $2.50 above), low…max (default
    medium).
  - All four Active; none deprecated, retired or renamed. Every rung's level is one its model takes.
- **The result**: nothing retired, renamed, added or repriced, no level changed against M0-TE-V12.3's read the same day.
  The `Models:` line already reads 2026-10-11 and stays byte-identical. No question to the lead: no rung they run
  changed (§0), and the lead gave the ladder itself (M0-TE-V12.3's flag).
- Claude Code 2.1.296's VS Code extension is on disk (installed 2026-10-10 08:34) but not running: the probe reads
  2.1.295, and no 2.1.296 bundle is extracted. Its model table is the next re-read's (an adoption, a TM).

## The record
`py -3.12 tools/pb/rung_record.py report` (2026-10-11 11:4x, win-laptop):
```
report: 60 DONE blocks · 10 class/rung rows · 1 session folder(s) · 28 block(s) with no session file readable (rung from the handoff, cost null) · 0 block(s) priced from tokens (a session file with no cost line) · 0 priced from list prices · 3 D block(s) naming no cause in the record · $101.58 read from the sessions' own cost
BUILD on Opus 5.5, high: 18 block(s) · first-try 13/18 · $2.46 per DONE block · usual
BUILD on Sonnet 5.5, high: 10 block(s) · first-try 8/10 · $2.05 per DONE block · cheaper on Sonnet 5.5, high · vs same class on (usual)
BUILD on unknown: 17 block(s) · first-try 7/12 · n/a per DONE block · off the ladder
BUILD·E on Sonnet 5.5, medium: 1 block(s) · first-try 1/1 · $1.15 per DONE block · n<10
BUILD·E on unknown: 2 block(s) · first-try 2/2 · n/a per DONE block · off the ladder
CHECK on Opus 5.5, max: 1 block(s) · first-try 1/1 · $16.30 per DONE block · above (usual)
CHECK on unknown: 1 block(s) · first-try 1/1 · n/a per DONE block · off the ladder
MOVE on unknown: 1 block(s) · first-try n/a · n/a per DONE block · off the ladder
PLAN on Opus 5.5, max: 4 block(s) · first-try 3/4 · $8.95 per DONE block · above (usual)
PLAN on unknown: 5 block(s) · first-try 2/2 · n/a per DONE block · off the ladder
=== GO ===
```
- **Rule (4) rates from M0-V2's table.** §0: "The record is the phase-closing V's table (§2.3)"; §2.3: that table is "the
  record §0's rubric rates from (rule (4))". The last phase-closing V is M0-V2 (Phase 4's close), whose table moved BUILD on
  Sonnet 5.5, high one rung: $1.70 against $1.44 per DONE block, 10 blocks each. Phase 5 is still open, and its V is M0-V3.
  So the class stays moved, and its blocks stay on (usual), the cap. M0-TM2 tied the return to the same event: "If a later
  phase-closing V's table no longer shows the move, the rubric returns the 89 to Sonnet 5.5, high at the next rating pass."
- **Today's report reads the comparison the other way. Measured, not acted on.** The Sonnet row is $2.05 per DONE block,
  against $2.46 on (usual).
  - Sonnet row: $1.85 at M0-TM2 → $2.05. M0-D19 ($1.94) is now traced to M0-T15, whose total is $8.18 (own $4.72, plus
    M0-D17 and M0-D19). Its median is $1.11.
  - (usual) row: $1.45 over 11 blocks at M0-TM2 → $2.46 over 18 (17 priced). The seven blocks added are lot 3's M0-T18–T22
    and M0-D18, D19. T18–T22 are rule (5) blocks, each with a reason from (3)'s list, at $2.39, $7.14 (with M0-D18), $4.63,
    $5.85 and $3.83: a $4.77 mean. Without them the row is 12 small fixes (M0-D2–D8, D15–D19) at $1.50. Its median is $1.59.
  - A traced D counts twice: in its own row and in its cause's total. M0-D17 and M0-D19 are on (usual) and in M0-T15's
    Sonnet total; M0-D18 is on (usual) twice. That is rule (4)'s "plus the D blocks traced to it", as the tool reads it.
  - No rule-(4) block has run on (usual) yet; the first are M0-TH-adopt-a and M0-T23–T27. M0-V3's table (Phase 5's
    close) is the first like-for-like read, and it decides whether the move holds.
- **BUILD·E**: Haiku 5.5, high has no record, and Sonnet 5.5, medium has one block (M0-D13, n < 10). No move.
- **Rule (1)**: no open block's `Carried flags:` holds an escalation trail.

## The rating: every open block, 124 of them
**Method.** I diffed every open block against its text at M0-TM2's close (2f176dd). The only changes are carried flags:
ten from lot 3's builders (M0-T18, T19, T21, T22) into twelve later blocks, and M0-TE-V12.3's into M0-T27. There is one
new block (M0-TH-adopt-a, besides this one). Then I read every open block's heading, `Sizing:`, `Carried flags:`, `Deliver:`
and `Verify:` (and `Pass:`). I applied §0's rubric, first match wins, with the closed list read as M0-TB, M0-TM1 and M0-TM2
read it:
- (a) a FROZEN passage, a schema, a persisted format or a migration changed
- (b) a D whose cause is not measured
- (c) a design the spec leaves open
- (d) secrets, money or data loss
- (e) numeric or precision semantics, or concurrency, as the block's own acceptance
- (f) a library's internals read to find a mechanism

A block that reuses a mechanism an earlier block built does not earn the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 16 | V3–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM3, TJ1*, TJ2*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 8 | T26, T30, T31, T34, T42, T46, T51, T56 | 0 |
| Opus 5.5, high (usual) | (4), the record — BUILD on Sonnet 5.5, high moved once (M0-V2) | 89 | the list below | 0 |
| Sonnet 5.5, high | (3), no reason from the list | 0 | — | 0 |
| Sonnet 5.5, medium | — | 0 | — | −7 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung; BUILD·E has no record there | 7 | T39, T48, T52b, T91, T93a, T94, T111 | +7 |
| Fable 5.1, high · Opus 5.5, xhigh | — (rule (1) only; no trail) | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`.

### The 97 BUILD blocks above the cheapest rung that fits, each with its rule
The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. All 97 sit one rung above
it, on (usual).

**Rule (4), the record — 89 blocks, no reason from (3)'s list.** M0-TM2's 89 less M0-T31 (now (5), below), plus
M0-TH-adopt-a:
TH-adopt-a, T23, T24, T25, T27, T28, T29, T32, T33a, T33b, T35, T36, T37, T38, T40, T41, T43, T44a, T44b, T45, T47, T49,
T50, T52a, T53, T54, T55, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75,
T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a,
T102, T103, T104, T105, T106, T107, T98b, T99b, T100, T101, T108, T109, T110, T112, T113, T114*, T115, TD, T116.

**M0-TH-adopt-a — the roll-out's placeholder, re-rated.** It is a BUILD (TH-adopt) without `Sizing: E`, so its class is
BUILD on rule (3)'s Sonnet 5.5, high, which the record moved → (4), Opus 5.5, high. The rung is unchanged; it now has its
rule. No reason holds: the annex is the spec. It changes no schema or persisted format (it adds one `tools_selftest` case,
the manifest's own shape), and the local-fix merge is careful work, not a list item.

**Rule (3)'s list → rule (5) — 8 blocks.** Each reason is re-derived from the block's own text:

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T26 | (e) | "2 × 100 steps through a dump equal 200 straight steps, bit-identical (§1.3.4)" — every bit of hidden state round-trips |
| M0-T30 | (c), (e) | "a multi-pass scheme there", "as few dispatches as the limits allow" — the GPU FFT left to the builder; "≤ 10⁻⁵ relative L2 in f32" |
| M0-T31 | (c) — **new** | M0-T21's flag (2026-10-10): "The FFT size per axis lives only on the GPU (ActiveBox.fft_w/fft_h, boxfit.rs); choosing a cached kernel transform per size on the CPU would need a readback — size gravity's dispatches indirectly or hold every power-of-two size's kernel (32 … 4096)". The contract fixes "the kernel's transform is cached per size" (§1.3.3) and that no dispatch is decided by a readback (M0-T21's box). It leaves open how P2 picks a size's kernel and sizes a multi-pass FFT when only the GPU knows the size. Neither option is built: M0-T30's tests run sizes the CPU picks, and M0-T21 sized single per-cell passes. |
| M0-T34 | (c), (e) | "fused into as few dispatches as the limits allow" "within 8 storage buffers per stage"; M0-T33b's scheme in f32 (M0-T21's new flag — the box's ghosts read Σ < Σ_vac as the floor — adds no reason) |
| M0-T42 | (e) | "RKL2's s stages as dispatches with coefficients frozen at P5's start", in f32 |
| M0-T46 | (e) | "M0-T44b's sub-cycles per cell in f32", f(T) within 10⁻⁴ |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)" |
| M0-T56 | (e) | every §1.9.1 field reduced in fixed order, "read back asynchronously (≤ 2 frames late)" |

M0-T18–T22 were the other five of M0-TM2's twelve; they are DONE.

The (4)/(5) split decides no rung today: both land on (usual). It decides rungs only if M0-V3's table (or a later one) no
longer moves the class. The next rating pass then returns the 89 to Sonnet 5.5, high and keeps these 8 on (usual).

### The 7 E blocks — Sonnet 5.5, medium → Haiku 5.5, high (rule (3))
Rule (3): "`Sizing: E` and a MOVE the ladder's lowest rung", now Haiku 5.5, high. BUILD·E on Haiku 5.5, high has no
record, so rule (4) moves nothing. Each block was re-read for a reason from the list; none holds:
- **M0-T39** — G-BOX's cases on M0-T21's box and whole-world switch (its flag: box.rs exists, 6 cases, `Step::with_box(…,
  BoxMode::WholeWorld)`). The 10⁻⁶ sums are the summary's f64 sums; the mechanism is M0-T21's.
- **M0-T48** — G-REF's third case, "copied from M0-T38's shape"; `cpu-gpu`'s count 2 → 3.
- **M0-T52b** — the dump header's `sinks` list. SRDUMP01's header carries `sinks` from M0-T26 (§2.12.3), so no format
  changes; its bit-identical round trip is M0-T26's mechanism.
- **M0-T91** — the views section of the left panel, its view-model cases, one capture action.
- **M0-T93a** — `inspect(cell)`, an asynchronous one-cell readback "never blocking". This is the closest call. Its
  asynchrony reuses M0-T22's map-on-submit `poll()` readback and M0-T56's ≤ 2-frames-late summary readback, both built
  earlier. M0-TM2 read it the same way.
- **M0-T94** — the web build's URL parameters and one smoke case.
- **M0-T111** — G-LATCH's contract case on M0-T22's latch and test hook (its flag: "The mechanism and a test hook exist").
  The dump's remake, when over 10 minutes, is the lead's run, not a `LEAD go` on this block.

### The flags new since M0-TM2, read for the rating
- **M0-T31** `[M0-T21]`: the reason (c) above.
- **M0-T24** `[M0-T21]`: an all-vacuum world's Δt is f32's largest (3.4e38), and "Δt_est, N and the sandbox clock must
  survive it". I weighed (e). It is a guard against one sentinel value, "cheap to reverse in reduce.wgsl", not the block's
  own acceptance (§1.8's rules at their edges, 14 cases). No reason. `[M0-T22]`: the FrameReport fields — an interface,
  fixed.
- **M0-T37** `[M0-T19]`: P8's f64 twin moved into reference/floors.rs, matching §2.7, with "Σ_vac and Σ_floor compared as
  f32 (the GPU's)". This is a fixed spec, and the block's acceptance is CPU against CPU (identical dumps). No reason.
- **M0-T54** `[M0-T22]`: a third indirect buffer added to latch.wgsl's save, zero and restore, and the assert widened. That
  extends M0-T22's mechanism by its recipe, like the GPU-zeroed sizes from P6's sum (M0-T21, M0-T22) and M0-T30's
  convolution it already reused. No reason.
- **M0-T57** `[M0-T18]`: book_totals is a 22 MB readback, and the per-frame path needs "a fixed-order GPU partial reduction
  in TERMS order". M0-T56's block summary builds it ("the frame accumulators summed (M0-T18)", in fixed order), and
  M0-T57 reads "the summary's fixed-order block partials". This is reuse. No reason.
- **M0-T23, T47, T52a, T76, T111** `[M0-T21, M0-T22]`: where the frame report, the latch flag and the box's edit flag live.
  These reuse built mechanisms. No reason.
- **M0-T27, M0-TH-adopt-a** `[M0-TE-V12.3]`: docs rows and the adoption's calls. No reason.
- The flags on M0-V3 and M0-TZ sit on a CHECK and a PLAN block. Rule (2) pins those ratings.
- M0-T25's `[M0-T4]` and M0-T98a's and T99a's `[M0-TB]` flags are read as M0-TM1 and M0-TM2 read them. No reason.

### Considered — observations, never defects (§0, no step up on a guess)
- **Haiku 5.5, high is a large step down for the 7 E blocks** (list price $0.50 against Sonnet's $10 per output MTok).
  Rule (3) names the rung, and "no record yet" is never a reason. Rules (1) and (4) price a miss there: a NO-GO claim run
  rates the block (usual) and returns it TODO.
- M0-TM1's and M0-TM2's other observations still hold for the open blocks:
  - M0-T66 and M0-T95 write into the calibration file. The file is regenerable, and their tests write a partial file or a
    copy, so (d) does not hold.
  - M0-T38 and M0-T48 check tolerances on mechanisms built earlier.
  - M0-T110 tests the determinism T18, T20, T22 and T26 build.
  - M0-T114 reuses M0-T21's GPU-side sizes.
  - M0-T23: timestamp-query is public API.
  - M0-T65: SHA-256 is written against FIPS 180-4's vectors, and it holds no secret.
  - M0-T98a and M0-T99a: tuning is not on the list.

## What changed in the plan
- **Ratings: 7 changed.** By rung, now (changed):
  - Opus 5.5, max: 20 (0).
  - Opus 5.5, high: 97 (0).
  - Sonnet 5.5, high: 0 (0).
  - Sonnet 5.5, medium: 0 (−7).
  - Haiku 5.5, high: 7 (+7).
  - Fable 5.1, high and Opus 5.5, xhigh: 0 (0).
- **The `Models:` line**: unchanged (read 2026-10-11, the same day and the same table as M0-TE-V12.3).
- **The register**: `Model ratings: 2026-10-11 by M0-TM3`, written through `plan.py close --set`.
- **How the edits were made.** plan.py has no call that rewrites a rating, so I used the file-edit tool, one heading per
  edit, as M0-TM2 did. Each `old_string` was the whole heading line: its id and title made it unique, so no other
  block's heading could be hit.
- **The diff**, checked by checkdiff.py against a snapshot taken right after `plan.py status`: 7 headings differ only in
  `Sonnet 5.5, medium` → `Haiku 5.5, high`, with 0 other lines, before the close. The close then adds my own Status, Handoff
  and Flow row, and the register's `Model ratings:` line (§ The routine).

## Measured and not acted on
- **Today's record no longer moves BUILD on Sonnet 5.5, high** (§ The record). §0's record is M0-V2's table, so this pass
  keeps the 89. M0-V3 reads Phase 5's table. If the move no longer holds there, the next rating pass returns the 89 to
  Sonnet 5.5, high; whether that alone files a TM is that V's reading of §0's triggers. A flag into M0-V3 would change bytes
  outside this block's diff, so the point lives here and in the handoff.
- **The switch's `RUNGS` probably lacks Haiku 5.5, high**: the box's plugin is 12.2.1, built for the 2026-10-08 ladder.
  M0-TH-adopt-a reads it after the swap (its flag). Until the lead rebuilds and reinstalls, the 7 E blocks run on the lead's
  pick (§B.3). A rating never waits on it (§0). The first of them, M0-T39, is in lot 5.
- **The Pipeline state's `Next task: M0-TM3`** goes stale once this block closes; the next task is M0-TH-adopt-a. This
  block's diff is bounded to rating segments, the `Models:` line and the register's `Model ratings:` line (M0-TM1's and
  M0-TM2's precedent), so the next close sets it.
- **Claude Code 2.1.296** is installed but not running (§ The ladder). The ladder cites 2.1.295, the running agent.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max: the picker's level, which the switch
  overrides (§B.3).

## The routine, on win-laptop (box Laser2025-20)
- **`py -3.12 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**, after the 7 edits:
  [ALREADY RUN — PASS (GO, 506 checks, 0 failed, 0 warnings, ladder 7 rungs, refresh 2/5) on win-laptop].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (7 headings, 0 other lines) on win-laptop].
- **No red-arm.** This task added no scope and changed no scope's check (R9, PLAYBOOK §4 Rule 4, v12.3).
- **The close**, `py -3.12 tools/pb/plan.py close M0-TM3 --status "DONE" --text "<handoff>" --set "Model ratings=2026-10-11
  by M0-TM3"`, then checkdiff.py again. Results are below, appended after the runs.
- **Claim run**, the last step, after the close: `py -3.12 tools/pb/verify.py --changed --base ab36299 --task M0-TM3`.
  v12.2's `plan.py show` printed no `baseline:` line, so the base is named (R10). ab36299 is the roll-out's commit, the
  last before this task; M0-TE-V12.3's edits are uncommitted in the same tree, as its flag foresaw. The result is below.

### Results, appended after the runs
- **The close** (11:56): `close: checks=4 failed=0 warnings=0 · handoff_lines=1 · flow=per-task · set=1 · replaced=1` —
  GO.
- **checkdiff.py** after the close: [ALREADY RUN — PASS (7 rating segments, 1 register line `Model ratings: 2026-10-11 by
  M0-TM3`, 3 own lines — Status, Handoff, Flow row — 0 other) on win-laptop].
- **The claim run** — `py -3.12 tools/pb/verify.py --changed --base ab36299 --task M0-TM3`, the last step:
  [ALREADY RUN — PASS (GO: 505 passed, 0 failed, plan_lint 1/1 scope, 19 scopes not impacted, 0.23 s; tree
  ab36299eb49e+82ef1ec0569c; python 3.12.10; load 14.69 → 14.06) on win-laptop].
  - Its one FLAG reads `FLAG type: tools/pb/verify.anchor.json changed since ab36299eb49e - a PLAN block writes no product
    code and no tool`. This is the harness's own write. `git diff` shows 2 lines out and 2 in: the plan's `plan_lint` entry
    now names this run (task M0-TM3), and m0_rules.md's names M0-TE-V12.3's. The file is in the manifest's skip class
    "the harness's own manifest and anchor". No tool was written; M0-TM2 and M0-TE-V12.3 recorded the same flag.
  - lint's count reads 505 after the close, against 506 before it (M0-TM2 saw 496 → 495 the same way).
- **Files this task wrote**:
  - the plan: 7 rating segments, its own Status, Handoff and Flow row, and the register's `Model ratings:` line;
  - this file and logs/M0-TM3.log.
  - verify.py wrote logs/M0-TM3.plan_lint.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
  - Only this section was written after the claim run. It is a skip-class path, graded by no scope.
