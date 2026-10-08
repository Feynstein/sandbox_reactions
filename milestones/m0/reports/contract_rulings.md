# Contract rulings — M0-TC (2026-10-08)

The record of M0-TC's questions to the lead, by the question tool, and of the forks M0-TC ruled
itself as declared, reversible defaults (PLAYBOOK §4 Rule 6: a fork below the ask-bar is defaulted
and declared, never asked). Answers are verbatim; the option the lead picked was on screen with its
description, which is part of what was approved (as in reports/sandbox_interview.md §2). Every
ruling lands in milestones/m0/m0_contrat.md, which cites it by number (Q1–Q5, D1–D12).

Readers: M0-TH, M0-TG and M0-TB read the contract, not this file; this file is the lead's words
behind it, and the place a later PLAN task looks before amending a ruled clause.

## 1. Call 1 — the physics forks (2026-10-08 11:23 EDT, on linux-pc)
Four questions, one call. Each restated its consequence in the question or in the option shown.

### Q1 · Gravity in a flat world (B32 — sim_models.md M2)
- Asked: "How should gravity work in the sandbox's flat world? A real star is a ball but the world
  is a flat sheet of cells, so a trade-off is needed. Everything later builds on this; changing it
  afterwards means rewriting the physics."
- Options shown:
  - "Real 3D pull, thin sheet (Recommended)" — "Gravity weakens with distance exactly as in real
    space, acting inside a razor-thin sheet. A shrinking star heats up until fusion lights, and
    white dwarfs and neutron stars have a maximum mass, so all three endings can come out of the
    physics. Some sizes differ from real stars (e.g. a white dwarf's size barely depends on its
    mass)."
  - "Flat-universe gravity" — "The gravity of a truly 2D universe (it weakens more slowly with
    distance). Simpler maths, but a shrinking star does not heat up and nothing has a maximum
    mass: no white-dwarf limit, supernova or black hole could come out of the physics without
    hand-made rules."
  - "Slice through a round star" — "The screen is a cut through a 3D world that is round around a
    vertical axis. True round stars, but every blob painted off that axis is really a ring around
    it, and two clouds are two rings."
- **The lead's answer (verbatim): "Real 3D pull, thin sheet (Recommended)"**
- Ruled: sim_models.md option B — the 3D 1/r² law between cells of a razor-thin sheet; pressure,
  heat and light act in the plane. Contract §1.4, §2.3.

### Q2 · The top speed, if over budget (time_warp.md R8 F1)
- Asked: "If the first physics build shows the top speed (a Sun-like life in ~10 s) can't keep 60
  pictures a second on the mid-range graphics card, what should give? Today's measurement: it fits
  only if the star's slow and fast clocks turn out close together; the first build that runs a
  whole main sequence measures that."
- Options shown:
  - "30 pictures/s at top speed (Recommended)" — "Only the top speed drops to 30 pictures a second,
    doubling the time for physics; the ~10 s life stays and every other speed keeps 60. If even
    that is not enough, we come back to you."
  - "Shortcut jumps when calm" — "Keeps 60 pictures a second and the ~10 s life by leaping ahead
    while the star is calm. Costs a second way of advancing the physics with its own tests (a
    bigger M0), and a small risk of leaping past an event."
  - "A slower top speed" — "Keeps 60 pictures a second everywhere; a Sun-like life takes about
    25-50 s at top speed instead of ~10 s. This reopens your 'life in ~10 s' answer."
- **The lead's answer (verbatim): "30 pictures/s at top speed (Recommended)"**
- Ruled: the top rung may run at 30 frames a second when, and only when, the measured step cost
  needs it; every other rung keeps 60. If 30 frames cannot hold it either, the build stops and the
  lead is asked again (the option's own words: "we come back to you") — projective jumps and a
  lower top speed stay unruled, never a silent fallback. Contract §1.7, §5 G-FPS and G-TOP.

### Q3 · Stand-in laws for the two unresolved moments (sim_models.md M6, flags (b) and B5's risk)
- Asked: "Two moments of a dying star happen at scales too small for the sandbox's cells: the wind
  that strips a Sun-like star down to a glowing shell around a white dwarf, and the neutrino blast
  that makes a heavy star's supernova explode. May the sandbox use stand-in laws modelled on real
  stars for these?"
- Options shown:
  - "Yes, as a backup (Recommended)" — "The sandbox's own physics tries first (the star's light
    pushing its outer layers; part of the collapsing core's energy heating the gas around it).
    Where that can't do it, a law modelled on real stars takes over, acting only when the star's
    state calls for it, never on a timer, so the endings show reliably."
  - "No, sandbox physics only" — "Purest. But if the cells can't do it, a Sun-like star may keep
    its swollen envelope (no glowing shell) and a heavy star may collapse without exploding; we
    would come back to you."
- **The lead's answer (verbatim): "Yes, as a backup (Recommended)"**
- Ruled: first the sandbox's own mechanisms — the radiation force κF/c on the envelope, and a
  neutrino-heating share (≤ 0.1) of the newborn neutron star's neutrino losses deposited around it;
  then, only if a measured run shows the first mechanism failing at its bound, a stand-in — S1, a
  dust opacity in cool giant envelopes (the mechanism of real AGB winds), for the white dwarf's
  shell; S2, the heating share raised up to all of those losses (a thermal bomb), for the
  explosion — each acting through the physics state, never by time or label. Why S1 is not
  sim_models.md's Reimers-form wind: D12 below. Contract §1.6.4, §5 G-END.

### Q4 · Browsers without WebGPU (engine_stack.md flag (b))
- Asked: "The web build runs the same GPU physics through WebGPU, which most current browsers have,
  but not Firefox on Linux (yet) or older browsers. What should those browsers get?"
- Options shown:
  - "A 'needs WebGPU' page (Recommended)" — "A short message naming the browsers that work. No
    second physics engine to build; those players wait until their browser adds WebGPU."
  - "A slower CPU version" — "A second, CPU-only physics engine for those browsers: far slower (a
    star's life at top speed is unlikely) and built and tested alongside the GPU one, a large
    addition to M0."
- **The lead's answer (verbatim): "A 'needs WebGPU' page (Recommended)"**
- Ruled: no CPU fallback in the game; the web build shows the exact message of contract §4.9 when
  `navigator.gpu` or an adapter is missing. (The CPU f64 reference of §5 is a test oracle, never
  shipped.) Contract §4.9, §6.2.

## 2. Call 2 — the dependency gate (PLAYBOOK §2.2: the seed lot's table, cleared in one question)
The table engine_stack.md started (`## Dependency table`), versions fixed and every licence checked
on its registry page on 2026-10-08 (the four UNVERIFIED there — Rust, wasm-bindgen, wasm-pack,
trunk — now sourced). Versions are the newest stable on that date. Cargo.toml asks for the
approved minor (`wgpu = "30.0"`, `eframe = "0.36"`) and Cargo.lock is committed: a patch release
within it needs no ask; a new minor or major is a new ask (§4 Rule 2). Every crate these pull in
transitively is held to the same rule by contract §5 G-LIC (no copyleft anywhere in the tree).

| Name | Version | Role in M0 | Licence (source, accessed 2026-10-08) | Cost | Exit path |
|---|---|---|---|---|---|
| Rust toolchain — rustc, cargo, std, clippy, rustfmt | 1.99.0 stable, installed 2026-10-06 | language, build, lint | "dual-licensed: Apache License, Version 2.0 · MIT license" — https://www.rust-lang.org/policies/licenses | $0, installed | — (ruled at M0-R1) |
| Rust target `wasm32-unknown-unknown` | as the toolchain | the web build | as the toolchain | $0 — `rustup target add`, user-level (R3) | — |
| wgpu | 30.0.1 (MSRV 1.87) | GPU compute + drawing, native (Vulkan) and web (WebGPU) | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/wgpu | $0 | WGSL and the WebGPU model carry to Dawn (C++) or Bevy (engine_stack.md) |
| eframe (pulls egui, egui-wgpu, egui-winit, epaint) | 0.36.2 (MSRV 1.95) | window, input, panels; the web canvas | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/eframe ; egui-wgpu 0.36.2 needs wgpu ^30.0 and winit ^0.30.13 — https://crates.io/api/v1/crates/egui-wgpu/0.36.2/dependencies | $0 | egui-wgpu + egui-winit directly, or any immediate-mode UI |
| winit (through eframe) | 0.30.13 | window and input on the desktop | Apache-2.0 — https://crates.io/api/v1/crates/winit | $0 | SDL3 |
| bytemuck | 1.25.2 | cell and uniform structs to GPU buffers | Zlib OR Apache-2.0 OR MIT — https://crates.io/api/v1/crates/bytemuck | $0 | hand-written casts |
| pollster | 1.0.1 | waits on GPU start-up in headless runs | Apache-2.0/MIT — https://crates.io/api/v1/crates/pollster | $0 | any ten-line block_on |
| serde + serde_json | 1.0.229 + 1.0.151 | scenes, settings, run summaries (JSON) | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/serde · https://crates.io/api/v1/crates/serde_json | $0 | hand-written JSON |
| png | 0.18.1 | writing headless frames | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/png | $0 | PPM by hand |
| log + env_logger | 0.4.34 + 0.11.11 | logging (desktop) | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/log · https://crates.io/api/v1/crates/env_logger | $0 | eprintln |
| wasm-bindgen + wasm-bindgen-futures + web-sys (+ js-sys) | 0.2.129 + 0.4.79 + 0.3.106 | web glue (eframe needs wasm-bindgen ^0.2.126, web-sys ^0.3.103) | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/wasm-bindgen · https://crates.io/api/v1/crates/wasm-bindgen-futures · https://crates.io/api/v1/crates/web-sys | $0 | — |
| console_error_panic_hook | 0.1.7 | web: a panic reaches the browser console | Apache-2.0/MIT — https://crates.io/api/v1/crates/console_error_panic_hook | $0 | a ten-line hook |
| trunk | 0.21.14 | builds and bundles the web version | MIT/Apache-2.0 — https://crates.io/api/v1/crates/trunk | $0 — `cargo install`, user-level (R3) | wasm-pack 0.15.0, MIT OR Apache-2.0 — https://crates.io/api/v1/crates/wasm-pack |
| wasm-bindgen-cli (trunk fetches it) | 0.2.129 — must equal the wasm-bindgen crate | generates the web glue | MIT OR Apache-2.0 — https://crates.io/api/v1/crates/wasm-bindgen-cli | $0 | — |
| wasm-opt, from binaryen (trunk fetches it) | trunk's pinned release | shrinks the web build | Apache-2.0 — https://github.com/WebAssembly/binaryen | $0 | skip it: a larger file |
| playwright-core (npm) | 1.64.0 (needs Node >= 20; Node 20.20.2 installed) | drives the installed Chrome for the web smoke — the Playwright `tools/pb/capture_web.mjs` resolves and never installs (PLAYBOOK §A.5) | Apache-2.0 — https://registry.npmjs.org/playwright-core/latest | $0 — `npm install` in the repo, git-ignored node_modules | the Chrome DevTools Protocol by hand |
| Google Chrome (installed) | 155.0.8059.39 | the web smoke's browser on linux-pc | proprietary, a test tool only, never shipped | $0, installed | Safari 26, Firefox on Windows (engine_stack.md E3) |
| Python 3.12 stdlib, Node 20 | installed | the playbook's harness (`tools/pb/`) | PSF, MIT | $0, installed | — |
| Not in M0 | — | steamworks-rs 0.13 (Steam — a later milestone); wasm-bindgen-rayon (web threads — needs nightly Rust, engine_stack.md E14) | — | — | — |

### Q5 · The dependency gate (2026-10-08 11:28 EDT, on linux-pc)
- Asked: "Approve M0's libraries and tools (the dependency gate)? All are free and permissively
  licensed (MIT / Apache / Zlib), no copyleft (your rule R7). Once approved, agents fetch and install
  exactly these, in your home folder, without asking each time (rule R3); anything not on the list
  still asks. Full table with sources: milestones/m0/reports/contract_rulings.md section 2."
- Options shown: "Approve the whole list (Recommended)" — "Agents fetch these libraries and install
  the three tools (the web target, trunk, Playwright) as each task needs them. Patch updates of these
  versions need no ask; a newer version is a new question." (with the table above as its preview) ·
  "Libraries yes, ask per tool" — "The game libraries are approved; the three tools (web target,
  trunk, Playwright) are asked one by one when first needed: a few more pauses for you." · "Hold:
  I'll read it first" — "Nothing is fetched or installed until you approve; the harness task (M0-TH)
  waits on this answer."
- **The lead's answer (verbatim): "Approve the whole list (Recommended)"**
- Ruled: the table above is M0's approved dependency table (R2's "approved libraries", R3's
  "approved table"); contract §1.2, FROZEN.

## 3. Declared defaults — ruled by M0-TC, reversible (PLAYBOOK §4 Rule 6: below the ask-bar)
Each was routed to M0-TC by a research report or the interview; none is expensive to reverse, so
none was asked. The lead may overturn any of them — one line to the next PLAN task does it.
| # | Default | Why | Reversible by | Contract |
|---|---|---|---|---|
| D1 | A new world's edge lets matter leave for good; the player may switch to bouncing walls at any time, even while a star lives; light (heat) leaves through the edge in both modes | "It leaves for good (Recommended)" was the option shown with answer 18; "any time" is answer 4's; a walled box that kept its light could never cool a white dwarf (sim_models.md M7) | a settings default and one toggle | §2.8 |
| D2 | Three presets, one per ending — Sun-like (1 Sun), Massive (15 Suns), Giant (40 Suns) — their masses placed by the sandbox's measured thresholds; no free mass field (painting reaches any mass) | answer 2's shown text: "drop a ready-made cloud of a chosen mass to reach a given ending in one click" | a mass field is one small UI block | §2.10, §4.5 |
| D3 | A cloud too light to ignite shows "Failed star: too light to ignite"; no preset sits there; the lightest real stars, which outlive the universe, simply play out (their age reads long) | star_physics.md flag (a): "a label, and whether a preset may sit there, are yours" | a label string | §1.9.3, §2.10 |
| D4 | One black-hole threshold, not the real non-monotonic boundary | already ruled: the goal paragraph's "as its mass decides", approved at M0-TP; the non-monotonic boundary needs physics M0 does not model (star_physics.md E6) | a later milestone's physics | §1.9.5 |
| D5 | Readout translation: mass by four anchors (0.08, 1.44, 8 and 25 Suns at the sandbox's measured thresholds), temperature by anchors (10 K at the floor … 3.3 × 10⁹ K at silicon burning), age by a stage clock | a deviation from star_physics.md's one-factor mass proposal: the sheet's virial T ∝ M/R makes a 40-fold heavier star some 20 times wider, so the sandbox must squeeze the mass ladder to fit a giant on screen — one factor would then read a black hole at a few Suns; anchors keep the readout and its predicted ending in line with real astronomy (painting stays monotonic, not exactly additive) | display functions only | §1.10 |
| D6 | The cell inspector: temperature in kelvins (the readouts' translation), the element mix in %, density and speed in sandbox units, speed also as a Mach number | sizes are squeezed differently at every stage, so a real-unit density or speed would be a guess | display only | §4.6 |
| D7 | The readout's surface temperature is the effective temperature from luminosity and perimeter, L = 2πRσT⁴ | sim_models.md M4's proposal | display only | §1.10.3 |
| D8 | Time: ×1 = one sandbox time unit per second (≈ one dynamical time of the settled Sun-like star); rungs ×0.1 · ×0.3 · ×1 · ×3 · ×10 · ×30 · ×100 (· ×300 · ×1000) below a measured top rung that gives a Sun-like life in ~10 s; no auto-return after an automatic slow-down (a banner says why time slowed); the age readout a monotonic stage clock | time_warp.md F2 and R7's recommendations | constants in one module | §1.8, §1.10.4 |
| D9 | The world's size is a setting (default 600 × 400 cells, any multiple of 8 from 64 to 2048), not built in; M0's UI never changes it | sandbox_interview.md I9: the later per-graphics-card choice (B21) then needs no rewrite | — | §2.8 |
| D10 | Neutron matter is a tenth, non-paintable species, not a state flag | a deviation from star_physics.md and sim_models.md's "a state": a species keeps mass conserved per reaction record and advects like the rest | a registry row | §1.5 |
| D11 | The GPU path's oracle is a CPU f64 twin of every scheme (tolerances in §5); native devices use WebGPU's default limits so one set of shaders runs everywhere | engine_stack.md flag (c): GPU floats differ between cards (E20) | — | §5 G-REF, §6.2.2 |
| D12 | How Q2 and Q3 are carried out: the 30-frames switch engages per machine, by measurement, at the top rung only; the core-collapse slow-down is latched on the GPU within one step, ignition and supernova come from the per-object tracker (both span far more steps than its two-frame delay); S1 is a dust opacity rather than sim_models.md's Reimers-form wind; S2 is the neutrino-heating share raised to a thermal bomb; each stand-in's enabling failure is defined now | a Reimers law needs each star's L, R and M — a CPU-to-GPU feedback that would break the bit-identical warp guarantee (G-WARP); a dust opacity is local, needs no star identity, and is how real AGB winds work (Höfner & Olofsson 2018) | a PLAN amendment | §1.6.4, §1.8 |

## 4. Where every carried flag landed
| Flag (from) | Disposition |
|---|---|
| B29 default edge mode; change while a star lives (M0-TI) | D1 |
| B30 how a preset's mass is chosen (M0-TI) | D2 |
| B31 the real-equivalent translation; the inspector's units (M0-TI) | D5, D6 |
| B32 gravity's law in a flat world (M0-TI, M0-R2b) | Q1 |
| I9 the world size kept a setting (M0-TI) | D9 |
| RT16 the 60-frames promise has no check (M0-TP) | G-FPS on the Quadro RTX 4000, a named scene, every rung (contract §5.3) |
| RT17 the ending readout is a prediction (M0-TP) | G-END: presets and threshold ±25 % clouds end as predicted, thresholds measured in the sandbox's own physics (§5.2) |
| (a) the Quadro confirmed RTX 2070 class (M0-R1) | G-FPS names it |
| (b) browsers without WebGPU (M0-R1) | Q4 |
| (c) GPU floats differ; the CPU f64 reference (M0-R1) | D11, G-REF |
| (d) Repo facts' "Stack: not chosen" (M0-R1) | the plan's Stack line updated with the source layout (contract §1.1) |
| (e) the dependency gate; four licences UNVERIFIED (M0-R1) | Q5; all four sourced (§2 above) |
| (a) failed stars; the lightest stars (M0-R2a) | D3 |
| (b) one black-hole threshold (M0-R2a) | D4 (ratified by the goal) |
| (c) the translation (M0-R2a) | D5 — changed from one factor to anchors, with the reason |
| B32's fork (M0-R2b) | Q1 |
| models M1–M8, oracles O1–O16, tolerances (M0-R2b) | contract §1–§2 and §5 (every proposed tolerance adopted as proposed, fixed before code) |
| light leaves in bounce mode; T_eff (M0-R2b) | D1, D7 |
| shell ejection and the explosion, UNVERIFIED (M0-R2b) | Q3 (the stand-ins), §5.5 |
| F1 the top speed not shown held (M0-R3) | Q2 |
| F2 auto-return, ×1, the rung ladder, the render reserve (M0-R3) | D8; the reserve measured by the drawing lot (`--measure-ui`) |
| the age readout as a stage clock (M0-R3) | D8 |
| T1–T3 (M0-R3) | G-WARP, G-BOX, G-CAD |

## 5. Sources M0-TC added (each also cited where the contract uses it)
- crates.io registry API pages for every crate in §2's table, and npm's registry for
  playwright-core — URLs in the table, accessed 2026-10-08.
- https://www.rust-lang.org/policies/licenses — Rust's dual MIT / Apache-2.0 licence (accessed
  2026-10-08).
- https://github.com/WebAssembly/binaryen — Apache-2.0 (accessed 2026-10-08).
- https://developer.chrome.com/blog/supercharge-web-ai-testing (2024-01-16) — the flags for WebGPU in
  headless Chrome on Linux with NVIDIA (accessed 2026-10-08).
- https://research.chalmers.se/en/publication/500120 — Höfner & Olofsson 2018, dust-driven AGB winds
  (accessed 2026-10-08).
- https://arxiv.org/abs/2209.10989 — Imasheva, Janka & Weiss (MNRAS; arXiv 2022): "Thermal bombs
  are a widely used method to artificially trigger explosions of core-collapse supernovae"
  (abstract, accessed 2026-10-08).
- https://en.wikipedia.org/wiki/Interstellar_medium — molecular clouds at 10–20 K (accessed
  2026-10-08).
- https://jcgt.org/published/0002/02/01/ — Wyman, Sloan & Shirley 2013, the CIE fit (Eq. 4, Table 1
  read from the paper, accessed 2026-10-08); https://en.wikipedia.org/wiki/SRGB and
  https://en.wikipedia.org/wiki/Planck%27s_law (accessed 2026-10-08).
- https://arxiv.org/abs/astro-ph/0008432 — Janka 2000, read for the neutrino-heating share; it fixes
  no share, so f_dep's bound stays opinion.
- Probes on linux-pc (2026-10-08, read-only; logs/M0-TC.log): git (Phase 1 pushed as a5065fb); the
  Vulkan drivers (lavapipe present); /usr/bin/google-chrome; Rust components (clippy, rustfmt).
