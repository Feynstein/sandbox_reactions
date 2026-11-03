#!/usr/bin/env python3
"""M0-R13 scratch: build one binary per candidate in /tmp/r13/w (the scratch copy), each edit reverted after its build."""
import pathlib, shutil, subprocess, sys

W = pathlib.Path("/tmp/r13/w")
OUT = pathlib.Path("/tmp/r13/bin")
OUT.mkdir(exist_ok=True)
FL = "crates/sr-engine/shaders/floors.wgsl"
FLOOR_OLD = """        let raw = eps_th_raw(sigma, mom, energy, comp.y_e, x_n);
        if (raw < eps_floor) {
            let e = sigma * (eps_floor + eps_cold(sigma, comp.y_e, x_n)) + 0.5 * dot(mom, mom) / sigma;"""


def floor_with(extra):
    return f"""        let raw = eps_th_raw(sigma, mom, energy, comp.y_e, x_n);
        var fl = eps_floor;
        {extra}
        if (raw < fl) {{
            let e = sigma * (fl + eps_cold(sigma, comp.y_e, x_n)) + 0.5 * dot(mom, mom) / sigma;"""


# the calibration's hash stamped to the candidate's assets (a probe: the calibration's values are today's, stale)
HASH = '"physics_hash": "56df8ade55f98f0e95b03c4c2ff2a643cd247c689401ab526a00246fb0fe1dea"'
CANDS = {
    # B: a heat floor for degenerate matter, ε_th ≥ 0.1 ε_cold where Σ ≥ σ_hold (the hold's η).
    "B": [(FL, FLOOR_OLD, floor_with("if (sigma >= 10.0) { fl = max(fl, 0.1 * eps_cold(sigma, comp.y_e, x_n)); }"))],
    # B9: a floor at the reading 10^9 K (T_sb 120.73) in dense neutron-bearing cells.
    "B9": [(FL, FLOOR_OLD, floor_with("if (sigma >= 10.0 && x_n > 0.0) { fl = max(fl, 120.73 * comp.inv_mu); }"))],
    # C: neutrino cooling's rate a hundred times slower.
    "C": [("assets/physics.json", '"a_nu": 1e-3,', '"a_nu": 1e-5,'),
          ("assets/calibration.json", HASH, HASH.replace("56df8ade55f98f0e95b03c4c2ff2a643cd247c689401ab526a00246fb0fe1dea", "8b91cce8041376791aaaa0c4dc215d934f0854a8fd4262f6634552cc8872bd48"))],
    # D: N_Fe's q a tenth of today's.
    "D": [("assets/reactions.json", '"q_ratio": -1.14,', '"q_ratio": -0.114,'),
          ("assets/calibration.json", HASH, HASH.replace("56df8ade55f98f0e95b03c4c2ff2a643cd247c689401ab526a00246fb0fe1dea", "9052d390fc15ce9fbcbbf1ddeac9a3242374a94ec143da23de341c2fd1316dac"))],
    # F: the hold takes iron once the cell has neutronized (X_n ≥ 0.5).
    "F": [(FL, "let iron = x[ID_FE] > 0.0;", "let iron = x[ID_FE] > 0.0 && x[ID_N] < 0.5;")],
}

for name in sys.argv[1:]:
    saved = {}
    try:
        for f, old, new in CANDS[name]:
            p = W / f
            text = saved.setdefault(f, p.read_text())
            cur = p.read_text()
            assert cur.count(old) == 1, (name, f, cur.count(old))
            p.write_text(cur.replace(old, new))
        r = subprocess.run(["/home/ybelanger/.cargo/bin/cargo", "build", "--release", "-p", "sr-app"], cwd=W,
                           env={"CARGO_TARGET_DIR": str(W / "target"), "PATH": "/usr/bin:/bin", "HOME": "/home/ybelanger"},
                           capture_output=True, text=True)
        print(name, "build", r.returncode, r.stderr.strip().splitlines()[-1] if r.stderr.strip() else "")
        if r.returncode == 0:
            shutil.copy(W / "target/release/sandbox-reactions", OUT / f"sr-{name}")
    finally:
        for f, text in saved.items():
            (W / f).write_text(text)
