#!/usr/bin/env python3
"""The cases of tools/calibrate_status.py (M0-D73, M0-D74, M0-D77). Standard library only; no GPU, no engine, no real calibration run.

  python3 tools/calibrate_status_cases.py

Each case builds a fixture invocation folder (`<label>-<NN>/summary.json`) under a temp dir and reads what the script
prints. Ends in a counts line (`N passed, F failed`) and `=== GO ===` or `=== NO-GO: <reason> ===`.
"""
import contextlib
import io
import json
import math
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import calibrate_status as cs  # noqa: E402

NOW = 1_000_000.0


def life(root, label, nn, mass, wall=600.0, steps=1000, ending="white_dwarf", met=True):
    path = os.path.join(root, "%s-%02d" % (label, nn))
    os.makedirs(path, exist_ok=True)
    summary = {"scene": "calibrate:%s:%.3f" % (label, mass), "wall_s": wall, "steps": steps, "until": {"condition": "ending", "met": met},
               "events": [{"kind": "ignition", "detail": {}}, {"kind": "ending", "detail": {"stage": ending}}]}
    with open(os.path.join(path, "summary.json"), "w", encoding="utf-8") as f:
        json.dump(summary, f)
    return path


def running(root, label, nn, started=NOW - 90.0):
    path = os.path.join(root, "%s-%02d" % (label, nn))
    os.makedirs(os.path.join(path, "dumps"), exist_ok=True)
    os.utime(path, (started, started))
    return path


def started_at(seconds_ago):
    """`calibrate.rs`'s `utc_iso` of NOW - seconds_ago."""
    import datetime
    return datetime.datetime.fromtimestamp(NOW - seconds_ago, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def progress_file(root, label, nn, updated=NOW - 5.0, text=None, **fields):
    """The running life's progress.json as calibrate writes it (item, mass, step, sim_time, stage, stages, heat_unconverged, started)."""
    path = os.path.join(root, "%s-%02d" % (label, nn), "progress.json")
    record = {"item": label, "mass": 46914.448, "step": 60000, "sim_time": 480.25, "stage": "main_sequence",
              "stages": [{"stage": "cloud", "since_step": 10000}, {"stage": "collapsing", "since_step": 20000}, {"stage": "main_sequence", "since_step": 30000}],
              "heat_unconverged": 0, "started": started_at(600.0)}
    record.update(fields)
    folder = os.path.dirname(path)
    os.makedirs(folder, exist_ok=True)
    mtime = os.stat(folder).st_mtime
    with open(path, "w", encoding="utf-8") as f:
        f.write(text if text is not None else json.dumps(record))
    os.utime(path, (updated, updated))
    os.utime(folder, (mtime, mtime))  # the fixture's folder keeps the age `running` gave it, as the case reads it
    return path


def time_now():
    import time
    return time.time()


def report(root, **kw):
    """(lines, nogo reason) of `status` on `root`."""
    try:
        return cs.status(root, now=NOW, **kw), None
    except ValueError as e:
        return None, str(e)


def listing(root):
    """Every path under root with its size and mtime: a read that changes a byte changes this."""
    out = []
    for base, dirs, files in os.walk(root):
        for name in sorted(dirs + files):
            st = os.stat(os.path.join(base, name))
            out.append((os.path.join(base, name), st.st_size, st.st_mtime_ns))
    return sorted(out)


def port_runs(lo, hi, tolerance=0.02):
    """The closed form of the same count: both ends, then ceil(log2(ln(hi/lo) / ln(1 + tolerance))) halvings."""
    if hi / lo - 1.0 <= tolerance:
        return 2
    return 2 + math.ceil(math.log2(math.log(hi / lo) / math.log(1 + tolerance)))


def case_finished_lives(tmp):
    life(tmp, "m_up_sb", 1, 3351.032, wall=510.0, steps=142801)
    life(tmp, "m_up_sb", 2, 46914.448, wall=420.0, ending="black_hole")
    life(tmp, "m_up_sb", 3, 12000.0, wall=2200.0)
    lines, why = report(tmp)
    text = "\n".join(lines or [])
    assert why is None and lines[0].startswith("item m_up_sb") and "3 finished" in lines[0], (why, text)
    assert "life 01  mass 3351.032  white_dwarf  142801 steps  8m30s" in text, text
    assert "life 02  mass 46914.448  black_hole" in text, text
    # 14 M0 over 1 M0: ln(14)/ln(1.02) = 133.3 → 2^7 = 128 < 133.3 ≤ 256: 8 halvings, 10 lives, 7 left.
    assert "lives left 7 of 10" in text, text
    assert "bracket from lives 1 and 2" in text, text
    # 7 lives left at 420–2200 s each: 49m to 4h16m, mean (510+420+2200)/3 = 1043 s → 2h01m. Three lives: not a guess.
    assert "ETA 49m00s to 4h16m, mean 2h01m" in text and "guess" not in text, text


def case_running_life(tmp):
    life(tmp, "m_ign_sb", 1, 167.552, wall=1.0)
    life(tmp, "m_ign_sb", 2, 3351.032, wall=3.0)
    running(tmp, "m_ign_sb", 3, started=NOW - 90.0)
    lines, why = report(tmp)
    text = "\n".join(lines or [])
    assert why is None and "life 03 running: its folder was made 1m30s ago" in text, (why, text)
    assert "2 finished" in lines[0] and "lives left 8 of 10" in text and "a guess: 2 of 3 lives needed" in text, text
    # Finished lives must not be taken for the running one, nor the running one for finished.
    assert "life 03  mass" not in text, text


def case_torn_and_cut(tmp):
    life(tmp, "m_up_sb", 1, 3351.032)
    torn = os.path.join(tmp, "m_up_sb-02")
    os.makedirs(torn)
    with open(os.path.join(torn, "summary.json"), "w", encoding="utf-8") as f:
        f.write('{"scene": "calibrate:m_up_sb:46914.448", "wall_s": 12.0, "steps": 5, "until": {"con')
    os.utime(torn, (NOW - 30.0, NOW - 30.0))
    lines, why = report(tmp)
    text = "\n".join(lines or [])
    assert why is None and "1 finished" in lines[0] and "life 02 running" in text, (why, text)
    life(tmp, "m_up_sb", 2, 46914.448, wall=77.0, met=False)  # a cut life (until not met) is not a finished life
    lines, why = report(tmp)
    text = "\n".join(lines or [])
    assert why is None and "1 finished" in lines[0] and "life 02  mass" not in text and "life 02 running" in text, (why, text)


def case_stage_from_log(tmp):
    life(tmp, "m_up_sb", 1, 3351.032)
    running(tmp, "m_up_sb", 2)
    log = os.path.join(tmp, "out.log")

    def write(*lines):
        with open(log, "w", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")

    old = "SR-CALIBRATE progress item=m_up_sb mass=3351.032 step=10000 sim_time=80.5 stage=gas"
    new = "SR-CALIBRATE progress item=m_up_sb mass=46914.448 step=20000 sim_time=161.25 stage=hydrogen burning"
    write(old, "SR-CALIBRATE run item=m_up_sb side=Light mass=3351.032", new, "SR-CALIBRATE progress item=m_ign_sb mass=1.000 step=5 sim_time=1.0 stage=other item")
    lines, why = report(tmp, log=log)
    assert why is None and "stage hydrogen burning, step 20000, sim_time 161.25 (from the log)" in "\n".join(lines), (why, lines)
    # A run line after the last progress line: that life ended, the running one has not reported.
    write(new, "SR-CALIBRATE run item=m_up_sb side=Heavy mass=46914.448")
    lines, why = report(tmp, log=log)
    assert why is None and "no progress line in the log yet" in "\n".join(lines) and "stage" not in "\n".join(lines), (why, lines)
    lines, why = report(tmp, log=os.path.join(tmp, "missing.log"))
    assert lines is None and "--log" in why, (lines, why)
    lines, why = report(tmp)  # without --log the stage is not claimed
    assert why is None and "stage" not in "\n".join(lines), (why, lines)


def case_bracket_unknown(tmp):
    # HI is life 02's once it has finished; before that --bracket, else the item's default, named.
    life(tmp, "m_up_sb", 1, 3351.032)
    lines, why = report(tmp)
    assert why is None and "lives left 9 of 10 · bracket from the item's default" in "\n".join(lines), (why, lines)
    lines, why = report(tmp, bracket=(3351.032, 3351.032 * 200.0))  # ln 200 / ln 1.02 = 267.5 → 9 halvings → 11 lives
    assert why is None and "lives left 10 of 11 · bracket from --bracket" in "\n".join(lines), (why, lines)
    life(tmp, "m_up_sb", 2, 3351.032 * 1.5)  # a finished life 02 beats the bracket asked for
    lines, why = report(tmp, bracket=(3351.032, 3351.032 * 200.0))
    assert why is None and "bracket from lives 1 and 2" in "\n".join(lines), (why, lines)


def case_bh_no_default_lo(tmp):
    running(tmp, "m_bh_sb", 1)
    lines, why = report(tmp)
    assert lines is None and "m_bh_sb" in why and "--bracket" in why, (lines, why)
    lines, why = report(tmp, bracket=(4000.0, 67020.0))
    assert why is None and "lives left 9 of 9" not in "\n".join(lines) and "no life has finished" in "\n".join(lines), (why, lines)


def case_unknown_label(tmp):
    life(tmp, "m_up_sb", 1, 3351.032)
    lines, why = report(tmp, item="m_zzz_sb")
    assert lines is None and "m_zzz_sb" in why, (lines, why)
    empty = os.path.join(tmp, "empty")
    os.makedirs(os.path.join(empty, "notes-x"))
    lines, why = report(empty)
    assert lines is None and "no <item>-<NN> folder" in why, (lines, why)
    lines, why = report(os.path.join(tmp, "nope"))
    assert lines is None and "not a folder" in why, (lines, why)


def case_item_complete(tmp):
    for nn, mass in ((1, 3351.0), (2, 3351.0 * 1.019)):  # a bracket already within 2 % spends two lives
        life(tmp, "m_up_sb", nn, mass, wall=100.0 * nn)
    lines, why = report(tmp)
    assert why is None and "item complete: 2 lives, 5m00s of wall time" in "\n".join(lines), (why, lines)


def case_preset_lives(tmp):
    life(tmp, "m_up_sb", 1, 3351.032)  # another item's folder in the same invocation is not this item's life
    life(tmp, "preset_sun", 2, 3351.032, wall=2220.0)
    life(tmp, "preset_heavy", 3, 20000.0, wall=480.0, ending="supernova")
    lines, why = report(tmp, item="preset_mass_sb")
    text = "\n".join(lines or [])
    assert why is None and "2 finished" in lines[0] and "lives left 1 of 3" in text and "m_up_sb" not in text, (why, text)
    life(tmp, "preset_ns", 4, 9000.0, wall=900.0, ending="black_hole")
    lines, why = report(tmp)  # the default item is the newest folder's: the preset lives
    assert why is None and "item complete: 3 lives" in "\n".join(lines), (why, lines)


def case_progress_json(tmp):
    life(tmp, "m_up_sb", 1, 3351.032, wall=510.0, steps=142801)
    running(tmp, "m_up_sb", 2, started=NOW - 90.0)  # the folder's own mtime is 1m30s old; the life started 10 min ago (a --resume'd folder)
    progress_file(tmp, "m_up_sb", 2, heat_unconverged=1234)
    lines, why = report(tmp)
    text = "\n".join(lines or [])
    assert why is None, why
    assert "life 02 running 10m00s · stage main_sequence, step 60,000, sim_time 480.25 (from progress.json, updated 5s ago)" in text, text
    assert "1m30s" not in text, "the life's age is read from `started`, not the folder's mtime: " + text
    assert "path: cloud › collapsing › [main_sequence] since step 30,000 · heat unconverged in 1,234 steps (counted, not printed)" in text, text
    # 60,000 of life 01's 142,801 steps is 42 %; at 100 steps/s (60,000 over 600 s) 82,801 steps are 13m48s.
    assert "this life: [########------------] 42% of life 01's 142,801 steps (nearest mass) at 100 steps/s · ~13m48s left (a guess)" in text, text
    # The item's ETA takes the running life's own: 8 lives after it at 510 s (one finished life: a guess) plus 13m48s.
    assert "ETA 1h21m" in text and "a guess: 1 of 3 lives needed" in text, text
    assert len(lines) <= 8, lines


def case_progress_fallbacks(tmp):
    life(tmp, "m_up_sb", 1, 3351.032, steps=1000)
    running(tmp, "m_up_sb", 2)
    old = "its folder was made 1m30s ago"
    # Missing, torn, a field of the wrong type and another item's file are each the old line, never a stage.
    for text in (None, '{"item": "m_up_sb", "mass": 46914.4, "step": 600', json.dumps({"item": "m_up_sb", "mass": "x", "step": 1}), ""):
        if text is not None:
            progress_file(tmp, "m_up_sb", 2, text=text)
        lines, why = report(tmp)
        assert why is None and old in "\n".join(lines) and "progress.json" not in "\n".join(lines) and "path:" not in "\n".join(lines), (text, why, lines)
    progress_file(tmp, "m_up_sb", 2, item="m_ign_sb")
    lines, why = report(tmp)
    assert why is None and old in "\n".join(lines), (why, lines)
    # A finished life's progress.json (summary.json beside it) is not a running life: nothing is read for it.
    progress_file(tmp, "m_up_sb", 1)
    lines, why = report(tmp)
    assert why is None and "stage" not in "\n".join(lines).replace("stages", ""), (why, lines)
    # A file that nothing has updated for 2 h names itself as stale; the life it names still reads.
    progress_file(tmp, "m_up_sb", 2, updated=NOW - 7200.0)
    lines, why = report(tmp)
    assert why is None and "updated 2h00m ago — stale, is it stopped?" in "\n".join(lines), (why, lines)
    # --log still wins for the stage when it holds a line of the running life.
    log = os.path.join(tmp, "out.log")
    with open(log, "w", encoding="utf-8") as f:
        f.write("SR-CALIBRATE progress item=m_up_sb mass=46914.448 step=70000 sim_time=500.5 heat_unconverged=7 stage=h_shell\n")
    progress_file(tmp, "m_up_sb", 2)
    lines, why = report(tmp, log=log)
    text = "\n".join(lines or [])
    assert why is None and "stage h_shell, step 70000, sim_time 500.5 (from the log)" in text, (why, text)


def case_progress_life_size(tmp):
    running(tmp, "m_up_sb", 1)
    progress_file(tmp, "m_up_sb", 1, stages=[], stage="cloud")  # no history in the file: the trail is the stage alone
    lines, why = report(tmp, bracket=(3351.032, 46914.448))
    text = "\n".join(lines or [])
    assert why is None and "path: [cloud]" in text and "no finished life to size it against" in text and "at 100 steps/s" in text, (why, text)
    life(tmp, "m_up_sb", 1, 3351.032, steps=50000)  # the life now finished is life 01; life 02 runs and has gone past it
    running(tmp, "m_up_sb", 2)
    progress_file(tmp, "m_up_sb", 2)
    lines, why = report(tmp)
    text = "\n".join(lines or [])
    assert why is None and "step 60,000, past life 01's 50,000 steps (nearest mass) at 100 steps/s · no ETA for it" in text, (why, text)
    # The nearest life by mass sizes it: 14 M0 is nearer to life 03's 46,914 than to life 01's 3,351.
    life(tmp, "m_up_sb", 3, 46914.448, steps=200000)
    running(tmp, "m_up_sb", 4)
    progress_file(tmp, "m_up_sb", 4)
    lines, why = report(tmp)
    assert why is None and "30% of life 03's 200,000 steps" in "\n".join(lines), (why, lines)


def case_stale_follows_the_tick(tmp):
    """M0-D75: a file is stale past max(15 min, 3 × the tick's expected interval); the interval is `progress_every` over the life's
    pace from `started` to the file's last write. Here the life wrote its first tick (step 10,000) 16 min after it started."""
    life(tmp, "m_up_sb", 1, 3351.032, steps=1000)
    running(tmp, "m_up_sb", 2)
    ran = 960.0  # 10,000 steps in 16 min: the expected interval is 960 s, three of them 48 min
    for age, stale in ((1200.0, False), (2700.0, False), (3000.0, True)):
        progress_file(tmp, "m_up_sb", 2, updated=NOW - age, step=10000, progress_every=10000, started=started_at(age + ran))
        lines, why = report(tmp)
        text = "\n".join(lines or [])
        assert why is None and "from progress.json, updated" in text, (why, text)
        assert ("stale, is it stopped?" in text) == stale, (age, text)
    # A file from before `progress_every` says none: the default's 10,000-step tick is read, the same 16 min interval.
    progress_file(tmp, "m_up_sb", 2, updated=NOW - 1200.0, step=10000, started=started_at(1200.0 + ran))
    lines, why = report(tmp)
    assert why is None and "stale, is it stopped?" not in "\n".join(lines), (why, lines)
    # A fast tick (1,000 steps at 625 steps/s: 1.6 s) keeps the 15 min floor: 20 min old is stale.
    progress_file(tmp, "m_up_sb", 2, updated=NOW - 1200.0, step=600000, progress_every=1000, started=started_at(1200.0 + ran))
    lines, why = report(tmp)
    assert why is None and "stale, is it stopped?" in "\n".join(lines), (why, lines)


def case_continued_life(tmp):
    """M0-D77: a life continued from a checkpoint is paced on this process's steps, says what a stop would lose, and goes stale on
    its own tick — never on the steps it did not run."""
    life(tmp, "m_up_sb", 1, 3351.032, steps=200000)
    running(tmp, "m_up_sb", 2)
    # 10,000 steps in 100 s since the continued start at step 50,000 (a pace of 100 steps/s, not 600), a checkpoint at 55,000.
    progress_file(tmp, "m_up_sb", 2, step=60000, started=started_at(100.0), continued_from_step=50000, checkpoint_step=55000)
    lines, why = report(tmp)
    text = "\n".join(lines)
    assert why is None and "at 100 steps/s" in text, (why, lines)
    assert "checkpoint at step 55,000: a stop now loses 5,000 steps" in text, lines
    # No checkpoint yet: a stop loses the life so far; a file from before M0-D77 (no key) says nothing of a checkpoint.
    progress_file(tmp, "m_up_sb", 2, step=60000, started=started_at(600.0), continued_from_step=None, checkpoint_step=None)
    text = "\n".join(report(tmp)[0])
    assert "no checkpoint yet: a stop loses the life so far" in text and "at 100 steps/s" in text, text
    progress_file(tmp, "m_up_sb", 2, step=60000, started=started_at(600.0))
    text = "\n".join(report(tmp)[0])
    assert "checkpoint" not in text and "at 100 steps/s" in text, text
    # Stale on its own tick: 10,000 steps over an hour since step 50,000 is a 1 h tick, so a file 2 h old is not stale; the same
    # file read as a life that ran all 60,000 steps (a 10 min tick: stale past 30 min) would be.
    progress_file(tmp, "m_up_sb", 2, updated=NOW - 7200.0, step=60000, progress_every=10000, started=started_at(7200.0 + 3600.0),
                  continued_from_step=50000, checkpoint_step=50000)
    lines, why = report(tmp)
    assert why is None and "stale, is it stopped?" not in "\n".join(lines), (why, lines)
    progress_file(tmp, "m_up_sb", 2, updated=NOW - 7200.0, step=60000, progress_every=10000, started=started_at(7200.0 + 3600.0))
    assert "stale, is it stopped?" in "\n".join(report(tmp)[0])


def case_progress_keys_in_source(_tmp):
    root = os.path.abspath(os.path.join(HERE, ".."))
    with open(os.path.join(root, "crates", "sr-engine", "src", "calibrate.rs"), encoding="utf-8") as f:
        text = f.read()
    record = re.search(r"let record = json!\(\{(.*?)\}\);", text)
    assert record, "calibrate.rs writes progress.json from one json! record"
    keys = re.findall(r'"(\w+)":', record[1])
    assert keys == ["item", "mass", "step", "sim_time", "stage", "stages", "heat_unconverged", "started", "progress_every", "continued_from_step", "checkpoint_step"], keys
    assert 'json!({"stage": stage, "since_step": step})' in text and 'out.join("progress.json")' in text and "std::fs::rename(&tmp, path)" in text
    assert "SR-CALIBRATE progress item={label} mass={mass:.3} step={step} sim_time={sim_time:.3} heat_unconverged={unconverged} stage={stage}" in text
    # The status script's progress regex reads that very line, with and without the count.
    for line in ("SR-CALIBRATE progress item=m_up_sb mass=1.000 step=10000 sim_time=80.500 heat_unconverged=3 stage=main_sequence",
                 "SR-CALIBRATE progress item=m_up_sb mass=1.000 step=10000 sim_time=80.500 stage=main_sequence",
                 # M0-D80: the frame's, the world's and the heaviest object's numbers after the stage.
                 "SR-CALIBRATE progress item=m_up_sb mass=1.000 step=10000 sim_time=80.500 heat_unconverged=3 stage=main_sequence | frame=1 dt=0.0123"
                 " s_per_step=2.5 p5_ceiling=1 | escaped=0 objects=1 | M=3351 R=12.5 raw=core_collapse"):
        m = cs.PROGRESS.search(line)
        assert m and m["stage"] == "main_sequence" and m["step"] == "10000", line


def case_bisect_count(_tmp):
    brackets = [(0.05, 1.0), (1.0, 14.0), (1.0, 20.0), (100.0, 20000.0), (167.552, 3351.032), (5000.0, 5099.0), (1.0, 1.019)]
    assert len(brackets) >= 6
    for lo, hi in brackets:
        assert cs.bisect_lives(lo, hi) == port_runs(lo, hi), (lo, hi, cs.bisect_lives(lo, hi), port_runs(lo, hi))
    # On the tolerance's edge the loop (calibrate.rs's `while hi / lo - 1.0 > tolerance`) is the count, not the closed form.
    assert 5100.0 / 5000.0 - 1.0 > 0.02 and cs.bisect_lives(5000.0, 5100.0) == 3
    assert cs.bisect_lives(100.0, 20000.0) == 2 + math.ceil(math.log2(math.log(200) / math.log(1.02))) == 11
    # The shipped ratios of the three items: m_ign 10 lives (the lead's real invocation), m_up 10, m_bh from 1.25 m_up.
    assert cs.bisect_lives(0.05 * cs.UNIT_MASS, cs.UNIT_MASS) == 10 and cs.bisect_lives(cs.UNIT_MASS, 14 * cs.UNIT_MASS) == 10


def case_unit_mass_in_source(_tmp):
    root = os.path.abspath(os.path.join(HERE, ".."))
    with open(os.path.join(root, "crates", "sr-engine", "src", "scene.rs"), encoding="utf-8") as f:
        text = f.read()
    sigma = float(re.search(r"const SUN_SIGMA_C: f64 = ([0-9.]+);", text)[1])
    radius = float(re.search(r"const SUN_RADIUS: f64 = ([0-9.]+);", text)[1])
    assert "2.0 * std::f64::consts::PI * SUN_SIGMA_C * SUN_RADIUS * SUN_RADIUS / 3.0" in text
    assert abs(2.0 * math.pi * sigma * radius * radius / 3.0 - cs.UNIT_MASS) < 5e-4, (sigma, radius, cs.UNIT_MASS)
    with open(os.path.join(root, "crates", "sr-engine", "src", "calibrate.rs"), encoding="utf-8") as f:
        text = f.read()
    assert "Threshold::Ign => (0.05 * UNIT_MASS, UNIT_MASS)" in text and "Threshold::Up => (UNIT_MASS, 14.0 * UNIT_MASS)" in text
    assert "BISECT_TOLERANCE: f64 = 0.02" in text


def case_read_only_and_shape(tmp):
    life(tmp, "m_up_sb", 1, 3351.032)
    life(tmp, "m_up_sb", 2, 46914.448)
    running(tmp, "m_up_sb", 3)
    progress_file(tmp, "m_up_sb", 3, updated=time_now())
    before = listing(tmp)
    proc = subprocess.run([sys.executable, os.path.join(HERE, "calibrate_status.py"), tmp], capture_output=True, text=True)
    lines = proc.stdout.splitlines()
    assert proc.returncode == 0 and lines[-1] == "=== GO ===" and 3 <= len(lines) <= 9, (proc.returncode, proc.stdout, proc.stderr)
    assert listing(tmp) == before, "the script changed something under the invocation folder"
    proc = subprocess.run([sys.executable, os.path.join(HERE, "calibrate_status.py"), tmp, "--item", "nope"], capture_output=True, text=True)
    assert proc.returncode == 1 and proc.stdout.startswith("=== NO-GO: ") and "nope" in proc.stdout, (proc.returncode, proc.stdout)
    assert listing(tmp) == before


CASES = [case_finished_lives, case_running_life, case_stage_from_log, case_torn_and_cut, case_bracket_unknown, case_bh_no_default_lo, case_unknown_label,
         case_progress_json, case_progress_fallbacks, case_progress_life_size, case_stale_follows_the_tick, case_continued_life, case_progress_keys_in_source,
         case_item_complete, case_preset_lives, case_bisect_count, case_unit_mass_in_source, case_read_only_and_shape]


def main():
    passed, failed = 0, []
    for case in CASES:
        with tempfile.TemporaryDirectory() as tmp:
            try:
                case(tmp)
                passed += 1
            except Exception as e:  # a case's assert, or the script raising where it should not
                failed.append("%s: %s: %s" % (case.__name__, type(e).__name__, e))
    for line in failed:
        print("FAIL " + line)
    print("%d passed, %d failed" % (passed, len(failed)))
    if failed or passed < len(CASES):
        print("=== NO-GO: %d case(s) failed ===" % len(failed))
        return 1
    print("=== GO ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
