# M0-R14 — Rotation and jets: what spinning stars and the jets of black holes and neutron stars would take in this engine

Box: laserax-ai-server, NVIDIA GeForce RTX 4090 (R18). Binary: the server's `build/target/release/sandbox-reactions`, built
2026-11-06 10:21:52; its crates/, assets/, Cargo.toml and Cargo.lock match linux-pc's tree file for file by md5 (only old
`crates/sr-engine/build/calibrate_runs` outputs differ). Date: 2026-11-06, model=claude-opus-5-5 level=high (`rung_record.py now`).
Raw lines: logs/M0-R14.log, logs/M0-R14.spinread.log. Probe sources: tasks/M0-R14_probe/ (spin.py, spinread.py,
massive_disk.json). Commands: tasks/M0-R14.md. Lives and dumps: the server's /tmp/r14 (885 MB), removable.

## Problem
The lead, 2026-11-01, during M0-R12: « oh and also file something so that we can have rotation, id like for us to be able to see
the relativistic jets from black holes and neutron stars. MAybe they could rotate. If thats not too much work ». Then, at the
question tool: « R block in M0, research only (Recommended) ».

**What rotation is today.**
- **The gas carries momentum.** Σu and Σv are state (§2.2). P4's finite volumes conserve linear momentum; angular momentum is
  conserved only to the scheme's truncation error, because a Cartesian scheme does not conserve it by construction (S6).
- **Nothing gives a spin.**
  - The brush paints at rest (§1.12).
  - The presets drop at rest (§2.10).
  - A scene's `disk` or `rect` takes one velocity `v`, a translation, and scenes are for tests only (§2.12.1).
  - No line of crates/ names angular momentum (grep: `angular|spin|vortic|rotat` hits only SHA-256 rotations and the rates' Omega).
  - So a player's world starts with L_z = 0. Only an erase of moving gas, or gas leaving through an open edge, can give it net spin.
- **The remnants cannot spin.**
  - A sink carries position, velocity and mass, and no angular momentum (§2.6).
  - Its accretion test is energy only: bound gas within r_acc is swallowed whatever its angular momentum.
  - The neutron star's core is held (§2.7 [M0-TG2], [M0-R13]). A newly held cell's momentum is zeroed, and no face passes mass
    into a held cell ([M0-D71f]). So the core can neither rotate nor accrete.
- **The ledger books no angular momentum** (§2.9: mass, energy, momentum). A spin's drift cannot be graded today.

**What a spun cloud does now.** This was measured on three lives. Each is the Massive preset's cloud as a scene disk (a 56.464,
Σc 1.0, T₀ 6.6856, so no calibration is read; the shipped calibration is stale until M0-V14). One step was run, then the dump
was spun about its centre by tasks/M0-R14_probe/spin.py and run to t 30. β is the start's rotational support at every radius:
rigid rotation in a Maclaurin disk's quadratic potential, Ω² = β π² G Σc / (2a) (SM-E6). All runs had 0 warnings and exit 0.

| life | Ω (edge speed) | events | stage at t 30 | R50 at t 30 | densest Σ | the core (r ≤ 4) at t 30 |
|---|---|---|---|---|---|---|
| control, β 0 | 0 | ignition (fuel) t 3.92 · core_collapse 7.67 · supernova 7.75 | **neutron_star** from t 21.57 | 3.1 | 876.9 (held) | X_n 0.65; held, so no spin; the gas around it Ω 1.2 |
| slow, β 0.01 | 0.0296 (1.67) | ignition (fuel) 3.94 · core_collapse 7.88 · supernova **9.53** | **iron_core** — no neutron star | 7.9 | 117.9 | X_n 0.08, Ω 6.7, centrifugal/gravity 1.37: a spinning ring |
| spun, β 0.2 | 0.132 (7.47) | ignition **by light 12.38** — no collapse | main_sequence from t 12.75 | 17.1 | 5.6 | Ω 0.63, support 0.09 |

How to read the table:
- **Angular momentum holds until a core collapses.** L_z about the centre of mass moves 0.4 % over the spun life (2,633 steps,
  no collapse). In the slow life it moves 1.9 % up to step 750.
- **After the collapse it does not hold.** The slow life's L_z goes ×2.9, from 2.5 × 10⁵ to 7.3 × 10⁵. The control's goes from
  0 to 2.1 × 10⁵. Both lives lose mass through the open edge (956 and 2,251 sb). The dumps cannot separate a numerical torque from
  the L that escaping ejecta carry out, because the ledger books none (UNVERIFIED cause).
- **Any spin changes the ending.** At 1 % support the collapse spins the core up past centrifugal balance: it sits as a ring of
  Σ ~120, and no neutron star forms by t 30. At 20 % the cloud stops as a fat disk that burns: a star, but never the Massive's life.
- **The fate readout predicts by mass alone.** It says «supernova» for all three lives.
- **Why nature differs.** A real collapse contracts far more, yet real cores arrive slow. They have shed their angular momentum
  through magnetic torques, disks and the outflows themselves (S5: magnetic torques make collapsar-ready cores hard to produce). The
  sandbox has none of those channels.
- **A rigidly rotating, axisymmetric disk draws the same in Glow as a still one**, since colour follows temperature (§1.11). This is
  my reading; no frames were captured.

**What jets would need.**
- **Geometry.** The world is a thin sheet seen face-on (§1.4.1). Gas in the sheet can only spin about the axis out of the screen,
  so a disk's jets point at the player and away. No in-plane axis exists.
- **A launching engine.**
  - In nature, jets are magnetic. Blandford–Znajek extracts a spinning hole's energy through the field threading it,
    P ∝ Ω_H² Φ², up to P ≃ F(a) φ² Ṁc² with F(1) ≃ 2 (S2). Disk winds do the same on the disk (Blandford–Payne).
  - The engine has no magnetic field.
  - A jet also needs gas with enough angular momentum to form a disk outside the hole's innermost stable orbit, j ≳ r_g c (S5).
- **Neutron-star jets** exist in accreting X-ray binaries. They are about 10× weaker than a black hole's at the same X-ray
  luminosity, but just as fast: Cir X-1's Γ > 15 (S3).
- **Speed.** c_sb = 400 cells/t.u. (physics.json); today's gas is at ≲ 22 (reports/lives.md). A jet that flows on the grid sets the
  step by CFL, 0.4 cells a step (§2.3.3):
  - At SS 433's 0.26c (S4) that is 104 cells/t.u., so Δt ≤ 0.0038 against the control's mean 0.0058 (30/5,188): about 1.5–2×
    the steps while it flows.
  - At 0.9c it is 360 cells/t.u., about 5×.
  - P4 is Newtonian, so at 0.26c the kinetic energy reads 8 % low (½β² 0.0338 against γ − 1 0.0365, my arithmetic); near c it is
    meaningless.
  - A jet that leaves the sheet costs no step at all.

## Candidates
Each candidate states which of its parts are physics and which are drawing (the Adversarial line). The costs are in blocks of
this plan's size.

**Rotation (the prerequisites of any jet):**

| | what | physics | drawing | cost | place |
|---|---|---|---|---|---|
| S1 | **Spin at the drop.** A preset choice: still · slow (β 0.01) · fast (β 0.2) | the momentum the drop writes (P0, booked as the preset's) | none | 1 (scene, strings §4.5, twin, case) | M0 or later |
| S2 | **A Stir tool.** The brush adds Σ Ω × r about its centre per frame held | the momentum and energy added, booked under `tools` | none | 1 | later |
| S3 | **Seeing it.** A Motion view: colour by the velocity along a fixed in-plane line of sight, blue toward and red away, as rotation is measured in nature. A spin readout: L and the period, in days through the age clock | the view and the readout read the state | the colour map | 2 | M0 or later |
| S4 | **An angular-momentum ledger.** L_z: grid, sinks, escaped, tools; a G-CONS row | its booking | none | 1 | with S1 |
| S5 | **Angular-momentum transport.** An α-viscosity (Shakura–Sunyaev), the sub-grid stand-in for the magnetic turbulence that lets real disks feed their centre; then the fate readout learns spin | a new law in P4 or its own pass | none | research + 2–3 | later |
| S6 | **Spinning remnants.** A sink's J and its Kerr a = c_sb J/(GM²) from the gas it swallows, and a disk gate on accretion (j < j_isco). A held core that turns rigidly at its Ω instead of being zeroed | J's booking, the gate, the hold's rotation | the a readout | 1 (sinks) + 2 (the hold: a design against [M0-TG2]/[M0-R13]) | later |

**Jets:**

| | what | physics | drawing | cost | place |
|---|---|---|---|---|---|
| J1 | **A sub-grid jet off the sheet, seen face-on.** A sink fed by gas with j ≥ j_isco ejects f_m (0.1–0.4) of what it accretes (S1's model, Federrath 2014). Its power is η(a) Ṁc² by S2's form. The mass, energy and L leave the sheet, booked `jet` | the gate on j, the ejected mass, energy and L, power ∝ Ṁ, the a² dependence | the beam: a beamed spot and flare at the hole, since the jet points at the player | 3, after S4 + S6 | later |
| J2 | **J1 + jet matter above the sheet in a tilted view.** Ejected matter as particles moving in 3D under the sheet's own 3D gravity (§1.4.1's law already is 3D). The camera pitches the sheet so the two jets read as beams up and down | J1's, plus the particles' motion | the tilt and the particles' glow | 5–7, after J1 | later |
| J3 | **An in-plane jet.** J1's launch into two cones in the sheet, along an axis; the hydro carries it (bow shock, cocoon); capped at 0.26 c_sb | the launch, the flow, the shocks | **the axis** — the sheet has none, so it is chosen, not physics | 3 + S4, S6; Δt 1.5–2× while it flows | later, if ever |
| J4 | **A side-on world.** Axisymmetric (r, z): true spinning stars, with v_φ as a channel and jets along the axis on screen | all of it | none | a milestone of its own (L): gravity's ring kernel, cylindrical hydro and heat, the twin, the calibration. Painting means rings (SM M2's option C, rejected for the sandbox) | a later milestone, a second mode |
| J5 | **Magnetic fields.** B channels, an MHD solver and div-B control | the true launch | none | a milestone. A face-on sheet still cannot launch along z (it shows braking only); GRMHD is out of a 60-fps sandbox's reach (opinion) | after J4, if ever |

**Neutron stars (any J).** The held core accretes nothing ([M0-D71f]'s walls). A neutron-star jet waits on S6's hold design and on
an accretion rate into the hold. Black holes come first.

## Evidence
- **E1 · The three lives** (measured, laserax-ai-server, RTX 4090). Table in § Problem; every dump's line is in
  logs/M0-R14.spinread.log. Wall times: control 101 s (5,188 steps), slow 56 s (4,508), spun 16 s (2,632).
- **E2 · Nothing gives a spin** (source read).
  - §1.12: the brush paints «at rest (momentum unchanged)». §2.10: the cloud is «at rest». §2.12.1: an object takes one `v`.
  - grep of crates/ for angular momentum: none.
  - **NOT PROVEN (source only)** that no other path exists.
- **E3 · The remnants cannot spin** (source read): §2.6 (the sink's state), §2.7 [M0-TG2] (a held cell's momentum zeroed),
  §1.4.2 [M0-D71f] (no mass into a held cell). The control's densest Σ stays 876.9 from step 1,000 to 5,189, so its core is held.
- **E4 · The ledger** (§2.9) books mass, energy and momentum, and no angular momentum.
- **E5 · Speeds:** c_sb 400 (assets/physics.json); CFL 0.4 cells a step (§2.3.3, §1.3.3); the control's mean Δt 0.0058 (E1).
- **S1 · Federrath et al. 2014**, «Modeling jet and outflow feedback during star cluster formation», ApJ 790, 128
  (arXiv:1406.3625). The sub-grid outflow is bound to the sink and launched along its angular momentum, in two 30° cones. It
  carries a fraction f_m of the accreted mass, 0.3 by default (0.1–0.4 in other work), and returns f_a = 0.9 of the accreted
  angular momentum. It converges at a thousandth of the resolution a resolved jet would need.
- **S2 · Tchekhovskoy, Narayan & McKinney 2011** (arXiv:1108.0412). Blandford–Znajek power P ≈ (κ/4π) Ω_H² Φ² f(Ω_H): no field
  or no spin, no jet. In magnetically arrested disks P ≃ F(a) φ² Ṁc², F(0.5) ≃ 0.2, F(1) ≃ 2. McKinney 2005
  (arXiv:astro-ph/0506367) gives the GRMHD-calibrated forms.
- **S3 · Migliari & Fender 2006**, MNRAS 366, 79 (arXiv:astro-ph/0510698). Neutron-star X-ray binaries' jets are about an order of
  magnitude less powerful than black holes' at the same L_X; the power is linear in the accretion rate in both. Cir X-1's jets have
  Γ > 15, so relativistic jets do not need a black hole.
- **S4 · SS 433:** antiparallel jets at 0.26c (β = 0.2647 ± 0.0008), precessing every ~163 days (Margon 1984; arXiv:1606.01240).
- **S5 · The collapsar** (MacFadyen & Woosley 1999; reviewed in arXiv:1105.4193, arXiv:astro-ph/0609142). A jet needs a black
  hole and enough angular momentum to form a disk outside R_ISCO, j ≳ r_g c. With magnetic torques, stellar models find
  collapsar-ready cores hard to make (Woosley & Heger 2006).
- **S6 · Byerly et al. 2014** (arXiv:1404.5942): Cartesian advection conserves linear momentum, not angular momentum; their
  hybrid scheme advects the cylindrical components to conserve it.

## Recommendation
**Route rotation and jets past M0 as one later milestone, «Rotation and jets»; build nothing in M0.** (Opinion, on E1–E5.)

Why:
- **Spin is a second fate axis, not a decoration.** E1 shows even 1 % support removing the neutron star, and 20 % removing the
  collapse. Any spin control in M0 reopens the endings, the fate readout and the G-rows that grade them, a week before M0-V14's
  recalibration.
- **Spin alone does not show.** A smooth spinning disk draws as a still one, so S1 needs S3 and S4 to be worth seeing. That is
  4 blocks before any jet.
- **A jet needs S4 and S6 first.** It also needs a view the sheet does not have: face-on, J1 is a spot.

The milestone, in order:
1. S4, S1, S2, S3 — spin you can give, see and grade.
2. S6 — remnants that keep and gain spin.
3. S5's research — whether spun clouds should still reach their mass's ending (transport), or whether spin should change the
   fate as E1 shows.
4. J1, then J2 — the jets seen as beams.

J4 (a side-on world) is a mode of its own, for the lead to place separately if wanted. J3's chosen axis breaks «never a script».

The alternative in M0 is S1 + S3 + S4 (≈ 4 blocks, off by default so the calibration is untouched), with a fate line that says
spin may change it. It lets the player see a spinning star in M0, but no jet, and it moves the remnant work no closer.

## Ruling
The lead at M0-R14, 2026-11-06, question tool (PLAYBOOK §12, one call, two questions), verbatim:

- **Where:** «Later milestone (Recommended)». The option was shown as: "Build nothing in M0. One line in milestones.md routes a
  'Rotation and jets' milestone past M0: spin you can give, see and grade, then spinning remnants, then jets. M0's endings and the
  M0-V14 recalibration are untouched."
- **The jets' look:** «Tilted view (Recommended)». The option was shown as: "The sheet is drawn tilted, and jet matter is ejected
  off the sheet as particles moving under real 3D gravity, so you see two beams up and down. Launched by the remnant's accretion and
  spin. About 8–10 blocks after the rotation work."
- **Not picked:** «Spin in M0, jets later», «Spot, face-on only», «Side-on world mode» and «In-plane jets».

The contract is unchanged; no [M0-R14] tag.

## Blocks filed
None in M0. milestones.md's Deltas gains the routing line (2026-11-06 · M0-R14). The later milestone carries § Recommendation's
order: S4, S1, S2, S3, then S6, then S5's research, then J1 and J2, with J2's tilted view as the lead's pick. Its sizing and
placement are for the milestone that follows M0 (M0-TZ places it with the lead).
