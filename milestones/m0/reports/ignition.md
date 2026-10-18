# Ignition — the constants that give the Sun-like preset a main sequence (M0-D50)

Written by M0-D50, 2026-10-18, on win-laptop (Laser2025-20, NVIDIA GeForce RTX 4080 Laptop GPU, Vulkan). Runs from
2026-10-16 (tree 18b3a13 + M0-D53's uncommitted work, the same source the lead committed as 454209d) and 2026-10-18 (tree
454209d, then 454209d + this task's changes for the case). Raw tables: `logs/M0-D50.runs.log`; the run log:
`logs/M0-D50.log`; the probe scripts: `tasks/M0-D50_probe/`; the task's notes: `tasks/M0-D50.md`.

## Verdict
**Written: the ladder × 0.56 and a_r × 100** (contract §0.5 [M0-D50]; the plan's Superseded mirror). On these values the
Sun-like preset bounces with a mild flash, settles, and burns hydrogen steadily: the centre's X_H falls 0.72473 → 0.72142 →
0.71789 over steps 3,000–5,000, the star stays bound (99.4 % of its mass), it shines what it burns (L_nuc/L 0.70) and only
hydrogen burns (T_max 35.9 against helium's 39.2). The new case `sun_burn` grades exactly that: GO on the RTX 4080 Laptop,
and both plants red. linux-pc's RTX 5090 run is owed (R15).

## The ruling applied
The lead at M0-V18, 2026-10-16, to "How should the set of constants be chosen?": **«Keep white dwarfs, D50 writes
(Recommended)»**. The option as put: D50 prefers sets that leave the cold pressure alone, mainly by lowering the burning
temperatures (the whole ladder moves together); white dwarfs keep their radius of ≈ 4 cells and each life's step count
stays lowest; D50 writes the working set and tells the lead after; the lead is asked only if nothing within the limits
works, or if only a weaker cold pressure works. A set within the limits works and K₁ₑ is untouched, so no question was
needed.

## The values (assets/physics.json)
| key | M0-TC (shipped until now) | written (W2) | bound (contract) |
|---|---|---|---|
| t_h | 100 | **56** | > 0 |
| t_he, t_c, t_ne, t_o, t_si | 140, 196, 274, 300, 420 | **78.4, 109.8, 153.4, 168, 235.2** (× 0.56) | K6's order ✓; T_Si/T_H ≤ 10 (4.2) ✓ |
| t_nu | 166 | **93** (× 0.56) | between T_He and T_C (§2.5.3) ✓ |
| a_r | 4 × 10⁻⁷ | **4 × 10⁻⁵** (× 100) | c_sb > 2 × max_gas_speed (unchanged c_sb) |
| everything else | — | unchanged: K₁ₑ 21.3, K₂ₑ 134, K₁ₙ, K₂ₙ, Q_H 8,700, every A (H_burn 5 × 10⁻⁴), every ν, Σ_N 160, c_sb 200, κ₀ 1, k_cond 0, A_ν, m | — |

## Why the shipped set stalls (M0-V8's measurement, and the mechanism)
On M0-TC's values the preset never lives a main sequence (M0-V8 on the RTX 5090, M0-T63 on the Quadro RTX 4000;
reports/v8.md § The stall). The electrons' cold pressure holds the centre up before it reaches hydrogen's scale: the
bounce reaches T_c 59 (against H_burn's threshold 0.5 T_H = 50), then the core settles at T_c ≈ 29 with X_H,c untouched. And
a_r 4 × 10⁻⁷ lets the cloud shine only 2–4 × 10⁻⁷, so it cannot cool and contract either (τ_KH ~ 10¹² t.u.).

What the runs below show:
- **The cold collapse overshoots.** The bounce reaches T_c ≈ 57–66, about twice the quasi-static ceiling (≈ 29 at d_c 0.6,
  ∝ M/K₁ₑ).
- **A ladder low enough for the settled core also burns at the bounce.** H_burn's Gamow terms (ν 4, and ν 18 at 1.15 T_H)
  make that bounce burn a runaway unless it is slow against τ_dyn. T_H ≈ 56 is the sweet spot: the bounce burns about 20 ×
  faster than the settled core. A lower T_H brings the ν 18 term in; a higher one puts the settled core under threshold.
- **Stronger light (a_r × 100) does two jobs.** It damps the bounce's flash: × 30 still runs away, × 100 does not. It also
  sets L ≈ 2,300, which the burn then matches.

## The runs
Method (M0-T98a's pattern): §2.10's cloud as a `disk` scene at M₀ (a 40, Σc 1, T 4.7324 = the preset's U_th = 0.1|W| on the
grid's own W, the Sun's mix, at rest, 600 × 400, leave), candidates as the scene's `overrides`, so no run needs a
calibration. The centre is read from each dump in f64 (`dumpstat.py`): the densest cell, T by §2.3's EOS, d_c, and M_unb by
§1.4.2's kernel. The chunked runs (100 steps by `--load-dump`) give each chunk's L = Δradiated/Δt and L_nuc =
Δnuclear/Δt. The base run reproduces M0-V8 to the digit at step 1,000 (Σc 7.11, T_c 33.71).

Commands (repo root, Git Bash):
- chunked: `py -3.12 -I milestones/m0/tasks/M0-D50_probe/chunk.py <name> <steps> 100 <k=v…>`
- continuous: `build/target/release/sandbox-reactions.exe headless --scene <scene.json from mk.py> --steps <n> --dump-every 1000 --out <dir> --adapter "RTX 4080"`

Every row ran on the RTX 4080 Laptop.

| run | overrides (ladder × s, a_r, A_H) | the bounce | after | verdict |
|---|---|---|---|---|
| base | shipped | T_c 57.6, no burn | T_c 29, X_H,c 0.7343 | the stall (M0-V8) |
| s50 | × 0.5 | runaway: 2.86 × 10⁷ by step 250, T_max 1,346 | core burnt | ✗ (as K₁ₑ ÷ 4, M0-V8's P2) |
| s56 | × 0.56 | runaway (2.86 × 10⁷ by 300) | burnt | ✗ |
| s56r10 | × 0.56, a_r 4 × 10⁻⁶ | runaway | burnt; a helium flash at 800 | ✗ |
| s56r10a | × 0.56, a_r 4 × 10⁻⁶, A 2.5 × 10⁻⁴ | mild | steady from 1,500: T_c 33.6–34.3, L_nuc/L 0.77–0.87 | W1: works, A's margin < × 1.4 |
| s56r10b / s56r10c | W1 with A 1.25 × 10⁻⁴ / 3.5 × 10⁻⁴ | mild / runaway | L_nuc/L 0.25–0.4 / — | W1 sits on a knife edge |
| s54r10a / s58r10a | W1's ladder × 0.54 / × 0.58 | runaway / mild | — / L_nuc/L 0.35 at 3,000 | ✗ |
| s56r100a | × 0.56, a_r 4 × 10⁻⁵, A 2.5 × 10⁻⁴ | mild | L 2,200–2,600, L_nuc/L ≈ 0.4, d_c rising | too much light for this A |
| **s56r100b (W2)** | **× 0.56, a_r 4 × 10⁻⁵, A 5 × 10⁻⁴** | **mild (T_c 57, X_H,c 0.7346 → 0.7315)** | **steady to 8,000 (below)** | **chosen** |
| s56r100c | W2 with A 1 × 10⁻³ | runaway (2.67 × 10⁷) | burnt, M_unb/M 0.2 | ✗: the `burn-runaway` plant |
| s56r100d | W2 with A 7 × 10⁻⁴ | mild | steady by 3,000, L_nuc/L 0.73 | W2's A margin ≥ × 1.4 |
| s56r30b | × 0.56, a_r 1.2 × 10⁻⁵, A 5 × 10⁻⁴ | runaway | — | light damps the bounce: × 100 holds where × 30 fails |
| s58cont (2026-10-18) | W2's ladder × 0.58 | mild | X_H,c 0.7290 → 0.7264 → 0.7236 (2,000 / 3,000 / 4,000), T_c 35.5, bound | W2's T margin ≥ + 4 % |
| s54cont (2026-10-18) | W2's ladder × 0.54 | mild | X_H,c 0.7258 → 0.7225 → 0.7187, T_c 35.6, bound | W2's T margin ≥ − 4 % |

**W2 as one continuous run** (2026-10-18, tree 454209d, 8,000 steps, 3 m 26 s wall). Chunked and continuous agree to four
digits at every 1,000 steps:

| step | 1,000 | 2,000 | 3,000 | 4,000 | 5,000 | 6,000 | 7,000 | 8,000 |
|---|---|---|---|---|---|---|---|---|
| T_c | 28.93 | 33.31 | 35.08 | 35.63 | 35.86 | 36.09 | 36.31 | 36.49 |
| X_H,c | 0.7299 | 0.7276 | 0.7247 | 0.7214 | 0.7179 | 0.7143 | 0.7105 | 0.7067 |
| M_unb/M | 0.0007 | 0.0039 | 0.0024 | 0.0010 | 0.0004 | 0.0001 | 0.0000 | 0.0000 |

- d_c (the cold pressure's share at the centre) stays 0.60–0.615.
- summary.json shows the `ignition` event at step 769 and `main_sequence` from step 3,000 to the end, under M0-D53's
  0.5 L ≤ L_H ≤ 2 L.
- L_nuc/L over each window (chunked ledger): 0.31 (1,000–2,000), 0.53 (2,000–3,000), 0.67 (3,000–4,000), 0.75
  (4,000–6,000), 0.80 (6,000–8,000). The star is still contracting onto its burn, so the case grades from step 3,000.

## Effects elsewhere (the Adversarial line)
- **The cold pressure, M_ch, white dwarfs: unchanged.** K₁ₑ and K₂ₑ are untouched, so M_ch ≈ 3,970 (1.2 M₀), the cold C/O
  disk's radius ≈ 4.0 cells, and Σ_N's bound are as before.
- **K9's m_ign < m_ch holds by inclusion.** M₀ ignites and M₀ < M_ch, so m_ign ≤ M₀ < m_ch. M0-T68 measures m_ign.
- **Helium's threshold is now 39.2, close to the core.** W2's T_c is 36.5 at step 8,000 and rising about 0.2 per 1,000
  steps. Only hydrogen burns so far (T_max < 0.5 T_He at every dump). Whether the main sequence's own T_c reaches helium's
  threshold before X_H,c falls far is not measured: 8,000 steps are under 1 % of the life.
- **The life (my arithmetic, UNVERIFIED).** X_H,c falls ≈ 3.5 × 10⁻³ per 1,000 steps (≈ 11.4 t.u.) at step 5,000. Linearly,
  X_H,c reaches 0.35 (G-THERMO's and M0-T70's mark) near 1,200 t.u., about 10⁵ steps. At the measured 26 ms a step on the
  RTX 4080 Laptop that is about 45 minutes. This is M0-TJ2's step count and M0-T70's life_sun_sb to measure.
- **G-SQUEEZE's gaps (my arithmetic from the 8,000-step ledger, UNVERIFIED).** τ_KH = |W|/2L ≈ 6.4 × 10⁵ / (2 × 2,380) ≈
  134 t.u. τ_nuc = X_H,c · M_core · Q_H / L ≈ 0.72 × 1,350 (r ≤ 8 cells) × 8,700 / 2,380 ≈ 3,550 t.u. So τ_nuc/τ_KH ≈ 26
  (≥ 10). τ_dyn ≈ 1, so τ_KH/τ_dyn ≥ 10. M0-T98a starts from these values.
- **White dwarfs now cool 100 × faster** (a_r × 100, c_sb unchanged) — the ending's clock shortens.
- **The step's cost: no blow-up seen.** Chunks ran 2–4 s per 100 steps against 2–3 s shipped (wall, including process
  start). The continuous W2 run took 3 m 26 s for 8,000 steps. The Quadro's P5 cost (contract §0.5 [M0-T122]) is owed on
  linux-pc.
- **The Massive and Giant presets: not run.** Their bounce on the lower ladder is unmeasured: owed to the lots that run their
  lives (M0-T97, M0-T99a, lot 15).
- **Tests that pin M0-TC's values.** Grepped, not built here: registry's `physics_initial_values_verbatim`;
  `tests/gpu/burn.rs`'s H_burn oracles, written for T_H 100; docs/agent/physics.md's G-ORDER crossings (H 147 …). The
  claim run measures them: § The claim.

## The case — scope `sun_burn` (crates/sr-engine/tests/gpu/sunburn.rs)
`headless --scene preset:sun` on the physics adapter, in two parts:
1. 3,000 steps straight.
2. 2,000 more, resumed from that run's end dump, with a dump every 1,000.

It is graded in f64 from the dumps at 3,000, 4,000 and 5,000 and from the second run's ledger. The margins were stated
before the case's first run, from the disk runs:
- **Hydrogen burns steadily.** X_H,c falls ≥ 0.002 over the window and ≥ 0.001 in each 1,000 steps, and stays ≥ 0.5.
  Measured: 0.00684 over the window, 0.00331 and 0.00353 per 1,000 steps.
- **The star stays bound.** The bound gas holds ≥ 95 % of the preset's mass at each dump, and no sink forms. Measured:
  0.9949, 0.9946, 0.9943.
- **It shines what it burns.** 0.5 L ≤ L_nuc ≤ 2 L over the window. Measured: L 2,377.6, L_nuc 1,663.5, ratio 0.6997.
- **Only hydrogen burns.** T_max < 0.5 T_He at each dump. Measured: 35.08, 35.62, 35.85 against 39.2.
- **The readings' own arithmetic** on hand-made planes (NOT PROVEN on a star).

Plants:
- `stall-constants`: physics.json's ladder, T_ν and a_r back to M0-TC's. It turns 3 of the 4 cases red.
- `burn-runaway`: A_H 1 × 10⁻³.

The scope's paths are its own test, its plants, the three registries and tests/gpu.sh — not `crates/**`, because a run
costs ≈ 2.5 min on the RTX 4080 Laptop. A code change is graded by the complete loop at each V.

## The claim — what the new constants turn red elsewhere (filed as M0-D55)
`py -3.12 tools/pb/verify.py --changed --task M0-D50`, win-laptop, 2026-10-18, against the task's start
(2026-10-16T16:37). **NO-GO**: 924 passed, 29 failed, 38 of 46 scopes GO, 276 s. Logs: `logs/M0-D50.<scope>.log`.
Every red is a consumer of the shared constants. None is in the preset's own run, which printed no `SR-WARN` in 8,000
steps.

**26 hand oracles pin M0-TC's values** (cause measured). Each test reads the shipped physics.json, but its oracle was
written for T_H 100, T_He 140, T_Ne 274 or T_ν 166:
- `registry` (5): the verbatim list, the reaction temperatures, and the ladder's edge cases built on the old neighbours.
- `rates` (5): T_thr, the two-term crossing 119.5, f and ω by hand, and the Gamow slope at 140.
- `burn_cell` (6): the closed forms at T 100, T_He and T_Ne.
- `burn_order` (2): H's power by hand at T 82.5.
- `neutrino` (8): the closed forms at T_ν 166.

**3 cases change under a_r × 100** (cause not measured):
- `heat_limiter`'s thin-gas case: P5's pass 0 reaches the 64-pass ceiling unconverged.
- `heat_stiff`'s (e): one pass keeps 2.66 × its refinement's heat, against the bound 2.2.
- `timing`'s long-frame case: 267 whole steps against 300.

M0-D55 measures each cause and fixes each test. An oracle of a law pins the constants it was written for; never a looser
tolerance. If the solver itself cannot hold at the shipped a_r on gas the game can make, M0-D55 brings the lead that
question.
