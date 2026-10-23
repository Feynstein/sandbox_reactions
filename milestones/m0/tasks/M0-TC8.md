# M0-TC8 · compaction of the header — DONE

Ran on linux-pc (laserax-ai), 2026-10-23, model=claude-opus-5-5 level=high (rung_record now, matches the heading).
A first try the same day (the heading then read Haiku 5.5, high) parked the block with numbered options in the chat; the lead re-launched it on
Opus 5.5, high, and the question went again through the question tool.

## The lead's answer
«Superseded to an annex (Recommended)» — shown as: "Move Superseded's 51 entries byte for byte to a new
milestones/m0/m0_superseded.md (binding like m0_rules.md), with a one-line pointer under the plan's Superseded heading, and add
one Rules line saying future mirrors are written there. Header ≈ 525 lines, and later mirrors stop growing it."

## Header before and after (`plan.py lint`)
- Before: `header=705/600` (lines above `# Tasks`, by awk: 702 — Flow 293, Rules 41, Superseded 182, Repo facts 144, Resolve 5).
- After: `header=528/600` · `pipeline=9/60` · checks=675 failed=0 · `=== GO ===`.

## Moves, diffed
| Move | Lines out | Lines in | Check |
|---|---|---|---|
| Superseded's 51 entries (plan lines 368–546) → m0_superseded.md | 179 | 1 pointer | `cmp`: the annex's body == the cut lines; the new plan with the pointer replaced by the cut lines == the plan before (IDENTICAL) |
| R19 index row (plan Rules) + § R19 (m0_rules.md) | 0 | 1 + 10 | new text, the lead's ruling |
| (i) `plan.py stub` ×13 — M0-TC7, M0-V18, M0-D50–D57, M0-TJ2, M0-T123, M0-T125 → plan_archive.md | 416 | stubs | the tool: 416 moved, 416 verified byte-for-byte; M0-D54's commit-gate trailer stays in the plan under its stub (tool WARN) |

Diff count = moved count: 179 + 416 = 595 lines moved, 595 verified.

## The four moves — what each did
- (i) 13 DONE stubs: done (below `# Tasks`; they do not move the header count).
- (ii) Pipeline state: left as is — 9/60 lines; its closed-D entries carry a parenthesis each, rewording them gains no line
  and risks dropping a fact that lives only there.
- (iii) No line repeated in three or more blocks (the lint flags none); Rules is already an index.
- (iv) Repo facts: not de-duplicated — the header is under budget without it; the overlaps measured (the cargo PATH fact in
  Toolchains, Run commands and Hazards; the two Agent lines) point at each other, none dropped.

## Deviations
- The plan edit was a `sed` range-change (one call) plus the `cmp` round trip above, not the file-edit tool: 179 lines of
  exact text through an editor's old-string is the less safe route. Declared.
- PLAYBOOK §2.6 P3 names "the plan's Superseded section"; R19 reads it as m0_superseded.md (the lead's ruling).

## Not acted on
- Repo facts' `Folders` line does not name m0_superseded.md (R19 does); stale "no app yet" / "no game source tree yet"
  facts in Repo facts — a TZ's, not a MOVE's.
