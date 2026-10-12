# M0-TR1b — the web scope's locks cover only what needs them

## What changed (tests/web/check.sh, docs/agent/testing.md's web row)
- **Port lock** (47812, `${TMPDIR:-/tmp}/sr-web-47812.lock`, flock or mkdir as before): taken after the build and `npm ci`, held for
  `launch.py start web`, the smoke and the stop. Before: taken at the script's first line, the trunk run included.
- **Build lock** (new, `build/web-build.lock.d` in the tree's own `build/`): a mkdir lock with the holder's pid, broken when that pid
  is gone (or when it has no pid a minute on). It covers the freshness check, `trunk build` (its `rm -rf dist`), `npm ci`, and,
  for `ready`/`no-webgpu`, the launch and the smoke. So no case rebuilds dist under another case's smoke. Lock order is always
  build then port, so the two locks can't deadlock.
- **One trunk run per arm**: in a red-arm copy (no `.git`), `build/` is the arm's own. The first case to build records its verdict
  (`build/web-build.result`: `GO`, or the NO-GO reason). Its sibling cases reuse that verdict while no input is newer: a recorded
  failure is reported again as `<reason> (this arm's build, reused)`, and `build` runs the dist checks again on a reused GO dist.
  In the real tree nothing changes: `build` always rebuilds, and the smokes build only a stale dist.
- **Exit paths**: one EXIT trap stops what the case started, then frees only the locks it holds. TERM, INT and HUP turn into an
  exit so the trap runs. A killed holder (TerminateProcess on Windows) is broken by the pid check.
- **Unchanged**: the shared-target lock (`sr_in_shared_target` still wraps the whole trunk run), `wasm-opt -Oz`, every case,
  plant, expected count, the 30 s deadline and verify.json.

## Measured (win-laptop, Laser2025-20, 2026-10-12)
| Run | Result | Wall | Load |
|---|---|---|---|
| `verify.py web` (Verify 1) | GO, 3/3 | 51.7 s | 4.1 → 3.2 % |
| `--redarm web` #1 (task M0-TR1b) | GO; web-no-gpu-check, web-autoload, never-ready, smoke-no-sandbox RED for their own reasons; clean GO | **257.0 s** (≤ 330; V3 505) | 4.9 %, 5 arms |
| `--redarm web` #2 (task M0-TR1b-r2) | GO; the same four RED; clean reused by fingerprint (4 arms) | **330.7 s**: **0.7 s over the 330 s bound** | 16.5 % |

- Run 1's arms finished their builds at 65, 121, 166, 208 and 250 s: the trunk runs are queued one by one on the shared-target lock,
  ≈ 42 s each. The other arms' Chrome runs fit in between.
- Run 2's trunk runs took ≈ 65 s each, at a load of 16.5 % that came from outside the run: its own work was lighter than run 1's (4
  arms against 5). The never-ready arm built last, so its 30 s deadline came at the end of the run.
- The clean arm's `ready` (run 1) reached srState "ready" after **1.1 s** of its 30 s, at about 208–228 s, while web-autoload's
  trunk run (208 → 250 s) was in flight. A busy box took about 1 s of that budget. No clean `ready` timed out.
- Where the wall stops now: N arms × one trunk run on the shared lock, plus the last arm's smoke. The remaining lever is to let
  wasm-opt run outside the shared-target lock, which this block's Deliver rules out.

## The lead's ruling on the miss (2026-10-12)
Asked: run 2 was 0.7 s over the bound at 16.5 % load. Answer: «DONE, miss recorded». The block closes DONE with both walls
recorded, and M0-V4's sweep measures web again on its box.

## Not done / owed
- linux-pc (laserax-ai): `NOT RUN — owed on laserax-ai` (R15). The flock path of the port lock and the mkdir build lock under
  Linux were not run. Flagged to M0-V4.
