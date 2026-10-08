# M0-R2b — detail (2026-10-08, 10:32–10:47 America/Toronto, linux-pc)

- **Start read.** The grep of the heading (`· switch` present), then `python3
  tools/pb/rung_record.py now` and `python3 tools/pb/plan.py show M0-R2b` — both NOT RUN (no
  tools/pb/ yet; M0-TH extracts it). The plan header and this block read with the file tool, then
  the `Read:` line: reports/sandbox_interview.md §1, §3, §4; reports/star_physics.md (whole);
  reports/engine_stack.md `## Recommendation` and `## Ruling`.
- **Delivered.** reports/sim_models.md: `## Evidence` (31 dated sources, E1–E31; local checks
  C1–C3), `## Models` (M1 representation, M2 gravity and B32's fork, M3 gas, M4 heat and light,
  M5 burning and the reaction registry, M6 degeneracy and the three endings, M7 the edge modes,
  M8 cost; a worked example), `## Oracles` (O1–O16 with sources and proposed tolerances),
  `## UNVERIFIED`.
- **Sources, by weight.** Pols' Utrecht notes ch. 5–6 and 12–13 (PDFs downloaded to the session
  scratchpad, read with pdftotext); Chavanis 2007 (D-dimensional white dwarfs, ar5iv); Roshan et al.
  2016 (the Maclaurin disk's Ω², checked in the PDF's eq. 9); Chira et al. 2018 (Ostriker's line
  mass, checked in the PDF's eq. 6); Vaidya et al. 2017 (RKL2), Turner & Stone 2001 (FLD), Myers et
  al. 2013 (Truelove), Federrath et al. 2010 (sinks), Zou et al. 2021 (Hockney–Eastwood), Wang et al.
  2016 (thin-disk gravity), McDonald & Zijlstra 2015 (Reimers); Wikipedia for textbook laws.
- **Local checks** (stdlib Python; numpy absent and an install is not this task's): C1 the
  Maclaurin disk's force on a grid, worst interior error 1.0–1.6 % at a = 40 cells, 2.0–3.3 % at
  a = 20 (first order), softening worse; C2 the free-fall formula to 2 × 10⁻⁷; C3 a uniform disk's W
  to −2.4 %; the worked example's arithmetic. All in logs/M0-R2b.log.
- **Verify 1.** [ALREADY RUN — PASS (GO, 3 sections, 31 dated sources) on linux-pc]; red arm on a
  scratch copy with `## Oracles` un-headed → `=== NO-GO: missing sections ===`, exit 1. Both in
  logs/M0-R2b.log. `verify.py --redarm` / `--changed`: NOT RUN (no verify.py yet — M0-TH).
- **Adversarial line, applied.** Every recommendation states its stability condition and its cost
  at 600 × 400; one star is worked by hand (sim_models.md, "Worked by hand"): CFL 0.0103, explicit
  heat fine in the core (0.0171) but 60 sub-steps at the surface, RKL2 s = 16; and the top speed
  comes out over budget on this arithmetic.
- **Findings for later tasks (flagged into their Carried flags).**
  - M0-R3: steps per life ≈ (5–10 × R/Δx) × the K1 gaps; ~25 steps per frame × ~0.88 ms ≈ 22 ms
    against 16.6 ms — measure it; levers listed.
  - M0-TC: B32's fork (recommended: the 3D law in a flat sheet); the oracles and tolerances for R8;
    two small forks (light leaving in bounce mode, the surface-temperature readout); the model
    risks.
  - M0-TZ: the lead's game-picker ask for the website, and space_tykun's M1-R2.
- **Carried flags answered.** (a) The maximum mass: none in 2D log gravity (critical γ = 1, E4);
  yes in the sheet (critical γ = 3/2, the 2D ultra-relativistic Fermi gas) — my arithmetic,
  UNVERIFIED, O9 checks it. (b) Shell ejection: radiation force κF/c from the heat solver,
  Reimers-shaped wind as fallback; risk kept UNVERIFIED. (c) The neutrino sink: ε_ν ∝ T^m below
  the burning exponent, Pols' mechanism (E24, E20). (d) The K1 gap: oracle O14, and it sets the
  steps per life (to M0-R3).
- **Mid-session ask from the lead** (2026-10-08, verbatim in M0-TZ's flag): a game picker on the
  website, space_tykun among the games. Done at their request: `M1-R2` (web build research) filed in
  `../space_tykun/milestones/m1/m1_implementation_plan.md` through its `plan.py append --after
  M1-T23` (the tool first needed an `R` counter: `plan.py register --set "Counters=… · R=1"`, which
  the append raised to 2); `plan.py lint` GO, header 574/600; committed and pushed as 727dca8 on
  `origin/main` — the lead asked for the commit and the push.
- **Deviations.** plan.py and rung_record.py absent here — the plan edited by hand. My first
  status edit used a short Python script over the plan; it shifted an offset and broke this
  block's heading for one edit — repaired with the file-edit tool at once, and every later edit
  was made by hand (an ad-hoc script over the plan is a defect, §C).
- **Model and level:** NOT RUN (no rung_record.py yet).
