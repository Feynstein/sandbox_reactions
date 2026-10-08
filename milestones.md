# Sandbox Reactions — milestones

The tracker: navigation, the lead's worklist and short dated deltas (PLAYBOOK §1). Project law
lives in the current plan — `milestones/m0/m0_implementation_plan.md` today.

## Milestones
| Milestone | Goal | Size | Status | Plan |
|---|---|---|---|---|
| M0 | The sandbox engine and a star's life — time control, desktop first, a web build kept alive | L | IN PROGRESS — next: M0-R2a | milestones/m0/m0_implementation_plan.md |

## Lead worklist
1. Done 2026-10-08 — the instruction files (AGENTS.md, CLAUDE.md, GEMINI.md) and the switch's
   check hook (`.claude/settings.json`) in place, the bootstrap committed and pushed.
2. **Once M0-TH has run** — `python3 tools/pb/rung_record.py now` says whether the installed switch
   plugin is current (this playbook's switch is 12.2.1) and prints the install line when it is
   missing or stale. On win-laptop, the switch is installed once there too.
3. **Before a Steam release** — Steamworks partner enrolment and the per-app fee (M0-R1's report
   states the current terms).
4. **Before the web release** — an AWS account for a small Lightsail box with a Lightsail CDN in
   front, and the AdSense H5 Games Ads application; the consent rules for ads to rule
   (milestones/m0/reports/web_hosting.md; the milestone placed by M0-TZ).

## Deltas
- 2026-10-08 · Bootstrap under PLAYBOOK v12.2: this tracker, the M0 plan (size L), m0_rules.md,
  the bootstrap record (milestones/m0/reports/bootstrap.md) and .gitignore. AGENTS.md, CLAUDE.md,
  GEMINI.md and the switch's hook are left to the lead — auto mode refused them to the agent.
- 2026-10-08 · The bootstrap agent staged those four files in a scratch folder at the lead's
  request, checked against §C and §B.3; the lead moved them in and pushed the bootstrap as
  9a07813 and 7acd1fc. Next: M0-TI.
- 2026-10-08 · M0-TI: the lead interviewed, 18 answers (milestones/m0/reports/sandbox_interview.md).
  M0-TP: the plan red-teamed (milestones/m0/reports/plan_redteam.md); the lead approved the
  revised M0 goal (size L kept), no copyleft code (R7), the modules as assumed (R6), the research
  rewritten for "Real laws, squeezed scale", M0-R2 split into M0-R2a/R2b, the walk-through over
  all three endings, and the probed facts (Rust 1.99 and Godot 4.7 already on linux-pc). Next:
  the Phase 0 commit gate, then M0-R1.
- 2026-10-08 · M0-R1: the lead picked Rust with our own thin engine (wgpu, winit, egui) —
  milestones/m0/reports/engine_stack.md. M0-D1 filed: the research Verify lines can pass a report
  with no real sections. The lead's web-release direction (Lightsail, CDN, AdSense H5 Games Ads)
  recorded in milestones/m0/reports/web_hosting.md and routed to M0-TZ. Next: M0-D1, then M0-R2a.
- 2026-10-08 · M0-D1: the four research Verify 1 checks (M0-R1, R2a, R2b, R3) now count a section
  only as a real `## ` heading line, and M0-R1's reads the lead's answer from under the Ruling
  heading — applied with the lead's approval; engine_stack.md still passes
  (milestones/m0/tasks/M0-D1.md). Next: M0-R2a.
