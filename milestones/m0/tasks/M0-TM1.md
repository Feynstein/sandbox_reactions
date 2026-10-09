# M0-TM1 — the rating pass (2026-10-09, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the
launch prompt), the heading's rung, no mismatch. Raw runs: logs/M0-TM1.log, plus
logs/M0-TM1.plan_lint*.log written by verify.py. Read-only helpers (never run on the plan's behalf, never
in the repo): blocks.py (each open block's Sizing, Carried flags, Deliver, Verify) and ratings.py (the tally),
in the session scratchpad's tm1/.

## What fired it
`plan.py lint`'s refresh count, 5/5: five carried flags on open BUILD blocks, each stamped 2026-10-09, after
the register's `Model ratings: 2026-10-08 by M0-TB`:
- M0-T5 `[M0-T2]`: eframe's wgpu path and cargo feature unification; `--adapter` through `match_adapter`.
- M0-T9 `[M0-T3]` and `[M0-T4]`: testing.md rows for the `state` and `boot` scopes.
- M0-T19 `[M0-T3]`: the P8 order, a §2.7 default stated by asking, the CPU f64 twin.
- M0-T25 `[M0-T4]`: scene files plugged into `check_scene` / `run` in headless.rs.

M0-V1's `[M0-D9]` flag sits on a CHECK block. Rule (2) pins its rating, so it does not count.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=high`, `AI_AGENT=claude-code_2-1-295_agent`. The
  agent is still Claude Code. `CLAUDE_EFFORT` holds the launch level. The switch ran this session's requests
  at max (`rung_record.py now`), and the environment is a hint, never the session's own value.
- **The model pages**: the claude-api skill's model table, now cached 2026-10-06 (Claude Code 2.1.295's
  bundled copy). I diffed its `shared/models.md` against the copy that 2.1.292 bundled, which was still on disk
  (extracted 2026-10-08, the copy the ladder's last read used). There is one change: **Claude Haiku 5.5
  added**. It costs $0.10 / $0.50 per MTok for prompts up to 100K and $0.50 / $2.50 above, with levels
  low–max and medium the default.
  - Opus 5.5 is unchanged: $4 / $20, levels low–max.
  - Sonnet 5.5 is unchanged: $2 / $10, levels low–max, high the default.
  - Fable 5.1 ($10 / $50) was already listed. M0-TP's RT22 ruled on it.
  - No model was retired or renamed.
- **Rule (4)'s price ratio**: Sonnet 5.5 over Opus 5.5, per output token, is $10 / $20 = 0.5, and 1 between
  two levels of one model. The lead's "half Opus's price" (bootstrap Q7) still holds.
- **The result**: a model was added, and no rung was retired, renamed, repriced or relevelled. So the
  `Models:` line is re-dated (read 2026-10-09, cache 2026-10-06) and its four rungs are unchanged. No
  question went to the lead: no rung they run changes (§0), and RT22 is the precedent for an added model
  that is not a rung.
- **Haiku 5.5 as a rung** is the lead's ladder call (§0: "a ladder change … the lead's word"). It needs a
  `RUNGS` row in tools/pb/switch.mjs (§A.10) and a reinstall on each box before any block could run on it.

## The record
`rung_record.py report` (2026-10-09, linux-pc):
```
report: 25 DONE blocks · 6 class/rung rows · 2 session folder(s) · 10 block(s) with no session file readable (rung from the handoff, cost null) · 0 block(s) priced from tokens (a session file with no cost line) · 0 priced from list prices · 1 D block(s) naming no cause in the record · $110.87 read from the sessions' own cost
BUILD on Opus 5.5, high: 6 block(s) · first-try 1/2 · $4.05 per DONE block · usual
BUILD on Opus 5.5, max: 1 block(s) · first-try n/a · $3.29 per DONE block · above (usual)
BUILD on Sonnet 5.5, high: 3 block(s) · first-try 1/3 · $2.00 per DONE block · n<10
BUILD on unknown: 9 block(s) · first-try 5/9 · n/a per DONE block · off the ladder
PLAN on Opus 5.5, max: 5 block(s) · first-try 2/2 · $16.12 per DONE block · above (usual)
PLAN on Sonnet 5.5, high: 1 block(s) · first-try 0/1 · n/a per DONE block · n<10
=== GO ===
```
- No phase-closing V has run yet; M0-V1 will keep the first table.
- **Rule (4)** moves no class. No class has ≥10 blocks on a rung below (usual): BUILD has 3 blocks on Sonnet
  5.5, high and none on Sonnet 5.5, medium.
- **Rule (1)** escalates no open block. No open block's `Carried flags:` holds a trail. M0-T2's trail is on a
  DONE block, which its own session re-rated to (usual).
- **Observation, not a step up** (§0): BUILD on Sonnet 5.5, high reads first-try 1/3. One of the two misses
  is M0-T2's Sonnet try, which went red on M0-T1's defect (M0-D9), not on the rung. The record decides once
  the class has ≥10 blocks.

## The rating: every open block, 144 of them
**Method.** I read each open block's heading, `Sizing:`, `Carried flags:`, `Deliver:` and `Verify:`, and
applied §0's rubric, first match wins. I read the closed list as M0-TB did (reports/size_pass.md §9):
- (a) a FROZEN passage, a schema, a persisted format or a migration changed
- (b) a D whose cause is not measured
- (c) a design the spec leaves open
- (d) secrets, money or data loss
- (e) numeric or precision semantics, or concurrency, as the block's own acceptance
- (f) a library's internals read to find a mechanism

A block that reuses a mechanism an earlier block built does not earn the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) |
|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 18 | V1–V17, TW (the CHECK scribe) |
| Opus 5.5, max (gate) | (2), PLAN | 4 | TM1, TJ1*, TJ2*, TZ |
| Opus 5.5, high (usual) | (3)'s list → (5) | 13 | the table below |
| Sonnet 5.5, high | (3), no reason from the list | 102 | T6, T7, T8, T9, TV1, T10, T11, T12, T13, T14, T15, T16, T17, T23, T24, T25, T27, T28, T29, T31, T32, T33a, T33b, T35, T36, T37, T38, T40, T41, T43, T44a, T44b, T45, T47, T49, T50, T52a, T53, T54, T55, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107, T98b, T99b, T100, T101, T108, T109, T110, T112, T113, T114*, T115, TD, T116 |
| Sonnet 5.5, medium | (3), `Sizing: E` | 7 | T39, T48, T52b, T91, T93a, T94, T111 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps
its ` · switch`.

### The 13 blocks above the cheapest rung that fits, each with its rule
| Block | Rule (3)'s reason → rule (5) | From the block's own text |
|---|---|---|
| M0-T5 | (f), (e) | Inside eframe 0.36's wgpu backend: "ready" once the first frame is presented, and `--adapter` into its wgpu setup. M0-T2's flag adds wgpu's webgl/gles features coming back through cargo feature unification. `--offscreen-window` needs a window outside every monitor, never activated, out of the taskbar. eframe's and winit's public APIs do not show these mechanisms. A status thread shares the ready state. |
| M0-T18 | (c), (e) | "one booking layout every pass writes" — every later pass books into it; f32 with no atomics on state, f64 fixed-order sums, two runs bit-identical |
| M0-T19 | (c) | eos.wgsl, "the helpers every later shader shares": WGSL has no includes. M0-T3 built one shader and no sharing (tasks/M0-T3.md), so the design is still open here. |
| M0-T20 | (e) | "a fixed-order tree reduction (§1.3.4)", bit-identical over two runs |
| M0-T21 | (e) | the box "written on the GPU as every pass's indirect dispatch sizes — never read back" |
| M0-T22 | (e) | a flag set with atomicOr, a controller dispatch zeroing every later indirect dispatch of the frame, `poll()` never blocking |
| M0-T26 | (e) | "2 × 100 steps through a dump equal 200 straight steps, bit-identical" — every bit of hidden state round-trips |
| M0-T30 | (c), (e) | "a multi-pass scheme there", "as few dispatches as the limits allow" — the GPU FFT left to the builder; f32 within 10⁻⁵ of the twin |
| M0-T34 | (c), (e) | "fused into as few dispatches as the limits allow" within 8 storage buffers per stage; M0-T33b's scheme in f32 |
| M0-T42 | (e) | RKL2's s stages in f32, coefficients frozen at P5's start |
| M0-T46 | (e) | M0-T44b's backward-Euler sub-cycles per cell in f32, within 10⁻⁴ of analytic |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)" |
| M0-T56 | (e) | every §1.9.1 field reduced in fixed order, "read back asynchronously (≤ 2 frames late)" |

### The five flags, read for the rating
- **M0-T5** `[M0-T2]`: the flag adds to (f). The block stays on (usual).
- **M0-T9** `[M0-T3]`, `[M0-T4]`: two testing.md rows, a docs edit with no reason from the list. The block
  stays on Sonnet 5.5, high.
- **M0-T19** `[M0-T3]`: the P8 order, a §2.7 default stated by asking, and the CPU f64 twin. §2.7 is not
  FROZEN (m0_contrat.md marks §0.2, §1.2, §1.4.1, §1.6.4, §1.8.6 and §6.2.4), so (a) does not apply, and no
  other letter is added. The block stays on (usual) by (c).
- **M0-T25** `[M0-T4]`: plumbing into headless.rs (`check_scene`, `run`, `sun_disk_planes`). The scene
  format is §2.12.1's, implemented and not changed. The block stays on Sonnet 5.5, high.

### Considered and left on the cheaper rung
These are observations, never defects (§0, no step up on a guess).
- **M0-T93a** (E): its acceptance is an asynchronous one-cell readback, which is (e) as M0-TB reads it. But
  M0-T56 builds the asynchronous readback (≤ 2 frames) first, so it is reused and earns no reason.
- **M0-T52b** (E): the dump's `sinks` list is in SRDUMP01's header from M0-T26 (§2.12.3). The block
  completes it; no persisted format changes.
- **M0-T110**: bit-identical dumps across rungs. It tests the determinism M0-T18, T20, T22 and T26 build,
  and builds no mechanism of its own.
- **M0-T114** (DEFERRED): its k = 4 switch, decided from the state, reuses M0-T21's GPU-side dispatch sizes.
- **M0-T7, M0-T8**: the first-presented-frame flag and the off-screen window come from M0-T5 (one App type,
  M0-TB's flags). eframe's web runner and its viewport screenshot are public API.
- **M0-T23**: pass-boundary timestamp writes are wgpu's public API (`timestamp-query`).
- **M0-T65**: SHA-256 is written against FIPS 180-4's test vectors, so it is an algorithm with a reference.
  It is a content hash and holds no secret.
- **M0-T98a, M0-T99a**: tuning. "Judgment-heavy" is not on the list (M0-TB's reading, kept).
- M0-TB's other observations (reports/size_pass.md §9) were re-read and still hold.

## What changed in the plan
- **Ratings: 0 changed.** Opus 5.5, max 0 · Opus 5.5, high 0 · Sonnet 5.5, high 0 · Sonnet 5.5, medium 0.
- **The `Models:` line** is re-dated: read 2026-10-08 → 2026-10-09, and the cache 2026-09-25 → 2026-10-06.
  The rungs are byte-identical.
- **The register**: `Model ratings: 2026-10-09 by M0-TM1`, written through `plan.py close --set`.
- **The diff**, against a snapshot taken after `plan.py status`, holds only the `Models:` line, the register's
  `Model ratings:` line, my own Status and Handoff, and my Flow row, which plan.py syncs. I rehearsed it first
  on a scratch copy made by tools/pb/scratch_copy.sh, at
  /tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/b6304831-a28f-43d2-a238-f22dbc8974d9/scratchpad/tm1/tree
  (255 files, 3.9 MB, nothing linked; a throwaway in the session scratchpad).

## Measured and not acted on
- **Pipeline state's "Next task: M0-T4"**: M0-T4 is DONE (09:40), and the next task is M0-T5. TM1's diff is
  limited to rating segments, the `Models:` line and the register's `Model ratings:` line, so the next close
  sets it.
- **Repo facts' "Claude Code 2.1.292"** on linux-pc: the probe reads 2.1.295 today. The switch still holds:
  `rung_record.py now` reads the rung's model and level, `plugin=installed`. The update belongs to the next
  block that edits Repo facts (a V's close, or TZ).
- **Haiku 5.5**: a ladder change only by the lead's word (above).
- **M0-T9's and M0-T25's M0-T4 flags** carry the stamp twice (`[M0-T4, 2026-10-09] [M0-T4, 2026-10-09]`).
  This is cosmetic and outside TM1's diff.

## The routine, on linux-pc (box laserax-ai)
- **Pass 1, `plan.py lint`**: [ALREADY RUN — PASS (GO, 492 checks, 0 failed, 9 pre-existing stub warnings,
  refresh 5/5 before the register line) on linux-pc].
- **`verify.py plan_lint --task M0-TM1`**: [ALREADY RUN — PASS (492 passed, 1/1 scope) on linux-pc].
- **`verify.py --redarm plan_lint --task M0-TM1`**: [ALREADY RUN — PASS (clean GO, plant plan-bad-status red)
  on linux-pc].
- **Claim run, `verify.py --changed --base 516ec7c --task M0-TM1`**, the last step, after the close:
  [ALREADY RUN — PASS (GO, 510 passed, 0 failed, 5/5 scopes, 2 not impacted, 4.99 s) on linux-pc].
  - Its FLAG lines name tools/pb/launch.json and tools/pb/verify.json, and the scopes `adapter`, `state` and
    `boot` added to verify.json, all changed since 516ec7c. That is M0-T2's, M0-T3's and M0-T4's uncommitted
    work: the files' mtimes, 09:38:59 and 09:39:09, predate this session's first action at 09:46:03 (local
    time). The remaining FLAG names verify.anchor.json, which verify.py advanced itself (a skip class).
  - None of these is a TM1 write. The outstanding commit gate carries them.
- **Pass 1 on the final plan, `plan.py lint`**, after the close and before the claim run: [ALREADY RUN — PASS
  (GO, 491 checks, 0 failed, refresh 0/5) on linux-pc].
