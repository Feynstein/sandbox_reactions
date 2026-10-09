#!/usr/bin/env bash
# The boot scope's runner (G-BOOT, m0_contrat.md §5.4): bash tests/smoke.sh <service> — builds the release binary into
# THIS tree's own build/target (in a red-arm scratch copy that is the copy's own, never the shared redarm-target of
# tests/cargo.sh, §6.5: two arms must not launch one binary), then `launch.py smoke <service>`. For `headless-boot` it
# also runs the same command once into a folder it keeps and checks summary.json against §2.12.2 (launch.py removes
# the throwaway --out it gave the booted run). `window` is the desktop scope's case (G-DESK, M0-T5): the box's route —
# `game-xvfb` (the private Xvfb, lavapipe) on linux-pc, `game-offscreen` on win-laptop (R15; contract §6.6) — whose
# /status must read "ready". Ends in `=== GO ===` or `=== NO-GO: <reason> ===`.
source "$(dirname "${BASH_SOURCE[0]}")/cargo.sh"
service="${1:-}"
[ -n "$service" ] || { echo "=== NO-GO: usage: bash tests/smoke.sh <service> ==="; exit 1; }
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) PY="py -3.12"; exe=".exe"; window="game-offscreen" ;;
  *) PY="python3"; exe=""; window="game-xvfb" ;;
esac
[ "$service" = "window" ] && service="$window"
launcher=0; [ "$service" = "launcher" ] && { launcher=1; service="$window"; }
cd "$_sr_root" || exit 1
start=$SECONDS
unset CARGO_TARGET_DIR
cargo_out="$(command cargo build --release --locked -p sr-app --target-dir build/target 2>&1)" \
  || { echo "$cargo_out" | tail -n 25; echo "=== NO-GO: cargo build --release -p sr-app failed ==="; exit 1; }
built=$((SECONDS - start))

# The desktop scope's cases run in parallel yet share launch.json's port 47811 and one pid file (M0-T6): take a lock keyed
# on the port, outside any tree, so two cases (or a clean arm and a plant's) boot one at a time. flock, else a mkdir lock
# broken when its holder is gone (Git Bash has no flock).
lock="${TMPDIR:-/tmp}/sr-boot-47811.lock"
_sr_unlock() { rm -rf "${dir:-}"; rm -rf "$lock.d" 2>/dev/null; }
if command -v flock >/dev/null 2>&1; then
  exec 8>"$lock" && flock 8
else
  until mkdir "$lock.d" 2>/dev/null; do
    holder="$(cat "$lock.d/pid" 2>/dev/null)"
    [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null && rm -rf "$lock.d"
    sleep 1
  done
  echo $$ > "$lock.d/pid"
  trap _sr_unlock EXIT
fi

if [ $launcher -eq 1 ]; then
  # `launcher` is the desktop scope's second case (G-DESK, M0-T6): the double-click launcher, fed its Enter, on the box's
  # window route — starts, reaches ready, stops, and leaves nothing running (R15: start.bat on win-laptop).
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) run_launcher() { printf '\n' | cmd //c "start.bat $service" 2>&1; } ;;
    *) run_launcher() { printf '\n' | bash start.sh "$service" 2>&1; } ;;
  esac
  out="$(run_launcher)"; rc=$?
  echo "$out"
  left="$($PY tools/pb/launch.py status --task "${PB_TASK:-launcher}" 2>&1)"
  # whatever the verdict, never leave a game or its Xvfb behind (a red-armed plant leaves one by design)
  $PY tools/pb/launch.py stop --task "${PB_TASK:-launcher}" >/dev/null 2>&1
  [ $rc -eq 0 ] || { echo "=== NO-GO: the launcher exited $rc ==="; exit 1; }
  echo "$out" | grep -q "^$service: ready on " || { echo "=== NO-GO: the launcher never reached ready ==="; exit 1; }
  echo "$out" | grep -q "^$service: pid .* stopped" || { echo "=== NO-GO: the launcher did not stop $service ==="; exit 1; }
  echo "$left" | grep -q "^services: .*, running: 0$" || { echo "$left"; echo "=== NO-GO: a process is left running after the launcher ==="; exit 1; }
  echo "launcher $service: built in $built s, started, stopped, nothing left running"
  echo "=== GO ==="
  exit 0
fi

out="$($PY tools/pb/launch.py smoke "$service" --task "${PB_TASK:-boot}" 2>&1)"; rc=$?
echo "$out" | grep -v '^=== '
[ $rc -eq 0 ] || { echo "=== NO-GO: launch.py smoke $service did not end GO ==="; exit 1; }

if [ "$service" = "headless-boot" ]; then
  dir="$(mktemp -d)"
  trap _sr_unlock EXIT
  run="$(build/target/release/sandbox-reactions$exe headless --scene preset:sun --steps 200 --out "$dir" 2>&1)" \
    || { echo "$run"; echo "=== NO-GO: the headless run exited non-zero ==="; exit 1; }
  echo "$run" | head -n 1 | grep -q '^SR-ADAPTER ' || { echo "$run"; echo "=== NO-GO: stdout's first line is not SR-ADAPTER ==="; exit 1; }
  echo "$run" | tail -n 1 | grep -q '^SR-HEADLESS DONE steps=200 ' \
    || { echo "$run"; echo "=== NO-GO: stdout's last line is not SR-HEADLESS DONE steps=200 ==="; exit 1; }
  $PY -I - "$dir/summary.json" <<'PY' || { echo "=== NO-GO: summary.json does not match §2.12.2 ==="; exit 1; }
import json, sys
s = json.load(open(sys.argv[1], encoding="utf-8"))
need = {"version": None, "tool": None, "git": None, "adapter": {"name", "backend", "driver"}, "scene": None,
        "world": {"width", "height"}, "steps": None, "sim_time": None, "wall_s": None,
        "ledger": {"start", "end"}, "events": None, "objects": None, "until": {"condition", "met"}}
bad = [k for k in need if k not in s]
bad += [f"{k}.{sub}" for k, subs in need.items() if subs and k in s for sub in subs if sub not in s[k]]
if bad:
    sys.exit("summary.json lacks: " + ", ".join(bad))
if s["steps"] != 200:
    sys.exit(f"summary.json steps = {s['steps']}, expected 200")
print(f"summary.json: {len(need)} keys of §2.12.2, steps = {s['steps']}")
PY
fi
echo "boot $service: built in $built s, booted and checked in $((SECONDS - start - built)) s"
echo "=== GO ==="
