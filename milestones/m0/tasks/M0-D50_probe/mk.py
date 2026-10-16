# writes a D50 candidate scene: §2.10's Sun-like cloud as a disk at M0 (T 4.7324: U_th = 0.1|W|), overrides from argv k=v
import json, sys
out = sys.argv[1]
ov = {k: float(v) for k, v in (a.split("=") for a in sys.argv[2:])}
scene = {"version": 1, "world": {"width": 600, "height": 400, "edge": "leave"},
         "objects": [{"kind": "disk", "x": 300, "y": 200, "a": 40, "sigma_c": 1.0, "t": 4.7324,
                      "composition": {"H": 0.7346, "He": 0.2485, "O": 0.0077, "C": 0.0092}, "v": [0, 0]}]}
if ov:
    scene["overrides"] = ov
json.dump(scene, open(out, "w"), indent=1)
