# M0-TV1 — UI/UX pass, Phase 3 (the first window and the web pages)

Box: linux-pc (laserax-ai). Model: model=claude-sonnet-5-5 level=high (rung_record.py now). 2026-10-09.

## The set — images/tv1/ (git-ignored by design), 9 captures, every one looked at
| id | what | verdict |
|---|---|---|
| first-frame | desktop, 1760×940, llvmpipe on a private Xvfb (`--capture`, M0-T8's harness; script desktop-script.json) | **flagged** — bare bg.space window with a "step 3" stand-in label; §4.2's panels arrive in Phase 14 (TV2) → not a D |
| paused | desktop, after `pause` | passed — label reads "step 3 · paused", the state the script set |
| stepped | desktop, paused + one `steps` | passed — "step 4 · paused": one step taken, still paused |
| running | desktop, `pause:false`, 30 frames | passed — "step 34", no suffix: running again |
| web-no-webgpu-1280x800 | `/?no-webgpu=1`, Chrome headless `--screenshot` | **flagged** — the only real web screen; strings = §4.1, heading 18 px / body 14 px, text.primary on bg.space, no canvas |
| web-no-webgpu-390x844 | same, phone width | passed — text wraps inside 16 px gutters, no horizontal overflow (read from the image; the capture_web.mjs overflow measure was not run, it needs Playwright, M0-T15) |
| web-default-noflags-1280x800 | `/` with no flags: this Chrome has no adapter, so §4.9 shows by itself | passed — byte-identical to the ?no-webgpu=1 capture (cmp) |
| web-loading-held-1280x800 / -390x844 | the loading line, **SYNTHETIC**: the real page served with a stub `navigator.gpu` whose `requestAdapter` never answers | passed — `web.loading` centred on bg.space; NOT PROVEN (synthetic) as a state: a real run holds it only with WebGPU |

The lead (review1.md): no remark on the two flagged captures. No D filed: nothing in the set fails the contract.

## What could not be captured
- The loading line with a real adapter: headless Chrome here exposes no WebGPU adapter under three flag sets (contract §6.2.3's
  Vulkan flags among them; the logs/M0-TV1.chrome.log); the stub above shows the markup and CSS, nothing of the loading state.
  M0-T15's smoke grades `srState` `loading` → `ready`.
- The desktop window on win-laptop (`--offscreen-window`): NOT RUN here (linux-pc).
- Phase 3's `error` page (§4.8): needs a failing wasm load; not built into a state a capture can reach.

## NOT PROVEN (static capture) — Rules' list
Motion, timing, frame rate and input feel: a capture shows none (the step counter's change between captures is a count,
not a rate).

## Commands (all run on linux-pc, 2026-10-09)
- Desktop: `xvfb-run -a -s "-screen 0 1920x1080x24" env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE build/target/release/sandbox-reactions --adapter llvmpipe --capture milestones/m0/images/tv1/desktop-script.json --out milestones/m0/images/tv1` → exit 0, 4 PNGs (logs/M0-TV1.desktop.log)
- Web: `python3 -m http.server 47812 --bind 127.0.0.1 --directory web/dist` (started and stopped by this task), then `google-chrome --headless=new --window-size=… --virtual-time-budget=4000 --screenshot=… <url>` (logs/M0-TV1.chrome.log)
- Page: `review_page.py build`, then `serve` on 8765 (logs/M0-TV1.serve.log), stopped after the lead's answer; port 8765 free again.
