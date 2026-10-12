# M0-TR1 — task file

Done on win-laptop (Laser2025-20), model=claude-sonnet-5-5 level=high. The findings, the span table, the projection, the declined levers
and the restated budget line are in reports/test_time_1.md; the raw output is logs/M0-TR1.log. This file keeps what the report does not.

## What ran (all on win-laptop, scratch copies under %TEMP%, removed at the end)
- `scratch_copy.sh` → a copy of the tree; cold `cargo build --release --locked -p sr-app --target-dir build/target`: 198.9 s (load 3 %),
  777 MB; no-change 0.8 s; workspace touched 11.1 s.
- The shared `redarm-target` through the copy's `cargo` wrapper: 14.0 s, 10.1 s; the exe copy 0.09 s.
- Warm runs: headless-boot 5 s, window 2 s, launcher 4 s, capture smoke ≈ 14 s, refuse 1.4 s; web build 55 s / 40 s after a touch,
  ready 33 s, no-webgpu 11 s, `npm ci` 2.7 s; web build with `data-wasm-opt="0"`: 10.8 s (5.2 MB wasm).
- The prototype (smoke.sh + `sr_build_app` in a scratch copy, two arms at once): first attempt cold-built in each arm (161 s, 186 s;
  smoke.sh's `unset CARGO_TARGET_DIR`), the second 31.9 s wall, both GO.
- `verify.py report`: 329 runs, 10 complete loops, win-laptop median 97.68 s raw / 91.12 s idle.

## Not measured
- Window, launcher and capture on a copied binary (M0-TR1a's Verify 2); the plants in the prototype; linux-pc.
- The web projection (~270 s) is an estimate from the per-arm spans; M0-TR1b's Verify measures it.
- Why `launcher-no-stop`'s launcher waits ≈ 100 s in V3 (not traced; M0-TR1a's walls will show whether it is intrinsic).

## Lint note for the next author
`plan.py lint` reads a `;` in a `Deliver:` field as an item break (and a `Pass:` token inside it as a criteria line): M0-TR1a and
M0-TR1b write their Deliver items with commas only.
