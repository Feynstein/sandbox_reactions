# Sandbox Reactions — milestones

The tracker: navigation, the lead's worklist and short dated deltas (PLAYBOOK §1). Project law
lives in the current plan — `milestones/m0/m0_implementation_plan.md` today.

## Milestones
| Milestone | Goal | Size | Status | Plan |
|---|---|---|---|---|
| M0 | The sandbox engine and a star's life — time control, desktop first, a web build kept alive | L | TODO — next: M0-TI, once worklist items 1–2 are done | milestones/m0/m0_implementation_plan.md |

## Lead worklist
1. **Generate the instruction files, before M0-TI** — AGENTS.md, CLAUDE.md and GEMINI.md, from the
   playbook's §C: run PLAYBOOK.md §B.2 step 0's one line from the repo root. Claude Code's auto
   mode refused it to the bootstrap agent (2026-10-08).
2. **Add the switch's check hook, before M0-TI** (needed by M0-TH at the latest) — PLAYBOOK §B.3
   "The check": one UserPromptSubmit hook in `.claude/settings.json`. The same refusal.
3. **Commit and push the bootstrap**, after items 1 and 2 — from the repo root, one at a time:
   `git add -A` · `git commit -m "M0 bootstrap: playbook v12.2, tracker, M0 plan"` ·
   `git push -u origin main`.
4. **Once M0-TH has run** — `python3 tools/pb/rung_record.py now` says whether the installed switch
   plugin is current (this playbook's switch is 12.2.1) and prints the install line when it is
   missing or stale. On win-laptop, the switch is installed once there too.
5. **Before a Steam release** — Steamworks partner enrolment and the per-app fee (M0-R1's report
   states the current terms).

## Deltas
- 2026-10-08 · Bootstrap under PLAYBOOK v12.2: this tracker, the M0 plan (size L), m0_rules.md,
  the bootstrap record (milestones/m0/reports/bootstrap.md) and .gitignore. AGENTS.md, CLAUDE.md,
  GEMINI.md and the switch's hook are left to the lead — auto mode refused them to the agent.
