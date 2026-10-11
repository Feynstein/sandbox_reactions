# M0-TE-V12.3 · PLAYBOOK v12.3 adoption — the full record

**Summary.** win-laptop (Laser2025-20), Git Bash, `py -3.12` for the toolkit (R15); model=claude-opus-5-5 level=max
(`rung_record.py now`, the heading's rung). One §16 entry applied, v12.3's. Structural pass done: the lead's ladder
written and re-read on the day (no change to ask about), R9/R10/R15's box-switch sentence and the budget's TR trigger now
PLAYBOOK §4 Rule 4 (old wording in Superseded), R16 retired N/A with its text kept, 11 closed blocks stubbed (180 lines,
byte-identical), M0-TM3 filed to do the ratings, two flags filed. Nothing to put to the lead: every v12.3 change is
ratified for every project. Verify 1 GO, Verify 2 GO. Claim run: § The claim run, the last step.

## Which §16 entries
The plan has no other `TE-V` block, in the plan or plan_archive.md. Its Rules read v12.3 because the roll-out renamed
them (ab36299, `PLAYBOOK.md v12.2` → `v12.3`); before the bump they named v12.2 (Repo facts: "green field (2026-10-08)
— PLAYBOOK.md v12.2"). So the entries newer than v12.2: **v12.3's alone** (§16, 2026-10-10). Read: §16's v12.3 entry
and the sections it names — §4 Rules 1 and 4, §2.3, §8, §10, §11, §14.1, §14.3, §15, §A.2's section text; plus §0, §3.1–
§3.3, §B.3, §2.1, §2.6 for this block's own Deliver; annex §A.0, §A.1, §A.2, §A.9, §A.10 section texts (not the source).
No `tasks/local_law_extract_v12.3.md` exists: the roll-out's AGENTS.md diff (ab36299) changes only the mirror comment's
version and §C's Rule 4 bullet — no local content to extract.

## STRUCTURAL — item by item
| Item | Result |
|---|---|
| Local law into Repo facts / Hazards / Rules + annex; "now PLAYBOOK §x" | R9 (index + m0_rules.md): now §4 Rule 4's two tiers — red-arm only a scope the task added or whose check it changed; `--changed` once, last, against the task's start; `--all` then the sweep `--redarm --moved` at phase-closing V and TR. R10 (index + annex): `verify.py --changed --task <ID>` against the task's start; `--base <a commit's SHA>` only for a block whose `show` recorded no start (one started before M0-TH-adopt-a's swap), never `HEAD`. R15 (annex), Switching boxes: the claim reads each task's start on its box. Budget line (Rules + R9's Budget bullet): the TR fires on the median of the box's last three counted loops, normalised to an idle box, raw beside it. Each tagged [M0-TE-V12.3]; the replaced wording quoted in Superseded. Nothing dropped. Hazards: none added or retired. |
| Changes-by-asking line | Present, §0's shape (`every block — …`); unchanged. |
| No live file claims the old version | Rules' version line already v12.3 (roll-out). Repo facts' "this playbook's switch (12.2.1)" → v12.3's 12.3.1 once M0-TH-adopt-a swaps `switch.mjs`, the project's 12.2.1 until then. History kept: the green-field fact, the 2026-10-08 win-laptop agent line, the tracker's bootstrap delta, closed blocks, archives, handoffs. `git grep` for v12.2 / 12.2.1 outside them: only `tools/pb/` (the toolkit, M0-TH-adopt-a's). |
| Every open block's Status in §3.3's shapes | `plan.py lint` GO, 507 checks, 0 warnings (Verify 2). |
| `Models:` line — the ladder | Written from this block's carried flag (the lead's ladder, the canonical repository, 2026-10-10; Haiku 5.5, high added 2026-10-11): Fable 5.1, high · Opus 5.5, max (gate) · Opus 5.5, xhigh · Opus 5.5, high (usual) · Sonnet 5.5, high · Sonnet 5.5, medium · Haiku 5.5, high. Re-read 2026-10-11 — the probe: `CLAUDECODE=1`, `CLAUDE_EFFORT=xhigh`, `AI_AGENT=claude-code_2-1-295_agent`; the model pages: the claude-api skill's table in Claude Code 2.1.295 (cached 2026-10-06): Fable 5.1 $10/$50, Opus 5.5 $4/$20, Sonnet 5.5 $2/$10, Haiku 5.5 $0.10/$0.50 (prompts ≤ 100K), each taking low…max. No model retired, renamed or repriced against M0-TM2's read (same table) → no question. Lint: `ladder=7 rungs`, one `(gate)`, one `(usual)`. |
| Every open block rated | 123 open blocks besides this one (97 Opus 5.5, high · 19 Opus 5.5, max · 7 Sonnet 5.5, medium), past §0's ~15 → **M0-TM3 filed to run next** (`plan.py append`, Counters TM=3); it rewrites the register's `Model ratings:` line. No role word (`gate rung` …) in any heading. M0-TH-adopt-a's Opus 5.5, high is the roll-out's usual-rung placeholder (§3.2) — named to M0-TM3. |
| Switch wired (§B.3) | `.claude/settings.json` holds the UserPromptSubmit hook line, its text §B.3's verbatim (it fired on this session's prompt); no other key to keep; no `.claude/skills/pb-*` (`.claude/` holds settings.json only) → nothing missing, nothing written. RUNGS against the new ladder and the box's install: **NOT RUN (before the swap)** — flagged to M0-TH-adopt-a. ` · switch` segments: a later adoption keeps each as found — every open heading carries one, unchanged; M0-TM3 carries one (its filer writes it). |
| `Kind kept:` | None needed: no open block whose id v12.3's letters type otherwise (lint: no type/id disagreement). |
| Pipeline state a register of pointers | Next task → M0-TM3; Outstanding commit gates → none, Phases 2b–4 and Phase 5's DONE blocks through M0-T22 are in per-block commits through ab36299, pushed (`git status -sb`: `main...origin/main`, nothing ahead); next Phase 5's, under M0-V3. Counters TM=3 (by append). |
| Closed blocks stubbed | `plan.py stub … --by M0-TE-V12.3` over the 11 DONE blocks of closed phases the lint named: Phase 3 — M0-TC1, D10, D11, D14, D15, D16, D12, D13; Phase 4 — M0-V2, D17, TM2. 180 lines moved, 180 verified by the tool; my own extraction before and after: **11 of 11 blocks byte-identical, 180 lines**. Two commit-gate trailers (Phase 3's, Phase 4's) stay in the plan under their stubs. Phase 5's DONE blocks stay (current phase); the DEFERRED M0-TJ1, M0-TJ2, M0-T114 stay (future phases, as M0-TC1 and M0-V2 left them). |
| `logs/ reports/ tasks/ images/` | Exist (lint). |
| `M0-TB-V12.3` | Not filed: v12.3's entry changes no §2.1 sizing rule. |
| Toolkit | Left to M0-TH-adopt-a — no tool written. |

Header: 422 → 442 of 600 (Flow +1, Rules +1, Superseded +12, Repo facts +6); Pipeline state 9 of 60.

## BEHAVIOUR-CHANGING — put to the lead
None. v12.3's changes — the task's own start, the claim record, the red-arm proof, the two tiers and the sweep, the TR
trigger at an idle box, the per-block commit line retired, git optional everywhere — are all on its "Ratified for every
project" line (the lead, 2026-10-10); the tool swap is v12.1's ratified item. No grant widened or narrowed, no task type
added to the pipeline. The ladder is the lead's answer (the carried flag); the ratings it moves are M0-TM3's, by the
ratified rubric.

## RETIRED
R16 marked `N/A — superseded by PLAYBOOK v12.3` in the Rules row and in m0_rules.md (heading and a first bullet), its
text kept, with the lead's words *"In V12.3 if we dont need git well have to remove that rule"*; a Superseded line
names the rule, the day and those words. Phase commit gates (§10) stay the lead's. This block's last message carries no
commit line.

## Flags filed
- M0-TH-adopt-a: (1) its `show` is v12.2's, so no start — its claim names `--base <SHA>` (ab36299 if none later), else
  v12.3's `--changed` reads every scope with plants once; (2) after the swap, `rung_record.py now` → the RUNGS gaps and
  the box's install line (`stale` expected while the plugin is 12.2.1) named to the lead; (3) "Using the tools" already
  written.
- M0-T27: testing.md § The calls still shows `--changed --base <commit>` and a red-arm for every task — v12.3's rows.
- M0-TM3 (in its block, at filing): the three new rungs, rule (3)'s lowest rung now Haiku 5.5, high, TH-adopt-a's
  placeholder, its own claim needing `--base` before the swap.

## Verify
[ALREADY RUN — PASS (copy PLAYBOOK.md 12.3, the Rules name 12.3) on win-laptop]
```bash
py -3.12 -c "<Verify 1's one-liner, as in the block>" milestones/m0/m0_implementation_plan.md 12.3
```
[ALREADY RUN — PASS (lint GO, 507 checks, 0 warnings, header 442/600, ladder 7 rungs, refresh 2/5) on win-laptop]
```bash
py -3.12 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md
```

## Findings measured, not acted on
- milestones.md's M0 row still reads "IN PROGRESS — next: the Phase 2 commit gate, then M0-TE-win on win-laptop" (stale
  since 2026-10-08) — the tracker is not a PLAN block's write here; M0-T116 aligns it, a gate re-checks it.
- Repo facts' "Scopes today: `plan_lint` · `tools_selftest` · `toolchain`" and "the harness is `tools/pb/verify.json`
  (three scopes, M0-TH …); no app yet" are stale since lot 1 (`verify.py list` holds the truth) — outside v12.3's delta;
  the next TC or TZ.
- The Bash env reads `CLAUDE_EFFORT=xhigh` (the picker's) while the session's requests ran at max (`now`) — §B.3: the env
  is no detector; not a mismatch.
- M0-TH-adopt-a's order clause still reads `(AFTER M0-TE-V12.3)` though M0-TM3 now runs between — plan order decides.

## The claim run
The last step, after the close (the routine's step 5): `py -3.12 tools/pb/verify.py --changed --base ab36299 --task
M0-TE-V12.3` — v12.2's tools need a named base; ab36299 (the roll-out's commit) is the last before this task. Result
below, appended after the run.

[ALREADY RUN — PASS (GO: 506 passed, 0 failed, plan_lint 1/1 scope, 19 scopes not impacted, 0.17 s; tree
ab36299eb49e+1161fb678d7f; python 3.12.10; load 4.12 → 2.76) on win-laptop]
```bash
py -3.12 tools/pb/verify.py --changed --base ab36299 --task M0-TE-V12.3
```
Its one FLAG — `FLAG type: tools/pb/verify.anchor.json changed since ab36299eb49e - a PLAN block writes no product code
and no tool` — is the harness's own write in this run: the coverage anchor advanced its two `plan_lint` entries
(milestones/m0/m0_implementation_plan.md and m0_rules.md) to this task (`git diff`: 2 lines out, 2 in; the file is in the
manifest's skip class "the harness's own manifest and anchor"). No tool written. Logs: logs/M0-TE-V12.3.plan_lint.log,
logs/loop_times.jsonl. Only this file was written after the run (a skip-class path, graded by no scope).
