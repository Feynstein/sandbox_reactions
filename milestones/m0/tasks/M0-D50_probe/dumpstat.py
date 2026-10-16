# M0-D50 probe (read-only): each dump read in f64 with contract §2.3's EOS and §1.4.2's kernel, independent of the engine.
# usage: py -3.12 -I dumpstat.py [K1E=..] <dump.bin>...
# prints step, t, the densest cell's Σ, T, X_H, d_c; the grid mass; M_unb (½|u|² + ε_th + φ > 0, Σ ≥ σ_vac);
# the hydrogen mass inside r ≤ 8 cells of the densest cell; the bound mass's mean X_H.
import json, math, struct, sys
import numpy as np

import glob
sys.stdout.reconfigure(encoding="utf-8")
args = sorted((f for a in sys.argv[1:] if "=" not in a for f in glob.glob(a)), key=lambda f: int(f.rsplit("_", 1)[1].split(".")[0]))
kv = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
K1E, K2E = float(kv.get("K1E", 21.3)), float(kv.get("K2E", 134.0))
K1N, K2N, T_FLOOR, SIG_VAC = 2.67, 58.2, 0.05, 1e-6
A = np.array([1, 4, 12, 16, 20, 24, 28, 32, 56, 1.0])
Z = np.array([1, 2, 6, 8, 10, 12, 14, 16, 26, 0.0])


def cold_p(x, k1, k2):
    x = np.maximum(x, 1e-300)
    return 1.0 / (1.0 / (k1 * x * x) + 1.0 / (k2 * x ** 1.5))


def cold_u(x, k1, k2):
    x = np.maximum(x, 1e-300)
    y = k1 / k2 * np.sqrt(x)
    g = np.where(y < 1e-3, y * y / 2 - y ** 3 / 3 + y ** 4 / 4, y - np.log1p(np.maximum(y, 1e-300)))
    return x * 2.0 * k2 * k2 / k1 * g


def read(path):
    raw = open(path, "rb").read()
    assert raw[:8] == b"SRDUMP01", path
    n = struct.unpack("<I", raw[8:12])[0]
    head = json.loads(raw[12:12 + n].decode("utf-8"))
    w, h = head["width"], head["height"]
    p = np.frombuffer(raw, dtype="<f4", count=14 * w * h, offset=12 + n).astype(np.float64).reshape(14, h, w)
    return head, p


_kern = {}


def potential(src):
    h, w = src.shape
    key = (h, w)
    if key not in _kern:
        H, W = 2 * h, 2 * w
        jj = np.arange(H); jj = np.where(jj < h, jj, jj - H)
        ii = np.arange(W); ii = np.where(ii < w, ii, ii - W)
        r = np.sqrt(jj[:, None] ** 2 + ii[None, :] ** 2)
        k = np.where(r > 0, -1.0 / np.maximum(r, 1e-30), -4 * math.log(1 + math.sqrt(2)))
        _kern[key] = np.fft.rfft2(k)
    H, W = 2 * h, 2 * w
    pad = np.zeros((H, W)); pad[:h, :w] = src
    return np.fft.irfft2(np.fft.rfft2(pad) * _kern[key], s=(H, W))[:h, :w]


def stat(path):
    head, p = read(path)
    sig = p[0]
    x = p[4:14]
    inv_mu = np.tensordot((Z + 1) / A, x, axes=1)
    y_e = np.tensordot(Z / A, x, axes=1)
    s = np.maximum(sig, 1e-300)
    u, v = p[1] / s, p[2] / s
    eps_cold = (cold_u(y_e * s, K1E, K2E) + np.where(x[9] > 0, cold_u(x[9] * s, K1N, K2N), 0)) / s
    eps_th = np.maximum(p[3] / s - 0.5 * (u * u + v * v) - eps_cold, T_FLOOR * inv_mu)
    T = eps_th / inv_mu
    gas = sig >= SIG_VAC
    phi = potential(np.where(gas, sig, 0.0))
    for snk in head.get("sinks", []):
        hh, ww = sig.shape
        yy, xx = np.mgrid[0:hh, 0:ww]
        phi = phi - snk["m"] / np.sqrt((xx - snk["x"]) ** 2 + (yy - snk["y"]) ** 2 + 4.0)
    unb = gas & (0.5 * (u * u + v * v) + eps_th + phi > 0)
    flat = sig.ravel()
    best = int(np.lexsort((np.arange(flat.size), -flat))[0])
    by, bx = divmod(best, sig.shape[1])
    pi_th = sig[by, bx] * eps_th[by, bx]
    pi_c = cold_p(y_e[by, bx] * sig[by, bx], K1E, K2E)
    d_c = pi_c / (pi_th + pi_c)
    yy, xx = np.mgrid[0:sig.shape[0], 0:sig.shape[1]]
    core = (xx - bx) ** 2 + (yy - by) ** 2 <= 64
    m = sig.sum()
    m_unb = sig[unb].sum()
    bound = gas & ~unb
    xh_bound = (sig * x[0])[bound].sum() / max(sig[bound].sum(), 1e-300)
    return dict(step=head["step"], t=head["sim_time"], sc=sig[by, bx], tc=T[by, bx], xhc=x[0][by, bx], dc=d_c,
                m=m, munb=m_unb, mh_core=(sig * x[0])[core].sum(), m_core=sig[core].sum(), xh_bound=xh_bound,
                tmax=T[gas].max(), cell=(bx, by))


if __name__ == "__main__":
    for path in args:
        r = stat(path)
        print(f"step {r['step']:>6} t {r['t']:8.3f}  Σc {r['sc']:8.4f}  Tc {r['tc']:8.3f}  Tmax {r['tmax']:9.3f}  "
              f"X_H,c {r['xhc']:.4f}  d_c {r['dc']:.3f}  M {r['m']:9.2f}  M_unb/M {r['munb'] / r['m']:.4f}  "
              f"M_core(r≤8) {r['m_core']:8.2f} X_H,core {r['mh_core'] / max(r['m_core'], 1e-300):.4f}  "
              f"X_H(bound) {r['xh_bound']:.4f}  cell {r['cell']}")
