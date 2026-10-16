# M0-T63 — The step-cost probe: a Sun-like cloud's steps and the box's cost on the Quadro

Verdict: **the nominal Sun-like cloud (`preset:sun`, M₀ exactly, M0-T61) does not reach the end of its main sequence.** It
collapses, ignites at step 225 (sim 3.83), is shown as `main_sequence` for steps 257–416, then as `cloud` from step 417 to the end
of the run (25,000 steps, sim 324.8), the hydrogen at its densest cell untouched (X_H 0.734 at every checkpoint) and the nuclear
energy released frozen at 1598.05 from step 3,000 to step 25,000. `--until stage:h_shell` was never met: exit 2, `until=unmet`.
That stall is the finding the plan names for this block (§ Steps). Its cost is measured regardless: **4.7 to 12.5 ms a step on
the Quadro RTX 4000** (GPU timestamps, medians of 3), 43× to 97× the 0.11 ms a step the time-warp study (reports/time_warp.md §
Problem 3, M4) took to hold L and C, so no scenario is held (§ Projection).

Every number below ran on the **Quadro RTX 4000** (`SR-ADAPTER name=Quadro RTX 4000 backend=Vulkan driver=NVIDIA`, `--adapter
"Quadro RTX 4000"` on every command), box linux-pc (hostname laserax-ai), release build `build/target/release/sandbox-reactions`
(up to date: `cargo build --release` finished with nothing to rebuild), **SHA 60dc024 with a dirty tree** (the lot 8 work of
M0-T56–M0-T62 is uncommitted; the binary is built from it). The box was **loaded by other jobs** the whole time, `uptime` 1-min
load 24–44 (log lines in milestones/m0/logs/M0-T63.log); the per-pass numbers are GPU timestamps (`source: gpu-timestamp`), the
wall times are not and are named as such. Untuned: every constant of assets/physics.json (K1's gap included, M0-T74/M0-V9's to
tune), `headless::FRAME` = 16 steps (M0-T60).

## Run

The probe, once to the end of the budget I could run under 10 minutes (the 10-minute line is the lead's, Rule 4):

[ALREADY RUN — PASS (exit 2, `until=unmet`, 25,000 steps, 6 m 33 s wall; the stall is the finding) on linux-pc, Quadro RTX 4000]
```
build/target/release/sandbox-reactions headless --scene preset:sun --until stage:h_shell --max-steps 25000 --timing --adapter "Quadro RTX 4000" --out build/m0-t63-run1
```

| field | value |
|---|---|
| stdout | `SR-HEADLESS DONE steps=25000 sim_time=324.834736 until=unmet` |
| wall | 392.2 s (summary.json `wall_s`) = 15.7 ms a step, against the GPU's 10.96 ms: the 16-step frames' summaries and the submissions sit in the gap |
| load at launch / end | 26.2 / 40.9 (1-min) |
| world | 600 × 400, edge `leave`; the box (M0-T36) is the whole world at the end (dump s_25000: 600 × 400 at 0,0) |
| mass | 3351.03 at the start; at 25,000: grid 3130.09, escaped 220.50 (6.6 %), vacuum_reset 0.44; the ledger's mass invariant 3351.0308 |
| nuclear energy released | 1598.0463580340777 at step 3,000 and 1598.0463580340777 at 25,000 (identical to the last digit: no burning after step 3,000) |

A first, shorter run (3,000 steps, the same flags, `--out build/m0-t63-probe`) and a third with `--dump-every 100` (`--out
build/m0-t63-dumps`, the checkpoints below) read the same stages and the same events; both in the log.

[ALREADY RUN — PASS (exit 2 `until=unmet` at 3,000 steps, 43.9 s) on linux-pc, Quadro RTX 4000]
```
build/target/release/sandbox-reactions headless --scene preset:sun --until stage:h_shell --max-steps 3000 --timing --dump-every 100 --adapter "Quadro RTX 4000" --out build/m0-t63-dumps
```

## Steps

From summary.json's `objects[0].stages` and `events` (run 1; the 3,000-step runs read the same `from_step`s). A stage shows after it
has held 3 frames of 16 steps (§1.9.3, M0-T60), so each `from_step` is the first step it held at, not when the state turned:

| stage | from step | sim time | steps in it | note |
|---|---|---|---|---|
| no displayed stage yet | 1 | 0.030 | 32 | a stage shows after 3 frames; the first step's Δt is 0.0303 |
| `failed_star` | 33 | 0.913 | 224 | |
| ignition (event `ignition`, object 1) | 225 | 3.832 | — | detail `mass_sb` 3351.01 |
| `main_sequence` | 257 | 4.135 | 160 | |
| `cloud` | 417 | 5.406 | 24,584 and still there | the stall: `h_shell` never shows |

The cloud's state at the dump checkpoints (`--dump-every 100`, read from the dumps' header and Σ and X_H planes; R₉₀ is the radius
about the densest cell holding 90 % of the Σ above 10⁻³, τ_dyn = √(R₉₀³ / (G M)) with G = 1, §1.9.2):

| step | sim time | Δt (t.u.) | active box (cells) | Σ max | X_H at that cell | R₉₀ | τ_dyn | steps per τ_dyn |
|---|---|---|---|---|---|---|---|---|
| 100 | 2.298 | 0.01685 | 120 × 128 | 2.00 | 0.735 | 31.1 | 3.00 | 178 |
| 200 | 3.584 | 0.01019 | 136 × 128 | 10.10 | 0.735 | 24.0 | 2.03 | 199 |
| 300 | 4.532 | 0.00896 | 136 × 144 | 12.39 | 0.734 | 14.8 | 0.98 | 109 |
| 400 | 5.315 | 0.00539 | 136 × 144 | 7.39 | 0.734 | 19.4 | 1.48 | 274 |
| 600 | 6.457 | 0.00641 | 216 × 208 | 4.92 | 0.734 | 28.6 | 2.65 | 413 |
| 1,000 | 9.608 | 0.00925 | 360 × 368 | 7.11 | 0.734 | 43.9 | 5.03 | 544 |
| 2,000 | 20.038 | 0.01137 | 600 × 400 | 6.35 | 0.734 | 31.0 | 3.02 | 266 |
| 3,000 | 32.586 | 0.01267 | 600 × 400 | ~6.5 | 0.734 | 22.6 | 1.91 | 151 |

(Checkpoints 500, 700–900, 1,100–1,600, 2,400 and 2,800 are in logs/M0-T63.log: Δt 0.0054 to 0.0127, the box growing from 136 × 144
at step 400 to the whole world by step 2,000.)

- **Δt's range**: 0.0303 (step 1) down to 0.0054 (step 400, near the ignition's overshoot) and back up to 0.0127 (step 3,000); the
  run's mean is 324.83 / 25,000 = 0.0130 t.u. a step.
- **Steps per τ_dyn**: 109 to 544 over the checkpoints against the 150 the time-warp study assumed for a settled Sun-like cloud
  (CFL 0.4, R = 15 cells); this cloud is not settled (R₉₀ swings 14.8 → 43.9 → 22.6 as it pulses), so the range is the pulse's.
- The densest cell never burns its hydrogen: the star is a pulsing cloud with a faint ignition, which is not the main sequence M0-T62
  and G-END expect. Whether that is the untuned constants (assets/physics.json, K1's gap, the preset's thermal share 0.1|W|) or a
  defect is not measured here: tasks/M0-T63.md § Not acted on.

## Cost

The step's cost on the Quadro, ms a step (summary.json `timing`, `source: gpu-timestamp`). The five rows start from a dump (`--load-dump`,
so the box is the one the dump recorded and the run refits it as it goes); the three runs of each give the median:

[ALREADY RUN — PASS (12 runs, exit 0, medians of 3) on linux-pc, Quadro RTX 4000]
```
build/target/release/sandbox-reactions headless --scene preset:sun --load-dump build/m0-t63-dumps/dumps/s_<S>.bin --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-t63-at<S>-<i>
```
(S = 200, 500, 1000, 2000; i = 1..3; load 29 at the end of the set, 1-min.)

| from dump (box at the dump) | step, 3 runs | **step median** | P2 gravity | P3 sinks | P4 hydro | P5 heat | P6 reactions | P8 floors | P0+P1+P7+P9 |
|---|---|---|---|---|---|---|---|---|---|
| s_200 (136 × 128) | 5.769 · 4.376 · 4.742 | **4.742** | 0.269 | 0.261 | 0.711 | 2.491 | 0.396 | 0.056 | 0.181 |
| s_500 (184 × 176) | 12.493 · 12.515 · 12.655 | **12.515** | 0.159 | 0.153 | 0.607 | **11.111** | 0.245 | 0.042 | 0.073 |
| s_1000 (360 × 368) | 5.663 · 5.643 · 5.657 | **5.657** | 0.542 | 0.331 | 1.869 | 1.878 | 0.687 | 0.115 | 0.109 |
| s_2000 (600 × 400, whole world) | 10.709 · 10.753 · 10.605 | **10.709** | 1.107 | 0.471 | 4.166 | 3.287 | 1.269 | 0.157 | 0.125 |

(Per-pass cells are each pass's own median; they do not add to the step median. Wall for 200 steps: 2.8 s, 3.2 s, 1.3 s, 2.4 s.)

The whole 25,000-step run (the real mix of boxes, one run, no median): **step 10.956 ms**, P2 1.119, P3 0.485, P4 4.212, P5 3.500,
P6 1.221, P7 0.038, P8 0.159, P9 0.075, P0 0.013, P1 0.006. The 3,000-step run: step 8.049, P4 2.508, P5 3.334.

- **P5 is the cost where the cloud pulses**: 11.1 of 12.5 ms at step 500 (the box 184 × 176 and growing, Σ max falling 12.4 → 5.7); the
  implicit solve's passes and rounds (M0-T122: a hard step carries over submissions) are not counted here, the run prints none.
- P4 and P2 scale with the box (P4 0.71 → 4.17, P2 0.27 → 1.11 from 136 × 128 to the whole world); P5 does not scale with it.
- Beside the study's numbers (M4, a Sun-like box, 0.11 ms a step; M2, the whole world, 2.13 ms): the measured step is 43× (the
  smallest box) to 97× (the whole world) M4, and 5× M2 for the whole world. M0-T122's P5 alone read 0.6189 ms on sun_disk at 200
  steps; here it reads 2.5 ms on the same card's first 200 steps of this cloud, a different load (the cloud's box and Σ differ from sun_disk's at step 200), so the two are not the same measurement.
- The first run of the s_200 set (5.769) is 22–32 % over the other two, as M0-T122 saw: a first run's warm-up, cause not measured.
- Load: GPU timestamps ignore the host's load, the wall per step does not (15.7 ms against 10.96 on the long run).

## Projection

What the time-warp study's scenarios need, in steps per ~10 s life (600 frames at 60 a second), at the cost measured above. The
scenarios' steps per frame are the study's own (reports/time_warp.md § Problem 2: L 22,500 steps = 37.5 a frame, C 63,000 = 105, H
340,000 = 560); the measured cost per step comes from § Cost. **The main sequence's own length is not measured: it never ended, so
the "steps per life" below is the study's arithmetic at this probe's costs, not a measured life.**

Frame cost, ms, at 600 frames (a ~10 s life at 60 a second) against the 16.7 ms frame — the simulation's share is ~13 ms
(time_warp.md R8, ~3 ms drawn):

| scenario | steps a frame | box 136 × 128 (4.742 ms a step) | box 360 × 368 (5.657) | whole world (10.709) | against 16.7 ms |
|---|---|---|---|---|---|
| steps that fit in 13 ms | — | 2.7 | 2.3 | 1.2 | |
| L | 37.5 | 178 | 212 | 402 | 11× to 24× over |
| C | 105 | 498 | 594 | 1,124 | 30× to 67× over |
| H | 560 | 2,656 | 3,168 | 5,997 | 159× to 359× over |

At 30 a second (Q2) a ~10 s life is 300 frames: each frame holds twice the steps and costs twice the figures above (L 356 to 803 ms),
against a budget twice as long (33.3 ms), so the ratio is the same as the last column — **no scenario holds at 30 frames either**.
The ratios are those of the measured step; the cheapest (4.742 ms) gives the low end, the whole world's 10.709 the high end.

How **K1's gap** moves the projection (the study's formula, my arithmetic: life = 10 τ_dyn pre-MS + 1.4 × main sequence, the main
sequence = gap² τ_dyn, 150 steps a τ_dyn; the study's L is gap 10 → 22,500 steps; the real gap is not measured, SM-O14, and the
measured steps per τ_dyn were 109 to 544):

| K1 gap | life (τ_dyn) | steps per life | steps a frame (600) | frame ms at 4.742 · 5.657 · 10.709 ms a step |
|---|---|---|---|---|
| 10 (L) | 150 | 22,500 | 37.5 | 178 · 212 · 402 |
| 5 | 45 | 6,750 | 11.2 | 53 · 64 · 121 |
| 4 | 32.4 | 4,860 | 8.1 | 38 · 46 · 87 |
| 3 | 22.6 | 3,390 | 5.7 | 27 · 32 · 61 |
| 2 | 15.6 | 2,340 | 3.9 | 18.5 · 22 · 42 |

So at this probe's cost a life of about 10 s holds 33.3 ms only for a gap of about 3 in the smallest box (not in the whole world,
where the cloud sits from step 2,000), and holds 16.7 ms for nothing above a gap of 2. Re-reading L at the measured 109 to 544 steps
per τ_dyn instead of 150 moves it to 16,350 to 81,600 steps (27 to 136 a frame): ±3× on every row above. **The cost per step, not
the number of steps, is what puts every scenario out of reach**: halving the gap's steps does not recover a 43× to 97× step.
Both readings are the untuned constants'; none is final (assets/physics.json, K1, `headless::FRAME` and the preset's 0.1|W| are
the untuned ones this run depends on). The Quadro's frame cost at a ~10 s life reads over 33.3 ms (the 30-frame budget) in every row of the scenarios: that is the line
M0-V8 reads for M0-TJ2 (the plan's wake condition).

## Not run, and what stays open

- [NOT RUN — for you]: a run past this one's 25,000 steps. It would not change the finding (the nuclear energy has been frozen since
  step 3,000 and the stage `cloud` since step 417), and a run over 10 minutes is the lead's (Rule 4); the command, if wanted:
  `build/target/release/sandbox-reactions headless --scene preset:sun --until stage:h_shell --max-steps 100000 --timing --adapter "Quadro RTX 4000" --out build/m0-t63-long`.
- [NOT RUN — for you]: the same probe on an idle box (a heavy job of the same user ran throughout: load 24–44). The GPU-timestamp
  numbers should move little (M0-T122: loaded and idle medians agreed within 0.7 %); the wall numbers will.
- [NOT RUN — owed on win-laptop]: nothing here is the promise's card; the RTX 4080 Laptop is not the Quadro and no number above
  names it.
- Not measured: the star's temperature (the engine's summary.json carries no T), which pass costs fell with the box's growth between
  checkpoints (the dumps' boxes are the ones read at 100-step intervals), and the number of Newton passes and PCG rounds behind
  P5's 11.1 ms at step 500 (M0-T122 probed them with a throwaway test; `headless` prints none).
