# M0-TR1 — where the phase-close sweep's game-launching red-arms spend their time

Measured on win-laptop (Laser2025-20, Windows 11, 22 threads; box load 3 % at the start of each series), 2026-10-12, by M0-TR1.
Raw numbers and the commands' output: logs/M0-TR1.log. Before = M0-V3's sweep (the lead's run, 2026-10-11, logs/M0-V3.<scope>.redarm.log,
logs/loop_times.jsonl); this report adds no check and moves no plant.

## The finding in one line
**Every game-launching arm compiles the whole dependency tree again.** tests/smoke.sh and tests/capture/check.sh build sr-app into the
copy's own `build/target` (a scratch copy leaves `build` out by name, so each arm — the clean one and each plant's — starts with no
cache), and a cold release build of sr-app is 199 s on an idle box against **11 s** when only the three workspace crates change.
The run itself is 2–15 s. Web already shares the dependency cache (trunk runs under `sr_in_shared_target`); its time is serial
waiting on two locks and a 29 s `wasm-opt` per arm, not compiling.

## Before — the four campaigns in M0-V3's sweep (1,790 of 2,221 s)
| Scope | Wall | Arms (clean + plants) | Where the seconds went (from the logs) |
|---|---|---|---|
| boot | 344 s | 2 at once | cold build 330 s in each arm, then the run: 7 s |
| desktop | 652 s | 3 at once, two cases each | cold build 438 s in each arm (6 cases queued on 3 build dirs); `status-never-ready`: window +98 s and launcher +97 s waiting out the 90 s ready deadline, serial on port 47811's lock; `launcher-no-stop`: launcher +100 s |
| web | 505 s | 5, three cases each | no cold build (shared cache); all 15 case runs queue on the port-47812 lock held from a case's first line to its last, ≈ 100 s of work per arm × 5 |
| capture | 289 s | 2 at once | cold build ≈ 280 s in each arm, then the run: ≈ 5 s |

## Per span, win-laptop, now
| Span | Cold (no cache) | Warm | Note |
|---|---|---|---|
| scratch copy | 13.1 s (`scratch_copy.sh`, a proxy) | — | verify.py's own copy started the arms within 2 s in V3's logs; not the cost |
| sr-app release build, own `build/target` | **198.9 s** alone (load 3 %); 161 s and 186 s with two arms at once; **330–438 s** in V3's sweep (box 15 % busy outside, 2–3 arms) | 0.8 s no change; **11.1 s** workspace touched (sr-physics, sr-engine, sr-app) | target dir 777 MB |
| the same build into the shared `redarm-target` through the copy's `cargo` wrapper | — | 14.0 s, then 10.1 s | the same 3 crates; the dependencies stay cached; the `--workspace` build of the `build` scope and `-p sr-app` share them |
| the binary into the arm's own `build/target/release` | — | 0.09 s | 13 MB |
| boot run (`launch.py smoke headless-boot` + summary.json check) | — | 5 s | |
| window run (game-offscreen) / launcher run (start.bat) | — | 2 s / 4 s | |
| capture smoke / refuse | — | ≈ 14 s / 1.4 s | the smoke includes a 1 s cargo no-op |
| web: `trunk build --release` | 55 s the arm's first, 40 s after a touch | | cargo 7.4 s; **≈ 29 s is `wasm-opt -Oz`** (`data-wasm-opt="z"`) and wasm-bindgen; the whole trunk run holds the shared-target lock |
| web: `npm ci` / `ready` / `no-webgpu` | — | 2.7 s / 33 s (npm ci and Chrome's first start included) / 11 s | |
| plants that wait out a deadline | — | 90 s (desktop ready, ×2 cases), 30 s (web never-ready) | these are the measurement: a plant is red because the wait expires |

## Causes, named
1. **Cold dependency build in every arm** (boot, desktop, capture): 330 / 438 / 280 s of 344 / 652 / 289 s. §6.5 allows the shared cache
   for dependencies and forbids it for "a binary a scope launches" because two arms would race on one uplifted `sandbox-reactions.exe`;
   M0-T4 and M0-T8 chose a build dir per arm and got a cache per arm.
2. **Concurrency on a busy box** doubles the cold build (199 s alone, 330–438 s in the sweep) — a cost the first cause multiplies.
3. **Web: serial queues.** One lock on the port held from a case's start (build included) and one on the shared target held for a
   whole trunk run (wasm-opt included): five arms cannot overlap their builds with each other's Chrome runs. Sum of the serial work
   (5 × ≈ 100 s) equals the wall, 505 s.
4. **Deadlines of the red plants** (desktop ≈ 300 s, web 30 s) on the same port lock: intrinsic, kept.

## The lever, prototyped (scratch copy, never the tree)
Build sr-app into the shared target under the existing lock (`sr_in_shared_target` already touches the inputs, so a clean arm cannot
read a plant's build — M0-D9) and copy the binary into the arm's own `build/target/release/` inside that lock; the scope launches its
own copy. Two arms of boot's shape (both GO, each launching its own copy) took **31.9 s wall** (24.0 s and 31.8 s per arm; 16 s and
27 s of that the serialised builds) against 344 s. A first attempt that left smoke.sh's `unset CARGO_TARGET_DIR` in place cold-built
in each arm (161 s and 186 s concurrently) — the unset is part of the fix. NOT PROVEN: window, launcher and capture on a copied
binary (only headless-boot ran from one); the plants' red arms (no plant applied in the prototype); linux-pc.

## Projection for the sweep (estimates marked ~; the filed blocks measure them)
| Scope | V3 | after M0-TR1a | after M0-TR1b |
|---|---|---|---|
| boot | 344 s | ~35 s (prototype 32 s) | same |
| capture | 289 s | ~55 s | same |
| desktop | 652 s | ~350 s (floor: the deadlines, ≈ 300 s) | same |
| web | 505 s | 505 s | ~270 s |
| the other 10 moved scopes | 431 s | 431 s | 431 s |
| **sweep** | **2,221 s** | **~1,375 s** | **~1,140 s** |
The sweep stays over the 10-minute line (the lead's run, in a window, Rule 4) — about 19 min instead of 37. What is left is the
`build` scope (162 s), the deadlines (≈ 330 s), web's trunk runs and the cheap scopes; none is a repeated dependency build.

## Filed from this report
- **M0-TR1a** — the three native game-launching scopes build through the shared cache and launch their own copy (boot, desktop,
  capture).
- **M0-TR1b** — web's locks cover only what needs them (the port: launch and smoke; the shared target: the trunk run), and a per-tree
  build lock keeps one arm's three cases from building one `dist` at once.
Both run ahead of M0-D21 and so ahead of M0-V4's sweep. Each Verify re-measures its campaigns with the same harness.

## Declined or not filed, with the measure
- **`wasm-opt` off in test builds**: web build 40 s → 10.8 s (measured; 5 arms ≈ 145 s). Declined: the wasm would be 5.2 MB, not the
  4.1 MB the page ships, and the ready smoke would run a binary nobody ships — a saving bought by measuring something else.
- **A port per arm for desktop's red plants** (their two 90 s deadlines would overlap, ≈ 100 s): not measured, and it reopens
  launch.json's port (contract §3.6) and start.bat/start.sh; the gain does not warrant a contract change. Left for the lead's call.
- **Fewer arms at once / sharing the clean arm across scopes**: no gain once the build is warm (builds serialise at 11 s each).
- **Cutting the 90 s / 30 s deadlines for plant arms**: a shorter wait is a weaker check. Kept.

## The budget line, restated
Header: « Full-loop budget: 600 s on each box ». Re-measured by `verify.py report`: win-laptop median over 5 counted complete loops
**91.1 s idle (97.7 s raw)**; the last, M0-V3's, **183.9 s normalised (187.7 s raw)**, 665 cases — under the line, no TR by the loop.
The line is the complete loop (`--all`); **the red-arm sweep is not under it**: 2,221 s at V3, ~1,140 s after the two fixes. The
plan has no line for the sweep. Proposed for the lead, not edited here (Rules are not this block's): « red-arm sweep at a phase-close V:
the lead's run, ≤ 1,200 s after M0-TR1a/b; over it, a TR ». M0-V4's sweep is its first reading.

## Owed
linux-pc (laserax-ai): none of this was run there (`NOT RUN — owed on laserax-ai`, R15); the lever's `cp` and lock are the same code
path, but the Xvfb routes (`game-xvfb`, `xvfb-run`) run on a copied binary only there.
