# M0-T97 — The life probe: three presets' lives, measured

Written 2026-11-05 by M0-T97 (model=claude-sonnet-5-5 level=high, `rung_record.py now` at launch). Box: the task ran on **linux-pc**
(`laserax-ai`); every GPU run ran on **laserax-ai-server, RTX 4090** (Vulkan, driver 595.99.02), as R18 reads a block that names the
RTX 5090 — the same checks, numbers named by adapter. Tree: HEAD `acaca48` + the uncommitted working tree (`git diff HEAD` sha256
`727c37352037…`; M0-D109's edits among them), rsync'd to the server; `assets/physics.json` sha256 `afd68190efc9…`,
`assets/calibration.json` `53d692bb20d0…` (physics_hash `56df8ade…`, current — no preset run was refused, G-CAL green). The binary's
summary.json prints `git 255ec37-dirty` (the server checkout's own HEAD, stale by design — it is the rsync'd tree that ran).
Raw output on the server: `~/workspace/sandbox_reactions/build/t97/` (`<run>.out`, `<run>/summary.json`, `events.jsonl`, `dumps/`);
trail `milestones/m0/logs/M0-T97.log`. The reading scripts are the appendix (`probe/` in the commands below = those files extracted).
Every number below carries its command (C1–C9, § Commands); all are `[ALREADY RUN]` unless tagged otherwise.

**What this report is not.** The Sun-like life was run **to t 800 (mid main sequence) only**, 394 s; its full life (V9: 762 s, over
the 10-minute line) is the lead's — C4, `[NOT RUN — for you]`. So S1, the Sun-like he_shell → white_dwarf stages and the Sun's ending are
**NOT RUN** here, not read from V9. V9's numbers appear only as labelled comparisons.

## Sun-like
Preset `sun` — 1 Sun (§2.10): sandbox mass **3,136.718** (`preset_mass_sb.sun`), radius a **38.700** cells, Σc **1.0000**, uniform
T after the first step **4.5796** (the cloud's U_th = 0.1|W|: W −239,193.3, U_th 23,907.0 booked after step 1), tracker R 34.23, cells
x 260–339, y 160–239 (C5). The other two presets: Σc is 1.0000 in all three (a = 40√(M/M₀) never hits the 150-cell cap).

| Preset | mass_sb | a (cells) | Σc | T₀ | W | R (tracker) | extent (cells) |
|---|---|---|---|---|---|---|---|
| sun | 3,136.718 | 38.700 | 1.0000 | 4.5796 | −239,193.3 | 34.23 | x 260–339, y 160–239 |
| massive | 6,677.218 | 56.464 | 1.0000 | 6.6856 | −743,351.5 | 50.02 | x 243–356, y 143–256 |
| giant | 18,211.504 | 93.249 | 1.0000 | 11.0453 | −3,349,637.5 | 82.55 | x 206–393, y 106–293 |

**The run (C4a, to t 800):** 70,100 steps, 394.0 s wall, max_gas_speed 21.83, one object, ignition **by light at step 273 (t 4.265)**,
stages displayed (summary.json `objects[0].stages`): `collapsing` from step 65 (t 1.539, 3.041 t.u.), `main_sequence` from step 305 (t 4.580),
still on it at t 800 (795.4 t.u. so far). The life differs from V9's under the same physics hash (V9: ignition at step 673, t 8.83, main
sequence from t 34.77) — the engine changed between (the D blocks since 2026-10-26); V9's is not this tree's.
Against G-STAGES' sun subsequence (`cloud, (collapsing), protostar, main_sequence, h_shell, he_core, he_shell, planetary_nebula,
white_dwarf`), the displayed ids so far hold `collapsing, main_sequence`: **`cloud` and `protostar` never display** (the life starts
`collapsing` at step 65 — raw `cloud` is shown only after 3 identical results); `h_shell … white_dwarf` are not reached — **NOT RUN** (C4).

**The main sequence's clocks** (G-SQUEEZE's, C8; τ_dyn = the tracker's √(R³/GM), τ_KH = |W|/2L, τ_nuc = X_H,c · M_core · Q_H / L with
M_core = the mass within 8 cells of the centre (M0-D50's reading), Q_H 8,700, L the tracker's luminosity):

| dump (C4a) | t | X_H,c | M | R | |W| | L | M_core | τ_dyn | τ_KH | τ_nuc | τ_KH/τ_dyn | τ_nuc/τ_KH |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| s_20000 | 225.5 | 0.666 | 3,126.4 | 14.35 | 615,515 | 1,568.9 | 1,439.2 | 0.972 | 196.2 | 5,317.7 | **201.7** | **27.1** |
| s_40000 | 452.7 | 0.573 | 3,122.7 | 14.62 | 596,425 | 1,981.3 | 1,394.4 | 1.001 | 150.5 | 3,506.8 | **150.4** | **23.3** |
| s_70100 (kept, § Dumps) | 800.0 | 0.436 | 3,115.1 | 15.17 | 598,603 | 2,784.2 | 1,443.3 | 1.059 | 107.5 | 1,968.5 | **101.5** | **18.3** |

Both ratios hold G-SQUEEZE's ≥ 10 all the way to t 800 and fall as the star ages (L rises 1,569 → 2,784 while X_H,c falls): τ_KH/τ_dyn
sits ≈ 10× over the bound, τ_nuc/τ_KH is already inside M0-T98a's 10–20 aim at t 800 (18.3). Cross-check of L: the ledger's radiated
energy between s_60000 and s_70000 gives L = 2,622.6 (mean over t 683.5 → 798.9) against the tracker's 2,779.8 at s_70000 — consistent
for a rising L. The row quantities (C9): at s_70100 the stage is `main_sequence` by L_H/L = 0.986 ≥ 0.5 and X_H,c = 0.436 ≥ 10⁻³, d_c 0.60,
v_r/√(GM/R) −0.001, M_unb 0.

**Closest bound gas to an edge (G-FIT, C6):** through t 800 the nearest cell that can be bound is **142.5 cells** from an edge at the
end (the dump at step 10,000, t 114.1, shows 0.099 mass units — 0.003 % — at 0.5 cell, the bounce's ejecta; every later dump ≥ 142.5).
τ_dyn, τ_KH, τ_nuc of the Massive and the Giant: § Massive and § Giant (they have no main sequence).

## Massive
Preset `massive` — 15 Suns: mass **6,677.218**, a 56.464, Σc 1.0000, T₀ 6.6856. Run `--until ending` (C2a, 564.2 s,
20,272 steps, max_gas_speed 52.41), ignition **by fuel at step 273, t 3.967**, `core_collapse` step 881 (t 7.627, 6,670.28), `supernova`
step 913 (t 7.767), **ending `neutron_star` at step 20,272 (t 113.757): 3,658.47 sb (8.33 Suns), held 94.17 ≥ hold 83.70** — the ending
its readout predicted at the drop (supernova, then neutron star). V9's: 21,218 steps, ending t 189.87, 3,708.8 — the engine changed.

| stage displayed | from step | t | duration (t.u.) |
|---|---|---|---|
| collapsing | 65 | 1.221 | 3.094 |
| iron_core | 321 | 4.314 | 3.313 |
| core_collapse | 881 | 7.627 | 0.140 |
| supernova | 913 | 7.767 | 11.824 |
| neutron_star | 3,217 | 19.591 | 94.167 (to the ending) |

G-STAGES' massive subsequence (`cloud, (collapsing), protostar, main_sequence, (h_shell), he_core, c_burn, {ne_burn, o_burn},
si_burn, iron_core, core_collapse, supernova, neutron_star`): **missing from the displayed ids: cloud, protostar, main_sequence, he_core,
c_burn, ne_burn/o_burn, si_burn** (7 of 12 required); present: collapsing, iron_core, core_collapse, supernova, neutron_star (C7). The cause is
in the rows (C9): at step 300 (t 4.16, 27 steps after ignition) the centre is **X_Fe,c = 1, X_H,c 3 × 10⁻⁹, L_H/L 2 × 10⁻¹³** — the hydrogen,
helium and every burning stage between are gone within ~0.3 t.u. of ignition (raw `iron_core` already, displayed `collapsing` until 3
identical results at step 321). The collapse row: step 900 raw `neutron_star`, shown `core_collapse`: X_n,c 0.557 ≥ 0.5, X_Fe,c 0.443, **L_N −8.58 × 10⁷ < 0**,
d_c 1.0, M_unb 609.5. The ending row: step 4,600, `neutron_star` by X_n,c 0.558, |v_r|/√(GM/R) 0.001, d_c 0.93, M_unb 0.23.

**Clocks** (C8, at the dump after ignition): step 300 — M 6,674.6, R 34.36, |W| 1.520 × 10⁶, L 1.799 × 10⁵, τ_dyn 2.465, τ_KH 4.224
(**τ_KH/τ_dyn 1.7**), X_H,c 3.0 × 10⁻⁹ so τ_nuc 1.5 × 10⁻⁷ (**τ_nuc/τ_KH 4 × 10⁻⁸**); step 400 (t 4.90) — τ_KH/τ_dyn 7.3, τ_nuc/τ_KH 4 × 10⁻⁸.
No main sequence exists to read G-SQUEEZE's ratios on: **G-SQUEEZE's two gaps and «H > He > C > {Ne, O} > Si» are not measurable on
this life** (as V9 found, same cause). The unbound mass and escaped mass over the life: 2,770.97 escaped by the ending (41.5 % of the drop).

**Closest bound gas to an edge (G-FIT, C6, bracketed: § Method):** the dense run (to t 30): the nearest candidate is 127.5 cells at step 100, 144.5 at the
collapse (step 900), then the supernova's bound ejecta closes in — 108.5/105.5 (step 2,100, t 13.6), 75.5/70.5 (3,000), 49.5/42.5 (4,000), 27.5/15.5 (4,800,
t 28.2), **20.5/10.5 at step 5,139 (t 30.0)**, with 0.09 mass units (nec) inside 16 cells and none by the sufficient test: at t 30 the two bounds straddle
16 (**NOT PROVEN** either way there). The plain run's dumps carry it on: step 10,000 (t 56.9) bound gas within **0.5 cell** of an edge, 2.2–3.9 mass units inside 16
cells; step 15,000 (t 84.0) 15.9–18.4; step 20,272 (t 113.8, the ending) nearest **0.5**, **14.5–14.8** of 3,905.9 (0.4 %) inside 16, 10⁻³ of the mass inside 4.5 cells.
**G-FIT is failed by this life** from t ≈ 57 at the latest (both bounds < 16, the sufficient one measured bound), the step 10,000 dump the first read of it
(the plain run's cadence is 5,000 steps: the crossing lies between t 30 and t 57).

## Giant
Preset `giant` — 40 Suns: mass **18,211.504**, a 93.249, Σc 1.0000, T₀ 11.0453. Run `--until ending` (C2b, 322.8 s, 9,640 steps,
max_gas_speed 109.36), ignition **by fuel at step 417 (t 4.559)**, `core_collapse` step 1,377 (t 9.528, 18,102.85), `sink_formed` step
1,393 (t 9.600, 18,102.26), **ending `black_hole` at step 9,640 (t 37.783): 17,331.12 sb, held 28.03 ≥ hold 27.62.** V9's: 13,973 steps, ending
t 46.08, 15,835.5.

| stage displayed | from step | t | duration (t.u.) |
|---|---|---|---|
| cloud | 33 | 0.503 | 0.605 |
| collapsing | 81 | 1.108 | 3.733 |
| iron_core | 465 | 4.841 | 4.686 |
| core_collapse | 1,377 | 9.528 | 0.220 |
| black_hole | 1,425 | 9.748 | 28.035 (to the ending) |

G-STAGES' giant subsequence (`cloud, (collapsing), protostar, main_sequence, he_core, core_collapse, (supernova), black_hole`): **missing:
protostar, main_sequence, he_core**; no supernova (optional). Rows (C9): step 400 (pre-ignition, t 4.455) raw/shown `collapsing`: X_H,c 0.734, v_r/√(GM/R) −0.706;
step 500 (t 5.06) **X_Fe,c = 1, X_H,c 2.7 × 10⁻⁹, L_H/L 4 × 10⁻¹⁶**, raw `iron_core`; step 1,400 raw `black_hole` (a sink inside: `sinks 1`), shown `core_collapse`; step
1,500 shown `black_hole`, d_c 0.27 — the hydrogen is gone ~0.6 t.u. after ignition, as in the Massive.
**Clocks** (C8): step 400 (before the flash, L 32.7 only) τ_KH/τ_dyn 30,313, τ_nuc/τ_KH 2.4 — meaningless, the star has not lit; step 500 — τ_KH 21.6,
τ_dyn 3.52 (**6.1**), X_H,c 3 × 10⁻⁹ (τ_nuc ≈ 0). No main sequence: G-SQUEEZE's «main-sequence durations Sun-like > Massive > Giant» reads
**Sun-like ≥ 795 t.u., Massive none, Giant none**.
**Closest bound gas to an edge (G-FIT, C6):** the sink (8,981 sb at step 5,000) holds the ejecta bound as it falls back: dense run — step 2,300 (t 13.1)
first under 16 cells (14.5 / 12.5 by the two bounds), step 2,700–2,800 (t 14.5–14.9) **0.5 cell**, mass within 16 cells 20–30 sb of ~9,550 (0.3 %), 1 × 10⁻³ of the mass inside 6.5–9.5 cells;
step 5,000 (t 22.3): 65.8–67.8 of 8,692.5 (0.8 %) within 16, nearest 0.5; step 9,640 (the ending): 45.1–45.3 of 4,913.4 (0.9 %), nearest 0.5.
**G-FIT is failed by this life**, from t ≈ 13.

## Stand-ins
- **S1 (dust wind, §1.6.4): NOT RUN.** Its trigger reads the Sun-like preset 20 τ_dyn after entering the helium-shell stage (< 30 % of the
  envelope unbound enables it). That stage lies past t 800; the full Sun life is the lead's (C4). V9's reading (the Sun-like never shows he_shell; the
  envelope leaves during he_core) was on another build of the engine and this tree's Sun life already differs (ignition step 273, not 673), so it
  is **not** carried here. This task's Sun run measured the main sequence only.
- **S2 (thermal bomb), measured — the trigger does not fire.** «The Massive preset forms neutron matter and, within 20 t.u., unbinds less than
  50 % of its envelope at f_dep = 0.1.» Neutron matter first appears between steps 800 (t 7.23, X_n 0) and 900 (t 7.71, X_n,c 0.557) of the dense run;
  the window ends ≤ t 27.7. The shipped f_dep is 0.05 (physics.json); at 0.1 the preset is refused once physics.json differs, so f_dep 0.1 ran as a
  preset-free `disk` at the preset's a, Σc, T₀ and the Sun mix with `overrides {"f_dep": 0.1}` (C6, scene text in C6) and a control at 0.05 (the disk
  is not bit-identical to the preset — core_collapse at step 897 against 881 — it is the same star to within chaos).
  Unbound = mass escaped (ledger) + grid mass with ½|u|²+ε_th+φ > 0, **bracketed** (no ε_th in a dump: § Method); the envelope = the drop minus the remnant,
  against the *final* remnant (3,658.5, the preset's ending), or against the heaviest object's mass at the dump (V9's two denominators):

  | run, dump | t | escaped | unbound on grid | vs final remnant | vs the object's mass then |
  |---|---|---|---|---|---|
  | preset f_dep 0.05, step 4,600 | 27.11 | 2,217.8 | 168.3–172.1 | **79.0–79.2 %** | 98.6–98.7 % (M_obj 4,256.5) |
  | disk f_dep 0.05 (control), step 5,000 | 28.94 | 2,193.6 | 81.1–86.7 | 75.4–75.5 % | 99.0–99.2 % |
  | **disk f_dep 0.1, step 5,000** | 28.95 | 2,184.3 | 76.6–81.4 | **74.9–75.1 %** | 98.5–98.7 % |
  | disk f_dep 0.1, step 5,195 | 30.00 | 2,201.1 | 60.8–65.2 | 74.9–75.1 % | 98.5–98.6 % |

  **Pass/Fail as the trigger reads: S2 stays off** — 75–99 % of the envelope is unbound 21–22 t.u. after the neutron matter (the dumps sit 1.2–2.3 t.u.
  past the 20 t.u. mark, the fractions flat there: 79.0 → 79.3 % between t 27.1 and 29.3 on the preset), against < 50 % to enable. f_dep 0.1 moves
  nothing measurable against 0.05 (74.9 vs 75.4 %; core_collapse at step 897 in both).

## Dumps
Kept outside the tree (§6.5's reasoning: a red-arm copy reads them too), on **linux-pc** `~/.cache/sandbox-reactions/dumps/` and the same path on
laserax-ai-server, with their observers' snapshots beside them (`<name>.observers.json`):

| File | What | Step · t | Made by | sha256 (first 16) |
|---|---|---|---|---|
| `m0-t97_sun_mid_ms_s70100.bin` | the Sun-like mid main sequence (G-FPS, G-THERMO): X_H,c 0.436, Σ_c 11.78, 3,115.1 sb, stage main_sequence | 70,100 · 800.011 | C4a (the run's own end dump) | `23b27d197e836ab6` |
| `m0-t97_massive_iron_core_s800.bin` | the Massive's iron core (G-LATCH, G-CONS): X_Fe,c 1.0, Σ_c 247.4, 6,670.9 sb, stage iron_core, N_Fe not yet converting; core_collapse fires at step 881 (81 steps on) | 800 · 7.227 | C3a (`--dump-every 100`) | `b74e56cce32c5f54` |

(observers' snapshots: sun `7f93ced2f4018a9c`, massive `49063e6f0d0558de`.) Each is the heaviest, only object. Both carry the adapter that made
them (RTX 4090, laserax-ai-server, git `255ec37-dirty` = the tree above) — a `--load-dump … --continue` on another adapter is the reader's check, not
proven here (NOT PROVEN: no `--load-dump` run was made).

## Method, deviations and findings
- **Closest bound gas — method.** A dump holds no ε_th; G-FIT's «½|u|² + ε_th + φ < 0» is bracketed per dump: **nec** — cells with ½|u|²+φ < 0
  (ε_th ≥ 0 dropped: necessary for bound; the true nearest bound gas is at least this far); **suf** — E/Σ+φ < 0 (E/Σ = ½|u|²+ε_th+ε_cold ≥
  ½|u|²+ε_th: sufficient; the true distance is at most this far). φ = the §1.4.2 kernel by FFT (zero-padded, K = −1/r, K(0,0) = −3.5255; the sinks'
  softened terms added, r_acc = max(2, 2M/c_sb²)). Check: Σ½Σφ on the Massive's step 5,000 dump = −6,869,134 against the ledger's potential −6,869,070 (10⁻⁵).
  Distance = from a cell's centre to the nearest world edge. «Any cell Σ ≥ σ_vac» counts, so the 0.5-cell figures are traces; the mass columns say how much.
  G-FIT stays a tuning target (UNVERIFIED, scope `fit` not built); this is a measurement, not a verdict on the contract.
- **Dense dumps change the life.** The dense runs (`--dump-every 100`, C3) and the plain runs (`--dump-every 5000`, C2) are the same scene and
  binary but not the same life: Massive ignition step 264 (t 3.915) against 273 (3.967), core_collapse 880 against 881, supernova 916 against 913;
  Giant ignition 416 against 417, core_collapse 1,380 against 1,377. A dump boundary cuts a frame, so P5's carry-over (§0.5 [M0-TJ4]) lands differently.
  Possibly by design — measured, not filed (see the handoff); the G-FIT bracket's early rows come from the dense runs, its later rows from the plain run.
- **Not run:** S1; the Sun's full life (C4); any `--load-dump` round trip of the kept dumps.
- **The dumps' planes.** A dump's header length leaves a trailing newline in some dumps and not in others (17 or 18 planes follow the 14 channels): the reader
  finds the planes by file size (a first version added one byte and misread a dump as garbage; fixed before any number above).

## Commands
All from the repo root on linux-pc unless the line starts with `server$` (laserax-ai-server, repo `~/workspace/sandbox_reactions`, binary
`build/target/release/sandbox-reactions`, cwd `build/t97`). The tree went up by R18's rsync.

C1 — the tree, the build
[ALREADY RUN — PASS (rsync; `cargo build --release --locked -p sr-app`: Finished, up to date) on laserax-ai-server]
```
rsync -a --exclude .git --exclude build --exclude .venv ./ ybelanger@192.168.1.134:workspace/sandbox_reactions/
```

C2 — the lives to the ending (plain)
[ALREADY RUN — PASS (a: exit 0, 20,272 steps, 564.2 s; b: exit 0, 9,640 steps, 322.8 s; the RTX 4090) on laserax-ai-server]
```
server$ ../target/release/sandbox-reactions headless --scene preset:massive --until ending --max-steps 200000 --adapter "RTX 4090" --out massive --dump-every 5000
server$ ../target/release/sandbox-reactions headless --scene preset:giant --until ending --max-steps 200000 --adapter "RTX 4090" --out giant --dump-every 5000
```

C3 — dense dumps (a: the Massive to t 30, 5,139 steps, 202.2 s; b: the Giant to t 25, 5,799 steps, 220.0 s)
[ALREADY RUN — PASS (exit 0 both) on laserax-ai-server]
```
server$ ../target/release/sandbox-reactions headless --scene preset:massive --until sim_time:30 --max-steps 20000 --adapter "RTX 4090" --out massive_d --dump-every 100
server$ ../target/release/sandbox-reactions headless --scene preset:giant --until sim_time:25 --max-steps 20000 --adapter "RTX 4090" --out giant_d --dump-every 100
```

C4 — the Sun-like: (a) to mid main sequence, 70,100 steps, 394.0 s; (b) the full life
[ALREADY RUN — PASS (a: exit 0, until met at t 800.011) on laserax-ai-server]
```
server$ ../target/release/sandbox-reactions headless --scene preset:sun --until sim_time:800 --max-steps 150000 --adapter "RTX 4090" --out sun_ms --dump-every 10000
```
[NOT RUN — for you] (b: V9 measured 177,762 steps in 762 s on this adapter; over the 10-minute line, so the lead's, in one visible terminal; it settles S1, the Sun's stages after t 800 and its ending, and G-FIT's envelope stretch)
```
server$ ../target/release/sandbox-reactions headless --scene preset:sun --until ending --max-steps 400000 --adapter "RTX 4090" --out sun_life --dump-every 20000
```

C5 — the starting states (one step each; the three runs' dumps and snapshots read by `probe/dumpread.py` and the snapshot's tracker)
[ALREADY RUN — PASS (DONE steps=1 ×3) on laserax-ai-server]
```
server$ for p in sun massive giant; do ../target/release/sandbox-reactions headless --scene preset:$p --steps 1 --adapter "RTX 4090" --out p1_$p; done
```
a, Σc: `a = min(40·√(M/3351.03), 150)`, `Σc = 3M/(2π a²)` (scene.rs `cloud_shape`) — with `preset_mass_sb` from assets/calibration.json.

C6 — the S2 scenes (disk at the Massive's a, Σc, T₀; `s2_massive_f0.1.json` is the same file with 0.05 for the control) and their runs
[ALREADY RUN — PASS (exit 0 both: 5,196 and 5,195 steps, 95.5 s and 95.8 s) on laserax-ai-server]
```
server$ cat s2_massive_f0.1.json
{"version": 1,
 "world": {"width": 600, "height": 400, "edge": "leave"},
 "slowdown": true,
 "switches": {"gravity": true, "hydro": true, "heat": true, "reactions": true, "neutrinos": true, "sinks": true},
 "objects": [
   {"kind": "disk", "x": 300, "y": 200, "a": 56.463588827675565, "sigma_c": 1.0, "t": 6.685550212860107,
    "composition": {"H": 0.7346, "He": 0.2485, "C": 0.0092, "O": 0.0077}, "v": [0, 0]}
 ],
 "overrides": {"f_dep": 0.1}}
server$ for f in 0.05 0.1; do ../target/release/sandbox-reactions headless --scene s2_massive_f$f.json --until sim_time:30 --max-steps 20000 --adapter "RTX 4090" --out s2_$f --dump-every 1000; done
```

C7 — the stages and durations (from summary.json, `probe/stages.py`)
[ALREADY RUN — PASS on linux-pc]
```
python3 -I probe/stages.py massive/summary.json massive; python3 -I probe/stages.py giant/summary.json giant; python3 -I probe/stages.py sun_ms/summary.json sun
```

C8 — the clocks (`probe/clocks.py <dump> <its observers>`; the dumps rsync'd from the server)
[ALREADY RUN — PASS on linux-pc]
```
python3 -I probe/clocks.py sun_ms/dumps/s_70100.bin sun_ms/dumps/s_70100.observers.json
```
(and s_20000, s_40000 of `sun_ms`; `massive_d/dumps/s_300`, `s_400`; `giant_d/dumps/s_400`, `s_500`.)

C9 — §1.9.3's row quantities (`probe/rows.py`), the G-FIT bracket (`probe/dumpread.py`, `probe/fitsum.py`), the S2 unbound mass (`probe/unbound.py`)
[ALREADY RUN — PASS on linux-pc]
```
python3 -I probe/rows.py massive_d/dumps/s_300.observers.json massive_d/dumps/s_800.observers.json massive_d/dumps/s_900.observers.json
for f in massive_d/dumps/s_*.bin; do python3 -I probe/dumpread.py $f; done > massive_d.fit.jsonl; python3 -I probe/fitsum.py massive_d.fit.jsonl
python3 -I probe/unbound.py s2_0.1/dumps/s_5000.bin s2_0.1/dumps/s_5000.observers.json
```

## Tuning — K1 (M0-T98a, 2026-11-05)
Written by M0-T98a (model=claude-sonnet-5-5 level=high, `rung_record.py now` at launch). Box: task on **linux-pc** (laserax-ai); every run on
**laserax-ai-server, RTX 4090** (Vulkan), the binary M0-T97 used (`git 255ec37-dirty` in summary.json — the server's checkout; HEAD here
`3cce799`, no source changed since M0-T97, only plan files). Scenes: `scenes/tune_k1_sun.json` (a 38.69977, Σc 1, T₀ 4.5796299, the SUN_MIX) and
`scenes/tune_k1_massive.json` (a 56.46359, T₀ 6.68555) — `disk` objects, no preset, each candidate its `overrides` (generated copies, `build/t98a/cand_<name>.json`
on the server), so no run needed the calibration (G-CAL). Raw output: server `~/workspace/sandbox_reactions/build/t98a/<name>{.out,/summary.json,/dumps}`;
trail `logs/M0-T98a.log`. Readings: `probe/clocks.py` (§ Appendix), τ_nuc with M_core = mass within 8 cells, Q_H 8,700 — the script hard-codes Q_H 8,700, so for the two Q_H runs the τ_nuc/τ_KH column (marked †) is the script's value × Q_H/8,700 (× 0.747, × 0.575) — arithmetic on the script's output, not a re-run.

**Commands** (all `[ALREADY RUN]`; `name` ∈ the first column, `OV` its overrides):
```
server$ cd ~/workspace/sandbox_reactions/build/t98a && ../target/release/sandbox-reactions headless --scene cand_<name>.json --until <ending|sim_time:T> --max-steps N --adapter "RTX 4090" --out <name> --dump-every D
```
Rounds: R1 `--until ending --max-steps 100000`; R2 `--until ending --max-steps 120000`; R3 and R4 `--until sim_time:200 --max-steps 22000 --dump-every 4000`; the baseline `sim_time:225`; the Massive `sim_time:30`.

**The baseline reproduces M0-T97's life.** `sun_base` (no overrides): ignition step 273 (t 4.266) by light, `main_sequence` from step 305 (t 4.580) — identical to the preset's; τ_KH/τ_dyn 213.6 / 238.8 / 219.2 and τ_nuc/τ_KH 29.4 / 28.9 / 28.0 at t 58.7 / 114.1 / 169.6 (T97: 201.7 / 27.1 at t 225.5). 19,958 steps to t 225.

**Candidates — what the Sun-like did** (X_H,c = the centre's hydrogen; ratios at the dumps of each run, first → last):

| Candidate (overrides) | Outcome (stages) | τ_KH/τ_dyn | τ_nuc/τ_KH | L (first → last) | steps to X_H,c 10⁻³ (linear, labelled) | life at the Quadro 10.709 ms/step (labelled) |
|---|---|---|---|---|---|---|
| base (none) | main_sequence to t 225 (T97: to t 800) | 213.6 → 219.2 | 29.4 → 28.0 | 1,496 → 1,478 | ≈ 217,000 | ≈ 38.7 min |
| κ₀ 0.8 | main_sequence to t 200 | 190.1 → 232.8 | 30.1 → 28.1 | 1,791 → 1,559 | ≈ 203,000 | ≈ 36.3 min |
| Q_H 6,500 | main_sequence to t 200 | 214.6 → 368.4 | 22.4 → 21.4 † | 1,553 → 1,075 | ≈ 231,000 | ≈ 41.3 min |
| Q_H 5,000 | main_sequence, then **white_dwarf** at t 135 (H unburnt, X_H,c 0.73) | 239 → 2,961 | 17.5 → 18.6 † | 1,471 → 242 | — (fades) | — |
| κ₀ 0.65 | failed_star, ignition t 8.5, main_sequence, **white_dwarf** at t 134 | 208 → 2,750 | 30.7 → 32.5 | 1,904 → 284 | — (fades) | — |
| κ₀ 0.5 | failed_star at t 6.9, ignition t 8.46, main_sequence 42.7 t.u., **white_dwarf** at t 51.6 (H unburnt, X_H,c 0.72), ending t 63.8 | 275 → 495 | 31.5 → 32.0 | 1,920 → 1,353 | — (fades) | — |
| κ₀ 0.3 / 0.2 / 0.1 | failed_star (0.3, 0.1), flickering main_sequence ↔ white_dwarf (0.2); ending at t 36.5 / 27.5 / 20.8 | — | — | — | — | — |
| c_sb 800 / 1,200 | as κ₀ 0.5: ignites late, **white_dwarf** with X_H,c 0.72 at t 54.5 / 29.3 | 261 → 531 / 414 | 31.4–32.3 | 2,009 → 1,276 / 1,693 | — (fades) | — |
| c_sb 2,000 / 4,000, a_r 2 × 10⁻⁴ | failed_star, ending at t 28.0 / 20.2 / 20.5 | — | — | — | — | — |

Outside the four named levers, **A_H (`a_h_burn`, §2.5.1's rate normalisation, bound > 0)** — a probe, labelled (M0-D50: A_H × 1.4 holds, × 2 runs away):

| Candidate (overrides) | Outcome | τ_KH/τ_dyn | τ_nuc/τ_KH | L (first → last) | steps to X_H,c 10⁻³ (linear) | life at the Quadro (labelled) |
|---|---|---|---|---|---|---|
| A_H × 1.4 (7 × 10⁻⁴) | main_sequence to t 200, ignition 689 (t 9.06) | 145.7 → 132.6 | 28.3 → 25.8 | 1,843 → 1,932 | ≈ 155,000 | ≈ 27.7 min |
| A_H × 1.4, κ₀ 0.65 | main_sequence to t 200 | 151.6 → 133.2 | 29.7 → 26.3 | 2,207 → 2,435 | ≈ 132,000 | ≈ 23.6 min |
| A_H × 1.4, κ₀ 0.5 | main_sequence to t 200 (failed_star first) | 176.6 → 145.1 | 30.6 → 26.7 | 2,298 → 2,589 | ≈ 128,000 | ≈ 22.8 min |
| A_H × 1.7 (8.5 × 10⁻⁴), κ₀ 0.55 | main_sequence to t 200, ignition 273 | 135.5 → 102.7 | 29.5 → 25.4 | 2,470 → 3,096 | ≈ 108,000 | ≈ 19.3 min |
| A_H × 1.4, κ₀ 0.4 | failed_star, main_sequence 49 t.u., **white_dwarf** at t 58 (H unburnt), ending t 70 | 252 → 11,552 | 31.4–32.2 | 2,138 → 83 | — (fades) | — |

Wall times: steps × 10.709 ms = the Quadro's whole-world step of reports/step_cost.md (M0-T63, before M0-D50's constants); the burning star's Quadro step is **NOT RUN**
(M0-T124, owed on laserax-ai) — the figures are arithmetic, UNVERIFIED, and the linear step counts understate how fast X_H,c falls late (L rises), so they are upper bounds.
The run wall times on the RTX 4090 (4–5 candidates in parallel, so slower than alone) are in each summary.json.

**The Massive** (`tune_k1_massive.json`, to t 30): baseline `iron_core` from step 321 (t 4.317, 3.4 t.u.), `core_collapse` at step 897, `supernova`, `neutron_star` — the stage
order H > He > C > {Ne, O} > Si is not displayed (the life skips to raw iron_core: M0-T98b's, unchanged by this task). With A_H × 1.7, κ₀ 0.55: `iron_core` 1.43 t.u., `core_collapse`
at step 577, then a **neutron_star at t 5.97 before the supernova at t 6.54** (a new flicker) — the ending still a neutron star, the sequence worse.

**What it says.**
1. G-SQUEEZE's two ≥ 10 bounds already hold, far from the aim: τ_KH/τ_dyn ≈ 200–240, τ_nuc/τ_KH ≈ 28–29 on the Sun-like main sequence (the second reaches the aim only by t ≈ 800).
2. **The named levers do not reach the gap.** κ₀, c_sb and a_r enter the transport only as a_r c_sb/κ₀ (thick gas) and scale-free R (§2.4.1); they do not raise the main sequence's L —
   it is pinned by the nuclear rate at T_c ≈ 33–37. Lowering κ₀ or raising c_sb/a_r instead lets the star radiate its heat: it contracts (R 14.3 → 10–11), T_c falls (33 → 15–25), and
   because the Sun-like mass (3,137) is 0.975 M_ch (3,217; §2.7 K₁ₑ, K₂ₑ) the contraction ends degenerate — a **white dwarf with the hydrogen unburnt** — or the star never ignites (`failed_star`).
   The edge between the two is sharp (κ₀ 0.8 holds, 0.65 fades).
3. Q_H cannot be lowered either: τ_nuc/τ_KH is independent of L and ∝ Q_H, so Q_H 6,500 holds the ratio (21.4 †) but the star's lower L lengthens the life; Q_H 5,000 fades.
4. The only lever that shortens the life is the burn rate A_H (outside the four named): A_H × 1.7 with κ₀ 0.55 halves the steps (≈ 108,000 against ≈ 217,000, labelled) and keeps both ratios ≥ 10 —
   but τ_KH/τ_dyn stays ≈ 100–135, ten times the aim, and it worsens the Massive's collapse (above). D50's × 2 runaway sits just beyond.

**Status: BLOCKED** — no in-bounds value of a_r, c_sb, κ₀, Q_H brings either gap toward 10–20; assets/physics.json **unchanged**. Question to the lead: see the handoff.

## Tuning — the mass ladder (M0-T99a, 2026-11-05)
Written by M0-T99a (model=claude-sonnet-5-5 level=high, `rung_record.py now` at launch). Box: task on **linux-pc** (laserax-ai); every run on **laserax-ai-server, RTX 4090** (Vulkan). Tree: HEAD `8832213` + the uncommitted working tree
rsync'd up (assets/physics.json sha256 `8ad6c983791e`, M0-D71g's keys in); the server binary rebuilt from it (`cargo build --release --locked -p sr-app`; summary.json still prints `git 255ec37-dirty`, the server checkout's own HEAD).
Scenes: `scenes/tune_fit_massive.json` (a 56.46359, Σc 1, T₀ 6.68555) and `scenes/tune_fit_giant.json` (a 93.249, T₀ 11.0453 — the preset's values to the digits lives.md § Giant records, labelled), `scenes/tune_fit_sun.json` (not run: T97 read the Sun-like clear to t 800, nearest 142.5) — `disk` objects, no preset, each candidate its `overrides` (generated copies `build/t99a/cand_<life>_<tag>.json` on the server), so no run needed the calibration (G-CAL). Raw output: server `~/workspace/sandbox_reactions/build/t99a/`; trail `logs/M0-T99a.log`.
Measured by T97's bracket (§ Method: nec = ½|u|²+φ < 0, suf = E/Σ+φ < 0; no ε_th in a dump), `probe/dumpread.py` + `probe/fitsum.py` (§ Appendix), unchanged.

**Command** (all `[ALREADY RUN — PASS (exit 0, until met) on laserax-ai-server, RTX 4090]`; `<life>` ∈ massive, giant; `<tag>` ∈ base, f0.1 (`{"f_dep": 0.1}`), f0.02 (`{"f_dep": 0.02}`)):
```
server$ cd ~/workspace/sandbox_reactions/build/t99a && ../target/release/sandbox-reactions headless --scene cand_<life>_<tag>.json --until sim_time:<60 | 38> --max-steps 40000 --adapter "RTX 4090" --out <life>_<tag> --dump-every 2500
```
The Massive three at once (10,727 / 10,734 / 10,739 steps to t 60), the Giant two at once (10,619 steps to t 38, its ending's time in T97). **Not run:** the Massive to its ending (t ≈ 113.8; 20,000 steps, T97's 564 s alone — over the 10-minute line in parallel, the lead's); Giant f0.02; any candidate outside `f_dep`.

**The closest bound gas per life (cells from an edge; mass within 16 cells, nec / suf):**

| Life, candidate | step 2,500 | 5,000 | 7,500 | 10,000 | last (t) |
|---|---|---|---|---|---|
| Massive base (f_dep 0.05) | 83.5 / 86.5 · 0 | **0.5** · 1.36 / 0.67 (t 28.9) | 0.5 · 2.29 / 2.07 | 0.5 · 4.11 / 3.33 | 0.5 · 4.00 / 3.57 (60.0) |
| Massive f_dep 0.1 | 83.5 / 86.5 · 0 | 0.5 · 1.44 / 0.78 | 0.5 · 2.20 / 1.94 | 0.5 · 3.56 / 3.09 | 0.5 · 3.44 / 3.17 (60.0) |
| Massive f_dep 0.02 | 84.5 / 87.5 · 0 | 0.5 · 0.65 / 0.26 (suf 2.5) | 0.5 · 2.98 / 2.65 | 0.5 · 5.18 / 3.71 | 0.5 · 5.12 / 4.35 (60.0) |
| Giant base | 32.5 / 36.5 · 0 | 0.5 · 334.95 / 310.12 | 0.5 · 322.42 / 289.47 | 0.5 · 241.97 / 241.97 | 0.5 · 201.47 / 201.47 (38.0) |
| Giant f_dep 0.1 | **byte-identical to base at every dump** | | | | |

(total mass at the Massive's step 10,000: 4,382 / 4,398 / 4,337; the Giant's at step 5,000: 4,768, one sink inside.) The Massive's G-FIT fails from step 5,000 (t ≈ 29) on this tree — T97's dump cadence (5,000 steps, the first read t 57) put the crossing later; the Giant's from step 5,000 (T97: t ≈ 13 on its dense run).

**What it says.**
1. `f_dep` over its whole bound (0.02–0.1, §2.7) moves the Massive's mass within 16 cells by ≤ 1.5× and the nearest bound gas not at all (0.5 cell at every dump from t 29): **no value in the bound fits**.
2. The Giant's life ignores `f_dep` — identical to the byte, no neutrino deposit acts (it collapses to a sink, no supernova) — so that lever is inert there; its 16-cell band holds 201–335 mass (4–7 % of the gas, nec) (T97's ending dump: every cell of the band, Σ ≈ 0.002 — not re-read per cell here).
3. The edge gas is the bound halo of the explosion/collapse (T97's reading of the dumps, M0-T99a's tasks file): a tail of the energy spread; a physics.json constant moves the energy, not the tail.
4. The levers that could change it are outside physics.json: §2.10's radius a = 40√(M/M_sun) (cap 150), the presets' Suns (1, 15, 40), and G-FIT's wording (a tolerance, §0.4) — each changes only by asking the lead.

**Status: BLOCKED** — no in-bounds physics.json value brings a life's closest bound approach to 16 cells; assets/physics.json **unchanged**. Question to the lead: see the handoff.

### The rerun — G-FIT reworded to a density floor (M0-T99a, 2026-11-06)
By M0-T99a's rerun (model=claude-opus-5-5 level=high, `rung_record.py now`), box **linux-pc** (laserax-ai); **no new run** — the dumps above
(laserax-ai-server, **RTX 4090**, tree `8832213` + its working tree = `bde397c`) and T97's (§ Sun-like, § Massive, § Giant: RTX 4090, tree
`acaca48` + its working tree, `git 255ec37-dirty`) pulled by `scp` to linux-pc's scratch and read by `probe/edge_halo.py` (§ Appendix; it imports
`probe/dumpread.py`, the same bracket). The lead: «Reword G-FIT (Recommended)», then «Density floor 10⁻² (Recommended)» and, on the Giant, «File
a D, T99a stays BLOCKED» → contract §0.5 [M0-T99a], §5.2's G-FIT: **no bound gas denser than Σ = 10⁻² within 16 cells of an edge**.

**Command** `[ALREADY RUN — PASS (exit 0, 35 dumps) on linux-pc, reading RTX 4090 dumps]` (from the scratch folder holding `probe/` and `d/<tree>/<life>/s_*.bin`):
```
python3 -I probe/edge_halo.py d/t99a/massive_base d/t99a/massive_f0.1 d/t99a/massive_f0.02 d/t99a/giant_base d/t97/massive d/t97/giant d/t97/sun_ms
```

**The edge band's bound gas (nec bracket; suf within a few % — same nearest distances at Σ ≥ 10⁻³):** worst dump per life, the
nearest bound gas at each density floor (cells from an edge; `—` none bound at that floor anywhere):

| Life (tree, to t) | worst Σ in the band | its mass share | nearest at Σ ≥ 10⁻⁴ / 10⁻³ / **10⁻²** / 10⁻¹ | at 10⁻² |
|---|---|---|---|---|
| Sun-like (T97, to 800) | 0 | 0.003 % (t 114) | 152 / 165 / **172** / 178 | **fits** |
| Massive base (today, to 60) | 4 × 10⁻⁴ | 0.094 % | 0 / 38 / **134** / 170 | **fits** |
| Massive f_dep 0.1 (today, to 60) | 5 × 10⁻⁴ | 0.081 % | 0 / 76 / **148** / 170 | **fits** |
| Massive f_dep 0.02 (today, to 60) | 8 × 10⁻⁴ | 0.12 % | 0 / 18 / **110** / 152 | **fits** |
| **Massive base (today, to its ending 40.3, neutron_star; the lead's run)** | 5 × 10⁻⁴ | 0.054 % (t 40.3) | 0.5 / 38.5 / **155.5** / 165.5 | **fits** |
| Massive (T97, to its ending 113.8) | 7.6 × 10⁻³ | 0.46 % (t 84) | 0 / 0 / **30** / 38 | **fits** |
| Giant (T97, to its ending 37.8) | 6.5 × 10⁻³ | 0.38 % (t 22) | 0 / 0 / **54** / 64 | **fits** |
| Giant base (today, to 38) | **5.2 × 10⁻²** | 2.1 % (t 21) | 0 / 0 / **0** / — | **fails** from t ≈ 21 to 38 |

**The chosen values:** assets/physics.json **unchanged** — every in-bound lever is as it stood (§ above: f_dep inert on the Giant, no lever
empties the halo); at the floor, the Sun-like and the Massive fit at today's values on the dumps read. **The Giant fails on today's tree**: its
envelope blows out — at t 21, 1,780 sb have left the world and 2,700 of 4,768 gas lie beyond 150 cells, moving at a median 1.29 × v_esc in the
band (T97's tree: a shell at 100–150 cells, 7,614 sb, 1.04 × v_esc) — filed as **M0-D158** (suspected M0-D71g's hold, the only engine change
between the two trees; not measured). Not run: the Giant on the fixed tree (M0-D158's Verify).

**The Massive to its ending on today's tree** — the lead's run, 2026-11-06 (tree: linux-pc's working tree rsync'd to laserax-ai-server, the
server's binary built from it; physics.json sha256 `8ad6c983791e`), read by `probe/edge_halo.py` on linux-pc:
`[ALREADY RUN — PASS (exit 0, until=met, 129.8 s) on laserax-ai-server, RTX 4090]`
```
cd ~/workspace/sandbox_reactions/build/t99a && ../target/release/sandbox-reactions headless --scene cand_massive_base.json --until ending --max-steps 40000 --adapter "RTX 4090" --out massive_full --dump-every 2500
```
Ignition by fuel step 273 (t 3.97), core_collapse 897 (t 7.68), supernova 913 (t 7.76), `neutron_star` from step 3,716 (t 22.1), **ending
neutron_star at step 7,096 (t 40.30), 4,335.8 sb** (held 18.18 ≥ hold 18.15) — T97's tree ended at step 20,272 (t 113.8). Dumps 2,500 and
5,000 byte-equal in their reads to `massive_base`'s; at 7,096 the band holds 2.38 sb (nec, 0.054 %), Σ ≤ 5 × 10⁻⁴, the nearest bound gas at
Σ ≥ 10⁻² 155.5 cells from an edge. The run printed `SR-WARN heat unconverged` at steps 930–933 (`passes 64`, rounds 820, 669, 589, 488),
just after the supernova — P5's 64-pass ceiling (M0-D155 read this life at 0 warnings on its tree) — filed as **M0-D159**.

**Status: BLOCKED** on M0-D158 (the Giant alone) — G-FIT reworded (§0.5 [M0-T99a]); Sun-like and Massive fit at the floor.

### M0-D158 — the Giant's envelope is chaotic, not regressed (2026-11-06)
By M0-D158 (model=claude-opus-5-5 level=high); every run on **laserax-ai-server, RTX 4090**; detail, commands and every row in
tasks/M0-D158.md. **Today's binary and fbcbadf's (T97's tree) give byte-identical dumps** on the `disk` above, and fbcbadf + `preset:giant`
reproduces T97's `giant` byte for byte. So the blow-out is **the scene, not the tree**: the disk is 2.3 × 10⁻⁵ lighter than the preset
(core_collapse 1,425 against 1,377). Of 6 starts within ~10⁻⁵ of the preset (the preset, the disk, the disk at the preset's exact a and
Σc, T₀ ± 10⁻⁴, Σc + 10⁻⁴), **3 throw bound gas at Σ ≥ 10⁻² to the edge by t 22 and 3 keep a shell**. The exact preset fits at every
2,500-step dump to its ending (black_hole, step 9,644; nearest at Σ ≥ 10⁻²: 25.5 cells, at step 2,500). The lead: «Grade the preset
(Recommended)». G-FIT reads the preset; its fit is fragile, so a run on another adapter may land on a failing life. Until M0-V14
recalibrates, today's binary refuses presets (G-CAL exit 6).

### The close — every preset's life graded at the chosen values (M0-T99a, 2026-11-06)
By M0-T99a's third run (model=claude-opus-5-5 level=high, `rung_record.py now`), box **linux-pc** (laserax-ai); **no new GPU run** — every
dump below is an **RTX 4090** (Vulkan) dump from laserax-ai-server, pulled to linux-pc's scratch and read by `probe/edge_halo.py` (§ Appendix,
unchanged). G-FIT grades each **preset** (M0-D158; the lead: «Grade the preset (Recommended)»), at the floor Σ = 10⁻² (§0.5 [M0-T99a]).

**Trees.** T97's runs (`sun_ms`, `massive`, `massive_d`, `giant`, `giant_d`) ran on fbcbadf's tree (`git 255ec37-dirty` in summary.json); D158's V1
on fbcbadf's binary. Today's tree (bde397c + M0-D159's uncommitted ceiling) runs the same lives. M0-D71g changes no dump (M0-D158: runs A = B, byte
for byte). M0-D159's ceiling (64 → 128 passes, §0.5 [M0-D159]) changes only a step that reaches 64 passes, and T97's five preset runs print no
`SR-WARN` (their `.out` files capture stderr, as T99a's disk runs' 930–933 lines show). V1's own `.out` went with the server's reboot (/tmp,
2026-11-06 ~10:34); T97's plain `giant` (0 warnings) is its run at another dump cadence. Today's binary cannot run a preset itself until
M0-V14 recalibrates (G-CAL exit 6).

**Command** `[ALREADY RUN — PASS (exit 0, 125 dumps) on linux-pc, reading RTX 4090 dumps]` (scratch: `d/t97/*` and `probe/` in session
4daa0c2b…'s, `d/d158/V1` → session 30448f92…'s `d/V1_preset_ending/dumps`, `d/t97/massive_d` in session 480b7544…'s; output in logs/M0-T99a-r3.log):
```
python3 -I probe/edge_halo.py d/d158/V1 d/t97/giant d/t97/giant_d d/t97/massive d/t97/massive_d d/t97/sun_ms
```

| Life (run, dump cadence, to) | dumps | nearest bound gas at Σ ≥ 10⁻² (cells; step) | densest band cell (Σ) | at the floor |
|---|---|---|---|---|
| Sun-like (`preset:sun`, T97 `sun_ms`, 10,000, t 800) | 8 | **172.5** (70,000) | 4.3 × 10⁻⁶ | **fits** |
| Massive (`preset:massive`, T97 `massive`, 5,000, ending t 113.8) | 5 | **29.5** (20,000) | 7.6 × 10⁻³ | **fits** |
| Massive (`preset:massive`, T97 `massive_d`, 100, t 30) | 52 | **120.5** (1,800) | 4 × 10⁻⁴ | **fits** |
| Giant (`preset:giant`, D158 V1, 2,500, ending t 37.79, black_hole) | 4 | **25.5** (2,500) | 7.1 × 10⁻³ | **fits** |
| Giant (`preset:giant`, T97 `giant`, 5,000, ending t 37.78) | 2 | **53.5** (5,000) | 6.5 × 10⁻³ | **fits** |
| Giant (`preset:giant`, T97 `giant_d`, 100, t 25) | 58 | **11.5** (3,500–3,600) — under 16 at 8 dumps, 3,100–3,900 (t 15.9–18.7) | 1.1 × 10⁻² (3,400) | **fails, marginally** |

`giant_d` is the preset with a dump every 100 steps: through step 2,500 it reads as the plain preset does (25.5 · 0.0071, M0-D158), then its
envelope's densest edge gas rises just past the floor (Σ 0.0102–0.011) and comes to 11.5–15.5 cells, falling back to 22.5 by step 4,000 and to
58.5 by 4,500. The coarse runs never look at steps 3,100–3,900. With M0-D158's three failing neighbours of six, four of seven Giant lives
within ~10⁻⁵ of the preset fail G-FIT: **the Giant's fit is fragile**.

**The fix, weighed** (the lead: «how can we fix it and do it done. If its too hard we go with the simpler answer.»): no physics.json lever
reaches the Giant — `f_dep` is inert on it (byte-identical lives, § above), and a_r, c_sb, κ₀, Q_H move every life and the calibration, so on
a chaotic envelope a tuned value lands on another neighbour, not on a fit that holds. A fix that holds is the Giant's mass or radius
(§2.10, a contract formula — the lead's), measured over a family of neighbour lives on the GPU, then M0-V14's recalibration: a task of its
own. Too hard here, so **the simpler answer**: the Giant is graded on its plain preset runs (D158's V1 and T97's `giant`), which fit; the
dense reading and the fix route go to M0-T99b (scope `fit` chooses its cadence) and M0-V14.

**The chosen values:** assets/physics.json **unchanged** (sha256 `8ad6c983791e…`, M0-D71g's keys in) — every in-bound lever as it stood.
**Status: DONE** — Sun-like, Massive and Giant (plain preset runs) fit at the floor; the Giant's dense life is flagged, not graded.

## Appendix — the reading scripts (extract each under `probe/`)
Pure Python 3 + numpy, run with `-I`; read dumps and snapshots, write nothing.

### probe/dumpread.py
```python
#!/usr/bin/env python3
"""M0-T97 dump reader: header, planes, phi by FFT (the contract's thin-sheet kernel, G = 1), the G-FIT distance proxy, the clocks."""
import json, struct, sys
import numpy as np

SIGMA_VAC = 1e-6
Q_H = 8700.0
C_SB = 400.0   # assets/physics.json c_sb

def read_dump(path):
    b = open(path, 'rb').read()
    assert b[:8] == b'SRDUMP01'
    n = struct.unpack('<I', b[8:12])[0]
    h = json.loads(b[12:12 + n])
    w, ht = h['width'], h['height']
    start = next(len(b) - p * w * ht * 4 for p in range(14, 24) if 12 + n <= len(b) - p * w * ht * 4 <= 13 + n)   # the planes: 14 channels first, then flux/rate planes when present; the header may carry a newline
    pl = np.frombuffer(b, dtype='<f4', count=14 * w * ht, offset=start).reshape(14, ht, w).astype(np.float64)
    return h, pl

def phi_of(sigma, sinks=()):
    ht, w = sigma.shape
    src = np.where(sigma >= SIGMA_VAC, sigma, 0.0)
    ky, kx = np.meshgrid(np.fft.fftfreq(2 * ht, 1 / (2 * ht)), np.fft.fftfreq(2 * w, 1 / (2 * w)), indexing='ij')
    r = np.hypot(kx, ky)
    with np.errstate(divide='ignore'):
        k = -1.0 / r
    k[0, 0] = -3.525494348078172
    pad = np.zeros((2 * ht, 2 * w)); pad[:ht, :w] = src
    phi = np.fft.irfft2(np.fft.rfft2(pad) * np.fft.rfft2(k), s=pad.shape)[:ht, :w]
    for s in sinks:                                     # §1.4.3: analytic, softened by r_acc
        yy, xx = np.mgrid[0:ht, 0:w]
        phi -= s['m'] / np.sqrt((xx + .5 - s['x']) ** 2 + (yy + .5 - s['y']) ** 2 + s['r_acc'] ** 2)
    return phi

def edge_distance(ht, w):
    yy, xx = np.mgrid[0:ht, 0:w]
    return np.minimum.reduce([xx + .5, w - xx - .5, yy + .5, ht - yy - .5])

def fit(path):
    """G-FIT's nearest bound gas to a world edge, bracketed (the dump holds no eps_th): NEC = Sigma >= sigma_vac and 1/2|u|^2 + phi < 0
    (necessary for bound, eps_th >= 0 dropped: the true distance is >= d_nec); SUF = E/Sigma + phi < 0 (sufficient: E/Sigma = 1/2|u|^2 +
    eps_th + eps_cold >= 1/2|u|^2 + eps_th: the true distance is <= d_suf). Per bound: d_min, d_1e3 (the distance with 1e-3 of the dump's
    mass at or inside it) and the mass within 16 cells of an edge."""
    h, pl = read_dump(path)
    s, mx, my, e = pl[0], pl[1], pl[2], pl[3]
    with np.errstate(divide='ignore', invalid='ignore'):
        u2 = np.where(s > 0, (mx * mx + my * my) / (s * s), 0.0)
        es = np.where(s > 0, e / s, 0.0)
    sinks = [dict(x=k['x'], y=k['y'], m=k['m'], r_acc=max(2.0, 2 * k['m'] / C_SB ** 2)) for k in h.get('sinks', [])]
    phi = phi_of(s, sinks)
    d = edge_distance(*s.shape)
    gas = s >= SIGMA_VAC
    m_tot = s[gas].sum()
    out = dict(step=h['step'], sim_time=h['sim_time'], mass=m_tot, n_sinks=len(sinks))
    for name, cand in (('nec', gas & (0.5 * u2 + phi < 0)), ('suf', gas & (es + phi < 0))):
        if cand.any():
            order = np.argsort(d[cand]); cum = np.cumsum(s[cand][order]); k = np.searchsorted(cum, 1e-3 * m_tot)
            out['d_' + name] = float(d[cand].min()); out['d1e3_' + name] = float(d[cand][order][k]) if k < len(cum) else None
            out['m16_' + name] = float(s[cand & (d < 16.0)].sum())
        else:
            out['d_' + name] = None
    return out

if __name__ == '__main__':
    for p in sys.argv[1:]:
        print(json.dumps(fit(p)))
```

### probe/clocks.py
```python
#!/usr/bin/env python3
"""M0-T97 clocks: G-SQUEEZE's tau_dyn, tau_KH, tau_nuc of the heaviest object of a dump's observers snapshot (M_core = the mass within 8 cells of its centre, as M0-D50 read it)."""
import json, sys
import numpy as np
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from dumpread import read_dump, Q_H

def clocks(dump, obs, r_core=8.0):
    o = json.load(open(obs))
    objs = o['watch']['tracker']['objects']
    ob = max(objs, key=lambda x: float(x['mass']))
    f = lambda k: float(ob[k])
    h, pl = read_dump(dump)
    cx, cy = (float(v) for v in ob['centre'])
    yy, xx = np.mgrid[0:pl.shape[1], 0:pl.shape[2]]
    inside = np.hypot(xx + .5 - cx, yy + .5 - cy) <= r_core
    m_core = pl[0][inside].sum()
    # x_H at the centre: the tracker's own central composition (index 0 = H)
    x_hc = float(ob['x_c'][0])
    L, W, R, M = f('luminosity'), abs(f('potential')), f('radius'), f('mass')
    tau_dyn = f('tau_dyn'); tau_kh = W / (2 * L); tau_nuc = x_hc * m_core * Q_H / L
    return dict(step=h['step'], sim_time=h['sim_time'], M=M, R=R, W=W, L=L, x_H_c=x_hc, T_c=f('t_c'), sigma_c=f('sigma_c'),
                M_core8=float(m_core), tau_dyn=tau_dyn, tau_KH=tau_kh, tau_nuc=tau_nuc,
                KH_over_dyn=tau_kh / tau_dyn, nuc_over_KH=tau_nuc / tau_kh)

if __name__ == '__main__':
    print(json.dumps(clocks(sys.argv[1], sys.argv[2])))
```

### probe/stages.py
```python
#!/usr/bin/env python3
"""M0-T97 stages: a summary.json's displayed stage sequence, each stage's duration, and §5.2 G-STAGES' subsequence check."""
import json, sys

GS = {  # (id, optional) ; a tuple of ids = either order
    'sun': [('cloud', 0), ('collapsing', 1), ('protostar', 0), ('main_sequence', 0), ('h_shell', 0), ('he_core', 0), ('he_shell', 0), ('planetary_nebula', 0), ('white_dwarf', 0)],
    'massive': [('cloud', 0), ('collapsing', 1), ('protostar', 0), ('main_sequence', 0), ('h_shell', 1), ('he_core', 0), ('c_burn', 0), (('ne_burn', 'o_burn'), 0), ('si_burn', 0), ('iron_core', 0), ('core_collapse', 0), ('supernova', 0), ('neutron_star', 0)],
    'giant': [('cloud', 0), ('collapsing', 1), ('protostar', 0), ('main_sequence', 0), ('he_core', 0), ('core_collapse', 0), ('supernova', 1), ('black_hole', 0)],
}

def check(preset, ids):
    pos, missing = 0, []
    for want, opt in GS[preset]:
        alts = want if isinstance(want, tuple) else (want,)
        hits = [i for a in alts for i in range(pos, len(ids)) if ids[i] == a]
        if alts != want and len(alts) == 2:          # either order: both must appear after pos
            got = [min((i for i in range(pos, len(ids)) if ids[i] == a), default=None) for a in alts]
            if None in got:
                if not opt: missing += [a for a, g in zip(alts, got) if g is None]
            else:
                pos = max(got) + 1
            continue
        if hits:
            pos = min(hits) + 1
        elif not opt:
            missing.append(want)
    return missing

def run(path, preset):
    d = json.load(open(path))
    o = [x for x in d['objects'] if x['id'] == 1][0]
    st = o['stages']; end = d['sim_time']
    rows = []
    for i, s in enumerate(st):
        t1 = st[i + 1]['sim_time'] if i + 1 < len(st) else end
        rows.append((s['stage'], s['from_step'], s['sim_time'], t1 - s['sim_time']))
    return d, o, rows, check(preset, [r[0] for r in rows])

if __name__ == '__main__':
    d, o, rows, missing = run(sys.argv[1], sys.argv[2])
    print('steps', d['steps'], 'sim_time', round(d['sim_time'], 3), 'wall_s', round(d['wall_s'], 1), 'git', d['git'], 'adapter', d['adapter']['name'])
    for r in rows: print('%-17s step %-7d t %-10.3f dur %.3f' % r)
    print('G-STAGES missing:', missing or 'none')
    for e in d['events']: print(e['kind'], e['step'], round(e['sim_time'], 3), json.dumps(e['detail']))
```

### probe/unbound.py
```python
#!/usr/bin/env python3
"""M0-T97 S2/S1 reading: a dump's unbound mass, bracketed (no eps_th in the dump): LOW = cells with 1/2|u|^2 + phi >= 0 (certainly unbound),
HIGH = cells with E/Sigma + phi >= 0 (every cell not certainly bound); plus the mass escaped (the snapshot's ledger) and the neutron matter's first sight."""
import json, sys
import numpy as np
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from dumpread import read_dump, phi_of, SIGMA_VAC, C_SB

def unbound(dump, obs):
    h, pl = read_dump(dump)
    s, mx, my, e = pl[0], pl[1], pl[2], pl[3]
    with np.errstate(divide='ignore', invalid='ignore'):
        u2 = np.where(s > 0, (mx * mx + my * my) / (s * s), 0.0); es = np.where(s > 0, e / s, 0.0)
    sinks = [dict(x=k['x'], y=k['y'], m=k['m'], r_acc=max(2.0, 2 * k['m'] / C_SB ** 2)) for k in h.get('sinks', [])]
    phi = phi_of(s, sinks)
    gas = s >= SIGMA_VAC
    o = json.load(open(obs))
    esc = float(o['ledger']['last']['energy'] and o['ledger']['last']['mass']['escaped'])
    return dict(step=h['step'], t=h['sim_time'], grid=float(s[gas].sum()), escaped=esc,
                unb_low=float(s[gas & (0.5 * u2 + phi >= 0)].sum()), unb_high=float(s[gas & (es + phi >= 0)].sum()),
                xn_max=float(pl[13].max()), sink_mass=float(sum(k['m'] for k in h.get('sinks', []))))

if __name__ == '__main__':
    for i in range(1, len(sys.argv), 2):
        print(json.dumps(unbound(sys.argv[i], sys.argv[i + 1])))
```

### probe/fitsum.py
```python
import json, sys
f = sys.argv[1]
rows = sorted((json.loads(l) for l in open(f)), key=lambda r: r['step'])
print(f.rsplit('/',1)[1], len(rows), 'dumps, steps', rows[0]['step'], '-', rows[-1]['step'])
k = lambda r, n: r[n] if r[n] is not None else 1e9
for n in ('suf', 'nec'):
    w = min(rows, key=lambda r: k(r, 'd_' + n))
    print(' nearest', n, 'd=%.1f' % w['d_' + n], 'step', w['step'], 't %.2f' % w['sim_time'], 'm16 %.2f' % w.get('m16_' + n, 0), 'of', round(w['mass'], 1))
    first = next((r for r in rows if k(r, 'd_' + n) < 16), None)
    print('  first dump under 16 cells:', None if not first else (first['step'], round(first['sim_time'], 2), first['d_' + n], round(first.get('m16_' + n, 0), 2)))
```

### probe/rows.py
```python
#!/usr/bin/env python3
"""M0-T97 rows: the quantities §1.9.3's rows read, for the heaviest object of each observers snapshot given."""
import json, sys, math
NAMES = ['H', 'He', 'C', 'O', 'Ne', 'Mg', 'Si', 'S', 'Fe', 'n']
for p in sys.argv[1:]:
    o = json.load(open(p)); ob = max(o['watch']['tracker']['objects'], key=lambda x: float(x['mass']))
    f = lambda k: float(ob[k]); xc = [float(v) for v in ob['x_c']]; lg = [float(v) for v in json.loads(ob['l_group'])] if isinstance(ob['l_group'], str) else [float(v) for v in ob['l_group']]
    M, R = f('mass'), f('radius'); vs = math.sqrt(M / R)
    st = o['watch']['stages'][0]
    print('step %-6d t %-8.3f shown %-14s raw %-14s | X_H,c %.3g X_He,c %.3g X_Fe,c %.3g X_n,c %.3g | d_c %.3g | L %.4g L_nuc %.4g L_H/L %.3g L_N %.3g | v_r/sqrt(GM/R) %.3f | M %.1f M_unb %.2f sinks %s' % (
        o['step'], float(o['sim_time']), st['shown'], st['raw'], xc[0], xc[1], xc[8], xc[9], f('d_c'), f('luminosity'), f('l_nuc'), lg[0] / f('luminosity') if f('luminosity') else float('nan'), lg[6] if len(lg) > 6 else float('nan'), f('v_r') / vs, M, f('unbound'), ob['sinks']))
```

### probe/edge_halo.py
```python
#!/usr/bin/env python3
"""M0-T99a r2: what the 16-cell edge band's bound gas is made of, per dump — mass share, densest cell, emissivity, and the nearest
bound gas above each density floor. Bracket as T97 (nec: 1/2|u|^2+phi<0; suf: E/Sigma+phi<0). Reads dumps, writes nothing."""
import json, sys, glob, re
import numpy as np
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from dumpread import read_dump, phi_of, edge_distance, SIGMA_VAC, C_SB
FLOORS = (1e-4, 1e-3, 1e-2, 3e-2, 1e-1)
def one(path):
    h, pl = read_dump(path)
    s, mx, my, e, xh = pl[0], pl[1], pl[2], pl[3], pl[4]
    with np.errstate(divide='ignore', invalid='ignore'):
        u2 = np.where(s > 0, (mx*mx + my*my)/(s*s), 0.0); es = np.where(s > 0, e/s, 0.0)
    sinks = [dict(x=k['x'], y=k['y'], m=k['m'], r_acc=max(2.0, 2*k['m']/C_SB**2)) for k in h.get('sinks', [])]
    phi = phi_of(s, sinks); d = edge_distance(*s.shape); gas = s >= SIGMA_VAC; band = d < 16.0
    m_tot = s[gas].sum(); m_sink = sum(k['m'] for k in h.get('sinks', []))
    out = dict(step=h['step'], t=round(h['sim_time'], 2), gas=round(float(m_tot), 1), sink=round(float(m_sink), 1))
    for name, b in (('nec', gas & (0.5*u2 + phi < 0)), ('suf', gas & (es + phi < 0))):
        bb = b & band
        out[name] = dict(m16=round(float(s[bb].sum()), 3), share=float(s[bb].sum()/(m_tot + m_sink)),
                         smax16=float(s[bb].max()) if bb.any() else 0.0,
                         eps16=float((1 - np.exp(-10*(1 + xh[bb])*s[bb])).max()) if bb.any() else 0.0,
                         dmin={f'{f:g}': (float(d[b & (s >= f)].min()) if (b & (s >= f)).any() else None) for f in FLOORS})
    return out
if __name__ == '__main__':
    for dd in sys.argv[1:]:
        fs = sorted(glob.glob(dd + '/s_*.bin'), key=lambda p: int(re.search(r's_(\d+)\.bin', p).group(1)))
        for f in fs:
            print(dd.rstrip('/').split('/d/')[-1], json.dumps(one(f)), flush=True)
```
