#!/usr/bin/env python3
"""review_page.py — the lead's eye, one page (PLAYBOOK annex §A.4).

Python 3.9+, standard library only, Windows and POSIX. Relative paths resolve from the current directory.
  review_page.py build --images <dir> [--captions captions.json] --report reports/review<k>.md [--out <dir>/review.html]
  review_page.py serve <same> [--host 127.0.0.1] [--port 8765]
  review_page.py selftest [--inject <bug>]
The page holds only the captures the TV flagged for the lead (captions.json's `flag`), each with the TV's `note`: one
capture at a time, previous / next buttons and ← → keys, a counter (`7 / 12`), the caption (route · state · viewport ·
overflow flag · the note) and a remarks box. Under `serve` the box autosaves ~800 ms after typing (`POST /remark
{id, text}`, a visible saved marker); the server keeps `review<k>.remarks.json` beside the report and regenerates the
report's table from it on every save — capture id · route/state · remark, verbatim · time — between two marker
comments, so text written outside them (the triage, `Review verdict:`) survives. `build` alone writes a static page
whose "Download remarks" button assembles the same markdown in the browser. Images are referenced by relative path,
never embedded; `--out` defaults to `<images>/review.html`. `GET /health` answers {"status": "ready"} (a launch.json key).
captions.json (§A.5's file, with the TV's two keys): a JSON list of {id, file, route, path, state, viewport, overflow,
caption, flag, note}; `file` defaults to `<id>.png`; `flag` is true or false (absent: false) and a flagged capture
carries a `note` — why it needs the lead's eyes. Every image in --images has an entry, flagged or not: the TV looked at
every capture. With no captions.json nothing is flagged, so there is no page.
Every printed path goes through shown(): repo-relative under the project root (two folders above this file), `~` under
the home folder, and the home folder's name never printed (it names a person).
Every command ends with one counts line, failures only when non-zero, and `=== GO ===` / `=== NO-GO: <reason> ===`
(exit 0 / 1); `--json` behind a flag. UTF-8 (a byte-order mark read), atomic writes retried while another process holds
the file, line endings preserved, never deletes a file, never runs git.
"""
import argparse
import contextlib
import datetime
import functools
import html
import http.server
import io
import json
import os
import re
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import zlib
from pathlib import Path

IMAGE_MIME = {".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".webp": "image/webp", ".gif": "image/gif"}
BEGIN, END = "<!-- review_page:remarks:begin -->", "<!-- review_page:remarks:end -->"
BAD_ID = re.compile(r"[`|<>\x00-\x1f\x7f]")
MAX_BODY = 1 << 20
REPLACE_TRIES, REPLACE_WAIT_S = 6, 0.05
INJECTS = ("accept_unknown", "memory_only", "drop_outside", "show_unflagged", "raw_paths")
INJECT = set()  # in-memory bug switches: set only by `selftest` (its red arms), never by build / serve


# ---------------------------------------------------------------- printed paths
HOME_TOKEN = "<home>"
# A home folder named this commonly, or this shortly, is never rewritten as a bare word — `root` would eat
# "the repo root". Its paths are still rewritten: the prefix passes need no name.
GENERIC = {"root", "home", "user", "users", "admin", "administrator", "public", "default", "guest"}
START = r"(?<![\w.~$\\/])"  # where a root spelling starts: never inside a longer path (`/mnt/bak/home/jdoe`)
END_OF = r"(?![\w.~$-])"    # where it ends: never inside a longer name (`proj_old`, `jdoe2`, `jdoe.bak`)
SEP = r"(?:\\\\|[\\/])"      # one separator as printed: `\`, `/`, or a repr's doubled `\\`
REWRITE = None              # the rewriter shown() uses when set (the selftest's synthetic roots); else this_box()


def native_forms(p):
    """`p`'s 8.3 short and long spellings, read from the Windows API (`%TEMP%` is often spelled
    `C:\\Users\\ABCDEF~1\\AppData\\Local\\Temp`); nothing elsewhere, and nothing on any failure."""
    if os.name != "nt":
        return set()
    out = set()
    try:
        import ctypes
        k32 = ctypes.windll.kernel32
        for fn in (k32.GetShortPathNameW, k32.GetLongPathNameW):
            n = fn(p, None, 0)
            buf = ctypes.create_unicode_buffer(max(n, 1))
            if n and fn(p, buf, n):
                out.add(buf.value)
    except Exception:  # a spelling that cannot be read is one not rewritten, never a crash
        pass
    return out


def spellings(forms, doubled=True):
    """Every textual spelling of one root, longest first: each form without a trailing separator, and a Windows form
    (one holding a `\\`) also with `/` separators and, unless `doubled` is off, with a repr's doubled `\\\\`."""
    out = set()
    for f in forms:
        f = str(f).rstrip("\\/")
        if f:
            out.add(f)
            if "\\" in f:
                out.add(f.replace("\\", "/"))
                if doubled:
                    out.add(f.replace("\\", "\\\\"))
    return sorted(out, key=len, reverse=True)


class Rewriter:
    """Three passes, in order: every spelling of the project root → repo-relative (the root alone reads `.`), every
    spelling of the home folder → `~`, then the home folder's name, as a whole word, → `<home>` (a path spelled some
    other way: a scratchpad folder named after a path, an owner column). A root matches only whole — where it starts
    and where it ends; case-insensitive. `start`, `end` and `doubled` exist for the selftest's plants only."""

    def __init__(self, repo, home, names, start=START, end=END_OF, doubled=True):
        alt = lambda ss: "|".join(re.escape(s) for s in ss)  # noqa: E731
        r, h = spellings(repo, doubled), spellings(home, doubled)
        n = sorted({x for x in names if x and len(x) >= 3 and x.lower() not in GENERIC}, key=len, reverse=True)
        self.repo = re.compile(f"{start}(?:{alt(r)})(?:{SEP}|{end})", re.I) if r else None
        self.home = re.compile(f"{start}(?:{alt(h)}){end}", re.I) if h else None
        self.name = re.compile(rf"(?<!\w)(?:{alt(n)})(?!\w)", re.I) if n else None

    def __call__(self, text):
        if self.repo:  # the root AND the one separator after it go, so a relative path is left
            text = self.repo.sub(lambda m: "" if m.group(0)[-1:] in "\\/" else ".", text)
        if self.home:
            text = self.home.sub("~", text)
        if self.name:
            text = self.name.sub(HOME_TOKEN, text)
        return text


def project_root(p=None):
    """The folder two above this file (`<root>/tools/pb/review_page.py`), as the sibling tools read it."""
    p = Path(p or __file__)
    return p.parents[2] if len(p.parents) > 2 else p.parent


@functools.lru_cache(maxsize=1)
def this_box():
    """The rewriter for the box this runs on: the project root, the running user's home folder, every spelling."""
    repo = {str(project_root(Path(__file__).resolve())), str(project_root(Path(__file__).absolute()))}
    home = set()
    with contextlib.suppress(RuntimeError, KeyError, OSError):  # no resolvable home: the root pass alone
        home = {str(Path.home()), os.path.realpath(Path.home())}
    for forms in (repo, home):
        forms |= {s for f in list(forms) for s in native_forms(f)}
    return Rewriter(repo, home, {Path(h).name for h in home})


def shown(x):
    """`x` — a path, or a text carrying paths — as this tool prints it (the module's docstring). Never raises: it sits
    on the lines a verdict is made of."""
    try:
        text = x if isinstance(x, str) else os.fspath(x) if isinstance(x, os.PathLike) else str(x)
    except Exception:  # an object that cannot print itself still yields a line
        text = object.__repr__(x)
    return text if "raw_paths" in INJECT else (REWRITE or this_box())(text)


# ---------------------------------------------------------------- files
def read_text(path):
    with open(path, encoding="utf-8", newline="") as f:
        return f.read()


def read_json(path):
    with open(path, encoding="utf-8-sig") as f:  # a byte-order mark from a Windows editor is read, not a parse error
        return json.load(f)


def insist(op, *args):
    """`op(*args)`, retried while it raises PermissionError — on Windows another process (an editor, a scanner, the
    browser reading the page) can hold the file without share-delete. REPLACE_TRIES tries, the wait doubling from
    REPLACE_WAIT_S: bounded, so a file that stays held gets a verdict, never a hang. The last refusal is raised."""
    for n in range(REPLACE_TRIES):
        try:
            return op(*args)
        except PermissionError:
            if n == REPLACE_TRIES - 1:
                raise
            time.sleep(REPLACE_WAIT_S * 2 ** n)


def write_atomic(path, text, eol="\n"):
    """Whole or not at all: a temp beside the target, then one replace through insist(). On any failure the temp is
    removed (the only file this ever removes: its own) and the target is as it was."""
    d = os.path.dirname(os.path.abspath(path))
    os.makedirs(d, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=d, prefix=".review_tmp_")
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="") as f:
            f.write(text if eol == "\n" else re.sub(r"\r?\n", eol, text))
        insist(os.replace, tmp, path)
    except BaseException:
        with contextlib.suppress(OSError):
            insist(os.unlink, tmp)
        raise


def eol_of(path):
    try:
        return "\r\n" if "\r\n" in read_text(path) else "\n"
    except (OSError, ValueError):
        return "\n"


def verdict_out(counts, reason, as_json, lines=(), failures=(), warnings=(), skips=()):
    ok = reason is None
    if as_json:
        print(shown(json.dumps({"counts": dict(counts), "lines": list(lines), "failures": list(failures),
                                "warnings": list(warnings), "skips": list(skips), "verdict": "GO" if ok else "NO-GO",
                                "reason": reason}, indent=1, ensure_ascii=False)))
    else:
        for ln in lines:
            print(shown(ln))
        print(shown(" · ".join(f"{k} {v}" for k, v in counts)))
        for tag, items in (("FAIL", failures), ("WARN", warnings), ("SKIP", skips)):
            for x in items:
                print(shown(f"{tag} {x}"))
        print(shown("=== GO ===" if ok else f"=== NO-GO: {reason} ==="))
    sys.stdout.flush()
    return 0 if ok else 1


# ---------------------------------------------------------------- captures
def load_captures(images, captions=None):
    """→ (flagged, total, failures). Every entry names an image inside --images and every image there has an entry;
    the page gets the flagged ones, each with its note."""
    if not os.path.isdir(images):
        return [], 0, [f"--images {images}: not a directory"]
    on_disk = sorted(f for f in os.listdir(images) if os.path.splitext(f)[1].lower() in IMAGE_MIME)
    cpath = captions or os.path.join(images, "captions.json")
    if not captions and not os.path.isfile(cpath):
        return [], 0, [f"{cpath}: not found — the page holds only what the TV flags there (`flag`, `note`)"]
    try:
        raw = read_json(cpath)
    except (OSError, ValueError) as e:
        return [], 0, [f"captions {cpath}: {e}"]
    if not isinstance(raw, list):
        return [], 0, [f"captions {cpath}: not a JSON list of captures (§A.5's file)"]
    caps, fails, ids, captioned = [], [], set(), set()
    for n, c in enumerate(raw):
        cid = c.get("id") if isinstance(c, dict) else None
        if not isinstance(cid, str) or not cid.strip() or BAD_ID.search(cid):
            fails.append(f"captions entry {n}: id {cid!r} missing, or holds ` | < > or a control character")
            continue
        f = c.get("file") or cid + ".png"
        parts = [p for p in f.replace("\\", "/").split("/") if p not in ("", ".")] if isinstance(f, str) else [".."]
        flag, note = c.get("flag", False), c.get("note")
        if os.path.isabs(str(f)) or ".." in parts or os.path.splitext(parts[-1])[1].lower() not in IMAGE_MIME:
            fails.append(f"{cid}: file {f!r} is not an image path inside --images")
            continue
        captioned.add("/".join(parts))
        if cid in ids:
            fails.append(f"{cid}: duplicate capture id")
        elif not os.path.isfile(os.path.join(images, *parts)):
            fails.append(f"{cid}: image {f} missing from {images}")
        elif not isinstance(flag, bool):
            fails.append(f"{cid}: flag {flag!r} is not true or false")
        elif note is not None and not isinstance(note, str):
            fails.append(f"{cid}: note {note!r} is not a text")
        elif flag and not (note or "").strip():
            fails.append(f"{cid}: flagged with no note — the lead would not know why it is on the page")
        else:
            ids.add(cid)
            ov = c.get("overflow")
            caps.append({"id": cid, "file": "/".join(parts), "overflow": ov if isinstance(ov, bool) else None,
                         "flag": flag, "note": note or "",
                         **{k: "" if c.get(k) is None else str(c[k]) for k in ("route", "path", "state", "viewport", "caption")}})
    fails += [f"{f}: image with no captions.json entry — the TV never flagged or passed it" for f in on_disk
              if f not in captioned]
    if not raw:
        fails.append(f"captions {cpath}: no captures")
    flagged = caps if "show_unflagged" in INJECT else [c for c in caps if c["flag"]]
    if caps and not flagged:
        fails.append(f"captions {cpath}: nothing flagged for the lead (`flag`) — no page to show")
    return flagged, len(caps), fails


# ---------------------------------------------------------------- remarks → report
def remarks_path(report):
    return os.path.splitext(report)[0] + ".remarks.json"


def load_remarks(report):
    """{id: {text, time}}. A missing file is empty; an unreadable or malformed one raises and is never overwritten."""
    p = remarks_path(report)
    if not os.path.exists(p):
        return {}
    data = read_json(p)
    rem = data.get("remarks") if isinstance(data, dict) else None
    if not isinstance(rem, dict) or not all(isinstance(v, dict) and isinstance(v.get("text"), str)
                                            and isinstance(v.get("time"), str) for v in rem.values()):
        raise ValueError(f'{p}: expected {{"remarks": {{id: {{text, time}}}}}}')
    return rem


def esc(s):
    return re.sub(r"\r\n|\r|\n", "<br>", s.replace("|", "\\|"))


def region(captures, remarks):
    """The tool-owned part of review<k>.md, as lines. The page's region() mirrors it line for line (selftest: node)."""
    ids = [c["id"] for c in captures]
    orphans = sorted(k for k in remarks if k not in set(ids))
    rows, verbatim = [], []
    for cid, cap in [(c["id"], c) for c in captures] + [(k, None) for k in orphans]:
        where = "(not in this capture set)" if cap is None else esc(" · ".join(x for x in (cap["route"], cap["state"]) if x)) or "—"
        r = remarks.get(cid)
        if r is None:
            rows.append(f"| `{cid}` | {where} | _no remark_ | — |")
            continue
        cell = esc(r["text"])
        if cell != r["text"]:
            cell += " ↓"
            verbatim.append((cid, r["text"]))
        rows.append(f"| `{cid}` | {where} | {cell} | {r['time']} |")
    out = [BEGIN, f"remarks {sum(i in remarks for i in ids)} · flagged captures {len(ids)} · not in this capture set "
           f"{len(orphans)}", "", "| capture | route / state | remark (verbatim) | time |", "|---|---|---|---|"] + rows
    if verbatim:
        out += ["", "Verbatim — the remarks a table cell cannot hold as typed:"]
        for cid, text in verbatim:
            fence = "`" * max(3, 1 + max((len(m) for m in re.findall(r"`+", text)), default=0))
            out += ["", f"`{cid}`", fence + "text"] + re.split(r"\r\n|\r|\n", text) + [fence]
    return out + [END]


def report_text(title, captures, remarks):
    """A new review<k>.md: header + region. The static page's Download remarks builds exactly this text."""
    return "\n".join([f"# Review remarks — {title}", "",
                      "Written by `tools/pb/review_page.py` (PLAYBOOK annex §A.4). The table between the two markers is "
                      "tool-owned (rewritten on every save under `serve`); write the triage and the `Review verdict:` line "
                      "outside them.", ""] + region(captures, remarks)) + "\n"


def write_report(report, captures, remarks):
    """Rewrite only the marked region; a report without markers gets the region appended; a new one gets the header."""
    eol = eol_of(report)
    if not os.path.exists(report) or "drop_outside" in INJECT:
        return write_atomic(report, report_text(os.path.splitext(os.path.basename(report))[0], captures, remarks), eol)
    old = read_text(report).replace("\r\n", "\n")
    b, e = old.find(BEGIN), old.rfind(END)
    body = "\n".join(region(captures, remarks))
    write_atomic(report, old[:b] + body + old[e + len(END):] if 0 <= b < e else old.rstrip("\n") + "\n\n" + body + "\n", eol)


# ---------------------------------------------------------------- the page
PAGE = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>__TITLE__ · review page</title>
<style>
:root{--bg:#fff;--fg:#1d1d1f;--mut:#5f6368;--bd:#d0d4d9;--ok:#137333;--bad:#b3261e}
@media (prefers-color-scheme:dark){:root{--bg:#161616;--fg:#e8e8e8;--mut:#a0a4a8;--bd:#3c4043;--ok:#81c995;--bad:#f28b82}}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif}
header{position:sticky;top:0;z-index:1;display:flex;gap:8px;align-items:center;padding:8px 12px;background:var(--bg);border-bottom:1px solid var(--bd)}
h1{flex:1;min-width:0;margin:0;font-size:15px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
button{min-width:44px;min-height:44px;padding:0 14px;font:inherit;color:inherit;background:transparent;border:1px solid var(--bd);border-radius:8px;cursor:pointer}
button:disabled{opacity:.35;cursor:default}
#counter{font-variant-numeric:tabular-nums;white-space:nowrap}
main{display:grid;gap:12px;padding:12px}
figure{margin:0}
img{display:block;max-width:100%;height:auto;border:1px solid var(--bd)}
#meta{margin:0;color:var(--mut);font-size:14px;overflow-wrap:anywhere}
#note-line{margin:8px 0 0;white-space:pre-wrap;overflow-wrap:anywhere}
#caption{margin:6px 0 12px;color:var(--mut);font-size:14px;white-space:pre-wrap;overflow-wrap:anywhere}
textarea{display:block;width:100%;min-height:9em;margin-top:4px;padding:8px;font:inherit;color:inherit;background:transparent;border:1px solid var(--bd);border-radius:8px}
#saved{min-height:1.4em;margin:6px 0;font-size:14px;color:var(--mut)}
#saved.ok{color:var(--ok)}#saved.bad{color:var(--bad)}
@media (min-width:1000px){main{grid-template-columns:minmax(0,2fr) minmax(320px,1fr);align-items:start}}
</style>
</head>
<body>
<header><h1 id="title"></h1><button id="prev" aria-label="Previous capture">←</button><span id="counter"></span><button id="next" aria-label="Next capture">→</button></header>
<main>
<figure><a id="full" target="_blank" rel="noopener"><img id="img" alt=""></a></figure>
<section>
<p id="meta"></p><p id="note-line"><b>Why it is here:</b> <span id="note"></span></p><p id="caption"></p>
<label for="remark">Remark on <code id="cid"></code></label>
<textarea id="remark" placeholder="Your remark on this capture, saved as typed"></textarea>
<p id="saved" role="status"></p>
<button id="download">Download remarks</button>
</section>
</main>
<script id="review-data" type="application/json">__DATA__</script>
<script>
"use strict";
// review_page:md:begin (the report's markdown, line for line as review_page.py region() and report_text() write it)
const BEGIN = "<!-" + "- review_page:remarks:begin -" + "->", END = "<!-" + "- review_page:remarks:end -" + "->";
const has = (o, k) => Object.prototype.hasOwnProperty.call(o, k);
function esc(s) { return s.replace(/\|/g, "\\|").replace(/\r\n|\r|\n/g, "<br>"); }
function region(caps, rem) {
  const ids = caps.map((c) => c.id), known = new Set(ids), rows = [], verbatim = [];
  const orphans = Object.keys(rem).filter((k) => !known.has(k)).sort();
  for (const [cid, cap] of caps.map((c) => [c.id, c]).concat(orphans.map((k) => [k, null]))) {
    const where = cap === null ? "(not in this capture set)" : esc([cap.route, cap.state].filter((x) => x).join(" · ")) || "—";
    if (!has(rem, cid)) { rows.push("| `" + cid + "` | " + where + " | _no remark_ | — |"); continue; }
    let cell = esc(rem[cid].text);
    if (cell !== rem[cid].text) { cell += " ↓"; verbatim.push([cid, rem[cid].text]); }
    rows.push("| `" + cid + "` | " + where + " | " + cell + " | " + rem[cid].time + " |");
  }
  let out = [BEGIN, "remarks " + ids.filter((k) => has(rem, k)).length + " · flagged captures " + ids.length +
    " · not in this capture set " + orphans.length, "", "| capture | route / state | remark (verbatim) | time |",
    "|---|---|---|---|"].concat(rows);
  if (verbatim.length) out.push("", "Verbatim — the remarks a table cell cannot hold as typed:");
  for (const [cid, text] of verbatim) {
    const fence = "`".repeat(Math.max(3, 1 + (text.match(/`+/g) || []).reduce((a, m) => Math.max(a, m.length), 0)));
    out = out.concat(["", "`" + cid + "`", fence + "text"], text.split(/\r\n|\r|\n/), [fence]);
  }
  return out.concat([END]);
}
function reportText(title, caps, rem) {
  return ["# Review remarks — " + title, "", "Written by `tools/pb/review_page.py` (PLAYBOOK annex §A.4). The table between the two " +
    "markers is tool-owned (rewritten on every save under `serve`); write the triage and the `Review verdict:` line outside them.",
    ""].concat(region(caps, rem)).join("\n") + "\n";
}
// review_page:md:end
const D = JSON.parse(document.getElementById("review-data").textContent), caps = D.captures, $ = (id) => document.getElementById(id);
const remarks = Object.create(null), pending = Object.create(null), failed = Object.create(null), inflight = {}, timers = {};
const KEY = "review_page:" + location.pathname + ":" + D.report;
let live = false, dirty = false, i = 0;
for (const k of Object.keys(D.remarks)) remarks[k] = D.remarks[k];
function nowIso() {
  const d = new Date(), p = (n) => String(Math.abs(n)).padStart(2, "0"), o = -d.getTimezoneOffset();
  return d.getFullYear() + "-" + p(d.getMonth() + 1) + "-" + p(d.getDate()) + "T" + p(d.getHours()) + ":" + p(d.getMinutes()) + ":" +
    p(d.getSeconds()) + (o < 0 ? "-" : "+") + p(Math.trunc(o / 60)) + ":" + p(o % 60);
}
function mark(text, cls) { $("saved").textContent = text; $("saved").className = cls || ""; }
function status(id) {
  if (!live) return mark("Static page: remarks stay in this browser until you Download remarks" + (dirty ? " (not downloaded yet)." : "."), "bad");
  if (has(failed, id)) return mark("NOT SAVED (" + failed[id] + "). Keep typing to retry, or Download remarks.", "bad");
  if (has(pending, id) || inflight[id]) return mark("saving…");
  mark(has(remarks, id) ? "saved " + remarks[id].time : "autosaves as you type", has(remarks, id) ? "ok" : "");
}
function show(k) {
  i = Math.max(0, Math.min(caps.length - 1, k));
  const c = caps[i], ov = c.overflow === true ? "overflow: yes" : c.overflow === false ? "overflow: no" : "";
  $("img").src = c.src; $("img").alt = c.id; $("full").href = c.src; $("cid").textContent = c.id;
  $("counter").textContent = i + 1 + " / " + caps.length;
  $("meta").textContent = [c.route, c.state, c.viewport, ov].filter((x) => x).join(" · ");
  $("note").textContent = c.note;
  $("caption").textContent = c.caption;
  $("remark").value = has(pending, c.id) ? pending[c.id] : has(remarks, c.id) ? remarks[c.id].text : "";
  $("prev").disabled = i === 0; $("next").disabled = i === caps.length - 1;
  try { history.replaceState(null, "", "#" + encodeURIComponent(c.id)); } catch (e) { /* some browsers refuse it on file:// */ }
  status(c.id);
}
async function flush(id) {
  clearTimeout(timers[id]);
  if (!has(pending, id) || inflight[id]) return;
  const text = pending[id];
  inflight[id] = true;
  try {
    const r = await fetch("/remark", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ id, text }) });
    const j = await r.json().catch(() => ({}));
    if (!r.ok || !j.ok) throw new Error(j.error || "HTTP " + r.status);
    if (pending[id] === text) delete pending[id];
    delete failed[id];
    if (text.trim()) remarks[id] = { text, time: j.time }; else delete remarks[id];
  } catch (e) {
    failed[id] = e.message || String(e);
  } finally {
    inflight[id] = false;
    if (has(pending, id) && !has(failed, id)) flush(id);
    if (caps[i].id === id) status(id);
  }
}
function edited() {
  const id = caps[i].id, text = $("remark").value;
  if (live) {
    pending[id] = text; delete failed[id];
    clearTimeout(timers[id]); timers[id] = setTimeout(() => flush(id), 800);
    return mark("saving…");
  }
  if (text.trim()) remarks[id] = { text, time: nowIso() }; else delete remarks[id];
  dirty = true;
  try { localStorage.setItem(KEY, JSON.stringify(remarks)); } catch (e) { /* storage blocked: Download is the record */ }
  status(id);
}
function download() {
  const rem = Object.create(null), t = nowIso(), a = document.createElement("a");
  for (const k of Object.keys(remarks)) rem[k] = remarks[k];
  for (const k of Object.keys(pending)) { if (pending[k].trim()) rem[k] = { text: pending[k], time: t }; else delete rem[k]; }
  a.href = URL.createObjectURL(new Blob([reportText(D.title, caps, rem)], { type: "text/markdown;charset=utf-8" }));
  a.download = D.report; document.body.appendChild(a); a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1000);
  dirty = false; status(caps[i].id);
}
async function detect() {
  if (!/^https?:$/.test(location.protocol)) return false;
  try {
    const r = await fetch("/api/remarks", { cache: "no-store" }), j = await r.json();
    if (!r.ok || j.tool !== "review_page" || j.report !== D.report) return false;
    for (const k of Object.keys(remarks)) delete remarks[k];
    for (const k of Object.keys(j.remarks)) remarks[k] = j.remarks[k];
    return true;
  } catch (e) { return false; }
}
$("prev").onclick = () => show(i - 1);
$("next").onclick = () => show(i + 1);
$("download").onclick = download;
$("img").onerror = () => { $("meta").textContent = "image not found at " + caps[i].src + " (the page must stay where build wrote it)"; };
$("remark").addEventListener("input", edited);
$("remark").addEventListener("keydown", (e) => { if (e.key === "Escape") e.target.blur(); });
document.addEventListener("keydown", (e) => {
  const t = e.target.tagName;
  if (e.altKey || e.ctrlKey || e.metaKey || t === "TEXTAREA" || t === "INPUT") return;
  if (e.key === "ArrowLeft" || e.key === "ArrowRight") { e.preventDefault(); show(i + (e.key === "ArrowLeft" ? -1 : 1)); }
});
addEventListener("pagehide", () => {
  if (live) for (const k of Object.keys(pending)) navigator.sendBeacon("/remark", JSON.stringify({ id: k, text: pending[k] }));
});
addEventListener("beforeunload", (e) => {
  if (live ? Object.keys(pending).length + Object.keys(failed).length : dirty) { e.preventDefault(); e.returnValue = ""; }
});
(async () => {
  live = await detect();
  if (!live) {
    try {
      const s = JSON.parse(localStorage.getItem(KEY) || "null");
      if (s) { for (const k of Object.keys(remarks)) delete remarks[k]; for (const k of Object.keys(s)) remarks[k] = s[k]; }
    } catch (e) { /* no storage */ }
  }
  $("title").textContent = D.title + " · " + caps.length + " flagged of " + D.total + " captures · " + (live ? "autosave on" : "static page");
  let h = "";
  try { h = decodeURIComponent(location.hash.slice(1)); } catch (e) { /* malformed hash */ }
  show(Math.max(0, caps.findIndex((c) => c.id === h)));
})();
</script>
</body>
</html>
"""


def page_html(data):
    blob = json.dumps(data, ensure_ascii=False).replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")
    return PAGE.replace("__DATA__", blob, 1).replace("__TITLE__", html.escape(data["title"]), 1)


def build(images, captions, report, out=None):
    """Validate the captures and write the page of the flagged ones. → {captures, total, remarks, failures, warnings, out,
    root}; no page on a failure."""
    out = os.path.abspath(out or os.path.join(images, "review.html"))
    caps, total, fails = load_captures(images, captions)
    res = {"captures": caps, "total": total, "remarks": {}, "failures": fails, "warnings": [], "out": out, "root": None}
    if not report.endswith(".md"):
        fails.append(f"--report {report}: not a .md file (reports/review<k>.md)")
    try:
        res["remarks"] = load_remarks(report)
    except (OSError, ValueError) as e:
        fails.append(f"{remarks_path(report)} unreadable ({e}) — never overwritten: repair or move it by hand")
    try:
        res["root"] = os.path.commonpath([os.path.dirname(out), os.path.abspath(images)])
    except ValueError:
        fails.append("--out and --images are on different drives")
    if fails:
        return res
    for c in caps:
        rel = os.path.relpath(os.path.join(os.path.abspath(images), *c["file"].split("/")), os.path.dirname(out))
        c["src"] = urllib.parse.quote(rel.replace(os.sep, "/"))
    write_atomic(out, page_html({"report": os.path.basename(report), "title": os.path.splitext(os.path.basename(report))[0],
                                 "total": total, "captures": caps, "remarks": res["remarks"]}))
    known = {c["id"] for c in caps}
    res["warnings"] = [f"remark on {k}: not in this capture set — kept, listed in the report" for k in res["remarks"] if k not in known]
    return res


# ---------------------------------------------------------------- serve
class Server(http.server.ThreadingHTTPServer):
    allow_reuse_address = os.name != "nt"  # on Windows SO_REUSEADDR would let a second server share a held port
    daemon_threads = True

    def handle_error(self, request, client_address):  # one line, not a traceback (a browser dropping a connection)
        print(shown(f"WARN request from {client_address[0]} failed: {sys.exc_info()[1]!r}"), file=sys.stderr, flush=True)


def current(st):
    return dict(st["remarks"]) if "memory_only" in INJECT else load_remarks(st["report"])


def save_remark(st, cid, text):
    """One POST, under the lock: remarks.json, then the report, both on disk before the answer (memory_only: the bug)."""
    rem = current(st)
    t = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    if text.strip():
        rem[cid] = {"text": text, "time": t}
    else:
        rem.pop(cid, None)
    st["remarks"] = rem
    if "memory_only" not in INJECT:
        p = remarks_path(st["report"])
        write_atomic(p, json.dumps({"report": os.path.basename(st["report"]), "remarks": rem}, ensure_ascii=False, indent=1)
                     + "\n", eol_of(p))
        write_report(st["report"], st["captures"], rem)
    return t, len(rem)


def handler(st):
    class Handler(http.server.BaseHTTPRequestHandler):
        server_version = "review_page"

        def log_message(self, *args):  # the counts line at stop is the record
            pass

        def reply(self, code, body, ctype="application/json; charset=utf-8", headers=()):
            data = body if isinstance(body, bytes) else json.dumps(body, ensure_ascii=False).encode("utf-8")
            self.send_response(code)
            for k, v in (("Content-Type", ctype), ("Content-Length", str(len(data))), ("Cache-Control", "no-store")) + headers:
                self.send_header(k, v)
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            path = urllib.parse.unquote(urllib.parse.urlsplit(self.path).path)
            if path == "/":
                return self.reply(302, b"", "text/plain", (("Location", urllib.parse.quote(st["page_url"])),))
            if path == "/health":
                return self.reply(200, {"status": "ready", "tool": "review_page", "captures": len(st["ids"])})
            if path == "/api/remarks":
                with st["lock"]:
                    rem = current(st)
                return self.reply(200, {"tool": "review_page", "report": os.path.basename(st["report"]), "remarks": rem})
            if path not in st["files"]:
                return self.reply(404, {"ok": False, "error": "not a file of this review"})
            with open(st["files"][path][0], "rb") as f:
                return self.reply(200, f.read(), st["files"][path][1])

        def do_POST(self):
            if urllib.parse.urlsplit(self.path).path != "/remark":
                return self.reply(404, {"ok": False, "error": "POST /remark only"})
            if self.headers.get("Origin") not in (None, "http://" + str(self.headers.get("Host"))):
                return self.reply(403, {"ok": False, "error": "cross-origin POST refused"})
            too_big, body = False, None
            try:
                n = int(self.headers.get("Content-Length") or 0)
                too_big = n > MAX_BODY
                body = None if too_big or n < 0 else json.loads(self.rfile.read(n).decode("utf-8"))
            except (ValueError, UnicodeDecodeError):
                pass
            if too_big:
                self.close_connection = True
                return self.reply(413, {"ok": False, "error": f"body over {MAX_BODY} bytes"})
            if not isinstance(body, dict) or not isinstance(body.get("id"), str) or not isinstance(body.get("text"), str):
                return self.reply(400, {"ok": False, "error": "body must be JSON {id: string, text: string}"})
            with st["lock"]:
                if body["id"] not in st["ids"] and "accept_unknown" not in INJECT:
                    st["rejected"] += 1
                    return self.reply(422, {"ok": False, "error": f"unknown capture id {body['id']!r}"})
                try:
                    t, n_rem = save_remark(st, body["id"], body["text"])
                except (OSError, ValueError) as e:  # the page shows NOT SAVED; the stop verdict counts it
                    st["errors"] += 1
                    return self.reply(500, {"ok": False, "error": shown(f"{type(e).__name__}: {e}")})
                st["saved"] += 1
            return self.reply(200, {"ok": True, "id": body["id"], "time": t, "remarks": n_rem})
    return Handler


def serve_start(images, captions, report, out, host, port):
    """build → bind (a held port is refused, never shared) → the report written from remarks.json → a server thread."""
    res = build(images, captions, report, out)
    if res["failures"]:
        return None, None, res
    url = lambda p: "/" + os.path.relpath(p, res["root"]).replace(os.sep, "/")  # noqa: E731
    files = {url(res["out"]): (res["out"], "text/html; charset=utf-8")}
    for c in res["captures"]:
        fs = os.path.join(os.path.abspath(images), *c["file"].split("/"))
        files[url(fs)] = (fs, IMAGE_MIME[os.path.splitext(fs)[1].lower()])
    st = {"report": report, "captures": res["captures"], "ids": {c["id"] for c in res["captures"]}, "remarks": res["remarks"],
          "files": files, "page_url": url(res["out"]), "lock": threading.Lock(), "saved": 0, "rejected": 0, "errors": 0}
    try:
        httpd = Server((host, port), handler(st))
    except OSError as e:
        res["failures"].append(f"{host}:{port}: {e.strerror or e} — a held port is not reused")
        return None, st, res
    try:
        write_report(report, res["captures"], res["remarks"])
    except (OSError, ValueError) as e:
        httpd.server_close()
        res["failures"].append(f"{report}: {e}")
        return None, st, res
    threading.Thread(target=httpd.serve_forever, kwargs={"poll_interval": 0.05}, daemon=True).start()
    return httpd, st, res


def cmd_build(a):
    res = build(a.images, a.captions, a.report, a.out)
    reason = f"{len(res['failures'])} problem(s) — no page written" if res["failures"] else None
    return verdict_out([("captures", res["total"]), ("flagged", len(res["captures"])), ("remarks", len(res["remarks"])),
                        ("not in this capture set", len(res["warnings"]))], reason, a.json,
                       [] if reason else [f"page {res['out']}"], res["failures"], res["warnings"])


def cmd_serve(a):
    httpd, st, res = serve_start(a.images, a.captions, a.report, a.out, a.host, a.port)
    if httpd is None:
        return verdict_out([("captures", res["total"]), ("flagged", len(res["captures"])), ("remarks saved", 0)],
                           f"{len(res['failures'])} problem(s) — not serving", a.json, (), res["failures"], res["warnings"])
    err = sys.stderr if a.json else sys.stdout
    print(shown(f"serving http://{a.host}:{httpd.server_address[1]}{urllib.parse.quote(st['page_url'])} · "
                f"{len(st['ids'])} flagged of {res['total']} · report {a.report} · remarks {remarks_path(a.report)} · "
                "stop with Ctrl-C or SIGTERM"), file=err, flush=True)
    for w in res["warnings"]:
        print(shown(f"WARN {w}"), file=err, flush=True)
    stop = threading.Event()
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, lambda *_: stop.set())
    while not stop.wait(0.5):
        pass
    httpd.shutdown()
    httpd.server_close()
    try:
        on_file = len(current(st))
    except (OSError, ValueError):
        on_file = "unreadable"
    reason = f"{st['errors']} save(s) failed to write" if st["errors"] else None
    return verdict_out([("remarks saved", st["saved"]), ("rejected", st["rejected"]), ("write errors", st["errors"]),
                        ("remarks on file", on_file), ("flagged", len(st["ids"]))], reason, a.json)


# ---------------------------------------------------------------- selftest
C_READBACK = "a remark POSTed to the served page reads back verbatim from reports/review1.md on disk"
C_UNKNOWN = "a POST for an unknown capture id is rejected (HTTP 422), remarks.json and the report byte-identical"
C_OUTSIDE = "text outside the markers (a Review verdict: line) survives a save"
C_FLAGGED = "the page holds only the captures the TV flagged (2 of 3), each with its note; the unflagged one is absent"
C_PRINTED = "build prints the page's path through shown(): repo-relative, no absolute path (text and --json)"
RED_ARMS = {"accept_unknown": C_UNKNOWN, "memory_only": C_READBACK, "drop_outside": C_OUTSIDE,
            "show_unflagged": C_FLAGGED, "raw_paths": C_PRINTED}

# shown() on synthetic roots — nothing of this box is printed; each case: (name, text, what must be printed)
POSIX = ({"/home/jdoe/src/proj"}, {"/home/jdoe"}, {"jdoe"})
WIN = ({"D:\\dev\\proj"}, {"C:\\Users\\JaneExample", "C:\\Users\\JANEEX~1"}, {"JaneExample", "JANEEX~1"})
POSIX_CASES = [
    ("posix_repo_relative", "page /home/jdoe/src/proj/images/tv1/review.html", "page images/tv1/review.html"),
    ("posix_repo_root_alone", "cwd /home/jdoe/src/proj", "cwd ."),
    ("posix_repo_sibling_is_not_the_repo", "/home/jdoe/src/proj_old/x", "~/src/proj_old/x"),
    ("posix_home_relative", "/home/jdoe/.cache/tool/x.png", "~/.cache/tool/x.png"),
    ("posix_other_user_and_tmp_untouched", "/home/jdoe2/x · /tmp/pb_review_1/r.md", "/home/jdoe2/x · /tmp/pb_review_1/r.md"),
    ("posix_scratchpad_name_left", "/tmp/agent-1000/-home-jdoe-src-proj/s/x", "/tmp/agent-1000/-home-<home>-src-proj/s/x"),
    ("posix_several_in_one_text", 'File "/home/jdoe/src/proj/a.py" · /home/jdoe/b', 'File "a.py" · ~/b'),
    ("posix_root_inside_another_path", "/mnt/bak/home/jdoe/src/proj/x", "/mnt/bak/home/<home>/src/proj/x"),
]
WIN_CASES = [
    ("win_temp_short_form", "fixture at C:\\Users\\JANEEX~1\\AppData\\Local\\Temp\\pb_review_1\\cap1.png",
     "fixture at ~\\AppData\\Local\\Temp\\pb_review_1\\cap1.png"),
    ("win_long_form_any_case", "c:\\users\\janeexample\\appdata\\x", "~\\appdata\\x"),
    ("win_forward_slashes", "C:/Users/JaneExample/AppData/x", "~/AppData/x"),
    ("win_repr_doubled", "PermissionError: [WinError 5] Access is denied: 'C:\\\\Users\\\\JANEEX~1\\\\AppData\\\\t'",
     "PermissionError: [WinError 5] Access is denied: '~\\\\AppData\\\\t'"),
    ("win_repo_on_another_drive", "page D:\\dev\\proj\\images\\review.html", "page images\\review.html"),
    ("win_owner_name_left", "-rw-r--r-- 1 corp+JaneExample 4 Jan 1", "-rw-r--r-- 1 corp+<home> 4 Jan 1"),
]


def wrong(rw, cases):
    """The names of the cases `rw` prints differently from what must be printed."""
    return [c for c, text, want in cases if rw(text) != want]


def write_png(path, w, h, rgb):
    chunk = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)  # noqa: E731
    raw = b"".join(b"\x00" + bytes(rgb) * w for _ in range(h))
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def png_ok(path, w, h):
    with open(path, "rb") as f:
        b = f.read()
    pos, idat, dims = 8, b"", None
    while b[:8] == b"\x89PNG\r\n\x1a\n" and pos + 12 <= len(b):
        n, t = struct.unpack(">I4s", b[pos:pos + 8])
        d = b[pos + 8:pos + 8 + n]
        if struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF) != b[pos + 8 + n:pos + 12 + n]:
            return False
        dims = struct.unpack(">II", d[:8]) if t == b"IHDR" else dims
        idat += d if t == b"IDAT" else b""
        if t == b"IEND":
            return dims == (w, h) and len(zlib.decompress(idat)) == h * (1 + 3 * w)
        pos += 12 + n
    return False


@functools.lru_cache(maxsize=1)
def opener():
    """One opener for the run: each build_opener() loads the TLS trust store (≈10 ms), never used on 127.0.0.1."""
    return urllib.request.build_opener(urllib.request.ProxyHandler({}))


def fetch(method, url, body=None, headers=None):
    """→ (status, bytes); 0 when nothing answers. No proxy: the selftest only talks to 127.0.0.1."""
    req = urllib.request.Request(url, data=body, method=method, headers=headers or {})
    try:
        with opener().open(req, timeout=10) as r:
            return r.status, r.read()
    except urllib.error.HTTPError as e:
        return e.code, e.read()
    except (urllib.error.URLError, OSError):
        return 0, b""


def check_shown(check):
    """shown() on synthetic roots, its plants, and on this box (booleans only: the home folder's name is never printed,
    not even here)."""
    posix, win = Rewriter(*POSIX), Rewriter(*WIN)
    dump = json.dumps({"png": "C:\\Users\\JaneExample\\x.png", "log": "D:\\dev\\proj\\logs\\a.log"})
    try:
        back = json.loads(win(dump))
    except ValueError:
        back = None
    root = Rewriter({"/opt/proj"}, {"/root"}, {"root"})
    check("shown(): the synthetic POSIX and Windows cases print repo-relative, `~`, `<home>` — 8.3 form, `/`, a repr's "
          "doubled `\\\\`, a sibling folder and a root inside another path left alone; a JSON dump still decodes; a "
          "generic home name never rewritten as a word",
          not wrong(posix, POSIX_CASES) and not wrong(win, WIN_CASES) and back == {"png": "~\\x.png", "log": "logs\\a.log"}
          and root("the repo root · /root/x") == "the repo root · ~/x")
    plants = [(Rewriter(WIN[0], {"C:\\Users\\JaneExample"}, {"JaneExample"}), WIN_CASES, "win_temp_short_form"),
              (Rewriter(POSIX[0], POSIX[1], set()), POSIX_CASES, "posix_scratchpad_name_left"),
              (Rewriter(*POSIX, end=""), POSIX_CASES, "posix_repo_sibling_is_not_the_repo"),
              (Rewriter(*POSIX, start=""), POSIX_CASES, "posix_root_inside_another_path"),
              (Rewriter(*WIN, doubled=False), WIN_CASES, "win_repr_doubled"),
              (lambda text: text, WIN_CASES, "win_temp_short_form")]
    check("PLANT shown() with no 8.3 form · no name pass · no end guard · no start guard · no doubled spelling · no "
          "rewrite at all → each caught by its own case", all(want in wrong(rw, cases) for rw, cases, want in plants))
    home = None
    with contextlib.suppress(RuntimeError, KeyError, OSError):
        home = Path.home()
    here = project_root(Path(__file__).resolve())
    forms = sorted({str(home), *native_forms(str(home))}) if home else []
    names = {Path(f).name.lower() for f in forms} - GENERIC
    probe = " · ".join(forms + [f"-home-{Path(f).name}-x" for f in forms] + [tempfile.gettempdir()])
    check("shown() on this box: this file prints as tools/pb/review_page.py (the root two folders up), a path under the "
          "home folder `~`-relative, and the home folder's name survives in no spelling (the temp dir, a scratchpad-style name)",
          shown(Path(__file__).resolve()) == os.path.join("tools", "pb", "review_page.py")
          and (home is None or here == home or here in home.parents or shown(home / "y") == "~" + os.sep + "y")
          and not any(len(n) >= 3 and n in shown(probe).lower() for n in names) and this_box() is this_box())


def suite(check, bugs, node=True):
    """One pass over a throwaway fixture with `bugs` switched on in memory. Deletes only its own temp dir. → evidence."""
    global REWRITE, REPLACE_WAIT_S
    INJECT.clear()
    INJECT.update(bugs)
    tmp, servers, ev = tempfile.mkdtemp(prefix="pb_review_selftest_"), [], {}
    stop = lambda s: (s.shutdown(), s.server_close(), servers.remove(s)) if s in servers else None  # noqa: E731
    try:
        img, rep = os.path.join(tmp, "images", "tv1"), os.path.join(tmp, "reports", "review1.md")
        os.makedirs(img)
        sizes = ((64, 40, (200, 40, 40)), (40, 64, (40, 160, 60)), (48, 48, (40, 60, 220)))
        for k, (w, h, rgb) in enumerate(sizes):
            write_png(os.path.join(img, f"cap{k + 1}.png"), w, h, rgb)
        check("fixture: 3 PNGs generated with stdlib zlib are valid (signature, CRCs, IHDR, IDAT size)",
              all(png_ok(os.path.join(img, f"cap{k + 1}.png"), w, h) for k, (w, h, _) in enumerate(sizes)))
        evil = "The empty state reads as an error — your call? </script><script>alert(1)</script>"
        caps = [{"id": "home__empty__1280x800", "file": "cap1.png", "route": "home", "state": "empty",
                 "viewport": "1280x800", "overflow": None, "caption": "« Accueil » — first visit", "flag": True, "note": evil},
                {"id": "settings__long__390x844", "file": "cap2.png", "route": "settings", "state": "long | names",
                 "viewport": "390x844", "overflow": True, "flag": True, "note": "a long name wraps under its toggle"},
                {"id": "settings__default__390x844", "file": "cap3.png", "route": "settings", "state": "default",
                 "viewport": "390x844", "overflow": False, "flag": False}]
        write_atomic(os.path.join(img, "captions.json"), json.dumps(caps, ensure_ascii=False, indent=1) + "\n")
        res, page = build(img, None, rep), os.path.join(img, "review.html")
        check("build on the clean fixture → no failure, page written beside the images", not res["failures"] and os.path.isfile(page))
        text = read_text(page) if os.path.isfile(page) else ""
        m = re.search(r'<script id="review-data" type="application/json">(.*?)</script>', text, re.S)
        try:  # the HTML parser ends the data element at the first </script: parse exactly what a browser would
            data = json.loads(m.group(1))
        except (AttributeError, ValueError):
            data = {"captures": []}
        on_page = data["captures"]
        check(C_FLAGGED, [c.get("id") for c in on_page] == [caps[0]["id"], caps[1]["id"]] and data.get("total") == 3
              and [c.get("note") for c in on_page] == [evil, caps[1]["note"]] and caps[2]["id"] not in text
              and "cap3.png" not in text)
        srcs = [c.get("src", "") for c in on_page]
        check("page: images by relative path, never embedded (each src resolves to its file; no data: URI)",
              len(srcs) == 2 and "data:" not in text and all(not s.startswith(("/", "http")) and os.path.isfile(
                  os.path.join(img, urllib.parse.unquote(s))) for s in srcs))
        check("page: a note holding </script> stays data (never raw in the HTML, read back verbatim)",
              "</script><script>alert" not in text and bool(on_page) and on_page[0].get("note") == evil)
        check("page: counter, prev / next, ← → keys, the note under the caption, remarks box, 800 ms autosave, "
              "Download remarks, mobile-first",
              all(s in text for s in ('id="counter"', 'id="prev"', 'id="next"', '"ArrowLeft"', '"ArrowRight"', 'id="note"',
                                      '$("note").textContent = c.note', "<textarea", "flush(id), 800)", "Download remarks",
                                      'name="viewport"',
                                      "@media (min-width")))
        REWRITE, outs = Rewriter({tmp}, set(), set()), []
        try:
            for extra in ([], ["--json"]):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    rc = main(["build", "--images", img, "--report", rep] + extra)
                outs.append((rc, buf.getvalue()))
        finally:
            REWRITE = None
        want = "page " + os.path.join("images", "tv1", "review.html")
        try:
            printed = json.loads(outs[1][1])["lines"]
        except (ValueError, KeyError, TypeError):
            printed = None
        ev["printed"] = all(rc == 0 and tmp not in o for rc, o in outs) and want in outs[0][1].splitlines() and printed == [want]
        check(C_PRINTED, ev["printed"])
        check_shown(check)
        planted = os.path.join(tmp, "planted_captions.json")
        for name, entries, word in (("a caption naming a missing image", caps + [{"id": "ghost", "file": "ghost.png"}], "missing"),
                                    ("a duplicate capture id", caps + [dict(caps[0])], "duplicate"),
                                    ("an image with no captions.json entry", caps[:2], "no captions.json entry"),
                                    ("a file path escaping --images", caps + [{"id": "up", "file": "../../x.png"}], "inside"),
                                    ("a flagged capture with no note", [dict(caps[0], note="  ")] + caps[1:], "no note"),
                                    ("a flag that is not true or false", caps[:2] + [dict(caps[2], flag="yes")], "true or false"),
                                    ("nothing flagged", [dict(c, flag=False) for c in caps], "nothing flagged")):
            write_atomic(planted, json.dumps(entries))
            fails = build(img, planted, rep, os.path.join(tmp, "planted.html"))["failures"]
            check(f"PLANT {name} → build NO-GO on one failure naming it, no page", len(fails) == 1 and word in fails[0]
                  and not os.path.exists(os.path.join(tmp, "planted.html")))
        bare = os.path.join(tmp, "images", "bare")
        os.makedirs(bare)
        write_png(os.path.join(bare, "cap1.png"), 8, 8, (0, 0, 0))
        fails = build(bare, None, rep, os.path.join(tmp, "planted.html"))["failures"]
        check("PLANT no captions.json beside the images → build NO-GO, no page (nothing is flagged)",
              any("not found" in f for f in fails) and not os.path.exists(os.path.join(tmp, "planted.html")))
        write_atomic(planted, "\ufeff" + json.dumps(caps))
        bom = build(img, planted, rep, os.path.join(tmp, "bom.html"))
        check("a captions.json written with a byte-order mark reads", not bom["failures"] and len(bom["captures"]) == 2)
        bad = os.path.join(tmp, "reports", "corrupt.md")
        write_atomic(remarks_path(bad), '{"remarks": {"x": "a remark that is not an object"}}\n')
        fails, (h, _, _) = build(img, None, bad, os.path.join(tmp, "planted.html"))["failures"], serve_start(img, None, bad, None, "127.0.0.1", 0)
        check("PLANT an unreadable remarks.json → build and serve NO-GO, the file byte-identical, no report",
              any("unreadable" in f for f in fails) and h is None and not os.path.exists(bad)
              and read_text(remarks_path(bad)) == '{"remarks": {"x": "a remark that is not an object"}}\n')
        wait, REPLACE_WAIT_S, tries = REPLACE_WAIT_S, 0.001, []
        try:
            def held(k):
                tries.append(1)
                if len(tries) <= k:
                    raise PermissionError(13, "held open by another process")
                return "landed"
            landed, first = insist(held, 2), len(tries)
            tries.clear()
            try:
                insist(held, 99)
                stayed = False
            except PermissionError:
                stayed = True
            blocker = os.path.join(tmp, "blocker")
            os.makedirs(os.path.join(blocker, "keep"))
            try:
                write_atomic(blocker, "x\n")
                raised = False
            except OSError:
                raised = True
        finally:
            REPLACE_WAIT_S = wait
        check("a replace refused while another process holds the file is retried and lands; one refused every time is "
              f"raised after {REPLACE_TRIES} tries — bounded", landed == "landed" and first == 3 and stayed
              and len(tries) == REPLACE_TRIES)
        check("a write whose replace fails leaves the target as it was and no temp behind",
              raised and os.path.isdir(os.path.join(blocker, "keep")) and not any(
                  f.startswith(".review_tmp_") for f in os.listdir(tmp)))
        httpd, st, res = serve_start(img, None, rep, None, "127.0.0.1", 0)
        servers += [httpd] if httpd else []
        base = f"http://127.0.0.1:{httpd.server_address[1]}" if httpd else "http://127.0.0.1:9"
        h, _, r = serve_start(img, None, os.path.join(tmp, "reports", "held.md"), os.path.join(tmp, "held.html"), "127.0.0.1",
                              httpd.server_address[1] if httpd else 9)
        servers += [h] if h else []
        check("PLANT serve on the port a running review holds → refused (not reused), the running one still answers",
              h is None and any("not reused" in f for f in r["failures"]) and fetch("GET", base + "/health")[0] == 200)
        code, body = fetch("GET", base + "/health")
        check("serve on an ephemeral port → GET /health {status: ready}", code == 200 and json.loads(body)["status"] == "ready")
        with open(page, "rb") as f:
            check("serve: GET / lands on the built page, byte for byte", fetch("GET", base + "/") == (200, f.read()))
        got = [fetch("GET", urllib.parse.urljoin(base + "/review.html", s)) for s in srcs]
        check("serve: each capture's relative src fetches its image bytes", len(got) == 2 and all(
            g == (200, open(os.path.join(img, f"cap{k + 1}.png"), "rb").read()) for k, g in enumerate(got)))
        check("serve: anything but the page and its flagged images → 404 (captions.json, the unflagged image, ../ traversal)",
              fetch("GET", base + "/captions.json")[0] == 404 and fetch("GET", base + "/cap3.png")[0] == 404
              and fetch("GET", base + "/%2e%2e/%2e%2e/etc/passwd")[0] == 404)
        disk = read_text(rep) if os.path.isfile(rep) else ""
        check("serve writes the report at start: 2 rows without a remark, between the markers",
              disk.count("_no remark_") == 2 and 0 <= disk.find(BEGIN) < disk.find(END))
        write_atomic(rep, disk + "\nReview verdict: GO\n")
        t1 = "Reads as an error « état vide » — keep it, but say why <b>&amp;</b> \"q\" 'q' ≤ ≥"
        t2 = "row | one\nsecond line with ``` a fence\n\ttab, trailing spaces  "
        post = lambda d, ctype="application/json", extra=(): fetch("POST", base + "/remark", json.dumps(d, ensure_ascii=False)  # noqa: E731
                                                                   .encode("utf-8"), dict([("Content-Type", ctype), *extra]))
        codes = (post({"id": caps[0]["id"], "text": t1})[0], post({"id": caps[1]["id"], "text": t2}, "text/plain;charset=UTF-8")[0])
        disk = read_text(rep).replace("\r\n", "\n") if os.path.isfile(rep) else ""
        ev["readback"] = [codes[0] == 200 and t1 in disk, codes[1] == 200 and t2 in disk]
        check(C_READBACK, all(ev["readback"]))
        try:
            saved = load_remarks(rep)
        except (OSError, ValueError):
            saved = {}
        check("remarks.json beside the report holds both remarks verbatim, each with its time",
              [saved.get(c["id"], {}).get("text") for c in caps[:2]] == [t1, t2] and all(
                  re.fullmatch(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d[+-]\d\d:\d\d", v["time"]) for v in saved.values()))
        check("report: a row per flagged capture; the multi-line remark escaped in its cell (\\| and <br>), fenced verbatim below",
              disk.count("\n| `") == 2 and "row \\| one<br>second line" in disk and "````text\n" + t2 + "\n````" in disk)
        check(C_OUTSIDE, disk.count("\nReview verdict: GO\n") == 1)
        before = [read_text(p) if os.path.isfile(p) else None for p in (rep, remarks_path(rep))]
        code = post({"id": "no_such_capture", "text": "planted remark"})[0]
        ev["unknown"] = (code, before == [read_text(p) if os.path.isfile(p) else None for p in (rep, remarks_path(rep))])
        check(C_UNKNOWN, ev["unknown"] == (422, True))
        code = post({"id": caps[2]["id"], "text": "a remark on a capture the lead was not shown"})[0]
        ev["unflagged"] = (code, before == [read_text(p) if os.path.isfile(p) else None for p in (rep, remarks_path(rep))])
        check("a POST for the capture the TV did not flag is rejected (HTTP 422), nothing written", ev["unflagged"] == (422, True))
        check("POST: malformed JSON 400 · non-string text 400 · foreign Origin 403 · another path 404 · over 1 MiB 413", (
            fetch("POST", base + "/remark", b"{not json", {"Content-Type": "application/json"})[0],
            post({"id": caps[0]["id"], "text": 7})[0], post({"id": caps[0]["id"], "text": "x"}, extra=[("Origin", "http://evil.example")])[0],
            fetch("POST", base + "/other", b"{}")[0], fetch("POST", base + "/remark", b"", {"Content-Length": str(MAX_BODY + 1)})[0])
              == (400, 400, 403, 404, 413))
        post({"id": caps[1]["id"], "text": "  \n"})
        disk = read_text(rep).replace("\r\n", "\n")
        check("an emptied box clears its remark from remarks.json and the report",
              caps[1]["id"] not in load_remarks(rep) and t2 not in disk and disk.count("_no remark_") == 1)
        stop(httpd)
        httpd, st, res = serve_start(img, None, rep, None, "127.0.0.1", 0)
        servers += [httpd] if httpd else []
        base = f"http://127.0.0.1:{httpd.server_address[1]}" if httpd else "http://127.0.0.1:9"
        code, body = fetch("GET", base + "/api/remarks")
        check("a restarted server serves the saved remark: nothing lived only in the browser or in memory",
              code == 200 and json.loads(body)["remarks"].get(caps[0]["id"], {}).get("text") == t1)
        write_atomic(rep, read_text(rep), "\r\n")
        post({"id": caps[1]["id"], "text": "crlf kept"})
        raw = read_text(rep)
        check("a save keeps the report's CRLF line endings", "crlf kept" in raw and "\n" not in raw.replace("\r\n", ""))
        js = re.search(r"// review_page:md:begin[^\n]*\n(.*?)// review_page:md:end", text, re.S)
        if node and not shutil.which("node"):
            ev["node"] = "SKIP"
        elif node:
            t = "2000-01-01T12:00:00+00:00"
            rem = {caps[0]["id"]: {"text": t1, "time": t}, caps[1]["id"]: {"text": t2, "time": t}, "gone__id": {"text": "a | b", "time": t}}
            write_atomic(os.path.join(tmp, "md.js"), js.group(1) if js else "")
            write_atomic(os.path.join(tmp, "md.json"), json.dumps({"captures": on_page, "remarks": rem}, ensure_ascii=False))
            harness = ("const fs = require('fs'), f = new Function(fs.readFileSync(process.argv[1], 'utf8') + '\\nreturn reportText;')();"
                       "const d = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));"
                       "process.stdout.write(f('review1', d.captures, d.remarks));")
            r = subprocess.run([shutil.which("node"), "-e", harness, os.path.join(tmp, "md.js"), os.path.join(tmp, "md.json")],
                               capture_output=True, timeout=60)
            check("static page: Download remarks assembles the same markdown as the server (the page's JS, under node)",
                  r.returncode == 0 and r.stdout.decode("utf-8") == report_text("review1", on_page, rem))
    except Exception as e:  # a crash is a failed check, never a missing verdict
        check(f"suite raised {type(e).__name__}: {e}", False)
    finally:
        for s in list(servers):
            stop(s)
        INJECT.clear()
        REWRITE = None
        shutil.rmtree(tmp, ignore_errors=True)
    check("fixture removed (only the selftest's own temp dir)", not os.path.exists(tmp))
    return ev


def cmd_selftest(a):
    """§A.4: three generated PNGs, build, serve on an ephemeral port, POST one remark, read the report back from disk and
    find it verbatim → GO; a POST for an unknown capture id → rejected. Red arms: each planted bug, switched on in memory,
    must turn its own check red (`--inject <bug>` runs the whole suite with it: NO-GO)."""
    checks = []
    check = lambda name, ok: checks.append((name, bool(ok)))  # noqa: E731
    ev = suite(check, set(a.inject or ()), node=not a.inject)
    arms = []
    for bug, target in ({} if a.inject else RED_ARMS).items():
        sub = []
        suite(lambda n, ok: sub.append((n, bool(ok))), {bug}, node=False)
        arms.append(f"{bug} → {'red' if (target, False) in sub else 'STAYED GREEN'}")
        check(f"INJECT {bug} (in memory) turns red: {target}", (target, False) in sub)
    failed = [n for n, ok in checks if not ok]
    rb, (code, same) = ev.get("readback", []), ev.get("unknown", (None, False))
    lines = [f"INJECT {', '.join(a.inject)} (in memory)"] if a.inject else [
        f"  read back verbatim from the report on disk after a served POST: {sum(rb)}/{len(rb)} (JSON + text/plain beacon shape)",
        f"  unknown capture id POST: HTTP {code}, remarks.json + report {'unchanged' if same else 'CHANGED'}; "
        f"the unflagged capture's POST: HTTP {ev.get('unflagged', (None,))[0]}",
        "  red arms: " + " · ".join(arms)]
    skips = ["static Download remarks = server markdown: node not on PATH — NOT PROVEN (source only)"] if ev.get("node") else []
    reason = (f"{len(failed)} check(s) failed" + (f" under INJECT {', '.join(a.inject)}" if a.inject else "")) if failed else None
    return verdict_out([("checks", len(checks) + len(skips)), ("passed", len(checks) - len(failed)), ("failed", len(failed)),
                        ("skipped", len(skips))], reason, a.json, lines, failed, (), skips)


# ---------------------------------------------------------------- main
def main(argv=None):
    ap = argparse.ArgumentParser(prog="review_page.py", description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="command", required=True)
    for name in ("build", "serve"):
        sp = sub.add_parser(name)
        sp.add_argument("--images", required=True)
        sp.add_argument("--captions")
        sp.add_argument("--report", required=True)
        sp.add_argument("--out")
        sp.add_argument("--json", action="store_true")
        if name == "serve":
            sp.add_argument("--host", default="127.0.0.1")
            sp.add_argument("--port", type=int, default=8765)
    sp = sub.add_parser("selftest")
    sp.add_argument("--inject", action="append", choices=INJECTS)
    sp.add_argument("--json", action="store_true")
    a = ap.parse_args(argv)
    return {"build": cmd_build, "serve": cmd_serve, "selftest": cmd_selftest}[a.command](a)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):  # a piped Windows console is cp1252; a caption can carry `«`, `≤`, `→`
        with contextlib.suppress(AttributeError, ValueError, OSError):
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    sys.exit(main())
