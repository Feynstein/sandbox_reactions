# Sandbox Reactions — milestones

The tracker: navigation, the lead's worklist and short dated deltas (PLAYBOOK §1). Project law
lives in the current plan — `milestones/m0/m0_implementation_plan.md` today.

## Milestones
| Milestone | Goal | Size | Status | Plan |
|---|---|---|---|---|
| M0 | The sandbox engine and a star's life — time control, desktop first, a web build kept alive | L | IN PROGRESS — next: the Phase 2 commit gate, then M0-TE-win on win-laptop | milestones/m0/m0_implementation_plan.md |

## Lead worklist
1. Done 2026-10-08 — the instruction files (AGENTS.md, CLAUDE.md, GEMINI.md) and the switch's
   check hook (`.claude/settings.json`) in place, the bootstrap committed and pushed.
2. Done on linux-pc 2026-10-08 — `python3 tools/pb/rung_record.py now` reads `plugin=installed`
   (M0-TB's session). **On win-laptop**, the switch is installed once there too, when it first joins.
3. **Before a Steam release** — Steamworks partner enrolment and the per-app fee (M0-R1's report
   states the current terms).
4. **Before the web release** — an AWS account for a small Lightsail box with a Lightsail CDN in
   front, and the AdSense H5 Games Ads application; the consent rules for ads to rule
   (milestones/m0/reports/web_hosting.md; the milestone placed by M0-TZ).
5. **The first session on win-laptop** — push the Phase 2 commit gate from linux-pc first. On the
   laptop: Git for Windows and Claude Code installed, the repo cloned from `origin`, then launch
   M0-TE-win (it probes the laptop, installs what is missing, asks you for anything that needs an
   administrator prompt), then M0-TJ3 (what runs where), then the Phase 2b commit gate.

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
- 2026-10-08 · M0-R2a, M0-R2b, M0-R3: the star, simulation and time-warp research
  (milestones/m0/reports/star_physics.md, sim_models.md, time_warp.md); Phase 1 pushed as a5065fb.
- 2026-10-08 · M0-TC: the lead ruled gravity (the 3D pull in a thin sheet), the top speed's
  fallback (30 frames a second at the top rung), stand-in laws as a backup, a "needs WebGPU" page,
  and approved the dependency table — milestones/m0/reports/contract_rulings.md. The M0 contract is
  written: milestones/m0/m0_contrat.md. Next: M0-TH.
- 2026-10-08 · M0-TG: the M0 build pipeline is written into the plan — 16 lots (Phases 3–18), a final
  validation and the demo (Phase 19), 118 build blocks, 17 validations, 3 UI/UX passes, 2 deferred
  decisions for the lead (the stand-ins, the top speed); the lead approved moving cargo's build output to
  build/target/ (contract §3.6, §6.5). Next: M0-TB, then the Phase 2 commit gate.
- 2026-10-08 · M0-TB: the size, token and rating pass, every change approved by the lead — 4 splits
  (M0-T33 and M0-T44 at their natural seams; lot 14's tuning now runs before your one recalibration
  at M0-V14, and its four whole-life checks moved to lot 15), 14 GPU-heavy builds rated up to
  Opus 5.5 high, the shared build rules hoisted into Rules R10–R14, Phases 0–1 archived to stubs
  (milestones/m0/reports/size_pass.md). Next: the Phase 2 commit gate, then M0-T1.
- 2026-10-08 · At the lead's request: Phase 2's finished tasks archived to stubs, and Phase 2b filed before
  lot 1 — M0-TE-win (the Windows laptop probed, what is missing installed, the toolkit and the toolchain
  proven there) and M0-TJ3 (the lead rules which checks run on the laptop and which stay on linux-pc).
  Next: the Phase 2 commit gate on linux-pc, then M0-TE-win on win-laptop.
