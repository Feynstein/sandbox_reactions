# Star physics research — M0-R2a (2026-10-08)

**Summary.** A star's life is three clocks nested inside one another: the **dynamical** (collapse,
bounce: about half an hour for the Sun), the **thermal** (contraction, swelling: about 10^7 years)
and the **nuclear** (burning: about 10^10 years), with τnuc ≫ τKH ≫ τdyn (E1). Every stage of
B4–B6 is one of these clocks running while the others hold still. The endings are ordered by mass:
no ignition below ~0.08 Suns, a white dwarf up to ~8–9 Suns (published range 6–11), a supernova
leaving a neutron star from ~9–10 to ~20–25 Suns, a black hole above ~20–25 Suns — the upper
boundary is soft and, in detailed models, **not even monotonic** (E5, E6, E3). I11's four facts
from memory hold, two of them with a correction: the "10 billion years" is the main sequence only
(cloud to white dwarf is ~11 billion years, my arithmetic from E2), and a core collapse takes
**milliseconds to under a second** (E3, E8), so "in about a second" is the upper bound. Nine
elements cover the burning ladder — H, He, C, O, Ne, Mg, Si, S, Fe — with Ni-56 a tenth, optional.
What the squeeze must keep is a list of **orders and ratios** (§Squeeze K1–K9), never the real
values; real values appear only as the readouts' anchors. Proposed translation: one factor for mass,
anchor-to-anchor for temperature, a stage clock for age (with M0-R3).

Written to PLAYBOOK §4 Rule 7: every claim lives once in `## Evidence` with its URL and access date
on its line; the sections cite claims by `E<n>`. **Opinion** and **my arithmetic** are labelled;
every claim no source confirmed is in `## UNVERIFIED`. No local probe was run (pure research), so no
box is named for a measurement; the report was written on linux-pc.

## Evidence (each claim with its source)
**Stellar structure and time scales**
- E1 · Pols, *Stellar Structure and Evolution* (Utrecht lecture notes) ch. 2: the Sun's dynamical timescale τdyn ≈ 1600 s, "about half an hour"; its thermal timescale ≈ 1.5 × 10^7 yr; the nuclear timescale ≈ 10^10 yr; "τnuc ≫ τKH ≫ τdyn"; the nuclear timescale is "two to three orders of magnitude larger than the thermal timescale"; stars are in thermal equilibrium for "> 99 %" of their lives; virial theorem Eint = −½ Egr: "a star that contracts quasi-statically must get hotter"; fusion converts a fraction φ ≈ 0.007 of rest mass for hydrogen, "smaller by a factor 10 or more" for helium and heavier fuels — https://www.astro.ru.nl/~onnop/education/stev_utrecht_notes/chapter1-4.pdf (accessed 2026-10-08)
- E2 · Pols ch. 9–11: cloud fragments above the Jeans mass (M_J ~ 10^3–10^4 Suns in molecular clouds) collapse in near free fall on a timescale "of the order of millions of years"; fragmentation goes on to < 0.1 Suns; the pre-main-sequence star contracts on the thermal timescale, internal temperature T ∝ M^(2/3) ρ^(1/3), lifetime τPMS ≈ 10^7 (M/M☉)^−2.5 yr; a 1-Sun star exhausts central hydrogen (Xc = 10^−3) after 9 Gyr, then the subgiant phase "lasts about 2 Gyr"; after the He flash, smaller flashes during ≈ 1.5 Myr; core helium burning "about 120 Myr" (low mass), about 22 Myr in a 5-Sun star; the thermally-pulsing AGB lasts 1–2 × 10^6 yr, set by mass loss of 10^−7–10^−4 Suns/yr that removes the whole envelope; the carbon flash "probably never happens in AGB stars, even when the total mass is 8 M⊙"; white dwarfs cool by Mestel's law, crystallisation slows the cooling after ~2 Gyr, cooling times > 1 Gyr below 10^−3 L☉ — https://www.astro.ru.nl/~onnop/education/stev_utrecht_notes/chapter9-11.pdf (accessed 2026-10-08)
- E3 · Pols ch. 12–13: M_up ≈ 8 Suns for carbon ignition (C-O core > 1.06 Suns; core temperature > 5 × 10^8 K); above M_ec ≈ 11 Suns fuels burn to an Fe core that collapses; Table 12.1 (15 Suns, from Woosley et al. 2002) — central T (10^9 K) / timescale: H 0.035 / 1.1 × 10^7 yr, He 0.18 / 2.0 × 10^6 yr, C 0.83 / 2.0 × 10^3 yr, Ne 1.6 / 0.7 yr, O 1.9 / 2.6 yr, Si 3.3 / 18 d; products: H→He, He→C,O, C→O,Ne, Ne→O,Mg, O(,Mg)→Si,S, Si(,S)→Fe,Ni; above ~5 × 10^8 K neutrino losses dominate, so lifetimes run "from several 10^3 years for C-burning to about a day for Si-burning"; energy per gram ≈ 4.0, 1.1, 5.0 and 1.9 × 10^17 erg/g for C-, Ne-, O- and Si-burning; a ~1.4-Sun core collapses from ~3000 km to ~20 km; "the collapse only takes a few milliseconds", neutronization 3–10 s; SN 1987A ejected ≈ 0.07 Suns of 56Ni; maximum neutron-star mass "probably … 2–3 M⊙"; the NS/BH boundary "probably in the range 20–25 M⊙" and the initial-to-remnant relation possibly "non-monotonic" — https://www.astro.ru.nl/~onnop/education/stev_utrecht_notes/chapter12-13.pdf (accessed 2026-10-08)
- E4 · Pols ch. 5–6: for simple approximations ε_pp ∝ X² ρ T^4 and ε_CNO ∝ X X14 ρ T^18 (ν from 23 to 13 for T7 = 1–5); the 3α rate has ν ≈ 40 at T8 ≈ 1 — https://www.astro.ru.nl/~onnop/education/stev_utrecht_notes/chapter5-6.pdf (accessed 2026-10-08)
- E23 · Free-fall time t_ff ≈ 0.5427/√(Gρ), "≃ 35 min / √ρ (g cm^−3)" — https://en.wikipedia.org/wiki/Free-fall_time (accessed 2026-10-08)
- E24 · Kelvin–Helmholtz timescale of the Sun ≈ 2.874 × 10^14 s ≈ 8.9 Myr (with a different structure constant than E1: the same order) — https://en.wikipedia.org/wiki/Kelvin%E2%80%93Helmholtz_mechanism (accessed 2026-10-08)

**The endings and their mass thresholds**
- E5 · Heger, Fryer, Woosley, Langer & Hartmann 2003, *How massive single stars end their life* (ApJ 591, 288): stars below ~9 Suns end as white dwarfs; ~9–10 Suns form O-Ne cores that collapse by electron capture to a neutron star or lose their envelopes as white dwarfs; "Above ∼ 10 M⊙ core collapse is the only alternative"; the white-dwarf limit "Estimates range from 6 to 11 M⊙"; black holes by fallback from a helium core of ~8 Suns ("a ≳ 25 M⊙ main sequence star"), directly above a helium core of 15 Suns ("40 M⊙ main sequence star with no mass loss"); "a baryonic remnant mass of over 2.0 M⊙ will produce a black hole"; "stars up to at least 25 M⊙ do explode"; fallback turns a neutron star into a black hole "within one day" — https://arxiv.org/abs/astro-ph/0212469 (accessed 2026-10-08)
- E6 · Sukhbold, Ertl, Woosley, Brown & Janka 2016 (solar metallicity, 9–120 Suns, calibrated neutrino-driven explosions): "Many progenitors with extended core structures do not explode, but become black holes, and the masses of exploding stars do not form a simply connected set"; neutron-star mean mass near 1.4 Suns; black holes ~9 Suns (helium core implodes) or ~14 (whole star) — https://arxiv.org/abs/1510.04643 (accessed 2026-10-08)
- E7 · Type II supernova: core-burning stages of a 25-Sun star (Woosley & Janka 2005, Nature Physics 1, 147) — T (K) / duration: H 7 × 10^7 / 10^7 yr, He 2 × 10^8 / 10^6 yr, C 8 × 10^8 / 10^3 yr, Ne 1.6 × 10^9 / 3 yr, O 1.8 × 10^9 / 0.3 yr, Si 2.5 × 10^9 / 5 d; a progenitor "must be at least 8 times, but no more than 40 to 50 times" the Sun's mass; the outer core reaches "up to 70000 km/s (23% of the speed of light)", collapse "over a timescale of milliseconds"; below about 20 Suns the remnant is a neutron star, above it collapses to a black hole — https://en.wikipedia.org/wiki/Type_II_supernova (accessed 2026-10-08)
- E8 · OpenStax *Astronomy 2e* §23.2: "In less than a second, a core with a mass of about 1 M_Sun, which originally was approximately the size of Earth, collapses to a diameter of less than 20 kilometers"; infall reaches "one-fourth the speed of light"; "The fusion of silicon into iron turns out to be the last step in the sequence of nonexplosive element production" — iron is the most tightly bound nucleus; the upper mass of a neutron star "might only be about 3 M_Sun" — https://openstax.org/books/astronomy-2e/pages/23-2-evolution-of-massive-stars-an-explosive-finish (accessed 2026-10-08)
- E11 · Stellar evolution: protostars below ~0.08 Suns "never reach temperatures high enough for nuclear fusion of hydrogen to begin"; a Sun-like star stays on the main sequence "for about 10 billion years"; full carbon burning needs ~8–9 Suns; "Stars with around ten or more times the mass of the Sun can explode in a supernova" — https://en.wikipedia.org/wiki/Stellar_evolution (accessed 2026-10-08)
- E12 · Chandrasekhar limit "about 1.44 M☉"; "Stars above the limit can become neutron stars or black holes" — https://en.wikipedia.org/wiki/Chandrasekhar_limit (accessed 2026-10-08)
- E13 · Tolman–Oppenheimer–Volkoff limit 2.01–2.17 Suns from GW170817; models with baryon repulsion 2.2–2.9 Suns — https://en.wikipedia.org/wiki/Tolman%E2%80%93Oppenheimer%E2%80%93Volkoff_limit (accessed 2026-10-08)
- E14 · White dwarf: typical mass 0.5–0.7 Suns (observed 0.17–1.33); radius 0.8–2 % of the Sun's; density ~10^6 g/cm³; progenitors up to ~8–10 Suns; C-O cores typical, O-Ne from heavier progenitors, He from the lightest; hot ones > 30,000 K, coolest observed ~3050 K; a carbon white dwarf took ~1.5 Gyr to cool to 7140 K; ~97 % of Milky Way stars end as white dwarfs — https://en.wikipedia.org/wiki/White_dwarf (accessed 2026-10-08)
- E15 · Planetary nebula: ~10,000 years from formation to recombination, "a few tens of millennia" at most; made by stars of ~1–8 Suns; central star > 30,000 K, rising to ~100,000 K; nebula mass 0.1–1 Suns; the AGB wind removes 50–70 % of the star's mass — https://en.wikipedia.org/wiki/Planetary_nebula (accessed 2026-10-08)
- E16 · Red-giant branch: for a 1-Sun star "approximately 2 billion years from the time that hydrogen was exhausted in the core"; stars reaching the He flash have a helium core of "almost 0.5 M☉"; the foot of the branch near 5,000 K; stars of ~0.4–12 Suns become red giants; ≤ 2 Suns ignite helium explosively — https://en.wikipedia.org/wiki/Red-giant_branch (accessed 2026-10-08)
- E19 · Neutron star: radius "on the order of 10 kilometers"; typical mass ~1.4 Suns; progenitors "between 10 and 25 M☉ or possibly more"; newly formed surface temperature "ten million kelvins or more" — https://en.wikipedia.org/wiki/Neutron_star (accessed 2026-10-08)
- E20 · Schwarzschild radius r_s = 2GM/c²; the Sun's ≈ 3.0 km — https://en.wikipedia.org/wiki/Schwarzschild_radius (accessed 2026-10-08)

**The Sun, the main sequence, the reactions**
- E9 · NASA Sun fact sheet: mass 1.9884 × 10^30 kg; volumetric mean radius 695,700 km; central temperature 1.571 × 10^7 K; central density 1.622 × 10^5 kg/m³; effective temperature 5772 K; luminosity 3.828 × 10^26 W — https://nssdc.gsfc.nasa.gov/planetary/factsheet/sunfact.html (accessed 2026-10-08)
- E10 · Main sequence: τMS ≈ 10^10 yr (M/M☉)^−2.5; the Sun's main-sequence life "roughly 10^10 years"; pp and CNO equally efficient at 18 MK; minimum ~0.08 Suns; stars under 0.1 Suns may last "over a trillion years" — https://en.wikipedia.org/wiki/Main_sequence (accessed 2026-10-08)
- E17 · Triple-alpha: significant carbon production once the centre reaches 10^8 K; power "approximately proportional to the temperature to the 40th power, and the density squared"; in a degenerate core it runs away as the helium flash — https://en.wikipedia.org/wiki/Triple-alpha_process (accessed 2026-10-08)
- E18 · CNO cycle: self-maintaining from ~15 × 10^6 K, dominant from ~17 × 10^6 K; dominant in stars above ~1.3 Suns — https://en.wikipedia.org/wiki/CNO_cycle (accessed 2026-10-08)
- E25 · Proton–proton chain: dominant at the Sun's core temperature; the pp-I branch dominates at 10–18 MK — https://en.wikipedia.org/wiki/Proton%E2%80%93proton_chain (accessed 2026-10-08)
- E21 · Alpha process: the α-capture ladder C → O → Ne → Mg → Si → S → Ar → Ca → Ti → Cr → Fe → Ni; "56Ni is formed and decays into 56Fe"; silicon burning reaches the 56Ni peak by photodisintegration of 28Si — https://en.wikipedia.org/wiki/Alpha_process (accessed 2026-10-08)
- E22 · The Sun's composition by mass: "73.46% hydrogen, 24.85% helium, 0.77% oxygen", traces of heavier elements; formed ~4.6 Gyr ago — https://en.wikipedia.org/wiki/Sun (accessed 2026-10-08)

## Stages (B1–B7: the life M0-TI fixed, stage by stage)
Columns: what drives the stage, its real time scale, and the condition that ends it — written as a
physical state the readout can recognise, since labels watch and never drive (I2). Times are real;
the sandbox keeps only their order (§Squeeze).

**Common start (every mass)**
| # | Stage | What drives it | Real time scale | Ends when |
|---|---|---|---|---|
| S1 | Cloud collapse | Self-gravity beats gas pressure once the cloud exceeds the Jeans mass; the gas radiates freely, so early collapse is isothermal; the cloud fragments (E2) | Free fall, "millions of years" at cloud densities (E2); t_ff ∝ ρ^−1/2 (E23) | The centre turns opaque, heats and stops falling: a hydrostatic protostar (E2) |
| S2 | Protostar, pre-main-sequence contraction | Contraction on the thermal clock; by the virial theorem a contracting star heats, T ∝ M^(2/3) ρ^(1/3) (E1, E2) | τPMS ≈ 10^7 (M/M☉)^−2.5 yr (E2) — ~10 Myr for the Sun's mass, a few thousand years at 25 Suns (my arithmetic) | The core reaches hydrogen ignition (~10^7 K, E25, E9) and fusion power balances what the star radiates — or, below ~0.08 Suns, never does: a failed star that cools (E11, E10) |
| S3 | Ignition | Fusion's steep temperature law: ε_pp ∝ T^4, ε_CNO ∝ T^18 (E4) | An event, not a phase (B11's "ignition") | Fusion carries the luminosity (thermal equilibrium) — the main sequence starts |
| S4 | Main sequence | Core H → He (pp chain in the Sun; CNO above ~1.3 Suns, E18) in hydrostatic and thermal equilibrium; burning is a thermostat: hotter → more power → expansion → cooler (E1, E3) | τMS ≈ 10^10 (M/M☉)^−2.5 yr (E10): 9 Gyr for 1 Sun (E2), 1.1 × 10^7 yr for 15 Suns (E3) | Core hydrogen exhausted — Pols marks it at a central H fraction of 10^−3 (E2) |

**Up to ~8–9 Suns — the white dwarf (B4)**
| # | Stage | What drives it | Real time scale | Ends when |
|---|---|---|---|---|
| S5 | Subgiant → red giant | H burns in a shell around an inert He core; the core contracts and grows, the envelope swells and cools to ~5000 K at the branch's foot (E2, E16) | ~2 Gyr for 1 Sun from core H exhaustion (E2, E16) | The core reaches ~10^8 K and helium ignites (E17) — at ≤ 2 Suns in a degenerate ~0.5-Sun core, as the **helium flash** (E16) |
| S6 | Core helium burning | Triple-alpha He → C, then C + He → O; ε ∝ T^40 (E4, E17) | ~120 Myr at low mass, ~22 Myr at 5 Suns (E2) | Core helium exhausted: an inert C-O core |
| S7 | Asymptotic giant branch → planetary nebula | Two burning shells, thermal pulses, a wind of 10^−7–10^−4 Suns/yr that strips the whole envelope (E2); the ejected shell glows lit by the exposed core (E15) | TP-AGB 1–2 × 10^6 yr (E2); the nebula ~10^4 yr (E15) | The envelope is gone; the bare core (> 30,000 K, rising to ~100,000 K, E15) stops burning |
| S8 | White dwarf | No fusion; electron degeneracy holds it up; it only cools (E14, E2) | Cools over Gyr: > 1 Gyr below 10^−3 L☉, crystallising after ~2 Gyr (E2) | Never ends in the sandbox's sense — a cooling remnant, 0.5–0.7 Suns, Earth-sized, ~10^6 g/cm³ (E14); stable only below 1.44 Suns (E12) |

**From ~9–10 Suns — the supernova and the neutron star (B5)**
| # | Stage | What drives it | Real time scale | Ends when |
|---|---|---|---|---|
| S5′ | Supergiant, the burning ladder | Each core fuel burns out, the core contracts and heats to the next ignition: He → C,O; C → O,Ne; Ne → O,Mg; O → Si,S; Si → Fe,Ni, leaving shells like an onion (E3, E7). Above ~5 × 10^8 K neutrinos carry the energy away, so each stage is far shorter (E3) | 15 Suns: He 2 × 10^6 yr, C 2 × 10^3 yr, Ne 0.7 yr, O 2.6 yr, Si 18 d (E3); 25 Suns: He 10^6 yr, C 10^3 yr, Ne 3 yr, O 0.3 yr, Si 5 d (E7) | An iron core: fusing iron gives no energy, so the core has nothing left to burn (E8) |
| S6′ | Core collapse | The ~1.4-Sun Fe core loses its pressure support and falls inward at up to ~¼ the speed of light (E3, E7, E8) | "a few milliseconds" (E3); "less than a second" (E8) | The core bounces at nuclear density: a proto-neutron star ~20 km across (E3, E8) — B11's "core collapse" event |
| S7′ | Supernova | The bounce shock, revived by neutrinos (neutronization 3–10 s), blows the envelope off; ~0.07 Suns of radioactive 56Ni in the ejecta keeps it shining (E3) | Seconds for the engine (E3); the ejecta then expand for good — the edge rule (B20) takes over | The ejecta leave; what remains is the remnant |
| S8′ | Neutron star | Neutron degeneracy holds the remnant; ~1.4 Suns, ~10 km, born at ≥ 10^7 K surface (E19, E6) | Cools; no further stage in M0 | Stable while below the maximum neutron-star mass, 2–3 Suns (E3, E13) |

**From ~20–25 Suns — the black hole (B6)**
| # | Stage | What drives it | Real time scale | Ends when |
|---|---|---|---|---|
| S7″ | Failed or weak explosion | A heavier, more extended core resists explosion: either no supernova at all (**direct** collapse) or a weak one whose matter falls back (E5, E6) | Fallback: within one day (E5); direct: the collapse clock (S6′) | The remnant passes the maximum neutron-star mass (2–3 Suns, E3, E13; Heger takes 2.0 Suns of baryons, E5) |
| S8″ | Black hole | Gravity with no pressure that can stop it; matter that crosses r_s = 2GM/c² (≈ 3 km per Sun, E20) is gone — B6's "swallows nearby matter" | Grows as it swallows; no further stage | Never ends; in the sandbox, its mass grows by what it swallows |

**The ending thresholds (initial mass, single non-rotating stars, near solar metallicity)**
| Range (Suns) | Ending | Sources, and how far they agree |
|---|---|---|
| < ~0.08 | No ignition: a failed star that cools — **not one of the lead's three endings**, but the physics will show it | E10, E11 |
| ~0.08 to ~8–9 | White dwarf (He below ~0.5, C-O, O-Ne at the top) | 8 (E7, E11's "8–9" for full carbon burning, E3's M_up ≈ 8, E15's planetary nebulae "1–8"), 9 (E5), 8–10 (E14); published estimates 6–11 (E5) |
| ~8–9 to ~10–11 | Transition: an O-Ne core — electron-capture supernova and neutron star, or a white dwarf | E5 (9–10), E3 (M_ec ≈ 11) |
| ~10 to ~20–25 | Core-collapse supernova + neutron star | E5 (10–25), E19 (10–25 "or possibly more"), E3 (≤ 20 "probably"), E7 (< ~20) |
| ≳ 20–25 | Black hole — by fallback ≳ 25, directly ≳ 40 (E5); the boundary is 20–25 (E3); exploding masses "do not form a simply connected set" (E6) | Soft and non-monotonic in detailed models (E3, E6) |
| > 40–50 | No ordinary Type II supernova (E7); pair-instability regimes at ~140–260 (E5) — out of M0's scope (opinion) | E5, E7 |
| Remnant limits | White dwarf ≤ 1.44 (E12); neutron star ≤ 2–3 (E3, E13: 2.0–2.2 from GW170817, 2.2–2.9 in models) | E12, E13, E3 |

**I11's facts from memory, checked (B35)**
| I11 claim | Verdict | Sources |
|---|---|---|
| A supernova needs "about 8+ Suns" | **Holds** as the lower edge: 8 (E7, E3), 9 (E5), ~10 for a sure core collapse (E5, E11); the 8–10 window is a transition (O-Ne cores) | E3, E5, E7, E11 |
| A black hole needs "about 25+ Suns" | **Holds, soft**: 20–25 (E3), ≳ 25 by fallback and ≳ 40 directly (E5), ~20 (E7); not a clean line — some lighter stars implode and some heavier explode (E6) | E3, E5, E6, E7 |
| A core collapses "in about a second" | **Corrected to "under a second"**: the collapse takes milliseconds (E3, E7), "less than a second" (E8); the neutrino engine that drives the explosion runs 3–10 s (E3) | E3, E7, E8 |
| A Sun-like star lives "10 billion years" | **Holds for the main sequence** (E10, E11; 9 Gyr to core H exhaustion, E2). Cloud to white dwarf is longer: + ~2 Gyr as subgiant and red giant, + ~0.12 Gyr burning helium, + a few Myr on the AGB (E2) ≈ **~11 Gyr** (my arithmetic). B9's "cloud to white dwarf in about 10 seconds" squeezes this whole span, not 10 Gyr — for M0-R3 | E2, E10, E11 |

What B7 (touch any time) adds, from the same physics: adding gas raises the mass and moves the star
up the threshold table, and a heavier main-sequence star burns faster (τMS ∝ M^−2.5, E10); erasing
gas moves it down; heating a core speeds burning steeply (T^4, T^18, T^40, E4) and the thermostat
of S4 pushes back by expansion (E1, E3). No stage needs a script for this — each end condition
above is a physical state (opinion).

## Elements (B18: the star matter M0 paints and fusion makes)
Nine elements cover every burning stage of S4–S5′; a tenth, nickel, is optional. Each is paintable
(B18) and each but hydrogen is also made by fusion. Starting gas, for the painted cloud and the
presets: about 73 % hydrogen, 25 % helium, ~1 % heavier by mass, the Sun's mix (E22).

| # | Element | Made by (stage) | Burned by (stage) | Sources |
|---|---|---|---|---|
| 1 | Hydrogen (H) | — (primordial; 73 % of the Sun's mass) | H burning, pp chain or CNO cycle (S3–S4, red-giant shell S5) | E22, E4, E18 |
| 2 | Helium (He) | H burning (S4) | Triple-alpha (S6, S5′) | E3, E17 |
| 3 | Carbon (C) | He burning, triple-alpha (S6, S5′) | C burning (S5′, above ~8 Suns; > 5 × 10^8 K) | E3, E17 |
| 4 | Oxygen (O) | He burning (C + He); also C and Ne burning | O burning (S5′) | E3, E21 |
| 5 | Neon (Ne) | C burning (S5′) | Ne burning (S5′) | E3, E7 |
| 6 | Magnesium (Mg) | C and Ne burning (S5′) | O-stage burning (Table 12.1's "O, Mg" fuel) | E3, E7 |
| 7 | Silicon (Si) | O burning (S5′) | Si burning (S5′) | E3, E7 |
| 8 | Sulfur (S) | O burning (S5′) | Si burning (Table 12.1's "Si, S" fuel) | E3, E7 |
| 9 | Iron (Fe) | Si burning, via 56Ni that decays to 56Fe (S5′) | Nothing: fusing iron gives no energy — the end of the ladder and the trigger of S6′ | E3, E8, E21 |
| 10 (optional) | Nickel (Ni-56) | Si burning and the supernova (S5′, S7′) | Decays to Fe; powers the supernova's afterglow (~0.07 Suns in SN 1987A) | E3, E21 |

Left out (opinion): C burning's minor products Na and Al (E7), and the α-ladder's Ar, Ca, Ti, Cr
(E21) — trace amounts beside the nine; the CNO catalyst nitrogen (E4) — the CNO cycle changes the
rate, not the ash, so a temperature law can stand in for it. Remnant matter (white-dwarf, neutron
and black-hole matter) is a *state* of these elements, not new elements — how the registry holds it
is M0-R2b's and M0-TC's call (opinion).

## Squeeze (what a squeezed world must keep — orders and ratios only)
The lead's rule: "Nature's laws … but sizes and times are squeezed"; "Tests check each law's exact
answers, not the real Sun's numbers" (answer 1). So the sandbox keeps the **form** of each law and
the **order** of every quantity below; the real values are not targets. The real ratio is given
only to show how much room the order has. A squeeze must make every stage and every ending of
§Stages *come out of the physics* (answer 4).

**Time — the order of the clocks**
- **K1 · τdyn ≪ τKH ≪ τnuc.** Real: τKH/τdyn ≈ 3 × 10^11 and τnuc/τKH ≈ 10^2.5–10^3 (E1; the first
  ratio my arithmetic). The sandbox keeps the strict order with a real gap at each step — enough that
  the star is in pressure balance while it contracts and in heat balance while it burns, which is
  what makes S4 a long steady state and S5/S6′ visible as distinct events (E1). How big a gap is
  "enough" is not sourced → `## UNVERIFIED`; M0-R2b and M0-R3 measure it.
- **K2 · Each burning stage shorter than the one before, and the late ones collapse.** Real (15
  Suns): H/He ≈ 5.5, He/C ≈ 10^3, C/(Ne, O) ≈ 10^3, (Ne, O)/Si ≈ 15–50 (E3; ratios my arithmetic);
  25 Suns gives the same pattern (E7). Ne and O are of the **same** order (Ne shorter at 15 Suns,
  O shorter at 25) — the sandbox keeps H ≫ He ≫ C ≫ {Ne, O} ≫ Si, never a strict Ne > O. The cause
  to keep is the neutrino drain that takes over from carbon ignition upward (E3) — a cooling sink
  that grows with temperature faster than the photon losses, so late stages race (opinion from E3).
- **K3 · Heavier burns faster.** τMS ∝ M^−2.5 (E10), τPMS ∝ M^−2.5 (E2): doubling the mass shortens
  the life ~5.7× (my arithmetic). The sandbox keeps the decrease; the exponent is a tuning target, not
  a law to hit exactly (opinion).
- **K4 · Collapse is the fastest thing in a star's life.** Core collapse runs on τdyn of the core
  (milliseconds, E3) — far below every burning stage (Si: days, E3). The sandbox keeps S6′ an event
  measured in dynamical steps, and its B11 slow-down catches it (opinion).

**Temperature — the order of the ignitions**
- **K5 · Contraction heats.** Virial: a contracting gas ball gets hotter, T ∝ M^(2/3) ρ^(1/3) (E1,
  E2) — the engine of S2, S5 and every step of S5′. The sandbox must have it from gravity + gas
  pressure, never from a rule (answer 4).
- **K6 · Ignition thresholds in order, with their spacing.** H < He < C < Ne ≲ O < Si. Real ratios
  of central temperatures at ignition (15 Suns, E3): He/H ≈ 5, C/He ≈ 4.6, Ne/C ≈ 1.9, O/Ne ≈ 1.2,
  Si/O ≈ 1.7; Si/H ≈ 94 (my arithmetic). The sandbox keeps the order; it may keep the ratios too,
  which makes the temperature translation one factor (below).
- **K7 · Steep fusion laws.** ε ∝ T^4 (pp), T^18 (CNO), T^40 (3α) (E4). The steepness is what makes
  ignition a sharp event and steady burning a thermostat (S4); and a degenerate core cannot expand
  to cool, so a steep law there runs away — the helium flash (E16, E17). The sandbox keeps a steep
  exponent per reaction; the exact values are tuning (opinion).
- **K8 · Energy per gram: hydrogen first by an order of magnitude.** He and later fuels give "a
  factor 10 or more" less than H (E1); H/C ≈ 16, and C : Ne : O : Si ≈ 4 : 1 : 5 : 2 (from E1's φ
  and E3's yields; my arithmetic). Together with K2, this is why the main sequence dominates the life.

**Mass — the order of the endings**
- **K9 · The ladder of outcomes, in order.** no ignition < white dwarf < (transition) < neutron star
  < black hole, and the two remnant ceilings in order: white-dwarf limit M_Ch (E12) < neutron-star
  limit M_TOV (E3, E13). Real spacing (E5, E3, E10, E12): M_up/M_Ch ≈ 6, M_BH/M_up ≈ 2.2–3, M_Ch/M_min ≈
  18, M_TOV/M_Ch ≈ 1.4–2 (all my arithmetic). These two ceilings are what make the endings come out
  of physics: electron degeneracy stops a core below the first, neutron degeneracy below the
  second, nothing above it — so the sandbox needs **a pressure that holds a cold core up to a
  maximum mass, twice over** (opinion from E12, E13, E19). Whether the flat world's gravity law
  (B32) still gives a maximum mass is M0-R2b's question → `## UNVERIFIED`.
- The non-monotonic black-hole boundary (E6) is real but needs mass-loss and explosion physics M0
  does not model; the sandbox keeps the **simple** order (one threshold), which is the lead's "as
  its mass decides" (goal paragraph) — opinion, M0-TC to confirm.

**The real-equivalent translation (B31) — proposed; real values appear here only, as anchors**
- **Mass → Suns: one factor (recommended).** Sandbox mass × k = Suns. One factor keeps painting
  additive — twice the gas reads twice the Suns (B2) — and a readout that jumps would confuse the
  player (opinion). k is fixed by one anchor: the sandbox's own white-dwarf ceiling reads 1.44 Suns
  (E12). The sandbox's other thresholds then read wherever its physics puts them; a check compares
  them with the real ones (white dwarf / neutron star ~8–9, neutron star / black hole ~20–25 —
  E5, E3) within a tolerance M0-TC sets (R8). Fallback, if the physics cannot be tuned near those
  ratios: piecewise-linear between the threshold anchors — monotonic but no longer additive
  (opinion).
- **Temperature → kelvins: one factor if K6's ratios hold, else anchor to anchor.** Anchors (real,
  central, at ignition): H ~1–1.6 × 10^7 K (E25, E9), He ~10^8 K (E17), C ~5–8 × 10^8 K (E3, E7),
  Ne ~1.6 × 10^9 K, O ~1.8–1.9 × 10^9 K, Si ~2.5–3.3 × 10^9 K (E3, E7). Between anchors,
  interpolate in log T (opinion). Surface and gas anchors for the low end: the Sun's surface 5772 K
  (E9), a red giant's ~5000 K (E16), a hot white dwarf > 30,000 K (E14) — whether the sandbox has a
  "surface" temperature distinct from its cells is M0-R2b's.
- **Age → years: a stage clock (with M0-R3).** One factor cannot work: the sandbox squeezes K1–K2's
  ratios, so one factor would read either the main sequence too short or the collapse too long
  (opinion). Proposed: age = the real durations of the stages already passed + the fraction of the
  current stage done × its real duration, the fraction measured from the physics (e.g. the core's
  fuel burnt) and the durations scaled by mass (τ ∝ M^−2.5, E10, E2). Anchors: §Stages' real time
  scales. M0-R3 owns the speed side of this and may replace it.

## UNVERIFIED (refuted by default until a source confirms)
- **The gap K1 needs.** How large τKH/τdyn and τnuc/τKH must be in the sandbox for quasi-static
  contraction and steady burning to appear — no source; opinion says ≥ ~10 each. → M0-R2b, M0-R3
  measure.
- **A maximum mass in a flat world.** Whether degeneracy pressure plus B32's 2D gravity law gives a
  Chandrasekhar-like ceiling at all — no source read. → M0-R2b (B32), M0-TC fork.
- **The helium flash's duration** — "lasts seconds" (E17) and "on a timescale of days" (E11) disagree;
  Pols gives the follow-on flashes ≈ 1.5 Myr (E2). Nothing in M0 rests on it.
- **The Sun's red-giant size at the branch tip** (~179 R☉, read from a table summary of the
  red-giant-branch page, E16's page) — not confirmed in a primary source; not used.
- **The Sun's final white-dwarf mass** — not fetched; E14's typical 0.5–0.7 Suns stands in.
- **Painted gas at ~600 × 400 cells reaching every stage** — no source; whether a ~100-cell star has
  room for an onion of five shells (S5′) is M0-R2b's.
- **Envelope ejection (S7) from physics.** The real driver is pulsation- and dust-driven winds (E2);
  whether the sandbox's gas, gravity and heat alone can puff off a shell without a dedicated
  mass-loss model — no source. → M0-R2b, a model risk for B4.
- **The ~11 Gyr cloud-to-white-dwarf life** is my arithmetic from E2's stage durations, not a
  quoted figure.
- **Ignition temperatures for low-mass stars.** K6's ratios are from a 15-Sun model (E3) and a 25-Sun
  model (E7); a 1-Sun star ignites helium degenerately at ~10^8 K (E16, E17) but its exact central
  temperatures were not fetched.
