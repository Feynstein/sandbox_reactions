# Plan red-team — M0-TP (2026-10-08, linux-pc)

PLAYBOOK §2.1 step 1 on milestones/m0/m0_implementation_plan.md, after M0-TI. Every claim in the
plan was treated as wrong until it survived; facts were re-probed on linux-pc, never taken from
memory (probe output: logs/M0-TP.log). "My reading" and "my arithmetic" mark inference.
Readers: the lead (§8 holds the questions and the rulings), M0-R1…M0-TC (the re-scopes applied),
M0-TB (the sizing of the seed BUILD blocks, §6).

**Summary.** 30 distinct claims attacked (TI's F1–F7 and 23 of this gate's own — RT8 and RT14
restate F1 and F5 and count once): 20 confirmed, 10 refuted — kill rate 10/30 (33 %). The load-bearing findings: the three research blocks predate
the interview — M0-R2 still tests against the real Sun's numbers, against the lead's "Real laws,
squeezed scale" (RT8), and ten items the interview routed to M0-R1/R2/R3 land in none of their
`Deliver:` lines (RT7); M0-R2 holds two research threads (RT11); the goal paragraph is
pre-interview, and TI's proposed successor states an open item (60 frames a second at top
speed) as a fact (RT5); two Repo facts are false — Rust 1.99 is installed (off the PATH) and
Godot 4.7 with its web export templates is installed, unlisted (RT3, RT4). Rulings (§8): the
lead approved all seven proposals as recommended — the revised goal and size L, no copyleft code,
the modules as assumed, the research rewritten, M0-R2 split into R2a/R2b, the walk-through over
all three endings, and the facts corrected — and each is applied.

## 0. Method
- Facts first: every Repo-facts line a finding relies on was probed at 09:14 EDT (§1).
- §2.1's flags on sight run as a checklist (§2); TI's plan findings F1–F7 attacked one by one
  (§3); then every block, the header and the Rules attacked for contradictions, missing tasks,
  loops, false assumptions, un-failable checks and unmapped deliverables (§4).
- The goal against the MVP, row by row (§5). The seed BUILD blocks (M0-R1, R2, R3, TH) sized
  through §2.1's tests (§6): TB runs after them, so this gate is the only one that sizes them.
- Kill rate counts claims I suspected and then tested; checklist items that came out clean
  without a suspicion are not counted as refutations.

## 1. Repo facts, re-verified (probed 2026-10-08 13:14Z on linux-pc — logs/M0-TP.log)
| Repo-facts line | Probe | Verdict |
|---|---|---|
| Repo: "no code, no commit yet" | `git log`: 9a07813, 7acd1fc; `main` = `origin/main` = 7acd1fc | STALE → RT2 |
| Remote `origin`, branch `main` | `git branch -vv`: main tracks origin/main | CONFIRMED |
| Time zone America/Toronto | `timedatectl`: Timezone=America/Toronto | CONFIRMED |
| linux-pc: laserax-ai, Ubuntu 24.04.5, kernel 7.0.0-38, bash, 32 threads, 188 GiB | hostname, uname, os-release, nproc, free, $SHELL | CONFIRMED |
| GPUs: RTX 5090 32 GB, Quadro RTX 4000 8 GB, driver 595.99.02, Intel UHD 770 | nvidia-smi, lspci | CONFIRMED — display active on the Quadro only |
| OpenGL 4.6, renderer the Quadro; Vulkan ICDs, loader dev 1.3.275 | glxinfo -B; icd.d; pkg-config | CONFIRMED |
| Wayland session, XWayland on `:1` | XDG_SESSION_TYPE, DISPLAY, WAYLAND_DISPLAY | CONFIRMED |
| Present: g++/gcc 13.3.0, make 4.3, ninja 1.11.1, pkg-config 1.8.1, Python 3.12.3, Node 20.20.2, npm 10.8.2, git 2.43.0, jq, Xvfb | `--version`, `command -v` | CONFIRMED (+ GL 1.2 and EGL 1.5 dev present; uv 0.12.3) |
| Missing: "rustc, cargo, rustup" | not on the PATH; `~/.cargo/bin/rustc` → rustc 1.99.0 (2026-09-28), cargo 1.99.0, rustup 1.29.1, toolchain stable x86_64 only (no wasm32), installed 2026-10-06; no shell rc file sources `~/.cargo/env` | FALSE → RT3 |
| Missing: clang, cmake, SDL2/SDL3 dev, ALSA dev, vulkaninfo | `command -v`, pkg-config | CONFIRMED (+ glfw3 dev, emcc, wasm-pack, glslc missing) |
| (not listed) | `godot` on the PATH → ~/Applications/godot/Godot_v4.7-stable_linux.x86_64, "4.7.stable.official"; export templates 4.7.stable incl. web, Windows, Linux | OMITTED → RT4 |
| Agent: Claude Code 2.1.292 | §B.3 probe: CLAUDECODE=1, AI_AGENT=claude-code_2-1-292_agent, CLAUDE_EFFORT=max | CONFIRMED |
| Switch plugin installed | installed_plugins.json: pb-switch@pb, user scope, 12.2.1, installed 2026-10-05 | CONFIRMED |
| Folders + `.gitkeep` | ls; `git ls-files`: logs/tasks/images `.gitkeep`, added in 9a07813 | CONFIRMED |
| Hazard: disk 82 % (165 GB free) | df: 82 %, 163 GB free | CONFIRMED (dated measure, no edit) |
| Instruction files | AGENTS.md equals PLAYBOOK §C byte for byte; CLAUDE.md, GEMINI.md = `@AGENTS.md`; the hook's line equals §B.3's | CONFIRMED |

## 2. §2.1's flags on sight
| Flag | Result |
|---|---|
| A `Verify:` that cannot fail | None: R1–R3 fail on a missing report, section or source count; TH, TG, TB, TZ on their runner or lint. R1–R3's source count bites weaker than written (RT9) |
| A `Deliver:` item no check names | None unmapped. Two nominal maps: M0-R1's `## Ruling` → Verify 1, which checks the heading only (RT10); M0-TB's reports/size_pass.md → `plan.py lint`, which reads no report (annex §A.1's lint list) (RT15) |
| A deferred item reconfirmed once (§2.6 P8) | None — first milestone |
| Gates outnumbering builds | **Gates : builds = 7 : 4 (1.75)** — TI, TP, TC, TG, TB, TW, TZ : R1, R2, R3, TH. Above 1:1; the plan's one-line why holds (the build pipeline is M0-TG's to write). Now 7 : 5 (1.4) — M0-R2 split (§8 Q5) |
| A phase adding screens with no `TV`; more than one live `TW` | None: Phases 0–2 add no screen; one TW; Rules name what a capture cannot show |
| A second law or playbook file at the root | None: AGENTS.md is §C; CLAUDE.md and GEMINI.md one line each |
| Missing `logs/ reports/ tasks/ images/` | None |
| A stale tracker row | **Flagged:** "TODO — next: M0-TI" while M0-TI is DONE (09:10) → fixed by this gate in the same edit (§1) |
| A previous plan's fact, hazard or rule dropped | n/a — M0 is the first plan |
| A header past its budget with no `TC<n>` | None: header 143 / 600 lines, Pipeline state 8 / 60 |
| Standing triggers (TL, ceilings, re-runs, modules) | TL n/a (OPT-D off) · three $0 ceilings · no pending re-run · modules → §8 question 3 |
| Lead-only steps still open | None open now: worklist item 1 done; item 2 waits on M0-TH, item 3 on a Steam release; next lead step is the Phase 0 commit gate after this block |

## 3. TI's plan findings (sandbox_interview.md §5), attacked
| # | TI's claim | Verdict | Why |
|---|---|---|---|
| F1 | R2's oracles, R3's title and `## Problem` and R8's why were written for real numbers | CONFIRMED, one clause REFUTED | R2's `## Oracles` offers "a measured value (the Sun's luminosity, radius, age)" — answer 1's text: "Tests check each law's exact answers, not the real Sun's numbers". R3 "one world from a fraction of a second to billions of years" — under answer 6 the sandbox clock spans slow motion to a life in ~10 s. R8's rule stands, its why changes. **Refuted:** "a settled star handed to a reduced model and back may no longer be needed" — RT6's arithmetic says the top speed may need exactly that; R3 keeps the option |
| F2 | All three endings: R2's `## Models` covers them | REFUTED as a defect | R2 already reads "what the chosen endings need"; no edit buys anything |
| F3 | Two edge modes: R2 covers both for gas and gravity; TC §5 counts outflow | CONFIRMED | R2's `Deliver:` names no boundary condition; open edges change the gravity solver (an isolated boundary, not a periodic one — my reading) → RT7 |
| F4 | ~240,000 cells, not millions; the Quadro's class | CONFIRMED in part | R1's criterion says "millions of cells"; I11's card claim has no named check (RT7). The hazard needs no edit now: contract §6 owns the adapter choice (TC's `Deliver:`) |
| F5 | Rules' "what a capture cannot show … sound" | CONFIRMED (minor) | M0 has no sound (answer 17) → RT14 |
| F6 | The goal paragraph predates the interview | CONFIRMED | → RT5, §5 |
| F7 | The option texts carry much of the record | REFUTED as a defect | Tested: the numbers taken from option texts are labelled "my reading", listed in I11 or written "for example" — one gap, The Powder Toy's grid size behind "600 × 400" (RT13) |

## 4. This gate's findings
Severity: **H** a later block builds on something wrong · **M** a check or a record misleads ·
**L** bookkeeping.

| # | Finding | Sev | Verdict | Proposed change |
|---|---|---|---|---|
| RT1 | Tracker row "TODO — next: M0-TI" is stale | L | CONFIRMED | Fixed by this gate (§1: the tracker is the gates' to keep) |
| RT2 | Repo facts' "no commit yet" and the Hazard "No commit yet" are stale: 9a07813 and 7acd1fc are pushed | L | CONFIRMED | Both lines re-worded (§8 Q7) |
| RT3 | Repo facts list rustc, cargo, rustup as missing; Rust 1.99.0 stable is installed in ~/.cargo/bin (2026-10-06), off the PATH, host target only. Effect: M0-R1 would price an install that is done; a scope calling `cargo` from a non-login shell fails "not found" | M | CONFIRMED | Toolchains line corrected; a new dated Hazard for the PATH (§8 Q7) |
| RT4 | Godot 4.7 stable is on the PATH with its 4.7 export templates (web, Windows, Linux) — a candidate M0-R1 names, unlisted | M | CONFIRMED | Toolchains line; M0-R1 told to price present tools as install cost saved, never as merit (its own Adversarial: "familiarity or hype") (§8 Q4, Q7) |
| RT5 | TI's proposed goal states "It holds 60 frames a second on a mid-range gaming PC" while B33 ("reachable?") is open; it drops the old paragraph's routing sentence (story mode, Steam, the Windows laptop, the rest of the physics list) and the stack-pick clause; it omits the slow-down's off switch (B11), several clouds (B15), all star matter paintable (B18), the colour by temperature (B12) and the ending readout (B2) | M | CONFIRMED | The revised paragraph, §5 (§8 Q1) |
| RT6 | The top speed at 60 frames a second may be out of reach with explicit steps everywhere. My arithmetic, every input assumed (UNVERIFIED): a life of ~300 s at ×1 → top step ×30 → 0.5 s of sandbox time per frame; a collapse watchable at ×0.1 → dynamical time ~0.1–1 s; a 100-cell star under a CFL limit → Δt ~ 0.001–0.01 s → 50–500 steps per frame; at 0.1–1 ms a step on a mid-range card → 5–500 ms against a 16.7 ms frame. The span straddles the budget, so B33 is decisive, and calm phases stay CFL-bound unless the star is handed to a reduced model or the step goes implicit | H | CONFIRMED (risk) | M0-R3's `## Problem` and `## Recommendation` carry the frame budget and a verdict (§8 Q4); a flag to M0-TG: measure one step's cost at about 600 × 400 on the Quadro in the first lot that has a solver, before the endings are built |
| RT7 | The interview routes ten items to the research blocks, and their `Deliver:` lines (bootstrap-era) name none: R1 — I11's card class and the web-size claim (B14, B35); R2 — the element list (B18), both edge modes (B20), the translation (B31), gravity's law in a flat world (B32), I11's astrophysics (B35); R3 — the auto slow-down's event detection (B11), the age translation (B31), the top speed (B33). M0-TI flagged only M0-TP and M0-TC | H | CONFIRMED | R1–R3 re-scoped (§8 Q4) |
| RT8 | M0-R2's `## Oracles` names "a measured value (the Sun's luminosity, radius, age)" as a test reference — against answer 1, "Tests check each law's exact answers, not the real Sun's numbers" | H | CONFIRMED (= F1) | Oracles in sandbox units: analytic solutions, conservation counting outflow, the orderings the squeeze keeps; real values only anchor the readouts' translation; R8's why re-worded (§8 Q4) |
| RT9 | R1–R3's Verify counts "dated sources" with `https?://\S+.*?\b20\d\d-\d\d-\d\d` — any date after a URL on the line, a date in the URL path included; their `Deliver:` asks `(accessed YYYY-MM-DD)`. Shown on synthetic lines: a URL with a release date, or a date in its path, counts 1; the tighter `\(accessed 20\d\d-\d\d-\d\d\)` counts 0 | M | CONFIRMED — NOT PROVEN (synthetic) on a real report | The regex tightened in all three (§8 Q4) |
| RT10 | M0-R1's `## Ruling` (the lead's pick, verbatim) maps to Verify 1, which checks the heading only — an empty Ruling passes | L | CONFIRMED | Verify 1 also wants a quoted answer under `## Ruling` (§8 Q4) |
| RT11 | M0-R2 fails §2.1 (iii) and (iv): the star's astrophysics (stages, thresholds, elements, the squeeze) and the simulation's numerics (solvers, stability, cost, oracles) are two research threads; the models need the stages as an artifact — the split point. With RT7's additions it also carries three failure classes (ii) | M | CONFIRMED | Split M0-R2a (star) / M0-R2b (simulation) (§8 Q5) |
| RT12 | M0-TW walks "a star from gas cloud to its end" — one ending — while the MVP promises three (B4–B6) | M | CONFIRMED | The walk drives each ending, the presets one click each (§8 Q6) |
| RT13 | "About 600 × 400 cells" stands for "like The Powder Toy" (answer 10), but The Powder Toy's own grid size is cited nowhere and I11 does not list it | L | CONFIRMED | M0-R1 reads it from The Powder Toy's source or docs (§8 Q4) |
| RT14 | Rules: "What a capture cannot show here: … sound" — M0 has no sound | L | CONFIRMED (= F5) | "sound" dropped (§8 Q7) |
| RT15 | M0-TB maps reports/size_pass.md and its rating reasons to Verify 1, `plan.py lint`, which reads the plan only | L | CONFIRMED | Mapped honestly: `none — the record the lead and the next TG read` (§8 Q7) |
| RT16 | The 60-frames promise has no check: frame rate is on Rules' "what a capture cannot show" list, so neither a TV capture nor the walk can prove it | M | CONFIRMED | Flag to M0-TC: a contract guarantee — a timed headless run on a named adapter, a named scene, the top step — the Quadro if M0-R1 confirms its class (no ask: a flag) |
| RT17 | The ending readout (B2: "how much you paint sets the star's mass, and so its ending, shown on a readout") is a prediction the physics must then honour | M | CONFIRMED | Flag to M0-TC: a §5 guarantee that each preset ends as its readout predicts, the thresholds measured in the sandbox's own physics (no ask: a flag) |
| RT18 | M0-TH carries several artifact kinds (tools, a manifest, a doc page, config, plan text) — fails §2.1 (i)? | — | REFUTED | It is PLAYBOOK §14.3's TH template, bundled by design; its one deviation (launchers left to the first booting block) is sane — no app exists yet |
| RT19 | M0-TC is too big (four reports in, the forks, the dependency gate, contract §0–§7 out) | — | REFUTED as a split | PLAN blocks split on two subjects with two consumers (§0); the contract is one subject. Risk named once: if its session runs short, the cut is after reports/contract_rulings.md, as a continuation |
| RT20 | A research red-team (`TA`) is missing before the contract | — | REFUTED | Optional at size L (§2.5); §2.6 P6: M0-TC reads the reports from outside them on the gate rung, and RT6/RT7/RT9 put the load-bearing checks inside R1–R3 |
| RT21 | R1–R3 repeat one line ("the report's sections and its dated sources, from the repo root") — the lint's three-block warn | — | REFUTED as a defect | §2.6 P6: hoisting a label catches nothing; the warn ends when R3 closes |
| RT22 | The ladder lacks a model this session's environment lists (Fable 5.1) | — | REFUTED | The ladder is the rungs the lead runs (bootstrap Q7); no rung they run changed → no edit, no re-ask (§2.6 P1). M0-TB re-reads the ladder on its day (§0) |
| RT23 | M0-R2's Ask drops "Oh and" from the lead's message 4 | — | REFUTED | An excerpt, no word changed |
| RT24 | M0-TZ's list of routed items lacks the interview's later rows | — | REFUTED | Its `Read:` names the interview's later, dropped and open lists |
| RT25 | M0-TC's "a save format if M0-TI put saving in the MVP" is stale; some block leans on the GATES lane the lead did not grant | — | REFUTED | The condition resolves itself (saving is later, B24); V blocks run deterministic suites, which need no grant (§4 Rule 4) |

Loops: none — R1 → R2 → R3 → TC → TH → TG → TB → build → TW → TZ, each reading only earlier
artifacts; the installs TH needs wait on TC's dependency gate, which runs first.

## 5. The M0 goal against the MVP (sandbox_interview.md §4, B1–B20)
TI's §6 paragraph covers B1, B3–B10, B13, B17, B19, B20 as written; it misses or overstates the
rows below. The revised paragraph closes them.

| Row | TI's §6 text | Revised |
|---|---|---|
| B2 ending readout | "as its mass decides" — no readout | "readouts name the stage and the ending the mass points to" |
| B11 off switch | "an automatic slow-down at big events" | "… that a setting turns off" |
| B12 colour | "visible cells that glow when hot" | "visible cells coloured by temperature that glow when hot" |
| B14 / B33 | "It holds 60 frames a second" (B33 open) | "The target is 60 frames a second … M0-R3 checks that the top speed can hold it, and a shortfall comes back to the lead at M0-TC" |
| B15 several clouds | absent | "several clouds may share the world, and one star's life is what M0 promises" |
| B18 star matter | "paints hydrogen gas" | "paints star matter (hydrogen, helium and the heavier elements fusion makes)" |
| Routed past M0 | drops story mode, Steam, the Windows laptop, the rest of the physics list | kept, beside the interview's later rows |
| The stack | dropped | "in C++ or Rust, on the engine the lead picks from M0-R1's research" |

**The revised paragraph:** M0 builds Sandbox Reactions' engine and its first demo, in sandbox
mode, as a desktop app on linux-pc — in C++ or Rust, on the engine the lead picks from M0-R1's
research — with a tiny web build kept alive. The player paints star matter (hydrogen, helium and
the heavier elements fusion makes), or drops a ready-made cloud of a chosen mass, into a
Powder-Toy-sized world of about 600 × 400 cells (M0-TC fixes the size), drawn as visible cells
coloured by temperature that glow when hot, and watches a whole star's life under nature's laws,
squeezed to fit one screen and minutes at normal speed: the cloud collapses under its own
gravity, heats up until fusion lights, shines, and ends as a white dwarf, a supernova leaving a
neutron star, or a black hole, as its mass decides. The star can be changed at any time — gas
added or erased, heated or cooled — and every stage comes out of the physics, never a script;
several clouds may share the world, and one star's life is what M0 promises. Time runs on fixed
speed steps, from about ten times slower than normal to a Sun-like life in about ten seconds,
with pause, single step and an automatic slow-down at big events that a setting turns off;
readouts name the stage and the ending the mass points to, and translate mass, age and
temperature into real astronomy's units; heat, element and density views and a cell inspector
show the physics at work; at the world's edge, matter leaves for good or bounces back, as the
player chooses. The target is 60 frames a second on a mid-range gaming PC — M0-R3 checks that the
top speed can hold it, and a shortfall comes back to the lead at M0-TC. One reaction mechanism,
proven by fusion, waits for the chemistry to come; more elements, saving, sound, a world size
picked by graphics card, story mode, Steam, the Windows laptop and the rest of the physics list
are routed past M0 (reports/sandbox_interview.md §4, reports/bootstrap.md §3).

**Size class L** — the lead's own pick (bootstrap Q5), with the MVP inside L's full process; no
split proposed: answer 3 put all three endings in M0 (§2.6 P1).

## 6. The seed BUILD blocks through §2.1's tests (TB runs after them)
| Block | (i) one kind | (ii) ≤2 failure classes | (iii) no switch | (iv) cut | (v) paths | (vi) a notch smaller | Verdict |
|---|---|---|---|---|---|---|---|
| M0-R1 | a report | facts wrong or undated · a biased pick | one subject, the stack | — | engine_stack.md, new | the dependency table needs the pick: stays | fits |
| M0-R2 | a report | astrophysics wrong · models unstable or slow · oracles wrong — 3 | astrophysics → numerical methods | models need the stages (an artifact): cut there | star_physics.md, new | — | **split** (RT11) |
| M0-R3 | a report | the arithmetic wrong · a warp that breaks conservation | one subject, time | — | time_warp.md, new | the age translation feeds the recommendation: stays | fits |
| M0-TH | several, by template | extraction refused or broken · the toolchain scope wrong | one thread, the harness | — | tools/pb/, verify.json, docs/agent/testing.md, .gitignore (exists), .gitattributes (new) | launch.json already left out | fits (template) |

Ratings (§0's rubric) — every open heading checked: TP, TC, TG, TB, TZ (PLAN) and TW (CHECK
scribe) → rule (2), Opus 5.5, max ✓ · R1 (a design the spec leaves open), R2 and R3 (numeric
semantics; R3 also an algorithm with no reference) → rule (5), Opus 5.5, high ✓ · TH → rule (3),
Sonnet 5.5, high ✓. A split's halves keep R2's reason → Opus 5.5, high each.

## 7. Considered, not proposed
- A research red-team `TA` (RT20) and a split of M0-TC (RT19) — reasons in §4.
- The world size, the presets' masses, the edges' default, gravity's 2D law — M0-TC's carried
  flags already route them (B29–B32, I9); nothing to add here.

## 8. Questions to the lead and the rulings (2026-10-08, two calls of the question tool)
Every proposal went to the lead; all seven were approved as recommended and applied the same day.
| # | Question (as asked) | The lead's answer (verbatim) | Applied |
|---|---|---|---|
| Q1 | Should M0's goal paragraph become the revised one in the preview (built from your interview answers), with M0 kept at size L? If yes, your walk-through and every task measure M0 against this paragraph, and M0 runs the full process — research, contract, test harness, many small build tasks, checks — before the walk-through; nothing is skipped to reach the demo sooner. | "Revised goal, keep L (Recommended)" | The header's goal = §5's revised paragraph; size L kept; "Resolve with the lead first" item 1 records it |
| Q2 | May Sandbox Reactions ever contain copyleft (GPL-family) code, such as The Powder Toy's? If no, The Powder Toy's code can never be reused in Sandbox Reactions — only its ideas. | "No copyleft code (Recommended)" | R7, index line and m0_rules.md: the ruling replaces "until M0-TP's licence ruling" |
| Q3 | Keep the optional modules as the bootstrap assumed — the show-off demo on, the rest off? If yes, one extra task stages and polishes the star's life as a demo after the final check and before your walk-through. | "Yes, as assumed (Recommended)" | R6 confirmed, index line and m0_rules.md; the Rules' OPT line |
| Q4 | Should the three research tasks (M0-R1, R2, R3) be rewritten for your 'Real laws, squeezed scale' pick and for the items the interview routed to them? If yes, the research tests the laws' own exact answers — never the real Sun's numbers — and must settle 2D gravity, the edges, the element list, the readout translation and whether top speed can hold 60 fps before the contract is written. | "Rewrite them (Recommended)" | M0-R1, M0-R2a/R2b and M0-R3 re-scoped (RT6–RT10, RT13); R8's why re-worded in m0_rules.md |
| Q5 | Should the star-physics research (M0-R2) split into two tasks? If yes, the star itself is researched first and the simulation methods second, each in its own session. | "Split in two (Recommended)" | M0-R2a (star_physics.md: Stages, Elements, Squeeze) and M0-R2b (sim_models.md: Models, Oracles), each ≥10 dated sources, each rated Opus 5.5, high — rule (5), numeric semantics, R2's own reason; Flow rows; M0-R3 and M0-TC read both; gates : builds now 7 : 5 |
| Q6 | Should your walk-through (M0-TW, the final gate) drive a star to each of the three endings? If yes, every ending M0 promises is judged by you, not only by the automated checks. | "All three endings (Recommended)" | M0-TW's `Deliver:` |
| Q7 | Apply the red-team's small corrections to the plan's facts and rules (see preview)? They fix what the probes found on this PC and change no task's scope. | "Apply them (Recommended)" | Repo facts (Repo, Toolchains) · Hazards (the `.gitkeep` line re-worded, the Rust-off-the-PATH hazard added) · Rules ("sound" dropped, the gates line) · M0-TB's mapping |

The picked options' shown text — part of what was approved:
- Q1: "The interview's proposal plus 5 fixes: 60 fps stated as a target M0-R3 checks (it is still
  open), the slow-down's off switch, all star matter paintable, several clouds allowed with one
  star promised, and the 'routed past M0' list kept (story mode, Steam, the laptop)." — the
  preview was §5's revised paragraph, word for word.
- Q2: "The game stays closed-source and sellable on Steam with no duty to publish its code. GPL
  projects like The Powder Toy are studied for ideas only, never copied or translated; an LGPL
  library only as a separately linked file (rule R7 as written)."
- Q3: "OPT-B on: a demo task stages the star's life and iterates until it isn't janky. Off: OPT-A
  (measurement anti-gaming), OPT-D (real user data), OPT-E/F (several product lines or research
  tracks), OPT-G (client work). A later plan can switch OPT-A on for performance tuning."
- Q4: "Apply the changes in the preview to the three research blocks and to rule R8's 'why'." —
  the preview listed, per block, the changes now in the plan.
- Q5: "M0-R2a: the star — stages, endings and their mass thresholds, the element list, and what
  the squeeze must keep in order. M0-R2b: the simulation — solvers, 2D gravity, the edges, cost
  per frame, stability, and the tests' exact answers. One more task; each a focused session, R2b
  building on R2a's report."
- Q6: "You walk a white dwarf, a supernova + neutron star and a black hole — the presets make each
  one click away — at the speeds you pick."
- Q7: "Repo facts, two hazards, two Rules lines and one M0-TB mapping, as listed." — the preview
  listed each line.

**Applied without a question** — bookkeeping a gate owns: the tracker's M0 row and a dated delta
(§1); carried flags into M0-TC (RT16, RT17) and M0-TG (RT6); this block's Status, Handoff and
Flow row, and the Pipeline state.

**The new research checks, red-armed** before they went into the plan: the four `Verify:`
one-liners (M0-R1, R2a, R2b, R3) run on synthetic reports in a scratch folder — 18 cases, 4 GO
(a good report; typographic quotes under `## Ruling`) and 14 planted faults NO-GO (no report, a
missing section, too few sources, release dates or dates in URL paths instead of access dates,
an unanswered `## Ruling`); the plan's copies checked byte-identical to the tested ones
(logs/M0-TP.log). NOT PROVEN (synthetic) on real reports, which do not exist yet.
