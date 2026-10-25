# FilmCraft expanded RC review and build status

The 2026-10-24 16:20 UTC snapshot contained 754 open upstream PRs. The expanded integration contains 116 selected changes, including five selective adaptations; 628 candidates are held and ten are already covered in the combined tree. These are source integration dispositions, not runtime acceptance results.

Source branch: [FredApps/filmcraft â€” rc/20261024](https://github.com/FredApps/filmcraft/tree/rc/20261024). Version: `0.12.0-rc.20261024.1`. The exact reviewed PR tips, selective integration commits, hold reasons and covered dispositions are in `integration-receipts-expanded.json` and `expanded-integration-plan.json`.

## Scope and review

All 754 candidates were screened using open-state, head, review and discussion evidence. A focused 130-candidate integration pool was crosschecked for concrete correctness defects and unresolved choices. Eight concrete blockers were removed. #2951 was also held because its security change targets an obsolete local Matroska implementation: the actual pinned `craft-media` dependency still needs the recovery budget. Five existing/overlapping changes and five further covered changes are recorded separately; no-op integrations do not inflate the new-change count.

Important exclusions include #1400 (Backspace can delete a clip after effect deletion), #2802 (FCP7 track-limit/data integrity), #2826 (IME release behavior), #2830 (TTML parsing/data loss), #2979 (ripple-delete playhead), #3035 (non-macOS audio callback compile mismatch), #3145 (TTML boundary line-break loss), and #3152 (persistent non-finite DeReverb state). Additional holds include competing implementations, drafts, public API compatibility choices, platform evidence and unreviewed scope; a hold is not automatically a confirmed bug.

The selected #3148 paired FFT received an RC correction that replaces non-finite input with silence within its own channel before the shared FFT. Its new regression compares NaN and both infinities against zero-replacement controls across both members of a pair, the odd channel and hop boundaries. #2123 received a system Lao font fallback and a language-selection/saved-preference guard. #2539's actual half-aware pan feature was adapted, and #2677's cancellation regression was adapted to the resulting separate scroll model. #2153 uses the reviewed 0.02 Lumetri floor consistently across CPU and GPU; direct Premiere frame comparison remains unexecuted.

Held commits that were integrated early remain in Git history with targeted corrective reversals. An ancestor check alone therefore does not prove a held change ships. The receipt identifies these cases and selective adaptations.

## Validation and online build

Trusted source formatting and static inspection were completed, including conflict delimiter repairs, test-module relocation, RC workspace version checks, lockfile reference repairs, catalog key checks and exact named test/package mapping against the integrated source. The trusted Cargo resolver also successfully refreshed only workspace dependencies from the pre-upgrade lock: six required packages changed for cpal 0.18.2, and unrelated dependency versions were retained. No candidate code, build script, test or binary was built or executed on the operator machine.

The dedicated fork workflow now builds **Windows x64 only**, per the user's updated scope. It runs formatting, Clippy, workspace tests, doctests, dependency-layer and asset-attribution checks, then creates MSI and portable ZIP packages. Artifacts carry exact source/tree, lock/toolchain/font evidence, notices, hashes and the acceptance kit. Windows packages are unsigned. Linux and macOS jobs have been removed.

GitHub Actions access was restored on 2026-10-24. The earlier disabled-state diagnostics are historical; no Support action is currently required. Run 40122117619 passed formatting and Clippy but failed 13 workspace test targets before packaging. Subsequent fixes restore preview classification for resolved default Motion coordinates, report FCPXML cycles while preserving reserved sequence identities, correct clip-dialog parameter metadata and UI control-channel use, preserve exact-selection delete semantics, and repair oracle fixture assumptions. The AV1 panic test retains the actual caught error assertion.

The workflow now pins and hashes a complete FFmpeg oracle build and checks its required codecs before tests. Focused GPU diagnostics from run 40133603569 identified DX12 device loss on the stock Microsoft Basic Render Driver, with small buffers inside device limits. The next runner configuration installs the same WARP 1.0.20 test runtime used by wgpu's own CI, beside hosted test executables only. Required GPU coverage fails on missing-adapter skips. No WARP DLL or external oracle is included in the release packages. Run 40136934842 executed all 47 GPU library tests successfully on pinned WARP, with zero failures or skips. The subsequent packaging correction adds project and embedded-font notices to the MSI payload, matching the portable ZIP. Full build/test/package acceptance remains pending a successful exact-SHA run.

To dispatch the Windows-only branch:

```powershell
gh workflow run rc-candidate.yml -R FredApps/filmcraft --ref rc/20261024
gh run list -R FredApps/filmcraft --branch rc/20261024
```

The next required work is to resolve any hosted failures, obtain successful packages, verify downloaded artifact hashes and binary metadata as data, then execute Windows native graphical/hardware/parity cases. No tests, packages, graphics results or parity results are claimed to have passed.

## Acceptance and maintainer decisions

`TESTING-PROTOCOL.md`, `case-index.json`, `results-template.json` and `testledger.json` provide 116 active PR cases. The kit records inspected PR tips separately from the final artifact source SHA, requires named test discovery/execution with nonzero matches, and separates hosted, native visual and Premiere parity results. NVENC hardware is not available on stock hosted Windows; that lane remains `NOT_RUN`. All hosted and manual results currently remain `NOT_RUN`.

`MAINTAINER-DECISIONS.md` retains the 212-record decision register, with changed-head re-review caveats. `included-register-overlap-review.md` explains the RC choices for #1626, #1629, #1924 and #2153. The upstream approval/merge policy and acceptance of fork CI receipts remain maintainer decisions. No upstream merge or published release was made.

## Additional hosted regression repair

Run 40140677286 at source 319584296ffe9035741ca75391f095193990c561 passed formatting, native Clippy, and all 47 required GPU library tests (zero failures or skips). The full workspace test step reported 3,906 passes and seven failures across five targets; doctests and packaging were consequently not reached. The remaining failures were DNx HQX decode/encode oracle comparisons, duplicate-selection ripple-delete fixture sync-lock setup, FCPXML self-reference preservation, MP4 edit-list/padding oracle expectations, and the Lao Sequence menu term. This failed run is not an accepted RC.

The follow-up retains strict DNx numeric tolerances, corrects the CID 1271 weight-table assignment, preserves FCPXML self-references while emitting a cycle warning, and isolates selection deduplication from the separate sync-lock refusal contract. DNx and MP4 oracle regressions now run before the expensive full workspace compilation, with a black-box CID 1271 weight probe retained in the logs. Exact successful follow-up evidence remains required before package acceptance.

Run 40146002238 confirmed the CID 1271 external weight probe matches D.1, and all six active DNx integration tests passed with the original strict tolerances. MP4 cut-edit packet comparison and the explicit AAC presentation-duration regression passed; only the convenience AAC expectation failed because it omitted integer rounding through the default movie timescale (96,256 media ticks → 2,005 movie ticks → 96,240 reported media ticks). The next repair uses that documented conversion in the expected value. Early failure also now skips GPU-log publication when the GPU test step did not run.
