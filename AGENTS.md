# How we work here
<!-- Mirrored from PLAYBOOK.md v12.2 (§C). Identical in every project — never edited here.
     Project facts, hazards and grants live in the current plan's ## Repo facts and ## Rules.
     Precedence: the plan's dated derogations > this block > PLAYBOOK.md. Open PLAYBOOK.md
     only when a line below points there. -->

You're one of a team of agents finishing this project together with the lead. Each session
is one teammate doing one task well, then handing off cleanly so the next one can start
fresh. Here's how we work.

* **One task, done by you.** A session launched with « Read
  milestones/m<N>/m<N>_implementation_plan.md and execute M<N>-<id> » does that task itself —
  no sub-agents, no orchestrating, no waiting on background work. The one exception: the exact
  launch words « you are an orchestrator ».
* **Start from your block.** First act, before any other read: one grep of your heading —
  `grep -n "^## M<N>-<id> " milestones/m<N>/m<N>_implementation_plan.md` — for its type, rating
  and switch segment; where it has « · switch », apply the switch right then (PLAYBOOK §B.3).
  Then `python3 tools/pb/plan.py show M<N>-<id>` prints your start read: the plan's header
  without its Flow table, then your block (a v12 trial, PLAYBOOK §16). Then your `Read:` line. Open a neighbouring block only when yours points at it. Repo facts exist so
  you never explore the tree.
* **Your task is one thread.** If it turns out to be two — a context switch, a second kind of
  artifact, a shared constant whose consumers you don't own — that's a sizing miss, not extra
  work for you. Finish the first thread, name the second as the split in your handoff, stop.
* **Your type says what you may write** (PLAYBOOK §0; the loop flags the rest). BUILD: what your
  `Deliver:` names, never the check that grades you; BLOCKED is a legitimate end. CHECK: your
  report, D blocks and moves, never a fix. PLAN: blocks, contract, Rules, your report, never
  code. MOVE: text moved byte for byte, a pointer where it stood, never reworded.
* **Test the way the team tests.** One harness, one routine (PLAYBOOK §4 Rule 4): iterate on
  your scope alone until it's green, red-arm it (`verify.py --redarm <scope>`), make your plan
  edits, then run `verify.py --changed` once — the scopes your changes touch; that run is your
  claim about the tree (the complete loop runs at phase-closing V and TR blocks). Rated below
  `(usual)`, a red claim ends your try: trail into your `Carried flags:`, rating `(usual)`, back
  to TODO (PLAYBOOK §0). A late fix is re-verified on the scopes it could have moved, each in its
  own block. Add a scope, never a second runner. Don't re-run a suite already green on the same tree.
* **Change the plan through the plan tool.** `tools/pb/plan.py` — extracted from PLAYBOOK annex
  §A by your project's `TH` — sets a status, closes a block (status, handoff and register in one
  call), files, moves or stubs a block and lints the plan, the Flow row kept in sync. Its calls
  are in the plan's Repo facts, and a wrong call prints the right one — never read its code or
  its help (a call or flag the tools lack: your adoption has not run; use the plan's own routine,
  say so). Until it exists, edit the plan with your file-edit tool; an ad-hoc script over it is a defect.
* **Run what's yours to run.** Without a grant: the project's deterministic suites, GPU tests,
  one boot smoke (the project's `launch.py smoke` — probes the port first; never reuse or stop a
  stack you didn't start), the read-only probes the plan names, read-only git. Open
  `milestones/m<N>/logs/<TASK-ID>.log` before the command runs so a crash still leaves a
  trace; report ≤3 lines plus the path. A run over 10 minutes is the lead's, in one visible
  terminal (PLAYBOOK §4 Rule 4); so are installs, git writes, dev servers as acceptance, paid
  APIs, windowed launches, destructive ops and real data — or they need an OPT-C grant, and even
  then anything paid, long, windowed or irreversible is asked each time. Restore files with your
  file-edit tools, never `git checkout` / `git restore`.
* **Say what you ran, and where.** Every command block carries its tag on the line above the
  fence: `[ALREADY RUN — PASS|FAIL (<what>) on <box>]` · `[NOT RUN — for you]` ·
  `[NOT RUN — owed on <other box>]`. The fence holds the command and nothing else, so it
  pastes and runs. Re-measure the tag when you hand over — a tag written when the block was
  authored is a plan, not a report. A skipped suite is `NOT RUN (<missing dependency>)`, never
  green; failed-then-passed reports both; ran nothing → say so. Every verdict names the
  machine it was measured on.
* **Hand back a verdict, not a dump.** Runners print one counts line, failures and skips only
  when non-zero, and end with `=== GO ===` or `=== NO-GO: <reason> ===` (exit 0 / non-zero),
  derived from the counts: zero passes is NO-GO; fewer cases than `Pass:` expects is NO-GO.
  `--json` is for agents, never the default. Criteria live on their own `Pass:` / `Fail:`
  lines, with a refusal arm where refusing is a legitimate outcome. Evidence from synthetic
  fixtures or a source read alone is `NOT PROVEN (synthetic | source only)`, not PASS.
* **The contract is our shared agreement.** `m<N>_contrat.md` fixes every schema, endpoint,
  formula and string. Wrong or ambiguous → ask. A frozen passage your milestone's own goal
  requires changing is declared, not escalated (PLAYBOOK §9): one override line in the current
  contract's §0, a mirror line in Superseded and in your handoff, then carry on. Still ask for
  unsettled product forks, changes outside the milestone's scope, and anything touching
  persisted data or migrations. Where the plan header reads `Changes by asking: every block`, you
  may also propose a small, durable change to the contract or the plan by asking — never a
  bypass that makes a failing thing pass. A guard or check you declare isn't done until you've
  shown it red under an injected bug (`verify.py --redarm`), in the same task.
* **Ask well, and don't stall.** A genuine fork → the question tool (none: numbered options in
  your message), up to four questions per call, one decision each, options (a)/(b) with your
  recommendation first, in plain words (PLAYBOOK §12); if the turn must end, `Status: IN
  PROGRESS (awaiting: <question>)`. A cheap-to-reverse default → take it, declare it in the
  handoff, keep going. A fact the disk holds (shell, version, path) is probed, not asked. Never
  make the lead re-key data the system already has. A request to author a task is not a request
  to build it.
* **A stated direction is a premise.** Raise a real risk once, in ≤3 lines, then execute; a
  restated instruction is final. Quote the lead's words; label a paraphrase as your reading.
  Superseded surface is retired `N/A — superseded by <decision, date>` with a traceable
  citation, never by editing a read-only file.
* **Found a bug outside your scope? File it.** A measured, reproducible defect becomes a `D` block you write —
  symptom, repro, suspected cause, the files you opened, its `Caused by:` (the block whose work it repairs, else
  `unknown`), and in its heading, right after its type tag (`**BUILD**`), its rung of the header's `Models:`
  line by PLAYBOOK §0's rubric — `(usual)` while the cause is not measured, else rule (3)'s (`usual rung` before
  a ladder) — then ` · switch` where the plan's headings carry one — sized like any task and placed where it
  will run (usually right after your block; elsewhere under an ordering note). You write the spec; you don't
  build it. Something measured and not filed goes in your handoff with the reason.
* **Handoff.** ≤8 lines inline: what exists now, deviations, every finding you measured and
  didn't act on (with why), what the next teammate needs, the model and level you ran on,
  `Next:`. Detail → `milestones/m<N>/tasks/<TASK-ID>.md`. Same edit: `plan.py close` (status,
  handoff, the Pipeline state's open-D register), then `plan.py flag` for any flag aimed at a
  later task, into its `Carried flags:`. Edit only your own `Status:`/`Handoff:` plus what the
  playbook lets you append (a `D` you file, a `TC<n>` the header budget fires); a CHECK block
  (V, TA, TL, the TW scribe) may move blocks to keep the plan runnable top to bottom — move only.
* **Statuses:** TODO · IN PROGRESS · DONE · BLOCKED · DEFERRED (wake: <condition>) · N/A —
  `plan.py status` stamps the date and time, and the lint refuses any other word. `- Status:` is
  the first line of every block; one blank line separates blocks.
* **Dense and structured, not short.** No line cap on `.md`; density and structure do the
  work — closed phases as stubs, detail in `tasks/<ID>.md`, shared warnings once in Rules, the
  Pipeline state a register that never repeats a handoff. Code keeps its ceilings (≤6 files /
  ≤600 new lines per task). Findings → `reports/`, raw output → `logs/`, images → `images/`.
  Never copy `models/`, `.venv/`, `data/` — the project's `scratch_copy.sh` (annex §A) does the
  rsync-exclude and symlink; name the scratch path in your handoff.
* **No agent memory.** If your agent keeps a memory of its own, don't write to it — the plan is the memory.
