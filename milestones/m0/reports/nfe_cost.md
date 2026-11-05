# M0-T131 — N_Fe's cost at nature's share: the ratio bound is reachable, the lives break inside it

Box: linux-pc (laserax-ai) for edits; laserax-ai-server, RTX 4090 for the twin crop and every life (R18). Model: claude-sonnet-5-5,
level high (`rung_record.py now`). Date 2026-11-05. Tree: M0-D71g's (the working tree, uncommitted), M0-D153's s_900 dump for the twin crop.
Raw: logs/M0-T131.{sweep,lives,r13core}.log; scratch sources logs/M0-T131.scratch.{r13twin.rs, twin_nfe.diff, lives.sh}.
Verdict: **BLOCKED** (the lead's pick, 2026-11-05: «Keep −1.14, BLOCKED (Recommended)») — the value stays −1.14.

## 1. The ratio, by q_ratio (the twin crop, s_900 + 120 steps, Massive)
M0-R13's tally (logs/M0-R13.scratch.r13twin.rs, mod.rs's R13 diff) over the 128 × 128 crop centred on the max-Σ cell, the core being
Σ ≥ 10 at each step's start, with one addition: **N_Fe's own absorbed heat** = Δ(Σ·X_n over the core) across P6 × q (the P6 total
mixes in the burning records and, at small q, is positive, so it is not N_Fe's). The ratio is P4's heat into the core ÷ |N_Fe's|.
Start: logs of M0-D153's Massive (laserax-ai-server /tmp/d153/massive_a/dumps/s_900.bin; core_collapse fires at step 900 there).
Baseline check: q −1.14 reads P4 +2.13 × 10⁶ and P6 −2.69 × 10⁷ (ratio 0.079) against M0-R13's +2.23 × 10⁶, −2.77 × 10⁷ (0.08).

| q_ratio | P4 into core | N_Fe absorbed | ratio | Σ_c at +120 | held |
|---|---|---|---|---|---|
| −1.14 (today) | +2.132e+06 | −2.768e+07 | 0.08 | 964.9 | 3 |
| −0.57 | +2.365e+06 | −1.354e+07 | 0.17 | 1080.3 | 3 |
| −0.228 | +2.129e+06 | −3.988e+06 | 0.53 | 1014.0 | 1 |
| −0.114 | +5.866e+05 | −4.419e+05 | 1.33 | 436.9 | 0 |
| −0.057 | +6.528e+05 | −2.142e+05 | 3.05 | 448.9 | 0 |
| −0.0228 | +5.164e+05 | −7.874e+04 | 6.56 | 413.6 | 0 |
| **−0.0187** | +4.995e+05 | −6.398e+04 | **7.81** | 410.5 | 0 |
| −0.016 | +4.882e+05 | −5.448e+04 | 8.96 | 408.6 | 0 |
| −0.013 | +4.796e+05 | −4.404e+04 | 10.89 | 407.3 | 0 |
| −0.0114 | +4.733e+05 | −3.849e+04 | 12.30 | 406.3 | 0 |
| −0.0100 | +4.679e+05 | −3.366e+04 | 13.90 | 405.4 | 0 |
| **−0.0092** | +4.617e+05 | −3.092e+04 | **14.93** | 404.2 | 0 |
| −0.0085 | +4.590e+05 | −2.853e+04 | 16.09 | 403.8 | 0 |
| −0.0065 | +4.514e+05 | −2.175e+04 | 20.76 | 402.7 | 0 |
| −0.0057 | +4.486e+05 | −1.905e+04 | 23.55 | 402.3 | 0 |
| **−0.0045** | +4.443e+05 | −1.502e+04 | **29.59** | 401.7 | 0 |
| −0.00114 | +4.331e+05 | −3.789e+03 | 114.31 | 400.1 | 0 |

- The curve is smooth and monotone for |q| ≤ 0.1 (ratio ∝ 1/|q|^0.93); **the bound [7.5, 30] is reachable: q ∈ ≈ [−0.0187, −0.0045],
  nearest 15 at −0.0092 (14.93).** The refusal arm's «saturates as q → 0» does not happen.
- **The curve has a step at |q| ≈ 0.1–0.2:** at |q| ≥ 0.228 the crop's core compresses to Σ_c ≈ 1,000 and holds; at |q| ≤ 0.114 it
  stays at Σ_c ≈ 400–450 with nothing held. The whole bound is in the shallow regime: in this engine N_Fe's heat absorption is what
  deepens the collapse (it keeps the core cold), so nature's ratio and the deep collapse do not coexist.
- Self-consistency: the crop restarted from the q = −0.0092 life's own s_880 (its core_collapse step; it moved from 900 to 880) reads
  13.53, from its s_900 12.67 (the same bound).

## 2. The lives at q = −0.0092 (Verify 2, scratch tree, calibration's physics_hash stamped)
`bash logs/M0-T131.scratch.lives.sh` on /tmp/t131/w2 (M0-T130's script, M0-D153's life 01 bracket 4572.394,67020.643), each capped at
570 s; the Repro continued from s_3062 to 3,900. The control: the same script on a tree with q = −1.14 (the tree's own value, stamped
alike: /tmp/t131/w6) — D155's and D156's heat changes landed after M0-D153's lives, so M0-D153's numbers are not this tree's.

| life | q = −0.0092 | control q = −1.14, same tree |
|---|---|---|
| Massive, core at step 1,000 (r13core) | held 0, Σ_c 410, **T_c 1.005 × 10⁹ K** (D's 1.3 × 10⁹), ε_th/ε_cold 0.35, X_n,c 0.165 | held 4, Σ_c 780, T_c 15.7 K, ε_th/ε_cold 1.3 × 10⁻⁴, X_n,c 0.558 |
| Massive, held core late | held 2 at Σ 865.7 from ≈ 1,000–3,000, M(r ≤ 4) ≈ 1,600; ε_th/ε_cold 0.060 and T_c 2.15 × 10⁶ K at step 14,000 | held 4 at Σ 780.0, M(r ≤ 4) ≈ 2,880; 0.058, 9.5 × 10⁵ K at step 9,750 |
| Massive, ending | **none by step 14,250** (570 s cap): shown neutron_star from ≈ 6,000, the star's v_r/vdyn 0.45 (≤ 0.05 needed) | **none by step 10,500** (570 s cap, 4 lives at once): shown neutron_star, ended [] |
| Repro (20 M₀) | core_collapse 1,700, **sink_formed 3,000** (6,360.8 swallowed by 3,062), raw black_hole from 3,000, **shown black_hole from step ≤ 3,100** | sink_formed 2,700, black_hole shown at 3,062 |
| life 01 (m_bh_sb 4,572.394) | neutron_star shown from ≈ 4,751; the star bloats (R 245, M 2,440 → 2,023, M_unb 373 → 586, v_r/vdyn 0.33 → 1.13) and **reads `cloud` from step ≈ 12,751 (to 18,751)**; no ending | neutron_star at step 9,515, compact (R 42.9, M 3,671, v_r/vdyn −0.04); no ending by the cap |

- **What q does:** the hotter, shallower collapse (the Adversarial's) leaves ε_th/ε_cold 0.35 at step 1,000, past η_hold, so the hold
  waits; the held core that forms later is smaller and the envelope stays a large, pulsing, bound halo (life 01 is where it loses the
  stage).
- **What q does not do:** the Massive's «no ending in 570 s» is the control's too (shown neutron_star, ended [] at step 10,500). The
  tree has moved since M0-D153 (4,990) and M0-D155 (4,878); not q's, and not this block's to grade — flagged below.
- Other q in the window, the Massive only (570 s, three at once): −0.016 shown neutron_star from ≈ 6,250, no ending by 12,750 (core
  Σ 904.7, M 1,678, held 2); −0.0187 from ≈ 4,250, none by 16,250 (Σ 778.8, M 1,084, held 1); −0.0228 (outside the bound) none by 10,000.
  No value in the window gets the Massive to an ending in the cap; life 01 was not run at them.

## 3. Pass lines (Verify 2) at −0.0092, read against M0-T130's
| Pass | read |
|---|---|
| Massive neutron_star, held, T_c in [10⁵, 10⁸] K, ε_th/ε_cold ≤ 0.2 | held ✓ (2 cells), T_c 2.15 × 10⁶ K ✓, 0.060 ✓; **the ending not reached in 570 s** (the control's too) |
| the Repro's sink by step 3,062, black_hole | sink 3,000 ✓; shown black_hole by 3,100 (the stage's streak; 38 steps after 3,062) |
| life 01 neutron_star | **✗ — the neutron star is lost to `cloud`** (the control's holds) |
| the Massive core's T_c at step 1,000 beside D's 1.3 × 10⁹ K | 1.005 × 10⁹ K |

## 4. What was left
The value, the three tests that spell it (registry.rs ×2, neutrino.rs, burn.rs ×2) and the contract row are as at the start (−1.14); the
edits were reverted by hand. Nothing in the tree moved. The bound's re-set is the lead's (§0.4): the options measured above — raise
the bound's floor toward what the deep collapse supports (ratio ≲ 0.5), or make the collapse's depth stop depending on N_Fe's heat
(electron-capture pressure loss) so nature's share can be written; the second is a design, not filed here.
