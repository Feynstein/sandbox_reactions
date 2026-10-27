# M0-TC9 — Compaction, Phase 11's close (detail)

Model: claude-haiku-5-5 level=high (rung_record now, at launch and on the lead's prompt; the heading's rung).
Box: linux-pc (hostname laserax-ai). Start: `plan.py show` baseline M0-TC9, 586 files, 2026-10-27T15:28:08-04:00.

## Stub (Deliver: `plan.py stub` over Phase 11's closed blocks)
- 56 blocks, the Deliver's list verbatim, `plan.py stub … --by M0-TC9`: `stub: 1895 lines moved, 1895 verified byte-for-byte`
  → `milestones/m0/plan_archive.md`; `stub: checks=56 failed=0 warnings=0 · stubbed=56 · moved_lines=1895 · verified_lines=1895 · trailers_kept=0`.
- Counts, read-only, against the snapshots taken before the stub: plan 4977 → 3306 lines (−1895 moved, +224 = 56 stubs × 4 lines);
  archive 4207 → 6158 lines (+1895 moved, +56 separator lines). Archive removed lines: 0.
- TODO blocks: 70 before, 70 after; none touched.
- Not moved: M0-TJ5 (DONE, Phase 11, a PLAN + LEAD ruling). Its flag said «if your move takes it»; the Deliver's list does not
  name it, so it stays whole. Its text is cited by live blocks (M0-TZ's flags, §0.5 [M0-TJ5], §6.2.5).

## Header and Pipeline state
- Header (`plan.py lint`): 545/600 before and after; Pipeline state 9/60.
- Pipeline state: already a register (pointers only). Its `Next task` line updated at close, the only line that restated a
  task's own handoff.
- Repo facts de-duplicated: one line. The Boxes bullet's win-laptop call («as `py -3.12 tools/pb/…`») now points at Hazards, which
  holds the command and its root cause (M0-D7). Nothing dropped.

## Hoisting and Rules
- Lines repeated in three or more blocks after the stub: `NO-GO — each red a D (E4)` (5 blocks) and `Fail: the plant stays GO` (3).
  Both sit in TODO blocks, and hoisting would touch them, so left in place (the TC7 precedent).
- Rules: the table rows are already one-line index rows (full text in m0_rules.md); no entry moved.

## Verify 1 — `plan_lint`
- `python3 tools/pb/verify.py plan_lint --task M0-TC9` → `plan_lint 693/0/0 0.11 s GO` · `verify: 693 passed, 0 failed … 1/1 scopes GO`.
  Log: milestones/m0/logs/M0-TC9.log. Verify 1 [ALREADY RUN — PASS (GO, 693/0/0, 1/1 scopes) on linux-pc].

## Claim — `verify.py --changed --task M0-TC9`
- [ALREADY RUN — FAIL (NO-GO 11/1, 0/1 scopes GO: `dump` 1 failed) on linux-pc, RTX 5090].
- The failure: `crates/sr-engine/tests/gpu/dump.rs:345:5`, «F's 2 planes after the 14 channels» — M0-V9's F1 (the dump write now
  makes 17 planes; Caused by M0-D85). Log: milestones/m0/logs/M0-TC9.dump.log.
- Why `dump` is in the claim: the anchor has no record of it since M0-D85, so `--changed` selects it whatever the change is. No code path
  changed in this task (`git status`: plan, archive, logs, tasks only; the anchor JSON was already modified).
- Run on the wrong box: `dump` is a GPU scope, and R18 wants the server's RTX 4090 for it. The M0-TJ5 flag predicted this. Not re-run:
  M0-V9 measured the same F1 on laserax-ai-server, RTX 4090, and nothing since has changed the code.
- Anchor: 128 never checked, 0 stale, 0 unclassified (the M0-TJ5 record, unchanged).

## Measured, not acted on
- The claim's red is F1, not this task's change. Whether a red on F1 alone ends a try below `(usual)` is the lead's call (the
  M0-TC9 block's own flag). Asked; see the handoff.
