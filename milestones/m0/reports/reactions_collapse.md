# M0-R10 — Reactions research: why a core-collapse step costs ~3 s in P6, and a cheaper exact step for the neutronization

Box: every GPU number below is laserax-ai-server's **NVIDIA GeForce RTX 4090** (Vulkan, driver 595.99.02), measured on 2026-10-22 with the
box **idle** (`nvidia-smi` 0 % on both cards before each set, load average 0.07–0.40, nothing else running; R18). Every probe ran in two
scratch copies on the server, made by `tools/pb/scratch_copy.sh` from `~/workspace/sandbox_reactions` (03e763b, M0-D85, the tree identical
to linux-pc's in crates/assets/tests): `/tmp/r10s` (untouched, the shipped binary: it made the 20 M₀ life's cut) and `/tmp/r10` (the probe
binary below); the probe patch was written in `/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/803cc716-cc37-40eb-b6d3-e80611044cd1/scratchpad/r10`
on linux-pc. Nothing landed in the tree but this report, the plan's block and `logs/M0-R10.*`.

**The probe binary** (`/tmp/r10`, 3 files: react.wgsl, step/react.rs, headless.rs): P6's report grows from 8 to 16 words — each accepted
sub-step counted by the limit that set its h (time left, burn_dx, burn_dt, the carried 2h), the sub-steps that started at T_floor, cells
capped, cells stepped exactly; `SR_PROBE_REACT=1` prints them after every step (one step a chunk) with the step's wall; the header's spare
words carry a mode, `SR_PROBE_REACT_MODE` = **0** as shipped (bit-for-bit the shipped arithmetic: the limits rewritten as `if x < h`), **1**
option (b), **2** option (c) with `SR_PROBE_REACT_CAP`, **3** option (d). The modes act on the GPU only (the twin untouched).

## Evidence

Sources, all accessed 2026-10-22:

- Zingale, Katz, Nonaka & Rasmussen, "An Improved Method for Coupling Hydrodynamics with Astrophysical Reaction Networks", ApJ (2022) —
  https://arxiv.org/abs/2206.01285 (accessed 2026-10-22): grid codes split the stiff reactions from explicit hydrodynamics ("coupling of
  explicit hydrodynamics and stiff reaction sources", "compared to traditional Strang operator splitting"); the coupling is hardest "during
  explosive burning". From the search engine's excerpts of the same paper (not read in the PDF here — UNVERIFIED in wording): Castro limits
  the step so burning cannot change a cell's internal energy by more than a fraction f_e, a limit "used in a reactive manner — the next step
  will see a smaller timestep … but the current step is still accepted", and a step is retried with sub-cycled smaller steps when "the ODE
  integration [fails] to converge in the implicit solve (due to too many steps or the internal timestep falling below a minimum)". **A
  production code bounds energy change per step and treats an integrator that runs out of steps as a failure to retry, not a result to keep.**
- Timmes, "Integration of Nuclear Reaction Networks for Stellar Hydrodynamics", ApJS 124, 241 (1999) — bibcode 1999ApJS..124..241T, listed
  at https://cococubed.com/code_pages/net_integration.shtml (accessed 2026-10-22): the standard survey of stiff network integration in
  hydro codes — a first-order semi-implicit (linearised backward) Euler, a fourth-order Kaps–Rentrop and a variable-order Bader–Deuflhard,
  compared by cost per step and decomposition (the page shows them as figures: "CPU of Euler method", "Cost per decomposition", "Relative
  efficiency"). §2.5.2's sub-cycles are the first of these.
- Hochbruck & Ostermann, "Exponential integrators", Acta Numerica 19, 209–286 (2010), doi:10.1017/S0962492910000048 —
  https://www.cambridge.org/core/journals/acta-numerica/article/abs/exponential-integrators/8ED12FD70C2491C4F3FB7A0ACF922FCD (accessed
  2026-10-22): for stiff problems "characterized by a Jacobian that possesses eigenvalues with large negative real parts", integrators built
  on the exact solution of the stiff part carry "error bounds that are independent of stiffness". **Where the stiff part has a closed form,
  using it removes the step-size limit stiffness imposes.**
- Gutierrez, Reyes & Rosu, "A note on Verhulst's logistic equation and related logistic maps", J. Phys. A 43 (2010) —
  https://arxiv.org/abs/0910.1560 (accessed 2026-10-22): the logistic equation treated through "the general Riccati solution". N_Fe's law,
  read for Y_e, is a constant-coefficient Riccati (logistic) equation (below); its closed form is my derivation, checked against M2.

**The closed form of N_Fe** (§1.6.2: f ≡ 1, a = 0, one input of order 1, the gate p = 1; Σ is fixed in P6). With y_Fe = Z/A = 26/56 and
a_Y = the Y_e of every species but iron (constant in N_Fe's step), X_Fe = (Y_e − a_Y)/y_Fe and

  dY_e/dt = −y_Fe · A Σ^a X_Fe (Y_eΣ/Σ_N − 1) = −k (Y_e − a_Y)(Y_e − b),  k = A Σ^a Σ/Σ_N,  b = Σ_N/Σ.

Partial fractions give (Y − b)/(Y − a_Y) = (Y₀ − b)/(Y₀ − a_Y) · e^{−λt}, λ = k(b − a_Y), i.e. with d = Y₀ − b and E = e^{−λΔt}:

  **Y(Δt) − b = d E / (1 + d (1 − E)/(b − a_Y))**,  (1 − E)/(b − a_Y) → kΔt as b → a_Y,

valid on both sides (b > a_Y: Y relaxes to the gate, X_n,∞ = 1 − Σ_N/(0.464 Σ) for pure iron; b < a_Y: the iron runs out first). The heat
is q · ΔX_Fe, exact. It is M0-R5's "Y_e(Σ) law" (reports/neutronization.md), reached at the rate a_n_fe sets, with no sub-step.

## Problem

**M1** — **Why a collapse step costs ~3 s: N_Fe drives the cell to T_floor, where burn_dt's predictor sets h ≈ 7.6 × 10⁻⁹ and every cell
spends MAX_ATTEMPTS.** The iron block (tasks/M0-D80.md § The test scene: 16 × 16 Fe at Σ 400, T 1, 64 × 64, gravity on, hydro, heat,
neutrinos and sinks off), 20 steps, shipped mode (logs/M0-R10.iron.log, `m1_base`): on steps 1–15 **all 256 cells ran out of attempts**
(`unfinished=256`, 65,536 sub-steps each, 16,777,216 a step); every sub-step's h was set by **burn_dt** (16,777,472 of 16,777,216 + retries;
burn_dx 0, time left 0) and **started at T_floor** (16,777,472); 3.85 s a step. Steps 16–20 fall to 2.1, 1.1, 0.63, 0.35, 0.19 s as the
gate closes (the cells then finish: `unfinished=0`). By hand: N_Fe absorbs q = −1.14 Q_H = −9,918 per unit mass converted; a cell of T 1
holds ε_th = T/μ ≈ 0.48, so the first ~5 × 10⁻⁵ of conversion takes T to the floor (0.05), where it stays. There the predictor
h = burn_dt T/|dT/dt|, dT/dt ≈ μ q ω = 2.074 × 9,918 × 16.07 = 3.3 × 10⁵, gives h = 0.05 × 0.05/3.3 × 10⁵ = **7.6 × 10⁻⁹** t.u.; Δt = 5.87 ×
10⁻³ needs 7.8 × 10⁵ of them; MAX_ATTEMPTS 65,536 covers 4.96 × 10⁻⁴ t.u., **8.4 % of Δt**. The check itself never objects: T floored
moves 0 % (`retried` 0 on steps 2–15). burn_dx (h ≤ 0.1/ω = 6.2 × 10⁻³ ≥ Δt) never binds; 1/(a_n_fe g) = 1/16.07 = 0.062 t.u. — N_Fe is not
stiff against Δt at all; **the cost is the T limit predicting a change the floor will not let happen.**

**M2** — **The shipped step is also wrong, not only slow: the cap leaves 92 % of each Δt unburned, so the neutronization runs ~8 × slower
than a_n_fe says, and Δt-dependent.** The block's mean X_n against the closed form (§ Evidence; the block does not move, so each cell is
the 0-D problem; logs/M0-R10.iron.log, reader `r10read.py`):

| step (t) | closed form | shipped (0) | (b) exact | (d) floor-aware | (c) cap 256 | (c) cap 4,096 |
|---|---|---|---|---|---|---|
| 1 (0.0059) | 0.066601 | 0.008028 (−5.9 × 10⁻²) | 0.066601 (+4 × 10⁻⁸) | 0.070314 (+3.7 × 10⁻³) | 0.000069 | 0.000536 |
| 2 (0.0118) | 0.1001 | 0.016096 (−8.4 × 10⁻²) | +9 × 10⁻⁹ | +1.2 × 10⁻² | — | — |
| 4 (0.0237) | 0.1272 | 0.032441 (−9.5 × 10⁻²) | +6 × 10⁻⁸ | +7.1 × 10⁻³ | 0.000163 | 0.002032 |
| 8 (0.0475) | 0.1374 | 0.065980 (−7.2 × 10⁻²) | +4 × 10⁻⁸ | +9.1 × 10⁻⁴ | 0.000283 | 0.004024 |
| 20 (0.119) | 0.138461 | 0.137912 (−5.5 × 10⁻⁴) | +1.5 × 10⁻⁸ | +7.5 × 10⁻⁷ | 0.000656 | 0.010043 |

The shipped X_n gains 0.008 a step, exactly 16.07 × 4.96 × 10⁻⁴ (M1's cap): its effective rate is a_n_fe × 65,536 h/Δt, a number set by
MAX_ATTEMPTS, T_floor and Δt — not by §1.6.1. reports/neutronization.md's and M0-D79's "a_n_fe 100" lives ran on it.

**M3** — The iron block, `--timing` over 20 steps, 3 runs a mode (logs/M0-R10.iron.log § M3), P6's mean ms a step: **shipped 3,113.7**
(3,113.7, 3,115.8, 3,113.4; every other pass ≤ 0.012 ms), **(b) 0.0347** (0.0348, 0.0347, 0.0347), **(d) 0.609** (0.610, 0.609, 0.612; step 1
alone took 145 sub-steps in its worst cell). Wall: shipped 62.3 s; (b) 0.020 s; (d) 0.031 s.

**M4** — **The 20 M₀ life's collapse is the same mechanism, carried by 4–12 cells: one GPU thread's 65,536 serial sub-steps set the step.**
The shipped copy ran `calibrate --only m_bh_sb --bracket 67020.643,67087.664` (logs/M0-R10.life.log): steps to 1,554 cost 11–16 ms each,
raw `core_collapse` from **step 1,571 to 1,595** at **3.41–3.47 s a step** (2.88–3.14 at 1,585–1,587), raw `neutron_star` from 1,596, still
3.42 s at 1,612 and 1.74 s a step over 1,612–1,660, `P5_ceil` 0 throughout. (The 5090's record shows core_collapse at 1,489: M0-D81's
split.) Cut at 1,560 (`--max-steps 1560 --checkpoint-every 1560`), then continued to 1,700 by `calibrate --resume` under each mode (140
steps, one step a chunk, `m4sum.py`):

| mode | wall 1,561–1,700 | N_Fe live on | median ms a step with N_Fe live / without | worst cell | unfinished cells |
|---|---|---|---|---|---|
| shipped (0) | **409.8 s** | 123 steps (1,565–1,700, still live) | **3,514.6** / 26.6 | 65,536 sub-steps | 4–8 a step |
| (b) exact | **7.1 s** | 88 steps (1,565–1,688) | 51.1 / 39.7 | 2 | 0 |
| (d) floor-aware | **13.6 s** | 92 steps (1,565–1,688) | 62.0 / 48.0 | 32,775 | up to 2 |

Shipped: 5.96 × 10⁷ sub-steps over the 140, 4.14 × 10⁷ of them set by burn_dt and 4.13 × 10⁷ at T_floor; burn_dx set none. **Other records
do not sub-cycle with it**: on every step the ~127,000 other burning cells took one sub-step each (h = the time left, `max=1` on steps
without N_Fe); the sub-cycling is N_Fe's cells alone.

**M5** — **What (b) and (d) move in the life: the neutronization ~3 × faster, at the rate §1.6.1 writes.** The core's numbers at the same
steps (`m5rows.py`; the event `core_collapse` at step 1,565, t 10.118, M 66,458.1 in all three):

| step | shipped Ye_c / X_n,c | (b) Ye_c / X_n,c | (d) Ye_c |
|---|---|---|---|
| 1,566 | 0.4545 / 0.0211 | 0.4461 / 0.0391 | 0.4452 |
| 1,567 | 0.4504 / 0.0298 | 0.4301 / 0.0737 | 0.4288 |
| 1,570 | 0.4387 / 0.0551 | 0.3800 / 0.1816 | 0.3781 |
| raw neutron_star from | **1,596** (Ye_c 0.3484) | **1,573** (0.3448) | **1,573** (0.3431) |

L_N at 1,570: −3.6 × 10⁷ shipped, −1.25 × 10⁸ (b) — the same heat, absorbed sooner. T_c stays at the floor (0.050) in every mode.

**M6** — **P5's share of the collapse is under 1 %.** `headless --load-dump s_1560 --continue --steps 1590 --timing` (steps 1,561–1,590), 3
runs a mode (logs/M0-R10.life.log § M6), mean ms a step:

| mode | P6 | P5 | step | wall (30 steps) |
|---|---|---|---|---|
| shipped | **2,951.9** (2,951.9, 2,951.9, 2,954.9) | 22.5 | 2,975.9 | 106.1 s |
| (b) | **0.465** (0.466, 0.465, 0.465) | 28.3 | 30.2 | 17.7 s |
| (d) | **11.17** (11.12, 11.17, 11.19) | 28.3 | 40.9 | 18.0 s |

Every other pass ≤ 0.87 ms. P5 rises 22.5 → 28.3 ms because the core's state moves (M5), not because of P6. So M0-R7, M0-R8 and M0-R9's heat
work was the right answer for the hard stretch at 553–558 and not for the collapse: the collapse is 99.2 % P6.

**M7** — **What the scopes read** (GPU tests in `/tmp/r10`, `SR_TEST_ADAPTER="RTX 4090"`, logs/M0-R10.tests.log; the twin untouched):

| module | shipped (0) | (b) exact | (d) floor-aware |
|---|---|---|---|
| `burn::` (burn_cell 16, burn_order 4, + sunburn) | 21 passed, 3 failed | 21 / 3 | 21 / 3 |
| `neutrino` (18) | **18 passed, 59.9 s** | **17 passed, 1 failed, 1.6 s** | 18 passed, 1.6 s |
| `latch` (5), `collapse` (6), `cpu_gpu` (3) | all pass | all pass | all pass |
| `twin` (6) | 4 passed, 2 failed | 4 / 2 | — |

The 3 sunburn and 2 twin failures are the known `SR-ERROR calibration invalid: preset_mass_sb: missing` (exit 6; M0-V9's), red in every
mode. **(b)'s one red:** `neutrino::gpu::gpu_latch_is_set_by_a_rate_that_closes_within_the_step` — «the gate stayed open: Fe fell by
0.009900868, 0.00990099 closes it»: over 10⁶ t.u. the closed form lands **on** the gate (Y_eΣ = Σ_N within f32), never past it, while
linearised backward Euler overshoots it; the build must land a cell whose remaining gap is under f32's resolution at Y_eΣ ≤ Σ_N, or the case
reads "at the gate". The case's own latch assertions pass.

**M8** — **The whole life under (b)** (`calibrate --resume` from the 1,560 cut, mode 1, no per-step reads, timeout 540 s; logs/M0-R10.end.log):
steps 1,560 → **28,015** (t 148.9) in 540 s, no ending yet (the life was still neutron_star, holding); **the slowest frame 77 ms a step** over
the whole run. Its shown trail: neutron_star 2,008 · core_collapse 3,369 · neutron_star 4,009 · raw core_collapse 4,794–4,796 then raw
white_dwarf (M0-D81's reading) · neutron_star 6,013 · core_collapse 7,470 · supernova 10,015–12,015 · neutron_star from 14,015. NOT a
measured gain against the shipped life: the shipped life on today's binary was not run past 1,700 (one N_Fe-live step costs 3.5 s; the
whole life is over the 10-minute line — the lead's, below). For scale only: the 5090's shipped record of the same life (an older binary,
Jacobi P5) took 5,980 s for 27,249 steps.

## Options

| option | ms per collapse step (life, P6 / step, M6) | iron block P6 (M3) | answers it moves | effort |
|---|---|---|---|---|
| **(a) as it is** | 2,951.9 / 2,975.9 | 3,113.7 | none — and keeps M2's 8 × slow, Δt-dependent neutronization | 0 |
| **(b) N_Fe stepped exactly** (closed form over Δt, then the other records sub-cycle as before) | **0.465 / 30.2** | **0.0347** | the neutronization to §1.6.1's rate: iron block within 6 × 10⁻⁸ of the closed form; the life's raw neutron_star 23 steps sooner (1,573 vs 1,596); one neutrino case's "past the gate" | 2 D blocks (twin, then GPU — R13, Considered and REJECTED) |
| **(c) N_Fe's sub-steps capped, remainder deferred** | cap 4,096: 243 ms a step on the iron block | 30.6 ms (cap 256) – 258 ms (cap 4,096) | the neutronization **slower**: X_n 6.6 × 10⁻⁴ (cap 256) or 0.010 (cap 4,096) after 20 steps against 0.138 — undoes M0-D79's a_n_fe 100 by ×200–×1,000 | 1 D |
| **(d) a floor-aware T predictor** (no T limit when cooling within burn_dt of T_floor; generic, any absorbing record) | 11.17 / 40.9 | 0.609 | the neutronization to ~§1.6.1's rate, but up to +1.2 × 10⁻² ahead of the closed form (the gate's Y_e-dependence is outside J); a worst cell still 32,775 sub-steps, 2 cells unfinished in the life | 2 D blocks (twin, then GPU) |

What each moves beyond the table:
- **burn_cell, burn_order**: unmoved by (b) and (d) (M7). Their N_Fe rows (burn_order: "N never passes", "N_Fe's q < 0 → N", q·Δm within 1e-5)
  read (b)'s booking as one exact q·ΔX; the twin needs the same step (G-REF, cpu_gpu) — the probe changed the GPU alone.
- **neutrino**: (b) one case to re-read (M7); the scope's run 59.9 s → 1.6 s.
- **The neutronization's timing against reports/neutronization.md**: R5 measured "the new row holds at every snapshot from 0.5 t.u. on" on the
  capped scheme; (b) gives the rate R5's criterion was written for (1/(a_n_fe g) ≲ τ_dyn), so the same row is met sooner. R5's Evidence
  ("neutronization … finishes within about one free-fall time") is better met, not contradicted.
- **m_up_sb**: its lives stop at core_collapse (§0.5 [M0-D75]), which fires at the first N_Fe rate > 0 (the same step, 1,565, in every mode:
  M5) — expected unmoved, NOT measured. **m_bh_sb**: decided after collapse (neutron_star against black_hole); (b) moves the collapsing core
  (M5, M8) — M0-R6's and M0-D81's readings were taken on the capped scheme. calibration.json's physics_hash covers the three asset files only,
  so a P6 code change leaves its values valid by hash but measured on (a): M0-V9's calibration, still owed, would be the first on (b).

## Recommendation

**(b): step N_Fe in closed form over the pass's Δt, then sub-cycle the other records as now — twin first, then the GPU.**

Why: it removes the cause (M1: a T limit at a floor, against a record whose rate does not depend on T), not a symptom; it is the only option
that is both the fastest (P6 0.465 ms a collapse step against 2,951.9, ×6,300; the life's step 2,975.9 → 30.2 ms, ×98; the iron block
×90,000) and **more correct than today** (M2: within 6 × 10⁻⁸ of the law, where the shipped step is 8 × slow and Δt-dependent); the whole life
then never takes a frame over 77 ms (M8), against «1 frame every 30 seconds». (d) is generic but slower (×24 in P6), still inexact and still
leaves a 32,775-sub-step cell; (c) makes the physics wrong in the other direction; (a) keeps a wrong rate. It is a physics change, not a pure
speed-up: the neutronization runs at §1.6.1's rate for the first time, so it is ruled as a §0.5 line and the lives that read it re-measured.

What would falsify it: (1) on the built twin or GPU, the iron block's X_n off the closed form by more than 10⁻⁶ at any of 20 steps; (2) P6
over 1 ms a step on the 20 M₀ life's 1,561–1,590 (from the same cut) on the idle RTX 4090, median of 3; (3) a scope among burn_cell,
burn_order, neutrino, latch, collapse, cpu_gpu red other than the one neutrino case M7 names; (4) a collapsing cell where N_Fe coexists with a
burning record at T ≥ T_thr and the split moves G-ORDER or G-BURN (the iron block and the life showed none: M4's other cells took one
sub-step).

**The §0.5 line** (contract, when the lead picks (b)):
`[M0-R10] §2.5.2's sub-cycling and §1.7's reactions row read for N_Fe · N_Fe (f ≡ 1, a = 0, one input of order 1 into n, the gate p = 1) is
stepped in closed form over the pass's Δt before the other records sub-cycle: Y_e − b = d E/(1 + d (1 − E)/(b − a_Y)), b = Σ_N/Σ, a_Y the Y_e
of the species other than iron, d = Y_e − b, E = e^{−k(b − a_Y)Δt}, k = A Σ^a Σ/Σ_N ((1 − E)/(b − a_Y) → kΔt as b → a_Y); X_Fe falls by ΔY_e/y_Fe,
X_n gains it, the heat q·ΔX_Fe enters ε and nuclear.N at once; a gap under f32's resolution lands the cell at Y_eΣ ≤ Σ_N; the collapse latch
reads N_Fe's rate at the pass's start; twin and GPU; burn_dt and burn_dx unchanged for the other records — measured on the RTX 4090 (M0-R10):
the shipped sub-cycles spent MAX_ATTEMPTS at T_floor, 8.4 % of Δt burned, P6 2,951.9 ms a collapse step.`

**The D blocks** (filed after this block when the lead picks (b); two, as R13 and Considered and REJECTED keep a pass's twin and GPU apart):
- **D-twin** — `Deliver:` sr_physics::reference::react steps N_Fe in closed form per the §0.5 line before the sub-cycles; burn_cell gains a
  case: the iron block (16 × 16 Fe, Σ 400, T 1) as one cell, X_n within 10⁻⁹ of the closed form at each of 20 steps of Δt 5.87 × 10⁻³ and
  the heat booked = q ΔX within 10⁻¹²; ΣX = 1, X ≥ 0; a cell at the gate after 10⁶ t.u. has Y_eΣ ≤ Σ_N; plant `nfe-sub-cycled` (the step
  skipped: red on the closed-form case). `Verify:` `verify.py burn_cell burn_order` GO, `--redarm burn_cell`.
- **D-GPU** — `Deliver:` react.wgsl the same step (f32, the gap landing), its counters unchanged; burn_cell's GPU twin of the case within
  10⁻⁶; neutrino's `gpu_latch_is_set_by_a_rate_that_closes_within_the_step` reads "at the gate" (Y_eΣ ≤ Σ_N) and its second pass sets nothing;
  plant `nfe-gpu-sub-cycled`. `Verify:` `verify.py burn_cell burn_order neutrino latch collapse cpu_gpu` GO on laserax-ai-server, RTX 4090;
  `--redarm` of the scopes it changes; the 20 M₀ life cut at 1,560 then `headless --load-dump … --continue --steps 1590 --timing`: P6 ≤ 1 ms
  a step, median of 3, idle 4090; the lives M0-R6 and M0-D81 read re-run on it (the lead's, over the line).

## UNVERIFIED

- The Zingale et al. sentences on Castro's energy limiter and retries are from the search engine's excerpts, not read in the PDF.
- Hochbruck & Ostermann: read from the abstract page only; the closed form itself is my derivation, checked numerically (M2), not a source's.
- Option (b) is measured on the GPU only (the twin untouched); its G-REF agreement is expected by construction, NOT measured.
- The shipped life on today's binary past step 1,700, and so (b)'s whole-life gain and ending against it: NOT RUN — over the 10-minute line.
  The lead's run, on laserax-ai-server (the shipped copy, ~hours):
  `[NOT RUN — for you]`
  ```bash
  cd /tmp/r10s && ./build/target/release/sandbox-reactions calibrate --adapter "RTX 4090" --out /tmp/r10s/cal_full.json --only m_bh_sb --bracket 67020.643,67087.664 --progress-every 1000 --checkpoint-every 2000
  ```
- m_up_sb unmoved under (b): expected (M5's event at the same step), NOT measured.
- (d)'s 32,775-sub-step cell: where it sat and why its T stayed above the floor's band — not probed.
- RTX 5090 and Quadro numbers: none (R18: every GPU run here on the 4090).

## The lead's answer

The lead, 2026-10-22, M0-R10's question (one question, four options — (b) exact N_Fe, (d) floor-aware limit, (a) leave it, (c) cap N_Fe
sub-steps — with the measured times above): **«(b) Exact N_Fe (Recommended)»**. Filed after this block: M0-D86 (the twin, the §0.5 line) and
M0-D87 (the GPU, the measurement), before M0-D81.
