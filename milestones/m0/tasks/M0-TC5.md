# M0-TC5 — compaction, Phase 8's close (MOVE)

Model: model=claude-haiku-5-5 level=high (rung_record now, plugin=installed; matches the heading's rung).
Box: linux-pc (laserax-ai). Log: milestones/m0/logs/M0-TC5.log. Pre-stub plan copy (for the byte diff): scratchpad `M0-TC5.plan_before.md`.

## Stub (plan.py stub, 30 blocks, --by M0-TC5)
- Blocks: M0-T41, M0-T42, M0-T43, M0-TM6, M0-D33, M0-T44a, M0-T44b, M0-T45, M0-T46, M0-T47, M0-D34, M0-D35,
  M0-TJ4, M0-T117, M0-T118, M0-T119, M0-D36, M0-T120, M0-T121, M0-T122, M0-D37, M0-D38, M0-D39, M0-T48, M0-T49,
  M0-V6 (Phase 8, 26) + M0-TC4, M0-D30, M0-D31, M0-D32 (Phase 7's four). All 30 were DONE before the call.
- plan.py: 700 lines moved, 700 verified byte-for-byte into plan_archive.md. `git diff --numstat` on the archive:
  730 insertions, 0 deletions (the 30 extra lines are plan.py's per-block separators).
- Plan: 3670 → 3090 lines (the stubs stay in place). `plan.py lint` warnings 4 → 0 (GO, 554 checks).
- Kept in the plan by design: M0-D32's commit-gate trailer (`> **Commit gate (lead):** Phase 7 closes after M0-V5`),
  the same as M0-D8's in Phase 3. plan.py's stub WARN names it; lint does not count it.

## Rules (the Deliver's "long Rules entries moved")
- Rows R9, R10, R11, R12, R13, R15, R16, R17 — one line each, 201–458 chars — became pointers:
  `<title> · full text: m0_rules.md § Rn`. Each title is the m0_rules.md heading's own words.
- Full text already existed in m0_rules.md for every one of them (`## R9`, `## R10–R14` with `### R10`…`### R13`,
  `## R15`, `## R16`, `## R17`); no text was added to m0_rules.md. R10–R14's text there was reworded by M0-TB's
  hoist (not byte for byte), so those plan rows held wording that is not in the file — their facts are in it,
  checked by reading, not by a script. Each rule's facts are present in m0_rules.md (read per row).
- Left as they were: R1–R8 (short rows, full text in m0_rules.md) and R14 (174 chars, full text in m0_rules.md).
- Header is counted in lines, not characters: 541/600 before and after (each row is still one line).

## Pipeline state
- Next task: M0-TC5 → M0-D40 (the block after this one).
- Awaiting lead: M0-V6's three answers → one pointer line, `plan_archive.md § M0-V6` (the facts sit in the archive).
- Open D register, commit gates, counters, ratings: unchanged.

## Measured, not acted on
- Repo facts vs Hazards: 53 sentences of ≥60 chars in Repo facts; 0 verbatim in Hazards. Nothing to de-duplicate.
- Whole lines repeated 3+ times in the plan: one — `NO-GO · Fail: the plant stays GO` (5 occurrences, Verify fail
  arms of TODO blocks). Not hoisted: a hoist would edit TODO blocks, which the Pass forbids.
- Plan's `## Rules` intro (lines 278–279) still reads "Considered and REJECTED (false economy): m0_rules.md, binding
  (M0-TB)". It contradicts this Deliver and the file that exists. Not edited: the lead to reconcile (the same flag as
  M0-TC3's handoff).
- Lint header is unchanged by the stub: it counts lines, and the stubs stay as lines.

## Process note
- Two stray copy commands earlier in this session: one wrote a copy of the plan into `/tmp/x/` (a directory outside
  the scratchpad, which I did not create on purpose; it held only that copy). I deleted the copy and the empty
  directory; the plan's bytes were checked against the copy (md5 match) before deletion. A second cp into
  `milestones/m0/logs/..` failed as a same-file copy and changed nothing (plan line count checked: 3670).

## Claim
- `python3 tools/pb/verify.py --changed --task M0-TC5` [ALREADY RUN — PASS (GO, 554 passed, 1/1 scopes: plan_lint; 42 not impacted) on linux-pc].
- `plan.py lint` after the stub: GO, 554 checks, 0 warnings. Header 541/600, pipeline 9/60.

Next: M0-D40.
