# M0-TG2 — M0-D71's held core (H) cut into BUILD blocks, its contract lines written first — task file

Session: linux-pc (laserax-ai), 2026-11-02; model=claude-opus-5-5 level=max (`rung_record.py now` on the launch prompt and on the
lead's prompt; = the heading's rung). No code; no GPU run. Raw: logs/M0-TG2.log (the plan.py calls, the path check, the claim).

## The fork, and the lead's answers (verbatim)
- Asked (question tool): where H's two constants live — in code like M0-R6's r_core = 4, or in physics.json, whose hash stales
  the shipped calibration (G-CAL: the desktop game, the web build and every preset run exit 6 until `calibrate` re-runs,
  ≈ 2 h 17 min on the RTX 4090, M0-V9's O7). Measured before asking: desktop.rs:1571 refuses a stale calibration at start;
  sr-app's UI tests unwrap the shipped one; reference/sinks.rs:54 holds `R_CORE = 4.0` as a code constant.
- The lead: «In code (Recommended)», then, typed: «but would it be more proper to put them in the physics json?»
- My answer, in short: yes — §2.7's table is physics.json, scheme constants (cfl, eta_g, sigma_vac) live there, and only
  physics.json is under the calibration's hash; the cost is timing only. Offered: build them in code now and move them into
  physics.json in the window M0-T98a's tuning already makes stale, so M0-V14's planned recalibration covers it.
- The lead: «Code now, move later (Recommended)» → M0-D71g, filed right after M0-T98a; M0-V14 flagged.

## The contract (each passage tagged [M0-TG2]; one §0.5 line; the mirror in m0_superseded.md)
§1.3.2 (P4 and P8 rows) · §1.4.2 (no work across a held wall; inflow does the work) · §2.3.4 (held cells, the reflecting wall,
held–held and held–edge faces, one-way inflow = item 2's starting form, the ∇·u note, the limits = items 3–5) · §2.7 (the hold:
test, item 1's N_Fe gate as the starting form with the set-ceiling form named next, release, zeroing, the initial mask, Clear,
the constants in code until M0-D71g) · §2.9 (`held`; old records and snapshots read it as 0) · §5.1 G-CONS (+ held) ·
§2.12.3 (`resume.held` + one plane, written only when a cell is held; absent = none held, so un-held dumps keep their bytes) ·
§5.1 G-REST (new; M0-R11's ruled bounds verbatim from the report's § Ruling; scope `held_core`, plant `held-wall-pressure-only`)
· §5.5 (G-REST's GPU part UNVERIFIED).
- Report item 4 measured, no fork: scenes/cold_co_disk.json's 2,000 rects peak at Σ 0.150 (python over the scene), far under
  σ_hold 10, so G-CORE (d)'s resolved disk holds no cell; written as a limit, not a criterion change.

## The cut (run order; every block (usual) = Opus 5.5, high · switch, rule (3)'s reason named in its Rating line)
| Block | Artifact | Failure classes | Files (verify.json, plants apart) | ≈ new lines | Needs |
|---|---|---|---|---|---|
| M0-TB2 (PLAN, gate) | §2.1's re-run on this cut | — | tasks/M0-TB2.md | — | this block |
| M0-D71 | the twin's hold (reference P8 + P4) + `held_core` scope | P8's rule · P4's walls/inflow | 6: reference/{mod,hydro,floors}.rs, tests/gpu/held_core.rs (new), tests/gpu/main.rs, testing.md | ≈ 470 (≈ 215 code, ≈ 230 tests) | — |
| M0-D71b | P8's mask on the GPU + P8's `held` booking | the f32 rule against the twin | 6: floors.wgsl, step/mod.rs, state.rs, tests/gpu/{floors,booking}.rs, testing.md | ≈ 250 | D71's twin |
| M0-D71c | P4's walls on the GPU + P4's `held` booking + G-REST's GPU disks | walls/no work · inflow | 6: hydro.wgsl, step/hydro.rs, state.rs, tests/gpu/{held_core,booking}.rs, testing.md | ≈ 350 | D71b's mask |
| M0-D71d | the ledger's `held` term | the momentum invariant (+ old-snapshot read) | 3: ledger.rs, tests/gpu/ledger.rs, testing.md | ≈ 120 | D71b–c's booking |
| M0-D71e | the mask in dumps | the round trip / resume | 4: dump.rs, headless.rs, tests/gpu/dump.rs, testing.md | ≈ 150 | D71b's mask |
| M0-D71f | the GPU lives + the crop case + reports/held_core.md + items 1–2 to the lead | the endings (items 1–2) · the window | 5: tests/gpu/held_core.rs, scenes/remnant_window.bin (new), reports/held_core.md (new), the contract's §0.5, testing.md | ≈ 150 + the report | all of the above |
| M0-D71g (after M0-T98a) | the constants into physics.json | one | 5: assets/physics.json, reference/floors.rs, step/mod.rs, tests/gpu/held_core.rs, the contract's §2.7 | ≈ 60 | D71, D71b; T98a's stale window |

TB's table on my own output (§2.1 step 2):
- (i) one artifact kind each: yes. (ii) ≤ 2 failure classes: yes. (iii) no context switch: yes — D71f's runs, case and report are
  one thread (H measured on the GPU's real states); the D79 restore it may trigger is filed apart, not built in it.
- (iv) cut by artifact dependency (twin → GPU P8 → GPU P4 → ledger → dumps → lives); no cut needs a ruling, except D71f's own
  question on items 1–2, which closes it.
- (v) paths: 67 named across the eight blocks, 53 present, 14 labelled new (logs/M0-TG2.log, the path check).
- (vi) one notch smaller — tried and kept: the twin's P8 and P4 split would leave no check able to show a held disk until both
  land (a producer/consumer pair); the GPU bookings split from their passes would leave zeroed momentum unbooked with no case
  to see it and push D71d to 9 files; D71c's inflow split would re-open the same kernel lines.
- Edit jobs: none — each block changes a schema, a format or more than 2 files. Named exceptions: none.
- Booking layout, for D71b–D71d: the held terms are appended at `TERMS`' end, so a snapshot's 37-term list keeps its order and
  `Ledger::restore` reads the missing terms as 0 (it refuses a length mismatch today, ledger.rs:263); tests/gpu/booking.rs pins
  the TERMS list exactly, so both GPU blocks name it in their Deliver.
- hydro.wgsl's `hydro_update` binds 8 storage buffers today (F «the update's eighth»), WebGPU's default that §2.2 applies to
  native builds too — D71b chooses the mask's binding for D71c.

## Deviations
- One plan line (M0-D71's Carried-flags pointer, after the flags were saved byte for byte from `git show HEAD:` for M0-D71f) was
  trimmed by a Python one-off instead of the file-edit tool — the instruction file names an ad-hoc script over the plan a defect.
  The result: one line changed (diffed), the file's ending kept, lint GO. Every other plan edit went through plan.py or Edit.
- Deliver items were re-cut once after lint: a top-level `;` reads as an item separator (measured on M0-D71e: its warning went
  when its one `;` did), so the new blocks use ` · ` items, each `→ Verify <n>` or `none — <why>`.

## Not acted on (measured or seen, with why)
- Header 601/600 after the 7 Flow rows — inside the 100-line margin, so no TC fires (§2.3); the next TC or V may compact.
- Five DONE blocks outside the current phase want stubs (M0-TC13, M0-D144–M0-D147) — a V's or TC's move, not a PLAN's.
- M0-TG2's own Read (the scratch diff whole) and Deliver items (9) warn — written by M0-D71; my Deliver is not mine to rewrite.
- Report items 6 (§2.8's bounce edge may draw the floor as a pressure-only wall did) and 7 (η tuned on one dump): NOT MEASURED,
  so not filed; item 7 is written into §2.7's text.
- A held core's ∇·u = 0 fails §2.6's converging test while held (arithmetic, not measured): written into §2.3.4 and D71f's
  Adversarial; D71f reads the 67,020 life's sink against it.

## Next
M0-TB2 (§2.1's re-run on this cut), then M0-D71.
