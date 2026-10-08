# M0-TJ3 — Direction ruling: win-laptop in play for M0, what runs where

Box: win-laptop (Laser2025-20, Windows 11 Pro 26300), Git Bash; toolkit calls as `py -3.12 tools/pb/…` (M0-D7).
Rung: `rung_record.py now` → model=claude-opus-5-5 level=max plugin=installed — the heading's rung (Opus 5.5, max): no
MISMATCH. Log: logs/M0-TJ3.log. The record of the lead's words: reports/win_laptop.md § Ruling.

## Read
The plan's header and this block (`plan.py show`), reports/win_laptop.md, m0_rules.md (R4, R5 and the rules they touch),
contract §3.3–§3.6, §5.3–§5.5, §6, §2.11, PLAYBOOK §0, §2.6, §8, §12, §14.1, §14.3 (TJ); then every block that names a
linux-pc-only item (Xvfb, lavapipe/llvmpipe, the Quadro, the RTX 5090, start.sh, /usr/bin/google-chrome) — found by one
read-only awk pass over the plan (logs/M0-TJ3.log), each read whole where the ruling could change it: M0-T1–T9, TV1, V1,
T15, T31, T63, T66–T71, V9, T84–T88, TV2, V12, T94–T96, TV3, V13, T109–T113, TJ2, T114, T115, V16, V17, TD, TW;
tasks/M0-D7.md; reports/size_pass.md's rows for T2, T5, T6, T8, T95; reports/bootstrap.md §3's Windows row.

## The fork and the answers
One question-tool call, four questions (≤ 4, the recommendation first, each consequence one plain sentence) — verbatim
in reports/win_laptop.md § Ruling. Picks: Q1 «Build and close lots anywhere» (not the recommendation) · Q2 «Laptop
versions (Recommended)» · Q3 «Off-screen window (Recommended)» · Q4 «Skip it» (not the recommendation).

## Every surface changed (each cited to its answer)
- m0_rules.md: R4 + the off-screen window on win-laptop (Q3) · R5 rewritten, both boxes in play, any block runs on the
  lead's box (Q1) · R9's budget per box, win-laptop's 97.7 s (Q1, PLAYBOOK §8) · R12 + the RTX 4080 Laptop (Q2) · R15
  new, the box pattern (Q1–Q3).
- Plan header: goal paragraph (the Windows laptop no longer among the items routed past M0 — Q1) · Rules line (the
  budget per box; E blocks 8 → 7, M0-T6 unmarked) · Rules rows R4, R5, R12, R15 · Superseded (R5's text, §6.6's text, the
  goal's routing, M0-D7's 20-run loop) · Repo facts (Targets, win-laptop's Boxes line, the launchers line).
- Contract, in place, tagged [M0-TJ3]: §3.3 `--offscreen-window` (Q3) · §3.6 `game-offscreen`, start.bat (Q3, Q2) ·
  §5.4 G-DESK's and G-WEB's win-laptop routes (Q3, Q2) · §6.2.3 the Windows Chrome (Q2) · §6.4 the RTX 4080 Laptop (Q2) ·
  §6.6 rewritten (Q1–Q3). §0.5 stays "none": no FROZEN passage changed (§6.2.4 and §1.8.6 untouched).
- Blocks: M0-T2 each box's adapter list (Q2) · M0-T5 the off-screen route built with the window, case `xvfb` → `window`
  (Q3) · M0-T6 start.bat and the routed `launcher` case (Q2, Q3) · M0-T8 the capture check's laptop route (Q3) · M0-T9
  both boxes in running.md and testing.md's Boxes rule (Q1–Q3) · M0-T84 the renamed case in its Pass line (Q3) · M0-V1 its
  box, its launcher, the owed list (Q1, Q2) · M0-V17 on linux-pc, every check owed there run, laptop-owed named (Q1).
- Flag: M0-V1's `Carried flags:` ← the linux-pc toolkit checks M0-D2..D7 left owed (`plan.py flag`, logs/M0-TJ3.log).

## Ratings (PLAYBOOK §0 — the filer rates what it changes)
- M0-T6: `Sizing: E` removed — start.sh, start.bat, .gitattributes, tests/smoke.sh's route and verify.json take it past
  §0's ≤ 2 files; rated by rule (3), a BUILD block with no reason from the list → the highest rung below (usual):
  Sonnet 5.5, high (was Sonnet 5.5, medium as an E).
- M0-T2, T5, T8, T9, T84 keep their ratings: no new reason from rule (3)'s list; T5 stays on (usual) by TB's reasons (f),
  (e) (reports/size_pass.md, its rating table), the off-screen flag being one more window mechanism of the same kind. T5's size: 5
  (+2) files with tests/smoke.sh's route (TB counted 4 (+2), 350 lines) — under the 6-file ceiling.
- M0-V1, M0-V17: CHECK — rule (2), (gate), unchanged.

## Carried flags handled
- [M0-D3] content_gate selftest 36/36 and `--redarm tools_selftest` on linux-pc → flagged into M0-V1 (with M0-D2, D4–D7's
  owed linux-pc runs, all inside `tools_selftest` and its red-arm).
- [M0-D7] the 20-of-20 loop on win-laptop → retired by the lead (Q4 «Skip it»), Superseded; tools_selftest GO on linux-pc
  after `case_env` → in the same M0-V1 flag; "one more 0xC0000005 under 3.12 → a D with the faulthandler stack" stays
  (Hazards).

## Findings measured, not acted on
- Pipeline state's "Outstanding commit gates: Phase 2 (…)" was stale: `git log origin/main` = 251b054, Phase 2's 3be0d03
  pushed before it — set to Phase 2b at the close.
- Stale texts left for TZ, as M0-D5 and M0-D6 named them: the Hazard "now 11/12 (M0-D5 holds the last)" (12/12 since
  M0-D5); the "seen once" lines; reports/win_laptop.md's "Status: Verify 2 is NO-GO" (M0-TE-win's own text).
- verify.json's `box:` is per scope (docs/agent/testing.md, item 5 «Boxes»); R15 therefore routes a per-box check inside one scope's
  script rather than binding a case — a per-case `box:` is not offered by the doc (not measured in the tool).
- launch.json names `build/target/release/sandbox-reactions` with no `.exe`; whether launch.py starts it so on Windows is
  M0-T4's to measure there (R15's row says a script never assumes the suffix away).
- The Rules' budget line quotes M0-TH's "62 cases" and M0-TE-win-r2's "500 passed" as measured; the units differ (cases
  against checks).
- §6.2.3's Windows flags (`--headless=new --enable-unsafe-webgpu`, D3D12) are marked opinion and UNVERIFIED — no source
  read today; M0-T15 measures and names what it used.

## Filed
- M0-D8 (`plan.py append … --after M0-TJ3`; D=8): `tools_selftest[verify]` crashed in the claim run — «crashed after 30
  checks: IndexError: list index out of range (return [(s.split()[0], float(s.split()[1]), float(s.split()[2])))» — a
  Python exception inside the selftest's parser, not M0-D7's native 0xC0000005; not reproduced in four runs after. Filed
  on the lead's Q4 consequence («any later crash is caught with its stack and filed as a bug»); rated (usual), the cause
  not measured; placed before M0-T1 and above the Phase 2b gate (the gate line now names it) — a Windows toolkit fix like
  M0-D2..D7, in the same commit. `plan.py append` warned that M0-D8 went above M0-TJ3's trailer: that side is meant.
- The Flow row of M0-T6 re-worded by hand («The double-click launchers start.sh and start.bat (M0-TJ3)»): plan.py has
  no call for a goal's text.

## Runs (all on win-laptop, Laser2025-20, Git Bash; raw output in logs/M0-TJ3.log)
[ALREADY RUN — PASS (model=claude-opus-5-5 level=max plugin=installed — the heading's rung) on win-laptop]
```
sleep 2; python3 tools/pb/rung_record.py now
```
[ALREADY RUN — PASS (GO, 493 checks, 0 warnings — the third run; the first GO carried 1 WARN from two semicolons in
Deliver edits, the second 1 WARN on M0-D8's whole-file `Read:`, each fixed) on win-laptop]
```
py -3.12 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md
```
[ALREADY RUN — FAIL (499 passed, 1 failed: tools_selftest[verify] IndexError after 30 checks; plan_lint 490 GO; the
anchor's 11 «never checked» follow from that red; FLAG lines = M0-D2..D7's uncommitted tool edits; 75 s) on win-laptop]
```
py -3.12 tools/pb/verify.py --changed --base 251b054 --task M0-TJ3
```
[ALREADY RUN — PASS (3 of 3 GO, 168 checks, 99/99 plants red each) on win-laptop]
```
for i in 1 2 3; do py -3.12 tools/pb/verify.py tools_selftest --case verify --task M0-TJ3-v0$i | tail -n 1; done
```
[ALREADY RUN — PASS (10 passed, 1/1 scopes GO, 76.7 s, jobs 8; anchor 0 never checked) on win-laptop]
```
py -3.12 tools/pb/verify.py tools_selftest --task M0-TJ3-s01
```
[ALREADY RUN — PASS (493 passed, GO — the late plan edits, scoped) on win-laptop]
```
py -3.12 tools/pb/verify.py plan_lint --task M0-TJ3-p01
```
[NOT RUN — owed on linux-pc (flagged into M0-V1)]
```
python3 tools/pb/verify.py tools_selftest --task M0-V1-linux
python3 tools/pb/verify.py --redarm tools_selftest --task M0-V1-linux
```
