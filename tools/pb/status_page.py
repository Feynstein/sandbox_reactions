#!/usr/bin/env python3
"""status_page.py — the orchestrator's two-way page (PLAYBOOK annex §A.6, §7 O8).

Python 3.9+, standard library only, Windows and POSIX. Relative paths resolve from the current directory.
  status_page.py serve [--plan <path>] [--host 127.0.0.1|0.0.0.0|tailscale] [--port 8790]
  status_page.py mark <ID> running|done|failed|blocked [--note "<t>"] [--plan <path>]
  status_page.py ask "<question>" [--critical] [--id q<n>]
  status_page.py answer <qid> "<text>"
  status_page.py poll [--wait <s>]
  status_page.py attention "<text>"
  status_page.py selftest [--inject <bug>]
Every call but selftest takes --state <dir> (default tools/pb/.status) and --json; --plan defaults to plan.py's (the
newest milestones/*/*implementation_plan.md with open work).
State: three append-only JSON Lines files under --state — runs.jsonl, questions.jsonl, attention.jsonl — one event per
line, never rewritten: an answer is a new line and the last one wins. The folder holds a .gitignore of `*`, so a run's
record is never committed. Every append holds plan.py's lock across its read-modify-write and lands atomically.
The page (GET /) reads the plan through plan.py's own functions — this file holds no plan parser (§A.0): the queue in
file order (every block the run marked, and every TODO, IN PROGRESS or BLOCKED one) with its plan status, run status
and running time (now − start while running); the attention list, newest first; the questions, open critical ones on
top and marked, each with an answer form (POST /answer). GET /api/state is the same as JSON; GET /health answers
{"status": "ready"}. --host tailscale resolves the address with `tailscale ip -4` at run time and prints the URL; the
address is never written into any file, and an address that cannot be resolved stops the serve (no fallback). The
page's port is its own: a port that tools/pb/launch.json gives a service is refused, a held port is never shared.
poll prints the answered questions as JSON (one line each; --json: one document) and exits 3 while a critical question
is unanswered — the orchestrator halts that block (§7 O8): exit 0 go on, 3 halt, 1 the state cannot be read.
Every command ends with one counts line, failures only when non-zero, and `=== GO ===` / `=== NO-GO: <reason> ===`
(exit 0 / 1, poll's 3 above); --json behind a flag. UTF-8, line endings kept, never deletes a file, never runs git.
"""
import argparse
import contextlib
import datetime
import html
import http.server
import io
import ipaddress
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
if HERE not in sys.path:
    sys.path.insert(0, HERE)
import plan as pb_plan  # noqa: E402 — §A.0: the plan-parsing functions are plan.py's; a second parser would drift

RUN_STATES = ("running", "done", "failed", "blocked")
QUEUED = ("TODO", "IN PROGRESS", "BLOCKED")  # plan states the queue shows even when the run never marked them
FILES = {"runs": "runs.jsonl", "questions": "questions.jsonl", "attention": "attention.jsonl"}
QID = re.compile(r"q[1-9][0-9]*$")
DEFAULT_STATE = os.path.join("tools", "pb", ".status")
DEFAULT_PORT = 8790
MAX_BODY = 64 * 1024
REFRESH_S = 15
TAILSCALE = ["tailscale", "ip", "-4"]  # run by serve --host tailscale only; the selftest swaps it in memory
LAUNCH_JSON = os.path.join(HERE, "launch.json")  # the product's services and ports (§A.3): never the page's port
POLL_STEP_S = 1.0
APPEND_DELAY_S = 0.0  # selftest seam: widens read → write so the lock is proven, not hoped for
CLOCK = None  # selftest seam: a fixed aware datetime for the running times; else the wall clock
INJECTS = ("own_parser", "critical_ignored", "rewrite_state", "accept_unknown", "no_lock")
INJECT = set()  # in-memory bug switches: set only by `selftest` (its red arms), never by a real call


class StateError(Exception):
    """The state folder cannot be read or written as asked: a NO-GO naming the file, never a traceback."""


class UnknownQuestion(StateError):
    pass


# ---------------------------------------------------------------- time
def now():
    return CLOCK or datetime.datetime.now().astimezone().replace(microsecond=0)


def parse_time(s):
    t = datetime.datetime.fromisoformat(s)
    return t if t.tzinfo else t.astimezone()


def span(seconds):
    if seconds is None:
        return "—"
    if seconds < 60:
        return f"{seconds} s"
    if seconds < 3600:
        return f"{seconds // 60} min"
    return f"{seconds // 3600} h {seconds % 3600 // 60:02d}"


def clock_text(t, today):
    return t.strftime("%H:%M") if t.date() == today else t.strftime("%Y-%m-%d %H:%M")


# ---------------------------------------------------------------- the plan, through plan.py
def blocks_of(path):
    """[(id, state, status text)] in file order, read by plan.py's own functions (§A.0)."""
    if "own_parser" in INJECT:  # the drift the selftest must catch: a naive `## <id>` scan, fences ignored
        text, _ = pb_plan.read_text(path)
        return [(m.group(1), None, "") for m in re.finditer(r"^## (\d*M[\d.]+[a-z]?-\S+)", text, re.M)]
    p = pb_plan.Plan(path)
    return [(b.id, b.state, b.status_text or "") for b in p.blocks]


def plan_path(given):
    path = given or pb_plan.default_plan()
    if not path or not os.path.exists(path):
        raise StateError(f"no plan file {given or ''}".rstrip() + " (--plan <path>, or milestones/*/*implementation_plan.md)")
    return path


# ---------------------------------------------------------------- state
def state_file(state, kind):
    return os.path.join(state, FILES[kind])


def ensure_state(state):
    """The folder, and its `.gitignore` of `*` — a run's questions and answers never enter a commit."""
    os.makedirs(state, exist_ok=True)
    ignore = os.path.join(state, ".gitignore")
    if not os.path.exists(ignore):
        pb_plan.write_text(ignore, "*\n", "\n")


@contextlib.contextmanager
def locked(state, kind):
    """plan.py's lock on the state file (a sibling `.<file>.lock`), held across a whole read-modify-write."""
    ensure_state(state)
    lock = None if "no_lock" in INJECT else pb_plan.PlanLock(state_file(state, kind))
    if lock is not None and not lock.acquire():
        raise StateError(f"{state_file(state, kind)}: lock not taken within {lock.timeout:.0f}s — another write is in "
                         "progress; nothing written")
    try:
        yield
    finally:
        if lock is not None:
            lock.release()


def write_event(state, kind, event):
    """One event as one line at the end of the file, landed atomically (plan.py's write). The earlier bytes are kept
    as they are — the file is append-only; a last line left without its newline by a hand edit is closed first."""
    path = state_file(state, kind)
    text, nl = pb_plan.read_text(path) if os.path.exists(path) else ("", "\n")
    if APPEND_DELAY_S:
        time.sleep(APPEND_DELAY_S)
    if "rewrite_state" in INJECT:
        text = ""
    elif text and not text.endswith("\n"):
        text += "\n"
    pb_plan.write_text(path, text + json.dumps(event, ensure_ascii=False) + "\n", nl)


def append(state, kind, event):
    with locked(state, kind):
        write_event(state, kind, event)


def read_events(state, kind, problems):
    path = state_file(state, kind)
    if not os.path.exists(path):
        return []
    try:
        text, _ = pb_plan.read_text(path)
    except (OSError, UnicodeDecodeError) as e:
        problems.append(f"{path}: unreadable — {e}")
        return []
    out = []
    for n, line in enumerate(text.lstrip("\ufeff").split("\n"), 1):
        if not line.strip():
            continue
        try:
            ev = json.loads(line)
            if not isinstance(ev, dict):
                raise ValueError("not a JSON object")
            ev["time"] = parse_time(ev["time"])
        except (ValueError, KeyError, TypeError) as e:
            problems.append(f"{path}:{n}: {e.__class__.__name__}: {e} — the line is skipped, the file left as it is")
            continue
        out.append((n, ev))
    return out


def load(state):
    """→ (runs {id: [event]}, questions {qid: question} in ask order, attention [event], problems [str])."""
    problems, runs, questions, attention = [], {}, {}, []
    for n, ev in read_events(state, "runs", problems):
        if not isinstance(ev.get("id"), str) or ev.get("run") not in RUN_STATES:
            problems.append(f"{state_file(state, 'runs')}:{n}: a run event is {{time, id, run: {'|'.join(RUN_STATES)}}}")
            continue
        runs.setdefault(ev["id"], []).append(ev)
    for n, ev in read_events(state, "questions", problems):
        where, qid, text = f"{state_file(state, 'questions')}:{n}", ev.get("id"), ev.get("text")
        if ev.get("event") == "ask" and isinstance(qid, str) and QID.match(qid) and isinstance(text, str):
            if qid in questions:
                problems.append(f"{where}: {qid} asked twice — the first kept")
                continue
            questions[qid] = {"id": qid, "text": text, "critical": ev.get("critical") is True, "asked": ev["time"],
                              "answers": []}
        elif ev.get("event") == "answer" and isinstance(text, str) and qid in questions:
            questions[qid]["answers"].append({"text": text, "time": ev["time"], "via": ev.get("via", "")})
        else:
            problems.append(f"{where}: neither an ask {{id: q<n>, text, critical}} nor an answer to a question asked above")
    for n, ev in read_events(state, "attention", problems):
        if isinstance(ev.get("text"), str):
            attention.append(ev)
        else:
            problems.append(f"{state_file(state, 'attention')}:{n}: an attention event is {{time, text}}")
    return runs, questions, attention, problems


def run_summary(events, t_now):
    """The last mark, and the running time of the last `running` span: now − start while it runs, else its length."""
    last, starts, elapsed = events[-1], [i for i, e in enumerate(events) if e["run"] == "running"], None
    if starts:
        i = starts[-1]
        end = events[i + 1]["time"] if i + 1 < len(events) else t_now
        elapsed = max(0, int((end - events[i]["time"]).total_seconds()))
    return {"run": last["run"], "since": last["time"].isoformat(), "elapsed_s": elapsed, "note": last.get("note") or ""}


def open_critical(questions):
    if "critical_ignored" in INJECT:
        return []
    return [q for q in questions.values() if q["critical"] and not q["answers"]]


def answer_question(state, qid, text, via):
    """Appends the answer under the lock, after checking the question exists → the answer it supersedes, or None."""
    with locked(state, "questions"):
        _, questions, _, problems = load(state)
        if problems:
            raise StateError(f"{len(problems)} state problem(s), nothing written: " + " | ".join(problems))
        if qid not in questions and "accept_unknown" not in INJECT:
            raise UnknownQuestion(f"no question {qid!r} in {state_file(state, 'questions')}"
                                  + (f" — the questions: {' · '.join(questions)}" if questions else " — none asked yet"))
        write_event(state, "questions", {"time": now().isoformat(), "event": "answer", "id": qid, "text": text, "via": via})
        prev = questions[qid]["answers"] if qid in questions else []
        return prev[-1] if prev else None


# ---------------------------------------------------------------- the view (page and API)
def view(plan, state):
    t_now = now()
    runs, questions, attention, problems = load(state)
    try:
        blocks = blocks_of(plan)
    except (OSError, UnicodeDecodeError) as e:
        blocks = []
        problems.append(f"{plan}: unreadable — {e}")
    queue, in_plan = [], set()
    for bid, st, text in blocks:
        in_plan.add(bid)
        if bid in runs or st in QUEUED:
            queue.append(dict({"id": bid, "plan": text or (st or "?")},
                              **(run_summary(runs[bid], t_now) if bid in runs else
                                 {"run": "", "since": "", "elapsed_s": None, "note": ""})))
    for bid in runs:
        if bid not in in_plan:
            queue.append(dict({"id": bid, "plan": "not in the plan"}, **run_summary(runs[bid], t_now)))
    order = sorted(questions.values(), key=lambda q: (bool(q["answers"]), not q["critical"]))
    qs = [{"id": q["id"], "text": q["text"], "critical": q["critical"], "asked": q["asked"].isoformat(),
           "answer": q["answers"][-1]["text"] if q["answers"] else None,
           "answered": q["answers"][-1]["time"].isoformat() if q["answers"] else None,
           "via": q["answers"][-1]["via"] if q["answers"] else None} for q in order]
    return {"tool": "status_page", "plan": os.path.basename(plan), "now": t_now.isoformat(),
            "blocks": [{"id": b, "state": s} for b, s, _ in blocks], "queue": queue,
            "hidden": sum(1 for b, s, _ in blocks if b not in runs and s not in QUEUED),
            "questions": qs, "critical_open": [q["id"] for q in open_critical(questions)],
            "attention": [{"text": a["text"], "time": a["time"].isoformat()} for a in reversed(attention)],
            "problems": problems}


CSS = """
:root { --bg:#fff; --fg:#1b1b1b; --mute:#666; --line:#ddd; --card:#f6f6f6; --crit:#b3261e; --critbg:#fdecea;
  --run:#0b57d0; --ok:#1e7d32; --bad:#b3261e; }
@media (prefers-color-scheme: dark) { :root { --bg:#141414; --fg:#eee; --mute:#aaa; --line:#333; --card:#1f1f1f;
  --crit:#ff8a80; --critbg:#3a1512; --run:#8ab4f8; --ok:#81c995; --bad:#ff8a80; } }
* { box-sizing:border-box; }
body { margin:0 auto; padding:12px 16px 48px; max-width:760px; font:16px/1.45 system-ui,-apple-system,sans-serif;
  background:var(--bg); color:var(--fg); }
h1 { font-size:1.25rem; margin:4px 0; } h2 { font-size:1.05rem; margin:22px 0 8px; }
.mute { color:var(--mute); font-size:.9rem; } .warn { border:1px solid var(--bad); color:var(--bad); padding:8px;
  border-radius:8px; }
.q { background:var(--card); border-radius:10px; padding:10px 12px; margin:10px 0; border-left:4px solid var(--line); }
.q.crit.open { border-left-color:var(--crit); background:var(--critbg); }
.tag { font-weight:700; color:var(--crit); font-size:.85rem; }
.qt { white-space:pre-wrap; margin:6px 0; } .ans { white-space:pre-wrap; border-left:3px solid var(--ok);
  padding-left:8px; margin:6px 0; }
textarea { width:100%; min-height:4.5em; font:inherit; padding:8px; border-radius:8px; border:1px solid var(--line);
  background:var(--bg); color:var(--fg); }
button { margin-top:6px; font:inherit; padding:8px 18px; border-radius:8px; border:0; background:var(--run);
  color:#fff; }
ul { padding-left:18px; } li { margin:4px 0; white-space:pre-wrap; }
.row { display:flex; flex-wrap:wrap; gap:4px 10px; align-items:baseline; padding:8px 0;
  border-bottom:1px solid var(--line); }
.row .id { font-weight:700; } .row .time { margin-left:auto; font-variant-numeric:tabular-nums; }
.row .sub { flex-basis:100%; color:var(--mute); font-size:.9rem; }
.badge { font-size:.8rem; padding:1px 8px; border-radius:999px; border:1px solid var(--line); }
.running { color:var(--run); border-color:var(--run); } .done { color:var(--ok); border-color:var(--ok); }
.failed, .blocked { color:var(--bad); border-color:var(--bad); }
"""

JS = """
setInterval(function () {
  var busy = Array.prototype.some.call(document.querySelectorAll('textarea'), function (t) {
    return t.value.trim() !== '' || t === document.activeElement; });
  if (!busy) { location.reload(); }
}, %d);
""" % (REFRESH_S * 1000)


def page_html(v):
    e = html.escape
    t_now = parse_time(v["now"])
    today = t_now.date()
    at = lambda s: clock_text(parse_time(s), today)  # noqa: E731
    running = sum(1 for r in v["queue"] if r["run"] == "running")
    out = ["<!doctype html><html lang='en'><head><meta charset='utf-8'>",
           "<meta name='viewport' content='width=device-width, initial-scale=1'>",
           f"<title>Status · {e(v['plan'])}</title><style>{CSS}</style></head><body>",
           f"<h1>Status · {e(v['plan'])}</h1>",
           f"<p class='mute'>updated {e(t_now.strftime('%H:%M:%S'))} · queue {len(v['queue'])} · running {running} · "
           f"critical open {len(v['critical_open'])} · <a href='/'>refresh</a></p>"]
    for p in v["problems"]:
        out.append(f"<p class='warn'>{e(p)}</p>")
    out.append("<h2>Questions</h2>")
    if not v["questions"]:
        out.append("<p class='mute'>none asked</p>")
    for q in v["questions"]:
        state = "open" if q["answer"] is None else "answered"
        out.append(f"<section class='q{' crit' if q['critical'] else ''} {state}' id='{e(q['id'])}'><div><b>{e(q['id'])}</b> "
                   + ("<span class='tag'>CRITICAL — the block waits for this answer</span> " if q["critical"] else "")
                   + f"<span class='mute'>asked {e(at(q['asked']))}</span></div><p class='qt'>{e(q['text'])}</p>")
        form = (f"<form method='post' action='/answer'><input type='hidden' name='id' value='{e(q['id'])}'>"
                "<textarea name='text' placeholder='Your answer'></textarea><button type='submit'>Send</button></form>")
        if q["answer"] is None:
            out.append(form)
        else:
            out.append(f"<p class='ans'>{e(q['answer'])}</p><p class='mute'>answered {e(at(q['answered']))}"
                       f"{' on the page' if q['via'] == 'page' else ''}</p><details><summary>Change the answer</summary>"
                       f"{form}</details>")
        out.append("</section>")
    out.append("<h2>Attention</h2>")
    out.append("<ul>" + "".join(f"<li><span class='mute'>{e(at(a['time']))}</span> {e(a['text'])}</li>"
                                for a in v["attention"]) + "</ul>" if v["attention"] else "<p class='mute'>nothing yet</p>")
    out.append("<h2>Queue</h2>")
    for r in v["queue"]:
        sub = " · ".join(x for x in (r["plan"], r["note"]) if x)
        out.append(f"<div class='row'><span class='id'>{e(r['id'])}</span>"
                   + (f"<span class='badge {e(r['run'])}'>{e(r['run'])}</span>" if r["run"] else "")
                   + f"<span class='time'>{e(span(r['elapsed_s']))}</span><div class='sub'>{e(sub)}</div></div>")
    if v["hidden"]:
        out.append(f"<p class='mute'>{v['hidden']} other block(s) not shown: DONE, N/A or DEFERRED, and not marked by "
                   "this run</p>")
    out.append(f"<script>{JS}</script></body></html>")
    return "\n".join(out)


# ---------------------------------------------------------------- serve
class Server(http.server.ThreadingHTTPServer):
    allow_reuse_address = os.name != "nt"  # on Windows SO_REUSEADDR would let a second server share a held port
    daemon_threads = True

    def handle_error(self, request, client_address):  # one line, not a traceback (a phone dropping a connection)
        print(f"WARN request from {client_address[0]} failed: {sys.exc_info()[1]!r}", file=sys.stderr, flush=True)


class Server6(Server):
    address_family = socket.AF_INET6


def handler(st):
    class Handler(http.server.BaseHTTPRequestHandler):
        server_version = "status_page"

        def log_message(self, *args):  # the counts line at stop is the record
            pass

        def reply(self, code, body, ctype="application/json; charset=utf-8", headers=()):
            data = body if isinstance(body, bytes) else json.dumps(body, ensure_ascii=False).encode("utf-8")
            self.send_response(code)
            for k, val in (("Content-Type", ctype), ("Content-Length", str(len(data))), ("Cache-Control", "no-store")) + headers:
                self.send_header(k, val)
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            path = urllib.parse.urlsplit(self.path).path
            if path == "/health":
                return self.reply(200, {"status": "ready", "tool": "status_page"})
            if path not in ("/", "/api/state"):
                return self.reply(404, {"ok": False, "error": "GET /, /api/state or /health"})
            v = view(st["plan"], st["state"])
            if path == "/api/state":
                return self.reply(200, v)
            with st["lock"]:
                st["views"] += 1
            return self.reply(200, page_html(v).encode("utf-8"), "text/html; charset=utf-8")

        def do_POST(self):
            if urllib.parse.urlsplit(self.path).path != "/answer":
                return self.reply(404, {"ok": False, "error": "POST /answer only"})
            if self.headers.get("Origin") not in (None, "http://" + str(self.headers.get("Host"))):
                return self.reply(403, {"ok": False, "error": "cross-origin POST refused"})
            try:
                n = int(self.headers.get("Content-Length") or 0)
            except ValueError:
                n = -1
            if n < 0 or n > MAX_BODY:
                self.close_connection = True
                return self.reply(413, {"ok": False, "error": f"a body of 0 to {MAX_BODY} bytes"})
            is_json = (self.headers.get("Content-Type") or "").split(";")[0].strip().lower() == "application/json"
            qid = text = None
            try:
                raw = self.rfile.read(n).decode("utf-8")
                if is_json:
                    body = json.loads(raw)
                    qid, text = (body.get("id"), body.get("text")) if isinstance(body, dict) else (None, None)
                else:
                    form = urllib.parse.parse_qs(raw, keep_blank_values=True)
                    qid, text = form.get("id", [None])[0], form.get("text", [None])[0]
            except (ValueError, UnicodeDecodeError):
                pass
            if not isinstance(qid, str) or not isinstance(text, str) or not text.strip():
                return self.reply(400, {"ok": False, "error": "an answer is {id: q<n>, text: non-empty} — JSON or a form"})
            try:
                answer_question(st["state"], qid, text.replace("\r\n", "\n"), "page")
            except UnknownQuestion as ex:
                with st["lock"]:
                    st["rejected"] += 1
                return self.reply(422, {"ok": False, "error": str(ex)})
            except (StateError, OSError) as ex:  # the page shows the error; the stop verdict counts it
                with st["lock"]:
                    st["errors"] += 1
                return self.reply(500, {"ok": False, "error": f"{type(ex).__name__}: {ex}"})
            with st["lock"]:
                st["answers"] += 1
            if is_json:
                return self.reply(200, {"ok": True, "id": qid})
            return self.reply(303, b"", "text/plain", (("Location", "/#" + urllib.parse.quote(qid)),))
    return Handler


def resolve_host(host):
    """→ (address, None) or (None, why). `tailscale` runs TAILSCALE now; its answer lives in memory only."""
    if host != "tailscale":
        return host, None
    try:
        r = subprocess.run(TAILSCALE, capture_output=True, text=True, timeout=15)
    except (OSError, subprocess.SubprocessError) as e:
        return None, f"`{' '.join(TAILSCALE)}` did not run ({e}) — not serving; no fallback to another address"
    first = next((ln.strip() for ln in r.stdout.splitlines() if ln.strip()), "")
    try:
        ipaddress.IPv4Address(first)
    except ValueError:
        return None, (f"`{' '.join(TAILSCALE)}` gave {first!r} (exit {r.returncode}), not an IPv4 address — not serving; "
                      "no fallback to another address")
    return first, None


def product_ports():
    """{port: service} from launch.json beside this tool, and a warning when it cannot be read."""
    if not os.path.exists(LAUNCH_JSON):
        return {}, None
    try:
        with open(LAUNCH_JSON, encoding="utf-8-sig") as f:
            services = json.load(f).get("services") or {}
        return {s["port"]: name for name, s in services.items() if isinstance(s, dict) and isinstance(s.get("port"), int)}, None
    except (OSError, ValueError, AttributeError) as e:
        return {}, f"{LAUNCH_JSON}: unreadable ({e}) — the product's ports not checked"


def url_of(addr, port):
    return f"http://{'[%s]' % addr if ':' in addr else addr}:{port}/"


def serve_start(plan, state, host, port):
    """The checks, then bind (a held port is refused, never shared) and a server thread → (httpd, st, failures, warns)."""
    failures, warnings = [], []
    ports, warn = product_ports()
    if warn:
        warnings.append(warn)
    if port in ports:
        failures.append(f"port {port} is service {ports[port]!r}'s in {LAUNCH_JSON} — the page runs on its own port")
    addr, why = resolve_host(host)
    if why:
        failures.append(why)
    if failures:
        return None, None, failures, warnings
    ensure_state(state)
    st = {"plan": plan, "state": state, "lock": threading.Lock(), "answers": 0, "rejected": 0, "errors": 0, "views": 0}
    try:
        httpd = (Server6 if ":" in addr else Server)((addr, port), handler(st))
    except OSError as e:
        failures.append(f"{addr}:{port}: {e.strerror or e} — a held port is not reused")
        return None, None, failures, warnings
    st["url"] = url_of(addr, httpd.server_address[1])
    threading.Thread(target=httpd.serve_forever, kwargs={"poll_interval": 0.05}, daemon=True).start()
    return httpd, st, failures, warnings


# ---------------------------------------------------------------- the calls
def verdict_out(counts, reason, as_json, lines=(), failures=(), warnings=(), data=None, code=1):
    ok = reason is None
    if as_json:
        print(json.dumps({"counts": dict(counts), "failures": list(failures), "warnings": list(warnings),
                          "data": data or {}, "verdict": "GO" if ok else "NO-GO", "reason": reason},
                         indent=1, ensure_ascii=False))
    else:
        for ln in lines:
            print(ln)
        print(" · ".join(f"{k} {v}" for k, v in counts))
        for tag, items in (("FAIL", failures), ("WARN", warnings)):
            for x in items:
                print(f"{tag} {x}")
        print("=== GO ===" if ok else f"=== NO-GO: {reason} ===")
    sys.stdout.flush()
    return 0 if ok else code


def refuse(name, why, as_json):
    return verdict_out([(name, "refused"), ("written", 0)], why.split(" — ")[0], as_json, failures=[why])


def cmd_mark(a):
    try:
        path = plan_path(a.plan)
        blocks = {b[0]: b for b in blocks_of(path)}
    except (StateError, OSError, UnicodeDecodeError) as e:
        return refuse("mark", str(e), a.json)
    if a.id not in blocks:
        near = [b for b in blocks if pb_plan.id_tail(b) == a.id or b.lower() == a.id.lower()]
        return refuse("mark", f"no block {a.id} in {path}" + (f" — ids are written whole: {' · '.join(near)}" if near else ""),
                      a.json)
    ev = {"time": now().isoformat(), "id": a.id, "run": a.run}
    if a.note:
        ev["note"] = a.note
    try:
        append(a.state, "runs", ev)
        runs, _, _, problems = load(a.state)
    except (StateError, OSError) as e:
        return refuse("mark", str(e), a.json)
    s = run_summary(runs[a.id], now())
    _, st, text = blocks[a.id]
    return verdict_out([("marked", f"{a.id} {a.run}"), ("plan status", text or st), ("running time", span(s["elapsed_s"])),
                        ("blocks marked", len(runs))], None, a.json, warnings=problems, data=dict(s, id=a.id))


def cmd_ask(a):
    text = a.question.strip()
    if not text:
        return refuse("ask", "an empty question", a.json)
    try:
        with locked(a.state, "questions"):
            _, questions, _, problems = load(a.state)
            if problems:
                raise StateError(f"{len(problems)} state problem(s), nothing written: " + " | ".join(problems))
            qid = a.id or f"q{max((int(q[1:]) for q in questions), default=0) + 1}"
            if not QID.match(qid):
                raise StateError(f"--id {qid!r} — a question id is q<n>: q1, q2, …")
            if qid in questions:
                raise StateError(f"{qid} is already asked ({questions[qid]['text']!r}) — a new question takes a new id")
            write_event(a.state, "questions", {"time": now().isoformat(), "event": "ask", "id": qid, "text": text,
                                               "critical": bool(a.critical)})
    except (StateError, OSError) as e:
        return refuse("ask", str(e), a.json)
    crit = open_critical(questions) + ([{"id": qid}] if a.critical else [])
    opened = sum(1 for q in questions.values() if not q["answers"]) + 1
    return verdict_out([("asked", qid), ("critical", "yes" if a.critical else "no"), ("open", opened),
                        ("critical open", len(crit))], None, a.json,
                       [f"asked {qid}" + (" — critical: the block waits for its answer (poll exits 3)" if a.critical else "")],
                       data={"id": qid, "critical": bool(a.critical)})


def cmd_answer(a):
    if not a.text.strip():
        return refuse("answer", "an empty answer", a.json)
    try:
        prev = answer_question(a.state, a.qid, a.text, "cli")
        _, questions, _, problems = load(a.state)
    except (StateError, OSError) as e:
        return refuse("answer", str(e), a.json)
    warns = [f"{a.qid} was answered at {prev['time'].isoformat()} ({prev['text']!r}) — this answer supersedes it"] if prev else []
    return verdict_out([("answered", a.qid), ("open", sum(1 for q in questions.values() if not q["answers"])),
                        ("critical open", len(open_critical(questions)))], None, a.json,
                       warnings=warns + problems, data={"id": a.qid})


def cmd_attention(a):
    if not a.text.strip():
        return refuse("attention", "an empty item", a.json)
    try:
        append(a.state, "attention", {"time": now().isoformat(), "text": a.text})
        _, _, attention, problems = load(a.state)
    except (StateError, OSError) as e:
        return refuse("attention", str(e), a.json)
    return verdict_out([("attention items", len(attention))], None, a.json, warnings=problems)


def cmd_poll(a):
    deadline = time.monotonic() + max(0.0, a.wait or 0.0)
    while True:
        _, questions, _, problems = load(a.state)
        crit = open_critical(questions)
        left = deadline - time.monotonic()
        if problems or not crit or left <= 0:
            break
        time.sleep(min(POLL_STEP_S, left))
    answered = [{"id": q["id"], "question": q["text"], "critical": q["critical"], "answer": q["answers"][-1]["text"],
                 "answered": q["answers"][-1]["time"].isoformat(), "via": q["answers"][-1]["via"]}
                for q in questions.values() if q["answers"]]
    counts = [("answered", len(answered)), ("open", sum(1 for q in questions.values() if not q["answers"])),
              ("critical open", len(crit))]
    if problems:
        return verdict_out(counts, f"{len(problems)} state problem(s) — halt and fix the state", a.json,
                           failures=problems, data={"answered": answered})
    reason = (f"critical question(s) unanswered: {' · '.join(q['id'] for q in crit)} — halt the block(s) they concern"
              if crit else None)
    return verdict_out(counts, reason, a.json, [json.dumps(x, ensure_ascii=False) for x in answered],
                       data={"answered": answered, "critical_open": [q["id"] for q in crit]}, code=3)


def cmd_serve(a):
    try:
        path = plan_path(a.plan)
    except StateError as e:
        return refuse("serve", str(e), a.json)
    httpd, st, failures, warnings = serve_start(path, a.state, a.host, a.port)
    if httpd is None:
        return verdict_out([("serving", "no"), ("answers received", 0)], f"{len(failures)} problem(s) — not serving",
                           a.json, failures=failures, warnings=warnings)
    err = sys.stderr if a.json else sys.stdout
    print(f"serving {st['url']} · plan {path} · state {a.state} · stop with Ctrl-C or SIGTERM", file=err, flush=True)
    for w in warnings:
        print(f"WARN {w}", file=err, flush=True)
    stop = threading.Event()
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, lambda *_: stop.set())
    while not stop.wait(0.5):
        pass
    httpd.shutdown()
    httpd.server_close()
    reason = f"{st['errors']} answer(s) failed to write" if st["errors"] else None
    return verdict_out([("answers received", st["answers"]), ("rejected", st["rejected"]), ("write errors", st["errors"]),
                        ("page views", st["views"])], reason, a.json)


# ---------------------------------------------------------------- selftest
FIXTURE = """# M7 - Fixture plan for the status page selftest

## Flow
| Task | Type | Goal | Order | Status |
|---|---|---|---|---|
| M7-T1 | BUILD | the first step | FIRST | DONE (2000-01-01) |
| M7-T2 | BUILD | the second step | AFTER T1 | IN PROGRESS |
| M7-T3 | BUILD | the third step | AFTER T2 | TODO |
| M7-D1 | BUILD | a defect filed mid-run | AFTER T3 | BLOCKED (waits on q1) |
| M7-V1 | CHECK | the validation | AFTER D1 | TODO |
| M7-T4 | BUILD | parked | LAST | DEFERRED (wake: a later milestone) |

## Pipeline state
- Next task: M7-T2

# Tasks

## M7-T1 · the first step · **BUILD**
- Status: DONE (2000-01-01)
- Goal: one line.

## M7-T2 · the second step · **BUILD**
- Status: IN PROGRESS
- Notes: an example the block quotes, a heading inside a fence:
```markdown
## M7-T90 · a block quoted inside a backtick fence is text
- Status: TODO
```
````markdown
A block template quoted with its own fence inside:
```text
## M7-T94 · a block inside a fence inside a longer fence is text
- Status: DONE (2000-01-02)
```
````

## M7-T3 · the third step · **BUILD**
- Status: TODO
- Notes:
  1. a step whose fence opens after a list marker
     ```bash
     ## M7-T93 · indented, inside a list item's fence
     ```
~~~text
## M7-T91 · a block quoted inside a tilde fence is text
- Status: IN PROGRESS
~~~

### M7-T92 · a third-level heading is no block

## M7-D1 · a defect filed mid-run · **BUILD**
- Status: BLOCKED (waits on q1)

## M7-V1 · the validation · **CHECK**

- Status: TODO

## M7-T4 · parked · **BUILD**
- Status: DEFERRED (wake: a later milestone)
"""
EXPECTED = [("M7-T1", "DONE"), ("M7-T2", "IN PROGRESS"), ("M7-T3", "TODO"), ("M7-D1", "BLOCKED"), ("M7-V1", "TODO"),
            ("M7-T4", "DEFERRED")]
T0 = datetime.datetime(2000, 1, 1, 12, 0, 0, tzinfo=datetime.timezone.utc)

C_PARSER = ("the served block list (GET /api/state) is the fixture's six blocks in file order with their states — "
            "plan.py's own reading of the same file (its lint GO: blocks 6, Flow rows 6); the fenced and third-level "
            "headings are no blocks")
C_CRITICAL = "an unanswered critical question makes poll NO-GO with exit 3, naming it (the non-critical one not named)"
C_APPEND = "every state file is append-only: its bytes before the page's answers are a prefix of its bytes after"
C_UNKNOWN = "a POST /answer for an unknown question id is rejected (HTTP 422), questions.jsonl byte-identical"
C_CONCURRENT = "four concurrent appends, the read → write window forced wide, all land: plan.py's lock held around each"
RED_ARMS = {"own_parser": C_PARSER, "critical_ignored": C_CRITICAL, "rewrite_state": C_APPEND,
            "accept_unknown": C_UNKNOWN, "no_lock": C_CONCURRENT}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


OPENER = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())  # 127.0.0.1 only, no proxy


def fetch(method, url, body=None, headers=None):
    """→ (status, bytes, headers); 0 when nothing answers."""
    req = urllib.request.Request(url, data=body, method=method, headers=headers or {})
    try:
        with OPENER.open(req, timeout=10) as r:
            return r.status, r.read(), r.headers
    except urllib.error.HTTPError as e:
        return e.code, e.read(), e.headers
    except (urllib.error.URLError, OSError):
        return 0, b"", {}


def call(argv):
    """main(argv) in this process → (exit code, stdout)."""
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        code = main(argv)
    return code, buf.getvalue()


def lint_reading(path):
    """plan.py run as its own program on the fixture: its lint verdict and block and Flow-row counts."""
    r = subprocess.run([sys.executable, os.path.join(HERE, "plan.py"), "lint", "--json", "--plan", path],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    try:
        doc = json.loads(r.stdout.rsplit("\n===", 1)[0])
        return r.returncode == 0 and doc["failed"] == 0, doc["data"]["blocks"], doc["data"]["flow_rows"]
    except (ValueError, KeyError, TypeError):
        return False, None, None


def suite(check, bugs, lint, full=True):
    """One pass over a throwaway fixture with `bugs` switched on in memory. Deletes only its own temp dir. → evidence."""
    global CLOCK, TAILSCALE, LAUNCH_JSON, POLL_STEP_S, APPEND_DELAY_S
    saved = (TAILSCALE, LAUNCH_JSON, POLL_STEP_S, APPEND_DELAY_S)
    INJECT.clear()
    INJECT.update(bugs)
    tmp, httpd, ev = tempfile.mkdtemp(prefix="pb_status_selftest_"), None, {}
    try:
        plan = os.path.join(tmp, "plan.md")
        for d in pb_plan.DIRS:
            os.makedirs(os.path.join(tmp, d))
        with open(plan, "w", encoding="utf-8", newline="\n") as f:
            f.write(FIXTURE)
        with open(plan, "rb") as f:
            plan_bytes = f.read()
        state = os.path.join(tmp, "state")
        s = ["--state", state]
        POLL_STEP_S, LAUNCH_JSON = 0.05, os.path.join(tmp, "launch.json")
        marker = os.path.join(tmp, "address-command-ran")
        TAILSCALE = [sys.executable, "-c", f"open({marker!r}, 'w').close(); print('127.0.0.1')"]

        # mark: a run's marks at pinned times; ids written whole; unknown ids refused
        for at, argv in ((-900, ["mark", "M7-T2", "running"]), (-800, ["mark", "M7-T2", "failed", "--note", "a flaky test"]),
                         (-600, ["mark", "M7-T1", "running"]), (-60, ["mark", "M7-T1", "done", "--note", "merged"]),
                         (-125, ["mark", "M7-T2", "running"])):
            CLOCK = T0 + datetime.timedelta(seconds=at)
            code, _ = call(argv + ["--plan", plan] + s)
            check(f"mark {' '.join(argv[1:3])} → GO", code == 0)
        CLOCK = T0
        code, out = call(["mark", "T2", "running", "--plan", plan] + s)
        check("mark T2 (a local id) refused, naming the whole id M7-T2; nothing written", code == 1 and "M7-T2" in out
              and len(load(state)[0]) == 2)
        code, out = call(["mark", "M7-T90", "running", "--plan", plan] + s)
        check("mark M7-T90 (a heading inside a fence) refused: no such block", code == 1 and "no block M7-T90" in out)
        code, out = call(["mark", "M7-T2", "running", "--plan", os.path.join(tmp, "absent.md")] + s)
        check("mark against a plan that does not exist refused", code == 1 and "no plan file" in out)

        # ask / attention; the refusals
        code, out = call(["ask", "Which label for the new <button>?"] + s)
        check("ask → q1", code == 0 and "asked q1" in out)
        code, out = call(["ask", "Push the release branch to the shared remote?", "--critical"] + s)
        check("ask --critical → q2", code == 0 and "asked q2" in out)
        before = open(state_file(state, "questions"), "rb").read()
        refusals = [call(["ask", "again", "--id", "q2"] + s), call(["ask", "odd id", "--id", "x1"] + s), call(["ask", "  "] + s)]
        check("ask refuses a taken id, an id not q<n> and an empty question; questions.jsonl byte-identical",
              all(c == 1 for c, _ in refusals) and open(state_file(state, "questions"), "rb").read() == before)
        code, _ = call(["attention", "Commit the tool folder after the run"] + s)
        check("attention → GO", code == 0)

        # poll before any answer: the §A.6 NO-GO arm
        code, out = call(["poll"] + s)
        last = out.strip().splitlines()[-1] if out.strip() else ""
        check(C_CRITICAL, code == 3 and "q2" in last and "q1" not in last and last.startswith("=== NO-GO"))
        if full:
            r = subprocess.run([sys.executable, os.path.abspath(__file__), "poll"] + s, capture_output=True, timeout=60)
            check("poll as its own process exits 3 while q2 is open", r.returncode == 3)
        snapshot = {k: open(state_file(state, k), "rb").read() for k in FILES}

        # serve: --host tailscale through the swapped command, an ephemeral port
        httpd, st, failures, _ = serve_start(plan, state, "tailscale", 0)
        check("serve --host tailscale runs the address command and binds what it printed (127.0.0.1), on an ephemeral port",
              httpd is not None and not failures and st["url"].startswith("http://127.0.0.1:") and os.path.exists(marker))
        base = st["url"] if httpd else "http://127.0.0.1:9/"
        code, body, _ = fetch("GET", base + "api/state")
        api = json.loads(body.decode("utf-8")) if code == 200 else {}
        served = [(b["id"], b["state"]) for b in api.get("blocks", [])]
        ok, n_blocks, n_rows = lint
        ev["parser"] = (served == EXPECTED, ok, n_blocks, n_rows)
        check(C_PARSER, served == EXPECTED and ok and n_blocks == len(EXPECTED) and n_rows == len(EXPECTED))
        code, body, _ = fetch("GET", base)
        page = body.decode("utf-8", "replace")
        rows = [r["id"] for r in api.get("queue", [])]
        check("GET / shows the queue in file order — the marked M7-T1 (9 min) and M7-T2 (running again, 2 min: its last "
              "span), the TODO / BLOCKED ones; M7-T4 (DEFERRED, unmarked) left out and counted", code == 200
              and rows[:5] == ["M7-T1", "M7-T2", "M7-T3", "M7-D1", "M7-V1"] and "M7-T4" not in rows and api.get("hidden") == 1
              and all(x in page for x in ("M7-T2", "9 min", "2 min", "running", "DONE (2000-01-01) · merged",
                                          "1 other block(s) not shown")))
        check("GET / shows q2 marked CRITICAL above q1 (asked first), each with an answer form, the attention item, the "
              "lead-facing text HTML-escaped",
              page.count("CRITICAL") == 1 and page.find("CRITICAL") > page.find("id='q2'")
              and 0 <= page.find("id='q2'") < page.find("id='q1'") and page.count("action='/answer'") == 2
              and "Commit the tool folder after the run" in page and "new &lt;button&gt;?" in page
              and "new <button>?" not in page)
        check("GET /health → ready; GET /other → 404", fetch("GET", base + "health")[0] == 200
              and fetch("GET", base + "other")[0] == 404)

        # POST /answer: refusals first, then the lead's answer from the page
        form = lambda qid, text: urllib.parse.urlencode({"id": qid, "text": text}).encode("utf-8")  # noqa: E731
        fh = {"Content-Type": "application/x-www-form-urlencoded"}
        before = open(state_file(state, "questions"), "rb").read()
        code, _, _ = fetch("POST", base + "answer", form("q9", "yes"), fh)
        check(C_UNKNOWN, code == 422 and open(state_file(state, "questions"), "rb").read() == before)
        codes = [fetch("POST", base + "answer", form("q2", "yes"), dict(fh, Origin="http://elsewhere.invalid"))[0],
                 fetch("POST", base + "answer", form("q2", "   "), fh)[0], fetch("POST", base + "answer", b"{", {
                     "Content-Type": "application/json"})[0],
                 fetch("POST", base + "answer", form("q2", "x" * MAX_BODY), fh)[0]]
        check("POST /answer refuses a foreign Origin (403), an empty answer and malformed JSON (400), a body over 64 KiB "
              "(413); nothing written", codes == [403, 400, 400, 413]
              and open(state_file(state, "questions"), "rb").read() == before)
        code, _, hdrs = fetch("POST", base + "answer", form("q2", "Yes, push it.\r\nThen tag it."), fh)
        check("the page's form answers q2: 303 back to #q2", code == 303 and (hdrs.get("Location") or "") == "/#q2")
        code, out = call(["poll"] + s)
        got = [json.loads(ln) for ln in out.splitlines() if ln.startswith("{")]
        ev["roundtrip"] = got
        check("mark / ask / answer / poll round trip: poll GO, q1's answer from the page read back verbatim (line break "
              "kept, CRLF normalised)", code == 0 and [g["id"] for g in got] == ["q2"]
              and got[0]["answer"] == "Yes, push it.\nThen tag it." and got[0]["via"] == "page")

        # answer from the CLI; a second answer supersedes; an unknown id refused
        c1, _ = call(["answer", "q1", "Save"] + s)
        c2, out2 = call(["answer", "q1", "Save changes"] + s)
        c3, out3 = call(["answer", "q9", "x"] + s)
        code, out = call(["poll", "--json"] + s)
        doc = json.loads(out) if code == 0 else {}
        ans = {x["id"]: x["answer"] for x in doc.get("data", {}).get("answered", [])}
        check("answer q1 twice from the CLI: GO, the second WARNs it supersedes and wins; answer q9 refused",
              (c1, c2, c3) == (0, 0, 1) and "supersedes" in out2 and "no question 'q9'" in out3
              and ans == {"q1": "Save changes", "q2": "Yes, push it.\nThen tag it."})

        # poll --wait: a critical question answered on the page while the orchestrator waits
        if full:
            call(["ask", "Delete the old build folder?", "--critical"] + s)
            timer = threading.Timer(0.15, fetch, ("POST", base + "answer", json.dumps({"id": "q3", "text": "No"}).encode(),
                                                  {"Content-Type": "application/json"}))
            t_start = time.monotonic()
            timer.start()
            code, out = call(["poll", "--wait", "5"] + s)
            waited = time.monotonic() - t_start
            timer.join()
            check("poll --wait 5 holds while q3 (critical) is open and returns GO once the page answers it (≈0.15 s)",
                  code == 0 and 0.1 <= waited < 3 and '"q3"' in out)
        httpd.shutdown()
        httpd.server_close()
        httpd = None

        after = {k: open(state_file(state, k), "rb").read() for k in FILES}
        check(C_APPEND, all(after[k].startswith(snapshot[k]) for k in FILES) and after["questions"] != snapshot["questions"]
              and after["runs"].count(b"\n") == 5 and after["questions"].count(b"\n") == (7 if full else 5))
        check("the state folder holds a .gitignore of `*` (the run's record is never committed)",
              open(os.path.join(state, ".gitignore"), encoding="utf-8").read() == "*\n")
        texts = b"".join(open(os.path.join(root, f), "rb").read() for root, _, fs in os.walk(tmp) for f in fs)
        check("the served address is written into no file (the state, the plan, the temp dir)",
              base.split("//")[1].rstrip("/").encode() not in texts)
        with open(plan, "rb") as f:
            check("the plan is byte-identical: the page only reads it", f.read() == plan_bytes)

        # concurrency: plan.py's lock around the read-modify-write
        if full or "no_lock" in bugs:
            APPEND_DELAY_S, conc = 0.04, os.path.join(tmp, "conc")
            ensure_state(conc)
            gate, errs = threading.Barrier(4), []

            def writer(i):
                gate.wait()
                try:
                    append(conc, "attention", {"time": T0.isoformat(), "text": f"writer {i}"})
                except Exception as e:  # a crash is a lost line, never a hang
                    errs.append(e)
            threads = [threading.Thread(target=writer, args=(i,)) for i in range(4)]
            for t in threads:
                t.start()
            for t in threads:
                t.join(30)
            APPEND_DELAY_S = 0.0
            items = [a["text"] for a in load(conc)[2]]
            check(C_CONCURRENT, not errs and sorted(items) == [f"writer {i}" for i in range(4)])

        if full:
            # the refusals of serve: no fallback address, the product's port, a held port
            for cmd, what in (([sys.executable, "-c", "import sys; sys.exit(1)"], "exits 1, prints nothing"),
                              ([sys.executable, "-c", "print('not-an-address')"], "prints no IPv4 address"),
                              ([os.path.join(tmp, "no-such-command")], "does not exist")):
                TAILSCALE = cmd
                h, _, fails, _ = serve_start(plan, state, "tailscale", 0)
                check(f"serve --host tailscale refused when the command {what}: no fallback address, nothing bound",
                      h is None and len(fails) == 1 and "no fallback" in fails[0])
            with socket.socket() as held:
                held.bind(("127.0.0.1", 0))
                held.listen(1)
                port = held.getsockname()[1]
                with open(LAUNCH_JSON, "w", encoding="utf-8") as f:
                    json.dump({"services": {"web": {"cmd": ["x"], "port": port + 1}}}, f)
                h1, _, f1, _ = serve_start(plan, state, "127.0.0.1", port + 1)
                h2, _, f2, _ = serve_start(plan, state, "127.0.0.1", port)
            check("serve refuses the port launch.json gives a service, naming it, and a held port (never shared)",
                  h1 is None and "'web'" in " ".join(f1) and h2 is None and "held port" in " ".join(f2))

            # a state file broken by hand: poll NO-GO exit 1, naming file:line; the file left as it is
            with open(state_file(state, "questions"), "a", encoding="utf-8") as f:
                f.write("{broken\n")
            broken = open(state_file(state, "questions"), "rb").read()
            code, out = call(["poll"] + s)
            c2, out2 = call(["ask", "one more?"] + s)
            check("a malformed state line: poll NO-GO exit 1 naming questions.jsonl:8; ask refuses to write onto it; the "
                  "file unchanged", code == 1 and "questions.jsonl:8" in out and c2 == 1
                  and open(state_file(state, "questions"), "rb").read() == broken)
            hand = os.path.join(tmp, "hand")
            ensure_state(hand)
            with open(state_file(hand, "attention"), "w", encoding="utf-8", newline="") as f:
                f.write('\ufeff{"time": "2000-01-01T12:00:00+00:00", "text": "one"}\r\n'
                        '{"time": "2000-01-01T12:00:00+00:00", "text": "two, no final newline"}')
            code, _ = call(["attention", "three", "--state", hand])
            raw = open(state_file(hand, "attention"), "rb").read()
            _, _, items, probs = load(hand)
            check("an append onto a hand-edited file (a byte-order mark, CRLF, no final newline): the line closed first, "
                  "CRLF kept, the three items read", code == 0 and not probs and raw.count(b"\r\n") == 3
                  and raw.count(b"\n") == 3 and [x["text"] for x in items] == ["one", "two, no final newline", "three"])
            check("plan.py is imported from beside this file (the one parser, §A.0)",
                  os.path.samefile(pb_plan.__file__, os.path.join(HERE, "plan.py")))
    except Exception as e:  # a crash is a failed check, never a missing verdict
        check(f"suite raised {type(e).__name__}: {e}", False)
    finally:
        if httpd is not None:
            httpd.shutdown()
            httpd.server_close()
        INJECT.clear()
        CLOCK = None
        TAILSCALE, LAUNCH_JSON, POLL_STEP_S, APPEND_DELAY_S = saved
        shutil.rmtree(tmp, ignore_errors=True)
    check("fixture removed (only the selftest's own temp dir)", not os.path.exists(tmp))
    return ev


def cmd_selftest(a):
    """§A.6: mark / ask / answer / poll round-trip on a temp state dir, serve on an ephemeral port, GET / holds the
    marked id → GO; an unanswered critical question makes poll exit 3 → the NO-GO arm. Red arms: each planted bug,
    switched on in memory, must turn its own check red (`--inject <bug>` runs the whole suite with it: NO-GO)."""
    checks = []
    check = lambda name, ok: checks.append((name, bool(ok)))  # noqa: E731
    with tempfile.TemporaryDirectory(prefix="pb_status_lint_") as d:
        path = os.path.join(d, "plan.md")
        for sub in pb_plan.DIRS:
            os.makedirs(os.path.join(d, sub))
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(FIXTURE)
        lint = lint_reading(path)
    ev = suite(check, set(a.inject or ()), lint)
    arms = []
    for bug, target in ({} if a.inject else RED_ARMS).items():
        sub = []
        suite(lambda n, ok: sub.append((n, bool(ok))), {bug}, lint, full=False)
        arms.append(f"{bug} → {'red' if (target, False) in sub else 'STAYED GREEN'}")
        check(f"INJECT {bug} (in memory) turns red: {target}", (target, False) in sub)
    failed = [n for n, ok in checks if not ok]
    same, lint_ok, n_blocks, n_rows = ev.get("parser", (False, False, None, None))
    lines = [f"INJECT {', '.join(a.inject)} (in memory)"] if a.inject else [
        f"  block lists: the page's {'==' if same else '!='} the fixture's {len(EXPECTED)} · plan.py lint "
        f"{'GO' if lint_ok else 'NO-GO'}, blocks {n_blocks}, Flow rows {n_rows}",
        f"  round trip: {len(ev.get('roundtrip', []))} answered question(s) read back by poll after a served POST",
        "  red arms: " + " · ".join(arms)]
    reason = (f"{len(failed)} check(s) failed" + (f" under INJECT {', '.join(a.inject)}" if a.inject else "")) if failed else None
    return verdict_out([("checks", len(checks)), ("passed", len(checks) - len(failed)), ("failed", len(failed))],
                       reason, a.json, lines, failed)


# ---------------------------------------------------------------- main
def main(argv=None):
    ap = argparse.ArgumentParser(prog="status_page.py", description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="command", required=True)

    def add(name):
        sp = sub.add_parser(name)
        sp.add_argument("--state", default=DEFAULT_STATE)
        sp.add_argument("--json", action="store_true")
        return sp
    sp = add("serve")
    sp.add_argument("--plan")
    sp.add_argument("--host", default="127.0.0.1")
    sp.add_argument("--port", type=int, default=DEFAULT_PORT)
    sp = add("mark")
    sp.add_argument("id")
    sp.add_argument("run", choices=RUN_STATES)
    sp.add_argument("--note")
    sp.add_argument("--plan")
    sp = add("ask")
    sp.add_argument("question")
    sp.add_argument("--critical", action="store_true")
    sp.add_argument("--id")
    sp = add("answer")
    sp.add_argument("qid")
    sp.add_argument("text")
    add("poll").add_argument("--wait", type=float, default=0.0)
    add("attention").add_argument("text")
    sp = sub.add_parser("selftest")
    sp.add_argument("--inject", action="append", choices=INJECTS)
    sp.add_argument("--json", action="store_true")
    a = ap.parse_args(argv)
    return {"serve": cmd_serve, "mark": cmd_mark, "ask": cmd_ask, "answer": cmd_answer, "poll": cmd_poll,
            "attention": cmd_attention, "selftest": cmd_selftest}[a.command](a)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):  # a piped Windows console is cp1252; a question can carry `—`, `é`, `→`
        with contextlib.suppress(AttributeError, ValueError, OSError):
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    sys.exit(main())
