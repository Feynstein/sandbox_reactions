"""M0-R12 scratch — the live legend mocked under the world (world at s = 2, 1200 px wide, §4.2), and the headline sheet."""
import sys
import warnings
import numpy as np
from PIL import Image, ImageDraw, ImageFont
sys.path.insert(0, sys.argv[1])
import r12
import r12c

warnings.filterwarnings("ignore")
data, out = sys.argv[2], sys.argv[3]
BG = (5, 7, 13)
TEXT = (230, 234, 242)
SEC = (154, 164, 184)
ACC = (242, 179, 61)
try:
    FONT = ImageFont.truetype("DejaVuSans.ttf", 12)
except OSError:
    FONT = ImageFont.load_default()


def kelvin_tick(e):
    """A decade tick: §4.12's unit words, no trailing zeros (a proposed tick format)."""
    if e < 6:
        return f"{10**e:,} K"
    if e < 9:
        return f"{10**(e-6):,} million K"
    return f"{10**(e-9):,} billion K"


def legend(width, colours, lo, hi, ticks, marks, title):
    """A 44 px strip: the title, a 12 px bar of `colours` over [lo, hi] (log₁₀), ticks, and marks (value, label, colour)."""
    im = Image.new("RGB", (width, 44), BG)
    d = ImageDraw.Draw(im)
    x0, x1 = 8, width - 8
    bar = (np.clip(colours, 0, 1) * 255).astype(np.uint8)
    strip = Image.fromarray(np.repeat(bar[None], 12, axis=0)).resize((x1 - x0, 12), Image.BILINEAR)
    im.paste(strip, (x0, 16))
    d.text((x0, 1), title, fill=SEC, font=FONT)
    pos = lambda v: x0 + (v - lo) / (hi - lo) * (x1 - x0)
    for v, label in ticks:
        x = pos(v)
        d.line([(x, 28), (x, 31)], fill=SEC)
        w = d.textlength(label, font=FONT)
        d.text((min(max(x - w / 2, x0), x1 - w), 31), label, fill=SEC, font=FONT)
    for v, label, col in marks:
        x = pos(v)
        d.polygon([(x - 5, 9), (x + 5, 9), (x, 15)], fill=col)
        d.line([(x, 15), (x, 28)], fill=col, width=2)
        w = d.textlength(label, font=FONT)
        d.text((min(max(x + 7, x0), x1 - w), 1), label, fill=col, font=FONT)
    return im


def compose(world, strip, path):
    w = Image.fromarray(np.round(np.clip(world, 0, 1) * 255).astype(np.uint8)).resize((1200, 800), Image.NEAREST)
    im = Image.new("RGB", (1200, 800 + 8 + 44), BG)
    im.paste(w, (0, 0))
    im.paste(strip, (0, 808))
    im.save(path)


lt = np.linspace(1, 10, 1200)
T = 10**lt
# Glow — the massive supergiant at S = 10⁸ K (G3), the cursor on a cell of the envelope.
sc = r12.Scene(f"{data}/massive_sg")
s = 1e8
world = r12.with_glow(r12.glow_cells(sc, r12c.b_g3(sc.t, s)))
cx, cy = 300 + 40, 200
bar = r12.bb_lookup(sc.table, T) * r12c.b_g3(T, s)[:, None] + r12.hexrgb(r12.HAZE) * 0.6 * (1 - r12c.b_g3(T, s))[:, None]
strip = legend(1200, bar, 1, 10, [(e, kelvin_tick(e)) for e in range(1, 11)],
               [(np.log10(s), "Full brightness at 100 million K", ACC),
                (np.log10(sc.t[cy, cx]), f"Cursor {sc.t[cy, cx]/1e6:.3g} million K", TEXT)],
               "Glow — temperature (each colour drawn at its brightness)")
compose(world, strip, f"{out}/legend_mock.glow.png")
# Heat — the same star over a range of 10⁴–10⁹ K (the player's, or Fit to view's).
lo, hi = 4.0, 9.0
heat = r12.heat(sc, lo, hi)
bar = r12.smap(r12.HEAT, (lt - lo) / (hi - lo))
strip = legend(1200, bar[(lt >= lo) & (lt <= hi)], lo, hi, [(e, kelvin_tick(e)) for e in range(4, 10)],
               [(np.log10(sc.t[cy, cx]), f"Cursor {sc.t[cy, cx]/1e6:.3g} million K", TEXT)],
               "Heat — temperature, 10,000 K to 1 billion K")
compose(heat, strip, f"{out}/legend_mock.heat.png")

# The headline: today beside the recommendation, each scene.
TILE = (400, 267)
picks = {"sun_ms": 1e6, "sun_giant150": 1e6, "massive_sg": 1e8, "massive_sn": 1e7, "remnant": 1e6}
rows = []
for n, s in picks.items():
    sc = r12.Scene(f"{data}/{n}")
    m = ~sc.vac
    hlo, hhi = r12c.auto_range(np.log10(sc.t), m, (0.5, 100))
    imgs = [(r12.with_glow(r12.glow_cells(sc, r12.b_today(sc.t))), f"{n} · Glow today (3,000 K)"),
            (r12.with_glow(r12.glow_cells(sc, r12c.b_g3(sc.t, s))), f"G3 · full brightness at {s:.0e} K"),
            (r12.heat(sc), "Heat today (10 K – 10 billion K)"),
            (r12.heat(sc, hlo, hhi), f"Heat fit to view 10^{hlo:.2f} – 10^{hhi:.2f} K")]
    row = []
    for img, label in imgs:
        t = Image.fromarray(np.round(np.clip(img, 0, 1) * 255).astype(np.uint8)).resize(TILE, Image.BILINEAR)
        d = ImageDraw.Draw(t)
        d.rectangle([0, 0, TILE[0], 16], fill=(0, 0, 0))
        d.text((4, 2), label, fill=(255, 255, 255), font=FONT)
        row.append(t)
    rows.append(row)
sheet = Image.new("RGB", (4 * TILE[0], len(rows) * TILE[1]))
for i, r in enumerate(rows):
    for j, t in enumerate(r):
        sheet.paste(t, (j * TILE[0], i * TILE[1]))
sheet.save(f"{out}/headline.png")
