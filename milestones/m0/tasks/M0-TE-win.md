# M0-TE-win — detail (win-laptop Laser2025-20, 2026-10-08; model=claude-sonnet-5-5 level=high per `rung_record.py now`)

Full evidence: reports/win_laptop.md (Box · Installed · Toolkit · Lot 1 on win-laptop · Hazards); trace: logs/M0-TE-win.log,
logs/M0-TE-win.tools_selftest.log, logs/M0-TE-win.toolchain.log, logs/M0-TE-win.toolchain.redarm.log.

## Verify
1. `python3 --version` → Python 3.13.14 (the Store package's alias, real); `plan.py lint` GO (491 checks) — **PASS**.
2. `verify.py --all` on win-laptop — plan_lint GO 479, toolchain GO 3, tools_selftest NO-GO 6/4 (10 cases = linux-pc's) — **FAIL** → BLOCKED,
   each red named in § Toolkit and routed: M0-D2 (rung_record), M0-D3 (content_gate), M0-D4 (plan), M0-D5 (scratch_copy).
3. `verify.py --redarm toolchain` → clean GO, 3 plants / 3 red — **PASS**.
4. `grep -cE "^## (Box|Installed|Toolkit|Lot 1 on win-laptop|Hazards)" reports/win_laptop.md` → 5 — **PASS**.
Claim run: `verify.py --changed --base 3be0d03f77c2a183a72e7d7aa64506e9293e8b3b` — plan_lint GO, tools_selftest NO-GO (the same four, plus one
non-reproduced 0xC0000005 in `verify`'s case).

## Commands for the lead
[NOT RUN — for you] Restart VS Code (or open a new Claude Code session) so shells see the new user variable `MSYS=winsymlinks:nativestrict`
and `C:\Users\YohanBelanger\bin` (rsync); a shell opened before the install does not.

## Declared defaults (cheap to reverse)
- Node 26.8.2 kept (table: Node 20; ≥ 20 is what playwright-core 1.64.0 needs); Python 3.13.14 kept as `python3` (table: 3.12; Deliver: ≥ 3.12).
- No PATH change for `bash` (WSL's relay shadows Git's from PowerShell): toolkit calls go through the Bash tool.
- Rust upgraded by `rustup update stable` (1.97.1 → 1.99.0), the table's version; nothing downgraded or removed.
- D2–D5 placed "(AFTER M0-TE-win, BEFORE M0-TJ3)" so the lead's ruling can box-bind `tools_selftest` and retire them as N/A.

## Not filed, and why
- `tools_selftest[verify]` exit 3221225477 (0xC0000005), once in ~8 runs, GO in the other seven: not reproducible → a D waits for a second sighting.
- Flow: I edited the D blocks' wording once with a short Python replace over the plan after `plan.py append` (to clear lint WARNs of repeated lines);
  `plan.py` has no call for that. Content equals what an Edit would have produced; lint GO after.
