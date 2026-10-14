# M0-TJ4 — P5's implicit solver: the spec, its evidence and the lead's three answers (detail)

Ran on: model=claude-opus-5-5 level=max (`rung_record.py now`, plugin=installed — the heading's rung). Box: linux-pc
(laserax-ai). No GPU run: every number below is from scratch models on the CPU (NOT PROVEN — synthetic), raw output in
logs/M0-TJ4.scratch.log, the box at load ~120 from other work (counts and errors only, never timings).

## The rulings (the lead, 2026-10-14, question tool, one call, three questions)

1. Heat in fast changes — «Re-solve heat, light law fixed (Recommended)»: Newton on the heat capacity, the light-flow law
   (λ, κΣ) taken at the step's start. Options declined: «Re-solve both» (λ iterated too), «One solve (today's way)».
2. Heat's cost ceiling on the Quadro — «≤ 0.5 ms per step (Recommended)». Declined: «≤ 0.3 ms per step», «≤ 1.0 ms per
   step».
3. Hard steps — «Speed gives first (Recommended)»: a hard step carries over into the next frames, physics exact, 60 fps
   held, `running ×` shown; a last safety limit (4,096 rounds) warns. Declined: «Accuracy gives first».

Recorded as contract §0.5's three [M0-TJ4] lines and §5.1's G-STIFF row; Superseded mirrors in the plan.

## The scheme and why (contract §0.5 [M0-TJ4] (1)–(7))

- **Backward Euler, not a second-order form.** L-stable (stiff modes damped, never ringing) and an M-matrix (a maximum
  principle: no negative or overshooting T). P5 sits in a first-order operator split (P4, then P5, then P6), so a
  second-order P5 alone would not make the step second order; TR-BDF2 would double the solves. BE's own error measured
  first order on G-HEAT2's pulse (ratios 1.93–1.97 when Δt halves) and on the warm side in a stiff cooling (it cools no
  faster than its refinement).
- **u = a_r T⁴ as the unknown.** The twin's face law K_f (T_b − T_a) with K_f = c_sb λ a_r (T_a + T_b)(T_a² + T_b²)/(κΣ)_f
  is exactly −c_sb λ/(κΣ)_f · (u_b − u_a): linear in u, the T³ gone from the coefficient. Freezing K (today's RKL2, and a
  "lagged" BE) freezes that T³: on a ref_burn-like burnt field the step keeps 0.5 % of the heat where 256 refined
  sub-steps keep 16 % (~30× too much cooling). In u the only nonlinearity left is the cell's heat C T(u), diagonal and
  monotone, so Newton on it keeps every pass's system symmetric positive-definite (CG applies) and converges in 4–7 passes
  even there (the factor-2 damping guarding the first passes); one pass of the converged step keeps 30 % (16 % refined —
  BE's own one-step error).
- **λ frozen at the pass's start (the lead's answer 1).** Iterating λ too (Picard) changed the heat by ≤ 3 × 10⁻⁵ of the
  pass's but took 86–987 rounds instead of 9 around a thin star edge, erratically (19–32 passes at 10⁻⁵). Plain Picard on
  the T-form K (T³ included) does not converge at all on the burnt field (a 2-cycle). The cost of freezing λ: in cold thin
  gas heated within a pass the solved flux can exceed the free-streaming bound (850 × c_sb a_r T⁴ at heat_spot's cold front
  on the first pass, 3 × 10⁻⁸ of the spot's heat) — RKL2 moved that same energy and reported its start flux, which is what
  G-FLD bounded; G-FLD is re-read to say so ([M0-TJ4] G-FLD line).
- **Jacobi-preconditioned CG, one dispatch a round.** Stronger preconditioners cut rounds 2–4× (block symmetric
  Gauss–Seidel on 16 × 16 tiles with tile deflation: 9/27/25/12/21 rounds against Jacobi's 17/83/109/25/48 on the five
  linear cases), but each round then needs ~4 dependent dispatches (local solve, SpMV, a one-workgroup coarse solve,
  update) against Jacobi's one (every workgroup reducing the previous round's partial sums in index order — fixed order,
  §1.3.4). The Quadro is dispatch-bound (2.65 µs a dispatch, M0-R3 M1; M0-T42's 39-dispatch P5 0.19–0.26 ms), so Jacobi
  wins or ties in every case measured; deflation stays the upgrade if large stiff regions (N ≫ 64 cells) cost too much —
  M0-T122's report says.
- **The stop rules.** Inner: both (r, D⁻¹r) ≤ (10⁻⁵)² Σ c_i u_i² (the error in u) and Σ (C_i/Δt) ê_i² ≤ (10⁻⁵)² Σ (C_i/Δt)
  T_i² (each cell's heat error by its Jacobi estimate ê_i = r_i/(4 a_r T_i³ D_ii)). The first alone weighs a cell by ~C T⁵
  (c_i = C_i/(4 a_r T_i³ Δt) times u_i² = a_r² T_i⁸): it stopped heat_spot's first pass after 10 rounds with 11 % of the heat
  that pass moved wrong at the cold front (found while re-reading the spec; the contract had it alone for an hour of this
  session); the second alone is too loose on the burnt field (1.5 × 10⁻⁴ of the heat at 124 rounds, over G-STIFF (d)'s
  10⁻⁴). Together, at 10⁻⁵: typical 35 rounds (7 × 10⁻⁵ of the heat moved), heat_spot 16 (1.8 × 10⁻³), the burnt field 361
  (7.5 × 10⁻⁷ of the heat), the ring 320 (3.4 × 10⁻⁷). Measured against the stiffness-weighted scale (Σ D T⁰²) the same ε
  left 6.6 × 10⁻⁴–2 × 10⁻³ of the heat wrong on the burnt field (it drains 99.7 % of its heat, D ≫ C/Δt). 10⁻⁶ in a
  T-form test was past f32 on the stiffest field: both preconditioners stalled, then diverged (3,000 rounds, garbage) —
  10⁻⁵ it is. Outer: Σ C|ΔT| ≤ 10⁻⁶ Σ C T⁰ and every |ΔT| ≤ 10⁻³ T — 2–5 passes in every case. A per-cell inner rule (|r_i| ≤ 10⁻⁵ D_i u_i) cost 9 → 81–83 rounds on thin rims and still left
  wide-rim cells 34 % off: not adopted — thin cells (Σ ≈ 2 × 10⁻⁶) hold ~10⁻⁵ of the heat; their T is graded by heat (G-STIFF
  (d) reads the C-weighted L1, a max only where Σ ≥ 10⁻⁴).
- **The commit.** Solution form (E from T(u)), not flux form: a flux-form T in a cell of C ~ 10⁻⁶ amplifies the residual by
  the stiffness ratio (10⁶–10⁷), past f32. The heat balance is closed by δ (every gas cell's T scaled by 1 + δ): exact to
  f32's sums, positivity kept, δ ~ the solve's tolerance when converged; tile deflation would close it per tile but needs an
  exact coarse solve every round (the dispatch cost above). F is the solved flux — the energy the step moved, so P4's push
  matches it (a thin rim emptying its heat in a fraction of Δt no longer pushes with its start flux).
- **No history.** The first iterate is the pass's start state, so a dump resumes bit-identical on [M0-T26]'s keys (an
  extrapolated guess from the previous step would need a new header key).
- **Speed gives first (answer 3).** The rounds a submission records are the engine's choice and never change the result
  (the solve's state lives on the GPU); this lets the engine record only the rounds recent steps needed — zeroed indirect
  dispatches still cost something each, so recording the 4,096-round ceiling every step would cost more than the solve.

## The models (scratch, never in the repo — described so they can be rebuilt)

All in pure Python on the twin's own formulas (reference/heat.rs `conductance`, the vacuum state beyond a face, λ of
Levermore–Pomraning), grids 32–64 cells, f64; "f32" rounds every vector to f32 after each operation and sums dots in a
fixed-order f32 tree. be_iters/be_acc: the linear system with synthetic K and C (uniform, hot, star + thin rim) and nine
preconditioners. fld_be/fld_picard: heat_spot.json exactly (Σ 10⁻⁴, H, spot T 2 in T 0.05, leave edges), K lagged or
re-evaluated. hot_cool: a 32 × 32 burnt field (Σ 30, κ 1, 1/μ 0.5, T 18,800, leave, Δt 6.3 × 10⁻³), the reference 256 lagged
sub-steps. enewton/unl32: the u-form, Newton on C T(u), λ frozen or Picard, damping ×2. heat2_be: G-HEAT2's pulse,
reference explicit Δt 1/64. gstiff: G-STIFF's two scenes as the contract fixes them (He: 1/μ 0.75, κ 1; H: 1/μ 2, κ 2).

## G-STIFF's numbers (contract §5.1)

| Scene | RKL2 would need | one pass: Newton passes, f32 rounds | heat left: one pass / 64 refined | T range after |
|---|---|---|---|---|
| heat_hot 64 × 64 (He, Σ 30, T 18,800, Δt 0.02) | 580 stages | 7 (f64) · 5 passes, 361 rounds (f32) | 0.342 / 0.203 → 1.69 | [1,974, 8,108] |
| heat_hot 32 × 32 (the same) | 580 | 7 · 5, 198 | 0.252 / 0.129 → 1.95 | [1,930, 5,969] |
| heat_rim 64 × 64 (disk Σ 1 T 100 a 20, ring w 12 Σ 2 × 10⁻⁶ T 5, Δt 0.01) | 11,547 | 7 · 5, 320 | 0.993 / 0.991 → 1.00 | [64.9, 100] |

(e)'s band [1.0, 2.2]: the scheme reads 1.69; one damped Newton pass leaves T ≥ T⁰/2, heat ≥ 0.5 → ≥ 2.46 (red); the T³
frozen 0.03 (red). f32 lands within 3–8 × 10⁻⁷ of the f64 answer in C-weighted L1, ≤ 5.5 × 10⁻⁵ relative where Σ ≥ 10⁻⁴ —
(d)'s 10⁻⁴ and 10⁻³ hold with margin.

## The cost (contract §0.5 [M0-TJ4] (7)) — an estimate, M0-T122 measures

Inputs: a dependent dispatch 2.65 µs on the Quadro (M0-R3 M1); an active box-sized RKL2 stage ~9 µs and an idle one ~4.5
µs on the loaded Quadro (my reading of M0-T42's two means, 0.229 ms at s ≤ 4 and 0.360 ms at s ≥ 16); today's P5
0.188–0.261 ms of a 0.88–1.10 ms step (M0-T42, M0-T46); rounds per step from the models: ~35 (mild), 9–16 (thin edges and
heat_spot, λ frozen), 190–360 (the burnt field, and a ring heated 13× in one pass: the extremes). At ~6–9 µs a round plus ~0.06–0.1
ms of setup, passes and commit: ~0.3–0.5 ms in a mild step; 2–3 ms in an extreme one, carried over submissions. Frame
budget (M0-R3 § Problem 3): 13 ms of simulation in a 60-fps frame; today ~13 steps a frame (≈ 1.0 ms a step), ~10–14 with
the implicit P5 — M0-R3's smallest scenario (L) wanted 38: that gap is M0-T63's and M0-TJ2's.

## Blocks filed (placed between this block and M0-T48, whose order now reads AFTER M0-T122)

| Id | What | Rating (rule) | Sizing |
|---|---|---|---|
| M0-T117 | the twin's implicit step, its cases, `heat_rkl2` → `heat_implicit`, plant solve-two-rounds | Opus 5.5, high — (5) numeric/precision | M |
| M0-T118 | the GPU's solve and commit on the linear law (constant diffusivity) | Opus 5.5, high — (5) numeric + shared state | M |
| M0-T119 | the GPU's radiative law and Newton passes; RKL2 and the cap removed | Opus 5.5, high — (5) numeric | M |
| M0-T120 | G-STIFF: two scenes, twin and GPU cases, scope `heat_stiff`, plant newton-one-pass | Sonnet 5.5, high — (3) | M |
| M0-T121 | the carry-over across submissions, adaptive rounds, headless's warning | Opus 5.5, high — (5) shared state | M |
| M0-T122 | P5's cost on the Quadro against 0.5 ms → reports/p5_cost.md | Sonnet 5.5, high — (3) timing | — |

The GPU work is split by dependency (M0-T118's solve and commit are M0-T119's artifact; RKL2 serves the radiative scenes
in between, so the tree stays green), because one block would pass the ≤ 600-line ceiling (M0-T42's P5 ran ~1,110 lines).
Flags: M0-T48 (the new P5, the band unchanged, hard runaway steps), M0-T49 (physics.md and testing.md rows), M0-V6 (lot 6's
check of it), M0-T63 (the cost's report); rating refresh 0/5.

## Measured, not acted on

- M0-TM6 left "M0-T48 (E) now carries by its flags the twin driver's P5, F and P6 … split it or drop `Sizing: E`" for the
  next PLAN block. Not mine to resize (a second thread): T48's first try built the twin driver's P5 and P6, so its retry is
  a re-run on the new P5; its order changed only.
- The cost of a zero-workgroup indirect dispatch on the Quadro is unmeasured — it prices the rounds recorded but unused;
  M0-T121 and M0-T122 read it.
- Thin cells' T (Σ ≈ 2 × 10⁻⁶) in a wide stiff ring stays up to ~34–69 % off the scheme's exact answer under f32 at 10⁻⁵
  (their heat ~10⁻⁵ of the scene's): graded by heat only; a stronger preconditioner is the fix if it ever shows.

## Deviations

- Code written outside `Deliver:` — the scratch models above, analysis only, kept in the session scratchpad, never in the
  repo (as M0-R3's bench); their raw output in logs/M0-TJ4.scratch.log.
- M0-T48's heading and Flow row order (AFTER M0-T47 → AFTER M0-T122) and the Superseded mirrors edited by hand (no
  plan.py call for either).
