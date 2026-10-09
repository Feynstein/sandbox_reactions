#!/usr/bin/env bash
# The capture scope's runner (M0-T8, m0_contrat.md §3.3): bash tests/capture/check.sh <case>. Builds the release binary
# into THIS tree's own build/target (as tests/smoke.sh does), then:
#   smoke  — tests/capture/smoke.json (paused, then running) on the box's window route: `xvfb-run` with `--adapter
#            llvmpipe` on linux-pc, `--offscreen-window` on win-laptop (R4, R15). Pass: exit 0; two PNGs of the window's
#            size (the IHDR matching captions.json's viewport) whose pixels DIFFER (a screenshot taken before the frame it
#            names is presented would show the previous state); a valid captions.json in review_page.py's shape.
#   refuse — an action not built yet, an unknown action and a bad id each → exit 4 and an `SR-ERROR` line naming the
#            culprit, before any window opens (no display needed).
# Ends in `=== GO ===` or `=== NO-GO: <reason> ===`.
source "$(dirname "${BASH_SOURCE[0]}")/../cargo.sh"
case_id="${1:-}"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) PY="py -3.12"; exe=".exe"; window=(--offscreen-window); runner=() ;;
  *) PY="python3"; exe=""; window=(--adapter llvmpipe)
     # winit prefers Wayland whenever WAYLAND_DISPLAY is set, whatever DISPLAY says: without the `env -u` the window opens
     # on the lead's desktop, not in the Xvfb (found by the lead, M0-T8).
     runner=(xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE) ;;
esac
cd "$_sr_root" || exit 1
unset CARGO_TARGET_DIR
cargo_out="$(command cargo build --release --locked -p sr-app --target-dir build/target 2>&1)" \
  || { echo "$cargo_out" | tail -n 25; echo "=== NO-GO: cargo build --release -p sr-app failed ==="; exit 1; }
bin="build/target/release/sandbox-reactions$exe"
dir="$(mktemp -d)"
trap 'rm -rf "$dir"' EXIT

case "$case_id" in
  smoke)
    out="$("${runner[@]}" "$bin" "${window[@]}" --capture tests/capture/smoke.json --out "$dir" 2>&1)"; rc=$?
    echo "$out" | grep -v '^$' | tail -n 8
    [ $rc -eq 0 ] || { echo "=== NO-GO: the capture run exited $rc, expected 0 ==="; exit 1; }
    echo "$out" | grep -q '^SR-CAPTURE DONE 2 ' || { echo "=== NO-GO: no 'SR-CAPTURE DONE 2' line ==="; exit 1; }
    $PY -I - "$dir" <<'PY' || { echo "=== NO-GO: the captures do not match §3.3 ==="; exit 1; }
import json, os, struct, sys, zlib
d = sys.argv[1]
caps = json.load(open(os.path.join(d, "captions.json"), encoding="utf-8"))
want = ["paused", "running"]
if [c.get("id") for c in caps] != want:
    sys.exit(f"captions.json ids {[c.get('id') for c in caps]}, expected {want}")
keys = {"id", "file", "route", "path", "state", "viewport", "overflow", "caption", "flag"}
pixels, size = {}, set()
for c in caps:
    if not keys <= set(c):
        sys.exit(f"captions.json entry {c.get('id')} lacks {sorted(keys - set(c))}")
    if c["file"] != c["id"] + ".png" or not c["caption"] or c["flag"] is not False:
        sys.exit(f"captions.json entry {c['id']}: bad file, caption or flag")
    raw = open(os.path.join(d, c["file"]), "rb").read()
    if raw[:8] != b"\x89PNG\r\n\x1a\n":
        sys.exit(f"{c['file']}: not a PNG")
    pos, idat, ihdr = 8, b"", None
    while pos < len(raw):
        n, kind = struct.unpack(">I4s", raw[pos:pos + 8])
        body = raw[pos + 8:pos + 8 + n]
        if zlib.crc32(kind + body) != struct.unpack(">I", raw[pos + 8 + n:pos + 12 + n])[0]:
            sys.exit(f"{c['file']}: bad CRC in {kind!r}")
        if kind == b"IHDR": ihdr = struct.unpack(">IIBBBBB", body)
        if kind == b"IDAT": idat += body
        pos += 12 + n
    w, h = ihdr[0], ihdr[1]
    if f"{w}x{h}" != c["viewport"]:
        sys.exit(f"{c['file']}: IHDR {w}x{h} but captions.json says {c['viewport']}")
    if w < 800 or h < 500:
        sys.exit(f"{c['file']}: {w}x{h} is not a window's size")
    pixels[c["id"]] = zlib.decompress(idat)
    size.add((w, h))
if len(size) != 1:
    sys.exit(f"the two PNGs differ in size: {sorted(size)}")
if pixels["paused"] == pixels["running"]:
    sys.exit("the two PNGs have identical pixels: the second shows the first's state")
print(f"2 PNGs of {size.pop()} with different pixels, captions.json valid")
PY
    echo "capture smoke: ok"
    echo "=== GO ==="
    ;;
  refuse)
    expect() { # <script text> <needle>
      printf '%s' "$1" > "$dir/s.json"
      out="$("$bin" --capture "$dir/s.json" --out "$dir/o" 2>&1)"; rc=$?
      [ $rc -eq 4 ] || { echo "$out" | tail -n 5; echo "=== NO-GO: expected exit 4 for $2, got $rc ==="; exit 1; }
      echo "$out" | grep '^SR-ERROR' | grep -q -- "$2" || { echo "$out" | tail -n 5; echo "=== NO-GO: no SR-ERROR line naming $2 ==="; exit 1; }
      ls "$dir/o"/*.png >/dev/null 2>&1 && { echo "=== NO-GO: a refused script left a PNG ==="; exit 1; }
    }
    expect '[{"id":"a","caption":"c","actions":[{"view":"heat"}],"frames":1}]' '"view"'
    expect '[{"id":"a","caption":"c","actions":[{"frobnicate":1}],"frames":1}]' '"frobnicate"'
    expect '[{"id":"-a","caption":"c","actions":[],"frames":1}]' '"-a"'
    echo "capture refuse: a not-built action, an unknown action and a bad id each refused, exit 4"
    echo "=== GO ==="
    ;;
  *) echo "=== NO-GO: usage: bash tests/capture/check.sh smoke|refuse ==="; exit 1 ;;
esac
