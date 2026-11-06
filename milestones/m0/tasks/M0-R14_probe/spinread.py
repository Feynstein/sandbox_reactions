#!/usr/bin/env python3
"""M0-R14 probe: read a life's dumps for its rotation. usage: spinread.py <dumps dir> [<dumps dir> ...]
One line per dump: step, t, mass, L_z about the centre of mass (L_z = sum x p_y - y p_x, screen axes),
the core (r <= 4 cells of the densest cell): mass, L_z about that cell, Omega = L/I, its rotational support
Omega^2 r^2 / (G M_core / r) at r = 4 (G = 1); the densest cell's Sigma, x_n, its distance to the centre of mass; sinks; R50, the half-mass radius about the centre of mass."""
import json, math, os, re, struct, sys
from array import array

def read(path):
    raw = open(path, "rb").read()
    hlen = struct.unpack("<I", raw[8:12])[0]
    head = json.loads(raw[12:12 + hlen])
    body = array("f"); body.frombytes(raw[12 + hlen:])
    return head, body

R = 4
for d in sys.argv[1:]:
    print(f"== {d}")
    print("step    t        mass       L_z(cm)      L/L0   | core: M      L_z       Omega     support | Sig_max   x_n   off  | sinks | R50")
    files = sorted((int(m.group(1)), f) for f in os.listdir(d) if (m := re.fullmatch(r"s_(\d+)\.bin", f)))
    L0 = None
    for step, f in files:
        head, b = read(os.path.join(d, f))
        w, h = head["width"], head["height"]; n = w * h
        sig, px, py, xn = b[0:n], b[n:2 * n], b[2 * n:3 * n], b[13 * n:14 * n]
        m = sum(sig)
        xc = sum(sig[i] * (i % w + 0.5) for i in range(n)) / m
        yc = sum(sig[i] * (i // w + 0.5) for i in range(n)) / m
        L = sum((i % w + 0.5 - xc) * py[i] - (i // w + 0.5 - yc) * px[i] for i in range(n))
        L0 = L0 if L0 is not None else L
        k = max(range(n), key=sig.__getitem__)
        kx, ky = k % w, k // w
        mc = Lc = Ic = 0.0
        for y in range(max(0, ky - R), min(h, ky + R + 1)):
            for x in range(max(0, kx - R), min(w, kx + R + 1)):
                dx, dy = x - kx, y - ky
                if dx * dx + dy * dy > R * R:
                    continue
                i = y * w + x
                mc += sig[i]; Lc += dx * py[i] - dy * px[i]; Ic += sig[i] * (dx * dx + dy * dy)
        om = Lc / Ic if Ic else 0.0
        sup = om * om * R * R / (mc / R) if mc else 0.0
        off = math.hypot(kx + 0.5 - xc, ky + 0.5 - yc)
        rr = sorted(((i % w + 0.5 - xc) ** 2 + (i // w + 0.5 - yc) ** 2, sig[i]) for i in range(n) if sig[i] > 1e-3)
        acc = 0.0; r50 = 0.0
        for r2, s_ in rr:
            acc += s_
            if acc >= m / 2:
                r50 = math.sqrt(r2); break
        sinks = ";".join(f"m{s['m']:.0f}" for s in head.get("sinks", []))
        ratio = L / L0 if abs(L0) > 1e-9 else float("nan")
        print(f"{step:<7} {head['sim_time']:<8.3f} {m:<10.2f} {L:<12.5g} {ratio:<6.3f} | {mc:<8.1f} {Lc:<9.4g} {om:<9.4g} {sup:<7.3f} | "
              f"{sig[k]:<9.1f} {xn[k]:<5.2f} {off:<4.1f} | {sinks or '-'} | {r50:.1f}")
