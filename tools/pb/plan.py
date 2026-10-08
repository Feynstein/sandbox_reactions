#!/usr/bin/env python3
"""tools/pb/plan.py - every edit an agent makes to the plan, one call each.

PLAYBOOK annex §A.0 (conventions) + §A.1 (this tool).  Python 3.9+, standard library only.
UTF-8 in and out, on any console; line endings preserved; atomic writes, the replace retried
while another process still holds the file; a lock around every read-modify-write; nothing
deleted; git never run.  Every call ends in the §4 Rule 4 shape: one counts line,
failures/warnings only when non-zero, `=== GO ===` / `=== NO-GO: <reason> ===` derived from the
counts, exit 0 / 1, `--json` behind a flag.  A refusal writes nothing and says so; a wrong call
prints the right one.

The calls: show · status · close · handoff · flag · append · move · stub · register · lint ·
header · selftest.  The ones a task makes sit in the plan's Repo facts (PLAYBOOK §14.1).
"""
from __future__ import annotations

import argparse
import contextlib
import datetime
import glob
import io
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time

try:                      # POSIX advisory locks
    import fcntl
except ImportError:       # pragma: no cover - Windows has none
    fcntl = None
try:                      # Windows byte-range locks
    import msvcrt
except ImportError:       # pragma: no cover - POSIX has none
    msvcrt = None

PROG = "python3 tools/pb/plan.py"
STATES = ("N/A", "IN PROGRESS", "TODO", "DONE", "BLOCKED", "DEFERRED")
STUBBABLE = ("DONE", "N/A", "DEFERRED")
OPEN_STATES = ("TODO", "IN PROGRESS", "BLOCKED", "DEFERRED")
FINISHED = ("DONE", "N/A")      # a finished block is history to the lint: read, counted, never a failure
# A state word in the plan's own language (the fleet's other one is French): read, never written -
# `status` and `append` take §3.3's words only.
LANG_STATES = ((r"[ÀA] FAIRE", "TODO"), (r"EN COURS", "IN PROGRESS"),
               (r"TERMIN[ÉE]E?S?|FAITE?S?|COMPL[ÉE]T[ÉE]E?S?", "DONE"), (r"BLOQU[ÉE]E?S?", "BLOCKED"),
               (r"REPORT[ÉE]E?S?|DIFF[ÉE]R[ÉE]E?S?", "DEFERRED"), (r"ANNUL[ÉE]E?S?|SANS OBJET", "N/A"))
# A block heading: group 1 the id, a letter-suffixed milestone (`M1b-`) included (§A.1).
BLOCK_RE = re.compile(r"^## (\d*M[\d.]+[a-z]?-[A-Za-z][A-Za-z0-9.-]*)")
ID_START_RE = re.compile(r"[\[\s]*\d*M[\d.]+[a-z]?-[A-Za-z]")   # a Flow cell naming a block id
STATUS_FIELD_RE = re.compile(r"^- \**Status\**:")
# `Kind kept: <id> = <TYPE> (<the kind its plan gave it>)` - a Repo facts line, one per pre-v12 id (§0).
KIND_KEPT_RE = re.compile(r"Kind kept:\**\s*`?(\d*M[\d.]+[a-z]?-[A-Za-z][A-Za-z0-9.-]*?)`?\s*=\s*\**"
                          r"(BUILD|CHECK|PLAN|MOVE)(?![A-Za-z])")
TOP_RE = re.compile(r"^##? ")
TRAILER_RE = re.compile(r">|[ \t]*-{3,}[ \t]*$")   # a trailer line: a `>` banner line, a `---` rule
SUBHEAD_RE = re.compile(r"#{3,6}(?:[ \t]|$)")      # a `###`-`######` heading at column 0: a lot, a phase's part
GATE_RE = re.compile(r">|-{3,}[ \t]*$")            # at column 0: a `>` gate line, a `---` rule
HEADER_SECTIONS = ("## Flow", "## Rules", "## Superseded", "## Repo facts", "## Pipeline state")
FIELD_RE = re.compile(r"^- ([A-Za-z][A-Za-z0-9 ()/'-]*):")
DATE_RE = re.compile(r"\d{4}-\d{2}-\d{2}")
OPEN_RE = re.compile(r"^[ \t]*(?:[-*+]|\d+[.)])?[ \t]*(`{3,}|~{3,})")
RUNTAG_RE = re.compile(r"\[(ALREADY RUN|NOT RUN)")
PASS_RE, FAIL_RE = re.compile(r"Pass(\*\*)?:"), re.compile(r"Fail(\*\*)?:")
TAG_RE = re.compile(r"\*\*([^*]+)\*\*")
STAMP_RE = re.compile(r"IN PROGRESS \((\d{4}-\d{2}-\d{2}) (\d{2}:\d{2})\)")
TODAY = datetime.date.today().isoformat()
DIRS = ("logs", "reports", "tasks", "images")
READ_CAP = 500            # a `Read:` file over this many lines is named by section or range (§11)
REPEAT_MIN = 50           # shorter lines repeat by nature (field labels, placeholders)

# §3.3's closed set: a status starts with one of these words and has that word's shape.
CLOSED_SET = (
    ("TODO", re.compile(r"TODO$"),
     "TODO stands alone - a stall is BLOCKED (+ what unblocks it) or IN PROGRESS (awaiting: <question>)"),
    ("IN PROGRESS", re.compile(r"IN PROGRESS(?: \(.+)?$"),
     "IN PROGRESS stands alone or carries its (<stamp>) / (awaiting: <question>)"),
    ("DONE", re.compile(r"DONE\b.*\d{4}-\d{2}-\d{2}"), "DONE carries its date: DONE (YYYY-MM-DD ...)"),
    ("BLOCKED", re.compile(r"BLOCKED(?![A-Za-z])"), ""),
    ("DEFERRED", re.compile(r"DEFERRED \(wake: .+"),
     "DEFERRED carries its trigger: DEFERRED (wake: <condition>)"),
    ("N/A", re.compile(r"N/A(?![A-Za-z])"), ""),
)
CLOSED_SHAPES = ("TODO · IN PROGRESS (<stamp> | awaiting: <question>) · DONE (<date> ...) · BLOCKED "
                 "(+ what unblocks it) | BLOCKED-D<n> · DEFERRED (wake: <condition>) · N/A (<reason>)")

# §0's block types, set by the id.  Pre-v12 words (BUILDER, JUDGE, HUMAN and their composites)
# are read as aliases: never flagged, never rewritten.  A pre-v12 id whose plan gave it another
# kind carries it in a `Kind kept:` Repo facts line, which the lint reads before this table.
TYPES = ("BUILD", "CHECK", "PLAN", "MOVE")
ID_TYPES = (              # (the id's local part, the types it may carry, the tag `append` writes)
    (r"TC\d", ("MOVE",), "MOVE"),
    (r"TC(?![A-Za-z0-9])", ("PLAN",), "PLAN"),
    (r"TE-V\d", ("PLAN", "MOVE"), "PLAN"),               # MOVE: a TE-V<N>'s rename sweep
    (r"TB-V", ("PLAN",), "PLAN"),
    (r"TH(?![A-Z])", ("BUILD",), "BUILD"),
    (r"TW(?![A-Z])", ("CHECK",), "LEAD walk + CHECK"),
    (r"TV", ("BUILD",), "BUILD"),
    (r"TR", ("BUILD",), "BUILD"),
    (r"TD\d*(?:[a-z]\d*)?(?![A-Za-z0-9])", ("BUILD",), "BUILD"),         # TD, TDa - never TDOC
    (r"TE(?!-V\d)\d*(?:[a-z]\d*)?(?![A-Za-z0-9])", ("BUILD",), "BUILD"),  # TE, TE2, TEa, TE-<name>
    (r"T[LA]", ("CHECK",), "CHECK"),
    (r"T[IJ]", ("PLAN",), "PLAN + LEAD answers"),
    (r"T[PGBZ]", ("PLAN",), "PLAN"),
    (r"TM\d", ("PLAN",), "PLAN"),                         # the rating pass (v12.1); a bare v9 `TM` is no id here
    (r"DBG", ("BUILD",), "BUILD"),
    (r"[TRD]\d", ("BUILD",), "BUILD"),
    (r"V(?:\d|[a-z](?![A-Za-z]))", ("CHECK",), "CHECK"),                 # V1, and a split `Vf`
    (r"Q\d", (), "LEAD answers"),
)

# The calls, exactly as typed: a wrong call prints its line (§A.1).
CALLS = {
    "show": "show <ID>",
    "status": 'status <ID> "<STATUS>"',
    "close": 'close <ID> --status "<STATUS>" --text "<handoff>" [--set "<Key>=<value>"]...',
    "handoff": 'handoff <ID> --text "<handoff>"   (or --file <md>)',
    "flag": 'flag <ID> "<text>" --from <ID>',
    "append": 'append <block.md> --after <ID> [--goal "<one line>"]',
    "move": "move <ID> --after <ID2>",
    "stub": "stub <ID>... --by <ID> [--archive <file>]",
    "register": 'register --set "<Key>=<value>"...',
    "lint": "lint [--budget-header 600] [--budget-pipeline 60] [--tc-margin 100]",
    "header": "header [--budget-header 600] [--budget-pipeline 60]",
    "selftest": "selftest",
}
REMOVED = {"dt": "`dt` left with the D:T count (v12 §2.3: D per task is read from git by the audits)"}


# --------------------------------------------------------------------------- files
def read_text(path):
    with open(path, "rb") as fh:
        raw = fh.read()
    text = raw.decode("utf-8")
    nl = "\r\n" if "\r\n" in text else "\n"
    return text.replace("\r\n", "\n"), nl


class PlanWriteError(OSError):
    """The atomic write could not land.  Reported as a counted `REFUSED:` line, never as
    a traceback: a NO-GO writes nothing *and* says so."""


REPLACE_ATTEMPTS = 6          # 0.05+0.1+0.2+0.4+0.8 s of backoff, then refuse


def _injected_replace_failure():
    """Test seams: `PB_PLAN_TEST_REPLACE_SKIP=<k>` lets the next k replaces through, then
    `PB_PLAN_TEST_REPLACE_FAIL=<n>` fails the next n.

    A replace refused on Windows is a race against a scanner or an indexer holding the file,
    so it cannot be *scheduled*; the seam forces it, which is how the retry arm and the
    archive roll-back are red-armed on any box.
    """
    skip = int(os.environ.get("PB_PLAN_TEST_REPLACE_SKIP", "0") or 0)
    if skip > 0:
        os.environ["PB_PLAN_TEST_REPLACE_SKIP"] = str(skip - 1)
        return False
    n = int(os.environ.get("PB_PLAN_TEST_REPLACE_FAIL", "0") or 0)
    if n <= 0:
        return False
    os.environ["PB_PLAN_TEST_REPLACE_FAIL"] = str(n - 1)
    return True


def backoff():
    """The first retry's wait, doubling after; `PB_PLAN_TEST_BACKOFF` shortens it for the selftest."""
    return float(os.environ.get("PB_PLAN_TEST_BACKOFF", "0.05") or 0.05)


def replace_atomic(tmp, path, attempts=REPLACE_ATTEMPTS):
    """os.replace, retried on a transient PermissionError.

    On Windows a scanner or the search indexer can still hold the just-closed temp file (or
    the destination) for a few hundred ms; os.replace then raises PermissionError
    [WinError 5] - measured in the fleet twice in 8 selftest runs, once turning a full loop
    red.  The write stays atomic either way (the destination is never partial), so a bounded
    retry is safe; a permanent failure becomes a PlanWriteError that main() turns into a
    `REFUSED:` line.
    """
    delay, last = backoff(), None
    for i in range(attempts):
        try:
            if _injected_replace_failure():
                raise PermissionError(5, "injected [WinError 5] (PB_PLAN_TEST_REPLACE_FAIL)")
            os.replace(tmp, path)
            if i:
                print("WARN: replace of %s needed %d attempts - transient lock on this box"
                      % (os.path.basename(path), i + 1))
            return
        except PermissionError as exc:
            last = exc
            if i < attempts - 1:
                time.sleep(delay)
                delay *= 2
    raise PlanWriteError("could not replace %s after %d attempts (%s) - nothing was written"
                         % (path, attempts, last))


def _drop(tmp):
    """Remove a temp that never landed, retried like the replace: on Windows the process that
    refused the replace may still hold the temp too.  A temp that stays is left, never raised."""
    delay = backoff()
    for i in range(REPLACE_ATTEMPTS):
        try:
            os.unlink(tmp)
            return
        except FileNotFoundError:
            return
        except PermissionError:
            if i == REPLACE_ATTEMPTS - 1:
                return
            time.sleep(delay)
            delay *= 2


def write_text(path, text, nl):
    data = text.replace("\n", nl).encode("utf-8")
    folder = os.path.dirname(os.path.abspath(path)) or "."
    fd, tmp = tempfile.mkstemp(dir=folder, prefix=".pbplan.", suffix=".tmp")
    try:
        with os.fdopen(fd, "wb") as fh:
            fh.write(data)
        if os.path.exists(path):          # mkstemp makes the temp 0600: keep the file's own mode
            shutil.copymode(path, tmp)
        replace_atomic(tmp, path)
    finally:
        if os.path.exists(tmp):
            _drop(tmp)


LOCK_TIMEOUT_S = float(os.environ.get("PB_PLAN_LOCK_TIMEOUT", "20") or 20)
WRITE_CMDS = ("status", "close", "handoff", "flag", "append", "move", "stub", "register")


def lock_path_for(path):
    """A SIBLING lock file, never the plan itself: write_text lands by os.replace, and on
    Windows an open handle to the destination blocks that replace."""
    folder = os.path.dirname(os.path.abspath(path)) or "."
    return os.path.join(folder, "." + os.path.basename(path) + ".lock")


class PlanLock:
    """Advisory lock held across the whole load -> modify -> save window (§A.0).

    Every write call is a read-modify-write.  Unlocked, two agents that load before either
    saves both print `=== GO ===` with exit 0 and the second silently overwrites the first -
    measured in the fleet: 2 of 4 and 3 of 4 concurrent `status` writes landed, all four
    green.  A lost `- Handoff:` is a lost report.

    fcntl.flock on POSIX, msvcrt.locking on Windows; on any third platform this degrades to a
    documented no-op with a printed WARN rather than an ImportError.  On timeout the caller
    REFUSES - it never falls through to an unprotected write.
    """

    def __init__(self, path, timeout=None, shared=False):
        self.path = lock_path_for(path)
        self.timeout = LOCK_TIMEOUT_S if timeout is None else timeout
        self.shared = shared
        self.fh = None
        self.noop = fcntl is None and msvcrt is None

    def _try_once(self):
        if fcntl is not None:
            fcntl.flock(self.fh.fileno(), (fcntl.LOCK_SH if self.shared else fcntl.LOCK_EX)
                        | fcntl.LOCK_NB)
        else:
            self.fh.seek(0)
            msvcrt.locking(self.fh.fileno(), msvcrt.LK_NBLCK, 1)

    def acquire(self):
        if self.noop:
            print("WARN: no advisory file locking on this platform (%s) - concurrent"
                  " plan.py writes are UNPROTECTED" % sys.platform)
            return True
        self.fh = open(self.path, "a+b")
        deadline = time.time() + self.timeout
        while True:
            try:
                self._try_once()
                return True
            except OSError:
                if time.time() >= deadline:
                    self.fh.close()
                    self.fh = None
                    return False
                time.sleep(0.05)

    def release(self):
        if self.fh is None:
            return
        try:
            if fcntl is not None:
                fcntl.flock(self.fh.fileno(), fcntl.LOCK_UN)
            else:
                self.fh.seek(0)
                msvcrt.locking(self.fh.fileno(), msvcrt.LK_UNLCK, 1)
        except OSError:
            pass
        self.fh.close()
        self.fh = None

    def __enter__(self):
        return self.acquire()

    def __exit__(self, *exc):
        self.release()
        return False


def load_delay():
    """Test seam: `PB_PLAN_TEST_DELAY=<s>` widens load -> save so the race is FORCED rather
    than hoped for: a concurrency case must not pass merely because the box was fast."""
    d = float(os.environ.get("PB_PLAN_TEST_DELAY", "0") or 0)
    if d > 0:
        time.sleep(d)


def clock():
    """HH:MM for the §3.3 stamps; `PB_PLAN_TEST_CLOCK` pins it for the selftest."""
    return os.environ.get("PB_PLAN_TEST_CLOCK") or datetime.datetime.now().strftime("%H:%M")


def default_plan():
    cands = sorted(glob.glob("milestones/*/*implementation_plan.md"), key=os.path.getmtime, reverse=True)
    for path in cands:
        text, _ = read_text(path)
        for ln in text.split("\n"):
            if ln.startswith("- Status:") and state_of(ln.split(":", 1)[1]) in ("TODO", "IN PROGRESS"):
                return path
    return cands[0] if cands else None


# ------------------------------------------------------------------------- parsing
def fence_spans(lines):
    """[(open, end)] of every code fence, `end` exclusive: a heading inside one is no block.

    A fence opens on three or more backticks or tildes, after any indentation and at most one
    list marker (`  1. ` then the fence, as Verify steps are written); it closes on a line of
    the same character, as many or more, alone on its line (blanks and a CR trimmed).  A fence
    opened off column 0 also ends at the first non-blank line that starts at column 0 - its
    list item ends there - and that line is read as ordinary text.  A plain toggle on any
    fence-looking line loses real blocks on plans that open fences after a list marker
    (measured in the fleet: 1 block for 80, 11 for 24, 6 for 26).
    """
    spans, i, n = [], 0, len(lines)
    while i < n:
        m = OPEN_RE.match(lines[i])
        if not m:
            i += 1
            continue
        ch, width = m.group(1)[0], len(m.group(1))
        indented = lines[i][:1] not in ("`", "~")
        j, end = i + 1, None
        while j < n:
            t = lines[j].strip(" \t\r")
            if t[:1] == ch and len(t) >= width and t == ch * len(t):
                end = j + 1
                break
            if indented and re.match(r"[^ \t\r]", lines[j]):
                end = j
                break
            j += 1
        end = n if end is None else end
        spans.append((i, end))
        i = end
    return spans


def fence_mask(lines, spans=None):
    mask = [False] * len(lines)
    for lo, hi in (fence_spans(lines) if spans is None else spans):
        for k in range(lo, hi):
            mask[k] = True
    return mask


def gate_line(lines, mask, i):
    """A `>` line or a `---` rule at column 0, outside a fence, with a blank line above it: where a
    phase's gate starts (§10).  Glued to the line above, a `>` line is that line's quote."""
    return not mask[i] and GATE_RE.match(lines[i]) is not None and i > 0 and not lines[i - 1].strip()


def body_end(lines, mask, start, end):
    """One past a block's last body line.  The block's *trailer* is not the block's: from the first
    section line under its last field - a `###`-`######` heading, or a gate line (`gate_line`): a
    phase's `> **Commit gate (lead):**` banner (§10), the next lot's heading, a closing note after
    them - to the span's end; with none, the `>` lines and `---` rules at the span's end, blanks
    between.  A `>` line glued to the body (no blank above it) is the body's: a quote that ends a
    field; a `###` with a field of the block below it is the block's own."""
    fields = [i for i in range(start + 1, end) if not mask[i] and FIELD_RE.match(lines[i])]
    for i in range(fields[-1] + 1 if fields else end, end):
        if not mask[i] and (SUBHEAD_RE.match(lines[i]) or gate_line(lines, mask, i)):
            while i > start + 1 and not mask[i - 1] and not lines[i - 1].strip():
                i -= 1
            return i
    k = end
    while k > start + 1 and not mask[k - 1] and (not lines[k - 1].strip() or TRAILER_RE.match(lines[k - 1])):
        k -= 1
    if lines[k - 1].strip():
        while k < end and lines[k].startswith(">"):
            k += 1
    return k


class Block:
    def __init__(self, bid, start, end, lines, mask):
        self.id, self.start, self.end, self._lines, self._mask = bid, start, end, lines, mask
        self.last = body_end(lines, mask, start, end)

    @property
    def lines(self):
        return self._lines[self.start:self.end]

    @property
    def body(self):
        """The block's own lines, heading first: its trailer and the blanks around it left out."""
        return self._lines[self.start:self.last]

    @property
    def trailer(self):
        """The trailer's lines, blanks dropped: [] for a block that has none."""
        return [l for l in self._lines[self.last:self.end] if l.strip()]

    @property
    def heading(self):
        return self._lines[self.start]

    def status_index(self):
        for i in range(self.start + 1, self.end):
            if self._lines[i].strip():
                return i
        return -1

    @property
    def status_line(self):
        i = self.status_index()
        return self._lines[i] if i >= 0 else ""

    @property
    def status_text(self):
        ln = self.status_line
        return ln.split(":", 1)[1].strip() if ln.startswith("- Status:") else None

    @property
    def state(self):
        ln = self.status_line
        return state_of(ln.split(":", 1)[1]) if ":" in ln else None

    def field(self, name):
        """(first_line_index, end_index_exclusive) of `- <name>:` incl. continuations,
        trailing blank lines excluded so an edit never eats a block separator, nor the trailer; a
        line inside a fence neither starts nor ends a field, and no field runs over a gate line
        (a quote or a rule after a blank: a note between two fields is no field's)."""
        want = "- %s:" % name
        for i in range(self.start + 1, self.last):
            if not self._mask[i] and self._lines[i].startswith(want):
                j = i + 1
                while j < self.last and (self._mask[j] or not (FIELD_RE.match(self._lines[j])
                                                              or self._lines[j].startswith("## ")
                                                              or gate_line(self._lines, self._mask, j))):
                    j += 1
                while j - 1 > i and not self._lines[j - 1].strip():
                    j -= 1
                return i, j
        return None

    def field_text(self, name):
        span = self.field(name)
        if not span:
            return None
        return "\n".join(self._lines[span[0]:span[1]]).split(":", 1)[1]


def state_of(text):
    """A §3.3 state, anchored: the status text *starts* with it, as a whole word, after bold or
    code marks. `ALMOST DONE` is not a state; a state word in the plan's own language (`TERMINÉ`,
    `À FAIRE`) reads as its state, and nothing else does."""
    t = text.strip().lstrip("*`").strip().upper()
    for st in STATES:
        if re.match(re.escape(st) + r"(?!\w)", t):
            return st
    for rx, st in LANG_STATES:
        if re.match(r"(?:%s)(?!\w)" % rx, t):
            return st
    return None


def block_state(b):
    """The state the lint reads a block by, wherever an old plan keeps it: the `- Status:` first
    line, else the first `- Status:` field further down, else the heading after its type tag (an
    old stub's `**JUDGE** · **DONE (2026-08-12)**`); None when none of them starts with a state."""
    if b.status_line.startswith("- Status:"):
        return state_of(b.status_text)
    for i in range(b.start + 1, b.end):
        if not b._mask[i] and STATUS_FIELD_RE.match(b._lines[i]):
            return state_of(b._lines[i].split(":", 1)[1])
    tag = TAG_RE.search(b.heading)
    for seg in re.split(r"\s[·—–]\s", b.heading[tag.end():]) if tag else ():
        st = state_of(re.sub(r"^[\s(*`_]*(?:Status\W*)?", "", seg))
        if st:
            return st
    return None


def status_error(text):
    """Why `text` is off §3.3's closed set, else None: a stall written in prose or in another
    language is refused, never guessed, so it is counted like any other status."""
    t = text.strip()
    for st, shape, why in CLOSED_SET:
        if re.match(re.escape(st) + r"(?![A-Za-z])", t):
            return None if shape.match(t) else "%r: %s" % (t, why)
    return "%r is off §3.3's closed set - the statuses: %s" % (t, CLOSED_SHAPES)


def stamped(text, old, date, now):
    """§3.3's stamps: a bare `IN PROGRESS` / `DONE` (or one carrying only today's date) gains
    the date and time; DONE gains the start time read from the IN PROGRESS stamp it replaces.
    Any other text is written as typed."""
    for st in ("IN PROGRESS", "DONE"):
        if text in (st, "%s (%s)" % (st, date)):
            if st == "IN PROGRESS":
                return "IN PROGRESS (%s %s)" % (date, now)
            m = STAMP_RE.match(old or "")
            if not m:
                return "DONE (%s %s)" % (date, now)
            began = m.group(2) if m.group(1) == date else "%s %s" % (m.group(1), m.group(2))
            return "DONE (%s %s, started %s)" % (date, now, began)
    return text


def id_tail(bid):
    """`M4-TH-adopt-a` -> `TH-adopt-a`; `M12.1-T2a` -> `T2a`."""
    return bid.split("-", 1)[1] if "-" in bid else bid


def id_type(bid):
    """(the types the id may carry, the tag `append` writes) from §0's table, else None."""
    tail = id_tail(bid)
    for rx, allowed, written in ID_TYPES:
        if re.match(rx, tail):
            return allowed, written
    return None


def split_of(tail, tails):
    """True when `tail` is a split of a number the plan already holds (`T24a` beside `T24`,
    `D6b2` beside `D6b`, `TH-d` beside `TH-a`): §3.1 - a split mints no new number."""
    m = re.match(r"^([A-Z]+)(\d*)(.*)$", tail)
    if not m or not m.group(3):
        return False
    cls, num, rest = m.groups()
    if num and rest[:1].islower():
        fam = re.compile(r"^%s%s(?!\d)" % (cls, num))
    elif not num and re.fullmatch(r"(?:-[A-Za-z0-9]+)*-[a-z]", rest):
        fam = re.compile(r"^%s%s-" % (cls, re.escape(rest.rsplit("-", 1)[0])))
    else:
        return False
    return any(t != tail and fam.match(t) for t in tails)


def find_blocks(lines, mask):
    tops = [i for i, ln in enumerate(lines) if not mask[i] and TOP_RE.match(ln)]
    out = []
    for k, i in enumerate(tops):
        m = BLOCK_RE.match(lines[i])
        if m:
            out.append(Block(m.group(1), i, tops[k + 1] if k + 1 < len(tops) else len(lines), lines, mask))
    return out


def header_end(lines, blocks):
    return blocks[0].start if blocks else len(lines)


# ------------------------------------------------------------------ the ladder and the rating
# §0: the header's `Models:` line is the ladder once it reads `<agent> — <rung> · <rung> · … —
# strongest first; read <date> from <source>`; any other `Models:` line (a v12 plan's "the lead's
# pick") is no ladder.  §3.2: a heading's rating is the segment right after its bold type tag,
# `<model>, <level>`; ` · switch` may follow it (the lead deletes it to bypass Claude Code's switch).
MODELS_RE = re.compile(r"^[-*\s]*\**Models:\**\s*(.*)$")
MARK_RE = re.compile(r"\s+\((gate|usual|gate, usual)\)$")
LEVEL_RE = re.compile(r"[A-Za-z0-9][\w.-]*(?: [\w.-]+)*$")
ROLE_WORDS = ("gate rung", "usual rung", "lowest rung")    # a plan with no ladder yet (§3.2)
SWITCH = "switch"
TYPE_WORDS = set(TYPES) | {"BUILDER", "JUDGE", "HUMAN", "LEAD"}


def parse_rung(text):
    """(model, level, marks) for `<model>, <level>` with an optional ` (gate)` / ` (usual)` /
    ` (gate, usual)`, else None.  The level is the text after the last `, ` - a model may carry a
    comma or a dot (`Sonnet 5.5`, `gpt-5.5-codex`) and parses whole; `none` is a level."""
    t, marks = text.strip(), ()
    m = MARK_RE.search(t)
    if m:
        marks, t = tuple(w.strip() for w in m.group(1).split(",")), t[:m.start()]
    if ", " not in t:
        return None
    model, level = (s.strip() for s in t.rsplit(", ", 1))
    if not model or re.search(r"[·*`()\[\]]", model) or not LEVEL_RE.match(level):
        return None
    return model, level, marks


def rung_key(model, level):
    return " ".join(model.split()).casefold(), " ".join(level.split()).casefold()


class Ladder:
    """The header's `Models:` ladder: the agent, the rungs strongest first, what is wrong with it."""

    def __init__(self, line, text):
        self.line, self.text, self.rungs, self.errors = line, text, [], []
        parts = text.split(" — ")
        if len(parts) < 3 or not parts[2].startswith("strongest first"):
            self.agent = parts[0].strip()
            self.errors.append("the `Models:` line (line %d) says `strongest first` but is not `<agent> — "
                               "<model>, <level> (gate) · … — strongest first; read <date> from <source>` "
                               "(§0)" % (line + 1))
            return
        self.agent = parts[0].strip()
        for raw in parts[1].split(" · "):
            r = parse_rung(raw)
            if r:
                self.rungs.append((raw.strip(),) + r)
            else:
                self.errors.append("the ladder's rung %r (line %d) is not `<model>, <level>` (§0)"
                                   % (raw.strip(), line + 1))
        for mark in ("gate", "usual"):
            n = sum(mark in r[3] for r in self.rungs)
            if n != 1:
                self.errors.append("the ladder (line %d) marks %d rung(s) `(%s)` - exactly one (§0)"
                                   % (line + 1, n, mark))
        if not self.agent:
            self.errors.append("the ladder (line %d) names no agent (§0)" % (line + 1))

    def words(self):
        return " · ".join(r[0] for r in self.rungs) or "(no rung parses)"

    def holds(self, rating):
        r = parse_rung(rating)
        return bool(r) and not r[2] and rung_key(r[0], r[1]) in {rung_key(x[1], x[2]) for x in self.rungs}


def read_ladder(lines, mask, end):
    """The header's ladder, or None: the first `Models:` line above the first block, outside a
    fence, that says `strongest first` (§0) - every plan before its v12.1 adoption has none."""
    for i in range(end):
        if not mask[i]:
            m = MODELS_RE.match(lines[i])
            if m and "strongest first" in m.group(1):
                return Ladder(i, m.group(1).strip())
    return None


def type_tag(heading):
    """The heading's bold type tag (§0): the first bold segment naming a type or an alias - never
    a bold word in the title, never an old stub's `**DONE (…)**`."""
    for m in TAG_RE.finditer(heading):
        if TYPE_WORDS & set(re.findall(r"[A-Z]+", m.group(1))):
            return m
    return None


def rating_seg(seg):
    """True when a heading segment sits in the rating's slot: a role word, the switch, or
    `<model>, <level>` - on the ladder or off it (the check says which)."""
    s = seg.strip()
    r = parse_rung(s)
    return s in ROLE_WORDS or s == SWITCH or bool(r and not r[2])


def heading_rating(heading):
    """(rating, switch) from a heading (§3.2): the segment right after the type tag when it reads
    `<model>, <level>` or a role word, else None; `switch` True when ` · switch` sits right after
    the type tag or the rating.  The order clause, `∥ set-X` and an old stub's `**DONE (…)**` are
    never a rating; the switch is read, never written."""
    tag = type_tag(heading)
    segs = heading[tag.end():].rstrip().split(" · ") if tag else [""]
    segs = [s.strip() for s in segs[1:]] if segs[0] == "" else []
    if not segs:
        return None, False
    if segs[0] == SWITCH:
        return None, True
    r = parse_rung(segs[0])
    if segs[0] in ROLE_WORDS or (r and not r[2]):
        return segs[0], len(segs) > 1 and segs[1] == SWITCH
    return None, False


def rated(bid):
    """A block the rating rule binds: every id but a Q, which the lead answers (§3.2)."""
    typed = id_type(bid)
    return not (typed and typed[0] == ())


def rating_error(bid, heading, ladder):
    """Why a block's heading fails the plan's ladder (§3.2), else None."""
    rating, switch = heading_rating(heading)
    if rating is None:
        return ("%s: no rating after its type tag%s - write one rung of the ladder there: %s"
                % (bid, " (` · switch` with no rating before it)" if switch else "", ladder.words()))
    if not ladder.holds(rating):
        return ("%s: rating %r is off the ladder%s - its rungs: %s"
                % (bid, rating, " (a role word: the adoption converts it to a rung)"
                   if rating in ROLE_WORDS else "", ladder.words()))
    return None


# §0 and §11: the rating pass's refresh trigger, in a plan with a ladder only.  A carried flag counts
# on a BUILD block still to run - CHECK, PLAN, MOVE and a `LEAD go` are pinned by the rubric, a flag
# there moves no rating - when its `[<from>, <date>]` stamp (bold and a backticked id read through)
# is later than the register's `Model ratings: <date> by <id>` line - one stamped that day was there
# for the pass to read; an unstamped flag counts only while the register has no such line - a pass
# read it, after that it is history, still named in the lint's WARN.  At the Rules' `Rating refresh:
# <n> flags` (default 5) the lint is red until a rating pass is TODO or IN PROGRESS - the header
# budget's TC<n> shape.
REFRESH_RE = re.compile(r"Rating refresh:\**\s*(\d+)\s*flags?\b")
REFRESH_DEFAULT = 5
FLAG_STAMP_RE = re.compile(r"(?:^|·)\s*(?:\*\*)?\[`?([^\[\],`]+)`?, (\d{4}-\d{2}-\d{2})\]")
LEAD_GO_RE = re.compile(r"\bLEAD go\b")
NO_FLAGS = ("", "none", "-", "—", "n/a")
RATINGS_KEY = "Model ratings"


def carried_flags(text):
    """One entry per flag of a `Carried flags:` field: (from, date) for each stamp, which opens a
    flag; None for text before the first stamp (or a field with none) - one hand-typed flag."""
    t = (text or "").strip()
    stamps = list(FLAG_STAMP_RE.finditer(t))
    lead = (t[:stamps[0].start()] if stamps else t).strip(" ·\t\r\n")
    return ([None] if lead.casefold() not in NO_FLAGS else []) + [(m.group(1).strip(), m.group(2)) for m in stamps]


def flag_moves_rating(b, kept):
    """True when a carried flag can move the block's rating (§0's rubric): a BUILD by §0's id table
    or its `Kind kept:` line, never a `LEAD go` - rule (2) pins it to `(gate)`."""
    typed = id_type(b.id)
    kind = kept.get(b.id) or (typed[0][0] if typed and len(typed[0]) == 1 else None)
    return kind == "BUILD" and not any(LEAD_GO_RE.search(m.group(1)) for m in TAG_RE.finditer(b.heading))


class Refresh:
    """The refresh trigger's reading: the threshold, the window, the flags it counts, the pass
    pending (a TODO or IN PROGRESS `TM<n>`), and whether it fires."""

    def __init__(self, plan):
        self.threshold, self.since, self.by, self.counted, self.unstamped = REFRESH_DEFAULT, None, None, [], []
        span = plan.section("## Rules")
        for i in range(*span) if span else ():
            m = None if plan.mask[i] else REFRESH_RE.search(plan.lines[i])
            if m:
                self.threshold = int(m.group(1))
                break
        span = plan.section("## Pipeline state")
        for i in range(*span) if span else ():
            if not plan.mask[i] and register_key(plan.lines[i]) == RATINGS_KEY:
                val = plan.lines[i].split(":", 1)[1]
                d, by = DATE_RE.search(val), re.search(r"\bby\s+(.+?)\s*$", val)
                self.since, self.by = (d.group(0) if d else None), (by.group(1) if by else None)
                break
        states = {b.id: block_state(b) for b in plan.blocks}
        self.pending = next((b.id for b in plan.blocks if re.match(r"TM\d", id_tail(b.id))
                             and states[b.id] in ("TODO", "IN PROGRESS")), None)
        kept = kinds_kept(plan)
        for b in plan.blocks:
            if states[b.id] in FINISHED or not flag_moves_rating(b, kept):
                continue
            for stamp in carried_flags(b.field_text("Carried flags")):
                if stamp is None:
                    self.unstamped.append(b.id)
                if self.since is None or (stamp is not None and stamp[1] > self.since):
                    self.counted.append(b.id)
        self.count = len(self.counted)
        self.fires = self.count >= self.threshold and not self.pending
        self.anchor = self.next_id = None
        if self.fires:
            # It runs next, after a pending TC<n> (§0): after that, else after the block in progress,
            # else after the block before the first TODO.
            tc = [b.id for b in plan.blocks if re.match(r"TC\d", id_tail(b.id)) and states[b.id] == "TODO"]
            busy = [b.id for b in plan.blocks if states[b.id] == "IN PROGRESS"]
            todo = [k for k, b in enumerate(plan.blocks) if states[b.id] == "TODO"]
            self.anchor = (tc or busy or [plan.blocks[max(todo[0] - 1, 0)].id if todo else plan.blocks[-1].id])[0]
            nums = [int(m.group(1)) for b in plan.blocks for m in [re.match(r"TM(\d+)$", id_tail(b.id))] if m]
            span = plan.section("## Pipeline state")
            for i in range(*span) if span else ():
                m = re.search(r"\bTM=(\d+)", plan.lines[i]) if register_key(plan.lines[i]) == "Counters" else None
                nums += [int(m.group(1))] if m else []
            self.next_id = "%s-TM%d" % (self.anchor.split("-", 1)[0], max(nums + [0]) + 1)

    def window(self):
        return ("since `%s: %s by %s`" % (RATINGS_KEY, self.since, self.by or "?") if self.since
                else "with no dated `%s:` line in the register (every flag counted)" % RATINGS_KEY)

    def call(self, plan):
        return '%s --plan %s append <%s block.md> --after %s --goal "Rating pass - %d carried flags"' % (
            PROG, plan.path, self.next_id, self.anchor, self.count)

    def red(self, plan):
        return ("rating refresh: %d carried flag(s) on open BUILD blocks %s reach `Rating refresh: %d flags` and no TM<n> "
                "is pending - file the rating pass (§14.3's TM<n> template, rated), it runs next: %s"
                % (self.count, self.window(), self.threshold, self.call(plan)))


def section_span(lines, prefix, limit, mask=None):
    for i in range(min(limit, len(lines))):
        if (mask is None or not mask[i]) and lines[i].startswith(prefix):
            j = i + 1
            while j < limit and not ((mask is None or not mask[j]) and TOP_RE.match(lines[j])):
                j += 1
            return i, j
    return None


class Flow:
    """The first markdown table under `## Flow`, and how the plan's Flow reads (`kind`): `per-task`
    when most of its rows name a block (or it has none yet) - checked row by row; `prose` (a
    section with no table), `phase-table` (most rows name no block) or `none` (no `## Flow`) are
    history: no block needs a row and none is written, as `propagate.py` reads them (§6.4)."""

    def __init__(self, lines, limit, mask=None):
        self.rows = []          # list of (line_index, [cells])
        self.head = []
        self.start = self.stop = -1
        self.kind = "none"
        span = section_span(lines, "## Flow", limit, mask)
        if not span:
            return
        self.kind = "prose"
        i, end = span
        while i < end and not lines[i].lstrip().startswith("|"):
            i += 1
        j = i
        while j < end and lines[j].lstrip().startswith("|"):
            j += 1
        if j - i < 2:
            return
        self.start, self.stop = i, j
        self.head = cells(lines[i])
        for k in range(i + 2, j):
            self.rows.append((k, cells(lines[k])))
        named = sum(1 for _, cell in self.ids() if ID_START_RE.match(cell))
        self.kind = "per-task" if 2 * named > len(self.rows) or not self.rows else "phase-table"

    def col(self, *names):
        low = [c.strip().lower() for c in self.head]
        for n in names:
            if n in low:
                return low.index(n)
        return -1

    def status_col(self):
        c = self.col("status")
        return c if c >= 0 else len(self.head) - 1

    def row_for(self, bid):
        c = self.col("task", "id", "block")
        c = c if c >= 0 else 0
        for idx, cs in self.rows:
            if len(cs) > c and clean(cs[c]) == bid:
                return idx, cs
        return None

    def ids(self):
        c = self.col("task", "id", "block")
        c = c if c >= 0 else 0
        return [(idx, clean(cs[c])) for idx, cs in self.rows if len(cs) > c]


def cells(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return s.split("|")


def render_row(cs):
    return "|" + "|".join(cs) + "|"


def clean(cell):
    return cell.replace("`", "").replace("*", "").strip()


def set_cell(cs, idx, value):
    out = list(cs)
    while len(out) <= idx:
        out.append(" ")
    out[idx] = " %s " % value.strip()
    return out


# ------------------------------------------------------------------------- verdict
def verdict(name, checks, fails, warns=(), payload=None, as_json=False, refused=False, counts=(), history=""):
    """The §4 Rule 4 shape; `history` - what the lint read as history - is one `HISTORY:` line, never a failure."""
    if as_json:
        print(json.dumps({"cmd": name, "checks": checks, "failed": len(fails),
                          "warnings": len(warns), "failures": list(fails),
                          "warns": list(warns), "refused": bool(refused), "history": history,
                          "counts": dict(counts), "data": payload or {}}, indent=1))
    else:
        print("%s: checks=%d failed=%d warnings=%d%s" % (
            name, checks, len(fails), len(warns),
            "".join(" · %s=%s" % kv for kv in counts)))
        if history:
            print("HISTORY: " + history)
        for f in fails:
            print("FAIL: " + f)
        for w in warns:
            print("WARN: " + w)
        if refused:
            print("REFUSED: %s wrote nothing - the failed check(s) above blocked the write" % name)
    if checks == 0:
        print("=== NO-GO: zero checks ===")
        return 1
    if fails:
        print("=== NO-GO: %d violation(s) ===" % len(fails))
        return 1
    print("=== GO ===")
    return 0


class Plan:
    def __init__(self, path):
        self.path = path
        text, self.nl = read_text(path)
        load_delay()
        self.lines = text.split("\n")
        self.reparse()

    def reparse(self):
        self.spans = fence_spans(self.lines)
        self.mask = fence_mask(self.lines, self.spans)
        self.blocks = find_blocks(self.lines, self.mask)
        self.header_end = header_end(self.lines, self.blocks)
        self.flow = Flow(self.lines, self.header_end, self.mask)
        self.ladder = read_ladder(self.lines, self.mask, self.header_end)

    def block(self, bid):
        for b in self.blocks:
            if b.id == bid:
                return b
        return None

    def section(self, prefix):
        return section_span(self.lines, prefix, self.header_end, self.mask)

    def save(self):
        write_text(self.path, "\n".join(self.lines), self.nl)
        self.reparse()

    def save_if(self, fails):
        """A call whose own checks failed writes NOTHING: the in-memory edit is dropped, so a
        NO-GO always leaves the plan byte-identical and the same call can be retried without
        duplicating anything.  Returns True when the plan was written."""
        if fails:
            return False
        self.save()
        return True


def no_block(plan, bid, what="block"):
    """The refusal for an id the plan does not hold, naming the whole ids it may have meant."""
    near = [b.id for b in plan.blocks if id_tail(b.id) == bid or b.id.lower() == bid.lower()]
    return "no %s %s in %s%s" % (what, bid, plan.path,
                                 " - ids are written whole: %s" % " · ".join(near) if near else "")


def trailer_note(b):
    """A block's trailer named for a WARN: its first line, cut to 60 characters."""
    t = b.trailer[0].strip()
    return t if len(t) <= 60 else t[:59] + "…"


def above_trailer(bid, anchor):
    """The WARN when `bid` goes in right after `anchor`'s body, above its trailer (§10's banner)."""
    return ("%s went in above %s's trailer (%s), which stays last - check which side of it the block "
            "belongs on" % (bid, anchor.id, trailer_note(anchor)))


# ------------------------------------------------------------------------ the edits
def edit_status(plan, bid, text, a):
    """The Status line and its Flow cell, in memory: (checks, fails, final text)."""
    b = plan.block(bid)
    if not b:
        return 1, [no_block(plan, bid)], None
    if not b.status_line.startswith("- Status:"):
        return 1, ["%s: its first line is not `- Status:` (line %d) - fix the block first"
                   % (bid, b.status_index() + 1)], None
    text = stamped(text.strip(), b.status_text, a.date, clock())
    why = status_error(text)
    if why:
        return 1, [why], None
    # A Flow read as history (prose, a phase table, none) holds no row to keep in step.
    row = plan.flow.row_for(bid) if plan.flow.kind == "per-task" else None
    if plan.flow.kind == "per-task" and not row:
        return 1, ["no Flow row for %s - add the row first, then re-run (nothing was written)" % bid], None
    plan.lines[b.status_index()] = "- Status: " + text
    if row:
        idx, cs = row
        plan.lines[idx] = render_row(set_cell(cs, plan.flow.status_col(), text))
    plan.reparse()
    return 1 + bool(row), [], text


def cmd_status(plan, a):
    text = " ".join(a.status).strip()
    checks, fails, final = edit_status(plan, a.id, text, a)
    saved = plan.save_if(fails)
    return verdict("status", checks, fails, payload={"id": a.id, "status": final}, as_json=a.json,
                   refused=not saved, counts=[("flow", plan.flow.kind)])


def handoff_body(a):
    """The handoff's lines from --text or --file: (lines, fails)."""
    if a.file and not os.path.isfile(a.file):
        return None, ["no handoff file %s - nothing was written" % a.file]
    try:
        text = read_text(a.file)[0] if a.file else a.text
    except (OSError, UnicodeDecodeError) as exc:
        return None, ["cannot read %s: %s" % (a.file, exc)]
    body = [l.rstrip() for l in text.strip("\n").split("\n")]
    if not body or not body[0].strip():
        return None, ["an empty handoff - nothing was written"]
    return body, []


def edit_handoff(plan, bid, body, allow_long):
    """Replaces the `- Handoff:` field in memory: (checks, fails, warns)."""
    b = plan.block(bid)
    if not b:
        return 1, [no_block(plan, bid)], []
    if len(body) > 8 and not allow_long:
        return 1, ["handoff is %d lines (max 8; detail goes to tasks/<ID>.md, or --allow-long)"
                   % len(body)], []
    warns = []
    if len(body) > 8:
        warns.append("handoff is %d lines - over the 8-line bar, allowed by --allow-long" % len(body))
    detail = os.path.join(os.path.dirname(plan.path), "tasks", "%s.md" % bid)
    if not os.path.exists(detail):
        warns.append("no %s - detail belongs there, not in the plan" % detail)
    new = ["- Handoff: " + body[0]] + ["  " + l.lstrip() for l in body[1:]]
    span = b.field("Handoff")
    if span:
        plan.lines[span[0]:span[1]] = new
    else:
        plan.lines[b.last:b.last] = new
        # Not a partial edit: the handoff IS written, at the end of the block's body - above its
        # trailer.  A warning, not a failure, so the write is complete and lands.
        warns.append("block %s had no `- Handoff:` field; one was appended at the end of the block" % bid)
    plan.reparse()
    return 1, [], warns


def cmd_handoff(plan, a):
    body, fails = handoff_body(a)
    if fails:
        return verdict("handoff", 1, fails, as_json=a.json, refused=True)
    checks, fails, warns = edit_handoff(plan, a.id, body, a.allow_long)
    saved = plan.save_if(fails)
    return verdict("handoff", checks, fails, warns, payload={"lines": len(body)}, as_json=a.json,
                   refused=not saved)


def cmd_close(plan, a):
    """A task's whole close-out in one write: status, handoff, register lines (§A.1)."""
    body, fails = handoff_body(a)
    if fails:
        return verdict("close", 1, fails, as_json=a.json, refused=True)
    checks, fails, final = edit_status(plan, a.id, a.status, a)
    warns = []
    if not fails:
        c, f, warns = edit_handoff(plan, a.id, body, a.allow_long)
        checks, fails = checks + c, f
    counts = []
    if not fails and a.set:
        c, f, w, counts = edit_register(plan, a.set)
        checks, fails, warns = checks + c, f, warns + w
    saved = plan.save_if(fails)
    return verdict("close", checks, fails, warns, payload={"id": a.id, "status": final},
                   as_json=a.json, refused=not saved,
                   counts=[("handoff_lines", len(body)), ("flow", plan.flow.kind)] + counts)


def cmd_flag(plan, a):
    b = plan.block(a.id)
    if not b:
        return verdict("flag", 1, [no_block(plan, a.id)], as_json=a.json, refused=True)
    entry = "· [%s, %s] %s" % (a.frm, a.date, a.text)
    span = b.field("Carried flags")
    if span:
        plan.lines[span[1] - 1] = plan.lines[span[1] - 1].rstrip() + " " + entry
    else:
        plan.lines[b.status_index() + 1:b.status_index() + 1] = ["- Carried flags: " + entry[2:].strip()]
    saved = plan.save_if([])
    counts, warns, payload = [], [], {"id": a.id}
    if saved and plan.ladder:
        # §0: the running count against the plan's threshold, so the filer who tips it sees the call.
        rf = Refresh(plan)
        counts, payload["refresh"] = [("refresh", "%d/%d" % (rf.count, rf.threshold))], [rf.count, rf.threshold]
        if rf.fires:
            warns.append(rf.red(plan))
    return verdict("flag", 1, [], warns, payload=payload, as_json=a.json, refused=not saved, counts=counts)


def cmd_append(plan, a):
    anchor = plan.block(a.after)
    if not anchor:
        return verdict("append", 1, [no_block(plan, a.after, "anchor block")], as_json=a.json, refused=True)
    if not os.path.isfile(a.block):
        return verdict("append", 1, ["no block file %s - nothing to append" % a.block],
                       as_json=a.json, refused=True)
    try:
        text, _ = read_text(a.block)
    except (OSError, UnicodeDecodeError) as exc:
        return verdict("append", 1, ["cannot read %s: %s" % (a.block, exc)],
                       as_json=a.json, refused=True)
    body = [l.rstrip() for l in text.strip("\n").split("\n")]
    m = BLOCK_RE.match(body[0])
    if not m:
        return verdict("append", 1, ["%s does not start with a `## <ID> · <title> · **<TYPE>**` heading"
                                     % a.block], as_json=a.json, refused=True)
    new_id = m.group(1)
    if plan.block(new_id):
        return verdict("append", 1, ["id %s is already in the plan" % new_id], as_json=a.json, refused=True)
    first = next((l for l in body[1:] if l.strip()), "")
    why = status_error(first.split(":", 1)[1]) if first.startswith("- Status:") else \
        "the block's first line is not `- Status:`"
    if why:
        return verdict("append", 1, ["%s: %s" % (new_id, why)], as_json=a.json, refused=True)
    checks, fails, warns = 0, [], []
    # §0: the type follows the id - written from the table when the heading has none.
    typed = id_type(new_id)
    if not TAG_RE.search(body[0]):
        if typed:
            parts = body[0].split(" · ")    # the type goes before the rating, the switch, the order
            at = next((k for k in range(2, len(parts))
                       if parts[k].startswith("(") or rating_seg(parts[k])), len(parts))
            parts[at:at] = ["**%s**" % typed[1]]
            body[0] = " · ".join(parts)
        else:
            warns.append("no type written: %s's class is not in §0's id table" % new_id)
    # §3.2: once the header carries a ladder, a block is filed with a rung of it - by its filer, in
    # this edit; the heading is never rated (nor its switch written) by the tool.
    if plan.ladder and rated(new_id):
        why = rating_error(new_id, body[0], plan.ladder)
        if why:
            return verdict("append", 1, [why], as_json=a.json, refused=True)
    at = anchor.last                  # above the anchor's trailer, which stays last (§10's banner)
    if anchor.trailer:
        warns.append(above_trailer(new_id, anchor))
    plan.lines[at:at] = [""] + body
    checks += 1
    plan.reparse()
    per_task = plan.flow.kind == "per-task"    # a Flow read as history gets no row (§6.4, as propagate.py)
    row = plan.flow.row_for(a.after) if per_task else None
    if row:
        idx, cs = row
        bold = TAG_RE.search(body[0])
        parts = [p.strip() for p in body[0].split("·")]
        goal = a.goal or (parts[1] if len(parts) > 1 else new_id)
        out = [" " for _ in cs] or [" ", " ", " ", " ", " "]
        for k, name in enumerate(c.strip().lower() for c in plan.flow.head):
            if k >= len(out):
                out.append(" ")
            if name in ("task", "id", "block"):
                out[k] = " %s " % new_id
            elif name in ("type", "stance", "role", "model", "who"):
                out[k] = " %s " % (bold.group(1) if bold else "")
            elif name == "goal":
                out[k] = " %s " % goal
            elif name == "order":
                out[k] = " AFTER %s " % a.after
            elif name == "status":
                out[k] = " %s " % first.split(":", 1)[1].strip()
        plan.lines[idx + 1:idx + 1] = [render_row(out)]
        checks += 1
    elif per_task:
        fails.append("anchor %s has no Flow row - a block with no row would be a partial file; "
                     "nothing was written" % a.after)
    tail = id_tail(new_id)
    letters = re.match(r"^[A-Z]+", tail)
    split = split_of(tail, [id_tail(b.id) for b in plan.blocks])
    if letters and not split:
        span = plan.section("## Pipeline state")
        bumped = False
        if span:
            for i in range(*span):
                if plan.lines[i].startswith("- Counters:"):
                    pat = re.compile(r"(?<![A-Za-z])%s=(\d+)" % re.escape(letters.group(0)))
                    mm = pat.search(plan.lines[i])
                    if mm:
                        plan.lines[i] = pat.sub("%s=%d" % (letters.group(0), int(mm.group(1)) + 1),
                                                plan.lines[i], count=1)
                        bumped = True
                    break
        checks += 1
        if not bumped:
            # A hard refusal that writes nothing, NOT a warning that lands the block: `lint` has
            # no counter check, so a warning here would rot unseen.
            fails.append("no `%s=<n>` counter in the `- Counters:` line of the Pipeline state - "
                         "nothing was written; add it first, e.g. `%s --plan %s register "
                         "--set \"Counters=<the whole line> · %s=0\"`, then re-run this append"
                         % (letters.group(0), PROG, plan.path, letters.group(0)))
    saved = plan.save_if(fails)
    return verdict("append", checks, fails, warns, payload={"id": new_id, "after": a.after,
                                                            "split": split},
                   as_json=a.json, refused=not saved,
                   counts=[("flow", plan.flow.kind), ("flow_row", "written" if row and saved else "none")])


def cmd_move(plan, a):
    b, anchor = plan.block(a.id), plan.block(a.after)
    if not b or not anchor:
        return verdict("move", 1, [no_block(plan, a.id) if not b else no_block(plan, a.after, "anchor block")],
                       as_json=a.json, refused=True)
    if b.id == anchor.id:
        return verdict("move", 1, ["cannot move %s after itself" % a.id], as_json=a.json, refused=True)
    body = b.body
    original = list(body)
    row = plan.flow.row_for(a.id)
    row_line = plan.lines[row[0]] if row else None
    warns, stop = [], b.end
    if b.trailer:
        # The trailer stays where it sat: the block goes from its heading to the trailer's first line.
        stop = next(j for j in range(b.last, b.end) if plan.lines[j].strip())
        warns.append("%s's trailer (%s) stays where it sat - the block moved without it"
                     % (a.id, trailer_note(b)))
    drop = set(range(b.start, stop)) | ({row[0]} if row else set())
    plan.lines = [l for i, l in enumerate(plan.lines) if i not in drop]
    plan.reparse()
    anchor = plan.block(a.after)
    if anchor.trailer:
        warns.append(above_trailer(a.id, anchor))
    plan.lines[anchor.last:anchor.last] = [""] + body
    if row_line is not None:
        plan.reparse()
        arow = plan.flow.row_for(a.after)
        if arow:
            plan.lines[arow[0] + 1:arow[0] + 1] = [row_line]
    plan.reparse()
    moved = plan.block(a.id)
    got = moved.body if moved else []
    if got != original:
        return verdict("move", 1, ["the block's bytes would change (%d -> %d lines)"
                                   % (len(original), len(got))], as_json=a.json, refused=True)
    saved = plan.save_if([])
    print("moved %s after %s (%d lines, byte-identical). Write the ordering note (E8)." %
          (a.id, a.after, len(original)))
    return verdict("move", 1, [], warns, payload={"id": a.id, "lines": len(original)}, as_json=a.json,
                   refused=not saved)


def cmd_stub(plan, a):
    archive = a.archive or os.path.join(os.path.dirname(plan.path), "plan_archive.md")
    checks, fails, moved, picked = 0, [], 0, []
    # Phase 1 - resolve and check every id BEFORE a single byte moves: the archive is
    # append-only, so archiving id #1 and then failing on id #2 would leave a block in the
    # archive that the plan never stubbed, and a retry would double it.
    for bid in a.ids:
        b = plan.block(bid)
        checks += 1
        if not b:
            fails.append(no_block(plan, bid))
            continue
        if b.state not in STUBBABLE:
            fails.append("%s is %s - only %s may be stubbed" % (bid, b.state, "/".join(STUBBABLE)))
            continue
        picked.append((bid, b.body))
    if fails:
        print("stub: 0 lines moved - a check failed; %s and the plan are untouched" % archive)
        return verdict("stub", checks, fails, payload={"archive": archive}, as_json=a.json, refused=True)
    # Phase 2 - one archive write for every block, then read it back whole and compare byte
    # for byte.  Any failure from here on puts the archive's prior bytes back, so a refusal
    # leaves both files as they were and a re-run never appends a block twice.
    if not os.path.exists(archive):
        write_text(archive, "# Archived plan blocks (full text; the plan keeps the stub)\n", plan.nl)
    before, nl = read_text(archive)
    after = before.rstrip("\n") + "\n\n" + "\n\n".join("\n".join(body) for _, body in picked) + "\n"
    write_text(archive, after, nl)
    back, _ = read_text(archive)
    if os.environ.get("PB_PLAN_TEST_READBACK_FAIL"):      # test seam: a read-back that differs
        back += "\n"
    moved = sum(len(body) for _, body in picked)
    if back != after:
        write_text(archive, before, nl)
        print("stub: 0 lines moved - the archive read back different; both files are untouched")
        return verdict("stub", checks, ["the archive %s did not read back byte for byte - its prior bytes "
                                        "are back, the plan is untouched" % archive],
                       payload={"archive": archive}, as_json=a.json, refused=True)
    # Phase 3 - the archive is known good; stub the plan.  The stub keeps the block's own
    # Status text - and so its own date - before the `archived by` clause (§3.3); a trailer stays
    # under the stub, where it sat, with its blanks.
    rel = os.path.relpath(os.path.abspath(archive), os.path.dirname(os.path.abspath(plan.path)))
    kept, warns = 0, []
    for bid, _ in picked:
        b = plan.block(bid)
        hs = b.field("Handoff")
        hline = plan.lines[hs[0]].split(":", 1)[1].strip() if hs else "(none recorded)"
        stub = [b.heading,
                "- Status: %s · archived by %s, %s" % (b.status_line.split(":", 1)[1].strip(), a.by, a.date),
                "- Handoff (one line): " + hline,
                "- Full block: %s § %s" % (rel, bid)]
        if b.trailer:
            kept += 1
            warns.append("%s's trailer (%s) stays in the plan under its stub, not archived" % (bid, trailer_note(b)))
        plan.lines[b.start:b.end] = stub + (plan.lines[b.last:b.end] if b.trailer else [""])
        plan.reparse()
    try:
        plan.save()
    except PlanWriteError as exc:
        write_text(archive, before, nl)
        return verdict("stub", checks, ["%s - the archive's prior bytes are back" % exc],
                       payload={"archive": archive}, as_json=a.json, refused=True)
    print("stub: %d lines moved, %d verified byte-for-byte -> %s" % (moved, moved, archive))
    return verdict("stub", checks, [], warns, payload={"archive": archive, "moved": moved, "verified": moved},
                   as_json=a.json, counts=[("stubbed", len(picked)), ("moved_lines", moved),
                                           ("verified_lines", moved), ("trailers_kept", kept)])


REG_KEY_RE = re.compile(r"^- (.+?):(?= |$)")
MARKUP = set('*`_[]{}<>|#"\\\t\n')


def register_key(line):
    """The WHOLE label of a `## Pipeline state` register line.

    The label runs to the first colon followed by a space or end of line, so `- D:T: builder
    ...` is the key `D:T` and not the key `D`.  Matched by bare prefix, `--set "D=1"` overwrote
    the `- D:T:` line and still printed `=== GO ===` (a fleet defect): of the register's
    failure modes this one DESTROYS DATA, so it is matched exactly and, where a key is
    ambiguous against an existing label, refused rather than rewritten.
    """
    m = REG_KEY_RE.match(line)
    return m.group(1).strip() if m else None


def key_error(key, val):
    """Why `<key>=<val>` cannot land as one register row, else None - judged before any line
    moves.  A key is a LABEL: non-empty, at most 64 characters, starting with a letter or a
    digit, free of markup, carrying no `: `, parentheses balanced; its value stays on one line.
    Such a key reads back through `register_key` as itself, so the row lands where it was meant
    to.  Without the predicate an `=` inside a value makes a key out of the prose before it, and
    the register grows a second copy of a live row."""
    if not key:
        return "is empty"
    if len(key) > 64:
        return "is %d characters (> 64)" % len(key)
    if not key[0].isalnum():
        return "starts with neither a letter nor a digit"
    bad = "".join(sorted(set(key) & MARKUP))
    if bad:
        return "carries markup %r" % bad
    if ": " in key or key.endswith(":"):
        return "carries `: `"
    depth = 0
    for ch in key:
        depth += (ch == "(") - (ch == ")")
        if depth < 0:
            return "has an unbalanced `)`"
    if depth:
        return "has an unbalanced `(`"
    if "\n" in val or "\r" in val:
        return "has a value that carries a line break"
    return None


def edit_register(plan, pairs):
    """Rewrites register rows in memory: (checks, fails, warns, counts)."""
    span = plan.section("## Pipeline state")
    if not span:
        return 1, ["no `## Pipeline state` section"], [], []
    lo, hi = span
    checks, fails, warns, replaced, appended = 0, [], [], 0, 0
    for pair in pairs:
        checks += 1
        if "=" not in pair:
            fails.append("'%s' is not <Key>=<value> - the call: %s %s" % (pair, PROG, CALLS["register"]))
            continue
        key, val = pair.split("=", 1)
        key = key.strip()
        why = key_error(key, val)
        if why:
            fails.append("--set '%s': the key %r %s - no line was rewritten" % (pair, key, why))
    if fails:
        return checks, fails, [], []
    for pair in pairs:
        key, val = pair.split("=", 1)
        key = key.strip()
        want = "- %s:" % key
        hit = next((i for i in range(lo, hi) if register_key(plan.lines[i]) == key), None)
        if hit is None:
            # The bare-prefix match this replaces would have silently rewritten one of these.
            # Refuse the whole call instead: an ambiguous key is a typo, and the register line
            # it would have eaten is unrecoverable.
            near = [register_key(plan.lines[i]) or plan.lines[i].strip()
                    for i in range(lo, hi) if plan.lines[i].startswith(want)]
            if near:
                fails.append("--set key '%s' is ambiguous against the existing register line"
                             " `- %s:` - no line was rewritten; pass its whole label" % (key, near[0]))
                continue
            end = hi
            while end > lo and not plan.lines[end - 1].strip():
                end -= 1
            plan.lines[end:end] = ["%s %s" % (want, val.strip())]
            hi += 1
            appended += 1
            warns.append("appended a new register row `- %s:` - there was no such key" % key)
        else:
            plan.lines[hit] = "%s %s" % (want, val.strip())
            replaced += 1
    plan.reparse()
    return checks, fails, warns, [("set", len(pairs)), ("replaced", replaced), ("appended", appended)]


def cmd_register(plan, a):
    checks, fails, warns, counts = edit_register(plan, a.set)
    saved = plan.save_if(fails)
    return verdict("register", checks, fails, warns, as_json=a.json, refused=not saved, counts=counts)


# ------------------------------------------------------------------------ the reads
def cmd_show(plan, a):
    """The block's start read (§4 Rule 1): the header without its Flow table, then the block."""
    b = plan.block(a.id)
    if not b:
        return verdict("show", 1, [no_block(plan, a.id)], as_json=a.json)
    head = plan.lines[:plan.header_end]
    span = plan.section("## Flow")
    left_out = 0
    if span:
        left_out = span[1] - span[0]
        head = head[:span[0]] + head[span[1]:]
    while head and not head[-1].strip():
        head.pop()
    body = list(b.lines)
    while body and not body[-1].strip():
        body.pop()
    text = "\n".join(head + [""] + body)
    if not a.json:
        print(text)
        print()
    return verdict("show", 2, [], payload={"id": b.id, "text": text} if a.json else {"id": b.id},
                   as_json=a.json, counts=[("header", len(head)), ("flow_left_out", left_out),
                                           ("block", len(body)), ("total", len(head) + 1 + len(body))])


def cmd_header(plan, a):
    rows, total = [], plan.header_end
    for name in HEADER_SECTIONS:
        span = plan.section(name)
        rows.append((name, (span[1] - span[0]) if span else 0))
    pipeline = dict(rows)["## Pipeline state"]
    if not a.json:
        for name, n in rows:
            print("%-20s %4d lines%s" % (name, n, "" if n else "   (absent)"))
        print("header total       %5d / %d lines" % (total, a.budget_header))
        print("pipeline state     %5d / %d lines" % (pipeline, a.budget_pipeline))
    rf = Refresh(plan) if plan.ladder else None
    if rf and not a.json:
        print("rating refresh     %5d / %d flags%s" % (rf.count, rf.threshold, "   (fires: no TM<n> pending)"
                                                     if rf.fires else ""))
    fails = []
    if total > a.budget_header:
        fails.append("header %d > budget %d" % (total, a.budget_header))
    if pipeline > a.budget_pipeline:
        fails.append("Pipeline state %d > budget %d" % (pipeline, a.budget_pipeline))
    if any(n == 0 for _, n in rows):
        fails.append("missing header section(s): %s" % ", ".join(n for n, c in rows if c == 0))
    return verdict("header", len(rows) + 2, fails, payload={"header": total, "pipeline": pipeline,
                                                            "refresh": [rf.count, rf.threshold] if rf else None},
                   as_json=a.json, counts=[("header", "%d/%d" % (total, a.budget_header)),
                                           ("pipeline", "%d/%d" % (pipeline, a.budget_pipeline))]
                   + ([("refresh", "%d/%d" % (rf.count, rf.threshold))] if rf else []))


def phase_of(lines, idx):
    for i in range(idx, -1, -1):
        if re.match(r"^#{1,2} (Phase\b|P\d)", lines[i]):
            return lines[i].lstrip("# ").split("—")[0].split("-")[0].strip()
    return "(no phase)"


SECTION_RE = re.compile(r"§|:\d|#L\d|\b(?:lines?|rows?|sections?|part|cell|heading|head|tail|range)\b", re.I)
PATH_RE = re.compile(r"[\w./-]*\w\.[A-Za-z]{1,5}(?![\w/])")
MAPPED_RE = re.compile(r"→|\bnone\s+[—-]|\bVerify\s*\d|\bV\d\b")


def file_lines(plan, tok, cache):
    """Lines in the file a `Read:` token names, looked up from the working folder and from the
    plan's folder upwards; None when no such file exists."""
    if tok in cache:
        return cache[tok]
    bases, d = [os.getcwd()], os.path.dirname(os.path.abspath(plan.path))
    for _ in range(5):
        bases.append(d)
        d = os.path.dirname(d)
    n = None
    for base in bases:
        p = os.path.join(base, tok)
        if os.path.isfile(p):
            with open(p, "rb") as fh:
                n = sum(1 for _ in fh)
            break
    cache[tok] = n
    return n


def whole_reads(plan, b, cache):
    """[(path, lines)] a block's `Read:` names whole although it runs over READ_CAP (§11)."""
    text = b.field_text("Read")
    out = []
    for item in re.split(r"\s\+\s", text or ""):
        if SECTION_RE.search(item):
            continue
        for tok in PATH_RE.findall(item.replace("`", " ")):
            n = file_lines(plan, tok, cache)
            if n and n > READ_CAP and (tok, n) not in out:
                out.append((tok, n))
    return out


def unmapped_deliver(b):
    """The `Deliver:` items no Verify step names: an item is mapped by `→ <step>` / `none - <why>`,
    or by a Verify step naming one of its files or backticked terms (§3.2)."""
    dtext, vtext = b.field_text("Deliver"), b.field_text("Verify")
    if dtext is None or vtext is None or re.match(r"\s*\"?none\b", vtext, re.I):
        return []
    out = []
    for item in [x.strip() for x in re.split(r"\s·\s|;\s", " ".join(dtext.split())) if x.strip()]:
        if MAPPED_RE.search(item):
            continue
        toks = re.findall(r"`([^`]+)`", item) + PATH_RE.findall(item)
        names = {t.strip() for t in toks} | {os.path.basename(t.strip().rstrip("/")) for t in toks}
        if any(len(n) >= 4 and n in vtext for n in names):
            continue
        out.append(item)
    return out


def repeated_lines(plan, blocks):
    """{line: [ids]} for a line of REPEAT_MIN or more characters held by three or more of
    `blocks` (§11: a shape repeated in three blocks is stated once, in Rules)."""
    seen = {}
    for b in blocks:
        mine = set()
        for i in range(b.start + 1, b.last):
            t = " ".join(plan.lines[i].split())
            if plan.mask[i] or len(t) < REPEAT_MIN or t.startswith(("- Status:", "- Handoff:")):
                continue
            mine.add(t)
        for t in mine:
            seen.setdefault(t, []).append(b.id)
    return {t: ids for t, ids in seen.items() if len(ids) >= 3}


def kinds_kept(plan):
    """{id: type} from the plan's Repo facts: `Kind kept: <id> = <TYPE> (<the kind its plan gave
    it>)`, one line per pre-v12 id whose plan gave it another kind than its letters say (§0).  Read
    there only - the same words anywhere else keep nothing."""
    span, out = plan.section("## Repo facts"), {}
    for i in range(*span) if span else ():
        if not plan.mask[i]:
            for m in KIND_KEPT_RE.finditer(plan.lines[i]):
                out[m.group(1)] = m.group(2)
    return out


FLOW_WORDS = {"prose": "prose, no table", "phase-table": "a phase table", "none": "absent (no `## Flow`)"}


def cmd_lint(plan, a):
    fails, warns, checks = [], [], 0
    L = plan.lines
    fence_counts = {}
    # v12's rules bind the blocks still to run.  A finished block (DONE, N/A, as block_state reads
    # it) that one of them would refuse is history: named on the HISTORY line, never a failure.  A
    # block whose state reads as nothing is held to the rules - it may be open.
    history, states, kept = {}, {}, kinds_kept(plan)
    ladder, unrated = plan.ladder, []
    checks += 1
    if ladder:
        checks += 1
        fails.extend(ladder.errors)
    if not plan.blocks:
        fails.append("no task blocks in %s - nothing to lint" % plan.path)
    seen = {}
    for b in plan.blocks:
        checks += 1
        if b.id in seen:
            fails.append("duplicate id %s (lines %d and %d)" % (b.id, seen[b.id] + 1, b.start + 1))
        seen[b.id] = b.start
        state = states[b.id] = block_state(b)
        bad = []
        if not b.status_line.startswith("- Status:"):
            bad.append("%s: first line is not `- Status:` (line %d)" % (b.id, b.start + 2))
        else:
            why = status_error(b.status_text)
            if why:
                bad.append("%s: %s (line %d)" % (b.id, why, b.status_index() + 1))
        if b.start > 0 and L[b.start - 1].strip():
            bad.append("%s: heading not preceded by a blank line (line %d)" % (b.id, b.start + 1))
        tag, typed = TAG_RE.search(b.heading), id_type(b.id)
        allowed = (kept[b.id],) if b.id in kept else typed[0] if typed is not None else None
        if tag and allowed is not None:
            new = [w for w in re.findall(r"[A-Z]+", tag.group(1)) if w in TYPES]
            if new and new[0] not in allowed and b.id in kept:
                bad.append("%s: typed **%s**, but its Repo facts keep it %s (`Kind kept: %s = %s`)"
                           % (b.id, tag.group(1), kept[b.id], b.id, kept[b.id]))
            elif new and new[0] not in allowed:
                bad.append("%s: typed **%s**, but its id makes it %s (PLAYBOOK §0; BUILDER, JUDGE and HUMAN "
                           "are read as aliases; a pre-v12 id keeps another kind only by a `Kind kept: %s = "
                           "<TYPE> (...)` line in Repo facts)" % (b.id, tag.group(1), " or ".join(allowed)
                                                                  or "no type (LEAD answers)", b.id))
        # §3.2: once the header carries a ladder, every block still to run carries a rung of it; a
        # finished block is history and never rated after the fact - not counted either.
        if state not in FINISHED and rated(b.id):
            if ladder:
                checks += 1
                why = rating_error(b.id, b.heading, ladder)
                if why:
                    bad.append(why)
            elif heading_rating(b.heading)[0] is None:
                unrated.append(b.id)
        hs = b.field("Handoff")
        if hs and (hs[1] - hs[0]) > 8:
            warns.append("%s: handoff is %d lines (max 8)" % (b.id, hs[1] - hs[0]))
        vs = b.field("Verify")
        if vs:
            # Per command fence: the text after its close, up to the next fence or the field's
            # end, carries its own `Pass:` and `Fail:` - one pair above or inside a fence never
            # covers it (§3.2).
            inner = [(o, e) for o, e in plan.spans if vs[0] <= o < vs[1]]
            if inner:
                fence_counts[b.id] = len(inner)
            for k, (o, e) in enumerate(inner):
                stop = inner[k + 1][0] if k + 1 < len(inner) else vs[1]
                tail = "\n".join(L[e:stop])
                checks += 1
                missing = [w for w, rx in (("Pass:", PASS_RE), ("Fail:", FAIL_RE)) if not rx.search(tail)]
                if missing:
                    bad.append("%s: the `Verify:` fence at line %d has no %s line of its own"
                               % (b.id, o + 1, "/".join(missing)))
        if bad and state in FINISHED:
            history[b.id] = state
        else:
            fails.extend(bad)
    # The Flow: a per-task table is checked row by row; prose, a phase table or no Flow is history
    # (Flow.kind).  In a per-task table a gate or phase row an old plan keeps names no block: history.
    flow, gates = plan.flow, []
    flow_ids = flow.ids()
    checks += 1
    fseen = set()
    rows = dict(flow.rows)
    for idx, fid in flow_ids if flow.kind == "per-task" else ():
        if not ID_START_RE.match(fid):
            gates.append(fid or "(empty)")
            continue
        if fid in fseen:
            fails.append("duplicate Flow row for %s (line %d)" % (fid, idx + 1))
        fseen.add(fid)
        b = plan.block(fid)
        if not b:
            fails.append("Flow row %s (line %d) has no block" % (fid, idx + 1))
            continue
        checks += 1
        cs = rows[idx]
        cell = clean(cs[flow.status_col()]) if len(cs) > flow.status_col() else ""
        off = None
        if state_of(cell) != states[fid]:
            off = "Flow row %s says %r, block says %r" % (fid, state_of(cell), states[fid])
        else:
            d1, d2 = DATE_RE.search(cell), DATE_RE.search(b.status_line)
            if d1 and d2 and d1.group(0) != d2.group(0):
                off = "Flow row %s date %s disagrees with block date %s" % (fid, d1.group(0), d2.group(0))
        if off and states[fid] in FINISHED:
            history.setdefault(fid, states[fid])
        elif off:
            fails.append(off)
    for b in plan.blocks if flow.kind == "per-task" else ():
        if b.id in fseen:
            continue
        if states[b.id] in FINISHED:
            history.setdefault(b.id, states[b.id])
        else:
            fails.append("block %s has no Flow row" % b.id)
    notes = []
    if history:
        notes.append("%d finished block(s) a v12 check would refuse, read as history, never a failure: %s%s"
                     % (len(history), " · ".join(list(history)[:6]), " ..." if len(history) > 6 else ""))
    if flow.kind != "per-task":
        notes.append("the Flow is %s - no block needs a row, `append` writes none" % FLOW_WORDS[flow.kind])
    if gates:
        names = sorted(set(gates))
        notes.append("%d Flow row(s) name no block - gate or phase rows: %s%s"
                     % (len(gates), " · ".join(names[:4]), " ..." if len(names) > 4 else ""))
    for lo, hi in plan.spans:
        checks += 1
        for i in range(lo, hi):
            if RUNTAG_RE.search(L[i]):
                fails.append("run tag inside a fenced block (line %d) - the tag goes on the line above" % (i + 1))
    # Only a TODO `TC<n>` (a compaction) discharges a budget: a bare `TC` is the contract.
    tc_todo = any(re.match(r"TC\d", id_tail(b.id)) and states[b.id] == "TODO" for b in plan.blocks)
    span = plan.section("## Pipeline state")
    pipeline = (span[1] - span[0]) if span else 0
    checks += 2
    # §2.3, §11 (v12.2): a TC<n> fires only past a budget by more than the margin, 100 lines by
    # default; `header` still measures against the budget itself, the compaction's target.
    if plan.header_end > a.budget_header + a.tc_margin and not tc_todo:
        fails.append("header %d > %d + %d and no TODO TC<n> block"
                     % (plan.header_end, a.budget_header, a.tc_margin))
    if pipeline > a.budget_pipeline + a.tc_margin and not tc_todo:
        fails.append("Pipeline state %d > %d + %d and no TODO TC<n> block"
                     % (pipeline, a.budget_pipeline, a.tc_margin))
    # §0, §11: the rating pass's refresh trigger - the same shape, in a plan with a ladder only.
    rf = Refresh(plan) if ladder else None
    if rf:
        checks += 1
        if rf.fires:
            fails.append(rf.red(plan))
        if rf.unstamped:
            ids = sorted(set(rf.unstamped), key=rf.unstamped.index)
            warns.append("%d carried flag(s) on open BUILD blocks carry no `[<from>, <date>]` stamp - %s (§0): %s%s"
                         % (len(rf.unstamped), "history, read by the pass %s, never counted" % rf.window()
                            if rf.since else "counted toward the rating refresh until the register's first "
                            "dated `%s:` line" % RATINGS_KEY, " · ".join(ids[:6]), " ..." if len(ids) > 6 else ""))
    root = os.path.dirname(plan.path) or "."
    for d in DIRS:
        checks += 1
        home = os.path.join(root, d)
        if os.path.exists(home) and not os.path.isdir(home):
            fails.append("%s exists but is not a directory" % home)
        elif not os.path.isdir(home):
            fails.append("missing %s/ beside the plan - git carries no empty folder: create it with a"
                         " 0-byte .gitkeep inside" % home)
    cur = next((phase_of(L, b.start) for b in plan.blocks if states[b.id] in ("TODO", "IN PROGRESS")), None)
    for b in plan.blocks:           # longer than a stub (5 lines at most), its trailer not counted
        if states[b.id] == "DONE" and cur and phase_of(L, b.start) != cur and (b.last - b.start) > 5:
            warns.append("%s: DONE outside the current phase and %d lines long - stub it (§2.6)"
                         % (b.id, b.last - b.start))
    # The authoring warnings read the open blocks only: a closed block is not rewritten.
    live, cache = [b for b in plan.blocks if states[b.id] in OPEN_STATES], {}
    for b in live:
        for tok, n in whole_reads(plan, b, cache):
            warns.append("%s: `Read:` names %s whole (%d lines > %d) - name a section or a line range"
                         % (b.id, tok, n, READ_CAP))
    for t, ids in sorted(repeated_lines(plan, live).items(), key=lambda kv: -len(kv[1])):
        warns.append("a line in %d blocks (%s%s): '%s' - state it once in Rules and cite it"
                     % (len(ids), " · ".join(ids[:4]), " ..." if len(ids) > 4 else "",
                        t if len(t) <= 72 else t[:72] + "..."))
    unmapped = [(b.id, n) for b in live for n in [len(unmapped_deliver(b))] if n]
    if unmapped:
        warns.append("`Deliver:` items no Verify step names (write `→ Verify <n>` or `none — <why>`): %s"
                     % " · ".join("%s %d" % kv for kv in unmapped))
    if unrated:
        # No ladder yet (every plan before its v12.1 adoption): counted, never red, so a project's
        # agents keep filing blocks between the bump and the adoption that rates them (§3.2).
        warns.append("%d open block(s) carry no rating - no `Models:` ladder in the header yet, so nothing is "
                     "red; the plan's v12.1 adoption writes the ladder and rates them (§3.2): %s%s"
                     % (len(unrated), " · ".join(unrated[:6]), " ..." if len(unrated) > 6 else ""))
    nfences = sum(fence_counts.values())
    return verdict("lint", checks, fails, warns,
                   payload={"blocks": len(plan.blocks), "flow_rows": len(flow_ids),
                            "verify_blocks": len(fence_counts), "verify_fences": nfences,
                            "fences_by_block": fence_counts, "history": list(history), "flow": flow.kind,
                            "gate_rows": gates, "ladder": [r[0] for r in ladder.rungs] if ladder else None,
                            "unrated": unrated, "refresh": [rf.count, rf.threshold] if rf else None},
                   as_json=a.json, history=" · ".join(notes),
                   counts=[("blocks", len(plan.blocks)), ("flow_rows", len(flow_ids)),
                           ("verify_fences", nfences), ("history", len(history)), ("flow", flow.kind),
                           ("header", "%d/%d" % (plan.header_end, a.budget_header)),
                           ("pipeline", "%d/%d" % (pipeline, a.budget_pipeline)),
                           ("ladder", "%d rungs" % len(ladder.rungs) if ladder else "none")]
                   + ([("refresh", "%d/%d" % (rf.count, rf.threshold))] if rf else []))


# ------------------------------------------------------------------------ selftest
FIXTURE = """# M12.1 - Implementation plan (fixture)

Intro line.

## Flow
| Status | Task | Goal | Order | Stance |
|---|---|---|---|---|
| DONE (2026-01-02) | M12.1-TP | Red-team the plan | FIRST | JUDGE |
| DONE (2026-01-02) | M12.1-T1 | Open the widget | AFTER TP | BUILDER |
| TODO | M12.1-T2a | Extend the widget | AFTER T1 | BUILD |
| TODO | M12.1-T2b | Extend the widget, part two | AFTER T2a | BUILDER |
| TODO | M12.1-D1 | Debug the widget seam | AFTER T2a | BUILD |

## Rules
Header budget: 600 / 60.

## Superseded / retired
- none.

## Repo facts
- One widget, one seam.

## Pipeline state
- Next task: M12.1-T2a
- Counters: T=2 · D=1 · V=0 · TC=0
- D:T: builder - milestone 0/0 - gates 0 - DBG 0
- Loop count: not measured

# Phase 0 - gates

## M12.1-TP · Red-team the plan · **JUDGE** · (FIRST)
- Status: DONE (2026-01-02)
- Handoff: DONE - nothing to report.

# Phase A - build

## M12.1-T1 · Open the widget · **BUILDER** · (AFTER M12.1-TP)
- Status: DONE (2026-01-02)
- Deliver: widget.py
- Verify: `python3 widget.py --selftest` - Pass: GO · Fail: otherwise.
- Handoff: DONE - widget.py exists.

## M12.1-T2a · Extend the widget · **BUILD** · (AFTER M12.1-T1)
- Status: TODO
- Deliver: widget.py grows a seam
- Handoff: <placeholder>

## M12.1-T2b · Extend the widget, part two · **BUILDER** · (AFTER M12.1-T2a)
- Status: TODO
- Deliver: `widget.py` grows its second half → Verify 1, 2
- Verify:
  1. the check
     [NOT RUN - for you]
     ```
     python3 widget.py --check
     ```
     Pass: GO, 0 failed.
     Fail: NO-GO.
  2. the seam
     [NOT RUN - for you]
     ```
     python3 widget.py --seam
     ```
     Pass: the seam prints OK.
     Fail: anything else.
- Handoff: <placeholder>

## M12.1-D1 · Debug the widget seam · **BUILD** · (raised by M12.1-TP)
- Status: TODO
- Created by: M12.1-TP
- Deliver: the fix in `widget.py`
- Verify:
  [NOT RUN - for you]
  1. ```
     python3 widget.py --fixed
     ```
     Pass: GO.
     Fail: NO-GO.
- Handoff: <placeholder>
"""

NEWBLOCK = """## M12.1-D2 · Debug the second seam · (raised by M12.1-T1)
- Status: TODO
- Deliver: the second fix
- Handoff: <placeholder>
"""


def run(argv):
    """One call into main(), its output captured: (exit code, printed text)."""
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        rc = main(argv)
    return rc, buf.getvalue()


def cmd_selftest(_plan, a):
    root = tempfile.mkdtemp(prefix="pbplan_selftest_")
    checks, fails, plants = 0, [], 0
    kept_env = {k: os.environ.pop(k) for k in list(os.environ) if k.startswith("PB_PLAN_TEST_")}
    fast = {"PB_PLAN_TEST_BACKOFF": "0.001"}       # the retry arms wait 1 ms, not 50, per attempt

    def check(label, ok, out=""):
        nonlocal checks
        checks += 1
        if not ok:
            tail = " | ".join(l for l in out.strip().splitlines() if l.startswith(("FAIL", "WARN", "===")))[-300:]
            fails.append(label + (" - got: " + tail if tail else ""))
        return ok

    def plant(label, ok, out=""):
        nonlocal plants
        plants += 1
        return check("PLANT " + label, ok, out)

    try:
        mdir = os.path.join(root, "milestones", "m12")
        for d in DIRS:
            os.makedirs(os.path.join(mdir, d))
            write_text(os.path.join(mdir, d, ".gitkeep"), "", "\n")
        path = os.path.join(mdir, "m12_implementation_plan.md")
        archive = os.path.join(mdir, "plan_archive.md")
        base = ["--plan", path]

        def go(*args):
            return run(base + list(args))

        def text():
            return read_text(path)[0]

        def reset(t=None):
            write_text(path, FIXTURE if t is None else t, "\n")

        def untouched(label, t, args, watch=(), env=None, want=None):
            """A NO-GO write must change nothing: run it, then compare every watched file.  `env`
            (the test seams) applies to the call alone, after the fixture is written."""
            reset(t)
            keep = [(p, read_text(p)[0]) for p in (path,) + tuple(watch)]
            os.environ.update(env or {})
            try:
                rc, out = go(*args)
            finally:
                for k in env or {}:
                    os.environ.pop(k, None)
            check("no-partial-write: %s NO-GO" % label, rc != 0 and (want is None or want in out), out)
            for p, was in keep:
                check("no-partial-write: %s left %s byte-identical" % (label, os.path.basename(p)),
                      read_text(p)[0] == was)

        # --- the clean fixture is GO first, before any plant --------------------------------
        reset()
        rc, out = go("lint")
        check("the clean fixture lints GO, before any plant: 5 blocks, 5 rows, 3 fences, no ladder, one warning - "
              "its 3 unrated open blocks counted", rc == 0 and "blocks=5 · flow_rows=5 · verify_fences=3" in out
              and "warnings=1" in out and "WARN: 3 open block(s) carry no rating" in out and "ladder=none" in out, out)

        # --- what it reads ---------------------------------------------------------------
        check("a block heading: `M1b-` `2M2-` `M4.5-` `M0-TH-a` are blocks; `Ratified at M1b-TP`, `M1b notes` are not",
              all(BLOCK_RE.match("## %s · x" % h) for h in ("M1b-T1", "2M2-T1", "M4.5-TE-V12", "M0-TH-a"))
              and not any(BLOCK_RE.match(h) for h in ("## Ratified at M1b-TP", "## M1b notes")))
        reset(FIXTURE.replace("M12.1-", "M12b-"))
        rc, out = go("lint")
        check("a letter-suffixed milestone (`M12b-`) lints GO with the same 5 blocks", rc == 0 and "blocks=5" in out, out)
        fenced = ["## M9-T1 · one", "- Status: TODO", "- Verify:", "  1. ```bash", "     true", "     ```",
                  "     Pass: x. Fail: y.", "", "```markdown", "## M9-D<n> · a template in a column-0 fence",
                  "- Status: TODO", "```", "", "## M9-T2 · two", "- Status: TODO", "  ```",
                  "  an indented fence the author never closed", "## M9-T3 · three: column 0 ends it",
                  "- Status: TODO", "~~~", "```", "## M9-T9 · inside a tilde fence a backtick line closes nothing",
                  "~~~", "", "## M9-T4 · four", "- Status: TODO"]
        got = [b.id for b in find_blocks(fenced, fence_mask(fenced))]
        check("the fence rule: a fence opens after a list marker, a column-0 fence hides its heading, a column-0 "
              "line ends an indented fence, a backtick line never closes a tilde fence", got == ["M9-T1", "M9-T2", "M9-T3", "M9-T4"], str(got))

        # --- show --------------------------------------------------------------------------
        reset()
        rc, out = go("show", "M12.1-T2b")
        check("show: the header without its Flow table, then the block", rc == 0 and "## Rules" in out
              and "## Pipeline state" in out and "## M12.1-T2b" in out and "| M12.1-TP |" not in out
              and "## M12.1-T1 " not in out and "block=" in out and "flow_left_out=" in out, out)
        rc, out = go("show", "T2b")
        check("show with a local id: NO-GO naming the whole id", rc == 1 and "M12.1-T2b" in out, out)

        # --- status and its stamps -------------------------------------------------------------
        os.environ["PB_PLAN_TEST_CLOCK"] = "09:15"
        rc, out = go("status", "M12.1-T2a", "IN PROGRESS")
        t = text()
        check("status IN PROGRESS: stamped with the date and time, Status line and Flow cell",
              rc == 0 and "- Status: IN PROGRESS (%s 09:15)" % TODAY in t
              and "| IN PROGRESS (%s 09:15) | M12.1-T2a" % TODAY in t, out)
        os.environ["PB_PLAN_TEST_CLOCK"] = "10:40"
        rc, out = go("status", "M12.1-T2a", "DONE")
        t = text()
        check("status DONE: the date, the time and the start time read from the IN PROGRESS stamp",
              rc == 0 and "- Status: DONE (%s 10:40, started 09:15)" % TODAY in t
              and "| DONE (%s 10:40, started 09:15) | M12.1-T2a" % TODAY in t, out)
        reset(FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows",
                              "- Status: IN PROGRESS (2030-01-02 23:50)\n- Deliver: widget.py grows", 1)
              .replace("| TODO | M12.1-T2a |", "| IN PROGRESS (2030-01-02 23:50) | M12.1-T2a |", 1))
        rc, out = go("status", "M12.1-T2a", "DONE (%s)" % TODAY)
        check("status DONE typed with today's date: stamped, a start on another day keeps its date",
              rc == 0 and "DONE (%s 10:40, started 2030-01-02 23:50)" % TODAY in text(), out)
        reset()
        rc, out = go("status", "M12.1-T2a", "DONE (2026-01-03)")
        check("status with a past date typed: written as typed", rc == 0 and "- Status: DONE (2026-01-03)\n" in text(), out)
        os.environ.pop("PB_PLAN_TEST_CLOCK", None)
        for bad in ("ALMOST DONE", "TODO — BLOCKED on the lead", "DONE — waived", "FAIT (2026-01-03)",
                    "IN PROGRESS — nearly", "DEFERRED until later"):
            untouched("status refuses %r, off §3.3's closed set" % bad, None, ["status", "M12.1-T2b", bad])
        untouched("status with a local id", None, ["status", "T2b", "DONE (2026-01-03)"])
        untouched("status on a block with no Flow row",
                  FIXTURE + "\n## M12.1-T9 · Orphan · **BUILD** · (LAST)\n- Status: TODO\n",
                  ["status", "M12.1-T9", "DONE (2026-01-03)"])
        untouched("status on a block whose first line is not `- Status:`",
                  FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows", "- Deliver: x\n- Status: TODO\n"
                                  "- Deliver: widget.py grows", 1), ["status", "M12.1-T2a", "DONE (2026-01-03)"])
        crlf = os.path.join(mdir, "m12c_implementation_plan.md")
        write_text(crlf, FIXTURE, "\r\n")
        rc, out = run(["--plan", crlf, "status", "M12.1-T2a", "DONE (2026-01-03)"])
        with open(crlf, "rb") as fh:
            raw = fh.read()
        check("CRLF line endings kept through a write", rc == 0 and b"\n" not in raw.replace(b"\r\n", b""), out)
        if os.name == "posix":
            reset()
            os.chmod(path, 0o644)
            go("status", "M12.1-T2a", "DONE (2026-01-03)")
            check("the file's own mode kept through a write (0644, not the temp's 0600)",
                  stat.S_IMODE(os.stat(path).st_mode) == 0o644, oct(os.stat(path).st_mode))

        # --- close: status, handoff and register in one write -------------------------------
        reset()
        os.environ["PB_PLAN_TEST_CLOCK"] = "11:05"
        rc, out = go("close", "M12.1-T2b", "--status", "DONE", "--text", "Done: the seam.\nNext: M12.1-D1",
                     "--set", "Next task=M12.1-D1", "--set", "Loop count=1 / 1")
        os.environ.pop("PB_PLAN_TEST_CLOCK", None)
        t = text()
        check("close: status stamped, handoff and two register rows land in one call",
              rc == 0 and "- Status: DONE (%s 11:05)" % TODAY in t and "- Handoff: Done: the seam.\n  Next: M12.1-D1" in t
              and "- Next task: M12.1-D1" in t and "- Loop count: 1 / 1" in t, out)
        untouched("close with a malformed --set key", None,
                  ["close", "M12.1-T2b", "--status", "DONE (2026-01-03)", "--text", "x", "--set", "Next task: a=b"])
        untouched("close with a status off the closed set", None,
                  ["close", "M12.1-T2b", "--status", "FINISHED", "--text", "x"])
        untouched("close with a handoff over 8 lines", None,
                  ["close", "M12.1-T2b", "--status", "DONE (2026-01-03)", "--text", "\n".join("l%d" % i for i in range(9))])

        # --- handoff and flag ----------------------------------------------------------------
        reset()
        rc, out = go("handoff", "M12.1-T2a", "--text", "DONE - the seam is open.\n  - Next: T2b")
        check("handoff replaces the placeholder, continuation indented",
              rc == 0 and "- Handoff: DONE - the seam is open.\n  - Next: T2b\n" in text(), out)
        untouched("handoff over the 8-line bar", None,
                  ["handoff", "M12.1-T2b", "--text", "\n".join("l%d" % i for i in range(9))])
        untouched("handoff with a missing --file", None,
                  ["handoff", "M12.1-T2b", "--file", os.path.join(root, "does_not_exist.md")])
        reset()
        rc, out = go("flag", "M12.1-T2b", "watch the seam", "--from", "M12.1-T2a")
        rc2, out2 = go("flag", "M12.1-T2b", "second", "--from", "M12.1-TP")
        check("flag creates `- Carried flags:` under the Status line, then appends to it",
              rc == 0 and rc2 == 0 and "- Status: TODO\n- Carried flags: [M12.1-T2a, %s] watch the seam · [M12.1-TP, %s] second"
              % (TODAY, TODAY) in text(), out + out2)

        # --- append: block, type, Flow row, counter ------------------------------------------
        reset()
        nb = os.path.join(root, "newblock.md")
        write_text(nb, NEWBLOCK, "\n")
        rc, out = go("append", nb, "--after", "M12.1-T2a", "--goal", "Second fix")
        t = text()
        check("append: block, Flow row and `D=2` in one call; its type written from the id (**BUILD**)",
              rc == 0 and "## M12.1-D2 · Debug the second seam · **BUILD** · (raised by M12.1-T1)" in t
              and "| TODO | M12.1-D2 | Second fix | AFTER M12.1-T2a | BUILD |" in t and "D=2" in t, out)
        rc, out = go("lint")
        check("lint GO after the append", rc == 0, out)
        split = os.path.join(root, "split.md")
        write_text(split, NEWBLOCK.replace("M12.1-D2 · Debug the second seam", "M12.1-T2c · Extend, part three"), "\n")
        rc, out = go("append", split, "--after", "M12.1-T2b")
        t = text()
        check("append a split (T2c beside T2a, T2b): no new number - `T=2` stays",
              rc == 0 and "## M12.1-T2c" in t and "- Counters: T=2 · D=2" in t, out)
        untouched("append onto an anchor with no Flow row",
                  FIXTURE + "\n## M12.1-T9 · Orphan anchor · **BUILD** · (LAST)\n- Status: TODO\n",
                  ["append", nb, "--after", "M12.1-T9"])
        untouched("append with no `D=<n>` counter", FIXTURE.replace("- Counters: T=2 · D=1 ·", "- Counters: T=2 ·", 1),
                  ["append", nb, "--after", "M12.1-T2a"])
        offset = os.path.join(root, "offset.md")
        write_text(offset, NEWBLOCK.replace("M12.1-D2", "M12.1-D3").replace("- Status: TODO", "- Status: TODO — waiting"), "\n")
        untouched("append a block whose Status is off the closed set", None, ["append", offset, "--after", "M12.1-T2a"])
        untouched("append with a missing block file", None,
                  ["append", os.path.join(root, "does_not_exist.md"), "--after", "M12.1-T2a"])
        reset(FIXTURE.replace("- Counters: T=2 · D=1 ·", "- Counters: T=2 ·", 1))
        rc, out = go("append", nb, "--after", "M12.1-T2a")
        check("the counter refusal names the register call that fixes it",
              rc == 1 and "register --set \"Counters=" in out, out)

        # --- move ------------------------------------------------------------------------------
        reset()
        rc, out = go("move", "M12.1-D1", "--after", "M12.1-T2a")
        rc2, out2 = go("move", "M12.1-D1", "--after", "M12.1-T2b")
        check("move there and back: the plan byte-identical", rc == 0 and rc2 == 0 and text() == FIXTURE, out + out2)
        untouched("move a block after itself", None, ["move", "M12.1-D1", "--after", "M12.1-D1"])

        # --- stub --------------------------------------------------------------------------------
        reset()
        rc, out = go("stub", "M12.1-T1", "--by", "M12.1-T2a")
        t = text()
        check("stub: the stub shape, the block's own date kept before `archived by`",
              rc == 0 and "- Status: DONE (2026-01-02) · archived by M12.1-T2a, %s\n- Handoff (one line): DONE - "
              "widget.py exists.\n- Full block: plan_archive.md § M12.1-T1" % TODAY in t
              and "widget.py --selftest" not in t, out)
        check("stub: the full block in the archive", "- Verify: `python3 widget.py --selftest`" in read_text(archive)[0])
        rc, out = go("lint")
        check("lint GO after the stub", rc == 0, out)
        untouched("stub of a TODO block", None, ["stub", "M12.1-T2b", "--by", "M12.1-TP"], watch=(archive,))
        untouched("stub of a DONE block beside a TODO one", None,
                  ["stub", "M12.1-TP", "M12.1-T2b", "--by", "M12.1-T1"], watch=(archive,))
        untouched("stub whose archive reads back different", None, ["stub", "M12.1-TP", "--by", "M12.1-T1"],
                  watch=(archive,), env={"PB_PLAN_TEST_READBACK_FAIL": "1"}, want="prior bytes are back")
        untouched("stub whose plan write fails after the archive write", None,
                  ["stub", "M12.1-TP", "--by", "M12.1-T1"], watch=(archive,),
                  env=dict(fast, PB_PLAN_TEST_REPLACE_SKIP="1", PB_PLAN_TEST_REPLACE_FAIL="%d" % REPLACE_ATTEMPTS),
                  want="prior bytes are back")

        # --- a trailer: the `>` lines and `---` rules under a block's body (§10's commit-gate banner) --
        # Not the block's: every call leaves it where it sits.  A `>` line glued to the body is the body's.
        banner = "> **Commit gate (lead) — Phase 0:** `git push` - nothing else moves.\n> A banner's second line."
        tp_end = "- Handoff: DONE - nothing to report.\n"
        gated_tp = FIXTURE.replace(tp_end + "\n# Phase A", tp_end + "\n" + banner + "\n\n# Phase A", 1)
        tail = "\n\n" + banner + "\n\n# Phase A"

        def stub_tp(t):
            reset(t)
            if os.path.exists(archive):
                os.remove(archive)
            rc, out = go("stub", "M12.1-TP", "--by", "M12.1-T1")
            return rc, out, text(), read_text(archive)[0] if os.path.exists(archive) else ""

        rc, out, t, arch = stub_tp(gated_tp)
        plant("stub: a two-line banner after a blank stays in the plan under the stub, where it sat; the "
              "archive gets the body alone", rc == 0 and "- Full block: plan_archive.md § M12.1-TP" + tail in t
              and "Commit gate" not in arch and tp_end in arch and "trailers_kept=1" in out
              and "WARN: M12.1-TP's trailer (> **Commit gate" in out, out)
        rc, out = go("lint")
        plant("lint after that stub: GO, no warning but the unrated count - a stub holding a banner is still a stub",
              rc == 0 and "warnings=1" in out and "WARN: 3 open block(s) carry no rating" in out, out)
        rc, out, t, arch = stub_tp(FIXTURE.replace(tp_end + "\n# Phase A", tp_end + "\n---\n\n# Phase A", 1))
        plant("a `---` rule under a block is its trailer: stub leaves it under the stub",
              rc == 0 and "- Full block: plan_archive.md § M12.1-TP\n\n---\n\n# Phase A" in t and "---" not in arch, out)
        glued = "> the handoff's own closing quote, no blank above it"
        rc, out, t, arch = stub_tp(FIXTURE.replace(tp_end, tp_end + glued + "\n", 1))
        plant("a `>` line glued to the Handoff is the block's: stub archives it, the plan keeps none of it",
              rc == 0 and glued in arch and glued not in t and "trailers_kept=0" in out, out)
        reset(gated_tp)
        rc, out = go("move", "M12.1-D1", "--after", "M12.1-TP")
        t = text()
        plant("move after the block holding a banner: the block goes in above it, the banner stays last; a WARN "
              "names it", rc == 0 and tp_end + "\n## M12.1-D1 " in t and "- Handoff: <placeholder>" + tail in t
              and "above M12.1-TP's trailer (> **Commit gate" in out, out)
        rc, out = go("move", "M12.1-D1", "--after", "M12.1-T2b")
        plant("move the block now holding the banner: the banner stays where it sat - there and back, the plan "
              "byte-identical", rc == 0 and text() == gated_tp and "M12.1-D1's trailer" in out, out)
        reset(gated_tp)
        rc, out = go("append", nb, "--after", "M12.1-TP", "--goal", "Second fix")
        t = text()
        plant("append after the block holding a banner: the new block goes in above it, the banner stays last; "
              "a WARN names it", rc == 0 and tp_end + "\n## M12.1-D2 " in t and "- Handoff: <placeholder>" + tail in t
              and "above M12.1-TP's trailer (> **Commit gate" in out, out)
        reset(gated_tp)
        rc, out = go("close", "M12.1-TP", "--status", "DONE (2026-01-02)", "--text", "DONE - rewritten.")
        plant("close the block holding a banner: its handoff rewritten, the banner kept under it",
              rc == 0 and "- Handoff: DONE - rewritten." + tail in text(), out)
        reset(gated_tp.replace(tp_end, "", 1))
        rc, out = go("handoff", "M12.1-TP", "--text", "DONE - written.")
        plant("handoff on a block with no Handoff field and a banner: the field lands at the body's end, above it",
              rc == 0 and "- Status: DONE (2026-01-02)\n- Handoff: DONE - written." + tail in text(), out)

        # --- M0-D23: the trailer starts at the first section line under the last field - a `###`
        # heading, a quote or a rule after a blank - whatever follows it: the next lot's heading, its
        # intro, a closing note, the gate's own fence.  A `###` with fields below it is the block's.
        lot = "\n\n### Lot 0b - the second lot\n\nThe lot's intro line."
        lotted = FIXTURE.replace(tp_end + "\n# Phase A", tp_end + "\n" + banner + lot + "\n\n# Phase A", 1)
        lot_tail = "\n\n" + banner + lot + "\n\n# Phase A"
        reset(lotted)
        rc, out = go("close", "M12.1-TP", "--status", "DONE (2026-01-02)", "--text", "DONE - rewritten.")
        plant("close on a block whose gate is followed by the next lot's ### heading and intro: all kept under "
              "the new handoff", rc == 0 and "- Handoff: DONE - rewritten." + lot_tail in text(), out)
        rc, out, t, arch = stub_tp(lotted)
        plant("stub on it: the gate, the ### heading and the intro stay in the plan under the stub, none archived",
              rc == 0 and "- Full block: plan_archive.md § M12.1-TP" + lot_tail in t and "Lot 0b" not in arch
              and "Commit gate" not in arch and "trailers_kept=1" in out, out)
        reset(lotted)
        rc, out = go("move", "M12.1-D1", "--after", "M12.1-TP")
        plant("move after it: the block goes in above the gate; the gate and the lot heading stay below it",
              rc == 0 and "- Handoff: <placeholder>" + lot_tail in text()
              and "above M12.1-TP's trailer (> **Commit gate" in out, out)
        rc, out = go("move", "M12.1-D1", "--after", "M12.1-T2b")
        plant("move it back: the gate and the lot heading never moved - the plan byte-identical",
              rc == 0 and text() == lotted, out)
        reset(lotted)
        rc, out = go("append", nb, "--after", "M12.1-TP", "--goal", "Second fix")
        plant("append after it: the new block lands above the gate, in the gate's lot, not under the next heading",
              rc == 0 and tp_end + "\n## M12.1-D2 " in text() and "- Handoff: <placeholder>" + lot_tail in text(), out)
        reset(FIXTURE.replace(tp_end + "\n# Phase A", tp_end + lot[1:] + "\n\n# Phase A", 1))
        rc, out = go("handoff", "M12.1-TP", "--text", "DONE - written.")
        plant("a ### heading under the last field with no gate above it: the handoff stops above the heading",
              rc == 0 and "- Handoff: DONE - written." + lot + "\n\n# Phase A" in text(), out)
        note = "\n> **Commit gate (lead) — M12.1 close.** `git push`\n\n---\n\n*Authored by the fixture.*\n"
        reset(FIXTURE + note)
        rc, out = go("close", "M12.1-D1", "--status", "DONE (2026-01-03)", "--text", "DONE - fixed.")
        plant("close on the last block, its gate followed by a rule and a closing line: all three kept",
              rc == 0 and text().endswith("- Handoff: DONE - fixed.\n" + note), out)
        if os.path.exists(archive):
            os.remove(archive)
        rc, out = go("stub", "M12.1-D1", "--by", "M12.1-TP")
        arch = read_text(archive)[0] if os.path.exists(archive) else ""
        plant("stub on it: the gate, the rule and the closing line stay in the plan under the stub, none archived",
              rc == 0 and text().endswith("- Full block: plan_archive.md § M12.1-D1\n" + note)
              and "Authored" not in arch and "trailers_kept=1" in out, out)
        fenced = "> **Commit gate (lead) — Phase 0:** run it:\n\n```bash\ngit push\n```"
        reset(FIXTURE.replace(tp_end + "\n# Phase A", tp_end + "\n" + fenced + "\n\n# Phase A", 1))
        rc, out = go("handoff", "M12.1-TP", "--text", "DONE - written.")
        plant("a gate followed by its own fence at column 0: the handoff stops above the gate, the fence kept",
              rc == 0 and "- Handoff: DONE - written.\n\n" + fenced + "\n\n# Phase A" in text(), out)
        own = "### The widget's own context\nA line of the block's own.\n"
        owned = FIXTURE.replace("- Deliver: widget.py\n", "- Deliver: widget.py\n" + own, 1)
        reset(owned)
        rc, out = go("handoff", "M12.1-T1", "--text", "DONE - rewritten.")
        plant("a block's own ### with its fields below it: the block's - its Handoff replaced, the ### in place",
              rc == 0 and "- Deliver: widget.py\n" + own + "- Verify:" in text()
              and "- Handoff: DONE - rewritten." in text() and "widget.py exists" not in text(), out)
        reset(owned)
        if os.path.exists(archive):
            os.remove(archive)
        rc, out = go("stub", "M12.1-T1", "--by", "M12.1-TP")
        arch = read_text(archive)[0] if os.path.exists(archive) else ""
        plant("stub on it: the own ### archived with its block, none of it left in the plan",
              rc == 0 and own in arch and "own context" not in text() and "trailers_kept=0" in out, out)
        mid = "> **Ordering note:** between two fields, the block's own."
        between = FIXTURE.replace("- Status: DONE (2026-01-02)\n" + tp_end, "- Status: DONE (2026-01-02)\n- Carried "
                                  "flags: · [M12.1-T1, 2026-01-01] one\n\n" + mid + "\n\n" + tp_end, 1)
        reset(between)
        rc, out = go("flag", "M12.1-TP", "two", "--from", "M12.1-T1")
        plant("flag on a field with a quote after a blank below it: the entry lands on the field, the quote untouched",
              rc == 0 and "- Carried flags: · [M12.1-T1, 2026-01-01] one · [M12.1-T1, " in text()
              and "\n" + mid + "\n" in text(), out)
        rc, out, t, arch = stub_tp(between)
        plant("stub: a quote between two fields is the block's - archived with it, the stub's handoff line read",
              rc == 0 and mid in arch and mid not in t and "- Handoff (one line): DONE - nothing to report." in t, out)

        # --- register ----------------------------------------------------------------------------
        reset()
        rc, out = go("register", "--set", "Next task=M12.1-T2b", "--set", "Counters=T=2 · D=1 · V=1 · TC=0",
                     "--set", "Awaiting lead=none")
        t = text()
        check("register: rows rewritten in place, a value with `=` round-trips, a new key appended and named",
              rc == 0 and "- Next task: M12.1-T2b" in t and "- Counters: T=2 · D=1 · V=1 · TC=0" in t
              and t.count("- Awaiting lead: none") == 1 and "appended=1" in out and "Awaiting lead" in out, out)
        reset()
        rc, out = go("register", "--set", "D=1")
        check("register --set D=1 is refused: it must not eat the `- D:T:` line",
              rc == 1 and text() == FIXTURE, out)
        rc, out = go("register", "--set", "D:T=builder - milestone 1/1")
        check("... and the whole label `D:T` is accepted", rc == 0 and "- D:T: builder - milestone 1/1" in text(), out)
        for kv in ("=x", "K" * 65 + "=x", "-Next=x", "Next *task*=x", "Next `task`=x", "Next task: a=b",
                   "Loop (count=x", "Loop count)=x", "no equals sign", "Next task=a\nb"):
            untouched("register refuses the malformed pair %r" % kv, None, ["register", "--set", kv])
        untouched("register: one malformed pair refuses the whole call", None,
                  ["register", "--set", "Next task=M12.1-T2b", "--set", "bogus-pair"])

        # --- lint: every rule planted and shown red ------------------------------------------------
        def red(label, mutated, want, *extra):
            reset(mutated)
            rc, out = go("lint", *extra)
            plant(label, rc == 1 and want in out, out)

        red("a Flow row's state off its block", FIXTURE.replace("| TODO | M12.1-T2b |", "| DONE | M12.1-T2b |", 1),
            "Flow row M12.1-T2b says")
        red("a Flow row's date off its open block",
            FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows", "- Status: IN PROGRESS (2026-01-02 09:00)\n"
                            "- Deliver: widget.py grows", 1).replace("| TODO | M12.1-T2a |", "| IN PROGRESS (2026-01-05 09:00) | M12.1-T2a |", 1),
            "date 2026-01-05 disagrees")
        red("a run tag inside a fence", FIXTURE.replace("     python3 widget.py --check\n",
                                                        "     [ALREADY RUN - PASS] python3 widget.py --check\n", 1),
            "run tag inside a fenced block")
        red("a block without a Flow row", FIXTURE + "\n## M12.1-T4 · Orphan · **BUILD** · (LAST)\n- Status: TODO\n",
            "block M12.1-T4 has no Flow row")
        red("a Flow row without a block", FIXTURE.replace("| TODO | M12.1-D1 |", "| TODO | M12.1-D9 | x | x | x |\n| TODO | M12.1-D1 |", 1),
            "Flow row M12.1-D9")
        red("a first line that is not `- Status:`", FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows",
                                                                   "- Deliver: x\n- Status: TODO\n- Deliver: widget.py grows", 1),
            "M12.1-T2a: first line is not")
        red("a heading with no blank line above", FIXTURE.replace("\n\n## M12.1-D1", "\n## M12.1-D1", 1),
            "M12.1-D1: heading not preceded")
        red("a duplicate id", FIXTURE + "\n## M12.1-D1 · Duplicate · **BUILD** · (LAST)\n- Status: TODO\n", "duplicate id M12.1-D1")
        red("a status off §3.3's closed set (a stall in prose)",
            FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows", "- Status: TODO — BLOCKED on the lead\n"
                            "- Deliver: widget.py grows", 1).replace("| TODO | M12.1-T2a |", "| TODO — BLOCKED on the lead | M12.1-T2a |", 1),
            "M12.1-T2a: 'TODO — BLOCKED on the lead'")
        red("an open block in another language (`À FAIRE`)",
            FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows", "- Status: À FAIRE\n- Deliver: widget.py grows", 1),
            "M12.1-T2a: 'À FAIRE' is off")
        red("an open block in bold (`**TODO**`)",
            FIXTURE.replace("- Status: TODO\n- Deliver: widget.py grows", "- Status: **TODO**\n- Deliver: widget.py grows", 1),
            "M12.1-T2a: '**TODO**' is off")
        red("an open block deferred in bold, with notes",
            FIXTURE.replace("- Status: TODO\n- Created by: M12.1-TP", "- Status: **DEFERRED (wake: the lead)** - parked\n"
                            "- Created by: M12.1-TP", 1).replace("| TODO | M12.1-D1 |", "| DEFERRED (wake: the lead) | M12.1-D1 |", 1),
            "M12.1-D1: '**DEFERRED (wake: the lead)** - parked' is off")
        red("a block in a word no state starts (`OPEN (…)`) is held to the rules",
            FIXTURE.replace("- Status: TODO\n- Deliver: `widget.py` grows", "- Status: OPEN (2026-01-02)\n- Deliver: `widget.py` grows", 1),
            "M12.1-T2b: 'OPEN (2026-01-02)' is off")
        red("a Status line outranks the heading: a TODO block whose heading says DONE",
            FIXTURE.replace("## M12.1-T2a · Extend the widget · **BUILD** · (AFTER M12.1-T1)\n- Status: TODO\n",
                            "## M12.1-T2a · Extend the widget · **BUILD** · **DONE (2026-01-02)**\n- Status: TODO - later\n", 1)
            .replace("| TODO | M12.1-T2a |", "| TODO - later | M12.1-T2a |", 1),
            "M12.1-T2a: 'TODO - later': TODO stands alone")
        red("a state word in the title is no state: a block with no Status line is held to the rules",
            FIXTURE.replace("## M12.1-T2a · Extend the widget · **BUILD** · (AFTER M12.1-T1)\n- Status: TODO\n",
                            "## M12.1-T2a · DONE when the seam opens · **BUILD** · (AFTER M12.1-T1)\n", 1),
            "M12.1-T2a: first line is not `- Status:`")
        red("a type that disagrees with the id", FIXTURE.replace("## M12.1-T2a · Extend the widget · **BUILD**",
                                                                "## M12.1-T2a · Extend the widget · **CHECK**", 1),
            "M12.1-T2a: typed **CHECK**, but its id makes it BUILD")
        moved = FIXTURE.replace("     Pass: the seam prints OK.\n", "").replace(
            "     Pass: GO, 0 failed.\n", "     Pass: GO, 0 failed.\n     Pass: the seam prints OK.\n", 1)
        red("a `Verify:` fence whose Pass: moved into the fence before it", moved,
            "M12.1-T2b: the `Verify:` fence at line %d has no Pass: line" % moved.split("\n").index("     python3 widget.py --seam"))
        red("a `Verify:` fence whose Pass:/Fail: sit inside the fence, not after it",
            FIXTURE.replace("     python3 widget.py --fixed\n     ```\n     Pass: GO.\n     Fail: NO-GO.\n",
                            "     python3 widget.py --fixed\n     Pass: GO.\n     Fail: NO-GO.\n     ```\n", 1),
            "M12.1-D1: the `Verify:` fence at line")
        red("a `Verify:` fence whose Pass:/Fail: sit above it",
            FIXTURE.replace("  [NOT RUN - for you]\n  1. ```\n     python3 widget.py --fixed\n     ```\n     Pass: GO.\n     Fail: NO-GO.\n",
                            "  Pass: GO.\n  Fail: NO-GO.\n  [NOT RUN - for you]\n  1. ```\n     python3 widget.py --fixed\n     ```\n", 1),
            "M12.1-D1: the `Verify:` fence at line")
        first = FIXTURE.split("\n").index("     python3 widget.py --check")
        red("the first of two fences without its own pair: the FAIL names the first fence's line",
            FIXTURE.replace("     Pass: GO, 0 failed.\n     Fail: NO-GO.\n", "", 1),
            "the `Verify:` fence at line %d has no Pass:/Fail:" % first)
        fat = FIXTURE.replace("## Rules\n", "## Rules\n" + "filler\n" * 40, 1)
        red("a header over budget with no TODO TC<n>", fat, "no TODO TC<n> block", "--budget-header", "30",
            "--tc-margin", "0")
        tc = "\n## M12.1-TC · Contract · **PLAN** · (AFTER M12.1-D1)\n- Status: TODO\n"
        red("... a TODO bare `TC` (the contract) does not discharge it",
            fat.replace("| TODO | M12.1-D1 |", "| TODO | M12.1-TC | c | x | PLAN |\n| TODO | M12.1-D1 |", 1) + tc,
            "no TODO TC<n> block", "--budget-header", "30", "--tc-margin", "0")
        reset(fat.replace("| TODO | M12.1-D1 |", "| TODO | M12.1-TC1 | c | x | MOVE |\n| TODO | M12.1-D1 |", 1)
              + tc.replace("M12.1-TC ·", "M12.1-TC1 ·").replace("**PLAN**", "**MOVE**"))
        rc, out = go("lint", "--budget-header", "30", "--tc-margin", "0")
        check("... a TODO TC1 (a compaction) discharges it: lint GO", rc == 0, out)
        # §11 (v12.2): no TC<n> is owed until a budget is passed by more than the margin, 100 lines by default
        wide = FIXTURE.replace("## Rules\n", "## Rules\n" + "filler\n" * 140, 1)
        reset(wide)
        rc, out = go("lint", "--budget-header", "0", "--tc-margin", "0")
        m = re.search(r"header (\d+) > 0 \+ 0 and no TODO TC<n> block", out)
        size = int(m.group(1)) if m else 0
        check("... the wide header's size, read from lint's own FAIL line", size > 140, out)
        reset(wide)
        rc, out = go("lint", "--budget-header", str(size - 100))
        check("... a header 100 lines over its budget, the default margin's edge: lint GO", rc == 0, out)
        red("a header 101 lines over its budget with no TODO TC<n>", wide,
            "header %d > %d + 100 and no TODO TC<n> block" % (size, size - 101), "--budget-header", str(size - 101))
        red("a Pipeline state over budget with no TODO TC<n>", FIXTURE, "Pipeline state 6 > 2 + 0", "--budget-pipeline", "2",
            "--tc-margin", "0")
        reset()
        rc, out = go("lint", "--budget-pipeline", "2", "--tc-margin", "4")
        check("... a Pipeline state within its budget and margin (6 <= 2 + 4): lint GO", rc == 0, out)
        red("a Pipeline state past its budget and margin with no TODO TC<n>", FIXTURE, "Pipeline state 6 > 2 + 3",
            "--budget-pipeline", "2", "--tc-margin", "3")
        shutil.rmtree(os.path.join(mdir, "images"))
        red("images/ missing beside the plan", FIXTURE, "missing %s/" % os.path.join(mdir, "images"))
        write_text(os.path.join(mdir, "images"), "not a directory\n", "\n")
        red("images a plain file", FIXTURE, "exists but is not a directory")
        os.unlink(os.path.join(mdir, "images"))
        os.makedirs(os.path.join(mdir, "images"))
        empty = os.path.join(mdir, "m12e_implementation_plan.md")
        write_text(empty, "# empty\n\n## Flow\n\n## Rules\n\n## Superseded\n\n## Repo facts\n\n## Pipeline state\n", "\n")
        rc, out = run(["--plan", empty, "lint"])
        plant("a plan with zero blocks", rc == 1 and "no task blocks" in out, out)
        reset(FIXTURE.replace("## M12.1-T2a · Extend the widget · **BUILD**", "## M12.1-T2a · Extend the widget · **JUDGE**", 1)
              .replace("- Deliver: widget.py grows a seam", "- Deliver: widget.py grows a seam\n\n````markdown\n"
                       "## M12.1-D<n> · a template · **BUILD**\n- Status: TODO\n```\n````", 1))
        rc, out = go("lint")
        check("never flagged: a pre-v12 word (**JUDGE** on a T); a heading inside a fence is no block, and a "
              "shorter fence line inside closes nothing",
              rc == 0 and "blocks=5" in out, out)

        # --- lint: an old plan read as history (the lead at M0-TA2a: "Strict on open blocks only") ----
        def hist(label, mutated, n, *want):
            reset(mutated)
            rc, out = go("lint")
            check("read as history, GO: " + label, rc == 0 and "history=%d " % n in out and all(w in out for w in want), out)

        tp_st, t1_st = "- Status: DONE (2026-01-02)\n- Handoff: DONE - nothing", "- Status: DONE (2026-01-02)\n- Deliver: widget.py\n"
        hist("a finished block in bold with notes, another in the plan's own language (`TERMINÉ`)",
             FIXTURE.replace(tp_st, "- Status: **DONE (2026-01-02)** - notes kept\n- Handoff: DONE - nothing", 1)
             .replace(t1_st, "- Status: TERMINÉ (2026-01-02)\n- Deliver: widget.py\n", 1),
             2, "HISTORY: 2 finished block(s) a v12 check would refuse", "M12.1-TP · M12.1-T1")
        hist("a bare `DONE`, and a Status on a finished block's second line",
             FIXTURE.replace(tp_st, "- Status: DONE\n- Handoff: DONE - nothing", 1)
             .replace(t1_st, "- Created by: M12.1-TP\n" + t1_st, 1), 2)
        hist("an old stub: its state in the heading, no Status line",
             FIXTURE.replace("**JUDGE** · (FIRST)\n- Status: DONE (2026-01-02)\n", "**JUDGE** · **DONE (2026-01-02)**\n", 1), 1)
        hist("a finished block with a fence and no Pass/Fail, a type off its id, its heading glued to the line above",
             FIXTURE.replace("# Phase A - build\n\n## M12.1-T1 · Open the widget · **BUILDER**",
                             "# Phase A - build\n## M12.1-T1 · Open the widget · **CHECK**", 1)
             .replace("- Verify: `python3 widget.py --selftest` - Pass: GO · Fail: otherwise.\n",
                      "- Verify:\n  ```\n  python3 widget.py --selftest\n  ```\n", 1), 1)
        hist("a finished block's Flow row off its date, another with no row",
             FIXTURE.replace("| DONE (2026-01-02) | M12.1-T1", "| DONE (2026-01-05) | M12.1-T1", 1)
             .replace("| DONE (2026-01-02) | M12.1-TP | Red-team the plan | FIRST | JUDGE |\n", "", 1), 2)
        table = FIXTURE[FIXTURE.index("| Status | Task |"):FIXTURE.index("\n\n## Rules")]
        prose = FIXTURE.replace(table, "The blocks run top to bottom; M12.1-D1 waits on M12.1-T2a.", 1)
        hist("a prose Flow: no block needs a row", prose, 0, "flow=prose", "the Flow is prose, no table")
        hist("a phase table", FIXTURE.replace(table, "| # | Phase | Tasks |\n|---|---|---|\n| 0 | gates | TP |\n"
                                                     "| A | build | T1 to D1 |", 1), 0, "flow=phase-table")
        hist("no `## Flow` at all", FIXTURE.replace("## Flow\n" + table + "\n\n", "", 1), 0, "flow=none")
        gated = FIXTURE.replace("| TODO | M12.1-T2a |", "|  | [commit gate] |  |  |  |\n| ⛔ | ⛔ |  |  |  |\n| TODO | M12.1-T2a |", 1)
        hist("gate rows in a per-task table name no block", gated, 0, "flow=per-task", "2 Flow row(s) name no block")
        red("a per-task table with gate rows stays per-task: an open block without a row is refused",
            gated.replace("| TODO | M12.1-D1 | Debug the widget seam | AFTER T2a | BUILD |\n", "", 1),
            "block M12.1-D1 has no Flow row")
        reset(prose)
        rc, out = go("append", nb, "--after", "M12.1-T2a", "--goal", "Second fix")
        t = text()
        check("append on a prose Flow: the block, its type and counter, no row",
              rc == 0 and "## M12.1-D2 · Debug the second seam · **BUILD**" in t and "D=2" in t
              and "| M12.1-D2" not in t and "flow_row=none" in out, out)
        rc, out = go("status", "M12.1-T2b", "DONE (2026-01-03)")
        check("status on a prose Flow: the Status line alone",
              rc == 0 and "- Status: DONE (2026-01-03)\n- Deliver: `widget.py` grows" in text() and "flow=prose" in out, out)

        # --- lint: the id table's new rows, and a pre-v12 id's kept kind -----------------------------
        want = {"TD": ("BUILD",), "TDa": ("BUILD",), "TD2": ("BUILD",), "TE": ("BUILD",), "TE2": ("BUILD",),
                "TEa": ("BUILD",), "TE-demo": ("BUILD",), "TDOC": None, "TDoc": None, "TEST": None,
                "TE-V12": ("PLAN", "MOVE"), "Vf": ("CHECK",), "V2": ("CHECK",)}
        got = {t: (id_type("M1-" + t) or (None,))[0] for t in want}
        check("§0's id table: TD, its splits and a TE suffix BUILD - never TDOC, TEST or TE-V<N>; a split V (`Vf`) CHECK",
              got == want, str(got))
        td = os.path.join(root, "td.md")
        write_text(td, NEWBLOCK.replace("M12.1-D2 · Debug the second seam", "M12.1-TD · Show the widget"), "\n")
        reset(FIXTURE.replace("TC=0", "TC=0 · TD=0", 1))
        rc, out = go("append", td, "--after", "M12.1-T2a")
        check("append a `TD` with no type: **BUILD** written, `TD=1`",
              rc == 0 and "## M12.1-TD · Show the widget · **BUILD** · (raised by M12.1-T1)" in text() and "TD=1" in text(), out)
        tv = (FIXTURE.replace("| TODO | M12.1-D1 |", "| TODO | M12.1-TV | Verify the widget | AFTER D1 | JUDGE |\n| TODO | M12.1-D1 |", 1)
              + "\n## M12.1-TV · Verify the widget · **CHECK · JUDGE** · (AFTER M12.1-D1)\n- Status: TODO\n")
        kept_line = "- Kind kept: M12.1-TV = CHECK (the plan's verification gate)\n"
        red("an open `TV` tagged CHECK with no `Kind kept:` line", tv, "M12.1-TV: typed **CHECK · JUDGE**, but its id makes it BUILD")
        reset(tv.replace("- One widget, one seam.\n", "- One widget, one seam.\n" + kept_line, 1))
        rc, out = go("lint")
        check("... GO once Repo facts keep its kind: `Kind kept: M12.1-TV = CHECK (...)`", rc == 0 and "history=0 " in out, out)
        red("... the same line under Rules keeps nothing (Repo facts only)",
            tv.replace("Header budget: 600 / 60.\n", "Header budget: 600 / 60.\n" + kept_line, 1),
            "M12.1-TV: typed **CHECK · JUDGE**, but its id makes it BUILD")
        red("... a tag off the kept kind",
            tv.replace("**CHECK · JUDGE** · (AFTER", "**PLAN · JUDGE** · (AFTER", 1).replace(
                "- One widget, one seam.\n", "- One widget, one seam.\n" + kept_line, 1),
            "M12.1-TV: typed **PLAN · JUDGE**, but its Repo facts keep it CHECK")

        # --- lint: the authoring warnings (open blocks) --------------------------------------------
        big = os.path.join(root, "big_reference.md")
        write_text(big, "x\n" * (READ_CAP + 100), "\n")
        bigname = os.path.basename(big)     # beside the plan: a native absolute path reads C:\... on Windows
        reset(FIXTURE.replace("- Deliver: widget.py grows a seam", "- Read: this file + %s\n- Deliver: widget.py grows a seam" % bigname, 1))
        rc, out = go("lint")
        check("WARN: a `Read:` naming a file over %d lines whole" % READ_CAP,
              rc == 0 and "M12.1-T2a: `Read:` names %s whole (%d lines" % (bigname, READ_CAP + 100) in out, out)
        reset(FIXTURE.replace("- Deliver: widget.py grows a seam", "- Read: this file + %s §2\n- Deliver: widget.py grows a seam" % bigname, 1))
        rc, out = go("lint")
        check("... and none when it names a section", rc == 0 and "`Read:` names" not in out, out)
        rep = "- Adversarial: the seam may leak under load - measure it twice with the probe"
        three = FIXTURE
        for anchor in ("- Deliver: widget.py grows a seam", "- Deliver: `widget.py` grows its second half", "- Deliver: the fix in"):
            three = three.replace(anchor, rep + "\n" + anchor, 1)
        reset(three)
        rc, out = go("lint")
        check("WARN: a line repeated in three open blocks", rc == 0 and "a line in 3 blocks (M12.1-T2a · M12.1-T2b · M12.1-D1)" in out, out)
        reset(three.replace(rep + "\n- Deliver: the fix in", "- Deliver: the fix in", 1))
        rc, out = go("lint")
        check("... and none for two", rc == 0 and "a line in" not in out, out)
        reset(FIXTURE.replace("- Deliver: the fix in `widget.py`", "- Deliver: the fix in `widget.py` · a runbook page", 1))
        rc, out = go("lint")
        check("WARN: a `Deliver:` item no Verify step names", rc == 0 and "no Verify step names" in out and "M12.1-D1 1" in out, out)
        reset(FIXTURE.replace("- Deliver: the fix in `widget.py`", "- Deliver: the fix in `widget.py` · a runbook page → none — prose", 1))
        rc, out = go("lint")
        check("... and none once it is mapped", rc == 0 and "no Verify step names" not in out, out)

        # --- the ladder and the rating (v12.1, §0, §3.2) ---------------------------------------------
        rung_cases = {"Sonnet 5.5, medium": ("Sonnet 5.5", "medium", ()),
                      "gpt-5.5-codex, high (gate, usual)": ("gpt-5.5-codex", "high", ("gate", "usual")),
                      "Model X, Turbo, none": ("Model X, Turbo", "none", ()), "Auto, balanced": ("Auto", "balanced", ()),
                      "Opus 5.5": None, "(AFTER M1-T1, M1-T2)": None, "**DONE (2026-01-02, x)**": None}
        got = {k: parse_rung(k) for k in rung_cases}
        check("a rung parses whole: a dot, a hyphen or a comma in the model (`Sonnet 5.5`, `gpt-5.5-codex`, "
              "`Model X, Turbo`), `none` and Auto's tier as levels; an order clause or a stub's state is no rung",
              got == rung_cases, str(got))
        head_cases = {
            "## M1-T1 · t · **BUILD** · Sonnet 5.5, medium · switch · (AFTER M1-T0)": ("Sonnet 5.5, medium", True),
            "## M1-T1 · t · **BUILD** · Sonnet 5.5, medium · (AFTER M1-T0)": ("Sonnet 5.5, medium", False),
            "## M1-T1 · t · **CHECK · JUDGE** · gpt-5.5-codex, high": ("gpt-5.5-codex, high", False),
            "## M1-T1 · the **new** seam · **BUILD** · usual rung · (AFTER M1-T0)": ("usual rung", False),
            "## M1-T1 · t, with a comma · **BUILD** · (AFTER M1-T0)": (None, False),
            "## M1-T1 · t · **BUILD** · ∥ set-A": (None, False),
            "## M1-T1 · t · **JUDGE** · **DONE (2026-01-02)**": (None, False),
            "## M1-T1 · t · **BUILD** · Opus 5.5, max (gate) · (AFTER M1-T0)": (None, False),
            "## M1-T1 · t · **BUILD** · switch · (AFTER M1-T0)": (None, True),
            "## M1-T1 · t · (AFTER M1-T0)": (None, False)}
        got = {k: heading_rating(k) for k in head_cases}
        check("a heading's rating is the segment after the type tag - never the title, the order clause, `∥ set-X`, "
              "a stub's `**DONE (…)**` or a ladder's `(gate)`; ` · switch` read after it", got == head_cases, str(got))
        check("§0's id table: `TM1` (the rating pass) PLAN; a bare v9 `TM` no id",
              id_type("M1-TM1") == (("PLAN",), "PLAN") and id_type("M1-TM") is None)
        ladder_line = ("Models: Claude Code — Opus 5.5, max (gate) · Sonnet 5.5, medium (usual) · gpt-5.5-codex, high · "
                       "Model X, Turbo, none — strongest first; read 2026-01-01 from the probe and the lead's answer")
        heads = (("**BUILD** · (AFTER M12.1-T1)", "**BUILD** · Sonnet 5.5, medium · (AFTER M12.1-T1)"),
                 ("**BUILDER** · (AFTER M12.1-T2a)", "**BUILDER** · gpt-5.5-codex, high · switch · (AFTER M12.1-T2a)"),
                 ("**BUILD** · (raised by M12.1-TP)", "**BUILD** · Model X, Turbo, none · (raised by M12.1-TP)"))
        laddered = FIXTURE.replace("Intro line.\n", "Intro line.\n" + ladder_line + "\n", 1)
        for old, new in heads:
            laddered = laddered.replace(old, new, 1)
        laddered = laddered.replace("| TODO | M12.1-D1 | Debug the widget seam | AFTER T2a | BUILD |",
                                    "| TODO | M12.1-D1 | Debug the widget seam | AFTER T2a | BUILD |\n"
                                    "| TODO | M12.1-Q1 | Ask the lead | AFTER D1 | LEAD answers |", 1) + (
            "\n## M12.1-Q1 · Question for the lead - the seam · **LEAD answers**\n- Status: TODO\n")
        check("the laddered fixture: the ladder line, every open block rated, a Q with no rating, the finished ones not",
              ladder_line in laddered and all(new in laddered for _, new in heads) and "| M12.1-Q1 |" in laddered
              and "**JUDGE** · (FIRST)" in laddered and "**BUILDER** · (AFTER M12.1-TP)" in laddered)
        reset(laddered)
        rc, out = go("lint")
        check("a laddered plan whose open blocks each carry a rung lints GO, no warning: its unrated finished blocks are "
              "history never rated after the fact (not counted), its Q carries none",
              rc == 0 and "ladder=4 rungs" in out and "warnings=0" in out and "history=0 " in out, out)
        crlf_l = os.path.join(mdir, "m12l_implementation_plan.md")
        write_text(crlf_l, laddered, "\r\n")
        rc, out = run(["--plan", crlf_l, "lint"])
        check("... the same plan with CRLF line endings: GO, the ladder read", rc == 0 and "ladder=4 rungs" in out, out)
        t2a_rated = "**BUILD** · Sonnet 5.5, medium · (AFTER M12.1-T1)"
        red("an open block with no rating, once the plan has a ladder: red, the ladder's words printed",
            laddered.replace(t2a_rated, "**BUILD** · (AFTER M12.1-T1)", 1),
            "M12.1-T2a: no rating after its type tag - write one rung of the ladder there: Opus 5.5, max (gate) · "
            "Sonnet 5.5, medium (usual)")
        write_text(crlf_l, laddered.replace(t2a_rated, "**BUILD** · (AFTER M12.1-T1)", 1), "\r\n")
        rc, out = run(["--plan", crlf_l, "lint"])
        plant("... the same plant with CRLF line endings: red", rc == 1 and "M12.1-T2a: no rating" in out, out)
        red("a rating off the ladder", laddered.replace(t2a_rated, "**BUILD** · Sonnet 5.5, high · (AFTER M12.1-T1)", 1),
            "M12.1-T2a: rating 'Sonnet 5.5, high' is off the ladder")
        red("a word after the level (`medium thinking`) is off the ladder",
            laddered.replace(t2a_rated, "**BUILD** · Sonnet 5.5, medium thinking · (AFTER M12.1-T1)", 1),
            "M12.1-T2a: rating 'Sonnet 5.5, medium thinking' is off the ladder")
        red("a role word left once the plan has a ladder", laddered.replace(t2a_rated, "**BUILD** · usual rung · (AFTER M12.1-T1)", 1),
            "M12.1-T2a: rating 'usual rung' is off the ladder (a role word")
        red("a ladder's mark copied into a heading (`(usual)`) is no rating",
            laddered.replace(t2a_rated, "**BUILD** · Sonnet 5.5, medium (usual) · (AFTER M12.1-T1)", 1), "M12.1-T2a: no rating")
        red("` · switch` with no rating before it",
            laddered.replace("gpt-5.5-codex, high · switch · ", "switch · ", 1),
            "M12.1-T2b: no rating after its type tag (` · switch` with no rating before it)")
        red("a DEFERRED block is open: unrated, red",
            laddered.replace("**BUILD** · Model X, Turbo, none · (raised by M12.1-TP)\n- Status: TODO\n",
                             "**BUILD** · (raised by M12.1-TP)\n- Status: DEFERRED (wake: the lead)\n", 1)
            .replace("| TODO | M12.1-D1 |", "| DEFERRED (wake: the lead) | M12.1-D1 |", 1), "M12.1-D1: no rating")
        red("a ladder with no `(usual)` rung", laddered.replace("medium (usual)", "medium", 1),
            "marks 0 rung(s) `(usual)`")
        red("a ladder rung that does not parse", laddered.replace("· gpt-5.5-codex, high ·", "· gpt-5.5-codex ·", 1),
            "the ladder's rung 'gpt-5.5-codex'")
        red("a `Models:` line that says `strongest first` out of its place",
            laddered.replace(" — strongest first; read", " · strongest first; read", 1), "is not `<agent> — <model>, <level>")
        bypassed = laddered.replace("gpt-5.5-codex, high · switch · ", "gpt-5.5-codex, high · ", 1)
        reset(bypassed)
        rc, out = go("lint")
        check("the lead's bypass - ` · switch` deleted, the rating kept - lints GO", rc == 0, out)
        reset(laddered.replace(t2a_rated, "**BUILD** · sonnet 5.5, Medium · (AFTER M12.1-T1)", 1))
        rc, out = go("lint")
        check("a rung's case is not a rating's fault (`sonnet 5.5, Medium`): GO", rc == 0, out)
        reset(bypassed)
        h2b = "## M12.1-T2b · Extend the widget, part two · **BUILDER** · gpt-5.5-codex, high · (AFTER M12.1-T2a)"
        rcs = [go(*c)[0] for c in (("status", "M12.1-T2b", "IN PROGRESS"), ("flag", "M12.1-T2b", "x", "--from", "M12.1-TP"),
                                   ("move", "M12.1-T2b", "--after", "M12.1-D1"), ("move", "M12.1-T2b", "--after", "M12.1-T2a"),
                                   ("close", "M12.1-T2b", "--status", "DONE (2026-01-03)", "--text", "DONE."))]
        check("a bypassed heading through status, flag, move there and back and close: the switch never written back",
              rcs == [0] * 5 and h2b + "\n" in text() and " · switch" not in text(), str(rcs))
        # append, on a plan with a ladder: the filer rates; the tool writes the type, never a rating or a switch
        reset(laddered)
        untouched("append an unrated block on a laddered plan (the ladder's words printed)", laddered,
                  ["append", nb, "--after", "M12.1-T2a"], want="write one rung of the ladder there: Opus 5.5, max (gate)")
        off = os.path.join(root, "off.md")
        write_text(off, NEWBLOCK.replace("second seam · (raised", "second seam · Sonnet 5.5, low · (raised"), "\n")
        untouched("append a block rated off the ladder", laddered, ["append", off, "--after", "M12.1-T2a"],
                  want="rating 'Sonnet 5.5, low' is off the ladder")
        on = os.path.join(root, "on.md")
        write_text(on, NEWBLOCK.replace("second seam · (raised", "second seam · Sonnet 5.5, medium · switch · (raised"), "\n")
        reset(laddered)
        rc, out = go("append", on, "--after", "M12.1-T2a")
        check("append a rated block with no type: the type written before the rating, its switch kept, `D=2`",
              rc == 0 and "## M12.1-D2 · Debug the second seam · **BUILD** · Sonnet 5.5, medium · switch · (raised by M12.1-T1)"
              in text() and "D=2" in text(), out)
        write_text(on, NEWBLOCK.replace("second seam · (raised", "second seam · Sonnet 5.5, medium · (raised"), "\n")
        reset(laddered)
        rc, out = go("append", on, "--after", "M12.1-T2a")
        check("... a block filed with no switch gets none: the tool never writes the segment",
              rc == 0 and "## M12.1-D2 · Debug the second seam · **BUILD** · Sonnet 5.5, medium · (raised by M12.1-T1)" in text(), out)
        tm = os.path.join(root, "tm.md")
        write_text(tm, NEWBLOCK.replace("M12.1-D2 · Debug the second seam · (raised by M12.1-T1)",
                                        "M12.1-TM1 · Rating pass - five flags · Opus 5.5, max · (AFTER M12.1-T2a)"), "\n")
        reset(laddered.replace("TC=0", "TC=0 · TM=0", 1))
        rc, out = go("append", tm, "--after", "M12.1-T2a")
        check("append the rating pass `TM1`: **PLAN** written from the id, `TM=1`",
              rc == 0 and "## M12.1-TM1 · Rating pass - five flags · **PLAN** · Opus 5.5, max · (AFTER M12.1-T2a)" in text()
              and "TM=1" in text(), out)
        # the refresh trigger (§0, §11): carried flags on open blocks since the register's `Model ratings:` date
        check("a `Carried flags:` field read as flags: each stamp opens one, hand-typed text before them is one, "
              "`none` is none",
              [carried_flags(t) for t in (" none", " [M1-TP, 2026-01-03] a · b · [M1-T1, 2026-01-04] c",
                                          " typed by hand · [M1-TP, 2026-01-03] a", " typed by hand · still one")]
              == [[], [("M1-TP", "2026-01-03"), ("M1-T1", "2026-01-04")], [None, ("M1-TP", "2026-01-03")], [None]])
        check("... a stamp read through bold and a backticked id: `**[`M4-TW1`, 2026-09-15]`, `[`M4-TW1`, 2026-09-15]`",
              [carried_flags(t) for t in (" **[`M4-TW1`, 2026-09-15] three items** · **[`M4-TE-V11`, 2026-09-16] x**",
                                          " [`M4-TW1`, 2026-09-15] a · [M4-T2, 2026-09-17] b")]
              == [[("M4-TW1", "2026-09-15"), ("M4-TE-V11", "2026-09-16")], [("M4-TW1", "2026-09-15"), ("M4-T2", "2026-09-17")]])
        rated_plan = laddered.replace("- Loop count: not measured\n",
                                      "- Loop count: not measured\n- Model ratings: 2026-01-02 by M12.1-TP\n", 1)

        def flagged(after, same=0, hand=0, base_text=rated_plan):
            """`after` flags stamped after the rating date, `same` on its day, `hand` unstamped - spread over
            the open blocks T2a, T2b, D1; one more, stamped later, on the DONE T1, never counted."""
            pool = (["· [M12.1-TP, 2026-01-0%d] later %d" % (3 + k % 5, k) for k in range(after)]
                    + ["· [M12.1-TP, 2026-01-02] same day %d" % k for k in range(same)])
            per = [[], [], []]
            for k, f in enumerate(pool):
                per[k % 3].append(f)
            for k in range(hand):
                per[k % 3].insert(0, "typed by hand %d" % k)
            t = base_text.replace("- Status: DONE (2026-01-02)\n- Deliver: widget.py\n",
                                  "- Status: DONE (2026-01-02)\n- Carried flags: [M12.1-TP, 2026-01-05] done, read\n"
                                  "- Deliver: widget.py\n", 1)
            for head, fl in zip(("Sonnet 5.5, medium · (AFTER M12.1-T1)", "gpt-5.5-codex, high · switch · (AFTER M12.1-T2a)",
                                 "Model X, Turbo, none · (raised by M12.1-TP)"), per):
                if fl:
                    t = t.replace(head + "\n- Status: TODO\n",
                                  head + "\n- Status: TODO\n- Carried flags: " + " ".join(fl).lstrip("· ") + "\n", 1)
            return t

        under = flagged(4, same=3)
        reset(under)
        rc, out = go("lint")
        check("refresh: 4 flags after the rating date, 3 on its own day, 1 on a DONE block - GO, `refresh=4/5`",
              rc == 0 and "refresh=4/5" in out and "rating refresh" not in out, out)
        red("refresh: the rating's date moves the window - the same plan rated a day earlier counts its 3 same-day flags",
            under.replace("Model ratings: 2026-01-02", "Model ratings: 2026-01-01", 1),
            "rating refresh: 7 carried flag(s) on open BUILD blocks since `Model ratings: 2026-01-01 by M12.1-TP`")
        at = flagged(5)
        red("refresh: at the threshold, no TM<n> pending - red, the append call that files one printed",
            at, 'append <M12.1-TM1 block.md> --after M12.1-T1 --goal "Rating pass - 5 carried flags"')
        unrow = lambda t: t.replace("- Model ratings: 2026-01-02 by M12.1-TP\n", "", 1)
        red("refresh: before the first `Model ratings:` line, hand-typed flags with no stamp count, and are named",
            unrow(flagged(3, hand=2)), "rating refresh: 5 carried flag(s)")
        rc, out = go("lint")
        check("... the unstamped ones named in a WARN, counted until the first dated line",
              "WARN: 2 carried flag(s) on open BUILD blocks carry no `[<from>, <date>]` stamp - counted" in out
              and "M12.1-T2a · M12.1-T2b" in out, out)
        reset(flagged(3, hand=2))
        rc, out = go("lint")
        check("refresh: after the `Model ratings:` line an unstamped flag is history - GO, `refresh=3/5`, still named",
              rc == 0 and "refresh=3/5" in out and "stamp - history, read by the pass since `Model ratings: "
              "2026-01-02 by M12.1-TP`" in out and "M12.1-T2a · M12.1-T2b" in out, out)
        red("refresh: no `Model ratings:` line - every stamped flag counts",
            unrow(under), "with no dated `Model ratings:` line")
        # M0-D25's three: a pass read the unstamped flags (X-21), a bold stamp is a stamp (X-22), and a flag on a
        # block the rubric pins - CHECK, PLAN, a `LEAD go` - moves no rating (X-23).  Each counted exactly.
        bold = lambda t: t.replace("[M12.1-TP, 2026-01-0", "**[`M12.1-TP`, 2026-01-0")
        pinned_rows = ("| TODO | M12.1-V1 | Check the phase | AFTER D1 | CHECK |\n"
                       "| TODO | M12.1-TZ | Close the milestone | AFTER V1 | PLAN |\n"
                       "| TODO | M12.1-T3 | Push the widget | AFTER D1 | BUILD + LEAD go |\n")
        pinned_blocks = "".join(
            "\n## M12.1-%s · %s · **%s** · Opus 5.5, max · (AFTER M12.1-D1)\n- Status: TODO\n- Carried flags: "
            "[M12.1-TP, 2026-01-05] %s a · [M12.1-TP, 2026-01-06] %s b\n- Handoff: <placeholder>\n" % (i, t, k, i, i)
            for i, t, k in (("V1", "Check the phase", "CHECK"), ("TZ", "Close the milestone", "PLAN"),
                            ("T3", "Push the widget", "BUILD + LEAD go")))
        q1_row = "| TODO | M12.1-Q1 | Ask the lead | AFTER D1 | LEAD answers |\n"
        pinned = rated_plan.replace(q1_row, q1_row + pinned_rows, 1) + pinned_blocks
        reset(flagged(4, base_text=pinned))
        rc, out = go("lint")
        check("refresh: 6 flags on an open CHECK, PLAN and `LEAD go` block are left out - GO, `refresh=4/5`",
              rc == 0 and "refresh=4/5" in out, out)
        reset(bold(flagged(4, same=3)))
        rc, out = go("lint")
        check("refresh: bold stamps on the rating's own day read as stamps - GO, `refresh=4/5`, no unstamped WARN",
              rc == 0 and "refresh=4/5" in out and "no `[<from>, <date>]` stamp" not in out, out)
        for nl in ("\n", "\r\n"):
            for label, t in (("2 unstamped flags after the `Model ratings:` line, 5 stamped later", flagged(5, hand=2)),
                             ("5 bold stamps with a backticked id", bold(flagged(5))),
                             ("6 flags on a CHECK, a PLAN and a `LEAD go` block, 5 on BUILD blocks",
                              flagged(5, base_text=pinned))):
                write_text(crlf_l, t, nl)
                rc, out = run(["--plan", crlf_l, "lint"])
                plant("refresh counts exactly - %s: red at 5 (%s)" % (label, "CRLF" if nl == "\r\n" else "LF"),
                      rc == 1 and "rating refresh: 5 carried flag(s) on open BUILD blocks since" in out, out)
        red("refresh: the Rules' `Rating refresh: 3 flags` tunes the threshold",
            flagged(3).replace("Header budget: 600 / 60.\n", "Header budget: 600 / 60. Rating refresh: 3 flags.\n", 1),
            "reach `Rating refresh: 3 flags`")
        for nl in ("\n", "\r\n"):
            write_text(crlf_l, at, nl)
            rc, out = run(["--plan", crlf_l, "lint"])
            plant("refresh at the threshold (%s): red" % ("CRLF" if nl == "\r\n" else "LF"),
                  rc == 1 and "rating refresh: 5 carried flag(s)" in out, out)
            write_text(crlf_l, under, nl)
            rc, out = run(["--plan", crlf_l, "lint"])
            check("refresh under the threshold, same-day flags left out (%s): GO" % ("CRLF" if nl == "\r\n" else "LF"),
                  rc == 0 and "refresh=4/5" in out, out)
        write_text(tm, NEWBLOCK.replace("M12.1-D2 · Debug the second seam · (raised by M12.1-T1)",
                                        "M12.1-TM1 · Rating pass - five flags · Opus 5.5, max · (AFTER M12.1-T1)"), "\n")
        reset(at.replace("TC=0", "TC=0 · TM=0", 1))
        rc1, out1 = go("append", tm, "--after", "M12.1-T1")
        rc, out = go("lint")
        check("refresh: filing the TM1 the red printed discharges it - GO, the count still shown",
              rc1 == 0 and rc == 0 and "refresh=5/5" in out, out1 + out)
        rc, out = go("status", "M12.1-TM1", "IN PROGRESS")
        rc, out = go("lint")
        check("... and a TM1 in progress still holds it (the pass is running)", rc == 0, out)
        rc, out = go("close", "M12.1-TM1", "--status", "DONE (2026-01-09)", "--text", "DONE.",
                     "--set", "Model ratings=2026-01-09 by M12.1-TM1")
        rc, out = go("lint")
        check("... the pass closed with the register's line moved to its day: the count drops to 0, GO",
              rc == 0 and "refresh=0/5" in out, out)
        reset(flagged(3))
        rc, out = go("flag", "M12.1-T2a", "x", "--from", "M12.1-TP", "--date", "2026-01-04")
        check("flag prints the running count against the threshold", rc == 0 and "refresh=4/5" in out, out)
        rc, out = go("flag", "M12.1-T2b", "y", "--from", "M12.1-TP", "--date", "2026-01-02")
        check("... a flag stamped on the rating's own day does not move it", rc == 0 and "refresh=4/5" in out, out)
        rc, out = go("flag", "M12.1-D1", "z", "--from", "M12.1-TP", "--date", "2026-01-04")
        check("... the flag that tips it: GO, a WARN carrying the append call",
              rc == 0 and "refresh=5/5" in out and "WARN: rating refresh: 5" in out and "M12.1-TM1" in out, out)
        rc, out = go("header")
        check("header reports the refresh count", rc == 0 and "refresh=5/5" in out and "rating refresh" in out, out)
        reset(flagged(6).replace(ladder_line, "Models: the lead's pick, per session — no default model", 1))
        rc, out = go("lint")
        rc2, out2 = go("header")
        check("no ladder: 6 flags - no count, no red, `header` silent on it",
              "rating refresh" not in out and "refresh=" not in out and "refresh=" not in out2, out + out2)
        # a plan with no ladder (every plan before its v12.1 adoption): nothing red, the unrated counted
        noladder = (FIXTURE.replace("Intro line.\n", "Intro line.\nModels: the lead's pick, per session — no default model\n", 1)
                    .replace("**BUILD** · (AFTER M12.1-T1)", "**BUILD** · ∥ set-A", 1)
                    .replace("**BUILDER** · (AFTER M12.1-T2a)", "**BUILDER** · switch · (AFTER M12.1-T2a)", 1)
                    .replace("**BUILD** · (raised by M12.1-TP)", "**BUILD** · usual rung · (raised by M12.1-TP)", 1)
                    .replace("**JUDGE** · (FIRST)\n- Status: DONE (2026-01-02)\n", "**JUDGE** · **DONE (2026-01-02)**\n", 1))
        for nl in ("\n", "\r\n"):
            write_text(path, noladder, nl)
            rc, out = go("lint")
            check("no ladder (%s): an `∥ set-A` heading, a switch with no rating, a role word and an old stub heading - GO, "
                  "one WARN counting the 2 unrated open blocks" % ("CRLF" if nl == "\r\n" else "LF"),
                  rc == 0 and "ladder=none" in out and "WARN: 2 open block(s) carry no rating" in out
                  and "M12.1-T2a · M12.1-T2b" in out and "FAIL" not in out, out)

        # --- a wrong call prints the right one -----------------------------------------------------
        reset()
        rc, out = go("status")
        check("a call missing its arguments: NO-GO, the right call printed",
              rc == 1 and 'plan.py status <ID> "<STATUS>"' in out, out)
        rc, out = go("dt")
        check("a call that does not exist (`dt`): NO-GO naming why and the calls there are",
              rc == 1 and "`dt` left with" in out and "close" in out, out)
        rc, out = go("close", "M12.1-T2b", "--text", "x")
        check("a call missing a required flag: the right call printed", rc == 1 and "close <ID> --status" in out, out)

        # --- files: the retried replace, the lock, the console ---------------------------------------
        reset()
        os.environ.update(fast, PB_PLAN_TEST_REPLACE_FAIL="2")
        rc, out = go("status", "M12.1-T2b", "IN PROGRESS (retry)")
        os.environ.pop("PB_PLAN_TEST_REPLACE_FAIL", None)
        os.environ.pop("PB_PLAN_TEST_BACKOFF", None)
        check("a transient replace failure is retried, not fatal",
              rc == 0 and "- Status: IN PROGRESS (retry)" in text(), out)
        untouched("a permanent replace failure refuses with a counted REFUSED line, never a traceback", None,
                  ["status", "M12.1-T2b", "IN PROGRESS (never)"], env=dict(fast, PB_PLAN_TEST_REPLACE_FAIL="99"),
                  want="REFUSED:")
        check("... and leaves no temp file behind",
              not [f for f in os.listdir(mdir) if f.startswith(".pbplan.")], str(os.listdir(mdir)))
        reset()
        ids = ["M12.1-TP", "M12.1-T2a", "M12.1-T2b", "M12.1-D1"]
        env = dict(os.environ, PB_PLAN_TEST_DELAY="0.15", PB_PLAN_LOCK_TIMEOUT="60")
        procs = [subprocess.Popen([sys.executable, os.path.abspath(__file__), "--plan", path, "status", bid,
                                   "IN PROGRESS (w%d)" % i], env=env, stdout=subprocess.PIPE,
                                  stderr=subprocess.STDOUT) for i, bid in enumerate(ids)]
        outs = [pr.communicate()[0].decode("utf-8", "replace") for pr in procs]
        landed = text()
        check("four concurrent `status` writes under a forced 0.15 s load->save window: all four land",
              all("IN PROGRESS (w%d)" % i in landed for i in range(len(ids)))
              and all(pr.returncode == 0 for pr in procs), " | ".join(o.strip().splitlines()[-1] for o in outs if o.strip()))
        reset(FIXTURE.replace("## Superseded / retired\n", "## Superseded / retired — two calls of ≤ 4 (§2.6)\n", 1))
        child = subprocess.run([sys.executable, os.path.abspath(__file__), "--plan", path, "show", "M12.1-TP"],
                               capture_output=True, timeout=60, env=dict(os.environ, PYTHONIOENCODING="cp1252"))
        check("a cp1252 console: `show` exits 0 and prints the title in UTF-8",
              child.returncode == 0 and "— two calls of ≤ 4".encode("utf-8") in child.stdout
              and b"=== GO ===" in child.stdout, (child.stdout + child.stderr).decode("utf-8", "replace"))

        reset()
        rc, out = go("header")
        check("header GO within the budgets", rc == 0 and "header=" in out, out)
        rc, out = go("lint")
        check("lint GO on the restored fixture, last", rc == 0, out)
    except Exception as exc:                 # a crash is a failed check, never a traceback in place of the verdict
        check("selftest stopped after %d checks: %s: %s" % (checks, type(exc).__name__, exc), False)
    finally:
        for k in [k for k in os.environ if k.startswith("PB_PLAN_TEST_")]:
            os.environ.pop(k)
        os.environ.update(kept_env)
        shutil.rmtree(root, ignore_errors=True)
    return verdict("selftest", checks, fails, as_json=a.json, counts=[("plants", plants)])


# ---------------------------------------------------------------------------- main
class CallError(Exception):
    """A wrong call: argparse's message, and the call it was meant to be."""

    def __init__(self, call, message):
        Exception.__init__(self, message)
        self.call, self.message = call, message


class Parser(argparse.ArgumentParser):
    pb_call = None

    def error(self, message):
        raise CallError(self.pb_call, message)


def wrong_call(exc, as_json):
    if exc.call:
        fix = "the call: %s %s" % (PROG, CALLS[exc.call])
    else:
        m = re.search(r"invalid choice: '([^']*)'", exc.message)
        asked = m.group(1) if m else None
        fix = "%sthe calls: %s" % ((REMOVED[asked] + "; ") if asked in REMOVED else "",
                                   " · ".join("%s %s" % (PROG, CALLS[c]) for c in CALLS))
    return verdict(exc.call or "plan", 1, ["%s - %s" % (exc.message, fix)], as_json=as_json, refused=True)


def main(argv=None):
    for stream in (sys.stdout, sys.stderr):   # a piped Windows console is cp1252; plan titles carry `≤`
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors=stream.errors)
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--plan", default=argparse.SUPPRESS,
                        help="the plan (default: the newest milestones/*/*implementation_plan.md with open work)")
    common.add_argument("--json", action="store_true", default=argparse.SUPPRESS, help="machine output, for agents")
    common.add_argument("--date", default=argparse.SUPPRESS, help="the date written (default: today)")
    p = Parser(prog="plan.py", parents=[common], description="PLAYBOOK §A.1: every edit an agent makes to the plan")
    sub = p.add_subparsers(dest="cmd", required=True, parser_class=Parser, metavar="<call>")

    def add(name, fn, text):
        s = sub.add_parser(name, parents=[common], help=text, description="%s - %s %s" % (text, PROG, CALLS[name]))
        s.pb_call = name
        s.set_defaults(fn=fn)
        return s

    add("show", cmd_show, "the block's start read: the header without its Flow table, then the block").add_argument("id")
    s = add("status", cmd_status, "the Status line and its Flow cell in one write; IN PROGRESS / DONE stamped")
    s.add_argument("id")
    s.add_argument("status", nargs="+")
    s = add("close", cmd_close, "status, handoff and register lines in one write: a task's whole close-out")
    s.add_argument("id")
    s.add_argument("--status", required=True)
    g = s.add_mutually_exclusive_group(required=True)
    g.add_argument("--file")
    g.add_argument("--text")
    s.add_argument("--set", action="append", default=[])
    s.add_argument("--allow-long", action="store_true")
    s = add("handoff", cmd_handoff, "replace the `- Handoff:` field (8 lines at most unless --allow-long)")
    s.add_argument("id")
    g = s.add_mutually_exclusive_group(required=True)
    g.add_argument("--file")
    g.add_argument("--text")
    s.add_argument("--allow-long", action="store_true")
    s = add("flag", cmd_flag, "append `· [<from>, <date>] <text>` to `- Carried flags:`")
    s.add_argument("id")
    s.add_argument("text")
    s.add_argument("--from", dest="frm", required=True)
    s = add("append", cmd_append, "file a block after an anchor: block, type, Flow row and counter in one call")
    s.add_argument("block")
    s.add_argument("--after", required=True)
    s.add_argument("--goal")
    s = add("move", cmd_move, "move a block and its Flow row, byte for byte")
    s.add_argument("id")
    s.add_argument("--after", required=True)
    s = add("stub", cmd_stub, "DONE / N/A / DEFERRED blocks to the archive, verified, a stub left in place")
    s.add_argument("ids", nargs="+")
    s.add_argument("--archive")
    s.add_argument("--by", default="unspecified")
    s = add("register", cmd_register, "rewrite `- <Key>:` rows of the Pipeline state")
    s.add_argument("--set", action="append", required=True)
    for name, fn, text in (("lint", cmd_lint, "the plan's structure against annex §A.1"),
                           ("header", cmd_header, "lines per header section and the total, against the budgets")):
        s = add(name, fn, text)
        s.add_argument("--budget-header", type=int, default=600)
        s.add_argument("--budget-pipeline", type=int, default=60)
        if name == "lint":                          # §11 (v12.2): a TC<n> owed only past budget + margin
            s.add_argument("--tc-margin", type=int, default=100)
    add("selftest", cmd_selftest, "every call round-tripped on a synthetic plan; each lint rule planted and shown red")

    try:
        a = p.parse_args(argv)
    except CallError as exc:
        return wrong_call(exc, "--json" in (sys.argv[1:] if argv is None else argv))
    a.plan, a.json, a.date = getattr(a, "plan", None), getattr(a, "json", False), getattr(a, "date", TODAY)
    if a.cmd == "selftest":
        return cmd_selftest(None, a)
    path = a.plan or default_plan()
    if not path or not os.path.exists(path):
        return verdict(a.cmd, 1, ["no plan file (--plan <path>, or milestones/*/*implementation_plan.md)"],
                       as_json=a.json)
    lock = PlanLock(path) if a.cmd in WRITE_CMDS else None
    if lock is not None and not lock.acquire():
        return verdict(a.cmd, 1, ["could not take the plan lock %s within %.0fs - another"
                                  " plan.py write is in progress; nothing was written"
                                  % (lock.path, lock.timeout)], as_json=a.json, refused=True)
    try:
        try:
            plan = Plan(path)
        except (OSError, UnicodeDecodeError) as exc:
            return verdict(a.cmd, 1, ["cannot read %s: %s" % (path, exc)], as_json=a.json, refused=True)
        return a.fn(plan, a)
    except PlanWriteError as exc:
        return verdict(a.cmd, 1, [str(exc)], as_json=a.json, refused=True)
    finally:
        if lock is not None:
            lock.release()


if __name__ == "__main__":
    sys.exit(main())
