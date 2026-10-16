# M0-D50 driver: one candidate run in chunks (headless --load-dump), each chunk's L = Δradiated/Δt and L_nuc =
# Δnuclear_released/Δt from summary.json's ledger, plus dumpstat's centre at the chunk's end.
# usage: py -3.12 -I chunk.py <name> <total steps> <chunk> [k=v overrides...]
import json, os, subprocess, sys, time
sys.stdout.reconfigure(encoding="utf-8")
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import dumpstat
REPO = r"C:\workspace\sandbox_reactions"
BIN = os.path.join(REPO, r"build\target\release\sandbox-reactions.exe")
name, total, chunk = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
ov = sys.argv[4:]
run = os.path.join(HERE, "runs", name)
os.makedirs(run, exist_ok=True)
scene = os.path.join(run, "scene.json")
subprocess.run([sys.executable, "-I", os.path.join(HERE, "mk.py"), scene] + ov, check=True)
log = open(os.path.join(run, "table.txt"), "a", encoding="utf-8")
def say(s):
    print(s, flush=True); log.write(s + "\n"); log.flush()
say(f"# {name} overrides {ov} total {total} chunk {chunk} · {time.strftime('%Y-%m-%dT%H:%M:%S')}")
prev, done = None, 0
import glob as _g
_old = sorted(d for d in _g.glob(os.path.join(run, "c*")) if _g.glob(os.path.join(d, "dumps", "s_*.bin")))
if _old:
    _s = json.load(open(os.path.join(_old[-1], "summary.json"), encoding="utf-8"))
    done = _s["ledger"]["end"]["step"]
    prev = os.path.join(_old[-1], "dumps", f"s_{done}.bin")
    say(f"# resumed at step {done}")
while done < total:
    n = min(chunk, total - done)
    out = os.path.join(run, f"c{done + n:07d}")
    cmd = [BIN, "headless", "--scene", scene, "--steps", str(n), "--out", out, "--adapter", "RTX 4080"]
    if prev:
        cmd += ["--load-dump", prev]
    t0 = time.time()
    p = subprocess.run(cmd, capture_output=True, text=True)
    wall = time.time() - t0
    if p.returncode != 0:
        say(f"exit {p.returncode}: {p.stdout[-800:]} {p.stderr[-800:]}"); break
    s = json.load(open(os.path.join(out, "summary.json"), encoding="utf-8"))
    a, b = s["ledger"]["start"], s["ledger"]["end"]
    dt = b["sim_time"] - a["sim_time"]
    L = (b["energy"]["radiated"] - a["energy"]["radiated"]) / dt
    Ln = (b["energy"]["nuclear_released"] - a["energy"]["nuclear_released"]) / dt
    esc = b["mass"]["escaped"] + b["mass"]["vacuum_reset"]
    step = b["step"]
    old_prev = prev
    prev = os.path.join(out, "dumps", f"s_{step}.bin")
    if old_prev and os.path.exists(old_prev) and int(old_prev.rsplit("_", 1)[1].split(".")[0]) % 1000:
        os.remove(old_prev)
    r = dumpstat.stat(prev)
    warn = sum(1 for l in p.stdout.splitlines() if l.startswith("SR-WARN"))
    st = [o["stage"] for o in s["objects"]]
    say(f"step {step:>6} t {b['sim_time']:8.2f} L {L:10.4g} Lnuc {Ln:10.4g} nuc {b['energy']['nuclear_released']:10.4g}"
        f" | Σc {r['sc']:7.3f} Tc {r['tc']:8.3f} Tmax {r['tmax']:8.2f} XHc {r['xhc']:.4f} XHcore {r['mh_core'] / r['m_core']:.4f}"
        f" d_c {r['dc']:.3f} Munb/M {r['munb'] / r['m']:.4f} M {r['m']:.1f} | {wall:5.1f}s warn {warn} stage {st}")
    # keep only the end dump of the last two chunks
    done += n
