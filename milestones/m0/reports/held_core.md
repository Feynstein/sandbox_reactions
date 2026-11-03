# M0-D71f — The held core on the GPU lives

Box: laserax-ai-server, NVIDIA GeForce RTX 4090 (Vulkan, driver 595.99.02), idle at each run's start (R18). Date 2026-11-03. Tree:
HEAD c9a4fc2 plus the working tree's uncommitted changes (M0-D71b–M0-D71e, M0-D148–M0-D151: the hold on the GPU), rsync'd to
`~/workspace/sandbox_reactions` and built there (`build/target/release/sandbox-reactions`). Scratch on the server: `/tmp/d71f/`
(runs and stdlib readers `window.py`, `heldset.py`; their sources in this session's scratchpad on linux-pc). Raw lines:
logs/M0-D71f.log.

## The window
`build/target/release/sandbox-reactions headless --scene preset:sun --load-dump build/v9-sun-life/dumps/s_450000.bin --steps 10000
--dump-every 5000 --adapter "RTX 4090" --out /tmp/d71f/window` — the whole 600 × 400 world, every pass, from the Sun-like life's step
450,000 (M0-D71's Repro at 10,000 steps); **46.0 s wall**. Read from summary.json's `ledger` {start, end} and the step-455,000
observers' snapshot's `ledger.last`:

| span | floor_added | radiated | share | nuclear |
|---|---|---|---|---|
| steps 450,001–455,000 | 1.719 × 10³ | 1.239 × 10⁵ | **1.39 %** | 1.74 × 10⁴ |
| steps 455,000–460,000 | 4.311 × 10³ | 8.348 × 10³ | **51.6 %** | 0 |
| steps 450,001–460,000 | 6.030 × 10³ | 1.323 × 10⁵ | **4.56 %** (G-REST's bound 15 %) | 1.74 × 10⁴ |

- Against M0-R11's twin crop under H (η 0.1): 1.2 %, 46 %, 3.8 % — the same shape on the GPU's whole world; today's tree before the
  hold read 71.2 % (twin) and 47.5 % over the first half (GPU, M0-R11's baseline). **G-REST's window: met.**
- The second half is the twin's 46 %: the light left is the cooling remnant's own (8.3 × 10³ in 5,000 steps), and the floor's
  4.3 × 10³ is of the order M0-R11 placed in the free envelope beside the core (HLL's residual non-balance, ≈ 0.86 a step). Over
  the whole window the floor pays 1 part in 22 of the light.
- Held: 37 cells at steps 455,000 and 460,000 (box (298, 193)–(304, 199), 1,321.9 of the remnant's 1,495.3; Σ_max 61.29) — the twin's
  37 from step 3,500 (M0-R11).
- Energy invariant: −1.715729 × 10⁵ → −1.730291 × 10⁵ (Δ −1.456 × 10³, all in the first half: −1.730294 × 10⁵ at 455,000). The
  remnant's τ_dyn = √(R³/GM) is 0.345 at the start (R 5.62, M 1,495.3) and 0.145 from step 455,000 (R 3.16); over t 92.66 that is
  ≥ 269 τ_dyn, so the drift is ≤ 5.4 per τ_dyn = **1.6 × 10⁻⁵ |W₀| per τ_dyn** (|W₀| 3.42 × 10⁵), against G-CONS's 10⁻³ (graded
  per τ_dyn as SM-O10 reads it; over the whole window 4.3 × 10⁻³ |W₀|, the twin's H 4.0 × 10⁻⁴ of the invariant).
- Momentum at the end: grid (1.37, 15.95), held (−102.0, −21.6) — item 3's forces on a remnant that cannot move (the twin's crop
  (78.6, −226)), not graded here.
- The observer reads the remnant `failed_star` (a dump without its observers' snapshot starts with no ignition on record; the
  window's object ends on that stage at step 455,256) — the stage is not this window's subject.

## The lives
`build/target/release/sandbox-reactions calibrate --only m_bh_sb --bracket 4572.394,67020.643 --adapter "RTX 4090" --out
/tmp/d71f/lives/cal.json --checkpoint-every 50 --progress-every 100` under `timeout -s INT 570` (a life over 10 minutes is the
lead's), every checkpoint pair hard-linked aside as written (`/tmp/d71f/lives.sh`, kept in `/tmp/d71f/lives/keep/`); record
build/calibrate_runs/2026-11-03T20-01-35Z/ on the server; exit 124 at 570 s. A first cut (`--max-steps 1000`, 11.0 s, record
2026-11-03T20-00-36Z) put life 01's events at ignition 257, core_collapse 833 (t 7.47), supernova 863.

- **Life 01 (4,572.394): ends neutron_star** — `side=Light ignited=true core_collapse=true ending=neutron_star steps=4239 sim_time=26.965
  wall_s=87.8` (the last calibration's record, 2026-10-26T18-37-19Z, ended it neutron_star at step 37,570, 383 s). No sink. The held
  set (`heldset.py` over the checkpoints): none to step 801; 6 cells (1,301.0, X_Fe 0.944, Y_e 0.439) at step 863, the core
  collapsing; 13 by 1,007, 19 (3,617.1, X_Fe 0.838, X_n 0.162, Y_e 0.389, one 4-connected piece at (298–302, 198–202)) from step ≈ 3,300
  to the end. Σ_max 607.8 at (300, 199) from step 911 on, bit for bit: its Y_e,c 0.2632 puts Y_eΣ at 160.0 = Σ_N — N_Fe neutronized
  it to the gate, the hold took it there, and a held cell never compresses past it. Releases between the 78 checkpoints kept (≈ 1 in
  3 of those written): 13, none at N_Fe's gate, none thin — warmth by elimination (ε_th > 2η ε_cold). **The ending kept, on a remnant that is an iron core frozen at the gate** (3,617 >
  m_ch_sb 3,216.4).
- **Life 02 (67,020.643 = 20 M₀): no sink — the hold's failure** (the block's Adversarial). Cut at step 8,807 (t 48.14, 0.12 s a step,
  displayed neutron_star, 3 objects, escaped 12,294, sinks 0; Σ_c 461.6, X_Fe,c 0.7465, v_r −0.035). The iron ring's clumps are held
  as they form: 16 cells (4,147.7, X_Fe 0.998) at step 1,655, 85 (19,926) at 1,911, 277 (42,818.1, X_Fe 0.987, Y_e 0.458) at 3,159 —
  a dozen pieces of 2,979–3,731 (18–21 cells each, X_Fe 0.97–1.00). The densest cell (Σ 461.6 at (342, 157), Y_eΣ 213 > Σ_N: gated,
  free) is walled in by held cells and frozen from step 1,655. No piece reaches m_tov_sb 4,853.2: held cells neither compress nor move,
  so the clumps can neither collapse nor merge. Releases between the 169 checkpoints kept to step 8,855: 836,
  none at the gate, none thin (warmth by elimination); 458 cells (51,426.0) held at step 8,855. Its ending (≈ 20 min, the
  last calibration's 1,177 s to step 14,099) is **NOT RUN** — the lead's in one visible terminal, and moot under this form.
- **The hold's part** (the ending moved, so the life ran again on the tree before M0-D71's build): f98f96d by `git archive` into a
  scratch on linux-pc, rsync'd to `/tmp/d71f/pre` and built there (`CARGO_TARGET_DIR=/tmp/d71f/pre/build/target`, 34 s); both binaries
  from life 02's checkpoint at step 1,362 (none held yet), `headless --scene preset:giant --load-dump
  /tmp/d71f/lives/keep/m_bh_sb-02/s_1362.bin --steps 1700 --dump-every 100 --adapter "RTX 4090"` (`/tmp/d71f/replay.sh`):

| tree | core_collapse | sink | held at 3,062 | floor_added | nuclear | invariant | wall |
|---|---|---|---|---|---|---|---|
| today's (the hold) | step 1,700 | **none** | 254 | +4.04 × 10⁶ | −5.62 × 10⁶ | −0.01 × 10⁶ | 31.1 s |
| f98f96d (no hold) | step 1,700 | **step 2,600** (t 14.41, 5,165.7 = 1.06 m_tov_sb at (243.8, 142.9)); a second by 3,000 (5,668.8) | 0 | +168.00 × 10⁶ | −150.33 × 10⁶ | +17.35 × 10⁶ | 60.1 s |

  The sink forms from a free ring clump, not from fallback onto a held core, and the object reads black_hole (62,203.5) by step 3,062.
  The hold is the whole difference: same state, same physics, the sink at 2,600 without it and none with it.
- Item 2 (accretion past m_tov_sb): under the gate form no core reaches m_tov_sb, held or free, so no life measured needs inflow; before
  the hold the 20 M₀ life's sink needs none (above). At M0-D71 inflow took the M 900 disk's floor to 392.6 (bound 10).
- Step cost (M0-D151's flag): the replay's 1,700 steps take 31.1 s with the hold against 60.1 s without — the held run has no violent
  collapse (no P5 ceiling steps), so P2's ≈ 2× while held is not separable here; life 01 at 0.021 s a step over 4,239 steps.

## The crop
`held_core::gpu_the_remnant_window_shines_on_its_own_heat` (crates/sr-engine/tests/gpu/held_core.rs): scenes/remnant_window.bin —
`cut_window` of build/v9-sun-life/dumps/s_450000.bin, the box (248, 144, 104, 104) at (12, 12) of a 128 × 128 world, all 16 planes
(the 14 channels and F; the source has no rate and no mask), the source's corner cell (vacuum) outside, the box record's x, y moved to
(12, 12) so the window's box, FFT size and records are the life's; written by `SR_WRITE_SCENES=1` on the server (1,048,971 bytes,
sha256 3c5e3102…801107), checked against the cut wherever the source is on disk; then `headless --scene preset:sun --load-dump
scenes/remnant_window.bin --edge leave --steps 10000 --dump-every 5000` in-process.

- RTX 4090: floor_added 5.057 × 10³ of radiated 1.3125 × 10⁵ = **3.85 %** (bound 15 %); halves **1.24 %** (1.527 × 10³ / 1.236 × 10⁵) and
  **45.95 %** (3.529 × 10³ / 7.680 × 10⁴); 37 held at step 460,000; the invariant −1.615 × 10³ of |W| 3.42 × 10⁵. M0-R11's twin crop
  under H: 1.2 %, 46 %, 3.8 %. 34 s for the case (41 s with the build).
- Red by hand: `held-wall-gpu-pressure-only` applied in a scratch copy (`/tmp/d71f/arm`, its own target) — the case fails at **51.20 %**
  (24.51 % and 94.92 %, 24 held). The full `--redarm held_core` (5 runs of 21 cases, ≈ 750 s from M0-D151's 577 s + 5 × 34 s) is over
  10 minutes: run by the lead (Verify 3) — GO, 4 plants red, 0 green (773.91 s); held-wall-gpu-pressure-only red in 6 cases,
  the crop's among them at 51.20 % (logs/M0-D71f.held_core.redarm.log). The lead's claim (`--changed --task M0-D71f-claim`): 0
  scopes chosen, 83 not impacted — NO-GO by the zero-case rule only, every scope's last GO on this tree.
- Verify 1: `SR_TEST_ADAPTER="RTX 4090" python3 tools/pb/verify.py held_core cold_core cpu_gpu ledger jeans --task M0-D71f` — GO, 46
  passed, 5/5 scopes, held_core 21 (145.4 s).

## The flags
- **M0-D79 (|v_r| ≤ 0.05 √(GM/R) on the held neutron star).** Life 01 shows neutron_star from step 2,607 to its ending at 4,239; 6 of
  its 17 progress lines read |v_r|/√(GM/R) over 0.05 (0.1295–0.1555 at steps 2,607–3,007, 0.0505 at 3,103, while the envelope leaves:
  R 120 → 25.5), every line from 3,215 on ≤ 0.0175, and ≈ 0.005 once R is 2.0 (step 3,615 on). It holds — trivially: the centre is a
  held cell at u = 0, and it is the iron core item 1 replaces. **Carried to M0-D152** (the lead: «Carry the flag on (Recommended)»).
- **M0-R6 (the floor paying N_Fe at the 20 M₀ collapse).** Same 1,700 steps from s_1362 (the table above): before the hold floor
  +168.0 × 10⁶ against nuclear −150.3 × 10⁶ — M0-R6's M3 (+103.0 against −93.7 over 1,600 → 2,100) on today's engine — and the
  invariant +17.35 × 10⁶; with the hold +4.04 against −5.62 and −0.01: the held iron never neutronizes, so N_Fe draws nothing for the
  floor to refill. Under the ruled form (no iron held) the pump returns. **Carried to M0-D152** with these totals.
- **M0-D68 (the split against the late remnant's floor).** The whole window's first 2,500 steps (`headless … --steps 2500`, 8.1 s):
  floor_added **−0.32** against radiated 1.165 × 10⁵, where M0-D68's twin crop read 6.1 × 10³ (its split) and 1.09 × 10⁴ (the old one)
  over the same span. No work crosses a held wall, so at a held remnant the split no longer sets the floor; the window's later floor
  (4.3 × 10³ over steps 455,000–460,000) is the free envelope's. **Read; no D.**
- **M0-D71 (item 2, one-way inflow).** Ruled below: walls only, final.
- **M0-D71d (G-CONS's momentum on the lives).** Not read here (the lives' momentum invariant is M0-T106's); **carried to M0-D152**.
- **M0-D151 (P2 ≈ 2× while held).** Read above (§ The lives, step cost): not separable on these runs.
- **M0-D71e (a resume from a pre-M0-D71e dump starts with none held).** The window, the crop and the replays start from such dumps
  (s_450000, s_1362: none held at their step); P8 writes the mask at the end of their first step, so their cells are held from step
  1 — read, nothing to file.

## Ruling
Put to the lead with the measurements above (PLAYBOOK §12, one question call, three questions, 2026-11-03); the answers verbatim:
- Item 1 (collapse) — options «Hold no iron (Recommended)» · «Release at the ceiling»: **«Hold no iron (Recommended)»**. Contract §0.5
  [M0-D71f], §2.7 amended in place (a cell with X_Fe > 0 is never held, a held cell released once X_Fe > 0); the build is **M0-D152**,
  filed after this block.
- Item 2 (accretion) — options «Walls only, final (Recommended)» · «Inflow, designed later»: **«Walls only, final (Recommended)»**.
  §2.3.4's one-way inflow and §1.4.2's inflow face retired, tagged [M0-D71f]; §0.5 [M0-D71]'s walls are the final form.
- M0-D79's flag — options «Carry the flag on (Recommended)» · «File its return now»: **«Carry the flag on (Recommended)»** → M0-D152.
- G-REST struck from §5.5's register: its GPU disks (M0-D71c) and window and crop (here) are measured. The Superseded mirror is in
  m0_superseded.md.
