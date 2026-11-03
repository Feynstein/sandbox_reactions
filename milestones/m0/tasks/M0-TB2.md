# M0-TB2 — §2.1's TB table re-run over M0-TG2's cut of M0-D71 — task file

Session: linux-pc (laserax-ai), 2026-11-02; model=claude-opus-5-5 level=max (`rung_record.py now` on the launch prompt; = the
heading's rung). No code; no GPU run. Raw: logs/M0-TB2.log (the plan.py calls, the lint, the claim).

## Method
- Read: the cut's seven blocks, M0-TG2's handoff and task file (§ The cut, § TB's table), PLAYBOOK §0, §2.1 step 2, §3.2, the
  contract's [M0-TG2] passages (§0.5, §1.3.2, §1.4.2, §2.3.4, §2.7, §2.9, §2.12.3, §5.1 G-REST and G-CONS).
- Counting: M0-TB's convention (reports/size_pass.md §2). Files = what the block writes, with tools/pb/verify.json and the plants
  apart. One-line registrations are shown in brackets: a `mod` line, a testing.md row, a pinned list's entries.
- Paths (v): each path the seven blocks name was resolved to a full path and checked against `git ls-files` and the disk. A
  read-only parser drafted the list; it lives in the session scratchpad
  (/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/618fda07-4480-4523-95b8-69930d52b76e/scratchpad/tb2/). Then each
  symbol a Files or Deliver line names was grepped where the block says it lives, and each Deliver was read for the files it
  must touch.
- Checks: every scope a Verify line calls is in verify.json or created earlier in the cut. Every plant a re-arm's Pass names is
  listed for its scope. Each `≥ N cases` was compared with its scope's `expected` count today.

## The table (after the two fixes below)
| Block | Files written [registrations] (+ apart) | New·edit | ≈ new lines | (i) kind | (ii) classes | (iii)–(iv) | (v) | (vi) / E |
|---|---|---|---|---|---|---|---|---|
| M0-D71 | 6 [main.rs `mod`, testing.md row] (+2) | 1·5 | 470 | the twin's hold + its scope | 2: P8's rule · P4's walls and inflow | ✓ | ✓ | kept · not E |
| M0-D71b | 7 [booking.rs's pinned entries, testing.md row] (+2) | 0·7 | 260 | GPU P8's hold + its cases | 2: the f32 rule against the twin · the mask's start and Clear | ✓ | F1, fixed | kept · not E |
| M0-D71c | 6 [booking.rs's pinned entries, testing.md row] (+2) | 0·6 | 350 | GPU P4's held walls + G-REST's GPU cases | 2: the held wall · one-way inflow | ✓ | ✓ | kept · not E |
| M0-D71d | 3 [testing.md row] (+2) | 0·3 | 120 | the ledger's `held` term + its cases | 2: the invariant · old records read as 0 | ✓ | ✓ | kept · not E (a schema) |
| M0-D71e | 4 [testing.md row] (+2) | 0·4 | 150 | the dump's mask + its cases | 1–2: the resume · an absent key read as none held | ✓ | ✓ | kept · not E (a format) |
| M0-D71f | 5 [testing.md row] (+1) | 2·3 | 150 + the report | a measurement report, with the window's standing case | 2: the endings (items 1–2) · the window | ✓ | ✓ | kept · not E |
| M0-D71g | 6 (+2) | 0·6 | 70 | two physics.json keys + one case | 1: read from physics.json, twin and GPU | ✓ | F2, fixed | kept · not E (a schema) |

- Every block is under ≈ 6 files and 600 new lines once registrations are set apart. Line counts are estimates. They are
  M0-TG2's, plus about 10 lines for M0-D71b's Clear. M0-D71's ≈ 215 code lines match H's share of M0-R11's scratch (545 added
  lines over mod.rs, hydro.rs and r11.rs, which also carried candidates N and D and two drivers).
- (iii)–(iv): each cut is at an artifact, in this order: the twin's rule (the oracle) → the mask → the booking terms → the dump →
  all of them (the lives). M0-D71g needs D71's and D71b's constants and M0-T98a's stale window. No cut waits on a ruling. M0-D71f's
  own question to the lead closes that block, which (iv) keeps inside it.
- Splits: 0. So no TG defect (§2.1: two splits or more), and no flag to the next TG. Named exceptions: none, so no targeted V.
- Edit jobs: 0 of 7. Each changes a schema or a format, or writes more than 2 files. M0-D71d is 2 files without its doc row and
  ≈ 120 lines, but it changes a schema.

## Path findings (test (v)), fixed with the lead's yes
- **F1 · M0-D71b's Clear.** Its Files said step/mod.rs holds Clear, but step/mod.rs has no `Clear` (grep). Clear is `Edit::Clear`
  in crates/sr-engine/src/step/edit.rs (`Step::queue_edit`), applied per cell by shaders/edit.wgsl's `edit_cells` (record kind 6).
  The contract's «Clear world empties it» (§2.7 [M0-TG2]) lands there.
  - Fix: Files names step/edit.rs; Verify 1 adds `edits` (17 cases, the tools' tests, Clear included).
- **F2 · M0-D71g's keys.** physics.json is parsed against `SCHEMA: [(&str, Class); 52]` in crates/sr-physics/src/registry.rs.
  It refuses a key SCHEMA lacks (registry.rs:316) and a SCHEMA key the file lacks (:328). Its own tests pin the count
  (`shipped_files_load`, :797) and every key's value (`physics_initial_values_verbatim`, :843–859). So the two keys cannot
  land without registry.rs.
  - Fix: Files names registry.rs; Verify 1 adds `registry` (29 cases).
- Unique paths after the fixes: 44. 9 are absent, each labelled (new) at its first block and named only by later ones:
  held_core.rs, remnant_window.bin, reports/held_core.md and six plants. build/v9-sun-life/dumps/s_450000.bin is on disk only
  (git-ignored, labelled). The other 34 are tracked.
- Anchors resolved where named, one line per block:
  - D71: `Twin::new`, `Twin::step` and `booked` (mod.rs:93, 118, 224); `step_forced`, `wall` and `face_work` (hydro.rs:573,
    483, 785); `Floors::apply` (floors.rs:118); `R_CORE` (sinks.rs:54); `cold_disk_scene` (calibrate.rs:294).
  - D71b: `floor_cells` and `renormalise_species` (floors.wgsl:103, 145); `floors` and step 0's initial passes (step/mod.rs:897,
    577–611); `TERMS` and `BOOK_GROUPS`' `booking.floors` (state.rs:446, 510); booking.rs's pinned list (:208–250); Y_eΣ in
    react.wgsl (:504), `composition` shared from eos.wgsl (:54).
  - D71c: `hydro_update`, `hydro_edges`, `hydro_commit` and `face_work` (hydro.wgsl:728, 913, 956, 294).
  - D71d: `Momentum`, `invariant`, `json` and `Ledger::restore` with its length check (ledger.rs:79, 107, 279, 260, 263).
  - D71e: `Resume`, `Dump`, `write`, `read`, `dump_state` and `load_dump` (dump.rs:51, 61, 174, 179, 313, 327); `twin_dump`
    (headless.rs:605).
  - D71f: calibrate's `--only`, `--bracket` and `--out` (calibrate.rs:1227); the scene cut behind `SR_WRITE_SCENES`
    (cold_core.rs:353).
  - Read anchors: unresolved_remnant.md § Candidates measured, § SCF cold disk, § What H leaves open; black_hole.md M3, M8;
    tasks/M0-D71.md § Cause. The scratch-diff ranges 2–313 and 490–508 cover mod.rs's and hydro.rs's hunks and `update_held`.

## Checked and clean (no change)
- **Scopes and plants.** All 14 scopes the Verify lines call exist; `held_core` is created by M0-D71, before D71c, D71f and D71g
  use it. Every plant a re-arm's Pass names is listed for its scope: floors-no-cold-energy, booking-row-skip, ledger's 3, dump's 7.
- **Case counts bite.** Each Pass count is above today's `expected` by the block's new cases:
  - floors 7 → ≥ 9 (D71b);
  - ledger 5 → ≥ 7 (D71d);
  - dump 12 → ≥ 14 (D71e);
  - held_core ≥ 6 → ≥ 10 → ≥ 11 (D71, D71c, D71f).
- **Run order.** No scope comparing the twin and the GPU can hold a cell between M0-D71 (the twin's hold) and M0-D71c (the GPU's
  walls):
  - cpu_gpu's three 64 × 64 scenes and twin's cold_disk run with the cold pressure off. Then ε_cold = 0, so ε_th ≤ η · ε_cold
    cannot pass. Its Sun-like disk sits below σ_hold.
  - calibrate_cold solves the sheet, with no step and no GPU.
  - sink.rs forms sinks on built states (`sinks().form`), not through P4 and P8.
  - G-CORE (d)'s disk peaks at Σ 0.150 (M0-TG2's measurement).
- **Bindings.** M0-TG2's note holds.
  - `hydro_update` and its helpers use 7 group-0 storage buffers: hydro, species_a, species_b, next_lo, next_hi, rad_flux, grav.
  - eos.wgsl's group-1 `eos_table` makes 8, WebGPU's default per stage.
  - floors.wgsl uses 4 plus `eos_table`, so P8 has room for a mask buffer.
- **Booking.** state.rs holds `Booking.groups: [wgpu::Buffer; 8]` and `BOOK_GROUPS: [BookGroup; 8]`, both in M0-D71c's files.
  - Each group's planes are const-asserted ≤ 128 MiB at 2048² (16 MiB a plane). booking.floors goes from 2 to 4 planes (64 MiB).
  - P4's buffer is found by `position(writer == Pass::Hydro)` (step/hydro.rs:160). That finds the first P4 group, so P4's new
    `held` group needs its own lookup, inside step/hydro.rs (named).
  - ledger.rs maps terms by name (:298) and reads every non-accumulator group (:358), so M0-D71b's appended terms break nothing
    before M0-D71d. Only `Ledger::restore`'s length check (:263) refuses an old snapshot, as M0-D71b's Adversarial says.
  - No observers' snapshot is committed (git ls-files).
- **M0-D71e.** The twin never loads a dump: `--cpu-reference` refuses `--load-dump` (headless.rs:214–216). M0-D71's held_core cases
  read the twin's held set from sr-engine (the held count), so `twin_dump` can read it with no further file.
- **M0-D71f.** Each m_bh_sb life runs under 10 minutes on the RTX 4090 (reports/black_hole.md, M0-D90's Done-when), so its runs are
  the agent's own.
- **Dependents.** M0-T106 reads the ledger's momentum invariant, which M0-D71d extends; no edit. No [M0-TG2] passage is FROZEN
  (§1.2, §1.4.1, §1.6.4, §1.8.6 and §6.2.4 are).

## (vi) — one notch smaller, considered and not split
- **M0-D71.** P8's rule could leave with its hysteresis and item 1 cases. But the disks, the block's acceptance, need P8 and P4
  together (a producer/consumer pair, M0-TG2). At ≈ 470 lines it is not near the ceiling.
- **M0-D71b.** The mask's start and Clear could leave (≈ 40 lines); kept at ≈ 260.
- **M0-D71c.** Inflow could leave, but it re-opens the same kernel lines (M0-TG2). G-REST's GPU cases are its acceptance.
- **M0-D71f.** The crop case could leave with no ruling dependency, either into M0-D71c beside G-REST's GPU disks or into a block of
  its own. Kept: the block is not near the ceiling, and the crop is the standing form of its own window measurement (a), from the
  same dump and the same bound. M0-TB's reading: split when the block is near the ceiling or a check needs a later artifact.
- **M0-D71d, M0-D71e, M0-D71g.** One deliverable each.

## The rating — §0's rubric, first match
- Rule (1), escalation: none. No claim run failed on any of the seven. M0-D71's two sessions closed GO (loop_times, 2026-10-28
  and M0-D71-r2, 2026-11-02), with no failed fix attempt.
- Rule (3)'s list → rule (5): all seven (usual), Opus 5.5, high, as filed. Rule (4) was not consulted, since every block has a
  reason. No heading changed. The register's `Model ratings:` line is left as it is: the cut's blocks only.

| Block | Reason (rule (3)'s list) | From the block and the code |
|---|---|---|
| M0-D71 | numeric semantics | G-REST's measured bounds over 10,000 steps grade the held wall's flux and no work across it, not an analytic oracle. M0-R11 measured two plausible wall forms failing (Π alone; a mirrored stencil), and a rule that holds too much passes u = 0 (its Adversarial). M0-TG2's second reason, «a design the spec leaves open (items 1–2)», does not hold for the twin: §2.7 and §2.3.4 [M0-TG2] write the starting forms, and §0 says «never a fixed spec's wording». The rung stands on the first reason. |
| M0-D71b | numeric semantics; a persisted format | the f32 hold test against the twin's f64 at the η and 2η edges; `TERMS` 37 → 39 lengthens the observers' snapshot's `booked` list, refused until M0-D71d |
| M0-D71c | numeric semantics; a design left open | the wall's HLL in f32; how P4 reads the mask with `hydro_update` at 8 storage buffers, and a second P4 booking group where the lookups assume one per pass |
| M0-D71d | a schema | §2.9's momentum, summary.json's records, the snapshot's read |
| M0-D71e | a persisted format | §2.12.3's dump |
| M0-D71f | a design left open | items 1–2, open in the contract until this block's measurement |
| M0-D71g | a schema | physics.json's `SCHEMA` and §2.7's table, under G-CAL's hash |

## The lead's answers — the question tool, one call of three, 2026-11-02
- M0-D71b's files: «Fix it (Recommended)»
- M0-D71g's files: «Fix it (Recommended)»
- The register's Phase 15 gate lines: «Update them (Recommended)»

## Edits applied
- **M0-D71b**: Files names crates/sr-engine/src/step/edit.rs for Clear, and Verify 1 adds `edits`. Done with the Edit tool,
  because plan.py has no call for a block's Files or Verify text.
- **M0-D71g**: Files names crates/sr-physics/src/registry.rs, and Verify 1 adds `registry`. Same tool.
- **The register**, by `plan.py register --set`. Each value was built from the old line in the scratchpad and compared byte for
  byte after the call.
  - Outstanding commit gates: every gate through Phase 15 is in (54c9a5f, pushed; main = origin/main). The R18 clause is kept
    verbatim.
  - Awaiting lead: the Phase 15 item dropped, the rest kept verbatim.
  - Next task: M0-D71 first, the rest kept verbatim.
- **A flag to M0-D71** (`plan.py flag`): the seams its later blocks read (below).

## Declared defaults (cheap to reverse)
- §2.1's gate re-check of the lead's open steps: not put again. M0-V13 put them today, and this re-run's Deliver is the cut's
  blocks only. The one stale item, Phase 15's gate, was put to the lead and closed.

## Not acted on (measured or seen, with why)
- **The lint's 7 warnings**, all older than this session:
  - 5 DONE blocks to stub: M0-TC13 and M0-D144–M0-D147. Stubbing is a V's or a TC's move.
  - M0-D91's Read names heat.wgsl whole (1,472 lines), and 3 of its Deliver items have no Verify step. M0-D91 is outside the cut.
- **The header**: 601/600, inside the margin (M0-TG2 noted it). My edits replaced lines one for one.
- **M0-D71f's ≥ 11** catches a missing crop case only if M0-D71c lands exactly 10 cases. The crop case's existence is graded by its
  Verify 3 arm («the crop's case red under one at least»).
- **Seams, flagged to M0-D71 rather than changed:**
  - `Hydro::step_forced` has two callers outside D71's files (tests/gpu/radforce.rs:193, :418). M0-R11's scratch kept the
    signature, with the mask as a field on `Hydro`.
  - M0-D71g's Deliver greps for `SIGMA_HOLD|ETA_HOLD`, names M0-D71 does not pin.

## The routine
[ALREADY RUN — PASS (GO, 810 checks, 0 failed, 7 warnings, none on the cut) on linux-pc]
```
python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md
```
[ALREADY RUN — PASS (GO, 810 passed, 1/1 scopes, base M0-TB2's start) on linux-pc]
```
python3 tools/pb/verify.py --changed --task M0-TB2
```
- win-laptop: nothing owed (plan only).

## Next
M0-D71, the twin and the first of the cut.
