#!/usr/bin/env bash
# The `web` scope's case runner (G-WEB, m0_contrat.md §3.5, §6.2.1, §6.2.4; M0-T7): bash tests/web/check.sh build
#   build  `trunk build --release` of web/index.html into web/dist: dist holds index.html, the .wasm and the
#          wasm-bindgen glue; index.html keeps the canvas, the web.* texts verbatim and the inline WebGPU check, which
#          stands ahead of every use of the wasm; trunk's own auto-loader is gone (a page that fetched the wasm before
#          the check would half-boot in a browser without WebGPU).
#   ready      the page in the box's installed Chrome (web/smoke.mjs, §6.2.3): `launch.py start web`, then `/` reaches srState
#              "ready" within 30 s; srAdapter and Chrome's own adapter go in the log (M0-T15)
#   no-webgpu  `/?no-webgpu=1` shows srState "no-webgpu" and the three web.no_webgpu_* texts exactly (M0-T15)
# The scope's cases run in parallel, yet they share one dist, one port (launch.json's 47812) and one pid file. Two locks, each
# held only for what needs it (M0-TR1b), always taken in this order and released on every exit path:
#   the tree's build lock (build/web-build.lock.d) - the dist from its build to the end of the smoke that serves it: `trunk
#     build` (its `rm -rf dist` included), `npm ci`, the launch and the smoke; no case rebuilds dist under another's smoke
#   the port lock (keyed on 47812, outside any tree) - taken after the build and `npm ci`, held for `launch.py start web`, the
#     smoke and the stop, so one red-arm copy's trunk run overlaps another copy's Chrome run
# The shared target's lock (tests/cargo.sh) still covers a whole trunk run, wasm-opt included. `ready` and `no-webgpu` serve the
# dist of a build that is fresh against every input (build/web-built.stamp), else they build it first - a smoke never runs
# against a stale dist. In a red-arm copy (no .git) build/ is the arm's own: the first case to build records its verdict
# (build/web-build.result) and its siblings reuse it while no input is newer - the arm runs trunk once, pass or fail; `build`
# checks the reused dist again. In the real tree `build` always rebuilds.
# Ends in `=== GO ===` or `=== NO-GO: <reason> ===`.
set -u
case_="${1:-}"
source "$(dirname "${BASH_SOURCE[0]}")/../cargo.sh"
root="$_sr_root"
die() { echo "web $case_: $1"; echo "=== NO-GO: $1 ==="; exit 1; }

case "$case_" in build | ready | no-webgpu) ;; *) die "unknown case '$case_' (build|ready|no-webgpu)" ;; esac
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) PY="py -3.12" ;;
  *) PY="python3" ;;
esac
# Node >= 20 (playwright-core's floor), picked here, never the caller's PATH alone (M0-D19): a non-interactive shell on
# linux-pc reads /usr/bin/node 18, while nvm's Node 20 is on the PATH only once ~/.bashrc has run. PATH's node when it is
# >= 20, else the newest install matching nvm's default alias; its bin goes first on the PATH, for trunk's hook and npm too.
node_major() { "$1" -e 'process.stdout.write(process.versions.node.split(".")[0])' 2>/dev/null; }
pick_node() {
  local found nvm alias cand
  found="$(command -v node 2>/dev/null)"
  [ -n "$found" ] && [ "$(node_major "$found")" -ge 20 ] 2>/dev/null && return 0
  nvm="${NVM_DIR:-$HOME/.nvm}"
  alias="$(cat "$nvm/alias/default" 2>/dev/null)"
  if [ -n "$alias" ]; then
    cand="$(ls -d "$nvm/versions/node/v${alias#v}"* 2>/dev/null | sort -V | tail -1)"
    if [ -n "$cand" ] && [ "$(node_major "$cand/bin/node")" -ge 20 ] 2>/dev/null; then
      export PATH="$cand/bin:$PATH"; return 0
    fi
  fi
  die "no Node >= 20: PATH's node is ${found:-absent}${found:+ $("$found" --version 2>/dev/null)}, nvm's default alias '${alias:-none}' in $nvm gives none (playwright-core needs 20)"
}
pick_node

task="${PB_TASK:-web}"
lock="${TMPDIR:-/tmp}/sr-web-47812.lock"
build_lock="$root/build/web-build.lock.d"
result="$root/build/web-build.result"
own_build=0 own_port=0 started=0
# sr_lock <dir>: a mkdir lock (Git Bash has no flock), broken when its holder's pid is gone, or when it has no pid a minute on
sr_lock() {
  until mkdir "$1" 2>/dev/null; do
    local holder; holder="$(cat "$1/pid" 2>/dev/null)"
    if [ -n "$holder" ]; then
      kill -0 "$holder" 2>/dev/null || rm -rf "$1"
    elif [ -n "$(find "$1" -maxdepth 0 -mmin +1 2>/dev/null)" ]; then
      rm -rf "$1"
    fi
    sleep 1
  done
  echo $$ >"$1/pid"
}
_sr_exit() {
  # web alone: launch.py's records are the box's, not the task's - a bare stop takes another scope's game too (M0-D28)
  [ "$started" = 1 ] && $PY tools/pb/launch.py stop web --task "$task" >/dev/null 2>&1
  [ "$own_port" = 1 ] && rm -rf "$lock.d" 2>/dev/null
  [ "$own_build" = 1 ] && rm -rf "$build_lock" 2>/dev/null
}
trap _sr_exit EXIT
trap 'exit 143' TERM INT HUP
take_port_lock() {
  if command -v flock >/dev/null 2>&1; then
    exec 8>"$lock" && flock 8   # released with the process
  else
    sr_lock "$lock.d"; own_port=1
  fi
}

# a build's failure, recorded for the arm's sibling cases (build/web-build.result) before the NO-GO
bdie() { echo "$1" >"$result"; die "$1"; }

build_web() {
command -v trunk >/dev/null 2>&1 || bdie "trunk not found on PATH"
command -v node >/dev/null 2>&1 || bdie "node not found on PATH (the post-build hook strip-autoload.mjs needs it)"
trunk --version | grep -q '^trunk 0\.21\.14' || bdie "trunk is not 0.21.14 (contract §1.2): $(trunk --version)"
cd "$root/web" || bdie "no web/ folder"
rm -rf dist
# trunk runs the cargo binary itself: in a red-arm copy it builds under the shared target's lock and touch (tests/cargo.sh)
sr_in_shared_target trunk build --release --locked >"$root/build/web-trunk.log" 2>&1 || bdie "trunk build --release failed (build/web-trunk.log)"
check_dist
touch "$root/build/web-built.stamp"
echo GO >"$result"
}

# the dist's checks: run after every build, and again by `build` on a dist its arm reused
check_dist() {
dist="$root/web/dist"
page="$dist/index.html"
[ -f "$page" ] || bdie "no index.html in $dist"
ls "$dist"/*.wasm >/dev/null 2>&1 || bdie "no .wasm in $dist"
ls "$dist"/*.js >/dev/null 2>&1 || bdie "no wasm-bindgen .js glue in $dist"
[ "$(head -c 4 "$dist"/*.wasm | od -An -c | tr -d ' ')" = '\0asm' ] || bdie "the .wasm is not a wasm module"

grep -q '<canvas id="sr-canvas"' "$page" || bdie "index.html lost <canvas id=\"sr-canvas\">"
grep -q 'navigator\.gpu' "$page" || bdie "index.html has no navigator.gpu check"
grep -q 'requestAdapter({ *powerPreference: *"high-performance" *})' "$page" || bdie "index.html does not await requestAdapter({powerPreference: \"high-performance\"})"
grep -q 'no-webgpu' "$page" || bdie "index.html has no ?no-webgpu=1 hook or srState \"no-webgpu\""
for text in \
  'Sandbox Reactions needs WebGPU' \
  'Your browser does not support WebGPU yet. Sandbox Reactions uses it to run its physics on your graphics card.' \
  'It runs in Google Chrome on Windows, macOS and ChromeOS (and on Linux with recent Intel or NVIDIA graphics), in Safari 26, and in Firefox on Windows and macOS.' \
  'Loading Sandbox Reactions…'; do
  grep -qF "$text" "$page" || bdie "index.html lacks the text '${text:0:40}…' verbatim (§4.1 web.*)"
done
# Nothing may fetch or start the wasm before the check: trunk's auto-loader and preloads are stripped, and the first
# mention of the glue or the wasm comes after the check.
if grep -q 'TrunkApplicationStarted\|rel="modulepreload"\|rel="preload"' "$page"; then bdie "trunk's auto-loader or preload for the wasm is still in index.html"; fi
gpu_line="$(grep -n 'navigator\.gpu' "$page" | head -1 | cut -d: -f1)"
wasm_line="$(grep -n 'sandbox-reactions\(_bg\.wasm\|\.js\)' "$page" | head -1 | cut -d: -f1)"
[ -n "$wasm_line" ] || bdie "index.html never loads the wasm"
[ "$gpu_line" -lt "$wasm_line" ] || bdie "index.html mentions the wasm (line $wasm_line) before the WebGPU check (line $gpu_line)"
echo "web build: $(trunk --version), $(du -h "$dist"/*.wasm | cut -f1) wasm; check on line $gpu_line, wasm on line $wasm_line"
}

# no input newer than $1 (a web/ source, a crate, the lockfile, the toolchain pin)
inputs_older_than() {
  [ -f "$1" ] && [ -z "$(find "$root/web/index.html" "$root/web/Trunk.toml" "$root/web/strip-autoload.mjs" "$root/crates" "$root/Cargo.toml"     "$root/Cargo.lock" "$root/rust-toolchain.toml" "$root/.cargo" -newer "$1" -type f -print -quit 2>/dev/null)" ]
}
# fresh = built, and no input newer than the build
dist_is_fresh() { [ -f "$root/web/dist/index.html" ] && inputs_older_than "$root/build/web-built.stamp"; }

mkdir -p "$root/build"
sr_lock "$build_lock"; own_build=1
reuse=""
if [ ! -e "$root/.git" ] && inputs_older_than "$result"; then   # a red-arm copy: a sibling case's verdict on these inputs
  reuse="$(cat "$result")"
  [ "$reuse" = GO ] && ! dist_is_fresh && reuse=""
fi
case "$reuse" in
  GO) [ "$case_" = build ] && check_dist ;;
  "") if [ "$case_" = build ] || ! dist_is_fresh; then build_web; fi ;;
  *) die "$reuse (this arm's build, reused)" ;;
esac
if [ "$case_" = build ]; then
  echo "=== GO ==="
  exit 0
fi

# ready, no-webgpu: serve web/dist through launch.py's `web` service and run web/smoke.mjs against it
command -v node >/dev/null 2>&1 || die "node not found on PATH (web/smoke.mjs)"
cd "$root/web" || die "no web/ folder"
if [ ! -d node_modules/playwright-core ]; then   # a red-arm scratch copy leaves node_modules out by name
  npm ci --ignore-scripts --no-audit --no-fund --prefer-offline >"$root/build/web-npm.log" 2>&1     || die "npm ci failed (build/web-npm.log)"
fi
cd "$root" || exit 1
take_port_lock
started=1
start="$($PY tools/pb/launch.py start web --task "$task" 2>&1)" || { echo "$start"; die "launch.py start web did not end GO"; }
echo "$start" | grep -v '^=== '
node web/smoke.mjs "$case_" --base http://127.0.0.1:47812 --timeout-s 30; rc=$?
exit $rc
