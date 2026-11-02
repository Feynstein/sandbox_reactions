# Review remarks — review3

Written by `tools/pb/review_page.py` (PLAYBOOK annex §A.4). The table between the two markers is tool-owned (rewritten on every save under `serve`); write the triage and the `Review verdict:` line outside them.

<!-- review_page:remarks:begin -->
remarks 0 · flagged captures 9 · not in this capture set 0

| capture | route / state | remark (verbatim) | time |
|---|---|---|---|
| `m02-steps300` | desktop · m02-steps300 | _no remark_ | — |
| `m06-glow-1e8` | desktop · m06-glow-1e8 | _no remark_ | — |
| `m09-heat-fit` | desktop · m09-heat-fit | _no remark_ | — |
| `m12-density-fit` | desktop · m12-density-fit | _no remark_ | — |
| `n1950` | desktop · n1950 | _no remark_ | — |
| `p2-outside` | desktop · p2-outside | _no remark_ | — |
| `s04-steps300` | desktop · s04-steps300 | _no remark_ | — |
| `v03-heat-fit` | desktop · v03-heat-fit | _no remark_ | — |
| `x08-no-room` | desktop · x08-no-room | _no remark_ | — |
<!-- review_page:remarks:end -->

## Triage (M0-TV3, 2026-11-02, one pass)
No remark: the lead did not page through the page and answered «alright ill go with your remarks for that one» (2026-11-02) — the agent's own look at all 96 captures stands (images/tv3/final/captions.json, each with its pass / filed / flagged reason; lavapipe on a private Xvfb, linux-pc). The page was served at 127.0.0.1:8766 (`/health` ready, 9 captures), then stopped by SIGTERM (log: logs/M0-TV3.review_serve.log); port 8765 was held by a server this task did not start and was left alone.
Filed: `s04-steps300`, `m02-steps300` → **M0-D134** (stage readout: «Failed star» for 1- and 15-Sun presets, sequence runs backwards) · `v03-heat-fit` → **M0-D135** (legend title and cursor label overprint) · `x08-no-room` → **M0-D136** (armed hint over the legend's ticks) · `p2-outside` → **M0-D137** (capture `hover` off the world keeps the last reading).
Flagged, not filed: `m06-glow-1e8`, `m09-heat-fit`, `m12-density-fit` (M0-R12's carried flag: the supergiant's views toned down — Glow at 1e8 K still one white disk at the core, Heat and Density fitted read as intended, each Reset back to today's look; the remnant half of the flag is NOT REACHED, see tasks/M0-TV3.md) · `n1950` (supernova banner seen; core and neutron matter read 10.0 K — a model question for the lead).
Seen before, not refiled: M0-D121 (a round star grows square — on every star here).

Review verdict: four D's filed (M0-D134–M0-D137), no remark; the web build with a star NOT PROVEN (headless Chrome's canvas screenshot is blank), the remnant at zoom 8 NOT REACHED on lavapipe in 570 s. Motion, timing, frame rate and input feel: NOT PROVEN (static capture).
