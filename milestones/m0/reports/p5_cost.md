# M0-T122 â€” P5's cost on the Quadro against the lead's 0.5 ms a step

Verdict: **RED against contract §0.5 [M0-TJ4] (7)** — P5 heat's median over 3 runs on the idle box is **0.6189 ms** a step
(ceiling 0.5; over by 24 %). Re-run idle after a first set at load ~14 read 0.6148 ms: the load did not move it. Not a looser
number: back to the lead (§0.4). Every number below is the Quadro RTX 4000 (Vulkan, NVIDIA), box laserax-ai, SHA 34e41ca (clean
tree; `build/target/release/sandbox-reactions` up to date).

## Runs

Command (3 times, i = 1..3; `milestones/m0/logs/M0-T122.idle.log`; outputs `build/m0-t122-timing-<i>/summary.json`):

    build/target/release/sandbox-reactions headless --scene scenes/sun_disk.json --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-t122-timing-<i>

| set | run | exit | adapter (SR-ADAPTER) | SHA | `uptime` load (1/5/15 min) at launch |
|---|---|---|---|---|---|
| **idle (graded)** | 1 | 0 | Quadro RTX 4000, Vulkan, NVIDIA | 34e41ca | 0.33 6.33 13.75 |
| idle | 2 | 0 | same | 34e41ca | 0.33 6.33 13.75 |
| idle | 3 | 0 | same | 34e41ca | 0.38 6.24 13.69 |
| loaded (earlier; kept in `build/m0-t122-loaded-<i>/`, `logs/M0-T122.log`) | 1–3 | 0 | same | 34e41ca | 14.3 / 14.3 / 14.3 (1 min), 18–19 (5 min) |

The loaded set ran while a six-worker Python `trading_app` job of the same user held ~500 % CPU (not stopped, not mine); the lead
reported the box idle afterwards and the idle set was taken at 1-min load 0.33–0.38 (5-min still draining from the job).

## Cost

P5 heat and the step's mean, ms a step (200 steps, gpu-timestamp source):

| run | idle P5 heat | idle step_ms_mean | loaded P5 heat | loaded step_ms_mean |
|---|---|---|---|---|
| 1 | 0.8067 | 1.6362 | 0.8463 | 1.7782 |
| 2 | 0.6189 | 1.1833 | 0.6143 | 1.1801 |
| 3 | 0.6162 | 1.1811 | 0.6148 | 1.1791 |
| **median** | **0.6189** | **1.1833** | 0.6148 | 1.1801 |

- Against contract §0.5 [M0-TJ4] (7): idle median **0.6189 ms > 0.5 ms**. Runs 2 and 3 agree to 0.4 %, and equal the loaded
  set's to 0.7 %: the load does not explain the excess.
- Run 1 of each set is 30–38 % slower on every pass, loaded or idle (cause not measured; the median ignores it).
- Beside RKL2 (M0-T42, the 32-stage-capped explicit P5): 0.188–0.261 ms a step — the implicit P5 costs 2.4–3.3× that.
  M0-T119's baseline before M0-T121 was 20.42 ms (every one of 4,146 slots recorded); M0-T121's engine-chosen slots cut it ~33×.
- Steps a 60-fps frame holds (13 ms for the simulation, M0-R3 / time_warp.md): 13 / 1.1833 = **11.0 steps** of the whole sun_disk
  step; P5 alone would hold 13 / 0.6189 = 21.0 steps; at the ceiling (0.5 ms) 26.

## Rounds

200 steps of scenes/sun_disk.json, one step per `Step::run_counted(1)` then `Step::read_heat`, on the Quadro (a scratch probe,
below; log `milestones/m0/logs/M0-T122.probe.log`; load 19â€“22 at that time):

| per step | median | p95 | max | mean |
|---|---|---|---|---|
| Newton passes | 1 | 3 | 12 | 1.7 |
| Jacobi-PCG rounds (over all passes) | 0 | 1 | 46 | 1.2 |
| slots recorded (engine's choice, `heat_rounds`) | 10 | 20 | 110 | 15.4 |

- Unconverged steps in 200: **0** (no `SR-WARN heat unconverged` in any of the three timing runs either). Rounds never exceeded the
  recorded slots. Step 4 took 5 passes / 32 rounds here, not the 16-pass ceiling M0-D37 measured at 1000 steps on the RTX 5090.
- The max is the start: steps 0â€“5 take 7â€“12 passes and 11â€“46 rounds; from there the median step is one pass with no PCG round.
- Idle cost (M0-T118's flag): not split. P5 is timed as one pass (`per_pass_ms`); a typical step records ~10–20 `heat_round`
  slots of which 0–1 do work, so the dispatches of the zero-workgroup rounds, the Newton-pass machinery and the gaps
  between them are the candidates, unmeasured.

## Carried

Steps carried over submissions in the 200: **2** â€” step 2 (2 submissions, 46 rounds, 12 passes; 110 slots recorded) and step 32
(2 submissions; the record read after it shows 0 rounds). Every other step ended in 1 submission (mean 1.01, max 2). Submissions in
all: 202 for 200 steps.

## Method and deviations

- The Rounds and Carried numbers come from a throwaway test (`tests/gpu/p5probe.rs`, `mod p5probe;`) in a scratch copy
  (`tools/pb/scratch_copy.sh` â†’ ~/private/scratch_t122, CARGO_TARGET_DIR the real tree's build/): `headless` prints no per-step
  passes or rounds (summary.json holds none). It is not in the tree.
- The Rounds probe was not re-run at idle (its numbers carry no timing).
