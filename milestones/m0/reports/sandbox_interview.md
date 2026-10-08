# Sandbox interview — M0-TI (2026-10-08)

The record of M0-TI: the lead interviewed by the question tool on 2026-10-08 — 18 questions in
five rounds — on the star's life, the time control, the world, the sandbox and the feel. Answers
are verbatim and numbered 1–18; every later section cites them by number. The description of the
option the lead picked was on screen and is part of what was approved (§2), as in
reports/bootstrap.md §2. Inference is labelled "my reading" and kept in §3. Not asked, because
reports/bootstrap.md §2 settled it (PLAYBOOK §2.6 P1): the language and the engine, the targets
(desktop first, web kept alive), English, the build boxes, the size class, the ladder, the grants.

Readers: M0-TP rules on §5 and §6 with the lead · M0-R1, M0-R2 and M0-R3 take §4's MVP as input ·
M0-TC rules the open rows routed to it · M0-TG sizes the build pipeline against §4's MVP.

## 1. The answers, verbatim (2026-10-08)

### Round 1 — the star, and how true to nature
| # | Question (as asked) | The lead's answer (verbatim) |
|---|---|---|
| 1 | How true to real astronomy should the star be? This choice shapes the physics research, the time control and how the tests check the star. | "Real laws, squeezed scale (Recommended)" |
| 2 | How does the player make a star? | "Paint, or drop a preset (Recommended)" |
| 3 | Which endings must M0 show? Each ending you pick adds its own physics and checks to M0; the others move to a later milestone. | "White dwarf (Recommended), Supernova + neutron star (Recommended), Black hole" |
| 4 | While a star is alive, can the player change it? | "Yes, any time (Recommended)" |

### Round 2 — the time control
| # | Question (as asked) | The lead's answer (verbatim) |
|---|---|---|
| 5 | How does the player set the speed of time? Every option keeps a pause and a single-step key (moves the world one tick while paused), like The Powder Toy. | "Speed steps (Recommended)" |
| 6 | How wide should the speed range be? (At normal speed a whole life takes some minutes, as squeezed scale implies; the research sets the exact length.) | "Slow-mo to a life in ~10 s (Recommended)" |
| 7 | What does the screen tell the player about the star? The sandbox computes in its own squeezed units either way; this is only what is displayed. | "Real-equivalent + stage (Recommended)" |
| 8 | When something big starts during a fast warp (ignition, core collapse, explosion), should time slow down by itself? | "Yes, with an off switch (Recommended)" |

### Round 3 — the world and the feel
| # | Question (as asked) | The lead's answer (verbatim) |
|---|---|---|
| 9 | What should the world look like? | "Pixels + glow (Recommended)" |
| 10 | How big is the world, counted in cells (the squares the player paints)? | "we do like the powder toy, but in a later milestone I want the player to choose at the start depending on his graphics card" (typed, not an option) |
| 11 | What is the weakest PC that must run M0 smoothly, at 60 pictures (frames) a second? | "A mid-range gaming PC (Recommended)" |
| 12 | Can the world hold several stars at once? | "Allowed, one promised (Recommended)" |

### Round 4 — the sandbox
| # | Question (as asked) | The lead's answer (verbatim) |
|---|---|---|
| 13 | Besides the brush, the eraser, the element menu and the heat/cool tools (already part of 'Yes, any time'), which tools must M0 have? Each is cheap: a view only re-colours what the physics already computes. | "Heat view (Recommended), Element view (Recommended), Density view (Recommended), Cell inspector (Recommended)" |
| 14 | Which elements can the player paint in M0? | "we do star matter in M0, and expand in later milestones" (typed, not an option) |
| 15 | Does M0 ship any chemical reaction, or only the reaction machinery that fusion uses? Both can share one mechanism: ingredients plus conditions in, products plus energy out. | "Machinery only, via fusion (Recommended)" |
| 16 | Can the player save a world and load it back in M0? | "Later (Recommended)" |

### Round 5 — two gaps the topic list left (sound, the world's edge)
| # | Question (as asked) | The lead's answer (verbatim) |
|---|---|---|
| 17 | Should M0 make any sound? | "No sound in M0 (Recommended)" |
| 18 | What happens to matter that reaches the edge of the world, for example a supernova's blast? | "the player can choose if it leaves for good or bounces back" (typed, not an option) |

## 2. The options as the lead saw them — ✔ picked; a typed answer picks none
**1 · Realism**
- ✔ Real laws, squeezed scale (Recommended) — "Nature's laws (gravity, gas pressure, heat flow, fusion that speeds up as it gets hotter), but sizes and times are squeezed so a star forms, lives and dies on one screen in minutes at normal speed. Like The Powder Toy: every pixel of the star can be touched. Tests check each law's exact answers, not the real Sun's numbers."
- Real numbers, zooming view — "Real sizes, masses and times: a Sun-like star lives 10 billion years on the clock, so top speed must reach billions of years per second. Sizes run from a cloud wider than the solar system to an Earth-sized white dwarf, so the view zooms as the star shrinks or swells. Tests check against the real Sun. Much harder: the long calm phases need a shortcut model, and the star's inside is mostly a model, not pixels."
- Cartoon rules, fun first — "Simple rules drive the stages (enough gas packed together lights up; fuel runs out, the star dies), with no real equations. Cheapest and most predictable, but the least physics, and nothing in nature to test it against."

**2 · Star start**
- ✔ Paint, or drop a preset (Recommended) — "Paint hydrogen gas with the brush and gravity pulls it together; how much you paint sets the star's mass, and so its ending, shown on a readout. Or drop a ready-made cloud of a chosen mass to reach a given ending in one click (handy for demos and tests)."
- Paint only — "The brush is the only way: the star's mass is whatever you paint. Purest sandbox, but getting a specific ending takes trial and error."
- Pick a mass only — "Choose a mass from a menu and a cloud of that mass appears, ready to collapse. No painting stars by hand."

**3 · Endings** (several allowed)
- ✔ White dwarf (Recommended) — "The Sun's fate: the star swells into a red giant, puffs off its outer layers as a glowing shell, and leaves a small, very dense, white-hot core that slowly cools."
- ✔ Supernova + neutron star (Recommended) — "A heavy star (about 8+ Suns) burns up to iron, its core collapses in about a second and the star explodes, leaving a tiny, super-dense neutron star."
- ✔ Black hole — "The heaviest stars (about 25+ Suns) collapse into a black hole that swallows nearby matter. Bending light around it (gravity lensing) stays in a later milestone either way."

**4 · Touch star**
- ✔ Yes, any time (Recommended) — "Add gas to a shining star (it gets heavier and its fate changes), erase a chunk, heat or cool it, and it reacts through the same physics. A true sandbox, but every stage must come out of the physics, not a script, which is harder to get right."
- Watch only — "Once a star lights up, its life plays out on its own; the player controls time and the view. Stages can follow a script, so each one is easier to make look right."

**5 · Time keys**
- ✔ Speed steps (Recommended) — "Fixed speeds on buttons and number keys (for example x0.1, x1, x10, x100). Easy to find the same speed again, and easy to test."
- Smooth slider — "One slider from slowest to fastest: any speed in between, but hard to land on the same speed twice."
- Steps and a slider — "Keys for the fixed speeds plus a slider for anything in between. A little more interface to build and test."

**6 · Speed range**
- ✔ Slow-mo to a life in ~10 s (Recommended) — "Slowest: about 10 times slower than normal, to watch a collapse or a supernova unfold. Fastest: a Sun-like star goes from cloud to white dwarf in about 10 seconds."
- Slow-mo to a life in ~1 min — "The same slow motion, but the fastest speed shows a whole life in about a minute. Less computing at the top end, so it stays smooth on slower PCs."
- Extreme: x0.01 to a life in ~1 s — "From 100 times slower than normal to a whole life in a second or two. Most dramatic and hardest: the physics must stay stable with both very small and very large time steps."

**7 · Readouts**
- ✔ Real-equivalent + stage (Recommended) — "A label names the stage (e.g. 'Red giant: burning helium') and the numbers are translated into real astronomy: mass in Suns, age in years, temperature in kelvins. Approximate, since the sandbox squeezes size and time, but they read like the real thing."
- Sandbox units + stage — "The same stage label, with the sandbox's own units (cells, ticks). Exact, but abstract."
- No readouts — "No labels or numbers: the player reads the star from how it looks."

**8 · Big events**
- ✔ Yes, with an off switch (Recommended) — "The warp drops to normal speed (or slow motion for an explosion) so the player doesn't miss it; a setting turns this off."
- No, the player decides — "Time stays at the speed the player set. At top speed an explosion may pass in a blink."

**9 · Look**
- ✔ Pixels + glow (Recommended) — "Visible square cells like The Powder Toy, plus a soft glow around hot, bright matter, so the star shines and a supernova flashes. The star's colour follows its temperature (red, yellow, white, blue)."
- Pure pixels — "Exactly The Powder Toy's look: flat coloured cells, no glow. Cheapest, and the clearest view of what each cell holds."
- Smooth clouds — "Gas drawn as smooth, blended clouds with no visible cells, like a space photo. Prettiest, but it hides the grid the player paints on, and it is more drawing work."

**10 · World size** (typed answer, §1)
- Like The Powder Toy (Recommended) — "About 600 x 400 cells, each drawn as a 2-3 pixel square. A big star spans roughly 100 cells. Light enough for a laptop and for the web build."
- Large — "About 1600 x 900 cells, close to one cell per screen pixel. Finer stars and explosions, but about 6 times the work per tick, so it needs a gaming graphics card."
- Player picks at start — "Small, medium or large, chosen when a new world starts. Every size must be tested, and the physics must hold at all three."

**11 · Weakest PC**
- ✔ A mid-range gaming PC (Recommended) — "A graphics card around an RTX 2060-2070, about 6 years old. This PC's second card, the Quadro RTX 4000, is in that class, so the target can be measured right here."
- Any recent laptop — "Built-in laptop graphics. The physics must stay light: fewer cells and simpler models."
- This PC only, for M0 — "M0 only has to run well on this PC's RTX 5090; tuning for weaker PCs waits for a later milestone. Fastest to build, but likely reworked later."

**12 · Many stars**
- ✔ Allowed, one promised (Recommended) — "Paint as many clouds as you like and they pull on each other, but M0 only promises, and tests, a single star's life. Pairs, orbits and collisions get polished in a later milestone."
- Several, all promised — "Two or more stars orbiting, merging or colliding are part of M0's promise and its tests. More physics and more checks in M0."
- One star only — "The world holds a single star at a time; painting a second cloud is not allowed. Simplest."

**13 · Tools** (several allowed)
- ✔ Heat view (Recommended) — "Colours every cell by its temperature, to watch the core heat up until fusion lights."
- ✔ Element view (Recommended) — "Colours cells by element (hydrogen, helium, carbon ... iron), to watch fusion's ash pile up in the core, layer by layer."
- ✔ Density view (Recommended) — "Colours cells by how tightly packed the gas is, to watch a cloud collapse and a supernova's shock wave travel out."
- ✔ Cell inspector (Recommended) — "Hover a cell to read what it holds: element, temperature, density, speed."

**14 · Elements** (typed answer, §1)
- Star matter (Recommended) — "Hydrogen and helium gas, plus the heavier elements fusion makes (carbon, oxygen, neon, silicon, iron ...), which can be painted too. About 8-10 elements."
- Hydrogen and helium only — "Only the two starting gases can be painted; heavier elements appear only from fusion."
- Star matter + everyday ones — "Also a few everyday elements (sand, water, rock) so the world feels like The Powder Toy from day one. They must then behave right too (fall, flow, melt, boil): a lot more physics in M0."

**15 · Chemistry**
- ✔ Machinery only, via fusion (Recommended) — "M0 builds one reaction mechanism and proves it with fusion (hydrogen into helium, and onward). The first chemical reactions, like mixing two elements with heat, arrive in a later milestone on that same mechanism. M0 shows no chemistry."
- One or two in M0 — "Also one or two simple chemical reactions (e.g. hydrogen burning with oxygen into water) to prove the mechanism handles both. They need cool, everyday conditions in the world, so more physics in M0."

**16 · Saving**
- ✔ Later (Recommended) — "M0 starts each world fresh; the presets already put any ending one click away. Saving comes in a later milestone, once the file format can be designed for all elements at once."
- Yes, in M0 — "Save any moment (say, just before a supernova) and load it back. The file format then has to keep opening in later versions, or M0's saves are declared throwaway."

**17 · Sound**
- ✔ No sound in M0 (Recommended) — "M0 is silent; sound comes in a later milestone. One less library to pick, approve and test now."
- A few effects — "For example a low hum while a star shines and a boom for a supernova. Needs a sound library (one more dependency to approve) and its own checks."

**18 · World edge** (typed answer, §1)
- It leaves for good (Recommended) — "The edge is open space: matter flying out is gone for good, as in real space. The physics checks count what left, so nothing goes missing unexplained."
- Walls bounce it back — "Solid edges: the blast reflects back inward. All matter stays in play, but a boxed-in supernova looks and behaves unlike real space."
- Wrap around — "Matter leaving one side comes back on the opposite side, like old arcade games. Nothing is lost, but it is unphysical for a lone star."

## 3. Inference — kept apart (my reading)
- **I1 · A flat world on one screen.** "we do like the powder toy" (10), "Visible square cells" (9)
  and "on one screen" (1) — my reading: a 2D grid that fits one screen, with no zooming camera in
  M0. How gravity's law carries to a flat world, while staying "nature's laws" (1), is a physics
  fork: M0-R2 researches it, M0-TC puts it to the lead.
- **I2 · Labels watch, never drive.** Every stage "must come out of the physics, not a script"
  (4), yet the screen names the stage (7) and slows down at "ignition, core collapse, explosion"
  (8) — my reading: stage names and events are recognised from the physics state as it runs,
  and never steer it.
- **I3 · One preset per ending, at least.** "drop a ready-made cloud of a chosen mass to reach a
  given ending in one click" (2) with three endings (3) — my reading: at least three presets,
  their masses set from M0-R2's thresholds. How the player chooses the mass (a short list, or any
  value) is not said → open, M0-TC.
- **I4 · The binding performance target.** Top speed, "a Sun-like star goes from cloud to white
  dwarf in about 10 seconds" (6), held at "60 pictures (frames) a second" on "A graphics card
  around an RTX 2060-2070" (11), on about 600 × 400 cells (10). My reading: this combination is
  M0's hardest performance promise. My arithmetic: with a life of "some minutes" at normal speed
  (6), a life in ~10 s puts the top step near ×10–×100, the steps answer 5's option showed.
  M0-R3 checks it is reachable; a shortfall is a fork for M0-TC (a lower top speed, or fewer
  pictures a second during a warp), never a silent cap.
- **I5 · The squeeze is the core problem.** "sizes and times are squeezed" while the laws stay
  (1). My arithmetic, sources to come from M0-R2: in nature a core collapse takes about a second
  and a Sun-like star shines about 10 billion years (≈ 3 × 10^17 s), so the stages' time scales
  differ by some 17 orders of magnitude; fitting them into minutes while each law keeps its form
  is what M0-R2 and M0-R3 must now solve.
- **I6 · Two sets of units.** The sandbox computes in squeezed units; the screen shows mass in
  Suns, age in years, temperature in kelvins (7). How each sandbox unit is translated (one factor
  per quantity, or stage by stage for age) is open → M0-R2 and M0-R3 propose, M0-TC fixes. In
  which units the cell inspector (13) reads is not said → open, M0-TC.
- **I7 · No "down" in M0.** Only star matter (14), in open space "as in real space" (18's shown
  option) — my reading: gravity in M0 comes from the matter itself; a Powder-Toy "down" for
  falling sand belongs to the milestone that brings everyday elements → open, M0-TZ.
- **I8 · Two edge modes.** "the player can choose if it leaves for good or bounces back" (18) — my
  reading: both modes ship in M0, each reaching the gas flow and the gravity at the edge. Which
  mode a new world starts in, and whether it may change while a star lives, are not said → open,
  M0-TC.
- **I9 · The world's size, kept a setting.** The size is chosen per graphics card "in a later
  milestone" (10) — my reading: M0 keeps the size a setting under the hood, not built in, so that
  later choice is no rewrite. A seam, proposed — M0-TC decides.
- **I10 · Premises the lead let stand.** The brush, the eraser and the element menu were the
  premise of question 13; the heat and cool tools come from 4 ("heat or cool it"); pause and
  single step were the premise of question 5; 60 frames a second the premise of 11. None was
  objected to; §4 cites them as premises, not picks.
- **I11 · Facts the questions stated, UNVERIFIED.** Stated from memory to explain the options; no
  source has checked them yet: the Quadro RTX 4000 is in the RTX 2060–2070 class (11); 600 × 400
  cells is "Light enough for a laptop and for the web build" (10); a supernova needs "about 8+
  Suns", a black hole "about 25+ Suns", and a core collapses "in about a second" (3); a Sun-like
  star lives "10 billion years" (1). M0-R1 checks the card and the web claim, M0-R2 the
  astrophysics, each with a source, before anything rests on them.
- **I12 · The recommendations' weight.** 14 of 18 answers took the recommended option as worded;
  answers 10 and 14 were typed but match the recommended option for M0; answer 3 added the
  not-recommended black hole; answer 18 typed a combination no option offered. The option texts,
  written by the interviewer, carry much of this record's detail — M0-TP's red-team tests the MVP
  against them, not only against the labels.

## 4. Behaviours, mapped (PLAYBOOK §2.6 P9)
Quotes are the lead's answer or the picked option's shown text (§2); "premise" marks I10.

| # | Behaviour | Tier | Rests on | Routed to |
|---|---|---|---|---|
| B1 | Nature's laws, squeezed: "sizes and times are squeezed so a star forms, lives and dies on one screen in minutes at normal speed"; "Tests check each law's exact answers, not the real Sun's numbers" | MVP | 1 | M0 — M0-R2 (models, oracles), M0-R3, M0-TC §5 |
| B2 | Start by painting: "Paint hydrogen gas with the brush and gravity pulls it together; how much you paint sets the star's mass, and so its ending, shown on a readout" | MVP | 2 | M0 |
| B3 | Presets: "drop a ready-made cloud of a chosen mass to reach a given ending in one click" | MVP | 2 | M0 — how the mass is chosen: M0-TC §4 (I3) |
| B4 | White dwarf: "swells into a red giant, puffs off its outer layers as a glowing shell, and leaves a small, very dense, white-hot core that slowly cools" | MVP | 3 | M0 — M0-R2 |
| B5 | Supernova + neutron star: "burns up to iron, its core collapses in about a second and the star explodes, leaving a tiny, super-dense neutron star" | MVP | 3 | M0 — M0-R2 |
| B6 | Black hole: "collapse into a black hole that swallows nearby matter" | MVP | 3 | M0 — M0-R2 |
| B7 | Touch any time: "Add gas to a shining star (it gets heavier and its fate changes), erase a chunk, heat or cool it, and it reacts through the same physics"; "every stage must come out of the physics, not a script" | MVP | 4 | M0 — M0-R2, M0-TC §5 (I2) |
| B8 | Speed steps: "Fixed speeds on buttons and number keys (for example x0.1, x1, x10, x100)"; a pause and a single-step key (premise) | MVP | 5 | M0 — M0-R3, M0-TC §3–§4 |
| B9 | Range: "Slowest: about 10 times slower than normal"; "Fastest: a Sun-like star goes from cloud to white dwarf in about 10 seconds" | MVP | 6 | M0 — M0-R3 (I4) |
| B10 | Readouts: "A label names the stage" and "the numbers are translated into real astronomy: mass in Suns, age in years, temperature in kelvins" | MVP | 7 | M0 — translation: M0-R2/R3 propose, M0-TC §4 (I6) |
| B11 | Auto slow-down: "The warp drops to normal speed (or slow motion for an explosion) so the player doesn't miss it; a setting turns this off" — at "ignition, core collapse, explosion" | MVP | 8 | M0 — M0-R3 (I2) |
| B12 | Look: "Visible square cells like The Powder Toy, plus a soft glow around hot, bright matter"; "The star's colour follows its temperature" | MVP | 9 | M0 — M0-TC §4, a TV pass |
| B13 | World size: "we do like the powder toy" — read as the shown option "Like The Powder Toy" (my reading): "About 600 x 400 cells, each drawn as a 2-3 pixel square" | MVP | 10 | M0 — M0-R2 (cost per frame), M0-TC §2 (I9) |
| B14 | Weakest PC: "A mid-range gaming PC" — "A graphics card around an RTX 2060-2070"; 60 frames a second (premise) | MVP | 11 | M0 — M0-R1 (criteria), M0-TC §6 (I4, I11) |
| B15 | Several stars: "Paint as many clouds as you like and they pull on each other, but M0 only promises, and tests, a single star's life" | MVP | 12 | M0 |
| B16 | Base tools: brush, eraser, element menu (premise); heat and cool ("heat or cool it") | MVP | 4, 13 | M0 — M0-TC §4 (I10) |
| B17 | Views and inspector: heat view, element view, density view; "Hover a cell to read what it holds: element, temperature, density, speed" | MVP | 13 | M0 — M0-TC §4 (I6) |
| B18 | Elements: "we do star matter in M0" — read as the shown option "Star matter" (my reading): "Hydrogen and helium gas, plus the heavier elements fusion makes (carbon, oxygen, neon, silicon, iron ...), which can be painted too" | MVP | 14 | M0 — M0-R2 picks the list, M0-TC §1 (element registry) |
| B19 | One reaction mechanism: "M0 builds one reaction mechanism and proves it with fusion (hydrogen into helium, and onward)"; "M0 shows no chemistry" | MVP — the seam for B23 | 15 | M0 — M0-R2, M0-TC §1 (one reaction registry) |
| B20 | Edges: "the player can choose if it leaves for good or bounces back" | MVP | 18 | M0 — M0-R2 (both edges), M0-TC (I8) |
| B21 | The world's size picked at start: "in a later milestone I want the player to choose at the start depending on his graphics card" | later — a seam proposed (I9) | 10 | after M0: M0-TZ places it with the lead |
| B22 | More elements: "expand in later milestones" | later | 14 | M0-TZ |
| B23 | Chemistry: "The first chemical reactions, like mixing two elements with heat, arrive in a later milestone on that same mechanism" | later — on B19's seam | 15 | M0-TZ |
| B24 | Saving and loading: "Saving comes in a later milestone, once the file format can be designed for all elements at once" | later | 16 | M0-TZ |
| B25 | Sound: "M0 is silent; sound comes in a later milestone" | later | 17 | M0-TZ |
| B26 | Pairs, orbits, collisions promised: "Pairs, orbits and collisions get polished in a later milestone" | later | 12 | M0-TZ |
| B27 | Gravity lensing: "Bending light around it (gravity lensing) stays in a later milestone either way" | later — as reports/bootstrap.md §3 already routed it | 3 | M0-TZ |
| B28 | Not picked, so dropped for M0 — re-adding one is a new ask, never a silent change: real numbers with a zooming view, cartoon rules (1) · watch only, scripted stages (4) · a speed slider (5) · a range down to x0.01, or a life in ~1 min at top speed (6) · sandbox units on screen, or no readouts (7) · smooth cell-less clouds (9) · a laptop, or this PC only, as the weakest PC (11) · one star only (12) · hydrogen and helium only (14) · wrap-around edges (18) | dropped | 1, 4–7, 9, 11, 12, 14, 18 | — |
| B29 | The default edge mode, and whether it may change while a star lives | open | 18 | M0-TC (I8) |
| B30 | How a preset's mass is chosen, and which masses | open | 2, 3 | M0-TC §4, from M0-R2's thresholds (I3) |
| B31 | The real-equivalent translation; the inspector's units | open | 7, 13 | M0-R2/R3 propose, M0-TC fixes (I6) |
| B32 | Gravity's law in a flat world | open — a physics fork | 1, 10 | M0-R2 researches, M0-TC puts it to the lead (I1) |
| B33 | The top speed at 60 frames a second on the mid-range card — reachable? | open — technical | 6, 11 | M0-R3; a shortfall → a M0-TC fork (I4) |
| B34 | A "down" for falling everyday matter, beside the matter's own gravity | open — later | 14, 18 | M0-TZ, with the milestone that brings everyday elements (I7) |
| B35 | The facts the questions stated unverified | open — to verify | 1, 3, 10, 11 | M0-R1, M0-R2 (I11) |

**reports/bootstrap.md §3's rows, resolved:** "chemical reactions …" (open, "M0-TI asks") → later,
with M0 building the mechanism through fusion (15; B19, B23). "The physics the star needs … black
holes and neutrons if a massive star's end is in M0" (MVP, "the subset M0-TI fixes") → all three
endings are in (3), so neutron stars and black holes are in M0's subset; "n-body" there reads as
B15 (12); light reaches the screen as the glow of B12 (9) — whether it also travels as particles
is M0-R2's to propose.

## 5. What the answers change in this plan — for M0-TP
- **F1 · The research was written for real numbers; the lead chose squeezed scale (1).**
  M0-R2's `## Oracles` offers "a published relation (mass–luminosity, main-sequence lifetime), a
  measured value (the Sun's luminosity, radius, age)" — under 1, tests check "each law's exact
  answers, not the real Sun's numbers"; real values serve only the readouts' translation (7).
  M0-R3's title and `## Problem` ("one world from a fraction of a second to billions of years",
  the stages' time scales "against one interactive step") — under 1, the sandbox clock spans slow
  motion to a life in ~10 s (6), and "billions of years" lives in the translated readout only
  (7); the problem becomes the squeeze (I5) and the top speed (I4), and R3's "a settled star
  handed to a reduced model and back" may no longer be needed — R3's to find. R8 stands as
  written; its why ("whose real data are nature's numbers") now reads as the laws' exact answers.
- **F2 · All three endings are in (3).** M0-R2's `## Models` ("what the chosen endings need")
  covers white dwarf, neutron star and black hole alike.
- **F3 · Two edge modes (18).** M0-R2's models cover both edges for the gas and for gravity; M0-TC
  §5's conservation guarantees count what leaves through an open edge.
- **F4 · The performance reference (11).** M0-R1's criterion "GPU compute for millions of cells"
  meets ~240,000 cells in M0 (600 × 400, answer 10) and larger worlds later (B21). If M0-R1
  confirms the Quadro RTX 4000's class (I11), the 60-frames check has its card on linux-pc — the
  GPU-adapter Hazard gains a use: that check runs on the Quadro, named.
- **F5 · No sound in M0 (17).** Rules' "What a capture cannot show here: … sound" may drop sound.
- **F6 · The goal paragraph** in the header predates the interview — §6 proposes its successor.
- **F7 · The recommendations' weight (I12)** — a red-team input, not a defect.

## 6. Proposed for M0-TP — the goal paragraph and the size class
**Goal paragraph (proposed; M0-TP rules on it with the lead):** M0 builds Sandbox Reactions'
engine and its first demo, in sandbox mode, as a desktop app on linux-pc with a tiny web build
kept alive: the player paints hydrogen gas — or drops a ready-made cloud of a chosen mass — into a
Powder-Toy-sized world of about 600 × 400 cells drawn as visible cells that glow when hot, and
watches it live a whole star's life under nature's laws, squeezed to fit one screen and minutes at
normal speed: the cloud collapses under its own gravity, heats up until fusion lights, shines, and
ends as a white dwarf, a supernova leaving a neutron star, or a black hole, as its mass decides. The star can be changed
at any time — gas added or erased, heated or cooled — and every stage comes out of the physics,
never a script. Time runs on fixed speed steps, from about ten times slower than normal to a
Sun-like life in about ten seconds, with pause, single step and an automatic slow-down at big
events; readouts name the stage and translate mass, age and temperature into real astronomy's
units; heat, element and density views and a cell inspector show the physics at work; at the
world's edge, matter leaves for good or bounces back, as the player chooses. It holds 60 frames a
second on a mid-range gaming PC. One reaction mechanism, proven by fusion, waits for the chemistry
to come; more elements, saving, sound and a world size picked by graphics card are routed to later
milestones. (Rests on: reports/bootstrap.md §2 Q2 for the targets; every other clause on a row
of §4.)

**Size class: L, unchanged** — the lead's "Large: engine, time control, star's life
(Recommended)" (reports/bootstrap.md §2, Q5); the MVP above sits inside L's full process. The risk,
once: the MVP is at L's heavy end — three endings that must come out of the physics (3, 4) and
the top speed at 60 frames on a mid-range card (I4). No split proposed: answer 3 put all three
endings in M0, so a split would re-open it (§2.6 P1); if M0-TG's or M0-TB's count calls for one,
§2.5 asks each half to keep its own user gate.
