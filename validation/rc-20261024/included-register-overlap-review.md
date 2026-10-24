# Included-pool register overlap review

Snapshot: 2026-10-24 UTC. Current plan at review time: 122 included, 627 held, 5 covered. These four register overlaps do not justify reinstating blanket holds.

- **#1626 — INCLUDE.** The exact-head reviewer found no defect; the only behavior note is that an unlinked opposite-kind clip in a mixed selection stays on its track. Linked audio/video remains synchronized while the partner keeps its track. Open timeline PRs edit nearby code but are distinct features, so resolve/rebase them without discarding this rule. Adobe documents linked selection/movement and sync semantics, but not this mixed-selection corner.
- **#1629 — INCLUDE.** The change deliberately shows all failed/refused commands, including routine “nothing selected” and locked-track cases, in danger styling for 20 seconds. The exact-head reviewer found no defect; the large mechanical patch may conflict textually but is not a competing UX implementation or code blocker.
- **#1924 — INCLUDE.** Selected-key interpolation is atomic, bounded, and one undo step; the existing single-key command remains compatible. The reviewer says the issue is fixed and clicked-only Clear is reasonable as a separate scope.
- **#2153 — INCLUDE #2153 only; exclude #2140.** The current soft-floor algorithm preserves output at/above 0.02 and intentionally changes near-black output; current PR discussion contains golden and GPU evidence. #2140 is still open and overlaps the same three files with a different hard cap and black behavior. Select one implementation to avoid a competing stack. Evidence is not a direct Premiere comparison: Adobe’s public docs explain control semantics, not its numeric output. The pure-black choice therefore remains an explicit behavior note, but no clear divergence is established for this broad RC experiment.

Current-head evidence and links are in `included-register-overlap-review.json`.
