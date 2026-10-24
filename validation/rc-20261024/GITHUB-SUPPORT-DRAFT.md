# Draft: Actions disabled despite enabled permissions

This is a prepared draft, not a submitted support request.

Repository: `FredApps/filmcraft` (public fork of `storytold/filmcraft`).

On 2026-10-24, both push-triggered execution and manual dispatch of the dedicated RC workflow failed to create a run. The CLI dispatch returns:

```text
HTTP 422: Actions has been disabled for this repository.
```

The repository Actions permissions endpoint returns `enabled: true`, `allowed_actions: all`. The dedicated workflow, ID `395281842`, path `.github/workflows/rc-candidate.yml`, returns state `active`. Re-enabling repository Actions and the workflow did not clear the rejection. The `Release` workflow remains manually disabled. The RC workflow has read-only contents permission and builds/tests/packages the editor; it does not publish a release.

Please review whether this repository is in a separate GitHub-controlled disabled state and explain any steps required to restore normal Actions execution. I am not asking to bypass a restriction or a usage policy.

The exact commands, responses and timestamp are recorded in `github-actions-blocker.json`. Intended invocation:

```text
gh workflow run rc-candidate.yml -R FredApps/filmcraft --ref rc/20261024
```
