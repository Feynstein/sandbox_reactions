#!/usr/bin/env bash
# The measure-ui scope's runner (M0-T95, m0_contrat.md §3.3): bash tests/measure/check.sh <case>. A check of the flag, never
# the measurement (R4, §6.1): the cost it reads here is lavapipe's, on a private display, and goes into a COPY of the calibration.
# `sr_build_app` leaves the release binary in this tree's build/target (tests/capture/check.sh's route), then:
#   run    — `--measure-ui 20 --calibration <copy>` on the box's window route (`xvfb-run` + `--adapter llvmpipe` on linux-pc,
#            `--offscreen-window` on win-laptop). Pass: exit 0; one `SR-UI p95_ms=<x>` line with x finite and > 0; the copy
#            holds render_reserve_ms = x + 0.5 and every other key as it was; the real assets/calibration.json is byte for byte
#            what it was (hash before and after).
#   refuse — a frame count of 0 or not a number, a missing one, `--measure-ui` with `--capture`, `--calibration` alone → each exit 4
#            with an `SR-ERROR` line naming the flag, before any window opens; a file that is no calibration → exit 3 `SR-ERROR`
#            and the file untouched; (linux-pc) a software adapter with no `--calibration` → exit 3 `SR-ERROR` naming it, the real
#            file untouched.
# Ends in `=== GO ===` or `=== NO-GO: <reason> ===`.
source "$(dirname "${BASH_SOURCE[0]}")/../cargo.sh"
case_id="${1:-}"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) PY="py -3.12"; exe=".exe"; window=(--offscreen-window); runner=(); software=0 ;;
  *) PY="python3"; exe=""; window=(--adapter llvmpipe); software=1
     # winit prefers Wayland whenever WAYLAND_DISPLAY is set (M0-T8): without the `env -u` the window opens on the lead's desktop.
     runner=(xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE) ;;
esac
cd "$_sr_root" || exit 1
sr_build_app "$exe" || { echo "=== NO-GO: cargo build --release -p sr-app failed ==="; exit 1; }
bin="build/target/release/sandbox-reactions$exe"
dir="$(mktemp -d)"
trap 'rm -rf "$dir"' EXIT
real="assets/calibration.json"
hash_real() { $PY -I -c "import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest())" "$real"; }
before="$(hash_real)"
unchanged() { [ "$(hash_real)" = "$before" ] || { echo "=== NO-GO: $real changed ==="; exit 1; }; }

case "$case_id" in
  run)
    cp "$real" "$dir/cal.json"
    out="$("${runner[@]}" "$bin" "${window[@]}" --measure-ui 20 --calibration "$dir/cal.json" 2>&1)"; rc=$?
    echo "$out" | grep -v '^$' | tail -n 4 | cut -c1-160
    [ $rc -eq 0 ] || { echo "=== NO-GO: the measure run exited $rc, expected 0 ==="; exit 1; }
    [ "$(echo "$out" | grep -c '^SR-UI p95_ms=')" -eq 1 ] || { echo "=== NO-GO: not exactly one 'SR-UI p95_ms=' line ==="; exit 1; }
    p95="$(echo "$out" | sed -n 's/^SR-UI p95_ms=\([^ ]*\) .*/\1/p')"
    $PY -I - "$real" "$dir/cal.json" "$p95" <<'PY' || { echo "=== NO-GO: the copy does not hold p95 + 0.5 and nothing else new ==="; exit 1; }
import json, math, sys
real, copy, p95 = json.load(open(sys.argv[1])), json.load(open(sys.argv[2])), float(sys.argv[3])
if not (math.isfinite(p95) and p95 > 0):
    sys.exit(f"p95_ms {p95} is not finite and > 0")
got = copy.pop("render_reserve_ms", None)
if got is None or abs(got - (p95 + 0.5)) > 1e-9:
    sys.exit(f"render_reserve_ms {got}, expected {p95} + 0.5")
real.pop("render_reserve_ms", None)
if real != copy:
    sys.exit("another key of the calibration changed")
PY
    unchanged
    echo "measure-ui run: SR-UI p95_ms=$p95, the copy holds render_reserve_ms = p95 + 0.5, every other key and the real file unchanged"
    echo "=== GO ==="
    ;;
  refuse)
    expect() { # <exit> <needle> <args...>
      local want="$1" needle="$2"; shift 2
      out="$("${runner[@]}" "$bin" "${window[@]}" "$@" 2>&1)"; rc=$?
      [ $rc -eq "$want" ] || { echo "$out" | tail -n 5 | cut -c1-200; echo "=== NO-GO: expected exit $want for $*, got $rc ==="; exit 1; }
      echo "$out" | grep '^SR-ERROR' | grep -q -- "$needle" || { echo "$out" | tail -n 5 | cut -c1-200; echo "=== NO-GO: no SR-ERROR line naming $needle ==="; exit 1; }
    }
    printf '{}' > "$dir/not-a-calibration.json"
    expect 4 '--measure-ui' --measure-ui 0
    expect 4 '--measure-ui' --measure-ui many
    expect 4 '--measure-ui needs' --measure-ui
    expect 4 '--capture' --measure-ui 5 --capture "$dir/s.json" --out "$dir/o"
    expect 4 '--calibration' --calibration "$dir/cal.json"
    expect 3 'not a calibration' --measure-ui 5 --calibration "$dir/not-a-calibration.json"
    [ "$(cat "$dir/not-a-calibration.json")" = "{}" ] || { echo "=== NO-GO: a refused file was written ==="; exit 1; }
    if [ "$software" = 1 ]; then
      expect 3 'software adapter' --measure-ui 5
      unchanged
    fi
    echo "measure-ui refuse: a bad frame count, a missing one, --capture beside it, --calibration alone → exit 4; a file that is no calibration → exit 3, untouched; a software adapter without --calibration → exit 3, the real file untouched"
    echo "=== GO ==="
    ;;
  *) echo "=== NO-GO: usage: bash tests/measure/check.sh run|refuse ==="; exit 1 ;;
esac
