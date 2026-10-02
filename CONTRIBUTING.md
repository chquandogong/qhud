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

On `main`, ordinary stderr breadcrumbs omit identifying values. Published
v0.7.1 binaries predate that hardening. Explicit JSON commands such as `--dump`
and `--codex-usage` still print full operator-requested payloads on either
revision; inspect and redact those outputs before sharing.

## Build and check

The repository declares Rust 1.90 or later; CI builds on stable Rust, and the
`Rust minimum version` job checks the declared version. The application
build uses plain HTML/CSS/JavaScript and needs neither Node.js nor npm. Node.js
22 is used only for the standalone frontend system-metrics regression test.
qhud pins qmonster to the revision in `src-tauri/Cargo.toml`.

Run this check on either development platform:

```sh
node --test tests/system-metrics.test.cjs
```

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

### Which documents are mirrored

- **Complete three-language mirrors.** `README.en.md` with its Korean
  `README.md` and Chinese `README.zh-CN.md`; `CHANGELOG.md`,
  `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, and `SUPPORT.md`; and
  every Markdown file under `docs/`, including dated research, decisions,
  retrospectives, and release notes. Each mirror is a complete translation, not
  a summary.
- **One canonical language, no mirror.** `AGENTS.md` and
  `.github/copilot-instructions.md` are English instructions for coding agents.
  The PR template and issue forms carry all three languages inline. `LICENSE`
  keeps its original English text. Source comments, commit messages, and
  workflow files are English.
- **Canonical record plus summaries.** No document uses this tier today. The
  dated internal records (office hours, research notes, feasibility report,
  alternatives, cross-validation log, and retrospective) are the only
  candidates. Moving one requires a decision-log entry and a change to the
  registry in `scripts/check-repository.mjs`; until then they remain complete
  mirrors.

The repository checker holds this registry and rejects a new root Markdown page
that it does not classify.

### Keep facts mechanically comparable

For a documentation change, update both translations in the same pull request.
Translate the prose, not the facts. For every mirror,
`node scripts/check-repository.mjs` compares the following with the English
source and fails on any difference:

- requirement, decision, risk, and assumption IDs (`FR-`, `NFR-`, `D-`, `CV-`,
  `R1`, and `A1` forms) outside code blocks;
- `YYYY-MM-DD` dates and the set of release versions;
- fenced code blocks, line by line after indentation is trimmed;
- the number of table rows and of headings.

Keep command syntax, API identifiers, and inline code unchanged, and keep every
table row. Historical records describe their own date; do not silently rewrite
an old observation as a current fact. Keep the MIT license text in its original
form.

For documentation-only changes, run `node scripts/check-repository.mjs` to
check mirrored file sets, translation facts, versions, release-note links, and
local Markdown links.
Keep the version named in downloads aligned with the latest published release;
describe later `main` behavior as unreleased until a new binary is tagged.

Check relative links and section anchors from each document's actual directory.
Use existing or clearly labeled demo screenshots; do not present illustrative
usage as a live account reading. The documentation index lists every page in
[English](docs/README.md), [Korean](docs/i18n/ko/docs/README.md), and
[Simplified Chinese](docs/i18n/zh-CN/docs/README.md).

### Pull-request translation signal

On a pull request, the `docs · release metadata` check also runs the checker
with `--base origin/<base branch>`. It fails when the pull request changes an
English source but not both of its mirrors. Run the same comparison before
pushing:

```sh
node scripts/check-repository.mjs --base origin/main
```

When an English-only change is correct, such as a typo, broken-link, or
formatting fix that the translations do not share, acknowledge it with a trailer
in any commit message of the pull request. Name the path and a reason:

```text
Translation-Exempt: docs/GUIDE.md fix an English misspelling
```

The check then passes and reports the acknowledgement as a notice. Do not use the
trailer for a change of meaning, an added or removed fact, or a version update;
those need both translations. A translation-only fix needs no trailer.

### Reviewer checklist

Before approving a documentation change, check each language version for:

1. **Meaning.** The mirror makes the same claims, conditions, limits, and
   caveats as the English source, without additions, omissions, or softening. A
   dated observation stays dated in every language.
2. **Facts the checker cannot see.** Numbers and units in prose, product,
   command, and setting names, and quoted UI labels match the source.
3. **Links.** Relative links and anchors resolve from the translated file's own
   directory. Translated headings keep the English anchor through
   `<!-- qhud:anchor -->` and `<a id>`, and release-note language links stay
   absolute and tag-pinned.
4. **Privacy.** No version adds an email address, account or organization ID,
   token, cookie, session text, local path, or unredacted diagnostic, and
   screenshots are demo or redacted in every language.
5. **Acknowledgements.** Each `Translation-Exempt` trailer names the right path
   and describes a genuinely English-only change.

The [repository operations guide](docs/05-ops/REPOSITORY.md#translation-maintenance)
records how CI enforces this and what the first trial cost.

## Submit a pull request

Describe what changed and why, list relevant validation, and state any remaining
platform or visual verification gaps. Add regression tests when changing parser,
scope, persistence, or identity behavior. Documentation-only changes need link,
translation, and rendering checks rather than application rebuilds.

Open a focused branch and PR against `main`; do not push a release tag as a
substitute for review. The protected branch requires the Linux and Windows
build/test jobs, repository-integrity and dependency-review jobs, and Rust,
JavaScript/TypeScript, and Actions CodeQL analyses. Resolve review conversations
and update the branch if checks become stale. Solo maintenance uses zero required
approvals; a team repository should require an independent reviewer. See the
[repository operations guide](docs/05-ops/REPOSITORY.md) for those settings and
their private/public variants.

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
