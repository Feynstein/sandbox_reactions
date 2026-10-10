# M0 Contract — Sandbox Reactions: the engine and a star's life   [sections marked FROZEN are immutable]

Authored by M0-TC on 2026-10-08, on linux-pc. The single source for every M0 build block: the
layout, registries, formulas, constants, interfaces, exact English strings, design tokens and the
guarantees that check them (PLAYBOOK §9, §14.4). The lead's rulings Q1–Q5 and M0-TC's declared
defaults D1–D12 are recorded verbatim in reports/contract_rulings.md. Research is cited by report:
SP-E<n> (star_physics.md), SM-E<n> / SM-O<n> / SM-C<n> (sim_models.md evidence, oracles, local
checks), TW-E<n> / TW-M<n> / TW-R<n> (time_warp.md evidence, measurements, recommendations), ES-E<n>
(engine_stack.md); K1–K9 are star_physics.md's squeeze invariants; B<n> and I<n> are
sandbox_interview.md's behaviours and inferences. A claim this contract adds carries its own URL and
access date; "my arithmetic" and "opinion" keep the research reports' meaning.

## §0 Conventions & precedence
- **0.1 Scope.** The first contract of the project: purely additive, nothing to override. Inside M0
  a FROZEN clause prevails over §5, and §5 over the rest; where a research report disagrees with
  this contract, this contract wins and the report is history.
- **0.2 FROZEN at authoring** — the lead's rulings, each beside the lead's words: Q1 gravity (§1.4.1),
  Q2 the top rung's frame rate (§1.8.6), Q3 stand-in laws (§1.6.4), Q4 browsers without WebGPU
  (§6.2.4), Q5 the dependency gate (§1.2). They change only by a new question to the lead.
- **0.3 Amendments** happen in place, by a PLAN task, tagged `[M0-<id>]`. A §5 clause counts as
  frozen once its scope has a GO `verify.py --redarm` record; the next PLAN task marks it
  `FROZEN [<id>]` (PLAYBOOK §9: a clause that names its enforcement is frozen only once proven red).
- **0.4 Measured numbers never live here.** Everything the physics must measure — the ending
  thresholds, the sandbox side of each readout anchor, the top rung, stage durations, the render
  reserve — lives in `assets/calibration.json` (§2.11), written by `sandbox-reactions calibrate`.
  This contract fixes how each is measured and the bounds it must meet, stated now, before any
  code: no tolerance here may be chosen after seeing a result. A tolerance a measured run proves
  unsatisfiable is re-set only by a PLAN amendment that asks the lead ("Changes by asking" covers
  formulas, never a bypass that makes a failing check pass) — never loosened inside a build block.
- **0.5 Declared overrides** (PLAYBOOK §9 — passage · what replaces it · why):
  - [M0-T5] §1.2's "Features: eframe `wgpu`" · eframe `wgpu_no_default_features` (with `wayland`, `x11`,
    `default_fonts`) · in eframe 0.36.2 `wgpu` also turns on egui-wgpu's defaults, i.e. wgpu's `webgl` and `gles`
    (measured, `cargo tree -e features -i wgpu`), against §1.2's own "no `webgl`" (Q4); the same renderer, no package
    added — the lead's answer at M0-T5, 2026-10-09: «wgpu_no_default_features (Recommended)».
  - [M0-T14] §5.4 G-LIC's list ("a missing licence or any other … is NO-GO") · the same, plus one named exception: package 
    `epaint_default_fonts` with exactly the expression `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`; any other
    package or a changed expression stays NO-GO and an ask · eframe's approved `default_fonts` (§1.2) pulls that crate, which
    bundles font files (data, not GPL-family code), measured by `cargo metadata` on 2026-10-10 (164 packages native, 143
    wasm32; no other refusal) — the lead's answer at M0-T14, 2026-10-10: «Named exception (Recommended)».
- **0.6 Arithmetic corrections:** none.
- **0.7 Words.** *cell* — one square of the 600 × 400 world (§2.8); *step* — one pass sequence P0–P9
  (§1.3.2); *rung* — a speed setting (§1.8); *object* — a connected body of gas (§1.9.2); *star* —
  the object a readout shows; *sandbox units* — §2.1; *real-equivalent* — a readout's translation
  to real astronomy (§1.10); *the box* — the active box (§1.3.3); *plant* — a red-arm patch (§5.0);
  *preset* — a ready-made cloud (§2.10); *t.u.* — one sandbox time unit.

## §1 Architecture & registries

### 1.1 Source layout — Rust workspace (the stack ruled at M0-R1: Rust + wgpu + winit + egui)
```
Cargo.toml             [workspace] members = ["crates/*"]; [workspace.dependencies] = the §1.2 table
Cargo.lock             committed
rust-toolchain.toml    channel "1.99.0"; components clippy, rustfmt;
                       targets x86_64-unknown-linux-gnu, wasm32-unknown-unknown
assets/                data embedded at compile time (include_str!), loaded and validated at start
  elements.json        §1.5 element registry
  reactions.json       §1.6 reaction registry
  physics.json         §2.1–§2.7 tunable constants
  calibration.json     §2.11 measured constants (written by `sandbox-reactions calibrate`)
crates/sr-physics/     no GPU: units, registries (load + validate), EOS, rate laws, time control (§1.8),
                       `reference` — the CPU f64 twin of every pass (§5's oracle), `observe` — the
                       tracker, stage classifier, readout translation and age clock (§1.9, §1.10)
crates/sr-engine/      wgpu: GPU state and passes (shaders in crates/sr-engine/shaders/*.wgsl), the box,
                       the latch, block summaries, rendering (views, glow), headless / bench / calibrate
crates/sr-app/         the game: eframe UI, tools, readouts, inspector, the status endpoint, the string
                       table (src/strings.rs, §4.1); bin `sandbox-reactions`; the wasm entry
web/                   index.html (the trunk entry), Trunk.toml, smoke.mjs (§6.2.3), package.json
                       (playwright-core); web/dist/ and node_modules/ git-ignored
scenes/                the test scenes of §5 (schema §2.12)
tests/plants/          one unified-diff patch per plant id (§5.0)
docs/agent/            §7
```
Dependencies point one way: sr-app → sr-engine → sr-physics. The step code (`sr_engine::step` and
every shader) never imports `sr_physics::observe`: labels watch, they never drive (I2; G-WATCH).

### 1.2 Approved dependencies — the dependency gate (Q5) — FROZEN
The lead, 2026-10-08 11:28 EDT: **"Approve the whole list (Recommended)"**; the full table with
licences, sources, costs and exit paths is reports/contract_rulings.md §2. Rust 1.99.0 (+ target
wasm32-unknown-unknown, clippy, rustfmt) · wgpu 30.0.1 · eframe 0.36.2 (egui, egui-wgpu,
egui-winit; winit 0.30.13) · bytemuck 1.25.2 · pollster 1.0.1 · serde 1.0.229 + serde_json 1.0.151 ·
png 0.18.1 · log 0.4.34 + env_logger 0.11.11 · wasm-bindgen 0.2.129 + wasm-bindgen-futures 0.4.79 +
web-sys 0.3.106 (+ js-sys) · console_error_panic_hook 0.1.7 · tools: trunk 0.21.14 (fetches
wasm-bindgen-cli 0.2.129 and wasm-opt) · playwright-core 1.64.0 (npm). Cargo.toml asks for the
approved minor (`wgpu = "30.0"`, `eframe = "0.36"`); a patch release within it needs no ask; any
other version or package is a new ask (PLAYBOOK §4 Rule 2). Transitive crates are held by G-LIC.
Features: eframe `wgpu`, `wayland`, `x11`, `default_fonts` (no `glow`, no `persistence`); wgpu's
native backends and `webgpu` only — no `webgl`, so a browser without WebGPU reaches the §4.9 page,
never a half-working WebGL2 path (Q4).

### 1.3 Modules and one simulation step
**1.3.1 Modules.** sr-engine: `state` (§2.2) · `grav` (§1.4) · `hydro` (§2.3) · `heat` (§2.4) ·
`react` (§1.6, §2.5) · `sinks` (§2.6) · `edges` (§2.8) · `boxfit` (§1.3.3) · `latch` (§1.8.5) ·
`summary` (§1.9.1) · `render` (§1.11) · `edit` (§1.12) · `ledger` (§2.9) · `headless`, `bench`,
`calibrate` (§3). sr-physics: `units`, `registry`, `eos`, `rates`, `time` (§1.8), `reference`,
`observe`.

**1.3.2 One step, in this order.** Each pass is a fixed dispatch sequence; no pass reads the wall
clock, the rung, the frame or a label.
| Pass | What it does |
|---|---|
| P0 edits | queued player edits (§1.12) applied at the step boundary; the box re-fit if an edit lands outside it; ledger entries |
| P1 Δt | Δt_n is the reduction P9 made at the end of step n−1 (step 0 reduces the initial state first) |
| P2 gravity | φ = K ∗ Σ_src over the box (§1.4.2), re-solved on the cadence of §1.4.4; sinks' potentials added (§1.4.3); g = −∇φ |
| P3 sinks | formation (at most one per step) and accretion, then the sinks' kick-drift (§2.6) |
| P4 hydro | MUSCL-Hancock + HLL (§2.3) with the gravity source, the radiation force from the previous step's flux (§2.4.4), the edge ghosts (§2.8); escaped counters |
| P5 heat | flux-limited diffusion stepped by RKL2 with s stages (§2.4); radiated energy counted; the flux field F kept for step n+1's P4 |
| P6 reactions | every enabled registry record and neutrino cooling, sub-cycled per cell (§2.5); the neutrino source field S_ν kept; the latch flag (§1.8.5) |
| P7 neutrino heating | the S_ν share deposited through a convolution kernel (§2.5.4); zero dispatches when ΣS_ν = 0 |
| P8 floors | vacuum reset, temperature floor, species renormalised; ledger floor terms (§2.7) |
| P9 reductions | Δt_{n+1} (§2.3.3); the non-finite guard every 64 steps (§3.3, exit 5); the per-cell frame accumulators (§1.9.1) fed by P5–P7 |
After P9, at step indices ≡ 0 (mod 16): the box re-fit (§1.3.3) and the gravity cadence decision
(§1.4.4), both from the state alone.

**1.3.3 The active box (TW-R1 A2).** The bounding box of non-vacuum cells (Σ ≥ Σ_vac), grown by a
margin of 8 cells, rounded out to multiples of 8, clamped to the world. Every pass runs inside it
(plus its ghost cells); outside, cells are vacuum and static. The margin holds: 16 steps × the CFL
bound of 0.4 cells per step (§2.3.3) = 6.4 < 8, so matter cannot outrun the box between re-fits (my
arithmetic). Gravity's FFT size per axis: the smallest power of two ≥ 2 × the box size, at least 32;
the kernel's transform is cached per size.

**1.3.4 Determinism.** No floating-point atomics in a pass that writes the state or a value the
state depends on; reductions that feed the state (Δt, sink accretion, the box) use a fixed-order
tree; flags use u32 atomicOr / atomicMax (order-independent). Same adapter, driver, binary, scene and
step count ⇒ a bit-identical state (G-WARP). Block summaries and the ledger feed only observers.

### 1.4 Gravity
**1.4.1 The law (Q1) — FROZEN.** The lead, 2026-10-08 11:23 EDT: **"Real 3D pull, thin sheet
(Recommended)"**. The world is a razor-thin sheet in 3D space (SM M2 option B): every cell pulls
every other with the 3D law — potential −Gm/r, r measured in the plane — while gas pressure, heat
and light act in the plane only. Contraction heats (K5: T ∝ M/R in the sheet) and a cold core has a
ceiling mass (K9: the in-plane ultra-relativistic Fermi gas, Π ∝ Σ^{3/2}, is exactly critical) —
SM M2-B, my arithmetic there, checked by G-CORE. Free space in both edge modes: no image masses
(SM-E8). G = 1 (§2.1).

**1.4.2 Kernel.** φ(x) = Σ_y K(x − y) Σ_src(y), with Σ_src = Σ in non-vacuum cells and 0 in vacuum.
K(i, j) = −G / √(i² + j²) for (i, j) ≠ (0, 0); K(0, 0) = −G · 4 ln(1 + √2) = −3.525494 G — the
exact potential at a unit square's centre from its own uniform mass, ∫∫_{[−½,½]²} dA / r =
4 ln(1 + √2) (my arithmetic), not Plummer softening (SM-C1: softening ε = 0.5 doubled the force
error). Computed by a zero-padded FFT convolution over the box (Hockney–Eastwood, SM-E8), radix-2
sizes (§1.3.3). g = −∇φ by central differences. P4 adds the momentum source Σg and the energy source
Σu·g, time-centred on the predictor's half-step velocity.

**1.4.3 Sinks** add −G M_s / √(r² + r_acc²) to φ (r from the sink's position, §2.6).

**1.4.4 Cadence (TW-R1 A3) — a lever, built only if M0-TG's budget needs it.** k ∈ {1, 4}: k = 4
only when, over the last 16 steps, no non-vacuum cell's Σ changed by more than 0.1 % and the latch
saw no neutronization or sink formation in the last 1.0 t.u.; decided at step ≡ 0 (mod 16) from the
state; otherwise k = 1. Guarded by G-CAD.

### 1.5 Element registry — `assets/elements.json`
Schema: `{"version": 1, "species": [{"key", "name", "symbol", "a", "z", "paintable", "color"}]}`;
the list order is the species id (0-based) and the element menu's order. M0's ten species — the
nine of SP §Elements (nickel left out, optional there; nitrogen left out: a temperature law stands
in for the CNO catalyst, SP) and neutron matter:

| id | key | name (§4.4) | A | Z | paintable | colour (§4.11) |
|---|---|---|---|---|---|---|
| 0 | H | Hydrogen | 1 | 1 | yes | #7FB2FF |
| 1 | He | Helium | 4 | 2 | yes | #FFE38A |
| 2 | C | Carbon | 12 | 6 | yes | #9E9E9E |
| 3 | O | Oxygen | 16 | 8 | yes | #6BE3C9 |
| 4 | Ne | Neon | 20 | 10 | yes | #FF6FAE |
| 5 | Mg | Magnesium | 24 | 12 | yes | #B58CFF |
| 6 | Si | Silicon | 28 | 14 | yes | #D8B27A |
| 7 | S | Sulfur | 32 | 16 | yes | #F2E14C |
| 8 | Fe | Iron | 56 | 26 | yes | #D9572B |
| 9 | n | Neutron matter | 1 | 0 | no | #FFFFFF |

Neutron matter is a species, not a state flag (D10): the neutronization record converts matter
into it, so mass is conserved per record and it advects like any species. Derived per cell, gas
fully ionized at every temperature (a declared simplification): 1/μ = Σ_i X_i (Z_i + 1)/A_i;
electrons per nucleon Y_e = Σ_i X_i Z_i/A_i; metals Z_met = 1 − X_H − X_He − X_n. A later element
is a new row, not new code (B22).

### 1.6 Reaction registry — `assets/reactions.json`: the one mechanism (B19)
**1.6.1 A record.** `{"id", "group", "inputs": [[key, share], …], "outputs": [[key, share], …],
"q_ratio", "rate": {"a", "orders": […], "terms": [[t_key, t_factor, nu, w], …], "t_thr_factor",
"gate": null | {"sigma_key", "p"}, "a_coef"}, "enabled", "standin"}`. Input shares sum to 1 and
output shares sum to 1 (mass conserved per record; checked at load). The specific conversion rate:

  ω = A · Σ^a · Π_i X_i^{n_i} · f(T) · g(Σ)
  f(T) = Σ_k w_k (T / T_k)^{ν_k} for T ≥ T_thr, else 0   (T_k = t_factor · the physics.json temperature named by t_key)
  g(Σ) = 1, or for a gated record ((Y_e Σ)/Σ_g − 1)^p when Y_e Σ > Σ_g, else 0

Each input i loses share_i · ω and each output gains share_o · ω (mass fraction per unit time); the
cell's thermal energy gains q · ω per unit mass, q = q_ratio · Q_H (q < 0 absorbs). The GPU may
tabulate f on 256 log-spaced temperatures if the interpolation error stays ≤ 10⁻⁴ relative (G-REF
holds it). Chemistry later adds records, never code (B23).

**1.6.2 M0's records** (q ratios fixed to nature's — K8, not tuned; A, T_k and ν are tuning within
§2.5's bounds):

| id | group | inputs → outputs | a; orders | f(T) terms (T_k, ν, w) | q_ratio | source |
|---|---|---|---|---|---|---|
| H_burn | H | H → He | 1; [2] | (T_H, 4, 1) + (1.15 T_H, 18, 1) | 1 | SP-E4: pp ν ≈ 4, CNO ν 13–23; SP-E10: pp and CNO equal at 18 MK against the Sun's 15.7 MK centre, ratio 1.15 (my arithmetic) |
| He_burn | He | He → C | 2; [3] | (T_He, 40, 1) | 1/10.8 | SP-E4, SP-E17 (ν ≈ 40, ∝ ρ²); SP-E1: "smaller by a factor 10 or more"; the ratio is my arithmetic from 3α's 7.275 MeV per 12 u (from memory — UNVERIFIED) against H's 6.29 × 10¹⁸ erg/g (SP-E1's φ = 0.007 × c²) |
| C_alpha | He | C 0.75 + He 0.25 → O | 1; [1, 1] | (T_He, 40, 1) | 1/14.6 | SM M5 (C + He → O); the ratio is my arithmetic from ¹²C(α,γ)¹⁶O's 7.16 MeV per 16 u (from memory — UNVERIFIED) |
| C_burn | C | C → Ne 0.8 + Mg 0.2 | 1; [2] | (T_C, 30, 1) | 1/15.7 | SP-E3 (4.0 × 10¹⁷ erg/g), K8 |
| Ne_burn | Ne | Ne → O 0.4 + Mg 0.6 | 1; [1] | (T_Ne, 30, 1) | 1/57 | SP-E3 (1.1 × 10¹⁷ erg/g) |
| O_burn | O | O → Si 0.7 + S 0.3 | 1; [2] | (T_O, 30, 1) | 1/12.6 | SP-E3 (5.0 × 10¹⁷ erg/g) |
| Mg_burn | O | Mg → Si | 1; [1] | (T_O, 30, 1) | 1/12.6 | SP-E3, Table 12.1's "O, Mg" fuel |
| Si_burn | Si | Si → Fe | 1; [1] | (T_Si, 30, 1) | 1/33 | SP-E3 (1.9 × 10¹⁷ erg/g), SP-E21 |
| S_burn | Si | S → Fe | 1; [1] | (T_Si, 30, 1) | 1/33 | SP-E3, the "Si, S" fuel |
| N_Fe | N | Fe → n | 0; [1] | f ≡ 1; gate Σ_g = Σ_N, p = 1 | −1.14 | SM-E24: an iron core's photodisintegration absorbs ~2 × 10⁵² erg ≈ 7.2 × 10¹⁸ erg/g for a ~1.4-Sun core, against H's 6.3 × 10¹⁸ (my arithmetic) |

Split-product shares (Ne 0.8 / Mg 0.2, O 0.4 / Mg 0.6, Si 0.7 / S 0.3) are opinion: they keep the
ladder's products (SP-E3) with mass conserved, and are not tuned. ν = 30 for C, Ne, O and Si is a
tuning default (K7: steep, values are tuning; ν ≥ 4 is the bound). T_thr_factor = 0.5 for every
burning record (no burning below half the reference temperature). A non-iron core heavier than the
cold ceiling heats by contraction (K5) until it burns onward to iron, so N_Fe is the only
neutronization record M0 needs; a measured run that shows a non-iron core collapsing without burning
is a D block, not a silent new record.

**1.6.3 Ignition order.** T_H < T_He < T_C < T_Ne ≤ T_O < T_Si (K6; Ne and O may tie). The real
15-Sun ratios (Si/H ≈ 94, SP-E3) cannot be kept: under the sheet's T ∝ M/R a core would have to
shrink 94-fold in radius between hydrogen and silicon burning, below one cell for any star that fits
the screen (my arithmetic) — so the sandbox squeezes the ratios (§2.5.1) and keeps the order
(G-ORDER).

**1.6.4 Stand-in laws (Q3) — FROZEN.** The lead, 2026-10-08 11:23 EDT: **"Yes, as a backup
(Recommended)"**. The sandbox's own mechanism runs first; each stand-in ships disabled — S1 as
`kappa_dust = 0`, S2 as f_dep's bound of 0.1 (§2.7) — and is enabled only by a PLAN amendment that
cites a measured failure of the primary mechanism at its bound (a later stand-in that is a registry
record carries `"standin": true` and ships `"enabled": false`):
- **S1 · dust-driven wind (the white-dwarf ending, B4).** Primary: the radiation force κΣF/c_sb on
  every cell (§2.4.4), gas opacity only. Stand-in: a dust opacity κ_dust · Z_met added to κ where
  T < T_dust — dust grains forming in cool giant envelopes "scattering and absorbing stellar photons
  and transferring their outward-directed momentum to the surrounding gas through collisions"
  (Höfner & Olofsson 2018, Astronomy and Astrophysics Review —
  https://research.chalmers.se/en/publication/500120, accessed 2026-10-08). Failure that enables it:
  the Sun-like preset, 20 τ_dyn after entering the helium-shell stage, has unbound less than 30 %
  of its envelope (the mass outside 2 R_core).
- **S2 · thermal bomb (the supernova ending, B5).** Primary: neutrino heating with share
  f_dep ≤ 0.1 (§2.5.4). Stand-in: f_dep raised up to 1.0 — all of the newborn neutron star's
  neutrino losses returned to the gas around it, a thermal bomb: "Thermal bombs are a widely used
  method to artificially trigger explosions of core-collapse supernovae" (Imasheva, Janka & Weiss,
  MNRAS, arXiv:2209.10989, 2022 — https://arxiv.org/abs/2209.10989, accessed 2026-10-08).
  Failure that enables it: the Massive preset forms neutron matter and, within 20 t.u., unbinds less
  than 50 % of its envelope at f_dep = 0.1.
- Both act only through the state (temperature, metals, neutron matter), never on a timer or a
  stage label (G-WATCH). Energy stays conserved: deposition never exceeds what the core emitted.

### 1.7 Solver registry
| Solver | Pass | Scheme | Stability limit | Oracles (§5) |
|---|---|---|---|---|
| gravity | P2 | FFT convolution (§1.4) | — (a source term) | G-GRAV1, G-GRAV2, G-COLL |
| hydro | P4 | MUSCL-Hancock, HLL fluxes, unsplit 2D (SM-E13) | CFL 0.4 and the gravity limit (§2.3.3) | G-SOD, G-VIR, G-JEANS |
| heat | P5 | flux-limited diffusion + RKL2 (SM-E18, SM-E19) | s from Δt (§2.4.3) | G-HEAT1, G-HEAT2, G-FLD |
| reactions | P6 | per-cell linearised implicit sub-cycles (SM M5) | ≤ 10 % ΔX and ≤ 5 % ΔT per sub-cycle | G-BURN, G-ORDER, G-THERMO |
| neutrino heating | P7 | FFT convolution with an energy cap (§2.5.4) | — | G-CONS |
| sinks | P3 | Federrath-style criteria, bound-gas accretion (SM-E28) | — | G-SINK |
| edges | P4, P5 | ghost cells, two modes (§2.8) | — | G-EDGE |
Every solver has a CPU f64 twin of the same scheme in `sr_physics::reference` (G-REF), and a scene
switch (§2.12) that turns it off for unit scenes.

### 1.8 Time control — `sr_physics::time`, pure logic (TW-R2 to TW-R4)
**1.8.1 ×1** = one sandbox time unit per wall second — about one dynamical time √(R³/GM) of the
settled Sun-like star (≈ 1.0 for M = 3351, R = 15, SM's worked example; my arithmetic) (D8).

**1.8.2 Rungs.** The ladder ×0.1 · ×0.3 · ×1 · ×3 · ×10 · ×30 · ×100 · ×300 · ×1000, keeping the
rungs below TOP / 1.5, then TOP itself. TOP = `life_sun_sb` (calibration: sim time from dropping the
Sun-like preset to its white-dwarf stage) ÷ 10 s, rounded to two significant digits — a Sun-like
life in ~10 s at the top rung (B9). Keys 1…n select a rung in order; + faster, − slower (§4.10). The
game starts at ×1, running.

**1.8.3 A frame.** owed += rung / f_target (sandbox time owed this frame); N = min(N_max,
⌈owed / Δt_est⌉) steps, Δt_est the last Δt read back (≤ 2 frames late); owed −= the sim time
actually advanced (read back), owed ≥ −Δt_est. N_max = max(1, ⌊(1000 / f_target − R_reserve) / c_step⌋)
with c_step the 16-frame moving average of the GPU-timestamped cost per step and R_reserve the
calibration's `render_reserve_ms` (3.0 until the first drawing lot measures it — TW §3's
reservation, opinion there). When less than one step per frame is owed (slow motion), the renderer
draws the interpolation between the last two states with α = owed / Δt_est (TW-E4).

**1.8.4 The cap is never silent** (TW-R2): when N_max binds in ≥ 30 of the last 60 frames, the
speed readout reads "Speed ×{rung} (running ×{actual})" (§4.3), actual = the sim time advanced over
the last 60 frames ÷ their wall time.

**1.8.5 Pause, step, the slow-down and its latch.** Pause stops stepping. Step: while paused, exactly
one step; while running, it pauses. The automatic slow-down (B11), setting "Slow down for big
events", on by default: ignition (§1.9.4) → ×1; core collapse (the latch) → ×0.1; supernova
(§1.9.4) → ×0.1 — only ever slower, never faster; the rung stays where the slow-down put it until
the player changes it (no auto-return, D8), and the banner (§4.3) shows until then. The latch (a
u32 flag): set in P6 when any cell's N_Fe rate is > 0, and in P3 when a sink forms; armed when
neither happened in the last 1.0 t.u.; when an armed latch is set and the slow-down is on, a
controller dispatch zeroes every later indirect dispatch of that frame (TW-E16), so no further step
runs; the CPU then sets ×0.1 and the latch re-arms by the rule. With the setting off, steps run
through events at the chosen rung and the "running ×" readout shows any shortfall.

**1.8.6 The top rung at 30 frames a second (Q2) — FROZEN.** The lead, 2026-10-08 11:23 EDT:
**"30 pictures/s at top speed (Recommended)"**. f_target = 60 at every rung. At TOP only, f_target
becomes 30 when N_max at 60 binds in ≥ 30 of the last 60 frames, and returns to 60 after 120
consecutive frames that needed fewer than half of N_max at 60. The sim time per wall second does
not change with the switch. If TOP cannot hold at 30 either (G-TOP red on the Quadro), the build
stops and the lead is asked again ("If even that is not enough, we come back to you"); shortcut
jumps and a slower top speed stay unruled — neither is ever a silent fallback.

**1.8.7 Time never touches the physics** (TW-R2): Δt is the CFL step computed from the state;
rungs, the latch, the frame rate and the 30-frames switch change only how many steps a frame runs
(G-WARP).

### 1.9 Observers — `sr_physics::observe`: they watch, never drive (I2)
**1.9.1 Block summary** (GPU, after each frame's last step, over the box): blocks of 4 × 4 cells;
per block: mass; Σx·m, Σy·m; momentum (x, y); kinetic, thermal and cold energy; the pressure
integral ∫Π dA; the ten species masses; Σm·φ; the max-Σ cell's index, Σ, T, X (ten) and
Π_cold / Π; and the per-cell frame
accumulators summed — radiated energy, nuclear energy by group (H, He, C, Ne, O, Si, N), neutrino
energy lost and deposited; unbound mass (cells with ½|u|² + ε_th + φ > 0); the sinks inside. Read
back asynchronously (≤ 2 frames late); the accumulators reset after each summary.

**1.9.2 Objects.** 8-connected blocks that hold a cell with Σ ≥ Σ_obj (§2.7); an object lighter than
1 % of the Sun-like preset is a wisp, not listed. Identity: an object inherits the id of the
previous frame's object with which it shares the most block mass (≥ 25 %), else it gets a new id;
on a merge the heavier id survives. Per object: M (gas + sinks); centre c (the max-Σ cell, or the
heaviest sink inside); R (the radius about c holding 90 % of M, from block masses); τ_dyn =
√(R³/GM); L (radiated ÷ the frame's sim time); L_g per group (L_N, neutronization, is an absorbed
power ≤ 0); L_nuc = the sum of the positive L_g; L_ν; Σ_c, T_c, X_c at c; degeneracy
d_c = Π_cold / Π at c; v_r (the mass-weighted radial velocity about c); U (thermal), K (kinetic),
P = ∫Π dA, W = ½Σmφ; M_unb; the
envelope mass (outside 2 R_core, R_core the radius where the group of the dominant burning stage
produces 90 % of its power); sink mass.

**1.9.3 Stages** — evaluated per object each summary; the first matching row wins; a stage other
than core collapse and supernova is displayed after 3 consecutive identical results. Labels (§4.6)
follow the stage; "Supergiant" replaces "Red giant" for objects with M ≥ M_up (calibration).

| Stage id | Matches when | Fuel of φ (§1.10.4) |
|---|---|---|
| black_hole | a sink lies inside the object | remnant |
| neutron_star | X_n,c ≥ 0.5, \|v_r\| ≤ 0.05 √(GM/R), no sink | remnant |
| supernova | the object passed core_collapse within the last 20 t.u. and M_unb ≥ 0.1 M | — |
| core_collapse | N_Fe is converting matter (L_N < 0) | — |
| white_dwarf | ignited, d_c ≥ 0.5, L_nuc < 0.01 L, M_unb < 0.05 M, X_n,c < 0.5 | remnant |
| failed_star | never ignited, d_c ≥ 0.5, L_H < 0.01 L | remnant |
| planetary_nebula | ignited, d_c ≥ 0.5, L_nuc < 0.1 L, M_unb ≥ 0.05 M | — |
| iron_core | X_Fe,c ≥ 0.5 and L_nuc < 0.1 L | — |
| si_burn · o_burn · ne_burn · c_burn | that group is the largest L_g and ≥ 0.1 L_nuc, its fuel at c ≥ 10⁻³ | Si · O · Ne · C |
| he_core | L_He ≥ 0.1 L and Y_c ≥ 10⁻³ | He |
| he_shell | ignited, Y_c < 10⁻³, X_H,c < 10⁻³, no carbon burning | — |
| h_shell | ignited, X_H,c < 10⁻³, L_He < 0.1 L | — |
| main_sequence | L_H ≥ 0.5 L and X_H,c ≥ 10⁻³ | H |
| collapsing | never ignited and v_r ≤ −0.1 √(GM/R) | — |
| protostar | never ignited, \|v_r\| ≤ 0.05 √(GM/R) and 2P/\|W\| ≥ 0.8 (pressure-supported, near virial) | — |
| cloud | otherwise (a cold cloud at rest has 2P/\|W\| ≈ 0.2, §2.10) | — |
Collapsing precedes protostar on purpose: a cloud in free fall passes 2(U + K)/|W| = 1 halfway
down, so only the pressure integral and a small v_r mark a protostar (my arithmetic).

**1.9.4 Events.** *Ignition*: the object's `ignited` flag first turns on, at L_H ≥ 0.5 L (after
MIST's ZAMS criterion, TW-E7, relaxed from 99.9 % for a coarse sandbox star — opinion). *Core
collapse*: the latch (§1.8.5). *Supernova*: the stage supernova is entered. *Sink formed*: logged
only (the collapse latch covers its slow-down). Each fires once per object.

**1.9.5 Predicted ending** (B2 — RT17): from M in Suns (§1.10.1): below 0.08 failed star; below 8
white dwarf; below 25 supernova then neutron star; otherwise black hole. Within ±10 % in sandbox
mass of a calibrated threshold the readout names both, "(close call)". Once a remnant stage
(white_dwarf, neutron_star, black_hole, failed_star) has held for 10 t.u., the readout shows the
ending instead of the prediction.

**1.9.6 The selected star** is the object last right-clicked; by default, the heaviest object.

### 1.10 Real-equivalent readouts (B10, B31; D5)
Display functions only — the physics never reads them (G-WATCH).

**1.10.1 Mass → Suns.** Piecewise log-linear through four anchors, the sandbox side measured by
calibration: M_ign ↔ 0.08 Suns (SP-E10, SP-E11) · M_Ch ↔ 1.44 (SP-E12) · M_up ↔ 8 (SP-E3's
M_up ≈ 8; SP-E7) · M_BH ↔ 25 (SP-E5's "≳ 25 M⊙"; the "about 25+ Suns" the lead saw at M0-TI); the
end segments extended beyond. Monotonic, not additive (D5): the sandbox squeezes the mass ladder so
the heaviest preset fits the screen — under the sheet's virial T ∝ M/R, a star 40 times heavier
burning at a similar core temperature would be some 20 times wider (my arithmetic), which no
600 × 400 world holds beside a 15-cell Sun. Preset masses are the inverse translation of 1, 15 and
40 Suns (§2.10).

**1.10.2 Temperature → kelvins.** Piecewise log-linear through anchors: T_floor ↔ 10 K (molecular
clouds, 10–20 K — https://en.wikipedia.org/wiki/Interstellar_medium Table 1, accessed 2026-10-08) ·
the Sun-like preset's T_eff at X_H,c = 0.35 (calibration `t_eff_sun_sb`) ↔ 5,772 K (SP-E9) · T_H ↔
1.571 × 10⁷ K (SP-E9, the Sun's centre) · T_He ↔ 1.8 × 10⁸ K · T_C ↔ 8.3 × 10⁸ K · T_Ne ↔
1.6 × 10⁹ K · T_O ↔ 1.9 × 10⁹ K · T_Si ↔ 3.3 × 10⁹ K (SP-E3 Table 12.1, 15 Suns); above T_Si the
O–Si segment extended; below T_floor, 10 K. Every displayed temperature uses it (readouts,
inspector, colours). The sandbox side must increase strictly (calibration fails otherwise).

**1.10.3 Surface temperature** (D7): T_eff from L = 2πR σ T_eff⁴ with σ = a_r c_sb / 4 (SM M4),
then translated.

**1.10.4 Age → years: a stage clock** (TW-R7; D8). Each summary: age += max(Δφ, 0) · τ_s(M_suns),
where φ is the current stage's progress: for a stage with a fuel (§1.9.3), φ = (X_f,c,start −
X_f,c) / (X_f,c,start − 10⁻³) (SP-E2's end of the main sequence at 10⁻³); for other stages,
φ = sandbox time in the stage ÷ that stage's calibrated sandbox duration at this mass
(`stage_durations_sb`, log-log in mass; 10 t.u. where unmeasured); remnant stages add
1 × 10⁸ years per t.u. (opinion). The age never decreases and never jumps (G-AGE): a mass change
alters the rate going forward, never the accumulated age (SSE's fractional age, TW-E5, TW-E6). The
rate "1 s ≈ …" is the age gained over the last wall second. τ_s(M) in years, log-log in mass
between the listed masses, held constant beyond them:

| Stage | 1 Sun | 5 Suns | 15 Suns | 25 Suns | Source |
|---|---|---|---|---|---|
| cloud, collapsing | 1 × 10⁶ | 1 × 10⁶ | 1 × 10⁶ | 1 × 10⁶ | SP-E2 ("millions of years") |
| protostar | 1 × 10⁷ | 1.8 × 10⁵ | 1.1 × 10⁴ | 3.2 × 10³ | SP-E2: 10⁷ (M/M☉)^−2.5 yr |
| main_sequence | 1 × 10¹⁰ | 1.8 × 10⁸ | 1.1 × 10⁷ | 1 × 10⁷ | SP-E10 (10¹⁰ (M/M☉)^−2.5); SP-E3 (15 Suns); SP-E7 (25 Suns) |
| h_shell | 2 × 10⁹ | 3.6 × 10⁷ | 2.3 × 10⁶ | 6.4 × 10⁵ | SP-E2, SP-E16 (2 Gyr at 1 Sun), scaled as (M/M☉)^−2.5 — opinion |
| he_core | 1.2 × 10⁸ | 2.2 × 10⁷ | 2.0 × 10⁶ | 1 × 10⁶ | SP-E2, SP-E3, SP-E7 |
| he_shell | 1.5 × 10⁶ | 1.5 × 10⁶ | 1.5 × 10⁶ | 1.5 × 10⁶ | SP-E2 (TP-AGB 1–2 × 10⁶ yr) |
| planetary_nebula | 1 × 10⁴ | 1 × 10⁴ | 1 × 10⁴ | 1 × 10⁴ | SP-E15 |
| c_burn | 2.0 × 10³ | 2.0 × 10³ | 2.0 × 10³ | 1 × 10³ | SP-E3, SP-E7 |
| ne_burn | 0.7 | 0.7 | 0.7 | 3 | SP-E3, SP-E7 |
| o_burn | 2.6 | 2.6 | 2.6 | 0.3 | SP-E3, SP-E7 |
| si_burn | 0.049 (18 d) | 0.049 | 0.049 | 0.0137 (5 d) | SP-E3, SP-E7 |
| iron_core | 0.0027 (1 d) | 0.0027 | 0.0027 | 0.0027 | opinion — no source |
| core_collapse | 9.5 × 10⁻¹¹ (3 ms) | same | same | same | SP-E3 ("a few milliseconds") |
| supernova | 3.2 × 10⁻⁷ (10 s) | same | same | same | SP-E3 (neutronization 3–10 s) |

### 1.11 Rendering — `sr_engine::render` (B12, B17)
- **The cell image.** A compute pass writes a W × H RGBA8 texture from the state (blended with the
  previous state by α in slow motion, §1.8.3) for the active view; drawn as a quad with nearest
  filtering at an integer scale s = max(1, ⌊min(available width / W, available height / H)⌋) —
  visible square cells (B12); s = 2 in the default window (§4.2).
- **Glow view** (the default): colour = blackbody(T_K) · b + haze · (1 − b), with T_K the translated
  temperature (§1.10.2), b = 1 − exp(−(T_K / 3000 K)⁴), haze = #2A3550 ·
  clamp((log₁₀ Σ + 8) / 8, 0, 1) · 0.6. blackbody(T): Planck's B_λ(λ, T) = (2hc²/λ⁵) / (e^{c₂/(λT)} − 1)
  (https://en.wikipedia.org/wiki/Planck%27s_law, accessed 2026-10-08), c₂ = hc/k_B =
  1.438777 × 10⁷ nm·K (my arithmetic from the exact SI h, c, k_B), integrated over 380–780 nm in
  5 nm steps against the CIE 1931 colour-matching functions in Wyman, Sloan & Shirley's multi-lobe
  fit (Journal of Computer Graphics Techniques 2(2), 2013, Eq. 4 and Table 1 —
  https://jcgt.org/published/0002/02/01/, accessed 2026-10-08), converted by the IEC 61966-2-1 XYZ →
  linear-sRGB matrix [3.2406 −1.5372 −0.4986; −0.9689 1.8758 0.0415; 0.0557 −0.2040 1.0570]
  (https://en.wikipedia.org/wiki/SRGB, accessed 2026-10-08), clamped ≥ 0, normalised so the largest
  channel is 1, then sRGB-encoded; a 512-entry table over log T from 1,000 K to 40,000 K, built once
  on the CPU; outside it, the end colours. Wyman's Table 1 (α, β, γ, δ per lobe): x̄ (0.362, 442.0,
  0.0624, 0.0374), (1.056, 599.8, 0.0264, 0.0323), (−0.065, 501.1, 0.0490, 0.0382); ȳ (0.821, 568.8,
  0.0213, 0.0247), (0.286, 530.9, 0.0613, 0.0322); z̄ (1.217, 437.0, 0.0845, 0.0278), (0.681, 459.0,
  0.0385, 0.0725); each lobe α · exp(−½[(λ − β) · (λ < β ? γ : δ)]²).
- **Glow** (the Glow view only): bright pass on luminance > 0.8 at half resolution, a separable
  9-tap Gaussian (σ = 3 px at half resolution), upsampled and added at intensity 0.6 (§4.11) — hot
  matter glows and a supernova flashes (B12).
- **Heat view**: the sequential map of §4.11 over log₁₀ T_K from 1 to 10. **Element view**: the
  mass-fraction-weighted blend of the species colours (§1.5) in linear RGB, times a visibility
  clamp((log₁₀ Σ + 6) / 8, 0.15, 1); vacuum black. **Density view**: the sequential map of §4.11 over
  log₁₀ Σ from −6 to 4 (sandbox units).

### 1.12 Edits — the player's tools (B7, B16), applied in P0 and booked in the ledger
- **Brush**: inside the brush disk (radius 1–40 cells, §4.4), each frame the button is held adds
  ΔΣ = `paint_sigma` (§2.7) of the selected element at T = `paint_t`, at rest (momentum unchanged);
  composition mixes by mass. **Eraser**: sets every cell in the disk to the floor state. **Heat** /
  **Cool**: multiplies ε_th by `heat_factor` / `cool_factor` per frame held (floored at T_floor).
  **Preset drop**: writes the preset's cloud (§2.10) centred on the click; refused with the §4.5
  message if any of its cells would fall outside the world. Painting, erasing, heating and cooling
  work on a shining star exactly as on gas (B7: "it reacts through the same physics").
- **Clear world**: after the §4.7 confirmation, every cell returns to the floor state, sinks are
  removed, the ledger restarts; settings (rung, view, edge mode, slow-down) are kept.

## §2 Data model

### 2.1 Units and scales
- **Sandbox units** (SM §Models): length — one cell (Δx = Δy = 1); G = 1; the mass unit is chosen so
  the Sun-like preset's cloud has central surface density Σc = 1 at radius a = 40 cells, i.e. a mass
  M = 2πΣc a²/3 ≈ 3,351 (my arithmetic) — the calibrated `preset_mass_sb.sun` sets the exact figure
  (§2.10); one time unit (t.u.) = √(cell³ / (G · mass unit)); temperature T in specific-energy units
  (cell² / t.u.², Boltzmann's constant over the atomic mass unit absorbed), so Π_th = ΣT/μ.
- Real values never enter the physics ("Tests check each law's exact answers, not the real Sun's
  numbers", answer 1); they appear only in §1.10's display translation.

### 2.2 Cell state — the canonical channels
| # | Name | Meaning |
|---|---|---|
| 0 | sigma | Σ, surface density (mass per cell area) |
| 1, 2 | mom_x, mom_y | Σu, Σv |
| 3 | energy | E = Σ(ε_th + ε_cold) + ½Σ\|u\|² per cell area |
| 4–13 | x_H, x_He, x_C, x_O, x_Ne, x_Mg, x_Si, x_S, x_Fe, x_n | mass fractions in §1.5's order, Σ_i X_i = 1 |
f32 on the GPU, f64 in the CPU twin. This order is canonical for dumps (§2.12.3) and for every
CPU/GPU comparison; the GPU may pack channels as it likes within the WebGPU default of 8 storage
buffers per shader stage (TW-E15), which binds native builds too (§6.2.2). Not state, per cell:
φ, the heat flux F, the neutrino source S_ν, the frame accumulators (§1.9.1). ~14 × 4 B per cell ≈
13.4 MB at 240,000 cells (SM M1's arithmetic).

### 2.3 Equation of state and gas flow
- **2.3.1 Ideal gas in the plane:** γ = 2 — a monatomic gas whose pressure acts in two dimensions
  (opinion; it sits above the sheet's critical γ = 3/2, SM M2-B, as 5/3 sits above 4/3 in 3D).
  Π_th = (γ − 1) Σ ε_th = Σ ε_th; T = μ ε_th.
- **2.3.2 Cold pressure** (degeneracy, SM M6): P(x; K₁, K₂) = 1 / (1/(K₁x²) + 1/(K₂x^{3/2})) — the
  2D Fermi gas's non-relativistic (x²) and ultra-relativistic (x^{3/2}) limits (SM-E4, SM-E5),
  blended (opinion, SM M6). Electrons: Π_e = P(Y_eΣ; K₁ₑ, K₂ₑ); neutron matter: Π_n = P(X_nΣ; K₁ₙ,
  K₂ₙ). Their energy per area, u(x) = x ∫₀^x P(s)/s² ds, is tabulated once on the CPU (512 points
  log-spaced over x ∈ [10⁻⁸, 10⁸]) and interpolated log-log on CPU and GPU alike; ε_cold =
  (u_e + u_n)/Σ. In the ultra-relativistic limit the ceiling is M_ch ≈ 1.77 K₂'²/G² for a uniform
  disk, K₂' = K₂Y_e^{3/2} (SM M6's arithmetic, × my Y_e scaling) — calibration measures the real
  value (§2.11).
- **2.3.3** Π = Π_th + Π_e + Π_n. Wave-speed bound for fluxes and Δt: c² = 2Π/Σ (every term's
  log-slope in Σ is ≤ 2). ε_th = E/Σ − ½|u|² − ε_cold, floored at T_floor/μ; the floor's energy is
  booked (§2.9).
- **2.3.4 Scheme** (SM M3): finite volume, MUSCL-Hancock with the monotonised-central slope limiter
  on primitive variables (Σ, u, v, Π, X_i); HLL fluxes with Davis wave speeds S_L = min(u_L − c_L,
  u_R − c_R), S_R = max(u_L + c_L, u_R + c_R); unsplit (both directions' fluxes from the half-step
  states); species advected with the mass flux; the gravity source of §1.4.2 and the radiation force
  of §2.4.4.
- **2.3.5 The step:** Δt = min(C · min_cells [ (|u|+c)/Δx + (|v|+c)/Δy ]⁻¹, η_g · min_cells
  √(Δx/|g|)), C = 0.4 (SM M3; SM-E12's C_max = 1 is the 1D limit), η_g = 0.3 (free fall resolved by
  about 20 steps — opinion, SM M2). Burning and heat never limit Δt (sub-cycles, RKL2 stages).
  Resolution: every collapsing preset keeps the Jeans number Δx/λ_J ≤ 1/4 (SM-E16), λ_J ≈ c²/(GΣ)
  in the sheet (UNVERIFIED, SM — from memory).

### 2.4 Heat and light
- **2.4.1 One-temperature transport** (SM M4): ∂(Σε_th)/∂t = −∇·F, F = F_rad + F_cond, with
  F_rad = −(c_sb λ(R)/(κΣ)) ∇(a_r T⁴), Levermore–Pomraning λ(R) = (2 + R)/(6 + 3R + R²),
  R = |∇(a_r T⁴)| / (κΣ a_r T⁴) (SM-E19). Thick gas: λ → 1/3, the radiative conductivity
  (4/3) a_r c_sb T³/(κΣ) (SM-E20). Thin gas: |F_rad| ≤ c_sb a_r T⁴. F_cond = −k_cond ∇T (a small
  conductivity; 0 allowed).
- **2.4.2 Opacity:** κ = κ₀(1 + X_H) — electron-scattering-like, constant per mass (opinion); plus
  κ_dust · Z_met where T < T_dust when S1 is enabled (§1.6.4; κ_dust = 0 until then).
- **2.4.3 Stepping** — RKL2 super-time-stepping (SM-E18): Δt_p = 0.5 / max_cells(χ/Δx² + χ/Δy²),
  χ the cell's diffusivity (conductivity over Σc_v, c_v = 1/μ); s = the smallest integer ≥ 2 with
  Δt_p (s² + s − 2)/4 ≥ Δt; coefficients frozen at the start of P5. If s > 32 in any step the run
  prints `SR-WARN heat stages <s>` (SM M4's ~30-stage fallback point); an implicit solver is a PLAN
  decision, never a silent switch.
- **2.4.4 Light leaving, and its push.** The flux into a vacuum cell, or through the world edge in
  either edge mode (D1), is radiated: booked `radiated`, and added to the emitting cell's frame
  accumulator (§1.9.1). The radiation force per area f = κΣF/c_sb (SM-E22) is P4's momentum source
  in the next step; E is unchanged by it (the work comes out of thermal energy).
- **2.4.5** No photon particles: luminosity is the heat flux leaving the gas (SM M4), and the glow is
  drawn from T (§1.11).

### 2.5 Reactions and neutrinos
- **2.5.1 Burning constants** (physics.json; initial values from SM's worked example — opinion;
  bounds fixed now):

| Constant | Initial | Bound | Role |
|---|---|---|---|
| T_H | 100 | > 0 | hydrogen's reference temperature: the settled Sun-like core runs near μc²/2 ≈ 114 for SM's c ≈ 19.5 (my arithmetic) |
| T_He, T_C, T_Ne, T_O, T_Si | 140, 196, 274, 300, 420 | T_H < T_He < T_C < T_Ne ≤ T_O < T_Si (K6); T_Si/T_H ≤ 10 (§1.6.3) | the ladder |
| Q_H | 8,700 | the Sun-like main sequence keeps τ_nuc ≥ 10 τ_KH (K1; G-SQUEEZE) | hydrogen's energy per unit mass; every q = q_ratio · Q_H (K8, not tuned) |
| A per record | H_burn 5 × 10⁻⁴, others 1 × 10⁻³ | > 0 | rate normalisation |
| ν of C_burn, Ne_burn, O_burn, Mg_burn, Si_burn, S_burn | 30 | ≥ 4 (K7) | steepness |
| Σ_N (the N_Fe gate, on Y_eΣ) | 160 | ≥ 2 (K₂ₑ/K₁ₑ)², the relativistic regime | iron neutronizes only once its electrons are relativistic |
| burn_dx, burn_dt | 0.1, 0.05 | — | a sub-cycle changes no X by more than 0.1 (absolute) and T by no more than 5 % |

- **2.5.2 Sub-cycling** (SM M5): per cell, after the flow step, linearised backward-Euler
  sub-steps under burn_dx and burn_dt; fractions clamped ≥ 0 and renormalised; q·ω into ε_th at each
  sub-step; the nuclear energy booked per group in the frame accumulator.
- **2.5.3 Neutrino cooling** (SM M4 flag (c), SM-E24): ε_ν = A_ν (T/T_ν)^m per unit mass for
  T ≥ T_ν, else 0; m = 8 (photo-neutrinos, SM-E20: "roughly ε_ν ∝ T⁸"); bound m < ν of every late
  record, so burning stays stable where ε_nuc = ε_ν and the late stages race (SM-E24); T_ν initial
  166, between T_He and T_C (the real onset, ~5 × 10⁸ K, sits below carbon burning — SM-E24).
  Booked `neutrino_lost`. S_ν = ε_νΣ in cells with X_n ≥ 0.5 (the newborn neutron star) feeds P7.
- **2.5.4 Neutrino heating** (P7): D(x) = Σ(x) · (S_ν ∗ K_ν)(x) with K_ν(r) = 1/(r² + 1) (r in
  cells), computed over the box by the FFT machinery of §1.4.2, and D = 0 where X_n ≥ 0.5; the
  deposited power is q(x) = f_dep · (Σ_y S_ν(y)) · D(x) / Σ_x D(x) — exactly f_dep of the emitted
  power, shared by mass and nearness; booked `neutrino_deposited`. f_dep: initial 0.05, bound
  0 ≤ f_dep ≤ 0.1 (opinion: no source fixes the share; the bound keeps the primary mechanism an
  order of magnitude inside S2's f_dep = 1). Energy is conserved by construction (deposited ≤
  emitted).

### 2.6 Sinks — black holes (SM M6, SM-E28)
- **State:** position and velocity (cells, cells per t.u.), mass M_s; up to 16 sinks; at most one
  forms per step.
- **Formation** (P3) — all four at one cell (SM-E28: "a sole density threshold … is insufficient"):
  Σ ≥ Σ_BH; ∇·u < 0; φ no greater than at any of its 8 neighbours; bound, ½|u|² + ε_th + φ < 0. The
  densest candidate forms (ties: the lowest y, then the lowest x); the sink takes the cell's mass
  above the floor and its momentum, and the cell's thermal and cold energy is booked `swallowed`.
- **Accretion** (P3, every step): r_acc = max(2 cells, 2GM_s/c_sb²) (SM-E29's form, SP-E20's
  r_s = 2GM/c² with the sandbox's light speed); a cell whose centre lies within r_acc and whose gas
  is bound to the sink (½|u − v_s|² + ε_th − GM_s/max(r, 0.5) < 0) gives the sink its mass above the
  floor and that mass's momentum; its energy is booked `swallowed`. One fixed-order reduction per
  sink.
- **Motion:** kick-drift-kick with the grid's g at the sink (bilinear) plus the other sinks' pull
  softened by r_acc; two sinks closer than the larger r_acc merge (mass and momentum summed).
- Σ_BH: initial 4,000 (opinion); bound ≥ 4 × calibration's `ns_sigma_max_sb` (no stable neutron
  star can trip it).

### 2.7 Floors, objects, tools and the remaining constants — physics.json
| Constant | Initial | Bound / role |
|---|---|---|
| sigma_floor | 1 × 10⁻⁸ | the vacuum state's Σ |
| sigma_vac | 1 × 10⁻⁶ | below it a cell is vacuum: P8 resets it to the floor state (Σ_floor, u = 0, T = T_floor), booking the difference `vacuum_reset` |
| t_floor | 0.05 | the lowest gas temperature (↔ 10 K, §1.10.2) |
| sigma_obj | 1 × 10⁻³ | a block joins an object when one of its cells is at least this dense (§1.9.2) |
| paint_sigma | 0.02 per frame held | the brush's deposit, ≈ 1.2 Σ per second |
| paint_t | 2.0 | painted gas temperature, cloud-like |
| heat_factor, cool_factor | 1.05, 0.95 per frame held | the heat and cool tools |
| K₁ₑ, K₂ₑ | 21.3, 134 | electrons: with Y_e = 0.5, M_ch ≈ 1.77 × 134² / 8 ≈ 3,970 ≈ 1.2 × the Sun-like mass, and a cold disk radius 2K₁Y_e²/(παG) ≈ 4.0 cells (SM M2-B's R = 2K₁/(παG), α = 8/3π; my arithmetic) |
| K₁ₙ, K₂ₙ | 2.67, 58.2 | neutron matter: M_tov ≈ 1.77 × 58.2² ≈ 6,000 ≈ 1.5 M_ch, radius ≈ 2.0 cells; bound m_tov/m_ch ∈ [1.4, 2.0] (K9's real 1.4–2: SP-E12, SP-E13) |
| a_r, c_sb, κ₀, k_cond | 4 × 10⁻⁷, 200, 1, 0 | heat and light; tuned for K1's gaps (G-SQUEEZE); c_sb > 2 × calibration's `max_gas_speed` |
| A_ν, T_ν, m | 1 × 10⁻³, 166, 8 | neutrino cooling (§2.5.3) |
| f_dep | 0.05 | ≤ 0.1 (§2.5.4); ≤ 1.0 under S2 |
| sigma_bh | 4,000 | §2.6 |
| cfl, eta_g | 0.4, 0.3 | §2.3.5 |
| kappa_dust, t_dust | 0, 0 | S1, disabled (§1.6.4) |
| block | 4 | summary block edge in cells (§1.9.1) |

### 2.8 World and edges (B20, I8, I9; D1, D9)
- **WorldConfig {width, height}** is a setting, never built in (I9; D9): default 600 × 400 cells —
  the size the lead was shown (B13); any width and height that are multiples of 8 within [64, 2048].
  M0's UI never changes it (choosing it by graphics card is a later milestone, B21); headless runs
  and tests may.
- **Edge mode {leave, bounce}.** A new world starts in *leave* — "It leaves for good (Recommended)",
  the option shown with answer 18 (D1); the player may switch at any time, even while a star lives
  (answer 4's "any time"; D1). Two ghost cells per side.
  - *leave:* ghosts copy the edge cell with the outward velocity component clamped outward; the
    boundary face's mass flux is clamped to outflow, so inflow is exactly 0 (SM-O16); whatever
    crosses — mass, momentum, energy, each species — is booked `escaped`.
  - *bounce:* ghosts mirror the edge cell with the normal velocity negated; the wall face's mass and
    energy fluxes are set to exactly 0, its momentum flux to the pressure term alone.
  - Gravity is free space in both (§1.4.1). Heat and light leave through the edge in both (D1;
    else a walled box fills with heat and nothing cools into a white dwarf — SM M7).

### 2.9 The ledger (SM-O10) — f64 on the CPU, from fixed-order block partials
- **Mass:** grid · sinks · escaped · painted · erased · preset_dropped · vacuum_reset · cleared.
- **Energy:** kinetic · thermal · cold · potential W = ½Σmφ (sinks included) · radiated ·
  neutrino_lost · neutrino_deposited · escaped · swallowed · nuclear_released (Σ q·ω·Δm, signed) ·
  floor_added · tools (painted, erased, heated, cooled, preset, cleared).
- **Momentum:** grid · sinks · escaped.
- Each booked term is cumulative since the ledger (re)started: escaped, erased and vacuum_reset
  count mass that left the grid by that route; painted and preset_dropped mass that entered;
  floor_added and tools are net energies added (signed).
- **Invariants (G-CONS):** mass — grid + sinks + escaped + erased + vacuum_reset − painted −
  preset_dropped is constant (escaped stays 0 in bounce mode). Energy — kinetic + thermal + cold + W
  + radiated + neutrino_lost − neutrino_deposited + escaped + swallowed − nuclear_released −
  floor_added − tools is constant, within G-CONS's tolerance. Clear world restarts the ledger.

### 2.10 Presets (B3, I3; D2, D3)
- **Three presets:** `sun` — 1 Sun, ends as a white dwarf · `massive` — 15 Suns, a supernova and a
  neutron star · `giant` — 40 Suns, a black hole. Each mass is the inverse translation (§1.10.1) of
  its Suns, stored as calibration `preset_mass_sb`. 15 Suns sits 0.55 of the way from 8 to 25 in log
  mass; 40 lies 0.41 of that band's width above 25 (my arithmetic) — each well inside its ending.
- **The cloud:** a Maclaurin disk Σ(r) = Σc√(1 − r²/a²) (SM-E6) with a = 40√(M / M_sun) cells
  (Σc = 1 at the Sun-like mass), capped at 150 cells (Σc then grows); the Sun's composition by mass —
  H 0.7346, He 0.2485, O 0.0077 (SP-E22) and the remaining 0.0092 as C (opinion: the traces lumped);
  a uniform temperature set so U_th = 0.1|W| (a cold cloud that collapses — opinion); at rest.
- **No preset below ignition** (D3): a failed star comes only from painting too little; the physics
  shows it and the readout names it (§1.9.3).

### 2.11 Calibration — `assets/calibration.json`, written by `sandbox-reactions calibrate` (§3.3)
- **Schema:** `{"version": 1, "physics_hash": "<sha256 of the canonical JSON of elements.json +
  reactions.json + physics.json>", "measured": {"date", "git", "adapter", "box"}, "m_ch_sb",
  "m_tov_sb", "ns_sigma_max_sb", "m_ign_sb", "m_up_sb", "m_bh_sb", "preset_mass_sb": {"sun",
  "massive", "giant"}, "t_eff_sun_sb", "life_sun_sb", "top_rung", "stage_durations_sb":
  {"<stage id>": [[mass_sb, t.u.], …]}, "render_reserve_ms", "max_gas_speed"}`.
- **Procedure** (headless runs on any adapter; the CPU twin may stand in for items 1–2):
  1. m_ch_sb — a cold disk (T_floor; C 0.5 + O 0.5, so Y_e = 0.5; a = 12 cells) run 10 τ_dyn: it
     holds when R₉₀ changes by < 3 %; bisection on M to 0.5 % (SM-O9).
  2. m_tov_sb, ns_sigma_max_sb — the same with X_n = 1; ns_sigma_max_sb = the largest Σ_c among
     holding runs.
  3. m_ign_sb — the preset cloud at mass M (solar mix, §2.10) until it ignites or fails (fails: d_c ≥
     0.5 and L_H < 0.01 L held for 20 τ_dyn); bisection to 2 %.
  4. m_up_sb (white dwarf against core collapse) and m_bh_sb (neutron star against black hole) —
     preset clouds run to their ending; bisection to 2 %.
  5. The order (K9): m_ign < m_ch < m_up < m_bh, m_ch < m_tov and m_tov/m_ch ∈ [1.4, 2.0] — else
     exit 6, and a K9 failure is reported, never re-tuned inside the run.
  6. Then preset_mass_sb (§1.10.1); t_eff_sun_sb at the Sun-like preset's X_H,c = 0.35; life_sun_sb
     (drop → white_dwarf) and top_rung (§1.8.2); stage_durations_sb from the three preset lives;
     max_gas_speed over every run (c_sb must exceed twice it — else exit 6).
  7. render_reserve_ms is written by the drawing lot's `--measure-ui` run (§3.3); calibrate keeps it.
- The engine refuses a calibration whose physics_hash differs from its embedded assets (G-CAL):
  change a constant, calibrate again. A run over 10 minutes is the lead's (§6.4).

### 2.12 File formats — tests and headless runs only (none is a save format)
- **2.12.1 Scene** (`scenes/*.json`):
```json
{"version": 1,
 "world": {"width": 600, "height": 400, "edge": "leave"},
 "slowdown": true,
 "switches": {"gravity": true, "hydro": true, "heat": true, "reactions": true, "neutrinos": true, "sinks": true},
 "test": {"pin_temperature": false, "constant_diffusivity": null, "flux_limiter": true},
 "objects": [
   {"kind": "preset", "preset": "sun", "x": 300, "y": 200},
   {"kind": "disk", "x": 100, "y": 100, "a": 20, "sigma_c": 0.5, "t": 2.0,
    "composition": {"H": 0.7346, "He": 0.2485, "O": 0.0077, "C": 0.0092}, "v": [0, 0]},
   {"kind": "rect", "x0": 0, "y0": 0, "w": 400, "h": 4, "sigma": 1.0, "pressure": 1.0,
    "composition": {"H": 1.0}, "v": [0, 0]}
 ],
 "overrides": {"<physics.json key>": 0.0}}
```
  Unknown keys are refused (exit 4). A `disk` or `rect` takes `t` or `pressure`, not both.
  `overrides` are for tests only; a run with overrides never writes calibration.
- **2.12.2 Run summary** (`summary.json`): `{"version": 1, "tool": "sandbox-reactions <semver>",
  "git", "adapter": {"name", "backend", "driver"}, "scene", "world", "steps", "sim_time", "wall_s",
  "ledger": {"start": {…}, "end": {…}}, "events": [{"step", "sim_time", "kind":
  "ignition|core_collapse|supernova|sink_formed|ending", "object", "detail"}], "objects": [{"id",
  "stage", "mass_sb", "mass_suns", "predicted_ending", "ending", "age_years", "stages": [{"stage",
  "from_step", "sim_time"}]}], "until": {"condition", "met"}, "timing": {"per_pass_ms": {…},
  "step_ms_mean"}}` (timing only with `--timing`).
- **2.12.3 State dump** (`dumps/s_<step>.bin`): the eight bytes `SRDUMP01`, a little-endian u32
  header length, a UTF-8 JSON header `{"version": 1, "width", "height", "step", "sim_time",
  "channels": [§2.2's names], "sinks": [{"x", "y", "vx", "vy", "m"}]}`, then each channel as a
  little-endian f32 plane in §2.2's order, row-major.
- **2.12.4 Coordinates:** x to the right, y down, cell (0, 0) at the top-left, in every file, API
  and readout.
- **2.12.5 Edits file** (`--edits`, for G-TOUCH and G-AGE): a list of `{"at": <trigger>, "edit":
  {"kind": "paint|erase|heat|cool", "x", "y", "radius", "species", "frames"}}`, applied in P0 at the
  first step its trigger holds — `{"step": n}`, or `{"object_until": {"x_h_c_below": 0.5}}` (the
  heaviest object's central hydrogen first falls below the value), or `{"mass_until":
  {"at_least": m}}` / `{"at_most": m}` (the edit repeats each step until that object's mass in
  sandbox units passes m); `frames` = how many frames of holding the tool it stands for (§1.12).
  Deterministic: triggers read the state at step boundaries, never the wall clock.

### 2.13 Saving
No save format in M0 — saving is a later milestone (B24, answer 16).

## §3 The engine's interfaces

### 3.1 The engine API (sr-engine) — names and meanings fixed; exact Rust types are the builder's
- `Engine::new(gpu, world, assets, calibration)` → an engine, or an error naming the cause (registry
  or calibration invalid, physics_hash mismatch, adapter limits below WebGPU's defaults).
- `load_scene(scene)` clears the world and writes a scene (§2.12.1); `load_dump(dump)` restores one.
- `queue_edit(edit)` — Paint {cell, radius, species} · Erase {cell, radius} · Heat {cell, radius} ·
  Cool {cell, radius} · DropPreset {preset, cell} · Clear; applied at the next P0 (§1.12).
- `set_edge_mode(mode)`, `set_switches(switches)` — applied at the next step boundary.
- `encode_frame(plan, encoder)` — the plan's steps as indirect dispatches under the latch (§1.8.5),
  then the block summary.
- `render(view, alpha, target, encoder)` — the cell image, and for Glow the glow passes (§1.11).
- `poll()` — never blocks; returns frame reports from the asynchronous readbacks: Δt, sim time,
  steps run, latch fired, cost per step (GPU timestamps), the block summary, the ledger partials,
  inspect replies.
- `inspect(cell)` — an asynchronous request; the reply (Σ, u, T, X) arrives in a later `poll()`.
- `step_blocking(n)`, `dump_state()` — tests and headless runs only.
- Observers (`sr_physics::observe`) take frame reports: `Tracker::update(report)` → objects with
  stage, readouts and events (§1.9, §1.10).

### 3.2 The time-control API (`sr_physics::time`, no GPU)
`TimeControl` holds the rungs (from calibration's top_rung), the selected rung, paused, the slow-down
setting, f_target and owed sim time; `select(k)`, `faster()`, `slower()`, `toggle_pause()`,
`step_once()`, `plan_frame(estimates) → {steps, f_target}`, `on_report(report)`,
`on_event(event)`, and `speed_text() → (rung label, running speed or none, paused)`. Unit-tested
without a GPU against §1.8.

### 3.3 The binary `sandbox-reactions`
- **Desktop** (no subcommand): `sandbox-reactions [--adapter <name substring>] [--world <W>x<H>]
  [--status-port <port>] [--capture <script.json> --out <dir>] [--measure-ui <frames>]
  [--offscreen-window]`.
  - `--offscreen-window` [M0-TJ3]: the window placed outside every monitor's area, never activated and
    kept out of the taskbar, the app otherwise unchanged — the route of the window, capture and UI-cost
    checks on a box with no private display (win-laptop, §6.6; R4 as amended). Never in the web build.
- **Headless:** `sandbox-reactions headless --scene <file | preset:sun | preset:massive |
  preset:giant> [--world WxH] [--edge leave|bounce] (--steps N | --until <condition>)
  [--max-steps N] [--out DIR] [--frames-every N] [--view glow|heat|element|density]
  [--dump-every N] [--load-dump FILE] [--edits FILE] [--adapter S] [--cpu-reference] [--timing]`
  - `--until`: `sim_time:<t>` · `event:<ignition|core_collapse|supernova|sink_formed>` ·
    `stage:<stage id>` · `ending` (any object holds a remnant stage for 10 t.u.).
  - `--cpu-reference` runs the CPU f64 twin instead of the GPU (worlds up to 128 × 128).
  - Writes DIR/summary.json (§2.12.2), DIR/events.jsonl (one event per line), DIR/frames/f_<step>.png
    (with --frames-every), DIR/dumps/s_<step>.bin (with --dump-every, and always at the end).
  - stdout: first `SR-ADAPTER name=<…> backend=<…> driver=<…>`; last, whenever the run ends without
    an error (exit 0 or 2), `SR-HEADLESS DONE steps=<n> sim_time=<t> until=<met|unmet|none>`; every
    error line begins
    `SR-ERROR`, every warning `SR-WARN`.
  - Exit codes: 0 done · 2 condition not met within --max-steps (default 2,000,000) · 3 no adapter
    or a GPU error · 4 bad arguments or scene · 5 a non-finite value in the state (the message names
    the step, cell and channel) · 6 calibration failed (§2.11) · 101 a panic.
- **Bench:** `sandbox-reactions bench --scene … --rung <label|top> (--frames N | --until <condition>)
  --adapter S [--slowdown on|off] [--load-dump FILE] --out DIR` — §1.8's frame loop with every frame
  rendered offscreen at the default window's world size (1200 × 800, Glow view with glow; the UI
  panels' cost is R_reserve), no vsync. Writes DIR/bench.json: `{"adapter", "rung", "frames",
  "f_target_switches", "frame_ms": {"p50", "p95", "max"}, "steps_per_frame": {"p50", "max"},
  "sim_time", "life_wall_s", "until": {…}}`, life_wall_s = Σ_frames max(frame_ms, 1000/f_target) /
  1000 — the wall time the run would take on screen.
- **Calibrate:** `sandbox-reactions calibrate --adapter S --out assets/calibration.json
  [--only <item>]` — §2.11; `--only` re-measures one item and keeps the rest.
- **Capture** (TV passes, PLAYBOOK §8 — "game engines feed their own captures"): `--capture
  <script.json> --out <dir>` runs a scripted desktop session, writes one PNG per step and
  captions.json in review_page.py's shape (PLAYBOOK §A.4), then exits 0. Script: a list of
  `{"id", "caption", "actions": [ … ], "frames": n}`, ids matching `[A-Za-z0-9][A-Za-z0-9.-]*`;
  actions: `{"preset": "sun", "at": [x, y]}` · `{"tool": "brush|eraser|heat|cool"}` ·
  `{"element": "<key>"}` · `{"paint": [[x, y], …]}` · `{"view": "glow|heat|element|density"}` ·
  `{"rung": k}` · `{"pause": true|false}` · `{"steps": n}` · `{"hover": [x, y]}` ·
  `{"select": [x, y]}` · `{"edge": "leave|bounce"}` · `{"slowdown": true|false}`; the screenshot is
  taken after `frames` frames.
- **Measure UI:** `--measure-ui <frames>` runs the desktop app with the Sun-like preset at ×1 and
  prints `SR-UI p95_ms=<x>` — the panels' and presentation's cost per frame — then writes
  render_reserve_ms = p95 + 0.5 into calibration.json. A visible window: asked of the lead each
  time (R4).

### 3.4 Ready signals — what `tools/pb/launch.py` polls (its three kinds, PLAYBOOK §A.3)
- **Headless boot:** kind `exit` — exits 0 within the deadline; `ready_forbid`: `["SR-ERROR",
  "panicked at"]`.
- **Desktop:** kind `url` — `--status-port <port>` serves `GET /status` on 127.0.0.1 only (plain
  HTTP/1.0 on std::net, no new dependency) with JSON `{"status": "booting" | "ready" | "error",
  "frame", "step", "sim_time", "adapter", "fps"}`; "ready" once the first frame is presented;
  ready_key `status`, ready_values `["ready"]`. Off unless the flag is given; never in the web
  build.
- **Web:** the static server (`python3 -m http.server 47812 --bind 127.0.0.1 --directory
  web/dist` — standard library) is kind `port`; the page signals through
  `document.body.dataset.srState` (§3.5).

### 3.5 The web entry
- trunk builds sr-app for wasm32 from web/index.html into web/dist/ (`trunk build --release`).
- web/index.html holds a full-window `<canvas id="sr-canvas">`, the loading line (§4.9) and an inline
  script that, before loading the wasm, checks `navigator.gpu` and awaits
  `navigator.gpu.requestAdapter({powerPreference: "high-performance"})`; when either is missing —
  or the URL carries `?no-webgpu=1`, the test hook — it shows the §4.9 page, sets
  `document.body.dataset.srState = "no-webgpu"` and never loads the wasm (Q4).
- The app sets `srState` to `"loading"`, then `"ready"` after its first presented frame, and
  `document.body.dataset.srAdapter` to the adapter's description; on a fatal error `"error"` with
  §4.8's message.
- URL parameters (smoke and captures): `preset=<sun|massive|giant>` drops it at the world's centre ·
  `steps=<n>` runs n steps then pauses · `view=<glow|heat|element|density>` · `rung=<k>`.

### 3.6 Launch manifest entries (`launch.json`, written by the build blocks that boot each service [M0-TG])
| Service | Command | Ready | Ask |
|---|---|---|---|
| `game` | `build/target/release/sandbox-reactions --status-port 47811` [M0-TG: cargo's target is build/target/, §6.5] | url http://127.0.0.1:47811/status | yes — a visible window (R4) |
| `game-xvfb` | the same plus `--adapter llvmpipe`, under launch.py's private Xvfb (`xvfb: true`) — Mesa's software Vulkan (lavapipe; `lvp_icd.json` present on linux-pc, probed 2026-10-08) | url, as above | no — a private display, never the lead's desktop (R4) |
| `game-offscreen` [M0-TJ3] | the same as `game` plus `--offscreen-window` — win-laptop's route for the window checks (§6.6) | url, as above | no — off-screen, never on the lead's screen (R4 as amended) |
| `headless-boot` | `build/target/release/sandbox-reactions headless --scene preset:sun --steps 200 --out <tmp>` [M0-TG] | exit; ready_forbid SR-ERROR, panicked at | no |
| `web` | the static server on 47812 | port | no |
The double-click launchers PLAYBOOK §8 requires (start.sh: `launch.py start game --task <id>`, wait
for Enter, `launch.py stop`) come with the first build block that runs a window. [M0-TJ3] start.bat sits
beside start.sh for win-laptop: `py -3.12 tools\pb\launch.py start game --task <id>`, wait for Enter,
`py -3.12 tools\pb\launch.py stop` (the lead's Q2, reports/win_laptop.md § Ruling).

## §4 UI — exact English strings and design tokens (B12, B16, B17; answer 7, answer 13)

### 4.1 The string table
Every user-visible string of the desktop and web app is one row of the block below. sr-app holds
them in crates/sr-app/src/strings.rs as `pub const STRINGS: &[(&str, &str)]` — exactly these keys
and texts — and web/index.html carries the `web.*` texts verbatim (G-STR reads this block).
`{name}` is a placeholder filled at run time (§4.12). Characters: × U+00D7, — U+2014, ≈ U+2248,
· U+00B7, … U+2026.
```strings
app.title = "Sandbox Reactions"
time.pause = "Pause"
time.play = "Play"
time.step = "Step"
time.rung = "×{rung}"
time.speed = "Speed ×{rung}"
time.speed_capped = "Speed ×{rung} (running ×{actual})"
time.paused = "Paused"
time.slowdown = "Slow down for big events"
banner.ignition = "Slowed down: hydrogen ignition — press + to speed up again"
banner.core_collapse = "Slowed down: core collapse — press + to speed up again"
banner.supernova = "Slowed down: supernova — press + to speed up again"
edge.label = "World edge:"
edge.leave = "Leave for good"
edge.bounce = "Bounce back"
world.clear = "Clear world"
world.clear_confirm = "Clear the world? Everything in it will be lost."
world.clear_yes = "Clear"
world.clear_no = "Cancel"
tools.heading = "Tools"
tools.brush = "Brush"
tools.eraser = "Eraser"
tools.heat = "Heat"
tools.cool = "Cool"
tools.brush_size = "Brush size"
tools.element_heading = "Element"
element.H = "Hydrogen"
element.He = "Helium"
element.C = "Carbon"
element.O = "Oxygen"
element.Ne = "Neon"
element.Mg = "Magnesium"
element.Si = "Silicon"
element.S = "Sulfur"
element.Fe = "Iron"
element.n = "Neutron matter"
presets.heading = "Drop a star"
presets.sun = "Sun-like star"
presets.sun_note = "1 Sun · ends as a white dwarf"
presets.massive = "Massive star"
presets.massive_note = "15 Suns · ends in a supernova"
presets.giant = "Giant star"
presets.giant_note = "40 Suns · ends as a black hole"
presets.place_hint = "Click in the world to drop the star. Esc cancels."
presets.no_room = "Not enough room here for this star."
views.heading = "View"
views.glow = "Glow"
views.heat = "Heat"
views.element = "Element"
views.density = "Density"
star.heading = "Star"
star.none = "No star yet. Paint gas or drop a star."
star.select_hint = "Right-click a star to select it."
star.mass = "Mass: {mass}"
star.fate = "Fate: {fate}"
star.ending = "Ending: {ending}"
star.age = "Age: {age}"
star.rate = "1 s ≈ {rate}"
star.surface = "Surface: {temperature}"
star.core = "Core: {temperature}"
fate.failed_star = "failed star (too light to ignite)"
fate.white_dwarf = "white dwarf"
fate.supernova = "supernova, then a neutron star"
fate.black_hole = "black hole"
fate.close_call = "{a} or {b} (close call)"
ending.failed_star = "failed star"
ending.white_dwarf = "white dwarf"
ending.neutron_star = "neutron star"
ending.black_hole = "black hole"
stage.cloud = "Gas cloud"
stage.collapsing = "Collapsing cloud"
stage.protostar = "Protostar: contracting"
stage.failed_star = "Failed star: too light to ignite"
stage.main_sequence = "Main sequence: burning hydrogen"
stage.h_shell = "Red giant: burning a hydrogen shell"
stage.h_shell_super = "Supergiant: burning a hydrogen shell"
stage.he_core = "Red giant: burning helium"
stage.he_core_super = "Supergiant: burning helium"
stage.he_shell = "Giant: burning a helium shell"
stage.c_burn = "Supergiant: burning carbon"
stage.ne_burn = "Supergiant: burning neon"
stage.o_burn = "Supergiant: burning oxygen"
stage.si_burn = "Supergiant: burning silicon"
stage.iron_core = "Supergiant: iron core"
stage.core_collapse = "Core collapse"
stage.supernova = "Supernova"
stage.planetary_nebula = "Planetary nebula: shedding its shell"
stage.white_dwarf = "White dwarf: cooling"
stage.neutron_star = "Neutron star"
stage.black_hole = "Black hole"
inspect.heading = "Cell"
inspect.empty = "Hover over a cell to inspect it."
inspect.vacuum = "Empty space"
inspect.mix_item = "{element} {percent}%"
inspect.mix_sep = " · "
inspect.temperature = "Temperature: {temperature}"
inspect.density = "Density: {density} (sandbox units)"
inspect.speed = "Speed: {speed} cells per time unit (Mach {mach})"
units.suns = "{value} Suns"
units.kelvin = "{value} K"
units.kelvin_million = "{value} million K"
units.kelvin_billion = "{value} billion K"
units.milliseconds = "{value} milliseconds"
units.seconds = "{value} seconds"
units.days = "{value} days"
units.years = "{value} years"
units.years_thousand = "{value} thousand years"
units.years_million = "{value} million years"
units.years_billion = "{value} billion years"
units.years_trillion = "{value} trillion years"
status.line = "{adapter} · {fps} fps · {steps} steps per frame · {width} × {height} cells"
error.no_adapter = "No compatible graphics card found. Sandbox Reactions needs a graphics card with Vulkan, Metal or DirectX 12 support."
error.device_lost = "The graphics card stopped responding. Please restart Sandbox Reactions."
web.loading = "Loading Sandbox Reactions…"
web.no_webgpu_title = "Sandbox Reactions needs WebGPU"
web.no_webgpu_body = "Your browser does not support WebGPU yet. Sandbox Reactions uses it to run its physics on your graphics card."
web.no_webgpu_browsers = "It runs in Google Chrome on Windows, macOS and ChromeOS (and on Linux with recent Intel or NVIDIA graphics), in Safari 26, and in Firefox on Windows and macOS."
```
The browser list restates ES-E3 (Chrome on Windows, macOS and ChromeOS since 113, on Linux for Intel
Gen12+ since 144 and NVIDIA on Wayland since 147; Safari 26; Firefox on Windows since 141 and macOS
since 147); a later amendment updates it when that page changes.

### 4.2 Layout (desktop; the web build draws the same into its canvas)
- Default window 1760 × 940 logical px (fits a 1920 × 1080 screen), title `app.title`.
- **Top bar** (44 px), left to right: Pause/Play · Step · the rung buttons (`time.rung`, rungs above
  TOP never shown) · the speed readout (`time.speed`, `time.speed_capped` or `time.paused`) · the
  `time.slowdown` checkbox · `edge.label` with its two choices · `world.clear`.
- **Left panel** (220 px): `tools.heading` — Brush, Eraser, Heat, Cool (one selected), the
  `tools.brush_size` slider (1–40 cells); `tools.element_heading` — the nine paintable elements with
  colour swatches (§1.5; neutron matter is never listed); `presets.heading` — three buttons, each
  with its note; `views.heading` — Glow, Heat, Element, Density.
- **Centre:** the world at integer scale s (§1.11), centred on `bg.space`; at the default size
  s = 2, so the world is 1200 × 800 px. While a preset is armed, `presets.place_hint` shows under
  the world and its outline follows the cursor.
- **Right panel** (300 px): `star.heading` and the star readout (§4.6); `inspect.heading` and the
  cell inspector (§4.6).
- **Banner:** over the top centre of the world, after an automatic slow-down (§1.8.5), until the
  player changes the rung.
- **Status bar** (24 px): `status.line`.
- Selected tool, rung and view use `accent`; disabled items `text.secondary`.

### 4.3 Time controls
Rung buttons in ladder order; keys 1…n (§4.10). The banner key matches the event: ignition →
`banner.ignition`, core collapse → `banner.core_collapse`, supernova → `banner.supernova`.
`time.speed_capped` replaces `time.speed` under §1.8.4's rule; `{actual}` takes two significant
digits.

### 4.4 Tools, brush and elements
Brush, Eraser, Heat and Cool act on the cells under a disk of `tools.brush_size` cells radius (1–40,
default 8) while the left button is held (§1.12). The element list paints the selected element
(default Hydrogen — B2: "Paint hydrogen gas with the brush").

### 4.5 Presets
A preset button arms it; the next left click in the world drops it there (§1.12) or shows
`presets.no_room` for 3 s if it does not fit; Esc disarms. The notes state the mass and the ending
each preset reaches (B3, RT17 — guaranteed by G-END).

### 4.6 The star readout and the inspector
- **Star** (the selected object, §1.9.6), one line each: the stage label (`stage.*`; "Supergiant"
  rows when M ≥ M_up) · `star.mass` · `star.fate` with `fate.*`, or `star.ending` with `ending.*`
  once a remnant holds (§1.9.5) · `star.age` · `star.rate` (hidden while paused) · `star.surface` ·
  `star.core`. No object: `star.none`; several objects: `star.select_hint` under the readout.
- **Cell** (the cell under the cursor): the mix — up to three species of ≥ 0.5 % by mass, joined by
  `inspect.mix_sep` — then `inspect.temperature`, `inspect.density`, `inspect.speed` (Mach =
  |u| / √(2Π/Σ)). Vacuum: `inspect.vacuum`. Cursor outside the world: `inspect.empty` (D6).

### 4.7 World actions
`world.clear` opens a dialog with `world.clear_confirm`, `world.clear_yes` and `world.clear_no`.

### 4.8 Errors
No adapter: the desktop app prints `SR-ERROR` + `error.no_adapter` and exits 3 (§3.3); a lost
device shows `error.device_lost` in a dialog and stops stepping.

### 4.9 Web pages
Loading: `web.loading` centred on `bg.space`. Without WebGPU (Q4): `web.no_webgpu_title` as a
heading, then `web.no_webgpu_body` and `web.no_webgpu_browsers` as paragraphs, in `text.primary` on
`bg.space` — no canvas, no wasm.

### 4.10 Keyboard and mouse
| Input | Action |
|---|---|
| Space | Pause / Play |
| . (period) | Step |
| 1 … 9 | select rung n (§1.8.2) |
| + or = / − | faster / slower |
| B · E · H · C | Brush · Eraser · Heat · Cool |
| [ / ] | brush size −2 / +2 |
| F1 · F2 · F3 · F4 | Glow · Heat · Element · Density view |
| Esc | disarm a preset |
| Delete | Clear world (opens the dialog) |
| left button | apply the tool, or drop the armed preset |
| right click | select the star under the cursor |
| hover | the cell inspector |

### 4.11 Design tokens
| Token | Value |
|---|---|
| bg.space | #05070D — the world's background and vacuum |
| bg.panel | #10141E |
| border.panel | #232A3A |
| text.primary | #E6EAF2 |
| text.secondary | #9AA4B8 |
| accent | #F2B33D |
| banner.bg / banner.text | #3A2A0B / #FFD27A |
| error.text | #FF6B6B |
| haze | #2A3550 (cold gas, §1.11) |
| font sizes | body 14 px · heading 18 px · banner 20 px · small 12 px (status line, preset notes); egui's default fonts |
| spacing | 8 px unit; panels 220 px (left) and 300 px (right); top bar 44 px; status bar 24 px |
| glow | threshold 0.8 · σ 3 px at half resolution · intensity 0.6 |
| heat map | stops at log₁₀ T_K = 1, 2.8, 4.6, 6.4, 8.2, 10: #000004, #3B0F70, #8C2981, #DE4968, #FE9F6D, #FCFDBF |
| density map | stops at log₁₀ Σ = −6, −3.5, −1, 1.5, 4: #440154, #3B528B, #21918C, #5EC962, #FDE725 |
| element colours | §1.5 |
Maps interpolate linearly in linear RGB between stops and clamp outside.

### 4.12 Number formats (English: "," groups thousands, "." is the decimal point)
- **Mass:** 3 significant digits, `units.suns` ("1.00 Suns", "15.0 Suns", "0.0800 Suns").
- **Temperature** (translated, §1.10.2): below 10⁶ K, 3 significant digits with grouping,
  `units.kelvin` ("5,770 K", "150,000 K"); 10⁶ to 10⁹ K, `units.kelvin_million` ("15.7 million K");
  above, `units.kelvin_billion`.
- **Age and rate:** the largest unit whose value is ≥ 1 — trillion, billion, million, thousand
  years, years, days (1 year = 365.25 days), seconds, milliseconds — at 3 significant digits
  ("4.57 billion years", "18.0 days", "3.00 milliseconds").
- **Speeds:** ladder rungs as written (×0.1 … ×1000); TOP and `{actual}` at 2 significant digits
  ("×42", "×0.83").
- **Inspector numbers:** 3 significant digits; scientific notation (1.23e-5) below 0.001 or at
  100,000 and above. **Mix percentages:** 3 significant digits ("73.5", "0.770").
- **fps** and **steps per frame:** integers.

## §5 Non-regression guarantees (R8; PLAYBOOK §8)

### 5.0 How to read a row
Each guarantee states: the claim · its oracle and source · the tolerance, fixed now · whether it is
satisfiable at authoring (*by construction*, *measured*, *standard* — opinion from common practice —
or *UNVERIFIED*, then listed in §5.5) · its verify.json scope · its plant — the bug that must turn
it red, kept as `tests/plants/<plant>.patch` and applied only in a scratch copy by
`verify.py --redarm` (PLAYBOOK §A.2). A physics row passes only against its oracle within its
tolerance; a capture that looks right proves looks only (R8). Any adapter unless a row names one;
elsewhere a box-bound row reads `owed on <box>`. ⏱ marks a row that runs whole lives — it may take
over 10 minutes on the Quadro (§6.4). A row's scripted edits come from an edits file (§2.12.5).

### 5.1 The laws
- **G-GRAV1 · the force law.** Maclaurin disk Σc√(1 − r²/a²): in-plane g(r) = −(π²GΣc/2a)·r (SM-E6,
  SM-O1). |Δg|/|g| ≤ 2 % at every cell with r ≤ 0.75a for a = 40; the largest error at a = 20 over
  the largest at a = 40 within [1.6, 2.4] (first order, SM-C1). *Measured* for the direct-sum force
  (SM-C1: 1.0–1.6 %); the central-difference force of §1.4.2 is unmeasured — if the first lot finds
  it over 2 %, the build convolves force kernels directly (two more inverse FFTs), the tolerance
  stays. Scope `grav-force` (GPU and CPU twin). Plant `grav-kernel-exponent`: K = −G/r^1.15.
- **G-GRAV2 · no net self-force.** An asymmetric blob (scenes/asym_blob.json: a disk and an
  off-centre rectangle): |Σ m·g| ≤ 10⁻⁵ Σ m|g| (SM-O4, Newton's third law). *By construction* (a
  symmetric kernel). Scope `grav-selfforce`. Plant `grav-kernel-shift`: the kernel offset one cell
  in x.
- **G-COLL · homologous collapse.** A cold Maclaurin disk (T_floor, Σc = 1, a = 40; heat and
  reactions off): r₅₀(t)/r₅₀(0) = cos²η with t/t_ff = (2/π)(η + ½ sin 2η), t_ff = ½√(a/GΣc) = 3.162
  — so 0.8368 at t = 0.5 t_ff and 0.5279 at 0.8 t_ff (SM-O2, SM-C2; my arithmetic), each within
  3 %. *Standard* (pressure is negligible at T_floor; the grid's force error is ≤ 2 %). Scope
  `collapse`. Plant `grav-half`: g × 0.5.
- **G-VIR · virial equilibrium.** The Sun-like preset 20 τ_dyn after ignition: |(2K + 2∫Π dA)/W + 1|
  ≤ 3 % (SM-O3; the sheet's 2K + 2∫Π dA + W = 0, SM-E2 with n = −1). *UNVERIFIED*. Scope `virial` ⏱.
  Plant `pressure-scale`: the pressure in the momentum flux × 1.2.
- **G-SOD · gas dynamics.** Sod's tube (SM-E14: ρ 1 | 0.125, P 1 | 0.1, at rest) on a 400 × 4 strip
  (bounce edges), γ = 2, at t = 80 t.u. (Sod's 0.2 on a unit domain, scaled to 400 cells), against
  the exact Riemann solution for γ = 2 computed in the test: the shock within 1 cell; L1 density
  error ≤ 2 % (SM-O5). *Standard* for second-order MUSCL at 400 cells. Scope `sod`. Plant
  `hll-swap`: the face flux takes the left state on both sides.
- **G-JEANS · no artificial fragmentation.** A disk cloud (U_th = 0.1|W|, a = 40) with a
  deterministic 1 % ripple (cos 3θ + cos 5θ), collapsing with J ≤ 1/4 everywhere until its central Σ
  is 100 × the start: exactly one object (§1.9.2) and no secondary density peak above 10 % of the
  central one (SM-O6, SM-E16). *UNVERIFIED*. Scope `jeans`. Plant `no-pressure-flux`: the pressure
  term dropped from the momentum flux.
- **G-HEAT1 · the diffusion law.** A Gaussian heat pulse in a uniform medium with a constant
  diffusivity χ (scene `test.constant_diffusivity`), heat only, bounce edges: σ²(t) = σ₀² + 2χt per
  axis within 1 %; total heat constant to 10⁻⁵ relative (SM-O7, SM-E21). *Standard*. Scope
  `heat-gauss`. Plant `chi-scale`: χ × 0.9.
- **G-HEAT2 · RKL2 matches explicit.** The same pulse by RKL2 against explicit sub-steps: σ² within
  1 % (SM-O8, SM-E18). *Standard*. Scope `heat-rkl2`. Plant `rkl2-s-minus-one`: one stage too few.
- **G-FLD · the flux limiter.** Thin gas (Σ = 10⁻⁴) around a hot spot: |F| ≤ c_sb a_r T⁴ (1 + 10⁻⁶)
  at every face (SM-E19). *By construction*. Scope `heat-limiter`. Plant `limiter-off`: λ ≡ 1/3.
- **G-CORE · the cold ceiling.** Cold C/O disks: at 0.95 m_ch_sb, R₉₀ steady within 3 % over
  10 τ_dyn; at 1.1 m_ch_sb, R₉₀ below 50 % of its start within 10 τ_dyn; the GPU's m_ch within 5 % of
  the CPU twin's; with the relativistic branch removed (pure Σ²), R₉₀(M) flat within 3 % over
  M ∈ {0.25, 0.5, 1} × m_ch_sb (SM-O9; SM-E4: "For D=2, the radius is independent on mass" — the
  sheet's analogue, SM M2-B). *UNVERIFIED* (the sheet's ceiling is SM's arithmetic). Scope
  `cold-core`. Plant `cold-exponent`: the relativistic exponent 3/2 → 1.6.
- **G-CONS · conservation.** A whole Sun-like life (bounce) and a supernova (leave; the Massive
  preset from its iron core): the mass invariant of §2.9 constant to 10⁻⁶ relative; momentum (grid
  + sinks + escaped) to 10⁻⁶ of Σm|u|; the energy invariant drifting ≤ 10⁻³ |W| per τ_dyn (SM-O10).
  Mass and momentum *by construction*; energy *UNVERIFIED* — source-term gravity conserves it only
  approximately (SM, from memory): more drift calls for a conservative gravity formulation, never a
  looser bound (§0.4). Scope `conservation` ⏱. Plant `no-grav-work`: the energy source Σu·g dropped.
- **G-BURN · the rate law.** One cell, T pinned, flow off, k = AΣ^a f(T): H_burn (order 2)
  X = X₀/(1 + kX₀t); He_burn (order 3) X = X₀/√(1 + 2kX₀²t); Ne_burn (order 1) X = X₀e^{−kt} —
  X within 10⁻⁴ absolute down to X₀/10; heat gained = q·Δm within 10⁻⁵ relative; ΣX = 1 within
  10⁻⁶; every record's shares conserve mass at load (SM-O11). *By construction*. Scope `burn-cell`.
  Plant `q-double`: H_burn's heat applied twice.
- **G-ORDER · the ignition order.** A cell of equal parts H, He, C, O, Ne and Si heated at a steady
  rate: each group's specific power first passes ε* = 10⁻³ Q_H per t.u. in the order H, He, C,
  {Ne, O}, Si (SM-O12, K6). Exact. *By construction* (§2.5.1's order bound). Scope `burn-order`.
  Plant `swap-he-c`: T_He and T_C exchanged.
- **G-THERMO · the thermostat.** The Sun-like preset at X_H,c = 0.5: a +10 % kick to the central
  5 × 5 cells' ε_th returns T_c within 2 % of its prior value inside 3 τ_KH (τ_KH = |W|/2L, measured),
  and T_c never exceeds 1.5 × that value (SM-O13; SM-E24; SP-E1). *UNVERIFIED*. Scope `thermostat`
  ⏱. Plant `burn-heat-to-ledger`: H_burn's q booked but never added to ε_th.
- **G-SINK · accretion.** A sink in a uniform bound cloud: grid + sink mass and momentum constant
  to 10⁻⁶ relative; no mass flows outward across r_acc (SM-O15, SM-E28). *By construction*. Scope
  `sink`. Plant `sink-momentum`: accreted momentum not given to the sink.
- **G-EDGE · both edges.** Bounce: the wall mass flux is exactly 0 and a pulse sent at a wall returns
  with its mass-weighted normal velocity within 2 % of −v₀; leave: mass lost from the grid equals
  `escaped` to 10⁻⁶ and inflow is exactly 0 (SM-O16). *By construction* (§2.8). Scope `edges`. Plant
  `wall-copy`: bounce ghosts copy instead of mirror.
- **G-REF · the CPU twin.** GPU f32 against the CPU f64 twin on three 64 × 64 scenes (a collapsing
  disk, a Sod strip, a burning field), 200 steps: relative L1 difference ≤ 10⁻³ in Σ and E, ≤ 10⁻⁴
  absolute in each X (SM §Oracles; ES-E20: floats differ between GPUs). *UNVERIFIED*. Scope
  `cpu-gpu`. Plant `cfl-gpu`: the GPU's CFL 0.4 → 0.39.

### 5.2 The squeeze, the endings, the stages, touch
- **G-SQUEEZE · the clocks' order** (K1–K3, K9; SM-O14). Sun-like main sequence: τ_KH/τ_dyn ≥ 10 and
  τ_nuc/τ_KH ≥ 10 (τ_dyn = √(R³/GM), τ_KH = |W|/2L, τ_nuc = X_H,c·M_core·Q_H/L, measured); the
  Massive preset's stage durations H > He > C > {Ne, O} > Si; main-sequence durations Sun-like >
  Massive > Giant. Orders exact; the gap of 10 is precedent, not proof (TW-E10–E12) — *UNVERIFIED*.
  Scope `squeeze` ⏱. Plant `q-h-tiny`: Q_H ÷ 100.
- **G-END · each ending as predicted** (RT17; B2, B3). The three presets end as their readouts
  predicted at the drop (white dwarf; supernova then neutron star; black hole); and preset-shaped
  clouds at each calibrated threshold ÷ 1.25 and × 1.25 in sandbox mass (outside the ±10 % close-call
  band) end on their side as predicted — failed star / white dwarf, white dwarf / neutron star,
  neutron star / black hole. Exact. summary.json records whether S1 or S2 was enabled. *UNVERIFIED*
  (shell ejection, the explosion — SM). Scope `endings` ⏱. Plant `fate-swap`: the prediction uses
  M_up where M_BH belongs.
- **G-STAGES · the visible life** (B4–B6). The displayed stage ids contain these as a subsequence,
  in this order (parenthesised ones may be missing; other stages may appear between): sun — cloud,
  (collapsing), protostar, main_sequence, h_shell, he_core, he_shell, planetary_nebula, white_dwarf;
  massive — cloud, (collapsing), protostar, main_sequence, (h_shell), he_core, c_burn, ne_burn and
  o_burn in either order, si_burn, iron_core, core_collapse, supernova, neutron_star — its h_shell
  and he_core labels the "Supergiant" ones (§1.9.3); giant — cloud, (collapsing), protostar,
  main_sequence, he_core, core_collapse, (supernova), black_hole. *UNVERIFIED*. Scope `stages` ⏱.
  Plant `no-push`: the radiation force zeroed (no planetary nebula while S1 is off).
- **G-TOUCH · touch any time** (B7: "Add gas to a shining star (it gets heavier and its fate
  changes), erase a chunk, heat or cool it"). Edits files: hydrogen painted onto the Sun-like preset
  at X_H,c = 0.5 until it weighs 1.25 × m_up_sb — it then ends as a neutron star or black hole, as
  its new prediction says; the Massive preset at X_H,c = 0.5 erased from outside in down to
  m_up_sb ÷ 1.25 — it then ends as a white dwarf. Exact. *UNVERIFIED*. Scope `touch` ⏱. Plant
  `edit-ignored`: P0 drops edits on non-vacuum cells.
- **G-FIT · every life fits the world.** Through each preset's life no bound gas
  (½|u|² + ε_th + φ < 0) comes within 16 cells of an edge of the default world (D5's squeezed
  ladder). *UNVERIFIED* (a tuning target). Scope `fit` ⏱. Plant `preset-wide`: preset radii × 3.
- **G-MULTI · several clouds allowed** (B15: "Allowed, one promised"). Two Sun-like presets
  240 cells apart, 20 τ_dyn: two objects with stable ids, no non-finite value, both readouts filled.
  *By construction*. Scope `multi`. Plant `one-object`: the tracker joins blocks across empty
  blocks.

### 5.3 Time, performance, determinism
- **G-WARP · warps never touch the physics** (TW-R2 T1). 2,000 steps of the Sun-like preset at ×0.1,
  ×1 and TOP, slow-down on and off: bit-identical dumps on one adapter. *By construction* (§1.3.4,
  §1.8.7). Scope `warp`. Plant `dt-from-frame`: Δt taken from the frame's owed time.
- **G-BOX · the box equals the world** (TW-R5 T2). The Sun-like preset, 2,000 steps, box against
  whole world: mass, momentum, energy and each species' mass equal within 10⁻⁶ relative (f64 sums);
  a blob at 0.35 cells per step aimed at the box's edge loses no mass in bounce mode (10⁻⁶). *By
  construction* (vacuum is static; FFT sizes differ in round-off only). Scope `box`. Plant
  `margin-zero`: the margin 8 → 0.
- **G-CAD · the gravity cadence** (TW-R5 T3) — only if §1.4.4 is built. k = 4 against k = 1 on the
  settled Sun-like star: energy-ledger drift ≤ 10⁻³ |W| per τ_dyn; k = 1 whenever §1.4.4's conditions
  fail. *UNVERIFIED*. Scope `cadence`. Plant `cadence-16`: k = 16 with the quiet test off.
- **G-LATCH · the collapse caught within one step** (TW-R4). The Massive preset from an iron-core
  dump at TOP, slow-down on: no step runs after the first step whose N_Fe rate is > 0 within that
  frame; the next frame runs at ×0.1 with `banner.core_collapse`. Exact. *By construction*. Scope
  `latch`. Plant `latch-off`: the controller never zeroes the dispatches.
- **G-FPS · the frame rate** (RT16; B14, I4). `bench` on linux-pc, adapter "Quadro RTX 4000" (the
  mid-range card: an RTX 2070 by its chip, ES-E21–E23), the Sun-like preset: from a
  mid-main-sequence dump, 600 frames at each rung below TOP — p95 frame ≤ 16.7 ms; the whole life at
  TOP — p95 ≤ 16.7 ms, or ≤ 33.3 ms once §1.8.6's switch engaged; the slowest frame ≤ 50 ms at any
  rung. *UNVERIFIED* (TW: holds only with the box and a small K1 gap). Box: linux-pc. Scope `fps` ⏱.
  Plant `no-clamp`: N_max ignored.
- **G-TOP · a Sun-like life in about 10 s** (B9; Q2). The same bench at TOP until
  `stage:white_dwarf`: life_wall_s ≤ 12. *UNVERIFIED* (TW-R8: not shown held). Red at 30 frames,
  the build stops for the lead (§1.8.6). Box: linux-pc, Quadro. Scope `top-speed` ⏱. Plant
  `owed-half`: owed grows by rung/120 per frame.

### 5.4 The product surface
- **G-WATCH · labels watch, never drive** (I2; B7's "every stage must come out of the physics, not a
  script"). A static scan: no file under crates/sr-engine/src/step/ or crates/sr-engine/shaders/
  names `observe`, `Stage`, `Tracker` or a calibration value other than the physics hash; the
  stand-ins read only state. *By construction*. Scope `watch-only`. Plant `stage-in-step`: `use
  sr_physics::observe::Stage;` added to the step module.
- **G-READ · the translations** (§1.10). Mass and temperature translations strictly increase over
  their whole range; each anchor maps exactly (10⁻⁹ relative); the presets read 1.00, 15.0 and
  40.0 Suns; the Sun-like preset's mid-main-sequence surface reads 5,772 K within 1 %. *By
  construction*. Scope `readouts`. Plant `anchor-reverse`: two mass anchors swapped.
- **G-AGE · the age clock** (TW-R7). Over the Massive preset's life with G-TOUCH-style edits (paint
  +50 % hydrogen onto the core at X_H,c = 0.5; erase 30 % of the envelope later) the age never
  decreases and no summary adds more than 10 × its stage's median increment; the Sun-like preset
  reaches its white-dwarf stage reading within a factor 1.5 of 1.21 × 10¹⁰ years (the τ_s sum at
  1 Sun, §1.10.4; SP's ~11 Gyr arithmetic). *By construction* and by calibration. Scope `age` ⏱.
  Plant `phi-unclamped`: Δφ < 0 not clamped.
- **G-CAL · calibration matches the physics** (§2.11). calibration.json's physics_hash equals the
  embedded assets' hash; on a mismatch the engine refuses (`SR-ERROR calibration out of date`, exit
  6). *By construction*. Scope `calibration`. Plant `const-drift`: one physics.json constant changed
  without recalibrating.
- **G-STR · the string table** (§4.1). crates/sr-app/src/strings.rs equals §4.1's block both ways —
  the same keys, byte-identical texts; web/index.html carries each `web.*` text verbatim. *By
  construction*. Scope `strings`. Plant `string-typo`: one character changed in `stage.supernova`.
- **G-LIC · no copyleft** (R7). `cargo metadata` over the workspace (native and wasm32 targets) and
  `npm ls --all --json` in web/: every package's licence expression has an alternative made only of
  MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Zlib,
  Unicode-3.0, Unicode-DFS-2016, BSL-1.0, CC0-1.0, MIT-0, 0BSD or Unlicense; a missing licence or any
  other (GPL, LGPL, AGPL and MPL among them) is NO-GO and an ask (PLAYBOOK §4 Rule 2). *Measured* for the
  approved table (Q5); the transitive tree is measured by the first build. Scope `licences`. Plant
  `gpl-dep`: a path crate with `license = "GPL-3.0-only"` added to the workspace.
- **G-BOOT · the headless boot.** `launch.py smoke` of `headless-boot` (§3.6) is GO within 60 s and
  its summary.json parses against §2.12.2. *By construction*. Scope `boot`. Plant `scene-refuse`:
  the loader rejects `preset:sun`.
- **G-DESK · the desktop boot on a private display** (R4). `launch.py smoke` of `game-xvfb` — on
  win-laptop of `game-offscreen` (§6.6) [M0-TJ3]: /status reads "ready" within 90 s. *UNVERIFIED*
  (lavapipe presenting on Xvfb; an off-screen window presenting on Windows). Scope `desktop`.
  Plant `status-never-ready`: the status stays "booting".
- **G-WEB · the web build** (Q4; the web kept alive). `trunk build --release`, the `web` service,
  web/smoke.mjs (§6.2.3) loading `/?preset=sun&steps=200` in headless Chrome: srState reaches
  "ready" within 30 s and the canvas's central 100 × 100 px are not all black; `/?no-webgpu=1` shows
  srState "no-webgpu" and the three `web.no_webgpu_*` texts exactly. *UNVERIFIED* (WebGPU in
  headless Chrome on linux-pc — ES — and on win-laptop, §6.2.3). Box: linux-pc or win-laptop, each
  with §6.2.3's Chrome [M0-TJ3 — was "Box: linux-pc"]. Scope `web`. Plant `never-ready`: srState never
  set to "ready".

### 5.5 UNVERIFIED — the satisfiability register
G-VIR, G-JEANS, G-CORE, G-CONS (energy), G-THERMO, G-REF, G-SQUEEZE, G-END, G-STAGES, G-TOUCH,
G-FIT, G-CAD, G-FPS, G-TOP, G-DESK, G-WEB rest on runs not yet made, and on the model risks behind
them: a red giant forming at all in the sheet; shell ejection (S1's trigger); the explosion (S2's
trigger); the onion of burning shells in a ~60-cell star; K1's gap of 10; the post-main-sequence
cost per unit time; the sheet's Jeans length (SM, TW UNVERIFIED lists). Each is first measured by
the lot whose `Deliver:` builds its pass; a red row there becomes a D block — or, when the
tolerance or a FROZEN clause itself is at stake, a question to the lead; never a looser number
(§0.4).

## §6 Modes and environments (R5)

### 6.1 linux-pc, the desktop
Vulkan through wgpu (ES-E1). The adapter: `--adapter <substring>` (case-insensitive, against the
adapter's name; no match → exit 3 naming the adapters seen), else wgpu's high-performance preference
among adapters that can present to the window; printed as `SR-ADAPTER …` and shown in the status
line. linux-pc has three GPUs and drives its display from the Quadro (Hazards): every performance
number names its adapter, and G-FPS and G-TOP name "Quadro RTX 4000". Windows through winit on
Wayland or X11 (eframe `wayland` and `x11`); the lead's session is Wayland with XWayland (Hazards).

### 6.2 The web build
- **6.2.1** trunk 0.21.14 builds web/dist from web/index.html (wasm32-unknown-unknown, wasm-bindgen
  0.2.129, wasm-opt in release builds).
- **6.2.2 One set of shaders.** The native device is created with WebGPU's default limits (wgpu's
  `Limits::default()`), so whatever runs natively runs in the browser (TW-E15: 16,384 B of workgroup
  storage, 256 invocations, 8 storage buffers per stage); M0 uses no native-only feature.
- **6.2.3 The web smoke.** web/smoke.mjs (project code) launches the installed Chrome
  (/usr/bin/google-chrome, 155 — probed 2026-10-08) through playwright-core, headless, with
  `--headless=new --use-angle=vulkan --enable-features=Vulkan --disable-vulkan-surface
  --enable-unsafe-webgpu` — Chrome's guidance for WebGPU in headless Chrome on Linux with NVIDIA
  (https://developer.chrome.com/blog/supercharge-web-ai-testing, 2024-01-16, accessed 2026-10-08),
  its `--no-sandbox` left out — records srAdapter, and runs G-WEB's checks.
  tools/pb/capture_web.mjs launches Chromium with no flags (PLAYBOOK §A.5's source), so WebGPU
  captures go through smoke.mjs; the no-WebGPU page captures through capture_web.mjs as is.
  UNVERIFIED until the first web smoke runs (ES).
  **On win-laptop** [M0-TJ3, the lead's Q2]: the installed Chrome at `C:\Program
  Files\Google\Chrome\Application\chrome.exe` (154 — probed by M0-TE-win, 2026-10-08), headless, with
  `--headless=new --enable-unsafe-webgpu` — Chrome's own GPU backend on Windows (D3D12), the Vulkan flags
  above being Linux's (opinion) — its `--no-sandbox` left out; smoke.mjs picks the box's Chrome and
  flags. UNVERIFIED until the first web smoke runs there; the block that runs it names what it used.
- **6.2.4 Browsers without WebGPU (Q4) — FROZEN.** The lead, 2026-10-08 11:23 EDT: **"A 'needs
  WebGPU' page (Recommended)"**. They get §4.9's page. The game ships no CPU physics (the CPU f64
  twin is a test oracle, never shipped) and no browser threads (no SharedArrayBuffer, no COOP/COEP
  — ES).
- **6.2.5 Hosting:** none in M0 (reports/web_hosting.md feeds a later milestone); the smoke serves
  web/dist locally.

### 6.3 Headless (R4)
wgpu with no surface (ES-E19): frames go to an offscreen RGBA8 texture and are read back; no window
and no display needed. The CPU twin needs no GPU at all.

### 6.4 Long runs
calibrate, the ⏱ scopes and benches at low rungs can pass 10 minutes on the Quadro; a run over 10
minutes is the lead's, in one visible terminal (PLAYBOOK §4 Rule 4; R2's exclusion). M0-TG places
them in V windows and splits what splits (`calibrate --only <item>`, one life per scope case).
Physics-only scopes may run on the RTX 5090 (`--adapter "RTX 5090"`) to stay short; timing scopes
never do. [M0-TJ3] On win-laptop they run on the RTX 4080 Laptop (`--adapter "RTX 4080"`, §6.6) — the
same oracles and tolerances, longer runs; one over 10 minutes is the lead's.

### 6.5 Builds and caches
The disk is 82 % full (Hazards): cargo's target directory is `build/target/`, set by
`.cargo/config.toml` (`[build] target-dir = "build/target"`), because verify.py's scratch copies leave
`build` out by name and copy a repo-root `target/` or `.cache/` into every red-arm arm (M0-TH, measured)
[M0-TG — replaces "cargo's target/" and the shared `.cache/redarm-target/`]. Red-arm scratch copies may
share one CARGO_TARGET_DIR outside the tree, `~/.cache/sandbox-reactions/redarm-target/` (a cache, safe
to delete), so a plant rebuilds only the workspace's crates — for the dependencies' artifacts, never
for a binary a scope launches (two arms would race on one uplifted binary); the mechanism is M0-T1's,
written in docs/agent/testing.md [M0-TG]. web/dist/, node_modules/ and trunk's cache are git-ignored.
`~/.cargo/bin` is off the PATH (Hazards): commands call `~/.cargo/bin/cargo` or source `~/.cargo/env`
first.

### 6.6 win-laptop [M0-TJ3 — replaces "Later (R5): …", mirrored in the plan's Superseded section]
In play for every M0 block, the lot V's included (the lead, 2026-10-08, at M0-TJ3: reports/win_laptop.md
§ Ruling; R5, R15). Hostname Laser2025-20; the agent's shell Git Bash (the plan's Repo facts). DX12 or Vulkan
through wgpu under §6.1's adapter rule (Vulkan preferred where a name matches on two backends); the
adapters, probed by M0-TE-win: NVIDIA GeForce RTX 4080 Laptop GPU and Intel Arc Graphics — WARP (Microsoft
Basic Render Driver) too where wgpu lists it (UNVERIFIED). Windows through winit's Win32 backend: no Xvfb,
Wayland or X11 here.
- **The window's checks run off-screen** (the lead's Q3): `--offscreen-window` (§3.3) on the box's
  high-performance adapter; the service `game-offscreen` (§3.6) is the window, capture and UI-cost checks'
  route here where linux-pc's is `game-xvfb` or `xvfb-run` — the same /status ready signal (§3.4). UNVERIFIED
  until M0-T5 runs it; if it cannot reach "ready", the window tasks run on linux-pc (Q3's consequence).
- **Physics** (the lead's Q2; §6.4): a scope, a `calibrate` run or a probe that names the RTX 5090 runs here
  on the RTX 4080 Laptop — results agree across adapters within G-REF's tolerances (§6.7), never bit for
  bit; every number names its adapter (R13).
- **Timing, never here:** G-FPS, G-TOP, the per-pass costs, step_cost.md and `--measure-ui`'s
  render_reserve_ms are the Quadro's (§6.1, §5.3; `box: laserax-ai`) and read `owed on laserax-ai` here (R15).
- **The web smoke:** §6.2.3's win-laptop line. **Launchers:** start.bat beside start.sh (§3.6).
- **Builds and caches:** §6.5 holds; `~/.cargo/bin` is on the PATH here; the disk is 97 % full (the plan's
  Hazards) — read the free space before a lot's first big build. The binary is `sandbox-reactions.exe`; a
  script never assumes the suffix away.

### 6.7 How far determinism reaches
Bit-identical on one adapter, driver and binary (§1.3.4); across adapters, results agree within
G-REF's tolerances (ES-E20).

## §7 Documentation — docs/agent/
Each lot's V checks the code against this contract and the page its lot owns.
| Page | Holds | From | First written by |
|---|---|---|---|
| architecture.md | the crates, modules, the pass order, data flow, the observer boundary | §1.1–§1.3, §1.9 | the first build lot |
| physics.md | the laws, the registries, the constants and their bounds, the stand-ins and their triggers | §1.4–§1.7, §2.1–§2.7 | each physics lot, for its pass |
| time_control.md | rungs, the frame loop, the cap, the latch, the 30-frames switch | §1.8 | the time-control lot |
| readouts.md | objects, stages, events, the translations, the age clock | §1.9, §1.10 | the observer lot |
| testing.md | §5's scopes and plants, the CPU twin, running headless, bench and calibrate | §3.3, §5 | M0-TH, then every lot that adds a scope |
| running.md | the launchers, adapters, the web build and its smoke, long runs | §3.4–§3.6, §6 | the first runnable lot |
Pages are English and dense; they cite this contract by section and never restate a number that this
contract or calibration.json holds — they point to it.
