# Cold ceiling — what m_ch_sb, m_tov_sb and ns_sigma_max_sb can mean on a one-cell grid, and what G-CORE grades (M0-R4)

M0-R4 · BUILD (research) · linux-pc (`laserax-ai`), 2026-10-19 · model=claude-opus-5-5 level=high (rung_record.py now).
Every run was made in the scratch copy `/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/307c9bc4-f704-4bcf-8637-d329238076cd/scratchpad/r4`
(`tools/pb/scratch_copy.sh`). There, M0-T66's parked draft was wired in and given four probes (`calibrate r4-twin | r4-atmo | r4-scene`, and
`SR_R4_LIMITER` on the twin's slope limiter). The stdlib solvers are in `…/scratchpad/r4tools/`. Nothing of it is in the tree. Logs:
logs/M0-R4.sheet.log (continuum solvers), M0-R4.twin.log (CPU f64 twin), M0-R4.atmo.log (fixed-g atmosphere) and M0-R4.gpu.log (headless,
RTX 5090).

**In one paragraph.** On this grid the cold ceiling is a sub-cell event by construction. For §2.3.2's blended law the equilibrium radius
depends on M/M_ch alone, R ≈ R₀(1 − √(M/M_ch)) with R₀ = 3K₁′/4G ≈ 4 cells. A disk's R₉₀ falls under one cell from M ≈ 0.51 M_ch, so no grid
run can tell 0.95 M_ch from 3 M_ch. Measured: nothing collapses and no sink forms at any mass up to 12,000. Separately, every under-resolved
cold equilibrium swells. That swelling is the scheme's: MUSCL-Hancock with gravity as a source term is not well-balanced. A static atmosphere
under a fixed g drives flows first order in Δx/H, and their dissipation heats the gas second order. Radiation does not take that heat out.
The swelling converges away with resolution, and a disk with R₉₀ ≳ 15 cells holds within 3 % over 10 τ_dyn on the GPU and the twin alike.
The sheet's real ceiling for its own equilibrium profile is **1.4331 K₂′²/G²**, not the uniform disk's 1.767. That is 3,216.5 for C/O and
4,854 for neutron matter (ratio 1.509), K₁-independent and computable in the continuum from the EoS the game runs. Recommendation: (A), that
continuum computation as items 1–2; G-CORE graded on it plus one resolved grid hold; ns_sigma_max_sb = m_tov_sb per cell; Σ_BH raised above
it. A heat-on runaway that empties the world unbooked is filed as M0-D58.

## Evidence

Sources (each line: what it says, then where):
- **S1** · Zingale et al. 2002, *Mapping Initial Hydrostatic Models in Godunov Codes* (ApJS 143, 539): it maps a model "while maintaining hydrostatic equilibrium", studying "the effects of spatial resolution, boundary conditions" and "the treatment of the gravitational source terms in the hydrodynamics solver", to find the mapping "that yields the lowest ambient velocities" — https://arxiv.org/abs/astro-ph/0208031 (accessed 2026-10-19)
- **S2** · Almgren et al. 2010, *CASTRO I* §VIII.4 "Stationary Star Gravity": on "a nearly-Chandrasekhar mass, carbon-oxygen white dwarf", "small motions can be driven by the inexact cancellation of" the pressure gradient and gravity, worse for a sphere on a Cartesian grid; run at 192³, 384³ and 768³, with the peak velocity under 1 % of the central sound speed at the finest — https://ar5iv.labs.arxiv.org/html/1005.0114 (accessed 2026-10-19)
- **S3** · Grosheintz-Laval & Käppeli 2018 (after Käppeli & Mishra 2014, J. Comput. Phys. 259, 199–219): "A high-order well-balanced scheme for the Euler equations with gravitation", "based on a novel local hydrostatic reconstruction", which "in combination with any standard high-order accurate reconstruction procedure" keeps the discrete equilibrium — https://arxiv.org/abs/1807.04074 (accessed 2026-10-19)
- **S4** · Edelmann et al. 2021 (A&A 652, A53): "discretization errors on grids of reasonable size can lead to spurious flows" "orders of magnitude faster than the physical flow"; the well-balanced methods "can accurately maintain hydrostatic solutions" — https://arxiv.org/abs/2102.13111 (accessed 2026-10-19)
- **S5** · Font et al. 2002, *Three-dimensional general relativistic hydrodynamics II* (PRD 65, 084024): tests "single non-rotating stars in stable equilibrium", stars on "the unstable branch … migrating to the stable branch", and "non-rotating stars undergoing gravitational collapse to a black hole", with HRSC schemes "based on Riemann solvers and slope limiters" — https://arxiv.org/abs/gr-qc/0110047 (accessed 2026-10-19)
- **S6** · Chandrasekhar limit: the ultra-relativistic polytrope of index 3 "has a total mass, M_limit, depending only on K₂"; "the model radius still decreases with mass, but becomes zero at M_limit"; the limit is reached by "taking the limit of large central density" — https://en.wikipedia.org/wiki/Chandrasekhar_limit (accessed 2026-10-19)
- **S7** · The potential of an axisymmetric thin ring is closed-form in the complete elliptic integral K(k), k² = 4ϖa/[(ϖ + a)² + z²] (MacMillan; Kellogg 1929; Cohl & Tohline 1999) — https://tohline.education/SelfGravitatingFluids/index.php/2DStructure/ToroidalCoordinates (accessed 2026-10-19)
- **S8** · The self-consistent-field method: the density and the potential "solving the Poisson equation as well as the equilibrium condition, alternatively and iteratively"; Hachisu's (1986) version builds rotating and non-rotating polytropes — https://ar5iv.labs.arxiv.org/html/1709.00039 (accessed 2026-10-19)
- **S9** · SM-E4 (Chavanis, *White dwarf stars in D dimensions*) re-read, not re-accessed: polytropes are stable iff γ > 2(D−1)/D, and "For D=2, the radius is independent on mass" — https://ar5iv.arxiv.org/html/astro-ph/0604012 (accessed 2026-10-08)

What the sources settle: grid codes hold a hydrostatic star only to their discretisation's truncation error. Unbalanced schemes drive
spurious flows that shrink with resolution (S1, S2, S4), and well-balanced reconstructions remove them (S3, S4). A degenerate ceiling is a
**continuum** limit reached as Σ_c → ∞ with R → 0 (S6). Codes test the stable side (a held star, S2, S5) and the unstable side (a collapse,
S5) on stars that are many cells wide. None measures a ceiling on a star a few cells across. No source treats the razor-thin sheet: its
ceiling below is my computation (M3), validated against the Maclaurin disk.

Measurements (box linux-pc `laserax-ai`; adapter named per line; scratch paths abbreviated `…/` = the scratchpad above):

**M1** · M0-T66's numbers, re-read (tasks/M0-T66.md, logs/M0-T66.log; RTX 5090, Vulkan; `calibrate --only m_ch_sb` and the `SR_CAL_*`
probes). §2.11's recipe as written (Maclaurin a = 12, R₉₀ within 3 % over 10 τ_dyn) found no bracket from 3,973 down to 30.6 (exit 6).
Every a = 12 disk at 30–12,000 falls to ~0.28 R₀ and swells to ~2.8 R₀. The SCF equilibrium on the grid's kernel gives R₉₀ 3.56 (M = 30),
2.80 (300), 1.91 (1,000) and a 2 × 2 block from 2,500. From it the GPU swells at every mass: 0.70 (30), 1.02 (300), 4.25 (1,000), 8.4
(2,500), and 8 % at M = 100 with k1e × 3.

**M2** · The equilibrium radius against M (`python3 r4tools/sheet_scf.py blend 5.325 47.37615 30 … 3200`, `r4tools/msigc.py 5.325 47.37615 1.5 1 … 1e9`;
CPU, stdlib Python; logs/M0-R4.sheet.log). The uniform-disk virial of the blended law, 2πR²P(Y_eM/πR²) = (8/3π)GM²/R, solves exactly to
**R = R₀(1 − √(M/M_ch,u))**, with R₀ = 3K₁′/4G = 3.994 and M_ch,u = (9π/16)K₂′²/G² = 3,966 (my arithmetic). R/R₀ therefore depends on M/M_ch
alone, and K₁ sets only the scale. The continuum equilibrium's R₉₀ (cells), with the uniform disk's R₉₀ = √0.9 R alongside:

| M | continuum R₉₀ | uniform R₉₀ | grid SCF R₉₀ (M1) |
|---|---|---|---|
| 30 | 3.34 | 3.46 | 3.56 |
| 300 | 2.59 | 2.75 | 2.80 |
| 1,000 | 1.67 | 1.89 | 1.91 |
| 1,650 (Σ_c 10³) | 1.09 | — | — |
| 2,000 | 0.81 | 1.10 | 2 × 2 block |
| 2,500 | 0.46 | 0.78 | 1.075 (2 × 2) |
| 3,000 | 0.15 | 0.49 | 1.075 |

The grid's own floor sits at M ≈ 2,000–2,500 (0.62–0.78 of the true ceiling, M3). From there the grid SCF stops being a disk.

**M3** · The sheet's ceiling for its own equilibrium profile (`python3 r4tools/sheet_scf.py pure 1.5 100` / `200`; `msigc.py`; logs/M0-R4.sheet.log).
The solver is an axisymmetric thin-disk SCF using the ring kernel −G∫Σ(r′)4r′/(r + r′)K(k)dr′ (S7; K by the AGM; each annulus by
Gauss–Legendre, split at the field point by r′ = r ± t²). Kernel check: the Maclaurin disk's potential within **6.4 × 10⁻⁵** (r ≤ 0.95a),
and γ = 3's SCF equilibrium is the Maclaurin profile within **2.6 × 10⁻⁴** (the sheet's γ = 3 disk, S9's family). For the pure Π = KΣ^{3/2}
law (Hachisu, edge and Σ_max fixed) the mass is unique, **M G²/K² = 1.43322** (100 annuli) and **1.43307** (200), with virial 2∫Π dA/|W| =
0.99998. The uniform disk's 9π/16 = 1.767 is therefore 23 % high: a uniform disk is not an equilibrium. On the blended law, M(Σ_c) rises
monotonically to it at every decade (S6's shape). C/O (K₁′ 5.325, K₂′ 47.376): 22.4 (Σ_c 1), 684 (10²), 2,497 (10⁴), 3,124.7 (10⁶),
3,186.8 (10⁷), 3,207.0 (10⁸), 3,213.5 (10⁹) → **3,216.5**. Neutron matter (2.67, 58.2): 4,530.7 (10⁶), 4,746.0 (10⁷), 4,819.3 (10⁸)
→ **4,854.2**. The ratio is **1.509**, inside K9's [1.4, 2.0]. The deficit falls as Σ_c^{−1/2}, so the Richardson limit
M₈ + (M₈ − M₇)/(√10 − 1) reads 3,216.42 (C/O) and 4,853.17 (n), against the eigen-mass at 3 × 10⁻⁵ and 2 × 10⁻⁴. The limit from
decades 6–7 agrees within 0.03 % and 0.16 %.

**M4** · What breaks it (the same scan, the plants as edits to the law; logs/M0-R4.sheet.log). With the relativistic exponent 3/2 → 1.6
(G-CORE's `cold-exponent`) there is **no ceiling**: M = 10,911 (Σ_c 10⁴), 40,814 (10⁶), 110,391 (10⁸), 280,670 (10¹⁰). With K₂′ = K₂Y_e
in place of K₂Y_e^{3/2} (67.0) the ceiling is **6,425**, ×2.0. A wrong K₂ moves the value as K₂²; a wrong exponent removes it.

**M5** · The SCF equilibrium on the CPU f64 twin, gravity + hydro, bounce, 64 × 64, 10 τ_dyn, over the slope limiter
(`SR_R4_LIMITER=<mc|central|first> sandbox-reactions calibrate r4-twin co <M> 10`; CPU; logs/M0-R4.twin.log). Largest |R₉₀/R₀ − 1|:

| M | twin, MC (shipped) | GPU, M1 | twin, no limiter (central) | twin, first order |
|---|---|---|---|---|
| 30 | 0.696 | 0.70 | non-finite E, step 1,215 | 6.55 |
| 300 | 1.019 | 1.02 | non-finite E, step 622 | 8.23 |
| 1,000 | 4.28 | 4.25 | non-finite E, step 554 | 10.7 |
| 2,500 | 8.42 | 8.4 | non-finite E, step 702 | 12.6 |

The twin is the GPU within 3 % of the deviation, so the swelling is the scheme's, not f32's. The more dissipative scheme swells 3–10× more.
At M = 300 the thermal energy above T_floor reaches 5.9 % of |W| by 0.55 τ_dyn (9 steps), against a cold energy of 0.55 |W|, and 21 % by
10 τ_dyn. The disk never contracts (min R/R₀ = 1.000 in every run).

**M6** · Resolution, at fixed M/M_ch: K₁ (both K₁ₑ and K₁ₙ) scaled, which widens the disk and leaves the ceiling where it is
(`calibrate r4-twin co <M> 10 <k1x>`; twin, CPU; and `r4tools/gpu_run.sh co 100 gravity,hydro <10τ> 16 <tag> <k1x>`; headless, RTX 5090;
logs/M0-R4.twin.log, M0-R4.gpu.log):

| M, K₁ × | R₉₀ (cells) | Eth⁺/\|W\| at 1 τ | twin dev, 10 τ | GPU dev, 10 τ |
|---|---|---|---|---|
| 100, × 1 | 3.25 | 5.4 × 10⁻² | 0.682 | — |
| 100, × 2 | 6.34 | 8.7 × 10⁻³ | 0.239 | — |
| 100, × 3 | 9.41 | 2.7 × 10⁻³ | 0.093 | 0.092 |
| 100, × 5 | 15.65 | 5.6 × 10⁻⁴ | **0.0259** | **0.0257** |
| 100, × 6 | 18.81 | — | **0.0165** | **0.0162** |
| 1,000, × 1 / 2 / 3 / 5 | 1.91 / 3.65 / 5.32 / 8.67 | 0.20 / 0.053 / 0.019 / 0.0045 | 4.28 / 0.87 / 0.39 / 0.20 | — |

The heating falls about as (Δx/R)³ and the deviation about as (Δx/R)². A 3 % hold over 10 τ_dyn needs R₉₀ ≳ 15 cells. At 0.95 M_ch that
means R₀ ≈ 15 / 0.02 ≈ 750 cells (M2's law), past §0.5 [M0-V4]'s 1,024-cell world with gravity.

**M7** · The scheme's hydrostatic balance alone: a cold C/O atmosphere under a **fixed**, uniform g (no self-gravity), 64 wide, bounce, Σ 200
at the bottom, balanced on the cell centres, L cells deep, 20 sound crossings (`calibrate r4-atmo <L> 20`; twin hydro + floors, CPU;
logs/M0-R4.atmo.log):

| L (H_mid, cells) | max\|v\|/c_b at 1 crossing | Eth⁺/U_cold at 1 crossing | at 20 crossings | ΔΣ L1/M at 20 |
|---|---|---|---|---|
| 4 (1.8) | 0.32 | 0.32 | 4.0 | 1.00 |
| 8 (3.4) | 0.12 | 0.096 | 1.18 | 0.58 |
| 16 (6.6) | 0.067 | 0.025 | 0.35 | 0.26 |
| 32 (12.9) | 0.034 | 0.0064 | 0.10 | 0.088 |
| 64 (25.4) | 0.017 | 0.0016 | 0.030 | 0.028 |

The spurious speed is first order in Δx/H and the heating second order. This is S1/S2/S4's imbalance, present without any self-gravity.

**M8** · Does radiation take the heat out? The same equilibria, heat off against heat on, otherwise as M5
(`r4tools/gpu_run.sh co <M> gravity,hydro[,heat] <10τ> <every> <tag> [k1x]`; headless, RTX 5090; logs/M0-R4.gpu.log; deviation read against
the first dump, step 2–8). Off / on: M = 300, 0.967 / 0.896. M = 1,000, 4.08 / 3.88. M = 100 at K₁ × 3, 0.0923 / 0.0923. **No** effect that
matters: the heating is made in a few steps (M5), and light leaves a 2–9-cell disk of κΣ ≫ 1 per cell over many τ_dyn.

**M9** · Above the ceiling, heat and sinks on (P2 P3 P4 P5 P8; `r4tools/gpu_run.sh <co|n> <M> gravity,hydro,heat,sinks 0.5 20 <tag>`;
headless, RTX 5090; logs/M0-R4.gpu.log). No run forms a sink (summary.json: no `sink_formed`), and none contracts:

| matter, M | start | outcome to sim time 0.5 |
|---|---|---|
| C/O 2,500 | 2 × 2 block, Σ 625 | swells, R₉₀ × 5.2 |
| C/O 5,000 / 8,000 / 12,000 | 2 × 2 (Σ 1,250 / 2,000), 1 cell (Σ 12,000) | **E runaway → NaN → every cell gone, exit 0** (M0-D58) |
| C/O 8,000, heat off | 2 × 2, Σ 2,000 | swells × 7.5, Σ_max 562, no sink |
| n 2,000 / 3,500 / 5,000 | 2 × 2 | swells × 2.9 / × 2.9 / × 4.9 |
| n 8,000 | 2 × 2 | runaway → NaN, as above |

The largest Σ any surviving run reached is 1,021, a quarter of Σ_BH = 4,000. The runaway (C/O 5,000, heat on, a dump every step): E in the
densest cell reads 2.2 × 10⁶ (step 1), 1.8 × 10⁹ (9), 2.9 × 10²³ (11), then NaN (12). Σ is NaN by step 14. `SR-WARN heat unconverged` fires
at steps 9, 11 and 12. The world's finite mass is 0.003 at step 14, the ledger's mass invariant goes 5,000 → 6 × 10⁻⁵, and the run reaches
`--until sim_time:0.5` at step 17 (Δt grows on the emptied world) with exit 0, before P9's step-64 guard.

**M10** · Where Σ_BH stands: a single cell of Σ = 12,000 (3 × Σ_BH, the grid SCF's own 12,000 start), at rest, sinks on, heat off
(`gpu_run.sh co 12000 gravity,hydro,sinks 0.3 10 co12000_s`; RTX 5090). **No sink.** ∇·u = 0 at rest, the cell's own g is 0 at its centre
(central differences, §1.4.2), and the cold pressure blows it apart (R₉₀ × 13.6, Σ_max 1,830 after step 0). Neutron matter at 5,000, same
switches: swells × 5.0, no sink. On this grid a sink needs gas *driven* into a cell. No cold equilibrium, at any mass, does that.

## Problem

1. **The ceiling is below the grid's resolution, by the physics.** S6's mechanism holds in the sheet: R → 0 as M → M_ch, and for §2.3.2's
   blend R/R₀ = f(M/M_ch) (M2). A cold disk's R₉₀ is under one cell from 0.51 M_ch, and the grid's discrete equilibrium is a 2 × 2 block
   from 0.62–0.78 M_ch (M1, M2). §2.11's « holds below, collapses above » has no grid object on either side of the ceiling. Above it,
   gravity cannot pull a cell into itself (its own g is 0, M10), so the grid has no collapse to show: none at 3 × M_ch (M1, M9).
2. **The swelling is the scheme's numerical heating, separated by measurement.** (a) It is the twin's too, mass for mass (M5): not f32,
   not the GPU. (b) It scales with the scheme's dissipation (first order 3–10× worse, no limiter non-finite; M5). (c) Without self-gravity,
   a static atmosphere under a fixed g drives flows first order in Δx/H and heats second order (M7). The scheme is not well-balanced (S3,
   S4): the gravity source and the HLL face pressures do not cancel. (d) The same disk converges to holding as it is resolved, and 3 % over
   10 τ_dyn needs R₉₀ ≳ 15 cells (M6). (e) Radiation does not remove the heat (M8). Hypothesis (a) of M0-T66, « the ceiling sits where
   R_eq → 0 », is M2's law. (b) and (c) of M0-T66 are one chain, both measured: imbalance → spurious flows → dissipation → thermal pressure
   (γ = 2, never radiated in time) → swelling.
3. **§2.3.2's number is the uniform disk's, 23 % high.** The sheet's own equilibrium profile has M_ch = 1.4331 K₂′²/G² (M3; S9's family of
   critical polytropes, validated on the Maclaurin disk). That puts m_ch ≈ 3,217 = 0.96 × the Sun-like mass (§2.1's 3,351) and m_tov ≈
   4,854. §2.7's « ≈ 1.2 × the Sun-like mass » no longer holds, and ignition.md's « m_ign < m_ch holds by inclusion » (M₀ < 3,970) fails:
   M0-T68 must measure m_ign < 3,217.
4. **ns_sigma_max_sb has no finite continuum value.** Σ_c → ∞ at the ceiling (M3: Σ_c 10⁹ at 0.999 m_tov). On the grid no neutron-matter
   run holds (M9), so « the largest Σ_c among holding runs » is empty. The only bound the grid itself guarantees is mass: a holding neutron
   star (M < m_tov) cannot put more than m_tov into one cell. §2.6's shipped Σ_BH = 4,000 sits **below** m_tov ≈ 4,854, so a stable neutron
   star of 4,000–4,854 gathered into one cell meets the density criterion. The arithmetic's 6,000 is above Σ_BH too.
5. **Σ_BH is never reached by a cold disk** (M9, M10: largest Σ 1,830 even from a 12,000 single cell). Whether a dynamic core collapse
   (M0-T68's preset lives) drives gas into one cell past Σ_BH is not measured here (UNVERIFIED).
6. **A heat-on runaway destroys the world silently** (M9). P5 on a dense, sub-resolution hot block runs E to NaN. The run then empties, and
   because the run ends before P9's step-64 guard it exits 0. Filed as M0-D58 (cause not measured). It bears on every whole life M0-T68
   runs, since its remnants are exactly such blocks.

## Options

What each changes. « T66 » is M0-T66's build, « Σ_BH » §2.6's bound, « K9 » §2.11 item 5's order and the band.

**(A) The continuum ceiling of the EoS the game runs: a CPU f64 SCF sequence, extrapolated (recommended).** T66 builds
`sr_physics::reference::sheet` (M3's solver, in Rust: the ring kernel, Hachisu with Σ_c fixed, M(Σ_c) at Σ_c = 10⁰…10⁸, the Richardson
limit) and `calibrate` items 1–2 on it, CPU only, in under a second. The draft's GPU trial and bisection are dropped, and its grid SCF is kept
for G-CORE's resolved scene. Values: m_ch_sb 3,216.4, m_tov_sb 4,853.2. ns_sigma_max_sb = m_tov_sb per cell area, and Σ_BH's bound becomes
Σ_BH ≥ ns_sigma_max_sb, which the shipped 4,000 fails (a question). K9: m_tov/m_ch 1.509 ∈ [1.4, 2.0] ✓; m_ch < m_up ✓ by inclusion (the
Sun-like preset, M₀ > m_ch, ends a white dwarf); m_ign < m_ch measured by M0-T68 (no longer by inclusion). G-CORE's four cases become:
0.95 / 1.1 → an equilibrium exists at 0.95 m_ch_sb and none above 1.1 m_ch_sb (the sequence's supremum). GPU against twin within 5 % → a
resolved grid hold, GPU and twin (M6: 1.62 % / 1.65 % at K₁ × 6). The pure-Σ² radius flat → kept, in the continuum (exact for γ = 2, S9).
Plus m_ch_sb against c_sheet K₂′² from an independent pure-law solve. Cost: CPU seconds plus one ~1,100-step GPU run and its twin.
**Fails** if the solver itself is wrong. Caught by the Maclaurin potential (10⁻⁴), the γ = 3 profile, the virial and the two independent
solves (pure eigen-mass against the EoS sequence). Plant `cold-exponent` removes the ceiling (M4); a wrong K₂ moves it as K₂² (M4). It does
not grade what the grid does near the ceiling. Nothing can (Problem 1), and the report says so in G-CORE's text.

**(B) Measure on a resolved copy (K₁ scaled).** The ceiling is K₁-independent (M2), but a 3 % hold needs R₉₀ ≳ 15 cells (M6). At 0.95 M_ch
that is R₀ ≈ 750 cells, past the 1,024-cell world (§0.5 [M0-V4]) and hours a run. It is forbidden as written anyway: a run with overrides
never writes calibration (§2.12.1). ns_sigma_max_sb is still empty (Problem 4). K9 is as for (A). G-CORE's 0.95 and 1.1 runs are infeasible.
**Rejected**: infeasible, and it would grade the scheme's imbalance, not the ceiling.

**(C) Read the ceiling off M0-T68's preset lives (m_up_sb's ladder); m_ch an arithmetic cross-check.** T66 writes the arithmetic, and m_ch
then means nothing measured. ns_sigma_max_sb is the largest Σ seen in the neutron-star remnants of the `massive` lives (each a whole life,
⏱). K9's m_ch < m_up becomes circular. G-CORE becomes G-END's duplicate. Cost: hours of whole lives per bisection step. **Fails** because a
remnant on this grid is a swollen 2 × 2 block (M9) whose Σ measures the scheme's heating, and a collapse may never show (M10). **Rejected.**

**(D) §2.7's arithmetic fixed as the ceilings (3,970 and 6,000), with a grader a wrong K₂ breaks.** T66 writes 1.767 K₂′²/G² with no
measurement. ns_sigma_max_sb = 6,000 per cell, and Σ_BH 4,000 fails the bound (as in (A)). K9 is as now (ratio 1.509). G-CORE: m_ch_sb =
1.767 K₂′² exactly, and an EoS slope test (Π/x^{3/2} → K₂ at the table's top) that `cold-exponent` turns red. Cost: trivial. **Fails**
because it ships a number 23 % above the sheet's own ceiling (M3) as « the cold ceiling ». The uniform disk is not an equilibrium, which is
exactly the « definition chosen because it is easy » the Adversarial line warns about. **Rejected.** (D with c_sheet = 1.4331 written in is
(A) without the computation through the EoS. It is weaker, because the plant would reach only the slope test.)

**(E) K₁ re-ruled so a cold disk is resolved on the shipped grid.** R₀ ∝ K₁ while R/R₀ = f(M/M_ch) (M2), so resolving 0.95 M_ch needs K₁
× ~190 (R₀ 750). Even resolving only 0.5 M_ch at 15 cells needs K₁ × ~13 (R₀ ≈ 51): every white dwarf 100 cells wide, and the Sun-like
core's cold pressure (d_c 0.6, ignition.md) re-tuned with M0-D50's ladder. ns_sigma_max_sb, K9 and G-CORE are as now, and still have no
collapse to show (M10). **Rejected**: it reshapes the game's stars to fit a check and still cannot reach the ceiling.

(Not an option for the ceiling, but the cure for Problem 2: a well-balanced reconstruction, S3, S4, would hold M6's under-resolved disks.
It is a hydro change, outside M0-R4's ruling. The UNVERIFIED section records what the imbalance does to remnants.)

## Recommendation

**(A).** It is §2.3.2's physics, the sheet's ultra-relativistic ceiling, K₁-independent and computed on the EoS the GPU tables, and it is
not the grid's artifact: the grid cannot hold a disk at that scale (Problem 1). Its grader breaks under a wrong exponent (no ceiling) and
under a wrong K₂ (∝ K₂²) (M4). The ceiling it writes is checked against an independent solve of the critical sheet. **What would falsify
it:** an independent thin-disk ceiling for Π ∝ Σ^{3/2} that differs from 1.4331 K₂′²/G² by over 0.5 % (a source, or T66's Rust solver at
400 annuli). Or the shipped EoS's M(Σ_c) not rising monotonically (a second branch). Or M0-T68's lives showing a cold remnant that holds
above m_ch_sb on a resolved scale.

### The amendment, ready for the lead (§0.5 override line, Superseded mirror, then in place)

**§0.5 line** (to append, after the lead's answers): `[M0-R4] §2.11 items 1–2 (a cold disk, a = 12, run 10 τ_dyn, bisection; « the CPU
twin may stand in »), §2.3.2's « M_ch ≈ 1.77 K₂'²/G² for a uniform disk … calibration measures the real value », §2.7's K rows and Σ_BH
row, §2.6's « bound ≥ 4 × calibration's ns_sigma_max_sb », §3.3's Calibrate line and §5.1's G-CORE · the texts below · on the shipped grid a
cold disk's R₉₀ is under one cell from 0.51 M_ch and nothing collapses at any mass to 12,000 — the ceiling is a continuum limit (R → 0),
1.4331 K₂′²/G² for the sheet's own profile (the uniform disk's 1.767 is no equilibrium); the grid's cold disks swell by the scheme's
numerical heating (not well-balanced: M7), converging to a 3 % hold only at R₉₀ ≳ 15 cells — measured on linux-pc, 2026-10-19
(reports/cold_ceiling.md M1–M10) — the lead's answers at M0-R4, 2026-10-19: «Continuum SCF (Recommended)», «Raise to 6,000
(Recommended)».` — **applied** 2026-10-19 (the contract's §0.5 [M0-R4] carries the full line; the plan's Superseded section mirrors it).

**The lead's answers** (question tool, M0-R4, 2026-10-19): Q1 « What should m_ch_sb / m_tov_sb be … » → «Continuum SCF (Recommended)»;
Q2 « Σ_BH … What should it be? » → «Raise to 6,000 (Recommended)».

**§2.3.2**, the sentence « In the ultra-relativistic limit the ceiling is M_ch ≈ 1.77 K₂'²/G² for a uniform disk, K₂' = K₂Y_e^{3/2} (SM M6's
arithmetic, × my Y_e scaling) — calibration measures the real value (§2.11). » becomes:
« In the ultra-relativistic limit the sheet has a ceiling, M_ch = c_sheet K₂′²/G², K₂′ = K₂Y_e^{3/2}, c_sheet = 1.4331, the critical sheet's
mass for its own equilibrium profile (reports/cold_ceiling.md M3; a uniform disk, which is no equilibrium, gives 9π/16 ≈ 1.767). The disk's
radius falls to zero at it, R ≈ R₀(1 − √(M/M_ch)), R₀ = 3K₁′/4G, so no grid run can show it: calibration computes it from this EoS in the
continuum (§2.11). »

**§2.7**, the rows:
| K₁ₑ, K₂ₑ | 21.3, 134 | electrons: with Y_e = 0.5, M_ch = 1.4331 × (134 × 0.5^{3/2})² ≈ 3,217 ≈ 0.96 × the Sun-like mass (§2.3.2); a cold disk's radius 3K₁Y_e²/(4G) ≈ 4.0 cells at low mass, its R₉₀ under one cell from M ≈ 0.5 M_ch (reports/cold_ceiling.md M2) |
| K₁ₙ, K₂ₙ | 2.67, 58.2 | neutron matter: M_tov = 1.4331 × 58.2² ≈ 4,854 ≈ 1.51 M_ch, radius ≈ 2.0 cells at low mass; bound m_tov/m_ch ∈ [1.4, 2.0] (K9's real 1.4–2: SP-E12, SP-E13) |
| sigma_bh | 6,000 [M0-R4] | ≥ calibration's ns_sigma_max_sb (§2.6, §2.11 item 5) |

**§2.6**, « Σ_BH: initial 4,000 (opinion); bound ≥ 4 × calibration's `ns_sigma_max_sb` (no stable neutron star can trip it). » becomes:
« Σ_BH: 6,000 (§2.7) [M0-R4]; bound ≥ calibration's `ns_sigma_max_sb` = m_tov_sb per cell area: a holding neutron star (M < m_tov)
cannot put more than its mass into one cell, so none can trip it; checked by `calibrate` (§2.11 item 5). »

**§2.11**, items 1–2 become:
« 1. m_ch_sb — the sheet's cold ceiling for this EoS, computed on the CPU in f64 (no adapter, no grid; `sr_physics::reference::sheet`): a
cold axisymmetric thin disk (T_floor; C 0.5 + O 0.5, so Y_e = 0.5) in hydrostatic equilibrium, h(Σ) + φ = const, h(Σ) = ∫ Π′(s)/s ds of
§2.3.3's Π through `sr_physics::eos`, φ(r) = −G ∫ Σ(r′) 4r′/(r + r′) K(k) dr′, k² = 4rr′/(r + r′)² (§1.4.1's law, axisymmetric). It is solved
by Hachisu's self-consistent field with the central Σ_c fixed: 200 equal annuli out to the edge, the disk's radius solved for, each
annulus's integral split at the field point, rounds until no Σ moves by 10⁻¹¹ Σ_c. The sequence M_k = M(Σ_c = 10^k) is taken for
k = 0 … 8, and m_ch_sb = M₈ + (M₈ − M₇)/(√10 − 1) (the deficit falls as Σ_c^{−1/2}). Exit 6 unless M_k rises at every k, every equilibrium's
|2∫Π dA/W + 1| ≤ 10⁻³, and the same limit from k = 6, 7 lies within 0.5 % (SM-O9).
2. m_tov_sb — the same with X_n = 1. ns_sigma_max_sb = m_tov_sb per cell area: the densest a holding neutron star can make one cell. »
and item 5 gains « and sigma_bh ≥ ns_sigma_max_sb (§2.6) »; the heading's « (the CPU twin may stand in for items 1–2) » becomes « (items 1–2
run on the CPU) ».

**§3.3**, the Calibrate line gains: « Items 1–2 run on the CPU in f64 and open no adapter; `--adapter` is needed only when an item that runs
the GPU is measured (`measured.adapter` names it, or `cpu` when none did). »

**§5.1 G-CORE** becomes:
« **G-CORE · the cold ceiling** [M0-R4]. (a) The sequence: on the CPU in f64, §2.11 item 1's cold C/O disk has M(Σ_c) rising at every
decade Σ_c = 10⁰ … 10⁸, each equilibrium's |2∫Π dA/W + 1| ≤ 10⁻³, and calibration's m_ch_sb within 0.5 % of c_sheet (K₂ₑ Y_e^{3/2})²/G²,
c_sheet the pure Π = KΣ^{3/2} sheet's eigen-mass computed in the test by an independent solver (Hachisu with Σ_max and the edge fixed) and
within 10⁻³ of 1.4331. The same for neutron matter (m_tov_sb, K₂ₙ). The ring kernel gives the Maclaurin disk's potential within 10⁻⁴
(r ≤ 0.95a). (b) The ceiling: an equilibrium exists at 0.95 m_ch_sb, and none at 1.1 m_ch_sb (above every M_k). (c) With the relativistic
branch removed (pure Σ²), R₉₀(M) flat within 3 % over M ∈ {0.25, 0.5, 1} × m_ch_sb (SM-E4: "For D=2, the radius is independent on mass" —
the sheet's analogue). (d) The grid holds a resolved cold disk: scenes/cold_co_disk.json, the grid's own SCF equilibrium (§1.4.2's kernel) of
M = 100 C/O at T_floor with k1e × 6 by `overrides` (R₉₀ 18.8 cells), 64 × 64, bounce, gravity and hydro only. R₉₀ stays within 3 % of its
start over 10 τ_dyn on the GPU and the twin, and the GPU's final R₉₀ is within 1 % of the twin's. The ceiling is a continuum limit (R → 0),
so no grid run grades it (reports/cold_ceiling.md). (a)–(c) *Standard* (the SCF method; M3's values), (d) *measured* (1.62 % GPU,
1.65 % twin, RTX 5090, 2026-10-19). Scope `cold_core`. Plants `cold-exponent`: the relativistic exponent 3/2 → 1.6 in the EoS (no ceiling:
M past 10⁵ at Σ_c 10⁸); `cold-gpu-pressure`: the GPU's cold pressure × 0.9 (d). »
and §5.5 drops G-CORE from its list.

The Superseded section mirrors each passage above: « <passage> — superseded by contract §0.5 [M0-R4]: <one line> (the lead, 2026-10-19,
M0-R4: «Continuum SCF (Recommended)», «Raise to 6,000 (Recommended)») ». Applied, in three lines, the header budget being 600.
M0-T66b also carries assets/physics.json's sigma_bh 4,000 → 6,000 (a constant is a build's to change).

### The blocks, rewritten (sizes: new lines)

**M0-T66 · The sheet's cold equilibria — `sr_physics::reference::sheet`** · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-R4)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.1, §2.3.2–§2.3.3, §2.11 items 1–2 [M0-R4] + reports/cold_ceiling.md
  (M2–M4, § Recommendation) + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/sheet.rs (new, ≈ 300): the ring kernel (K(k) by the AGM; each annulus by 8-point Gauss–Legendre,
  split at the field point by r′ = r ± t²), Hachisu's SCF for any barotropic h(Σ) with Σ_c fixed (the radius solved for) and for a pure
  Π = KΣ^γ with Σ_max and the edge fixed, M, R₉₀, W, ∫Π dA, and §2.11 items 1–2's sequence and limit with its refusals → Verify 1 · its tests
  (in the module, ≈ 150): the Maclaurin disk's potential within 10⁻⁴ (M0-R4: 6.4 × 10⁻⁵), γ = 3's equilibrium the Maclaurin profile within
  10⁻³ (2.6 × 10⁻⁴), c_sheet within 10⁻³ of 1.4331 at 100 and 200 annuli with its virial within 10⁻⁴, the shipped C/O and neutron limits
  within 0.5 % of c_sheet K₂′² (3,216.4 and 4,853.2), a γ_UR = 1.6 law refused (no ceiling) → scope `sheet` → Verify 1 · plant
  tests/plants/sheet-ring-kernel.patch (the ring's 4r′/(r + r′) → 2r′/(r + r′)) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py sheet --task M0-T66` · Pass: GO, ≥ 5 cases · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm
  sheet --task M0-T66` · Pass: GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: a solver checked only against its own output proves nothing; the Maclaurin potential and the γ = 3 profile are closed forms
  (S7, SM-E6), and c_sheet comes from a second iteration (Σ_max fixed) the sequence never uses.

**M0-T66b · The `calibrate` command and items 1–2** · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T66)
- Carried flags: M0-T65's three (Calibration::to_json, Calibration::parse wants the complete file, a run with overrides never writes)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.6, §2.11 [M0-R4], §2.12.1, §3.3 + milestones/m0/tasks/M0-T66.calibrate.rs.txt
  (the argument parser, `base_of`, `check_k9`, `utc_iso`, `equilibrium` and `cold_disk_scene`) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/calibrate.rs (new, ≈ 400: the draft without its GPU trial, bisection and `SR_CAL_*` hooks) and its
  subcommand in crates/sr-app/src/main.rs: `calibrate --out <file> [--adapter S] [--only <item>]`; items 1–2 from `sheet` (CPU, no adapter);
  the `--out` merge; K9 and sigma_bh ≥ ns_sigma_max_sb on the merged file, exit 6; the grid SCF kept as `cold_disk_scene` for G-CORE (d) →
  Verify 1 · tests/gpu/calibrate_cold.rs (new, ≈ 150): `--only m_ch_sb` keeps the other items, a file under another hash dropped with
  `SR-WARN`, overrides refused, K9's order and band, a sigma_bh under ns_sigma_max_sb → exit 6 naming it, the values written equal `sheet`'s
  limits → scope `calibrate_cold` → Verify 1 · plant tests/plants/calibrate-cold-matter.patch (item 1 on neutron matter's composition: K9
  band 1.0) → Verify 2 · the two items once into build/calibration.partial.json, wall time logged → Verify 3
- Verify: 1. `python3 tools/pb/verify.py calibrate_cold --task M0-T66b` · Pass: GO, ≥ 5 cases · Fail: NO-GO · 2. `python3 tools/pb/verify.py
  --redarm calibrate_cold --task M0-T66b` · Pass: GO — the plant NO-GO · Fail: the plant stays GO · 3. `build/target/release/sandbox-reactions
  calibrate --out build/calibration.partial.json --only m_ch_sb` · Pass: exit 0, m_ch_sb within 0.5 % of 3,216.5 · Fail: exit 6 or another value

**M0-T67 · The cold ceiling holds (G-CORE)** · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T66b)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.1 (K9), §2.3.2, §5.1 G-CORE [M0-R4], §5.5 + reports/cold_ceiling.md
  (M3, M4, M6) + docs/agent/testing.md
- Deliver: tests/gpu/cold_core.rs (new, ≈ 250) with scenes/cold_co_disk.json (new; the test rebuilds it with `cold_disk_scene` and refuses a
  stale file): G-CORE (a)–(d) [M0-R4] → scope `cold_core` → Verify 1 · plants tests/plants/cold-exponent.patch (eos.rs's relativistic
  exponent 3/2 → 1.6) and tests/plants/cold-gpu-pressure.patch (eos.wgsl's cold pressure × 0.9) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py cold_core --task M0-T67` · Pass: GO, ≥ 6 cases · Fail: NO-GO — a red is a D, or a question when the
  sheet's ceiling itself is at stake · 2. `python3 tools/pb/verify.py --redarm cold_core --task M0-T67` · Pass: GO — both plants NO-GO · Fail:
  a plant stays GO
- Adversarial: m_ch_sb checked against the computation that wrote it proves nothing — (a)'s c_sheet comes from the test's own pure-law
  solver, and (b) is a fresh sequence. (d) grades the grid's statics and its hydro, never the ceiling, and says so.

Retired: §2.11's GPU bisection (the draft's `Calibrator::trial`, `bisect`), the planned plant `calibrate-cold-hold`, and the scope name
`calibrate-cold` (now `calibrate_cold`, R17).

## UNVERIFIED
- The blended law's virial at each step of the sequence (≤ 10⁻³): measured only on the pure law (2 × 10⁻⁵). The sequence runs the analytic
  blend, not `sr_physics::eos`'s tabulated u(x) with T_floor's isothermal term. The table's interpolation and Π_th = ΣT_floor/μ are expected
  to move the limit far under 0.5 % (they vanish as Σ_c → ∞); T66 measures it.
- c_sheet = 1.4331 is my computation (two resolutions agree to 10⁻⁴, the kernel validated on closed forms). No source treats the degenerate
  razor-thin sheet (SM's UNVERIFIED flag (a) stands, now with a number).
- The `cold-gpu-pressure` plant turning (d) red is expected (an R ∝ K₁ disk 10 % off its equilibrium), not measured. The same goes for
  `sheet-ring-kernel` and `calibrate-cold-matter`.
- Whether a dynamic core collapse in M0-T68's `massive` and `giant` lives drives gas into one cell past Σ_BH (any value): no cold equilibrium
  does (M9, M10). If none does, the black-hole ending cannot form a sink, a risk for G-END and m_bh_sb.
- What the scheme's imbalance does to remnants over a life: every under-resolved cold equilibrium swells (M5, M6), and heat on does not stop
  it within 10 τ_dyn (M8). A white dwarf of 0.6 M_ch on this grid is a ~1-cell object, so G-STAGES' white_dwarf may read a swollen, warm blob.
  Longer runs are not measured.
- M0-D58's cause (P5's unconverged pass, the radiation push, or the closure δ) is not measured, nor whether the twin runs away too.
- win-laptop: nothing here ran there (NOT RUN — owed on Laser2025-20 for the GPU rows, R15).
