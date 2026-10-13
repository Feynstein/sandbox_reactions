# M0-TC4 — compaction at Phase 7's close

Ran on win-laptop (Laser2025-20), model=claude-haiku-5-5 level=high (rung_record now; matches the heading's rung).

## What moved
15 DONE blocks, stubbed by `plan.py stub … --by M0-TC4` into `milestones/m0/plan_archive.md`:
M0-TC3, M0-D27, M0-T33a, M0-T33b, M0-T34, M0-TM5, M0-D28, M0-D29, M0-T35, M0-T36, M0-T37, M0-T38, M0-T39, M0-T40, M0-V5.

- `plan.py stub`: 295 lines moved, 295 verified byte-for-byte.
- Independent check (python, scratchpad): each of the 15 original blocks appears verbatim in the archive; the other 189 blocks are unchanged; no TODO block touched.
- Plan file 3277 → 3042 lines; archive 2025 → 2335 lines (the +15 is one separator line per archived block).

## Lint
- Before: `plan.py lint` GO, 524 checks, warnings 2 (M0-TC3 and M0-D27 DONE outside the phase).
- After: GO, 523 checks, warnings 0. Header 486/600 before and after (the stubs sit in the block area, not the header).

## Not done, measured
- Rules: each row is already an index line; the full text of R1–R17 is already in `m0_rules.md`, so there was no long rule text in the plan to move. The paragraph above the table (gates, grants, budget) holds facts, not rule text, and stays.
- Pipeline state: already a register of pointers; only `Next task` changed (→ M0-D30).
- Hoist: no whole line repeats in three or more live blocks. The repeated fragments are the Verify fail-arm wording (`Fail: the plant stays GO`) inside TODO blocks, which this task must not touch.
- Repo facts vs Hazards: no sentence of 50+ characters appears in both; nothing to de-duplicate without rewording.

## Found, not acted on
- The Rules intro says `m0_rules.md` was "Considered and REJECTED (false economy)" (M0-TB), yet the file exists and M0-TC4's Deliver asks for moves into it. Needs the lead to reconcile.

## Claim
`python3 tools/pb/verify.py --changed --task M0-TC4` — GO, 524 passed, 1/1 scopes (plan_lint), 33 not impacted, on win-laptop. Log: `milestones/m0/logs/M0-TC4.log`.
