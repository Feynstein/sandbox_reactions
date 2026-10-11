# M0-TH-adopt-a — Adopt the toolkit: the tools v12.3 changed

Box: win-laptop (Laser2025-20), 2026-10-11. Rung: `rung_record.py now` → `model=claude-opus-5-5 level=high`
(heading: Opus 5.5, high — match). Status: DONE (resumed after M0-D20, same rung line on resume). Log:
logs/M0-TH-adopt-a.log; raw diffs: logs/M0-TH-adopt-a.toolkit_diff.log; claims: logs/M0-TH-adopt-a.claim.out (the
first, FAIL) and logs/M0-TH-adopt-a.claim2.out (the resume, GO). Diff record: tasks/toolkit_local_diff.md.

## Resume after M0-D20 (2026-10-11, win-laptop)
M0-D20 DONE fixed the selftest's walk fixture (its selftest GO 345/0, 157/157 red; `--redarm tools_selftest` GO,
the redarm this block owed). Tools unchanged since then. The claim was re-run, and it measures Verify 2's verify arm too:
[ALREADY RUN — PASS (517 passed, 0 failed; plan_lint 507/0/0 GO, tools_selftest 10/0/0 GO incl. verify 167 s) on win-laptop]
```
py -3.12 tools/pb/verify.py --changed --base ab36299 --task M0-TH-adopt-a
```
Five `FLAG oracle` lines, one for each swapped tool. The Deliver names them as « each changed tool », so there is
nothing to act on (same as the first claim).

## What exists now
- tools/pb/plan.py, verify.py, content_gate.py, rung_record.py, switch.mjs — v12.3's annex builds (§16 v12.3's five),
  each with the project's fixes the annex lacks re-added (toolkit_local_diff.md, fixes 1–4, 1b, 1c).
- tools/pb/scratch_copy.sh, status_page.py — kept at the project's (v12.2) build, byte for byte: §16 does not name
  them, and v12.3's annex builds drop their Windows fixes (`canon()`, `drain()`).
- launch.py, review_page.py, capture_web.mjs — identical to v12.3's annex; verify.json untouched (no tool new).
- M0-D20 filed after this block; M0-TZ flagged with every annex gap.

## Verify
1. §B.4's line, PLAYBOOK.md (v12.3) → scratch folder
   [ALREADY RUN — PASS (extracted: 10 - refused: 0, GO) on win-laptop]
2. Each swapped tool's selftest, in place:
   plan [ALREADY RUN — PASS (GO) on win-laptop] · content_gate [ALREADY RUN — PASS (GO) on win-laptop] ·
   rung_record [ALREADY RUN — PASS (GO) on win-laptop] · switch [ALREADY RUN — PASS (GO) on win-laptop] ·
   verify [ALREADY RUN — FAIL (checks=345 failed=2, plants 157/157 red, 162 s) on win-laptop] → M0-D20.
   Before the re-adds, in the scratch folder: plan NO-GO 1, content_gate NO-GO 2, rung_record NO-GO 4, switch GO,
   verify NO-GO 3 (the third, the `exclusive` plant, was the lost trace lock — red once 1c was back).
3. `plan.py lint` [ALREADY RUN — PASS (checks=505 failed=0, GO) on win-laptop]

## The claim
[ALREADY RUN — FAIL (517 passed, 1 failed; plan_lint 508/0/0 GO, tools_selftest 9/1/0 NO-GO on `verify` = M0-D20; anchor: 11 never checked, all tools_selftest's paths, no entry while it is red) on win-laptop]
```
py -3.12 tools/pb/verify.py --changed --base ab36299 --task M0-TH-adopt-a
```
Base ab36299 as carried flag (1) says: this block's `show` was v12.2's, so no start was recorded. The base read
includes M0-TE-V12.3's and M0-TM3's uncommitted plan edits (plan_lint GO). The claim printed three `FLAG oracle`
lines (rung_record.py, switch.mjs, verify.py "a check … Deliver: does not name"). The Deliver names them as « each
changed tool », a form the flag's reader does not match, so there is nothing to act on.
`--redarm tools_selftest`: NOT RUN (its clean arm reads NO-GO until M0-D20) — owed by M0-D20 (its Verify 2).

## Carried flag (2)
`sleep 2; py -3.12 tools/pb/rung_record.py now` after the swap → `model=claude-opus-5-5 level=high plugin=stale`.
The swapped switch is 12.3.1 and its `RUNGS` holds all seven rungs of the 2026-10-11 ladder (none missing). The
install line for this box (the lead's, once per box, PowerShell):

[NOT RUN — for the lead]
```
node "C:\workspace\sandbox_reactions\tools\pb\switch.mjs" build --out "C:\Users\YohanBelanger/.claude\pb-switch-src"; & "C:\Users\YohanBelanger\.vscode\extensions\anthropic.claude-code-2.1.295-win32-x64\resources\native-binary\claude.exe" plugin marketplace add "C:\Users\YohanBelanger/.claude\pb-switch-src"; & "C:\Users\YohanBelanger\.vscode\extensions\anthropic.claude-code-2.1.295-win32-x64\resources\native-binary\claude.exe" plugin update pb-switch@pb --scope user
```
linux-pc: its own `now` line, owed on linux-pc.
The lead ran it on win-laptop, 2026-10-11: `Plugin "pb-switch" updated from 12.2.2 to 12.3.1 for scope user`. This box
had 12.2.2, not the 12.2.1 the Repo facts and this handoff named. `now` then read
`model=claude-opus-5-5 level=high plugin=installed`; it takes effect after a client restart.

## Findings not acted on
- v12.3's annex was cut from a v12.2 base that lacks the win-laptop fixes (M0-D2..D7, D16 and the selftest ports).
  This is a playbook defect, not this project's — flagged to M0-TZ for the next bump.
- Repo facts' win-laptop git reads 2.55.0; the box has 2.56.0.windows.2. Not edited — Repo facts belong to a PLAN block.
- The v12.2 originals of the five swapped tools are at ab36299 (and were in this session's scratch folder).
