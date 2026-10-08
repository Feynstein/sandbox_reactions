# M0-TB — the size, token and rating pass (2026-10-08, linux-pc)

PLAYBOOK §2.1 step 2 over M0-TG's pipeline: every BUILD block through tests (i)–(vi), the token half,
the type-tag and check audit, and every open block rated by §0's rubric. Every change below was put to
the lead first, by the question tool, and applied only as approved (§10 of this report). Raw runs:
logs/M0-TB.log. Model and level: `model=claude-opus-5-5 level=max` (`rung_record.py now`), the
heading's rung.

## 1. Summary
- **Paths grep-verified: 495 before the edits** (273 the block creates, 222 it edits, across 122
  BUILD blocks), **510 after** (278 · 232, across 126). Two `(new)` labels were wrong (§3.1), on three
  blocks.
- **4 splits** (S1–S4, §4): M0-T33 → T33a/T33b, M0-T44 → T44a/T44b, M0-T98 → T98a/T98b, M0-T99 →
  T99a/T99b. Two or more splits means TG's own run of this table missed them. That defect is named
  in §4 and flagged to M0-TZ, for M1's TG.
- **1 ordering defect fixed** (§4, S3/S4). Lot 14's tuning changed physics.json, which makes the
  calibration stale (G-CAL). The checks that need named presets therefore ran before the lead's
  recalibration and could not pass. The lead ruled "Tune first, check after".
- **Named exceptions: none.** No block needs one, so no targeted V was added.
- **E blocks: 8 of 126** open BUILD blocks qualify: M0-T6, T39, T48, T52b, T91, T93a, T94, T111.
- **Token half**: the Build preamble is hoisted to Rules R10–R14, moved byte for byte (20 lines
  diffed). The answered "Resolve with the lead first" list is now a 3-line pointer. Phases 0–1's
  7 finished blocks are stubs: 263 lines archived, each verified. The REJECTED list is started with
  10 items. The header stands at 335/600 lines, Pipeline state 9/60.
- **Checks**: type tags are all right; no `Verify:` was found that cannot fail; 13 `Deliver:` items
  that no check exercised now have a check or a named owner (§8).
- **Rating**: 14 BUILD blocks move to Opus 5.5 high (usual), each for a named reason from rule
  (3)'s list (§9). The rest keep their rung: 104 builds on Sonnet 5.5 high, 8 edit jobs on Sonnet
  5.5 medium, and every CHECK and PLAN block on Opus 5.5 max (gate). Rule (4) has no record yet.
  The register reads `Model ratings: 2026-10-08 by M0-TB`.
- `plan.py lint`: **GO, 474 checks, 0 warnings** (baseline before the edits: GO, 470 checks, 7 stub
  warnings).

## 2. Method
- Read: the plan whole, PLAYBOOK §0 and §2.1, and the passages these rest on (§3.2, §8, §11, §12,
  §14.3, E8). The contract was read only at the sections a check needed (§0, §1.1, §1.6.1, §1.10.1,
  §2.10–§2.12, §3.5, §5.2), with grep for its headings, its §5 scopes and plants, and FROZEN.
- **Paths, by hand, then by machine.** Each block's `Deliver:` was read and its written files listed
  (N creates, E edits). Manifest `paths` globs, run outputs (DIR/…) and prose mentions were left
  out; tools/pb/verify.json and the plant patches are counted apart. A read-only parser drafted the
  list as a checklist. A provenance check then walked the plan in run order against `git ls-files`
  and the disk: a path marked new must not exist and must not be created by an earlier block; an
  edited path must exist or be created earlier. The scratch tools live in the session scratchpad,
  never in the repo.
- **Scopes.** Every `verify.py <scope>` a Verify line calls was checked to exist at or before its
  block. Run-order findings: 0. 89 scopes after the splits (84 game scopes + 3 harness + riemann +
  rates). The contract's §5 names 38 scopes and 38 plants, and all are placed in the plan.
- **Read anchors.** Every report section an open block's `Read:` cites exists: time_warp.md R1, R4,
  R5, R6, R8, M2–M4, A2, Summary; sim_models.md SM-C1, C2, E13, E14, E18, E22, E28, E29, O2, O5,
  O7, O8; plan_redteam.md RT6; contract_rulings.md §2 and Q3; sandbox_interview.md B28.
- **Counting convention** (TG's, kept): files = what the block writes, the manifest and plants apart.
  One-line registrations do not count (a `mod` line, a pass in a dispatch list, a one-line step in
  a capture script). The generated Cargo.lock counts as a file in M0-T1, so it reads 7 files for 6
  written by hand. Line counts are estimates; for M0-TG's blocks they are TG's numbers, and for the
  new halves they are mine.

## 3. The size table — every open BUILD block through (i)–(vi)
(iii) no context switch, (iv) cuts at artifacts, never at rulings, and (v) every `Deliver:` path
grep-verified hold for every row after the splits. Hence ✓ in that column. (vi) and the ceilings are
in the last column. Rung: S5.5 = Sonnet 5.5, O5.5 = Opus 5.5.

### 3.1 Path findings (test (v)), fixed with the lead's yes
- M0-T37 marked `crates/sr-physics/src/reference/mod.rs` new; M0-T28 creates it. It is now named as
  M0-T28's file.
- M0-T80 and M0-T81 marked `crates/sr-engine/src/render.rs` new; M0-T79 creates it. Both now name it
  as M0-T79's file.

### 3.2 The table
| Block | Rung | Files (+manifest/plants) | Paths new·edit | Lines (est.) | (i) kind | (ii) classes | (iii)–(v) | (vi) / note |
|---|---|---|---|---|---|---|---|---|
| M0-T1 | S5.5, high | 7 (+2) | 8·1 | 150 | build setup + scope | 2: workspace config · the build check | ✓ | tests/cargo.sh is the check — cannot leave; Cargo.lock generated |
| M0-T2 | S5.5, high | 4 (+2) | 5·1 | 300 | code + its scope | 2: adapter matching · device and limits | ✓ | — |
| M0-T3 | O5.5, high | 4 (+2) | 5·1 | 350 | code + its scope | 2: layout round trip · renormalisation + determinism | ✓ | — |
| M0-T4 | S5.5, high | 5 (+2) | 6·1 | 450 | command + its smoke | 2: the headless command · the boot smoke | ✓ | G-BOOT is the command's only check — cannot leave |
| M0-T5 | O5.5, high | 4 (+2) | 3·3 | 350 | app shell + scope | 2: window and surface · status endpoint | ✓ | status.rs is the window's only check — cannot leave; `game` mapped to the lead's run [fix] |
| M0-T6 | S5.5, medium | 1 (+1) | 1·1 | 30 | script + case | 1 | ✓ | E |
| M0-T7 | S5.5, high | 5 (+2) | 5·2 | 350 | web entry + scope | 2: the page's GPU check · the wasm runner | ✓ | behaviour graded at M0-T15 (named in its Adversarial) |
| M0-T8 | S5.5, high | 4 (+2) | 4·2 | 300 | code + its scope | 2: actions and PNGs · captions.json | ✓ | — |
| M0-T9 | S5.5, high | 3 (+0) | 2·1 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-TV1 | S5.5, high | 1 (+0) | 1·0 | — | captures + review page | 1 | ✓ | — |
| M0-T10 | S5.5, high | 4 (+2) | 5·1 | 400 | data + validator + scope | 2: the data · its validation | ✓ | the code embeds the data — cannot leave |
| M0-T11 | S5.5, high | 2 (+2) | 2·2 | 250 | data + validator | 2: records · validation | ✓ | — |
| M0-T12 | S5.5, high | 1 (+2) | 2·1 | 400 | code + its scope | 2: EOS formulas · the u(x) table | ✓ | — |
| M0-T13 | S5.5, high | 3 (+2) | 3·2 | 300 | code + its scope | 1 | ✓ | flag: bin-only crate test |
| M0-T14 | S5.5, high | 1 (+2) | 2·1 | 200 | script + scope | 1 | ✓ | — |
| M0-T15 | S5.5, high | 4 (+2) | 4·2 | 250 | smoke + scope | 2: the smoke · the launch entry | ✓ | the npm licence case is one manifest line — kept |
| M0-T16 | S5.5, high | 1 (+2) | 2·1 | 120 | script + scope | 1 | ✓ | — |
| M0-T17 | S5.5, high | 2 (+0) | 1·1 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T18 | O5.5, high | 2 (+2) | 2·2 | 300 | code + its scope | 2: layout and clearing · f64 sums | ✓ | — |
| M0-T19 | O5.5, high | 4 (+2) | 3·3 | 450 | code + its scope | 2: GPU EOS helpers · P8 floors and booking | ✓ | eos.wgsl could leave (own check vs the CPU EOS) — 450 lines, not near the ceiling: kept |
| M0-T20 | O5.5, high | 3 (+2) | 4·1 | 400 | code + its scope | 2: Δt reduction · non-finite guard | ✓ | — |
| M0-T21 | O5.5, high | 3 (+2) | 3·2 | 400 | code + its scope | 2: box fit · dispatch sizes | ✓ | — |
| M0-T22 | O5.5, high | 3 (+2) | 4·1 | 450 | code + its scope | 2: frame encoding · latch control | ✓ | — |
| M0-T23 | S5.5, high | 3 (+2) | 2·3 | 250 | code + its scope | 2: timestamps · --timing output | ✓ | --timing gets a case [fix] |
| M0-T24 | S5.5, high | 1 (+2) | 2·1 | 500 | code + its scope | 1 (pure logic, a case per §1.8 rule) | ✓ | 500 lines, one module |
| M0-T25 | S5.5, high | 5 (+2) | 5·2 | 450 | code + its scope | 2: parsing and refusals · objects | ✓ | the headless scene flags get a case [fix] |
| M0-T26 | O5.5, high | 3 (+2) | 3·2 | 350 | code + its scope | 2: the format · resume determinism | ✓ | — |
| M0-T27 | S5.5, high | 3 (+0) | 1·2 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T28 | S5.5, high | 2 (+2) | 3·1 | 400 | code + its scope | 2: FFT · convolution | ✓ | — |
| M0-T29 | S5.5, high | 3 (+3) | 5·1 | 450 | code + its scope | 2: potential · force | ✓ | — |
| M0-T30 | O5.5, high | 3 (+2) | 4·1 | 550 | code + its scope | 2: shared-memory sizes · multi-pass 1024–2048 | ✓ | 550 lines; one deliverable, no separable item — kept (rated up for (c)/(e), not for size) |
| M0-T31 | S5.5, high | 3 (+1) | 2·2 | 400 | code + its scope + measurement | 2: GPU gravity · step cost | ✓ | — |
| M0-T32 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T33a | S5.5, high | 2 (+2) | 2·2 | ~200 | oracle module + scope | 1 | ✓ | SPLIT S1 from M0-T33: the Riemann oracle, its own scope `riemann` |
| M0-T33b | S5.5, high | 3 (+2) | 4·1 | ~380 | code + its scope | 2: scheme · walls | ✓ | SPLIT S1: the twin scheme graded against M0-T33a |
| M0-T34 | O5.5, high | 3 (+1) | 2·2 | 580 | code + its scope | 2: scheme in f32 · walls | ✓ | 580 lines; no separable deliverable — kept |
| M0-T35 | S5.5, high | 5 (+2) | 3·4 | 300 | extension + scope | 2: leave ghosts · escaped booking | ✓ | — |
| M0-T36 | S5.5, high | 4 (+2) | 3·3 | 250 | extension + scope | 1 | ✓ | — |
| M0-T37 | S5.5, high | 4 (+2) | 3·3 | 400 | code + its scope | 2: twin driver · --cpu-reference | ✓ | reference/mod.rs is M0-T28's, not new [fix] |
| M0-T38 | S5.5, high | 3 (+2) | 4·1 | 200 | tests + scope | 1 | ✓ | — |
| M0-T39 | S5.5, medium | 2 (+1) | 1·2 | 120 | tests + scope | 1 | ✓ | E |
| M0-T40 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | + the riemann row |
| M0-T41 | S5.5, high | 4 (+4) | 7·1 | 550 | code + its scope | 2: diffusion solve · limiter | ✓ | 550 lines; one deliverable — kept |
| M0-T42 | O5.5, high | 3 (+1) | 2·2 | 500 | code + its scope | 2: stages · limiter | ✓ | — |
| M0-T43 | S5.5, high | 3 (+2) | 2·3 | 200 | extension + scope | 1 | ✓ | — |
| M0-T44a | S5.5, high | 1 (+2) | 2·1 | ~200 | module + scope | 1 | ✓ | SPLIT S2 from M0-T44: the rate law, its own scope `rates` |
| M0-T44b | S5.5, high | 2 (+3) | 4·1 | ~380 | code + its scope | 2: sub-cycles · ignition order | ✓ | SPLIT S2: P6 on the twin, on M0-T44a's law |
| M0-T45 | S5.5, high | 2 (+2) | 2·2 | 300 | extension + scope | 2: cooling · the N_Fe gate | ✓ | — |
| M0-T46 | O5.5, high | 3 (+1) | 2·2 | 450 | code + its scope | 2: burning · cost | ✓ | — |
| M0-T47 | S5.5, high | 3 (+1) | 0·4 | 250 | extension | 2: GPU neutrinos · the latch trigger | ✓ | — |
| M0-T48 | S5.5, medium | 2 (+1) | 1·2 | 80 | tests + scope | 1 | ✓ | E |
| M0-T49 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | + the rates row |
| M0-T50 | S5.5, high | 4 (+2) | 4·2 | 500 | code + its scope | 2: formation · accretion, motion, merging | ✓ | 500 lines |
| M0-T51 | O5.5, high | 3 (+1) | 2·2 | 450 | code + its scope | 2: reductions · KDK and merging | ✓ | — |
| M0-T52a | S5.5, high | 3 (+1) | 0·4 | 200 | extension | 2: the pull · the latch on formation | ✓ | split by M0-TG |
| M0-T52b | S5.5, medium | 2 (+1) | 0·3 | 80 | extension | 1 | ✓ | E — split by M0-TG |
| M0-T53 | S5.5, high | 2 (+2) | 3·1 | 300 | code + its scope | 1 | ✓ | — |
| M0-T54 | S5.5, high | 3 (+1) | 2·2 | 300 | code + its scope | 2: deposit · zero dispatch | ✓ | — |
| M0-T55 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T56 | O5.5, high | 3 (+2) | 4·1 | 580 | code + its scope | 2: the reduction · async readback | ✓ | 580 lines; one deliverable — kept |
| M0-T57 | S5.5, high | 3 (+2) | 3·2 | 350 | code + its scope | 1 | ✓ | — |
| M0-T58 | S5.5, high | 2 (+2) | 3·1 | 520 | code + its scope | 2: identity · quantities | ✓ | 520 lines; one deliverable — kept |
| M0-T59 | S5.5, high | 1 (+2) | 2·1 | 450 | code + its scope | 1 | ✓ | — |
| M0-T60 | S5.5, high | 2 (+2) | 2·2 | 300 | extension + scope | 2: conditions · outputs | ✓ | — |
| M0-T61 | S5.5, high | 2 (+2) | 2·2 | 250 | extension + scope | 1 | ✓ | — |
| M0-T62 | S5.5, high | 2 (+2) | 3·1 | 200 | tests + scope | 1 | ✓ | — |
| M0-T63 | S5.5, high | 1 (+0) | 1·0 | — | report (measurement) | 1 | ✓ | — |
| M0-T64 | S5.5, high | 3 (+0) | 1·2 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T65 | S5.5, high | 2 (+2) | 3·1 | 450 | code + its scope | 2: the hash · the refusal | ✓ | SHA-256 could leave (FIPS vectors) — 450 lines, not near the ceiling: kept |
| M0-T66 | S5.5, high | 3 (+2) | 3·2 | 450 | code + its scope + runs | 2: the command · items 1–2 | ✓ | — |
| M0-T67 | S5.5, high | 2 (+2) | 3·1 | 250 | tests + scope | 1 | ✓ | — |
| M0-T68 | S5.5, high | 2 (+2) | 2·2 | 400 | extension + scope | 1 | ✓ | — |
| M0-T69 | S5.5, high | 2 (+2) | 3·1 | 350 | code + its scope | 2: mass · temperature | ✓ | — |
| M0-T70 | S5.5, high | 2 (+2) | 2·2 | 400 | extension + scope | 2: the order · derived values | ✓ | — |
| M0-T71 | S5.5, high | 4 (+0) | 0·4 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T72 | S5.5, high | 1 (+2) | 2·1 | 250 | code + its scope | 1 | ✓ | — |
| M0-T73 | S5.5, high | 1 (+2) | 2·1 | 350 | code + its scope | 1 | ✓ | — |
| M0-T74 | S5.5, high | 3 (+1) | 2·2 | 250 | extension + scope | 1 | ✓ | — |
| M0-T75 | S5.5, high | 1 (+2) | 2·1 | 400 | code + its scope | 1 | ✓ | — |
| M0-T76 | S5.5, high | 3 (+2) | 4·1 | 500 | code + its scope | 2: effect · booking | ✓ | 500 lines |
| M0-T77 | S5.5, high | 3 (+2) | 3·2 | 350 | extension + scope | 2: parsing · triggers | ✓ | — |
| M0-T78 | S5.5, high | 3 (+0) | 0·3 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T79 | S5.5, high | 1 (+2) | 2·1 | 300 | code + its scope | 1 | ✓ | — |
| M0-T80 | S5.5, high | 3 (+2) | 3·2 | 450 | code + its scope | 2: views · interpolation | ✓ | render.rs is M0-T79's, not new [fix] |
| M0-T81 | S5.5, high | 3 (+2) | 3·2 | 250 | code + its scope | 1 | ✓ | render.rs is M0-T79's, not new [fix] |
| M0-T82 | S5.5, high | 2 (+2) | 2·2 | 200 | extension + scope | 1 | ✓ | — |
| M0-T83 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-T84 | S5.5, high | 3 (+2) | 2·3 | 350 | extension + case | 2: the loop · capture actions | ✓ | — |
| M0-T85 | S5.5, high | 5 (+2) | 4·3 | 400 | code + its scope | 2: view model · egui layer | ✓ | capture actions checked by the new `actions` case [fix] |
| M0-T86 | S5.5, high | 2 (+2) | 2·2 | 250 | code + its scope | 1 | ✓ | — |
| M0-T87 | S5.5, high | 3 (+2) | 1·4 | 250 | extension + scope | 1 | ✓ | capture action checked [fix] |
| M0-T88 | S5.5, high | 3 (+0) | 0·3 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-TV2 | S5.5, high | 1 (+0) | 1·0 | — | captures + review page | 1 | ✓ | — |
| M0-T89 | S5.5, high | 4 (+2) | 2·4 | 400 | code + its scope | 2: selection · edits queued | ✓ | capture actions checked [fix] |
| M0-T90 | S5.5, high | 2 (+2) | 1·3 | 250 | extension + scope | 1 | ✓ | preset action checked through desktop's world case [fix] |
| M0-T91 | S5.5, medium | 3 (+1) | 0·4 | 100 | extension | 1 | ✓ | E (a one-line step in actions.json is a registration) [fix] |
| M0-T92 | S5.5, high | 4 (+2) | 2·4 | 400 | code + its scope | 2: lines · selection | ✓ | capture action checked [fix] |
| M0-T93a | S5.5, medium | 2 (+1) | 0·3 | 80 | extension | 1 | ✓ | E — split by M0-TG |
| M0-T93b | S5.5, high | 3 (+2) | 1·4 | 300 | extension + scope | 1 | ✓ | capture action checked [fix] |
| M0-T94 | S5.5, medium | 2 (+1) | 0·3 | 120 | extension + case | 1 | ✓ | E — holds if M0-T5/T7 share the app (flagged) |
| M0-T95 | S5.5, high | 3 (+2) | 2·3 | 300 | extension + scope | 1 | ✓ | — |
| M0-T96 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-TV3 | S5.5, high | 1 (+0) | 1·0 | — | captures + review page | 1 | ✓ | — |
| M0-T97 | S5.5, high | 1 (+0) | 1·0 | — | report (measurement) | 1 | ✓ | + each preset's mass, a, Σc and temperature recorded |
| M0-T98a | S5.5, high | 4 (+0) | 2·2 | ~150 | tuning + record | 1 | ✓ | SPLIT S3 from M0-T98: tuning on preset-free clouds |
| M0-T99a | S5.5, high | 5 (+0) | 3·2 | ~150 | tuning + record | 1 | ✓ | SPLIT S4 from M0-T99: tuning on preset-free clouds |
| M0-T102 | S5.5, high | 1 (+0) | 0·1 | 80–200 prose | doc pages | 1 | ✓ | rows moved to M0-T108 |
| M0-T103 | S5.5, high | 1 (+2) | 2·1 | 200 | tests + scope | 1 | ✓ | — |
| M0-T104 | S5.5, high | 2 (+2) | 2·2 | 300 | tests + scope | 1 | ✓ | — |
| M0-T105 | S5.5, high | 3 (+1) | 3·1 | 250 | tests + scope | 1 | ✓ | — |
| M0-T106 | S5.5, high | 1 (+2) | 2·1 | 250 | tests + scope | 1 | ✓ | — |
| M0-T107 | S5.5, high | 2 (+2) | 3·1 | 250 | tests + scope | 1 | ✓ | — |
| M0-T98b | S5.5, high | 1 (+2) | 2·1 | ~200 | tests + scope | 1 | ✓ | SPLIT S3: G-SQUEEZE after the recalibration (lot 15) |
| M0-T99b | S5.5, high | 1 (+2) | 2·1 | ~150 | tests + scope | 1 | ✓ | SPLIT S4: G-FIT after the recalibration (lot 15) |
| M0-T100 | S5.5, high | 1 (+2) | 2·1 | 150 | tests + scope | 1 | ✓ | moved to lot 15 |
| M0-T101 | S5.5, high | 1 (+2) | 2·1 | 200 | tests + scope | 1 | ✓ | moved to lot 15 |
| M0-T108 | S5.5, high | 2 (+0) | 0·2 | 80–200 prose | doc pages | 1 | ✓ | + four rows from M0-T102 |
| M0-T109 | S5.5, high | 3 (+2) | 3·2 | 450 | code + its scope | 2: the frame loop · bench.json | ✓ | — |
| M0-T110 | S5.5, high | 1 (+2) | 2·1 | 200 | tests + scope | 1 | ✓ | — |
| M0-T111 | S5.5, medium | 1 (+1) | 0·2 | 100 | tests + scope | 1 | ✓ | E |
| M0-T112 | S5.5, high | 1 (+2) | 2·1 | 250 | tests + scope | 1 | ✓ | — |
| M0-T113 | S5.5, high | 1 (+2) | 2·1 | 150 | tests + scope | 1 | ✓ | — |
| M0-T114 | S5.5, high | 2 (+2) | 2·2 | 250 | extension + scope | 1 | ✓ | DEFERRED |
| M0-T115 | S5.5, high | 3 (+0) | 0·3 | 80–200 prose | doc pages | 1 | ✓ | — |
| M0-TD | S5.5, high | 1 (+0) | 1·0 | — | demo note + captures | 1 | ✓ | — |
| M0-T116 | S5.5, high | 7 (+0) | 0·7 | 80–200 prose | doc pages | 1 | ✓ | ≈ the 6-file ceiling: the playbook's one documentation task (§2.4), small edits |

## 4. Splits — applied with the lead's yes
- **S1 · M0-T33 → M0-T33a + M0-T33b.**
  - T33a: the exact Riemann solution for γ = 2 (the Sod oracle), with its own scope `riemann` and
    plant `riemann-gamma`.
  - T33b: the twin's MUSCL-Hancock scheme and G-SOD's twin case, graded against T33a.
  - Why: (vi) — riemann.rs is a separate deliverable with its own check (an independent root-find
    plus the jump conditions) and no ruling dependency. (iv) — T33b needs T33a's file. M0-T33 sat at
    ~550 lines. Written first, in its own session, the oracle is also independent of the scheme it
    grades.
- **S2 · M0-T44 → M0-T44a + M0-T44b.**
  - T44a: the rate law (rates.rs: ω, f(T) and its 256-point table, the gate), with its own scope
    `rates` and plant `rates-table-shift`.
  - T44b: P6's sub-cycles on the twin and G-BURN and G-ORDER's twin cases.
  - Why: (vi) — rates.rs has its own check (the table against the direct form, ≤ 10⁻⁴), and both
    T44b and M0-T46 evaluate it. M0-T44 sat at ~550 lines.
- **S3 · M0-T98 → M0-T98a + M0-T98b, and S4 · M0-T99 → M0-T99a + M0-T99b,** with M0-T100 and M0-T101
  moved to lot 15 (`plan.py move`, byte-identical).
  - What was wrong: as written, each tuning block changed assets/physics.json, then ran its own
    check on the named presets. After any physics.json change the engine refuses those presets
    (exit 6, G-CAL, §2.11) until the lead recalibrates — and that recalibration sat at M0-V14, after
    T98–T101. Four checks could not pass (§8: "a check that cannot pass on the artifact it runs
    against is equally useless").
  - Why split: (iv) — the checks need an artifact that only the lead's recalibration after the
    tuning makes: a calibration matching the tuned physics. (iii) — an iterative tuning loop and a
    test scope are a context switch.
  - New lot 14: T97 → TJ1 → T98a → T99a → T102 → V14. The tuning is measured on preset-free clouds
    with the candidates as scene `overrides`, and physics.json is written once, last. V14 holds the
    lead's one recalibration and the two lives.md dumps remade on the tuned physics.
  - New lot 15: T103–T107 → T98b → T99b → T100 → T101 → T108 → V15. V15 runs all nine ⏱ scopes as the
    lead's runs. An E8 ordering note sits under Phase 17's heading.
  - Follow-on edits:
    - T97 now records each preset's mass, a, Σc and starting temperature.
    - T102 keeps the tuning record; its four testing rows move to T108.
    - V14 and V15 are rewritten to match.
    - Phase 17's title and commit message now cover the whole-life checks.
- **The TG defect** (§2.1: two or more splits name it). TG's test (vi) stopped at the ceilings. Two
  near-ceiling blocks kept a separable deliverable with its own check, and the tuning blocks kept
  checks that need the recalibration their own tuning forces. Flagged to M0-TZ for M1's TG.

### 4.1 Considered and not split
- **M0-T30** (550 lines): one deliverable; its fallback split lives inside one file.
- **M0-T34** (580 lines): one deliverable; no green intermediate (TG's note).
- **M0-T41** (550 lines): transport and stepping form one class, the limiter the second; one
  deliverable.
- **M0-T56** (580 lines): one reduction.
- **M0-T58** (520 lines): one module.
- **M0-T19**: eos.wgsl could leave with its own check, but at 450 lines the block is rated up for
  its shared-shader design instead.
- **M0-T65**: SHA-256 could leave with its FIPS vectors; 450 lines, kept.
- **M0-T15**: the npm licence case is one manifest line.
- **M0-T5**: status.rs is the window's only check.
- **M0-T116**: about at the 6-file ceiling, but it is the playbook's one documentation task (§2.4),
  with small edits.

## 5. Named exceptions
None. After the splits, every block passes (i)–(vi) and the ceilings. No block carries a `Sizing
exception:` line, and no targeted V was filed.

## 6. E blocks — counted
**8 of 126**: M0-T6, T39, T48, T52b, T91, T93a, T94, T111. Each has ≤ 2 files and ≤ 150 lines, no new
module, dependency, schema or string, and an existing scope to verify it.
- M0-T91 keeps E with its one-line capture step, counted as a registration.
- M0-T94 holds E only if M0-T5 and T7 share one eframe App; that is flagged to both.
- No other block qualifies: the new halves each create a module or a scope.

## 7. The token half
- **The Build preamble → Rules R10–R14.** The index lines are in the plan's Rules table; the full
  text is in m0_rules.md, 20 lines byte-identical, diffed in logs/M0-TB.log. Measured reason:
  `plan.py show M0-T1` prints the header and the block, never the preamble, so no builder ever saw
  the claim run's base, the scope conventions, the ⏱ rule or the physics rule. One pointer line
  remains under "# Build".
- **Header de-duplicated.**
  - "Resolve with the lead first" (14 lines) is now a 3-line pointer. Every answer is recorded in
    plan_redteam.md §8, Repo facts' Stack line and the contract's §0.2.
  - Repo facts' loop time is now a pointer to Rules' budget line.
  - The Rules prose now carries the E count, gates to builds (26:132) and where the REJECTED list
    lives.
- **Stubs.** M0-TI, TP, R1, D1, R2a, R2b and R3 moved to plan_archive.md: 263 lines, each verified
  byte for byte by `plan.py stub`. The Phase 0 and Phase 1 commit gates stay in place.
- **The REJECTED list** is started in m0_rules.md with 10 items, each with the reason it is false
  economy.
- **Header against its budget**: 335/600 lines by the lint's measure (Flow table included), Pipeline
  state 9/60, both inside budget. The plan file is 3,350 lines, down from 3,491, with 4 blocks added.

## 8. Type tags, Verify lines, Deliver items
- **Type tags**: every open block's tag matches its id. T, TV and TD are BUILD; V is CHECK; TW is LEAD
  walk + CHECK scribe; TJ is PLAN + LEAD answers; TZ and TB are PLAN. The lint's type check agrees.
- **A `Verify:` that cannot fail**: none found. Every Verify fails before its block's work exists
  (a missing heading, row, case or file). A "≥ n cases" floor on an extended scope is backed by the
  verify.json `expected` count the block raises, and by the scope's red-arm.
- **`Deliver:` items no check exercised, fixed:**
  - **M0-T5**: launch.json's `game` (a visible window) now maps to the lead's start.sh run in M0-V1.
    Its Verify 1 runs `game-xvfb` only.
  - **M0-T23**: `headless --timing` gets a case. timing.rs had tested only the poll report. Pass is
    now ≥ 4.
  - **M0-T25**: the headless `--scene`, `--world` and `--edge` flags get a case. Nothing ran them
    before M0-T31. Pass is now ≥ 6.
  - **Capture actions** (M0-T85: rung, slowdown, and pause through the bar; T87: edge; T89: tool,
    element, paint; T91: view; T92: select; T93b: hover):
    - They were mapped to view-model scopes that never drive capture.rs.
    - Fix: tests/capture/actions.json, new at T85, runs as the `capture` scope's `actions` case.
      Each block appends its step and adds `capture` to its Verify.
    - M0-T90's switched `preset` action: its Verify adds `desktop`, whose `world` case drives it.
    - M0-T88 documents the new case.
  - **M0-T40, T49, T108**: testing rows added for `riemann`, `rates` and the four moved scopes.

## 9. The rating — §0's rubric on every open block
- **The ladder** is the header's Models line, read 2026-10-08 (today). No rung was retired, renamed
  or added, so it was not re-read.
- **Rule (1), escalation**: none — no claim run has failed.
- **Rule (2), gates**: the 18 CHECK blocks (V1–V17 and the TW scribe) and the 4 PLAN blocks (TB, TJ1,
  TJ2, TZ) → Opus 5.5, max. All were already there.
- **Rule (3), the cheapest rung that fits**: edit jobs → Sonnet 5.5, medium (8 blocks). Every other
  BUILD block → Sonnet 5.5, high, unless a reason from the closed list holds.
- **Rule (4), the record**: no class moved. The record holds one BUILD block on a cheaper rung
  (M0-TH, Sonnet 5.5 high, first try GO), and "no record yet" is never a reason.
- **Rule (5)**: the 14 blocks below → Opus 5.5, high (usual).
- **How I read the closed list**, applied the same way to every block:
  - **(c) a design the spec leaves open**: the contract leaves a structure to the builder, and
    later blocks build on it.
  - **(e) numeric or precision semantics, or concurrency**: the block's own text makes one of these
    its acceptance — bit-identical results from parallel code, GPU-side control state, an
    asynchronous readback, or f32 precision over many sub-steps. Not for an f64 twin graded by an
    analytic oracle, where errors are loud.
  - **(f) a library's internals read to find a mechanism**: the mechanism is not in the library's
    public API.
  - A block that reuses a mechanism an earlier block built does not earn the reason again.

| Block | Reason (rule (3)'s list) | From the block's own text |
|---|---|---|
| M0-T3 | (c), (e) | the 14 channels "packing free within 8 storage buffers per stage" — the GPU state layout every later pass binds; "two runs bit-identical (§1.3.4)" |
| M0-T5 | (f), (e) | "ready once the first frame is presented" and `--adapter` inside eframe 0.36's wgpu backend — mechanisms its public API does not show; a status thread sharing the ready state |
| M0-T18 | (c), (e) | "one booking layout every pass writes" — every later pass books into it; f32 with no atomics on state, f64 fixed-order sums, two runs bit-identical |
| M0-T19 | (c) | eos.wgsl, "the helpers every later shader shares" — WGSL has no includes, so how shaders share code is open in the contract and every later shader depends on it |
| M0-T20 | (e) | "a fixed-order tree reduction (§1.3.4)", bit-identical over two runs; the Adversarial's workgroup-scheduling hazard |
| M0-T21 | (e) | the box "written on the GPU as every pass's indirect dispatch sizes — never read back" — GPU-side state that later dispatches consume |
| M0-T22 | (e) | a flag set with atomicOr, a controller dispatch zeroing every later dispatch of the frame, `poll()` never blocking |
| M0-T26 | (e) | "2 × 100 steps through a dump equal 200 straight steps, bit-identical" — every bit of hidden state must round-trip |
| M0-T30 | (c), (e) | "a multi-pass scheme there", "as few dispatches as the limits allow" — a GPU FFT the contract leaves to the builder; f32 within 10⁻⁵ of the twin |
| M0-T34 | (c), (e) | "fused into as few dispatches as the limits allow" within 8 storage buffers per stage — the kernel layout is the builder's (TG's note); the scheme in f32 |
| M0-T42 | (e) | RKL2's s stages (up to 32 a step) in f32, coefficients frozen at P5's start, graded against analytic spreading |
| M0-T46 | (e) | stiff per-cell backward-Euler sub-cycles in f32 within 10⁻⁴ of analytic down to X₀/10; divergence across a workgroup |
| M0-T51 | (e) | "candidates by a fixed-order reduction, accretion per sink in fixed order (§1.3.4)"; two sinks' accretion over one cell |
| M0-T56 | (e) | every §1.9.1 field reduced in fixed order, "read back asynchronously (≤ 2 frames late)", the accumulators reset |

Considered and left on the cheaper rung. These are observations, never defects (§0, no step up on a
guess):
- **M0-T12**: EOS table in f64 against two analytic limits.
- **M0-T28, T29**: f64 FFT and gravity with analytic oracles.
- **M0-T31**: P2 reuses M0-T30's convolution.
- **M0-T33a, T33b, T44a, T44b**: f64 twins with exact oracles.
- **M0-T35**: the boundary bookkeeping is fully specified.
- **M0-T41**: RKL2 in f64.
- **M0-T47**: the atomicOr latch comes from M0-T22.
- **M0-T54**: indirect zeroing comes from M0-T21/T22.
- **M0-T76**: P0 edits.
- **M0-T77**: triggers at step boundaries, synchronous in headless runs.
- **M0-T84**: `poll()` already never blocks, by M0-T22's design.
- **M0-T85**: the view-model pattern TG declared.
- **M0-T98a, T99a**: tuning has no closed-list reason; "judgment-heavy" is not on the list.

## 10. The lead's answers — the question tool, one call of four, 2026-10-08
- Splits (M0-T33, M0-T44): "Split both (Recommended)"
- Lot 14's order: "Tune first, check after (Recommended)"
- Ratings (the 14): "Yes, those 14 (Recommended)"
- Housekeeping and check fixes: "Apply all (Recommended)"

## 11. Flags written (`plan.py flag … --from M0-TB`, same-day stamps: refresh 0/5)
- **M0-T5 and M0-T7**: build one eframe App that the web entry runs too, so M0-T94 stays an edit job.
- **M0-T13**: sr-app is bin-only (R11), so crates/sr-app/tests/strings.rs cannot `use` its modules.
  Either `#[path]`-include strings.rs or make the check a unit test.
- **M0-T98a and M0-T99a**: their one physics.json write makes the calibration stale until M0-V14. A
  claim-run red from G-CAL is the design working: name it and ask the lead before the claim run.
- **M0-TZ**: the TG defect (§4), to carry to M1's TG.

## 12. Not acted on, with the reason
- **Claim runs of the tuning blocks.** A scope that runs named presets and lists
  assets/physics.json in its `paths` would go red there by design. Which scopes do depends on paths
  the builders set; it is flagged (§11), not pre-ruled.
- **No new check for spec-only details**: T13's desktop title read from STRINGS, T57's summary.json
  `ledger` keys, T104's S1/S2 record. The lot V's contract-vs-code covers them.
- **Lead worklist item 2** (the switch plugin) is settled on linux-pc: `rung_record.py now` reads
  `plugin=installed`. The tracker says so; win-laptop is still owed when that machine joins.
