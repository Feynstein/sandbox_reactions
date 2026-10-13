# Physics — units, registries, the equation of state, the constants

Written by M0-T17 (2026-10-10) for the first physics lot (M0-T10 – M0-T12); each later physics lot adds its pass here
(contract §7). The page points: a value lives in `assets/physics.json`, `assets/elements.json`, `assets/reactions.json` or
`assets/calibration.json` (§2.11) and in the contract section named beside it — **never copy one here**, it goes stale
at the first tuning. A law's form (an exponent, an ordering) is stated; a constant's value is looked up.
Pass order and module map: `architecture.md`. Scopes that grade these pages: `testing.md` (`registry`, `eos`; gravity's `fft_twin`, `fft`, `grav_force`, `grav_selfforce`; the flow's `riemann`, `sod`, `edges`, `collapse`; the twin's `twin`, `cpu_gpu`).

## Units (contract §2.1; `crates/sr-physics/src/units.rs`)
- Sandbox units: length is one cell (Δx = Δy = 1), G = 1, the mass unit is fixed by the Sun-like preset's central surface
  density and radius (`units::disk_mass(Σc, a)`; the calibrated figure is `preset_mass_sb.sun`, §2.10). One time unit (t.u.) is
  √(cell³ / (G · mass unit)). Temperature T is a specific energy (cell² / t.u.²) with Boltzmann's constant over the atomic
  mass unit absorbed, so the thermal pressure is Π_th = ΣT/μ.
- Real values never enter the physics. They appear only in the display translation (§1.10; readouts, `readouts.md`).
- Every channel is f32 on the GPU and f64 in the CPU twin (§2.2). A threshold, a bound or a tolerance is stated once, in the
  contract or in a constant here, and read from there.

## Elements (contract §1.5; `assets/elements.json`; `registry::Elements`)
- The file is `{"version", "species": [{key, name, symbol, a, z, paintable, color}]}`; **list order is the species id**, the element
  menu's order and the state's mass-fraction channel order (§2.2). The loader pins the file to §1.5's ten species — the state has
  exactly that many channels — so a new element is a new row *and* a channel change (B22), and a mismatch is refused.
- Neutron matter is a species (`n`, not paintable), not a state flag (D10): the neutronization record converts matter into it, mass
  is conserved per record, and it advects like any species.
- Derived per cell from the mass fractions X_i (`Elements::composition`): 1/μ = Σ X_i (Z_i + 1)/A_i (gas fully ionised at every
  temperature — a declared simplification), Y_e = Σ X_i Z_i/A_i, metals Z_met = 1 − X_H − X_He − X_n. The sum ΣX_i = 1 is kept by
  renormalising after every pass that moves it (`state` scope).
- A refusal is a `RegistryError {file, key, message}` — which file, which key, why — and never a silent default.

## Constants (contract §2.5.1, §2.7; `assets/physics.json`; `registry::Physics`)
- One flat object of numbers. `Physics::get(key)` reads one; `Physics::with(overrides)` makes a scene's copy (a scene may override a key,
  and every dependent value is re-resolved from the copy: build the reactions with `Reactions::shipped(&elements, &physics.with(…)?)`).
- **Key spelling.** The contract gives symbols, the file gives keys — lower-case, the ones scenes' `overrides` name: the six temperatures
  `t_h … t_si` (the `t_key` of the rate terms), `q_h`, one rate normalisation per record `a_<record>`, one steepness `nu_<record>` per late
  record, `sigma_n` (Σ_N), `burn_dx`, `burn_dt`, and §2.7's floors, tools, optics, neutrino, stand-in and step constants. `Physics::keys()` is the
  list; the file is its single source.
- **Bounds the loader enforces** (a violation is refused naming the key; each bound's source is the contract row, not this page):
  - the ignition ladder in strict order T_H < T_He < T_C < T_Ne ≤ T_O < T_Si (Ne and O may tie) and the ladder's overall ratio limit (§1.6.3, K6);
  - every steepness of a late record at least the K7 floor, and the neutrino-cooling exponent m strictly below each of them (§2.5.3);
  - the neutronization gate Σ_N above the relativistic-regime bound, checked while the electron pair is on;
  - f_dep inside [0, the primary cap]; the cap rises only with the S2 stand-in (below);
  - m_tov/m_ch inside the contract's interval, from the two K₂ constants, checked while both pairs are on;
  - every key finite, scales positive, switches that may be off non-negative, `block` a whole number ≥ 1.
- **Pairs.** K₁/K₂ for electrons and for neutron matter are each *on* (both > 0) or *off* (both 0 — a scene that needs cold pressure off, M0-T33b/T36);
  half a pair is refused.
- **Not checked at load:** `c_sb` against calibration's `max_gas_speed` (needs calibration.json, G-CAL) and the K1 timescale gap
  (τ_nuc against τ_KH; measured by G-SQUEEZE). Both are measured numbers, not file checks. Also `sigma_bh` against 4 × calibration's `ns_sigma_max_sb` (§2.6:
  a black hole's density must clear any stable neutron star's): the loader holds `sigma_bh` only to > 0 (the `Pos` class), so the bound is not enforced at load — M0-V7 names
  it, M0-V9 runs it.
- Retuning is by editing the file and re-running calibration (G-CAL refuses a stale `physics_hash`, §2.11). A tuned value is a PLAN-visible change
  only where the contract marks it fixed (the q ratios, K8); ν, T_k and A are tuning within the bounds above.

## Reactions (contract §1.6; `assets/reactions.json`; `registry::Reactions`)
- One mechanism for every reaction (B19): a record `{id, group, inputs, outputs, q_ratio, rate, enabled, standin}`; chemistry later adds
  records, never code (B23). The file is `{"version", "records": […]}`, the records in §1.6.2's table order.
- **The rate law** (§1.6.1): ω = A · Σ^a · Π X_i^{n_i} · f(T) · g(Σ); f(T) sums power-law terms (T/T_k)^ν above a threshold temperature and is zero below
  it; g gates a record on the electron column Y_eΣ (the neutronization only). Each input loses its share of ω, each output gains its share (mass fraction per
  time), and the cell's thermal energy gains q·ω per unit mass with q = q_ratio · Q_H (`Record::q(&Physics)`); a negative q absorbs.
- **What a record may name.** `a_coef` is a physics.json key (A is a tuned constant); a term's ν is a number or a physics.json key (the late records name
  theirs, so a scene's override reaches them); `t_key` is one of the six temperatures; `sigma_key` is a physics.json key. N_Fe has no terms (f ≡ 1) and a
  null threshold factor; a burning record has terms and a threshold factor in (0, 1].
- **Refused at load** (each names `records[<id>].<field>`): an unknown species, a species twice on one side, a share outside (0, 1], shares not summing to 1
  within `SHARE_TOLERANCE` on either side (this is what conserves mass per record), a `t_key` outside the six, a key not in physics.json, any ν below the K7 floor
  (checked on the resolved value), `orders` not one per input, a malformed threshold or gate, a duplicate id, an unknown field, a bad version, a group outside the
  seven accumulators (§1.9.1), a record that is both a stand-in and enabled.
- **Groups** are the seven energy accumulators of §1.9.1 (H, He, C, Ne, O, Si, N). A new group is a code change in the accumulator, not only a row.
- **q ratios are nature's** (K8): they come from the sources cited in §1.6.2's table and are not tuned. Only A, T_k and ν are.
- **Ignition order** is the ladder above (G-ORDER, §1.6.3). The sandbox squeezes the real ratios and keeps the order.
- **Stand-ins** (§1.6.4, FROZEN — the lead's «Yes, as a backup»): the primary mechanism runs first. S1 (dust-driven wind) ships as `kappa_dust = 0`; S2 (thermal
  bomb) ships as the f_dep bound of the primary mechanism. `registry::Standins {s1, s2}` holds the switches (`Standins::SHIPPED` = both off); `Physics::shipped()` uses
  that. Only a PLAN amendment that cites a measured failure of the primary mechanism (the contract names the failure and its threshold for each) flips a switch — and
  edits the loader's checks with it. A stand-in acts only through state (temperature, metals, neutron matter), never on a timer or a stage label (G-WATCH).

## Equation of state (contract §2.3; `crates/sr-physics/src/eos.rs`; `eos::Eos`)
- **Thermal part:** an ideal gas in the plane with adiabatic index γ = 2 (§2.3.1) — Π_th = (γ − 1) Σ ε_th = Σ ε_th, and T = μ ε_th. μ is **per cell** (from the
  composition), so `thermal` / `temperature` take it as an argument and the EOS stores none.
- **Cold pressure** (§2.3.2): P(x; K₁, K₂) = 1 / (1/(K₁x²) + 1/(K₂x^{3/2})), the 2D Fermi gas's non-relativistic (x²) and ultra-relativistic (x^{3/2}) limits blended.
  Electrons: Π_e = P(Y_eΣ; K₁ₑ, K₂ₑ). Neutron matter: Π_n = P(X_nΣ; K₁ₙ, K₂ₙ). Total Π = Π_th + Π_e + Π_n; wave speed c² = 2Π/Σ (§2.3.3).
- **Cold energy** u(x) = x ∫₀^x P(s)/s² ds is tabulated once per pair (`ColdTable`, node count and range in §2.3.2) by Simpson quadrature in √s, and read **log-log
  interpolated, on CPU and GPU alike**; ε_cold = (u_e + u_n)/Σ. The closed form (`cold_energy_exact`, u = 2x(K₂²/K₁)(y − ln(1+y)), y = (K₁/K₂)√x) is the *oracle*,
  never the source; the table's nodes agree with it far inside the interpolation error budget (the budget is spent by the interpolation between nodes, not by the nodes).
- **Outside the table:** below the first node u follows K₁x² (exact there); above the last node the last segment's log-slope continues. The GPU port (M0-T19) must
  extrapolate identically — it reads `ColdTable::ln_u()`, `node_u()` and `step()`.
- **Limits are checked where the blend has converged**, not across the table for one pair (the blend approaches 2K₂x^{3/2} only logarithmically; M0-T12's reading 1):
  the table's high end is graded with a pair whose transition lies low, its low end with the shipped pair, and the exact integral far outside the table.
- **Floor:** ε_th = E/Σ − ½|u|² − ε_cold, floored at T_floor/μ; `Eos::thermal` returns `Thermal {eps_th, floor_added}` and the caller books `floor_added`
  (energy per area) as the ledger's `floor_added` (§2.9). A pair at 0 has no table and contributes zero pressure and zero ε_cold; a half-on pair, a negative K or a
  non-positive T_floor is refused as a `RegistryError`.
- **Scheme, step and Δt** (§2.3.4–§2.3.5): § Gas flow below.

## Gravity (contract §1.4, P2; `crates/sr-physics/src/reference/{fft,grav}.rs` the f64 twin, `crates/sr-engine/src/step/{fft,grav}.rs` and `shaders/{fft,grav}.wgsl` the GPU)
- **The law (§1.4.1, Q1 — FROZEN; the lead, 2026-10-08).** The world is a razor-thin sheet in 3D space: every cell pulls every other with the **3D** law — potential −Gm/r, r measured in the
  plane — while gas pressure, heat and light act in the plane only. Two consequences the sandbox is built on: contraction heats (K5, T ∝ M/R in the sheet) and a cold core has a ceiling mass
  (K9: the in-plane ultra-relativistic Fermi gas, Π ∝ Σ^{3/2}, is exactly critical; graded by G-CORE). Free space in both edge modes — no image masses. G = 1 (§2.1).
- **The kernel (§1.4.2).** φ(x) = Σ_y K(x − y) Σ_src(y), K(i, j) = −G/√(i² + j²) for (i, j) ≠ (0, 0), and **K(0, 0) = −G · 4 ln(1 + √2)** — the exact potential at a unit square's centre from its
  own uniform mass (∫∫ dA/r over the square, `grav::SELF_CELL`; the contract gives the decimal). It is **not softened**: Plummer softening was SM-C1's candidate and was rejected — a softening of
  half a cell doubled the force error. Nothing in P2 or the twin softens K, and a changed exponent or shifted offset is what the plants `grav-kernel-exponent` and `grav-kernel-shift` plant.
  A sink's potential, −G M_s/√(r² + r_acc²) (§1.4.3), is the one place a softening length appears; sinks arrive with P3's lot and are not in P2 yet.
- **The source.** Σ_src = Σ in non-vacuum cells (Σ ≥ Σ_vac) and 0 in vacuum — the mask is applied where the plane is first read (`grav_rows` on the GPU, `grav::source` on the twin).
- **The FFT over the box (§1.3.3, §1.4.2).** φ is a zero-padded (Hockney–Eastwood) convolution over the active box, never the world: per axis the padded size is the smallest power of two ≥ 2 × the
  box side, at least 32 (twin: `fft::padded_size`; GPU: the box fit writes it). The twin is a radix-2 f64 FFT with each twiddle from its own sin/cos. The GPU holds one row or column of up to
  2048 points in one workgroup (the whole of WebGPU's default workgroup memory; the twiddles sit in a storage table built in f64 on the CPU, so no cos/sin runs in a shader) — a transform is
  2 dispatches, a convolution 3, P2 four (rows with the mask, columns × K̂ and back, the rows' inverse into φ, the gradient). Forward is unnormalised, the inverse divides by N, row-major
  `y · nx + x`, the kernel indexed `K(x_target − x_source)`. A box whose padded axis would pass 2048 is refused (`FftError`; `headless` exits 4 naming the axis) — a world axis over half of that
  cannot be solved today.
- **The kernel's transform lives for every size.** The box's padded size is written by `box_fit` on the GPU and never read back, so the CPU cannot pick a transform per frame. K is real and even
  on both axes, so is K̂, and each size keeps one quarter of it; the table holds every size the world's boxes can take, made once per `Step` in f64 from the twin's own `grav::kernel` (so a bug in
  the twin's K reaches the shader too). Its size and cost at the default world: `milestones/m0/tasks/M0-T31.md`.
- **The force: central differences (M0-T29's choice).** g = −∇φ by central differences, one-sided on the box's outermost ring (y down). G-GRAV1 was measured on the twin **before** the GPU
  was built: the force on a Maclaurin disk is far inside its bound over every cell with 0 < r ≤ 0.75a (worst error and its cell are printed; `testing.md`, `grav_force`), so the direct force
  kernels (two more inverse FFTs) stayed unbuilt — φ is one forward FFT, one multiply, one inverse, then the differences. The rim rule matters to G-GRAV2: the net self-force
  Σ m·g cancels exactly (to rounding) only while no mass sits on that outermost ring, which the box's 8-cell margin guarantees; the shader keeps the twin's rule. φ and g outside the box keep
  their last values.
- **Δt (§2.3.5).** The gravity term η_g/√max|g| reads P2's |g|; step 0's initial reduction runs after P2 on the initial state, so P2 runs twice on step 0 (one extra solve per run).
- **Not built.** The solve cadence k ∈ {1, 4} (§1.4.4) is a lever for M0-TG's budget; every step solves (k = 1). Sinks' potentials (§1.4.3) wait for P3.
- **Cost.** The measurements — the RTX 4080 Laptop's smoke and the Quadro RTX 4000 figures owed on `laserax-ai` — live in `milestones/m0/tasks/M0-T31.md` and `M0-T30.md`; this page holds none.

## Gas flow (contract §2.3.4–§2.3.5 and §1.4.2's source, P4; `crates/sr-physics/src/reference/{hydro,riemann}.rs` the f64 twin and its oracle, `crates/sr-engine/src/step/hydro.rs` and `shaders/hydro.wgsl` the GPU)
- **The scheme (§2.3.4).** Unsplit finite volume, MUSCL-Hancock on primitives (Σ, u, v, Π, X_i): per cell a monotonised-central slope of each primitive; the face states go to conserved variables through the **full EOS**
  (E = ε_cold + ε_th + kinetic, with ε_th from Π − Π_e − Π_n, floored at T_floor/μ); the Hancock half-step advances every face with **both** axes' flux differences, so the update is unsplit; a predicted state that is not a state
  (Σ ≤ 0, no thermal energy left) falls back to the unevolved face state. The species slopes share **one** limiter factor (the smallest over the species), so Σ_i slope_i = 0 and ΣX_i = 1 holds to rounding.
  Fluxes are HLL with Davis speeds from c² = 2Π/Σ (§2.3.3); a species flux is the HLL mass flux times the **upwind** side's X_i, so mass and species cannot disagree. A cell below Σ_vac is read as the floor state (Σ_floor, at rest,
  T_floor); a cell and its four neighbours all below Σ_vac are left untouched (P8 owns vacuum).
- **Gravity's source in the flow (§1.4.2).** The Hancock predictor kicks each **face state** by half a step of the cell's g — momentum += ½Δt·Σ_f·g, energy += ½Δt·Σ_f(u_f + ¼Δt·g)·g, i.e. the face's velocity moves by ½Δt·g and its thermal
  energy is untouched (a centre kick could drive a face's ε_th to the floor in a cold disk: M0-T36's reading, not the contract's words) — and returns the cell centre's half-step (Σ, Σu, Σv); the update adds Δt·Σ_½·g to the momenta and
  Δt·(Σu)_½·g to the energy. No gravity is a branch, not an added zero. The radiation force (§2.4.4) and heat are not in P4 yet (M0-T43, M0-T41).
- **Δt (§2.3.5).** CFL term over (|u| + c)/Δx + (|v| + c)/Δy and the gravity term η_g/√max|g|. On the GPU Δt_{n+1} is reduced at the end of step n (P9) from step n's own P2 field — the pre-hydro state's — so the twin's driver
  takes g the same way (§ The CPU twin); a loop that recomputes g from the state it reduces differs by 4e-5 in Σ after 12 steps of `cold_disk.json` and passes the collapse oracle either way.
- **The GPU's layout (M0-T34, M0-T36).** Two entry points over the box's 8 × 8 tiles: `hydro_update` — every cell **recomputes its four neighbours' predicted faces** (≈ 5× the predictor per cell and 25 `read_cell` EOS evaluations; a stored face set is
  ≈ 1 GB at the largest gravity-off world, past the 128 MiB binding) and writes a **next state**, never the state it reads, so no invocation sees a half-updated neighbour — then `hydro_commit` (next → state); between them `hydro_edges` (§ Edges).
  `hydro_update` binds WebGPU's whole 8 storage buffers: the state's 3, the next state's 2 (`next_lo` = Σ, Σu, Σv, E, x_H, x_He, x_C; `next_hi` the other seven — 7 planes at 2048² stay under the 128 MiB binding, a const assert says so), the Δt record, g (binding 14;
  a 2-word placeholder is bound without P2) and eos.wgsl's table. Flux arrays are indexed by constants only (`if dir == 0u`): an index from `dir` keeps a WGSL array out of registers. P4 is **opt-in** in the step API: `Step::with_passes(Passes { mode, gravity, hydro })`;
  `new`, `with_box` and `with_gravity` keep it off, and `headless` turns it on by the scene's `hydro` switch (and hands P2's field to P4 when both are on).
- **Cost, flagged.** P4 dominates the step (≈ 1.0 ms of ≈ 1.2 ms on the Quadro RTX 4000 for sun_disk, `tasks/M0-T34.md`). A tiled `hydro_update` (12 × 12 primitives in workgroup memory) is the known fix, unbuilt: whether it is owed is G-TOP's / M0-R3's budget's call. This page holds no figure.
- **The oracles and the overrides they need.** `reference::riemann` is the exact γ = 2 solution (star p*, u* by a safeguarded Newton inside a bisection bracket; `Solution::solve(State{rho,u,p}, …).at(x, x0, t)`; refusals `BadState`, `Vacuum`), the test-side truth for G-SOD.
  Two scenes switch **cold pressure off**, through `overrides` naming exactly `k1e`, `k2e`, `k1n`, `k2n` = 0 and nothing else, and a later reader must not take that for a bug: **Sod** (`scenes/sod.json`) because the exact solution is the *ideal gas's* — the shipped electron pressure at Σ = 1 sits above Sod's P = 1 (my arithmetic from the shipped K's, not a measured run) and would
  make the gas not γ = 2 (the scope's own cold-*on* case, Σ 2 | 0.5 at T = 2 with electrons 90 % of Π, grades only conservation, never the shock); **the collapse** (`scenes/cold_disk.json`, G-COLL) because §5.1's oracle is a pressureless free fall ("pressure is negligible at T_floor") and a degenerate pressure at Σc = 1 would hold the disk up (the reading; no task ran the scene with the shipped K's).
  `ref_disk.json` and `ref_sod.json` (G-REF) copy the same four overrides, and `edge_pulse.json` runs cold off too. The collapse scope **asserts** the scene's overrides are exactly those four keys and that every other physics key is the shipped value — G-COLL green because more was switched off would be the failure.
  Sod's world is **400 × 64**, not 400 × 4 (§2.8's floor of 64 a side): 64 identical rows, the twin's strip is the first four.
- **Not built.** Radiation force and heat's coupling (lot 6), sinks and burning's source terms, the tiled update.

## Edges (contract §2.8, §2.9's escaped; P4's ghosts and `hydro_edges`, `Step::set_edge_mode`)
- **Two modes, a setting.** A new world starts in *leave* (D1; a scene naming no edge leaves), the player or `headless --edge` may switch at any time; the mode reaches the GPU as a uniform (`Edges`, binding 13) written with `queue.write_buffer`, so a change lands at the next
  step boundary, between submissions (§3.1) — a run that is *bounce* for 64 steps and *leave* for the rest equals a *leave* run from step 64 bit for bit (graded). Two ghost cells per side. Gravity is free space in both modes (§1.4.1); heat and light leave through the edge in both (P5, lot 6).
- **Bounce.** Ghosts mirror the edge cell with the normal velocity negated; the wall face's mass, energy and species fluxes are exactly 0, its normal-momentum flux is the predicted face state's Π. Bounce books nothing.
- **Leave.** Ghosts copy the edge cell with the normal velocity component clamped outward. The boundary face's flux is HLL between the edge cell's predicted face and the ghost's predicted face (the predictor is a method run on a ghost too: `Hydro::predicted`; the GPU's `boundary` function is shared by `hydro_update` and `hydro_edges`).
  **A face whose HLL mass flux points into the world is a wall instead** — mass, energy and species 0, momentum Π — so inflow is exactly 0. That is M0-T35's reading of "clamped to outflow": clamping only the mass term would let energy and momentum cross without mass. The momentum through such a face is booked as escaped (in leave, all of a boundary face's flux is booked).
  HLL's diffusion term still lets a little out where gas lies stagnant against the edge (s_L·s_R·(Σ_R − Σ_L) with the edge face denser than the ghost's), **never in**; so "inflow is 0" is graded per face per step, not as "the upwind sides book 0".
- **Booking.** Escaped mass, momentum, energy and each species are booked **per boundary face**: 2 (w + h) slots, the booking's `Domain::EdgeFaces` order (the twin's `State::escaped` keeps the same slot order). On the GPU `hydro_edges` runs one invocation per slot, 64 a workgroup, at a fixed size (the latch can
  replay it), between update and commit on the state as the step found it, skips faces outside the box or over empty cells, and adds ±Δt·flux into `booking.escaped`; it binds 7 storage buffers (g included; 6 before M0-T36). The twin books only for cells the update moved, from the flux the update applied.
- **Why the bounce oracle is two oracles.** G-EDGE's pulse (a right-going acoustic pulse in `scenes/edge_pulse.json`, returning within 2 % of −v₀) cannot see its own plant `wall-copy`: the wall flux alone turns the pulse, and copy ghosts in place of mirrors leave it unchanged to 7 digits. The scope adds the GPU's bounce against the twin's mirrors on a wind
  (relative L1 1e-4); the plant is in the shader only, so the twin's mirrors grade it. The leave cases run an inline wind rather than the pulse (the pulse's escape, ~1e-4 of its grid, is below what f32 resolves at 1e-6).
- **Not built.** Light through the edge (`radiated`, lot 6).

## The CPU twin (contract §1.7, §3.3, §5.1's G-REF, R13; `crates/sr-physics/src/reference/{mod,hydro,floors,riemann,grav,fft}.rs`; `headless --cpu-reference`)
- **What it is.** `sr_physics::reference` holds the f64 twin of every pass built so far, a test oracle never shipped: the FFT and gravity (§ Gravity), the flow (§ Gas flow), P8 (`reference::floors`, §2.7 [M0-T19], moved there out of the GPU's floors test: `Floors::{cell, apply}`, `Booked {vacuum_reset, floor_added}`; Σ_vac and Σ_floor are
  rounded to f32 as the GPU has them), and the exact Riemann solution. `State` is §2.2's 14 planes in f64, channel-major.
- **The driver, `Twin` (M0-T37).** One step is §1.3.2's order on the whole world: P1 takes the Δt that P9 made at the end of the last step, **P2 on the state as it stands**, P4 with that g, P8, P9 reducing Δt_{n+1} from the **P2 field of the step just run** — the GPU's order, so the two take the same Δt every step.
  Step 0 runs on a Δt reduced from the initial state with P2 run on it. `Twin::pass_list` / `passes_run` give the labels `Pass::label` gives the GPU's (the scope prints them beside the GPU's for each of the four gravity/hydro switch pairs); `first_non_finite` is P9's guard by cell then channel.
  The scene's switches are honoured; passes not built yet (heat, burning, neutrinos, sinks) are simply absent from both lists. **A world over `MAX_WORLD` = 128 on an axis is `TwinError::TooLarge`**: the twin has no active box and its FFT and per-step cost are the whole world's.
- **`headless --cpu-reference`.** Runs the twin from the same scene planes in place of the GPU and writes the same `summary.json` (adapter `cpu-reference`; the `SR-ADAPTER` line says `backend=cpu driver=sr_physics::reference`) and dumps, whose box record is the **whole world's** (`BoxMode::WholeWorld`'s form). A world over 128 on an axis is **exit 4**, no summary.
  `--adapter` and `--timing` (they mean the GPU) and `--load-dump` (a dump holds no g, and Δt_{n+1} needs step n's, so the twin cannot resume bit for bit) are refused with it, exit 4 — declared defaults, cheap to reverse. The flag is listed in `running.md`.
- **G-REF's role (scope `cpu_gpu`, M0-T38).** The twin's job is to be the GPU's check: the same scene, exactly 200 steps on each (not the same sim time — a shorter Δt reads as a state difference, which is how the plant `cfl-gpu` is seen), read side by side, relative L1 ≤ 1e-3 in Σ and in E and ≤ 1e-4 absolute in each X,
  on a cold collapsing disk (leave, gravity + hydro) and Sod's strip (bounce, hydro) with **mixed** compositions (an all-hydrogen scene grades nothing in X). Agreement is within f32 against f64 — never bit for bit, and across adapters likewise (§6.7). Three declared choices: **X is read where both states hold Σ ≥ 100 × Σ_vac** (at the vacuum threshold whether a neighbour crossed Σ_vac this step
  decides whether P8's pure-hydrogen floor matter mixes in, so rim cells read |ΔX| ~1e-3; the contract's G-REF does not say which cells); the disk's Δt is bound by gravity's term, not the CFL, so Sod carries the CFL separation; the burning field is the third scene and joins with burning (M0-T48).
- **Where the twin and the box differ.** The twin runs the whole world; the GPU leaves vacuum outside the fitted box as uploaded (Σ = 0, x_H = 1) where the twin's P8 resets it to Σ_floor — negligible against G-REF's bands, but a per-cell comparison of vacuum cells (or of X in them) sees it. The overrides the oracles need (§ Gas flow) apply to the twin and the GPU alike.
- **The twin's own scopes.** `riemann` (the oracle), `sod`'s and `edges`' and `collapse`'s twin cases (graded before the GPU's, with tolerances fixed before the first GPU run), `twin` (the driver and `--cpu-reference`), `cpu_gpu` (G-REF). Their costs and plants: `testing.md`.

## Where a number lives (so this page never holds one)
| A value | Lives in | Checked by |
|---|---|---|
| Burning, floors, tools, optics, neutrino, stand-in and step constants | `assets/physics.json`, symbols in §2.5.1 and §2.7 | `registry` (bounds), `eos` (EOS use) |
| Species A, Z, colours, paintability | `assets/elements.json`, §1.5 | `registry` (verbatim against §1.5) |
| Record shares, q ratios, rate terms, gate | `assets/reactions.json`, §1.6.2 | `registry` (shapes and mass conservation) |
| Measured scales: max gas speed, core and mass thresholds, preset masses | `assets/calibration.json`, §2.11 | `calibration` (G-CAL, later) |
| Display anchors (real Sun, years, kelvin) | §1.10 only | `readouts` (later) |
| G, the kernel, K(0, 0), the FFT sizes | §2.1, §1.4.2, §1.3.3 (`grav::SELF_CELL` in code) | `grav_force`, `grav_selfforce`, `fft_twin`, `fft` |
| The scheme's constants (C, η_g, the limiter, HLL), the edge modes, the twin's size cap | §2.3.4–§2.3.5, §2.8 (`MAX_WORLD` in code) | `sod`, `collapse`, `edges`, `twin`, `cpu_gpu` |
