# M0-R7 — Heat-solve research: a multigrid preconditioner for P5's PCG

Box: every GPU number below is from **laserax-ai-server, NVIDIA RTX 4090** (driver 595.99.02, Vulkan), idle at each start (load 0.4–3.3; the
3.3 was from this task's own CPU model runs, never a GPU job; `nvidia-smi` showed 0 % and 15 MiB before each probe). The work ran in a scratch
copy, `/tmp/m0r7` on the server (`tools/pb/scratch_copy.sh` of `~/workspace/sandbox_reactions` at 40bdeac, the tree's HEAD), plus probe hooks
that are not in the tree (described below). The f64 solver model is `/tmp/r7model` on the server (source in the session scratchpad). Logs:
`logs/M0-R7.log` (the probes, every model run and the timing summaries) and `logs/M0-R7.bench.log` (the dispatch microbenchmark).
model=claude-opus-5-5 level=high.

**The short answer.**
- Multigrid cuts the rounds of the hard steps by about 25× (6,914 → 275). The 4090 runs these solves at the cost of each dependent
  dispatch, not of memory traffic, and a V-cycle round costs 6–10 Jacobi rounds. The net gain is therefore estimated at **1.4–2.7× on the
  hard steps against today's capped cost**, and 2.4–4.5× against a Jacobi solve run to the end.
- Every one of those steps also **converges**. Today each one stops at 4,096 rounds and commits a heat-balance closure δ of 6–8 × 10⁻³
  (M3).
- No preconditioner gives an order of magnitude here. A hard step is 13 Newton passes of a few hundred Jacobi rounds each, so the larger
  lever is the passes, which is M0-R8's and M0-R9's ground.

## Evidence
- Zhang, Howell, Almgren, Burrows & Bell, "CASTRO: A New Compressible Astrophysical Solver. II. Gray Radiation Hydrodynamics", ApJS 196, 20
  (2011), arXiv:1105.2466, https://ar5iv.arxiv.org/html/1105.2466 (accessed 2026-10-22). It is the closest production analogue to P5: a
  flux-limited gray radiation diffusion step taken by backward Euler and solved with hypre's parallel multigrid. The paper says «With hypre
  we have a choice between two parallel multigrid solvers: Schaffer multigrid (SMG) and PFMG», PFMG «typically more efficient and scalable».
  Where the system is not symmetric, the multigrid is used as the preconditioner of a Krylov method.
- McAdams, Sifakis & Teran, "A Parallel Multigrid Poisson Solver for Fluids Simulation on Large Grids", SCA 2010,
  https://pages.cs.wisc.edu/~sifakis/project_pages/mgpcg.html (accessed 2026-10-22). Geometric multigrid used as CG's preconditioner (MGPCG)
  on irregular voxel domains mixing Dirichlet and Neumann boundaries. In the paper's words, «Our approach employs a multigrid cycle as a
  preconditioner for the conjugate gradient method» and «typically reduces the residual by one order of magnitude every 2 iterations».
  It is the graphics and film standard, and GPU ports (e.g. arXiv:2409.06201) follow its restriction, prolongation and smoothing.
- Adams, Brezina, Hu & Tuminaro, "Parallel multigrid smoothing: polynomial versus Gauss-Seidel", J. Comput. Phys. 188(2), 593–610 (2003),
  doi:10.1016/S0021-9991(03)00194-3, abstract via https://www.osti.gov/biblio/1122232 and OceanRep (accessed 2026-10-22). Chebyshev
  polynomial smoothers keep parallel scalability where Gauss-Seidel's ordering does not, and are «often competitive with Gauss-Seidel even
  on serial machines». This is the basis for option (c)'s polynomial preconditioner and for a parallel smoother in (b).
- Bolz, Farmer, Grinspun & Schröder, "Sparse matrix solvers on the GPU: conjugate gradients and multigrid", ACM TOG 22(3), SIGGRAPH 2003,
  https://folk.idi.ntnu.no/elster/master-studs/spampinato/p917-bolz.pdf (accessed 2026-10-22). It is the first GPU CG and regular-grid
  multigrid, built for real-time fluid and geometry solves.
- Harris, "Fast Fluid Dynamics Simulation on the GPU", GPU Gems ch. 38 (2004), https://developer.nvidia.com/gpugems/GPUGems/gpugems_ch38.html
  (accessed 2026-10-22). This is what games do: a fixed 40–80 Jacobi iterations for the pressure Poisson solve, never run to a tolerance.
  In the chapter's words, «More sophisticated methods, such as conjugate gradient and multigrid methods, converge faster», and Jacobi is
  chosen «because of its simplicity». A game gets its speed by not converging, which §0.5 [M0-TJ2] («no physics traded for speed») rules
  out here.
- The cost model these sources share: a V-cycle costs a few fine-grid sweeps, and its rounds stay flat as the grid grows or stiffens,
  where Jacobi-PCG's grow as √κ. On a GPU, though, each level is at least one dependent dispatch, so at small grids a V-cycle is bound by
  dispatch latency. Bolz et al. and the MGPCG GPU ports name this. M5 measures it on the 4090.

## Problem
The probes: `SR_PROBE_DUMP_EVERY` (M0-R6's hook, plus keeping the life's `dumps/`) wrote a dump every 100 steps of the 20 M₀ m_bh_sb life
(mass_sb 67,020.643, the tree's code, 1,600 steps in 75 s). `SR_PROBE_HEAT` prints each step's P5 record (passes, rounds, converged, δ).
`SR_PROBE_SYSTEM=<step>` writes that step's frozen system: the box, Δt, every face's D_f, and T⁰ and C. Each `headless` run loads a dump
into a scene with the life's switches (600 × 400, leave edges, the shipped physics).

**M1** — P5 is 98 % of a hard step. Median of 3 runs of `headless --load-dump s_1400.bin --steps 20 --timing`: the step is 106.536 ms and
P5 104.571 ms. The other passes are under 0.9 ms (P4 hydro 0.871 ms, P6 reactions 0.398 ms). On the easy stretch (s_600) the step is
6.165 ms and P5 4.703 ms (76 %). The hard stretch's time is P5's here, not P6's: M0-D80's iron-block collapse (P6 3,047 ms a step) is
M0-R10's and does not occur in this life up to step 1,600.

**M2** — Where the ceiling binds. Steps 1–1,313 converge. From step 1,314, every step reaches the ceiling (the life's `heat_unconverged`
goes 25 → 299 by step 1,600, about one a step), at 0.11–0.15 s wall a step in calibrate's one-step frames. Steps 1,401–1,420 each stop at
**4,096 rounds after 4–5 passes**, so the round cap binds, not the 64 passes. Near step 1,300 a step converges in 1,093–1,318 rounds over
4–9 passes (34–46 ms wall). On the easy stretch (601–620) it converges in 89–357 rounds (mean 126) over 4–7 passes.

**M3** — The price of the ceiling. The 20 capped steps 1,401–1,420 commit a heat-balance closure δ of 5.6–8.3 × 10⁻³. The converged
steps' δ is 10⁻⁶–6 × 10⁻⁵ (§0.5 [M0-TJ4] (5): δ of the order of the 10⁻⁵ tolerance when the step converged). That is about 0.7 % of the
heat redistributed by rescaling on every capped step. The 5090's record of this life has 5,257 such steps.

**M4** — The conditioning, read off the GPU's frozen system and solved in f64 with the contract's stopping test (scratch model; κ is
Jacobi-PCG's own Lanczos estimate):

| system | gas cells | D_f range (gas–gas faces) | c_i range (pass 1) | max Σ_f D_f / c_i | Jacobi passes / rounds (f64) | κ(D⁻¹A) |
|---|---|---|---|---|---|---|
| step 601 (easy) | 187,925 | 13.4 – 5.2 × 10⁶ | 5.5 × 10⁻⁴ – 1.3 × 10¹⁰ | 5.0 × 10⁸ | 6 / 90 | 1.5 × 10³ |
| step 1,301 | 239,695 | 0.43 – 2.4 × 10⁶ | 2.2 × 10⁻⁴ – 182 | 4.9 × 10⁸ | 4 / 1,155 | 2.4 × 10⁵ |
| step 1,401 (capped) | 239,999 | 0.32 – 6.0 × 10⁵ | 2.6 × 10⁻³ – 2.4 × 10¹⁰ | 8.5 × 10⁷ | **13 / 6,914** | 0.85–1.6 × 10⁵ |
| heat_rim step 1 | 4,096 | 33 – 1.7 × 10⁷ | 0.02 – 1.25 | 3.3 × 10⁹ | 5 / 486 | 1.4 × 10⁷ |
| heat_hot step 1 | 4,096 | 2.22 (uniform) | 1.8 × 10⁻⁵ (uniform) | 7.5 × 10⁵ | 6 / 402 | 1.7 × 10³ |

The model reproduces the GPU's rounds where both can be compared: step 1,301 is 1,155 in f64 against 1,156 on the GPU; heat_rim 486
against 472; heat_hot 402 against 331. On step 1,401 the GPU's f32 needs at least 35 % more: 4,096 rounds in its first 4 passes, against
3,024 in the model's. So the cap is not an f32 artefact. Jacobi needs about 7,000 rounds in exact arithmetic (13 passes of 500–950), with
85 % of the world's cells "heat-free" (Σ_f D_f > 10³ c_i): diffusion-dominated, the regime where Jacobi's rounds grow with the grid.

**M5** — What a round costs on the 4090: `r7_bench`, a wgpu microbenchmark in the scratch copy, median of 5, three runs, idle 4090.
- **Jacobi round, measured.** A P5 Jacobi-PCG round at the full 600 × 400 box is **25.4 µs** (104.5 ms ÷ 4,110 slots, M1).
- **A dependent dispatch.** A red-black half-sweep costs **6.1 µs at 600 × 400**, 4.8 µs at 300 × 200, 3.9 µs at 150 × 100 and
  **3.3 µs at 75 × 50 and smaller**. That floor is the cost of a dependent dispatch whatever its size. An empty dispatch with no bindings
  costs 0.4–0.5 µs.
- **A replica of P5's fused round** (every workgroup sums every tile's partial, then a stencil, then its own partial) is 12 µs at
  600 × 400 and 6–7 µs at 75 × 50. So the round's all-tiles sum costs about 6 µs of the real round's 25: splitting it would not pay.
- **A V(1,1) multigrid round** (red-black Gauss-Seidel, 9 levels, the five finest each a dispatch per operation, 33 dispatches) is
  264 µs, of which 100 µs is a naive single-workgroup coarse solve. **V(2,2)** (53 dispatches) is 327 µs.
- **A Chebyshev-8 round's 9 full-grid dispatches** are 66 µs.

**M6** — Rounds by preconditioner, the f64 model, every Newton pass, the contract's stopping test unchanged (passes are 13 on step 1,401
whatever the preconditioner):

| preconditioner | dispatches a round (est.) | 601 | 1,301 | 1,401 | heat_rim | heat_hot |
|---|---|---|---|---|---|---|
| Jacobi (today) | 1 | 90 | 1,155 | 6,914 | 486 | 402 |
| red-black SGS ×2 | 1 + 5 | 222 | 399 | 2,196 | — | — |
| block-Jacobi 16 × 16 tiles (8 inner sweeps, shared memory) | 1 + 1 | 43 | 382 | 2,276 | — | — |
| Chebyshev-4 on Jacobi | 1 + 3 | 57 | 321 | 1,901 | — | — |
| Chebyshev-8 on Jacobi | 1 + 7 | 60 | 168 | 1,021 | 79 | 65 |
| MG V(1,1), red-black GS, 9 levels | ~33 | 38 | 60 | 463 | 82 | 74 |
| MG V(2,2), red-black GS | ~53 | 28 | 44 | 314 | 64 | 56 |
| MG V(1,1), tile smoother (16 × 16 tiles, 4 inner RB sweeps in shared memory) | ~18 | 24 | 38 | **275** | 56 | 47 |

The multigrid is a geometric one: 2 × 2 piecewise-constant aggregates with Galerkin coarse operators. These keep the 5-point form (a
coarse face sums the two fine faces between two aggregates, and a coarse cell's own term sums its cells' c_i and vacuum faces), so one
shader serves every level, and the vacuum cells drop out.

**M7** — What a preconditioner moves. Against a tight solve of the same step (tile-MG run to 10⁻¹⁰ and Newton to 10⁻¹²), the contract's
stopping test leaves the following C-weighted relative L1 error. The worst cell is taken among cells with C ≥ 10⁻⁴.

| system | Jacobi: L1 (worst cell) | Chebyshev-8: L1 (worst cell) | tile-MG: L1 (worst cell) |
|---|---|---|---|
| step 1,401 | **3.5 × 10⁻⁵** (1.1 × 10⁻³) | 3.2 × 10⁻⁶ (1.3 × 10⁻⁴) | 1.8 × 10⁻⁶ (7.1 × 10⁻⁴) |
| step 1,301 | 1.5 × 10⁻⁵ | 1.3 × 10⁻⁵ | 5.4 × 10⁻⁶ |
| heat_rim, heat_hot | ≤ 10⁻⁶ | ≤ 10⁻⁶ | ≤ 10⁻⁶ |

A stronger preconditioner lands **closer** to the exact step under the same test. The measures under-read the error by up to κ, so the
answers move toward the exact solve, by less than G-STIFF (d)'s 10⁻⁴ GPU-to-twin band.

## Options
Each option's time is the M6 rounds (f64) × the M5 round cost (low–high), for P5 alone on the RTX 4090 at 600 × 400:

| option | step 1,401 (hard) | step 1,301 | step 601 (easy) | converges the hard step? | effort | what it moves |
|---|---|---|---|---|---|---|
| **(a) Jacobi kept, the cap as it is** | **104.5 ms measured** (capped, δ 6–8 × 10⁻³); about 176 ms if run to the end | ≈ 29.4 ms (1,156 measured rounds × 25.4 µs; 34–46 ms wall measured) | 4.70 ms measured | no (4,096 rounds) | none | nothing |
| **(b) geometric multigrid V-cycle, tile smoother** (5 fine levels plus one coarse-solve dispatch, ~18 dispatches a round at 140–270 µs) | **39–74 ms** | 5.3–10.3 ms | 3.4–6.5 ms | yes, 275 rounds (13 passes) | 3 BUILD tasks | answers toward the exact step, ≤ 3.5 × 10⁻⁵ C-weighted (M7) |
| (b′) V(1,1)/V(2,2) red-black GS, a dispatch per operation (33–53 dispatches at 210–420 µs) | 85–162 ms | 9.2–21 ms | 6–16 ms | yes | 3 tasks | the same |
| **(c) Chebyshev-8 on Jacobi** (7 more full-grid dispatches a round, 76–128 µs) | 78–131 ms | 13–22 ms | 4.6–7.7 ms | yes, 1,021 rounds | 1 task | the same, ≤ 3.2 × 10⁻⁶ (M7) |
| (c′) block-Jacobi on 16 × 16 tiles (one shared-memory dispatch more, 37–45 µs) | 84–102 ms | 14–17 ms | 1.6–1.9 ms | yes, 2,276 rounds | 1 task | the same |

- **The time each step budgets.** A slower round costs more per round, but the ceiling stays the contract's 4,096 rounds ([M0-D37]). A
  step at that ceiling would cost 4,096 × the round. (b) is never near it (≤ 60 rounds a pass in M6), and (c) is at a quarter of it on
  step 1,401.
- **What any option keeps.**
  - P5's equations, its tolerances, the 64 passes / 4,096 rounds ceiling, and the carry-over machinery's contract (a step carried over
    submissions, G-STIFF (d) bit for bit at 8 rounds).
  - The twin is unchanged: it solves to 10⁻¹², where the preconditioner does not change the answer.
  - The scopes that read the change — heat_implicit, heat_stiff (d) at 10⁻⁴, radiation_force — see answers that move toward the exact
    step (M7), never away.
- **The 4,096-round cap in (b) and (c).** The cap counts rounds, so the same cap allows more GPU time per step. A future TJ may want a cap
  in wall terms. That is not needed here: (b) is far under it.

## Recommendation
**(b): the geometric multigrid V-cycle with a tile-local smoother, as the preconditioner of P5's PCG.**

Why:
- It is the only option estimated to beat today's capped hard step outright: 39–74 ms against 104.5 ms.
- Every one of those steps converges, instead of committing δ of 6–8 × 10⁻³ (M3). That is a physics gain, in line with [M0-TJ2]'s «no
  physics traded for speed».
- Near step 1,300 it is 3–5.5× faster (5–10 ms against ≈ 29.4 ms).
- Its rounds stay flat as the world grows or stiffens. Jacobi's and Chebyshev's grow with the grid.
- Its design fits the 4090's measured cost, which is dispatch latency (M5): one shared-memory dispatch does several sweeps on a 16 × 16
  tile plus its halo (about 9 KB, under WebGPU's 16 KB workgroup default). It needs about 3 dispatches a level and one single-workgroup
  dispatch for the levels from 19 × 13 down.
- (c) is the cheap fallback: one task, converges, roughly today's time on hard steps.

**The time it saves a life is modest.** On the 5090's 20 M₀ record, 5,257 capped steps × 30–65 ms ≈ 3–6 min of P5 a life on a 4090-class
card. The order-of-magnitude lever is the 13 Newton passes a hard step needs (M4, M6: unchanged by any preconditioner), which is M0-R8's
warm start and M0-R9's smaller Δt.

**What would falsify it.** Any one of these sends the build to (c):
- The built V-cycle round costs more than 380 µs at 600 × 400 on the RTX 4090 (104.5 ms ÷ 275 rounds: no gain over today).
- The GPU's f32 rounds on step 1,401 exceed 2 × the model's 275.
- The easy stretch's P5 mean grows past 1.25 × 4.70 ms. The first remedy is a hybrid: Jacobi until a pass has run 64 rounds, then
  restart that pass's CG with the V-cycle (within the step, so dumps still resume bit for bit).

**The D block, if the lead picks (b)** — three BUILD tasks:
- D-a builds the level hierarchy (Galerkin 2 × 2 aggregation each pass), the tile smoother, restriction, prolongation and the coarse
  dispatch in heat.wgsl, with a GPU test that the V-cycle is a symmetric positive operator that cuts the residual.
- D-b wires it into the round machine (the slot chain, the carry-over and the latch), keeping G-STIFF (d) bit for bit at 8-round
  submissions.
- D-c is the cost case.

Its spec, to be filed after this block:
- `Deliver:` (D-a) the V-cycle preconditioner behind an engine setter like `Step::set_heat_rounds` — `Step::set_heat_precond`, tests only, no scene key (default `jacobi` until D-c grades it); (D-b) the round
  machine on either preconditioner; (D-c) the default `multigrid`, contract §0.5 recording the measured costs.
- `Verify:` heat_implicit, heat_stiff, heat_limiter, heat_gauss and radiation_force GO on both preconditioners, plus a new
  `heat_multigrid` scope.
  - On the dump of the 20 M₀ life at step 1,400 (re-made by the D from the shipped `--dump-every`, never committed: 13 MB), 20 steps on
    the RTX 4090: every step converged, P5's median-of-3 mean ≤ 75 ms (Fail: > 104.5 ms).
  - On the life's s_600 stretch: P5's mean ≤ 5.9 ms (1.25 × 4.70).
  - The step's T against Jacobi's: C-weighted relative L1 ≤ 10⁻⁴.
  - Plant `vcycle-no-coarse` (the coarse correction dropped): red on the round count.

## UNVERIFIED
- **(b)'s and (c)'s times are estimates, not measurements** (NOT PROVEN — synthetic):
  - The rounds are the f64 model's (M6), not f32 GPU rounds. f32 needed 1.0–1.35× the model's rounds for Jacobi (M4), and may differ for
    a V-cycle.
  - The round costs are built from M5's per-dispatch costs, not from built kernels. The real fused round costs about 2× its replica
    (25.4 against 12 µs), so the high ends assume the same factor on the finest levels.
  - The tile smoother's per-dispatch cost (12–20 µs at 600 × 400, inner sweeps in shared memory) was not benchmarked as such. Neither was
    a shared-memory coarse solve, assumed at 20–100 µs; the naive one measured 100.
- **The Quadro RTX 4000 and the 5090 were not measured.** On the Quadro (2.65 µs a dispatch, M0-R3) the balance may differ. sun_disk, the
  game's Sun-like case (P5 0.6189 ms on the Quadro, [M0-T122]), could not run: `headless --scene scenes/sun_disk.json` exits 6,
  `calibration invalid: preset_mass_sb: missing` (the calibration M0-V9 owes). The life's s_600 stretch stands in for an easy step.
- **The modelled stretch is not the record's.** This life on the tree's code reads `white_dwarf` from about step 800 and shows no
  core_collapse by step 1,600 on the 4090, where the 5090's record shows core_collapse at 1,489. That is M0-D81's split. The probes measure
  the life the tree computes on the 4090, steps 600–1,600 only. The record's later capped steps (after its supernova) were not probed.
- **The model's start of each pass** omits P5's Galerkin shift along the heat-free cells (M0-D36), which may change pass 1's rounds by a
  few.
- **The probe hooks** live only in `/tmp/m0r7` on laserax-ai-server (`SR_PROBE_DUMP_EVERY`, `SR_PROBE_HEAT`, `SR_PROBE_SYSTEM`; the
  frozen-plane buffer made readable). Nothing in the tree changed.

## The lead's answer
2026-10-22, the question tool (PLAYBOOK §12), options (b) recommended, (c), (a), with the measured times above in each option. The lead
picked: «(b) Multigrid V-cycle (Recommended)». Filed as M0-D82 (the V-cycle and its hierarchy), M0-D83 (the round machine) and M0-D84 (the
default and its cost), after M0-R7.
