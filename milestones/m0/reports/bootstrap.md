# Bootstrap record — Sandbox Reactions (2026-10-08)

What the bootstrap settled with the lead (PLAYBOOK §B.2), in the lead's own words, and what it
assumed. M0-TI reads this first: nothing settled here is asked again (§2.6 P1). Inference is
labelled "my reading".

## 1. The lead's words, verbatim, in order (2026-10-08)
1. "read this. My project is: I want to create a game just like "The powder toy", a sandbox with
   physics. I want light physics, thermal physics, phase transitions, nuclear reactions, fluid
   dynamics, heat transfer, black hole simulations, gravity lensing, n-body, as much, electricity,
   circuits, photons, xrays, nuclear decay, neutrons, fusion, fission, as much physics as we can.
   As a first demo I want to be able to simulate the life of a star. I want to be able to slow or
   speed up time. I dont know whats the best engine to use (custom or other) but I know were going
   to have to build it in c++ or rust. Boostrap it."
2. "we can create a .venv here in this folder for python if we need it"
3. "Ideally I would for it to run on a website, or to be able to sell it on steam."
4. "the game will be named sandbox reactions. Oh and I also want to have chemical reactions, like
   mixing two elements with heat can produce them. We could have a story mode where people
   discover elements or something and a real sandbox mode. We start with the sandbox mode"

## 2. The interview — two rounds of the question tool
| # | Question | The lead's answer (verbatim) | Where it landed |
|---|---|---|---|
| 1 | How should we pick the language (C++ or Rust) and the engine (our own, or an existing one)? | "Research first, then you pick (Recommended)" | M0-R1 researches; the lead picks at its end, before any code |
| 2 | Where must the first demo (the star's life) run? | "Desktop first, web kept alive (Recommended)" | the plan's goal; Repo facts › Targets |
| 3 | Which language should the game's menus and texts use? | "English (Recommended)" | Repo facts › Product language |
| 4 | Which machines will build and test the game? | "this pc only for now, but I also work on a windows laptop, so when an agent finds out hes on windows, he drains the installs and the setup" | Rules R3, R5 — my reading: on the Windows laptop, the agent does the installs and the setup itself |
| 5 | How big should the first milestone (M0) be? | "Large: engine, time control, star's life (Recommended)" | size class L |
| 6 | When a task finds a wrong formula or rule in the contract or the plan, who may propose the fix? | "Any task, by asking you (Recommended)" | `Changes by asking: every block` |
| 7 | Which models and levels should the tasks recommend (the ladder)? | "Opus max checks, Opus high builds (Recommended)" | the `Models:` line |
| 8 | What may agents run on their own, without asking you each time? | "Build and test (Recommended), Install approved tools, both PCs (Recommended), Run the game off-screen (Recommended)" | Rules R2, R3, R4 |

The options as the lead saw them — their descriptions are part of what was approved:
- Q1 "Research first, then you pick": "A research task compares C++ vs Rust and our own engine vs
  existing ones: web + Steam reach, GPU physics, licences, cost, exit path. You choose from its
  recommendation before any code is written."
- Q2 "Desktop first, web kept alive": "The demo runs as a desktop app on this PC. A tiny web build
  is checked early so the website option never silently breaks. Steam packaging comes later."
- Q5 "Large": "Size L, the full process: research → contract → test harness → many small build
  tasks → your walk-through. M0 ends when you watch a star form, burn and die in the sandbox, at
  the speed you pick."
- Q6 "Any task, by asking you": "Any task may propose a small lasting change and applies only what
  you approve — never a shortcut that makes a failing test pass. Physics formulas will need tuning
  as they get built."
- Q7 "Opus max checks, Opus high builds": "Checks and planning on Opus 5.5 at max; most building on
  Opus 5.5 at high; simpler builds on Sonnet 5.5 (half Opus's price)."
- Q8 "Build and test": "Compile, run the test suites and fetch the approved libraries on their
  own." · "Install approved tools, both PCs": "Like your Windows rule: an agent installs what the
  approved list names (e.g. the Rust toolchain in your home folder). Anything paid still asks." ·
  "Run the game off-screen": "Agents run their own build headless to check their work — never as
  your acceptance. Opening a visible window still asks you each time." Not picked: "Run the
  milestone's check scripts" (the GATES lane).

## 3. Behaviours, mapped (PLAYBOOK §2.6 P9) — M0-TI refines the MVP with the lead
| Behaviour (the lead's words) | Tier | Routed to |
|---|---|---|
| "simulate the life of a star", "as a first demo" | MVP | M0 — its user gate is M0-TW |
| "slow or speed up time" | MVP | M0 |
| "We start with the sandbox mode" | MVP | M0 |
| "Desktop first, web kept alive" | MVP | M0: the demo on desktop, a tiny web build checked early |
| The physics the star needs, from the list: gravity and n-body, thermal physics, heat transfer, fusion and nuclear reactions, photons and light, fluid dynamics of a gas; black holes and neutrons if a massive star's end is in M0 | MVP — the subset M0-TI fixes | M0 |
| "chemical reactions, like mixing two elements with heat can produce them" | open — does M0 ship any chemical reaction, or only the reaction mechanism fusion uses (my reading: one mechanism serves both) | M0-TI asks |
| "a story mode where people discover elements" | later — no code in M0 | a milestone after M0, named by M0-TZ |
| "be able to sell it on steam" — packaging, store page, Steamworks | later — no code in M0 | after M0; store enrolment on the tracker's worklist |
| "run on a website" — a full web release | later, beyond M0's web smoke | after M0 |
| The rest of the list: electricity, circuits, X-rays, nuclear decay, neutrons, fission, black holes, gravitational lensing, phase transitions, fluid dynamics beyond the star — "as much physics as we can" | later — no code in M0 beyond what the star needs | M1 onward, ordered by M0-TZ with the lead |
| The Windows laptop as a build box | later | when the lead first runs a task there (R3, R5) |

## 4. Assumed by the bootstrap (declared, reversible at M0-TP)
- OPT-B (show-off demo) on — the lead asked for the star's life "as a first demo"; M0-TG places a
  TD after the final V.
- OPT-A, OPT-D, OPT-E, OPT-F and OPT-G off — no measurement campaign yet, no real user data, one
  product (story and sandbox are modes of one game), the lead's own game ("be able to sell it on
  steam").
- Copyleft studied, never copied (R7) — my reading of "sell it on steam": a closed-source
  commercial game. M0-TP puts the licence question to the lead.
- Box names: `linux-pc` (this PC) and `win-laptop` (the Windows laptop). Time zone America/Toronto
  (probed). Plans, reports and handoffs in English.

## 5. Ratings — PLAYBOOK §0; the bootstrap rated every block it wrote, M0-TB re-rates at the seed
| Block | Rung | Rule |
|---|---|---|
| M0-TI, M0-TP, M0-TC, M0-TG, M0-TB, M0-TZ | Opus 5.5, max | (2) gates — PLAN blocks |
| M0-TW | Opus 5.5, max | (2) gates — the CHECK scribe of the user gate |
| M0-R1 | Opus 5.5, high | (5) by (3)'s list: a design the spec leaves open — the engine and the language |
| M0-R2 | Opus 5.5, high | (5) by (3)'s list: numeric or precision semantics — physics models, units, step limits |
| M0-R3 | Opus 5.5, high | (5) by (3)'s list: an algorithm with no reference — one world from seconds to billions of years |
| M0-TH | Sonnet 5.5, high | (3) the cheapest rung that fits — the highest below (usual); no reason from the list |

Every rung is in the switch's `RUNGS` table (annex §A.10), so no rung needs adding there.

## 6. Refused to the bootstrap agent by Claude Code's auto mode (2026-10-08)
- Extracting the playbook's tools into a scratch folder and running them — to lint this plan and
  read the switch's install — refused as "Code from External". Not essential: M0-TH extracts the
  tools in the project. The plan was checked by hand against the plan tool's rules (annex §A.1).
- Writing AGENTS.md, CLAUDE.md and GEMINI.md from §C — refused as "Instruction Poisoning". They are
  the lead's to generate (milestones.md worklist), and so is the switch's check hook in
  `.claude/settings.json`, which is the same kind of auto-loaded instruction.
