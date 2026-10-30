# M0-TV2 — detail (sessions 1 and 2, 2026-10-29/30, linux-pc laserax-ai, model=claude-opus-5-5 level=high)

The lead's words at launch: «Look at the screenshots yourself, file the necessary tasks yourself. I dont want to be
involved in this task. Complete it yourself.» So: no capture is flagged for the lead's remarks. The page step still has
to be built, served and stopped with no remark, and review2.md has to quote these words as the reason there are no
remarks (TV1's review1.md is the shape to follow). The Clear dialog cannot go to the lead's visible window, so it is
captured on the private display or marked NOT PROVEN. Session 1 stopped at the lead's word: «if theres still a lot of
stuff to do write a handoff in your task, put it as in progress».

## Done so far
- Release build current (`cargo build --release --locked -p sr-app`, 0.1 s, nothing to rebuild). Log: logs/M0-TV2.log.
- Probes on a private Xvfb with lavapipe, using check.sh's route:
  `xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE build/target/release/sandbox-reactions --adapter llvmpipe --capture <script> --out <dir>`.
  The scripts and three frames are in images/tv2/probes/ (git-ignored).
  - probe.json: sun + 200 steps, then 1,000 steps → 18.9 s wall. Sun-like steps run at about 65 per second on lavapipe.
  - probe2.json: rung 5 (×10), massive, 20 steps, then 29 × 100 steps. **This ran over 10 minutes** (cut off at my
    600 s timeout, 28 of 30 PNGs written). Massive steps are far slower than Sun steps on lavapipe. Do not re-run it as is:
    reach the banners in about 300–1,600 steps (below).
- Facts measured:
  - TOP = 200 (calibration.json `top_rung`). The bar shows ×0.1 ×0.3 ×1 ×3 ×10 ×30 ×100 Max, so keys 1–7 and 0 (Max).
  - The **ignition banner** renders: «Slowed down: hydrogen ignition — press + to speed up again», over the world's top
    centre, in banner tokens, and the rung drops ×10 → ×1. Between probe2's m2 and m3 (about 220–320 steps) — see
    probes/massive-ignition-banner.png.
  - At m16 (about 1,520–1,620 steps) the rung changes again (the ×0.1 button's selection fill widens). This is most likely
    core collapse → ×0.1 and its banner. Not looked at yet: open probes/massive-m16-rung-x0.1.png first. The supernova
    banner should come soon after it (stage durations: core_collapse 0.10 t.u., supernova 3.2 t.u.).
  - Under a capture `steps N` action, the status line reads «256 steps per frame» (the capture's forced batch), and at ×1
    «1 steps per frame».

## Findings to file as D blocks (E1), not filed yet
1. **The selected rung's fill is not a §4.11 token.** The selected rung's text is `accent` (#F2B33D, measured #F0B33E) on
   egui's default selection fill #005C80, which is not in §4.11, so the amber text sits on teal-blue. Repro:
   probes/sun-1200steps.png, pixel (218, 22). Suspected cause: desktop.rs draws the rung as an egui selectable or
   toggle button and keeps `visuals.selection.bg_fill`. Caused by: M0-T85. Rate it `(usual)`.
2. **A round preset disk draws as a rounded square.** §2.10 drops a Maclaurin **disk**. After 1,200 Sun steps, the bright
   body itself is a rounded square (probes/sun-1200steps.png). In the massive life, the cold haze is a rounded square around
   a round core (probes/massive-ignition-banner.png). Cause not measured: grid anisotropy in hydro or gravity, the deposit,
   or the drawing. Confirm first with a 1-step capture of the drop (is the disk round at step 0?) and a headless
   `--frames-every` run on the real GPU (R18: laserax-ai-server). Grep plan_archive.md and reports/ for an earlier
   finding before filing — session 1's grep was cut off. The rubric gives `(usual)`. File it as a D even if it is physics:
   the capture shows it.
- Not to be filed — passed with this reason: `status.line` reads «1 steps per frame». That string is contract §4.1
  byte-exact, and changing it is a contract change. Note it in review2.md.

## What is left (the Deliver, in order)
1. Write images/tv2/tv2-script.json and run it once on the route above. Keep it Sun-based so it stays under 10 minutes;
   the banners need a separate massive script. States, one PNG each:
   - start (empty, ×1, running)
   - star at ×1, running (sun + about 300 steps)
   - paused
   - single-stepped (steps 1)
   - each rung 1–7 running, a few frames each
   - the capped readout (lavapipe should cap ×30 or ×100: «Speed ×N (running ×M)»)
   - Max (`time.speed_max`)
   - slow-down off
   - edge bounce, then edge leave
   Then a massive script: rung 5, massive, steps 300 → ignition banner; steps to about 1,600 → core collapse banner; then
   more steps → supernova banner. Each `steps` chunk can only be ≤ 300 if the run must stay under 10 minutes: time it.
2. The **Clear dialog**: there is no capture action (M0-T87's flag). Route: start the plain desktop binary on its own
   `Xvfb :NN` with `--adapter llvmpipe`, then `DISPLAY=:NN xdotool key Delete` (Delete opens the dialog, §4.10), then
   `DISPLAY=:NN import -window root out.png` (xdotool, xwd and import are present in /usr/bin). This stays on a private
   display (R4), so no window shows on the lead's screen. If it fails, mark it NOT PROVEN (no capture action, the lead
   declined the window run).
3. Look at every PNG. Write images/tv2/captions.json in review_page.py's shape (§A.4: id, file, route, path, state,
   viewport, overflow, caption, flag, note), with one pass / file / flag reason each. Per the Adversarial line, flag for
   M0-V12's launcher run whatever lavapipe cannot prove: the glow, the capped readout's numbers.
4. File the D's above, plus any new ones: `plan.py append <block.md> --after M0-TV2`.
5. `review_page.py serve --images milestones/m0/images/tv2 --captions …/captions.json --report milestones/m0/reports/review2.md`,
   GET /health, then stop it. Write review2.md's triage and the `Review verdict:` line, with no remark and quoting the
   lead's words above.
6. Claim: `verify.py --changed --task M0-TV2` (the baseline was recorded by `plan.py show` at 15:30:41). Then
   `plan.py close`.

## Session 2 (2026-10-30, model=claude-opus-5-5 level=high, rung_record) — what was done
- The lead: «make sure you follow the rule and rule everything GPU on the 4090 server and never here» — every capture ran on
  laserax-ai-server (R18); none on linux-pc. The server lacked Xvfb/xdotool/ImageMagick: the lead installed them
  (`sudo apt-get install -y xvfb xdotool imagemagick`, the lead's answer «You install them (Recommended)»).
- The RTX 4090 cannot present to Xvfb: `SR-ERROR … adapter NVIDIA GeForce RTX 4090 … cannot present to this window`, exit 3. Every capture
  is lavapipe (LLVM 20.1.2) on the server's private Xvfb. Route, from the server's repo:
  `xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE build/target/release/sandbox-reactions --adapter llvmpipe --capture <script> --out <dir>`.
- Scripts in images/tv2/scripts/ (git-ignored): tv2-sun.json (16 states, 35 s), tv2-massive.json (26 chunks; stopped by its own
  `timeout 590` after m25, ≈ 2,520 steps — about 4 steps/s once the star is big), clear.sh + tv2-clear.json (the Clear dialog).
- **The Clear dialog**: `Xvfb :77`, the capture script holding 200 frames, `xdotool windowfocus` + click + `key Delete`, then
  `import -window root` — the dialog shows in the harness's own c02-hold.png and the root grab. This closes M0-T87's carried flag.
- The capped readout needs N_max binding in ≥ 30 of the last 60 frames (§1.8.4): 8 frames a rung is too few; 40 show it.
- Measured: ignition banner at ≈ 320 steps (m03, ×10 → ×1), core collapse at ≈ 1,520 (m15, ×1 → ×0.1 one pass late → M0-D120),
  no supernova banner by ≈ 2,520 (NOT PROVEN, flagged to M0-V12). Server lavapipe: Sun ≈ 40 steps/s; massive ≈ 15 steps/s early.
- Colours measured (s03, c02): on token — bg.panel, border.panel (y 42), text.primary, bg.space, banner.bg/text; off token —
  #005C80, #3C3C3C, #1B1B1B, #C8C8C8 → M0-D119.
- Final set: images/tv2/final/ (45 PNGs + captions.json, each with its pass / filed / flagged reason).
- **Deviation**: review_page.py refuses a page with nothing flagged («nothing flagged for the lead — no page to show», NO-GO); session 1's
  plan (no flag) cannot serve. Default taken: the three D-evidence captures (s03-star-x1, m15, c02-hold) flagged "for the record", no
  remark asked. Served at 127.0.0.1:8765, /health ready (3 captures), stopped by SIGTERM (GO). Log: logs/M0-TV2.review_serve.log.
- Filed: M0-D119 (after M0-TV2), M0-D120 (after M0-D119), M0-D121 (after M0-D109, Phase 16's head). Flag → M0-V12 (glow, readout
  numbers, supernova banner).
- Session 1's finding 2's check «is the disk round at step 0?» answered: round at step 1 (s02-dropped) — the square grows.
