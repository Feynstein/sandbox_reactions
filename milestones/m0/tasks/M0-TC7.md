# M0-TC7 — compaction, Phase 10's close (MOVE)

Model/level: claude-haiku-5-5 level=high (rung_record now; matches the heading's rung).

## Moved (plan.py stub --by M0-TC7)
- Phase 10 (12): M0-T56, M0-T57, M0-T58, M0-T59, M0-T60, M0-D49, M0-T61, M0-T62, M0-TM8, M0-T63, M0-T64, M0-V8.
- Phase 9's six that ran after its close: M0-TC6, M0-D45, M0-D46, M0-TM7, M0-D47, M0-D48.
- 393 lines moved, 393 verified byte-for-byte by plan.py. Independent check against the pre-task snapshot: the 17 blocks without a trailer are identical in plan_archive.md; M0-D48 matches except its Phase 9 commit-gate trailer (4 lines, stays in the plan under its stub; WARN by design, as at M0-D8).
- Non-stubbed blocks: 247 before, 247 after, none changed. No TODO block touched.

## Deliver items with nothing to move
- Lines repeated in 3+ blocks: none, except `- Handoff: <placeholder>` (78 TODO blocks). Hoisting it would touch TODO blocks, so it stays.
- Repo facts: no internal duplicate lines.
- Rules: already one-line index rows, full text in m0_rules.md.
- Pipeline state: already a register of pointers.

## Numbers
- Plan 3371 -> 3050 lines; plan_archive.md 3367 -> 3778 lines (no archive line removed).
- Lint: warnings 6 -> 0, GO 570 checks. Header 565/600 before and after.
- verify.py --changed --task M0-TC7: plan_lint 570 passed, 0 failed, 1/1 scopes GO, on linux-pc (laserax-ai).
