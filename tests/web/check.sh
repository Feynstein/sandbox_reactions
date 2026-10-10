#!/usr/bin/env bash
# The `web` scope's case runner (G-WEB, m0_contrat.md §3.5, §6.2.1, §6.2.4; M0-T7): bash tests/web/check.sh build
#   build  `trunk build --release` of web/index.html into web/dist: dist holds index.html, the .wasm and the
#          wasm-bindgen glue; index.html keeps the canvas, the web.* texts verbatim and the inline WebGPU check, which
#          stands ahead of every use of the wasm; trunk's own auto-loader is gone (a page that fetched the wasm before
#          the check would half-boot in a browser without WebGPU).
#   ready      the page in the box's installed Chrome (web/smoke.mjs, §6.2.3): `launch.py start web`, then `/` reaches srState
#              "ready" within 30 s; srAdapter and Chrome's own adapter go in the log (M0-T15)
#   no-webgpu  `/?no-webgpu=1` shows srState "no-webgpu" and the three web.no_webgpu_* texts exactly (M0-T15)
# The scope's cases run in parallel, yet they share one dist, one port (launch.json's 47812) and one pid file: every case takes a
# lock keyed on the port, outside any tree, and holds it to its end. `ready` and `no-webgpu` serve the dist of a build that is
# fresh against every input (build/web-built.stamp), else they build it first - a smoke never runs against a stale dist.
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
lock="${TMPDIR:-/tmp}/sr-web-47812.lock"
_sr_unlock() { rm -rf "$lock.d" 2>/dev/null; }
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

build_web() {
command -v trunk >/dev/null 2>&1 || die "trunk not found on PATH"
command -v node >/dev/null 2>&1 || die "node not found on PATH (the post-build hook strip-autoload.mjs needs it)"
trunk --version | grep -q '^trunk 0\.21\.14' || die "trunk is not 0.21.14 (contract §1.2): $(trunk --version)"
cd "$root/web" || die "no web/ folder"
rm -rf dist
mkdir -p "$root/build"
# trunk runs the cargo binary itself: in a red-arm copy it builds under the shared target's lock and touch (tests/cargo.sh)
sr_in_shared_target trunk build --release --locked >"$root/build/web-trunk.log" 2>&1 || die "trunk build --release failed (build/web-trunk.log)"
dist="$root/web/dist"
page="$dist/index.html"
[ -f "$page" ] || die "no index.html in $dist"
ls "$dist"/*.wasm >/dev/null 2>&1 || die "no .wasm in $dist"
ls "$dist"/*.js >/dev/null 2>&1 || die "no wasm-bindgen .js glue in $dist"
[ "$(head -c 4 "$dist"/*.wasm | od -An -c | tr -d ' ')" = '\0asm' ] || die "the .wasm is not a wasm module"

grep -q '<canvas id="sr-canvas"' "$page" || die "index.html lost <canvas id=\"sr-canvas\">"
grep -q 'navigator\.gpu' "$page" || die "index.html has no navigator.gpu check"
grep -q 'requestAdapter({ *powerPreference: *"high-performance" *})' "$page" || die "index.html does not await requestAdapter({powerPreference: \"high-performance\"})"
grep -q 'no-webgpu' "$page" || die "index.html has no ?no-webgpu=1 hook or srState \"no-webgpu\""
for text in \
  'Sandbox Reactions needs WebGPU' \
  'Your browser does not support WebGPU yet. Sandbox Reactions uses it to run its physics on your graphics card.' \
  'It runs in Google Chrome on Windows, macOS and ChromeOS (and on Linux with recent Intel or NVIDIA graphics), in Safari 26, and in Firefox on Windows and macOS.' \
  'Loading Sandbox Reactions…'; do
  grep -qF "$text" "$page" || die "index.html lacks the text '${text:0:40}…' verbatim (§4.1 web.*)"
done
# Nothing may fetch or start the wasm before the check: trunk's auto-loader and preloads are stripped, and the first
# mention of the glue or the wasm comes after the check.
if grep -q 'TrunkApplicationStarted\|rel="modulepreload"\|rel="preload"' "$page"; then die "trunk's auto-loader or preload for the wasm is still in index.html"; fi
gpu_line="$(grep -n 'navigator\.gpu' "$page" | head -1 | cut -d: -f1)"
wasm_line="$(grep -n 'sandbox-reactions\(_bg\.wasm\|\.js\)' "$page" | head -1 | cut -d: -f1)"
[ -n "$wasm_line" ] || die "index.html never loads the wasm"
[ "$gpu_line" -lt "$wasm_line" ] || die "index.html mentions the wasm (line $wasm_line) before the WebGPU check (line $gpu_line)"
echo "web build: $(trunk --version), $(du -h "$dist"/*.wasm | cut -f1) wasm; check on line $gpu_line, wasm on line $wasm_line"
touch "$root/build/web-built.stamp"
}

# fresh = built, and no input newer than the build (a web/ source, a crate, the lockfile, the toolchain pin)
dist_is_fresh() {
  [ -f "$root/build/web-built.stamp" ] && [ -f "$root/web/dist/index.html" ] || return 1
  [ -z "$(find "$root/web/index.html" "$root/web/Trunk.toml" "$root/web/strip-autoload.mjs" "$root/crates" "$root/Cargo.toml"     "$root/Cargo.lock" "$root/rust-toolchain.toml" "$root/.cargo" -newer "$root/build/web-built.stamp" -type f -print -quit 2>/dev/null)" ]
}

if [ "$case_" = build ]; then
  build_web
  echo "=== GO ==="
  exit 0
fi

# ready, no-webgpu: serve web/dist through launch.py's `web` service and run web/smoke.mjs against it
dist_is_fresh || build_web
command -v node >/dev/null 2>&1 || die "node not found on PATH (web/smoke.mjs)"
cd "$root/web" || die "no web/ folder"
if [ ! -d node_modules/playwright-core ]; then   # a red-arm scratch copy leaves node_modules out by name
  npm ci --ignore-scripts --no-audit --no-fund --prefer-offline >"$root/build/web-npm.log" 2>&1     || die "npm ci failed (build/web-npm.log)"
fi
cd "$root" || exit 1
task="${PB_TASK:-web}"
_sr_stop() { $PY tools/pb/launch.py stop --task "$task" >/dev/null 2>&1; _sr_unlock; }
trap _sr_stop EXIT
start="$($PY tools/pb/launch.py start web --task "$task" 2>&1)" || { echo "$start"; die "launch.py start web did not end GO"; }
echo "$start" | grep -v '^=== '
node web/smoke.mjs "$case_" --base http://127.0.0.1:47812 --timeout-s 30; rc=$?
exit $rc
