#!/usr/bin/env python3
"""content_gate.py — the `neutral` gate: no pushed file names what the denylist holds (PLAYBOOK annex §A.8).

check · cases · selftest.
Python 3.9+, standard library only; UTF-8 in and out, on any console, a byte-order mark read as no text.
`check <file>…` reads every line of every file — prose, quotes, italics, code spans, fences, HTML
comments — against the entries of tools/pb/denylist.txt (the list beside this file), and prints one
counts line, one `<file>:<line>: <entry>` line per hit, then `=== GO ===` when there is none or
`=== NO-GO: <reason> ===` (exit 0 / 1); `--json` behind a flag. An entry matches case-insensitively as a
whole token (no letter or digit on either side); inside it `_`, `-`, a space or nothing stand for one
another, and a markdown `\\_` reads as `_`. A file named by `--fenced` is read in fence mode: only the lines
inside ```<tag> fences count (`--tag`, default v12) and every other hit is listed `allowed`.
`cases <pathspec>…` prints the runner's case ids and nothing else, one per line: the files git lists there,
tracked or untracked but never ignored, minus the list itself, plus each `--fenced` file present. The list
is local — never pushed, never scanned. Nothing is written and git is only read; the selftest's `git init`
and `git add` run inside its own temp folder. Usage: docs/agent/testing.md.
"""
import argparse
import contextlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

TOOL_DIR = os.path.dirname(os.path.abspath(__file__))
SEPARATORS = "-_ "
ENTRY_RE = re.compile(r"^[^\W_](?:.*[^\W_])?$")         # an entry starts and ends on a letter or a digit
ESCAPED_RE = re.compile(r"\\([-_])")                     # markdown's `\_` and `\-` are `_` and `-`


# ---------------------------------------------------------------- the list and the scan
def read(path):
    with open(path, encoding="utf-8-sig", errors="replace") as f:   # universal newlines: a CRLF file reads the same;
        return f.read()                                              # a list saved with a BOM keeps its first entry


def load(path):
    """([(entry, pattern)], [errors]) of the list: one entry per line, `#` starts a comment, blanks skipped."""
    try:
        text = read(path)
    except OSError as e:
        return [], [f"no denylist: {type(e).__name__}: {e}"]
    entries, errors, seen, name = [], [], set(), os.path.basename(path)
    for n, raw in enumerate(text.split("\n"), 1):
        e = raw.split("#", 1)[0].strip()
        if not e or e.casefold() in seen:
            continue
        if not ENTRY_RE.match(e):
            errors.append(f"{name}:{n}: {e!r} does not start and end on a letter or a digit")
            continue
        seen.add(e.casefold())
        body = "".join("[-_ ]?" if c in SEPARATORS else re.escape(c) for c in e)
        entries.append((e, re.compile(rf"(?<![^\W_]){body}(?![^\W_])", re.I)))
    if not entries and not errors:
        errors.append(f"{name} holds no entry — a gate that cannot fail")
    return entries, errors


def fenced(lines, tag):
    """The 1-based numbers of the lines inside ```<tag> / ~~~<tag> fences; a fence left open runs to the end."""
    opener = re.compile(rf"^\s*(`{{3,}}|~{{3,}})\s*{re.escape(tag)}(?:\s.*)?$", re.I)
    inside, close, out = False, None, set()
    for n, line in enumerate(lines, 1):
        if inside and close.match(line):
            inside = False
        elif inside:
            out.add(n)
        elif (m := opener.match(line)):
            inside, close = True, re.compile(rf"^\s*{re.escape(m.group(1)[0])}{{{len(m.group(1))},}}\s*$")
    return out


def scan(path, entries, tag=None):
    """([(line, entry)] that count, [(line, entry)] allowed) of one file; with `tag`, only its fences count."""
    lines = read(path).split("\n")
    keep = fenced(lines, tag) if tag else None
    hits, allowed = [], []
    for n, line in enumerate(lines, 1):
        text = ESCAPED_RE.sub(r"\1", line)
        for e, rx in entries:
            if rx.search(text):
                (hits if keep is None or n in keep else allowed).append((n, e))
    return hits, allowed


# ---------------------------------------------------------------- the commands
def verdict(a, counts, reason, lines=(), **data):
    """The §4 Rule 4 ending: one counts line, the finding lines only when there are any, the verdict."""
    if getattr(a, "json", False):
        print(json.dumps({"counts": dict(counts), **data, "verdict": "NO-GO" if reason else "GO", "reason": reason},
                         ensure_ascii=False, indent=1))
    else:
        print(" · ".join(f"{k}: {v}" for k, v in counts))
        for line in lines:
            print(line)
        print(f"=== NO-GO: {reason} ===" if reason else "=== GO ===")
    return 1 if reason else 0


def cmd_check(a):
    entries, errors = load(a.denylist)
    deny, fence = os.path.realpath(a.denylist), {os.path.realpath(p) for p in a.fenced}
    hits, allowed, files = [], [], 0
    for f in a.files:
        real = os.path.realpath(f)
        if real == deny:
            errors.append(f"{f}: the list is never scanned itself")
        elif not os.path.isfile(f):
            errors.append(f"{f}: no such file")
        elif entries:
            h, al = scan(f, entries, a.tag if real in fence else None)
            files += 1
            hits += [f"{f}:{n}: {e}" for n, e in h]
            allowed += [f"{f}:{n}: {e} — allowed (outside {a.tag} passages)" for n, e in al]
    reason = " · ".join([f"{len(hits)} hit(s)"] * bool(hits) + [f"{len(errors)} error(s)"] * bool(errors)) or \
        ("no file checked" if not files else None)
    return verdict(a, [("files", files), ("hits", len(hits)), ("allowed", len(allowed)), ("errors", len(errors))],
                   reason, [f"error: {x}" for x in errors] + hits + allowed, hits=hits, allowed=allowed, errors=errors)


def cmd_cases(a):
    """The case ids for the runner's `cases_cmd` — ids only, no counts line; exit 1 when git cannot list."""
    r = subprocess.run(["git", "--no-optional-locks", "ls-files", "-co", "--exclude-standard", "-z", "--", *a.paths],
                       capture_output=True)
    if r.returncode:
        print(f"cases: git ls-files exit {r.returncode}: {r.stderr.decode('utf-8', 'replace').strip()}", file=sys.stderr)
        return 1
    deny = os.path.realpath(a.denylist)
    ids = [p for p in r.stdout.decode("utf-8", "replace").split("\0")
           if p and os.path.isfile(p) and os.path.realpath(p) != deny]
    ids += [p for p in a.fenced if os.path.isfile(p) and p not in ids]
    for p in ids:
        print(p)
    return 0


# ---------------------------------------------------------------- selftest (red-armed, annex §A.0) — made-up names only
LIST = """# selftest list — snarfblat, named in a comment, is no entry
zorblax_widget        # a compound: `_`, `-`, a space or nothing
quuxcorp
frobnic-9-site
ünterplex
10.20.30.40           # a private address
"""
FIXTURE = """# Widget playbook — selftest fixture
The lead, 2030-01-02, verbatim: *"I want the best build out there."* · snarfblat stays clear.

## 1 Layout
- **Box:** `<box>`, bash 5; the checkout is `~/src/<project>`.
<!-- a comment the renderer hides -->
```bash
python3 tools/pb/verify.py --all --task M1-T1
```
| Unit | Remote |
|---|---|
| alpha | example · main |
Near misses, all clear: quuxcorporation · aquuxcorp · zorblax widgetry · frobnic 9 sites · unterplex ·
10.20.30.401 · 110.20.30.40.

## 2 Proposal items
```v11
The v11 passage.
```
```v12
The v12 passage: every tool is neutral.
```
The end.
"""
PLANTS = (  # name · the line the plant goes after · the planted line · the entry it must hit
    ("a name inside a quoted ask, in italics", "The lead,", '*"and ship it the Zorblax_Widget way."*', "zorblax_widget"),
    ("`_` written `-`", "## 1", "- the zorblax-widget checkout", "zorblax_widget"),
    ("`_` written as a space", "## 1", "the zorblax widget build", "zorblax_widget"),
    ("joined, CamelCase", "## 1", "see ZorblaxWidget/README", "zorblax_widget"),
    ("markdown-escaped `\\_`", "## 1", "zorblax\\_widget ran", "zorblax_widget"),
    ("in a code span", "## 1", "run `quuxcorp` first", "quuxcorp"),
    ("inside a bash fence", "python3 tools/pb/verify.py", "cd ~/src/quuxcorp", "quuxcorp"),
    ("inside an HTML comment", "<!-- a comment", "<!-- quuxcorp -->", "quuxcorp"),
    ("a path component", "- **Box:**", "  under /srv/x/frobnic-9-site/tools", "frobnic-9-site"),
    ("a private address, a port after it", "- **Box:**", "  ssh 10.20.30.40:22", "10.20.30.40"),
    ("inside an underscore compound", "## 1", "my_quuxcorp_notes", "quuxcorp"),
    ("alone on its line", "## 1", "quuxcorp", "quuxcorp"),
    ("in a table cell", "| alpha |", "| quuxcorp | example · main |", "quuxcorp"),
    ("non-ASCII, upper case", "## 1", "ÜNTERPLEX", "ünterplex"),
)


def planted(after, line, text=FIXTURE):
    """(text with `line` inserted after the first line holding `after`, the planted line's number)."""
    lines = text.split("\n")
    i = next(k for k, s in enumerate(lines) if after in s) + 1
    return "\n".join(lines[:i] + [line] + lines[i:]), i + 1


def cmd_selftest(a):
    checks, shapes, home, saved = [], [], os.getcwd(), os.environ.get("GIT_CEILING_DIRECTORIES")

    def check(name, ok):
        checks.append((name, bool(ok)))

    def run(*argv):
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            rc = main(list(argv))
        text = out.getvalue()
        if argv[0] == "check":              # every verdict line agrees with its exit code
            shapes.append(text.rstrip().endswith("=== GO ===") == (rc == 0) and rc in (0, 1))
        return rc, text

    def put(path, text, newline="\n"):
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "w", encoding="utf-8", newline=newline) as f:
            f.write(text)

    try:
        with tempfile.TemporaryDirectory(prefix="content_gate_selftest_") as tmp:
            deny, doc = os.path.join(tmp, "denylist.txt"), os.path.join(tmp, "doc.md")
            put(deny, LIST)
            put(doc, FIXTURE)
            gate = ("check", "--denylist", deny)
            rc, out = run(*gate, doc)
            check("clean fixture → GO: near misses, a comment's word, quotes, fences — zero hits", rc == 0 and
                  "files: 1 · hits: 0" in out)
            for name, after, line, entry in PLANTS:
                put(doc, planted(after, line)[0])
                rc, out = run(*gate, doc)
                check(f"PLANT {name} → NO-GO on its line", rc == 1 and f"doc.md:{planted(after, line)[1]}: {entry}\n"
                      in out)
            put(doc, planted("## 1", "zorblax_widget and quuxcorp")[0], newline="\r\n")
            rc, out = run(*gate, doc)
            check("PLANT in a CRLF copy → NO-GO, two entries on one line, the line number right",
                  rc == 1 and "hits: 2" in out and "doc.md:5: zorblax_widget" in out and "doc.md:5: quuxcorp" in out)

            fence, where = (*gate, "--fenced", doc), FIXTURE.split("\n")
            end, passage = where.index("The end.") + 1, where.index("The v12 passage: every tool is neutral.") + 1
            put(doc, FIXTURE)
            check("fence mode: the clean fixture → GO", run(*fence, doc)[0] == 0)
            put(doc, FIXTURE.replace("The end.", "The quuxcorp end."))
            rc, out = run(*fence, doc)
            check("fence mode: a hit outside the v12 fences → GO, listed allowed", rc == 0 and
                  f"doc.md:{end}: quuxcorp — allowed (outside v12 passages)" in out and "allowed: 1" in out)
            check("the same file while --fenced names another → NO-GO: fence mode is per file",
                  run(*gate, "--fenced", deny, doc)[0] == 1)
            put(doc, FIXTURE.replace("The v11 passage.", "The quuxcorp v11 passage."))
            check("fence mode: a hit inside a v11 fence → GO, allowed", run(*fence, doc)[0] == 0)
            put(doc, FIXTURE.replace("every tool is neutral", "every quuxcorp tool"))
            rc, out = run(*fence, doc)
            check("fence mode: PLANT inside the v12 fence → NO-GO", rc == 1 and f"doc.md:{passage}: quuxcorp\n" in out)
            put(doc, FIXTURE + "- After (v12):\n  ```v12\n  every quuxcorp tool\n  ```\nThe quuxcorp tail.\n")
            rc, out = run(*fence, doc)
            check("fence mode: PLANT inside an indented v12 fence (under a list item) → NO-GO, the line after it allowed",
                  rc == 1 and "hits: 1 · allowed: 1" in out)
            put(doc, FIXTURE + "```v12\nleft open: quuxcorp\n")
            check("fence mode: PLANT under a v12 fence left open → NO-GO (it runs to the end)",
                  run(*fence, doc)[0] == 1)

            put(doc, FIXTURE)
            put(deny, "# comments only\n\n")
            rc, out = run(*gate, doc)
            check("a list with no entry → NO-GO: a gate that cannot fail", rc == 1 and "a gate that cannot fail" in out)
            put(deny, LIST + "-edge\n")
            check("a malformed entry → NO-GO", run(*gate, doc)[0] == 1)
            put(deny, LIST)
            check("no list at the path → NO-GO",
                  run("check", "--denylist", os.path.join(tmp, "absent.txt"), doc)[0] == 1)
            rc, out = run(*gate, deny)
            check("the list named as a file → NO-GO: never scanned itself", rc == 1 and "never scanned itself" in out)
            rc, out = run(*gate, os.path.join(tmp, "absent.md"))
            check("a file that does not exist → NO-GO", rc == 1 and "no such file" in out)
            put(deny, "﻿" + LIST)
            clean_rc = run(*gate, doc)[0]
            put(deny, "﻿quuxcorp\n")
            put(doc, planted("## 1", "quuxcorp")[0])
            rc, out = run(*gate, doc)
            check("a list saved with a byte-order mark → the clean fixture GO, its first entry still a hit",
                  clean_rc == 0 and rc == 1 and f"doc.md:{planted('## 1', 'quuxcorp')[1]}: quuxcorp\n" in out)
            put(deny, LIST)
            check("every verdict line agrees with its exit code", shapes and all(shapes))

            kit, decoy = os.path.join(tmp, "kit", "tools", "pb"), os.path.join(tmp, "decoy")
            os.makedirs(kit)
            shutil.copyfile(os.path.abspath(__file__), os.path.join(kit, "content_gate.py"))
            put(os.path.join(kit, "denylist.txt"), LIST)
            for rel in ("denylist.txt", "tools/pb/denylist.txt"):          # a list that would let the plant through
                put(os.path.join(decoy, rel), "decoyword\n")

            def child(*argv, enc="utf-8"):
                p = subprocess.run([sys.executable, os.path.join(kit, "content_gate.py"), *argv], cwd=decoy,
                                   capture_output=True, timeout=60, env=dict(os.environ, PYTHONIOENCODING=enc))
                return p.returncode, p.stdout.decode("utf-8", "replace")
            put(doc, planted(PLANTS[0][1], PLANTS[0][2])[0])
            rc, out = child("check", doc)
            check("no --denylist → the list beside the tool, not one in the working folder: the italic plant NO-GO",
                  rc == 1 and f"doc.md:{planted(PLANTS[0][1], PLANTS[0][2])[1]}: zorblax_widget\n" in out)
            put(doc, FIXTURE.replace("The end.", "The quuxcorp end."))
            runs = [child("check", "--fenced", doc, doc, enc=enc) for enc in ("cp437", "ascii")]
            check("a cp437 or ASCII console → the verdict in UTF-8, not a crash: fence mode GO, the `—` printed",
                  all(rc == 0 and out.rstrip().endswith("=== GO ===") and "— allowed (outside v12" in out
                      for rc, out in runs))

            repo, plain = os.path.join(tmp, "repo"), os.path.join(tmp, "plain")
            os.makedirs(plain)
            os.environ["GIT_CEILING_DIRECTORIES"] = tmp                    # `plain` is no git checkout
            subprocess.run(["git", "-c", "init.defaultBranch=main", "init", "-q", repo], check=True)
            for rel in ("PLAYBOOK.md", "tools/pb/tool.py", "tools/new.md", "tools/local.txt", "tools/gone.py",
                        "notes/other.md", "tools/pb/denylist.txt"):
                put(os.path.join(repo, rel), "x\n")
            put(os.path.join(repo, ".gitignore"), "tools/local.txt\nproposals/\n")
            subprocess.run(["git", "-C", repo, "add", "PLAYBOOK.md", "tools/gone.py"], check=True)
            os.remove(os.path.join(repo, "tools", "gone.py"))                # tracked, then deleted: nothing to scan
            os.chdir(repo)
            args = ("cases", "--denylist", "tools/pb/denylist.txt", "--fenced", "proposals/P.md", "PLAYBOOK.md", "tools")
            rc, out = run(*args)
            check("cases: tracked + untracked under the pathspecs; ignored, deleted, the list and the rest left out",
                  rc == 0 and sorted(out.split()) == ["PLAYBOOK.md", "tools/new.md", "tools/pb/tool.py"])
            rc, out = run("cases", "--denylist", "tools/pb/denylist.txt")
            check("cases with no pathspec: every file git lists, the list still left out",
                  rc == 0 and sorted(out.split()) == [".gitignore", "PLAYBOOK.md", "notes/other.md", "tools/new.md",
                                                      "tools/pb/tool.py"])
            put(os.path.join(repo, "proposals", "P.md"), "x\n")
            rc, out = run(*args)
            check("cases: a --fenced file present is a case, ignored or not",
                  rc == 0 and sorted(out.split()) == ["PLAYBOOK.md", "proposals/P.md", "tools/new.md", "tools/pb/tool.py"])
            os.chdir(plain)
            check("cases outside a git checkout → exit 1", run(*args)[0] == 1)
    except Exception as e:                  # a crash is a failed check, never a traceback in place of the verdict
        check(f"selftest stopped after {len(checks)} checks: {type(e).__name__}: {e}", False)
    finally:
        os.chdir(home)
        os.environ.pop("GIT_CEILING_DIRECTORIES", None) if saved is None else \
            os.environ.__setitem__("GIT_CEILING_DIRECTORIES", saved)
    failed = [n for n, ok in checks if not ok]
    return verdict(a, [("checks", len(checks)), ("passed", len(checks) - len(failed)), ("failed", len(failed))],
                   "zero checks ran" if not checks else f"{len(failed)} check(s) failed" if failed else None,
                   [f"FAIL {n}" for n in failed], failed=failed)


# ---------------------------------------------------------------- CLI
def main(argv=None):
    ap = argparse.ArgumentParser(prog="content_gate.py", description="The neutral gate: no pushed file names what "
                                 "tools/pb/denylist.txt holds. Guide: docs/agent/testing.md")
    sub = ap.add_subparsers(dest="cmd", required=True, metavar="<command>")
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--denylist", default=os.path.join(TOOL_DIR, "denylist.txt"), help="default: tools/pb/denylist.txt")
    common.add_argument("--fenced", action="append", default=[], metavar="FILE", help="read FILE in fence mode (repeatable)")
    sp = sub.add_parser("check", parents=[common], help="scan files: one line per hit, GO on zero hits")
    sp.add_argument("files", nargs="+", metavar="file")
    sp.add_argument("--tag", default="v12", help="fence mode: the fences that count, ```<tag> (default: v12)")
    sp.add_argument("--json", action="store_true", help="machine output, for agents")
    sp = sub.add_parser("cases", parents=[common], help="the case ids: git's files under the pathspecs, minus the list, "
                        "plus the --fenced files present")
    sp.add_argument("paths", nargs="*", metavar="pathspec")
    sp = sub.add_parser("selftest", help="red-armed selftest on a temp fixture")
    sp.add_argument("--json", action="store_true", help="machine output, for agents")
    a = ap.parse_args(argv)
    return {"check": cmd_check, "cases": cmd_cases, "selftest": cmd_selftest}[a.cmd](a)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):      # a cp437 or ASCII console must not crash on a `·` or a `—`
        with contextlib.suppress(AttributeError, ValueError, OSError, io.UnsupportedOperation):
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    try:
        sys.exit(main())
    except Exception as e:                  # a crash is a NO-GO with its cause, never a bare traceback
        print(f"=== NO-GO: content_gate.py stopped: {type(e).__name__}: {e} ===")
        sys.exit(1)
