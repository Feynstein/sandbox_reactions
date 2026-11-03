#!/usr/bin/env bash
# The capture scope's runner (M0-T8, m0_contrat.md §3.3): bash tests/capture/check.sh <case>. `sr_build_app` leaves the
# release binary in THIS tree's own build/target (as tests/smoke.sh does; in a copy, through the shared cache, M0-TR1a), then:
#   smoke  — tests/capture/smoke.json (paused, then running) on the box's window route: `xvfb-run` with `--adapter
#            llvmpipe` on linux-pc, `--offscreen-window` on win-laptop (R4, R15). Pass: exit 0; two PNGs of the window's
#            size (the IHDR matching captions.json's viewport) whose pixels DIFFER (a screenshot taken before the frame it
#            names is presented would show the previous state); a valid captions.json in review_page.py's shape.
#   actions — tests/capture/actions.json (M0-T85, M0-T127c): one step per action built (pause, steps, preset, rung, slowdown, edge x2, tool, element, paint, view, select, hover x2, glow_white, display_reset, fit, zoom, preset's at), run on the same
#            route. Pass: exit 0; a PNG per step, in the script's order, each the window's size; the top bar (§4.2's 44 px)
#            differs from the previous step's after `paused`, `rung`, `slowdown`, `rung-max` and `edge-leave` (a paused readout both times, so only the radios differ); the accent (the selected
#            rung's text) sits further right in `rung` than in `start`, and further right again in `rung-max`. The left panel
#            (§4.2's 220 px) differs after `tool` and after `element`, its accent (the selected tool and element) sitting lower
#            in `tool` than in `edge-leave` and lower again in `element`; the world's upper-left corner (35 % each way) differs in `paint`
#            (two blobs of iron painted at cells (100, 100) and (130, 100), far from the star at the centre); the right panel (§4.2's 300 px)
#            differs in `select` from `start`'s "No star yet" and holds more lit pixels (the star's readout, M0-T92), and differs in `hover`
#            from `select`'s one "Hover over a cell" line and holds more lit pixels (the Cell inspector's four lines for the star's
#            centre, M0-T93b); `hover-outside` (hover on cell (700, 500), off the 600 x 400 world, M0-D137) takes it back: its right panel holds
#            fewer lit pixels than `hover`'s by more than 100 (the inspector's four lines gone for its «Hover over a cell» line). Then the Massive preset at its supergiant (step 768, paused): the world (the centre between the
#            panels) after `glow_white` 10^8 K differs from the default Glow step's, after `display_reset` equals it pixel for pixel
#            (the two steps hold the same paused state), and Heat after `fit` differs from Heat's default (M0-T127c). Then `star-zoom` (M0-T129):
#            zoom 8 about cell (300, 200) of that same paused state — the world differs from `star-heat-fit`'s, and six cells read off both
#            shots (2 x 2 px whole, 16 x 16 px at z = 8) are the same colour, three colours at least.
#            Then `drop-at` and `drop-no-room` (M0-D145): the preset's `at` — a Sun-like star dropped at cell (80, 80) shows as a cloud about (400, 204) px,
#            over 120 px wide, in the world's upper-left; the one armed at (10, 10), too near the edge, shows `presets.no_room` in red under the world
#            (100 pixels or more, none after the drop that fit), the left panel holds the armed button, and the world is as `drop-at`'s.
#   refuse — a malformed hover, a malformed select, a malformed glow_white, range, fit, display_reset and zoom, an unknown action and a bad id each → exit 4 and an `SR-ERROR` line naming the
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
    echo "$out" | grep -q '^SR-CAPTURE DONE 24 ' || { echo "=== NO-GO: no 'SR-CAPTURE DONE 24' line ==="; exit 1; }
    why="$($PY -I - "$dir" 2>&1 <<'PY'
import json, struct, sys, zlib
d = sys.argv[1]
WANT = ["start", "paused", "dropped", "stepped", "rung", "slowdown", "rung-max", "edge", "edge-leave", "tool", "element", "paint", "view", "select", "hover", "hover-outside", "star-glow", "star-glow-white", "star-glow-reset", "star-heat", "star-heat-fit", "star-zoom", "drop-at", "drop-no-room"]
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
shots, full, size = {}, {}, set()
for name in WANT:
    w, h, rows = read(name)
    size.add((w, h))
    bar = round(44 * w / 1760)                          # the top bar's height in the window's pixels (§4.2: 44 px of 1760)
    shots[name] = (w, [bytes(r) for r in rows[:bar]])
    full[name] = (w, h, [bytes(r) for r in rows])
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
# The left panel (220 px of 1760, below the bar) and the world's upper-left corner (the centre between the panels).
w, h, _ = full["start"]
bar, left_w, status = round(44 * w / 1760), round(220 * w / 1760), round(24 * w / 1760)
def left_panel(name):
    return [r[:3 * left_w] for r in full[name][2][bar:h - status]]
def accent_y(name):                                    # mean row of the accent-coloured pixels in the left panel
    ys = []
    for y, r in enumerate(left_panel(name)):
        for x in range(left_w):
            R, G, B = r[3 * x:3 * x + 3]
            if abs(R - 0xF2) < 50 and abs(G - 0xB3) < 50 and abs(B - 0x3D) < 70: ys.append(y)
    return sum(ys) / len(ys) if len(ys) >= 10 else None
for name, before in [("tool", "edge-leave"), ("element", "tool")]:
    if left_panel(name) == left_panel(before):
        sys.exit(f"the left panel of {name} is the same as {before}'s: the action did not reach the panel")
y_edge, y_tool, y_element = accent_y("edge-leave"), accent_y("tool"), accent_y("element")
if None in (y_edge, y_tool, y_element):
    sys.exit(f"no accent text in the left panel: edge-leave {y_edge}, tool {y_tool}, element {y_element}")
if not y_edge < y_tool < y_element:
    sys.exit(f"the selected tool and element's accent should move down: edge-leave {y_edge:.0f}, tool {y_tool:.0f}, element {y_element:.0f}")
def quarter(name):                                     # the centre's upper-left 35 % each way: both blobs, not the star's halo
    rows = full[name][2][bar:bar + (h - bar - status) * 35 // 100]
    return [r[3 * left_w:3 * (left_w + (w - left_w) * 35 // 100)] for r in rows]
changed = sum(ra[i:i + 3] != rb[i:i + 3] for ra, rb in zip(quarter("element"), quarter("paint")) for i in range(0, len(ra), 3))
if changed < 300:
    sys.exit(f"the world's upper-left corner of paint differs from element's in {changed} pixels: the paint did not reach the world")
if left_panel("view") == left_panel("paint"):
    sys.exit("the left panel of view is the same as paint's: the view action did not reach the panel")
def right_panel(name):                                 # the right panel (300 px of 1760, below the bar, above the status bar)
    return [r[3 * (w - right_w):] for r in full[name][2][bar:h - status]]
def lit(name):                                         # pixels of the right panel that are not bg.panel (#10141E): text and the border
    return sum(any(abs(p[i] - q) > 24 for i, q in enumerate((0x10, 0x14, 0x1E))) for r in right_panel(name) for p in (r[i:i + 3] for i in range(0, len(r), 3)))
right_w = round(300 * w / 1760)
if right_panel("select") == right_panel("start"):
    sys.exit("the right panel of select is the same as start's: the star's readout never reached it")
if lit("select") <= lit("start") + 100:
    sys.exit(f"the right panel of select holds {lit('select')} lit pixels, start's note {lit('start')}: no star readout")
if right_panel("hover") == right_panel("select"):
    sys.exit("the right panel of hover is the same as select's: the Cell inspector never read the hovered cell")
if lit("hover") <= lit("select") + 100:
    sys.exit(f"the right panel of hover holds {lit('hover')} lit pixels, select's {lit('select')}: the Cell inspector's lines are missing")
if lit("hover-outside") >= lit("hover") - 100:
    sys.exit(f"the right panel of hover-outside holds {lit('hover-outside')} lit pixels, hover's {lit('hover')}: a cell off the world left the inspector on the last cell it read")
def centre(name):                                      # the world's area, between the panels: the display controls act here only
    return [r[3 * left_w:3 * (w - right_w)] for r in full[name][2][bar:h - status]]
if centre("star-glow-white") == centre("star-glow-reset"):
    sys.exit("the world of star-glow-white is the same as star-glow-reset's: glow_white did not reach the renderer")
if centre("star-glow-reset") != centre("star-glow"):
    sys.exit("the world of star-glow-reset differs from star-glow's: display_reset did not bring Glow back to its default")
if centre("star-heat-fit") == centre("star-heat"):
    sys.exit("the world of star-heat-fit is the same as star-heat's: fit did not change Heat's range")
# The zoom (M0-T129): `star-zoom` is `star-heat-fit`'s state (paused, the same view and range) at z = 8 around cell (300, 200). The world's
# place is the viewport at z = 1 (s = 2 in the 1760 x 940 window: x from 240, y from 44, 1200 x 800), so a cell (300 + dx, 200 + dy) is
# 2 x 2 px there and 16 x 16 px at z = 8, the cell (300, 200) centred on the viewport's centre (840, 444).
if w != 1760:
    sys.exit(f"the zoom's check reads the 1760-px window, got {w}")
def pixel(name, x, y):
    return full[name][2][y][3 * x:3 * x + 3]
if centre("star-zoom") == centre("star-heat-fit"):
    sys.exit("the world of star-zoom is the same as star-heat-fit's: zoom did not reach the view")
colours = set()
for dx, dy in [(0, 0), (-3, 2), (4, -4), (10, 7), (-12, -9), (20, 11)]:
    whole = pixel("star-heat-fit", 240 + 2 * (300 + dx), 44 + 2 * (200 + dy))
    zoomed = pixel("star-zoom", 840 + 16 * dx, 444 + 16 * dy)
    if whole != zoomed:
        sys.exit(f"the cell ({300 + dx}, {200 + dy}) is {tuple(whole)} whole and {tuple(zoomed)} at z = 8: the zoom is not about (300, 200) at 16 px a cell")
    colours.add(whole)
if len(colours) < 3:
    sys.exit(f"the zoom's probes read {len(colours)} colours: they prove nothing")
# The preset's `at` (M0-D145): `drop-at` is `star-zoom`'s paused state back at z = 1 (the zoom's own action) with a Sun-like star dropped by the
# panel's path on cell (80, 80) — 2 x 2 px a cell from (240, 44), so centred on (400, 204) px, a cloud of 39 cells' radius — and `drop-no-room`
# arms it again at cell (10, 10), too near the edge: the refusal on screen, the preset armed, no cell of the world changed.
if centre("drop-at") == centre("star-heat-fit"):
    sys.exit("the world of drop-at is the same as star-heat-fit's: the drop at (80, 80) never reached the world")
def moved(a, b):                                       # bounding box (px) of the pixels that differ in the world's upper-left, left of the iron blob and the supergiant
    xs, ys = [], []
    for y in range(44, 44 + 300):
        ra, rb = full[a][2][y], full[b][2][y]
        for x in range(240, 481):                      # up to the cloud's right edge (478): the iron blob painted at (130, 100) lies past it
            if ra[3 * x:3 * x + 3] != rb[3 * x:3 * x + 3]: xs.append(x); ys.append(y)
    return (min(xs), max(xs), min(ys), max(ys)) if xs else None
box = moved("star-heat-fit", "drop-at")
if box is None:
    sys.exit("the world's upper-left of drop-at is the same as star-heat-fit's: a preset with `at` was not dropped at (80, 80)")
cx, cy, wide = (box[0] + box[1]) / 2, (box[2] + box[3]) / 2, box[1] - box[0]
if abs(cx - 400) > 6 or abs(cy - 204) > 6 or wide < 120:
    sys.exit(f"the dropped star spans x {box[0]}..{box[1]}, y {box[2]}..{box[3]} px (centre {cx:.0f}, {cy:.0f}, {wide} wide): expected a cloud about (400, 204), over 120 px wide")
def error_text(name):                                  # pixels near ERROR_TEXT (#FF6B6B) in the strip under the legend's labels (the bar above is pink): the refusal's line
    return sum(abs(p[0] - 0xFF) < 40 and abs(p[1] - 0x6B) < 40 and abs(p[2] - 0x6B) < 40
               for r in full[name][2][h - status - 24:h - status] for p in (r[i:i + 3] for i in range(3 * left_w, 3 * (w - right_w), 3)))
if error_text("drop-at") > 5 or error_text("drop-no-room") < 100:
    sys.exit(f"the refusal's red text: {error_text('drop-at')} pixels after the drop that fit, {error_text('drop-no-room')} after the one at (10, 10): expected none, then 100 or more")
if left_panel("drop-no-room") == left_panel("drop-at"):
    sys.exit("the left panel of drop-no-room is the same as drop-at's: the refused preset is not armed")
if moved("drop-at", "drop-no-room") is not None:
    sys.exit("the world's upper-left changed between drop-at and drop-no-room: a drop that did not fit changed the world")
print(f"24 PNGs of {next(iter(size))}; the bar changed after each action; accent at x = {x_start:.0f} (x1), {x_rung:.0f} (x10), {x_max:.0f} (Max); the left panel's accent at y = {y_edge:.0f} (Brush), {y_tool:.0f} (Heat), {y_element:.0f} (Iron); {changed} pixels painted; the right panel {lit('start')} lit pixels without a star, {lit('select')} with one, {lit('hover-outside')} with the pointer off the world; Glow toned down differs, its Reset matches the default, Heat fitted differs; zoom 8 about (300, 200) matches the whole world on 6 probed cells ({len(colours)} colours), 16 px a cell; the preset dropped at (80, 80) spans x {box[0]}..{box[1]} px about ({cx:.0f}, {cy:.0f}), the one at (10, 10) refused ({error_text('drop-no-room')} red pixels, preset armed, world as it was)")
PY
)" || { echo "=== NO-GO: ${why##*$'\n'} ==="; exit 1; }
    echo "$why"
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
    expect '[{"id":"a","caption":"c","actions":[{"hover":"x"}],"frames":1}]' '"hover"'
    expect '[{"id":"a","caption":"c","actions":[{"select":"x"}],"frames":1}]' '"select"'
    expect '[{"id":"a","caption":"c","actions":[{"glow_white":"hot"}],"frames":1}]' '"glow_white"'
    expect '[{"id":"a","caption":"c","actions":[{"range":[5,3]}],"frames":1}]' '"range"'
    expect '[{"id":"a","caption":"c","actions":[{"fit":false}],"frames":1}]' '"fit"'
    expect '[{"id":"a","caption":"c","actions":[{"display_reset":1}],"frames":1}]' '"display_reset"'
    expect '[{"id":"a","caption":"c","actions":[{"zoom":9,"at":[1,1]}],"frames":1}]' '"zoom"'
    expect '[{"id":"a","caption":"c","actions":[{"zoom":2}],"frames":1}]' '"zoom"'
    expect '[{"id":"a","caption":"c","actions":[{"zoom":2,"at":[1,1],"x":3}],"frames":1}]' '"zoom"'
    expect '[{"id":"a","caption":"c","actions":[{"preset":"sun","at":"x"}],"frames":1}]' '"preset"'
    expect '[{"id":"a","caption":"c","actions":[{"preset":"sun","at":[1,1],"x":3}],"frames":1}]' '"preset"'
    expect '[{"id":"a","caption":"c","actions":[{"preset":"sun","x":3}],"frames":1}]' '"preset"'
    expect '[{"id":"a","caption":"c","actions":[{"preset":"moon","at":[1,1]}],"frames":1}]' '"preset"'
    expect '[{"id":"a","caption":"c","actions":[{"frobnicate":1}],"frames":1}]' '"frobnicate"'
    expect '[{"id":"-a","caption":"c","actions":[],"frames":1}]' '"-a"'
    echo "capture refuse: a malformed hover, select, glow_white, range, fit, display_reset, zoom and preset (a bad or extra key beside it), an unknown action and a bad id each refused, exit 4"
    echo "=== GO ==="
    ;;
  *) echo "=== NO-GO: usage: bash tests/capture/check.sh smoke|refuse|actions ==="; exit 1 ;;
esac
