# M0-TC — the M0 contract: detail behind the handoff (2026-10-08, linux-pc)

Started 11:14 EDT, closed 12:06 EDT. Type PLAN: wrote the contract, the rulings report, this file
and the plan's own lines (Status, Handoff, Flow row, Pipeline state, Repo facts' Stack /
Toolchains / Folders lines that the work outdated, two carried flags). No code, no suite.

## What exists now
- **milestones/m0/m0_contrat.md** — §0 conventions (FROZEN rulings, amendment and freeze rules,
  measured numbers kept out of the contract); §1 layout, approved dependencies, modules, the step's
  pass order P0–P9, the active box, determinism, gravity (Q1), the element and reaction registries,
  stand-ins (Q3), the solver registry, time control (Q2), observers (objects, stages, events,
  predicted ending), readout translations and the age clock, rendering, edits; §2 units, cell state,
  EOS, heat, reactions and neutrinos, sinks, constants with initial values and bounds, world and
  edges, the ledger, presets, calibration (schema and procedure), file formats; §3 the engine API,
  time-control API, the binary's commands (headless, bench, calibrate, capture, measure-ui), ready
  signals, the web entry, launch manifest entries; §4 the ```strings table (every UI string, keyed),
  layout, controls, readouts, errors, web pages, keys, design tokens, number formats; §5 38
  guarantees — oracle and source, tolerance fixed now, satisfiability, scope, plant — and the
  UNVERIFIED register; §6 environments; §7 docs/agent/ pages.
- **milestones/m0/reports/contract_rulings.md** — the five questions with every option as shown and
  the lead's answers verbatim (§1–§2), the dependency table with a source per licence (§2), the
  declared defaults D1–D12 with reasons (§3), where every carried flag landed (§4), the sources
  M0-TC added (§5).

## The lead's rulings (question tool, 2 calls)
| # | Asked at | Answer (verbatim) | Contract |
|---|---|---|---|
| Q1 gravity in a flat world | 11:23 | "Real 3D pull, thin sheet (Recommended)" | §1.4.1 FROZEN |
| Q2 the top speed if over budget | 11:23 | "30 pictures/s at top speed (Recommended)" | §1.8.6 FROZEN |
| Q3 stand-in laws for the shell and the explosion | 11:23 | "Yes, as a backup (Recommended)" | §1.6.4 FROZEN |
| Q4 browsers without WebGPU | 11:23 | "A 'needs WebGPU' page (Recommended)" | §6.2.4 FROZEN |
| Q5 the dependency gate | 11:28 | "Approve the whole list (Recommended)" | §1.2 FROZEN |

## Deviations from the research reports (each declared, with its reason)
- **D5 — the mass readout uses four anchors, not one factor** (star_physics.md proposed one factor).
  Under the sheet's virial T ∝ M/R a star 40 times heavier, burning at a similar core temperature, is
  some 20 times wider (my arithmetic); a 15-cell Sun-like star and a giant cannot share a 600 × 400
  world unless the sandbox squeezes the mass ladder — and with a squeezed ladder one factor would
  read a black hole at a few Suns. Anchors at the measured thresholds keep the mass readout and the
  predicted ending in line with real astronomy; painting stays monotonic, not exactly additive.
- **Temperature ratios squeezed** (§1.6.3): the real 15-Sun ignition ratios (Si/H ≈ 94) would need a
  core to shrink 94-fold between hydrogen and silicon burning — below one cell. The order K6 is kept;
  T_Si/T_H ≤ 10 is the bound.
- **D10 — neutron matter is a species**, not a state flag (both research reports said "a state"): a
  species keeps mass conserved per reaction record and advects conservatively.
- **D12 — S1 is a dust opacity, not sim_models.md's Reimers-form wind:** a Reimers law needs each
  star's L, R and M, a CPU-to-GPU feedback timed by readbacks — it would make the state depend on
  frame grouping and break G-WARP. A dust opacity is local, needs no star identity, and is how real
  AGB winds work (Höfner & Olofsson 2018).
- **D12 — the GPU latch covers the core collapse only** (time_warp.md R4 put all three detectors on
  the GPU): ignition and the supernova come from the per-object tracker, whose ≤ 2-frame delay is far
  shorter than either event; the collapse (25–50 steps, under one top-speed frame) keeps the latch.

## Findings measured or found and not acted on (why)
- WebGPU in headless Chrome on linux-pc and lavapipe presenting on a private Xvfb are UNVERIFIED —
  no build exists to try them; G-WEB and G-DESK carry them (§5.5).
- The 3α and ¹²C(α,γ) energies behind two q-ratios are textbook values from memory, marked
  UNVERIFIED in §1.6.2; the q-ratio of C_alpha only shapes a minor channel.
- 16 guarantees' satisfiability rests on runs not yet made (§5.5) — by design: tolerances are fixed
  before code; a red row becomes a D or a question to the lead, never a looser number (§0.4).
- The Pipeline state listed the Phase 1 commit gate as outstanding; git shows it committed and
  pushed (a5065fb = origin/main) — the line was updated.

## Done-when self-check (read-through, not a run — NOT PROVEN (source only))
"Done when a builder can implement every M0 block from the contract alone (§9)." Each kind of block
M0-TG will write finds its spec here: a physics pass — §1.3.2 order, its law and scheme (§1.4–§1.7,
§2.3–§2.6), constants and bounds (§2.5.1, §2.7), its CPU twin and guarantees (§5.1); time control —
§1.8, §3.2, G-WARP/G-LATCH/G-FPS/G-TOP; observers — §1.9, §1.10, §5.4; the UI — §4 with every
string keyed; the web build — §3.5, §4.9, §6.2; tooling — §3.3, §3.4, §3.6. Left to measurement on
purpose: calibration.json's values (§2.11) and the render reserve. Left to builders on purpose:
exact Rust types, GPU buffer packing (within WebGPU's limits), the FFT's internals (radix-2,
within G-GRAV1).

## Process notes
- `tools/pb/rung_record.py`: NOT RUN (no rung_record.py yet) — so the model and level read the same.
- `tools/pb/plan.py` absent: the plan was edited with the file-edit tool, per Repo facts — except
  one edit: M0-TG's carried flag was appended by a five-line Python script, which the instruction
  file calls a defect ("an ad-hoc script over it is a defect"). The diff shows one appended
  segment and nothing else; every later plan edit used the file-edit tool.
- Commands, all read-only, on linux-pc (log: milestones/m0/logs/M0-TC.log): git status / log; the
  Vulkan driver list, Chrome's path and the Rust components. No suite ran (no code exists).
