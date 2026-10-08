# M0-TP — detail (2026-10-08, 09:12–09:40 America/Toronto, linux-pc)

- **Start read.** The grep of the heading (`· switch` present), then `python3
  tools/pb/rung_record.py now` and `python3 tools/pb/plan.py show M0-TP` — both NOT RUN (no
  tools/pb/ yet; M0-TH extracts it). The plan read whole with the file tool, as the `Read:` line
  says, then reports/bootstrap.md and reports/sandbox_interview.md; PLAYBOOK §0, §1, §2, §3, §4,
  §5, §8–§14.1, §B and annex §A.1's lint list (its prose, not its code) by line range.
- **Model and level.** NOT RUN (no rung_record.py yet). The §B.3 probe read CLAUDE_EFFORT=max — a
  level variable, not a measurement of the running request; the model is unmeasured.
- **What ran** (logs/M0-TP.log, all on linux-pc): read-only probes of the box, the GPUs, OpenGL,
  Vulkan, the toolchains and dev libraries, the user-level Rust and Godot installs, the §B.3
  agent probe, the switch plugin's record, read-only git (`log`, `diff`, `status`, `ls-files`,
  `branch -vv`); a byte comparison of AGENTS.md, CLAUDE.md and GEMINI.md against PLAYBOOK §C;
  the red arm of the four new research `Verify:` lines on synthetic reports in the session's
  scratch folder (18 cases: 4 GO, 14 planted faults NO-GO) and a byte check that the plan's
  copies equal the tested ones; a read-only structural check of the plan against the lint rules
  annex §A.1 documents (12 blocks = 12 Flow rows, header 167 / 600, Pipeline state 8 / 60).
- **The questions.** Two calls of the question tool, seven questions (four, then three), each
  restating its consequence, the recommendation first; all seven answered with the recommended
  option. Verbatim in reports/plan_redteam.md §8.
- **Plan edits, by the file tool** (no plan tool yet): the goal paragraph; "Resolve with the lead
  first" items 1–3 ruled; Flow rows (M0-R2 → M0-R2a, M0-R2b; M0-R3's goal and order; this block);
  Rules (the OPT line, the gates line 7:5, "sound" dropped, R6 and R7 index lines); Repo facts
  (Repo, Toolchains); Hazards (the `.gitkeep` line re-worded, the Rust PATH hazard added); M0-R1,
  M0-R2a, M0-R2b, M0-R3 re-scoped with tightened `Verify:` lines; M0-TC's `Read:` and `Deliver:`
  for the split, plus two flags; one flag into M0-TG; M0-TW's and M0-TB's `Deliver:`; this
  block's Status and Handoff; the Pipeline state. m0_rules.md: R6, R7, R8's why.
- **Ratings filed.** M0-R2a and M0-R2b: Opus 5.5, high · switch — rule (5), R2's own reason from
  rule (3)'s list (numeric or precision semantics: units, scales, step limits). No other rating
  changed; the register's `Model ratings:` line stays the bootstrap's (no pass rated the plan).
- **Flags.** M0-TC ← the 60-frames target needs a timed check (RT16); the ending readout is a
  prediction to guarantee (RT17). M0-TG ← measure one step's cost early (RT6). Both PLAN blocks:
  no rating-refresh count moves.
- **Tracker.** The M0 row ("TODO — next: M0-TI", stale) fixed and a dated delta added, as §1 asks
  every gate.
- **Found as is, left alone.** The tracker's and the plan's uncommitted edits from the bootstrap
  hand-over and M0-TI (git shows them modified) — kept; the Phase 0 commit gate carries them.
- **Condensed.** The log's raw listing of ~/.local/bin, /opt and the full PATH named unrelated
  software on the lead's machine; it is replaced by the three facts the findings use, said so in
  the log.
- **Not acted on** (reasons in the report): a research red-team `TA` (RT20) and a split of M0-TC
  (RT19) — §2.6 P6; TI's flag text in M0-TC still says "M0-R2" for B32 — another task's stamped
  flag, left as written: M0-R2b now owns gravity's 2D law.
- **Next.** The lead's Phase 0 commit gate (the plan's block after M0-TP), then M0-R1.
