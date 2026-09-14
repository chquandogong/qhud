<!-- qhud:languages -->
<p align="center">
  <a href="docs/i18n/ko/SUPPORT.md">한국어</a> · <strong>English</strong> · <a href="docs/i18n/zh-CN/SUPPORT.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# Support

qhud is a community-maintained project. Help is provided on a best-effort basis
in English, 한국어, and 简体中文.

## Before asking

Check the [user guide](docs/GUIDE.md), the platform instructions in the
[operations runbook](docs/05-ops/RUNBOOK.md), and
[existing issues](https://github.com/chquandogong/qhud/issues). Use the newest
release when possible; when reporting a version, enter the exact release (for
example, `vX.Y.Z`) or commit SHA.

## Choose the right channel

- Use the [bug report form](https://github.com/chquandogong/qhud/issues/new?template=bug_report.yml)
  for reproducible qhud failures.
- Use the [feature request form](https://github.com/chquandogong/qhud/issues/new?template=feature_request.yml)
  for a new behavior or use case.
- Use the [support question form](https://github.com/chquandogong/qhud/issues/new?template=support_question.yml)
  when setup, configuration, or documented behavior is unclear.
- Report suspected vulnerabilities privately as described in
  [SECURITY.md](SECURITY.md).
- Contact the relevant AI provider for provider account access, subscriptions,
  billing, service availability, or provider-side authentication failures.

Include the operating system and desktop, installation method, qhud version,
provider or component, what you tried, and the exact result. Maintainers may ask
you to confirm the problem on the newest release before investigating an older
one.

## Protect private data

Diagnostics and screenshots can contain email addresses, account or
organization IDs, local paths, session content, tokens, and credentials.
Inspect and redact them before posting. Never attach `auth.json`,
`.credentials.json`, a provider configuration directory, or a full unreviewed
diagnostic bundle. Use a small synthetic example whenever possible.
