# Readouts — the block summary, the ledger, objects, stages, events, and the observers in a headless run

Written by M0-T64 (2026-10-16) from `milestones/m0/m0_contrat.md` §1.9, §2.9, §2.12.2, §3.3 and the handoffs of
M0-T56–M0-T63. The contract owns every number, threshold, formula and string; this page names the parts and points to
the section that fixes them, never a value §1.9, §2.7 or `assets/calibration.json` holds. What waits for later lots is
said where it comes up: the mass and temperature translations (§1.10.1–§1.10.3) are § Translations (M0-T69, M0-T71); the age clock, the predicted
ending (§1.9.5), the selected star (§1.9.6) and the readout strings (§4) are not built yet — M0-T72, M0-T73 and the drawing lots own them.

The one rule (I2, G-WATCH): **observers watch, never drive.** Everything here reads what a step produced and changes
nothing in it. The boundary is in `docs/agent/architecture.md` § The observer boundary.

## The pipeline, in one line
```
 Step::encode_frame ─► summary.wgsl (GPU, per frame) ─► Frame {report, summary} (poll_frames, ≤ 2 frames late)
 Step::run (headless, bare steps) ─► Step::summarise after each chunk ─► the same block summary, read at once
      ─► ledger.rs (f64, + take_booked)           ─► summary.json `ledger`
      ─► watch.rs: Report (f64) ─► Tracker ─► StageTracker ─► objects · stages · events ─► summary.json, events.jsonl
```
| Part | Where | Contract |
|---|---|---|
| block summary | `sr_engine::summary` (`BlockSummary`, `Block`, `Frame`) and `shaders/summary.wgsl` | §1.9.1 |
| ledger | `sr_engine::ledger` (`Ledger`, `Record`, `take_booked`) | §2.9 |
| the f64 hand-over | `sr_engine::watch::report` → `sr_physics::observe::Report` | §1.9, §3.1 |
| objects | `sr_physics::observe::tracker` (`Tracker`, `Object`) | §1.9.2 |
| stages and events | `sr_physics::observe::stages` (`classify`, `ROWS`, `StageTracker`) | §1.9.3, §1.9.4 |
| the run's observers | `sr_engine::watch` (`Watch`, `Until`; the hold `sr_physics::observe::stages::ending_hold`) and `sr_engine::headless` | §2.12.2, §3.3 |

## The block summary (§1.9.1, M0-T56)
- **What it is.** After a frame's last step, over the active box, one 4 × 4-cell block an invocation: the fields §1.9.1
  lists (mass, first moments, momentum, the three energies, ∫Π dA, the ten species masses, Σm·φ, the max-Σ cell, the
  frame's accumulators summed, the unbound mass, the sinks inside). Two box-sized dispatches (`summary_cells` for the
  state, φ and the max-Σ cell; `summary_book` for the accumulators and the sinks' mask); f32 sums in row-major order, no
  atomics, so two runs agree bit for bit.
- **When it is read.** `encode_frame` records it after `latch_end`, outside the frame's timing, and it rides the frame's own
  readback: `Step::poll_frames()` returns `Frame { report, summary }` (`poll()` is unchanged and still returns the
  `FrameReport` alone). It arrives up to two frames late, and the observers take it as it comes.
- **The accumulators** (radiated, nuclear by group, neutrino lost and deposited) are cleared at the *next* frame's start,
  so the booking holds them between frames; a frame that the latch stopped is summarised as the state stands; a cell with no
  φ reads as 0. The reasons are in `milestones/m0/tasks/M0-T56.md`.
- **Cost, unmeasured:** the whole world's blocks buffer is copied each frame (2.9 MB at 600 × 400); it is outside the frame's
  cost and nothing has priced it.

## The ledger (§2.9, M0-T57)
- **What it books.** Mass, energy and momentum, each term in f64 from the summary's fixed-order block partials plus the
  booked terms (`take_booked`: `book_totals` over the booking, the booked groups cleared, the accumulators left to
  `Step::summarise`). The terms and the two invariants (G-CONS) are §2.9's — the page does not restate them. The sinks the
  ledger carries have their ½Mv² in `kinetic`; the box's outside is filled with `sigma_floor` a cell; `cleared` is booked and
  is not in the mass invariant.
- **When it starts.** After the run's first step — the box and φ exist only after step 0 — and cumulative from there;
  *Clear world* restarts it (`Ledger::restart`). A dump does not carry it: a resumed run's ledger starts again.
- **What a run writes.** `summary.json`'s `ledger`: `{start, end}`, each `{step, sim_time, mass, energy, momentum, <every
  term>, invariant}`.
- **What is graded where.** `ledger` (the scope) grades the arithmetic of every term and both invariants on short runs
  (mass and momentum to the limits in `docs/agent/testing.md`'s row); the **energy drift is reported there, never graded**
  — G-CONS grades it over whole lives (M0-T106). Momentum in a `leave` world with gravity is graded wider than the ideal,
  because gravity is a source term (M0-V7 observation 1) and a vacuum reset books its momentum nowhere (§2.7 [M0-T19]).
- **Open:** `take_booked` reads a chunk's booking whole (22 MB); the GPU partial reduction that replaces it is flagged to
  M0-T18 and is not here.

## Objects (§1.9.2, M0-T58)
- **Entry.** `Tracker::new(TrackerConfig::from_physics(&physics, sun_mass))`, then `update(&Report) -> &[Object]` once per
  summary. `Report`, `BlockData` and `SinkData` (`observe/mod.rs`) are the f64 widenings of the engine's `BlockSummary`;
  `watch::report` makes one (the frame's sim time is `sim_dt`).
- **What an object is.** 8-connected blocks holding a cell at or above the object density (§2.7); a wisp under the
  fraction of the Sun-like mass §1.9.2 names is not listed. Per object: every quantity §1.9.2 lists — mass (gas and sinks),
  centre, R, τ_dyn, the luminosities by group and in total, the neutrino luminosity, the centre's Σ, T and X, the
  degeneracy, v_r, U, K, P, W, the unbound mass, the envelope mass and the sink mass.
- **Identity.** An object inherits the id of the previous frame's object it shares the most block mass with (above the
  fraction §1.9.2 names); else a fresh id. A merge keeps the heavier id; a split gives the larger part the id and the rest
  fresh ones. **Ids are never reused.** A box that moves between frames is handled (blocks are matched in world cells).
- **Declared where §1.9.2 is silent** (all in `milestones/m0/tasks/M0-T58.md`, cheap to reverse): a sink's block joins its
  object even under the density threshold; R and R_core are floored at half a block; the identity weight of a block is its
  gas plus its sinks, the shared weight the per-block minimum of the two frames', matched greedily (ties: the lower id); the
  centre's Σ, T, X and d come from the centre's block (the heaviest sink's block, else the max-Σ block); v_r includes the
  sinks; R_core and the envelope are `None` while nothing but N burns.
- **L is the nearest star's** (contract §0.5 [M0-D53], the lead at M0-V18): the light of every block outside the listed
  objects — a halo under the object density, a wisp's blocks — is added whole to the listed object whose member block is
  nearest (block centres; a tie to the heavier object, then the lower id). `luminosity` is that total; `luminosity_own` the
  part that left the object's own blocks. Why: §2.4.4 books light at the last gas before vacuum, and on the Sun-like preset
  that gas is a halo outside the object — its own blocks held 4.4 % of the run's light (`milestones/m0/tasks/M0-D53.md`).
- **Evidence.** The `objects` scope grades `Tracker` on synthetic block summaries (`NOT PROVEN (synthetic)` on a star);
  the engine's own summary has run it since M0-T60 (below), and the star's own objects across a life are G-STAGES and
  G-END's to grade (lot 15: M0-T103, M0-T104).

## Stages (§1.9.3, M0-T59)
- **The table is `ROWS`** (`observe/stages.rs`, 19 rows) and the contract's §1.9.3 table is its source: **thresholds are not
  copied here** — read §1.9.3, and `ROWS` is checked against it by the `classify` scope. The first matching row wins, so the
  order is part of the rule (black hole first, then supernova, neutron star, core collapse; Si · O · Ne · C on a tie).
- **The neutron star row (§0.5 [M0-D79])** reads the neutronized, degenerate centre: Y_e,c from the centre's X
  (`registry::electron_fraction`, §1.5's Z/A), d_c, no sink — no |v_r| term until M0-D71's well-balanced P4 (a 2-cell
  remnant's grid flow holds |v_r| at 0.05–0.49 √(GM/R)). After supernova, so a supernova's window still shows; before core
  collapse, so N_Fe's gate re-opening for a frame (L_N < 0) does not restart the remnant's ending hold. It also needs the object
  to have passed core collapse (`History::last_collapse`, §0.5 [M0-D88]): a centre that neutronizes inside one frame shows
  core_collapse first, so the event fires, the supernova can follow and calibrate's one-step frames start.
- **m_ch_sb splits the remnant rows (§0.5 [M0-D81], [M0-D92])**: white_dwarf and planetary_nebula read an object of at most
  m_ch_sb, core_collapse one past it (`Watch` computes it from the run's physics; no electrons' cold pressure, no ceiling, none
  refused). A light remnant whose cold centre overshoots into N_Fe's gate shows white_dwarf or planetary_nebula and bounces back
  — the game's slow-down latch (§1.8.5, any N_Fe rate) still fires there.
- **`classify(&Object, &History, now) -> Stage`** is pure. `History` carries what a frame cannot: whether the object ever
  ignited and the last time it classified as core collapse.
- **The display hold.** `StageTracker::update(&[Object], now)` shows a stage only after §1.9.3's count of consecutive
  identical results (core collapse and supernova are shown at once); a different result restarts the count; until one has
  held, the object's displayed stage is none. `StageTracker::stage(id)` returns the raw result, the shown stage and the
  history. An object gone from a frame is forgotten.
- **Readings declared** (`milestones/m0/tasks/M0-T59.md`): the speed scale √(GM/R) is R/τ_dyn; a ratio against L needs L > 0
  where a comparison would read 0 ≥ 0 true, but `L_nuc < c·L` is literal (a remnant radiating nothing is not a white dwarf);
  "the largest L_g" ties in Si, O, Ne, C order; "no carbon burning" is not (L_C > 0 and ≥ a tenth of L_nuc); "passed core
  collapse" is a frame whose result was core collapse, the last such time kept. **Ignition and main_sequence's hydrogen test
  are a band** (§0.5 [M0-D53]): L_H between half and twice L — a flash inside a cloud that barely shines does not ignite it. **Collapsing and protostar are disjoint as
  written** (their v_r bands do not meet), so `classify-order` grades the table's order, not a behaviour.
- **Not here:** the fuel column of φ (§1.10.4), the "Supergiant" label and the stage's text (§4.6) — a label and the age
  clock's blocks.

## Events (§1.9.4)
| Kind | Fires when | Notes |
|---|---|---|
| `ignition` | the object's `ignited` flag first turns on: L_H within the band of L that §0.5 [M0-D53] names, or the centre has burned a tenth of its hydrogen (X_H,c ≤ 0.9 × the most it showed, L_H > 0; §0.5 [M0-D78]) | once per object; `ignited` never turns off; `detail.by` names the rule, `light` or `fuel` |
| `core_collapse` | the stage row's own condition, L_N < 0 in an object past m_ch_sb (§0.5 [M0-D92]) | the engine's latch flag is **not** read (G-WATCH); the latch's own P6 condition is L_N < 0 alone |
| `supernova` | the supernova stage is entered | |
| `sink_formed` | the first frame an object has a sink | logged only — the latch covers the slow-down |
| `ending` | an object has *displayed* a remnant stage (black hole, neutron star, white dwarf, failed star) for max(10 t.u., 20 τ_dyn), τ_dyn the object's at that frame (§0.5 [M0-D78]) | added by the engine (`watch.rs`, reading `stages::ending_hold`), not by `StageTracker`; counted between frames' sim times, the remnant unchanged throughout |
Each fires once per object, in object order. An event's JSON is `{step, sim_time, kind, object, detail}`: `detail` is
`{mass_sb}`; for `ignition` `{mass_sb, by}`; for `ending` `{stage, held_t, hold_t, mass_sb}`. **`step` counts from the scene**, as the dumps' `s_<step>.bin` do —
a run resumed from a dump counts on from the dump's step — while `summary.json`'s own `steps` is the steps this run ran.

## The observers in a headless run (§2.12.2, §3.3, M0-T60)
- **Frames.** The observers see the summary of each *frame* — a chunk of steps. `headless::FRAME` is 16 steps with
  `--until` and 256 without (untuned: with 64 or more the cold disk's `collapsing` is never shown, because a stage must
  hold for several frames and the identity rule gives a shrinking cloud a new id). The run's first step is a frame of its
  own (the ledger's start), so a stop is at step 1 + 16k.
- **Files.** `DIR/events.jsonl` (one event a line, equal to `summary.json`'s `events`), and in `summary.json`:
  `events`, `objects` and `until`. Each object: `id`, `stage` (the displayed one — `null` until a stage has held),
  `mass_sb` and `stages` — `[{stage, from_step, sim_time}]` for each stage displayed, from the frame that showed it. The objects
  listed are the last frame's. **Left out until their blocks:** `mass_suns` (M0-T69), `predicted_ending` (M0-T72),
  `ending` and `age_years` (M0-T73) — the summary has no placeholder for them.
- **`--until`** (exactly one of `--steps` / `--until`; refused with `--cpu-reference`): `sim_time:<t>` · `event:<kind>` ·
  `stage:<id>` · `ending`. `--max-steps` bounds either; reaching it with the condition unmet exits 2 and the DONE line reads
  `until=unmet`. `sim_time:<t>` stops at the first step past `t` (the last chunks are single steps); the others are read on a
  frame's summary, so they stop at that frame's end — **up to one frame past the step the condition first held**
  (the scope asserts both, and that the run one frame shorter is `unmet`).
- **What is proven on the engine's own summary** (`observe_run`, M0-T60): the cold disk's `collapsing` at step 49 and the
  `sink_cloud` scene with a sink dropped into a seed dump giving `sink_formed` and, 10.06 t.u. later, `ending` (the scene
  never forms a sink by itself in 20,000 steps — M0-T52's `sink` scope owns real formation). A **star's** whole sequence of stages is not yet
  proven here: the Sun-like preset on the Quadro left the main sequence at step 416 and sat in `cloud` to step 25,000, h_shell
  never met (`reports/step_cost.md`, M0-T63) — measured before M0-D50 retuned the constants (`reports/ignition.md`); a star's sequence is G-STAGES's, lot 15.
- **The preset cloud** (§2.10, M0-T61) is a scene object, not an observer: `Object::Preset { mass }`, `cloud_shape(mass)`
  (radius and Σc), `Scene::preset_mass(name, mass, world)`. The cells hold the mass exactly (so the nominal cloud's Σ is a
  hair under 1); W is one FFT solve of the cells' own, so U_th / |W| is the ratio §2.10 names to six digits. Scope
  `preset_cloud`.
- **G-JEANS** (scope `jeans`, M0-T62) grades the cloud's collapse for artificial fragmentation with the tracker on the
  engine's own summary (exactly one object at the end). It is graded at **U_th = 0.05|W|, not §5.1's 0.1|W|** — the lead's
  ruling, contract §0.5 [M0-T62]: the 0.1|W| Sun-like cloud turns round at Σ_max 62.7 and never meets 100 ×. The preset's own
  collapse to ignition and the heat-on case are G-END's and G-STAGES's to grade, not proven by `jeans`.

## Translations (§1.10.1–§1.10.3, G-READ, M0-T69; `crates/sr-physics/src/observe/translate.rs`)
Display functions only: **the physics never reads them** (G-WATCH, I2). A readout, the inspector and the colours show a sandbox number through
them; nothing a step computes depends on one. They live in `sr_physics::observe::translate` beside the observers and read calibration
(`assets/calibration.json`) and `physics.json`, never the grid.
- **Mass → Suns** (`MassScale::new(m_ign, m_ch, m_up, m_bh)`; `to_suns`, `to_sb`). Four sandbox anchors — calibration's `m_ign_sb`, `m_ch_sb`,
  `m_up_sb`, `m_bh_sb` — paired with the contract's real masses (`MASS_SUNS`, §1.10.1's 0.08, 1.44, 8, 25). `to_sb` is the exact inverse: the preset
  masses are `to_sb` of `PRESET_SUNS` (1, 15, 40 Suns, §2.10), which is how `calibrate` writes `preset_mass_sb`. A mass of 0 reads 0.
- **Temperature → kelvins** (`TempScale::new([t_floor, t_eff_sun_sb, t_h, t_he, t_c, t_ne, t_o, t_si])`; `to_kelvin`, `to_sb`). Eight sandbox
  anchors — `t_floor` and the ladder from `physics.json`, the Sun-like preset's T_eff from calibration — paired with `TEMP_KELVIN` (§1.10.2's
  10 K, 5,772 K and the ladder). At and below T_floor the reading is 10 K, and the inverse of any reading at or under 10 K is T_floor
  (the map is flat there, so "strictly increasing" is graded from T_floor up).
- **Surface temperature** (`surface_temperature_sb(l, r, a_r, c_sb)`; `Translations::surface_kelvin(l, r)`): T_eff from L = 2πRσT⁴ with
  σ = a_r c_sb / 4 (§1.10.3), then translated. A luminosity or radius not positive reads 0.
- **`Translations::new(&Calibration, &Physics)`** builds both scales (and σ's two constants) once; it is what the readouts, the inspector and the
  colours hold. A set of anchors that is not finite and positive, or not strictly increasing on either side, is `TranslateError` — which is how
  §1.10.2's "calibration fails otherwise" reads here.
- **The segment rule, declared at M0-T69 (cheap to reverse).** The contract says "piecewise log-linear"; it is read as **ln y straight in ln x**
  on each segment (a power law per segment), the only reading that keeps both sides positive across nine orders of magnitude. The end segments are
  extended (a power law below M_ign and above M_BH; the O–Si segment above T_Si). A midpoint therefore reads the *geometric* mean of the real
  ends, never the linear one — the check `readouts` makes.
- **The anchors' values are the contract's and calibration's** — read `MASS_SUNS`, `TEMP_KELVIN`, §1.10 and `assets/calibration.json`; this page
  copies none. The scope `readouts` (`testing.md`) runs on `Calibration::test_calibration`, and its real-calibration case stays **NOT PROVEN**
  until M0-V9's run writes `assets/calibration.json`.
- **Not wired yet.** summary.json's `mass_suns` and the readout strings come with M0-T72, M0-T73 and the drawing lots; nothing in the engine
  consumes `Translations` today but `calibrate` (the preset masses, the Sun-like T_eff check). The age clock (§1.10.4) is not built.

## What is not here yet
The age → years clock (§1.10.4; M0-T73), the predicted ending and its "close call" (§1.9.5), the selected star (§1.9.6), the labels (§4.6) and the
readout panel (the drawing lots). When they land their sections go on this page; a number that calibration holds is pointed to, never copied.
