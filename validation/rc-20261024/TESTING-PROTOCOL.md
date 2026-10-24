# FilmCraft RC graphical and behavior test protocol

## Purpose and execution boundary

RC version: `0.12.0-rc.20261024.1`; candidate SHA comes from the exact artifact `source-commit.txt` or equivalent build receipt until the release owner fixes the manifest. All cases start `NOT_RUN`. Run only on hosted CI, explicitly qualified VM, or native QA machine after exact artifact provenance is recorded. Never build, compile, or execute candidate PR code on an analyst workstation.

Premiere Pro 2026 v26.5.2 is the behavioral/export reference. Use only original synthetic fixtures. Adobe interaction is manual and local; keep notes in ignored `plan/premiere/`. Do not collect Adobe screenshots, logs, proprietary projects/media, inspect bundles, or put reference data in repo/CI. Never enable or recommend unsigned CEP extensions.

The final 116-case manifest is `case-index.json`. Each `reviewed_pr_head` is the inspected PR tip, not the final build SHA. Fill `integrated_candidate_sha` from the exact artifact `source-commit.txt` or build receipt after integration. Detailed triggers, expected observables, controls, tolerances, relevant added/changed tests, hosted commands and undo/reopen instructions are in `results-template.json`. Tests added by a diff are kept distinct from source-existing tests. `testledger.json` tracks integration verification and separate `hosted_smoke`, `native_visual`, `premiere_parity` statuses. Eight QC holds and 11 earlier cases outside the active set are documented as excluded.

## Provenance and run record

For every attempt capture PR, full candidate and base SHAs, workflow URL/run ID, artifact URL and SHA-256, signature identity if present, invocation/profile/features, Rust/toolchain and lock hash, OS/version/architecture, GPU/device/driver/API/backend, display scale/color profile, audio device/sample rate, locale, fixture recipe/version/hash, operator and UTC start/end. Verify artifact hash against hosted manifest before launch. No `latest` links. Preserve failures and sanitized raw logs; reruns are new attempts with unique IDs.

## Platform matrix

| Track | Windows x64 | Linux x64 | macOS arm64/x64 |
|---|---|---|---|
| Hosted smoke | Exact-SHA artifact; startup, fixture import, render/export, save/reopen | Same where artifact exists | Same where artifact exists |
| Native visual | Qualified QA GPU/display; default, CPU, DX12 and Vulkan when supported | Qualified GPU/display; default, CPU, Vulkan when supported | Qualified Apple GPU/display; default, CPU, Metal when supported |
| Premiere parity | Manual local comparison when installed | NOT_RUN unless qualified local installation exists | NOT_RUN unless qualified local installation exists |

Keep statuses separate. This RC builds and qualifies Windows x64 only. Linux/macOS rows are historical reference lanes outside the current build scope; mark them `NOT_APPLICABLE` for this attempt, never pass. Compare default, forced CPU and available explicit compositor backends; collect FilmCraft-only UI and backend logs.

## Original neutral fixtures

Generate only on hosted runners, qualified VM, or native QA machine. Keep deterministic recipes, generator version, parameters, and SHA-256; do not commit media. Use original color bars and gray/color patches, alpha ramp/checker, short synthetic clips, tone/silence/impulse WAVs, multi-track timeline, captions with punctuation/whitespace, and minimal interchange projects authored from scratch. Use local Premiere with the same synthetic fixture as a manual oracle. No proprietary/copyrighted assets or Adobe samples.

## Graphical workflow

For each applicable PR: start a clean disposable profile/project; import generated fixture; exercise the case-specific path; inspect viewer/timeline/effect/caption/export UI; render/export as applicable; compare base and candidate on identical machine/settings/fixture; save, close, reopen, and verify state/references. For edits, verify one undo and redo where applicable. Record exact measurements. Apply source-defined tolerances only. If none exists, record baseline first and obtain an acceptance limit before pass/fail. Run each exact test command from `test_command_map` in hosted CI. Each filter is mapped to the file defining that test and that crate’s package; run its `discovery_command` first, then its `filter_command`. Accept coverage only if output lists the named test and reports at least one test executed and passed. A zero-match successful command is NOT coverage. If no named test is present, use the affected hosted package/workflow and require a nonzero executed-test count. Never invent a gate.

For DSP optimization and recovery cases, pair deterministic finite-input numerical regression with GUI render/listen evidence. NaN/Inf injection belongs only in a named hosted/unit harness that supports it; do not present it as a normal GUI action.

## Acceptance cases

There are 116 integrated PR cases, each bound to `reviewed_pr_head`; integration dependencies may reduce the pool. Use the corresponding `results-template.json` record for concrete trigger, observable, controls, test commands, tolerance and undo/reopen. Bind `integrated_candidate_sha` from the exact final artifact receipt. The records are intentionally NOT_RUN; this protocol makes no claim that any PR has been tested.

## Results and excluded earlier cases

`results-template.json` and `testledger.json` are separate results artifacts. Use `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`, or `NOT_APPLICABLE` per lane and preserve attempts/evidence. A missing lane, stale approval, bot status, or hosted smoke alone is not a pass. The 11 earlier 30-case entries absent from the finalized manifest are marked `EXCLUDED_FROM_ACTIVE_RC_ACCEPTANCE`; they do not count as active coverage unless the release owner revises the manifest.

## Final integration scope

116 selected PR changes ship, including five selective adaptations recorded in integration-receipts-expanded.json. #2951 is excluded because its security fix targets a replaced Matroska implementation; #3077 is covered by current playback cleanup. The #3148 paired FFT and #2123 Lao interface have additional RC integration corrections and regressions. All native visual, Premiere and hardware results remain NOT_RUN until executed.

Four no-op integrations (#2033, #2909, #2947, #2964) are covered controls, not new changes. The final accounting is 116 integrated / 628 held / 10 covered. GitHub Actions was re-enabled and the Windows-only build is running. The earlier dispatch rejection is historical; hosted results must be filled from the completed exact-SHA run. Every manual result remains NOT_RUN.
