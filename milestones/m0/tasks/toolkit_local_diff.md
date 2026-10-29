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
