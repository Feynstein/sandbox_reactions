#!/usr/bin/env python3
"""M0-V9's probe life, watched: one Sun-like preset life on the physics adapter, with a line per progress dump.

Run from the repo root (linux-pc):   python3 milestones/m0/tasks/M0-V9.watch_life.py
Options: --out DIR (build/v9-sun-life) · --max-steps N (600000) · --dump-every N (5000) · --adapter S ("RTX 5090")

It starts `sandbox-reactions headless --scene preset:sun --until ending ...` itself (Ctrl-C stops both), then, each time a dump
appears in DIR/dumps, reads the star's centre (the densest cell) in f64 with contract §2.3's equation of state, the constants read
from assets/physics.json and assets/elements.json (the reader of M0-V8's task file, independent of the engine), and prints:
the step, the sim time, the centre's Σ, T, hydrogen and helium fractions, the gas still in the star (cells with Σ ≥ sigma_obj),
a verdict and the ETAs. When the run ends it reads DIR/summary.json: the ending, the stages shown and the X_H,c 0.35 mark.
Nothing here changes the engine or the tree; DIR is under build/ (git-ignored). Standard library only.
"""
import argparse, json, math, os, signal, struct, subprocess, sys, threading, time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
M0 = 2.0 * math.pi * 1.0 * 40.0 * 40.0 / 3.0  # §2.1's unit mass, the preset sun's cloud (3351.03)
TJ2_LIFE_STEPS = 200_000  # reports/top_speed.md's Sun-like life, extrapolated (UNVERIFIED)


def load_constants():
    physics = json.load(open(os.path.join(ROOT, "assets", "physics.json")))
    species = json.load(open(os.path.join(ROOT, "assets", "elements.json")))["species"]
    a = [float(s["a"]) for s in species]
    z = [float(s.get("z") or 0) for s in species]
    return physics, a, z


def cold_p(x, k1, k2):
    if x <= 0 or k1 <= 0 or k2 <= 0:
        return 0.0
    return 1.0 / (1.0 / (k1 * x * x) + 1.0 / (k2 * x ** 1.5))


def cold_u(x, k1, k2):  # u(x) = x ∫0^x P(s)/s² ds in closed form (eos.rs's cold_energy_exact)
    if x <= 0 or k1 <= 0 or k2 <= 0:
        return 0.0
    y = k1 / k2 * math.sqrt(x)
    g = sum((1 if k % 2 == 0 else -1) * y ** k / k for k in range(2, 10)) if y < 1e-3 else y - math.log1p(y)
    return x * 2.0 * k2 * k2 / k1 * g


def read_dump(path):
    with open(path, "rb") as f:
        raw = f.read()
    if raw[:8] != b"SRDUMP01":
        raise ValueError("not a dump")
    (n,) = struct.unpack_from("<I", raw, 8)
    head = json.loads(raw[12:12 + n].decode("utf-8"))
    cells = head["width"] * head["height"]
    off = 12 + n
    planes = [struct.unpack_from("<%df" % cells, raw, off + 4 * cells * c) for c in range(14)]
    return head, planes


def centre(head, planes, physics, a, z):
    sig = planes[0]
    w = head["width"]
    best = max(range(len(sig)), key=lambda i: (sig[i], -i))  # the largest Σ, first in row-major order
    s = sig[best]
    x = [planes[4 + k][best] for k in range(10)]
    inv_mu = sum(x[k] * (z[k] + 1) / a[k] for k in range(10))
    y_e = sum(x[k] * z[k] / a[k] for k in range(10))
    u = (planes[1][best] / s, planes[2][best] / s)
    k1e, k2e, k1n, k2n = physics["k1e"], physics["k2e"], physics["k1n"], physics["k2n"]
    eps_cold = (cold_u(y_e * s, k1e, k2e) + cold_u(x[9] * s, k1n, k2n)) / s
    eps_th = planes[3][best] / s - 0.5 * (u[0] ** 2 + u[1] ** 2) - eps_cold
    t = max(eps_th, physics["t_floor"] * inv_mu) / inv_mu
    pi_th = s * max(eps_th, 0.0)
    pi_cold = cold_p(y_e * s, k1e, k2e) + cold_p(x[9] * s, k1n, k2n)
    finite = all(math.isfinite(v) for v in (s, t, planes[3][best], *x))
    star = sum(v for v in sig if v >= physics["sigma_obj"])
    world = sum(sig)
    return {"step": head["step"], "t": head["sim_time"], "sigma_c": s, "t_c": t, "x_h": x[0], "y_he": x[1],
            "x_c": x[2], "x_o": x[3], "d_c": pi_cold / (pi_th + pi_cold) if pi_th + pi_cold > 0 else 0.0,
            "star": star, "world": world, "cell": (best % w, best // w), "finite": finite}


def fmt_time(seconds):
    if seconds is None or not math.isfinite(seconds) or seconds < 0:
        return "?"
    seconds = int(seconds)
    h, m = seconds // 3600, seconds % 3600 // 60
    return f"{h}h{m:02d}m" if h else f"{m}m{seconds % 60:02d}s"


def phase(c, physics):
    if c["x_h"] > 0.35:
        return "main sequence (H burning, X_H,c above the 0.35 mark)"
    if c["x_h"] > 1e-3:
        return "main sequence past the X_H,c 0.35 mark"
    if c["y_he"] > 1e-3:
        return "hydrogen gone at the centre — helium core / shell stages"
    return "helium gone at the centre — carbon/oxygen core, heading for a remnant"


def verdict(history, physics):
    c = history[-1]
    he_hot = 0.5 * physics["t_he"]
    if not c["finite"]:
        return "BAD", "a non-finite value at the centre — the run will stop with exit 5"
    if c["x_h"] > 0.35 and c["star"] < 0.5 * M0:
        return "BAD", f"the star holds {c['star']:.0f} of {M0:.0f} while still burning hydrogen — blown apart?"
    # Helium burning shows as the centre's helium falling, not as T_c passing 0.5 T_He (where its rate only starts: M0-V9's
    # probe ran past it from step ~30,000 and reached X_H,c 0.35 near step 82,000 with helium still rising).
    if c["x_h"] > 0.35 and len(history) >= 2 and c["y_he"] < history[-2]["y_he"] - 1e-4 and c["t_c"] >= he_hot:
        return "WATCH", f"the centre's helium fell {history[-2]['y_he'] - c['y_he']:.4f} at T_c {c['t_c']:.2f} before X_H,c 0.35 — helium may be burning first, then t_eff_sun_sb cannot be read (exit 6)"
    if c["x_h"] > 0.35 and len(history) >= 4:
        fall = history[-4]["x_h"] - c["x_h"]
        if fall < 1e-4:
            return "WATCH", f"X_H,c fell only {fall:.1e} over the last {c['step'] - history[-4]['step']} steps — a stalled burn?"
    if c["x_h"] > 0.35:
        return "GOOD", "burning hydrogen, centre below helium's threshold"
    return "GOOD", "past X_H,c 0.35 (t_eff_sun_sb's mark)"


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--out", default=os.path.join("build", "v9-sun-life"))
    ap.add_argument("--max-steps", type=int, default=600_000)
    ap.add_argument("--dump-every", type=int, default=5000)
    ap.add_argument("--adapter", default="RTX 5090")
    args = ap.parse_args()
    os.chdir(ROOT)
    exe = os.path.join("build", "target", "release", "sandbox-reactions" + (".exe" if os.name == "nt" else ""))
    if not os.path.isfile(exe):
        sys.exit(f"V9-WATCH no binary at {exe}: build it first (~/.cargo/bin/cargo build --release --locked -p sr-app)")
    dumps = os.path.join(args.out, "dumps")
    if os.path.isdir(args.out) and os.listdir(args.out):
        sys.exit(f"V9-WATCH {args.out} is not empty — delete it or give another --out, so two runs never mix")
    os.makedirs(dumps, exist_ok=True)
    physics, a, z = load_constants()
    cmd = [exe, "headless", "--scene", "preset:sun", "--until", "ending", "--max-steps", str(args.max_steps),
           "--dump-every", str(args.dump_every), "--adapter", args.adapter, "--out", args.out]
    print("V9-WATCH command:", " ".join(f'"{c}"' if " " in c else c for c in cmd), flush=True)
    print(f"V9-WATCH a line every {args.dump_every} steps; the life ends by itself (a remnant held 10 t.u.) or at --max-steps "
          f"{args.max_steps}. Ctrl-C stops both.", flush=True)
    log = open(os.path.join(args.out, "headless.stdout.log"), "w", encoding="utf-8")
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)

    def pump():
        for line in proc.stdout:
            log.write(line)
            log.flush()
            if line.startswith(("SR-ADAPTER", "SR-ERROR", "SR-HEADLESS", "SR-WARN")):
                print("  " + line.rstrip(), flush=True)

    reader = threading.Thread(target=pump, daemon=True)
    reader.start()
    started, seen, history = time.time(), set(), []
    try:
        while True:
            done = proc.poll() is not None
            names = sorted((n for n in os.listdir(dumps) if n.startswith("s_") and n.endswith(".bin")), key=lambda n: int(n[2:-4]))
            fresh = [n for n in names if n not in seen]
            # A dump still being written is read on the next pass (its size settles); the last pass reads everything.
            for name in fresh if done else fresh[:-1] if fresh and time.time() - os.path.getmtime(os.path.join(dumps, fresh[-1])) < 2 else fresh:
                path = os.path.join(dumps, name)
                try:
                    head, planes = read_dump(path)
                except (ValueError, struct.error, OSError):
                    continue
                seen.add(name)
                c = centre(head, planes, physics, a, z)
                c["wall"] = os.path.getmtime(path) - started
                history.append(c)
                rate = None
                if len(history) >= 2 and history[-1]["wall"] > history[-2]["wall"]:
                    rate = (history[-1]["step"] - history[-2]["step"]) / (history[-1]["wall"] - history[-2]["wall"])
                eta_mark = "—"
                if c["x_h"] > 0.35 and len(history) >= 3 and rate:
                    dx = (history[-3]["x_h"] - c["x_h"]) / max(1, c["step"] - history[-3]["step"])
                    if dx > 0:
                        steps = (c["x_h"] - 0.35) / dx
                        eta_mark = f"~{steps:,.0f} steps, {fmt_time(steps / rate)}"
                cap = fmt_time((args.max_steps - c["step"]) / rate) if rate else "?"
                word, why = verdict(history, physics)
                print(f"[{word:5}] step {c['step']:>8,} ({c['step'] / TJ2_LIFE_STEPS:4.0%} of TJ2's ~200k guess) · t {c['t']:9.2f} · "
                      f"wall {fmt_time(c['wall'])} · {rate or 0:5.0f} steps/s", flush=True)
                print(f"        centre Σ {c['sigma_c']:8.3f}  T {c['t_c']:7.3f}  X_H {c['x_h']:.4f}  Y_He {c['y_he']:.4f}  "
                      f"X_C+O {c['x_c'] + c['x_o']:.4f}  d_c {c['d_c']:.2f}  · star {c['star']:7.1f} of {M0:.0f}  · {phase(c, physics)}", flush=True)
                print(f"        ETA X_H,c 0.35: {eta_mark} · worst case (the step cap): {cap} · {why}", flush=True)
            if done:
                break
            time.sleep(3)
    except KeyboardInterrupt:
        proc.send_signal(signal.SIGINT)
        proc.wait()
        print("V9-WATCH interrupted: the run was stopped, no summary.json", flush=True)
        return 130
    reader.join(timeout=5)
    code = proc.returncode
    meaning = {0: "done", 2: "no ending within --max-steps", 3: "no adapter or a GPU error", 4: "bad arguments",
               5: "a non-finite value in the state", 101: "a panic"}.get(code, "see headless.stdout.log")
    print(f"V9-WATCH headless exit {code} ({meaning}) after {fmt_time(time.time() - started)}", flush=True)
    path = os.path.join(args.out, "summary.json")
    if not os.path.isfile(path):
        print("V9-WATCH no summary.json — BAD", flush=True)
        return 1
    s = json.load(open(path))
    star = max(s.get("objects", []), key=lambda o: o.get("mass_sb", 0.0), default=None)
    ending = next((e for e in s.get("events", []) if e.get("kind") == "ending"), None)
    print(f"V9-WATCH steps {s['steps']:,} · sim time {s['sim_time']:.2f} · wall {fmt_time(s['wall_s'])} · until met: "
          f"{s['until']['met']} · max_gas_speed {s.get('max_gas_speed', 0):.2f} (c_sb {physics['c_sb']} must beat twice it)", flush=True)
    for e in s.get("events", []):
        print(f"  event {e['kind']:13} step {e['step']:>8,} t {e['sim_time']:9.2f} object {e['object']} {json.dumps(e.get('detail', {}))}", flush=True)
    if star:
        print("  stages shown: " + " → ".join(f"{st['stage']} ({st['sim_time']:.1f})" for st in star.get("stages", [])), flush=True)
        print(f"  X_H,c 0.35 mark: {json.dumps(star.get('at_x_h'))}", flush=True)
    if ending and ending.get("detail", {}).get("stage") == "white_dwarf" and star and star.get("at_x_h"):
        print(f"V9-WATCH GOOD — ended as a white dwarf at step {ending['step']:,}: the calibration's Sun-like life can work "
              f"(--max-steps ≥ {3 * ending['step']:,} would leave it room)", flush=True)
    elif ending:
        print(f"V9-WATCH BAD for the calibration — ended as {ending.get('detail', {}).get('stage')}"
              f"{'' if star and star.get('at_x_h') else ', and X_H,c never reached 0.35'}", flush=True)
    else:
        print("V9-WATCH BAD — no ending event", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
