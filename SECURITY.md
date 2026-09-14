<!-- qhud:languages -->
<p align="center">
  <a href="docs/i18n/ko/SECURITY.md">한국어</a> · <strong>English</strong> · <a href="docs/i18n/zh-CN/SECURITY.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# Security policy

## Supported versions

Security fixes are developed on `main` and released in the newest published
version. Please reproduce a suspected vulnerability with the latest release or
the current `main` branch when practical. Earlier releases are unsupported
unless a maintainer announces otherwise.

## Report a vulnerability privately

Do not open a public issue, discussion, or pull request for a suspected
vulnerability. Use GitHub's
[private vulnerability reporting form](https://github.com/chquandogong/qhud/security/advisories/new).
If that form is unavailable, do not publish the details; wait until the private
reporting channel is restored.

Include only the information needed to investigate:

- affected version or commit, operating system, and installation method;
- impact and the conditions required to trigger it;
- minimal reproduction steps or a proof of concept using synthetic data; and
- any mitigation or fix you have already tested.

Never submit a live credential, `auth.json`, `.credentials.json`, provider
configuration directory, unredacted diagnostic bundle, or another person's
data. Revoke any credential that was exposed before sending the report.

The maintainer aims to acknowledge a report within three business days and
provide an initial assessment within seven business days. These are best-effort
targets, not a service-level guarantee. Please allow time for a fix and a
coordinated advisory before public disclosure.

## Scope and safe research

Security-sensitive areas include local credential discovery, account and
snapshot ownership, provider requests, rendered WebView content, filesystem
paths, and the build and release pipeline. A vulnerability in an upstream AI
provider should be reported to that provider; report it here only when qhud
introduces or amplifies the exposure.

Use accounts and machines you own, synthetic or revoked credentials, and the
smallest test needed to demonstrate impact. Do not access other people's data,
degrade a provider service, persist on a machine, or use social engineering.
This project does not currently offer a bug bounty.

For ordinary bugs and support requests, follow [SUPPORT.md](SUPPORT.md).
