# M0-R13 — The cold remnant: why a fresh neutron star reads 10 K, and what it should read

Box: laserax-ai-server, NVIDIA GeForce RTX 4090 (Vulkan, driver 595.99.02), R18. The twin crops ran on that box's CPU. Tree: the
one M0-D152 left there (255ec37 + the rsync'd working tree; the build reported nothing changed). Dates: 2026-11-03, two sessions,
both model=claude-opus-5-5 level=high (`rung_record.py now`). Raw lines: logs/M0-R13.log. Scratch sources:
logs/M0-R13.scratch.{r13core.rs, r13twin.rs, twin_r13.diff, cands.py}. Detail: tasks/M0-R13.md.

The lives, dumps and candidate binaries are on the server in `~/workspace/sandbox_reactions/build/r13` (16 GB, git-ignored). The
lead moved them there from `/tmp/r13`, and they are the lead's to remove.

## Problem
The lead, 2026-11-01, at M0-R12: « is it normal that we cant see the remnant solid core? »

- **What today's tree does.** The Massive preset collapses at step 900 and goes supernova at 920. It shows neutron_star from 3,474
  and ends neutron_star at step 29,208 (t 279.9, mass 4,118.1, hold 257.4 t.u.).
- **What the core reads.** Its core (Σ ≥ 10) reads **T_floor = 10 K** at most dumps from step 920 to the ending, and flickers up to
  10³–10⁴ K. Its pressure is the cold term alone: ε_th/ε_cold is 10⁻⁴–10⁻³ while at the floor.
- **What the views show.** Heat draws the core black and Glow draws it as dense haze. No display setting from M0-R12 lights 10 K.
- **The stage text disagrees.** It says «Neutron star» (`stage.neutron_star`), and the fate reads «supernova, then a neutron star»
  (`fate.supernova`). The inspector beside it reads a molecular cloud's 10 K.
- **The core is never held.** It keeps X_Fe 0.36 and X_n 0.64 from step ≈ 1,000 on, so §2.7 [M0-D71f]'s «a cell holding any iron is
  never held» excludes every cell.
- **It expands instead of staying compact.** Σ_c goes 1,020 (step 1,000, R₉₀ 2.0) → 214 (2,000) → 82 (10,000) → 45 (the ending,
  R₉₀ ≈ 16).

**What nature does** (sources):
- A neutron star is born at an interior **~10¹¹ K**: «several times 10¹¹ K», entropy ~2 k_B per nucleon (S1, S2).
- It cools by neutrinos to **10⁹–10¹⁰ K within about a day** (S2, S3).
- Its surface stays **~10⁶ K** for 10³–10⁶ years, about 3 × 10⁶ K for the first century (S2, S4).
- Its Fermi temperature is **T_F ≈ 10¹² K**, so T ≪ T_F: the star is degenerate, and thermal pressure is a correction of order
  (T/T_F)² (S5).

A real fresh neutron star is therefore both very hot and degenerate. The sandbox's is neither hot nor compact.

## Candidates

| | change | physics | cost |
|---|---|---|---|
| A | none | the core reads T_floor, as §1.10.2 translates the state | 0. Unphysical: a dense remnant reads as cold as a molecular cloud |
| B | P8 floor ε_th ≥ 0.1 ε_cold where Σ ≥ 10 | a heat floor for degenerate matter (P8 pays it) | one floors.wgsl line + twin. The core reads 10³–10⁴ K, red in Glow (the lead at M0-R12: «I wouldnt like for something like a neutron star to show up as red») |
| B9 | P8 floor at T_sb 120.73 (≙ 10⁹ K) where Σ ≥ 10 and X_n > 0 | a heat floor at a real neutron star's kelvins | as B. The core stops being degenerate (the Adversarial) |
| C | a_nu 10⁻³ → 10⁻⁵ | slower neutrino cooling | one physics.json value → recalibration. It changes the burning stages, since neutrinos are their sink, not the remnant's |
| D | N_Fe q_ratio −1.14 → −0.114 | iron dissociation at nature's share of the collapse energy (Evidence 2) | one physics.json value → recalibration |
| E | display-only: a remnant core's shown T from its degenerate state (μ ε_cold through §1.10.2) or a cooling clock | a reading, not the state | a contract line against §1.10.2's «Every displayed temperature uses it» and the goal's «never a script». Inspector, Heat/Glow shaders and readouts; not designed |
| F | the held core extended: hold a cell once X_n ≥ 0.5, iron or not | P4's pump stopped on the degenerate core (M0-D71's cure, applied to the neutron core) | floors.wgsl only (no twin built). The 20 M₀ black hole breaks (Evidence 4) |
| G | F gated so it cannot freeze a core heading for a black hole — held only while the cell's object is under m_tov_sb (4,853.2), released past it | as F, with the black-hole path left free | an object mask into P8 (the observer's §1.9.2 objects, per frame), the twin, three lives. **NOT RUN** — a design |

## Evidence
All runs are on the RTX 4090. Up to five lives ran at once, so wall times do not compare.

**1. Where the core's heat goes, by pass.** These numbers are from a 128 × 128 twin crop centred on the core, with the R13 tally.
The crop has crop gravity only, but it reproduces the GPU ledger.

| window | P4 | P5 | P6 | P8 (floor) |
|---|---|---|---|---|
| s_900 + 120 steps (the collapse) | +2.23 × 10⁶ | −8.2 × 10⁵ | −2.77 × 10⁷ | +2.64 × 10⁷ |
| s_1000 + 800 steps (the free core; cut at 560 s) | −6.1 × 10⁶ | −1.9 × 10⁵ | ≈ 0 after 100 steps | +5.8 × 10⁶ |
| s_20250 + 2,000 steps (late) | −1.75 × 10⁵ | −9.8 × 10³ | 0 | +1.45 × 10⁵ |

- At the collapse, P6's neutrinos are 2.9 × 10⁴ of the P6 total; the rest is N_Fe's q = −1.14 Q_H. The GPU ledger over steps 900–1,000
  reads nuclear −2.79 × 10⁷ and floor +2.56 × 10⁷, which matches.
- **The cause is two-fold.**
  1. At the collapse, N_Fe's endothermic q takes about 12× the heat compression gives the core, and P8 refills it to T_floor.
  2. After the collapse, P4's non-balance pump on the free core draws its heat every step, and P8 refills it at the floor. This is
     M0-D71's cause, back because the core holds iron.
- **Neutrinos are not the remnant's sink.** neutrino_lost is 5.49 × 10⁷ over the life, all of it before step 500; it is 4.9 × 10³
  from step 920 to the ending.

**2. Nature's ratio is the other way round.**
- The collapse releases a binding energy of about **3 × 10⁵³ erg**, roughly GM²/2R for 1.4 M☉ at 10 km (S6, S7).
- Dissociating iron costs **~8–9 MeV per nucleon**, about 2 × 10⁵² erg for a 1.4 M☉ core: **≈ 7 %** of the binding (S6, S8).
- So in nature the release is **≈ 15× the cost**. In the sandbox the cost is ≈ 12× the release, a factor of ≈ 180 the wrong way.
- D's ÷10 is the step toward nature's ratio. Nature then sends ~99 % of the release out as neutrinos (S7) and still leaves the star
  at 10¹¹ K.

**3. The ceiling: the sandbox cannot be both degenerate and at real kelvins.**
- The kelvin map (§1.10.2, read by r13core `R13_SB`): 10⁶ K ↔ T_sb 32.4, 10⁹ K ↔ 120.7, 10¹⁰ K ↔ 462, 10¹¹ K ↔ 1,881.
- The late core's ε_cold is ≈ 38 per unit mass (Σ 45), with 1/μ = 0.81.
- Reading 10⁹ K would need ε_th ≈ 98 ≈ 2.6 ε_cold, which is past degeneracy.
- The squeezed ladder puts the sandbox remnant's Fermi temperature (μ ε_cold ≈ 47 sandbox) at **≈ 10⁶ K**, against nature's 10¹² K.
- So a degenerate sandbox core reads at most ~10⁶–10⁷ K. That is a real neutron star's surface, not its interior. B9 measures the
  other branch (row 4).

**4. Each candidate on the Massive life to its ending.**
- The core columns are r13core over the dumps (every 250 steps).
- The ledger columns run from s_1000 to the last dump, from observers' `ledger.last` (session 2).

| cand | ending | core late (Σ_c, R₉₀, T_c, ε_th/ε_cold) | floor_added | radiated | floor / radiated |
|---|---|---|---|---|---|
| today | neutron_star, step 29,208, 4,118.1 | Σ 45, R₉₀ 16, 10 K (flickers ≤ 10⁴ K), 10⁻⁴ | 9.72 × 10⁶ | 2.47 × 10⁶ | **3.93** |
| B | neutron_star, step 41,224, 3,121 (hold 395.5) | Σ 33–108, R₉₀ 15–17, 1.5–15 × 10³ K, 0.10–0.15 — **red in Glow** | 1.27 × 10⁷ | 3.34 × 10⁶ | 3.80 |
| B9 | **none by step 60,000** (exit 2) | the core disperses: Σ_c ≈ 10, ε_th/ε_cold 3–6, 0–6 cells ≥ 10, 223.2 left within r 8 | 3.04 × 10⁷ | 1.98 × 10⁷ | 1.54 |
| C | **black_hole** (a sink at step 2,474, ending t 26.9) | the life moved before the collapse (X_Fe,c 1.0 at step 1,000, no neutronization) | 1.03 × 10⁷ | 4.86 × 10⁶ | 2.12 |
| D | neutron_star, step 41,750, 3,345 | **1.3 × 10⁹ K at step 1,000** (X_n,c 0.20), back to 10 K by step 2,000 (P4's pump); late 10–3,000 K | 1.22 × 10⁷ | 3.97 × 10⁶ | 3.07 |
| F | neutron_star, **step 14,394, t 79.1**, 3,919 (hold 63.7) | 4 held cells at Σ 783.1 from ≈ 1,000 on, compact. T_c 4 × 10⁵ → 3 × 10⁷ (step 5,000) → **2.1 × 10⁶ K at the ending**, ε_th/ε_cold 0.07 (degenerate) | 8.51 × 10⁶ | 5.10 × 10⁶ | **1.67** |

- **B9's stage trail:** collapsing 65 · iron_core 330 · core_collapse 910 · supernova 926 · **he_shell from 4,176** (261 mass). The
  scattered remnant reads as a helium-shell star, which is the Adversarial's failure in full.
- **F's ending dump** (s_14394, read in session 2): T_c 2.098 × 10⁶ K, 5 cells ≥ 10, Hth 1.43 × 10⁵ against Ecold 2.10 × 10⁶. It is
  degenerate, compact and hot.
- **F on M0-D152's Repro** (20 M₀, from m_bh_sb-02's s_1362, 1,700 steps, 74.8 s):
  - core_collapse at step 1,700 and **no sink** by step 3,062; it shows neutron_star (61,874.5).
  - Today's tree forms a sink at step 2,600 from a free ring clump of 5,165.7 (1.06 m_tov_sb) and shows black_hole (M0-D152).
  - Floor +1.54 × 10⁸ against nuclear −1.42 × 10⁸.
  - **F breaks the 20 M₀ black hole as it stands**, as M0-D71f's iron hold did.
- **The floor after the supernova.** It pays more than the light in every candidate (M0-D152's flag, carried here). F cuts the
  share the most (3.93 → 1.67), because it stops the pump on the core. The rest is the free envelope's HLL residual, which M0-R11
  placed beside the core.

**5. What it should read.**
- Nature's fresh neutron star is ~10¹¹ K inside, ~10⁹ K within a day, and ~10⁶ K at its surface (S1–S4).
- The sandbox can carry a degenerate core at ~10⁶–10⁷ K (Evidence 3). That is the surface's kelvins: hot enough for Glow to draw it
  blue-white and Heat to draw it bright. F measured exactly that band, 4 × 10⁵ – 3 × 10⁷ K.
- 10⁹ K or more is reachable only by a display reading (E), or by a heat that ends degeneracy (B9, measured: no remnant).

Sources (accessed 2026-11-03):
- S1 Lattimer & Prakash, «Neutron star cooling» — https://arxiv.org/pdf/astro-ph/0402143
- S2 Yakovlev & Pethick, «Neutron star cooling» (ARA&A 2004) — https://ar5iv.arxiv.org/html/astro-ph/0306143
- S3 Page et al., «Evolution of a neutron star from its birth to old age» — https://arxiv.org/pdf/astro-ph/0012136
- S4 Page, Geppert & Weber, «Theory of cooling neutron stars versus observations» — https://arxiv.org/pdf/0710.2047
- S5 «Degenerate matter» — https://en.wikipedia.org/wiki/Degenerate_matter (T_F ≈ 10¹² K for 1.4 M☉ at 10 km)
- S6 Burrows, «Core-collapse supernovae: reflections and directions» — https://arxiv.org/pdf/1211.1378
- S7 Mackay lectures, Cambridge, «SNRs, GRBs» — https://people.ast.cam.ac.uk/~vasily/Lectures/Introduction_Astrophysics_Mackay/Mod6_SNRs-GRBs.pdf
- S8 Janka, «Explosion energies for core-collapse supernovae I» — https://arxiv.org/pdf/2007.06087

## Recommendation
**G: hold the neutron core, gated by m_tov_sb.** P8 holds a cell once X_n ≥ 0.5, iron or not, while its object's mass is under
m_tov_sb, and releases it past m_tov_sb. Everything else in §2.7 [M0-TG2]/[M0-D71f] is unchanged: walls only, η_hold 0.1, σ_hold 10.

Why G:
- It fixes the measured cause, P4's pump on the free core, with the cure the project already ruled and built (M0-R11, M0-TG2).
- F measured the gain: the core stays compact and degenerate and reads 10⁶–10⁷ K, which is a real neutron star's surface. Glow
  draws it white rather than red, and the floor's share of the light drops 3.93 → 1.67.
- The gate keeps the black-hole path that F broke. The Massive remnant (3,919–4,118) sits under m_tov_sb, and the 20 M₀ sink formed
  from a 5,165.7 clump above it. **This is my reading; G itself is not run.** Its build measures it on the three lives: the Massive
  life to its ending (≈ 250 s), M0-D152's 20 M₀ Repro (≈ 75 s, a sink by step ≈ 2,600) and m_bh_sb's 4,572 life 01 (≈ 280 s) — the
  lead's time.

The others:
- **Rejected on measurement:** B9 (no remnant), C (a black hole from the wrong cause), D alone (hot, then 10 K by step 2,000).
- **Rejected by the lead's word:** B (red).
- **E is the fallback** if G cannot hold the 20 M₀ path. It is the only way to show 10⁹–10¹¹ K, and it needs a contract line against
  §1.10.2.
- **D is a separate, physical correction**, worth keeping. Nature's release is ≈ 15× the iron cost (Evidence 2). It is not needed
  for the reading once G holds, it moves the life (ending 29,208 → 41,750), and it costs a recalibration. I recommend not now.

Cost:
- G changes the life: F's ending moved 29,208 → 14,394. So the shipped calibration's endings are re-measured, a recalibration of
  ≈ 2 h 17 min on the RTX 4090 (M0-V9's O7), the lead's.
- It shares that recalibration with M0-D71g's constants move.

## Ruling
The lead at M0-R13, 2026-11-03, question tool, verbatim:

- **The fix:** «G: gated hold (Recommended)». The option was shown as: "Hold the neutron core (X_n ≥ 0.5, iron or not) only while
  its object is under m_tov_sb (4,853), so heavy cores still make black holes. Physics, not a script. Build + 3 lives (~10 min,
  your time), then a ~2 h 17 min recalibration (yours)."
- **The iron cost:** «File it with the fix». The option was shown as: "Add a block to bring the cost to nature's ratio, measured
  together with the chosen fix, so one recalibration covers both."
- **Not picked:** «E: display-only reading», «A: keep it as is» and «Not now (Recommended)».

**The contract**, amended in place and tagged [M0-R13]:
- §0.5 gains the [M0-R13] line.
- §2.7's hold paragraph: the neutron-rich branch, and its release at m_tov_sb.
- §2.3.4's wall line: «none under X_n 0.5, and none whose object reaches m_tov_sb».
- §2.5.1's N_Fe row: q_ratio −1.14 until M0-T131 writes the value inside the bound — the collapse's P4 heat ÷ N_Fe's in [7.5, 30].

## Blocks filed
- **M0-T130** — the neutron core held (G), on the twin and in floors.wgsl. Placed right after this block, before M0-D91.
  Opus 5.5, high (usual): a design the spec leaves open, since P8 must learn a cell's object mass. Its refusal arm covers the
  gate's risk: the Massive object is 6,670 at the supernova, over m_tov_sb with its envelope.
- **M0-T131** — N_Fe's cost at nature's share (D). Placed after M0-D71g, inside the stale-calibration window M0-T98a opens, so
  M0-V14's recalibration covers both blocks. Sonnet 5.5, high (rule 3).
- **Flagged to M0-V14:** the recalibration covers M0-T131's physics hash and the endings M0-T130 moves.
