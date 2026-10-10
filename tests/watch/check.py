#!/usr/bin/env python3
"""G-WATCH — labels watch, never drive (m0_contrat.md §1.1, §5.4). Standard library only.

  python3 tests/watch/check.py          scan the step code and the shaders

No file under crates/sr-engine/src/step/ or crates/sr-engine/shaders/ may name `observe`, `Stage` or `Tracker`
(the labels: sr_physics::observe) or a §2.11 calibration key other than `physics_hash`. A name is a whole identifier,
comments and strings included: the step reads state, never a label and never a measured constant.

The calibration keys are read from the contract's §2.11 "Schema:" line, never listed here: the top-level keys of its
JSON, less the provenance keys (`version`, `measured`, which carry no value) and `physics_hash`, the one key the
engine may know (G-CAL). A scan that finds fewer than MIN_FILES files, or fewer than MIN_KEYS keys, is NO-GO — an empty
folder or an unreadable schema would be green for nothing. Ends in `=== GO ===` or `=== NO-GO: … ===`.
"""
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CONTRACT = os.path.join(ROOT, "milestones", "m0", "m0_contrat.md")
FOLDERS = ("crates/sr-engine/src/step", "crates/sr-engine/shaders")
LABELS = ("observe", "Stage", "Tracker")
PROVENANCE = {"version", "measured"}  # §2.11's bookkeeping keys: no calibration value behind them
ALLOWED_KEY = "physics_hash"
MIN_FILES = 2  # step/mod.rs and shaders/floors.wgsl today
MIN_KEYS = 10  # §2.11's schema names 13 value keys today; far fewer means the line was not read


def calibration_keys():
    """The top-level keys of §2.11's schema line (`- **Schema:** `{ … }`), in order."""
    with open(CONTRACT, encoding="utf-8") as f:
        text = f.read()
    start = text.find("### 2.11 ")
    if start < 0:
        raise SystemExit("=== NO-GO: contract §2.11 not found ===")
    end = text.find("\n### ", start + 1)
    section = text[start:end if end > 0 else len(text)]
    m = re.search(r"\*\*Schema:\*\*(.*?)\n- \*\*Procedure", section, re.S)
    if not m:
        raise SystemExit("=== NO-GO: contract §2.11 has no Schema line ===")
    body = m.group(1)
    # Walk the braces: a key is a quoted word at depth 1; deeper words are sub-keys.
    keys, depth, i = [], 0, 0
    while i < len(body):
        c = body[i]
        if c in "{[":
            depth += 1
        elif c in "}]":
            depth -= 1
        elif c == '"':
            j = body.index('"', i + 1)
            word = body[i + 1:j]
            before = body[:i].rstrip()
            # The schema lists most keys bare ("m_ch_sb",), a few with a value ("version": 1): a quoted word at depth 1
            # is a key unless a colon sits just before it (then it is that key's string value).
            if depth == 1 and not before.endswith(":"):
                keys.append(word)
            i = j
        i += 1
    return [k for k in keys if k not in PROVENANCE and k != ALLOWED_KEY]


def forbidden(keys):
    words = list(LABELS) + list(keys)
    return {w: re.compile(r"(?<![A-Za-z0-9_])" + re.escape(w) + r"(?![A-Za-z0-9_])") for w in words}


def files_under(folders):
    out = []
    for folder in folders:
        base = os.path.join(ROOT, *folder.split("/"))
        for dirpath, _, names in os.walk(base):
            for n in sorted(names):
                out.append(os.path.join(dirpath, n))
    return sorted(out)


def main():
    keys = calibration_keys()
    if len(keys) < MIN_KEYS:
        print(f"calibration keys read from §2.11: {len(keys)}")
        print(f"=== NO-GO: only {len(keys)} calibration keys read from §2.11's schema line (expected ≥ {MIN_KEYS}) ===")
        return 1
    pats = forbidden(keys)
    files = files_under(FOLDERS)
    hits = []
    for path in files:
        rel = os.path.relpath(path, ROOT).replace(os.sep, "/")
        with open(path, encoding="utf-8", errors="replace") as f:
            for n, line in enumerate(f, 1):
                for word, pat in pats.items():
                    if pat.search(line):
                        hits.append(f"{rel}:{n}: names `{word}`")
    print(f"files scanned: {len(files)} in {', '.join(FOLDERS)}; words forbidden: {len(pats)} "
          f"({len(LABELS)} labels + {len(keys)} calibration keys)")
    if len(files) < MIN_FILES:
        print(f"=== NO-GO: {len(files)} files scanned (expected ≥ {MIN_FILES}): the scan saw an empty folder ===")
        return 1
    if hits:
        for h in hits:
            print(h)
        print(f"=== NO-GO: {len(hits)} name(s) a label or a calibration value in the step code ===")
        return 1
    print(f"1 passed, 0 failed ({len(files)} files)")
    print("=== GO ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
