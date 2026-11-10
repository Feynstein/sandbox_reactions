# Heavy end — can a squeezed sandbox show a heavy star's whole stage ladder? (M0-R15)

M0-R15 · BUILD (research) · task on **linux-pc** (laserax-ai), every run on **laserax-ai-server, RTX 4090** (Vulkan, R18) · 2026-11-10 ·
model=claude-opus-5-5 level=high (`rung_record.py now`). Raw output: `logs/M0-R15.log`. Run folder: `build/r15/` on the server (screens
1–3, their scenes and runners). Probe scripts: § Appendix (`lifeclk.py`, `groups.py`, `coremass.py`, `order_r15.py`; M0-D163's `flash.py`
and `mkmap.py`, M0-D165's `order.py`, unchanged). Tags: **[M]** measured (box, run named), **[A]** arithmetic (my own, from the measured
numbers or the contract's constants), **[NP]** not proven (synthetic or concurrent-load timing).

**The physics every life below runs** is the shipped tree's: assets/physics.json sha256 cc1ff0e8… on both boxes (burn_k 10, the clock of
§2.5.2 [M0-D165]; ladder T_H 56 · T_He 100 · T_C 109.8 · T_Ne 153.4 · T_O 168 · T_Si 235.2; c_sb 800, a_r 10⁻⁵), the server's binary of
2026-11-10 11:05. The calibration is stale (a6c041b1… against the assets), so no preset scene runs; every life is a disk equal to its
preset (`mkmap.py`, the same scene rule) or a continuation of M0-D163's Verify 1 dumps (`--load-dump`: the Massive's
`v1/massive_rows/dumps/s_14000.bin`, t 138.7, X_H,c 0.275; the Giant's `v1/giant/dumps/s_1200.bin`, t 5.33, before ignition), each capped at
570 s, five at once on the RTX 4090. Lives "before the clock" are M0-D163's (calibration a6c041b1…, its v1 and item-6 runs) and M0-V14's.

## In one paragraph
**No, not as G-STAGES is written — and three of the four walls are not the ones M0-D166 and M0-D167 name.** (1) The late burning *does*
happen in the Massive's centre — oxygen burning to Si/S (L_O 4.7 × 10⁴, X_O 0.93 → 0.63) and carbon burning (L_C 1.1 × 10⁴) are measured —
but §1.9.3's c/ne/o/si_burn rows need that group to be the *star's* largest L_g, and the hydrogen shell (L_H 0.9–1.8 × 10⁵) always outshines
the core, so the stage reads he_shell or main_sequence. (2) The Massive's main-sequence mass loss is a wind, not the edge: a 600 × 800 world
loses 1,195 sb by t 114 against 1,357–1,558 in 600 × 400. (3) The C/O core is **2.8–4.9 cells** in radius (1,100 → 3,213 sb) — the degenerate
scale R₀ ≈ 4 cells — and the sandbox's mass ladder is squeezed: m_up_sb/m_ch_sb = 1.38 where nature has ≈ 5.6, so the Massive must build a
core of M_ch from a star only 3.4 M_ch heavy; with its envelope lost (leave) the core stops at 2,813 sb < M_ch and **the Massive ends a white
dwarf** — G-END's Massive row breaks on the clock physics; with the mass kept (bounce) the core grows to 3,213 sb (= M_ch), cold, and collapses
to a neutron star. (4) The Giant is never hydrostatic after ignition (τ_KH/τ_dyn 0.2–0.7): a helium-led event at t ≈ 11 throws its envelope
past any world up to 800 × 800 (R₉₀ 827), and with the mass kept it relaxation-oscillates (he_core flashes every 35–50 t.u.).
(5) **On the clock physics nothing up to m_bh collapses** within the caps: 10,837, 15,000 and 22,000 sb all burn their centres He → O → Si/S
(with the late ladder lowered, on to X_Fe,c 0.999), then meet the same event at t ≈ 180–285 — hydrogen in the densest cell and T_c at the floor
within one snapshot — and in leave end as 2,800–3,000 sb remnants under M_ch. So the one recalibration, on today's physics, would likely find no
m_up_sb below m_bh_sb (K9) — the ending is the wall to clear before it runs. **Only one knob moves the ladder**: the late ignition temperatures
lowered within K6 (T_Ne 118, T_O 122, T_Si 135 — G-ORDER holds) let the core burn to iron in order over ~50 t.u.; the light, the clock's K and
a taller world do not.

## (1) The constraints as numbers
### 1.1 The clocks and sizes, stage by stage
τ_dyn is the tracker's √(R₉₀³/GM), τ_KH = |W|/2L, both medians over the stage's snapshots; R/hh is R₉₀ over the world's half-height (200 in
600 × 400); M at the stage's first and last snapshot. Stages are the shown ones (§1.9.3), so a flicker is a row. `lifeclk.py` over each run's
observers snapshots; every row **[M] laserax-ai-server, RTX 4090**.

**Sun-like** (M0-V14's `build/v14/sun_life`, preset 3,113.6, before M0-D163 and the clock; M0-D163's item-6 life `calibrate_runs/2026-11-09T20-58-25Z/preset_sun-01`
for the stage times on a6c041b1…):

| stage | t (t.u.) | M in → out | R₉₀ (R/hh) | τ_dyn | τ_KH | τ_KH/τ_dyn |
|---|---|---|---|---|---|---|
| main_sequence | 12.8 → 1,589 (a6c041b1…: 9.2 → 1,631.7) | 3,109 → 3,049 | ≤ 19.0 (0.09) | 1.04 | 116 | **111** |
| he_core | 1,589 → 1,982 (a6c041b1…: 1,631.7 → 1,858.2) | 3,045 → 2,061 | ≤ 29.6 (0.15) | 1.6–2.2 | 18–30 | 8–19 |
| iron_core (a flash) → white_dwarf | 1,982 → 2,003.5 (a6c041b1…: → 1,872.6) | 2,026 → 1,871 | 12–18 | 1.0–1.7 | 82–216 | — |

τ_nuc/τ_KH on its main sequence 18.3–29.5 (M0-V14, reports/v14.md). The Sun-like fits the squeeze: every clock gap ≥ 10, R₉₀ under a sixth of
the half-height, 99 % of its mass bound through the main sequence.

**Massive** (10,836.6 sb), before the clock (M0-D163's `v1/massive_rows`, a6c041b1…) to t 160.3, then on the shipped clock from s_14000
(`build/r15/m_clk_leave`, edge leave; `m_clk_bounce`, edge bounce):

| stage | t | M in → out | R₉₀ (R/hh) | τ_dyn | τ_KH | τ_KH/τ_dyn | T_c max |
|---|---|---|---|---|---|---|---|
| main_sequence (before the clock) | 5.3 → 158.7 | 10,815 → 8,086 | 57–150 (0.29–0.75) | 7.8 | 16.5 | **2.1** | 102.6 |
| main_sequence ↔ he_core flickers (clock, leave) | 157.4 → 193.5 | 8,616 → 6,692 | 155–182 (0.77–0.91) | 19–30 | 10–14 | 0.3–0.7 | 120 |
| he_core (clock, leave) | 193.5 → 243.8 | 6,417 → 3,926 | ≤ 194 (0.97) | 38 | 7.1 | **0.18** | 131.5 |
| main_sequence (a shell label), he_shell (clock, leave) | 243.8 → 277.8 | 3,748 → 2,812 | 177 → 5 | 35 → 0.2 | 5.5 → 3.9 | — | 171.1 |
| the remnant, labelled he_core / main_sequence (clock, leave) | 277.8 → 287 (cap) | 2,812 → 2,813 | 2.0–2.3 | 0.05 | 4.4 | — | 24 (falling) |
| he_core (clock, bounce) | 193.5 → 244.0 | 8,768 kept | ≤ 298 (1.49, at the walls) | 55 | 7.9 | **0.14** | 131.9 |
| he_shell (clock, bounce) | 259.8 → 281.6 | 8,768 | ≤ 302 (1.51) | 55 | 5.1 | 0.09 | 177.2 |

Escaped (leave): 2,068 sb by t 138.7 (19 %), 2,750 by t 158.7, 5,928 by t 287 (55 % of the preset); the remnant is 26 % of the preset mass.

**Giant** (43,461 sb) from s_1200 on the shipped clock (`build/r15/g_clk_bounce`; leave from M0-D165's `opts/g_lim10`, the same law as a
GPU-only prototype — its Massive twin `opts/m_lim10` and the tree's `m_clk_leave` agree stage for stage to t 281 and to 1 sb at the remnant):

| stage | t | M | R₉₀ (R/hh) | τ_dyn | τ_KH | τ_KH/τ_dyn | T_c max |
|---|---|---|---|---|---|---|---|
| main_sequence (a contraction) | 6.5 → 11.2 | 43,050 → 42,381 | 156–165 (0.78–0.82) | 9.4–9.8 | 47–64 | 4.8–6.8 | 104–107 |
| he_core — the helium-led event | 11.2 → 12.5 | 42,235 | 158–181 | 9.6–11.7 | 2.0–3.2 | **0.2–0.3** | 162–163 |
| main_sequence (leave) | 11.9 → 81.3 | 42,091 → 18,095 | ≤ 657 (3.3) | 81 | 38 | 0.47 | 104 |
| main_sequence ↔ he_core flickers, he_shell (leave) | 81.3 → 276.8 | 18,092 → 4,346 | 164–223 | 19–49 | 5–15 | 0.1–0.7 | 149 |
| main_sequence / he_core flashes (bounce) | 12.5 → 137.6 | 43,238 kept | 180–602 (to 3.0, the box) | 12–50 | 2.4–33 | 0.2–0.7 | flashes of 171–178 at t 60.2, 96.4, 133.2 |

### 1.2 The core — where the late ladder would burn
`coremass.py` on dumps copied home (linux-pc, numpy), **[M]** from the RTX 4090 runs: the C/O-or-heavier core is the cells whose C … n sum
passes 0.5; r is its equal-area radius.

| life | t | M (whole) | C/O+ core (r) | Si+ core | centre T · Σ · d |
|---|---|---|---|---|---|
| Massive, clock, leave (`opts/m_lim10`) | 235.4 | 4,121 | 1,101 (2.8) | 0 | 129.5 · 70 · 0.72 |
| | 259.5 | 3,226 | 1,672 (3.1) | 0 | 146.5 · 110 · 0.77 |
| | 274.3 | 2,946 | 2,234 (4.1) | 577 (1.4) | **171.1** · 127 · 0.76 |
| | 281.0 | 2,879 | 2,813 (4.9) | 351 | **24.1** · 231 · 0.97 |
| Massive, H ν 8 + He ν 20 + ladder raised, bounce (`opts/m_h8he20_lad_bounce`) | 244.7 | 8,768 | 1,319 (2.5) | 0 | 163.8 · 97 · 0.72 |
| | 273.2 | 8,768 | 2,397 (3.5) | 0 | **188.2** · 155 · 0.76 |
| | 285.3 | 8,768 | **3,213** (3.9) | 0 | **35.3** · 304 · 0.96 |
| | 307.7 | 8,768 | (neutron_star: X_Fe,c 0.448, X_n,c 0.530, Σ_c 717) | | 0.9 · 717 · 1.00 |

Group powers at the centre's stages (`groups.py`, **[M]**): in `m_lim10` the centre goes He → O (X_O 0.93 at t 217, C_alpha having turned
the carbon) → **oxygen burning, L_O 6.1 × 10³ → 4.7 × 10⁴ (t 259–274), X_O 0.93 → 0.63, Si + S 0 → 0.35** — shown he_shell, since L_H was
9.0 × 10⁴ → 1.3 × 10⁵; in the bounce life **carbon burning, L_C 1.6 × 10³ → 1.06 × 10⁴ (t 229–252), X_C 0.46 → 0.13** — shown he_core and
main_sequence (L_H 6.6–8.8 × 10⁴).

### 1.3 What a full ladder needs of each constraint
- **(a) Hydrostatic stages (lives of minutes).** A stage that shows as a stage, not a flash, needs τ_KH ≫ τ_dyn (K1's precedent: ≥ 10).
  **[M]** the Sun-like has 111; the Massive 2.1 on its main sequence and 0.09–0.7 after it; the Giant 4.8–6.8 before ignition, 0.2–0.7 after.
  **[A]** τ_KH ∝ 1/L at a given structure, so the heavy stars need their light ÷ 5 (Massive's main sequence) to ÷ 50–100 (the post-main-
  sequence and the Giant) at fixed |W| — one law for every mass, so the Sun-like's life lengthens by the same factor where its light binds
  (its 1,872 t.u. life took 699 s of wall at Max with three lives sharing the card, M0-D163's item 6 **[M]**).
- **(b) Room (a world 400 cells tall).** **[M]** the Massive's main-sequence R₉₀ 57–150 (0.29–0.75 hh) and its envelope after it 155–223 in
  leave and ≥ 300 against bounce's walls (1.5 hh — a lower bound of its free radius); the Giant's eruption R₉₀ 600–830. **[A]** a world that
  holds the Massive's post-main-sequence envelope inside 16 cells of the edge (G-FIT) is ≥ 2 × 316 = 632 tall → 640; the Giant's eruption
  needs ≥ 1,700. **[M]** but a taller world does not stop the main-sequence loss (§ 2.1).
- **(c) The core's heat before degeneracy.** **[M]** the cores peak at T_c 171 (leave) and 188 (bounce) at Σ_c 127–155, d_c 0.76, then go cold
  on the degenerate branch (T_c 24–35 at Σ_c 231–304). **[M]** the ladder's ε* crossings (G-ORDER's cell, `order_r15.py`) on the shipped
  constants: H 114 · He 117 · C 137 · Ne 188 · O 208 · **Si 282.5** — so carbon and some oxygen burn, silicon cannot. **[A]** a core that peaks
  near 170–190 burns to silicon only if Si's crossing sits under ~165.
- **(d) The core's mass against the ceiling.** **[M]** calibration a6c041b1…: m_ign 2,636 · m_ch 3,216 · m_up 4,449 · m_bh 22,338 · m_tov 4,853.
  **[A]** m_up/m_ch = 1.38 and m_ch/m_ign = 1.22, where nature has 8/1.44 ≈ 5.6 and 1.44/0.08 ≈ 18: the sandbox's mass ladder is squeezed ~4×
  at the light end. A collapse needs a core above M_ch; the Massive (3.37 M_ch) must make one of ~30 % of its mass. **[M]** leave: its core
  stops at 2,813 (0.87 M_ch) when the envelope is gone — a white dwarf; bounce: 3,213 (0.999 M_ch), already degenerate, then past M_ch and a
  neutron star with no iron_core shown (the iron made and neutronized between two snapshots 1,000 steps apart).
- **(e) Resolution.** **[M]** the C/O core is 2.5–4.9 cells in radius, the cold disk's R₀ ≈ 4 cells (reports/cold_ceiling.md); every late
  stage burns in ≤ 5 cells, where the sheet's equilibria are not resolved (a cold disk holds only from R₉₀ ≳ 15, cold_ceiling.md). This is
  §5.5's "onion of burning shells in a ~60-cell star", now measured: the onion is five cells. **[A]** resolving it 3× needs a grid 3× finer
  (9× the cells, ~3× the steps per t.u.) — 27× the cost per t.u.
- **(f) Lives of minutes.** Not the binding wall: **[M]** the Massive's life is ~300 t.u. (5 min at ×1) and its 21,000 steps ran in 190 s
  wall (M0-D163's item 6, three lives sharing the RTX 4090); the Giant's 15,687 steps in 831 s.

## (2) The routes — what each costs and gives up
Every screen continues the same two dumps on the shipped clock binary with `overrides` in the scene (physics.json keys only, no contract
law changed, no scratch build), so each row differs from the control (`m_clk_leave` / `m_clk_bounce` / `g_lim10` / `g_clk_bounce`) by one
thing. **[M] laserax-ai-server, RTX 4090**, screens 1–5, 570 s each, five at once (`build/r15/r15*.status`: exit 124 = the cap, 0 = the
`--until` met; `g_ar4_leave` and `g_k3_leave` exit 2: the gas left the world, step cap 80,000 met at t > 32,000 on an empty grid).

### 2.1 A taller or larger world (D9: WorldConfig is a setting; §2.8: M0's UI never changes it)
| life | world | what it gives | escaped |
|---|---|---|---|
| Massive disk = preset from t 0 (`m_tall`) | 600 × 800 | main sequence to t 114 (cap), R₉₀ 55–117 (0.14–0.29 hh) | **1,195 sb by t 114** — against 1,357 (t 108) – 1,558 (t 118) in 600 × 400 |
| Giant disk = preset from t 0 (`g_tall`) | 800 × 800 | the helium-led event at t 11.1 (T_c 162) as in 600 × 400; R₉₀ to **827** (2.07 hh) | 6,223 sb by t 78 |
| Massive from s_14000, bounce (the upper bound of any world: nothing leaves) | 600 × 400 walls | 8,768 kept; he_core 193.5 → 244, he_shell, O → Si 0.40 + S 0.16 in the centre (L_O 5.5 × 10⁴); the centre cold at t 284.7; the core splits off the envelope as its own object, **2,939 sb (0.91 M_ch)**, shown o_burn | 0 |
| Giant from s_1200, bounce | 600 × 400 walls | relaxation oscillations: he_core flashes at t 60.2, 96.4, 133.2 (T_c 171–178, ~1 t.u. each) between main-sequence spells of 35–48 t.u., R₉₀ to 602 (the box) | 0 |

Cost: **[NP]** steps a second with five lives sharing the RTX 4090 — 600 × 400: 36.5–43.7; 600 × 800: 30.1; 800 × 800: 19.0 (the Giant's box
grows with its eruption, so part of its drop is the active box, not the world). **[A]** a 600 × 640 default (the Massive's envelope with G-FIT's
16-cell margin) is 1.6× the cells; the Giant needs ≥ 1,700 tall (≈ 4–7× the cells), and the default world is the size the lead was shown (B13).
Gives up: nothing physical. **Does not move** the main-sequence wind, the core, the ending, or the Giant's pulsation.

### 2.2 A different preset mass map (§2.10, §1.10.1's anchors) or a different ceiling
- **[M] the cores stall under their ceiling in every life at 10,837 sb**: C/O 2,813 (leave) and 2,939 (bounce) against M_ch 3,216; iron 1,431–1,440
  (dn2, below) against the iron ceiling **[A]** 1.4331 (K₂ₑ Y_e^{3/2})² = 2,575 at Y_e 26/56. The heavier disks of screen 5 (`build/r15/d15k`, `d15kb`, `d22k`,
  `d22kb`: `mkmap.py` disks of 15,000 and 22,000 sb — 22,000 ≈ m_bh_sb — from t 0, leave and bounce, `--until stage:neutron_star`, all at the
  570 s cap): main sequences of 131 and 82 t.u.; the centre burns He → O → O-burning (Si 0.69 + S 0.29 at t 171 in `d22kb`); then the same
  cold-centre event at t 182–222; **leave remnants 2,867 (15,000) and 3,012 (22,000)** — with the Massive's 2,813, the remnant is ~2,800–3,000 sb
  whatever the star, under M_ch; bounce: core_collapse flickers (L_N −38 at t 194.5 in `d22kb`, −5,570 at t 196.5 in `d22k`, X_n,c 0.001) and
  o_burn shown once (t 185.9), **no neutron star** by t 206–234.
- **[A] lowering the ceiling** (K₂ₑ; M_ch ∝ K₂ₑ²) until those cores pass it needs M_ch(C/O) < 2,813 — and the light end's remnants are 1,871
  (M0-V14's Sun-like) and 2,489 (the clock's 3,351 sb life): the window is 2,489 < M_ch < 2,813, 6 % wide each side, and m_ign 2,636 sits inside it
  (K9: m_ign < m_ch). Not a route worth a run.
- **[A] the calibration moves the anchors itself**: m_up_sb is measured as the white-dwarf / collapse divide (§2.11 item 4), so if 10,837 ends a
  white dwarf on the clock, m_up rises and §1.10.1 re-maps "15 Suns" heavier. Whether any mass below m_bh collapses to a neutron star on this
  physics is what screen 5 asks.

### 2.3 A heavy-end squeeze of its own (§2.5's bounds; physics.json only)
| change (overrides) | Massive from s_14000 | Giant from s_1200 | Sun-like disk t 0 → 60 |
|---|---|---|---|
| control (burn_k 10) | leave: centre He → O → O-burning (Si + S 0.35), cold at t 281, remnant **2,813** (WD-bound); bounce: as § 2.1 | leave: eruption at t 11.3, MS t 55 → 207, flickers, **4,346** at t 277 (M0-D165's `g_lim10`) | ignition 8.85, MS 9.24 (M0-D165) |
| **the light ÷ 4** (a_r 2.5 × 10⁻⁶ — also the push ÷ 4) | τ_KH/τ_dyn 0.94–1.03 on the MS (×1.5–2), MS to t 301–345 (×2.3), he_core 100 t.u.; leave still loses 42 % on the MS (8,615 → 5,001); at the cap (t 523–531) he_core/he_shell, no late stage | the helium event at **T_c 229**, **42,146 → 266 sb** by t 45 — blown apart | ignition 12.5 (from 9.2), τ_KH 302 (from 116) — the life ~2.6× longer |
| the burn clock slower (burn_k 40, bounce) | O burning starts (L_O 761, T_c 199), then **hydrogen in the centre** (X_H,c 0.27) and T_c → 0.1, core_collapse flickers (L_N < 0), no remnant at the cap | (M0-D165: cold degenerate centre at 6,660 sb by t 191) | (M0-D165: unchanged) |
| the burn clock faster (burn_k 3) | leave: 8,616 → 2,293 by t 475, no late stage; bounce: centre cold O/Si by t 338, a core_collapse flicker (L_N −49) at t 379, no neutron star | the event at **T_c 207**, **42,112 → 269** by t 31 | — |
| **the late ladder lowered** (dn2: T_Ne 118, T_O 122, T_Si 135 — G-ORDER holds: H 114 · He 117 · C 137 · Ne 144.5 · O 151 · Si 162) | **the centre burns He → O → Si/S → Fe** (bounce and leave alike): O-burning t 193 → 222 (L_O to 6.6 × 10³, Si 0.46), Si-burning t 222 → 241 (L_Si to 2.1 × 10³, Fe 0.34 → 0.985), **X_Fe,c 0.999 from t 250**, iron core 971 → 1,431 sb (r 1.9–2.8); shown he_core / main_sequence / he_shell throughout; the centre cold at t 280; leave remnant 2,641 | as the control (the Giant's wall is its helium event, not the ladder) | — (**[A]** the Sun-like's helium core peaks at T_c 91.8, M0-V14, under dn2's O crossing 151) |
| dn1 (T_Ne 125, T_O 130, T_Si 150) | the same ladder slower: Si-burning t 240 → 273, X_Fe,c 0.48 at t 273 before the centre goes cold | — | — |
| dn2 + burn_k 3, leave | iron_core **shown** at t 308.4 on a 2,192 sb remnant (X_Fe,c 0.966, Σ_c 220, d_c 0.93) — a white-dwarf-mass iron core | — | — |
| H ν 8 + He ν 20 + ladder raised (M0-D165's screen 3, scratch contract) | bounce: C burning (L_C 1.06 × 10⁴, X_C 0.46 → 0.13), core to 3,213 = M_ch, cold, then neutron_star at t 307.7 (X_Fe,c 0.448 + X_n,c 0.530) with no iron_core shown | pulsating, iron at t 40.95 from X_H,c 0.596 (D165's Fail arm) | unchanged |

Readings:
- **Only the lowered ladder moves the core's ladder**, and it moves it all the way: a centre that peaks at T_c 156–188 (§ 1.3 (c)) burns to
  iron in ~50 t.u., in order, at the burn clock's pace — the onion the lead asked for (B5 «burns up to iron», B18's element view «fusion's ash
  pile up in the core, layer by layer»). Cost: the physics hash moves (the one recalibration, planned anyway); the stage durations change (G-SQUEEZE's
  O ~29 t.u., Si ~19 against He ~35–50 and H ~153); carbon is never its own phase (C_alpha turns the core's carbon to oxygen as it forms, X_C,c
  ≤ 0.13) — so c_burn and ne_burn would be parenthesised in G-STAGES, where nature has them.
- **The light, the clock's K and the late ν do not help** (M0-D165 measured ν; here a_r and burn_k): each either leaves the core as it was or
  blows the Giant apart.
- **The labels hide the ladder in every variant.** What shows is what §1.9.3 reads: the star's largest L_g, the hydrogen shell's.

### 2.4 Labels that read the core (§1.9.3 — a contract change by asking)
**[M]** the stage rows already show the core's stage when the core is its own object: `m_clk_bounce`'s 2,939 sb core read **o_burn** (streak 110)
once the tracker split the envelope off. **[A]** reading the burning rows (c/ne/o/si_burn) on the core — the group whose power is largest inside
2 R_core, the radius §1.9.2 already computes — and iron_core on X_Fe,c ≥ 0.5 with the centre no longer burning (the star's L_nuc, which the H
shell keeps high, left out) would show the ladder that § 2.3 measures. Cost: §1.9.3's text, crates/sr-physics/src/observe/stages.rs's rows and
their tests, a tracker field per group inside 2 R_core (one more reduction), a block of its own; no physics hash move. Gives up: the readout
naming the shell's burning while the core burns (the real label convention names the core: «Supergiant: burning oxygen in the core»).

### 2.5 Amending G-STAGES / G-SQUEEZE to what the sandbox shows honestly
**[M]** what every clock life shows, whatever the knob: main_sequence (~150 t.u.) → he_core (35–100 t.u., flickering with main_sequence while
the shell leads) → the core's later burning (hidden today, § 2.4) → a cold centre at t ≈ 280. A short ladder — massive: main_sequence, he_core,
(c/ne/o/si_burn), (iron_core), core_collapse, supernova, neutron_star; giant: main_sequence, (he_core), core_collapse, (supernova), black_hole —
is an honest target only if the ending is reached; it changes no physics and costs the lead's ask of a visible ladder (B5's «burns up to iron» left
to the element view).

### 2.6 The wall none of these moves: the ending
**[M]** no clock life at 10,837 sb reaches core_collapse → neutron_star (the control, both edges, a_r, burn_k 3/40, dn1, dn2, dn2 + k3) within
the 570 s caps; every one of them meets, at t ≈ 272–285, the same event: **hydrogen appears in the densest cell (X_H,c 0.05–0.27) and T_c falls to
the floor (0.0–0.1, d_c 1.00, Σ_c 180–305) inside one 500–1,000-step snapshot** — in `m_lim10`, `m_clk_bounce`, `m_k40_bounce`, `m_dn2_*`,
`m_dn1_*` and M0-D165's ν-cut life. Before the clock (M0-D163's item 6) the same preset went iron_core → core_collapse → supernova → neutron_star
at t 160–176, through the flash. Whether a heavier star collapses on the clock is screen 5's (§ 2.2).

## (3) What each route does to the blocks it touches
| route | M0-D166 (edge loss) | M0-D167 (cold C/O centre) | M0-T103 (G-STAGES) | M0-T98b (G-SQUEEZE) | M0-T99b (G-FIT) |
|---|---|---|---|---|---|
| 2.1 taller world | reshapes: the main-sequence loss is a wind (same in 600 × 800) — a world cannot close it; closes only the post-MS envelope's R ≥ hh | untouched (bounce, its upper bound, still stalls the core) | untouched | untouched | reshapes: G-FIT's default world grows |
| 2.2 mass map / ceiling | untouched | the ceiling is the ending's wall; K₂ₑ has no room (2,489 < M_ch < 2,813) | the presets move with the anchors | the main-sequence order moves with them | the presets' radii move |
| 2.3 dn2 ladder | untouched | **reshapes**: carbon is not the wall — the centre burns to iron (X_Fe,c 0.999) and *then* goes cold; the wall is the t ≈ 280 event and the iron core under its ceiling | **reshapes**: o_burn, si_burn, iron_core happen in the core; c_burn and ne_burn never their own phase | **reshapes**: measurable H > He > O > Si once the labels read the core | untouched |
| 2.4 labels read the core | untouched | untouched | **closes the label half**: the ladder § 2.3 measures becomes the displayed one | needs it (stage durations are read from displayed stages) | untouched |
| 2.5 amend the targets | retires its G-STAGES arm | retires its stage arm, keeps the ending | rewritten | rewritten | untouched |
| 2.6 the ending | — | carries it | its neutron_star / black_hole rows | — | — |

## The light end (M0-D165's second carried flag)
M0-D165 flagged it here: «Measure where the clock binds over the Sun-like's whole life and whether burn_k (or the law's form) holds across
the mass range before the one recalibration». What the disk holds, **[M] laserax-ai-server, RTX 4090**:
- **The same 3,351 sb life on both physics** (calibrate item 4's first life, m_up_sb-01): on a6c041b1… (`calibrate_runs/2026-11-09T18-59-04Z`)
  collapsing 1.5 → main_sequence 9.1 → he_core 1,214.0 → **iron_core 1,400.1** → he_shell → iron_core → main_sequence → iron_core →
  white_dwarf 1,418.5 (step 132,113); on the clock (`calibrate_runs/2026-11-10T16-07-13Z`, M0-D165's stopped run) identical to he_core
  1,214.0, then he_core → main_sequence 1,459.7 → he_core 1,465.8 → **main_sequence from 1,483.7 to t 3,464 (step 450,001)**. So the clock
  binds in the light star's late helium core — the degenerate core's flash that took it to iron (an iron white dwarf at 3,351 sb, which nature
  does not make) — and nowhere before it: the same stage times to the digit through 1,214 t.u., and M0-D165's Sun-like disk to t 60
  unchanged (ignition 8.85, main_sequence 9.24).
- **What it leaves** (the checkpoint's observers): a 2,489 sb C/O remnant (X_O 0.861, X_C 0.022), R 2, T_c 0.050, d_c 1.000 — the right
  remnant (nature's 1–8 Suns end as C/O white dwarfs) — **misread**: raw main_sequence for 19,432 frames, because §1.9.3's white_dwarf row
  needs settled() (2P/|W| reads 0.78 < 0.8 on the 2-cell remnant) and main_sequence's burns() reads L_H 3.85 against L_nuc 5.15 alone, not
  against L 1.79 × 10⁵; its ledger books floor_added 3.40 × 10⁸ against radiated 3.60 × 10⁸ (the floor's heat leaving as light, 18× the
  nuclear energy). So calibrate's item 4 cannot end a light life on this physics. **Filed: M0-D168** (after this block, before the
  recalibration).
- **Across the mass range** the clock removes the flash at 3,351 (light), 10,837 (Massive) and 43,461 sb (Giant) [M0-D165's screens and
  § 1.1] and leaves the Sun-like's early life as it was. Whether burn_k 10 holds over the Sun-like preset's *whole* life is not measured: its
  ~172,000 steps are over 10 minutes (M0-D163's item 6: 699 s with three lives sharing the card) — owed on the one recalibration (M0-D165's
  first flag carries the run; M0-D168 blocks its ending).

## The ruling
The lead (question tool, 2026-11-10), verbatim:
- Route: **«Ladder down + core labels (Recommended)»** — as put: lower T_Ne/T_O/T_Si to 118/122/135 in physics.json (in bounds, G-ORDER holds);
  stage labels read the core's burning, not the whole star's (contract §1.9.3 change); G-STAGES keeps the Massive's ladder with c_burn and
  ne_burn optional; M0-D167 rewritten to the ending wall; M0-D166 rewritten to the real causes (a main-sequence wind and the Giant's helium
  eruption, not the world size).
- The recalibration: **«Hold until endings work (Recommended)»** — the one recalibration runs only after M0-D168 and the rewritten ending block
  make a heavy star collapse again; M0-D165's owed Verify stays carried.

Written into the plan (this block's close): **M0-D169** filed (the ladder hidden by the labels, and the ruled build: the ladder down, the core
labels, G-STAGES' massive row) after M0-D168; **M0-D166** rewritten and woken (the main-sequence wind and the Giant's helium eruption); **M0-D167**
rewritten and woken (the ending: the t ≈ 180–285 cold-centre event and the cores under their ceiling; then the one recalibration, M0-D165's two
flags moved to it); order R15 → D168 → D169 → D166 → D167. Flags to M0-T103, M0-T98b and M0-T99b.

## Appendix — the probe scripts (verbatim)
### lifeclk.py (server, no numpy)
```python
#!/usr/bin/env python3
"""M0-R15: a life's clocks from its observers' snapshots (no numpy). Per snapshot of the heaviest object: t, shown stage, M, R90,
R/(H/2), L, L_nuc, W, tau_dyn (the tracker's), tau_KH = |W|/2L, T_c, Sigma_c, d_c, X_c (H, He, C+O, Fe), escaped mass. Then per
contiguous shown stage: t from/to, duration, M at entry/exit, R90 max, medians of tau_dyn, tau_KH, L, and tau_KH/tau_dyn.
Usage: lifeclk.py [--rows] <run dir>..."""
import json, sys, glob, re, os
rows_out = '--rows' in sys.argv
def med(v):
    v = sorted(x for x in v if x == x)
    return v[len(v) // 2] if v else float('nan')
for d in [a for a in sys.argv[1:] if not a.startswith('--')]:
    snaps = sorted(glob.glob(d + '/dumps/s_*.observers.json'), key=lambda p: int(re.search(r's_(\d+)', p).group(1)))
    rows = []
    for p in snaps:
        try: j = json.load(open(p))
        except Exception: continue
        objs = j['watch']['tracker']['objects']
        if not objs: continue
        o = max(objs, key=lambda x: float(x['mass']))
        st = {s['id']: s['shown'] for s in j['watch']['stages']}.get(o['id'], '?')
        H = j['world'][1]; f = lambda k: float('nan') if o.get(k) is None else float(o[k])
        L = f('luminosity'); W = f('potential'); xc = [float(v) for v in o['x_c']]
        rows.append(dict(step=j['step'], t=float(j['sim_time']), st=st, M=f('mass'), R=f('radius'), Rh=f('radius') / (H / 2), L=L,
                         Ln=f('l_nuc'), W=W, td=f('tau_dyn'), tk=abs(W) / (2 * L) if L > 0 else float('inf'), Tc=f('t_c'),
                         Sc=f('sigma_c'), dc=f('d_c'), xh=xc[0], xhe=xc[1], xco=xc[2] + xc[3], xfe=xc[8], xn=xc[9],
                         esc=float(j['ledger']['last']['mass']['escaped']), rc=f('r_core'), H=H, lives=j['watch']['lives']))
    if not rows: print(d, 'no snapshots'); continue
    print('=== %s — %d snapshots, steps %d..%d, t %.2f..%.2f, world height %d' % (d, len(rows), rows[0]['step'], rows[-1]['step'],
          rows[0]['t'], rows[-1]['t'], rows[0]['H']))
    lv = rows[-1]['lives']
    for oid, hist in lv:
        if hist: print('  shown stages, object %d: %s' % (oid, ' '.join('%s(%.1f)' % (h[0], float(h[2])) for h in hist)))
    if rows_out:
        print('  %7s %8s %-14s %8s %6s %5s %9s %9s %10s %7s %8s %7s %7s %5s %6s %6s %6s %6s %7s' % ('step', 't', 'stage', 'M', 'R90', 'R/hh',
              'L', 'L_nuc', 'W', 'tau_dyn', 'tau_KH', 'T_c', 'Sig_c', 'd_c', 'X_H', 'X_He', 'X_CO', 'X_Fe', 'escaped'))
        for r in rows:
            print('  %7d %8.2f %-14s %8.0f %6.1f %5.2f %9.3g %9.3g %10.4g %7.2f %8.2f %7.1f %7.2f %5.2f %6.3f %6.3f %6.3f %6.3f %7.0f' % (
                r['step'], r['t'], r['st'], r['M'], r['R'], r['Rh'], r['L'], r['Ln'], r['W'], r['td'], r['tk'], r['Tc'], r['Sc'], r['dc'],
                r['xh'], r['xhe'], r['xco'], r['xfe'], r['esc']))
    segs = []
    for r in rows:
        if segs and segs[-1][0] == r['st']: segs[-1][1].append(r)
        else: segs.append((r['st'], [r]))
    print('  %-14s %8s %8s %7s %8s %8s %6s %5s %7s %8s %8s %7s %6s' % ('stage(snaps)', 't_from', 't_to', 'dur', 'M_in', 'M_out', 'R90max',
          'R/hh', 'tau_dyn', 'tau_KH', 'L', 'KH/dyn', 'T_cmx'))
    for i, (st, rs) in enumerate(segs):
        t_to = segs[i + 1][1][0]['t'] if i + 1 < len(segs) else rs[-1]['t']
        td, tk = med([r['td'] for r in rs]), med([r['tk'] for r in rs])
        print('  %-14s %8.2f %8.2f %7.2f %8.0f %8.0f %6.1f %5.2f %7.2f %8.2f %8.3g %7.2f %6.1f' % ('%s(%d)' % (st, len(rs)), rs[0]['t'], t_to,
              t_to - rs[0]['t'], rs[0]['M'], rs[-1]['M'], max(r['R'] for r in rs), max(r['Rh'] for r in rs), td, tk, med([r['L'] for r in rs]),
              tk / td if td > 0 else float('nan'), max(r['Tc'] for r in rs)))
```

### groups.py (server, no numpy)
```python
#!/usr/bin/env python3
"""M0-R15: per snapshot of the heaviest object, the shown stage, L, each group's L_g (H He C Ne O Si N) and the centre's full X_c.
Usage: groups.py <run dir> [every]"""
import json, sys, glob, re
d = sys.argv[1]; ev = int(sys.argv[2]) if len(sys.argv) > 2 else 1
snaps = sorted(glob.glob(d + '/dumps/s_*.observers.json'), key=lambda p: int(re.search(r's_(\d+)', p).group(1)))
print('%7s %8s %-14s %7s %8s | %9s %9s %9s %9s %9s %9s %9s | X_c H He C O Ne Mg Si S Fe n | T_c Sig_c d_c' % (
    'step', 't', 'stage', 'M', 'L', 'L_H', 'L_He', 'L_C', 'L_Ne', 'L_O', 'L_Si', 'L_N'))
for i, p in enumerate(snaps):
    if i % ev and i != len(snaps) - 1: continue
    j = json.load(open(p)); objs = j['watch']['tracker']['objects']
    if not objs: continue
    o = max(objs, key=lambda x: float(x['mass'])); st = {s['id']: s['shown'] for s in j['watch']['stages']}.get(o['id'], '?')
    lg = [float(v) for v in o['l_group']]; xc = [float(v) for v in o['x_c']]
    print('%7d %8.2f %-14s %7.0f %8.3g | %s | %s | %.1f %.1f %.2f' % (j['step'], float(j['sim_time']), st, float(o['mass']), float(o['luminosity']),
          ' '.join('%9.3g' % v for v in lg), ' '.join('%.3f' % v for v in xc), float(o['t_c']), float(o['sigma_c']), float(o['d_c'])))
```

### coremass.py (linux-pc, numpy; imports M0-D163's flash.py from its folder)
```python
#!/usr/bin/env python3
"""M0-R15: per dump, the cores by composition — M of gas, the He-rich core (X_H < 0.1), the C/O-or-heavier core (sum of C..n > 0.5),
the Si-or-heavier core (Si+S+Fe+n > 0.5) — their equal-area radii, and the centre's T, Sigma, d (flash.py's state). Reads dumps only.
  python3 -I coremass.py <assets> <dump>..."""
import sys, math
import numpy as np
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from flash import read_dump, state, PH
for p in sys.argv[2:]:
    h, pl = read_dump(p); st = state(pl); s = np.where(st['s'] >= PH['sigma_vac'], st['s'], 0.0); X = st['X']
    heavy = X[2:10].sum(axis=0); sife = X[6:10].sum(axis=0)
    iy, ix = np.unravel_index(np.argmax(s), s.shape)
    def core(mask): m = float(s[mask].sum()); n = int(mask.sum()); return m, math.sqrt(n / math.pi)
    mhe, rhe = core((X[0] < 0.1) & (s > 0)); mco, rco = core((heavy > 0.5) & (s > 0)); msi, rsi = core((sife > 0.5) & (s > 0))
    print('%-40s step %6d t %7.2f M %7.0f | He-core %6.0f (r %5.1f) | C/O+ core %6.0f (r %5.1f) | Si+ core %6.0f (r %4.1f) | centre T %6.1f Sig %6.1f d %.2f mu %.2f Ye %.3f' % (
        p.rsplit('/', 2)[-2] + '/' + p.rsplit('/', 1)[-1], h['step'], h['sim_time'], s.sum(), mhe, rhe, mco, rco, msi, rsi,
        st['T'][iy, ix], s[iy, ix], st['d'][iy, ix], st['mu'][iy, ix], st['ye'][iy, ix]))
```

### r15screen.sh (build/r15 on the server; scenes from mkmap.py (M0-D163) re-sized)
```bash
#!/bin/bash
# M0-R15 screen: the shipped clock physics (burn_k 10, the tree's binary of 2026-11-10 11:05), RTX 4090, each <= 570 s.
cd ~/workspace/sandbox_reactions/build && mkdir -p r15 && cd r15 || exit 1
B=../target/release/sandbox-reactions
G=../m0-d163/v1/giant/dumps/s_1200.bin
M=../m0-d163/v1/massive_rows/dumps/s_14000.bin
run() { local n=$1; shift; timeout 570 $B headless "$@" --adapter "RTX 4090" --out $n > $n.out 2>&1; echo "$n exit $? $(grep -h 'SR-HEADLESS DONE\|SR-ERROR' $n.out | tail -1)" >> r15.status; }
rm -f r15.status r15.done
run m_clk_bounce --scene clk_bounce.json --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run m_clk_leave  --scene clk_leave.json  --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run g_clk_bounce --scene clk_bounce.json --load-dump $G --until stage:iron_core --max-steps 80000 --dump-every 100 &
run m_tall --scene m_tall.json --until stage:iron_core --max-steps 80000 --dump-every 1000 &
run g_tall --scene g_tall.json --until stage:iron_core --max-steps 80000 --dump-every 200 &
wait; touch r15.done
```

### r15screen2.sh (build/r15 on the server)
```bash
#!/bin/bash
# M0-R15 screen 2: the shipped clock binary with physics.json overrides (a_r / 4; burn_k 40 in bounce), RTX 4090, each <= 570 s.
cd ~/workspace/sandbox_reactions/build/r15 || exit 1
B=../target/release/sandbox-reactions
G=../m0-d163/v1/giant/dumps/s_1200.bin
M=../m0-d163/v1/massive_rows/dumps/s_14000.bin
run() { local n=$1; shift; timeout 570 $B headless "$@" --adapter "RTX 4090" --out $n > $n.out 2>&1; echo "$n exit $? $(grep -h 'SR-HEADLESS DONE\|SR-ERROR' $n.out | tail -1)" >> r15b.status; }
rm -f r15b.status r15b.done
run m_ar4_leave  --scene ar4_leave.json  --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run m_ar4_bounce --scene ar4_bounce.json --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run g_ar4_leave  --scene ar4_leave.json  --load-dump $G --until stage:iron_core --max-steps 80000 --dump-every 100 &
run m_k40_bounce --scene k40_bounce.json --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run s_ar4 --scene s_ar4.json --until sim_time:60 --dump-every 1000 &
wait; touch r15b.done
```

### r15screen3.sh (build/r15 on the server)
```bash
#!/bin/bash
# M0-R15 screen 3: the late ladder lowered (dn2: t_ne 118, t_o 122, t_si 135; dn1: 125, 130, 150 — G-ORDER holds), RTX 4090, each <= 570 s.
cd ~/workspace/sandbox_reactions/build/r15 || exit 1
B=../target/release/sandbox-reactions
G=../m0-d163/v1/giant/dumps/s_1200.bin
M=../m0-d163/v1/massive_rows/dumps/s_14000.bin
run() { local n=$1; shift; timeout 570 $B headless "$@" --adapter "RTX 4090" --out $n > $n.out 2>&1; echo "$n exit $? $(grep -h 'SR-HEADLESS DONE\|SR-ERROR' $n.out | tail -1)" >> r15c.status; }
rm -f r15c.status r15c.done
run m_dn2_leave  --scene dn2_leave.json  --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run m_dn2_bounce --scene dn2_bounce.json --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run m_dn1_bounce --scene dn1_bounce.json --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run m_dn1_leave  --scene dn1_leave.json  --load-dump $M --until stage:iron_core --max-steps 80000 --dump-every 500 &
run g_dn2_leave  --scene dn2_leave.json  --load-dump $G --until stage:iron_core --max-steps 80000 --dump-every 100 &
wait; touch r15c.done
```

### order_r15.py — M0-D165's order.py (tasks/M0-D165.md § Appendix) with its variant list replaced by
```python
import itertools
base = {}
for name, ov in [('shipped', {}),
                 ('dn1 ne125 o130 si150', dict(t_ne=125, t_o=130, t_si=150)),
                 ('dn2 ne118 o122 si135', dict(t_ne=118, t_o=122, t_si=135)),
                 ('dn3 ne130 o135 si160', dict(t_ne=130, t_o=135, t_si=160)),
                 ('dn4 c105 ne120 o125 si140', dict(t_c=105, t_ne=120, t_o=125, t_si=140))]:
    o, ok = crossings(ov=ov); print('%-28s %s  G-ORDER %s' % (name, ' '.join('%s %s' % (k, v) for k, v in o.items()), 'holds' if ok else 'BROKEN'))
```

Scenes (build/r15, JSON): `clk_leave.json` / `clk_bounce.json` — world 600 × 400, edge leave / bounce, every switch on, no objects, no overrides (the `--load-dump` carries the state); `m_tall.json` — `mkmap.py 10836.552761071447`'s disk (a 71.93, T₀ 42.5871) in 600 × 800 at (300, 400); `g_tall.json` — `mkmap.py 43460.9873614944`'s (a 144.05, T₀ 85.3346) in 800 × 800 at (400, 400); `ar4_*.json` — clk's with `overrides` {a_r 2.5e-6}; `k40_bounce.json` {burn_k 40}; `s_ar4.json` — `mkmap.py 3136.7178531263357`'s Sun-like disk (a 38.70, T₀ 4.5792) with {a_r 2.5e-6}; `dn1_*.json` {t_ne 125, t_o 130, t_si 150}; `dn2_*.json` {t_ne 118, t_o 122, t_si 135}.
