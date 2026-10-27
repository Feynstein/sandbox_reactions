# M0-TJ5 — task notes (2026-10-27, linux-pc laserax-ai)

The ruling's record is its own surface: R20 (m0_rules.md § R20), contract §0.5 [M0-TJ5] and §6.2.5, the four [M0-TJ5]
lines at the end of m0_superseded.md (the negative scope the last), M0-TZ's flag [M0-TJ5], the tracker's item 4 and its
2026-10-27 delta. This file holds what the session ran, asked and edited, and the Adversarial's hit list.

## Start
- `grep -n "^## M0-TJ5 "` → line 3466, `· switch` after the rating (Opus 5.5, max).
- `sleep 2; python3 tools/pb/rung_record.py now` → `model=claude-opus-5-5 level=max plugin=installed` — the heading's rung,
  no MISMATCH.
- `python3 tools/pb/plan.py show M0-TJ5` → `baseline: M0-TJ5, 585 files, 2026-10-27T14:57:53-04:00`.
- `../tykun_games`: present (`test -d`), read-only from here — never opened, nothing written there or on `games-box`.

## The question
One call of the question tool, one question: the block's list put in plain words (what goes to `../tykun_games`, rule R20,
what stays, the tracker line), the recommendation first. The alternative offered: keep `[M3-TB1]`'s serving facts (HTTPS, the
wasm type, the gzip files, the shared origin's storage keys `/userfs` and `tykun_lang`) binding until `../tykun_games` sends its
own. The lead: «Confirm, as listed (Recommended)» — `[M3-TB1]` retired whole.

## Edits
- **m0_rules.md** § R20, before «Considered and REJECTED»: the rule verbatim from Deliver 2 (reflowed, words unchanged), the
  lead's four quotes verbatim, the answer, a pointer to what it retired and kept, and my reading of «deploy», labelled.
- **Plan**: Rules row R20; Repo facts' Targets line — «the site and its live deploy are `../tykun_games`'s (R20, M0-TJ5)» (the
  Read line names it; not a Deliver item); `plan.py flag M0-TZ` (the re-routing); `plan.py flag M0-TC9` (below); `plan.py
  register` — Next task, Outstanding commit gates and Awaiting lead's first item (Phase 11's gate measured in, below);
  `plan.py close`.
- **Contract**: §0.5 [M0-TJ5] — §6.2.5's passage · no hosting in this project, the site's · the lead's direction; §6.2.5
  amended in place, tagged [M0-TJ5].
- **m0_superseded.md**, four lines appended (R19): M0-TZ's [M0-R1] and [M0-R2b] asks and [M3-TB1] whole (with the note it
  carried, `../space_tykun`'s `milestones/m3/m3_contrat.md` §2.5); worklist item 4 and web_hosting.md § Routed to; contract
  §6.2.5 and bootstrap.md §3's «a full web release» row (narrowed); the negative scope.
- **milestones.md**: item 4 closed — N/A with its citation, its old text kept after «Was:»; one dated delta.
- Not edited: reports/web_hosting.md (kept as the record; its § Routed to retired by the mirror line, the report untouched),
  reports/bootstrap.md, the other blocks' Carried flags (append-only: the new flags carry the retirement).

## Adversarial — every hit, retired or kept with its reason
The grep: « host », « CDN », « AdSense », « consent », « Lightsail », « deploy », widened to ads, picker, website, web release,
upload, publish, serve, site, live, COOP/COEP, `games-box` and `tykun` — over the contract and the 72 open blocks (status TODO,
IN PROGRESS, BLOCKED or DEFERRED on 2026-10-27, this one excluded).

Contract:
- §6.2.5 «Hosting: none in M0 (reports/web_hosting.md feeds a later milestone)» — **retired**: §0.5 [M0-TJ5], in place.
- §6.2.4 «no browser threads (no SharedArrayBuffer, no COOP/COEP — ES)» — kept: R20 asks the same, for the site's ads too.
- «ghost» (a §0.5 entry's «a ghost's φ», §1.3's P4 row and its margin, §1.7's edges row, §2.8's two modes, §5.1's
  `wall-copy` plant) — «host» inside «ghost cells», not hosting.
- «Hostname Laser2025-20» (§6.6) — a box's name.
- §3.4 «serves `GET /status` on 127.0.0.1 only» and §3.6's `game` row — the desktop's local status port, kept.
- «live» (§0.4 «never live here», a §0.5 entry's «two live sinks»), «uploaded» / «upload» (two §0.5 entries: a state upload
  to the GPU), «published» (§1.11's jcgt.org citation) — other senses.

Open blocks:
- **M0-TZ**'s Carried flags — [M0-R1] (Lightsail, CDN, AdSense, consent), [M0-R2b] (the picker), [M3-TB1] (this project's
  upload over SSH to `ubuntu@games-box`, its « Ready to go up » line, «Place this game's web release — with M0-R1's hosting, CDN,
  ads and consent») — **retired** by M0-TZ's new flag [M0-TJ5] and the mirror lines.
- M0-V11 «an independent published blackbody … table» — a citation, kept.
- M0-T84 «`status.line` live» · M0-TD «can still stutter live» — other senses.
- M0-TV2, M0-TV3 «serve it (tools/pb/review_page.py serve)», «page served» — the lead's local review page, not a deploy (R20's
  reading).
- The web build in open blocks — M0-T94 (web.rs, web/smoke.mjs's `preset` case), M0-T96 (the web build's parameters page),
  M0-TV3 (the web capture), M0-V13 (G-WEB), M0-V17 («the web build kept alive»), M0-V21 (F2's build-wasm-only plant) — kept
  (the negative scope); none deploys.

Reports M0-TZ reads (outside the grep's scope; read because TZ places routed items from them):
- bootstrap.md §3's row « "run on a website" — a full web release | later, beyond M0's web smoke | after M0 » — **narrowed** by
  the mirror line: what TZ places is this game's static web build; its going live is `../tykun_games`'s.
- bootstrap.md's other two hits (the lead's «run on a website» quote; the web build kept alive) — kept. sandbox_interview.md: none.

## Runs (logs/M0-TJ5.log; linux-pc, laserax-ai)
1. Verify 1, before the claim: `plan.py lint` → GO, 708 checks, header 545/600; 2 WARNs — M0-D91's `Read:` (not mine) and
   «`Deliver:` items no Verify step names», M0-TJ5's 10 among them (its Deliver as filed; a closed block's field, left).
2. `verify.py touched` before any claim → NO-GO «no claim record of M0-TJ5 names its files» — expected, step 5 writes it.
3. The claim, `verify.py --changed --task M0-TJ5` → **NO-GO: 723 passed, 1 failed, 3/4 scopes GO, 32.51 s**, base M0-TJ5's
   start. plan_lint (the plan), strings and watch_only (the contract) GO. `dump` was selected by the anchor, not by a path of
   mine («never: .cargo/config.toml (+127)»: its 128 paths hold no record, red since M0-D85) and failed on M0-V9's F1 alone —
   crates/sr-engine/tests/gpu/dump.rs:345 «F's 2 planes after the 14 channels» (logs/M0-TJ5.dump.log) — the known red
   M0-V21 converts. **A miss:** `dump` is a GPU scope and ran on linux-pc's RTX 5090, where R18 runs it on laserax-ai-server's
   RTX 4090 — I had read which scopes my paths reach (verify.json), not the anchor's never-checked ones. Not re-run there:
   F1 is a plane count, already measured on the RTX 4090 by M0-V9, and a re-run says nothing about this change.
4. Step 7, after the post-claim plan edits (two flags, the register): `verify.py plan_lint --task M0-TJ5` → GO, 708 passed.
5. After `plan.py close` (15:25): `plan.py lint` → GO, 707 checks, 2 WARNs not mine (M0-D91's `Read:`; the Deliver items of
   M0-D65 … D91 — M0-TJ5 no longer among them, closed).

## Measured, not acted on
- **milestones.md is stale beyond item 4**: the M0 row reads «next: the Phase 2 commit gate, then M0-TE-win on win-laptop» and
  item 5 the first win-laptop session (both 2026-10-08); the deltas stop at 2026-10-11 before this one. `../tykun_games` reads
  this tracker for « Web build ready » lines. Refreshing it is outside this Deliver (one delta) — the lead's, or M0-TZ's.
- **Every claim selects `dump` until F1 is fixed** (the anchor) and reads NO-GO on it; M0-TC9 is rated below `(usual)`, where
  PLAYBOOK §0's escalation reads any NO-GO claim — flagged to M0-TC9 with the facts, the call left to the lead.
- **Phase 11's commit gate is in**: `bcbd2e5` 'M0 Phase 11: calibration' is HEAD and `origin/main` (read-only git) while the
  register still called it outstanding — the register corrected; R18's push to laserax-ai-server not read from here.
- `refresh=5/5` printed at each `plan.py flag` — M0-TM12 is already queued for it; this task's two flags went to a PLAN and a
  MOVE block.
