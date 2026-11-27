# Toolkit local diff — the project's tools/pb/ against PLAYBOOK v12.3's annex §A (M0-TH-adopt-a, 2026-10-11)

Box: win-laptop (Laser2025-20), Windows 11 Pro 26300, Python 3.12.10 (`py -3.12`), Node 26.8.2, git 2.56.0.windows.2
(Repo facts say 2.55.0; `core.autocrlf=true`). Written before the swap (PLAYBOOK §14.3 TH-adopt); § After the swap added after.

## How it was read
- v12.3's annex: §B.4's one line over the project's PLAYBOOK.md (v12.3) into a scratch folder — `extracted: 10 - refused: 0`, GO.
- v12.2's annex: the same line over `git show b1da721:PLAYBOOK.md` (the last v12.2 copy, read-only git) — 10 extracted, 0 refused.
- Three diffs per tool: project ↔ v12.2 annex (the project's local fixes), project ↔ v12.3 annex (what the swap
  changes), v12.2 annex ↔ v12.3 annex (what the bump changed). Raw `diff -u` of the project against v12.3, seven tools:
  logs/M0-TH-adopt-a.toolkit_diff.log.

## Per tool
| Tool | project = v12.2 annex? | project ↔ v12.3 (diff lines) | Named changed by §16 v12.3 | Action |
|---|---|---|---|---|
| plan.py | yes | 187 | yes | swapped; v12.2's Windows selftest fix re-added (below) |
| verify.py | no — M0-D16's `PB_PYTHON` (20 lines) | 2541 | yes | swapped; `PB_PYTHON` and v12.2's lost fixes re-added (below, 1–1c) |
| content_gate.py | yes | 154 | yes | swapped; v12.2's two Windows selftest fixes re-added (below) |
| rung_record.py | yes | 225 | yes | swapped; v12.2's Windows selftest fix re-added (below) |
| switch.mjs | yes | 122 | yes (12.3.1) | swapped as is |
| scratch_copy.sh | yes | 9 | **no** | kept byte for byte — v12.3's annex drops v12.2's `canon()` (Git Bash 8.3 names, case, /tmp) |
| status_page.py | yes | 13 | **no** | kept byte for byte — v12.3's annex drops v12.2's `drain()` (WinError 10053 on a refused body) |
| launch.py, review_page.py, capture_web.mjs | yes | 0 | no | untouched (identical) |

No tool is new to the project: no `tools_selftest` case added, verify.json untouched.

## Fixes the v12.3 annex lacks — re-added after the swap, flagged for M0-TZ
1. **verify.py — `PB_PYTHON` (local, M0-D16):** `case_env` sets `PB_PYTHON=sys.executable` (a caller's value wins), so a
   non-Python case (capture_web.mjs reads `process.env.PB_PYTHON`) never spawns the PATH's first `python` (win-laptop: a
   stdlib-less 3.14 ahead of 3.12). Hunks: the `case_env` docstring and `env.setdefault("PB_PYTHON", sys.executable)`; the
   selftest stub's `pbpy` kind; `PB_PYTHON` in the selftest's scrubbed-variable tuple and its `os.environ.pop`; the check
   "a case gets PB_PYTHON …" and its plant "an env without PB_PYTHON reads red". The v12.3 annex has no `PB_PYTHON` at
   all, while its own capture_web.mjs reads it.
2. **plan.py — selftest `bigname` (v12.2 annex, reverted by v12.3):** the `Read:` cap fixture names
   `os.path.basename(big)` — a native absolute path reads `C:\…` on Windows and the lint does not parse it. Without it:
   `FAIL: WARN: a Read: naming a file over 500 lines whole`, NO-GO 1 on this box.
3. **content_gate.py — selftest (v12.2 annex, reverted by v12.3):** (a) the `child()` helper's stdout
   `.replace("\r\n", "\n")` (Windows' text stdout); (b) `os.chdir(home)` before the temp folder's cleanup (Windows
   removes no folder that is the cwd). Without them: the denylist-beside-the-tool check FAILs and the selftest stops on
   `PermissionError: [WinError 32] … \plain`, NO-GO 2.
4. **rung_record.py — selftest `host` (v12.2 annex, reverted by v12.3):** the expected `now` install line is
   PowerShell's (`; & "…"`) when `windows()`, POSIX elsewhere — v12.3's selftest expects POSIX always, while its tool
   still prints PowerShell on Windows. Without it: 4 `now:` checks FAIL, NO-GO 4.

## The annex builds' selftests on this box, before the swap (scratch folder)
| Tool | v12.3 annex | project (v12.2 build) |
|---|---|---|
| plan.py | NO-GO 1 (fix 2) | GO |
| content_gate.py | NO-GO 2 (fix 3) | GO |
| rung_record.py | NO-GO 4 (fix 4) | GO |
| switch.mjs | GO | — |
| verify.py | NO-GO 3, 340 checks, 153/154 plants red, 146 s | (GO at M0-V2) |

verify.py's three reds are in v12.3's new code, not a reverted fix — filed as a D, not fixed here:
- "the walk lists what the .gitignore files leave …": `['trail']` — the fixture's file `trail ` (trailing space) is
  written as `trail` by Windows (Win32 strips a trailing space), so the walk lists a file the rule `trail\ ` never names.
- "the walk equals `git ls-files -co --exclude-standard` …": `['sub2/a.crlf']` — git 2.55 here lists `sub2/a.crlf` under
  a CRLF `.gitignore` (`*.crlf\r\n`) that the walk honours; cause not measured (`core.autocrlf=true` on the box).
- PLANT "arms of an `exclusive` scope side by side": not red — the mutant's run recorded one span
  (`[('nogo', …)]`), so no overlap could show; cause not measured.

## After the swap (measured on win-laptop, 2026-10-11)
The v12.2 → v12.3 diff of verify.py showed v12.3's annex lacks three more v12.2 fixes the project's build carried —
re-added with fix 1, flagged for M0-TZ:
1b. **`case_env` itself (M0-D7):** UTF-8 output and `PYTHONFAULTHANDLER=1` for every case (Hazards: a native crash
   leaves its stack). v12.3 builds the env inline twice (`redarm`, `run_scopes`) with neither — both now call
   `case_env(task)`; the selftest's J2 check and plant "an env without faulthandler reads red" re-added.
1c. **The selftest stub's trace lock:** one writer at a time on `PB_TRACE` ("two appends at once tear a line on
   Windows; 5 s at most") and the torn-tolerant `spans()` with its plant "a torn trace record …". Without it the
   `exclusive` plant above stayed GREEN — with it, red: that red was this lost fix.

| Tool | selftest, swapped file in place |
|---|---|
| plan.py | GO |
| content_gate.py | GO |
| rung_record.py | GO |
| switch.mjs | GO (12.3.1) |
| verify.py | NO-GO 2 — checks=345 · failed=2 · plants=157/157 red, 162 s → M0-D20 |

The two left, root-caused (fixture, Windows only — the walk itself is right):
- `['trail']`: Win32 strips a trailing space — `open('trail ','w')` creates `trail` (probed).
- `['sub2/a.crlf']`: `Path.write_text("*.crlf\r\n")` in text mode writes `b'*.crlf\r\r\n'` on Windows (probed); git strips
  one `\r`, keeps `*.crlf\r`, and lists `a.crlf`; the walk drops both. In a clean folder with a true CRLF line git 2.56
  ignores `a.crlf` (probed) — the fixture, not git, differs.

`rung_record.py now` after the swap: `model=claude-opus-5-5 level=high plugin=stale` (box plugin 12.2.1, switch 12.3.1);
the swapped switch's `RUNGS` holds all seven rungs of the 2026-10-11 ladder.

## Local diffs after the swap — a future annex swap must carry them
5. **verify.py — the steps numbered on the console (local, M0-D27, 2026-10-12; the lead at M0-V4: « the number of the
   current step and the number of steps remaining »).** A run of several steps numbers each: `step k/N, m left` on a
   scope's row (`--all`, `--changed`, a list of scopes; a step is a scope that runs, plain ones then `exclusive` ones,
   the smoke last), `arm k/N, m left` on each arm's line of `--redarm <scope>`, `campaign k/N, m left` on each
   campaign's `redarm:` line of `--redarm --moved` (its arms numbered within it). `m` counts the steps after this one,
   so the last reads `0 left`; a one-step run, campaign or single arm carries no number. On a terminal (stderr a tty,
   no `--no-progress`) each step is named live as it starts: `>> <scope> · step k/N, m left · started`, an arm
   `>> [campaign k/N, m left · ]<scope> · <arm> · arm k/N, m left · started`, a campaign `>> <scope> · campaign … ·
   started`. The counts line, the failure and skip lines, the verdict, the per-scope logs, loop_times.jsonl and the
   anchor are unchanged.
   Hunks: `SAY` (a lock) beside `PART`; `step_tag()` and `say()` after `shown()` — every live line, `run_unit`'s
   `.. <label> <s>s` too, written whole under `SAY`; `ScopeRun.step`; `run_scopes` numbers the scopes in `todo`, `one()`
   says a step's start once, its rows gain the tag; `redarm(..., prefix="")` numbers the arms in `todo`, says each
   arm's start, its lines gain the tag; `moved_sweep` classifies every armed scope first (the same lines, the same
   order, the same failures), then runs the campaigns numbered. Selftest: section « L · the steps numbered » (a
   three-step run reads 1/3, 2/3, 3/3 and `0 left`, the counts line and verdict without a number, a one-step run
   unnumbered, a fake tty's live lines one per step and whole) with its plant « steps left counted off by one »; the
   arm and campaign checks after « their lines in declared order » and « --redarm --moved: no proof yet »; four
   existing checks read a row's head rather than the whole row (`r["lines"][0].startswith("par · clean · GO")` ×3,
   the `noted` row's `GO( · step …)?$`). Measured on win-laptop: selftest checks=352 · plants=158/158 red, GO.
6. **rung_record.py — the handoff fallback reads the quoted `now` line first (local, M0-D32, 2026-10-13).** A DONE block
   with no session file readable took the first ladder-rung string in its handoff's first line (`RUNG_RE`), so a handoff
   quoting `model=claude-opus-5-5 level=max` after prose naming "Sonnet 5.5, high" read as Sonnet; one quoting with no
   prose rung read `unknown` (69 of 71 quoting handoffs misread on this plan with no session read). Now `handoff_rung()`
   reads the `- Handoff:` field (its line and the indented lines under it): the quoted `model=<id> level=<level>`
   (`QUOTE_RE`) in the ladder's words (`rung_name`, a dated id loses its date) first, else the first line's prose rung as
   before, else `unknown`; the row's `rung_basis` reads `no session; rung from the handoff's quote (model= level=)` · `…
   the handoff's prose` · `… nothing: the handoff names none`. Hunks: `QUOTE_RE` beside `RUNG_RE`; the module docstring's
   "keeps the rung of its handoff" sentence; `handoff_rung()` before `build()`; `build()`'s fallback calls it. Selftest:
   `HANDOFF_PLAN` (five blocks: a prose rung before the quote, a gate quoted after a cheaper rung's prose, the quote on a
   continuation line with a dated id, prose only, neither with a quote outside the handoff), `_prose_first` (the old
   reading) and `_handoff_rows`; the check « clean: no session → the handoff's quoted model= level= … » after « a block
   with no session keeps the rung of its handoff », with its plant « a handoff's quoted model= level= ignored, its prose's
   first rung read ». The annex's §A.9 text ("No session file readable … → the rung from the handoff") and its plant count
   (thirteen for `report`) lag by this one. Measured on win-laptop: selftest checks=72 · plants 30/30 red, GO.
7. **rung_record.py — a stub's handoff is read from its archived block (local, M0-D41, 2026-10-15).** A stub keeps one line
   of its block's handoff, which can drop the quote the archived block holds on a continuation line; with no session file
   the fallback read that one line, so M0-D5 and M0-T19 read `unknown`. Now `handoff_rung(text, folder)` follows the stub's
   `- Full block: <file> § <id>` (`FULL_RE`) to that archived block's `- Handoff` field (`_archived_field`, through
   `P.find_blocks`): its quote first, else the stub's quote, else the archived field's first line's prose, else the stub's
   one line where the file or the block is missing. `build()` passes the plan's folder. Hunks: `FULL_RE` beside `QUOTE_RE`;
   `_field` and `_archived_field` before `handoff_rung`; `handoff_rung`'s signature and body; `build()`'s call; `_prose_first`
   takes `folder`; the selftest's `HANDOFF_PLAN` (stubs M9-T5 and M9-T6), `HANDOFF_ARCHIVE` (M9-T5's full block, quoted on
   its second line), `_handoff_rows` (writes the archive), the `quoted` check and its plant « a stub's archived block never
   followed, its one line read ». The annex's §A.9 text and its plant count lag by this one, as item 6's do. Measured on
   linux-pc: selftest checks=73 · plants 31/31 red, GO; `report --json` GO, M0-D5 and M0-T19 on Opus 5.5, high, and 53 of 53
   quoting handoff rows on their quote's rung (was 51 of 53).
8. **verify.py — `box` takes a list (local, M0-D111, 2026-10-29; the lead at M0-D111: «Let `box` take a list
   (Recommended)»).** A check both PCs run by their own routes (R15) and laserax-ai-server cannot (no Xvfb, Chrome or trunk,
   R18) had no manifest form: one host name owed it on win-laptop too. Now `box` is one host name or a list of them;
   `box_names(sc)` reads either, `owed_on` owes on every box not in the list (case and spaces ignored, as before),
   `owed_label(sc)` prints «A or B» in the owed lines, the refusal, the sweep line and `list`; `scope_errors` refuses an
   empty list, an empty name or a non-string. Hunks: `scope_errors`'s `box` test; `box_names` and `owed_label` before
   `owed_on`; `redarm`'s refusal; `moved_sweep`'s owed line; `run_scopes`' `sr.owed`; `list`'s `box` field; the selftest's
   check « a `box` list runs on each box it names and owes on every other (M0-D111) » after the anchor's owed check, its
   plant `tools-selftest-box-list` (verify.json, case `verify`: the list read by its first name only). The annex's §A.2
   manifest shape still reads `"box": "<the one box …>"`. Measured on linux-pc: `--redarm tools_selftest` 2/2 plants RED, GO.
9. **verify.py — a scope's `arms_at_once` (local, M0-D108, 2026-10-30; the lead at M0-D108: «arms_at_once key (Rec.)»).**
   A red-arm copy runs `cargo test` whole inside tests/cargo.sh's shared-target lock, so a cargo scope's arms ran one after
   another while each arm's timeout ran from the campaign's start: win-laptop's sweep timed out six arms queued (observe_run,
   dump, calibrate_bisect — the arms' ends a staircase, three never holding the lock). Now `arms_at_once` (an integer ≥ 1) caps
   the arms a campaign runs at once; `scope_errors` refuses anything else (`arms_at_once` 0, a bool, a string: NOT RUN).
   verify.json sets it to 1 on the 65 scopes whose command is `bash tests/gpu.sh|cpu.sh|cargo.sh`. Hunks: `SCOPE_KEYS`;
   `scope_errors`' test after `rerun`'s; `arm_workers`' docstring and its `min`; the selftest's planted-manifest row `v_arms`,
   the redarm fixture's scope `aa`, the check « arm_workers: never more than the scope's `arms_at_once` », the check
   « --redarm: an `arms_at_once: 1` scope's arms one at a time » and its plant « `arms_at_once` ignored »; verify.json's
   plant `tools-selftest-arms-at-once` (case `verify`: `arm_workers` without the cap). The annex's §A.2 manifest keys and
   its red-arm paragraph lag by this one. Measured on win-laptop: selftest checks=357 · plants 160/160 red, GO.
10. **launch.py — the records' lock and merge (local, M0-D131, 2026-10-31; the lead at M0-D131: «Go: lock + merge
   (Recommended)»).** Every run read the pid file once (`Context.__init__`) and wrote the whole dict back at each `save`, on
   one shared `<pid_file>.tmp`: two runs at once dropped each other's entries or tore the file (M0-D128's probe (c), 6 of 6
   rounds bad on linux-pc), and a dropped service ran on unseen by `stop`. Now `records_lock(path)` holds `<pid_file>.lock`
   (flock on POSIX, msvcrt on Windows; the OS drops it with a dead holder; held over `LOCK_WAIT_S` = 30 s → WriteRefused
   naming the lock), and `save` re-reads the file under it and changes only the entries this run set, changed or dropped
   since its last read (`Context.seen`). Hunks: the module docstring's paragraph; `LOCK_WAIT_S`; `records_lock` before
   `read_pids`; `Context.__init__`'s `seen`; `Context.save`; the selftest's « 8 starts at once on one pid file … » and
   « 8 stops at once … » before the ask gate; .gitignore's `tools/pb/.launch.pid.json.lock`; verify.json's plant
   `tools-selftest-launch-records` (case `launch`: the merge over the records as last read, not as they are now). The
   annex's §A.3 says nothing of runs at once. Measured on linux-pc: probe (c) 20/20 rounds clean, `--redarm tools_selftest`
   4/4 plants RED, GO; win-laptop (msvcrt) owed (R15).
11. **verify.py — SIGINT live in every case however the run was launched (local, M0-D125, 2026-10-31; the lead at M0-D125:
   «Fix verify.py (Recommended)»).** A shell starts an async job (`nohup … &`, no job control) with SIGINT and SIGQUIT ignored
   (`SigIgn: 0000000000000006`, probed on linux-pc), every descendant inherits the ignore, and Python keeps it: `stop_tree`'s
   SIGINT then stopped nothing, a timed-out case waited its 15 s grace for SIGKILL and its runner's teardown never ran. Seen as
   `tools_selftest[verify]`'s red « PLANT a command that hangs » (0.2 s + 15 s = 15.20 s, over the plant's 10 s) in M0-V12's two
   `nohup` loops and its `nohup` sweep on laserax-ai-server, never in a foreground run; reproduced on linux-pc as a `bash -c '… &
   wait'` job, 1 of 1. Now `live_sigint()` (POSIX), first thing in `main()`, puts SIGINT back to Python's own handler when it
   arrives ignored — a handler, unlike an ignore, is reset at exec, so every case starts with SIGINT live. Side effect, named: a
   backgrounded verify.py itself now stops on SIGINT, as a foreground one always did. Hunks: `live_sigint` after `stop_tree`;
   its call in `main()`; the selftest's check « a run handed SIGINT ignored still stops a hung command at its SIGINT, not 15 s on »
   after the plant « a command that hangs » (the selftest sets SIGINT ignored around one inner run); verify.json's plant
   `tools-selftest-sigint` (case `verify`: the call dropped from `main()`). The annex's §A.2 says nothing of the launch's signal
   dispositions. Measured on linux-pc: selftest checks=359 · plants 160/160 red, GO, in the foreground and as an async job;
   `--redarm tools_selftest` 5/5 plants RED, GO; laserax-ai-server and win-laptop owed (R15; a no-op on Windows).
12. **verify.py — `--redarm <scope>` runs the plants only; the clean GO is the scope's own green run on these bytes (local,
   M0-D178, 2026-11-15; the lead at M0-T104: «the red arm shouldnt run the clean copies if it already runs prior», «Make it
   so the red arm only runs the red arm», and at M0-D178: «make sure if the clean arm is run, other tests re-use it»).** The
   clean arm repeated the scope run the routine had just made (R9 (2) then (3)) — for a ⏱ scope a second whole run
   (`endings` ≈ 45 min). Now every whole run of an armed scope (no `--case`, no case vouched by the anchor, the tree not
   moved under it) writes the scope's `fingerprint` and `adapter` (`SR_TEST_ADAPTER`) into its record entry, red or green;
   `green_proof` reads the newest record on this box with that fingerprint and adapter — a whole run GO, or a red-arm whose
   clean arm ran GO, is the proof; the same bytes run red is none. `redarm(clean=…)`: `"record"` (the CLI's `--redarm
   <scope>`) refuses with no proof, nothing run, naming `verify.py <scope> --task <ID>`; `"auto"` (the sweep `--redarm
   --moved`, the loop's plants) reads it, else runs the clean arm as before; `"fresh"` (`--redarm <scope> --with-clean`)
   always runs it. A clean arm that runs GO is recorded with fingerprint and adapter, so every later reader reuses it. The
   loop's hook gets the run's record before it is written (`hook(rec)`), so `--changed` / `--all` red-arm a changed scope
   off the green run they just made. Hunks: `ADAPTER_VAR`, `adapter_now`, `green_proof` in place of `clean_proof`;
   `redarm`'s `clean`/`fresh` parameters, its refusal, its proof line and log line (`(<what ran>; fingerprint …) - not
   re-run`), its record's `adapter`; `moved_sweep` unchanged (`clean="auto"` is the default); `loop_checks`' `fresh`;
   `run_scopes`' `tool`, `fp_before`, the fingerprint written after the run, `rec["scopes"]` built before the hook,
   `hook(rec)`; `loop_hook`'s two lambdas; `CALLS`, `--with-clean` and `check_call`'s refusal; `cmd_run`. Selftest: `qarm`'s
   `clean`; scopes `rd`, `mk`; the old clean-proof checks re-cut on `green_proof` (Q4–Q6, the legacy reader as six synthetic
   records); section « G3 · the red-arm runs the plants only » (refused with no green run; the record written; plants only
   after it; another adapter refused; a path edited since refused; `--with-clean` then reused; a `--case` run and a moved
   tree write no fingerprint; the hook sees the record; the CLI sequence); two sweep checks (record read, else clean run);
   two wrong calls; `ADAPTER_VAR` saved and popped. verify.json's plants `redarm-runs-clean` and `redarm-stale-record`
   (case `verify`). The annex's §A.2 red-arm paragraph («the clean copy GO, each plant NO-GO») lags by this one — flagged
   for M0-TZ. Measured on linux-pc: selftest checks=376 · plants 163/163 red, GO; `--redarm tools_selftest` 7/7 RED, GO,
   its clean read from the scope's run; laserax-ai-server and win-laptop owed (R15).
13. **verify.py — a long command says its start and a heartbeat with its ETA (local, M0-D179, 2026-11-15; the lead: «can
   you also make sure that verify.py shows some progress and and ETA. right now the endings run is running and its not
   printing anything»).** M0-D27's and M0-D65's live lines were said only by a run of several steps, and a command's own
   line only when it ended, so a one-scope, one-command run (`endings`, ≈ 45 min) said nothing until its verdict. Now, on a
   terminal only: a one-step run says `>> <scope> · started · ≈ <s> by this box's last run` (or `no past run …`); a
   red-arm's lone arm says its start; a command still running says every `PB_HEARTBEAT_S` (default 60 s) `.. <label>[ ·
   <arm>] · running <elapsed> · <k> of <expected> cases done · ≈ <left> left by its pace (…)`, else `… by this box's last
   run (…)`, else `over this box's last run (…) by <s>`, with `now <case>` while one runs; the cases are libtest's lines in
   the command's part file — `test <name> ... ` started, `ok|FAILED|ignored` ended, on the same line or alone after the
   test's own output (`--nocapture`, as tests/gpu.sh runs: checked on M0-T104's bh, giant and mass_up logs, started = ended). Display only: verdicts, counts, logs, records and the anchor unchanged. Hunks: `CASE_DONE_RE`,
   `heartbeat_s`, `beat_line`, `wait_beating` after `eta_line`; `run_command`'s wait; `ScopeRun.past`/`tag`;
   `run_scopes`' `live_tag`, `hist` read once, the one-step start line; `redarm`'s lone-arm start and `live_tag`. Selftest:
   EMIT's `cases` kind; section « L2 · a long command says its start and a heartbeat » (four checks, the plant « a command
   waited on in silence »); `PB_HEARTBEAT_S` saved and popped. The annex's §A.2 says nothing of live lines.
   Measured on linux-pc: selftest checks=381 · plants 164/164 red, GO; on a pty (`script`) `--redarm ending` said its lone
   arm's start and a heartbeat each 0.5 s (`PB_HEARTBEAT_S=0.5`) while cargo rebuilt (logs/M0-D179.pty.log).
14. **verify.py — a run's commands end with the run, however it ends (local, M0-D220, 2026-11-27; filed by M0-V15).** Each
   command runs in a session of its own (`new_group`, for stop_tree's one-command stop), so a capper's TERM or KILL to
   verify.py reached none of them: eight `tests/gpu.sh` groups of M0-D219's and M0-D214's capped claims ran on under PID 1,
   the RTX 5090 at 99 %, writing into part files nobody joined. Now a guard — one helper process per verify.py process, in
   a session of its own, so a capper's signal to verify.py's group never reaches it — reads on a pipe the groups the run
   holds (`+<pgid>\t<log>\t<part>\t<label>`, `-<pgid>` once the command ended, so no late signal reaches a finished
   scope); at the pipe's end (verify.py exited, crashed or was killed, KILL included) or at `!` it sends each group still
   held SIGINT, waits `GUARD_GRACE_S` (4 s) for a teardown, sends SIGKILL, and notes `exit=stopped … its output kept in
   <part>` in the scope's log; the part file stays under its name. And the CLI's TERM, INT and HUP (HUP unless handed it
   ignored, `nohup`) stop the run at once: no command starts after (`STOPPING` under `SPAWN`, the start and the hold as
   one), the guard stops the groups, then `=== NO-GO: stopped by SIG<X> - its commands stopped, their part files kept ===`
   and exit 128+signal — before, INT (Ctrl-C) raised in the main thread while the pool waited on commands Ctrl-C never
   reaches. Why a guard: PR_SET_PDEATHSIG is Linux only, reaches the direct child (bash) and not cargo or the gpu binary
   under it, and fires when the spawning thread ends (a pool thread); the capper's group is signalled only by GNU
   `timeout` without `--foreground`, never by `kill -9 <pid>`. Hunks: `SPAWN`, `STOPPING`, `GUARD_GRACE_S` beside `SAY`;
   `GUARD_SRC`, `Guard`, `GUARD`, `stop_on_signals` after `live_sigint`; `run_command`'s start under `SPAWN` with
   `GUARD.hold`, `GUARD.drop` after the wait, the early return under `STOPPING` (a start refused there removes its
   empty part and logs `exit=NOT RUN (the run was stopped by SIG<X>)`); the `STOPPING` check under `LOCK` before a
   command's header; the handler's `LOCK` before `os._exit`; `stop_on_signals()` in `__main__` only
   (the selftest's in-process runs keep their handlers). Selftest: three checks after « a run handed SIGINT ignored … » — a
   real verify.py on a `sleep 60` scope stopped by TERM, INT, KILL mid-command: the child gone within 5 s, one part file
   kept, `exit=stopped` in the log, the NO-GO line (not KILL), exit 128+signal, a queued `exclusive` scope never
   started (no part, its log if any NOT RUN); POSIX only. verify.json's plant `guard`
   (case `verify`: the hold dropped from `run_command`, tests/plants/tools-selftest-guard.patch). Windows: no guard and no
   signal stop (`CREATE_NEW_PROCESS_GROUP`, no job object) — a killed run there still leaves its commands; a job object
   with KILL_ON_JOB_CLOSE is the route, not built; the three checks are skipped and the plant reads green there (R15,
   owed). The annex's §A.2 says nothing of a run's end. Measured on linux-pc: selftest checks=384 · plants 164/164 red,
   GO; `--redarm tools_selftest` 8/8 RED, GO; `timeout -s TERM|KILL 60 verify.py conservation` with `gpu-…` live on the
   RTX 5090: nothing of the run left 5 s after (logs/M0-D220.log).
