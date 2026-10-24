# FilmCraft maintainer decision register

The register retains 212 decision records from the prior audit, reconciled against the 754-open-PR snapshot of 2026-10-24 16:20 UTC. Open PR status does not prove an old question remains unresolved at a changed head; entries flagged for re-review require current-head confirmation. This register is separate from the RC inclusion manifest.

The RC uses #1626, #1629, #1924 and #2153 after a fresh review; #2140 is excluded as the competing color implementation. Details are in included-register-overlap-review.md. The fork workflow supplies RC validation; upstream policy on accepting fork receipts and approval authority remains a maintainer choice.

## Repository gates

### D001 — Repository-wide: Repository core CI gate

Restore or replace the missing core CI workflow, then define which exact-head checks and fork-run receipts are required before merge.

**Options:** Require cargo xtask ci and relevant native lanes; specify approval of fork runs and a baseline-lint remediation order (#2007/#2370).

**Recommendation:** Restore the core gate and unblock baseline failures before bulk merging; accept independently verified exact-head fork runs with their platform limits recorded.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/tree/74242b51d675514da12d9bbf6b331ef2e6399211/.github/workflows). Confidence: high; current-head re-review: True.

### D002 — Repository-wide: Approval freshness and authority

Specify who can authorize merging and require review of the current commit when an approved PR changes.

**Options:** Renew stale approvals or explicitly approve a narrowly scoped post-review delta; do not infer merge authorization from contributor approval events alone.

**Recommendation:** Require current-head sign-off and preserve the distinction between human review, bot success, platform tests and full CI.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pulls). Confidence: high; current-head re-review: True.

## Architecture, dependencies and trust

### D003 — [PR #2821](https://github.com/storytold/filmcraft/pull/2821): Export: chapter markers are written into MP4 and MOV (#2747)

Approve writing both QuickTime chapter tracks and Nero chpl metadata, including fragmented MP4 limitation.

**Options:** Write both formats for broad player compatibility; choose one representation; define behavior for fragmented exports.

**Recommendation:** Defer to maintainers; evidence does not establish a settled choice.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2821). Confidence: medium; current-head re-review: None.

### D004 — [PR #2737](https://github.com/storytold/filmcraft/pull/2737): android: the NativeActivity entry crate and the shared-storage access module

Choose Android shared-storage permission policy.

**Options:** Use MANAGE_EXTERNAL_STORAGE via explicit system grant screen; or omit All-files access in v1 and use scoped READ_MEDIA_* permissions, retaining private storage.

**Recommendation:** Prefer scoped media permissions for a media editor unless all-files access is required for documented workflows; confirm target distribution policy before retaining broad access.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2737). Confidence: high; current-head re-review: None.

### D005 — [PR #2737](https://github.com/storytold/filmcraft/pull/2737): android: the NativeActivity entry crate and the shared-storage access module

Choose the standing unsafe-policy exception for the Android JNI adapter.

**Options:** Allow `apps/filmcraft-android/src/android_access.rs` as a second named exception under the same safety contract as platform; or require narrower per-file/two-floor allowlisting.

**Recommendation:** If Android is in scope, allow only the single adapter module with the stated safe Result API and documented unsafe blocks; record that exact rule in AGENTS.md.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2737). Confidence: high; current-head re-review: None.

### D006 — [PR #2735](https://github.com/storytold/filmcraft/pull/2735): android: vendored winit with the Android mouse input filled in

Approve carrying the workspace-wide vendored winit patch for Android, its maintenance owner, and the Android platform rollout.

**Options:** Carry a pinned/reproducible patch with desktop validation and an exit plan; isolate the patch to a separate Android build; or defer the target.

**Recommendation:** Decide the platform and vendoring policy explicitly before landing the Android stack; the split alone does not authorize the dependency fork.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2083#pullrequestreview-5571719224). Confidence: high; current-head re-review: None.

### D007 — [PR #2725](https://github.com/storytold/filmcraft/pull/2725): #2714: native Wayland file drops; Project panel drops go into the bin under the pointer

Should platform unsafe be expanded beyond media FFI for native Wayland DnD, with ADR 0002 covering Wayland and existing Windows cursor FFI?

**Options:** Approve proposed platform window-system FFI exception and lifecycle rules; reject it and retain Xwayland fallback.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2725). Confidence: high; current-head re-review: None.

### D008 — [PR #2657](https://github.com/storytold/filmcraft/pull/2657): #30: VideoToolbox ProRes, Motion JPEG, VP9, AV1 and HEVC 10-bit / 4:2:2 on macOS

Decide VideoToolbox retry budget and recovery contract when driver failure is irrecoverable; decide integration base/landing sequence for 4:2:2 support.

**Options:** Approve bounded retries and defined recovery semantics; choose failure/fallback behavior; agree on shared API/base before integration while preserving NV16/P210 paths.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2657). Confidence: high; current-head re-review: None.

### D009 — [PR #2649](https://github.com/storytold/filmcraft/pull/2649): feat: built-in music generation (Stable Audio 3 + hosted providers)

Should the local music backend download model and auxiliary files at runtime, after consent and hash/size validation?

**Options:** Permit the described runtime download flow; or require bundled/local-only model/runtime assets.

**Recommendation:** Explicitly approve the user-triggered, license-confirmed, pinned model download contract and runtime loading path, or specify a local-only policy.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2649#pullrequestreview-5602864417). Confidence: medium; current-head re-review: True.

### D010 — [PR #2507](https://github.com/storytold/filmcraft/pull/2507): G19: speech-to-text (Whisper) in release builds, with a model download dialog

Should Whisper be enabled by default in release builds, with model download at runtime?

**Options:** Ship default feature and user-confirmed model download; or keep Whisper opt-in / remove runtime download.

**Recommendation:** Decide based on release size/build cost and download policy; verify the consent, license, source and integrity protections in the proposed flow.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2507#pullrequestreview-5586150466). Confidence: high; current-head re-review: None.

### D011 — [PR #2407](https://github.com/storytold/filmcraft/pull/2407): Playback: Linux audio through PipeWire/PulseAudio's native protocol, with reconnects and the ALSA fallback (#106, #2117; part 2 of 2, on #2644)

Should PulseAudio/PipeWire native protocol become Linux default when a socket is reachable, and what channel-count/fallback behavior is required?

**Options:** Roll out by default with robust multichannel and fallback; make opt-in; retain ALSA/cpal default until real-server evidence.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2407). Confidence: high; current-head re-review: None.

### D012 — [PR #2389](https://github.com/storytold/filmcraft/pull/2389): #1878: list fonts activated through Adobe Fonts

Should extensionless-font detection apply to every scanned system font directory or only Adobe CoreSync folders?

**Options:** Keep broad signature detection in all scanned roots; or scope it to Adobe directories.

**Recommendation:** Confirm whether the wider acceptance of extensionless font-tagged files on Windows/Linux is wanted; reviewer calls it harmless and probably wanted, so this is nonblocking.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2389#pullrequestreview-5571436236). Confidence: medium; current-head re-review: None.

### D013 — [PR #2332](https://github.com/storytold/filmcraft/pull/2332): automation: every control on screen has an id; `ui.coverage`; `mcp --launch-app`

Should the automation CLI expose --launch-app to start a program, and should it be split from automation IDs/coverage?

**Options:** Approve launch capability with security boundary; reject/defer it; split IDs and coverage from launch behavior.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2332). Confidence: high; current-head re-review: True.

### D014 — [PR #2316](https://github.com/storytold/filmcraft/pull/2316): Add Move Playhead to Cursor shortcut

Should menus-only German/Russian/Ukrainian catalogs accept a new non-menu command label?

**Options:** Keep those catalogs menus-only and translate the command only in the eight full catalogs; or broaden the catalog policy and adjust its guard tests.

**Recommendation:** Do not weaken the menus-only contract for a single command; if broader coverage is desired, make it a repository-wide localization policy decision.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2316#pullrequestreview-5571816759). Confidence: high; current-head re-review: True.

### D015 — [PR #2312](https://github.com/storytold/filmcraft/pull/2312): Premiere: import .prproj projects with source provenance

Should Premiere provenance be stored in a new project workflow tag that older builds refuse, or be omitted/encoded compatibly?

**Options:** Accept new `filmcraft.project.workflow` and version compatibility break; or omit/change provenance representation to preserve older-build readability.

**Recommendation:** Decide the project compatibility policy before merge and document the consequences.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2312#pullrequestreview-5571817760). Confidence: high; current-head re-review: True.

### D016 — [PR #2300](https://github.com/storytold/filmcraft/pull/2300): fix(fcpxml): import and export adjust-conform (fit by default) (#2218)

Should FCPXML import support adjust-conform=fill by adding a clip model/project-format/render field, or report it as fit with a warning?

**Options:** Add full fill-mode support; or preserve current fit behavior and warn on import.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2300). Confidence: high; current-head re-review: None.

### D017 — [PR #2178](https://github.com/storytold/filmcraft/pull/2178): Dock: dividers shrink-then-push in any panel arrangement

Choose the dock resize model and approve its workspace-wide minimum sizes; also split the independent Tools reflow feature.

**Options:** Accept flatten/write-back shrink-then-push model and 40/221/48 pt minimums, or retain per-split resizing; choose whether Tools reflow belongs separately.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2178). Confidence: high; current-head re-review: None.

### D018 — [PR #2163](https://github.com/storytold/filmcraft/pull/2163): Integrate CollabCraft cloud sync for FilmCraft

Should FilmCraft ship CollabCraft cloud sync at all, and if so what default/service contract should it use?

**Options:** Accept cloud sync with UI/network disabled unless an HTTPS service URL is configured; or defer/reject cloud sync pending an org-owned dependency/service and product/privacy contract.

**Recommendation:** Decide product adoption and default explicitly; do not merge personal-fork dependencies, and align the implementation after the crates move under `storytold/` and after the egui 0.37 line.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2163#issuecomment-6271272390). Confidence: high; current-head re-review: True.

### D019 — [PR #2152](https://github.com/storytold/filmcraft/pull/2152): G1: fidelity harness against Premiere Pro

Should the Premiere fidelity harness persistently enable unsigned CEP extensions on maintainers’ machines, and what isolation/cleanup/security constraints are required?

**Options:** Allow optional installer to enable CEP and bridge; reject persistent install; require scoped installation and project isolation before merge.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2152). Confidence: high; current-head re-review: None.

### D020 — [PR #2147](https://github.com/storytold/filmcraft/pull/2147): fix(import): import files a macOS drag only promised (Photos.app videos arrived as JPEGs)

Accept runtime method replacement against winit private delegate internals, and define upgrade validation.

**Options:** Accept the private-structure hook with explicit winit upgrade checks; or use another supported API / defer.

**Recommendation:** Require a documented integration check against the pinned winit version and a real macOS promised-file drag before accepting this dependency.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2147#pullrequestreview-5566178058). Confidence: high; current-head re-review: None.

### D021 — [PR #2122](https://github.com/storytold/filmcraft/pull/2122): Add experimental native AI rotoscoping with EdgeTAM and ViTMatte

Decide whether raster alpha uses schema 16 and establish migration/rollback guarantees.

**Options:** Accept schema 16 with documented migration and rollback behavior; or choose a backward-compatible format design / defer.

**Recommendation:** Require an explicit migration and downgrade policy before format change; confirm latest description’s rollback claims against the project compatibility contract.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2122#pullrequestreview-5566224514). Confidence: high; current-head re-review: None.

### D022 — [PR #2122](https://github.com/storytold/filmcraft/pull/2122): Add experimental native AI rotoscoping with EdgeTAM and ViTMatte

Decide whether to ship the selected optional native inference runtime/model stack.

**Options:** Accept current EdgeTAM/ViTMatte runtime; or require alternative runtime/model comparison before adoption.

**Recommendation:** Approve only after platform portability, packaging, and licensing review establish the runtime is maintainable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2122#pullrequestreview-5566224514). Confidence: high; current-head re-review: None.

### D023 — [PR #2044](https://github.com/storytold/filmcraft/pull/2044): Search font pickers and fix ripple, snapping and window preferences

Decide whether enabling eframe persistence should store egui memory as well as window geometry.

**Options:** Persist egui memory; or persist only selected window state and keep egui storage disabled/unwritten.

**Recommendation:** Persist only the explicitly intended window state to avoid an undocumented broader persistence change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2044#pullrequestreview-5566317389). Confidence: high; current-head re-review: None.

### D024 — [PR #2031](https://github.com/storytold/filmcraft/pull/2031): fix(mcp): retain request ownership after stdio failures

Approve the MCP stdio failure architecture and output-file cleanup behavior when cancelled/failed exports leave partial output.

**Options:** Accept bounded stdin bridge, drain semantics, and retain partial files with explicit unconfirmed-cleanup status; keep existing cleanup semantics; split the independent changes.

**Recommendation:** Resolve explicitly with the linked PR/issue owners; preserve documented contract unless the maintainer approves the behavior change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2031). Confidence: high; current-head re-review: True.

### D025 — [PR #1892](https://github.com/storytold/filmcraft/pull/1892): cleanup: remove unused dependencies, dead code and committed target/ fixtures

Decide whether to remove the listed public functions from the library API.

**Options:** Remove the public items; or preserve them as supported/possibly planned API.

**Recommendation:** Approve removals only after checking downstream/API policy; split reversible fixture/dependency cleanup from public API removals.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1892#pullrequestreview-5566523198). Confidence: high; current-head re-review: None.

### D026 — [PR #1857](https://github.com/storytold/filmcraft/pull/1857): ui: refresh completion status for repeated interchange exports

Choose whether status_seen should be cleared in the shared set_status helper or only for interchange export completion.

**Options:** Keep the narrow local fix; or generalize status replacement behavior across all status writes.

**Recommendation:** Prefer a small shared set_status fix only if all call sites should permit identical status messages to refresh; otherwise keep this local behavior.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1857#pullrequestreview-5566567685). Confidence: medium; current-head re-review: None.

### D027 — [PR #1847](https://github.com/storytold/filmcraft/pull/1847): fix(render): apply nested audio channel picks before downmix

Approve the new source-channel validation invariant in AGENTS.md and the new dialog limits (64 channels/4096 cells).

**Options:** Accept the invariant and limits; or keep the behavior change but move the engineering rule to a separate policy change and revise limits.

**Recommendation:** Approve limits only with product rationale and add the general invariant in a dedicated engineering-policy change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1847#pullrequestreview-5566591224). Confidence: high; current-head re-review: None.

### D028 — [PR #1836](https://github.com/storytold/filmcraft/pull/1836): wasm: SIMD YUV->RGBA conversion, optional threaded web build, benchmarks

Should FilmCraft ship a threaded WebAssembly build, with a second artifact, nightly toolchain, atomics/shared memory and a distinct repaint path?

**Options:** Keep threaded build as opt-in/separate artifact; or decline the added browser/toolchain requirements and ship only single-threaded wasm.

**Recommendation:** Decide the web architecture and nightly toolchain requirement separately from the SIMD optimization; retain per-feature measurement and scope.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1836#pullrequestreview-5566606465). Confidence: high; current-head re-review: None.

### D029 — [PR #1768](https://github.com/storytold/filmcraft/pull/1768): Search: add optional local visual retrieval

Decide whether to adopt Candle dependency stack, local model download, and model licensing/installation flow for visual search.

**Options:** Accept optional local inference and downloadable model; or defer/decompose until dependency and model distribution policy is set.

**Recommendation:** Keep disabled by default and split model installation/runtime from UI; approve only after license, size, provenance and offline failure behavior are reviewed.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1768#pullrequestreview-5567162740). Confidence: high; current-head re-review: None.

### D030 — [PR #1764](https://github.com/storytold/filmcraft/pull/1764): codecs: detect explicit Apple Log metadata in MOV

Should Apple Log metadata add project schema v16, making projects saved by the new build unreadable by the previous release?

**Options:** Accept the global schema bump; or serialize unknown transfer values compatibly so older builds can read projects without Apple Log.

**Recommendation:** Choose a backward-compatibility policy before merge and coordinate with any other v15→v16 change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1764#pullrequestreview-5567173932). Confidence: high; current-head re-review: None.

### D031 — [PR #1755](https://github.com/storytold/filmcraft/pull/1755): Add effect reordering, copy/paste, duplication, and keyboard deletion

Choose effect automation command naming and split/sequence work across overlapping effect copy/delete/reorder PRs.

**Options:** Select canonical command names and implementation; decide which PR carries render changes and how to sequence #1319/#1400/#1738/#1731.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1755). Confidence: high; current-head re-review: None.

### D032 — [PR #1745](https://github.com/storytold/filmcraft/pull/1745): platform: VA-API H.264 stays in hardware after an open-GOP seek on Mesa radeonsi

Should VA-API open-GOP recovery reject stand-in frames on all drivers or restrict the new error to drivers that require it?

**Options:** Apply conservative behavior globally; scope to affected driver(s) after Intel evidence; retain old behavior.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1745). Confidence: high; current-head re-review: None.

### D033 — [PR #1648](https://github.com/storytold/filmcraft/pull/1648): NVENC H.265 export on Linux, and NVENC export at all (#1624)

Choose Linux NVENC HEVC probe/session lifecycle among competing PRs #1648 and #1332.

**Options:** Take #1648 lazy probe/startup path; take #1332 warmup/join path; reconcile lifecycle in one implementation and retain proper author credit.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1648). Confidence: high; current-head re-review: None.

### D034 — [PR #1646](https://github.com/storytold/filmcraft/pull/1646): feat(updater): crash-resilient self-updating for desktop app

Decide whether the new TLS/updater dependency stack belongs in every desktop build unconditionally.

**Options:** Keep unconditional updater and TLS dependencies; or gate the updater/transport behind an optional feature or platform-specific setup.

**Recommendation:** Avoid adding an alpha TLS stack to every desktop build until dependency maturity and licensing are reviewed.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1646#pullrequestreview-5567483323). Confidence: medium; current-head re-review: None.

### D035 — [PR #1646](https://github.com/storytold/filmcraft/pull/1646): feat(updater): crash-resilient self-updating for desktop app

Decide whether to ship self-updating before signed/verified release artifacts and trust-chain support exist.

**Options:** Ship with the current unsigned-release trust model; or defer until release-signing/provenance verification exists.

**Recommendation:** Defer automatic replacement until updates can be authenticated independently; require an explicit trust and rollback design.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1646#issuecomment-6311772189). Confidence: high; current-head re-review: None.

### D036 — [PR #1618](https://github.com/storytold/filmcraft/pull/1618): fix(control): process remote commands when Wayland window is hidden

Choose the approach for processing control commands while a Wayland window is hidden, and whether to add a Linux-only winit wrapper/dependency.

**Options:** Use synthetic redraw wrapper; use competing #1584 route; retain existing behavior. Keep ticket lifecycle fix independently.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1618). Confidence: high; current-head re-review: None.

### D037 — [PR #1572](https://github.com/storytold/filmcraft/pull/1572): feat(plugin): sandboxed WebAssembly (WASM) plugin system with SDK, engine and UI integration (G11)

Approve the WASM plugin ABI/runtime and dependency architecture?

**Options:** Approve proposed sandboxed WASM architecture/ABI and runtime; choose a different runtime/ABI; defer plugin support.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1572). Confidence: high; current-head re-review: None.

### D038 — [PR #1549](https://github.com/storytold/filmcraft/pull/1549): Embed craft-fonts by default via .cargo/config.toml [env]

Should CRAFT_FONTS_DIR be configured by default in .cargo/config.toml, embedding craft-fonts in every clone/build?

**Options:** Keep optional build input; make default and require local checkout; use another opt-in mechanism.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1549). Confidence: high; current-head re-review: None.

### D039 — [PR #1413](https://github.com/storytold/filmcraft/pull/1413): #65: CommandSpec carries which target keys a command takes by name

Should CommandSpec store named target keys explicitly on every command literal, or derive them via constructor/side table?

**Options:** Per-command explicit field as proposed; or derive/centralize the field to avoid editing every literal.

**Recommendation:** Choose the representation based on maintainability; author says either path is easy to change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1413#issuecomment-6279083104). Confidence: medium; current-head re-review: None.

### D040 — [PR #1363](https://github.com/storytold/filmcraft/pull/1363): perf(av1): optimize decoder allocations, zero-fill elimination and DC transforms

Decide whether AV1 decoders with the same thread count share one Rayon pool.

**Options:** Share pool across decoders; or isolate concurrency per decoder.

**Recommendation:** Keep concurrency isolated unless shared scheduling has benchmarked benefits and bounded interference.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1363#issuecomment-6139860664). Confidence: medium; current-head re-review: None.

### D041 — [PR #1179](https://github.com/storytold/filmcraft/pull/1179): MCP/Automation: Agent control server, UI map, and automation IDs

Choose whether generated docs/ui-map.md (~4k lines) should be checked in or generated at build time.

**Options:** Check generated UI map into repository; or generate when needed/build time.

**Recommendation:** Generate it through a documented command and keep source of truth in code unless committed output is required for offline consumers.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1179#pullrequestreview-5569645095). Confidence: medium; current-head re-review: None.

### D042 — [PR #1172](https://github.com/storytold/filmcraft/pull/1172): Gate the control server behind a per-run token (#163)

Choose the per-run control-server authentication wire format and whether to align with shared craft-control auth frames.

**Options:** Use the proposed token JSON first-line scheme; wait for/use the sibling-app auth frame; choose another compatible protocol.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1172). Confidence: high; current-head re-review: None.

### D043 — [PR #1115](https://github.com/storytold/filmcraft/pull/1115): Wayland: native file drag and drop through a patched winit (#1022)

Should FilmCraft vendor and patch a 62,000-line winit tree for native Wayland DnD, given overlap with #634 and ongoing maintenance/security updates?

**Options:** Choose patched native Wayland support; prefer #634 XWayland route; or defer until GNOME/mutter evidence and a sustainable patch strategy exist.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1115). Confidence: high; current-head re-review: None.

### D044 — [PR #1079](https://github.com/storytold/filmcraft/pull/1079): #1065: JPEG XL DNG behind an off-by-default feature (needs maintainer decision: new dependency)

Should JPEG XL DNG decoding remain behind an off-by-default feature, and does the new JPEG XL dependency receive maintainer approval?

**Options:** Approve dependency/feature gate; reject new dependency; choose an alternative implementation or defer 16-bit float support.

**Recommendation:** Record the choice on the PR or associated issue; keep the change scoped to the chosen route.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1079). Confidence: high; current-head re-review: True.

### D045 — [PR #1077](https://github.com/storytold/filmcraft/pull/1077): #1065: DNG raw settings, faster decode, opcode lists, lossy JPEG DNG

Decide whether to add JPEG XL DNG decoding dependency, currently isolated in a separate off-by-default draft.

**Options:** Accept a new optional dependency; or defer JPEG XL support pending dependency/security/packaging review.

**Recommendation:** Keep it isolated and off by default until the dependency and distribution costs are reviewed.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1077). Confidence: medium; current-head re-review: True.

### D046 — [PR #883](https://github.com/storytold/filmcraft/pull/883): h264: safe-SIMD 6-tap luma spike (decoder throughput half of #848)

Choose whether to land the 6-tap SIMD kernels before or after the u16 pipeline, coordinating with #848 so kernels are ported once.

**Options:** Land current spike first; wait and rebase both onto u16 pipeline; select one implementation path.

**Recommendation:** Resolve explicitly with the linked PR/issue owners; preserve documented contract unless the maintainer approves the behavior change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/883). Confidence: medium; current-head re-review: True.

### D047 — [PR #408](https://github.com/storytold/filmcraft/pull/408): Linux: VA-API hardware decoding (H.264, HEVC, VP9, AV1) and H.264 / H.265 export encoding

Approve the new `libloading` dependency and choose whether the author should rebuild this broad VA-API change on the main architecture as separate decode/encode PRs.

**Options:** Approve dependency and accept the requested split/rebuild; reject dependency or defer; select only specific new VP9/AV1/encode pieces.

**Recommendation:** Record the choice on the PR or associated issue; keep the change scoped to the chosen route.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/408). Confidence: high; current-head re-review: None.

## Competing implementations and landing order

### D048 — [PR #2518](https://github.com/storytold/filmcraft/pull/2518): Speed up settled stereo reverb and mixer automation tails

How should this reverb-tail optimization be coordinated with overlapping broader PR #1978?

**Options:** Merge this narrower change first; merge #1978 first or only; or combine/rebase the work.

**Recommendation:** Compare overlap and choose a single merge order to avoid duplicate/conflicting edits.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2518). Confidence: medium; current-head re-review: None.

### D049 — [PR #2496](https://github.com/storytold/filmcraft/pull/2496): Tools: a widened panel fills its width with columns (builds on #2168)

Select one implementation for issue #1983; three open approaches conflict in panels/tools.rs.

**Options:** #2168 scroll-only; #2496 scroll plus columns when widened; #2495 columns with dock widening based on #2178.

**Recommendation:** Choose #2496 if the goal is a small standalone improvement after #2168; otherwise select #2495 if automatic strip widening is required to meet the report. Decide on the issue thread before merging conflicting branches.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2496#pullrequestreview-5573839561). Confidence: high; current-head re-review: None.

### D050 — [PR #2495](https://github.com/storytold/filmcraft/pull/2495): Tools: reflow into columns, widening the strip to fit (builds on #2178)

Choose one of the competing Tools panel approaches (#2168 scrolling, this PR columns/widening, or #2496 hybrid), and approve widening space allocation.

**Options:** Pick one layout approach and sequencing; if widening is chosen, decide on scrolling fallback when columns do not fit.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2495). Confidence: high; current-head re-review: None.

### D051 — [PR #2431](https://github.com/storytold/filmcraft/pull/2431): platform: NVDEC on Windows for 4:2:2 and 10-bit H.264 camera footage (draft; builds on the h264 PR)

Determine landing order for software decoder #2301 and Windows NVDEC follow-up.

**Options:** Land software decoder first and rebase NVDEC; or review/merge hardware path on top of the shared branch.

**Recommendation:** Keep the requested split: land software decode first, then rebase/review NVDEC as a distinct platform change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2431). Confidence: medium; current-head re-review: None.

### D052 — [PR #2312](https://github.com/storytold/filmcraft/pull/2312): Premiere: import .prproj projects with source provenance

Which of the competing Premiere `.prproj` importer PRs should land?

**Options:** Choose this provenance importer or one of #1694, #1652, #2296, #495; consolidate selected work and close/supersede the others.

**Recommendation:** Compare approaches against Premiere Pro parity and choose one importer implementation; author offers to close this PR if another is selected.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2312#pullrequestreview-5571817760). Confidence: high; current-head re-review: True.

### D053 — [PR #2308](https://github.com/storytold/filmcraft/pull/2308): ui: replace hand-drawn UI icons with Tabler Icons (MIT)

Which icon source/direction should FilmCraft standardize on, given overlap between this all-app Tabler replacement and #816’s craft-icons/Lucide-compatible set?

**Options:** Adopt Tabler for all variants; adopt #816 craft-icons for overlapping toolbar/bin icons and extend coverage; or create a shared icon package and unify.

**Recommendation:** Choose a single app-wide icon direction; do not merge overlapping replacements independently.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2308#pullrequestreview-5571561541). Confidence: high; current-head re-review: None.

### D054 — [PR #2296](https://github.com/storytold/filmcraft/pull/2296): G2: Import Premiere projects and restore time remapping keyframes

Which Premiere importer should land, and what is the supported time-remapping compatibility contract?

**Options:** Choose among #2296, #2312, #1694, #1652 and #495; define whether Premiere speed curves map to FilmCraft retiming or remain unsupported.

**Recommendation:** Select one importer only after comparing representative Premiere Pro exports and specify the retiming mapping in the chosen PR.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2296#pullrequestreview-5566028678). Confidence: medium; current-head re-review: None.

### D055 — [PR #2288](https://github.com/storytold/filmcraft/pull/2288): i18n: Russian covers the whole interface

Which full Russian localization PR should land, and should a shared `Edit` translation be changed from «Правка» to «Изменить»?

**Options:** Choose #2288 or #1066 for `ru.tsv`; retain shared `Edit` label «Правка» or introduce a distinct key if only the mode tab should say «Изменить».

**Recommendation:** Choose one catalog and preserve the conventional menu term unless the UI needs a separate tab-specific key; author explicitly defers to maintainers.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2288#issuecomment-6273883160). Confidence: high; current-head re-review: None.

### D056 — [PR #2217](https://github.com/storytold/filmcraft/pull/2217): add Nix flake for building and installing on NixOS

Which duplicate Nix flake implementation should be retained, this PR or #1374?

**Options:** Choose this flake or #1374, then close the superseded duplicate; decide who maintains output hashes and docs.

**Recommendation:** Compare maintenance burden and supported systems, select one, and name an owner for ongoing Nix upkeep.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2217). Confidence: medium; current-head re-review: None.

### D057 — [PR #2189](https://github.com/storytold/filmcraft/pull/2189): fix(export): unpreset H.264 exports start from the default preset

Sequence this change with #2133, which also changes unpreset export behavior.

**Options:** Review both side by side and land compatible defaults; or defer/split until one policy is selected.

**Recommendation:** Make a single combined maintainer decision on the desired unpreset export defaults before merging either.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2189#pullrequestreview-5566112730). Confidence: high; current-head re-review: None.

### D058 — [PR #2168](https://github.com/storytold/filmcraft/pull/2168): Fix #1983: keep toolbar tools reachable at large UI scale

Choose between scroll-only #2168 and overlapping #2178/#2495 implementations for issue #1983.

**Options:** #2168 scroll-only; #2178 overlapping/dock-based approach; #2495 columns with dock widening; or #2496 scroll plus columns after widening.

**Recommendation:** Make one issue-level choice and rebase/remove overlapping tools.rs portions; #2496 review adds a third viable scope.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2168#issuecomment-6264525025). Confidence: high; current-head re-review: None.

### D059 — [PR #2156](https://github.com/storytold/filmcraft/pull/2156): #1994: the main window remembers its position, size and maximized state between launches

Choose between this persistence implementation and overlapping #2169.

**Options:** Land this PR; or land #2169, provided egui memory remains unpersisted.

**Recommendation:** Select one implementation and close/supersede the other before merge to avoid competing persistence paths.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2156#issuecomment-6258025199). Confidence: high; current-head re-review: None.

### D060 — [PR #2153](https://github.com/storytold/filmcraft/pull/2153): #2094: the Lumetri shadow lift no longer amplifies near-black chroma into blue specks

Choose shadow-lift cap and whether pure black should be lifted, coordinating with competing implementation.

**Options:** Choose the cap and pure-black behavior proposed here or the alternative in #2094-related PR; only one implementation should land.

**Recommendation:** Compare against Premiere Pro behavior and representative near-black footage, then choose one shared rule and close the competing PR.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2153#issuecomment-6257950317). Confidence: medium; current-head re-review: None.

### D061 — [PR #2111](https://github.com/storytold/filmcraft/pull/2111): refactor(export): improve GPU export robustness and render planning

How should #2111’s prepared-render robustness change integrate and land alongside fallible renderer API work in #1019/#1021?

**Options:** Coordinate API changes and landing order so fallible planning/materialization errors remain distinct from unsupported-work CPU fallback; or defer one change.

**Recommendation:** Set a merge order and retain a prepared-path export test; after the author declined more work, a maintainer accepted ownership of follow-up, but final acceptance/landing order remains open.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2111#issuecomment-6318996146). Confidence: high; current-head re-review: None.

### D062 — [PR #1969](https://github.com/storytold/filmcraft/pull/1969): apps/filmcraft: add --x11 option for file drag-and-drop on Linux (Wayland)

Choose one Wayland file-drop workaround for issue #1865.

**Options:** #1969 adds opt-in --x11/FILMCRAFT_X11; #2012 prefers working X11/XWayland by default, falls back to default backend and exposes FILMCRAFT_WINDOW_BACKEND.

**Recommendation:** Choose a single backend policy on the issue; #2012 appears to handle the regression automatically while retaining override control, so compare behavior and close the other route.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1969#pullrequestreview-5566405810). Confidence: high; current-head re-review: None.

### D063 — [PR #1960](https://github.com/storytold/filmcraft/pull/1960): #1910: Interpret Footage from the item menu takes the whole selection

Choose between this implementation and #1966 for the same selection behavior.

**Options:** Keep #1960; keep #1966; or combine their distinct coverage.

**Recommendation:** The recorded comparison supports #1960, including right-click outside-selection coverage, but choose explicitly and close the other PR.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1960#issuecomment-6279093276). Confidence: medium; current-head re-review: True.

### D064 — [PR #1919](https://github.com/storytold/filmcraft/pull/1919): #1216: Export Settings CPU thread cap, preset round trip and CLI --threads

Which thread-cap implementation should land, and should Export CPU Threads be a new app-wide default in Settings ▸ Playback?

**Options:** Select this PR or overlapping #1928 for decoder/encoder caps; then decide whether to add a persistent app-wide thread cap and where the setting belongs.

**Recommendation:** Pick one implementation for overlapping internals, then handle the dialog/CLI/default setting as a separate change; location/default are explicitly maintainer decisions.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1919#pullrequestreview-5566488621). Confidence: high; current-head re-review: None.

### D065 — [PR #1911](https://github.com/storytold/filmcraft/pull/1911): export: omit sidecar cues rounded to zero duration

Should this PR supersede #1749 via a duplicate branch, or be integrated into that existing PR?

**Options:** Continue this PR and close #1749; contribute the change to #1749; preserve the existing PR as canonical.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1911). Confidence: medium; current-head re-review: None.

### D066 — [PR #1906](https://github.com/storytold/filmcraft/pull/1906): edit: preserve identities in repeated move and copy plans

Should move identity uniqueness become a repository-wide engineering invariant, and which duplicate PR should land?

**Options:** Accept the binding engineering-rule change and choose #1906 or identical #1915; or keep the rule local to this implementation.

**Recommendation:** Decide the invariant with maintainers; land only one PR and rebase it onto main.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1906#pullrequestreview-5566498667). Confidence: high; current-head re-review: None.

### D067 — [PR #1886](https://github.com/storytold/filmcraft/pull/1886): i18n: Arabic menus in display order

Choose one of three competing Arabic catalog PRs and the supported translation scope.

**Options:** Select #1886 menu-only, #1469 or #1896 broader catalog; also decide menus-only versus full UI coverage.

**Recommendation:** Pick a single catalog source and scope before translation review; current review confirms overlapping files cannot all land.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1886#pullrequestreview-5566537419). Confidence: high; current-head re-review: None.

### D068 — [PR #1859](https://github.com/storytold/filmcraft/pull/1859): fix(codecs): remove dead mpegts PES guard and duplicate AVI scan (#1706)

Choose which duplicate PR (#1859 or #1707) carries the MPEG-TS/AVI fix and preserve contributor credit.

**Options:** Merge this PR; merge #1707 and transfer tests; co-author the original contributor on chosen commit.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1859). Confidence: high; current-head re-review: None.

### D069 — [PR #1855](https://github.com/storytold/filmcraft/pull/1855): Fix ten export, import, caption, and editing reliability bugs

Should the broad reliability PR close nine issues and supersede several open source PRs, or split/retain issue references for incomplete work?

**Options:** Accept broad closure/supersession; split changes by issue; use Refs for partial fixes and leave source PRs/issues open.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1855). Confidence: high; current-head re-review: None.

### D070 — [PR #1848](https://github.com/storytold/filmcraft/pull/1848): #1687: FCP7 XML clip edges under a transition come from the neighbouring transitionitem

Which FCP7 XML transition-edge fix should land, this PR or #1754?

**Options:** Choose the compact fix here; or choose #1754 and port the useful malformed-edge warnings.

**Recommendation:** Compare both on a Premiere export containing `pproTicksIn` and a `-1` start; choose one and carry the needed warnings.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1848#pullrequestreview-5566586722). Confidence: high; current-head re-review: None.

### D071 — [PR #1835](https://github.com/storytold/filmcraft/pull/1835): fix(codecs): read AC-3/E-AC-3 MP4 layout from dac3/dec3 (#1701)

Choose which of #1835 and #1859 lands for the duplicate MP4 layout fix.

**Options:** Land #1835; land #1859; or consolidate their equivalent implementations/tests.

**Recommendation:** Compare current heads, merge one implementation only and close the duplicate.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1835#pullrequestreview-5567044271). Confidence: high; current-head re-review: None.

### D072 — [PR #1829](https://github.com/storytold/filmcraft/pull/1829): fix(codecs): read the frame rate phone clips average to (#1700)

Which of the overlapping phone-frame-rate recovery implementations should land for #1700?

**Options:** Choose #1829’s 0.5% snap tolerance, #1739’s grid recovery, or #2239’s approach.

**Recommendation:** Compare all implementations on shared 90-kHz and 1-kHz fixtures; select one rule and close/supersede the others.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1829#pullrequestreview-5567054191). Confidence: high; current-head re-review: None.

### D073 — [PR #1828](https://github.com/storytold/filmcraft/pull/1828): Project panel: Sequence Settings… in a sequence's right-click menu

Choose between this sequence context-menu implementation and #2245, which edits the same item_menu path.

**Options:** Land #1828; land #2245; or consolidate.

**Recommendation:** Select one implementation and rebase the selected PR on current main; review notes another overlap with #1555 for dialog semantics.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1828#pullrequestreview-5567057341). Confidence: high; current-head re-review: None.

### D074 — [PR #1811](https://github.com/storytold/filmcraft/pull/1811): Windows: guard initial AV1 color metadata for hardware decoding

Choose AV1 hardware-to-software fallback policy shared with #2221.

**Options:** Allow the sequence-header differences in this PR; or use the stricter/different threshold in #2221.

**Recommendation:** Set the trust boundary once across both PRs and merge neither until it is explicit; preserve fallback for untrusted metadata.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1811#issuecomment-6268467722). Confidence: high; current-head re-review: None.

### D075 — [PR #1757](https://github.com/storytold/filmcraft/pull/1757): effects: drag to reorder effect controls

Choose the public command name for effect reordering because automation/scripts will bind to it.

**Options:** Use `effects.reorder {clip, from, to}` from #1757; or `effects.move {clip?, index?, to}` from #1755.

**Recommendation:** Choose `effects.reorder` as the smaller, cleaner option if compatible with command naming conventions, then close or rework #1755.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1757#pullrequestreview-5567197368). Confidence: high; current-head re-review: None.

### D076 — [PR #1739](https://github.com/storytold/filmcraft/pull/1739): fix(codecs): recover fps/frame-grid of ms-rounded phone clips

Which rule should recover nominal frame rate from millisecond-rounded phone durations across differing track timescales?

**Options:** Choose this approach, #1829’s 0.5% tolerance, or #1729’s average rounded-duration rule.

**Recommendation:** Compare all approaches against a shared real/synthetic fixture including the 2970/3060-at-90-kHz pattern before selecting the rule.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1739#pullrequestreview-5567243868). Confidence: high; current-head re-review: None.

### D077 — [PR #1717](https://github.com/storytold/filmcraft/pull/1717): Add Taiwan Traditional Chinese (zh-TW) interface

Choose the canonical PR for Traditional Chinese (zh-TW) support between this PR and #1668.

**Options:** Take #1717 or #1668; consolidate work and preserve correct author credit.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1717). Confidence: high; current-head re-review: None.

### D078 — [PR #1707](https://github.com/storytold/filmcraft/pull/1707): mpegts/avi: drop redundant PES header check and doubled frame_info call

Choose which duplicate demux refactor lands, #1707 or #1859.

**Options:** Keep #1707 (older, author comparison supports it); keep #1859; or consolidate.

**Recommendation:** Select one and close the duplicate; do not merge both conflicting changes.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1707#issuecomment-6279093579). Confidence: medium; current-head re-review: None.

### D079 — [PR #1698](https://github.com/storytold/filmcraft/pull/1698): #978: opening or importing a Premiere project says how to bring it over

When one of the competing Premiere project readers lands, decide merge order and removal of the temporary foreign_project guard.

**Options:** Choose canonical .prproj reader among #1694/#1652/#2312/#2296, then remove guard; keep guard until importer availability.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1698). Confidence: high; current-head re-review: None.

### D080 — [PR #1694](https://github.com/storytold/filmcraft/pull/1694): prproj import

Choose merge order and shared schema for competing Premiere project readers #1694, #1652, #2312 and #2296.

**Options:** Choose one reader to land first; consolidate format/imported field changes; or reject/defer overlapping approaches.

**Recommendation:** Select a single base architecture and rebase later readers; do not merge multiple incompatible readers independently.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1694#pullrequestreview-5567309840). Confidence: high; current-head re-review: None.

### D081 — [PR #1684](https://github.com/storytold/filmcraft/pull/1684): Speed/Duration ▸ Maintain Audio Pitch time-stretches a clip's audio instead of resampling it

Choose which of #1684 and #1208 implements Maintain Audio Pitch for clips.

**Options:** Land #1684; land #1208; or combine after selecting one algorithm.

**Recommendation:** Select one implementation, then evaluate documented algorithm tradeoffs; authors confirm choice belongs to maintainers.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1684#issuecomment-6287227085). Confidence: high; current-head re-review: None.

### D082 — [PR #1671](https://github.com/storytold/filmcraft/pull/1671): codecs: start an edited MP4 at its first presented picture

Which duplicate edited-MP4 start-frame fix should land, and how should the original author be credited?

**Options:** Take #1289 after rebase; or take this newer PR with `Co-authored-by: bfenty`.

**Recommendation:** Select one PR and preserve the original author credit on the merged contribution.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1671#pullrequestreview-5567352878). Confidence: high; current-head re-review: None.

### D083 — [PR #1668](https://github.com/storytold/filmcraft/pull/1668): Add Traditional Chinese (Taiwan) interface support

Choose between competing Traditional Chinese catalogs and whether Taiwan Chinese becomes a ninth required catalog.

**Options:** Land #1668 or #1717; require complete zh-tw coverage for all future strings or allow English fallback until translated.

**Recommendation:** Select one catalog and require ninth-locale completeness only if maintainers are ready to sustain it; otherwise keep fallback policy consistent.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1668#pullrequestreview-5567361643). Confidence: high; current-head re-review: None.

### D084 — [PR #1625](https://github.com/storytold/filmcraft/pull/1625): Drag-and-drop media files from the OS file manager

Choose Wayland file-drop strategy between manual XWayland notice and automatic #634 routing, and whether the Timeline notice should reflect the preference state.

**Options:** Use XWayland startup fallback; keep native unsupported notice; select other native DnD implementation; fix notice copy/visibility.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1625). Confidence: high; current-head re-review: None.

### D085 — [PR #1617](https://github.com/storytold/filmcraft/pull/1617): #1583: still images honour EXIF Orientation

How should existing projects with stills saved before EXIF auto-orientation be handled?

**Options:** Keep stored dimensions and document that existing clips change orientation on re-probe; or migrate existing project dimensions. Also choose between this PR and overlapping #1607.

**Recommendation:** Decide whether to preserve historic project appearance or migrate; select one implementation for the overlap and retain the compatible parts of the other.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1617#pullrequestreview-5567611371). Confidence: medium; current-head re-review: None.

### D086 — [PR #1607](https://github.com/storytold/filmcraft/pull/1607): fix(media): respect EXIF orientation for imported still images

Choose between EXIF orientation approaches in #1607 and #1617, including changed behavior for existing importer callers.

**Options:** Use #1607; use #1617 header-only orientation handling; or combine with explicit compatibility tests.

**Recommendation:** Choose one approach after documenting the existing-output change and reuse #1617’s PNG/WebP coverage if this PR wins.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1607#pullrequestreview-5567627624). Confidence: high; current-head re-review: None.

### D087 — [PR #1601](https://github.com/storytold/filmcraft/pull/1601): dock: a divider changes only the two panels beside it

Choose dock divider resize semantics against #2178.

**Options:** Only adjacent panels change size (#1601); or shrink-then-push across the panel arrangement (#2178).

**Recommendation:** Select one consistent resize rule and close/split the other; #1601 matches issue #1595 according to both reviewer and author.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1601#issuecomment-6268039579). Confidence: high; current-head re-review: None.

### D088 — [PR #1600](https://github.com/storytold/filmcraft/pull/1600): tools: the Tools panel lays its buttons out along its longer side and wraps

Choose whether the Tools panel should wrap into columns in a wide strip or keep one row along the long side.

**Options:** Adopt wrapped/column layout; keep the wide strip row; choose a hybrid based on panel orientation.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1600). Confidence: high; current-head re-review: None.

### D089 — [PR #1555](https://github.com/storytold/filmcraft/pull/1555): fix(control): sequence.settings without params reports the settings instead of opening a dialog

Choose between narrow sequence.settings behavior in #1555 and global no-parameter dialog suppression in #1854.

**Options:** Suppress only this dialog; or make engine.execute skip all parameter dialogs on empty params.

**Recommendation:** Prefer consistent engine-wide behavior if empty parameter calls should never open dialogs; otherwise retain narrow command-specific semantics.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1555#pullrequestreview-5567735288). Confidence: high; current-head re-review: None.

### D090 — [PR #1509](https://github.com/storytold/filmcraft/pull/1509): #1458: --graphics gl, an opt-in OpenGL mode for Intel Gen 9 graphics

Coordinate whether the opt-in OpenGL mode remains necessary alongside #2138/#2139 GPU compositor changes.

**Options:** Land GL option as fallback; or defer/adjust after deciding whether DX12 GPU compositor support covers target hardware.

**Recommendation:** Review all three together and retain GL as an explicit fallback only if the hardware matrix still needs it.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1509#pullrequestreview-5567765793). Confidence: medium; current-head re-review: None.

### D091 — [PR #1470](https://github.com/storytold/filmcraft/pull/1470): i18n: add initial Turkish interface catalog

Should Turkish catalog PR #1470 proceed as a language-addition alternative to #2344, or should work be consolidated into #2344?

**Options:** Continue this PR and resolve overlapping catalog changes; close/contribute wording to #2344; choose one canonical language PR.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1470). Confidence: high; current-head re-review: None.

### D092 — [PR #1453](https://github.com/storytold/filmcraft/pull/1453): ui: Ctrl-click multi-select in Effect Controls feeds Save Preset (#1394)

Choose between overlapping Ctrl-click multi-select PRs #1453 and #1779 (which adds Shift-click).

**Options:** Land one implementation; or consolidate both interaction models in one patch.

**Recommendation:** Select and document the intended modifier semantics, then close or supersede the overlapping PR.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1453#pullrequestreview-5567815031). Confidence: medium; current-head re-review: None.

### D093 — [PR #1412](https://github.com/storytold/filmcraft/pull/1412): #1411: docs/gaps.md: remove entries for features that already shipped

Choose application order and issue-closing policy for overlapping docs PRs #1412, #1419 and #1423.

**Options:** Apply in a fixed sequence to keep parity counts consistent; or consolidate into one documentation update.

**Recommendation:** Consolidate or explicitly sequence all three, then close issue references only after checking each shipped item.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1412#pullrequestreview-5567895364). Confidence: medium; current-head re-review: None.

### D094 — [PR #1380](https://github.com/storytold/filmcraft/pull/1380): #1229: Finder opens the project it hands the app

Choose between competing Finder open-event implementations in #1380 and #1993.

**Options:** Use this in-tree handler; or use the shared crate / implementation in #1993, which also addresses cold-launch and media claims.

**Recommendation:** Compare cold-launch coverage and avoid double registration; choose one and have the losing PR close or split any unique work.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1380#pullrequestreview-5567953792). Confidence: high; current-head re-review: None.

### D095 — [PR #1374](https://github.com/storytold/filmcraft/pull/1374): Add a Nix flake (NixOS, GNOME and Hyprland)

Which Nix flake PR should be retained (#1374 or #2217), and who owns Nix upkeep?

**Options:** Choose either competing implementation and close the other; decide a maintainer for updates/output hashes.

**Recommendation:** Select one implementation after exact-system receipts and assign maintenance ownership.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1374#pullrequestreview-5567986512). Confidence: high; current-head re-review: None.

### D096 — [PR #1296](https://github.com/storytold/filmcraft/pull/1296): fix(timeline): make unlink visible and Clip ▸ Link say Unlink

Resolve conflicting behavior expectations in #1296 and #1297 for mixed selection and command side effects.

**Options:** Choose linked-item-only behavior; or allow unlinking all selected items and changing selection as part of command.

**Recommendation:** Apply one consistent rule across UI, CLI and control APIs; avoid UI-only behavior differences.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1296#pullrequestreview-5568109073). Confidence: medium; current-head re-review: None.

### D097 — [PR #1289](https://github.com/storytold/filmcraft/pull/1289): codecs: start an edited MP4 at its first presented picture (#1276)

Choose which duplicate PR carries the MP4 first-presented-picture fix (#1289 or #1671), and preserve the original author/test credit.

**Options:** Merge #1289; merge #1671; consolidate commit and retain author credit on chosen PR.

**Recommendation:** Record the choice on the PR or associated issue; keep the change scoped to the chosen route.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1289). Confidence: medium; current-head re-review: None.

### D098 — [PR #1208](https://github.com/storytold/filmcraft/pull/1208): clips: Maintain Audio Pitch for sped-up clips (follows #1146 / #959)

Which overlapping Maintain Audio Pitch implementation should land, #1208 or #1684?

**Options:** Keep this PR, keep #1684, or combine selected parts; only one can own the setting/TrackItem behavior.

**Recommendation:** Choose one implementation; #1684 is reported to apply cleanly while #1208 conflicts with main, but assess code/design before selection.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1208#issuecomment-6287227449). Confidence: high; current-head re-review: None.

### D099 — [PR #1087](https://github.com/storytold/filmcraft/pull/1087): i18n: Turkish (tr), Italian (it) and Vietnamese (vi) menus

Choose one Turkish catalog PR and define scope of #1087 relative to #1470 and #2344.

**Options:** Keep Turkish changes in #1087; or let #2344 replace Turkish and retain only Italian/Vietnamese here.

**Recommendation:** Use #2344 as the Turkish source (it already declares replacements) and remove/close overlapping Turkish portions of #1087.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1087#pullrequestreview-5569865612). Confidence: medium; current-head re-review: None.

### D100 — [PR #884](https://github.com/storytold/filmcraft/pull/884): h264: safe-SIMD weighted pred, chroma, and IDCT add (#848)

Coordinate #884 with #2301: rebase this u8 implementation after the u16 pipeline PR lands, or retain it independently if that dependency/order changed.

**Options:** Author proposed rebasing after #2301; alternatively maintainers can choose a different order if #2301 is not the landing base.

**Recommendation:** Verify #2301 state/current implementation and coordinate rebase before merge; do not carry forward the superseded assumption that both port kernels to u16.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/884#issuecomment-6195944701). Confidence: medium; current-head re-review: True.

## Behavior, defaults and compatibility

### D101 — [PR #2770](https://github.com/storytold/filmcraft/pull/2770): Playback: optional shuttle-style J/L that steps speed through zero (#2718)

Should optional signed-speed J/L shuttle be accepted with zero meaning pause/no commit in dynamic trim and off-ladder speeds rounding toward zero?

**Options:** Accept described behavior; choose different zero/trim semantics or no preference option.

**Recommendation:** Defer to maintainers; evidence does not establish a settled choice.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2770). Confidence: medium; current-head re-review: None.

### D102 — [PR #2769](https://github.com/storytold/filmcraft/pull/2769): Program and Source monitors: scrollbars and panning when zoomed past Fit (#2685)

Should plain wheel scroll a magnified monitor image, with Cmd/Ctrl+wheel or pinch reserved for zoom?

**Options:** Keep the new split gesture; or preserve wheel-to-zoom behavior from #807 and offer another scroll gesture.

**Recommendation:** Keep the new split gesture if scrolling is the desired behavior in #2685; document the tradeoff.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2769). Confidence: high; current-head re-review: None.

### D103 — [PR #2767](https://github.com/storytold/filmcraft/pull/2767): Audio: track effect tails ring out; edit any mixer insert's parameters (#2731)

Approve audio effect tail ring-out transport semantics and generic mixer insert editor behavior for keyframed parameters.

**Options:** Keep transport playing with playhead parked up to 10s and Space stops ring-out; static generic editor values defer to keyframes; or choose alternatives.

**Recommendation:** Defer to maintainers; evidence does not establish a settled choice.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2767). Confidence: medium; current-head re-review: None.

### D104 — [PR #2766](https://github.com/storytold/filmcraft/pull/2766): #2760: an export to a path naming another container says what the file is called

Should export accept a path extension that names a different container than the selected codec?

**Options:** Allow the path and report the actual container; or reject the mismatched extension.

**Recommendation:** Allow with a clear notice to preserve documented #341 behavior unless maintainers intend to reverse that contract.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2766). Confidence: medium; current-head re-review: True.

### D105 — [PR #2711](https://github.com/storytold/filmcraft/pull/2711): #2689: Reveal in Finder resolves project-relative paths; Reveal Proxy in Finder

Decide whether Russian and Ukrainian catalogs should expand beyond menu-bar strings to cover project context-menu actions.

**Options:** Keep ru/uk constrained to menu-bar catalog; or extend them to all visible interface strings.

**Recommendation:** Keep this PR aligned with current catalog policy and track expansion as a separate localization project.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2711#issuecomment-6326480847). Confidence: medium; current-head re-review: None.

### D106 — [PR #2660](https://github.com/storytold/filmcraft/pull/2660): #2419: Bokeh Blur runs on the GPU

Should GPU Bokeh Blur keep the implemented appearance, or use the cheaper early-halving behavior discussed in the issue?

**Options:** Preserve current look; or halve earlier for a cheaper but visually different result.

**Recommendation:** Preserve look absent a product decision to accept the visual change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2660). Confidence: medium; current-head re-review: True.

### D107 — [PR #2520](https://github.com/storytold/filmcraft/pull/2520): #2421: At Startup ▸ Show Home starts on Home with no project; the demo only on a first run or --demo

Should Show Home start on Home even when recent projects exist, or fall back to Open Most Recent?

**Options:** Keep Home as startup destination when selected; fall back to most recent project if history exists.

**Recommendation:** Defer to maintainers; evidence does not establish a settled choice.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2520). Confidence: medium; current-head re-review: None.

### D108 — [PR #2475](https://github.com/storytold/filmcraft/pull/2475): #2469: the media pool closes least recently used decoded stills past a budget

Approve the default decoded-still resident memory budget and eviction behavior, including repeated re-decode on misses?

**Options:** Accept current default budget/LRU close behavior; choose a different budget; expose a user setting or defer eviction.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2475). Confidence: high; current-head re-review: None.

### D109 — [PR #2466](https://github.com/storytold/filmcraft/pull/2466): Link stacked video clips that overlap the same audio together

Should multi-video linking require overlapping video time ranges, or allow consecutive cuts to share one audio link?

**Options:** Restrict to true temporal overlap as #2433 suggests; or accept the broader behavior and document/test consecutive cuts.

**Recommendation:** Choose one explicit rule because broader behavior reverses part of #507; update description and a consecutive-cut test to match.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2466#pullrequestreview-5570957122). Confidence: high; current-head re-review: None.

### D110 — [PR #2465](https://github.com/storytold/filmcraft/pull/2465): Preserve remix seams, waveform and manual Out trimming

Decide whether seam attenuation and correlated-crossfade correction should become the default for all new remix plans.

**Options:** Ship the new seam-level treatment by default; or keep it opt-in / defer until listening evidence supports the change.

**Recommendation:** Require representative listening and oracle comparison before changing the rendering default; if accepted, document expected audible level changes.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2465#pullrequestreview-5570974852). Confidence: high; current-head re-review: None.

### D111 — [PR #2439](https://github.com/storytold/filmcraft/pull/2439): Reject unsupported Time Remapping speed edits

Should unsupported Time Remapping speed edits be rejected at all API mutation points while the visible control remains present?

**Options:** Reject edits until rendering support exists; continue accepting/storing edits or hide/disable the control.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2439). Confidence: high; current-head re-review: None.

### D112 — [PR #2432](https://github.com/storytold/filmcraft/pull/2432): fix(export): a bitrateKbps in settings is the bitrate over an adaptive preset

Should unpreset H.264 exports continue using ExportSettings::default() or adopt the revised bitrate handling?

**Options:** Keep current default behavior; or revise the unpreset H.264 default behavior associated with #2189.

**Recommendation:** Keep this PR scoped to preset bitrate override; make the default behavior a separate explicit decision.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2432). Confidence: medium; current-head re-review: None.

### D113 — [PR #2420](https://github.com/storytold/filmcraft/pull/2420): Preserve original audio when proxies have fewer channels

Choose behavior when a proxy has fewer channels and its original is unavailable.

**Options:** Drop proxy and show offline slate; or continue showing proxy picture and proxy audio while unavailable original is unresolved.

**Recommendation:** Keep the proxy usable for picture when the original is offline, while making audio limitations explicit, unless preserving original channel layout is a hard correctness requirement.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2420#pullrequestreview-5571317375). Confidence: high; current-head re-review: None.

### D114 — [PR #2397](https://github.com/storytold/filmcraft/pull/2397): #2350: copy a selected transition and paste it at other edits

Should keyboard copy/paste of a transition place a textual label on the system clipboard so Ctrl/Cmd+V reaches FilmCraft on Windows/Linux?

**Options:** Add a label such as the effect name in this PR; or leave this as a follow-up and support Edit-menu paste only for now.

**Recommendation:** Decide whether keyboard parity is part of this PR’s scope; the reviewer calls the fix small and says it may be a follow-up if maintainers prefer.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2397#pullrequestreview-5571410812). Confidence: medium; current-head re-review: None.

### D115 — [PR #2378](https://github.com/storytold/filmcraft/pull/2378): Lumetri: Auto Color Correction for clips or the whole sequence (#1152)

Choose default Auto correction strength.

**Options:** Ship 50% strength; or use another default / expose a user-controlled strength.

**Recommendation:** Use 100% correction as the predictable meaning of Auto and make strength an explicit control if needed.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2378#pullrequestreview-5571457821). Confidence: medium; current-head re-review: None.

### D116 — [PR #2378](https://github.com/storytold/filmcraft/pull/2378): Lumetri: Auto Color Correction for clips or the whole sequence (#1152)

Decide whether Auto may silently clear existing Lumetri keyframes and re-enable a disabled effect.

**Options:** Treat Auto as a fresh-start action that clears affected animation and enables Lumetri; or preserve animation/effect enabled state and require an explicit reset.

**Recommendation:** Preserve user animation and enabled state, or present a clear confirmation before destructive replacement.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2378#pullrequestreview-5571457821). Confidence: high; current-head re-review: None.

### D117 — [PR #2362](https://github.com/storytold/filmcraft/pull/2362): #2213: Show Clip Keyframes picks the parameter a clip's timeline line…

Should clip_lanes and track_lanes UI lane selections persist in the project file?

**Options:** Persist both maps together with save/reopen support; keep them session-only.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2362). Confidence: high; current-head re-review: None.

### D118 — [PR #2240](https://github.com/storytold/filmcraft/pull/2240): Clarify the first-launch demo project name

Choose the exact demo project name given it becomes default filename and export title.

**Options:** Use “FilmCraft – Demo Project” with en dash; or plain “Demo Project” as the issue suggests.

**Recommendation:** Use plain “Demo Project” for a portable default filename; keep branding in application chrome.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2240#pullrequestreview-5566070541). Confidence: medium; current-head re-review: None.

### D119 — [PR #2224](https://github.com/storytold/filmcraft/pull/2224): Removed: full-screen button

Should FilmCraft remove the full-screen button from the header?

**Options:** Keep the button; or remove it for a cleaner header while retaining a menu path.

**Recommendation:** Treat removal as a design change that needs an issue/rationale and a retained menu path before accepting.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2224#pullrequestreview-5566088534). Confidence: medium; current-head re-review: None.

### D120 — [PR #2189](https://github.com/storytold/filmcraft/pull/2189): fix(export): unpreset H.264 exports start from the default preset

Decide whether unpreset scripted H.264 exports should change from fixed 20,000 kbps to Match Source adaptive bitrate.

**Options:** Change the default for all callers; or retain existing output and make adaptive bitrate opt-in through named preset/documentation.

**Recommendation:** Preserve compatibility for unpreset scripted exports and document explicit preset opt-in, unless maintainers deliberately approve the breaking default change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2189#pullrequestreview-5566112730). Confidence: high; current-head re-review: None.

### D121 — [PR #2156](https://github.com/storytold/filmcraft/pull/2156): #1994: the main window remembers its position, size and maximized state between launches

Decide whether full-screen state should be restored on relaunch.

**Options:** Restore full screen along with maximized/size/position; or restore only geometry and maximized state.

**Recommendation:** Restore full-screen state for consistency with common editor behavior, while retaining the stated constraint that egui memory itself is not persisted.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2156#issuecomment-6258025199). Confidence: medium; current-head re-review: None.

### D122 — [PR #2154](https://github.com/storytold/filmcraft/pull/2154): #2019: the Speed/Duration dialog reopens with the Ripple Edit choice last applied

Should Speed/Duration’s Ripple Edit choice persist across application restarts?

**Options:** Persist with UI state, as this PR does and Premiere preferences do; or remember only for the current session.

**Recommendation:** Keep persistence if matching Premiere Pro’s preference behavior is the goal; author explicitly leaves session-only as an alternative for maintainers.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2154#issuecomment-6259097192). Confidence: high; current-head re-review: None.

### D123 — [PR #2146](https://github.com/storytold/filmcraft/pull/2146): fix(macos): colour-match the window on wide-gamut displays (part of #2115)

Should wide-gamut Mac window color matching always be on, or controlled by existing preference; is partial matching acceptable?

**Options:** Honor existing toggle; remove toggle and make always-on; decide whether current SDR-only/partial wide-gamut match is acceptable.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2146). Confidence: high; current-head re-review: None.

### D124 — [PR #2139](https://github.com/storytold/filmcraft/pull/2139): fix(gpu): Intel Gen 9 to Gen 11 integrated graphics draw through DirectX 12 (#512, #1425)

Should DX12 substitution apply to every Intel Gen 9–11 family, including families without direct reports?

**Options:** Use all eleven families based on shared legacy-driver branch; or narrow to families with reported failures.

**Recommendation:** Decide the hardware-class support scope; author confirms reports exist for four families plus Comet Lake and offers to narrow the unreported groups.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2139#pullrequestreview-5566198154). Confidence: high; current-head re-review: None.

### D125 — [PR #2118](https://github.com/storytold/filmcraft/pull/2118): Allow floating panels to span multiple displays

Should every floating panel use an OS window by default on desktop?

**Options:** Enable native-window panels by default; or retain embedded panels unless the user opts in/platform behavior is validated.

**Recommendation:** Confirm default only after real-window behavior is checked across displays and minimize/restore; review explicitly calls the broad desktop default a maintainer decision.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2118#pullrequestreview-5566230172). Confidence: high; current-head re-review: None.

### D126 — [PR #2054](https://github.com/storytold/filmcraft/pull/2054): docs: add structured issue intake and security reporting guidance

Should SECURITY.md promise no response timeline for vulnerability reports?

**Options:** Keep no response commitment; or set an explicit response expectation for maintainers.

**Recommendation:** Confirm the wording before publishing policy on behalf of maintainers; reviewer labels this a reasonable item for maintainer confirmation.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2054#pullrequestreview-5566299530). Confidence: medium; current-head re-review: None.

### D127 — [PR #2044](https://github.com/storytold/filmcraft/pull/2044): Search font pickers and fix ripple, snapping and window preferences

Decide whether new installs should default snap_playhead to false, and how older settings without the key should behave.

**Options:** Adopt false for missing values; or preserve current true default and submit the behavior change separately.

**Recommendation:** Keep current default until the UX change is separately approved and clearly labelled; migration semantics must be explicit.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2044#pullrequestreview-5566317389). Confidence: high; current-head re-review: None.

### D128 — [PR #2029](https://github.com/storytold/filmcraft/pull/2029): Timeline Slider Overhaul

Choose whether the new slider settings are per-project or global user preferences.

**Options:** Keep them in project settings; or store as global preferences like mouse_scrolling.

**Recommendation:** Use global preferences for personal interaction choices unless the settings alter project interpretation or interchange.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2029#pullrequestreview-5566340178). Confidence: medium; current-head re-review: None.

### D129 — [PR #2029](https://github.com/storytold/filmcraft/pull/2029): Timeline Slider Overhaul

Decide whether session recovery should reopen sequences when restore_open_sequences is disabled.

**Options:** Restore project view unconditionally on recovery; or honor restore_open_sequences as open_project does.

**Recommendation:** Honor the preference consistently in recovery and normal project opening.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2029#pullrequestreview-5566340178). Confidence: high; current-head re-review: None.

### D130 — [PR #1995](https://github.com/storytold/filmcraft/pull/1995): Time Remapping: play static speeds and keyframed ramps, with linked sound (#510)

Approve Time Remapping’s migration effects on existing projects and the default-on Remap Audio behavior versus Premiere’s picture-only default.

**Options:** Accept timing changes for existing non-default speed values and audio remap on by default; keep audio picture-only/off by default; require migration prompt or compatibility behavior.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1995). Confidence: high; current-head re-review: None.

### D131 — [PR #1965](https://github.com/storytold/filmcraft/pull/1965): Add Reactor FastH3 Omni generation to FilmCraft desktop

Decide whether FilmCraft should ship a built-in integration with a single commercial, paid hosted generation service.

**Options:** Accept the vendor-specific integration that makes billable network requests on the user’s key; or decline/defer a built-in integration.

**Recommendation:** Require a product decision on service scope, key ownership, billing transparency and dependency licensing before full review.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1965#pullrequestreview-5566414691). Confidence: high; current-head re-review: None.

### D132 — [PR #1957](https://github.com/storytold/filmcraft/pull/1957): render: Transform with its own shutter angle stays on the CPU (the GPU path dropped its motion blur)

Should Transform with a non-default shutter angle remain on CPU, or should GPU output be made parity-correct before enabling?

**Options:** Keep CPU fallback; or repair GPU motion blur and enable GPU path.

**Recommendation:** Preserve CPU fallback until GPU matches Premiere Pro motion-blur output.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1957#pullrequestreview-5566432494). Confidence: medium; current-head re-review: None.

### D133 — [PR #1924](https://github.com/storytold/filmcraft/pull/1924): Effect Controls: the keyframe menu's interpolation applies to every selected keyframe

Should Clear interpolation affect only the clicked keyframe or all selected keyframes, matching the other interpolation action?

**Options:** Keep clicked-keyframe-only Clear; apply to selection; follow Premiere behavior if confirmed.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1924). Confidence: medium; current-head re-review: None.

### D134 — [PR #1899](https://github.com/storytold/filmcraft/pull/1899): #1877: graphics.set can put keyframes on the properties it sets

Decide whether graphics.set with a time parameter closes #1877, should return an error when no keyframe exists, or should imply keyframe creation.

**Options:** Close issue with current behavior/documentation; keep issue open for explicit error; make time imply keyframe.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1899). Confidence: high; current-head re-review: None.

### D135 — [PR #1886](https://github.com/storytold/filmcraft/pull/1886): i18n: Arabic menus in display order

Decide whether tr() should return visually reordered RTL strings for all consumers, including native menus, or reorder only in renderers.

**Options:** Keep display-order conversion inside catalog lookup; or apply it only to painted UI text and leave native menu logical strings to OS bidi handling.

**Recommendation:** Keep catalog strings in logical order and do direction-specific presentation at rendering boundaries; obtain native Arabic review.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1886#pullrequestreview-5566537419). Confidence: medium; current-head re-review: None.

### D136 — [PR #1883](https://github.com/storytold/filmcraft/pull/1883): effects.apply adds the effect at the end of the clip's stack, so existing effect indices stay valid

Should effects.apply place an effect at the end of the visible Effect Controls stack or preserve current render-order placement, and should that change close #1879?

**Options:** Keep render-order placement with stable returned index; change placement to end of visible list; decide issue closure after choosing.

**Recommendation:** Resolve explicitly with the linked PR/issue owners; preserve documented contract unless the maintainer approves the behavior change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1883). Confidence: high; current-head re-review: None.

### D137 — [PR #1864](https://github.com/storytold/filmcraft/pull/1864): #1831: proxy and transcode jobs run one at a time, in the order they were asked for

Should proxy/transcode work run in one global FIFO queue, including synchronous `wait:true` calls?

**Options:** Serialize every job in request order; or allow synchronous/independent jobs to start without waiting for unrelated background work.

**Recommendation:** Choose a resource-ordering contract and test that synchronous requests do not unexpectedly wait behind unrelated work.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1864#pullrequestreview-5566553250). Confidence: high; current-head re-review: None.

### D138 — [PR #1858](https://github.com/storytold/filmcraft/pull/1858): HEVC Main software encoder (I/P, single-slice) with export wiring

Should the new HEVC software encoder be automatically wired as the default fallback, feature-gated/explicit, or deferred while hardware HEVC PRs land?

**Options:** Default built-in fallback; behind a feature/explicit setting; or defer wiring. Coordinate with #1575, #1648, #1774 and #1612.

**Recommendation:** Use a feature/explicit setting or defer default wiring until output quality and hardware-backend priority are settled; author explicitly offers those options.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1858#issuecomment-6259027555). Confidence: high; current-head re-review: None.

### D139 — [PR #1851](https://github.com/storytold/filmcraft/pull/1851): #905: the Hard Limiter's Look-Ahead Time sets how early the gain falls

Accept the change that corrects look-ahead timing for existing projects, including projects storing 30 ms.

**Options:** Apply corrected timing to existing stored values; or preserve older output and migrate/default older projects differently.

**Recommendation:** Accept only with explicit compatibility documentation and a regression showing export loudness limiter output remains unchanged where promised.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1851#issuecomment-6259331879). Confidence: medium; current-head re-review: None.

### D140 — [PR #1850](https://github.com/storytold/filmcraft/pull/1850): #1593: the Timeline zooms out no farther than its whole extent in view

Should Timeline zoom-out floor recalculate with window width and automatically raise an already zoomed-out view?

**Options:** Keep dynamic floor and auto-adjust; preserve prior zoom while changing available width; establish explicit user expectation.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1850). Confidence: medium; current-head re-review: None.

### D141 — [PR #1823](https://github.com/storytold/filmcraft/pull/1823): interchange: preserve EDL round-trip rates and accept explicit import rates

Decide whether to write the frame-rate comment to every EDL, changing standard output for external editors.

**Options:** Write comment to every EDL; or only when rate is ambiguous / make it an export option.

**Recommendation:** Limit metadata change to ambiguous rates unless Premiere compatibility evidence supports unconditional output change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1823#issuecomment-6307041054). Confidence: high; current-head re-review: None.

### D142 — [PR #1808](https://github.com/storytold/filmcraft/pull/1808): #1061: watchdog for a graphics start that hangs: save the next mode, and the next launch ends the stuck start

Approve watchdog timeout and automatic renderer downgrade policy, including process termination capability.

**Options:** Keep 30s threshold and saved downgrade chain; adjust timeout based on cold-start measurements; reject automatic downgrade/process ending.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1808). Confidence: high; current-head re-review: None.

### D143 — [PR #1806](https://github.com/storytold/filmcraft/pull/1806): Program monitor: Go to Previous / Next Edit Point in the Button Editor, beside the frame steps

Choose whether existing saved transport layouts should automatically include the new edit-point buttons.

**Options:** Allow new buttons to appear in layouts whose saved hidden-button list predates them; or migrate saved layouts to preserve prior visibility defaults.

**Recommendation:** Preserve the intended Premiere-style discoverability with an explicit one-time migration and document the layout change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1806#pullrequestreview-5567088855). Confidence: medium; current-head re-review: None.

### D144 — [PR #1801](https://github.com/storytold/filmcraft/pull/1801): fix(engine): razor and copy select the individual created clips (#1703)

After Add Edit/razor/copy, should selection move to the newly created right-hand clips?

**Options:** Select the newly created clips; or preserve the prior selection / leave new clips unselected.

**Recommendation:** Decide and document the selection behavior as an explicit timeline product rule; the reviewer says the current change may be right but changes behavior for everyone.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1801#pullrequestreview-5567094396). Confidence: medium; current-head re-review: None.

### D145 — [PR #1799](https://github.com/storytold/filmcraft/pull/1799): edit: preserve audio source choices across through edits

Accept the editing-core rule changes needed to preserve source choices and link IDs for locked tracks across through edits.

**Options:** Adopt the behavior and associated new invariant; or split the focused through-edit fix from broader editing-core semantics.

**Recommendation:** Split the through-edit behavior and review link-ID/locked-track implications separately before accepting the new editing invariant.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1799#pullrequestreview-5567097413). Confidence: medium; current-head re-review: None.

### D146 — [PR #1784](https://github.com/storytold/filmcraft/pull/1784): perf: auto-scale memory budgets, GOP cache, and GPU upload cache to host RAM/VRAM

Choose the automatic calibration curve and preference scope for memory/cache budgets.

**Options:** Apply new calibration below RAM threshold; or preserve current defaults and make calibration opt-in; store settings per session/preferences.

**Recommendation:** Avoid overriding user-selected budgets; make any automatic calibration explicit and persist only as an intentional user preference.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1784#pullrequestreview-5567119697). Confidence: medium; current-head re-review: None.

### D147 — [PR #1778](https://github.com/storytold/filmcraft/pull/1778): Ultra Key: colour-difference matte for green/blue screens; red edges solved from their surroundings

Should the new colour-difference Ultra Key behavior replace rendering for existing saved projects?

**Options:** Change all existing projects; or preserve their prior matte behavior and use the new method only for newly applied effects/explicit mode.

**Recommendation:** Preserve prior output for saved projects through a version/mode parameter or a new effect identity unless maintainers explicitly accept silent render changes.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1778#pullrequestreview-5567125412). Confidence: high; current-head re-review: None.

### D148 — [PR #1765](https://github.com/storytold/filmcraft/pull/1765): Project panel: Select All / Deselect All from the macOS menu act on the focused panel with a sequence open

Should edit.selectAll / edit.deselectAll control-channel commands act on whichever UI panel has focus, or preserve literal sequence-wide API semantics?

**Options:** Keep focus-sensitive behavior for native menu and control channel; separate UI focus routing from remote commands so control calls remain literal.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1765). Confidence: high; current-head re-review: None.

### D149 — [PR #1762](https://github.com/storytold/filmcraft/pull/1762): Unify FilmCraft standard Dark chrome with ArtCraft palette

Decide whether the suite palette becomes default on upgrade for users with no saved theme preference.

**Options:** Migrate default from Premiere-style Darkest to new Dark palette; or keep current default and offer suite palette as opt-in.

**Recommendation:** Keep existing default for compatibility until maintainers approve the broad visual change based on side-by-side screenshots.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1762#pullrequestreview-5567177331). Confidence: high; current-head re-review: None.

### D150 — [PR #1729](https://github.com/storytold/filmcraft/pull/1729): codecs: the frame rate is what a clip's rounded durations average to (#1700)

Choose the relative 0.5% frame-rate tolerance behavior and preferred rounding pattern for unusual PTS sequences.

**Options:** Use relative tolerance; retain prior absolute tolerance; compare competing 90 kHz patterns against shared fixture.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1729). Confidence: medium; current-head re-review: None.

### D151 — [PR #1647](https://github.com/storytold/filmcraft/pull/1647): ui: highlight selected items in the project bin (#1445)

Should selected project-bin rows use the accent highlight, the existing row-selected tint, or both?

**Options:** Use accent stroke, row tint, or both knowingly.

**Recommendation:** Choose a consistent selection style because both currently layer on list rows while cards get only the tint.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1647#pullrequestreview-5567471219). Confidence: medium; current-head re-review: None.

### D152 — [PR #1641](https://github.com/storytold/filmcraft/pull/1641): fix(files): prevent silent replacement of existing files

Should APIs/scripts/CLI refuse to replace existing paths by default, or preserve the existing overwrite behavior outside interactive UI?

**Options:** Require explicit `overwrite: true` for all callers; or have GUI prompt while scripts/CLI that name a path retain prior replace behavior (with an explicit CLI flag as an alternative).

**Recommendation:** Decide the caller contract separately from the UI Replace/Cancel flow; maintainers explicitly say this default decision remains open.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1641#issuecomment-6329425767). Confidence: high; current-head re-review: None.

### D153 — [PR #1635](https://github.com/storytold/filmcraft/pull/1635): timeline: fade handles on a clip's top corners apply and size the default transition

Should clicking a clip’s top corner change the frequent select gesture into adding/resizing a transition, and should selection-modifier clicks preserve old behavior?

**Options:** Accept corner handle behavior; preserve select gesture; ignore modifier presses to retain prior gesture semantics.

**Recommendation:** Resolve explicitly with the linked PR/issue owners; preserve documented contract unless the maintainer approves the behavior change.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1635). Confidence: high; current-head re-review: None.

### D154 — [PR #1633](https://github.com/storytold/filmcraft/pull/1633): theme: a selected row is tinted with the highlight colour, so it reads as selected

Approve broader selected-row tint change across all default dark/medium theme surfaces and its interaction with card styling.

**Options:** Adopt token change across app and cards; or restrict selected tint to intended list rows / coordinate with card token changes.

**Recommendation:** Review screenshot evidence across every affected surface, including theme cards, then choose token scope deliberately.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1633#issuecomment-6260720927). Confidence: medium; current-head re-review: None.

### D155 — [PR #1629](https://github.com/storytold/filmcraft/pull/1629): status bar: an error is drawn larger, in the danger colour, and stays longer

Choose whether every refused command should appear as red error text for 20 seconds rather than dim status for 8 seconds.

**Options:** Adopt global emphasis for all refusals; or limit strong treatment to serious errors and retain shorter routine status messages.

**Recommendation:** Differentiate user-correctable failures from routine refusals so common “nothing selected” feedback does not dominate the status bar.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1629#pullrequestreview-5567558554). Confidence: high; current-head re-review: None.

### D156 — [PR #1628](https://github.com/storytold/filmcraft/pull/1628): timeline: right-clicking the ruler opens the ruler's menu, not a stale clip menu

Should right-clicking the ruler’s Mark In/Out menu apply at the click position or the current playhead position?

**Options:** Use click time; preserve current playhead behavior matching the Mark menu.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1628). Confidence: medium; current-head re-review: None.

### D157 — [PR #1626](https://github.com/storytold/filmcraft/pull/1626): timeline: a linked clip moved to another track keeps its partner on its own track

Approve changed behavior when moving a mixed selection containing linked and unlinked clips.

**Options:** Move each linked partner on its own track; or preserve previous behavior for mixed unlinked selections.

**Recommendation:** Approve only with a clear user-facing rule and reconcile overlapping timeline PRs before landing.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1626#pullrequestreview-5567583159). Confidence: medium; current-head re-review: None.

### D158 — [PR #1623](https://github.com/storytold/filmcraft/pull/1623): timeline: drop transitions when an edit moves their clips apart

Should orphan-transition cleanup become an engine-wide invariant after any edit to a sequence?

**Options:** Sweep all transitions after any sequence edit (including unrelated edits); or limit cleanup to commands that change clip adjacency.

**Recommendation:** Choose and document the broader invariant because it can silently drop transitions during unrelated edits and changes all existing callers.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1623#pullrequestreview-5567595581). Confidence: high; current-head re-review: None.

### D159 — [PR #1613](https://github.com/storytold/filmcraft/pull/1613): timeline: snap transition drops to Start/Center/End at Cut

Approve changing transition-drop alignment at cuts to Start/Center/End, including fades to nothing labeled Cut.

**Options:** Use current alignment and Cut label; or retain In/Out wording for one-sided fades.

**Recommendation:** Use labels that describe whether there is a cut; accept alignment behavior only after confirming Premiere parity.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1613#pullrequestreview-5567621384). Confidence: medium; current-head re-review: None.

### D160 — [PR #1599](https://github.com/storytold/filmcraft/pull/1599): timeline, monitors: the zoom bar's handles zoom about the playhead

When a user drags a timeline/monitor zoom-bar handle, should the opposite edge stay fixed or should the view center toward the playhead?

**Options:** Preserve #436’s opposite-edge-fixed behavior; or adopt centered-on-playhead behavior from #1153; also decide whether out-of-view playheads should retain old behavior.

**Recommendation:** Resolve whether #436’s maintainer instruction still stands, then make the implementation match #1153’s stated rule or document a deliberate variant.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1599#pullrequestreview-5567655089). Confidence: high; current-head re-review: None.

### D161 — [PR #1597](https://github.com/storytold/filmcraft/pull/1597): timeline, panels: only the primary button starts a gesture; a right-button drag no longer draws a box

Decide whether remaining right-button gesture starts on dock grip, window drag, and Effect Controls graph should be changed as a follow-up.

**Options:** Leave them as-is; extend primary-button-only behavior to all gesture starts; split by panel.

**Recommendation:** Record the choice on the PR or associated issue; keep the change scoped to the chosen route.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1597). Confidence: medium; current-head re-review: None.

### D162 — [PR #1580](https://github.com/storytold/filmcraft/pull/1580): #1221: H.264 High 10 and High 10 Intra export in the built-in encoder

Decide whether to accept the generic-code performance cost of High 10 support and whether to split High 10 from intra-only encoding.

**Options:** Accept measured ~1% cost for all default 8-bit exports and keep combined; or specialize/split independent features and require per-build measurements.

**Recommendation:** Split 10-bit sample support from intra-only mode and require reproducible performance evidence before accepting a global generic-path cost.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1580#pullrequestreview-5567676785). Confidence: medium; current-head re-review: None.

### D163 — [PR #1578](https://github.com/storytold/filmcraft/pull/1578): Timeline: Select Clip at Playhead selects the gap over empty time

Confirm Premiere-parity behavior for Select Clip at Playhead selecting gaps during empty time.

**Options:** Adopt proposed selection semantics; choose another track order or retain current behavior.

**Recommendation:** Defer to maintainers; evidence does not establish a settled choice.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1578). Confidence: medium; current-head re-review: None.

### D164 — [PR #1557](https://github.com/storytold/filmcraft/pull/1557): Export: captions past the last clip no longer extend the range

Should captions that extend beyond the final clip extend default export duration?

**Options:** Keep export ending at last clip; extend for outlasting captions/end cards.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1557). Confidence: high; current-head re-review: None.

### D165 — [PR #1520](https://github.com/storytold/filmcraft/pull/1520): Translate the README into Arabic (README.ar.md)

Should FilmCraft carry a translated Arabic README, and who keeps it synchronized with English?

**Options:** Carry and maintain an Arabic copy with an owner/update rule; or omit the translated README and its navigation link.

**Recommendation:** Accept only with a clear owner or rule that keeps it current with main; the review explicitly leaves the decision to maintainers.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1520#pullrequestreview-5567758231). Confidence: high; current-head re-review: None.

### D166 — [PR #1495](https://github.com/storytold/filmcraft/pull/1495): timeline: dragging a track's edge resizes only that track

Should track-specific heights persist per sequence/project, and should height commands preserve each track’s offset or reset it?

**Options:** Persist per-track maps (bounded and validated); keep shared-only heights; choose reset or relative-preservation semantics for increase/decrease and ui.set.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1495). Confidence: high; current-head re-review: None.

### D167 — [PR #1469](https://github.com/storytold/filmcraft/pull/1469): Add Arabic (العربية) interface translation

Which Arabic text presentation should FilmCraft support in the whole interface, given competing RTL implementations?

**Options:** Use this PR’s per-word visual-order preprocessing while retaining LTR chrome; choose #1886/#1740’s approach; or defer RTL presentation and ship Arabic strings without claiming full RTL.

**Recommendation:** Select one shared Arabic/RTL policy before merging one of the competing `Language::Ar` implementations; validate wrapping and interpolation rather than relying only on short labels.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1469#pullrequestreview-5567807651). Confidence: medium; current-head re-review: None.

### D168 — [PR #1446](https://github.com/storytold/filmcraft/pull/1446): fix(monitor): the playback and paused resolution set the rendered frame size

Should monitor rendering remove the pixel cap by default, increasing 4K playback work?

**Options:** Render at requested scale; or retain playback cap / limit uncapped rendering to paused view / add setting.

**Recommendation:** Decide after measuring frame time and dropped frames at default settings; separate playback and paused policies if needed.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1446#pullrequestreview-5567842290). Confidence: high; current-head re-review: None.

### D169 — [PR #1438](https://github.com/storytold/filmcraft/pull/1438): #1437: mpegts: drop TEI/duplicate packets, skip scrambled, first PMT wins

Should MPEG-TS accept only the first PMT or incorporate later PMT changes/new PIDs?

**Options:** Keep first PMT and ignore later updates; merge new PIDs and warn on changed/removed entries.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1438). Confidence: high; current-head re-review: None.

### D170 — [PR #1380](https://github.com/storytold/filmcraft/pull/1380): #1229: Finder opens the project it hands the app

Decide how opening a project delivered to an already-running app handles dirty current projects.

**Options:** Prompt before replacing dirty project; or replace immediately / specify another policy.

**Recommendation:** Prompt before destructive replacement, consistent with normal project opening safeguards.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1380#pullrequestreview-5567953792). Confidence: high; current-head re-review: None.

### D171 — [PR #1273](https://github.com/storytold/filmcraft/pull/1273): color: tone-map iPhone HLG footage consistently to SDR

Decide whether the new tone mapping changes existing HLG appearance by default or is opt-in for compatibility.

**Options:** Change existing project appearance by default; or gate behind Interpret Footage option so existing projects retain look.

**Recommendation:** Preserve old-project appearance and make new interpretation opt-in until reproducible reference measurements establish parity.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1273#pullrequestreview-5568143307). Confidence: high; current-head re-review: None.

### D172 — [PR #1256](https://github.com/storytold/filmcraft/pull/1256): Linux: VA-API H.264 export encoding (AMD / Intel), on main's `vaapi` module (#30)

Choose device-selection semantics for VA-API Auto encoding when multiple render nodes are available.

**Options:** Document/use first available render node; select preferred device explicitly; try subsequent devices after failure.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1256). Confidence: medium; current-head re-review: None.

### D173 — [PR #1255](https://github.com/storytold/filmcraft/pull/1255): ui: prevent Properties controls from overlapping on resize

Is a 360-point minimum width for the Properties dock acceptable across layouts and floating panels?

**Options:** Accept 360-point minimum; or choose a smaller/adaptive minimum or apply only to selected layouts.

**Recommendation:** Decide using default workspaces at 1280/1440-point laptop widths and assess space taken from Timeline/monitors.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1255#pullrequestreview-5568164754). Confidence: high; current-head re-review: None.

### D174 — [PR #1222](https://github.com/storytold/filmcraft/pull/1222): Transition edge drags keep their alignment

Confirm Premiere behavior for dragging transition edges and choose parity semantics for alignment.

**Options:** Adopt PR behavior if Premiere shifts the opposite edge; retain documented current behavior if Premiere leaves it fixed; verify exact cases in Premiere 26.5.2.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1222). Confidence: high; current-head re-review: None.

### D175 — [PR #1197](https://github.com/storytold/filmcraft/pull/1197): fix(timeline): reject selection transitions on locked tracks (#1124)

When a multi-selection transition edit touches locked tracks, should the whole action reject or skip locked edges?

**Options:** Reject whole action (move-style atomic behavior); or skip locked clips/edges (slip/trim-style partial behavior).

**Recommendation:** Use an explicit consistent timeline command policy; current PR follows atomic move behavior to avoid silent partial edits.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1197). Confidence: medium; current-head re-review: None.

### D176 — [PR #1196](https://github.com/storytold/filmcraft/pull/1196): transcript: background speech jobs and editable transcript workflows

Decide how a diarization refusal (>512 speech chunks) affects an already completed transcription.

**Options:** Propagate error and discard transcript result; or keep completed transcription while reporting speaker-labeling failure separately.

**Recommendation:** Preserve the completed transcription and report diarization as a partial failure.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1196#pullrequestreview-5568203773). Confidence: medium; current-head re-review: None.

### D177 — [PR #1181](https://github.com/storytold/filmcraft/pull/1181): feat(codecs): scale decoded-frame GOP budget with host physical RAM

Should each active GOP cache budget scale with physical RAM, and what evidence should support the claimed playback benefit?

**Options:** Scale per-cache budgets with pool; scale only shared pool and keep per-cache cap; defer default increase until measured.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1181). Confidence: high; current-head re-review: None.

### D178 — [PR #1179](https://github.com/storytold/filmcraft/pull/1179): MCP/Automation: Agent control server, UI map, and automation IDs

Choose behavior when control connect fails but FilmCraft is already open without the control port.

**Options:** Launch another app instance and wait; or detect existing app and report/reuse it without duplicate launch.

**Recommendation:** Avoid silently starting a second editor instance; expose a clear failure or attach to a supported existing process.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1179#pullrequestreview-5569645095). Confidence: medium; current-head re-review: None.

### D179 — [PR #1177](https://github.com/storytold/filmcraft/pull/1177): Text-based editing: Transcript panel, transcript commands, and transcript engine

Should transcript generation commands use the changed defaults/foreground execution behavior proposed here?

**Options:** Accept new defaults and execution behavior; or preserve previous/default async semantics and document separately.

**Recommendation:** Approve documented defaults and thread behavior only after considering UI responsiveness and command-schema accuracy.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1177#pullrequestreview-5569681402). Confidence: medium; current-head re-review: None.

### D180 — [PR #1173](https://github.com/storytold/filmcraft/pull/1173): fix(export): validate filenames in Export and Quick Export (#1158)

Set cross-platform path-character policy for colon and backslash, including compatibility with existing export callers.

**Options:** Reject `:` and `\` on every OS; or reject only platform-invalid characters and preserve native valid names.

**Recommendation:** Apply platform-native validity rules and preserve Unix backslash filenames; document any compatibility break before rollout.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1173#issuecomment-6332227876). Confidence: high; current-head re-review: None.

### D181 — [PR #1170](https://github.com/storytold/filmcraft/pull/1170): Prefer Microsoft YaHei UI for Windows Simplified Chinese

Should Windows Simplified Chinese font ordering put YaHei UI before Japanese forms, and should installed font override embedded craft-fonts?

**Options:** Reverse #363 ordering/use installed YaHei; or keep established Chinese-after-Japanese and embedded font precedence.

**Recommendation:** Decide the zh-CN-only precedence explicitly; require cross-platform/reinstall receipts before broad adoption.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1170#pullrequestreview-5569723739). Confidence: high; current-head re-review: None.

### D183 — [PR #1122](https://github.com/storytold/filmcraft/pull/1122): Window: redesigned menu bar with title and mode bars, system title bar option

Should custom-drawn window frames become the default on Windows/Linux, and should frame layout be split from the default flip?

**Options:** Set `systemTitleBar=false` broadly; default only on platforms with receipts; or land custom frame separately while deciding default later.

**Recommendation:** Gate default flip on platform receipts; consider splitting layout from default. Decide translation completion and platform validation as acceptance gates.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1122#pullrequestreview-5569849161). Confidence: high; current-head re-review: None.

### D184 — [PR #1077](https://github.com/storytold/filmcraft/pull/1077): #1065: DNG raw settings, faster decode, opcode lists, lossy JPEG DNG

Decide whether public ProcessOptions should accept Highlights::Recover when the implementation behaves like Clip.

**Options:** Keep the variant and implement true Recover; or reject/remove it until supported.

**Recommendation:** Reject unsupported Recover at the library boundary until implemented, matching engine behavior.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1077#pullrequestreview-5569933290). Confidence: medium; current-head re-review: True.

### D185 — [PR #553](https://github.com/storytold/filmcraft/pull/553): linux: one top bar — the header is the window's title bar (PhotoCraft's caption buttons)

Should the Linux custom in-app title bar replace server-side decorations by default across all Linux sessions?

**Options:** Prefer winit Adwaita client-side decorations for GNOME Wayland; or make the custom bar Wayland-only/opt-in and retain native decorations elsewhere.

**Recommendation:** Keep server-side decorations as the default outside a clearly supported Wayland/opt-in path; the maintainer explicitly says the global Linux default is too broad.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/553#issuecomment-6096129740). Confidence: high; current-head re-review: None.

### D182 — [PR #1156](https://github.com/storytold/filmcraft/pull/1156): i18n: add Czech (cs) interface language

Does the maintainer accept the Czech language addition on its current scope after the Czech-only coverage gate was removed?

**Options:** Accept the Czech catalog on the current shared localization approach; or request further changes before accepting it.

**Recommendation:** Treat removal of the Czech-only test as settled; decide language inclusion only against the current catalog and stated localization acceptance criteria.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1156#pullrequestreview-5569747016). Confidence: medium; current-head re-review: True.

## Scope, issue closure and ownership

### D186 — [PR #2465](https://github.com/storytold/filmcraft/pull/2465): Preserve remix seams, waveform and manual Out trimming

Decide whether to accept the PR as bundled or split its four independent changes.

**Options:** Land together; or split seam levels, waveform, manual Out trimming and related edits into independently reviewable PRs.

**Recommendation:** Split unrelated concerns so the rendering-default decision can be reviewed independently.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2465#pullrequestreview-5570974852). Confidence: high; current-head re-review: None.

### D187 — [PR #2460](https://github.com/storytold/filmcraft/pull/2460): #2264, #2265: copy, paste and duplicate graphic layers

Confirm whether the PR claims to satisfy Ctrl+D duplication requirements or only adds menu/context-menu duplication.

**Options:** Keep Cmd+D on Apply Video Transition and describe menu-based duplicate support; or change shortcut semantics/issue scope.

**Recommendation:** Keep the existing Cmd+D binding because it is already assigned, and state clearly that the issue’s shortcut request remains out of scope.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2460#pullrequestreview-5571024917). Confidence: medium; current-head re-review: None.

### D188 — [PR #2455](https://github.com/storytold/filmcraft/pull/2455): #2194: FCPXML <title> imports as editable text instead of being skipped

Should this FCPXML title importer close linked issue #2194, or treat the PR as a partial reference?

**Options:** Declare issue fixed/close it; or change link semantics to “Refs #2194” and leave issue open.

**Recommendation:** Use “Refs” if issue scope exceeds editable titles; close only if maintainers consider the full issue satisfied.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2455#pullrequestreview-5571071158). Confidence: medium; current-head re-review: None.

### D189 — [PR #2398](https://github.com/storytold/filmcraft/pull/2398): #1364: the caption track style moves captions up or down

Should caption vertical margin be disabled or explained for Middle anchor/WebVTT line overrides, and should the feature be per-caption?

**Options:** Add per-caption offsets if required by #1364; keep track style only; gate margin when it has no effect.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2398). Confidence: medium; current-head re-review: None.

### D190 — [PR #2240](https://github.com/storytold/filmcraft/pull/2240): Clarify the first-launch demo project name

Decide whether a rename alone satisfies issue #1669 or whether the demo should stop opening on startup / use a Home screen.

**Options:** Accept rename as sufficient and close issue; or mark PR as partial and retain the broader first-launch issue.

**Recommendation:** Keep the issue open as a partial fix unless the maintainer confirms rename alone resolves the startup concern.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2240#pullrequestreview-5566070541). Confidence: high; current-head re-review: None.

### D191 — [PR #2187](https://github.com/storytold/filmcraft/pull/2187): Improve phrase-aware music remix planning

Should phrase-aware remix force melody matching on, and should the large implementation be split into focused PRs?

**Options:** Keep a user control or always enable matching; decide whether to split search, seam handling, waveform and UI work.

**Recommendation:** Resolve explicitly in the PR discussion before merge; prefer documented policy and Premiere parity where applicable.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2187). Confidence: high; current-head re-review: None.

### D192 — [PR #2147](https://github.com/storytold/filmcraft/pull/2147): fix(import): import files a macOS drag only promised (Photos.app videos arrived as JPEGs)

Determine if the remaining three file dialogs need a follow-up routed through pick_ui due to modal-loop risk.

**Options:** Leave those dialogs on the existing path; or scope a follow-up to route them through pick_ui.

**Recommendation:** Track a follow-up if these dialogs share the nested-modal risk, but keep the current fix narrowly scoped.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2147#pullrequestreview-5566178058). Confidence: medium; current-head re-review: None.

### D193 — [PR #2075](https://github.com/storytold/filmcraft/pull/2075): fix(macOS): show file dialogs as sheets instead of a nested modal run loop (#2018)

Decide whether Linux Save As should also split a full output path into suggested basename and parent folder.

**Options:** Keep Linux behavior as-is; or align it with the macOS path behavior in this PR.

**Recommendation:** Keep this PR focused; file a separate Linux fix unless parity is part of the acceptance scope.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2075#issuecomment-6280580221). Confidence: medium; current-head re-review: None.

### D194 — [PR #2075](https://github.com/storytold/filmcraft/pull/2075): fix(macOS): show file dialogs as sheets instead of a nested modal run loop (#2018)

Decide whether remaining file dialogs with nested-modal risk should be routed through pick_ui.

**Options:** Leave three dialogs on existing implementation; or schedule a follow-up to route all affected dialogs through pick_ui.

**Recommendation:** Track the remaining risk as a separate scoped follow-up; the reported repro fix can remain narrow.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2075#pullrequestreview-5571735616). Confidence: medium; current-head re-review: None.

### D195 — [PR #2012](https://github.com/storytold/filmcraft/pull/2012): Add Linux file-manager video drops to Source and Timeline

Should Linux file-drop implementation reject non-boolean `linked` values that main currently treats as follow-links, and how should split scope be handled?

**Options:** Preserve permissive coercion; reject malformed types; split engine/API behavior from Linux drops and keep only this PR’s changes.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2012). Confidence: medium; current-head re-review: None.

### D196 — [PR #1892](https://github.com/storytold/filmcraft/pull/1892): cleanup: remove unused dependencies, dead code and committed target/ fixtures

Choose whether independent fixture, dependency and source cleanup changes should be split for review.

**Options:** Keep the four commits bundled; or separate the clearly independent cleanup scopes.

**Recommendation:** Split API deletions from safe fixture/dependency cleanup so the public API policy can be decided independently.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1892#pullrequestreview-5566523198). Confidence: medium; current-head re-review: None.

### D198 — [PR #1847](https://github.com/storytold/filmcraft/pull/1847): fix(render): apply nested audio channel picks before downmix

Choose whether to split nested-audio correction from dialog limits, command validation and AGENTS policy update.

**Options:** Keep all together; or split four independent concerns.

**Recommendation:** Separate the rendering correctness change from limits and policy text for independent approval.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1847#pullrequestreview-5566591224). Confidence: medium; current-head re-review: None.

### D199 — [PR #1827](https://github.com/storytold/filmcraft/pull/1827): codecs, export: uncompressed QuickTime video reads and writes (2vuy, v210, RGB)

Should the PR include a new uncompressed QuickTime export format and UI presets, or split reader and writer scope?

**Options:** Land reader only; accept large uncompressed export and its new control/presets; split reader/writer and require format compatibility evidence.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1827). Confidence: high; current-head re-review: None.

### D200 — [PR #1806](https://github.com/storytold/filmcraft/pull/1806): Program monitor: Go to Previous / Next Edit Point in the Button Editor, beside the frame steps

Decide whether this PR closes #1800 even though it implements optional transport buttons rather than the requested default behavior.

**Options:** Keep Fixes #1800; change to Refs #1800 and leave broader request open.

**Recommendation:** Change to Refs #1800 so issue closure reflects actual partial scope.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1806#pullrequestreview-5567088855). Confidence: high; current-head re-review: None.

### D201 — [PR #1751](https://github.com/storytold/filmcraft/pull/1751): codecs: preserve MP4 Dolby surround channels

Should this PR mark issue #1701 fixed if its AAC 5.1 report remains unaddressed?

**Options:** Use Fixes and close issue; use Refs/keep issue open; split the issue or declare AAC report separate.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1751). Confidence: high; current-head re-review: None.

### D202 — [PR #1660](https://github.com/storytold/filmcraft/pull/1660): window: the OS title bar shows the open project (file name, project name, or Untitled) (#1059)

Does issue #1059 require showing the containing folder as well as the project filename/title, and should issue closure wait for reporter confirmation?

**Options:** Treat filename/project name/Untitled as satisfying the request and close; or keep the issue open until the reporter confirms folder display was not intended.

**Recommendation:** Ask the reporter or leave issue closure to maintainers after confirmation.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1660#pullrequestreview-5567409610). Confidence: medium; current-head re-review: None.

### D203 — [PR #1631](https://github.com/storytold/filmcraft/pull/1631): view: Centered Audio Waveforms draws the timeline waveform on both sides of its centre line

Does centered waveform support include per-channel lanes, or only a centered mixed-peak waveform?

**Options:** Treat centered mixed waveform as sufficient; expand scope to separate per-channel lanes under #1567.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1631). Confidence: medium; current-head re-review: None.

### D204 — [PR #1605](https://github.com/storytold/filmcraft/pull/1605): timecodes: Effect Controls, the Audio Track Mixer and the Reference Monitor scrub and take a typed time; one font for them all

Should the three new typed timecode scrub fields also change Timeline and monitor timecode fonts from Premiere-matching proportional fonts to a unified font?

**Options:** Keep existing Timeline/monitor fonts and limit this PR to new fields; unify fonts; use monospace for stable digit alignment.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1605). Confidence: high; current-head re-review: None.

### D205 — [PR #1179](https://github.com/storytold/filmcraft/pull/1179): MCP/Automation: Agent control server, UI map, and automation IDs

Choose whether to accept the ~1,600-line worker-process test harness and its worker spawning.

**Options:** Keep the infrastructure; or slim/split it into narrower test helpers.

**Recommendation:** Keep only if the ongoing maintenance cost is justified by reproducible integration coverage; otherwise split or reduce.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1179#pullrequestreview-5569645095). Confidence: medium; current-head re-review: None.

## Acceptance and validation

### D206 — [PR #2839](https://github.com/storytold/filmcraft/pull/2839): FreeBSD: package the aarch64 cross build (craft-common-actions#39)

Decide when to opt releases back into FreeBSD aarch64 and what runtime acceptance evidence is required for the cross-built package.

**Options:** Keep x86_64 only until a real aarch64 build/package validation is available; or opt into aarch64 with a clearly stated cross-build/runtime support policy.

**Recommendation:** Land the packaging correction separately, validate the real cross build, then make the release-architecture opt-in decision.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2839). Confidence: medium; current-head re-review: True.

### D207 — [PR #2488](https://github.com/storytold/filmcraft/pull/2488): Windows: add Media Foundation H.264 and HEVC export

What rollout and shared-D3D11 acceptance gates are required for Windows Media Foundation H.264/HEVC export?

**Options:** Accept current evidence and rollout broadly; or require exact-head hardware export receipts across adapters/platforms before rollout.

**Recommendation:** Require exact-head export acceptance and maintainer decision on rollout/shared-D3D11, particularly Intel coverage; historical Qualcomm/AMD evidence is insufficient.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/2488). Confidence: high; current-head re-review: None.

### D208 — [PR #1796](https://github.com/storytold/filmcraft/pull/1796): Harden audio-dsp and speech against hostile input (+ fuzz tests for audio crates)

Choose a product policy for oversized FFT buffer behavior, extreme resampling ratios/logging, and timing-based fuzz hang assertions.

**Options:** Return an error/log instead of panic for oversized FFT; choose supported ratio ceiling and one-time diagnostics; remove or relax elapsed-time assertions.

**Recommendation:** Make the product/architecture choice explicit in the PR or linked issue; preserve Premiere parity and current documented contracts where evidence supports them.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1796). Confidence: medium; current-head re-review: None.

### D209 — [PR #1627](https://github.com/storytold/filmcraft/pull/1627): widgets: drag across checkboxes to check or uncheck several

Should `clippy::new_without_default`-style API patterns be enforced through a shared lint/configuration rule for widgets?

**Options:** Add a project lint/allowlist route in clippy.toml; or leave API consistency to review without a new shared rule.

**Recommendation:** Choose a repository-wide consistency rule separately from this checkbox behavior.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1627#pullrequestreview-5567571180). Confidence: medium; current-head re-review: None.

### D210 — [PR #1575](https://github.com/storytold/filmcraft/pull/1575): Media Foundation hardware H.264 / HEVC encoding on Windows (#30)

What exact-head Media Foundation hardware encoding rollout and shared-D3D11 acceptance should gate merge?

**Options:** Accept existing device results; or require exact-head evidence across adapter classes and sequence/export conditions.

**Recommendation:** Require adapter-specific acceptance including Intel and shared-D3D11 behavior; historical success on other vendors is not sufficient.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1575#pullrequestreview-5567690346). Confidence: high; current-head re-review: None.

### D211 — [PR #1556](https://github.com/storytold/filmcraft/pull/1556): Graphics: show each family in its own face in the Font menu

How broadly should the font preview feature cover #1464, and what evidence is required for synchronous font-file reads?

**Options:** Keep this PR limited to Essential Graphics and clarify the issue scope; or extend the shared row to Graphics templates/Replace Fonts and possibly other pickers. For performance, require a frame-time receipt or adjust the budget/file-read strategy.

**Recommendation:** Decide whether #1464 means all font lists; label partial scope accurately or extend. Treat the large-font scroll measurement as an acceptance follow-up, not a code defect.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1556#pullrequestreview-5567729245). Confidence: high; current-head re-review: None.

### D212 — [PR #1182](https://github.com/storytold/filmcraft/pull/1182): feat(av1enc): clean-room pure-Rust baseline AV1 software encoder

Decide whether to enable this AV1 encoder by default before independent decode and quality evidence is available.

**Options:** Enable for all machines now; or keep experimental/off until ffmpeg/dav1d oracle and quality evidence are supplied.

**Recommendation:** Keep opt-in until independent decoder validation and representative quality/compatibility data satisfy the previously requested proof.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1182#pullrequestreview-5568484574). Confidence: high; current-head re-review: None.

### D213 — [PR #1078](https://github.com/storytold/filmcraft/pull/1078): #1065: CinemaDNG in MXF and camera raw Source Settings

Are the proposed machine-generated translations acceptable, or must they be reviewed by native speakers before merge?

**Options:** Accept machine translations; or require native-speaker review for affected catalogs.

**Recommendation:** Require native-speaker review for user-facing translations.

**Evidence:** [Recorded discussion/source](https://github.com/storytold/filmcraft/pull/1078#issuecomment-6279030948). Confidence: medium; current-head re-review: True.

