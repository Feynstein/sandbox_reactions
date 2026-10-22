# M0-R9 — Heat-solve research: a smaller Δt on hard steps

Box: every GPU number below was measured on **laserax-ai-server, NVIDIA RTX 4090** (driver 595.99.02, Vulkan), R18. Before each set,
`nvidia-smi` listed no compute process on the card, and the load was 0.16–1.1. The work ran in scratch copies on the server under
`/tmp/r9/`: `tools/pb/scratch_copy.sh` of `~/workspace/sandbox_reactions` at 03e763b (the tree's HEAD, M0-D85's warm start and the
V-cycle in it), plus this task's probe patches. Nothing in the tree changed. Every probe line, timing line and run is in
`logs/M0-R9.log`. model=claude-opus-5-5 level=high.

**The short answer.**
- **A smaller Δt is not a speed-up on any stretch measured.** Per unit of sim time:
  - on M0-R7's hard stretch, Δt/2 for the whole step ties (+1.0 %), Δt/4 costs +51 %, and P5 sub-cycled costs +31 % to +174 %;
  - on today's only ceiling episode, every Δt option costs +39 % to +631 %;
  - on G-STIFF's two scenes, Δt/2 costs +39 % and +47 %.
- **The "seconds a step" symptom is gone from P5 on today's code.** The 5,257 capped steps were the RTX 5090 record's, on Jacobi before
  M0-D84. Today the 20 M₀ life to step 1,400 reaches P5's ceiling on **4 steps** (553, 554, 555, 557). Its hardest steps cost 28–107 ms on
  the 4090, 94 % of it in P5 (M1, M2). The collapse step that costs ~3 s is P6's, which is M0-R10's question.
- **What the ceiling costs is accuracy, not time.** Step 553 commits an iterate whose closure δ is 0.112. It sits 10 % (C-weighted L1)
  from the same step solved to convergence, and over the next 10 steps the capped life radiates 3× less light (M3).
- **But that episode is not Δt-converged under any option.** Its light per t.u. moves 3–14× between Δt, Δt/2 and Δt/8 (M8). A smaller
  Δt changes the answer; it does not just reach the same answer faster.
- **Recommendation: (a), Δt as it is — no build.**

## Evidence
- SUNDIALS, "CVODE: Mathematical Considerations", https://sundials.readthedocs.io/en/latest/cvode/Mathematics_link.html (accessed
  2026-10-22).
  - This is the reference stiff-ODE integrator's policy when Newton fails to converge within a step: «If convergence fails with J or P
    current, we are forced to reduce the step size», by η_cf = 0.25 by default, with «at most 3 iterations» a step.
  - «The integration is halted after a preset number of convergence failures; the default value of this limit is 10».
  - So the standard stiff integrator **cuts h and retries the step** when the nonlinear solve fails. It never commits an unconverged
    iterate, which is what our ceiling does ([M0-TJ4] (6)).
  - It caps Newton at 3 iterations, against our 64. There a failure is cheap and the retry is the norm; P5 instead pays up to 64 passes
    before it gives up.
- PETSc, `TSSetMaxSNESFailures`, https://petsc.org/release/manualpages/TS/TSSetMaxSNESFailures/ (accessed 2026-10-22): «Sets the maximum
  number of failed SNES solves allowed before TSSolve() is ended with a TSConvergedReason of TS_DIVERGED_NONLINEAR_SOLVE».
  - The other major library counts nonlinear-solve failures as rejected steps under a budget, the same reject-and-retry design.
  - Its TS manual (https://petsc.org/release/manual/ts/, accessed 2026-10-22) describes the error-based rejection: «otherwise, the step
    is rejected and retaken with the step length computed in» the adaptor.
- Wikipedia, "Conjugate gradient method", § Convergence properties — Convergence theorem,
  https://en.wikipedia.org/wiki/Conjugate_gradient_method (accessed 2026-10-22): «k = ½√κ(A) log(‖e₀‖_A ε⁻¹) iterations suffices to reduce
  the error to 2ε».
  - In P5's system, the diagonal is c_i + Σ D_f with c_i ∝ 1/Δt. So κ grows roughly with Δt × the stiffness D/(C Δx²), and plain CG's
    rounds grow with √(Δt × the stiffness). Halving Δt saves at most a factor √2 in rounds a step, while doubling the steps.
  - The V-cycle (M0-D82) flattens κ further. That is why M4's rounds a step fall only 1.3–2.5× per halving and never pay for the extra
    steps, except once.
- Glenn Fiedler, "Fix Your Timestep!", Gaffer on Games, https://gafferongames.com/post/fix_your_timestep/ (accessed 2026-10-22).
  - This is the standard game-loop pattern. «the renderer produces time and the simulation consumes it in discrete dt sized steps».
  - Its failure mode is the «spiral of death … what happens when your physics simulation can't keep up with the steps it's asked to
    take».
  - Its remedy: «clamp at a maximum # of steps per-frame and the simulation will appear to slow down under heavy load».
  - That is already our design: N_max per frame and the latch (§1.8.3–§1.8.5), and Max as fast as the card goes ([M0-TJ2]).
  - A smaller Δt on hard steps feeds the spiral. It makes more steps per sim second exactly when each step is dearest.
- What the sources share, and where P5 differs:
  - Library integrators cut Δt for **accuracy and robustness** (the step was rejected), not for speed.
  - The game pattern keeps the frame budget by **slowing the sim clock**, not by shortening steps.
  - None of them promises that more, shorter implicit steps cost less in total. M6 measures that they do not, here.

## Problem
**What P5 does at its ceiling, read from the code** (`crates/sr-engine/shaders/heat.wgsl`, `crates/sr-engine/src/step/heat.rs`):
- A step's Newton passes stop at `MAX_PASSES` 64 or `MAX_ROUNDS` 4,096 ([M0-D37]).
- The last iterate is committed with (5)'s closure, which rescales every gas cell's heat by (1 + δ) so the step's balance closes.
- The step is logged (`SR-WARN heat unconverged`, counted by calibrate, M0-D74).
- Δt is P9's CFL and gravity reduction alone (§2.3.5: «Burning and heat never limit Δt»).

The probes (scratch only; `r9_patch.py`, `r9_patch2.py`, in `/tmp/r9/`):
- **`SR_PROBE_DT_DIV=n`**: `dt_advance` takes Δt_next / n. Every pass of every step runs at Δt/n, which is option (b) at its strongest
  (on every step). n = 1 is today's binary: it reproduces the shipped life to the bit (sim_time 9.250446 at step 1,400 = M0-D85's).
- **`SR_PROBE_SUB=n`**: P5's whole list (`heat_cells` … `heat_apply`, the slots, `heat_park`, `latch_carry`) is recorded n times in the
  step, with `heat_open` reading Δt_n / n; P0–P4 and P6–P9 run once. This is option (c).
  - It is valid only without a carry, so the slots are fixed (`SR_PROBE_ROUNDS`); no run carried.
  - `side.flux` is the last sub-step's F. A build would average it.
- **`SR_PROBE_HEAT`**: one record a step (passes, rounds, δ, Δt); `SR_PROBE_LOGALL` logs every sub-step.
- **`ceil`**: a build with the ceiling at 1,024 passes and 32,768 rounds, every tolerance unchanged. It gives the converged answer of a
  step that hits today's ceiling. No step needed more than 1,502 rounds, so a passes-only raise would behave the same.
- **`r9_tcmp`**: M0-D84's `tcmp` (C-weighted relative L1 of T, worst cell with C ≥ 10⁻⁴) plus the heat ratio and the cells off by more
  than 10 %.

The stretches:
- **h, hard**: M0-R7's 20 M₀ dump at step 1,400 (`/tmp/d84/r7_s_1400.bin`), stepped once on today's binary for its rate plane, then steps
  1,402–1,421. This is the stretch M0-D84 and M0-R8 graded on.
- **e, the ceiling episode**: today's life (`calibrate --only m_bh_sb --bracket 67020.643,67087.664`) checkpointed at step 552, then
  steps 553–562.
- **rim, hot**: G-STIFF's `scenes/heat_rim.json` and `scenes/heat_hot.json`: 64 × 64, heat only, 20 steps from the scene.
- Scene for h and e: `life_scene.json` (600 × 400, leave edges, every switch on).

**M1** — P5's share of the step today (median of 3 `headless --timing`, (a)):

| stretch | P5 (ms/step) | step (ms) | P5's share |
|---|---|---|---|
| h, hard (1,402–1,421) | 28.040 | 29.838 | 94 % |
| e, ceiling episode (553–562) | 101.225 | 107.390 | 94 % |
| heat_rim | 4.427 | 4.503 | 98 % |
| heat_hot | 3.215 | 3.290 | 98 % |

- The 20 M₀ life to step 1,400 on today's binary: **14.929 s** wall (14.929, 14.989, 14.860).
- The episode's 10 steps take 1.07 s of it (7 %).
- The 4 ceiling steps carry 2,461 of the episode's 3,316 rounds, ≈ 0.75 s, ≈ 5 % of the life. This is derived in proportion to rounds,
  not timed step by step.

**M2** — Where P5's ceiling binds today (`SR_PROBE_HEAT`, the life to 1,400):
- Steps 553, 554, 555 and 557, every one at **64 passes** (not rounds): 1,326, 506, 297 and 332 rounds.
- Their closures δ are 0.112, 8.95 × 10⁻⁴, 1.30 × 10⁻⁵ and 2.29 × 10⁻⁵.
- Steps 556 and 558 converge in 47 and 45 passes. Every other step of the 1,400 converged.
- On the `ceil` build, step 553 converges in **78 passes**, 1,502 rounds (δ 1.6 × 10⁻⁸). The episode's later steps then need 19–22
  passes, against 45–64 on today's.
- The pass count is the front-per-pass mechanism M0-D37 measured: «each Newton pass moves it about one cell». A front crossing more cells
  in one step needs more passes. It is not the clamp's ⌈log₂⌉ alone (M0-R8 M5): no cell changes 2⁶¹× in a step.

**M3** — What the ceiling-hit step leaves unconverged: step 553, its committed T against the same step solved by `ceil` (`r9_tcmp`, the
dumps after it):
- C-weighted L1 **0.1005**, the worst cell 693× off, 99,719 of 180,180 gas cells off by more than 10 %.
- Σ C T converged / committed = **0.8996**. The closure's (1 + δ) put back, uniformly, heat the solve should have radiated or moved.
- Over steps 553–562, the ledger's `radiated`:
  - converged at Δt: 1.253 × 10⁶;
  - today, capped: 0.413 × 10⁶ — **3.0× less light**.
- The episode is violent. Thermal energy falls from 1.07 × 10⁷ to 2.1 × 10⁶ and kinetic rises from 3.8 × 10⁶ to 1.1 × 10⁷ in 10 steps
  (stage `cloud`, raw `iron_core`).

**M4** — (b) Δt/n on every pass, passes and rounds a step (`SR_PROBE_HEAT`, every step converged unless named):

| stretch | n = 1 (passes / rounds a step) | n = 2 | n = 4 | n = 8 |
|---|---|---|---|---|
| h, hard | 12.45 / 98.2 (sum 249 / 1,963) | 9.40 / 38.6 (376 / 1,545) | 7.99 / 25.9 (639 / 2,072) | 6.67 / 19.8 (1,068 / 3,161) |
| e, episode | 37.0 / 331.6, **4 at the ceiling**, max 64 | 11.9 / 182.8, max 64 (converged on it) | 7.25 / 193.2, max 33 | 5.08 / 116.6, max 23 |
| heat_rim | 5.90 / 16.9 | 5.40 / 10.1 | 3.94 / 5.8 | 2.73 / 3.0 |
| heat_hot | 3.35 / 12.1 | 2.70 / 7.5 | 2.31 / 4.6 | 2.15 / 2.8 |

- On the hard stretch, halving Δt cuts the passes a step by only 25 %, and the rounds by 61 % once.
- Over the same sim time, the passes rise at every n, and the rounds rise from Δt/4.
- On the episode, the flow's own Δt shrinks as the stretch is refined: the sim time covered is 0.0591, 0.0539, 0.0421 and 0.0411 t.u.
  The trajectories part.

**M5** — (c) P5 sub-cycled n times inside the step (`SR_PROBE_SUB`, totals over the stretch's steps):

| stretch | n = 1 (passes / rounds) | n = 2 | n = 4 | n = 8 |
|---|---|---|---|---|
| h, hard (20 steps) | 249 / 1,963 | 385 / 2,415 | 650 / 3,256 | 1,125 / 4,458 |
| e, episode (10 steps) | 370 / 3,316, 4 at the ceiling | 164 / 4,054, **1 at the ceiling** (553's first half-step: 64 passes, 1,473 rounds) | 181 / 2,709 | 290 / 2,769 |

- A heat-only stand-in for (c) misleads, which is why (c) was built in the step:
  - The proxy was P5 alone, run from each full step's start state (no P2–P4).
  - From the state before step 554 it converges in **19** passes. The full step from the same state hits the ceiling, because P4's
    compression and push set P5's start state within the step.
  - The proxy's runs are logged (`HEAT …`) and not used.

**M6** — The cost per sim time: median of 3 `headless --timing` runs, interleaved, GPU timestamps; load 0.53–1.1, no other process on the
4090. Each run covers the stretch's sim time at Δt/n.
- **(b)** runs today's binary at Δt/n, with the engine's own slots.
- **(c)** runs on fixed slots, R = even(1.25 × the run's most slots + 4), as the engine would learn.
  - Fixing the slots moves (a) by +0.05 % (h) and +0.7 % (e): 5,705 against 5,702, and 18,286 against 18,160.
- **(d)** is the `ceil` build at Δt.

| ms per t.u. (P5 ms/step) | h, hard | e, episode | heat_rim | heat_hot |
|---|---|---|---|---|
| (a) Δt | **5,702** (28.04) | **18,160** (101.2) | **362** (4.43) | **684** (3.22) |
| (b) Δt/2 | 5,758 (13.19), +1.0 % | 25,646 (63.8), +41 % | 531 (3.27), +47 % | 951 (2.37), +39 % |
| (b) Δt/4 | 8,587 (9.34), +51 % | 67,676 (64.4), +273 % | 755 (2.32), +109 % | 1,441 (1.85), +111 % |
| (b) Δt/8 | 14,161 (7.38), +148 % | 132,681 (65.0), +631 % | 1,100 (1.68), +204 % | 2,354 (1.53), +244 % |
| (c) P5 × 2 at Δt/2 | 7,476 (36.95), +31 % | 30,002 (137.2), +65 % | = (b) | = (b) |
| (c) P5 × 4 | 10,603 (53.19), +86 % | 25,190 (114.2), +39 % | = (b) | = (b) |
| (c) P5 × 8 | 15,643 (79.35), +174 % | 51,791 (253.1), +185 % | = (b) | = (b) |
| (d) the ceiling raised, Δt | 5,754 (28.32), +0.8 % | **13,390** (78.9), **−26 %** | — | — |

(For (d), the reference is the same-session (a): h 5,707, e 18,173.) On rim and hot every pass is P5, so (c) is (b).

**M7** — The whole life, 20 M₀ to step 1,400 (`calibrate … --max-steps 1400 --checkpoint-every 1400`, 3 runs a binary interleaved):
- today: **14.929 s** (14.929, 14.989, 14.860), 4 steps at the ceiling, sim_time 9.250446;
- (d) `ceil`: **16.634 s** (16.648, 16.574, 16.634), **+11.4 %**, none at the ceiling, sim_time 9.273883.
- The two lives part at step 553. (d)'s −26 % on the episode does not carry to the life: the converged life after 553 is not the capped
  one, and costs more.
- (b) and (c) were not run on the whole life (UNVERIFIED). Their measured cost is above (a)'s on every stretch, so they cannot be cheaper
  there unless the life parts into easier steps, as (d)'s did not.

**M8** — Do the answers move? Light per t.u. over each run's ledger window, and T after step 553:

| | h, hard (radiated / t.u.) | e, episode (radiated / t.u.) | heat_hot (radiated / t.u.) | heat_rim (radiated / t.u.) |
|---|---|---|---|---|
| (a) Δt | 1.165 × 10⁶ | 7.52 × 10⁶ (capped) | 3.69 × 10⁹ | 3.15 × 10⁵ |
| (d) Δt, converged | = (a) | 2.12 × 10⁷ | — | — |
| (b) Δt/2 · /4 · /8 | 1.162 · 1.158 · 1.156 × 10⁶ (−0.8 %) | 4.32 × 10⁷ · 8.27 × 10⁷ · 1.03 × 10⁸ | 4.11 · 4.69 · 5.41 × 10⁹ (+47 %) | 3.42 · 3.63 · 3.80 × 10⁵ (+20 %) |
| (c) P5 × 2 · × 4 · × 8 | 1.142 · 1.124 · 1.114 × 10⁶ (−4.4 %) | 3.51 · 3.93 · 3.40 × 10⁷ | = (b) | = (b) |

- **The hard stretch is converged in Δt to 1–4 %.** There a smaller Δt only costs.
- **The stiff scenes are not**, as G-STIFF (e) states: backward Euler at Δt keeps more heat than its refinement, by up to 2.2×. heat_hot
  radiates 47 % more at Δt/8.
- **The episode is not converged in any option.**
  - Its light moves 3× from capped to converged at Δt, a further 1.6–1.9× to (c), and 14× to (b) Δt/8, whose hydro also refines.
  - After step 553, (c)'s T sits 0.098 / 0.134 / 0.264 (n = 2 / 4 / 8) from the converged Δt step, against the capped step's 0.112.
  - **The ceiling's error is the size of the time-discretisation error itself.** Removing only the ceiling, by (b), (c) or (d), does not
    make the episode right.
- Every option **changes the life's answers**. None is a pure speed-up: the heat scopes heat_implicit, heat_stiff and radiation_force,
  and every calibrated value M0-V9 reads, would be re-read on any build.

## Options
Times are M6's and M7's on the RTX 4090. "Moves" lists what each changes beyond the time.

| option | hard stretch | ceiling episode | stiff scenes | the life to 1,400 | moves | effort |
|---|---|---|---|---|---|---|
| **(a) Δt as it is** — the ceiling, a step carried | 5,702 ms/t.u. | 18,160 ms/t.u., 4 steps capped (δ to 0.112, 3× less light) | 362 / 684 | **14.93 s** | nothing | none |
| **(b) a heat Δt limit when the last step hit the ceiling** (§2.3.5 gains a term; the step's other passes follow) | at Δt/2: 5,758 (+1.0 %); at /4: +51 % | at Δt/2: 25,646 (+41 %), none capped; but step 553 is still committed capped (the limit acts on the step after) | +39 % / +47 % at Δt/2 | not run; ≥ (a) by M6 | Δt's record and the dumps' `dt_next`; the sim clock; [M0-D80]'s trigger: a life that no longer hits the ceiling no longer frames one step at a time; §2.3.5's «heat never limits Δt»; every calibrated value | 1 BUILD (P9's term + G-STIFF/G-WARP cases + a plant) + re-calibration |
| **(c) P5 sub-cycled inside a step** (n heat sub-steps of Δt/n, the rest once) | +31 % to +174 % | n = 2: +65 %, still 1 capped; n = 4: +39 %, none | = (b) | not run; ≥ (a) by M6 | the step's dispatch list ×n; F's average for P4 (a design point); the round machine's carry, which today restarts a step and would have to resume a sub-step (the probe could not carry); the rate plane per sub-step; every calibrated value | 2 BUILD (the sub-step loop with its carry; the scopes, plants and the docs) + re-calibration |
| **(d) the pass ceiling raised** (64 → 256, rounds 4,096 unchanged; not a Δt option, measured because M2 shows the hits are pass-bound) | +0.8 % | **−26 %**, none capped | — | **16.63 s (+11.4 %)**: the converged life parts at 553 and costs more after | [M0-D37]'s ceiling; the life's trajectory; every calibrated value | 1 BUILD (two constants, shader and twin; SLOTS; a scope case) + re-calibration |

## Recommendation
**(a): keep Δt as it is. No build, no §0.5 line, no D.**

Why:
- The lead's ask was speed: «so that our testing here doesnt take ages», «if a black hole collapse runs at 1 frame every 30 seconds its
  kind of an issue». No smaller-Δt option is faster on any stretch measured:
  - (b) ties at Δt/2 on the hard stretch and costs +39 % to +631 % elsewhere;
  - (c) costs +31 % to +185 % everywhere (M6).
- The "seconds a step" symptom it was asked against is gone from P5:
  - M0-D84's V-cycle and M0-D85's warm start leave 4 ceiling steps in the life's 1,400, ≈ 0.75 s of 14.9 s.
  - The hard stretch runs 28 ms a step (M1–M2).
  - The ~3 s collapse step is P6's (M0-R10).
- Its accuracy case is real, but none of the options settles it. The ceiling-hit step is 10 % off, with 3× less light (M3), yet the episode
  is not converged in Δt under any option (M8). Buying the ceiling away costs time and still leaves the answer Δt-dependent. That is a
  question of the physics' time resolution (CFL 0.4 through a violent episode), not of P5's solver. It is named below and not filed.

What would falsify it:
- A stretch where the ceiling binds on many consecutive steps — the 5090 record's 5,257 capped steps after its supernova, which this 4090
  life does not reach (M0-D81's split) — on which (b) or (d) measures **faster** per t.u. than (a), median of 3 on the RTX 4090.
- Or the lead ruling the episode's accuracy (its light, its ending) worth the cost. Then (d) is the cheapest fix of the ceiling alone
  (+11 % on the life), and a Δt-convergence study of the episode's physics comes before (b) or (c).

**If the lead picks (b) instead** — one BUILD block, filed after M0-R9, before M0-V9 (the calibration it moves):
- §0.5 line:
  > [M0-R9] §2.3.5's «Burning and heat never limit Δt» · P9's Δt_{n+1} gains a heat term: when step n's P5 ended at the ceiling
  > ([M0-TJ4] (6)), Δt_{n+1} = min(the CFL and gravity Δt, Δt_n / 2), and after a step that converged within 32 passes the term relaxes
  > ×2 a step until it no longer binds; G-WARP holds (the term is the state's) · measured (reports/heat_hard_dt.md M4–M6) …
- `Deliver:` the term in `reduce.wgsl`'s `dt_partials` (P5's last report read on the GPU), its twin in sr-physics, `dt_next` in the dump
  unchanged in meaning, contract §0.5 and §2.3.5 amended, docs/agent/testing.md's rows.
- `Verify:`
  - the 20 M₀ life to 1,400 on the RTX 4090: 0 steps at the ceiling after step 553; wall ≤ 1.10 × 14.93 s, median of 3;
  - heat_implicit, heat_stiff, heat_limiter, heat_gauss, radiation_force, heat_multigrid and warp GO;
  - plant `heat-dt-never-relaxes` (the term never relaxes): red on a new case, a ceiling step followed by easy ones whose Δt returns to
    the CFL's within 8 steps.

**If the lead picks (d)** — one BUILD block, same place:
- §0.5 line: [M0-R9] [M0-D37]'s «64 passes or 4,096 rounds» · 256 passes or 4,096 rounds, GPU and twin · measured (M2, M6, M7).
- `Deliver:` `MAX_PASSES` 256 in heat.wgsl and `sr_physics::reference::heat`, `SLOTS` following, contract §0.5.
- `Verify:`
  - the life to 1,400 has 0 steps at the ceiling, wall reported against 14.93 s (+11 % expected);
  - the heat scopes GO;
  - plant `ceiling-64` (the old constant) red on a new case, step 553 from the life's checkpoint at 552, converged within 256 passes.

## UNVERIFIED
- **One life, one card.** Every figure is the 20 M₀ life on the RTX 4090 (and G-STIFF's scenes). Not measured:
  - the RTX 5090 record's post-supernova stretch (5,257 capped steps on Jacobi), which this life does not reach by step 1,600 (M0-D81);
  - the Sun-like life (sun_disk exits 6: `preset_mass_sb` missing, M0-V9);
  - the Quadro RTX 4000 (the game's card).
- **(b) as an adaptive rule was not built.** M4–M6 run Δt/n on every step of a stretch. The rule «after a ceiling step» would apply it to a
  few steps, and still commit the first hit capped.
- **(b) and (c) on the whole life were not run.** M7 holds (a) and (d) only.
- **(c)'s F for P4** is the last sub-step's in the probe, not the sub-steps' average a build would push with. Its slots are fixed (no carry).
  The fixing moves (a) ≤ 0.7 %, but (c)'s easy steps after a hard one may record more slots than the engine's learnt count would.
- **Why the episode is under-resolved in time** (which cells, which pass sets its light) was not mapped. Its 3–14× spread in light per
  t.u. across Δt is measured, not explained.
- **The four ceiling steps' share of the life's wall** (≈ 0.75 s) is derived from their share of the episode's rounds, not timed one by
  one.

## The lead's answer
2026-10-22, the question tool (PLAYBOOK §12). The options were (a) recommended, (d), (b) and (c), each with M6–M7's measured times on the
RTX 4090. The question named the 4 ceiling steps' 10 % error and 3× less light, and that no option makes the episode Δt-converged. The
lead picked: «(a) Keep Δt, no build (Recommended)». No D is filed, and no §0.5 line is written.
