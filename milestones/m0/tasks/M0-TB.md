# M0-TB — the size, token and rating pass (2026-10-08, linux-pc)

Model and level: `model=claude-opus-5-5 level=max plugin=installed` (`rung_record.py now`, on the
launch prompt) — the heading's rung, no mismatch. The record: reports/size_pass.md. Raw runs:
logs/M0-TB.log, plus logs/M0-TB.plan_lint*.log written by verify.py.

## What exists now
- **reports/size_pass.md**: every open BUILD block through §2.1 (i)–(vi), 126 rows. Deliver paths
  grep-verified: 495 before (273 new · 222 edit), 510 after (278 · 232). Also: the splits, the token
  half, the check audit, the rating with each rule (3) reason, and the lead's answers verbatim.
- **The plan**, changed only as the lead approved: one question call of four, every answer "…
  (Recommended)".
  - Splits S1–S4:
    - M0-T33 → T33a (Riemann oracle; scope `riemann`, plant `riemann-gamma`) + T33b.
    - M0-T44 → T44a (the rate law; scope `rates`, plant `rates-table-shift`) + T44b.
    - M0-T98 → T98a (tuning) + T98b (G-SQUEEZE).
    - M0-T99 → T99a (tuning) + T99b (G-FIT).
    - M0-T100 and T101 moved to lot 15.
    - Follow-on edits: T97, T102, V14, T108, V15, the Phase 17 heading with its E8 ordering note,
      and Phase 17's commit message.
  - 14 headings rated up to Opus 5.5, high: M0-T3, T5, T18, T19, T20, T21, T22, T26, T30, T34, T42,
    T46, T51, T56.
  - Check fixes: T5, T23, T25, T37, T40, T49, T80, T81, T85, T87, T88, T89, T90, T91, T92, T93b.
  - Rules: R10–R14 index lines; the E count and gates to builds (26:132); a pointer to the REJECTED
    list.
  - Header: the answered Resolve list became a pointer; Repo facts' loop time became a pointer.
  - Register: `Model ratings: 2026-10-08 by M0-TB`.
- **m0_rules.md**: R10–R14, the Build preamble moved byte for byte (20 lines, diffed). Also "Considered
  and REJECTED (false economy)", 10 items.
- **plan_archive.md** (new, written by `plan.py stub`): M0-TI, TP, R1, D1, R2a, R2b and R3, 263 lines,
  each verified byte for byte.
- **milestones.md**: the M0 row's next step, worklist item 2 settled on linux-pc, and the M0-TB delta.

## How the paths were verified
- **By hand first.** I read each Deliver and listed the files the block writes: N creates, E edits.
  Manifest `paths` globs, run outputs and prose mentions were left out.
- **Then by machine.** Read-only scratch tools in the session scratchpad, never in the repo: blocks.py,
  paths.py, paths2.py, canon.py and prov.py, under
  /tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/8d40e67c-0c49-47d0-9643-7debfaeacacd/scratchpad/tb/.
  - The parser drafted each block's list as a checklist.
  - prov.py walked the plan in run order against `git ls-files` and the disk.
  - It also checked that every `verify.py <scope>` a Verify line calls is created at or before its
    block.
- **Before the edits**: 3 findings (the `(new)` labels, fixed) and 0 scopes used before they exist.
- **After the edits**: 0 findings and 0 scopes used early; 89 scopes in all.
- The contract's §5 names 38 scopes and 38 plants, all placed in the plan.

## Decisions taken without a question (cheap to reverse, declared)
- **Splits rename the parent's id to the a/b halves.** No N/A stub is kept for the parent. This is
  the playbook's own convention (annex §A.1's fixture plan: T2a/T2b, no T2) and M0-TG's (T52a/b,
  T93a/b).
- **(vi) is read on `Deliver:` items.** A block splits when an item with its own check can leave
  without a ruling dependency and the parent sits near the ceiling, or when a check needs an artifact
  only a later step makes. Sub-features inside one file never trigger a split. Considered and kept:
  §4.1 of the report.
- **The lot-15 order is T103–T107, then T98b, T99b, T100, T101.** All of them depend only on the
  recalibration. `plan.py` inserts only after a block, and the first slot after V14 crosses its
  commit-gate trailer and the phase heading.
- **The capture-action check lives in one script**, tests/capture/actions.json, run as the `capture`
  scope's `actions` case. It is created at M0-T85 and appended by later blocks. In M0-T91 (an E
  block) the one-line step counts as a registration, so the block stays at 2 files.

## Deviations
- **Edit-tool changes.** Block renames, rating changes and heading or Flow-row order fixes were made
  with the Edit tool: plan.py has no split, rename or rate call (Repo facts' call list). The
  appends, moves, stubs, register line and flags went through plan.py.
- **Lint round.** My first T98a/T99a wording made the lint warn: it read a `;` after
  `[NOT RUN — for you]` as an item boundary, so one Deliver item had no Verify mapping. I found the
  cause by linting variants of a scratch copy, then rephrased with commas. The lint is now 0 warnings.

## Measured and not acted on
- **Tuning claim runs**: a red from G-CAL is by design; flagged to M0-T98a/T99a (ask the lead before
  the claim run).
- **No new checks for small spec details**: M0-T13's desktop title read from STRINGS, M0-T57's
  summary.json `ledger` keys, M0-T104's S1/S2 record. Each lot V's contract-vs-code covers them.
- **M0-T85's `pause`** already exists since M0-T8. Its Deliver now says it is routed through the bar,
  not added twice.

## The routine, on linux-pc (box laserax-ai)
- **Verify 1** `plan.py lint`: [ALREADY RUN — PASS (GO, 474 checks, 0 warnings) on linux-pc].
  Baseline before edits: GO, 470 checks, 7 stub warnings.
- **`verify.py plan_lint --task M0-TB`**: [ALREADY RUN — PASS (474 passed, 1/1 scope) on linux-pc].
- **`verify.py --redarm plan_lint --task M0-TB`**: [ALREADY RUN — PASS (clean GO, plant
  plan-bad-status red) on linux-pc].
- **Claim run** `verify.py --changed --base a5065fb --task M0-TB`: [ALREADY RUN — PASS (3/3 scopes,
  487 cases, 3.28 s) on linux-pc]. Its FLAG lines name only M0-TH's uncommitted toolkit and toolchain
  files since a5065fb, which the Phase 2 commit gate carries.

## After the close — the lead's follow-up (2026-10-08, same session)
- The lead, verbatim: "Yes you can stub them, and then can you write a task after TB for me. Next time I open
  the plan I will be using windows, so id like for an agent to run a task to check that everything is good
  and to install anything missing so we can continue smoothly on the windows laptop."
- `rung_record.py now` on that prompt: `model=claude-opus-5-5 level=max plugin=installed` — no mismatch.
- **Stubbed** M0-TC, TH, TG (96 lines) and, after its handoff named the follow-up, M0-TB (29 lines) →
  plan_archive.md, each verified byte for byte. The Phase 2 commit gate stays under M0-TB's stub.
- **Filed as Phase 2b**, between the Phase 2 commit gate and lot 1, with its own commit gate and an E8
  ordering note:
  - **M0-TE-win** (BUILD, Sonnet 5.5 high — rule (3), no closed-list reason). It probes the box,
    installs from the approved table, makes `python3` answer and proves the toolkit and toolchain
    there (`--all`, the toolchain red-arm). It also maps lot 1 against the laptop:
    reports/win_laptop.md.
  - **M0-TJ3** (PLAN + LEAD answers, Opus 5.5 max — rule (2)). It takes R5's change to the lead from
    that map and applies §2.6.
- **Why two blocks.** The setup cannot rewrite the plan, and lot 1's checks need a ruling.
  M0-T2's Pass names linux-pc's GPUs; M0-T5, M0-T8 and the TVs use Xvfb with lavapipe; M0-T6 builds
  start.sh only; M0-V1 has the lead double-click start.sh. Two kinds, two blocks (§2.1 (i), (iii)).
- **Placement mechanics.**
  - Filing needed a `TE` counter, added to the Counters line through `plan.py register`.
  - `plan.py append --after M0-TB` would put the block above TB's commit-gate trailer, so I filed after
    M0-T1 and moved M0-T1 below both, all rehearsed on a scratch copy first.
  - By hand: two Order fields, M0-T1's heading, the Phase 2b heading and gate, and the "# Build"
    heading moved below Phase 2b (it names M0-TG's lots).
- **Not in the approved table, so the agent asks the lead on win-laptop** (§4 Rule 2): the MSVC C++
  build tools (the msvc target's linker), and rsync for scratch_copy.sh's selftest. An install needing
  an administrator prompt is the lead's command (R3).
- **Runs**: lint GO (479 checks, 0 warnings); `verify.py plan_lint --task M0-TB` GO after the filing (479
  passed), on linux-pc.
