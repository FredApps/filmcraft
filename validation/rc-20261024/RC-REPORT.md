# FilmCraft expanded RC review and build status

The 2026-10-24 16:20 UTC snapshot contained 754 open upstream PRs. The expanded integration contains 116 selected changes, including five selective adaptations; 628 candidates are held and ten are already covered in the combined tree. These are source integration dispositions, not runtime acceptance results.

Source branch: [FredApps/filmcraft — rc/20261024](https://github.com/FredApps/filmcraft/tree/rc/20261024). Version: `0.12.0-rc.20261024.1`. The exact reviewed PR tips, selective integration commits, hold reasons and covered dispositions are in `integration-receipts-expanded.json` and `expanded-integration-plan.json`.

## Scope and review

All 754 candidates were screened using open-state, head, review and discussion evidence. A focused 130-candidate integration pool was crosschecked for concrete correctness defects and unresolved choices. Eight concrete blockers were removed. #2951 was also held because its security change targets an obsolete local Matroska implementation: the actual pinned `craft-media` dependency still needs the recovery budget. Five existing/overlapping changes and five further covered changes are recorded separately; no-op integrations do not inflate the new-change count.

Important exclusions include #1400 (Backspace can delete a clip after effect deletion), #2802 (FCP7 track-limit/data integrity), #2826 (IME release behavior), #2830 (TTML parsing/data loss), #2979 (ripple-delete playhead), #3035 (non-macOS audio callback compile mismatch), #3145 (TTML boundary line-break loss), and #3152 (incomplete per-channel non-finite recovery). Additional holds include competing implementations, drafts, public API compatibility choices, platform evidence and unreviewed scope; a hold is not automatically a confirmed bug.

The selected #3148 paired FFT received an RC correction that replaces non-finite input with silence within its own channel before the shared FFT. Its new regression compares NaN and both infinities against zero-replacement controls across both members of a pair, the odd channel and hop boundaries. #2123 received a system Lao font fallback and a language-selection/saved-preference guard. #2539's actual half-aware pan feature was adapted, and #2677's cancellation regression was adapted to the resulting separate scroll model. #2153 uses the reviewed 0.02 Lumetri floor consistently across CPU and GPU; direct Premiere frame comparison remains unexecuted.

Held commits that were integrated early remain in Git history with targeted corrective reversals. An ancestor check alone therefore does not prove a held change ships. The receipt identifies these cases and selective adaptations.

## Validation and online build

Trusted source formatting and static inspection were completed, including conflict delimiter repairs, test-module relocation, RC workspace version checks, lockfile reference repairs, catalog key checks and exact named test/package mapping against the integrated source. The trusted Cargo resolver also successfully refreshed only workspace dependencies from the pre-upgrade lock: six required packages changed for cpal 0.18.2, and unrelated dependency versions were retained. No candidate code, build script, test or binary was built or executed on the operator machine.

The dedicated fork workflow now builds **Windows x64 only**, per the user's updated scope. It runs formatting, Clippy, workspace tests, doctests, dependency-layer and asset-attribution checks, then creates MSI and portable ZIP packages. Artifacts carry exact source/tree, lock/toolchain/font evidence, notices, hashes and the acceptance kit. Windows packages are unsigned. Linux and macOS jobs have been removed.

GitHub Actions access was restored on 2026-10-24. The earlier disabled-state diagnostics are retained as historical evidence; no Support action is currently required. The first Windows run (40119629548) passed formatting and then found a duplicate AV1 helper from overlapping integrations. The duplicate was consolidated, retaining the bounded helper and regression test. Build/test/package acceptance remains pending the subsequent hosted run.

To dispatch the Windows-only branch:

```powershell
gh workflow run rc-candidate.yml -R FredApps/filmcraft --ref rc/20261024
gh run list -R FredApps/filmcraft --branch rc/20261024
```

The next required work is to resolve any hosted failures, obtain successful packages, verify downloaded artifact hashes and binary metadata as data, then execute Windows native graphical/hardware/parity cases. No tests, packages, graphics results or parity results are claimed to have passed.

## Acceptance and maintainer decisions

`TESTING-PROTOCOL.md`, `case-index.json`, `results-template.json` and `testledger.json` provide 116 active PR cases. The kit records inspected PR tips separately from the final artifact source SHA, requires named test discovery/execution with nonzero matches, and separates hosted, native visual and Premiere parity results. NVENC hardware is not available on stock hosted Windows; that lane remains `NOT_RUN`. All hosted and manual results currently remain `NOT_RUN`.

`MAINTAINER-DECISIONS.md` retains the 212-record decision register, with changed-head re-review caveats. `included-register-overlap-review.md` explains the RC choices for #1626, #1629, #1924 and #2153. The upstream approval/merge policy and acceptance of fork CI receipts remain maintainer decisions. No upstream merge or published release was made.
