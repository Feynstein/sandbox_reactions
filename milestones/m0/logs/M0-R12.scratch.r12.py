"""M0-R12 scratch renderer — never in the tree (kept as logs/M0-R12.scratch.py). Reads r12_extract's per-cell planes
(T_K, Σ, X through the CPU EOS and TempScale in f64) and draws today's four views (§1.11, §4.11, §0.5 [M0-D112], [M0-D113]) and
each display-range candidate in numpy, the glow's passes included."""
import json
import numpy as np

HEAT = ["#000004", "#3B0F70", "#8C2981", "#DE4968", "#FE9F6D", "#FCFDBF"]
DENS = ["#440154", "#3B528B", "#21918C", "#5EC962", "#FDE725"]
HAZE = "#2A3550"
BG = "#05070D"


def hexrgb(s):
    return np.array([int(s[1 + 2 * i:3 + 2 * i], 16) / 255.0 for i in range(3)])


def lin(c):
    c = np.asarray(c, dtype=np.float64)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def enc(c):
    c = np.clip(c, 0.0, 1.0)
    return np.where(c <= 0.0031308, 12.92 * c, 1.055 * np.power(c, 1 / 2.4) - 0.055)


class Scene:
    def __init__(self, d):
        self.meta = json.load(open(f"{d}/meta.json"))
        w, h = self.meta["width"], self.meta["height"]
        p = np.fromfile(f"{d}/cells.f32", dtype="<f4").astype(np.float64).reshape(12, h, w)
        self.t, self.sigma, self.x = p[0], p[1], p[2:]
        self.table = np.array(self.meta["blackbody"])  # 512 × 3, sRGB
        self.species = np.array(self.meta["species"]) / 255.0
        self.vac = self.sigma < self.meta["sigma_vac"]
        self.w, self.h = w, h


def bb_lookup(table, t):
    n = len(table)
    u = np.log(np.clip(t, 1000.0, 40000.0) / 1000.0) / np.log(40.0) * (n - 1)
    i = np.minimum(np.floor(u).astype(int), n - 2)
    f = (u - i)[..., None]
    return table[i] * (1 - f) + table[i + 1] * f


def smap(stops, p):
    s = np.clip(p, 0, 1) * (len(stops) - 1)
    i = np.minimum(np.floor(s).astype(int), len(stops) - 2)
    f = (s - i)[..., None]
    L = np.array([lin(hexrgb(c)) for c in stops])
    return enc(L[i] * (1 - f) + L[i + 1] * f)


def haze_term(sigma):
    return hexrgb(HAZE) * (np.clip((np.log10(sigma) + 8) / 8, 0, 1) * 0.6)[..., None]


def glow_cells(sc, b, colour=None):
    """§1.11's Glow cell image for a brightness b (today: 1 − exp(−(T/3000)⁴)); colour defaults to the table at T."""
    c = bb_lookup(sc.table, sc.t) if colour is None else colour
    out = c * b[..., None] + haze_term(sc.sigma) * (1 - b)[..., None]
    out[sc.vac] = hexrgb(BG)
    return np.clip(out, 0, 1)


def b_today(t, ts=3000.0):
    return 1 - np.exp(-((t / ts) ** 4))


def heat(sc, lo=1.0, hi=10.0):
    out = smap(HEAT, (np.log10(sc.t) - lo) / (hi - lo))
    out[sc.vac] = hexrgb(BG)
    return out


def density(sc, lo=-6.0, hi=4.0):
    out = smap(DENS, (np.log10(sc.sigma) - lo) / (hi - lo))
    out[sc.vac] = hexrgb(BG)
    return out


def element(sc, lo=-6.0, span=8.0, floor=0.15):
    L = np.einsum("khw,kc->hwc", sc.x, lin(sc.species))
    vis = np.clip((np.log10(sc.sigma) - lo) / span, floor, 1)
    out = enc(L * vis[..., None])
    out[sc.vac] = hexrgb(BG)
    return out


def quant(img):
    return np.round(np.clip(img, 0, 1) * 255) / 255


def halo(cells, threshold=0.8, sigma=3.0, radius=4):
    """glow.wgsl's bright pass, 9-tap Gaussian and bilinear upsample, in f64 (the model of tests/gpu/glow.rs)."""
    h, w, _ = cells.shape
    hh, hw = (h + 1) // 2, (w + 1) // 2
    pad = np.zeros((hh * 2, hw * 2, 3))
    pad[:h, :w] = cells
    kept = np.where((pad.max(axis=2) > threshold)[..., None], pad, 0.0)
    half = kept.reshape(hh, 2, hw, 2, 3).mean(axis=(1, 3))
    taps = np.exp(-np.arange(-radius, radius + 1) ** 2 / (2 * sigma * sigma))
    taps /= taps.sum()

    def blur(a, axis):
        out = np.zeros_like(a)
        n = a.shape[axis]
        for k, wk in zip(range(-radius, radius + 1), taps):
            src = np.roll(a, -k, axis=axis)
            idx = np.arange(n) + k
            ok = (idx >= 0) & (idx < n)
            shape = [1, 1, 1]
            shape[axis] = n
            out += wk * src * ok.reshape(shape)
        return out

    half = blur(blur(half, 1), 0)
    fy = (np.arange(h) + 0.5) * 0.5 - 0.5
    fx = (np.arange(w) + 0.5) * 0.5 - 0.5
    y0, x0 = np.floor(fy).astype(int), np.floor(fx).astype(int)
    ty, tx = (fy - y0)[:, None, None], (fx - x0)[None, :, None]
    cy = lambda y: np.clip(y, 0, hh - 1)
    cx = lambda x: np.clip(x, 0, hw - 1)
    a = half[cy(y0)][:, cx(x0)]
    b = half[cy(y0)][:, cx(x0 + 1)]
    c = half[cy(y0 + 1)][:, cx(x0)]
    d = half[cy(y0 + 1)][:, cx(x0 + 1)]
    return (a * (1 - tx) + b * tx) * (1 - ty) + (c * (1 - tx) + d * tx) * ty


def with_glow(cells, intensity=0.6, keep_hue=False):
    q = quant(cells)
    out = q + intensity * halo(q)
    if keep_hue:
        m = np.maximum(out.max(axis=2, keepdims=True), 1.0)
        return out / m
    return np.clip(out, 0, 1)
