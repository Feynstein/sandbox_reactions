#!/usr/bin/env python3
"""tools/pb/verify.py - the one runner, the routine's instrument (PLAYBOOK annex §A.2).

  <scope>... --task <ID>                   those scopes
  --all --task <ID>                        the complete loop: every scope, then the smoke
  <scope> --case <id> --task <ID>          one case: PARTIAL, never the loop
  --changed [--base <commit>] --task <ID>  every scope a changed path touches, read from the coverage anchor
  --redarm <scope> --task <ID>             one scratch copy per plant: the clean copy GO, each plant NO-GO
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
run (read-only git, always with --no-optional-locks), and no process this run did not start is signalled.
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
COMMANDS = ("list", "report", "selftest")
CALLS = ("<scope>... --task <ID>", "--all --task <ID>", "<scope> --case <id> --task <ID>",
         "--changed [--base <commit>] --task <ID>", "--redarm <scope> --task <ID>", "list",
         "report [--since YYYY-MM-DD] [--budget <s>]", "selftest")
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
              "expected", "exclusive", "timeout_s", "cwd", "paths", "box", "plants", "free_ports", "good"}
NOTE_KEYS = ("note", "desc", "why", "hint", "reason")    # and any `_` key: a note - read, never acted on
SPEC_KEYS = ("cmd", "cases", "cases_cmd", "case_cmd", "parse", "regex", "count", "expected", "cwd", "good")
PLANT_KEYS = {"id", "patch"}
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
               "went red loses its entries. `verify.py --changed` reads it.")
PROGRESS = True                                          # live stderr progress on a terminal, display only
WIN32_LOAD_WINDOW_S = 0.25                               # the GetSystemTimes window, never inside `seconds`
LOCK = threading.Lock()
PART = [0]                                               # part-file counter (under LOCK)
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
    if "paths" in s and not str_list(s["paths"]):
        out.append("`paths` is a list of globs")
    if "box" in s and not (isinstance(s["box"], str) and s["box"].strip()):
        out.append("`box` names the one box that can run it")
    if not all(isinstance(p, int) and not isinstance(p, bool) for p in s.get("free_ports", [])):
        out.append("`free_ports` is a list of port numbers")
    plants, seen = s.get("plants", []), set()
    if not isinstance(plants, list):
        out.append("`plants` is a list of {id, patch}")
        plants = []
    for p in plants:
        if not isinstance(p, dict) or {k for k in p if not is_note(k)} - PLANT_KEYS or not all(
                isinstance(p.get(k), str) and p.get(k) for k in ("id", "patch")):
            out.append(f"a plant is {{\"id\": \"<plant>\", \"patch\": \"<patch file or command>\"}}: {p!r}")
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


def owed_on(sc, box):
    return bool(sc.get("box")) and sc["box"].strip().lower() != box.strip().lower()


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
    exe = toks[0] if os.path.dirname(toks[0]) else shutil.which(toks[0])
    if not exe:
        raise ValueError(f"{toks[0]!r} is not on PATH")
    return [exe] + toks[1:]


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

    def ran(self):
        return not (self.skip or self.owed or self.notrun)

    def red(self):
        """The NO-GO reasons, [] when GO - a scope that did not run (placeholder, owed, NOT RUN) is never red
        itself, and never GO: the loop reads NO-GO while any scope is NOT RUN (run_scopes)."""
        if not self.ran():
            return []
        out = list(self.reasons)
        if self.passed == 0 and not out:
            out.append("zero passed")
        if self.passed + self.failed < self.expected:
            out.append(f"{self.passed + self.failed} case(s) < expected {self.expected}")
        return out

    def record(self):
        if self.skip:
            return {"verdict": "SKIP", "reason": self.skip}
        if self.owed:
            return {"verdict": "OWED", "box": self.owed}
        if self.notrun:
            return {"verdict": "NOT RUN", "reason": self.notrun}
        return {"passed": self.passed, "failed": self.failed, "skipped": self.skipped,
                "seconds": round(self.seconds, 3), "verdict": "NO-GO" if self.red() else "GO"}

    def line(self, w, box):
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
    if PROGRESS and sys.stderr.isatty():
        print(f".. {label} {secs:.2f}s", file=sys.stderr, flush=True)


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


def tree_id(root, man, logs, git=git_out):
    """`<HEAD 12>+<sha1 12>` over `git status --porcelain -uall`'s entries, the content of each file it
    lists, and every file under `tree_extra` - content hashed CRLF-as-LF, so one commit's two checkouts
    name ONE tree, and a second edit to an already-dirty file moves the id. Held out: the logs folder and
    the anchor (every run rewrites them). None outside a work tree, or with `tree_extra` unreadable: two
    unknown trees are not one tree."""
    extra = extra_files(root, man, logs)
    if extra is None:
        return None
    head, st = git(root, ["rev-parse", "HEAD"], 30), git(root, ["status", "--porcelain", "-z", "-uall"])
    if head is None or st is None:
        return None
    held, digest, items, i = held_out(root, man, logs), hashlib.sha1(), st.split("\0"), 0
    while i < len(items):
        e = items[i]
        i += 1 + (e[:1] in ("R", "C"))                   # a rename's origin follows in its own field
        path = e[3:]
        if len(e) < 4 or is_held(path, held):
            continue
        digest.update(e.encode("utf-8", "surrogateescape") + b"\0")
        with contextlib.suppress(OSError):
            if (Path(root) / path).is_file():
                digest.update(content_sha1((Path(root) / path).read_bytes()).encode("ascii"))
    for p in extra:
        if not is_held(p, held):
            digest.update(p.encode("utf-8", "surrogateescape") + b"\0" + (sha1_of(root, p) or "?").encode())
    return head.strip()[:12] + "+" + digest.hexdigest()[:12]


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


def visible_files(root, man, logs, git=git_out):
    """The anchor's subject: every file git can see (tracked, and untracked but not ignored - a new file is
    part of the tree the moment it exists) plus every file under `tree_extra`, the held-out two excepted.
    None when git cannot answer, or `tree_extra` cannot be read: a gate that cannot name its subject
    certifies nothing."""
    out, extra = git(root, ["ls-files", "-co", "--exclude-standard", "-z"]), extra_files(root, man, logs)
    if out is None or extra is None:
        return None
    held = held_out(root, man, logs)
    paths = {p for p in out.split("\0") if p} | set(extra)
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
    text = ('{\n  "version": ' + json.dumps(ANCHOR_VERSION) + ',\n  "note": ' + json.dumps(ANCHOR_NOTE)
            + ',\n  "paths": {\n' + rows + ("\n" if rows else "") + "  }\n}\n")
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


# ------------------------------------------------------- --redarm: one scratch copy per plant, each NO-GO
def scratch_copy(src, dest):
    """Annex §A.7's copy, in the standard library: the tree without the folders it names (by name, at any
    depth), `images/` folders kept empty, `.venv` and `node_modules` linked back when they exist."""
    src, dest = Path(src), Path(dest)

    def ignore(d, names):
        return set(names) if os.path.basename(d) in EMPTIED else {n for n in names if n in EXCLUDES}

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


def redarm(man, name, root, task, jobs, box=None, logs=None, record=True, tracked_fn=None):
    """The red-arm campaign of one scope: a fresh scratch copy per arm - the clean copy must go GO (a red
    with no plant proves nothing), then each declared plant, planted in its own copy, must go NO-GO; a
    timeout or a command that did not start is an error, never a red. The tree itself is never written
    but for the campaign's log."""
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
        raise Refused(f"`{name}` runs on {s['box']} only (this box: {box}) - red-arm it there; owed on {s['box']}")
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
    env = dict(os.environ, PB_TASK=task)
    env.setdefault("PYTHONIOENCODING", "utf-8")
    lines, fails, red, green, errors, t0 = [], [], 0, 0, 0, time.monotonic()
    tmp = Path(tempfile.mkdtemp(prefix="pb-redarm-"))
    arms = {}
    try:
        for i, arm in enumerate([None] + plants):
            label = "clean" if arm is None else f"plant {arm['id']}"
            copy = tmp / f"arm{i}"
            scratch_copy(root, copy)
            with LOCK, open(log, "a", encoding="utf-8") as fh:
                fh.write(f"==== {stamp()} {name} · {label} · a fresh scratch copy of the tree\n")
            if arm is not None:
                why = plant(arm, root, copy, one, env, log)
                if why:
                    errors += 1
                    arms[arm["id"]] = "ERROR"
                    lines.append(f"{name} · {label} · ERROR ({why})")
                    fails.append(f"{label}: {why} - an error, not a red")
                    continue
            r = run_scopes(one, [name], copy, task, jobs, mode="redarm", box=box, logs=logs, log_for=lambda n: log,
                           anchor=False, record=False, tracked_fn=lambda _r, _p: [])
            row = r["data"]["rows"][0]
            if arm is None:
                lines.append(f"{name} · clean · {row['verdict']}")
                if row["verdict"] != "GO":
                    fails.append(f"clean: {'; '.join(row['reasons'])} - red with no plant, so a red plant would prove "
                                 f"nothing; the plants did not run")
                    break
                continue
            honest = row["verdict"] == "NO-GO" and not any(
                k in x for x in row["reasons"] for k in ("timed out", "not started", "NOT RUN ("))
            arms[arm["id"]] = "RED" if honest else "GREEN" if row["verdict"] == "GO" else "ERROR"
            if honest:
                red += 1
                lines.append(f"{name} · {label} · RED (proven: {'; '.join(row['reasons'])[:120]})")
            elif row["verdict"] == "GO":
                green += 1
                lines.append(f"{name} · {label} · GREEN (unfailable)")
                fails.append(f"{label}: stayed GO - the scope cannot see this bug")
            else:
                errors += 1
                lines.append(f"{name} · {label} · ERROR ({'; '.join(row['reasons'])})")
                fails.append(f"{label}: {'; '.join(row['reasons'])} - an error, not a red")
    finally:
        shutil.rmtree(str(tmp), ignore_errors=True)               # this campaign's own copies
    secs = round(time.monotonic() - t0, 2)
    if record:
        rec = {"ts": stamp(), "task": task, "mode": "redarm", "seconds": secs, "tree": tree_id(root, man, logs),
               "verdict": "NO-GO" if fails or not red else "GO", "box": box, "python": PYTHON,
               "scopes": {name: {"plants": arms, "verdict": "NO-GO" if fails or not red else "GO"}}}
        with LOCK, open(logs / "loop_times.jsonl", "a", encoding="utf-8") as fh:
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    counts = (f"redarm: {len(plants)} plants · {red} red · {green} green · {errors} errors · scope {name} · {secs:.2f} s · "
              f"box {box} · log {rel(log, root)}")
    return result(counts, failures=fails, lines=lines,
                  nogo=None if not fails and red else ("zero plants red" if not fails else None),
                  red=red, green=green, errors=errors)


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
    if not isinstance(olds, dict):
        return None
    canon = lambda s: json.dumps({k: v for k, v in s.items() if not is_note(k)} if isinstance(s, dict) else s,  # noqa: E731
                                 sort_keys=True)
    return {n: ("added" if n not in olds else "changed") for n, s in man["scopes"].items()
            if n not in olds or not isinstance(olds[n], dict) or canon(s) != canon(olds[n])}


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
        return ["flags: git could not list the changed paths - no flag read"]
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
                mchanges=False):
    """What the loop adds to its scopes (PLAYBOOK §8, §A.2): the plants of every scope added or changed
    since the base (HEAD unless --base names one) run, and must each go NO-GO; the flags. -> (lines, failures)."""
    rev, lines, fails = base_sha or "HEAD", [], []
    manifest_rel = rel(manifest, root)
    block = find_block(root, task) if block is False else block
    changed = changed_paths(root, rev, git) if changed is False else changed
    mchanges = manifest_changes(man, root, manifest, rev, git) if mchanges is False else mchanges
    if mchanges is None:
        lines.append(f"plants: git cannot show {manifest_rel} at {rev[:12]} - the scopes added or changed are unknown")
    bl = blocked(man)
    for n, how in sorted((mchanges or {}).items()):
        s = man["scopes"][n]
        if n in bl or "placeholder" in s or owed_on(s, box):          # a NOT RUN scope is named on its own line
            continue
        if not s.get("plants"):
            lines.append(f"FLAG plants: scope `{n}` {how} since {rev[:12]} and declares no plant - a check it "
                         f"cannot fail is not a check (§8)")
            continue
        r = redarm(man, n, root, task, jobs, box=box, record=False)
        lines += r["lines"] + [r["counts"]]
        fails += [f"redarm {n}: {x}" for x in (r["failures"] or ([r["nogo"]] if r["nogo"] else []))]
    return lines + loop_flags(block, changed, mchanges, man, manifest_rel, task, rev[:12]), fails


# ----------------------------------------------------------------------------------------------------- a run
def run_scopes(man, names, root, task, jobs, case=None, mode="scope", box=None, logs=None, log_for=None,
               anchor=True, gate=False, visible=None, not_run=(), why=None, hook=None, record=True,
               tracked_fn=None, extra=None):
    """Run these scopes: the plain ones in parallel (jobs), the exclusive ones alone after them, the smoke last
    on --all; one record appended; the anchor advanced by the green scopes; with `gate`, the anchor graded."""
    box, root = box or this_box(), Path(root)
    logs = Path(logs) if logs else logs_of(man, root)
    log_for = log_for or (lambda n: logs / f"{task}.{n}.log")
    logs.mkdir(parents=True, exist_ok=True)
    bl, parts = blocked(man), top_errors(man)
    specs = [(n, man["scopes"][n], bl.get(n)) for n in names]         # (name, spec, why it is NOT RUN or None)
    if mode == "all" and (man.get("smoke") or "smoke" in parts):
        specs.append(("smoke", {"cmd": man["smoke"], "exclusive": True}, parts.get("smoke")))
    refused = refuse_log(root, [log_for(n) for n, s, no in specs if not no and "placeholder" not in s
                                and not owed_on(s, box)], tracked_fn)
    if refused:                                  # nothing opened, nothing spawned, no record: the run did not happen
        raise Refused(refused)
    load_start, _src = sample_load()
    apath = anchor_path(man, root)
    use_anchor = (anchor and case is None and apath is not None and not anchor_off(man)
                  and any(s.get("paths") for _n, s, no in specs if not no))
    if (use_anchor or gate) and visible is None:
        visible = visible_files(root, man, logs)
        if visible is None and gate:
            raise Refused("git could not list the tree's files - the anchor cannot name its subject")
    snap = (anchor_snapshot(man, root, [n for n, _s, no in specs if not no], visible)
            if use_anchor and visible is not None else {})
    tree = tree_id(root, man, logs) if record else None
    env = dict(os.environ, PB_TASK=task)
    env.setdefault("PYTHONIOENCODING", "utf-8")
    t0, runs, todo = time.monotonic(), [], []
    for name, s, no in specs:
        sr = ScopeRun(name, s, 0 if no else 1 if case is not None or name == "smoke" else int(s.get("expected", 0)),
                      log_for(name))
        runs.append(sr)
        if no:
            sr.notrun = no                                            # never run without the setting it holds
        elif "placeholder" in s:
            sr.skip = f"placeholder until {s['placeholder']}"
        elif owed_on(s, box):
            sr.owed = s["box"]
        else:
            todo += [(sr,) + u for u in units(sr, case, root, man, env, task, box)]

    def one(u):
        sr, label, cmd, value = u
        with LOCK:
            sr.started = sr.started or time.time()
        run_unit(sr, label, cmd, value, root, man, env, task, box)
        with LOCK:
            sr.ended = time.time()

    with ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
        list(pool.map(one, [u for u in todo if not u[0].spec.get("exclusive")]))
    for u in [u for u in todo if u[0].spec.get("exclusive")]:        # alone, after the rest, in declared order
        one(u)
    elapsed = round(time.monotonic() - t0, 3)          # taken before the end sample: sampling never bills the run
    load, load_source = sample_load()
    ran = [sr for sr in runs if sr.ran()]
    reds = [sr for sr in ran if sr.red()]
    rec = {"ts": stamp(), "task": task, "mode": mode, "seconds": elapsed, "verdict": None, "tree": tree, "box": box,
           "python": PYTHON, "os": platform.system(), "load_start": load_start, "load": load,
           "load_source": load_source, "cpus": os.cpu_count(), "cases": sum(sr.passed + sr.failed for sr in ran)}
    rec.update({k: v for k, v in (extra or {}).items() if v is not None})
    if case is not None:
        rec["case"] = case
    moved = anchor_advance(man, root, {sr.name for sr in ran if not sr.red() and sr.name != "smoke"},
                           {sr.name for sr in reds if sr.name != "smoke"}, snap, rec, apath, visible) if snap else 0
    unrun = [sr for sr in runs if sr.notrun]
    lines = manifest_lines(man) + [f"why {n}: {w}" for n, w in (why or {}).items()]
    w = max([len(sr.name) for sr in runs] + [len(n) for n, _g in not_run] + [8]) + 2
    lines += [sr.line(w, box) for sr in runs] + [f"{n:<{w}}NOT RUN ({g})" for n, g in not_run]
    failures = [f"{sr.name}: {'; '.join(sr.red())} · log {rel(sr.log, root)}" for sr in reds]
    if mode in ("all", "changed"):              # a scoped run is judged on the scopes it ran; the loop is not complete
        failures += [f"{sr.name}: NOT RUN ({sr.notrun}) - " + ("the complete loop is not complete" if mode == "all"
                                                                else "the gate cannot vouch for a scope it cannot run")
                     for sr in unrun]
    if hook:
        hook_lines, hook_fails = hook()
        lines += hook_lines
        failures += hook_fails
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
    passed, failed, skipped = (sum(getattr(sr, k) for sr in ran) for k in ("passed", "failed", "skipped"))
    if passed == 0 and not failures:
        failures.append("zero passed in total" + (" - every scope named is NOT RUN, each named above"
                                                  if unrun and not ran else ""))
    rec["verdict"] = "NO-GO" if failures else "GO"
    rec["scopes"] = {sr.name: sr.record() for sr in runs}
    rec["scopes"].update({n: {"verdict": "NOT RUN", "reason": g} for n, g in not_run})
    if record:
        with LOCK, open(logs / "loop_times.jsonl", "a", encoding="utf-8") as fh:   # append-only: one line
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    lines.append(f"logs: {rel(logs, root)}/{task}.<scope>.log · record: loop_times.jsonl · tree {tree}")
    lines += tail
    partial = f" · PARTIAL, not the loop (--case {case})" if case is not None else ""
    n_go = sum(1 for sr in ran if not sr.red())
    counts = (f"verify: {passed} passed, {failed} failed, {skipped} skipped · {n_go}/{len(ran)} scopes GO · "
              f"{sum(1 for sr in runs if sr.skip)} skipped · {sum(1 for sr in runs if sr.owed)} owed · "
              f"{len(unrun)} NOT RUN · {len(not_run)} not impacted · {elapsed:.2f} s · mode {mode}{partial} · "
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


def changed_run(man, root, task, jobs, base=None, box=None, git=git_out, visible=None, hook=None):
    """--changed: every scope a changed path touches, chosen from the coverage anchor; then the gate - a path
    no scope has checked, or no scope covers, is NO-GO."""
    box, root, bl = box or this_box(), Path(root), blocked(man)
    off = anchor_off(man)
    if off:
        raise Refused(f"--changed: the coverage anchor is NOT RUN ({off}) - fix it, or run --all")
    if not any(s.get("paths") for n, s in man["scopes"].items() if n not in bl and "placeholder" not in s):
        raise Refused("--changed: no scope declares `paths`, so there is no coverage anchor to read - declare each "
                      "scope's paths and skip classes (docs/agent/testing.md), seed the anchor with one --all, "
                      "or run --all")
    logs = logs_of(man, root)
    visible = visible_files(root, man, logs, git) if visible is None else visible
    if visible is None:
        raise Refused("git could not list the tree's files - --changed cannot name its subject, so it certifies nothing")
    sha = resolve_base(root, base, git) if base is not None else None
    moved = moved_since(root, sha, git) if sha else set()
    st = anchor_status(man, root, anchor_read(anchor_path(man, root)), visible, box)
    chosen, why, not_run = choose(man, st, moved, box, base)
    if not [n for n in chosen if n in bl or ("placeholder" not in man["scopes"][n] and not owed_on(man["scopes"][n], box))]:
        return result(f"verify: 0 passed, 0 failed, 0 skipped · 0 scopes chosen · {len(not_run)} not impacted · "
                      f"mode changed · box {box}",
                      lines=[f"{n}  NOT RUN ({g})" for n, g in not_run],
                      nogo="nothing changed since every scope's last GO: no scope to run, zero cases - the anchor "
                           "already vouches for this tree (a loop on it is waste)")
    return run_scopes(man, chosen, root, task, jobs, mode="changed", box=box, gate=True, visible=visible,
                      not_run=not_run, why=why, hook=hook, extra={"base": sha})


def cmd_run(man, a, root, manifest):
    task = task_of(a)
    cap = 1 if "jobs_cap" in top_errors(man) else int(man.get("jobs_cap", 8))    # a bad cap: one command at a time
    jobs = 1 if a.serial else (a.jobs or min(os.cpu_count() or 1, cap))
    if a.redarm:
        return redarm(man, a.redarm, root, task, jobs)
    if a.changed:
        sha = resolve_base(root, a.base) if a.base else None
        hook = lambda: loop_checks(man, root, task, jobs, this_box(), manifest, sha)  # noqa: E731
        return changed_run(man, root, task, jobs, base=a.base, hook=hook)
    names = list(man["scopes"])
    if a.all:
        hook = lambda: loop_checks(man, root, task, jobs, this_box(), manifest)  # noqa: E731
        return run_scopes(man, names, root, task, jobs, mode="all", hook=hook)
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
        return run_scopes(man, chosen, root, task, jobs, case=a.case, mode="case")
    unknown = [n for n in a.scopes if n not in man["scopes"]]
    if unknown:
        raise Refused(f"unknown scope(s): {', '.join(unknown)} - `list` shows them ({', '.join(names)})")
    return run_scopes(man, list(dict.fromkeys(a.scopes)), root, task, jobs, mode="scope")


# --------------------------------------------------------------------------------------------- list · report
def cmd_list(man):
    bl, lines = blocked(man), manifest_lines(man)
    for name, s in man["scopes"].items():
        if name in bl:
            lines.append(f"{name:<18}NOT RUN ({bl[name]})")
            continue
        form = (f"placeholder until {s['placeholder']}" if "placeholder" in s else "cases from `cases_cmd`"
                if "cases_cmd" in s else f"{len(s['cases'])} case(s)" if "cases" in s else "1 command")
        more = [f for f, on in ((f"paths {len(s.get('paths', []))}", s.get("paths")),
                                (f"plants {len(s.get('plants', []))}", s.get("plants")),
                                (f"box {s.get('box')}", s.get("box")), ("exclusive", s.get("exclusive")),
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


def budget_lines(man, box, rows):
    """The manifest's per-box budget, applied per loop - a function of the case count and the load, never a
    constant. A loop with no load sample is graded at load 0 and says so."""
    bud = man.get("budget") or {}
    bb = (bud.get("boxes") or {}).get(box or "", {}) or {}
    if not bb.get("base_s"):
        return [f"{box or '?':<8} {len(rows)} loops · no budget measured for this box in the manifest - measure it "
                f"with {bud.get('measured_by', 'a TR block')}"]
    per, k = float(bb.get("per_case_s") or 0.0), float(bb.get("load_k") or 0.0)
    band, ref, base = float(bb.get("band_pct") or 0) / 100.0, int(bb["at_cases"]), float(bb["base_s"])
    scope = bud.get("cases_scope")
    out = [f"{box or '?':<8} {len(rows)} loops · budget E = ({base:g} s + {per:g} s/case x (cases - {ref}))"
           f" x (1 + {k:g} x load) · band +/-{band * 100:.0f} % · stated for load <= cpus"
           + (f" · {bb['measured']}" if bb.get("measured") else ""),
           f"{'':<8} {'task':<16}{'obs':>8}{'E':>8}{'delta':>8}{'load':>7}{'cases':>7}"]
    over, blind, oor = [], [], []
    for r in rows:
        cases, load = loop_cases(r, scope), r.get("load")
        cpus = r.get("cpus") or bb.get("cpus")
        obs, mark = float(r.get("seconds", 0)), ""
        exp = (base + per * (cases - ref)) * (1 + k * (load or 0.0))
        if len(r.get("scopes") or {}) != len(man.get("scopes") or {}):
            mark = (f"  SCOPE SET DIFFERS ({len(r.get('scopes') or {})} of {len(man.get('scopes') or {})}): the budget "
                    "is stated against the manifest as it stands, so this loop is not graded")
        elif load is None:
            blind.append(r.get("task"))
            mark = "  NO LOAD SAMPLE: graded at load 0, so E is a floor"
        elif cpus and load > cpus:
            oor.append(f"{r.get('task')} (load {load} > {cpus} cpus)")
            mark = "  OUT OF RANGE: the box is oversubscribed, so this run does not measure the tree"
        elif abs(obs - exp) > band * exp:
            over.append(f"{r.get('task')} {(obs - exp) / exp * 100:+.0f} %")
            mark = "  OVER BAND"
        out.append(f"{'':<8} {str(r.get('task')):<16}{obs:>8.1f}{exp:>8.1f}{(obs - exp) / exp * 100:>+7.1f}%"
                   f"{(load if load is not None else float('nan')):>7.2f}{cases:>7}{mark}")
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
    return {k: v for k, v in r.items() if not (k in ("box", "type", "model") and not isinstance(v, str)
                                               or k in ("load", "cpus", "cases") and not is_num(v))}


def entry_num(s, key):
    """A scope entry's number, or None when the entry is not in this runner's shape (an older runner's list)."""
    v = s.get(key) if isinstance(s, dict) else None
    return v if is_num(v) else None


def cmd_report(man, root, since=None, budget=None):
    """From loop_times.jsonl: loops per task, redundant loops, first-try passes, the complete loop's median and
    slowest scopes per box, the budget, and the TR line only over --budget. A record in another runner's shape is
    `foreign`: counted apart, the rest read - never a traceback, never a number read from a shape it cannot trust."""
    path, recs, foreign = logs_of(man, root) / "loop_times.jsonl", [], 0
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
                continue
            if not since or r["ts"][:10] >= since:
                recs.append(r)
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
        per_scope = {}
        for r in rows:
            for name, s in (r.get("scopes") or {}).items():
                if entry_num(s, "seconds") is not None:              # an entry not in this runner's shape is not read
                    per_scope.setdefault(name, []).append(float(s["seconds"]))
        med = statistics.median(float(r.get("seconds", 0)) for r in rows)
        medians.append(f"{box} {med:.2f} s")
        slow = sorted(((statistics.median(v), k) for k, v in per_scope.items()), reverse=True)
        if slow:
            lines.append(f"{box}: slowest scopes (median over {len(rows)} complete loops): "
                         + " · ".join(f"{k} {m:.2f} s" for m, k in slow[:5]))
        if man.get("budget") and "budget" not in top_errors(man):      # a bad budget: its NOT RUN line above
            lines += budget_lines(man, box, rows)
        if budget is not None and float(rows[-1].get("seconds", 0)) > budget:
            trs.append(box)
            lines.append(f"TR: the last complete loop on {box} took {float(rows[-1]['seconds']):.2f} s - over the "
                         f"--budget {budget:g} s: a TR fires (§4 Rule 4)")
    n_loop = sum(r.get("mode") in ("all", "changed") for r in recs)
    counts = (f"report: {len(recs)} runs · {foreign} foreign · {len(loops)} complete loops · "
              f"{sum(r.get('mode') == 'changed' for r in recs)} changed · {len(by_task)} task label(s) · loops per "
              f"task {n_loop / (len(looped) or 1):.2f} · redundant {len(redundant)} · first-try "
              f"{sum(first.values())}/{len(first)} · median complete loop {' · '.join(medians) or 'n/a'}"
              + (f" · TR {', '.join(trs) or 'none'} (budget {budget:g} s)" if budget is not None else ""))
    return result(counts, lines=lines, redundant=len(redundant), loops=len(loops), first_try=sum(first.values()),
                  tasks=len(first), tr=trs, foreign=foreign)


# ---------------------------------------------------------------------------------------------- selftest
EMIT = r'''import os, sys, time
t0, kind, args = time.time(), sys.argv[1], sys.argv[2:]
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
    kind = "nogo" if "BUG" in open(args[0], encoding="utf-8").read() else "go"
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
    with open(trace, "a") as f:
        f.write("%s %.6f %.6f\n" % (args[0] if kind == "trace" else kind, t0, time.time()))
sys.exit(rc)
'''


def selftest():
    """A temp root and a fake command per scope: GO clean; every planted violation must redden on its OWN
    reason. Never touches the real logs, the real anchor or the tree it sits in."""
    global PROGRESS
    checks, failed, plants = [], [], [0, 0]           # plants: [planted, red]
    saved = {k: os.environ.get(k) for k in ("PB_TASK", "PB_TRACE", "PB_BOX", "GIT_CEILING_DIRECTORIES")}
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

    try:
        with tempfile.TemporaryDirectory(prefix="pb-verify-") as tmpd:
            tmp = Path(tmpd)
            os.environ.pop("PB_TASK", None)
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

            def spans():
                return [(s.split()[0], float(s.split()[1]), float(s.split()[2]))
                        for s in trace.read_text(encoding="utf-8").splitlines()]

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
                   "aside, the scope ran GO on its one case", re.search(r"^noted +1/0/0 .*GO$", out, re.M) is not None, out[-600:])
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
            for p, b in (("tools/pb/plan.py", b"P"), ("tools/pb/verify.anchor.json", b"{}"), ("logs/r.log", b"L"),
                         ("extra/notes/n.md", b"N"), ("extra/logs/x.log", b"X")):
                (troot / p).write_bytes(b)
            tman = {"logs_dir": "logs", "tree_extra": ["extra"], "scopes": {}}
            tporc = "\0".join([" M tools/pb/plan.py", "?? tools/pb/verify.anchor.json", "?? logs/r.log", ""])
            tgit = lambda r, args, timeout=120: "0123456789abcdef\n" if args[0] == "rev-parse" else tporc  # noqa: E731
            tid = lambda man=tman, g=tgit: tree_id(troot, man, troot / "logs", g)  # noqa: E731
            t0_ = tid()
            (troot / "tools/pb/verify.anchor.json").write_bytes(b'{"paths": {"advanced": 1}}')
            (troot / "logs/r.log").write_bytes(b"L and this run's own log")
            (troot / "extra/logs/x.log").write_bytes(b"X moved")
            check("the tree id holds out the anchor and every logs folder", t0_ is not None and tid() == t0_)
            check("the tree id reads the anchor's path from the manifest", tid(dict(tman, anchor="tools/pb/elsewhere.json")) != t0_)
            (troot / "tools/pb/plan.py").write_bytes(b"P edited")
            t1_ = tid()
            check("the tree id sees a byte of a dirty file", t1_ != t0_)
            (troot / "extra/notes/n.md").write_bytes(b"N edited")
            check("the tree id sees a byte under tree_extra", tid() != t1_)
            check("the tree id is None when git cannot answer", tid(g=lambda r, args, timeout=120: None) is None)
            (troot / "tools/pb/plan.py").write_bytes(text_lf)
            t_lf = tid()
            (troot / "tools/pb/plan.py").write_bytes(crlf(text_lf))
            check("one commit's two checkouts name one tree", t_lf is not None and tid() == t_lf)

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
            scratch_copy(proot, dest)
            check("the scratch copy leaves out models, data, .git at any depth; keeps images/ empty; links .venv back",
                  (dest / "data.txt").is_file() and not (dest / "models").exists() and not (dest / "sub/data").exists()
                  and (dest / "images").is_dir() and not list((dest / "images").iterdir())
                  and (os.name == "nt" or (dest / ".venv").is_symlink()))
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
            bman = {"scopes": {"u": {"cmd": "python3 tests/run.py --fast"},
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
                 "scopes": {"a": {"passed": 1, "failed": 0, "skipped": 0, "seconds": s}}}) + "\n"
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
                   "seconds": 5, "scopes": {"a": {"passed": 1, "failed": 0, "skipped": 0, "seconds": 5}}}
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
    ap.add_argument("--redarm", metavar="SCOPE", help="the scope's plants, one scratch copy each: clean GO, each plant NO-GO")
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
    if a.all and (a.scopes or a.case is not None or a.changed or a.redarm):
        raise CallError("--all takes no scope, no --case, no --changed, no --redarm")
    if a.changed and (a.scopes or a.case is not None or a.redarm):
        raise CallError("--changed takes no scope, no --case, no --redarm")
    if a.redarm and (a.scopes or a.case is not None):
        raise CallError("--redarm names its one scope itself: --redarm <scope>")
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
                   else cmd_run(man, a, root, manifest))
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
