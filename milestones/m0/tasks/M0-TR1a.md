# M0-TR1a — boot, desktop and capture build through the shared dependency cache

Box: win-laptop (Laser2025-20), 2026-10-12. Model: claude-opus-5-5, level high (`rung_record.py now`).

## What changed
- `tests/cargo.sh`: `sr_build_app <exe-suffix>`. Real tree: `cargo build --release --locked -p sr-app --target-dir build/target`
  (as before). Scratch copy (no `.git`): inside `sr_in_shared_target` (lock + touch, M0-D9) it builds into the shared
  `CARGO_TARGET_DIR` and copies `sandbox-reactions<exe>` into the copy's own `build/target/release/`, with a stamp
  `.sr-app-inputs` = sha1 of the touch list's file contents. A current stamp + binary → no build, no copy.
- `tests/smoke.sh`, `tests/capture/check.sh`: call `sr_build_app "$exe"`; the `unset CARGO_TARGET_DIR` and the direct
  `command cargo build --target-dir build/target` are gone. No case, plant or count changed.
- `docs/agent/testing.md` § Cargo scopes: the smoke bullet and the capture/web bullet.

## Deviation from the block's Deliver (declared)
The Deliver's « copies … in the same lock » alone failed desktop's first red-arm: its two cases (window, launcher) run in
parallel in one arm; the second case's `cp` hit the first case's running `sandbox-reactions.exe` → `cp: … Device or
resource busy` (Windows), the case NO-GO for the wrong reason (logs/M0-TR1a.desktop.redarm.log, first run, lines 27–50).
A rebuild from the same sources is not byte-identical (probe in a scratch copy: two builds, sha1 52af1a… vs 9066d7…), so
`cmp` cannot skip it. The stamp of input contents, checked inside the lock, does: same arm, same sources → the arm's binary
is kept. Probe (scratch copy): first build 11 s, second 1 s (kept), after an edit 11 s (rebuilt, new stamp). A broken
source → cargo's last lines + `=== NO-GO: cargo build --release -p sr-app failed ===`, exit 1.

## Measured (win-laptop, Laser2025-20)
| Run | Result | Wall | Load |
|---|---|---|---|
| boot / desktop / capture (scope), first | GO 1, 2, 2 | 15.0 / 9.0 / 3.9 s | — |
| `--redarm boot` (before the stamp) | GO, scene-refuse RED | 51.1 s | 2 % |
| `--redarm desktop` (before the stamp) | GO by its counts, but status-never-ready's window case RED for the busy `cp` | 172.0 s | — |
| boot / desktop / capture (scope), after the stamp | GO 1, 2, 2 | 3.9 / 6.8 / 3.1 s | — |
| `--redarm boot` | GO, scene-refuse RED, clean GO | **33.1 s** (≤ 90; V3 344) | 6.0 % |
| `--redarm desktop` | GO, status-never-ready RED (window and launcher: not ready within 90 s), launcher-no-stop RED (exited 1), clean GO | **260.0 s** (≤ 450; V3 652) | 8.4 % |
| `--redarm capture` | GO, capture-no-captions RED (captures do not match §3.3), clean GO | **55.7 s** (≤ 120; V3 289) | 5.7 % |

Desktop's clean arm read « built in 90 s »: six builds queued on the shared lock (≈ 11–15 s each), not a cold build.

## Not done / owed
- linux-pc (laserax-ai): `NOT RUN — owed on laserax-ai` (R15) — the `game-xvfb` and `xvfb-run` routes on a copied binary.
- `tools/pb/verify.json`'s boot/desktop/capture `desc` still end « the build is the tree's own » — now « the binary is the
  tree's own »; not in this block's Files, left as is.
