#!/usr/bin/env python3
"""M0-R14 probe: spin a preset's dump — rigid rotation about its centre of mass, a fraction beta of full
rotational support at the Maclaurin disk's edge (Omega_0^2 = pi^2 G Sigma_c / (2a), SM-E6; G = 1).
usage: spin.py <in.bin> <out.bin> <beta> [<a>]    (beta 0 copies the state unchanged — the control)
Only §2.2's planes 1-3 change: mom = Sigma * Omega x r, energy += 1/2 Sigma |u|^2. Every other byte is kept."""
import json, math, struct, sys
from array import array

src, dst, beta = sys.argv[1], sys.argv[2], float(sys.argv[3])
raw = open(src, "rb").read()
assert raw[:8] == b"SRDUMP01", "not a dump"
hlen = struct.unpack("<I", raw[8:12])[0]
head = json.loads(raw[12:12 + hlen])
w, h = head["width"], head["height"]
n = w * h
body = array("f"); body.frombytes(raw[12 + hlen:])
sig = body[0:n]
m = sum(sig)
xc = sum(sig[i] * (i % w + 0.5) for i in range(n)) / m
yc = sum(sig[i] * (i // w + 0.5) for i in range(n)) / m
sc = max(sig)
a = float(sys.argv[4]) if len(sys.argv) > 4 else math.sqrt(3 * m / (2 * math.pi * sc))  # M = 2 pi Sc a^2 / 3
om0 = math.sqrt(math.pi ** 2 * sc / (2 * a))
om = math.sqrt(beta) * om0
spun = 0
for i in range(n):
    dx, dy = (i % w + 0.5) - xc, (i // w + 0.5) - yc
    if dx * dx + dy * dy > (a + 1) ** 2:
        continue
    s = sig[i]
    ux, uy = -om * dy, om * dx  # counter-clockwise on screen axes (x right, y down): L_z = x*p_y - y*p_x > 0
    body[n + i] += s * ux
    body[2 * n + i] += s * uy
    body[3 * n + i] += 0.5 * s * (ux * ux + uy * uy)
    spun += 1
with open(dst, "wb") as f:
    f.write(raw[:12 + hlen]); f.write(body.tobytes())
print(f"spin: step {head['step']} mass {m:.3f} centre ({xc:.3f}, {yc:.3f}) Sigma_c {sc:.4f} a {a:.3f} "
      f"Omega_0 {om0:.5f} beta {beta} Omega {om:.5f} edge_speed {om * a:.3f} cells {spun}")
