# FilmCraft PR review posting audit

This consolidation covers the 81 original finding records in the A/B/C finding files and review receipts from A, B, C, and root. It is not a deep review of all 754 open PRs.

GitHub API verification confirmed **27 newly posted substantive reviews** against the stored full commit SHA, state, and body (ignoring only a terminal newline). Valid tally: **18 REQUEST_CHANGES** and **9 COMMENT**. No API/head/body mismatches were found. The separate #2348 withdrawal correction is not part of this tally.

| PR | Review state | Review | Inspected commit |
|---:|---|---|---|
| #880 | REQUEST_CHANGES | [5639445214](https://github.com/storytold/filmcraft/pull/880#pullrequestreview-5639445214) | `b4b948f2a319…` |
| #1019 | COMMENT | [5639431674](https://github.com/storytold/filmcraft/pull/1019#pullrequestreview-5639431674) | `7b1f446400b0…` |
| #1021 | COMMENT | [5639431745](https://github.com/storytold/filmcraft/pull/1021#pullrequestreview-5639431745) | `7344ff1b0db3…` |
| #1069 | REQUEST_CHANGES | [5639429621](https://github.com/storytold/filmcraft/pull/1069#pullrequestreview-5639429621) | `2805a14b6388…` |
| #1128 | REQUEST_CHANGES | [5639436213](https://github.com/storytold/filmcraft/pull/1128#pullrequestreview-5639436213) | `394e5e699fd8…` |
| #1400 | REQUEST_CHANGES | [5639430098](https://github.com/storytold/filmcraft/pull/1400#pullrequestreview-5639430098) | `6dfc52eb02bb…` |
| #1647 | COMMENT | [5639429940](https://github.com/storytold/filmcraft/pull/1647#pullrequestreview-5639429940) | `ef802a4ca0e4…` |
| #1775 | REQUEST_CHANGES | [5639430029](https://github.com/storytold/filmcraft/pull/1775#pullrequestreview-5639430029) | `a0b018eab346…` |
| #1814 | REQUEST_CHANGES | [5639430118](https://github.com/storytold/filmcraft/pull/1814#pullrequestreview-5639430118) | `11b799929521…` |
| #2123 | REQUEST_CHANGES | [5639431804](https://github.com/storytold/filmcraft/pull/2123#pullrequestreview-5639431804) | `e63cf15165e8…` |
| #2187 | REQUEST_CHANGES | [5639454123](https://github.com/storytold/filmcraft/pull/2187#pullrequestreview-5639454123) | `1fda744094cb…` |
| #2296 | COMMENT | [5639430988](https://github.com/storytold/filmcraft/pull/2296#pullrequestreview-5639430988) | `404a7b0becff…` |
| #2344 | COMMENT | [5639431605](https://github.com/storytold/filmcraft/pull/2344#pullrequestreview-5639431605) | `b1bc3deceaca…` |
| #2348 | COMMENT | [5639446524](https://github.com/storytold/filmcraft/pull/2348#pullrequestreview-5639446524) | `5925c56f375b…` |
| #2417 | REQUEST_CHANGES | [5639430231](https://github.com/storytold/filmcraft/pull/2417#pullrequestreview-5639430231) | `2f2769792d4b…` |
| #2632 | REQUEST_CHANGES | [5639431433](https://github.com/storytold/filmcraft/pull/2632#pullrequestreview-5639431433) | `f3d5ab664391…` |
| #2802 | REQUEST_CHANGES | [5639430227](https://github.com/storytold/filmcraft/pull/2802#pullrequestreview-5639430227) | `deae2c7f0a4e…` |
| #2826 | REQUEST_CHANGES | [5639430332](https://github.com/storytold/filmcraft/pull/2826#pullrequestreview-5639430332) | `07f71acb41e8…` |
| #2830 | REQUEST_CHANGES | [5639430458](https://github.com/storytold/filmcraft/pull/2830#pullrequestreview-5639430458) | `0491bc315a92…` |
| #2951 | COMMENT | [5639432929](https://github.com/storytold/filmcraft/pull/2951#pullrequestreview-5639432929) | `6d8eb1ba86aa…` |
| #2979 | REQUEST_CHANGES | [5639430586](https://github.com/storytold/filmcraft/pull/2979#pullrequestreview-5639430586) | `6e5447cedff3…` |
| #3017 | COMMENT | [5639430308](https://github.com/storytold/filmcraft/pull/3017#pullrequestreview-5639430308) | `1eb86c4589e3…` |
| #3035 | REQUEST_CHANGES | [5639431283](https://github.com/storytold/filmcraft/pull/3035#pullrequestreview-5639431283) | `12bb6670aba2…` |
| #3145 | REQUEST_CHANGES | [5639431424](https://github.com/storytold/filmcraft/pull/3145#pullrequestreview-5639431424) | `ca17eb8f1593…` |
| #3147 | COMMENT | [5639431517](https://github.com/storytold/filmcraft/pull/3147#pullrequestreview-5639431517) | `6fc5740ec995…` |
| #3148 | REQUEST_CHANGES | [5639431670](https://github.com/storytold/filmcraft/pull/3148#pullrequestreview-5639431670) | `54690756b2e7…` |
| #3152 | REQUEST_CHANGES | [5639431543](https://github.com/storytold/filmcraft/pull/3152#pullrequestreview-5639431543) | `f3544a40bd8f…` |

## Other dispositions

- **51 finding/review records received no new review.** They are itemized in `post-reviews-audit.json`: 28 refer to existing exact-head review coverage: 27 existing CHANGES_REQUESTED reviews (A #1319/#1641/#2230 and 24 B records) plus the existing #2308 COMMENT; others are fixed/resolved/not applicable findings, English-fallback localization policy skips, invalid findings, or scope/owner decisions.
- **24 findings in B** were recorded as already covered by current-head FredApps CHANGES_REQUESTED reviews; **13** were fixed on the inspected head and the remaining B no-new-review cases are separately classified in the JSON.
- The A-slice translation-only findings on #2060 and #2347 were skipped because English fallback is permitted. The #1664 mojibake finding was invalid after rechecking the diff.
- The 20 records in `code-findings-c.json` include scope/feature descriptions and four focused INCLUDE candidates; all are preserved, including no-review records. Review-to-finding links in the JSON join on PR number for bookkeeping and do not claim that one review body answers every distinct finding for that PR.

## #2348 correction

Review [5639437220](https://github.com/storytold/filmcraft/pull/2348#pullrequestreview-5639437220) is a mistaken `REQUEST_CHANGES` recommendation and is **excluded from the valid tally**. It remains displayed as CHANGES_REQUESTED: the review author reports that GitHub returned 404 for both edit and dismissal attempts. The public withdrawal [comment/review 5639454066](https://github.com/storytold/filmcraft/pull/2348#pullrequestreview-5639454066) explicitly retracts the blocker and explains that untranslated strings may use the English fallback. At the user’s request, a later [APPROVED review 5639537972](https://github.com/storytold/filmcraft/pull/2348#pullrequestreview-5639537972), bound to the same current head, explicitly superseded the mistaken change request. The old review remains in the historical record; the latest substantive recommendation is approval.

The full per-PR dispositions, review bodies, source finding inventory, API verification, and correction receipt are in `post-reviews-audit.json`.


