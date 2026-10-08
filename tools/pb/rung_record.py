#!/usr/bin/env python3
"""rung_record.py — each DONE block's rung, claim run and cost, by class and rung (PLAYBOOK annex §A.9).

report · now · selftest.
Python 3.9+, standard library only (plan.py beside it for ids, types and block headings), read-only: it writes
nothing and runs no git. `report [--plan <plan>] [--since <YYYY-MM-DD>] [--sessions <dir>…] [--blocks] [--json]`
reads, per DONE block of the plan and its archives: its type (§0's id table, `Kind kept:` read) and `E`; its
sessions in Claude Code's own files (a launch prompt "do task <id>" with the plan opened, or "execute M<N>-<id>"
— else the requests inside its Status's start–end stamps); its claim run in <logs_dir>/loop_times.jsonl (the first
`changed` or `all` record, else the first run of any mode, flagged); its rung, the model and level most of the
main-loop requests ran on in the session holding that claim run (its first session when no claim record or no session
holds the claim's time, flagged; `mixed` when another rung ran >=10% there), a later session on another rung marking
the block `escalated` — its cost and the claim's verdict stay with the first rung's class; its cost, each session's last
`cost-state` `totalCostUSD`, a session naming several blocks split by each block's requests' tokens at that session's
own per-model rates, a session with no cost line priced from its own tokens at the rate the folder's other sessions
paid for that model, else at list prices (`priced from tokens`); and the D blocks whose `Caused by:` line names it.
Then per class (type and E) and rung: blocks, first-try rate, cost per DONE block, and against `(usual)` the verdict
§0's rule (4) reads. A block with no session file readable keeps the rung of its handoff and a null cost, named on the
counts line, apart from the sessions with no cost line. Ends in a counts line and `=== GO ===` (a record was read) or
`=== NO-GO: <reason> ===`; exit 0 / 1. Usage: docs/agent/rung_record.md.
`now` prints one line, `model=<id> level=<level> plugin=<installed|missing|unknown>`, the running request's model
and level read first, then whether the switch plugin is installed for this folder; when missing, a second line,
`install: <command>`, this machine's install line (PowerShell on Windows), from `$CLAUDE_CODE_EXECPATH`.
"""
import argparse
import collections
import contextlib
import datetime
import glob
import io
import json
import ntpath
import os
import posixpath
import re
import shutil
import sys
import tempfile

import plan as P

MODEL_RE = re.compile(r"^claude-([a-z]+)-(\d+)-(\d+)(?:-\d{8})?$")
UNITS = (1.0, 0.1, 1.25, 5.0)          # input, cache read, cache write, output: the price ratios; the cost sets the scale
DO_RE = re.compile(r"\bdo task ([A-Za-z][A-Za-z0-9.-]*[A-Za-z0-9])")
EXEC_RE = re.compile(r"\bexecute (M[\d.]+[a-z]?-[A-Za-z][A-Za-z0-9.-]*[A-Za-z0-9])")
OPENED_RE = re.compile(r"milestones/(m[\d.]+[a-z]?)/")
CAUSE_RE = re.compile(r"\bM[\d.]+[a-z]?-[A-Za-z][A-Za-z0-9.-]*[A-Za-z0-9]")
DATE_RE = re.compile(r"\d{4}-\d{2}-\d{2}")
DONE_RE = re.compile(r"DONE\b[^0-9]*(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2})(?:, started (\d{2}:\d{2}))?")
RUNG_RE = re.compile(r"\b(Opus|Sonnet|Haiku|Fable) (\d+(?:\.\d+)?)\b[^\n]{0,40}?\b(low|medium|high|xhigh|max)\b")
LADDER_RE = re.compile(r"^[-*\s]*\**Models:\**\s*(.*?)\s+—\s+strongest first")
MARK_RE = re.compile(r"^(.+?, \w+)(?: \((gate|usual|gate, usual)\))?$")
CLAIM_SLACK = datetime.timedelta(minutes=2)    # a claim run is recorded when it ends: just after the session's last request
BASIS = ("cost-state", "priced from tokens", "priced from list prices")   # best to worst
# First-party list prices, USD per million input tokens (cached 2026-09-25 from the claude-api skill); UNITS gives the rest
# (output x5, cache read x0.1, cache write x1.25 — Opus 5.5's and Fable's cache reads list cheaper, so a list-priced figure reads high there).
LIST_INPUT_PER_MTOK = {"claude-fable-5-1": 10.0, "claude-fable-5": 10.0, "claude-opus-5-5": 4.0, "claude-opus-5": 5.0,
                       "claude-opus-4-8": 5.0, "claude-opus-4-7": 5.0, "claude-opus-4-6": 5.0, "claude-sonnet-5-5": 2.0,
                       "claude-sonnet-5": 2.0, "claude-sonnet-4-6": 3.0, "claude-haiku-4-5": 1.0}
MIN_CLASS = 10                         # §0 rule (4): a class moves on >=10 blocks
MIXED = 0.10
PLUGIN, MARKET = "pb-switch", "pb"      # switch.mjs's PLUGIN and MARKET; `switch.mjs build` writes <client folder>/pb-switch-src


# ---------------------------------------------------------------- the session files
def rung_name(model, effort):
    """`claude-sonnet-5-5` + `high` -> `Sonnet 5.5, high`; a dated id loses its date."""
    m = MODEL_RE.match(model or "")
    return "%s, %s" % ("%s %s.%s" % (m.group(1).title(), m.group(2), m.group(3)) if m else model or "?", effort or "none")


def config_dir():
    return os.environ.get("CLAUDE_CONFIG_DIR") or os.path.expanduser("~/.claude")


def session_dirs(extra, cwd=None):
    """The folders Claude Code keeps sessions in for the working folder and each parent up to three, or `extra`."""
    if extra:
        return [d for d in extra if os.path.isdir(d)]
    base = os.path.join(config_dir(), "projects")
    out, here = [], os.path.abspath(cwd or os.getcwd())
    for _ in range(4):
        d = os.path.join(base, re.sub(r"[^A-Za-z0-9-]", "-", here))
        if os.path.isdir(d) and d not in out:
            out.append(d)
        here = os.path.dirname(here)
    return out


def local(ts):
    try:
        t = datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
    except (AttributeError, ValueError):
        return None
    return t.astimezone().replace(tzinfo=None) if t.tzinfo else t


def counted_once(seen, mid):
    """One API message spans several records, each repeating its usage: it counts once."""
    if mid in seen:
        return False
    seen.add(mid)
    return True


def is_main(q):
    return not q["side"]


def weight(q):
    return sum(u * n for u, n in zip(UNITS, q["u"]))


def read_session(path):
    """{id, dir, events: [("prompt", [text]) | ("req", {...})] in file order, cost: the last cost-state record or None}."""
    s, seen = {"id": os.path.basename(path)[:-6], "dir": os.path.dirname(path), "events": [], "cost": None}, set()
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                r = json.loads(line)
            except ValueError:
                continue
            if not isinstance(r, dict):
                continue
            t, m = r.get("type"), r.get("message")
            m = m if isinstance(m, dict) else {}
            if t == "cost-state":
                s["cost"] = r
            elif t == "assistant" and m.get("id") and counted_once(seen, m["id"]):
                u = m.get("usage") or {}
                s["events"].append(("req", {
                    "model": m.get("model"), "effort": r.get("effort"), "side": bool(r.get("isSidechain")),
                    "ts": local(r.get("timestamp")),
                    "u": tuple(u.get(k) or 0 for k in ("input_tokens", "cache_read_input_tokens",
                                                       "cache_creation_input_tokens", "output_tokens"))}))
            elif t == "user" and not r.get("isMeta") and not r.get("isSidechain"):
                c = m.get("content")
                items = [c] if isinstance(c, str) else [x.get("text", "") for x in c or []
                                                         if isinstance(x, dict) and x.get("type") == "text"]
                s["events"].append(("prompt", items))
    return s


def find_session(dirs, sid):
    """The session file `$CLAUDE_CODE_SESSION_ID` names in any folder, else the newest file of the folders; None when there is none."""
    files = [os.path.join(d, n) for d in dirs for n in os.listdir(d) if n.endswith(".jsonl")]
    if sid:
        for f in files:
            if os.path.basename(f) == sid + ".jsonl":
                return f
    return max(files, key=os.path.getmtime) if files else None


def pick_now(reqs):
    """The running request: the file's last main-loop one. A request's record is already written when its Bash runs."""
    main = [q for q in reqs if is_main(q)]
    return main[-1] if main else None


def level_of(req, env_level):
    """The level the request ran on is the record's own; `$CLAUDE_EFFORT` is only shown beside it."""
    return req["effort"]


def no_model(why):
    return {"model": None, "level": None, "rung": None, "why": why}


def now_state(dirs, sid, env_level):
    path = find_session(dirs, sid)
    if not path:
        return no_model("no session file under %s" % (", ".join(dirs) or "the working folder's Claude Code folders"))
    req = pick_now([q for k, q in read_session(path)["events"] if k == "req"])
    if not req or not req["model"]:
        return no_model("no main-loop request in %s" % os.path.basename(path))
    lvl = level_of(req, env_level)
    return {"model": req["model"], "level": lvl, "rung": rung_name(req["model"], lvl), "why": None}


def named(texts, own):
    """The block id a launch prompt names, in either form; `do task T47` takes its milestone from the opened plan."""
    pre, found = own, None
    for t in texts:
        o = OPENED_RE.search(t) if t.lstrip().startswith("<ide_opened_file>") else None
        if o:
            pre = o.group(1).upper()
    for t in texts:
        if t.lstrip().startswith("<"):
            continue
        m = EXEC_RE.search(t)
        if m:
            found = m.group(1)
        else:
            m = DO_RE.search(t)
            if m:
                found = m.group(1) if re.match(r"M[\d.]+[a-z]?-", m.group(1)) else "%s-%s" % (pre, m.group(1))
    return found


def attribute(sessions, own):
    """{session id: (session, {block id: [request]} named by a launch prompt, [request] before any)}."""
    out = {}
    for s in sessions:
        cur, groups, free = None, {}, []
        for kind, x in s["events"]:
            if kind == "prompt":
                cur = named(x, own) or cur
            elif cur:
                groups.setdefault(cur, []).append(x)
            else:
                free.append(x)
        out[s["id"]] = (s, groups, free)
    return out


# ---------------------------------------------------------------- cost
def session_cost(s):
    c = s["cost"] or {}
    t = c.get("totalCostUSD")
    return float(t) if isinstance(t, (int, float)) else None


def split_cost(total, raw, denom):
    """{block: usd}: the session's cost by each block's share of the requests' priced tokens."""
    if len(raw) == 1 and denom is None:
        return {b: total for b in raw}
    return {b: total * v / (denom if denom else sum(raw.values()) or 1) for b, v in raw.items()}


def list_rate(model):
    """USD per weight unit at first-party list prices; None for a model the table does not hold."""
    price = LIST_INPUT_PER_MTOK.get(re.sub(r"-\d{8}$", "", model or ""))
    return price / 1e6 if price else None


def folder_rates(sessions):
    """{folder: {model: USD per weight unit}} from each folder's sessions that hold a cost line: the `modelUsage` cost over
    that model's weighted tokens (a model the usage record leaves out, or prices at 0, gives no rate)."""
    cost, wt = collections.defaultdict(float), collections.defaultdict(float)
    for s in sessions:
        if session_cost(s) is None:
            continue
        usage, per = (s["cost"] or {}).get("modelUsage") or {}, collections.Counter()
        for k, q in s["events"]:
            if k == "req":
                per[q["model"]] += weight(q)
        for m, w in per.items():
            c = (usage.get(m) or {}).get("costUSD") or 0
            if c > 0 and w > 0:
                cost[(s["dir"], m)] += c
                wt[(s["dir"], m)] += w
    out = {}
    for (d, m), c in cost.items():
        out.setdefault(d, {})[m] = c / wt[(d, m)]
    return out


def token_cost(reqs, rates):
    """(usd, basis) for requests of a session with no cost line: its own tokens at the folder's rate for each model, else
    at list prices; (None, None) when a model has neither."""
    usd, basis = 0.0, 1
    for q in reqs:
        r = rates.get(q["model"])
        if r is None:
            r, basis = list_rate(q["model"]), 2
            if r is None:
                return None, None
        usd += weight(q) * r
    return usd, BASIS[basis]


def block_costs(s, groups, whole_session=True, rates=None):
    """{block: (usd or None, basis)} for one session's `groups` {block: [request]}; the session's own per-model rates price
    the tokens (modelUsage cost over the model's tokens), the total is the client's own `totalCostUSD`. A session with no
    cost line is priced from its tokens (`token_cost`) — exclusively: a total and a price never add."""
    total = session_cost(s)
    if total is None:
        return {b: token_cost(qs, rates or {}) for b, qs in groups.items()}
    every = [x for k, x in s["events"] if k == "req"]
    per_model = collections.Counter()
    for q in every:
        per_model[q["model"]] += weight(q)
    usage = (s["cost"] or {}).get("modelUsage") or {}
    rate = {m: (usage.get(m) or {}).get("costUSD", 0) / w if w else 0 for m, w in per_model.items()}
    raw = {b: sum(weight(q) * (rate.get(q["model"]) or 1) for q in qs) for b, qs in groups.items()}
    denom = None if whole_session else sum(weight(q) * (rate.get(q["model"]) or 1) for q in every)
    return {b: (v, BASIS[0]) for b, v in split_cost(total, raw, denom).items()}


# ---------------------------------------------------------------- the plan and the loop records
def read_blocks(plan_path):
    """{id: {heading, state, status, text}} over the plan and its archives; the plan's own status wins."""
    files = [plan_path] + sorted(glob.glob(os.path.join(os.path.dirname(plan_path) or ".", "*archive*.md")))
    out = {}
    for f in files:
        lines = P.read_text(f)[0].split("\n")
        for b in P.find_blocks(lines, P.fence_mask(lines)):
            e = out.setdefault(b.id, {"heading": b.heading, "state": None, "status": "", "text": ""})
            if f == plan_path or not e["state"]:
                e["heading"], e["state"], e["status"] = b.heading, b.state, b.status_text or ""
            e["text"] += "\n".join(b.body) + "\n"
    return out


def block_type(bid, kept, heading):
    if bid in kept:
        return kept[bid]
    t = P.id_type(bid)
    if t:
        return t[0][0] if t[0] else None
    tag = P.type_tag(heading)
    return next((w for w in P.TYPES if tag and w in tag.group(1)), None)


def claim_of(recs):
    """(the claim run, whether it is a fallback): the first `changed` or `all` record, else the first of any mode."""
    for r in recs:
        if r.get("mode") in ("changed", "all"):
            return r, False
    return (recs[0], True) if recs else (None, False)


def read_loops(plan_path):
    by = collections.defaultdict(list)
    with contextlib.suppress(OSError):
        with open(os.path.join(os.path.dirname(plan_path) or ".", "logs", "loop_times.jsonl"), encoding="utf-8") as f:
            for line in f:
                try:
                    r = json.loads(line)
                except ValueError:
                    continue
                if isinstance(r, dict) and isinstance(r.get("task"), str):
                    by[r["task"]].append(r)
    return by


def trace_targets(b, own):
    """The ids a D block's `Caused by:` line names — the block whose work it repairs. The order clause, `(raised by …)`,
    a Flow row and `Raised/Filed/Fired by:` name the finder, never the cause; `unknown` or no line names none."""
    out = set()
    for ln in b["text"].split("\n"):
        if re.match(r"- Caused by:", ln):
            out.update(CAUSE_RE.findall(ln))
    return out


def ladder_of(plan_path):
    """([rung strongest first], usual) from the header's `Models:` line - the first one above the first block, outside a
    fence, as plan.py reads it, however far down the header it sits; ([], None) when it holds no ladder."""
    lines = P.read_text(plan_path)[0].split("\n")
    mask = P.fence_mask(lines)
    for i in range(P.header_end(lines, P.find_blocks(lines, mask))):
        m = None if mask[i] else LADDER_RE.match(lines[i])
        if m:
            rungs, usual = [], None
            for piece in m.group(1).split(" — ")[-1].split(" · "):
                k = MARK_RE.match(piece.strip())
                if k:
                    rungs.append(k.group(1))
                    usual = k.group(1) if k.group(2) and "usual" in k.group(2) else usual
            return rungs, usual
    return [], None


# ---------------------------------------------------------------- the record
def main_rung(reqs):
    """(the rung most of the main-loop requests ran on or None, whether another rung ran >=10%)."""
    main = collections.Counter(rung_name(q["model"], q["effort"]) for q in reqs if is_main(q))
    top = main.most_common(1)
    return (top[0][0], any(n / sum(main.values()) >= MIXED for r, n in main.items() if r != top[0][0])) if top else (None, False)


def rung_of(parts, rec):
    """(rung, mixed, why, later): the block's rung is the main-loop rung of the session holding its claim run — the claim's
    time inside that session's requests; else its first session, `why` saying so — and `later` the other rungs of its later
    sessions (an escalation: its cost and the claim's verdict stay with the first rung's class)."""
    when, i, why = local(rec.get("ts")) if rec else None, 0, "first session: no claim record"
    if rec:
        why = "first session: claim time in no session"
        for n, p in enumerate(parts):
            ts = [q["ts"] for q in p["reqs"] if q["ts"]]
            if when and ts and min(ts) <= when <= max(ts) + CLAIM_SLACK:
                i, why = n, "session of the claim run"
                break
    rung, mixed = main_rung(parts[i]["reqs"])
    later = []
    for p in parts[i + 1:]:
        r = main_rung(p["reqs"])[0]
        if r and r != rung and r not in later:
            later.append(r)
    return rung, mixed, why, later


def build(plan_path, since, sess_dirs, stats=None):
    blocks = read_blocks(plan_path)
    own = next((i.split("-", 1)[0] for i in blocks), "M0")
    kept = {m.group(1): m.group(2) for m in P.KIND_KEPT_RE.finditer(P.read_text(plan_path)[0])}
    sessions = []
    for d in sess_dirs:
        for f in sorted(glob.glob(os.path.join(d, "*.jsonl"))):
            with contextlib.suppress(OSError):
                sessions.append(read_session(f))
    att = attribute(sessions, own)
    rates = folder_rates(sessions)
    priced = {sid: block_costs(s, groups, rates=rates.get(s["dir"], {})) for sid, (s, groups, free) in att.items()}
    loops = read_loops(plan_path)
    ids = [i for i, b in blocks.items() if b["state"] == "DONE" and (not since or (DATE_RE.search(b["status"]) or [""])[0] >= since)
           and block_type(i, kept, b["heading"])]
    rows = {}
    for bid in ids:
        b, ds = blocks[bid], DONE_RE.search(blocks[bid]["status"])
        parts, via = [], "prompt"
        for sid, (s, groups, free) in att.items():
            if bid in groups:
                parts.append(dict(zip(("usd", "basis"), priced[sid][bid]), sid=sid, reqs=groups[bid]))
        if not parts and ds:
            end = datetime.datetime.strptime(ds.group(1) + " " + ds.group(2), "%Y-%m-%d %H:%M")
            start = datetime.datetime.strptime(ds.group(1) + " " + (ds.group(3) or "00:00"), "%Y-%m-%d %H:%M") \
                if ds.group(3) else end - datetime.timedelta(minutes=15)
            for sid, (s, groups, free) in att.items():
                win = [q for q in free if q["ts"] and start <= q["ts"] <= end + datetime.timedelta(minutes=1)]
                if win:
                    parts.append(dict(zip(("usd", "basis"), block_costs(s, {bid: win}, whole_session=False,
                                                                        rates=rates.get(s["dir"], {}))[bid]), sid=sid, reqs=win))
                    via = "stamps"
        parts.sort(key=lambda p: min((q["ts"] for q in p["reqs"] if q["ts"]), default=datetime.datetime.max))
        rec, fallback = claim_of(loops.get(bid, []))
        rung, mixed, why, later = rung_of(parts, rec) if parts else (None, False, "no session", [])
        h = RUNG_RE.search("\n".join(l for l in b["text"].split("\n") if l.startswith("- Handoff")))
        rung = rung or ("%s %s, %s" % h.groups() if h else "unknown")
        cost = None if any(p["usd"] is None for p in parts) else sum(p["usd"] for p in parts)
        e = bool(re.search(r"^- Sizing: E\b", b["text"], re.M))
        rows[bid] = {"id": bid, "type": block_type(bid, kept, b["heading"]), "e": e, "rung": rung, "mixed": mixed,
                     "rung_basis": why, "escalated": later,
                     "claim": (rec.get("verdict") if rec else None), "claim_mode": rec.get("mode") if rec else None,
                     "claim_fallback": fallback, "cost_usd": (round(cost, 4) if parts and cost is not None else None),
                     "cost_basis": max((p["basis"] for p in parts if p["basis"]), key=BASIS.index, default=None),
                     "client_usd": sum(p["usd"] for p in parts if p["basis"] == BASIS[0]),
                     "sessions": [p["sid"] for p in parts], "via": via if parts else "handoff", "traced_d": []}
    unjoined = []
    for did, b in blocks.items():
        if did in rows and re.match(r"D\d|DBG", P.id_tail(did)):
            joined = False
            for t in trace_targets(b, own):
                if t in rows and did != t:
                    rows[t]["traced_d"].append(did)
                    joined = True
            if not joined:
                unjoined.append(did)
    for r in rows.values():
        extra = [rows[d]["cost_usd"] for d in r["traced_d"]]
        r["total_usd"] = None if r["cost_usd"] is None else round(r["cost_usd"] + sum(c or 0 for c in extra), 4)
    if stats is not None:
        stats["d_unjoined"] = unjoined
    return list(rows.values())


def classes_of(rows, rungs, usual):
    grp = collections.defaultdict(list)
    for r in rows:
        grp[(r["type"] + ("·E" if r["e"] else ""), r["rung"])].append(r)
    mean = lambda v: round(sum(v) / len(v), 4) if v else None
    out = []
    for (cls, rung), rs in sorted(grp.items()):
        tries = [x["claim"] == "GO" for x in rs if x["claim"]]
        out.append({"class": cls, "rung": rung, "n": len(rs), "first_try": round(sum(tries) / len(tries), 3) if tries else None,
                    "tries": "%d/%d" % (sum(tries), len(tries)) if tries else "n/a", "cost_per_done": mean([x["total_usd"] for x in rs if x["total_usd"] is not None])})
    usual_build = mean([x["total_usd"] for x in rows if x["type"] == "BUILD" and x["rung"] == usual and x["total_usd"] is not None])
    for c in out:
        c["verdict"] = verdict_of(c, out, rungs, usual, usual_build)
    return out


def verdict_of(c, out, rungs, usual, usual_build):
    """§0 rule (4)'s reading for one class: `cheaper on <rung>`, `costs more on <rung>`, `n<10`, else why not."""
    if not usual:
        return "no (usual) rung in the header"
    if c["rung"] == usual:
        return "usual"
    if c["rung"] not in rungs or rungs.index(c["rung"]) < rungs.index(usual):
        return "above (usual)" if c["rung"] in rungs else "off the ladder"
    if c["n"] < MIN_CLASS:
        return "n<10"
    twin = next((o for o in out if o["class"] == c["class"] and o["rung"] == usual and o["n"] >= MIN_CLASS), None)
    base, vs = (twin["cost_per_done"], "same class on (usual)") if twin else (usual_build, "(usual) BUILD blocks, all classes")
    if base is None or c["cost_per_done"] is None:
        return "list prices (no cost read)"
    return "%s on %s · vs %s" % ("costs more" if c["cost_per_done"] > base else "cheaper", c["rung"], vs)


# ---------------------------------------------------------------- the calls
def cmd_report(a):
    plan = a.plan or next(iter(sorted(glob.glob("milestones/*/*_implementation_plan.md"), key=os.path.getmtime, reverse=True)), None)
    if not plan or not os.path.isfile(plan):
        return finish(a, "report: 0 blocks", None, "no plan found (--plan <file>)")
    dirs = session_dirs(a.sessions)
    stats = {}
    rows = build(plan, a.since, dirs, stats)
    rungs, usual = ladder_of(plan)
    cls = classes_of(rows, rungs, usual)
    bare = [r for r in rows if not r["sessions"]]
    priced = lambda basis: [r for r in rows if r["cost_basis"] == basis]
    counts = ("report: %d DONE blocks · %d class/rung rows · %d session folder(s) · %d block(s) with no session file readable"
              "%s · %d block(s) priced from tokens (a session file with no cost line) · %d priced from list prices · "
              "%d D block(s) naming no cause in the record · $%.2f read from the sessions' own cost" % (
                  len(rows), len(cls), len(dirs), len(bare), " (rung from the handoff, cost null)" if bare else "",
                  len(priced(BASIS[1])), len(priced(BASIS[2])), len(stats["d_unjoined"]), sum(r["client_usd"] for r in rows)))
    lines = ["%s on %s: %d block(s) · first-try %s · %s per DONE block · %s" % (
        c["class"], c["rung"], c["n"], c["tries"], "$%.2f" % c["cost_per_done"] if c["cost_per_done"] is not None else "n/a",
        c["verdict"]) for c in cls]
    if a.blocks:
        lines += ["%s %s%s%s: %s · claim %s%s · %s%s · D %s" % (
            r["id"], r["rung"], " (mixed)" if r["mixed"] else "", " (escalated to %s)" % ", ".join(r["escalated"]) if r["escalated"] else "",
            r["via"], r["claim"], " (%s)" % r["claim_mode"] if r["claim_fallback"] else "",
            "$%.2f" % r["cost_usd"] if r["cost_usd"] is not None else "no cost",
            " (%s)" % r["cost_basis"] if r["cost_basis"] not in (None, BASIS[0]) else "",
            ",".join(r["traced_d"]) or "-") for r in rows]
    return finish(a, counts, {"blocks": rows, "classes": cls, "d_unjoined": stats["d_unjoined"]},
                  None if rows else "no DONE block read", lines)


def finish(a, counts, data, nogo, lines=()):
    if a.json:
        print(json.dumps({"cmd": "report", "counts": counts, "verdict": "NO-GO" if nogo else "GO", "data": data or {}}, indent=1))
    else:
        print(counts)
        for ln in lines:
            print(ln)
        print("=== NO-GO: %s ===" % nogo if nogo else "=== GO ===")
    return 1 if nogo else 0


# ---------------------------------------------------------------- the switch plugin: installed here?
def windows():
    return os.name == "nt"


def same_path(a, b, nt):
    m = ntpath if nt else posixpath
    return m.normcase(m.normpath(a)) == m.normcase(m.normpath(b))


def scope_here(entry, root, nt):
    """An install counts here when user-scoped, or project- or local-scoped to this very folder."""
    if entry.get("scope") == "user":
        return True
    p = entry.get("projectPath")
    return isinstance(p, str) and bool(p) and same_path(p, root, nt)


def triple(s):
    """`12.2.1` -> (12, 2, 1); anything else None."""
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", s) if isinstance(s, str) else None
    return tuple(int(x) for x in m.groups()) if m else None


def beside_version(tdir):
    """The VERSION of the switch.mjs beside this tool, as a triple; None when the file is not there or holds none."""
    try:
        with open(os.path.join(tdir, "switch.mjs"), encoding="utf-8") as f:
            m = re.search(r"^const VERSION = '([^']*)'", f.read(), re.M)
    except OSError:
        return None
    return triple(m.group(1)) if m else None


def plugin_state(cfg, root, nt, want=None):
    """`installed` · `stale` (the newest install that counts here is older than `want`, the VERSION of the switch.mjs
    beside the tool; a newer one is installed, versions compared, not bytes) · `missing` (no record, or no install that
    counts here) · `unknown` (the record unreadable, or a counting install with no readable version when `want` is set)."""
    path = os.path.join(cfg, "plugins", "installed_plugins.json")
    if not os.path.exists(path):
        return "missing"
    try:
        with open(path, encoding="utf-8") as f:
            entries = json.load(f)["plugins"].get("%s@%s" % (PLUGIN, MARKET), [])
        if not isinstance(entries, list):
            return "unknown"
        here = [e for e in entries if scope_here(e, root, nt)]
        if not here:
            return "missing"
        if want is None:
            return "installed"
        have = [v for v in (triple(e.get("version")) for e in here) if v]
        return "unknown" if not have else "installed" if max(have) >= want else "stale"
    except (OSError, ValueError, KeyError, TypeError, AttributeError):
        return "unknown"


def client_path():
    """The running client (`$CLAUDE_CODE_EXECPATH`), else `claude` — never a `claude` looked up on the PATH, which can be an older one."""
    return os.environ.get("CLAUDE_CODE_EXECPATH") or "claude"


def quoted(s, nt):
    """One double-quoted argument: PowerShell escapes ` " $ with a backtick, a POSIX shell \\ " $ ` with a backslash."""
    return '"%s"' % (re.sub(r'([`"$])', r"`\1", s) if nt else re.sub(r'([\\"$`])', r"\\\1", s))


def install_line(client, switch, folder, nt, stale=False):
    """This machine's install line: the build first (`node <switch.mjs> build --out <folder>`), then the client's two
    plugin calls - one PowerShell line on Windows, one POSIX shell line elsewhere. The last call is `plugin install`
    for `missing`; for `stale` it is `plugin update`, because the client answers `install` on an id already installed
    with "already installed" and leaves the recorded version old (Claude Code 2.1.289)."""
    c, s, d, pid = quoted(client, nt), quoted(switch, nt), quoted(folder, nt), "%s@%s" % (PLUGIN, MARKET)
    last = "plugin %s %s --scope user" % ("update" if stale else "install", pid)
    if nt:
        return "node %s build --out %s; & %s plugin marketplace add %s; & %s %s" % (s, d, c, d, c, last)
    return "node %s build --out %s && %s plugin marketplace add %s && %s %s" % (s, d, c, d, c, last)


def plugin_report(root, nt, tdir=None):
    """(state, install line or None); a crash reads `unknown`, so the model and level line always prints. The install
    line (on `missing` and `stale`) builds from the switch.mjs beside this tool into the client's own folder - never
    inside a checkout."""
    m = ntpath if nt else posixpath
    tdir = tdir or os.path.dirname(os.path.abspath(__file__))
    try:
        state = plugin_state(config_dir(), root, nt, beside_version(tdir))
    except Exception:                       # never let the plugin read hide the model and level
        return "unknown", None
    if state not in ("missing", "stale"):
        return state, None
    return state, install_line(client_path(), m.join(tdir, "switch.mjs"), m.join(config_dir(), PLUGIN + "-src"), nt, state == "stale")


def cmd_now(a):
    env = os.environ.get("CLAUDE_EFFORT") or ""
    s = now_state(session_dirs(a.sessions), os.environ.get("CLAUDE_CODE_SESSION_ID"), env)   # the model and level first
    nogo = s["model"] is None
    plugin, install = plugin_report(os.getcwd(), windows())
    if a.json:
        print(json.dumps({"cmd": "now", "model": s["model"], "level": s["level"], "rung": s["rung"], "env_level": env,
                          "plugin": plugin, "install": install}))
    elif nogo:
        print("model unknown (%s)\n=== NO-GO: no model read ===" % s["why"])
    else:
        print("model=%s level=%s plugin=%s" % (s["model"], s["level"] or "none", plugin))   # one short line, read in the chat
        if install:
            print("install: " + install)       # on `missing` and `stale` only
    return 1 if nogo else 0


# ---------------------------------------------------------------- selftest
FIXTURE_PLAN = """# M9 - fixture plan
Models: Claude Code — Opus 5.5, high (usual) · Sonnet 5.5, high · Sonnet 5.5, medium — strongest first; read 2026-10-05 from the probe

## M9-T1 · The first task · **BUILD** · Sonnet 5.5, high
- Status: DONE (2026-10-05 10:30)
- Handoff: ran on Sonnet 5.5, high.

## M9-D1 · A bug found while on the first task, caused by the third · **BUILD** · Opus 5.5, high · (raised by T1)
- Status: DONE (2026-10-05 10:40)
- Caused by: M9-T3 whose work it repairs

## M9-D2 · A second bug, caused by the first task · **BUILD** · Opus 5.5, high · (AFTER M9-D1)
- Status: DONE (2026-10-05 10:41)
- Filed by: M9-T1's reviewer
- Caused by: M9-T1

## M9-D3 · A bug whose cause is not measured · **BUILD** · Opus 5.5, high · (raised by T2)
- Status: DONE (2026-10-05 10:42)
- Caused by: unknown

## M9-D4 · A bug whose cause is outside the record · **BUILD** · Opus 5.5, high
- Status: DONE (2026-10-05 10:43)
- Caused by: M9-T99

## M9-T2 · Two blocks, one session · **BUILD** · Sonnet 5.5, high
- Status: DONE (2026-10-05 11:00)

## M9-T3 · The second of the pair · **BUILD** · Sonnet 5.5, high
- Status: DONE (2026-10-05 11:10)

## M9-T4 · No session file · **BUILD** · Sonnet 5.5, medium
- Status: DONE (2026-10-05 12:00)
- Handoff: Ran on Sonnet 5.5 (fast), medium, another box.

## M9-T5 · Found by its stamps · **BUILD** · Sonnet 5.5, medium
- Status: DONE (2026-10-05 13:20, started 13:00)

## M9-T6 · Escalated · **BUILD** · Sonnet 5.5, high
- Status: DONE (2026-10-05 15:30)

## M9-T7 · A session with no cost line · **BUILD** · Sonnet 5.5, high
- Status: DONE (2026-10-05 16:10)
"""


def _wr(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)


def _session(path, sid, events, total, usage):
    """events: ("p", text) | ("r", message id, model, effort, side, (in, cr, cc, out), local time, records)."""
    out = []
    for e in events:
        if e[0] == "p":
            out.append({"type": "user", "message": {"content": [{"type": "text", "text": t} for t in e[1:]]}})
        else:
            _, mid, model, effort, side, u, when, n = e
            ts = when.astimezone(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            for _i in range(n):
                out.append({"type": "assistant", "isSidechain": side, "effort": effort, "timestamp": ts, "message": {
                    "id": mid, "model": model, "usage": {"input_tokens": u[0], "cache_read_input_tokens": u[1],
                                                         "cache_creation_input_tokens": u[2], "output_tokens": u[3]}}})
    if total is not None:
        out.append({"type": "cost-state", "totalCostUSD": total, "modelUsage": usage})
    _wr(path, "\n".join(json.dumps(r) for r in out) + "\n")


def _fixture(tmp):
    """A plan, its loop records and six sessions; returns (plan path, session folder)."""
    mdir, sdir = os.path.join(tmp, "milestones", "m9"), os.path.join(tmp, "sessions")
    plan = os.path.join(mdir, "m9_implementation_plan.md")
    at = lambda h, m: datetime.datetime(2026, 10, 5, h, m).astimezone().isoformat()
    _wr(plan, FIXTURE_PLAN)
    _wr(os.path.join(mdir, "logs", "loop_times.jsonl"), "\n".join(json.dumps(r) for r in (
        {"ts": "t1", "task": "M9-T1", "mode": "scope", "verdict": "NO-GO"},
        {"ts": at(10, 0), "task": "M9-T1", "mode": "changed", "verdict": "GO"},
        {"ts": "t3", "task": "M9-T2", "mode": "all", "verdict": "NO-GO"},
        {"ts": "t4", "task": "M9-T3", "mode": "changed", "verdict": "GO"},
        {"ts": at(14, 10), "task": "M9-T6", "mode": "changed", "verdict": "NO-GO"},
        {"ts": at(15, 20), "task": "M9-T6", "mode": "changed", "verdict": "GO"},
        {"ts": at(16, 1), "task": "M9-T7", "mode": "changed", "verdict": "GO"})) + "\n")
    day = datetime.datetime(2026, 10, 5, 10, 0)
    opened = "<ide_opened_file>The user opened the file /x/milestones/m9/m9_implementation_plan.md in the IDE.</ide_opened_file>"
    sonnet, opus = "claude-sonnet-5-5", "claude-opus-5-5"
    use = lambda **k: {m: {"costUSD": c} for m, c in k.items()}
    _session(os.path.join(sdir, "s1.jsonl"), "s1", [
        ("p", opened, "read this and do task T1"),
        ("r", "m1", sonnet, "high", False, (2, 1000, 100, 200), day, 3),
        ("r", "m2", sonnet, "high", False, (2, 1000, 100, 200), day, 1),
        ("r", "m3", opus, "high", True, (2, 1000, 100, 200), day, 1)], 0.50, use(**{sonnet: 0.49, "claude-haiku-4-5-20251001": 0.01}))
    _session(os.path.join(sdir, "s2.jsonl"), "s2", [
        ("p", opened, "read this and do task D1"), ("r", "m4", opus, "high", False, (2, 500, 50, 100), day, 1)], 0.20, use(**{opus: 0.20}))
    _session(os.path.join(sdir, "s3.jsonl"), "s3", [
        ("p", opened, "read this and do task D2"), ("r", "m5", opus, "high", False, (2, 500, 50, 100), day, 1)], 0.10, use(**{opus: 0.10}))
    _session(os.path.join(sdir, "s4.jsonl"), "s4", [
        ("p", "Read milestones/m9/m9_implementation_plan.md and execute M9-T2"),
        ("r", "m6", sonnet, "high", False, (0, 0, 0, 300), day, 1),
        ("p", opened, "read this and do task T3"),
        ("r", "m7", sonnet, "high", False, (0, 0, 0, 100), day, 1)], 1.00, use(**{sonnet: 1.00}))
    _session(os.path.join(sdir, "s5.jsonl"), "s5", [
        ("p", "something unrelated"), ("r", "m8", sonnet, "medium", False, (0, 0, 0, 100),
                                       datetime.datetime(2026, 10, 5, 13, 10), 1)], 0.30, use(**{sonnet: 0.30}))
    t = lambda h, m: datetime.datetime(2026, 10, 5, h, m)
    _session(os.path.join(sdir, "s6.jsonl"), "s6", [
        ("p", opened, "read this and do task T6"),
        ("r", "m9", sonnet, "high", False, (0, 0, 0, 200), t(14, 0), 1),
        ("r", "m10", sonnet, "high", False, (0, 0, 0, 200), t(14, 20), 1)], 0.60, use(**{sonnet: 0.60}))
    _session(os.path.join(sdir, "s7.jsonl"), "s7", [
        ("p", opened, "read this and do task T6"),
        ("r", "m11", opus, "high", False, (0, 0, 0, 100), t(15, 0), 1),
        ("r", "m12", opus, "high", False, (0, 0, 0, 100), t(15, 10), 1),
        ("r", "m13", opus, "high", False, (0, 0, 0, 100), t(15, 20), 1)], 1.80, use(**{opus: 1.80}))
    _session(os.path.join(sdir, "s8.jsonl"), "s8", [
        ("p", opened, "read this and do task T7")] + [
        ("r", "m%d" % (14 + i), sonnet, "high", False, (0, 0, 0, 100), t(16, 0), 1) for i in range(5)], None, None)
    return plan, sdir


def _rows(plan, sdir):
    return {r["id"]: r for r in build(plan, None, [sdir])}


def _cost_from_tokens(s):
    return sum(weight(x) for k, x in s["events"] if k == "req") * 1e-6


def _finder_marks(b, own):
    """The old reading: the order clause, `(raised by …)` and `Filed by:` lines — the block that found a D, not its cause."""
    out = set()
    for ln in [b["heading"]] + [l for l in b["text"].split("\n") if re.match(r"- (?:Raised|Filed|Fired) by", l)]:
        for m in re.finditer(r"(?:(?:raised|filed|fired) by:?\**\s*`?|\(AFTER )([A-Za-z][A-Za-z0-9.-]*[A-Za-z0-9])", ln, re.I):
            out.add(m.group(1) if re.match(r"M[\d.]+[a-z]?-", m.group(1)) else "%s-%s" % (own, m.group(1)))
    return out


def _most_requests_rung(parts, rec):
    """The old reading: the rung most of the block's main-loop requests ran on, over all its sessions."""
    rung, mixed = main_rung([q for p in parts for q in p["reqs"]])
    return rung, mixed, "all sessions", []


def _no_price(reqs, rates):
    return None, None


def _scoped_first(recs):
    return (recs[0], False) if recs else (None, False)


def _windowed_ladder(plan_path):
    """The bug M0-D37 repaired: the ladder read from the first 60 lines of the plan only."""
    for ln in P.read_text(plan_path)[0].split("\n")[:60]:
        m = LADDER_RE.match(ln)
        if m:
            rungs, usual = [], None
            for piece in m.group(1).split(" — ")[-1].split(" · "):
                k = MARK_RE.match(piece.strip())
                if k:
                    rungs.append(k.group(1))
                    usual = k.group(1) if k.group(2) and "usual" in k.group(2) else usual
            return rungs, usual
    return [], None


def _deep_ladder_plan(tmp):
    """A plan whose ladder sits on line 81 of its header, after a fenced `Models:` line that is no ladder."""
    fenced = "```\nModels: Claude Code — Haiku 4.5, low (usual) — strongest first; read 2026-10-05 from a sample\n```\n"
    head = "# M9 - deep fixture\n" + fenced + "".join("Header line %d.\n" % i for i in range(75))
    path = os.path.join(tmp, "deep", "m9_implementation_plan.md")
    _wr(path, head + FIXTURE_PLAN.split("\n", 1)[1])
    return path


def _whole_to_each(total, raw, denom):
    return {b: total for b in raw}


def _now_fixture(tmp):
    """Two session files: `snow` (the one `$CLAUDE_CODE_SESSION_ID` names) and a newer decoy; returns the folder."""
    sdir, day = os.path.join(tmp, "nowsessions"), datetime.datetime(2026, 10, 5, 9, 0)
    _session(os.path.join(sdir, "snow.jsonl"), "snow", [
        ("p", "do task T1"),
        ("r", "n1", "claude-sonnet-5-5", "high", False, (0, 0, 0, 1), day, 1),
        ("r", "n2", "claude-opus-5-5", "xhigh", False, (0, 0, 0, 1), day, 2),
        ("r", "n3", "claude-haiku-4-5-20251001", None, True, (0, 0, 0, 1), day, 1)], 0.1, {})
    _session(os.path.join(sdir, "decoy.jsonl"), "decoy", [
        ("p", "other"), ("r", "d1", "claude-sonnet-5-5", "low", False, (0, 0, 0, 1), day, 1)], 0.1, {})
    os.utime(os.path.join(sdir, "snow.jsonl"), (1, 1))
    return sdir


@contextlib.contextmanager
def _env(keys):
    """The environment with `keys` set (None: unset), restored after."""
    saved = {k: os.environ.get(k) for k in keys}
    for k, v in keys.items():
        os.environ.pop(k, None)
        if v is not None:
            os.environ[k] = v
    try:
        yield
    finally:
        for k, v in saved.items():
            os.environ.pop(k, None)
            if v is not None:
                os.environ[k] = v


def _now_run(argv, sid, env, cfg, exe=None):
    """`main(argv)` with `$CLAUDE_CODE_SESSION_ID`, `$CLAUDE_EFFORT`, `$CLAUDE_CONFIG_DIR` and `$CLAUDE_CODE_EXECPATH`
    set (None: unset); returns (rc, stdout)."""
    buf = io.StringIO()
    with _env({"CLAUDE_CODE_SESSION_ID": sid, "CLAUDE_EFFORT": env, "CLAUDE_CONFIG_DIR": cfg, "CLAUDE_CODE_EXECPATH": exe}):
        with contextlib.redirect_stdout(buf):
            rc = main(argv)
    return rc, buf.getvalue()


LINUX_EXE = "/home/u/.vscode/extensions/anthropic.claude-code-2.1.289-linux-x64/resources/native-binary/claude"
WIN_EXE = r"C:\Users\Jane Doe\.vscode\extensions\anthropic.claude-code-2.1.289-win32-x64\resources\native-binary\claude.exe"
WIN_ROOT = r"C:\Users\Jane Doe\projects\app"
WIN_TOOLS = WIN_ROOT + r"\tools\pb"
WIN_CFG = r"C:\Users\Jane Doe\.claude"


def _plugin_cfgs(tmp):
    """Config folders, one per install record: user (the switch.mjs beside the tool's version) · none · another project's
    scope · this folder's · malformed · wrong shape · user, older than that version · user, newer · user, no version."""
    here = os.getcwd()
    rec = lambda *es: json.dumps({"version": 2, "plugins": {"%s@%s" % (PLUGIN, MARKET): list(es)}})
    cur = ".".join(str(n) for n in beside_version(_tools()))
    out = {}
    for name, text in [("user", rec({"scope": "user", "installPath": "/x", "version": cur})), ("none", None),
                       ("other", rec({"scope": "project", "projectPath": "/elsewhere/app", "version": cur})),
                       ("here", rec({"scope": "local", "projectPath": here + os.sep, "version": cur})),
                       ("old", rec({"scope": "user", "version": "0.0.1"})), ("new", rec({"scope": "user", "version": "99.0.0"})),
                       ("noversion", rec({"scope": "user"})),
                       ("bad", '{"version": 2, "plugins": {'), ("shape", '{"plugins": ["pb-switch@pb"]}')]:
        out[name] = os.path.join(tmp, "cfg_" + name)
        os.makedirs(os.path.join(out[name], "plugins"))
        if text is not None:
            _wr(os.path.join(out[name], "plugins", "installed_plugins.json"), text)
    return out


def _tools():
    return os.path.dirname(os.path.abspath(__file__))


def _unquoted(client, switch, folder, nt, stale=False):
    return "node %s build --out %s && %s plugin marketplace add %s && %s plugin install %s@%s --scope user" % (
        switch, folder, client, folder, client, PLUGIN, MARKET)


def _no_build(client, switch, folder, nt, stale=False):
    c, d = quoted(client, nt), quoted(folder, nt)
    return "%s plugin marketplace add %s && %s plugin install %s@%s --scope user" % (c, d, c, PLUGIN, MARKET)


def _folder_in_checkout(client, switch, folder, nt, stale=False, f=install_line):
    return f(client, switch, posixpath.join(os.getcwd(), ".claude", PLUGIN), nt, stale)


def _stale_install(client, switch, folder, nt, stale=False, f=install_line):
    return f(client, switch, folder, nt, False)


def _path_claude():
    return os.environ.get("CLAUDE_CODE_EXECPATH") or "/usr/local/bin/claude"


def _last_of_all(reqs):
    return reqs[-1] if reqs else None


def _first_main(reqs):
    return next((q for q in reqs if is_main(q)), None)


def _level_from_env(req, env_level):
    return env_level or req["effort"]


def _guessed_model(why):
    return {"model": "claude-opus-5-5", "level": "high", "rung": "Opus 5.5, high", "why": None}


def cmd_selftest(a):
    checks, plants = [], []

    def check(name, ok):
        checks.append((name, bool(ok)))

    def plant(name, fn, bug, probe):
        saved = globals()[fn]
        globals()[fn] = bug
        try:
            ok = probe()
        finally:
            globals()[fn] = saved
        plants.append((name, not ok))
        check("plant red: " + name, not ok)

    tmp = tempfile.mkdtemp(prefix="rung_record_")
    try:
        plan, sdir = _fixture(tmp)
        rows = _rows(plan, sdir)
        t1, t2, t3, t6, t7 = rows["M9-T1"], rows["M9-T2"], rows["M9-T3"], rows["M9-T6"], rows["M9-T7"]
        check("clean: eleven blocks read, the type from the id table",
              set(rows) == {"M9-T1", "M9-D1", "M9-D2", "M9-D3", "M9-D4", "M9-T2", "M9-T3", "M9-T4", "M9-T5", "M9-T6", "M9-T7"}
              and all(r["type"] == "BUILD" for r in rows.values()))
        check("clean: the rung is the main loop's, in the ladder's words, not mixed", t1["rung"] == "Sonnet 5.5, high" and not t1["mixed"])
        check("clean: one message with three records counts once (3 messages, one a sidechain's)",
              sum(1 for k, x in read_session(os.path.join(sdir, "s1.jsonl"))["events"] if k == "req") == 3)
        check("clean: the claim is the first changed run, GO; a scoped NO-GO before it is no claim",
              t1["claim"] == "GO" and t1["claim_mode"] == "changed" and not t1["claim_fallback"])
        check("clean: a block's cost is its session's totalCostUSD", t1["cost_usd"] == 0.5)
        stats = {}
        build(plan, None, [sdir], stats)
        check("clean: a D joins the block its `Caused by:` line names — D2 on T1, D1 on T3, though D1 was raised by T1 and D2 follows D1",
              t1["traced_d"] == ["M9-D2"] and t3["traced_d"] == ["M9-D1"] and not t2["traced_d"])
        check("clean: a D naming `unknown` or a block outside the record joins none, and is counted (D3, D4)",
              stats["d_unjoined"] == ["M9-D3", "M9-D4"])
        check("clean: the joined D's cost joins the block's total (T1 0.5 + D2 0.1, T3 0.25 + D1 0.2)",
              t1["total_usd"] == 0.6 and t3["total_usd"] == 0.45)
        check("clean: two blocks in one session split by tokens (300 vs 100 output tokens)",
              t2["cost_usd"] == 0.75 and t3["cost_usd"] == 0.25)
        check("clean: an escalated block keeps the first session's rung, the claim's verdict and both sessions' cost, marked escalated",
              t6["rung"] == "Sonnet 5.5, high" and t6["escalated"] == ["Opus 5.5, high"] and t6["claim"] == "NO-GO"
              and t6["rung_basis"] == "session of the claim run" and t6["cost_usd"] == 2.4 and not t1["escalated"])
        check("clean: a claim whose time falls in no session, or with no claim record → the first session's rung, flagged",
              t2["rung_basis"] == "first session: claim time in no session" and t2["rung"] == "Sonnet 5.5, high"
              and rows["M9-D1"]["rung_basis"] == "first session: no claim record")
        check("clean: a session with no cost line is priced from its tokens at the folder's rate for the model, named",
              t7["cost_basis"] == "priced from tokens" and t7["cost_usd"] == round(2500 * 2.39 / 6954, 4) and t7["rung"] == "Sonnet 5.5, high"
              and t7["sessions"] == ["s8"] and t7["claim"] == "GO" and t1["cost_basis"] == "cost-state")
        check("clean: no rate in the folder → list prices, named; a model with no price → null",
              token_cost([{"model": "claude-sonnet-5-5", "u": (0, 0, 0, 100)}], {}) == (0.001, "priced from list prices")
              and token_cost([{"model": "claude-x-9", "u": (1, 0, 0, 0)}], {}) == (None, None))
        check("clean: a block with no session keeps the rung of its handoff, cost null",
              rows["M9-T4"]["cost_usd"] is None and rows["M9-T4"]["rung"] == "Sonnet 5.5, medium" and rows["M9-T4"]["via"] == "handoff")
        check("clean: a block found by its stamps, cost the window's share of the session",
              rows["M9-T5"]["via"] == "stamps" and rows["M9-T5"]["sessions"] == ["s5"] and rows["M9-T5"]["cost_usd"] == 0.3
              and rows["M9-T5"]["rung"] == "Sonnet 5.5, medium")
        check("clean: the first-try rate counts a NO-GO claim as a miss",
              next(c for c in classes_of(list(rows.values()), ["Opus 5.5, high", "Sonnet 5.5, high", "Sonnet 5.5, medium"], "Opus 5.5, high")
                   if c["rung"] == "Sonnet 5.5, high")["tries"] == "3/5")
        check("clean: mixed when another rung ran >=10% of the main-loop requests",
              build_mixed(sdir) is True)
        lad = ["Opus 5.5, high", "Sonnet 5.5, high", "Sonnet 5.5, medium"]
        mk = lambda rung, n, cost, cls="BUILD": {"class": cls, "rung": rung, "n": n, "cost_per_done": cost}
        vs = [mk("Opus 5.5, high", 12, 1.0), mk("Sonnet 5.5, high", 12, 0.8), mk("Sonnet 5.5, medium", 12, 1.4), mk("Sonnet 5.5, medium", 9, 0.1, "BUILD·E")]
        got = [verdict_of(c, vs, lad, lad[0], 1.0).split(" · ")[0] for c in vs]
        check("verdicts: usual · cheaper · costs more · n<10", got == ["usual", "cheaper on Sonnet 5.5, high", "costs more on Sonnet 5.5, medium", "n<10"])
        check("verdicts: no (usual) rung → named, nothing guessed", verdict_of(vs[1], vs, [], None, None).startswith("no (usual)"))
        r = []
        for _ in range(2):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = main(["report", "--plan", plan, "--sessions", sdir, "--json"])
            r.append((rc, buf.getvalue()))
        check("report --json: GO, data.blocks and data.classes, the same twice",
              r[0][0] == 0 and r[0] == r[1] and {"blocks", "classes"} <= set(json.loads(r[0][1])["data"]))
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = main(["report", "--plan", os.path.join(tmp, "none.md")])
        check("report: no plan → NO-GO, exit 1", rc == 1 and "=== NO-GO:" in buf.getvalue())
        plant("one API message counted twice", "counted_once", lambda seen, mid: True,
              lambda: sum(1 for k, x in read_session(os.path.join(sdir, "s1.jsonl"))["events"] if k == "req") == 3)
        deep = _deep_ladder_plan(tmp)
        check("ladder: one on line 81 of the header, past a fenced `Models:` line, reads whole, its (usual) named",
              ladder_of(deep) == (["Opus 5.5, high", "Sonnet 5.5, high", "Sonnet 5.5, medium"], "Opus 5.5, high"))
        plant("a ladder read from the first 60 lines of the plan only", "ladder_of", _windowed_ladder, lambda: ladder_of(deep)[1] == "Opus 5.5, high")
        plant("a sidechain's tokens on the main block", "is_main", lambda q: True, lambda: not _rows(plan, sdir)["M9-T1"]["mixed"])
        plant("a scoped first run read as the claim", "claim_of", _scoped_first, lambda: _rows(plan, sdir)["M9-T1"]["claim"] == "GO")
        plant("a D joined to the block its order clause names (the finder's mark)", "trace_targets", _finder_marks,
              lambda: _rows(plan, sdir)["M9-T1"]["traced_d"] == ["M9-D2"] and _rows(plan, sdir)["M9-T3"]["traced_d"] == ["M9-D1"])
        plant("an escalated block counted on the rung most of its requests ran on", "rung_of", _most_requests_rung,
              lambda: _rows(plan, sdir)["M9-T6"]["rung"] == "Sonnet 5.5, high")
        plant("a session with no cost line nulling its block", "token_cost", _no_price,
              lambda: _rows(plan, sdir)["M9-T7"]["cost_usd"] is not None)
        plant("a two-block session charged whole to one block", "split_cost", _whole_to_each,
              lambda: _rows(plan, sdir)["M9-T2"]["cost_usd"] == 0.75)
        plant("a cost read from tokens where totalCostUSD exists", "session_cost", _cost_from_tokens,
              lambda: _rows(plan, sdir)["M9-T1"]["cost_usd"] == 0.5)
        ndir, cfg = _now_fixture(tmp), _plugin_cfgs(tmp)
        nowrun = lambda sid="snow", env="low": _now_run(["now", "--sessions", ndir, "--json"], sid, env, cfg["user"])
        rc, out = nowrun()
        d = json.loads(out)
        check("now: the last main-loop request, the model and the record's level (not $CLAUDE_EFFORT=low), one message of two records once",
              rc == 0 and d == {"cmd": "now", "model": "claude-opus-5-5", "level": "xhigh", "rung": "Opus 5.5, xhigh", "env_level": "low",
                                "plugin": "installed", "install": None})
        rc, out = _now_run(["now", "--sessions", ndir, "--json"], None, "high", cfg["user"])
        check("now: no session id → the newest file of the folders", rc == 0 and json.loads(out)["level"] == "low")
        rc, out = _now_run(["now", "--sessions", os.path.join(tmp, "empty")], "snow", "high", cfg["user"])
        check("now: no session file → `model unknown (<why>)`, NO-GO, exit 1, no model printed",
              rc == 1 and out.startswith("model unknown (") and "=== NO-GO: no model read ===" in out and "claude-" not in out)
        lines = lambda c, exe=LINUX_EXE: _now_run(["now", "--sessions", ndir], "snow", None, cfg[c], exe)[1].splitlines()
        head = "model=claude-opus-5-5 level=xhigh plugin="
        tool = os.path.join(_tools(), "switch.mjs")
        posix = lambda c, exe=LINUX_EXE, verb="install": 'install: node "%s" build --out "%s" && "%s" plugin marketplace add "%s" && "%s" plugin %s pb-switch@pb --scope user' % (
            tool, os.path.join(cfg[c], PLUGIN + "-src"), exe, os.path.join(cfg[c], PLUGIN + "-src"), exe, verb)
        installed = lambda: lines("user") == [head + "installed"]
        check("now: installed → one line, the model, the record's level and plugin=installed, no install line", installed())
        check("now: installed under local scope for this folder → installed", lines("here") == [head + "installed"])
        missing = lambda: lines("none") == [head + "missing", posix("none")]
        check("now: missing → the POSIX line: the build of the switch.mjs beside the tool first, then the client's two calls, quoted", missing())
        other = lambda: lines("other") == [head + "missing", posix("other")]
        check("now: installed under another project's scope only → missing here", other())
        stale = lambda: lines("old") == [head + "stale", posix("old", verb="update")]
        check("now: an install older than the switch.mjs beside the tool → plugin=stale and the install line whose last call is `plugin update` (`install` on an installed id leaves the old version recorded)", stale())
        newer = lambda: lines("new") == [head + "installed"]
        check("now: an install newer than the switch.mjs beside the tool is installed (versions compared, not bytes)", newer())
        check("now: a counting install with no readable version → unknown, no install line", lines("noversion") == [head + "unknown"])
        win = ('node "%s\\switch.mjs" build --out "%s\\pb-switch-src"; & "%s" plugin marketplace add "%s\\pb-switch-src"; '
               '& "%s" plugin install pb-switch@pb --scope user') % (WIN_TOOLS, WIN_CFG, WIN_EXE, WIN_CFG, WIN_EXE)
        def windows_line():
            with _env({"CLAUDE_CONFIG_DIR": WIN_CFG, "CLAUDE_CODE_EXECPATH": WIN_EXE}):
                return plugin_report(WIN_ROOT, True, WIN_TOOLS) == ("missing", win)
        check("now: missing on Windows → one PowerShell line, the build first, the spaced paths quoted", windows_line())
        win_up = win.replace("plugin install", "plugin update")
        def windows_stale():
            with _env({"CLAUDE_CONFIG_DIR": WIN_CFG, "CLAUDE_CODE_EXECPATH": WIN_EXE}):
                return install_line(WIN_EXE, WIN_TOOLS + "\\switch.mjs", WIN_CFG + "\\pb-switch-src", True, True) == win_up
        check("now: stale on Windows → the same PowerShell line, its last call `plugin update`", windows_stale())
        bare = lambda: lines("none", None) == [head + "missing", posix("none", "claude")]
        check("now: no $CLAUDE_CODE_EXECPATH → `claude`, never a path looked up", bare())
        unknown = lambda: lines("bad") == [head + "unknown"] and lines("shape") == [head + "unknown"]
        check("now: a malformed record or one of the wrong shape → plugin=unknown, no install line", unknown())
        saved = plugin_state
        globals()["plugin_state"] = lambda c, r, nt, want=None: 1 / 0
        try:
            check("now: a crash in the plugin read → plugin=unknown, the model and level still printed", lines("user") == [head + "unknown"])
        finally:
            globals()["plugin_state"] = saved
        plant("an install that counts here read as missing", "scope_here", lambda e, r, nt: False, installed)
        plant("the install line's paths unquoted", "install_line", _unquoted, missing)
        plant("a stale line that ends in `plugin install`", "install_line", _stale_install, stale)
        plant("a stale PowerShell line that ends in `plugin install`", "install_line", _stale_install, windows_stale)
        plant("an install line without its build", "install_line", _no_build, missing)
        plant("an install line whose folder is inside the checkout", "install_line", _folder_in_checkout, missing)
        plant("a stale copy read as installed", "plugin_state",
              lambda c, r, nt, want=None, f=plugin_state: f(c, r, nt) , stale)
        plant("a newer install read as stale (bytes, not versions)", "plugin_state",
              lambda c, r, nt, want=None, f=plugin_state: "stale" if f(c, r, nt) == "installed" and "99.0.0" in open(os.path.join(c, "plugins", "installed_plugins.json")).read() else f(c, r, nt, want), newer)
        plant("another project's install counted here", "scope_here", lambda e, r, nt: e.get("scope") in ("user", "project", "local"), other)
        plant("the POSIX line on Windows", "install_line", lambda c, s, d, nt, stale=False, f=install_line: f(c, s, d, False, stale), windows_line)
        plant("the PATH's claude named when $CLAUDE_CODE_EXECPATH is unset", "client_path", _path_claude, bare)
        plant("an unreadable record read as missing", "plugin_state",
              lambda c, r, nt, want=None, f=plugin_state: "missing" if f(c, r, nt, want) == "unknown" else f(c, r, nt, want), unknown)
        plant("a sidechain's record read as the running request", "pick_now", _last_of_all,
              lambda: json.loads(nowrun()[1])["model"] == "claude-opus-5-5")
        plant("the file's first main-loop record read instead of its last", "pick_now", _first_main,
              lambda: json.loads(nowrun()[1])["model"] == "claude-opus-5-5")
        plant("a level read from $CLAUDE_EFFORT where the record's differs", "level_of", _level_from_env,
              lambda: json.loads(nowrun()[1])["level"] == "xhigh")
        plant("no session file printed as a model", "no_model", _guessed_model,
              lambda: _now_run(["now", "--sessions", os.path.join(tmp, "empty")], "snow", "high", cfg["user"])[0] == 1)
    except Exception as e:                      # a crash is a failed check, never a traceback in place of the verdict
        check("selftest stopped after %d checks: %s: %s" % (len(checks), type(e).__name__, e), False)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    failed = [n for n, ok in checks if not ok]
    if a.json:
        print(json.dumps({"cmd": "selftest", "checks": len(checks), "failed": failed, "plants_red": sum(1 for _, r in plants if r)}))
    else:
        print("selftest: checks=%d passed=%d failed=%d · plants red %d/%d" % (
            len(checks), len(checks) - len(failed), len(failed), sum(1 for _, r in plants if r), len(plants)))
        for n in failed:
            print("FAIL: " + n)
    bad = not checks or failed
    if not a.json:
        print("=== NO-GO: %s ===" % ("zero checks ran" if not checks else "%d check(s) failed" % len(failed)) if bad else "=== GO ===")
    return 1 if bad else 0


def build_mixed(sdir):
    """A session whose main loop ran 3 requests on one rung and 1 on another reads `mixed`."""
    day = datetime.datetime(2026, 10, 5, 9, 0)
    p = os.path.join(os.path.dirname(sdir), "mixed")
    ev = [("p", "do task T1")] + [("r", "x%d" % i, "claude-sonnet-5-5", "high" if i else "medium", False, (0, 0, 0, 1), day, 1)
                                    for i in range(4)]
    _session(os.path.join(p, "m.jsonl"), "m", ev, 0.1, {})
    plan = os.path.join(os.path.dirname(sdir), "milestones", "m9", "m9_implementation_plan.md")
    rows = build(plan, None, [p])
    return bool(rows) and rows[0]["mixed"]


# ---------------------------------------------------------------- CLI
def main(argv=None):
    ap = argparse.ArgumentParser(prog="rung_record.py", description="Each DONE block's rung, claim run and cost, by class "
                                 "and rung. Guide: docs/agent/rung_record.md")
    sub = ap.add_subparsers(dest="cmd", required=True, metavar="<command>")
    sp = sub.add_parser("report", help="the record: per block, per class and rung, the rule (4) verdict")
    sp.add_argument("--plan", help="default: the newest milestones/*/*_implementation_plan.md")
    sp.add_argument("--since", metavar="YYYY-MM-DD", help="only blocks DONE on or after this date")
    sp.add_argument("--sessions", nargs="+", metavar="dir", help="default: the folders the working folder and each parent up to three map to")
    sp.add_argument("--blocks", action="store_true", help="also one line per block")
    sp.add_argument("--json", action="store_true", help="machine output, for agents")
    sp = sub.add_parser("now", help="the model and level of the running request, read from this session's file")
    sp.add_argument("--sessions", nargs="+", metavar="dir", help="default: the folders the working folder and each parent up to three map to")
    sp.add_argument("--json", action="store_true", help="machine output, for agents")
    sp = sub.add_parser("selftest", help="red-armed selftest on a temp fixture")
    sp.add_argument("--json", action="store_true", help="machine output, for agents")
    a = ap.parse_args(argv)
    return {"report": cmd_report, "now": cmd_now, "selftest": cmd_selftest}[a.cmd](a)


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):      # a cp437 or ASCII console must not crash on a `·` or a `—`
        with contextlib.suppress(AttributeError, ValueError, OSError, io.UnsupportedOperation):
            stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    try:
        sys.exit(main())
    except Exception as e:                  # a crash is a NO-GO with its cause, never a bare traceback
        print("=== NO-GO: rung_record.py stopped: %s: %s ===" % (type(e).__name__, e))
        sys.exit(1)
