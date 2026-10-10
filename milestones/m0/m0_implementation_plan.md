# M0 — Sandbox Reactions: the engine and a star's life (size class: L)
M0 builds Sandbox Reactions' engine and its first demo, in sandbox mode, as a desktop app on
linux-pc — in C++ or Rust, on the engine the lead picks from M0-R1's research — with a tiny web
build kept alive. The player paints star matter (hydrogen, helium and the heavier elements fusion
makes), or drops a ready-made cloud of a chosen mass, into a Powder-Toy-sized world of about
600 × 400 cells (M0-TC fixes the size), drawn as visible cells coloured by temperature that glow
when hot, and watches a whole star's life under nature's laws, squeezed to fit one screen and
minutes at normal speed: the cloud collapses under its own gravity, heats up until fusion lights,
shines, and ends as a white dwarf, a supernova leaving a neutron star, or a black hole, as its
mass decides. The star can be changed at any time — gas added or erased, heated or cooled — and
every stage comes out of the physics, never a script; several clouds may share the world, and one
star's life is what M0 promises. Time runs on fixed speed steps, from about ten times slower than
normal to a Sun-like life in about ten seconds, with pause, single step and an automatic slow-down
at big events that a setting turns off; readouts name the stage and the ending the mass points to,
and translate mass, age and temperature into real astronomy's units; heat, element and density
views and a cell inspector show the physics at work; at the world's edge, matter leaves for good
or bounces back, as the player chooses. The target is 60 frames a second on a mid-range gaming PC
— M0-R3 checks that the top speed can hold it, and a shortfall comes back to the lead at M0-TC.
One reaction mechanism, proven by fusion, waits for the chemistry to come; more elements, saving,
sound, a world size picked by graphics card, story mode, Steam and the rest of the physics list are
routed past M0 (reports/sandbox_interview.md §4, reports/bootstrap.md §3). The Windows laptop, routed
past M0 at bootstrap too, is in play for M0 since M0-TJ3 (Superseded section; R5, R15).
Models: Claude Code — Opus 5.5, max (gate) · Opus 5.5, high (usual) · Sonnet 5.5, high · Sonnet 5.5, medium — strongest first; read 2026-10-10 from the claude-api skill's model table (cached 2026-10-06), the probe and the lead's answer
Changes by asking: every block — the lead at bootstrap, 2026-10-08, "Any task, by asking you": physics formulas will need tuning as they get built; never a bypass that makes a failing check pass
Launch prompt: « Read milestones/m0/m0_implementation_plan.md and execute M0-<id> yourself — you
are the task agent, not an orchestrator. Grep for your own heading first. »

## Resolve with the lead first
None open. Ruled 2026-10-08, verbatim where recorded: the goal paragraph and size class L, R6, R7 at
M0-TP (reports/plan_redteam.md §8) · the language and engine at M0-R1 (Repo facts: Stack) · the
physics forks and the dependency table at M0-TC (m0_contrat.md §0.2, reports/contract_rulings.md).

## Flow
| Task | Type | Goal | Order | Status |
|---|---|---|---|---|
| M0-TI | PLAN + LEAD answers | The sandbox's mechanics and the star's life, from the lead | FIRST | DONE (2026-10-08 09:10, started 08:51) |
| M0-TP | PLAN | Red-team this plan; goal, size, modules and licence put to the lead | AFTER M0-TI | DONE (2026-10-08 09:40, started 09:12) |
| M0-R1 | BUILD | Engine and language research; the lead picks | AFTER M0-TP | DONE (2026-10-08 09:52, started 09:43) |
| M0-D1 | BUILD | Research Verify lines: match sections as headings, not substrings | AFTER M0-R1, BEFORE M0-R2a | DONE (2026-10-08 10:11, started 09:58) |
| M0-R2a | BUILD | Star research: stages, endings, elements, the squeeze | AFTER M0-R1 | DONE (2026-10-08 10:24, started 10:14) |
| M0-R2b | BUILD | Simulation research: models, flat-world gravity, edges, oracles | AFTER M0-R2a | DONE (2026-10-08 10:47) |
| M0-R3 | BUILD | Time-warp research: the squeeze, the speed range, the top speed's frame budget | AFTER M0-R2b | DONE (2026-10-08 11:06, started 10:49) |
| M0-TC | PLAN | Physics forks and the dependency gate with the lead; the M0 contract | AFTER M0-R3 | DONE (2026-10-08 12:05, started 11:14) |
| M0-TH | BUILD | The toolkit and the test harness | AFTER M0-TC | DONE (2026-10-08 12:16, started 12:07) |
| M0-TG | PLAN | The M0 build pipeline, sized and ordered | AFTER M0-TH | DONE (2026-10-08 13:40, started 12:36) |
| M0-TB | PLAN | Size, token and rating pass over the pipeline | AFTER M0-TG | DONE (2026-10-08 14:30, started 13:48) |
| M0-TE-win | BUILD | win-laptop joins — probe the box, install what is missing, prove the toolkit and the toolchain | AFTER M0-TB | DONE (2026-10-08 16:54, started 16:47) |
| M0-D2 | BUILD | rung_record.py's selftest fails 4 checks on Windows | AFTER M0-TE-win | DONE (2026-10-08 16:13, started 16:07) |
| M0-D3 | BUILD | content_gate.py's selftest stops on a Windows file lock | AFTER M0-D2 | DONE (2026-10-08 16:23, started 16:17) |
| M0-D4 | BUILD | plan.py's selftest fails one check on Windows | AFTER M0-D3 | DONE (2026-10-08 16:33) |
| M0-D5 | BUILD | scratch_copy.sh's selftest fails one check on Windows: «a dest inside the source» | AFTER M0-D4 | DONE (2026-10-08 16:46, started 16:40) |
| M0-D6 | BUILD | status_page's selftest flakes on Windows: «POST /answer refuses…» dies with ConnectionAbortedError (WinError 10053) | AFTER M0-D5 | DONE (2026-10-08 17:10, started 16:56) |
| M0-D7 | BUILD | verify.py's selftest crashes on Windows under the parallel loop: exit 3221225477 (0xC0000005), no output | AFTER M0-D6 | DONE (2026-10-08 17:26) |
| M0-TJ3 | PLAN + LEAD answers | Direction ruling — win-laptop in play for M0: what runs where | AFTER M0-TE-win | DONE (2026-10-08 18:05, started 17:50) |
| M0-D8 | BUILD | verify.py's selftest crashes on Windows under the parallel loop: IndexError in a three-field parse | AFTER M0-TJ3 | DONE (2026-10-08 18:16, started 18:10) |
| M0-T1 | BUILD | Workspace, toolchain pin and the build check (native + wasm32) | AFTER M0-TJ3 | DONE (2026-10-08 18:22) |
| M0-D9 | BUILD | red-arm's shared cargo target reuses a planted build: the second `--redarm` of a cargo scope reads a red clean arm | AFTER M0-T1 | DONE (2026-10-09 09:20) |
| M0-T2 | BUILD | GPU device and adapter choice (sr-engine); the GPU test binary | AFTER M0-D9 | DONE (2026-10-09 09:25, started 09:24) |
| M0-T3 | BUILD | The cell state and the step loop; P8's renormalisation as the first pass | AFTER M0-T2 | DONE (2026-10-09 09:34, started 09:30) |
| M0-T4 | BUILD | The headless command: one step, its run summary, the boot scope (G-BOOT) | AFTER M0-T3 | DONE (2026-10-09 09:40) |
| M0-TM1 | PLAN | Rating pass - 5 carried flags | AFTER M0-T4 | DONE (2026-10-09 09:57, started 09:54) |
| M0-T5 | BUILD | The desktop window and its status endpoint; the desktop scope (G-DESK) | AFTER M0-T4 | DONE (2026-10-09 10:17) |
| M0-T6 | BUILD | The double-click launchers start.sh and start.bat (M0-TJ3) | AFTER M0-T5 | DONE (2026-10-09 10:26) |
| M0-T7 | BUILD | The web entry: the page, the wasm build, the no-WebGPU page | AFTER M0-T6 | DONE (2026-10-09 10:58) |
| M0-T8 | BUILD | The capture harness, first form (--capture) | AFTER M0-T7 | DONE (2026-10-09 11:29) |
| M0-T9 | BUILD | Docs: architecture.md, running.md, testing.md rows for lot 1 | AFTER M0-T8 | DONE (2026-10-09 11:32, started 11:30) |
| M0-TV1 | BUILD | UI/UX pass: the first window and the web pages | AFTER M0-T9 | DONE (2026-10-09 11:54) |
| M0-V1 | CHECK | Validation, lot 1: the walking skeleton end to end | AFTER M0-TV1 | DONE (2026-10-09 12:35) |
| M0-TC1 | MOVE | Compaction — Phase 3's close: twelve DONE blocks to stub, more than ten (PLAYBOOK §8, §11) | AFTER M0-V1 | DONE (2026-10-09 12:36, started 12:36) |
| M0-D10 | BUILD | Lot 1's scopes leave out inputs their verdicts depend on: `--changed` skips G-BOOT on a headless.rs edit | AFTER M0-TC1 | DONE (2026-10-09 12:45) |
| M0-D11 | BUILD | The binary refuses its own flags by position: `--out` first, `--help` after a flag read "unknown argument" | AFTER M0-D10 | DONE (2026-10-09 12:48) |
| M0-D14 | BUILD | The adapter scope reads NO-GO: after one NVIDIA device is made, wgpu lists only the Intel iGPU and llvmpipe | AFTER M0-D13 | DONE (2026-10-09 18:16) |
| M0-D15 | BUILD | desktop[launcher] start.bat not recognised on win-laptop | AFTER M0-D14 | DONE (2026-10-09 19:27) |
| M0-D16 | BUILD | tools_selftest[capture_web] native crash on win-laptop | AFTER M0-D15 | DONE (2026-10-10 08:49) |
| M0-D12 | BUILD | Lot 1's docs name an adapter wgpu does not list, restate contract numbers and carry a pre-lot-1 loop line | AFTER M0-D11 | DONE (2026-10-09 18:27) |
| M0-D13 | BUILD | The boot scope deletes the summary.json it grades: a V cannot read what G-BOOT checked | AFTER M0-D12 | DONE (2026-10-10 08:56) |
| M0-T10 | BUILD | Sandbox units and the element and constant registries | AFTER M0-V1 | DONE (2026-10-10 09:04) |
| M0-T11 | BUILD | The reaction registry: ten records, shares conserving mass | AFTER M0-T10 | DONE (2026-10-10 09:13) |
| M0-T12 | BUILD | The equation of state: ideal gas, cold pressure, the u(x) table | AFTER M0-T11 | DONE (2026-10-10 09:24) |
| M0-T13 | BUILD | The string table and its check (G-STR) | AFTER M0-T12 | DONE (2026-10-10 09:32) |
| M0-T14 | BUILD | The licence gate (G-LIC) | AFTER M0-T13 | DONE (2026-10-10 09:49) |
| M0-T15 | BUILD | The web smoke in headless Chrome (G-WEB, first cases) | AFTER M0-T14 | DONE (2026-10-10 10:45) |
| M0-T16 | BUILD | Labels watch, never drive: the static scan (G-WATCH) | AFTER M0-T15 | DONE (2026-10-10 10:50) |
| M0-T17 | BUILD | Docs: physics.md (units, registries, EOS), lot 2's testing rows | AFTER M0-T16 | DONE (2026-10-10 10:53) |
| M0-V2 | CHECK | Validation, lot 2: registries, EOS, strings, licences, web smoke, scan | AFTER M0-T17 | DONE (2026-10-10 14:08) |
| M0-D17 | BUILD | web/smoke.mjs runs Chrome with `--no-sandbox`: playwright-core adds it by default, against §6.2.3 | AFTER M0-V2 | DONE (2026-10-10 14:22) |
| M0-TM2 | PLAN | Rating pass — the record moved BUILD on Sonnet 5.5, high (M0-V2's table) | AFTER M0-D17 | DONE (2026-10-10 14:52, started 14:43) |
| M0-T18 | BUILD | Booking: the per-cell side buffers and accumulators every pass books into | AFTER M0-V2 | DONE (2026-10-10 15:02) |
| M0-T19 | BUILD | P8 floors complete (vacuum reset, temperature floor) and the EOS on the GPU | AFTER M0-T18 | DONE (2026-10-10 15:25, started 15:10) |
| M0-T20 | BUILD | Δt and the non-finite guard (P1, P9) | AFTER M0-T19 | DONE (2026-10-10 15:46) |
| M0-D18 | BUILD | P8 zeroes a NaN species fraction before P9's guard reads it: the run goes on with the cell renormalised | AFTER M0-T20 | DONE (2026-10-10 15:53) |
| M0-T21 | BUILD | The active box (§1.3.3) | AFTER M0-T20 | DONE (2026-10-10 16:12) |
| M0-T22 | BUILD | Frames of steps and the collapse latch, on indirect dispatches | AFTER M0-T21 | TODO |
| M0-T23 | BUILD | GPU timestamps, cost per step and per pass, --timing | AFTER M0-T22 | TODO |
| M0-T24 | BUILD | Time control (sr_physics::time): rungs, frame plan, cap, slow-down, 30-frames switch | AFTER M0-T23 | TODO |
| M0-T25 | BUILD | Scene files (§2.12.1) | AFTER M0-T24 | TODO |
| M0-T26 | BUILD | State dumps (§2.12.3), --dump-every, --load-dump | AFTER M0-T25 | TODO |
| M0-T27 | BUILD | Docs: architecture.md (step, box, latch, files), time_control.md, lot 3 rows | AFTER M0-T26 | TODO |
| M0-V3 | CHECK | Validation, lot 3: the step framework | AFTER M0-T27 | TODO |
| M0-T28 | BUILD | The CPU twin's FFT and convolution (f64) | AFTER M0-V3 | TODO |
| M0-T29 | BUILD | Gravity on the CPU twin (P2); G-GRAV1/G-GRAV2 twin cases | AFTER M0-T28 | TODO |
| M0-T30 | BUILD | The GPU FFT (radix-2, within WebGPU's limits) | AFTER M0-T29 | TODO |
| M0-T31 | BUILD | Gravity on the GPU (P2); one step's cost at 600 x 400 on the Quadro | AFTER M0-T30 | TODO |
| M0-T32 | BUILD | Docs: physics.md gravity; lot 4's testing rows | AFTER M0-T31 | TODO |
| M0-V4 | CHECK | Validation, lot 4: gravity; the first step-cost measurement read | AFTER M0-T32 | TODO |
| M0-T33a | BUILD | Sod's exact solution: the Riemann oracle for γ = 2 (split from M0-T33 by M0-TB) | AFTER M0-V4 | TODO |
| M0-T33b | BUILD | Gas flow on the CPU twin (P4) — G-SOD's twin case | AFTER M0-T33a | TODO |
| M0-T34 | BUILD | Gas flow on the GPU (P4) with bounce walls; G-SOD GPU case | AFTER M0-T33b | TODO |
| M0-T35 | BUILD | The leave edge and escaped mass; G-EDGE | AFTER M0-T34 | TODO |
| M0-T36 | BUILD | Gravity's source in the flow; the cold collapse (G-COLL) | AFTER M0-T35 | TODO |
| M0-T37 | BUILD | The twin's step driver and --cpu-reference | AFTER M0-T36 | TODO |
| M0-T38 | BUILD | GPU against the twin (G-REF): the collapsing disk, the Sod strip | AFTER M0-T37 | TODO |
| M0-T39 | BUILD | The box against the whole world (G-BOX conservation cases) | AFTER M0-T38 | TODO |
| M0-T40 | BUILD | Docs: physics.md gas flow, edges, the CPU twin; lot 5 rows | AFTER M0-T39 | TODO |
| M0-V5 | CHECK | Validation, lot 5: gas flow | AFTER M0-T40 | TODO |
| M0-T41 | BUILD | Heat and light on the CPU twin (P5); G-HEAT1, G-HEAT2, G-FLD twin cases | AFTER M0-V5 | TODO |
| M0-T42 | BUILD | Heat and light on the GPU (P5), radiated energy at vacuum and edges | AFTER M0-T41 | TODO |
| M0-T43 | BUILD | The radiation force in the flow (§2.4.4) | AFTER M0-T42 | TODO |
| M0-T44a | BUILD | The rate law: ω and its temperature table (split from M0-T44 by M0-TB) | AFTER M0-T43 | TODO |
| M0-T44b | BUILD | Burning on the CPU twin (P6) — G-BURN, G-ORDER twin cases | AFTER M0-T44a | TODO |
| M0-T45 | BUILD | Neutrino cooling and the iron gate on the CPU twin (§2.5.3, N_Fe) | AFTER M0-T44b | TODO |
| M0-T46 | BUILD | Burning on the GPU (P6); G-BURN, G-ORDER GPU cases | AFTER M0-T45 | TODO |
| M0-T47 | BUILD | Neutrinos and the iron gate on the GPU, setting the latch | AFTER M0-T46 | TODO |
| M0-T48 | BUILD | The burning field in G-REF (edit job) | AFTER M0-T47 | TODO |
| M0-T49 | BUILD | Docs: physics.md heat, light, reactions, neutrinos; lot 6 rows | AFTER M0-T48 | TODO |
| M0-V6 | CHECK | Validation, lot 6: heat, light and burning | AFTER M0-T49 | TODO |
| M0-T50 | BUILD | Sinks on the CPU twin (P3); G-SINK twin case | AFTER M0-V6 | TODO |
| M0-T51 | BUILD | Sinks on the GPU (P3): formation, accretion, motion, merging | AFTER M0-T50 | TODO |
| M0-T52a | BUILD | The sinks' pull in P2 and the latch on formation | AFTER M0-T51 | TODO |
| M0-T52b | BUILD | Sinks in the state dump (edit job) | AFTER M0-T52a | TODO |
| M0-T53 | BUILD | Neutrino heating on the CPU twin (P7) | AFTER M0-T52b | TODO |
| M0-T54 | BUILD | Neutrino heating on the GPU (P7) | AFTER M0-T53 | TODO |
| M0-T55 | BUILD | Docs: physics.md sinks, neutrino heating, the stand-ins; lot 7 rows | AFTER M0-T54 | TODO |
| M0-V7 | CHECK | Validation, lot 7: black holes and neutrino heating | AFTER M0-T55 | TODO |
| M0-T56 | BUILD | The block summary (§1.9.1) | AFTER M0-V7 | TODO |
| M0-T57 | BUILD | The ledger (§2.9): f64 sums and the short invariants | AFTER M0-T56 | TODO |
| M0-T58 | BUILD | Objects: 8-connected blocks, identity, per-object quantities (§1.9.2) | AFTER M0-T57 | TODO |
| M0-T59 | BUILD | Stages and events (§1.9.3, §1.9.4) | AFTER M0-T58 | TODO |
| M0-T60 | BUILD | Observers in headless runs: --until, objects and events in the summary, events.jsonl | AFTER M0-T59 | TODO |
| M0-T61 | BUILD | The preset cloud at a given mass (§2.10's shape) | AFTER M0-T60 | TODO |
| M0-T62 | BUILD | No artificial fragmentation (G-JEANS) | AFTER M0-T61 | TODO |
| M0-T63 | BUILD | The step-cost probe: a main sequence's steps and the box's cost on the Quadro (O14) | AFTER M0-T62 | TODO |
| M0-T64 | BUILD | Docs: readouts.md (summary, ledger, objects, stages, events); lot 8 rows | AFTER M0-T63 | TODO |
| M0-V8 | CHECK | Validation, lot 8: watching the star; the step-cost probe read | AFTER M0-T64 | TODO |
| M0-T65 | BUILD | The calibration file, its physics hash and the refusal (G-CAL) | AFTER M0-V8 | TODO |
| M0-T66 | BUILD | The calibrate command and items 1-2: the cold ceilings | AFTER M0-T65 | TODO |
| M0-T67 | BUILD | The cold ceiling holds (G-CORE) | AFTER M0-T66 | TODO |
| M0-T68 | BUILD | calibrate items 3-4: ignition, white dwarf against collapse, neutron star against black hole | AFTER M0-T67 | TODO |
| M0-T69 | BUILD | Real-equivalent translations: mass, temperature, surface temperature (G-READ) | AFTER M0-T68 | TODO |
| M0-T70 | BUILD | calibrate items 5-6: the order, preset masses, the Sun's life, TOP, stage durations | AFTER M0-T69 | TODO |
| M0-T71 | BUILD | Docs: running.md (calibrating), physics.md (calibration), lot 9 rows | AFTER M0-T70 | TODO |
| M0-V9 | CHECK | Validation, lot 9: calibration; the lead's calibrate run | AFTER M0-T71 | TODO |
| M0-T72 | BUILD | The predicted ending and the close call (§1.9.5) | AFTER M0-V9 | TODO |
| M0-T73 | BUILD | The age clock: a stage clock that never runs backwards (§1.10.4) | AFTER M0-T72 | TODO |
| M0-T74 | BUILD | The three presets and several clouds (G-MULTI) | AFTER M0-T73 | TODO |
| M0-T75 | BUILD | Number formats (§4.12) | AFTER M0-T74 | TODO |
| M0-T76 | BUILD | Player edits on the GPU (P0): brush, eraser, heat, cool, preset drop, clear | AFTER M0-T75 | TODO |
| M0-T77 | BUILD | The edits file and --edits (§2.12.5) | AFTER M0-T76 | TODO |
| M0-T78 | BUILD | Docs: readouts.md (ending, age), physics.md (edits); lot 10 rows | AFTER M0-T77 | TODO |
| M0-V10 | CHECK | Validation, lot 10: readouts, presets, edits | AFTER M0-T78 | TODO |
| M0-T79 | BUILD | The blackbody table (§1.11) | AFTER M0-V10 | TODO |
| M0-T80 | BUILD | The cell image and the four views (§1.11) | AFTER M0-T79 | TODO |
| M0-T81 | BUILD | The glow: bright pass, Gaussian, upsample (§1.11) | AFTER M0-T80 | TODO |
| M0-T82 | BUILD | Frames from headless runs: --frames-every, --view | AFTER M0-T81 | TODO |
| M0-T83 | BUILD | Docs: architecture.md (rendering); lot 11 rows | AFTER M0-T82 | TODO |
| M0-V11 | CHECK | Validation, lot 11: drawing | AFTER M0-T83 | TODO |
| M0-T84 | BUILD | The world in the window: the frame loop, interpolation, the integer scale | AFTER M0-V11 | TODO |
| M0-T85 | BUILD | The top bar: time controls, the speed readout, the slow-down setting, time keys | AFTER M0-T84 | TODO |
| M0-T86 | BUILD | The banner and the automatic slow-down in the app | AFTER M0-T85 | TODO |
| M0-T87 | BUILD | World actions: the edge choice and Clear world with its dialog | AFTER M0-T86 | TODO |
| M0-T88 | BUILD | Docs: running.md (the window), time_control.md (the app's loop); lot 12 rows | AFTER M0-T87 | TODO |
| M0-TV2 | BUILD | UI/UX pass: the world and its time | AFTER M0-T88 | TODO |
| M0-V12 | CHECK | Validation, lot 12: the world and its time; the lead's launcher run | AFTER M0-TV2 | TODO |
| M0-T89 | BUILD | Tools and elements in the left panel (§4.4) | AFTER M0-V12 | TODO |
| M0-T90 | BUILD | Presets in the window: arm, place, refuse, Esc (§4.5) | AFTER M0-T89 | TODO |
| M0-T91 | BUILD | Views in the window: Glow, Heat, Element, Density, F1-F4 (edit job) | AFTER M0-T90 | TODO |
| M0-T92 | BUILD | The star readout and selection (§4.6, §1.9.6) | AFTER M0-T91 | TODO |
| M0-T93a | BUILD | The engine's cell inspection, inspect(cell) (edit job) | AFTER M0-T92 | TODO |
| M0-T93b | BUILD | The cell inspector panel (§4.6) | AFTER M0-T93a | TODO |
| M0-T94 | BUILD | The web build's star: URL parameters and G-WEB's preset case | AFTER M0-T93b | TODO |
| M0-T95 | BUILD | The UI's cost: --measure-ui (the lead's windowed run sets render_reserve_ms) | AFTER M0-T94 | TODO |
| M0-T96 | BUILD | Docs: running.md (tools, presets, captures, measure-ui); lot 13 rows | AFTER M0-T95 | TODO |
| M0-TV3 | BUILD | UI/UX pass: tools, stars and cells | AFTER M0-T96 | TODO |
| M0-V13 | CHECK | Validation, lot 13: tools, stars and cells; the lead's measure-ui run | AFTER M0-TV3 | TODO |
| M0-T97 | BUILD | The life probe: three presets' lives measured (reports/lives.md) | AFTER M0-V13 | TODO |
| M0-TJ1 | PLAN + LEAD answers | Stand-ins S1/S2: enabled only on a measured failure (deferred) | AFTER M0-T97 | DEFERRED (wake: reports/lives.md measures §1.6.4's failure — S1's unbound envelope under 30 %, or S2's under 50 % at f_dep = 0.1) |
| M0-T98a | BUILD | K1's gap tuned on preset-free clouds (split from M0-T98 by M0-TB) | AFTER M0-TJ1 | TODO |
| M0-T99a | BUILD | The mass ladder tuned on preset-free clouds (split from M0-T99 by M0-TB) | AFTER M0-T98a | TODO |
| M0-T102 | BUILD | Docs: physics.md (the tuning record) | AFTER M0-T99a | TODO |
| M0-V14 | CHECK | Validation, lot 14: a star's whole life measured and tuned; the lead's recalibration | AFTER M0-T102 | TODO |
| M0-T103 | BUILD | The visible life (G-STAGES) | AFTER M0-V14 | TODO |
| M0-T104 | BUILD | Each ending as predicted (G-END) | AFTER M0-T103 | TODO |
| M0-T105 | BUILD | Touch any time (G-TOUCH) | AFTER M0-T104 | TODO |
| M0-T106 | BUILD | Conservation over whole lives (G-CONS) | AFTER M0-T105 | TODO |
| M0-T107 | BUILD | The age clock over a touched life (G-AGE) | AFTER M0-T106 | TODO |
| M0-T98b | BUILD | The clocks' order (G-SQUEEZE) | AFTER M0-T107 | TODO |
| M0-T99b | BUILD | Every life fits the world (G-FIT) | AFTER M0-T98b | TODO |
| M0-T100 | BUILD | Virial equilibrium (G-VIR) | AFTER M0-T99b | TODO |
| M0-T101 | BUILD | The thermostat (G-THERMO) | AFTER M0-T100 | TODO |
| M0-T108 | BUILD | Docs: readouts.md (the stages over a life); lot 15 rows | AFTER M0-T101 | TODO |
| M0-V15 | CHECK | Validation, lot 15: the whole-life checks; the lead's long runs | AFTER M0-T108 | TODO |
| M0-T109 | BUILD | The bench (§3.3) | AFTER M0-V15 | TODO |
| M0-T110 | BUILD | Warps never touch the physics (G-WARP) | AFTER M0-T109 | TODO |
| M0-T111 | BUILD | The collapse caught within one step: G-LATCH's contract scenario | AFTER M0-T110 | TODO |
| M0-T112 | BUILD | The frame rate on the Quadro (G-FPS) | AFTER M0-T111 | TODO |
| M0-T113 | BUILD | A Sun-like life in about 10 s (G-TOP) | AFTER M0-T112 | TODO |
| M0-TJ2 | PLAN + LEAD answers | The top speed if it cannot hold at 30 frames (deferred) | AFTER M0-T113 | DEFERRED (wake: G-TOP red at 30 frames on the Quadro (M0-T113), or reports/step_cost.md projecting over 33.3 ms a frame — M0-V8 moves this block up then) |
| M0-T114 | BUILD | The gravity cadence (§1.4.4, G-CAD), only if the budget needs it (deferred) | AFTER M0-TJ2 | DEFERRED (wake: G-FPS or G-TOP red with P2's share of the step large enough, by per_pass_ms, that k = 4 brings the frame under budget — or the lead's word) |
| M0-T115 | BUILD | Docs: time_control.md (bench, cap, 30-frames switch), running.md (long runs); lot 16 rows | AFTER M0-T114 | TODO |
| M0-V16 | CHECK | Validation, lot 16: speed and frame rate; the lead's Quadro runs | AFTER M0-T115 | TODO |
| M0-V17 | CHECK | Final validation: the whole milestone, contract against code | AFTER M0-V16 | TODO |
| M0-TD | BUILD | Show-off demo: a star's life staged and polished (OPT-B) | AFTER M0-V17 | TODO |
| M0-TW | LEAD walk + CHECK | The user gate: the lead walks a star's life | AFTER the push that follows M0-V17 and M0-TD | TODO |
| M0-T116 | BUILD | Documentation: the tracker and the docs aligned with what shipped | AFTER M0-TW | TODO |
| M0-TZ | PLAN | The next plan, M1 | LAST | TODO |

## Rules
PLAYBOOK.md v12.2 + the instruction file bind every agent (cited once, never per block). Full text
of every rule below: m0_rules.md. Header budget: 600 / 60. Rating refresh: 5 flags. Full-loop
budget: 600 s on each box in play (the 10-minute line of Rule 4) — linux-pc measured 449.7 s wall at M0-V1
(2026-10-09): 10 scopes, 537 cases in 6.1 s, then 443 s red-arming the six scopes new since HEAD (desktop's
287 s); win-laptop 134 s wall at M0-V2 (2026-10-10): 15 scopes, 586 passed, none new since HEAD to red-arm, ≈ 113 s
normalised by the measured load (15.6 % of the box external; per box since M0-TJ3, PLAYBOOK §8) — every lot's V
restates its box's; over it, a TR fires. OPT modules: OPT-B (R6, confirmed at
M0-TP), OPT-C (R2–R4). Grants (OPT-C lanes: scope · exclusions · ceiling): R2 BUILD lane, R3
installs, R4 DEBUG lane — each $0. Derogations, dated: none. Sizing bar: §2.1 — ceilings and named
exceptions by id — none declared; E blocks: 7 of 127 open BUILD blocks (M0-T39, M0-T48, M0-T52b,
M0-T91, M0-T93a, M0-T94, M0-T111 — marked by M0-TG, counted by M0-TB, 2026-10-08; M0-T6 unmarked by M0-TJ3:
start.bat and the box route take it past an edit job). Gates to builds: 7:5 at
M0-TP (7:4 at bootstrap, then M0-R2 split); 26:128 after M0-TG's pipeline (2026-10-08 — 17 lot and final
V's, two deferred TJs); 26:132 after M0-TB's four splits; 27:133 with M0-TE-win and M0-TJ3, the
win-laptop pair the lead asked for (2026-10-08). Considered and REJECTED (false
economy): m0_rules.md, binding (M0-TB). What a capture cannot show here: the simulation's motion and
timing (a star evolving, flows, a time warp's smoothness), frame rate and input feel.

| # | Rule (one line — the annex carries the text) |
|---|---|
| R1 | A Python `.venv/` at the repo root is allowed when a task needs one (the lead, 2026-10-08) |
| R2 | Grant, BUILD lane: compile, run the deterministic suites and the smokes, fetch approved libraries — no ask; $0 |
| R3 | Grant, installs: the approved table's tools, user-level, on linux-pc and win-laptop; on win-laptop the agent does the setup itself; $0 |
| R4 | Grant, DEBUG lane: run your own build headless or off-screen to check it — on win-laptop the window placed off-screen, never focused, out of the taskbar (M0-TJ3) — never acceptance; a visible window still asks |
| R5 | Boxes: linux-pc and win-laptop both in play — any block, lot V's included, runs on the box the lead is on (M0-TJ3); probe the box first; every verdict names its box |
| R6 | Modules: OPT-B on; OPT-A, D, E, F, G off — assumed at bootstrap, confirmed by the lead at M0-TP (2026-10-08) |
| R7 | No copyleft code (GPL family) in the game: studied for ideas, never copied or translated — the lead's ruling at M0-TP (2026-10-08) |
| R8 | Physics oracles: a physics behaviour passes only against a cited reference result within a stated tolerance |
| R9 | The routine (Rule 4): own scope until green → `verify.py --redarm <scope>` → plan edits → `verify.py --changed` once, last; `--all` only at phase-closing V and TR; every scope ships a plant; docs/agent/testing.md says how to add one |
| R10 | The claim run: every BUILD block's last step is `python3 tools/pb/verify.py --changed --base <the last commit gate's SHA> --task <ID>` · Pass: GO — a named commit, never `HEAD` |
| R11 | Scopes: GPU scopes are modules of one test binary, crates/sr-engine/tests/gpu/ (`--test gpu <module>::`), CPU scopes `--lib <module>::` (sr-app's `--bin sandbox-reactions`); scripts source tests/cargo.sh; each lot's docs block writes its rows; (new) = absent on 2026-10-08 |
| R12 | ⏱ scopes: `paths` name only their own test and scene files; physics on the RTX 5090 (win-laptop: the RTX 4080 Laptop, R15), timing (fps, top-speed) on the Quadro only; over 10 minutes → the lead's, in the lot V's window |
| R13 | Physics: every GPU number names its adapter; an UNVERIFIED (§5.5) red is a D or a question, never a looser number (§0.4); each pass's CPU f64 twin before its GPU shader; box and latch first |
| R14 | Every V is done when its verdict is recorded, its D's created and the register updated (PLAYBOOK §14.3); a V that adds to that says so on its own `Done when:` line |
| R15 | Box pattern (M0-TJ3): on win-laptop a block reads the RTX 5090 as the RTX 4080 Laptop, Xvfb/lavapipe/`game-xvfb` as the off-screen window (`game-offscreen`), start.sh as start.bat, /usr/bin/google-chrome as contract §6.2.3's Windows Chrome, `python3 tools/pb/…` as `py -3.12 tools/pb/…`; the Quadro's timing stays `box: laserax-ai`; a check only one box runs is owed elsewhere, flagged V to V, run at the next V on its box or by M0-V17 |
| R16 | A block's last message ends with one line for the lead — `git add -A`, commit `M0-<ID> <status>`, push — in the probed box's shell: Windows → PowerShell (`git add -A; if ($?) { git commit -m "…" }; if ($?) { git push }`), linux-pc → bash (`&&`); agents never run it (the lead, 2026-10-10) |
| R17 | Scope names are letters, digits and `_` (verify.py): a scope the contract or a block writes with `-` reads `_` — `watch-only` is `watch_only`, `time-control` is `time_control`; case and plant ids keep their spelling (contract §0.5 [M0-V2]; the lead, 2026-10-10) |

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
Inherited debt (§2.4): none.

## Repo facts (so you don't explore)
- Tracker: `milestones.md`. Paths in this plan: `reports/`, `tasks/`, `logs/`, `images/` sit under
  `milestones/m0/`; every command runs from the repo root.
- Product: Sandbox Reactions — a falling-sand physics sandbox game; sandbox mode first, story mode
  later (reports/bootstrap.md §3). Product language: English (UI strings, element names,
  messages). Plans, reports and handoffs: English.
- Repo: green field (2026-10-08) — PLAYBOOK.md v12.2 and the bootstrap's files, no code; the
  bootstrap pushed as 9a07813 and 7acd1fc; remote `origin` =
  git@github.com:Feynstein/sandbox_reactions.git, branch `main`.
- Stack: ruled — Rust 1.99 with our own thin engine on wgpu 30 + eframe 0.36 (egui; winit 0.30)
  (the lead at M0-R1, 2026-10-08); the source layout (crates sr-physics, sr-engine, sr-app; assets/,
  web/, scenes/, tests/plants/) is m0_contrat.md §1.1, the approved dependency table §1.2.
- Targets: desktop on linux-pc first — built and run on win-laptop too since M0-TJ3 (R5, R15); a tiny
  web build kept alive from the first build lot; Steam later. The lead: "Ideally I would for it to run
  on a website, or to be able to sell it on steam."
- Time zone for run and output folder names: America/Toronto (probed); instants inside artifacts
  are UTC-Z (§1).
- Boxes (R5): `linux-pc` — this PC, hostname laserax-ai, Ubuntu 24.04.5 LTS, kernel 7.0.0-38,
  bash, 32 CPU threads, 188 GiB RAM; GPUs NVIDIA GeForce RTX 5090 32 GB and Quadro RTX 4000 8 GB
  (driver 595.99.02) and an Intel UHD 770; OpenGL 4.6 (renderer: the Quadro), Vulkan ICDs present,
  Vulkan loader dev 1.3.275; Wayland session with XWayland on DISPLAY=:1 — in play.
  `win-laptop` — the lead's Windows laptop, hostname Laser2025-20, probed by M0-TE-win (2026-10-08): Windows 11
  Pro build 26300, Intel Core Ultra 9 185H (16 cores / 22 threads), 31.5 GiB RAM, 31 GiB free of 953 GiB; GPUs Intel
  Arc Graphics (driver 32.0.101.6790) and NVIDIA GeForce RTX 4080 Laptop (32.0.15.8129); shells PowerShell 5.1 and
  Git Bash 5.3.15 (the Bash tool — every toolkit call goes through it, as `py -3.12 tools/pb/…`, M0-D7) — in play
  for every M0 block, the lot V's included, since M0-TJ3 (R5, R15; reports/win_laptop.md § Box, § Ruling).
- Toolchains on linux-pc (probed 2026-10-08, re-probed by M0-TP): present — g++/gcc 13.3.0, GNU
  make 4.3, ninja 1.11.1, pkg-config 1.8.1, Python 3.12.3 (uv 0.12.3), Node 20.20.2 + npm 10.8.2,
  git 2.43.0, jq, Xvfb + xvfb-run; Rust 1.99.0 stable (rustup 1.29.1, host target x86_64 only —
  no wasm32 until M0-TH added `wasm32-unknown-unknown` and trunk 0.21.14 — a prebuilt release,
  sha256-checked, in `~/.cargo/bin/trunk`, R3, 2026-10-08, logs/M0-TH.log) in `~/.cargo/bin`,
  installed 2026-10-06, off the PATH (Hazards); Godot 4.7 stable on
  the PATH (`~/.local/bin/godot` → `~/Applications/godot/`) with its 4.7 export templates (web,
  Windows, Linux); dev libraries x11 1.8.7, xcursor, xrandr, xi, wayland-client 1.22.0, xkbcommon
  1.6.0, vulkan 1.3.275, libudev 255, GL, EGL. Missing — clang, cmake, SDL2/SDL3 dev, ALSA dev,
  glfw3 dev, vulkaninfo, emcc, wasm-pack. Installs follow the approved table (m0_contrat.md §1.2,
  cleared by the lead at M0-TC, 2026-10-08) and R3. Also present (probed by M0-TC): clippy and
  rustfmt in the Rust toolchain; Mesa's software Vulkan (lavapipe, `lvp_icd.json`) beside the
  NVIDIA and Intel drivers; Chrome at /usr/bin/google-chrome.
- Python: a `.venv/` at the repo root when a task needs packages (R1); the playbook's tools need
  none.
- Agent (the probe, §B.3, 2026-10-08): Claude Code 2.1.292 (`AI_AGENT=claude-code_2-1-292_agent`,
  `CLAUDECODE=1`). The switch plugin `pb-switch@pb` is installed on linux-pc
  (`~/.claude/plugins/installed_plugins.json` lists it); `rung_record.py now` reads whether it is
  current against this playbook's switch (12.2.1) once M0-TH extracts it. Every rung of the ladder
  is in the switch's `RUNGS` table.
- Toolchains on win-laptop (M0-TE-win, 2026-10-08, logs/M0-TE-win.log): present — Python 3.13.14 as `python3` (the
  Store package's alias; `python` 3.12.10), Node 26.8.2 + npm 11.19.1 (the table says Node 20; kept, ≥ 20), git
  2.55.0, Git Bash 5.3.15, Chrome 154.0.8037.98 at `C:\Program Files\Google\Chrome\Application\chrome.exe`,
  Visual Studio 2026 Community with MSVC 14.44/14.51 and Windows SDKs; installed by M0-TE-win (R3) — rustc/cargo
  1.99.0 (rustup update; was 1.97.1) with clippy, rustfmt and `wasm32-unknown-unknown`, trunk 0.21.14 (release
  zip, sha256-checked) in `~/.cargo/bin`, and rsync 3.5.1 (outside the table — the lead's yes, GPL-3.0, a dev
  tool) in `~/bin`; user variable `MSYS=winsymlinks:nativestrict`. Missing — playwright-core (arrives with
  web/package.json, M0-T15), the Vulkan SDK. `~/.cargo/bin` is on the PATH here (it is not on linux-pc).
- Agent on win-laptop (M0-TE-win, 2026-10-08): Claude Code 2.1.294 (`AI_AGENT=claude-code_2-1-294_agent`); the
  switch plugin `pb-switch@pb` 12.2.1 is installed (`rung_record.py now` → `plugin=installed`).
- Folders: `milestones/m0/` holds this plan, m0_rules.md, the contract (m0_contrat.md) and
  logs/ reports/ tasks/ images/ (`.gitkeep` holds the empty ones); `tools/pb/` (the toolkit),
  `docs/agent/testing.md`, `tests/toolchain/` (the one-file program) and `tests/plants/` (planted-bug
  patches) arrived with M0-TH, 2026-10-08; no game source tree yet.
- Run commands, harness scopes, launchers: the harness is `tools/pb/verify.json` (three scopes,
  M0-TH; its loop time is Rules' budget line); no app yet — `tools/pb/launch.json`
  (contract §3.6) arrives with M0-T4 (`headless-boot`), M0-T5 (`game`, `game-xvfb`, `game-offscreen`) and
  M0-T15 (`web`), the double-click launchers start.sh and start.bat (§8, M0-TJ3) with M0-T6; cargo builds into `build/target/` (contract §6.5 as
  amended by M0-TG, the lead's yes, 2026-10-08). Run commands for the toolchain: `~/.cargo/bin/cargo`,
  `~/.cargo/bin/trunk` (off the PATH — Hazards); builds go under the git-ignored `build/`.
- Git: agents never write git (§10); each phase ends with the lead's commit gate.
### Using the tools — the calls a task needs (never the tool's code or its help)
- Extracted by M0-TH (2026-10-08, §B.4's line, 10 files, none refused) into `tools/pb/`; every
  call runs from the repo root, a wrong call prints the right one. Never read the tools' code.
  `plan.py` (the plan; the Flow row stays in sync): `show <ID>` (your start read) · `status <ID>
  "<STATUS>"` · `close <ID> --status "<STATUS>" --text "<handoff>" [--set "<Key>=<value>"]...` ·
  `handoff <ID> --text "<handoff>"` · `flag <ID> "<text>" --from <ID>` (a flag aimed at a later
  task's `Carried flags:`) · `append <block.md> --after <ID>` (a D you file) · `move <ID> --after
  <ID2>` · `stub <ID>... --by <ID>` · `register --set "<Key>=<value>"...` · `lint --plan
  milestones/m0/m0_implementation_plan.md`.
  `verify.py` (the harness — R9, docs/agent/testing.md; always `--task <ID>`): `<scope>` ·
  `<scope> --case <id>` · `--redarm <scope>` · `--changed --base <named commit>` · `--all` (phase
  V and TR only) · `list` · `report`. Scopes today: `plan_lint` · `tools_selftest` · `toolchain`.
  Others: `launch.py` (start, stop, status, smoke; no `launch.json` yet) · `scratch_copy.sh <src>
  <dest>` · `rung_record.py now` (alone in its Bash call, after `sleep 2`) · `review_page.py` and
  `capture_web.mjs` (the TV and TW blocks) · `content_gate.py` (OPT-D, off — unused).
### Hazards (dated, root-caused — carried forward by TZ until retired with a reason)
- 2026-10-08 · Claude Code's auto mode refused the bootstrap agent (a) extracting the playbook's
  tools to a scratch folder and running them ("Code from External") and (b) writing AGENTS.md,
  CLAUDE.md and GEMINI.md from §C ("Instruction Poisoning"). Root cause: its classifier reads
  code and auto-loaded instruction files taken from PLAYBOOK.md as external content. Effect:
  M0-TH's extraction (§B.4), and any later write of the instruction files or the switch's hook,
  may be refused too — a refusal goes to the lead as a question (approve it, or run the line
  themselves), never around it.
- 2026-10-08 · linux-pc drives its display from the Quadro RTX 4000 (glxinfo: "OpenGL renderer
  string: Quadro RTX 4000/PCIe/SSE2"); the RTX 5090 and an Intel UHD 770 are present too. Root
  cause: three GPUs, the display on the Quadro. Effect: a graphics API's default adapter may not be
  the 5090 — adapter choice is explicit (contract §6), and every GPU test and performance number
  names the adapter it ran on.
- 2026-10-08 · The desktop session is Wayland (`XDG_SESSION_TYPE=wayland`, XWayland on `:1`). Effect:
  windowing behaves differently under Wayland and X11; headless checks use offscreen rendering or
  a private Xvfb display (R4), never the lead's desktop.
- 2026-10-08 · The root filesystem is 82% full (165 GB free of 937 GB). Effect: build caches (a
  compiler's target folder, a web toolchain, shader caches) can take tens of GB — keep them in
  known, git-ignored folders; scratch copies go through `tools/pb/scratch_copy.sh` (§11).
- 2026-10-08 · git keeps no empty folder: `.gitkeep` files, committed in 9a07813, hold
  `milestones/m0/{logs,tasks,images}/`, so a fresh clone (win-laptop) has them for the plan lint.
- 2026-10-08 · Rust is installed user-level but off the PATH: `~/.cargo/bin` holds rustc and cargo
  1.99.0 (rustup, 2026-10-06) and no shell rc file sources `~/.cargo/env` (probed by M0-TP). Root
  cause: the rustup install's PATH line is absent. Effect: a scope, a launcher or an agent calling
  `cargo` bare fails "not found" — call `~/.cargo/bin/cargo` or source `~/.cargo/env` first; a
  PATH edit in the lead's shell files is the lead's.
- 2026-10-08 · win-laptop, `bash` is two programs (M0-TE-win): from PowerShell the toolkit's `bash` is
  `C:\WINDOWS\system32\bash.exe`, WSL's relay (no `/bin/bash` in its docker-desktop distro) — `verify.py
  toolchain` read NO-GO 0/3 there; in the Bash tool it is Git's `usr\bin\bash.EXE`. Every toolkit call on
  win-laptop goes through the Bash tool; no PATH edit (it would shadow Windows' find and sort).
- 2026-10-08 · win-laptop, symlinks and rsync (M0-TE-win): Git Bash copies instead of linking unless
  `MSYS=winsymlinks:nativestrict` (Developer Mode is on; a user variable now); Windows has no rsync (installed to
  `~/bin`, the lead's yes). Effect before: scratch_copy.sh's selftest 3/12; now 11/12 (M0-D5 holds the last).
- 2026-10-08 · win-laptop, the disk is 97 % full (31 GiB free of 953 GiB): `build/target`, the red-arm cache,
  trunk's cache and node_modules compete for it — read the free space before a lot's first big build.
- 2026-10-08 · win-laptop, `tools_selftest` read 6 of 10 GO (a Windows file lock, WinError 32, one root cause);
  M0-D2..D5 fixed the four reds and `verify.py --all` re-ran GO there (500 passed, 3/3 scopes, 98 s wall; logs
  M0-TE-win-r2.*). The loop is 75–215 s of case time here. A task's logs committed in a commit gate are never
  appended to — a re-run takes a fresh `--task` id. `tools_selftest[verify]` exit 3221225477 (0xC0000005), no
  output, no WER event, seen twice under the Store `python3` — routed by M0-D7 (the next Hazard).
- 2026-10-08 · win-laptop, `python3` is the Store package's alias (3.13.14), not the python.org 3.12.10 behind
  `python`; `py -3` picks it too (the launcher's default is 3.13). Root cause of M0-D7's suspect: `{python}` is
  `sys.executable`, here the App Execution Alias `…\WindowsApps\PythonSoftwareFoundation.Python.3.13_…\python.exe`,
  for every case and every child a selftest fans out. Effect: on win-laptop every toolkit call is `py -3.12
  tools/pb/<tool> …` (the lead, M0-D7, 2026-10-08); cases run with `PYTHONFAULTHANDLER=1` (verify.py `case_env`),
  so a native crash leaves its stack in the log — one more 0xC0000005 under 3.12 is a D with that stack.

## Pipeline state (a register of one-line pointers — never a handoff or a history)
- Next task: M0-T21
- Counters: T=116 · D=18 · V=17 · Q=0 · TI=0 · TJ=3 · TV=3 · TC=1 · TR=0 · TM=2 · TD=1 · TE=1
- Open D/BLOCKED register: none (M0-D18 DONE 2026-10-10)
- Outstanding commit gates: Phase 4 (M0-T10 … M0-V2, then M0-D17 and M0-TM2 that M0-V2 filed) — the gate under M0-TM2, its text « closes after M0-V2 »; Phase 2b and Phase 3 are in dde81d3, pushed (per-block commits, R16)
- Carryover: none
- Awaiting lead: none (M0-TC's five rulings made 2026-10-08 — reports/contract_rulings.md)
- Model ratings: 2026-10-10 by M0-TM2

# Tasks

# Phase 0 — the lead's mechanics, the plan red-teamed

## M0-TI · Lead interview — the sandbox's mechanics and the star's life · **PLAN + LEAD answers** · Opus 5.5, max · switch · (FIRST)
- Status: DONE (2026-10-08 09:10, started 08:51) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/sandbox_interview.md — 18 answers verbatim (5 rounds), inference apart (§3), the
- Full block: plan_archive.md § M0-TI

## M0-TP · Red-team the plan · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TI)
- Status: DONE (2026-10-08 09:40, started 09:12) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/plan_redteam.md — Repo facts re-probed (§1), §2.1's flags (§2), TI's F1–F7 and
- Full block: plan_archive.md § M0-TP

> **Commit gate (lead):** Phase 0 closes after M0-TP — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 0: lead interview and plan red-team"`
> `git push`

# Phase 1 — research: the engine, the star's physics, the time warp

## M0-R1 · Engine and language research — C++ or Rust, our own engine or an existing one · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TP)
- Status: DONE (2026-10-08 09:52, started 09:43) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/engine_stack.md exists: 6 candidates scored on 16 criteria, 36 evidence lines (39 dated URLs), UNVERIFIED register. **Ruled by the lead: "Rust + wgpu, own engine (Recommended)"** — Rust with wgpu + winit + egui; runner-up Rust + Bevy, answered.
- Full block: plan_archive.md § M0-R1

## M0-D1 · The research Verify lines pass a report with no real sections · **BUILD** · Opus 5.5, max · switch · (AFTER M0-R1, BEFORE M0-R2a)
- Status: DONE (2026-10-08 10:11, started 09:58) · archived by M0-TB, 2026-10-08
- Handoff (one line): The four research Verify 1 lines (M0-R1, R2a, R2b, R3) now match each section as a heading line, `re.search(r'(?m)^'+re.escape(s)+r'\b',t)`, and R1 reads the Ruling body from under its heading line; each Pass line says so. Applied by asking — the lead, 2026-10-08: "Apply to all four (Recommended)".
- Full block: plan_archive.md § M0-D1

## M0-R2a · Star research — the stages, the endings and the squeeze · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R1)
- Status: DONE (2026-10-08 10:24, started 10:14) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/star_physics.md exists — 25 dated sources (Pols' Utrecht notes, Heger 2003, Sukhbold 2016, NASA, OpenStax, Wikipedia); `## Stages` S1–S8″ with each stage's driver, real time scale and end state, the ending-threshold table, I11 checked; `## Elements` H He C O Ne Mg Si S Fe (+ optional Ni-56); `## Squeeze` K1–K9 as orders and ratios; the B31 translation proposed (mass: one factor; temperature: one factor or log-anchors; age: a stage clock).
- Full block: plan_archive.md § M0-R2a

## M0-R2b · Simulation research — the models that run the star, and their oracles · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R2a)
- Status: DONE (2026-10-08 10:47, started 10:32) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/sim_models.md exists — 31 dated sources (Pols, Chavanis 2007, Maclaurin-disk and thin-disk gravity papers, RKL2, FLD, Truelove, Federrath sinks, Wikipedia laws); `## Models` M1–M8: an Eulerian grid on the GPU; B32 recommended as the 3D 1/r² law inside a flat sheet (2D log gravity breaks K5 and K9) by zero-padded FFT convolution; MUSCL-Hancock gas; flux-limited diffusion under RKL2; one reaction registry (fusion now, chemistry later, B19); a Σ²→Σ^(3/2) cold pressure giving a maximum mass in the sheet; sink-particle black holes; both edge modes; a worked example; `## Oracles` O1–O16 with tolerances.
- Full block: plan_archive.md § M0-R2b

## M0-R3 · Time-warp research — the squeeze and the speed range, from slow motion to a star's life in about ten seconds · **BUILD** · Opus 5.5, high · switch · (AFTER M0-R2b)
- Status: DONE (2026-10-08 11:06, started 10:49) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/time_warp.md exists — 16 dated sources (SSE, BSE, MIST EEPs, MESA, Gear–Kevrekidis, reduced speed of light and of sound, GADGET-2, Dursi–Zingale, Fix Your Timestep, WebGPU limits) and measurements M1–M4: a scratch wgpu 30 bench (built offline from the cargo cache, headless; source in the session scratchpad, not the repo) put a whole-world 600 × 400 step at 2.13 ms on the Quadro (R2b estimated 0.88), the Sun-like star's box at 0.11 ms, 2.65 µs per dispatch. Steps per life re-derived: ~22,500 / ~63,000 / ~340,000 (scenarios L/C/H by the unmeasured K1 gap). Verdict: the top speed is NOT shown held — reachable only by computing the star's box (L held, C at the edge, H ~5× over); fork F1 to M0-TC with a measured decision point.
- Full block: plan_archive.md § M0-R3

> **Commit gate (lead):** Phase 1 closes after M0-R3 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 1: engine pick and physics research"`
> `git push`

# Phase 2 — the contract, the harness, the pipeline

## M0-TC · The M0 contract — physics forks and the dependency gate with the lead first · **PLAN** · Opus 5.5, max · switch · (AFTER M0-R3)
- Status: DONE (2026-10-08 12:05, started 11:14) · archived by M0-TB, 2026-10-08
- Handoff (one line): The lead ruled 5 forks by the question tool, all on the recommended option, verbatim in reports/contract_rulings.md: Q1 gravity = the 3D pull in a thin sheet · Q2 the top rung may drop to 30 frames a second (still short → back to the lead) · Q3 stand-in laws as a backup only (S1 dust opacity, S2 thermal bomb, each enabled by a PLAN amendment on a measured failure) · Q4 a "needs WebGPU" page · Q5 the dependency table approved, versions pinned, every licence sourced.
- Full block: plan_archive.md § M0-TC

## M0-TH · Test harness and toolkit · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TC, BEFORE M0-TG)
- Status: DONE (2026-10-08 12:16, started 12:07) · archived by M0-TB, 2026-10-08
- Handoff (one line): Toolkit extracted (10 files, 0 refused) and every selftest GO — capture_web's live tier NOT RUN (no Playwright yet). tools/pb/verify.json: plan_lint (49 cases) · tools_selftest (10) · toolchain (3: rustc 1.99 desktop, wasm32, trunk 0.21.14 page), each red-armed (plan_lint 1/1, tools_selftest 1/1, toolchain 3/3 plants red, clean copies GO); anchor seeded (21 covered · 30 skipped · 0 unclassified); docs/agent/testing.md; R9 + m0_rules.md; .gitignore, .gitattributes. Verify 1 [ALREADY RUN — PASS (3/3 scopes GO, 62 cases, loop 3.3 s warm) on linux-pc]; budget line 600 s in Rules.
- Full block: plan_archive.md § M0-TH

## M0-TG · Task generation — the M0 build pipeline · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TH)
- Status: DONE (2026-10-08 13:40, started 12:36) · archived by M0-TB, 2026-10-08
- Handoff (one line): Pipeline written under # Build: 141 blocks filed through plan.py append, forward order — 16 lots as Phases 3–18 (118 T incl. M0-T52a/b and M0-T93a/b, 16 lot V's, TV1–TV3 for the three phases that change screens), Phase 19 (final M0-V17, OPT-B M0-TD, the push gate), M0-T116 the docs task between M0-TW and M0-TZ; DEFERRED: M0-TJ1 (stand-ins), M0-TJ2 (top speed), M0-T114 (cadence). Verify 1 [ALREADY RUN — PASS (plan.py lint GO, 471 checks) on linux-pc]; claim run --changed --base a5065fb [ALREADY RUN — PASS (3/3 scopes, 484 cases, 3.3 s) on linux-pc], its FLAGs naming only M0-TH's uncommitted tools.
- Full block: plan_archive.md § M0-TG

## M0-TB · Task-size, token and rating pass · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TG)
- Status: DONE (2026-10-08 14:30, started 13:48) · archived by M0-TB, 2026-10-08
- Handoff (one line): reports/size_pass.md: every open BUILD block through §2.1 (i)–(vi), 126 rows; 495 Deliver paths grep-verified before the edits, 510 after; 3 wrong "(new)" labels fixed. The lead approved all four proposals ("… (Recommended)" ×4, verbatim in §10).
- Full block: plan_archive.md § M0-TB

> **Commit gate (lead):** Phase 2 closes after M0-TB — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 2: contract, harness and build pipeline"`
> `git push`

# Phase 2b — win-laptop joins: the box probed, set up and ruled
Ordering note (E8 — M0-TB, 2026-10-08, at the lead's request): the lead's next session is on win-laptop,
so its probe, setup and ruling run here, after the Phase 2 commit gate (pushed from linux-pc) and before
lot 1; M0-TE-win and M0-TJ3 were filed after M0-T1 and M0-T1 moved below them by `plan.py move`.

## M0-TE-win · win-laptop joins — probe the box, install what is missing, prove the toolkit and the toolchain · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TB)
- Status: DONE (2026-10-08 16:54, started 16:47) · archived by M0-TC1, 2026-10-09
- Handoff (one line): win-laptop Laser2025-20, Bash tool. Re-run after M0-D2..D5 (all DONE): Verify 1 PASS (python3 = 3.13.14, lint GO); Verify 2 PASS — verify.py --all GO, 500 passed, 3/3 scopes (plan_lint 487, tools_selftest 10/10, toolchain 3/3), counts equal linux-pc's; Verify 3 PASS (toolchain clean GO, 3 plants red); Verify 4 PASS (5 sections). Logs M0-TE-win-r2.* (fresh --task id: the first run's logs are committed and the harness never appends to them). Claim run --changed --base 251b054 (M0-TE-win-r3) NO-GO: one flake, tools_selftest[status_page] 'POST /answer refuses…' ConnectionAbortedError WinError 10053, reproduced 2 of 6 alone — filed M0-D6 (Windows-only, tool unopened). Deviation: rated below (usual), so §0 would send a red claim back to TODO; I closed DONE because the red is an unrelated tool's flake now filed, as M0-D2..D5's closes did — reopen if you want the strict path. I appended one line to the committed logs/M0-TE-win.log by mistake before the harness refused. Updated win_laptop.md § Toolkit and the Hazards line. Ran on model=claude-sonnet-5-5 level=high per rung_record. Next: M0-D6, then M0-TJ3.
- Full block: plan_archive.md § M0-TE-win

## M0-D2 · rung_record.py's selftest fails 4 checks on Windows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TE-win, BEFORE M0-TJ3)
- Status: DONE (2026-10-08 16:13, started 16:07) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Cause: the selftest's expected install line was always POSIX; `now` rightly prints PowerShell on Windows — the tool unchanged, the fixture's `posix` → `host` (hand-written per host). Landed in tools/pb/rung_record.py and PLAYBOOK.md §A.9, byte-identical, annex md5 marker → 3a3c3a3ae9b2 (the lead: «Both, identical»). win-laptop: `rung_record.py selftest` GO 62/62, plants 25/25 red. Claim `--changed --base 251b054` NO-GO: plan_lint GO, tools_selftest 7/3 (was 6/4) — the 3 reds are M0-D3/D4/D5's cases; `--redarm tools_selftest` blocked by the same 3. Owed on linux-pc: the selftest and the red-arm (expected string unchanged there). Detail tasks/M0-D2.md, log logs/M0-D2.log. Ran on model=claude-opus-5-5 level=high. Next: M0-D3.
- Full block: plan_archive.md § M0-D2

## M0-D3 · content_gate.py's selftest stops on a Windows file lock · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TE-win, BEFORE M0-TJ3)
- Status: DONE (2026-10-08 16:23, started 16:17) · archived by M0-TC1, 2026-10-09
- Handoff (one line): content_gate.py selftest GO 36/36 on win-laptop (was 35/37). Two causes, neither a held handle: the selftest stayed chdir'd into its temp `plain` folder through the cleanup (WinError 32), and the child's CRLF stdout missed the `…zorblax_widget\n` match (the --denylist FAIL). Fix is selftest-only: CRLF→LF in child(), chdir(home) before the cleanup, and `-c core.autocrlf=false` on its git add. Landed «Both, identical» (the lead): tools/pb/content_gate.py + PLAYBOOK annex block, md5 90a1a464e16f→993166b08d43. Deviation: Done-when's 37 counted the crash line; the full count is 36. Red-arm of tools_selftest is blocked here by M0-D4/D5 (clean run red); a hand plant in the scratchpad turned the check red. Claim --changed --base 251b054: NO-GO 498/2 (plan, scratch_copy = D4, D5), content_gate GO, plan_lint GO. Owed on linux-pc: the selftest + --redarm tools_selftest. Ran on model=claude-opus-5-5 level=high. Detail: tasks/M0-D3.md. Next: M0-D4.
- Full block: plan_archive.md § M0-D3

## M0-D4 · plan.py's selftest fails one check on Windows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TE-win, BEFORE M0-TJ3)
- Status: DONE (2026-10-08 16:33) · archived by M0-TC1, 2026-10-09
- Handoff (one line): plan.py selftest GO 246/246 on win-laptop (was 245/246). Cause: the selftest wrote the fixture's native absolute path into a `Read:` line; the lint's PATH_RE holds no \ or :, so on Windows it named only big_reference.md and the full-path match missed — neither suspect (the check is a substring test; the lock retry was absent on the failing run). Fix is selftest-only (the lead: «Selftest only»): the fixture names big_reference.md beside the plan and expects that name. Landed «Both, identical» (the lead): tools/pb/plan.py + PLAYBOOK annex block, md5 4886df610160→9c293922e286. Red-arm of tools_selftest blocked here by M0-D5 (clean run red); a hand plant in the scratchpad turned this check red. Claim --changed --base 251b054: NO-GO 498/1 (scratch_copy = D5), plan GO, plan_lint 489 GO. Owed on linux-pc: the selftest + --redarm tools_selftest. Ran on model=claude-opus-5-5 level=high per rung_record (the agent's runtime identity read claude-sonnet-5-5 — disagreement reported in tasks/M0-D4.md). Detail: tasks/M0-D4.md. Next: M0-D5.
- Full block: plan_archive.md § M0-D4

## M0-D5 · scratch_copy.sh's selftest fails one check on Windows: «a dest inside the source» · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TE-win, BEFORE M0-TJ3)
- Status: DONE (2026-10-08 16:46, started 16:40) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Fixed: scratch_copy.sh resolved src by pwd -P but dest by realpath -m; in Git Bash only pwd -P expands 8.3 names (YOHANB~1), case and /tmp, so «dest inside the source» was missed — a real guard hole, not the fixture. New canon() resolves dest like src; landed in tools/pb/ and PLAYBOOK annex §A.7 byte-identical (the lead: «Both, identical»), md5 de04c76b5215 → 02dd8f26fbf3; docs/agent/testing.md names the Git Bash without MSYS=winsymlinks:nativestrict (9/12) and PowerShell's WSL bash.
- Full block: plan_archive.md § M0-D5

## M0-D6 · status_page's selftest flakes on Windows: «POST /answer refuses…» dies with ConnectionAbortedError (WinError 10053) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TE-win, BEFORE M0-TJ3)
- Status: DONE (2026-10-08 17:10, started 16:56) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Cause measured: status_page's server replied 413 to a body over 64 KiB without reading it; the close on unread bytes is a reset on Windows (WinError 10053, 12/300 and 29/300 POSTs in a probe; 403/404 arms 0/300). Fix: drain(n) reads and drops the refused body (≤1 MiB, 2 s) before the 413 — landed in tools/pb/status_page.py and PLAYBOOK annex §A, byte-identical (the lead: «Both, identical»), md5 2fd9acd147e7 → 8cce7ffce793; selftest unchanged. Probe after: 0/300. On win-laptop: --case status_page 20/20 GO; --redarm tools_selftest GO (M0-D6-r2; the first run's clean arm crashed tools_selftest[verify] 0xC0000005 → filed M0-D7, the second sighting); claim --changed --base 251b054 GO, 502 passed, 2/2 scopes. Owed on linux-pc: --case status_page. Seen: M0-D5's red-arm proved its plant red through this flake, not its own check (M0-D6-r2 shows it red properly). Stale «seen once» texts (Hazards, win_laptop.md) left for TZ. Model: claude-opus-5-5, level high (rung_record). Detail: tasks/M0-D6.md. Next: M0-D7, then M0-TJ3.
- Full block: plan_archive.md § M0-D6

## M0-D7 · verify.py's selftest crashes on Windows under the parallel loop: exit 3221225477 (0xC0000005), no output · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D6, BEFORE M0-TJ3)
- Status: DONE (2026-10-08 17:26) · archived by M0-TC1, 2026-10-09
- Handoff (one line): verify.py: case_env() — the one env for scope runs and red-arm arms — adds PYTHONFAULTHANDLER=1 (setdefault), so a native crash leaves its stack in the case log; selftest J2 check + plant (168 checks, 99/99 red); red-arm GO under py -3.12 (148 s). Measured: {python} = sys.executable = the Store App Execution Alias for every case and child; no WER event at either sighting; not reproduced here (burst 0/8) — cause NOT PROVEN (source + probe). Routed by the lead (2026-10-08, « Switch + trace »): on win-laptop every toolkit call is `py -3.12 tools/pb/…` (Boxes fact, Hazard; `py -3` is the Store 3.13 too — the old Hazard's advice corrected). Owed: Done when's 20/20 on win-laptop (the lead's, ~30 min) and linux-pc's GO — flagged to M0-TJ3. Finding not acted on: verify selftest's « two shared scopes ran at the same time » (0.2 s overlap) reads NO-GO under 8× load — load-sensitive, not seen in a real run. Ran on model=claude-opus-5-5 level=high. Detail: tasks/M0-D7.md. Next: M0-TJ3.
- Full block: plan_archive.md § M0-D7

## M0-TJ3 · Direction ruling — win-laptop in play for M0: what runs where · **PLAN + LEAD answers** · Opus 5.5, max · switch · (AFTER M0-TE-win)
- Status: DONE (2026-10-08 18:05, started 17:50) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Ruling verbatim in reports/win_laptop.md § Ruling: Q1 «Build and close lots anywhere» (not my recommendation), Q2 «Laptop versions (Recommended)», Q3 «Off-screen window (Recommended)», Q4 «Skip it». §2.6 executed: R5 rewritten, R4/R9/R12 extended, R15 the box pattern (Rules + m0_rules.md); Superseded: R5's and contract §6.6's old text, the goal's routing of the laptop, M0-D7's 20-run loop (N/A, Q4); contract §3.3, §3.6, §5.4, §6.2.3, §6.4, §6.6 amended [M0-TJ3]; blocks M0-T2, T5 (--offscreen-window, case xvfb→window), T6 (start.bat; no longer an edit job → Sonnet 5.5, high, §0 rule 3), T8, T9, T84, V1, V17 edited; the ~40 later blocks R15 covers listed in the report, the negative scope stated there. Verify 1 [ALREADY RUN — PASS (lint GO, 493 checks, 0 warnings) on win-laptop]. Claim --changed --base 251b054 [ALREADY RUN — FAIL (499/1: tools_selftest[verify] IndexError after 30 checks, a Python crash in the selftest's own parser — my edits touch no tool) on win-laptop]; not reproduced after (case alone 3/3 GO, scope GO M0-TJ3-s01, plan_lint re-run GO M0-TJ3-p01) → filed M0-D8 (usual rung, before M0-T1, inside the Phase 2b gate) on Q4's «filed as a bug». Owed on linux-pc: M0-D2..D7's tools_selftest + its red-arm → flagged into M0-V1, carried V to V, M0-V17 at the latest. Not acted on: stale Hazard texts (11/12, «seen once») and win_laptop.md's TE-win status line — TZ's, as M0-D5/D6 named. Ran on model=claude-opus-5-5 level=max (rung_record). Detail: tasks/M0-TJ3.md. Next: M0-D8, then the Phase 2b commit gate, then M0-T1 on either box.
- Full block: plan_archive.md § M0-TJ3

## M0-D8 · verify.py's selftest crashes on Windows under the parallel loop: IndexError in a three-field parse · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TJ3, BEFORE M0-T1)
- Status: DONE (2026-10-08 18:16, started 18:10) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Cause measured: 19 EMIT trace children append to one trace.txt at once (crash = section C's first spans(), by the 30-check/18-plant count); a Windows append is seek+write, so records overwrite — probe 7/570 lost (HEAD) vs 0/570 (fix). Fix (the lead: «Reader + writer», «Both, identical»): EMIT writes under an O_EXCL lock file (5 s bound); spans() keeps whole records only, a torn one reads missing on its check; plant: a torn t2 record → overlap red, no crash (169 checks, 100/100 red); injected old reader → IndexError as at M0-TJ3. Landed in tools/pb/verify.py + PLAYBOOK annex block, cmp identical, md5 cdf8b7c56213→5f52675a3900. Deviation: the annex lacked M0-D7's case_env hunks — carried in byte for byte to make it identical. win-laptop: --case verify GO; --redarm tools_selftest GO (148 s). Claim --changed --base 251b054: logs/M0-D8.changed.log, run last. Owed on linux-pc: --case verify. Lead's optional: the 10-run repro loop (tasks/M0-D8.md). Ran on model=claude-opus-5-5 level=high. Detail: tasks/M0-D8.md. Next: the Phase 2b commit gate, then M0-T1.
- Full block: plan_archive.md § M0-D8

> **Commit gate (lead):** Phase 2b closes after M0-D8 (M0-TJ3, then the bug it filed) — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 2b: win-laptop joins"`
> `git push`

# Build — the lots M0-TG wrote (2026-10-08): each closed by its V, each phase by its commit gate
Shared by every block below: Rules R10–R14, full text in m0_rules.md — hoisted there by M0-TB (2026-10-08).

# Phase 3 — Lot 1 · the walking skeleton

## M0-T1 · Workspace, toolchain pin and the build check · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TJ3)
- Status: DONE (2026-10-08 18:22) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Workspace landed: Cargo.toml (members crates/*, §1.2 table at the approved minors, wgpu native+webgpu without webgl, eframe wgpu/wayland/x11/default_fonts, one release profile, [profile.test] inherits it), rust-toolchain.toml (1.99.0 + clippy, rustfmt, linux-gnu + wasm32), .cargo/config.toml (build/target), crates/sr-physics (empty), Cargo.lock, tests/cargo.sh (sourced env + build runner), scope build + plant build-wasm-only, docs/agent/testing.md row + § Cargo scopes. Verify 1 [ALREADY RUN — PASS (GO, 2 cases, 9.7 s) on win-laptop]; Verify 2 [ALREADY RUN — PASS (clean GO, plant red on the wasm case only) on win-laptop]; claim --changed --base 251b054 [ALREADY RUN — PASS (GO, 502 passed, 3/3 scopes, 72 s; 6 oracle FLAGs are the uncommitted Phase 2b tool edits, not mine) on win-laptop], logs/M0-T1.changed.log. Not acted on: (1) rustup auto-installed toolchain 1.99.0 beside stable on the first cargo call (R3, ~25 s) — linux-pc does the same; (2) eframe's wgpu path re-enables wgpu webgl+gles by feature unification, against §1.2/Q4 — the lot adding eframe to sr-app must check it; (3) build scope [NOT RUN — owed on linux-pc]. Ran on model=claude-sonnet-5-5 level=high (rung_record). Detail: tasks/M0-T1.md. Next: M0-T2.
- Full block: plan_archive.md § M0-T1

## M0-D9 · red-arm's shared cargo target reuses a planted build: the second `--redarm` of a cargo scope reads a red clean arm · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T1, BEFORE M0-T2)
- Status: DONE (2026-10-09 09:20) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Cause measured (linux-pc): cargo's mtime freshness + the copy's preserved mtimes + one shared target — the clean arm's gpu.rs (09:13:24) looked fresh against the plant's 09:14:03 build. Fix: tests/cargo.sh defines, in a scratch copy only, a cargo wrapper that takes the shared target's lock (flock, else a mkdir lock broken when its holder is gone), touches crates/ assets/ scenes/ Cargo.toml Cargo.lock .cargo/, and builds inside the lock — the lock keeps parallel arms honest too; testing.md § Cargo scopes says it. Evidence: --redarm adapter ×3 GO (clean 10/10, plant 5 red, ≈5.8 s each, only sr-engine recompiles); --redarm build ×3 GO (30.5/1.8/1.6 s); the fallback lock tested concurrent + stale. Deviation: the build plant was never actually fooled (its planted wasm check fails, no artifact). Not proven on win-laptop (Git Bash, mkdir-lock path) — owed at the next V there. A scope calling command cargo / ~/.cargo/bin/cargo bypasses the wrapper. Claim: verify.py --changed --base 516ec7c --task M0-D9 GO on linux-pc (504 passed, 3/3 scopes; 4 oracle FLAGs name M0-T2's uncommitted gpu.sh, adapter plant and verify.json — T2's, not D9's). Ran on claude-opus-5-5, high. Detail: tasks/M0-D9.md. Next: M0-T2 (retry its claim run).
- Full block: plan_archive.md § M0-D9

## M0-T2 · GPU device and adapter choice · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D9)
- Status: DONE (2026-10-09 09:25, started 09:24) · archived by M0-TC1, 2026-10-09
- Handoff (one line): Retry on Opus 5.5, high after M0-D9; no rewrite — try 1's files (sr-engine Cargo.toml, lib.rs, gpu.rs; tests/gpu/main.rs, 10-case adapter module; adapter-case plant; plus tests/gpu.sh, scope adapter in verify.json, testing.md row) read against Deliver and kept. Verify 1 GO on linux-pc (10/0/0; log names Quadro RTX 4000, RTX 5090, llvmpipe, Intel RPL-S, all Vulkan); Verify 2 GO (plant red, 5 failed). Claim run verify.py --changed --base 516ec7c --task M0-T2 is the last step, after this close (logs/M0-T2.changed.log). Findings: default high-performance pick is the Quadro, not the 5090 (flagged M0-T3); several-name match = Vulkan then list order, a declared default (§6.1 silent); two-backend rule proven on the fixed list only; win-laptop list NOT RUN, owed there (R15); eframe webgl feature unification (flagged M0-T5). Ran on claude-opus-5-5, high. Detail: tasks/M0-T2.md. Next: M0-T3.
- Full block: plan_archive.md § M0-T2

## M0-T3 · The cell state and the step loop · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T2)
- Status: DONE (2026-10-09 09:34, started 09:30) · archived by M0-TC1, 2026-10-09
- Handoff (one line): state.rs (WorldConfig 600×400 default, multiples of 8 in [64, 2048]; 14 channels as row-major planes, canonical §2.2 order on the CPU; GPU packed in 3 storage buffers of whole planes — hydro 0–3, species 4–8, 9–13 — + a dims uniform), step/mod.rs (Pass P0–P9, fixed dispatch lists, one compute pass per non-empty slot; only P8 filled), shaders/floors.wgsl (renormalise_species), tests/gpu/state.rs (6 cases), plant state-renorm-skip; plumbing: gpu.sh 'physics' arg → SR_TEST_ADAPTER RTX 5090 / RTX 4080 on Windows (the carried flag), scope state in verify.json (expected 6). Verify 1 [ALREADY RUN — PASS (GO 6/0/0, RTX 5090 Vulkan; worst |ΣX−1| 1.9e-7) on linux-pc]; Verify 2 [ALREADY RUN — PASS (plant RED, 1 failed) on linux-pc]; claim --changed --base 516ec7c [ALREADY RUN — FAIL (build[native]: clippy manual_is_multiple_of + chunks_exact_to_as_chunks in state.rs) then PASS after the fix (GO 508/0/0, 4/4 scopes) on linux-pc]; oracle FLAGs on T2/D9's uncommitted files, not mine. Declared default: all fractions ≤ 0 → pure hydrogen (contract silent; flagged M0-T19 with the missing CPU f64 twin). testing.md row left to M0-T9 (R11), flagged. Ran on model=claude-opus-5-5 level=high. Detail: tasks/M0-T3.md. Next: M0-T4.
- Full block: plan_archive.md § M0-T3

## M0-T4 · The headless command — one step and its run summary · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T3)
- Status: DONE (2026-10-09 09:40) · archived by M0-TC1, 2026-10-09
- Handoff (one line): linux-pc, model=claude-sonnet-5-5 level=high. `sandbox-reactions headless --scene preset:<name> --steps N [--world --max-steps --out --adapter]` runs M0-T3's step N times on the nominal Sun-like disk (Σc=1, a=40, §2.10 mix, at rest, U_th=0.1|W|) and writes summary.json (§2.12.2; ledger/events/objects empty); SR-ADAPTER first, SR-HEADLESS DONE last; exits 0/2/3/4 hand-checked (101 is Rust's, 5/6 have no producer yet). New: crates/sr-app/{Cargo.toml,src/main.rs}, crates/sr-engine/src/headless.rs, tools/pb/launch.json (headless-boot), tests/smoke.sh, tests/plants/scene-refuse.patch, scope `boot` in verify.json, a .gitignore line (launch.py's pid file), Cargo.lock (serde_json). Verify 1 PASS (GO 1/0/0, 1.07 s), Verify 2 PASS (clean GO, plant scene-refuse RED, 38 s); claim run: first --changed NO-GO (plan_lint: my two flags tripped the 5-flag rating refresh), I filed M0-TM1 after this block (the lint's own call), second --changed (task id M0-T4b) GO 511 passed, 5/5 scopes, both on linux-pc — a strict reading of «red claim ends your try» would have trailed this block; the red was the plan's count, not the code. Declared defaults (steps>max-steps → exit 2 until=unmet; no --out → no file; sim_time 0 until P1; massive/giant = nominal disk + SR-WARN; smoke builds in the tree it runs in) — tasks/M0-T4.md. Not done: events.jsonl/dumps/frames (their blocks); docs row flagged to M0-T9, scene files to M0-T25. Next: M0-TM1 (rating pass, runs next), then M0-T5.
- Full block: plan_archive.md § M0-T4

## M0-TM1 · Rating pass — 5 carried flags · **PLAN** · Opus 5.5, max · switch · (AFTER M0-T4)
- Status: DONE (2026-10-09 09:57, started 09:54) · archived by M0-TC1, 2026-10-09
- Handoff (one line): linux-pc, model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Ladder re-read 2026-10-09: the claude-api skill's table (cached 2026-10-06) diffed against the 2026-10-08 copy on disk: Claude Haiku 5.5 added; no rung retired, renamed, repriced or relevelled (Opus 5.5 $4/$20, Sonnet 5.5 $2/$10, ratio 0.5) → Models line re-dated, rungs unchanged, no question (no rung the lead runs changed); Haiku 5.5 as a rung is the lead's call (a RUNGS row + reinstall first). 144 open blocks re-derived, 0 changed (Opus 5.5 max 0 · Opus 5.5 high 0 · Sonnet 5.5 high 0 · Sonnet 5.5 medium 0): 22 gate by (2), 13 usual by (3)'s list, 102 Sonnet high by (3), 7 E on Sonnet medium; the 5 flags (T5, T9 ×2, T19, T25) move nothing; the record moves no class (< 10 blocks below usual); no open block carries a trail. Register: Model ratings 2026-10-09 by M0-TM1. Not acted on: Next task still reads M0-T4 (DONE), outside TM1's diff; Repo facts name Claude Code 2.1.292, the probe reads 2.1.295. lint GO (492), plan_lint GO, its red-arm GO; the claim run verify.py --changed --base 516ec7c --task M0-TM1 is the last step, after this close. Detail: tasks/M0-TM1.md. Next: M0-T5.
- Full block: plan_archive.md § M0-TM1

## M0-T5 · The desktop window and its status endpoint · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T4)
- Status: DONE (2026-10-09 10:17) · archived by M0-TC1, 2026-10-09
- Handoff (one line): linux-pc, model=claude-opus-5-5 level=high. The desktop app: crates/sr-app/src/desktop.rs (SandboxApp, one eframe App for both targets; 1760 × 940, 'Sandbox Reactions', world area bg.space — measured on Xvfb: xwininfo 1760x940, pixels #05070D; --adapter through match_adapter in egui-wgpu's selector, Limits::default(); --offscreen-window at (−20000, −20000), inactive, no taskbar), status.rs (127.0.0.1 HTTP/1.0 GET /status, §3.4's JSON, bound before the window), main.rs routes no-subcommand to it; launch.json game (ask) / game-xvfb / game-offscreen; tests/smoke.sh window (box route); scope desktop + plant status-never-ready. 'ready' = an egui Screenshot event, delivered only for a frame whose surface texture was acquired and then presented — booting at frames 0–2, ready ≈ frame 5, 0.3 s. Deviation, asked: eframe feature wgpu → wgpu_no_default_features (wgpu re-enabled webgl+gles through egui-wgpu's defaults; the lead's yes; contract §0.5 [M0-T5] + Superseded). Defaults: game-offscreen also xvfb:true on Linux (Wayland ignores positions — R4); bad/busy port exit 4. Verify 1 PASS (GO 1/0/0, 1.17 s), Verify 2 PASS (clean GO, plant RED at the 90 s deadline, 138 s), claim --changed --base 516ec7c GO 511 passed 6/6 scopes — all linux-pc. Owed on win-laptop: game-offscreen 'ready' (G-DESK UNVERIFIED, flagged M0-V1); the visible game is V1's launcher run. Flags: M0-T6 (--asked), M0-T7 (SandboxApp), M0-T9 (testing row). Detail tasks/M0-T5.md. Next: M0-T6.
- Full block: plan_archive.md § M0-T5

## M0-T6 · The double-click launcher · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T5)
- Status: DONE (2026-10-09 10:26) · archived by M0-TC1, 2026-10-09
- Handoff (one line): start.sh (LF, +x), start.bat (CRLF, ASCII) and — the lead's extra ask — start.ps1 (CRLF; powershell -ExecutionPolicy Bypass -File start.ps1 [-Service <name>]) at the repo root: start <name|game> --task launcher (--asked for game only: the double-click is the ask), wait for Enter, stop; .gitattributes: *.bat, *.ps1 eol=crlf. verify.json desktop: cases window+launcher (expected 2), paths, plant launcher-no-stop. tests/smoke.sh: case launcher = printf '\n' | bash start.sh game-xvfb, checks ready, stopped, 'running: 0', always stops what it started; plus a port-47811 lock (the scope's cases run in parallel on one port and one pid file — first run raced; my first plant also leaked a game+Xvfb, killed by pid, fixed). Measured on linux-pc (laserax-ai): desktop GO 2/2; --redarm desktop GO, 2 plants RED; verify --changed --base 516ec7c GO 510 cases, 6/6 scopes (logs M0-T6d.*). NOT PROVEN: start.bat/start.ps1 on win-laptop (owed there, R15; start.ps1 ran under pwsh on linux via a py shim only). Not done: docs/agent/testing.md's desktop row (a docs block's, R11). Ran on model=claude-sonnet-5-5 level=high. Next: M0-T7
- Full block: plan_archive.md § M0-T6

## M0-T7 · The web entry — the page, the wasm build, the no-WebGPU page · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T6)
- Status: DONE (2026-10-09 10:58) · archived by M0-TC1, 2026-10-09
- Handoff (one line): web/index.html (canvas, web.loading, inline navigator.gpu + requestAdapter check ahead of the wasm, ?no-webgpu=1 → §4.9 page + srState 'no-webgpu', wasm never loaded), web/Trunk.toml, web/strip-autoload.mjs (post_build hook run by node: trunk always injects a wasm auto-loader + preloads that would load the wasm before the check; the hook strips them and fails loudly if the shape is not found — deviation, not in Deliver), crates/sr-app/src/web.rs (eframe WebRunner on sr-canvas running desktop::SandboxApp; srState loading→ready on the first presented frame, srAdapter, 'error' + message on fatal/device-lost) with wasm32 deps in sr-app/Cargo.toml (console_error_panic_hook added to Cargo.lock); desktop.rs now shares wgpu_setup() and drops its wasm dead_code allow. tests/web/check.sh case build + scope web (verify.json) + plants web-no-gpu-check, web-autoload. Measured on linux-pc (laserax-ai): verify web GO 1/1; --redarm web GO (2 plants RED); --changed --base 516ec7c GO, 511 cases, 7/7 scopes (its FLAG oracle lines are earlier tasks' uncommitted files + the web scope itself). Headless Chrome 155 + WebGPU flags via a throwaway CDP script: srState 'ready', srAdapter set; ?no-webgpu=1 → page + 'no-webgpu' (NOT the T15 smoke; win-laptop NOT RUN). Findings not acted on: srAdapter reads ' (BrowserWebGpu, Other, driver )' (browsers hide adapter names); any web start failure shows error.no_adapter (detail in data-sr-error-detail); strings.rs (M0-T13) should replace web.rs's two error constants; docs/agent/testing.md web row left to the docs block (R11). Ran on model=claude-sonnet-5-5 level=high per rung_record. Next: M0-T8
- Full block: plan_archive.md § M0-T7

## M0-T8 · The capture harness, first form · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T7)
- Status: DONE (2026-10-09 11:29) · archived by M0-TC1, 2026-10-09
- Handoff (one line): capture.rs + --capture/--out in desktop.rs, tests/capture/{smoke.json,check.sh}, plant capture-no-captions, scope capture (smoke, refuse); png native-only dep in sr-app. linux-pc (laserax-ai): verify capture GO 2/2; --redarm GO (plant RED); PNGs 1846x1016, 'step 1 · paused' vs 'step 6' differ. Claim run (the lead's, M0-T8d): --changed --base 516ec7c GO, 512 passed, 8/8 scopes. The smoke leaked a window onto the Wayland desktop (winit prefers WAYLAND_DISPLAY over xvfb-run's DISPLAY); check.sh now runs the app under env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE; the lead's watched re-run (M0-T8e): GO, no window. Deviations: app counts steps itself (paused/step + stand-in readout); /status step stays 0. Not acted on: red-arm loop is long (desktop 262 s, cold build per arm, T5's design); win-laptop --offscreen-window route NOT RUN; testing.md row left to the docs block (R11). Ran on model=claude-sonnet-5-5 level=high per rung_record. Next: M0-T9. Detail: tasks/M0-T8.md
- Full block: plan_archive.md § M0-T8

## M0-T9 · Docs — architecture.md, running.md and lot 1's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T8)
- Status: DONE (2026-10-09 11:32, started 11:30) · archived by M0-TC1, 2026-10-09
- Handoff (one line): linux-pc (laserax-ai), model=claude-sonnet-5-5 level=high (rung_record). New docs/agent/architecture.md (crates and one-way deps, sr-engine modules, P0–P9 order, data flow, the observer boundary; no contract number restated) and docs/agent/running.md (start.sh/.bat/.ps1, the binary's commands as built, launch.py, adapters per box, the private-display rule and win-laptop's off-screen route, the web build, long runs). docs/agent/testing.md: rows state, boot, desktop, web, capture (build, adapter were there), gpu.sh's 'physics' argument, tests/smoke.sh's own-target rule and launch.json, a Boxes section naming laserax-ai and Laser2025-20 with R15's routes — the carried flags of T3, T4, T5 all written. Verify 1 [ALREADY RUN — PASS (50 and 32 § citations) on linux-pc]; Verify 2 [ALREADY RUN — PASS (7 rows) on linux-pc]; claim --changed --base 516ec7c [ALREADY RUN — PASS (GO, 510 passed, 8/8 scopes, 5.3 s) on linux-pc]; logs/M0-T9.log; docs only, no scope, no red-arm. Not acted on: testing.md's 'Scopes today' header and the '3.2 s, 62 cases' loop line are stale → flagged M0-V1; the new rows' win-laptop costs are owed there. Detail: tasks/M0-T9.md. Next: M0-TV1.
- Full block: plan_archive.md § M0-T9

## M0-TV1 · UI/UX pass — Phase 3: the first window and the web pages · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T9)
- Status: DONE (2026-10-09 11:54) · archived by M0-TC1, 2026-10-09
- Handoff (one line): 9 captures in images/tv1/ (git-ignored): desktop first-frame/paused/stepped/running (1760x940, llvmpipe, private Xvfb), no-WebGPU page at 1280x800 and 390x844 plus the flag-less default, loading line at both widths (SYNTHETIC: stub navigator.gpu; headless Chrome here has no adapter under 3 flag sets). All 9 looked at; 2 flagged (first-frame, no-webgpu-1280x800); the lead had no remark (asked twice); review1.md holds triage + verdict; no D filed — the window is a stand-in until TV2, web strings and tokens match §4.1/§4.11. Review server (8765) and web server (47812) stopped. Not acted on: capture_web.mjs overflow measure (needs Playwright, M0-T15); win-laptop off-screen route NOT RUN. Motion, timing, frame rate, input feel: NOT PROVEN (static capture). Claim run, linux-pc (laserax-ai): --changed --base 516ec7c GO, 510 passed, 8/8 scopes (its FLAG oracle lines are earlier tasks' uncommitted manifest changes). Ran on model=claude-sonnet-5-5 level=high per rung_record. Next: M0-V1. Detail: tasks/M0-TV1.md
- Full block: plan_archive.md § M0-TV1

## M0-V1 · Validation — lot 1: the walking skeleton · **CHECK** · Opus 5.5, max · switch · (AFTER M0-TV1)
- Status: DONE (2026-10-09 12:35) · archived by M0-TC1, 2026-10-09
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=max (rung_record now, launch + after the lead's answers). Verdict DEFECTS(4) — reports/v1.md: lot 1 conforms end to end on linux-pc. Verify 1 verify.py --all --task M0-V1 [ALREADY RUN — PASS (GO, 537 passed, 10/10 scopes; 449.7 s wall = 6.1 s cases + 443 s red-arming the six scopes new since HEAD; under the 600 s budget, Rules line restated by hand — plan.py has no call for it) on linux-pc]; --redarm of the 7 lot-1 scopes [ALREADY RUN — PASS (9 plants, 9 red) on linux-pc]; M0-TJ3's flag tools_selftest + its red-arm [ALREADY RUN — PASS (10/10, content_gate 36/36) on linux-pc]. Adversarial met: desktop[launcher] boots via start.sh → game-xvfb; the boot scope's own two summary.json files, caught in flight (it deletes them), carry the 13 §2.12.2 keys, steps 200. Strings 7/7 verbatim; §1.2 versions exact, 0 webgl/gles. The lead: Files run «Window stayed open», terminal run «Opened, closed on Enter». Filed: M0-TC1 (12 DONE blocks > 10, runs next), M0-D10 (scope paths: --changed skips G-BOOT on a headless.rs edit), D11 (--out first, --help after a flag refused), D12 (docs: «Intel UHD 770», §7 restatements, flag 4's loop line — a CHECK writes no docs), D13 (boot deletes the summary.json it grades). NOT PROVEN: win-laptop entire (2 flags → M0-V2), the page in a browser (M0-T15), --offscreen-window (source only). Blind spot: window and engine never ran together (M0-T84). Waste named: my separate red-arm (474 s) repeated --all's for six scopes. Verify 1's 44 FLAG lines = lot 1's uncommitted files, none mine. A 10:20 start.sh still waits in VS Code terminal pts/23 — not mine, untouched. Detail tasks/M0-V1.md. Next: M0-TC1.
- Full block: plan_archive.md § M0-V1

## M0-TC1 · Compaction — Phase 3's close: twelve DONE blocks to stub, more than ten (PLAYBOOK §8, §11) · **MOVE** · Sonnet 5.5, medium · switch · (AFTER M0-V1)
- Status: DONE (2026-10-09 12:36, started 12:36)
- Read: this file (rules + this task) + plan_archive.md as the shape to keep
- Deliver: `plan.py stub <ID>... --by M0-TC1` over every DONE / N/A / DEFERRED block of the closed phases — Phase 2b's
  (M0-TE-win, M0-D2, M0-D3, M0-D4, M0-D5, M0-D6, M0-D7, M0-TJ3, M0-D8) and Phase 3's (M0-T1, M0-D9, M0-T2, M0-T3, M0-T4,
  M0-TM1, M0-T5, M0-T6, M0-T7, M0-T8, M0-T9, M0-TV1, M0-V1), never a TODO block (this one, the D blocks M0-V1 filed) ·
  the Pipeline state back to a register of pointers · lines repeated in three or more blocks hoisted, long Rules
  entries moved to m0_rules.md (index line kept, text unchanged) · Repo facts de-duplicated · every moved line
  diffed byte-for-byte, counts reported · `plan.py show`'s header count before and after in the handoff.
- Pass: `plan.py lint` GO, header within budget (600 / 60), diff count = moved count. Fail: any TODO block touched,
  any rule reworded, any fact dropped.
- Handoff: 22 DONE blocks stubbed (Phase 2b: M0-TE-win, D2–D8, TJ3; Phase 3: M0-T1–T9, D9, TM1, TV1, V1) into plan_archive.md: 450 lines moved, 450 verified byte-for-byte; M0-D8's commit-gate trailer kept in the plan (WARN, by design). No TODO block touched. Rules already index lines (text in m0_rules.md) and Pipeline state already pointers — nothing to move there; Repo facts left as is (no safe de-dup measured, nothing dropped). plan.py show header=225 before and after (lint header 404/600 after, pipeline 9/60); plan lint GO 499 checks, 1 warn (Read: names plan_archive.md whole, this block's own line). Ran on model=claude-sonnet-5-5 level=medium per rung_record (linux-pc). Next: M0-D10.

## M0-D10 · Lot 1's scopes leave out inputs their verdicts depend on: `--changed` skips G-BOOT on a headless.rs edit · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-TC1)
- Status: DONE (2026-10-09 12:45)
- Sizing: E
- Blocks: nothing — every later `--changed` claim under-selects until it lands
- Caused by: M0-T4
- Files: tools/pb/verify.json (the `paths` of boot, desktop, capture, web, adapter, state) — opened by M0-V1, not changed.
  Opened: tools/pb/verify.anchor.json, tests/plants/scene-refuse.patch, tests/smoke.sh, tests/gpu.sh, tests/web/check.sh
- Read: this file (rules + this task) + docs/agent/testing.md § Adding a scope (step 4) + tools/pb/verify.json
- Symptom: measured on linux-pc, 2026-10-09 (M0-V1): the `boot` scope grades `sandbox-reactions headless` (G-BOOT) but its
  `paths` are crates/sr-app/**, tools/pb/launch.json, tests/smoke.sh and its plants — its own plant `scene-refuse` patches
  crates/sr-engine/src/headless.rs, outside them; verify.anchor.json records headless.rs (and gpu.rs, state.rs,
  step/mod.rs) as covered by adapter, build and state only, Cargo.lock / Cargo.toml / rust-toolchain.toml by build only,
  and desktop.rs (the SandboxApp the web build runs) by boot, build, capture and desktop, never web. So `verify.py
  --changed` after an edit to headless.rs alone runs build, adapter and state and never G-BOOT; a dependency bump in
  Cargo.lock reruns only the build scope — never the GPU tests, the window or `trunk build` (whose wasm-bindgen must equal
  trunk's wasm-bindgen-cli, §1.2). The same gap, in the scopes of M0-T2, M0-T3 (Cargo.*, tests/cargo.sh), M0-T5, M0-T8
  (crates/sr-engine/**) and M0-T7 (all of sr-app and sr-engine). Repro: read the anchor —
  `python3 -I -c "import json;print(sorted(json.load(open('tools/pb/verify.anchor.json'))['paths']['crates/sr-engine/src/headless.rs']))"`
  → ['adapter', 'build', 'state']. Cause: measured (the manifest's globs).
- Deliver: tools/pb/verify.json — each scope's `paths` names every input its cases compile or source: boot, desktop and
  capture + `crates/sr-engine/**`; web: `crates/sr-app/**` and `crates/sr-engine/**` in place of its three narrower globs;
  boot, desktop, capture, web, adapter and state + `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/**`,
  `tests/cargo.sh` (every one of their runners sources it); no case, plant or `expected` changed.
- Done when: on linux-pc, the anchor read above lists boot after the claim run, and Cargo.lock lists adapter, boot,
  build, capture, desktop, state and web; `python3 tools/pb/verify.py --changed --base <the last commit gate's SHA>
  --task M0-D10` GO (the changed entries' plants red).
- Handoff: verify.json: boot, desktop, capture + crates/sr-engine/**; web: crates/sr-app/** + crates/sr-engine/** replace its three narrower globs; boot, desktop, capture, web, adapter, state + Cargo.toml, Cargo.lock, rust-toolchain.toml, .cargo/**, tests/cargo.sh; no case, plant or expected changed. Anchor on linux-pc: headless.rs -> adapter, boot, build, capture, desktop, state, web; Cargo.lock -> the same seven. Claim --changed --base 516ec7c [ALREADY RUN — PASS (522 passed, 8/8 scopes GO) on linux-pc], logs/M0-D10.*.log; --redarm boot [ALREADY RUN — PASS (scene-refuse red) on linux-pc]. Deviations: base is the latest commit 516ec7c (Phase 3 gate not yet made), so FLAG oracle lines list lot-1 plants/tests as changed — expected, not this task's edits; verify.json rewritten via json.dump (the scopes' non-ASCII text now \u-escaped, content equal). Not owed on win-laptop: untested there. Ran on model=claude-sonnet-5-5 level=medium (rung_record). Next: M0-D11.

## M0-D11 · The binary refuses its own flags by position: `--out` first, `--help` after a flag read "unknown argument" · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-D10)
- Status: DONE (2026-10-09 12:48)
- Sizing: E
- Blocks: nothing
- Caused by: M0-T8
- Files: crates/sr-app/src/main.rs (the first-argument `match` in `main`), crates/sr-app/src/desktop.rs (`native::parse`)
  — opened by M0-V1, not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §3.3 (Desktop, Capture) + the two files above
- Symptom: measured on linux-pc, 2026-10-09 (M0-V1, logs/M0-V1.log § hand checks): `build/target/release/sandbox-reactions
  --out <dir> --capture tests/capture/smoke.json` → `SR-ERROR unknown argument "--out"`, exit 4, while `--capture
  tests/capture/smoke.json --out <dir>` runs (exit 0, two PNGs); `sandbox-reactions --adapter llvmpipe --help` →
  `SR-ERROR unknown argument "--help"`, exit 4, while `--help` alone prints the usage, exit 0. Cause, measured in the
  source: main.rs routes on the first argument alone and its desktop arm lists `--adapter | --status-port |
  --offscreen-window | --world | --capture | --measure-ui`, not `--out`; `native::parse` has no `-h` / `--help` arm.
- Deliver: the smallest fix in those two places — every argument list whose first word is not `headless` (and not
  `-h` / `--help`) reaches the desktop parser, which answers `-h` / `--help` anywhere with the usage, exit 0; an unknown
  argument stays `SR-ERROR` naming it, exit 4; `--world` and `--measure-ui` keep their "not built yet" refusal.
- Done when: on linux-pc, `--out <dir> --capture tests/capture/smoke.json --adapter llvmpipe` under the capture's route
  (`xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE …`) exits 0 with two PNGs;
  `--adapter llvmpipe --help` prints the usage, exit 0; `--bogus` exit 4; `verify.py --changed --base <the last commit
  gate's SHA> --task M0-D11` GO.
- Handoff: main.rs now sends every non-headless, non---help-first argument list to desktop::run; desktop parse answers -h/--help anywhere with the usage (exit 0), unknown stays SR-ERROR exit 4, --world/--measure-ui keep their refusal. Measured on linux-pc: --out first + --capture under xvfb/llvmpipe exit 0 with 2 PNGs; --adapter llvmpipe --help exit 0; --bogus exit 4 (logs/M0-D11.log). Not guarded: no scope tests flag order and the existing capture plant does not cover it — a case is a CHECK/new-scope job, not filed as a D (lead's call). Ran on rung_record: model=claude-sonnet-5-5 level=medium. Next: M0-D12.

## M0-D14 · The adapter scope reads NO-GO: after one NVIDIA device is made, wgpu lists only the Intel iGPU and llvmpipe · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D11)
- Status: DONE (2026-10-09 18:16)
- Blocks: M0-D12's claim run, and every `--changed`/`--all` that reaches the adapter scope
- Caused by: unknown
- Files: crates/sr-engine/tests/gpu/main.rs (lines 105–140, `live_this_boxs_adapters_are_listed`, `live_one_device_per_adapter_found_by_a_differently_cased_name`), crates/sr-engine/src (the headless device path, `Gpu::new_headless`) — opened by M0-D12, not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §6.1–§6.3 + milestones/m0/logs/M0-D12b.adapter.log
- Symptom: measured on linux-pc (laserax-ai), 2026-10-09 evening, 4 times (verify.py --changed, verify.py adapter, `bash tests/gpu.sh adapter` ×3, with and without `DISPLAY=:1`, shell `XDG_SESSION_TYPE=tty`): `adapter` reads 8 passed, 2 failed. The tests before them list all four adapters (Quadro, RTX 5090, Intel `Intel(R) Graphics (RPL-S)`, llvmpipe); in `live_one_device_per_adapter_found_by_a_differently_cased_name` the device for the Quadro is made, then the next `Gpu::new_headless(Some("nvidia geforce rtx 5090"))` lists only [Intel, llvmpipe] and panics (main.rs:129); `live_this_boxs_adapters_are_listed` then sees the same two and reports « missing: Quadro RTX 4000, RTX 5090 » (main.rs:118). `nvidia-smi` lists both GPUs idle. M0-V1 read the same scope 10/10 GO earlier the same day (logs/M0-V1.adapter.log). Repro: `bash tests/gpu.sh adapter`. Suspected cause: not measured — the NVIDIA Vulkan ICD failing to enumerate again once a device of the same driver is alive in the process (or a changed box state since M0-V1); check the first failing instance's `Instance` flags and `VK_*` / ICD state, then whether a fresh process lists them.
- Deliver: the cause measured and the smallest fix in the test or the headless device path (or, if the box is at fault, the box state named in docs/agent/testing.md § Hazards); no loosened expectation.
- Done when: on linux-pc, `python3 tools/pb/verify.py adapter --task M0-D14` GO, 10 cases, and `--redarm adapter` GO (adapter-case still red).
- Handoff: linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record). Cause measured: each wgpu::Instance dropped unloads the Vulkan ICDs; every NVIDIA reload leaks glibc static TLS and from the 11th instance in a process the loader logs « libnvidia-tls.so.595.99.02: cannot allocate memory in static TLS block » and ignores nvidia_icd.json, so wgpu lists only Intel + llvmpipe (VK_LOADER_DEBUG; probe: 10 ok then 30 without NVIDIA; 40/40 with instances kept alive or GLIBC_TUNABLES optional_static_tls raised). Fix: sr-engine gpu.rs headless_instance() — one process-wide instance (OnceLock; wasm keeps one per call), used by Gpu::new_headless and the test's live_adapters; no expectation loosened. verify.py adapter [ALREADY RUN — PASS (10/0/0) on linux-pc]; --redarm adapter [ALREADY RUN — PASS (clean GO, adapter-case RED 5 failed) on linux-pc]; claim --changed --base 516ec7c [ALREADY RUN — PASS (523 passed, 8/8 scopes GO) on linux-pc] — run twice on the same tree by mistake, both GO; its FLAG oracle lines are earlier tasks' files since 516ec7c. Not explained: why M0-V1 read 10/10 the same day (same driver, glibc, loader, layers, source) — reading, not measured: unload timing. Not acted on: a plan Hazards line (one wgpu instance per process on linux-pc) is TZ's/the lead's; win-laptop NOT RUN (owed on win-laptop at its next V). Plan: close --set "Caused by=…" wrote a register row, not the block's field — that one row removed with the file-edit tool (plan.py has no row delete); Caused by stays unknown (a box/driver limit, no block). Detail: tasks/M0-D14.md. Next: M0-D12 (re-run its claim).

## M0-D15 · desktop[launcher] reads « 'start.bat' is not recognised » on win-laptop: the case's `cmd //c "start.bat window"` never reaches the file · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D14)
- Status: DONE (2026-10-09 19:27)
- Blocks: M0-D13's claim run, and every `--changed`/`--all` that reaches the desktop scope on win-laptop
- Caused by: M0-T6
- Files: tests/smoke.sh (`run_launcher`, the MINGW branch), start.bat — opened by M0-D13, not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §8 + tests/smoke.sh + milestones/m0/logs/M0-D13b.desktop.log
- Symptom: measured on win-laptop (Laser2025-20), 2026-10-09 (M0-D13's claim, `verify.py --changed --base fc0a0fe --task M0-D13b`): `desktop[launcher]` exit 1 in 4.95 s, output « 'start.bat' n'est pas reconnu en tant que commande interne ou externe… », then « === NO-GO: the launcher exited 1 === ». `desktop[window]` is GO the same run. start.bat exists at the repo root. Repro: `py -3.12 tools/pb/verify.py desktop --case launcher --task M0-D15`. Suspected cause: not measured — Git Bash rewrites `cmd //c "start.bat window"` (path conversion/quoting) or cmd's cwd/PATH lacks the repo root (`.\start.bat`); smoke.sh edited only the headless-boot branch, so the break predates M0-D13.
- Deliver: the cause measured and the smallest fix in the launcher case's call (or start.bat); no loosened expectation.
- Done when: on win-laptop, `py -3.12 tools/pb/verify.py desktop --task M0-D15` GO and `--redarm desktop` GO.
- Handoff: win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record). Cause measured: the Bash tool's env sets NoDefaultCurrentDirectoryInExePath=1, so cmd skips the cwd for a bare name; A/B/C on an echo-only probe.bat: bare+var → « not recognised », var unset → runs, .\ prefix → runs (logs/M0-D15.log). Fix: tests/smoke.sh run_launcher MINGW → cmd //c ".\start.bat $service"; nothing loosened. Deviation, the lead's yes (« Extend the plant (Recommended) »): launcher-no-stop.patch patched start.sh only, which made it unfailable on win-laptop (first --redarm NO-GO); it now also drops start.bat's stop line (LF hunk, applies to CRLF via git apply and patch). verify.py desktop [ALREADY RUN — PASS (2/0/0) on win-laptop]; --redarm desktop [ALREADY RUN — PASS (clean GO, both plants RED, 606 s) on win-laptop]; claim --changed --base fc0a0fe [ALREADY RUN — FAIL (514/1: plan_lint, boot, desktop GO; tools_selftest[capture_web] 2 failures, M0-D16's native crash, the same in M0-D13b's run before this fix; the anchor's 11 never-checked are tools/pb files left unrecorded by that red, my reading) on win-laptop]. Finding: a desktop red-arm takes 606 s here, over the 10-minute line (Rule 4), so it is the lead's run on win-laptop until shortened. testing.md's desktop row still reads « owed ». Detail: tasks/M0-D15.md. Next: M0-D16, then M0-D13's claim.

## M0-D16 · tools_selftest[capture_web] crashes in review_page.py's two cases on win-laptop: a native crash with a faulthandler stack and no Python frame · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D15)
- Status: DONE (2026-10-10 08:49)
- Blocks: M0-D13's claim run, and every `--changed`/`--all` that reaches tools_selftest on win-laptop
- Caused by: unknown
- Files: tools/pb/review_page.py, the capture_web selftest cases — opened by M0-D13 (log only), not changed
- Read: this file (rules + this task) + milestones/m0/logs/M0-D13b.tools_selftest.log (lines 27–29) + the Hazards line on M0-D7's 0xC0000005
- Symptom: measured on win-laptop (Laser2025-20), py 3.12.10, 2026-10-09 (M0-D13's claim): `tools_selftest[capture_web]` 1 failed, 2 failures — « review_page.py reads every capture, and shows none until the TV flags one » and « review_page.py builds its page once the TV flags a capture with its note », each with a faulthandler « Current thread … (most recent call first): <no Python frame> ». Same family as M0-D7's 0xC0000005 (Hazards). Repro: `py -3.12 tools/pb/verify.py tools_selftest --case capture_web --task M0-D16`. Suspected cause: not measured — the Node child (capture_web.mjs) or Chrome dying, versus the Python process; check the exit code and whether a leftover process from the killed claim run (M0-D13 ran `taskkill` on python/cargo) was in the way: re-run first on a quiet box.
- Deliver: the cause measured and the smallest fix; if it was only the killed processes, say so and close with no change.
- Done when: on win-laptop, `py -3.12 tools/pb/verify.py tools_selftest --task M0-D16` GO.
- Handoff: win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record). Cause measured, not a native crash and not M0-D13's killed processes (quiet box, case alone, same 2 reds): capture_web.mjs spawns PB_PYTHON || bare 'python' on win32, the harness never set PB_PYTHON, and bare python is now C:\Program Files\Python314 — Python 3.14.8 installed machine-wide 2026-10-09 without its Standard Library (Lib\ holds only site-packages); it dies « Failed to import encodings module », whose dump tail is the « <no Python frame> » we saw. Fix: tools/pb/verify.py case_env sets PB_PYTHON=sys.executable (setdefault, a caller's value wins), plus a selftest check and a plant (verify 171/0, plants 101/101); nothing loosened. Injected bug (line removed) → both cases red, restored. tools_selftest [ALREADY RUN — PASS (10/0/0, 72.8 s) on win-laptop]; --redarm tools_selftest [ALREADY RUN — PASS (clean GO, plant RED, 152 s) on win-laptop]; claim --changed --base fabab0c [ALREADY RUN — PASS (511/0/0, 2/2 scopes, 69.3 s) on win-laptop]. Finding, the lead's: repair or uninstall Python 3.14.8 (Modify → Standard library) — until then bare python, py and py -3 fail at startup on win-laptop (py -3.12 is fine); Repo facts do not list it. Detail: tasks/M0-D16.md. Next: M0-D13's claim on a fresh id.

## M0-D12 · Lot 1's docs name an adapter wgpu does not list, restate contract numbers and carry a pre-lot-1 loop line · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D14 — the lead moved D14 first, 2026-10-09: D12's claim run needs the adapter scope green)
- Status: DONE (2026-10-09 18:27)
- Carried flags: [M0-D12, 2026-10-09] [M0-D12, 2026-10-09] Docs done and in the tree (Done-when greps clean, M0-T9's two greps 50/32 § and 7 rows); only the claim run is red: `verify.py --changed --base 516ec7c --task M0-D12` NO-GO on the adapter scope (2 of 10 fail — NVIDIA adapters drop out of wgpu's list after the first device; repro `bash tests/gpu.sh adapter`, logs/M0-D12.log) — not caused by a docs edit, filed as M0-D14. Re-try: keep the three edited pages (git diff docs/agent), re-run the claim once M0-D14 is green, then close. · [M0-D14, 2026-10-09] M0-D14 DONE: the adapter scope is GO again on linux-pc (10/0/0; one process-wide wgpu instance in sr-engine gpu.rs) — re-run your claim --changed --base 516ec7c --task M0-D12-r2 (a fresh id: your logs are not appended to), then close.
- Blocks: nothing
- Caused by: M0-T9
- Files: docs/agent/running.md (line 50, the Adapters table; § The binary's exit codes), docs/agent/testing.md (line 28,
  the `adapter` row; § Scopes today's header and its loop line), docs/agent/architecture.md (line 65) — opened by
  M0-V1, not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §7 + milestones/m0/reports/v1.md § The complete
  loop + the three pages, by the lines named
- Symptom: measured on linux-pc, 2026-10-09 (M0-V1, logs/M0-V1.log): (1) running.md's column « Adapters wgpu lists »
  and testing.md's adapter row name « Intel UHD 770 »; wgpu lists that GPU as « Intel(R) Graphics (RPL-S) », so a
  reader's `--adapter "UHD 770"` exits 3 (`--adapter "RPL-S"` exits 0). (2) §7: pages « never restate a number that
  this contract or calibration.json holds » — running.md restates §3.3's exit codes (0 · 2 · 3 · 4 · 5, 6 · 101) and
  architecture.md:65 « read back ≤ 2 frames late » (§1.9.1). (3) testing.md's « Scopes today (linux-pc, 2026-10-08,
  warm caches) » header and « The complete loop: 3.2 s wall on linux-pc (62 cases, `jobs` 8) » predate lot 1's seven
  scopes (M0-T9's flag to M0-V1, which writes no docs — PLAYBOOK §0, CHECK); the web row's « ≈ 40 s `--redarm` » read
  11.4 s at M0-V1. Cause: measured (the pages against the binary and the contract).
- Deliver: the three pages corrected in place — the adapter named as wgpu lists it, with the substring that matches
  it; §3.3's exit codes and §1.9.1's figure replaced by pointers to their sections; testing.md's header dated and its
  loop line restated from reports/v1.md § The complete loop (M0-V1's measured `--all`, linux-pc), each row's cost from
  M0-V1's logs where it moved; no other text.
- Done when: on linux-pc, `grep -n "UHD 770" docs/agent/*.md` and `grep -nE "frames late|101"
  docs/agent/architecture.md docs/agent/running.md` print nothing; M0-T9's two Verify greps still pass (≥ 5 § lines
  in each page; 7 scope rows in testing.md).
- Handoff: linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record). Docs corrected in place, uncommitted, as the first try left them plus one tightening: running.md's exit-code line now reads « Exit codes: §3.3. » (the first try's « a refused flag or scene is 4; no adapter is 3 » still restated §3.3, against Deliver and §7). Also in the tree: running.md's iGPU named « Intel(R) Graphics (RPL-S) » with the --adapter "RPL-S" match; architecture.md:65 figure dropped, § 1.9.1 cited; testing.md's adapter row, Scopes header dated M0-V1, loop line from reports/v1.md § The complete loop, the seven rows' costs from M0-V1's logs. Done-when [ALREADY RUN — PASS (both greps empty; 50/32/17 § lines; 7 scope rows) on linux-pc]. Claim --changed --base 516ec7c --task M0-D12-r2 [ALREADY RUN — PASS (GO 522/0/0, 8/8 scopes, 4.91 s) on linux-pc], logs/M0-D12-r2.log; its one flag (M0-D12-r2 is no block) is the fresh-id rule, expected. First try's red (adapter 8/10) was M0-D14, now DONE. Not acted on: architecture.md:65 reads « read back async. » (terse, not wrong); testing.md's win-laptop costs stay as written. Next: M0-D13.

## M0-D13 · The boot scope deletes the summary.json it grades: a V cannot read what G-BOOT checked · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D12)
- Status: DONE (2026-10-10 08:56)
- Carried flags: [M0-D13, 2026-10-09] Change is in the tree (tests/smoke.sh prints the checked summary.json verbatim, after the check; boot GO, logs/M0-D13.boot.log holds version/adapter/steps 200; --redarm boot GO, scene-refuse red) — measured on win-laptop (Laser2025-20), not linux-pc. Only the claim run is red: py -3.12 tools/pb/verify.py --changed --base fc0a0fe --task M0-D13b NO-GO, 508 passed 2 failed: (1) desktop[launcher] 'start.bat' not recognised as a command (tests/smoke.sh run_launcher: cmd //c "start.bat window" from Git Bash, start.bat exists at the root) (2) tools_selftest[capture_web] review_page.py cases crash with a faulthandler stack, no Python frame (the M0-D7 0xC0000005 family). Neither is in smoke.sh's headless-boot branch; smoke.sh is in both scopes' paths. Also the harness rewrote tools/pb/verify.anchor.json (62 lines). Re-try: keep the smoke.sh edit, get those two reds filed/fixed, re-run the claim on a fresh id, then close. The unverified 0.5 claim: base fc0a0fe is HEAD. · [M0-D16, 2026-10-10] M0-D16 DONE: tools_selftest[capture_web]'s two reds were a stdlib-less Python 3.14 on win-laptop's machine PATH, not a crash; verify.py case_env now passes PB_PYTHON (the harness's interpreter) to every case — tools_selftest GO 10/0/0 on win-laptop. With M0-D15's fix, re-run your claim on a fresh id (M0-D13c), then close.
- Sizing: E
- Blocks: nothing — each lot V's Adversarial reads the boot scope's summary.json
- Caused by: M0-T4
- Files: tests/smoke.sh (the `headless-boot` branch and `_sr_unlock`) — opened by M0-V1, not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.12.2, §5.4 (G-BOOT) + tests/smoke.sh
- Symptom: measured on linux-pc, 2026-10-09 (M0-V1): tests/smoke.sh says it runs the boot command « once into a folder
  it keeps », but that folder is `mktemp -d` and the EXIT trap's `_sr_unlock` runs `rm -rf "$dir"`; launch.py removes
  its own `--out`. The scope's log keeps one line, « summary.json: 13 keys of §2.12.2, steps = 200 », so no reader can
  see the file G-BOOT graded — M0-V1's Adversarial (« reads the summary.json the boot scope wrote ») was met only by an
  inotify watcher copying it in flight (reports/v1.md). Cause: measured (the script).
- Deliver: the smallest change in tests/smoke.sh — the `headless-boot` branch writes the summary.json it checked,
  verbatim, into its own output (so the scope's log holds it), and its comment says what it keeps; the check itself
  unchanged.
- Done when: on linux-pc, `python3 tools/pb/verify.py boot --task M0-D13` GO and logs/M0-D13.boot.log holds the
  summary.json's text (its `"version"`, `"adapter"` and `"steps": 200`); `python3 tools/pb/verify.py --redarm boot --task
  M0-D13` GO (scene-refuse still red); the `--changed` claim GO.
- Handoff: tests/smoke.sh's headless-boot branch prints the summary.json it checked, verbatim, after the unchanged §2.12.2 check (in the tree since fabab0c). Claim on win-laptop (Laser2025-20): verify.py --changed --base fc0a0fe --task M0-D13c GO, 514 passed 0 failed, 4/4 scopes (plan_lint, tools_selftest, boot, desktop); logs/M0-D13c.boot.log holds version/adapter/steps 200. The 2026-10-09 try's two claim reds were filed and fixed as M0-D15 and M0-D16. Deviation: measured on win-laptop, not linux-pc — boot + --redarm boot on linux-pc owed (R15), flagged to M0-V2. Ran on Opus 5.5, high (model=claude-opus-5-5 level=high). Detail: tasks/M0-D13.md. Next: the Phase 3 commit gate (lead), then the next TODO.

> **Commit gate (lead):** Phase 3 closes after M0-V1 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 3: the walking skeleton"`
> `git push`

# Phase 4 — Lot 2 · the registries, the equation of state and the guards

## M0-T10 · Sandbox units and the element and constant registries · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V1)
- Status: DONE (2026-10-10 09:04) · archived by M0-V2, 2026-10-10
- Handoff (one line): win-laptop (Laser2025-20), model=claude-sonnet-5-5 level=high (rung_record). assets/elements.json (§1.5 verbatim) + assets/physics.json (52 keys, §2.5.1 + §2.7 initial values, flat object; spellings in tasks/M0-T10.md) embedded and validated by sr-physics registry.rs (Elements, Physics, Standins, composition → 1/mu, Y_e, Z_met) and units.rs (M0 = 2πΣc a²/3 = 3351.03); tests/cpu.sh is the CPU scopes' runner. Verify 1 [ALREADY RUN — PASS (GO, 16/16, 0.7 s) on win-laptop]; Verify 2 [ALREADY RUN — PASS (clean GO, plant registry-order-unchecked red, 11.7 s) on win-laptop]; claim --changed --base 8484a25 [ALREADY RUN — PASS (GO, 539 passed, 9/9 scopes, 50 s) on win-laptop], logs/M0-T10.changed.log; its 2 oracle FLAGs name tests/cpu.sh and verify.json — the scope's own runner and manifest entry the Deliver's 'scope registry' implies. Declared (cheap to reverse): each K pair on-or-off (K's = 0 is a test scene's legal override, so Σ_N and m_tov/m_ch bounds apply only while on); m_nu < every nu_*; sanity classes (finite, >0, >=0) where the table says nothing; elements pinned to the ten. Not checked: c_sb > 2 max_gas_speed (needs calibration, G-CAL). Cargo.lock regenerated offline (sr-physics + serde_json, approved). Owed: testing.md rows (lot 2's docs block, R11); [NOT RUN — owed on linux-pc] scope registry. Next: M0-T11.
- Full block: plan_archive.md § M0-T10

## M0-T11 · The reaction registry · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T10)
- Status: DONE (2026-10-10 09:13) · archived by M0-V2, 2026-10-10
- Handoff (one line): win-laptop (Laser2025-20), model=claude-sonnet-5-5 level=high (rung_record). assets/reactions.json (§1.6.2's ten records, {version, records}) parsed and validated by sr-physics registry.rs: Reactions/Record/RateLaw, keys resolved against species and physics.json (A via a_coef key, ν a number or a key, T_k, Σ_g), shares summing to 1 within 1e-12 per side, ν ≥ 4, stand-in must be disabled, every refusal key records[<id>].<field>; Registry::load carries them. Verify 1 [ALREADY RUN — PASS (GO, 29/29) on win-laptop]; Verify 2 [ALREADY RUN — PASS (clean GO, both plants red: order 1 failed, shares 2 failed) on win-laptop]; claim --changed --base 8484a25 [ALREADY RUN — PASS (GO, 552 passed, 9/9 scopes, 52 s) on win-laptop], per-scope logs/M0-T11.<scope>.log; its 3 oracle FLAGs name tests/cpu.sh, M0-T10's plant and verify.json (this scope's entry: expected 16 → 29, reactions.json in paths). Declared (cheap to reverse; tasks/M0-T11.md): a_coef is a physics.json key; term ν number-or-key, checked ≥ 4 on every term; T_thr = t_thr_factor × first term's T_k, N_Fe = terms [] with t_thr_factor null; group one of the seven accumulators. Owed: testing.md registry row (lot 2 docs block, R11); [NOT RUN — owed on linux-pc] scope registry. Next: M0-T12.
- Full block: plan_archive.md § M0-T11

## M0-T12 · The equation of state · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T11)
- Status: DONE (2026-10-10 09:24) · archived by M0-V2, 2026-10-10
- Handoff (one line): eos.rs: Π_th=Σε_th, T=με_th, P(x;K₁,K₂), ColdTable (512 pts, 1e-8..1e8, log-log, built by quadrature, closed form is the oracle), ε_cold, ε_th from E floored at T_floor/μ with floor_added, c²=2Π/Σ, K=0 → none; scope eos 13/0/0 GO, plant eos-exponent RED (7 failed), claim --changed --base 8484a25 GO 564 passed 10/10 on win-laptop Laser2025-20. Deviation (declared, tasks/M0-T12.md): u's 1e-4 closeness to 2K₂x^{3/2} holds only where the blend has converged (shipped electrons at 1e8 are 4.6e-3 off), so the table's high end is tested with a K2≪K1 pair and the shipped pairs beyond the table; interpolation worst 3.55e-5. Owed: testing.md rows (eos, registry), eos on linux-pc. Ran model=claude-sonnet-5-5 level=high. Next: M0-T13
- Full block: plan_archive.md § M0-T12

## M0-T13 · The string table and its check (G-STR) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T12)
- Status: DONE (2026-10-10 09:32) · archived by M0-V2, 2026-10-10
- Handoff (one line): strings.rs: STRINGS = §4.1's 117 rows byte-identical (generated from the contract block) + get(key); the desktop title now reads get("app.title") (APP_TITLE gone). tests/strings.rs includes strings.rs by #[path] (sr-app is bin-only; the carried flag's first option) — 3 cases: same keys both ways, texts byte-identical, web.* verbatim in web/index.html. Scope strings 3/0/0 GO [ALREADY RUN — PASS on win-laptop Laser2025-20]; plant string-typo (stage.supernova: Supernova→Supernoba) RED 1 failed under --redarm; claim --changed --base 8484a25 GO 566 passed 11/11 scopes [ALREADY RUN — PASS on win-laptop Laser2025-20]. Deviations: tests/cpu.sh gained a '<crate> --test <file>' form (the scope's command); verify.json skip-class reason reworded (the contract leaves the class via the scope's paths). Owed: testing.md row (strings; the docs block, as eos/registry), strings on linux-pc. Ran model=claude-sonnet-5-5 level=high per rung_record. Next: M0-T14
- Full block: plan_archive.md § M0-T13

## M0-T14 · The licence gate (G-LIC) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T13)
- Status: DONE (2026-10-10 09:49) · archived by M0-V2, 2026-10-10
- Handoff (one line): tests/licences/check.py (stdlib): cargo metadata --locked --filter-platform <host|wasm32> over resolve.nodes, SPDX parser (OR/AND/WITH, '/'=OR, only 'Apache-2.0 WITH LLVM-exception' passes WITH), its fixed-expression checks run first in each case ('MIT AND GPL-3.0' refused); scope licences (cargo-native 164 pkgs, cargo-wasm32 143 pkgs) in verify.json; plant gpl-dep.patch. Verify 1 scope licences 2/0/0 GO [ALREADY RUN — PASS on win-laptop Laser2025-20]; Verify 2 --redarm licences GO, plant RED (_gpl-dep refused) [ALREADY RUN — PASS on win-laptop Laser2025-20]; claim --changed --base 8484a25 GO 567 passed 12/12 scopes [ALREADY RUN — PASS on win-laptop Laser2025-20]. ASKED + RULED: epaint_default_fonts 0.36.2 (eframe default_fonts; bundled fonts) is '(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0' — the lead: «Named exception (Recommended)» → exact-match EXCEPTIONS entry in check.py, override line in contract §0.5 [M0-T14] + mirror in Superseded. Deviations: a workspace member with no licence is counted not refused (publish=false; one declaring GPL is refused); the npm path ('check.py npm', walks npm ls + node_modules/*/package.json) is NOT PROVEN (synthetic fixture only, no web/package.json) — M0-T15 adds its scope case and must run it. Owed: testing.md row for licences (docs block, as eos/registry/strings); both cases on linux-pc. Ran model=claude-sonnet-5-5 level=high per rung_record. Next: M0-T15
- Full block: plan_archive.md § M0-T14

## M0-T15 · The web smoke (G-WEB, first cases) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T14)
- Status: DONE (2026-10-10 10:45) · archived by M0-V2, 2026-10-10
- Handoff (one line): Try 2 (rung_record: model=claude-opus-5-5 level=high; win-laptop Laser2025-20). The web smoke from try 1 stands (detail tasks/M0-T15.md); srAdapter reads « (BrowserWebGpu, Other, driver ) » — wgpu on the web names no GPU — and Chrome's own adapter nvidia/lovelace, fallback=false, so the GPU path, not software. Root cause of both NO-GO claims, measured: in a red-arm copy trunk runs the cargo binary itself, bypassing M0-D9's lock+touch wrapper, so a clean web arm reused the never-ready plant's artifacts from the shared redarm-target (scratch wasm lacked the "ready" string; reproduced 2/2, fixed 1/1). Fix: tests/cargo.sh's wrapper lifted into sr_in_shared_target (cargo() unchanged), check.sh runs trunk under it, testing.md says so; the 404 is /favicon.ico. Deviations: tests/cargo.sh is a shared helper (behaviour unchanged for cargo scopes); the lead committed T15 in a552c96 and ruled « dont re-run the whole 10 minute thing, just run the new commit and move on » — so --redarm web on the fixed tree was stopped twice, NOT RUN (the plant was RED in try 1's Verify 3; the fixed clean arm GO in a scratch copy only). Verify: verify.py web M0-T15-r3 GO 3/0/0 [ALREADY RUN — PASS on win-laptop]; claim py -3.12 tools/pb/verify.py --changed --base a552c96 --task M0-T15-r4 GO 563/0/0, 10/10 impacted, 21 s [ALREADY RUN — PASS on win-laptop] (web, licences not impacted since a552c96). Owed: --redarm web on the fixed tree, web and licences npm on linux-pc (flagged to M0-V2); five try-1 pb-redarm-* dirs remain in %TEMP% (disk 98 %), not deleted. Next: M0-T16.
- Full block: plan_archive.md § M0-T15

## M0-T16 · Labels watch, never drive — the static scan (G-WATCH) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T15)
- Status: DONE (2026-10-10 10:50) · archived by M0-V2, 2026-10-10
- Handoff (one line): Built tests/watch/check.py (stdlib): scans every file under crates/sr-engine/src/step/ + shaders/ for whole-identifier observe, Stage, Tracker and §2.11's 13 value keys (read from the contract's Schema line; version/measured/physics_hash left out); NO-GO names file:line, or on <2 files / <10 keys. Scope watch_only in verify.json (the 'oracle' FLAGs on it are the scope entry Deliver calls for), plant tests/plants/stage-in-step.patch. Verify 1 GO (2 files, 16 words); Verify 2 --redarm GO (plant NO-GO at step/mod.rs:9); claim run --changed --base 7251a24: GO 494 passed, 2/2 scopes [ALREADY RUN — PASS on win-laptop Laser2025-20]. Deviations: scope is watch_only not watch-only (verify.py refuses a hyphen; flagged to M0-T17, which writes the testing.md row); paths add milestones/m0/m0_contrat.md (the scan reads it). Detail tasks/M0-T16.md. Ran model=claude-sonnet-5-5 level=high. Next: M0-T17.
- Full block: plan_archive.md § M0-T16

## M0-T17 · Docs — physics.md (units, registries, equation of state) and lot 2's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T16)
- Status: DONE (2026-10-10 10:53) · archived by M0-V2, 2026-10-10
- Handoff (one line): docs/agent/physics.md (6 sections, no physics.json value) and testing.md's lot-2 table (registry, eos, strings, licences, watch_only; web's ready/no-webgpu already in T15's row). Deviation: row/scope is watch_only, not watch-only (M0-T16's flag; Verify 2 run with it, count 5; contract §5.4 still reads watch-only). Verify 1 = 6, Verify 2 = 5, claim --changed --base b9ce600 GO (493 passed, win-laptop Laser2025-20). Found, not acted on: testing.md's old « Scopes today » closing paragraph still says licences/strings 'can be armed as soon as code exists' (stale; V's to refresh). Ran on model=claude-sonnet-5-5 level=high. Next: M0-T18.
- Full block: plan_archive.md § M0-T17

## M0-V2 · Validation — lot 2: the registries, the equation of state and the guards · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T17)
- Status: DONE (2026-10-10 14:08)
- Carried flags: [M0-V1, 2026-10-09] owed on win-laptop (M0-D9; carried by M0-V1, which ran on linux-pc): tests/cargo.sh's red-arm cargo wrapper — py -3.12 tools/pb/verify.py --redarm adapter --task <id> and --redarm build, each three times in a row, read clean GO + plant red; Git Bash may lack flock, so the mkdir-lock path runs (linux-pc proved it only with flock forced off); else carried to the next V on win-laptop, M0-V17 at the latest (R15) · [M0-V1, 2026-10-09] owed on win-laptop (R15; M0-V1 ran lot 1 on linux-pc only — reports/v1.md § Owed): py -3.12 tools/pb/verify.py --all --task <id> with lot 1's seven scopes GO there — adapter listing the RTX 4080 Laptop and the Intel Arc (WARP where wgpu lists it), state on the RTX 4080, boot, desktop through game-offscreen (G-DESK UNVERIFIED, §6.6: NO-GO sends the window tasks to linux-pc, Q3 — M0-T5's flag), web (trunk there), capture with --offscreen-window — then --redarm of build, adapter, state, boot, desktop, web and capture; the human-run tier there: the lead runs start.bat (and start.ps1) once by double-click and says what they saw; read the free disk first (97 % full, Hazards); else carried V to V, M0-V17 at the latest · [M0-D13, 2026-10-10] M0-D13 measured on win-laptop only: on linux-pc run verify.py boot (summary.json verbatim in its log: version, adapter, steps 200) and --redarm boot (scene-refuse red) — owed by R15 · [M0-T15, 2026-10-10] Owed by M0-T15: py -3.12 tools/pb/verify.py --redarm web on win-laptop — the clean arm after the sr_in_shared_target fix (tests/cargo.sh, check.sh) never ran to its end in the real harness (the lead waived it, 2026-10-10); plus verify.py web and licences (npm) on linux-pc.
- Read: milestones/m0/m0_contrat.md §1.5, §1.6, §2.1, §2.3, §2.5.1, §2.7, §4.1, §5.4 (G-STR, G-LIC,
  G-WATCH, G-WEB), §6.2 + the handoffs of M0-T10–M0-T17 + docs/agent/physics.md, testing.md + the
  delivered files, each by the section it covers
- Deliver: contract-vs-code on lot 2 — assets/*.json against §1.5, §1.6.2, §2.5.1 and §2.7 value by value
  and bound by bound, the EOS against §2.3, the string table both ways, the licence tree, the scan, the
  web smoke (its adapter named), each defect its own D (E4), every scope the lot added red-armed, the
  verdict, its blind spot, its box, NOT PROVEN where only synthetic evidence exists → reports/v2.md →
  Verify 1 · Phase 4's close (stubs, the header budget, the complete loop and its seconds,
  `rung_record.py report`'s table in reports/v2.md, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V2` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: a registry that matches the contract's table but not its bounds — the V checks each
  bound's edge case, not only the initial values.
- Handoff: win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now). Verdict DEFECTS(1), reports/v2.md: lot 2 conforms to §1.5, §1.6.2, §2.3, §2.5.1, §2.7, §4.1 and §5.4 value by value and bound by bound (27/27 registry mutants killed); Verify 1 py -3.12 tools/pb/verify.py --all --task M0-V2 [ALREADY RUN — PASS (GO, 586 passed, 15/15 scopes, 134 s wall, ≈ 113 s normalised by load) on win-laptop]; lot 2's six scopes and lot 1's seven red-armed here, 16/16 plants red (desktop 746 s, web 752 s raw — over 600 s). Filed: M0-D17 (smoke.mjs runs Chrome with --no-sandbox, playwright-core's default — chrome://version measured), M0-TM2 (the record moves BUILD on Sonnet 5.5, high: $1.70 vs $1.44 per DONE block, 10 blocks each). The lead: «Yes, rename once (Recommended)» → contract §0.5 [M0-V2], Superseded, R17 (declared: R17 reads all 30 hyphenated scope names in the plan, not only §5's 8). Carried flags discharged: M0-D9's wrapper ×3 (adapter, build; mkdir lock), lot 1 on win-laptop with the lead's start.bat «Opened, stayed till closed» (start.ps1's double-click opened Notepad — Windows' default; its off-screen run works), --redarm web on the fixed tree; owed on linux-pc → M0-V3 (lot 2, M0-T15's web/npm, M0-D13's boot). Not filed (E1 (i), § Observations): T84's stand-in readout, Chrome ignoring powerPreference on Windows, start.ps1 without a case, G-LIC's workspace-member reading without a §0.5 line, red-arm cost here. Phase 4: T10–T17 stubbed, header 418/600, flags → M0-T27, M0-T32; Phase 3's 8 late blocks a TC's. Next: M0-D17.

## M0-D17 · web/smoke.mjs runs Chrome with `--no-sandbox`: playwright-core adds it by default, against §6.2.3 · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V2)
- Status: DONE (2026-10-10 14:22)
- Blocks: nothing — G-WEB reaches "ready" on the GPU path today (nvidia, fallback=false); the gap is the browser it proves it in
- Caused by: M0-T15
- Files: web/smoke.mjs (its `chromium.launch({ executablePath: chrome, headless: true, args: chromeArgs })`), tools/pb/verify.json (the `web` scope's plants), tests/plants/ — opened by M0-V2, not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §6.2.3, §5.4 (G-WEB) + docs/agent/testing.md (the `web` row) + milestones/m0/logs/M0-V2.log (« probe: Chrome's command line ») + milestones/m0/tasks/M0-V2.md (the probe's script)
- Symptom: measured on win-laptop (Laser2025-20), Chrome 154.0.8037.99, playwright-core 1.64.0, 2026-10-10 (M0-V2): Chrome launched with smoke.mjs's exact options (executable, `--headless=new --enable-unsafe-webgpu`, headless) shows `--no-sandbox` in chrome://version's Command Line; the same launch with `chromiumSandbox: true` shows it absent. §6.2.3 leaves `--no-sandbox` out of Chrome's guidance and smoke.mjs's own header says « never `--no-sandbox` »: the smoke proves WebGPU in an unsandboxed browser, not the one a player runs. Repro: the probe in M0-V2's task file, or chrome://version from a page smoke.mjs opens. Cause measured: playwright-core's `chromiumSandbox` defaults to false, which appends `--no-sandbox` to every Chromium launch.
- Deliver: smoke.mjs launches with `chromiumSandbox: true` and refuses its own run when the launched Chrome's command line holds `--no-sandbox` (NO-GO naming it; the command line's sandbox state in both cases' logs) · plant `smoke-no-sandbox` (tests/plants/, the option dropped) in the `web` scope · testing.md's `web` row says so. If `ready` cannot reach "ready" with the sandbox on (WebGPU in a sandboxed headless Chrome is UNVERIFIED, §6.2.3), stop: BLOCKED with the measured failure and a question to the lead — never the flag put back.
- Done when: on win-laptop, `py -3.12 tools/pb/verify.py web --task M0-D17` GO with the log reading the sandbox on, and `py -3.12 tools/pb/verify.py --redarm web --task M0-D17` GO with `smoke-no-sandbox` red — 752 s at M0-V2 on a loaded box, over the 10-minute line: the lead's run in one visible terminal, or normalised by measured load (the lead, 2026-10-10: « you can normalize by load if you have it »); linux-pc owed (R15).
- Handoff: win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record now, the heading's rung). smoke.mjs launches Chrome with chromiumSandbox: true and both cases read chrome://version's #command_line: 'sandbox: on' in the log, NO-GO naming --no-sandbox otherwise (an unreadable command line is NO-GO too); plant smoke-no-sandbox (the option dropped) in the web scope; testing.md's web row (guard, plant, red-arm cost) and verify.json's desc say so. web GO 3/0/0 (ready 0.8 s, SR-CHROME-ADAPTER nvidia lovelace fallback=false: WebGPU holds in a sandboxed headless Chrome here, the BLOCKED arm did not fire); --redarm web GO 4/4 red, smoke-no-sandbox red on both cases, 515 s wall, 6.8 % external load, about 480 s normalised. Claim run verify.py --changed --base 2ada210 --task M0-D17: GO, 499 passed, 2/2 scopes (its web red-arm 4/4 red, 600.00 s, not load-measured); its two oracle FLAGs (the plant, verify.json) are the plant and scope entry Deliver names - declared, not a deviation. Owed on linux-pc (R15): web and its red-arm with the sandbox on - flagged to M0-V3. Detail: tasks/M0-D17.md. Next: M0-TM2.

## M0-TM2 · Rating pass — the record moved BUILD on Sonnet 5.5, high (M0-V2's table) · **PLAN** · Opus 5.5, max · switch · (AFTER M0-D17)
- Status: DONE (2026-10-10 14:52, started 14:43)
- Read: this file (rules + this task) + PLAYBOOK.md §0 (the ladder, the rubric) and §B.3 (the probe) + the agent's model
  pages the ladder cites + the record (`rung_record.py report`; milestones/m0/reports/v2.md § Phase 4's close)
- Deliver: the ladder re-read on the day — the probe and the model pages: a model retired, renamed or added, a price or a
  level changed → the `Models:` line rewritten and dated, the lead's answer first where a rung they run changes (§12) ·
  every open block's rating re-derived by §0's rubric from its heading, `Sizing:`, `Deliver:`, `Verify:` and `Carried
  flags:` and the record — rule (4): BUILD on Sonnet 5.5, high at $1.70 per DONE block against $1.44 for BUILD on Opus
  5.5, high (`usual`), 10 blocks each, costs read from win-laptop's session folder only (28 of 49 DONE blocks had none
  readable, linux-pc's) — its rating segment rewritten, nothing else in the block · the register's `Model ratings:
  <YYYY-MM-DD> by M0-TM2` line · in the handoff, the ratings changed, counted by rung.
- Pass: `plan.py lint` GO; every open block on a rung of the ladder, each above the cheapest rung that fits named in the
  task file with its rule; the diff touches rating segments, the `Models:` line and the register's line only. Fail: any
  other byte changed; a BUILD or MOVE block above the cheapest rung that fits with none of §0's rules (1)–(4) named; a
  CHECK or PLAN block below `(gate)`.
- Handoff: win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Ladder re-read 2026-10-10: the claude-api skill's table in Claude Code 2.1.295's bundle (cached 2026-10-06, the copy M0-TM1 read) - no rung retired, renamed, repriced or relevelled (Opus 5.5 $4/$20, Sonnet 5.5 $2/$10) → Models line re-dated, rungs unchanged, no question. Record today (rung_record report): BUILD on Sonnet 5.5, high 10 blocks at $1.85 per DONE block (M0-V2's $1.70 plus M0-D17, traced to M0-T15) against (usual) 11 blocks at $1.45 → rule (4) moves the class one rung, to (usual). 128 open blocks re-derived, 89 changed - Opus 5.5 max 20 (0) · Opus 5.5 high 101 (+89: 89 by rule (4), 12 by (3)'s list → (5)) · Sonnet 5.5 high 0 (−89) · Sonnet 5.5 medium 7 (0: E, their class n=1); no trail (rule (1)); every heading keeps · switch; the diff is the 89 rating segments and the Models line (checked against a post-status snapshot). Not acted on: the record's rows differ in kind ((usual) = 10 D fixes, Sonnet = 9 feature T's + TE-win; medians $1.12 vs $1.42; T15's escalation $6.24) and linux-pc's 28 blocks are unread - observations, rule (4) applied as written; Next task still reads M0-TM2 (outside TM2's diff; next is M0-T18); Repo facts name Claude Code 2.1.294, the probe reads 2.1.295. lint GO (496), plan_lint GO, its red-arm GO; the claim run verify.py --changed --base aa2e690 --task M0-TM2 is the last step, after this close. Detail: tasks/M0-TM2.md. Next: M0-T18.

> **Commit gate (lead):** Phase 4 closes after M0-V2 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 4: registries, equation of state and guards"`
> `git push`

# Phase 5 — Lot 3 · the step framework: booking, floors, Δt, the box, the latch, time

## M0-T18 · Booking — the side buffers and accumulators every pass books into · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V2)
- Status: DONE (2026-10-10 15:02)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.4, §1.9.1 (the frame
  accumulators), §2.2 (the last paragraph), §2.9 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/state.rs: §2.2's per-cell side buffers (φ, the heat flux F, the
  neutrino source S_ν) and one booking layout every pass writes its §2.9 terms into — §1.9.1's frame
  accumulators (radiated, nuclear by group H/He/C/Ne/O/Si/N, neutrino lost and deposited) and the booked
  terms (vacuum_reset, floor_added, escaped per mass, momentum, energy and species, swallowed, the tools)
  — f32, no atomics on a value the state depends on (§1.3.4), cleared on request, read back and summed in
  f64 in fixed order by a helper the ledger (M0-T57) reuses → Verify 1 · tests/gpu/booking.rs (new): a
  test pipeline books known per-cell values over 16 steps → each term's f64 sum equals the expected
  total to 10⁻⁹ relative, two runs bit-identical, the clear empties every term → scope `booking` →
  Verify 1 · plant tests/plants/booking-row-skip.patch (new: the f64 helper skips the last row) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py booking --task M0-T18` · Pass: GO, ≥ 3 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm booking --task M0-T18` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a layout too small for a term a later pass needs forces every pass block to reopen this
  one — the layout lists every §2.9 term and §1.9.1 accumulator now, with its writer pass named.
- Handoff: state.rs: SideFields (side.phi 1, side.flux 2 F_x/F_y, side.s_nu 1 planes), TERMS (37: every §2.9 booked term + the 10 §1.9.1 accumulators, writer pass named), BOOK_GROUPS (8 buffers, one writer each, const-asserted ≤128 MiB at 2048²), Booking (clear, readback), book_totals (f64, term→row→slot); State::readback now shares read_buffers. Scope booking 4 cases + plant booking-row-skip, testing.md lot-3 row.
  Declared defaults: escaped booked per boundary face (Domain::EdgeFaces, 2(w+h) slots), not per cell; slots hold one frame (reader reads then clears; the ledger keeps the f64 cumulative); P5–P7 add straight into their accumulators (P9 needs no dispatch); swallowed is energy only (§2.9).
  Measured, not acted on → flagged to M0-T57: floor_added's one writer is P8 (a P4 floor write-back needs its own term); book_totals sums full planes (~22 MB/readback at 600×400), so §2.9's per-frame path needs a GPU fixed-order partial reduction first.
  Runs, win-laptop Laser2025-20, RTX 4080 Laptop: booking GO 4/0/0 (worst rel. error 0, tol 1e-9); --redarm booking GO (plant RED); claim --changed --base 2f176dd in tasks/M0-T18.md. linux-pc owed (R15).
  Model: claude-opus-5-5, level high (rung_record.py). Detail: tasks/M0-T18.md. Next: the plan's next TODO block.

## M0-T19 · P8 floors, complete, and the equation of state on the GPU · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T18)
- Status: DONE (2026-10-10 15:25, started 15:10)
- Carried flags: [M0-T3, 2026-10-09] from M0-T3: P8's list holds renormalise_species (floors.wgsl) — put the vacuum reset and the temperature floor AHEAD of it in Step's P8 list; declared default, contract silent: a cell whose fractions are all ≤ 0 becomes pure hydrogen — state it in §2.7 by asking, and match it in the CPU f64 twin, which R13 wants and M0-T3's Deliver did not name (its test carries an inline f64 oracle); NaN fractions are left to P9's guard
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P8), §2.3.2–§2.3.3, §2.7
  (sigma_floor, sigma_vac, t_floor), §2.9 (vacuum_reset, floor_added) + docs/agent/testing.md
- Deliver: crates/sr-engine/shaders/eos.wgsl (new): μ, Y_e and Z_met from the species table, the cold
  pressure and ε_cold from M0-T12's u(x) table uploaded once, c² — the helpers every later shader shares
  → Verify 1 · crates/sr-engine/shaders/floors.wgsl and src/step/mod.rs: P8 complete — cells below Σ_vac
  reset to the floor state (Σ_floor, u = 0, T = T_floor), ε_th floored at T_floor/μ, species
  renormalised, the differences booked `vacuum_reset` (mass) and `floor_added` (energy) (M0-T18) →
  Verify 1 · tests/gpu/floors.rs (new): a 64 × 64 field of vacuum, cold, hot and unnormalised cells →
  one step → each rule within f32 rounding of M0-T12's CPU EOS, the booked terms equal the mass and energy
  changed within 10⁻⁶ relative → scope `floors` → Verify 1 · plant tests/plants/floors-no-cold-energy.patch
  (new: ε_th computed without ε_cold) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py floors --task M0-T19` · Pass: GO, ≥ 5 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm floors --task M0-T19` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a floor that books the mass it removes but not the energy it adds lets G-CONS drift for a
  reason no physics scope finds — both terms are cases.
- Handoff: eos.wgsl (new; group-1 EosParams + per-pair ln u and f64 segment slopes, uploaded once by step::EosGpu; composition, cold_pressure, cold_u, ε_cold, Π, c², T, floors) · floors.wgsl: floor_cells (vacuum reset → Σ_floor, u = 0, T_floor; else T floor; books vacuum_reset + floor_added) ahead of renormalise_species, both on one renormalised() · Step::new(device, state, booking, eos), Dispatch with labels and bind groups · sr-engine → sr-physics path dep (Cargo.lock +1 line) · headless builds Booking + EosGpu · tests/gpu/floors.rs (6 cases, inline f64 twin) · plant floors-no-cold-energy · scope floors · contract §2.7 [M0-T19] (the lead's yes: all ≤ 0 → pure H, μ of the renormalised fractions, reset energy booked floor_added) · testing.md row.
  Deviations: state test field made hot non-vacuum (its all-negative Σ would now reset), P8 count 2, case renamed; state paths + sr-physics/assets; >6 files by forced one-liners. Claim's oracle FLAGs: floors.rs, main.rs, state.rs, verify.json — those.
  Measured: first floors run red (ε_cold above the table 5.3e-5 > 2e-5: f32 slope ln u[511]−ln u[510]); fixed by f64 slopes, tolerance unchanged; worst ε_cold 5.1e-6. Not measured: NaN fractions masked as pure H by P8 (flagged M0-T20); sun_disk_planes' E lacks ε_cold so P8 floors part of it (tasks file).
  [ALREADY RUN — PASS on win-laptop Laser2025-20, RTX 4080 Laptop Vulkan]: floors FAIL 5/1 then GO 6/0/0; --redarm floors GO (plant RED); state GO 6/0/0; --redarm state GO (plant RED); claim --changed --base a7c38c2 GO 579/0/0, 15/15 scopes. linux-pc owed (R15).
  Flags: M0-T37 (move p8_twin to reference/floors.rs, match §2.7 [M0-T19]), M0-T20 (NaN). Ran on model=claude-opus-5-5 level=high. Detail: tasks/M0-T19.md. Next: M0-T20.

## M0-T20 · Δt and the non-finite guard (P1, P9) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T19)
- Status: DONE (2026-10-10 15:46)
- Carried flags: [M0-T19, 2026-10-10] Source only, not measured: P8 (before P9) can mask a NaN species fraction — renormalised() in floors.wgsl turns a NaN sum into pure hydrogen (sum > 0 is false), so P9's non-finite guard never sees it; a NaN Σ or E passes P8 untouched. Decide where the guard reads the fractions (M0-T19)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P1, P9), §1.3.4, §2.3.3,
  §2.3.5, §3.3 (exit 5) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/dt.rs and shaders/reduce.wgsl (new): P9's Δt_{n+1} = min(C·min[(|u|
  + c)/Δx + (|v| + c)/Δy]⁻¹, η_g·min √(Δx/|g|)) by a fixed-order tree reduction (§1.3.4, the gravity
  term infinite until P2 writes g), P1 using step n−1's value, step 0 reducing the initial state first,
  every 64 steps the non-finite guard — a NaN or Inf in any channel stops the run, the binary exiting 5
  with a message naming step, cell and channel (§3.3) → Verify 1 · tests/gpu/dt.rs (new): Δt against the
  formula on fields of known u and c (f64 within 10⁻⁶), bit-identical over two runs, a NaN planted at a
  known cell and step → the guard's step, cell and channel → scope `dt` → Verify 1 · plant
  tests/plants/dt-cfl-one.patch (new: C = 1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py dt --task M0-T20` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm dt --task M0-T20` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a reduction whose order depends on workgroup scheduling breaks G-WARP's bit-identity later
  — the two-run case on a 600 × 400 field.
- Handoff: win-laptop (Laser2025-20, RTX 4080 Laptop), model=claude-opus-5-5 level=high (rung_record). Built: reduce.wgsl + step/dt.rs — P9's Δt by a fixed-order tree (C, η_g from physics.json; gravity term infinite until P2), P1 dt_advance on step n−1's value, step 0 preceded by the initial state's reduction, the guard every 64th step (index ≡ 0 mod 64; non-finite told by bits; lowest cell, then channel) → headless exit 5 naming step, cell, channel (exit-5 path NOT PROVEN (source only): no scene can plant a NaN yet). Step::new now takes &Physics; Step::read_dt, dt_record (Δt_n = word 0) for later passes. Vacuum cells count as P8's floor state in Δt; whole world until M0-T21's box. Deviation: 9 code files / ≈680 added lines over the 6/600 ceiling — state.rs, floors.rs, state test edits are the new signature and the P1/P9 dispatches (the FLAG oracle lines). Carried flag decided: guard reads all 14 channels after P8; measured P8 zeroes a NaN fraction (logs/M0-T20.probe.log) → filed M0-D18. sim_time still 0 (Δt not summed — not this Deliver). verify.py dt [ALREADY RUN — PASS (5/0/0) on win-laptop]; --redarm dt [ALREADY RUN — PASS (dt-cfl-one RED) on win-laptop]; --changed --base 2ee163f [ALREADY RUN — PASS (538 passed, 12/12 scopes GO) on win-laptop]; linux-pc owed (R15). Detail: tasks/M0-T20.md. Next: M0-D18.

## M0-D18 · P8 zeroes a NaN species fraction before P9's guard reads it: the run goes on with the cell renormalised · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T20)
- Status: DONE (2026-10-10 15:53)
- Caused by: M0-T19
- Files: crates/sr-engine/shaders/floors.wgsl (`renormalised`, lines 34–59), crates/sr-engine/shaders/reduce.wgsl (`non_finite`, `guard_scan`), crates/sr-engine/tests/gpu/dt.rs — opened by M0-T20, floors.wgsl not changed
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P8, P9), §2.7 [M0-T19], §3.3 (exit 5) + milestones/m0/logs/M0-T20.probe.log
- Symptom: measured on win-laptop (Laser2025-20, RTX 4080 Laptop, Vulkan), 2026-10-10, M0-T20's probe (logs/M0-T20.probe.log; a temporary case in tests/gpu/dt.rs, removed): a NaN planted in x_C at (20, 10) before step 0 → after one step the guard (step 0's scan, P9) finds nothing and the cell's fractions read [0.2375, 0.0399, 0.0, 0.0955, …], x_C silently 0 and the rest renormalised; an Inf planted the same way reads x_C = NaN after P8 and the guard names it (step 0, (20, 10), x_C). Repro: plant `f32::NAN` in a species plane of tests/gpu/dt.rs's `field`, run one step, read `Step::read_dt`. Cause (source, consistent with the measure): `renormalised` clamps with `max(x, 0.0)`, which returns 0 for a NaN on this adapter, before the sum — so §3.3's "a non-finite value in the state" upstream of P8 (a P4 or P6 bug writing a NaN fraction) never stops the run. Σ, momentum and energy NaNs pass P8 untouched and are caught (tests/gpu/dt.rs).
- Deliver: floors.wgsl — a cell holding any non-finite fraction (told by its bits, as reduce.wgsl's `non_finite`, never by a float comparison WGSL may fold) is left for P9's guard — its fractions written back as read, not renormalised, the floors kept as they are, no expectation loosened → Verify 1, 3 · a case in tests/gpu/floors.rs (a NaN fraction still non-finite after P8) → Verify 1 · a case in tests/gpu/dt.rs (a NaN x_C planted, the guard naming its step, cell and x_C) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py floors --task M0-D18` · Pass: GO, 7 cases · Fail: NO-GO · 2. `python3 tools/pb/verify.py dt --task M0-D18` · Pass: GO, 6 cases · Fail: NO-GO · 3. `python3 tools/pb/verify.py --redarm floors --task M0-D18` · Pass: GO — floors-no-cold-energy NO-GO · Fail: the plant stays GO
- Handoff: win-laptop (Laser2025-20, RTX 4080 Laptop), model=claude-opus-5-5 level=high (rung_record now, the heading's rung). floors.wgsl: non_finite (bits, reduce.wgsl's test) + fractions_non_finite; floor_cells and renormalise_species return at once for a cell holding an Inf/NaN fraction - fractions as read, Σ/u/E untouched, nothing booked; finite cells' rules unchanged. Deviation (declared, my reading of 'the floors kept as they are'): floor_cells skips the cell too, else a vacuum cell would get E = NaN and the guard would name energy, not the fraction. verify.json: floors expected 7, dt 6 (the Pass counts). New cases: floors a_cell_with_a_non_finite_fraction_is_left_for_the_guard (NaN/+Inf/-Inf in 5 kinds of cell), dt the_guard_names_a_nan_fraction_that_p8_would_renormalise_away (M0-T20's probe, now step 0, (20, 10), x_C); both shown red with the check stubbed out (logs M0-D18-red.*). floors [ALREADY RUN — PASS (7/0/0) on win-laptop]; dt [ALREADY RUN — PASS (6/0/0) on win-laptop]; --redarm floors [ALREADY RUN — PASS (floors-no-cold-energy RED) on win-laptop]; claim --changed --base 5206293 runs after this close, result in tasks/M0-D18.md. Not acted on: Twin's dead-code warning (M0-T19, cosmetic). linux-pc owed (R15). Detail: tasks/M0-D18.md. Next: M0-T21.

## M0-T21 · The active box · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T20)
- Status: DONE (2026-10-10 16:12)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (the re-fit), §1.3.3, §1.3.4,
  §5.3 (G-BOX) + milestones/m0/reports/time_warp.md R1 (A2) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/boxfit.rs (new) with its reduction in shaders/reduce.wgsl: the
  bounding box of non-vacuum cells (Σ ≥ Σ_vac) grown by 8 cells, rounded out to multiples of 8, clamped to
  the world, re-fit at step indices ≡ 0 (mod 16) after P9 and when P0 lands an edit outside it, written on
  the GPU as every pass's indirect dispatch sizes — never read back to decide a dispatch — and as the
  FFT size per axis (the smallest power of two ≥ 2 × the box, at least 32), a whole-world switch for
  tests → Verify 1 · tests/gpu/box.rs (new): blobs at known places, edges and corners → the box exactly,
  the re-fit's step indices, the dispatch sizes the passes read → scope `box` (G-BOX's, its conservation
  cases come with M0-T39) → Verify 1 · plant tests/plants/margin-zero.patch (new, §5.3: the margin 8 → 0)
  → Verify 2
- Verify: 1. `python3 tools/pb/verify.py box --task M0-T21` · Pass: GO, ≥ 5 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm box --task M0-T21` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a box read back to the CPU to size dispatches costs a frame of lag and breaks the latch's
  in-frame stop — the sizes are written and read on the GPU only (TW-E16).
- Handoff: On win-laptop Laser2025-20 (RTX 4080 Laptop): step/boxfit.rs + reduce.wgsl's box_cells/box_fit/box_gate write the box record, the passes' indirect sizes and the FFT size per axis on the GPU, never read back; P8's two dispatches and dt_cells run indirect over the box; re-fit before step 0, after P9 at n ≡ 0 (mod 16), and in P0 when Step::box_edit_flag is set; Step::with_box(…, BoxMode::WholeWorld) is the switch. Scope box GO 6/6, --redarm GO (margin-zero red, 5 of 6 failed); claim: --changed --base 103a236 (logs/M0-T21.*). Deviations: 7 code files, ≈ 720 new code lines (P8/P9 moved onto the box's sizes in the same thread); defaults — empty box → zero-sized dispatches and Δt = the largest f32; the re-fit's scan and the guard read the whole world; vacuum outside the box is never floored. Not acted on: clippy --tests red on 4 pre-existing lints in tests/gpu/floors.rs (no scope runs it). Model claude-opus-5-5, level high. Detail: tasks/M0-T21.md. Next: M0-T22.

## M0-T22 · Frames of steps and the collapse latch · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T21)
- Status: TODO
- Carried flags: [M0-T21, 2026-10-10] Box-sized dispatches (Step::box_sized: floor_cells, renormalise_species, dt_cells) take their counts from box.args on the GPU; P1, dt_partials, the guard and the box's own are Fixed — a latch that zeroes a frame's dispatches must reach both kinds (M0-T21).
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.3, §1.8.5, §1.8.7, §3.1
  (encode_frame, poll), §5.3 (G-LATCH) + milestones/m0/reports/time_warp.md R4 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/latch.rs and shaders/latch.wgsl (new): `encode_frame(plan,
  encoder)` — a frame's N steps as indirect dispatches in one submission, the latch, a u32 flag a pass
  sets with atomicOr (P6 and P3 later), armed when not set during the last 1.0 t.u. of sim time, with an
  armed latch set and the slow-down on, a controller dispatch zeroes every later indirect dispatch of the
  frame — the box re-fit's included — so no later step runs (TW-E16), `poll()` reporting steps run, sim
  time, Δt and the latch fired, never blocking (§3.1) → Verify 1 · tests/gpu/latch.rs (new): a test hook
  setting the flag at step k of a 40-step frame → exactly k steps run and the state equals a k-step run,
  bit-identical, slow-down off → all 40, re-armed only after 1.0 t.u. → scope `latch` (G-LATCH's — its
  contract scenario arrives with M0-T111, until then NOT PROVEN (synthetic)) → Verify 1 · plant
  tests/plants/latch-off.patch (new, §5.3: the controller never zeroes the dispatches) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py latch --task M0-T22` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm latch --task M0-T22` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a box re-fit inside the frame restoring the zeroed sizes lets steps run past the event —
  the test fires the latch at step 15 so that step 16's re-fit falls inside the stopped stretch.
- Handoff: <placeholder>

## M0-T23 · GPU timestamps, cost per step and `--timing` · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T22)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.3 (c_step), §2.12.2 (timing),
  §3.1 (poll), §3.3 (`--timing`), §6.2.2 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/mod.rs: timestamp writes at compute-pass boundaries where the
  adapter offers `timestamp-query` (a WebGPU feature, not native-only, §6.2.2), wall time otherwise and
  named, cost per step and per pass in `poll()`'s frame report (c_step's input) → Verify 1 ·
  crates/sr-engine/src/headless.rs: `--timing` writing summary.json's `timing` {per_pass_ms,
  step_ms_mean} → Verify 1 · tests/gpu/timing.rs (new): every pass in the sequence timed, positive, their
  sum within 40 % of the step's (TW's submission-overhead note), the source named, and a `headless
  --timing` run of the nominal disk writing summary.json's `timing` with every pass in per_pass_ms
  [M0-TB] → scope `timing` → Verify 1 · plant tests/plants/timing-skip-pass.patch (new: one pass missing
  from per_pass_ms) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py timing --task M0-T23` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm timing --task M0-T23` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: timestamps inside passes are native-only (wgpu's TIMESTAMP_QUERY_INSIDE_PASSES) — a build
  that uses them breaks in the browser; boundaries only.
- Handoff: <placeholder>

## M0-T24 · Time control — rungs, the frame plan, the cap, the slow-down, the 30-frames switch · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T23)
- Status: TODO
- Carried flags: [M0-T21, 2026-10-10] An all-vacuum world has an empty box: Δt_{n+1} is the largest finite f32 (3.4e38), not the vacuum floor's — Δt_est, N and the sandbox clock must survive it (M0-T21 default, cheap to reverse in reduce.wgsl dt_partials).
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8, §3.2, §4.3, §4.12 (speeds) +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/time.rs (new): `TimeControl` per §3.2 — §1.8.2's ladder (TOP from
  calibration's top_rung once it exists, M0-T70, until then the nine ladder rungs), select, faster,
  slower, toggle_pause, step_once (paused: exactly one step, running: it pauses), plan_frame (owed,
  N = min(N_max, ⌈owed/Δt_est⌉), N_max from c_step's 16-frame average and R_reserve 3.0, §1.8.3),
  on_report (owed −= advanced, owed ≥ −Δt_est), the cap readout (N_max binding in ≥ 30 of the last 60
  frames, §1.8.4), on_event (ignition → ×1, core collapse and supernova → ×0.1, only ever slower, no
  auto-return, §1.8.5), the 30-frames switch at TOP (§1.8.6), speed_text → Verify 1 · its tests, no GPU:
  one per §1.8 rule at its edge (29 against 30 frames of 60, 119 against 120 quiet frames, a slow-down
  never raising the rung, the game starting at ×1, running) → scope `time-control` → Verify 1 · plant
  tests/plants/slowdown-auto-return.patch (new: the rung returns after the banner) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py time-control --task M0-T24` · Pass: GO, ≥ 14 cases · Fail: NO-GO
  or fewer · 2. `python3 tools/pb/verify.py --redarm time-control --task M0-T24` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: owed time carried as a float that grows without bound while capped breaks slow motion
  after a long run — owed is bounded below by −Δt_est and the cap case runs 10,000 frames.
- Handoff: <placeholder>

## M0-T25 · Scene files · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T24)
- Status: TODO
- Carried flags: [M0-T4, 2026-10-09] [M0-T4, 2026-10-09] scene files plug into check_scene / run in crates/sr-engine/src/headless.rs (a file path is refused with exit 4 today; preset:massive and preset:giant load the nominal Sun-like disk with an SR-WARN until calibration gives preset_mass_sb; the disk builder sun_disk_planes is there to move into the scene loader)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.8, §2.10 (the cloud's shape),
  §2.12.1, §2.12.4 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/scene.rs (new): §2.12.1 parsed — unknown keys refused (exit 4), `t` or
  `pressure` but not both, `world`, `edge`, `slowdown`, `switches`, `test`, `overrides` (physics.json keys
  only, a run with overrides never writes calibration), objects `disk` (Σc√(1 − r²/a²)), `rect`, and
  `preset` (the nominal Sun-like disk until the presets, M0-T74), coordinates per §2.12.4 → Verify 1 ·
  crates/sr-engine/src/headless.rs: `--scene <file>`, `--world`, `--edge` read through it → Verify 1 ·
  scenes/sun_disk.json and scenes/fill_world.json (new: the nominal Sun-like disk, a thin field filling
  the world — the box and timing runs use them) and tests/gpu/scene.rs (new): the refusals (unknown key,
  both t and pressure, a bad override key), a disk's mass against 2πΣc a²/3 within the sampling error it
  states, every file in scenes/ parsing, and `headless --scene scenes/sun_disk.json --world 256x256
  --edge bounce --steps 5` writing summary.json's `scene` and `world` as given [M0-TB] → scope `scene`
  (paths include scenes/**) → Verify 1 · plant tests/plants/scene-unknown-key.patch (new: unknown keys
  accepted) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py scene --task M0-T25` · Pass: GO, ≥ 6 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm scene --task M0-T25` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a scene with a misspelt switch (`"hydor": false`) silently running hydro — unknown keys
  at every level refused, not only at the top.
- Handoff: <placeholder>

## M0-T26 · State dumps · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T25)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.12.3, §3.1 (load_dump,
  dump_state), §3.3 (`--dump-every`, `--load-dump`) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/dump.rs (new): SRDUMP01 written and read — the magic, the header length,
  the JSON header (version, width, height, step, sim_time, channels in §2.2's order, sinks), f32 planes
  little-endian and row-major, `load_dump` and `dump_state` (§3.1) → Verify 1 · headless.rs:
  `--dump-every N`, the final dump always, `--load-dump FILE` → Verify 1 · tests/gpu/dump.rs (new): write
  → read → a bit-identical state, 2 × 100 steps through a dump equal 200 straight steps, bit-identical
  (§1.3.4), a truncated file and a wrong magic refused → scope `dump` → Verify 1 · plant
  tests/plants/dump-channel-order.patch (new: two channels swapped on write) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py dump --task M0-T26` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm dump --task M0-T26` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a resumed run that restores the cells but not the latch's arming time or the step index
  diverges from the straight run — the 2 × 100 case compares the full state and the reports.
- Handoff: <placeholder>

## M0-T27 · Docs — architecture.md (the step, the box, the latch, the files) and time_control.md · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T26)
- Status: TODO
- Carried flags: [M0-V2, 2026-10-10] testing.md beside your lot-3 rows, stale since lot 2 (M0-V2, reports/v2.md): (a) the closing paragraph's « licences and strings … can be armed as soon as code exists » — both armed by lot 2; « §5 names 38 » counts rows (38 rows, 32 scope names); (b) the complete-loop line is M0-V1's linux-pc run — add M0-V2's win-laptop loop (reports/v2.md § The complete loop); (c) the build row's « ≈ 5 s on win-laptop (the empty sr-physics) », and the desktop and capture rows' « win-laptop route owed there » — measured by M0-V2 on a loaded box: --redarm build 300 s cold, 53–60 s warm; boot 431 s; capture 526 s; desktop 746 s (≈ 688 s normalised by load); web 752 s; (d) the watch_only row's « not the contract's watch-only » — the contract reads watch_only now (§0.5 [M0-V2], R17); (e) running.md's « The two ways a person starts the game (the lead, double-clicking) » lists start.ps1, which Windows does not run on a double-click (Explorer offers Notepad — the lead, M0-V2): start.bat is the double-click, start.ps1 runs from PowerShell (its command)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3, §1.8, §2.9, §2.12, §7 + the
  handoffs of M0-T18–M0-T26
- Deliver: docs/agent/architecture.md: the step's sequence and dispatch lists, the booking layout, the
  box, the frame and the latch, the files (scene, dump, summary) → Verify 1 · docs/agent/time_control.md
  (new): rungs, the frame plan, the cap, pause and step, the slow-down, the 30-frames switch (§1.8) →
  Verify 1 · docs/agent/testing.md: rows for booking, floors, dt, box, latch, timing, time-control,
  scene, dump → Verify 2
- Verify: 1. `grep -c "^## " docs/agent/architecture.md docs/agent/time_control.md` · Pass:
  architecture.md ≥ 8, time_control.md ≥ 5 · Fail: fewer · 2. `grep -cE
  "^[|] .(booking|floors|dt|box|latch|timing|time-control|scene|dump). [|]" docs/agent/testing.md` ·
  Pass: 9 · Fail: fewer
- Adversarial: a page describing the latch as built when its contract scenario is still synthetic — the
  page names G-LATCH NOT PROVEN until M0-T111.
- Handoff: <placeholder>

## M0-V3 · Validation — lot 3: the step framework · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T27)
- Status: TODO
- Carried flags: [M0-V2, 2026-10-10] owed on linux-pc (R15; M0-V2 ran lot 2 on win-laptop only — reports/v2.md § Owed): python3 tools/pb/verify.py --all --task <id> with lot 2's scopes GO there — registry, eos, strings, licences (its cargo-native case over the linux-gnu set: M0-V2 read it GO from win-laptop by --filter-platform, 189 packages, 48 Linux-only, 0 refused — a probe, not the scope), watch_only, web's ready and no-webgpu with §6.2.3's Linux flags (srAdapter and Chrome's own adapter named) — then --redarm of registry, eos, strings, licences, watch_only and web there; carried from M0-T15: web and licences (npm) on linux-pc; carried from M0-D13: verify.py boot (summary.json verbatim in its log: version, adapter, steps 200) and --redarm boot (scene-refuse red); else carried V to V, M0-V17 at the latest · [M0-D17, 2026-10-10] M0-D17 (win-laptop, 2026-10-10): web/smoke.mjs now launches Chrome with chromiumSandbox: true and refuses --no-sandbox — owed on linux-pc: verify.py web and --redarm web (plant smoke-no-sandbox); a sandboxed headless /usr/bin/google-chrome under §6.2.3's Vulkan flags is unmeasured — a NO-GO there is a D or a question, never the flag put back
- Read: milestones/m0/m0_contrat.md §1.3, §1.8, §1.9.1, §2.2, §2.3.5, §2.7, §2.9, §2.12.1, §2.12.3 + the
  handoffs of M0-T18–M0-T27 + docs/agent/architecture.md, time_control.md, testing.md + the delivered
  files, each by the section it covers
- Deliver: contract-vs-code on lot 3 — the pass order and determinism (§1.3), the box, the frame and the
  latch, TimeControl against §1.8 rule by rule, booking against §2.9, the scene and dump formats, each
  defect its own D (E4), every scope the lot added red-armed, G-LATCH named NOT PROVEN (synthetic) until
  M0-T111 → reports/v3.md → Verify 1 · Phase 5's close (stubs — ten blocks, the header budget, the
  complete loop and its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V3` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: a latch proven by a hook only — the V checks that the controller also zeroes the box
  re-fit's dispatch, from the shader's code and the step-15 case.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 5 closes after M0-V3 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 5: the step framework"`
> `git push`

# Phase 6 — Lot 4 · gravity

## M0-T28 · The CPU twin's FFT and convolution · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V3)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.3 (FFT sizes), §1.4.2, §2.5.4
  (K_ν, the same machinery) + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/mod.rs and reference/fft.rs (new): a radix-2 complex FFT in
  f64 for sizes 32…2048 per axis, 2D transforms, the zero-padded convolution of a field with a kernel
  over a box (Hockney–Eastwood, §1.4.2), the kernel's transform cached per size — written here, §1.2's
  table holding no FFT crate → Verify 1 · its tests: the FFT against a direct DFT on small sizes
  (10⁻¹² relative), the convolution against direct summation on a 48 × 40 box with an asymmetric kernel
  (10⁻¹⁰), Parseval → scope `fft-twin` → Verify 1 · plant tests/plants/fft-twiddle-sign.patch (new) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py fft-twin --task M0-T28` · Pass: GO, ≥ 4 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm fft-twin --task M0-T28` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a convolution tested with a symmetric kernel hides a flipped index — the kernel is
  asymmetric; zero padding one cell short wraps mass around — the box's edge cells are in the direct-sum
  comparison.
- Handoff: <placeholder>

## M0-T29 · Gravity on the CPU twin (P2) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T28)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.1–§1.4.3, §1.7, §5.1 (G-GRAV1,
  G-GRAV2) + milestones/m0/reports/sim_models.md SM-C1 (the force-error measurement) +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/grav.rs (new): φ = K ∗ Σ_src over the box with K(i, j) = −G/
  √(i² + j²) and K(0, 0) = −G·4 ln(1 + √2) (§1.4.2), Σ_src = 0 in vacuum, free space (no image masses),
  g = −∇φ by central differences — measured first against G-GRAV1: over 2 % → the force kernels convolved
  directly (two more inverse FFTs, §5.1), the choice recorded for M0-T31 → Verify 1 · tests/gpu/grav.rs
  (new, twin cases): the Maclaurin disk's in-plane g(r) = −(π²GΣc/2a)·r within 2 % at every cell with
  r ≤ 0.75a for a = 40, the a = 20 over a = 40 error ratio in [1.6, 2.4] (G-GRAV1), scenes/asym_blob.json
  (new) with |Σ m·g| ≤ 10⁻⁵ Σ m|g| (G-GRAV2) → scopes `grav-force` and `grav-selfforce`, their twin cases
  → Verify 1 · plants tests/plants/grav-kernel-exponent.patch and grav-kernel-shift.patch (new, §5.1) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py grav-force grav-selfforce --task M0-T29` · Pass: GO, ≥ 3 cases
  in all, the largest force error printed · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm
  grav-force --task M0-T29` · Pass: GO — grav-kernel-exponent NO-GO · Fail: the plant stays GO · 3.
  `python3 tools/pb/verify.py --redarm grav-selfforce --task M0-T29` · Pass: GO — grav-kernel-shift NO-GO
  · Fail: the plant stays GO
- Adversarial: the error measured only at the disk's centre, where it is smallest — every cell with
  r ≤ 0.75a, the worst one printed; K(0, 0) softened instead of exact (SM-C1 doubled the error) — the
  constant asserted.
- Handoff: <placeholder>

## M0-T30 · The GPU FFT · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T29)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.3, §1.4.2, §6.2.2 +
  milestones/m0/reports/time_warp.md (the FFT proxy and its UNVERIFIED list) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/fft.rs and shaders/fft.wgsl (new): 2D complex FFTs of 32…2048 per
  axis within WebGPU's defaults (a 2048-point row does not fit one workgroup's 16,384 B with anything else
  — a multi-pass scheme there), the zero-padded convolution over the box with a cached kernel transform
  per size, as few dispatches as the limits allow (TW: small boxes are dispatch-bound, 2.65 µs per
  dispatch) → Verify 1 · tests/gpu/fft.rs (new): against M0-T28's twin on random fields at 64 × 64,
  256 × 128 and 2048 × 1024 (≤ 10⁻⁵ relative L2 in f32), the convolution against the twin's → scope `fft`
  → Verify 1 · plant tests/plants/fft-gpu-bitreverse.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py fft --task M0-T30` · Pass: GO, ≥ 4 cases, the dispatch count per
  transform in the log · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm fft --task M0-T30` · Pass:
  GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: a scheme tuned on the 5090's limits that exceeds WebGPU's defaults — the device is M0-T2's
  `Limits::default()` one; per-butterfly cos/sin costing more than the box's whole step (TW's proxy) — the
  twiddles tabulated, the cost in the log.
- Handoff: <placeholder>

## M0-T31 · Gravity on the GPU (P2) and one step's cost at 600 × 400 · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T30)
- Status: TODO
- Carried flags: [M0-T21, 2026-10-10] The FFT size per axis lives only on the GPU (ActiveBox.fft_w/fft_h, boxfit.rs); choosing a cached kernel transform per size on the CPU would need a readback — size gravity's dispatches indirectly or hold every power-of-two size's kernel (32 … 4096) (M0-T21).
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.3, §1.4, §5.1 (G-GRAV1,
  G-GRAV2), §6.1 + milestones/m0/reports/time_warp.md M2–M4 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/grav.rs and shaders/grav.wgsl (new): P2 by M0-T30's convolution over
  the box with §1.4.2's kernel (or M0-T29's force kernels if the twin found the central difference over
  2 %), g = −∇φ, φ and g kept for P4 and the summary, the gravity limit entering Δt (M0-T20) → Verify 1 ·
  tests/gpu/grav.rs: G-GRAV1's and G-GRAV2's GPU cases against their oracles and within G-REF's
  tolerance of the twin → `grav-force` and `grav-selfforce` gain them → Verify 1 · the TP flag's
  measurement, before any ending is built: `headless --timing` on the Quadro RTX 4000 for
  scenes/fill_world.json (the box = the whole 600 × 400 world) and scenes/sun_disk.json (a Sun-sized box)
  — P2's and the step's ms per step, with TW's proxy (1.24 ms whole-world FFT) beside them, in the log and
  the handoff → Verify 2, 3
- Verify: 1. `python3 tools/pb/verify.py grav-force grav-selfforce --task M0-T31` · Pass: GO, ≥ 6 cases in
  all · Fail: NO-GO · 2. `build/target/release/sandbox-reactions headless --scene scenes/fill_world.json
  --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-t31-world` · Pass: exit 0, P2 in
  per_pass_ms, SR-ADAPTER naming the Quadro · Fail: a non-zero exit, no timing, or another adapter · 3.
  `build/target/release/sandbox-reactions headless --scene scenes/sun_disk.json --steps 200 --timing
  --adapter "Quadro RTX 4000" --out build/m0-t31-sun` · Pass: as 2 · Fail: as 2 · 4. `python3
  tools/pb/verify.py --redarm grav-force --task M0-T31` · Pass: GO — the plant NO-GO on the GPU cases too
  · Fail: the plant stays GO
- Adversarial: a cost measured on the RTX 5090 reads several times better than the mid-range card the
  promise is about — the Quadro is named in the command and checked in SR-ADAPTER.
- Handoff: <placeholder>

## M0-T32 · Docs — physics.md (gravity) and lot 4's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T31)
- Status: TODO
- Carried flags: [M0-V2, 2026-10-10] physics.md's « Not checked at load » (Constants) names c_sb against max_gas_speed and the K1 timescale gap but not Σ_BH ≥ 4 × calibration's ns_sigma_max_sb (contract §2.6): the loader holds sigma_bh only to > 0 (registry.rs's Pos class), so the page reads as if that bound were checked; add it to the list when you edit the page (its check: M0-V7 names it, M0-V9 runs it) — M0-V2
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.3, §1.4, §7 + the handoffs of
  M0-T28–M0-T31
- Deliver: docs/agent/physics.md § Gravity: the sheet law (Q1), the kernel, the FFT over the box, M0-T29's
  force-error choice, M0-T31's measured cost pointed to (never restated) → Verify 1 ·
  docs/agent/testing.md: rows for fft-twin, grav-force, grav-selfforce, fft → Verify 2
- Verify: 1. `grep -c "^## Gravity" docs/agent/physics.md` · Pass: 1 · Fail: 0 · 2. `grep -cE
  "^[|] .(fft-twin|grav-force|grav-selfforce|fft). [|]" docs/agent/testing.md` · Pass: 4 · Fail: fewer
- Adversarial: a page that explains the softened kernel SM-C1 rejected — the V checks the page against
  §1.4.2's exact K(0, 0).
- Handoff: <placeholder>

## M0-V4 · Validation — lot 4: gravity · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T32)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.3.3, §1.4, §5.1 (G-GRAV1, G-GRAV2) + milestones/m0/reports/
  time_warp.md M2–M4, R8 + the handoffs of M0-T28–M0-T32 + docs/agent/physics.md + the delivered files,
  each by the section it covers
- Deliver: contract-vs-code on §1.4 and §1.3.3's FFT sizes, G-GRAV1 and G-GRAV2 on the twin and the GPU,
  each defect its own D (E4), every scope the lot added red-armed, M0-T31's step cost read against TW's
  proxy — the whole world is expected over a frame (TW M2), the Sun-sized box is the one that counts (TW
  M4: 0.11 ms) — a box cost already over budget is a finding for the lead, now, before any ending is built
  (M0-TP's flag) → reports/v4.md → Verify 1 · Phase 6's close (stubs, the header budget, the complete loop
  and its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V4` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: gravity graded only by its twin, so the same mistake twice agrees — the oracles are
  analytic (Maclaurin, Newton's third law), and the V checks each test reads the oracle, not the twin.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 6 closes after M0-V4 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 6: gravity"`
> `git push`

# Phase 7 — Lot 5 · gas flow

## M0-T33a · Sod's exact solution — the Riemann oracle for γ = 2 · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V4)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.3 (γ = 2), §5.1 (G-SOD) +
  milestones/m0/reports/sim_models.md SM-O5 + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/riemann.rs (new, its `mod` line in reference/mod.rs): the exact
  Riemann solution for an ideal γ = 2 gas — the star-region pressure and velocity, the wave pattern, the
  solution sampled at x/t — the oracle M0-T33b's twin case and M0-T34's GPU case share → Verify 1 · its
  tests in the module: the star-region pressure against an independent root-find (bisection on its own
  pressure function, 10⁻¹⁰ relative), Sod's states (ρ 1 | 0.125, P 1 | 0.1, at rest) satisfying the
  Rankine–Hugoniot jumps across the shock and the isentrope across the rarefaction (10⁻¹⁰), the mirrored
  states giving the mirrored solution → scope `riemann` (`cargo test -p sr-physics --release --lib
  reference::riemann::`) → Verify 1 · plant tests/plants/riemann-gamma.patch (new: γ = 2 → 1.4 in the
  star-region solve only) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py riemann --task M0-T33a` · Pass: GO, ≥ 3 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm riemann --task M0-T33a` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: an oracle written by the same hand as the scheme agrees with its mistakes — it is written
  first, in its own block, and its star-region pressure is checked by an independent root-find and by the
  jump conditions, never by a scheme's output.
- Handoff: <placeholder>

## M0-T33b · Gas flow on the CPU twin (P4) — G-SOD's twin case · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T33a)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.7 (hydro), §2.3, §2.8 (bounce),
  §5.1 (G-SOD) + milestones/m0/reports/sim_models.md SM-E13, SM-E14, SM-O5 + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/hydro.rs (new): MUSCL-Hancock with the monotonised-central
  limiter on (Σ, u, v, Π, X_i), HLL with Davis speeds, unsplit, species advected with the mass flux, the
  full EOS of M0-T12 (cold pressure off when its K's are 0), bounce ghosts (two per side, §2.8), Δt per
  §2.3.5 → Verify 1 · tests/gpu/sod.rs (new, twin case) with scenes/sod.json (new: 400 × 4, bounce,
  ρ 1 | 0.125, P 1 | 0.1, at rest, cold pressure off through `overrides` — the oracle is an ideal γ = 2
  gas — read at t = 80): against M0-T33a's exact solution, the shock within 1 cell, L1 density error
  ≤ 2 % → scope `sod`, its twin case → Verify 1 · plant tests/plants/hll-swap.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py sod --task M0-T33b` · Pass: GO, ≥ 2 cases (the shock, the L1
  error) · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm sod --task M0-T33b` · Pass: GO — the
  plant NO-GO · Fail: the plant stays GO
- Adversarial: an unsplit scheme graded on a strip along x hides a wrong y flux — M0-T34's y-oriented copy
  of the strip and G-REF's collapsing disk (M0-T38) grade the second axis; the case reads M0-T33a's exact
  solution, never the scheme's own output.
- Handoff: <placeholder>

## M0-T34 · Gas flow on the GPU (P4) with bounce walls · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T33b)
- Status: TODO
- Carried flags: [M0-T21, 2026-10-10] Run P4 over the box: Size::Indirect(boxfit.args(), ARGS_TILES) and the ActiveBox uniform (named abox — active is a WGSL reserved word). Ghosts at the box edge read cells P8 never floors: an uploaded vacuum cell may hold Σ = 0, not Σ_floor — treat Σ < Σ_vac as the floor state on read, as dt's cell_rates does (M0-T21).
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P4), §2.2 (packing), §2.3,
  §2.8 (bounce), §5.1 (G-SOD), §6.2.2 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/hydro.rs and shaders/hydro.wgsl (new): P4 per §2.3.4 over the box
  and its ghosts — M0-T33b's scheme in f32 within 8 storage buffers per stage, eos.wgsl's pressure, the
  bounce walls of §2.8, fused into as few dispatches as the limits allow (TW: dispatch-bound boxes) →
  Verify 1 · tests/gpu/sod.rs: G-SOD's GPU case → Verify 1 · P4's ms per step on the Quadro for
  scenes/sun_disk.json (`--timing`) in the handoff → Verify 2
- Verify: 1. `python3 tools/pb/verify.py sod --task M0-T34` · Pass: GO, ≥ 4 cases (twin and GPU) · Fail:
  NO-GO · 2. `build/target/release/sandbox-reactions headless --scene scenes/sun_disk.json --steps 200
  --timing --adapter "Quadro RTX 4000" --out build/m0-t34-timing` · Pass: exit 0, P4 in per_pass_ms ·
  Fail: a non-zero exit or no P4 · 3. `python3 tools/pb/verify.py --redarm sod --task M0-T34` · Pass:
  GO — hll-swap NO-GO on the GPU case too · Fail: the plant stays GO
- Adversarial: the GPU passing Sod because the strip is 4 cells tall while the 2D unsplit fluxes are wrong
  — G-REF's collapsing disk (M0-T38) grades the second axis; Sod's strip run along x only — a y-oriented
  copy of the case.
- Handoff: <placeholder>

## M0-T35 · The leave edge and escaped mass (G-EDGE) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T34)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.8, §2.9 (escaped), §5.1 (G-EDGE)
  + docs/agent/testing.md
- Deliver: crates/sr-engine/shaders/hydro.wgsl and src/step/hydro.rs, and the twin's
  crates/sr-physics/src/reference/hydro.rs: *leave* ghosts (the edge cell copied with the outward velocity
  clamped outward, the boundary face's mass flux clamped to outflow — inflow exactly 0), whatever crosses
  booked `escaped` per mass, momentum, energy and species (M0-T18), `set_edge_mode` at the next step
  boundary, a new world in leave (D1) → Verify 1 · tests/gpu/edges.rs (new) with scenes/edge_pulse.json
  (new): bounce — the wall's mass flux exactly 0 and a pulse returning with its mass-weighted normal
  velocity within 2 % of −v₀, leave — the grid's mass loss equal to `escaped` to 10⁻⁶ and inflow exactly
  0, GPU and twin → scope `edges` (G-EDGE) → Verify 1 · plant tests/plants/wall-copy.patch (new, §5.1) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py edges --task M0-T35` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm edges --task M0-T35` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a leave edge that books escaped mass from the ghost's state rather than the face flux
  drifts by the ghost's update — the case compares the grid's loss with the booked term to 10⁻⁶.
- Handoff: <placeholder>

## M0-T36 · Gravity's source in the flow — the cold collapse (G-COLL) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T35)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.2 (the sources), §2.7 (the K's),
  §5.1 (G-COLL) + milestones/m0/reports/sim_models.md SM-O2, SM-C2 + docs/agent/testing.md
- Deliver: crates/sr-engine/shaders/hydro.wgsl and the twin's reference/hydro.rs: P4's momentum source Σg
  and energy source Σu·g, time-centred on the predictor's half-step velocity (§1.4.2) → Verify 1 ·
  tests/gpu/collapse.rs (new) with scenes/cold_disk.json (new: T_floor, Σc = 1, a = 40, heat and reactions
  off, cold pressure off through `overrides` — G-COLL's "pressure is negligible" holds only so: at Σ = 1
  §2.7's K's give Π_cold ≈ 14 against Π_th ≈ 0.1, my arithmetic): r₅₀/r₅₀(0) = 0.8368 at 0.5 t_ff and
  0.5279 at 0.8 t_ff within 3 %, GPU and twin → scope `collapse` (G-COLL) → Verify 1 · plant
  tests/plants/grav-half.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py collapse --task M0-T36` · Pass: GO, ≥ 4 cases · Fail: NO-GO —
  red with the K's off is a D on the source terms, never a looser band · 2. `python3 tools/pb/verify.py
  --redarm collapse --task M0-T36` · Pass: GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: a collapse that passes because the override switched off more than cold pressure —
  the scene's overrides list only the four K's, named in the handoff.
- Handoff: <placeholder>

## M0-T37 · The twin's step driver and `--cpu-reference` · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T36)
- Status: TODO
- Carried flags: [M0-T19, 2026-10-10] P8's f64 twin lives inline in crates/sr-engine/tests/gpu/floors.rs (p8_twin, on sr_physics::eos::Eos + Elements::composition): move it into reference/floors.rs and match contract §2.7 [M0-T19] — order vacuum reset, temperature floor, renormalise; μ/Y_e/X_n of the renormalised fractions; all ≤ 0 → pure H; the reset keeps the fractions; E_floor − E and the floor's energy both booked floor_added; Σ_vac and Σ_floor compared as f32 (the GPU's)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2, §1.7 (every solver's twin),
  §3.3 (`--cpu-reference`) + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/mod.rs (M0-T28's) and reference/floors.rs (new): the twin's state
  (§2.2's channels in f64) and step — the passes built so far in §1.3.2's order, the scene's switches
  honoured, P8's floors and Δt in f64 (later passes join with their twins) → Verify 1 ·
  crates/sr-engine/src/headless.rs: `--cpu-reference` running the twin from the same scene (worlds up to
  128 × 128, exit 4 above), the same summary.json and dumps → Verify 1 · tests/gpu/twin.rs (new): a 64 ×
  64 disk run by `--cpu-reference` and in-process gives identical dumps, a 200 × 200 world refused → scope
  `twin` → Verify 1 · plant tests/plants/twin-skip-floors.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py twin --task M0-T37` · Pass: GO, ≥ 2 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm twin --task M0-T37` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a twin that skips a pass the GPU runs agrees with nothing in G-REF — the driver's pass list
  printed beside the GPU's in the log.
- Handoff: <placeholder>

## M0-T38 · GPU against the twin (G-REF) — the collapsing disk and the Sod strip · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T37)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.7, §2.2 (canonical order), §5.1
  (G-REF), §6.7 + docs/agent/testing.md
- Deliver: tests/gpu/cpu_gpu.rs (new) with scenes/ref_disk.json and scenes/ref_sod.json (new, 64 × 64):
  GPU f32 against the twin f64 over 200 steps — relative L1 difference ≤ 10⁻³ in Σ and E, ≤ 10⁻⁴ absolute
  in each X (the burning field joins with M0-T48) → scope `cpu-gpu` (G-REF) → Verify 1 · plant
  tests/plants/cfl-gpu.patch (new, §5.1: the GPU's CFL 0.4 → 0.39) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py cpu-gpu --task M0-T38` · Pass: GO, 2 cases, the largest
  difference per channel printed · Fail: NO-GO — G-REF is UNVERIFIED (§5.5): a red is a D, never a
  looser band · 2. `python3 tools/pb/verify.py --redarm cpu-gpu --task M0-T38` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: a 200-step comparison too short to separate CFL 0.4 from 0.39 makes the plant unfailable —
  the red-arm proves it fails, else the scene runs longer and the handoff says so.
- Handoff: <placeholder>

## M0-T39 · The box against the whole world (G-BOX) · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T38)
- Status: TODO
- Carried flags: [M0-T21, 2026-10-10] tests/gpu/box.rs exists (module r#box, 6 cases, scope expected 6 — raise it); the switch is Step::with_box(…, BoxMode::WholeWorld) (M0-T21).
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.3, §5.3 (G-BOX) +
  milestones/m0/reports/time_warp.md R5 + docs/agent/testing.md
- Deliver: tests/gpu/box.rs: G-BOX's cases — scenes/sun_disk.json run 2,000 steps in the box against a
  run under M0-T21's whole-world switch: mass, momentum, energy and each species' mass equal within 10⁻⁶
  relative (f64 sums), a blob at 0.35 cells per step aimed at the box's edge (scenes/box_blob.json, new)
  losing no mass in bounce mode (10⁻⁶) → scope `box` gains them → Verify 1 · margin-zero (M0-T21) now red
  on the blob case too → Verify 2
- Verify: 1. `python3 tools/pb/verify.py box --task M0-T39` · Pass: GO, ≥ 7 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm box --task M0-T39` · Pass: GO — margin-zero NO-GO on the blob
  case · Fail: the plant stays GO
- Adversarial: a blob slower than the CFL bound never tests the margin — 0.35 cells per step is §5.3's
  speed, close to the 0.4 the margin was sized for (§1.3.3).
- Handoff: <placeholder>

## M0-T40 · Docs — physics.md (gas flow, edges, the CPU twin) and lot 5's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T39)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.7, §2.3, §2.8, §7 + the handoffs of
  M0-T33a–M0-T39
- Deliver: docs/agent/physics.md § Gas flow, § Edges, § The CPU twin (the driver, `--cpu-reference`,
  G-REF's role, the overrides the oracles need) → Verify 1 · docs/agent/testing.md: rows for riemann, sod,
  edges, collapse, twin, cpu-gpu and box's new cases → Verify 2
- Verify: 1. `grep -cE "^## (Gas flow|Edges|The CPU twin)" docs/agent/physics.md` · Pass: 3 · Fail:
  fewer · 2. `grep -cE "^[|] .(riemann|sod|edges|collapse|twin|cpu-gpu). [|]" docs/agent/testing.md` ·
  Pass: 6 · Fail: fewer
- Adversarial: the overrides that switch cold pressure off in Sod and the collapse, left out of the page,
  read later as a bug — the page names them and why.
- Handoff: <placeholder>

## M0-V5 · Validation — lot 5: gas flow · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T40)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.3.3, §1.4.2, §1.7, §2.3, §2.8, §5.1 (G-SOD, G-EDGE, G-COLL,
  G-REF), §5.3 (G-BOX) + the handoffs of M0-T33a–M0-T40 + docs/agent/physics.md + the delivered files,
  each by the section it covers
- Deliver: contract-vs-code on lot 5 — the scheme against §2.3.4, the edges against §2.8, the gravity
  sources, the twin's driver, G-REF and G-BOX, each defect its own D (E4), every scope the lot added
  red-armed, the overrides the oracles need (cold pressure off) named as the only departures from the
  contract's scenes → reports/v5.md → Verify 1 · Phase 7's close (stubs, the header budget, the complete
  loop and its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V5` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: G-COLL green only because more was switched off than cold pressure — the V diffs the
  scene against §5.1's description.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 7 closes after M0-V5 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 7: gas flow"`
> `git push`

# Phase 8 — Lot 6 · heat, light and burning

## M0-T41 · Heat and light on the CPU twin (P5) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V5)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.4 (S1), §1.7 (heat), §2.4, §5.1
  (G-HEAT1, G-HEAT2, G-FLD) + milestones/m0/reports/sim_models.md SM-E18–SM-E22, SM-O7, SM-O8 +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/heat.rs (new): one-temperature flux-limited diffusion with
  Levermore–Pomraning's λ(R) (§2.4.1), κ = κ₀(1 + X_H) plus κ_dust·Z_met where T < T_dust (S1, zero while
  disabled, §2.4.2), RKL2 with s from Δt and coefficients frozen at the pass's start (§2.4.3, `SR-WARN heat
  stages <s>` above 32), the flux into vacuum or through the edge in either mode booked `radiated`
  (§2.4.4), F kept, the scene's `test` flags (pin_temperature, constant_diffusivity, flux_limiter) →
  Verify 1 · tests/gpu/heat.rs (new, twin cases) with scenes/heat_pulse.json and scenes/heat_spot.json
  (new): σ²(t) = σ₀² + 2χt per axis within 1 % and total heat to 10⁻⁵ (G-HEAT1), RKL2 against explicit
  sub-steps, σ² within 1 % (G-HEAT2), thin gas (Σ = 10⁻⁴) around a hot spot, |F| ≤ c_sb a_r T⁴ (1 + 10⁻⁶)
  at every face (G-FLD), a hot slab beside vacuum whose booked `radiated` equals the heat the grid lost, to
  10⁻⁶ (in `heat-gauss`) → scopes `heat-gauss`, `heat-rkl2`, `heat-limiter`, their twin cases → Verify 1 ·
  plants tests/plants/chi-scale.patch, rkl2-s-minus-one.patch, limiter-off.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py heat-gauss heat-rkl2 heat-limiter --task M0-T41` · Pass: GO, ≥ 5
  cases in all · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm heat-gauss --task M0-T41` · Pass:
  GO — chi-scale NO-GO · Fail: the plant stays GO · 3. `python3 tools/pb/verify.py --redarm heat-rkl2
  --task M0-T41` · Pass: GO — rkl2-s-minus-one NO-GO · Fail: the plant stays GO · 4. `python3
  tools/pb/verify.py --redarm heat-limiter --task M0-T41` · Pass: GO — limiter-off NO-GO · Fail: the
  plant stays GO
- Adversarial: a pulse wide enough that one stage too few still lands within 1 % makes rkl2-s-minus-one
  unfailable — the red-arm proves it, else the pulse narrows and the handoff says so.
- Handoff: <placeholder>

## M0-T42 · Heat and light on the GPU (P5) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T41)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P5), §2.4, §5.1 (G-HEAT1,
  G-HEAT2, G-FLD), §6.2.2 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/heat.rs and shaders/heat.wgsl (new): P5 per M0-T41 in f32 — RKL2's s
  stages as dispatches with coefficients frozen at P5's start, the limiter, radiated energy into the frame
  accumulators (M0-T18), F kept for step n+1's P4 → Verify 1 · tests/gpu/heat.rs: the GPU cases of
  G-HEAT1, G-HEAT2 and G-FLD and of the radiated booking → Verify 1 · P5's ms per step and its s on the Quadro for
  scenes/sun_disk.json (`--timing`) in the handoff → Verify 2
- Verify: 1. `python3 tools/pb/verify.py heat-gauss heat-rkl2 heat-limiter --task M0-T42` · Pass: GO, ≥ 10
  cases in all · Fail: NO-GO · 2. `build/target/release/sandbox-reactions headless --scene
  scenes/sun_disk.json --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-t42-timing` · Pass:
  exit 0, P5 in per_pass_ms · Fail: a non-zero exit or no P5 · 3. `python3 tools/pb/verify.py --redarm
  heat-rkl2 --task M0-T42` · Pass: GO — the plant NO-GO on the GPU case · Fail: the plant stays GO
- Adversarial: up to 32 stages a step on a dispatch-bound box (TW: 2.65 µs each) may cost more than the
  rest of the step — s and P5's share are in the handoff for M0-T63's probe.
- Handoff: <placeholder>

## M0-T43 · The radiation force in the flow · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T42)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P4), §2.4.4 +
  docs/agent/testing.md
- Deliver: crates/sr-engine/shaders/hydro.wgsl and the twin's reference/hydro.rs: the force per area
  f = κΣF/c_sb from the previous step's F as P4's momentum source, E unchanged — the work comes out of
  thermal energy (§2.4.4) → Verify 1 · tests/gpu/radforce.rs (new): a hot slab in a uniform medium — the
  momentum gained over n steps equals Σ f Δt to 10⁻⁶ relative, E unchanged to f32 rounding, GPU and twin
  → scope `radiation-force` → Verify 1 · plant tests/plants/radforce-zero.patch (new: f zeroed in P4) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py radiation-force --task M0-T43` · Pass: GO, ≥ 3 cases · Fail:
  NO-GO · 2. `python3 tools/pb/verify.py --redarm radiation-force --task M0-T43` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: the force taken from this step's F instead of the previous one changes the scheme's order
  silently — the case steps F by hand and checks which one P4 read.
- Handoff: <placeholder>

## M0-T44a · The rate law — ω and its temperature table · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T43)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.1–§1.6.3, §2.5.1 +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/rates.rs (new): ω = A Σ^a Π X_i^{n_i} f(T) g(Σ) from a registry record
  (§1.6.1), f(T) with T_thr and its 256-point log table (≤ 10⁻⁴ relative against the direct form), the
  gate g(Σ) — the law M0-T44b's sub-cycles and M0-T46's shader evaluate → Verify 1 · its tests in the
  module: the table against the direct form at every node and midpoint (≤ 10⁻⁴), f(T) = 0 below T_thr,
  g(Σ) on both sides of Σ_g (N_Fe's gate), ω for each record of assets/reactions.json against the formula
  worked by hand → scope `rates` (`cargo test -p sr-physics --release --lib rates::`) → Verify 1 · plant
  tests/plants/rates-table-shift.patch (new: the table read one node off) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py rates --task M0-T44a` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm rates --task M0-T44a` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: log-log interpolation is exact for one power law and not for H_burn's two terms (ν 4 and
  18) — the midpoint cases use H_burn; a table interpolated across T_thr smears the step f takes there —
  a case brackets T_thr.
- Handoff: <placeholder>

## M0-T44b · Burning on the CPU twin (P6) — G-BURN, G-ORDER twin cases · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T44a)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.1–§1.6.3, §1.7 (reactions),
  §2.5.1, §2.5.2, §5.1 (G-BURN, G-ORDER) + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/react.rs (new): P6's per-cell linearised backward-Euler
  sub-cycles under burn_dx and burn_dt on M0-T44a's ω, fractions clamped ≥ 0 and renormalised, q·ω into
  ε_th at each sub-step, nuclear energy booked per group (§2.5.2) → Verify 1 · tests/gpu/burn.rs (new,
  twin cases): one cell, T pinned, flow off — H_burn X = X₀/(1 + kX₀t), He_burn X = X₀/√(1 + 2kX₀²t),
  Ne_burn X = X₀e^{−kt} within 10⁻⁴ down to X₀/10, heat = q·Δm within 10⁻⁵, ΣX = 1 within 10⁻⁶ (G-BURN),
  a cell of equal parts H, He, C, O, Ne and Si heated at a steady rate passing ε* = 10⁻³ Q_H per t.u. in
  the order H, He, C, {Ne, O}, Si (G-ORDER) → scopes `burn-cell`, `burn-order`, their twin cases →
  Verify 1 · plants tests/plants/q-double.patch, swap-he-c.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py burn-cell burn-order --task M0-T44b` · Pass: GO, ≥ 6 cases in all
  · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm burn-cell --task M0-T44b` · Pass: GO — q-double
  NO-GO · Fail: the plant stays GO · 3. `python3 tools/pb/verify.py --redarm burn-order --task M0-T44b` ·
  Pass: GO — swap-he-c NO-GO · Fail: the plant stays GO
- Adversarial: sub-cycles that meet the analytic X but book the heat once per step, not per sub-step —
  the heat case sums q·Δm per sub-step against the total.
- Handoff: <placeholder>

## M0-T45 · Neutrino cooling and the iron gate on the CPU twin · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T44b)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.2 (N_Fe), §1.8.5 (the latch's P6
  trigger), §2.5.3 + docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/react.rs: ε_ν = A_ν (T/T_ν)^m for T ≥ T_ν booked
  `neutrino_lost`, S_ν = ε_νΣ where X_n ≥ 0.5, kept for P7, N_Fe's gate on Y_eΣ against Σ_N (p = 1), its
  q < 0 absorbing, the latch's condition (any cell's N_Fe rate > 0) exposed → Verify 1 ·
  tests/gpu/neutrino.rs (new, twin cases): cooling against the closed form at pinned T (10⁻⁶), none below
  T_ν, S_ν only where X_n ≥ 0.5, N_Fe off just under Σ_N and on just over, mass conserved per record →
  scope `neutrino` → Verify 1 · plant tests/plants/nfe-gate-off.patch (new: the gate ignored) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py neutrino --task M0-T45` · Pass: GO, ≥ 5 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm neutrino --task M0-T45` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: the gate read on Σ instead of Y_eΣ fires early for iron (Y_e ≈ 0.46) — the threshold case
  sits between Σ_N and Σ_N/Y_e.
- Handoff: <placeholder>

## M0-T46 · Burning on the GPU (P6) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T45)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P6), §1.6.1, §2.5.1, §2.5.2,
  §5.1 (G-BURN, G-ORDER), §6.2.2 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/react.rs and shaders/react.wgsl (new): the registry's enabled records
  uploaded, f(T) evaluated or tabulated (≤ 10⁻⁴, M0-T44a's table), M0-T44b's sub-cycles per cell in f32,
  nuclear energy per group into the frame accumulators → Verify 1 · tests/gpu/burn.rs: the GPU cases of G-BURN and G-ORDER →
  Verify 1 · P6's ms per step on the Quadro for a burning scenes/sun_disk.json run (`--timing`) in the
  handoff → Verify 2
- Verify: 1. `python3 tools/pb/verify.py burn-cell burn-order --task M0-T46` · Pass: GO, ≥ 12 cases in all
  · Fail: NO-GO · 2. `build/target/release/sandbox-reactions headless --scene scenes/sun_disk.json --steps
  200 --timing --adapter "Quadro RTX 4000" --out build/m0-t46-timing` · Pass: exit 0, P6 in per_pass_ms ·
  Fail: a non-zero exit or no P6 · 3. `python3 tools/pb/verify.py --redarm burn-cell --task M0-T46` ·
  Pass: GO — q-double NO-GO on the GPU case · Fail: the plant stays GO
- Adversarial: a per-cell loop of up to hundreds of sub-cycles diverging across a workgroup — the cost per
  step of a hot core against a cold field is in the handoff.
- Handoff: <placeholder>

## M0-T47 · Neutrinos and the iron gate on the GPU, setting the latch · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T46)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.5, §2.5.3, §5.3 (G-LATCH) +
  docs/agent/testing.md
- Deliver: crates/sr-engine/shaders/react.wgsl and src/step/react.rs: neutrino cooling, S_ν and N_Fe's
  gate (M0-T45) in f32, P6 setting the latch (M0-T22) when any cell's N_Fe rate is > 0 → Verify 1 ·
  tests/gpu/neutrino.rs: the GPU cases, a frame whose step k turns N_Fe on stops after step k with the
  slow-down on — the latch's first trigger by the physics → Verify 1
- Verify: 1. `python3 tools/pb/verify.py neutrino latch --task M0-T47` · Pass: GO, ≥ 10 cases in neutrino
  and ≥ 5 in latch · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm neutrino --task M0-T47` · Pass:
  GO — nfe-gate-off NO-GO on the GPU cases · Fail: the plant stays GO
- Adversarial: the flag set from the rate of the step's last sub-cycle only misses a rate that turned on
  and off within the step — the flag is set from any sub-cycle's rate.
- Handoff: <placeholder>

## M0-T48 · The burning field in G-REF · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T47)
- Status: TODO
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §5.1 (G-REF) + tests/gpu/cpu_gpu.rs
  (M0-T38's cases, the shape to copy)
- Deliver: tests/gpu/cpu_gpu.rs: the third case — scenes/ref_burn.json (new: 64 × 64, a hot burning
  field, heat and reactions on) run 200 steps on the GPU and the twin within §5.1's tolerances, the case
  copied from M0-T38's shape, `cpu-gpu`'s expected count 2 → 3 in tools/pb/verify.json → Verify 1
- Verify: 1. `python3 tools/pb/verify.py cpu-gpu --task M0-T48` · Pass: GO, 3 cases · Fail: NO-GO, or
  fewer — a red is a D (G-REF is UNVERIFIED, §5.5)
- Adversarial: a "burning" field too cool to burn in 200 steps compares nothing — the case asserts that
  H's mass fell by ≥ 1 % in the twin.
- Handoff: <placeholder>

## M0-T49 · Docs — physics.md (heat, light, reactions, neutrinos) and lot 6's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T48)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6, §2.4, §2.5, §7 + the handoffs of
  M0-T41–M0-T48
- Deliver: docs/agent/physics.md § Heat and light, § Reactions, § Neutrinos — S1's switch named (§1.6.4)
  → Verify 1 · docs/agent/testing.md: rows for heat-gauss, heat-rkl2, heat-limiter, radiation-force,
  rates, burn-cell, burn-order, neutrino and cpu-gpu's third case → Verify 2
- Verify: 1. `grep -cE "^## (Heat and light|Reactions|Neutrinos)" docs/agent/physics.md` · Pass: 3 ·
  Fail: fewer · 2. `grep -cE "^[|] .(heat-|rates|burn-|neutrino|radiation-force)" docs/agent/testing.md` ·
  Pass: 8 · Fail: fewer
- Adversarial: the q ratios written as tuned when K8 fixes them to nature's — the page says which numbers
  are tuning (A, T_k, ν) and which are not.
- Handoff: <placeholder>

## M0-V6 · Validation — lot 6: heat, light and burning · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T49)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.6, §1.7, §1.8.5, §2.4, §2.5, §5.1 (G-HEAT1, G-HEAT2, G-FLD, G-BURN,
  G-ORDER, G-REF) + the handoffs of M0-T41–M0-T49 + docs/agent/physics.md + the delivered files, each by
  the section it covers
- Deliver: contract-vs-code on lot 6 — the transport and its stepping, the opacity's stand-in term held
  at zero, the rate law and its sub-cycles, neutrino cooling, the iron gate and the latch it sets, each
  defect its own D (E4), every scope the lot added red-armed, P5's and P6's costs on the Quadro gathered
  beside P2's and P4's (lots 4–5) for M0-T63 → reports/v6.md → Verify 1 · Phase 8's close (stubs, the
  header budget, the complete loop and its seconds, `rung_record.py report`'s table, a moved class → a
  TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V6` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: q ratios checked on H_burn only — the V checks each record's q against §1.6.2's ratio ×
  Q_H, and the two UNVERIFIED ones (He_burn, C_alpha) stay flagged as §1.6.2 says.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 8 closes after M0-V6 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 8: heat, light and burning"`
> `git push`

# Phase 9 — Lot 7 · black holes and neutrino heating

## M0-T50 · Sinks on the CPU twin (P3) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V6)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.3, §1.7 (sinks), §2.6, §2.9
  (swallowed), §5.1 (G-SINK) + milestones/m0/reports/sim_models.md SM-E28, SM-E29 +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/sinks.rs (new): formation (all four criteria at one cell, the
  densest candidate, ties lowest y then x, at most one per step), accretion within r_acc = max(2,
  2GM_s/c_sb²) of gas bound to the sink, its energy booked `swallowed`, kick-drift-kick with the grid's g
  (bilinear) plus the other sinks' pull softened by r_acc, merging, up to 16 sinks, and in
  reference/grav.rs the sinks' potential −GM_s/√(r² + r_acc²) (§1.4.3) → Verify 1 · tests/gpu/sink.rs
  (new, twin cases) with scenes/sink_cloud.json (new): grid + sink mass and momentum constant to 10⁻⁶,
  no mass outward across r_acc, formation refused when any one criterion fails (four cases) → scope `sink`
  (G-SINK), its twin cases → Verify 1 · plant tests/plants/sink-momentum.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py sink --task M0-T50` · Pass: GO, ≥ 6 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm sink --task M0-T50` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a density threshold alone forming sinks in a bouncing core (SM-E28's warning) — each of the
  other three criteria has its own refusal case.
- Handoff: <placeholder>

## M0-T51 · Sinks on the GPU (P3) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T50)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P3), §1.3.4, §2.6, §5.1
  (G-SINK) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/sinks.rs and shaders/sinks.wgsl (new): the sink state in a small
  buffer, candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4), KDK and
  merging in one small dispatch — no CPU round trip inside a frame, `swallowed` booked (M0-T18) → Verify 1
  · tests/gpu/sink.rs: G-SINK's GPU cases → Verify 1
- Verify: 1. `python3 tools/pb/verify.py sink --task M0-T51` · Pass: GO, ≥ 12 cases (twin and GPU) ·
  Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm sink --task M0-T51` · Pass: GO — the plant NO-GO
  on the GPU cases · Fail: the plant stays GO
- Adversarial: two sinks reducing into each other's accretion in parallel take the same cell twice — the
  case puts two sinks' r_acc over one cell and checks the mass once.
- Handoff: <placeholder>

## M0-T52a · The sinks' pull and the latch on formation · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T51)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.3, §1.8.5, §2.6 +
  docs/agent/testing.md
- Deliver: crates/sr-engine/shaders/grav.wgsl: the sinks' potential −GM_s/√(r² + r_acc²) added to φ in
  P2 (§1.4.3) → Verify 1 · crates/sr-engine/shaders/sinks.wgsl: the latch set when a sink forms (M0-T22)
  → Verify 1 · tests/gpu/sink.rs: a test mass's g near a sink against −GM_s r/(r² + r_acc²)^{3/2}, φ
  carrying the same term, and a formation inside a frame stopping it with the slow-down on → Verify 1
- Verify: 1. `python3 tools/pb/verify.py sink latch --task M0-T52a` · Pass: GO, ≥ 15 cases in sink ·
  Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm sink --task M0-T52a` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: a sink's pull added to g but not to φ leaves W wrong in the ledger and the virial — the
  case reads both.
- Handoff: <placeholder>

## M0-T52b · Sinks in the state dump · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T52a)
- Status: TODO
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.12.3 (sinks) +
  crates/sr-engine/src/dump.rs (M0-T26's writer and reader)
- Deliver: crates/sr-engine/src/dump.rs: the header's `sinks` list — x, y, vx, vy and m per sink —
  written and restored (§2.12.3), and tests/gpu/dump.rs: a round trip with two sinks, bit-identical, its
  case joining the `dump` scope (the expected count raised in tools/pb/verify.json) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py dump --task M0-T52b` · Pass: GO, ≥ 5 cases · Fail: NO-GO
- Adversarial: a dump that restores the cells but drops the sinks resumes a world without its black
  hole — the round trip compares the sinks' state too.
- Handoff: <placeholder>

## M0-T53 · Neutrino heating on the CPU twin (P7) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T52b)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.4 (S2), §2.5.4, §2.7 (f_dep) +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/reference/nuheat.rs (new): D(x) = Σ(x)·(S_ν ∗ K_ν)(x) with K_ν(r) =
  1/(r² + 1) through M0-T28's convolution, D = 0 where X_n ≥ 0.5, q(x) = f_dep·(Σ S_ν)·D(x)/Σ D — exactly
  f_dep of the emitted power — booked `neutrino_deposited`, nothing when Σ S_ν = 0 (§2.5.4) → Verify 1 ·
  tests/gpu/nuheat.rs (new, twin cases): deposited = f_dep × emitted to 10⁻⁹, none inside the neutron
  star, nearer and denser cells receiving more, f_dep above 0.1 refused by the registry while S2 is off
  (§1.6.4) → scope `nu-heat` → Verify 1 · plant tests/plants/nuheat-no-cap.patch (new: f_dep applied
  twice) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py nu-heat --task M0-T53` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm nu-heat --task M0-T53` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a deposit normalised per cell instead of over ΣD exceeds what the core emitted — the
  energy case is exact by construction and must stay so.
- Handoff: <placeholder>

## M0-T54 · Neutrino heating on the GPU (P7) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T53)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P7), §2.5.4 +
  docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/nuheat.rs and shaders/nuheat.wgsl (new): P7 through M0-T30's
  convolution, zero dispatches when Σ S_ν = 0 (the indirect sizes zeroed from P6's sum, on the GPU) →
  Verify 1 · tests/gpu/nuheat.rs: the GPU cases (deposited = f_dep × emitted within 10⁻⁵ in f32, no
  dispatch when nothing is emitted, read from the timing report) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py nu-heat --task M0-T54` · Pass: GO, ≥ 8 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm nu-heat --task M0-T54` · Pass: GO — the plant NO-GO on the GPU
  cases · Fail: the plant stays GO
- Adversarial: a P7 that always dispatches costs two FFTs a step through every life without a neutron
  star — the zero-dispatch case.
- Handoff: <placeholder>

## M0-T55 · Docs — physics.md (sinks, neutrino heating, the stand-ins) and lot 7's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T54)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.4, §2.5.4, §2.6, §7 + the
  handoffs of M0-T50–M0-T54
- Deliver: docs/agent/physics.md § Sinks, § Neutrino heating, § The stand-ins (S1, S2: their switches,
  their measured triggers, who enables them — §1.6.4) → Verify 1 · docs/agent/testing.md: rows for sink
  and nu-heat → Verify 2
- Verify: 1. `grep -cE "^## (Sinks|Neutrino heating|The stand-ins)" docs/agent/physics.md` · Pass: 3 ·
  Fail: fewer · 2. `grep -cE "^[|] .(sink|nu-heat). [|]" docs/agent/testing.md` · Pass: 2 · Fail: fewer
- Adversarial: the stand-ins described as on — the page says they ship disabled and only a PLAN amendment
  citing a measured failure enables them.
- Handoff: <placeholder>

## M0-V7 · Validation — lot 7: black holes and neutrino heating · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T55)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.4.3, §1.6.4, §1.8.5, §2.5.4, §2.6, §2.12.3, §5.1 (G-SINK) + the
  handoffs of M0-T50–M0-T55 + docs/agent/physics.md + the delivered files, each by the section it covers
- Deliver: contract-vs-code on lot 7 — sink formation's four criteria, accretion, motion, merging, their
  pull, the latch on formation, P7's share and cap, S2 held at f_dep ≤ 0.1, each defect its own D (E4),
  every scope the lot added red-armed → reports/v7.md → Verify 1 · Phase 9's close (stubs, the header
  budget, the complete loop and its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) →
  Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V7` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: Σ_BH's bound (≥ 4 × ns_sigma_max_sb) cannot be checked before calibration — the V names it
  owed to M0-V9, not proven.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 9 closes after M0-V7 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 9: black holes and neutrino heating"`
> `git push`

# Phase 10 — Lot 8 · watching the star

## M0-T56 · The block summary · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V7)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.4, §1.9.1, §3.1 (encode_frame's
  summary, poll) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/summary.rs and shaders/summary.wgsl (new): after each frame's last step,
  over the box, blocks of 4 × 4 cells with every §1.9.1 field — mass, Σx·m, Σy·m, momentum, kinetic,
  thermal and cold energy, ∫Π dA, the ten species masses, Σm·φ, the max-Σ cell's index, Σ, T, X and
  Π_cold/Π, the frame accumulators summed (M0-T18), unbound mass, the sinks inside — read back
  asynchronously (≤ 2 frames late), the accumulators reset after each summary, appended by
  `encode_frame` (§3.1) → Verify 1 · tests/gpu/summary.rs (new): each field against an f64 CPU sum of the
  same dump within the f32 rounding it states per field, the readback ≤ 2 frames late, the accumulators
  reset → scope `summary` → Verify 1 · plant tests/plants/summary-skip-row.patch (new: the last block row
  omitted) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py summary --task M0-T56` · Pass: GO, ≥ 6 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm summary --task M0-T56` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a box whose height is not a multiple of 4 drops a partial block row — the box is a
  multiple of 8 (§1.3.3), and the plant drops a row to prove the check sees it.
- Handoff: <placeholder>

## M0-T57 · The ledger · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T56)
- Status: TODO
- Carried flags: [M0-T18, 2026-10-10] M0-T18's booking (state.rs TERMS/BOOK_GROUPS/book_totals): (1) energy.floor_added has one writer, P8 — if P4 writes a floored E back (§2.3.3), add a P4 term, never share P8's buffer; (2) book_totals sums full per-slot planes (~22 MB a readback at 600×400) — §2.9's per-frame path needs a fixed-order GPU partial reduction in TERMS order first; slots hold one frame, the ledger keeps the f64 cumulative (tasks/M0-T18.md)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.9, §2.12.2 (ledger) +
  docs/agent/testing.md
- Deliver: crates/sr-engine/src/ledger.rs (new): the mass, energy and momentum terms of §2.9 in f64 from
  the summary's fixed-order block partials and the booked terms, cumulative since the ledger (re)started,
  the two invariants, summary.json's `ledger` {start, end} through headless.rs → Verify 1 ·
  tests/gpu/ledger.rs (new): 500 steps of a collapsing disk in bounce, then in leave — the mass invariant
  constant to 10⁻⁶ relative, momentum (grid + sinks + escaped) to 10⁻⁶ of Σm|u|, the energy drift
  reported (G-CONS grades it over whole lives, M0-T106) → scope `ledger` → Verify 1 · plant
  tests/plants/ledger-no-escaped.patch (new: escaped left out of the mass sum) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ledger --task M0-T57` · Pass: GO, ≥ 3 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm ledger --task M0-T57` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a leave-mode run where nothing reaches the edge cannot see the escaped term — the disk is
  placed so mass leaves within the 500 steps, and the escaped total is printed.
- Handoff: <placeholder>

## M0-T58 · Objects · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T57)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.2, §2.7 (sigma_obj), §3.1
  (Tracker::update) + docs/agent/testing.md
- Deliver: crates/sr-physics/src/observe/mod.rs and observe/tracker.rs (new): `Tracker::update(report)` —
  8-connected blocks holding a cell with Σ ≥ Σ_obj, wisps under 1 % of the Sun-like preset left out,
  identity by the largest shared block mass (≥ 25 %), the heavier id surviving a merge, per object every
  §1.9.2 quantity (M, c, R₉₀, τ_dyn, L, L_g, L_nuc, L_ν, Σ_c, T_c, X_c, d_c, v_r, U, K, P, W, M_unb, the
  envelope mass with R_core, the sink mass) → Verify 1 · its tests on synthetic block summaries: one blob,
  two blobs and their stable ids, a merge, a split, a wisp, each quantity on a field of known answer →
  scope `objects` → Verify 1 · plant tests/plants/one-object.patch (new, §5.2's G-MULTI plant: blocks
  joined across empty blocks) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py objects --task M0-T58` · Pass: GO, ≥ 8 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm objects --task M0-T58` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: an identity rule tested only on still blobs keeps ids that a moving star would lose — a
  blob shifted by two blocks between summaries keeps its id.
- Handoff: <placeholder>

## M0-T59 · Stages and events · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T58)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.5 (the latch event), §1.9.3,
  §1.9.4 + docs/agent/testing.md
- Deliver: crates/sr-physics/src/observe/stages.rs (new): each object's stage by §1.9.3's table, first
  match wins, a stage other than core collapse and supernova displayed after 3 identical summaries, the
  `ignited` flag (L_H ≥ 0.5 L), events ignition, core collapse (the latch's report), supernova and sink
  formed, each once per object (§1.9.4) → Verify 1 · its tests: each row's match on a synthetic object,
  the order (collapsing before protostar, failed_star only when never ignited), the hold, each event once
  → scope `classify` → Verify 1 · plant tests/plants/classify-order.patch (new: protostar tested before
  collapsing) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py classify --task M0-T59` · Pass: GO, ≥ 20 cases (one per row and
  rule) · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm classify --task M0-T59` · Pass: GO — the
  plant NO-GO · Fail: the plant stays GO
- Adversarial: a label that reads the rung or the frame breaks I2 — the classifier takes the object and
  its history only, and lives in observe, which G-WATCH keeps out of the step.
- Handoff: <placeholder>

## M0-T60 · Observers in headless runs · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T59)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.12.2, §3.3 (`--until`, `--max-steps`,
  events.jsonl, exit 2) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/headless.rs: `--until sim_time:<t> | event:<…> | stage:<id> | ending` (an
  object holding a remnant stage for 10 t.u.), `--max-steps` (exit 2 when unmet), DIR/events.jsonl,
  summary.json's `events`, `objects` (id, stage, mass_sb, stages with from_step and sim_time — mass_suns,
  predicted_ending, ending and age_years once M0-T69, M0-T72 and M0-T73 exist) and `until` → Verify 1 ·
  tests/gpu/observe.rs (new): scenes/cold_disk.json `--until stage:collapsing` met, `--until sim_time:5`
  met at the first step past 5, an unmet condition → exit 2 with the DONE line reading `until=unmet`,
  events.jsonl one JSON object per line → scope `observe-run` → Verify 1 · plant
  tests/plants/until-never-met.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py observe-run --task M0-T60` · Pass: GO, ≥ 4 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm observe-run --task M0-T60` · Pass: GO — the plant NO-GO · Fail:
  the plant stays GO
- Adversarial: a condition checked on the summary of a frame that ran many steps overshoots by a frame —
  the stop step and its sim time are recorded, and the case states the overshoot it allows.
- Handoff: <placeholder>

## M0-T61 · The preset cloud at a given mass · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T60)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.1, §2.10, §2.11 (items 3–4 use it)
  + docs/agent/testing.md
- Deliver: crates/sr-engine/src/scene.rs: the preset cloud at any mass M — a Maclaurin disk with
  a = 40√(M/M₀) (M₀ the unit mass of §2.1), capped at 150 cells (Σc then grows), §2.10's solar
  composition, a uniform temperature set so U_th = 0.1|W| with W from one gravity solve of the cloud
  itself, at rest, `preset:<name>` still the nominal mass until the presets (M0-T74) → Verify 1 ·
  tests/gpu/preset.rs (new): the mass within 10⁻⁶ of M, a and Σc per formula including the cap,
  U_th/|W| = 0.1 within 10⁻⁴, the composition summing to 1 → scope `preset-cloud` → Verify 1 · plant
  tests/plants/preset-uth-half.patch (new: U_th = 0.05|W|) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py preset-cloud --task M0-T61` · Pass: GO, ≥ 4 cases · Fail: NO-GO
  · 2. `python3 tools/pb/verify.py --redarm preset-cloud --task M0-T61` · Pass: GO — the plant NO-GO ·
  Fail: the plant stays GO
- Adversarial: W taken from the analytic Maclaurin formula while the grid's W differs by the force error —
  W is the grid's own, from P2.
- Handoff: <placeholder>

## M0-T62 · No artificial fragmentation (G-JEANS) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T61)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.3.5 (the Jeans number), §5.1
  (G-JEANS), §5.5 + docs/agent/testing.md
- Deliver: tests/gpu/jeans.rs (new) with scenes/jeans_ripple.json (new): the disk cloud (U_th = 0.1|W|,
  a = 40) with the deterministic 1 % ripple (cos 3θ + cos 5θ), J ≤ 1/4 everywhere checked each summary,
  collapsing until its central Σ is 100 × the start: exactly one object (§1.9.2) and no secondary density
  peak above 10 % of the central one → scope `jeans` → Verify 1 · plant tests/plants/no-pressure-flux.patch
  (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py jeans --task M0-T62` · Pass: GO, ≥ 3 cases · Fail: NO-GO —
  G-JEANS is UNVERIFIED (§5.5): a red is a D, or a question to the lead when the tolerance is at stake ·
  2. `python3 tools/pb/verify.py --redarm jeans --task M0-T62` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a collapse stopped by cold pressure before Σ reaches 100 × never meets the condition — the
  case reports the largest Σ reached and fails rather than passing on a timeout.
- Handoff: <placeholder>

## M0-T63 · The step-cost probe — a main sequence's steps and the box's cost on the Quadro · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T62)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.2, §1.8.6, §5.3 (G-FPS, G-TOP),
  §5.5 + milestones/m0/reports/time_warp.md (Summary, R1, R8) + milestones/m0/reports/plan_redteam.md RT6
  + reports/v4.md–v6.md (the passes' costs)
- Deliver: milestones/m0/reports/step_cost.md (new): the nominal Sun-like cloud (M0-T61 at M₀) run on the
  Quadro RTX 4000 from its drop to the end of its main sequence (`--until stage:h_shell`, `--timing`) —
  steps per stage, Δt's range, the box's size and its cost per step and per pass, the steps per life
  projected and placed among TW's scenarios L, C and H, the frame cost at a ~10 s life against 16.7 ms
  (60 frames) and 33.3 ms (30 frames, Q2), each number with its command, SHA and adapter, untuned
  constants named as such, a run over 10 minutes is the lead's [NOT RUN — for you], a star that never
  ends its main sequence is the finding itself, with the stage it stalls in → none — the record M0-V8 and
  M0-TJ2 read (a BUILD block's report)
- Verify: 1. `grep -cE "^## (Run|Steps|Cost|Projection)" milestones/m0/reports/step_cost.md` · Pass: 4 ·
  Fail: fewer, or a number without its command and adapter
- Adversarial: a probe on the RTX 5090 looks several times faster than the promise's card — every number
  names the Quadro; a projection from untuned constants read as final — the report shows how K1's gap moves
  the projection (TW's scenarios).
- Handoff: <placeholder>

## M0-T64 · Docs — readouts.md (summary, ledger, objects, stages, events) and lot 8's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T63)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9, §2.9, §7 + the handoffs of
  M0-T56–M0-T63
- Deliver: docs/agent/readouts.md (new): the block summary, the ledger, objects, stages, events and the
  observers' place in headless runs (§1.9, §2.9) → Verify 1 · docs/agent/architecture.md § Observers (the
  boundary G-WATCH holds) and docs/agent/testing.md rows for summary, ledger, objects, classify,
  observe-run, preset-cloud, jeans → Verify 2
- Verify: 1. `grep -c "^## " docs/agent/readouts.md` · Pass: ≥ 5 · Fail: fewer, or no file · 2. `grep -cE
  "^[|] .(summary|ledger|objects|classify|observe-run|preset-cloud|jeans). [|]" docs/agent/testing.md` ·
  Pass: 7 · Fail: fewer
- Adversarial: stage thresholds copied into the page drift from §1.9.3 — the page points to the table.
- Handoff: <placeholder>

## M0-V8 · Validation — lot 8: watching the star · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T64)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.8.6, §1.9, §2.9, §2.10, §5.1 (G-JEANS), §5.3 + the handoffs of
  M0-T56–M0-T64 + reports/step_cost.md + docs/agent/readouts.md + the delivered files, each by the
  section it covers
- Deliver: contract-vs-code on lot 8 — the summary's fields, the ledger's invariants, objects, stages and
  events, the observers in headless runs, the preset cloud, G-JEANS, the tracker run on a real collapse's
  summaries (M0-T63's run), not only synthetic ones, each defect its own D (E4), every scope the lot added
  red-armed → reports/v8.md → Verify 1 · reports/step_cost.md read: a projected frame cost over 33.3 ms at
  a ~10 s life wakes M0-TJ2 now — moved here (E8) — else it stays DEFERRED for G-TOP (M0-T113) → none — the
  move or the line in reports/v8.md · Phase 10's close (stubs, the header budget, the complete loop and
  its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V8` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: a probe that stalled before the end of its main sequence read as a cost answer — the V
  checks the run reached h_shell, else the stall is the finding.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 10 closes after M0-V8 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 10: watching the star"`
> `git push`

# Phase 11 — Lot 9 · calibration

## M0-T65 · The calibration file, its physics hash and the refusal (G-CAL) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V8)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.4, §2.11 (schema, refusal), §3.1
  (Engine::new's errors), §3.3 (exit 6), §5.4 (G-CAL) + docs/agent/testing.md
- Deliver: crates/sr-physics/src/calibration.rs (new): §2.11's schema parsed and validated, physics_hash =
  SHA-256 of the canonical JSON (keys sorted, compact) of elements.json + reactions.json + physics.json —
  SHA-256 written here against FIPS 180-4's test vectors, §1.2's table holding no hash crate, the engine
  refusing a calibration whose hash differs (`SR-ERROR calibration out of date`, exit 6) and running
  without one for headless, test and calibrate runs whose scene names no preset (the step never reads
  calibration — G-WATCH), assets/calibration.json embedded once M0-V9's run writes it → Verify 1 ·
  tests/gpu/calibration.rs
  (new): the hash against the canonical bytes, SHA-256's vectors, a calibration written from the embedded
  assets accepted and a mismatched one refused with exit 6 → scope `calibration` (G-CAL) → Verify 1 ·
  plant tests/plants/const-drift.patch (new, §5.4) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py calibration --task M0-T65` · Pass: GO, ≥ 5 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm calibration --task M0-T65` · Pass: GO — the plant NO-GO · Fail:
  the plant stays GO
- Adversarial: a hash over the files' bytes changes with whitespace and key order while the physics does
  not — the canonical form is hashed, and a reformatted physics.json keeps its hash.
- Handoff: <placeholder>

## M0-T66 · The `calibrate` command and items 1–2 — the cold ceilings · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T65)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.3.2 (M_ch), §2.7 (the K's), §2.11
  (the procedure, items 1–2), §2.12.1 (overrides), §3.3 (calibrate), §6.4 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/calibrate.rs (new) and its subcommand in crates/sr-app/src/main.rs:
  `calibrate --adapter S --out <file> [--only <item>]` — `--only` re-measures one item and keeps the
  rest, the `measured` block {date, git, adapter, box}, never written by a run with scene overrides
  (§2.12.1), its later items filled by M0-T68 and M0-T70 → Verify 1 · items 1–2: m_ch_sb — a cold disk (T_floor, C 0.5 + O 0.5, Y_e = 0.5,
  a = 12) run 10 τ_dyn, holding when R₉₀ changes by < 3 %, bisection on M to 0.5 % (SM-O9), m_tov_sb and
  ns_sigma_max_sb the same with X_n = 1, ns_sigma_max_sb the largest Σ_c among holding runs, the CPU twin
  may stand in (§2.11), each through `--only` → Verify 1 · tests/gpu/calibrate_cold.rs (new): the
  bisection's bracket and tolerance on a coarse run, `--only` keeping the other items, a run with
  overrides refused, the order m_ch < m_tov, both within a factor 2 of
  §2.7's arithmetic (M_ch ≈ 3,970, M_tov ≈ 6,000 — a sanity band, never the oracle) → scope
  `calibrate-cold` → Verify 1 · plant tests/plants/calibrate-cold-hold.patch (new: "holds" at < 30 %) →
  Verify 2 · the two items run once on linux-pc's RTX 5090 (physics only, §6.4) into
  build/calibration.partial.json, their wall times logged, over 10 minutes → the lead's, in M0-V9's
  window → Verify 3
- Verify: 1. `python3 tools/pb/verify.py calibrate-cold --task M0-T66` · Pass: GO, ≥ 5 cases · Fail:
  NO-GO · 2. `python3 tools/pb/verify.py --redarm calibrate-cold --task M0-T66` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO · 3. `build/target/release/sandbox-reactions calibrate --adapter "RTX
  5090" --out build/calibration.partial.json --only m_ch_sb` · Pass: exit 0, m_ch_sb written, its time
  logged · Fail: exit 6 or no value
- Adversarial: a disk that "holds" because 10 τ_dyn at the wrong mass is too short to show the collapse —
  the 1.1 × case of G-CORE (M0-T67) is the counter-check.
- Handoff: <placeholder>

## M0-T67 · The cold ceiling holds (G-CORE) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T66)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.1 (K9), §2.3.2, §5.1 (G-CORE),
  §5.5 + docs/agent/testing.md
- Deliver: tests/gpu/cold_core.rs (new) with scenes/cold_co_disk.json (new): cold C/O disks at 0.95
  m_ch_sb (R₉₀ steady within 3 % over 10 τ_dyn) and 1.1 m_ch_sb (R₉₀ below 50 % of its start within 10
  τ_dyn), the GPU's m_ch within 5 % of the twin's, the pure-Σ² radius flat within 3 % over M ∈ {0.25,
  0.5, 1} × m_ch_sb — m_ch_sb from M0-T66's run → scope `cold-core` (G-CORE) → Verify 1 · plant
  tests/plants/cold-exponent.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py cold-core --task M0-T67` · Pass: GO, ≥ 4 cases · Fail: NO-GO —
  G-CORE is UNVERIFIED (§5.5): a red is a D, or a question when the sheet's ceiling itself is at stake ·
  2. `python3 tools/pb/verify.py --redarm cold-core --task M0-T67` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: m_ch_sb taken from the same run the case grades proves nothing — the 0.95 and 1.1 cases
  run fresh from the stored value.
- Handoff: <placeholder>

## M0-T68 · `calibrate` items 3–4 — the ignition and ending thresholds · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T67)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.3, §1.9.4, §2.10, §2.11 items 3–4,
  §6.4 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/calibrate.rs: m_ign_sb — the preset cloud at M (M0-T61) until it ignites
  or fails (d_c ≥ 0.5 and L_H < 0.01 L held 20 τ_dyn), bisection to 2 %, m_up_sb (white dwarf against
  core collapse) and m_bh_sb (neutron star against black hole) — preset clouds run `--until ending`,
  bisection to 2 %, each `--only`, each run's mass, ending, steps and wall time logged, the runs are whole
  lives, the lead's in M0-V9's window (§6.4) → Verify 1 · tests/gpu/calibrate_bisect.rs (new): the
  bisection driver on a stubbed outcome (its bracket, tolerance and run count), a refusal when the
  bracket's ends agree, a run's failure (no ending within --max-steps) reported, never guessed → scope
  `calibrate-bisect` → Verify 1 · plant tests/plants/bisect-tolerance.patch (new: stops at 20 %) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py calibrate-bisect --task M0-T68` · Pass: GO, ≥ 4 cases · Fail:
  NO-GO · 2. `python3 tools/pb/verify.py --redarm calibrate-bisect --task M0-T68` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: a bracket whose ends never reach an ending turns a bisection into hours of unmet runs — the
  driver refuses a bracket whose first runs end alike, and every run's ending is logged.
- Handoff: <placeholder>

## M0-T69 · Real-equivalent translations (G-READ) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T68)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.10.1–§1.10.3, §5.4 (G-READ) +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/observe/translate.rs (new): mass → Suns, piecewise log-linear through
  the four anchors (M_ign ↔ 0.08, M_Ch ↔ 1.44, M_up ↔ 8, M_BH ↔ 25), the end segments extended, and its
  inverse (M0-T70's preset masses), temperature → kelvins through §1.10.2's anchors, the O–Si segment
  extended above, 10 K below T_floor, a sandbox side that must increase strictly, T_eff from L = 2πRσT⁴,
  σ = a_r c_sb/4, translated (§1.10.3) → Verify 1 · tests/gpu/readouts.rs (new): both translations strictly
  increasing over their whole range, each anchor exact (10⁻⁹ relative), the inverse exact, the presets
  reading 1.00, 15.0 and 40.0 Suns and the Sun-like mid-main-sequence surface 5,772 K within 1 % — on a
  test calibration now, on assets/calibration.json once M0-V9 writes it → scope `readouts` (G-READ) →
  Verify 1 · plant tests/plants/anchor-reverse.patch (new, §5.4: two mass anchors swapped) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py readouts --task M0-T69` · Pass: GO, ≥ 6 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm readouts --task M0-T69` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: interpolation in linear space where §1.10 says log-linear passes at the anchors and errs
  between them — a midpoint case per segment.
- Handoff: <placeholder>

## M0-T70 · `calibrate` items 5–6 — the order, preset masses, the Sun's life, the top rung · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T69)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.2 (TOP), §1.10.1, §2.10, §2.11
  items 5–7, §5.5 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/calibrate.rs: the order (m_ign < m_ch < m_up < m_bh, m_ch < m_tov,
  m_tov/m_ch ∈ [1.4, 2.0]) — else exit 6, a K9 failure reported, never re-tuned in the run, preset_mass_sb
  for 1, 15 and 40 Suns through M0-T69's inverse, t_eff_sun_sb at the Sun-like preset's X_H,c = 0.35,
  life_sun_sb (drop → white_dwarf) and top_rung (life_sun_sb ÷ 10 s, two significant digits, §1.8.2),
  stage_durations_sb from the three preset lives, max_gas_speed over every run, exit 6 unless c_sb > 2 ×
  it, render_reserve_ms kept (item 7) → Verify 1 · tests/gpu/calibrate_order.rs (new): each inequality's
  refusal on a stubbed table, TOP's rounding, item 7 kept across a re-run → scope `calibrate-order` →
  Verify 1 · plant tests/plants/calibrate-order-skip.patch (new: m_tov/m_ch unchecked) → Verify 2 · the
  whole calibration — `sandbox-reactions calibrate --adapter "RTX 5090" --out assets/calibration.json`,
  likely hours — is the lead's, in M0-V9's window [NOT RUN — for you] → none — M0-V9 runs it
- Verify: 1. `python3 tools/pb/verify.py calibrate-order --task M0-T70` · Pass: GO, ≥ 6 cases · Fail:
  NO-GO · 2. `python3 tools/pb/verify.py --redarm calibrate-order --task M0-T70` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: a K9 failure re-tuned inside the run hides the physics problem the calibration exists to
  surface — exit 6 and the report, never a retry with other constants.
- Handoff: <placeholder>

## M0-T71 · Docs — calibrating, and lot 9's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T70)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.4, §1.10, §2.11, §6.4, §7 + the
  handoffs of M0-T65–M0-T70
- Deliver: docs/agent/running.md § Calibrating (the command, `--only`, the long runs and who runs them,
  the hash rule: change a constant, calibrate again) and docs/agent/physics.md § Calibration (what is
  measured and its bounds, pointing to assets/calibration.json) and docs/agent/readouts.md § Translations
  → Verify 1 · docs/agent/testing.md rows for calibration, calibrate-cold, cold-core, calibrate-bisect,
  readouts, calibrate-order → Verify 2
- Verify: 1. `grep -cE "^## (Calibrating|Calibration|Translations)" docs/agent/running.md
  docs/agent/physics.md docs/agent/readouts.md` · Pass: one per file · Fail: a file without its section ·
  2. `grep -cE "^[|] .(calibration|calibrate-|cold-core|readouts)" docs/agent/testing.md` · Pass: ≥ 6 ·
  Fail: fewer
- Adversarial: measured values copied into the docs go stale at the next calibration — the pages point
  to the file.
- Handoff: <placeholder>

## M0-V9 · Validation — lot 9: calibration · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T71)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §0.4, §1.6.4, §1.10, §2.6 (Σ_BH's bound), §2.11, §5.1 (G-CORE), §5.4
  (G-CAL, G-READ), §5.5, §6.4 + the handoffs of M0-T65–M0-T71 + docs/agent/running.md + the delivered
  files, each by the section it covers
- Deliver: the lead's runs owed in this window, one visible terminal each [NOT RUN — for you]: the whole
  calibration (`build/target/release/sandbox-reactions calibrate --adapter "RTX 5090" --out
  assets/calibration.json`, or item by item with `--only`, §6.4), then `python3 tools/pb/verify.py
  calibration cold-core readouts --task M0-V9` → none — the lead's run, read here · the result read:
  K9's order holding, c_sb > 2 × max_gas_speed, Σ_BH ≥ 4 × ns_sigma_max_sb (owed since M0-V7), every value
  inside its §2.11 bound — a red is a D, or a question to the lead when a tolerance or a FROZEN clause is at
  stake (§5.5), never a looser number (§0.4), the S1 and S2 triggers seen in the preset lives (§1.6.4)
  noted for M0-T97 and M0-TJ1 → reports/v9.md → Verify 1 · contract-vs-code on lot 9, each defect its
  own D (E4), every scope the lot added red-armed → Verify 1 · Phase 11's close (stubs, the header budget,
  the complete loop and its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V9` · Pass: GO, every scope present, readouts
  green on the real calibration · Fail: NO-GO — each red a D (E4)
- Done when: verdict recorded; D's created; register updated; assets/calibration.json written by the
  lead's run.
- Adversarial: a calibration measured on the 5090 and used on the Quadro — the V names the adapter it was
  measured on and leans on G-REF's cross-adapter tolerance (§6.7), not on identity.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 11 closes after M0-V9 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 11: calibration"`
> `git push`

# Phase 12 — Lot 10 · the readouts, the presets and the player's edits

## M0-T72 · The predicted ending · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V9)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.5, §1.10.1 + docs/agent/testing.md
- Deliver: crates/sr-physics/src/observe/ending.rs (new): from M in Suns (M0-T69) — below 0.08 failed
  star, below 8 white dwarf, below 25 supernova then neutron star, else black hole, within ±10 % in
  sandbox mass of a calibrated threshold both named (close call), a remnant stage held 10 t.u. → the
  ending instead of the prediction (§1.9.5) → Verify 1 · its tests: each band, both close-call edges at
  exactly ±10 % in sandbox mass, the switch to the ending at 10 t.u. → scope `ending` → Verify 1 · plant
  tests/plants/ending-band-swap.patch (new: M_up used where M_BH belongs, in the unit) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ending --task M0-T72` · Pass: GO, ≥ 7 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm ending --task M0-T72` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a close-call band computed in Suns instead of sandbox mass widens at the squeezed ends —
  §1.9.5 says sandbox mass, and the edge cases are taken there.
- Handoff: <placeholder>

## M0-T73 · The age clock · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T72)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.3 (the fuels), §1.10.4 +
  docs/agent/testing.md
- Deliver: crates/sr-physics/src/observe/age.rs (new): age += max(Δφ, 0)·τ_s(M_suns) each summary — φ
  from the stage's fuel (X_f,c from its start to 10⁻³) or the stage's calibrated duration (10 t.u. where
  unmeasured), remnants 1 × 10⁸ years per t.u., τ_s from §1.10.4's table log-log in mass and held beyond
  it, never decreasing, a mass change altering the rate only, the "1 s ≈ …" rate over the last wall
  second → Verify 1 · its tests: the table at 1, 5, 15 and 25 Suns, interpolation between, a mass jump
  keeping the age, Δφ < 0 clamped, the rate → scope `age-clock` → Verify 1 · plant
  tests/plants/age-unclamped-unit.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py age-clock --task M0-T73` · Pass: GO, ≥ 8 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm age-clock --task M0-T73` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a stage change resetting φ to the new stage's start without banking the old one makes the
  age jump back — the clock integrates Δφ, and a case changes stage mid-run.
- Handoff: <placeholder>

## M0-T74 · The three presets and several clouds (G-MULTI) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T73)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.10, §3.3 (`preset:`), §5.2
  (G-MULTI) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/scene.rs: the presets `sun`, `massive` and `giant` at calibration's
  preset_mass_sb through M0-T61's cloud, `preset:<name>` in scenes and in headless (§3.3) → Verify 1 ·
  tests/gpu/multi.rs (new) with scenes/two_suns.json (new): two Sun-like presets 240 cells apart, 20
  τ_dyn — two objects with stable ids, no non-finite value, both readouts filled → scope `multi` (G-MULTI)
  → Verify 1 · plant one-object (M0-T58's patch, §5.2) listed for `multi` → Verify 2
- Verify: 1. `python3 tools/pb/verify.py multi --task M0-T74` · Pass: GO, ≥ 3 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm multi --task M0-T74` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: two presets so close that their clouds overlap at the drop make one object by design — the
  scene's 240 cells against each cloud's radius is asserted before the run.
- Handoff: <placeholder>

## M0-T75 · Number formats · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T74)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §4.1 (units.*, time.*), §4.12 +
  docs/agent/testing.md
- Deliver: crates/sr-app/src/format.rs (new): mass (3 significant digits, `units.suns`), temperature
  (grouping below 10⁶ K, then million, then billion), age and rate (the largest unit whose value is ≥ 1,
  365.25 days a year), speeds (ladder rungs as written, TOP and the actual at 2 significant digits),
  inspector numbers (3 significant, scientific below 0.001 and at 100,000 and above), mix percentages,
  integers for fps and steps — each through the string table's templates → Verify 1 · its tests: every
  example §4.12 gives, byte-identical, and each boundary → scope `formats` (`cargo test -p sr-app --release
  --bin sandbox-reactions format::`) → Verify 1 · plant tests/plants/format-grouping.patch (new: no
  thousands separator) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py formats --task M0-T75` · Pass: GO, ≥ 15 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm formats --task M0-T75` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: rounding to 3 significant digits that crosses a unit boundary (999,999 K → "1,000,000 K")
  — the boundary cases pick the unit after rounding.
- Handoff: <placeholder>

## M0-T76 · Player edits on the GPU (P0) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T75)
- Status: TODO
- Carried flags: [M0-T21, 2026-10-10] P0's edit pass ORs 1 into Step::box_edit_flag() (one u32) when an edit lands outside the box, read from the ActiveBox uniform; box_gate, box_cells and box_fit already follow the edits in P0 and clear the flag (M0-T21).
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2 (P0), §1.12, §2.7 (paint_sigma,
  paint_t, heat_factor, cool_factor), §2.9 (painted, erased, preset_dropped, cleared, tools), §3.1
  (queue_edit) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/edit.rs and shaders/edit.wgsl (new): `queue_edit` applied at the
  next P0 — Paint {cell, radius, species} adding paint_sigma at paint_t at rest, composition mixing by
  mass, Erase to the floor state, Heat and Cool multiplying ε_th by heat_factor or cool_factor, floored at
  T_floor, DropPreset writing the preset's cloud centred on the cell, refused when a cell would fall
  outside the world (an error the app shows as `presets.no_room`), Clear (cells to the floor state, sinks
  removed, the ledger restarted, settings kept), the box re-fit when an edit lands outside it, each
  booked in §2.9's terms → Verify 1 · tests/gpu/edit.rs (new): each kind on a known field — mass and
  energy changes equal their booked terms to 10⁻⁶, momentum unchanged by paint, an edit on a hot dense
  field acting exactly as on gas (B7), a drop at the edge refused, the box growing to an edit outside it
  → scope `edits` → Verify 1 · plant tests/plants/edit-ignored.patch (new, §5.2's G-TOUCH plant: P0 drops
  edits on non-vacuum cells) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py edits --task M0-T76` · Pass: GO, ≥ 8 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm edits --task M0-T76` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: paint that adds mass at rest but keeps the cell's old momentum density speeds the cell up
  — the case checks momentum, not velocity.
- Handoff: <placeholder>

## M0-T77 · The edits file and `--edits` · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T76)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.12.5, §3.3 (`--edits`) +
  docs/agent/testing.md
- Deliver: crates/sr-engine/src/headless.rs: §2.12.5's list parsed — triggers `{"step": n}`,
  `{"object_until": {"x_h_c_below": v}}` and `{"mass_until": {"at_least" | "at_most": m}}` (repeating each
  step until passed), `frames` standing for frames of holding the tool — applied in P0 at the first step
  the trigger holds, read from the state at step boundaries, never the wall clock → Verify 1 ·
  tests/gpu/edits_file.rs (new) with scenes/edits_triggers.json (new): each trigger kind on a short run, a
  repeat stopping once its mass passes, two runs bit-identical → scope `edits-file` → Verify 1 · plant
  tests/plants/edits-trigger-late.patch (new: a trigger applied one step late) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py edits-file --task M0-T77` · Pass: GO, ≥ 4 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm edits-file --task M0-T77` · Pass: GO — the plant NO-GO · Fail:
  the plant stays GO
- Adversarial: a trigger read from a summary that lags two frames fires late and differs between runs —
  triggers read the state at the step boundary, and the two-run case proves it.
- Handoff: <placeholder>

## M0-T78 · Docs — readouts.md (the ending, the age), physics.md (edits) and lot 10's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T77)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.5, §1.10.4, §1.12, §2.12.5, §4.12,
  §7 + the handoffs of M0-T72–M0-T77
- Deliver: docs/agent/readouts.md § The predicted ending, § The age clock, § Number formats,
  docs/agent/physics.md § Player edits (P0, the edits file) → Verify 1 · docs/agent/testing.md rows for
  ending, age-clock, multi, formats, edits, edits-file → Verify 2
- Verify: 1. `grep -cE "^## (The predicted ending|The age clock|Number formats|Player edits)"
  docs/agent/readouts.md docs/agent/physics.md` · Pass: 3 in readouts.md, 1 in physics.md · Fail: fewer ·
  2. `grep -cE "^[|] .(ending|age-clock|multi|formats|edits|edits-file). [|]" docs/agent/testing.md` ·
  Pass: 6 · Fail: fewer
- Adversarial: the τ_s table copied into the page — the page points to §1.10.4.
- Handoff: <placeholder>

## M0-V10 · Validation — lot 10: the readouts, the presets and the edits · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T78)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.9.5, §1.10.4, §1.12, §2.10, §2.12.5, §4.12, §5.2 (G-MULTI) + the
  handoffs of M0-T72–M0-T78 + docs/agent/readouts.md, physics.md + the delivered files, each by the
  section it covers
- Deliver: contract-vs-code on lot 10 — the ending's bands and close call, the age clock against
  §1.10.4's table, the presets' masses read back as 1.00, 15.0 and 40.0 Suns on the real calibration, the
  number formats against every §4.12 example, the edits and their bookings, the edits file's triggers,
  each defect its own D (E4), every scope the lot added red-armed → reports/v10.md → Verify 1 · Phase 12's
  close (stubs, the header budget, the complete loop and its seconds, `rung_record.py report`'s table, a
  moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V10` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: presets checked on the test calibration only — the V reads them on assets/calibration.json.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 12 closes after M0-V10 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 12: readouts, presets and edits"`
> `git push`

# Phase 13 — Lot 11 · drawing, headless

## M0-T79 · The blackbody table · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V10)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.11 (the Glow view's colour) +
  docs/agent/testing.md
- Deliver: crates/sr-engine/src/render.rs (new, its CPU part): the 512-entry table over log T from 1,000 K
  to 40,000 K — Planck's B_λ integrated over 380–780 nm in 5 nm steps against Wyman, Sloan & Shirley's
  multi-lobe fit (§1.11's lobes verbatim), the IEC 61966-2-1 matrix, clamped ≥ 0, normalised to the
  largest channel, sRGB-encoded, the end colours outside → Verify 1 · its tests: c₂ = 1.438777 × 10⁷ nm·K
  from the exact SI constants, the blue-to-red ratio rising with T across the table, 6,500 K near white
  (each channel ≥ 0.9), 1,000 K red-dominant, 40,000 K blue-dominant, the fit's x̄ȳz̄ at 440, 555 and
  600 nm against the CIE 1931 values within 5 % → scope `blackbody` → Verify 1 · plant
  tests/plants/blackbody-matrix-transpose.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py blackbody --task M0-T79` · Pass: GO, ≥ 6 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm blackbody --task M0-T79` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a table checked only against itself (monotonic) can be uniformly wrong — the CIE values
  and 6,500 K's near-white are external anchors.
- Handoff: <placeholder>

## M0-T80 · The cell image and the four views · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T79)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.3 (α), §1.10.2, §1.11, §3.1
  (render), §4.11 (the maps) + docs/agent/testing.md
- Deliver: crates/sr-engine/src/render.rs (M0-T79's) and shaders/render.wgsl (new): a compute pass writing the
  W × H RGBA8 image from the state for the active view — Glow (blackbody(T_K)·b + haze·(1 − b)), Heat
  (§4.11's map over log₁₀ T_K from 1 to 10), Element (the species colours blended by mass fraction in
  linear RGB, times the visibility clamp), Density (§4.11's map over log₁₀ Σ from −6 to 4) — maps
  interpolated in linear RGB and clamped, T_K through M0-T69's translation, blended with the previous state
  by α in slow motion (§1.8.3), `render(view, alpha, target, encoder)` (§3.1) → Verify 1 ·
  tests/gpu/views.rs (new): known cells in each view against the CPU formula within 1/255 per channel,
  vacuum black, α = 0 and α = 1 giving the two states exactly → scope `views` → Verify 1 · plant
  tests/plants/views-heat-linear.patch (new: the heat map read in linear T) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py views --task M0-T80` · Pass: GO, ≥ 8 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm views --task M0-T80` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: maps blended in sRGB instead of linear RGB pass at the stops and drift between them — a
  midpoint case per map.
- Handoff: <placeholder>

## M0-T81 · The glow · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T80)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.11 (Glow), §4.11 (glow) +
  docs/agent/testing.md
- Deliver: crates/sr-engine/src/render.rs (M0-T79's) and shaders/glow.wgsl (new): the bright pass on luminance > 0.8
  at half resolution, a separable 9-tap Gaussian (σ = 3 px at half resolution), upsampled and added at
  0.6 — the Glow view only → Verify 1 · tests/gpu/glow.rs (new): a single bright cell's halo against the
  Gaussian's weights within 1/255, nothing added below the threshold, the other views untouched → scope
  `glow` → Verify 1 · plant tests/plants/glow-threshold.patch (new: threshold 0.8 → 0) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py glow --task M0-T81` · Pass: GO, ≥ 3 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm glow --task M0-T81` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a glow applied in every view tints the Heat and Density maps — the other-views case.
- Handoff: <placeholder>

## M0-T82 · Frames from headless runs · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T81)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §3.3 (`--frames-every`, `--view`), §6.3
  + docs/agent/testing.md
- Deliver: crates/sr-engine/src/headless.rs: DIR/frames/f_<step>.png every N steps in the chosen view
  (glow by default, with the glow passes), rendered offscreen (§6.3) and written with `png` → Verify 1 ·
  tests/gpu/frames.rs (new): a 50-step run with `--frames-every 10` writes its PNGs at the right steps and
  W × H, each decodable, the Glow frame of a hot disk not all black and brighter at its centre than at its
  rim → scope `frames` → Verify 1 · plant tests/plants/frames-off-by-one.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py frames --task M0-T82` · Pass: GO, ≥ 3 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm frames --task M0-T82` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: frames read back before the GPU finished draw the previous step — the step in the file name
  matches the dump of the same step.
- Handoff: <placeholder>

## M0-T83 · Docs — architecture.md (rendering) and lot 11's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T82)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.11, §4.11, §7 + the handoffs of
  M0-T79–M0-T82
- Deliver: docs/agent/architecture.md § Rendering (the views, the glow, the blackbody table, the
  interpolation, headless frames) → Verify 1 · docs/agent/testing.md rows for blackbody, views, glow,
  frames → Verify 2
- Verify: 1. `grep -c "^## Rendering" docs/agent/architecture.md` · Pass: 1 · Fail: 0 · 2. `grep -cE
  "^[|] .(blackbody|views|glow|frames). [|]" docs/agent/testing.md` · Pass: 4 · Fail: fewer
- Adversarial: design tokens restated in the page drift from §4.11 — the page points to it.
- Handoff: <placeholder>

## M0-V11 · Validation — lot 11: drawing · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T83)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.8.3, §1.10.2, §1.11, §4.11 + the handoffs of M0-T79–M0-T83 +
  docs/agent/architecture.md + the delivered files, each by the section it covers
- Deliver: contract-vs-code on lot 11 — the blackbody table, each view's formula and map, the glow, the
  interpolation, headless frames, two table entries checked against an independent published blackbody
  colour list (its URL and access date in the report, within ~5 %), each defect its own D (E4), every scope
  the lot added red-armed, looks are NOT PROVEN (static capture) for motion → reports/v11.md → Verify 1 ·
  Phase 13's close (stubs, the header budget, the complete loop and its seconds, `rung_record.py report`'s
  table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V11` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: colours graded against the formula they were written from agree with its mistakes — the
  external blackbody list.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 13 closes after M0-V11 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 13: drawing"`
> `git push`

# Phase 14 — Lot 12 · the window: the world and its time

## M0-T84 · The world in the window — the frame loop · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V11)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.3, §1.11 (the integer scale),
  §3.1 (encode_frame, render, poll), §4.1 (status.line), §4.2 (centre, status bar) + docs/agent/running.md
  + docs/agent/testing.md
- Deliver: crates/sr-app/src/desktop.rs: each frame TimeControl's plan (M0-T24) → `encode_frame` →
  `render` of the active view into the world area at the integer scale s (s = 2 in the default window,
  §1.11), nearest filtering, centred on bg.space, `poll()` feeding `on_report`, the slow-motion
  interpolation α (§1.8.3), the status bar's `status.line` live (§4.12 integers), a new world empty, at ×1,
  running → Verify 1 · crates/sr-app/src/capture.rs: actions `preset` (the engine's DropPreset until the
  panel's own, M0-T90) and `steps` running the real loop → Verify 1 · tests/capture/world.json (new): the
  Sun-like preset dropped and 200 steps → the world area's centre not bg.space, the status line's steps per
  frame > 0 → scope `desktop` gains case `world` → Verify 1 · plant tests/plants/world-not-drawn.patch
  (new: the world area never rendered) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py desktop capture --task M0-T84` · Pass: GO, desktop 3 cases
  (window, launcher, world — M0-TJ3 renamed `xvfb`) · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm desktop --task M0-T84` ·
  Pass: GO — the plant NO-GO on `world` · Fail: the plant stays GO
- Adversarial: a loop that waits on `poll()` stalls every frame by the readback's latency — `poll()` never
  blocks (§3.1), and the case's fps at ×1 is printed.
- Handoff: <placeholder>

## M0-T85 · The top bar — time controls, the speed readout, the slow-down setting · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T84)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.2–§1.8.4, §4.1 (time.*), §4.2 (the
  top bar), §4.3, §4.10 (Space, ., 1–9, + or =, −), §4.11, §4.12 (speeds) + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/mod.rs and ui/topbar.rs (new): a pure view model of the bar computed from
  TimeControl — Pause/Play, Step, the rung buttons (`time.rung`, rungs above TOP never shown), the speed
  readout (`time.speed`, `time.speed_capped` under §1.8.4, `time.paused`), the `time.slowdown` checkbox,
  the selected rung in `accent`, every text from STRINGS and §4.12's formats — and the thin egui layer
  drawing it in desktop.rs, keys Space, ., 1…9, + or = and − (§4.10) → Verify 1 · capture actions `rung`
  and `slowdown` in capture.rs, and M0-T8's `pause` now through the bar, each a step of
  tests/capture/actions.json (new: one step per action built since M0-T8, each writing its PNG — later
  blocks append theirs) run as the `capture` scope's case `actions` [M0-TB] → Verify 1 · the view
  model's tests: each label byte-exact, the rungs above TOP hidden, the capped and paused readouts, each
  key mapped → scope `ui-topbar` (`cargo test -p sr-app --release --bin sandbox-reactions ui::topbar::`)
  → Verify 1 · plant tests/plants/topbar-above-top.patch (new: a rung above TOP shown) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-topbar capture --task M0-T85` · Pass: GO, ui-topbar ≥ 10
  cases, capture 3 cases (its `actions` case new) · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm ui-topbar --task M0-T85` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a view model tested while egui draws something else — the egui layer reads only the view
  model, and TV2's captures compare the two.
- Handoff: <placeholder>

## M0-T86 · The banner and the automatic slow-down in the app · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T85)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.5, §1.9.4, §4.1 (banner.*), §4.2
  (the banner), §4.3, §4.11 (banner tokens) + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/banner.rs (new) and its wiring in desktop.rs: the Tracker's events
  (M0-T59) and the latch's report (M0-T22) into TimeControl's `on_event`, the banner over the world's top
  centre after an automatic slow-down — `banner.ignition`, `banner.core_collapse` or `banner.supernova` by
  the event — shown until the player changes the rung (no auto-return, §1.8.5), in the banner tokens →
  Verify 1 · the view model's tests: each event's text and rung (×1, ×0.1), the banner gone on a rung
  change, the setting off → no slow-down and no banner → scope `ui-banner` → Verify 1 · plant
  tests/plants/banner-wrong-event.patch (new: supernova shown for core collapse) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-banner --task M0-T86` · Pass: GO, ≥ 6 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm ui-banner --task M0-T86` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: an event that arrives two frames late (the summary's lag) after the latch already slowed the
  frame shows a second banner — each event fires once per object (§1.9.4), and a case sends both.
- Handoff: <placeholder>

## M0-T87 · World actions — the edge choice and Clear world · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T86)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.12 (Clear world), §2.8 (D1), §4.1
  (edge.*, world.*), §4.2, §4.7, §4.10 (Delete) + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/topbar.rs: `edge.label` with `edge.leave` and `edge.bounce`
  (`set_edge_mode`, at any time — D1), `world.clear` opening the dialog (`world.clear_confirm`,
  `world.clear_yes`, `world.clear_no`) whose yes queues Clear (§1.12, settings kept), Delete opening it
  (§4.10), capture action `edge`, its step appended to tests/capture/actions.json [M0-TB] → Verify 1 · the
  view model's tests: the edge state both ways, the dialog open, confirm and cancel, Clear queued only on
  yes → scope `ui-world` → Verify 1 · plant tests/plants/clear-without-confirm.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-world capture --task M0-T87` · Pass: GO, ui-world ≥ 5 cases,
  capture's `actions` case running `edge` · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm ui-world --task M0-T87` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: Clear resetting the rung, the view or the edge mode breaks §1.12 ("settings kept") — the
  case reads all four after a Clear.
- Handoff: <placeholder>

## M0-T88 · Docs — running.md (the window) and time_control.md (the app's frame loop) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T87)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8, §4.2, §4.3, §4.10, §7 + the
  handoffs of M0-T84–M0-T87
- Deliver: docs/agent/running.md § The window (the layout so far, the keys), docs/agent/time_control.md §
  The frame loop in the app (the plan, the interpolation, the banner) → Verify 1 · docs/agent/testing.md
  rows for ui-topbar, ui-banner, ui-world, desktop's world case and capture's `actions` case → Verify 2
- Verify: 1. `grep -cE "^## (The window|The frame loop in the app)" docs/agent/running.md
  docs/agent/time_control.md` · Pass: one per file · Fail: a file without its section · 2. `grep -cE
  "^[|] .(ui-topbar|ui-banner|ui-world). [|]" docs/agent/testing.md` · Pass: 3 · Fail: fewer
- Adversarial: keys documented that the code does not bind — the V checks the page's table against
  §4.10 and the view models' cases.
- Handoff: <placeholder>

## M0-TV2 · UI/UX pass — Phase 14: the world and its time · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T88)
- Status: TODO
- Read: this file (rules + this task) + docs/agent/running.md + milestones/m0/m0_contrat.md §3.3 (capture),
  §4.2, §4.3, §4.7, §4.11 + PLAYBOOK.md §8 (TV) and §A.4 (review_page.py's section text)
- Deliver: capture every screen and state Phase 14 touched into images/tv2/ with the capture harness on a
  private display: a star running at ×1, paused, single-stepped, each rung shown and TOP's, the capped
  readout, the slow-down banner, the edge choice both ways, the clear dialog — a state lavapipe cannot
  reach in reasonable time captured in a visible window with the lead's yes (R4) or marked NOT PROVEN with
  the reason, look at every capture yourself and file a D for each misfit (E1, never a fix here) → Pass ·
  build the page and serve it (tools/pb/review_page.py serve) with only the captures that need the lead's
  eyes, each with your note, remarks verbatim in reports/review2.md, one triage pass (E10), the server
  stopped (§6) → Pass · motion, timing, frame rate and input feel NOT PROVEN (static capture) — the lead
  sees them at M0-V12's launcher run and at M0-TW → none — the page's notes
- Pass: every state captured or marked with its reason; every capture looked at — filed, flagged or passed
  with a one-line reason; page served; remarks file written; triage filed. Fail: a state missing without a
  reason, a capture neither filed, flagged nor passed, or a remark not traceable to a capture id.
- Adversarial: lavapipe draws what the Quadro draws only up to its own precision and speed — a state that
  differs on the real GPU (the glow, the capped readout's numbers) is flagged for M0-V12's launcher run,
  never passed on lavapipe's word.
- Handoff: <placeholder>

## M0-V12 · Validation — lot 12: the world and its time · **CHECK** · Opus 5.5, max · switch · (AFTER M0-TV2)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.8, §1.11, §4.1–§4.3, §4.7, §4.10 + the handoffs of M0-T84–M0-T88 and
  M0-TV2 + docs/agent/running.md, time_control.md + the delivered files, each by the section it covers
- Deliver: contract-vs-code on lot 12 — the frame loop against §1.8.3, the top bar against §4.2–§4.3, the
  banner against §1.8.5, the world actions against §4.7 and §2.8, each defect its own D (E4), every scope
  the lot added red-armed → reports/v12.md → Verify 1 · the human-run tier owed here: the lead runs
  start.sh on their own desktop (a visible window, the real GPU), drops nothing yet but watches the window
  open, pauses, steps, changes rungs, and says what they saw → none — the lead's words in reports/v12.md ·
  Phase 14's close (stubs, the header budget, the complete loop and its seconds, `rung_record.py report`'s
  table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V12` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: a frame loop proven on lavapipe only — the lead's launcher run is on the real GPU, and its
  fps lands in the report.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 14 closes after M0-V12 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 14: the world and its time in the window"`
> `git push`

# Phase 15 — Lot 13 · the window: tools, stars and cells

## M0-T89 · Tools and elements · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V12)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.5 (colours), §1.12, §4.1 (tools.*,
  element.*), §4.2 (left panel), §4.4, §4.10 (B, E, H, C, [, ]) + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/left.rs (new) and its wiring in desktop.rs: `tools.heading` — Brush,
  Eraser, Heat, Cool, one selected in `accent`, the `tools.brush_size` slider 1–40 (default 8), [ and ]
  ±2, B, E, H and C, `tools.element_heading` — the nine paintable elements with their §1.5 swatches
  (neutron matter never listed), Hydrogen by default, the left button applying the tool through
  `queue_edit` each frame it is held (M0-T76), the brush disk outlined → Verify 1 · capture actions
  `tool`, `element` and `paint`, their steps appended to tests/capture/actions.json [M0-TB] → Verify 1 ·
  the view model's tests: selection, keys, the slider's bounds, the list's order without neutron matter,
  one edit queued per frame with the right radius and species → scope `ui-tools` → Verify 1 · plant
  tests/plants/tools-lists-neutron.patch (new) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-tools capture --task M0-T89` · Pass: GO, ui-tools ≥ 8 cases,
  capture's `actions` case running the three · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm ui-tools --task M0-T89` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a held button that queues one edit per mouse event instead of per frame paints faster on a
  fast mouse — the case drives frames, not events.
- Handoff: <placeholder>

## M0-T90 · Presets in the window · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T89)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.12 (DropPreset), §4.1 (presets.*),
  §4.2, §4.5, §4.10 (Esc) + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/left.rs: `presets.heading` with the three buttons and their notes (small
  type), a button arming its preset, `presets.place_hint` under the world and the cloud's outline following
  the cursor, the next left click dropping it (DropPreset) or showing `presets.no_room` for 3 s, Esc
  disarming, the `preset` capture action switched to the panel's path (the `desktop` scope's `world`
  case drives it, M0-T84) → Verify 1 · the view model's tests: arm, place, refuse with its 3 s, Esc →
  scope `ui-presets` → Verify 1 · plant tests/plants/preset-no-disarm.patch (new: Esc ignored) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-presets desktop --task M0-T90` · Pass: GO, ui-presets ≥ 5
  cases, desktop's `world` case through the panel's path [M0-TB] · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm ui-presets --task M0-T90` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: the outline drawn at the nominal radius while the cloud is capped at 150 cells (§2.10)
  misleads the no-room check — the outline and the refusal use the same radius.
- Handoff: <placeholder>

## M0-T91 · Views in the window · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T90)
- Status: TODO
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §4.1 (views.*), §4.2 (views), §4.10
  (F1–F4) + crates/sr-app/src/ui/left.rs (M0-T89's shape)
- Deliver: crates/sr-app/src/ui/left.rs: the `views.heading` section — Glow, Heat, Element and Density,
  the selected one in `accent`, F1–F4 — setting the renderer's active view (M0-T80), with its view-model
  cases in the `ui-tools` scope (expected count raised in tools/pb/verify.json), crates/sr-app/src/
  capture.rs: action `view`, its one-line step appended to tests/capture/actions.json [M0-TB] → Verify 1
- Verify: 1. `python3 tools/pb/verify.py ui-tools capture --task M0-T91` · Pass: GO, ui-tools ≥ 11 cases,
  capture's `actions` case running `view` · Fail: NO-GO
- Adversarial: a view switch that also resets the brush or the selected element — the cases read the
  tools after a switch.
- Handoff: <placeholder>

## M0-T92 · The star readout and selection · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T91)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.3 (Supergiant), §1.9.5, §1.9.6,
  §4.1 (star.*, fate.*, ending.*, stage.*), §4.2 (right panel), §4.6, §4.12 + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/right.rs (new) and its wiring in desktop.rs: `star.heading` and one line
  each — the stage label (`stage.*`, the Supergiant rows when M ≥ M_up), `star.mass`, `star.fate` with
  `fate.*` or `star.ending` with `ending.*` once a remnant holds, `star.age`, `star.rate` (hidden while
  paused), `star.surface`, `star.core` — from the Tracker through M0-T69, M0-T72, M0-T73 and §4.12's
  formats, `star.none` with no object, `star.select_hint` with several, a right click selecting the object
  under the cursor, the heaviest by default (§1.9.6), capture action `select`, its step appended to
  tests/capture/actions.json [M0-TB] → Verify 1 · the view model's tests on synthetic objects: each line
  byte-exact, the Supergiant switch at M_up, the ending replacing the fate, the rate hidden when paused,
  none and the hint → scope `ui-star` → Verify 1 · plant tests/plants/star-rate-while-paused.patch (new)
  → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-star capture --task M0-T92` · Pass: GO, ui-star ≥ 9 cases,
  capture's `actions` case running `select` · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm ui-star --task M0-T92` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a selection kept by object index rather than id jumps to another star when the list
  reorders — selection holds the tracker's id.
- Handoff: <placeholder>

## M0-T93a · The engine's cell inspection — `inspect(cell)` · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T92)
- Status: TODO
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §3.1 (inspect, poll) +
  crates/sr-engine/src/state.rs (M0-T3's readback)
- Deliver: crates/sr-engine/src/state.rs: `inspect(cell)` — an asynchronous one-cell readback whose reply
  (Σ, u, T, X) arrives in a later `poll()`, never blocking (§3.1) — and tests/gpu/state.rs: the reply
  equal to the dump's cell and arriving within 2 frames, its cases joining the `state` scope (the expected
  count raised in tools/pb/verify.json) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py state --task M0-T93a` · Pass: GO, ≥ 6 cases · Fail: NO-GO
- Adversarial: a readback that maps its buffer synchronously stalls the frame — the case polls without
  waiting and counts the frames until the reply.
- Handoff: <placeholder>

## M0-T93b · The cell inspector · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T93a)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §4.1 (inspect.*), §4.6, §4.12
  (inspector numbers, percentages) + docs/agent/testing.md
- Deliver: crates/sr-app/src/ui/right.rs: `inspect.heading` and the cell under the cursor through
  M0-T93a's `inspect` — the mix (up to three species ≥ 0.5 % by mass, `inspect.mix_item` joined by
  `inspect.mix_sep`), `inspect.temperature`, `inspect.density`, `inspect.speed` with Mach = |u|/√(2Π/Σ),
  `inspect.vacuum`, `inspect.empty` outside the world (D6), capture action `hover`, its step appended to
  tests/capture/actions.json [M0-TB] → Verify 1 · the view model's tests: the mix's cut and order, Mach,
  vacuum, outside, §4.12's inspector numbers → scope `ui-inspect` → Verify 1 · plant
  tests/plants/inspect-mix-four.patch (new: four species listed) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py ui-inspect capture --task M0-T93b` · Pass: GO, ui-inspect ≥ 6
  cases, capture's `actions` case running `hover` · Fail: NO-GO
  · 2. `python3 tools/pb/verify.py --redarm ui-inspect --task M0-T93b` · Pass: GO — the plant NO-GO ·
  Fail: the plant stays GO
- Adversarial: a hover that sends a request every frame floods the readback queue — one request in
  flight at a time, the latest cell winning, and a case hovers every frame for 120 frames with the frame
  time printed.
- Handoff: <placeholder>

## M0-T94 · The web build's star — URL parameters and G-WEB's preset case · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T93b)
- Status: TODO
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §3.5 (URL parameters), §5.4 (G-WEB),
  §6.2.3 + docs/agent/testing.md
- Deliver: crates/sr-app/src/web.rs: the same window as the desktop's (panels included) in the canvas, and
  `preset=<sun|massive|giant>` dropped at the world's centre, `steps=<n>` run then paused, `view=…`,
  `rung=<k>` (§3.5) → Verify 1 · web/smoke.mjs: case `preset` — `/?preset=sun&steps=200` reaching "ready"
  within 30 s and the canvas's central 100 × 100 px not all black → the `web` scope's fourth case →
  Verify 1 · never-ready still red → Verify 2
- Verify: 1. `python3 tools/pb/verify.py web --task M0-T94` · Pass: GO, 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm web --task M0-T94` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: "not all black" passes on an empty canvas of bg.space (#05070D) — the case also reports
  whether the central pixels differ from bg.space; a stricter case is the lead's to grant (Changes by
  asking), and M0-V13 names the gap.
- Handoff: <placeholder>

## M0-T95 · The UI's cost — `--measure-ui` · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T94)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.3 (R_reserve), §2.11 item 7,
  §3.3 (Measure UI) + docs/agent/testing.md
- Deliver: crates/sr-app/src/desktop.rs and its flag in main.rs: `--measure-ui <frames>` — the desktop app
  with the Sun-like preset at ×1, the panels' and presentation's cost per frame measured, `SR-UI
  p95_ms=<x>` printed, render_reserve_ms = p95 + 0.5 written into the calibration file (outside
  physics_hash, so G-CAL holds) → Verify 1 · tests/measure/check.sh (new): under `xvfb-run` with lavapipe
  (a check of the flag, never the measurement — R4), N frames run, the line printed, the key written into a
  copy of calibration.json → scope `measure-ui` → Verify 1 · plant tests/plants/measure-no-write.patch
  (new) → Verify 2 · the real run — `build/target/release/sandbox-reactions --adapter "Quadro RTX 4000"
  --measure-ui 600`, a visible window on the lead's desktop — the lead's, in M0-V13's window (R4, §3.3) →
  none — M0-V13
- Verify: 1. `python3 tools/pb/verify.py measure-ui --task M0-T95` · Pass: GO, ≥ 2 cases · Fail: NO-GO ·
  2. `python3 tools/pb/verify.py --redarm measure-ui --task M0-T95` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: a reserve measured on lavapipe and written as the Quadro's would mislead every frame budget
  — the agent's run writes only a copy, and the real file's value names its adapter.
- Handoff: <placeholder>

## M0-T96 · Docs — running.md (tools, presets, captures, measuring the UI) and lot 13's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T95)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §3.3, §3.5, §4.4–§4.6, §7 + the
  handoffs of M0-T89–M0-T95
- Deliver: docs/agent/running.md § Tools and presets, § Captures (the full action list), § Measuring the
  UI, § The web build's parameters → Verify 1 · docs/agent/testing.md rows for ui-tools, ui-presets,
  ui-star, ui-inspect, measure-ui and web's preset case → Verify 2
- Verify: 1. `grep -cE "^## (Tools and presets|Captures|Measuring the UI)" docs/agent/running.md` · Pass:
  3 · Fail: fewer · 2. `grep -cE "^[|] .(ui-tools|ui-presets|ui-star|ui-inspect|measure-ui). [|]"
  docs/agent/testing.md` · Pass: 5 · Fail: fewer
- Adversarial: a capture action listed that capture.rs refuses — the page's list is checked against
  §3.3's and the harness's.
- Handoff: <placeholder>

## M0-TV3 · UI/UX pass — Phase 15: tools, stars and cells · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T96)
- Status: TODO
- Read: this file (rules + this task) + docs/agent/running.md + milestones/m0/m0_contrat.md §3.3 (capture),
  §4.2, §4.4–§4.6, §4.11, §6.2.3 + PLAYBOOK.md §8 (TV) and §A.4 (review_page.py's section text)
- Deliver: capture every screen and state Phase 15 touched into images/tv3/: each tool selected, brush
  sizes, the element list, a preset armed with its outline and hint, the no-room message, each view on the
  same star, the star readout at several stages and several stars with the hint, the inspector on a cell,
  on vacuum and outside the world, the web build with a star (web/smoke.mjs's capture, §6.2.3), a state
  lavapipe cannot reach in reasonable time captured in a visible window with the lead's yes (R4) or marked
  with its reason, look at every capture yourself and file a D per misfit (E1, never a fix here) → Pass ·
  the review page with only what needs the lead's eyes, remarks verbatim in reports/review3.md, one triage
  pass (E10), the server stopped (§6) → Pass · feel, motion and timing NOT PROVEN (static capture) → none —
  the page's notes
- Pass: every state captured or marked with its reason; every capture looked at — filed, flagged or passed
  with a one-line reason; page served; remarks file written; triage filed. Fail: a state missing without a
  reason, a capture neither filed, flagged nor passed, or a remark not traceable to a capture id.
- Adversarial: a readout captured at one stage hides the labels that change over a life — several
  stages, a Supergiant among them, and several stars for the hint.
- Handoff: <placeholder>

## M0-V13 · Validation — lot 13: tools, stars and cells · **CHECK** · Opus 5.5, max · switch · (AFTER M0-TV3)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.9.6, §1.12, §3.3, §3.5, §4.1, §4.2, §4.4–§4.6, §4.12, §5.4 (G-WEB)
  + the handoffs of M0-T89–M0-T96 and M0-TV3 + docs/agent/running.md + the delivered files, each by the
  section it covers
- Deliver: contract-vs-code on lot 13 — every panel string against §4.1 through G-STR and the view
  models, the tools' edits, the presets' arming and refusal, the views, the readout and the inspector, the
  web build's parameters, G-WEB's "not all black" read against bg.space and named if weak, each defect its
  own D (E4), every scope the lot added red-armed → reports/v13.md → Verify 1 · the lead's runs owed here,
  one at a time: `--measure-ui 600` on the Quadro in a visible window (render_reserve_ms written, R4) and
  start.sh — a preset dropped, painted on, inspected — with what they saw → none — the lead's words and
  the reserve in reports/v13.md · Phase 15's close (stubs, the header budget, the complete loop and its
  seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V13` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Done when: verdict recorded; D's created; register updated; render_reserve_ms measured on the Quadro.
- Adversarial: view models green while the drawn panel differs — the V compares TV3's captures with the
  view models' texts.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 15 closes after M0-V13 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 15: tools, stars and cells in the window"`
> `git push`

# Phase 16 — Lot 14 · a star's whole life, measured and tuned

## M0-T97 · The life probe — three presets' lives, measured · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V13)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.4 (the stand-ins' triggers),
  §1.9.3, §2.10, §5.2, §5.5, §6.4 + milestones/m0/reports/step_cost.md
- Deliver: milestones/m0/reports/lives.md (new): the three presets (sun, massive, giant) each run
  `--until ending` (or to `--max-steps`) on the RTX 5090 (physics only, §6.4) — each preset's sandbox
  mass, radius a, Σc and starting temperature recorded (M0-T98a and M0-T99a rebuild them as preset-free
  clouds, [M0-TB]), the displayed stage
  sequence against G-STAGES' subsequences, each stage's duration, the main sequence's τ_dyn, τ_KH and
  τ_nuc (G-SQUEEZE's), the closest bound gas to an edge (G-FIT's 16 cells), the stand-ins' triggers
  measured — S1: the Sun-like preset's unbound envelope 20 τ_dyn after entering he_shell (< 30 % enables),
  S2: the Massive preset's unbound envelope within 20 t.u. of forming neutron matter at f_dep = 0.1
  (< 50 % enables) — and two dumps kept in ~/.cache/sandbox-reactions/dumps/ (outside the tree, so
  red-arm copies read them too — §6.5's reasoning) with the commands that remake them: the Sun-like
  mid-main-sequence (G-FPS, G-THERMO) and the Massive iron core (G-LATCH, G-CONS), each number with its
  command, SHA and adapter, a run over 10 minutes the lead's [NOT RUN — for you], a life that stalls is the
  finding, with its stage → none — the record M0-TJ1, M0-T98a, M0-T99a, M0-V14 and lot 15's checks read
  (a BUILD block's report)
- Verify: 1. `grep -cE "^## (Sun-like|Massive|Giant|Stand-ins|Dumps)" milestones/m0/reports/lives.md` ·
  Pass: 5 · Fail: fewer, or a number without its command and adapter
- Adversarial: a life read from labels alone can look right while the physics is wrong — each stage
  claim carries the measured quantity its §1.9.3 row reads (X_c, d_c, L_g).
- Handoff: <placeholder>

## M0-TJ1 · Direction ruling — the stand-ins S1 and S2, only on a measured failure · **PLAN + LEAD answers** · Opus 5.5, max · switch · (AFTER M0-T97)
- Status: DEFERRED (wake: reports/lives.md measures §1.6.4's failure — S1's unbound envelope under 30 %, or S2's under 50 % at f_dep = 0.1)
- Ask (verbatim): Q3, the lead, 2026-10-08 11:23 EDT: "Yes, as a backup (Recommended)" (m0_contrat.md
  §1.6.4)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.4, §2.4.2, §2.5.4, §2.7 +
  milestones/m0/reports/lives.md + milestones/m0/reports/contract_rulings.md (Q3)
- Deliver: for each stand-in whose trigger fired, the measured failure put to the lead with Q3 quoted —
  enable it as §1.6.4 writes (Recommended), or keep tuning the primary mechanism — the answer verbatim in
  milestones/m0/reports/standins.md → none — the lead's words are the record · on yes, the in-place
  amendment (§0.3, tagged [M0-TJ1]): S1 — κ_dust and T_dust within a stated bound, S2 — f_dep's bound up to
  1.0, and the blocks it needs filed — the physics.json change, its recalibration in the next V's window,
  G-STAGES and G-END re-run — summary.json recording which is on (G-END) → Pass · not woken by M0-V17 →
  N/A with a pointer to reports/lives.md → none
- Pass: every fired trigger answered by the lead, the amendment tagged and its blocks filed, or N/A with
  its pointer. Fail: a stand-in enabled without a measured failure or without the lead's answer.
- Handoff: <placeholder>

## M0-T98a · K1's gap tuned — the clocks spread apart · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TJ1)
- Status: TODO
- Carried flags: [M0-TB, 2026-10-08] the one physics.json write makes assets/calibration.json stale (G-CAL): scopes that run named presets or read the real calibration are refused (exit 6) until M0-V14's recalibration — if your claim run's --changed set holds one, that red is G-CAL working: name it and ask the lead before the claim run (Changes by asking), never a looser check
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.4, §2.4 (a_r, c_sb, κ₀), §2.5.1
  (Q_H), §2.7, §2.11 (the refusal), §2.12.1 (overrides), §5.2 (G-SQUEEZE), §5.5 +
  milestones/m0/reports/lives.md + milestones/m0/reports/time_warp.md (the gap's scenarios)
- Deliver: K1's gap tuned within §2.4's and §2.5.1's bounds (a_r, c_sb, κ₀, Q_H — M0-TC's flag (d): the
  first full-life lot's target) toward G-SQUEEZE (§5.2: τ_KH/τ_dyn ≥ 10 and τ_nuc/τ_KH ≥ 10 on the Sun-like
  main sequence, the Massive life's stage durations H > He > C > {Ne, O} > Si), measured on scenes that
  name no preset — §2.10's cloud as `disk` objects at the masses and temperatures reports/lives.md
  recorded, each candidate value as the scene's `overrides` — so no run needs the calibration a change
  makes stale (§2.11, G-CAL); each candidate's effect on all three ratios and on the top speed's
  projection (reports/step_cost.md) appended to reports/lives.md § Tuning — K1, with its command, SHA and
  adapter, the chosen values then written once into assets/physics.json, a run over 10 minutes the
  lead's [NOT RUN — for you], no in-bounds value reaching the gap → BLOCKED with the measurements and a
  question to the lead (§5.5), never a looser number (§0.4) → Verify 1 · scenes/tune_k1_sun.json and
  scenes/tune_k1_massive.json (new: those measuring clouds) → Verify 1 · the recalibration the write
  forces → none — the lead's, in M0-V14's window, before lot 15's checks (M0-T98b)
- Verify: 1. `grep -cE "^## Tuning — K1" milestones/m0/reports/lives.md` · Pass: 1 — both ratios ≥ 10
  and the stage order measured at the chosen values, each with its command, SHA and adapter · Fail: 0, a
  ratio under 10, or a number without its command — or BLOCKED as the Deliver says
- Adversarial: a gap reached by pushing a constant to its bound breaks another clock — each candidate's
  effect on all three ratios and on the top speed's projection is recorded; a candidate tried on a preset
  run is refused once physics.json differs from the calibration (G-CAL) — candidates ride as overrides on
  preset-free clouds until the one final write.
- Handoff: <placeholder>

## M0-T99a · The mass ladder tuned — every life fits the world · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T98a)
- Status: TODO
- Carried flags: [M0-TB, 2026-10-08] the one physics.json write makes assets/calibration.json stale (G-CAL): scopes that run named presets or read the real calibration are refused (exit 6) until M0-V14's recalibration — if your claim run's --changed set holds one, that red is G-CAL working: name it and ask the lead before the claim run (Changes by asking), never a looser check
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.4, §1.10.1 (D5's squeezed ladder),
  §2.10, §2.11 (the refusal), §2.12.1 (overrides), §5.2 (G-FIT), §5.5 + milestones/m0/reports/lives.md
- Deliver: the squeezed mass ladder tuned within the contract's bounds — the levers §1.10.1 and §2.10
  name — toward G-FIT (§5.2: through each preset's life no bound gas within 16 cells of an edge of the
  default world), measured on scenes that name no preset (§2.10's cloud as `disk` objects at the masses
  and temperatures reports/lives.md recorded, candidates as `overrides`), so no run needs a stale
  calibration (§2.11); each candidate's closest bound approach per life appended to reports/lives.md
  § Tuning — the mass ladder, with its command, SHA and adapter, the chosen values written once into
  assets/physics.json, a run over 10 minutes the lead's [NOT RUN — for you], a lever outside the bounds
  → BLOCKED and a question to the lead → Verify 1 · scenes/tune_fit_sun.json, tune_fit_massive.json and
  tune_fit_giant.json (new: the three measuring clouds) → Verify 1 · the recalibration → none — the
  lead's, in M0-V14's window, before M0-T99b
- Verify: 1. `grep -cE "^## Tuning — the mass ladder" milestones/m0/reports/lives.md` · Pass: 1 — each
  life's closest bound gas ≥ 16 cells from an edge at the chosen values, with its command, SHA and adapter
  · Fail: 0, a life under 16 cells, or a number without its command — or BLOCKED as the Deliver says
- Adversarial: unbound ejecta near the edge counted as a fit failure — the measurement reads bound gas
  only (½|u|² + ε_th + φ < 0); a lever that is a contract formula (§2.10's radius) rather than a
  physics.json key changes only by asking the lead (Changes by asking, §0.3), never in place here.
- Handoff: <placeholder>

## M0-T102 · Docs — physics.md (the tuning record) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T99a)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.4, §2.5.1, §2.7, §7 +
  milestones/m0/reports/lives.md + the handoffs of M0-T97, M0-T98a and M0-T99a (and M0-TJ1's if woken)
- Deliver: docs/agent/physics.md § Tuning — each constant moved from its initial value, why, and the
  scope that will hold it in lot 15 (squeeze, fit), pointing to physics.json and reports/lives.md →
  Verify 1 · the four whole-life scopes' testing rows → none — M0-T108 writes them with the scopes
  (lot 15, [M0-TB])
- Verify: 1. `grep -c "^## Tuning" docs/agent/physics.md` · Pass: 1 · Fail: 0
- Adversarial: the tuned values written into the page — it points to physics.json (§7).
- Handoff: <placeholder>

## M0-V14 · Validation — lot 14: a star's whole life, measured and tuned · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T102)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §0.4, §1.6.4, §2.11, §5.2 (G-SQUEEZE, G-FIT), §5.5 +
  milestones/m0/reports/lives.md + the handoffs of M0-T97, M0-T98a, M0-T99a and M0-T102 (and M0-TJ1's if
  woken) + docs/agent/physics.md + the delivered files, each by the section it covers
- Deliver: the lead's runs owed here, one visible terminal each [NOT RUN — for you]: the recalibration the
  tuning forced (`calibrate`, or `--only` per item, §6.4), then the two dumps of reports/lives.md § Dumps
  remade on the tuned physics by their recorded commands (lot 15's G-THERMO and G-CONS, and lot 16's
  G-LATCH and G-FPS, load them) → none — the lead's runs, read here · contract-vs-code on lot 14 — every
  tuned constant inside its bound, each tuning section's numbers against its recorded command, the
  stand-ins' state against M0-TJ1, each defect its own D (E4), every UNVERIFIED red a D or a question
  (§5.5) → reports/v14.md → Verify 1 · Phase 16's close (stubs, the header budget, the complete loop and
  its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V14` · Pass: GO, every scope present, the
  presets' scopes green on the new calibration · Fail: NO-GO — each red a D (E4)
- Done when: verdict recorded; D's created; register updated; assets/calibration.json matching the tuned
  physics (G-CAL GO) and both dumps remade.
- Adversarial: a tuning section whose numbers no recorded command reproduces reads as tuned — the V
  re-runs one candidate's command per section; the whole-life checks come after this window (lot 15,
  [M0-TB]), so a calibration taken before the last physics.json write is caught here by G-CAL.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 16 closes after M0-V14 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 16: a star's whole life, measured and tuned"`
> `git push`

# Phase 17 — Lot 15 · a whole life checked: stages, endings, the clocks, the fit, equilibrium
Ordering note (E8 — M0-TB, 2026-10-08, the lead's "Tune first, check after (Recommended)"): M0-T98b,
M0-T99b, M0-T100 and M0-T101 run in this lot, after M0-V14's recalibration, because their checks run the
named presets, which the engine refuses once lot 14's tuning changes physics.json (G-CAL) until that run;
M0-T98b and M0-T99b were filed here, M0-T100 and M0-T101 moved here from lot 14 by `plan.py move`.

## M0-T103 · The visible life (G-STAGES) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V14)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.3, §5.2 (G-STAGES), §5.5 +
  milestones/m0/reports/lives.md + docs/agent/testing.md
- Deliver: tests/gpu/stages.rs (new): the displayed stage ids of each preset's life contain §5.2's
  subsequences — the Sun-like's down to white_dwarf, the Massive's down to neutron_star with its h_shell and
  he_core labels the Supergiant ones, the Giant's down to black_hole → scope `stages` ⏱ (one life per case)
  → Verify 1 · plant tests/plants/no-push.patch (new, §5.2: the radiation force zeroed) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py stages --task M0-T103` · Pass: GO, 3 cases — on the RTX 5090,
  else the lead's [NOT RUN — for you] · Fail: NO-GO — UNVERIFIED (§5.5): a red is a D, a question, or
  M0-TJ1's trigger · 2. `python3 tools/pb/verify.py --redarm stages --task M0-T103` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: a subsequence check that passes on labels flickering through every stage — the displayed
  stage needs 3 identical summaries (§1.9.3), and the case reads displayed stages, not raw matches.
- Handoff: <placeholder>

## M0-T104 · Each ending as predicted (G-END) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T103)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9.5, §2.12.2, §5.2 (G-END), §5.5,
  §6.4 + docs/agent/testing.md
- Deliver: tests/gpu/endings.rs (new): the three presets ending as predicted at the drop, preset-shaped
  clouds at each calibrated threshold ÷ 1.25 and × 1.25 in sandbox mass ending on their side — failed star
  against white dwarf, white dwarf against neutron star, neutron star against black hole → scope `endings`
  ⏱ (one life per case, §6.4) → Verify 1 · crates/sr-engine/src/headless.rs: summary.json recording whether
  S1 or S2 was enabled (§5.2) → Verify 1 · plant tests/plants/fate-swap.patch (new, §5.2) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py endings --task M0-T104` · Pass: GO, 9 cases — on the RTX 5090,
  else the lead's [NOT RUN — for you] · Fail: NO-GO — UNVERIFIED (§5.5): a red is a D or a question · 2.
  `python3 tools/pb/verify.py --redarm endings --task M0-T104` · Pass: GO — the plant NO-GO · Fail: the
  plant stays GO
- Adversarial: an ending read before the remnant held 10 t.u. counts a passing stage as the end — the
  case uses `--until ending` (§3.3).
- Handoff: <placeholder>

## M0-T105 · Touch any time (G-TOUCH) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T104)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.12, §2.12.5, §5.2 (G-TOUCH), §5.5 +
  docs/agent/testing.md
- Deliver: tests/gpu/touch.rs (new) with scenes/touch_paint.json and scenes/touch_erase.json (new edits
  files): hydrogen painted onto the Sun-like preset at X_H,c = 0.5 until it weighs 1.25 × m_up_sb — it then
  ends as a neutron star or a black hole, as its new prediction says, the Massive preset at X_H,c = 0.5
  erased from outside in down to m_up_sb ÷ 1.25 — it then ends as a white dwarf → scope `touch` ⏱ →
  Verify 1 · plant edit-ignored (M0-T76's patch, §5.2) listed for `touch` → Verify 2
- Verify: 1. `python3 tools/pb/verify.py touch --task M0-T105` · Pass: GO, 2 cases — on the RTX 5090, else
  the lead's · Fail: NO-GO — UNVERIFIED (§5.5): a red is a D or a question · 2. `python3
  tools/pb/verify.py --redarm touch --task M0-T105` · Pass: GO — the plant NO-GO · Fail: the plant stays
  GO
- Adversarial: a prediction re-read at the drop instead of after the edit passes by luck — the case
  records the prediction after the edit and checks the ending against it.
- Handoff: <placeholder>

## M0-T106 · Conservation over whole lives (G-CONS) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T105)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.4, §2.9, §5.1 (G-CONS), §5.5 +
  milestones/m0/reports/lives.md (the dumps) + docs/agent/testing.md
- Deliver: tests/gpu/conservation.rs (new): a whole Sun-like life in bounce and a supernova in leave (the
  Massive preset from its iron-core dump) — the mass invariant constant to 10⁻⁶ relative, momentum to 10⁻⁶
  of Σm|u|, the energy invariant drifting ≤ 10⁻³ |W| per τ_dyn → scope `conservation` ⏱ → Verify 1 · plant
  tests/plants/no-grav-work.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py conservation --task M0-T106` · Pass: GO, ≥ 4 cases — on the RTX
  5090, else the lead's · Fail: NO-GO — energy is UNVERIFIED: more drift is a D for a conservative gravity
  formulation, never a looser bound (§0.4) · 2. `python3 tools/pb/verify.py --redarm conservation --task
  M0-T106` · Pass: GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: a missing booked term (escaped, swallowed) shows as drift blamed on gravity — the report
  lists each term's size beside the drift.
- Handoff: <placeholder>

## M0-T107 · The age clock over a touched life (G-AGE) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T106)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.10.4, §2.12.5, §5.4 (G-AGE) +
  docs/agent/testing.md
- Deliver: tests/gpu/age.rs (new) with scenes/age_touch.json (new edits file): the Massive preset with
  +50 % hydrogen painted onto its core at X_H,c = 0.5 and 30 % of its envelope erased later — the age never
  decreases and no summary adds more than 10 × its stage's median increment, the Sun-like preset reaching
  white_dwarf reading within a factor 1.5 of 1.21 × 10¹⁰ years → scope `age` ⏱ → Verify 1 · plant
  tests/plants/phi-unclamped.patch (new, §5.4) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py age --task M0-T107` · Pass: GO, ≥ 3 cases — on the RTX 5090, else
  the lead's · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm age --task M0-T107` · Pass: GO — the
  plant NO-GO · Fail: the plant stays GO
- Adversarial: the 1.21 × 10¹⁰ figure hit by a clock that ignores the measured stage durations — the case
  also prints each stage's share of the age.
- Handoff: <placeholder>

## M0-T98b · The clocks' order (G-SQUEEZE) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T107)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §5.2 (G-SQUEEZE), §5.5, §6.4 +
  milestones/m0/reports/lives.md (§ Tuning — K1) + docs/agent/testing.md
- Deliver: tests/gpu/squeeze.rs (new): on the presets and the calibration M0-V14's run wrote — the Sun-like
  main sequence's τ_KH/τ_dyn ≥ 10 and τ_nuc/τ_KH ≥ 10 (§5.2's definitions, measured), the Massive preset's
  stage durations H > He > C > {Ne, O} > Si, the main-sequence durations Sun-like > Massive > Giant →
  scope `squeeze` ⏱ (paths: its test and scenes) → Verify 1 · plant tests/plants/q-h-tiny.patch (new,
  §5.2) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py squeeze --task M0-T98b` · Pass: GO, ≥ 4 cases — on the RTX 5090,
  else the lead's [NOT RUN — for you] · Fail: NO-GO — G-SQUEEZE is UNVERIFIED (§5.5): a red is a D, or a
  question when the gap itself is at stake, never a looser number · 2. `python3 tools/pb/verify.py
  --redarm squeeze --task M0-T98b` · Pass: GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: a run on a calibration older than the last physics.json write is refused (G-CAL) and can
  pass for a physics red — the case prints the calibration's physics_hash beside its verdict; ratios read
  off M0-T98a's preset-free clouds would grade the tuning by itself — the case runs the presets.
- Handoff: <placeholder>

## M0-T99b · Every life fits the world (G-FIT) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T98b)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §5.2 (G-FIT), §5.5, §6.4 +
  milestones/m0/reports/lives.md (§ Tuning — the mass ladder) + docs/agent/testing.md
- Deliver: tests/gpu/fit.rs (new): on the presets and the calibration M0-V14's run wrote, through each
  preset's life no bound gas (½|u|² + ε_th + φ < 0) within 16 cells of an edge of the default world →
  scope `fit` ⏱ → Verify 1 · plant tests/plants/preset-wide.patch (new, §5.2: preset radii × 3) →
  Verify 2
- Verify: 1. `python3 tools/pb/verify.py fit --task M0-T99b` · Pass: GO, 3 cases (one life each) — on the
  RTX 5090, else the lead's [NOT RUN — for you] · Fail: NO-GO — G-FIT is UNVERIFIED (§5.5, a tuning
  target): a red is a D, or a question to the lead, never a looser number · 2. `python3
  tools/pb/verify.py --redarm fit --task M0-T99b` · Pass: GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: unbound ejecta near the edge counted as a fit failure — the check reads bound gas only;
  the recalibration moves the presets' masses (§1.10.1's inverse) and can eat the tuning's margin — the
  case prints each life's closest approach beside M0-T99a's.
- Handoff: <placeholder>

## M0-T100 · Virial equilibrium (G-VIR) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T99b)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §5.1 (G-VIR), §5.5 +
  docs/agent/testing.md
- Deliver: tests/gpu/virial.rs (new): the Sun-like preset 20 τ_dyn after ignition, |(2K + 2∫Π dA)/W + 1|
  ≤ 3 % → scope `virial` ⏱ → Verify 1 · plant tests/plants/pressure-scale.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py virial --task M0-T100` · Pass: GO, 1 case — on the RTX 5090, else
  the lead's · Fail: NO-GO — G-VIR is UNVERIFIED (§5.5): a red is a D, or a question when the tolerance is
  at stake · 2. `python3 tools/pb/verify.py --redarm virial --task M0-T100` · Pass: GO — the plant NO-GO ·
  Fail: the plant stays GO
- Adversarial: W summed without the sinks or with the wrong half (½Σmφ) — the case recomputes W from the
  dump against the summary's.
- Handoff: <placeholder>

## M0-T101 · The thermostat (G-THERMO) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T100)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §5.1 (G-THERMO), §5.5 +
  milestones/m0/reports/lives.md (the dumps) + docs/agent/testing.md
- Deliver: tests/gpu/thermostat.rs (new): the Sun-like preset at X_H,c = 0.5, a +10 % kick to the central
  5 × 5 cells' ε_th — T_c back within 2 % of its prior value inside 3 τ_KH (measured), never above 1.5 ×
  → scope `thermostat` ⏱ → Verify 1 · plant tests/plants/burn-heat-to-ledger.patch (new, §5.1) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py thermostat --task M0-T101` · Pass: GO, ≥ 2 cases — on the RTX
  5090, else the lead's · Fail: NO-GO — UNVERIFIED (§5.5): a red is a D or a question · 2. `python3
  tools/pb/verify.py --redarm thermostat --task M0-T101` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a kick that the next floors pass or the box re-fit erases tests nothing — the kicked
  energy is read back one step after the kick.
- Handoff: <placeholder>

## M0-T108 · Docs — readouts.md (the stages over a life) and lot 15's testing rows · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T101)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.9, §5.1 (G-VIR, G-THERMO), §5.2, §7 +
  the handoffs of M0-T103–M0-T107, M0-T98b, M0-T99b, M0-T100 and M0-T101
- Deliver: docs/agent/readouts.md § Stages over a life (pointing to reports/lives.md and the scopes) →
  Verify 1 · docs/agent/testing.md rows for stages, endings, touch, conservation, age, squeeze, fit,
  virial and thermostat, each marked ⏱ with who runs it → Verify 2
- Verify: 1. `grep -c "^## Stages over a life" docs/agent/readouts.md` · Pass: 1 · Fail: 0 · 2. `grep -cE
  "^[|] .(stages|endings|touch|conservation|age|squeeze|fit|virial|thermostat). [|]"
  docs/agent/testing.md` · Pass: 9 · Fail: fewer
- Adversarial: a page claiming every stage when the probe saw one missing — the page reads the scopes'
  last verdicts.
- Handoff: <placeholder>

## M0-V15 · Validation — lot 15: a whole life checked · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T108)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.9, §1.10.4, §2.9, §5.1 (G-CONS, G-VIR, G-THERMO), §5.2 (G-STAGES,
  G-END, G-TOUCH, G-SQUEEZE, G-FIT), §5.4 (G-AGE), §5.5 + the handoffs of M0-T103–M0-T107, M0-T98b,
  M0-T99b, M0-T100, M0-T101 and M0-T108 + docs/agent/readouts.md + the delivered files, each by the
  section it covers
- Deliver: the lead's runs owed here, one visible terminal each [NOT RUN — for you]: `python3
  tools/pb/verify.py stages endings touch conservation age squeeze fit virial thermostat --task M0-V15`
  and their red-arms, wherever over 10 minutes → none — the lead's runs, read here · contract-vs-code on
  lot 15 — the three lives' stages, the nine endings, touch, conservation, the age clock, the clocks'
  order, the fit, the virial and the thermostat on the tuned physics, each defect its own D (E4), every
  UNVERIFIED red a D or a question (§5.5), summary.json's stand-in record read against M0-TJ1 →
  reports/v15.md → Verify 1 · Phase 17's close (stubs, the header budget, the complete loop and its
  seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V15` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4)
- Adversarial: an all-green table on UNVERIFIED rows is itself a finding (§8) — the V reports what each
  ending's run measured, not only its verdict.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 17 closes after M0-V15 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 17: a whole life checked — stages, endings, clocks, fit and equilibrium"`
> `git push`

# Phase 18 — Lot 16 · speed and frame rate

## M0-T109 · The bench · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V15)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8, §3.3 (bench), §5.3 (G-FPS,
  G-TOP), §6.1 + docs/agent/testing.md
- Deliver: crates/sr-engine/src/bench.rs (new) and its subcommand in crates/sr-app/src/main.rs:
  `sandbox-reactions bench --scene … --rung <label|top> (--frames N | --until <condition>) --adapter S
  [--slowdown on|off] [--load-dump FILE] --out DIR` — §1.8's frame loop (TimeControl, N_max from c_step and
  render_reserve_ms, the 30-frames switch at TOP) with every frame rendered offscreen at 1200 × 800 in the
  Glow view with glow, no vsync, DIR/bench.json per §3.3 (life_wall_s = Σ max(frame_ms, 1000/f_target)/1000)
  → Verify 1 · tests/gpu/bench.rs (new): a short bench writes bench.json with every key, the frame count
  right, life_wall_s per its formula from the frame list, the switch counted → scope `bench` → Verify 1 ·
  plant tests/plants/bench-no-render.patch (new: frames not rendered) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py bench --task M0-T109` · Pass: GO, ≥ 4 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm bench --task M0-T109` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: a bench that skips rendering reads fast frames the window never gets — the plant removes
  rendering and the frame-cost case must see it.
- Handoff: <placeholder>

## M0-T110 · Warps never touch the physics (G-WARP) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T109)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.4, §1.8.7, §5.3 (G-WARP) +
  docs/agent/testing.md
- Deliver: tests/gpu/warp.rs (new): 2,000 steps of the Sun-like preset at ×0.1, ×1 and TOP, the slow-down
  on and off, through the bench's frame loop — bit-identical dumps on one adapter → scope `warp` → Verify 1
  · plant tests/plants/dt-from-frame.patch (new, §5.3) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py warp --task M0-T110` · Pass: GO, ≥ 5 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm warp --task M0-T110` · Pass: GO — the plant NO-GO · Fail: the plant
  stays GO
- Adversarial: dumps compared after different step counts (a frame overshooting 2,000) — every run stops
  at exactly step 2,000.
- Handoff: <placeholder>

## M0-T111 · The collapse caught within one step — G-LATCH's scenario · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-T110)
- Status: TODO
- Sizing: E
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.5, §4.3, §5.3 (G-LATCH) +
  milestones/m0/reports/lives.md (the iron-core dump) + docs/agent/testing.md
- Deliver: tests/gpu/latch.rs: the contract case — the Massive preset from M0-T97's iron-core dump in
  ~/.cache/sandbox-reactions/dumps/ (remade by its recorded command when absent — a whole massive life,
  the lead's when over 10 minutes) at TOP, the slow-down on: no step runs after the first step whose
  N_Fe rate is > 0 within that frame, and the next frame runs at ×0.1 with `banner.core_collapse` (the
  speed and banner text from TimeControl and the banner's view model) → `latch` gains it, G-LATCH's NOT
  PROVEN (synthetic) lifted → Verify 1 · latch-off red on it → Verify 2
- Verify: 1. `python3 tools/pb/verify.py latch --task M0-T111` · Pass: GO, ≥ 6 cases · Fail: NO-GO · 2.
  `python3 tools/pb/verify.py --redarm latch --task M0-T111` · Pass: GO — the plant NO-GO on the contract
  case · Fail: the plant stays GO
- Adversarial: a dump past the collapse's first step catches nothing — the case asserts N_Fe is off at the
  dump's step and turns on within the frame.
- Handoff: <placeholder>

## M0-T112 · The frame rate (G-FPS) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T111)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.6, §5.3 (G-FPS), §6.1, §6.4 +
  milestones/m0/reports/lives.md (the mid-main-sequence dump) + docs/agent/testing.md
- Deliver: tests/gpu/fps.rs (new): `bench` on linux-pc, adapter "Quadro RTX 4000", the Sun-like preset —
  from the mid-main-sequence dump, 600 frames at each rung below TOP with p95 ≤ 16.7 ms, the whole life at
  TOP with p95 ≤ 16.7 ms, or ≤ 33.3 ms once §1.8.6's switch engaged, the slowest frame ≤ 50 ms at any rung
  → scope `fps` ⏱ (`box: laserax-ai`, never the RTX 5090) → Verify 1 · plant tests/plants/no-clamp.patch
  (new, §5.3) → Verify 2 · a red: the per-pass costs handed on, M0-T114 woken when P2's share warrants →
  none — M0-T114's wake
- Verify: 1. `python3 tools/pb/verify.py fps --task M0-T112` · Pass: GO, ≥ 9 cases, each frame figure with
  the Quadro named — the lead's run when over 10 minutes · Fail: NO-GO · 2. `python3 tools/pb/verify.py
  --redarm fps --task M0-T112` · Pass: GO — the plant NO-GO · Fail: the plant stays GO
- Adversarial: a bench on the 5090 passing for the mid-range card — the case refuses any adapter but the
  Quadro.
- Handoff: <placeholder>

## M0-T113 · A Sun-like life in about 10 s (G-TOP) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T112)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.2, §1.8.6, §5.3 (G-TOP), §6.4 +
  docs/agent/testing.md
- Deliver: tests/gpu/top.rs (new): the bench at TOP on the Quadro until stage:white_dwarf — life_wall_s ≤ 12
  → scope `top-speed` ⏱ (`box: laserax-ai`, the Quadro) → Verify 1 · plant tests/plants/owed-half.patch
  (new, §5.3) → Verify 2 · red at 30 frames → M0-TJ2 wakes and the build stops for the lead (§1.8.6) →
  none — M0-TJ2
- Verify: 1. `python3 tools/pb/verify.py top-speed --task M0-T113` · Pass: GO, 1 case, life_wall_s printed
  · Fail: NO-GO → M0-TJ2 · 2. `python3 tools/pb/verify.py --redarm top-speed --task M0-T113` · Pass: GO —
  the plant NO-GO · Fail: the plant stays GO
- Adversarial: a top rung computed from an old calibration after tuning — G-CAL's hash check holds
  calibration and physics together.
- Handoff: <placeholder>

## M0-TJ2 · Direction ruling — the top speed, if it cannot hold at 30 frames · **PLAN + LEAD answers** · Opus 5.5, max · switch · (AFTER M0-T113)
- Status: DEFERRED (wake: G-TOP red at 30 frames on the Quadro (M0-T113), or reports/step_cost.md projecting over 33.3 ms a frame — M0-V8 moves this block up then)
- Ask (verbatim): Q2, the lead, 2026-10-08 11:23 EDT: "30 pictures/s at top speed (Recommended)"; §1.8.6:
  "If even that is not enough, we come back to you"
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8.6, §5.3 (G-TOP) +
  milestones/m0/reports/step_cost.md + milestones/m0/reports/time_warp.md R6, R8 (F1) +
  milestones/m0/reports/sandbox_interview.md B28
- Deliver: §1.8.6's question to the lead with the measured numbers — the unruled options: projective
  jumps (TW R6: a second integrator and its tests), a slower top speed (re-opens answer 6 — B28's "a life
  in ~1 min" was not picked), or the build kept as it is — the answer verbatim in
  milestones/m0/reports/top_speed.md → none — the lead's words are the record · the amendment the ruling
  calls for (§0.3, tagged [M0-TJ2]) and its blocks filed → Pass · not woken by M0-V17 → N/A, G-TOP's GO
  cited → none
- Pass: the lead's answer recorded, the amendment tagged and its blocks filed, or N/A with G-TOP's GO.
  Fail: a slower top speed or shortcut jumps built without the lead's answer — never a silent fallback
  (§1.8.6).
- Handoff: <placeholder>

## M0-T114 · The gravity cadence (G-CAD), only if the budget needs it · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TJ2)
- Status: DEFERRED (wake: G-FPS or G-TOP red with P2's share of the step large enough, by per_pass_ms, that k = 4 brings the frame under budget — or the lead's word)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.4.4, §1.8.5, §5.3 (G-CAD) +
  docs/agent/testing.md
- Deliver: crates/sr-engine/src/step/grav.rs: k ∈ {1, 4} — k = 4 only when no non-vacuum cell's Σ changed
  by more than 0.1 % over the last 16 steps and the latch saw no neutronization or sink formation in the
  last 1.0 t.u., decided at step ≡ 0 (mod 16) from the state → Verify 1 · tests/gpu/cadence.rs (new): k = 4
  against k = 1 on the settled Sun-like star — the energy ledger's drift ≤ 10⁻³ |W| per τ_dyn, k = 1
  whenever a condition fails → scope `cadence` (G-CAD) → Verify 1 · plant tests/plants/cadence-16.patch
  (new, §5.3) → Verify 2 · not woken by M0-V17 → N/A ("built only if M0-TG's budget needs it", §1.4.4) →
  none
- Verify: 1. `python3 tools/pb/verify.py cadence --task M0-T114` · Pass: GO, ≥ 3 cases · Fail: NO-GO —
  UNVERIFIED (§5.5) · 2. `python3 tools/pb/verify.py --redarm cadence --task M0-T114` · Pass: GO — the plant
  NO-GO · Fail: the plant stays GO
- Adversarial: a quiet test that reads a summary two frames late lets k = 4 run through a collapse's first
  steps — the decision reads the state at the step boundary (§1.4.4).
- Handoff: <placeholder>

## M0-T115 · Docs — time_control.md (the bench and the frame budget), running.md (long runs) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T114)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.8, §3.3 (bench), §6.4, §7 + the
  handoffs of M0-T109–M0-T113
- Deliver: docs/agent/time_control.md § The bench and the frame budget (the cap, the 30-frames switch, the
  latch's scenario), docs/agent/running.md § Long runs (who runs what, on which adapter) → Verify 1 ·
  docs/agent/testing.md rows for bench, warp, fps, top-speed and latch's contract case → Verify 2
- Verify: 1. `grep -cE "^## (The bench and the frame budget|Long runs)" docs/agent/time_control.md
  docs/agent/running.md` · Pass: one per file · Fail: a file without its section · 2. `grep -cE
  "^[|] .(bench|warp|fps|top-speed). [|]" docs/agent/testing.md` · Pass: 4 · Fail: fewer
- Adversarial: frame figures restated in the page go stale at the next bench — it points to bench.json and
  the V's report.
- Handoff: <placeholder>

## M0-V16 · Validation — lot 16: speed and frame rate · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T115)
- Status: TODO
- Read: milestones/m0/m0_contrat.md §1.4.4, §1.8, §3.3 (bench), §5.3 (G-WARP, G-LATCH, G-FPS, G-TOP, G-CAD),
  §6.1, §6.4 + the handoffs of M0-T109–M0-T115 + docs/agent/time_control.md + the delivered files, each by
  the section it covers
- Deliver: the lead's timing runs owed here on the Quadro RTX 4000, one visible terminal each [NOT RUN —
  for you]: `python3 tools/pb/verify.py fps top-speed --task M0-V16` and their red-arms → none — the lead's
  runs, read here · contract-vs-code on lot 16 — the bench against §3.3, G-WARP's bit-identity, G-LATCH's
  scenario, the frame budget at every rung, G-TOP red at 30 frames → M0-TJ2 woken (the build stops for the
  lead, §1.8.6), M0-T114 woken or left DEFERRED with its reason, each defect its own D (E4), every scope
  the lot added red-armed → reports/v16.md → Verify 1 · Phase 18's close (stubs, the header budget, the
  complete loop and its seconds, `rung_record.py report`'s table, a moved class → a TM<n>) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V16` · Pass: GO, every scope present (fps and
  top-speed from the lead's runs) · Fail: NO-GO — each red a D (E4) or M0-TJ2
- Adversarial: frame figures from a machine under other load — the V records the GPU's other processes
  during the lead's runs (nvidia-smi), and a noisy run is repeated, not averaged away.
- Handoff: <placeholder>

> **Commit gate (lead):** Phase 18 closes after M0-V16 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 18: speed and frame rate"`
> `git push`

# Phase 19 — the final validation and the demo

## M0-V17 · Final validation — the whole milestone, contract against code · **CHECK** · Opus 5.5, max · switch · (AFTER M0-V16)
- Status: TODO
- Read: milestones/m0/m0_contrat.md, section by section against the lots' reports + reports/v1.md–v16.md +
  the Pipeline state's register + docs/agent/*.md
- Deliver: the whole contract against the code (PLAYBOOK §2.4): every §5 guarantee's scope GO on linux-pc
  or its D named, guarantee by guarantee — this V runs on linux-pc and runs there every check still owed
  on linux-pc (R15, the lead at M0-TJ3: "the final one (M0-V17) runs them all"), and names every check
  owed on win-laptop with its last GO and the block that ran it (PLAYBOOK §8) — every FROZEN clause (Q1–Q5) honoured, the string table, the
  licences, the web build kept alive, every NOT PROVEN carried with its reason, the DEFERRED blocks
  (M0-TJ1, M0-TJ2, M0-T114) closed N/A with their pointers or woken, each defect its own D (E4) →
  reports/v17.md → Verify 1 · the complete loop once — its ⏱ part the lead's when over 10 minutes — and the
  phase's close (stubs, the header budget, `rung_record.py report`'s table) → Verify 1
- Verify: 1. `python3 tools/pb/verify.py --all --task M0-V17` · Pass: GO, every scope present · Fail:
  NO-GO — each red a D (E4); the push waits
- Adversarial: a final V that re-reads the lot V's verdicts instead of the code — it checks one guarantee
  per lot against the code directly and says which.
- Handoff: <placeholder>

## M0-TD · Show-off demo — a star's life staged and polished · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V17)
- Status: TODO
- Read: this file (rules + this task) + milestones/m0/reports/lives.md + docs/agent/running.md +
  PLAYBOOK.md §13 (OPT-B) and §8 (TV)
- Deliver: milestones/m0/reports/demo.md (new, ≤ 1 page): how to see each ending from the launcher — the
  preset, the rungs, what to watch at each stage, the slow-down's moments — the walk's script for M0-TW →
  Pass · images/td/: the demo's key moments captured with the capture harness (each ending, the views, the
  readout at each stage), iterated until nothing in them reads janky — a misfit in the engine or the UI
  becomes a D (E1), never a fix inside the TD → Pass · what a capture cannot show (motion, timing, frame
  rate, feel) listed for the walk → none — M0-TW
- Pass: every ending captured at its key stages; every capture looked at — passed, filed or flagged; the
  note ≤ 1 page. Fail: an ending missing, or a janky capture neither filed nor flagged.
- Adversarial: a demo polished on still captures can still stutter live — the note lists what the walk
  must watch (motion, the slow-down's timing), and a stutter seen in a capture run's frame times is a D.
- Handoff: <placeholder>

> **Commit gate (lead):** the push before the user gate (PLAYBOOK §10) — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 19: final validation and the demo"`
> `git push`

# Close-out

## M0-TW · Live walk — the user gate: a star's life · **LEAD walk + CHECK scribe** · Opus 5.5, max · switch · (AFTER the push that follows M0-V17 and M0-TD)
- Status: TODO
- Read: this file (rules + this task) + the launchers named in Repo facts
- Deliver: the lead drives the increment from the double-click launcher, one step at a time — a
  star from gas cloud to each of its three ends (a white dwarf, a supernova leaving a neutron star,
  a black hole), the presets making each one click away, at the time speeds the lead picks (the
  lead at M0-TP, 2026-10-08: "All three endings (Recommended)") — while the scribe journals
  every defect and friction as a numbered line (TW-W1…) in reports/walk1.md, no fixes mid-walk,
  its own wrong assertions journalled too → none — the journal is the record · one triage pass
  afterwards (E10): collected first, measured once against the code, forks to the lead in blocks of
  ≤4, D blocks filed batched and placed where they run → none — the filed blocks are the record ·
  the scribe may MOVE blocks to keep the plan runnable top to bottom (E8, move only) → none — each
  move named in the handoff
- Pass: walk completed, journal triaged, register updated. Fail: the increment won't boot — a
  NO-GO defect to fix and re-walk. Approval withheld is a NO-GO, never an approval with a punch
  list.
- Handoff: <placeholder>

## M0-T116 · Documentation — the tracker and the docs aligned with what shipped · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TW)
- Status: TODO
- Read: this file (rules + this task) + milestones.md + docs/agent/*.md + milestones/m0/reports/walk1.md +
  milestones/m0/reports/v17.md
- Deliver: milestones.md — M0's row and worklist aligned with what shipped and what the walk found (the
  tracker is the agent's to keep, PLAYBOOK §4 Rule 6) → Verify 1 · each docs/agent page re-checked against
  the code it describes, each stale line fixed — itemized and confirmed by the lead first (§4 Rule 6:
  build docs) → Verify 1 · the items the walk routed past M0 left for M0-TZ → none — M0-TZ places them
- Verify: 1. `grep -n "^| M0" milestones.md` · Pass: the M0 row reads the walk's verdict and its date ·
  Fail: the row unchanged
- Adversarial: docs "aligned" by reading the plan instead of the code — each page's check names the files
  it read.
- Handoff: <placeholder>

## M0-TZ · Next-plan authoring — M1 · **PLAN** · Opus 5.5, max · switch · (LAST)
- Status: TODO
- Carried flags: · [M0-R1, 2026-10-08] the lead, after the engine ruling: "nice can you quickly check if I could serve it on something like a small amazon lightsail? and move to a larger box when traffic justifies it? If so were gonna have to add some google adsense ad serving to the website, to generate passive revenue" — and, after the check, "great record all that, the CDN and the adsense stuff, its perfect!" A web-release behaviour, routed past M0 (reports/bootstrap.md §3). The sourced record is reports/web_hosting.md: static hosting on a small Lightsail box; a CDN as the first scaling step; AdSense H5 Games Ads at game breaks; no COEP `require-corp` on the game page. Place the web release — hosting, CDN, ads, consent — in a milestone with the lead. Consent rules and revenue are the lead's to rule. · [M0-R2b, 2026-10-08] the lead, mid-session: "one thing id like to add in this project is for the website running the game id like for the user to be able to choose which game they want to play. I might add my space_tykun game from ../ on the website also. It might require something so it can run on the players own hardware, but this is something to settle with the agents running space_tykun." — place the site's game picker with the web release; space_tykun's side is its own `M1-R2` (filed at the lead's request, pushed as 727dca8 in that repo; its report will be `../space_tykun/milestones/m1/reports/web_build.md`) · [M0-TB, 2026-10-08] carry to M1's TG as a flag — M0-TB split 4 blocks (M0-T33, M0-T44, M0-T98, M0-T99), the TG defect: test (vi) stopped at the ceilings — two near-ceiling blocks kept a separable deliverable with its own check (M0-T33's Riemann oracle, M0-T44's rate law), and two tuning blocks kept checks that need the recalibration their own tuning forces (G-CAL); reports/size_pass.md §4
- Read: this file (whole) + milestones.md + reports/bootstrap.md §3 + reports/sandbox_interview.md
  (its later, dropped and open lists) + reports/walk1.md
- Deliver: PLAYBOOK §2.4's TZ → milestones/m1/m1_implementation_plan.md, m1_rules.md and
  m1/{logs,reports,tasks,images}/ — Repo facts re-measured on disk and never copied, every rule,
  grant and hazard of this plan carried by citation, retired with a reason or replaced by "now
  PLAYBOOK §x", the header under budget from line one, blocks sized to §2.1 with `E` marked and
  rated (§0), the ladder re-read on the day → Verify 1 · the items routed past M0 (story mode,
  chemistry beyond M0, the rest of the physics list, Steam, win-laptop) placed in M1 or a named
  later milestone with the lead → none — the lead's ruling is the record · this plan closed clean:
  every block DONE or N/A with a pointer, any walk never played named (§2.4) → none — M1's TP
  checks the carry
- Verify: 1. `python3 tools/pb/plan.py lint --plan milestones/m1/m1_implementation_plan.md` ·
  Pass: GO · Fail: NO-GO
- Adversarial: a fact, hazard or grant of M0 dropped in silence — the inheritance ledger lists each
  one carried, retired or replaced, with its count.
- Handoff: <placeholder>
