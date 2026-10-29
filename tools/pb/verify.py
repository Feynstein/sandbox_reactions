#!/usr/bin/env python3
"""tools/pb/verify.py - the one runner, the routine's instrument (PLAYBOOK annex §A.2).

  <scope>... --task <ID>                   those scopes
  --all --task <ID>                        the complete loop: every scope, then the smoke
  <scope> --case <id> --task <ID>          one case: PARTIAL, never the loop
  <scope>... [--case <id>] --force ...     the same, a scope already GO on this tree run again (else it is not re-run)
  --changed [--base <commit>] --task <ID>  every scope a changed path touches, read from the coverage anchor
  --redarm <scope> --task <ID>             one scratch copy per arm, the arms side by side: clean GO, each plant NO-GO
  --redarm --moved --task <ID>             the phase's re-proof sweep: every armed scope its proof no longer vouches for
  list      report [--since YYYY-MM-DD] [--budget <s>]      selftest
  options: -j N | --serial · --json · --manifest <file> · --root <dir> · --no-progress

PLAYBOOK annex §A.0 (conventions) + §A.2 (this tool).  Python 3.9+, standard library only, Windows and
POSIX.  The scopes are the entries of tools/pb/verify.json; docs/agent/testing.md says how to add one and
what each costs.  Every call ends in the §4 Rule 4 shape: its lines, the failures only when there are any,
one counts line, then `=== GO ===` or `=== NO-GO: <reason> ===` derived from the counts (exit 0 / 1);
`--json` prints the same result as one object; a wrong call prints the right one.
A run refuses to start without `--task` (or $PB_TASK): no log is shared.  Each scope's log is opened at
<logs_dir>/<TASK>.<scope>.log before its first command starts, and every run appends one line to
<logs_dir>/loop_times.jsonl naming its box and interpreter.  No project file is deleted, no git write is
run, and no process this run did not start is signalled.  The tree is listed by a walk that reads .gitignore
and its id comes from content: git is read only where a call names it (--base) or a repository can answer
(the HEAD prefix, the committed-evidence guard) - always read-only, with --no-optional-locks.
"""
from __future__ import annotations

import argparse
import contextlib
import datetime as _dt
import difflib
import hashlib
import io
import json
import os
import platform
import posixpath
import re
import shlex
import shutil
import signal
import socket
import statistics
import subprocess
import sys
import tempfile
import threading
import time
import traceback
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent
ROOT = TOOL_DIR.parent.parent                       # the repository: tools/pb/verify.py
MANIFEST = TOOL_DIR / "verify.json"
PROG = "python3 tools/pb/verify.py"
PARSERS = ("gonogo", "pytest", "jest", "regex", "exit")
COMMANDS = ("list", "report", "selftest", "start", "touched")
BASELINES_KEPT = 20                                 # the newest starts a box keeps (M0-T62)
CALLS = ("<scope>... --task <ID>", "--all --task <ID>", "<scope> --case <id> --task <ID>",
         "--changed [--base <commit>] --task <ID>", "--force <scope>... --task <ID>", "--redarm <scope> --task <ID>", "--redarm --moved --task <ID>", "list",
         "report [--since YYYY-MM-DD] [--budget <s>]", "start [--fresh] --task <ID>",
         "touched [--task <ID>] [--path <glob>]", "selftest")
TOUCH_KINDS = ("added", "modified", "deleted")
NAME_RE = re.compile(r"^[a-z][a-z0-9_]*$")
ID_RE = re.compile(r"^[\w./@+-]+$")             # a case id lands in a command: no space, no shell character
TASK_RE = re.compile(r"^[\w.-]+$")              # a task id lands in a file name
COUNTS_RE = re.compile(r"(\d+) passed, (\d+) failed, (\d+) skipped")
VERDICT_RE = re.compile(r"^=== (GO|NO-GO(?::[^\n]*?)?) ===[ \t]*$")
PYTEST_LINE = re.compile(r"\d+ (?:passed|failed|skipped|errors?)\b.* in [\d.]+s")
JEST_LINE = re.compile(r"^Tests:\s.*\d+ total")
PAIR_RE = re.compile(r"\b(\d+) (passed|failed|skipped|todo|errors?)\b")
TEST_RE = re.compile(r"(?:^|/)(?:tests?|spec|__tests__)/|(?:^|/)test_[^/]*$|_test\.[^/]+$|\.(?:test|spec)\.[^/]+$")
PLAN_FILE_RE = re.compile(r"(?:implementation_plan|plan_archive)[^/]*\.md$")
CODE_SUFFIXES = (".py", ".sh", ".mjs", ".js", ".cjs", ".ts", ".tsx", ".jsx", ".ps1", ".bat", ".cmd", ".go", ".rs",
                 ".java", ".c", ".cc", ".cpp", ".h", ".hpp", ".cs", ".rb", ".php", ".swift", ".kt")
TOP_KEYS = {"scopes", "smoke", "jobs_cap", "logs_dir", "tree_extra", "anchor", "skip_classes", "vars", "budget"}
SCOPE_KEYS = {"cmd", "cases", "cases_cmd", "case_cmd", "placeholder", "parse", "regex", "count",
              "expected", "exclusive", "timeout_s", "cwd", "paths", "box", "plants", "free_ports", "good", "rerun"}
NOTE_KEYS = ("note", "desc", "why", "hint", "reason")    # and any `_` key: a note - read, never acted on
SPEC_KEYS = ("cmd", "cases", "cases_cmd", "case_cmd", "parse", "regex", "count", "expected", "cwd", "good")
PLANT_KEYS = {"id", "patch", "case"}
RECORD_KEYS = {"ts", "task", "mode", "scopes", "seconds", "verdict", "tree"}
PLACEHOLDER_RE = re.compile(r"\{[A-Za-z_]\w*\}")         # in a path only this runner reads, nothing fills it
RULES_FILE_RE = re.compile(r"(?:^|/)\d*m[\w.]*_rules\.md$")   # m<N>_rules.md, where a MOVE block moves Rules entries
PARTS = {"smoke": "the smoke: --all reads NO-GO until it is fixed",
         "jobs_cap": "the default job count: one command at a time unless -j N",
         "logs_dir": "every run and `report`: they open their logs and records there",
         "anchor": "the coverage anchor: never advanced, --changed refused",
         "tree_extra": "the tree id (null) and the coverage anchor (never advanced, --changed refused)",
         "skip_classes": "the coverage anchor: never advanced, --changed refused",
         "vars": "every scope whose command holds a placeholder",
         "budget": "report's budget lines"}              # what each top-level key drives - NOT RUN when it is bad
EXCLUDES = (".git", ".venv", "venv", "node_modules", "models", "data", "var", "output", "__pycache__",
            ".pytest_cache", "dist", "build")          # the scratch copy's, annex §A.7: by name, at any depth
EMPTIED = ("images",)                                    # the folder is copied, its contents are not
LINKS = (".venv", "node_modules")                        # linked back, so the copy still runs
ANCHOR_DEFAULT = "tools/pb/verify.anchor.json"
ANCHOR_VERSION = 3
ANCHOR_NOTE = ("written by tools/pb/verify.py - covered path -> the scope that graded it, at the content hash "
               "it graded and the scope's own definition. Advanced only by a scope that went GO; a scope that "
               "went red loses its entries. `cases`: per scope whose cases are files, each case -> the content "
               "hash it last went GO on, the scope's suite hash and its count. `verify.py --changed` reads it.")
PROGRESS = True                                          # live stderr progress on a terminal, display only
WIN32_LOAD_WINDOW_S = 0.25                               # the GetSystemTimes window, never inside `seconds`
DEFAULT_LOAD_K = 0.15                                    # seconds / (1 + k x load / cpus): the fleet's median, M0-T64d
TR_LOOPS = 3                                             # the TR line reads the median of a box's last 3 counted loops
LOCK = threading.Lock()
PART = [0]                                               # part-file counter (under LOCK)
SAY = threading.Lock()                                   # one live console line at a time, whole (M0-D27)
HOME = str(Path.home())


def this_box():
    """The box's name: $PB_BOX, else the host name - the name the plan's run tags use (`on <box>`)."""
    return (os.environ.get("PB_BOX") or socket.gethostname() or "?").strip()


PYTHON = platform.python_version()


class Refused(Exception):
    """Nothing runs; the verdict is NO-GO with this reason (and these lines, when there are several)."""

    def __init__(self, reason, lines=()):
        Exception.__init__(self, reason)
        self.lines = list(lines)


class CallError(Exception):
    """A wrong call: nothing runs, and the calls are printed."""


def result(counts, failures=(), lines=(), nogo=None, **data):
    return {"counts": counts, "failures": list(failures), "lines": list(lines), "nogo": nogo, "data": data}


def stamp():
    return _dt.datetime.now().astimezone().isoformat(timespec="seconds")


def rel(path, root):
    return os.path.relpath(str(path), str(root)).replace(os.sep, "/")


def shown(text):
    """A printed path never carries the home folder's name (it names a person): `~` stands in its place."""
    text = str(text)
    return text.replace(HOME, "~") if len(HOME) > 1 else text


def step_tag(what, k, n):
    """A step's place in its run (M0-D27): `step 2/5, 3 left` - its number, the total, the steps after it."""
    return f"{what} {k}/{n}, {n - k} left"


def say(text):
    """One live line on a terminal's stderr, display only: written whole under SAY, so threads never tear one."""
    if PROGRESS and sys.stderr.isatty():
        with SAY:
            sys.stderr.write(shown(text) + "\n")
            sys.stderr.flush()


def live():
    """Whether `say` writes at all - the live ETA's history is read only then."""
    return PROGRESS and sys.stderr.isatty()


def past_seconds(logs, box, redarm=False):
    """This box's newest seconds per scope from loop_times.jsonl - a loop's scope rows, or (`redarm`) one scope's
    red-arm campaign. Display only: the live ETA's guess, never a verdict (M0-D65's lead, 2026-10-28)."""
    out = {}
    for r in records_in(Path(logs) / "loop_times.jsonl")[1]:
        sc = r.get("scopes") if r.get("box") == box else None
        if not isinstance(sc, dict):
            continue
        if redarm:
            if r.get("mode") == "redarm" and len(sc) == 1 and is_num(r.get("seconds")):
                out[next(iter(sc))] = float(r["seconds"])
        elif r.get("mode") != "redarm":
            out.update({n: float(e["seconds"]) for n, e in sc.items() if isinstance(e, dict) and is_num(e.get("seconds"))})
    return out


def dur(s):
    """Seconds as `1h05m`, `7m10s` or `45s`."""
    s = int(round(max(0.0, s)))
    return f"{s // 3600}h{s % 3600 // 60:02d}m" if s >= 3600 else f"{s // 60}m{s % 60:02d}s" if s >= 60 else f"{s}s"


def eta_line(what, done, running, todo, of, elapsed, work_done, work_left, at_once=1):
    """`.. <what>: 12 done · 8 running · 29 to do (of 49) · 3m10s elapsed · ≈ 7m02s left`: the history's seconds
    still to run (`work_left`), scaled by this run's own pace - its elapsed wall time over the history's seconds it
    has got through (`work_done`, a running one's capped at its own); with none yet, shared `at_once`."""
    pace = elapsed / work_done if work_done > 0 and elapsed > 0 else 1.0 / max(1, at_once)
    return (f".. {what}: {done} done · {f'{running} running · ' if running else ''}{todo} to do (of {of}) · "
            f"{dur(elapsed)} elapsed · ≈ {dur(work_left * pace)} left")


def guesses(names, hist):
    """Each name's history seconds; one without any takes the known ones' median (60 s with none) -> (dict, count)."""
    known = sorted(hist[n] for n in names if n in hist)
    mid = known[len(known) // 2] if known else 60.0
    return {n: hist.get(n, mid) for n in names}, sum(1 for n in names if n not in hist)


# ----------------------------------------------------------- the load sample: a number, or WHY there is none
def busy_cpus(idle0, all0, idle1, all1, cpus):
    """Two GetSystemTimes samples as CPUs' worth of busy - a load average's own unit."""
    spent = all1 - all0
    if spent <= 0:
        raise OSError(f"GetSystemTimes did not advance ({all0} -> {all1})")
    return round((1.0 - (idle1 - idle0) / spent) * (cpus or 1), 2)


def win32_load(cpus=None, window_s=WIN32_LOAD_WINDOW_S, sleep=time.sleep):
    """Windows has no load average (`os.getloadavg` does not exist there). GetSystemTimes over a short
    window gives the busy fraction; times the CPU count it is in a load average's own unit, so the two
    kinds of box record numbers that mean the same thing. ctypes only: no spawn, nothing to hang on."""
    import ctypes
    from ctypes import wintypes

    k32 = ctypes.WinDLL("kernel32", use_last_error=True)

    def ticks():
        idle, kern, user = wintypes.FILETIME(), wintypes.FILETIME(), wintypes.FILETIME()
        if not k32.GetSystemTimes(ctypes.byref(idle), ctypes.byref(kern), ctypes.byref(user)):
            raise OSError(ctypes.get_last_error(), "GetSystemTimes failed")
        whole = lambda f: (f.dwHighDateTime << 32) | f.dwLowDateTime  # noqa: E731
        return whole(idle), whole(kern) + whole(user)     # kernel time INCLUDES idle, so this is the total

    idle0, all0 = ticks()
    sleep(window_s)
    idle1, all1 = ticks()
    return busy_cpus(idle0, all0, idle1, all1, cpus or os.cpu_count() or 1)


def sample_load(osmod=os, win32=win32_load):
    """(value, source) - never a bare None: a box that cannot be sampled records the reason, so no row
    is a silent blank and nobody has to guess what a missing load meant."""
    if hasattr(osmod, "getloadavg"):
        try:
            return round(osmod.getloadavg()[0], 2), "getloadavg1"
        except OSError as exc:
            return None, f"none: os.getloadavg() failed: {exc}"
    if win32 is None:
        return None, f"none: no sampler for os.name {osmod.name!r}"
    try:
        return win32(), f"GetSystemTimes/{WIN32_LOAD_WINDOW_S:g}s x ncpu"
    except Exception as exc:                  # a denied counter, a stopped clock, no ctypes: the reason IS the record
        return None, f"none: {type(exc).__name__}: {exc}"


# ------------------------------------- the seconds at an idle box: seconds / (1 + k x load / cpus), load the run's
def budget_box(man, box):
    """The manifest's budget entry for this box, {} when there is none or the budget cannot be read."""
    bud = man.get("budget") if "budget" not in top_errors(man) else None
    return ((bud or {}).get("boxes") or {}).get(box or "") or {}


def norm_k(man, box, cpus=None):
    """(k, source) - k relative to the box's CPUs, so one factor reads every box. The manifest box's fitted `load_k`
    (seconds per load unit, the budget's form) times its CPUs wins; else the fleet's median, DEFAULT_LOAD_K."""
    bb = budget_box(man, box)
    lk, n = bb.get("load_k"), bb.get("cpus") or cpus
    if is_num(lk) and lk >= 0 and is_num(n) and n > 0:
        return round(lk * n, 4), f"manifest {box} load_k {lk:g} x {n:g} cpus"
    return DEFAULT_LOAD_K, f"default {DEFAULT_LOAD_K:g} - the fleet's median, M0-T64d"


def normalise(rec, man):
    """The record's `seconds_norm` (and `loop_seconds_norm` beside a `loop_seconds`) from its end `load` - the
    1-minute average over the run's last minute - and `load_norm` naming the k. No load sample: null, the reason
    named - never the raw seconds copied in."""
    load, box = rec.get("load"), rec.get("box")
    cpus = rec.get("cpus") or budget_box(man, box).get("cpus")
    keys = ["seconds_norm"] + (["loop_seconds_norm"] if "loop_seconds" in rec else [])
    if not is_num(load) or not (is_num(cpus) and cpus > 0):
        src = str(rec.get("load_source") or "no load sample")
        why = "none: no CPU count" if is_num(load) else src if src.startswith("none:") else f"none: {src}"
        rec.update({k: None for k in keys}, load_norm=why)
        return rec
    k, src = norm_k(man, box, cpus)
    d = 1.0 + k * float(load) / float(cpus)
    for key, raw in (("seconds_norm", "seconds"), ("loop_seconds_norm", "loop_seconds")):
        if key in keys:
            rec[key] = round(float(rec[raw]) / d, 3) if is_num(rec.get(raw)) else None
    rec["load_norm"] = (f"k {k:g} ({src}) · load {load:g} / {cpus:g} cpus"
                        + (" · OUT OF RANGE: load > cpus" if load > cpus else ""))
    return rec


# ------------------------------------------------------------------------------------------------ manifest
def regex_error(rx, group):
    try:
        return None if group in re.compile(rx).groupindex else f"has no (?P<{group}>...) group"
    except (re.error, TypeError) as exc:
        return f"does not compile: {exc}"


def str_list(v):
    return isinstance(v, list) and all(isinstance(x, str) and x for x in v)


def is_num(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def is_note(key):
    """A note - read, never acted on: any `_` key, and the note names the fleet's manifests use."""
    return key.startswith("_") or key in NOTE_KEYS


def near(key, known):
    """` (did you mean `x`?)` for a key one slip from a known one - a misspelt key is named, never guessed."""
    hit = difflib.get_close_matches(key, sorted(known), n=1, cutoff=0.8)
    return f" (did you mean `{hit[0]}`?)" if hit else ""


def commands(s):
    """The command strings a scope holds: `cmd`, `case_cmd`, `cases_cmd`, each `cases` entry."""
    if not isinstance(s, dict):
        return []
    cases = s.get("cases")
    return [c for c in [s.get(k) for k in ("cmd", "case_cmd", "cases_cmd")]
            + (list(cases.values()) if isinstance(cases, dict) else []) if isinstance(c, str)]


def scope_errors(name, s):
    if not isinstance(s, dict):
        return ["is not an object"]
    unknown = sorted(k for k in s if k not in SCOPE_KEYS and not is_note(k))
    out = [f"unknown key{'s' * (len(unknown) > 1)} " + ", ".join(f"`{k}`{near(k, SCOPE_KEYS)}" for k in unknown)] * bool(unknown)
    if not NAME_RE.match(name) or name in COMMANDS or name == "smoke":
        out.append("a scope name is lowercase [a-z0-9_] and not list, report, selftest or smoke")
    runs = [k for k in ("cmd", "cases", "cases_cmd") if k in s]
    if "placeholder" in s and (runs or "case_cmd" in s):
        out.append("a placeholder runs nothing - drop `placeholder` when you give the scope its command")
    elif "placeholder" not in s and len(runs) != 1:
        out.append("needs exactly one of `cmd`, `cases`, `cases_cmd` - or a `placeholder`")
    cases = s.get("cases", [])
    if isinstance(cases, dict):
        out += ["`cases` as an object holds each case's command: drop `case_cmd`"] * ("case_cmd" in s)
        out += [f"case `{c}`: its command is not a string" for c, v in cases.items() if not isinstance(v, str)]
    elif not isinstance(cases, list):
        out.append("`cases` is a list of ids, or an object id -> command")
        cases = []
    elif ("cases" in s or "cases_cmd" in s) and "case_cmd" not in s:
        out.append("a list of cases or a `cases_cmd` needs a `case_cmd`")
    if "case_cmd" in s and "{case}" not in str(s["case_cmd"]):
        out.append("`case_cmd` holds `{case}`")
    if "{case}" in str(s.get("cmd", "")):
        out.append("`{case}` belongs in `case_cmd`, not `cmd`")
    out += [f"case id `{c}` has a space or a shell character" for c in cases if not ID_RE.match(str(c))]
    if s.get("parse", "gonogo") not in PARSERS:
        out.append(f"`parse` is one of {', '.join(PARSERS)}")
    if s.get("parse") == "regex" and regex_error(s.get("regex"), "passed"):
        out.append(f"`regex` {regex_error(s.get('regex'), 'passed')}")
    if "count" in s and s.get("parse", "gonogo") != "gonogo":
        out.append("`count` goes with the `gonogo` parse")
    elif "count" in s and regex_error(s["count"], "n"):
        out.append(f"`count` {regex_error(s['count'], 'n')}")
    if not isinstance(s.get("expected", 0), int) or isinstance(s.get("expected"), bool) or s.get("expected", 0) < 0:
        out.append("`expected` is an integer >= 0")
    if not isinstance(s.get("timeout_s", 1), (int, float)) or s.get("timeout_s", 1) <= 0:
        out.append("`timeout_s` is a number > 0")
    if not isinstance(s.get("exclusive", False), bool):
        out.append("`exclusive` is true or false")
    if not isinstance(s.get("rerun", False), bool):
        out.append("`rerun` is true or false")
    if "paths" in s and not str_list(s["paths"]):
        out.append("`paths` is a list of globs")
    if "box" in s and not (box_names(s) and all(isinstance(b, str) and b.strip() for b in box_names(s))):
        out.append("`box` names the one box that can run it, or a list of the boxes that can")
    if not all(isinstance(p, int) and not isinstance(p, bool) for p in s.get("free_ports", [])):
        out.append("`free_ports` is a list of port numbers")
    plants, seen = s.get("plants", []), set()
    if not isinstance(plants, list):
        out.append("`plants` is a list of {id, patch}")
        plants = []
    for p in plants:
        if not isinstance(p, dict) or {k for k in p if not is_note(k)} - PLANT_KEYS or not all(
                isinstance(p.get(k), str) and p.get(k) for k in ("id", "patch")):
            out.append(f"a plant is {{\"id\": \"<plant>\", \"patch\": \"<patch file or command>\"[, \"case\": "
                       f"\"<one case id>\"]}}: {p!r}")
        elif "case" in p and not (isinstance(p["case"], str) and ID_RE.match(p["case"])):
            out.append(f"plant `{p['id']}`: `case` is one case id of the scope")
        elif not ID_RE.match(p["id"]) or p["id"] in seen:
            out.append(f"plant id `{p['id']}` is not a unique id")
        else:
            seen.add(p["id"])
    return out


def path_error(key, v):
    """Why `v` is no path this runner can use for `key`, or None - a `{name}` in it is a placeholder nothing fills."""
    if not isinstance(v, str) or not v.strip():
        return f"`{key}` is a path"
    ph = PLACEHOLDER_RE.search(v)
    return f"`{key}` holds `{ph.group(0)}`, a placeholder this runner does not fill" if ph else None


def top_errors(man):
    """{key: why} - each known top-level key whose value this runner cannot read: the part it drives (PARTS) is
    NOT RUN, named, and nothing is guessed in its place."""
    errs = {}
    if man.get("smoke") is not None and not (isinstance(man["smoke"], str) and man["smoke"].strip()):
        errs["smoke"] = "`smoke` is a command or null"
    jc = man.get("jobs_cap", 8)
    if not isinstance(jc, int) or isinstance(jc, bool) or jc < 1:
        errs["jobs_cap"] = "`jobs_cap` is an integer >= 1"
    for key, default in (("logs_dir", "logs"), ("anchor", ANCHOR_DEFAULT)):
        why = path_error(key, man.get(key, default))
        if why:
            errs[key] = why
    te = man.get("tree_extra", [])
    if not isinstance(te, list) or not all(isinstance(p, str) and not path_error("x", p) for p in te):
        errs["tree_extra"] = "`tree_extra` is a list of paths"
    sk = man.get("skip_classes", [])
    if not isinstance(sk, list) or not all(isinstance(c, dict) and str_list(c.get("match")) and isinstance(c.get("reason"), str)
                                           and c["reason"] for c in sk):
        errs["skip_classes"] = 'a skip class is {"match": ["<glob>"], "reason": "<why no scope grades it>"}'
    vs = man.get("vars", {})
    if not isinstance(vs, dict) or not all(str_list(v) for v in vs.values()):
        errs["vars"] = "`vars` maps a name to a list of candidate paths"
    bud = man.get("budget", {})
    boxes = bud.get("boxes", {}) if isinstance(bud, dict) else None
    if not isinstance(boxes, dict) or not all(isinstance(b, dict) and all(b.get(k) is None or is_num(b[k]) for k in (
            "base_s", "per_case_s", "load_k", "band_pct", "at_cases", "cpus")) and (is_num(b.get("at_cases")) or not b.get("base_s"))
                                              for b in boxes.values()):          # null: a box not measured yet
        errs["budget"] = '`budget` is {"boxes": {"<box>": {"base_s": <s>, "at_cases": <n>, ...}}}'
    return errs


def blocked(man):
    """{scope: why} - every scope this runner cannot read whole: an unknown key, a value outside its sets, a
    placeholder a bad `vars` cannot fill. Each is NOT RUN, named, and never run without its setting."""
    bad_vars, out = top_errors(man).get("vars"), {}
    for n, s in (man.get("scopes") or {}).items():
        errs = scope_errors(n, s)
        held = sorted({m for c in commands(s) for m in PLACEHOLDER_RE.findall(c)} - {"{python}", "{case}"})
        if bad_vars and held and not errs:
            errs = [f"{', '.join(held)}: {bad_vars}"]
        if errs:
            out[n] = "; ".join(errs)
    return out


def manifest_lines(man):
    """One WARN line per unknown top-level key, one NOT RUN line per part a bad known key drives."""
    return ([f"WARN top-level key `{k}`: unknown to this runner{near(k, TOP_KEYS)} - read as nothing, so a scope "
             f"that needs it runs without it (map it into the scopes, or make it a note)"
             for k in man if k not in TOP_KEYS and not is_note(k)]
            + [f"NOT RUN ({why}) - {PARTS[k]}" for k, why in top_errors(man).items()])


def logs_of(man, root):
    """<root>/<logs_dir> - refused, named, when `logs_dir` is no path this runner can use: every run opens its
    logs and appends its record there, and `report` reads them there."""
    why = top_errors(man).get("logs_dir")
    if why:
        raise Refused(f"NOT RUN ({why}) - every run opens its logs and appends its record there: nothing ran")
    return Path(root) / man.get("logs_dir", "logs")


def load_manifest(path):
    """The manifest, read as far as it can be: only an unreadable file or no `scopes` refuses the run. A note is
    read as a note; a scope this runner cannot read whole is NOT RUN, named (`blocked`), while the rest run; an
    unknown top-level key is a WARN line, a bad known one NOT RUNs the part it drives (`manifest_lines`). A
    misspelt key is never guessed: its scope does not run - so a manifest a project kept at adoption still runs
    every scope this runner understands."""
    try:
        man = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise Refused(f"manifest {os.path.basename(str(path))}: {type(exc).__name__}: {exc} - nothing ran")
    if not isinstance(man, dict):
        raise Refused("the manifest is not a JSON object - nothing ran")
    if not isinstance(man.get("scopes"), dict) or not man["scopes"]:
        raise Refused("`scopes` is missing, empty or not an object - nothing ran")
    return man


def spec_hash(sc):
    """What a scope checks, as a hash: its entries in the anchor stand only while this stays the same."""
    return hashlib.sha1(json.dumps({k: sc.get(k) for k in SPEC_KEYS}, sort_keys=True).encode()).hexdigest()[:12]


def box_names(sc):
    """A scope's `box` as a list: one host name, or the list of the boxes that can run it (M0-D111, a local divergence
    from annex §A at the lead's word: a check two boxes run by their own routes and a third cannot)."""
    b = sc.get("box")
    return [b] if isinstance(b, str) else list(b) if isinstance(b, list) else []


def owed_label(sc):
    return " or ".join(str(b).strip() for b in box_names(sc))


def owed_on(sc, box):
    names = [str(b).strip().lower() for b in box_names(sc)]
    return bool(names) and box.strip().lower() not in names


# ------------------------------------------------------------------------------------ one command, one scope
def parse(sc, text, rc):
    """((passed, failed, skipped) or None, the NO-GO reasons the output itself carries)."""
    kind, lines = sc.get("parse", "gonogo"), [s.strip() for s in text.splitlines()]
    if kind == "exit":                           # a command that reports an exit code only: one case
        return ((1, 0, 0), []) if rc == 0 else ((0, 1, 0), [f"exit {rc}, want: {sc.get('good', 'exit 0')}"])
    if kind == "gonogo":                         # the LAST verdict line decides, and the exit code must agree
        at = [i for i, s in enumerate(lines) if VERDICT_RE.match(s)]
        if not at:
            return None, [f"no `=== GO ===` / `=== NO-GO: ... ===` line (exit {rc})"]
        own = VERDICT_RE.match(lines[at[-1]]).group(1)
        if (own == "GO") != (rc == 0):
            return None, [f"the verdict says {own.split(':')[0]} but the exit code is {rc}"]
        notes = [] if own == "GO" else [f"its own verdict {own}"]
        if sc.get("count"):                      # the case count, read from the output
            found = list(re.finditer(sc["count"], text, re.M))
            if not found:
                return None, ["`count` matched nothing"]
            n = int(found[-1].group("n"))
            return ((n, 0, 0) if own == "GO" else (0, max(n, 1), 0)), notes
        above = next((s for s in reversed(lines[:at[-1]]) if s), "")
        found = COUNTS_RE.search(above)          # `N passed, M failed, K skipped` directly above the verdict -
        if found:                                # a counts line further up is some other run's, quoted
            return tuple(int(g) for g in found.groups()), notes
        return ((1, 0, 0) if own == "GO" else (0, 1, 0)), notes     # else the command is one case
    if kind == "regex":
        found = list(re.finditer(sc["regex"], text, re.M))
        if not found:
            return None, ["`regex` matched nothing"]
        g = found[-1].groupdict()
        return tuple(int(g.get(k) or 0) for k in ("passed", "failed", "skipped")), []
    hits = [s for s in lines if (JEST_LINE if kind == "jest" else PYTEST_LINE).search(s)]
    if not hits:
        return None, [f"no {kind} summary line"]
    n = {"passed": 0, "failed": 0, "skipped": 0}
    for num, word in PAIR_RE.findall(hits[-1]):
        n["failed" if word.startswith("error") else "skipped" if word == "todo" else word] += int(num)
    return (n["passed"], n["failed"], n["skipped"]), []


def port_open(port):
    with contextlib.suppress(OSError):
        with socket.create_connection(("127.0.0.1", int(port)), timeout=0.5):
            return True
    return False


def argv_of(cmd, man, root, case=None):
    """shlex (POSIX rules) splits the command - no shell, so nothing expands; {python} is this interpreter,
    a manifest var its first existing candidate (never symlink-resolved: a venv is known by the path it is
    invoked through), {case} the case id."""
    values = {"python": sys.executable}
    for key, cands in ({} if "vars" in top_errors(man) else man.get("vars") or {}).items():
        values[key] = next((os.path.abspath(str(Path(root) / c)) for c in cands if (Path(root) / c).is_file()), None)
    toks = []
    for tok in shlex.split(cmd):
        for key, val in values.items():
            if "{" + key + "}" in tok:
                if val is None:
                    raise ValueError(f"{{{key}}}: none of {man['vars'][key]} exists")
                tok = tok.replace("{" + key + "}", val)
        toks.append(tok.replace("{case}", case) if case is not None else tok)
    if not toks:
        raise ValueError("an empty command")
    exe = toks[0] if os.path.dirname(toks[0]) else shutil.which(toks[0], path=tool_path())
    if not exe:
        raise ValueError(f"{toks[0]!r} is not on PATH")
    return [exe] + toks[1:]


def tool_path(environ=None):
    """PATH for the commands this harness spawns. Windows (win-laptop, M0-D23): a plain PowerShell's `bash` is
    System32's WSL relay, which has no /bin/bash - so Git's usr/bin (bash and the Unix tools the scripts call) and
    ~/bin (rsync) go first, in the spawned commands only. Elsewhere PATH as is."""
    path = (os.environ if environ is None else environ).get("PATH", os.defpath)
    if os.name != "nt":
        return path
    git = shutil.which("git", path=path)
    roots = [*(Path(git).parents if git else ()), Path(os.environ.get("ProgramFiles", r"C:\Program Files")) / "Git"]
    usr = next((r / "usr" / "bin" for r in roots if (r / "usr" / "bin" / "bash.exe").is_file()), None)
    first = [str(d) for d in (usr, Path.home() / "bin") if d and d.is_dir()]
    return os.pathsep.join(first + [path])


def case_env(task):
    """Every command's environment: the task id, UTF-8 output, and faulthandler on - a native crash (an access
    violation, no traceback) still leaves its Python stack in the log - and PB_PYTHON, this harness's own
    interpreter, so a non-Python case (capture_web.mjs) never spawns whatever `python` the PATH finds first
    (win-laptop, M0-D16: a stdlib-less 3.14 ahead of 3.12). A caller's own value wins."""
    env = dict(os.environ, PB_TASK=task)
    env.setdefault("PYTHONIOENCODING", "utf-8")
    env.setdefault("PYTHONFAULTHANDLER", "1")
    env.setdefault("PB_PYTHON", sys.executable)
    env["PATH"] = tool_path(env)
    return env


def new_group():
    return {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == "nt" else {"start_new_session": True}


def stop_tree(proc):
    """Only the process group THIS run spawned. SIGINT first, so a runner's own teardown (the stack it
    booted) runs; then SIGKILL. Windows: taskkill /T on the child's own tree."""
    if os.name == "nt":
        subprocess.run(["taskkill", "/PID", str(proc.pid), "/T", "/F"], stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL)
    else:
        for sig, grace in ((signal.SIGINT, 15), (signal.SIGKILL, 5)):
            with contextlib.suppress(OSError):
                os.killpg(proc.pid, sig)
            with contextlib.suppress(subprocess.TimeoutExpired):
                proc.wait(timeout=grace)
                return
    proc.wait()


class ScopeRun:
    """One scope's tally in a run; its verdict is derived from the counts, never stated."""

    def __init__(self, name, spec, expected, log):
        self.name, self.spec, self.expected, self.log = name, spec, expected, Path(log)
        self.passed = self.failed = self.skipped = 0
        self.seconds, self.skip, self.owed, self.notrun, self.reasons = 0.0, None, None, None, []
        self.started = self.ended = None
        self.listed, self.booked, self.case_paths, self.suite = None, False, None, None   # cases that are files
        self.case_now, self.case_out, self.vouched, self.vouched_n, self.pick = {}, {}, 0, 0, None
        self.green = None                                    # (ts, entry) of the record it was already GO in (M0-T64c)
        self.step = None                                     # (its number, the run's steps) when the run has several

    def ran(self):
        return not (self.skip or self.owed or self.notrun or self.green)

    def red(self):
        """The NO-GO reasons, [] when GO - a scope that did not run (placeholder, owed, NOT RUN) is never red
        itself, and never GO: the loop reads NO-GO while any scope is NOT RUN (run_scopes)."""
        if not self.ran():
            return []
        out = list(self.reasons)
        if self.passed == 0 and not out:
            out.append("zero passed")
        got = self.passed + self.failed + self.vouched_n     # read against the cases it LISTED: the anchor's own
        if got < self.expected:                               # counts for the ones it vouched for, never the ran alone
            out.append(f"{got} case(s) < expected {self.expected}"
                       + (f" ({self.vouched_n} vouched by the anchor)" if self.vouched_n else ""))
        return out

    def record(self):
        if self.green:
            return {"verdict": "GO", "not_rerun": self.green[0]}
        if self.skip:
            return {"verdict": "SKIP", "reason": self.skip}
        if self.owed:
            return {"verdict": "OWED", "box": self.owed}
        if self.notrun:
            return {"verdict": "NOT RUN", "reason": self.notrun}
        return {"passed": self.passed, "failed": self.failed, "skipped": self.skipped,
                "seconds": round(self.seconds, 3), "verdict": "NO-GO" if self.red() else "GO",
                **({"vouched": self.vouched} if self.vouched else {})}

    def line(self, w, box):
        if self.green:
            return f"{self.name:<{w}}GO (already GO on this tree at {self.green[0]} \u2014 not re-run)"
        if self.skip:
            return f"{self.name:<{w}}SKIP ({self.skip})"
        if self.notrun:
            return f"{self.name:<{w}}NOT RUN ({self.notrun})"
        if self.owed:
            return f"{self.name:<{w}}owed on {self.owed} (this box: {box}) - not run here, never NO-GO"
        return (f"{self.name:<{w}}{f'{self.passed}/{self.failed}/{self.skipped}':<10}{self.seconds:7.2f} s  "
                f"{'NO-GO' if self.red() else 'GO'}")


def run_command(sr, label, cmd, case, root, man, env, task, box):
    """One command. The scope's log gets its header BEFORE the command starts; the output streams into a
    part file beside the log - on disk even if this runner dies - and joins the log as one block when the
    command ends. -> (exit code or None, output, seconds, why it did not run or None, timed out)."""
    spec = sr.spec
    with LOCK:
        PART[0] += 1
        part = Path(f"{sr.log}.{os.getpid()}.{PART[0]}.part")
        with open(sr.log, "a", encoding="utf-8") as fh:
            fh.write(f"== {stamp()} {label} · task {task} · box {box} :: {cmd}\n")
    t0, rc, note, timed_out, out = time.monotonic(), None, None, False, ""
    busy = [p for p in spec.get("free_ports", []) if port_open(p)]
    if busy:
        note = f"NOT RUN (:{busy[0]} answers - a stack this run did not start: never reused, never stopped)"
    else:
        try:
            argv = argv_of(cmd, man, root, case)
        except ValueError as exc:
            note = f"not started: {exc}"
        else:
            with open(part, "wb") as fh:
                try:
                    proc = subprocess.Popen(argv, cwd=str(Path(root) / spec.get("cwd", ".")), env=env,
                                            stdin=subprocess.DEVNULL, stdout=fh, stderr=subprocess.STDOUT,
                                            **new_group())
                except OSError as exc:
                    note = f"not started: {exc}"
                else:
                    try:
                        rc = proc.wait(timeout=float(spec.get("timeout_s", 900)))
                    except subprocess.TimeoutExpired:
                        timed_out = True
                        stop_tree(proc)
            with contextlib.suppress(OSError):
                out = part.read_bytes().decode("utf-8", "replace").replace("\r\n", "\n")
            with contextlib.suppress(OSError):
                part.unlink()                                       # this run's own part file, joined below
    secs = time.monotonic() - t0
    with LOCK, open(sr.log, "a", encoding="utf-8") as fh:
        fh.write(f"-- {label} · exit={'timeout' if timed_out else note or rc} · {secs:.2f} s\n"
                 + out + ("\n" if out and not out.endswith("\n") else "") + "\n")
    return rc, out, secs, note, timed_out


def run_unit(sr, label, cmd, case, root, man, env, task, box):
    rc, out, secs, note, timed_out = run_command(sr, label, cmd, case, root, man, env, task, box)
    counts, notes = (None, [note]) if note else (None, []) if timed_out else parse(sr.spec, out, rc)
    reasons = ([f"timed out after {sr.spec.get('timeout_s', 900)} s"] if timed_out else []) + notes
    if counts is not None and counts[1] and sr.spec.get("parse", "gonogo") != "exit":
        reasons.insert(0, f"{counts[1]} failed")
    if rc not in (0, None) and not reasons:
        reasons.append(f"exit {rc}")
    with LOCK:
        p, f, k = counts or (0, 0, 0)
        sr.passed, sr.failed, sr.skipped, sr.seconds = sr.passed + p, sr.failed + f, sr.skipped + k, sr.seconds + secs
        sr.reasons += [r if label == sr.name else f"{label}: {r}" for r in reasons]
        if case is not None:                              # one case's own verdict and count, for the anchor's book
            sr.case_out[case] = (counts is not None and not reasons and p > 0 and not f, p + f)
    say(f".. {label} {secs:.2f}s")


def units(sr, case, root, man, env, task, box):
    """[(label, command, value for {case})] of one scope; sets `skip` or a reason when nothing can run."""
    s, name, cases = sr.spec, sr.name, sr.spec.get("cases")
    if case is not None:
        if isinstance(cases, dict) and case in cases:
            return [(f"{name}[{case}]", cases[case], None)]
        if cases is not None and case not in cases:
            sr.reasons.append(f"no case `{case}` - its cases: {', '.join(cases)}")
        elif "case_cmd" in s:
            return [(f"{name}[{case}]", s["case_cmd"], case)]
        else:
            sr.reasons.append("this scope has no cases: no `cases`, no `case_cmd`")
        return []
    if isinstance(cases, dict):
        return [(f"{name}[{c}]", cmd, None) for c, cmd in cases.items()]
    if isinstance(cases, list):
        return [(f"{name}[{c}]", s["case_cmd"], c) for c in cases]
    if "cases_cmd" in s:                                 # cases from files present: the command lists the ids
        rc, out, secs, note, timed_out = run_command(sr, f"{name} (cases_cmd)", s["cases_cmd"], None, root, man,
                                                     env, task, box)
        sr.seconds += secs
        found = [c.strip() for c in out.splitlines() if c.strip()]
        bad = [c for c in found if not ID_RE.match(c)]
        if note or timed_out or rc != 0:
            sr.reasons.append(f"`cases_cmd` {note or ('timed out' if timed_out else f'exit {rc}')}")
        elif bad:
            sr.reasons.append(f"`cases_cmd` printed a bad id {bad[0]!r}")
        elif not found and not sr.expected:
            sr.skip = "no case present"
        else:
            sr.listed = found
        return [] if sr.reasons or sr.skip else [(f"{name}[{c}]", s["case_cmd"], c) for c in found]
    return [(name, s["cmd"], None)]


# ---------------------------------------------------------------------- content, git, the tree a run graded
def normalised(b):
    """CRLF read as LF on TEXT, binary untouched (git's own rule: a NUL in the first 8000 bytes). git hands
    each box different bytes for one commit - a text file is CRLF on a box with autocrlf - so a hash of raw
    bytes records which box read the file, never what was verified. The cost, stated: a text file whose
    line endings alone moved hashes the same; a real content change still moves the hash."""
    return b if b"\0" in b[:8000] else b.replace(b"\r\n", b"\n")


def content_sha1(b):
    return hashlib.sha1(normalised(b)).hexdigest()


def sha1_of(root, relpath):
    """The path's content hash, or None when it cannot be read - and None never equals a recorded hash, so
    an unreadable covered path reads STALE, never checked."""
    with contextlib.suppress(OSError):
        return content_sha1((Path(root) / relpath).read_bytes())[:16]
    return None


def git_out(root, args, timeout=120):
    """Read-only git, or None when git cannot answer - refused upstream, never read as `clean`."""
    try:
        out = subprocess.run(["git", "--no-optional-locks"] + list(args), cwd=str(root), capture_output=True,
                             timeout=timeout)
    except (OSError, subprocess.SubprocessError):
        return None
    return None if out.returncode else out.stdout.decode("utf-8", "replace")


def held_out(root, man, logs):
    """The two paths every run rewrites: the logs folder and the coverage anchor - bookkeeping about the
    tree, never a surface a scope grades."""
    ap = anchor_path(man, root)
    return rel(logs, root).rstrip("/") + "/", rel(ap, root) if ap is not None else None


def is_held(p, held):
    logs, anchor = held
    return p == anchor or p.startswith(logs)


def extra_files(root, man, logs):
    """Every file under the manifest's `tree_extra` (ignored paths the scopes read), any folder named
    `logs` left out - the runner writes there. None when `tree_extra` is no list of paths (NOT RUN)."""
    if "tree_extra" in top_errors(man):
        return None
    out, logs = [], os.path.abspath(str(logs))
    for p in man.get("tree_extra") or ():
        full = Path(root) / p
        if full.is_file():
            out.append(rel(full, root))
        for d, dirs, files in os.walk(str(full)):
            dirs[:] = sorted(x for x in dirs if x != "logs" and os.path.abspath(os.path.join(d, x)) != logs)
            out += [rel(os.path.join(d, fn), root) for fn in sorted(files)]
    return out


# ------------------------------------------- the tree without git: a walk that reads .gitignore as git does
POSIX_CLASSES = {"alnum": "0-9A-Za-z", "alpha": "A-Za-z", "digit": "0-9", "lower": "a-z", "upper": "A-Z",
                 "space": " \\t\\n\\r\\f\\v", "xdigit": "0-9A-Fa-f", "blank": " \\t", "punct": "!-/:-@\\[-`{-~",
                 "word": "0-9A-Za-z_", "print": " -~", "graph": "!-~", "cntrl": "\\x00-\\x1f\\x7f"}


def ignore_class(pat, i):
    """`[...]` at pat[i] -> (regex, index after it), or None when no `]` closes it (then `[` is a literal)."""
    j, neg = i + 1, False
    if j < len(pat) and pat[j] in "!^":
        neg, j = True, j + 1
    body, first = "", True
    while j < len(pat):
        c = pat[j]
        if c == "]" and not first:
            return "[" + ("^" if neg else "") + (body or "") + "]", j + 1
        first = False
        if c == "[" and pat.startswith("[:", j) and ":]" in pat[j:]:
            k = pat.index(":]", j)
            body, j = body + POSIX_CLASSES.get(pat[j + 2:k], ""), k + 2
        elif c == "\\" and j + 1 < len(pat):
            body, j = body + re.escape(pat[j + 1]), j + 2
        else:
            body, j = body + (re.escape(c) if c in "\\^[" else c), j + 1
    return None


def ignore_regex(pat):
    """One .gitignore glob (slashes in, no trailing slash) -> its regex: `*` and `?` stay inside a segment,
    `**` crosses them only as a whole segment (leading `**/`, middle `/**/`, trailing `/**`)."""
    out, i, n = [], 0, len(pat)
    while i < n:
        c = pat[i]
        if c == "*":
            j = i
            while j < n and pat[j] == "*":
                j += 1
            whole = j - i >= 2 and (i == 0 or pat[i - 1] == "/") and (j == n or pat[j] == "/")
            if whole and j == n:
                out.append(".*")
            elif whole:
                out.append("(?:.*/)?")
                j += 1
            else:
                out.append("[^/]*")
            i = j
        elif c == "?":
            out.append("[^/]")
            i += 1
        elif c == "[":
            cls = ignore_class(pat, i)
            out.append(cls[0] if cls else re.escape(c))
            i = cls[1] if cls else i + 1
        elif c == "\\" and i + 1 < n:
            out.append(re.escape(pat[i + 1]))
            i += 2
        else:
            out.append(re.escape(c))
            i += 1
    return re.compile("".join(out) + r"\Z", re.S)


def ignore_rules(text):
    """The rules of one .gitignore (or .git/info/exclude), in file order: (regex, negated, dir_only, anchored).
    Blank lines and `#` comments skipped, trailing spaces dropped unless escaped, a leading `!` negates,
    a trailing `/` is directories only, a `/` first or inside the pattern anchors it to the file's folder."""
    rules = []
    for line in text.lstrip("\ufeff").split("\n"):
        line = line.rstrip("\r")
        while line.endswith(" ") and not line.endswith("\\ "):
            line = line[:-1]
        if not line or line.startswith("#"):
            continue
        neg = line.startswith("!")
        pat = line[1:] if neg else line
        dir_only = pat.endswith("/")
        pat = pat.rstrip("/") if dir_only else pat
        if not pat:
            continue
        anchored = "/" in pat
        rules.append((ignore_regex(pat.lstrip("/")), neg, dir_only, anchored))
    return rules


def read_rules(path):
    """The rules of the file at `path`, [] when there is none; an unreadable one raises (never a guess)."""
    if not os.path.isfile(path):
        return []
    with open(path, "rb") as fh:
        return ignore_rules(fh.read().decode("utf-8", "replace"))


def ignored(sources, rel_path, is_dir):
    """git's verdict on `rel_path`: the nearest source (deepest .gitignore first, `.git/info/exclude` last)
    with a matching rule decides, and in it the LAST matching rule."""
    for prefix, rules in sources:
        sub = rel_path[len(prefix):]
        base = sub.rsplit("/", 1)[-1]
        for rx, neg, dir_only, anchored in reversed(rules):
            if (is_dir or not dir_only) and rx.match(sub if anchored else base):
                return not neg
    return False


def global_rules():
    """The user's global excludes file, read the way git finds it - `core.excludesFile` named in
    `~/.gitconfig` or `$XDG_CONFIG_HOME/git/config`, else `$XDG_CONFIG_HOME/git/ignore` (default
    `~/.config/git/ignore`) - so the walk lists what `git ls-files -co --exclude-standard` lists on this box.
    A file, never a git call; [] when none."""
    cfg = os.environ.get("XDG_CONFIG_HOME") or os.path.join(os.path.expanduser("~"), ".config")
    path = os.path.join(cfg, "git", "ignore")
    for conf in (os.path.join(os.path.expanduser("~"), ".gitconfig"), os.path.join(cfg, "git", "config")):
        with contextlib.suppress(OSError):
            m = re.search(r"(?mi)^[ \t]*excludesfile[ \t]*=[ \t]*(\S.*?)[ \t]*$", Path(conf).read_text("utf-8", "replace"))
            if m:
                path = os.path.expanduser(m.group(1).strip("\""))
    return read_rules(path)


def repo_top(root):
    """`(top, up)` for `root`: the nearest folder at or above it holding a `.git` entry (found without git) and
    the `/`-separated path from `top` down to `root` with a trailing `/` (empty at the top itself); `(root, "")`
    when no folder above holds one, so a tree with no repository is walked as its own top."""
    root = os.path.abspath(os.fspath(root))
    here, parts = root, []
    while not os.path.lexists(os.path.join(here, ".git")):
        parent = os.path.dirname(here)
        if parent == here:
            return root, ""
        parts.insert(0, os.path.basename(here))
        here = parent
    return here, "".join(x + "/" for x in parts)


def walk_tree(root):
    """The relative `/`-separated paths, sorted, of every file under `root` that git would show as tracked or
    untracked-not-ignored: every `.gitignore` in the tree read as git reads it - the folders from the
    repository's top down to `root` too, each rule relative to its own folder, so a `root` below its
    repository's top (a track) lists what git lists there - `.git/info/exclude` of the top where it exists, an
    ignored folder never entered (a `root` an ignored folder holds lists nothing), `.git` and a folder holding
    its own `.git` (a nested repository) never entered. The user's global excludes file is read last, as git
    reads it. No git call. A folder or `.gitignore` that cannot be read raises OSError - a list that silently
    lacks files certifies nothing. (Files git tracks although a `.gitignore` names them are not listed.)"""
    top, up = repo_top(root)
    out = []
    last = [("", r) for r in (read_rules(os.path.join(top, ".git", "info", "exclude")), global_rules()) if r]
    local, folder, prefix = [], top, ""
    for name in up.split("/")[:-1]:                       # the folders above `root`: their rules, and are they ignored
        rules = read_rules(os.path.join(folder, ".gitignore"))
        local = [(prefix, rules)] + local if rules else local
        folder, prefix = os.path.join(folder, name), prefix + name + "/"
        if ignored(local + last, prefix[:-1], True):
            return []

    def visit(folder, prefix, local):
        with os.scandir(folder) as it:
            entries = sorted(it, key=lambda e: e.name)
        rules = read_rules(os.path.join(folder, ".gitignore"))
        local = [(prefix, rules)] + local if rules else local
        sources = local + last
        for e in entries:
            rel_path = prefix + e.name
            if e.is_dir(follow_symlinks=False):
                if e.name == ".git" or os.path.lexists(os.path.join(e.path, ".git")) \
                        or ignored(sources, rel_path, True):
                    continue
                visit(e.path, rel_path + "/", local)
            elif os.path.isfile(e.path) and not ignored(sources, rel_path, False):
                out.append(rel_path[len(up):])

    visit(os.fspath(root), up, local)
    return sorted(out)


def tree_id(root, man, logs, git=git_out):
    """`[<HEAD 12>+]<sha1 12>`: the sha1 runs over every file the walk lists (and every file under
    `tree_extra`), each as its path and the hash of its content - CRLF read as LF, so one commit's two
    checkouts name ONE tree, and a second edit to an already-edited file moves the id. Held out: the logs
    folder and the anchor (every run rewrites them). `<HEAD 12>+` leads only where a repository answers; no
    repository, no prefix, the same hash. None when a listed file cannot be read, or `tree_extra` cannot:
    two unknown trees are not one tree."""
    paths = visible_files(root, man, logs)
    if paths is None:
        return None
    digest = hashlib.sha1()
    for p in paths:
        try:
            body = (Path(root) / p).read_bytes()
        except OSError:
            return None
        digest.update(p.encode("utf-8", "surrogateescape") + b"\0" + content_sha1(body).encode("ascii") + b"\n")
    head = git(root, ["rev-parse", "HEAD"], 30)
    return (head.strip()[:12] + "+" if head and head.strip() else "") + digest.hexdigest()[:12]


def tracked(root, paths):
    """Which of `paths` git already tracks - somebody's committed evidence. Empty outside a work tree."""
    names = [rel(p, root) for p in paths]
    out = git_out(root, ["ls-files", "-z", "--"] + names, 30) if names else ""
    got = set((out or "").split("\0"))
    return [p for p, r in zip(paths, names) if r in got]


def refuse_log(root, paths, tracked_fn=None):
    """Why this run must not open these logs, or None: committed evidence is never appended to."""
    hit = (tracked_fn or tracked)(root, paths)
    if hit:
        return (f"NOT RUN ({', '.join(p.name for p in hit)}: git already tracks it - another task's evidence "
                f"is never appended to; pass --task <YOUR-TASK-ID>)")
    return None


# ------------------------------------------------ the coverage anchor: WHICH PATHS a green run covered
# A narrower check is only a check if something durable records what was verified and up to what point,
# which the check ADVANCES, and which makes "this path has never been checked" a state the tool prints and
# goes NO-GO on. Every visible path lands in exactly one class - covered (a scope's `paths` match it),
# skipped (a declared class says why no scope grades it), or UNCLASSIFIED, which is NO-GO.
def anchor_path(man, root):
    """The anchor's file - None when `anchor` is no path this runner can use (NOT RUN: never advanced)."""
    return None if "anchor" in top_errors(man) else Path(root) / man.get("anchor", ANCHOR_DEFAULT)


def anchor_off(man):
    """Why the coverage anchor is NOT RUN (its `anchor`, `tree_extra` or `skip_classes` unreadable), or None."""
    parts = top_errors(man)
    return next((parts[k] for k in ("anchor", "tree_extra", "skip_classes") if k in parts), None)


def globs(pats):
    """One matcher for a pattern list, or None. `*` and `?` stay INSIDE a segment and `**` crosses them:
    fnmatch's `*` crosses `/`, which would let one careless pattern swallow a whole root and read as coverage."""
    srcs = []
    for pat in pats or ():
        out, i = "", 0
        while i < len(pat):
            if pat.startswith("**/", i):
                out, i = out + "(?:[^/]+/)*", i + 3
            elif pat.startswith("**", i):
                out, i = out + ".*", i + 2
            elif pat[i] in "*?":
                out, i = out + ("[^/]*" if pat[i] == "*" else "[^/]"), i + 1
            else:
                out, i = out + re.escape(pat[i]), i + 1
        srcs.append(f"(?:{out})\\Z")
    return re.compile("|".join(srcs)) if srcs else None


def visible_files(root, man, logs):
    """The anchor's subject: every file `walk_tree` lists (tracked, and untracked but not ignored - a new file
    is part of the tree the moment it exists) plus every file under `tree_extra`, the held-out two excepted.
    None when the tree or `tree_extra` cannot be read: a gate that cannot name its subject certifies
    nothing. No git."""
    extra = extra_files(root, man, logs)
    try:
        listed = walk_tree(root)
    except OSError:
        return None
    if extra is None:
        return None
    held = held_out(root, man, logs)
    paths = set(listed) | set(extra)
    return sorted(p for p in paths if not is_held(p, held) and (Path(root) / p).is_file())


def coverage(man, paths):
    """(covered {path: [scope...]}, skipped {path: reason}, unclassified [path...]); a placeholder or a NOT RUN
    scope covers nothing."""
    bl = blocked(man)
    scopes = [(n, globs(sc.get("paths"))) for n, sc in (man.get("scopes") or {}).items()
              if n not in bl and "placeholder" not in sc]
    classes = [] if "skip_classes" in top_errors(man) else [(globs(c.get("match")), c.get("reason") or "?")
                                                             for c in man.get("skip_classes") or []]
    covered, skipped, loose = {}, {}, []
    for p in paths:
        hit = [n for n, rx in scopes if rx is not None and rx.match(p)]
        if hit:
            covered[p] = hit
            continue
        why = next((r for rx, r in classes if rx is not None and rx.match(p)), None)
        if why:
            skipped[p] = why
        else:
            loose.append(p)
    return covered, skipped, loose


def anchor_read(path):
    """A missing anchor is an EMPTY one - nothing has been checked. An unreadable one is REFUSED: an anchor
    nobody can read is exactly the check that reads green. One at another version is dropped whole: an
    entry at a hash or a shape this build no longer computes can be neither trusted nor translated."""
    p = Path(path)
    fresh = {"version": ANCHOR_VERSION, "note": ANCHOR_NOTE, "paths": {}, "superseded": None}
    if not p.exists():
        return fresh
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise Refused(f"{p.name}: {exc} - an anchor that cannot be read is not an anchor")
    if not isinstance(data, dict) or not isinstance(data.get("paths"), dict):
        raise Refused(f"{p.name}: no `paths` object - an anchor that cannot be read is not an anchor")
    if not isinstance(data.get("cases", {}), dict) or not all(isinstance(v, dict) for v in data.get("cases", {}).values()):
        raise Refused(f"{p.name}: `cases` is no object of scope -> cases - an anchor that cannot be read is not an anchor")
    if data.get("version") != ANCHOR_VERSION:
        return dict(fresh, superseded=data.get("version"))
    return dict(data, superseded=None)


def replace_retry(tmp, dst, attempts=6):
    """os.replace, retried while another process still holds the target open (Windows)."""
    for i in range(attempts):
        try:
            os.replace(str(tmp), str(dst))
            return
        except PermissionError:
            if i == attempts - 1:
                raise
            time.sleep(0.05 * 2 ** i)


def anchor_write(path, data):
    """Atomic, one line per path: a large advance stays a readable diff, and a crash mid-write never leaves
    half a record where a whole one is claimed."""
    p = Path(path)
    rows = ",\n".join(f"    {json.dumps(k)}: {json.dumps(v, sort_keys=True)}"
                      for k, v in sorted((data.get("paths") or {}).items()))
    books = ",\n".join(f"    {json.dumps(s)}: {{\n" + ",\n".join(f"      {json.dumps(c)}: {json.dumps(e, sort_keys=True)}"
                                                       for c, e in sorted(ents.items())) + "\n    }"
                       for s, ents in sorted((data.get("cases") or {}).items()) if ents)
    text = ('{\n  "version": ' + json.dumps(ANCHOR_VERSION) + ',\n  "note": ' + json.dumps(ANCHOR_NOTE)
            + ',\n  "paths": {\n' + rows + ("\n" if rows else "") + "  }"
            + (',\n  "cases": {\n' + books + "\n  }" if books else "") + "\n}\n")
    tmp = p.with_name(p.name + ".tmp")
    tmp.write_text(text, encoding="utf-8", newline="\n")
    replace_retry(tmp, p)


def anchor_status(man, root, anchor, paths, box=None):
    """Every visible path in exactly one class, and WHY. `checked` means every scope that covers it holds an
    entry at the bytes on disk RIGHT NOW and at the scope's current definition: ONE stale covering scope is
    enough to un-check a path. A path whose only unchecked scopes run on another box is `owed`, never NO-GO.
    `by_scope` says, per scope, what it owes; `gone` its entries for a path that left the tree."""
    box, scopes = box or this_box(), man.get("scopes") or {}
    covered, skipped, loose = coverage(man, paths)
    have, checked, never, stale, owed = anchor.get("paths") or {}, [], [], [], []
    by_scope = {n: {"never": [], "stale": [], "checked": 0, "gone": []} for n in scopes}
    for p, scs in sorted(covered.items()):
        now, ent = sha1_of(root, p), have.get(p) or {}
        missing = [s for s in scs if s not in ent]
        drift = [s for s in scs if s in ent and (ent[s].get("sha1") != now or ent[s].get("spec") != spec_hash(scopes[s]))]
        for s in scs:
            by_scope[s]["never" if s in missing else "stale" if s in drift else "checked"] += (
                [p] if s in missing or s in drift else 1)
        if not missing and not drift:
            checked.append(p)
        elif all(owed_on(scopes[s], box) for s in missing + drift):
            owed.append(p)
        elif missing:
            never.append((p, missing, drift))
        else:
            was = ent[drift[0]].get("sha1") or "?"
            stale.append((p, drift, (was, now or "unreadable")))
    seen = set(paths)
    for p, ent in have.items():
        for s in ent:
            if s in by_scope and p not in seen:
                by_scope[s]["gone"].append(p)
    return {"covered": covered, "skipped": skipped, "loose": loose, "checked": checked, "never": never,
            "stale": stale, "owed": owed, "tracked": len(paths), "by_scope": by_scope,
            "superseded": anchor.get("superseded")}


def anchor_snapshot(man, root, names, paths):
    """The bytes the scopes about to run will SEE, hashed BEFORE anything is spawned. An edit made mid-run
    then leaves the anchor at the pre-run hash and the path reads stale - never the other way round, which
    would anchor bytes no scope graded."""
    covered, _sk, _lo = coverage(man, paths)
    want = set(names)
    return {p: sha1_of(root, p) for p, scs in covered.items() if want.intersection(scs)}


def anchor_advance(man, root, green, red, snapshot, rec, path, paths=None):
    """ONLY the scopes that went GO gain entries, and ONLY at the bytes they ran against; an entry already at
    those bytes and that definition is left alone, so a no-op re-run writes nothing. A scope that went red
    LOSES its entries - its earlier GO says nothing about the bytes it just failed on, so the next --changed
    chooses it again; a green scope loses its entries for paths it no longer covers among the visible
    `paths` (deleted, or out of its globs). -> entries moved."""
    data, moved, scopes = anchor_read(path), 0, man["scopes"]
    covered, _sk, _lo = coverage(man, sorted(snapshot if paths is None else paths))
    entries = data.setdefault("paths", {})
    for p, scs in covered.items():
        if snapshot.get(p) is None:
            continue
        for s in scs:
            ent = (entries.get(p) or {}).get(s) or {}
            if s not in green or (ent.get("sha1") == snapshot[p] and ent.get("spec") == spec_hash(scopes[s])):
                continue
            entries.setdefault(p, {})[s] = {"sha1": snapshot[p], "spec": spec_hash(scopes[s]), "tree": rec.get("tree"),
                                            "task": rec.get("task"), "ts": rec.get("ts"), "box": rec.get("box")}
            moved += 1
    for p in list(entries):
        for s in list(entries[p]):
            if s in red or (s in green and s not in covered.get(p, [])):
                del entries[p][s]
                moved += 1
        if not entries[p]:
            del entries[p]
    if moved or data.get("superseded") is not None:
        anchor_write(path, data)
    return moved


def anchor_lines(st):
    """(the counts line, one line per bad path, one per skipped CLASS with its reason) - so a run that covered
    nothing can never read like a run that covered everything and found it clean."""
    by_class = {}
    for why in st["skipped"].values():
        by_class[why] = by_class.get(why, 0) + 1
    lines = ([f"RECORD SUPERSEDED  version {st['superseded']} -> {ANCHOR_VERSION}: every entry dropped - every "
              f"covered path reads NEVER CHECKED until its own scope re-runs"] if st["superseded"] is not None else [])
    lines += [f"NEVER CHECKED  {p}  (no entry for {', '.join(miss)}" + (f"; stale for {', '.join(dr)}" if dr else "")
              + ")" for p, miss, dr in st["never"]]
    lines += [f"STALE          {p}  (graded at {was}, on disk {now}, or its scope's definition moved: {', '.join(scs)})"
              for p, scs, (was, now) in st["stale"]]
    lines += [f"UNCLASSIFIED   {p}  (no scope's `paths` match it and no skip class names it - declare one)"
              for p in st["loose"]]
    lines += [f"skipped {n:>5}  {why}" for why, n in sorted(by_class.items(), key=lambda kv: (-kv[1], kv[0]))]
    counts = (f"anchor: {len(st['checked'])} paths checked · {len(st['skipped'])} skipped ({len(by_class)} classes) · "
              f"{len(st['never'])} never checked · {len(st['stale'])} stale · {len(st['loose'])} unclassified · "
              f"{len(st['owed'])} owed · {len(st['covered'])} covered of {st['tracked']} visible")
    return counts, lines


def anchor_nogo(st):
    bad = len(st["never"]) + len(st["stale"]) + len(st["loose"])
    if bad:
        return (f"{len(st['never'])} never checked, {len(st['stale'])} stale, {len(st['loose'])} unclassified - "
                f"every one named above")
    if not st["checked"] and not st["owed"]:
        return "zero paths checked - a gate that covered nothing is not a clean sweep"
    return None


# ----------------------------------------------- --changed: the scopes a changed path touches, and the gate
def resolve_base(root, base, git=git_out):
    """A named commit, never HEAD: HEAD moves with every commit, so a base read from it forgets what the
    last commit carried. -> the commit's sha."""
    if re.match(r"(?i)^(?:head|@)(?:$|[~^@{])", base.strip()) or base.strip() == "@":
        raise Refused(f"--base {base}: the base is a named commit (a sha or a tag), never HEAD - HEAD moves with "
                      f"every commit and forgets what it carried")
    sha = git(root, ["rev-parse", "--verify", "--quiet", base + "^{commit}"], 30)
    if not sha or not sha.strip():
        raise Refused(f"--base {base}: not a commit this checkout knows")
    return sha.strip()


def moved_since(root, sha, git=git_out):
    d, u = git(root, ["diff", "--name-only", "-z", sha, "--"]), git(root, ["ls-files", "-o", "--exclude-standard", "-z"])
    if d is None or u is None:
        raise Refused(f"git could not diff against {sha[:12]} - --changed cannot name what moved")
    return {p for p in (d + "\0" + u).split("\0") if p}


def choose(man, st, moved, box, base=None):
    """(chosen, why {scope: ground}, not_run [(scope, ground)]): every scope with no `paths` (it cannot be
    narrowed), every scope owning a never-checked, stale or vanished path, and - with --base - every scope a
    path moved since the base matches. Placeholders, owed and NOT RUN scopes stay in, to print their own line."""
    chosen, why, not_run, bl = [], {}, [], blocked(man)
    for n, s in man["scopes"].items():
        if n in bl or "placeholder" in s or owed_on(s, box):
            chosen.append(n)
            continue
        if not s.get("paths"):
            chosen.append(n)
            why[n] = "declares no `paths`: it cannot be narrowed, so it always runs"
            continue
        bs, rx = st["by_scope"][n], globs(s["paths"])
        grounds = [(k, bs[k]) for k in ("never", "stale", "gone") if bs[k]]
        hit = sorted(p for p in moved if rx.match(p))
        if hit:
            grounds.append((f"moved since {base}", hit))
        if grounds:
            chosen.append(n)
            why[n] = " · ".join(f"{k}: {ps[0]}" + (f" (+{len(ps) - 1})" if len(ps) > 1 else "") for k, ps in grounds)
        else:
            not_run.append((n, f"not impacted: its {bs['checked']} path(s) checked at the anchor"
                            + (f", none moved since {base}" if base else "")))
    return chosen, why, not_run


# ------------------------- a scope whose cases are files: under --changed, only the cases whose file moved
# PLAYBOOK §A.2 (v12.3, M0-T64b). Beside its paths the anchor keeps, per scope whose `cases_cmd` lists files,
# each case's content hash at its last GO, the scope's SUITE hash then - its manifest entry, every file its case
# commands name, every file under `tree_extra` - and the count the case reported. A case runs again when its
# file is new or moved, or it never went GO; EVERY case runs when the suite moved (a rule that reads across
# files - a denylist, a shared config - is the suite's), a case id is no file, or a covered path the anchor no
# longer vouches for is no case: the case-level skip is never wider than the anchor can prove.
def case_paths(s, ids, visible, held=None, root=None):
    """{case id: its path from the root} when every listed id names a visible file (read from the scope's
    `cwd`) or a file the runner holds out of the tree (the logs folder, the anchor: a log can leak a value, so a
    gate lists it, and its content hash vouches for it like any other), else None: a case that is no file has no
    bytes the anchor could vouch for."""
    vis, out = set(visible or ()), {}
    for c in ids:
        p = posixpath.normpath(posixpath.join(str(s.get("cwd", ".")).replace("\\", "/"), c))
        if p not in vis and not (held and root is not None and is_held(p, held) and (Path(root) / p).is_file()):
            return None
        out[c] = p
    return out


def named_files(man, root, s):
    """The files of the tree the scope's `cases_cmd` and `case_cmd` name - a `bash -c "..."` read inside, an
    `--opt=<file>` after its `=`, a manifest var as the candidate it runs - each from the root. A folder is
    named too, but as the place the cases are listed from, never as a file every case reads."""
    root = Path(root).resolve()
    here = root / str(s.get("cwd", "."))
    values = {k: next((str(root / c) for c in cands if (root / c).is_file()), None)
              for k, cands in ({} if "vars" in top_errors(man) else man.get("vars") or {}).items()}
    out = set()

    def visit(text, depth):
        try:
            toks = shlex.split(text)
        except ValueError:
            return
        for tok in toks:
            if depth < 3 and any(ch.isspace() for ch in tok):
                visit(tok, depth + 1)
            for piece in {tok, tok.split("=", 1)[-1]}:
                for k, v in values.items():
                    piece = piece.replace("{" + k + "}", v) if v else piece
                if not piece or piece.startswith("-") or PLACEHOLDER_RE.search(piece):
                    continue
                with contextlib.suppress(ValueError, OSError):
                    full = Path(os.path.normpath(str(here / piece)))
                    if full.is_file():
                        out.add(full.relative_to(root).as_posix())
    for cmd in (s.get("cases_cmd"), s.get("case_cmd")):
        if isinstance(cmd, str):
            visit(cmd, 0)
    return out


def suite_hash(man, name, root, logs):
    """What every case of the scope reads beside its own file, as one hash: its manifest entry (notes aside),
    every file its case commands name, every file under `tree_extra`. None when `tree_extra` cannot be read:
    then no case is vouched for."""
    s, extra = man["scopes"][name], extra_files(root, man, logs)
    if extra is None:
        return None
    parts = [["entry", json.dumps({k: v for k, v in s.items() if not is_note(k)}, sort_keys=True)]]
    parts += [[p, file_sha256(Path(root) / p) or "absent"] for p in sorted(named_files(man, root, s) | set(extra))]
    return hashlib.sha256(json.dumps(parts).encode("utf-8")).hexdigest()[:16]


def case_vouched(ent, now, suite):
    """The anchor vouches for a case: an entry at the bytes on disk now, under the suite as it is now."""
    return (isinstance(ent, dict) and now is not None and ent.get("sha1") == now and ent.get("suite") == suite
            and isinstance(ent.get("n"), int) and not isinstance(ent.get("n"), bool))


def case_pick(us, cases, now, suite, kept, owes):
    """-> (the units a --changed run runs, its line, the count the vouched cases stand for). `us` the scope's units
    (label, command, case), `cases` {id: path} or None, `now` {id: content hash} taken before any case ran,
    `kept` the anchor's entries for the scope, `owes` the covered paths it no longer vouches for (never checked,
    stale, gone). Never zero cases: every case vouched for, yet the scope chosen, runs them all."""
    n = len(us)

    def whole(why):
        return us, f"cases: {n} of {n} (all: {why})", 0
    if cases is None:
        return whole("a case id is no file of the tree")
    if suite is None:
        return whole("its suite cannot be hashed - `tree_extra` unreadable")
    if any(isinstance(kept.get(c), dict) and kept[c].get("suite") != suite for c in cases):
        return whole("its manifest entry, a file its case commands name or a file under `tree_extra` moved")
    loose = sorted(set(owes) - set(cases.values()))
    if loose:
        return whole(f"{loose[0]} moved and is no case" + (f" (+{len(loose) - 1})" if len(loose) > 1 else ""))
    run = [u for u in us if not case_vouched(kept.get(u[2]), now.get(u[2]), suite)]
    if not run:
        return whole("the anchor vouches for every case, yet the scope was chosen")
    ran = {u[2] for u in run}
    if len(run) == n:
        return run, f"cases: {n} of {n} (none vouched by the anchor yet)", 0
    return run, f"cases: {len(run)} of {n} (the rest vouched by the anchor)", sum(
        (kept.get(c) or {}).get("n", 0) for c in cases if c not in ran)


def suite_chosen(man, root, logs, book, chosen, why, not_run):
    """A scope left out because no covered path moved is chosen still when its case entries stand on a suite
    that moved since: a denylist outside its `paths` reddens every case, none of whose files moved."""
    keep = []
    for n, g in not_run:
        kept = book.get(n) or {}
        h = suite_hash(man, n, root, logs) if kept and "cases_cmd" in man["scopes"][n] else None
        if kept and "cases_cmd" in man["scopes"][n] and any(e.get("suite") != h for e in kept.values()):
            chosen, why[n] = chosen + [n], "its suite moved since its cases last went GO (its entry, a file its case commands name, or tree_extra)"
        else:
            keep.append((n, g))
    return [n for n in man["scopes"] if n in chosen], why, keep


def cases_advance(path, runs, rec):
    """The case entries of each booked scope: a case that went GO gains its entry at the bytes it ran against,
    under the suite it ran under, with the count it reported; one that went red loses it, and so does a case no
    longer listed. A scope whose ids are no files, or whose suite cannot be hashed, keeps none. -> entries moved."""
    data, moved = anchor_read(path), 0
    book = data.setdefault("cases", {})
    for sr in runs:
        if sr.case_paths is None or sr.suite is None:
            moved += len(book.pop(sr.name, None) or {})
            continue
        ent = book.setdefault(sr.name, {})
        for c in [c for c in ent if c not in sr.case_paths]:
            del ent[c]
            moved += 1
        for c, (ok, cnt) in sorted(sr.case_out.items()):
            new = {"sha1": sr.case_now.get(c), "suite": sr.suite, "n": cnt}
            if ok and new["sha1"] is not None:
                if {k: (ent.get(c) or {}).get(k) for k in new} != new:
                    ent[c] = dict(new, task=rec.get("task"), ts=rec.get("ts"), box=rec.get("box"))
                    moved += 1
            elif ent.pop(c, None) is not None:
                moved += 1
        if not ent:
            book.pop(sr.name, None)
    if moved:
        anchor_write(path, data)
    return moved


# ------------------------------------------------------- --redarm: one scratch copy per plant, each NO-GO
def scratch_copy(src, dest):
    """Annex §A.7's copy, in the standard library: the tree without the folders it names (by name, at any
    depth), `images/` folders kept empty, `.venv` and `node_modules` linked back when they exist. A `*.part`
    file - a running command's output, joined and removed when it ends - is left out: arms side by side
    write theirs beside the logs while another arm copies the tree, and one gone mid-copy fails the copy."""
    src, dest = Path(src), Path(dest)

    def ignore(d, names):
        return set(names) if os.path.basename(d) in EMPTIED else {n for n in names if n in EXCLUDES
                                                                   or n.endswith(".part")}

    shutil.copytree(str(src), str(dest), symlinks=True, ignore=ignore)
    for name in LINKS:
        if (src / name).exists() and not (dest / name).exists():
            with contextlib.suppress(OSError, NotImplementedError):
                os.symlink(str(src / name), str(dest / name), target_is_directory=True)


def diff_path(tok):
    return tok.split("\t")[0].strip()                   # a timestamp after a tab is not part of the path


def read_hunk(lines, i, want_old, want_new):
    """The hunk body from line i: (old text, new text, the next line's index)."""
    old_l, new_l, tag = [], [], " "
    while i < len(lines) and (len(old_l) < want_old or len(new_l) < want_new or lines[i].startswith("\\")):
        t = lines[i]
        if t.startswith("\\"):                          # "\ No newline at end of file": the line above has none
            for side, tags in ((old_l, " -"), (new_l, " +")):
                if tag in tags and side and side[-1].endswith("\n"):
                    side[-1] = side[-1][:-1]
        elif t in ("\n", "\r\n"):                       # an empty context line some tools write bare
            tag = " "
            old_l.append(t)
            new_l.append(t)
        elif t[:1] in (" ", "-", "+"):
            tag = t[0]
            if tag != "+":
                old_l.append(t[1:])
            if tag != "-":
                new_l.append(t[1:])
        else:
            raise ValueError(f"line {i + 1}: {t.rstrip()!r} is not a hunk line")
        i += 1
    return "".join(old_l), "".join(new_l), i


def apply_unified(root, text):
    """A unified diff (`diff -u`, `git diff`) applied to the files under `root`, strictly: every hunk's old
    lines must appear exactly once in the file - no fuzz, no guessed offset. -> the files it changed;
    ValueError names the first hunk that does not apply. A file's CRLF endings are kept."""
    files, cur, lines, i = [], None, text.splitlines(keepends=True), 0
    while i < len(lines):
        ln = lines[i]
        if ln.startswith("--- ") and i + 1 < len(lines) and lines[i + 1].startswith("+++ "):
            old, new = diff_path(ln[4:]), diff_path(lines[i + 1][4:])
            if (old.startswith("a/") or old == "/dev/null") and (new.startswith("b/") or new == "/dev/null"):
                old, new = old[2:] if old != "/dev/null" else old, new[2:] if new != "/dev/null" else new  # git's a/ b/
            cur = {"old": old, "new": new, "hunks": []}
            files.append(cur)
            i += 2
            continue
        m = re.match(r"@@ -\d+(?:,(\d+))? \+\d+(?:,(\d+))? @@", ln)
        if m and cur is not None:
            want = [int(g) if g is not None else 1 for g in m.groups()]
            old, new, i = read_hunk(lines, i + 1, want[0], want[1])
            cur["hunks"].append((old, new, ln.strip()))
            continue
        i += 1
    if not files or not any(f["hunks"] for f in files):
        raise ValueError("no hunk in the patch")
    done = []
    for f in files:
        target = f["new"] if f["new"] != "/dev/null" else f["old"]
        parts = target.replace("\\", "/").split("/")
        if target.startswith("/") or ".." in parts or re.match(r"^[A-Za-z]:", target):
            raise ValueError(f"{target}: a path outside the tree")
        path = Path(root) / target
        body = "" if f["old"] == "/dev/null" else path.read_bytes().decode("utf-8")
        crlf = "\r\n" in body
        for old, new, head in f["hunks"]:
            if crlf:
                old, new = old.replace("\r\n", "\n").replace("\n", "\r\n"), new.replace("\r\n", "\n").replace("\n", "\r\n")
            if not old:
                if body:
                    raise ValueError(f"{target} {head}: a hunk with no old lines only creates a file")
                body = new
                continue
            n = body.count(old)
            if n != 1:
                raise ValueError(f"{target} {head}: its old lines appear {n} times in the file, not once")
            body = body.replace(old, new, 1)
        if f["new"] == "/dev/null":
            path.unlink()                                        # a deletion, in the scratch copy only
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(body.encode("utf-8"))
        done.append(target)
    return done


def plant(arm, root, copy, man, env, log):
    """Plant one bug in the copy: `patch` names a `.patch` / `.diff` file of the tree - a unified diff - or is
    a function: a command run in the copy that changes it. -> None, or why it could not be planted."""
    patch = arm["patch"]
    if patch.endswith((".patch", ".diff")):
        try:
            changed = apply_unified(copy, (Path(root) / patch).read_text(encoding="utf-8"))
        except (OSError, ValueError, UnicodeDecodeError) as exc:
            return f"the patch {patch} does not apply: {exc}"
        with LOCK, open(log, "a", encoding="utf-8") as fh:
            fh.write(f"planted {arm['id']}: {patch} -> {', '.join(changed)}\n")
        return None
    try:
        argv = argv_of(patch, man, copy)
        p = subprocess.run(argv, cwd=str(copy), env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=300)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        return f"the plant's command does not run: {exc}"
    with LOCK, open(log, "a", encoding="utf-8") as fh:
        fh.write(f"planted {arm['id']}: {patch} -> exit {p.returncode}\n"
                 + p.stdout.decode("utf-8", "replace") + p.stderr.decode("utf-8", "replace"))
    return None if p.returncode == 0 else f"the plant's command exited {p.returncode}"


# ------------------------------------- the red-arm proof: plants already red on the same bytes are not re-run
def file_sha256(path):
    """The file's content hash (CRLF read as LF, as `tree_id` reads it), or None when it cannot be read."""
    with contextlib.suppress(OSError):
        return hashlib.sha256(normalised(Path(path).read_bytes())).hexdigest()
    return None


def patch_targets(text):
    """The files a unified diff patches, as `apply_unified` reads its headers (git's a/ b/ dropped); a path
    outside the tree is left out - the patch never applies, so the plant re-runs as an error anyway."""
    lines, out = text.splitlines(), []
    for i, ln in enumerate(lines[:-1]):
        if not (ln.startswith("--- ") and lines[i + 1].startswith("+++ ")):
            continue
        old, new = diff_path(ln[4:]), diff_path(lines[i + 1][4:])
        if (old.startswith("a/") or old == "/dev/null") and (new.startswith("b/") or new == "/dev/null"):
            old, new = old[2:] if old != "/dev/null" else old, new[2:] if new != "/dev/null" else new
        t = (new if new != "/dev/null" else old).replace("\\", "/")
        if t and not (t.startswith("/") or ".." in t.split("/") or re.match(r"^[A-Za-z]:", t)):
            out.append(t)
    return out


def fingerprint(man, name, root, visible, tool=None):
    """What a scope's plants are a function of, as one sha256 (PLAYBOOK §A.2, the red-arm proof): its manifest
    entry (canonical JSON, notes aside, as `scope_changes` reads it), every visible file its `paths` match,
    each `.patch` / `.diff` plant file and every file it patches (outside `paths` too; one it creates reads
    `absent`), the scripts a function plant or the scope's own command names, and this runner. Left out: the
    toolchain, the interpreter, the drivers, the box's GPU - a proof is read on its own box only. None when it
    cannot be told (the tree unlisted, a member unreadable): a proof nobody can recompute proves nothing."""
    s = man["scopes"].get(name)
    if visible is None or not isinstance(s, dict):
        return None
    root, rx = Path(root), globs(s.get("paths"))
    members = {p: False for p in visible if rx is not None and rx.match(p)}          # path -> may be absent
    cmds = list(commands(s))
    for a in s.get("plants") or []:
        patch = a.get("patch") if isinstance(a, dict) else None
        if not isinstance(patch, str):
            continue
        if not patch.endswith((".patch", ".diff")):
            cmds.append(patch)
            continue
        members[patch] = False
        with contextlib.suppress(OSError, UnicodeDecodeError):
            members.update({t: True for t in patch_targets((root / patch).read_text(encoding="utf-8"))
                            if t not in members})
    for cmd in cmds:
        with contextlib.suppress(ValueError):
            members.update({t: False for t in shlex.split(cmd) if t.endswith(CODE_SUFFIXES) and not t.startswith("-")
                            and t not in members and (root / t).is_file()})
    parts = [["entry", json.dumps({k: v for k, v in s.items() if not is_note(k)}, sort_keys=True)]]
    for p in sorted(members):
        h = file_sha256(root / p)
        if h is None and not (members[p] and not (root / p).exists()):
            return None
        parts.append([p, h or "absent"])
    h = file_sha256(Path(tool or __file__))
    return None if h is None else hashlib.sha256(json.dumps(parts + [["runner", h]]).encode("utf-8")).hexdigest()


def newest_redarm(name, logs, box):
    """(the newest `redarm` record naming this scope on this box - any task's - , its entry), or (None, None)."""
    last = None
    with contextlib.suppress(OSError, UnicodeDecodeError):
        for ln in (Path(logs) / "loop_times.jsonl").read_text(encoding="utf-8").splitlines():
            with contextlib.suppress(ValueError, AttributeError, TypeError):
                r = json.loads(ln)
                if r.get("mode") == "redarm" and r.get("box") == box and name in (r.get("scopes") or {}):
                    last = r
    sc = (last or {}).get("scopes")
    sc = sc.get(name) if isinstance(sc, dict) else None
    return (last, sc) if isinstance(sc, dict) else (None, None)


def redarm_proof(man, name, logs, box, fp):
    """The newest red-arm record of this scope on this box - any task's - when it still vouches for the scope:
    GO, every plant RED, the plant ids the manifest's, its fingerprint `fp` (today's). -> (task, ts) or None."""
    if fp is None:
        return None
    ids = {a.get("id") for a in man["scopes"][name].get("plants") or [] if isinstance(a, dict)}
    last, sc = newest_redarm(name, logs, box)
    if sc is None or sc.get("verdict") != "GO" or sc.get("fingerprint") != fp:
        return None
    plants = sc.get("plants")
    if not isinstance(plants, dict) or not ids or set(plants) != ids or any(v != "RED" for v in plants.values()):
        return None
    return str(last.get("task", "?")), str(last.get("ts", "?"))


def clean_proof(name, logs, box, fp):
    """The clean arm already proven: the newest red-arm record of this scope on this box - any task's - holds a
    clean GO (or a clean it took as proven) on fingerprint `fp`, today's. -> (task, ts) or None. A record from
    before the `clean` field: its verdict GO says its clean went GO; a NO-GO one says nothing."""
    last, sc = newest_redarm(name, logs, box) if fp is not None else (None, None)
    if sc is None or sc.get("fingerprint") != fp:
        return None
    clean = sc.get("clean", "GO" if sc.get("verdict") == "GO" else None)
    return (str(last.get("task", "?")), str(last.get("ts", "?"))) if clean in ("GO", "PROVEN") else None


def arm_workers(s, jobs, n):
    """How many arms run at once: up to the run's jobs, one at a time for a scope that must run alone - `exclusive`,
    bound to one `box` (a GPU, a device), or holding `free_ports` (two arms would race for one port)."""
    if s.get("exclusive") or s.get("box") or s.get("free_ports"):
        return 1
    return max(1, min(int(jobs), n))


def plant_case(arm):
    """The one case a plant's arm runs - its `case` - or None: the whole scope."""
    return arm.get("case") if isinstance(arm, dict) else None


def case_listed(sr, case, copy, man, env, task, box):
    """None when `case` is one of the scope's cases - its `cases`, or what its `cases_cmd` lists in the planted
    copy - else why not: an arm on a case the scope does not list reads red on that reason alone, a red that
    proves nothing, so it is an error, never a red and never a skip."""
    cases = sr.spec.get("cases")
    if isinstance(cases, (list, dict)):
        found = [str(c) for c in cases]
    elif "cases_cmd" in sr.spec:
        found = [u[2] for u in units(sr, None, copy, man, env, task, box)]
        if sr.reasons or sr.skip:
            return f"cannot be checked: the scope's `cases_cmd` lists none ({'; '.join(sr.reasons) or sr.skip})"
    else:
        return "names a case of a scope that has none (no `cases`, no `cases_cmd`)"
    return None if case in found else f"`{case}` is not one of the scope's cases ({', '.join(found[:8]) or 'none'})"


def redarm(man, name, root, task, jobs, box=None, logs=None, record=True, tracked_fn=None, tool=None, prefix=""):
    """The red-arm campaign of one scope: a fresh scratch copy per arm - the clean copy must go GO (a red
    with no plant proves nothing), each declared plant, planted in its own copy, must go NO-GO; a timeout
    or a command that did not start is an error, never a red. The arms run side by side, up to `jobs`
    (`arm_workers`: a scope that must run alone, one at a time), the jobs shared out between them; their
    lines, logs and record in declared order. A plant's `case` runs that case alone. The clean arm is not
    re-run when the newest red-arm record on this box holds a clean GO on today's fingerprint (`clean_proof`).
    Arms that run, when there are several, are numbered (M0-D27): named live as each starts - after `prefix`,
    a sweep's campaign - and on its line. The tree itself is never written but for the campaign's log."""
    box, root = box or this_box(), Path(root)
    logs = Path(logs) if logs else logs_of(man, root)
    if name not in man["scopes"]:
        raise Refused(f"no scope {name!r} - `list` shows them")
    bl = blocked(man)
    if name in bl:
        raise Refused(f"`{name}` is NOT RUN ({bl[name]}): nothing to red-arm - map the setting first")
    s = man["scopes"][name]
    if "placeholder" in s:
        raise Refused(f"`{name}` is a placeholder until {s['placeholder']}: nothing to red-arm")
    if owed_on(s, box):
        raise Refused(f"`{name}` runs on {owed_label(s)} only (this box: {box}) - red-arm it there; owed on {owed_label(s)}")
    plants = s.get("plants") or []
    if not plants:
        return result(f"redarm: 0 plants · scope {name} · box {box}",
                      nogo=f"`{name}` declares no plant - an unarmed scope (§8: a check ships red-armed)")
    logs.mkdir(parents=True, exist_ok=True)
    log = logs / f"{task}.{name}.redarm.log"
    refused = refuse_log(root, [log], tracked_fn)
    if refused:
        raise Refused(refused)
    one = dict(man, scopes={name: s})
    env = case_env(task)
    t0 = time.monotonic()
    fp = fingerprint(man, name, root, visible_files(root, man, logs), tool)       # the bytes, before
    proof = clean_proof(name, logs, box, fp)
    todo = [(i, arm) for i, arm in enumerate([None] + plants) if arm is not None or not proof]
    workers = arm_workers(s, jobs, len(todo))
    num = {i: k for k, (i, _arm) in enumerate(todo, 1)} if len(todo) > 1 else {}
    at = lambda i: f" · {step_tag('arm', num[i], len(todo))}" if i in num else ""   # noqa: E731
    inner = max(1, int(jobs) // workers)                     # the run's jobs shared out between the arms
    part = lambda i: Path(f"{log}.arm{i}.part")              # noqa: E731 - each arm's own log, joined in order
    tmp = Path(tempfile.mkdtemp(prefix="pb-redarm-"))

    def arm_run(i, arm):
        """One arm in its own copy -> (why it is an error, or None; its scope's row, or None)."""
        label = "clean" if arm is None else f"plant {arm['id']}"
        if i in num:
            say(f">> {prefix}{name} · {label}{at(i)} · started")
        with LOCK, open(part(i), "a", encoding="utf-8") as fh:
            fh.write(f"==== {stamp()} {name} · {label} · a fresh scratch copy of the tree\n")
        copy = tmp / f"arm{i}"
        scratch_copy(root, copy)
        case = plant_case(arm)
        if arm is not None:
            why = plant(arm, root, copy, one, env, part(i))
            if why:
                return why, None
            if case is not None:
                why = case_listed(ScopeRun(name, s, 0, part(i)), case, copy, one, env, task, box)
                if why:
                    return f"its `case` {why}", None
        r = run_scopes(one, [name], copy, task, inner, case=case, mode="redarm", box=box, logs=logs,
                       log_for=lambda n: part(i), anchor=False, record=False, tracked_fn=lambda _r, _p: [])
        return None, r["data"]["rows"][0]

    done = {}
    try:
        with ThreadPoolExecutor(max_workers=workers) as pool:
            futs = {i: pool.submit(arm_run, i, arm) for i, arm in todo}
            if 0 in futs and futs[0].result()[1]["verdict"] != "GO":
                for f in futs.values():                       # a red clean: the arms not started never start
                    f.cancel()
            done = {i: f.result() for i, f in futs.items() if not f.cancelled()}
    finally:
        with LOCK, open(log, "a", encoding="utf-8") as fh:
            if proof:
                fh.write(f"==== {stamp()} {name} · clean: proven by {proof[0]} {proof[1]} (fingerprint {fp[:12]})\n")
            for i, _arm in todo:                              # declared order, whatever order they ended in
                with contextlib.suppress(OSError):
                    fh.write(part(i).read_text(encoding="utf-8", errors="replace"))
                    part(i).unlink()
        shutil.rmtree(str(tmp), ignore_errors=True)               # this campaign's own copies
    lines, fails, red, green, errors, arms = [], [], 0, 0, 0, {}
    if proof:
        clean = "PROVEN"
        lines.append(f"{name} · clean: proven by {proof[0]} {proof[1]} (fingerprint {fp[:12]})")
    else:
        row = done[0][1]
        clean = row["verdict"]
        lines.append(f"{name} · clean · {clean}{at(0)}")
        if clean != "GO":
            fails.append(f"clean: {'; '.join(row['reasons'])} - red with no plant, so a red plant would prove "
                         f"nothing; the plants are not counted")
    for i, arm in enumerate(plants, 1):
        if clean not in ("GO", "PROVEN"):
            break
        label = f"plant {arm['id']}" + (f" (case {plant_case(arm)})" if plant_case(arm) is not None else "")
        why, row = done[i]
        if why:
            errors += 1
            arms[arm["id"]] = "ERROR"
            lines.append(f"{name} · {label} · ERROR ({why}){at(i)}")
            fails.append(f"{label}: {why} - an error, not a red")
            continue
        honest = row["verdict"] == "NO-GO" and not any(
            k in x for x in row["reasons"] for k in ("timed out", "not started", "NOT RUN (", "no case `"))
        arms[arm["id"]] = "RED" if honest else "GREEN" if row["verdict"] == "GO" else "ERROR"
        if honest:
            red += 1
            lines.append(f"{name} · {label} · RED (proven: {'; '.join(row['reasons'])[:120]}){at(i)}")
        elif row["verdict"] == "GO":
            green += 1
            lines.append(f"{name} · {label} · GREEN (unfailable){at(i)}")
            fails.append(f"{label}: stayed GO - the scope cannot see this bug")
        else:
            errors += 1
            lines.append(f"{name} · {label} · ERROR ({'; '.join(row['reasons'])}){at(i)}")
            fails.append(f"{label}: {'; '.join(row['reasons'])} - an error, not a red")
    secs = round(time.monotonic() - t0, 2)
    if record:
        if fp is not None and fingerprint(man, name, root, visible_files(root, man, logs), tool) != fp:
            fp = None                          # the bytes moved under the campaign: its arms saw two trees, no proof
        rec = {"ts": stamp(), "task": task, "mode": "redarm", "seconds": secs, "tree": tree_id(root, man, logs),
               "verdict": "NO-GO" if fails or not red else "GO", "box": box, "python": PYTHON, "arms_at_once": workers,
               "scopes": {name: {"plants": arms, "verdict": "NO-GO" if fails or not red else "GO",
                                 "clean": clean, "fingerprint": fp}}}
        load, load_source = sample_load()                  # after `secs`: sampling never bills the campaign
        rec.update(load=load, load_source=load_source, cpus=os.cpu_count())
        normalise(rec, man)
        with LOCK, open(logs / "loop_times.jsonl", "a", encoding="utf-8") as fh:
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    counts = (f"redarm: {len(plants)} plants · {red} red · {green} green · {errors} errors · scope {name} · {secs:.2f} s · "
              f"{workers} arm(s) at a time · "
              f"box {box} · log {rel(log, root)}")
    return result(counts, failures=fails, lines=lines,
                  nogo=None if not fails and red else ("zero plants red" if not fails else None),
                  red=red, green=green, errors=errors, clean=clean, workers=workers)


def moved_sweep(man, root, task, jobs, box=None, logs=None, tool=None):
    """--redarm --moved, the heavy tier's re-proof sweep (PLAYBOOK §A.2): every armed scope the red-arm proof no
    longer vouches for on this box - no record, another fingerprint, a NO-GO, a plant not RED, the ids moved - is
    red-armed, one campaign each, each writing its own record; the rest are named with the proof they stand on.
    The campaigns are counted first, so each is numbered among them, its arms within it (M0-D27).
    GO only when every campaign is GO; nothing moved is GO, named; no armed scope at all is NO-GO."""
    box, root = box or this_box(), Path(root)
    logs = Path(logs) if logs else logs_of(man, root)
    bl = blocked(man)
    armed = [n for n, s in man["scopes"].items() if isinstance(s, dict) and s.get("plants")]
    if not armed:
        return result(f"moved: 0 of 0 armed scopes, plants RED 0, GREEN 0, ERROR 0 · box {box}",
                      nogo="no scope declares plants - nothing to sweep (§8: a check ships red-armed)")
    visible = visible_files(root, man, logs)
    if visible is None:
        raise Refused("the tree's files cannot be listed (a folder or .gitignore is unreadable) - no fingerprint, "
                      "so the sweep cannot tell a moved scope: nothing ran")
    t0, lines, fails, n = time.monotonic(), [], [], {"moved": 0, "here": 0, "red": 0, "green": 0, "errors": 0}
    steps = []                      # declared order: (name, its line, its failure or None) - a campaign's line is None
    for name in armed:
        s = man["scopes"][name]
        if name in bl:
            steps.append((name, f"{name} · NOT RUN ({bl[name]})",
                          f"{name}: NOT RUN ({bl[name]}) - the sweep cannot vouch for a scope it cannot run"))
            continue
        if "placeholder" in s:
            steps.append((name, f"{name} · SKIP (placeholder until {s['placeholder']})", None))
            continue
        if owed_on(s, box):
            steps.append((name, f"{name} · owed on {owed_label(s)} (this box: {box}) - swept there, never NO-GO here", None))
            continue
        n["here"] += 1
        fp = fingerprint(man, name, root, visible, tool)
        proof = redarm_proof(man, name, logs, box, fp)
        steps.append((name, f"{name} · proven by {proof[0]} {proof[1]} (fingerprint {fp[:12]}) - not moved"
                      if proof else None, None))
    total = sum(1 for _n, line, _f in steps if line is None)    # every campaign counted before one starts (M0-D27)
    todo = [name for name, line, _f in steps if line is None]
    exp, unknown = guesses(todo, past_seconds(logs, box, redarm=True)) if live() and total > 1 else ({}, 0)
    if exp:
        say(f".. sweep: {total} campaigns, one at a time · ≈ {dur(sum(exp.values()))} by this box's past red-arms"
            + (f" ({unknown} without one: the median)" if unknown else ""))
    for name, line, fail in steps:
        if line is not None:
            lines.append(line)
            fails += [fail] if fail else []
            continue
        if exp and n["moved"]:                                   # the campaigns before this one are done
            ran = todo[:n["moved"]]
            say(eta_line("sweep", n["moved"], 0, total - n["moved"], total, time.monotonic() - t0,
                         sum(exp[x] for x in ran), sum(exp[x] for x in todo[n["moved"]:])))
        n["moved"] += 1
        at = step_tag("campaign", n["moved"], total) if total > 1 else ""
        if at:
            say(f">> {name} · {at} · started")
        try:
            r = redarm(man, name, root, task, jobs, box=box, logs=logs, tool=tool, prefix=f"{at} · " if at else "")
        except Refused as exc:
            lines.append(f"{name} · refused ({exc})")
            fails.append(f"redarm {name}: {exc}")
            continue
        lines += r["lines"] + [r["counts"] + (f" · {at}" if at else "")]
        for k in ("red", "green", "errors"):
            n[k] += r["data"].get(k, 0)
        fails += [f"redarm {name}: {x}" for x in (r["failures"] or ([r["nogo"]] if r["nogo"] else []))]
    if not n["moved"] and not fails:
        lines.append(f"nothing moved: every armed scope on this box stands on its proof ({n['here']})")
    counts = (f"moved: {n['moved']} of {n['here']} armed scopes, plants RED {n['red']}, GREEN {n['green']}, "
              f"ERROR {n['errors']} · {time.monotonic() - t0:.2f} s · box {box}")
    return result(counts, failures=fails, lines=lines, **n)


# --------------------------------------------------- the loop's flags: an oracle edit, a write off its type
def find_block(root, task):
    """{type, deliver, plan} of the task's block in a plan under milestones/, or None. The type is the new
    word in the heading's bold tag, else the one its id carries (PLAYBOOK §0) - plan.py's reading."""
    try:
        if str(TOOL_DIR) not in sys.path:
            sys.path.insert(0, str(TOOL_DIR))
        import plan as pbplan                                    # the sibling annex module
    except Exception:                                             # pragma: no cover - plan.py always ships beside
        return None
    for path in sorted(Path(root).glob("milestones/*/*implementation_plan*.md")):
        with contextlib.suppress(OSError, UnicodeDecodeError):
            if f"## {task} " not in path.read_text(encoding="utf-8"):
                continue
            p = pbplan.Plan(str(path))
            b = p.block(task)
            if b is None:
                continue
            tag = pbplan.TAG_RE.search(b.heading)
            words = [w for w in re.findall(r"[A-Z]+", tag.group(1)) if w in pbplan.TYPES] if tag else []
            typed = pbplan.id_type(b.id)
            return {"type": words[0] if words else (typed[0][0] if typed and typed[0] else None),
                    "deliver": b.field_text("Deliver") or "", "plan": rel(path, root)}
    return None


def changed_paths(root, rev, git=git_out):
    d, u = git(root, ["diff", "--name-only", "-z", rev, "--"]), git(root, ["ls-files", "-o", "--exclude-standard", "-z"])
    return None if d is None or u is None else sorted({p for p in (d + "\0" + u).split("\0") if p})


def manifest_changes(man, root, manifest, rev, git=git_out):
    """{scope: 'added' | 'changed'} against the manifest at `rev`, a note's wording aside; None when git
    cannot show that manifest."""
    old = git(root, ["show", f"{rev}:{rel(manifest, root)}"], 30)
    try:
        olds = (json.loads(old).get("scopes") or {}) if old is not None else None
    except (ValueError, AttributeError):
        olds = None
    return scope_changes(man, olds)


def scope_changes(man, olds):
    """{scope: 'added' | 'changed'} of the manifest in `man` against the scopes mapping `olds`, a note's
    wording aside; None when `olds` is no mapping."""
    if not isinstance(olds, dict):
        return None
    canon = lambda s: json.dumps({k: v for k, v in s.items() if not is_note(k)} if isinstance(s, dict) else s,  # noqa: E731
                                 sort_keys=True)
    return {n: ("added" if n not in olds else "changed") for n, s in man["scopes"].items()
            if n not in olds or not isinstance(olds[n], dict) or canon(s) != canon(olds[n])}


# ----------------------------------------- the task's own start: the tree as the task found it (PLAYBOOK §A.2)
# `start` records, once per task, every file the walk lists (size, mtime, content hash) and the manifest's text,
# under <logs_dir>/.baseline/ - per box, never committed (a `.gitignore` of `*` inside), held out of the
# anchor's subject as the logs folder is. The loop reads what THIS task changed against it, never HEAD: a
# commit between tasks - or none - no longer decides how much a check runs, and no repository is needed.
def baseline_dir(man, root):
    return logs_of(man, root) / ".baseline"


def read_baseline(path):
    """The baseline record at `path`, or None when it is absent or no record of ours."""
    try:
        rec = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None
    return rec if isinstance(rec, dict) and isinstance(rec.get("files"), dict) and rec.get("task") else None


def baselines(man, root):
    """Every baseline on this box, newest first: [(epoch, record)]."""
    bdir = baseline_dir(man, root)
    got = [(rec.get("epoch") or 0, rec) for rec in (read_baseline(p) for p in sorted(bdir.glob("*.json")))
           if rec is not None] if bdir.is_dir() else []
    return sorted(got, key=lambda t: -t[0])


def snapshot(root, man, logs, prev=None):
    """{path: [size, mtime_ns, content hash]} of every file `walk_tree` lists, the logs folder and the anchor
    held out (every run rewrites them); `tree_extra` is the anchor's, not the task's. A file whose size and
    mtime match `prev`'s entry (the newest baseline's) keeps that hash - no re-read. None when the tree cannot
    be listed or a listed file cannot be read: a start that cannot name the tree certifies nothing."""
    try:
        held = held_out(root, man, logs)
        paths = [p for p in walk_tree(root) if not is_held(p, held)]
    except OSError:
        return None
    out = {}
    for p in paths:
        try:
            st = os.stat(Path(root) / p)
            old = (prev or {}).get(p)
            if old and old[0] == st.st_size and old[1] == st.st_mtime_ns:
                out[p] = list(old)
            else:
                out[p] = [st.st_size, st.st_mtime_ns, content_sha1((Path(root) / p).read_bytes())]
        except OSError:
            return None
    return out


def cmd_start(man, a, root, manifest):
    """`start --task <ID>`: write the task's baseline once; a second call for the same id keeps the first (an
    escalated or resumed task keeps its start) unless --fresh. One line: `baseline: <ID>, <n> files, <ts>`."""
    task = task_of(a)
    bdir = baseline_dir(man, root)
    target, kept = bdir / f"{task}.json", None
    first = read_baseline(target)
    if first is not None and not a.fresh:
        kept = first
    else:
        logs = logs_of(man, root)
        known = baselines(man, root)
        files = snapshot(root, man, logs, known[0][1]["files"] if known else None)
        if files is None:
            raise Refused("the tree's files cannot be listed or read (a folder, a .gitignore or a file is unreadable) "
                          "- a start that cannot name the tree certifies nothing")
        try:
            text = Path(manifest).read_text(encoding="utf-8")
        except OSError:
            text = ""
        kept = {"version": 1, "task": task, "ts": stamp(), "epoch": time.time(), "manifest": text, "files": files}
        bdir.mkdir(parents=True, exist_ok=True)
        (bdir / ".gitignore").write_text("*\n", encoding="utf-8")
        tmp = bdir / f"{task}.json.tmp"
        tmp.write_text(json.dumps(kept, ensure_ascii=False), encoding="utf-8")
        os.replace(tmp, target)
        for _epoch, old in baselines(man, root)[BASELINES_KEPT:]:
            with contextlib.suppress(OSError):
                (bdir / f"{old['task']}.json").unlink()
    line = f"baseline: {task}, {len(kept['files'])} files, {kept['ts']}"
    return result(f"start: {task} · {len(kept['files'])} files · {'kept' if kept is first and not a.fresh else 'written'}"
                  f" · box {this_box()}", lines=[line], baseline=line)


def touched_of(now, old):
    """{added, modified, deleted}: the paths `now` adds to, changes in and drops from `old` (a start's files),
    by content hash - the task's own changes, as its claim record names them."""
    return {"added": sorted(p for p in now if p not in old),
            "modified": sorted(p for p in now if p in old and now[p][2] != old[p][2]),
            "deleted": sorted(p for p in old if p not in now)}


def touched_git(root, rev, git=git_out):
    """{added, modified, deleted} against the named commit (--base), untracked files as added; None when git
    cannot list them."""
    d, u = git(root, ["diff", "--name-status", "--no-renames", "-z", rev, "--"]), git(root, ["ls-files", "-o", "--exclude-standard", "-z"])
    if d is None or u is None:
        return None
    out, parts = {k: set() for k in TOUCH_KINDS}, d.split("\0")
    for st, p in zip(parts[0::2], parts[1::2]):
        if p:
            out[{"A": "added", "D": "deleted"}.get(st[:1], "modified")].add(p)
    out["added"] |= {p for p in u.split("\0") if p}
    return {k: sorted(v) for k, v in out.items()}


def loop_base(man, root, task, manifest, git=git_out, now=None):
    """What the loop reads this task's changes against: -> {changed, touched, mchanges, label, line}. The task's
    own start; else the newest start on this box, named; else none - every scope with plants re-armed once and no
    path flagged. `touched` is {added, modified, deleted} or None, `changed` their union, `mchanges` {scope:
    added|changed} or None, all against the baseline - never HEAD. `line` is the `base:` line for the counts'
    tail; the claim record keeps it, so a list read against another task's start says whose."""
    known = baselines(man, root)
    own = next((r for _e, r in known if r["task"] == task), None)
    base = own or (known[0][1] if known else None)
    if base is None:
        armed = {n: "added" for n, s in man["scopes"].items() if isinstance(s, dict) and s.get("plants")}
        return {"changed": None, "touched": None, "mchanges": armed, "label": "no start",
                "line": "base: none \u2014 every plant"}
    now = snapshot(root, man, logs_of(man, root)) if now is None else now
    touched = None if now is None else touched_of(now, base["files"])
    changed = None if touched is None else sorted(p for k in TOUCH_KINDS for p in touched[k])
    try:
        olds = json.loads(base.get("manifest") or "").get("scopes")
    except (ValueError, AttributeError):
        olds = None
    label = f"{base['task']}'s start"
    line = f"base: {label} {base['ts']}" + ("" if own else f" \u2014 no start for {task}")
    return {"changed": changed, "touched": touched, "mchanges": scope_changes(man, olds), "label": label, "line": line}


def command_paths(man):
    """The scripts the scopes' own commands run - the checks themselves; a file a command only reads (the
    plan a lint grades, the documents a gate scans) is its subject, not its check."""
    out = set()
    for s in man["scopes"].values():
        for cmd in commands(s):
            with contextlib.suppress(ValueError):
                out |= {t for t in shlex.split(cmd) if t.endswith(CODE_SUFFIXES) and not t.startswith("-")}
    return out


def names(text, token):
    """True when `text` names `token` whole - a word or a path, never a piece of a longer one."""
    return bool(token) and re.search(r"(?<![\w./-])" + re.escape(token) + r"(?![\w/-])", text) is not None


def outside_type(typ, p):
    code = p.endswith(CODE_SUFFIXES) or p.startswith("tools/")
    if typ in ("CHECK", "PLAN") and code:
        return f"a {typ} block writes no product code and no tool (PLAYBOOK §0)"
    if typ == "MOVE" and not (PLAN_FILE_RE.search(p) or RULES_FILE_RE.search(p) or "/logs/" in "/" + p
                              or "/tasks/" in "/" + p):
        return "a MOVE block writes the plan, its archive and m<N>_rules.md only, byte for byte (PLAYBOOK §0)"
    return None


def loop_flags(block, changed, mchanges, man, manifest_rel, task, rev):
    """The loop's flags - lines, never a verdict (PLAYBOOK §0, §A.2): a changed test, case or manifest entry
    the block's `Deliver:` does not name (a block never edits its own check unless Deliver names it), and a
    changed path outside the block's type."""
    if block is None:
        return [f"flags: {task} is no block of a plan under milestones/ - no Deliver and no type to read them against"]
    if changed is None:
        return ["flags: no base to list the changed paths against (no start, or git could not list them) - no flag read"]
    deliver, lines = block["deliver"], []
    named = lambda p: names(deliver, p) or names(deliver, os.path.basename(p))  # noqa: E731
    checks = command_paths(man)
    for p in changed:
        oracle = p == manifest_rel or TEST_RE.search(p) or any(p == c or p.startswith(c + "/") for c in checks)
        if oracle and not named(p):
            lines.append(f"FLAG oracle: {p} changed since {rev} - a test, case or check {task}'s Deliver: does not name")
        why = outside_type(block["type"], p)
        if why:
            lines.append(f"FLAG type: {p} changed since {rev} - {why}")
    for n, how in sorted((mchanges or {}).items()):
        if not names(deliver, n) and not named(manifest_rel):
            lines.append(f"FLAG oracle: scope `{n}` {how} in {manifest_rel} - {task}'s Deliver: names neither it nor the manifest")
    return lines


def loop_checks(man, root, task, jobs, box, manifest, base_sha=None, git=git_out, block=False, changed=False,
                mchanges=False, label=None, tool=None, proven=None):
    """What the loop adds to its scopes (PLAYBOOK §8, §A.2): the plants of every scope added or changed
    since the base run, and must each go NO-GO - unless a red-arm record on this box, any task's, already
    proves them on the same bytes (`redarm_proof`; `proven` collects {scope: "<task> <ts> <fp 12>"} for the
    claim's record); the flags. The base is the task's own start (`loop_base`, which hands `changed`,
    `mchanges` and `label` in); a `--base` commit reads git; passing neither reads HEAD. -> (lines, failures)."""
    rev, lines, fails = base_sha or "HEAD", [], []
    label = label or rev[:12]
    manifest_rel = rel(manifest, root)
    block = find_block(root, task) if block is False else block
    changed = changed_paths(root, rev, git) if changed is False else changed
    mchanges = manifest_changes(man, root, manifest, rev, git) if mchanges is False else mchanges
    if mchanges is None:
        lines.append(f"plants: {manifest_rel} as of {label} cannot be read - the scopes added or changed are unknown")
    bl, logs, visible = blocked(man), logs_of(man, root), False
    for n, how in sorted((mchanges or {}).items()):
        s = man["scopes"][n]
        if n in bl or "placeholder" in s or owed_on(s, box):          # a NOT RUN scope is named on its own line
            continue
        if not s.get("plants"):
            lines.append(f"FLAG plants: scope `{n}` {how} since {label} and declares no plant - a check it "
                         f"cannot fail is not a check (§8)")
            continue
        visible = visible_files(root, man, logs) if visible is False else visible
        fp = fingerprint(man, n, root, visible, tool)
        proof = redarm_proof(man, n, logs, box, fp)
        if proof:
            lines.append(f"{n} · red-arm proven by {proof[0]} {proof[1]} (fingerprint {fp[:12]}) - not re-run")
            if proven is not None:
                proven[n] = f"{proof[0]} {proof[1]} {fp[:12]}"
            continue
        r = redarm(man, n, root, task, jobs, box=box, record=False)
        lines += r["lines"] + [r["counts"]]
        fails += [f"redarm {n}: {x}" for x in (r["failures"] or ([r["nogo"]] if r["nogo"] else []))]
    return lines + loop_flags(block, changed, mchanges, man, manifest_rel, task, label), fails


def loop_bill(elapsed, before_s, hook_s):
    """The claim's two clocks, each second once: `seconds` the scopes alone, as every record has it;
    `loop_seconds` what the loop adds - its base read before the scopes, its plants and flags after them."""
    return {"seconds": elapsed, "loop_seconds": round(before_s + hook_s, 3)}


# ----------------------------------------------------------------------------------------------------- a run
def run_scopes(man, names, root, task, jobs, case=None, mode="scope", box=None, logs=None, log_for=None,
               anchor=True, gate=False, visible=None, not_run=(), why=None, hook=None, record=True,
               tracked_fn=None, extra=None, pick=None, reuse=False):
    """Run these scopes: the plain ones in parallel (jobs), the exclusive ones alone after them, the smoke last
    on --all; one record appended; the anchor advanced by the green scopes; with `gate`, the anchor graded.
    `pick` ({"cases", "by_scope"}, --changed's): a scope whose cases are files runs the cases that moved.
    `reuse` (a scoped or `--case` run's, never the loop's): a scope the task's newest record on this box has GO on
    today's tree id is not re-run, unless it declares `rerun` (M0-T64c). Several scopes that run: each a step,
    numbered in the order it starts, named live as it starts and on its line (M0-D27)."""
    box, root = box or this_box(), Path(root)
    logs = Path(logs) if logs else logs_of(man, root)
    log_for = log_for or (lambda n: logs / f"{task}.{n}.log")
    logs.mkdir(parents=True, exist_ok=True)
    bl, parts = blocked(man), top_errors(man)
    specs = [(n, man["scopes"][n], bl.get(n)) for n in names]         # (name, spec, why it is NOT RUN or None)
    if mode == "all" and (man.get("smoke") or "smoke" in parts):
        specs.append(("smoke", {"cmd": man["smoke"], "exclusive": True}, parts.get("smoke")))
    tree = tree_id(root, man, logs) if record else None
    green = {}
    if reuse and tree:
        recs = records_in(logs / "loop_times.jsonl")[1]
        green = {n: g for n, s, no in specs if not no and "placeholder" not in s and not owed_on(s, box)
                 and not s.get("rerun") and (g := green_cover(recs, task, box, tree, n, case))}
    refused = refuse_log(root, [log_for(n) for n, s, no in specs if not no and "placeholder" not in s
                                and not owed_on(s, box) and n not in green], tracked_fn)
    if refused:                                  # nothing opened, nothing spawned, no record: the run did not happen
        raise Refused(refused)
    load_start, _src = sample_load()
    apath = anchor_path(man, root)
    use_anchor = (anchor and case is None and apath is not None and not anchor_off(man)
                  and any(s.get("paths") for n, s, no in specs if not no and n not in green))
    if (use_anchor or gate) and visible is None:
        visible = visible_files(root, man, logs)
        if visible is None and gate:
            raise Refused("the tree's files cannot be listed (a folder or .gitignore is unreadable) - the anchor cannot name its subject")
    snap = (anchor_snapshot(man, root, [n for n, _s, no in specs if not no and n not in green], visible)
            if use_anchor and visible is not None else {})
    env = case_env(task)
    t0, runs, todo = time.monotonic(), [], []
    for name, s, no in specs:
        sr = ScopeRun(name, s, 0 if no else 1 if case is not None or name == "smoke" else int(s.get("expected", 0)),
                      log_for(name))
        runs.append(sr)
        if no:
            sr.notrun = no                                            # never run without the setting it holds
        elif name in green:                                           # already GO on this tree: nothing runs
            sr.green = green[name]
            sr.passed, sr.failed, sr.skipped = (int(green[name][1].get(k) or 0) for k in ("passed", "failed", "skipped"))
        elif "placeholder" in s:
            sr.skip = f"placeholder until {s['placeholder']}"
        elif owed_on(s, box):
            sr.owed = owed_label(s)
        else:
            us = units(sr, case, root, man, env, task, box)
            if use_anchor and visible is not None and s.get("paths") and sr.listed and us:
                sr.booked = True                                               # hashed BEFORE any case runs
                sr.case_paths = case_paths(s, sr.listed, visible, held_out(root, man, logs), root)
                sr.suite = suite_hash(man, name, root, logs)
                sr.case_now = {c: sha1_of(root, f) for c, f in (sr.case_paths or {}).items()}
                if pick is not None:
                    bs = (pick.get("by_scope") or {}).get(name) or {}
                    us, sr.pick, sr.vouched_n = case_pick(us, sr.case_paths, sr.case_now, sr.suite,
                                                          (pick.get("cases") or {}).get(name) or {},
                                                          list(bs.get("never", [])) + list(bs.get("stale", []))
                                                          + list(bs.get("gone", [])))
                    sr.vouched = len(sr.listed) - len(us)
            todo += [(sr,) + u for u in us]
    steps = list(dict.fromkeys(u[0] for u in todo if not u[0].spec.get("exclusive")))   # a step: a scope that runs,
    steps += list(dict.fromkeys(u[0] for u in todo if u[0].spec.get("exclusive")))      # in the order it starts (M0-D27)
    for k, sr in enumerate(steps if len(steps) > 1 else [], 1):
        sr.step = (k, len(steps))
    units_left = {}                                    # each step's units not ended yet: 0 is a step done
    for u in todo:
        units_left[u[0].name] = units_left.get(u[0].name, 0) + 1
    exp, unknown = guesses([sr.name for sr in steps], past_seconds(logs, box)) if live() and len(steps) > 1 else ({}, 0)
    if exp:                                            # the plain ones share the jobs, the exclusive ones run alone
        first_guess = (sum(exp[sr.name] for sr in steps if not sr.spec.get("exclusive")) / max(1, jobs)
                       + sum(exp[sr.name] for sr in steps if sr.spec.get("exclusive")))
        say(f".. {mode}: {len(steps)} scopes to run, {max(1, jobs)} at a time · ≈ {dur(first_guess)} by this box's "
            f"past runs" + (f" ({unknown} without one: the median)" if unknown else ""))

    def progress():
        """The live ETA line - under LOCK, so the counts are one moment's."""
        now = time.time()
        done = [sr for sr in steps if not units_left[sr.name]]
        running = [sr for sr in steps if units_left[sr.name] and sr.started is not None]
        got = {sr.name: min(exp[sr.name], now - sr.started) for sr in running}
        work_done = sum(exp[sr.name] for sr in done) + sum(got.values())
        work_left = sum(exp[sr.name] for sr in steps if sr.started is None) + sum(exp[n] - g for n, g in got.items())
        return eta_line(mode, len(done), len(running), len(steps) - len(done) - len(running), len(steps),
                        time.monotonic() - t0, work_done, work_left, jobs)

    def one(u):
        sr, label, cmd, value = u
        with LOCK:
            first, sr.started = sr.started is None, sr.started or time.time()
        if first and sr.step:
            say(f">> {sr.name} · {step_tag('step', *sr.step)} · started")
        run_unit(sr, label, cmd, value, root, man, env, task, box)
        with LOCK:
            sr.ended = time.time()
            units_left[sr.name] -= 1
            line = progress() if exp and not units_left[sr.name] else None
        if line:
            say(line)

    with ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
        list(pool.map(one, [u for u in todo if not u[0].spec.get("exclusive")]))
    for u in [u for u in todo if u[0].spec.get("exclusive")]:        # alone, after the rest, in declared order
        one(u)
    elapsed = round(time.monotonic() - t0, 3)          # taken before the end sample: sampling never bills the run
    load, load_source = sample_load()
    ran = [sr for sr in runs if sr.ran()]
    kept = [sr for sr in runs if sr.green]                 # not re-run: counted as the run they were, named in the record
    reds = [sr for sr in ran if sr.red()]
    rec = {"ts": stamp(), "task": task, "mode": mode, "seconds": elapsed, "verdict": None, "tree": tree, "box": box,
           "python": PYTHON, "os": platform.system(), "load_start": load_start, "load": load,
           "load_source": load_source, "cpus": os.cpu_count(), "cases": sum(sr.passed + sr.failed for sr in ran)}
    rec.update({k: v for k, v in (extra or {}).items() if v is not None})
    if case is not None:
        rec["case"] = case
    if kept:
        rec["not_rerun"] = {sr.name: sr.green[0] for sr in kept}
    moved = anchor_advance(man, root, {sr.name for sr in ran if not sr.red() and sr.name != "smoke"},
                           {sr.name for sr in reds if sr.name != "smoke"}, snap, rec, apath, visible) if snap else 0
    booked = [sr for sr in ran if sr.booked]
    moved += cases_advance(apath, booked, rec) if booked else 0
    unrun = [sr for sr in runs if sr.notrun]
    lines = manifest_lines(man) + [f"why {n}: {w}" for n, w in (why or {}).items()]
    lines += [f"{sr.name} {sr.pick}" for sr in runs if sr.pick]
    w = max([len(sr.name) for sr in runs] + [len(n) for n, _g in not_run] + [8]) + 2
    lines += [sr.line(w, box) + (f" · {step_tag('step', *sr.step)}" if sr.step else "") for sr in runs]
    lines += [f"{n:<{w}}NOT RUN ({g})" for n, g in not_run]
    failures = [f"{sr.name}: {'; '.join(sr.red())} · log {rel(sr.log, root)}" for sr in reds]
    if mode in ("all", "changed"):              # a scoped run is judged on the scopes it ran; the loop is not complete
        failures += [f"{sr.name}: NOT RUN ({sr.notrun}) - " + ("the complete loop is not complete" if mode == "all"
                                                                else "the gate cannot vouch for a scope it cannot run")
                     for sr in unrun]
    if hook:
        th = time.monotonic()
        hook_lines, hook_fails = hook()
        lines += hook_lines
        failures += hook_fails
        rec.update(loop_bill(elapsed, rec.get("loop_seconds") or 0.0, time.monotonic() - th))
    tail = []
    if snap or gate:
        with contextlib.suppress(Refused, OSError):
            st = anchor_status(man, root, anchor_read(apath), visible or [], box)
            counts_a, alines = anchor_lines(st)
            tail.append(f"anchor: {moved} path×scope entries moved by this run" if snap else "anchor: this run moved nothing")
            tail.append(counts_a)
            if gate:
                tail += alines
                why_not = anchor_nogo(st)
                if why_not:
                    failures.append(f"anchor: {why_not}")
    passed, failed, skipped = (sum(getattr(sr, k) for sr in ran + kept) for k in ("passed", "failed", "skipped"))
    if passed == 0 and not failures:
        failures.append("zero passed in total" + (" - every scope named is NOT RUN, each named above"
                                                  if unrun and not ran else ""))
    rec["verdict"] = "NO-GO" if failures else "GO"
    rec["scopes"] = {sr.name: sr.record() for sr in runs if not sr.green}
    rec["scopes"].update({n: {"verdict": "NOT RUN", "reason": g} for n, g in not_run})
    normalise(rec, man)                                    # after the loop's bill: its seconds normalised too
    if record:
        with LOCK, open(logs / "loop_times.jsonl", "a", encoding="utf-8") as fh:   # append-only: one line
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    lines.append(f"logs: {rel(logs, root)}/{task}.<scope>.log · record: loop_times.jsonl · tree {tree}")
    lines += tail
    partial = f" · PARTIAL, not the loop (--case {case})" if case is not None else ""
    n_go = sum(1 for sr in ran if not sr.red()) + len(kept)
    counts = (f"verify: {passed} passed, {failed} failed, {skipped} skipped · {n_go}/{len(ran) + len(kept)} scopes GO · "
              f"{sum(1 for sr in runs if sr.skip)} skipped · {sum(1 for sr in runs if sr.owed)} owed · "
              f"{f'{len(kept)} not re-run · ' if kept else ''}{len(unrun)} NOT RUN · {len(not_run)} not impacted · {elapsed:.2f} s"
              + (f" + loop {rec['loop_seconds']:.2f} s" if is_num(rec.get("loop_seconds")) else "") + f" · mode {mode}{partial} · "
              f"jobs {jobs} · box {box} · "
              f"python {PYTHON} · load {load_start}->{load if load is not None else load_source}")
    rows = [dict(sr.record(), scope=sr.name, reasons=sr.red(), log=str(sr.log), started=sr.started, ended=sr.ended)
            for sr in runs]
    return result(counts, failures=failures, lines=lines, rows=rows, record=rec)


def task_of(a):
    task = a.task or os.environ.get("PB_TASK")
    if not task:
        raise Refused("no --task: every run names its task, so its logs are its own (`--task <ID>`, or $PB_TASK)")
    if not TASK_RE.match(task):
        raise Refused(f"--task `{task}` is not an id ([A-Za-z0-9_.-]) - it names a file")
    return task


def changed_run(man, root, task, jobs, base=None, box=None, git=git_out, visible=None, hook=None, extra=None):
    """--changed: every scope a changed path touches, chosen from the coverage anchor; then the gate - a path
    no scope has checked, or no scope covers, is NO-GO. `extra`: the loop's record fields (`loop_hook`)."""
    box, root, bl = box or this_box(), Path(root), blocked(man)
    off = anchor_off(man)
    if off:
        raise Refused(f"--changed: the coverage anchor is NOT RUN ({off}) - fix it, or run --all")
    if not any(s.get("paths") for n, s in man["scopes"].items() if n not in bl and "placeholder" not in s):
        raise Refused("--changed: no scope declares `paths`, so there is no coverage anchor to read - declare each "
                      "scope's paths and skip classes (docs/agent/testing.md), seed the anchor with one --all, "
                      "or run --all")
    logs = logs_of(man, root)
    visible = visible_files(root, man, logs) if visible is None else visible
    if visible is None:
        raise Refused("the tree's files cannot be listed (a folder or .gitignore is unreadable) - --changed cannot name its subject, so it certifies nothing")
    sha = resolve_base(root, base, git) if base is not None else None
    moved = moved_since(root, sha, git) if sha else set()
    anchor = anchor_read(anchor_path(man, root))
    st = anchor_status(man, root, anchor, visible, box)
    chosen, why, not_run = suite_chosen(man, root, logs, anchor.get("cases") or {}, *choose(man, st, moved, box, base))
    if not [n for n in chosen if n in bl or ("placeholder" not in man["scopes"][n] and not owed_on(man["scopes"][n], box))]:
        return result(f"verify: 0 passed, 0 failed, 0 skipped · 0 scopes chosen · {len(not_run)} not impacted · "
                      f"mode changed · box {box}",
                      lines=[f"{n}  NOT RUN ({g})" for n, g in not_run],
                      nogo="nothing changed since every scope's last GO: no scope to run, zero cases - the anchor "
                           "already vouches for this tree (a loop on it is waste)")
    return run_scopes(man, chosen, root, task, jobs, mode="changed", box=box, gate=True, visible=visible,
                      not_run=not_run, why=why, hook=hook, extra={"base": sha} if extra is None else extra,
                      pick={"cases": anchor.get("cases") or {}, "by_scope": st["by_scope"]})


def loop_hook(man, root, task, jobs, manifest, sha=None):
    """-> (hook, counts tail, record fields): the loop's hook for --changed and --all. The task's own changes are
    read against its start BEFORE the scopes run (a scope's own writes are not the task's); --base <commit> reads
    git. The fields: `base` (the line's text - whose start, or the commit), `touched` (the task's own files,
    added / modified / deleted), `loop_seconds` so far (this read; `run_scopes` adds the hook's) and `proven`
    (each scope the red-arm proof spared, filled by the hook before the record is written)."""
    t0, proven = time.monotonic(), {}
    if sha:
        hook = lambda: loop_checks(man, root, task, jobs, this_box(), manifest, sha, proven=proven)  # noqa: E731
        line, touched = f"base: commit {sha[:12]}", touched_git(root, sha)
    else:
        lb = loop_base(man, root, task, manifest)
        hook = lambda: loop_checks(man, root, task, jobs, this_box(), manifest, changed=lb["changed"],  # noqa: E731
                                   mchanges=lb["mchanges"], label=lb["label"], proven=proven)
        line, touched = lb["line"], lb["touched"]
    fields = {"base": line[len("base: "):], "touched": touched, "loop_seconds": round(time.monotonic() - t0, 3),
              "proven": proven}
    return hook, " \u00b7 " + line, fields


def cmd_run(man, a, root, manifest):
    task = task_of(a)
    cap = 1 if "jobs_cap" in top_errors(man) else int(man.get("jobs_cap", 8))    # a bad cap: one command at a time
    jobs = 1 if a.serial else (a.jobs or min(os.cpu_count() or 1, cap))
    if a.redarm:
        return moved_sweep(man, root, task, jobs) if a.moved else redarm(man, a.redarm, root, task, jobs)
    if a.changed:
        sha = resolve_base(root, a.base) if a.base else None
        hook, tail, fields = loop_hook(man, root, task, jobs, manifest, sha)
        res = changed_run(man, root, task, jobs, base=a.base, hook=hook, extra=fields)
        res["counts"] += tail
        return res
    names = list(man["scopes"])
    if a.all:
        hook, tail, fields = loop_hook(man, root, task, jobs, manifest)
        res = run_scopes(man, names, root, task, jobs, mode="all", hook=hook, extra=fields)
        res["counts"] += tail
        return res
    if a.case is not None:
        bl = blocked(man)
        with_case = [n for n in names if n not in bl and (man["scopes"][n].get("case_cmd")
                                                          or isinstance(man["scopes"][n].get("cases"), dict))]
        chosen = a.scopes or (with_case if len(with_case) == 1 else [])
        if len(chosen) != 1:
            raise CallError(f"--case takes exactly one scope (scopes with cases: {', '.join(with_case) or 'none'})")
        if chosen[0] not in man["scopes"]:
            raise Refused(f"no scope {chosen[0]!r} - `list` shows them")
        if not ID_RE.match(a.case):
            raise Refused(f"--case `{a.case}` has a space or a shell character")
        return run_scopes(man, chosen, root, task, jobs, case=a.case, mode="case", reuse=not a.force)
    unknown = [n for n in a.scopes if n not in man["scopes"]]
    if unknown:
        raise Refused(f"unknown scope(s): {', '.join(unknown)} - `list` shows them ({', '.join(names)})")
    return run_scopes(man, list(dict.fromkeys(a.scopes)), root, task, jobs, mode="scope", reuse=not a.force)


# --------------------------------------------------------------------------------------------- list · report
def cmd_list(man):
    bl, lines = blocked(man), manifest_lines(man)
    for name, s in man["scopes"].items():
        if name in bl:
            lines.append(f"{name:<18}NOT RUN ({bl[name]})")
            continue
        form = (f"placeholder until {s['placeholder']}" if "placeholder" in s else "cases from `cases_cmd`"
                if "cases_cmd" in s else f"{len(s['cases'])} case(s)" if "cases" in s else "1 command")
        cased = [f"{p['id']}: case {p['case']}" + ("" if not isinstance(s.get("cases"), (list, dict)) or p["case"]
                                                   in s["cases"] else " - not one of its cases: its arm is an ERROR")
                 for p in s.get("plants") or [] if "case" in p]
        more = [f for f, on in ((f"paths {len(s.get('paths', []))}", s.get("paths")),
                                (f"plants {len(s.get('plants', []))}" + (f" ({'; '.join(cased)})" if cased else ""),
                                 s.get("plants")),
                                (f"box {owed_label(s)}", s.get("box")), ("exclusive", s.get("exclusive")), ("rerun", s.get("rerun")),
                                (f"free_ports {s.get('free_ports')}", s.get("free_ports"))) if on]
        note = next((s[k] for k in ("note", "desc") if isinstance(s.get(k), str) and s[k]), None)
        lines.append(f"{name:<18}{s.get('parse', 'gonogo'):<8}{form:<32}expected {s.get('expected', 0)}"
                     + "".join(" · " + f for f in more) + (f"  - {note}" if note else ""))
    sk = man.get("skip_classes")
    lines.append(f"smoke: {man.get('smoke') or 'none'} · logs: {man.get('logs_dir', 'logs')} · jobs: min(CPUs, "
                 f"{man.get('jobs_cap', 8)}) · anchor: {man.get('anchor', ANCHOR_DEFAULT)} · "
                 f"{len(sk) if isinstance(sk, list) else 0} skip class(es) · this box: {this_box()}")
    sc = [s for n, s in man["scopes"].items() if n not in bl]
    held = sum("placeholder" in s for s in sc)
    counts = (f"list: {len(man['scopes'])} scopes · {len(sc) - held} runnable · {held} placeholder(s) · {len(bl)} NOT RUN · "
              f"{sum(bool(s.get('paths')) for s in sc)} with paths · {sum(bool(s.get('plants')) for s in sc)} with plants")
    return result(counts, lines=lines, nogo=("zero runnable scopes" + (f" - {len(bl)} NOT RUN, each named above" if bl else ""))
                  if held == len(sc) else None)


def loop_cases(rec, scope=None):
    """The case count a budget is stated against: one scope's, or the whole loop's."""
    cases = lambda s: int(entry_num(s, "passed") or 0) + int(entry_num(s, "failed") or 0)  # noqa: E731
    sc = (rec.get("scopes") or {}).get(scope or "")
    if isinstance(sc, dict) and sc:
        return cases(sc)
    return int(rec.get("cases") or sum(cases(s) for s in (rec.get("scopes") or {}).values()))


def loop_cpus(r, man, box=None):
    """The CPUs a record's load is read against: its own count, else the manifest box's."""
    n = r.get("cpus") or budget_box(man, box or r.get("box")).get("cpus")
    return n if is_num(n) and n > 0 else None


def norm_factor(r, man, box=None):
    """The factor taking a record's seconds to an idle box's, or None: its own `seconds_norm` when it holds one,
    else derived from its `load` and CPUs with today's k (a record from before the field); a null `seconds_norm`
    (no load sample, named by the runner) or no load is None - never 1, the raw read as idle."""
    s, sn = r.get("seconds"), r.get("seconds_norm")
    if "seconds_norm" in r and sn is None:
        return None
    if is_num(sn) and is_num(s) and s > 0:
        return float(sn) / float(s)
    box = box or r.get("box")
    load, cpus = r.get("load"), loop_cpus(r, man, box)
    if not is_num(load) or cpus is None:
        return None
    return 1.0 / (1.0 + norm_k(man, box, cpus)[0] * float(load) / cpus)


def counted(r, f, man, box=None):
    """A complete loop the normalised median and the TR line count: a load sample, and a load within its CPUs (an
    oversubscribed box measures the queue, not the tree)."""
    cpus, load = loop_cpus(r, man, box), r.get("load")
    return f is not None and not (cpus and is_num(load) and load > cpus)


def budget_lines(man, box, rows):
    """The manifest's per-box budget, applied per loop - a function of the case count, stated at an idle box: each
    loop's seconds normalised by its load are graded against it, never a constant. A loop with no load sample is
    graded raw, as at load 0, and says so."""
    bud = man.get("budget") or {}
    bb = (bud.get("boxes") or {}).get(box or "", {}) or {}
    if not bb.get("base_s"):
        return [f"{box or '?':<8} {len(rows)} loops · no budget measured for this box in the manifest - measure it "
                f"with {bud.get('measured_by', 'a TR block')}"]
    per = float(bb.get("per_case_s") or 0.0)
    band, ref, base = float(bb.get("band_pct") or 0) / 100.0, int(bb["at_cases"]), float(bb["base_s"])
    k, src = norm_k(man, box, bb.get("cpus") or (rows[-1].get("cpus") if rows else None))
    scope = bud.get("cases_scope")
    out = [f"{box or '?':<8} {len(rows)} loops · budget E = {base:g} s + {per:g} s/case x (cases - {ref}) at an idle box"
           f" · seconds normalised by (1 + k x load / cpus), k {k:g} ({src}) · band +/-{band * 100:.0f} % · stated for"
           f" load <= cpus" + (f" · {bb['measured']}" if bb.get("measured") else ""),
           f"{'':<8} {'task':<16}{'obs':>8}{'norm':>8}{'E':>8}{'delta':>8}{'load':>7}{'cases':>7}"]
    over, blind, oor = [], [], []
    for r in rows:
        cases, load, f = loop_cases(r, scope), r.get("load"), norm_factor(r, man, box)
        cpus = loop_cpus(r, man, box)
        obs, mark = float(r.get("seconds", 0)), ""
        norm = obs * f if f is not None else None
        val, exp = (norm if norm is not None else obs), base + per * (cases - ref)
        delta = round((val - exp) / exp * 100, 1) + 0.0         # + 0.0: a -0.0 % is +0.0 %
        if len(r.get("scopes") or {}) != len(man.get("scopes") or {}):
            mark = (f"  SCOPE SET DIFFERS ({len(r.get('scopes') or {})} of {len(man.get('scopes') or {})}): the budget "
                    "is stated against the manifest as it stands, so this loop is not graded")
        elif norm is None:
            blind.append(r.get("task"))
            mark = "  NO LOAD SAMPLE: graded raw, as at load 0, so E is a floor"
        elif cpus and load > cpus:
            oor.append(f"{r.get('task')} (load {load} > {cpus} cpus)")
            mark = "  OUT OF RANGE: the box is oversubscribed, so this run does not measure the tree"
        elif abs(val - exp) > band * exp:
            over.append(f"{r.get('task')} {delta:+.0f} %")
            mark = "  OVER BAND"
        out.append(f"{'':<8} {str(r.get('task')):<16}{obs:>8.1f}{(norm if norm is not None else float('nan')):>8.1f}"
                   f"{exp:>8.1f}{delta:>+7.1f}%"
                   f"{(load if is_num(load) else float('nan')):>7.2f}{cases:>7}{mark}")
    out.append(f"{'':<8} over band: {', '.join(over) or 'none'} · out of range: {', '.join(oor) or 'none'}"
               + (f" · NO LOAD SAMPLE: {len(blind)} of {len(rows)} ({', '.join(blind[:3])}"
                  f"{'...' if len(blind) > 3 else ''})" if blind else ""))
    return out


def run_key(r):
    return (r.get("mode"), tuple(sorted(r.get("scopes") or {})))


def own_record(r):
    """The loop_times.jsonl line as this runner's record, or None: foreign - counted apart, never read. Foreign is
    a line that is not JSON, `scopes` not a mapping (an older runner's list), or a field every report reads in
    another shape. An optional field in another shape (an older runner's `load` as an object) reads as absent."""
    if not (isinstance(r, dict) and not RECORD_KEYS - set(r) and isinstance(r["scopes"], dict) and is_num(r["seconds"])
            and all(isinstance(r[k], str) for k in ("ts", "task", "mode"))
            and all(r[k] is None or isinstance(r[k], str) for k in ("verdict", "tree"))):
        return None
    return {k: v for k, v in r.items() if not (k in ("box", "type", "model", "base", "load_norm") and not isinstance(v, str)
                                               or k in ("load", "cpus", "cases", "loop_seconds") and not is_num(v)
                                               or k in ("seconds_norm", "loop_seconds_norm") and not (v is None or is_num(v))
                                               or k == "touched" and not isinstance(v, dict))}


def loop_total(r):
    """A complete loop's seconds as the box paid them: the scopes' plus the loop's plants and flags (a record
    from before the loop was billed reads its loop as 0 - `report` names how many)."""
    return float(r.get("seconds", 0)) + (float(r["loop_seconds"]) if is_num(r.get("loop_seconds")) else 0.0)


def read_records(man, root):
    """(path, this runner's records in file order, foreign count) from <logs_dir>/loop_times.jsonl."""
    return records_in(logs_of(man, root) / "loop_times.jsonl")


def records_in(path):
    """(path, this runner's records in file order, foreign count) from this record file."""
    path, recs, foreign = Path(path), [], 0
    with contextlib.suppress(OSError):
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
            if not line.strip():
                continue
            try:
                r = own_record(json.loads(line))
            except ValueError:
                r = None
            if r is None:
                foreign += 1
            else:
                recs.append(r)
    return path, recs, foreign


def green_cover(recs, task, box, tree, scope, case=None):
    """(ts, entry) of the record this scope is already GO in, or None: the task's own records on this box, newest
    first, only those that ran the scope. The newest one that is not a red-arm campaign decides - another tree, or
    a scope that did not go GO, is None (a red always re-runs); a run of one other case or a partly vouched
    `--changed` run says nothing about this one and is read past; a full run covers any case, a `--case` run only
    its own (M0-T64c)."""
    if not tree:
        return None
    for r in reversed(recs):
        e = (r.get("scopes") or {}).get(scope)
        if r.get("task") != task or r.get("box") != box or r.get("mode") == "redarm" or e is None:
            continue
        if r.get("tree") != tree or not isinstance(e, dict) or e.get("verdict") != "GO":
            return None
        if e.get("vouched") or (r.get("mode") == "case" and (case is None or r.get("case") != case)):
            continue
        return r["ts"], e
    return None


def entry_num(s, key):
    """A scope entry's number, or None when the entry is not in this runner's shape (an older runner's list)."""
    v = s.get(key) if isinstance(s, dict) else None
    return v if is_num(v) else None


def cmd_report(man, root, since=None, budget=None):
    """From loop_times.jsonl: loops per task, redundant loops, first-try passes, the complete loop's median and
    slowest scopes per box, raw and at an idle box, the budget, and the TR line only when the median of the box's last
    TR_LOOPS counted loops, at an idle box, is over --budget (a loop with no load sample, or a load over its CPUs, is
    never counted - named). A record in another runner's shape is
    `foreign`: counted apart, the rest read - never a traceback, never a number read from a shape it cannot trust."""
    path, recs, foreign = read_records(man, root)
    recs = [r for r in recs if not since or r["ts"][:10] >= since]
    if not recs:
        return result(f"report: 0 runs · {foreign} foreign", lines=manifest_lines(man),
                      nogo=f"no record of this runner's in {rel(path, root)}" + (f" since {since}" if since else "")
                      + (f" - {foreign} foreign line(s): another runner's shape, or not JSON" if foreign else ""))
    by_task = {}
    for r in recs:
        by_task.setdefault(r.get("task"), []).append(r)
    looped = [t for t, rs in by_task.items() if any(r.get("mode") in ("all", "changed") for r in rs)]
    modes = ("all", "changed", "scope", "case", "redarm")
    lines = manifest_lines(man) + [f"{t}: " + " · ".join(f"{sum(r.get('mode') == m for r in rs)} {m}" for m in modes)
                                   + f" · first run {rs[0].get('verdict')}" for t, rs in by_task.items() if t in looped]
    lines.append(f"({len(by_task) - len(looped)} further task label(s) ran scoped, case or redarm runs only)")
    first = {t: rs[0].get("verdict") == "GO" for t, rs in by_task.items()}
    groups = {}
    for t, rs in by_task.items():
        if rs[0].get("type") or rs[0].get("model"):
            groups.setdefault(f"{rs[0].get('type') or '?'}·{rs[0].get('model') or '?'}", []).append(first[t])
    if groups:
        lines.append("first-try by type·model: " + " · ".join(f"{g} {sum(v)}/{len(v)}" for g, v in sorted(groups.items())))
    last = {}
    redundant = []
    for r in recs:                               # loops only: a scoped or case run while iterating is no loop
        if r.get("mode") not in ("all", "changed"):
            continue
        k = run_key(r)
        p = last.get(k)
        if p and p.get("verdict") == r.get("verdict") == "GO" and r.get("tree") and p.get("tree") == r.get("tree"):
            redundant.append((p, r))
        last[k] = r
    lines += [f"redundant: {y.get('task')} {y.get('ts')} ({y.get('mode')}) - a second GO run on the tree of "
              f"{x.get('task')} {x.get('ts')}" for x, y in redundant]
    loops = [r for r in recs if r.get("mode") == "all"]
    boxes, medians, trs = {}, [], []
    for r in loops:
        boxes.setdefault(r.get("box") or "?", []).append(r)
    for box, rows in boxes.items():                  # never one median over two boxes: two different budgets
        fs = [norm_factor(r, man) for r in rows]
        idle = [(r, f) for r, f in zip(rows, fs) if counted(r, f, man)]          # file order: the newest last
        oor = [r for r, f in zip(rows, fs) if f is not None and not counted(r, f, man)]
        per_scope = {}
        for r, f in zip(rows, fs):
            for name, s in (r.get("scopes") or {}).items():
                if entry_num(s, "seconds") is not None:              # an entry not in this runner's shape is not read
                    raw = float(s["seconds"])
                    per_scope.setdefault(name, ([], []))[0].append(raw)
                    if counted(r, f, man):
                        per_scope[name][1].append(raw * f)
        med = statistics.median(loop_total(r) for r in rows)             # the scopes and the loop, each second once
        nmed = statistics.median(loop_total(r) * f for r, f in idle) if idle else None
        medians.append(f"{box} {med:.2f} s" + (f" ({nmed:.2f} s idle)" if nmed is not None else " (idle n/a)"))
        k, src = norm_k(man, box, loop_cpus(rows[-1], man))
        lines.append(f"{box}: at an idle box, seconds / (1 + k x load / cpus), k {k:g} ({src}): median "
                     + (f"{nmed:.2f} s over {len(idle)} counted complete loop(s), raw {med:.2f} s" if idle
                        else f"n/a - no complete loop counted, raw {med:.2f} s")
                     + (f" · out of range {len(oor)} (load > cpus, never counted: "
                        + ", ".join(f"{r.get('task')} load {r.get('load')}" for r in oor[:3]) + ("..." if len(oor) > 3 else "")
                        + ")" if oor else "")
                     + (f" · NO LOAD SAMPLE {fs.count(None)} of {len(rows)} (never counted)" if None in fs else ""))
        billed = [float(r["loop_seconds"]) for r in rows if is_num(r.get("loop_seconds"))]
        if billed:
            lines.append(f"{box}: the loop's plants and flags, median {statistics.median(billed):.2f} s over {len(billed)} "
                         f"complete loop(s) - counted in the median and the TR line")
        slow = sorted(((statistics.median(v[0]), k_, v[1]) for k_, v in per_scope.items()), reverse=True)
        if slow:
            lines.append(f"{box}: slowest scopes (median over {len(rows)} complete loops, idle over {len(idle)}): "
                         + " · ".join(f"{k_} {m:.2f} s" + (f" ({statistics.median(n):.2f} s idle)" if n else "")
                                      for m, k_, n in slow[:5]))
        if man.get("budget") and "budget" not in top_errors(man):      # a bad budget: its NOT RUN line above
            lines += budget_lines(man, box, rows)
        if budget is None:
            continue
        last = idle[-TR_LOOPS:]
        if not last:
            lines.append(f"TR: {box} - no complete loop with a load sample within its CPUs ({fs.count(None)} without a "
                         f"sample, {len(oor)} out of range): the TR line cannot read this box")
            continue
        tmed = statistics.median(loop_total(r) * f for r, f in last)
        if tmed > budget:
            trs.append(box)
            own = statistics.median(loop_total(r) - float(r.get("seconds", 0)) for r, _f in last)
            lines.append(f"TR: the last {len(last)} counted complete loop(s) on {box} take {tmed:.2f} s at an idle box "
                         f"(median; raw {statistics.median(loop_total(r) for r, _f in last):.2f} s, the loop's own "
                         f"{own:.2f} s in it) - over the --budget {budget:g} s: a TR fires (§4 Rule 4)")
    kept = [r for r in recs if isinstance(r.get("not_rerun"), dict) and r["not_rerun"]]
    n_kept = sum(len(r["not_rerun"]) for r in kept)
    if kept:
        lines.append(f"not re-run: {n_kept} scope(s) in {len(kept)} run(s) - already GO on the same tree, the task's own "
                     "record; not a redundant loop, and no seconds in any median")
    unbilled = [r for r in loops if not is_num(r.get("loop_seconds"))]
    if unbilled:
        since_b = next((r["ts"][:10] for r in loops if is_num(r.get("loop_seconds"))), None)
        lines.append((f"loop seconds unknown before {since_b}" if since_b else "loop seconds unknown: no complete loop "
                      "records them") + f" - {len(unbilled)} complete loop(s) read as 0")
    n_loop = sum(r.get("mode") in ("all", "changed") for r in recs)
    counts = (f"report: {len(recs)} runs · {foreign} foreign · {len(loops)} complete loops · "
              f"{sum(r.get('mode') == 'changed' for r in recs)} changed · {len(by_task)} task label(s) · loops per "
              f"task {n_loop / (len(looped) or 1):.2f} · redundant {len(redundant)} · first-try "
              f"{sum(first.values())}/{len(first)}" + (f" · not re-run {n_kept}" if kept else "")
              + f" · median complete loop {' · '.join(medians) or 'n/a'}"
              + (f" · TR {', '.join(trs) or 'none'} (budget {budget:g} s)" if budget is not None else ""))
    return result(counts, lines=lines, redundant=len(redundant), loops=len(loops), first_try=sum(first.values()),
                  tasks=len(first), tr=trs, foreign=foreign)


def cmd_touched(man, root, task=None, pattern=None):
    """From the claim records (`--changed`, `--all`): per task, the files it touched at its last claim; with
    --path, per matching path every task that touched it - newest first. A claim from before the field is
    counted, never read as `touched nothing`."""
    path, recs, foreign = read_records(man, root)
    claims = [r for r in recs if r["mode"] in ("all", "changed")]
    named = [r for r in claims if isinstance(r.get("touched"), dict)]
    last = {}
    for r in named:                                  # file order is time order: the last claim wins
        last[r["task"]] = r
    if task is not None:
        last = {task: last[task]} if task in last else {}
    m = globs([pattern]) if pattern else None
    rows = {t: [(p, k) for k in TOUCH_KINDS for p in (r["touched"].get(k) or []) if isinstance(p, str)
                and (m is None or m.match(p))] for t, r in last.items()}
    order = sorted(last, key=lambda t: last[t]["ts"], reverse=True)
    lines = []
    if m is None:
        for t in order:
            r = last[t]
            lines.append(f"{t} · claim {r['ts']} ({r['mode']}) · base {r.get('base', 'not recorded')} · {len(rows[t])} file(s)")
            lines += [f"  {k}: " + ", ".join(p for p, kk in rows[t] if kk == k) for k in TOUCH_KINDS
                      if any(kk == k for _p, kk in rows[t])]
    by_path = {}
    for t in order:
        for p, k in rows[t]:
            by_path.setdefault(p, []).append(f"{t} ({k}, {last[t]['ts']})")
    if m is not None:
        lines += [f"{p}: " + " · ".join(v) for p, v in sorted(by_path.items())]
    counts = (f"touched: {sum(1 for t in order if rows[t])} task(s) · {len(by_path)} path(s) · {len(named)} claim "
              f"record(s) naming their files · {len(claims) - len(named)} without (before the field) · {foreign} foreign")
    nogo = (f"no claim record names the files it touched in {rel(path, root)}" if not named
            else f"no claim record of {task} names its files" if not last
            else f"no task's last claim touched a path matching `{pattern}`" if not by_path and m is not None
            else None)
    return result(counts, lines=lines, nogo=nogo, tasks={t: dict(last[t]["touched"], base=last[t].get("base"),
                                                                  ts=last[t]["ts"]) for t in order}, paths=by_path)


# ---------------------------------------------------------------------------------------------- selftest
EMIT = r'''import os, sys, time
t0, kind, args, tag = time.time(), sys.argv[1], sys.argv[2:], None
if kind == "ls":
    import glob
    kind, args = "rows", sorted([os.path.join(d, f).replace(os.sep, "/") for d, _s, fs in os.walk(args[0]) for f in fs]
                                + [g.replace(os.sep, "/") for a in args[1:] for g in glob.glob(a)])
if kind == "deny":
    words, body = open(args[0], encoding="utf-8").read().split(), open(args[1], encoding="utf-8").read()
    kind, tag = ("nogo" if any(w in body for w in words) else "go"), args[1]
if kind == "sleep":
    open("sleep.pid", "w").write(str(os.getpid()))
if kind in ("trace", "sleep"):
    time.sleep(float(args[-1]))
if kind == "seelog":
    lines = open(args[0], encoding="utf-8").read().splitlines() if os.path.isfile(args[0]) else []
    kind = "go" if lines and lines[-1].startswith("== ") and "seelog" in lines[-1] else "nogo"
if kind == "marker":
    open("ran.marker", "w").close()
if kind == "check":
    if len(args) > 1:
        time.sleep(float(args[1]))
    kind = "nogo" if "BUG" in open(args[0], encoding="utf-8").read() else "go"
if kind == "fault":
    import faulthandler
    kind = "go" if faulthandler.is_enabled() else "nogo"
if kind == "pbpy":
    kind = "go" if os.path.realpath(os.environ.get("PB_PYTHON", "-")) == os.path.realpath(sys.executable) else "nogo"
if kind == "exit":
    text, rc = "", int(args[0])
elif kind == "verdict":
    text, rc = ("\n".join(args[1:]) + "\n=== %s ===" % args[0]), (0 if args[0] == "GO" else 1)
elif kind == "count":
    text, rc = "checked: %s repositories\n=== GO ===" % args[0], 0
else:
    text, rc = {"nogo": ("checks=1 failed=1\nFAIL planted\n=== NO-GO: 1 check(s) failed ===", 1),
                "lie": ("checks=1 failed=1\n=== GO ===", 1),
                "mute": ("checks=1 failed=0", 0),
                "raw": (" ".join(args), 0),
                "rows": ("\n".join(args), 0)}.get(kind, ("checks=1 failed=0\n=== GO ===", 0))
print(text)
trace = os.environ.get("PB_TRACE")
if trace:
    line, lock, until = "%s %.6f %.6f\n" % (tag or (args[0] if kind == "trace" else kind), t0, time.time()), trace + ".lock", time.time() + 5
    while lock:                       # one writer at a time: two appends at once tear a line on Windows; 5 s at most
        try:
            os.close(os.open(lock, os.O_CREAT | os.O_EXCL))
            break
        except OSError:               # held (or being deleted, on Windows)
            lock = lock if time.time() < until else None
            time.sleep(0.005)
    with open(trace, "a") as f:
        f.write(line)
    if lock:
        os.remove(lock)
sys.exit(rc)
'''


def selftest():
    """A temp root and a fake command per scope: GO clean; every planted violation must redden on its OWN
    reason. Never touches the real logs, the real anchor or the tree it sits in."""
    global PROGRESS
    checks, failed, plants = [], [], [0, 0]           # plants: [planted, red]
    saved = {k: os.environ.get(k) for k in ("PB_TASK", "PB_TRACE", "PB_BOX", "GIT_CEILING_DIRECTORIES",
                                            "PYTHONFAULTHANDLER", "PB_PYTHON")}
    saved_progress, PROGRESS = PROGRESS, False

    def check(name, ok, detail=""):
        checks.append(name)
        if not ok:
            failed.append(f"{name}: {detail}" if detail else name)

    def gone(pid):                                  # the killed child is no process any more (POSIX)
        if os.name == "nt":
            return True
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return True
        except PermissionError:
            return False
        return False

    def plant_(name, ok, detail=""):
        plants[0] += 1
        plants[1] += bool(ok)
        check(f"PLANT {name}", ok, detail)

    def swap_(fn_name, mutant, call):                    # one mutant of this runner, put back whatever happens
        real = globals()[fn_name]
        globals()[fn_name] = mutant
        try:
            return call()
        finally:
            globals()[fn_name] = real

    try:
        with tempfile.TemporaryDirectory(prefix="pb-verify-") as tmpd:
            tmp = Path(tmpd)
            os.environ.pop("PB_TASK", None)
            os.environ.pop("PYTHONFAULTHANDLER", None)               # the harness, not the caller, must set it
            os.environ.pop("PB_PYTHON", None)                        # the same
            os.environ["PB_BOX"] = "SELFBOX"
            os.environ["GIT_CEILING_DIRECTORIES"] = str(tmp.parent)   # the fixture root is no git checkout
            trace = tmp / "trace.txt"
            os.environ["PB_TRACE"] = str(trace)
            (tmp / "emit.py").write_text(EMIT, encoding="utf-8")
            (tmp / "extra").mkdir()
            (tmp / "extra" / "doc.md").write_text("v1\n", encoding="utf-8")
            py, logs = "{python} -S {emit}", tmp / "logs"
            base_top = {"logs_dir": "logs", "tree_extra": ["extra"], "vars": {"emit": ["emit.py"]}}
            rx = r"(?P<passed>\d+) passed, (?P<failed>\d+) failed"
            clean = {"a": {"cmd": f"{py} go"},
                     "listed": {"cases": ["x", "y"], "case_cmd": f"{py} go {{case}}", "expected": 2},
                     "object": {"cases": {"p": f"{py} go", "q": f"{py} go"}},
                     "found": {"cases_cmd": f"{py} rows u v w", "case_cmd": f"{py} go {{case}}", "expected": 3},
                     "counted": {"cmd": f"{py} count 3", "count": r"checked: (?P<n>\d+) repositories", "expected": 3},
                     "line": {"cmd": f"{py} verdict GO '3 passed, 0 failed, 1 skipped'", "expected": 3},
                     "quoted": {"cmd": f"{py} verdict GO '9 passed, 0 failed, 0 skipped' 'checks=1 failed=0'"},
                     "rx": {"cmd": f"{py} raw '7 passed, 0 failed'", "parse": "regex", "regex": rx},
                     "pyt": {"cmd": f"{py} raw '== 5 passed, 1 skipped in 0.1s =='", "parse": "pytest"},
                     "jst": {"cmd": f"{py} raw 'Tests: 1 skipped, 7 passed, 8 total'", "parse": "jest"},
                     "ext": {"cmd": f"{py} exit 0", "parse": "exit"},
                     "seen": {"cmd": f"{py} seelog {shlex.quote(str(tmp / 'logs' / 'T9.seen.log'))}"},
                     "t1": {"cmd": f"{py} trace t1 0.2"}, "t2": {"cmd": f"{py} trace t2 0.2"},
                     "here": {"cmd": f"{py} go", "box": "selfbox"},
                     "away": {"cmd": f"{py} go", "box": "OTHERBOX"},
                     "later": {"placeholder": "M9-T9", "expected": 0},
                     "alone": {"cmd": f"{py} trace alone 0.01", "exclusive": True}}

            def run_(scopes, *args, task="T9", **top):
                mpath = tmp / "verify.json"
                mpath.write_text(json.dumps({**base_top, **top, "scopes": scopes}), encoding="utf-8")
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    rc = main(["--manifest", str(mpath), "--root", str(tmp), "--no-progress"]
                              + (["--task", task] if task else []) + list(args))
                return rc, out.getvalue()

            def red(change, scope, why):
                """the planted scope beside one clean scope -> NO-GO naming the planted one alone, on its reason"""
                rc, out = run_({"ok": clean["a"], **change}, "--all")
                fails = [ln for ln in out.splitlines() if ln.startswith("  ")]
                return (rc == 1 and bool(fails) and all(ln.strip().startswith(f"{scope}:") for ln in fails)
                        and why in out), out[-400:]

            def spans():                    # whole records only: a torn one reads missing on its check, never a crash
                return [(m[1], float(m[2]), float(m[3])) for s in trace.read_text(encoding="utf-8").splitlines()
                        if (m := re.fullmatch(r"(\S+) (\d+\.\d+) (\d+\.\d+)", s))]

            # -- A · the clean fixture: every form counts right, and the run's shape ---------------------------
            trace.write_text("", encoding="utf-8")
            rc, out = run_(clean, "--all", "-j", "4", smoke=f"{py} trace smoke 0.01")
            got = {ln.split()[0]: ln.split()[1] for ln in out.splitlines() if len(ln.split()) > 1}
            check("clean fixture: --all GO, exit 0", rc == 0 and out.rstrip().endswith("=== GO ==="), out[-600:])
            check("counts per scope: a list 2 · an object 2 · cases_cmd 3 · count 3 · a counts line 3+1 skip · one quoted "
                  "further up 1 · regex 7 · pytest 5+1 skip · jest 7+1 skip · exit 1 · the smoke 1",
                  all(got.get(n) == c for n, c in (("listed", "2/0/0"), ("object", "2/0/0"), ("found", "3/0/0"),
                                                   ("counted", "3/0/0"), ("line", "3/0/1"), ("quoted", "1/0/0"), ("rx", "7/0/0"),
                                                   ("pyt", "5/0/1"), ("jst", "7/0/1"), ("ext", "1/0/0"),
                                                   ("smoke", "1/0/0"))), got)
            check("a placeholder prints SKIP, counts nowhere, opens no log",
                  re.search(r"^later +SKIP \(placeholder until M9-T9\)$", out, re.M) is not None
                  and not (logs / "T9.later.log").exists(), out)
            check("a scope bound to this box runs; one bound to another reads `owed on`, never NO-GO, opens no log",
                  got.get("here") == "1/0/0" and re.search(r"^away +owed on OTHERBOX", out, re.M) is not None
                  and "1 owed" in out and not (logs / "T9.away.log").exists(), out)
            check("each scope's log is open before its command starts (`seen` read its own header last)",
                  got.get("seen") == "1/0/0" and all((logs / f"T9.{n}.log").is_file()
                                                     for n in clean if n not in ("later", "away")), got.get("seen"))
            check("no part file is left beside a log", not list(logs.glob("*.part")), list(logs.glob("*.part")))
            sp = spans()
            last = {s[0]: s for s in sp if s[0] in ("alone", "smoke")}
            check("the exclusive scope ran alone after the rest, the smoke last",
                  len(last) == 2 and last["alone"][1] >= max(s[2] for s in sp if s[0] not in last)
                  and last["smoke"][1] >= last["alone"][2], sp)
            t = {s[0]: s for s in sp if s[0] in ("t1", "t2")}
            check("two shared scopes ran at the same time",
                  len(t) == 2 and t["t1"][1] < t["t2"][2] and t["t2"][1] < t["t1"][2], t)
            recs = [json.loads(ln) for ln in (logs / "loop_times.jsonl").read_text(encoding="utf-8").splitlines()]
            check("loop_times.jsonl: one line per run, with the box and the interpreter",
                  len(recs) == 1 and RECORD_KEYS | {"box", "python", "load_source", "cases"} <= set(recs[0])
                  and recs[0]["box"] == "SELFBOX" and recs[0]["python"] == PYTHON and recs[0]["mode"] == "all"
                  and recs[0]["verdict"] == "GO" and recs[0]["scopes"]["later"]["verdict"] == "SKIP"
                  and recs[0]["scopes"]["away"]["verdict"] == "OWED", recs)
            check("the record's cases match the scopes'",
                  recs[0]["cases"] == sum(s.get("passed", 0) + s.get("failed", 0) for s in recs[0]["scopes"].values()))
            trace.write_text("", encoding="utf-8")
            rc, out = run_({"a": clean["a"], "t1": {"cmd": f"{py} trace t1 0.1"}, "t2": {"cmd": f"{py} trace t2 0.1"}},
                           "--all", "--serial")
            sp = sorted(spans(), key=lambda s: s[1])
            check("--serial: no two commands overlap",
                  rc == 0 and len(sp) == 3 and all(b[1] >= x[2] for x, b in zip(sp, sp[1:])), sp)
            trace.write_text("t1 10.000000 12.000000\nt2 11.0\n", encoding="utf-8")    # a torn record: two appends at once
            t = {s[0]: s for s in spans() if s[0] in ("t1", "t2")}
            plant_("a torn trace record: read missing, its check red - never an IndexError",
                   len(t) != 2 and spans() == [("t1", 10.0, 12.0)], spans())

            # -- B · plants: each beside a clean scope, each NO-GO on its own reason -------------------------
            with socket.socket() as held:
                held.bind(("127.0.0.1", 0))
                held.listen(1)
                port = held.getsockname()[1]
                for name, change, scope, why in (
                        ("failed=1", {"a": {"cmd": f"{py} nogo"}}, "a", "1 failed"),
                        ("a scope that ran zero cases", {"rx": {**clean["rx"], "cmd": f"{py} raw '0 passed, 0 failed'"}},
                         "rx", "zero passed"),
                        ("`count` reads 0", {"counted": {**clean["counted"], "expected": 0, "cmd": f"{py} count 0"}},
                         "counted", "zero passed"),
                        ("`count` reads fewer than expected", {"counted": {**clean["counted"], "cmd": f"{py} count 2"}},
                         "counted", "2 case(s) < expected 3"),
                        ("`cases_cmd` loses a case", {"found": {**clean["found"], "cases_cmd": f"{py} rows u v"}},
                         "found", "2 case(s) < expected 3"),
                        ("`cases_cmd` prints an id with a space", {"found": {**clean["found"],
                                                                            "cases_cmd": f"{py} rows u 'v w' x"}},
                         "found", "printed a bad id 'v w'"),
                        ("the last counts line wins", {"line": {"cmd": f"{py} verdict GO '  -> 5 passed, 0 failed, 0 skipped' "
                                                                       f"'0 passed, 1 failed, 0 skipped'"}}, "line", "1 failed"),
                        ("a GO line with exit 1", {"a": {"cmd": f"{py} lie"}}, "a", "says GO but the exit code is 1"),
                        ("no verdict line", {"a": {"cmd": f"{py} mute"}}, "a", "no `=== GO ===`"),
                        ("its own NO-GO", {"a": {"cmd": f"{py} verdict 'NO-GO: foreign stack' 'checks=4 failed=0'"}},
                         "a", "foreign stack"),
                        ("a pytest failure", {"pyt": {**clean["pyt"], "cmd": f"{py} raw '== 1 failed, 4 passed in 0.2s =='"}},
                         "pyt", "1 failed"),
                        ("a jest failure", {"jst": {**clean["jst"], "cmd": f"{py} raw 'Tests: 1 failed, 4 passed, 5 total'"}},
                         "jst", "1 failed"),
                        ("a regex failure", {"rx": {**clean["rx"], "cmd": f"{py} raw '3 passed, 2 failed'"}}, "rx", "2 failed"),
                        ("an exit parser's exit 2", {"ext": {"cmd": f"{py} exit 2", "parse": "exit", "good": "exit 0"}},
                         "ext", "exit 2, want: exit 0"),
                        ("a busy free port", {"a": {"cmd": f"{py} marker", "free_ports": [port]}}, "a", f"NOT RUN (:{port}"),
                        ("a command not on PATH", {"a": {"cmd": "no-such-command-pb-verify --x"}}, "a", "not started")):
                    ok, detail = red(change, scope, why)
                    plant_(name, ok, detail)
                check("a busy port's command never ran, its holder untouched",
                      not (tmp / "ran.marker").exists() and held.fileno() != -1)
            t0 = time.monotonic()
            ok, detail = red({"a": {"cmd": f"{py} sleep 20", "timeout_s": 0.2}}, "a", "timed out after 0.2 s")
            plant_("a command that hangs: killed at its timeout, its log kept",
                   ok and time.monotonic() - t0 < 10 and "exit=timeout" in (logs / "T9.a.log").read_text(encoding="utf-8")
                   and gone(int((tmp / "sleep.pid").read_text())), detail)
            rc, out = run_({"later": clean["later"]}, "--all")
            plant_("nothing but placeholders: zero passed in total", rc == 1 and "zero passed in total" in out, out[-300:])

            # -- C · a kept manifest, read as far as it can be: a note is a note; a scope this runner cannot read
            #    whole is NOT RUN, named, never run, while the rest run - and the loop is not complete -----------
            tr = lambda n: f"{py} trace {n} 0"  # noqa: E731
            planted = {   # scope -> (its definition, what its NOT RUN line names): each a shape the fleet's manifests hold
                "f_env": ({"cmd": tr("f_env"), "env": {"OLLAMA_HOST": "http://127.0.0.1:1"}}, "unknown key `env`"),
                "f_lane": ({"cmd": tr("f_lane"), "lane": "gpu", "vram_mib": 1024}, "unknown keys `lane`, `vram_mib`"),
                "f_inject": ({"cmd": tr("f_inject"), "inject": ["p"], "inject_cmd": "{python} x.py"}, "`inject_cmd`"),
                "f_requires": ({"cmd": tr("f_requires"), "requires": [{"port_free": 5630}]}, "unknown key `requires`"),
                "f_arms": ({"cmd": tr("f_arms"), "arms": {"--red-leak": "split"}}, "unknown key `arms`"),
                "f_covers": ({"cmd": tr("f_covers"), "covers": ["src/*.py"]}, "unknown key `covers`"),
                "v_parse": ({"cmd": tr("v_parse"), "parse": "unittest"}, "`parse` is one of"),
                "v_exclusive": ({"cmd": tr("v_exclusive"), "exclusive": "gpu"}, "`exclusive` is true or false"),
                "v_rerun": ({"cmd": tr("v_rerun"), "rerun": "yes"}, "`rerun` is true or false"),
                "v_placeholder": ({"placeholder": "M9", "cmd": tr("v_placeholder")}, "a placeholder runs nothing"),
                "v_nocmd": ({"parse": "gonogo"}, "needs exactly one of"),
                "v_regex": ({"cmd": tr("v_regex"), "parse": "regex", "regex": r"(\d+) ok"}, "has no (?P<passed>"),
                "v_casecmd": ({"cmd": tr("v_casecmd"), "case_cmd": f"{py} x"}, "holds `{case}`"),
                "v_plant": ({"cmd": tr("v_plant"), "plants": [{"id": "p"}]}, "a plant is"),
                "v_paths": ({"cmd": tr("v_paths"), "paths": "src/*.py"}, "`paths` is a list"),
                "smoke": ({"cmd": tr("smoke")}, "a scope name is lowercase"),
                "t_placehoder": ({"cmd": tr("t_placehoder"), "placehoder": "M9"}, "`placehoder` (did you mean `placeholder`?)"),
                "t_expcted": ({"cmd": tr("t_expcted"), "expcted": 3}, "`expcted` (did you mean `expected`?)")}
            noted = {"cmd": tr("noted"), "desc": "what it checks", "why": "w", "hint": "h", "reason": "r", "note": "n",
                     "_expected": 99, "_timeout_s": 0.001, "_inject": ["p"], "_": ["a", "b"]}
            trace.write_text("", encoding="utf-8")
            rc, out = run_({"ok": {"cmd": tr("ok")}, "noted": noted, **{n: d for n, (d, _w) in planted.items()}}, "--all")
            ran_, fails = {s[0] for s in spans()}, [ln.strip() for ln in out.splitlines() if ln.startswith("  ")]
            for n, (_d, why) in planted.items():
                line = next((ln for ln in out.splitlines() if re.match(rf"{n} +NOT RUN \(", ln)), "")
                plant_(f"a kept manifest's `{n}`: NOT RUN, named on its line, never run, --all NO-GO on it",
                       why in line and n not in ran_ and any(f.startswith(f"{n}: NOT RUN (") for f in fails), line or out[-300:])
            plant_("notes (`_` keys, desc, why, hint, reason, note) read, never acted on: `_expected: 99`, `_timeout_s` "
                   "aside, the scope ran GO on its one case", re.search(r"^noted +1/0/0 .*GO( · step \d+/\d+, \d+ left)?$", out, re.M) is not None, out[-600:])
            check("a kept manifest: the scopes it understands ran; each NOT RUN scope counted, no log opened for it",
                  rc == 1 and ran_ == {"ok", "noted"} and len(fails) == len(planted) and f"{len(planted)} NOT RUN" in out
                  and not (logs / "T9.f_env.log").exists(), (ran_, fails[:3]))
            trace.write_text("", encoding="utf-8")
            rc, out = run_({"ok": {"cmd": tr("ok")}}, "--all", _comment="a note", note="a note too",
                           **{"//": "a comment", "lanes": {"gpu": "GPU 0"}, "timeout_scale": 3})
            warns = [ln for ln in out.splitlines() if ln.startswith("WARN ")]
            plant_("unknown top-level keys: one WARN line each, named, the verdict not moved; a top-level note: none",
                   rc == 0 and len(warns) == 3 and all(any(f"`{k}`" in x for x in warns) for k in ("//", "lanes", "timeout_scale"))
                   and "_comment" not in out, warns or out[-300:])
            trace.write_text("", encoding="utf-8")
            rc, out = run_({"ok": {"cmd": tr("ok")}}, "--all", smoke={"posix": tr("smk"), "nt": tr("smk")})
            plant_("a `smoke` object (a known key, a bad value): the smoke NOT RUN, named, never run; the scopes ran; "
                   "--all NO-GO", rc == 1 and re.search(r"^smoke +NOT RUN \(`smoke` is a command or null\)", out, re.M) is not None
                   and {s[0] for s in spans()} == {"ok"} and "smoke: NOT RUN" in out, out[-400:])
            trace.write_text("", encoding="utf-8")
            rc, out = run_({"t1": {"cmd": f"{py} trace t1 0.1"}, "t2": {"cmd": f"{py} trace t2 0.1"}}, "--all", jobs_cap="8")
            sp = sorted(spans(), key=lambda s: s[1])
            plant_("`jobs_cap` \"8\" (a string): NOT RUN, named - one command at a time, never eight at once",
                   rc == 0 and "NOT RUN (`jobs_cap` is an integer >= 1)" in out and " jobs 1 " in out and len(sp) == 2
                   and sp[1][1] >= sp[0][2], sp)
            trace.write_text("", encoding="utf-8")
            rc, out = run_({"ok": {"cmd": tr("ok")}}, "--all", logs_dir="milestones/{tenant}/logs")
            plant_("`logs_dir` holding `{tenant}` (a placeholder nothing fills): the run refused, named - nothing ran, no "
                   "`{tenant}` folder", rc == 1 and "`logs_dir` holds `{tenant}`" in out and not trace.read_text(encoding="utf-8")
                   and not (tmp / "milestones").exists(), out[-300:])
            rc, out = run_({"ok": {"cmd": tr("ok")}}, "list", logs_dir="milestones/{tenant}/logs")
            check("... while `list` reads the scopes, the logs named NOT RUN", rc == 0 and "NOT RUN (`logs_dir` holds" in out, out[-300:])
            for name, text, why in (("a manifest that is not JSON", "{ not json", "JSONDecodeError"),
                                    ("a manifest with no `scopes`", json.dumps(dict(base_top, _note="x")), "`scopes` is missing")):
                (tmp / "verify.json").write_text(text, encoding="utf-8")
                trace.write_text("", encoding="utf-8")
                with contextlib.redirect_stdout(io.StringIO()) as buf:
                    rc = main(["--manifest", str(tmp / "verify.json"), "--root", str(tmp), "--no-progress", "--task", "T9", "--all"])
                plant_(f"{name}: refused whole, nothing run", rc == 1 and why in buf.getvalue()
                       and not trace.read_text(encoding="utf-8"), buf.getvalue()[-300:])
            mixed = {"ok": {"cmd": tr("ok")}, "gpu": {"cmd": tr("gpu"), "case_cmd": f"{py} go {{case}}", "lane": "gpu"}}
            trace.write_text("", encoding="utf-8")
            rc, out = run_(mixed, "ok", "gpu")
            plant_("a scoped run is judged on the scopes it ran: `ok` GO, `gpu` NOT RUN on its line and counted, never run",
                   rc == 0 and re.search(r"^gpu +NOT RUN \(unknown key `lane`\)", out, re.M) is not None
                   and "1 NOT RUN" in out and {s[0] for s in spans()} == {"ok"}, out[-300:])
            rc, out = run_(mixed, "gpu")
            plant_("a scoped run of a NOT RUN scope alone: NO-GO, nothing ran",
                   rc == 1 and "every scope named is NOT RUN" in out, out[-300:])
            rc, out = run_(mixed, "gpu", "--case", "x")
            plant_("--case on a NOT RUN scope: NO-GO, never run", rc == 1 and "every scope named is NOT RUN" in out
                   and {s[0] for s in spans()} == {"ok"}, out[-300:])
            rc, out = run_(mixed, "--redarm", "gpu")
            plant_("--redarm of a NOT RUN scope: refused", rc == 1 and "is NOT RUN (unknown key `lane`)" in out, out[-300:])
            rc, out = run_(mixed, "list")
            check("list: a NOT RUN scope named on its line and counted; GO while one scope runs",
                  rc == 0 and "list: 2 scopes · 1 runnable · 0 placeholder(s) · 1 NOT RUN" in out
                  and re.search(r"^gpu +NOT RUN \(unknown key `lane`\)", out, re.M) is not None, out[-300:])
            rc, out = run_({"gpu": mixed["gpu"]}, "list")
            plant_("list: every scope NOT RUN - zero runnable, NO-GO", rc == 1 and "zero runnable scopes - 1 NOT RUN" in out,
                   out[-300:])

            # -- C' · the call, the task: refused before anything runs ------------------------------------------
            n_rec = len((logs / "loop_times.jsonl").read_text(encoding="utf-8").splitlines())
            trace.write_text("", encoding="utf-8")
            rc, out = run_(clean, "--all", task=None)
            plant_("no --task and no $PB_TASK: refused, no log, no record", rc == 1 and "no --task" in out
                   and not trace.read_text(encoding="utf-8"), out[-300:])
            rc, out = run_(clean, "--all", task="T 9;x")
            plant_("a task id that is no id: refused", rc == 1 and "is not an id" in out, out[-200:])
            check("a refused run appended no record",
                  len((logs / "loop_times.jsonl").read_text(encoding="utf-8").splitlines()) == n_rec)
            for name, args, want in (("--all with a scope", ("--all", "a"), "the calls:"),
                                     ("--base without --changed", ("a", "--base", "abc"), "the calls:"),
                                     ("--redarm beside --all", ("--all", "--redarm", "a"), "the calls:"),
                                     ("no scope at all", (), "the calls:"),
                                     ("an unknown flag", ("--frobnicate",), "the calls:")):
                rc, out = run_(clean, *args)
                plant_(f"a wrong call ({name}) prints the right one", rc == 1 and want in out and "--changed" in out, out[-300:])
            mine = logs / "T9.a.log"
            before = mine.read_bytes()
            try:
                run_scopes({"scopes": {"a": clean["a"]}, **base_top}, ["a"], tmp, "T9", 1, tracked_fn=lambda r, ps: list(ps))
                plant_("a log git already tracks: refused", False, "it ran")
            except Refused as exc:
                plant_("a log git already tracks: refused", "git already tracks it" in str(exc) and mine.read_bytes() == before,
                       str(exc))
            check("outside a work tree nothing reads as tracked", tracked(tmp, [mine]) == [])
            rc, out = run_(clean, "listed", "--case", "x")
            check("--case runs one case of a list, PARTIAL", rc == 0 and re.search(r"^listed +1/0/0", out, re.M) is not None
                  and "PARTIAL" in out, out[-300:])
            rc, out = run_(clean, "object", "--case", "q")
            check("--case runs one case of an object", rc == 0 and re.search(r"^object +1/0/0", out, re.M) is not None, out[-300:])
            for name, s, c in (("an unknown case", "object", "nope"), ("a scope without cases", "a", "x"),
                               ("a shell character", "listed", "x;y")):
                plant_(f"--case refuses {name}", run_(clean, s, "--case", c)[0] == 1)
            plant_("an unknown scope: NO-GO, nothing run", run_(clean, "nope")[0] == 1)

            # -- D · content, the tree a run graded --------------------------------------------------------
            crlf = lambda b: b.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")  # noqa: E731
            text_lf, edit_lf = b"line one\nline two\n", b"line one\nline TWO\n"
            check("a content hash survives a CRLF checkout", content_sha1(text_lf) == content_sha1(crlf(text_lf)))
            check("a content hash still moves on a real edit", content_sha1(text_lf) != content_sha1(edit_lf)
                  and content_sha1(crlf(text_lf)) != content_sha1(crlf(edit_lf)))
            blob = b"\x89PNG\r\n\x1a\n\x00\x00rows\r\nmore\r\n"
            check("a binary is never translated", content_sha1(blob) == hashlib.sha1(blob).hexdigest())
            troot = tmp / "tree"
            for d in ("tools/pb", "logs", "extra/logs", "extra/notes"):
                (troot / d).mkdir(parents=True)
            for p, b in ((".gitignore", b"extra/\n"), ("tools/pb/plan.py", b"P"), ("tools/pb/verify.anchor.json", b"{}"),
                         ("logs/r.log", b"L"), ("extra/notes/n.md", b"N"), ("extra/logs/x.log", b"X")):
                (troot / p).write_bytes(b)
            tman = {"logs_dir": "logs", "tree_extra": ["extra"], "scopes": {}}
            tgit = lambda r, args, timeout=120: "0123456789abcdef\n"  # noqa: E731
            nogit = lambda r, args, timeout=120: None  # noqa: E731
            tid = lambda man=tman, g=tgit: tree_id(troot, man, troot / "logs", g)  # noqa: E731
            t0_ = tid()
            (troot / "tools/pb/verify.anchor.json").write_bytes(b'{"paths": {"advanced": 1}}')
            (troot / "logs/r.log").write_bytes(b"L and this run's own log")
            (troot / "extra/logs/x.log").write_bytes(b"X moved")
            check("the tree id holds out the anchor and every logs folder", t0_ is not None and tid() == t0_)
            check("the tree id reads the anchor's path from the manifest", tid(dict(tman, anchor="tools/pb/elsewhere.json")) != t0_)
            (troot / "tools/pb/plan.py").write_bytes(b"P edited")
            t1_ = tid()
            check("the tree id sees a byte of an edited file", t1_ != t0_)
            (troot / "extra/notes/n.md").write_bytes(b"N edited")
            check("the tree id sees a byte under tree_extra", tid() != t1_)
            t2_ = tid()
            check("the tree id leads with HEAD where a repository answers", t2_.startswith("0123456789ab+"))
            check("with no repository it is the same hash alone", tid(g=nogit) == t2_.split("+", 1)[1])
            (troot / "tools/pb/plan.py").write_bytes(text_lf)
            t_lf = tid()
            (troot / "tools/pb/plan.py").write_bytes(crlf(text_lf))
            check("one commit's two checkouts name one tree", t_lf is not None and tid() == t_lf)
            if hasattr(os, "geteuid") and os.geteuid() != 0:
                (troot / "tools/pb/plan.py").chmod(0)
                check("the tree id is None when a listed file cannot be read", tid() is None)
                (troot / "tools/pb/plan.py").chmod(0o644)

            def id_moves(idfn):
                before = idfn()
                (troot / "tools/pb/plan.py").write_bytes(os.urandom(8))
                return idfn() != before
            check("the tree id moves with an edit, nothing committed", id_moves(tid))
            real_visible = visible_files
            globals()["visible_files"] = lambda root, man, logs: []            # the id left to HEAD alone
            try:
                plant_("the tree id on HEAD alone: red - an edit does not move it", not id_moves(tid))
            finally:
                globals()["visible_files"] = real_visible

            # -- D2 · the tree without git: the walk against git's own list, then every mode with no repository --
            have_git = shutil.which("git") is not None
            wenv = {k: os.environ.get(k) for k in ("HOME", "XDG_CONFIG_HOME", "GIT_CONFIG_NOSYSTEM")}
            whome = tmp / "whome"
            (whome / "git").mkdir(parents=True)
            (whome / "git" / "ignore").write_text("*.glob\n", encoding="utf-8")
            os.environ.update(HOME=str(whome), XDG_CONFIG_HOME=str(whome), GIT_CONFIG_NOSYSTEM="1")
            wroot = tmp / "walk"
            wfiles = {
                ".gitignore": "# a comment\n*.log\n!keep.log\n/build\ndocs/*.tmp\ncache/\n**/tmpdir\nout/**/gen\nsec[0-9]?.dat\n"
                              "\\#hash\n\\!bang\ntrail\\ \nlogs2/**\n!logs2/keep.txt\nvar/\n!var/inside.txt\nclass[[:digit:]].x\n"
                              "*.o\n!keep.glob\ndata/**/\n",
                "a.txt": "", "a.log": "", "keep.log": "", "build/x.txt": "", "sub/build/y.txt": "", "docs/a.tmp": "",
                "docs/sub/a.tmp": "", "cache/c.txt": "", "sub/cache/c.txt": "", "a/tmpdir/z": "", "tmpdir": "",
                "out/gen/f": "", "out/x/y/gen/f": "", "out/genx": "", "sec1a.dat": "", "sec12.dat": "", "secab.dat": "",
                "#hash": "", "!bang": "", "trail ": "", "logs2/a.txt": "", "logs2/keep.txt": "", "var/other.txt": "",
                "var/inside.txt": "", "class5.x": "", "classa.x": "", "x.o": "", "x.glob": "", "keep.glob": "",
                "data/d/e/f.txt": "", "é.txt": "", "with space.txt": "", "from-exclude.txt": "",
                "sub/.gitignore": "!*.log\n/only-here\n*.bak\n", "sub/a.log": "", "sub/only-here": "",
                "sub/deep/only-here": "", "sub/z.bak": "", "sub/deep/.gitignore": "!x.bak\n/*.cfg\n",
                "sub/deep/x.bak": "", "sub/deep/y.bak": "", "sub/deep/a.cfg": "", "sub/deep/in/a.cfg": "",
                "sub2/.gitignore": "*.crlf\r\n", "sub2/a.crlf": "", "sub2/b.txt": "",
                "nested/n.txt": "", "nested/.git/HEAD": "ref: refs/heads/main\n",
            }
            ignored_by_rule = {
                "a.log", "build/x.txt", "docs/a.tmp", "cache/c.txt", "sub/cache/c.txt", "a/tmpdir/z", "tmpdir", "out/gen/f",
                "out/x/y/gen/f", "sec1a.dat", "sec12.dat", "#hash", "!bang", "trail ", "logs2/a.txt", "var/other.txt",
                "var/inside.txt", "class5.x", "x.o", "x.glob", "data/d/e/f.txt", "sub/only-here", "sub/z.bak",
                "sub/deep/y.bak", "sub/deep/a.cfg", "sub2/a.crlf"}
            in_nested_repo = {"nested/n.txt", "nested/.git/HEAD"}
            (tmp / "probe ").write_bytes(b"")                   # Win32 strips a trailing space: does this file system keep one?
            trail_kept = "probe " in os.listdir(tmp)
            (tmp / ("probe " if trail_kept else "probe")).unlink()
            if not trail_kept:                                  # the case cannot exist here - left out of both sides, not tolerated
                wfiles.pop("trail ")
                ignored_by_rule.discard("trail ")
            for rp, body in wfiles.items():
                (wroot / rp).parent.mkdir(parents=True, exist_ok=True)
                (wroot / rp).write_bytes(body.encode("utf-8"))  # the bytes exact: text mode turns "\r\n" into "\r\r\n" on Windows
            with contextlib.suppress(OSError, NotImplementedError):
                (wroot / "link.txt").symlink_to("a.txt")
                (wroot / "dlink").symlink_to("sub", target_is_directory=True)
            for d in ("objects", "refs"):                      # enough of a repository for git to see one
                (wroot / "nested" / ".git" / d).mkdir()
            hand_off = {"from-exclude.txt"}                    # git's, from .git/info/exclude - only a repository holds one
            want = sorted((set(wfiles) | ({"link.txt"} if (wroot / "link.txt").is_symlink() else set()))
                          - ignored_by_rule - in_nested_repo)

            def gitlist(r):
                out = git_out(r, ["ls-files", "-co", "--exclude-standard", "-z"])
                return None if out is None else sorted(x for x in out.split("\0") if x and os.path.isfile(os.path.join(r, x)))
            check("the walk lists what the .gitignore files leave (comments, !, / anchor, trailing /, * ? [..] **, "
                  "escapes, nested files, a nested repository, CRLF, the global excludes)",
                  walk_tree(wroot) == want, sorted(set(walk_tree(wroot)) ^ set(want)))
            if have_git:
                subprocess.run(["git", "init", "-q"], cwd=str(wroot), check=True, capture_output=True)
                (wroot / ".git" / "info").mkdir(exist_ok=True)
                (wroot / ".git" / "info" / "exclude").write_text("from-exclude.txt\n", encoding="utf-8")
                check("the walk equals `git ls-files -co --exclude-standard` on the same fixture",
                      walk_tree(wroot) == gitlist(wroot) == [x for x in want if x not in hand_off],
                      sorted(set(walk_tree(wroot)) ^ set(gitlist(wroot) or [])))
            real_rules = read_rules
            globals()["read_rules"] = lambda path: []                           # a walk blind to every ignore file
            try:
                plant_("the walk blind to .gitignore: red - it lists the ignored files",
                       walk_tree(wroot) != [x for x in want if x not in hand_off])
            finally:
                globals()["read_rules"] = real_rules
            # a root below its repository's top (a track): the parents' ignore files read, each rule relative to its folder
            ntop = tmp / "ntop"
            nfiles = {
                ".gitignore": "*.log\n__pycache__/\nout/\n!keep.log\n/build\n",
                "build/top.txt": "", "trk/build/x.txt": "", "trk/mid/.gitignore": "*.tmp\n/zed\n",
                "trk/mid/root/a.txt": "", "trk/mid/root/a.log": "", "trk/mid/root/keep.log": "",
                "trk/mid/root/__pycache__/a.pyc": "", "trk/mid/root/build/b.txt": "", "trk/mid/root/b.tmp": "",
                "trk/mid/root/zed": "", "trk/mid/root/sub/zed": "", "trk/mid/root/.gitignore": "*.cfg\n",
                "trk/mid/root/c.cfg": "", "trk/mid/root/out2/d.txt": "",
                "trk/out/inner/e.txt": "", "trk/out/inner/e.log": "",
            }
            for rp, body in nfiles.items():
                (ntop / rp).parent.mkdir(parents=True, exist_ok=True)
                (ntop / rp).write_text(body, encoding="utf-8")
            if have_git:
                subprocess.run(["git", "init", "-q"], cwd=str(ntop), check=True, capture_output=True)
            else:
                (ntop / ".git").mkdir()
            nroot, nout = ntop / "trk" / "mid" / "root", ntop / "trk" / "out" / "inner"
            nwant = [".gitignore", "a.txt", "build/b.txt", "keep.log", "out2/d.txt", "sub/zed", "zed"]
            check("a root below its repository's top lists what the parents' .gitignore files leave "
                  "(*.log, __pycache__/, a re-include, /build anchored at the top, /zed anchored at a middle folder), "
                  "and a root an ignored folder holds lists nothing",
                  walk_tree(nroot) == nwant and walk_tree(nout) == [], (walk_tree(nroot), walk_tree(nout)))
            if have_git:
                check("the nested-root walk equals `git ls-files -co --exclude-standard` run in the root",
                      walk_tree(nroot) == gitlist(nroot) and walk_tree(nout) == gitlist(nout) == [],
                      (walk_tree(nroot), gitlist(nroot), walk_tree(nout), gitlist(nout)))
            real_top = repo_top
            globals()["repo_top"] = lambda r: (os.path.abspath(os.fspath(r)), "")      # the parents' ignore files unread
            try:
                plant_("the parents' ignore files unread: red - the nested root lists what git ignores",
                       walk_tree(nroot) != nwant and walk_tree(nout) != [])
            finally:
                globals()["repo_top"] = real_top
            big = tmp / "big"
            for i in range(60):
                (big / f"d{i}" / "ign").mkdir(parents=True)
                for j in range(50):
                    (big / f"d{i}" / f"f{j}.txt").write_text(str(j), encoding="utf-8")
            (big / ".gitignore").write_text("ign/\n*.tmp\n", encoding="utf-8")
            tw = time.perf_counter()
            nbig = len(walk_tree(big))
            tw = time.perf_counter() - tw
            tg, note = None, ""
            if have_git:
                subprocess.run(["git", "init", "-q"], cwd=str(big), check=True, capture_output=True)
                tg = time.perf_counter()
                gbig = gitlist(big)
                tg = time.perf_counter() - tg
                note = f" (git {tg:.3f}s)"
            check(f"the walk of {nbig} files takes {tw:.3f}s{note}: under 2s and under 50x git",
                  nbig == 3001 and tw < 2 and (tg is None or tw < max(0.5, 50 * tg)), (nbig, tw, tg))
            for k, v in wenv.items():
                if v is None:
                    os.environ.pop(k, None)
                else:
                    os.environ[k] = v

            # every mode in a folder with no repository
            nrep = tmp / "norepo"
            nrep.mkdir()
            (nrep / "emit.py").write_text(EMIT, encoding="utf-8")
            for rp, body in (("data.txt", "ok\n"), ("plant.py", "open('data.txt', 'w').write('BUG')\n"), ("doc.md", "d\n"),
                             (".gitignore", "ign.txt\nlogs/\nanchor.json\n"), ("ign.txt", "ignored\n")):
                (nrep / rp).write_text(body, encoding="utf-8")
            nman = {"logs_dir": "logs", "anchor": "anchor.json", "vars": {"emit": ["emit.py"]},
                    "skip_classes": [{"match": ["*.md", "emit.py", "plant.py", ".gitignore"], "reason": "fixture"}],
                    "scopes": {"chk": {"cmd": f"{py} check data.txt", "paths": ["data.txt"],
                                       "plants": [{"id": "fn", "patch": "{python} -S plant.py"}]}}}
            check("git cannot answer in the folder", git_out(nrep, ["rev-parse", "HEAD"], 30) is None)
            nvis = visible_files(nrep, nman, nrep / "logs")
            check("the anchor's subject in a folder with no repository: the files the .gitignore leaves",
                  nvis == [".gitignore", "data.txt", "doc.md", "emit.py", "plant.py"], nvis)
            n1 = changed_run(nman, nrep, "T9", 1, box="SELFBOX")
            check("--changed with no repository: runs, GO, the anchor seeded",
                  not n1["failures"] and not n1["nogo"] and (nrep / "anchor.json").exists(), n1["failures"] or n1["nogo"])
            (nrep / "data.txt").write_text("ok edited\n", encoding="utf-8")
            n2 = changed_run(nman, nrep, "T9", 1, box="SELFBOX")
            check("--changed with no repository: the edited file chooses its scope",
                  {r["scope"] for r in n2["data"]["rows"]} == {"chk"} and not n2["failures"], n2["failures"] or n2["lines"])
            n3 = run_scopes(nman, ["chk"], nrep, "T9", 1, mode="all",
                            hook=lambda: loop_checks(nman, nrep, "T9", 1, "SELFBOX", nrep / "verify.json"))
            check("--all with no repository: GO", not n3["failures"] and not n3["nogo"], n3["failures"] or n3["nogo"])
            n4 = redarm(nman, "chk", nrep, "T9", 1, box="SELFBOX")
            check("--redarm with no repository: the plant goes red, GO", not n4["failures"] and not n4["nogo"]
                  and n4["data"]["red"] == 1, n4["failures"] or n4["nogo"])
            recs = [json.loads(x) for x in (nrep / "logs" / "loop_times.jsonl").read_text(encoding="utf-8").splitlines() if x.strip()]
            check("the records' tree id is the content hash alone, no HEAD in front",
                  bool(recs) and all(re.fullmatch(r"[0-9a-f]{12}", r.get("tree") or "") for r in recs), [r.get("tree") for r in recs])
            check("refuse_log refuses nothing where no repository answers", refuse_log(nrep, [nrep / "logs" / "T9.chk.log"]) is None)

            # -- D3 · the task's own start: three tasks in a row, no commit, no repository ----------------
            sq = tmp / "seq"
            sq.mkdir()
            (sq / "emit.py").write_text(EMIT, encoding="utf-8")
            (sq / "plant.py").write_text("import sys\nopen(sys.argv[1], 'w').write('BUG')\n", encoding="utf-8")
            (sq / ".gitignore").write_text("*.tmp\n", encoding="utf-8")       # the logs folder is held out by the tool
            sq_man = {"logs_dir": "logs", "anchor": "anchor.json", "vars": {"emit": ["emit.py"]},
                      "skip_classes": [{"match": ["emit.py", "plant.py", ".gitignore", "verify.json"], "reason": "fixture"}],
                      "scopes": {}}
            sq_mf = sq / "verify.json"

            def sq_add(name):                       # a task adds a scope with a plant, and the file it grades
                (sq / f"{name}.py").write_text("ok\n", encoding="utf-8")
                sq_man["scopes"][name] = {"cmd": f"{py} check {name}.py", "paths": [f"{name}.py"],
                                          "plants": [{"id": "p", "patch": f"{{python}} -S plant.py {name}.py"}]}
                sq_mf.write_text(json.dumps(sq_man), encoding="utf-8")

            def sq_start(task, fresh=False, root=sq, mf=sq_mf):
                return cmd_start(load_manifest(mf), argparse.Namespace(task=task, fresh=fresh), root, mf)

            def sq_claim(task, root=sq, mf=sq_mf):
                m = load_manifest(mf)
                lb = loop_base(m, root, task, mf)               # looked up at the call: the plants below swap it
                lines_, fails_ = loop_checks(m, root, task, 1, "SELFBOX", mf, block={"type": "CHECK", "deliver": "", "plan": "p"},
                                             changed=lb["changed"], mchanges=lb["mchanges"], label=lb["label"])
                armed = sorted({x.group(1) for x in (re.match(r"(\w+) · plant ", ln) for ln in lines_) if x})
                flagged = sorted({x.group(1) for x in (re.match(r"FLAG (?:oracle|type): (\S+\.py) changed", ln) for ln in lines_) if x})
                return lb, armed, flagged, fails_
            sq_add("s0")
            r_a = sq_start("TA")
            check("start: one line, the task, the file count, a timestamp",
                  re.fullmatch(r"baseline: TA, \d+ files, 20\d\d-\d\d-\d\dT\S+", r_a["lines"][0]) is not None, r_a["lines"])
            bdir = sq / "logs" / ".baseline"
            check("start: a .gitignore of `*` inside the baselines folder, the logs folder held out of the files",
                  (bdir / ".gitignore").read_text(encoding="utf-8") == "*\n"
                  and not any(k.startswith("logs/") or k == "anchor.json" for k in read_baseline(bdir / "TA.json")["files"]))
            check("start: the manifest's text is kept", json.loads(read_baseline(bdir / "TA.json")["manifest"])["scopes"].keys() == {"s0"})
            sq_add("sa")                                        # task A's work, uncommitted - there is no commit
            lb_a, armed_a, flagged_a, f_a = sq_claim("TA")
            check("a claim after the task's own start: its added scope's plant re-armed, red; the scope it found is not",
                  armed_a == ["sa"] and not f_a and lb_a["line"].startswith("base: TA's start 20"), (armed_a, f_a, lb_a["line"]))
            check("a claim after the task's own start flags the paths it changed", flagged_a == ["sa.py"], flagged_a)
            sq_start("TB")
            sq_add("sb")
            sq_start("TC")
            lb_b, armed_b, flagged_b, _f = sq_claim("TB")
            check("task B's claim reads its own start, though task A left its work uncommitted and TC started after",
                  armed_b == ["sb"] and flagged_b == ["sb.py"], (armed_b, flagged_b))
            sq_add("sc")
            real_loop_base = loop_base
            lb_c, armed_c, flagged_c, _f = sq_claim("TC")
            check("the third task of a row: its claim re-arms its own scope only and flags its own paths only",
                   armed_c == ["sc"] and flagged_c == ["sc.py"], (armed_c, flagged_c))
            globals()["loop_base"] = lambda m, root, task, mf: {                      # the loop back on HEAD: no commit,
                "changed": sorted(walk_tree(root)), "mchanges": {n: "added" for n in m["scopes"]},    # so all of it differs
                "label": "HEAD", "line": "base: HEAD"}
            try:
                _l, armed_h, flagged_h, _f = sq_claim("TC")
                plant_("the loop back on HEAD: red - every uncommitted task's scope re-armed and path flagged",
                       not (armed_h == ["sc"] and flagged_h == ["sc.py"]), (armed_h, flagged_h))
            finally:
                globals()["loop_base"] = real_loop_base
            # the claim's record (M0-T63): whose start, the task's own files, the loop's seconds apart
            real_checks, real_bill, real_touched = loop_checks, loop_bill, touched_of

            def sq_loop(task, slow=0.0):                 # one real --all claim; the loop slowed by `slow` s
                globals()["loop_checks"] = lambda *x, **k: (time.sleep(slow), real_checks(*x, **k))[1]
                try:
                    cmd_run(load_manifest(sq_mf), build_parser().parse_args(["--all", "--task", task, "-j", "2"]), sq, sq_mf)
                finally:
                    globals()["loop_checks"] = real_checks
                return json.loads((sq / "logs" / "loop_times.jsonl").read_text(encoding="utf-8").splitlines()[-1])
            rc_c = sq_loop("TC", slow=1.0)
            check("the claim record names the task's own files against its own start, and whose start",
                  rc_c.get("touched") == {"added": ["sc.py"], "modified": ["verify.json"], "deleted": []}
                  and rc_c.get("base", "").startswith("TC's start 20") and "no start" not in rc_c.get("base", ""),
                  (rc_c.get("touched"), rc_c.get("base")))
            check("the claim bills the loop apart: `seconds` the scopes alone, `loop_seconds` the plants and flags",
                  is_num(rc_c.get("loop_seconds")) and rc_c["loop_seconds"] >= 1.0 and rc_c["seconds"] < 1.0,
                  (rc_c.get("seconds"), rc_c.get("loop_seconds")))
            globals()["loop_bill"] = lambda e, before, h: {"seconds": e + before + h, "loop_seconds": round(before + h, 3)}
            try:
                rc_x = sq_loop("TC", slow=1.0)
            finally:
                globals()["loop_bill"] = real_bill
            plant_("`loop_seconds` folded into `seconds`: red - the scopes' clock bills the loop too",
                   not (rc_x["seconds"] < 1.0 <= rc_x["loop_seconds"]), (rc_x["seconds"], rc_x["loop_seconds"]))
            # the seconds at an idle box (M0-T64d): from the end load, k the default without a fitted load_k
            idle_of = lambda rc, k: tuple(round(float(rc[x]) / (1 + k * rc["load"] / rc["cpus"]), 3)  # noqa: E731
                                          for x in ("seconds", "loop_seconds"))
            check("the claim record: `seconds_norm` and `loop_seconds_norm` from its load, `load_norm` naming the default k",
                  (rc_c.get("seconds_norm"), rc_c.get("loop_seconds_norm")) == idle_of(rc_c, DEFAULT_LOAD_K)
                  and "default 0.15" in rc_c.get("load_norm", "") if is_num(rc_c.get("load"))
                  else rc_c.get("seconds_norm") is None, (rc_c.get("seconds_norm"), rc_c.get("load_norm")))
            real_sample = sample_load

            def blind_loop(task):                    # a box that cannot sample its load
                globals()["sample_load"] = lambda *a, **k: (None, "none: a denied counter")
                try:
                    return sq_loop(task)
                finally:
                    globals()["sample_load"] = real_sample
            nulled = lambda rc: ("seconds_norm" in rc and "loop_seconds_norm" in rc and rc["seconds_norm"] is None  # noqa: E731
                                 and rc["loop_seconds_norm"] is None and rc.get("load_norm") == "none: a denied counter")
            rc_n = blind_loop("TC")
            check("no load sample: `seconds_norm` null, `load_norm` the reason - never the raw copied in", nulled(rc_n),
                  {k: rc_n.get(k) for k in ("seconds", "seconds_norm", "loop_seconds_norm", "load_norm")})
            rc_p = swap_("normalise", lambda rec, man: rec.update(seconds_norm=rec["seconds"],
                                                                  loop_seconds_norm=rec.get("loop_seconds"),
                                                                  load_norm="none: a denied counter") or rec,
                         lambda: blind_loop("TC"))
            plant_("the raw seconds copied into a null `seconds_norm`: red", not nulled(rc_p), rc_p.get("seconds_norm"))
            nk = {"scopes": {}, "budget": {"boxes": {"B32": {"load_k": 0.007, "cpus": 32}}}}
            rn = normalise({"box": "B32", "seconds": 10.0, "load": 16.0, "cpus": 32}, nk)
            check("normalise: the manifest box's fitted load_k wins, in the relative form (0.007 x 32 = k 0.224)",
                  rn["seconds_norm"] == round(10 / 1.112, 3) and "manifest B32 load_k 0.007 x 32 cpus" in rn["load_norm"]
                  and "seconds_norm" in normalise({"box": "B8", "seconds": 1.0, "load": 0.0, "cpus": 8}, nk)
                  and normalise({"box": "B8", "seconds": 1.0, "load": 0.0, "cpus": 8}, nk)["load_norm"].startswith("k 0.15 "),
                  rn)
            check("normalise: a load over the CPUs is computed and named out of range",
                  "OUT OF RANGE" in normalise({"box": "B8", "seconds": 1.0, "load": 9.0, "cpus": 8}, nk)["load_norm"])
            globals()["touched_of"] = lambda now, old: {k: [] for k in TOUCH_KINDS}
            try:
                rc_e = sq_loop("TC")
            finally:
                globals()["touched_of"] = real_touched
            plant_("`touched` empty for a task that wrote files: red", rc_e.get("touched") != rc_c.get("touched"),
                   rc_e.get("touched"))
            _h, tail_d, fields_d = loop_hook(load_manifest(sq_mf), sq, "TD", 1, sq_mf)
            check("a task with no start of its own: the record's base names whose start its list was read against",
                  fields_d["base"].startswith("TC's start 20") and fields_d["base"].endswith("— no start for TD")
                  and tail_d.endswith(fields_d["base"]), fields_d["base"])
            first_bytes = (bdir / "TA.json").read_bytes()
            (sq / "late.py").write_text("late\n", encoding="utf-8")
            r_a2 = sq_start("TA")
            check("a resumed task keeps its first start: the same line, the same bytes, the late file not in it",
                  r_a2["lines"] == r_a["lines"] and (bdir / "TA.json").read_bytes() == first_bytes
                  and "late.py" not in read_baseline(bdir / "TA.json")["files"], r_a2["lines"])
            real_read = read_baseline
            globals()["read_baseline"] = lambda path: None                             # a start that never sees the first
            try:
                sq_start("TA")
            finally:
                globals()["read_baseline"] = real_read
            plant_("a second start overwriting the first: red - the resumed task lost its start",
                   (bdir / "TA.json").read_bytes() != first_bytes)
            (bdir / "TA.json").write_bytes(first_bytes)
            lb_d, armed_d, flagged_d, _f = sq_claim("TD")
            check("a task with no start of its own reads the newest start on the box, named",
                  lb_d["line"].startswith("base: TC's start 20") and lb_d["line"].endswith("— no start for TD")
                  and armed_d == ["sc"] and flagged_d == ["late.py", "sc.py"],
                  (lb_d["line"], armed_d, flagged_d))
            r_f = sq_start("TA", fresh=True)
            check("--fresh writes the start again, the late file now in it",
                  "late.py" in read_baseline(bdir / "TA.json")["files"] and r_f["lines"][0].startswith("baseline: TA, "))
            nb = tmp / "nobase"
            nb.mkdir()
            for f in ("emit.py", "plant.py"):
                (nb / f).write_text((sq / f).read_text(encoding="utf-8"), encoding="utf-8")
            nb_man = dict(sq_man, scopes={n: sc for n, sc in sq_man["scopes"].items() if n in ("sa", "sb")})
            nb_man["scopes"]["bare"] = {"cmd": f"{py} check sa.py", "paths": ["sa.py"]}
            for n in ("sa", "sb"):
                (nb / f"{n}.py").write_text("ok\n", encoding="utf-8")
            (nb / "verify.json").write_text(json.dumps(nb_man), encoding="utf-8")
            lb_n, armed_n, _fl, f_n = sq_claim("TZ", root=nb, mf=nb / "verify.json")
            check("a task with no start and no start on the box: every scope with plants re-armed once, the line says so",
                   armed_n == ["sa", "sb"] and lb_n["line"] == "base: none — every plant" and not f_n, (armed_n, lb_n["line"]))
            globals()["loop_base"] = lambda m, root, task, mf: dict(real_loop_base(m, root, task, mf), mchanges={})
            try:
                _l, armed_x, _fl, _f = sq_claim("TZ", root=nb, mf=nb / "verify.json")
                plant_("the no-start fallback running no plant: red - nothing re-armed", armed_x != ["sa", "sb"])
            finally:
                globals()["loop_base"] = real_loop_base
            # the shortcut: a file whose size and mtime match the newest start keeps its hash, unread
            (sq / "same.py").write_text("one\n", encoding="utf-8")
            sq_start("TE")
            st_ = os.stat(sq / "same.py")
            (sq / "same.py").write_text("two\n", encoding="utf-8")
            os.utime(sq / "same.py", ns=(st_.st_atime_ns, st_.st_mtime_ns))
            sq_start("TF")
            check("start: a file whose size and mtime match the newest start keeps that hash, no re-read; a changed one is read",
                  read_baseline(bdir / "TF.json")["files"]["same.py"][2] == read_baseline(bdir / "TE.json")["files"]["same.py"][2]
                  and read_baseline(bdir / "TF.json")["files"]["sc.py"][1] == read_baseline(bdir / "TE.json")["files"]["sc.py"][1])
            for i in range(BASELINES_KEPT + 2):
                sq_start(f"TX{i}")
            kept_ = sorted(x.name for x in bdir.glob("*.json"))
            check(f"start: the newest {BASELINES_KEPT} baselines kept, the oldest gone",
                  len(kept_) == BASELINES_KEPT and f"TX{BASELINES_KEPT + 1}.json" in kept_ and "TA.json" not in kept_, kept_)
            def sq_cli(*args):
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    rc = main(["--manifest", str(sq_mf), "--root", str(sq), "--no-progress"] + list(args))
                return rc, out.getvalue()
            rc, out = sq_cli("start", "--task", "TG")
            check("the CLI: `start --task` prints the baseline line, the counts, GO",
                  rc == 0 and re.search(r"^baseline: TG, \d+ files, 20", out, re.M) and "=== GO ===" in out, out[-300:])
            rc, out = sq_cli("start")
            check("the CLI: `start` without --task is refused", rc == 1 and "no --task" in out, out[-200:])
            rc, out = sq_cli("sa", "--fresh", "--task", "TG")
            check("the CLI: --fresh goes with `start` only", rc == 1 and "--fresh goes with `start`" in out, out[-200:])

            # -- E · the coverage anchor -----------------------------------------------------------------
            aroot = tmp / "anchor"
            (aroot / "src").mkdir(parents=True)
            for p, b in (("src/a.py", b"A"), ("src/b.py", b"B"), ("doc.md", b"D")):
                (aroot / p).write_bytes(b)
            aman = {"scopes": {"s1": {"cmd": py, "paths": ["src/*.py"]}, "s2": {"cmd": py, "paths": ["src/a.py"]}},
                    "skip_classes": [{"match": ["*.md"], "reason": "prose"}], "logs_dir": "logs", "anchor": "anchor.json"}
            apath, alist = aroot / "anchor.json", ["src/a.py", "src/b.py", "doc.md"]
            arec = {"tree": "t1", "task": "selftest", "ts": stamp(), "box": "SELFBOX"}
            gate = lambda paths=alist, man=aman: anchor_status(man, aroot, anchor_read(apath), paths, "SELFBOX")  # noqa: E731
            hashes = lambda *ps: {p: sha1_of(aroot, p) for p in ps}  # noqa: E731
            check("a glob stays inside a segment, `**` crosses",
                  bool(globs(["src/*.py"]).match("src/a.py")) and not globs(["src/*.py"]).match("src/x/a.py")
                  and bool(globs(["src/**/*.py"]).match("src/x/a.py")) and bool(globs(["m/*/l/**"]).match("m/i/l/x/y.log")))
            check("coverage puts every path in exactly one class",
                  coverage(aman, alist + ["loose.bin"]) == ({"src/a.py": ["s1", "s2"], "src/b.py": ["s1"]},
                                                            {"doc.md": "prose"}, ["loose.bin"]))
            g0 = gate()
            c0, l0 = anchor_lines(g0)
            check("the gate names every never-checked path, and prints its arithmetic",
                  sum(x.startswith("NEVER CHECKED") for x in l0) == 2 and "0 paths checked" in c0 and "1 skipped" in c0
                  and any(x.startswith("skipped") and "prose" in x for x in l0), (c0, l0))
            moved = anchor_advance(aman, aroot, {"s1", "s2"}, set(), hashes("src/a.py", "src/b.py"), arec, apath, alist)
            g1 = gate()
            check("the anchor advances on green", moved == 3 and anchor_nogo(g1) is None and len(g1["checked"]) == 2
                  and json.loads(apath.read_text(encoding="utf-8"))["paths"]["src/a.py"]["s2"]["tree"] == "t1", (moved, g1))
            again = apath.read_bytes()
            check("a no-op advance writes nothing",
                  anchor_advance(aman, aroot, {"s1", "s2"}, set(), hashes("src/a.py", "src/b.py"), arec, apath, alist) == 0
                  and apath.read_bytes() == again)
            (aroot / "src/a.py").write_bytes(b"A edited by nobody's scope")
            check("a red scope never advances and loses its entries",
                  anchor_advance(aman, aroot, set(), {"s2"}, hashes("src/a.py"), arec, apath, alist) == 1
                  and "s2" not in json.loads(apath.read_text(encoding="utf-8"))["paths"]["src/a.py"]
                  and json.loads(apath.read_text(encoding="utf-8"))["paths"]["src/a.py"]["s1"]["sha1"] != hashes("src/a.py")["src/a.py"])
            anchor_advance(aman, aroot, {"s1"}, set(), hashes("src/a.py"), arec, apath, alist)
            g3 = gate()
            check("one unchecked scope un-checks the path", len(g3["never"]) == 1 and g3["never"][0][0] == "src/a.py"
                  and g3["never"][0][1] == ["s2"] and len(g3["checked"]) == 1, g3["never"])
            anchor_advance(aman, aroot, {"s1", "s2"}, set(), hashes("src/a.py", "src/b.py"), arec, apath, alist)
            sman = json.loads(json.dumps(aman))
            sman["scopes"]["s2"]["cmd"] = py + " other"
            g_spec = gate(man=sman)
            g_loose = gate(alist + ["loose.bin"])
            g_zero = anchor_status({"scopes": {"s1": {"cmd": py, "paths": ["nothing/*"]}}, "anchor": "anchor.json",
                                    "skip_classes": [{"match": ["**"], "reason": "all of it"}]}, aroot,
                                   {"paths": {}}, alist, "SELFBOX")
            (aroot / "src/b.py").write_bytes(b"B edited")
            g_stale = gate()
            oman = json.loads(json.dumps(aman))
            oman["scopes"]["s1"]["box"] = "OTHERBOX"
            g_owed = gate(man=oman)
            for name, st, want, drives in (("a scope's definition moved: its paths read stale", g_spec, "stale", "stale"),
                                           ("an unclassified path", g_loose, "unclassified", "loose"),
                                           ("a path whose bytes moved", g_stale, "stale", "stale"),
                                           ("zero paths checked is no pass", g_zero, "zero paths checked", None)):
                others = [k for k in ("never", "stale", "loose") if k != drives and st[k]]
                nogo = anchor_nogo(st) or ""
                plant_(name, want in nogo and not others and (drives is None or bool(st[drives])), (nogo, others))
            check("a path only another box's scope owes reads owed, never NO-GO",
                  g_owed["owed"] == ["src/b.py"] and anchor_nogo(g_owed) is None, (g_owed["owed"], anchor_nogo(g_owed)))
            oman["scopes"]["s1"]["box"] = ["OTHERBOX", "selfbox "]
            g_listed = gate(man=oman)
            check("a `box` list runs on each box it names and owes on every other (M0-D111)",
                  not g_listed["owed"] and not owed_on(oman["scopes"]["s1"], "SELFBOX")
                  and owed_on({"box": ["A", "B"]}, "SELFBOX") and owed_label({"box": ["A", "B"]}) == "A or B"
                  and not scope_errors("s", {"cmd": py, "box": ["A", "B"]})
                  and all(scope_errors("s", {"cmd": py, "box": b}) for b in ([], ["A", ""], ["A", 3], "", 7)),
                  (g_listed["owed"], owed_on(oman["scopes"]["s1"], "SELFBOX")))
            apath.write_text("{ not json", encoding="utf-8")
            try:
                gate()
                plant_("an unreadable anchor: refused", False, "read as empty")
            except Refused as exc:
                plant_("an unreadable anchor: refused", "not an anchor" in str(exc), str(exc))
            apath.write_text(json.dumps({"version": 2, "note": "", "paths": {"src/a.py": {"s1": {"sha1": "x"}}}}), encoding="utf-8")
            sup = anchor_read(apath)
            plant_("an anchor at another version is dropped whole", sup["paths"] == {} and sup["superseded"] == 2
                   and "never checked" in (anchor_nogo(gate()) or ""))
            apath.unlink()
            check("a missing anchor reads empty and names everything", anchor_read(apath)["paths"] == {} and bool(anchor_nogo(gate())))

            # -- F · --changed: chosen from the anchor, then the gate ------------------------------------
            croot = tmp / "changed"
            (croot / "src").mkdir(parents=True)
            (croot / "emit.py").write_text(EMIT, encoding="utf-8")
            for p, b in (("src/a.py", "A"), ("src/b.py", "B"), ("doc.md", "D"), ("t.py", "T")):
                (croot / p).write_text(b, encoding="utf-8")
            cman = {"logs_dir": "logs", "anchor": "anchor.json", "vars": {"emit": ["emit.py"]},
                    "skip_classes": [{"match": ["*.md", "emit.py"], "reason": "prose and the fixture"}],
                    "scopes": {"sa": {"cmd": f"{py} go", "paths": ["src/a.py"]},
                               "sb": {"cmd": f"{py} go", "paths": ["src/b.py"]},
                               "free": {"cmd": f"{py} go"},
                               "tst": {"cmd": f"{py} go", "paths": ["t.py"]}}}
            cvis = ["src/a.py", "src/b.py", "doc.md", "t.py", "emit.py"]
            cgit = lambda moved="": (lambda r, args, timeout=120: "f" * 40 + "\n" if args[0] == "rev-parse"  # noqa: E731
                                     else moved if args[0] == "diff" else "")
            ch = lambda man=cman, base=None, git=None, vis=cvis: changed_run(man, croot, "T9", 2, base=base, box="SELFBOX",  # noqa: E731
                                                                             git=git or cgit(), visible=vis)
            r1 = ch()
            check("--changed on an empty anchor runs every never-checked scope and seeds the anchor, GO",
                  not r1["failures"] and not r1["nogo"] and {r["scope"] for r in r1["data"]["rows"]} == {"sa", "sb", "free", "tst"}
                  and (croot / "anchor.json").exists(), r1["failures"] or r1["nogo"])
            (croot / "src/a.py").write_text("A edited", encoding="utf-8")
            r2 = ch()
            ran2 = {r["scope"] for r in r2["data"]["rows"]}
            check("--changed then runs the scope a changed path touches, and the scope with no paths; the rest NOT RUN "
                  "with their ground", ran2 == {"sa", "free"} and not r2["failures"]
                  and any(ln.startswith("sb") and "NOT RUN (not impacted: its 1 path(s) checked" in ln for ln in r2["lines"])
                  and any(ln.startswith("why sa:") and "stale: src/a.py" in ln for ln in r2["lines"]), r2["lines"])
            r3 = ch(base="v1.0", git=cgit("src/b.py\0"))
            check("--base: a path moved since the named commit chooses its scope too",
                  {r["scope"] for r in r3["data"]["rows"]} == {"sb", "free"}
                  and any("moved since v1.0: src/b.py" in ln for ln in r3["lines"]), r3["lines"])
            tg = touched_git(croot, "v1.0", git=lambda r, args, timeout=120: "M\0src/a.py\0D\0gone.py\0A\0new.py\0"
                             if args[0] == "diff" else "u.py\0")
            check("--base: the record's touched read from git, untracked files as added",
                  tg == {"added": ["new.py", "u.py"], "modified": ["src/a.py"], "deleted": ["gone.py"]}, tg)
            for name, base in (("HEAD", "HEAD"), ("HEAD~1", "HEAD~1"), ("@", "@")):
                try:
                    ch(base=base)
                    plant_(f"--base {name}: refused", False, "accepted")
                except Refused as exc:
                    plant_(f"--base {name}: refused", "never HEAD" in str(exc), str(exc))
            (croot / "new.bin").write_text("x", encoding="utf-8")
            r4 = ch(vis=cvis + ["new.bin"])
            plant_("--changed: a path no scope covers and no class names is NO-GO, named",
                   any("unclassified" in f for f in r4["failures"]) and any(ln.startswith("UNCLASSIFIED   new.bin") for ln in r4["lines"]),
                   r4["failures"])
            rman = json.loads(json.dumps(cman))
            rman["scopes"]["tst"]["cmd"] = f"{py} check t.py"
            (croot / "t.py").write_text("BUG", encoding="utf-8")
            r5 = ch(man=rman)
            ent = json.loads((croot / "anchor.json").read_text(encoding="utf-8"))["paths"].get("t.py", {})
            plant_("--changed: a chosen scope that goes red is NO-GO and loses its entries",
                   any(f.startswith("tst:") for f in r5["failures"]) and "tst" not in ent
                   and any("never checked" in f for f in r5["failures"]), r5["failures"])
            (croot / "t.py").write_text("T", encoding="utf-8")
            r6 = ch(man=rman)
            check("the next --changed chooses it again and goes GO",
                  "tst" in {r["scope"] for r in r6["data"]["rows"]} and not r6["failures"], r6["failures"])
            gman = json.loads(json.dumps(rman))
            gman["scopes"]["gpu"] = {"cmd": f"{py} go", "paths": ["src/a.py"], "lane": "gpu"}
            rg = ch(man=gman)
            plant_("--changed: a NOT RUN scope is named, and the gate reads NO-GO on it - it cannot vouch for a scope it "
                   "cannot run", any(f.startswith("gpu: NOT RUN (unknown key `lane`)") for f in rg["failures"]), rg["failures"])
            nman = json.loads(json.dumps(rman))
            del nman["scopes"]["free"]
            r7 = ch(man=nman)
            plant_("--changed with nothing changed: NO-GO, zero cases - the anchor already vouches",
                   bool(r7["nogo"]) and "nothing changed" in r7["nogo"], r7["nogo"])
            try:
                ch(man={**cman, "scopes": {"x": {"cmd": py}}})
                plant_("--changed with no scope declaring paths: refused", False, "ran")
            except Refused as exc:
                plant_("--changed with no scope declaring paths: refused", "no scope declares `paths`" in str(exc), str(exc))

            # -- F2 · a scope whose cases are files: under --changed, only the cases whose file moved (M0-T64b) --
            kroot = tmp / "casefiles"
            (kroot / "d").mkdir(parents=True)
            (kroot / "x").mkdir()
            (kroot / "emit.py").write_text(EMIT, encoding="utf-8")
            (kroot / "deny.txt").write_text("BAD\n", encoding="utf-8")
            (kroot / "x/conf.txt").write_text("1\n", encoding="utf-8")
            for f in ("a", "b", "c"):
                (kroot / f"d/{f}.txt").write_text(f"{f}\n", encoding="utf-8")
            kman = {"logs_dir": "logs", "anchor": "anchor.json", "vars": {"emit": ["emit.py"]}, "tree_extra": ["x"],
                    "skip_classes": [{"match": ["emit.py", "deny.txt", "x/**"], "reason": "the fixture"}],
                    "scopes": {"docs": {"cases_cmd": f"{py} ls d", "case_cmd": f"{py} deny deny.txt {{case}}",
                                        "expected": 3, "paths": ["d/**"]}}}

            def kch(m=None, task="K"):
                trace.write_text("", encoding="utf-8")
                r = changed_run(m or kman, kroot, task, 2, box="SELFBOX")
                return r, sorted(s[0] for s in spans() if s[0].startswith("d/"))

            def kline(r):
                return next((ln for ln in r["lines"] if ln.startswith("docs cases: ")), "")

            def kmut(fn_name, mutant, call):              # a mutant's run, the anchor put back as it found it
                saved = (kroot / "anchor.json").read_bytes()
                try:
                    return swap_(fn_name, mutant, call)
                finally:
                    (kroot / "anchor.json").write_bytes(saved)

            def kbook():
                return json.loads((kroot / "anchor.json").read_text(encoding="utf-8")).get("cases", {}).get("docs", {})
            r, ran = kch()
            check("cases that are files: the first --changed runs them all and books each in the anchor, GO",
                  not r["failures"] and not r["nogo"] and ran == ["d/a.txt", "d/b.txt", "d/c.txt"]
                  and sorted(kbook()) == ["d/a.txt", "d/b.txt", "d/c.txt"] and all(e["n"] == 1 for e in kbook().values())
                  and kline(r) == "docs cases: 3 of 3 (none vouched by the anchor yet)", (r["failures"], ran, kline(r)))
            (kroot / "d/b.txt").write_text("b2\n", encoding="utf-8")
            r, ran = kch()
            check("... then only the case whose file moved, `cases: 1 of 3 (the rest vouched by the anchor)`, GO - "
                  "`expected` 3 read against the 3 listed, never the 1 run",
                  not r["failures"] and ran == ["d/b.txt"] and kline(r) == "docs cases: 1 of 3 (the rest vouched by the anchor)"
                  and r["data"]["record"]["scopes"]["docs"].get("vouched") == 2, (r["failures"], ran, kline(r)))
            (kroot / "d/n.txt").write_text("n\n", encoding="utf-8")
            r, ran = kch()
            check("... a new case file runs alone", not r["failures"] and ran == ["d/n.txt"], (r["failures"], ran))
            (kroot / "d/c.txt").write_text("BAD c\n", encoding="utf-8")
            r, ran = kch()
            check("... a moved case that goes red: NO-GO, and it loses its entry", ran == ["d/c.txt"]
                  and any(f.startswith("docs:") for f in r["failures"]) and "d/c.txt" not in kbook(), (r["failures"], ran))
            (kroot / "d/c.txt").write_text("c\n", encoding="utf-8")
            r, ran = kch()
            check("... the scope red, its path entries gone: the red case alone runs again - the others' case "
                  "entries still vouch - GO", not r["failures"] and ran == ["d/c.txt"]
                  and kline(r) == "docs cases: 1 of 4 (the rest vouched by the anchor)", (r["failures"], ran, kline(r)))
            (kroot / "deny.txt").write_text("BAD\nb2\n", encoding="utf-8")             # the list every case reads
            rm, mran = kmut("suite_chosen", lambda _m, _r, _l, _b, *cwn: cwn, lambda: kch(task="KM"))
            r, ran = kch()
            plant_("a list every case reads moved and no case file did, the scope left out: red - the real run chooses "
                   "it, runs every case and reddens the one the new entry names",
                   mran == [] and "nothing changed" in (rm["nogo"] or "") and len(ran) == 4
                   and any(f.startswith("docs:") for f in r["failures"]), (rm["nogo"], ran))
            check("the list every case reads moved, no case file did: the scope chosen, every case run, the old file "
                  "the new entry names red", len(ran) == 4 and any(f.startswith("docs:") for f in r["failures"])
                  and "file its case commands name" in kline(r) + " ".join(r["lines"]), (r["failures"], ran, kline(r)))
            (kroot / "deny.txt").write_text("BAD\n", encoding="utf-8")
            r, ran = kch()
            check("... put back: every case again, GO", not r["failures"] and len(ran) == 4, (r["failures"], ran))
            (kroot / "x/conf.txt").write_text("2\n", encoding="utf-8")
            r, ran = kch()
            check("a file under `tree_extra` moved: every case", not r["failures"] and len(ran) == 4
                  and "all: its manifest entry" in kline(r), (r["failures"], ran, kline(r)))
            kman2 = json.loads(json.dumps(kman))
            kman2["scopes"]["docs"]["timeout_s"] = 30
            r, ran = kch(kman2)
            check("the scope's manifest entry moved: every case", not r["failures"] and len(ran) == 4
                  and "all: its manifest entry" in kline(r), (r["failures"], ran, kline(r)))
            r, ran = kch(kman2)
            plant_("nothing moved: NO-GO, zero cases - the anchor vouches for the scope and its cases",
                   bool(r["nogo"]) and ran == [], (r["nogo"], ran))
            n_ok = lambda r: not r["failures"] and not r["nogo"]                            # noqa: E731
            (kroot / "d/a.txt").write_text("a2\n", encoding="utf-8")
            rm, mran = kmut("case_pick", lambda us, *a: ([], "cases: 0 of 4 (MUTANT)", 0), lambda: kch(kman2, "KM"))
            plant_("a pick that runs no case: red - zero cases is NO-GO, never a vouched GO",
                   not n_ok(rm) and mran == [] and any("zero passed" in f for f in rm["failures"]), rm["failures"])
            r, ran = kch(kman2)
            check("... the real pick runs the one moved case", n_ok(r) and ran == ["d/a.txt"], (r["failures"], ran))
            (kroot / "d/a.txt").write_text("a3\n", encoding="utf-8")
            (kroot / "d/z.txt").write_text("BAD z\n", encoding="utf-8")                 # never GO, and red
            real_v = case_vouched
            rm, mran = kmut("case_vouched", lambda e, now, s: e is None or real_v(e, now, s), lambda: kch(kman2, "KM"))
            r, ran = kch(kman2)
            plant_("a pick that skips a never-GO case: red - the case the anchor never saw go GO runs, and reddens",
                   n_ok(rm) and mran == ["d/a.txt"] and not n_ok(r) and ran == ["d/a.txt", "d/z.txt"], (mran, ran))
            (kroot / "d/z.txt").unlink()
            r, ran = kch(kman2)
            check("... the red case's file deleted: the scope chosen, every case, GO", n_ok(r) and len(ran) == 4, (r["failures"], ran))
            (kroot / "d/n.txt").unlink()
            r, ran = kch(kman2)
            check("a case file deleted from a GO tree: a covered path gone, no case - every case",
                  n_ok(r) and ran == ["d/a.txt", "d/b.txt", "d/c.txt"] and "d/n.txt moved and is no case" in kline(r),
                  (r["failures"], ran, kline(r)))
            kman3 = json.loads(json.dumps(kman2))
            kman3["scopes"]["docs"]["case_cmd"] = f"{py} deny d/c.txt {{case}}"      # the entry: now `c` is denied

            def entry_moved():                            # seeded on the mutant's suite, one case file moved, the entry too
                kch(kman2, "KM")
                (kroot / "d/b.txt").write_text("b3\n", encoding="utf-8")
                return kch(kman3, "KM")
            sm, smran = kmut("suite_hash", lambda *_a: "MUTANT", entry_moved)
            r, ran = kch(kman3)
            plant_("a pick that skips every case on an entry change: red - the changed entry runs all, and its new "
                   "rule reddens a case whose file did not move", smran == ["d/b.txt"] and n_ok(sm)
                   and len(ran) == 3 and any(f.startswith("docs:") for f in r["failures"]), (smran, ran, r["failures"]))
            ids = [("l", "go", "x")]
            check("case_pick: a case id that is no file - every case, named",
                  case_pick(ids, None, {}, "S", {}, [])[1] == "cases: 1 of 1 (all: a case id is no file of the tree)"
                  and case_paths({}, ["d/a.txt", "nope"], ["d/a.txt"]) is None
                  and case_paths({"cwd": "d"}, ["a.txt"], ["d/a.txt"]) == {"a.txt": "d/a.txt"})
            two = [("l", "go", "p"), ("l", "go", "q")]
            kept = {"p": {"sha1": "h", "suite": "S", "n": 1}, "q": {"sha1": "h", "suite": "S", "n": 1},
                    "gone": {"sha1": "h", "suite": "S", "n": 9}}
            check("case_pick: a moved covered path that is no case - every case; every case vouched yet chosen - "
                  "every case; the vouched count sums the listed only",
                  "all: t.cfg moved and is no case" in case_pick(two, {"p": "p", "q": "q"}, {"p": "h", "q": "h"}, "S", kept, ["t.cfg"])[1]
                  and "all: the anchor vouches for every case" in case_pick(two, {"p": "p", "q": "q"}, {"p": "h", "q": "h"}, "S", kept, [])[1]
                  and case_pick(two, {"p": "p", "q": "q"}, {"p": "h", "q": "h2"}, "S", kept, ["q"])[1:] == (
                      "cases: 1 of 2 (the rest vouched by the anchor)", 1))
            check("named_files: inside `bash -c`, after `=`, a var's candidate - files only, never a folder or {case}",
                  named_files({"vars": {"emit": ["emit.py"]}}, kroot, {
                      "cases_cmd": "bash -c \"python3 {emit} ls d --deny=deny.txt\"", "case_cmd": "x {case} x/conf.txt"})
                  == {"emit.py", "deny.txt", "x/conf.txt"})
            bad = kroot / "bad_anchor.json"
            bad.write_text(json.dumps({"version": ANCHOR_VERSION, "paths": {}, "cases": {"docs": []}}), encoding="utf-8")
            try:
                anchor_read(bad)
                plant_("an anchor whose `cases` is no object: refused", False, "read")
            except Refused as exc:
                plant_("an anchor whose `cases` is no object: refused", "`cases`" in str(exc), str(exc))

            # -- F2b · a case that is a file the runner holds out of the tree: booked by its hash like any (M0-D42) --
            hroot = tmp / "heldcases"
            (hroot / "d").mkdir(parents=True)
            (hroot / "logs").mkdir()
            (hroot / "emit.py").write_text(EMIT, encoding="utf-8")
            (hroot / "deny.txt").write_text("BAD\n", encoding="utf-8")
            for f_, v_ in (("d/a.txt", "a"), ("d/b.txt", "b"), ("logs/g.log", "g")):
                (hroot / f_).write_text(v_ + "\n", encoding="utf-8")
            hman = {"logs_dir": "logs", "anchor": "anchor.json", "vars": {"emit": ["emit.py"]},
                    "skip_classes": [{"match": ["emit.py", "deny.txt"], "reason": "the fixture"}],
                    "scopes": {"docs": {"cases_cmd": f"{py} ls d logs/g*.log",
                                        "case_cmd": f"{py} deny deny.txt {{case}}", "expected": 3, "paths": ["d/**"]}}}

            def hch(m=None, task="H"):
                trace.write_text("", encoding="utf-8")
                r = changed_run(m or hman, hroot, task, 2, box="SELFBOX")
                return r, sorted(s[0] for s in spans() if s[0].endswith((".txt", ".log")))

            def hline(r):
                return next((ln for ln in r["lines"] if ln.startswith("docs cases: ")), "")
            r, ran = hch()
            check("a case in the logs folder: the first --changed runs all three and books the log's hash like the rest, GO",
                  not r["failures"] and not r["nogo"] and ran == ["d/a.txt", "d/b.txt", "logs/g.log"]
                  and hline(r) == "docs cases: 3 of 3 (none vouched by the anchor yet)", (r["failures"], ran, hline(r)))
            (hroot / "d/a.txt").write_text("a2\n", encoding="utf-8")
            r, ran = hch()
            check("... a doc moved: only that case, `cases: 1 of 3 (the rest vouched by the anchor)` - the log is vouched, "
                  "not re-run", not r["failures"] and ran == ["d/a.txt"]
                  and hline(r) == "docs cases: 1 of 3 (the rest vouched by the anchor)", (r["failures"], ran, hline(r)))
            (hroot / "logs/g.log").write_text("g2\n", encoding="utf-8")
            (hroot / "d/b.txt").write_text("b2\n", encoding="utf-8")
            r, ran = hch()
            check("... the log moved with a doc: both run, the other vouched", not r["failures"]
                  and ran == ["d/b.txt", "logs/g.log"] and hline(r) == "docs cases: 2 of 3 (the rest vouched by the anchor)",
                  (r["failures"], ran, hline(r)))
            (hroot / "d/a.txt").write_text("a3\n", encoding="utf-8")
            real_cp = case_paths
            hm, hmran = swap_("case_paths", lambda s, ids, vis, held=None, root=None: real_cp(s, ids, vis),
                              lambda: hch(task="HM"))
            plant_("a held-out id read as no file: red - the pick runs every case again (`all: a case id is no file of the tree`)",
                   hmran == ["d/a.txt", "d/b.txt", "logs/g.log"] and "all: a case id is no file" in hline(hm), (hmran, hline(hm)))

            # -- F3 · a scope already GO on this tree, in this task, on this box, is not re-run (M0-T64c) --------
            zroot = tmp / "greens"
            for d_ in ("d", "e"):
                (zroot / d_).mkdir(parents=True)
            (zroot / "emit.py").write_text(EMIT, encoding="utf-8")
            (zroot / "deny.txt").write_text("BAD\n", encoding="utf-8")
            for f_, v_ in (("d/a.txt", "a"), ("d/b.txt", "b"), ("e/x.txt", "x")):
                (zroot / f_).write_text(v_ + "\n", encoding="utf-8")
            zman = {"logs_dir": "logs", "anchor": "anchor.json", "vars": {"emit": ["emit.py"]},
                    "skip_classes": [{"match": ["emit.py", "deny.txt"], "reason": "the fixture"}],
                    "scopes": {"zg": {"cmd": f"{py} deny deny.txt e/x.txt", "paths": ["e/**"]},
                               "zr": {"cmd": f"{py} trace zr 0.0", "rerun": True},
                               "zt": {"cmd": f"{py} trace zt 0.0"},
                               "zf": {"cases_cmd": f"{py} ls d", "case_cmd": f"{py} deny deny.txt {{case}}",
                                      "expected": 2, "paths": ["d/**"]}}}

            def zrun(*args, task="Z", box="SELFBOX"):
                trace.write_text("", encoding="utf-8")
                (tmp / "greens.json").write_text(json.dumps(zman), encoding="utf-8")
                os.environ["PB_BOX"] = box
                out = io.StringIO()
                try:
                    with contextlib.redirect_stdout(out):
                        rc = main(["--manifest", str(tmp / "greens.json"), "--root", str(zroot), "--no-progress",
                                   "--task", task] + list(args))
                finally:
                    os.environ["PB_BOX"] = "SELFBOX"
                return rc, out.getvalue(), [s[0] for s in spans() if s[0] != "rows"]

            def zlast():
                return records_in(zroot / "logs" / "loop_times.jsonl")[1][-1]
            rc, out, ran = zrun("zg")
            first_ts = zlast()["ts"]
            check("not re-run: a scope's first run runs it, GO", rc == 0 and ran == ["e/x.txt"]
                  and "not re-run" not in out, (rc, ran, out[-300:]))
            rc, out, ran = zrun("zg")
            rec = zlast()
            check("... the same scope again, same task, same tree: nothing runs, `<scope>  GO (already GO on this tree at "
                  "<ts> — not re-run)`, GO, the counts say so",
                  rc == 0 and ran == [] and out.rstrip().endswith("=== GO ===") and "1 not re-run" in out
                  and re.search(r"^zg +GO \(already GO on this tree at " + re.escape(first_ts) + " — not re-run\\)$", out, re.M)
                  and "1/1 scopes GO" in out, (rc, ran, out[-500:]))
            check("... its record names the scope it skipped (`not_rerun`: the run it stands on) and holds it in no `scopes`",
                  rec["not_rerun"] == {"zg": first_ts} and rec["scopes"] == {} and rec["verdict"] == "GO", rec)
            rc, out, ran = zrun("zg")
            check("... and again: it still stands on the first run (a skip is no run to stand on)",
                  ran == [] and f"at {first_ts} " in out, out[-300:])
            rc, out, ran = zrun("zg", "zt")
            check("... a run of this scope and a new one: the new one runs, this one does not",
                  rc == 0 and ran == ["zt"] and "zg  " in out and "already GO" in out, (ran, out[-400:]))
            rc, out, ran = zrun("zr")
            rc2, out2, ran2 = zrun("zr")
            check("`rerun: true`: a scope that reads what the tree id does not hold always runs",
                  ran == ["zr"] and ran2 == ["zr"] and "already GO" not in out2, (ran, ran2))
            rc, out, ran = zrun("zg", "--force")
            check("`--force` runs the scope again, and its record holds it", rc == 0 and ran == ["e/x.txt"]
                  and "zg" in zlast()["scopes"] and "not_rerun" not in zlast(), (ran, zlast()))
            (zroot / "e/x.txt").write_text("y\n", encoding="utf-8")
            rc, out, ran = zrun("zg")
            check("the tree moved: the scope runs", ran == ["e/x.txt"], ran)
            (zroot / "e/x.txt").write_text("BAD\n", encoding="utf-8")
            rc, out, ran = zrun("zg")
            rc2, out2, ran2 = zrun("zg")
            check("a red always re-runs: red twice on one tree, both runs run it",
                  rc == 1 and rc2 == 1 and ran == ["e/x.txt"] and ran2 == ["e/x.txt"] and "already GO" not in out2, (rc, rc2, ran2))
            (zroot / "e/x.txt").write_text("y\n", encoding="utf-8")
            rc, out, ran = zrun("zg")
            rc2, out2, ran2 = zrun("zg")
            check("... a GO that follows it is GO again, and stands", rc == 0 and ran == ["e/x.txt"] and ran2 == [], (ran, ran2))
            rc, out, ran = zrun("zg", task="Z2")
            check("another task's GO is not this task's: the scope runs", ran == ["e/x.txt"] and "already GO" not in out, ran)
            rc, out, ran = zrun("zg", box="OTHERBOX")
            check("another box's GO is not this box's: the scope runs", ran == ["e/x.txt"] and "already GO" not in out, ran)
            rc, out, ran = zrun("--all", "-j", "1")
            rc2, out2, ran2 = zrun("--all", "-j", "1")
            check("`--all` always runs, on the tree a scoped run was GO on, and again on the same tree",
                  "e/x.txt" in ran and "e/x.txt" in ran2 and "not re-run" not in out + out2, (ran, ran2))
            rc, out, ran = zrun("zf", "--case", "d/a.txt", task="ZC")
            rc2, out2, ran2 = zrun("zf", "--case", "d/a.txt", task="ZC")
            rc3, out3, ran3 = zrun("zf", "--case", "d/b.txt", task="ZC")
            check("`--case`: the same case again is not re-run, another case runs (a case run says nothing of the others)",
                  ran == ["d/a.txt"] and ran2 == [] and "already GO" in out2 and ran3 == ["d/b.txt"], (ran, ran2, ran3))
            rc, out, ran = zrun("zf", task="ZC")
            rc2, out2, ran2 = zrun("zf", "--case", "d/a.txt", task="ZC")
            check("... a run of the whole scope is not covered by a case run, and covers every case after it",
                  sorted(ran) == ["d/a.txt", "d/b.txt"] and ran2 == [] and "already GO" in out2, (ran, ran2))
            rc, out, ran = zrun("zf", "--case", "d/a.txt", "--force", task="ZC")
            check("... `--force` with `--case` runs it", ran == ["d/a.txt"], ran)
            out_ = io.StringIO()
            with contextlib.redirect_stdout(out_):
                rcf = main(["--all", "--force", "--task", "Z"])
            check("`--force` with `--all` is no call: the loop always runs", rcf == 1 and "--force goes with a scope run" in out_.getvalue(),
                  out_.getvalue()[-300:])
            (zroot / "e/x.txt").write_text("x\n", encoding="utf-8")
            zrun("zg", task="ZM")
            (zroot / "anchor.json").unlink(missing_ok=True)                  # the anchor forgot: --changed chooses it
            rc, out, ran = zrun("--changed", task="ZM")
            check("`--changed` always runs: the scope was GO on this tree in this task, and still runs",
                  "e/x.txt" in ran and "not re-run" not in out, (ran, out[-300:]))

            real_gc = green_cover

            def forcing(real):                                              # a runner that reuses in every call
                return lambda *a, **k: real(*a, **dict(k, reuse=True))

            def nocase(recs, *a, **k):                                      # a cover that reads a case run as a full run
                return real_gc([dict(r, mode="scope") if r.get("mode") == "case" else r for r in recs], *a, **k)

            def anytask(recs, task, *a, **k):                               # a cover that reads every task's records
                return real_gc([dict(r, task=task) for r in recs], task, *a, **k)

            def reddish(recs, *a, **k):                                     # a cover that reads a red as GO
                return real_gc([dict(r, scopes={n: dict(e, verdict="GO") if isinstance(e, dict) else e
                                                    for n, e in r["scopes"].items()}) for r in recs], *a, **k)

            def norerun(real):                                              # a runner that does not read `rerun`
                return lambda man, *a, **k: real(dict(man, scopes={n: {k2: v for k2, v in s.items() if k2 != "rerun"}
                                                                   for n, s in man["scopes"].items()}), *a, **k)
            real_rs = run_scopes
            (zroot / "e/x.txt").write_text("BAD\n", encoding="utf-8")
            zrun("zg", task="ZP1")
            rc, out, ran = swap_("green_cover", reddish, lambda: zrun("zg", task="ZP1"))
            plant_("not re-run: a cover that reads a red as GO skips the red (a red always re-runs)", ran == [], (rc, ran))
            (zroot / "e/x.txt").write_text("x\n", encoding="utf-8")
            zrun("zg", task="ZP2")
            (zroot / "anchor.json").unlink(missing_ok=True)
            rc, out, ran = swap_("run_scopes", forcing(real_rs), lambda: zrun("--changed", task="ZP2"))
            plant_("not re-run: a runner that reuses under `--changed` skips a scope the claim must run", "e/x.txt" not in ran
                   and "not re-run" in out, (ran, out[-300:]))
            zrun("zr", task="ZP3")
            rc, out, ran = swap_("run_scopes", norerun(real_rs), lambda: zrun("zr", task="ZP3"))
            plant_("not re-run: a runner that ignores `rerun` skips the scope that must always run", ran == [], (rc, ran))
            zrun("zf", "--case", "d/a.txt", task="ZP4")
            rc, out, ran = swap_("green_cover", nocase, lambda: zrun("zf", "--case", "d/b.txt", task="ZP4"))
            plant_("not re-run: a cover that reads one case as the whole scope skips a case never run", ran == [], (rc, ran))
            zrun("zg", task="ZP5")
            rc, out, ran = swap_("green_cover", anytask, lambda: zrun("zg", task="ZP6"))
            plant_("not re-run: a cover that reads another task's records skips what this task never ran", ran == [], (rc, ran))
            rcm = swap_("green_cover", lambda *a, **k: None, lambda: zrun("zg", task="ZP5"))
            plant_("not re-run: a cover that never finds a green runs the scope again (the arm the checks above stand on)",
                   rcm[2] == ["e/x.txt"], rcm)
            rep = cmd_report({"scopes": {"a": {}}, "logs_dir": "logs"}, zroot)
            check("report: the scopes not re-run are counted apart, never as a redundant loop",
                  "not re-run " in rep["counts"] and any(ln.startswith("not re-run: ") for ln in rep["lines"]),
                  (rep["counts"], rep["lines"][-4:]))

            # -- G · --redarm: one scratch copy per plant ------------------------------------------------
            proot = tmp / "proj"
            for d in ("plants", "images", "models", ".venv", "sub/data"):
                (proot / d).mkdir(parents=True)
            (proot / "emit.py").write_text(EMIT, encoding="utf-8")
            (proot / "data.txt").write_text("ok\n", encoding="utf-8")
            (proot / "other.txt").write_text("fine\n", encoding="utf-8")
            (proot / "images/a.png").write_bytes(b"\x89PNG")
            (proot / "models/m.bin").write_bytes(b"M")
            (proot / ".venv/pyvenv.cfg").write_text("x", encoding="utf-8")
            (proot / "sub/data/d.csv").write_text("d", encoding="utf-8")
            (proot / "plants/bug.patch").write_text("--- a/data.txt\n+++ b/data.txt\n@@ -1 +1 @@\n-ok\n+BUG\n", encoding="utf-8")
            (proot / "plants/nop.patch").write_text("--- a/other.txt\n+++ b/other.txt\n@@ -1 +1 @@\n-fine\n+still fine\n",
                                                    encoding="utf-8")
            (proot / "plants/off.patch").write_text("--- a/data.txt\n+++ b/data.txt\n@@ -1 +1 @@\n-not there\n+BUG\n",
                                                    encoding="utf-8")
            (proot / "plant.py").write_text("open('data.txt', 'w').write('BUG')\n", encoding="utf-8")
            pman = {"logs_dir": "logs", "vars": {"emit": ["emit.py"]},
                    "scopes": {"chk": {"cmd": f"{py} check data.txt",
                                       "plants": [{"id": "diff", "patch": "plants/bug.patch"},
                                                  {"id": "fn", "patch": "{python} -S plant.py"}]},
                               "unarmed": {"cmd": f"{py} check data.txt"}}}
            before = {p: p.read_bytes() for p in proot.rglob("*") if p.is_file()}
            rr = redarm(pman, "chk", proot, "T9", 2, box="SELFBOX")
            check("--redarm: the clean copy GO, a patch plant and a function plant each RED -> GO",
                  not rr["failures"] and not rr["nogo"] and rr["data"]["red"] == 2, (rr["lines"], rr["failures"]))
            check("--redarm writes nothing in the tree but its log and its record",
                  {p: p.read_bytes() for p in proot.rglob("*") if p.is_file() and "logs" not in p.parts} == before)
            for name, plants_, want in (("a plant the scope cannot see", [{"id": "nop", "patch": "plants/nop.patch"}], "stayed GO"),
                                        ("a patch that does not apply", [{"id": "off", "patch": "plants/off.patch"}], "does not apply"),
                                        ("a plant command that fails", [{"id": "bad", "patch": "{python} -S -c \"raise SystemExit(3)\""}],
                                         "exited 3"),
                                        ("a plant that leaves the scope unable to start",
                                         [{"id": "gone", "patch": "{python} -S -c \"import os; os.remove('emit.py')\""}],
                                         "an error, not a red")):
                m2 = json.loads(json.dumps(pman))
                m2["scopes"]["chk"]["plants"] = plants_
                r = redarm(m2, "chk", proot, "T9", 2, box="SELFBOX")
                plant_(f"--redarm: {name} is NO-GO, never a red", any(want in f for f in r["failures"]), r["failures"])
            r = redarm(pman, "unarmed", proot, "T9", 1, box="SELFBOX")
            plant_("--redarm: a scope that declares no plant is NO-GO", bool(r["nogo"]) and "declares no plant" in r["nogo"])
            (proot / "data.txt").write_text("BUG\n", encoding="utf-8")
            r = redarm(pman, "chk", proot, "T9", 1, box="SELFBOX")
            plant_("--redarm: a clean copy that is already red is NO-GO, its plants not run",
                   any(f.startswith("clean:") for f in r["failures"]) and r["data"]["red"] == 0, r["failures"])
            (proot / "data.txt").write_text("ok\n", encoding="utf-8")
            dest = tmp / "copy"
            (proot / "logs" / "T9.chk.log.1.part").write_text("a running command's output\n", encoding="utf-8")
            scratch_copy(proot, dest)
            (proot / "logs" / "T9.chk.log.1.part").unlink()
            check("the scratch copy leaves out models, data, .git at any depth; keeps images/ empty; links .venv back",
                  (dest / "data.txt").is_file() and not (dest / "models").exists() and not (dest / "sub/data").exists()
                  and (dest / "images").is_dir() and not list((dest / "images").iterdir())
                  and (os.name == "nt" or (dest / ".venv").is_symlink()))
            plant_("the scratch copy leaves out a `*.part` file - another arm's, gone mid-copy, would fail the copy",
                   (dest / "logs").is_dir() and not list((dest / "logs").glob("*.part")))
            dpatch = tmp / "dp"
            dpatch.mkdir()
            (dpatch / "w.txt").write_bytes(b"one\r\ntwo\r\n")
            (dpatch / "k.txt").write_text("same\nsame\n", encoding="utf-8")
            try:
                applied = apply_unified(dpatch, "--- a/w.txt\n+++ b/w.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+TWO\n"
                                                "--- /dev/null\n+++ b/n.txt\n@@ -0,0 +1 @@\n+new\n")
            except ValueError as exc:
                applied = str(exc)
            check("the patch applier keeps a file's CRLF, creates a new file",
                  applied == ["w.txt", "n.txt"] and (dpatch / "w.txt").read_bytes() == b"one\r\nTWO\r\n"
                  and (dpatch / "n.txt").read_text() == "new\n", applied)
            for name, patch, want in (("old lines found twice", "--- a/k.txt\n+++ b/k.txt\n@@ -1 +1 @@\n-same\n+x\n", "2 times"),
                                      ("a path outside the tree", "--- a/../x\n+++ b/../x\n@@ -1 +1 @@\n-a\n+b\n", "outside the tree"),
                                      ("no hunk", "just words\n", "no hunk")):
                try:
                    apply_unified(dpatch, patch)
                    plant_(f"the patch applier refuses {name}", False, "applied")
                except (ValueError, OSError) as exc:
                    plant_(f"the patch applier refuses {name}", want in str(exc), str(exc))

            # -- H · the loop: the plants of a changed scope, the flags --------------------------------------
            lc = lambda mch, blk=None, chg=(): loop_checks(pman, proot, "T9", 1, "SELFBOX", proot / "verify.json",  # noqa: E731
                                                          block=blk, changed=list(chg), mchanges=mch)
            lines_, fails_ = lc({"chk": "changed"})
            check("the loop runs the plants of a changed scope; all red, no failure",
                  not fails_ and any("plant diff · RED" in ln for ln in lines_), (lines_, fails_))
            m3 = json.loads(json.dumps(pman))
            m3["scopes"]["chk"]["plants"] = [{"id": "nop", "patch": "plants/nop.patch"}]
            _l, f3 = loop_checks(m3, proot, "T9", 1, "SELFBOX", proot / "verify.json", block=None, changed=[],
                                 mchanges={"chk": "added"})
            plant_("the loop: a changed scope's green plant is NO-GO", any(f.startswith("redarm chk:") for f in f3), f3)
            lines_, _f = lc({"unarmed": "added"})
            plant_("the loop flags a changed scope with no plant", any(ln.startswith("FLAG plants: scope `unarmed` added")
                                                                       for ln in lines_), lines_)
            # the red-arm proof the loop reads: a scope whose newest red-arm record on this box is GO, every plant RED,
            # on the same fingerprint, is not re-run - whichever task recorded it (Tracelax's M0-D13, M0-T64)
            (proot / "tool.py").write_text("v1\n", encoding="utf-8")
            fman = json.loads(json.dumps(pman))
            fman["scopes"]["chk"]["paths"] = ["data.txt"]
            fman["scopes"]["chk"]["plants"] = [{"id": "diff", "patch": "plants/bug.patch"}]
            fpl = proot / "logs" / "loop_times.jsonl"
            fnow = lambda m=None: fingerprint(m or fman, "chk", proot, visible_files(proot, m or fman, proot / "logs"),  # noqa: E731
                                              proot / "tool.py")

            def frec(task, verdict="GO", plants=None, fp=None, box="SELFBOX", logs=fpl):
                with open(logs, "a", encoding="utf-8") as fh:
                    fh.write(json.dumps({"ts": "2999-01-01T00:00:00", "task": task, "mode": "redarm", "box": box,
                                         "scopes": {"chk": {"plants": {"diff": "RED"} if plants is None else plants,
                                                            "verdict": verdict, "fingerprint": fp or fnow()}}}) + "\n")

            def fgate(m=None, proven=None):
                ls, fl = loop_checks(m or fman, proot, "T9", 1, "SELFBOX", proot / "verify.json", block=None,
                                     changed=[], mchanges={"chk": "changed"}, tool=proot / "tool.py", proven=proven)
                return ("skipped" if any("red-arm proven by" in x for x in ls) else
                        "ran" if any("plant diff · RED" in x for x in ls) else "?"), ls, fl

            fp0 = fnow()
            redarm(fman, "chk", proot, "T7", 1, box="SELFBOX", tool=proot / "tool.py")
            last = json.loads(fpl.read_text(encoding="utf-8").splitlines()[-1])
            check("--redarm's record carries the scope's fingerprint, the plants RED and the verdict",
                  last["scopes"]["chk"].get("fingerprint") == fp0 and last["scopes"]["chk"]["plants"] == {"diff": "RED"}
                  and last["scopes"]["chk"]["verdict"] == "GO" and bool(fp0), last)
            proven_ = {}
            verdict, ls_, fl_ = fgate(proven=proven_)
            check("the loop skips a scope whose red-arm, another task's, has the same fingerprint - and says by whom",
                  verdict == "skipped" and not fl_ and any("proven by T7" in x and fp0[:12] in x for x in ls_), (ls_, fl_))
            check("... and hands the claim's record the proof it relied on",
                  list(proven_) == ["chk"] and proven_["chk"].startswith("T7 ") and proven_["chk"].endswith(fp0[:12]),
                  proven_)
            with open(proot / "data.txt", "ab") as fh:
                fh.write(b"x")
            plant_("the loop re-runs a scope when one byte of a `paths` file changed", fgate()[0] == "ran")
            real_proof = redarm_proof
            globals()["redarm_proof"] = lambda *_a, **_k: ("MUTANT", "0")    # the skip made unconditional
            try:
                mutant = fgate()[0]
            finally:
                globals()["redarm_proof"] = real_proof
            plant_("an unconditional skip (any record vouching, whatever the bytes) is seen by the re-run cases",
                   mutant == "skipped" and fgate()[0] == "ran", mutant)
            (proot / "data.txt").write_text("ok\n", encoding="utf-8")
            check("... and skips again once the bytes are back", fgate()[0] == "skipped")
            fout = json.loads(json.dumps(fman))
            fout["scopes"]["chk"]["paths"] = ["other.txt"]                   # the plant patches data.txt, outside `paths`
            frec("T8", fp=fnow(fout))
            check("a proof of a scope whose plant patches a file outside its `paths` skips on the same bytes",
                  fgate(fout)[0] == "skipped")
            with open(proot / "data.txt", "ab") as fh:
                fh.write(b"x")
            plant_("the loop re-runs a scope when a file its plant patches, outside its `paths`, changed",
                   fgate(fout)[0] == "ran")
            (proot / "data.txt").write_text("ok\n", encoding="utf-8")
            frec("T8")                                               # the proof of fman's own entry, newest again
            (proot / "plants/bug.patch").write_text("--- a/data.txt\n+++ b/data.txt\n@@ -1 +1 @@\n-ok\n+BUG2\n", encoding="utf-8")
            plant_("the loop re-runs a scope when a plant's patch file changed", fgate()[0] == "ran")
            (proot / "plants/bug.patch").write_text("--- a/data.txt\n+++ b/data.txt\n@@ -1 +1 @@\n-ok\n+BUG\n", encoding="utf-8")
            f2 = json.loads(json.dumps(fman))
            f2["scopes"]["chk"]["timeout_s"] = 77
            plant_("the loop re-runs a scope when its manifest entry changed", fgate(f2)[0] == "ran")
            f2["scopes"]["chk"].pop("timeout_s")
            f2["scopes"]["chk"]["_note"] = "wording only"
            f2["scopes"]["chk"]["note"] = "a note is no entry"
            check("... but not when only a note's wording changed", fgate(f2)[0] == "skipped")
            (proot / "tool.py").write_text("v2\n", encoding="utf-8")
            plant_("the loop re-runs a scope when the runner changed", fgate()[0] == "ran")
            frec("T8")
            check("... and a record of the new runner vouches again, whichever task wrote it", fgate()[0] == "skipped")
            frec("T8", verdict="NO-GO")
            plant_("the loop re-runs a scope whose newest red-arm record is NO-GO", fgate()[0] == "ran")
            frec("T8", plants={"diff": "GREEN"})
            plant_("the loop re-runs a scope whose record has a GREEN plant", fgate()[0] == "ran")
            f3 = json.loads(json.dumps(fman))
            f3["scopes"]["chk"]["plants"].append({"id": "fn", "patch": "{python} -S plant.py"})
            frec("T8", fp=fnow(f3))                                  # a record of the one-plant entry, read for the two-plant one
            plant_("the loop re-runs a scope when a plant was added", fgate(f3)[0] == "ran")
            frec("T8", plants={"diff": "RED", "fn": "RED"}, fp=fnow())
            plant_("the loop re-runs a scope whose record names a plant the manifest no longer has", fgate()[0] == "ran")
            fp_fn = fnow(f3)
            (proot / "plant.py").write_text("open('data.txt', 'w').write('BUG!')\n", encoding="utf-8")
            plant_("the fingerprint reads a function plant's own script", fnow(f3) != fp_fn)
            (proot / "plant.py").write_text("open('data.txt', 'w').write('BUG')\n", encoding="utf-8")
            fnew = json.loads(json.dumps(fman))
            (proot / "plants/new.patch").write_text("--- /dev/null\n+++ b/made.txt\n@@ -0,0 +1 @@\n+BUG\n", encoding="utf-8")
            fnew["scopes"]["chk"]["plants"] = [{"id": "new", "patch": "plants/new.patch"}]
            check("a plant that creates a file: the file absent is a fingerprint, not an unreadable member",
                  fnow(fnew) is not None and patch_targets((proot / "plants/new.patch").read_text()) == ["made.txt"])
            (proot / "plants/new.patch").unlink()
            other = tmp / "otherbox"
            other.mkdir()
            frec("T8", box="OTHERBOX", logs=other / "loop_times.jsonl")
            plant_("a record from another box is no proof",
                   redarm_proof(fman, "chk", other, "SELFBOX", fnow()) is None
                   and redarm_proof(fman, "chk", other, "OTHERBOX", fnow()) is not None)
            plant_("no fingerprint (the tree could not be listed) is no proof",
                   redarm_proof(fman, "chk", proot / "logs", "SELFBOX", None) is None
                   and fingerprint(fman, "chk", proot, None) is None)
            frec("T8")
            check("a good record is the proof again - the cases above left the tree as they found it", fgate()[0] == "skipped")

            # -- G2 · a faster red arm: the arms side by side, a plant's own case, the clean arm proven, the sweep
            qroot = tmp / "arms"
            (qroot / "plants").mkdir(parents=True)
            (qroot / "emit.py").write_text(EMIT, encoding="utf-8")
            qtool = qroot / "tool.py"
            qtool.write_text("v1\n", encoding="utf-8")
            for f in ("data", "a", "b", "x", "y"):
                (qroot / f"{f}.txt").write_text("ok\n", encoding="utf-8")
                (qroot / f"plants/{f}.patch").write_text(f"--- a/{f}.txt\n+++ b/{f}.txt\n@@ -1 +1 @@\n-ok\n+BUG\n", encoding="utf-8")
            three = [{"id": f"p{k}", "patch": "plants/data.patch"} for k in (1, 2, 3)]
            qman = {"logs_dir": "logs", "vars": {"emit": ["emit.py"]}, "scopes": {
                "par": {"cmd": f"{py} check data.txt 0.15", "paths": ["a.txt"], "plants": three},
                "ex": {"cmd": f"{py} check data.txt 0.15", "exclusive": True, "plants": three[:2]},
                "cs": {"cases": ["x", "y"], "case_cmd": f"{py} check {{case}}.txt", "expected": 2,
                       "plants": [{"id": "yb", "patch": "plants/y.patch", "case": "y"}]},
                "cf": {"cases_cmd": f"{py} rows x y", "case_cmd": f"{py} check {{case}}.txt", "expected": 2,
                       "plants": [{"id": "yb", "patch": "plants/y.patch", "case": "y"}]},
                "m1": {"cmd": f"{py} check a.txt", "paths": ["a.txt"], "plants": [{"id": "a", "patch": "plants/a.patch"}]},
                "m2": {"cmd": f"{py} check b.txt", "paths": ["b.txt"], "plants": [{"id": "b", "patch": "plants/b.patch"}]},
                "bare": {"cmd": f"{py} go"}}}
            qrecs = qroot / "logs" / "loop_times.jsonl"
            qcopy = lambda m=None: json.loads(json.dumps(m or qman))                     # noqa: E731
            overlap = lambda sp: any(a[1] < b[2] and b[1] < a[2] for i, a in enumerate(sp) for b in sp[i + 1:])  # noqa: E731

            def qarm(scope, task, m=None, jobs=4, record=True):
                trace.write_text("", encoding="utf-8")
                r = redarm(m or qman, scope, qroot, task, jobs, box="SELFBOX", record=record, tool=qtool)
                return r, spans()

            def swap(fn_name, mutant, call):                 # one mutant of this runner, put back whatever happens
                real = globals()[fn_name]
                globals()[fn_name] = mutant
                try:
                    return call()
                finally:
                    globals()[fn_name] = real

            r, sp = qarm("par", "Q1")
            check("--redarm: the arms side by side - the clean and three plants, each in its own copy, GO",
                  not r["failures"] and r["data"]["red"] == 3 and r["data"]["workers"] == 4 and len(sp) == 4 and overlap(sp),
                  (r["lines"], sp))
            check("... their lines in declared order, whatever order they ended in",
                  [ln.split(" · ")[1] for ln in r["lines"]] == ["clean", "plant p1", "plant p2", "plant p3"], r["lines"])
            check("... each arm's line its number among the four and the arms left (M0-D27)",
                  [ln.rsplit(" · ", 1)[1] for ln in r["lines"]] == [f"arm {k}/4, {4 - k} left" for k in (1, 2, 3, 4)],
                  r["lines"])
            qlog = (qroot / "logs" / "Q1.par.redarm.log").read_text(encoding="utf-8")
            heads = [qlog.find(f"par · {x} · a fresh scratch copy") for x in ("clean", "plant p1", "plant p2", "plant p3")]
            check("... the campaign's log one arm after another in declared order, no part file left",
                  -1 not in heads and heads == sorted(heads) and not list((qroot / "logs").glob("*.part")), heads)
            check("arm_workers: a `box` or `free_ports` scope one arm at a time, the rest up to the jobs",
                  (arm_workers({"box": "B"}, 8, 3), arm_workers({"free_ports": [9]}, 8, 3), arm_workers({}, 8, 3),
                   arm_workers({}, 2, 5)) == (1, 1, 3, 2))
            r, sp = qarm("ex", "Q2")
            check("--redarm: an `exclusive` scope's arms one at a time", not r["failures"] and r["data"]["workers"] == 1
                  and len(sp) == 3 and not overlap(sp), sp)
            _r, msp = swap("arm_workers", lambda s, j, n: max(1, min(int(j), n)), lambda: qarm("ex", "Q2m", record=False))
            plant_("arms of an `exclusive` scope side by side: red - the serial case sees them overlap",
                   overlap(msp) and not overlap(qarm("ex", "Q2b", record=False)[1]), msp)
            r, sp = qarm("par", "Q3")
            check("--redarm: the clean arm a record on today's fingerprint proved is not re-run - and says by whom",
                  not r["failures"] and r["lines"][0].startswith("par · clean: proven by Q1 ") and len(sp) == 3
                  and r["data"]["clean"] == "PROVEN", (r["lines"], sp))
            last = json.loads(qrecs.read_text(encoding="utf-8").splitlines()[-1])
            check("... its record says so: clean PROVEN, the campaign GO, the fingerprint kept",
                  last["scopes"]["par"]["clean"] == "PROVEN" and last["scopes"]["par"]["verdict"] == "GO"
                  and bool(last["scopes"]["par"]["fingerprint"]), last)
            (qroot / "data.txt").write_text("ok\nmore\n", encoding="utf-8")       # a file the plants patch, outside `paths`
            r, sp = qarm("par", "Q4")
            plant_("the clean arm re-runs when a file the plants patch, outside `paths`, changed",
                   r["lines"][0].startswith("par · clean · GO") and len(sp) == 4 and not r["failures"], r["lines"])
            (qroot / "data.txt").write_text("ok\n", encoding="utf-8")
            rm, _sp = swap("clean_proof", lambda *_a, **_k: ("MUTANT", "0"), lambda: qarm("par", "Q5m", record=False))
            r, sp = qarm("par", "Q5")
            plant_("a clean arm skipped whatever the bytes: red - the moved-bytes case runs it",
                   "proven by MUTANT" in rm["lines"][0] and r["lines"][0].startswith("par · clean · GO"), (rm["lines"][0], r["lines"][0]))
            last = json.loads(qrecs.read_text(encoding="utf-8").splitlines()[-1])
            last["task"], last["scopes"]["par"]["clean"] = "Q6x", "NO-GO"
            with open(qrecs, "a", encoding="utf-8") as fh:
                fh.write(json.dumps(last) + "\n")
            r, _sp = qarm("par", "Q6", record=False)
            plant_("the clean arm re-runs when the newest record's clean was NO-GO", r["lines"][0].startswith("par · clean · GO"),
                   r["lines"])
            leg = tmp / "legacy"
            leg.mkdir()
            (leg / "loop_times.jsonl").write_text(json.dumps({"ts": "t", "task": "OLD", "mode": "redarm", "box": "SELFBOX", "scopes": {
                "par": {"plants": {"p1": "RED"}, "verdict": "GO", "fingerprint": "f"}}}) + "\n", encoding="utf-8")
            plant_("the clean proof: a record from before the field proves on its GO, its own box and fingerprint only",
                   clean_proof("par", leg, "SELFBOX", "f") == ("OLD", "t") and clean_proof("par", leg, "OTHERBOX", "f") is None
                   and clean_proof("par", leg, "SELFBOX", "g") is None and clean_proof("par", leg, "SELFBOX", None) is None)
            r, sp = qarm("cs", "Q7")
            check("a plant's `case`: its arm runs that case alone and goes red - the clean arm runs the whole scope",
                  not r["failures"] and r["data"]["red"] == 1 and len(sp) == 3 and "(case y)" in r["lines"][1], (r["lines"], sp))
            qw = qcopy()
            qw["scopes"]["cs"]["plants"][0]["case"] = "x"                       # x cannot see a bug planted in y
            r, _sp = qarm("cs", "Q8", m=qw, record=False)
            check("a `case` that cannot see the plant reads GREEN, so NO-GO - never silently green",
                  r["data"]["green"] == 1 and any("stayed GO" in f for f in r["failures"]), r["failures"])
            rm, _sp = swap("plant_case", lambda _a: None, lambda: qarm("cs", "Q8m", m=qw, record=False))
            plant_("a `case` arm run as the whole scope: red - the wrong-case plant reads RED under it",
                   rm["data"]["red"] == 1 and not rm["failures"] and r["data"]["green"] == 1, rm["lines"])
            for nm, scope, case, patch_, want in (("a listed scope", "cs", "z", "y", "not one of the scope's cases"),
                                                  ("a `cases_cmd` scope", "cf", "z", "y", "not one of the scope's cases"),
                                                  ("a scope with no cases", "m1", "y", "a", "a scope that has none")):
                qz = qcopy()
                qz["scopes"][scope]["plants"] = [{"id": "z", "patch": f"plants/{patch_}.patch", "case": case}]
                r, _sp = qarm(scope, "Q9", m=qz, record=False)
                plant_(f"a plant `case` {nm} does not list: an ERROR, never a red or a skip",
                       r["data"]["errors"] == 1 and r["data"]["red"] == 0 and any(want in f for f in r["failures"]), r["failures"])
            r, _sp = qarm("cf", "Q10")
            check("a `cases_cmd` scope's plant `case`: listed in the planted copy, run alone, red",
                  not r["failures"] and r["data"]["red"] == 1, r["lines"])
            check("a plant `case` that is no id blocks its scope, named", "`case` is one case id" in blocked(
                {"scopes": {"s": {"cmd": "x", "plants": [{"id": "p", "patch": "q.patch", "case": "a b"}]}}}).get("s", ""))
            qz = qcopy()
            qz["scopes"]["cs"]["plants"][0]["case"] = "z"
            check("list shows a plant's case, and names one its scope does not list",
                  any("yb: case y)" in x for x in cmd_list(qman)["lines"])
                  and any("yb: case z - not one of its cases" in x for x in cmd_list(qz)["lines"]))
            mman = dict(qman, scopes={k: qman["scopes"][k] for k in ("m1", "m2", "bare")})
            mman["scopes"]["far"] = dict(qman["scopes"]["m1"], box="OTHERBOX")
            sw = lambda task, m=None: moved_sweep(m or mman, qroot, task, 2, box="SELFBOX", tool=qtool)  # noqa: E731
            r = sw("Q11")
            check("--redarm --moved: no proof yet - every armed scope here red-armed, one campaign each; the owed one named",
                  r["counts"].startswith("moved: 2 of 2 armed scopes, plants RED 2, GREEN 0, ERROR 0") and not r["failures"]
                  and not r["nogo"] and any(x.startswith("far · owed on OTHERBOX") for x in r["lines"]), (r["counts"], r["lines"]))
            check("... each campaign's line its number among the two and the campaigns left (M0-D27)",
                  [x.rsplit(" · ", 1)[1] for x in r["lines"] if x.startswith("redarm: ")]
                  == ["campaign 1/2, 1 left", "campaign 2/2, 0 left"], r["lines"])
            r = sw("Q12")
            check("--redarm --moved: nothing moved - GO, named, each scope on the proof it stands on",
                  r["counts"].startswith("moved: 0 of 2 armed scopes") and not r["failures"] and not r["nogo"]
                  and any(x.startswith("nothing moved") for x in r["lines"]) and sum("proven by Q11" in x for x in r["lines"]) == 2,
                  r["lines"])
            (qroot / "b.txt").write_text("ok\nb\n", encoding="utf-8")
            r = sw("Q13")
            plant_("--redarm --moved: a moved scope is red-armed, the unmoved named on its proof",
                   r["counts"].startswith("moved: 1 of 2") and any(x.startswith("m2 · plant b · RED") for x in r["lines"])
                   and any(x.startswith("m1 · proven by") for x in r["lines"]), r["lines"])
            (qroot / "b.txt").write_text("ok\n", encoding="utf-8")
            rm = swap("redarm_proof", lambda *_a, **_k: ("MUTANT", "0"), lambda: sw("Q14m"))
            r = sw("Q14")
            plant_("--moved skipping a moved scope: red - the moved case sees it re-armed",
                   rm["counts"].startswith("moved: 0 of 2") and r["counts"].startswith("moved: 1 of 2"), (rm["counts"], r["counts"]))
            mg = qcopy(mman)
            mg["scopes"]["m2"]["plants"] = [{"id": "nop", "patch": "plants/a.patch"}]     # m2 reads b.txt: blind to it
            r = sw("Q15", mg)
            plant_("--redarm --moved: one campaign NO-GO makes the sweep NO-GO",
                   bool(r["failures"]) and "GREEN 1" in r["counts"] and any("stayed GO" in f for f in r["failures"]), r["counts"])
            r = moved_sweep({"logs_dir": "logs", "scopes": {"bare": {"cmd": "x"}}}, qroot, "Q16", 1, box="SELFBOX")
            plant_("--redarm --moved with no armed scope: NO-GO, named", "nothing to sweep" in (r["nogo"] or ""), r["nogo"])
            for name, args in (("--moved without --redarm", ("--moved",)), ("--redarm with no scope, no --moved", ("--redarm",)),
                               ("--redarm <scope> --moved", ("--redarm", "a", "--moved"))):
                rc, out = run_(clean, *args)
                plant_(f"a wrong call ({name}) prints the right one", rc == 1 and "the calls:" in out and "--redarm --moved" in out,
                       out[-300:])
            rc, out = run_(clean, "--redarm", "--moved")
            check("--redarm --moved through the CLI: the fixture arms no scope - NO-GO, nothing to sweep",
                  rc == 1 and "nothing to sweep" in out, out[-300:])
            (proot / "tool.py").unlink()
            bman ={"scopes": {"u": {"cmd": "python3 tests/run.py --fast"},
                               "l": {"cmd": "python3 tools/pb/lint.py --plan milestones/m1/m1_implementation_plan.md"}}}
            build = {"type": "BUILD", "deliver": "src/x.py, the new parser, run by hand", "plan": "p"}
            fl = loop_flags(build, ["tests/test_a.py", "src/x.py", "tools/pb/lint.py", "tools/pb/verify.json",
                                    "milestones/m1/m1_implementation_plan.md"],
                            {"u": "changed"}, bman, "tools/pb/verify.json", "M1-T3", "HEAD")
            plant_("the oracle flag: a test, a check and the manifest the Deliver does not name - never the plan a check reads",
                   sum(ln.startswith("FLAG oracle:") for ln in fl) == 4 and not any("src/x.py" in ln for ln in fl), fl)
            fl = loop_flags(dict(build, deliver="src/x.py and tests/test_a.py; tools/pb/verify.json"),
                            ["tests/test_a.py", "src/x.py", "tools/pb/verify.json"], {"u": "changed"}, bman,
                            "tools/pb/verify.json", "M1-T3", "HEAD")
            check("no oracle flag when the Deliver names the test and the manifest", fl == [], fl)
            fl = loop_flags({"type": "CHECK", "deliver": "reports/v.md"}, ["src/x.py", "milestones/m1/reports/v.md"], {},
                            bman, "tools/pb/verify.json", "M1-V2", "HEAD")
            plant_("the type flag: a CHECK block that changed code", len(fl) == 1 and fl[0].startswith("FLAG type: src/x.py"), fl)
            fl = loop_flags({"type": "MOVE", "deliver": ""}, ["milestones/m1/m1_implementation_plan.md",
                                                              "milestones/m1/plan_archive.md", "README.md"], {}, bman,
                            "tools/pb/verify.json", "M1-TC2", "HEAD")
            plant_("the type flag: a MOVE block that wrote outside the plan", len(fl) == 1 and "README.md" in fl[0], fl)
            fl = loop_flags({"type": "MOVE", "deliver": ""}, ["milestones/m9/m9_rules.md", "src/x.py"], {}, bman,
                            "tools/pb/verify.json", "M9-TC1", "HEAD")
            plant_("the type flag: a MOVE block moving a Rules entry to m<N>_rules.md is not flagged (PLAYBOOK §0)",
                   not any("m9_rules.md" in ln for ln in fl), fl)
            plant_("the type flag: a MOVE block writing code beside it still is", len(fl) == 1 and "src/x.py" in fl[0], fl)
            check("no block: one line, no flag", loop_flags(None, ["x"], {}, bman, "v.json", "M1-T9", "HEAD")[0].startswith("flags: M1-T9"))
            proot2 = tmp / "planned"
            (proot2 / "milestones/m1").mkdir(parents=True)
            (proot2 / "milestones/m1/m1_implementation_plan.md").write_text(
                "# M1 - fixture\n\n## Flow\n| Task | Type | Goal | Order | Status |\n|---|---|---|---|---|\n"
                "| M1-V2 | CHECK | v | FIRST | TODO |\n| M1-T3 | JUDGE | t | AFTER V2 | TODO |\n\n# Tasks\n\n"
                "## M1-V2 · Validation · **CHECK** · (FIRST)\n- Status: TODO\n- Deliver: reports/v.md\n\n"
                "## M1-T3 · A tool · **JUDGE** · (AFTER V2)\n- Status: TODO\n- Deliver: tools/pb/x.py\n", encoding="utf-8")
            b2, b3, b9 = (find_block(proot2, t) for t in ("M1-V2", "M1-T3", "M1-T9"))
            check("find_block reads the type from the tag, and by id under an old word (JUDGE on a T: BUILD)",
                  b2 and b2["type"] == "CHECK" and "reports/v.md" in b2["deliver"] and b3 and b3["type"] == "BUILD"
                  and b9 is None, (b2, b3, b9))

            # -- I · report ------------------------------------------------------------------------------
            rlogs = tmp / "rep" / "logs"
            rlogs.mkdir(parents=True)
            rman = {"scopes": {"a": {}}, "logs_dir": "logs"}
            rows_ = [("T1", "all", "GO", "t1", "BOXA", 10), ("T1", "all", "GO", "t1", "BOXA", 12),
                     ("T2", "scope", "NO-GO", "t2", "BOXA", 1), ("T2", "changed", "GO", "t3", "BOXB", 90),
                     ("T2", "all", "GO", "t3", "BOXB", 95), ("T3", "changed", "GO", "t4", "BOXA", 2),
                     ("T4", "case", "GO", "t5", "BOXA", 1), ("T4", "case", "GO", "t5", "BOXA", 1)]
            (rlogs / "loop_times.jsonl").write_text("".join(json.dumps(
                {"ts": f"2999-01-0{i + 1}T00:00:00", "task": t, "mode": m, "verdict": v, "tree": tr, "box": b, "seconds": s,
                 "load": 0.0, "cpus": 8, "scopes": {"a": {"passed": 1, "failed": 0, "skipped": 0, "seconds": s}}}) + "\n"
                for i, (t, m, v, tr, b, s) in enumerate(rows_)), encoding="utf-8")
            rep = cmd_report(rman, tmp / "rep", budget=50)
            check("report: first-try passes per task, medians per box, loops per task",
                  rep["data"]["first_try"] == 3 and rep["data"]["tasks"] == 4 and "BOXA 11.00 s" in rep["counts"]
                  and "BOXB 95.00 s" in rep["counts"] and "loops per task 1.67" in rep["counts"], rep["counts"])
            plant_("report: a second GO loop of one kind on one tree is redundant; two case runs are no loop",
                   rep["data"]["redundant"] == 1, rep["lines"])
            plant_("report: the TR line only over --budget, on its box", rep["data"]["tr"] == ["BOXB"]
                   and cmd_report(rman, tmp / "rep", budget=500)["data"]["tr"] == [] and "TR" not in cmd_report(rman, tmp / "rep")["counts"])
            plant_("report --since a later date: NO-GO, no record", bool(cmd_report(rman, tmp / "rep", since="3000-01-01")["nogo"]))
            with open(rlogs / "loop_times.jsonl", "a", encoding="utf-8") as fh:
                fh.write("{not json\n")
            rep = cmd_report(rman, tmp / "rep")
            plant_("report: a line that is not JSON is foreign - counted, the rest read, never NO-GO on its own",
                   not rep["nogo"] and rep["data"]["foreign"] == 1 and " 1 foreign " in rep["counts"]
                   and rep["data"]["tasks"] == 4, rep["counts"])
            own = {"ts": "2999-02-01T00:00:00", "task": "T1", "mode": "all", "verdict": "GO", "tree": "t1", "box": "BOXA",
                   "seconds": 5, "load": 0.0, "cpus": 8, "scopes": {"a": {"passed": 1, "failed": 0, "skipped": 0, "seconds": 5}}}
            fman = {"scopes": {"a": {}}, "logs_dir": "logs", "budget": {"cases_scope": "plan_index", "boxes": {
                "BOXA": {"base_s": 5.0, "at_cases": 1, "per_case_s": 0.1, "load_k": 0.0, "band_pct": 50}}}}   # every reader runs
            for name, other in (("a list of scope names (one fleet runner's)",
                                 {"scopes": ["plan_lint_m7"], "per_scope": [{"scope": "plan_lint_m7", "seconds": 0.0}]}),
                                ("a list of scope objects (five fleet runners')",
                                 {"scopes": [{"name": "b", "seconds": 3.0}, {"name": "c", "seconds": 1.0}]})):
                (rlogs / "loop_times.jsonl").write_text(json.dumps(own) + "\n" + json.dumps(dict(own, task="T2", **other))
                                                        + "\n", encoding="utf-8")
                rep = cmd_report(fman, tmp / "rep")
                plant_(f"report: `scopes` as {name} is foreign - counted, never read, no traceback",
                       not rep["nogo"] and rep["data"]["foreign"] == 1 and rep["data"]["tasks"] == 1, rep["counts"])
            (rlogs / "loop_times.jsonl").write_text(json.dumps(own) + "\n" + json.dumps(dict(
                own, task="T2", seconds=7, scopes={"plan_index": [1.32, "GO"], "x": "1.2 seconds"})) + "\n", encoding="utf-8")
            rep = cmd_report(fman, tmp / "rep")
            plant_("report: `scopes` as a mapping of lists (two fleet runners') is read as a record, its entries are not - "
                   "no traceback", not rep["nogo"] and rep["data"]["foreign"] == 0 and rep["data"]["tasks"] == 2
                   and "BOXA 6.00 s" in rep["counts"], rep["counts"])
            (rlogs / "loop_times.jsonl").write_text(json.dumps(dict(own, load={"start": [0.2, 0.1], "end": [0.2, 0.1]}))
                                                    + "\n", encoding="utf-8")
            rep = cmd_report(fman, tmp / "rep")
            plant_("report: an optional field in another shape (one fleet runner's `load` as an object) reads as absent - "
                   "the record read, its budget line NO LOAD SAMPLE, no traceback", not rep["nogo"]
                   and rep["data"]["foreign"] == 0 and any("NO LOAD SAMPLE" in ln for ln in rep["lines"]), rep["lines"][-3:])
            (rlogs / "loop_times.jsonl").write_text(json.dumps(dict(own, scopes=["a"])) + "\n", encoding="utf-8")
            rep = cmd_report(rman, tmp / "rep")
            plant_("report: every line foreign - NO-GO naming the count, no traceback", "1 foreign line(s)" in (rep["nogo"] or ""),
                   rep["nogo"])
            # the loop's own seconds (M0-T63): counted once in the median and the TR line, a record without them named
            (rlogs / "loop_times.jsonl").write_text("".join(json.dumps(dict(own, box="BOXC", ts=ts, **more)) + "\n" for ts, more in (
                ("2999-03-01T00:00:00", {"seconds": 10}),
                ("2999-03-02T00:00:00", {"seconds": 10, "loop_seconds": 5}),
                ("2999-03-03T00:00:00", {"seconds": 10, "loop_seconds": 5}))), encoding="utf-8")
            rep = cmd_report(rman, tmp / "rep", budget=12)
            check("report: the median counts the scopes and the loop, each second once (10 + 5, never 20)",
                  "BOXC 15.00 s" in rep["counts"], rep["counts"])
            check("report: a complete loop without the field read as 0 and named",
                  any(ln == "loop seconds unknown before 2999-03-02 - 1 complete loop(s) read as 0" for ln in rep["lines"]),
                  rep["lines"][-4:])
            check("report: the TR line fires on the loop's seconds too", rep["data"]["tr"] == ["BOXC"], rep["lines"][-4:])
            real_total = loop_total
            globals()["loop_total"] = lambda r: float(r.get("seconds", 0))
            try:
                blind = cmd_report(rman, tmp / "rep", budget=12)["data"]["tr"]
            finally:
                globals()["loop_total"] = real_total
            plant_("report's TR line blind to `loop_seconds`: red - a 15 s loop under a 12 s budget fires", blind != ["BOXC"], blind)
            (rlogs / "loop_times.jsonl").write_text(json.dumps(dict(own, loop_seconds="5 s", touched=["a.py"], base=3)) + "\n",
                                                    encoding="utf-8")
            rep = cmd_report(rman, tmp / "rep")
            check("report: the loop's fields in another shape read as absent, the record read",
                  not rep["nogo"] and rep["data"]["foreign"] == 0 and "BOXA 5.00 s" in rep["counts"], rep["counts"])
            tw = lambda t, m, ts, touched=None: dict(own, task=t, mode=m, ts=f"2999-04-0{ts}T00:00:00",  # noqa: E731
                                                       **({"touched": touched, "base": f"{t}'s start"} if touched else {}))
            (rlogs / "loop_times.jsonl").write_text("".join(json.dumps(r) + "\n" for r in (
                tw("T1", "all", 1, {"added": ["a.py", "keep.py"], "modified": [], "deleted": []}),
                tw("T2", "changed", 2, {"added": [], "modified": ["a.py", "b/c.md"], "deleted": []}),
                tw("T1", "all", 3, {"added": [], "modified": ["keep.py", "x.py"], "deleted": []}),
                tw("T3", "all", 4),
                tw("T5", "all", 5, {"added": [], "modified": [], "deleted": ["a.py"]}),
                tw("T4", "scope", 6, {"added": ["a.py"], "modified": [], "deleted": []}))) + "{not json\n", encoding="utf-8")
            tch = cmd_touched(rman, tmp / "rep")
            check("touched: per task its last claim's files, newest first; a scoped run is no claim",
                  not tch["nogo"] and list(tch["data"]["tasks"]) == ["T5", "T1", "T2"]
                  and tch["data"]["tasks"]["T1"]["modified"] == ["keep.py", "x.py"] and tch["data"]["tasks"]["T1"]["added"] == []
                  and "1 without (before the field) · 1 foreign" in tch["counts"], (tch["counts"], tch["lines"]))
            tch = cmd_touched(rman, tmp / "rep", pattern="a.py")
            check("touched --path: every task whose last claim touched it, newest first",
                  tch["data"]["paths"] == {"a.py": ["T5 (deleted, 2999-04-05T00:00:00)", "T2 (modified, 2999-04-02T00:00:00)"]}
                  and tch["lines"] == ["a.py: T5 (deleted, 2999-04-05T00:00:00) · T2 (modified, 2999-04-02T00:00:00)"], tch["lines"])
            tch = cmd_touched(rman, tmp / "rep", pattern="b/*.md", task="T2")
            check("touched --task --path: the one task, the matching paths", not tch["nogo"]
                  and tch["data"]["paths"] == {"b/c.md": ["T2 (modified, 2999-04-02T00:00:00)"]}, tch["data"]["paths"])
            for name, kw, want in (("a task whose claims carry no list", {"task": "T3"}, "no claim record of T3"),
                                   ("a glob nothing matches", {"pattern": "*.rs"}, "matching `*.rs`")):
                check(f"touched: {name} - NO-GO, named", want in (cmd_touched(rman, tmp / "rep", **kw)["nogo"] or ""))
            (rlogs / "loop_times.jsonl").write_text(json.dumps(tw("T3", "all", 4)) + "\n", encoding="utf-8")
            check("touched: no record names its files - NO-GO", "no claim record names" in (cmd_touched(rman, tmp / "rep")["nogo"] or ""))
            bman2 = {"scopes": {"s": {}}, "budget": {"cases_scope": "s", "boxes": {
                "B": {"base_s": 100.0, "at_cases": 400, "per_case_s": 0.5, "load_k": 0.01, "band_pct": 10, "cpus": 8}}}}

            def bl(secs, cases, load, scopes=1):
                return {"task": f"t{secs}", "seconds": secs, "load": load, "cpus": 8,
                        "scopes": {n: {"passed": cases if n == "s" else 0, "failed": 0, "seconds": secs} for n in ("s", "x")[:scopes]}}
            for name, rows, want in (("fits a loop on the line", [bl(104.0, 400, 4.0)], "+0.0%"),
                                     ("flags over band", [bl(150.0, 400, 4.0)], "OVER BAND"),
                                     ("prices the per-case term", [bl(110.0, 420, 0.0)], "110.0   110.0   +0.0%"),
                                     ("refuses an oversubscribed box", [bl(400.0, 400, 50.0)], "OUT OF RANGE"),
                                     ("names a missing load sample", [bl(999.0, 420, None)], "NO LOAD SAMPLE"),
                                     ("skips a different scope set", [bl(999.0, 400, 4.0, scopes=2)], "SCOPE SET DIFFERS")):
                check(f"budget {name}", want in "\n".join(budget_lines(bman2, "B", rows)[2:-1]))
            check("budget: inside the band is not flagged", "OVER BAND" not in "\n".join(budget_lines(bman2, "B", [bl(112.0, 400, 4.0)])))
            check("budget: an unmeasured box says so", "no budget measured" in budget_lines(bman2, "OTHER", [bl(1.0, 1, 1.0)])[0])
            check("budget: the normalised seconds beside the raw (104 s at load 4 of 8, k 0.08: 100.0)",
                  "   104.0   100.0   100.0   +0.0%" in "\n".join(budget_lines(bman2, "B", [bl(104.0, 400, 4.0)])))

            # -- I2 · report at an idle box (M0-T64d): the median beside the raw, the TR line on the normalised only
            def idle_rep(loops, budget):              # loops: (seconds, load); box BOXN, 8 CPUs, the default k
                (rlogs / "loop_times.jsonl").write_text("".join(json.dumps(dict(
                    own, box="BOXN", ts=f"2999-05-{i + 1:02d}T00:00:00", seconds=s, load=ld,
                    scopes={"a": {"passed": 1, "failed": 0, "skipped": 0, "seconds": s}})) + "\n"
                    for i, (s, ld) in enumerate(loops)), encoding="utf-8")
                return cmd_report(rman, tmp / "rep", budget=budget)
            busy = [(10.0, 8.0)] * 3                  # 10 s at load 8 of 8 CPUs: 10 / 1.15 = 8.70 s at an idle box
            rep = idle_rep(busy, 9.5)
            check("report: the complete loop's median raw and at an idle box, per box; the slowest scopes both ways",
                  "BOXN 10.00 s (8.70 s idle)" in rep["counts"] and any("a 10.00 s (8.70 s idle)" in ln for ln in rep["lines"])
                  and any(ln.startswith("BOXN: at an idle box") and "k 0.15 (default" in ln for ln in rep["lines"]),
                  (rep["counts"], rep["lines"]))
            check("report: a loop over the budget raw, under it at an idle box - no TR", rep["data"]["tr"] == [], rep["lines"])
            raw_tr = swap_("norm_factor", lambda r, man, box=None: 1.0, lambda: idle_rep(busy, 9.5))["data"]["tr"]
            plant_("the TR line reading the raw seconds: red - a loaded 10 s loop fires a 9.5 s budget", raw_tr == ["BOXN"], raw_tr)
            rep = idle_rep([(9.0, 0.0), (12.0, 0.0), (12.0, 0.0)], 9.5)
            check("report: a tree that got slower at load 0 still fires the TR line - no load to hide behind",
                  rep["data"]["tr"] == ["BOXN"] and any(ln.startswith("TR: the last 3 counted") for ln in rep["lines"]),
                  rep["lines"])
            rep = idle_rep([(9.0, 0.0)] * 3 + [(20.0, 0.0)] * 2, 15)
            check(f"report: the TR line reads the last {TR_LOOPS} counted loops - two slow loops after three fast fire it",
                  rep["data"]["tr"] == ["BOXN"] and "BOXN 9.00 s (9.00 s idle)" in rep["counts"], (rep["counts"], rep["lines"]))
            over = [(9.0, 0.0), (40.0, 20.0), (40.0, 20.0)]           # load 20 on 8 CPUs: the queue, not the tree
            rep = idle_rep(over, 9.5)
            check("report: an oversubscribed loop is out of range - named, never counted, no TR",
                  rep["data"]["tr"] == [] and any("out of range 2 (load > cpus" in ln for ln in rep["lines"]), rep["lines"])
            oor_tr = swap_("counted", lambda r, f, man, box=None: f is not None, lambda: idle_rep(over, 9.5))["data"]["tr"]
            plant_("an oversubscribed loop counted: red - it fires the TR line", oor_tr == ["BOXN"], oor_tr)
            rep = idle_rep([(50.0, None)] * 2, 9.5)
            check("report: a box with no load sample is named - the TR line cannot read it, never the raw in its place",
                  rep["data"]["tr"] == [] and "BOXN 50.00 s (idle n/a)" in rep["counts"]
                  and any("TR: BOXN - no complete loop with a load sample" in ln for ln in rep["lines"]), rep["lines"])
            (rlogs / "loop_times.jsonl").write_text(json.dumps(dict(own, box="BOXN", seconds=10.0, load=8.0, seconds_norm=5.0))
                                                    + "\n" + json.dumps(dict(own, box="BOXN", seconds=10.0, load=8.0,
                                                                             seconds_norm=None)) + "\n", encoding="utf-8")
            rep = cmd_report(rman, tmp / "rep", budget=9.5)
            check("report: a record's own `seconds_norm` is read; a null one is not counted",
                  "BOXN 10.00 s (5.00 s idle)" in rep["counts"] and rep["data"]["tr"] == [], rep["counts"])

            # -- J · the load sample -----------------------------------------------------------------------
            class NoLoadAvg:
                name = "nt"
            posix = type("P", (), {"getloadavg": staticmethod(lambda: (1.5, 0.9, 0.8))})
            broken = type("P", (), {"getloadavg": staticmethod(lambda: (_ for _ in ()).throw(OSError("unsupported")))})
            check("load: a POSIX box", sample_load(osmod=posix) == (1.5, "getloadavg1"))
            check("load: this box records something, or why not", (lambda g: (g[0] is None) == g[1].startswith("none: "))(sample_load()))
            check("load: a Windows box", sample_load(osmod=NoLoadAvg, win32=lambda: 4.25) == (4.25, f"GetSystemTimes/{WIN32_LOAD_WINDOW_S:g}s x ncpu"))
            check("load: a denied counter names why", (lambda g: g[0] is None and "denied" in g[1])(
                sample_load(osmod=NoLoadAvg, win32=lambda: (_ for _ in ()).throw(OSError(5, "access is denied")))))
            check("load: no sampler names why", sample_load(osmod=NoLoadAvg, win32=None)[1].startswith("none: no sampler"))
            check("load: a failing getloadavg names why", "unsupported" in sample_load(osmod=broken)[1])
            check("busy_cpus: the arithmetic", (busy_cpus(0, 100, 25, 200, 8), busy_cpus(0, 100, 100, 200, 4)) == (6.0, 0.0))
            try:
                busy_cpus(0, 100, 0, 100, 8)
                check("busy_cpus refuses a stopped clock", False)
            except OSError as exc:
                check("busy_cpus refuses a stopped clock", "did not advance" in str(exc))

            # -- J2 · a native crash leaves a stack: every case runs with faulthandler on ----------------------
            rc, out = run_({"f": {"cmd": f"{py} fault"}}, "f")
            check("a case runs with faulthandler on (PYTHONFAULTHANDLER from the harness, not the caller)",
                  rc == 0 and out.rstrip().endswith("=== GO ==="), out[-300:])
            real = globals()["case_env"]
            globals()["case_env"] = lambda task: dict(os.environ, PB_TASK=task, PYTHONIOENCODING="utf-8")
            try:
                rc, out = run_({"f": {"cmd": f"{py} fault"}}, "f")
            finally:
                globals()["case_env"] = real
            plant_("an env without faulthandler reads red", rc == 1 and "f:" in out, out[-300:])
            rc, out = run_({"p": {"cmd": f"{py} pbpy"}}, "p")
            check("a case gets PB_PYTHON, the harness's own interpreter (never the PATH's first python)",
                  rc == 0 and out.rstrip().endswith("=== GO ==="), out[-300:])
            globals()["case_env"] = lambda task: dict(os.environ, PB_TASK=task, PYTHONIOENCODING="utf-8",
                                                      PYTHONFAULTHANDLER="1")
            try:
                rc, out = run_({"p": {"cmd": f"{py} pbpy"}}, "p")
            finally:
                globals()["case_env"] = real
            plant_("an env without PB_PYTHON reads red", rc == 1 and "p:" in out, out[-300:])

            # -- K · the console, the list ------------------------------------------------------------------
            (tmp / "u.json").write_text(json.dumps({"scopes": {"a": {"cmd": "x", "note": "≤ 5 s"}}}), encoding="utf-8")
            env = dict(os.environ, PYTHONIOENCODING="cp1252")
            p = subprocess.run([sys.executable, str(Path(__file__).resolve()), "list", "--manifest", str(tmp / "u.json")],
                               capture_output=True, env=env, timeout=60)
            check("a cp1252 console still prints the list and its verdict", p.returncode == 0 and b"=== GO ===" in p.stdout,
                  p.stdout[-200:] + p.stderr[-200:])
            rc, out = run_(clean, "list")
            check("list: every scope and form, the counts line", rc == 0 and "placeholder until M9-T9" in out
                  and "cases from `cases_cmd`" in out and "box OTHERBOX" in out, out[-300:])

            # -- L · the steps numbered on the console (M0-D27) ------------------------------------------------
            three = {"s1": clean["a"], "s2": clean["a"], "s3": clean["alone"]}

            def numbered(task):                 # a three-step run: 1/3, 2/3, 3/3, none left at the end
                rc, out = run_(three, "s1", "s2", "s3", "-j", "2", task=task)
                return (rc == 0 and all(re.search(rf"^s{k} .* GO · step {k}/3, {3 - k} left$", out, re.M) for k in (1, 2, 3))
                        and out.rstrip().endswith("=== GO ===")), out[-500:]

            ok, out = numbered("TS1")
            check("a three-step run: each scope's line its step 1/3, 2/3, 3/3 (the exclusive last), none left at the end",
                  ok, out)
            check("... the counts line and the verdict as before: no number on them", ok and re.search(
                r"^verify: 3 passed, 0 failed, 0 skipped · 3/3 scopes GO · ", out, re.M) is not None
                and "step" not in out.splitlines()[-1] + out.splitlines()[-2], out)
            rc, out = run_({"a": clean["a"]}, "a", task="TS2")
            check("a one-step run's line carries no number", rc == 0 and " · step " not in out, out[-300:])
            ok, out = swap_("step_tag", lambda what, k, n: f"{what} {k}/{n}, {n - k + 1} left", lambda: numbered("TS3"))
            plant_("steps left counted off by one: red - the three-step run reads 1 left on its last", not ok, out)

            class Tty(io.StringIO):
                def isatty(self):
                    return True

            tty, PROGRESS = Tty(), True
            try:
                with contextlib.redirect_stderr(tty):
                    run_scopes({**base_top, "scopes": three}, list(three), tmp, "TS4", 2)
            finally:
                PROGRESS = False
            live = tty.getvalue().splitlines()
            eta = [ln for ln in live if ln.startswith(".. scope: ")]
            check("on a terminal each step is named as it starts, its number once, every line whole; the ETA "
                  "announced, then one line per step done, the last 3 done and none to do (M0-D65's lead)",
                  sorted(ln for ln in live if ln.startswith(">> "))
                  == [f">> s{k} · step {k}/3, {3 - k} left · started" for k in (1, 2, 3)]
                  and all(re.fullmatch(r">> s\d · step \d/3, \d left · started|\.\. s\d \d+\.\d\ds"
                                       r"|\.\. scope: 3 scopes to run, 2 at a time · ≈ \S+ by this box's past runs"
                                       r"( \(\d without one: the median\))?"
                                       r"|\.\. scope: \d done · (\d running · )?\d to do \(of 3\) · \S+ elapsed · ≈ \S+ left",
                                       ln) for ln in live)
                  and len(eta) == 4 and eta[-1].startswith(".. scope: 3 done · 0 to do (of 3) · "), live)
    except Exception as exc:                   # a crash is a failed check, never a traceback in place of the verdict
        failed.append(f"crashed after {len(checks)} checks: {type(exc).__name__}: {exc} "
                      f"({traceback.format_exc().strip().splitlines()[-3].strip()})")
    finally:
        PROGRESS = saved_progress
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
    counts = f"selftest: checks={len(checks)} · failed={len(failed)} · plants={plants[1]}/{plants[0]} red"
    nogo = ("zero checks ran" if not checks else f"{len(failed)} check(s) failed" if failed
            else f"{plants[0] - plants[1]} plant(s) stayed green" if plants[1] != plants[0] else None)
    return result(counts, failures=failed, nogo=nogo)


# ----------------------------------------------------------------------------------------------------- CLI
def emit(res, as_json):
    """The §4 Rule 4 ending; every printed line through `shown`."""
    reason = res["nogo"] or ("; ".join(res["failures"]) if res["failures"] else None)
    verdict = f"NO-GO: {reason}" if reason else "GO"
    if len(verdict) > 400:
        verdict = verdict[:397] + "..."
    if as_json:
        print(shown(json.dumps(dict(res, verdict=verdict), ensure_ascii=False, default=str)))
        return 1 if reason else 0
    for line in res["lines"]:
        print(shown(line))
    if res["failures"]:
        print("FAILED:")
        for f in res["failures"]:
            print("  " + shown(f))
    print(shown(res["counts"]))
    print(shown(f"=== {verdict} ==="))
    return 1 if reason else 0


class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise CallError(message)


def build_parser():
    ap = Parser(prog="verify.py", description="PLAYBOOK annex §A.2 - the one runner. The calls: "
                + " · ".join(CALLS) + ". Guide: docs/agent/testing.md")
    ap.add_argument("scopes", nargs="*", metavar="scope", help="scope name(s), or one of: list, report, selftest")
    ap.add_argument("--all", action="store_true", help="the complete loop: every scope, then the smoke")
    ap.add_argument("--case", help="one case of the one scope named, through its `case_cmd` or `cases` entry")
    ap.add_argument("--changed", action="store_true", help="every scope a changed path touches, from the coverage anchor")
    ap.add_argument("--base", help="--changed: also every scope a path moved since this named commit touches (never HEAD)")
    ap.add_argument("--force", action="store_true", help="a scope or --case run: run every scope named, even one already GO on this tree")
    ap.add_argument("--fresh", action="store_true", help="start: write the task's baseline again (default: keep the first)")
    ap.add_argument("--path", help="touched: the paths matching this glob, each with every task that touched it")
    ap.add_argument("--redarm", metavar="SCOPE", nargs="?", const=True,
                    help="the scope's plants, one scratch copy each, side by side: clean GO, each plant NO-GO")
    ap.add_argument("--moved", action="store_true",
                    help="with --redarm alone: every armed scope whose red-arm proof no longer holds, one campaign each")
    ap.add_argument("--task", help="names the logs; required for a run (else $PB_TASK)")
    ap.add_argument("-j", "--jobs", type=int, help="parallel commands (default: min(CPUs, jobs_cap))")
    ap.add_argument("--serial", action="store_true", help="one command at a time")
    ap.add_argument("--since", help="report: records from this date (YYYY-MM-DD)")
    ap.add_argument("--budget", type=float, help="report: the complete loop's budget in seconds - the TR line over it")
    ap.add_argument("--json", action="store_true", help="one JSON object instead of the lines, for agents")
    ap.add_argument("--manifest", help="default: tools/pb/verify.json")
    ap.add_argument("--root", help="the folder commands run from (default: the repository root)")
    ap.add_argument("--no-progress", dest="progress", action="store_false", help="no live progress on stderr")
    return ap


def wrong_call(exc):
    return result("verify: wrong call - nothing run", lines=["the calls:"] + [f"  {PROG} {c}" for c in CALLS],
                  nogo=f"{exc} - the calls are listed above")


def check_call(a):
    """The combinations that are no call; raises CallError."""
    cmd = a.scopes[0] if a.scopes[:1] and a.scopes[0] in COMMANDS else None
    run_flags = [f for f, on in (("--all", a.all), ("--case", a.case is not None), ("--changed", a.changed),
                                 ("--redarm", a.redarm)) if on]
    if cmd and (len(a.scopes) > 1 or run_flags):
        raise CallError(f"`{cmd}` takes no scope and no {', '.join(run_flags) or 'other name'}")
    if a.base and not a.changed:
        raise CallError("--base goes with --changed")
    if a.force and (cmd or a.all or a.changed or a.redarm or not (a.scopes or a.case is not None)):
        raise CallError("--force goes with a scope run: <scope>... --force, or <scope> --case <id> --force "
                        "(--all, --changed and --redarm always run)")
    if a.fresh and cmd != "start":
        raise CallError("--fresh goes with `start`")
    if a.path is not None and cmd != "touched":
        raise CallError("--path goes with `touched`")
    if cmd == "start" and (len(a.scopes) > 1 or a.base):
        raise CallError("`start` takes --task <ID> and --fresh, nothing else")
    if a.all and (a.scopes or a.case is not None or a.changed or a.redarm):
        raise CallError("--all takes no scope, no --case, no --changed, no --redarm")
    if a.changed and (a.scopes or a.case is not None or a.redarm):
        raise CallError("--changed takes no scope, no --case, no --redarm")
    if a.redarm and (a.scopes or a.case is not None):
        raise CallError("--redarm names its one scope itself: --redarm <scope>")
    if a.moved and a.redarm is not True:
        raise CallError("--moved goes with --redarm alone: --redarm --moved")
    if a.redarm is True and not a.moved:
        raise CallError("--redarm names its scope: --redarm <scope>, or --redarm --moved for the sweep")
    if not (cmd or run_flags or a.scopes):
        raise CallError("name a scope, or --all, --changed, --redarm, list, report or selftest")
    return cmd


def main(argv=None):
    global PROGRESS
    for stream in (sys.stdout, sys.stderr):      # a cp1252 pipe must not eat the verdict
        with contextlib.suppress(AttributeError, ValueError, OSError, io.UnsupportedOperation):
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    argv = sys.argv[1:] if argv is None else list(argv)
    as_json = "--json" in argv
    try:
        a = build_parser().parse_args(argv)
        cmd = check_call(a)
    except CallError as exc:
        return emit(wrong_call(exc), as_json)
    PROGRESS = PROGRESS and a.progress
    what = cmd or ("redarm" if a.redarm else "changed" if a.changed else "all" if a.all else "verify")
    try:
        if cmd == "selftest":
            res = selftest()
        else:
            root = Path(a.root).resolve() if a.root else ROOT
            manifest = Path(a.manifest).resolve() if a.manifest else MANIFEST
            man = load_manifest(manifest)
            res = (cmd_list(man) if cmd == "list" else cmd_report(man, root, a.since, a.budget) if cmd == "report"
                   else cmd_start(man, a, root, manifest) if cmd == "start"
                   else cmd_touched(man, root, a.task, a.path) if cmd == "touched" else cmd_run(man, a, root, manifest))
    except CallError as exc:
        res = wrong_call(exc)
    except Refused as exc:
        res = result(f"{what}: refused - nothing run", failures=exc.lines, nogo=str(exc))
    except Exception as exc:                     # a crash still ends in a verdict; the trace goes to stderr
        print(shown(traceback.format_exc()), end="", file=sys.stderr)
        res = result(f"{what}: crashed", nogo=f"{type(exc).__name__}: {exc}")
    return emit(res, as_json)


if __name__ == "__main__":
    sys.exit(main())
