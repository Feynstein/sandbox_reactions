# M0 — Sandbox Reactions: the engine and a star's life (size class: L)
Sandbox Reactions is a falling-sand physics sandbox in the spirit of The Powder Toy: the player
places matter and watches physics and chemistry play out. M0 builds the engine and its first demo,
in sandbox mode: a cloud of gas collapses under its own gravity, heats up, ignites fusion, shines,
runs out of fuel and dies — with a time control that slows down or speeds up the world so any
stage can be watched. It runs as a desktop app on linux-pc, and a tiny web build is checked from
the first build lot so the website stays reachable. The language (C++ or Rust) and the engine (our
own or an existing one) are the lead's pick from M0-R1's research, before any code. M0-TI fixes the
star's stages, the time control and the sandbox's tools with the lead; story mode, Steam, the
Windows laptop and the rest of the physics list are routed past M0 (reports/bootstrap.md §3).
Models: Claude Code — Opus 5.5, max (gate) · Opus 5.5, high (usual) · Sonnet 5.5, high · Sonnet 5.5, medium — strongest first; read 2026-10-08 from the claude-api skill's model table (cached 2026-09-25), the probe and the lead's answer
Changes by asking: every block — the lead at bootstrap, 2026-10-08, "Any task, by asking you": physics formulas will need tuning as they get built; never a bypass that makes a failing check pass
Launch prompt: « Read milestones/m0/m0_implementation_plan.md and execute M0-<id> yourself — you
are the task agent, not an orchestrator. Grep for your own heading first. »

## Resolve with the lead first
Answered at M0-TP by the question tool (≤4 per call), each restating its consequence:
1. The M0 goal as M0-TI reshapes it, and size class L — if L holds, M0 runs the full process
   (research, contract, harness, many small build tasks, checks) before the walk-through; nothing
   is skipped to reach the demo sooner.
2. The licence stance (R7): may the game ship copyleft (GPL-family) code? — if not, The Powder
   Toy's code can never be reused in Sandbox Reactions, only its ideas.
3. The modules assumed at bootstrap (R6): OPT-B on, the rest off — with OPT-B on, one more task
   polishes the star demo before the walk-through.
4. Not at TP — listed so no block decides them earlier: the language and the engine (the lead's
   pick at the end of M0-R1); the physics forks and the dependency table (at M0-TC). Once ruled,
   every later milestone builds on them; changing them later is a rewrite.

## Flow
| Task | Type | Goal | Order | Status |
|---|---|---|---|---|
| M0-TI | PLAN + LEAD answers | The sandbox's mechanics and the star's life, from the lead | FIRST | TODO |
| M0-TP | PLAN | Red-team this plan; goal, size, modules and licence put to the lead | AFTER M0-TI | TODO |
| M0-R1 | BUILD | Engine and language research; the lead picks | AFTER M0-TP | TODO |
| M0-R2 | BUILD | Star physics research: stages, models, oracles | AFTER M0-R1 | TODO |
| M0-R3 | BUILD | Time-warp research: one world from seconds to billions of years | AFTER M0-R2 | TODO |
| M0-TC | PLAN | Physics forks and the dependency gate with the lead; the M0 contract | AFTER M0-R3 | TODO |
| M0-TH | BUILD | The toolkit and the test harness | AFTER M0-TC | TODO |
| M0-TG | PLAN | The M0 build pipeline, sized and ordered | AFTER M0-TH | TODO |
| M0-TB | PLAN | Size, token and rating pass over the pipeline | AFTER M0-TG | TODO |
| M0-TW | LEAD walk + CHECK | The user gate: the lead walks a star's life | AFTER the push that follows the final V and the TD | TODO |
| M0-TZ | PLAN | The next plan, M1 | LAST | TODO |

## Rules
PLAYBOOK.md v12.2 + the instruction file bind every agent (cited once, never per block). Full text
of every rule below: m0_rules.md. Header budget: 600 / 60. Rating refresh: 5 flags. Full-loop
budget: not measured yet — M0-TH measures it on linux-pc. OPT modules: OPT-B (assumed, R6), OPT-C
(R2–R4). Grants (OPT-C lanes: scope · exclusions · ceiling): R2 BUILD lane, R3 installs, R4 DEBUG
lane — each $0. Derogations, dated: none. Sizing bar: §2.1 — ceilings and named exceptions by id;
E blocks: 0 (M0-TG marks them). Gates to builds: 7:4 at bootstrap — the build pipeline is
M0-TG's to write; M0-TB re-counts. What a capture cannot show here: the simulation's motion and
timing (a star evolving, flows, a time warp's smoothness), frame rate and input feel, sound.

| # | Rule (one line — the annex carries the text) |
|---|---|
| R1 | A Python `.venv/` at the repo root is allowed when a task needs one (the lead, 2026-10-08) |
| R2 | Grant, BUILD lane: compile, run the deterministic suites and the smokes, fetch approved libraries — no ask; $0 |
| R3 | Grant, installs: the approved table's tools, user-level, on linux-pc and win-laptop; on win-laptop the agent does the setup itself; $0 |
| R4 | Grant, DEBUG lane: run your own build headless or off-screen to check it — never acceptance; a visible window still asks |
| R5 | Boxes: linux-pc for every M0 gate, win-laptop later — probe the box first; every verdict names its box |
| R6 | Modules assumed at bootstrap: OPT-B on; OPT-A, D, E, F, G off — M0-TP confirms with the lead |
| R7 | Copyleft code (GPL family) is studied for ideas, never copied or translated — until M0-TP's licence ruling |
| R8 | Physics oracles: a physics behaviour passes only against a cited reference result within a stated tolerance |

## Superseded / retired (§2.6)
none — purely additive (a green-field project). Inherited debt (§2.4): none.

## Repo facts (so you don't explore)
- Tracker: `milestones.md`. Paths in this plan: `reports/`, `tasks/`, `logs/`, `images/` sit under
  `milestones/m0/`; every command runs from the repo root.
- Product: Sandbox Reactions — a falling-sand physics sandbox game; sandbox mode first, story mode
  later (reports/bootstrap.md §3). Product language: English (UI strings, element names,
  messages). Plans, reports and handoffs: English.
- Repo: green field (2026-10-08) — PLAYBOOK.md v12.2 and the bootstrap's files, no code, no commit
  yet; remote `origin` = git@github.com:Feynstein/sandbox_reactions.git, branch `main`.
- Stack: not chosen — C++ or Rust, our own engine or an existing one; the lead picks at the end of
  M0-R1, and M0-TC fixes the source layout in the contract.
- Targets: desktop on linux-pc first; a tiny web build kept alive from the first build lot; Steam
  and win-laptop later. The lead: "Ideally I would for it to run on a website, or to be able to
  sell it on steam."
- Time zone for run and output folder names: America/Toronto (probed); instants inside artifacts
  are UTC-Z (§1).
- Boxes (R5): `linux-pc` — this PC, hostname laserax-ai, Ubuntu 24.04.5 LTS, kernel 7.0.0-38,
  bash, 32 CPU threads, 188 GiB RAM; GPUs NVIDIA GeForce RTX 5090 32 GB and Quadro RTX 4000 8 GB
  (driver 595.99.02) and an Intel UHD 770; OpenGL 4.6 (renderer: the Quadro), Vulkan ICDs present,
  Vulkan loader dev 1.3.275; Wayland session with XWayland on DISPLAY=:1 — in play.
  `win-laptop` — the lead's Windows laptop; not in play for M0's gates; probed when first used.
- Toolchains on linux-pc (probed 2026-10-08): present — g++/gcc 13.3.0, GNU make 4.3, ninja 1.11.1,
  pkg-config 1.8.1, Python 3.12.3, Node 20.20.2 + npm 10.8.2, git 2.43.0, jq, Xvfb + xvfb-run; dev
  libraries x11 1.8.7, xcursor, xrandr, xi, wayland-client 1.22.0, xkbcommon 1.6.0, vulkan
  1.3.275, libudev 255. Missing — rustc, cargo, rustup, clang, cmake, SDL2/SDL3 dev, ALSA dev,
  vulkaninfo. Installs follow M0-TC's dependency gate and R3.
- Python: a `.venv/` at the repo root when a task needs packages (R1); the playbook's tools need
  none.
- Agent (the probe, §B.3, 2026-10-08): Claude Code 2.1.292 (`AI_AGENT=claude-code_2-1-292_agent`,
  `CLAUDECODE=1`). The switch plugin `pb-switch@pb` is installed on linux-pc
  (`~/.claude/plugins/installed_plugins.json` lists it); `rung_record.py now` reads whether it is
  current against this playbook's switch (12.2.1) once M0-TH extracts it. Every rung of the ladder
  is in the switch's `RUNGS` table.
- Folders: `milestones/m0/` holds this plan, m0_rules.md, the contract to come (m0_contrat.md) and
  logs/ reports/ tasks/ images/ (`.gitkeep` holds the empty ones); `tools/pb/` and `docs/agent/`
  arrive with M0-TH; no source tree yet.
- Run commands, harness scopes, launchers: none yet — M0-TH writes the harness's; the first build
  block (M0-TG places it) writes the app's run commands and its double-click start/stop launchers
  (§8).
- Git: agents never write git (§10); each phase ends with the lead's commit gate.
### Using the tools — the calls a task needs (never the tool's code or its help)
- None yet: `tools/pb/plan.py` and `verify.py` arrive with M0-TH, which writes the calls here.
  Until then, edit this plan with your file-edit tool — your Status line (stamped `IN PROGRESS
  (YYYY-MM-DD HH:MM)`, then `DONE (YYYY-MM-DD HH:MM, started HH:MM)`), your Handoff, your Flow
  row's Status cell (the same state and date) and the Pipeline state, in one edit; never a script
  over the plan (§C).
### Hazards (dated, root-caused — carried forward by TZ until retired with a reason)
- 2026-10-08 · Claude Code's auto mode refused the bootstrap agent (a) extracting the playbook's
  tools to a scratch folder and running them ("Code from External") and (b) writing AGENTS.md,
  CLAUDE.md and GEMINI.md from §C ("Instruction Poisoning"). Root cause: its classifier reads
  code and auto-loaded instruction files taken from PLAYBOOK.md as external content. Effect:
  M0-TH's extraction (§B.4), and any later write of the instruction files or the switch's hook,
  may be refused too — a refusal goes to the lead as a question (approve it, or run the line
  themselves), never around it.
- 2026-10-08 · linux-pc drives its display from the Quadro RTX 4000 (glxinfo: "OpenGL renderer
  string: Quadro RTX 4000/PCIe/SSE2"); the RTX 5090 and an Intel UHD 770 are present too. Root
  cause: three GPUs, the display on the Quadro. Effect: a graphics API's default adapter may not be
  the 5090 — adapter choice is explicit (contract §6), and every GPU test and performance number
  names the adapter it ran on.
- 2026-10-08 · The desktop session is Wayland (`XDG_SESSION_TYPE=wayland`, XWayland on `:1`). Effect:
  windowing behaves differently under Wayland and X11; headless checks use offscreen rendering or
  a private Xvfb display (R4), never the lead's desktop.
- 2026-10-08 · The root filesystem is 82% full (165 GB free of 937 GB). Effect: build caches (a
  compiler's target folder, a web toolchain, shader caches) can take tens of GB — keep them in
  known, git-ignored folders; scratch copies go through `tools/pb/scratch_copy.sh` (§11).
- 2026-10-08 · No commit yet, and git keeps no empty folder: `.gitkeep` files hold
  `milestones/m0/{logs,tasks,images}/`, so a fresh clone (win-laptop) has them for the plan lint.

## Pipeline state (a register of one-line pointers — never a handoff or a history)
- Next task: M0-TI — once the lead's two bootstrap steps are done (Awaiting lead)
- Counters: T=0 · D=0 · V=0 · Q=0 · TI=0 · TJ=0 · TV=0 · TC=0 · TR=0 · TM=0
- Open D/BLOCKED register: none
- Outstanding commit gates: the bootstrap's files — not committed yet (milestones.md worklist)
- Carryover: none
- Awaiting lead: the instruction files and the switch's check hook (milestones.md worklist) — before M0-TI
- Model ratings: 2026-10-08 by the bootstrap

# Tasks

# Phase 0 — the lead's mechanics, the plan red-teamed

## M0-TI · Lead interview — the sandbox's mechanics and the star's life · **PLAN + LEAD answers** · Opus 5.5, max · switch · (FIRST)
- Status: TODO
- Ask (verbatim): "As a first demo I want to be able to simulate the life of a star. I want to be able to slow or speed up time." · "We start with the sandbox mode" (the lead, 2026-10-08 — the four messages whole in reports/bootstrap.md §1)
- Read: this file (rules + this task) + reports/bootstrap.md
- Deliver: the lead interviewed by the question tool — ≤4 questions per call, plain words, the
  mechanics and the reasons behind them, never a wish-list, never a question reports/bootstrap.md
  already settles — on: (1) the star's life, stage by stage: how a star starts (the player paints
  a gas cloud, or picks a mass), what the player sees and can do at each stage, which endings M0
  must show (white dwarf, supernova, neutron star, black hole); (2) the time control: its range,
  steps or a slider, pause and single step, what the screen shows during a fast warp; (3) the
  world: a pixel grid like The Powder Toy or another look, its size on screen, one star or
  several, real units or game units, faithful against fun; (4) the sandbox: the tools (brush,
  eraser, element menu, heat and pressure views), the elements M0 ships, whether any chemical
  reaction lands in M0 or only the reaction mechanism fusion uses, saving and loading; (5) the
  feel: frame rate, the slowest PC it must run on, the look (pixels, glow, smooth) → none — the
  interview itself; M0-TP checks the record against this block's Ask · the answers verbatim in
  reports/sandbox_interview.md, inference kept apart, each behaviour mapped MVP / later / dropped /
  open, open items routed (§2.6 P9), and a proposed M0 goal paragraph and size class for M0-TP →
  none — M0-TP rules on them with the lead
- Verify: none — a PLAN gate whose record M0-TP reads
- Adversarial: inventing a detail the lead did not say — every MVP line cites the verbatim answer
  it rests on; re-asking what the bootstrap settled (§2.6 P1) — reports/bootstrap.md §2 read first.
- Placement note: FIRST of the seed lot. Fenced: no contract edit, no fork decided (the engine is
  M0-R1's pick), no code.
- Handoff: <placeholder — ≤8 lines inline when done; detail → tasks/M0-TI.md>

## M0-TP · Red-team the plan · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TI)
- Status: TODO
- Read: this file (whole) + reports/bootstrap.md + reports/sandbox_interview.md
- Deliver: PLAYBOOK §2.1 step 1 on this plan → reports/plan_redteam.md, opened by re-verifying
  every Repo-facts line its findings rely on (probed, never from memory) · every claim attacked:
  contradictions, missing tasks, loops, false assumptions, a `Verify:` that cannot fail, a
  `Deliver:` item no check names, the gates-to-builds ratio printed, the M0 goal against
  reports/sandbox_interview.md's MVP, §2.1's flags on sight (the folders, the tracker row, a
  second law file at the root) → none — the gate's record · "Resolve with the lead first" calls
  1–3 and every change it proposes put to the lead by the question tool (≤4 per call, each
  restating its consequence), and only what is approved applied — the goal paragraph, the size
  class, R6, R7, blocks added, split or re-scoped, each rated by §0's rubric → none — M0-TB
  re-reads the plan
- Verify: none — a PLAN gate
- Adversarial: a red-team that confirms everything — its kill rate (findings refuted / raised) in
  the report; an all-confirmed table is itself a finding (§8).
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 0 closes after M0-TP — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 0: lead interview and plan red-team"`
> `git push`

# Phase 1 — research: the engine, the star's physics, the time warp

## M0-R1 · Engine and language research — C++ or Rust, our own engine or an existing one · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TP)
- Status: TODO
- Ask (verbatim): "I dont know whats the best engine to use (custom or other) but I know were going to have to build it in c++ or rust." · "Ideally I would for it to run on a website, or to be able to sell it on steam." · "Research first, then you pick (Recommended)" (the lead, 2026-10-08)
- Read: this file (rules + this task) + reports/bootstrap.md §1–§3 + reports/sandbox_interview.md
  (its MVP and its feel and performance answers)
- Deliver: milestones/m0/reports/engine_stack.md, written to §4 Rule 7 — every claim with its URL
  and `(accessed YYYY-MM-DD)` on the same line, evidence apart from opinion, what would falsify
  each recommendation → Verify 1 · `## Candidates`: at least Rust with our own engine (a GPU layer
  such as wgpu, a window and input layer, an immediate-mode UI), Rust with an existing engine (such
  as Bevy), C++ with our own engine (SDL3 with OpenGL, or WebGPU through Dawn, or bgfx), C++ with an
  existing engine (such as Godot with GDExtension, or raylib), and any other the research finds
  shipping a falling-sand or astrophysics sandbox → Verify 1 · `## Criteria`, each candidate scored
  with its sources: one code base for desktop (Linux and Windows, macOS noted) and the web
  (WebAssembly with WebGPU or WebGL2, browser threads and their price — SharedArrayBuffer and
  cross-origin isolation), GPU compute for millions of cells, CPU threads, Steam (Steamworks SDK
  bindings and their terms), headless runs and frame captures for tests, determinism, build and
  iteration speed, licences (commercial use, copyleft flagged — R7), cost, exit path, and prior art
  (The Powder Toy, Sandspiel, Noita, Universe Sandbox, Sandboxels — their stacks and licences) →
  Verify 1 · `## Recommendation`: one stack and the runner-up, the runner-up's strongest case
  written out and answered → Verify 1 · `## Dependency table` for the recommendation (name,
  licence, cost, exit path) → Verify 1 · `## UNVERIFIED`: every claim no source confirmed →
  Verify 1 · `## Ruling`: the pick put to the lead by the question tool — one call, the
  recommendation first and the runner-up second, each with its consequence in one sentence — and
  the answer verbatim → Verify 1
- Verify:
  1. the report's sections and its dated sources, from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/engine_stack.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Candidates','## Criteria','## Recommendation','## Dependency table','## UNVERIFIED','## Ruling');miss=[s for s in need if s not in t];n=len(re.findall(r'https?://\S+.*?\b20\d\d-\d\d-\d\d',t));ok=bool(t) and not miss and n>=15;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 15 dated sources')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the six sections present and at least 15 sources with their access date.
     Fail: NO-GO — no report, a missing section, or fewer than 15 dated sources. A pick the lead
     has not given yet is `IN PROGRESS (awaiting: the engine and language pick)`, never a defect.
- Adversarial: a recommendation driven by familiarity or hype — the runner-up's strongest case is
  written out and answered; a "runs on the web" claim read off a README — each web claim cites a
  working demo, a release note or an issue tracker.
- Handoff: <placeholder>

## M0-R2 · Star physics research — the stages of a star's life and the models that can run them · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R1)
- Status: TODO
- Ask (verbatim): "As a first demo I want to be able to simulate the life of a star." · "I also want to have chemical reactions, like mixing two elements with heat can produce them." (the lead, 2026-10-08)
- Read: this file (rules + this task) + reports/sandbox_interview.md (the star, world and sandbox
  answers) + reports/engine_stack.md (`## Recommendation` and `## Ruling`)
- Deliver: milestones/m0/reports/star_physics.md, written to §4 Rule 7 (a URL and `(accessed
  YYYY-MM-DD)` beside every claim) → Verify 1 · `## Stages`: the star's life as M0-TI's MVP fixed
  it, stage by stage — what drives it, its physical time scale, the condition that ends it (the
  mass thresholds of each ending M0 shows) → Verify 1 · `## Models`: for each phenomenon the MVP
  needs — self-gravity, gas pressure and flow, heat transport by conduction and radiation, nuclear
  burning (its temperature dependence and the energy released), light emission, degeneracy
  pressure, and what the chosen endings need — the candidate models for an interactive sandbox on
  the picked engine (grid or particles, direct or tree or FFT gravity, explicit or implicit heat
  steps), each one's cost per frame at the world size M0-TI set and its stability limit (CFL,
  diffusion), one recommendation each, and how nuclear burning and chemistry share one reaction
  mechanism (reactants and conditions in, products and energy out) → Verify 1 · `## Oracles`
  (R8): per recommended model, the reference result a test can check — an analytic solution (the
  free-fall time, a Lane–Emden polytrope, the heat equation's spreading Gaussian), a published
  relation (mass–luminosity, main-sequence lifetime), a measured value (the Sun's luminosity,
  radius, age) — with its source and a proposed tolerance → Verify 1 · `## UNVERIFIED` → Verify 1
- Verify:
  1. the report's sections and its dated sources, from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/star_physics.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Stages','## Models','## Oracles','## UNVERIFIED');miss=[s for s in need if s not in t];n=len(re.findall(r'https?://\S+.*?\b20\d\d-\d\d-\d\d',t));ok=bool(t) and not miss and n>=15;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 15 dated sources')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the four sections present and at least 15 sources with their access date.
     Fail: NO-GO — no report, a missing section, or fewer than 15 dated sources.
- Adversarial: a model right in a textbook but unstable or too slow at the sandbox's step sizes —
  each recommendation states its stability condition and its cost at M0-TI's world size, and one
  is worked by hand in the report.
- Handoff: <placeholder>

## M0-R3 · Time-warp research — one world from a fraction of a second to billions of years · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R2)
- Status: TODO
- Ask (verbatim): "I want to be able to slow or speed up time." (the lead, 2026-10-08)
- Read: this file (rules + this task) + reports/sandbox_interview.md (the time-control answers) +
  reports/star_physics.md (`## Stages` and `## Models`) + reports/engine_stack.md (`## Ruling`)
- Deliver: milestones/m0/reports/time_warp.md, written to §4 Rule 7 → Verify 1 · `## Problem`: the
  time scales of star_physics.md's stages against one interactive step, the arithmetic written out
  → Verify 1 · `## Prior art`: how others span them — game sandboxes (Universe Sandbox's time
  controls among them), stellar-evolution codes (MESA, the SSE fitting formulas), adaptive and
  multi-rate time stepping in simulation codes → Verify 1 · `## Options`: an adaptive global step,
  per-region sub-cycling, a settled star handed to a reduced model and back, event-driven jumps —
  each with what the player sees during a warp (the time readout, what keeps moving), its cost,
  how it fails (instability, a missed event, a star that jumps when it changes model) and the test
  that catches that failure → Verify 1 · `## Recommendation`, with M0-TI's time-control behaviour
  (range, steps, pause, single step) mapped onto it → Verify 1 · `## UNVERIFIED` → Verify 1
- Verify:
  1. the report's sections and its dated sources, from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/time_warp.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Problem','## Prior art','## Options','## Recommendation','## UNVERIFIED');miss=[s for s in need if s not in t];n=len(re.findall(r'https?://\S+.*?\b20\d\d-\d\d-\d\d',t));ok=bool(t) and not miss and n>=10;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 10 dated sources')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the five sections present and at least 10 sources with their access date.
     Fail: NO-GO — no report, a missing section, or fewer than 10 dated sources.
- Adversarial: a warp that looks smooth but breaks the conservation of energy or mass at a switch
  between models — the recommendation names the conserved quantities and the test that checks
  them across a switch.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 1 closes after M0-R3 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 1: engine pick and physics research"`
> `git push`

# Phase 2 — the contract, the harness, the pipeline

## M0-TC · The M0 contract — physics forks and the dependency gate with the lead first · **PLAN** · Opus 5.5, max · switch · (AFTER M0-R3)
- Status: TODO
- Read: this file (rules + this task) + reports/sandbox_interview.md + reports/engine_stack.md +
  reports/star_physics.md + reports/time_warp.md + PLAYBOOK.md §9 and §14.4
- Deliver: the forks the three reports leave to the lead — accuracy against speed, the endings'
  models, the time-warp design — and the dependency table (name, licence, cost, exit path),
  cleared in one question as the seed lot's dependency gate (§2.2), all put by the question tool
  (≤4 per call, plain words, recommendation first, each restating its consequence), the answers
  verbatim in reports/contract_rulings.md → none — the lead's answers are the record ·
  milestones/m0/m0_contrat.md per §14.4: §0 conventions; §1 architecture and registries (the
  modules, the order of the passes in one simulation step, the element registry, one reaction
  registry for chemistry and nuclear reactions, the solver registry, the time-control model); §2
  data model (cell or particle state, units and scales, the world's size, a save format if M0-TI
  put saving in the MVP); §3 the engine's interfaces (the step and time-control API, the headless
  entry point that loads a scene, runs N steps or until a condition and writes state and frames,
  the ready signal `tools/pb/launch.py` polls); §4 UI (every English string exact — the time
  control, the element menu, the readouts — and the design tokens); §5 non-regression guarantees,
  each with its oracle (R8), its tolerance and the planted bug that proves its check red; §6 modes
  and environments (linux-pc desktop, the web build's smoke, win-laptop later, explicit GPU adapter
  choice — Hazards); §7 documentation (the docs/agent/ pages) → none — M0-TG and M0-TB read it,
  each lot's V checks the code against it
- Verify: none — a PLAN gate; done when a builder can implement every M0 block from the contract
  alone (§9)
- Adversarial: a guarantee with no oracle, or a tolerance chosen after seeing a result — every §5
  clause names its reference result and source now, before any code, its satisfiability checked
  at freeze (§8).
- Handoff: <placeholder>

## M0-TH · Test harness and toolkit · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TC, BEFORE M0-TG)
- Status: TODO
- Read: this file (whole) + milestones/m0/m0_contrat.md §3, §5 and §6 + PLAYBOOK.md annex §A, each
  tool's section text (the source is extracted with §B.4's line, never read)
- Deliver: the toolkit extracted from annex §A into tools/pb/ with §B.4's one line (`<dest>` =
  `.`) — a refusal by the agent's own permission mode goes to the lead as a question (approve it,
  or run the line themselves), never around it (Hazards) — and every tool's selftest GO, a tool the
  stack cannot run marked `NOT RUN (<missing dependency>)` → Verify 1 · tools/pb/verify.json with
  every scope that has something to check today — `plan_lint`, `tools_selftest`, and a `toolchain`
  scope proving the picked stack builds a one-file program for the desktop and the web targets
  (the approved toolchain installed first under R3) — each with its parser, expected count, paths
  and planted bugs → Verify 1 · docs/agent/testing.md: how the block that builds a contract
  surface adds its scope (the stack's test runner, its count parser, its planted-bug pattern) and
  what each scope costs → none — text the next blocks read · `launch.json` and the launchers left
  to the first block that boots the app (M0-TG places it) → none — no app yet · the plan tool's
  lock file (`.*implementation_plan.md.lock`) and the stack's build folders in .gitignore, and a
  .gitattributes keeping LF in scripts and sources for win-laptop checkouts → none — config the
  next commit carries · the coverage anchor seeded → Verify 1 · the routine (§4 Rule 4) written
  into Rules as R9 and m0_rules.md, and the "Using the tools" part of Repo facts → none — text
  the next blocks read · one measured complete loop on linux-pc → the budget line in Rules →
  Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-TH` · Pass: GO, every scope present, the
  loop time recorded · Fail: a scope with zero cases, or a verdict not derived from counts
- Adversarial: a `toolchain` scope that passes on a box with the compiler missing — red-armed with
  the compiler's name planted wrong; a harness that only proves itself — `tools_selftest` GO is not
  the stack's GO.
- Handoff: <placeholder>

## M0-TG · Task generation — the M0 build pipeline · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TH)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md (whole — every block is sized
  against it) + reports/sandbox_interview.md (its MVP) + docs/agent/testing.md + PLAYBOOK.md §2.1
  (TB's tests) and §2.2 (TG)
- Deliver: the implementation pipeline written into this plan under "# Build", sized to §2.1's
  tests from line one — every `Deliver:` path grep-verified or marked new, kinds and failure
  classes counted, the ruling-dependency cut, one notch smaller, `E` blocks marked → none — M0-TB
  re-checks it · the first lot opening with the walking skeleton: the app builds on linux-pc and
  for the web, opens its window, answers launch.py's ready probe, runs one headless step, and ships
  its scope, launch.json and the double-click start/stop launchers (§8) → none — M0-TB re-checks
  it · a V closing each lot (E2), a TV UI/UX pass in each phase that adds or changes screens (§8),
  a final V, then OPT-B's TD (the star's life staged and polished), the push's commit gate, then
  M0-TW, then the documentation task before M0-TZ → none — M0-TB re-checks it · every heading
  rated by type alone (§0: CHECK and PLAN on the gate rung, BUILD on rule (3)'s rung without its
  list — Sonnet 5.5, high, an E block Sonnet 5.5, medium — M0-TB applies the list), with the
  switch segment after each rating → Verify 1 · each `Deliver:` item mapped to its Verify step or
  `none — <why>`, the Flow rows in run order, the Pipeline state's counters updated → Verify 1 ·
  TB's table run on TG's own output, in the handoff → none — M0-TB runs it again
- Verify: 1. `python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md` ·
  Pass: GO · Fail: NO-GO — each finding on an open block fixed, the run repeated
- Adversarial: blocks cut by theme instead of by dependency, or a lot V that cannot find anything —
  TB's table run on TG's own output, every split it would make named.
- Handoff: <placeholder>

## M0-TB · Task-size, token and rating pass · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TG)
- Status: TODO
- Read: this file (whole) + PLAYBOOK.md §0 (the rubric) and §2.1 (TB)
- Deliver: PLAYBOOK §2.1 step 2 over M0-TG's pipeline → reports/size_pass.md: every BUILD block
  through tests (i)–(vi) with its `Deliver:` paths grep-verified, splits applied and named (two or
  more → the TG defect named in the handoff and carried to the next TG as a flag), named
  exceptions on their `Sizing exception:` line each followed by a targeted V, E blocks counted →
  Verify 1 · the token half: per-block boilerplate hoisted to Rules, the header de-duplicated, the
  "considered and REJECTED (false economy)" list started in m0_rules.md, the header against its
  budget → Verify 1 · type tags audited against ids, a `Verify:` that cannot fail and a `Deliver:`
  item no check names fixed → Verify 1 · every open block rated by §0's rubric, each rule (3)
  reason or rule (4) record named in reports/size_pass.md, and the register's `Model ratings:`
  line → Verify 1
- Verify: 1. `python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md` ·
  Pass: GO · Fail: NO-GO — each finding on an open block fixed, the run repeated
- Adversarial: a size pass that finds nothing because it never opened the `Deliver:` paths — the
  count of paths grep-verified, in the report.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 2 closes after M0-TB — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 2: contract, harness and build pipeline"`
> `git push`

# Build — M0-TG writes the lots here: each closed by its V, each phase by its commit gate

# Close-out

## M0-TW · Live walk — the user gate: a star's life · **LEAD walk + CHECK scribe** · Opus 5.5, max · switch · (AFTER the push that follows the final V and the TD — M0-TG places them)
- Status: TODO
- Read: this file (rules + this task) + the launchers named in Repo facts
- Deliver: the lead drives the increment from the double-click launcher, one step at a time — a
  star from gas cloud to its end, at the time speeds the lead picks — while the scribe journals
  every defect and friction as a numbered line (TW-W1…) in reports/walk1.md, no fixes mid-walk,
  its own wrong assertions journalled too → none — the journal is the record · one triage pass
  afterwards (E10): collected first, measured once against the code, forks to the lead in blocks of
  ≤4, D blocks filed batched and placed where they run → none — the filed blocks are the record ·
  the scribe may MOVE blocks to keep the plan runnable top to bottom (E8, move only) → none — each
  move named in the handoff
- Pass: walk completed, journal triaged, register updated. Fail: the increment won't boot — a
  NO-GO defect to fix and re-walk. Approval withheld is a NO-GO, never an approval with a punch
  list.
- Handoff: <placeholder>

## M0-TZ · Next-plan authoring — M1 · **PLAN** · Opus 5.5, max · switch · (LAST)
- Status: TODO
- Read: this file (whole) + milestones.md + reports/bootstrap.md §3 + reports/sandbox_interview.md
  (its later, dropped and open lists) + reports/walk1.md
- Deliver: PLAYBOOK §2.4's TZ → milestones/m1/m1_implementation_plan.md, m1_rules.md and
  m1/{logs,reports,tasks,images}/ — Repo facts re-measured on disk and never copied, every rule,
  grant and hazard of this plan carried by citation, retired with a reason or replaced by "now
  PLAYBOOK §x", the header under budget from line one, blocks sized to §2.1 with `E` marked and
  rated (§0), the ladder re-read on the day → Verify 1 · the items routed past M0 (story mode,
  chemistry beyond M0, the rest of the physics list, Steam, win-laptop) placed in M1 or a named
  later milestone with the lead → none — the lead's ruling is the record · this plan closed clean:
  every block DONE or N/A with a pointer, any walk never played named (§2.4) → none — M1's TP
  checks the carry
- Verify: 1. `python3 tools/pb/plan.py lint --plan milestones/m1/m1_implementation_plan.md` ·
  Pass: GO · Fail: NO-GO
- Adversarial: a fact, hazard or grant of M0 dropped in silence — the inheritance ledger lists each
  one carried, retired or replaced, with its count.
- Handoff: <placeholder>
