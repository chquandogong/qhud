<!-- qhud:languages -->
<p align="center">
  <a href="../i18n/ko/docs/05-ops/REPOSITORY.md">한국어</a> · <strong>English</strong> · <a href="../i18n/zh-CN/docs/05-ops/REPOSITORY.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# Repository operations

This document separates qhud's tracked workflows from GitHub settings, which can change without a commit. The repository, `main` and release-tag rulesets below were read from GitHub on **2026-09-15**. Security features, Actions settings, the release environment, and the zero-alert count were last checked on **2026-09-14**; recheck them before relying on them after a settings change. qhud is currently a **public repository maintained by one person**, with `main` as its default branch. The private-repository columns below are operating choices for a future change of visibility or team size, not claims about the present repository.

## Choose controls for the actual working situation

| Control | One-person private | Team private | Public qhud (current) |
| --- | --- | --- | --- |
| Visibility and intake | Keep source and operational issues visible only to the owner. Use an internal issue list and a private security channel. | Grant the smallest needed repository permissions; use internal issues and a private security channel. | Source and ordinary issues are public. Accept focused issues and PRs in English, Korean, or Chinese; route suspected vulnerabilities to private reporting. |
| `main` changes | Use a PR, the seven status checks, resolved review threads, linear history, and squash merge. An approval count of zero lets a sole maintainer merge after checks. | Keep the PR and checks; require at least one independent approval, consider code-owner review and approval of the last push, and assign an independent release reviewer. | The active ruleset requires a PR, seven strict checks, resolved review threads, linear history, and squash merge. Approval count is zero while there is only one maintainer; it does not waive the checks. |
| Releases | Restrict tag creation and publication to the owner; verify release notes and artifacts before sharing them. | Limit tag/release access to designated maintainers and require an independent release approval. | Protect `v*` tags and publish only through the gated workflow and `release` environment. Inspect the release draft before publication, then the four assets, checksums, and attestations. |
| Security and alerts | Check which GitHub security features the repository's plan permits; review alerts privately and avoid copying real secrets into tests. | Assign alert triage and remediation owners, and review security PRs independently. | Maintain private vulnerability reporting, dependency and code scanning, secret scanning and push protection; triage new alerts without putting exploit details in public issues. |

Do not copy a public repository's settings into a private repository without checking feature availability, billing, and the intended audience. If qhud gains another maintainer, change the approval policy only after the reviewer and release-approver roles are staffed and a test PR proves the new rule can be satisfied.

## Current branch and tag protection

The active [Protect main ruleset](https://github.com/chquandogong/qhud/rules/23268129) (ID `23268129`) applies to the default branch. It prevents deletion and non-fast-forward updates, requires linear history and a PR, allows **squash** as the PR merge method, requires review-thread resolution, and requires extra approval for unattributed changes. It asks for **zero approving reviews**, does not require code-owner review or last-push approval, and has **no bypass actors**. Required checks use the strict policy: the PR branch must be updated against `main` before merging. The [Protect release tags ruleset](https://github.com/chquandogong/qhud/rules/23267522) (ID `23267522`) applies to `refs/tags/v*`, prevents deletion, update, and non-fast-forward changes, and has no bypass actors. These rules are GitHub settings; the repository files alone cannot enforce them.

GitHub currently allows squash merges only, offers auto-merge and branch updating, and deletes a PR's head branch on merge. [CODEOWNERS](../../.github/CODEOWNERS) names `@chquandogong` for all changes and explicitly for `.github/`, `scripts/`, and `src-tauri/`. CODEOWNERS identifies the owner; the present ruleset does not make a code-owner approval mandatory.

The seven exact required status-check contexts are:

1. `fmt · clippy · test · build`
2. `Windows MSVC · test · build`
3. `docs · release metadata`
4. `dependency review`
5. `Analyze (rust)`
6. `Analyze (javascript-typescript)`
7. `Analyze (actions)`

The first three come from [CI](../../.github/workflows/ci.yml): Ubuntu 24.04 runs the Node system-metrics test, Rust formatting, Clippy with warnings treated as errors, Rust tests, and a release build; Windows 2022 tests and builds with the scoped dependency patch and checks that `Cargo.toml`/`Cargo.lock` remain unchanged; the integrity job runs [the repository checker](../../scripts/check-repository.mjs), including versions, release metadata, three-language document parity, and local links. CI runs on pushes to `main` and `codex/**`, on PRs, and as a reusable release gate. [Dependency review](../../.github/workflows/dependency-review.yml) runs on PRs and fails on newly introduced dependencies of moderate or higher severity. The three Analyze contexts come from GitHub CodeQL default setup, rather than a tracked CodeQL workflow; the last verified setup scanned Rust, JavaScript/TypeScript, and Actions on PRs and weekly. Check the CodeQL configuration and each context before changing a required-check name or language.

## Actions, dependency maintenance, and security

The last verified Actions policy allowed selected GitHub-owned actions plus `dtolnay/rust-toolchain` and `Swatinem/rust-cache`, with third-party actions pinned to full commit SHAs. The default workflow token was read-only. The workflow files also declare narrow permissions: CI and dependency review read contents; release packaging adds `id-token: write` and `attestations: write` for provenance; only the publish job has `contents: write`. Recheck both the repository-wide Actions policy and each job's permissions when adding an action or changing a release job.

[Dependabot configuration](../../.github/dependabot.yml) requests Cargo updates weekly on Monday at 09:00 Asia/Seoul and GitHub Actions updates monthly at 09:00, with at most five open PRs for each ecosystem and grouped minor/patch changes. The last verified GitHub settings enabled Dependabot security updates, private vulnerability reporting, secret scanning with push protection, and CodeQL default setup. Those server-side switches are separate from `dependabot.yml`. On **2026-09-14**, the security-warning queues inspected for this repository were empty (**0 alerts**). That is a dated observation, not a promise that no alert exists now. Review new code-scanning, dependency, and secret-scanning alerts, confirm the affected version and reachability, prioritize exposed credentials immediately, and record a redacted fix and validation in a PR. Do not publish a vulnerability's reproduction before coordinated disclosure; follow [SECURITY.md](../../SECURITY.md).

## PRs, issues, and release publication

Before a PR, search existing issues and make a focused change. The [PR template](../../.github/PULL_REQUEST_TEMPLATE.md) asks for an issue link or reason none is needed, behavior, exact validation, unverified behavior, a privacy/security check, and documentation in all three languages. Resolve review threads, update the branch for strict checks, and merge with squash only after all seven statuses pass. Auto-merge can wait for the gate; it does not bypass it. Ordinary bugs, feature requests, and support questions use the three [issue forms](../../.github/ISSUE_TEMPLATE/) with versions and environment details. Blank issues are disabled; the issue chooser links to documentation and a private security report. Issues are enabled, while Wiki, Projects, and Discussions are disabled. Redact credentials, account data, local paths, and screenshots before posting. Triage public issues by reproduction, supported version, platform, and whether the fault is in qhud or an upstream provider; use [SUPPORT.md](../../SUPPORT.md) for channel selection.

The [Release workflow](../../.github/workflows/release.yml) starts on `v*` tags. Preflight accepts only a stable `vX.Y.Z` tag, matching Cargo and Tauri versions, nonempty versioned release notes, a CHANGELOG entry, and a tagged commit already contained in `origin/main`. It calls the full CI quality gate, packages Linux x86_64 and Windows x86_64, creates SHA-256 files and build-provenance attestations, then verifies exactly four assets and their checksums. Publication goes through the `release` environment, first as a draft. It refuses to replace a published release or overwrite a draft asset whose bytes differ; after asset validation it publishes the draft. The last verified `release` environment required the maintainer as reviewer and allowed self-review for this one-person repository. **Immutable releases were enabled on 2026-09-14 for newly published releases; this does not retroactively change the existing `v0.7.1` release, whose API reports `immutable: false`.** Tag protection already prevents editing `v*` refs. The exact operator sequence is in the [runbook](RUNBOOK.md#release-procedure).

## Recheck after a GitHub or workflow change

1. Read repository metadata, both active rulesets, merge settings, Actions permissions, the `release` environment, CodeQL default setup, Dependabot/security switches, and current alert queues from GitHub. Record the check date. Do not infer a server setting from a YAML file.
2. Compare the required context names and CodeQL languages with the checks produced by a fresh test PR. Recheck strict branch updating, review-thread resolution, merge method, CODEOWNERS, and whether a sole maintainer can actually satisfy any new approval rule.
3. Run `node scripts/check-repository.mjs` and the relevant CI checks for tracked-file changes. For a release change, also verify tag ancestry, versions, three-language notes, draft approval, checksums, attestations, and the downloaded artifacts on the supported platforms.
4. Update this document and its Korean and Chinese mirrors with the observed settings, evidence date, and any gap between the intended and active policy. Treat an alert count as a time-stamped observation and recheck it before claiming closure.
