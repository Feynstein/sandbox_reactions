"""M0-R12 scratch — does a hotter body ever draw redder? T swept 10³–10¹¹ K at each setting S, the drawn 8-bit sRGB of a uniform
field (its halo is its own colour when its brightest channel passes 0.8), read as CIE 1931 x (redder = larger x)."""
import sys
import warnings
import numpy as np
sys.path.insert(0, sys.argv[1])
import r12
import r12c

warnings.filterwarnings("ignore")
data = sys.argv[2]
table = r12.Scene(f"{data}/sun_ms").table
M = np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]])
T = np.logspace(3, 11, 1601)


def drawn(colour, b, sigma, keep_hue=False):
    c = colour * b[:, None] + r12.haze_term(np.full(len(b), sigma)) * (1 - b)[:, None]
    q = r12.quant(c)
    out = q + 0.6 * q * (q.max(axis=1) > 0.8)[:, None]
    out = out / np.maximum(out.max(axis=1, keepdims=True), 1) if keep_hue else np.clip(out, 0, 1)
    return r12.quant(out)


def x_of(rgb):
    xyz = r12.lin(rgb) @ M.T
    return xyz[:, 0] / np.maximum(xyz.sum(axis=1), 1e-12)


def worst(rgb, tol=0.005):
    """The largest rise of x over the lowest x of every cooler drawn colour (max channel ≥ 16/255), and where."""
    ok = rgb.max(axis=1) >= 16 / 255
    x = np.where(ok, x_of(rgb), np.nan)
    best, w, at = np.inf, 0.0, None
    for i in range(len(T)):
        if np.isnan(x[i]):
            continue
        if x[i] - best > w:
            w, at = x[i] - best, (T[i], T[int(np.nanargmin(np.where(np.arange(len(T)) < i, x, np.nan)))])
        best = min(best, x[i])
    return w, at


bb = r12.bb_lookup(table, T)
cands = {
    "G1": lambda s: (bb, r12c.b_g1(T, s), False),
    "G3": lambda s: (bb, r12c.b_g3(T, s), False),
    "G3-hue": lambda s: (bb, r12c.b_g3(T, s), True),
    "G4": lambda s: (r12.bb_lookup(table, T * 3000 / s), r12.b_today(T * 3000 / s), False),
}
for sigma, label in [(1e-6, "Σ = Σ_vac (haze ×0.25)"), (1.0, "Σ = 1 (full haze)")]:
    print(f"-- {label}")
    for name, f in cands.items():
        for s in [10, 100, 3e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10]:
            col, b, kh = f(s)
            w, at = worst(drawn(col, b, sigma, kh))
            where = f"at {at[0]:.3g} K vs {at[1]:.3g} K" if at else ""
            flag = "REDDER" if w > 0.005 else "ok"
            print(f"{name:7s} S {s:8.0e}  worst x rise {w:.4f} {flag:6s} {where}")
# The lead's case: what colour a 10⁹ K body (a new neutron star) draws at each setting, beside a 3,000 K body at the default.
print("-- a 1e9 K body's drawn colour (Σ = 1)")
ref = drawn(r12.bb_lookup(table, np.array([3000.0])), r12.b_today(np.array([3000.0])), 1.0)
print(f"3,000 K body at the default: {np.round(ref[0]*255).astype(int)} x {x_of(ref)[0]:.3f}")
for name, f in cands.items():
    for s in [3e3, 1e7, 1e8, 1e9, 1e10]:
        col, b, kh = f(s)
        i = np.argmin(np.abs(T - 1e9))
        d = drawn(col[i:i + 1], b[i:i + 1], 1.0, kh)
        print(f"{name:7s} S {s:8.0e}: {np.round(d[0]*255).astype(int)} x {x_of(d)[0]:.3f}")
