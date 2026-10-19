# M0-TM9 — the rating pass (2026-10-19, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch prompt right after
the heading's grep). That is the heading's rung, so no mismatch. Box: linux-pc (laserax-ai, Linux 7.0.0-38-generic). Raw
runs: logs/M0-TM9.log, plus the logs verify.py writes. Read-only helpers sit in the session scratchpad
(`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/e08fd4db-98c5-47a7-8b83-f9b3ba24e802/scratchpad/tm9/`). They
are never in the repo and never write the plan:
- blocks.py: each open block's heading, type, rating, switch, status and `Sizing:`; `diff <rev>` compares the open blocks
  against an older plan (identical / new / changed, with the changed lines), `full <id> [--rev]` prints a block;
- checkdiff.py: the diff against a snapshot, each changed line classed rating · models · register · own · other (GO when
  other = 0). Red-tested on a mutant (M0-T67's rating changed, M0-T68's ` · switch` dropped, M0-T67's `Read:` line edited):
  rating=1, other=2, NO-GO, exit 1. Its control (the `Models:` line alone): models=1, GO;
- scan.py: rule (1)'s trail words over the open blocks.

## What fired it
M0-T66 filed it on 2026-10-19, when it went BLOCKED on M0-R4 and `plan_lint` read the refresh at **7 of 5** (Rules: `Rating
refresh: 5 flags`). These are the carried flags on open BUILD blocks stamped after `Model ratings: 2026-10-16 by M0-TM8`, as
the lint counts them:
- M0-T66 `[M0-T65]` (one flag, its stamp written twice; the block's `Deliver:` reads it as "M0-T65's two"): write the file
  with `Calibration::to_json` (keys sorted) and the hash with `physics_hash()`; `Calibration::parse` wants the complete file,
  so `--only`'s partial is plain JSON; a run with overrides never writes; `calibrate` itself needs no calibration.
- M0-R4 `[M0-T66]`: M0-T66's measurements. As written, §2.11 items 1–2 have no bracket on the GPU: the a = 12 disk falls to
  ~0.28 R₀ and swells to ~2.8 R₀ at every mass from 30 to 12,000, and the SCF equilibrium swells at every mass. The harness is
  parked in tasks/M0-T66.calibrate.rs.txt.
- M0-R4 `[M0-T65]`: M0-T66's file flags pass to whichever block rebuilds `calibrate`.
- M0-T67 `[M0-T66]`: G-CORE's four cases cannot pass as written. M0-R4's ruling re-scopes the block, which never runs as it
  stands.
- M0-T70 `[M0-D50, 2026-10-18]`: on M0-D50's constants the core settles near T_c 36. Whether helium lights before X_H,c 0.35 is
  unmeasured. X_H,c reaches 0.35 near ~1,200 t.u. (UNVERIFIED).
- M0-T97 `[M0-D55, 2026-10-18]`: at a_r 4 × 10⁻⁵, heat_hot's one P5 step keeps 2.657 × the heat its refinement keeps, so a
  burnt star's cooling per step is under-resolved.
- M0-T98a `[M0-D50, 2026-10-18]`: start from M0-D50's W2. The margins are measured (A_H × 1.4 holds and × 2 runs away; the
  ladder holds at ± 4 %; a_r × 30 runs away and × 100 holds). τ_nuc/τ_KH ≈ 26 and τ_KH ≈ 134 t.u. (UNVERIFIED). Keep
  `sun_burn` GO.

Also read, though the lint does not count them:
- **M0-V8's flags, stamped 2026-10-16.** That is the day of M0-TM8's line, so §0 counts them as "there to read", but they
  were appended after M0-TM8 closed (10:58; M0-V8 ran from 11:17).
  - M0-T103: the displayed stages depend on the frame's size, and the tracker's identity loses a shrinking cold_disk's history
    at coarse frames.
  - M0-T106: the ledger's W leaves out the sinks' own M·φ, while §2.9 reads "sinks included".
  - M0-V9 (CHECK): owed on win-laptop.
- **M0-TZ `[M0-TJ2, 2026-10-18]`** (PLAN): the player's choice of a life's pace, and further optimisation, routed past M0.
- **M0-TJ2's rewrites (2026-10-18)**: TOP becomes Max, and performance is reported, never graded. The rewrites touched
  M0-T75, T85, TV2, T98a, T109, T110, T111, T112, T113, T115, V16 and V17. Their `Deliver:`, `Verify:` or `Read:` text changed,
  so each is re-derived. M0-TJ2 also filed M0-T124, rated (usual) by its filer.

## The ladder, re-read on the day
- **The probe** (§B.3): `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-301_agent`.
  - Claude Code is 2.1.301, the version the `Models:` line cites.
  - `CLAUDE_EFFORT` is the process's launch level, a hint. The switch ran this session at max (`now`).
- **The model pages: read.** The Skill tool loaded the claude-api skill from 2.1.301's bundle. Its "Current Models (cached:
  2026-10-06)" table:
  - Prices per MTok: Fable 5.1 $10 / $50 · Opus 5.5 $4 / $20 · Sonnet 5.5 $2 / $10 · Haiku 5.5 $0.10 / $0.50 for prompts
    ≤ 100K tokens ($0.50 / $2.50 beyond).
  - Effort levels: low…max on all four. The default is medium for Opus 5.5 and Haiku 5.5, high for Sonnet 5.5.
- **shared/models.md.** In 2.1.301's bundle (`/tmp/claude-1000/bundled-skills/2.1.301/…`) it hashes to sha256
  c1af1f093d46fd58e50ef3e8e28d339650f333059bfc49a63d01d0175675161b, raw and with CRs stripped alike. That is the
  c1af1f09…5675161b that M0-TM6, TM7 and TM8 read, so it is byte for byte the same page. All four ladder models are `Active`.
- **The result.** Nothing was retired, renamed, added, repriced or relevelled, and every rung's level is one its model takes.
  No rung the lead runs changed, so no question was owed (§0, §12).
- **The `Models:` line**, re-dated in M0-TM7's and M0-TM8's form:
  - Before: "read 2026-10-16 from the claude-api skill's model table (Claude Code 2.1.301, cached 2026-10-06), the probe and
    the lead's answer (…)".
  - After: "read 2026-10-19 from the claude-api skill's model table (Claude Code 2.1.301, cached 2026-10-06), the probe and
    the lead's answer (…)".
  - The rungs and marks are byte-identical.
- **Not read: the live pricing pages** (the skill's shared/live-sources.md). The ladder cites the skill's table, cached
  2026-10-06. That list prices have not moved since is NOT PROVEN (source: the bundle's cache). No plausible move changes a
  rating:
  - rule (4)'s session-cost branch reads $1.87 against $6.52;
  - its list-price branch moves BUILD on Sonnet 5.5, high only if Sonnet 5.5's output price exceeds 1.27 × Opus 5.5's (40/54 =
    0.741 against 31/53 = 0.585);
  - it moves BUILD·E on Haiku 5.5, high only if Haiku 5.5's exceeds 1.37 × Opus 5.5's (8/10 = 0.80 against 0.585).

## The record
- **The binding record** (§0: the phase-closing V's table) is M0-V8's: reports/v8.md § Phase 10's close (linux-pc,
  2026-10-16). It is the one after M0-TM8's (M0-V7's). No V has closed a phase since; M0-V18 was a triage (E6).
  - BUILD on Sonnet 5.5, high: 53 blocks, 37/50 first try, $1.92 per DONE block.
  - BUILD on Opus 5.5, high (usual): 55 blocks, 31/51, $6.50.
  - Rule (4): ≥ 10 blocks below (usual), at a cost below (usual)'s → **the class does not move**. Every other class below
    (usual) has n < 10. M0-V8: "No TM."
- **Today's read on linux-pc** (`python3 tools/pb/rung_record.py report`, logs/M0-TM9.log): 177 DONE blocks, 13 class/rung
  rows, 76 blocks with no session file readable here, 5 D blocks naming no cause, $447.80 read from the sessions' own cost.
  - BUILD on Sonnet 5.5, high: 57 blocks, 40/54, $1.87. BUILD on Opus 5.5, high: 57, 31/53, $6.52. They agree: no move.
  - BUILD·E on Haiku 5.5, high reaches 10 blocks for the first time: 8/10, $0.82. BUILD·E has 1 block on (usual), under 10,
    so §0 compares it with the plan's (usual) BUILD blocks, $6.52. It is below them, so it does not move. M0-V8's binding table
    has 8 here; the next phase-closing V reads the 10.
  - BUILD·E on Sonnet 5.5, medium: 2. MOVE on Haiku 5.5, high: 4, $0.13. MOVE on Sonnet 5.5, medium: 1. Each has n < 10.
  - BUILD on Opus 5.5, xhigh: 1 block, 0/1, above (usual). No open block's class is read there.
- **Rule (1)**: no open block's text holds an escalation trail.
  - scan.py searched the 70 other open blocks for the trail's words (escalat, trail, NO-GO claim, claim run red, back to TODO,
    rated (usual), failed or fix attempt, try 1/2, second attempt, one rung up or above). There were 2 hits, both M0-TZ's
    test-file name `trail `, as M0-TM8 found.
  - M0-T66 is BLOCKED, not escalated. Its handoff reads "Verify 1–3 NOT RUN (no scope, no command)": there was no claim run,
    so there is no red claim on Sonnet 5.5, high.

## The rating: every open block, 71 of them
**Method.**
1. blocks.py diffed every open block against two commits: 60dc024, the committed text M0-TM8 diffed against, and a0f18d9,
   the first commit after M0-TM8's close (2026-10-16 12:35).
   - **Unchanged since M0-TM8's read: 48.** 47 are byte-identical to 60dc024. M0-T74's only change since then is the
     `[M0-T61]` flag M0-TM8 read, and it is byte-identical to a0f18d9.
   - **New: 3.** M0-TM9, M0-R4 and M0-T124.
   - **Changed after M0-TM8's read: 20.**
     - flags appended: M0-T66 (with its BLOCKED status and handoff), T67, T70, T97, T103, T106, V9 and TZ;
     - M0-TJ2's rewrites: M0-T75, T85, TV2, T109, T110, T111, T112, T113, T115, V16 and V17;
     - both: M0-T98a.
2. The ladder is unchanged and the record moves no class, so an unchanged block keeps M0-TM8's derivation (tasks/M0-TM8.md §
   The rating). That derivation is M0-TM7's and M0-TM4's kinds, re-checked by type, `Sizing:` and flags.
   - Read in full: the 23 new or changed blocks — heading, `Sizing:`, `Carried flags:`, `Deliver:`, `Verify:`, and `Pass:` and
     `Adversarial:` where present.
3. §0's rubric, first match wins. The closed list is read as M0-TB set it (reports/size_pass.md §9) and M0-TM1–TM8 kept it:
   (a) a FROZEN passage, a schema, a persisted format or a migration changed · (b) a D whose cause is not measured · (c) a
   design the spec leaves open · (d) secrets, money or data loss · (e) numeric or precision semantics, or concurrency, as the
   block's own acceptance · (f) a library's internals. A block that reuses a mechanism an earlier block built does not earn
   the reason again.

| Rung | Rule | Count | Blocks (M0-; * DEFERRED) | Changed |
|---|---|---|---|---|
| Opus 5.5, max (gate) | (2), CHECK | 10 | V9–V17, TW (the CHECK scribe) | 0 |
| Opus 5.5, max (gate) | (2), PLAN | 3 | TM9, TJ1*, TZ | 0 |
| Opus 5.5, high (usual) | (3)'s list → (5) | 2 | R4, T124 | 0 |
| Sonnet 5.5, high | (3), no reason from the list | 52 | below | 0 |
| Haiku 5.5, high | (3), `Sizing: E` — the ladder's lowest rung | 4 | T91, T93a, T94, T111 | 0 |
| Fable 5.1, high · Opus 5.5, xhigh · Sonnet 5.5, medium | — | 0 | — | 0 |

No MOVE block and no `LEAD go` block is open. Every rating is a rung of the ladder, and every heading keeps its ` · switch`
(71 of 71).

The 52 on Sonnet 5.5, high: T66 (BLOCKED), T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82,
T83, T84, T85, T86, T87, T88, TV2, T89, T90, T92, T93b, T95, T96, TV3, T97, T98a, T99a, T102, T103, T104, T105, T106, T107,
T98b, T99b, T100, T101, T108, T109, T110, T112, T113, T115, TD, T116. These are M0-TM8's 56 less four: T63, T64 and T65
(DONE) and T114 (N/A, superseded by M0-TJ2).

### The blocks above the cheapest rung that fits, with their rule
- **BUILD: two on Opus 5.5, high (usual)**, each by rule (3)'s list → rule (5). Both are new, and their filers rated them
  there.
  - **M0-R4.** M0-T66 filed it on the lead's word: «Write an Opus R task before T66, thatll decide what we do.»
    - (c) a design the spec leaves open. M0-T66 measured §2.11 items 1–2 unworkable as written: there is no bracket at any
      mass from 30 to 12,000. So what m_ch_sb, m_tov_sb and ns_sigma_max_sb mean on the shipped grid, what G-CORE grades,
      and the replacement contract text are the block's to design, with no reference.
    - (e) numeric semantics, as the block's own subject. It must separate the swelling's cause by measurement: the scheme's
      hydrostatic balance on the twin with the limiter on and off, with heat on, and under a fixed g.
    - The lead's "an Opus R task" is the lead's pick (§0: the heading recommends, the lead picks). The rubric lands on the same
      model, at (usual).
  - **M0-T124.** M0-TJ2 filed it; its own `Rating:` line names (c) and (e).
    - (c): which lever to build is chosen by measurement, and how to build it has no reference: a solve ending at the same
      tests in fewer rounds, fused or reordered dispatches, a cheaper kernel.
    - (e): P5's Newton–PCG solve changes under unchanged tolerances, with every case as before.
    - Its subject is cost, but neither reason is the timing choice the list excludes.
- **The 13 gates on Opus 5.5, max (gate), by rule (2), by type:**
  - CHECK: M0-V9, V10, V11, V12, V13, V14, V15, V16, V17 and M0-TW (the CHECK scribe of `LEAD walk + CHECK`).
  - PLAN: M0-TM9, M0-TZ, and M0-TJ1* (`PLAN + LEAD answers`, DEFERRED).

### The E blocks: Haiku 5.5, high (rule (3))
- M0-T91, T93a and T94 are byte-identical to the text M0-TM8 read.
- M0-T111's only change is M0-TJ2's "at TOP" → "at Max [M0-TJ2]" in its contract case. It is the same test file and the same
  scope, with no new module, string or schema, and it still carries `Sizing: E`.
- BUILD·E on Haiku 5.5, high has 10 blocks in today's read, at $0.82 against the (usual) BUILD blocks' $6.52, and 8 in M0-V8's
  binding table. Rule (4) moves nothing.

### The new and changed blocks, re-derived (every rating unchanged)
Each lands on **Sonnet 5.5, high** by rule (3), with no reason from the list, unless marked otherwise.
- **M0-T66** `[M0-T65]`, BLOCKED on M0-R4.
  - The flag names the interface M0-T65 built: `Calibration::to_json`, `physics_hash`, `Calibration::parse`,
    `Scene::writes_calibration` and `resolve`. Reusing a built mechanism earns no (c).
  - BLOCKED is a legitimate end, not a red claim run (Verify NOT RUN), so rule (1) does not apply.
  - The open design its measurements exposed, what the cold ceilings mean, belongs to M0-R4, rated (usual) by (c) above. What
    M0-T66 then builds is the ruled spec.
- **M0-T67** `[M0-T66]`.
  - The flag says M0-R4's ruling re-scopes the block and it never runs as it stands. Today's text grades contract §5.1's
    fixed G-CORE bands, and its own Fail line makes a red a D or a question.
  - The design is M0-R4's.
- **M0-T70** `[M0-D50]`.
  - M0-TM4's kind: calibrate items (an order check refusing on a stubbed table, the items fixed).
  - The flag gives measured inputs to the whole calibration, which is the lead's run in M0-V9's window: T_c 36, helium's
    threshold 39.2, and X_H,c 0.35 near ~1,200 t.u. (UNVERIFIED).
- **M0-T75** (M0-TJ2: speeds now include Max as `time.rung_max`, and TOP is never shown).
  - One more string through the table's templates, which is a fixed spec's wording.
- **M0-T85** (M0-TJ2: Max's button last, `time.speed_max`, key 0).
  - A view model of a fixed spec. Max is held by M0-T123's `rungs()`, which is built.
- **M0-TV2** (M0-TJ2: Max's states captured).
  - A UI/UX capture pass.
- **M0-T97** `[M0-D55]`.
  - M0-TM4's kind: a measurement report.
  - The flag is a measured caveat the report carries: P5's first order under-resolves a burnt star's cooling per step at the
    shipped a_r, by 2.657 ×. Its Verify is the report's sections.
- **M0-T98a** (M0-TJ2's shortest-life objective, and `[M0-D50]`'s starting point and margins).
  - M0-TM4's kind: tuning within the contract's bounds, on overrides, BLOCKED with a question when no in-bounds value reaches
    the gap.
  - The objective is named: the fewest steps among the candidates that hold every bound with `sun_burn` GO. A parameter sweep
    inside fixed bounds is not an algorithm with no reference.
  - The constants' values change; no schema or FROZEN passage does.
- **M0-T103** `[M0-V8]` (2026-10-16).
  - The flags fix a test parameter: name the frame size, with the hold at 3 summaries.
  - They also name a measured hazard in M0-T58's and M0-T60's tracker identity. A red from it is a D by the block's own Fail
    line, and that D's filer rates it.
- **M0-T106** `[M0-V8]` (2026-10-16).
  - The flag says ledger.rs's W leaves out the sinks' own M·φ. That is M0-T57's mechanism, declared there, and its fix lies
    outside this block's `Deliver:` (a test file and a plant).
  - A red from it is the D its Fail line names. (b) belongs to that D's filer. (e) would rest on M0-T57's reused sums and
    §5.1's fixed bands, so it is not this block's own.
  - See § Considered.
- **M0-T109** (M0-TJ2: `--rung max`, Max at 30 frames with N_max steps a frame, `off_cap`).
  - bench.rs is a new module, but contract §3.3 and §0.5 [M0-TJ2] fix its interface (the CLI, bench.json's keys,
    `off_cap`). It is a fixed spec, so no (c).
- **M0-T110** (M0-TJ2: Max for TOP; now AFTER M0-T124).
  - Bit-identical dumps through the bench's loop.
- **M0-T112** (M0-TJ2: G-FPS's frame figures reported, `off_cap` graded).
  - Lighter than the text M0-TM8 read: only the clamp is graded.
- **M0-T113** (M0-TJ2: the top speed as fast as the card goes; `off_cap` graded, life_wall_s reported, plant `max-half`).
- **M0-T115** (M0-TJ2: the docs for Max and `off_cap`, and `top_speed`'s row).
  - A docs block.
- **M0-T111**: above, under the E blocks. **M0-R4** and **M0-T124**: above, on (usual).
- **M0-V9** `[M0-V8]`, **M0-V16** and **M0-V17** (CHECK, M0-TJ2's rewrites), and **M0-TZ** `[M0-TJ2]` (PLAN): rule (2) pins
  the gate.
- **M0-TM9**: PLAN, rule (2).

### Considered — observations, never defects (§0, no step up on a guess)
- **M0-T66 and M0-T67 after M0-R4.** M0-R4's close rewrites their `Deliver:` and `Verify:` after the lead's answers ("nothing
  wider", so it writes no rating). So do M0-T68–T70, if the chosen option reaches them ("any block the option adds or
  retires"; M0-R4 reads M0-T68).
  - Their ratings here come from today's text. A ruled option could land a reason from (3)'s list on one of them, for
    example option (A)'s CPU f64 scan on the real EoS table, where numeric semantics would be the block's own acceptance. That
    block would then want re-rating when it is rewritten.
  - Today none is named, so none is stepped up.
  - A rewrite adds no flag, so the refresh count will not catch it. The routes are §0's other triggers: the lead's word, or
    the next pass.
- **M0-T106's claim run is measured red-prone on the shipped scheme** (M0-TM8 § Considered). There is a fourth measurement
  now: W leaves out the sinks' M·φ, so G-CONS on a life with a sink needs the term (M0-V8).
  - Momentum: 3.9 × 10⁻³ and 3 × 10⁻³ of Σm|u| from source-term gravity, and 2.9 × 10⁻⁴ from a vacuum reset.
  - Energy: 0.4–0.9 of its scale on pressureless collapses, cause not measured.

  No rung makes the scheme conserve. The route is the lead's ruling, or the D its Fail line names. As at M0-TM8, this is for
  the lead, before lot 15 reaches M0-T106.
- **The lead picks** (§0): deleting ` · switch` from a heading runs that block on the picker's model and level, and the
  rating is kept.

## What changed in the plan
- **Ratings: 0 changed.** By rung, now (changed):
  - Opus 5.5, max: 13 (0). M0-V8, M0-TJ2 and M0-TM8 are closed, and M0-TM9 is new.
  - Opus 5.5, high: 2 (0). M0-R4 and M0-T124 are new, filed there.
  - Sonnet 5.5, high: 52 (0). M0-T63, T64 and T65 are DONE, and M0-T114 is N/A.
  - Haiku 5.5, high: 4 (0).
  - Fable 5.1, high, Opus 5.5, xhigh and Sonnet 5.5, medium: 0 (0).
- **The `Models:` line**: re-dated (above).
- **The register**: `Model ratings: 2026-10-19 by M0-TM9`, written through `plan.py register --set` before the claim run, so
  the claim covers it. The lint's refresh went from 7/5 to 0/5.
- **How the `Models:` line was edited.** plan.py has no call that rewrites the `Models:` line or a rating. So, as M0-TM2–TM8
  did, I used the file-edit tool, one edit. `old_string` was the line's tail, grep-counted to 1 first.
- **The diff**, checked by checkdiff.py before the close:
  - against the snapshot taken right after `plan.py status`: 0 rating segments, 1 `Models:` line, 1 register line, 0 other;
  - against the start's snapshot: the same, plus 2 own lines (this block's Status and its Flow row).

## Measured and not acted on
This block's diff is bounded to rating segments, the `Models:` line and the register's line (its `Pass:` and `Fail:`), so
nothing below is filed or flagged from here (M0-TM5–TM8's precedent).
- **M0-T66 and M0-T67 (perhaps T68–T70) after M0-R4's rewrite**: see § Considered.
- **M0-T106's G-CONS**: see § Considered. It is for the lead.
- **The refresh count's blind spot.** M0-V8's flags on M0-T103 and M0-T106 are stamped 2026-10-16, the day of M0-TM8's line,
  but were appended after M0-TM8 closed. §0's rule leaves them outside the count ("one stamped the pass's own day was there to
  read"). They were read here. That is the rule as written, not a defect in the plan.
- **The heading's "seven" against the `Deliver:`'s list.** The `Deliver:` names M0-T65's "two" on M0-T66 (one flag whose stamp
  is written twice) and M0-T66's three. The lint's seven also count M0-D50's flags on M0-T70 and M0-T98a and M0-D55's on
  M0-T97 (2026-10-18). All seven were read.
- **The lint's 14 WARNs.**
  - Thirteen DONE blocks outside the current phase want stubs: M0-TC7, V18, D53, D50, D55, D56, TJ2, T123, T125, D51, D57, D52
    and D54. Stubbing them is a compaction's job, or the next V's close.
  - M0-R4's `Deliver:` item 8 names no Verify step. That is M0-R4's own text.
- **M0-T115's heading reads (AFTER M0-T114)**, and M0-T114 is N/A (M0-TJ2). It sits between M0-T113 and M0-T115, so the
  order still holds.
- **Repo facts' agent line** names Claude Code 2.1.292 at the 2026-10-08 probe; today's probe reads 2.1.301. It is a dated
  probe, outside this diff, as M0-TM8 noted.
- **`CLAUDE_EFFORT=xhigh`** is in the environment while the session ran at max. It is the process's launch level, which the
  switch overrides (§B.3).

## The routine, on linux-pc (laserax-ai)
- **`python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md`**:
  - after the `Models:` line: [ALREADY RUN — PASS (GO, 581 checks, 0 failed, 14 warnings — the stubs and M0-R4's item 8
    above; header 595/600, ladder 7 rungs, refresh 7/5) on linux-pc];
  - after the register: [ALREADY RUN — PASS (GO, 581 checks, 0 failed, 14 warnings, refresh 0/5) on linux-pc].
- **checkdiff.py** against the post-status snapshot: [ALREADY RUN — PASS (0 rating segments, 1 `Models:` line, 1 register
  line, 0 other) on linux-pc]. On the mutant: [ALREADY RUN — PASS (rating=1, other=2, NO-GO, exit 1 — the red it must show)
  on linux-pc]. On the control: [ALREADY RUN — PASS (models=1, GO) on linux-pc].
- **No red-arm.** This task added no scope and changed no scope's check (R9; PLAYBOOK §4 Rule 4, v12.3).
- **The claim run**, after the structural edits: `python3 tools/pb/verify.py --changed --task M0-TM9`. `plan.py show` printed
  a `baseline:` line, so no `--base` is named (R10). [ALREADY RUN — PASS (GO: plan_lint 581 passed, 0 failed; 1/1 scopes GO,
  53 not impacted, 0 never checked, 0 stale, 0 unclassified, 0 owed; 0.07 s; base M0-TM9's start 2026-10-19T09:03:52-04:00;
  tree e788da44e8ee+c526acde92d3; python 3.12.3; load 8.06) on linux-pc].
- **The close**: `python3 tools/pb/plan.py close M0-TM9 --status "DONE" --text "<handoff>"`, written from the claim run.
  Then checkdiff.py and the lint run again, and `plan_lint` is re-verified scoped, since the close's edit lands after the claim
  (§4 Rule 4 step 7).
- The results are below, appended after the runs.

### Results, appended after the runs
- **The close** (09:15): `close: checks=3 failed=0 warnings=0 · handoff_lines=1 · flow=per-task`, GO. Status `DONE
  (2026-10-19 09:15, started 09:10)`. The register's `Next task:` already reads M0-R4, which runs next; left as found.
- **checkdiff.py** after the close, against the post-status snapshot and the start's: [ALREADY RUN — PASS (0 rating segments,
  1 `Models:` line, 1 register line `Model ratings:`, 3 own lines — Status, Handoff, Flow row — 0 other) on linux-pc].
- **The lint** after the close: [ALREADY RUN — PASS (GO, 580 checks, 0 failed, 14 warnings — the stubs and M0-R4's item 8
  above; header 595/600, ladder 7 rungs, refresh 0/5) on linux-pc]. The count drops by one at a close, as M0-TM2–TM8 saw.
- **`plan_lint`, re-verified scoped** after the close (§4 Rule 4 step 7), `python3 tools/pb/verify.py plan_lint --task
  M0-TM9`: [ALREADY RUN — PASS (GO, 580 passed, 0 failed, 1/1 scopes, 0.07 s, load 14.12) on linux-pc].
- **Files this task wrote**:
  - the plan: the `Models:` line, the register's `Model ratings:` line, and its own Status, Handoff and Flow row;
  - this file and logs/M0-TM9.log.
  - verify.py wrote logs/M0-TM9.plan_lint.log, logs/loop_times.jsonl and tools/pb/verify.anchor.json.
  - Only this section was written after the claim run. It is a skip-class path (tasks), graded by no scope.
