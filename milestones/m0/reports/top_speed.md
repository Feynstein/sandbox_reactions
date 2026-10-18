# M0-TJ2 — the top speed: as fast as the card goes (2026-10-18)

Box **win-laptop** (Laser2025-20, Windows 11, Git Bash; physics on the NVIDIA GeForce RTX 4080 Laptop GPU, Vulkan) ·
2026-10-18, from 11:25 EDT (`plan.py show`'s baseline) · model=claude-opus-5-5 level=max (`rung_record.py now` at launch —
the heading's rung) · tree 454209d plus M0-D50/D55/D56's uncommitted work, release binary current (`cargo build --release`:
nothing to rebuild). Woken by M0-V8 (reports/v8.md § M0-TJ2), not by M0-T113: the plan's condition, a projected frame
cost over 33.3 ms at a ~10 s life, held before G-TOP was built. Raw output: `logs/M0-TJ2.log`; commands and edits:
`tasks/M0-TJ2.md`.

## Ruling — the questions as asked, and the answers (the record; PLAYBOOK §12)
One call of the question tool, three questions, the recommendation first. Before it, the lead read: a Sun-like life is now
about 200,000 steps (M0-D50's star, extrapolated, unproven); once the star burns the engine computes the whole 600 × 400
world, ~10 ms a step on the Quadro and 17.4 ms on the RTX 4080 Laptop; so the Quadro fits about one step a frame and the
burning star runs at ~0.7× normal speed at any setting, a life in about an hour; a ~10 s life needs ~670 steps a frame at
30 frames a second, each step ~240× cheaper. Picked: **bold**, ← picked; a typed answer quoted whole.

**Q1 · Top speed** — «A ~10 s Sun-like life can't hold on the mid-range card (the Quadro RTX 4000): it's about 240× too
slow at 30 frames a second, and once the star burns the card runs it at only ~0.7× normal speed. What should M0's top
speed be?»
- «As fast as the card holds (Recommended)» — «The top speed becomes the fastest the Quadro keeps up at 30 frames a second
  over a whole Sun-like life, measured by lot 16's frame-rate test. It is never a speed the card can't hold. The ~10 s life
  and shortcut jumps move to a later milestone, recorded. If yes, M0 never promises a 10-second life: a life takes as long
  as the card needs, about an hour today and maybe a few minutes if questions 2 and 3 pay off (unproven).»
- «A life in ~1 min» — «The top speed is set for a Sun-like life in about a minute (the interview option you didn't pick).
  That needs both a shorter life (question 2) and a ~3–5× cheaper step (question 3), neither proven. If lot 16 still misses
  it, the build stops for you again near M0's end.»
- «Shortcut jumps, keep 10 s» — «A new lot after lot 14 builds 'projective jumps': in calm stretches the engine skips
  ahead many steps at once, then lets the star settle. That's about 6–9 more tasks and a second way of advancing time,
  with its own tests. It still doesn't reach 10 s: the cloud's collapse alone (~3,000 steps, before any calm stretch)
  takes ~20–25 s on the card.»
- «Keep the build as is» — «No change: the top speed stays labelled for a ~10 s life. The card runs it at ~0.7× normal
  speed once the star burns, and the speed readout shows that. On the mid-range card the speed keys above ×1 then do
  nothing; the 10-second check stays red and is reported at M0's end.»
- **Typed:** **«we go as fast as the card goes. We dont compromise physics over performance for now, maybe later in another
  milestone will we optimize further but not now, we do not block anything on performance»** ← answered

**Q2 · Life length** — «A life is ~200,000 steps because the star's clocks are far apart: it could shine on its stored
heat for ~134 'bounce times' (~1 time unit each), and its fuel lasts ~26 times longer still. These are estimates; the
contract only asks for 10× each (G-SQUEEZE). Should lot 14 tune the physics so a life takes as few steps as those rules
allow?»
- «Yes, shortest life (Recommended)» — «Lot 14's clock task (M0-T98a) aims each gap at about 10–20×, near the contract's
  minimum, instead of only checking it's at least 10×. That could mean up to ~10× fewer steps per life (unproven), so a
  life at normal speed takes minutes instead of ~40 min. Risk: the burning balance is delicate (M0-D50), so the gain may be
  smaller.»
- «No, keep lot 14's aim» — «Lot 14 only checks that the gaps stay at least 10× (today's already do). A life stays
  ~200,000 steps, and the time a life takes at top speed is set by the step cost alone.»
- **Typed:** **«I would like to maybe the user have a choice on time step length or something, if possible. But we go with
  shortest life otherwise. »** ← answered

**Q3 · Step cost** — «Each step of the burning star costs ~10 ms on the Quadro (about 100× the time-warp study's
estimate) and 17 ms on your laptop's RTX 4080 (measured today). Two reasons: the whole world is computed, because thin gas
thrown out at the collapse keeps the active area full-size, and the heat solve is costly. Should M0 add a task to make
each step cheaper on the mid-range card?»
- **«Yes, one cost task (Recommended)»** — «One task on the Linux PC measures where the Quadro's ~10 ms goes and builds the
  cheapest cut that leaves the physics unchanged. A cut that would change the physics (for example, freezing thin gas far
  from the star) comes back to you as a question. That's one or two more tasks before lot 16, for maybe 2–5× faster steps
  (unproven).» ← picked
- «No» — «The step cost stays as it is in M0, and the gravity-cadence task stays deferred. The top speed then stays near
  normal speed on the mid-range card.»

### My reading (labelled — the words above are the record)
1. **«we go as fast as the card goes»** — the top rung becomes **Max**: on whatever card runs the game, each frame runs
   N_max steps (§1.8.3's budget from the measured cost per step) at 30 frames a second — Q2's FROZEN «30 pictures/s at top
   speed» kept — with no sim time owed, and the speed readout always shows what the card advanced. This is a per-card top
   speed, not the shown option's single number measured on the Quadro: the typed words name «the card», and a Quadro
   number would hold a faster card below what it goes. Cheap to reverse: a PLAN block can turn Max back into a measured
   TOP. TOP (life_sun_sb ÷ 10 s) stays, only as the cap of the numbered rungs (×0.1 … ×1000).
2. **«We dont compromise physics over performance for now»** — nothing that changes the physics is built for speed in M0:
   no shortcut jumps (TW R6), no gravity cadence (§1.4.4, M0-T114 N/A), no fixed slower top speed; the cost task's cuts
   change no physics.
3. **«maybe later in another milestone will we optimize further but not now»** — the optimisations are routed past M0
   (§ Routed), through M0-TZ.
4. **«we do not block anything on performance»** — §1.8.6's « the build stops and the lead is asked again » is retired; G-FPS's
   frame times and G-TOP's life wall time are measured and reported beside their targets, never a NO-GO and never a D;
   what stays graded in both is the controller (no frame plans more than its N_max; Max plans exactly N_max) — a
   correctness property, not a speed. The 60-frames target stays the goal's target, reported.
5. **Q2** — «we go with shortest life otherwise» re-aims M0-T98a at the shortest Sun-like life G-SQUEEZE's bounds allow
   (each gap as near 10 as holds, never under it). «the user have a choice on time step length or something, if possible»
   is an idea outside M0's scope, so PLAYBOOK §2.6 P9 applies: its words land verbatim in § Routed, tier later, no code in
   M0. Whether it is possible, as I read it: a step longer than the CFL step the physics computes (§1.8.7) makes the gas
   unstable, so not as a setting; a choice of life length is a choice between sets of constants, each tuned like lot 14 and
   each with its own calibration and whole-life checks (G-CAL) — possible, at a cost that multiplies lots 9, 14 and 15; a
   «skip the calm stretches» setting needs shortcut jumps first. The speed rungs already let the player choose how fast
   time runs.
6. **Q3** — M0-T124, one cost task on linux-pc. The picked option said a physics-changing cut «comes back to you as a
   question»; Q1's typed words answer that question for M0 (not now), so M0-T124 builds none, routes each such lever past
   M0 and names it with its measured gain in its handoff, where the lead can still pull one in — information, not a
   blocking question.

## The measured numbers (what the question rested on)
| Quantity | Value | Where, how |
|---|---|---|
| The Sun-like preset's start | ignition at step 769 (sim 9.83); `cloud` from 1,512; `main_sequence` from step 3,000 (sim 36.15) | run 1 below, summary.json |
| Δt in the main sequence | 0.011511 (step 3,000), 0.011418 (4,000), 0.011394 (5,000) — 88 steps a t.u.; τ_dyn ≈ 1 t.u. (reports/ignition.md) | the dumps' `resume.dt` |
| The active box | 360 × 352 at step 1,000, 552 × 400 at 2,000, the whole world 600 × 400 from 3,000 on | the dumps' `resume.box` |
| X_H,c's fall | 0.7247 (3,000) → 0.7067 (8,000): 3.6 × 10⁻³ per 1,000 steps | M0-D50, reports/ignition.md |
| Steps per life (UNVERIFIED) | X_H,c 0.35 at ≈ 1.05 × 10⁵ steps (≈ 1,200 t.u.); 10⁻³, the core's hydrogen gone, at ≈ 2.0 × 10⁵ (≈ 2,300 t.u.); the life after it not measured | linear from the line above, my arithmetic |
| The burning star's step, RTX 4080 Laptop | **17.392 ms** (17.392 · 17.377 · 17.621); per pass, medians: P4 8.065 · P5 5.937 · P6 1.425 · P2 0.744 · P3 0.678 · P8 0.207 · P9 0.096 · P7 0.058 · P0 0.028 · P1 0.012; wall 4.33–4.44 s a run | 3 × 200 steps from s_5000, GPU timestamps |
| The first 5,000 steps' mean, RTX 4080 Laptop | 20.760 ms (P5 10.477, P4 7.055); 158.2 s wall | run 1's summary.json |
| The whole world's step, Quadro RTX 4000 | 10.709 ms (P4 4.166, P5 3.287, P6 1.269, P2 1.107) — the shipped constants before M0-D50 | M0-T63, reports/step_cost.md |
| The burning star's step, Quadro | not measured — [NOT RUN — owed on laserax-ai], M0-T124's | — |

The runs (win-laptop, RTX 4080 Laptop; `logs/M0-TJ2.log`):

[ALREADY RUN — PASS (exit 0, 5,000 steps, sim 59.025, 158.2 s) on win-laptop]
```
build/target/release/sandbox-reactions.exe headless --scene preset:sun --steps 5000 --dump-every 1000 --timing --adapter "RTX 4080" --out build/m0-tj2-sun
```
[ALREADY RUN — PASS (3 runs, exit 0, medians above) on win-laptop]
```
build/target/release/sandbox-reactions.exe headless --scene preset:sun --load-dump build/m0-tj2-sun/dumps/s_5000.bin --steps 200 --timing --adapter "RTX 4080" --out build/m0-tj2-at5000-1
```
(and `-2`, `-3`). The first try of run 1 exited 127 before the binary started: `/usr/bin/time` is absent in Git Bash.

The arithmetic (mine; every life count UNVERIFIED):
- **A ~10 s life at 30 frames a second** is 300 frames: ≥ 2.0 × 10⁵ steps → ≥ 670 steps a frame. The simulation's share of
  a 33.3 ms frame is 30.3 ms (§1.8.3's R_reserve 3.0), so a step would have to cost ≤ 0.045 ms — ≈ 240× under the
  Quadro's whole-world 10.709 ms, under M0-R3's dispatch floor (0.077 ms). The old TOP would read ≥ ×230.
- **What the Quadro runs once the star burns** (the whole world, 10.709 ms): N_max = 1 at 60 frames (⌊13.67 / 10.709⌋)
  and 2 at 30 (⌊30.33 / 10.709⌋) — 60 steps a second either way, 0.68 t.u. a second: «running ×0.68» at ×1 and above, a
  life of ≥ 2,300 t.u. in ≥ 56 min. On the RTX 4080 Laptop (17.392 ms) one step plus the reserve takes ≈ 20.4 ms: 60 frames
  a second does not hold while the star burns (≈ 49 a second, ≈ ×0.56).
- **The collapse alone** (3,000 steps, the box growing from 136 × 128 to the world): ≈ 21 s of the Quadro's time at
  M0-T63's box costs (4.742 ms small, 10.709 ms whole), ≈ 23 s at 30 frames — twice the 10 s budget before any calm stretch.
- **A life in ~1 min** with lot 14's shortest life (~175–300 t.u., UNVERIFIED) needs ×3–×5: 9–15 steps a frame at 30, so a
  step of 2.1–3.4 ms — 3–5× under 10.709 ms.
- **Two causes**: the clocks (τ_KH ≈ 134 τ_dyn, τ_nuc ≈ 26 τ_KH, M0-D50's arithmetic) give ~10× the steps of the time-warp
  study's scenario L (13,200 at 88 steps a τ_dyn); the step is ~100× the study's Sun-like box (0.11 ms, M4) because the box
  is the world and P5's implicit solve is costly.

## What the ruling changed — each surface cited to its answer
| Surface | Change | Answer |
|---|---|---|
| Contract §0.5 | three [M0-TJ2] lines: the top speed and the stop; G-FPS's bounds; the cadence | Q1 |
| Contract §0.2 | Q2's entry names the ruling that added the top speed | Q1 |
| Contract §1.8.2 | TOP caps the numbered rungs and is never a rung; Max is the top rung; keys 1…9 and 0 | Q1 |
| Contract §1.8.3 · §1.8.4 | at Max N = N_max, nothing owed · Max's readout always the running speed (`time.speed_max`) | Q1 |
| Contract §1.8.6 | FROZEN [M0-TJ2] beside the lead's words: f_target 30 at Max, 60 at every numbered rung; nothing stops on performance; jumps, a slower top speed and the physics-for-speed levers routed past M0 | Q1 |
| Contract §1.4.4 · §5.3 G-CAD · §5.5 | not built in M0, k = 1 · G-CAD out of the register | Q1 |
| Contract §3.2 · §3.3 | TimeControl's numbered rungs and Max · `bench --rung <label\|max>`, bench.json's `off_cap` | Q1 |
| Contract §4.2 · §4.3 · §4.10 · §4.12 | Max's button and readout, the two strings M0-T125 adds to §4.1's block (not now: the `strings` scope reads the contract, and strings.rs must change with it), key 0, TOP no longer shown | Q1 |
| Contract §5.3 G-WARP · G-LATCH | TOP → Max | Q1 |
| Contract §5.3 G-FPS | the frame times reported, never NO-GO; the clamp graded (`off_cap` = 0) | Q1 |
| Contract §5.3 G-TOP | at Max to the white dwarf: every frame plans N_max at 30 frames (`off_cap` = 0); life_wall_s reported; plant `max-half` | Q1 |
| The plan's goal paragraph · Superseded | «as fast as the card goes», the 60-frames target reported · three mirror lines | Q1 |
| M0-T123 (new, after this block) | the Max rung in `sr_physics::time`: its time_control cases, plant `time-max-paced`, verify.json's entry, docs/agent/time_control.md and its testing row | Q1 |
| M0-T125 (new, after M0-T123) | the two strings in §4.1's block and strings.rs, in one edit (G-STR stays green) — a second artifact kind, so its own block (PLAYBOOK §2.1 (i), (vi)) | Q1 |
| M0-T124 (new, lot 16, after the bench, before G-WARP, G-FPS and G-TOP) | the cost task on the Quadro: where the burning star's step goes, the cheapest cut that changes no physics; inside lot 16 rather than before it, so it measures lot 14's tuned star and the bench exists | Q3 (Q1's reading 2) |
| M0-T98a | the shortest Sun-like life G-SQUEEZE's bounds allow | Q2 |
| M0-T85 · T75 · TV2 · T109 · T110 · T111 · T112 · T113 · T115 · V16 · V17 | re-scoped in place, tagged [M0-TJ2] (TOP → Max; G-FPS and G-TOP as above; V16 and V17 no longer wake this block) | Q1 |
| M0-T114 | N/A — superseded by this ruling | Q1 |
| M0-TZ | flagged: § Routed | Q1, Q2 |

## Routed (PLAYBOOK §2.6 P9) — tier later, no code in M0, to M0-TZ (it places them with the lead)
- **The player's choice of the life's pace** — the lead, Q2: «I would like to maybe the user have a choice on time step length
  or something, if possible.» Technically open (reading 5): constant sets each calibrated, or a setting over shortcut jumps
  once they exist; a step longer than the CFL step is not a setting.
- **Optimising further** — the lead, Q1: «maybe later in another milestone will we optimize further but not now»: shortcut
  jumps (TW R6, a second integrator and its tests); the gravity cadence (§1.4.4, G-CAD, M0-T114's spec); every lever M0-T124
  finds that changes the physics (a box that freezes thin gas far from the star, a looser solve); a cheaper step beyond
  M0-T124's one cut; a speed per graphics card beside B21's world size.
- **A fixed top speed** (a life in ~10 s, or in ~1 min — B28's) — Q1 picked neither; a later milestone re-asks only if the
  lead raises it (P8).

## Negative scope — not retired, explicitly
- Q2's FROZEN «30 pictures/s at top speed»: kept, at Max. The numbered ladder, ×1's anchor (§1.8.1), the slow-down and its
  latch (§1.8.5), time never touching the physics (§1.8.7), the cap's honesty (§1.8.4) and N_max's formula (§1.8.3).
- G-WARP's bit-identity and G-LATCH's one-step catch (only TOP → Max); G-FPS's clamp and its plant `no-clamp`; G-BOX; G-CAL.
- `life_sun_sb` and `top_rung` in calibration (M0-T70 measures both as before; top_rung now caps the numbered rungs);
  render_reserve_ms (M0-T95).
- G-SQUEEZE's bounds (≥ 10 and ≥ 10) and every physics constant's bound: only lot 14's aim inside them moved.
- The active box's rule (§1.3.3: outside it, vacuum, static) — a box that drops thin gas changes the physics: routed.
- P5's accepted cost (§0.5 [M0-T122]); the harness's loop budget (R9) and the ⏱ scopes' routing (R12) — test time, not the
  product's speed.
- The goal's 60-frames target on a mid-range PC, now reported, not gated. Steam, the web build and B21 untouched.

## Measured on the way, not acted on
- **The RTX 4080 Laptop ran P4 at 8.07 ms on the whole world, the Quadro 4.17 ms** (M0-T63, the same box, other constants:
  hydro's cost does not depend on them) — on this workload this laptop card runs about 2× slower than the Quadro; cause not
  measured (a laptop GPU's power state is the first suspect). Not filed: a speed, which the ruling never blocks on; handed
  to M0-T124 as a lead, and the reason the lead's laptop will show the burning star under 60 frames.
- **The box is the whole world from step ~3,000** on the shipped constants too (M0-T63: by step 2,000, to 25,000): the bounce's
  thin ejecta hold it there. M0-T124's first measurement.
