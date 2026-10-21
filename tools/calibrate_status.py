#!/usr/bin/env python3
"""Where a running `calibrate` item stands (M0-D73, M0-D74). Standard library only, read-only.

  python3 tools/calibrate_status.py [DIR] [--item <label>] [--bracket LO,HI] [--log FILE]

DIR is an invocation's build/calibrate_runs/<stamp>/ (default: the newest one). It reads DIR/<label>-<NN>/summary.json
(a finished life), the running life's progress.json and the folders' times and nothing else: it writes nothing, opens no GPU
and links no engine code, so it runs beside a live `calibrate`. Prints at most 8 lines and ends `=== GO ===` or
`=== NO-GO: <reason> ===`.

The lives of a bisected item (m_ign_sb, m_up_sb, m_bh_sb) are counted exactly: `calibrate::bisect` runs both ends of the
bracket, then halves it (geometric mean) until it is within 2 % (contract §2.11), so the count is
2 + ceil(log2(ln(HI/LO) / ln(1 + 0.02))) from the bracket alone. LO is life 01's mass and HI life 02's once it has
finished; before that --bracket, else the item's default (calibrate.rs `default_bracket`, in units of M0 = UNIT_MASS).
While a life runs, `calibrate` writes DIR/<label>-<NN>/progress.json every `--progress-every` steps, 10,000 by default
(atomically; M0-D74, M0-D75): item, mass, step, sim_time, stage, the stages it has shown with the step each began,
heat_unconverged, `started` (UTC, this process's start of the life), `progress_every` and, since M0-D77, `continued_from_step`
(the checkpoint a `--resume` continued the life from, else null) and `checkpoint_step` (the last one saved, else null). A file is
marked stale past max(15 min, 3 × the tick's expected interval): `progress_every` over the life's pace from `started` to the file's
last write — the pace of this process's own steps (those past `continued_from_step`), and the life's line says what a stop would
lose: the steps since `checkpoint_step`. The running life's line, its stage trail and an ETA for the life come from it; --log FILE (the run's output kept with
`| tee FILE`) still wins for the stage when it holds a progress line of the running life. The ETA is a guess: the running life
is sized by the finished life nearest in mass (its steps, at the rate this life shows), the item's by the finished lives' wall
times as a range and a mean, marked so while fewer than 3 have finished.
"""
import datetime
import json
import math
import os
import re
import sys
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
RUNS = os.path.join(ROOT, "build", "calibrate_runs")
TOLERANCE = 0.02  # calibrate.rs BISECT_TOLERANCE, contract §2.11
GUESS_BELOW = 3  # lives finished before an ETA stops being a guess
# calibrate.rs UNIT_MASS = 2π · SUN_SIGMA_C · SUN_RADIUS² / 3 (scene.rs); the cases check it against the source.
UNIT_MASS = 3351.032
# Default brackets in M0 units (calibrate.rs `Threshold::default_bracket`); m_bh's LO is 1.25 × the file's m_up_sb, so
# it has none here: life 01's mass or --bracket gives it.
DEFAULT_BRACKET = {"m_ign_sb": (0.05, 1.0), "m_up_sb": (1.0, 14.0), "m_bh_sb": (None, 20.0)}
PRESET_LIVES = 3  # preset_mass_sb: the Sun-like, the heavy and the neutron-star presets, each one life
ITEMS = tuple(DEFAULT_BRACKET) + ("preset_mass_sb",)
PROGRESS = re.compile(r"SR-CALIBRATE progress item=(?P<item>\S+) mass=(?P<mass>\S+) step=(?P<step>\d+) sim_time=(?P<sim>\S+)(?: heat_unconverged=\d+)? stage=(?P<stage>.*?)\s*$")
STALE_S = 900.0  # a progress.json older than this, or than STALE_TICKS ticks at the life's pace if longer, is marked as a stopped life's
STALE_TICKS = 3
TICK_STEPS = 10_000  # calibrate.rs PROGRESS_EVERY: the tick of a progress.json that does not say (written before M0-D75)
BAR = 20
FOLDER = re.compile(r"^(?P<label>.+)-(?P<nn>\d{2,})$")


def bisect_lives(lo, hi, tolerance=TOLERANCE):
    """The lives `calibrate::bisect` spends on lo..hi: both ends, then one per halving (geometric mean) down to `tolerance`.
    The loop is bisect's own, so a bracket on the tolerance's edge counts as it does; the closed form is
    2 + ceil(log2(ln(hi/lo) / ln(1 + tolerance)))."""
    if not (0 < lo < hi) or not math.isfinite(hi):
        raise ValueError("bracket %s..%s: needs 0 < LO < HI" % (lo, hi))
    runs = 2
    while hi / lo - 1.0 > tolerance:
        lo = math.sqrt(lo * hi)  # the side the run lands on does not change how many halvings the ratio needs
        runs += 1
    return runs


def item_of(label):
    """The item a folder's label belongs to: a bisected item's own name, the preset lives' `preset_<name>` → preset_mass_sb."""
    if label in DEFAULT_BRACKET:
        return label
    return "preset_mass_sb" if label.startswith("preset_") else None


def read_life(path):
    """The finished life in `path`/summary.json, or None: a file that is missing, torn or whose `until` was not met."""
    try:
        with open(os.path.join(path, "summary.json"), encoding="utf-8") as f:
            summary = json.load(f)
        if summary["until"]["met"] is not True:
            return None
        mass = float(str(summary["scene"]).rsplit(":", 1)[1])
        wall = float(summary["wall_s"])
        steps = int(summary["steps"])
    except (OSError, ValueError, KeyError, IndexError, TypeError):
        return None
    ending = "-"
    for event in summary.get("events") or []:
        if isinstance(event, dict) and event.get("kind") == "ending":
            ending = str((event.get("detail") or {}).get("stage", "ending"))
    return {"mass": mass, "wall": wall, "steps": steps, "ending": ending}


def scan(run_dir):
    """Every `<label>-<NN>` folder of the invocation, in run order: (nn, label, path, finished life or None)."""
    folders = []
    for name in os.listdir(run_dir):
        path = os.path.join(run_dir, name)
        m = FOLDER.match(name)
        if m and os.path.isdir(path):
            folders.append((int(m["nn"]), m["label"], path, read_life(path)))
    return sorted(folders, key=lambda f: f[0])


def last_progress(log, label):
    """The last `SR-CALIBRATE progress` line of `label`'s running life in the run's output `log`, or None: a `run` line after it
    means that life ended and the next has not reported yet. Raises ValueError when the log cannot be read."""
    try:
        with open(log, encoding="utf-8", errors="replace") as f:
            text = f.read()
    except OSError as e:
        raise ValueError("--log %s: %s" % (log, e.strerror or e))
    found = None
    for line in text.splitlines():
        if "SR-CALIBRATE run " in line:
            found = None
        m = PROGRESS.search(line)
        if m and (label is None or m["item"] == label):
            found = m
    return found


def fmt_time(seconds):
    seconds = int(round(seconds))
    h, rest = divmod(seconds, 3600)
    m, s = divmod(rest, 60)
    return "%dh%02dm" % (h, m) if h else ("%dm%02ds" % (m, s) if m else "%ds" % s)


def parse_utc(text):
    """`2026-10-20T19:07:59Z` (calibrate.rs `utc_iso`) as epoch seconds, or None."""
    try:
        return datetime.datetime.strptime(text, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=datetime.timezone.utc).timestamp()
    except (TypeError, ValueError):
        return None


def read_progress(path, label, now):
    """The running life's progress.json (M0-D74) as a dict, or None: a file that is missing, torn (a half-written file does not
    parse; calibrate renames a whole one into place, so this is only a guard), wrong in a field, or another item's. It is read only
    for the life the folders name as running, never for a finished one."""
    file = os.path.join(path, "progress.json")
    try:
        with open(file, encoding="utf-8") as f:
            raw = json.load(f)
        if raw["item"] != label:
            return None
        found = {"mass": float(raw["mass"]), "step": int(raw["step"]), "sim": float(raw["sim_time"]), "stage": str(raw["stage"]),
                 "unconverged": int(raw.get("heat_unconverged", 0)), "started": parse_utc(raw.get("started")),
                 "every": int(raw.get("progress_every", TICK_STEPS)),
                 # M0-D77: where this process began (a continued life) and the last checkpoint saved; a file written before it
                 # has neither key, which reads as "no checkpoint to speak of".
                 "continued": int(raw["continued_from_step"]) if raw.get("continued_from_step") is not None else 0,
                 "save_known": "checkpoint_step" in raw,
                 "saved": int(raw["checkpoint_step"]) if raw.get("checkpoint_step") is not None else None,
                 "age": max(0.0, now - os.stat(file).st_mtime)}
        found["stages"] = [(str(x["stage"]), int(x["since_step"])) for x in raw.get("stages") or []] or [(found["stage"], None)]
    except (OSError, ValueError, KeyError, TypeError):
        return None
    return found


def stale_after(progress, now):
    """Seconds a progress.json may go unwritten before it reads stale: STALE_S, or STALE_TICKS ticks at the life's own pace
    (`progress_every` steps over its steps per second, from `started` to the file's last write) when that is longer."""
    ran = None
    if progress["started"] is not None:
        ran = (now - progress["age"]) - progress["started"]
    own = progress["step"] - progress["continued"]  # this process's steps: a continued life did not run the ones before its checkpoint
    if not ran or ran <= 0 or own <= 0 or progress["every"] <= 0:
        return STALE_S
    return max(STALE_S, STALE_TICKS * progress["every"] * ran / own)


def trail(progress):
    """The stages the life has shown, in order, the current one in brackets: `cloud › collapsing › [main_sequence] since step 60,000`."""
    names = [name for name, _ in progress["stages"]]
    since = progress["stages"][-1][1]
    text = " › ".join(names[:-1] + ["[%s]" % names[-1]])
    return text + (" since step {:,}".format(since) if since is not None else "")


def life_line(progress, done, elapsed):
    """(the running life's progress line, its seconds left or None). The life is sized by the finished life nearest in mass —
    a guess: lives of other masses differ — and its pace is this life's own steps over the time it has run."""
    step = progress["step"]
    own = step - progress["continued"]
    rate = own / elapsed if elapsed and elapsed > 0 and own > 0 else None
    pace = " at {:.0f} steps/s".format(rate) if rate else ""
    if progress["save_known"]:
        if progress["saved"] is None:
            pace += " · no checkpoint yet: a stop loses the life so far"
        else:
            pace += " · checkpoint at step {:,}: a stop now loses {:,} steps".format(progress["saved"], max(0, step - progress["saved"]))
    if not done or not progress["mass"] > 0:
        return "  this life: step {:,}{} · no finished life to size it against".format(step, pace), None
    nn, ref = min(done, key=lambda d: abs(math.log(d[1]["mass"] / progress["mass"])))
    if step >= ref["steps"]:
        return "  this life: step {:,}, past life {:02d}'s {:,} steps (nearest mass){} · no ETA for it".format(step, nn, ref["steps"], pace), None
    fraction = step / ref["steps"]
    filled = int(fraction * BAR)
    left = (ref["steps"] - step) / rate if rate else None
    eta = " · ~{} left (a guess)".format(fmt_time(left)) if left is not None else ""
    return "  this life: [{}{}] {:.0f}% of life {:02d}'s {:,} steps (nearest mass){}{}".format(
        "#" * filled, "-" * (BAR - filled), fraction * 100, nn, ref["steps"], pace, eta), left


def parse_bracket(text):
    try:
        lo, hi = (float(x) for x in text.split(","))
    except ValueError:
        raise ValueError("--bracket %r: expected LO,HI, two numbers" % text)
    return lo, hi


def status(run_dir, item=None, bracket=None, now=None, log=None):
    """The report's lines for `item` in `run_dir`; raises ValueError (the NO-GO reason) when there is nothing to say."""
    now = time.time() if now is None else now
    if not os.path.isdir(run_dir):
        raise ValueError("%s is not a folder" % run_dir)
    folders = scan(run_dir)
    if item is None:
        known = [f for f in folders if item_of(f[1])]
        if not known:
            raise ValueError("no <item>-<NN> folder of a calibrate item under %s" % run_dir)
        item = item_of(known[-1][1])
    elif item not in ITEMS:
        raise ValueError("--item %s: not a calibrate item with lives (%s)" % (item, ", ".join(ITEMS)))
    mine = [f for f in folders if item_of(f[1]) == item]
    done = [(nn, life) for nn, _, _, life in mine if life]
    lives = [life for _, life in done]
    unfinished = [f for f in mine if not f[3]]
    running = unfinished[-1] if unfinished and unfinished[-1][0] == mine[-1][0] else None
    progress = read_progress(running[2], running[1], now) if running else None
    lines = ["item %s · %s · %d finished" % (item, os.path.basename(os.path.normpath(run_dir)), len(done))]
    for nn, life in done[-(2 if progress else 3):]:
        lines.append("  life %02d  mass %.3f  %s  %d steps  %s" % (nn, life["mass"], life["ending"], life["steps"], fmt_time(life["wall"])))
    elapsed, life_left = None, None
    if running:
        elapsed = max(0.0, now - os.stat(running[2]).st_mtime)
        seen = last_progress(log, running[1]) if log else None
        if progress and progress["started"] is not None:
            elapsed = max(0.0, now - progress["started"])  # this process's own start: a --resume'd life keeps its old folder's mtime
        if seen:
            lines.append("  life %02d running %s · stage %s, step %s, sim_time %s (from the log)" % (running[0], fmt_time(elapsed), seen["stage"], seen["step"], seen["sim"]))
        elif progress:
            age = "updated %s ago" % fmt_time(progress["age"]) + (" — stale, is it stopped?" if progress["age"] > stale_after(progress, now) else "")
            lines.append("  life %02d running %s · stage %s, step {:,}, sim_time %.2f (from progress.json, %s)".format(progress["step"]) % (
                running[0], fmt_time(elapsed), progress["stage"], progress["sim"], age))
        else:
            lines.append("  life %02d running: its folder was made %s ago (a stopped item, or a life re-run by --resume in an old folder, reads the same)%s" % (
                running[0], fmt_time(elapsed), " · no progress line in the log yet" if log else ""))
        if progress:
            heat = " · heat unconverged in {:,} steps (counted, not printed)".format(progress["unconverged"]) if progress["unconverged"] else ""
            lines.append("  path: " + trail(progress) + heat)
            line, life_left = life_line(progress, done, elapsed)
            lines.append(line)
    else:
        lines.append("  no life running in %s" % item)

    named = ""
    if item == "preset_mass_sb":
        total = PRESET_LIVES
    else:
        lo_default, hi_default = DEFAULT_BRACKET[item]
        lo, hi = bracket if bracket else (lo_default and lo_default * UNIT_MASS, hi_default * UNIT_MASS)
        named = "bracket from --bracket" if bracket else "bracket from the item's default"
        if mine and mine[0][3]:
            lo = mine[0][3]["mass"]
        if len(mine) >= 2 and mine[1][3]:
            hi = mine[1][3]["mass"]
            named = "bracket from lives 1 and 2"
        if lo is None:
            raise ValueError("%s has no default bracket before its first life ends: give --bracket LO,HI" % item)
        total = bisect_lives(lo, hi)
    left = max(0, total - len(done))
    if left == 0:
        lines.append("item complete: %d lives, %s of wall time" % (len(done), fmt_time(sum(l["wall"] for l in lives))))
        return lines
    lines.append("lives left %d of %d%s" % (left, total, (" · " + named) if named else ""))
    if not lives:
        lines.append("ETA: none yet, no life has finished (a Sun-like life is ~37 min, a heavy one ~8: the lead's measure)")
        return lines
    walls = [l["wall"] for l in lives]
    mark = " (a guess: %d of %d lives needed)" % (len(walls), GUESS_BELOW) if len(walls) < GUESS_BELOW else ""
    pieces = []
    for x in (min(walls), sum(walls) / len(walls), max(walls)):
        # The running life has `elapsed` behind it, never below zero left: a life past its guess counts as about done. Its own ETA
        # (progress.json's step against the nearest finished life) stands in for the wall-time guess when there is one.
        rest = life_left if life_left is not None else max(0.0, x - elapsed) if running else x
        pieces.append(fmt_time((left - 1) * x + rest if running else left * x))
    lines.append("ETA %s to %s, mean %s%s" % (pieces[0], pieces[2], pieces[1], mark))
    return lines


def newest_dir():
    if not os.path.isdir(RUNS):
        raise ValueError("no build/calibrate_runs folder here")
    stamps = sorted(d for d in os.listdir(RUNS) if os.path.isdir(os.path.join(RUNS, d)))
    if not stamps:
        raise ValueError("build/calibrate_runs holds no invocation")
    return os.path.join(RUNS, stamps[-1])


def main(argv):
    args, item, bracket, run_dir, log = list(argv), None, None, None, None
    try:
        while args:
            a = args.pop(0)
            if a == "--item" and args:
                item = args.pop(0)
            elif a == "--bracket" and args:
                bracket = parse_bracket(args.pop(0))
            elif a == "--log" and args:
                log = args.pop(0)
            elif a.startswith("-"):
                raise ValueError("usage: calibrate_status.py [DIR] [--item <label>] [--bracket LO,HI] [--log FILE]")
            elif run_dir is None:
                run_dir = a
            else:
                raise ValueError("one DIR only")
        lines = status(run_dir or newest_dir(), item, bracket, log=log)
    except ValueError as e:
        print("=== NO-GO: %s ===" % e)
        return 1
    print("\n".join(lines[:8]))
    print("=== GO ===")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
