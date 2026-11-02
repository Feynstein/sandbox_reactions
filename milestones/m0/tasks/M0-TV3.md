# M0-TV3 — UI/UX pass, Phase 15 (tools, stars and cells)

Box: linux-pc (laserax-ai); model=claude-sonnet-5-5 level=high (rung_record.py now, at launch). 2026-11-02.
Every capture is lavapipe (LLVM 20.1.2) on a private Xvfb (R4): no GPU used, so R18's server route does not apply (the RTX 4090 cannot present to Xvfb, M0-TV2).

## The set — images/tv3/final/ (96 PNGs + captions.json; sources in sun/, massive/, xd/, probe/; scripts/ holds the scripts)
- `sun/` (34, tv3-sun.json, ~2 min): empty world; the Sun-like preset at 1/100/300/1000/2000 steps; each tool; the element list; each tool's edit; the four views on one star with Fit, a typed range, Reset; Glow at 1e8 K; zoom 4/8/1; the inspector on a star, its edge, vacuum, «outside» (a probe: see M0-D137); the star selected.
- `massive/` (26, tv3-massive.json, cut by its 570 s bound after 2,400 steps): readouts at 1/100/300/500/768 (a Giant) steps, the supergiant in every view with each display control and Reset, the legend's cursor, zoom 8; then 900–2,400 steps: Giant, **Supergiant: iron core**, core collapse (banner), **supernova (banner)** — TV2's NOT PROVEN supernova banner is now seen.
- `xd/` (25, scripts/xd.sh, real xdotool input on Xvfb :78): brush sizes 1/14/34/40, each preset armed with its outline and hint, the no-room message, Esc, two stars with the hint, a right click on each, the inspector on a star / vacuum / outside the world (left and right panels), wheel zoom (held at 8), pan, Home, the held brush, the Heat and Cool tools, the E key.
- `probe/` (8): evidence for M0-D137 (scripted hover off the world) and that Heat/Cool change a cell's temperature (713 → 918 → 1,130 K; Cool 765 K, read off the inspector).

## Filed (E1, never fixed here)
M0-D134 star readout stage sequence runs backwards («Failed star» for 1- and 15-Sun presets) · M0-D135 legend title and cursor label overprint · M0-D136 armed hint over the legend's ticks beside the no-room message · M0-D137 capture `hover` off the world keeps the last reading. Seen before, not refiled: M0-D121 (a round star grows square — every star here).
Not filed, flagged for the lead (n1950): the supernova's core reads «10.0 K» and neutron matter «Temperature: 10.0 K» — a model question, not a measured contract breach.

## Not captured, with the reason
- **The remnant (neutron star) at zoom 8, and each view on it (M0-R12's carried flag, second half)**: not reached on lavapipe in 570 s — the run ends at ~2,400 steps, in the supernova stage (≈ 4 steps/s once the star is big); the remnant is later. A visible window on the real GPU (R4, the lead's yes) or M0-V13/M0-TW's window. The supergiant half of the flag is captured (m04–m17).
- **The web build with a star**: `tools/pb`-free route: web/dist served by `python3 -m http.server 47851` (started and stopped by this task), Chrome through playwright-core with smoke.mjs's Linux flags; every URL (`/`, `?preset=sun&steps=200`, `&view=heat`, `?preset=massive&steps=100&view=density`) reached srState `ready` and 4 screenshots (images/tv3/web/) are **blank white** — headless Chrome's screenshot of the WebGPU canvas reads empty (web/smoke.mjs and running.md already say so). NOT PROVEN (no usable capture); not placed in the review set. The lead's own Chrome is the proof (M0-V13's gap).
- Brush-size and zoom *by the egui path* are captured here by real xdotool input (x02–x04, x18–x21), which TV2-era scripts could not do.
- Held-button motion, drag feel, wheel step size, animation, timing, frame rate: NOT PROVEN (static capture).
- Win-laptop: NOT RUN (owed on Laser2025-20, R15).

## Commands (linux-pc)
- `milestones/m0/images/tv3/scripts/run.sh <script.json> <outdir> <timeout>` = `timeout N xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE build/target/release/sandbox-reactions --adapter llvmpipe --capture … --out …`: tv3-sun.json (exit 0, 123 s), tv3-massive.json (570 s bound, exit 124 by design), probe-out.json, probe-heat.json.
- `scripts/xd.sh` (97 s), `scripts/webcap.mjs` (the web set). Logs: logs/M0-TV3.*.log.
- Review page: `review_page.py serve … --port 8766` (8765 was held by a server this task did not start — left alone, never reused).
