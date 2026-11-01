"""M0-R12 scratch — today's four views from the extracted planes against the binary's own frames (each channel within 1/255)."""
import sys
import numpy as np
from PIL import Image
sys.path.insert(0, sys.argv[1])
import r12

data = sys.argv[2]
for name in sys.argv[3:]:
    sc = r12.Scene(f"{data}/{name}")
    ours = {
        "glow": r12.with_glow(r12.glow_cells(sc, r12.b_today(sc.t))),
        "glow_cells": r12.glow_cells(sc, r12.b_today(sc.t)),
        "heat": r12.heat(sc),
        "element": r12.element(sc),
        "density": r12.density(sc),
    }
    for v in ["glow", "heat", "element", "density"]:
        got = np.asarray(Image.open(f"{data}/{name}/bin_{v}.png").convert("RGB"), dtype=np.float64)
        d = np.abs(ours[v] * 255 - got)
        worst = d.max()
        over = (d.max(axis=2) > 1.0).mean() * 100
        y, x = np.unravel_index(d.max(axis=2).argmax(), d.shape[:2])
        print(f"{name:13s} {v:8s} max {worst:6.2f} levels · cells over 1 level {over:7.4f} % · worst at ({x},{y}) T_K {sc.t[y,x]:.4g} Σ {sc.sigma[y,x]:.3g}")
