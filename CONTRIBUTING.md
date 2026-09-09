<!-- qhud:languages -->
<p align="center">
  <a href="docs/i18n/ko/CONTRIBUTING.md">한국어</a> · <strong>English</strong> · <a href="docs/i18n/zh-CN/CONTRIBUTING.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# Contributing to qhud

Thank you for helping make qhud clearer and more reliable. Issues, pull requests,
and documentation improvements are welcome in **English, 한국어, and 简体中文**.

## Before you start

- Read the [user guide](docs/GUIDE.md) for supported behavior and known limits.
- Search [existing issues](https://github.com/chquandogong/qhud/issues) before
  opening a new report. Include a small, reproducible example when possible.
- Discuss a large behavior or architecture change in an issue before investing
  in a broad implementation. Small fixes and translations can go directly to a PR.
- Keep each change focused. Explain the problem, the resulting behavior, and
  how you verified it.

## Report a bug

Use the [bug report form](https://github.com/chquandogong/qhud/issues/new?template=bug_report.yml).
Include the qhud version, OS/desktop, installation method, provider, reproduction
steps, expected result, and actual result. Distinguish fresh provider data from
stored snapshots. For a visual issue, describe whether the desktop is unlocked
and whether the tray, refresh, and window controls respond.

Diagnostics can contain emails, account IDs, local paths, session content, and
credentials. Inspect and redact material before sharing it. Never attach
`auth.json`, `.credentials.json`, or a complete provider configuration directory.
If a report concerns a credential exposure, describe the issue without posting
the credential in a public issue.

## Build and check

The repository declares Rust 1.88 or later; CI uses stable Rust. The frontend is
plain HTML/CSS/JavaScript and does not require npm. qmonster is pinned to the
revision in `src-tauri/Cargo.toml`.

### Linux

Install the development dependencies documented in the [runbook](docs/05-ops/RUNBOOK.md),
then run:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

### Windows

Use Developer PowerShell for Visual Studio with the C++ workload and Windows SDK:

```powershell
.\scripts\Build-Windows.ps1 -Test
git diff --exit-code -- Cargo.toml Cargo.lock
```

The script prepares the pinned qmonster checkout and applies the Windows patch
only for its Cargo commands. It restores the canonical lockfile afterward.
Do not run another Cargo command concurrently in that checkout. See the
[Windows guide](docs/05-ops/WINDOWS.md) for tool discovery and explicit paths.

## Protect the data contract

- Keep each quota attached to its account, organization, model scope, duration,
  and reset instant. Missing data must not become an invented zero.
- Preserve snapshot age and ownership. A successful parse is not proof that a
  stored value belongs to the current login.
- Keep provider requests behind explicit refresh actions. qhud does not own
  provider logins or run its own OAuth refresh grant.
- Keep Windows-specific adaptations scoped. Linux builds must continue to work
  from a normal clone without a developer's sibling dependency directory.
- On Linux, do not install Unix signal handlers in the Tauri/WebKitGTK process;
  see [D-012](docs/02-decisions/DECISION_LOG.md).

The [specification](docs/03-spec/SPEC.md), [architecture](docs/03-spec/ARCHITECTURE.md),
and [decision log](docs/02-decisions/DECISION_LOG.md) explain these constraints.

## Documentation and translations

The default `README.md` is Korean. `README.en.md` provides English and
`README.zh-CN.md` provides Simplified Chinese; keep `README.ko.md` identical to
`README.md` for existing links. Other English source documents remain at the
repository root and under `docs/`. Complete Korean and Simplified Chinese
mirrors live under `docs/i18n/ko/` and `docs/i18n/zh-CN/`, preserving the
source-relative document paths.

For a documentation change, update the matching translations. Preserve command
syntax, API identifiers, dates, requirement/decision IDs, and complete table
rows. Historical records describe their own date; do not silently rewrite an
old observation as a current fact. Keep the MIT license text in its original form.

Check relative links and section anchors from each document's actual directory.
Use existing or clearly labeled demo screenshots; do not present illustrative
usage as a live account reading. The documentation index lists every page in
[English](docs/README.md), [Korean](docs/i18n/ko/docs/README.md), and
[Simplified Chinese](docs/i18n/zh-CN/docs/README.md).

## Submit a pull request

Describe what changed and why, list relevant validation, and state any remaining
platform or visual verification gaps. Add regression tests when changing parser,
scope, persistence, or identity behavior. Documentation-only changes need link,
translation, and rendering checks rather than application rebuilds.

A passed build does not establish desktop behavior. For window/layer changes,
follow the [test plan](docs/04-quality/TEST_PLAN.md) and distinguish automated
checks from actual compositor/pixel verification. Never claim a live provider
test from a synthetic fixture alone.

## Release maintenance

Versioned archives are published through the existing release workflow only
after both platform jobs pass. Keep the package version, Tauri config, lockfile,
changelog, and release notes aligned. Documentation and design updates alone do
not require a new binary release. The full procedure is in the
[runbook](docs/05-ops/RUNBOOK.md#release-procedure).
