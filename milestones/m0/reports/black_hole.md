# M0-R6 — Black holes: why no collapse leaves more than ~M_ch, and what lets a heavy star make a sink

Box: laserax-ai-server, NVIDIA GeForce RTX 4090 (Vulkan, driver 595.99.02), **idle** at each probe's start (0 % utilisation, 17 MiB, load
0.2–1.0 on 20 threads; each header in logs/M0-R6-s2.probe.log). Date 2026-10-23. The lead: «do task R6 on ther server RTX 4090» (R18). Tree:
HEAD 8332496 plus the working tree's uncommitted changes (M0-D81, M0-D87, M0-D88), physics_hash 8ca855fb…, the same physics as the kept
record `build/calibrate_runs/2026-10-21T19-39-14Z/` (M0-D79's) but a later engine (M0-D80–D88). Scratch copy on linux-pc
`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/d9a1a68d-c3f3-4dd2-964c-eaddd51cdc51/scratchpad/r6`, built and run on the
server at `/tmp/r6` (probe folders `/tmp/r6p/<name>/`); its four scratch-only env hooks, none in the tree: `SR_PROBE_DUMP_EVERY` (a dump and
an observers' snapshot every N steps, kept after the life), `SR_PROBE_OVERRIDES` (§2.12.1 overrides), `SR_PROBE_EDGE=bounce`,
`SR_PROBE_RADIUS` (replaces §2.10's 150-cell cap). Each probe is one `calibrate --only m_bh_sb` life from step 0 (`scratchpad/scripts/
probe.sh`), progress every 100 steps, capped at 570 s; the dumps are read by stdlib Python (`scripts/rows.py`, `core.py`, `clumps.py`,
`ledger.py`, `maxcore.py`). Nothing landed in the tree but this report, the plan's blocks and `logs/M0-R6.*`.

## Evidence

Sources, all accessed 2026-10-23:

- Fryer, Belczynski, Wiktorowicz, Dominik, Kalogera & Holz, "Compact Remnant Mass Function: Dependence on the Explosion Mechanism and
  Metallicity", ApJ 749:91 (2012) — https://arxiv.org/abs/1110.1726 (accessed 2026-10-23): "analytical prescriptions for compact object
  masses" for population-synthesis codes. The prescriptions (the paper's body, from memory — UNVERIFIED) build the remnant as the
  proto-compact object plus the fallback, and call it a black hole above the maximum neutron-star mass: **simplified codes decide the
  ending by the remnant's mass, not by a density**.
- O'Connor & Ott, "Black Hole Formation in Failing Core-Collapse Supernovae", ApJ 730:70 (2011) —
  https://arxiv.org/abs/1010.5550 (accessed 2026-10-23): with GR1D and "a three-species neutrino leakage/heating scheme" over 100+ progenitors, the outcome "can be
  estimated, to first order, by a single parameter, the compactness of the stellar core at bounce"; thermal pressure raises "the maximum
  PNS mass by up to 25% above the cold NS value"; whether a core explodes is "the critical neutrino heating efficiency required for
  exploding a given progenitor structure". A black hole is a proto-neutron star pushed past its maximum mass by accretion.
- Sukhbold, Ertl, Woosley, Brown & Janka, "Core-Collapse Supernovae from 9 to 120 Solar Masses Based on Neutrino-powered Explosions",
  ApJ 821:38 (2016) — https://arxiv.org/abs/1510.04643 (accessed 2026-10-23): "Many progenitors with extended core structures do not
  explode, but become black holes"; mean black-hole mass ~9 M☉ when only the helium core collapses, ~14 M☉ for the whole star — in nature a
  black hole keeps **far more** than the iron core, through a failed or weak explosion.
- Federrath, Banerjee, Clark & Klessen, "Modeling Collapse and Accretion in Turbulent Gas Clouds: Implementation and Comparison of Sink
  Particles in AMR and SPH", ApJ 713:269 (2010) — https://arxiv.org/abs/1001.4456 (accessed 2026-10-23), summary page
  https://www.mso.anu.edu.au/~chfeder/pubs/sinks/sinks.html (accessed 2026-10-23): grid sinks form "in regions of local gravitational
  collapse"; a density threshold alone is insufficient, so they add the bound state, the potential minimum, Jeans instability and a
  converging flow — §2.6's four tests (SM-E28) come from here.
- Bate, Bonnell & Price, "Modelling Accretion in Protobinary Systems", MNRAS 277:362 (1995) —
  https://arxiv.org/abs/astro-ph/9510149 (accessed 2026-10-23): "each high-density region is replaced by a single, non-gaseous particle" that "contains all the mass in the
  region and accretes any infalling mass" — a sink takes **the region's** mass, not one cell's.
- In-house: reports/cold_ceiling.md (M0-R4) M10 — a single cell of Σ 12,000 at rest «swells × 5.0, no sink. On this grid a sink needs gas
  *driven* into a cell. No cold equilibrium, at any mass, does that»; M2 — R ≈ R₀(1 − √(M/M_ch)), R₀ = 3K₁′/4G ≈ 4 cells for C/O.

What the sources fix for the sandbox: (1) the black hole against the neutron star is decided by the **remnant's mass** against the maximum
neutron-star mass (Fryer; O'Connor & Ott), the remnant being the core plus what falls back; (2) heavy stars make black holes because their
explosion fails or is weak, so they keep their helium core or more (Sukhbold); (3) grid and particle codes make sinks from a region's bound,
collapsing mass (Federrath; Bate). §2.6 asks a **density** a one-cell grid cannot reach (M0-R4 M10) — the mismatch below.

## Problem

The remnant's mass separated by measurement, on this tree (the kept record's engine was older: its collapse at step 1,489 is 1,748 here).

**M1 · The 20 M₀ cloud never makes one core: it collapses to a ring that breaks into 8–16 clumps, each near M_ch** (P1 `base20d`,
67,020.643, 6,000 steps, 262 s, dumps every 100; `clumps.py`). The whole cloud burns to iron at once (X_Fe 66,747 of 67,016 at step 600,
t 4.3; the 5.7 × 10⁸ of burning leaves as neutrinos in the same window, M3). The cold iron disk (a = 150, the cap) then collapses to a
**ring at r ≈ 92 cells** (4 peaks at step 1,200), 16 peaks by 1,600 (8 of Σ 265–287 at r ≈ 110, M within 4 cells 2,112–3,152), and each clump
collapses on its own: at core collapse (step 1,748, t 10.93) the four heaviest hold **3,428–3,526 within 4 cells** (iron, X_Fe 1.00,
step 1,700) — already above iron's continuum ceiling 2,576, which the grid holds (M0-R4). Neutronized (step 2,000: X_n 0.53–0.57) they hold
3,411–3,538, and no clump merges with another through step 6,000 (the heaviest's r ≤ 4 mass 5,298 at step 4,100, by accretion). The
world's centre (300, 200) never holds a core. The light end does not fragment (M2).

**M2 · The light end makes one central core** (P2 `base_lo`, 4,572.394, ending neutron_star step 5,974, t 29.126, 192 s): one peak at
(300, 199) from step 1,250; 3,219 within 4 cells at step 1,500 (iron), 3,167 neutronized (step 1,750, Y_e ≤ 0.35); core_collapse 1,516,
supernova 1,517; remnant **2,380** (summary), 2,349 of the grid's 2,496 within 4 cells at step 5,750 (99 % of the remnant inside r = 4).
Escaped 2,103.6 (46 %).

**M3 · What unbinds it: the floor, not the neutrinos** (`ledger.py` over each dump's observers' snapshot, P1; energies in 10⁶):

| step (t) | M_grid | escaped | E_gas = K + U + U_cold + W | nuclear | ν lost | ν deposited | floor_added | radiated |
|---|---|---|---|---|---|---|---|---|
| 600 (4.29) | 67,016 | 4 | −26.63 | 569.26 | 567.89 | 0.00 | −0.00 | 3.58 |
| 1,600 (10.17) | 66,462 | 558 | −33.26 | 569.26 | 567.89 | 0.00 | 0.06 | 7.74 |
| 2,100 (12.39) | 65,213 | 1,807 | −33.39 | 475.53 | 568.39 | 0.01 | 103.02 | 10.09 |
| 2,600 (14.16) | 61,344 | 5,676 | −21.12 | 461.96 | 568.54 | 0.01 | 135.82 | 11.95 |
| 3,600 (17.54) | 48,694 | 18,326 | −20.49 | 448.00 | 568.71 | 0.02 | 167.85 | 16.80 |
| 5,600 (25.32) | 40,256 | 26,765 | −19.14 | 437.52 | 568.83 | 0.02 | 197.11 | 25.54 |

  At the collapse, N_Fe's endothermic draw (nuclear −93.7 from 1,600 to 2,100) is paid by P8's floor (+103.0): the cores sit at T_floor
  (T_c 0.050, d_c 0.9999 in every collapse row), so the heat N_Fe takes is refilled — the floor-fed absorption M0-R5 named, now measured.
  After it the floor keeps adding: **+197.1 against nuclear −131.7 from step 1,600 to 5,600, a net +65 × 10⁶ — twice the gas's binding at
  collapse (−33.3)**, while E_gas rises from −33.4 to −21.1 during the supernova (2,100 → 2,600). Neutrino heating deposits **2 × 10⁴** in the
  whole life, 10⁻⁴ of the floor's: f_dep 0.05 does not unbind anything. The light end the same: floor +35.1 against nuclear −18.4 (1,250 →
  5,750), net +16.7 against a binding of −2.66 at collapse (6 ×). The electrons' cold energy (U_cold 14.2 → 8.2 from 2,100 to 2,600) is
  inside E_gas: its fall is the cold → heat transfer M0-R5 read, smaller than the floor's net and not separated per cell (UNVERIFIED).

**M4 · Where the mass goes: through the leave edge, because the star is wider than the world.** The heaviest object's R is 220–290 cells
from step 1,600 on (P1 rows), against the world's half-height of 200: the envelope crosses the top and bottom edges from the main sequence on
(escaped 0.02 at step 300, 558 before the collapse, 920 at it). M_unb peaks at 13,925 (step 2,500) and falls to ~2,000; the escaped mass
reaches 27,596 by step 6,000 (41 %), 63,541.8 by the kept record's end (95 %). The observer's one object (objects = 1 throughout) is the whole
bound cloud with its clumps (M 39,398 at step 6,000), not a remnant: the record's "remnant 2,633" was the object left after 95 % had gone
through the edge.

**M5 · f_dep = 0 changes nothing** (P3 `fdep0_20`, 214 s; P4 `fdep0_lo`, ending neutron_star step 5,878 t 28.744, remnant 2,392.1, 192 s):
at 20 M₀ escaped 25,800 against 27,596 at step 6,000, the ledger the same to 1 % until 2,100 (floor 102.88 against 103.02); the light end's
remnant 2,392 against 2,380.

**M6 · A walled world keeps all the mass and still makes no sink** (P5 `bounce20`, 240 s; P6 `bounce_lo`, ending neutron_star step 6,282
t 30.326, 272 s): at 20 M₀ the grid keeps 67,015–67,020, M_unb peaks at 6,704 and falls back to 610 (step 5,814) — bound ejecta return —
but onto the clumps of M1: the heaviest neutronized core reaches 6,460 within 4 cells (step 3,500), Σ_max 1,001. The light end walled:
remnant 2,342.6 plus a second object of 2,152.4 (the walled ejecta, planetary_nebula, re-ignited at step 3,724) — nothing falls onto the
neutron star.

**M7 · Heavier or more compact does not make one core** (P7 `m40`, 134,041.287 = 40 M₀, 354 s; P8 `compact20`, 20 M₀ with the cap at 60
cells, 214 s; `mid15k`, 15,000, a = 84.6, 240 s): 40 M₀ makes a ring of 70 peaks at step 1,000 and ~19 clumps of 3,000–5,500 within 4 cells,
Σ_max ≤ 1,149; the compact 20 M₀ makes a ring at r ≈ 52 (8 clumps at step 500) then 4 cores of 4,300–6,650 within 4 cells (6,653 at step 1,500,
over m_tov_sb), **Σ_max ≤ 1,554**; 15,000 makes 4 clumps at r ≈ 33 that merge at the centre by step 3,000 (iron, 4,996 within 4 cells for
one dump, X_n 0.03), then one neutron star of 2,550–3,730. Over every probe the densest cell reached is **1,554 against Σ_BH 6,000**:
§2.6's one-cell density is never approached, whatever the mass in the core (M0-R4 M10's grid fact, now on dynamic collapses).

**M8 · A mass test separates the ends** (`maxcore.py` over every dump: the largest mass within r of a local Σ maximum ≥ 100 whose r ≤ 3
mass-weighted Y_e ≤ 0.35, against m_tov_sb 4,853.2; offline, on the dumps of P1–P8 and mid15k):

| probe | r ≤ 2 | r ≤ 3 | r ≤ 4 | r ≤ 5 | r ≤ 6 | first step ≥ m_tov_sb at r ≤ 4 |
|---|---|---|---|---|---|---|
| base_lo (4,572) | 2,664 | 3,081 | 3,167 | 3,198 | 3,209 | never |
| fdep0_lo | 2,667 | 3,082 | 3,166 | 3,198 | 3,208 | never |
| bounce_lo | 2,795 | 3,100 | 3,166 | 3,193 | 3,204 | never |
| mid15k (15,000) | 3,378 | 4,054 | 4,318 | 4,625 | 4,919 | never (r ≤ 6: 3,250) |
| base20d (20 M₀) | 3,626 | 4,376 | 5,298 | 5,866 | 6,050 | 3,900 (t 18.3) |
| fdep0_20 | 3,708 | 4,575 | 5,537 | 6,399 | 6,713 | 3,400 |
| bounce20 | 4,014 | 5,300 | 6,460 | 7,268 | 7,539 | 2,750 |
| compact20 | 5,314 | 6,617 | 7,369 | 8,486 | 9,504 | 1,500 |
| m40 (40 M₀) | 4,538 | 5,925 | 7,047 | 7,524 | 7,882 | 2,000 |

  At r = 4 the light end stays at 0.65 m_tov_sb in all three variants and 15,000 at 0.89; the 20 M₀ life passes at step 3,900 by 9 %, the
  40 M₀ life at 2,000 by 45 %. r ≤ 3 misses the plain 20 M₀ life; r ≤ 6 catches 15,000. Without the Y_e filter, 15,000's iron merger
  (4,996 at step 3,000, X_n 0.03) passes at r = 4 — the filter is what keeps an iron core that is still collapsing out.

**M9 · A defect met on the way: one step of the 40 M₀ life wipes the grid's heat** (P7's dumps, then `headless --load-dump s_2510.bin
--continue --steps 2520 --dump-every 1`, 2 s): in the step from s_2517 to s_2518 — the step whose P5 prints `SR-WARN heat unconverged step
2517 passes 1 rounds 4096` — the grid's thermal energy goes from 5.67 × 10⁶ to 0.00, floor_added and radiated each jump by **4.193 × 10¹⁶** and the energy invariant by
**+1.71 × 10⁹** (−162.3 × 10⁶ → +1,550.3 × 10⁶); the replay from s_2500 reproduced it between 2,510 and 2,520, and one step from the kept s_2517 (server
`build/r6_evidence/m40/`) reproduces it in 2 s. Filed as M0-D89.

## Options

Each was measured on the 20 M₀ life and the light end 4,572.394 (both must differ: the light end a neutron star, the heavy end a sink), on
the RTX 4090, idle. Wall times are the probes' (20 M₀ to step 6,000; the light end to its ending).

- **(a) The same physics, a wider bracket.** 40 M₀ (P7) and a compact 20 M₀ (P8) put 7,047 and 7,369 within 4 cells, but Σ_max stays at
  1,149 and 1,554: no sink under §2.6 at any mass the screen holds (M7). Wall: 354 s (40 M₀), 214 s. **Refused by measurement**: both ends
  stay neutron stars, the bracket is refused again. Moves nothing.
- **(b) Less heating.** f_dep 0 (P3, P4): no change (M5), because neutrino heating deposits 2 × 10⁴ against the floor's 10⁸ (M3). Wall:
  214 s, 192 s. Routing the cold energy's heat to neutrinos was not built: the floor-fed N_Fe draw and the floor's post-collapse net
  (+65 × 10⁶) are the larger source and belong to M0-D71's well-balanced P4 (flagged there); even with no explosion at all, M1's clumps
  hold under §2.6's density. **Refused**: no measured lever.
- **(c) Fallback.** What the sandbox lacks is a world that keeps the envelope: the star is wider than the world (M4). A walled world
  keeps it and it falls back (M_unb 6,704 → 610), but onto 8–16 clumps, Σ_max 1,001 — no sink (M6). Wall: 240 s, 272 s. A bounce edge in
  calibrate would also make m_bh_sb measure a world the player does not start in (§2.8: a new world leaves). **Refused** as the fix; it
  helps under (e) (bounce20 passes at step 2,750 against 3,900).
- **(d) Relabel: m_bh_sb places the remnant's mass, not the sink.** The observer's object is the whole bound cloud (M 39,398 at step
  6,000), so «M ≥ m_tov_sb» would read black_hole on a cloud of separate neutron stars — a stage from a script, not the physics. Refused.
- **(e) A sink from mass, not from one cell's density** (new; the sources' rule): P3 also forms a sink where a local Σ maximum holds
  **m_tov_sb or more within 4 cells, neutronized (Y_e ≤ 0.35 over r ≤ 3)** with §2.6's other three tests (∇·u < 0, φ minimum, bound) at that
  cell, and the sink takes the disk's mass above the floor. Measured offline on the dumps (M8): the light end never passes (3,167, 0.65 ×,
  all three variants), 15,000 never (4,318), the 20 M₀ life at step 3,900, 40 M₀ at 2,000. Wall: the 20 M₀ life to step 3,900 is ~170 s on
  the RTX 4090 (P1's pace), plus its black-hole hold. Moves: the massive preset (15 Suns, placed between m_up_sb and the new m_bh_sb by
  §1.10.1) must still end neutron_star — 15,000 holds 0.89 m_tov_sb; S2's failure rule unchanged (the explosion comes first; the sink forms
  after neutronization, t 18.3 at 20 M₀ against the supernova at 12.46); m_up_sb unchanged (its lives stop at core collapse,
  `collapse|ending`); Σ_BH 6,000 and K9's Σ_BH ≥ ns_sigma_max_sb unchanged (not lowered).

## Recommendation

**(e): a sink forms from the mass a neutronized core gathers within 4 cells, against m_tov_sb** — §2.6's density test kept beside it.

Why. (1) It is how the sources decide: the remnant's mass against the maximum neutron-star mass (Fryer; O'Connor & Ott), and a grid sink
from a region's bound, collapsing mass (Federrath; Bate). (2) §2.6's Σ ≥ Σ_BH in one cell is unreachable on this grid at any core mass —
the densest cell of every probe is 1,554 (M7), as M0-R4 M10 found for cold disks; m_tov_sb is a mass, and the grid can measure a mass.
(3) r = 4 is where the grid keeps its neutron star (99 % of the light end's remnant, M2; M0-R4's R₀ ≈ 4 cells) and the radius M8 supports:
3 misses the 20 M₀ life, 6 catches 15,000. (4) The Y_e ≤ 0.35 filter is the neutron_star row's own bound (§0.5 [M0-D79]), so an iron core
still collapsing (15,000's merger, M8) cannot form one. (5) Both ends differ by measurement: the light end at 0.65 m_tov_sb in every variant
(f_dep 0, bounce included), the heavy end past it.

What would falsify it — each measured by the D: the 20 M₀ life (life 02 of m_bh_sb's bracket) forming no sink within its `--max-steps`, or
its sink failing §2.6's other tests at every passing step (∇·u < 0 at the clump's centre is not read offline); the 4,572.394 life forming one;
the 15,000 life forming one (the massive preset's range); the black_hole stage not following the sink (M0-D70: a sink with no gas around
it is no object). What a wrong radius breaks: 3 → the bracket refused again (4,376 < 4,853 at 20 M₀); ≥ 6 → the massive preset's range at
risk (15,000 at 4,919).

**The contract's §0.5 line (the D writes it, its mirror in m0_superseded.md):**

> - [M0-R6] §2.6's Formation ("all four at one cell … Σ ≥ Σ_BH") · **a sink also forms at a cell that is a local Σ maximum whose disk r ≤
>   r_core = 4 cells holds M_core ≥ m_tov_sb (§2.11 item 2's value) with the disk's mass-weighted Y_e over r ≤ 3 at most 0.35 (§0.5
>   [M0-D79]'s bound), the cell passing §2.6's other three tests (∇·u < 0, φ no greater than its 8 neighbours, bound); that sink takes the
>   disk's mass above the floor and its momentum, its thermal and cold energy booked `swallowed`; Σ ≥ Σ_BH alone still forms one** — a
>   one-cell grid holds a neutron star's mass in a block of a few cells and never drives Σ past Σ_BH (densest cell 1,554 over every probe,
>   reports/black_hole.md M7; M0-R4 M10), while the black hole is decided by the remnant's mass against the maximum neutron-star mass
>   (Fryer et al. 2012; O'Connor & Ott 2011) · measured on laserax-ai-server, 2026-10-23, RTX 4090: within 4 cells the 4,572.394 life's
>   neutronized core peaks at 3,167 (0.65 m_tov_sb), 15,000's at 4,318, the 20 M₀ life passes at step 3,900 and 40 M₀ at 2,000
>   (reports/black_hole.md M8) — the lead, 2026-10-23, at M0-R6: «Mass sink, r=4 (Recommended)».

**The D's `Deliver:` / `Verify:`** (filed as M0-D90 with the lead's answer):
- Deliver: the mass test in P3's formation on the CPU twin (crates/sr-physics/src/reference/sinks.rs) first, then the GPU
  (crates/sr-engine/src/step/sinks.rs, shaders/sinks.wgsl), bit-for-bit order as §2.6's (the densest qualifying candidate; ties the lowest
  y, then x); m_tov_sb reaching P3 as §2.11 item 2's value; the sink taking the disk's mass; `sink` cases — a 4-cell block of neutron
  matter of 1.05 m_tov_sb forms one, 0.95 does not, the same mass at Y_e 0.464 (iron) does not, one passing the mass but diverging (∇·u > 0)
  does not, Σ ≥ Σ_BH alone still does — twin and GPU (cpu_gpu's tolerance); one plant (the Y_e filter removed); the §0.5 line and its
  mirror; testing.md's sink row; readouts.md if the stage text names Σ_BH.
- Verify: 1. `python3 tools/pb/verify.py sink cpu_gpu classify observe_run calibrate_bisect --task M0-D90` (on laserax-ai-server,
  `SR_TEST_ADAPTER="RTX 4090"`) · Pass: GO, every scope · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm sink --task M0-D90` ·
  Pass: every plant red, the new one included · Fail: a plant green.
- Done when: on the shipped binary, RTX 4090, `calibrate --only m_bh_sb --bracket 4572.394,67020.643` — life 01 ends neutron_star with
  no sink, life 02 forms a sink (its step and mass named) and ends black_hole, or the D's measurement of why not; the 15,000 life
  (`--bracket 15000,15015`, life 01) forms none. Each life < 10 minutes on the RTX 4090 (P1–P8's pace); the 5090's readings and the
  m_bh_sb calibration are the lead's.

## UNVERIFIED

- (e) is measured **offline** on dumps every 100 or 250 steps: the mass, Y_e and radius tests only. ∇·u < 0, the φ minimum and the bound
  test at the candidate were not read (φ is not in a dump), and a formed sink's accretion and its effect on the rest of the life (more
  accretion, the black_hole stage, the ending's hold) were not run — NOT PROVEN (offline replay). Between dumps a core may pass the
  threshold earlier than the step named.
- The floor's net post-collapse injection (+65 × 10⁶ at 20 M₀, M3) is measured as ledger totals, not per cell; that it is P8 refilling
  cold cores under N_Fe and P4's well-imbalance (M0-D71) is my reading. The cold-energy-to-heat transfer is not separated from it.
- Fryer et al.'s remnant = proto-compact object + fallback, black hole above the maximum neutron-star mass: from the paper's body, from
  memory; the abstract read here says only "analytical prescriptions for compact object masses".
- The ring fragmentation (M1, M7) is read from Σ peaks; its cause — a cold, thin, self-gravitating iron disk after the whole cloud burns
  and its heat leaves as neutrinos at t ≈ 4 (M3), the square grid's 4-fold symmetry seeding 8 clumps — is not separated; the fix does not
  depend on it, and a cloud that fragments in nature would not be one star (a later milestone's question).
- This tree's engine, not the kept record's: the record's life 02 collapsed at 1,489 and ended with 2,633 at step 27,249; here the
  collapse is at 1,748 and the lives were cut at 6,000 steps (each probe < 10 minutes). The 20 M₀ life's ending was not reached in any probe.
- Every number is the RTX 4090's; the RTX 5090 was not used (R18).

## The lead's answer (2026-10-23, PLAYBOOK §12, one question call, verbatim)

- Q1 (the fix; options «Mass sink, r=4 (Recommended)» · «Mass sink, r=3» · «Defer black holes»): «Mass sink, r=4 (Recommended)».
- Q2 (M0-D89's place; options «Before the BH D (Recommended)» · «After M0-V9»): «Before the BH D (Recommended)».
- Filed: M0-D89 (the one-step heat wipe, M9) right after M0-R6, then M0-D90 (option (e), the Recommendation's Deliver / Verify), both
  before M0-V9.
