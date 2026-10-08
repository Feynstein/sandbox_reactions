# M0-R2a — detail (2026-10-08, 10:14–10:24 America/Toronto, linux-pc)

- **Start read.** The grep of the heading (`· switch` present), then `python3
  tools/pb/rung_record.py now` and `python3 tools/pb/plan.py show M0-R2a` — both NOT RUN (no
  tools/pb/ yet; M0-TH extracts it). The plan header and this block read with the file tool, then
  the `Read:` line: reports/sandbox_interview.md §1–§4, reports/engine_stack.md `## Recommendation`
  and `## Ruling`.
- **Delivered.** reports/star_physics.md: `## Evidence` (25 dated sources, E1–E25), `## Stages`
  (S1–S8 common and white-dwarf path, S5′–S8′ supernova path, S7″–S8″ black-hole path; the ending
  thresholds table; I11's four facts checked), `## Elements` (9 + optional Ni-56), `## Squeeze`
  (K1–K9 orders and ratios; the translation proposed for mass, temperature, age), `## UNVERIFIED`.
- **Sources, by weight.** Pols' Utrecht lecture notes (ch. 1–4, 5–6, 9–11, 12–13 — downloaded as
  PDFs into the session scratchpad and read with pdftotext; quotes checked against the text, not a
  summary), Heger et al. 2003 (PDF read the same way), Sukhbold et al. 2016 (abstract), the NASA Sun
  fact sheet, OpenStax Astronomy 2e, and Wikipedia pages for the rest. The Woosley & Janka 2005
  25-Sun table is cited as Wikipedia carries it (E7), not from the paper.
- **Verify 1.** [ALREADY RUN — PASS (GO, 4 sections, 25 dated sources) on linux-pc]; red arm on a
  scratch copy with `## Squeeze` un-headed → `=== NO-GO: missing sections ===`, exit 1. Both in
  logs/M0-R2a.log. Not `verify.py --redarm` / `--changed`: NOT RUN (no verify.py yet — M0-TH).
- **Adversarial line, applied.** `## Squeeze`'s invariants K1–K9 carry orders and ratios only; three
  real values that slipped in on the first draft (a neutrino-onset temperature, energies per gram,
  the two remnant ceilings) were rewritten as ratios. Real values appear in the translation bullets
  only, labelled as anchors.
- **Findings for later tasks (flagged into their Carried flags).**
  - M0-R2b: (1) the endings need a pressure that holds a cold core up to a maximum mass, twice
    (electron, then neutron degeneracy) — whether B32's flat-world gravity still yields a maximum
    mass is unsourced; (2) the planetary-nebula ejection (S7) is wind-driven in nature — a model risk
    for B4 if gas + gravity + heat alone cannot puff a shell off; (3) K2's late-stage speed-up comes
    from a neutrino sink steeper in T than photon losses; (4) the K1 gap size is unmeasured.
  - M0-R3: cloud to white dwarf is ~11 Gyr (main sequence 9–10 + ~2 subgiant/RGB + ~0.12 He
    burning; my arithmetic from E2), so B9's "life in ~10 s" squeezes ~11 Gyr; the age readout
    is proposed as a stage clock, not one factor.
  - M0-TC: (1) a cloud under the no-ignition threshold becomes a failed star that cools — an
    outcome the physics will show though not one of the lead's three endings (a label and whether a
    preset may sit there are TC's); (2) the real black-hole boundary is soft and non-monotonic
    (E3, E6); the report keeps one threshold as "as its mass decides" — TC confirms; (3) the
    translation proposals (B31): one factor for mass anchored on the white-dwarf ceiling, one factor
    or log-anchors for temperature, a stage clock for age.
- **Not acted on.** The helium flash's duration conflicts across two Wikipedia pages (seconds vs
  days) — recorded in UNVERIFIED; nothing in M0 rests on it. Red dwarfs below ~0.5 Suns outlive the
  universe in nature (E10) yet the squeezed sandbox will play them out — a presentation question,
  left for TC with the failed-star flag (not flagged separately: same decision surface).
- **Scratch.** /tmp/claude-1000/-home-ybelanger-private-sandbox-reactions/0124cbbf-ec1d-4c73-95a3-1210cc731603/scratchpad
  (pols/, heger/, redarm/) — downloaded PDFs and their text; nothing copied into the repo.
- **Model and level.** NOT RUN (no rung_record.py yet).
