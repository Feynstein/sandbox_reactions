#!/usr/bin/env bash
# The `web` scope's case runner (G-WEB, m0_contrat.md §3.5, §6.2.1, §6.2.4; M0-T7): bash tests/web/check.sh build
#   build  `trunk build --release` of web/index.html into web/dist: dist holds index.html, the .wasm and the
#          wasm-bindgen glue; index.html keeps the canvas, the web.* texts verbatim and the inline WebGPU check, which
#          stands ahead of every use of the wasm; trunk's own auto-loader is gone (a page that fetched the wasm before
#          the check would half-boot in a browser without WebGPU).
# The page's behaviour in a browser is graded by M0-T15's smoke. Ends in `=== GO ===` or `=== NO-GO: <reason> ===`.
set -u
case_="${1:-}"
source "$(dirname "${BASH_SOURCE[0]}")/../cargo.sh"
root="$_sr_root"
die() { echo "web $case_: $1"; echo "=== NO-GO: $1 ==="; exit 1; }

[ "$case_" = build ] || die "unknown case '$case_' (build)"
command -v trunk >/dev/null 2>&1 || die "trunk not found on PATH"
command -v node >/dev/null 2>&1 || die "node not found on PATH (the post-build hook strip-autoload.mjs needs it)"
trunk --version | grep -q '^trunk 0\.21\.14' || die "trunk is not 0.21.14 (contract §1.2): $(trunk --version)"
cd "$root/web" || die "no web/ folder"
rm -rf dist
mkdir -p "$root/build"
trunk build --release --locked >"$root/build/web-trunk.log" 2>&1 || die "trunk build --release failed (build/web-trunk.log)"
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
echo "=== GO ==="
