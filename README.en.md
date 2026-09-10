<!-- qhud:languages -->
<p align="center">
  <a href="README.md">한국어</a> · <strong>English</strong> · <a href="README.zh-CN.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<p align="center">
  <img src="docs/assets/qhud-banner.svg" alt="qhud — AI usage, resets, and sessions at a glance" width="100%">
</p>

<p align="center">
  A quiet desktop companion for <strong>Claude Code, Codex, and Antigravity</strong>.<br>
  See account usage, reset times, and supported terminal sessions in one place.
</p>

<p align="center">
  <a href="https://github.com/chquandogong/qhud/releases/latest"><img src="https://img.shields.io/github/v/release/chquandogong/qhud?style=flat-square&color=28a99e" alt="Latest release"></a>
  <a href="https://github.com/chquandogong/qhud/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/chquandogong/qhud/ci.yml?branch=main&style=flat-square&label=build" alt="Ubuntu and Windows build status"></a>
  <img src="https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-52677d?style=flat-square" alt="Linux and Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-52677d?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="#install">Download</a> · <a href="docs/GUIDE.md">User guide</a> · <a href="docs/README.md">Documentation</a> · <a href="CONTRIBUTING.md">Contribute</a>
</p>

---

## One view for your AI work

qhud brings account quotas and session signals out of separate command-line
interfaces and into a small desktop widget. It uses the observation pipeline
from [qmonster](https://github.com/chquandogong/qmonster), with a Rust/Tauri
backend and a lightweight webview. No Node.js or npm is required to build it.

| See | What you get |
| --- | --- |
| **Usage and resets** | Account windows, model-specific pools when available, reset countdowns, and exact reset timestamps. |
| **Account context** | Separate account/workspace rows, configurable display names, and dated snapshots after restart. |
| **Session activity** | On supported Linux setups: status, context pressure, model, effort, branch, working directory, and conflict indicators. |
| **Freshness** | Local observation every 2 seconds. Provider usage requests run when you explicitly refresh. |

## A compact view, with room for detail

<table>
  <tr>
    <td align="center"><img src="docs/assets/widget-compact.png" width="330" alt="Linux demo with a compact provider strip and session tiles"><br><sub>Compact session overview</sub></td>
    <td align="center"><img src="docs/assets/widget-expanded.png" width="330" alt="Linux demo with an expanded session and conflict details"><br><sub>Details when you need them</sub></td>
  </tr>
</table>

*These screenshots show Linux demo fixtures with illustrative values. The
current account layout and available model windows depend on the provider,
login, and platform.*

## Platform support

| Capability | Linux x86_64 | Windows x64 |
| --- | --- | --- |
| Native desktop widget | GTK / WebKitGTK | WebView2; no WSL required |
| Account usage and reset times | Supported | Supported |
| Multiple signed-in config directories | Supported | Supported |
| Terminal-pane observation | tmux / herdr | Native Windows terminal tabs are not supported |
| Desktop integration | Reference: Ubuntu 24.04, GNOME/Wayland through XWayland | Native window and tray; Linux desktop-layer behavior is not guaranteed |
| Distribution | Portable `.tar.gz` | Portable `.zip` |

Other Linux desktops are best effort. No macOS package is published.
Without a supported multiplexer, qhud shows real local accounts and zero panes.
Example data is available only with `--demo`.

## Install

**Current release: [v0.6.2](https://github.com/chquandogong/qhud/releases/tag/v0.6.2).**
Both platform build/test jobs gate publication. Each archive has a SHA-256 file.

| Download | Verify |
| --- | --- |
| [Windows x64 ZIP](https://github.com/chquandogong/qhud/releases/download/v0.6.2/qhud-v0.6.2-windows-x86_64.zip) | [SHA-256](https://github.com/chquandogong/qhud/releases/download/v0.6.2/qhud-v0.6.2-windows-x86_64.zip.sha256) |
| [Linux x86_64 tarball](https://github.com/chquandogong/qhud/releases/download/v0.6.2/qhud-v0.6.2-linux-x86_64.tar.gz) | [SHA-256](https://github.com/chquandogong/qhud/releases/download/v0.6.2/qhud-v0.6.2-linux-x86_64.tar.gz.sha256) |

### Windows

Extract the ZIP and run `qhud.exe` inside its versioned directory. Microsoft
Edge WebView2 Runtime is required. The portable package does not register
shortcuts or automatic startup. See the [Windows guide](docs/05-ops/WINDOWS.md)
for prerequisites, source builds, and configuration.

### Linux

After downloading the archive and its SHA-256 file into the same directory:

```sh
sha256sum -c qhud-v0.6.2-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.6.2-linux-x86_64.tar.gz
install -Dm755 qhud-v0.6.2-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

Runtime dependencies on Ubuntu 24.04 include `libwebkit2gtk-4.1-0`,
`libgtk-3-0`, and `libayatana-appindicator3-1` for the tray.
[Source builds and GNOME autostart →](docs/05-ops/RUNBOOK.md)

## Everyday use

- **Move and resize:** drag the header or footer; use the corner grip to resize.
- **Inspect:** select an account row or session tile; hover a gauge for its exact reset time.
- **Refresh:** use the top-bar ⟳ for all providers or the ⟳ next to an account.
- **Peek:** use the tray's *Pin above windows* action.
- **Quit:** choose *Quit qhud* from the tray.

For an **already running** widget, the equivalent shortcuts are:

```sh
qhud --refresh-all
qhud --peek
```

[Commands, account setup, and troubleshooting →](docs/GUIDE.md)

## Understand the numbers

A percentage belongs to an **account and a time window**. Model-specific
limits, including Spark 5H/7D, appear only when the provider returns them for
that login. Missing limits are never filled with invented zeros, and an
account's saved usage is not reused under another account's name.

Saved readings carry their age. A remaining reset time is a countdown;
hovering a gauge reveals the exact date and time. Local observation does not
make provider API requests. An explicit refresh may read existing access
tokens for the provider's usage endpoint; qhud does not run OAuth refresh
grants itself. Codex can delegate its fallback fetch to its own CLI, and
Antigravity uses the running CLI's loopback service.

Account settings stay outside the repository. See the
[configuration and data guide](docs/GUIDE.md#configuration-and-local-data).

## Explore the documentation

Every document, including design decisions and historical records, is available
in English, Korean, and Simplified Chinese.

| Start here | Go deeper |
| --- | --- |
| [User guide](docs/GUIDE.md) | [Architecture](docs/03-spec/ARCHITECTURE.md) |
| [Windows setup](docs/05-ops/WINDOWS.md) · [Linux operations](docs/05-ops/RUNBOOK.md) | [Specification](docs/03-spec/SPEC.md) · [Decision log](docs/02-decisions/DECISION_LOG.md) |
| [Contributing](CONTRIBUTING.md) | [Test plan](docs/04-quality/TEST_PLAN.md) · [Risk register](docs/04-quality/RISK_REGISTER.md) |
| [Changelog](CHANGELOG.md) | [Complete documentation index](docs/README.md) |

## Contribute

Bug reports, documentation corrections, and translation improvements are
welcome in **English, 한국어, or 简体中文**. Start with the
[contribution guide](CONTRIBUTING.md) or [open an issue](https://github.com/chquandogong/qhud/issues/new/choose).

qhud is an independent project and is not affiliated with Anthropic, OpenAI,
or Google. Provider names identify compatible tools.

[MIT License](LICENSE) · Built on [qmonster](https://github.com/chquandogong/qmonster)
