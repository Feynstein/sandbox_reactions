# M0-TG — Task generation: the M0 build pipeline (2026-10-08, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the launch
prompt and on the lead's "carry on") — the heading's rung, no mismatch.

## What exists now
- **The pipeline, under "# Build" in the plan** — 141 new blocks, filed one by one through
  `plan.py append` (the Flow rows and counters written by the tool): 118 T blocks (M0-T1…M0-T116, with
  M0-T52a/b and M0-T93a/b from my own size check), 17 V (16 lot V's closing Phases 3–18, the final
  M0-V17), 3 TV (Phases 3, 14 and 15 — the only phases that add or change screens), 2 TJ (DEFERRED
  decision points), the OPT-B M0-TD, and the documentation task M0-T116 between M0-TW and M0-TZ.
  Counters: T=116 · V=17 · TJ=2 · TV=3 · TD=1 (a `TD` counter added first — `append` asked for it).
- **The Build preamble** (once, never per block): the claim run (R9 step 5), the scope conventions (one
  GPU test binary, crates/sr-engine/tests/gpu/, a module per scope; pure-CPU scopes by module filter),
  the ⏱ rule, the physics rules (R8, §5.5, §0.4), and the V's common Done-when line (lint asked for it
  once).
- **Contract amended, the lead's yes** — §3.6 (launch commands `build/target/release/…`, heading) and
  §6.5 (cargo's target `build/target/` via `.cargo/config.toml`; red-arm copies may share
  `~/.cache/sandbox-reactions/redarm-target/` for dependencies only), and docs/agent/testing.md's hazard
  line, all tagged [M0-TG]. The question and the answer, verbatim, are in logs/M0-TG.log:
  "Approve all three (Recommended)".
- **Header edits**: Rules' E-block and gates-to-builds counts; Repo facts' run-commands line (which block
  writes each launch.json service and start.sh; the build/target/ amendment); M0-TW's order now names
  M0-V17 and M0-TD (heading and Flow row).

## The lots (Phase = lot, each closed by its V and a commit gate — §10's "~6 tasks" a phase)
| Phase | Lot | Blocks | What it builds | Screens → TV |
|---|---|---|---|---|
| 3 | 1 · the walking skeleton | T1–T9, TV1, V1 | workspace + build check, GPU device, cell state + first pass, headless + G-BOOT, window + status + G-DESK, start.sh, web entry, capture harness, docs | yes — TV1 |
| 4 | 2 · registries, EOS, guards | T10–T17, V2 | units + registries, reactions, EOS, G-STR, G-LIC, web smoke, G-WATCH, docs | no |
| 5 | 3 · step framework | T18–T27, V3 | booking, floors, Δt + guard, the box, frames + latch, timing, TimeControl, scenes, dumps, docs | no |
| 6 | 4 · gravity | T28–T32, V4 | twin FFT, twin gravity, GPU FFT, GPU gravity + the first step-cost measurement (RT6) | no |
| 7 | 5 · gas flow | T33–T40, V5 | twin hydro + Riemann, GPU hydro, leave edge, G-COLL, twin driver, G-REF, G-BOX | no |
| 8 | 6 · heat, light, burning | T41–T49, V6 | twin/GPU heat, radiation force, rates + twin/GPU burning, neutrinos + iron gate + latch | no |
| 9 | 7 · black holes, ν heating | T50–T55, V7 | twin/GPU sinks, their pull + latch, sinks in dumps, twin/GPU P7 | no |
| 10 | 8 · watching the star | T56–T64, V8 | summary, ledger, objects, stages/events, headless observers, preset cloud, G-JEANS, the O14 probe | no |
| 11 | 9 · calibration | T65–T71, V9 | G-CAL, the calibrate command + items 1–2, G-CORE, items 3–4, translations (G-READ), items 5–6 | no |
| 12 | 10 · readouts, presets, edits | T72–T78, V10 | ending, age clock, presets + G-MULTI, number formats, P0 edits, the edits file | no |
| 13 | 11 · drawing (headless) | T79–T83, V11 | blackbody table, views, glow, headless frames | no (no window wired yet) |
| 14 | 12 · the window: world, time | T84–T88, TV2, V12 | frame loop in the window, top bar, banner, world actions | yes — TV2 |
| 15 | 13 · the window: tools, stars, cells | T89–T96, TV3, V13 | tools, presets, views, star readout, inspector (engine + panel), web star, `--measure-ui` | yes — TV3 |
| 16 | 14 · a life measured, tuned | T97, TJ1, T98–T102, V14 | the life probe, stand-ins (deferred), G-SQUEEZE + K1, G-FIT + ladder, G-VIR, G-THERMO | no |
| 17 | 15 · stages and endings | T103–T108, V15 | G-STAGES, G-END, G-TOUCH, G-CONS, G-AGE | no |
| 18 | 16 · speed, frame rate | T109–T115, V16 | bench, G-WARP, G-LATCH's scenario, G-FPS, G-TOP, TJ2 (deferred), cadence (deferred) | no |
| 19 | final | V17, TD, push gate | the whole contract against the code; the demo | — |
| close-out | — | TW, T116, TZ | the walk, the docs task, the next plan | — |

## Carried flags — each answered
- **[M0-TP] the top speed is M0's hardest promise; the first solver lot measures one step at ~600 × 400
  on the named adapter before any ending** → M0-T31 (lot 4, the first solver) runs `--timing` on the
  Quadro for a world-filling field and a Sun-sized box; M0-V4 reads it against TW's proxy; M0-T34, T42,
  T46 add their passes' costs; M0-T63 is the main-sequence probe.
- **[M0-D1] no `!` before a letter or backslash in Verify one-liners** → none in the 141 blocks
  (checked mechanically before filing).
- **[M0-R3] the box (A2) and the GPU latch (R4) are first-lot architecture; fewer, fused kernels; the
  first solver lot measures steps per life (O14) and the box's cost on the Quadro** → the box (M0-T21)
  and the latch (M0-T22) land in lot 3, before any pass; GPU pass blocks name few dispatches; O14 is
  M0-T63 (the first point where a main sequence can run — gravity, flow, heat and burning all exist),
  read by M0-V8, which wakes M0-TJ2 early on a projection over 33.3 ms a frame.
- **[M0-TC] (a)** box, latch, twin-before-shader → lot 3; every pass's twin block precedes its GPU block.
  **(b)** calibration its own lot (lot 9) after the solver lots and before presets/readouts; its decision
  points as PLAN amendments: M0-TJ1 (stand-ins, wakes on M0-T97's measured §1.6.4 failures) and M0-TJ2
  (top speed red at 30 frames, wakes on M0-T113 or M0-V8). **(c)** calibrate and ⏱ scopes are the
  lead's in V windows (M0-V9, V14, V15, V16), split by `--only` and one life per case. **(d)** §5.5's
  UNVERIFIED rows each carry the "red is a D or a question, never a looser number" arm in their block;
  G-FIT's ladder and K1's gap are M0-T99 and M0-T98, the first full-life lot's targets. **(e)**
  render_reserve_ms: M0-T95 builds `--measure-ui`, the lead's windowed run in M0-V13. **(f)** the
  cadence is M0-T114, DEFERRED with a measured wake. **(g)** every lot's blocks add their own §5 scopes and
  plants — all 38 §5 scopes are placed (grav-force … web).
- **[M0-TH] (a)** scratch copies carry target/ and .cache/ → settled by the contract amendment above (the
  lead's yes), M0-T1 writes the mechanism (tests/cargo.sh) and M0-T9 documents it; binary-launching
  scopes build in their own copy (tests/smoke.sh, M0-T4) — never the shared cache. **(b)** the web target
  adds crates, not tooling → M0-T7. **(c)** capture_web.mjs's live tier waits for Playwright (M0-T15
  brings playwright-core) → TV1 captures the no-WebGPU page with Chrome's own `--screenshot`; launch.json
  and the launchers come with M0-T4, T5, T6 and T15.

## Declared defaults (cheap to reverse; no question asked)
- Assumed: one GPU test binary (crates/sr-engine/tests/gpu/, a module per scope) — because one link per
  change instead of ~38; reversible by splitting the binary.
- Assumed: ⏱ scopes name only their own test and scene files in `paths` — because otherwise every physics
  edit's claim run would carry whole lives past 10 minutes; reversible in verify.json. A complete loop
  over 600 s then fires a TR by rule.
- Assumed: UI blocks test a pure view model and keep egui a thin layer — because no UI-testing crate is in
  §1.2's table; TV captures check the drawing.
- Assumed: the probe's dumps live in ~/.cache/sandbox-reactions/dumps/ — because red-arm copies leave
  build/ out; reversible by a fixture path.
- Assumed: the doc pages are written by one docs block per lot (contract §7: "each lot … the page its lot
  owns"; testing.md "every lot that adds a scope").
- Assumed: S1 and S2 ask the lead once even though Q3 pre-ruled them — a TJ is PLAN + LEAD answers; one
  question with the measurement.

## Not acted on (measured or seen, with the reason)
- G-WEB's "not all black" passes on a canvas of plain bg.space (#05070D) — a weak bar in a FROZEN-adjacent
  row; M0-T94's Adversarial and M0-V13 name it, a stricter case is the lead's to grant (Changes by asking).
- G-COLL's scene needs cold pressure off: at Σ = 1, §2.7's K's give Π_cold ≈ 14 against Π_th ≈ 0.1 (my
  arithmetic) — written into M0-T36 as a scene override within the contract's test hooks; not a contract
  change. Sod (M0-T33) the same, for the γ = 2 oracle.
- The same arithmetic says the Sun-like preset starts with cold pressure worth ~⅓ of its virial support —
  physics for M0-T63/T97 to measure, not a defect today.
- `plan.py append` writes a Flow row's Order from `--after`, not the heading: my first (reverse-order)
  filing wrote "AFTER M0-TB" on 138 rows; I restored the plan from my pre-filing copy
  (scratchpad/plan.before-TG.md) and re-filed in forward order, phase gates added by hand (the tool has no
  call for a heading or a trailer). A tool note, not a defect: `move` does not resync the Order either.
- M0-TG's own `Read:` names the contract whole (lint WARN, the TP's text) — left; every block I wrote names
  sections.

## TB's table on my own output (PLAYBOOK §2.1 step 2, tests i–vi)
Counting convention: files = code, tests, scripts, scenes, data and docs a block writes (crate-relative
paths counted); its verify.json entry and plant patches apart (+n); one-line registrations (a `mod` line,
a pass in step/mod.rs's list) not counted. Paths checked on disk: of the 122 BUILD blocks' Deliver paths,
only docs/agent/testing.md, tools/pb/verify.json, tools/pb/review_page.py (the TVs run it), milestones.md
and the contract exist today; every other path is written by its own block or an earlier one (the tree
held no game code, `git ls-files`, 2026-10-08).
- (i) one artifact kind: 122/122 after my splits (M0-T93 → engine API M0-T93a + panel M0-T93b).
- (ii) ≤2 independent failure classes: 122/122 after M0-T52 → M0-T52a (pull + latch) and M0-T52b (dump).
- (iii) no context switch: none found; every lot ends in its own docs block.
- (iv) cuts by dependency: each pass's twin before its GPU block (artifact); rulings live in the two
  DEFERRED TJs, never mid-block; tuning blocks carry a BLOCKED arm instead of a ruling.
- (v) Deliver lists: every path named exactly or marked (new).
- (vi) one notch smaller: the calibrate command moved from M0-T65 to M0-T66; 8 E blocks marked (M0-T6, T39,
  T48, T52b, T91, T93a, T94, T111). Candidates I left for TB's judgment, each with its reason in the
  table: M0-T33 (riemann.rs could leave first), M0-T98/T99 (scope + tuning in one thread).
- Ceilings: max 6 files (M0-T1, config of 3–40 lines each); 7 blocks near 600 lines by estimate (M0-T30,
  T33, T34, T41, T44, T56, T58), each with the fallback split named. Splits TB should still make, by my
  count: 0.

| Block | Files (+manifest/plants) | Est. new lines | Kind | Classes | E | Note |
|---|---|---|---|---|---|---|
| M0-T1 | 6 (+3) | 150 | code module + its scope | ≤2 |  | config files of 3–40 lines; no smaller block leaves a covered green state |
| M0-T2 | 4 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T3 | 4 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T4 | 5 (+1) | 450 | code module + its scope | ≤2 |  | binary + its contract smoke (G-BOOT) kept together |
| M0-T5 | 4 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T6 | 1 (+1) | 30 | code module + its scope | ≤2 | E |  |
| M0-T7 | 6 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T8 | 4 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T9 | 4 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-TV1 | 1 (+0) | — | captures + review page | 1 |  |  |
| M0-T10 | 4 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T11 | 2 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T12 | 1 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T13 | 3 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T14 | 4 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T15 | 4 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T16 | 1 (+1) | 120 | code module + its scope | ≤2 |  |  |
| M0-T17 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T18 | 2 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T19 | 4 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T20 | 3 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T21 | 3 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T22 | 3 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T23 | 3 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T24 | 1 (+1) | 500 | code module + its scope | ≤2 |  | one pure-logic module, one case per §1.8 rule |
| M0-T25 | 5 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T26 | 2 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T27 | 3 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T28 | 2 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T29 | 3 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T30 | 3 (+1) | 550 | code module + its scope | ≤2 |  | near the ceiling — fallback: shared-memory sizes ≤512 first, multi-pass 1024–2048 second |
| M0-T31 | 5 (+0) | 400 | code module + its scope | ≤2 |  |  |
| M0-T32 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T33 | 4 (+1) | 550 | code module + its scope | ≤2 |  | near the ceiling — riemann.rs (the oracle) could leave first as its own block |
| M0-T34 | 4 (+0) | 580 | code module + its scope | ≤2 |  | near the ceiling — fused-kernel layout is the builder's; no green intermediate (predictor alone has no oracle) |
| M0-T35 | 5 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T36 | 4 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T37 | 4 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T38 | 3 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T39 | 3 (+0) | 120 | code module + its scope | ≤2 |  |  |
| M0-T40 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T41 | 4 (+1) | 550 | code module + its scope | ≤2 |  | 3 scopes, 2 classes (transport+stepping+booking · limiter) |
| M0-T42 | 4 (+0) | 500 | code module + its scope | ≤2 |  |  |
| M0-T43 | 3 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T44 | 3 (+1) | 550 | code module + its scope | ≤2 |  | 2 classes (rate law + sub-cycles · ignition order) |
| M0-T45 | 2 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T46 | 4 (+0) | 450 | code module + its scope | ≤2 |  |  |
| M0-T47 | 3 (+0) | 250 | code module + its scope | ≤2 |  |  |
| M0-T48 | 2 (+1) | 80 | code module + its scope | ≤2 | E |  |
| M0-T49 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T50 | 4 (+1) | 500 | code module + its scope | ≤2 |  |  |
| M0-T51 | 3 (+0) | 450 | code module + its scope | ≤2 |  |  |
| M0-T52a | 3 (+0) | 200 | code module + its scope | ≤2 |  | split from M0-T52 by M0-TG (test ii) |
| M0-T52b | 2 (+1) | 80 | code module + its scope | ≤2 | E | split from M0-T52 by M0-TG (test ii) |
| M0-T53 | 2 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T54 | 3 (+0) | 300 | code module + its scope | ≤2 |  |  |
| M0-T55 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T56 | 3 (+1) | 580 | code module + its scope | ≤2 |  | near the ceiling — every §1.9.1 field in one reduction |
| M0-T57 | 2 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T58 | 2 (+1) | 520 | code module + its scope | ≤2 |  | near the ceiling — identity and quantities, 2 classes |
| M0-T59 | 1 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T60 | 3 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T61 | 2 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T62 | 2 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T63 | 1 (+0) | — | report (measurement) | 1 |  |  |
| M0-T64 | 3 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T65 | 3 (+1) | 450 | code module + its scope | ≤2 |  | calibrate command scaffold moved to M0-T66 (test vi) |
| M0-T66 | 3 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T67 | 2 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T68 | 2 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T69 | 3 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T70 | 2 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T71 | 5 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T72 | 1 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T73 | 1 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T74 | 3 (+0) | 250 | code module + its scope | ≤2 |  |  |
| M0-T75 | 1 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T76 | 3 (+1) | 500 | code module + its scope | ≤2 |  | one pass, 2 classes (effect · booking) |
| M0-T77 | 3 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T78 | 3 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T79 | 1 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T80 | 3 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T81 | 3 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T82 | 2 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T83 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T84 | 3 (+1) | 350 | code module + its scope | ≤2 |  |  |
| M0-T85 | 2 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T86 | 1 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T87 | 1 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T88 | 3 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-TV2 | 1 (+0) | — | captures + review page | 1 |  |  |
| M0-T89 | 1 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T90 | 1 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T91 | 1 (+1) | 100 | code module + its scope | ≤2 | E |  |
| M0-T92 | 1 (+1) | 400 | code module + its scope | ≤2 |  |  |
| M0-T93a | 2 (+1) | 80 | code module + its scope | ≤2 | E | split from M0-T93 by M0-TG (test i: engine API vs panel) |
| M0-T93b | 1 (+1) | 300 | code module + its scope | ≤2 |  | split from M0-T93 by M0-TG |
| M0-T94 | 2 (+0) | 120 | code module + its scope | ≤2 |  |  |
| M0-T95 | 2 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T96 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-TV3 | 1 (+0) | — | captures + review page | 1 |  |  |
| M0-T97 | 1 (+0) | — | report (measurement) | 1 |  |  |
| M0-T98 | 2 (+1) | 300 | code module + its scope | ≤2 |  | scope + tuning in one thread (the routine: artifact and its scope); BLOCKED arm on bounds |
| M0-T99 | 1 (+1) | 250 | code module + its scope | ≤2 |  | as M0-T98 |
| M0-T100 | 1 (+1) | 150 | code module + its scope | ≤2 |  |  |
| M0-T101 | 1 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T102 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T103 | 1 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T104 | 2 (+1) | 300 | code module + its scope | ≤2 |  |  |
| M0-T105 | 3 (+0) | 250 | code module + its scope | ≤2 |  |  |
| M0-T106 | 1 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T107 | 2 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T108 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-T109 | 3 (+1) | 450 | code module + its scope | ≤2 |  |  |
| M0-T110 | 1 (+1) | 200 | code module + its scope | ≤2 |  |  |
| M0-T111 | 1 (+0) | 100 | code module + its scope | ≤2 |  | reads a cached dump outside the tree |
| M0-T112 | 1 (+1) | 250 | code module + its scope | ≤2 |  |  |
| M0-T113 | 1 (+1) | 150 | code module + its scope | ≤2 |  |  |
| M0-T114 | 2 (+1) | 250 | code module + its scope | ≤2 |  | DEFERRED — built only on its wake |
| M0-T115 | 3 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
| M0-TD | 1 (+0) | — | demo note + captures | 1 |  |  |
| M0-T116 | 2 (+0) | 80–200 (prose) | doc pages | 1 |  |  |
