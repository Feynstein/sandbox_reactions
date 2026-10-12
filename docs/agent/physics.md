# Physics — units, registries, the equation of state, the constants

Written by M0-T17 (2026-10-10) for the first physics lot (M0-T10 – M0-T12); each later physics lot adds its pass here
(contract §7). The page points: a value lives in `assets/physics.json`, `assets/elements.json`, `assets/reactions.json` or
`assets/calibration.json` (§2.11) and in the contract section named beside it — **never copy one here**, it goes stale
at the first tuning. A law's form (an exponent, an ordering) is stated; a constant's value is looked up.
Pass order and module map: `architecture.md`. Scopes that grade these pages: `testing.md` (`registry`, `eos`; gravity's `fft_twin`, `fft`, `grav_force`, `grav_selfforce`).

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
- **Scheme, step and Δt** (§2.3.4–§2.3.5) belong to the flow pass, not to this page until that pass lands (M0-T19+).

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

## Where a number lives (so this page never holds one)
| A value | Lives in | Checked by |
|---|---|---|
| Burning, floors, tools, optics, neutrino, stand-in and step constants | `assets/physics.json`, symbols in §2.5.1 and §2.7 | `registry` (bounds), `eos` (EOS use) |
| Species A, Z, colours, paintability | `assets/elements.json`, §1.5 | `registry` (verbatim against §1.5) |
| Record shares, q ratios, rate terms, gate | `assets/reactions.json`, §1.6.2 | `registry` (shapes and mass conservation) |
| Measured scales: max gas speed, core and mass thresholds, preset masses | `assets/calibration.json`, §2.11 | `calibration` (G-CAL, later) |
| Display anchors (real Sun, years, kelvin) | §1.10 only | `readouts` (later) |
| G, the kernel, K(0, 0), the FFT sizes | §2.1, §1.4.2, §1.3.3 (`grav::SELF_CELL` in code) | `grav_force`, `grav_selfforce`, `fft_twin`, `fft` |
