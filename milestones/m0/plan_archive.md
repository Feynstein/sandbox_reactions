# Archived plan blocks (full text; the plan keeps the stub)

## M0-TI · Lead interview — the sandbox's mechanics and the star's life · **PLAN + LEAD answers** · Opus 5.5, max · switch · (FIRST)
- Status: DONE (2026-10-08 09:10, started 08:51)
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
- Handoff: reports/sandbox_interview.md — 18 answers verbatim (5 rounds), inference apart (§3), the
  P9 map (§4: 20 MVP · 7 later · 1 dropped · 7 open, routed), plan findings (§5), goal + size L (§6).
  MVP: real laws, squeezed scale; paint or a preset; white dwarf, supernova + neutron star, black
  hole; touch any time, stages from physics; speed steps x0.1 to a life in ~10 s, pause, step, auto
  slow-down; stage + real-equivalent readouts; pixels + glow, ~600x400 cells, 60 fps on a mid-range
  card; star matter; fusion-only mechanism; player-chosen edges; no saving, no sound. Deviation:
  round 5 asked sound and edges. Not acted on: R2/R3/R8 assume real numbers (F1); I11 unverified.
  Flags → M0-TP, M0-TC. Model, level: NOT RUN (no rung_record.py yet). tasks/M0-TI.md. Next: M0-TP.

## M0-TP · Red-team the plan · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TI)
- Status: DONE (2026-10-08 09:40, started 09:12)
- Carried flags: · [M0-TI, 2026-10-08] the lead chose "Real laws, squeezed scale" (reports/sandbox_interview.md, answer 1): M0-R2's `## Oracles`, M0-R3's title and `## Problem` and R8's why were written for real numbers — §5 F1–F7 names each change; §6 proposes the goal paragraph and size L
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
- Handoff: reports/plan_redteam.md — Repo facts re-probed (§1), §2.1's flags (§2), TI's F1–F7 and
  23 own findings attacked: 20 confirmed, 10 refuted, kill rate 10/30; gates : builds now 7 : 5.
  The lead approved all 7 proposals (§8): revised goal + L, no copyleft (R7), modules as assumed
  (R6), R1/R3 re-scoped and R2 split into R2a/R2b for "Real laws, squeezed scale", TW over all
  three endings, facts fixed (Rust 1.99 off the PATH — new Hazard; Godot 4.7 present). Tracker row
  fixed. Flags → M0-TC (60-fps check, ending readout), M0-TG (step cost early). Not acted on: a TA,
  a TC split (P6). New Verify lines red-armed, synthetic only (18 cases). Model, level: NOT RUN
  (no rung_record.py yet). tasks/M0-TP.md. Next: the Phase 0 commit gate, then M0-R1.

## M0-R1 · Engine and language research — C++ or Rust, our own engine or an existing one · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TP)
- Status: DONE (2026-10-08 09:52, started 09:43)
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
  cross-origin isolation), GPU compute for M0's world of about 600 × 400 cells (~240,000) at the
  top speed's steps per frame and for millions in later milestones (B21), CPU threads, Steam
  (Steamworks SDK bindings and their terms), headless runs and frame captures for tests,
  determinism, build and iteration speed — the tools already on linux-pc (Repo facts: Rust 1.99,
  Godot 4.7 with its export templates) priced as install time saved, never as merit — licences
  (commercial use, copyleft flagged — R7), cost, exit path, and prior art (The Powder Toy,
  Sandspiel, Noita, Universe Sandbox, Sandboxels — their stacks and licences, and The Powder Toy's
  grid size, so "like The Powder Toy" (reports/sandbox_interview.md, answer 10) has a number) →
  Verify 1 · the claims reports/sandbox_interview.md I11 leaves to M0-R1, each checked with its
  source: the Quadro RTX 4000, linux-pc's display card, against the "RTX 2060-2070" class answer 11
  named — if it holds, the 60-frames check runs on it here (contract §6) — and "about 600 x 400
  cells … light enough for a laptop and for the web build" (answer 10's option) → Verify 1 ·
  `## Recommendation`: one stack and the runner-up, the runner-up's strongest case
  written out and answered → Verify 1 · `## Dependency table` for the recommendation (name,
  licence, cost, exit path) → Verify 1 · `## UNVERIFIED`: every claim no source confirmed →
  Verify 1 · `## Ruling`: the pick put to the lead by the question tool — one call, the
  recommendation first and the runner-up second, each with its consequence in one sentence — and
  the answer verbatim → Verify 1
- Verify:
  1. the report's sections, its dated sources and the lead's pick, from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/engine_stack.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Candidates','## Criteria','## Recommendation','## Dependency table','## UNVERIFIED','## Ruling');miss=[s for s in need if not re.search(r'(?m)^'+re.escape(s)+r'\b',t)];n=len(re.findall(r'https?://\S+.*?\(accessed 20\d\d-\d\d-\d\d\)',t));m=re.search(r'(?m)^## Ruling\b.*',t);r=t[m.end():].split('\n## ',1)[0] if m else '';q=bool(re.search(r'[\"“][^\"”\n]{2,}[\"”]',r));ok=bool(t) and not miss and n>=15 and q;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n,'- answer quoted under Ruling:',q);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 15 dated sources' if n<15 else 'no answer quoted under Ruling')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the six sections present as heading lines (each `## <name>` at a line's start; a
     suffix after the name allowed), at least 15 sources with `(accessed YYYY-MM-DD)`, and the
     lead's answer quoted under `## Ruling`.
     Fail: NO-GO — no report, a missing section, fewer than 15 dated sources, or no answer quoted
     under `## Ruling`. A pick the lead has not given yet is `IN PROGRESS (awaiting: the engine and
     language pick)`, never a defect.
- Adversarial: a recommendation driven by familiarity or hype — the runner-up's strongest case is
  written out and answered; a "runs on the web" claim read off a README — each web claim cites a
  working demo, a release note or an issue tracker.
- Handoff: reports/engine_stack.md exists: 6 candidates scored on 16 criteria, 36 evidence lines (39 dated URLs), UNVERIFIED register. **Ruled by the lead: "Rust + wgpu, own engine (Recommended)"** — Rust with wgpu + winit + egui; runner-up Rust + Bevy, answered.
  Measured: wgpu runs compute on desktop and in the browser via WebGPU; Godot and raylib web builds are WebGL2-only, which has no compute. Quadro RTX 4000 = RTX 2070 class (I11 holds), so the 60-frames check can run on it. The Powder Toy's grid is 612 × 384. WebGPU is missing from Firefox on Linux.
  Deviations: the question offered a third option (C++ + Godot), because the lead named C++. plan.py and rung_record.py are absent, so the plan was edited by hand per Repo facts.
  Filed M0-D1: Verify 1 (here and in R2a/R2b/R3) gives a false GO on a report with no real headings — measured; repro in tasks/M0-R1.md.
  Not acted on: Repo facts' "Stack: not chosen" line is left for M0-TC (it fixes the layout); the web smoke on linux-pc's Chrome 155 is NOT PROVEN (no build yet).
  Verify 1: [ALREADY RUN — FAIL (no report), FAIL (ruling pending), FAIL (false NO-GO, see D1), then PASS — GO on linux-pc]; logs/M0-R1.log.
  Model and level: NOT RUN (no rung_record.py yet).
  Next: M0-D1, then M0-R2a.

## M0-D1 · The research Verify lines pass a report with no real sections · **BUILD** · Opus 5.5, max · switch · (AFTER M0-R1, BEFORE M0-R2a)
- Status: DONE (2026-10-08 10:11, started 09:58)
- Blocks: M0-R2a (its Verify 1 is the first to run next)
- Caused by: M0-TP (it wrote the four research Verify lines)
- Files: milestones/m0/m0_implementation_plan.md — the Verify 1 one-liners of M0-R1, M0-R2a, M0-R2b,
  M0-R3 (each contains `miss=[s for s in need if s not in t]`; opened and verified by M0-R1)
- Read: this file (rules + this task + the four Verify 1 lines) + tasks/M0-R1.md (the repro)
- Symptom: Verify 1 checks each section as a substring anywhere in the report and cuts the Ruling
  section at the heading text's first occurrence. False GO, measured by M0-R1 on linux-pc: a report
  whose single prose line names all six headings, then `"yes"`, plus 15 lines `https://x.example/<n>
  (accessed 2026-10-08)` → `=== GO ===`, exit 0. False NO-GO, measured on the real
  reports/engine_stack.md: a summary sentence mentioning "`## Ruling`" made the check read that
  sentence as the Ruling section → `no answer quoted under Ruling` while the answer was there.
  Suspected cause: measured — the substring test `s not in t` and `t.split('## Ruling',1)`.
- Deliver: the smallest fix — each section matched as a heading line (e.g. `re.search(r'(?m)^' +
  re.escape(s) + r'\s*$', t)`) and the Ruling body taken from that heading line — applied to all
  four Verify lines by asking the lead first (`Changes by asking: every block`; a live plan's edit,
  rule (2)); each line red-armed on the two repros above plus a GO on a correct synthetic report;
  never a weaker check. One attempt + self-verify.
- Done when: on linux-pc, each fixed Verify 1 gives NO-GO on the prose-only repro, GO on
  reports/engine_stack.md as M0-R1 left it, and the R2a/R2b/R3 lines NO-GO on a missing report.
- Handoff: The four research Verify 1 lines (M0-R1, R2a, R2b, R3) now match each section as a heading line, `re.search(r'(?m)^'+re.escape(s)+r'\b',t)`, and R1 reads the Ruling body from under its heading line; each Pass line says so. Applied by asking — the lead, 2026-10-08: "Apply to all four (Recommended)".
  Done when: [ALREADY RUN — PASS (prose-only repro NO-GO on all four, engine_stack.md GO with 39 dated sources, R2a/R2b/R3 NO-GO no report) on linux-pc]; bench 84/84, then 42/42 on the plan's own bytes; logs/M0-D1.log. The old lines also passed a `###` heading, `## Candidatess` and a quote right after an inline `## Ruling` mention — all NO-GO now.
  Deviation: `\b` after the name, not the Deliver's example `\s*$` — that form NO-GOes engine_stack.md (`## Dependency table (for C1)`, `## UNVERIFIED (refuted …)`); not `(?!\w)` either — a `!` stops a paste into an interactive bash (`event not found`, measured; flagged to M0-TG).
  Not acted on: headings only inside a code fence still pass (measured GO) — outside the smallest fix; a report has to fence its headings on purpose.
  Tools absent (plan.py, verify.py, rung_record.py): plan edited by hand; red arm by the scratch bench (/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/2378687f-6403-49ea-8717-df8ad6f950f5/scratchpad/d1), not `verify.py --redarm`; `verify.py --changed` NOT RUN (no verify.py yet — M0-TH). Detail: tasks/M0-D1.md.
  Model and level: NOT RUN (no rung_record.py yet).
  Next: M0-R2a.

## M0-R2a · Star research — the stages, the endings and the squeeze · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R1)
- Status: DONE (2026-10-08 10:24, started 10:14)
- Ask (verbatim): "As a first demo I want to be able to simulate the life of a star." (the lead, 2026-10-08) · "Real laws, squeezed scale (Recommended)" (reports/sandbox_interview.md, answer 1)
- Read: this file (rules + this task) + reports/sandbox_interview.md (§1–§4: the star, world and
  sandbox answers, I1–I11, the MVP map) + reports/engine_stack.md (`## Recommendation` and
  `## Ruling`)
- Deliver: milestones/m0/reports/star_physics.md, written to §4 Rule 7 (a URL and `(accessed
  YYYY-MM-DD)` beside every claim) → Verify 1 · `## Stages`: the star's life as M0-TI's MVP fixed
  it (B1–B7), stage by stage — what drives it, its real time scale, the condition that ends it —
  and the real mass thresholds of the three endings, each sourced, the facts I11 stated from
  memory checked ("about 8+ Suns", "about 25+ Suns", a core collapse "in about a second", a "10
  billion years" life — B35) → Verify 1 · `## Elements`: the star matter M0 paints and fusion
  makes (B18), about 8–10 elements, each with the burning stage that makes it → Verify 1 ·
  `## Squeeze`: what a squeezed world must keep for every stage and ending to come out of the
  physics (answers 1 and 4) — the order of its time scales (dynamical, thermal, nuclear) and the
  order of the endings by mass — and the real-equivalent translation proposed for mass, age and
  temperature (B31; the age with M0-R3) → Verify 1 · `## UNVERIFIED` → Verify 1
- Verify:
  1. the report's sections and its dated sources (the star), from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/star_physics.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Stages','## Elements','## Squeeze','## UNVERIFIED');miss=[s for s in need if not re.search(r'(?m)^'+re.escape(s)+r'\b',t)];n=len(re.findall(r'https?://\S+.*?\(accessed 20\d\d-\d\d-\d\d\)',t));ok=bool(t) and not miss and n>=10;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 10 dated sources')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the four sections present as heading lines (each `## <name>` at a line's start; a
     suffix after the name allowed) and at least 10 sources with `(accessed YYYY-MM-DD)`.
     Fail: NO-GO — no star_physics.md, a missing section, or fewer than 10 dated sources.
- Adversarial: real-scale numbers smuggled in as the sandbox's targets — every number in
  `## Squeeze` is an order or a ratio the sandbox keeps, and real values appear only as the
  translation's anchors ("Tests check each law's exact answers, not the real Sun's numbers",
  answer 1's shown text).
- Handoff: reports/star_physics.md exists — 25 dated sources (Pols' Utrecht notes, Heger 2003, Sukhbold 2016, NASA, OpenStax, Wikipedia); `## Stages` S1–S8″ with each stage's driver, real time scale and end state, the ending-threshold table, I11 checked; `## Elements` H He C O Ne Mg Si S Fe (+ optional Ni-56); `## Squeeze` K1–K9 as orders and ratios; the B31 translation proposed (mass: one factor; temperature: one factor or log-anchors; age: a stage clock).
  Verify 1: [ALREADY RUN — PASS (GO, 4 sections, 25 dated sources) on linux-pc]; red arm (`## Squeeze` un-headed, scratch copy) NO-GO exit 1; logs/M0-R2a.log. `verify.py --redarm/--changed` NOT RUN (no verify.py yet — M0-TH).
  Findings: I11 holds, two corrected — "10 billion years" is the main sequence (cloud to white dwarf ~11 Gyr, my arithmetic), a core collapse is milliseconds to under a second; the black-hole boundary is soft and non-monotonic (kept as one threshold); clouds below the ignition threshold become failed stars — flagged to M0-R2b, M0-R3, M0-TC.
  Not acted on: the helium flash's duration conflicts across two sources (UNVERIFIED; nothing rests on it).
  Deviation: plan.py and rung_record.py absent — plan edited by hand. Detail: tasks/M0-R2a.md.
  Model and level: NOT RUN (no rung_record.py yet).
  Next: M0-R2b.

## M0-R2b · Simulation research — the models that run the star, and their oracles · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R2a)
- Status: DONE (2026-10-08 10:47, started 10:32)
- Ask (verbatim): "I also want to have chemical reactions, like mixing two elements with heat can produce them." (the lead, 2026-10-08) · "Machinery only, via fusion (Recommended)" · "the player can choose if it leaves for good or bounces back" (reports/sandbox_interview.md, answers 15 and 18)
- Carried flags: · [M0-R2a, 2026-10-08] from reports/star_physics.md: (a) the endings exist only if a pressure holds a cold core up to a maximum mass, twice — electron then neutron degeneracy (K9; E12, E13, E19); whether B32's flat-world gravity still gives a maximum mass is UNVERIFIED — yours; (b) the white dwarf's shell ejection (S7) is wind-driven in nature (E2) — a model risk for B4 if gas, gravity and heat alone cannot puff a shell off; (c) K2's late-stage speed-up comes from a neutrino sink steeper in temperature than photon losses (E3); (d) the gap K1 needs between the dynamical, thermal and nuclear clocks is unmeasured — an oracle candidate
- Read: this file (rules + this task) + reports/sandbox_interview.md (§3 I1, I7, I8; §4 B13, B19,
  B20, B32) + reports/star_physics.md + reports/engine_stack.md (`## Recommendation` and
  `## Ruling`)
- Deliver: milestones/m0/reports/sim_models.md, written to §4 Rule 7 (a URL and `(accessed
  YYYY-MM-DD)` beside every claim) → Verify 1 · `## Models`: for each phenomenon star_physics.md's
  stages need — self-gravity, with gravity's law in a flat world (B32, I1), gas pressure and flow,
  heat transport by conduction and radiation, nuclear burning (its temperature dependence and the
  energy released), light emission, degeneracy pressure, and what each ending needs (a black
  hole's formation and its swallowing among them) — the candidate models for an interactive
  sandbox on the picked engine (grid or particles, direct or tree or FFT gravity, explicit or
  implicit heat steps), each one's cost per frame at about 600 × 400 cells and its stability limit
  (CFL, diffusion), both edge modes — matter leaving for good or bouncing back — for the gas and
  for gravity (B20, I8), one recommendation each, and how nuclear burning and chemistry share one
  reaction mechanism (reactants and conditions in, products and energy out — B19) → Verify 1 ·
  `## Oracles` (R8): per recommended model, the law's exact answer a test can check in sandbox
  units — an analytic solution (the free-fall time, a Lane–Emden polytrope, the heat equation's
  spreading Gaussian, a shock tube), a conservation law (mass and energy, counting what leaves
  through an open edge), an ordering the squeeze keeps (star_physics.md `## Squeeze`: heavier
  stars burn out sooner, the endings in order of mass) — with its source and a proposed
  tolerance; real values serve only the readouts' translation ("Tests check each law's exact
  answers, not the real Sun's numbers", answer 1's shown text) → Verify 1 · `## UNVERIFIED` →
  Verify 1
- Verify:
  1. the report's sections and its dated sources (the simulation), from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/sim_models.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Models','## Oracles','## UNVERIFIED');miss=[s for s in need if not re.search(r'(?m)^'+re.escape(s)+r'\b',t)];n=len(re.findall(r'https?://\S+.*?\(accessed 20\d\d-\d\d-\d\d\)',t));ok=bool(t) and not miss and n>=10;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 10 dated sources')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the three sections present as heading lines (each `## <name>` at a line's start; a
     suffix after the name allowed) and at least 10 sources with `(accessed YYYY-MM-DD)`.
     Fail: NO-GO — no sim_models.md, a missing section, or fewer than 10 dated sources.
- Adversarial: a model right in a textbook but unstable or too slow at the sandbox's step sizes —
  each recommendation states its stability condition and its cost at about 600 × 400 cells, and
  one is worked by hand in the report.
- Handoff: reports/sim_models.md exists — 31 dated sources (Pols, Chavanis 2007, Maclaurin-disk and thin-disk gravity papers, RKL2, FLD, Truelove, Federrath sinks, Wikipedia laws); `## Models` M1–M8: an Eulerian grid on the GPU; B32 recommended as the 3D 1/r² law inside a flat sheet (2D log gravity breaks K5 and K9) by zero-padded FFT convolution; MUSCL-Hancock gas; flux-limited diffusion under RKL2; one reaction registry (fusion now, chemistry later, B19); a Σ²→Σ^(3/2) cold pressure giving a maximum mass in the sheet; sink-particle black holes; both edge modes; a worked example; `## Oracles` O1–O16 with tolerances.
  Verify 1: [ALREADY RUN — PASS (GO, 3 sections, 31 dated sources) on linux-pc]; red arm (`## Oracles` un-headed, scratch copy) NO-GO exit 1; local checks C1–C3 in logs/M0-R2b.log. `verify.py --redarm/--changed` NOT RUN (no verify.py yet — M0-TH).
  Findings: C1 measured the gravity oracle at ≤ 1.6 % at a 40-cell radius (first order); by arithmetic the top speed needs ~25 steps per frame × ~0.88 ms ≈ 22 ms vs 16.6 ms — flagged to M0-R3; B32's fork and the oracles flagged to M0-TC; the sheet's maximum mass, shell ejection and the supernova's explosion stay UNVERIFIED.
  Lead's mid-session ask: a game picker on the website (space_tykun too) — flagged to M0-TZ; `M1-R2` filed in ../space_tykun's plan through its plan.py, lint GO, committed and pushed as 727dca8 at the lead's request.
  Deviation: plan.py and rung_record.py absent — plan edited by hand; one early scripted status edit broke this heading and was repaired at once. Detail: tasks/M0-R2b.md.
  Model and level: NOT RUN (no rung_record.py yet).
  Next: M0-R3.

## M0-R3 · Time-warp research — the squeeze and the speed range, from slow motion to a star's life in about ten seconds · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R2b)
- Status: DONE (2026-10-08 11:06, started 10:49)
- Ask (verbatim): "I want to be able to slow or speed up time." (the lead, 2026-10-08) · "Speed steps (Recommended)" · "Slow-mo to a life in ~10 s (Recommended)" · "Yes, with an off switch (Recommended)" (reports/sandbox_interview.md, answers 5, 6, 8)
- Carried flags: · [M0-R1, 2026-10-08] the stack is Rust + wgpu (reports/engine_stack.md, Ruling): price the top speed's frame budget as wgpu compute on the Quadro RTX 4000, which is confirmed RTX 2070 class (E21–E23) · "about 600 × 400 cells … light enough for a laptop and for the web build" holds only for Powder-Toy-class rules (E29, E32); for this sim's physics at top speed it is UNVERIFIED — yours to price, on desktop and in the browser through WebGPU · [M0-R2a, 2026-10-08] from reports/star_physics.md: B9's "cloud to white dwarf in about 10 seconds" squeezes ~11 Gyr, not 10 (main sequence 9–10 Gyr + ~2 subgiant and red giant + ~0.12 helium burning — my arithmetic from E2); the age readout is proposed as a stage clock (real durations of passed stages + the current stage's fraction, scaled by mass), not one factor — yours to confirm or replace (`## Squeeze`, translation) · [M0-R2b, 2026-10-08] from reports/sim_models.md (M8, worked example): steps per life ≈ (5–10 × R/Δx) × the K1 gaps — the sound speed cancels; a 15-cell star with gaps of 10 needs ~15,000 steps for its main sequence, ~25 steps per frame for a life in ~10 s; ~0.88 ms per full step at 600 × 400 (peak-bandwidth arithmetic, unmeasured) → ~22 ms per frame vs 16.6 ms. Measure it on the Quadro through wgpu; levers named there (gravity every k steps, heat only where needed, smaller presets); the K1 gap is the oracle candidate O14
- Read: this file (rules + this task) + reports/sandbox_interview.md (answers 1, 5–8, 11; §3 I2,
  I4–I6) + reports/star_physics.md (`## Stages` and `## Squeeze`) + reports/sim_models.md
  (`## Models`) + reports/engine_stack.md (`## Ruling`)
- Deliver: milestones/m0/reports/time_warp.md, written to §4 Rule 7 → Verify 1 · `## Problem`: the
  squeeze's arithmetic written out — the stages' real time scales (star_physics.md) against the
  sandbox clock the lead picked (a life in minutes at normal speed, about ten seconds at the top
  step, slow motion about ten times slower — answers 1, 5, 6), so "billions of years" lives in the
  readout only — and the top speed's frame budget: steps per frame at the top step times one
  step's cost (sim_models.md) against 60 frames a second on the mid-range card (I4, B33) →
  Verify 1 · `## Prior art`: how others span time scales — game sandboxes (Universe Sandbox's time
  controls among them), stellar-evolution codes (MESA, the SSE fitting formulas — for the age
  readout's translation too), adaptive and multi-rate time stepping in simulation codes → Verify 1
  · `## Options`: an adaptive global step, per-region sub-cycling, a settled star handed to a
  reduced model and back, event-driven jumps — each with what the player sees during a warp (the
  time readout, what keeps moving), its cost, how it fails (instability, a missed event, a star
  that jumps when it changes model) and the test that catches that failure → Verify 1 ·
  `## Recommendation`, with M0-TI's time-control behaviour mapped onto it — the speed steps, pause
  and single step (B8), the range (B9), the automatic slow-down at ignition, core collapse and
  explosion, recognised from the physics state and never steering it, with its off switch (B11,
  I2), and the age readout's translation (B31) — and a verdict on the top speed: held at 60 frames
  a second on the mid-range card, or the shortfall named as a fork for M0-TC (a lower top speed,
  or fewer frames a second during a warp — I4), never a silent cap → Verify 1 · `## UNVERIFIED` →
  Verify 1
- Verify:
  1. the report's sections and its dated sources (the time warp), from the repo root
     ```bash
     python3 -c "import os,re,sys;p='milestones/m0/reports/time_warp.md';t=open(p,encoding='utf-8').read() if os.path.isfile(p) else '';need=('## Problem','## Prior art','## Options','## Recommendation','## UNVERIFIED');miss=[s for s in need if not re.search(r'(?m)^'+re.escape(s)+r'\b',t)];n=len(re.findall(r'https?://\S+.*?\(accessed 20\d\d-\d\d-\d\d\)',t));ok=bool(t) and not miss and n>=10;print('report:',p if t else 'missing','- sections missing:',len(miss),*miss,'- dated sources:',n);print('=== GO ===' if ok else '=== NO-GO: '+('no report' if not t else 'missing sections' if miss else 'fewer than 10 dated sources')+' ===');sys.exit(0 if ok else 1)"
     ```
     Pass: GO — the five sections present as heading lines (each `## <name>` at a line's start; a
     suffix after the name allowed) and at least 10 sources with `(accessed YYYY-MM-DD)`.
     Fail: NO-GO — no time_warp.md, a missing section, or fewer than 10 dated sources.
- Adversarial: a warp that looks smooth but breaks the conservation of energy or mass at a switch
  between models or a sub-cycling boundary — the recommendation names the conserved quantities and
  the test that checks them across a switch; a top speed declared reachable without the frame
  budget's arithmetic — `## Problem` writes it out, its inputs sourced or measured.
- Handoff: reports/time_warp.md exists — 16 dated sources (SSE, BSE, MIST EEPs, MESA, Gear–Kevrekidis, reduced speed of light and of sound, GADGET-2, Dursi–Zingale, Fix Your Timestep, WebGPU limits) and measurements M1–M4: a scratch wgpu 30 bench (built offline from the cargo cache, headless; source in the session scratchpad, not the repo) put a whole-world 600 × 400 step at 2.13 ms on the Quadro (R2b estimated 0.88), the Sun-like star's box at 0.11 ms, 2.65 µs per dispatch. Steps per life re-derived: ~22,500 / ~63,000 / ~340,000 (scenarios L/C/H by the unmeasured K1 gap). Verdict: the top speed is NOT shown held — reachable only by computing the star's box (L held, C at the edge, H ~5× over); fork F1 to M0-TC with a measured decision point.
  Recommended: one adaptive CFL step, speed in sandbox time, an active box, gravity every k steps, a GPU event latch for the slow-down, an SSE-style monotonic stage-clock age; projective jumps as the fallback; sub-cycling and a reduced model rejected.
  Not acted on: browser WebGPU overhead and the render reserve are unmeasured (no wasm32 target, no drawing code) — UNVERIFIED. Deviation: the IN PROGRESS edit was a Python script over the plan (landed cleanly; a §C defect, declared); later edits by hand.
  Verify 1: [ALREADY RUN — PASS (GO, 16 dated sources) on linux-pc]; red arm NO-GO exit 1 (logs/M0-R3.log). Model and level: NOT RUN (no rung_record.py yet). Detail: tasks/M0-R3.md.
  Next: the lead's Phase 1 commit gate (below), then M0-TC.

## M0-TC · The M0 contract — physics forks and the dependency gate with the lead first · **PLAN** · Opus 5.5, max · switch · (AFTER M0-R3)
- Status: DONE (2026-10-08 12:05, started 11:14)
- Carried flags: · [M0-TI, 2026-10-08] reports/sandbox_interview.md §4 routes here the default edge mode (B29), how a preset's mass is chosen (B30), the real-equivalent translation and the inspector's units (B31), gravity's law in a flat world (B32, with M0-R2), and proposes keeping the world size a setting (I9) — each ruled or put to the lead · [M0-TP, 2026-10-08] the 60-frames target has no check yet — frame rate is on Rules' "what a capture cannot show" list, so neither a TV capture nor the walk can prove it: a §5 or §6 guarantee measured by a timed headless run on a named adapter (the Quadro RTX 4000 if M0-R1 confirms its class), a named scene, the top speed step (reports/plan_redteam.md RT16) · [M0-TP, 2026-10-08] the ending readout ("how much you paint sets the star's mass, and so its ending, shown on a readout", answer 2's shown text) is a prediction: a §5 guarantee that each preset ends as its readout predicts, the thresholds measured in the sandbox's own physics, never taken from real astronomy (reports/plan_redteam.md RT17) · [M0-R1, 2026-10-08] from reports/engine_stack.md: (a) the Quadro RTX 4000 is confirmed RTX 2070 class (E21–E23), so the 60-frames guarantee above names it; (b) a browser without WebGPU (Firefox on Linux today, E3) gets a "needs WebGPU" message or a CPU fallback — a fork for the lead, recommended: the message; (c) GPU floats differ across GPUs (WGSL allows fused multiply-add and flushed denormals, E20) — R8's tolerances, and whether a CPU f64 reference of each law is the oracle for the GPU path; (d) Repo facts' "Stack: not chosen" now reads ruled — update it with the source layout; (e) the dependency gate starts from the report's `## Dependency table`, its UNVERIFIED licences (Rust, wasm-bindgen, wasm-pack/trunk) to be confirmed · [M0-R2a, 2026-10-08] from reports/star_physics.md: (a) a cloud below the ignition threshold (~0.08 Suns real) becomes a failed star that cools — the physics will show it though it is not one of the lead's three endings; and the lightest stars outlive the universe in nature (E10) but play out in the squeezed sandbox — a label, and whether a preset may sit there, are yours; (b) the real black-hole boundary is soft and non-monotonic (E3, E6) — the report keeps one threshold as the goal's "as its mass decides": confirm; (c) the B31 translation, proposed: mass one factor anchored on the sandbox's white-dwarf ceiling = 1.44 Suns (painting stays additive), temperature one factor if the ignition ratios hold else log-anchors, age a stage clock with M0-R3 · [M0-R2b, 2026-10-08] from reports/sim_models.md: B32's fork for the lead — recommended B, the 3D 1/r² law inside a flat sheet (keeps K5 "contraction heats" and K9 "a maximum mass", the latter by my arithmetic, UNVERIFIED), against A, 2D log gravity (breaks both) and C, axisymmetric (painting makes rings); the models M1–M8 and the oracles O1–O16 with proposed tolerances for R8; small forks: light still leaves in bounce mode (M7), the readout's surface temperature as T_eff from L and the perimeter (M4); model risks for B4 shell ejection and B5's explosion stay UNVERIFIED · [M0-R3, 2026-10-08] from reports/time_warp.md: (F1) the top speed is NOT shown held — measured on the Quadro (wgpu 30, Vulkan), a whole-world step costs 2.13 ms (~6 steps per frame), the Sun-like star's box 0.11 ms (~115); a life in ~10 s needs ~38 / ~105 / ~560 steps per frame as the unmeasured K1 gap is small / central / large (L/C/H). Put to the lead with its decision point (the first solver lot measures steps per life, O14): if over budget, (a) 30 frames a second on the top rung only [recommended first], (b) projective jumps in quiet phases (R6), (c) a lower top speed — a life in ~1 min re-opens answer 6 (B28), a new ask; (F2) small forks: no auto-return after a slow-down, ×1 = 1 τ_dyn of the Sun-like preset per second, a 1-3-10 rung ladder ×0.1…×100, a ~3 ms render reserve to measure; the age readout recommended as a monotonic stage clock (EEP milestones, SSE fractional age on mass change, R7); the warp-invariance, box and gravity-cadence guarantees T1–T3 (R5) for §5
- Read: this file (rules + this task) + reports/sandbox_interview.md + reports/engine_stack.md +
  reports/star_physics.md + reports/sim_models.md + reports/time_warp.md + PLAYBOOK.md §9 and
  §14.4
- Deliver: the forks the four research reports leave to the lead — accuracy against speed, the endings'
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
- Handoff: The lead ruled 5 forks by the question tool, all on the recommended option, verbatim in reports/contract_rulings.md: Q1 gravity = the 3D pull in a thin sheet · Q2 the top rung may drop to 30 frames a second (still short → back to the lead) · Q3 stand-in laws as a backup only (S1 dust opacity, S2 thermal bomb, each enabled by a PLAN amendment on a measured failure) · Q4 a "needs WebGPU" page · Q5 the dependency table approved, versions pinned, every licence sourced.
  m0_contrat.md exists, §0–§7: layout, registries, the step's pass order, formulas with bounded initial constants, the calibration file and procedure (every measured number lives there, never in the contract), headless/bench/calibrate/capture/status interfaces, a keyed ```strings table, tokens, and 38 guarantees, each with oracle, tolerance fixed now, scope and plant; 12 reversible defaults D1–D12 in the rulings report §3.
  Deviations from the research, each with its reason there: the mass readout uses anchors, not one factor (a giant must fit the screen under the sheet's T ∝ M/R) · ignition-temperature ratios squeezed (Si/H ≤ 10) · neutron matter is a species · S1 is a dust opacity, not a Reimers wind (that would break G-WARP's determinism).
  Not acted on: 16 guarantees' satisfiability awaits first runs (§5.5 register), including WebGPU in headless Chrome and lavapipe on Xvfb; two q-ratios rest on Q-values from memory (marked UNVERIFIED). Phase 1's commit gate was already pushed (a5065fb) — Pipeline state corrected.
  One defect of mine: M0-TG's flag was appended to the plan by a 5-line Python script, not the edit tool (the diff shows that one segment only). Done-when self-check is read-through only, NOT PROVEN (source only). No suite ran (no code). Detail: tasks/M0-TC.md.
  Model and level: NOT RUN (no rung_record.py yet). Next: M0-TH (flags carried into M0-TH and M0-TG).

## M0-TH · Test harness and toolkit · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TC, BEFORE M0-TG)
- Status: DONE (2026-10-08 12:16, started 12:07)
- Carried flags: · [M0-TC, 2026-10-08] from m0_contrat.md: (a) §5 names 38 verify.json scopes (`grav-force` … `web`), each with its own plant kept as tests/plants/<plant>.patch for `--redarm` — the scopes arrive with the lots that build their passes; the two that need no physics, `licences` (G-LIC) and `strings` (G-STR, reading §4.1's ```strings block), can be armed as soon as code exists; (b) red-arm scratch copies share one CARGO_TARGET_DIR, `.cache/redarm-target/` (§6.5 — the disk is 82 % full), so a plant rebuilds only the workspace crates; (c) tools/pb/capture_web.mjs launches Chromium without flags, so WebGPU captures and G-WEB go through the project's web/smoke.mjs (§6.2.3) — the no-WebGPU page captures through capture_web.mjs as is; (d) launch.json's four services are §3.6 (`game` asks, `game-xvfb` runs under the private Xvfb with lavapipe, `headless-boot` exit-kind, `web` port-kind); (e) ⏱ scopes can pass 10 minutes on the Quadro — the manifest's timeouts and boxes per §6.4
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
- Handoff: Toolkit extracted (10 files, 0 refused) and every selftest GO — capture_web's live tier NOT RUN (no Playwright yet). tools/pb/verify.json: plan_lint (49 cases) · tools_selftest (10) · toolchain (3: rustc 1.99 desktop, wasm32, trunk 0.21.14 page), each red-armed (plan_lint 1/1, tools_selftest 1/1, toolchain 3/3 plants red, clean copies GO); anchor seeded (21 covered · 30 skipped · 0 unclassified); docs/agent/testing.md; R9 + m0_rules.md; .gitignore, .gitattributes. Verify 1 [ALREADY RUN — PASS (3/3 scopes GO, 62 cases, loop 3.3 s warm) on linux-pc]; budget line 600 s in Rules.
  R3 installs: wasm32 target and trunk 0.21.14 (release tarball, sha256-checked, ~/.cargo/bin/trunk). Deviations: launch.json/launchers left to the first runnable lot as Deliver says; plan_lint's plant is a command (the plan moves).
  Not acted on: scratch copies carry a repo-root target/ and .cache/ (verify.py EXCLUDES omit them; contract §6.5 expects .cache/redarm-target/) — flagged to M0-TG, written in testing.md. My defects: the first status stamp was a Python script over the plan (plan.py did not exist yet) and R9 in m0_rules.md was added by script; Verify 1's loop ran 4 times (3 redundant).
  Model and level: model=claude-sonnet-5-5 level=high (rung_record.py now), the heading's rung. Detail: tasks/M0-TH.md. Next: M0-TG.

## M0-TG · Task generation — the M0 build pipeline · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TH)
- Status: DONE (2026-10-08 13:40, started 12:36)
- Carried flags: · [M0-TP, 2026-10-08] the top speed at 60 frames a second is M0's hardest promise (reports/plan_redteam.md RT6, M0-R3's verdict): the first lot that has a solver measures one step's cost at about 600 × 400 cells on the named adapter, before any ending is built, so a shortfall surfaces early · [M0-D1, 2026-10-08] a Verify one-liner pasted into an interactive bash stops at a `!` before a letter or a backslash — `(?!x)` and `{x!r}` print `event not found`, `!=` passes (measured, logs/M0-D1.log); the research lines use `\b` for that reason · [M0-R3, 2026-10-08] from reports/time_warp.md: computing only the star's box (A2) and the GPU event latch for the slow-down (R4) are first-lot architecture, not late optimisation — whole-world computing holds no scenario (2.13 ms per step measured on the Quadro); small boxes are dispatch-bound (2.65 µs per dispatch, 29 per step), so fewer, fused kernels are the lever; the first solver lot measures steps per life (O14) and the box's step cost on the Quadro — the decision point of M0-TC's top-speed fork · [M0-TC, 2026-10-08] from m0_contrat.md and reports/contract_rulings.md: (a) first-lot architecture, not late optimisation: the active box (§1.3.3) and the GPU latch (§1.8.5), and each pass's CPU f64 twin before its GPU shader (D11, G-REF); (b) calibration (§2.11) is a lot of its own, after the solver lots and before the preset and readout lots that read it; the decision points it measures are PLAN amendments — Q2's 30-frames switch (G-TOP red at 30 frames goes back to the lead) and Q3's stand-ins S1/S2 (enabled only on §1.6.4's measured failures); (c) calibrate and the ⏱ scopes can pass 10 minutes on the Quadro: the lead's runs, placed in V windows, split by `calibrate --only` (§6.4); (d) §5.5's UNVERIFIED register is the first lots' risk list — a red row is a D or a question to the lead, never a looser number (§0.4); G-FIT's squeezed mass ladder and K1's gap are the first full-life lot's tuning targets; (e) render_reserve_ms needs one windowed `--measure-ui` run by the drawing lot, asked of the lead (§3.3); (f) the gravity cadence (§1.4.4, G-CAD) is optional — place it only if the budget needs it; (g) each lot's Deliver adds its own §5 scopes and plants · [M0-TH, 2026-10-08] (a) verify.py's scratch copy (--redarm) leaves out by name only .git .venv venv node_modules models data var output __pycache__ .pytest_cache dist build — a repo-root target/ and contract §6.5's .cache/redarm-target/ ARE copied into every red-arm arm (the disk is 82 % full); until a PLAN task settles it (cargo's target-dir under build/ via .cargo/config.toml, or an absolute shared CARGO_TARGET_DIR outside the tree, with §3.6/§6.5 amended), every lot's cargo scope must keep its build output under build/ or outside the tree — docs/agent/testing.md Hazards; (b) the toolchain scope (tests/toolchain/check.sh) already proves rustc 1.99, wasm32 and trunk 0.21.14 + wasm-bindgen 0.2.129 build a one-file program, so the first lot's web target adds crates, not tooling; (c) capture_web.mjs's live tier is NOT RUN until web/package.json brings Playwright; launch.json and the launchers are still the first runnable lot's (contract §3.6)
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
- Handoff: Pipeline written under # Build: 141 blocks filed through plan.py append, forward order — 16 lots as Phases 3–18 (118 T incl. M0-T52a/b and M0-T93a/b, 16 lot V's, TV1–TV3 for the three phases that change screens), Phase 19 (final M0-V17, OPT-B M0-TD, the push gate), M0-T116 the docs task between M0-TW and M0-TZ; DEFERRED: M0-TJ1 (stand-ins), M0-TJ2 (top speed), M0-T114 (cadence). Verify 1 [ALREADY RUN — PASS (plan.py lint GO, 471 checks) on linux-pc]; claim run --changed --base a5065fb [ALREADY RUN — PASS (3/3 scopes, 484 cases, 3.3 s) on linux-pc], its FLAGs naming only M0-TH's uncommitted tools.
  Contract amended with the lead's yes ("Approve all three (Recommended)"): §3.6 and §6.5 — cargo builds into build/target/, red-arm copies may share ~/.cache/sandbox-reactions/redarm-target/ for dependencies only; docs/agent/testing.md's hazard line; all tagged [M0-TG].
  Flags answered (tasks/M0-TG.md): box and latch in lot 3, each twin before its shader, the Quadro step cost in M0-T31, O14 in M0-T63 (M0-V8 may wake M0-TJ2 early), calibration as lot 9 before presets and readouts, ⏱ and calibrate runs in V windows, render_reserve via M0-T95 and M0-V13.
  TB's table on my output: 0 splits left by my count — I split M0-T52 and M0-T93, moved the calibrate command into M0-T66, marked 8 E blocks; ≤6 files a block (manifest and plants apart); 7 blocks near 600 lines named with fallbacks — the 122-row table in tasks/M0-TG.md, flagged to M0-TB.
  Deviations: the plan restored once from my own pre-filing copy and re-filed (append writes a Flow row's Order from --after: the reverse-order try left 138 rows reading AFTER M0-TB); phase headings and commit gates added by hand (no tool call for them); one redundant claim run, to list the FLAGs.
  Not acted on: G-WEB's 'not all black' passes on plain bg.space (M0-T94 and M0-V13 name it); cold pressure worth about a third of the Sun-like cloud's support at the start (my arithmetic — M0-T63 and M0-T97 measure it).
  Model and level: model=claude-opus-5-5 level=max (rung_record.py now), the heading's rung. Detail: tasks/M0-TG.md. Next: M0-TB.

## M0-TB · Task-size, token and rating pass · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TG)
- Status: DONE (2026-10-08 14:30, started 13:48)
- Carried flags: [M0-TG, 2026-10-08] tasks/M0-TG.md holds TB's table run on M0-TG's output (122 BUILD blocks, one row each): files counted without the verify.json entry, the plants and one-line registrations; max 6 files (M0-T1, config of 3–40 lines); seven blocks near 600 lines by estimate (M0-T30, T33, T34, T41, T44, T56, T58), each with its fallback split; left to your judgment: M0-T33 (riemann.rs could leave first) and M0-T98/T99 (a scope and its tuning in one thread); 8 E blocks marked (M0-T6, T39, T48, T52b, T91, T93a, T94, T111); the Build preamble is per-block boilerplate hoisted once — Rules or keep, your token half's call
- Read: this file (whole) + PLAYBOOK.md §0 (the rubric) and §2.1 (TB)
- Deliver: PLAYBOOK §2.1 step 2 over M0-TG's pipeline → reports/size_pass.md: every BUILD block
  through tests (i)–(vi) with its `Deliver:` paths grep-verified, splits applied and named (two or
  more → the TG defect named in the handoff and carried to the next TG as a flag), named
  exceptions on their `Sizing exception:` line each followed by a targeted V, E blocks counted →
  Verify 1 for the plan edits (the splits, the `Sizing exception:` lines, the V blocks);
  reports/size_pass.md itself → none — the record the lead and the next TG read (`plan.py lint`
  reads the plan, never a report) · the token half: per-block boilerplate hoisted to Rules, the
  header de-duplicated, the "considered and REJECTED (false economy)" list started in m0_rules.md,
  the header against its budget → Verify 1 · type tags audited against ids, a `Verify:` that
  cannot fail and a `Deliver:` item no check names fixed → Verify 1 · every open block rated by
  §0's rubric and the register's `Model ratings:` line → Verify 1 (the lint checks each rating
  against the ladder); each rule (3) reason or rule (4) record named in reports/size_pass.md →
  none — the report's record
- Verify: 1. `python3 tools/pb/plan.py lint --plan milestones/m0/m0_implementation_plan.md` ·
  Pass: GO · Fail: NO-GO — each finding on an open block fixed, the run repeated
- Adversarial: a size pass that finds nothing because it never opened the `Deliver:` paths — the
  count of paths grep-verified, in the report.
- Handoff: reports/size_pass.md: every open BUILD block through §2.1 (i)–(vi), 126 rows; 495 Deliver paths grep-verified before the edits, 510 after; 3 wrong "(new)" labels fixed. The lead approved all four proposals ("… (Recommended)" ×4, verbatim in §10).
  Splits — 4, so the TG defect is named (test (vi) stopped at the ceilings), flagged to M0-TZ for M1's TG: M0-T33 → T33a (Riemann oracle) + T33b · M0-T44 → T44a (rate law) + T44b · M0-T98/T99 → tuning T98a/T99a in lot 14, checks T98b/T99b in lot 15 with T100/T101 — as written G-CAL refused the presets after the tuning, so four checks could not pass; M0-V14 holds the one recalibration and remakes the two dumps.
  Ratings: 14 builds → Opus 5.5 high, each with its rule (3) reason (§9); 104 Sonnet 5.5 high; E counted 8 (Sonnet 5.5 medium); gates Opus 5.5 max; rule (4): no record yet; register Model ratings: 2026-10-08 by M0-TB.
  Token half: Build preamble → Rules R10–R14 + m0_rules.md (20 lines byte-identical — `plan.py show` never printed it); Resolve list → pointer; Phases 0–2's finished blocks stubbed (plan_archive.md); REJECTED list (10); header within budget. No named exceptions, no un-failable Verify; 13 unchecked Deliver items given a check or an owner.
  Flags: M0-T5/T7 (one eframe App), M0-T13 (bin-only crate test), M0-T98a/T99a (G-CAL claim-run red → ask), M0-TZ (the TG defect). Deviation: renames, ratings, order fixes, Phase 2b's heading and gate by the Edit tool (no plan.py call); appends, moves, stubs, flags by plan.py.
  After the close, the lead: "Yes you can stub them, and then can you write a task after TB" — Phase 2's TC, TH, TG, TB stubbed; filed M0-TE-win (win-laptop: probe, installs, the toolkit and toolchain proven — BUILD, Sonnet 5.5 high, rule (3)) and M0-TJ3 (what runs where — PLAN + LEAD answers, Opus 5.5 max, rule (2)) as Phase 2b, its own gate, M0-T1 after them.
  Verify 1 [ALREADY RUN — PASS (lint GO, 474 checks, 0 warnings, at the close; 479 checks after the filing) on linux-pc]; plan_lint GO, red-armed, re-run after the filing (479 passed); claim run --changed --base a5065fb [ALREADY RUN — PASS (3/3 scopes, 487 cases, 3.3 s) on linux-pc], its FLAGs naming only M0-TH's uncommitted tools.
  Model and level: model=claude-opus-5-5 level=max (rung_record.py now), the heading's rung. Detail: tasks/M0-TB.md. Next: the lead's Phase 2 commit gate on linux-pc, then M0-TE-win on win-laptop.
