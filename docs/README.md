<!-- qhud:languages -->
<p align="center">
  <a href="i18n/ko/docs/README.md">한국어</a> · <strong>English</strong> · <a href="i18n/zh-CN/docs/README.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# Documentation


qhud's complete documentation is available in all three languages, from first
installation to architecture, decisions, and release history.

## Start using qhud

| Document | What it covers |
| --- | --- |
| [Project introduction](../README.en.md) | Features, screenshots, platform support, and downloads. |
| [User guide](GUIDE.md) | Usage/reset semantics, commands, multiple accounts, configuration, and troubleshooting. |
| [Windows setup](05-ops/WINDOWS.md) | WebView2 requirements, portable installation, and native MSVC builds. |
| [Operations runbook](05-ops/RUNBOOK.md) | Linux setup, GNOME integration, diagnostics, and release procedure. |
| [Contributing](../CONTRIBUTING.md) | Bug reports, development checks, translations, and pull requests. |
| [Changelog](../CHANGELOG.md) | Version-by-version changes and dated verification notes. |

## Understand the implementation

| Document | What it covers |
| --- | --- |
| [Project dashboard](00-overview/DASHBOARD.md) | Current release, verification evidence, work board, and resume point. |
| [Project brief](00-overview/PROJECT_BRIEF.md) | Purpose, supported scope, requirements, and non-goals. |
| [Specification](03-spec/SPEC.md) | Requirements, CLI flags, environment variables, and payload contract. |
| [Architecture](03-spec/ARCHITECTURE.md) | Data flow, modules, identity, provider fetch paths, and platform boundaries. |
| [Decision log](02-decisions/DECISION_LOG.md) | Design decisions and the evidence behind them. |
| [Test plan](04-quality/TEST_PLAN.md) | Automated checks and dated desktop verification. |
| [Risk register](04-quality/RISK_REGISTER.md) | Known risks, mitigations, and observed failures. |
| [Assumptions](01-discovery/ASSUMPTIONS.md) | Current premises and their verification status. |

## Explore the project history

These documents preserve dated research and decisions. An early proposal or
old verification gap is a historical statement, not necessarily the current
product behavior. Start with the specification and dashboard for the current state.

| Document | Record |
| --- | --- |
| [Office hours](01-discovery/OFFICE_HOURS.md) | Initial problem framing and discussion. |
| [Research notes](01-discovery/RESEARCH_NOTES.md) | Early technical references and observations. |
| [Feasibility report](01-discovery/FEASIBILITY_REPORT.md) | Initial implementation options and feasibility. |
| [Alternatives](02-decisions/ALTERNATIVES.md) | Approaches considered and tradeoffs. |
| [Cross-validation log](02-decisions/CROSS_VALIDATION_LOG.md) | External review and adopted changes. |
| [Retrospective](05-ops/RETRO.md) | Lessons recorded through development. |
| [v0.6.0 release notes](05-ops/releases/v0.6.0.md) | Native Windows support and quota/reset fixes. |
| [v0.6.1 release notes](05-ops/releases/v0.6.1.md) | Documentation rewrite and Ubuntu verification. |
| [v0.6.2 release notes](05-ops/releases/v0.6.2.md) | Automatic Codex email display from local login data. |

## Languages and source fidelity

The repository opens in Korean through `README.md`; English is available in
`README.en.md`, and `README.ko.md` remains a matching Korean copy for existing
links. Other English pages keep their original paths. Korean and Chinese mirrors
preserve those paths inside their language directories. Every page links to its
counterparts; translated headings retain anchors for source-document links.

Translations include the entire document, not only an overview. Command blocks,
API/schema identifiers, requirement/decision IDs, dates, and verification values
are preserved. The [MIT license](../LICENSE) remains in its original English text.

For a correction, open an [issue](https://github.com/chquandogong/qhud/issues/new/choose)
or follow the [contribution guide](../CONTRIBUTING.md). All three languages are welcome.
