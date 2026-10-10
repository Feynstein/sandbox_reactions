# M0-TM2 — the rating pass (2026-10-10, win-laptop)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch
prompt), the heading's rung, no mismatch. Raw runs: logs/M0-TM2.log, plus logs/M0-TM2.plan_lint*.log written by
verify.py. Read-only helpers (never run on the plan's behalf, never in the repo): blocks.py (each open block's
heading, Sizing, Carried flags, Deliver, Verify) and checkdiff.py (the diff against a snapshot, line by line), in
the session scratchpad's tm2/.

## What fired it
M0-V2's Phase 4 close (reports/v2.md § Phase 4's close): rule (4) moved BUILD on Sonnet 5.5, high — $1.70 per DONE
block against $1.44 on (usual), 10 blocks each. The lint's refresh count read 2/5 (M0-V2's flags into M0-T27 and
M0-T32), under its threshold.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-295_agent`. The agent is
  still Claude Code. `CLAUDE_EFFORT` holds the picker's launch level, a hint; the switch ran this session's requests
  at max (`rung_record.py now`).
- **The model pages**: the claude-api skill bundled with Claude Code 2.1.295 on win-laptop — its "Current Models
  (cached: 2026-10-06)" table and `shared/models.md` (sha256 1601794b…a2ec). It is the same bundle version and cache
  date M0-TM1 read on linux-pc on 2026-10-09; the other copies on this box (2.1.187, 2.1.210) are older.
  - Opus 5.5: $4 / $20 per MTok, effort low–max (default medium). Unchanged.
  - Sonnet 5.5: $2 / $10 per MTok, effort low–max (default high). Unchanged.
  - No model retired or renamed. The table also lists Fable 5.1 and Mythos 5.1 (Project Glasswing only) at
    $10 / $50, and Haiku 5.5 (added at M0-TM1's read). None is a rung.
- **Rule (4)'s price ratio**: $10 / $20 = 0.5. It is not used here, because the costs were read from session
  files, not list prices.
- **The result**: no rung retired, renamed, repriced or relevelled. The `Models:` line is re-dated (read
  2026-10-09 → 2026-10-10). Its cache date and four rungs are byte-identical. No question went to the lead,
  because no rung they run changed (§0), and `RUNGS` needs no row.

## The record
`py -3.12 tools/pb/rung_record.py report` (2026-10-10 14:3x, win-laptop):
```
report: 51 DONE blocks · 10 class/rung rows · 1 session folder(s) · 28 block(s) with no session file readable (rung from the handoff, cost null) · 0 block(s) priced from tokens (a session file with no cost line) · 0 priced from list prices · 3 D block(s) naming no cause in the record · $58.68 read from the sessions' own cost
BUILD on Opus 5.5, high: 11 block(s) · first-try 6/11 · $1.45 per DONE block · usual
BUILD on Sonnet 5.5, high: 10 block(s) · first-try 8/10 · $1.85 per DONE block · costs more on Sonnet 5.5, high · vs same class on (usual)
BUILD on unknown: 17 block(s) · first-try 7/12 · n/a per DONE block · off the ladder
BUILD·E on Sonnet 5.5, medium: 1 block(s) · first-try 1/1 · $1.15 per DONE block · n<10
BUILD·E on unknown: 2 block(s) · first-try 2/2 · n/a per DONE block · off the ladder
CHECK on Opus 5.5, max: 1 block(s) · first-try 1/1 · $16.30 per DONE block · above (usual)
CHECK on unknown: 1 block(s) · first-try 1/1 · n/a per DONE block · off the ladder
MOVE on unknown: 1 block(s) · first-try n/a · n/a per DONE block · off the ladder
PLAN on Opus 5.5, max: 2 block(s) · first-try 1/2 · $9.73 per DONE block · above (usual)
PLAN on unknown: 5 block(s) · first-try 2/2 · n/a per DONE block · off the ladder
=== GO ===
```
- **Rule (4) moves BUILD on Sonnet 5.5, high, read again today.** The class has 10 blocks below (usual), at $1.85
  per DONE block. BUILD on (usual) has 11 blocks, 10 of them with a session cost, at $1.45.
  - M0-V2 read $1.70. The difference is M0-D17's $1.52: M0-D17 is now DONE, and it is traced to M0-T15.
  - The costs come from session files (win-laptop's one folder), so the cost comparison applies, not the list-price
    branch.
  - One move takes the class one rung above (3)'s rung: Sonnet 5.5, high → Opus 5.5, high, which is (usual), the cap.
- **BUILD·E on Sonnet 5.5, medium** has 1 block (M0-D13), fewer than 10. It does not move.
- **No other class** has ≥10 blocks below (usual).
- **Rule (1)** escalates no open block: no open block's `Carried flags:` holds a trail.

## The rating: every open block, 128 of them
**Method.** I read each open block's heading, `Sizing:`, `Carried flags:`, `Deliver:` and `Verify:` (and `Pass:`
where a block has one), and applied §0's rubric, first match wins. I read the closed list as M0-TB did
(reports/size_pass.md §9) and M0-TM1 kept it:
- (a) a FROZEN passage, a schema, a persisted format or a migration changed
- (b) a D whose cause is not measured
- (c) a design the spec leaves open
- (d) secrets, money or data loss
- (e) numeric or precision semantics, or concurrency, as the block's own acceptance
- (f) a library's internals read to find a mechanism

A block that reuses a mechanism an earlier block built does not earn the reason again. The contract's FROZEN
passages are unchanged since M0-TM1 (§0.2, §1.2, §1.4.1, §1.6.4, §1.8.6, §6.2.4). No open BUILD block changes one;
several implement or test one, which is not a change.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) |
|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 16 | V3–V17, TW (the CHECK scribe) |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM2, TJ1*, TJ2*, TZ |
| Opus 5.5, high (usual) | (3)'s list → (5) | 12 | T18, T19, T20, T21, T22, T26, T30, T34, T42, T46, T51, T56 |
| Opus 5.5, high (usual) | (4), the record — BUILD on Sonnet 5.5, high moved once | 89 | the list below |
| Sonnet 5.5, high | (3), no reason from the list | 0 | — |
| Sonnet 5.5, medium | (3), `Sizing: E` — BUILD·E's class not moved (n = 1) | 7 | T39, T48, T52b, T91, T93a, T94, T111 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its
` · switch`.

### The 101 BUILD blocks above the cheapest rung that fits, each with its rule
The cheapest rung that fits a BUILD block without `Sizing: E` is rule (3)'s Sonnet 5.5, high. All 101 sit one rung
above it, on (usual).

**Rule (4), the record — 89 blocks, no reason from (3)'s list, re-rated by this pass** (Sonnet 5.5, high →
Opus 5.5, high): T23, T24, T25, T27, T28, T29, T31, T32, T33a, T33b, T35, T36, T37, T38, T40, T41, T43, T44a, T44b,
T45, T47, T49, T50, T52a, T53, T54, T55, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70,
T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, TV2, T89, T90, T92,
T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107, T98b, T99b, T100, T101, T108, T109,
T110, T112, T113, T114*, T115, TD, T116.

**Rule (3)'s list → rule (5) — 12 blocks, unchanged.** Each reason was re-derived from the block's own text:

| Block | Reason | From the block's own text |
|---|---|---|
| M0-T18 | (c), (e) | "one booking layout every pass writes its §2.9 terms into" — every later pass books into it; "f32, no atomics on a value the state depends on (§1.3.4)", f64 fixed-order sums, "two runs bit-identical" |
| M0-T19 | (c) | eos.wgsl, "the helpers every later shader shares" — WGSL has no includes, so how shaders share code is open |
| M0-T20 | (e) | "a fixed-order tree reduction (§1.3.4)", Δt "bit-identical over two runs" |
| M0-T21 | (e) | the box "written on the GPU as every pass's indirect dispatch sizes — never read back to decide a dispatch" |
| M0-T22 | (e) | "a u32 flag a pass sets with atomicOr", a controller dispatch zeroing every later indirect dispatch of the frame, `poll()` "never blocking" |
| M0-T26 | (e) | "2 × 100 steps through a dump equal 200 straight steps, bit-identical (§1.3.4)" — every bit of hidden state round-trips |
| M0-T30 | (c), (e) | "a multi-pass scheme there", "as few dispatches as the limits allow" — the GPU FFT left to the builder; "≤ 10⁻⁵ relative L2 in f32" |
| M0-T34 | (c), (e) | "fused into as few dispatches as the limits allow" "within 8 storage buffers per stage"; M0-T33b's scheme in f32 |
| M0-T42 | (e) | "RKL2's s stages as dispatches with coefficients frozen at P5's start", in f32 |
| M0-T46 | (e) | "M0-T44b's sub-cycles per cell in f32", f(T) within 10⁻⁴ |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)" |
| M0-T56 | (e) | every §1.9.1 field reduced in fixed order, "read back asynchronously (≤ 2 frames late)" |

Today the (4)/(5) split decides no rung: both land on (usual). It matters only if a later record no longer moves the
class. Then the rubric would return the 89 to Sonnet 5.5, high and keep the 12 on (usual).

### The flags, read for the rating
- **M0-T27** `[M0-V2]`: testing.md and running.md lines gone stale. This is a docs edit, with no reason from the list.
- **M0-T32** `[M0-V2]`: one line in physics.md's « Not checked at load » list. This is a docs edit, with no reason.
- **M0-T19** `[M0-T3]` and **M0-T25** `[M0-T4]`: read as M0-TM1 read them. §2.7 is not FROZEN, and the scene format
  is §2.12.1's, implemented and not changed. They add no reason.
- **M0-T98a, M0-T99a** `[M0-TB]`: G-CAL's refusal during tuning. This is an instruction for the claim run, not a
  reason.
- The flags on M0-V3 and M0-TZ sit on a CHECK and a PLAN block. Rule (2) pins those ratings.

### Considered — observations, never defects (§0, no step up on a guess)
- **The E blocks** keep the ladder's lowest rung. None of them carries a reason, and their class has one block on
  record.
  - M0-T93a: its asynchronous one-cell readback reuses M0-T56's.
  - M0-T52b: the dump's `sinks` list is in SRDUMP01's header from M0-T26 (§2.12.3), and no format changes.
  - M0-T48: G-REF's third case copies M0-T38's shape.
  - M0-T111: G-LATCH's scenario reuses M0-T22's latch.
  - M0-T94: the web build's parameters, plus one smoke case.
- **M0-T66 and M0-T95** write into the calibration file. I weighed (d), data loss. The file is regenerable by
  `calibrate` and tracked in git, and their tests write a partial file (T66) or a copy (T95). The real runs are the
  lead's. No reason.
- **M0-T38 and M0-T48** test GPU-against-twin tolerances. The mechanisms are built earlier (M0-T34, M0-T46), and a
  red is a D, so (e) is not earned again.
- M0-TM1's other observations still hold for the open blocks:
  - M0-T110: it tests the determinism T18, T20, T22 and T26 build.
  - M0-T114: it reuses M0-T21's GPU-side dispatch sizes.
  - M0-T23: timestamp-query is public API.
  - M0-T65: SHA-256 is written against FIPS 180-4's vectors, and it holds no secret.
  - M0-T98a, M0-T99a: "judgment-heavy" tuning is not on the list.

### The record, read for the lead — what the comparison holds (rule (4) applied as written)
- **The two rows hold different kinds of block.**
  - The (usual) row is 10 D blocks (M0-D2–D8, D15, D16, D17), small fixes, plus M0-T2, which has no session cost.
  - The Sonnet row is 9 feature T blocks (M0-T1, T10–T17) plus M0-TE-win, a box setup with installs.
- **Two blocks carry the Sonnet row's total of $18.52.**
  - M0-T15 costs $6.24: two sessions, escalated to Opus 5.5, high, plus M0-D17 traced to it.
  - M0-TE-win costs $3.59.
  - Without M0-T15, the Sonnet mean is $1.36. The medians read the other way: Sonnet $1.12, (usual) $1.42.
  - First try on the claim run: 8/10 on Sonnet against 6/11 on (usual).
- **linux-pc's 28 DONE blocks are unreadable from this box.** Among them are M0-T4, T6, T7, T8, T9 and TV1 (headings
  on Sonnet 5.5, high), and the D blocks traced to them (M0-D10, D11, D12, D13, D15). Their costs would join the
  Sonnet row, and they are not in it.
- **What comes next.** With the 89 on (usual), lot 3's T blocks add feature builds to the (usual) row. If a later
  phase-closing V's table no longer shows the move, the rubric returns the 89 to Sonnet 5.5, high at the next rating
  pass. Whether that alone files a TM is that V's reading of §0's triggers.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and
  the rating is kept.

## What changed in the plan
- **Ratings: 89 changed.** By rung:
  - Opus 5.5, max: 20 (0 changed).
  - Opus 5.5, high: 101 (+89).
  - Sonnet 5.5, high: 0 (−89).
  - Sonnet 5.5, medium: 7 (0 changed).
- **The `Models:` line** is re-dated: read 2026-10-09 → 2026-10-10. The cache date and the rungs are
  byte-identical.
- **The register**: `Model ratings: 2026-10-10 by M0-TM2`, written through `plan.py close --set`.
- **How the edits were made.** plan.py has no call that rewrites a rating; its calls are show, status, close,
  handoff, flag, append, move, stub, register and lint. So the 89 rating segments and the `Models:` line were
  edited with the file-edit tool, one heading per edit, the plan's own routine. Each `old_string` was the
  heading's ` · **BUILD** · Sonnet 5.5, high · switch · (AFTER …)`, checked unique in the file first, so no DONE
  block's heading could be hit.
- **The diff**, checked by checkdiff.py against a snapshot taken right after `plan.py status`:
  - 89 headings differ only in `Sonnet 5.5, high` → `Opus 5.5, high`.
  - 1 `Models:` line differs only in its read date.
  - 0 other changes before the close.
  - The close then adds my own Status, Handoff and Flow row (plan.py syncs the row) and the register's
    `Model ratings:` line.

## Measured and not acted on
- **Pipeline state's `Next task: M0-TM2`** goes stale once TM2 closes; the next task is M0-T18. TM2's diff is
  bounded to rating segments, the `Models:` line and the register's `Model ratings:` line (M0-TM1's precedent), so
  the next close sets it.
- **Repo facts' "Claude Code 2.1.294"** on win-laptop: the probe reads 2.1.295 today. The switch holds: `plugin=installed`,
  and the session ran on the rung's model and level. The update belongs to the next block that edits Repo facts.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the picker's level, which the
  switch overrides, as designed (§B.3).
- **The lint's 8 WARNs**: Phase 3's eight late DONE blocks (M0-TC1, M0-D10–D16) are unstubbed. They are outside
  TM2's diff and belong to a TC (M0-V2's observation 10).
- **Haiku 5.5 as a rung** is still a ladder change only by the lead's word (M0-TM1).

## The routine, on win-laptop (box Laser2025-20)
- **Pass 1, `py -3.12 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**, after the 89 edits
  and the `Models:` line: [ALREADY RUN — PASS (GO, 496 checks, 0 failed, 8 pre-existing stub WARNs, refresh 2/5) on
  win-laptop].
- **`py -3.12 tools/pb/verify.py plan_lint --task M0-TM2`**: [ALREADY RUN — PASS (496 passed, 1/1 scope, 0.28 s) on
  win-laptop].
- **`py -3.12 tools/pb/verify.py --redarm plan_lint --task M0-TM2`**: [ALREADY RUN — PASS (clean GO, plant
  plan-bad-status red, 5.39 s) on win-laptop].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (89 headings, 1 `Models:` line, 0 other)
  on win-laptop].
- **The close**, `py -3.12 tools/pb/plan.py close M0-TM2 --status "DONE" --text "<handoff>" --set "Model ratings=2026-10-10
  by M0-TM2"` (14:52): GO. checkdiff.py afterwards: 89 headings, 1 `Models:` line, 4 own or register lines (Status,
  Handoff, Flow row, `Model ratings:`), 0 other.
- **Pass 1 on the final plan, `plan.py lint`**, after the close and before the claim run: [ALREADY RUN — PASS (GO, 495
  checks, 0 failed, refresh 0/5) on win-laptop].
  - It shows 11 WARNs, the 8 above plus M0-V2, M0-D17 and M0-TM2. Phase 4 is now all DONE, so its whole blocks read
    as stub candidates. That is a TC's job (§2.6), outside TM2's diff.
- **Claim run, `py -3.12 tools/pb/verify.py --changed --base aa2e690 --task M0-TM2`**, the last step, after the close:
  [ALREADY RUN — PASS (GO, 495 passed, 0 failed, 1/1 scope (plan_lint), 14 not impacted, 0.31 s) on win-laptop].
  - Its one FLAG names tools/pb/verify.anchor.json. That is verify.py's own coverage anchor: this task's runs
    rewrote the plan's `plan_lint` entry (task M0-TM2, tree aa2e690b4fc8+4d4050061a43). It is not a TM2 write; M0-TM1
    and M0-V2 (observation 11) recorded the same.
- **Files this task wrote**: the plan (89 rating segments, the `Models:` line, its own Status, Handoff and Flow row,
  the register's `Model ratings:` line), this file, and logs/M0-TM2.log. verify.py wrote
  logs/M0-TM2.plan_lint.log, logs/M0-TM2.plan_lint.redarm.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
