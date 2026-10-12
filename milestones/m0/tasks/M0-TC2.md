# M0-TC2 — compaction (MOVE), detail

Box: win-laptop (Laser2025-20). Rung: model=claude-haiku-5-5 level=high (rung_record.py now; the heading's Haiku 5.5, high).

## What moved
- 17 DONE blocks, Phase 5, stubbed by `plan.py stub … --by M0-TC2` into `milestones/m0/plan_archive.md` (appended after M0-TM2):
  M0-T18, M0-T19, M0-T20, M0-D18, M0-T21, M0-D19, M0-T22, M0-TE-V12.3, M0-TM3, M0-TH-adopt-a, M0-D20, M0-T23, M0-T24, M0-T25, M0-T26, M0-T27, M0-V3.
- Each stub: heading kept, `- Status: DONE (…) · archived by M0-TC2, 2026-10-12`, one-line handoff, `Full block: plan_archive.md § <id>`.

## Checks
- Byte check (script in the scratchpad, `tc2/`): each of the 17 original block spans appears verbatim in plan_archive.md: True.
- Non-target blocks: text unchanged (0 blocks differ between the snapshot and the plan).
- Plan diff vs the snapshot: 17 hunks, lines 764–1157 of the old plan, all inside the moved blocks' span.
- plan_archive.md: 1218 → 1614 lines; the plan: 3516 → 3205 lines. The difference is the blank separator line per archived block.
- `plan.py lint --plan …`: GO before (checks 521, header 454/600, verify_fences 2) and after (checks 517, header 454/600, verify_fences 0). `show` reported header=257, a different measure; both are unchanged by the stub.
- The two verify fences were in the stubbed blocks, so they left the plan with them.

## Declared, not done (TC's shape, PLAYBOOK §8 TC<n> (i)–(iv))
- (ii) Pipeline state: already a register of pointers. Only `Next task` changed (M0-TC2 → M0-TM4).
- (iii) Hoisting: exactly 3 lines of ≥25 characters repeat in three or more blocks. All three are wrapped continuation fragments of Verify fail arms ("NO-GO — each red a D (E4)" in 10 blocks; "Fail: the plant stays GO" in 7; "plant NO-GO — Fail: the plant stays GO" in 3). None is a standalone line, so none was hoisted. The Rules table rows are already index lines, and their full text sits in m0_rules.md, so no Rules entry was moved.
- (iv) Repo facts: de-duplication not done. Their duplicates sit next to the dated Hazards, which are carried until retired, so removing either copy needs a judgement this task did not settle.

## Mistake
- The first `plan.py stub` call sent its stdout to `/tmp/x` and discarded it, against the scratchpad rule, and it was not logged. The result was checked afterwards by diff, not by the tool's own output. The stub was not run twice.
