# M0-R5 — Neutronization: the a_n_fe at which a collapsing iron core finishes as a neutron star

Box: linux-pc (laserax-ai), NVIDIA GeForce RTX 5090, Vulkan, driver 595.99.02 — **shared** throughout with two other projects'
GPU jobs (`backend/forecast/run.py`, `tests/run.py` of vision_ai_demo: 5090 at 100 % utilisation, 23 GB of 32 GB used, load 13–21
on 32 threads at 13:00 EDT); every wall time below is measured under that load. Date 2026-10-21. Every probe ran in the scratch copy
`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/6d5df45f-6ff7-44f5-a454-ddcb53e17161/scratchpad/` (`r5/` the tree copy,
`evidence/` the stalled life's folder copied before anything ran, `m1/`, `m2/`, `m3/`, `m4/` the probes); nothing landed in the tree
but this report, the plan's blocks and `logs/M0-R5.*`.

## Evidence

Sources, all accessed 2026-10-21:

- Liebendörfer, "A simple parameterization of the consequences of deleptonization for simulations of stellar core collapse",
  ApJ 633:1042 (2005) — https://arxiv.org/abs/astro-ph/0504072 (accessed 2026-10-21). Grid codes do not integrate electron
  captures during collapse; they read Y_e from a table of density alone, fitted to a Boltzmann-transport bounce profile — "fairly
  independent of time so that it can be used throughout the collapse phase", within 5 % of the reference. **The deleptonisation is
  a function of the state (ρ), not a slow clock**: whatever density the collapse reaches, Y_e is already at its value there.
- B. Müller, "Core-Collapse Supernovae and their Gravitational Wave Signals: The Status of Theory and Modeling" (review, v2 2026-07-15),
  §2.2 Collapse and Bounce — https://arxiv.org/abs/2603.24243 (accessed 2026-10-21): electron captures "reduce the electron degeneracy
  pressure in the core" as it nears the Chandrasekhar mass, "leading to gravitational collapse on a free-fall time scale"; "Electron
  capture removes lepton number from the core until densities of" ~10¹² g cm⁻³, when "the emitted electron neutrinos are trapped".
  The capture runs **with** the collapse, not after it: one free fall is the whole deleptonisation.
- Langanke, Martínez-Pinedo, Sampaio, Dean, Hix, Messer, Mezzacappa, Liebendörfer, Janka, Rampp, "Electron capture rates on nuclei and
  implications for stellar core collapse", PRL 90:241102 (2003) — https://arxiv.org/abs/astro-ph/0302459 (accessed 2026-10-21):
  "electron capture on nuclei dominates over capture on free protons", with "significant changes in core collapse simulations" — the
  rate is fast enough at collapse densities that its detail moves the bounce, i.e. it is not the bottleneck of the collapse's clock.
- Boccioli & Roberti, "The physics of Core-Collapse Supernovae: explosion mechanism and explosive nucleosynthesis" (2024), §3.1 The
  collapse phase — https://arxiv.org/abs/2403.12942 (accessed 2026-10-21): during collapse "the electron fraction goes from ∼0.42−0.45
  down to ∼0.3"; beyond the sonic point "matter is in approximate free fall".
- O'Connor & Ott, "A New Open-Source Code for Spherically-Symmetric Stellar Collapse to Neutron Stars and Black Holes", CQG 27:114103
  (2010) — https://arxiv.org/abs/0912.2393 (accessed 2026-10-21): GR1D carries "an approximate deleptonization scheme" for the collapse
  phase and "a neutrino-leakage/heating scheme" after bounce — a public grid code parameterises the capture, it does not resolve it
  (that the scheme is Liebendörfer's Y_e(ρ) fit is from memory — UNVERIFIED).
- Bertulani & Kajino, "Frontiers in Nuclear Astrophysics", Prog. Part. Nucl. Phys. (2016) —
  https://arxiv.org/abs/1604.03197 (accessed 2026-10-21): an iron core of 1.2–1.5 M☉, ~10⁸ g cm⁻³ at its edge and ~3 × 10⁹ at its centre, collapses on a free-fall time of order
  0.13 ρ₈^(−1/2) s, i.e. tenths of a second (from the search engine's excerpt of the paper; the PDF did not decode here — UNVERIFIED).

What the sources fix for the sandbox: (1) in nature, neutronization during collapse finishes **within about one free-fall time** of
the core reaching collapse densities — the capture timescale is not longer than τ_dyn there; (2) codes that cannot afford the rates
make Y_e a function of density (Liebendörfer), so a core that sits at a density is at that density's Y_e at once; (3) the full turn to
neutron matter (Y_e → ~0.1) is the proto-neutron star's cooling over seconds (tens of τ_dyn) — the sandbox's N_Fe carries both in one
record, iron → n. The sandbox's equivalent of "within a free fall" is 1/(a_n_fe · g(Σ)) ≲ a few τ_dyn at the collapsing core's Σ —
the block's criterion.

## Problem

The stall, separated by measurement. Probe M1-base: the shipped binary (M0-D78's, `build/target/release/sandbox-reactions`, a_n_fe
10⁻³) continued from the kept checkpoint `s_60001` with `headless --load-dump … --continue --steps 61001 --dump-every 100` (the scene
argument a file named `calibrate:m_bh_sb:4572.394` in the probe's folder, so `--continue`'s scene-label check passes — see M1 below),
1,000 steps, 193 s wall on the shared 5090 (5.2 steps/s); every dump read in f64 by the probe's own reader (`scratchpad/scripts/
dumpread.py`): the densest cell, Y_e = Σ_k X_k Z_k/A_k, the gate g = Y_eΣ/Σ_N − 1, the neutron mass ∫X_nΣ, v_r about the densest cell
in the object's centre-of-mass frame.

- **Why X_n,c rises ~7 × under §1.6.1's centre rate — dilution, not a slow law.** At s_60001 the whole remnant (2,534) sits within 3
  cells of its centre: the centre cell holds 403.2, its 8 neighbours 1,505 (Σ ≤ 234), the next 16 cells 593. **Only the centre cell has
  its gate open** (Y_eΣ = 0.459 × 403.2 = 185.1 > 160; the neighbours' Y_eΣ ≤ 107). Its rate is a_n_fe · X_Fe · g = 10⁻³ × 0.989 × 0.157 =
  1.55 × 10⁻⁴ per t.u. — §1.6.1's centre rate — so the core makes 403.2 × 1.55 × 10⁻⁴ = 0.0626 neutron mass per t.u., and the
  core's flow mixes it through all 2,534: X_n reads 0.0102–0.0112 in every ring out to r = 8. 0.0626 / 2,534 = **2.47 × 10⁻⁵ per t.u.,
  the measured rise** (M1-base: ∫X_nΣ 26.40 → 26.66 over 4.82 t.u., 0.053 per t.u.; X_n,c 0.01116 → 0.01128, 2.5 × 10⁻⁵ per t.u.). The
  burn sub-cycle and the observer's block are not the cause: the ratio 403/2,534 = 1/6.3 is the whole "7 ×".
- **Σ_c past the gate's regime: only transiently, and the gate itself caps X_n.** The gate is open while Y_eΣ > Σ_N, and N_Fe lowers Y_e
  (iron's 26/56 = 0.464 → neutrons' 0): a cell left at Σ neutronizes until **Y_e = Σ_N/Σ** and stops — the gate is a Y_e(Σ) law, the
  sandbox's form of Liebendörfer's Y_e(ρ) (Evidence), and a_n_fe sets only how fast a cell relaxes to it. For iron, X_n = 1 − Σ_N/(0.464 Σ):
  **X_n,c ≥ 0.5 needs Σ_c ≥ 2Σ_N/0.464 = 689 held**, whatever a_n_fe is. The continuum would collapse long before: the ceiling M_ch =
  1.4331 (K₂ₑY_e^{3/2})² = 25,733 Y_e³ (§2.3.2) is 2,576 for iron, 2,489 at the stall's Y_e 0.459 — under the remnant's 2,534 — and the
  neutron matter's 4,854 X_n³ is 6 at X_n 0.011; but the grid holds the core at Σ_c ≈ 403 for 20,000 t.u. at a_n_fe 10⁻³ and, once
  neutronized fast (M1 below), at Σ_c 350–600 with Y_e ≈ 0.25 and d_c 0.9999: the grid's one-cell scale holds what the continuum cannot,
  as §2.3.2 says of the cold ceiling ("no grid run can show it", reports/cold_ceiling.md). The remnant (2,534 against m_ch_sb 3,216.4,
  computed at Y_e = 0.5) is under the calibrated ceiling and also under its own continuum ceiling at Y_e 0.464 only by 2 %.
- **What holds v_r at −2.9 — the core itself, not a halo.** At s_60001 the mass outside r = 3 is 5 of 2,534; the mass-weighted v_r of
  the 8 neighbours is −4.64, of the next ring −1.79 (cells within r = 3: 2,529 of 2,534), and Σ_c is flat (403.2 → 402.4 in 1,000 steps)
  while that inflow would bring ~7,000 per t.u. into the centre. The momentum field points inward in a core whose mass does not move: a
  steady discretisation flow of P4 at a 2-cell core, the defect M0-D71 names ("P4 not well-balanced at a remnant's core"); the core's
  kinetic energy is 18,467 against its thermal 134,599 (observer, step 61,001). v_r reads −2.92 to −2.94 at every dump of M1-base and in
  the lead's window (57,009 → 60,001) — it does not decay. The neutron_star row's |v_r| ≤ 0.05 √(GM/R) = 1.78 is therefore failed by the
  remnant's own grid flow, independently of X_n,c.

## Options

How each probe ran. **M1** — the shipped binary continued from `s_60001` under `"overrides": {"a_n_fe": <v>}` (§2.12.1: tests only, a run
with overrides never writes calibration), `headless --scene calibrate:m_bh_sb:4572.394 --load-dump s_60001.bin --continue --until ending
--max-steps 70001 --dump-every 100 --adapter "RTX 5090"`, each capped at 570 s wall (the 10-minute line). **What `--continue` does with a
changed constant:** it accepts it — `read_carried` checks the snapshot's version, step, sim time, scene label and world, never its
`physics_hash` (M0-D77's snapshot carries one; only `calibrate --resume` compares it, and refuses the checkpoint: `SR-WARN checkpoint not
used (the checkpoint's physics_hash is …)`, the life from step 0). The scene label is matched as text, so the probe's scene is a file named
`calibrate:m_bh_sb:4572.394` (world 600 × 400 leave, no objects, the overrides) — the restored observers keep the life's `ignited`, its
stage trail and its events. **M2/M3** — `calibrate --only m_bh_sb --bracket 4572.393953137947,67020.64327658225` on the scratch build
with a_n_fe 100 in its physics.json (`scratchpad/sr-a100`), life 01 the low end, life 02 20 M₀. **M4** — `verify.py <scopes>` in the scratch
copy rebuilt at the value. Snapshots every 100 steps; "new row" is the candidate neutron_star row of the Recommendation (Y_e,c ≤ 0.35,
d_c ≥ 0.5, no sink), evaluated on each snapshot's observer object by `scratchpad/scripts/rowcheck.py` — the shipped row (X_n,c ≥ 0.5,
|v_r| ≤ 0.05 √(GM/R), no sink) beside it.

**M1 — the stalled remnant continued from s_60001** (one 570 s window each; the 5090 shared: 0.9–4.6 steps/s while a core neutronizes, 15 steps/s for 10⁻²'s near-stalled core):

| a_n_fe | steps run (t.u.) | M_n 26.4 → | X_n,c | Y_e,c | Σ_c | \|v_r\|/√(GM/R) | shown / raw stage at the end | shipped row | new row |
|---|---|---|---|---|---|---|---|---|---|
| 10⁻³ (shipped) | 1,000 (4.8), 193 s | 26.7 | 0.011 | 0.459 | 402 | 0.082 | core_collapse / core_collapse | never | never |
| 10⁻² | 8,700 (42.1) | 52.8 | 0.024 | 0.453 | 407–408 | 0.088 | core_collapse / core_collapse | never | never (~1,000 t.u. to 0.35 at this pace) |
| 10⁻¹ | 2,600 (12.7) | 113 | 0.045 | 0.443 | 403 → 106 | 0.011 | white_dwarf / white_dwarf | never | never |
| 1 | 1,200 (6.0) | 563 | 0.246 | 0.350 | 370–590 | 0.01–0.24 | core_collapse / core_collapse | never | at 0.350 by t + 6.0 |
| 10 | 900 (4.5) | 1,016 | 0.36–0.43 | 0.26–0.30 | 227–636 | 0.02–0.29 | core_collapse / core_collapse | never | every snapshot from + 0.5 t.u. |
| 100 | 800 (4.0) | 1,147 | 0.41–0.46 | 0.25–0.27 | 353–705 | 0.05–0.49 | core_collapse / white_dwarf | never | every snapshot from + 0.5 t.u. |
| 100, Σ_N 80 | 500 (2.5) | 1,691 | 0.69–0.74 | 0.12–0.14 | 458–742 | 0.04–0.33 | core_collapse / he_shell | once (+1.5 t.u.) | every snapshot |

- M1 reading. Every a_n_fe ≥ 10 turns the core over to its gate's Y_e = Σ_N/Σ within the first snapshot (≤ 0.5 t.u. ≈ 9 τ_dyn; the centre's
  own rate 1/(a g) = 0.64 t.u. at 10, 0.064 t.u. = 1.1 τ_dyn at 100), and then stops: X_n,c 0.41–0.46 (the gate's cap, Problem) with the
  gate re-opening whenever the sloshing core passes Y_eΣ = Σ_N, so the shown stage keeps returning to core_collapse (displayed at once,
  §1.9.3) and the remnant hold restarts. 10⁻¹ is slower than the core's own evolution: by t + 10 the half-neutronized centre's lost
  pressure is outrun by a re-expansion (Σ_c 403 → 106, R 2.0 → 3.5, v_r −0.3, the gate shut) into a quiet white dwarf of 4.5 % neutrons —
  a slow rate loses the neutron star outright. 1 is between: 6.0 t.u. to Y_e,c 0.35. No value makes the **shipped** row hold: its X_n,c ≥ 0.5
  is above the gate's cap at the Σ the grid core keeps, and its |v_r| bound is under the core's grid flow at every value (the stall's own
  0.082 included). Σ_N at its bound 80 lifts the cap (X_n,c 0.72) and the shipped row matches once, then |v_r| fails again (the raw stage
  flickers core_collapse / neutron_star / he_shell) — and a lower Σ_N opens the gate in lighter cores: m_up_sb and every iron scene move.
- M1 cost. A neutronizing core costs 3–15 × a stalled one's step on this shared card (0.9–4.6 against 15 steps/s for 10⁻²; P5 at its
  4,096-round ceiling most steps); the lead's own 60,001-step run averaged 11.5 steps/s on the same card.

**M2 — the low-end life from step 0 under a_n_fe 100** (`calibrate --only m_bh_sb --bracket …`, scratch build, `--progress-every 100
--checkpoint-every 1000`, 570 s cap): 1,601 steps (t 11.97) in 570 s, then cut — `failed_star` from 113, ignition (`by` fuel) at step 257
(the lead's life: 257), `white_dwarf` 305, `planetary_nebula` 1,313, `supernova` 1,505 — the lead's trail tick for tick to its supernova
(1,473 there): N_Fe's rate is 0 until a cell's Y_eΣ first passes Σ_N, so a_n_fe cannot move a life before its first gated cell (§1.6.1's g;
the probe shows the trail, not a bit-for-bit match: the lead's run kept no early dump). Its ending and wall time: **NOT RUN** — the post-
collapse steps ran at ~0.5 steps/s under the load; the whole life is the lead's (the answer below) — M0-D79's binary, the calibration's
m_bh_sb life 01.

**M3 — the bracket's high end, 20 M₀ = 67,020.643 sandbox mass, under a_n_fe 100: NOT RUN** — life 02 of the same line; m_up_sb's 14 M₀
life alone took 649 s to its collapse (2026-10-20); the lead's (the answer below). What would refuse the value: life 02 ending without a
sink (both ends neutron_star, "a bracket whose ends agree is refused after two lives"), or life 01 forming one (both black_hole).

**M4 — what else the value moves** (the scratch copy at a_n_fe 100; `verify.py registry rates calibration neutrino burn_cell latch --task
M0-R5-m4`, laserax-ai, 153 s, 84 passed / 4 failed):
- `registry` red — `physics_initial_values_verbatim` pins a_n_fe 1 × 10⁻³ (§2.5.1's table): moves with the value, by design.
- `rates` red — `omega_of_each_record_by_hand` writes N_Fe's hand value with A = 10⁻³ (1.75 × 10⁻⁴ against 17.5): moves with the value.
- `neutrino` red — `gpu_n_fe_absorbs_and_conserves_mass` (iron at Σ 500, T 1,000, one pass of 1 t.u.): "E gained −241,059, booked −282,673"
  — the same numbers at a_n_fe 1, 10 and 100 (`--task M0-R5-m4-a1`, `-a10`): at any a ≥ 1 the cell reaches its gate within the window and
  its heat reaches the floor, which P8 refills, so ΔE stops tracking the booking; its window must shrink with the rate (≤ 10 % of the iron
  converted). The other 17 cases GO (the gate's edges, the latch).
- `calibration` red — `the_calibration_the_binary_embeds_matches_the_assets`: `preset_mass_sb` missing, the same red as the tree's O13 today
  (the calibration is items 1–3 + m_up_sb); a changed constant also moves the physics_hash, so every item is re-measured anyway (G-CAL).
- `burn_cell` GO (16, iron only as a spectator below the gate), `latch` GO (5, its hook, not N_Fe's rate).
- The Sun-like life: none expected and none possible before iron — N_Fe's g is 0 until Y_eΣ > Σ_N, and the Sun-like preset never makes iron
  (`sun_burn` reads its main sequence); not run here.
- m_up_sb's side: the `core_collapse` event fires on L_N < 0, i.e. at the first gated cell, which a_n_fe does not move (M2's trail); the
  steps after it do move — m_up_sb's lives stop there (`collapse|ending`, [M0-D75]), so its value should not move, but the calibration
  re-measures it (the hash).
- The scopes whose scenes neutronize, from testing.md: `neutrino`, `latch` (its contract scenario arrives with M0-T111), and the calibrate
  lives; `rates` and `registry` read the constant. The presets' lives (item 6) run through their supernovae: the massive preset's ending
  must read neutron_star, the giant's black_hole — the lead's calibration.

## Recommendation

**a_n_fe = 100, and the neutron_star row read on the neutronized, degenerate centre: Y_e,c ≤ 0.35, d_c ≥ 0.5, no sink — listed after
supernova; the |v_r| term back once M0-D71's well-balanced P4 lands.** (The lead's pick, below.)

Why. (1) The rate: nature finishes the capture within the collapse's free fall (Evidence: Müller §2.2, Boccioli & Roberti §3.1), and grid
codes treat Y_e as a function of density (Liebendörfer) — the sandbox's gate already is that function (Y_e → Σ_N/Σ); a_n_fe only sets
the relaxation, and at 100 it is 1/(a g) = 0.064 t.u. = 1.1 τ_dyn at the stall's Σ_c 403 — "within a few τ_dyn". 10 takes ~10 τ_dyn and
re-opens the same way (M1); 1 takes 6 t.u.; 10⁻¹ loses the neutron star to a re-expansion (M1). (2) The row: no constant makes the
shipped row hold — its X_n,c ≥ 0.5 sits above the gate's own cap at the Σ a grid core keeps, and its |v_r| bound sits under the grid flow
of a 2-cell remnant (M1 at every value, the stall included). Y_e,c ≤ 0.35 is nature's centre at the end of collapse (0.42–0.45 → ~0.3,
Evidence; 0.35 keeps a margin above the gate's 0.25–0.30 measured at 10–100) and is impossible without neutron matter (iron 0.464,
everything else ≥ 0.5) — no white dwarf can match it; d_c ≥ 0.5 (degenerate support, as white_dwarf's row) stands for "held" while v_r
cannot be read on a 2-cell core. Listed after supernova so a supernova's 20 t.u. still shows; before core_collapse so the gate's
re-openings (L_N < 0 for a step) no longer restart the remnant's hold.

What would falsify it — each measured by M0-D79 or the lead's calibration on M0-D79's binary: m_bh_sb's life 01 (4,572.394) not ending
`neutron_star` within its `--max-steps`; life 02 (20 M₀) ending without a sink (both ends neutron_star — the bracket refused, "the same
refusal from the other side" — a fast rate making every collapse a black hole would show here as life 01 forming a sink: the low end's
core stayed at Σ_c ≤ 742 against Σ_BH 6,000 at 100 in M1, never near it); the massive preset ending other than neutron_star or losing its
`supernova` event; any pre-iron life moving (it cannot: g = 0 below the gate). What a wrong a_n_fe breaks: ≤ 10⁻¹ → the low end ends
white_dwarf (exit 6 again, M1-10⁻¹); ≫ 100 → only cost (more burn sub-cycles per step: ΔX ≤ 0.1 each) and the `neutrino` window.

**The contract's §0.5 line (M0-D79 writes it, its Superseded mirror beside it):**

> - [M0-D79] §2.5.1's "A per record … others 1 × 10⁻³" for N_Fe and §1.9.3's neutron_star row ("X_n,c ≥ 0.5, |v_r| ≤ 0.05 √(GM/R), no
>   sink", listed before supernova) · **a_n_fe = 100**: a collapsing iron core reaches its gate's Y_e = Σ_N/Σ within about one τ_dyn
>   (1/(a_n_fe g) = 0.064 t.u. at Σ 403, τ_dyn 0.056), as electron capture finishes within a free fall · **neutron_star: Y_e,c ≤ 0.35
>   (Y_e,c = Σ_k X_k,c Z_k/A_k at c), d_c ≥ 0.5, no sink, listed after supernova** — the gate caps an iron centre at X_n = 1 − Σ_N/(0.464 Σ)
>   (0.41–0.46 measured, the row asked 0.5) and a 2-cell remnant's grid flow holds |v_r| at 0.05–0.49 √(GM/R) (M0-D71); the |v_r| term
>   returns with M0-D71's well-balanced P4 · measured on linux-pc, 2026-10-21, RTX 5090: m_bh_sb's low end (4,572.394) held core_collapse
>   from step 2,001 to 60,001 at 10⁻³ (X_n,c +2.5 × 10⁻⁵ per t.u., ~20,000 t.u. to 0.5); continued from that step at 100 the new row holds
>   at every snapshot from 0.5 t.u. on (reports/neutronization.md M1) — the lead, 2026-10-21, at M0-R5: «Rate + NS row (Recommended)».

**M0-D79 re-scoped** (written into its block): Deliver — a_n_fe 100 in assets/physics.json; the neutron_star row in
`crates/sr-physics/src/observe/stages.rs` (its place after supernova, Y_e,c from the centre's X, d_c ≥ 0.5, no |v_r|) with classify's row
cases (the table-order check, the row either side of Y_e,c 0.35 and d_c 0.5, a white dwarf never matching) and a plant (the row back to
X_n,c ≥ 0.5 and |v_r|); the tests the value moves: registry's verbatim pin, rates' N_Fe hand value, neutrino's
`gpu_n_fe_absorbs_and_conserves_mass` window cut to ≤ 10 % of the iron converted; the §0.5 line and its Superseded mirror; readouts.md /
testing.md rows; a flag into M0-D71 (restore the |v_r| term); the lead's calibration lines (its Carried flags) in the handoff. Verify —
`verify.py registry rates neutrino classify burn_cell latch --task M0-D79` GO; `--redarm classify` and `--redarm neutrino` each plant red.

## UNVERIFIED

- M2's and M3's endings and wall times — NOT RUN (the lead's answer: in M0-D79 and the calibration); the low end's neutron_star ending under
  the new row and the high end's black hole are predictions from M1's snapshots and M0-D75's 14 M₀ life, not measurements.
- The new row is evaluated on observer snapshots every 100 steps, not per frame: a raw flicker between snapshots (the stage needs 3
  identical frames to display) is not excluded — NOT PROVEN (sampled). M1's windows are 2.5–42.1 t.u. each, short of the block's ≥ 50 t.u.
  (the shared card's pace); the hold of max(10 t.u., 20 τ_dyn) itself was not reached in any probe.
- Why the 10⁻¹ core re-expanded (Σ_c 403 → 106, R 2 → 3.5) is not separated. My reading, unmeasured: N_Fe lowers Y_e, the electrons'
  cold energy u_e(Y_eΣ) falls inside an unchanged E, so ε_th = E/Σ − ½u² − ε_cold rises — the electrons' Fermi energy becomes heat (in
  nature it leaves with the neutrinos); at 100 T_c reached 26.6 and d_c 0.82 for a few hundred steps (M1-100, steps 60,300–60,400). Not
  filed: not measured to its cause (a ledger split of ε_cold against ε_th over a probe would) — named in the handoff.
- The N_Fe absorption in a cold core is floor-fed: at T_floor its q·ω takes heat the cell does not hold and P8 refills it (booked
  `floor_added`); L_N −620 at the stall, bursts of −2.6 × 10⁷ at 100 — booked both sides (G-CONS), no energy created unbooked; not separated
  per term here.
- That GR1D's collapse scheme is Liebendörfer's Y_e(ρ) fit; Bertulani & Kajino's 0.13 ρ₈^(−1/2) s (from a search excerpt, the PDF did not
  decode).
- The Sun-like life and m_up_sb's value unchanged — by the gate's form (g = 0 below Σ_N) and M2's trail to its supernova, not by a run;
  the calibration re-measures both (the physics_hash moves).

## The lead's answer (2026-10-21, PLAYBOOK §12, one question call, verbatim)

- Q1 (the fix): «Rate + NS row (Recommended)».
- Q2 (the long probes M1 ≥ 50 t.u., M2, M3): «In D79 + calibration (Recommended)».
