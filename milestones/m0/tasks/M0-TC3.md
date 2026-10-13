# M0-TC3 — detail

Compaction at Phase 6's close (MOVE). Box: win-laptop (Laser2025-20). Model: claude-haiku-5-5, level=high (rung_record now, plugin=installed).

## Stub (plan.py stub, --by M0-TC3)
- Blocks: M0-TC2, M0-TM4, M0-TR1, M0-TR1a, M0-TR1b, M0-D21, M0-D22, M0-D23, M0-D24, M0-T28, M0-T29, M0-T30, M0-T31, M0-T32, M0-V4.
- plan.py: 15 stubbed, 320 lines moved, 320 verified byte-for-byte into plan_archive.md.
- My own check (scratchpad, read-only): for each of the 15, the archived block equals the original block line for line, non-blank lines. M0-D24 differs only by its trailer, which stays in the plan.
- Kept in the plan: M0-D24's 4-line trailer, the Phase 5 commit-gate quote (`> **Commit gate (lead):** Phase 5 closes after M0-V3`). plan.py's choice; a lead line, not a block.

## Header
- lint before: header=465/600, warnings=9 (the nine DONE blocks outside the current phase).
- lint after stub: header=465/600, warnings=0. Stubs sit in the task body, so the header figure did not move.
- lint after edits: header=465/600, warnings=0, GO. Pipeline state: 9/60.

## De-dup (Repo facts → pointers; the fact kept verbatim in Hazards)
1. Boxes, win-laptop: "31 GiB free of 953 GiB" → "(disk: Hazards)". Kept in Hazards: "the disk is 97 % full (31 GiB free of 953 GiB)".
2. Toolchains, win-laptop: "user variable `MSYS=winsymlinks:nativestrict`" → "user variable for symlinks (Hazards)". Kept in Hazards: "MSYS=winsymlinks:nativestrict".
3. Toolchains, win-laptop: "(the Store package's alias; `python` 3.12.10)" → "(Hazards: the Store alias; `python` 3.12.10)". Kept in Hazards: "python3 is the Store package's alias (3.13.14)".

## Verify
- Own scope, alone (before the close): `python3 tools/pb/verify.py plan_lint --task M0-TC3` → 520 passed, GO.
- Claim (after the close and the handoff): `python3 tools/pb/verify.py --changed --task M0-TC3` → 519 passed, 1/1 scopes GO (plan_lint), 27 not impacted, base M0-TC3's start. The 520 → 519 difference is unexplained; flagged.
- Deviation: R15 asks for `py -3.12` on win-laptop. The claim ran under python3 (3.13.14). A `py -3.12 ... --changed` re-run answered NO-GO "nothing changed since every scope's last GO … zero cases", so it was not repeated.
- Log: milestones/m0/logs/M0-TC3.log (the task log) and milestones/m0/logs/M0-TC3.plan_lint.log (the scope's log).

## Declared, not done
- Hoisting: no whole line repeats in 3+ blocks. The repeats are wrapped Verify fail-arm fragments ("Fail: the plant stays GO", "NO-GO — each red a D (E4)"); hoisting them would reflow each block's Pass/Fail lines. Same finding as M0-TC2.
- Rules: each row is an index line; full text for R1–R17 is in m0_rules.md (unchanged). None moved.
- Pipeline state: already a register of pointers; only `Next task` changed (M0-TC3 → M0-D27).
- Repo facts duplicates beyond the three above sit beside carried Hazards with different detail; not de-duplicated.

## Measured, not acted on (for the lead)
- Repo facts, Boxes: "every toolkit call goes through [the Bash tool]" on win-laptop. Hazard M0-D23 (2026-10-12): "both shells work" (verify.py puts Git's usr\bin on the PATH). The two conflict; a fact correction is not a move, so it is flagged, not edited.
