# M0-R11 — Unresolved remnant: what lets a sub-cell white dwarf or neutron star sit still and shine on its own heat

Box: linux-pc (laserax-ai), CPU f64 twin only — no GPU run. Date 2026-10-28. Tree: HEAD c6c8b4b plus the working tree's uncommitted
changes (M0-D68, M0-D69's P4 and gravity), the twin unchanged. Scratch copy
`/tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/500019b7-ff89-4ff7-af9e-27460e4c5d5b/scratchpad/w` (built into M0-D71's target
dir), runs in `…/scratchpad/runs/`; the candidates are scratch-only (`crates/sr-physics/src/reference/r11.rs` and hooks in `reference/mod.rs`,
`reference/hydro.rs`, chosen by `SR_R11`, `SR_R11_ETA`, `SR_R11_SHOLD`); drivers `crates/sr-engine/examples/r11c.rs` (M0-D67's crop) and
`r11d.rs` (calibrate's SCF cold disk). Nothing landed in the tree but this report, the plan's edits and `logs/M0-R11.log`.

## Bottom line
The floor-fed light is the cost of a sub-cell core that P4 cannot hold still. Changing where that energy is booked does not change how
much of it there is: dual energy (D) and the neighbours' heat (N) both fail. Taking the remnant's cells out of P4 does work. In **H, the
held core**, a cold, dense cell is frozen, and its faces to free gas are reflecting walls that do no gravitational work. On M0-D67's crop
H cuts the floor 57×: 3.6 × 10³ against 2.06 × 10⁵ over steps 5,000–10,000. It books 3.8 % of radiated over the 10,000 steps, against
71.2 % today. Both SCF disks hold at rest, and the energy invariant keeps today's drift. H changes P4, P8 and the momentum ledger, and it
leaves three questions for the build: collapse, accretion and motion (§ What H leaves open).

## Scenes and baseline
- **Crop**: M0-D67's 128 × 128 crop of `build/v9-sun-life/dumps/s_450000.bin` (box [248,144,104,104], vacuum outside, leave), every twin
  pass (P2, P4, P5, P6, P8), **10,000 steps** (the window opens before the remnant cools: centre ε_th/ε_cold 0.39, Σ 28).
- **Disk**: calibrate's grid SCF cold C/O disk (`cold_disk_scene`, shipped K's, T_floor, 64 × 64, bounce), gravity + hydro, heat off,
  M = 1,800 (Σ_max 408, 12 cells ≥ 10) for 10,000–30,000 steps, M = 900 (Σ_max 115) for 10,000.
- Baseline (today's tree): crop 0–5,000 floor 1.136 × 10⁵ / radiated 2.346 × 10⁵ = **48.4 %** (M0-D71's twin figure, reproduced; GPU
  47.5 %); 5,000–10,000 2.058 × 10⁵ / 2.138 × 10⁵ = **96.3 %**; 0–10,000 **71.2 %**. From step ≈ 2,500 the centre (Σ 78, ε_th/ε_cold
  1.4 × 10⁻⁴) draws ≈ 2.1 × 10⁴ per 500 steps, nuclear falls to ≈ 6 × 10², and every deficit appears after P4 in free cells (per-pass
  tally, steps 3,000–4,000: 4.28 × 10⁴ after P4, 0 after P5 and P6). The energy invariant (KE + E_th + E_cold + W + radiated − nuclear −
  floor) holds: −1.71427 × 10⁵ → −1.71476 × 10⁵ (2.9 × 10⁻⁴).

## Candidates measured (twin, linux-pc)
- **N** — P8 borrows a short cell's deficit from its four neighbours' heat above their floor (each gives ≤ ¼) before the floor refills
  the rest (M0-D67's 500-step candidate, run long).
- **D** — dual energy (Bryan et al. 1995's formalism, from memory — UNVERIFIED): Σε_th carried beside E, advected upwind with P4's mass
  flux plus −Π_th ∇·u, P5's and P6's changes added; where the conservative ε_th < η ε_cold (η 0.1), E is re-synced to it and the change
  booked `sync` (signed). This is the Deliver's "energy update for cold-dominated cells that keeps E − ε_cold consistent".
- **H** — held core: a cell with Σ ≥ Σ_hold (10) and ε_th ≤ η ε_cold is held (P8, after the floors); released past 2η or below ½Σ_hold.
  P4 leaves a held cell as it is. A face between a held and a free cell is a reflecting wall: HLL between the free side's predicted face
  and its mirror, with mass, energy and species flux 0 and momentum Π + Σu² + SΣu. No gravitational work crosses it, and the free cell
  keeps its Σg kick. P5, P6 and P8 run on held cells as on any other. A newly held cell's momentum is zeroed; its KE stays in E as heat and
  the momentum is booked. This is the Deliver's "cells held as one object below a resolution threshold", kept on the grid.
- **B** — the floor's refill of a cold cell booked as a term of its own, or a debt left in E — **not run**. Every reader floors ε_th
  (`Eos::thermal`, P4's `read`), so the dynamics and the light are today's by construction. Only the ledger line moves; 71 % of the
  light stays fed by the pump under another name. It is an accounting option, not a cure.

### Crop, 10,000 steps (floor_added, or D's sync, against radiated)
| candidate | 0–5,000 | 5,000–10,000 | 0–10,000 | the remnant |
|---|---|---|---|---|
| baseline | 1.14e5 / 2.35e5 = **48.4 %** | 2.06e5 / 2.14e5 = **96.3 %** | **71.2 %** | centre Σ 78 at T_floor, a pump |
| N | 6.82e4 / 2.16e5 = 31.6 % (borrowed 1.9e5) | 1.72e5 / 1.71e5 = 100.9 % (borrowed 8.1e5) | 62.2 % | pump moves to the neighbours |
| D (η 0.1) | sync 9.31e4 / 2.00e5 = 46.5 % | sync 1.38e5 / 1.61e5 = 85.8 % | 64.0 % | sync replaces the floor 1:1 |
| **H (η 0.1)** | 1.54e3 / 1.25e5 = **1.2 %** | 3.56e3 / 7.70e3 = 46 % | **3.8 %** | first cell held at 1,500, 23 at 2,500, 37 from 3,500 (Σ_max 61, ε_th/ε_cold 0.10 → 0.065); nuclear 0 after 5,000 |
| H (η 10⁻³) | 3.42e3 / 1.60e5 = 2.1 % | 1.57e4 / 1.55e4 = 101 % | 10.9 % | first held at 2,500, 28 by 5,000 (Σ_max 77, cold); the pump ran until then |

H's invariant: −1.71427 × 10⁵ → −1.71495 × 10⁵ (η 0.1, 4.0 × 10⁻⁴) and → −1.71533 × 10⁵ (η 10⁻³, 6.2 × 10⁻⁴), today's order. Under H
the light is the remnant's own heat. At η 0.1, over steps 5,000–10,000, the dense gas's heat falls 2.38 × 10⁴ → 1.97 × 10⁴ while 7.7 × 10³
leaves, and the 3.6 × 10³ of floor goes to free envelope cells five cells off the core (Σ ≈ 1.5, |u| ≈ 2.3 inward): HLL's own
non-balance in the envelope, ≈ 0.7 per step against today's ≈ 41. At η 10⁻³ about half the residual goes to held cells (Σ ≈ 21, u = 0),
which P4 never touches. Which pass short-changes them (P5 or P6) is NOT MEASURED: the per-pass tally was stopped at step 3,000–4,000,
before that state, at the 10-minute line.

### SCF cold disk (gravity + hydro)
| candidate | M 1,800: KE, max \|u\| (Σ > 10⁻³), Σ_max | M 1,800: floor (or sync) after the first 2,000 steps | M 900 |
|---|---|---|---|
| baseline | 1.1–1.4e3, 10–12, 146 → 142 | +2.9e3 to 10,000, +1.13e4 to 30,000 (3.21e5 in the first 50 steps) | floor 1.73e4 → 9.02e4 (+7.2e4 over 2,000–10,000), KE 2.2e4, Σ_max 60 → 22: the core disperses |
| N | 8.2e4, 18, 40 at 10,000 — **unstable** | floor 6.8e5, borrowed 5.4e5 | — |
| D (η 0.1) | 8e2, 5.3–5.6, 118 | sync +2.2e3 to 10,000, +4.5e3 to 30,000 | — |
| D (η 0.01 / 1) | 1.1e3 / 68; Σ_max 143 / 32 (η 1 spreads the disk) | +1.3e4 / +1.5e3 | — |
| **H** (Σ_hold 10) | **0, 0, 408** (the SCF profile, 12 cells held from step 0) | **0** (5.5 × 10⁻² over 30,000: vacuum resets) | **16 held, KE 155 steady, floor +2.1 over 1,000–10,000** |
| H (Σ_hold 50) | 6–8e3, 10–13, 408 (4 held, the Σ 10–50 cells free) | +1.1e3 | — |

Two wall forms were tried and rejected on the way, and both bear on the build:
1. A **pressure-only wall** (§2.8 bounce's flux) with the free cell's own ½Δt·p·Δφ share of gravity's work at the wall. Without its
   partner D = Δt(q − ½(p_i + p_j))Δφ, that share creates energy: +3.4 × 10⁵ unbooked over 10,000 crop steps. Even with no work, a
   pressure-only wall lets gravity drive a free cell into it without bound: the M 900 disk ran to KE 5 × 10⁶, every unit paid by the
   floor. At a held wall the Σu² + SΣu terms are what stop the cell.
2. A **mirrored stencil** for a free cell beside a held one (its slopes read its own mirror, as at a bounce edge) zeroes the cell's
   pressure slope, so gravity is unbalanced. Crop KE rose to 7.3 × 10³ (unmirrored 2.4 × 10²). The free cell keeps its normal slopes.

## What each changes, what it costs on the GPU (estimates — NOT PROVEN, no GPU run)
| | contract | GPU |
|---|---|---|
| B | §2.9 a new energy term (or none: a debt in E); §2.7 [M0-T19] | none |
| N | §2.7 [M0-T19]'s P8 rule | P8 in two dispatches (deficit and heat planes, then the borrow): +1 dispatch, 2 scratch planes |
| D | §2.2 a 15th canonical channel (dumps, the CPU↔GPU layout), §2.3.3–2.3.4, §2.9 `sync` | one more plane through every pass, P4's flux 14 → 15 components (≈ +7 % P4 bandwidth), P8 re-sync |
| **H** | §2.3.4 (held cells and their walls), §1.4.2 / §0.5 [M0-D64], [M0-D68] (no work across a held wall), §2.7 (`sigma_hold` 10, `eta_hold` 0.1, release at 2η or ½Σ_hold; [M0-T19]'s P8 rule writes the mask and zeroes a newly held cell's momentum), §2.9 a momentum term `held`, §1.3.2 (P8 writes the mask, P4 reads it), §2.12 (the mask in dumps — it carries hysteresis), §5 a new row (below) | one mask plane (≈ 1 MB at 600 × 400) written by P8 from the ε_th and ε_cold it already computes; P4's flux kernel reads two mask words per face and, at a wall, runs one HLL against a mirror; P4's update returns early in held cells; one more fixed-order block reduction for `held` momentum. Estimate ≤ 2 % of P4 |

## A "remnant at rest" check H could hold (bounds proposed from the measurements above)
- Disk, twin and GPU, SCF cold C/O at M 900 and M 1,800, 10,000 steps: held cells' u = 0 exactly; floor_added over steps 1,000–10,000
  ≤ 10 (measured 2.1 and 0; today 7.2 × 10⁴ and 2.9 × 10³); KE at 10,000 ≤ 2 × KE at 1,000 (155 → 154; 0 → 0).
- The crop and the GPU's step-450,000 window, 10,000 steps: floor_added ≤ 15 % of radiated (the lead's M0-D67 bound; measured 3.8 % at η
  0.1, 10.9 % at η 10⁻³; today 71.2 %).
- The neutron_star row's |v_r| ≤ 0.05 √(GM/R) (M0-D79's dropped term, flagged here): held cells read u = 0, but the row's v_r over the
  object's centre is NOT MEASURED under H.

## What H leaves open — the build's to measure (none measured here)
1. **Collapse.** An iron core is cold and dense. If held, it cannot compress and core collapse never fires. The hold must exclude
   cells whose N_Fe capture rate is > 0, or release a held set whose mass passes its ceiling. Read on M0-D79's 4,572 life and R6's 20 M₀.
2. **Accretion.** Walls stop inflow, so a held proto-neutron star cannot take fallback past M_tov and a heavy star could end neutron_star
   instead of black_hole. Proposed: let a face's inflow into a held cell through (one way), with its mass, energy and momentum, the
   momentum booked to `held`.
3. **Motion.** A held remnant cannot move: forces on it go to `held` momentum (crop, 10,000 steps: (78.6, −226) at η 0.1, (162, −832) at η 10⁻³).
   A kicked or orbiting remnant needs whole-cell moves, or a release.
4. **Resolved remnants.** G-CORE's resolved hold (k1e × 6, R₉₀ 18.8 cells) would be held too, which makes that test trivial. The criterion
   may need an unresolved test (Σ falling by ≥ a factor per cell), or G-CORE runs with the hold off.
5. **Δt.** Held cells still set Δt through their cold sound speed. Dropping them from P9's reduction is an unmeasured speed-up.
6. **§2.8's bounce edge** has the same pressure-only flux that ran away against a held wall under gravity. Gas pushed onto a bounce edge
   by gravity may draw the floor the same way. Seen at held walls only, not at the world edge — NOT MEASURED there.
7. **η**: 0.1 catches the core warm and lets it cool on the grid; 10⁻³ lets the pump run until the core is cold. Both are tuned on one dump.

## Recommendation
H, at η 0.1 and Σ_hold 10, with reflecting HLL walls that do no work. It goes to M0-D71, re-opened, as its Deliver. Items 1–2 go in the same
build, measured on the GPU lives before anything is ruled final (a PLAN + BUILD split if past one task's ceiling). Items 3–5 are recorded
in the contract as the hold's limits.

## Ruling
The lead, 2026-10-28, at M0-R11: «H, held core (Recommended)» — M0-D71 re-opened to build H on GPU and twin, collapse and accretion
(items 1–2) measured in the same build on the GPU lives, a PLAN + BUILD split if past one task's ceiling; «η 0.1 (Recommended)» — the
build starts from η 0.1, Σ_hold 10; «Measured bounds (Recommended)» — the check: the SCF disks at M 900 and 1,800, 10,000 steps, held
u = 0, floor_added over steps 1,000–10,000 ≤ 10, KE(10,000) ≤ 2 × KE(1,000); the step-450,000 window over 10,000 steps, floor_added
≤ 15 % of radiated. Recorded in M0-D71's block (its Deliver re-written to it, the lead's word in M0-R11's Deliver). Its place, the
same day: «Game first (Recommended)», «move the deffered tasks where they will run» — M0-D71 runs at the head of Phase 16, after the
game is on screen (Phases 12–15).
