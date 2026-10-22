# M0-R8 — Heat-solve research: a warm start for P5's implicit step

Box: every GPU number below was measured on **laserax-ai-server, NVIDIA RTX 4090** (driver 595.99.02, Vulkan). Before each set,
`nvidia-smi` listed no process on the card, and the load was 0.1–1.2 (5.4 during the scopes' own parallel cases). The lead's word at
launch: «run everything that requires a gpu on laserax-ai-server, just like D84 did, even the tests at the end». The work ran in a
scratch copy, `/tmp/r8/probe` on the server: `tools/pb/scratch_copy.sh` of `~/workspace/sandbox_reactions` at a3aaefd, the tree's HEAD,
with M0-D84's probe patch and this task's `r8_patch.py`. Nothing in the tree changed. Logs: `logs/M0-R8.log` holds every probe line,
timing line and scope tail. model=claude-opus-5-5 level=high.

**The short answer.**
- A warm start does **not** fix the hard steps. On M0-R7's hard stretch, P5 falls from 28.6 to 26.8–28.2 ms (−1.5 to −6 %), and the
  Newton passes stay at 12.45 a step in every mode.
- On those steps the pass count is set by the update's factor-2 clamp ([M0-TJ4] (3)): a few cells change T by up to 2¹⁰·⁶ within one
  step, and passes = ⌈that log₂⌉ + 2–3 on every step measured (M5). No guess built from the last step sees those cells.
- Where the steps are moderate, the warm start does help:
  - the tree's own step-1,400 stretch: P5 −37 to −40 %;
  - the whole 20 M₀ life to step 1,400: **−25 % wall** with the rate extrapolation (c), 19.02 → 14.20 s.
- It has costs:
  - a contract change ([M0-TJ4] (3): «nothing kept from the previous step»);
  - one more plane in every dump (+6 %);
  - a reset on every state upload;
  - answers that move within the solve's tolerance.

## Evidence
- Clemens, Wilke & Weiland, "Efficient extrapolation methods for electro- and magnetoquasistatic field simulations", Advances in Radio
  Science 1, 81–86 (2003), https://ars.copernicus.org/articles/1/81/2003/ars-1-81-2003.pdf (accessed 2026-10-22).
  - It compares PCG start vectors across implicit time steps: zero, the last step's solution x⁽ⁿ⁾ (their eq. 4: our option (b)), and a
    second-order Taylor step x⁽ⁿ⁾ + Δt dx/dt (eq. 5: our option (c)).
  - Its caution matches M4: «the complete reduction of the spectral error components should not benefit from an available good start
    solution x₀. However, since the PCG process will be terminated when a user-specified finite accuracy is attained, small error
    components of the start vectors may not require additional iterations».
  - It also finds that no single extrapolation suits every transient. Its remedy is to pick, per step, the candidate with the smallest
    residual.
- Fischer, "Projection techniques for iterative solution of Ax = b with successive right-hand sides", NASA ICASE report (1993),
  https://ntrs.nasa.gov/citations/19940020363 (accessed 2026-10-22).
  - The guess is built from the span of earlier solutions, for Navier–Stokes pressure solves.
  - In the report's words, «these projection techniques lead to as much as a two-fold reduction in solution time» in flows with
    significant dynamic activity.
- Austin, Chalmers & Warburton, "Initial Guesses for Sequences of Linear Systems in a GPU-Accelerated Incompressible Flow Solver",
  arXiv:2009.10863 (2020), https://ar5iv.labs.arxiv.org/html/2009.10863 (accessed 2026-10-22). This is the GPU analogue.
  - On pressure solves at Re = 500, the previous step's solution takes «about 15 PCG iterations on average» against ≈ 24 from zero
    (−37 %).
  - Projection or least-squares extrapolation over 8 earlier solutions takes ≈ 2.4–2.6 iterations (−90 %).
  - Over a whole run, extrapolation is 38–45 % faster than the previous-step guess.
  - At Re = 7,000 the previous-step guess «saves just 4 iterations per solve». The more dynamic the flow, the less the last answer
    predicts the next.
- What these sources share, and where P5 differs:
  - They study **linear** solves, where a guess only shortens the Krylov rounds.
  - P5 is a **Newton** iteration whose update is clamped to a factor 2 in T per pass ([M0-TJ4] (3)). There, a guess shortens the passes
    only where it lands within a factor 2 of the answer on every cell that the clamp would otherwise bind.
  - None of the sources covers that case. M4–M5 measure it.

## Problem
**What P5 starts from today, read from the code** (`crates/sr-engine/shaders/heat.wgsl`):
- `heat_round`'s `CELL_BUILD` sets every gas cell's first iterate to the start state: `solve[SV_X] = u_of(t0)`, `t0 = frozen[T0]`, the T
  that `heat_cells` reads from E after P1–P4 of this step. That is [M0-TJ4] (3)'s «the first iterate is the pass's start state (nothing
  kept from the previous step)».
- Each later pass's CG starts from the last Newton iterate. `CELL_RESID` takes r = b − A x at x = u^{k−1}. `CELL_START` shifts x along
  the heat-free indicator (M0-D36). Each solve inside a step is therefore already warm-started from the pass before.
- Only the step's first iterate is cold. Since M0-D84, the preconditioner is the V-cycle.

The probe (scratch only, `r8_patch.py`):
- **`SR_PROBE_WARM`** (0 | 1 | 2) is read when P5 is built, in the params' spare word. It picks the first iterate:
  - 0: T⁰, today's start;
  - 1: the last step's solved T;
  - 2: T⁰ + r_i Δt_n, where r_i = (T_i − T_i⁰)/Δt_{n−1} is the cell's P5 rate in the last step.
  - A guess outside [T⁰/2, 2 T⁰] (or not finite) falls back to T⁰, as the Newton update's own clamp would. It is then floored at
    T_floor.
- **The stash.** `heat_apply` writes r_i and T_i into krylov's two S planes. These planes are free between steps: a round writes them
  before it reads them. So the probe needs no new GPU buffer.
- **`SR_PROBE_HEAT`** prints each step's passes, rounds, δ and Δt. It also prints each candidate's distance to the solved T:
  - the C-weighted relative L1, Σ C|T − g| / Σ C T⁰;
  - the cells off by more than 10⁻³;
  - the cells off by more than a factor 2 or 16;
  - the largest |log₂(T/g)|.
- **`SR_PROBE_STASH_OUT` / `_IN`** write the stash every step and load it before a run.

The stretches:
- **Hard**: M0-R7's 20 M₀ dump at step 1,400 (`/tmp/d84/r7_s_1400.bin`, the stretch M0-D84's Pass/Fail lines graded on).
- **The tree's 1,400**: the shipped V-cycle life's dump (`v_s_1400.bin`).
- **Two easy stretches**: `r7_s_600`, `v_s_600`.
- Each was stepped once cold from its dump to make a stash. The 20 steps after that step run with a stash in every mode, from the same
  state. The scene is `life_scene.json`: 600 × 400, leave edges, every switch on.

**M1** — P5's share of the step, cold (median of 3 `headless --timing`, 20 steps from the stashed state):

| stretch | P5 (ms) | step (ms) | P5's share |
|---|---|---|---|
| hard (r7 1,401–1,420) | 28.605 | 30.400 | 94 % |
| tree 1,401–1,420 | 10.418 | 12.257 | 85 % |
| easy r7 601–620 | 4.987 | 6.446 | 77 % |
| easy tree 601–620 | 3.933 | 5.410 | 73 % |

- The hard stretch's time is P5's, as M0-R7's M1 found.
- This life has no iron-core collapse in these stretches. M0-D80's 3 s collapse step is P6's (M0-R10), and a warm start cannot touch it.

**M2** — How far one step's answer moves from each candidate start (cold run, 20 steps a stretch; solved T against the guess):

| stretch | C-weighted L1: cold T⁰ | (b) last T | (c) T⁰ + r Δt | cells off > 10⁻³ (cold → c) |
|---|---|---|---|---|
| hard | 2.08–2.26 × 10⁻² | 1.69–1.96 × 10⁻² | 2.71–4.77 × 10⁻³ | 237–239 k → 141–224 k of 240 k |
| tree 1,400 | 1.75–1.91 × 10⁻² | 6.26–6.77 × 10⁻³ | 1.54–1.75 × 10⁻³ | 238 k → 115–134 k of 240 k |
| easy r7 600 | 3.61–4.53 × 10⁻³ | 5.32–6.55 × 10⁻³ | 4.47–5.12 × 10⁻⁴ | 104–122 k → 39–67 k of 188 k |
| easy tree 600 | 2.46–2.97 × 10⁻³ | 3.40–4.68 × 10⁻³ | 3.04–3.37 × 10⁻⁴ | 151–166 k → 58–80 k of 188 k |

- Against the solve's own scales, every start is far from the answer: the CG's 10⁻⁵ tolerance, and the passes' 10⁻⁶ in L1 and 10⁻³ a
  cell.
  - The best guess, (c), is 30–480× the solve tolerance.
  - It also leaves 21–56 % of the cells outside the per-cell 10⁻³ end test.
- (c) brings the start **5–10× closer** in L1.
- (b) is better than cold only where the state barely moves: the tree's 1,400 stretch. On the easy stretches it is worse than cold,
  because T⁰ already holds P1–P4's change and the last answer does not.

**M3** — Passes and rounds a step, with each start in place (20 steps from the same stashed state; every step converged):

| stretch | (a) cold: passes / rounds (mean) | (b) last T | (c) T⁰ + r Δt |
|---|---|---|---|
| hard | 6–14, **12.45** / 59–164, **100.2** | 12.45 / 92.5 | 12.45 / 98.2 |
| tree 1,400 | 4.00 / 36.7 | 3.45 / **18.4** | 3.00 / **20.7** |
| easy r7 600 | 4.50 / 11.6 | 5.15 / 15.2 | 3.85 / 12.3 |
| easy tree 600 | 3.00 / 8.8 | 3.00 / 8.1 | 3.00 / 8.8 |

**M4** — Why the rounds barely move.
- The rounds a pass needs are set by its end test against the pass's own scale. They are not set by the distance of the first iterate,
  because every pass's CG already starts from the last iterate.
- A better first iterate removes one pass where it lands every cell within the end test sooner: the tree's 1,400 stretch, 4 → 3. It
  also halves the first pass's rounds there.
- Elsewhere the remaining error is spread across the spectrum, and Clemens et al.'s caution applies.

**M5** — Why the passes do not move on the hard stretch.
- The number of cells whose T changes by more than a factor 2 within one step: 7–55 (cold), 7–49 (c).
- Of these, 0–9 change by more than 16×, and the largest change is **2^2.6 – 2^10.6** whatever the guess.
- Each Newton pass moves a cell by at most a factor 2 ([M0-TJ4] (3)). So a cell that heats 1,500× needs ≥ 11 passes.
- Step by step on the hard stretch, the passes read ⌈largest log₂⌉ + 2 or 3. The pairs (passes / ⌈log₂⌉) are: 13/11 ×12, 13/10 ×5,
  14/11 ×3, and 6/3 ×2.
- Those cells are ones the last step did not heat: their rate r_i is ~0, so (c) predicts nothing there.
- **The hard step's 13 passes are the clamp's**, and only a change to the update (or a smaller Δt, M0-R9) reaches them.

**M6** — The time, median of 3 `headless --timing` runs interleaved, 20 steps from the same stashed state (load 0.69–1.01, no other
process on the 4090; every run's 20 steps converged):

| stretch | (a) cold P5 (ms) | (b) last T | (c) T⁰ + r Δt |
|---|---|---|---|
| hard | 28.605 (28.59, 28.61, 29.10) | **26.836** (−6.2 %) | 28.190 (−1.5 %) |
| tree 1,400 | 10.418 (10.40, 10.42, 10.42) | **6.222** (−40 %) | **6.587** (−37 %) |
| easy r7 600 | 4.987 | 5.951 (**+19 %**) | 4.973 (−0.3 %) |
| easy tree 600 | 3.933 | 3.798 (−3.4 %) | 3.955 (+0.6 %) |

**M7** — The whole life.
- The command: `calibrate --only m_bh_sb --bracket 67020.643,67087.664 --max-steps 1400 --checkpoint-every 1400 --adapter "RTX 4090"`,
  the 20 M₀ life M0-D84 measured, on the probe binary, 3 runs per mode interleaved. Each mode's runs were identical to the ms.

| mode | wall to step 1,400 (s) | sim_time at 1,400 | wall per t.u. | heat_unconverged |
|---|---|---|---|---|
| (a) cold | **19.019, 19.007, 19.037** | 9.042959 (= M0-D84's V-cycle life, 9.0427) | 2.10 s | 3 |
| (b) last T | 18.399, 18.449, 18.444 (−3 %) | 9.212082 | 2.00 s | 0 |
| (c) T⁰ + r Δt | **14.204, 14.214, 14.199 (−25 %)** | 9.250446 | 1.54 s (−27 %) |  4 |

- The three lives part after the first warm step, as M0-D84's two preconditioners did. Each is a different f32 trajectory within the
  solve's tolerance, so the sim times differ.

**M8** — Do the answers move? One step from the same state.
- Each comparison is one step from the same state: the cold run's dump at every other step of each stretch, with its stash, 10 states a
  stretch. That step was run in each mode and on a **tight** build (SOLVE_TOL 10⁻⁶, NEWTON_HEAT 10⁻⁷, MAX_ROUNDS 32,768, cold).
- T was compared by M0-D84's `tcmp`: C-weighted relative L1, and the worst cell with C ≥ 10⁻⁴.

| stretch | cold vs tight | (b) vs tight | (c) vs tight | (c) vs cold |
|---|---|---|---|---|
| hard | 1.6 × 10⁻⁶ – 1.3 × 10⁻⁵ | 1.6 × 10⁻⁶ – 1.3 × 10⁻⁵ | 1.6 × 10⁻⁶ – 1.3 × 10⁻⁵ | 1.3 × 10⁻⁸ – 5.1 × 10⁻⁷ |
| tree 1,400 | 3.6 – 5.6 × 10⁻⁶ | 2.7 × 10⁻⁶ – 1.1 × 10⁻⁵ | 1.4 × 10⁻⁶ – 1.6 × 10⁻⁵ | 3.0 × 10⁻⁶ – 1.5 × 10⁻⁵ |
| easy r7 600 | 2.2 × 10⁻⁷ – 2.8 × 10⁻⁶ | 3.6 × 10⁻⁷ – 1.6 × 10⁻⁶ | 6.6 × 10⁻⁸ – 1.9 × 10⁻⁶ | 2.7 × 10⁻⁷ – 2.2 × 10⁻⁶ |
| easy tree 600 | 1.5 – 2.4 × 10⁻⁶ | 1.3 – 2.6 × 10⁻⁶ | 3.2 × 10⁻⁷ – 1.3 × 10⁻⁶ | 1.2 – 2.0 × 10⁻⁶ |

- Every start lands within the solve's 10⁻⁵ band of the tight answer, at G-STIFF (d)'s 10⁻⁴ GPU-to-twin band or under it.
- The warm start moves the answer **within** the tolerance: by up to 1.5 × 10⁻⁵ against cold on the tree's stretch. It is neither
  closer to the exact step nor farther, on the whole.
- |δ| grows on the tree's stretch under (c): up to 7.9 × 10⁻⁶ against cold's 3.4 × 10⁻⁷, still of (4)'s order.
- The worst cells (C ≥ 10⁻⁴) differ by up to 4.8 × 10⁻² on the hard stretch and 0.12 on r7 600, the same in every mode against tight.
- **It is not a pure speed-up.** It changes the answers at the solve's own tolerance, the way M0-D84's preconditioner change did. A life
  measured on one start is not the life on the other (M7).

**M9** — The scopes on the probe copy (RTX 4090, `SR_TEST_ADAPTER="RTX 4090"`): heat_implicit, heat_stiff, heat_limiter, heat_gauss,
radiation_force and heat_multigrid.
- **(a) cold** (`--task M0-R8-warm0`): GO, 53/53.
- **(c)** (`M0-R8-warm2`): NO-GO, 48/53. **(b)** (`M0-R8-warm1`): NO-GO, 49/53.
- The reds have two causes:
  - **Hidden state** (4 of (c)'s 5, all of (b)'s 4). Three cases are affected:
    - `gpu_one_pass_is_the_twins_and_runs_bit_for_bit_twice` (::jacobi, ::multigrid). The second run, after `restart()`, finds the
      first run's stash and starts at its answer: 1 pass, 0 rounds, against 2 passes and 51 / 6 rounds.
    - heat_stiff's `(d) carried` (::jacobi, ::multigrid). The same: the carried run is warmed by the uncarried run's stash, so its
      record differs (8 → 7 passes; δ 3.1 → 2.8 × 10⁻⁵).
    - A shipped warm start must clear the stash on every state upload, or the same state no longer gives the same step.
  - **A bound moved** ((c) only, ::jacobi). `gpu_a_sharp_pulse_stays_non_negative_and_spreads_as_two_chi_t` keeps the pulse's heat to
    3.7 × 10⁻⁵ against the 10⁻⁵ bound (cold: 9.1 × 10⁻⁶). On the V-cycle, (c) keeps it to 1.1 × 10⁻⁷ (cold 3.1 × 10⁻⁶).
    - Under the same stopping test, Jacobi's looser stop (M0-R7's M7) lets the rate-extrapolated start leave more heat behind over 15
      passes.

**M10** — Resume.
- The setup: (c) unbroken from step 1,401 to 1,421, against a life cut at 1,411 and continued from that dump.
  - **With the stash carried** (loaded with the dump): the dump at 1,421 is **byte-identical** to the unbroken one, for both (b) and
    (c).
  - **Without it**, the continued life's first step is cold. By step 1,412 it already differs by 8 × 10⁻⁶ – 1.2 × 10⁻⁵ C-weighted L1,
    and by 6.3–7.7 × 10⁻⁶ at 1,421.
- So [M0-T26]'s bit-identical resume, and [M0-D77]'s continued calibrate life, need the stash in the dump and the checkpoint.

## Options
Times are M6's and M7's on the RTX 4090.

| option | hard P5 | tree 1,400 P5 | easy P5 | the life to 1,400 | memory | dump | answers |
|---|---|---|---|---|---|---|---|
| **(a) the start as it is (T⁰)** | 28.6 ms | 10.4 ms | 4.99 / 3.93 ms | 19.02 s | — | — | — |
| **(b) the last step's T** | 26.8 ms (−6 %) | 6.2 ms (−40 %) | 5.95 (**+19 %**) / 3.80 ms | 18.44 s (−3 %) | 0 (krylov's free S plane) | +1 plane, 960 KB of 15.4 MB (+6 %) | move within 10⁻⁵ (M8) |
| **(c) T⁰ + the last step's P5 rate × Δt** (the extrapolation from the last two steps' P5 states) | 28.2 ms (−1.5 %) | 6.6 ms (−37 %) | 4.97 / 3.96 ms (±0.6 %) | **14.20 s (−25 %)** | 0 (krylov's free S plane) | +1 plane (+6 %) | move within 10⁻⁵ (M8); Jacobi's sharp pulse 3.7 × 10⁻⁵ > its 10⁻⁵ (M9) |

What any warm start costs, beyond the table:
- **A contract change.** [M0-TJ4] (3)'s «the first iterate is the pass's start state (nothing kept from the previous step: a dump
  resumes bit-identical on [M0-T26]'s keys)» is superseded. So is the dump's layout in §2.12.3 / [M0-D33]: a third plane after F, flagged
  in `resume`. That makes it a change to persisted data.
- **A reset.** Every state upload (scene start, `--load-dump` of an older dump, a test's `restart`) clears the stash. A player's edit
  is not a reset: the guess falls back to T⁰ wherever it is off by more than a factor 2, and it stays a guess.
- **The calibration.** M0-V9's values would be measured on whichever start ships. The 20 M₀ life's trajectory moves at the solve's
  tolerance (M7), as M0-D84's preconditioner change moved the Sun-like life's ignition by 5 t.u.
- **Not the hard steps.** No warm start reaches M5's clamp-bound passes. The order-of-magnitude lever there is the update's factor-2
  clamp (a Newton step in log T, or a line search) or M0-R9's smaller Δt. That is named here, not filed.

## Recommendation
**(c): start each step's Newton passes from T⁰ + the last step's P5 rate × Δt, on the V-cycle, the stash carried by the dump.**

Why:
- It is the only option measured to save real time on a whole life: −25 % wall on the 20 M₀ life to step 1,400 (M7). That is the lead's
  "so that our testing here doesnt take ages", and it comes for one GPU plane that already exists.
- It never costs more than +0.6 % on an easy step (M6), where (b) costs +19 %.
- Its answers stay within the solve's tolerance of the tight step (M8).
- It is honest about its reach. It does not touch the hard steps (−1.5 %), whose passes are the clamp's (M5).

What would falsify it:
- The built (c) runs the 20 M₀ life to step 1,400 on the RTX 4090 in more than 0.85 × cold's wall: 16.2 s against 19.02 s, median of 3.
- Any heat scope red on the V-cycle once the stash is reset on upload.
- A continued life that is not byte-identical to the unbroken one.

**The D block, if the lead picks (c)** — one BUILD task, filed after M0-R8, run before M0-V9 (the calibration it moves):
- `Deliver:`
  - P5's first Newton iterate on the V-cycle is T_i⁰ + r_i Δt_n.
    - r_i = (T_i − T_i⁰)/Δt_{n−1}, written by `heat_apply` into krylov's first S plane.
    - The guess falls back to T⁰ outside [T⁰/2, 2 T⁰] or when not finite, and is floored at T_floor.
    - Jacobi keeps T⁰ (its sharp pulse, M9).
  - r is zeroed on every `State::upload` and every `load_dump` of a dump without it.
  - r is carried as the dump's 17th plane (`resume.rate`, after F's two, §2.12.3 / [M0-D33]), restored by `--load-dump` and in
    calibrate's checkpoint.
  - Contract §0.5 records the override of [M0-TJ4] (3)'s first iterate and the dump's layout, with this report's M7–M10.
- `Verify:`
  - heat_implicit, heat_stiff, heat_limiter, heat_gauss, radiation_force and heat_multigrid GO on both preconditioners. That includes
    `runs_bit_for_bit_twice` and `(d) carried`, which the reset keeps.
  - A new case: a (c) life cut by a dump and continued is byte-identical to the unbroken one at the cut + 10.
  - The 20 M₀ life to 1,400 (the calibrate line of M7) on the RTX 4090, median of 3: ≤ 16.2 s. Fail: > 19.0 s.
  - Plants: `warm-rate-not-dumped` (the plane left out of the dump) red on the resume case; `warm-no-reset` red on
    `runs_bit_for_bit_twice`.

## UNVERIFIED
- **One life, one card.** M7 is the 20 M₀ life to step 1,400 on the RTX 4090. Not measured:
  - lighter lives, the Sun-like disk (sun_disk exits 6: `preset_mass_sb: missing`, M0-V9's calibration);
  - the Quadro RTX 4000 (the game's card, [M0-T122]) and the RTX 5090.
  - The −25 % may not carry to the Sun-like life, whose steps are easy (M6's easy rows: ±0.6 %).
- **The record's later hard steps were not probed.** These are the 5090 record's 5,257 capped steps after its supernova, which this
  4090 life does not reach by step 1,600 (M0-D81's split). The hard stretch here is M0-R7's dump run on today's code.
- **The reset is reasoned, not built.** The probe never clears the stash. M9's hidden-state reds follow from the second run finding the
  first's stash (1 pass, 0 rounds: it started at the answer), but no probe cleared it and re-ran the scopes.
- **A player's edit was not probed.** The fallback to T⁰ beyond a factor 2 makes a stale guess harmless to the answer (the solve still
  ends at its tests), but its cost on the edited step was not measured.
- **Where M5's cells are** (which cells change 2¹⁰ in a step: likely thin cold gas heated by a front, as in M0-D37) was not mapped.
- **observe_run and the calibrated scopes** (sun_burn, dump, twin, …) were not run with a warm start. The Sun-like life's ignition step
  may move, as it did under M0-D84.

## The lead's answer
2026-10-22, the question tool (PLAYBOOK §12). The options were (c) recommended, (a) and (b), each with M6–M7's measured times, and the
question named the 17th dump plane and the contract change. The lead picked: «(c) Rate extrapolation (Recommended)». Filed as M0-D85, after
M0-R8.
