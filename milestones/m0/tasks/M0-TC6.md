# M0-TC6 — compaction at Phase 9's close

Ran on: linux-pc (laserax-ai), model=claude-haiku-5-5 level=high (rung_record now, plugin=installed; matches the heading's rung `Haiku 5.5, high`).
Start: `plan.py show M0-TC6` → GO (checks 2, failed 0); baseline 2026-10-15T13:59:21-04:00.

## Before
- `plan.py lint`: checks 558, warnings 5 (M0-TC5, M0-D40, M0-D41, M0-D42, M0-D43 — DONE outside the phase, uncut), header 551/600, pipeline 9/60.
- Plan 3301 lines; plan_archive.md 3065 lines. Copies in the scratchpad: `tc6/plan_before.md`, `tc6/archive_before.md`.

## Stub (plan.py stub … --by M0-TC6)
- 14 blocks: Phase 9's nine — M0-T50, M0-T51, M0-D44, M0-T52a, M0-T52b, M0-T53, M0-T54, M0-T55, M0-V7; Phase 8's five — M0-TC5, M0-D40, M0-D41, M0-D42, M0-D43. All were DONE (checked before the call).
- Tool result: 288 lines moved, 288 verified byte-for-byte → plan_archive.md. One trailer kept in the plan under its stub (M0-D43's `Commit gate (lead)` line — the tool's WARN, by design). No TODO block touched.
- Archive grew 3065 → 3367 lines (+302: the 288 moved plus the tool's archive framing; not measured line by line).

## Rules index (Deliver: long Rules entries → m0_rules.md, index kept)
- Rows R1–R8 and R14 of the plan's Rules table held text that m0_rules.md did not hold verbatim (its sections were worded differently). Each row's cell text was appended, unchanged, as one line under its section heading in m0_rules.md (`- Index row, moved verbatim from the plan's Rules table (M0-TC6): …`). The plan's row became `| Rn | <title> · full text: m0_rules.md § Rn |`, the R9–R17 shape.
- Check: all 9 cell texts, taken from plan_before.md, found exactly once in m0_rules.md (`grep -cF`): 9/9.
- m0_rules.md 266 → 275 lines. Plan header count unchanged (551/600): rows are one line each, so a shorter row does not lower the header count (the same reading TC5 measured).
- R10–R13, R15–R17 already pointers (TC5); R16 already a pointer row with its gist.

## Measured, not acted on
- Pipeline state is already a register of pointers (9 lines): no change.
- No whole line of 60+ characters repeats three or more times across the plan (`uniq -c`): nothing to hoist.
- Repo facts: no sentence repeats (split on `.` and `;`, lines of 40+ characters): nothing to de-duplicate.
- Not reconciled (the lead's): the plan's Rules intro says "Considered and REJECTED (false economy): m0_rules.md, binding (M0-TB)", while m0_rules.md exists and is the rules' full text. Flagged in TC4 and TC5 too.

## After
- `plan.py lint`: checks 558 before the close, 557 after it (the close's own status edit; not chased), failed 0, warnings 0 (was 5), header 551/600, pipeline 9/60. GO. The inline handoff's "GO 558" is the pre-close count.
- Claim: `python3 tools/pb/verify.py --changed --task M0-TC6` → GO, 558 passed, 0 failed, 1/1 scopes (plan_lint), 44 not impacted, box laserax-ai, python 3.12.3. Raw output: milestones/m0/logs/M0-TC6.log.

## Next
- Pipeline Next task: M0-D45 (Flow order: TC6 → D45 → D46 → D47 → D48, all TODO).
