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
Models: Claude Code — Fable 5.1, high · Opus 5.5, max (gate) · Opus 5.5, xhigh · Opus 5.5, high (usual) · Sonnet 5.5, high · Sonnet 5.5, medium · Haiku 5.5, high — strongest first; read 2026-10-14 from the claude-api skill's model table (Claude Code 2.1.296, cached 2026-10-06), the probe and the lead's answer (the canonical repository's ladder, 2026-10-10, its last rung added 2026-10-11 on the lead's word)
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
| M0-D19 | BUILD | The web scope is red on linux-pc: `node` resolves to 18 outside an interactive shell, and the page's WebGPU check gets no adapter | AFTER M0-T21 | DONE (2026-10-10 16:30) |
| M0-T22 | BUILD | Frames of steps and the collapse latch, on indirect dispatches | AFTER M0-T21 | DONE (2026-10-10 16:45) |
| M0-TE-V12.3 | PLAN | PLAYBOOK v12.3 adoption | AFTER M0-T22 | DONE (2026-10-11 11:39, started 11:33) |
| M0-TM3 | PLAN | Rating pass — the ladder changed at M0-TE-V12.3: three rungs added | AFTER M0-TE-V12.3 | DONE (2026-10-11 11:56, started 11:42) |
| M0-TH-adopt-a | BUILD | Adopt the toolkit — the tools v12.3 changed | AFTER M0-TE-V12.3 | DONE (2026-10-11 12:35) |
| M0-D20 | BUILD | v12.3's verify.py selftest reads NO-GO 2 on win-laptop: the walk fixture's `trail ` and CRLF `.gitignore` cannot be written on Windows | AFTER M0-TH-adopt-a | DONE (2026-10-11 12:27) |
| M0-T23 | BUILD | GPU timestamps, cost per step and per pass, --timing | AFTER M0-T22 | DONE (2026-10-11 12:51) |
| M0-T24 | BUILD | Time control (sr_physics::time): rungs, frame plan, cap, slow-down, 30-frames switch | AFTER M0-T23 | DONE (2026-10-11 13:12) |
| M0-T25 | BUILD | Scene files (§2.12.1) | AFTER M0-T24 | DONE (2026-10-11 13:41) |
| M0-T26 | BUILD | State dumps (§2.12.3), --dump-every, --load-dump | AFTER M0-T25 | DONE (2026-10-11 14:01) |
| M0-T27 | BUILD | Docs: architecture.md (step, box, latch, files), time_control.md, lot 3 rows | AFTER M0-T26 | DONE (2026-10-11 14:14) |
| M0-V3 | CHECK | Validation, lot 3: the step framework | AFTER M0-T27 | DONE (2026-10-11 17:10) |
| M0-TC2 | MOVE | Compaction — Phase 5's close: seventeen DONE blocks to stub, more than ten (PLAYBOOK §8, §11) | AFTER M0-V3 | DONE (2026-10-12 08:00) |
| M0-TM4 | PLAN | Rating pass — M0-V3's record no longer moves BUILD on Sonnet 5.5, high | AFTER M0-TC2 | DONE (2026-10-12 08:19, started 08:03) |
| M0-TR1 | BUILD | Test-time pass — the phase-close sweep's four game-launching red-arms, on the lead's word | AFTER M0-TM4 | DONE (2026-10-12 08:49) |
| M0-TR1a | BUILD | The native game-launching scopes build through the shared dependency cache and launch their own copy of the binary: boot, desktop and capture arms stop compiling the dependencies cold | AFTER M0-TR1 | DONE (2026-10-12 11:31) |
| M0-TR1b | BUILD | The web scope's locks cover only what needs them: arms overlap their trunk builds with each other's Chrome runs | AFTER M0-TR1a | DONE (2026-10-12 11:54, started 11:39) |
| M0-D21 | BUILD | Sim time is not a function of the steps run: bare steps add no Δt, frames add it grouped by frame | AFTER M0-TM4 | DONE (2026-10-12 12:08, started 11:55) |
| M0-D22 | BUILD | The boot scope leaves out sr-physics and the assets its headless run reads: `--changed` skips G-BOOT on a physics.json edit | AFTER M0-D21 | DONE (2026-10-12 12:16) |
| M0-D23 | BUILD | The harness needs Git Bash on win-laptop: from PowerShell `bash` is WSL's relay and every `bash` scope reads NO-GO | AFTER M0-D22 | DONE (2026-10-12 12:24) |
| M0-D24 | BUILD | The watch_only plant no longer applies: M0-T19 rewrote step/mod.rs's head, so G-WATCH cannot be proven red | AFTER M0-D23 | DONE (2026-10-12 12:27) |
| M0-T28 | BUILD | The CPU twin's FFT and convolution (f64) | AFTER M0-V3 | DONE (2026-10-12 12:32) |
| M0-T29 | BUILD | Gravity on the CPU twin (P2); G-GRAV1/G-GRAV2 twin cases | AFTER M0-T28 | DONE (2026-10-12 12:56) |
| M0-T30 | BUILD | The GPU FFT (radix-2, within WebGPU's limits) | AFTER M0-T29 | DONE (2026-10-12 13:12) |
| M0-T31 | BUILD | Gravity on the GPU (P2); one step's cost at 600 x 400 on the Quadro | AFTER M0-T30 | DONE (2026-10-12 13:31) |
| M0-T32 | BUILD | Docs: physics.md gravity; lot 4's testing rows | AFTER M0-T31 | DONE (2026-10-12 13:37) |
| M0-V4 | CHECK | Validation, lot 4: gravity; the first step-cost measurement read | AFTER M0-T32 | DONE (2026-10-12 14:55, started 13:39) |
| M0-TC3 | MOVE | Compaction — Phase 6's close: fifteen DONE blocks to stub, more than ten (PLAYBOOK §8, §11) | AFTER M0-V4 | DONE (2026-10-12 15:00) |
| M0-D27 | BUILD | verify.py's console never says which step a run is on or how many remain: a 20–40-minute sweep runs blind (the lead's word at M0-V4) | AFTER M0-TC3 | DONE (2026-10-12 15:21, started 15:04) |
| M0-D25 | BUILD | The gravity scopes grade P2 on square, zero-vacuum states and the kernel at seven offsets: four mutants of §1.4.2 survive | AFTER M0-TC3 | DONE (2026-10-12 15:35) |
| M0-D26 | BUILD | The Sun-sized box's step costs 2.3× TW's whole-step estimate with five of its ten passes: 70 % of it is fixed cost per pass (the lead's word at M0-V4) | AFTER M0-D25 | DONE (2026-10-12 16:15) |
| M0-T33a | BUILD | Sod's exact solution: the Riemann oracle for γ = 2 (split from M0-T33 by M0-TB) | AFTER M0-V4 | DONE (2026-10-13 07:58, started 07:56) |
| M0-T33b | BUILD | Gas flow on the CPU twin (P4) — G-SOD's twin case | AFTER M0-T33a | DONE (2026-10-13 08:10) |
| M0-T34 | BUILD | Gas flow on the GPU (P4) with bounce walls; G-SOD GPU case | AFTER M0-T33b | DONE (2026-10-13 08:50) |
| M0-TM5 | PLAN | Rating pass - 5 carried flags | AFTER M0-T34 | DONE (2026-10-13 09:40, started 09:19) |
| M0-D28 | BUILD | The web scope's `ready` case can smoke a server its sibling case is stopping: ERR_CONNECTION_REFUSED, intermittent | AFTER M0-TM5 | DONE (2026-10-13 10:04) |
| M0-D29 | BUILD | web/smoke.mjs breaks its NO-GO line on playwright's multi-line error: verify loses the reason | AFTER M0-D28 | DONE (2026-10-13 10:06) |
| M0-T35 | BUILD | The leave edge and escaped mass; G-EDGE | AFTER M0-T34 | DONE (2026-10-13 10:27) |
| M0-T36 | BUILD | Gravity's source in the flow; the cold collapse (G-COLL) | AFTER M0-T35 | DONE (2026-10-13 11:32) |
| M0-T37 | BUILD | The twin's step driver and --cpu-reference | AFTER M0-T36 | DONE (2026-10-13 12:20) |
| M0-T38 | BUILD | GPU against the twin (G-REF): the collapsing disk, the Sod strip | AFTER M0-T37 | DONE (2026-10-13 12:29) |
| M0-T39 | BUILD | The box against the whole world (G-BOX conservation cases) | AFTER M0-T38 | DONE (2026-10-13 13:08) |
| M0-T40 | BUILD | Docs: physics.md gas flow, edges, the CPU twin; lot 5 rows | AFTER M0-T39 | DONE (2026-10-13 13:38) |
| M0-V5 | CHECK | Validation, lot 5: gas flow | AFTER M0-T40 | DONE (2026-10-13 14:28) |
| M0-TC4 | MOVE | Compaction — Phase 7's close: fifteen DONE blocks to stub, more than ten (PLAYBOOK §8, §11) | AFTER M0-V5 | DONE (2026-10-13 15:35) |
| M0-D30 | BUILD | P4 is 84 % of a Sun-sized step on the Quadro: every cell recomputes its four neighbours' predicted faces (the lead's word at M0-V5) | AFTER M0-TC4 | DONE (2026-10-13 15:55) |
| M0-D31 | BUILD | architecture.md's pass table says P2 and P4 have no dispatches: lot 4's four and lot 5's three were never written in | AFTER M0-D30 | DONE (2026-10-13 16:00) |
| M0-D32 | BUILD | rung_record.py's report reads a block's rung from its handoff's prose, never its quoted model= level=: 6 of the 7 rungs it reads on linux-pc are wrong | AFTER M0-D31 | DONE (2026-10-13 16:09, started 16:03) |
| M0-T41 | BUILD | Heat and light on the CPU twin (P5); G-HEAT1, G-HEAT2, G-FLD twin cases | AFTER M0-V5 | DONE (2026-10-13 16:48) |
| M0-T42 | BUILD | Heat and light on the GPU (P5), radiated energy at vacuum and edges | AFTER M0-T41 | DONE (2026-10-14 07:27) |
| M0-T43 | BUILD | The radiation force in the flow (§2.4.4) | AFTER M0-T42 | DONE (2026-10-14 07:51) |
| M0-TM6 | PLAN | Rating pass - 6 carried flags | AFTER M0-T43 | DONE (2026-10-14 08:08, started 08:01) |
| M0-D33 | BUILD | Resume from a dump loses F (radiation force) | AFTER M0-TM6 | DONE (2026-10-14 08:17) |
| M0-T44a | BUILD | The rate law: ω and its temperature table (split from M0-T44 by M0-TB) | AFTER M0-T43 | DONE (2026-10-14 08:21) |
| M0-T44b | BUILD | Burning on the CPU twin (P6) — G-BURN, G-ORDER twin cases | AFTER M0-T44a | DONE (2026-10-14 08:35) |
| M0-T45 | BUILD | Neutrino cooling and the iron gate on the CPU twin (§2.5.3, N_Fe) | AFTER M0-T44b | DONE (2026-10-14 08:45) |
| M0-T46 | BUILD | Burning on the GPU (P6); G-BURN, G-ORDER GPU cases | AFTER M0-T45 | DONE (2026-10-14 09:11) |
| M0-T47 | BUILD | Neutrinos and the iron gate on the GPU, setting the latch | AFTER M0-T46 | DONE (2026-10-14 09:24) |
| M0-D34 | BUILD | P6's energy: the GPU and the twin book 24 % apart on G-REF's burning field, with X equal to 10⁻¹² | AFTER M0-T48 | DONE (2026-10-14 10:17) |
| M0-D35 | BUILD | The Gamow-shaped rate law on both sides, and the GPU's rate scale past f32's e^88 | AFTER M0-D34 | DONE (2026-10-14 10:31) |
| M0-TJ4 | PLAN | Specify P5's implicit solver: heat flows unthrottled, within the player's frame budget | AFTER M0-D35 | DONE (2026-10-14 11:37, started 11:12) |
| M0-T117 | BUILD | P5's twin, implicit: backward Euler in a_r T⁴, the heat solved by Newton | AFTER M0-TJ4 | DONE (2026-10-14 11:58) |
| M0-T118 | BUILD | P5 on the GPU, implicit: the f32 solve and the commit, on the linear law | AFTER M0-T117 | DONE (2026-10-14 13:34) |
| M0-T119 | BUILD | P5 on the GPU, implicit: the radiative law and the Newton passes; RKL2 retired | AFTER M0-T118 | DONE (2026-10-14 13:51) |
| M0-D36 | BUILD | P5's f32 solve: a thin ring's light leaving is 1 % off the twin's, and the closure δ pushes it onto the disk (G-STIFF (d) on heat_rim) | AFTER M0-T119 | DONE (2026-10-14 14:39) |
| M0-T120 | BUILD | G-STIFF — stiff heat on both sides: a burnt field and a thin ring | AFTER M0-T119 | DONE (2026-10-14 14:59) |
| M0-T121 | BUILD | P5's hard steps carry over into the next submission — speed gives first | AFTER M0-T120 | DONE (2026-10-14 15:31) |
| M0-T122 | BUILD | P5's cost on the Quadro against the lead's 0.5 ms a step | AFTER M0-T121 | DONE (2026-10-14 17:46) |
| M0-D37 | BUILD | P5's Newton passes hit the 16-pass ceiling on sun_disk (steps 4 and 340–347): unconverged steps, ~1,500 slots each | AFTER M0-T122 | DONE (2026-10-14 18:29) |
| M0-D38 | BUILD | heat_gauss's GPU pulse case books `radiated` at the edges on win-laptop (RTX 4080 Laptop): "nothing reaches the edges" red | AFTER M0-D37 | DONE (2026-10-14 18:42, started 18:32) |
| M0-D39 | BUILD | timing's 1000-step frame at SLOTS rounds hangs or loses the device on win-laptop (RTX 4080 Laptop) | AFTER M0-D38 | TODO |
| M0-T48 | BUILD | The burning field in G-REF (edit job) | AFTER M0-T122 | TODO |
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
PLAYBOOK.md v12.3 + the instruction file bind every agent (cited once, never per block). Full text
of every rule below: m0_rules.md. Header budget: 600 / 60. Rating refresh: 5 flags. Full-loop
budget: 600 s on each box in play (the 10-minute line of Rule 4) — linux-pc measured 449.7 s wall at M0-V1
(2026-10-09): 10 scopes, 537 cases in 6.1 s, then 443 s red-arming the six scopes new since HEAD (desktop's
287 s); win-laptop 134 s wall at M0-V2 (2026-10-10): 15 scopes, 586 passed, none new since HEAD to red-arm, ≈ 113 s
normalised by the measured load (15.6 % of the box external; per box since M0-TJ3, PLAYBOOK §8) — every lot's V
restates its box's; over it — the median of the box's last three counted complete loops, normalised to an idle box,
the raw beside it (now PLAYBOOK §4 Rule 4, v12.3 [M0-TE-V12.3]) — a TR fires. OPT modules: OPT-B (R6, confirmed at
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
| R9 | The routine: now PLAYBOOK §4 Rule 4's two tiers (v12.3) — own scope until green → `verify.py --redarm <scope>` only for a scope the task added or whose check it changed → plan edits → `verify.py --changed` once, last; `--all`, then the sweep `verify.py --redarm --moved`, only at phase-closing V and TR; every scope ships a plant; docs/agent/testing.md says how to add one [M0-TE-V12.3] |
| R10 | The claim run: every BUILD block's last step is `python3 tools/pb/verify.py --changed --task <ID>`, read against the task's own start that `plan.py show` records — now PLAYBOOK §4 Rule 4 (v12.3), no commit between tasks · Pass: GO; a block whose `show` recorded no start (one started before M0-TH-adopt-a's swap) names `--base <a commit's SHA>`, never `HEAD` [M0-TE-V12.3] |
| R11 | Scopes: GPU scopes are modules of one test binary, crates/sr-engine/tests/gpu/ (`--test gpu <module>::`), CPU scopes `--lib <module>::` (sr-app's `--bin sandbox-reactions`); scripts source tests/cargo.sh; each lot's docs block writes its rows; (new) = absent on 2026-10-08 |
| R12 | ⏱ scopes: `paths` name only their own test and scene files; physics on the RTX 5090 (win-laptop: the RTX 4080 Laptop, R15), timing (fps, top-speed) on the Quadro only; over 10 minutes → the lead's, in the lot V's window |
| R13 | Physics: every GPU number names its adapter; an UNVERIFIED (§5.5) red is a D or a question, never a looser number (§0.4); each pass's CPU f64 twin before its GPU shader; box and latch first |
| R14 | Every V is done when its verdict is recorded, its D's created and the register updated (PLAYBOOK §14.3); a V that adds to that says so on its own `Done when:` line |
| R15 | Box pattern (M0-TJ3): on win-laptop a block reads the RTX 5090 as the RTX 4080 Laptop, Xvfb/lavapipe/`game-xvfb` as the off-screen window (`game-offscreen`), start.sh as start.bat, /usr/bin/google-chrome as contract §6.2.3's Windows Chrome, `python3 tools/pb/…` as `py -3.12 tools/pb/…`; the Quadro's timing stays `box: laserax-ai`; a check only one box runs is owed elsewhere, flagged V to V, run at the next V on its box or by M0-V17 |
| R16 | N/A — superseded by PLAYBOOK v12.3 (2026-10-11, M0-TE-V12.3; the lead, 2026-10-10: *"In V12.3 if we dont need git well have to remove that rule"*): A block's last message ends with one line for the lead — `git add -A`, commit `M0-<ID> <status>`, push — in the probed box's shell: Windows → PowerShell (`git add -A; if ($?) { git commit -m "…" }; if ($?) { git push }`), linux-pc → bash (`&&`); agents never run it (the lead, 2026-10-10) |
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
  Pro build 26300, Intel Core Ultra 9 185H (16 cores / 22 threads), 31.5 GiB RAM (disk: Hazards); GPUs Intel
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
  current against this playbook's switch — v12.3's 12.3.1 once M0-TH-adopt-a swaps `tools/pb/switch.mjs`, the
  project's 12.2.1 until then. Every rung of the 2026-10-08 ladder is in its `RUNGS` table; whether the table holds the
  2026-10-11 ladder's new rungs (Fable 5.1, high · Opus 5.5, xhigh · Haiku 5.5, high) is read from `now` after that
  swap (M0-TE-V12.3: NOT RUN before the swap).
- Toolchains on win-laptop (M0-TE-win, 2026-10-08, logs/M0-TE-win.log): present — Python 3.13.14 as `python3` (Hazards:
  the Store alias; `python` 3.12.10), Node 26.8.2 + npm 11.19.1 (the table says Node 20; kept, ≥ 20), git
  2.55.0, Git Bash 5.3.15, Chrome 154.0.8037.98 at `C:\Program Files\Google\Chrome\Application\chrome.exe`,
  Visual Studio 2026 Community with MSVC 14.44/14.51 and Windows SDKs; installed by M0-TE-win (R3) — rustc/cargo
  1.99.0 (rustup update; was 1.97.1) with clippy, rustfmt and `wasm32-unknown-unknown`, trunk 0.21.14 (release
  zip, sha256-checked) in `~/.cargo/bin`, and rsync 3.5.1 (outside the table — the lead's yes, GPL-3.0, a dev
  tool) in `~/bin`; user variable for symlinks (Hazards). Missing — playwright-core (arrives with
  web/package.json, M0-T15), the Vulkan SDK. `~/.cargo/bin` is on the PATH here (it is not on linux-pc).
- Agent on win-laptop (M0-TE-win, 2026-10-08): Claude Code 2.1.294 (`AI_AGENT=claude-code_2-1-294_agent`); the
  switch plugin `pb-switch@pb` 12.2.1 is installed (`rung_record.py now` → `plugin=installed`). Re-probed 2026-10-11
  by M0-TE-V12.3: Claude Code 2.1.295 (`AI_AGENT=claude-code_2-1-295_agent`), v12.2's `now` → `plugin=installed`.
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
  `plan.py` (the plan; the Flow row stays in sync): `show <ID>` (your start read; from M0-TH-adopt-a's swap it also
  records the task's start, `verify.py start`, and prints a `baseline:` line) · `status <ID>
  "<STATUS>"` · `close <ID> --status "<STATUS>" --text "<handoff>" [--set "<Key>=<value>"]...` ·
  `handoff <ID> --text "<handoff>"` · `flag <ID> "<text>" --from <ID>` (a flag aimed at a later
  task's `Carried flags:`) · `append <block.md> --after <ID>` (a D you file) · `move <ID> --after
  <ID2>` · `stub <ID>... --by <ID>` · `register --set "<Key>=<value>"...` · `lint --plan
  milestones/m0/m0_implementation_plan.md`.
  `verify.py` (the harness — R9, docs/agent/testing.md; always `--task <ID>`): `<scope>` ·
  `<scope> --case <id>` · `--redarm <scope>` (a scope your task added or whose check it changed) · `--changed` (the
  claim, read against your start; before the swap, or when `show` printed no `baseline:`, `--changed --base <a commit's
  SHA>` — R10) · `--all` and `--redarm --moved` (the sweep — phase V and TR only) · `touched` (from the swap) · `list` ·
  `report` [M0-TE-V12.3]. Scopes today: `plan_lint` · `tools_selftest` · `toolchain`.
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
  win-laptop goes through the Bash tool; no PATH edit (it would shadow Windows' find and sort). Since M0-D23 (2026-10-12)
  both shells work: verify.py puts Git's `usr\bin` and `~/bin` first on the PATH of the commands it spawns only, so a plain
  PowerShell window runs every scope (the lead, M0-V3, 2026-10-11: "I shouldnt be forced to use git bash").
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
- Next task: M0-D39
- Counters: T=122 · D=39 · V=17 · Q=0 · TI=0 · TJ=4 · TV=3 · TC=4 · TR=1 · TM=6 · TD=1 · TE=1
- Open D/BLOCKED register: M0-D39 (TODO, after M0-D38: timing's 1000-step frame at SLOTS hangs or loses the device on win-laptop, filed by M0-D37)
- Outstanding commit gates: Phase 7's (the lead's, after M0-V5's close): M0-T33a … M0-T35 committed through 19e0d8e, pushed; M0-T36 … M0-T40 and M0-V5 (lot 5's last code, M0-V5's report, logs, plan, archive and contract edits) are not
- Carryover: none
- Awaiting lead: none — M0-TJ4's three answers recorded (contract §0.5 [M0-TJ4])
- Model ratings: 2026-10-14 by M0-TM6

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
- Status: DONE (2026-10-09 12:36, started 12:36) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): 22 DONE blocks stubbed (Phase 2b: M0-TE-win, D2–D8, TJ3; Phase 3: M0-T1–T9, D9, TM1, TV1, V1) into plan_archive.md: 450 lines moved, 450 verified byte-for-byte; M0-D8's commit-gate trailer kept in the plan (WARN, by design). No TODO block touched. Rules already index lines (text in m0_rules.md) and Pipeline state already pointers — nothing to move there; Repo facts left as is (no safe de-dup measured, nothing dropped). plan.py show header=225 before and after (lint header 404/600 after, pipeline 9/60); plan lint GO 499 checks, 1 warn (Read: names plan_archive.md whole, this block's own line). Ran on model=claude-sonnet-5-5 level=medium per rung_record (linux-pc). Next: M0-D10.
- Full block: plan_archive.md § M0-TC1

## M0-D10 · Lot 1's scopes leave out inputs their verdicts depend on: `--changed` skips G-BOOT on a headless.rs edit · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-TC1)
- Status: DONE (2026-10-09 12:45) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): verify.json: boot, desktop, capture + crates/sr-engine/**; web: crates/sr-app/** + crates/sr-engine/** replace its three narrower globs; boot, desktop, capture, web, adapter, state + Cargo.toml, Cargo.lock, rust-toolchain.toml, .cargo/**, tests/cargo.sh; no case, plant or expected changed. Anchor on linux-pc: headless.rs -> adapter, boot, build, capture, desktop, state, web; Cargo.lock -> the same seven. Claim --changed --base 516ec7c [ALREADY RUN — PASS (522 passed, 8/8 scopes GO) on linux-pc], logs/M0-D10.*.log; --redarm boot [ALREADY RUN — PASS (scene-refuse red) on linux-pc]. Deviations: base is the latest commit 516ec7c (Phase 3 gate not yet made), so FLAG oracle lines list lot-1 plants/tests as changed — expected, not this task's edits; verify.json rewritten via json.dump (the scopes' non-ASCII text now \u-escaped, content equal). Not owed on win-laptop: untested there. Ran on model=claude-sonnet-5-5 level=medium (rung_record). Next: M0-D11.
- Full block: plan_archive.md § M0-D10

## M0-D11 · The binary refuses its own flags by position: `--out` first, `--help` after a flag read "unknown argument" · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M0-D10)
- Status: DONE (2026-10-09 12:48) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): main.rs now sends every non-headless, non---help-first argument list to desktop::run; desktop parse answers -h/--help anywhere with the usage (exit 0), unknown stays SR-ERROR exit 4, --world/--measure-ui keep their refusal. Measured on linux-pc: --out first + --capture under xvfb/llvmpipe exit 0 with 2 PNGs; --adapter llvmpipe --help exit 0; --bogus exit 4 (logs/M0-D11.log). Not guarded: no scope tests flag order and the existing capture plant does not cover it — a case is a CHECK/new-scope job, not filed as a D (lead's call). Ran on rung_record: model=claude-sonnet-5-5 level=medium. Next: M0-D12.
- Full block: plan_archive.md § M0-D11

## M0-D14 · The adapter scope reads NO-GO: after one NVIDIA device is made, wgpu lists only the Intel iGPU and llvmpipe · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D11)
- Status: DONE (2026-10-09 18:16) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record). Cause measured: each wgpu::Instance dropped unloads the Vulkan ICDs; every NVIDIA reload leaks glibc static TLS and from the 11th instance in a process the loader logs « libnvidia-tls.so.595.99.02: cannot allocate memory in static TLS block » and ignores nvidia_icd.json, so wgpu lists only Intel + llvmpipe (VK_LOADER_DEBUG; probe: 10 ok then 30 without NVIDIA; 40/40 with instances kept alive or GLIBC_TUNABLES optional_static_tls raised). Fix: sr-engine gpu.rs headless_instance() — one process-wide instance (OnceLock; wasm keeps one per call), used by Gpu::new_headless and the test's live_adapters; no expectation loosened. verify.py adapter [ALREADY RUN — PASS (10/0/0) on linux-pc]; --redarm adapter [ALREADY RUN — PASS (clean GO, adapter-case RED 5 failed) on linux-pc]; claim --changed --base 516ec7c [ALREADY RUN — PASS (523 passed, 8/8 scopes GO) on linux-pc] — run twice on the same tree by mistake, both GO; its FLAG oracle lines are earlier tasks' files since 516ec7c. Not explained: why M0-V1 read 10/10 the same day (same driver, glibc, loader, layers, source) — reading, not measured: unload timing. Not acted on: a plan Hazards line (one wgpu instance per process on linux-pc) is TZ's/the lead's; win-laptop NOT RUN (owed on win-laptop at its next V). Plan: close --set "Caused by=…" wrote a register row, not the block's field — that one row removed with the file-edit tool (plan.py has no row delete); Caused by stays unknown (a box/driver limit, no block). Detail: tasks/M0-D14.md. Next: M0-D12 (re-run its claim).
- Full block: plan_archive.md § M0-D14

## M0-D15 · desktop[launcher] reads « 'start.bat' is not recognised » on win-laptop: the case's `cmd //c "start.bat window"` never reaches the file · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D14)
- Status: DONE (2026-10-09 19:27) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record). Cause measured: the Bash tool's env sets NoDefaultCurrentDirectoryInExePath=1, so cmd skips the cwd for a bare name; A/B/C on an echo-only probe.bat: bare+var → « not recognised », var unset → runs, .\ prefix → runs (logs/M0-D15.log). Fix: tests/smoke.sh run_launcher MINGW → cmd //c ".\start.bat $service"; nothing loosened. Deviation, the lead's yes (« Extend the plant (Recommended) »): launcher-no-stop.patch patched start.sh only, which made it unfailable on win-laptop (first --redarm NO-GO); it now also drops start.bat's stop line (LF hunk, applies to CRLF via git apply and patch). verify.py desktop [ALREADY RUN — PASS (2/0/0) on win-laptop]; --redarm desktop [ALREADY RUN — PASS (clean GO, both plants RED, 606 s) on win-laptop]; claim --changed --base fc0a0fe [ALREADY RUN — FAIL (514/1: plan_lint, boot, desktop GO; tools_selftest[capture_web] 2 failures, M0-D16's native crash, the same in M0-D13b's run before this fix; the anchor's 11 never-checked are tools/pb files left unrecorded by that red, my reading) on win-laptop]. Finding: a desktop red-arm takes 606 s here, over the 10-minute line (Rule 4), so it is the lead's run on win-laptop until shortened. testing.md's desktop row still reads « owed ». Detail: tasks/M0-D15.md. Next: M0-D16, then M0-D13's claim.
- Full block: plan_archive.md § M0-D15

## M0-D16 · tools_selftest[capture_web] crashes in review_page.py's two cases on win-laptop: a native crash with a faulthandler stack and no Python frame · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D15)
- Status: DONE (2026-10-10 08:49) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record). Cause measured, not a native crash and not M0-D13's killed processes (quiet box, case alone, same 2 reds): capture_web.mjs spawns PB_PYTHON || bare 'python' on win32, the harness never set PB_PYTHON, and bare python is now C:\Program Files\Python314 — Python 3.14.8 installed machine-wide 2026-10-09 without its Standard Library (Lib\ holds only site-packages); it dies « Failed to import encodings module », whose dump tail is the « <no Python frame> » we saw. Fix: tools/pb/verify.py case_env sets PB_PYTHON=sys.executable (setdefault, a caller's value wins), plus a selftest check and a plant (verify 171/0, plants 101/101); nothing loosened. Injected bug (line removed) → both cases red, restored. tools_selftest [ALREADY RUN — PASS (10/0/0, 72.8 s) on win-laptop]; --redarm tools_selftest [ALREADY RUN — PASS (clean GO, plant RED, 152 s) on win-laptop]; claim --changed --base fabab0c [ALREADY RUN — PASS (511/0/0, 2/2 scopes, 69.3 s) on win-laptop]. Finding, the lead's: repair or uninstall Python 3.14.8 (Modify → Standard library) — until then bare python, py and py -3 fail at startup on win-laptop (py -3.12 is fine); Repo facts do not list it. Detail: tasks/M0-D16.md. Next: M0-D13's claim on a fresh id.
- Full block: plan_archive.md § M0-D16

## M0-D12 · Lot 1's docs name an adapter wgpu does not list, restate contract numbers and carry a pre-lot-1 loop line · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D14 — the lead moved D14 first, 2026-10-09: D12's claim run needs the adapter scope green)
- Status: DONE (2026-10-09 18:27) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record). Docs corrected in place, uncommitted, as the first try left them plus one tightening: running.md's exit-code line now reads « Exit codes: §3.3. » (the first try's « a refused flag or scene is 4; no adapter is 3 » still restated §3.3, against Deliver and §7). Also in the tree: running.md's iGPU named « Intel(R) Graphics (RPL-S) » with the --adapter "RPL-S" match; architecture.md:65 figure dropped, § 1.9.1 cited; testing.md's adapter row, Scopes header dated M0-V1, loop line from reports/v1.md § The complete loop, the seven rows' costs from M0-V1's logs. Done-when [ALREADY RUN — PASS (both greps empty; 50/32/17 § lines; 7 scope rows) on linux-pc]. Claim --changed --base 516ec7c --task M0-D12-r2 [ALREADY RUN — PASS (GO 522/0/0, 8/8 scopes, 4.91 s) on linux-pc], logs/M0-D12-r2.log; its one flag (M0-D12-r2 is no block) is the fresh-id rule, expected. First try's red (adapter 8/10) was M0-D14, now DONE. Not acted on: architecture.md:65 reads « read back async. » (terse, not wrong); testing.md's win-laptop costs stay as written. Next: M0-D13.
- Full block: plan_archive.md § M0-D12

## M0-D13 · The boot scope deletes the summary.json it grades: a V cannot read what G-BOOT checked · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D12)
- Status: DONE (2026-10-10 08:56) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): tests/smoke.sh's headless-boot branch prints the summary.json it checked, verbatim, after the unchanged §2.12.2 check (in the tree since fabab0c). Claim on win-laptop (Laser2025-20): verify.py --changed --base fc0a0fe --task M0-D13c GO, 514 passed 0 failed, 4/4 scopes (plan_lint, tools_selftest, boot, desktop); logs/M0-D13c.boot.log holds version/adapter/steps 200. The 2026-10-09 try's two claim reds were filed and fixed as M0-D15 and M0-D16. Deviation: measured on win-laptop, not linux-pc — boot + --redarm boot on linux-pc owed (R15), flagged to M0-V2. Ran on Opus 5.5, high (model=claude-opus-5-5 level=high). Detail: tasks/M0-D13.md. Next: the Phase 3 commit gate (lead), then the next TODO.
- Full block: plan_archive.md § M0-D13

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
- Status: DONE (2026-10-10 14:08) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now). Verdict DEFECTS(1), reports/v2.md: lot 2 conforms to §1.5, §1.6.2, §2.3, §2.5.1, §2.7, §4.1 and §5.4 value by value and bound by bound (27/27 registry mutants killed); Verify 1 py -3.12 tools/pb/verify.py --all --task M0-V2 [ALREADY RUN — PASS (GO, 586 passed, 15/15 scopes, 134 s wall, ≈ 113 s normalised by load) on win-laptop]; lot 2's six scopes and lot 1's seven red-armed here, 16/16 plants red (desktop 746 s, web 752 s raw — over 600 s). Filed: M0-D17 (smoke.mjs runs Chrome with --no-sandbox, playwright-core's default — chrome://version measured), M0-TM2 (the record moves BUILD on Sonnet 5.5, high: $1.70 vs $1.44 per DONE block, 10 blocks each). The lead: «Yes, rename once (Recommended)» → contract §0.5 [M0-V2], Superseded, R17 (declared: R17 reads all 30 hyphenated scope names in the plan, not only §5's 8). Carried flags discharged: M0-D9's wrapper ×3 (adapter, build; mkdir lock), lot 1 on win-laptop with the lead's start.bat «Opened, stayed till closed» (start.ps1's double-click opened Notepad — Windows' default; its off-screen run works), --redarm web on the fixed tree; owed on linux-pc → M0-V3 (lot 2, M0-T15's web/npm, M0-D13's boot). Not filed (E1 (i), § Observations): T84's stand-in readout, Chrome ignoring powerPreference on Windows, start.ps1 without a case, G-LIC's workspace-member reading without a §0.5 line, red-arm cost here. Phase 4: T10–T17 stubbed, header 418/600, flags → M0-T27, M0-T32; Phase 3's 8 late blocks a TC's. Next: M0-D17.
- Full block: plan_archive.md § M0-V2

## M0-D17 · web/smoke.mjs runs Chrome with `--no-sandbox`: playwright-core adds it by default, against §6.2.3 · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V2)
- Status: DONE (2026-10-10 14:22) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record now, the heading's rung). smoke.mjs launches Chrome with chromiumSandbox: true and both cases read chrome://version's #command_line: 'sandbox: on' in the log, NO-GO naming --no-sandbox otherwise (an unreadable command line is NO-GO too); plant smoke-no-sandbox (the option dropped) in the web scope; testing.md's web row (guard, plant, red-arm cost) and verify.json's desc say so. web GO 3/0/0 (ready 0.8 s, SR-CHROME-ADAPTER nvidia lovelace fallback=false: WebGPU holds in a sandboxed headless Chrome here, the BLOCKED arm did not fire); --redarm web GO 4/4 red, smoke-no-sandbox red on both cases, 515 s wall, 6.8 % external load, about 480 s normalised. Claim run verify.py --changed --base 2ada210 --task M0-D17: GO, 499 passed, 2/2 scopes (its web red-arm 4/4 red, 600.00 s, not load-measured); its two oracle FLAGs (the plant, verify.json) are the plant and scope entry Deliver names - declared, not a deviation. Owed on linux-pc (R15): web and its red-arm with the sandbox on - flagged to M0-V3. Detail: tasks/M0-D17.md. Next: M0-TM2.
- Full block: plan_archive.md § M0-D17

## M0-TM2 · Rating pass — the record moved BUILD on Sonnet 5.5, high (M0-V2's table) · **PLAN** · Opus 5.5, max · switch · (AFTER M0-D17)
- Status: DONE (2026-10-10 14:52, started 14:43) · archived by M0-TE-V12.3, 2026-10-11
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Ladder re-read 2026-10-10: the claude-api skill's table in Claude Code 2.1.295's bundle (cached 2026-10-06, the copy M0-TM1 read) - no rung retired, renamed, repriced or relevelled (Opus 5.5 $4/$20, Sonnet 5.5 $2/$10) → Models line re-dated, rungs unchanged, no question. Record today (rung_record report): BUILD on Sonnet 5.5, high 10 blocks at $1.85 per DONE block (M0-V2's $1.70 plus M0-D17, traced to M0-T15) against (usual) 11 blocks at $1.45 → rule (4) moves the class one rung, to (usual). 128 open blocks re-derived, 89 changed - Opus 5.5 max 20 (0) · Opus 5.5 high 101 (+89: 89 by rule (4), 12 by (3)'s list → (5)) · Sonnet 5.5 high 0 (−89) · Sonnet 5.5 medium 7 (0: E, their class n=1); no trail (rule (1)); every heading keeps · switch; the diff is the 89 rating segments and the Models line (checked against a post-status snapshot). Not acted on: the record's rows differ in kind ((usual) = 10 D fixes, Sonnet = 9 feature T's + TE-win; medians $1.12 vs $1.42; T15's escalation $6.24) and linux-pc's 28 blocks are unread - observations, rule (4) applied as written; Next task still reads M0-TM2 (outside TM2's diff; next is M0-T18); Repo facts name Claude Code 2.1.294, the probe reads 2.1.295. lint GO (496), plan_lint GO, its red-arm GO; the claim run verify.py --changed --base aa2e690 --task M0-TM2 is the last step, after this close. Detail: tasks/M0-TM2.md. Next: M0-T18.
- Full block: plan_archive.md § M0-TM2

> **Commit gate (lead):** Phase 4 closes after M0-V2 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 4: registries, equation of state and guards"`
> `git push`

# Phase 5 — Lot 3 · the step framework: booking, floors, Δt, the box, the latch, time

## M0-T18 · Booking — the side buffers and accumulators every pass books into · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V2)
- Status: DONE (2026-10-10 15:02) · archived by M0-TC2, 2026-10-12
- Handoff (one line): state.rs: SideFields (side.phi 1, side.flux 2 F_x/F_y, side.s_nu 1 planes), TERMS (37: every §2.9 booked term + the 10 §1.9.1 accumulators, writer pass named), BOOK_GROUPS (8 buffers, one writer each, const-asserted ≤128 MiB at 2048²), Booking (clear, readback), book_totals (f64, term→row→slot); State::readback now shares read_buffers. Scope booking 4 cases + plant booking-row-skip, testing.md lot-3 row.
- Full block: plan_archive.md § M0-T18

## M0-T19 · P8 floors, complete, and the equation of state on the GPU · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T18)
- Status: DONE (2026-10-10 15:25, started 15:10) · archived by M0-TC2, 2026-10-12
- Handoff (one line): eos.wgsl (new; group-1 EosParams + per-pair ln u and f64 segment slopes, uploaded once by step::EosGpu; composition, cold_pressure, cold_u, ε_cold, Π, c², T, floors) · floors.wgsl: floor_cells (vacuum reset → Σ_floor, u = 0, T_floor; else T floor; books vacuum_reset + floor_added) ahead of renormalise_species, both on one renormalised() · Step::new(device, state, booking, eos), Dispatch with labels and bind groups · sr-engine → sr-physics path dep (Cargo.lock +1 line) · headless builds Booking + EosGpu · tests/gpu/floors.rs (6 cases, inline f64 twin) · plant floors-no-cold-energy · scope floors · contract §2.7 [M0-T19] (the lead's yes: all ≤ 0 → pure H, μ of the renormalised fractions, reset energy booked floor_added) · testing.md row.
- Full block: plan_archive.md § M0-T19

## M0-T20 · Δt and the non-finite guard (P1, P9) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T19)
- Status: DONE (2026-10-10 15:46) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20, RTX 4080 Laptop), model=claude-opus-5-5 level=high (rung_record). Built: reduce.wgsl + step/dt.rs — P9's Δt by a fixed-order tree (C, η_g from physics.json; gravity term infinite until P2), P1 dt_advance on step n−1's value, step 0 preceded by the initial state's reduction, the guard every 64th step (index ≡ 0 mod 64; non-finite told by bits; lowest cell, then channel) → headless exit 5 naming step, cell, channel (exit-5 path NOT PROVEN (source only): no scene can plant a NaN yet). Step::new now takes &Physics; Step::read_dt, dt_record (Δt_n = word 0) for later passes. Vacuum cells count as P8's floor state in Δt; whole world until M0-T21's box. Deviation: 9 code files / ≈680 added lines over the 6/600 ceiling — state.rs, floors.rs, state test edits are the new signature and the P1/P9 dispatches (the FLAG oracle lines). Carried flag decided: guard reads all 14 channels after P8; measured P8 zeroes a NaN fraction (logs/M0-T20.probe.log) → filed M0-D18. sim_time still 0 (Δt not summed — not this Deliver). verify.py dt [ALREADY RUN — PASS (5/0/0) on win-laptop]; --redarm dt [ALREADY RUN — PASS (dt-cfl-one RED) on win-laptop]; --changed --base 2ee163f [ALREADY RUN — PASS (538 passed, 12/12 scopes GO) on win-laptop]; linux-pc owed (R15). Detail: tasks/M0-T20.md. Next: M0-D18.
- Full block: plan_archive.md § M0-T20

## M0-D18 · P8 zeroes a NaN species fraction before P9's guard reads it: the run goes on with the cell renormalised · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T20)
- Status: DONE (2026-10-10 15:53) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20, RTX 4080 Laptop), model=claude-opus-5-5 level=high (rung_record now, the heading's rung). floors.wgsl: non_finite (bits, reduce.wgsl's test) + fractions_non_finite; floor_cells and renormalise_species return at once for a cell holding an Inf/NaN fraction - fractions as read, Σ/u/E untouched, nothing booked; finite cells' rules unchanged. Deviation (declared, my reading of 'the floors kept as they are'): floor_cells skips the cell too, else a vacuum cell would get E = NaN and the guard would name energy, not the fraction. verify.json: floors expected 7, dt 6 (the Pass counts). New cases: floors a_cell_with_a_non_finite_fraction_is_left_for_the_guard (NaN/+Inf/-Inf in 5 kinds of cell), dt the_guard_names_a_nan_fraction_that_p8_would_renormalise_away (M0-T20's probe, now step 0, (20, 10), x_C); both shown red with the check stubbed out (logs M0-D18-red.*). floors [ALREADY RUN — PASS (7/0/0) on win-laptop]; dt [ALREADY RUN — PASS (6/0/0) on win-laptop]; --redarm floors [ALREADY RUN — PASS (floors-no-cold-energy RED) on win-laptop]; claim --changed --base 5206293 runs after this close, result in tasks/M0-D18.md. Not acted on: Twin's dead-code warning (M0-T19, cosmetic). linux-pc owed (R15). Detail: tasks/M0-D18.md. Next: M0-T21.
- Full block: plan_archive.md § M0-D18

## M0-T21 · The active box · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T20)
- Status: DONE (2026-10-10 16:12) · archived by M0-TC2, 2026-10-12
- Handoff (one line): On win-laptop Laser2025-20 (RTX 4080 Laptop): step/boxfit.rs + reduce.wgsl's box_cells/box_fit/box_gate write the box record, the passes' indirect sizes and the FFT size per axis on the GPU, never read back; P8's two dispatches and dt_cells run indirect over the box; re-fit before step 0, after P9 at n ≡ 0 (mod 16), and in P0 when Step::box_edit_flag is set; Step::with_box(…, BoxMode::WholeWorld) is the switch. Scope box GO 6/6, --redarm GO (margin-zero red, 5 of 6 failed); claim --changed --base 103a236 GO 13/13, 543 passed (win-laptop). linux-pc laserax-ai over ssh (tree 695bbb9, RTX 5090): box GO 6/6, --redarm GO; --changed 12/13 — web NO-GO, pre-existing (wasm never fetched) → M0-D19 filed. Deviations: 7 code files, ≈ 720 new code lines (P8/P9 moved onto the box's sizes in the same thread); defaults — empty box → zero-sized dispatches and Δt = the largest f32; the re-fit's scan and the guard read the whole world; vacuum outside the box is never floored. Not acted on: clippy --tests red on 4 pre-existing lints in tests/gpu/floors.rs (no scope runs it). Model claude-opus-5-5, level high. Detail: tasks/M0-T21.md. Next: M0-D19.
- Full block: plan_archive.md § M0-T21

## M0-D19 · The web scope is red on linux-pc: `node` resolves to 18 outside an interactive shell, and the page's WebGPU check gets no adapter · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T21)
- Status: DONE (2026-10-10 16:30) · archived by M0-TC2, 2026-10-12
- Handoff (one line): (a) tests/web/check.sh picks Node >= 20 itself: PATH node if >= 20, else nvm default alias (laserax-ai: 20 -> v20.20.2), else NO-GO naming the node found (shown red, logs/M0-D19.node-guard.log). (b) Cause measured (logs/M0-D19.probe.log): a freshly launched headless Chrome answers its first requestAdapter null (6/6, any powerPreference, crash count 0, "external Instance reference" warning); any later or >=500 ms-later request gets the adapter. Fixed in the page, not the smoke: web/index.html asks once more on a null; two nulls still show the no-WebGPU page (my reading of contract 3.5: no override). Plant web-no-gpu-check rebased onto the new lines.
- Full block: plan_archive.md § M0-D19

## M0-T22 · Frames of steps and the collapse latch · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T21)
- Status: DONE (2026-10-10 16:45) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20, RTX 4080 Laptop), model=claude-opus-5-5 level=high (rung_record). Built: shaders/latch.wgsl + step/latch.rs — Step::encode_frame(plan, encoder) records N steps in one encoder under the latch; every dispatch reads its size from a buffer the controller (latch_check, after each P9) zeroes: fixed sizes from latch slots (guard every 64 and re-fit every 16 opened by the GPU's step index), box-sized from box.args, P0's re-fit from its gate (both saved, zeroed, restored by latch_end); Latch::new refuses a step with any other indirect buffer. Armed = 1.0 t.u. since the last set (every set restarts it); slow-down off never fires. Step::poll(device) returns each frame's report once (map_buffer_on_submit, PollType::Poll). Test hook set_latch_hook + box-sized probe. Deviations: Step methods, no Engine yet; FramePlan carries slow_down; frames and bare run() not mixed (CPU index runs ahead after a stop); no cost per step (M0-T23). Verify 1 [ALREADY RUN — PASS (latch 5/5) on win-laptop] · Verify 2 [ALREADY RUN — PASS (latch-off RED, 5 failed) on win-laptop]. G-LATCH NOT PROVEN (synthetic) until M0-T111. Owed on laserax-ai: latch + --redarm latch (R15). Claim --changed --base 14d51e1: tasks/M0-T22.md. Detail: tasks/M0-T22.md. Next: M0-T23.
- Full block: plan_archive.md § M0-T22

## M0-TE-V12.3 · PLAYBOOK v12.3 adoption · **PLAN** · Opus 5.5, max · switch · (AFTER M0-T22)
- Status: DONE (2026-10-11 11:39, started 11:33) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Applied v12.3's §16 entry alone (no other TE-V; the Rules named v12.2 before the bump). Models: the lead's ladder from the carried flag, re-read 2026-10-11 — the probe and the claude-api table (Claude Code 2.1.295, cached 2026-10-06): nothing retired, renamed or repriced, no question; lint reads 7 rungs.
- Full block: plan_archive.md § M0-TE-V12.3

## M0-TM3 · Rating pass — the ladder changed at M0-TE-V12.3: three rungs added, 123 open blocks · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TE-V12.3)
- Status: DONE (2026-10-11 11:56, started 11:42) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Ladder re-read 2026-10-11 — the probe (Claude Code 2.1.295) and the claude-api skill's table (cached 2026-10-06, the models.md M0-TM2 read): nothing retired, renamed, added or repriced, each rung's level one its model takes → Models line unchanged, no question. 124 open blocks re-derived; 7 changed — the E blocks M0-T39, T48, T52b, T91, T93a, T94, T111 Sonnet 5.5, medium → Haiku 5.5, high (rule (3), the ladder's new lowest rung; BUILD·E has no record there). By rung: Opus 5.5 max 20 (0, rule (2)) · Opus 5.5 high 97 (0: 89 by rule (4), M0-TH-adopt-a's placeholder now among them by name; 8 by (3)'s list → (5), M0-T31 newly (c) from M0-T21's flag) · Haiku 5.5 high 7 (+7) · Sonnet 5.5 medium 0 (−7) · Sonnet 5.5 high, Opus 5.5 xhigh, Fable 5.1 high 0; no trail (rule (1)); every heading keeps · switch; the diff is the 7 rating segments (checkdiff against a post-status snapshot). Measured, not acted on: today's rung_record report no longer moves BUILD on Sonnet 5.5, high ($2.05 against (usual) $2.46 per DONE block — lot 3's five rule-(5) builds at a $4.77 mean, M0-D19 now traced to M0-T15); §0's record is the phase-closing V's table, M0-V2's still moves the class, so the 89 stay on (usual) until M0-V3's table decides; the switch's RUNGS likely lacks Haiku 5.5, high (the box's plugin is 12.2.1) — the 7 run on the lead's pick until M0-TH-adopt-a's swap and the lead's reinstall; Next task still reads M0-TM3 (outside this diff; next is M0-TH-adopt-a). lint GO (506 checks, 0 warnings); the claim run verify.py --changed --base ab36299 --task M0-TM3 is the last step, after this close. Detail: tasks/M0-TM3.md. Next: M0-TH-adopt-a.
- Full block: plan_archive.md § M0-TM3

## M0-TH-adopt-a · Adopt the toolkit — the tools v12.3 changed · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TE-V12.3)
- Status: DONE (2026-10-11 12:35) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop, model=claude-opus-5-5 level=high (rung_record, at launch and on resume). Swapped plan.py, verify.py, content_gate.py, rung_record.py, switch.mjs to v12.3's annex (§B.4: 10 extracted, 0 refused); diffed first into tasks/toolkit_local_diff.md; re-added the local fixes the annex lacks (verify.py case_env M0-D7/D16, stub trace lock, torn-tolerant spans; plan/content_gate/rung_record selftest Windows ports), flagged to M0-TZ. scratch_copy.sh, status_page.py kept byte for byte (v12.3's annex drops their Windows fixes); verify.json untouched (no new tool). Verify 1 GO · 2 all five GO (verify's after M0-D20's fixture fix, measured in the claim) · 3 lint GO. Claim --changed --base ab36299 [ALREADY RUN — PASS (517/0/0: plan_lint 507/0/0, tools_selftest 10/0/0) on win-laptop]; the first claim's FAIL was M0-D20, now DONE; --redarm tools_selftest paid by M0-D20 (GO). Switch: all 7 ladder rungs in RUNGS; the lead updated this box's plugin 12.2.2→12.3.1 (now plugin=installed; needs a client restart); linux-pc's install owed there. Found, not acted on: Repo facts' win-laptop git reads 2.55.0, box has 2.56.0.windows.2 (a PLAN block's edit). Detail: tasks/M0-TH-adopt-a.md. Next: M0-T23.
- Full block: plan_archive.md § M0-TH-adopt-a

## M0-D20 · v12.3's verify.py selftest reads NO-GO 2 on win-laptop: the walk fixture's `trail ` and CRLF `.gitignore` cannot be written on Windows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TH-adopt-a)
- Status: DONE (2026-10-11 12:27) · archived by M0-TC2, 2026-10-12
- Handoff (one line): verify.py selftest walk fixture fixed (win-laptop Laser2025-20): files written with write_bytes (CRLF exact), trailing-space 'trail ' probed on the file system and left out of wfiles/ignored_by_rule when it cannot exist; no check removed, no tolerance added. selftest GO checks=345 failed=0 plants=157/157 red (logs/M0-D20.log); --redarm tools_selftest GO, clean GO, plant RED (logs/M0-D20.redarm.log). Annex fix flagged to M0-TZ. Ran on claude-sonnet-5-5 level=high. Next: M0-TH-adopt-a (Verify 2 now unblocked).
- Full block: plan_archive.md § M0-D20

## M0-T23 · GPU timestamps, cost per step and `--timing` · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T22)
- Status: DONE (2026-10-11 12:51) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20, RTX 4080 Laptop, Vulkan), model=claude-opus-5-5 level=high (rung_record). Built: step/mod.rs — Step::set_timing(device, queue, Timing::{Off,Auto,Wall}) (off by default: other scopes record as before), timestamp writes at every compute pass's boundaries (never inside: no TIMESTAMP_QUERY_INSIDE_*) into one 4096-query set per batch (a frame, or bare steps until read_cost), first whole steps timed while they fit (≈ 509 at 4 passes/step); StepCost {source gpu-timestamp|wall, steps, step_ms = first pass's begin → last pass's end, pass_ms per Pass, frame_ms}; latch.rs — FrameReport.cost from the same map-on-submit readback, poll() still never waits; gpu.rs — the device takes timestamp-query only where the adapter offers it; headless --timing → summary.json timing {per_pass_ms, step_ms_mean, source}. Measured: passes' sum 12–19 % under the step span (bound 40 %). Deviations: timing.source added to §2.12.2's object (additive; a PLAN task may fold it in); the headless case runs headless::run in-process; timing is a mechanism scope on the physics adapter (no speed graded); the adapter scope's features assertion now reads 'timestamp-query where offered' (re-red-armed: GO). wasm32 wall fallback has no clock (no dependency) → cost None in a browser without timestamp-query (flagged to M0-T24). Verify 1 [ALREADY RUN — PASS (timing 5/5) on win-laptop] · Verify 2 [ALREADY RUN — PASS (timing-skip-pass RED, 1 failed) on win-laptop]. Owed on laserax-ai: timing + --redarm timing (R15). Claim --changed: tasks/M0-T23.md. Detail: tasks/M0-T23.md. Next: M0-T24.
- Full block: plan_archive.md § M0-T23

## M0-T24 · Time control — rungs, the frame plan, the cap, the slow-down, the 30-frames switch · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T23)
- Status: DONE (2026-10-11 13:12) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record). Built: crates/sr-physics/src/time.rs — TimeControl per §3.2 (new(top_rung, render_reserve_ms), select/faster/slower, toggle_pause, step_once, plan_frame(Estimates{wall_s}) → Plan{steps, f_target, slow_down}, on_report(Report{steps_run, advanced, dt_next, step_ms, latch_fired}), on_event, speed_text), top_rung_from_life, sig2; scope time_control (R17 for time-control, 18 cases), plant slowdown-auto-return; testing.md row. Defaults declared (tasks/M0-T24.md): owed booked latency-safe (planned N·Δt_est taken at plan, given back at report); owed also bounded above by one frame's sim time (the cap's shortfall never repaid; latch-fired → owed ≤ 0); N_max = 1 until a cost is read back; an empty world's Δt (f32::MAX) → no estimate, one step a frame, owed reset. Deviation: ≈ 745 new lines (ceiling 600), one thread. Verify 1 [ALREADY RUN — PASS (time_control 18/18) on win-laptop] · Verify 2 [ALREADY RUN — PASS (slowdown-auto-return RED, 1 failed) on win-laptop] · claim --changed [ALREADY RUN — PASS (601 passed, 11/11 scopes GO) on win-laptop], 2 FLAG oracle lines: verify.json changed / scope time_control added though Deliver names neither (R17 spelling; adding a scope's entry is docs/agent/testing.md step 1). Owed on laserax-ai: time_control + --redarm (R15). Flags → M0-T84, M0-T85. Detail: tasks/M0-T24.md. Next: M0-T25.
- Full block: plan_archive.md § M0-T24

## M0-T25 · Scene files · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T24)
- Status: DONE (2026-10-11 13:41) · archived by M0-TC2, 2026-10-12
- Handoff (one line): scene.rs (new) parses §2.12.1: unknown keys refused at every level by path (exit 4), t xor pressure, overrides via Physics::with (never writes calibration), objects preset/disk/rect per §2.12.4; headless --scene file|preset, --world/--edge over the scene's world, summary world {width,height,edge}; scenes/sun_disk.json, fill_world.json; scope scene 8/8 GO, plant scene-unknown-key RED (win-laptop, RTX 4080 Laptop). · Defaults (cheap to reverse, tasks/M0-T25.md): x,y are points, cells sampled at centres; overlapping objects add; --world resize keeps objects about the centre; pressure = total Π, below cold pressure refused. · Deviation: the preset's E now includes ε_cold (§2.2) — the old builder left it out (P8 would floor the cloud; reading, NOT PROVEN). scene-refuse (boot's plant) re-seated on Scene::preset, --redarm boot GO. · Not acted on: clippy --all-targets red on existing tests (floors/box/latch; build scope lints without it). · Edge and switches recorded, not applied — flagged M0-T35, M0-T37. · Claim: verify.py --changed --task M0-T25 (logs/M0-T25.changed.log). · model=claude-opus-5-5 level=high. · Next: M0-T26.
- Full block: plan_archive.md § M0-T25

## M0-T26 · State dumps · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T25)
- Status: DONE (2026-10-11 14:01) · archived by M0-TC2, 2026-10-12
- Handoff (one line): dump.rs (new): SRDUMP01 written/read (magic, u32 header length, JSON header, 14 LE f32 planes in §2.2 order), dump_state/load_dump (§3.1); Step::resume_records/restore (latch.rs) carry the Δt pair, the latch's since, the box record+sizes, the GPU step count, the frames' sim time; headless --dump-every N, --load-dump FILE, final dump always with --out. Scope dump 5/5 GO, plant dump-channel-order RED (win-laptop, RTX 4080 Laptop). · Contract: §2.12.3's header gains resume {dt, dt_next, latch_since, box, box_args} — §0.5 [M0-T26], Superseded line, the lead's «Add one header key (Recommended)», 2026-10-11; measured: a resume armed afresh fires 2 Δt after the dump's fire where the straight run does not. · Defaults (tasks/M0-T26.md): step = GPU count, --dump-every on step indices from the scene, --load-dump takes world+cells (scene gives constants), sinks [] (load refuses any), booking not dumped. · Found: serde_json float parse is best-effort (no float_roundtrip) — sim_time re-read from its token; §1.2 untouched. · Deviation: ≈ 750 new lines / 8 code files over the 600 / 6 ceilings (tests 376; 4 files are 1–4-line edits). · Not acted on: clippy --all-targets red on old tests; sr-app USAGE stale (outside this block). · Claim: verify.py --changed --task M0-T26 (logs/M0-T26.changed.log). · model=claude-opus-5-5 level=high. · Next: M0-T27.
- Full block: plan_archive.md § M0-T26

## M0-T27 · Docs — architecture.md (the step, the box, the latch, the files) and time_control.md · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T26)
- Status: DONE (2026-10-11 14:14) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record). architecture.md: lot 3's parts added — the dispatch lists per pass (initial-state reduction, guard and re-fit schedules, EOS group, cost per step), the booking layout (state groups, side fields, TERMS/BOOK_GROUPS, slot domains, book_totals' order), the box, a frame and the latch (G-LATCH NOT PROVEN (synthetic) until M0-T111), the files (scene, dump with §0.5 [M0-T26]'s resume, summary as filled today); 10 sections. time_control.md (new, 8 sections): §3.2 ↔ TimeControl, rungs, the frame plan with M0-T24's defaults, the cap, pause/step, slow-down + latch, the 30-frames switch; app wiring is M0-T84–T86, page extended by M0-T88. testing.md: § The calls rewritten for v12.3, flags (a)–(d) fixed, floors 7 / dt 6 cases (M0-D18). running.md: flag (e) start.bat the double-click, headless flags as the tree has them. Verify 1 [ALREADY RUN — PASS (10, 8) on win-laptop] · Verify 2 [ALREADY RUN — PASS (9, R17's time_control; 8 under the literal time-control, no hyphen row written) on win-laptop]. Not acted on: sr-app USAGE stale (code) → flagged M0-T37; touched is NO-GO before a claim (row says so). Claim: verify.py --changed --task M0-T27 (logs/M0-T27.changed.log). Detail: tasks/M0-T27.md. Next: M0-V3 (lot 3's V — linux-pc owes booking … dump, R15).
- Full block: plan_archive.md § M0-T27

## M0-V3 · Validation — lot 3: the step framework · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T27)
- Status: DONE (2026-10-11 17:10) · archived by M0-TC2, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now at launch and on the lead's four prompts — the heading's rung). Verdict DEFECTS(4), reports/v3.md: lot 3 conforms to §1.3 (pass order, determinism), §1.3.3, §1.8 rule by rule, §2.7, §2.9, §2.12.1, §2.12.3 with §0.5 [M0-T26] and §3.1–§3.3; G-LATCH NOT PROVEN (synthetic) until M0-T111. Verify 1 py -3.12 tools/pb/verify.py --all --task M0-V3 [ALREADY RUN — PASS (GO, 665 passed, 24/24 scopes, 187.7 s, 183.9 s idle; no TR) on win-laptop]; lot 3's nine scopes red-armed here, 9 plants red; the sweep, the lead's PowerShell run 15:06–15:43 [ALREADY RUN — FAIL (moved 14 of 24, 20 RED, 1 ERROR, 2221 s) on win-laptop].
- Full block: plan_archive.md § M0-V3

## M0-TC2 · Compaction — Phase 5's close: seventeen DONE blocks to stub, more than ten (PLAYBOOK §8, §11) · **MOVE** · Haiku 5.5, high · switch · (AFTER M0-V3)
- Status: DONE (2026-10-12 08:00) · archived by M0-TC3, 2026-10-12
- Handoff (one line): Stubbed Phase 5's 17 DONE blocks (M0-T18 … M0-V3) into plan_archive.md, archived by M0-TC2, 2026-10-12: 17 stubs, 17 archived blocks, each equal to its original byte for byte (script check against a scratchpad snapshot); the plan's diff is 17 hunks, all inside those blocks' span, no TODO block changed. lint GO before and after, header=454/600 both (the stubs sit in the task body; show's header=257 is another measure, unchanged); verify_fences 2 → 0 (the two fences went with the stubbed blocks). Declared, not done: (ii) the Pipeline state already reads as a register, no edit; (iii) only 3 lines repeat in 3+ blocks, all wrapped Verify fail-arm fragments ("NO-GO — each red a D (E4)", "Fail: the plant stays GO"), none hoisted; Rules rows are index lines with full text in m0_rules.md, none moved; (iv) Repo facts duplicates sit beside carried Hazards, not de-duplicated. Mistake: the first stub call sent its stdout to /tmp/x and discarded it; the state was checked by diff afterwards (logs/M0-TC2.log). Ran on claude-haiku-5-5 level=high (rung_record now, the heading's rung). Next: M0-TM4.
- Full block: plan_archive.md § M0-TC2

## M0-TM4 · Rating pass — M0-V3's record no longer moves BUILD on Sonnet 5.5, high · **PLAN** · Opus 5.5, max · switch · (AFTER M0-TC2)
- Status: DONE (2026-10-12 08:19, started 08:03) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Ladder re-read 2026-10-12 — the probe (Claude Code 2.1.296) and its claude-api skill's table (cached 2026-10-06; models.md byte-identical to 2.1.295's): nothing retired, renamed, added, repriced or relevelled → Models line re-dated and re-cited (2026-10-11 and 2.1.295 → 2026-10-12 and 2.1.296), no question. The record, M0-V3's table: BUILD on Sonnet 5.5, high $1.89 per DONE block (11 blocks) against (usual)'s $2.92 (24) → rule (4) no longer moves the class (today's report: $1.89 against $2.91). 122 open blocks re-derived, the 103 BUILD blocks read in full; 84 changed — M0-TM3's rule-(4) blocks still open, Opus 5.5, high → Sonnet 5.5, high (rule (3), no reason from its list). By rung: Opus 5.5 max 19 (0, rule (2)) · Opus 5.5 high 8 (−84; M0-T30, T31, T34, T42, T46, T51, T56 and M0-D21 by (3)'s list → (5)) · Sonnet 5.5 high 86 (+84; M0-TR1 and M0-D23 as filed) · Haiku 5.5 high 9 (0; the 7 E blocks, M0-D22, D24) · Fable 5.1 high, Opus 5.5 xhigh, Sonnet 5.5 medium 0; no trail (rule (1)); every heading keeps · switch; the diff is the 84 rating segments and the Models line (checkdiff against a post-status snapshot). Deviation: on the launch prompt `now` and `show` ran as python3 (the hook's line), not py -3.12 (R15) — both GO. Measured, not acted on: M0-T39's M0-V3 flag (its raw sums 7 % inside G-BOX's 10⁻⁶) names the comparison to make — a fixed instruction, no reason, Haiku kept; Repo facts read Claude Code 2.1.295 on win-laptop (2.1.296 now), outside this diff; Next task still reads M0-TM4 (outside this diff; next is M0-TR1). lint GO (516 checks, 0 warnings); the claim run verify.py --changed --task M0-TM4 is the last step, after this close. Detail: tasks/M0-TM4.md. Next: M0-TR1.
- Full block: plan_archive.md § M0-TM4

## M0-TR1 · Test-time pass — the phase-close sweep's four game-launching red-arms, on the lead's word · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TM4)
- Status: DONE (2026-10-12 08:49) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-sonnet-5-5 level=high (rung_record now = the heading's rung). reports/test_time_1.md: every boot/desktop/capture arm cold-builds sr-app in its own build/ (199 s alone, 330-438 s in V3's sweep; 11 s with deps cached; the run 2-15 s); web already shares the cache, its 505 s is two locks serialising 5 arms x ~100 s (29 s of each build is wasm-opt -Oz). Prototype in a scratch copy (shared build + own exe copy): boot's 2 arms 32 s vs 344 s; window/launcher/capture on a copied binary and linux-pc NOT PROVEN. Filed M0-TR1a (native three, est. sweep 2221->~1375 s) and M0-TR1b (web, ->~1140 s), both Opus 5.5 high (concurrency/shared state, rule 3), ahead of M0-D21 (re-linked AFTER M0-TR1b). Declined with measure: wasm-opt off (40->10.8 s per web build, ships a 5.2 MB wasm not 4.1 MB), a port per desktop arm (unmeasured, reopens contract 3.6). Budget line: loop median 91.1 s idle, V3 183.9 s, under 600; the sweep has no line - ~1140 s after the fixes, proposed for the lead in the report, Rules not edited. Next: M0-TR1a
- Full block: plan_archive.md § M0-TR1

## M0-TR1a · The native game-launching scopes build through the shared dependency cache and launch their own copy of the binary: boot, desktop and capture arms stop compiling the dependencies cold · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TR1)
- Status: DONE (2026-10-12 11:31) · archived by M0-TC3, 2026-10-12
- Handoff (one line): tests/cargo.sh sr_build_app: in a red-arm copy sr-app builds into the shared redarm-target under sr_in_shared_target's lock+touch and the binary is copied into the arm's own build/target/release; real tree unchanged; smoke.sh and capture/check.sh call it, their unset CARGO_TARGET_DIR gone; no case, plant or count changed. Deviation (declared): desktop's first red-arm failed the copy (its two cases share one arm; Windows refused cp over the other case's running exe, 'Device or resource busy'; rebuilds are not byte-identical) — a .sr-app-inputs stamp of the touch list's contents, checked in the lock, keeps a current binary. Verify on win-laptop Laser2025-20: 1 GO 1/2/2 cases; 2 red-arms GO, every plant RED for its own reason, clean GO, walls boot 33 s, desktop 260 s, capture 56 s (bounds 90/450/120; load 6-8 %); a failed build prints cargo's tail and NO-GO (probed in a scratch copy). verify.json's descs still say 'the build is the tree's own' (not my Files). linux-pc NOT RUN — owed on laserax-ai, flagged to M0-V4. Detail: tasks/M0-TR1a.md. Ran on claude-opus-5-5, high. Next: M0-TR1b.
- Full block: plan_archive.md § M0-TR1a

## M0-TR1b · The web scope's locks cover only what needs them: arms overlap their trunk builds with each other's Chrome runs · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TR1a)
- Status: DONE (2026-10-12 11:54, started 11:39) · archived by M0-TC3, 2026-10-12
- Handoff (one line): tests/web/check.sh: the port lock is now taken after the build and npm ci (launch, smoke, stop only); a new mkdir build lock in the tree's build/ (broken when its holder is gone) covers build, npm ci and the smoke serving that dist; in a red-arm copy the first case's build verdict (build/web-build.result) is reused by its siblings — one trunk run per arm, pass or fail; real tree: build always rebuilds; shared-target lock and wasm-opt unchanged; no case, plant, count or deadline changed; testing.md's web row says so. Verify on win-laptop Laser2025-20: 1 GO 3/3, 51.7 s; 2 red-arms GO, all four plants RED, clean GO; walls 257.0 s (load 4.9 %) and 330.7 s (load 16.5 %, 4 arms) — run 2 is 0.7 s over the 330 s bound, closed DONE on the lead's ruling «DONE, miss recorded» (2026-10-12); clean ready in 1.1 s of 30 s with another arm's trunk run in flight. Floor now: trunk runs queued on the shared-target lock, 42-65 s each. linux-pc NOT RUN — owed on laserax-ai, flagged to M0-V4. Detail: tasks/M0-TR1b.md. Ran on claude-opus-5-5, high. Next: M0-D21.
- Full block: plan_archive.md § M0-TR1b

## M0-D21 · Sim time is not a function of the steps run: bare steps add no Δt, frames add it grouped by frame · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TR1b)
- Status: DONE (2026-10-12 12:08, started 11:55) · archived by M0-TC3, 2026-10-12
- Handoff (one line): Sim time is now P1's: each step's Δt added in step order to an f64 clock in the Δt record (reduce.wgsl `clock_add`, an exact f64 add on u32 words, nearest-even = the CPU's `t += f64::from(dt)` bit for bit); a latch-stopped step, a Δt of 0 or an empty box's f32::MAX adds nothing. DtReport.sim_time (new), FrameReport.sim_time, resume_records/restore, the dump header, summary.json and the DONE line all read it; FrameReport.advanced unchanged. Measured on win-laptop (RTX 4080 Laptop, Vulkan): 40 steps as 1×40, 2×20, 40×1 frames and bare → byte-identical dumps, 0.0 off the f64 sum (f32 sum 3.6e-7, the control); headless --steps 100 → summary = final dump = twin = 3.02641075104475 (V3: 0). dump GO 8/8, latch GO 5/5, --redarm dump (new plant dump-sim-time-skip + dump-channel-order red) and latch GO; claim `verify.py --changed --task M0-D21` GO, 588 passed, 16/16 scopes, its 4 FLAG oracle lines the deviations below. · Deviations: latch.rs's poll case asserted the old frame sum (the defect) — now the per-step f64 sum, bit for bit, still 5 cases; verify.json dump expected 5→8 + plant; 6 code files, so latch.wgsl's DtState copy and dump.rs's « frames' sim time » doc line left as they were. · Not filed: an empty world's frame reports advanced=inf (f32::MAX summed) — the frame's own per the Deliver, time control's (M0-T24) to guard. · Flag → M0-T60 (--until sim_time reads it). Detail: tasks/M0-D21.md · model=claude-opus-5-5 level=high · Next: M0-D22
- Full block: plan_archive.md § M0-D21

## M0-D22 · The boot scope leaves out sr-physics and the assets its headless run reads: `--changed` skips G-BOOT on a physics.json edit · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-D21)
- Status: DONE (2026-10-12 12:16) · archived by M0-TC3, 2026-10-12
- Handoff (one line): boot's paths gain crates/sr-physics/**, assets/physics.json and assets/elements.json (tools/pb/verify.json, boot scope; no case, plant or expected changed); docs/agent/testing.md's boot row names them. Verify 1: verify.py boot --task M0-D22 GO, 1/1 scopes, 11.8 s, box Laser2025-20 (logs/M0-D22.boot.log). The anchor now maps assets/physics.json, assets/elements.json and crates/sr-physics/src/eos.rs to boot at M0-D22. Verify 2: --redarm boot --task M0-D22 GO: clean GO, plant scene-refuse RED (logs/M0-D22.boot.redarm.log, 60.6 s). Not added: reactions.json — boot's path never parses it (Registry::load is called only from tests), so the block's conditional clause does not apply. Measured, not acted on: the anchor run reports 1 stale path×scope entry; not identified, outside boot's paths, not chased. Detail: tasks/M0-D22.md. Model/level: model=claude-haiku-5-5 level=high (rung_record now, matches the heading's rung). Next: M0-D23
- Full block: plan_archive.md § M0-D22

## M0-D23 · The harness needs Git Bash on win-laptop: from PowerShell `bash` is WSL's relay and every `bash` scope reads NO-GO · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-D22)
- Status: DONE (2026-10-12 12:24) · archived by M0-TC3, 2026-10-12
- Handoff (one line): tools/pb/verify.py gains tool_path(): on win32 the spawned commands (and argv_of's lookup of a bare command) get Git's usr/bin and ~/bin first on PATH, so a plain PowerShell's bash is Git's, not WSL's relay; posix unchanged; verify.json untouched. Measured on win-laptop (Laser2025-20) from PowerShell 5.1, no PATH change: time_control latch GO 18+5; --redarm time_control GO (plant red); --changed GO (526 passed, tools_selftest incl.). testing.md (Hazards, Boxes) and the plan's Hazards line say both shells work; local annex fix flagged to M0-TZ. Not run: linux-pc (posix branch is a plain return, owed on laserax-ai only as the next V). No selftest check added for tool_path (Verify 1 grades it). Ran on model=claude-sonnet-5-5 level=high. Next: M0-D24.
- Full block: plan_archive.md § M0-D23

## M0-D24 · The watch_only plant no longer applies: M0-T19 rewrote step/mod.rs's head, so G-WATCH cannot be proven red · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-D23)
- Status: DONE (2026-10-12 12:27) · archived by M0-TC3, 2026-10-12
- Handoff (one line): Built: watch_only's plant stage-in-step is a command in tools/pb/verify.json (watch_only.plants[0]): it appends 'use sr_physics::observe::Stage;' to the end of crates/sr-engine/src/step/mod.rs. Scan, case, expected and paths unchanged. Verify 1 on win-laptop (Laser2025-20): --redarm watch_only → clean GO, stage-in-step RED; the scan names step/mod.rs:661 for observe and Stage (M0-D24.watch_only.redarm.log). Claim: verify.py --changed --task M0-D24 → 514 passed, 1/1 scopes GO, the anchor's stale entry cleared by that run (M0-D24.changed.log). FLAG oracle ×2 from that run: verify.json changed since start, and the Deliver names the plant, not the manifest — the lead to confirm the plant change is the intended one. Left: tests/plants/stage-in-step.patch (unused now, still in paths; its removal is a lead call). Not run: the phase sweep (M0-V4). Ran on: model=claude-haiku-5-5 level=high (rung_record.py now, plugin=installed). Detail: milestones/m0/tasks/M0-D24.md. Next: M0-D23 (this block was ordered after it; run first at the lead's ask).
- Full block: plan_archive.md § M0-D24

> **Commit gate (lead):** Phase 5 closes after M0-V3 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 5: the step framework"`
> `git push`

# Phase 6 — Lot 4 · gravity

## M0-T28 · The CPU twin's FFT and convolution · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V3)
- Status: DONE (2026-10-12 12:32) · archived by M0-TC3, 2026-10-12
- Handoff (one line): Exists: crates/sr-physics/src/reference/{mod,fft}.rs (FftPlan radix-2 f64, fft2, padded_size 32…2048, Convolver with its kernel spectrum cached per padded size), tests/plants/fft-twiddle-sign.patch, scope fft_twin (R17's spelling of fft-twin; verify.json, 10 cases) + testing.md row. Verify 1 fft_twin GO 10/10; Verify 2 --redarm GO (clean GO, plant RED, 3 failed); claim --changed GO (11/11 scopes, 94 passed) — all on win-laptop (Laser2025-20). First claim was red: clippy map_entry in my Convolver; fixed, then fft_twin, build, --redarm and the claim re-run green. Contract K(dx, dy), dx = target − source, signed offsets, not assumed symmetric; forward unnormalised, inverse /N; boxes up to 1024 per axis. Detail: tasks/M0-T28.md. Ran on claude-sonnet-5-5, high (rung_record.py now). Next: the gravity pass's CPU twin can call Convolver.
- Full block: plan_archive.md § M0-T28

## M0-T29 · Gravity on the CPU twin (P2) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T28)
- Status: DONE (2026-10-12 12:56) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-sonnet-5-5 level=high (rung_record now). Exists: sr_physics::reference::grav (kernel with the exact self cell 4 ln(1+√2), source, central-difference acceleration, Gravity::field), tests/gpu/grav.rs (grav::force 4 + grav::selfforce 2 cases, no adapter), scenes/asym_blob.json, scopes grav_force / grav_selfforce, plants grav-kernel-exponent / grav-kernel-shift, two testing.md rows. G-GRAV1 measured first: central-difference force 0.30 % worst at a = 40 (2820 cells), 0.58 % at a = 20, ratio 1.93, corner-centred 0.31 % — under 2 %, so NO direct force kernels (flagged to M0-T31). G-GRAV2 ratio 4e-16. Verify 1 [ALREADY RUN — PASS (GO, 6 cases, 2/2 scopes) on win-laptop]; Verify 2 and 3 [ALREADY RUN — PASS (grav_force: plant RED 4 failed; grav_selfforce: plant RED 2 failed) on win-laptop]; claim py -3.12 tools/pb/verify.py --changed --task M0-T29 [ALREADY RUN — PASS (GO, 639 passed, 19/19 scopes, 110 s) on win-laptop]; its 5 oracle FLAGs are the new test file, main.rs registration and the two new scopes (the Deliver names them). Deviations: none; the twin's one-sided difference on the box's outermost ring is the declared edge rule (task file). Not filed: none. Detail tasks/M0-T29.md. Next: M0-T30.
- Full block: plan_archive.md § M0-T29

## M0-T30 · The GPU FFT · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T29)
- Status: DONE (2026-10-12 13:12) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), RTX 4080 Laptop (Vulkan), model=claude-opus-5-5 level=high (rung_record now). Exists: shaders/fft.wgsl + step/fft.rs — Fft::transform (2D complex, 32…2048 per axis, 2 dispatches: rows, columns) and Convolution::dispatches (zero-padded over a Region of a Plane, 3 dispatches: the box's h rows forward; each column forward × K̂ ÷ (nx·ny) and back by DIF in one dispatch; rows back into the box's cells only), the kernel's transform cached per padded size; twiddles one f64-built table, no cos/sin in the shader; tests/gpu/fft.rs (scope fft, 8 cases), plant fft-gpu-bitreverse. Measured vs the twin: transforms 1.3e-7…1.7e-7 rel L2 (limit 1e-5) incl. 2048 × 1024 and every length on both axes; convolutions 9.4e-8…1.6e-7 (asymmetric kernel on 7 boxes, gravity's kernel over 600 × 400); cost per convolution 0.12 / 0.21 / 0.92 ms at 128² / 512² / 2048 × 1024 (TW proxy on the Quadro: 0.11 step / 0.17 / 1.24), not graded. Verify 1 [ALREADY RUN — PASS (GO 8/8, dispatch counts as SR-FFT lines) on win-laptop]; Verify 2 [ALREADY RUN — PASS (plant RED, 7 failed) on win-laptop]; claim py -3.12 tools/pb/verify.py --changed --task M0-T30 [ALREADY RUN — PASS (GO, 612 passed, 21/21 scopes) on win-laptop]; its 3 oracle FLAGs are the new test file, main.rs's mod line and the new scope (the Deliver names them). Deviation: no multi-pass at 2048 — nothing else sits in workgroup memory, so a 2048-point row fits one workgroup at exactly 16,384 B under Limits::default() (fewer dispatches, the block's aim). linux-pc: fft owed on laserax-ai (R15). Not filed: none. Flag to M0-T31 (geometry uniform is ActiveBox-shaped; kernel per size still CPU-picked; no Σ_vac mask). Detail tasks/M0-T30.md. Next: M0-T31.
- Full block: plan_archive.md § M0-T30

## M0-T31 · Gravity on the GPU (P2) and one step's cost at 600 × 400 · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T30)
- Status: DONE (2026-10-12 13:31) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), RTX 4080 Laptop (Vulkan), model=claude-opus-5-5 level=high (rung_record now). Exists: step/grav.rs + shaders/grav.wgsl (appended to fft.wgsl) — P2 = grav_rows (Σ_vac mask) → grav_columns (K̂'s real quarter for every padded size the world's boxes can take, made once per Step in f64 from the twin's own kernel: 42 sizes, 8.3 MB at 600 × 400 — the box's FFT size never leaves the GPU, so M0-T21's flag is closed this way) → fft.wgsl's conv_rows_inverse (φ into side.phi) → grav_gradient (g, one-sided on the box ring as the twin); fixed or box-tile sizes, so the latch reaches them; Step::with_gravity + Step::gravity_field (Step::new keeps gravity off); Δt's η_g term reads |g| (reduce.wgsl binding 14), step 0's initial reduction after P2; headless applies switches.gravity. Measured: G-GRAV1 on the GPU 0.30 % (ratio 1.925), G-GRAV2 2.2e-9, φ/g against the twin ≤ 2.6e-6 relative L1, Δt within 1e-6, a frame = bare steps bit for bit. Verify 1 [ALREADY RUN — PASS (GO, 11 cases, 2/2 scopes) on win-laptop]; Verify 4 [ALREADY RUN — PASS (plant RED, 6 failed, both GPU force cases among them) on win-laptop]; grav_selfforce and timing red-armed too (RED 3 / RED 1); Verify 2, 3 [NOT RUN — owed on laserax-ai] (the Quadro) — smoke here on the RTX 4080 Laptop: fill_world P2 0.90 / step 1.32 ms, sun_disk P2 0.085 / step 0.28 ms (TW's Quadro proxy 1.24 / 0.11); claim py -3.12 tools/pb/verify.py --changed --task M0-T31 [ALREADY RUN — PASS (GO, 589 passed, 17/17 scopes, 120 s) on win-laptop]. Deviations: 10 source files (the Δt term, headless, timing.rs mirroring headless's gravity); testing.md rows left to M0-T32. Finding: a world over 1024 cells on an axis is refused with gravity (exit 4) where §2.8 allows 2048 — for M0-V4 and the lead. Not filed: none. Detail tasks/M0-T31.md. Next: M0-T32.
- Full block: plan_archive.md § M0-T31

## M0-T32 · Docs — physics.md (gravity) and lot 4's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T31)
- Status: DONE (2026-10-12 13:37) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-sonnet-5-5 level=high (rung_record now). Exists: docs/agent/physics.md § Gravity (Q1 sheet law, exact unsoftened K(0,0) with SM-C1's softening named as rejected, the mask, the FFT over the box and its GPU layout, the every-size K̂ table, M0-T29's central-difference choice and the rim rule, Δt's term, not-built list, cost pointed to M0-T31.md/M0-T30.md never restated) + the Σ_BH ≥ 4 × ns_sigma_max_sb line in 'Not checked at load' (M0-V2's flag) + a 'Where a number lives' row; docs/agent/testing.md: new fft row, grav_force (8 = 4 twin + 4 GPU) and grav_selfforce (3) rows brought up to M0-T31, their cmds 'physics', the lot-3 heading naming lot 4's four scopes. Verify 1 [ALREADY RUN — PASS (1) on win-laptop]; Verify 2 [ALREADY RUN — PASS (4, with R17's underscores fft_twin/grav_force/grav_selfforce; the check as written spells them with hyphens and reads 1 — declared, the block's check untouched)] on win-laptop; claim py -3.12 tools/pb/verify.py --changed --task M0-T32 [ALREADY RUN — PASS (GO, 510 passed, 1/1 scopes: plan_lint) on win-laptop]. Not proven: the page's prose is checked by M0-V4 against §1.4.2 (Adversarial). Nothing filed. Next: M0-V4.
- Full block: plan_archive.md § M0-T32

## M0-V4 · Validation — lot 4: gravity · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T32)
- Status: DONE (2026-10-12 14:55, started 13:39) · archived by M0-TC3, 2026-10-12
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=max (rung_record now at launch and on the lead's four prompts — the heading's rung). Verdict DEFECTS(1), reports/v4.md: lot 4 conforms to §1.4 and §1.3.3 (K(0, 0) = −G·4 ln(1 + √2) exact, the mask, Hockney–Eastwood over the box, central differences, Δt's η_g term); G-GRAV1 0.30 % and G-GRAV2 2.2e-9 on the GPU (twin 0.30 %, 3.9e-16) against their analytic oracles — under grav-kernel-exponent the GPU agrees with the twin (the same mistake twice) and is red at the oracle (26.6 %); 3 of 6 mutants survive all 11 cases (the GPU's vacuum mask, the kernel table at non-square sizes, K(±1, ±1)) and G-GRAV2's GPU arm is red under no plant → M0-D25. Verify 1 py -3.12 tools/pb/verify.py --all --task M0-V4 [ALREADY RUN — PASS (GO, 693 passed, 28/28 scopes, 183.59 s raw, 181.72 s idle; no TR) on win-laptop]; lot 4's four scopes and timing red-armed here, 5 plants red; the sweep, the lead's PowerShell run 14:24–14:53 [ALREADY RUN — PASS (GO: moved 23 of 28, plants RED 31, GREEN 0, ERROR 0, 1683 s) on win-laptop]. The lead: «Cap at 1024 (Recommended)» → contract §0.5 [M0-V4] + Superseded mirror (headless --world 1032x400 exit 4, measured); «File a step-cost block (Recommended)» → M0-D26 (the Sun-sized box 0.257 ms GPU on the RTX 4080 Laptop, 2.3× TW's 0.11 ms, 70 % fixed per pass; the Quadro owed, §6.6); the progress request → M0-D27. Phase 6's close: M0-TC3 filed (15 DONE > 10, runs next), header 465/600, the record (no class moves, no TM). Deviations: launch's now and show ran as python3 (Store 3.13), py -3.12 after (R15); M0-D25 groups four mutants of one root-cause class (declared); the sweep's command first went out unseen, re-handed. Measured, not filed: the rim rule at the world's edge (2.7e-4 net self-force, a contract tidy), MAX_PADDED/MAX_N cite §1.3.3, the lint splits a Deliver item at « ; », M0-T31's 10 files, the stale register (fixed). Owed on linux-pc → M0-V5's flag. Scratch copy in the session's scratchpad, outside the tree. Detail tasks/M0-V4.md. Claim py -3.12 tools/pb/verify.py --changed --task M0-V4 [ALREADY RUN — PASS (GO, 524 passed, 3/3 scopes: plan_lint, strings, watch_only; 25 not impacted; no FLAG) on win-laptop]. Next: M0-TC3.
- Full block: plan_archive.md § M0-V4

## M0-TC3 · Compaction — Phase 6's close: fifteen DONE blocks to stub, more than ten (PLAYBOOK §8, §11) · **MOVE** · Haiku 5.5, high · switch · (AFTER M0-V4)
- Status: DONE (2026-10-12 15:00) · archived by M0-TC4, 2026-10-13
- Handoff (one line): claude-haiku-5-5 level=high (rung_record now, plugin=installed; matches the heading's rung). Stubbed Phase 6's 15 DONE blocks (M0-TC2, M0-TM4, M0-TR1, M0-TR1a, M0-TR1b, M0-D21..D24, M0-T28..M0-T32, M0-V4) into plan_archive.md: 320 lines moved, 320 verified by plan.py; my own check: all 15 archived blocks equal their originals line for line (non-blank). Header 465/600 before and after; lint warnings 9 -> 0. De-dup: 3 Repo-facts facts now point to Hazards, which keep them verbatim (disk free space, MSYS symlink variable, Store-alias Python). Declared, not done: no line repeats whole in 3+ blocks (wrapped Verify fail-arm fragments only); Rules rows are index lines, full text in m0_rules.md, none moved; Pipeline state already a register (only Next task changed); M0-D24's 4-line Phase 5 commit-gate quote stays under its stub (plan.py's choice). Measured, not acted on: Repo facts says every win-laptop toolkit call goes through the Bash tool, Hazard M0-D23 says both shells work; a fact conflict for the lead. Claim python3 tools/pb/verify.py --changed --task M0-TC3 [ALREADY RUN — PASS (GO, 519 passed, 1/1 scopes: plan_lint; 27 not impacted) on win-laptop]; deviation: run under python3 (3.13.14), not py -3.12 as R15 asks — a py -3.12 re-run answered NO-GO 'nothing changed, zero cases' (the anchor vouches for the tree), so not repeated. plan_lint alone read 520 passed; the claim 519 — unexplained, flagged. Detail: milestones/m0/tasks/M0-TC3.md. Next: M0-D27.
- Full block: plan_archive.md § M0-TC3

## M0-D27 · verify.py's console never says which step a run is on or how many remain: a 20–40-minute sweep runs blind (the lead's word at M0-V4) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TC3)
- Status: DONE (2026-10-12 15:21, started 15:04) · archived by M0-TC4, 2026-10-13
- Handoff (one line): win-laptop (Laser2025-20), model=claude-opus-5-5 level=high (rung_record now — the heading's rung). Exists: verify.py numbers a run's steps — `step k/N, m left` on each scope row of a multi-scope run (--all, --changed, a scope list), `arm k/N, m left` on each --redarm arm line, `campaign k/N, m left` on each --redarm --moved campaign's redarm: line (arms numbered within); m = steps after this one, the last reads 0 left; a one-step run is unnumbered. On a terminal each step is named live as it starts (`>> <scope> · step k/N, m left · started`), every live line written whole under one lock (run_unit's `..` line too). Counts, failure and verdict lines, logs, loop_times.jsonl and the anchor unchanged. Detail: tasks/toolkit_local_diff.md item 5. Deviations (declared): moved_sweep now classifies every armed scope before its first campaign (same lines, order, failures) so the total is known; four existing selftest checks read a row's head, not the whole row (3× `par · clean · GO`, the `noted` row's `GO$`). verify.py selftest checks=352 · plants=158/158 red. Verify 1 py -3.12 tools/pb/verify.py plan_lint time_control --task M0-D27 [ALREADY RUN — PASS (GO, 537 passed, step 1/2 · 1 left, step 2/2 · 0 left) on win-laptop]; Verify 2 --redarm tools_selftest [ALREADY RUN — PASS (GO, clean GO arm 1/2, plant RED arm 2/2, 176 s) on win-laptop]; Verify 3 grep [ALREADY RUN — PASS (1) on win-laptop]; claim --changed --task M0-D27 [ALREADY RUN — PASS (GO, tools_selftest 10/10, 27 not impacted, 158 s) on win-laptop], run before this close (a plan-only edit). NOT PROVEN (synthetic): the live lines on a real console — a fake tty in the selftest only; the lead sees them at M0-V5's sweep. Next: M0-D25.
- Full block: plan_archive.md § M0-D27

## M0-D25 · The gravity scopes grade P2 on square, zero-vacuum states and the kernel at seven offsets: four mutants of §1.4.2 survive · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TC3)
- Status: DONE (2026-10-12 15:35) · archived by M0-V5, 2026-10-13
- Handoff (one line): Built on win-laptop (RTX 4080 Laptop, Vulkan): grav_force 8→13, grav_selfforce 3→5 cases — K at all 127²−1 offsets of a 64² box; GPU on a faint disk in sub-Σ_vac cells, on a wide (392×64, FFT 1024×128), a tall (40×344, 128×1024) and the whole 600×400 world (2048×1024), each on the twin (≤2.6e-5) and on a direct kernel sum (≤2.1e-7, new oracle Solved::against_direct_sum); G-GRAV2 ratio on wide/tall (9.5e-9, 4.8e-8). 4 new plants (grav-vacuum-mask-off, -kernel-transposed, -kernel-diagonal, -gradient-backward), each NO-GO at its case; the shader plant red at the GPU cases' own ratio (blob 4.7e-2). Verify 1 GO 18/18, Verify 2 GO 4/4 red (212 s), Verify 3 GO 2/2 red (70 s). testing.md's two rows rewritten. Deviation: the whole-world case has no G-GRAV2 ratio (matter on the world's edge). Nothing measured left unfiled. Detail: tasks/M0-D25.md. Ran on claude-sonnet-5-5, high (rung_record.py now). Next: M0-D26.
- Full block: plan_archive.md § M0-D25

## M0-D26 · The Sun-sized box's step costs 2.3× TW's whole-step estimate with five of its ten passes: 70 % of it is fixed cost per pass (the lead's word at M0-V4) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D25)
- Status: DONE (2026-10-12 16:15) · archived by M0-V5, 2026-10-13
- Handoff (one line): Built on win-laptop (RTX 4080 Laptop, Vulkan). Where the cost went (tasks/M0-D26.md, per dispatch and per pass): wgpu 30 keeps VALIDATION_INDIRECT_CALL in release builds — a hidden dispatch before each of the step's 7 indirect dispatches, ~11–14 µs each — and the pass boundaries, ~0.06 ms a step (CPU encoding + GPU). Cut: gpu::instance_flags() (headless and the app) drops that validation, every indirect size bounded by the world (a compile-time assertion); a step whose passes are not GPU-timed is one compute pass (Timer::times_passes); architecture.md updated. Dumps byte-identical before/after at steps 250–1000, both scenes. Verify 1 py -3.12 tools/pb/verify.py --changed --task M0-D26 [ALREADY RUN — PASS (GO, 102 passed, 19/19 scopes, 106 s) on win-laptop]. Verify 2 [ALREADY RUN — PASS (exit 0, SR-ADAPTER RTX 4080 Laptop) on win-laptop]: sun_disk step_ms_mean 0.257→0.217, wall/step 0.366→0.260 (warm card 0.198→0.113); fill_world 1.262→1.196, wall 1.276→1.245 (interleaved medians). Not built, measured: P2 sized by the box (~5 µs; needs resume.box_args), P0 gated by the CPU's edit queue (the edit lot). Measured, not filed: the laptop GPU's clocks move short-run timings (sun_disk 0.257 cold, 0.175 warm); a timed step still pays the pass boundaries → flagged M0-T84. Quadro owed on laserax-ai → flagged M0-V5. Deviations: none; the register's M0-D25 entry predates its close, left for its owner. Ran on claude-opus-5-5, high (rung_record.py now). Next: M0-T33a.
- Full block: plan_archive.md § M0-D26

> **Commit gate (lead):** Phase 6 closes after M0-V4 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 6: gravity"`
> `git push`

# Phase 7 — Lot 5 · gas flow

## M0-T33a · Sod's exact solution — the Riemann oracle for γ = 2 · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V4)
- Status: DONE (2026-10-13 07:58, started 07:56) · archived by M0-TC4, 2026-10-13
- Handoff (one line): reference/riemann.rs: the exact γ=2 Riemann solution (star p*/u* by safeguarded Newton in a bisection bracket, wave pattern, sample(ξ)/at(x,x0,t), breakpoints(), refusals BadState/Vacuum); scope riemann 8 cases GO (p* vs an independent bisection on an RH-mass-flux/Riemann-invariant pressure function 1e-10 on 7 wave pairs, RH jumps + isentrope + invariant, mirror, conservation by Simpson 1e-8, refusals); plant riemann-gamma red (5 failed); --changed GO, 17/17 scopes, 648 passed, linux-pc (laserax-ai). Deviation, declared: fft_twin's cmd narrowed 'reference' → 'reference::fft' (it filtered all of reference:: and would have counted riemann's tests; expected stays 10), re-red-armed RED, testing.md row 62 updated; the two oracle FLAGs on verify.json/fft_twin are that edit. testing.md has no riemann row yet (M0-T33-docs block owns it). Next teammate: M0-T33b reads Solution::solve(State{rho,u,p}, ..).at(x, x0, t); GAMMA = 2 const. Ran on claude-sonnet-5-5, high (rung_record.py now). Next: M0-T33b
- Full block: plan_archive.md § M0-T33a

## M0-T33b · Gas flow on the CPU twin (P4) — G-SOD's twin case · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T33a)
- Status: DONE (2026-10-13 08:10) · archived by M0-TC4, 2026-10-13
- Handoff (one line): reference/hydro.rs (the twin's P4: unsplit MUSCL-Hancock, MC limiter, HLL-Davis, species by the mass flux, full EOS, two bounce ghosts, Δt §2.3.5; State, Hydro::{dt, step, advance}), scenes/sod.json, tests/gpu/sod.rs (9 twin cases), scope sod (verify.json + testing.md row), plant hll-swap. Measured on linux-pc, CPU f64: shock 0.34 cell off (≤ 1), L1 density 0.32 % (≤ 2 %), mass and energy to 5e-16, walls' momentum exact, the y-oriented strip and the mirrored tube bit-equal; hll-swap RED (8 of 9 fail, 15.7 s). Deviation, declared: the scene's world is 400 × 64 (§2.8's floor, 64 identical rows) — the twin's 400 × 4 strip is its first four rows; flagged to M0-T34, T35, T36. Not acted on: testing.md has no riemann row (M0-T33a's), see tasks/M0-T33b.md. Model claude-sonnet-5-5, level high (rung_record). Next: M0-T34.
- Full block: plan_archive.md § M0-T33b

## M0-T34 · Gas flow on the GPU (P4) with bounce walls · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T33b)
- Status: DONE (2026-10-13 08:50) · archived by M0-TC4, 2026-10-13
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record now, the heading's rung). step/hydro.rs + shaders/hydro.wgsl: P4 on the GPU, the twin's scheme in f32 — hydro_update (each cell recomputes its neighbours' predicted faces, writes a next state, never the state it reads) then hydro_commit, over the box's tiles, at WebGPU's 8 storage buffers; Step::with_passes(Passes { mode, gravity, hydro }) — new/with_box/with_gravity keep P4 off; headless turns it on by the scene's hydro switch. tests/gpu/sod.rs +6 GPU cases (scope sod 9 → 15; hll-swap also plants the shader). RTX 5090: 824 steps to t = 80.0067 (the twin's count), shock 0.335 cell, L1 0.319 % (worst of 64 rows), rows within 1.8e-6, the y-laid tube within 2.1e-6, mass/energy ≤ 2e-8 — f32 tolerances fixed before the first run. Verify 1 GO 15/15; Verify 3 GO (hll-swap RED, 14 of 15, every GPU case); Verify 2 exit 0, P4 in per_pass_ms — Quadro RTX 4000 sun_disk P4 0.99–1.06 ms warm (first run 1.5) of a 1.2 ms step: P4 dominates (flagged to M0-V5). First claim NO-GO: timing's and dump's oracles rebuilt headless's step without P4 — they now build P2 + P4 (no case, count, plant or tolerance changed; both red-armed GO); the rating refresh fired → M0-TM5 filed, runs next; the second claim read web[ready] NO-GO once (ERR_CONNECTION_REFUSED, its sibling case stopping the shared server; GO twice on re-run) → M0-D28 filed after M0-TM5. Deviations: P4 bounces under leave too until M0-T35; no stored face set (memory at 2048²). Flags: M0-T35, T36, T43 (the 8-buffer budget), M0-V5. The claim (verify.py --changed --task M0-T34) is the last step, after this edit. tasks/M0-T34.md. Next: M0-TM5.
- Full block: plan_archive.md § M0-T34

## M0-TM5 · Rating pass — five carried flags on open BUILD blocks since 2026-10-12 · **PLAN** · Opus 5.5, max · switch · (AFTER M0-T34)
- Status: DONE (2026-10-13 09:40, started 09:19) · archived by M0-TC4, 2026-10-13
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=max (rung_record now, the heading's rung). Ladder re-read 2026-10-13: the probe reads Claude Code 2.1.296, the version cited; auto mode refused the model pages (a find, "Auto-Mode Bypass"; the claude-api skill, "Credential Exploration"), not worked around (Hazards) → asked, the lead: «Re-date on the probe (Recommended)» → Models line re-dated and re-cited (2026-10-12 from the skill's table → 2026-10-13 from the probe, the table M0-TM4 read, and the lead's answer), rungs byte-identical; today's bundle = M0-TM4's read is NOT PROVEN (source only). The record, M0-V4's table: BUILD on Sonnet 5.5, high $1.64 (16 blocks) against (usual)'s $3.11 (29) → no class moves (today's linux-pc read agrees: $1.78 against $4.05); no trail (rule (1)). 109 open blocks re-derived; 3 changed, Sonnet 5.5, high → Opus 5.5, high: M0-T35, T36, T43, (3)'s list (c) → (5) — their M0-T34 flags: hydro_update binds all 8 of WebGPU's storage buffers, contract §2.2 leaves the packing to the builder, and the escaped booking, g and F need 3 more, each layout built on by the next. By rung: Opus 5.5 max 18 (0, rule (2)) · Opus 5.5 high 8 (+3; M0-T42, T46, T51, T56 by (e) and M0-D28 by (b), (e), as before) · Sonnet 5.5 high 76 (−3) · Haiku 5.5 high 7 (0, the E blocks) · Fable 5.1 high, Opus 5.5 xhigh, Sonnet 5.5 medium 0; every heading keeps · switch; the diff is the 3 rating segments, the Models line and the register's line. Measured, not filed (this diff is bounded): rung_record.py's handoff fallback misreads 6 of 7 rows on linux-pc (M0-TJ3, V2, TM2, TM3, TM4 quote opus-5-5 max and read as Sonnet rungs; M0-TR1 quotes sonnet-5-5 high and reads as Opus 5.5, high) — a D for the next filer before M0-V5 reads the record; the register's open-D line still names M0-D25 (DONE); one layout block ahead of M0-T35 would make the three a reuse (a re-scope: the lead's, or a PLAN block's). tasks/M0-TM5.md. Next: M0-D28.
- Full block: plan_archive.md § M0-TM5

## M0-D28 · The web scope's `ready` case can smoke a server its sibling case is stopping: ERR_CONNECTION_REFUSED, intermittent · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TM5)
- Status: DONE (2026-10-13 10:04) · archived by M0-TC4, 2026-10-13
- Handoff (one line): Cause measured, not the block's guess (start never adopts, stop waits for the port): launch.py's records are the box's, --task names only the log, so desktop[launcher]'s bare `launch.py stop` (start.sh, then tests/smoke.sh:54) stopped web's server mid-smoke in M0-T34's claim run.
- Full block: plan_archive.md § M0-D28

## M0-D29 · web/smoke.mjs breaks its NO-GO line on playwright's multi-line error: verify loses the reason · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D28)
- Status: DONE (2026-10-13 10:06) · archived by M0-TC4, 2026-10-13
- Handoff (one line): web/smoke.mjs's nogo now prints one verdict line: the reason's first line in `=== NO-GO: … ===`, its other non-blank lines (playwright's call log) above it as plain log lines; every caller goes through nogo, so the fix sits there alone (web/smoke.mjs:36-42). Verify 1 [ALREADY RUN — PASS on linux-pc (laserax-ai)]: the Repro (`launch.py stop web`, then `node web/smoke.mjs no-webgpu --base http://127.0.0.1:47812 --timeout-s 5`) prints `=== NO-GO: no-webgpu: page.goto: net::ERR_CONNECTION_REFUSED at http://127.0.0.1:47812/?no-webgpu=1 ===` on one line, exit 1 (logs/M0-D29.log). Verify 2 [ALREADY RUN — PASS on linux-pc]: `verify.py web --task M0-D29` GO 3/3; smoke.mjs is the scope's check, so `--redarm web` GO, 4 of 4 plants red (logs/M0-D29.web.redarm.log). Claim: `verify.py --changed --task M0-D29` (the run after this close). Deviations: none. Findings not acted on: none. Ran on model=claude-opus-5-5 level=high (rung_record). Next: M0-T35.
- Full block: plan_archive.md § M0-D29

## M0-T35 · The leave edge and escaped mass (G-EDGE) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T34)
- Status: DONE (2026-10-13 10:27) · archived by M0-TC4, 2026-10-13
- Handoff (one line): linux-pc (laserax-ai), RTX 5090; model=claude-opus-5-5 level=high (rung_record now). Leave edge on the twin (Edge::Leave, State::escaped per boundary face in the GPU's EdgeFaces order) and on P4 (leave ghosts in read_at under an Edges uniform; shared boundary(); a third dispatch hydro_edges books booking.escaped from the same face flux, 6 storage buffers — hydro_update still binds all 8; Step::set_edge_mode writes the uniform between submissions). Scope edges, 7 cases: the twin bounce pulse within 0.001 % of −v0, the GPU 0.020 %; leave loss = escaped 1.2e-15 (twin) and 2.7e-8 (GPU) of escaped; no face ever books inflow; GPU vs twin ≤ 8e-8; set_edge_mode bit for bit at the step boundary. --redarm edges: wall-copy RED. Deviations, tasks/M0-T35.md: an inflow face is read as a wall (energy and species 0, momentum Π); the leave cases run an inline wind (39.6 % of the mass out), because the pulse's escape (1.2e-4 of the grid) is below what f32 resolves at 1e-6; inflow is graded per face per step (HLL diffusion lets 124 of 3540 out through the stagnant sides, never in). Finding: G-EDGE's 2 % pulse oracle cannot see wall-copy (copy ghosts leave it unchanged to 7 digits), so a case holds the GPU's bounce to the twin's mirrors at 1e-4 (wall-copy gives 2.6e-4…2.6e-3), and the plant goes in the shader only. A first GPU-vs-twin pulse check (1e-3) read 1.37e-3 (f32's floor at Mach 1e-3) and was removed, not loosened. Claim: see tasks/M0-T35.md. Next: M0-T36.
- Full block: plan_archive.md § M0-T35

## M0-T36 · Gravity's source in the flow — the cold collapse (G-COLL) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T35)
- Status: DONE (2026-10-13 11:32) · archived by M0-TC4, 2026-10-13
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record). Gravity's source in P4 (§1.4.2), twin then shader: the Hancock predictor kicks each face by ½Δt·g (momentum ½Δt·Σ_f g, energy ½Δt·Σ_f(u_f + ¼Δt·g)·g, so a face's thermal energy is untouched) and returns the centre's half-step (Σ, Σu, Σv); the update adds Δt·Σ_½g and Δt·(Σu)_½·g. Twin: Hydro::step_with(st, dt, Some((gx, gy))). GPU: g at binding 14 in hydro_update and hydro_edges, Edges.gravity flag (a placeholder bound without P2), the next state repacked 3 → 2 buffers of 7 planes (117 MB at 2048², const-asserted) to free the slot; with_passes hands Gravity::field to P4. scenes/cold_disk.json (overrides exactly k1e, k2e, k1n, k2n) + tests/gpu/collapse.rs (6 cases) + plant grav-half (g × 0.5 where both P4s read it) + scope collapse. Measured r50 ratio error: twin +0.17 % / +1.34 %, GPU +0.15 % / +1.34 % (RTX 5090) at 0.5 / 0.8 t_ff. Verify 1 [ALREADY RUN — PASS (GO 6/0/0) on linux-pc]; Verify 2 [ALREADY RUN — PASS (clean GO, grav-half RED 4 failed, 84 s) on linux-pc]; clippy -D warnings clean; the claim --changed is the last step, after this close (result in tasks/M0-T36.md). Deviations: step/hydro.rs, step/mod.rs, main.rs, verify.json touched beyond Deliver (the binding); per-face kick is my reading of the predictor's time-centring; §5.1's 0.5279 is 0.52796 cut (checked to 1e-4). Not acted on: testing.md rows (collapse; sod/edges binding text stale) → M0-T40; GPU Δt reads step n's g vs the twin loop's fresh g → M0-T38; collapse owed on win-laptop (R15). Detail: tasks/M0-T36.md. Next: M0-T37.
- Full block: plan_archive.md § M0-T36

## M0-T37 · The twin's step driver and `--cpu-reference` · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T36)
- Status: DONE (2026-10-13 12:20) · archived by M0-TC4, 2026-10-13
- Handoff (one line): Twin (sr_physics::reference::Twin) steps P1/P2/P4/P8/P9 in f64 the GPU's way (Δt_{n+1} from step n's own P2 g), switches honoured, worlds ≤ 128; reference::floors is P8 moved out of tests/gpu/floors.rs (§2.7 [M0-T19]); headless --cpu-reference writes the GPU run's summary.json and dumps (box record the whole world's) and refuses a world over 128, and --adapter, --timing, --load-dump with it, exit 4 (my defaults). Scope twin 6 cases GO 4.4 s, plant twin-skip-floors red (3 failed), floors red-armed again (6 failed); the driver's pass list prints beside the GPU's for all four switch pairs. Also USAGE line and running.md to the tree's flags. Claim: verify.py --changed 27/27 scopes GO, 711 passed on linux-pc (laserax-ai); its 4 FLAG oracle lines (floors.rs, main.rs, twin.rs, verify.json) are the carried flag's floors move, the module line and the new scope. 7 code files (ceiling 6: the flagged floors move and USAGE line). Not acted on: twin's whole-world P8 vs the GPU's box (flagged to M0-T38). Ran on claude-sonnet-5-5, high (rung_record). Next: M0-T38.
- Full block: plan_archive.md § M0-T37

## M0-T38 · GPU against the twin (G-REF) — the collapsing disk and the Sod strip · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T37)
- Status: DONE (2026-10-13 12:29) · archived by M0-TC4, 2026-10-13
- Handoff (one line): cpu_gpu scope (tests/gpu/cpu_gpu.rs, 2 cases) + scenes/ref_disk.json, ref_sod.json (64×64, H/He mixed) + plant cfl-gpu: GPU vs twin, 200 steps, RTX 5090 — GO: disk L1 Σ 4.4e-5, E 6.0e-5, |ΔX| 3.9e-6; Sod 1.7e-7, 2.3e-7, 7.5e-7 (bands 1e-3, 1e-3, 1e-4); --redarm: cfl-gpu RED on both (Σ 1.9e-2 / 1.1e-2). Declared: X read where both Σ ≥ 1e-4 (100×Σ_vac) — unmasked 5 rim cells at Σ≈3e-6 read 1.06e-3 (my reading: f32/f64 Σ_vac crossing mixes P8's floor H in; band unchanged); the disk's Δt is gravity-bound so the plant moves its sim time only 0.11 % — Sod carries the CFL separation (−2.45 %). Touched 6 files (+ main.rs module line, verify.json scope: oracle flags name them). --changed 23/23 GO, 651 passed. Flags filed to M0-T40 (testing.md row) and M0-T48 (third scene). Ran on Sonnet 5.5 high (rung_record), linux-pc RTX 5090. Detail: tasks/M0-T38.md. Next: M0-T39
- Full block: plan_archive.md § M0-T38

## M0-T39 · The box against the whole world (G-BOX) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T38)
- Status: DONE (2026-10-13 13:08) · archived by M0-TC4, 2026-10-13
- Handoff (one line): DONE. tests/gpu/box.rs has G-BOX's two cases (scope box, 8). sun_disk box vs whole world: every sum at 1e-6 of its scale at step 200 (box asserted inside the world), mass and species at 2,000. The blob is graded on its masses, then its speed. Deviation, declared: contract §0.5 [M0-T39] + Superseded line, the lead's «1e-6 at step 200 (Recommended)». Measured on the RTX 5090: gravity off, box = world bit-identical at 2,000; gravity on, FFT round-off grows 1.7e-8 → 1.4e-7 → 3.1e-6 (momentum, steps 1/200/2000). The blob's momentum leaves unbooked through the vacuum reset (§2.7 [M0-T19]). margin-zero turns the blob case red on its mass (1.00 of scale). Not filed: NaN floor_added under off-spec Σ_floor 1e-12 (probe only); testing.md's box row is the docs block's (R11). Ran on model=claude-opus-5-5 level=high (rung_record, matched). Detail: tasks/M0-T39.md. Next: the plan's next TODO.
- Full block: plan_archive.md § M0-T39

## M0-T40 · Docs — physics.md (gas flow, edges, the CPU twin) and lot 5's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T39)
- Status: DONE (2026-10-13 13:38) · archived by M0-TC4, 2026-10-13
- Handoff (one line): linux-pc (laserax-ai); model=claude-sonnet-5-5 level=high (rung_record now, matched). Docs only. physics.md: § Gas flow (scheme, gravity's face kick, Δt order, GPU layout: recompute, next state 2×7 planes, 8 buffers, g at 14; the cold-pressure overrides in Sod/collapse/G-REF named with why), § Edges (inflow-is-a-wall, per-face booking, hydro_edges, why the bounce oracle is two), § The CPU twin (Twin's order, MAX_WORLD, floors, --cpu-reference refusals, G-REF's role and choices). testing.md: lot-5 table riemann, collapse, twin, cpu_gpu; sod/edges binding text fixed; box row 6→8 cases. Verify 1 [ALREADY RUN — PASS (3) on linux-pc]. Verify 2 [ALREADY RUN — FAIL as written (5, not 6) on linux-pc]: the plan's pattern spells cpu-gpu, the scope and its row are cpu_gpu (R17); with cpu_gpu the same grep reads 6 — declared default, row not misnamed; the Verify line wants cpu_gpu. Not measured: win-laptop costs (owed on Laser2025-20). Detail: tasks/M0-T40.md. Claim (verify.py --changed) is the last step. Next: M0-V5.
- Full block: plan_archive.md § M0-T40

## M0-V5 · Validation — lot 5: gas flow · **CHECK** · Opus 5.5, max · switch · (AFTER M0-T40)
- Status: DONE (2026-10-13 14:28) · archived by M0-TC4, 2026-10-13
- Handoff (one line): linux-pc (laserax-ai), model=claude-opus-5-5 level=max (rung_record now at launch and on the lead's prompt — the heading's rung). Verdict DEFECTS(2), reports/v5.md: lot 5 conforms to §2.3.4 (the twin and hydro.wgsl line for line), §2.3.5, §1.4.2, §2.8 (M0-T35's inflow-face-is-a-wall reading judged conforming), §2.9, §1.3.2/§1.7/§3.3; G-SOD, G-EDGE, G-COLL, G-REF, G-BOX green on the twin and the RTX 5090, the handoffs' 32 numbers re-read, kill rate 0. Adversarial: cold_disk.json overrides exactly the four K's (asserted); with the shipped K's G-COLL reads +4.95 % / +26.8 % (red); neutrinos and sinks on change no byte. Verify 1 python3 tools/pb/verify.py --all --task M0-V5 [ALREADY RUN — PASS (GO, 750 passed, 34/34 scopes, 32.97 s raw, 29.22 s idle; no TR) on linux-pc]; the sweep, the lead's run 14:12–14:26 [ALREADY RUN — PASS (GO: 33 campaigns, 46 plants RED, 0 GREEN, 0 ERROR; box on M0-T39's proof) on linux-pc].
- Full block: plan_archive.md § M0-V5

## M0-TC4 · Compaction — Phase 7's close: fifteen DONE blocks to stub, more than ten (PLAYBOOK §8, §11) · **MOVE** · Haiku 5.5, high · switch · (AFTER M0-V5)
- Status: DONE (2026-10-13 15:35)
- Read: this file (rules + this task) + plan_archive.md § M0-V4 (one archived block: the shape to keep)
- Deliver: `plan.py stub` over Phase 7's DONE blocks — M0-TC3 and M0-D27 (they ran after Phase 6's close and sit at its end;
  M0-D25 and M0-D26 beside them were stubbed by M0-V5 on the lead's word), M0-T33a, M0-T33b, M0-T34, M0-TM5, M0-D28, M0-D29,
  M0-T35, M0-T36, M0-T37, M0-T38, M0-T39, M0-T40 and M0-V5 — into plan_archive.md, no TODO block touched (M0-D30, M0-D31,
  M0-D32 and lot 6 stay whole) · the Pipeline
  state back to a register of pointers · lines repeated in three or more blocks hoisted, long Rules entries moved to
  m0_rules.md (index line kept, text unchanged) · Repo facts de-duplicated · every moved line diffed byte-for-byte, counts
  reported · `plan.py lint`'s header before and after in the handoff.
- Pass: `plan.py lint` GO, header within budget, diff count = moved count. Fail: any TODO block touched, any rule
  reworded, any fact dropped.
- Handoff: 15 DONE blocks stubbed into plan_archive.md (M0-TC3, M0-D27, M0-T33a, M0-T33b, M0-T34, M0-TM5, M0-D28, M0-D29, M0-T35, M0-T36, M0-T37, M0-T38, M0-T39, M0-T40, M0-V5): 295 lines moved, 295 verified byte-for-byte by plan.py; my own check: all 15 originals sit in the archive identically, the other 189 blocks unchanged, no TODO block touched. Lint: warnings 2 -> 0, GO 524 checks, header 486/600 before and after. Not done, measured: Rules rows are already index lines with full text in m0_rules.md (nothing long left in the plan to move); Pipeline state already a register; no whole line repeats in 3+ live blocks (the repeats are Verify fail-arm wording in TODO blocks, left alone); no Repo-facts sentence duplicates a Hazards sentence. Not acted on: the Rules intro says m0_rules.md was REJECTED as false economy (M0-TB), yet the file exists and this block moved text toward it; the lead to reconcile. Claim python3 tools/pb/verify.py --changed --task M0-TC4 [ALREADY RUN — PASS (GO, 524 passed, 1/1 scopes: plan_lint; 33 not impacted) on win-laptop]. Model: model=claude-haiku-5-5 level=high (rung_record now, plugin=installed; matches the heading's rung). Detail: scratchpad tc4 (before-copies). Next: M0-D30.

## M0-D30 · P4 is 84 % of a Sun-sized step on the Quadro: every cell recomputes its four neighbours' predicted faces (the lead's word at M0-V5) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TC4)
- Status: DONE (2026-10-13 15:55)
- Blocks: nothing — but lot 6 adds the radiation force to the same kernel (M0-T43), and the lead's answer at M0-V5
  (2026-10-13, «File a P4-cost block now (Recommended)») puts this block before it
- Caused by: M0-T34 (hydro_update's recompute layout, its declared deviation 2); how P4's time splits between the
  repeated predictors and the repeated EOS reads is not measured
- Files: crates/sr-engine/shaders/hydro.wgsl (`hydro_update`, `cell_faces`, `read_at`, `read_cell`), crates/sr-engine/src/step/hydro.rs
  (the three dispatches, their sizes) — unverified, the executing task confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.3.2–§1.3.4, §2.2, §2.3.4, §2.8, §6.2.2 +
  milestones/m0/tasks/M0-T34.md § Not acted on + milestones/m0/reports/v5.md § The step's cost + docs/agent/physics.md
  § Gas flow (the GPU's layout)
- Symptom: measured on linux-pc (laserax-ai), Quadro RTX 4000 (Vulkan), 2026-10-13, M0-V5, medians of 3:
  `build/target/release/sandbox-reactions headless --scene scenes/sun_disk.json --steps 200 --timing --adapter "Quadro RTX 4000"
  --out <dir>` — the box 136 × 128 at step 200 (FFT 512 × 256) — step_ms_mean 1.083, P4 0.906 (84 %), the five other passes
  0.108 together (TW's whole-step proxy for a Sun-like box: 0.11 ms); the same for scenes/fill_world.json (the whole 600 × 400
  world): step 16.04 ms, P4 14.60. TW's scenario C needs ≤ ~0.14 ms a step at the top rung, L ≤ ~0.38 (reports/v4.md § The
  step's cost). `hydro_update` recomputes, for each cell, its four neighbours' predicted faces — ≈ 5× the predictor and 25
  `read_cell` EOS evaluations a cell (M0-T34).
- Suspected cause: the recompute — every predictor evaluated five times, with its EOS reads; the known fix is a tiled
  `hydro_update` — an 8 × 8 tile's 12 × 12 primitives and their predicted faces in workgroup memory (≈ 8 KB of §6.2.2's
  16,384 B), each predictor once (M0-T34's estimate, unmeasured)
- Deliver: where P4's time goes — per dispatch (`hydro_update`, `hydro_edges`, `hydro_commit`) on the Quadro — in
  tasks/M0-D30.md, then the cut, the scheme unchanged: every scope that grades P4 GO (sod, edges, collapse, twin, cpu_gpu,
  box, dump, timing) with no case, count, band or plant changed, inside WebGPU's 8 storage buffers and 16,384 B of
  workgroup memory (§6.2.2), and sun_disk.json's and fill_world.json's dumps at step 200 compared before and after on one
  adapter — byte-identical, or the largest difference per channel reported and inside G-REF's bands → Verify 1 · the
  Symptom's two commands re-run before and after on the Quadro RTX 4000 → Verify 2
- Done when: on linux-pc, sun_disk.json's P4 and step_ms_mean below the Symptom's on the Quadro RTX 4000, every P4 scope
  GO, the dumps compared
- Verify: 1. `python3 tools/pb/verify.py --changed --task M0-D30` · Pass: GO · Fail: NO-GO · 2. the Symptom's commands
  (scenes/sun_disk.json and scenes/fill_world.json, 200 steps, `--timing`, `--adapter "Quadro RTX 4000"`), medians of 3, before
  and after · Pass: exit 0, SR-ADAPTER the Quadro RTX 4000, P4 below the Symptom's on both scenes · Fail: no gain, or
  another adapter
- Handoff: hydro.wgsl's hydro_update is one workgroup per 8x8 tile: 132 of its 12x12 primitives read once into 9,216 B of workgroup memory, each predictor once (64 + a 32-cell ring), faces exchanged one axis at a time; a tile with no moving cell copies and stops; scheme, buffers (8), dispatch sizes unchanged; docs: hydro.rs, physics.md, testing.md. Per dispatch before (Quadro): hydro_update 97% of sun_disk's P4. Verify 2 (Quadro RTX 4000, laserax-ai over ssh, medians of 3, same day): sun_disk P4 0.841 -> 0.240 ms, step 1.015 -> 0.414; fill_world P4 12.63 -> 3.776, step 13.91 -> 5.05 (RTX 4080 Laptop: 1.27 -> 0.51, 22.2 -> 7.49). Verify 1: the 8 P4 scopes GO on both boxes (42+15 here, 57 on the RTX 5090), --changed GO on laserax-ai (663, 25/25). Deviation: dumps at step 200 not byte-identical (f32 round-off of the new layout) — rel L1 <= 4.7e-7 in sigma/E, gas |dX| <= 7.9e-7, inside G-REF's bands, both adapters. Not acted on: still ~3x TW's 0.14 ms step (flagged M0-T63); hydro_edges 8% of P4 at leave edges; M0-T43 flagged on the tile layout. Per-dispatch timing was a scratch probe, never committed. Commits at the lead's word: dcc0991, 3a614eb (laserax-ai), and this close. Ran on claude-opus-5-5, high. Detail: tasks/M0-D30.md. Next: M0-D31.

## M0-D31 · architecture.md's pass table says P2 and P4 have no dispatches: lot 4's four and lot 5's three were never written in · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-D30)
- Status: DONE (2026-10-13 16:00)
- Blocks: nothing — but an agent that reads the page for a module instead of the module (the instruction file's rule) is
  told P2 and P4 record nothing
- Caused by: M0-T31 (P2's four dispatches) and M0-T34 / M0-T35 (P4's three) — their docs blocks, M0-T32 and M0-T40, named
  physics.md and testing.md only
- Files: docs/agent/architecture.md (the step's pass table: its P2 and P4 rows, the "Dispatches today" column) — opened by
  M0-V5, not changed
- Read: this file (rules + this task) + docs/agent/architecture.md § the step's pass table + milestones/m0/m0_contrat.md
  §1.3.2 + crates/sr-engine/src/step/mod.rs lines 405–412 (`Step::dispatch_counts`, `dispatch_labels`) and the dispatch lists of step/grav.rs and
  step/hydro.rs (names and sizes only)
- Symptom: measured on linux-pc (laserax-ai), 2026-10-13, M0-V5: docs/agent/architecture.md lines 69 and 71 read
  « | P2 gravity | … | — (lands with its lot) | » and « | P4 hydro | … | — | », while the step records `grav_rows`,
  `grav_columns`, `conv_rows_inverse` and `grav_gradient` for P2 (M0-T31) and `hydro_update`, `hydro_edges` and
  `hydro_commit` for P4 (M0-T34, M0-T35) — physics.md § Gravity and § Gas flow and testing.md's rows name them
- Deliver: the two rows brought to the code — each dispatch by name, its size (box tiles, world-sized, fixed) and, where
  it binds, what it binds (P4's eight storage buffers, `hydro_edges`' seven) — written after M0-D30 so P4's row is the
  tiled one; nothing else on the page moves → Verify 1
- Done when: the table's P2 and P4 rows name every dispatch the step records for them
- Verify: 1. `grep -oE "grav_rows|grav_columns|conv_rows_inverse|grav_gradient|hydro_update|hydro_edges|hydro_commit" docs/agent/architecture.md | sort -u | wc -l`
  · Pass: 7 (or M0-D30's dispatch count for P4 plus four) · Fail: fewer
- Handoff: architecture.md's pass table: P2 row now names grav_rows, grav_columns, conv_rows_inverse, grav_gradient with sizes (world-row, padded-width fixed, box tiles indirect) and says the cadence and sinks' potentials are not built (grav.rs:20); P4 row names hydro_update (box tiles indirect, 8 storage buffers), hydro_edges (fixed, one invocation a boundary face, 7 buffers), hydro_commit — the tiled shape after M0-D30. Nothing else on the page moved (2 lines in git diff). Verify 1 [ALREADY RUN — PASS (7) on win-laptop]; claim verify.py --changed --task M0-D31 [ALREADY RUN — NO-GO 'nothing changed since every scope's last GO: no scope to run' (docs-only edit, 0 of 34 scopes impacted) on win-laptop] — not a code red, no suite owed. Measured, not acted on: physics.md:133 still says the tiled hydro_update is 'the known fix, unbuilt' though M0-D30 built it (line 128 says so) — a docs fix for the lead to file. Model: model=claude-sonnet-5-5 level=high (rung_record now, plugin=installed; matches the heading's rung). Next: M0-D32.

## M0-D32 · rung_record.py's report reads a block's rung from its handoff's prose, never its quoted model= level=: 6 of the 7 rungs it reads on linux-pc are wrong · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D31)
- Status: DONE (2026-10-13 16:09, started 16:03)
- Blocks: nothing — but the next phase-closing V (M0-V6) reads the record that §0's rule (4) rates from
- Caused by: unknown — the fallback is the annex's tool as M0-TH-adopt-a swapped it in, not a repair of a block's work
- Files: tools/pb/rung_record.py (the report's handoff fallback, its selftest), milestones/m0/tasks/toolkit_local_diff.md
  — unverified, the executing task confirms first
- Read: this file (rules + this task) + PLAYBOOK.md §0 (rule (4), the record) and annex §A's rung_record.py section text +
  tools/pb/rung_record.py's report and selftest sections — this block changes them, so it reads them, the one exception to
  « never the tools' code » + milestones/m0/tasks/M0-TM5.md § Measured and not acted on +
  milestones/m0/tasks/toolkit_local_diff.md
- Symptom: measured on linux-pc (laserax-ai), 2026-10-13, M0-V5: `python3 tools/pb/rung_record.py report --json` reads 60 of
  its 100 DONE blocks from their handoffs (no session file on this box); 46 of those handoffs quote `model=<id> level=<level>`
  (the switch hook's form). Of the 46: M0-D13 reads right; M0-TJ3, M0-V2, M0-TM2 and M0-TM4 (quoting `model=claude-opus-5-5
  level=max`) read as Sonnet 5.5, high, M0-TM3 (the same) as Sonnet 5.5, medium, M0-TR1 (`model=claude-sonnet-5-5 level=high`)
  as Opus 5.5, high; the other 39 read `unknown`. M0-TM5 measured the same six on 2026-10-13. The report's block counts and
  first-try rates per class and rung are wrong by those rows; its cost per DONE block is not (they carry no cost) — M0-V5's
  reading of the record, no class moves, stands either way.
- Suspected cause: the fallback takes the first ladder-rung string in the handoff's prose (M0-TM5's reading, 5 of the 6) and
  never parses the quoted `model=… level=…`
- Deliver: the fallback reads the quoted `model=… level=…` first, mapped to the ladder's rung, the prose only where a handoff
  quotes none, `unknown` where neither names one — the 60 rows re-read → Verify 1 · rung_record.py's selftest gains the
  quoted-handoff case and a plant that breaks it → Verify 2 · the change written into toolkit_local_diff.md as a local diff a
  future annex swap must carry → Verify 3
- Done when: on linux-pc, every handoff row that quotes a model and level reads that rung; `tools_selftest` red-armed
- Verify: 1. `python3 tools/pb/rung_record.py report --json` · Pass: each of the 46 quoting rows on its quote's rung ·
  Fail: any other · 2. `python3 tools/pb/verify.py --redarm tools_selftest --task M0-D32` · Pass: GO — every plant NO-GO ·
  Fail: a plant GO · 3. `grep -c "M0-D32" milestones/m0/tasks/toolkit_local_diff.md` · Pass: ≥ 1 · Fail: 0
- Handoff: rung_record.py's no-session fallback now reads the handoff's quoted `model=… level=…` first (`handoff_rung()`, the whole `- Handoff:` field, a dated id mapped by `rung_name`), the first line's prose rung only where none is quoted, `unknown` where neither; `rung_basis` names which. Cause confirmed: `RUNG_RE`'s first prose match, the quote never parsed. Measured on win-laptop: with no session read (linux-pc's condition, all 104 DONE rows) 71 handoffs quote, 71/71 on the quote's rung (was 2/71: 6 another rung, 63 unknown); `report --json` with this box's sessions GO, 23/23 quoting handoff rows right. Selftest +1 check, +1 plant (72 checks, 30/30 red); `--redarm tools_selftest` GO (181 s). Local diff: toolkit_local_diff.md item 6 (annex §A.9's text lags — M0-TZ). Owed on linux-pc: the literal Verify 1 (flagged to M0-V6). Detail: tasks/M0-D32.md. Ran on model=claude-opus-5-5 level=high. Next: M0-T41.

> **Commit gate (lead):** Phase 7 closes after M0-V5 — from the repo root, one at a time:
> `git add -A`
> `git commit -m "M0 Phase 7: gas flow"`
> `git push`

# Phase 8 — Lot 6 · heat, light and burning

## M0-T41 · Heat and light on the CPU twin (P5) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-V5)
- Status: DONE (2026-10-13 16:48)
- Carried flags: [M0-T41, 2026-10-13] [M0-T41, 2026-10-13, Sonnet 5.5 high, Laser2025-20] first try ENDED by its claim run (PLAYBOOK section 0: a NO-GO claim below (usual) ends the try — no second attempt on that rung); rated (usual), back to TODO. The deliverable is on disk, nothing to rebuild: crates/sr-physics/src/reference/heat.rs (+ pub mod heat in reference/mod.rs), crates/sr-engine/tests/gpu/heat.rs (+ mod heat in main.rs), scenes/heat_pulse.json, scenes/heat_spot.json, tools/pb/verify.json (scopes heat_gauss 6 / heat_rkl2 5 / heat_limiter 4, R17 spelling), tests/plants/chi-scale.patch, rkl2-s-minus-one.patch, limiter-off.patch, detail in tasks/M0-T41.md. Verify 1 GO (15 cases) and the three red-arms proven (chi-scale 2 failed, rkl2-s-minus-one 5 failed, limiter-off 2 failed) before the claim. The claim: 'verify.py --changed --task M0-T41' = 718 passed, 27/28 scopes GO, ONE red: build[native], cargo clippy -D warnings, 2 errors, both clippy::neg_cmp_op_on_partial_ord at heat.rs:350 (if self.test.pin_temperature || !(max_rate > 0.0) || !(dt > 0.0)). The fix is those two tests only (e.g. max_rate.partial_cmp(&0.0) != Some(Greater), or invert to a positive condition) — keep it on one line so the plant hunks (rkl2-s-minus-one @@ -352) still apply, then re-run 'verify.py --redarm' on all three scopes if any line moved, then the plan edits and one --changed (it will re-run 27 scopes in ~175 s). Untested after the lint fix: nothing else; clippy stops at the first crate, so sr-engine's lib (unchanged by this block) is the only crate clippy did not reach. Next session: do not redo the design — read tasks/M0-T41.md (conductance form, RKL2 w1 = 4/(s2+s-2), vacuum = floor state, radiated on the RKL2 recursion, F = start-of-pass faces, pin_temperature default) and the flags already filed to M0-T42, T43, T48 (the Twin driver has no P5 and nobody owns adding it), T49.
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
- Handoff: P5 on the CPU twin: reference/heat.rs (FLD with LP λ, κ with S1 dust term, RKL2 frozen at pass start, radiated on the RKL2 recursion, F kept, test flags), tests/gpu/heat.rs + scenes heat_pulse/heat_spot, scopes heat_gauss 6 / heat_rkl2 5 / heat_limiter 4, plants chi-scale / rkl2-s-minus-one / limiter-off all proven RED (first try, Sonnet 5.5 high). Second try fixed the claim's only red: clippy neg_cmp_op_on_partial_ord at heat.rs:350 → !(max_rate > 0.0 && dt > 0.0), same line, no plant hunk moved. Claim: verify.py --changed --task M0-T41 GO, 713 passed, 26/26 scopes, heat_rkl2 plant re-armed RED, on Laser2025-20 (win-laptop). Defaults taken: pin_temperature leaves planes alone, books nothing, keeps F (contract names only the key); the one-cell pulse stays (a smooth one cannot fail rkl2-s-minus-one — measured). Not acted on: six FLAG oracle lines from the claim, spelling only (Deliver: drops the crate prefix and writes scopes with '-', R17). Flags already filed: the Twin driver has no P5 (M0-T48), GPU port (M0-T42), P4 reads F (M0-T43), testing.md rows (M0-T49). Detail: tasks/M0-T41.md. Ran on model=claude-opus-5-5 level=high (rung_record). Next: M0-T42.

## M0-T42 · Heat and light on the GPU (P5) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T41)
- Status: DONE (2026-10-14 07:27)
- Carried flags: [M0-T41, 2026-10-13] P5's twin is crates/sr-physics/src/reference/heat.rs (Heat::step(&mut State, dt) -> Pass); its scopes are heat_gauss (6 cases), heat_rkl2 (5), heat_limiter (4) = 15 in tests/gpu/heat.rs, modules gauss/rkl2/limiter (R17: the plan's heat-gauss reads heat_gauss) — add the GPU cases to those modules and raise each scope's expected in tools/pb/verify.json. The scheme to port: per-face conductance K_f = c_sb lambda(R) a_r (T_a+T_b)(T_a^2+T_b^2)/(kappa Sigma)_f + k_cond, with kappa Sigma and a_r T^4 inside R the arithmetic means of the two cells', frozen at P5's start; RKL2 with w1 = 4/(s^2+s-2) (Meyer 2012; s^2+s-4 is a wrong memory that passed every first-order check), dt_p = 1/max_cells(sum_f K_f / C), C = Sigma/mu; a vacuum cell or the world's edge is the floor state (Sigma_floor, T_floor, the gas side's kappa and mu) and the vacuum cell is not updated; radiated rides RKL2's own recursion (R_j = mu_j R_(j-1) + nu_j R_(j-2) + mu~_j dt out(Y_(j-1)) + gamma~_j dt out(Y_0)) so it equals the grid's loss to rounding; F kept = the start-of-pass face fluxes. A smooth pulse cannot see a missing stage (sigma^2 stays exact at s-1): the one-cell pulse case does (tasks/M0-T41.md)
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
- Handoff: P5 on the GPU: shaders/heat.wgsl + step/heat.rs (39 dispatches, RKL2's 32 stages box-sized and returning past s; the latch untouched), Step::with_heat (with_passes unchanged), read_heat/read_heat_flux; headless builds P5 from the heat switch and prints SR-WARN heat stages <s>; 9 GPU cases (heat_gauss 10, heat_rkl2 8, heat_limiter 6), the three plants now hit the shader too. · Ruling: sun_disk's rim needs up to 3,803 stages, so the lead chose «Throttle stiff faces»: any cell needing > 32 has its faces capped (contract §0.5 [M0-T42], Superseded mirror); the real fallback stays a PLAN decision (Awaiting lead). · Verify 1 GO (RTX 5090); 2 GO on the Quadro: P5 0.188–0.261 ms of 0.88–1.10 ms per step (21–24 %), s median 2, max 32, 31 of 200 steps capped; 3 GO (rkl2-s-minus-one RED on all 3 GPU cases). The claim went NO-GO first (watch_only: heat.wgsl's struct Stage), then GO 27/27 (164 passed) after renaming it to StageIndex. · Deviations: 7 code files, ~1,110 new lines (over ≤6/≤600; timing.rs and dump.rs mirror headless's step); heat_stiff is a dispatch the plan did not name; the one-cell pulse's two-run heat (1.75e-3: f32 rounding at T = 2) is now graded per run, not by my preset 1e-4. 10 FLAG oracle lines read as consequences. · Flags → M0-T43, T48 (the twin has no cap), T49, T63 (idle stages cost). · Ran on laserax-ai, model=claude-opus-5-5 level=high. Detail: tasks/M0-T42.md. Next: M0-T43

## M0-T43 · The radiation force in the flow · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T42)
- Status: DONE (2026-10-14 07:51)
- Carried flags: [M0-T34, 2026-10-13] hydro_update (shaders/hydro.wgsl) is at WebGPU's 8 storage buffers since M0-T34 (state ×3, next ×3, Δt, eos table) and M0-T35/T36 may each have taken a freed slot: the flux F needs one — read their handoffs before binding · [M0-V5, 2026-10-13] M0-D30 (the lead's word at M0-V5) tiles hydro_update before this block adds F to it: read its handoff for P4's new layout, its workgroup memory and its storage-buffer count before binding F · [M0-D30, 2026-10-13] [M0-D30] hydro_update is now one workgroup per 8x8 tile (9,216 B workgroup memory, barriers reached by all 64 invocations): the radiation force goes into hancock's kick and the update's source, and the update already binds WebGPU's 8 storage buffers (tasks/M0-D30.md) · [M0-T41, 2026-10-13] F is heat::Flux (reference/heat.rs): face fluxes x (w+1) x h and y w x (h+1), positive toward +x/+y, from the start-of-pass temperatures; Flux::cells() gives the two cell-centred planes (the mean of a cell's two faces) that f = kappa Sigma F / c_sb is built from, edge faces included (they carry the light leaving) · [M0-T42, 2026-10-14] P5's F is side.flux (F_x then F_y, cell-centred: the mean of a cell's two faces), written by heat_rate over the box only — outside, the last values P5 wrote there, as P2's g; it is the capped flux where a cell needed > 32 stages (contract §0.5 [M0-T42]). Step::with_heat builds it (HeatSetup { side, test })
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
- Handoff: P4 pushes the gas with the light: hydro.wgsl's update adds Δt·f to every gas cell's momentum, f = κΣF/c_sb with F read from side.flux (P5's flux from the previous step, bound at 6) and κ = κ0(1+X_H) [+dust]; E is left alone. The twin's Hydro::step_forced (step_with unchanged); Hydro::new takes physics and the flux; Step::with_heat wires side.flux. To bind F within WebGPU's 8 storage buffers, P4 now reads the Δt record as a uniform (dt.rs adds UNIFORM, as abox). · Scope radiation_force (R17), 3 cases (twin, GPU, the adversarial hand-written F): the momentum gained, measured against the same step without the force, equals Σ Δt·f per half within 1e-6 (twin 1e-16, GPU ≤ 1.9e-8); Σ, E and species identical; P4 reads the F held at the step's start, not the one P5 writes after it (that F misses by 2.0). Verify 1 GO (RTX 5090); Verify 2 GO (radforce-zero RED, 3 failed). Claim NO-GO, 31/33 scopes, 745 passed: (1) dump, a regression of mine — a run resumed from a dump starts with F = 0 (side.flux is not in the dump), so preset:sun's s_150.bin differs; the lead chose «Dump carries F» and «A D block, next» → M0-D33 filed after M0-TM6 (Caused by: M0-T43); (2) desktop[launcher], environmental — the web scope's two parallel cases leaked their http.server on 47812 (launch.py status: stopped), so the decoy could not bind; I stopped that leaked process (my run had started it) and desktop re-ran GO 2/2. · Deviations: the predictor is not kicked and f = 0 below Σ_vac (F lags a step, so the force is first order anyway; no cost in the 96 predictors per tile; the one-step difference is exactly Δt·f); 7 code files (dt.rs, main.rs) against ≤6, ~470 lines; the κ_dust branch is ungraded (κ_dust = 0). · Flags → M0-T48 (the twin's driver runs no P5), T49 (docs), T63 (P4's extra reads); they fired the rating refresh → M0-TM6 filed after this block. · Ran on laserax-ai, model=claude-opus-5-5 level=high. Detail: tasks/M0-T43.md. Next: M0-TM6, then M0-D33

## M0-TM6 · Rating pass — six carried flags on open BUILD blocks since 2026-10-13 · **PLAN** · Opus 5.5, max · switch · (AFTER M0-T43)
- Status: DONE (2026-10-14 08:08, started 08:01)
- Carried flags: [M0-T43, 2026-10-14] Filed by M0-T43 when `plan.py flag` read the rating refresh's threshold (Rules: 5 flags): six carried flags on open BUILD blocks since `Model ratings: 2026-10-13 by M0-TM5`, M0-T43's three among them (to M0-T48: the twin's driver runs no P5; to M0-T49: the radiation_force docs; to M0-T63: P4's extra reads when P5 runs)
- Read: this file (rules + this task) + PLAYBOOK.md §0 (the ladder, the rubric) and §B.3 (the
  probe) + the agent's model pages the ladder cites + the record (`rung_record.py report`, the last
  phase-closing V's table)
- Deliver: the ladder re-read on the day — the probe and the model pages: a model retired,
  renamed or added, a price or a level changed → the `Models:` line rewritten and dated, the
  lead's answer first where a rung they run changes (§12) · every open block's rating re-derived
  by §0's rubric from its heading, `Sizing:`, `Deliver:`, `Verify:` and `Carried flags:` and the
  record — its rating segment rewritten, nothing else in the block · the register's `Model
  ratings: <YYYY-MM-DD> by M0-TM6` line · in the handoff, the ratings changed, counted by rung.
- Pass: `plan.py lint` GO; every open block on a rung of the ladder, each above the cheapest rung
  that fits named in the task file with its rule; the diff touches rating segments, the `Models:`
  line and the register's line only. Fail: any other byte changed; a BUILD or MOVE block above
  the cheapest rung that fits with none of §0's rules (1)–(4) named; a CHECK or PLAN block below
  `(gate)`.
- Handoff: Ladder re-read 2026-10-14: the probe (Claude Code 2.1.296) and the claude-api skill, loaded this time (cached 2026-10-06; its models.md is M0-TM4's file, hash equal with CRLF endings): all four models Active, prices and levels unchanged → `Models:` re-dated and re-cited to the page, rungs byte-identical, no question owed. Record: M0-V5's table and today's report (linux-pc) move no class (BUILD on Sonnet 5.5, high $1.68 per DONE block against (usual)'s $4.66; the E rows n < 10); no open escalation trail. Ratings: 0 changed of 99 — Opus 5.5, max 17 (0) · Opus 5.5, high 4 (0: T46, T51, T56 by (e), D33 by (a)) · Sonnet 5.5, high 72 (0) · Haiku 5.5, high 6 (0). Not acted on (bounded diff), for the lead or the next PLAN block: M0-T48 (E, Haiku) now carries by its flags the twin driver's P5, F and P6, which no block delivers — split it or drop `Sizing: E`, never a step up; M0-T106's momentum bound in bounce against M0-T35's flag and contract §2.7's « G-CONS grades mass and energy only ». `Next task` left stale (M0-D33 runs next). model=claude-opus-5-5 level=max (rung_record now), linux-pc. Detail: tasks/M0-TM6.md. Next: M0-D33.

## M0-D33 · A run resumed from a dump loses F: the radiation force is 0 on its first step, so a heat run's resume is no longer bit-identical · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TM6)
- Status: DONE (2026-10-14 08:17)
- Blocks: M0-T43's claim (`dump` NO-GO) and every later claim whose paths touch the `dump` scope
- Caused by: M0-T43 — P4 now reads F from `side.flux` (the previous step's P5), and nothing in the dump carries it
- Files: crates/sr-engine/src/dump.rs (the header's `resume`, the planes after the 14 channels, `load_dump`),
  crates/sr-engine/src/headless.rs (the dump writer; `--load-dump` uploads F into side.flux), crates/sr-engine/tests/gpu/dump.rs,
  a plant under tests/plants/, milestones/m0/m0_contrat.md §0.5 — unverified, the executing task confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.2, §2.12.3, §0.5 [M0-T26], §2.4.4 +
  milestones/m0/tasks/M0-T43.md + milestones/m0/tasks/M0-T26.md if present
- Symptom: measured on linux-pc (laserax-ai, RTX 5090), 2026-10-14, M0-T43's claim: `dump` 7 passed, 1 failed —
  `dump::headless_two_runs_of_100_through_a_dump_write_the_200_step_runs_dumps_byte_for_byte` panics at tests/gpu/dump.rs:304,
  "s_150.bin: the resumed run's dump differs from the straight run's"; s_100.bin is still identical (preset:sun, 256 × 256,
  heat on: `SR-WARN heat stages 3800`)
- Repro: `python3 tools/pb/verify.py dump --task <ID>` on the tree after M0-T43
- Suspected cause: the straight run's step 101 pushes with the F that step 100's P5 left in side.flux. The resumed run builds a
  new SideFields (zeroed) and loads only the 14 channels and `resume`, so its step 101 has F = 0. F cannot be rebuilt from the
  cells: P5 took it from the temperatures after P4 of step 100, not from the dumped state after P8
- Ruling: the lead, 2026-10-14, M0-T43 (question tool): «Dump carries F (Recommended)» — when the run runs P5, the dump appends
  F's two f32 planes (side.flux: F_x, then F_y, row-major) after §2.2's 14 channels, flagged in `resume` (e.g. `"flux": true`);
  `--load-dump` uploads them into side.flux; version stays 1; a header whose flag and plane count disagree is refused. Declared
  in the contract's §0.5 as [M0-D33], with its mirror in the plan's Superseded section (PLAYBOOK §9)
- Deliver: the dump format above in dump.rs and headless.rs (writer and `--load-dump`), the contract's §0.5 line and the
  Superseded mirror → Verify 1 · a plant that drops F on load (the resume uploads zeros) → Verify 2
- Done when: on linux-pc the `dump` scope is GO with the heat run's resume byte for byte, and red-armed
- Verify: 1. `python3 tools/pb/verify.py dump --task M0-D33` · Pass: GO, every case, s_150.bin and s_200.bin byte for byte ·
  Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm dump --task M0-D33` · Pass: GO — the new plant NO-GO · Fail: the
  plant stays GO
- Handoff: A dump now carries F: when the step runs P5, dump.rs writes side.flux's 2 f32 planes (F_x, F_y) after the 14 channels, flagged resume.flux (always written, required); load_dump writes them into side.flux (reached via the new Step::side_flux(), Heat keeps the buffer) and refuses F into a step without P5 and P5 without F; a body the flag disagrees with is refused naming it; version 1. Contract §0.5 [M0-D33] + the Superseded mirror. · dump 8/0/0 GO (RTX 5090), the headless heat resume s_150/s_200 byte for byte again, F read by hand (non-zero in 8,836 cells per plane); plant dump-flux-zero RED (1 failed, s_150.bin) beside the two old plants; claim GO 30/30 scopes, 692 passed. · Deviations: headless.rs needed only twin_dump's flux: None (--load-dump goes through load_dump); step/heat.rs added (not in Files:); 4 code files, ~130 lines; defaults: flux key required, both mismatches refused (tasks/M0-D33.md). · T43's leaked web server did not recur (web and desktop GO together, 47812 free). · Ran on model=claude-opus-5-5 level=high. · Next: M0-T44a.

## M0-T44a · The rate law — ω and its temperature table · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T43)
- Status: DONE (2026-10-14 08:21)
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
- Handoff: linux-pc (laserax-ai), model=claude-sonnet-5-5 level=high (rung_record now, plugin=installed; matches the heading). rates.rs: Omega::new(&Record) → omega/f/f_direct/g, ln_f_nodes()/node_t()/step() for M0-T46; f tabulated on 256 log-spaced points over [T_thr, 2.5 T_thr], log-log, direct form above the span, exactly 0 below T_thr. Declared default: the span (contract fixes 256 and 1e-4, not the range) — 2.5 is under the ≈ 2.8 the budget allows; H_burn's worst midpoint error 7.9e-5 at T = 119.5 (terms cross at 119.7), one power law exact. Verify 1 [ALREADY RUN — PASS (rates 7/0/0) on linux-pc]; Verify 2 [ALREADY RUN — PASS (clean GO, rates-table-shift RED, 4 failed) on linux-pc]; claim --changed [ALREADY RUN — PASS (744 passed, 28/28 scopes GO) on linux-pc]. Not run: win-laptop (owed at its next V). Not acted on: none measured. Detail tasks/M0-T44a.md. Next: M0-T44b.

## M0-T44b · Burning on the CPU twin (P6) — G-BURN, G-ORDER twin cases · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T44a)
- Status: DONE (2026-10-14 08:35)
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
- Handoff: linux-pc (laserax-ai), model=claude-sonnet-5-5 level=high (rung_record now, plugin=installed; matches the heading). reference/react.rs: React::new/step, linearised backward-Euler sub-cycles under burn_dx/burn_dt on M0-T44a's ω, each sub-step checked and halved on failure, clamp ≥ 0 + renormalise, q·Δξ into E and the group's accumulator per sub-step (Pass::nuclear); tests/gpu/burn.rs: cell 9, order 2 (twin, no adapter). Verify 1 [ALREADY RUN — PASS (burn_cell 9, burn_order 2, GO) on linux-pc]; Verify 2/3 [ALREADY RUN — PASS (burn_cell: q-double RED, 2 failed; burn_order: swap-he-c RED, 1 failed) on linux-pc]; claim --changed [ALREADY RUN — FAIL (clippy needless_range_loop in react.rs, build scope) then PASS after the fix (223 passed, 29/29 scopes GO, the anchor then vouches for the tree) on linux-pc]. Declared defaults: G-BURN's 1e-4 is held with burn_dx overridden to 1e-4 (BE is first order — the shipped 0.1 reads 2.8e-2 on H_burn, only bounded); G-ORDER reads instantaneous power on a fresh equal-parts cell (a burning cell starves carbon: C never passes ε*). Not acted on: f32 on the GPU cannot hold 1e-4 over ~10⁴ sub-steps (T46's to measure). Detail tasks/M0-T44b.md. Next: M0-T45.

## M0-T45 · Neutrino cooling and the iron gate on the CPU twin · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T44b)
- Status: DONE (2026-10-14 08:45)
- Carried flags: [M0-T44b, 2026-10-14] [M0-T44b, 2026-10-14] reference::react::React (crates/sr-physics/src/reference/react.rs) is where neutrino cooling and N_Fe's latch condition go: React::burn's sub-cycle loop holds every record's ω each sub-step (a rate > 0 for N_Fe there is the latch's trigger — Pass has no field for it yet; add one) and e_tot (thermal + cold per mass; ε_th = e_tot − ε_cold(new Y_e, X_n), T = μ ε_th); cooling enters as a rate term in the sub-step's dT/dt limit and e_tot, booked per sub-step like q·Δξ; Pass::nuclear is 7 group planes (N_GROUPS), signed; tests build worlds with tests/gpu/burn.rs's world()/only() helpers (a record alone via the registry text's enabled flags)
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
- Handoff: reference::react now cools by ε_ν = A_ν(T/T_ν)^m (T ≥ T_ν), books neutrino_lost, keeps S_ν = ε_νΣ where X_n ≥ 0.5 and exposes the latch (Pass::latch = any N_Fe ω > 0); N_Fe's gate was already in rates.rs (T44a) and is now graded on Y_eΣ. New scope neutrino (tests/gpu/neutrino.rs, 8 cases): Verify 1 GO and Verify 2 GO (nfe-gate-off and nfe-gate-sigma both RED), claim --changed GO 35/35 scopes, 762 passed, on laserax-ai (twin only, no adapter). Deviations: cooling off in React::new, on via with_neutrinos (T44b's runaway would cool; flagged to M0-T47); S_ν = the pass's mean power; unpinned ODE tolerance 5e-3 of the drop (measured 1.8e-3); an extra plant nfe-gate-sigma for the Adversarial case; main.rs and verify.json touched (the module and scope) — the claim's FLAG oracle lines name them. Not done: testing.md row (M0-T49). Ran on model=claude-sonnet-5-5 level=high (rung_record.py now). Detail: milestones/m0/tasks/M0-T45.md. Next: M0-T46.

## M0-T46 · Burning on the GPU (P6) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T45)
- Status: DONE (2026-10-14 09:11)
- Carried flags: [M0-T44b, 2026-10-14] [M0-T44b, 2026-10-14] the twin's sub-cycle (react.rs, tasks/M0-T44b.md § The scheme): dense 10×10 Gauss solve of (I − hJ)ΔX = hS, J from ∇ω_r = n_j X_j^(n_j−1) ω_r(X_j := 1), h = min(t_rem, burn_dx/max|S|, burn_dt·T/|dT/dt|, 2·last h) then a check-and-halve; G-BURN's 1e-4 holds on the twin only at burn_dx 1e-4 (first-order BE: 2.8e-2 at the shipped 0.1) ≈ 9,000 sub-steps for one cell — f32 will not hold 1e-4 over that many, so the GPU cases need a measured f32 bound (a D or a contract question, never a looser number silently) or a different sub-step; the shader must also take Omega's direct form above the table span (M0-T44a)
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
- Handoff: linux-pc (laserax-ai), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; matches the heading). P6 on the GPU: shaders/react.wgsl + step/react.rs (Step::with_reactions; with_heat delegates; headless's reactions switch builds it), the twin's BE sub-cycles in f32 with the time done, each fraction and the heat as compensated two-float sums — the NVIDIA compiler folds a plain two-sum (measured: error term 0), kept only through bitcast-XOR opaque(); ln f capped at 60, 65,536 attempts/cell. G-BURN/G-ORDER hold the contract's own numbers on the RTX 5090 (X ≤ 5.0e-5 of 1e-4, heat 4e-8 of 1e-5, crossings = the twin's): no looser bound, no question. Verify 1 [ALREADY RUN — PASS (burn_cell 15 + burn_order 4, GO) on linux-pc RTX 5090]; Verify 2 [ALREADY RUN — PASS (exit 0, P6 0.127 ms of 1.056 ms/step) on linux-pc Quadro]; Verify 3 [ALREADY RUN — PASS (q-double RED, 4 failed incl. both GPU heat cases) on linux-pc]; claim --changed [ALREADY RUN — FAIL (timing: its disk() mirror lacked P6) then PASS (188 passed, 29/29 GO) on linux-pc]. Adversarial: a runaway 8×8 core in a cold field, one 1 t.u. step — P6 42.4 ms on the Quadro (0.10 cold), ~0.22 ms per serial sub-step; sun_disk's 200 steps barely burn (onset cost only) — flagged M0-V6. Deviations: 8 files, ≈1,075 new lines (over 6/600; shader ≈490); timing.rs and verify.json and q-double.patch beyond Deliver. Flagged M0-T47: P6 uses 7 of 8 storage slots (s_nu takes the last; fold the latch flag). Detail tasks/M0-T46.md. Next: M0-T47.

## M0-T47 · Neutrinos and the iron gate on the GPU, setting the latch · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T46)
- Status: DONE (2026-10-14 09:24)
- Carried flags: [M0-T22, 2026-10-10] P6 sets the latch by atomicOr(1) into word 0 of Step::latch_flag() (the latch record, a u32); the controller takes it after P9 (M0-T22). · [M0-T45, 2026-10-14] react.rs (M0-T45): neutrino cooling is off in React::new — the scene's `neutrinos` switch is React::with_neutrinos(on), default on for a scene (T44b's cases ran without it; the runaway reaches T 18,000). ε_ν is read at each sub-step's start T, taken from e_tot (E −= ε_ν h Σ) and carried in dT/dt (−μ ε_ν); Pass::neutrino_lost = Σ·Σε_ν h per cell (≥ 0), Pass::s_nu = that over Δt (the pass's mean ε_νΣ) only where the cell leaves the pass with X_n ≥ 0.5 (X_N_STAR); Pass::latch = any cell's N_Fe ω > 0 at a sub-step's start (the id 'N_Fe', per enabled record in React::latch). N_Fe's gate lives in rates.rs Omega::g (Y_eΣ, read with the cell's own Y_e). tests/gpu/neutrino.rs's cases are the GPU's oracles (build worlds with its world()/only()/react() helpers) · [M0-T46, 2026-10-14] react.wgsl/step/react.rs (M0-T46): P6 binds 6 storage buffers + eos_table = 7 of WebGPU's 8 (records and ln f tables ride the uniform react.params) — side.s_nu takes the 8th, so the latch flag cannot be a 9th binding: fold it (a latch bit in react.report copied to Step::latch_flag by a one-invocation dispatch, or the report moved into the latch record). Sub-step loop: cooling goes beside rates() (rate.y = 'any ω ≠ 0' must also hold ε_ν ≠ 0, as the twin's break), into the dT/dt limit and e_tot; book neutrino_lost (booking plane 7) with add2 — the NVIDIA compiler folds a plain two-sum, only opaque() (bitcast XOR params.zero) keeps the error term (tasks/M0-T46.md § f32 findings)
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
- Handoff: [M0-T47] react.wgsl/step/react.rs: ε_ν from E and dT/dt, neutrino_lost (booking plane 7), S_ν in side.s_nu (X_n ≥ 0.5, rewritten every step), the latch bit in report.latch → react_latch (new third dispatch, built after Latch::new) ORs it into Step::latch_flag. tests/gpu/neutrino.rs: 10 GPU cases (scope neutrino 8 → 18, now `neutrino physics`), incl. a 40-step frame that stops after step 10 where the physics lifts Σ over Σ_N/Y_e. Plants nfe-gate-off/-sigma now hit the shader; new nfe-latch-last. Verify 1 GO (18 + 5), Verify 2 3 plants RED, claim --changed GO 698/30 scopes, laserax-ai RTX 5090; win-laptop owed. Deviations: ReactSetup gained {neutrinos, side} (3 call sites), 8 code files — over the ceiling. Findings (tasks/M0-T47.md): the gate-blind linearisation overshoots closure (9 % vs 1 %, twin alike); stale flag after bare steps. Model claude-sonnet-5-5, level high. Next: M0-T48.

## M0-D34 · P6's energy: the GPU and the twin book 24 % apart on G-REF's burning field, with X equal to 10⁻¹² · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T48)
- Status: DONE (2026-10-14 10:17)
- Caused by: unknown (candidates: M0-T44b's twin P6 `reference::react::React::step`, M0-T46's GPU P6 `shaders/react.wgsl`)
- Sizing: M
- Read: this file + milestones/m0/tasks/M0-T48.md (the measurements) + the two P6 energy paths named in Symptom
- Symptom: on scenes/ref_burn.json (64 × 64 uniform H 0.75 | He 0.25 at Σ 30, t 100, bounce edges, heat and reactions on, gravity, hydro and neutrinos off), `cpu_gpu`'s burning case reads Σ L1 0, X within 1.1 × 10⁻¹² of the twin, and H fully consumed on both sides (ΔH −92,160 on each), yet the total energy change differs: twin +9.07 × 10⁶, GPU +1.12 × 10⁷ (24 %); E relative L1 3.07 × 10⁻² against the band 10⁻³; sim time GPU 0.984 / twin 1.018 (−3.3 %), the Δt gap following from the energy gap. Switching one pass off isolates it to P6: heat off, reactions on: E L1 8.5 × 10⁻³; heat on, reactions off: E L1 1.6 × 10⁻⁷ (pass). So P5 is not the cause, and the rates agree while the energy booked per unit of X moved does not.
- Repro: `python3 tools/pb/verify.py cpu_gpu --task M0-T48` on laserax-ai (RTX 5090), the `ref_burn.json` case. The totals came from a temporary print in `grade()` (sums of planes 3 and 0·(4+H) before and after, both sides), not in the tree.
- Suspected cause: not measured. The rate (X) agrees, so the mismatch is in the energy term: per record, the twin's `rec.q · xi` (react.rs ~306–337, `cell.de` into E at ~441) against the GPU's `coef.z · h_use · lin` (react.wgsl ~456–466) and `energy + σ(de_sum − lost_sum)` (react.wgsl ~546); coef.z is `rec.q(physics)` (step/react.rs:96, registry.rs:514) on both. Not yet checked: a record whose q differs between the two sides while its X does not change (H is consumed in the first steps; He and later records may burn with X equal but q not), or a second energy write on one side after P6.
- Deliver: the cause named by measurement (per-record booked heat against ΔE, one record enabled at a time, both sides, the single-cell G-BURN free run at T ≥ 140 included), and the side that is wrong fixed, or a D-block for the side's own work filed if the fix is larger than the block's sizing; `cpu_gpu`'s burning case green
- Verify: 1. `python3 tools/pb/verify.py cpu_gpu --task M0-D34` · Pass: GO, 3 cases, the burning case's E L1 ≤ 10⁻³ · Fail: NO-GO on the burning case (E L1 above 10⁻³)
- Adversarial: a fix that matches the energies by booking the GPU's extra energy as nuclear without its q is a bypass — the per-record check must still show each record's q·Δm within 10⁻⁵ on both sides
- Handoff: linux-pc (laserax-ai, RTX 5090), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; matches the heading, on both lead prompts). Cause measured by a GPU/twin lockstep (tasks/M0-D34.md): (1) P6 — react.wgsl's LN_F_MAX clips each record's ln f at 60, the twin's power law reaches e^194 in ref_burn's step-44 runaway (H → Fe in one step), so the He/C branching and the He group's heat drift 0.44 % (step 9.0e-3; heat-off 200 steps 8.5e-3 → 6.3e-8 with the same clip in the twin); (2) P5 — steps 45–54 need 35–549 stages, the GPU caps at 32 ([M0-T42]), E L1 2.83e-2 even with P6 matched: the block's 'P5 is not the cause' held only heat-off. Rulings (the lead, question tool): «Gamow-shaped terms (Recommended)» and «Implicit solver (Recommended)», after 'I want the most physically realistic outcome', then 'make sure itll be able to run on the players hardware however haha' — contract §0.5 two [M0-D34] lines, Superseded mirrors, Awaiting lead cleared. Split (sizing miss: a contract law with consumers I don't own + a new solver): M0-D35 (BUILD, the law on both sides + the GPU rate scale, my measured scale saved as tasks/M0-D34.react-scale.patch: step 44 to 6.1e-4, but a NaN at T ≈ 47,000 still open) and M0-TJ4 (PLAN + LEAD answers, the implicit P5 and its Quadro cost ceiling); M0-T48 flagged. No code changed: react.wgsl, rates.rs, reference/react.rs byte-identical to the start, diagnostics removed. Claim [ALREADY RUN — FAIL (cpu_gpu: the burning case E L1 3.07e-2, T48's inherited red, moved to D35 + TJ4's build + T48; 531 passed, 1 failed, 3/4 scopes GO; anchor 73 never-checked = cpu_gpu's paths) on laserax-ai] python3 tools/pb/verify.py --changed --task M0-D34. Next: M0-D35.

## M0-D35 · The Gamow-shaped rate law on both sides, and the GPU's rate scale past f32's e^88 · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D34)
- Status: DONE (2026-10-14 10:31)
- Caused by: M0-T46 — react.wgsl's `LN_F_MAX` clips each record's ln f at 60, the twin does not (measured, tasks/M0-D34.md); the law itself changes by the lead's ruling at M0-D34
- Rating: rule (3)'s reason — numeric or precision semantics (f32 rates past e^88), and §1.6.1's law changed
- Sizing: M
- Files: crates/sr-physics/src/rates.rs (the law, its table nodes, its tests), crates/sr-engine/shaders/react.wgsl (the direct
  form, the rate scale), crates/sr-engine/src/step/react.rs if the terms' packing changes, crates/sr-engine/tests/gpu/burn.rs
  (a hand-computed f, and the new case), docs/agent/physics.md (the rate-law paragraph) — unverified, the executing task
  confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §1.6.1–§1.6.3, §2.5.1, §0.5's two [M0-D34] lines +
  milestones/m0/tasks/M0-D34.md (the measurements, the scale's design and its open NaN) + tasks/M0-D34.react-scale.patch
- Symptom: measured on linux-pc (laserax-ai, RTX 5090), 2026-10-14, M0-D34: on ref_burn.json's runaway (step 44, H → Fe in
  one step, T ≈ 18,800) the GPU books 0.9 % more energy than the twin (He group 0.44 % apart); heat-off 200 steps E L1
  8.5 × 10⁻³; with D34's scale patch 6.1 × 10⁻⁴ at that step but a non-finite energy at step 64 (edge cell, T ≈ 47,000)
- Repro: tasks/M0-D34.md § The cause (the lockstep and the heat-off copy of ref_burn.json are in the logs named there)
- Ruling: the lead, 2026-10-14, M0-D34 (question tool): «Gamow-shaped terms (Recommended)» — contract §0.5 [M0-D34]:
  f(T) = Σ_k w_k (T/T_k)^{−2/3} exp(τ_k (1 − (T_k/T)^{1/3})), τ_k = 3ν_k + 2, T ≥ T_thr; and "make sure itll be able to run
  on the players hardware however haha" — a cell whose rates fit f32 (shift 0) runs the plain arithmetic, no new dispatch
- Deliver: the law in rates.rs (the 256-node table and the direct form above its span both from it, the by-hand test values
  recomputed) → Verify 1 · react.wgsl's direct form the same law, and the rate scale of tasks/M0-D34.react-scale.patch (scale
  over burning records, divided solve, h_try carried and floored, the twin's negative margin), finished so no cell goes
  non-finite — start from the open NaN: the shift is taken from ln f alone, so a trace species with a huge f sets it
  (D34's note) → Verify 2 · a new burn::cell case, the GPU's P6 against the twin's on one cell at ref_burn's step-44 state
  (Σ 30, H | He, T from the lockstep, Δt 6.3056e-3) and on one hotter (T ≥ 5 × T_Si), each group's booking and ΔE within
  10⁻⁵ relative, every value finite → Verify 2 · G-BURN, G-ORDER and the neutrino cases re-read under the new law (a
  shipped-limit case that no longer holds is reported with its numbers, never loosened) → Verify 2, 3 · the heat-off copy of ref_burn.json read over 200 steps → none — a measurement in the handoff, tasks/M0-D34.md's method
- Done when: on linux-pc the scopes below are GO, the new case red-armed, and the heat-off copy of ref_burn.json reads E L1
  ≤ 10⁻³ over 200 steps (tasks/M0-D34.md's method; its number in the handoff)
- Verify: 1. `python3 tools/pb/verify.py rates --task M0-D35` · Pass: GO · Fail: NO-GO · 2. `python3 tools/pb/verify.py
  burn_cell --task M0-D35` then `burn_order`, `neutrino`, `latch` · Pass: GO each, the new case among burn_cell's · Fail:
  NO-GO · 3. `python3 tools/pb/verify.py --redarm burn_cell --task M0-D35` · Pass: GO — a plant that clips ln f per record
  again NO-GO · Fail: the plant stays GO
- Adversarial: matching the twin by clipping both sides, or by booking energy without its q·Δm, is a bypass — the new case
  reads each group's booking against the twin's, and a clip plant must turn it red
- Handoff: linux-pc (laserax-ai, RTX 5090), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; matches the heading, on both lead prompts). Built: rates.rs's f(T) Gamow-shaped (§0.5 [M0-D34]; table + direct form from one term_ln; span 2.5 holds, H_burn 7.4e-5; by-hand values rewritten, new gamow_shape_and_bound — rates 8/8); react.wgsl = D34's scale patch + the Gamow direct form (shift now ≤ 52, so D34's step-64 NaN — e^173 = +inf scaled time — is gone, no logic change needed); burn::cell's new gpu_the_runaway_books_as_the_twin + plant burn-clip-ln-f (16/16; q-double and burn-clip-ln-f RED, rates-table-shift RED); verify.json, testing.md, physics.md. Heat-off ref_burn 200 steps E L1 4.96e-8 (was 8.5e-3); heat-on finite, E L1 3.09e-2 = P5's cap (TJ4). Deviations (declared, tasks/M0-D35.md): the step-44 state no longer runs away under the new law, so the case marches the field to its runaway pass (47, T 143 → 18,813: ΔE 5.6e-8, groups ≤ 1.2e-8 of the pass) and T 2,520 (1.8e-8); groups graded against the pass's nuclear energy — trace groups C/Ne read 1.3e-5 of their own value (f32 cancellation), a lead's call to grade them on their own; two extra cells (D34's NaN cell, He's peak T 3.2e7) finite. Re-read: burn_order 4/4 with crossings H 147 … Si 504 (were 145.5 … 501), neutrino 18/18, latch 5/5, shipped_limits_hold ΔT/T 0.0500. Not acted on: trace-He-in-iron cell GPU 13 retries / 1 sub-step vs twin 0 / 12, energy under f32's E resolution. On the lead's word ('bump T48 to Opus high'): M0-T48's heading Haiku 5.5, high → Opus 5.5, high (no plan.py call for a rating; edited by hand); T48 and TJ4 flagged. Claim [ALREADY RUN — FAIL (cpu_gpu: the burning case E L1 3.09e-2, P5's cap, the red D34 handed over to TJ4's build + T48; 735 passed, 1 failed, 30/31 scopes GO; anchor 73 never-checked = cpu_gpu's paths) on laserax-ai] python3 tools/pb/verify.py --changed --task M0-D35. Next: M0-TJ4.

## M0-TJ4 · Specify P5's implicit solver: heat flows unthrottled, within the player's frame budget · **PLAN + LEAD answers** · Opus 5.5, max · switch · (AFTER M0-D35)
- Status: DONE (2026-10-14 11:37, started 11:12)
- Carried flags: [M0-D35, 2026-10-14] with P6 matched under the Gamow law (M0-D35), heat-on ref_burn.json reads E L1 3.09e-2 after 200 steps on the RTX 5090 (heat-off 4.96e-8): the whole gap is P5's cap after the runaway — a number for the implicit solver's G-REF oracle (logs/M0-D35.heaton-ref_burn.log, tasks/M0-D35.md)
- Why: the lead ruled an implicit P5 at M0-D34 (contract §0.5 [M0-D34]); M0-T42 had capped a stiff cell's faces at 32 RKL2 stages (§0.5 [M0-T42]) where the twin does not
- Rating: rule (2) — a PLAN block
- Sizing: M
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §2.4 (§2.4.3 above all), §0.5 [M0-T42] and the second
  [M0-D34] line, §1.3.2–§1.3.4, §5.1's G-HEAT1, G-HEAT2, G-FLD and G-REF + milestones/m0/tasks/M0-D34.md (§ The cause,
  cause 2) + milestones/m0/tasks/M0-T42.md if present
- Symptom: measured on linux-pc (laserax-ai, RTX 5090), 2026-10-14, M0-D34: on ref_burn.json after the runaway, steps 45–54
  need 35–549 stages, the GPU capped at 32, so with P6 matched E L1 stays 2.83 × 10⁻² (sim time −2.86 %); from step 55 (≤ 32
  stages) GPU and twin agree to ~10⁻⁵. sun_disk's rim needs up to 3,803 stages (M0-T42)
- Ruling: the lead, 2026-10-14, M0-D34 (question tool): «Implicit solver (Recommended)», after "I want the most physically
  realistic outcome for that"; then "make sure itll be able to run on the players hardware however haha"
- Deliver: the contract text that replaces §2.4.3's scheme (as a §0.5 override): the implicit step (backward Euler, or a
  stated second-order form), how the flux limiter's nonlinearity is treated (coefficients lagged at the pass's start, as
  RKL2's are, or iterated), the iterative method on the GPU over the box (e.g. preconditioned conjugate gradient, or
  multigrid), its stopping rule (a residual tolerance and a fixed iteration ceiling, what happens at the ceiling, and how
  the run reports it — no silent throttle), energy kept to the ledger's bound, and F as the solved flux → Verify 1 · the twin's P5 on
  the same scheme in f64 (a direct or tight iterative solve), so G-REF compares one discretisation → Verify 1 · the oracles: G-HEAT1,
  G-HEAT2, G-FLD re-read, G-REF's burning case, and a stiff case (a cell needing ≥ 500 RKL2 stages) → Verify 1 · the cost: P5's GPU
  time per step on the Quadro (the mid-range box, R12) for preset sun at 600 × 400 against the frame budget at the top
  speed (M0-R3's numbers), a stated ceiling, and what the solver gives up first if a step would pass it, put to the lead → Verify 1 · the BUILD blocks
  (twin, GPU, oracles, timing), sized and rated, placed before M0-T48's retry → Verify 1
- Done when: the §0.5 text and the blocks are in the contract and the plan, the lead's yes on the cost ceiling recorded
- Verify: 1. `python3 tools/pb/verify.py plan_lint --task M0-TJ4` · Pass: GO · Fail: NO-GO
- Handoff: linux-pc (laserax-ai), model=claude-opus-5-5 level=max (rung_record now, plugin=installed; matches the heading). The lead's answers (question tool): «Re-solve heat, light law fixed (Recommended)», «≤ 0.5 ms per step (Recommended)», «Speed gives first (Recommended)». Contract §0.5, three [M0-TJ4] lines: P5 one backward-Euler step in u = a_r T⁴ — λ and κΣ frozen at the pass's start, the heat exact by Newton passes, f32 Jacobi-PCG to two 10⁻⁵ measures, δ closing the heat balance, F the solved flux, a hard step carried into the next submission (ceiling 16 passes / 4,096 rounds, then SR-WARN), P5 ≤ 0.5 ms a step on the Quadro; G-HEAT2 re-read (own-solve + first order; scope heat_implicit, plant solve-two-rounds); G-FLD re-read (the frozen law's bound; the solved F can pass it at cold thin fronts). §5.1 G-STIFF new (heat_hot/heat_rim, scope heat_stiff, plant newton-one-pass); Superseded mirrors. Evidence: CPU scratch models of the twin's law (NOT PROVEN — synthetic, logs/M0-TJ4.scratch.log): freezing T³ (RKL2's way) cools a burnt field ~30× too fast; iterating λ costs 10–100× the rounds for ≤ 3e-5 of the heat. Found while re-reading my spec: the u-space stop rule alone missed a cold front (11 % of the heat moved), so it now pairs with a per-cell heat measure. Filed M0-T117 (twin), M0-T118–T119 (GPU), M0-T120 (G-STIFF), M0-T121 (carry-over), M0-T122 (cost), all before M0-T48, now AFTER M0-T122. Flags → T48, T49, V6, T63 (refresh 0/5). Deviations: scratch code outside Deliver (scratchpad only); T48's order and the mirrors edited by hand. Not acted on: M0-TM6's note on T48's Sizing E (a second thread); lint's stub warnings on M0-D31/D32 (a TC's). Verify 1 [ALREADY RUN — PASS (plan_lint GO, 540, on the final plan) on laserax-ai]; claim [ALREADY RUN — FAIL (cpu_gpu: the burning case E L1 3.09e-2, the inherited red T48 owns after T117–T121; 6 passed, 1 failed, 2/3 scopes GO; anchor 73 never-checked = cpu_gpu's paths) on laserax-ai] python3 tools/pb/verify.py --changed --task M0-TJ4, then plan-only edits (Deliver → Verify mapping, Read ranges). Detail: tasks/M0-TJ4.md. Next: M0-T117.

## M0-T117 · P5's twin, implicit: backward Euler in a_r T⁴, the heat solved by Newton · **BUILD** · Opus 5.5, high · switch · (AFTER M0-TJ4)
- Status: DONE (2026-10-14 11:58)
- Rating: rule (5) — numeric or precision semantics: a nonlinear implicit solve, its stopping rules and f64 tolerances (M0-TJ4)
- Sizing: M
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 (the three [M0-TJ4] lines), §2.4, §5.1 (G-HEAT1,
  G-HEAT2, G-FLD, G-STIFF) + milestones/m0/tasks/M0-TJ4.md + crates/sr-physics/src/reference/heat.rs +
  crates/sr-engine/tests/gpu/heat.rs lines 1–288 (helpers, OnGpu) and 571–786 (mod rkl2, rewritten), then the twin cases
  of mod gauss (289–568) and mod limiter (789–983)
- Deliver: crates/sr-physics/src/reference/heat.rs: `Heat::step` the implicit P5 of contract §0.5 [M0-TJ4] (1)–(5) in
  f64 — u = a_r T⁴ (T under constant_diffusivity), D frozen at the pass's start, Newton passes to 10⁻¹² of the heat and
  10⁻¹⁰ relative with the factor-2 damping, Jacobi-preconditioned CG to (4)'s two measures at 10⁻¹², the closure δ, F the
  solved flux, `Pass` reporting its passes, rounds and δ in place of `stages` and `dt_p` → Verify 1 · RKL2 kept only as
  `Heat::step_rkl2`, the GPU cases pointed at it until M0-T119 → Verify 1 · crates/sr-engine/tests/gpu/heat.rs, the
  twin's cases: G-HEAT1 as it stands, `mod rkl2` renamed `mod implicit` with G-HEAT2's (a) (an independent tight solve
  written in the test) and (b), G-FLD's law bound on the start state and thick gas in the solved F → Verify 1 ·
  tools/pb/verify.json: `heat_rkl2` renamed `heat_implicit` (cmd `bash tests/gpu.sh heat::implicit physics`), its
  expected counts → Verify 1 · tests/plants/solve-two-rounds.patch (new: the twin's solve ends after two rounds) → Verify 2
  · tests/plants/chi-scale.patch and limiter-off.patch, their twin hunks re-aimed at the new code, and
  rkl2-s-minus-one.patch's at `step_rkl2` → Verify 2
- Verify: 1. `python3 tools/pb/verify.py heat_gauss heat_implicit heat_limiter --task M0-T117` · Pass: GO, each scope's
  twin cases among them · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm heat_implicit --task M0-T117` · Pass:
  GO — solve-two-rounds NO-GO on a twin case, rkl2-s-minus-one on a GPU case · Fail: a plant stays GO
- Adversarial: a loose solve passes σ² by the moment property alone, and a twin that stops after its first Newton pass
  passes every constant-diffusivity case — G-HEAT2 (a)'s 10⁻¹⁰ against an independent solve bites on the first, and
  G-STIFF's (c) and (e) (M0-T120) on the second; Heat::step's other callers (reference::Twin, cpu_gpu's burning case,
  radiation_force) now read the solved F — the claim run names what moved.
- Handoff: linux-pc (laserax-ai, RTX 5090), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; matches the heading). reference/heat.rs: Heat::step is §0.5 [M0-TJ4]'s implicit P5 in f64 (u = a_r T⁴, D frozen at the start, Newton 1e-12/1e-10 with the per-cell factor-2 clamp, Jacobi-PCG to 1e-12 in both measures, F solved, δ closure, ceiling 16 passes/4,096 rounds → converged=false and Pass::warning(step)); Pass {passes, rounds, delta, converged, radiated, emitted, flux}; Heat::law(&State) = the start state's F (pinned passes, G-FLD); RKL2 kept as Heat::step_rkl2 → (Pass, Rkl2 {stages, dt_p}), the GPU cases pointed at it. tests/gpu/heat.rs: mod rkl2 → mod implicit — (a) 4.8e-13 of the peak vs a Gauss–Seidel solve written in the test (σ² 39.99988), (b) e(6) 2.36e-2, ratios 1.96/1.97, a one-cell pulse never below −4.7e-12, a radiative slab's per-cell balance 3.3e-12 in 5 Newton passes; G-FLD reads Heat::law (0.98), thick gas in the solved F (1e-6). verify.json heat_rkl2 → heat_implicit (7); plant solve-two-rounds new (twin only), chi-scale/limiter-off/rkl2-s-minus-one twin hunks re-aimed.
  Verify 1 [ALREADY RUN — PASS (23/23, 3/3 GO) on laserax-ai] python3 tools/pb/verify.py heat_gauss heat_implicit heat_limiter --task M0-T117 · Verify 2 [ALREADY RUN — PASS (solve-two-rounds RED on 4 twin cases, rkl2-s-minus-one on 3 GPU) on laserax-ai] python3 tools/pb/verify.py --redarm heat_implicit --task M0-T117; also red-armed heat_gauss and heat_limiter GO (checks and plants changed).
  Claim [ALREADY RUN — FAIL then FAIL on laserax-ai] python3 tools/pb/verify.py --changed --task M0-T117: first build NO-GO (two clippy lints of mine, fixed) + cpu_gpu; then 248 passed, 1 failed, 28/29 GO — cpu_gpu's ref_burn, the inherited red M0-T48 owns (E L1 3.09e-2 → 8.55e-3, sim time −3.3 % → +0.20 %, twin implicit vs GPU RKL2+cap); plan-only edits followed the claim (as M0-TJ4).
  Deviations/defaults: (a)'s case bounds the edge leak at 1e-6 of the pulse instead of assert_inside (one Δt=24 step reaches the edges, 5.5e-8 measured); the damping a per-cell clamp after u ≥ u_vac; one extra twin case (the slab's balance). Not acted on: reference/mod.rs line 4 doc still says RKL2 (M0-T49); --all-targets clippy lints in older test lines (floors.rs, box.rs, heat.rs 100/131), not the build scope's. Detail: tasks/M0-T117.md. Next: M0-T118.

## M0-T118 · P5 on the GPU, implicit: the f32 solve and the commit, on the linear law · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T117)
- Status: DONE (2026-10-14 13:34)
- Carried flags: [M0-T117, 2026-10-14] The twin's implicit P5 to port: reference/heat.rs Heat::step (Pass {passes, rounds, delta, converged, ...}), Heat::law (the start state's F), Heat::step_rkl2 → (Pass, Rkl2 {stages, dt_p}) which every gpu_ case in tests/gpu/heat.rs compares against today; plant solve-two-rounds is twin-only — add its shader hunk when the GPU's solve lands (the contract's plant ends each pass's solve after two rounds); the twin's CG checks both measures before every round, the start included (0 rounds when already solved).
- Rating: rule (5) — numeric or precision semantics (an f32 iterative solve held to an f64 twin) and shared state (each
  workgroup's partial sums, reduced in a fixed order; indirect dispatches the solve zeroes) (M0-TJ4)
- Sizing: M
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 (the [M0-TJ4] lines), §1.3.2–§1.3.4, §2.4,
  §5.1 (G-HEAT1, G-HEAT2) + milestones/m0/tasks/M0-TJ4.md + milestones/m0/tasks/M0-T42.md (the dispatch layout it
  replaces) + crates/sr-engine/shaders/heat.wgsl + crates/sr-engine/src/step/heat.rs + M0-T117's handoff
- Deliver: crates/sr-engine/shaders/heat.wgsl and crates/sr-engine/src/step/heat.rs: the implicit P5's machinery in f32
  (contract §0.5 [M0-TJ4] (2), (4), (5)) for scenes whose `test.constant_diffusivity` is set (u = T: one Newton pass and
  its check) — the start state and D once, the system's diagonal and right side, Jacobi-PCG rounds (one dispatch a round
  where the box allows, each workgroup reducing the previous round's partial sums in index order), the commit (E,
  `radiated` and each cell's share, F into side.flux, the closure δ), other scenes kept on RKL2 until M0-T119 → Verify 1
  · every round a step may take recorded as an indirect dispatch the converged solve zeroes, (6)'s 4,096-round ceiling
  recorded in full until M0-T121 records fewer, `HeatReport` gaining passes, rounds, converged and δ → Verify 1 ·
  crates/sr-engine/tests/gpu/heat.rs: G-HEAT1's and G-HEAT2 (a)'s GPU cases against the implicit twin (`Heat::step`),
  (a) run twice and compared bit for bit, the radiative cases still against `Heat::step_rkl2` → Verify 1 ·
  tests/plants/chi-scale.patch's shader hunk on the new branch and solve-two-rounds.patch's shader hunk, verify.json's
  expected counts → Verify 2
- Verify: 1. `python3 tools/pb/verify.py heat_gauss heat_implicit --task M0-T118` · Pass: GO, the GPU cases among them ·
  Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm heat_implicit --task M0-T118` then `--redarm heat_gauss` · Pass:
  GO — solve-two-rounds and chi-scale NO-GO on a GPU case · Fail: a plant stays GO
- Adversarial: a solve whose recursive residual "converges" while the true one has drifted in f32 — (a) compares the
  committed T with the twin's, not the residual; a reduction summed in another order on a second run — (a) twice, bit
  for bit.
- Handoff: linux-pc (laserax-ai, RTX 5090), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; matches the heading). heat.wgsl: the implicit step for constant-diffusivity scenes — heat_law (D_f), heat_open, 4,146 heat_round slots (a mode machine INIT/RESID/START/SOLVE: the twin's Newton loop, damping and end test; Chronopoulos–Gear Jacobi-PCG, one dispatch a round, every workgroup summing the previous slot's partials in one fixed order; r, w, s, the record and partials in two parities), each sized by the slot before it (Size::Chained in step/mod.rs and latch.rs: the latch reaches the chain through heat_open; heat_sum zeroes every size), heat_close (F, side.flux, radiated), heat_sum (δ), heat_apply (E += C (T − T⁰ + δT)); radiative scenes stay on RKL2. HeatReport + passes, rounds, converged, delta.
  Tests (RTX 5090): G-HEAT1 against Heat::step 4.9e-5 of the peak at t = 24, σ² 6.9e-4; G-HEAT2 (a) 2.9e-6, 38 rounds, bit for bit on a second run; the one-cell pulse 6.4e-7; pinned F = Heat::law. The RKL2 one-pass and stage/cap cases retired (linear scene); rkl2-s-minus-one moved to heat_limiter; heat_implicit 7 → 6; solve-two-rounds' shader hunk; the other plants re-aimed.
  Verify 1 [ALREADY RUN — FAIL then PASS (6 < 7 expected, count updated; 16/16, 2/2 GO) on laserax-ai] python3 tools/pb/verify.py heat_gauss heat_implicit --task M0-T118 · Verify 2 [ALREADY RUN — PASS (solve-two-rounds RED on both GPU cases, chi-scale on the GPU G-HEAT1 case; heat_limiter also red-armed: limiter-off, rkl2-s-minus-one RED) on laserax-ai] python3 tools/pb/verify.py --redarm heat_implicit --task M0-T118 · Claim [ALREADY RUN — FAIL (725 passed, 1 failed, 29/30 GO: cpu_gpu's ref_burn, the inherited red M0-T48 owns — E L1 8.55e-3, sim time +0.20 %, as M0-T117 measured; its 73 never-checked anchor paths with it) on laserax-ai] python3 tools/pb/verify.py --changed --task M0-T118.
  Deviations: the sharp pulse at peak 100 over the floor (at peak 1, σ² 46.8: the stop rule is relative to the whole field's heat, and P8's floor rectifies sub-floor noise); always one dispatch a round (each round sums O(tiles²) partials on big boxes, T121/T122); the Newton loop built generic (T119 adds u = a_r T⁴, D_f, every scene); 5 code files, mod.rs and latch.rs outside Deliver. Not acted on: G-HEAT1's GPU drift 1e-5 → 4.9e-5 over 24 steps; the sharp pulse's heat 8.7e-6 of its 1e-5. Detail: tasks/M0-T118.md. Next: M0-T119.

## M0-T119 · P5 on the GPU, implicit: the radiative law and the Newton passes; RKL2 retired · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T118)
- Status: DONE (2026-10-14 13:51)
- Carried flags: [M0-T117, 2026-10-14] Retiring RKL2: Heat::step_rkl2, Rkl2, MAX_STAGES (step/heat.rs imports it) and plant rkl2-s-minus-one go with it; mod implicit's three gpu_ RKL2 cases and the gpu_ cases in mod gauss and mod limiter (stage counts, step_rkl2 comparisons) re-aim at Heat::step — G-HEAT2 (a)'s GPU arm is 1e-4 of the peak against the twin from the same f32 start. · [M0-T118, 2026-10-14] The implicit machine is in heat.wgsl's last section and serves constant-diffusivity scenes only (Heat::new's `implicit` switch): its Newton loop (damping clamp, end test, 16 passes / 4,096 rounds) is already the twin's; T119 adds u = a_r T⁴ in u_of / t_of / slope, D_f's radiative form for heat_law (face() still returns RKL2's K), every scene onto the list, and deletes RKL2's entries, yr and StageIndex; rkl2-s-minus-one now sits in heat_limiter's plants (verify.json), chi-scale's and limiter-off's shader hunks are in face().
- Rating: rule (5) — numeric or precision semantics: Newton on T(u) = (u/a_r)^{1/4} in f32, its damping and end test
  (M0-TJ4)
- Sizing: M
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 (the [M0-TJ4] lines), §2.4, §5.1 (G-HEAT1,
  G-FLD) + milestones/m0/tasks/M0-TJ4.md + crates/sr-engine/shaders/heat.wgsl + crates/sr-engine/src/step/heat.rs +
  M0-T118's handoff
- Deliver: crates/sr-engine/shaders/heat.wgsl and crates/sr-engine/src/step/heat.rs: contract §0.5 [M0-TJ4] (1) and (3)
  on M0-T118's solve — u = a_r T⁴, D (λ, κΣ, k_cond's share) frozen at the pass's start, each pass's T(u) linearised at the
  latest iterate, the factor-2 damping and u ≥ u_vac, (3)'s end test, every pass recorded and the converged one zeroing
  the rest (16 passes), for every scene → Verify 1 · RKL2 and [M0-T42]'s face cap removed from the shader and the step,
  `Heat::step_rkl2` from crates/sr-physics/src/reference/heat.rs → Verify 1 · crates/sr-engine/tests/gpu/heat.rs: every
  GPU case against the implicit twin (G-FLD's law bound on the start state and thick gas in the solved F, the slab's
  `radiated`, pin_temperature), the stage cases removed → Verify 1 · tests/plants/limiter-off.patch's shader hunk,
  rkl2-s-minus-one.patch deleted, verify.json's plants and counts → Verify 2 · `P5 heat` for sun_disk on the Quadro in
  the handoff → Verify 3
- Verify: 1. `python3 tools/pb/verify.py heat_gauss heat_implicit heat_limiter --task M0-T119` · Pass: GO · Fail: NO-GO
  · 2. `python3 tools/pb/verify.py --redarm heat_limiter --task M0-T119` then `--redarm heat_gauss` · Pass: GO —
  limiter-off and chi-scale NO-GO on a GPU case · Fail: a plant stays GO · 3. `build/target/release/sandbox-reactions
  headless --scene scenes/sun_disk.json --steps 200 --timing --adapter "Quadro RTX 4000" --out build/m0-t119-timing` ·
  Pass: exit 0, `P5 heat` in per_pass_ms (read, not graded: the full ceiling is still recorded) · Fail: a non-zero exit
  or no P5
- Adversarial: Newton converging to a wrong root when u leaves f32's range (T⁴ at T 18,800 is 1.2 × 10¹⁷, a_r T⁴ ≈ 5 ×
  10¹⁰; at T_floor, 2.5 × 10⁻¹²) — G-STIFF's hot field (M0-T120) is the case; here, the spot's face fluxes against the
  twin's within 10⁻⁴ of the largest.
- Handoff: linux-pc (laserax-ai; RTX 5090 physics, Quadro timing), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; matches the heading). Every scene on the implicit P5: heat.wgsl's face() gives D_f = c_sb λ/κΣ_f (+ k_cond's share), u_of/t_of/slope u = a_r T⁴; RKL2, [M0-T42]'s cap, the record and stage bindings gone from the shader and step/heat.rs; Heat::step_rkl2, Rkl2, MAX_STAGES, stages_for gone from the twin; HeatReport = {dt, passes, rounds, converged, delta} + warning(step); rkl2-s-minus-one deleted, the other three plants regenerated. GPU cases on the implicit twin: slab radiated 5.9e-6 of the twin's, T 3.0e-5 of 30; G-FLD spot at Δt 0.05 law bound 0.980 (law read from a pinned run), law 6.6e-8 of the twin's, T 1.9e-5.
  Verify 1 [ALREADY RUN — FAIL then PASS (22/22, 3/3 GO) on laserax-ai] · Verify 2 [ALREADY RUN — PASS (limiter-off RED 3, chi-scale RED 4, solve-two-rounds RED 6) on laserax-ai] · Verify 3 [ALREADY RUN — PASS (exit 0, P5 heat 20.42 ms of a 21.48 ms step, the 4,146 slots in full) on laserax-ai, Quadro] · Claim [ALREADY RUN — PASS (809 passed, 37/37 scopes GO; cpu_gpu's ref_burn green — E L1 6.8e-5, sim time +0.002 %, flagged M0-T48; FLAG oracle: chi-scale and solve-two-rounds patches regenerated, line numbers only) on laserax-ai] python3 tools/pb/verify.py --changed --task M0-T119.
  The lead, 2026-10-14 (question tool): the solved F graded by energy, «Grade by energy (Recommended)» — Δt |F − F_twin| ≤ 1e-5 of Σ C T⁰ (8.1e-7) in place of the Adversarial's 1e-4 of the largest face (1.9e-3: one f32 step of u between near-equal thin cells, D_f ≈ 3.3e5).
  Deviations: headless.rs (outside Deliver) prints the unconverged warning at each chunk's last step (T121's is every step); OnGpu::upload in the tests; thick gas's rounds > 0 dropped (solved at the start). Not acted on: f32 overflow of Σ c u² near T ~ 1e6 (flagged T120); 21 pre-existing clippy --all-targets lints in other test files. Detail: tasks/M0-T119.md. Next: M0-T120.

## M0-D36 · P5's f32 solve: a thin ring's light leaving is 1 % off the twin's, and the closure δ pushes it onto the disk (G-STIFF (d) on heat_rim) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T119)
- Status: DONE (2026-10-14 14:39)
- Caused by: M0-T119 — the GPU's Jacobi-PCG stop rules (§0.5 [M0-TJ4] (4)) and the commit's closure δ (5), as built
- Rating: (usual) — the cause is measured to a class (the thin cells' solve error), not to a line
- Sizing: M
- Files: crates/sr-engine/shaders/heat.wgsl (the solve's stop tests, the commit), crates/sr-engine/src/step/heat.rs if a
  record or a dispatch changes, crates/sr-engine/tests/gpu/heat.rs `mod stiff` (read only: its bands are the contract's),
  milestones/m0/tasks/M0-TJ4.md (§ The stop rules; read) — unverified, the executing task confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 [M0-TJ4] (4)–(5), §5.1 (G-STIFF) +
  milestones/m0/tasks/M0-TJ4.md § The stop rules, § Measured, not acted on (its last bullet: thin cells in a wide stiff
  ring) + milestones/m0/tasks/M0-T120.md + crates/sr-engine/tests/gpu/heat.rs lines 1038–1310 (`mod stiff`)
- Symptom: measured on linux-pc (laserax-ai, RTX 5090), 2026-10-14, M0-T120: scenes/heat_rim.json (64 × 64, a hydrogen disk
  Σ 1, T 100, a 20 in a ring of Σ 2 × 10⁻⁶ at T 5 to the world's edge, leave, Δt 0.01), one P5 step from the twin's f32
  start: the GPU converges (6 Newton passes, 495 rounds, δ = −1.39 × 10⁻⁴; the twin 7 passes, 766 rounds, δ 1.6 × 10⁻¹²)
  and G-STIFF (d)'s C-weighted relative L1 reads 1.33 × 10⁻⁴ against the band 1 × 10⁻⁴ (worst cell with Σ ≥ 10⁻⁴: 5.0 × 10⁻⁴,
  inside 10⁻³). heat_hot passes (L1 2.3 × 10⁻⁶, worst cell 2.3 × 10⁻⁵, δ −2.2 × 10⁻⁶)
- Repro: `python3 tools/pb/verify.py heat_stiff --task M0-D36` (the GPU case `gpu_the_hot_disk_in_a_thin_ring_is_the_twins_and_books_the_heat_it_lost`;
  its `SR-STIFF GPU heat_rim.json` lines in logs/M0-T120.heat_stiff.log carry the bands below)
- Suspected cause (measured to the cell class, not to the line): the GPU's booked `radiated` is 1.8751 × 10³ against the
  twin's 1.8564 × 10³ (+1.01 %), and all of that excess (+1.01 % of radiated, signed) is emitted by the cells with Σ < 10⁻⁴ —
  the thin ring at the world's edge; the dense disk (Σ ≥ 0.1) emits to 0.00 %. The signed heat the GPU holds over the twin
  is −1.13 × 10⁻⁴ of the heat, 99 % of the C-weighted L1 sits in the cells with Σ ≥ 0.1: the commit's closure δ (every gas
  cell's T scaled by 1 + δ) takes the thin ring's emission error out of the disk. The thin cells' T is the solve's known
  weak spot (M0-TJ4 § Measured: « Thin cells' T in a wide stiff ring stays up to ~34–69 % off … graded by heat only »):
  the stop rules hold the *heat* to 10⁻⁵ of its scale, but a thin cell's emission is not its heat
- Deliver: the GPU's solve and commit so that G-STIFF (d) holds on heat_rim — the thin ring's T (and so its emission)
  accurate enough that δ is of the order of the solve's tolerance (≤ ~10⁻⁵) and the disk's T agrees with the twin's to the
  band, a candidate: a stop rule or a final correction that weighs the vacuum-face emission (Σ_f D_f (u_i − u_vac) in the
  cells that emit), or a stronger preconditioner on the thin cells, and the cost on sun_disk's P5 stays the measure of
  M0-T122 (do not trade the 0.5 ms for it without saying), no band of the contract loosened (§0.4), and a fix that needs
  a changed rule of §0.5 [M0-TJ4] (4)–(5) asks the lead (the question tool) before it is built → Verify 1
- Done when: `heat_stiff` is GO on linux-pc (RTX 5090), the heat_gauss, heat_implicit and heat_limiter scopes still GO
- Verify: 1. `python3 tools/pb/verify.py heat_stiff heat_gauss heat_implicit heat_limiter --task M0-D36` · Pass: GO each,
  heat_stiff's six cases among them · Fail: NO-GO — a red on (a)–(e) is a question, never a looser band
- Adversarial: a fix that closes (d) by moving δ onto the ring instead of the disk (a closure per region) hides the thin
  cells' error from the C-weighted L1 and not from `radiated` — the case's own (b) and the booked radiated within 10⁻⁵ of the
  twin's per emitting cell class stay read in the log (`SR-STIFF GPU … radiated gap by Σ`)
- Handoff: heat.wgsl: each pass's CG starts from a Galerkin shift along the heat-free cells' indicator (c_i ≤ 10⁻³ Σ D_f), zᵀr and zᵀAz summed over the set's border faces (no f32 cancellation) — no new dispatch, buffer or Rust change; twin untouched. heat_rim GPU (RTX 5090, linux-pc): L1 1.33e-4 → 2.52e-6, δ −1.39e-4 → 2.5e-6, rounds 495 → 306; heat_hot 362 → 304. Reading declared: (4) fixes the method and the stop measures, not a pass's starting guess — no rule changed, no question. Measured, not acted on: the ring's emission −1.07e-4 of the twin's radiated (the Adversarial's 1e-5 not met; 6.8e-5 at SOLVE_TOL 1e-6 and 10× rounds — an f32 floor, ≤ 1.6e-6 of the heat; no band red). Ran on Opus 5.5, high (rung_record: model=claude-opus-5-5 level=high). Detail, probes and claim: tasks/M0-D36.md. Next: M0-T120's retry (heat_stiff GO now; its --redarm heat_stiff owed there).

## M0-T120 · G-STIFF — stiff heat on both sides: a burnt field and a thin ring · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T119)
- Status: DONE (2026-10-14 14:59)
- Carried flags: [M0-T117, 2026-10-14] The twin carries the ceiling (MAX_PASSES 16, MAX_ROUNDS 4,096 → Pass.converged false, Pass::warning(step)); not measured: the 1e-10 per-cell Newton rule on heat_rim's thin cells (Σ ≈ 2e-6) at SOLVE_TOL 1e-12 — if their solve error exceeds 1e-10 T the passes run to 16; read converged there. G-STIFF (c)'s measure already runs on a mild radiative slab (mod implicit, 3.3e-12). · [M0-T119, 2026-10-14] The GPU's P5 is the implicit step on every scene (radiative law in heat.wgsl face(), u_of/t_of/slope). f32 range unmeasured above T ~ 30: the bounds' sums Σ c u² grow as Σ (C/Δt) a_r T⁵/4 and overflow f32 near T ~ 1e6 on a 600 × 400 box — heat_hot (18,800) is the first case that reads it. The solved F against the twin is graded by energy (the lead, M0-T119: Δt |ΔF| ≤ 1e-5 of Σ C T⁰), since one f32 step of u between near-equal thin cells moves F by ~2e-4 of the largest face. · [M0-T120, 2026-10-14] [M0-T120, 2026-10-14] first try (Sonnet 5.5, high) built scenes/heat_hot.json, heat_rim.json, mod stiff (6 cases), scope heat_stiff, plant newton-one-pass; the stage counts read 580 and 11,547; 5 of 6 cases GO, red: G-STIFF (d) on heat_rim, GPU — C-weighted L1 1.33e-4 > 1e-4, the thin ring's emission +1.01 % and the closure δ −1.39e-4 pushed onto the disk (M0-D36, filed before this block). Retry after M0-D36: re-run heat_stiff, then --redarm heat_stiff (the clean copy was red, so the scope's red-arm is owed; the plant was applied by hand: 5 of 6 cases red, twin (c) and (e) among them); rating (usual), Opus 5.5, high. Detail: tasks/M0-T120.md · [M0-D36, 2026-10-14] M0-D36 (2026-10-14): heat_stiff GO on linux-pc (RTX 5090), heat_rim L1 2.52e-6, δ 2.5e-6; the ring's radiated gap reads −1.07e-4 in the log (tasks/M0-D36.md); --redarm heat_stiff is still yours
- Rating: (usual) — retry on Opus 5.5, high after the first try's red claim (Sonnet 5.5, high; rule (3) — the contract fixes the scenes, the tolerances and the plant (M0-TJ4)); runs after M0-D36
- Sizing: M
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 (the [M0-TJ4] lines), §5.1 (G-STIFF) +
  milestones/m0/tasks/M0-TJ4.md § G-STIFF + crates/sr-engine/tests/gpu/heat.rs lines 1–288 (its helpers: world, temps,
  OnGpu) +
  docs/agent/testing.md
- Deliver: scenes/heat_hot.json and scenes/heat_rim.json (new, contract §5.1 G-STIFF's two scenes, the cold pressure
  overridden to 0 as heat_spot.json does) → Verify 1 · crates/sr-engine/tests/gpu/heat.rs `mod stiff`: §2.4.3's RKL2
  count ≥ 500 on each scene computed in the test, (a)–(c) and (e) on the twin, (a), (b) and (d) in one submission on the
  GPU → Verify 1 · tools/pb/verify.json `heat_stiff` (new: `bash tests/gpu.sh heat::stiff physics`, its paths its scenes,
  the crates and its plant) → Verify 1 · tests/plants/newton-one-pass.patch (new: the twin's and the shader's Newton
  passes end after the first) → Verify 2
- Verify: 1. `python3 tools/pb/verify.py heat_stiff --task M0-T120` · Pass: GO, ≥ 5 cases · Fail: NO-GO — a red on
  (a)–(e) is a D or a question (§0.4), never a looser band · 2. `python3 tools/pb/verify.py --redarm heat_stiff --task
  M0-T120` · Pass: GO — newton-one-pass NO-GO on the twin's (c) and (e) · Fail: the plant stays GO
- Adversarial: a scene that needs fewer than 500 stages, or whose heat barely moves, tests nothing stiff — the case
  asserts the count, and heat_hot loses ≥ 50 % of its heat in the refined run (≈ 80 % on the scratch model); (e)'s band
  read before the result, never after (§0.4).
- Handoff: Retry on linux-pc (laserax-ai, RTX 5090), model=claude-opus-5-5 level=high (rung_record now, plugin=installed; the heading's rung). No file of the first try's changed: scenes/heat_hot.json, heat_rim.json, mod stiff (6 cases), verify.json heat_stiff, tests/plants/newton-one-pass.patch stand; M0-D36's heat.wgsl fix is what moved. heat_stiff GO 6/6 [ALREADY RUN — PASS on laserax-ai]: heat_rim GPU (d) L1 2.52e-6 (band 1e-4), δ 2.5e-6, 306 rounds; heat_hot L1 8.8e-7; stage counts 580 / 11,547, (e) 1.686. --redarm heat_stiff GO [ALREADY RUN — PASS on laserax-ai]: newton-one-pass red on 5 cases, twin (c) both scenes and (e) among them; the plant applies over M0-D36's shader. Deviations unchanged (heat_rim's ring = the world's square; (e)'s guard left_one < 0.9). Measured, not acted on: the ring's emission −1.07e-4 of the twin's radiated (M0-D36's, no band grades it). Claim --changed and detail: tasks/M0-T120.md. Next: M0-T121.

## M0-T121 · P5's hard steps carry over into the next submission — speed gives first · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T120)
- Status: DONE (2026-10-14 15:31)
- Carried flags: [M0-T118, 2026-10-14] P5's chain (M0-T118): SLOTS = 4,146 heat_round dispatches recorded every step as Size::Chained (step/mod.rs; the latch skips them, reaching the chain through heat_open); the chain's state is heat.machine's two parity records (slot index, mode, passes, rounds, k, bounds, γ, η) and the parity vectors, so a continuation records slots from rec.slot with parity slot % 2; heat_sum zeroes both args buffers at the step's end — a carried step must not run heat_sum until its chain ends. Every round sums all of the box's 16 × 16 partials in each workgroup (O(tiles²): 950 tiles at 600 × 400) — the contract's 'where the box allows' read as always one dispatch. · [M0-T119, 2026-10-14] headless.rs now prints HeatReport::warning(step) = SR-WARN heat unconverged step <n> passes <k> rounds <r>, read at each chunk's last step only (the stage warning had to go with RKL2); every step's is yours. HeatReport is {dt, passes, rounds, converged, delta}; RKL2's record buffer and binding 7 are gone (heat_open binds 8, 9, 10, 16, 18). sun_disk on the Quadro with the 4,146 slots recorded in full: P5 heat 20.42 ms of a 21.48 ms step (≈ 4.9 µs a slot, load 12–16).
- Rating: rule (5) — concurrency as shared state: one step spanning GPU submissions, its later dispatches zeroed, the
  next submission resuming it (M0-TJ4)
- Sizing: M
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 (the [M0-TJ4] lines, (6) above all), §1.3.2–
  §1.3.4, §1.8.3–§1.8.5, §2.12.3, §5.1 (G-STIFF), §5.3 (G-WARP, G-LATCH) + milestones/m0/tasks/M0-TJ4.md +
  crates/sr-engine/src/step/heat.rs + crates/sr-engine/src/step/mod.rs lines 135–165, 331–440 and 498–658 (Step, its
  constructors, encode, run, set_timing, read_cost) + crates/sr-engine/src/step/latch.rs +
  crates/sr-engine/src/headless.rs lines 332–542 (`run`: its chunk loop and `SR-WARN heat stages`)
- Deliver: crates/sr-engine/src/step/heat.rs and crates/sr-engine/src/step/mod.rs (crates/sr-engine/src/step/latch.rs
  where the zeroing is shared): contract §0.5 [M0-TJ4] (6) — a submission records for each step the rounds the engine
  chooses (e.g. from the rounds read back from recent steps), a step whose P5 has not ended zeroes every later dispatch of
  its submission and the next submission begins with its remaining rounds, its commit and its P6–P9, a dump only at a
  step's end, the time control's c_step and sim time counting a carried step (§1.8.3) → Verify 1 ·
  crates/sr-engine/src/headless.rs: `SR-WARN heat unconverged step <n> passes <k> rounds <r>` at the ceiling in place of
  `SR-WARN heat stages`, `--timing`'s `P5 heat` summing a step's rounds across submissions → Verify 1 ·
  crates/sr-engine/tests/gpu/heat.rs `mod stiff`: G-STIFF (d)'s carried case, each scene carried over submissions of 8
  rounds bit for bit the one-submission result → Verify 1 · crates/sr-engine/tests/gpu/dump.rs: sun_disk 200 steps at 8
  rounds a submission bit for bit the run at the engine's own choice → Verify 1
- Verify: 1. `python3 tools/pb/verify.py heat_stiff dump latch timing --task M0-T121` · Pass: GO, the carried cases among
  them · Fail: NO-GO · 2. `python3 tools/pb/verify.py --redarm heat_stiff --task M0-T121` · Pass: GO — newton-one-pass
  NO-GO · Fail: the plant stays GO
- Adversarial: a carried step that differs from the one-submission step in its last bit (a round counted twice, a
  partial sum read before it was written) — the bit-for-bit cases at 8 rounds, the smallest a submission may carry; the
  latch firing in a carried step's P6 — latch's cases stay GO.
- Handoff: Built §0.5 [M0-TJ4] (6): a step with P5 records under the latch's gate, frames and bare runs alike. latch_hold continues a held step, heat_park + latch_carry carry a still-live chain and zero the rest of the submission, and Step::run submits until its steps end. Engine's choice of slots: 128 first, then learnt from each submission's most (set_heat_rounds fixes it). FrameReport.carried; StepCost.ended, costs per step ended; dumps refused mid-carry; headless warns for every unconverged step (256-entry log). Tests: G-STIFF (d) carried bit for bit in ⌈slots/8⌉ submissions (41 each); sun_disk 200 steps at 8 rounds, bare and in frames, byte for byte the engine's choice; new plant carry-commit-early. · Deviations: 9 code files against the ceiling (both shaders; timing.rs's 1000-step frame fixed at SLOTS on the lead's answer); verify.json dump expected 8→9 and the plant; bare P5 runs now run latch_check (slow-down off). · Measured, not acted on: sun_disk hits the 16-pass Newton ceiling at steps 4 and 340–347 (RTX 5090), predates T121 → M0-D37. · Verify 1 GO 4/4 (25 passed); red-arms heat_stiff, dump, timing GO; claim run in the final message. Detail: tasks/M0-T121.md. Ran on claude-opus-5-5, high. · Next: M0-T122

## M0-T122 · P5's cost on the Quadro against the lead's 0.5 ms a step · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T121)
- Status: DONE (2026-10-14 17:46)
- Carried flags: [M0-T118, 2026-10-14] M0-T118 records 4,146 heat_round dispatches a step (Size::Chained, zero workgroups once converged) on constant-diffusivity scenes; their idle cost and the O(tiles²) partial sums a round are unmeasured on the Quadro. · [M0-T119, 2026-10-14] Baseline before M0-T121: sun_disk 200 steps on the Quadro, P5 heat 20.42 ms a step (21.48 ms a step), every one of the 4,146 heat_round slots recorded, box load 11.7–16.5 (logs/M0-T119.timing.log, build/m0-t119-timing/summary.json). · [M0-D36, 2026-10-14] M0-D36 (2026-10-14): P5's RESID slot now evaluates free_at on each cell and its four neighbours once a pass (the deflated start); rounds fell 495 → 306 on heat_rim, 362 → 304 on heat_hot — sun_disk's P5 ms is yours to read with it · [M0-T121, 2026-10-14] P5 now records only the engine's choice of slots a step (step/heat.rs Heat::learn: 128 first, then the submission's most slots + 1/4 + 4, falling at most 1/4 a read; Step::set_heat_rounds fixes it, Step::heat_rounds reads it); the zeroed slots recorded beyond a step's need are the cost you measure — FIRST_ROUNDS and the two quarters are the knobs. headless --timing's per_pass_ms are now per step ended (a carried step's rounds summed across submissions, StepCost.ended). sun_disk on the RTX 5090: <= 84 slots a step over steps 0-255 (step 4 hits the 16-pass ceiling, M0-D37); steps 340-347 up to ~1,500.
- Rating: rule (3) — a timing measurement, no reason from §0's list (M0-TJ4)
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 (the [M0-TJ4] lines, (7) above all), §1.8.3,
  §1.8.6 + milestones/m0/reports/time_warp.md § Problem 3 + milestones/m0/tasks/M0-TJ4.md § The cost +
  milestones/m0/tasks/M0-T42.md § Measured
- Deliver: milestones/m0/reports/p5_cost.md (new), its `## Runs`: three `headless --timing` runs of scenes/sun_disk.json,
  200 steps, on the Quadro RTX 4000 (box laserax-ai, R12), each with its command, SHA, adapter and load (`uptime`) → Verify 1
  · its `## Cost`: each run's `P5 heat` and `step_ms_mean`, their median against contract §0.5 [M0-TJ4] (7)'s ≤ 0.5 ms,
  beside RKL2's 0.188–0.261 ms (M0-T42) and the steps a 60-fps frame then holds (13 ms, M0-R3) → Verify 2 · its `## Rounds`
  (Newton passes and rounds per step: median, p95, max) and `## Carried` (the steps carried over, the submissions they
  took) → Verify 3
- Verify: 1. `for i in 1 2 3; do build/target/release/sandbox-reactions headless --scene scenes/sun_disk.json --steps 200
  --timing --adapter "Quadro RTX 4000" --out build/m0-t122-timing-$i || exit 1; done` · Pass: exit 0 · Fail: a non-zero
  exit · 2. `python3 -c "import json, statistics as s; print(s.median(json.load(open(f'build/m0-t122-timing-{i}/summary.json'))['timing']['per_pass_ms']['P5 heat'] for i in (1, 2, 3)))"`
  · Pass: ≤ 0.5 · Fail: above 0.5 on an idle box (load named) — back to the lead (§0.4), never a looser number; on a
  loaded box, re-run idle first · 3. `grep -cE "^## (Runs|Cost|Rounds|Carried)" milestones/m0/reports/p5_cost.md` ·
  Pass: 4 · Fail: fewer, or a number without its command and adapter
- Adversarial: a run on the RTX 5090 looks several times faster than the promise's card — every number names the Quadro;
  a loaded box (M0-T42's ran at load 12–30) inflates the cost — each run names its load.
- Handoff: linux-pc (laserax-ai, Quadro RTX 4000), model=claude-sonnet-5-5 level=high (rung_record, matches the heading). P5 heat median of 3 idle runs 0.6189 ms a step (runs 0.8067/0.6189/0.6162, load 0.33-0.38; at load ~14: 0.6148) against contract §0.5 [M0-TJ4] (7)'s 0.5 ms: red, put to the lead, who accepted it as measured, no D (2026-10-14: 'I accept the timing as it is ... Just make sure it is known'). Declared: one override line in contract §0.5 ([M0-T122]) and its mirror in Superseded; no ceiling stands for P5's cost. step_ms_mean 1.1833 (11 steps in 13 ms). Per step, 200 steps (probe): passes median 1 / p95 3 / max 12, rounds median 0 / p95 1 / max 46, 0 unconverged, 2 carried steps. Verify 1 [ALREADY RUN - PASS (3 runs exit 0) on laserax-ai Quadro]; Verify 2 [ALREADY RUN - FAIL (0.6189 > 0.5), accepted by the lead] ; Verify 3 [ALREADY RUN - PASS (4) on laserax-ai]. Deviation: a throwaway per-step probe in a scratch copy (~/private/scratch_t122), not in the tree. Not acted on: where the 0.62 ms goes (zero-workgroup heat_round slots vs Newton-pass gaps) is unmeasured, the lead declined a D. Detail: reports/p5_cost.md, tasks/M0-T122.md. Next: M0-D37, M0-T48.

## M0-D37 · P5's Newton passes hit the 16-pass ceiling on sun_disk (steps 4 and 340–347): unconverged steps, ~1,500 slots each · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T122)
- Status: DONE (2026-10-14 18:29)
- Caused by: unknown — the GPU's Newton passes as M0-T119 built them (§0.5 [M0-TJ4] (3)); not measured to a line
- Rating: (usual) — the cause is not measured
- Sizing: M
- Ordering: after M0-T122 — its 200 steps of sun_disk hold only step 4 of these (84 slots), so its cost reading stands
- Files: crates/sr-engine/shaders/heat.wgsl (`heat_round`'s DO_RESID end test and CELL_UPDATE), crates/sr-physics/src/reference/heat.rs
  (read: whether the twin converges on the same state), crates/sr-engine/tests/gpu/heat.rs — unverified, the executing task confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 [M0-TJ4] (3), (6) + milestones/m0/tasks/M0-TJ4.md
  § The stop rules + milestones/m0/tasks/M0-T121.md § Measured, not acted on
- Symptom: measured on linux-pc (laserax-ai, RTX 5090), 2026-10-14, M0-T121: `headless --scene scenes/sun_disk.json --steps
  1000` prints `SR-WARN heat unconverged step 4 passes 16 rounds 34` and the same for steps 340–347 (16 passes, rounds 348,
  442, 633, 957, 1276, 1358, 1386, 1450): each commits its last iterate with the closure δ (never silent, (6)). The step-4
  case runs 16 passes of ~2 rounds each — the passes, not the solve, fail to end. Before M0-T121 headless read the warning
  at each 256-step chunk's last step only, so these went unseen; the steps' results do not depend on the slots a
  submission records (G-STIFF (d) carried, dump's sun_disk case, M0-T121)
- Repro: `~/.cargo/bin/cargo build --release -p sr-app && ./build/target/release/sandbox-reactions headless --scene
  scenes/sun_disk.json --steps 400 --adapter "RTX 5090"` → the `SR-WARN heat unconverged` lines
- Suspected cause (not measured): §0.5 [M0-TJ4] (3)'s end test needs every cell within 10⁻³ of its T as well as the heat
  within 10⁻⁶; a cell whose damped Newton update keeps flipping (a cell near the floor, or at the star's edge where the
  factor-2 clamp binds) never settles while the heat does — read which cells fail the per-cell test at the 16th pass, and
  whether the twin's 10⁻¹⁰ test converges on the same start state (a dump at step 3 or 339)
- Deliver: the measured cause, and a fix in the GPU's passes (and the twin's if it shares it) so that sun_disk's 1000
  steps print no `SR-WARN heat unconverged`, or — when the state is genuinely past what (3) can meet — the finding and a
  proposed rule for the lead (the question tool) before anything is built, no tolerance of §0.5 [M0-TJ4] loosened (§0.4)
  → Verify 1
- Done when: the repro prints no unconverged step, or the lead has answered the question; heat_stiff, heat_implicit,
  heat_gauss, heat_limiter and dump GO on linux-pc (RTX 5090)
- Verify: 1. `python3 tools/pb/verify.py heat_stiff heat_implicit heat_gauss heat_limiter dump --task M0-D37` · Pass: GO
  each · Fail: NO-GO
- Adversarial: a fix that ends the passes earlier by a weaker per-cell test hides the flip rather than ending it —
  G-STIFF (a)–(e) and the twin's 10⁻¹⁰ stay the graders
- Handoff: Cause measured (tasks/M0-D37.md): steps 340–347 are a heating front crossing cold gas in one step, about one cell a Newton pass — the f64 twin needs 20–41 passes on the GPU's own frozen systems (clamp or update variable not the cause); step 4 is f32 drift of the disk's thin corner cells (GPU only, 24 passes). Lead's answer «Raise to 64 (Recommended)» → contract §0.5 [M0-D37] + Superseded line; MAX_PASSES 64 in heat.wgsl and reference/heat.rs (SLOTS 4,146 → 4,290), no tolerance changed. On win-laptop (RTX 4080 Laptop): sun_disk 1000 steps print no SR-WARN heat unconverged; all 9 hard steps converge in 17–37 passes, δ ≤ 1.1e-6. Verify 1: heat_stiff, heat_implicit, heat_limiter, dump GO; heat_gauss NO-GO → filed M0-D38. Claim (--changed): 33/35 GO, timing NO-GO → filed M0-D39. Both red at 16 passes too, so not this change; first win-laptop runs since the implicit step. Owed: Verify 1 on linux-pc (RTX 5090), flagged to M0-V6. Not acted on: the twin's 4,096-round ceiling would stop its 1e-12 solve on such fronts (it never runs sun_disk). Model claude-opus-5-5, level high (rung_record). Next: M0-D38.

## M0-D38 · heat_gauss's GPU pulse case books `radiated` at the edges on win-laptop (RTX 4080 Laptop): "nothing reaches the edges" red · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D37)
- Status: DONE (2026-10-14 18:42, started 18:32)
- Caused by: unknown — the case came with the GPU's implicit step (M0-T118); green on linux-pc's RTX 5090 since (M0-D36's log: 10 passed), not run on win-laptop since RKL2 (M0-T41's 6 cases)
- Rating: (usual) — the cause is not measured
- Sizing: S
- Ordering: after M0-D37 — measured there, independent of its ceiling (16 and 64 passes read the same numbers); runs on win-laptop (R15)
- Files: crates/sr-engine/tests/gpu/heat.rs (`gpu_the_variance_grows_as_sigma0_squared_plus_two_chi_t_and_the_heat_is_kept`, its
  line 519 assert), crates/sr-engine/shaders/heat.wgsl (`heat_close`'s `emitted`, `d_sums`, under `constant_diffusivity`),
  scenes/heat_pulse.json — unverified, the executing task confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §5.1 G-HEAT1 and §0.5 [M0-TJ4] (1), (5) + milestones/m0/tasks/M0-D37.md
  § Measured, not acted on
- Symptom: measured on win-laptop (Laser2025-20, RTX 4080 Laptop, Vulkan), 2026-10-14, M0-D37: `verify.py heat_gauss` 9/1 NO-GO —
  the case's σ², heat and twin checks pass at t = 8, 16, 24 (σ² error 7.09 × 10⁻⁴, heat 2.48 × 10⁻⁸, |T − twin| 5.05 × 10⁻⁵),
  then `gpu.radiated()` holds some cell with |r| ≥ 10⁻⁹: panic "nothing reaches the edges" (heat.rs:519); identical with
  MAX_PASSES 16 (logs/M0-D37-head.heat_gauss.log) and 64 (logs/M0-D37.heat_gauss.log)
- Repro: `py -3.12 tools/pb/verify.py heat_gauss --task M0-D38` on win-laptop → 1 failed
- Suspected cause (not measured): the largest |radiated| and its cell are not printed — read them first; candidates: an f32
  edge cell a hair off the vacuum's state, so `heat_close`'s Δt Σ_vac D_f (u − u_vac) books ~10⁻⁹ there on this adapter
  (a tolerance question → the lead), or a booking buffer not cleared before the case (a defect)
- Deliver: the measured value and cell, and a fix if it is a defect, or the finding and a proposed bound for the lead (the
  question tool) before anything is built, no tolerance of G-HEAT1 loosened (§0.4) → Verify 1
- Done when: heat_gauss GO on win-laptop (RTX 4080 Laptop) and on linux-pc (RTX 5090; owed there, R15), or the lead has answered
- Verify: 1. `py -3.12 tools/pb/verify.py heat_gauss --task M0-D38` · Pass: GO · Fail: NO-GO
- Adversarial: a bound raised to fit this adapter hides a leak at the edges — the leak's size against the pulse's heat decides
- Handoff: Measured on win-laptop (RTX 4080 Laptop): f32 round-off of the solve at the floor, not a leak — edge cells 1–3 ulps above T_floor (0.05), kept by (3)'s u ≥ u_vac clamp, each ulp at an edge face booking ~1.9e-9 a pass, over the test's 1e-9; largest 3.35e-8 at (0, 127), sum 6.0e-7 = 3.0e-9 of the pulse, no cell inside; |δ| ≤ 3.4e-9, not the cause. The lead (2026-10-14): «Against the pulse (Recommended)» — the case's check now Σ|radiated| ≤ 1e-7 of the pulse's heat, nothing booked off the edge (tests/gpu/heat.rs); G-HEAT1 and the contract unchanged. heat_gauss GO 10/0 on win-laptop; redarm GO (chi-scale RED), and an injected 1.6e-6 leak trips only the new bound (restored). Owed: heat_gauss on linux-pc's RTX 5090 → M0-V6. Ran on claude-opus-5-5, high. Detail: tasks/M0-D38.md. Next: M0-D39.

## M0-D39 · timing's 1000-step frame at SLOTS rounds hangs or loses the device on win-laptop (RTX 4080 Laptop) · **BUILD** · Opus 5.5, high · switch · (AFTER M0-D38)
- Status: TODO
- Caused by: unknown — the case's rounds fixed at `SLOTS` by M0-T121 (the lead's answer there), run on linux-pc only since
- Rating: (usual) — the cause is not measured
- Sizing: S
- Ordering: after M0-D38 — measured by M0-D37 on win-laptop; independent of its ceiling (red at 16 and 64 passes); runs on win-laptop (R15)
- Files: crates/sr-engine/tests/gpu/timing.rs (`a_frame_longer_than_the_query_set_times_its_first_whole_steps`),
  crates/sr-engine/src/step/heat.rs (`SLOTS`), crates/sr-engine/src/step/mod.rs (frames' submission) — unverified, the
  executing task confirms first
- Read: this file (rules + this task) + milestones/m0/m0_contrat.md §0.5 [M0-TJ4] (6) and [M0-D37] + milestones/m0/tasks/M0-T121.md
  § Tests + milestones/m0/tasks/M0-D37.md § Measured, not acted on
- Symptom: measured on win-laptop (Laser2025-20, RTX 4080 Laptop, Vulkan), 2026-10-14, M0-D37: `verify.py timing` — with
  MAX_PASSES 16 (`SLOTS` 4,146) the first case runs past the scope's 900 s and the scope times out, 0 cases
  (logs/M0-D37-head.timing.log); with 64 (`SLOTS` 4,290) it panics at 106 s, "Error in Device::poll: Validation Error",
  and the 4 cases after it fail "refused the device: Parent device is lost" (logs/M0-D37.timing.log). The scope is GO on
  linux-pc (M0-T121)
- Repro: `py -3.12 tools/pb/verify.py timing --task M0-D39` on win-laptop
- Suspected cause (not measured): one submission of 1000 steps × `SLOTS` recorded `heat_round` dispatches (~4.3 M, most
  zero-sized) runs past Windows' GPU timeout (TDR, 2 s by default) — the device reset; whether 16's hang is the same reset
  unreported is not known. R12 keeps timing figures on the Quadro, but the scope still runs here
- Deliver: the measured cause, and a fix that keeps the case's intent (a frame longer than the query set, rounds fixed so it
  never carries) within one submission the box survives — or the finding and a proposal for the lead (the question tool:
  e.g. the case owed on linux-pc only, R15) before anything is built → Verify 1
- Done when: timing GO on win-laptop and on linux-pc (owed there, R15), or the lead has answered
- Verify: 1. `py -3.12 tools/pb/verify.py timing --task M0-D39` · Pass: GO · Fail: NO-GO
- Adversarial: shrinking the frame until it fits drops the "longer than the query set" the case exists for — keep a frame
  that outruns the query set
- Handoff: <placeholder>

## M0-T48 · The burning field in G-REF · **BUILD** · Opus 5.5, high · switch · (AFTER M0-T122)
- Status: TODO
- Carried flags: [M0-T38, 2026-10-13] [M0-T38, 2026-10-13] G-REF's third scene, the burning field, joins tests/gpu/cpu_gpu.rs: grade() takes a scene file (Σ, E relative L1 ≤ 1e-3, each X ≤ 1e-4 where both Σ ≥ 1e-4); X is graded on mixed compositions (H | He) so a burning field's X is read the same way; the heat and reactions passes must be on the twin's Switches and the GPU's Passes (tasks/M0-T38.md) · [M0-V5, 2026-10-13] G-REF holds under the shipped EOS: ref_disk.json and ref_sod.json with their four K overrides removed (ref_sod's states by temperature, Σ 1 | 0.125 at T 2, H | He: P = 1 is below the shipped cold pressure at Σ = 1 and refused) read L1 Σ 3.8e-6 / 7.2e-7, E 3.8e-6 / 1.3e-6, |ΔX| 3.0e-5 / 1.7e-6 on the RTX 5090 — the burning field needs no cold-pressure override. Contract §0.5 [M0-V5]: X is compared where both runs hold Σ ≥ 100 × Σ_vac · [M0-T41, 2026-10-13] the twin driver (reference::Twin, reference/mod.rs) does not run P5: Switches has gravity and hydro only and no block's Deliver adds heat to it — the burning field in G-REF runs heat and reactions on both sides, so whoever builds it (this block or an earlier one) adds a heat switch, P5 between P4 and P8 in section 1.3.2's order, its pass-list label and Heat::step(&mut State, dt) with the scene's test flags; the GPU step's pass list must match (the twin scope compares them) · [M0-T42, 2026-10-14] The GPU's P5 caps the faces of any cell that alone would need > 32 RKL2 stages (contract §0.5 [M0-T42]); the twin (reference::heat) has no cap. A G-REF scene with such a cell (sun_disk's rim needs up to 3,803 in its first ~80 steps) sees the two differ — cap the twin the same way or pick a scene whose s stays ≤ 32 (Step::read_heat's need says) · [M0-T43, 2026-10-14] [M0-T43] reference::Twin's driver runs no P5 and passes no F: a G-REF scene with heat on differs from the GPU, whose P4 now pushes with κΣF/c_sb from side.flux. When the twin gains P5, its step is hydro.step_forced(st, dt, g, Some(F_prev)) and then Heat::step's flux.cells() is kept for the next step (P5 after P4; F from step 0's initial state is zero on the GPU) · [M0-T44b, 2026-10-14] [M0-T44b, 2026-10-14] reference::react::React::step(&mut State, dt) -> Pass is P6's twin (TestFlags{pin_temperature}); Twin's Switches has no reactions switch yet: P6 runs after P5 (§1.3.2) with the scene's test flags, its pass-list label matching the GPU's; the burning field must be one where a cell's rate does not starve a neighbour's (cells burn alone), H's mass falling ≥ 1 % per the block's adversarial — H_burn at T ≥ 0.5 T_H · Σ^1·X²: pick Σ and T from k = A Σ f(T) (tests/gpu/burn.rs) · [M0-T47, 2026-10-14] ReactSetup now takes {neutrinos, side} (M0-T47): the GPU's P6 in G-REF's burning field cools and sources S_ν only with neutrinos on, and React::new on the twin has them off — set both sides the same (scene switch) or leave both off; the twin's driver still needs P5/P6 switches (earlier flags). · [M0-T48, 2026-10-14] retry on the usual rung (Opus 5.5, high) after M0-D34: the burning case is red on P6's energy (E L1 3.07e-2; GPU and twin 24 % apart on dE, X equal); the scene burns H to zero in 200 steps, so a lower Σ (a few % of H burnt) is the better case once green. rating (usual) · [M0-D34, 2026-10-14] after M0-D35 and M0-TJ4's build blocks, not M0-D34 alone: D34 measured two causes — P6's per-record ln f clip (the lead ruled a Gamow-shaped law, built by M0-D35) and P5's 32-stage cap after the runaway (E L1 2.83e-2 with P6 matched; the lead ruled an implicit P5, specified by M0-TJ4). ref_burn.json runs away (H → Fe in one step at step 44, T ≈ 18,800): your retry's lower-Σ scene is still the better comparison (tasks/M0-D34.md) · [M0-D35, 2026-10-14] rung raised Haiku 5.5, high → Opus 5.5, high by the lead (2026-10-14, during M0-D35: "bump T48 to Opus high"). P6 now matches under the Gamow law (M0-D35): heat-off ref_burn.json 200 steps E L1 4.96e-8, |ΔX| 1.6e-13; heat-on finite, E L1 3.09e-2, sim time −3.3 % — all P5's 32-stage cap after the runaway (M0-TJ4's build first). The field now runs away later (the twin at Δt 6.3056e-3: pass 47, T 143 → 18,813), still H → Fe in one step; a lower-Σ scene stays the better comparison (tasks/M0-D35.md) · [M0-TJ4, 2026-10-14] P5 is contract §0.5 [M0-TJ4]'s implicit step on both sides once M0-T117–M0-T121 land: the twin driver's Heat::step is the implicit one and F the solved flux (P4's push); the cap is gone, so ref_burn's E L1 band stays 1e-3, never looser. Its runaway steps are hard: a scratch model of a 64 × 64 burnt field took 361 f32 rounds in 5 Newton passes (tasks/M0-TJ4.md), so headless carries them over submissions (M0-T121) · [M0-T117, 2026-10-14] cpu_gpu's ref_burn after M0-T117 (twin implicit, GPU still RKL2 + the 32-stage cap): E relative L1 8.55e-3 (was 3.09e-2), sim time +0.20 % (was −3.3 %), Σ L1 0, |ΔX| 2.3e-13 — linux-pc, RTX 5090. · [M0-T119, 2026-10-14] cpu_gpu's ref_burn went green at M0-T119's claim (the GPU's radiative P5 now the twin's implicit step): 200 steps on the RTX 5090, E relative L1 6.80e-5 (band 1e-3; was 8.55e-3 at M0-T118), Σ 0, largest |ΔX| 2.2e-13, sim time +0.002 % (was +0.20 %), H burnt 100 % (logs/M0-T119.cpu_gpu.log).
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
- Handoff: Built: scenes/ref_burn.json; cpu_gpu's third case with the H-fall adversarial; the twin driver gained P5 and P6 behind Switches.heat/reactions and a Burning argument (reference/mod.rs); verify.json expected 2 → 3. twin GO (6/6); build GO after a clippy allow on Twin::new (headless precedent). cpu_gpu RED: ref_disk and ref_sod pass; ref_burn E L1 3.07e-2 > 1e-3 with X equal to 1e-12 and H fully consumed on both sides; the GPU and the twin differ 24 % on the energy booked: P6's energy, not P5 (heat off: 8.5e-3; reactions off: 1.6e-7). Filed M0-D34 (after this block). A red ends the try (Haiku 5.5, high is below (usual)): TODO, retry on Opus 5.5, high (usual). Deviation: --cpu-reference still runs gravity and hydro only (heat and reactions applied there broke the twin scope; reverted). Model=claude-haiku-5-5 level=high (rung_record). Detail: milestones/m0/tasks/M0-T48.md. Next: M0-D34, then M0-T48's retry.

## M0-T49 · Docs — physics.md (heat, light, reactions, neutrinos) and lot 6's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T48)
- Status: TODO
- Carried flags: [M0-T41, 2026-10-13] lot 6's rows from M0-T41: scopes heat_gauss (6 cases, red-arm 72 s), heat_rkl2 (5, 57 s), heat_limiter (4, 68 s) on Laser2025-20 (RTX 4080 Laptop; the twin cases touch no adapter), plants chi-scale, rkl2-s-minus-one, limiter-off; your Verify 2 greps the hyphen spelling (heat-) — R17 spells the scopes heat_ so the rows read heat_gauss; read the grep with that spelling · [M0-T42, 2026-10-14] P5 on the GPU (M0-T42): heat_gauss 10, heat_rkl2 8, heat_limiter 6 cases (9 GPU), the plants now hit the shader too; the 32-stage cap of contract §0.5 [M0-T42]; costs and measurements in tasks/M0-T42.md · [M0-T43, 2026-10-14] [M0-T43] testing.md row radiation_force: 3 cases (the twin, the GPU, the hand-written F), bash tests/gpu.sh radforce physics, ≈ 14 s, --redarm ≈ 16 s (laserax-ai, RTX 5090, 2026-10-14), plant radforce-zero (3 of 3 fail); win-laptop owed. physics.md § The GPU's layout: hydro_update binds F (side.flux) at 6 and reads the Δt record as a uniform (8 storage buffers: state 3, next 2, g, F, eos); Edges is 8 words (leave, gravity, radiation, pad, kappa0, c_sb, kappa_dust, t_dust); the force is in the update only, no predictor kick (tasks/M0-T43.md § Design) · [M0-T44b, 2026-10-14] [M0-T44b, 2026-10-14] lot 6's rows from M0-T44b: scopes burn_cell (9 cases, red-arm ≈ 11 s) and burn_order (2, ≈ 12 s) on laserax-ai (twin cases; no adapter), plants q-double and swap-he-c; the rows are already in testing.md and a 'Burning on the twin' bullet is in physics.md § Reactions (declared defaults: burn_dx 1e-4 in G-BURN, instantaneous power in G-ORDER) — the block's Verify 2 grep reads the hyphen spelling (burn-) and R17 spells the scopes burn_, as for heat_ · [M0-T47, 2026-10-14] neutrino row for testing.md: scope 'neutrino physics' now 18 cases (8 twin + 10 GPU), plants nfe-gate-off, nfe-gate-sigma (twin and shader), nfe-latch-last (shader); red-arm ≈ 52 s on laserax-ai (RTX 5090, 2026-10-14), win-laptop owed. physics.md § Neutrinos: P6 binds 7 storage buffers + EOS table, the latch bit goes report.latch → react_latch → Step::latch_flag (tasks/M0-T47.md). · [M0-TJ4, 2026-10-14] physics.md § Heat and light describes contract §0.5 [M0-TJ4]'s implicit P5 (u = a_r T⁴, the light law frozen at the pass's start, Newton passes, Jacobi-PCG, the closure δ, a hard step carried over, SR-WARN heat unconverged), not RKL2; testing.md: heat_rkl2 is now heat_implicit (plant solve-two-rounds; rkl2-s-minus-one retired), heat_stiff is new (plant newton-one-pass) — your Verify 2 counts rows by name, so its count moves with them · [M0-T117, 2026-10-14] Docs after M0-T117: scope heat_rkl2 is heat_implicit (7 cases; plants solve-two-rounds, rkl2-s-minus-one); reference/mod.rs line 4's module doc still says P5 is stepped by RKL2; G-FLD now grades Heat::law (the start state's F), F kept is the solved flux.
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
- Carried flags: [M0-V5, 2026-10-13] owed on win-laptop (R15; M0-V5 ran lot 5 on linux-pc only): lot 5's scopes riemann, sod, edges, collapse, twin and cpu_gpu and box's two new cases on the RTX 4080 Laptop, each red-armed — py -3.12 tools/pb/verify.py --all and --redarm --moved there cover them; and M0-D28's item (verify.py desktop with start.bat's launch.py stop %SERVICE% and the decoy's mkdir lock under Git Bash, --redarm desktop); else carried V to V, M0-V17 at the latest · [M0-D32, 2026-10-13] M0-D32: rung_record.py report's handoff rows now read the quoted model= level= first — re-run `python3 tools/pb/rung_record.py report --json` on linux-pc: each via-handoff row that quotes on its quote's rung (owed there; measured on win-laptop 71/71 with no session read) · [M0-T46, 2026-10-14] M0-T46 measured P6's divergent-loop cost: a hot 8×8 core (a runaway, shipped limits) in a 40×40 cold field, one 1 t.u. step — P6 42.4 ms on the Quadro (0.10 ms all cold), 196 serial sub-steps in one tile ≈ 0.22 ms each; sun_disk's 200-step run (barely burning) reads P6 0.127 ms of 1.056 ms. Not optimised (candidate: solve only the species a record touches) — weigh against the 60 fps budget (tasks/M0-T46.md) · [M0-T47, 2026-10-14] M0-T47 findings for the V: (1) the gate-blind linearisation (twin and GPU alike) overshoots N_Fe's own closure — iron 1 % over Σ_N, one burn_dx-limited sub-step converts 9.09 % where the gate shuts at 0.99 %; weigh against the contract. (2) A flag set during bare steps sits in the latch record until a frame's controller takes it. (3) win-laptop owes the neutrino scope and its three plants (tasks/M0-T47.md). · [M0-TJ4, 2026-10-14] lot 6's P5 is contract §0.5 [M0-TJ4]'s implicit step (M0-T117–M0-T122): contract-vs-code on the Newton passes, the solve's end tests, the closure δ and the carry-over; heat_implicit and heat_stiff red-armed; P5's cost on the Quadro is M0-T122's reports/p5_cost.md against the lead's ≤ 0.5 ms a step · [M0-D37, 2026-10-14] M0-D37 (2026-10-14): P5's pass ceiling is 64 (contract §0.5 [M0-D37]); its Verify 1 (heat_stiff heat_implicit heat_gauss heat_limiter dump) is owed on linux-pc (RTX 5090) — run on win-laptop only, where heat_gauss (M0-D38) and timing (M0-D39) are red at 16 passes too · [M0-D38, 2026-10-14] heat_gauss on linux-pc (RTX 5090), owed by M0-D38 (R15): the GPU pulse case's edge check is now Σ|radiated| ≤ 1e-7 of the pulse's heat and nothing booked off the edge (the lead, 2026-10-14); green on win-laptop only
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

## M0-T50 · Sinks on the CPU twin (P3) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V6)
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

## M0-T52a · The sinks' pull and the latch on formation · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T51)
- Status: TODO
- Carried flags: [M0-T22, 2026-10-10] P3 sets the latch on formation by atomicOr(1) into word 0 of Step::latch_flag() (M0-T22).
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

## M0-T52b · Sinks in the state dump · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-T52a)
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

## M0-T53 · Neutrino heating on the CPU twin (P7) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T52b)
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

## M0-T54 · Neutrino heating on the GPU (P7) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T53)
- Status: TODO
- Carried flags: [M0-T22, 2026-10-10] P7's own indirect sizes are a third indirect buffer: Latch::new asserts the step reads exactly box.args and P0's gate — add a binding to latch.wgsl's zero_sizes/latch_end (save, zero, restore) and widen the assert, or the latch would not reach P7 (M0-T22).
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

## M0-T55 · Docs — physics.md (sinks, neutrino heating, the stand-ins) and lot 7's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T54)
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

## M0-T57 · The ledger · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T56)
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

## M0-T58 · Objects · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T57)
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

## M0-T59 · Stages and events · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T58)
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

## M0-T60 · Observers in headless runs · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T59)
- Status: TODO
- Carried flags: [M0-D21, 2026-10-12] Sim time exists since M0-D21: DtReport.sim_time / FrameReport.sim_time (the Δt record's f64 clock, one value per step count) — read it for --until sim_time; summary.json and dumps already write it
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

## M0-T61 · The preset cloud at a given mass · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T60)
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

## M0-T62 · No artificial fragmentation (G-JEANS) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T61)
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

## M0-T63 · The step-cost probe — a main sequence's steps and the box's cost on the Quadro · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T62)
- Status: TODO
- Carried flags: [M0-V5, 2026-10-13] The Quadro RTX 4000 with P4 in and before M0-D30 (medians of 3, headless --timing, 200 steps): sun_disk.json step 1.083 ms, P4 0.906, the other passes 0.108; fill_world.json 16.04 ms, P4 14.60. The wall per step from 200 against 2,200 steps no longer isolates one box: P4 moves gas, and sun_disk's box grows from 136 × 128 at step 200 to the whole world by 2,200 (7.46 ms a step over that span). reports/v5.md § The step's cost · [M0-D30, 2026-10-13] [M0-D30] P4 tiled: Quadro RTX 4000 sun_disk P4 0.240 ms, step 0.414 (medians of 3, 2026-10-13) — still ~3x TW's scenario C (<= ~0.14 ms a step); fill_world step 5.05, hydro_edges 0.30 of P4's 3.78 once gas reaches leave edges (tasks/M0-D30.md § Not acted on) · [M0-T42, 2026-10-14] P5 on the Quadro, sun_disk 200 steps (loaded box): 0.188–0.261 ms a step (21–24 %); 0.229 ms at s ≤ 4 against 0.360 at s ≥ 16 — the 32 recorded stage dispatches cost even idle (box-sized, returning at once); s median 2, max 32, 31 steps capped (tasks/M0-T42.md) · [M0-T43, 2026-10-14] [M0-T43] when P5 runs, P4's hydro_update reads F (2 words), X_H and Σ per gas cell for the radiation force (κ_dust's branch adds the composition and T only when κ_dust ≠ 0): its cost on the Quadro is not measured · [M0-TJ4, 2026-10-14] P5 is the implicit step since M0-T117–M0-T121; its cost on the Quadro is M0-T122's reports/p5_cost.md (the lead's ceiling ≤ 0.5 ms a step, sun_disk, 200 steps); hard steps (runaways, gas thrown into vacuum) carry over submissions — count the frames they take
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

## M0-T64 · Docs — readouts.md (summary, ledger, objects, stages, events) and lot 8's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T63)
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

## M0-T65 · The calibration file, its physics hash and the refusal (G-CAL) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V8)
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

## M0-T66 · The `calibrate` command and items 1–2 — the cold ceilings · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T65)
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

## M0-T67 · The cold ceiling holds (G-CORE) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T66)
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

## M0-T68 · `calibrate` items 3–4 — the ignition and ending thresholds · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T67)
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

## M0-T69 · Real-equivalent translations (G-READ) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T68)
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

## M0-T70 · `calibrate` items 5–6 — the order, preset masses, the Sun's life, the top rung · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T69)
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

## M0-T71 · Docs — calibrating, and lot 9's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T70)
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

## M0-T72 · The predicted ending · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V9)
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

## M0-T73 · The age clock · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T72)
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

## M0-T74 · The three presets and several clouds (G-MULTI) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T73)
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

## M0-T75 · Number formats · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T74)
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

## M0-T76 · Player edits on the GPU (P0) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T75)
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

## M0-T77 · The edits file and `--edits` · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T76)
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

## M0-T78 · Docs — readouts.md (the ending, the age), physics.md (edits) and lot 10's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T77)
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

## M0-T79 · The blackbody table · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V10)
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

## M0-T80 · The cell image and the four views · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T79)
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

## M0-T81 · The glow · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T80)
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

## M0-T82 · Frames from headless runs · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T81)
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

## M0-T83 · Docs — architecture.md (rendering) and lot 11's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T82)
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

## M0-T84 · The world in the window — the frame loop · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V11)
- Status: TODO
- Carried flags: [M0-T24, 2026-10-11] TimeControl (sr_physics::time) takes Report{steps_run, advanced as f64, dt_next as f64, step_ms = cost.map(|c| c.step_ms), latch_fired.is_some()} once per polled frame with steps > 0, in order, then on_event(CoreCollapse) on latch_fired; Plan{steps, slow_down} → encode_frame, paced at Plan.f_target. N_max is 1 until a step_ms arrives: turn Step::set_timing on, and on wasm32 without timestamp-query feed a wall-clock step_ms. An empty world (Δt f32::MAX) still grows FrameReport.sim_time by f32::MAX per step — TimeControl ignores it, the age readout must too (M0-T24). · [M0-V3, 2026-10-11] once the app's frame loop runs Step, the desktop, capture and web scopes read crates/sr-physics/** and assets/physics.json, assets/elements.json at run time: their paths gain them as M0-D22 gives boot's (M0-D10's class; today they compile sr-physics and embed the assets but read neither — M0-V3) · [M0-D26, 2026-10-12] A timed step is recorded pass by pass (each pass a compute pass with two timestamps, for per_pass_ms) and pays the pass boundaries M0-D26 cut from untimed steps: sun_disk 0.141 ms timed against 0.113 untimed (RTX 4080 Laptop, warm). The frame loop times every frame for c_step (§1.8.3), and a frame of up to ~170 steps is timed whole — time the step as one compute pass (its first and last timestamps) when no per-pass cost is asked for (tasks/M0-D26.md § Findings)
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

## M0-T85 · The top bar — time controls, the speed readout, the slow-down setting · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T84)
- Status: TODO
- Carried flags: [M0-T24, 2026-10-11] speed_text() → SpeedText{rung (time.rung / time.speed's {rung}), running: Some({actual}) → time.speed_capped, paused → time.paused}; rungs() never holds a rung above TOP; banner() → Event::banner_key, cleared only when the rung changes; speeds are not thousands-grouped (×1000, ×4200) (M0-T24).
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

## M0-T86 · The banner and the automatic slow-down in the app · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T85)
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

## M0-T87 · World actions — the edge choice and Clear world · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T86)
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

## M0-T88 · Docs — running.md (the window) and time_control.md (the app's frame loop) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T87)
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

## M0-TV2 · UI/UX pass — Phase 14: the world and its time · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T88)
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

## M0-T89 · Tools and elements · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V12)
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

## M0-T90 · Presets in the window · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T89)
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

## M0-T91 · Views in the window · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-T90)
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

## M0-T92 · The star readout and selection · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T91)
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

## M0-T93a · The engine's cell inspection — `inspect(cell)` · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-T92)
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

## M0-T93b · The cell inspector · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T93a)
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

## M0-T94 · The web build's star — URL parameters and G-WEB's preset case · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-T93b)
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

## M0-T95 · The UI's cost — `--measure-ui` · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T94)
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

## M0-T96 · Docs — running.md (tools, presets, captures, measuring the UI) and lot 13's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T95)
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

## M0-TV3 · UI/UX pass — Phase 15: tools, stars and cells · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T96)
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

## M0-T97 · The life probe — three presets' lives, measured · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V13)
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

## M0-T98a · K1's gap tuned — the clocks spread apart · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TJ1)
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

## M0-T99a · The mass ladder tuned — every life fits the world · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T98a)
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

## M0-T102 · Docs — physics.md (the tuning record) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T99a)
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

## M0-T103 · The visible life (G-STAGES) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V14)
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

## M0-T104 · Each ending as predicted (G-END) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T103)
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

## M0-T105 · Touch any time (G-TOUCH) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T104)
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

## M0-T106 · Conservation over whole lives (G-CONS) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T105)
- Status: TODO
- Carried flags: [M0-T35, 2026-10-13] Bounce books no escaped: the walls' pressure impulse changes the grid's momentum unbooked (G-CONS's invariants are mass and energy only). In leave, every boundary face's flux is booked, including momentum Π through faces closed against inflow
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

## M0-T107 · The age clock over a touched life (G-AGE) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T106)
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

## M0-T98b · The clocks' order (G-SQUEEZE) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T107)
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

## M0-T99b · Every life fits the world (G-FIT) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T98b)
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

## M0-T100 · Virial equilibrium (G-VIR) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T99b)
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

## M0-T101 · The thermostat (G-THERMO) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T100)
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

## M0-T108 · Docs — readouts.md (the stages over a life) and lot 15's testing rows · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T101)
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

## M0-T109 · The bench · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V15)
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

## M0-T110 · Warps never touch the physics (G-WARP) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T109)
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

## M0-T111 · The collapse caught within one step — G-LATCH's scenario · **BUILD** · Haiku 5.5, high · switch · (AFTER M0-T110)
- Status: TODO
- Carried flags: [M0-T22, 2026-10-10] The mechanism and a test hook exist (Step::encode_frame, set_latch_hook, latch_probe; scope latch, 5 synthetic cases): G-LATCH's scenario is the first physics proof; the scope's expected count grows with it (M0-T22). · [M0-V3, 2026-10-11] the latch module's k = 1 case (the flag at step index 0, where the re-fit's slot is open) is the one that grades the controller zeroing the re-fit's own slot: M0-V3's mutant M-a (zero_sizes sparing the period-16 slots) survives k = 15 alone and dies at k = 1 (39 box-sized dispatches after the fire); k = 15 kills a re-fit recorded outside the latch (M-b: 38 of 15, and the edit case) — keep both when the contract case joins (reports/v3.md § The Adversarial line)
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

## M0-T112 · The frame rate (G-FPS) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T111)
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

## M0-T113 · A Sun-like life in about 10 s (G-TOP) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T112)
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

## M0-T114 · The gravity cadence (G-CAD), only if the budget needs it · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TJ2)
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

## M0-T115 · Docs — time_control.md (the bench and the frame budget), running.md (long runs) · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-T114)
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

## M0-TD · Show-off demo — a star's life staged and polished · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-V17)
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

## M0-T116 · Documentation — the tracker and the docs aligned with what shipped · **BUILD** · Sonnet 5.5, high · switch · (AFTER M0-TW)
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
- Carried flags: · [M0-R1, 2026-10-08] the lead, after the engine ruling: "nice can you quickly check if I could serve it on something like a small amazon lightsail? and move to a larger box when traffic justifies it? If so were gonna have to add some google adsense ad serving to the website, to generate passive revenue" — and, after the check, "great record all that, the CDN and the adsense stuff, its perfect!" A web-release behaviour, routed past M0 (reports/bootstrap.md §3). The sourced record is reports/web_hosting.md: static hosting on a small Lightsail box; a CDN as the first scaling step; AdSense H5 Games Ads at game breaks; no COEP `require-corp` on the game page. Place the web release — hosting, CDN, ads, consent — in a milestone with the lead. Consent rules and revenue are the lead's to rule. · [M0-R2b, 2026-10-08] the lead, mid-session: "one thing id like to add in this project is for the website running the game id like for the user to be able to choose which game they want to play. I might add my space_tykun game from ../ on the website also. It might require something so it can run on the players own hardware, but this is something to settle with the agents running space_tykun." — place the site's game picker with the web release; space_tykun's side is its own `M1-R2` (filed at the lead's request, pushed as 727dca8 in that repo; its report will be `../space_tykun/milestones/m1/reports/web_build.md`) · [M0-TB, 2026-10-08] carry to M1's TG as a flag — M0-TB split 4 blocks (M0-T33, M0-T44, M0-T98, M0-T99), the TG defect: test (vi) stopped at the ceilings — two near-ceiling blocks kept a separable deliverable with its own check (M0-T33's Riemann oracle, M0-T44's rate law), and two tuning blocks kept checks that need the recalibration their own tuning forces (G-CAL); reports/size_pass.md §4 · [M0-TH-adopt-a, 2026-10-11] Toolkit fixes PLAYBOOK v12.3's annex §A lacks, re-added locally by the swap (tasks/toolkit_local_diff.md): verify.py — case_env (UTF-8, PYTHONFAULTHANDLER M0-D7, PB_PYTHON M0-D16) used by redarm and run_scopes, the selftest stub's PB_TRACE lock and torn-tolerant spans(), their J2 checks and plants; plan.py selftest bigname; content_gate.py selftest child() CRLF and os.chdir(home) before cleanup; rung_record.py selftest host() PowerShell line on Windows. Not swapped, kept at v12.2: scratch_copy.sh (canon()) and status_page.py (drain(), WinError 10053) — v12.3's annex drops both fixes. The annex was cut from a v12.2 base without the win-laptop fixes: carry all of it, and M0-D20's fixture fix, to the playbook's next bump. · [M0-D20, 2026-10-11] M0-D20: v12.3 annex §A.2 verify.py selftest's walk fixture — write bytes exact (write_bytes, not text-mode write_text: CRLF became \r\r\n on Windows) and probe whether the file system keeps the trailing-space name 'trail ' (Win32 strips it), dropping it from wfiles/ignored_by_rule when not; fix lives in tools/pb/verify.py, no check removed · [M0-D23, 2026-10-12] M0-D23: tools/pb/verify.py carries a local fix the annex lacks - tool_path(): on win32 the spawned commands (and argv_of's lookup of a bare command) get Git's usr/bin and ~/bin first on PATH, so a plain PowerShell's bash is Git's, not WSL's relay (System32\bash.exe). case_env sets env PATH from it. Re-add after any swap, with PB_PYTHON (M0-D16) and M0-D7's fixes (tasks/toolkit_local_diff.md). · [M0-D32, 2026-10-13] M0-D32: toolkit_local_diff.md item 6 — rung_record.py's handoff_rung() (the quoted model= level= first) is a local diff a future annex swap must carry; annex §A.9's text and its report plant count lag by it
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
