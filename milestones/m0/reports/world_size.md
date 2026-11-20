# World size — does a bigger default world pay? (M0-R16)

M0-R16 · BUILD (research) · **linux-pc** (laserax-ai) · 2026-11-20 · model=claude-fable-5-2 level=high (`rung_record.py now`, the heading's
rung). Raw output: `logs/M0-R16.log`, `logs/M0-R16.giant_800x800.log`, `logs/M0-R16.massive_800x800.log`. Run folder: `build/m0-r16/`
(q600, q800 — the two step-cost lines, 59 MB). Tags: **[M]** measured (box and run named), **[A]** arithmetic (mine, from measured numbers
or the contract's constants), **[NP]** not proven (not run on the size named, or one life where the life is chaotic).

**What every run below ran:** the shipped tree (physics_hash 8e2de6e4…, calibration of 2026-11-13: Sun-like 3,136.7 sb, Massive 9,810.4,
Giant 18,578.6; m_up_sb 6,515.5, m_bh_sb 13,681.1), leave edges. The two lives are fit.rs's own reading (the same bound test, floor and
band as scope `fit`) from a scratch copy whose only change is three lines passing `--world` (§ Appendix); nothing in the tree changed.

## In one paragraph
Of the 20 places the plan and the reports blame the edge, the world's size or a want of room, **a bigger world removes 2** (the Giant's
G-FIT red — the one open defect, M0-D202 — and the Massive's 1.5-cell margin), **removes part of 3** (the Giant's chaotic neighbours, the
painted star's swelling, the room to drop a preset), **cannot remove 12** (every conservation drift is a booking or a rounding, measured
and already fixed; the mass heavy stars lose is their own wind, the same in both worlds) and has **nothing to remove in 3**. The one size
measured, **800 × 800**, holds the Giant: its bound gas at Σ ≥ 10⁻² reaches 285.5 cells from the centre and never comes nearer than
**114.5 cells** to an edge (600 × 400: on the edge for 76 of 167 checkpoints); the Massive's envelope reaches **182.5 cells in both
worlds**, so its margin goes from 1.5 cells to 201.5 — all **[M]** linux-pc RTX 5090, § 2.2. It costs **2.3 × the step** whenever the gas
fills the world (**[M]** linux-pc Quadro RTX 4000 and RTX 5090, § 2.2; heavy lives: almost always), **1-pixel cells** in the default
window where today's are 2 (**[A]**, § 2.1), one recalibration and a redo list. **Recommended: 800 × 800** — it
closes M0-D202 without touching the preset or the gate — said with its limit: it fixes one open red and one thin margin, not «many issues».
**Ruled by the lead, 2026-11-20: «800 × 800 (Recommended)»** (§ 4).

## (1) The census
One row per defect or finding that names the edge, the world's size or a refusal for room (the block's list, plus what a grep of the
plan's headings and reports/ added: M0-D138, M0-D156, M0-D191, G-MULTI). **Cause** is the block's own measurement, never its title; the
box is the one that block measured on. **Verdict**: *removes* · *part* · *cannot* (the star's own, or a booking) · *nothing* (no defect).
Reach = cells from the star's centre; hh = the world's smaller half-extent (200 today); G-FIT needs hh ≥ reach + 16.

| # | Where (status) | Symptom, one line | Cause as measured | Size that would remove it | Verdict |
|---|---|---|---|---|---|
| 1 | M0-D202 (TODO); M0-T99b BLOCKED on it | Giant: bound gas at Σ ≥ 10⁻² on the edge, 76 of 167 checkpoints, t 20.5–77.6, up to Σ 0.033 — **[M]** linux-pc RTX 5090, M0-T99b | The bound envelope reaches **285.5 cells** (t 77.1) — **[M]** linux-pc RTX 5090, this block's Giant life; the world's hh is 200 | hh ≥ 301.5 → both axes ≥ 608 **[A]**; at 800 × 800 nearest 114.5 cells, 0 of 189 checkpoints red **[M]** | **removes** |
| 2 | M0-T99b's finding (tasks/M0-T99b.md) | Massive: nearest bound gas at Σ ≥ 10⁻² 17.5 cells from an edge (step 13,809) — a 1.5-cell margin — **[M]** linux-pc RTX 5090 | Its envelope reaches **182.5 cells at t 131.2 in both worlds** — **[M]** M0-T99b and this block's Massive life: the reach is the star's | any hh ≥ 198.5 holds it; at 800 × 800 nearest 217.5 cells **[M]** | **removes** (the margin, 1.5 → 201.5) |
| 3 | M0-D158 (DONE), lives.md § M0-D158 | 3 of 6 starts within 10⁻⁵ of the Giant preset threw bound gas at Σ ≥ 10⁻² to the edge by t 22 — **[M]** laserax-ai-server RTX 4090, the pre-2026-11-13 Giant | A chaotic envelope: neighbour lives differ | a margin the neighbours stay inside: 98.5 cells past the band at 800 × 800 **[A]** from row 1; no neighbour run on today's Giant | **part** — **[NP]** one life measured |
| 4 | M0-D166 (DONE), heavy_end.md § 2.1 | Heavy stars lose their mass: Massive 55 % in a leave world, the old Giant (43,461 sb) R₉₀ 827 in 800 × 800 — **[M]** laserax-ai-server RTX 4090 | A main-sequence wind and an eruption, the same in 600 × 800. Here: Massive 9,810 → 5,125 sb at t 188 in 800 × 800 against 5,115 at t 170 in 600 × 400; Giant 18,579 → 10,509 at t 98 against 9,374 at t 92.5 — **[M]** linux-pc RTX 5090: the loss arrives ~19 t.u. later, not smaller | none | **cannot** — the star's |
| 5 | M0-D180 (DONE; ruled bounce) | Hydrogen painted onto the Sun-like to 1.26 × m_up_sb: envelope R 200–212, 5,500 sb leave, ends a white dwarf — **[M]** linux-pc RTX 5090 | Burst paint: the unbinding is the burn at the overshoot (reactions off: 0 sb unbound). Paced paint: no burst, yet the star swells to R ≥ 200 and loses 5,132 sb through the edges; in bounce R 270–290 (the box) and it goes supernova | hh > 228 at least **[A]** (the edge cut the envelope at 212; its true reach is not measured) | **part** — **[NP]** not run on 800 × 800 |
| 6 | M0-D191 (DONE; ruled «erase deeper») | The erased Massive swells to R 98, sheds 10 %, later ≈ 2,300 sb leave — **[M]** linux-pc RTX 5090 | R 98 is inside today's world; what leaves is unbound, after the floor's energy | none | **cannot** |
| 7 | M0-D181 (DONE) | Supernova life, leave: momentum invariant moves 1.04 × Σm‖u‖ — **[M]** linux-pc RTX 5090 | Gravity's momentum source and the radiation force (M0-D183, M0-D184); the `escaped` term is (−43, −59) of 2.3 × 10⁴ | none | **cannot** — a booking |
| 8 | M0-D182 (DONE) | Same life: energy drifts 6.3 × 10⁻² \|W\| per τ_dyn — **[M]** linux-pc RTX 5090 | Mass leaving took its potential energy unbooked (Δ = δm·\|φ_e\|, \|φ_e\| ≈ M/200) | none: the outflow leaves any world (row 4); a bigger one halves \|φ_e\| **[A]**, the term stays | **cannot** — a booking, booked since |
| 9 | M0-D192, M0-D193 (DONE) | Same life: 13, then 5 windows still over — **[M]** linux-pc RTX 5090 | The face work's remainder; the ledger's W read half a step late — the interior's | none | **cannot** |
| 10 | M0-D194 (DONE) | Sun-like, bounce: momentum drifts 9.0 × 10⁻³ of the peak — **[M]** linux-pc RTX 5090 | Gravity's kick net (−0.0371 of the grid's −0.0348): the f32 FFT's floor; the walls' impulse is +0.0032, and «bounce and leave drift alike» | none | **cannot** — the interior's |
| 11 | M0-D196 (DONE) | `edges`' bounce cases assert «bounce books nothing» | A check's assertion against M0-D194's ruling (4 cases red at one `assert!`, **[M]** linux-pc RTX 5090) | none | **cannot** |
| 12 | M0-D199 (DONE) | Sun-like, bounce: still 3.8 × 10⁻³ — **[M]** linux-pc RTX 5090 | The vacuum reset's momentum, unbooked, once the box shrinks off the walls (step ~21,000) | none | **cannot** — a booking |
| 13 | M0-D195 (DONE) | Sun-like, bounce: mass drifts 1.3 × 10⁻⁶ — **[M]** linux-pc RTX 5090 | Each cell's f32 rounding of Σ while the red giant's envelope fills the world | none — more cells round as much **[A]** | **cannot** |
| 14 | M0-D69 (DONE) | A cold disk 12 cells from a leave edge drifts 266 in momentum, 21 «in a wider world» — **[M]** laserax-ai-server RTX 4090 | A one-sided difference of φ on the box's outer ring: any gas on any edge met it | none — a wider world hid it for that disk only | **cannot** — a stencil, fixed |
| 15 | M0-D156 (DONE) | Sun-like: P5 at its 64-pass ceiling 221 times — **[M]** laserax-ai-server RTX 4090 | 21 floor cells at the top edge carry 95 % of the heat-free shift's weight: the solver's rule | none — thin floor cells sit at any edge **[A]** | **cannot** |
| 16 | M0-D138 (DONE) | After a supernova the ejecta fill the world and the remnant is a dot | Unbound gas leaves any finite world **[A]** — the block measured no size; its ruling was a ring on the remnant | none | **cannot** |
| 17 | §1.12 «Not enough room here for this star» (M0-D136, M0-D145) | A preset is refused where any cell would fall outside | The Giant (a 94.2 cells) can be dropped in 36.3 % of the world's area, the Massive (a 68.4) in 50.8 %, the Sun-like (a 38.7) in 70.2 % **[A]** | 800 × 800: 58.5 %, 68.7 %, 81.6 % **[A]** | **part** |
| 18 | §2.10's radius cap, 150 cells | Named as a size limit | Inert: it binds from 47,124 sb and the Giant is 18,579 (a 94.2) **[A]**. M0-D202's «a = 144 cells» is the pre-2026-11-13 Giant's (43,461 sb) **[A]** | — | **nothing** |
| 19 | §5.2 G-FIT's 16-cell band | Named as a size limit | 8 % of hh today, 4 % at 800 × 800 **[A]**; no defect of its own | — | **nothing** |
| 20 | §5.2 G-MULTI, two Suns 240 cells apart | Named for room | Fits today (600 wide) | — | **nothing** |

**Read across:** every row a bigger world removes or eases is about **the envelope's reach** (rows 1–3, 5, 17). Every conservation row
(7–15) was a booking, a rounding or a stencil that happened to show at an edge; all nine are fixed, none by room. Row 4 is the answer to
«the star loses half its mass through the edge»: it does in 800 × 800 too.

**M0-R15's ruling, re-read.** heavy_end.md § 2.1 kept 600 × 400 because the Giant of that day (43,461 sb) reached R₉₀ 827 in 800 × 800.
The calibration of 2026-11-13 moved the Giant to 18,578.6 sb; that preset change is the one the block's Adversarial line asks for, and it
has already happened. Today's Giant was measured here directly, so no further preset change is paired with the size — with one condition:
the recalibration a new size needs (§ 2) moves the masses again, and `fit` must be read again after it.

## (2) The cost
### 2.1 The candidate sizes, by arithmetic
Bounds first: with gravity on **no axis may pass 1,024 cells** (§1.3.3: the FFT pads to the power of two ≥ 2 × the box, at most 2,048),
and the default window (§4.2: 1,760 × 940) leaves the world **1,240 × 844 px** (220 + 300 px of panels, a 44 px bar, the 52 px legend) **[A]**.
The star's need is round, so the smaller half-extent is what counts: 600 × 400 spends 100 cells of half-width no star uses.

| size | cells (× today) | FFT when the box is the world (× today) | hh | closes row 1? | px per cell: default window · 1440p · 4K **[A]** | 14 planes |
|---|---|---|---|---|---|---|
| 600 × 400 (today) | 240,000 | 2,048 × 1,024 | 200 | no **[M]** | 2 · 3 · 5 | 13.4 MB |
| 512 × 512 | 262,144 (1.09) | 1,024 × 1,024 (0.5) | 256 | no — 285.5 > 240 **[A]** | 1 · 2 · 3 | 14.7 MB |
| 608 × 608 | 369,664 (1.54) | 2,048 × 2,048 (2.0) | 304 | by 2.5 cells **[A]** — as thin as the Massive's today | 1 · 2 · 3 | 20.7 MB |
| 640 × 640 | 409,600 (1.71) | 2,048 × 2,048 (2.0) | 320 | by 18.5 cells **[A]** | 1 · 2 · 3 | 22.9 MB |
| **800 × 800** | **640,000 (2.67)** | **2,048 × 2,048 (2.0)** | **400** | **by 98.5 cells [M]** | **1 · 1 · 2** | **35.8 MB** |
| 1,024 × 1,024 (M0's ceiling) | 1,048,576 (4.37) | 2,048 × 2,048 (2.0) | 512 | by 210.5 **[A]** | does not fit at 1 · 1 · 1 | 58.7 MB |

- **What every size that closes the Giant pays alike [A]:** the doubled FFT (any height over 512) and 1-pixel cells in the default window
  (2 px needs a height ≤ 422). So the choice among them is cells against margin, and the margin matters: the Giant's envelope is chaotic
  (row 3) and a 1.5-cell margin is what M0-T99b flagged as fragile on the Massive.
- **Memory is not a limit [A]:** 56 B a cell; the largest plane group at 1,024 × 1,024 stays under WebGPU's 128 MiB binding (state.rs's
  own bound); a dump grows from 13.4 to 35.8 MB.
- **The arithmetic's pick: 800 × 800** — the only candidate that both fits the default window and leaves the Giant a margin (35 % of its
  reach) a chaotic life can live inside. It is the one size measured below; the other rows stay arithmetic.

### 2.2 Measured on 800 × 800 (two lives and one step-cost line — nothing else was run)
| what | 600 × 400 | 800 × 800 | |
|---|---|---|---|
| Giant, nearest bound gas at Σ ≥ 10⁻² to an edge | 0.5 cells (76 of 167 checkpoints inside the band) | **114.5 cells** (step 12,514, t 77.1); 0 of 189 inside | **[M]** linux-pc RTX 5090: M0-T99b · this block |
| Giant, the same at Σ ≥ 10⁻¹ / 10⁻³ | 72.5 / 0.5 | 271.5 / 0.5 (the faint halo §0.5 [M0-T99a] allows; densest band cell Σ 4.0 × 10⁻³) | **[M]** same runs |
| Massive, nearest at Σ ≥ 10⁻² | 17.5 cells (t 131.2) | **217.5 cells** (t 131.2); 0 of 252 inside; densest band cell Σ 2.5 × 10⁻⁴ | **[M]** same boxes |
| Giant's life | black hole, step 16,786, t 93.4, 265 s | **not ended at the 570 s cap**: step 18,914, t 98.3, 10,509 sb, the sink there by t 64.3 (600 × 400: by t 64.2) | **[M]** |
| Massive's life | neutron star, step 23,170, 222 s | **not ended at the 570 s cap**: step 25,202, t 198.2, 4,697 sb, no sink | **[M]** |
| steps a second, the life with its checkpoints read | Massive 104 · Giant 63 | Massive 45 · Giant 33 — **2.3 × and 1.9 × slower** | **[M]** linux-pc RTX 5090 |
| one step, the box the whole world, `fill_world` | 20.8 ms (P4 12.7 · P5 4.8 · P2 1.2) | **47.2 ms** (P4 32.5 · P5 7.3 · P2 2.4) — **2.28 ×** | **[M]** linux-pc Quadro RTX 4000, gpu-timestamp, 200 steps, one run each |

- **Not measured:** either ending on 800 × 800 (both lives ran past the 10-minute line — the lead's runs, with the recalibration); the
  Sun-like (its G-FIT margin is 172.5 cells today; its box is small on the main sequence, so by G-BOX its cost there does not move **[A]**).
- **What the 2.3 × buys on the mid-range card [A]:** M0-T63's Sun-like step with the box the whole world is 10.7 ms on the Quadro; × 2.28
  ≈ 24 ms — under 60 frames a second at one step a frame, where today's fits. G-FPS reports and never blocks (M0-TJ2). A heavy life pays
  it nearly throughout: the Massive's halo is on the edge from step 2,705 of 23,170 (M0-T99b), so its box is the world from there.
- The 600 × 400 `fill_world` line is today's default as the baseline of the same scene, not a second candidate size.

### 2.3 The redo list, if the size changes
1. **The one recalibration** — `calibrate`'s lives run on the default world, and how much mass a leave edge takes decides an ending:
   m_up_sb, m_bh_sb, the three preset masses, life_sun_sb, top_rung and the stage durations are measured again (hours, about twice
   today's wall; the lead's run). **A trap:** `physics_hash` covers the three asset files and not the world's size (§2.11), so G-CAL
   would read today's calibration.json as fresh on the new world — the build block must make it stale. *(source read, not run)*
2. **The life scopes** read the default world: `fit`, `squeeze`, `stages`, `endings`, `touch`, `conservation`, `calibrate_bisect`,
   `multi`, the fps bench — each about 2 × its wall, and every run owed to M0-V15 with them.
3. **Dumps cut from preset lives** at 600 × 400: scenes/sun_13593.bin (+ .observers), sun_13900.bin, massive_930.bin,
   remnant_window.bin and M0-V14's cached dumps (a dump keeps its own world, so each still loads — as a 600 × 400 fixture, no longer
   «the default life»); touch_paint.json and touch_erase.json aim at a star's centre.
4. **Files that write 600 × 400 or 1,200 × 800 [M, a grep on linux-pc]:** 22 under crates/ (state.rs's default, three UI test
   constants, the rest tests), 10 scenes (9 name their own world and keep running at it), 4 docs, verify.json, 2 under tests/.
5. **The window:** §1.11's s falls to 1 in the default window, §4.2's «s = 2, 1,200 × 800 px» and the capture size move with it.
6. **The contract and the goal** — §2.8's default, B13's row, the goal paragraph's «a Powder-Toy-sized world of about 600 × 400 cells»:
   the lead's words to change, by an override line (§9).

## (3) The recommendation
**A default world of 800 × 800 cells**, with the recalibration it needs and `fit` read again after it.

- **Fixes** — row 1: the Giant fits (114.5 cells, **[M]** § 2.2), so M0-D202 closes with the preset and the gate untouched and M0-T99b
  can go green; row 2: the Massive's margin, 1.5 → 201.5 cells (**[M]** § 2.2); eases rows 3, 5 and 17 (a margin for the Giant's neighbours, room for a painted star to swell,
  room to drop a preset).
- **Does not fix** — the mass heavy stars lose (row 4: the same in both worlds, later); any conservation drift (rows 7–15: bookings,
  already fixed); the ejecta filling the screen (row 16). The lead's question «it seems like we have problems with such a small world
  size, is that right?» — for one open defect and one thin margin, yes; for the drifts and the lost mass, no.
- **Costs** — § 2.3's list: one recalibration, the life scopes' walls about doubled, the dumps, the files, the contract's and the goal's
  words.
- **Gives up** — the look: 1-pixel cells in the default window and on a 1440p screen, 2 px only at 4K (**[A]** § 2.1), against B13's «each drawn as a
  2–3 pixel square» (M0-T129's zoom shows 2 px cells on a part of the world; a default zoom is a product fork the build block would
  ask); and the step: 2.3 × whenever the gas fills the world (**[M]** § 2.2), which drops the mid-range card under 60 frames a second there
  (≈ 24 ms a step, **[A]** § 2.2).
- **Not proven** — that either heavy life still reaches its ending on 800 × 800 (both ran out the 10 minutes); that the Giant's
  neighbours fit (one life); that the recalibrated Giant fits. If the recalibrated Giant does not, M0-D202's option (1) — a lighter or
  narrower Giant — is what is left.

**If the lead stays at 600 × 400:** M0-D202's fork stands as filed — (1) a lighter or narrower Giant, which no in-bounds lever reached at
M0-T99a, or (2) the Giant's bound halo reported and not graded; the Massive keeps its 1.5-cell margin (`fit` re-run after any change to
its mass or outflow); rows 5 and 17 stay as ruled (bounce for the paint case, the refusal message).

## (4) The ruling
The lead (question tool, 2026-11-20), verbatim: **«800 × 800 (Recommended)»** — as put: «Closes M0-D202 with the Giant preset and the
G-FIT gate untouched. I file one build block (the default, the stale calibration, the window's scale, the contract's and goal's words by
override) — then the recalibration and the `fit` re-run are your runs. Gives up 2-pixel cells on a 1080p/1440p screen and 60 fps on the
mid-range card when gas fills the world. Not proven yet: both heavy lives' endings on this size (they ran past 10 minutes).» The two
options not taken: «Stay, Giant reported» (M0-D202's option 2) and «Stay, lighter Giant» (its option 1).

Written into the plan (this block's close) — no contract line, asset or code changed here:
- **M0-D203** filed — the default world 800 × 800: §2.8's default and the words that name it by override, state.rs's default, the
  calibration refused on a world it was not measured on (its own fork: how), and the fallout measured and filed, not fixed.
- **M0-D204** filed — the window at the new default (§1.11's s = 1, §4.2, the capture size), with its product fork: the whole world at
  1 px a cell, or opened zoomed. My reading of «one build block»: two, since the window is a second thread with a fork of its own.
- **M0-D202** retired `N/A — superseded` by this ruling: the Giant fits the ruled world as it is; its option (1) is the fallback if the
  recalibrated Giant does not (§ 3, Not proven).
- **M0-T99b** stays BLOCKED, now on M0-D203 and the recalibration; its Verify 1 rewritten for the 800 × 800 default (all three lives
  the lead's: the Massive and the Giant ran past 570 s there), its Carried flags given this block's readings.
- The census's other defects (rows 3–16) are closed blocks; none is woken or rewritten — a bigger world removes none of them.
- Flags to M0-V15 (the owed life runs read the default world: about twice as long after M0-D203, and they wait for the
  recalibration), M0-T112 and M0-T124 (the Quadro's whole-world step, 20.8 → 47.2 ms).

## Commands
The scratch copy (this session's scratchpad, not durable):
`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/a592e1e6-a449-44f7-8ad3-fff8c836469f/scratchpad/tree` — `tools/pb/scratch_copy.sh . <that path>`,
then § Appendix's three lines; built into the shared red-arm target (`cargo test --release --locked -p sr-engine --test gpu --no-run`, 11 s).

[ALREADY RUN — PASS (0 of 189 checkpoints inside the band; nearest 114.5 cells) then cut by the 570 s cap before the ending (exit 124) on linux-pc, RTX 5090] — from the scratch copy
```
(source tests/cargo.sh && cd "$_sr_root" && SR_FIT_WORLD=800x800 SR_FIT_NO_CUT=1 SR_TEST_ADAPTER="RTX 5090" timeout 570 cargo test --release --locked -p sr-engine --test gpu fit::the_giant -- --nocapture --test-threads=1)
```
[ALREADY RUN — PASS (0 of 252 checkpoints inside the band; nearest 217.5 cells) then cut by the 570 s cap before the ending (exit 124) on linux-pc, RTX 5090] — from the scratch copy
```
(source tests/cargo.sh && cd "$_sr_root" && SR_FIT_WORLD=800x800 SR_FIT_NO_CUT=1 SR_TEST_ADAPTER="RTX 5090" timeout 570 cargo test --release --locked -p sr-engine --test gpu fit::the_massive -- --nocapture --test-threads=1)
```
[ALREADY RUN — PASS (exit 0, step_ms_mean 47.23) on linux-pc, Quadro RTX 4000] — from the repo root
```
build/target/release/sandbox-reactions headless --scene scenes/fill_world.json --world 800x800 --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-r16/q800
```
[ALREADY RUN — PASS (exit 0, step_ms_mean 20.76) on linux-pc, Quadro RTX 4000] — from the repo root
```
build/target/release/sandbox-reactions headless --scene scenes/fill_world.json --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-r16/q600
```
Not run: win-laptop (R15: `owed` — the two lives and the step-cost line on its RTX 4080 Laptop); either life to its ending on 800 × 800.

**Deviations.** (1) The 600 × 400 `fill_world` line is a fourth run: today's default as the baseline of the 800 × 800 line, since no
whole-world step of that scene was on record — not a second candidate size. (2) The block's «one frame-time line from the step-cost
bench» was read with `headless --timing` (M0-T63's method; the `bench` command is M0-T109's, not built). (3) Each life was capped at
570 s and neither ended, so both are readings to t 98 and t 198, not whole lives.

## Appendix — the probe (scratch copy only; crates/sr-engine/tests/gpu/fit.rs, after the `SR_TEST_ADAPTER` lines of `run_life`)
```
    if let Ok(w) = std::env::var("SR_FIT_WORLD") {
        args.extend(["--world".to_string(), w]);
    }
```
`--world` re-centres the scene's objects (scene.rs), so the preset sits at the new world's centre; the reading takes each dump's own size.
