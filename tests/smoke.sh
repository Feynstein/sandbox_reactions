#!/usr/bin/env bash
# The boot scope's runner (G-BOOT, m0_contrat.md §5.4): bash tests/smoke.sh <service> — `sr_build_app` (tests/cargo.sh)
# leaves the release binary in THIS tree's own build/target (in a red-arm scratch copy it builds through the shared
# redarm-target and copies the binary in: the arm launches its own copy, §6.5, M0-TR1a), then `launch.py smoke <service>`. For `headless-boot` it
# also runs the same command once into a throwaway folder, checks summary.json against §2.12.2 and prints the file it
# checked, verbatim, into its own output (the folder goes at exit, launch.py removes its own --out), so the scope's log
# keeps the text a V reads. `window` is the desktop scope's case (G-DESK, M0-T5): the box's route —
# `game-xvfb` (the private Xvfb, lavapipe) on linux-pc, `game-offscreen` on win-laptop (R15; contract §6.6) — whose
# /status must read "ready". `race` is its third (M0-D51): the port lock's hand-over, raced on purpose. `port-taken` its fourth
# (M0-D99): 47811 held, the game serves /status on the next free port and prints it. Ends in `=== GO ===` or
# `=== NO-GO: <reason> ===`.
source "$(dirname "${BASH_SOURCE[0]}")/cargo.sh"
service="${1:-}"
[ -n "$service" ] || { echo "=== NO-GO: usage: bash tests/smoke.sh <service> ==="; exit 1; }
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) PY="py -3.12"; exe=".exe"; window="game-offscreen" ;;
  *) PY="python3"; exe=""; window="game-xvfb" ;;
esac
[ "$service" = "window" ] && service="$window"
launcher=0; [ "$service" = "launcher" ] && { launcher=1; service="$window"; }
race=0; [ "$service" = "race" ] && { race=1; service="$window"; }
taken=0; [ "$service" = "port-taken" ] && { taken=1; service="$window"; }
cd "$_sr_root" || exit 1
start=$SECONDS
sr_build_app "$exe" || { echo "=== NO-GO: cargo build --release -p sr-app failed ==="; exit 1; }
built=$((SECONDS - start))

# The desktop scope's cases run in parallel yet share launch.json's port 47811 and one pid file (M0-T6): take a lock keyed
# on the port, outside any tree, so two cases (or a clean arm and a plant's) boot one at a time. flock, else a mkdir lock
# broken when its holder is gone (Git Bash has no flock).
lock="${TMPDIR:-/tmp}/sr-boot-47811.lock"
_sr_unlock() { rm -rf "${dir:-}"; rm -rf "$lock.d" 2>/dev/null; }
_sr_lock() {
  if command -v flock >/dev/null 2>&1; then
    exec 8>"$lock" && flock 8
  else
    until mkdir "$lock.d" 2>/dev/null; do
      holder="$(cat "$lock.d/pid" 2>/dev/null)"
      [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null && rm -rf "$lock.d"
      sleep 1
    done
    echo $$ > "$lock.d/pid"
  fi
}
# A case hands a port's lock on only once the port can be bound again (M0-D51): launch.py's « stopped » comes first — on
# win-laptop the stopped game's socket still took connections 2 ms – 3 s after it, its pid already gone
# (logs/M0-D51.holder.log) — and the next case's start read « port 47811 held by a process we did not start ». A bind,
# as the game binds (SO_REUSEADDR on POSIX, as Rust's std; none on Windows), never a connect: a listener whose backlog is
# full refuses connections yet still holds the port (M0-D51: the game's own bind then failed, os error 10048).
# Bounded: 30 s, then the case says so.
_sr_port_closed() {
  $PY -I - "$1" <<'PY'
import os, socket, sys, time
port, end = int(sys.argv[1]), time.monotonic() + 30
while time.monotonic() < end:
    with socket.socket() as s:
        if os.name != "nt":
            s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            s.bind(("127.0.0.1", port))
            sys.exit(0)
        except OSError:
            pass
    time.sleep(0.05)
sys.exit(1)
PY
}
_sr_release() { [ "$service" = "headless-boot" ] || _sr_port_closed 47811; _sr_unlock; }

_sr_lock; trap _sr_release EXIT

if [ $race -eq 1 ]; then
  # `race` (M0-D51): the hand-over raced on purpose, under the lock throughout so no other case slips in between. A
  # listener launch.py did not start holds 47811 for 10 s — the stopped game's socket, the measured gap made long and
  # certain; the case waits for the port as every case's hand-over does, then starts the window route straight behind
  # it, which must reach ready.
  up="build/.sr-race-$$.up"; rm -f "$up"
  $PY -I - "$up" 8>&- <<'PY' &
import os, socket, sys, time
s = socket.socket()
if os.name != "nt":
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)  # the game's own TIME_WAITs on 47811 (Linux only)
s.bind(("127.0.0.1", 47811)); s.listen()
open(sys.argv[1], "w").close()
time.sleep(10)
PY
  lingerer=$!
  while [ ! -e "$up" ] && kill -0 $lingerer 2>/dev/null; do sleep 0.1; done
  [ -e "$up" ] || { echo "=== NO-GO: the listener on 47811 never came up (the port taken?) ==="; exit 1; }
  rm -f "$up"
  _sr_port_closed 47811; held=$?
  out="$($PY tools/pb/launch.py start "$service" --task "${PB_TASK:-boot}-race" 2>&1)"; rc=$?
  $PY tools/pb/launch.py stop "$service" --task "${PB_TASK:-boot}-race" >/dev/null 2>&1
  echo "$out" | grep -v '^=== '
  wait $lingerer
  [ $held -eq 0 ] || { echo "=== NO-GO: the listener's port 47811 still bound 30 s on ==="; exit 1; }
  [ $rc -eq 0 ] || { echo "=== NO-GO: $service, started straight behind the hand-over, did not reach ready ==="; exit 1; }
  _sr_port_closed 47811 || { echo "=== NO-GO: port 47811 still bound 30 s after $service stopped ==="; exit 1; }
  echo "race $service: built in $built s, started straight behind a hand-over whose port outlived its holder, stopped"
  echo "=== GO ==="
  exit 0
fi

if [ $taken -eq 1 ]; then
  # `port-taken` (M0-D99, contract §0.5 [M0-D99]): a listener launch.py did not start holds 47811 throughout, under the lock;
  # the game, started straight (launch.py refuses a held port before the game runs), must print `SR-STATUS-PORT <n>`, n in
  # 47812–47830, and reach "ready" there within 90 s. The box's window route by hand: a private Xvfb and lavapipe as
  # game-xvfb on linux-pc (Wayland's variables dropped, so no window reaches the lead's desktop, R4), off-screen on win-laptop.
  up="build/.sr-taken-$$.up" log="build/.sr-taken-$$.out"; rm -f "$up" "$log"
  $PY -I - "$up" 8>&- <<'PY' &
import os, socket, sys, time
s = socket.socket()
if os.name != "nt":
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)  # as the game binds; a listener still refuses it the port
s.bind(("127.0.0.1", 47811)); s.listen()
open(sys.argv[1], "w").close()
time.sleep(300)
PY
  holder=$!
  while [ ! -e "$up" ] && kill -0 $holder 2>/dev/null; do sleep 0.1; done
  [ -e "$up" ] || { echo "=== NO-GO: the listener on 47811 never came up (the port taken?) ==="; exit 1; }
  rm -f "$up"
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*)
      build/target/release/sandbox-reactions.exe --status-port 47811 --offscreen-window >"$log" 2>&1 8>&- &
      game=$!; _sr_game_down() { kill $game 2>/dev/null; wait $game 2>/dev/null; } ;;
    *)
      env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET -u XDG_SESSION_TYPE setsid xvfb-run -a -s "-screen 0 1280x1024x24" \
        build/target/release/sandbox-reactions --status-port 47811 --adapter llvmpipe >"$log" 2>&1 8>&- &
      game=$!; _sr_game_down() { kill -TERM -- -$game 2>/dev/null; wait $game 2>/dev/null; } ;;
  esac
  trap '_sr_game_down; kill $holder 2>/dev/null; rm -f "$log"; _sr_release' EXIT
  port="$($PY -I - "$log" 2>&1 <<'PY'
import json, re, sys, time, urllib.request
end, port, last = time.monotonic() + 90, None, ""
while time.monotonic() < end:
    out = open(sys.argv[1], encoding="utf-8", errors="replace").read()
    if "SR-ERROR" in out or "panicked at" in out:
        sys.exit(f"the game failed: {out.strip().splitlines()[-1]}")
    m = re.search(r"^SR-STATUS-PORT (\d+)\s*$", out, re.M)
    port = port or (m and int(m.group(1)))
    if port:
        try:
            last = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/status", timeout=2)).get("status", "")
            if last == "ready":
                print(port)
                sys.exit(0)
        except OSError:
            pass
    time.sleep(0.25)
sys.exit(f"no ready within 90 s (SR-STATUS-PORT {port or 'never printed'}, last status {last or 'none'})")
PY
)"; rc=$?
  _sr_game_down; kill $holder 2>/dev/null; wait $holder 2>/dev/null
  grep -v '^=== ' "$log"
  [ $rc -eq 0 ] || { echo "=== NO-GO: port-taken: $port ==="; exit 1; }
  [ "$port" -ge 47812 ] && [ "$port" -le 47830 ] || { echo "=== NO-GO: the game served on $port, outside 47812–47830 ==="; exit 1; }
  _sr_port_closed "$port" || { echo "=== NO-GO: port $port still bound 30 s after the game stopped ==="; exit 1; }
  echo "port-taken $service: built in $built s, 47811 held, ready on $port as printed, stopped"
  echo "=== GO ==="
  exit 0
fi

if [ $launcher -eq 1 ]; then
  # `launcher` is the desktop scope's second case (G-DESK, M0-T6): the double-click launcher, fed its Enter, on the box's
  # window route — starts, reaches ready, stops, and leaves nothing running (R15: start.bat on win-laptop).
  # `.\start.bat`, never bare: with NoDefaultCurrentDirectoryInExePath set (Claude Code's shell sets it), cmd skips the
  # current directory when it looks a bare name up and reads « not recognised » (M0-D15, logs/M0-D15.log).
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) run_launcher() { printf '\n' | cmd //c ".\\start.bat $service" 2>&1; } ;;
    *) run_launcher() { printf '\n' | bash start.sh "$service" 2>&1; } ;;
  esac
  # launch.py's records are the box's, not the task's: a bare `launch.py stop` stops every service, a parallel scope's
  # included (M0-D28: it took the web scope's server mid-smoke). A decoy web server, started here under the web scope's
  # port lock (its lock file, flock or mkdir as tests/web/check.sh takes it), must outlive the launcher.
  decoy_task="${PB_TASK:-launcher}-decoy" web_lock="${TMPDIR:-/tmp}/sr-web-47812.lock" decoy=0
  _sr_decoy_down() {
    [ "$decoy" = 1 ] && { $PY tools/pb/launch.py stop web --task "$decoy_task" >/dev/null 2>&1; _sr_port_closed 47812; }
    [ -f "$web_lock.d/pid" ] && [ "$(cat "$web_lock.d/pid" 2>/dev/null)" = "$$" ] && rm -rf "$web_lock.d"
  }
  trap '_sr_decoy_down; _sr_release' EXIT
  if command -v flock >/dev/null 2>&1; then
    exec 9>"$web_lock" && flock 9
  else
    until mkdir "$web_lock.d" 2>/dev/null; do
      holder="$(cat "$web_lock.d/pid" 2>/dev/null)"
      [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null && rm -rf "$web_lock.d"
      sleep 1
    done
    echo $$ > "$web_lock.d/pid"
  fi
  decoy=1
  dout="$($PY tools/pb/launch.py start web --task "$decoy_task" 2>&1)" || { echo "$dout"; echo "=== NO-GO: the decoy web server did not start ==="; exit 1; }
  out="$(run_launcher)"; rc=$?
  echo "$out"
  left="$($PY tools/pb/launch.py status --task "${PB_TASK:-launcher}" 2>&1)"
  # whatever the verdict, never leave a game or its Xvfb behind (a red-armed plant leaves one by design)
  $PY tools/pb/launch.py stop "$service" --task "${PB_TASK:-launcher}" >/dev/null 2>&1
  [ $rc -eq 0 ] || { echo "=== NO-GO: the launcher exited $rc ==="; exit 1; }
  echo "$out" | grep -q "^$service: ready on " || { echo "=== NO-GO: the launcher never reached ready ==="; exit 1; }
  echo "$out" | grep -q "^$service: pid .* stopped" || { echo "=== NO-GO: the launcher did not stop $service ==="; exit 1; }
  echo "$left" | grep -q "^$service stopped$" || { echo "$left"; echo "=== NO-GO: a process is left running after the launcher ==="; exit 1; }
  _sr_port_closed 47811 || { echo "=== NO-GO: port 47811 still bound 30 s after the launcher stopped $service ==="; exit 1; }
  echo "$left" | grep -q "^web running " || { echo "$left"; echo "=== NO-GO: the launcher stopped a service it did not start (the decoy web server) ==="; exit 1; }
  echo "launcher $service: built in $built s, started, stopped, nothing left running, the decoy web server untouched"
  echo "=== GO ==="
  exit 0
fi

out="$($PY tools/pb/launch.py smoke "$service" --task "${PB_TASK:-boot}" 2>&1)"; rc=$?
echo "$out" | grep -v '^=== '
[ $rc -eq 0 ] || { echo "=== NO-GO: launch.py smoke $service did not end GO ==="; exit 1; }
[ "$service" = "headless-boot" ] || _sr_port_closed 47811 \
  || { echo "=== NO-GO: port 47811 still bound 30 s after launch.py stopped $service ==="; exit 1; }

if [ "$service" = "headless-boot" ]; then
  dir="$(mktemp -d)"
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
  # The revision a run records marks uncommitted sources (M0-D63): in a throwaway repository, summary.json's `git` is the
  # bare sha when clean — a rewritten assets/calibration.json and a log beside the sources not counted — and `<sha>-dirty`
  # once a source changed or a new one appeared, and `unknown` outside any repository. The binary embeds its assets, so
  # it runs from the repository's own folder.
  exe_abs="$PWD/build/target/release/sandbox-reactions$exe"
  repo="$dir/repo"; mkdir -p "$repo/crates/x" "$repo/assets"
  echo "fn a() {}" > "$repo/crates/x/lib.rs"; echo '{}' > "$repo/assets/calibration.json"
  g() { git -C "$repo" -c user.name=t -c user.email=t@t "$@" >/dev/null; }
  g init -q; g add -A; g commit -q -m base
  sha="$(git -C "$repo" rev-parse --short HEAD)"
  run_git() {
    (cd "$repo" && "$exe_abs" headless --scene preset:sun --steps 1 --out "$dir/g" >/dev/null 2>&1) || return 1
    $PY -I -c 'import json,sys; print(json.load(open(sys.argv[1], encoding="utf-8"))["git"])' "$dir/g/summary.json"
  }
  expect_git() {
    got="$(run_git)"
    [ "$got" = "$2" ] || { echo "=== NO-GO: summary.json git is \"$got\" with $1, expected \"$2\" ==="; exit 1; }
    echo "git revision: $1 -> $got"
  }
  expect_git "a clean tree" "$sha"
  echo '{"measured": 1}' > "$repo/assets/calibration.json"; echo log > "$repo/run.log"
  expect_git "only assets/calibration.json rewritten and a log written" "$sha"
  echo "fn b() {}" > "$repo/crates/x/lib.rs"
  expect_git "a source changed" "$sha-dirty"
  g checkout -q -- crates/x/lib.rs; echo "fn c() {}" > "$repo/crates/x/new.rs"
  expect_git "a new source untracked" "$sha-dirty"
  rm -rf "$repo/.git"
  expect_git "no repository" "unknown"
  echo "--- summary.json (verbatim, as checked) ---"
  cat "$dir/summary.json"; echo
  echo "--- end summary.json ---"
fi
echo "boot $service: built in $built s, booted and checked in $((SECONDS - start - built)) s"
echo "=== GO ==="
