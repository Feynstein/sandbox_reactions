# Review remarks — review2

Written by `tools/pb/review_page.py` (PLAYBOOK annex §A.4). The table between the two markers is tool-owned (rewritten on every save under `serve`); write the triage and the `Review verdict:` line outside them.

<!-- review_page:remarks:begin -->
remarks 0 · flagged captures 3 · not in this capture set 0

| capture | route / state | remark (verbatim) | time |
|---|---|---|---|
| `c02-hold` | desktop · clear-dialog | _no remark_ | — |
| `m15` | desktop · collapse-banner | _no remark_ | — |
| `s03-star-x1` | desktop · star-x1 | _no remark_ | — |
<!-- review_page:remarks:end -->

## Triage (M0-TV2, 2026-10-30, one pass)
No remark, by the lead's choice: «Look at the screenshots yourself, file the necessary tasks yourself. I dont want to be involved in this
task. Complete it yourself.» (the lead at M0-TV2's launch, 2026-10-29). The page was built and served at 127.0.0.1:8765
(`/health` ready, 3 captures), then stopped; the three flagged captures are the evidence of the D's filed, each noted "for the record".
The agent's own look at all 45 captures (images/tv2/final/captions.json — each with its pass / filed / flagged reason), measured on
laserax-ai-server's lavapipe on a private Xvfb (R18, the lead: «rule everything GPU on the 4090 server and never here»; the RTX 4090
refuses to present to Xvfb, exit 3) filed three D's:
- `s03-star-x1`, `c02-hold` → **M0-D119** — the bar's and the dialog's widgets in egui's default theme (#005C80, #3C3C3C, #1B1B1B), not §4.11
- `m15` → **M0-D120** — the pass that reads a slow-down event draws the new banner beside the old rung (bar one pass stale)
- `s03-star-x1` → **M0-D121** — a round star grows square (M0-V11's O3, now filed), Phase 16's head
Passed with their reason, not filed: `status.line`'s «1 steps per frame» (§4.1 byte-exact, a contract string); the absent left and
right panels (§4.2 — M0-T89's and M0-T92's, after M0-V12).
Flagged for M0-V12's launcher run (lavapipe cannot prove them): the glow; the capped readouts' numbers (×0.075, ×0.072, ×0.067 are
lavapipe's); the supernova banner — not reached within the massive capture's 590 s bound (~2,520 steps, the core-collapse banner held
at ×0.1 from ~1,520), and whether §1.8.5 shows a new banner when the rung already sits at ×0.1 is not measured.

Review verdict: three D's filed (M0-D119, M0-D120, M0-D121), no remark — Phase 14's states all captured but the supernova banner
(NOT PROVEN, flagged to M0-V12). Motion, timing, frame rate and input feel: NOT PROVEN (static capture).
