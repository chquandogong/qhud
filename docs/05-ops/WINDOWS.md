<!-- qhud:languages -->
<p align="center">
  <a href="../i18n/ko/docs/05-ops/WINDOWS.md">한국어</a> · <strong>English</strong> · <a href="../i18n/zh-CN/docs/05-ops/WINDOWS.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# Windows native build and run

qhud provides a Windows x64 WebView2 widget starting with v0.6.0. It displays
account usage without WSL. The frontend uses the repository's `ui/` files, so
Node.js and npm are not required.

## Install a release

Download `qhud-v0.6.1-windows-x86_64.zip` and its checksum file from
[GitHub Releases](https://github.com/chquandogong/qhud/releases). Extract the
archive and run `qhud.exe` inside its versioned directory. Microsoft Edge
WebView2 Runtime is required.

The ZIP is portable. It does not automatically register an installer, login
startup, or shortcuts. To update, quit using **Quit qhud** in the tray before
replacing the executable. Settings and provider login files remain separate.

## Build from source

- Git for Windows, with `git.exe` on PATH.
- Rust 1.88 or later and the `x86_64-pc-windows-msvc` toolchain.
- Visual Studio Build Tools with **Desktop development with C++** and the Windows SDK.
- Microsoft Edge WebView2 Runtime.

In Developer PowerShell for Visual Studio, change to the repository and run:

```powershell
.\scripts\Build-Windows.ps1 -Test
```

Outside the developer shell, specify the C++ tools installation:

```powershell
.\scripts\Build-Windows.ps1 -Test -VisualStudioPath 'C:\BuildTools'
```

The output is `target\x86_64-pc-windows-msvc\release\qhud.exe`. `-Run` starts it
after building; `-DebugBuild` selects the debug profile; `-PrepareOnly` prepares
the tools and dependency without compiling. `-Test` runs tests with the same
build settings first. Do not combine `-PrepareOnly` with `-Run` or `-Test`.

The script prefers tools on PATH and an existing Visual Studio developer
environment. It also checks `tools/rust/cargo/bin`, `tools/cargo/bin`, and
`tools/vs-buildtools` beside the repository. It does not install tools or modify
the global PATH.

The Windows build prepares a pinned revision in the sibling
`qhud-deps/qmonster` directory and applies `patches/qmonster-windows.patch`.
It does not overwrite another modified checkout. The patch applies only through
temporary configuration for the corresponding Cargo command; a temporarily
modified lockfile is restored to its original bytes on success or failure.
Do not run another Cargo command concurrently in the same checkout. Normal
Linux `cargo build --release --locked` uses the pinned Git dependency and does
not need this preparation step.

## Run and interact

```powershell
Start-Process -FilePath '.\target\x86_64-pc-windows-msvc\release\qhud.exe'
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --peek
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --refresh-all
```

Drag the header or footer to move the window and use the corner to resize it.
Choose **Quit qhud** in the tray to exit. Once running, subsequent `--peek` and
`--refresh-all` requests are relayed to that instance. Usage is fetched through
⟳ or an explicit refresh command; the local observation interval is not a
network refresh interval.

## Account and model usage

A new Windows installation stores settings in `%APPDATA%\qhud`. `accounts.json`
holds display names and registered account paths; `fetched-usage.json` holds
usage with the fetch time. Personal settings and fetched results are not
included in the Git repository or release.

`QHUD_CONFIG_DIR` overrides the configuration location. `XDG_CONFIG_HOME`, or an
existing `%USERPROFILE%\.config\qhud`, is respected. Provider login information
is read from `%USERPROFILE%\.codex` and `%USERPROFILE%\.claude` by default, with
`CODEX_HOME` and `CLAUDE_CONFIG_DIR` overrides. If Codex exposes only a workspace
ID locally, `labels` in `accounts.json` can provide an email display. See the
[runbook](RUNBOOK.md) for the schema.

Model limits such as GPT-5.3-Codex-Spark 5H/7D are displayed when returned by the
server. Long names wrap and the usage area scrolls when there are many entries.
The right-hand time is the remaining time until reset; hover over a gauge for
the exact reset date and time. A missing model limit is not assumed to be 0%,
and another account's reading is not substituted.

## Support and verification

Linux GTK frame recovery, X11 window handling, and tmux/herdr observation remain
in place. Native observation of Windows Terminal, PowerShell, or WezTerm tabs
is not implemented. Without an observable multiplexer, qhud displays real local
accounts and `panes = 0`; sample data requires `--demo`. Linux `/proc` process
metrics may be absent on Windows.

CI runs tests and release builds on Ubuntu 24.04 and Windows; both must succeed
before publication. Unit tests cover preservation of Spark 5H/7D percentages and
resets, restoration of model-only snapshots, and concurrent saved results.
Automated builds and tests do not establish actual behavior on every desktop
or provider account.
