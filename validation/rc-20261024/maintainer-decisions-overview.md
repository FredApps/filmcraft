FilmCraft maintainer decisions and pagination audit
Snapshot: source files were reviewed on 2026-10-24; supplemental GitHub GraphQL page fetched 2026-10-24 16:46:23 UTC. No GitHub writes.

The 212-record decision register contains 212 still-open questions: 210 PR-level decisions plus 2 repository-level gates. Grouped by decision class: Behavior/defaults/compatibility 85; competing implementations/landing order 53; architecture/dependencies/trust 45; scope/issue closure/ownership 19; acceptance/validation 8; repository gates 2.

**Behavior, defaults and compatibility.** Examples: #1924 (interpolation target: clicked keyframe vs all selected), #1629 (error status duration/colour), #1626 (mixed linked/unlinked move behavior). Across the full register, decisions concern editing gestures, defaults, persistence, media-format behavior and backward compatibility.

**Competing implementations and landing order.** Examples: #2153 shadow-lift cap and pure-black treatment coordinated with a competing PR; #2495/#2168/#2178 competing Tools-panel layouts; #1648/#1332 competing Linux NVENC session/probe designs.

**Architecture, dependencies and trust.** Examples: #1115 vendoring a 62k-line patched winit fork; #2152 whether the Premiere fidelity harness may enable unsigned CEP extensions and under what isolation; #2737 Android JNI unsafe-policy exception and shared-storage permissions.

**Scope, issue closure and ownership.** Examples: #2455 whether FCPXML title import satisfies/ closes linked issue or is partial; #2465 whether a four-part remix PR should be accepted bundled or split; #2147 whether remaining macOS dialogs need follow-up due to modal-loop risk.

**Acceptance and validation.** Examples: #1182 whether a new AV1 encoder may be enabled by default before independent quality/decode evidence; #2488/#1575 hardware-encoding rollout gates; #2839 FreeBSD ARM64 release acceptance.

**Repository gates.** Two cross-cutting entries: define approver authority/current-head review freshness; restore/replace missing core CI, require exact-head cargo xtask ci/native lanes, and specify how verified fork runs count.

**Included-pool overlap.** 5 registered open-decision records touch one or more of the current 122 INCLUDEs: four PR-level product/interaction/competing-implementation choices (#1626, #1629, #1924, #2153) and the repository-level approver-authority/current-head rule (references #880, #1652, #2344 and #2365). These are actual owner choices; absence of CI is not itself a product decision, and the widened plan permits CI-only candidates subject to later hosted/native validation.

**Validation-only holds.** Treat missing exact-head checks, missing native/device receipts, or need for hosted CI as validation gaps, not as product decisions. The repository gate remains a cross-cutting merge/release condition; it should be applied to the whole RC manifest rather than converting every candidate into an individual product hold. Linked fork runs must be checked against the exact candidate SHA and platform before being counted.

**Evidence pagination.** Every `pageInfo.hasNextPage` in `pr-review-evidence.json` was false except PR #3017 `reviewThreads`. Its remaining 10 threads were fetched; all 10 are resolved and outdated, with no thread-comment pagination. PR #3017 is already HOLD and outside the current INCLUDE pool. No additional material blocker was found for the remaining 122 INCLUDEs from this pagination pass. The supplied review-evidence file has no check-context connection; consult the separate CI inventory for checks.
