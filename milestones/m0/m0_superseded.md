# M0 superseded / retired — full text (the plan's `## Superseded / retired (§2.6)` points here)

Binding as if it sat in the plan (PLAYBOOK §2.6 P3, §11; R19). Moved byte for byte from the plan's Superseded section by M0-TC8,
2026-10-23 (the lead: «Superseded to an annex (Recommended)»); a later mirror (§2.6 P3) is appended at the end, the same shape.

## Superseded / retired (§2.6)
- R5's text, "linux-pc is the box for every M0 gate; win-laptop joins when the lead first runs a task there" (index:
  "Boxes: linux-pc for every M0 gate, win-laptop later") — superseded by M0-TJ3's ruling (the lead, 2026-10-08, Q1
  «Build and close lots anywhere»; reports/win_laptop.md § Ruling).
- Contract §6.6's text, "Later (R5): DX12 or Vulkan through wgpu; a scope that names linux-pc reads `owed on win-laptop`
  there until a task runs it." — superseded by §6.6 [M0-TJ3] (the same ruling, Q1–Q3).
- The goal paragraph's "the Windows laptop" among the items routed past M0 (reports/bootstrap.md §3: "The Windows laptop
  as a build box — later — when the lead first runs a task there") — superseded by the same ruling (Q1); Steam and a
  Windows release stay past M0.
- M0-D7's owed 20-of-20 whole-scope loop on win-laptop — N/A, retired by the lead (2026-10-08, M0-TJ3 Q4 «Skip it»); a
  later 0xC0000005 is a D with its faulthandler stack (Hazards).
- Contract §1.2's text "Features: eframe `wgpu`" (FROZEN, Q5) — superseded by contract §0.5 [M0-T5]: eframe
  `wgpu_no_default_features`, because `wgpu` re-enables wgpu's webgl and gles through egui-wgpu's defaults (the lead,
  2026-10-09, M0-T5: «wgpu_no_default_features (Recommended)»).
- Contract §5.4's G-LIC list ("a missing licence or any other … is NO-GO") — superseded by contract §0.5 [M0-T14]: one named
  exception, `epaint_default_fonts` with exactly `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` (bundled fonts; the lead,
  2026-10-10, M0-T14: «Named exception (Recommended)»).
- Contract §5's scope names written with a hyphen (`grav-force`, `grav-selfforce`, `heat-rkl2`, `heat-limiter`, `burn-cell`,
  `burn-order`, `top-speed`, `watch-only`) and every block's scope name written so — superseded by contract §0.5 [M0-V2] and
  R17: the same names with `_` (the lead, 2026-10-10, M0-V2: «Yes, rename once (Recommended)»).
- Contract §2.12.3's dump header "`{"version": 1, "width", "height", "step", "sim_time", "channels": [§2.2's names],
  "sinks": [...]}`" — superseded by contract §0.5 [M0-T26]: the same plus `"resume": {"dt", "dt_next", "latch_since",
  "box", "box_args"}`, the step's records a bit-identical resume needs (the lead, 2026-10-11, M0-T26: «Add one header key
  (Recommended)»).
- R16, the commit line — every block's last message hands the lead `git add -A`, a commit `M0-<ID> <status>` and a push
  (the lead, 2026-10-10, after M0-T15) — N/A, superseded by PLAYBOOK v12.3 (2026-10-11, M0-TE-V12.3), ratified for every
  project at the bump: the lead, 2026-10-10, *"In V12.3 if we dont need git well have to remove that rule"*; a block's
  handoff ends without the line, a phase's commit gate (§10) stays the lead's, R16's text kept where it stands (Rules,
  m0_rules.md).
- Contract §2.8's "any width and height that are multiples of 8 within [64, 2048]" read with gravity on — superseded by
  contract §0.5 [M0-V4]: with gravity on, each axis at most 1024 cells (refused when the step is built, `headless` exit 4);
  [64, 2048] stays with gravity off (the lead, 2026-10-12, M0-V4: «Cap at 1024 (Recommended)»).
- R9's « (3) red-arm it while you iterate » and « (5) run `verify.py --changed --base <named commit> --task <ID>` » (index:
  « own scope until green → `verify.py --redarm <scope>` → plan edits → `verify.py --changed` once, last »), R10's
  « `python3 tools/pb/verify.py --changed --base <the last commit gate's SHA> --task <its ID>` … the base a named commit,
  never `HEAD` », R15's « The claim run's base stays the last commit gate's SHA (R10). » and the budget line's « over it,
  a TR fires » read raw — superseded by PLAYBOOK v12.3 §4 Rule 4 (the task's own start, the two tiers and the sweep, the
  TR trigger at an idle box — ratified for every project, the lead, 2026-10-10: "Per check, sweep at phase end
  (Recommended)"), applied by M0-TE-V12.3, 2026-10-11, where each stands [M0-TE-V12.3].
- Contract §3.6's launcher text « wait for Enter, `launch.py stop` » (start.sh) and « `py -3.12 tools\pb\launch.py stop` »
  (start.bat) — superseded by contract §0.5 [M0-D28]: `launch.py stop <the service it started> --task launcher`, because
  launch.py's records are the box's and a bare stop took a parallel scope's server (the lead, 2026-10-13, M0-D28:
  «Launchers stop their own (Recommended)»).
- Contract §5.3 G-BOX's "2,000 steps … mass, momentum, energy and each species' mass equal within 10⁻⁶ relative" —
  superseded by contract §0.5 [M0-T39]: every sum at 10⁻⁶ at step 200 (the box inside the world), the masses at 10⁻⁶ at
  2,000, the blob on its masses; gravity's FFT round-off grows to 3.1 × 10⁻⁶ of momentum by 2,000 (the lead, 2026-10-13,
  M0-T39: «1e-6 at step 200 (Recommended)»).
- Contract §5.1 G-REF's "≤ 10⁻⁴ absolute in each X" — superseded by contract §0.5 [M0-V5]: each X where both runs hold
  Σ ≥ 100 × Σ_vac, the band unchanged; over every cell the vacuum's floor gas reads 2.2 × 10⁻³, 4.8 × 10⁻⁷ once the vacuum
  carries the gas's own composition (the lead, 2026-10-13, M0-V5: «Amend G-REF (Recommended)»).
- Contract §5.1 G-SOD's "on a 400 × 4 strip (bounce edges)" — superseded by contract §0.5 [M0-V5]: a 400 × 64 world of
  identical rows, the twin graded on its first four, the GPU on all 64; §2.8 refuses a side under 64 cells (the lead,
  2026-10-13, M0-V5: «Amend G-SOD (Recommended)»).
- Contract §2.4.3's "If s > 32 in any step the run prints `SR-WARN heat stages <s>`" read as running s stages — superseded by
  contract §0.5 [M0-T42]: the GPU's P5 runs at most 32, capping the faces of each cell that alone would need more (sun_disk's
  rim needs up to 3,803; the lead, 2026-10-14, M0-T42: «Throttle stiff faces (Recommended)»).
- Contract §2.12.3's "then each channel as a little-endian f32 plane in §2.2's order, row-major" (with [M0-T26]'s `resume`) —
  superseded by contract §0.5 [M0-D33]: when the run runs P5, F's two f32 planes follow the 14 channels, flagged by
  `resume.flux`, and `--load-dump` restores them (the lead, 2026-10-14, M0-T43: «Dump carries F (Recommended)»).
- Contract §1.6.1's "f(T) = Σ_k w_k (T / T_k)^{ν_k} for T ≥ T_thr, else 0" — superseded by contract §0.5 [M0-D34]: each term
  Gamow-shaped, w_k (T/T_k)^{−2/3} exp(τ_k (1 − (T_k/T)^{1/3})), τ_k = 3ν_k + 2, on the twin and the GPU (the lead, 2026-10-14,
  M0-D34: «Gamow-shaped terms (Recommended)»); built by M0-D35.
- Contract §2.4.3's "an implicit solver is a PLAN decision" and §0.5 [M0-T42]'s 32-stage face cap — superseded by contract §0.5
  [M0-D34]: P5 runs unthrottled, an implicit solve on the GPU within the player's frame budget (the lead, 2026-10-14, M0-D34:
  «Implicit solver (Recommended)»; "make sure itll be able to run on the players hardware however haha"); specified by M0-TJ4,
  the cap standing until its build lands.
- Contract §2.4.3's stepping — RKL2, its stage count s, "coefficients frozen at the start of P5", `SR-WARN heat stages <s>` —
  with §0.5 [M0-T42]'s face cap, §1.3.2's P5 row and §1.7's heat row — superseded by contract §0.5 [M0-TJ4]: one implicit
  step in u = a_r T⁴, the light law frozen at the pass's start and the heat exact by Newton passes, Jacobi-PCG on the GPU, the
  heat balance closed, a hard step carried into the next submission, P5 ≤ 0.5 ms a step on the Quadro (the lead, 2026-10-14,
  M0-TJ4: «Re-solve heat, light law fixed (Recommended)», «≤ 0.5 ms per step (Recommended)», «Speed gives first
  (Recommended)»); built by M0-T117–M0-T122, the cap standing until M0-T119 lands.
- Contract §0.5 [M0-TJ4] (7)'s "P5's mean ≤ 0.5 ms per step on the Quadro RTX 4000" (and the line above's "P5 ≤ 0.5 ms a step on the Quadro") — superseded by contract §0.5 [M0-T122]: the cost is accepted as measured, 0.6189 ms a step (idle Quadro, median of 3), no ceiling and no cost block (the lead, 2026-10-14, M0-T122: "I accept the timing as it is … no need to write a D task for that. Just make sure it is known").
- Contract §5.1 G-HEAT2's "RKL2 matches explicit … Scope `heat-rkl2`. Plant `rkl2-s-minus-one`" — superseded by contract §0.5
  [M0-TJ4]: the implicit step against its own system's solve, backward Euler's first order, scope `heat_implicit`, plant
  `solve-two-rounds` (the same answers).
- Contract §0.5 [M0-TJ4] (6)'s "the only ceiling, 16 passes or 4,096 rounds in one step" — superseded by contract §0.5 [M0-D37]:
  64 passes or 4,096 rounds, GPU and twin, no tolerance changed — a heating front through cold gas moves about a cell a pass
  (sun_disk's steps 340–347: 17–37 passes on the GPU) (the lead, 2026-10-14, M0-D37: «Raise to 64 (Recommended)»).
- Contract §5.1 G-FLD's "|F| ≤ c_sb a_r T⁴ (1 + 10⁻⁶) at every face" read on F — superseded by contract §0.5 [M0-TJ4]: the bound
  on the law each pass freezes (its start-state flux); F, now the flux the step moved, is not bounded where cold thin gas heats
  within a pass (the same answers).
- Contract §5.1 G-ORDER's "A cell of equal parts H, He, C, O, Ne and Si heated at a steady rate" read as a cell left to burn —
  superseded by contract §0.5 [M0-V6]: the composition held at equal parts, each group's instantaneous power read along the ramp
  (a burning cell's carbon never passes ε*: C_alpha spends it, M0-T44b) (the lead, 2026-10-15, M0-V6: «Amend G-ORDER
  (Recommended)»).
- Contract §5.1 G-BURN's "X within 10⁻⁴ absolute down to X₀/10" with no burn_dx named — superseded by contract §0.5 [M0-V6]: the
  band graded with burn_dx = 10⁻⁴; the shipped 0.1 reads 2.76 × 10⁻² (the lead, 2026-10-15, M0-V6: «Amend G-BURN (Recommended)»).
- Contract §2.6's Motion, "kick-drift-kick with the grid's g at the sink (bilinear) plus the other sinks' pull softened by r_acc" —
  superseded by contract §0.5 [M0-V7]: a sink feels the exact reaction of its own softened pull on every gas cell (a fixed-order
  sum), two sinks pull softened by the pair's larger r_acc, graded like G-GRAV2 at 10⁻⁵ (the lead, 2026-10-15, M0-V7: «Equal and
  opposite (Recommended)»).
- Contract §5.1 G-JEANS's "A disk cloud (U_th = 0.1|W|, a = 40) … collapsing … until its central Σ is 100 × the start" —
  superseded by contract §0.5 [M0-T62]: U_th = 0.05|W| and the cold pressure off through `overrides`, the 100 ×, J ≤ 1/4, one
  object and 10 % bars unchanged; the 0.1|W| cloud turns round at Σ_max 62.7 and never meets 100 × (the lead, 2026-10-16,
  M0-T62: «U_th = 0.05|W|, keep 100× (Recommended)»).
- Contract §1.9.2's "L (radiated ÷ the frame's sim time)" over the object's own blocks, §1.9.4's ignition "at L_H ≥ 0.5 L" and
  §1.9.3's main_sequence row "L_H ≥ 0.5 L" — superseded by contract §0.5 [M0-D53]: L adds the light of every block outside the
  listed objects to the object whose member block is nearest; ignition and main_sequence at 0.5 L ≤ L_H ≤ 2 L (the lead,
  2026-10-16, M0-V18: «Nearest star (Recommended)», «Burning matches light (Recommended)»).
- Contract §2.5.1's initial ladder "T_H 100", "T_He, T_C, T_Ne, T_O, T_Si 140, 196, 274, 300, 420" and T_H's role "the settled
  Sun-like core runs near μc²/2 ≈ 114", §2.5.3's and §2.7's T_ν "166" and §2.7's a_r "4 × 10⁻⁷" — superseded by contract §0.5
  [M0-D50]: the ladder × 0.56 together (T_H 56 … T_Si 235.2, T_ν 93) and a_r 4 × 10⁻⁵, every bound unchanged; the core settles
  near T_c 36 (the lead, 2026-10-16, M0-V18: «Keep white dwarfs, D50 writes (Recommended)»).
- Contract §1.8.2's "then TOP itself … a Sun-like life in ~10 s at the top rung (B9)", §1.8.6's "At TOP only, f_target becomes
  30 when N_max at 60 binds in ≥ 30 of the last 60 frames, and returns to 60 after 120 consecutive frames …" and "If TOP cannot
  hold at 30 either (G-TOP red on the Quadro), the build stops and the lead is asked again …; shortcut jumps and a slower top
  speed stay unruled", §5.3 G-TOP's "life_wall_s ≤ 12 … Red at 30 frames, the build stops for the lead (§1.8.6)", and the goal
  paragraph's "normal to a Sun-like life in about ten seconds" and "— M0-R3 checks that the top speed can hold it, and a
  shortfall comes back to the lead at M0-TC" — superseded by contract §0.5 [M0-TJ2]: the top rung Max, as fast as the card goes,
  nothing in M0 blocked on performance (the lead, 2026-10-18, M0-TJ2: «we go as fast as the card goes. We dont compromise physics
  over performance for now, maybe later in another milestone will we optimize further but not now, we do not block anything on
  performance»; reports/top_speed.md).
- Contract §2.11 items 1–2 (a = 12 disk, bisection), §2.3.2's "1.77 K₂'²/G²", §2.7's K rows and sigma_bh 4,000, §2.6's "≥ 4 × ns_sigma_max_sb"
  and §5.1 G-CORE — superseded by contract §0.5 [M0-R4]: the continuum ceiling 1.4331 K₂′²/G² from the EoS on the CPU, ns_sigma_max_sb = m_tov_sb,
  Σ_BH 6,000 (the lead, 2026-10-19, M0-R4: «Continuum SCF (Recommended)», «Raise to 6,000 (Recommended)»; reports/cold_ceiling.md).
- Contract §5.3 G-FPS's "p95 frame ≤ 16.7 ms … ≤ 33.3 ms once §1.8.6's switch engaged; the slowest frame ≤ 50 ms at any rung"
  read as bounds — superseded by contract §0.5 [M0-TJ2]: measured and reported, the clamp graded (the same answer).
- Contract §1.4.4's cadence "built only if M0-TG's budget needs it", §5.3 G-CAD and M0-T114 — N/A in M0, superseded by contract
  §0.5 [M0-TJ2] (the same answer: «We dont compromise physics over performance for now, maybe later in another milestone will
  we optimize further but not now»); routed past M0 (reports/top_speed.md § Routed).
- Contract §2.4.4's "E is unchanged by it (the work comes out of thermal energy)" read with no bound — superseded by contract §0.5
  [M0-D58]: the push's work capped at the heat the cell holds above its floor, s·Δt·f with s ≤ 1, GPU and twin (the lead,
  2026-10-19, M0-D58: «Cap the work (Recommended)»).
- Contract §1.3.2 P9's "non-finite guard every 64 steps" and §3.3's exit 5 read as the only scan — superseded by contract §0.5
  [M0-D59]: `headless` (GPU and `--cpu-reference`) and `bench` also scan the state after a run's last step (the lead, 2026-10-19,
  M0-D58: «Scan the last step (Recommended)»).
- Contract §1.4.2's "the energy source Σu·g, time-centred on the predictor's half-step velocity" — superseded by contract §0.5
  [M0-D64]: the work each face's mass flux does across φ, each cell keeping its own Σu·g share and the rest shared by mass
  (the lead, 2026-10-19, M0-D64: «Face work, mass-shared (Recommended)», «Record it, no build (Recommended)»).
- Contract §0.5 [M0-T62]'s "U_th = 0.05|W| … T 2.3666" (G-JEANS) and §0.5 [M0-T39]'s "every sum … within 10⁻⁶ of its scale at step
  200" (G-BOX) — superseded by contract §0.5 [M0-D66]: G-JEANS's cloud at 0.07|W|, G-BOX's every sum at step 100 (the lead,
  2026-10-20, M0-D66: «0.07|W| + file a D (Recommended)», «Grade at step 100 (Recommended)»).
- Contract §3.3's Calibrate line (`calibrate --adapter S --out FILE [--only <item>]`) — extended by contract §0.5 [M0-D72]:
  `[--resume <dir>]` reads an earlier invocation's finished lives back (the lead, 2026-10-20, M0-V9: «is there anything possible
  to do so that the long runs can be resumable? I started the first long run and I probably will need to leave»).
- Contract §0.5 [M0-TJ4] (6)'s "prints `SR-WARN heat unconverged …` — never silent" — narrowed for a `calibrate` life by contract §0.5
  [M0-D74]: counted into `heat_unconverged=<n>` (the progress line, progress.json, the run line), not printed; headless still prints
  (the lead, 2026-10-20, M0-D74: «try to silence those warnings in the command line»).
- Contract §2.11 item 4's "preset clouds run to their ending" — narrowed for m_up_sb by contract §0.5 [M0-D75]: its clouds run until
  `core_collapse` fires, a neutron star or a black hole shows, or their ending; m_bh_sb's and the presets' run to their ending (the
  lead, 2026-10-20, M0-V9: «Stop at core collapse (Recommended)»); its cost: `max_gas_speed` no longer sees an m_up life's gas after
  its collapse. Contract §3.3's Calibrate line gains `[--progress-every N]` (the lead: «make the number of steps an arg to the
  program, id like to try 1000 steps for larger stars and higher values for smaller stars»).
- Contract §2.12.3's dump and §3.3's headless line — extended by contract §0.5 [M0-D76]: beside each dump of a GPU run its observers'
  snapshot `dumps/s_<step>.observers.json`, and `--load-dump FILE --continue` goes on as the cut run would have (`--steps` and
  `--max-steps` from the life's start); without `--continue`, `--load-dump` as before (the lead, 2026-10-20, M0-V9: «can you add another
  task so that it can really resume at the current step or something?»).
- Contract §3.3's Calibrate line and [M0-D72]'s `--resume` — extended by contract §0.5 [M0-D77]: `[--checkpoint-every M]` (the save
  interval, apart from `--progress-every`) and a cut life continued from its last checkpoint instead of from step 0 (the lead,
  2026-10-20, M0-V9: «make sure that the number of display step and the number of save steps can both be arguments that can be set
  seeparately»); [M0-D76]'s snapshot gains `adapter`, `driver` and `physics_hash`.
- Contract §1.9.4's ignition as §0.5 [M0-D53] reads it, §1.9.5's "has held for 10 t.u.", §3.3's `ending` gloss "(… for 10 t.u.)" and §2.11
  item 3's failure "held for 20 τ_dyn" — superseded by contract §0.5 [M0-D78]: ignition on the light window or once the centre has burned a
  tenth of its hydrogen (`by` light|fuel), every remnant's ending at max(10 t.u., 20 τ_dyn) (`hold_t`), the observers' snapshot version 2
  with `x_h_max` (the lead, 2026-10-21, M0-V9: «Burnt fuel also counts (Recommended)», «max(10 t.u., 20 τ_dyn) (Recommended)»).
- Contract §2.5.1's "A per record … others 1 × 10⁻³" for N_Fe and §1.9.3's neutron_star row "X_n,c ≥ 0.5, |v_r| ≤ 0.05 √(GM/R), no sink",
  listed before supernova — superseded by contract §0.5 [M0-D79]: a_n_fe 100; neutron_star on Y_e,c ≤ 0.35, d_c ≥ 0.5, no sink, listed after
  supernova, the |v_r| term back with M0-D71 (the lead, 2026-10-21, M0-R5: «Rate + NS row (Recommended)»; reports/neutronization.md).
- Contract §3.3's `--progress-every N` («a line every N steps», [M0-D75]'s «nothing a life computes changes») and [M0-D77]'s «no chunk is
  shortened: nothing the life computes moves», read for a `calibrate` life — superseded by contract §0.5 [M0-D80]: one-step frames, each a
  progress line, while P5's ceiling or a core collapse holds (the state, never wall time), the snapshot's `one_step`, the line's numbers
  after `stage=` (the lead, 2026-10-22, M0-R6: «The physics (Recommended)», «File a D, build next (Recommended)»).
- Contract §0.5 [M0-TJ4] (4)'s «Jacobi-preconditioned conjugate gradients» — superseded by contract §0.5 [M0-D82]: Jacobi or a geometric
  multigrid V-cycle (Galerkin 2 × 2 levels, a 16 × 16 tile smoother, one coarse workgroup), the stopping test, tolerances and ceiling unchanged,
  Jacobi the default until M0-D84 (the lead, 2026-10-22, M0-R7: «(b) Multigrid V-cycle (Recommended)»); and M0-D82's `Deliver:` «|⟨Mx, y⟩ −
  ⟨x, My⟩| ≤ 10⁻⁴ |⟨Mx, y⟩|» — graded normwise, ≤ 10⁻⁷ ‖Mx‖ ‖y‖ (the lead, 2026-10-22, M0-D82: «Normwise ≤ 1e-7 (Recommended)»).
- M0-D38's ruled bound on heat_gauss's GPU pulse, «the edges' sum is held to 1e-7 of the pulse's heat», and M0-D83's `Deliver:` «every bound
  unchanged» read for it on the V-cycle — superseded by contract §0.5 [M0-D83]: on the V-cycle ≤ 10⁻⁵ of the pulse, (4)'s tolerance, Jacobi's
  10⁻⁷ kept (measured 5.9 × 10⁻⁷: the closure δ ≈ 4 × 10⁻⁷ lifts the floor's edge cells) (the lead, 2026-10-22, M0-D83: «V-cycle at 1e-5
  (Recommended)»).
- Contract §0.5 [M0-TJ4] (3)'s «the first iterate is the pass's start state (nothing kept from the previous step …)» and §2.12.3's dump
  layout — superseded by contract §0.5 [M0-D85]: on the V-cycle T⁰ + the last step's P5 rate × Δt, the rate the dump's 17th plane
  (`resume.rate`), zeroed on upload (the lead, 2026-10-22, M0-R8: «(c) Rate extrapolation (Recommended)»).
- Repo facts' «the claim run stays on linux-pc» (M0-D80) — superseded by R18: every GPU run, the tests and the claim, on laserax-ai-server
  (the lead, 2026-10-22, M0-D85: «run everything thats on the gpu, even the tests on laserax-ai-server, when executing tasks on laserax-ai»).
- Contract §2.5.2's sub-cycling and §1.7's reactions row, read for N_Fe (linearised backward-Euler sub-steps under burn_dx and burn_dt, the
  latch from any sub-step's rate) — superseded by contract §0.5 [M0-R10]: N_Fe in closed form over the pass's Δt before the other records
  sub-cycle, the latch at the pass's start; the twin since M0-D86, the GPU with M0-D87 (the lead, 2026-10-22, M0-R10: «(b) Exact N_Fe
  (Recommended)»).
- Contract §1.9.3's white_dwarf and planetary_nebula rows, read for any object mass — superseded by contract §0.5 [M0-D81]: each also needs
  M ≤ m_ch_sb (the run's own sheet ceiling), a heavier star's quiet degenerate centre reading iron_core until it collapses (the lead,
  2026-10-23, M0-D81: «Refuse above m_ch_sb (Recommended)»).
- Contract §1.9.3's neutron_star row as §0.5 [M0-D79] reads it, matching a centre that never passed core collapse — superseded by
  contract §0.5 [M0-D88]: it also needs the object's passed core collapse, so a centre neutronized inside one frame shows core_collapse
  and its event first (the lead, 2026-10-23, M0-D88: «NS needs a collapse (Recommended)»).
- Contract §0.5 [M0-TJ4] (6)'s «the only ceiling … commits the last iterate with (5)'s closure» — superseded by contract §0.5 [M0-D89]:
  past the ceiling the step commits by limited fluxes (each gas cell's outflow scaled to its heat above the floor, δ = 0), after the 40
  M₀ life's step 2,517 emitted 4.19 × 10¹⁶ from a cell holding 5.4 and its closure wiped the grid's heat (the lead, 2026-10-23, M0-D89:
  «Limited fluxes (Recommended)»).
- Contract §2.6's Formation («all four at one cell … Σ ≥ Σ_BH», a sink only from one cell's density) — superseded by contract §0.5
  [M0-R6]: a sink also forms at a local Σ maximum whose disk r ≤ 4 holds m_tov_sb or more, neutronized (Y_e ≤ 0.35 over r ≤ 3), with
  §2.6's other three tests, and takes the disk; Σ ≥ Σ_BH alone still forms one (the lead, 2026-10-23, M0-R6: «Mass sink, r=4
  (Recommended)»; built by M0-D90).
- Contract §5's G-WATCH («names … a calibration value other than the physics hash», a ban on every §2.11 key in the step code) —
  superseded by contract §0.5 [M0-D90]: the step may name `m_tov_sb`, computed in-process from the run's neutron sheet for [M0-R6]'s mass
  test, and names no `calibration` at all (the lead, 2026-10-23, M0-D90: «Named exemption (Recommended)»).
- M0-D90's Done-when «`--bracket 15000,15015`'s life 01 forms none (the massive preset's range)» — N/A, superseded by the lead's
  «Keep the rule (Recommended)» (2026-10-23, at M0-D90): measured on the RTX 4090, the 15,000 life's central fallback core holds 1.03 ×
  m_tov_sb within 4 cells from step 2,998 and forms its sink at step 3,044 as its collapse brings Y_e over r ≤ 3 to 0.35 (a fallback
  black hole); m_bh_sb's bisection lands between 4,572.394 and 15,000 and preset_mass_sb places the massive preset under it
  (tasks/M0-D90.md § The probe).
- Contract §1.9.3's core_collapse row, read for any object mass — superseded by contract §0.5 [M0-D92]: it also needs M > m_ch_sb (none
  refused without a ceiling), a lighter remnant's cold centre overshooting into N_Fe's gate reading white_dwarf or planetary_nebula as it
  bounces back (the lead, 2026-10-25, M0-D92: «No collapse ≤ m_ch_sb (Recommended)»).
- Contract §0.5 [M0-TJ4] (3)–(4)'s ends and (6)'s «the only ceiling», read with sums that may leave the float's range — superseded by
  contract §0.5 [M0-D93]: a pass's bounds or a round's sums not finite end the step there, unconverged, committed by limited fluxes
  with the warning, after the 20 M₀ life's step 16,127 read 17 NaN-stopped passes as converged and booked 3.4 × 10²⁵ radiated (the
  lead, 2026-10-25, M0-D93: «End it unconverged (Recommended)»).
- Contract §2.7's a_r and c_sb (4 × 10⁻⁵ after [M0-D50], 200) — superseded by contract §0.5 [M0-D94]: c_sb 400 and a_r 2 × 10⁻⁵, a_r
  c_sb held (the light, σ and K1 unchanged; the push halved, the bound 200), after the 20 M₀ life's gas passed c_sb/2 at 100.10 in the
  supernova's ejecta and 100.97 beside its black hole on the RTX 4090; the calibration measured again from item 1 (the lead,
  2026-10-25, M0-D94: «c_sb 400, a_r 2e-5 (Recommended)»).
- Contract §1.9.2's objects of blocks alone and identity by block mass (M0-T58's tracker: a sink's mass counted in its block) —
  superseded by contract §0.5 [M0-D70]: a sink no block of the summary holds is an object of its own, and a pair shares the common
  blocks' lesser gas plus each sink's lesser mass, the sink matched where its velocity carries it, after the 20 M₀ life's black-hole
  object took 59 ids as its sink crossed blocks (RTX 4090) (the lead, 2026-10-26, M0-D70: «Its own object (Recommended)», «Share by
  sink (Recommended)»).
- Contract §2.3.5's Δt reduced over the active box alone, an empty box giving the largest finite f32 (M0-T21) — superseded by
  contract §0.5 [M0-D96]: an empty box steps on the floor gas of hydrogen, C / [2 c_floor], after M0-D66's T 3 cloud froze its clock
  at t 11.0 and threw its sink to infinity once the sink had eaten the gas (RTX 4090) (the lead, 2026-10-26, M0-D96: «Floor gas's step
  (Recommended)»).
- Contract §3.3's `ending` "(any object holds a remnant stage for 10 t.u.)" as [M0-D78] read it, the life stopped and §2.11 items
  3–4 and 6 placed by the first object to hold its remnant — superseded by contract §0.5 [M0-D95]: a life's ending is its heaviest
  object's (a lighter object's `ending` logged, never stopping the run or placing it; `until.object` names the ending's object),
  after m_bh_sb's 20 M₀ life stopped on a 1,700 fragment's failed_star while its 30,618 black hole held (RTX 4090) (the lead,
  2026-10-26, M0-D95: «Heaviest object's (Recommended)»).
- Contract §1.9.2's identity on a split as [M0-D70] kept it, «the larger part on a split» keeping the id — superseded by contract §0.5
  [M0-D97]: the part holding the previous object's centre (its max-Σ cell's block, or its heaviest sink) keeps the id and its history
  whatever its share, the others new, after m_bh_sb's 11,200.210 life's ejecta kept the star's id and its neutron-matter core, a fresh
  id with no passed collapse, ended the life failed_star (RTX 4090) (the lead, 2026-10-26, M0-D97: «Id follows the centre
  (Recommended)»).
- M0-TZ's carried asks [M0-R1] ("Place the web release — hosting, CDN, ads, consent — in a milestone with the lead. Consent rules
  and revenue are the lead's to rule.") and [M0-R2b] ("place the site's game picker with the web release"), and [M3-TB1] whole —
  `../space_tykun`'s note for this project (its `milestones/m3/m3_contrat.md` §2.5, never pasted here): this project uploading its
  own build over SSH to `ubuntu@games-box` into `/srv/site/sandbox-reactions/`, the box's serving facts for that path, the shared
  origin's storage keys, the « Ready to go up » tracker line and "Place this game's web release — with M0-R1's hosting, CDN, ads
  and consent — in a milestone with the lead" — N/A — superseded by the lead's direction of 2026-10-27 (M0-TJ5's Ask: «I want
  them to know we're taking over the live push of their games and the website itself.»): the site, its picker, look and ads, the
  CDN, the consent rules, `games-box` and every live deploy are `../tykun_games`'s, and what the site needs from a build arrives
  from it as a flag through the lead (R20; M0-TZ's flag [M0-TJ5]) (the lead, 2026-10-27, M0-TJ5: «Confirm, as listed
  (Recommended)»).
- milestones.md's worklist item 4 ("Before the web release — an AWS account for a small Lightsail box with a Lightsail CDN in
  front, and the AdSense H5 Games Ads application; the consent rules for ads to rule") and reports/web_hosting.md § Routed to
  (M0-TZ places the web release; the worklist's AWS account and AdSense application) — N/A — superseded by the lead's direction
  of 2026-10-27 (M0-TJ5's Ask): the box exists (`games-box`, the $5 Lightsail bundle in Canada (Central), up since 2026-10-16),
  and the CDN, the AdSense application and the consent rules are the site's; the report stays, unedited, as the record
  `../tykun_games` reads (the same answer).
- Contract §6.2.5's "Hosting: none in M0 (reports/web_hosting.md feeds a later milestone)" — superseded by contract §0.5
  [M0-TJ5]: no hosting in this project, in M0 or after it; and reports/bootstrap.md §3's row « "run on a website" — a full web
  release | later, beyond M0's web smoke | after M0 » narrowed by the same: what M0-TZ places from it is this game's static web
  build, marked ready in the tracker (R20), its going live `../tykun_games`'s (the lead's direction of 2026-10-27, M0-TJ5's Ask;
  the same answer).
- Not retired, explicitly (M0-TJ5's negative scope): the web build — `web/` (Trunk, the wasm bundle, `web/smoke.mjs`), G-WEB and
  « a tiny web build kept alive from the first build lot »; contract §6.2.4's no browser threads and no COOP/COEP, which R20 now
  also holds for the site's ads; Steam; reports/web_hosting.md, kept as the record (`../tykun_games` reads it); the folder
  `/srv/site/sandbox-reactions/` on the box, `../tykun_games`'s to fill; an in-game ad break (H5 Games Ads at game breaks, as
  web_hosting.md records) — if `../tykun_games` rules one, its hook arrives as a flag and is built here; M0-TZ placing this
  game's own web build in a milestone with the lead.
- Contract §3.4's Desktop "`--status-port <port>` serves `GET /status` on 127.0.0.1", the bind failing with `SR-ERROR` when the port
  is taken — superseded by contract §0.5 [M0-D99]: a taken port moves to the first free one of 47811–47830, printed as
  `SR-STATUS-PORT <n>`; `SR-ERROR` only when none is free; §3.6 and launch.py unchanged (the lead, 2026-10-28, M0-D99: «(b) App
  side only»).
- Contract §0.5 [M0-D64]'s split of the face work ("cell i taking (½p_i + w_i(q − ½(p_i + p_j)))(φ_i − φ_j)Δt … and j the rest"),
  §0.5 [M0-D66]'s "G-JEANS's cloud at U_th = 0.07|W| (T 3.3132)" and the ledger's leave momentum band 5e-3 — superseded by contract
  §0.5 [M0-D68]: the diffusive flux's work, when a cost, paid first from the pair's relative motion (at most half its kinetic
  energy, by equal and opposite impulses), the rest shared by mass; G-JEANS back at 0.05|W| with J ≤ (1 + 10⁻²) × its start; the
  leave band 1e-2 until M0-D69 (the lead, 2026-10-28, M0-D68: «Relative motion first (Recommended)», «(1 + 10⁻²) × start
  (Recommended)», «Back to 0.05|W| (Recommended)», «1e-2 until D69 (Recommended)»).
- Contract §0.5 [M0-D68]'s "the ledger's leave momentum band 1e-2 until M0-D69" and the one-sided difference on the box's
  outermost ring (M0-T29's twin, M0-T31's shader) — superseded by contract §0.5 [M0-D69]: central differences in every cell of the
  box, the ring reading the zero-padded convolution's exact φ one cell past the box; the leave band 7e-3 (the lead, 2026-10-28,
  M0-D69: «7e-3 (Recommended)»).
- Contract §5.2's G-MULTI "20 τ_dyn" — superseded by contract §0.5 [M0-T74]: the run is to `sim_time:40` (about 11 τ_dyn), the two
  Sun-like clouds 240 cells apart falling into one object by 20 τ_dyn (the lead, 2026-10-28, M0-T74: «Shorten to 40 t.u. (Recommended)»).
- Contract §1.3.2's P1 row "Δt_n is the reduction P9 made at the end of step n−1" for a step whose P0 paints or drops (M0-T76: the
  first stroke into an empty world stepped on the floor gas's 0.4472 against its own 0.0673) — superseded by contract §0.5 [M0-D110]:
  P0 re-makes the CFL term over the box it left after a paint or drop, Δt_n the lesser of it and P9's (whose gravity term stands);
  every other step keeps its bits (RTX 4090) (the lead, 2026-10-29, M0-D110: «Gas-speed term only (Recommended)»).
- Contract §1.11's Glow bullet "bright pass on luminance > 0.8 at half resolution" (M0-T81 built it as written; M0-V11 measured that it
  blooms only 3,907–13,478 K matter, so star cores and a massive star's whole disk never glow) — superseded by contract §0.5
  [M0-D112]: the bright pass keeps a cell whose brightest channel (max R, G, B of the Glow view's cell image) is above 0.8; the rest of
  §1.11 and §4.11's glow unchanged (the lead, 2026-10-29, M0-V11: «Brightest channel > 0.8 (Recommended)»).
- Contract §1.11's Heat, Element ("vacuum black") and Density views and Glow's haze, which give vacuum no colour of its own (M0-T80 built
  §1.11 as written; M0-V11 measured the empty corners #000000 / #000004 / #440154 against §4.11's bg.space) — superseded by contract §0.5
  [M0-D113]: a cell whose blended Σ is below Σ_vac draws bg.space #05070D in every view, ahead of the view's formula (the lead, 2026-10-29,
  M0-V11: «bg.space in every view (Recommended)»).
- Contract §3.4's Web "`python3 -m http.server 47812 --bind 127.0.0.1 --directory web/dist`" and §3.6's `web` row "the static server on
  47812" (one fixed port, refused when held: VS Code's helper held it all evening on win-laptop and `web` and `desktop[launcher]` read
  NO-GO, M0-D65, M0-V11-win) — superseded by contract §0.5 [M0-D107]: the same command on 47840 with launch.json's `port_range`
  [47840, 47859], the next free port taken and printed when 47840 is held (the lead, 2026-10-30, M0-D107: «I want the launchers to
  automatically find the next free port, lets not be stopped by that»).
- Contract §4.2's "Selected tool, rung and view use `accent`" and its top bar with no widths (in an empty world only the Pause/Play
  label told a pause from a run, and pausing, a rung or the cap resized the button or the readout and moved the rest of the bar,
  M0-V12) — superseded by contract §0.5 [M0-V12]: the paused readout in `accent`, the Pause/Play button and the readout of one width
  each (the lead, 2026-10-30, M0-V12: «I dont like when an interaction in a bar moves the whole bar»).
- Contract §4.12's heading ("," groups thousands) as read for the speed readout's `{actual}` (unsettled since M0-T85: `sig2` wrote it
  ungrouped, format.rs's `actual_number` grouped) — superseded by contract §0.5 [M0-V12]: `{actual}` is never grouped, as the rungs
  ×1000 are written (the lead, 2026-10-30, M0-V12: «Ungrouped, ×4200 (Recommended)»).
- Contract §1.8.5's banner that "shows until then" and §4.2's "after an automatic slow-down", for a supernova at ×0.1 (it slowed
  nothing, so the core-collapse banner stayed and `banner.supernova` never showed in a life the player left alone, M0-V12) —
  superseded by contract §0.5 [M0-V12]: the supernova takes over a showing core-collapse banner, the rung unchanged (the lead,
  2026-10-30, M0-V12: «Supernova takes over (Recommended)»).

- Contract §1.11's fixed Glow brightness at 3,000 K and fixed Heat (log₁₀ T_K 1–10) and Density (log₁₀ Σ −6–4) maps, §1.12's
  settings kept by Clear and §4.2's hint under the world at scale s — superseded by contract §0.5 [M0-R12]: a Glow setting S (G3,
  today's at 3,000 K), each map's range with «Fit to view», the legend under the world, the zoom, all kept by Clear (the lead,
  2026-11-01, M0-R12: «G3 luminance dimming (Recommended)», «Range + Fit to view (Recommended)», «Under the world
  (Recommended)», «File a zoom block after R12's (Recommended)»).
- Contract §1.9.3's failed_star row on d_c alone («never ignited, d_c ≥ 0.5, L_H < 0.01 L») and its display hold showing any
  stage held 3 frames — superseded by contract §0.5 [M0-D134]: failed_star also needs the protostar's settled test, and the shown
  stage never steps back (no cloud, collapsing or protostar once ignited; before it, those three forward only; no cloud after a
  failed star) (the lead, 2026-11-02, M0-D134: «Must be settled (Recommended)», «Display holds (Recommended)»).
- Contract §1.9.3's white_dwarf and planetary_nebula rows without a motion test («ignited, d_c ≥ 0.5, L_nuc < 0.01 L …»,
  «… L_nuc < 0.1 L, M_unb ≥ 0.05 M», each M ≤ m_ch_sb by [M0-D81]) — superseded by contract §0.5 [M0-D140]: each also needs the
  settled test, |v_r| ≤ 0.05 √(GM/R) and 2P/|W| ≥ 0.8 (the lead, 2026-11-02, M0-D140: «(a) Settled test (Recommended)»).
- Contract §1.9.3's main_sequence row on the light window («L_H ≥ 0.5 L and X_H,c ≥ 10⁻³», 0.5 L ≤ L_H ≤ 2 L by [M0-D53]) —
  superseded by contract §0.5 [M0-D141]: an ignited object whose largest burning group is hydrogen (≥ 0.1 L_nuc, X_H,c ≥ 10⁻³);
  the window holds the ignition event alone (the lead, 2026-11-02, M0-D141: «(a) Largest group (Recommended)»).
- Contract §4.2's Centre and §4.6's star readout as listed (nothing marks a remnant) — superseded by contract §0.5 [M0-D138]: from
  core collapse on, a ring around each such object's centre in every view and zoom, and `star.remnant` under the fate or ending
  line (the lead, 2026-11-02, M0-D138: «Mark the remnant (Recommended)», «From core collapse on (Recommended)»).
- Contract §1.8.5's «with the setting off, steps run through events» and §4.2's banner «after an automatic slow-down» as the only
  ignition sign — superseded by contract §0.5 [M0-D139]: an ignition the slow-down's banner does not name shows
  `banner.ignition_notice` in the banner's slot for 5 s, the rung unchanged (the lead, 2026-11-02, M0-D139: «(a) Notice in banner spot»).
- Contract §5.4 G-WEB's «the canvas's central 100 × 100 px are not all black» (a Chrome screenshot clip) — superseded by
  contract §0.5 [M0-D144]: the canvas's own image (`toDataURL`) at the world's centre (`srWorld`, §3.5) holds opaque pixels more
  than 2 levels off `bg.space`, the world's corners `bg.space` within 1; plant `web-preset-not-dropped` (M0-D144, 2026-11-02, the
  lead at M0-V13: «File a D for a stricter case (Recommended)»).
- Contract §1.11 [M0-R12]'s «clamped so the world covers the centre area whenever it is larger» and §4.2's «clipped to the centre
  area» — superseded by contract §0.5 [M0-D146]: the zoomed world is clipped to the world's place at z = 1 (`legend::layout`'s world
  rect), the drag clamped so it covers that rect, the legend and the hint kept in view (M0-D146, 2026-11-02, the lead at M0-V13:
  «Keep the build (Recommended)»).
- Contract §2.3.4's scheme updating every cell, §1.4.2's sources with work at every face ([M0-D64], [M0-D68]), §2.7
  [M0-T19]'s P8 rules, §2.9's momentum «grid · sinks · escaped» with G-CONS's sum, §1.3.2's P4 and P8 rows and §2.12.3's
  dump — superseded by contract §0.5 [M0-TG2]: the held core — P8 holds a cold dense cell (σ_hold 10, η_hold 0.1, code
  constants until M0-D71g moves them into physics.json), P4 leaves it still behind reflecting walls with no work across
  them and one-way inflow, a momentum term `held`, the mask in dumps, a new row G-REST (the lead at M0-R11, 2026-10-28:
  «H, held core (Recommended)», «η 0.1 (Recommended)», «Measured bounds (Recommended)»; at M0-TG2, 2026-11-02: «Code
  now, move later (Recommended)»).
- Contract §2.3.4 [M0-TG2]'s and §1.4.2 [M0-TG2]'s one-way inflow into a held cell, and §5.1 G-REST's disk momentum to
  10⁻⁶ of Σm|u| — superseded by contract §0.5 [M0-D71]: no inflow (item 2 to M0-D71f, measured on the GPU lives), the
  disks' momentum printed and the `held` booking graded to 10⁻¹² on strip cases (measured by M0-D71: inflow took the M 900
  disk's floor to 392.6 against 10; source-term gravity drifts 1.4 × 10⁻² of Σm|u| with no hold; the lead at M0-D71,
  2026-11-02: «Walls only, inflow later (Recommended)», «Booking exact, disks reported (Recommended)»).
- Contract §1.4.2 [M0-TG2]'s and §2.3.4 [M0-TG2]'s held cell «its Σg·Δt booked `held`» with P2's whole field —
  superseded by contract §0.5 [M0-D149]: a held cell's spared Σg·Δt reads the pull of all but the held cells (φ_pull of Σ
  with the held cells emptied, plus the sinks), solved by P2 beside φ while a cell is held; the core's own pull is never
  booked (measured by M0-D149 on the RTX 4090: P2's f32 field nets ≈ 1.5 × 10⁻⁴ of Δt·Σ·g a step, 5 × 10⁻⁸ of its Σ|·|,
  and the held core carried it all — 2.0 × 10⁻³ of Σm|u| on the M 900 disk over 1,000 steps, the twin's 10⁻¹¹; the lead
  at M0-D149, 2026-11-03: «Free-gas field (Recommended)»).
- Contract §0.5 [M0-D71]'s «the disks' grid + held momentum printed, not graded», for the ledger's M 900 disk — superseded by
  contract §0.5 [M0-D151]: its `held` over 1,000 steps graded at or under the invariant's drift without `held` (measured by
  M0-D151 on the RTX 4090 with g_pull: 6.6 × 10⁻⁶ against 2.0 × 10⁻⁵; the lead at M0-D151, 2026-11-03: «Grade it
  (Recommended)»).
- Contract §0.5 [M0-TG2]'s items 1–2 as «the forms the build starts from», §2.7 [M0-TG2]'s hold test «its N_Fe rate is 0: X_Fe = 0
  or Y_eΣ ≤ Σ_N» with its release «or its N_Fe rate is > 0» and the ceiling-release fallback, §2.3.4 [M0-TG2]'s one-way inflow and
  §1.4.2 [M0-TG2]'s inflow face — superseded by contract §0.5 [M0-D71f]: no iron is ever held (X_Fe = 0, released once X_Fe > 0;
  built by M0-D152) and a held wall passes nothing either way, final (measured by M0-D71f on the RTX 4090: the gate form froze
  the 4,572 life's iron core at the gate and a dozen iron clumps of the 20 M₀ life, which formed no sink by step 8,807 where the
  tree before M0-D71's build forms one at step 2,600; the lead at M0-D71f, 2026-11-03: «Hold no iron (Recommended)», «Walls only,
  final (Recommended)»).
- Contract §0.5 [M0-R13]'s and §2.7's neutron hold gated by «the cell's object (§1.9.2)» against m_tov_sb, and §2.3.4's «none
  whose object reaches m_tov_sb» — superseded by contract §0.5 [M0-T130]: the gate reads the mass within r ≤ 24 cells of the
  cell, never the observer's object (G-WATCH), built and measured by M0-D153; until then no neutron-rich cell is held (measured
  by M0-T130 on the RTX 4090: the Massive object stays over m_tov_sb until step ≈ 3,600, its core already spread; P3's r ≤ 4
  disk held the Massive core but froze the 20 M₀ Repro's clumps, no sink by step 3,062, and was reverted; the lead at M0-T130,
  2026-11-03: «P3's disk mass (Recommended)», «Build it here (Recommended)», «Revert to M0-D152's rule (Recommended)», «Same
  gate, r ≤ 24 (Recommended)»).
- Contract §2.3.4's «MUSCL-Hancock with the monotonised-central slope limiter on primitive variables (Σ, u, v, Π, X_i)», Σ and Π
  limited apart everywhere — superseded by contract §0.5 [M0-D155]: the thermal bracket, a slope whose either face would hold
  Π/Σ above 2 × the largest of its three cells' takes Σ and Π flat on that axis, the twin and the GPU alike (measured by M0-D155
  on the RTX 4090: in the Massive life's step 3,326 a dense cell faced a thin one with its Σ and a third of its Π, c 691 against
  8–13, and P4 alone sent the thin cell to |v| 2,135, the twin alike; the life read 6 such leaps and 13 heat warnings without
  the bracket, none with it, ending neutron_star; the lead at M0-D155, 2026-11-04: «Thermal bracket, K 2 (Recommended)»).
- Contract §0.5 [M0-D76]'s observers' snapshot as [M0-D78] left it, «the observers' snapshot is version 2» — superseded by
  contract §0.5 [M0-D103]: version 3, its `watch` gaining `ages` (the age clock of summary.json's `age_years`), a version-2
  snapshot refused by `--continue` and not used as a checkpoint, scenes/sun_13593.observers upgraded in place with an empty
  clock (measured by M0-D103 on the RTX 4090: the Sun-like preset cut at step 768 and continued writes the unbroken life's
  `age_years`, 3,279,122.408 years; the lead at M0-D103, 2026-11-04: «Upgrade it in place (Recommended)»).
- Contract §0.5 [M0-R13]'s «N_Fe's q_ratio re-set so the collapse keeps nature's share — … the core's P4 heat over the heat N_Fe
  absorbs inside [7.5, 30] … the value M0-T131 measures and writes, K8's «not tuned» lifted for this one row» and §2.5.1's N_Fe row
  «−1.14 until M0-T131 writes the value» — superseded by contract §0.5 [M0-T131]: the bound is withdrawn for M0 and q_ratio stays
  −1.14 (K8, not tuned) (measured by M0-T131 on the RTX 4090, reports/nfe_cost.md: the bound is reachable, q −0.0092 reads 14.93,
  but lies in a shallow collapse and breaks life 01's neutron star; the lead at M0-T131, 2026-11-05: «Keep −1.14, BLOCKED
  (Recommended)», then «well do your recommendation»).
- Contract §5.2 G-FIT's «no bound gas (½|u|² + ε_th + φ < 0) comes within 16 cells of an edge of the default world» — superseded by
  contract §0.5 [M0-T99a]: no bound gas denser than Σ = 10⁻² within 16 cells of an edge, a fainter bound halo allowed (measured by
  M0-T99a on the RTX 4090, reports/lives.md § Tuning — the mass ladder: the Massive's bound edge halo peaks at Σ 7.6 × 10⁻³, its
  bound gas at Σ ≥ 10⁻² ≥ 30 cells from an edge, and no physics.json lever in its bounds empties the halo; the lead at M0-T99a,
  2026-11-06: «Reword G-FIT (Recommended)», «Density floor 10⁻² (Recommended)»).
- Contract §0.5 [M0-D37]'s «the ceiling is 64 passes or 4,096 rounds in one step, on the GPU and the twin» — superseded by contract §0.5
  [M0-D159]: 128 passes or 4,096 rounds, every tolerance unchanged (measured by M0-D159 on the RTX 4090, tasks/M0-D159.md: the Massive
  disk's step 930 needs 97 passes on the f64 twin and 94 on the GPU from the same P5 start; at 128 the life ends with no warning; the
  lead at M0-D159, 2026-11-06: «128 passes (Recommended)»).
- Contract §1.9.3's settled test on pressure alone («2P/|W| ≥ 0.8», the protostar's, failed_star's by [M0-D134], white_dwarf's and
  planetary_nebula's by [M0-D140]) — superseded by contract §0.5 [M0-D160]: the support counts the ordered rotation about c,
  2(P + K_t)/|W| ≥ 0.8, one test for the four rows, the observers' snapshot carrying K_t (an older one restores with K_t 0) (measured
  by M0-D160 on the RTX 4090: m_up_sb's light end, a cold iron core in a Keplerian disk, 2P/|W| 0.356 and 2(P + K_t)/|W| 0.93–0.95;
  the lead at M0-D160, 2026-11-07: «(b) Rotation counted (Recommended)», «All four rows (Recommended)», «Read missing as 0
  (Recommended)»).
- Contract §0.5 [M0-D78]'s ending hold «max(10 t.u., 20 τ_dyn), τ_dyn the object's at that frame» — superseded by contract §0.5
  [M0-D161]: τ is the degenerate centre's, √(r_deg³ / G M_in) over the blocks whose densest cell has Π_cold / Π ≥ 0.5, else τ_dyn;
  the snapshot carries `tau_deg` (an older one restores with none) (measured by M0-D161 on the RTX 4090: m_up_sb's life 08, a cold
  core mostly of 4–5 cells in a Keplerian disk of R₉₀ 76–148, τ_deg 0.28–12 against τ_dyn 20–50 — it ends at step 201,248 of its
  s_200000 resume, never before; the lead at M0-D161, 2026-11-08: «(a) Core's τ (Recommended)», «R₉₀ as today (Recommended)»).
- Contract §1.6.2's H_burn terms «(T_H, 4, 1) + (1.15 T_H, 18, 1)» and physics.json's T_He 78.4 (§0.5 [M0-D50]) — superseded by
  contract §0.5 [M0-D163]: the second term at 1.6 T_H and T_He 100 (measured by M0-D163 on the RTX 4090: the 1.15 term ran the
  Massive's centre to iron in one pass and flashed the Giant mid-main-sequence; 1.6 alone broke G-ORDER, T_He 100 restores it; the
  lead at M0-D163, 2026-11-09: «Crossing 1.6·T_H (Recommended)», «A: 1.6 + T_He 100 (Recommended)»).
- Contract §2.10's «a uniform temperature set so U_th = 0.1|W|» — superseded by contract §0.5 [M0-D163]: U_th = min(0.5, 0.1 ·
  max(1, M/M₀)²) |W| (measured by M0-D163 on the RTX 4090: the cold Massive and Giant overshoot at the bounce to T_c 91 and 153
  and flash; warmer they hold a main sequence, the Sun-like only cold; the lead at M0-D163, 2026-11-09: «0.1·max(1,M/M₀)², cap
  0.5 (Recommended)»).
- Contract §2.7's «a_r, c_sb, κ₀, k_cond 4 × 10⁻⁷, 200, 1, 0» as §0.5 [M0-D94] left it (a_r 2 × 10⁻⁵, c_sb 400) — superseded by
  contract §0.5 [M0-D163]: c_sb 800 and a_r 1 × 10⁻⁵, a_r c_sb held (measured by M0-D163 on the RTX 4090: at c_sb 400 the long
  main sequences lost 50–56 % of the Massive's and Giant's mass through the edge and moved every calibrated threshold; at ×2 each
  keeps main_sequence → he_core → iron_core with 4,790 and 9,564 sb left; the lead at M0-D163, 2026-11-09: «Curb the mass loss
  first (Recommended)», «Adopt c_sb 800 (Recommended)»).
- Contract §2.5.2's sub-cycling with no bound on a cell's heating but the sub-step limits (burn_dx, burn_dt) — superseded by
  contract §0.5 [M0-D165]: the burn clock, a cell's specific nuclear heating held to q_max = ε_th c_s / K_burn per unit time
  (K_burn = physics.json's `burn_k`, 10; 0 turns it off), the reactions and heat per unit mass unchanged (measured by M0-D165 on
  the RTX 4090: the Giant's centre went H → Fe inside one P6 pass at T 128.5 with X_H,c 0.72, the Massive's at T 115 with X_H,c
  0.17, whatever the step; with the clock neither life made iron to the cap and the Sun-like's ignition and main sequence were
  unchanged; the lead at M0-D165, 2026-11-10: «Burn limiter (Recommended)», «Rescope + file 2 Ds (Recommended)»).
- Contract §1.9.3's white_dwarf row settled by the settled test alone (as §0.5 [M0-D140] and [M0-D160] read it) — superseded by
  contract §0.5 [M0-D168]: settled, or at rest (|v_r| ≤ 0.05 √(GM/R)) on a wholly degenerate centre (d_c ≥ 0.9); the other three
  rows keep the test alone (measured by M0-D168 on the RTX 4090: m_up_sb's 3,351 sb life on the burn clock leaves a 7 × 7-cell C/O
  remnant at d_c 1.000 and rest whose 2(P + K_t)/|W| reads 0.779, shown main_sequence to the cap; the lead at M0-D168, 2026-11-10:
  «Degenerate at rest (Recommended)»).
- Contract §1.9.3's si/o/ne/c_burn rows on the star's L_g («the largest L_g and ≥ 0.1 L_nuc») and iron_core on its L_nuc, [M0-D50]'s
  T_Ne 153.4, T_O 168, T_Si 235.2 and §5.2 G-STAGES' massive c_burn and ne_burn — superseded by contract §0.5 [M0-D169]: the rows
  read what c itself burns, q̇_c,g by the rate laws (the largest, its fuel at c ≥ 10⁻³; iron_core once no fuel burns at c), the
  ladder T_Ne 118, T_O 122, T_Si 135, c_burn and ne_burn parenthesised (measured by M0-R15 and M0-D169 on the RTX 4090: the
  Massive's centre burns O, Si and to X_Fe,c 0.999 under a hydrogen shell that always leads — and that also burns in c's own
  4 × 4 block —, shown he_core / main_sequence / he_shell; on the new rows he_core → o_burn → si_burn → iron_core; the lead at
  M0-R15, 2026-11-10: «Ladder down + core labels (Recommended)»; at M0-D169: «Centre block (Recommended)», then «Centre cell's
  rates (Recommended)»).
- Contract §2.7's a_r 1 × 10⁻⁵ and c_sb 800 (§0.5 [M0-D163]) and §2.5.1's K_burn 10 ([M0-D165]) — superseded by contract §0.5
  [M0-D166]: a_r 6.25 × 10⁻⁷, c_sb 12,800 (a_r c_sb held, the push per unit flux ÷ 16) and K_burn 40 (measured by M0-D166 on the
  RTX 4090: the Massive reaches X_Fe,c 0.61 with 7,168 sb against 3,252, the Giant keeps 41,836 through its he_core against an
  eruption to 18,098, the Sun-like's collapse and main sequence to t 60 unchanged; the lead at M0-D166, 2026-11-10: «Push ÷16 +
  burn_k 40 (Recommended)»).
- Contract §2.7's hold test «it holds no iron, X_Fe = 0» and its release «X_Fe > 0 with X_n < 0.5» ([M0-D71f], as [M0-R13] and
  [M0-T130] kept them) — superseded by contract §0.5 [M0-D170]: held at X_Fe < 10⁻³ (a trace read as none), released once X_Fe ≥ 10⁻³
  with X_n < 0.5 (measured by M0-D170 on the RTX 4090: a trace of 2.35 × 10⁻⁹ kept the hold off m_up_sb's remnant, the floor pumping
  1,141 per step into its light; with the trace the floor falls to 3.2, the Massive from s_14000 ends neutron_star at t 372 instead of
  sitting pumped cold, the Giant unchanged; the lead at M0-D170, 2026-11-12: «Trace: X_Fe < 1e-3 (Recommended)»).
- Contract §2.3.4 [M0-TG2]'s held wall as a reflecting wall alone — the free cell keeping whatever momentum into the wall its
  update left it — superseded by contract §0.5 [M0-D172]: the wall is sticky, a free cell's momentum into a held face-neighbour set
  to 0 after P4's update, E as it is, booked `held` (measured by M0-D172 on the RTX 4090 and the twin: the standing infall round
  m_up_sb's held remnant fed P8's floor 1.26 per step, 13.5 % of its light; sticky, ~10⁻⁸ of it, the Massive's ending kept; the lead
  at M0-D172, 2026-11-12: «Sticky held wall (Recommended)»).
- Contract §0.5 [M0-D151]'s grade of the ledger's M 900 disk, «its `held` over 1,000 steps graded at or under the invariant's drift
  without `held`» — superseded by contract §0.5 [M0-D174]: the momentum invariant with `held` graded, its drift over 1,000 steps at
  most 10⁻³ of Σm|u| (measured by M0-D174 on the RTX 4090: under the sticky wall `held` swings in sign every ~50 steps, the old grade
  a coin flip, red at 1.13 × 10⁻² over 1.08 × 10⁻²; the invariant with `held` 6.3 × 10⁻⁴, a held core on P2's whole g 2.6 × 10⁻³;
  the lead at M0-D174, 2026-11-12: «Band 1e-3 (Recommended)»).
- Contract §2.5.1's K_burn 40 ([M0-D166], the Giant's helium event survived as pulses) — superseded by contract §0.5 [M0-D167]: K_burn
  15 (measured by M0-D167 on the RTX 4090: at 40 and 30 the Massive's hydrogen shell cannot burn its infall, the whole star collapses
  inside r ≤ 4 and no supernova follows; at 15–20 the Massive ends supernova then neutron_star and the Giant keeps its mass and ends a
  black hole; the lead at M0-D167, 2026-11-12: «burn_k 15 (Recommended)»).
- Plan M0-D175's Verify 1 «… he_core, he_shell, planetary_nebula, white_dwarf, ending white_dwarf, X_Fe,c < 0.5 · Fail: an iron core
  at c or no ending» and contract §5.2 G-END's Sun-like «white dwarf» read as nature's Sun's (C/O, after a planetary nebula) — superseded
  by contract §0.5 [M0-D175]: the Sun-like's ending white_dwarf, its centre's composition not graded; G-STAGES' sun he_shell and
  planetary_nebula M0-T103's to grade (measured by M0-D175 on the RTX 4090: the Sun-like contracts whole onto ~2 cells and ends
  white_dwarf at X_Fe,c 0.64 with no nebula, every in-bound ladder and the pre-[M0-D166] light alike; M0-V14's old physics ended iron
  too; the lead at M0-D175, 2026-11-13: «Keep shipped, amend (Recommended)»).
- M0-D165's Verify 1 Fail arm «a centre from X_H,c > 0.1 to X_Fe,c > 0.5 within 1 t.u.» (carried in M0-D167's Carried flags) —
  superseded by contract §0.5 [M0-D175] (the second entry): the flash it targets, X_H,c falling by more than 0.1 and X_Fe,c rising
  past 0.5 within 1 t.u. (measured by M0-D175 on the RTX 4090: the recalibrated Giant trips the letter on a centre holding X_H,c
  ~0.2 while its O/Si burns to Fe over ~8 t.u.; the lead at M0-D175, 2026-11-13: «Amend to the flash (Recommended)»).
