# Engine and language research — M0-R1 (2026-10-08)

**Summary.** Recommended: **Rust with our own thin engine — wgpu (GPU), winit (window and input),
egui (panels)**. Runner-up: **Rust with Bevy**. The deciding fact: the only way to run the same
GPU simulation code on the desktop *and* in the browser is the WebGPU API, and Rust's wgpu targets
both from one code base as first-class backends (E1, E2); Godot's and raylib's web builds are
WebGL2-only and WebGL2 has no compute shaders (E4, E5, E10), so on those stacks the web build would
need a second, CPU-only simulation. Bevy runs on wgpu too, so it keeps the same web and GPU story,
but adds a breaking release every few months for engine features M0 barely uses (E7, E8). The
I11 card claim holds: the Quadro RTX 4000 is an RTX 2070 by its chip and core count (E21–E23). **The
lead picked the recommendation** (2026-10-08) — verbatim in the Ruling section.

Written to PLAYBOOK §4 Rule 7: every claim lives once in `## Evidence` with its URL and access
date on its line; the tables cite claims by `E<n>`. **Opinion** is labelled; every claim no source
confirmed is in `## UNVERIFIED`. Box for every local probe: linux-pc.

## Evidence (each claim with its source)
**Web platform**
- E1 · wgpu (latest v30.0.1) supports Vulkan, Metal, DX12 natively as first class and the web through WebGPU (first class) or WebGL2 (best effort); licence MIT / Apache-2.0; MSRV 1.87 — https://github.com/gfx-rs/wgpu (accessed 2026-10-08) · https://github.com/gfx-rs/wgpu/releases (accessed 2026-10-08)
- E2 · wgpu's `COMPUTE_SHADERS` downlevel flag: "WebGL2, and GLES3.0 devices do not support compute" (wgpu 30.0.1 docs) — https://docs.rs/wgpu/latest/wgpu/struct.DownlevelFlags.html (accessed 2026-10-08)
- E3 · WebGPU shipping status: Chrome on Windows/macOS/ChromeOS since 113; Chrome on Linux for Intel Gen12+ since 144 and for NVIDIA (driver 535.183.01+) on Wayland since 147; Firefox on Windows since 141, on macOS since 147, on Linux only in Nightly ("expected to ship in 2026"); Safari 26 on macOS/iOS/iPadOS/visionOS — https://github.com/gpuweb/gpuweb/wiki/Implementation-Status (accessed 2026-10-08)
- E4 · Godot 4.7 web export: "Godot 4 can only target WebGL 2.0 (using the Compatibility rendering method)"; "Godot currently does not support WebGPU"; threads need SharedArrayBuffer with COOP `same-origin` and COEP `require-corp`; GDExtensions "need to be specifically compiled for the web platform" — https://docs.godotengine.org/en/stable/tutorials/export/exporting_for_web.html (accessed 2026-10-08)
- E5 · Godot 4.7: "Compute shaders can only be used from RenderingDevice-based renderers (the Forward+ or Mobile renderer)" — so not Compatibility, so not the web — https://docs.godotengine.org/en/stable/tutorials/shaders/compute_shaders.html (accessed 2026-10-08)
- E6 · A third-party experimental WebGPU backend for Godot (a RenderingDeviceDriver running Forward+/Mobile in browsers) had a public beta on 2026-05-10; it is not part of Godot 4.7 (E4) — https://talks.godotengine.org/godotcon-boston-2026/talk/CT739D/ (accessed 2026-10-08)
- E9 · SharedArrayBuffer (browser threads sharing memory) needs a secure context (HTTPS) and cross-origin isolation, set by the `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers; without them the constructor is hidden — https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer (accessed 2026-10-08)
- E10 · raylib: zlib/libpng licence; OpenGL 1.1/2.1/3.3/4.3 and GLES 2/3 backends; web through Emscripten with WebGL1/2 — https://github.com/raysan5/raylib (accessed 2026-10-08)
- E11 · SDL3 on Emscripten: rendering "must be done on the main thread"; WebGL2 / GLES3 contexts where the hardware allows; a callback main loop driven by requestAnimationFrame — https://wiki.libsdl.org/SDL3/README-emscripten (accessed 2026-10-08)
- E12 · emdawnwebgpu is Dawn's `webgpu.h` for Emscripten, "works in any browser supporting WebGPU"; its C++ bindings "are not intended to be fully stable" — https://dawn.googlesource.com/dawn/+/HEAD/src/emdawnwebgpu/pkg/README.md (accessed 2026-10-08)
- E13 · bgfx: BSD-2-Clause; backends D3D11/12, Metal, OpenGL 4.3+, GLES 3.0+, Vulkan, WebGL 2.0, and "WebGPU (Dawn Native only)" — so in the browser bgfx is WebGL2 — https://github.com/bkaradzic/bgfx (accessed 2026-10-08)
- E14 · Rust web threads (wasm-bindgen-rayon): nightly Rust, `build-std`, `+atomics,+bulk-memory`, and cross-origin isolation; Apache-2.0 — https://github.com/RReverser/wasm-bindgen-rayon (accessed 2026-10-08)

**Engines and libraries**
- E7 · Bevy releases: 0.16 (2025-04-24), 0.17 (2025-09-30), 0.18 (2026-01-13), 0.19 (2026-06-19) — https://bevy.org/news/ (accessed 2026-10-08)
- E8 · Bevy: MIT / Apache-2.0; "Bevy is still in the early stages of development"; breaking releases on a train schedule; a "fast compiles" setup with dynamic linking is recommended — https://github.com/bevyengine/bevy (accessed 2026-10-08)
- E15 · winit: Apache-2.0; Windows, macOS, Linux X11 and Wayland, Web (WASM), iOS, Android — https://github.com/rust-windowing/winit (accessed 2026-10-08)
- E16 · egui: MIT / Apache-2.0; eframe runs on Web, Linux, Mac, Windows, Android; renders through egui-wgpu or egui_glow — https://github.com/emilk/egui (accessed 2026-10-08)
- E17 · Godot: MIT, "no royalties" — https://github.com/godotengine/godot (accessed 2026-10-08)
- E18 · Godot `--headless` = `--display-driver headless --audio-driver Dummy` (no picture); frame capture is `--write-movie`, which renders — https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html (accessed 2026-10-08)
- E19 · wgpu `RequestAdapterOptions.compatible_surface` is an `Option` — no window needed except on WebGL; `power_preference` asks for the high-performance or low-power adapter — https://docs.rs/wgpu/latest/wgpu/type.RequestAdapterOptions.html (accessed 2026-10-08)
- E20 · WGSL lets implementations fuse `a * b + c` into one operation and flush denormals, and states accuracy in ULPs — the same shader may give slightly different floats on different GPUs — https://www.w3.org/TR/WGSL/#floating-point-evaluation (accessed 2026-10-08)
- E30 · Tiny Glade (Steam, 2024-09-23, Windows and Linux; macOS 2026-05-13) was built on a modified Bevy — https://en.wikipedia.org/wiki/Tiny_Glade (accessed 2026-10-08)
- E31 · Metalrain: a GPU falling-sand engine in Rust on wgpu, with a public interactive demo — https://metalraindev.itch.io/metalrain-gpu-tests (accessed 2026-10-08)

**The card (I11)**
- E21 · Quadro RTX 4000 datasheet: Turing, 2304 CUDA cores, 8 GB GDDR6, 7.1 TFLOPS FP32, 160 W board power — https://www.nvidia.com/content/dam/en-zz/Solutions/design-visualization/quadro-product-literature/quadro-rtx-4000-datasheet.pdf (accessed 2026-10-08)
- E22 · NVIDIA's RTX 20 comparison: RTX 2060 1920 (or 2176) cores, 2060 Super 2176, RTX 2070 2304 at 1.71 GHz boost, 2070 Super 2560 — https://www.nvidia.com/en-us/geforce/graphics-cards/compare/?section=compare-20 (accessed 2026-10-08)
- E23 · ServeTheHome's review: in Geekbench, LuxMark and AIDA64 GPGPU the Quadro RTX 4000 lands "close to the ZOTAC RTX 2070 Blower"; in Hashcat, closer to an RTX 2060 — https://www.servethehome.com/nvidia-quadro-rtx-4000-review-a-versatile-ai-and-professional-gpu/3/ (accessed 2026-10-08)

**Steam**
- E24 · Steam Direct: "$100.00 USD fee for each product", recouped after $1,000 adjusted gross revenue — https://partner.steamgames.com/steamdirect (accessed 2026-10-08)
- E25 · Steam on open source: MIT, BSD, Apache 2.0 are fine with the Steamworks SDK; "copyleft" licences like the GPL are "problematic when combining code with the Steamworks SDK" — https://partner.steamgames.com/doc/sdk/uploading/distributing_opensource (accessed 2026-10-08)
- E26 · steamworks-rs 0.13.0: MIT / Apache-2.0, binds Steamworks SDK 1.80.0; the SDK's redistributable libraries ship next to the game — https://github.com/Noxime/steamworks-rs (accessed 2026-10-08)
- E27 · GodotSteam's GitHub repository was archived on 2026-10-05 and moved to Codeberg — https://github.com/GodotSteam/GodotSteam (accessed 2026-10-08)

**Prior art**
- E28 · The Powder Toy: C++ with SDL, Meson build, GPL-3.0 — https://github.com/The-Powder-Toy/The-Powder-Toy (accessed 2026-10-08)
- E29 · The Powder Toy's grid: `CELLS = Vec2(153, 96)`, `CELL = 4`, `RES = CELLS * CELL` = 612 × 384, `NPART = XRES * YRES` (235,008 particle slots); pressure, gravity and walls live on the coarse 153 × 96 grid — https://raw.githubusercontent.com/The-Powder-Toy/The-Powder-Toy/master/src/SimulationConfig.h (accessed 2026-10-08)
- E32 · The Powder Toy 100.1 offers "Play online" in WebAssembly — https://powdertoy.co.uk/Download.html (accessed 2026-10-08) · snapshot 401 (2026-10-04) notes web-build work — https://github.com/The-Powder-Toy/The-Powder-Toy/releases (accessed 2026-10-08)
- E33 · Sandspiel: MIT; the simulation is Rust compiled to WebAssembly, drawn with WebGL, with JS glue — https://github.com/MaxBittker/sandspiel (accessed 2026-10-08)
- E34 · Noita: proprietary, Nolla Games' own engine "Falling Everything", Lua-scripted — https://en.wikipedia.org/wiki/Noita_(video_game) (accessed 2026-10-08) · GDC 2019 talk on scaling the falling-sand simulation to large worlds — https://www.gdcvault.com/play/1025695/Exploring-the-Tech-and-Design (accessed 2026-10-08)
- E35 · Universe Sandbox: commercial, Unity engine, gravitational n-body, Windows/macOS/Linux — https://en.wikipedia.org/wiki/Universe_Sandbox (accessed 2026-10-08)
- E36 · Sandboxels: JavaScript, in-browser; the "R74n Content License" forbids commercial use without permission — not open source — https://raw.githubusercontent.com/R74nCom/sandboxels/main/license.txt (accessed 2026-10-08)

**Probed on linux-pc (2026-10-08, read-only)**
- P1 · Google Chrome 155.0.8059.39 and Firefox 157.0 installed; Rust targets installed: `x86_64-unknown-linux-gnu` only (no `wasm32-unknown-unknown`). Repo facts list the rest (Rust 1.99, Godot 4.7 with export templates, no cmake/clang/emcc/SDL dev).

## Candidates
| # | Stack | What it is | Web GPU path | Prior art on it |
|---|---|---|---|---|
| C1 | **Rust, own engine** — wgpu + winit + egui | We write the loop, the cell renderer, the glow pass and the sim; libraries give GPU, window, panels | WebGPU (compute) via wgpu; WebGL2 fallback without compute (E1, E2) | Metalrain (E31); Sandspiel's Rust→WASM sim (E33) |
| C2 | **Rust, Bevy** | A full ECS engine on top of wgpu: window, input, UI, render graph, bloom | Same as C1 — Bevy renders through wgpu | Tiny Glade (E30) |
| C3 | **C++, own engine** — SDL3 + one of: (a) OpenGL, (b) WebGPU through Dawn / emdawnwebgpu, (c) bgfx | Same shape as C1 in C++ | (a) WebGL2, no compute (E11); (b) WebGPU, compute (E12); (c) WebGL2 in the browser (E13) | The Powder Toy (C++/SDL, E28) |
| C4 | **C++, Godot 4.7 + GDExtension** | A full engine and editor; our sim as a C++ extension | None: web is WebGL2 / Compatibility only, no compute (E4, E5); an experimental third-party WebGPU beta exists (E6) | — |
| C5 | **C++, raylib** | A small game library | None: WebGL1/2 (E10); GL 4.3 compute on desktop only | — |
| C6 | **C#, Unity** (found by the research: Universe Sandbox) | Commercial engine | — | Universe Sandbox (E35) |

C6 is listed because the research found a shipped astrophysics sandbox on it, and dropped: the
lead fixed the language — "I know were going to have to build it in c++ or rust" (bootstrap §1).

## Criteria
Scores: **✔** meets it · **~** meets it with a cost · **✗** fails it. Each cell names its evidence;
**opinion** where it is a judgement.

| Criterion | C1 Rust + wgpu | C2 Rust + Bevy | C3 C++ own (best: b, Dawn) | C4 C++ Godot | C5 C++ raylib |
|---|---|---|---|---|---|
| One code base, desktop (Linux, Windows; macOS noted) | ✔ Vulkan/DX12/Metal (E1), winit (E15) | ✔ (E1, E8) | ✔ SDL3 + Dawn native (E12) | ✔ | ✔ |
| …and the web, same sim code | ✔ WebGPU with compute (E1, E2) | ✔ same as C1 | ~ emdawnwebgpu, C++ bindings "not … fully stable" (E12); (a) and (c) lose compute on the web (E11, E13) | ✗ WebGL2 only, no compute (E4, E5) — the web would need a CPU sim | ✗ WebGL only (E10) |
| Web reach today | ~ WebGPU: Chrome all desktops (Linux: Intel Gen12+, NVIDIA on Wayland), Safari 26, Firefox Win/mac — **not Firefox Linux** (E3) | ~ same | ~ same | ✔ WebGL2 everywhere — but no GPU sim | ✔ same as C4 |
| Browser threads and their price | ~ needs nightly Rust + build-std + COOP/COEP headers (E14, E9); not needed if the sim runs on the GPU | ~ same | ~ Emscripten pthreads + COOP/COEP (E11, E9) | ~ COOP/COEP when threads on (E4) | ~ same as C3 |
| GPU compute, ~240,000 cells at top speed (M0) | ✔ compute shaders native and on WebGPU (E1, E2) | ✔ through Bevy's render graph — **opinion**: more ceremony than raw wgpu | ✔ (b) | ~ desktop only, Forward+/Mobile RenderingDevice (E5) | ~ desktop GL 4.3 only (E10) |
| …and millions later (B21) | ✔ same API; scale is the sim's design, not the stack's — **opinion** | ✔ | ✔ | ~ desktop only | ~ desktop only |
| CPU threads (desktop) | ✔ std threads / rayon — **opinion**, no source fetched | ✔ Bevy task pools | ✔ std::thread | ✔ | ✔ |
| Steam | ✔ steamworks-rs 0.13, MIT/Apache, SDK 1.80 (E26); $100 per app (E24) | ✔ same crate | ✔ the SDK is C++, used directly — **UNVERIFIED** (no page fetched) | ~ GodotSteam moved to Codeberg 2026-10-05 (E27); licence UNVERIFIED | ✔ as C3 |
| Headless runs and frame captures | ✔ adapter without a window (E19): compute and render to a texture, read back — no display needed | ~ possible; Bevy headless rendering not sourced — **UNVERIFIED** | ✔ Dawn native, same model | ~ `--headless` draws nothing; captures need `--write-movie` with a display (E18) — Xvfb on linux-pc | ~ needs a GL context (Xvfb) — **opinion** |
| Determinism | ~ CPU path in Rust is ours to keep deterministic — **opinion**; GPU floats differ across GPUs (E20) → oracles by tolerance (R8) | ~ same, plus Bevy's scheduling to pin — **opinion** | ~ same as C1 (E20) | ~ same | ~ same |
| Build and iteration speed | ~ Rust compiles are slow-ish — **opinion**; small dependency tree | ~ heavier; "fast compiles" setup advised (E8) | ✗ Dawn must be built from source; cmake, emcc, SDL3 dev missing on linux-pc (Repo facts) | ~ editor + extension rebuilds — **opinion** | ~ |
| Tools already here (install time saved, not merit) | Rust 1.99 installed; add `wasm32-unknown-unknown` (P1) | same | none of cmake/emcc/SDL3/Dawn | Godot 4.7 + export templates installed (Repo facts) | none |
| Licences (commercial; copyleft — R7) | ✔ MIT/Apache (E1, E15, E16) | ✔ MIT/Apache (E8) | ✔ SDL zlib — **UNVERIFIED**; Dawn BSD-3 — **UNVERIFIED**; bgfx BSD-2 (E13) | ✔ MIT (E17) | ✔ zlib (E10) |
| Cost | $0; Steam $100 later (E24) | $0 | $0 | $0 | $0 |
| Exit path | ✔ WGSL shaders and the WebGPU model carry to Dawn/C++ or into Bevy (C2 is built on it) — **opinion** | ~ ECS-shaped code is Bevy-shaped; leaving means rewriting the glue — **opinion** | ✔ webgpu.h is a standard C API (E12) | ✗ a GDExtension is Godot-shaped — **opinion** | ✔ small surface |
| Engine churn | ✔ wgpu is one library — its releases also break, less surface — **opinion** | ✗ 4 breaking releases in 14 months (E7), "early stages" (E8) | ✔ | ✔ stable | ✔ |

**Prior art** — stacks, licences, and what "like The Powder Toy" means in numbers:
| Game | Stack | Licence | What it tells us |
|---|---|---|---|
| The Powder Toy | C++, SDL, Meson (E28) | GPL-3.0 (E28) — **ideas only, never code (R7)** | Grid 612 × 384 = 235,008 cells, with pressure and gravity on a coarse 153 × 96 grid (E29); ships a WebAssembly "Play online" build (E32) |
| Sandspiel | Rust → WASM, WebGL (E33) | MIT (E33) | A Rust sim in the browser on the CPU |
| Noita | own C++ engine "Falling Everything" (E34) | proprietary (E34) | Falling sand scaled to large worlds (E34) |
| Universe Sandbox | Unity (E35) | commercial (E35) | n-body astrophysics sandbox (E35) |
| Sandboxels | JavaScript (E36) | R74n Content License, no commercial use (E36) | — |
So "like The Powder Toy" (interview answer 10) = **612 × 384 cells**; the plan's "about 600 × 400"
is within 2 % of that count (235,008 vs 240,000).

**The claims interview I11 left to M0-R1:**
- **"The Quadro RTX 4000 is in the RTX 2060–2070 class" — holds.** Same 2304 CUDA cores as the
  RTX 2070 (E21, E22), 7.1 TFLOPS FP32 (E21), and compute benchmarks close to an RTX 2070, down to
  an RTX 2060 in one (E23). Consequence: the 60-frames check can run on linux-pc's Quadro, the
  adapter named (contract §6, Hazards). Falsified if: a game-style rendering benchmark puts it
  below an RTX 2060 — the sources here are compute benchmarks.
- **"About 600 × 400 cells … light enough for a laptop and for the web build" — holds for
  Powder-Toy-class rules, unproven for ours.** The Powder Toy runs its 612 × 384 grid in a browser
  (E29, E32), and Sandspiel runs a Rust sim in WASM (E33). But our cells carry gas dynamics,
  self-gravity, heat and fusion at many steps per frame at top speed — no source measures that;
  M0-R3 owns the frame budget. The laptop half has no source → `## UNVERIFIED`.

## Recommendation
**C1 — Rust with our own thin engine: wgpu for the GPU, winit for the window and input, egui for
the panels.** (Opinion, built on the evidence cited.)

Why:
1. **One simulation, desktop and web.** wgpu runs compute shaders natively and in the browser
   through WebGPU (E1, E2). M0's sim lives on the GPU; it is the only candidate pair (with C2 and
   C3b) where the web build runs the *same* sim. C4 and C5 would need a second, CPU sim for the web
   (E4, E5, E10).
2. **M0 needs little "engine".** One window, one cell texture, a glow pass, a few panels, a fixed
   tick — the hard part is the physics, which no engine supplies. Bevy's extra machinery costs a
   breaking upgrade every ~4 months (E7).
3. **Tests without a screen.** wgpu needs no window to compute and render to a texture (E19), so
   the harness can run the sim and read frames back headless on linux-pc (R4).
4. **Licences are clean** — MIT/Apache throughout (E1, E15, E16), Steam-compatible (E25, E26), no
   copyleft (R7).
5. **Install time saved** (not merit): Rust 1.99 is on linux-pc; add the wasm32 target (P1).

What this commits the next tasks to (consequences):
- The web build runs where WebGPU runs: Chrome (all desktops; on Linux, Intel Gen12+ or NVIDIA on
  Wayland), Safari 26, Firefox on Windows/macOS — **not Firefox on Linux yet** (E3). linux-pc's
  Chrome 155 on NVIDIA/Wayland qualifies (P1, E3) — the web smoke there is **NOT PROVEN** until a
  build runs. A browser without WebGPU gets a "needs WebGPU" message, not a CPU fallback — M0-TC
  rules it.
- GPU floats differ between GPUs (E20): R8's tolerances must be real, and a CPU reference of each
  law (in f64) is the natural oracle for the GPU path — M0-R2b / M0-TC to decide (opinion).
- Web threads are avoided in M0: the sim runs on the GPU, so no nightly Rust or COOP/COEP needed
  (E14, E9).

What would falsify this recommendation:
- wgpu cannot run M0's compute pipeline in Chrome on linux-pc (the first web smoke) → C3b or C2
  re-open.
- M0-R3 measures that a GPU step of ~240,000 cells at the top speed's steps per frame misses
  16.6 ms on the Quadro through wgpu, where a native API would hold it.
- Writing the cell renderer, glow and panels by hand costs more than one build lot (M0-TG's
  sizing) — then Bevy's (C2) built-ins pay for its churn.

**Runner-up: C2 — Rust with Bevy.** Its strongest case, written out: Bevy gives, for free, what
C1 must write — window and input plumbing, a render graph with a bloom (glow) effect, UI, asset
loading, an app structure that later milestones (story mode, sound, saving) can grow in; it is
on the same wgpu foundation, so it keeps C1's web and compute story whole; and a commercial game
shipped on Steam on a modified Bevy (E30). **Answer:** M0's engine needs are small and its risk is
the physics, not the plumbing; Bevy breaks on every release — 0.16 → 0.19 in 14 months (E7) —
and calls itself early-stage (E8), and each upgrade would be a task touching our sim's glue; the
sim's GPU code is wgpu either way, so a later move into Bevy stays open (the reverse exit costs
more). Bevy wins if the falsifier "hand-written renderer and UI cost more than one lot" fires.

**The C++ case, answered.** The lead named C++ as an equal option. Best C++ stack: C3b (SDL3 +
WebGPU through Dawn, emdawnwebgpu for the web) — the same shape as C1. It loses on today's facts:
Dawn built from source plus cmake, emcc and SDL3 dev to install (Repo facts), web C++ bindings
"not … fully stable" (E12). Godot (C4) is the most complete C++ option and is installed, but its
web build cannot run compute shaders (E4, E5), so the sandbox's GPU physics would never reach the
website the lead wants ("Ideally I would for it to run on a website, or to be able to sell it on
steam.").

## Dependency table (for C1)
| Name | Role | Licence | Cost | Exit path |
|---|---|---|---|---|
| Rust toolchain 1.99 (rustup) | language, cargo | MIT / Apache-2.0 — UNVERIFIED (no page fetched) | $0, installed | — |
| `wasm32-unknown-unknown` target | web build | as Rust | $0, `rustup target add` (R3) | — |
| wgpu 30.x | GPU: compute + render, native + WebGPU | MIT / Apache-2.0 (E1) | $0 | WGSL + WebGPU model → Dawn (C++) or Bevy |
| winit | window, input | Apache-2.0 (E15) | $0 | SDL3 |
| egui / eframe (egui-wgpu) | panels, readouts, inspector | MIT / Apache-2.0 (E16) | $0 | any immediate-mode UI |
| wasm-bindgen + a web bundler (wasm-pack or trunk) | web glue | UNVERIFIED (no page fetched) | $0 | — |
| steamworks-rs 0.13 + Steamworks SDK 1.80 | Steam — later milestone | crate MIT / Apache (E26); SDK Valve's terms (E25) | $0; Steam Direct $100 per app (E24) | — |
| Google Chrome 155 (installed) | the web smoke's browser on linux-pc | proprietary, test tool only | $0 | Safari / Firefox Win (E3) |
| Not in M0: wasm-bindgen-rayon | web threads | Apache-2.0 (E14) | needs nightly Rust | — |
M0-TC's dependency gate fixes versions; nothing here is installed by M0-R1.

## UNVERIFIED (refuted by default until a source confirms)
- 600 × 400 cells is "light enough for a laptop" — no source; and "for the web build" with *our*
  physics at top speed — no source (Powder-Toy-class rules only, E32). → M0-R3.
- linux-pc's Chrome 155 exposes WebGPU on the RTX 5090 / Quadro under Wayland — E3 says NVIDIA on
  Wayland since 147, but no build has run here. → the first web smoke.
- Rust 1.99, wasm-bindgen, wasm-pack / trunk licences — not fetched.
- SDL3 (zlib) and Dawn (BSD-3) licences — not fetched.
- The Steamworks SDK used directly from C++ — not fetched.
- GodotSteam's licence (moved to Codeberg, E27).
- Bevy headless rendering and frame capture — not sourced.
- Rust CPU-side determinism (same binary, same box) — opinion, no source.
- Rust compile time vs C++ for this project — opinion, no measurement.
- Noita's simulation being multithreaded CPU in chunks — common knowledge, the GDC talk (E34) is
  members-only; not used for any score.

## Ruling
Put to the lead by the question tool, one call: the recommendation first, the runner-up second,
each with its consequence in one sentence. A third option, C++ with Godot, was added because
the lead named C++ as an equal choice.

- Question (2026-10-08): "Which language and engine should Sandbox Reactions be built on? (Full comparison: milestones/m0/reports/engine_stack.md.) Every later milestone builds on this, so changing it later means a rewrite."
- Options shown: "Rust + wgpu, own engine (Recommended)" — "the same GPU physics code runs on desktop and in the browser (WebGPU: Chrome, Safari, Firefox on Windows/Mac, not yet Firefox on Linux), but we write the cell drawing, the glow and the menus ourselves." · "Rust + Bevy" — "same desktop and web reach, and the window, glow and menus come ready-made, but Bevy breaks its API with every release (four in the last 14 months), so each upgrade costs us a task." · "C++ + Godot" — "a mature editor and menus for free, but Godot's web build cannot run GPU compute, so the website version would need a second, slower physics engine on the CPU."
- **The lead's answer (verbatim, 2026-10-08): "Rust + wgpu, own engine (Recommended)"**

Ruled: Rust, with our own thin engine on wgpu + winit + egui. M0-TC fixes versions in its
dependency gate and the source layout in the contract.
