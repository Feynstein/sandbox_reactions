# M0-TJ2 — task notes (2026-10-18, win-laptop Laser2025-20)

The record is reports/top_speed.md (the questions as asked, the lead's answers verbatim, the numbers, the surfaces
changed, § Routed, the negative scope). This file holds what the session ran and edited.

## Start
- `grep -n "^## M0-TJ2 "` → line 1715, `· switch` after the rating (Opus 5.5, max).
- `sleep 2; python3 tools/pb/rung_record.py now` → `model=claude-opus-5-5 level=max plugin=installed` — the heading's
  rung, no MISMATCH.
- `python3 tools/pb/plan.py show M0-TJ2` → `baseline: M0-TJ2, 439 files, 2026-10-18T11:25:19-04:00`.
- Deviation: those two calls ran as `python3` (the hook's words); on win-laptop R15 reads `py -3.12`, which every later
  toolkit call used. Both read and wrote nothing a 3.12 run would differ on (the baseline is the tool's).

## Runs (logs/M0-TJ2.log, RTX 4080 Laptop — context for the question; the Quadro's numbers stay laserax-ai's, R15)
1. `cargo build --release` — nothing to rebuild (binary of 11:12, after M0-D56's shader).
2. `headless --scene preset:sun --steps 5000 --dump-every 1000 --timing --adapter "RTX 4080" --out build/m0-tj2-sun`
   — first try exit 127 before the binary ran (`/usr/bin/time` is absent in Git Bash), then exit 0, sim 59.025,
   158.2 s; ignition step 769, `main_sequence` from 3,000; the dumps' `resume.box` and `resume.dt` read by a one-off
   `py -3.12 -I -c` header read (logged).
3. `headless --scene preset:sun --load-dump build/m0-tj2-sun/dumps/s_5000.bin --steps 200 --timing --adapter "RTX 4080"
   --out build/m0-tj2-at5000-<i>`, i = 1..3 → 17.392 · 17.377 · 17.621 ms a step.
   The outputs stay in git-ignored build/ (≈ 70 MB of dumps; win-laptop's disk is 97 % full — delete
   build/m0-tj2-* when not needed).

## The question
One call of the question tool, three questions (top speed; life length; step cost), the recommendation first —
answers verbatim in reports/top_speed.md § Ruling. Q1 and Q2 answered by typed text, Q3 by the recommended option.

## Edits
- **Contract** (m0_contrat.md): §0.2 (Q2's entry); §0.5, three [M0-TJ2] lines (the top speed and the stop; G-FPS's
  bounds; the cadence); in place, tagged: §1.4.4, §1.8.2, §1.8.3, §1.8.4, §1.8.6 (FROZEN beside the lead's words), §3.2,
  §3.3 (bench `--rung <label|max>`, `off_cap`), §4.2, §4.3 (the two keys, for M0-T125), §4.10 (key 0), §4.12, §5.3
  (G-WARP, G-CAD, G-LATCH, G-FPS, G-TOP), §5.5. §4.1's `strings` block untouched: the `strings` scope reads the contract,
  and a key without strings.rs would turn it red — M0-T125 adds both in one edit.
- **Plan**: the goal paragraph; Superseded, three mirror lines; this block's heading and Flow row (AFTER M0-D56,
  "(deferred)" dropped — the stale pre-wake marks M0-V18 named); M0-T98a (Read, Deliver, Verify, Adversarial: the
  shortest life); M0-T85, M0-T75, M0-TV2, M0-T109, M0-T110, M0-T111 (TOP → Max); M0-T112 (G-FPS), M0-T113 (G-TOP: heading,
  Flow row, block); M0-T114 N/A (`plan.py status`); M0-T115, M0-V16, M0-V17; M0-D51's and M0-T110's AFTER (to M0-T125,
  M0-T124). Filed by `plan.py append`: M0-T123 and M0-T125 after this block, M0-T124 after M0-T109. `plan.py flag
  M0-TZ` (§ Routed).
- Hand edits of other blocks' bodies: re-scopes by this PLAN block (PLAYBOOK §0), each tagged [M0-TJ2]; plan.py has no
  call for a block's body. Ratings: M0-T112, M0-T113 and M0-T98a re-read under rule (3) — no reason from its list
  added, their rungs kept; M0-T123 and M0-T125 rule (3)'s Sonnet 5.5, high; M0-T124 `(usual)`, its reasons in its
  block.

## Sizing of the filed blocks
- M0-T123: one artifact (the time-control module) — time.rs with its tests, one plant, verify.json's entry, its page's
  two sections and its testing row.
- M0-T125: the string table — two keys on two files; split from M0-T123 by PLAYBOOK §2.1 (i) and (vi): no ruling
  dependency between them, and §4.1's block cannot gain a key before strings.rs does.
- M0-T124: a measurement with one cut when it fits ≈ 6 files; a bigger cut is its split, named in its handoff.
