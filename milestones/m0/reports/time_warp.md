# Time-warp research — M0-R3 (2026-10-08)

**Summary.** The squeeze maps ~20 orders of magnitude of real time (a core collapse in milliseconds,
a Sun-like life of ~11 Gyr) onto ~3–4 orders of sandbox time; the readout carries the rest. A whole
life costs **steps**, not seconds: steps per life = (5–10 × R/Δx) × the clock gaps (SM worked
example), and the gaps K1–K3 need put a Sun-like life at **~22,500 to ~340,000 steps** depending on
how large the gaps must be (scenarios L, C, H below — the gap is unmeasured, oracle O14). **Measured
on the Quadro RTX 4000 through wgpu 30 / Vulkan** (a proxy of sim_models.md M8's step, M1–M4): a
step over the whole 600 × 400 world costs **2.13 ms** (2.4× R2b's 0.88 ms estimate; the FFT gravity
alone 1.24 ms), so only ~6 steps fit a frame; a step restricted to the box the star occupies costs
**0.11 ms** (Sun-like) to **0.29 ms** (massive), bounded below by ~2.6 µs per dispatch. **Verdict:
the top speed (a Sun-like life in ~10 s at 60 frames a second) is not shown held.** It is reachable
only by computing the star's box rather than the world, and then only if the gap is small (L: held
with room; C: at the edge; H: ~5× over). The shortfall goes to M0-TC as a fork with a measured
decision point (`## Recommendation`, F1). Recommended design: one adaptive global step (CFL) with
the speed defined in sandbox time, steps grouped per frame and drawn with interpolation, the work
limited to an active box, gravity re-solved every k steps in quiet phases, an event latch on the GPU
for the automatic slow-down, and an age readout built on stage clocks with SSE-style fractional
ages. Projective jumps (Gear–Kevrekidis) are the designed fallback; sub-cycling and a reduced model
are rejected for M0.

Written to PLAYBOOK §4 Rule 7: every claim lives once in `## Evidence` with its URL and access date
on its line; the sections cite claims by `E<n>`, measurements by `M<n>`, star_physics.md's claims as
`SP-E<n>` and invariants as `K<n>`, sim_models.md's as `SM-E<n>`, `SM-O<n>` (oracles) and `M8`,
engine_stack.md's as `ES-E<n>`. **Opinion** and **my arithmetic** are labelled; every claim no
source confirmed is in `## UNVERIFIED`. Box for every measurement: **linux-pc**.

## Evidence (each claim with its source)
**Games: speed controls and warps**
- E1 · Universe Sandbox (Steam discussion): "Speeding it up definetly makes it less accurate"; rings stay stable "if you keep it at 1 day/ sec"; the integrator choice (Velocity Verlet vs Forest-Ruth / PEFRL) sets long-run error — players' statements, not the developer's — https://steamcommunity.com/app/230290/discussions/0/1699416432411189623 (accessed 2026-10-08)
- E2 · Universe Sandbox: the current version includes "simulation of stellar evolution" among its physics features — https://en.wikipedia.org/wiki/Universe_Sandbox (accessed 2026-10-08)
- E3 · RimWorld (Steam discussion): "The game disallows and grays out the speed up time functions at the start of a dangerous event" — https://steamcommunity.com/app/294100/discussions/0/1741139415983117446 (accessed 2026-10-08)
- E4 · Fiedler, *Fix Your Timestep!*: "The renderer produces time and the simulation consumes it in discrete dt sized steps" through an accumulator; the "spiral of death" is "what happens when your physics simulation can't keep up with the steps it's asked to take"; the accumulator's remainder ÷ dt gives the alpha to interpolate between the previous and current physics states for drawing — https://gafferongames.com/post/fix_your_timestep/ (accessed 2026-10-08)

**Stellar-evolution codes: stage clocks and the age readout**
- E5 · Hurley, Pols & Tout 2000 (SSE), MNRAS 315, 543: "continuous formulae accurate to within 5% of detailed models", for "very rapid but accurate evaluation of stellar properties, and in particular for use in combination with N-body codes"; t_BGB (eq. 4) and t_MS = max(t_hook, x·t_BGB) (eq. 5) as functions of mass; stellar types k = 0–14 (MS, Hertzsprung gap, giant branch, core He burning, early and TP-AGB, naked He stars, He/CO/ONe white dwarfs, neutron star, black hole); on mass change on the MS, "We must effectively age the star, so that the fraction of MS lifetime remains unchanged, by using t′ = (t′_MS/t_MS) t" (§7.1) — https://ar5iv.arxiv.org/html/astro-ph/0001295 (accessed 2026-10-08)
- E6 · Hurley, Tout & Pols 2002 (BSE), §2.6.6 "Rejuvenation": a star that transfers mass "must be aged so that the fractional age of the star, in terms of its current evolution phase, remains unchanged"; an accreting MS star's core "mixes in unburnt fuel … so that the star appears even younger" — https://ar5iv.labs.arxiv.org/html/astro-ph/0201220 (accessed 2026-10-08)
- E7 · Dotter 2016 (MIST 0): primary equivalent evolutionary points defined by physical conditions — PreMS (central T rises above a value), ZAMS ("the first point after the H-burning luminosity exceeds 99.9% of the total luminosity"), IAMS (Xc = 0.3), TAMS (Xc = 10^−12), RGBTip (luminosity maximum), ZACHeB ("onset of sustained core He burning"), TACHeB (Yc = 10^−4), TP-AGB, PostAGB, WDCS (Coulomb coupling Γ); EEPs because tracks of different mass "are likely to have (perhaps vastly) different lifetimes and numbers of timesteps" — https://ar5iv.labs.arxiv.org/html/1601.05144 (accessed 2026-10-08)
- E8 · Paxton et al. 2011 (MESA): "MESA star solves the fully coupled structure and composition equations simultaneously", with "sophisticated timestep controls" — a 1D hydrostatic code whose steps follow the evolution, not the sound crossing — https://ar5iv.arxiv.org/html/1009.1622 (accessed 2026-10-08)

**Multi-scale time stepping in simulation codes**
- E9 · Gear & Kevrekidis 2003, SIAM J. Sci. Comput. 24(4) 1091–1106: "there exist classes of explicit numerical integration methods that can handle very stiff problems if the eigenvalues are separated into two clusters, one containing the 'stiff,' or fast, components, and one containing the slow components" (projective integration: small steps, then a giant extrapolated step) — https://collaborate.princeton.edu/en/publications/projective-methods-for-stiff-differential-equations-problems-with/ (accessed 2026-10-08)
- E10 · Gnedin & Abel 2001 (OTVET): in the Newtonian limit "it actually does not matter what the specific value of the speed of light is as long as v/c ≪ 1"; for reionization "we can adopt a value for ĉ as low as 1,000 km/s, because typical gas velocities during reionization do not exceed 100 km/s" — a fast clock squeezed to ~10× the slow one — https://ar5iv.labs.arxiv.org/html/astro-ph/0106278 (accessed 2026-10-08)
- E11 · Gnedin 2016, *On the Proper Use of the Reduced Speed of Light Approximation*: used "as originally designed", the RSL approximation "remains a highly accurate numerical method"; results insensitive while ĉ stays above ~10 % of c — https://arxiv.org/abs/1607.07869 (accessed 2026-10-08)
- E12 · Hotta, Rempel, Yokoyama, Iida & Fan 2012: "Numerical simulations with artificially reduced speed of sound are a valid approach as long as the effective Mach number (based on the reduced speed of sound) remains less than 0.7" — a larger step bought by shrinking a gap, explicit, no global solve — https://arxiv.org/abs/1201.1061 (accessed 2026-10-08)
- E13 · Springel 2005 (GADGET-2): "we discretise the timesteps in a power of 2 hierarchy, where all timesteps are a power of 2 subdivision of a global timestep"; "Particles may always move to a smaller timestep, but to a larger one only every second step"; "The long-range PM force has a comparatively large timestep, which is sufficient for the slow time-variation of this force" — https://ar5iv.labs.arxiv.org/html/astro-ph/0505010 (accessed 2026-10-08)
- E14 · Dursi & Zingale 2003, *Efficiency Gains from Time Refinement on AMR Meshes and Explicit Timestepping*: time sub-cycling on Berger–Colella meshes; "Potential efficiency benefits from TR on these meshes are seen to be quite limited except in the case of refining a small number of points on a large mesh" — https://arxiv.org/abs/astro-ph/0310891 (accessed 2026-10-08)

**The GPU API**
- E15 · WebGPU default limits: maxComputeWorkgroupStorageSize 16384 bytes, maxComputeInvocationsPerWorkgroup 256, maxStorageBuffersPerShaderStage 8, maxComputeWorkgroupsPerDimension 65535 — https://developer.mozilla.org/en-US/docs/Web/API/GPUSupportedLimits (accessed 2026-10-08)
- E16 · `dispatchWorkgroupsIndirect()` dispatches a grid whose X, Y, Z sizes are read from a GPU buffer ("three 32-bit unsigned integer values … 12 bytes total") — the GPU can decide how much work a dispatch does — https://developer.mozilla.org/en-US/docs/Web/API/GPUComputePassEncoder/dispatchWorkgroupsIndirect (accessed 2026-10-08)

**Measured on linux-pc (2026-10-08; wgpu 30.0.1, Vulkan, NVIDIA driver 595.99.02, Mesa 25.2.8; WebGPU default limits; log: logs/M0-R3.log)**
The bench is a **proxy** of M8's step, not the game's scheme: 4 hydro passes over 14 f32 channels
(radius-2 stencil, a sqrt per channel); gravity as pack + zero-padded radix-2 FFT in workgroup memory
(rows) + tiled transposes, forward × kernel × inverse — 6 dispatches; 16 RKL2-shaped heat stages
(~24 B per cell); one burning pass (8 sub-cycles of exp/pow in registers); a CFL max-reduction and
its finalise — **29 dispatches per step**. Each figure: wall time per step, median of 15 submits of
25 steps after 3 warm-ups (GPU timestamps agree within 2–40 %; listed in the log).

| | Quadro RTX 4000 (the mid-range card) | RTX 5090 | Intel UHD 770 (laptop-class proxy) |
|---|---|---|---|
| M1 · per dispatch (1000 one-workgroup dispatches, one pass) | 2.65 µs (GPU 2.12) | 1.92 µs (GPU 0.87) | 2.45 µs (GPU 1.71) |
| M2 · whole world 600 × 400, FFT 2048 × 1024 — full step | **2.13 ms** — hydro 0.58, gravity 1.24, heat 0.30, burn 0.14, CFL 0.03 | 0.27 ms | 14.4 ms |
| M3 · massive-star box 160 × 160, FFT 512 × 512 | **0.29 ms** — gravity 0.17 | 0.12 ms | 1.92 ms |
| M4 · Sun-like box 64 × 64, FFT 128 × 128 | **0.11 ms** — each part 0.01–0.04 | 0.10 ms | 0.39 ms |

CPU encoding: ~0.03 ms per step on every card (M2–M4) — not a bottleneck. The Sun-like box costs
nearly the same on the 5090 as on the Quadro (0.10 vs 0.11 ms): small boxes are bound by the
dispatch count, not the card (my arithmetic: 29 × 2.65 µs = 0.077 ms of the Quadro's 0.113).

## Problem
**1 · The squeeze, written out.** Real time scales (star_physics.md `## Stages`): a core collapse
takes "a few milliseconds" (SP-E3), the Sun-like life from cloud to white dwarf ~11 Gyr ≈ 3.5 × 10^17
s (SP's arithmetic from SP-E2) — **~20 orders of magnitude** (my arithmetic). The lead's sandbox clock
(answers 1, 5, 6): slow motion "about 10 times slower than normal", a life of "some minutes" at
normal speed, "about 10 seconds" at the top step. The wall clock thus spans ~×0.1 to ~×10²; the
sandbox's own time spans only what K1–K3 force (below, ~3–4 orders between the core's dynamical
time and the life); **"billions of years" lives in the readout only** (B10, F1). The readout's rate
— real years per wall second — runs from ~1.3 Gyr/s (the main sequence at the top step: 9 Gyr over
~6.7 s, my arithmetic) to ~10^−3 real seconds per second (a few-ms collapse stretched over seconds
at ×0.1): ~19 orders, carried by the stage clock (`## Recommendation`, R7), never by the physics.

**2 · Steps per life — the quantity the frame budget pays for.** sim_models.md's worked example:
a settled Sun-like preset (R = 15 cells) takes **150 steps per τ_dyn** under CFL 0.4, and in general
5–10 × R/Δx — the sound speed cancels (SM, CFL from SM-E12). The life's length in τ_dyn is set by the
gaps between the clocks (K1), the order of the burning stages (K2) and "heavier burns faster" (K3):

| Scenario | What it assumes | Sun-like MS | Whole life (cloud → white dwarf) | Steps per life | Steps per frame, life in ~10 s (600 frames) |
|---|---|---|---|---|---|
| **L** (R2b's) | K1 gaps of 10 at each step: τ_KH = 10 τ_dyn, τ_nuc = 10 τ_KH | 100 τ_dyn | ~150 τ_dyn (pre-MS 10 τ_dyn + post-MS ~0.4 MS — opinion) | ~22,500 | **~38** |
| **C** | + K1 for core He burning too (steady burning needs τ_nuc ≥ 10 τ_KH in every burning stage) and K2's H > He by ≥ 3 (real: ~75 for 1 Sun, 5.5 at 15 Suns, SP-E2, SP-E3) | 300 τ_dyn | ~420 τ_dyn | ~63,000 | **~105** |
| **H** | + K3 against the massive preset: R = 60 cells (SM's onion item) and 20× the mass give τ_dyn,heavy = (60/15)^1.5/√20 = **1.79** τ_dyn,Sun in the sheet (τ_dyn ∝ R^1.5 M^−0.5 from SM's c² = αGM/2R); its own MS ≥ 300 of its τ_dyn = 537 Sun τ_dyn; the Sun-like MS ≥ 3× that (K3 kept visible, factor 3 — opinion; real 20^2.5 ≈ 1,800, SP-E10) | ~1,600 τ_dyn | ~2,240 τ_dyn | ~340,000 | **~560** |

All my arithmetic; the gap of 10 is precedent, not proof — the reduced-speed-of-light method keeps a
fast clock ~10× the slow one (E10, E11) and the reduced-sound-speed method a Mach number below 0.7
(E12) — and post-MS stages may cost more steps per unit time (a contracted, hotter core shrinks the
global CFL step), a factor not counted here (`## UNVERIFIED`). The gap is oracle SM-O14's to measure.

**3 · The top speed's frame budget, on the mid-range card (I4, B14, B33).** 60 frames a second is
16.6 ms per frame; drawing the cells, the glow and the panels is reserved **~3 ms** (opinion, not
measured), leaving **~13 ms for the simulation**. Steps per frame that fit, from M1–M4:

| Work per step (Quadro, measured) | ms per step | Steps per frame in 13 ms | L (38) | C (105) | H (560) |
|---|---|---|---|---|---|
| Whole world, gravity every step (M2) | 2.13 | **6** | 6× over | 17× over | 92× over |
| Whole world, gravity every 4th step (M2 − ¾ × 1.24, my arithmetic) | 1.20 | 10 | 3.5× over | 10× over | 52× over |
| Massive-star box (M3) | 0.29 | 44 | held | 2.4× over | 13× over |
| **Sun-like box (M4)** | **0.11** | **115** | **held (4.3 ms)** | **at the edge (11.9 ms)** | 4.9× over |
| Floor: 29 dispatches × 2.65 µs (M1) | 0.077 | 169 | held | held | 3.3× over |

So: **R2b's ~22 ms per frame was optimistic about the step (2.13 ms measured, not 0.88) and about the
life (L is the smallest scenario).** Whole-world computing cannot hold any scenario. The top speed is
reachable only by computing where the matter is (the Sun-like star covers ~700 of 240,000 cells, my
arithmetic from R = 15), and then only in L and C. The Sun-like box applies to the main sequence —
most of the steps; the cloud before collapse (a = 40, a ~96-cell box) and the red giant (a swelling
envelope) sit between M3 and M4. On a laptop-class iGPU (M4, Intel: 0.39 ms) only ~33 steps fit:
even L falls short there — the web build and laptops are past M0's promise (answer 11), and the
honest indicator below (R3) covers them.

## Prior art
- **Game sandboxes — speed as accuracy's price.** Universe Sandbox's players report that speeding up
  "makes it less accurate" and choose integrators for long runs (E1); it simulates stellar evolution
  (E2); its wiki describes an automatic limit on the simulation speed to keep accuracy, with the
  limiting object shown (`## UNVERIFIED` — the page refused the fetch). Kerbal Space Program's
  on-rails warp, where only orbits are computed and warp is refused under acceleration, is the
  classic "reduced model during a warp" — UNVERIFIED for the same reason. RimWorld "disallows and
  grays out the speed up time functions at the start of a dangerous event" (E3) — B11's slow-down,
  without B11's off switch. Fiedler's accumulator decouples the physics step from the frame, caps
  the steps to avoid the "spiral of death", and interpolates the drawn state (E4) — the frame loop
  this report recommends.
- **Stellar-evolution codes — stage clocks for the readout.** MESA steps a 1D hydrostatic star on
  the evolution's own time scale (E8): its steps follow the nuclear and thermal clocks, never the
  sound crossing — the reason a real code covers a life in thousands of steps, and the reason a
  hydrodynamic sandbox cannot. SSE's analytic formulae give each phase's duration as a function of
  mass to within 5 % of detailed models (E5) and keep the **fraction** of the current phase when the
  mass changes, t′ = (t′_MS/t_MS) t (E5), as BSE's rejuvenation does for mass transfer (E6). MIST's
  EEPs index a life by **physical milestones** (ZAMS at 99.9 % of luminosity from H burning, TAMS at
  Xc = 10^−12, the RGB tip at peak luminosity …) rather than by age, because stars of different mass
  have "vastly different lifetimes" (E7). All three are published formulas and definitions — no
  code is reused (R7).
- **Multi-scale time stepping.** Hierarchical individual steps — GADGET-2's power-of-two rungs, and
  its slow long-range force on a larger step than the short-range one (E13) — and Berger–Colella
  sub-cycling with refluxing, whose gains are "quite limited" unless the fine region is a small
  part of the mesh (E14). Shrinking a gap to buy a step: the reduced speed of light (E10, E11) and
  the reduced speed of sound (E12), each valid while the squeezed clock stays ~10× (or Mach < 0.7)
  away from the next. Projective integration: explicit small steps, then a giant extrapolated step,
  valid when the spectrum has a gap between fast and slow modes (E9) — exactly the gap K1 asks for.
- **The API.** WebGPU's default limits — 16 KB of workgroup memory, 256 invocations, 8 storage
  buffers per stage (E15) — fit a 2048-point complex f32 FFT row exactly, and the bench ran within
  them; indirect dispatch lets the GPU itself zero out work (E16), which the event latch uses.

## Options
Common to all: the CFL step stays the stability limit (SM-E12, M3); speed rungs change only how many
steps are run per frame. Costs are from `## Problem` §3.

| Option | What the player sees during a warp | Cost (Quadro, measured or my arithmetic) | How it fails | The test that catches it |
|---|---|---|---|---|
| **A · Adaptive global step** — one Δt for the world from the CFL max (a GPU reduction, 0.03 ms, M2); speed = sandbox time per wall second; steps per frame = speed × 1/60 ÷ Δt, the remainder carried (E4) | Everything moves, every frame; the age readout and a years-per-second rate run continuously; slow motion drawn by interpolating two states (E4) | Whole world: 2.13 ms/step → 6 steps/frame (M2) — holds no scenario alone | Spiral of death when steps exceed the budget (E4); a silent cap if the budget clamp is hidden | Warp invariance (T1 below); a budget test asserting the "behind" indicator shows whenever the clamp binds |
| **A2 · Active box** (with A) — compute only the bounding box of non-vacuum cells plus a margin; gravity by a zero-padded FFT over that box, exact for the same masses (SM-E8) | Identical to A | Sun-like 0.11 ms (M4), massive 0.29 ms (M3) → 115 / 44 steps per frame | Matter outruns the margin between box updates (cut flux); vacuum cells evolving under gravity break the box ≡ world equivalence | T2: a box run vs a whole-world run of the same preset — mass, momentum, energy, element masses within SM-O10's tolerances; a fast blob aimed at the box edge never loses mass |
| **A3 · Multi-rate gravity** (with A) — re-solve the potential every k steps while it changes slowly (GADGET-2's long-range force on a longer step, E13) | Identical to A | Gravity 1.24 → 0.31 ms per step at k = 4, whole world (M2, my arithmetic); proportional in a box | Energy drift from a stale potential; a missed fast collapse if k is not cut at once | T3: energy ledger (SM-O10) drift ≤ 10^−3 per τ_dyn vs k = 1; k forced to 1 while any collapse detector is armed |
| **B · Per-region sub-cycling** — cells on power-of-two rungs of Δt (E13), Berger–Colella with refluxing (E14) | Identical to A | Gains "quite limited" unless the fine region is small (E14): the work sits in the star, where c is highest; each rung is its own dispatch set and small boxes are already dispatch-bound (M1, M4) | Flux mismatch at rung boundaries breaks conservation without refluxing; a fast signal entering a slow region goes unstable | A pulse crossing a rung boundary: mass and energy conserved to 10^−6; the result vs A within SM-O5's tolerance |
| **C · Settled star → reduced model and back** — a 1D hydrostatic, MESA-like star (E8) while it is quiet; back to cells on any touch or event | The star frozen as a picture repainted from the 1D model; readouts run; touching it (B7) snaps it back | Cheapest life — steps follow the evolution (E8) — but two implementations of every law to build and keep equal | The star jumps at a switch (radius, temperature); energy or element mass lost at a switch; a missed event the 1D model lacks; two clouds (B15) cannot be one 1D star | A zero-time round trip cells → 1D → cells within 2 % in radius and 10^−6 in mass; the EEP timeline (E7) of a 1D-assisted life vs a full one |
| **D · Event-driven projective jumps** — in quasi-static phases: n real steps, then extrapolate the slow variables (composition, thermal content) over a giant step (E9), then relax | The star keeps moving during the inner steps; composition and the readouts advance in increments | Steps per life ÷ the jump ratio (÷5–20 in the MS, opinion) — would bring C and H inside the M4 budget | A missed event (a threshold crossed inside a jump); instability if the fast modes have not damped (the gap too small, E9); a touch mid-jump | A projective vs a fully stepped MS segment: Xc(t) within 1 %, radius within 2 %; a jump straddling a scripted ignition threshold is clipped and the event caught; element masses conserved to 10^−6 |

Rejected for M0 (opinion, on the evidence above): **B** — its gain lives where M0 has none (E14,
M4), and it multiplies dispatches; **C** — two copies of the physics, a star that can jump, and
"touch any time" (B7) forcing constant switches; it also trades "every stage from the physics" for a
1D stand-in.

## Recommendation
**R1 · The design: A + A2 + A3** — one adaptive global step, the work limited to an active box,
gravity re-solved every k steps in quiet phases. Licence: no third-party code beyond the ruled
wgpu (MIT / Apache-2.0, ES-E1); the methods are published (E4, E13). Cost: measured, M1–M4. What
would falsify it: O14's measured gap putting a Sun-like life above ~70,000 steps (115 steps per frame
× 600), or a red-giant box costing more than ~0.12 ms per step.

**R2 · Time is defined so that warps never touch the physics (I2).**
- Δt is always the CFL step computed on the GPU from the state; no speed, rung, slow-down or frame
  rate ever changes it. The simulation never reads the wall clock: each frame asks for 1/60 s ×
  speed of sandbox time, the remainder carried to the next frame (E4); slow motion that asks for
  less than one step per frame runs a step every few frames and **draws the interpolation** between
  the last two states (E4).
- Steps per frame N = ⌈speed × 1/60 ÷ Δt_est⌉, Δt_est read back from the GPU a fixed two frames late,
  N clamped to the measured budget N_max (13 ms ÷ the box's cost per step). Box changes and the
  gravity cadence k are decided from the state at fixed step indices, never from the frame.
- **The cap is never silent:** when N_max binds, the speed readout shows both — "×30 (running ×12)"
  (B33's "never a silent cap"; the spiral of death avoided by the clamp, E4).
- Consequence, the test **T1 · warp invariance**: the same preset run K steps at ×0.1, ×1 and the top
  rung, with the slow-down on and off, gives a **bit-identical** state on one card (states differ only
  in how steps were grouped into frames).

**R3 · The speed rungs, pause and single step (B8, B9).**
- ×1 is anchored on the dynamical clock, not on the life: **1 τ_dyn of the settled Sun-like preset
  per wall second** (~150 steps a second, ~2.5 per frame) — a collapse or a bounce is watchable at ×1
  and slow at ×0.1; the life at ×1 then follows the gap: 2.5 min (L), 7 min (C), 37 min (H) — L and C
  are the lead's "some minutes" (answer 6). Opinion; M0-TC fixes.
- Rungs: a 1-3-10 ladder **×0.1 · ×0.3 · ×1 · ×3 · ×10 · ×30 · ×100** on the number keys (the
  example the lead saw was ×0.1, ×1, ×10, ×100 — B8); the top rung offered is the first that gives a
  Sun-like life in ~10 s (×15 for L → ×10 or ×30; ×42 for C → ×30 or ×100; my arithmetic) — M0-TC
  fixes it once O14 measures the gap.
- Pause stops stepping; single step runs **one simulation step** (one CFL Δt) while paused — the
  Powder-Toy tick the question's premise named (answer 5).

**R4 · The automatic slow-down (B11, I2): recognised from the physics, enforced by a GPU latch.**
- Detectors ride the per-step CFL reduction (0.03 ms, M2) and read only the physics state:
  **ignition** — a star's burning power passes a share of its radiated luminosity, rising (after
  MIST's ZAMS criterion, E7) → drop to ×1; **core collapse** — central density rising on the core's
  dynamical time and inward core velocity above a fraction of the local sound speed, or the
  neutronization record (SM M6) firing → ×0.1; **explosion** — mass moving outward faster than escape
  speed with kinetic energy rising past a share of the binding energy → ×0.1. Thresholds: M0-TC.
- **Why a latch:** a core collapse spans ~25–50 steps in the sandbox (5–10 × a ~5-cell core, my
  arithmetic from SM's steps per τ_dyn), fewer than one top-speed frame (38–105 steps); a CPU readback
  is 1–2 frames late. So a small controller dispatch per step writes the next step's indirect
  dispatch sizes (E16), zeroing them once a detector fires while the slow-down is on; the CPU then
  sets the new rung and releases the latch. The event is caught within one step. Δt is untouched, so
  T1 still holds.
- The warp **stays** at the dropped rung until the player changes it (opinion: predictable; the
  picked option says only that it "drops", answer 8) — auto-return is a small fork for M0-TC. Each
  event fires once per star, with hysteresis. **The off switch** disables the latch; the warp then
  runs through events at the chosen rung, the "running ×N" indicator showing any shortfall.
- What the screen shows: a banner "Slowed for: core collapse — ↑ to resume ×30" (opinion).

**R5 · The conserved quantities across every switch, and their tests.** The recommended design has
no model switch; its switches are the rung, the latch, the box resize and the gravity cadence.
- Rung changes and the latch: T1 (bit-identical).
- Box resize: **mass, momentum, energy and each element's mass** identical to round-off across a
  resize step (only vacuum cells enter or leave) — T2, 10^−6 relative on f64 sums (SM-O10's form).
- Gravity cadence: the energy ledger kinetic + thermal + W + radiated + neutrino-lost + escaped −
  nuclear released (SM-O10) — T3, drift ≤ 10^−3 per τ_dyn against k = 1.
- If D (R6) is built: element masses exact across a jump, energy within T3's bound, Xc(t) within 1 %
  of a fully stepped run.

**R6 · The fallback, designed now: D (projective jumps) in quasi-static phases**, if O14 lands in C
or H. It keeps "every stage from the physics" (the same equations, a different integrator, E9;
opinion), keeps the latch (a jump is clipped before any detector threshold), and ends at once on a
touch (B7).

**R7 · The age readout (B10, B31): a stage clock, confirming star_physics.md's proposal, made
monotonic.**
- Stages are recognised by EEP-like physical milestones (E7) the labels already watch (I2): PreMS,
  ZAMS, IAMS, TAMS, RGB tip, ZACHeB, TACHeB, TP-AGB, post-AGB, white dwarf; for massive stars the
  C, Ne, O, Si ignitions, collapse, bounce, remnant.
- Age = Σ real durations of the passed stages + φ × the current stage's real duration, φ the stage's
  progress from the physics (MS: φ = (X_c0 − X_c)/(X_c0 − 10^−3), SP-E2's TAMS); durations as functions
  of the current mass from SSE's formulae (E5) or star_physics.md's tables.
- **Monotonic and jump-free:** age is integrated per frame, d(age) = dφ × τ_stage(M), with dφ < 0
  clamped to 0; a mass change keeps the fractional age, SSE's t′ = (t′_MS/t_MS) t (E5) and BSE's
  rejuvenation (E6) — so painting gas onto a shining star never makes the clock jump.
- Beside the age, a **rate**: "1 s ≈ 1.3 Gyr" during the main sequence, "1 s ≈ 1 ms" during a
  collapse at ×0.1 (my arithmetic, `## Problem` §1) — the squeeze shown, not hidden. Collapse and
  bounce add their real seconds (SP-E3, SP-E8) to a banner clock, not the year count. The inspector's
  units stay M0-TC's (I6).

**R8 · Verdict on the top speed (B33): not shown held — the shortfall is a fork for M0-TC (I4).**
Held at 60 frames a second on the Quadro only with A2 and only if the gap is small: L held (4.3 ms
of 13), C at the edge (11.9 ms), H short by ~5× (64 ms). Whole-world computing holds none (M2).
- **F1 · for M0-TC, with the lead — the top speed when the measured gap exceeds the budget.** Decision
  point: the first build lot that runs a preset to the end of its main sequence measures steps per
  life (SM-O14) and the box's cost per step on the Quadro. If they exceed 13 ms per frame at a ~10 s
  life: (a) **fewer frames a second during the top rung only** — 30 frames doubles the budget to
  ~29 ms (covers C) [recommended first: the promise "a life in ~10 s" stays]; (b) **D, projective
  jumps** (R6) — keeps 60 frames and ~10 s, at the cost of a second integrator and its tests (covers
  C and H); (c) **a lower top speed** — the rung where 13 ms holds (a life in ~25 s in C if the box
  grows to the massive size, ~50 s in H with the Sun-like box — my arithmetic from M3, M4); note "a life in ~1 min" was an option the lead did not pick (B28), so (c) re-opens answer 6 —
  a new ask, never a silent change.
- **F2 · for M0-TC — small forks:** auto-return after a slow-down (recommended: no); the ×1 anchor
  (recommended: 1 τ_dyn of the Sun-like preset per second); the rung ladder (recommended: 1-3-10);
  the render reserve (3 ms, to be measured by the first drawing lot).
- **For M0-TG:** box computing (A2) and the event latch (R4) are first-lot architecture, not late
  optimisation — whole-world computing cannot reach any scenario (M2); fusing kernels to fewer
  dispatches is the lever for small boxes (M1, M4: the floor is 0.077 ms of 0.113).

## UNVERIFIED (refuted by default until a source confirms)
- **The K1 gap of 10** and the scenarios' factors (post-MS ~0.4 MS, H/He ≥ 3, K3's factor 3): opinion,
  with precedent only (E10–E12); O14 measures the gap. The life's step count (22,500–340,000) rests on it.
- **Post-MS steps per unit time** — a contracted core raises the global CFL limit's sound speed; the
  factor is not estimated. → the first lot that reaches a red giant measures it.
- **The bench is a proxy**: its kernels have M8's traffic and dispatch count, not the real scheme's
  arithmetic; a real Riemann solver or burning network may cost more. The FFT is radix-2 at 2048 ×
  1024 with per-butterfly cos/sin and no zero-skipping — a mixed-radix 1200 × 800 real-to-complex FFT
  should cost less, unmeasured. GPU timestamps and wall times differ by up to 40 % on the small boxes
  (submission overhead); wall times are used.
- **Render and UI reserve of ~3 ms** — not measured (no drawing code exists).
- **WebGPU in a browser**: per-dispatch overhead through Chrome's Dawn is not measured; the Intel iGPU
  under Vulkan (M2–M4) is a laptop-class proxy, not the web build.
- **The controller dispatch and indirect dispatch's cost** (R4) — not measured.
- **The core collapse's step count** (~25–50) — my arithmetic from SM's 5–10 × R/Δx with an assumed
  ~5-cell core.
- **Universe Sandbox's automatic speed limit**: the wiki page (https://universesandbox.fandom.com/wiki/Sim_Settings_Menu)
  refused the fetch (HTTP 402); its content here is a search engine's summary — "Auto Limit
  Simulation Speed" keeps accuracy, the limiting object is displayed. Not used for any decision.
- **Kerbal Space Program's warp**: its wiki (https://wiki.kerbalspaceprogram.com/wiki/Time_Warp)
  refused the fetch (bot protection); "on rails" warp to 100,000× that stops all physics but gravity,
  and a 1×–4× physical warp — a search summary. Not used for any decision.
- **The sheet's τ_dyn scaling** (τ_dyn ∝ R^1.5 M^−0.5) is my arithmetic from SM's uniform-disk virial,
  itself UNVERIFIED there.
