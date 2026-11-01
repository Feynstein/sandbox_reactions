# M0-R12 — the views' display range

Research for the lead's ask of 2026-11-01 (M0-R12's `Ask (verbatim):`): a way to tone the Glow view (and every view that
saturates or loses resolution) down «to see more heated stuff», physically true colours kept — «I wouldnt like for something
like a neutron star to show up as red» — a Reset to today's 3,000 K look «to make sure the players can have the full
experience still», a slider with a typed value, and «a way to view the live colormap». Measured on linux-pc (laserax-ai) from
real state dumps made on laserax-ai-server's RTX 4090 (R18); no product code changed.

**Sources.** The dumps (logs/M0-R12.lives.log): the Sun-like preset to `stage:main_sequence` (step 2,720, +1 → 2,721), the
Massive preset to `event:supernova` (dumps every 64; the supergiant at 768 → 769, the supernova at 928 → 929) and on with
`--continue` to step 8,000 (a `neutron_star`, 4,373 sandbox mass → 8,001), and the Sun-like giant branch from the v9 life kept on
linux-pc (build/v9-sun-life, git e788da4, step 150,000 → 150,001: its mass falling from 3,350 to 2,913 and to 1,544 by 230,000;
its stage is not restored without its observers — a giant by its mass loss, my reading). Each dump was re-run one step by
today's binary (`headless --load-dump … --steps 1 --frames-every 1 --view <v>`, the four views) so the scratch renderer is graded
on the binary's own frames of the same state.

**The scratch renderer** (logs/M0-R12.scratch.*): `r12_extract.rs` turns a dump into per-cell T_K, Σ and the ten mass fractions
through the CPU EOS and `TempScale` in f64 (the path of tests/gpu/views.rs's CPU model, built in a scratch copy, never in the
tree); `r12.py` draws today's four views and the glow's passes (the model of tests/gpu/glow.rs) in numpy; `r12c.py` holds the
candidates; `r12_draw.py`, `r12_redder.py`, `r12_legend.py` draw and measure. **Today's picture reproduced** (logs/M0-R12.today.log):
Heat, Element and Density within 1/255 on every cell but 9 of the supernova's (its cold dense centre, Σ 142, T at the floor —
the EOS table's f32 cancellation, worst 4.2 levels); Glow within 1/255 on all but 0.0004–0.42 % of the cells, every one of them at
3,200–3,330 K, where a cell's brightest channel straddles the bright pass's 0.8 (§0.5 [M0-D112]) between the GPU's f32 and the
model's f64 temperature — the glow pass matched, that threshold named.

## Problem

Where each view saturates or loses resolution — the cells with Σ ≥ Σ_vac (logs/M0-R12.problem.log; images/r12/*.glow.today.png,
*.heat.today.png, *.density.today.png):

| Scene | Matter cells | T_K max | Glow: matter at b > 0.99 | Heat: matter's p1–p99 in the map's 9 decades | Density: p1–p99 in 10 decades |
|---|---|---|---|---|---|
| Sun-like, main sequence (2,721) | 219,676 | 1.1 × 10⁶ | 0.6 % (the 1,060 cells above 5,000 K span 2.3 decades) | 0.29 decades (3 %, 0.16 of a stop) | 2.84 (28 %) |
| Sun-like, giant (v9, 150,001) | 240,000 | 3.3 × 10⁷ | 2.0 % (3,359 cells, 3.8 decades) | 0.39 (4 %) | 2.74 (27 %) |
| Massive, supergiant (769) | 100,497 | 4.2 × 10⁹ | **100 %** — every cell the table's flat end colour | 2.44 (27 %) | 4.71 (47 %) |
| Massive, supernova (929) | 133,983 | 1.9 × 10⁸ | **100 %** | 1.80 (20 %) | 4.62 (46 %) |
| Massive, neutron-star remnant (8,001) | 240,000 | 1.6 × 10⁷ | 6.2 % (9,808 cells, 3.5 decades) | 1.10 (12 %) | 3.87 (39 %) |

- **Glow** — b = 1 − exp(−(T_K/3,000 K)⁴) is 0.9996 at 5,000 K, and the table's hue stops at 40,000 K: every hotter cell draws one
  colour at full brightness, and the glow's 0.6 add clamps it white per channel. The supergiant and the supernova are one white
  disk (images/r12/massive_sg.glow.today.png, massive_sn.glow.today.png); 2–6 decades of temperature inside a star draw as one
  colour. Read as physics: today's b is the bolometric surface brightness σT⁴ on an exponential film curve, 1 − e^{−x}, exposed
  for 3,000 K — that curve holds about 0.6 of a decade of T between black and white, so no single exposure shows a star.
- **Heat** — a stop every 1.8 decades from 10 K to 10 billion K: a star's matter sits in a fifth of the map, the Sun-like
  envelope in 0.16 of a stop (one flat purple; images/r12/sun_ms.heat.today.png).
- **Density** — 10 decades for matter spanning 2.7–4.7; the envelope one flat blue-green round a green core, no cell clamped.
- **Element** — the species colours are mass fractions, which no range touches; only the visibility factor clamp((log₁₀ Σ + 6)/8,
  0.15, 1) reads Σ, and it never reaches its cap (0.0 % of the matter in every scene) and holds its 0.15 floor in ≤ 5.2 % (the
  thinnest gas). It does not saturate: no control proposed.

## Candidates

Each drawn from the five dumps at S = 3,000 K (today), 10⁵, 10⁶, 10⁷, 10⁸, 10⁹ K and an automatic S, beside today's picture:
images/r12/<scene>.glow.sheet.png (rows G1, G3, G3 with a hue-kept glow, G4), the single frames
images/r12/<scene>.glow.<G1|G3|G3.hue|G4>.<S>.png, and images/r12/<scene>.maps.sheet.png (Heat, Density, Element). The
headline, today beside the recommendation: images/r12/headline.png.

**Glow** (S, the setting, a temperature; colour = the table's at T unless named; the haze and bg.space as today):
- **G1 — exposure** (the lead's «dimming», colours kept): b = 1 − exp(−(T/S)⁴). Today's at S = 3,000 K by construction. Holds 0.57
  of a decade at any S: a hard-edged disk of the matter near S, everything cooler black (images/r12/massive_sg.glow.G1.1e+07.png).
- **G2 — the true visible luminance on a log (asinh) stretch** (Lupton et al. 2004, PASP 116:133; Reinhard et al. 2002): Y(T) =
  ∫ B_λ(λ, T) ȳ(λ) dλ over 380–780 nm in 5 nm steps — §1.11's own Planck × ȳ integral before its normalisation, ∝ T at the hot
  end (measured slope 1.0001 at 10⁸ K) — and b = asinh(x/β)/asinh(1/β) clamped to [0, 1], x = Y(T)/Y(S), β = 10⁻⁴ (about four
  decades of luminance). Not today's at S = 3,000 K (it draws 1,000 K gas at 0.0001 against today's 0.012; 0.021 the largest gap),
  so it fails «the Reset draws today's picture» on its own — the reason for G3.
- **G3 — today's curve times a luminance dimming** (G2 made exact at the default): b = (1 − exp(−(T/min(S, 3,000 K))⁴)) · D,
  D = asinh(x/β)/asinh(1/β) clamped to [0, 1], x = Y(max(T, 3,000 K))/Y(S), β = 10⁻⁴. At S ≤ 3,000 K, D = 1: G1's exposure, which
  brightens cold gas below 3,000 K and **is today's picture exactly at 3,000 K** (largest |b − today| over 10–10¹¹ K: 0). Above
  it, matter cooler than 3,000 K keeps today's shading under one common dimming, and the hot side grades over 3–4.4 decades: at
  S = 10⁷ K, b > 0.02 from 5,370 K and > 0.98 from 8.2 × 10⁶ K; at 10⁹ K from 31,400 K to 8.2 × 10⁸ K. Monotone in T at every S
  (checked over 4,001 temperatures, 9 settings).
- **G3 with a hue-kept glow** — the glow composite divides the sum by its largest channel when that passes 1 (Lupton's
  saturation to the brightest channel) instead of clamping each channel: hot cores stay the table's blue-white (158, 184, 255)
  instead of white. It changes today's picture at the default (the supernova blue-white, the Sun-like core a blue disk in an
  orange ring — images/r12/sun_ms.glow.G3.hue.3e+03.png), so it cannot hold the Reset on its own.
- **G4 — the colour shift** (the lead's first idea): today's formula at T · 3,000 K / S. A 10⁹ K body draws (170, 129, 87) at
  S = 10⁹ K — exactly a 3,000 K body's colour at the default (logs/M0-R12.redder.log); the supergiant's 10⁸ K core draws
  orange at S = 10⁷–10⁹ K (images/r12/massive_sg.glow.G4.1e+08.png). The lead's «neutron star red», measured: rejected.
- **G5 — automatic exposure**: S at the 99.5th percentile of the matter's T_K picks the cloud's bulk, not the star (4,890 K
  for the Sun-like main sequence, 1.4 × 10⁸ K for the supergiant); S at the selected star's `star.core` (§4.6) would follow
  the core. Either moves the brightness while time runs, so a star brightening or fading would be hidden by the exposure — not
  proposed for M0; a button that sets S once is the cheap form, if wanted.

**Heat and Density**:
- **H1/D1 — a range the player sets** (both ends), today's 1–10 / −6–4 the default and the Reset.
- **H2/D2 — an automatic range**, the matter's own extent (p0.5–p99.5 by cell count, or p0.5 to the maximum): the supergiant's
  Heat over 10^4.96–10^8.15 K shows the core's gradient and the envelope's (images/r12/massive_sg.heat.auto.png). Recomputed
  every frame it flickers as matter moves and a colour stops naming a temperature; read once, on a button — «Fit to view» — it is
  H1 with a filled-in range.
- **H3/D3 — histogram equalisation**: the map at the cell's rank among the matter. It shows the most structure
  (images/r12/massive_sg.heat.equalised.png) but no colour means a temperature any more, its legend's ticks crowd where the matter
  is and move every frame: rejected.
- **Element**: none needed (Problem). E1 (its visibility read over the Density range) drawn for completeness
  (images/r12/<scene>.element.auto.png): indistinguishable from today.

**The live legend** — a 44 px strip under the world (the world at s = 2 is 1,200 × 800; 872 px between the top bar and the status
bar leave 72 under it: 8 + 44 for the legend, 20 for `presets.place_hint`, which moves below it): the active view's scale drawn
over its full span — Glow the drawn colour of each temperature from 10 K to 10 billion K at the current S, Heat and Density their
map over the set range, Element the species swatches — decade ticks in §4.12's units, the setting marked in `accent`, the
hovered cell's value (the inspector's, M0-T93b's `Sim::cell_panel`) in `text.primary`. Mocks: images/r12/legend_mock.glow.png (G3,
S = 10⁸ K, the supergiant), images/r12/legend_mock.heat.png (10⁴–10⁹ K).

## Evidence

**Does a hotter body ever draw redder?** (logs/M0-R12.redder.log) T swept 10³–10¹¹ K (1,601 temperatures) at S over 10 K–10¹⁰ K,
the drawn 8-bit sRGB of a uniform field (its halo its own colour when its brightest channel passes 0.8), read as CIE 1931 x
(redder = larger x), colours whose brightest channel is ≥ 16/255:
- Every candidate, today included, has a cooler cell bluer than a warmer one where the haze (#2A3550, blue) outweighs a dim
  blackbody — today: 1,330 K bluer than 2,110 K by 0.114 in x. The haze is not a temperature colour; this is today's design.
- G1 and G3 with today's glow: the hottest, bloomed cells clamp to white (x 0.311) while slightly cooler unbloomed ones keep the
  table's blue-white (x 0.247) — a rise of 0.064 in x, toward white, never past it.
- The bound that holds: **no drawn colour is redder than the larger of its own blackbody colour's x and D65 white's 0.3127**
  (+ 0.005) — G1, G3 and G3 hue-kept PASS at both Σ (worst excess +0.0000, +0.0000, +0.0012); **G4 FAILS** (+0.139 at Σ_vac,
  +0.102 at Σ 1). So under G1 or G3 a neutron star is never drawn orange or red: it draws blue-white, or white where it blooms.
- The default and the Reset: G1 and G3 are today's formula at S = 3,000 K (b identical; the images at 3,000 K are today's,
  images/r12/<scene>.glow.G3.3e+03.png against *.glow.today.png); G2 and the hue-kept glow are not.

**What each costs** (NOT PROVEN — source only, no GPU run): G3's b depends on T and S alone, so the CPU builds a brightness table over
log T (the blackbody table's shape, ~1,024 entries, one Planck × ȳ integral per entry) when S changes and the shader reads it as it
reads the colour table — one more lookup per cell of a 240,000-cell pass that already reads 14 channels; H1/D1 change two
uniforms. «Fit to view» is one reduction (the matter's min and max of log T and log Σ) or one read-back, on the click only. The
legend is egui's painting of ≤ 1,200 rects a frame. The web build runs the same WGSL and panels. The renderer's block measures
`bench`'s frame_ms before and after (§3.3: bench renders the Glow view with the glow).

**What the scopes can hold**: the `views` scope's CPU formula (tests/gpu/views.rs) takes S and the two ranges and grades the GPU's
cell image within 1/255 at settings away from the default (its plant: the setting ignored, S held at 3,000 K → red at 10⁷ K);
a sweep case draws a strip of T 10³–10¹¹ K at five settings and checks the bound above on the GPU's drawn values (its plant: G4's
shift → red); a case at S = 3,000 K checks the image is today's byte for byte. The app's controls and legend: ui-scope cases on
the value parsing, the Reset and the tick labels, the capture's new actions.

## Recommendation

1. **Glow: G3**, one setting «Full brightness at», a log slider from 10 K to 10 billion K with its value typed in kelvins, a
   Reset to 3,000 K — the lead's «dimming» with true colours: brightness from the blackbody's own visible luminance on an
   astronomer's log stretch, the colour the table's at T, today's picture exactly at 3,000 K, and below 3,000 K an exposure that
   brightens cold gas. The glow composite kept as today's (a camera-like white where the hottest matter blooms; the Reset holds) —
   the hue-kept glow is the lead's fork, since it changes today's look.
2. **Heat and Density: H1/D1 with «Fit to view»** — each view's range, From and To on log sliders over the map's full span with
   typed values, a Reset to today's range, and a button that sets the range once to the matter in the world. No continuous
   automatic range, no equalisation.
3. **Element: no control.**
4. **The live legend under the world** for every view, the setting and the hovered cell's value marked on it.
5. The display settings kept by Clear (§1.12, like the view) and by switching views; none saved across launches (saving is past
   M0). Three build blocks: the renderer's settings (sr-engine), the panel's controls with their Reset and «Fit to view»
   (sr-app), the legend (sr-app).

## Ruling

The lead, 2026-11-01, at M0-R12's fork (question tool), verbatim:
- Glow: « G3 luminance dimming (Recommended) »
- The bloom: « Keep today's white bloom (Recommended) »
- Heat and Density: « Range + Fit to view (Recommended) »
- The legend: « Under the world (Recommended) »

Asked during the task, outside the fork (the lead's words): « thats looks better, thanks. But id like to ask you a question: is it
normal that we cant see the remnant solid core? Did we think about doing a zoom or something? » — measured: the remnant's 33 cells
at Σ > 10 read 10–330 K (its pressure the cold term), so no display setting lights it, and it is ~6 × 6 cells (12 px at s = 2); a
zoom was never planned (the interview's «zooming view», B28, was real sizes, dropped) — then: « File an R block to look into it
(Recommended) » and « File a zoom block after R12's (Recommended) ». And: « oh and also file something so that we can have rotation,
id like for us to be able to see the relativistic jets from black holes and neutron stars. MAybe they could rotate. If thats not too
much work » — after the note that a 2D face-on world points a jet out of the screen and that jets need magnetic fields the sandbox
lacks: « R block in M0, research only (Recommended) ».

The contract amended in place, tagged [M0-R12]: §1.11 (the Glow setting S with its guarantee, the Heat and Density ranges and Fit to
view, the zoom), §1.12 (Clear keeps them), §3.3 (five capture actions), §4.1 (a prose passage: eleven keys, each naming the block
that adds it to the block and strings.rs), §4.2 (the controls, the legend, the hint under it), §4.10 (the zoom's inputs), §4.11 (the
legend row), §4.12 (legend ticks); §0.5 [M0-R12] and its mirror in m0_superseded.md (R19).

## Blocks filed

Rated by PLAYBOOK §0's rubric, rule (3) — the formulas, strings and layout now fixed by the contract (Sonnet 5.5, high); the two
research blocks rule (3)'s list → `(usual)`:
- **M0-T126** — the renderer's display settings (sr-engine; scope `views`) · after M0-R12.
- **M0-T127a** — Fit to view's histogram (sr-engine; new scope `extents`).
- **M0-T127b** — the display controls in the left panel (sr-app; new scope `ui_display`; the five `views.*` keys).
- **M0-T127c** — their capture actions (scope `capture`).
- **M0-T128** — the live legend under the world (sr-app; new scope `ui_legend`; the six `legend.*` keys).
- **M0-T129** — the zoom (sr-app; new scope `ui_zoom`; capture action `zoom`) — all six before M0-T94, so the web build's
  window (M0-T94) and the panels' cost (M0-T95) see them.
- **M0-R13** — the cold remnant (research) · after M0-D71, at Phase 16's head.
- **M0-R14** — rotation and jets (research only) · after M0-T102, at Phase 16's end.
- Flags: M0-T96 (running.md, testing.md's rows), M0-TV3 (each view toned down and reset, the legend, the zoom, on one star),
  M0-V13 (contract-vs-code of [M0-R12]).
