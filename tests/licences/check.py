#!/usr/bin/env python3
"""G-LIC — no copyleft (m0_contrat.md §5.4, R7). Standard library only.

  python3 tests/licences/check.py cargo-native    cargo metadata, the host target
  python3 tests/licences/check.py cargo-wasm32    cargo metadata, wasm32-unknown-unknown
  python3 tests/licences/check.py npm             npm ls --all --json in web/ (web/package.json, M0-T15; npm ci first when node_modules is absent)
  python3 tests/licences/check.py selftest        the expression parser against fixed expressions

Every package the target resolves to needs a licence expression with an alternative made only of the G-LIC list.
OR picks one side, AND needs both, WITH binds an exception to its licence (only the listed pair passes), `/` reads as
OR. A missing licence, an unparsable expression or any other licence is NO-GO and names the package. A named
exception (EXCEPTIONS) passes one package with exactly one expression. A workspace member (a path crate of ours)
that declares no licence is our own unlicensed code (publish = false): counted, not refused; one that declares a
licence is held to the list like any other. Ends in `=== GO ===` or `=== NO-GO: … ===`.
"""
import json
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

# §5.4's list, lower-cased for comparison (SPDX ids are case-insensitive).
ALLOWED = {x.lower() for x in (
    "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "Unicode-3.0", "Unicode-DFS-2016",
    "BSL-1.0", "CC0-1.0", "MIT-0", "0BSD", "Unlicense")}
ALLOWED_WITH = {("apache-2.0", "llvm-exception")}
# Named exceptions (the lead, M0-T14, 2026-10-10: "Named exception"): a package accepted with exactly this licence
# expression and no other. eframe's `default_fonts` pulls epaint_default_fonts, which bundles font files (OFL-1.1, the
# Ubuntu Font Licence) — data, not GPL-family code. A changed expression, or another package, is refused again.
EXCEPTIONS = {("epaint_default_fonts", "(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0")}


class Bad(Exception):
    pass


def tokenize(expr):
    # `/` is the old OR; ids are letters, digits, `.`, `-`, `+`.
    toks = re.findall(r"\(|\)|/|[A-Za-z0-9.+_-]+|\S", expr)
    out = []
    for t in toks:
        u = t.upper()
        if t in "()":
            out.append(t)
        elif t == "/":
            out.append("OR")
        elif u in ("AND", "OR", "WITH"):
            out.append(u)
        elif re.fullmatch(r"[A-Za-z0-9.+_-]+", t):
            out.append(t)
        else:
            raise Bad(f"unexpected {t!r}")
    return out


def accepts(expr):
    """True when the expression has an alternative made only of allowed licences; Bad when it does not parse."""
    toks = tokenize(expr)
    pos = 0

    def peek():
        return toks[pos] if pos < len(toks) else None

    def take():
        nonlocal pos
        pos += 1
        return toks[pos - 1]

    def or_expr():
        v = and_expr()
        while peek() == "OR":
            take()
            r = and_expr()  # both sides parsed, always
            v = v or r
        return v

    def and_expr():
        v = with_expr()
        while peek() == "AND":
            take()
            r = with_expr()
            v = v and r
        return v

    def with_expr():
        if peek() == "(":
            take()
            v = or_expr()
            if peek() != ")":
                raise Bad("missing )")
            take()
            return v
        t = peek()
        if t is None or t in ("AND", "OR", "WITH", ")"):
            raise Bad(f"expected a licence id, got {t!r}")
        lic = take().lower()
        if peek() == "WITH":
            take()
            t = peek()
            if t is None or t in ("AND", "OR", "WITH", "(", ")"):
                raise Bad("expected an exception id after WITH")
            return (lic, take().lower()) in ALLOWED_WITH
        return lic in ALLOWED

    if not toks:
        raise Bad("empty expression")
    v = or_expr()
    if pos != len(toks):
        raise Bad(f"unexpected {toks[pos]!r}")
    return v


def judge(pkgs, log=False):
    """pkgs: (name, version, licence-or-None, local). Returns the list of refusals as strings."""
    bad = []
    for name, ver, lic, local in pkgs:
        if lic is None or not str(lic).strip():
            if not local:
                bad.append(f"{name} {ver}: no licence")
            continue
        if (name, str(lic).strip()) in EXCEPTIONS:
            if log:
                print(f"EXCEPTION {name} {ver}: {lic} (named, M0-T14)")
            continue
        try:
            ok = accepts(str(lic))
        except Bad as e:
            bad.append(f"{name} {ver}: unreadable licence {lic!r} ({e})")
            continue
        if not ok:
            bad.append(f"{name} {ver}: licence {lic!r} has no alternative made only of the G-LIC list")
    return bad


def selftest():
    yes = ["MIT", "MIT OR Apache-2.0", "MIT/Apache-2.0", "Apache-2.0/MIT", "Zlib OR Apache-2.0 OR MIT",
           "Apache-2.0 WITH LLVM-exception", "Apache-2.0 WITH LLVM-exception OR MIT",
           "(MIT OR Apache-2.0) AND Unicode-3.0", "MIT AND (Apache-2.0 OR GPL-3.0-only)",
           "Apache-2.0 OR LGPL-2.1-or-later OR MIT", "mit or apache-2.0", "0BSD", "Unicode-DFS-2016", "MIT-0"]
    no = ["MIT AND GPL-3.0", "MIT AND GPL-3.0-only", "GPL-3.0-only", "LGPL-2.1-or-later", "AGPL-3.0", "MPL-2.0",
          "GPL-3.0 OR MPL-2.0", "GPL-2.0-only WITH Classpath-exception-2.0", "MIT WITH LLVM-exception",
          "Apache-2.0 WITH GCC-exception-3.1", "(MIT OR GPL-3.0) AND MPL-2.0", "MIT AND (GPL-3.0 OR MPL-2.0)",
          "GPL-2.0+", "CC-BY-4.0"]
    bad_syntax = ["", "MIT AND", "(MIT", "MIT)", "MIT OR OR Apache-2.0", "MIT WITH", "AND MIT", "MIT Apache-2.0", "MIT; GPL"]
    problems = []
    for e in yes:
        if accepts(e) is not True:
            problems.append(f"{e!r} should be accepted")
    for e in no:
        if accepts(e) is not False:
            problems.append(f"{e!r} should be refused")
    for e in bad_syntax:
        try:
            accepts(e)
            problems.append(f"{e!r} should not parse")
        except Bad:
            pass
    # judge(): a missing licence is refused, except on our own unlicensed workspace member.
    got = judge([("a", "1", None, False), ("b", "1", None, True), ("c", "1", "MIT", False), ("d", "1", "GPL-3.0-only", True)])
    if [g.split(":")[0] for g in got] != ["a 1", "d 1"]:
        problems.append(f"judge() refused {got!r}")
    fonts = "(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0"
    got = judge([("epaint_default_fonts", "1", fonts, False), ("epaint_default_fonts", "1", fonts + " AND X", False),
                 ("other_fonts", "1", fonts, False)])
    if [g.split(":")[0] for g in got] != ["epaint_default_fonts 1", "other_fonts 1"]:
        problems.append(f"the named exception read {got!r}")
    # the npm walk, on a fixture.
    tree = {"dependencies": {"x": {"version": "1.0.0", "dependencies": {"y": {"version": "2.0.0"}}}, "z": {"version": "3.0.0"}}}
    if sorted(walk_npm(tree)) != [("x", "1.0.0", "x"), ("y", "2.0.0", "x/node_modules/y"), ("z", "3.0.0", "z")]:
        problems.append(f"walk_npm() read {sorted(walk_npm(tree))!r}")
    return problems


def cargo_env():
    env = dict(os.environ)
    env["PATH"] = os.path.join(os.path.expanduser("~"), ".cargo", "bin") + os.pathsep + env.get("PATH", "")
    env.pop("CARGO_TARGET_DIR", None)
    return env


def host_triple(env):
    out = subprocess.run(["rustc", "-vV"], cwd=ROOT, env=env, capture_output=True, text=True, check=True).stdout
    m = re.search(r"^host:\s*(\S+)", out, re.M)
    if not m:
        raise RuntimeError("rustc -vV names no host")
    return m.group(1)


def cargo_case(case):
    env = cargo_env()
    if shutil.which("cargo", path=env["PATH"]) is None:
        return None, "cargo not found (is Rust installed? ~/.cargo/bin is put first here)"
    target = "wasm32-unknown-unknown" if case == "cargo-wasm32" else host_triple(env)
    r = subprocess.run(["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", target],
                       cwd=ROOT, env=env, capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0:
        tail = "\n".join(r.stderr.strip().splitlines()[-6:])
        return None, f"cargo metadata --locked --filter-platform {target} failed: {tail}"
    meta = json.loads(r.stdout)
    by_id = {p["id"]: p for p in meta["packages"]}
    # `packages` lists the whole lockfile; `resolve.nodes` only what this target builds.
    ids = [n["id"] for n in meta["resolve"]["nodes"]]
    pkgs = []
    for i in ids:
        p = by_id[i]
        lic = p.get("license") or (f"(license-file {p['license_file']})" if p.get("license_file") else None)
        pkgs.append((p["name"], p["version"], lic, p.get("source") is None))
    return (target, pkgs), None


def walk_npm(tree, parent=""):
    """(name, version, path under node_modules) for every node of `npm ls --all --json`, nested duplicates included."""
    for name, node in (tree.get("dependencies") or {}).items():
        path = f"{parent}/node_modules/{name}" if parent else name
        yield name, node.get("version", "?"), path
        yield from walk_npm(node, path)


def npm_licence(web, name, path):
    cands = [os.path.join(web, "node_modules", *path.split("/")), os.path.join(web, "node_modules", name)]
    for c in cands:
        f = os.path.join(c, "package.json")
        if os.path.isfile(f):
            with open(f, encoding="utf-8") as fh:
                j = json.load(fh)
            lic = j.get("license")
            if isinstance(lic, dict):
                lic = lic.get("type")
            if lic is None and isinstance(j.get("licenses"), list):
                lic = " OR ".join(x.get("type", "") if isinstance(x, dict) else str(x) for x in j["licenses"])
            return lic
    return None


def npm_case():
    web = os.path.join(ROOT, "web")
    if not os.path.isfile(os.path.join(web, "package.json")):
        return None, "web/package.json does not exist yet (M0-T15 adds it)"
    npm = shutil.which("npm")
    if npm is None:
        return None, "npm not found"
    if not os.path.isdir(os.path.join(web, "node_modules")):   # a red-arm scratch copy leaves node_modules out by name
        i = subprocess.run([npm, "ci", "--ignore-scripts", "--no-audit", "--no-fund", "--prefer-offline"], cwd=web,
                           capture_output=True, text=True, encoding="utf-8")
        if i.returncode != 0:
            return None, f"npm ci failed (exit {i.returncode}): {(i.stderr or i.stdout).strip()[-300:]}"
    r = subprocess.run([npm, "ls", "--all", "--json"], cwd=web, capture_output=True, text=True, encoding="utf-8")
    try:
        tree = json.loads(r.stdout)
    except ValueError:
        return None, f"npm ls --all --json printed no JSON (exit {r.returncode}): {r.stderr.strip()[-300:]}"
    seen, pkgs = set(), []
    for name, ver, path in walk_npm(tree):
        if (name, ver) in seen:
            continue
        seen.add((name, ver))
        pkgs.append((name, ver, npm_licence(web, name, path), False))
    return ("npm", pkgs), None


def main(argv):
    mode = argv[1] if len(argv) > 1 else ""
    if mode not in ("cargo-native", "cargo-wasm32", "npm", "selftest"):
        print("=== NO-GO: usage: check.py cargo-native|cargo-wasm32|npm|selftest ===")
        return 1
    problems = selftest()
    if problems:
        for p in problems:
            print("selftest:", p)
        print(f"=== NO-GO: the expression parser's own checks failed ({len(problems)}) ===")
        return 1
    if mode == "selftest":
        print("licence expressions: parser checks pass")
        print("=== GO ===")
        return 0
    res, err = npm_case() if mode == "npm" else cargo_case(mode)
    if err:
        print(f"=== NO-GO: {err} ===")
        return 1
    label, pkgs = res
    bad = judge(pkgs, log=True)
    local = sum(1 for p in pkgs if p[3])
    print(f"{mode} ({label}): {len(pkgs)} packages ({local} workspace members), {len(bad)} refused")
    for b in bad:
        print("REFUSED", b)
    if bad:
        names = ", ".join(b.split(":")[0] for b in bad[:5]) + (" ..." if len(bad) > 5 else "")
        print(f"=== NO-GO: {len(bad)} package(s) outside the G-LIC list: {names} ===")
        return 1
    print("=== GO ===")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
