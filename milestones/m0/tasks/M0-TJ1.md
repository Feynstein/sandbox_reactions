# M0-TJ1 — task notes (2026-11-05, linux-pc laserax-ai)

Not woken. No §1.6.4 trigger has fired on the record, so the block's ruling arm did not run: no Q3 question, no
reports/standins.md, no amendment, no block filed. This file holds what the session read, probed and asked, and where
S1's reading went (the lead's word: M0-V14, on the tuned lives).

## Start
- `grep -n "^## M0-TJ1 "` → line 3506, `· switch` after the rating (Opus 5.5, max).
- `sleep 2; python3 tools/pb/rung_record.py now` → `model=claude-opus-5-5 level=max plugin=installed` — the heading's rung,
  no MISMATCH.
- `python3 tools/pb/plan.py show M0-TJ1` → `baseline: M0-TJ1, 824 files, 2026-11-05T10:35:43-05:00`.
- Read: the block; contract §1.6.4, §2.4.2, §2.5.4, §2.7, and §1.9.2's envelope (the mass outside 2 R_core, which
  §1.6.4's S1 trigger names); reports/lives.md; reports/contract_rulings.md Q3. The heads of M0-V17 (the block names it
  for its N/A arm) and M0-V14 (its Deliver reads «the stand-ins' state against M0-TJ1» — the flag's target).

## The wake check
| Stand-in | Trigger (§1.6.4) | The record (reports/lives.md § Stand-ins, M0-T97, RTX 4090) | Fired? |
|---|---|---|---|
| S2 · thermal bomb | the Massive forms neutron matter and, within 20 t.u., unbinds < 50 % of its envelope at f_dep = 0.1 | 74.9–75.1 % against the final remnant (98.5–98.7 % against the object's mass then): a preset-free disk at f_dep 0.1 (the preset refuses a changed physics.json), step 5,000, t 28.95 | no — measured off |
| S1 · dust wind | the Sun-like, 20 τ_dyn after entering he_shell, has unbound < 30 % of its envelope | NOT RUN — the Sun ran to t 800 only (mid main sequence); its full life, C4b, is over the 10-minute line, the lead's | not measured |

S2's window, my reading: neutron matter first shows between t 7.23 and 7.71, so the window closes by t 27.7; the 0.1
run's dumps sit at t 28.95 and 30.00, 1.2–2.8 t.u. past it, both 74.9–75.1 %, and the preset's share moved 0.3 points
over t 27.1–29.3 (79.0 → 79.3 %). The share at the window's close is ≈ 75 %, 25 points over the trigger; the dumps'
offset cannot carry it under 50 %.

## Probes (read-only)
- laserax-ai-server `~/workspace/sandbox_reactions/build/t97/`: giant, giant_d, massive, massive_d, p1_*, s2_0.05,
  s2_0.1, sun_ms, newest 2026-11-05 10:25; **no sun_life** → C4b NOT RUN. RTX 4090 0 %, 20 MiB; RTX 3060 0 %, 13 MiB.
- `git diff --stat acaca48 fbcbadf` outside milestones/: 21 files, M0-D109's and M0-D157's edits — the uncommitted tree
  M0-T97 measured and rsync'd. The server's tree is HEAD's code: a C4b now would have read lives.md's tree.
- Not opened before these two probes ran: logs/M0-TJ1.log records them from the session's output (a deviation from
  the open-the-log-first rule; both read-only, neither crashed).

## The question
One call of the question tool, one question («S1 reading»): S2 measured off, S1 never measured — the Sun's full life
(C4b; V9's 762 s on an older build, M0-T97's 5.6 ms a step over V9's 177,762 steps ≈ 1,000 s — my arithmetic) is the
lead's and has not run; when should S1's trigger be read? (a) At M0-V14, after tuning (Recommended): TJ1 stays
DEFERRED, M0-V14 flagged to read S1 and re-read S2 on the tuned lives — M0-T98a, M0-T131 and M0-T99a retune the clocks,
the collapse and the mass ladder first, which reshapes the Sun's late life; (b) now: the lead runs C4b with denser
dumps, S1 read in this session. **The lead: «At M0-V14, after tuning (Recommended)».**

## Edits
- `plan.py flag M0-V14 … --from M0-TJ1` — S1 read on the tuned Sun-like's full life, its dumps dense enough for the
  20 τ_dyn window (C4b's 20,000-step cadence ≈ 228 t.u. on the main sequence, τ_dyn ≈ 1 there — my arithmetic from
  lives.md's 70,100 steps to t 800.011); no he_shell displayed → a question to the lead, never a pass; S2 re-read on the
  tuned Massive (C6's two runs); a fired trigger wakes M0-TJ1, none → DEFERRED to M0-V17. The full text is in M0-V14's
  `Carried flags:`.
- `plan.py close M0-TJ1` — status DEFERRED, its wake naming M0-V14's tuned lives beside reports/lives.md (a deviation:
  the wake's source widened to where the lead put the reading); the handoff; the register's `Next task`: `M0-T97 on`
  (stale since M0-T97's close) → `M0-TM15 on`, and M0-V14's carry list gains M0-TJ1's stand-in reads.
- Not edited: reports/lives.md (M0-T97's), the contract, m0_superseded.md, physics.json.

## Measured, not acted on
- C4b's other readings — the Sun-like after t 800 (its stages, its ending, G-FIT's envelope stretch) — stay owed by
  reports/lives.md as `[NOT RUN — for you]`; the lead's ruling moved S1's read only, and lives.md's readers (M0-T98a,
  M0-T99a, M0-V14) carry the rest.
- The flag printed `refresh=5/5`: the count M0-TM15 (filed by M0-T97, «five carried flags on open BUILD blocks») already
  answers; M0-V14 is a CHECK block, so this flag left the count at 5.
- The lint reads `header=615/600` with five stub warnings (M0-TC13, M0-D144–D147), exactly as at M0-T97's claim
  (logs/M0-T97-claim.plan_lint.log) — not moved by this task (the `Next task` edit replaced one line). The lint demands
  no TC (GO, no WARN on the header), so none is filed here; M0-V14, Phase 16's close, checks the budget and stubs.

## Claim
[ALREADY RUN — PASS (GO: 827 passed, 0 failed, plan_lint 1/1 scopes, 87 not impacted — no GPU scope, so R18 does not move it) on linux-pc (laserax-ai)]
```
python3 tools/pb/verify.py --changed --task M0-TJ1
```
Base: M0-TJ1's start (2026-11-05T10:35:43-05:00); tree fbcbadfa31e3+b803528f02a7; logs/M0-TJ1.plan_lint.log. The handoff's
claim line was written after this run (`plan.py handoff`), and `plan.py lint` re-read the plan once after it
(logs/M0-TJ1.log).
