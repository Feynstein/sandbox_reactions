#!/usr/bin/env bash
# The capture scope's runner (M0-T8, m0_contrat.md §3.3): bash tests/capture/check.sh <case>. `sr_build_app` leaves the
# release binary in THIS tree's own build/target (as tests/smoke.sh does; in a copy, through the shared cache, M0-TR1a), then:
#   smoke  — tests/capture/smoke.json (paused, then running) on the box's window route: `xvfb-run` with `--adapter
#            llvmpipe` on linux-pc, `--offscreen-window` on win-laptop (R4, R15). Pass: exit 0; two PNGs of the window's
#            size (the IHDR matching captions.json's viewport) whose pixels DIFFER (a screenshot taken before the frame it
#            names is presented would show the previous state); a valid captions.json in review_page.py's shape.
#   actions — tests/capture/actions.json (M0-T85): one step per action built (pause, steps, preset, rung, slowdown, edge x2), run on the same
#            route. Pass: exit 0; a PNG per step, in the script's order, each the window's size; the top bar (§4.2's 44 px)
#            differs from the previous step's after `paused`, `rung`, `slowdown`, `rung-max` and `edge-leave` (a paused readout both times, so only the radios differ); the accent (the selected
#            rung's text) sits further right in `rung` than in `start`, and further right again in `rung-max`.
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
sr_build_app "$exe" || { echo "=== NO-GO: cargo build --release -p sr-app failed ==="; exit 1; }
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
  actions)
    out="$("${runner[@]}" "$bin" "${window[@]}" --capture tests/capture/actions.json --out "$dir" 2>&1)"; rc=$?
    echo "$out" | grep -v '^$' | tail -n 8
    [ $rc -eq 0 ] || { echo "=== NO-GO: the capture run exited $rc, expected 0 ==="; exit 1; }
    echo "$out" | grep -q '^SR-CAPTURE DONE 9 ' || { echo "=== NO-GO: no 'SR-CAPTURE DONE 9' line ==="; exit 1; }
    $PY -I - "$dir" <<'PY' || { echo "=== NO-GO: the actions did not reach the top bar ==="; exit 1; }
import json, struct, sys, zlib
d = sys.argv[1]
WANT = ["start", "paused", "dropped", "stepped", "rung", "slowdown", "rung-max", "edge", "edge-leave"]
caps = json.load(open(f"{d}/captions.json", encoding="utf-8"))
if [c.get("id") for c in caps] != WANT:
    sys.exit(f"captions.json ids {[c.get('id') for c in caps]}, expected {WANT}")
def read(name):
    raw = open(f"{d}/{name}.png", "rb").read(); pos, idat, ihdr = 8, b"", None
    while pos < len(raw):
        n, kind = struct.unpack(">I4s", raw[pos:pos + 8]); body = raw[pos + 8:pos + 8 + n]
        if kind == b"IHDR": ihdr = struct.unpack(">IIBBBBB", body)
        if kind == b"IDAT": idat += body
        pos += 12 + n
    w, h, depth, ctype = ihdr[:5][:4]
    if (depth, ctype) != (8, 2): sys.exit(f"{name}.png: expected 8-bit RGB, got depth {depth} type {ctype}")
    data, stride, rows, prev, i = zlib.decompress(idat), w * 3, [], bytearray(w * 3), 0
    for _ in range(h):
        f = data[i]; line = bytearray(data[i + 1:i + 1 + stride]); i += 1 + stride
        for x in range(stride):
            a = line[x - 3] if x >= 3 else 0; b = prev[x]; c = prev[x - 3] if x >= 3 else 0
            if f == 1: line[x] = (line[x] + a) & 255
            elif f == 2: line[x] = (line[x] + b) & 255
            elif f == 3: line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c; pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line); prev = line
    return w, h, rows
shots, size = {}, set()
for name in WANT:
    w, h, rows = read(name)
    size.add((w, h))
    bar = round(44 * w / 1760)                          # the top bar's height in the window's pixels (§4.2: 44 px of 1760)
    shots[name] = (w, [bytes(r) for r in rows[:bar]])
if len(size) != 1 or min(next(iter(size))) < 500:
    sys.exit(f"the PNGs' sizes: {sorted(size)}")
def accent_x(name):                                    # mean column of the accent-coloured pixels (#F2B33D, text anti-aliased)
    w, rows = shots[name]; xs = []
    for r in rows:
        for x in range(w):
            R, G, B = r[3 * x:3 * x + 3]
            if abs(R - 0xF2) < 50 and abs(G - 0xB3) < 50 and abs(B - 0x3D) < 70: xs.append(x)
    return sum(xs) / len(xs) if len(xs) >= 10 else None
for name, before in [("paused", "start"), ("rung", "stepped"), ("slowdown", "rung"), ("rung-max", "slowdown"), ("edge-leave", "edge")]:
    if shots[name][1] == shots[before][1]:
        sys.exit(f"the top bar of {name} is the same as {before}'s: the action did not reach the bar")
x_start, x_rung, x_max = accent_x("start"), accent_x("rung"), accent_x("rung-max")
if None in (x_start, x_rung, x_max):
    sys.exit(f"no accent text in the bar: start {x_start}, rung {x_rung}, rung-max {x_max}")
if not x_start < x_rung < x_max:
    sys.exit(f"the selected rung's accent should move right: start {x_start:.0f}, rung {x_rung:.0f}, rung-max {x_max:.0f}")
print(f"9 PNGs of {next(iter(size))}; the bar changed after each action; accent at x = {x_start:.0f} (x1), {x_rung:.0f} (x10), {x_max:.0f} (Max)")
PY
    echo "capture actions: ok"
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
  *) echo "=== NO-GO: usage: bash tests/capture/check.sh smoke|refuse|actions ==="; exit 1 ;;
esac
