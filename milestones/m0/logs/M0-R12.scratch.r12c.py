"""M0-R12 scratch — the display-range candidates (never in the tree; kept as logs/M0-R12.scratch.py with r12.py)."""
import numpy as np
import r12

# Wyman, Sloan & Shirley 2013's multi-lobe ȳ (§1.11's Table 1) and Planck, unnormalised: the blackbody's visible luminance.
_LAM = np.arange(380.0, 780.0 + 1e-9, 5.0)
_C2 = 1.438777e7  # nm·K


def _lobe(l, a, b, g, d):
    t = (l - b) * np.where(l < b, g, d)
    return a * np.exp(-0.5 * t * t)


_YBAR = _lobe(_LAM, 0.821, 568.8, 0.0213, 0.0247) + _lobe(_LAM, 0.286, 530.9, 0.0613, 0.0322)


def lum(t):
    """∫ B_λ(λ, T) ȳ(λ) dλ over 380–780 nm in 5 nm steps, up to a constant (2hc² and the step): ∝ T at the hot end."""
    t = np.asarray(t, dtype=np.float64)[..., None]
    x = np.minimum(_C2 / (_LAM * t), 700.0)
    return (_YBAR / _LAM**5 / np.expm1(x)).sum(axis=-1) * 1e20


# -- Glow candidates: a brightness b(T; S) for the setting S (a temperature), the colour the table's at T (G4 excepted).
def b_g1(t, s):
    """G1 — exposure: today's curve with 3,000 K replaced by S (the lead's «dimming», colours kept)."""
    return r12.b_today(t, s)


def b_log(t, s, decades=4.0):
    """The true visible luminance relative to S's, on an asinh stretch (Lupton et al. 2004) over about `decades` decades."""
    beta = 10.0 ** (-decades)
    x = lum(t) / lum(s)
    return np.clip(np.arcsinh(x / beta) / np.arcsinh(1.0 / beta), 0.0, 1.0)


def b_g3(t, s, decades=4.0):
    """G3 — today's curve at min(S, 3,000 K) times a dimming D(T; S): the asinh stretch of the visible luminance relative to
    S's, read at max(T, 3,000 K) — so D = 1 everywhere at S ≤ 3,000 K (exactly today's at 3,000 K) and, above it, matter
    cooler than 3,000 K keeps today's shading under one common dimming."""
    return r12.b_today(t, min(s, 3000.0)) * b_log(np.maximum(t, 3000.0), s, decades)


def glow_g4(sc, s):
    """G4 — the colour shift: today's whole formula at T · 3,000 / S (a hot body drawn as a cooler one)."""
    ts = sc.t * 3000.0 / s
    return r12.glow_cells(sc, r12.b_today(ts), colour=r12.bb_lookup(sc.table, ts))


def auto_s(sc, q=99.5):
    """G5 — automatic exposure: S at the q-th percentile of T_K over the matter in view."""
    return float(np.percentile(sc.t[~sc.vac], q))


# -- Heat and Density: the range [lo, hi] of the map, its log₁₀ quantity.
def auto_range(v, mask, q=(0.5, 99.5)):
    lo, hi = np.percentile(v[mask], q)
    return float(lo), float(max(hi, lo + 0.1))


def equalised(stops, v, mask, bins=256):
    """H3/D3 — histogram equalisation: the map at the quantity's rank among the matter in view."""
    vals = np.sort(v[mask])
    p = np.searchsorted(vals, v) / max(len(vals), 1)
    return r12.smap(stops, p)
