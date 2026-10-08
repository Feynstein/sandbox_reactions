# Simulation research — M0-R2b (2026-10-08)

**Summary.** Recommended: an **Eulerian grid** (the drawn cells are the simulation's cells) carrying
mass, momentum, energy and nine element fractions per cell, on the GPU through wgpu. **Gravity: the
3D law 1/r² acting inside a flat sheet** (a razor-thin world, B32's recommended answer), solved by
a **zero-padded FFT convolution** (Hockney–Eastwood), the same in both edge modes. Gas: a second-order
**finite-volume** scheme (MUSCL-Hancock, HLL-type fluxes) under a CFL step. Heat: conduction and
radiation as **flux-limited diffusion**, stepped by **RKL2 super-time-stepping**. Burning: **one
reaction registry** — reactants and a rate law in, products and energy out — that fusion fills in
M0 and chemistry reuses later (B19). Degeneracy: a **cold pressure** whose stiffness falls from
Σ² to Σ^(3/2) — which, in the sheet world and only there, yields **a maximum mass** (flag (a)
answered, by my arithmetic). Black holes: **sink particles**. The deciding fact for gravity: the
flat world's other law — 2D Poisson, a logarithmic potential — breaks two of star_physics.md's
invariants (K5 "contraction heats", K9 "a maximum mass"); the sheet keeps both. Cost, my estimate:
~0.9 ms per full step at 600 × 400 cells on an RTX 2070-class card; at the top speed (~25 steps per
frame, my arithmetic) that is ~22 ms against 16.6 ms — **for M0-R3** (B33).

Written to PLAYBOOK §4 Rule 7: every claim lives once in `## Evidence` with its URL and access date
on its line; the sections cite claims by `E<n>`, and star_physics.md's claims as `SP-E<n>`, its
invariants as `K<n>`. **Opinion** and **my arithmetic** are labelled; every claim no source confirmed
is in `## UNVERIFIED`. Two local checks ran on linux-pc (stdlib Python, no GPU): the gravity oracle on
a grid and the worked example's arithmetic — output in `logs/M0-R2b.log`.

## Evidence (each claim with its source)
**Gravity in a flat world**
- E1 · Poisson's equation ∇²φ = 4πGρ; the Laplacian's fundamental solution is "−(1/2π) log|x|" in two dimensions and ∝ |x|^(2−n) (1/r in three) — https://en.wikipedia.org/wiki/Poisson%27s_equation (accessed 2026-10-08)
- E2 · Virial theorem: for pair potentials V ∝ r^n, "2⟨T⟩ = n⟨V_TOT⟩"; for gravity (n = −1) ⟨T⟩ = −½⟨V_TOT⟩, "a star held together by its own gravity" — https://en.wikipedia.org/wiki/Virial_theorem (accessed 2026-10-08)
- E3 · Ostriker (1964): an infinitely long isothermal self-gravitating cylinder has the critical line mass M_lin,crit = 2c_s²/G; "a cylindrical filament is only thermally supported against collapse if its line mass remains below this critical value" (Chira et al. 2018, eq. 6) — https://arxiv.org/abs/1711.01417 (accessed 2026-10-08)
- E4 · Chavanis, *White dwarf stars in D dimensions*: degenerate pressure P = K₁ρ^(1+2/D) (non-relativistic) and P = K₂ρ^(1+1/D) (ultra-relativistic); polytropes are dynamically stable iff "γ > γ₄/₃ ≡ 2(D−1)/D" (eq. 120); "For D=2, the radius is independent on mass"; "quantum mechanics cannot balance gravitational collapse for D ≥ 4" — https://ar5iv.arxiv.org/html/astro-ph/0604012 (accessed 2026-10-08)
- E5 · Fermi gas in d dimensions: E_F ∝ (N/V)^(2/d); in d = 2 the density of states is constant; ultra-relativistic E_F ≈ p_F c — https://en.wikipedia.org/wiki/Fermi_gas (accessed 2026-10-08)
- E6 · Maclaurin (Kalnajs) disk: "a two-dimensional thin disk of fluid, in which the pressure operates only in the plane of the disk"; Σ(R) = Σc √(1 − R²/a²) for R < a (eq. 7); its potential in the plane is quadratic, Ω₀² = π²GΣc/(2a) (eq. 9), after Binney & Tremaine (Roshan, Abbassi & Khosroshahi 2016) — https://arxiv.org/abs/1610.01286 (accessed 2026-10-08)
- E7 · Self-gravity of "infinitesimally thin gaseous disks" on grids: "a two-dimensional kernel derived for infinitesimally thin disks … free of artificial boundary conditions", made fast by an FFT technique (Wang, Taam & Yen 2016) — https://arxiv.org/abs/1603.03142 (accessed 2026-10-08)
- E8 · Hockney–Eastwood free-space FFT Poisson solver: "a popular scheme … used by many cutting-edge codes, because of its speed and its simplicity"; "the potential and its gradient obtained with this method have low accuracy, and the numerical error converges slowly with the number of grid points" (Zou, Kim & Cerfon 2021) — https://arxiv.org/abs/2103.08531 (accessed 2026-10-08)
- E9 · VkFFT: MIT licence; backends Vulkan, CUDA, HIP, OpenCL, Level Zero, Metal (no WebGPU listed); "native zero padding to model open systems (up to 2x faster than simply padding input array with zeros)"; convolution support — https://github.com/DTolm/VkFFT (accessed 2026-10-08)
- E10 · Barnes–Hut: "O(n log n) compared to a direct-sum algorithm which would be O(n²)"; the opening threshold θ trades speed for accuracy, θ = 0 degenerates to direct sum — https://en.wikipedia.org/wiki/Barnes%E2%80%93Hut_simulation (accessed 2026-10-08)
- E11 · Softening: the potential −1/√(r² + ε²) replaces −1/r so close pairs do not diverge — https://en.wikipedia.org/wiki/Softening (accessed 2026-10-08)

**Gas flow**
- E12 · CFL condition: C = Δt Σᵢ uᵢ/Δxᵢ ≤ C_max, "typically C_max = 1" for explicit solvers — https://en.wikipedia.org/wiki/Courant%E2%80%93Friedrichs%E2%80%93Lewy_condition (accessed 2026-10-08)
- E13 · MUSCL: a finite-volume method with second-order spatial accuracy from slope-limited reconstructed states, which avoids spurious oscillations at shocks; Riemann-solver-free Rusanov / Kurganov–Tadmor fluxes fit it — https://en.wikipedia.org/wiki/MUSCL_scheme (accessed 2026-10-08)
- E14 · Sod shock tube: left ρ = 1, P = 1, u = 0; right ρ = 0.125, P = 0.1, u = 0 (γ = 1.4 in the figure); a code is tested "against the analytical solution" — rarefaction, contact, shock — https://en.wikipedia.org/wiki/Sod_shock_tube (accessed 2026-10-08)
- E15 · SPH "guarantees conservation of mass without extra computation"; boundaries are "one of the most difficult technical points"; cost per particle "significantly larger than the cost of grid-based simulations per number of cells" — https://en.wikipedia.org/wiki/Smoothed-particle_hydrodynamics (accessed 2026-10-08)
- E16 · Truelove et al. (1997): with the Jeans number J ≡ Δx/λ_J, "J_th,max = 0.25 was adequate to suppress artificial fragmentation" (Myers et al. 2013, appendix) — https://ar5iv.arxiv.org/html/1211.3467 (accessed 2026-10-08)

**Heat and light**
- E17 · FTCS diffusion: stable for Δt ≤ Δx²/(2α) in 1D and Δt ≤ 1/(2α(1/Δx² + 1/Δy²)) in 2D (r ≤ 1/4 on a square grid) — https://en.wikipedia.org/wiki/FTCS_scheme (accessed 2026-10-08)
- E18 · RKL2 super-time-stepping (Meyer et al. 2012): "Δt_h = Δt_p (s² + s − 2)/4" for s stages, with the explicit parabolic step Δt_p = C_p / max(χ_x/Δx² + χ_y/Δy² + χ_z/Δz²), C_p ≤ 1/2; second order, explicit, scales to > 10⁴ processors (Vaidya et al. 2017) — https://ar5iv.arxiv.org/html/1702.05487 (accessed 2026-10-08)
- E19 · Flux-limited diffusion: F = −D∇E, D = cλ/χ; Levermore–Pomraning λ(R) = (2+R)/(6+3R+R²), R = |∇E|/(χE); λ → 1/3 optically thick, |F| ≤ cE optically thin; solved implicitly there because "implicit differencing … ensures stability" beyond (Δx)²/D (Turner & Stone 2001) — https://ar5iv.arxiv.org/html/astro-ph/0102145 (accessed 2026-10-08)
- E20 · Pols ch. 5–6: radiative conductivity K_rad = (4/3) acT³/(κρ), F_rad = −K_rad ∇T (eqs. 5.14–5.15); photo-neutrino losses "roughly ε_ν ∝ T⁸" — https://www.astro.ru.nl/~onnop/education/stev_utrecht_notes/chapter5-6.pdf (accessed 2026-10-08)
- E21 · Heat kernel: K(t, x, y) = (4πt)^(−d/2) exp(−‖x−y‖²/4t) solves the heat equation — https://en.wikipedia.org/wiki/Heat_kernel (accessed 2026-10-08)
- E22 · Eddington luminosity L_E = 4πGMc/κ, radiation force per mass κF/c; above it "a very intense radiation-driven stellar wind" — https://en.wikipedia.org/wiki/Eddington_luminosity (accessed 2026-10-08)
- E23 · Reimers (1975) mass loss Ṁ = 4 × 10⁻¹³ η_R LR/M [M☉ yr⁻¹], η_R = 0.477 ± 0.070 on the red-giant branch (McDonald & Zijlstra 2015) — https://ar5iv.arxiv.org/html/1501.00874 (accessed 2026-10-08)

**Burning, late stages, remnants**
- E24 · Pols ch. 12–13: above ~5 × 10⁸ K neutrino losses take energy "much more rapidly than photon diffusion"; "the T-dependence of ε_nuc is larger than that of ε_ν", burning sits where ε_nuc = ε_ν and stays stable; τ_nuc = E_nuc/L_ν ≪ E_nuc/L, so "the evolution of the core speeds up enormously"; iron-core photodisintegration absorbs ~2 × 10⁵² erg; collapse neutrinos take ~90 % of the energy and "the shock wave fizzles out" — no prompt explosion — https://www.astro.ru.nl/~onnop/education/stev_utrecht_notes/chapter12-13.pdf (accessed 2026-10-08)
- E25 · Gamow factor: fusion probability ∝ exp(−E/kT − √(E_G/E)); the Gamow peak at E_max = [E_G (kT/2)²]^(1/3) — rates rise steeply with T — https://en.wikipedia.org/wiki/Gamow_factor (accessed 2026-10-08)
- E26 · Arrhenius: k = A exp(−E_a/(RT)) — https://en.wikipedia.org/wiki/Arrhenius_equation (accessed 2026-10-08)
- E27 · Timmes' networks: stiff ODEs, "an analytical Jacobian, a variable-order Bader-Deuflhard integration method, and MA28 sparse linear algebra"; a 13-isotope alpha chain — https://cococubed.com/code_pages/burn.shtml (accessed 2026-10-08)
- E28 · Sink particles (Federrath et al. 2010): "a sole density threshold … is insufficient"; checks for "bound state, gravitational potential minimum, Jeans instability and converging flows are absolutely necessary"; accretion rate "in excellent agreement" with Shu's collapse — https://arxiv.org/abs/1001.4456 (accessed 2026-10-08)
- E29 · Bondi accretion: Bondi radius 2GM/c_s², Ṁ ≃ πρG²M²/c_s³ — https://en.wikipedia.org/wiki/Bondi_accretion (accessed 2026-10-08)
- E30 · Lane–Emden: P = Kρ^(1+1/n) in a spherical Newtonian polytrope; analytic for n = 0, 1 (θ = sin ξ/ξ, ξ₁ = π), 5 — https://en.wikipedia.org/wiki/Lane%E2%80%93Emden_equation (accessed 2026-10-08)

**Hardware**
- E31 · RTX 2070: 448 GB/s memory bandwidth, 7.5 TFLOPS FP32 — https://getdeploying.com/gpus/nvidia-rtx-2070 (accessed 2026-10-08)

**Local checks (linux-pc, 2026-10-08, stdlib Python, log: logs/M0-R2b.log)**
- C1 · Maclaurin disk on a grid (cells as point masses, direct sum — the same sum a zero-padded FFT
  convolution computes), in-plane force vs E6's linear law: worst error at r ≤ 0.75a **1.0–1.6 %
  at a = 40 cells**, 2.0–3.3 % at a = 20 (2.6 % / 5.8 % at r = 0.9a); Plummer softening ε = 0.5 cell
  worsens it (1.7–2.7 % at a = 40). Error halves when a doubles: first-order convergence.
- C2 · Edge free fall r̈ = −k/r², k = 3πGM/4, integrated: 3.16228 vs (π/2)√(R₀³/2k) = 3.16228.
- C3 · Uniform disk self-energy, pair sum (60 × 60 cells) vs −(8/3π) GM²/R: −2.4 %.

## Models
Units (opinion): the sandbox computes in its own units — one cell Δx = 1, G = 1, mass per cell Σ
(surface density) — and every number below is in them. Real values never enter (answer 1).

### M1 · Representation: an Eulerian grid on the GPU (recommended)
| Candidate | For | Against |
|---|---|---|
| **Eulerian grid, one cell = one drawn cell (recommended)** | The 600 × 400 drawn cells are the state — painting, erasing, the views and the inspector read and write cells directly (B16, B17); the FFT gravity needs a grid anyway (M2); boundary rules are one ghost-cell row (M7) | Advection smears sharp edges (mitigated by M3's second order); empty space needs density and pressure floors (opinion) |
| SPH particles | Mass conserved exactly (E15) | Boundaries "one of the most difficult technical points" (E15) — B20's two edge modes; costlier per element than a grid cell (E15); a particle-to-cell pass every frame to draw cells |
| Powder-Toy style (one particle per cell + a coarse air grid) | The look the lead named | A cell holds one particle, not a density — a star needs density ratios of many orders (K5, K9) inside one cell size (opinion); its code is GPL — ideas only (R7) |

Per cell (opinion): Σ, Σu, Σv, E, nine element fractions (H, He, C, O, Ne, Mg, Si, S, Fe —
star_physics.md `## Elements`), a matter state (normal, electron-degenerate, neutron) — about
14 × f32 = 56 B, so 13.4 MB for 240,000 cells (my arithmetic). Remnant matter is a **state** of
the elements, not new elements, as star_physics.md proposed.

### M2 · Self-gravity, and gravity's law in a flat world (B32, I1)
Three ways to keep "nature's laws" in a flat world — each is exact 3D physics for some geometry:
| Option | Geometry it is exact for | What it does to the star |
|---|---|---|
| **A · 2D Poisson, log potential** (∇²φ = 2πGΣ in 2D; Green's function ∝ ln r, E1) | Every cell an infinitely long rod: a world of filaments | Virial (E2, n → 0 limit): ∫P dA = GM²/4 — **fixed by mass alone, independent of radius** (my arithmetic). So a star that radiates **contracts without heating**: K5 fails, ignition only by mass. Isothermal gas has one critical mass, 2c_s²/G (E3), above which nothing thermal holds it; polytropes are stable iff γ > 1 (E4, D = 2), and both degenerate laws (γ = 2, 3/2, E4) beat 1 — **no maximum mass**: K9 fails |
| **B · 3D law 1/r² inside a flat sheet (recommended)** (potential −GΣ/r, pressure only in the plane — the Maclaurin-disk physics, E6, E7) | A razor-thin disk in 3D space | Virial: 2∫Π dA = −W, W ∝ −GM²/R, so **T ∝ M/R: contraction heats** (K5 kept, my arithmetic). Equilibrium radius for Π ∝ Σ^γ: R^(3−2γ) ∝ M^(2−γ) — critical γ = **3/2**, which is exactly the 2D ultra-relativistic Fermi gas (E4, E5): **a maximum mass exists**, K9 kept (my arithmetic, flag (a)) |
| C · Axisymmetric 3D (r, z): the screen is a slice through an axis | True spheres | Every blob painted off the axis is a ring around it; two clouds (B15) are two rings — a sandbox where painting means rings (opinion: rejected) |

**Recommendation: B** (opinion, built on the arithmetic above; M0-TC puts the fork to the lead,
I1). Its consequences: the world is a sheet, so the 3D law is used with r measured in the plane;
the real Chandrasekhar formula is not reused, only its mechanism; the white dwarf's radius is
**mass-independent** while its gas is non-relativistic (γ = 2: R = 2K₁/(παG) for a uniform disk,
α = 8/3π, my arithmetic — the sheet's analogue of E4's D = 2 result) and shrinks to zero at the
ceiling.

Solvers for B (cost per step at 600 × 400, my arithmetic from E31's 448 GB/s at 60 % efficiency):
| Solver | Cost per step | Accuracy, limits | Verdict |
|---|---|---|---|
| Direct sum over cell pairs | 240,000² × ~20 flop ≈ 154 ms at 7.5 TFLOPS (E31) | Exact | Rejected — 10× the whole frame |
| Barnes–Hut tree | O(N log N) (E10); a tree rebuilt every step on the GPU — irregular memory | Error set by θ (E10) | Rejected — costlier to write than an FFT on a regular grid, less exact (opinion) |
| Multigrid Poisson | O(N) | Solves the **2D** Laplacian — option A's law only (E1) | Only if the lead picks A |
| **Zero-padded FFT convolution (Hockney–Eastwood; recommended)** | Grid doubled to 1200 × 800 = 960,000 points, 7.7 MB per complex f32 array; forward FFT, multiply by the kernel's precomputed FFT, inverse; ~70 MB of traffic ≈ **0.26 ms** | Any kernel — the sheet's 1/r (E7) — exact convolution of the cell masses; free-space, so no image masses at the edges (E8). Measured C1: ≤ 1.6 % interior at a 40-cell radius, first order (E8's "converges slowly") | **Recommended**; FFT written in WGSL — VkFFT does it on six APIs, not WebGPU (E9) |

Kernel (opinion): the in-plane 1/r potential, with the self-cell term replaced by the cell's own
integrated value, not Plummer softening — C1 measured softening ε = 0.5 doubling the error.
Vico–Greengard–Ferrando is the higher-order successor at the same cost (E8) — a later upgrade, not
M0's. Stability: gravity enters as a source term; the step limit is M3's CFL plus the free-fall
time resolved by ≥ ~20 steps (opinion); the Jeans length resolved by ≥ 4 cells (E16).

### M3 · Gas pressure and flow
- **Recommended: finite volume, MUSCL-Hancock, second order, HLL-family fluxes (E13), unsplit 2D,
  gravity as a momentum and energy source.** Conservative to round-off in mass and momentum by
  construction (fluxes cancel between neighbours). Equation of state: ideal gas Π = ΣkT/m (in-plane
  pressure, E6) plus M6's cold pressure.
- Stability: CFL, Δt ≤ C / Σᵢ (|uᵢ| + c)/Δxᵢ (E12), C ≈ 0.4 for an unsplit 2D scheme (opinion; E12's
  C_max = 1 is 1D). Resolution: the Jeans number ≤ 1/4 (E16), or clouds fragment artificially.
- Cost: ~4 × 56 B × 240,000 ≈ 50 MB per step ≈ **0.19 ms** (my arithmetic).
- Rejected: semi-Lagrangian "stable fluids" — unconditionally stable but neither conservative nor
  compressible (opinion; no source read → UNVERIFIED); SPH (M1).

### M4 · Heat transport: conduction and radiation (and light)
- **Recommended: one diffusion law for both — radiative conductivity K = (4/3)acT³/(κρ) (E20), plus
  a conductive part — made flux-limited (Levermore–Pomraning, E19) so the flux never exceeds
  "light speed × energy" in thin gas, stepped by RKL2 super-time-stepping (E18).** Explicit, no
  global solve — GPU-friendly; the time step grows as s² with the stage count (E18).
- Stability: explicit FTCS needs Δt ≤ Δx²/(4χ) on a square grid (E17); RKL2 stretches it to
  Δt_p (s² + s − 2)/4 (E18). Rejected: plain explicit (thin, hot gas has χ ∝ T³/ρ² — huge — so the
  step collapses, worked below); implicit ADI as Turner & Stone (E19) — stable, but a global solve
  per step on the GPU (opinion: kept as the fallback if RKL2's s grows past ~30).
- Cost: ~24 B per cell per stage ≈ 5.8 MB; 16 stages ≈ **0.34 ms** (my arithmetic, worked below).
- **Light emission (recommended):** luminosity is the heat flux leaving the star — the FLD flux
  into empty cells and through the world's edge, summed; no photon particles in M0 (opinion; the
  bootstrap's "does light travel as particles" — proposed no). The glow (B12) colours each cell by
  its T. The readout's surface temperature (star_physics.md's open question) is an **effective
  temperature** from L and the star's perimeter, L = 2πR · σT_eff⁴ in the sheet (my arithmetic,
  opinion — M0-TC fixes the readout).
- **Neutrino cooling (flag (c)):** a per-cell sink ε_ν ∝ T^m with m below the burning law's ν
  above a threshold temperature — Pols' mechanism exactly: burning sits where ε_nuc = ε_ν, is
  stable because ε_nuc is steeper, and the late stages race because the sink, not the surface, sets
  their pace (E24); photo-neutrinos give m ≈ 8 (E20). Energy into the sink leaves the world (counted,
  M8 O10).

### M5 · Nuclear burning — and the one reaction mechanism (B19)
- **Recommended: a reaction registry**, one record per reaction:
  `inputs [(element, mass share)] · outputs [(element, mass share)] · Q (energy per unit mass
  converted; negative absorbs) · rate law r = A · Σ^a · Π Xᵢ · f(T) · threshold`.
  f(T) is a **lookup table** in T (and Σ where needed), so one mechanism holds a steep power law
  (fusion: ν = 4, 18, 40 — SP-E4; the Gamow-peak origin, E25), an Arrhenius law (chemistry, E26),
  and a density-driven capture (neutronization, M6). Fusion fills it in M0: H → He; 3 He → C;
  C + He → O; C + C → Ne, Mg; Ne → O, Mg; O + O → Si, S; Si, S → Fe (SP-E3's ladder). Chemistry later
  adds records, not code ("ingredients plus conditions in, products plus energy out", answer 15's
  question). Mass is conserved per record (inputs = outputs); the energy Q goes to the cell's heat —
  the real mass defect (SP-E1's φ) is a readout fact, not modelled (opinion).
- Stability: burning is stiff (E27). Per cell, operator-split after the flow step, sub-cycled in
  registers until each sub-step changes no fraction by more than ~10 % and T by more than ~5 %, with
  a linearised implicit (backward-Euler) step per sub-cycle (opinion — a 9-element chain needs no
  Bader–Deuflhard, E27's choice for full networks).
- Cost: one read and one write of the cell, ≈ 25 MB with the EOS ≈ **0.09 ms**; sub-cycles stay in
  registers (my arithmetic).

### M6 · Degeneracy pressure, and what each ending needs (B4–B6)
- **Cold pressure (recommended):** Π_cold(Σ) blended from K₁Σ² at low density to K₂Σ^(3/2) at
  high — the 2D Fermi gas's two limits (E4, E5), e.g. 1/Π = 1/(K₁Σ²) + 1/(K₂Σ^(3/2)) (opinion). In
  the sheet (M2-B) the Σ^(3/2) end is exactly critical, so the cold core has a ceiling:
  **M_ch ≈ 1.77 K₂²/G²** for a uniform disk (my arithmetic; the true value comes from the CPU
  reference, O9). Two of them: electron matter (K₁, K₂), and neutron matter with stiffer constants;
  their ratio sets K9's M_TOV/M_Ch (real 1.4–2), a tuning target (opinion).
- **White dwarf (B4):** the core burns out, contracts, heats (K5), and the cold pressure stops it
  below M_ch; it then only cools through M4. **Shell ejection (flag (b)):** gas, gravity and heat
  alone are unlikely to puff a shell off — in nature winds do it (SP-E2). Recommended: the radiation
  force κF/c per cell from M4's flux (E22) — physics, not a script; an envelope near the Eddington
  ratio is driven off. Fallback: a Reimers-shaped surface wind, Ṁ ∝ η LR/M (E23) — an empirical law
  with a form (opinion: risk stays open → UNVERIFIED).
- **Supernova + neutron star (B5):** an iron core has nothing to burn (SP-E8); a density-driven
  registry record — neutronization, Q < 0 (photodisintegration absorbs, E24) — removes electron
  pressure, the core falls, and the neutron cold pressure stops it: a bounce and a shock. In nature
  the bounce shock fizzles and neutrinos revive it (E24) — so the neutronization record also
  deposits part of its energy into the surrounding gas (a neutrino-heating share, a tuning value,
  opinion). Risk: whether the sandbox's shock blows the envelope off at all → UNVERIFIED.
- **Black hole (B6):** a neutron core above its ceiling has no pressure left; when a cell passes a
  density threshold **and** the gas there is bound, converging and at a potential minimum (E28),
  it becomes a **sink particle** that holds the mass and momentum. It swallows bound gas inside an
  accretion radius max(2 cells, 2GM/c_sb²) — E29's Bondi form with the sandbox's own light speed
  c_sb (SP-E20's r_s = 2GM/c²; opinion). The sink moves under the grid's gravity; its own gravity
  is added to the grid's as a point mass (opinion). Readout: the labels recognise the sink (I2).

### M7 · The two edge modes (B20, I8)
| | Leaves for good | Bounces back |
|---|---|---|
| Gas | Outflow: ghost cells copy the edge cell, inward normal velocity clamped to 0 — nothing ever enters; mass, momentum and energy crossing the edge are added to an "escaped" counter (opinion) | Reflecting wall: ghost cells mirror the edge cell with the normal velocity negated — zero mass flux, exactly (opinion) |
| Gravity | Free space (M2's zero padding, E8): no image masses; escaped matter takes its gravity with it | The same: walls are not mass. A periodic FFT (no padding) would mean wrap-around — dropped by the lead (B28) |
| Heat and light | Flux leaves through the edge — counted as luminosity | **Proposed:** light still leaves (else the box fills with radiation and nothing cools) — a small fork for M0-TC (opinion) |

### M8 · Cost per frame, summed (my arithmetic — UNVERIFIED until M0-R3 measures)
Per full step at 600 × 400: hydro 0.19 + gravity 0.26 + heat (16 RKL2 stages) 0.34 + burning and
EOS 0.09 ≈ **0.88 ms** on an RTX 2070-class card (E31 at 60 % of peak). Steps per frame at the top
speed — see the worked example: ~25, so ~22 ms per frame against 16.6 ms. Levers (opinion, for
M0-R3): re-solve gravity every k hydro steps in quasi-static phases; run heat only where χ is
large; a smaller preset radius (steps ∝ R, below).

### Worked by hand — the stability limits of one star (the Adversarial line)
A Sun-like preset: a Maclaurin disk, Σc = 1, a = 40 cells, so M = 2πΣc a²/3 = 3351.
1. **Collapse.** Edge free fall t_ff = (π/2)√(2a³/(3πGM)) = **3.16** time units (O2; C2 checked it).
2. **Settled star.** Say it settles at R = 15 cells. Uniform-disk virial: c² = αGM/(2R), α = 8/3π
   = 0.849 → c = 9.7 mean, ~19.5 in the core (×2, opinion).
3. **Gas step.** Unsplit CFL at rest: Δt ≤ 0.4 / ((0 + 19.5) + (0 + 19.5)) = **0.0103**. The
   dynamical time R/c = 1.54 → **150 steps** per τ_dyn. In general, steps per τ_dyn ≈ 5–10 × R/Δx:
   the sound speed cancels.
4. **Heat step.** K1 asks τ_KH ≥ 10 τ_dyn (UNVERIFIED gap): with τ_KH ~ R²/χ, χ ≤ 225/15.4 = 14.6.
   Explicit: Δt_p = 0.5/(2 × 14.6) = **0.0171** > 0.0103 — the core is fine explicitly. At the hot,
   thin surface χ is ~100× higher (χ ∝ T³/ρ², E20): Δt_p = **1.7 × 10⁻⁴**, 60 explicit sub-steps per
   gas step. RKL2: the smallest s with (s² + s − 2)/4 ≥ 60 is **s = 16** (E18) — 16 stages instead
   of 60 sub-steps, 3.75× cheaper. This is why plain explicit is rejected (M4).
5. **The life.** With gaps of 10 at each clock step (K1), the main sequence lasts 10 × 10 × 150 =
   **15,000 steps**; a life in ~10 s at 60 frames a second (B9) is 600 frames → **25 steps per
   frame**, × 0.88 ms = **22 ms > 16.6 ms**. The top speed is over budget by ~1.3× on this estimate
   — for M0-R3 (B33), never a silent cap.

## Oracles (R8)
Each recommended model's exact answer, checkable in sandbox units. Real values serve only the
readouts ("Tests check each law's exact answers, not the real Sun's numbers", answer 1). Tolerances
are proposed (opinion) for M0-TC; every GPU law also gets a **CPU f64 reference of the same scheme**
— GPU floats differ between cards (engine_stack.md's consequence) — matched within 10⁻³ relative
after N steps (opinion).

| # | Model | Law and exact answer | Source | Proposed tolerance |
|---|---|---|---|---|
| O1 | M2 gravity | Maclaurin disk Σc√(1 − r²/a²): in-plane g(r) = −(π²GΣc/2a) r | E6; C1 | ≤ 2 % for r ≤ 0.75a at a ≥ 40 cells; error ratio a=20 / a=40 within 1.6–2.4 (first order, C1) |
| O2 | M2 + M3 collapse | A cold (c → 0) Maclaurin disk collapses homologously; every radius reaches the centre at t_ff = (π/2)√(2R₀³/(3πGM)) | E6 + my arithmetic; C2 | t_ff within 3 % (before the centre's shock) |
| O3 | M2 + M3 equilibrium | Virial in the sheet: 2∫Π dA + W = 0, W = ½∫Σφ dA | E2 (n = −1) | \|2∫Π dA / W + 1\| ≤ 3 % in any settled star |
| O4 | M2 symmetry | A symmetric kernel exerts no net self-force: Σ m g = 0 | Newton's third law; my arithmetic | ≤ 10⁻⁵ of Σ m\|g\| |
| O5 | M3 gas | Sod shock tube: rarefaction, contact, shock positions and plateaus of the exact Riemann solution | E14 | shock position within 1 cell at 400 cells; L1 density error ≤ 2 % |
| O6 | M3 resolution | No fragmentation while J ≤ 1/4; a uniform cloud at J = 0.25 stays one piece | E16 | one bound object, ± 0 fragments |
| O7 | M4 heat | A point of heat spreads as E21's Gaussian: σ²(t) = σ₀² + 2χt per axis; total heat conserved in bounce mode | E21 | σ² within 1 %; heat 10⁻⁵ relative |
| O8 | M4 RKL2 | The same Gaussian with s stages matches the explicit result | E18, E21 | 1 % in σ² |
| O9 | M6 cold core | Below M_ch a cold disk holds (R steady over 10 τ_dyn); above, it collapses; γ = 2 gives an R independent of M | M2-B arithmetic; E4 | M_ch within 5 % of the CPU f64 reference; R(M) flat within 3 % for γ = 2 |
| O10 | all — conservation | Mass: bounce mode constant; leave mode, grid + escaped constant. Energy: kinetic + thermal + W + radiated + neutrino-lost + escaped − nuclear released constant | conservation laws | mass 10⁻⁶ relative (f64 sums); energy 10⁻³ per τ_dyn (source-term gravity conserves it only approximately — UNVERIFIED) |
| O11 | M5 burning | One cell, T pinned: X(t) = X₀e^(−kt) for a linear rate; ΔE_heat = Q · Δm exactly; inputs' mass = outputs' | M5's registry | X within 10⁻⁴; energy 10⁻⁵; mass 10⁻⁶ |
| O12 | M5 order | A uniform heating ramp ignites H, He, C, then {Ne, O}, then Si — in K6's order | SP-E3, K6 | order exact |
| O13 | M4 + M5 thermostat | +10 % core T on a main-sequence star → back within 2 % in a few τ_KH; burning stable while ε_nuc is steeper than ε_ν | E24, SP-E1 | returns; no runaway outside a degenerate core |
| O14 | squeeze — K1, K2, K3, K9 | τ_KH/τ_dyn ≥ 10 and τ_nuc/τ_KH ≥ 10 (measured, flag (d)); stage durations H > He > C > {Ne, O} > Si; heavier stars burn out sooner (3 masses); endings in order of mass across the presets | star_physics.md `## Squeeze` | orders exact; gaps ≥ 10 (UNVERIFIED threshold) |
| O15 | M6 sink | Accretion conserves mass and momentum; nothing inside the accretion radius leaves | E28 | 10⁻⁶ relative |
| O16 | M7 edges | Bounce: zero mass flux through a wall, a pulse returns with v_n reversed. Leave: mass lost from the grid = escaped counter; inflow exactly 0 | M7 | 10⁻⁶; inflow = 0 exactly |

## UNVERIFIED (refuted by default until a source confirms)
- **The sheet's maximum mass** (flag (a)): the critical γ = 3/2 and M_ch ≈ 1.77 K₂²/G² are my
  arithmetic from E2, E4, E5 and a uniform-disk W (C3: −2.4 % vs the formula); no source treats a
  degenerate razor-thin sheet under 1/r² gravity. O9's CPU reference is the first check.
- **Option A's verdict** (log gravity breaks K5 and K9) is my arithmetic from E1–E4; Chavanis (E4)
  gives γ_crit = 1 for D = 2, but his D = 2 relativistic case was not found in the text fetched.
- **The K1 gap**: ≥ 10 per clock step is opinion (star_physics.md's UNVERIFIED, flag (d)) — and it
  sets the steps per life (worked example 5), so it drives B33.
- **Cost per step** (~0.88 ms) and **25 steps per frame**: peak-bandwidth arithmetic at an assumed
  60 % efficiency, no GPU run. M0-R3 measures on the Quadro RTX 4000.
- **Shell ejection from physics** (flag (b)): whether the radiation force alone drives a shell off
  in the sandbox — no source, no run; the Reimers fallback is empirical (E23).
- **The supernova's explosion**: real bounce shocks fizzle (E24); whether the neutrino-heating
  share revives the sandbox's — no source, no run.
- **The onion in ~100 cells** (star_physics.md's open item): five shells × ≥ 3 cells each (opinion,
  after E16's 4 cells per Jeans length) needs a core radius ≥ ~15 cells — a massive preset of
  radius ≥ ~60 cells; not measured.
- **Semi-Lagrangian fluids** are non-conservative and incompressible — from memory, no source read.
- **The sheet's Jeans length** λ_J ≈ c²/(GΣ) (thin-disk dispersion, from memory of Binney &
  Tremaine) — not fetched; O6 uses E16's 1/4 with it.
- **Energy conservation of source-term gravity** (O10's 10⁻³) — from memory, not sourced.
- **WGSL FFT performance**: VkFFT shows the method on six APIs (E9); a WGSL FFT at 1200 × 800 in
  ~0.26 ms is arithmetic, not a benchmark.
