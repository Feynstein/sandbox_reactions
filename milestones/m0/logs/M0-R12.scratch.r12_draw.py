"""M0-R12 scratch — every candidate drawn from the real dumps beside today's picture, into images/r12/."""
import sys
import warnings
import numpy as np
from PIL import Image, ImageDraw
sys.path.insert(0, sys.argv[1])
import r12
import r12c

warnings.filterwarnings("ignore")
data, out = sys.argv[2], sys.argv[3]
SETTINGS = [3e3, 1e5, 1e6, 1e7, 1e8, 1e9]
TILE = (300, 200)


def png(img, path):
    Image.fromarray(np.round(np.clip(img, 0, 1) * 255).astype(np.uint8)).save(path)


def tile(img, label):
    im = Image.fromarray(np.round(np.clip(img, 0, 1) * 255).astype(np.uint8)).resize(TILE, Image.NEAREST)
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, TILE[0], 14], fill=(0, 0, 0))
    d.text((3, 2), label, fill=(255, 255, 255))
    return im


def sheet(rows, path):
    h, w = len(rows), max(len(r) for r in rows)
    s = Image.new("RGB", (w * TILE[0], h * TILE[1]), (40, 40, 40))
    for i, r in enumerate(rows):
        for j, t in enumerate(r):
            s.paste(t, (j * TILE[0], i * TILE[1]))
    s.save(path)


def k(s):
    return f"{s:.0e}".replace("+0", "").replace("+", "") + " K"


for name in sys.argv[4:]:
    sc = r12.Scene(f"{data}/{name}")
    today = r12.with_glow(r12.glow_cells(sc, r12.b_today(sc.t)))
    png(today, f"{out}/{name}.glow.today.png")
    auto = r12c.auto_s(sc)
    rows = []
    for cand, fn in [("G1 exposure", lambda s: r12.with_glow(r12.glow_cells(sc, r12c.b_g1(sc.t, s)))),
                     ("G3 luminance", lambda s: r12.with_glow(r12.glow_cells(sc, r12c.b_g3(sc.t, s)))),
                     ("G3 hue-kept glow", lambda s: r12.with_glow(r12.glow_cells(sc, r12c.b_g3(sc.t, s)), keep_hue=True)),
                     ("G4 colour shift", lambda s: r12.with_glow(r12c.glow_g4(sc, s)))]:
        row = []
        for s in SETTINGS + [auto]:
            img = fn(s)
            tag = cand.split()[0] + (".hue" if "hue" in cand else "")
            label = f"{cand} · S {k(s)}" + (" (auto p99.5)" if s == auto else "")
            png(img, f"{out}/{name}.glow.{tag}.{'auto' if s == auto else f'{s:.0e}'}.png")
            row.append(tile(img, label))
        rows.append(row)
    sheet(rows, f"{out}/{name}.glow.sheet.png")

    # Heat, Density, Element.
    mask = ~sc.vac
    lt, ls = np.log10(sc.t), np.log10(sc.sigma)
    hlo, hhi = r12c.auto_range(lt, mask)
    dlo, dhi = r12c.auto_range(ls, mask)
    vac = r12.hexrgb(r12.BG)

    def masked(img):
        img = img.copy()
        img[sc.vac] = vac
        return img

    heat = [r12.heat(sc), r12.heat(sc, hlo, hhi), masked(r12c.equalised(r12.HEAT, lt, mask))]
    dens = [r12.density(sc), r12.density(sc, dlo, dhi), masked(r12c.equalised(r12.DENS, ls, mask))]
    elem = [r12.element(sc), r12.element(sc, dlo, dhi - dlo), r12.element(sc, dlo, dhi - dlo, 0.0)]
    for v, imgs in [("heat", heat), ("density", dens), ("element", elem)]:
        for tag, img in zip(["today", "auto", "equalised"] if v != "element" else ["today", "auto", "auto_nofloor"], imgs):
            png(img, f"{out}/{name}.{v}.{tag}.png")
    sheet([
        [tile(heat[0], "H0 today 1–10"), tile(heat[1], f"H2 auto {hlo:.2f}–{hhi:.2f}"), tile(heat[2], "H3 equalised")],
        [tile(dens[0], "D0 today −6–4"), tile(dens[1], f"D2 auto {dlo:.2f}–{dhi:.2f}"), tile(dens[2], "D3 equalised")],
        [tile(elem[0], "E0 today"), tile(elem[1], "E1 visibility over D2's range"), tile(elem[2], "E1 no 0.15 floor")],
    ], f"{out}/{name}.maps.sheet.png")
    print(f"{name}: auto S {auto:.3g} K · heat auto {hlo:.2f}–{hhi:.2f} · density auto {dlo:.2f}–{dhi:.2f}")
