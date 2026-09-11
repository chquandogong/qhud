<!-- qhud:languages -->
<p align="center">
  <a href="i18n/ko/docs/GUIDE.md">한국어</a> · <strong>English</strong> · <a href="i18n/zh-CN/docs/GUIDE.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# User guide

A practical guide to installing qhud, connecting existing CLI accounts, and
reading usage without confusing a saved snapshot with a current response.

## Install and start

Choose the Windows ZIP or Linux tarball from
[Releases](https://github.com/chquandogong/qhud/releases/latest). Each archive
contains a versioned directory and comes with a separate SHA-256 file.

| Platform | Start | Requirements |
| --- | --- | --- |
| Windows x64 | Extract the ZIP; run `qhud.exe` inside the extracted directory. | Microsoft Edge WebView2 Runtime. See [Windows setup](05-ops/WINDOWS.md). |
| Linux x86_64 | Extract the tarball; install the binary inside its versioned directory. | GTK/WebKitGTK; Ubuntu 24.04 is the reference system. See [Linux operations](05-ops/RUNBOOK.md). |

For Linux v0.7.0, after downloading both files:

```sh
sha256sum -c qhud-v0.7.0-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.7.0-linux-x86_64.tar.gz
install -Dm755 qhud-v0.7.0-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

For a Windows download, compare the displayed digest with the `.sha256` file:

```powershell
Get-FileHash -Algorithm SHA256 .\qhud-v0.7.0-windows-x86_64.zip
Get-Content .\qhud-v0.7.0-windows-x86_64.zip.sha256
```

Sign in using each provider's own CLI. qhud reads existing account information;
it does not replace provider login flows. Start qhud, then press ⟳ to fetch usage.
The portable packages do not register shortcuts or automatic startup.

## Read the widget

**Provider → account → time window** is the main hierarchy. Each percentage
belongs to that account and window. An active Codex workspace merges into the
account row; other authenticated workspaces can appear as additional rows.

- **5H / 7D** describe the window duration, not the time left until reset.
- **The time at the right** is the remaining time until reset. Hover for the exact local date and time.
- **Model names** distinguish separate quota pools. They appear only when the provider supplies them.
- **A snapshot age** identifies stored usage. Restarting qhud does not turn an old reading into a fresh response.
- **Zero panes** means no supported terminal source is available. Account usage can still be shown.

A missing model quota is not the same as zero usage. For example, Spark 5H/7D
is displayed only if the current Codex account's response contains those limits.
Refreshing another account does not establish the limits for the active account.

On supported Linux setups, session tiles add status, context pressure, model,
effort, branch, working directory, and conflict indicators. Windows Terminal,
PowerShell, and WezTerm tabs are not observed natively by this release.

## System usage and About

The small bottom strip samples every 2 seconds and keeps up to 30 points from
roughly the last 60 seconds. Select CPU, MEM, GPU, DISK or NET to open its details.
Select the same metric again to close the panel, or another metric to switch it.
CPU, memory and GPU use a fixed 0–100% scale; disk and network graphs show
read + write or receive + send throughput with a scale that follows recent traffic.
Hover a metric for its current values. Keyboard users can Tab to a metric and
press Enter or Space.

| Metric | What it measures |
| --- | --- |
| CPU | Total CPU utilization across logical processors. |
| MEM | Used physical memory / total physical memory; the details show byte capacities. |
| GPU | One supported adapter, chosen by the busiest valid initial sample and retained for that sampling session. Windows reports the busiest engine after combining process instances of that engine. |
| DISK | Aggregate physical-disk read/write speed in bytes per second, excluding duplicate partition/stacked-device I/O. Details show used/total capacity of mounted local disk volumes, deduplicated by volume; capacity refreshes every 30 seconds. |
| NET | Aggregate receive/send speed on non-loopback interfaces with usable IP addresses. Linux also filters interfaces whose operational state is down. VPN/virtual and physical interfaces can count the same traffic more than once; this is not an ISP bandwidth meter. |

GPU measurement is conditional on the device and driver: Windows uses WDDM GPU
Engine counters; Linux supports AMD's available busy-percent sysfs value, Intel's
i915/xe idle-residency counters read as their complement, and NVIDIA through an
installed NVML library. Linux can show VRAM usage when the selected
provider supplies it; Windows does not currently report VRAM. Unsupported GPUs are
hidden. If a previously supported reading fails, the column remains with a gap.
The first CPU/rate sample, counter resets and unavailable readings show `--` or a
gap, never a fabricated zero. A valid idle sample is zero.

System sampling pauses when the window is hidden or minimized. Showing it again
starts a new history and warms up rate counters. A long sampling gap also resets
the baseline. History stays in memory; system sampling does not scan processes,
launch monitoring commands, read account credentials or send telemetry. `--demo`
uses clearly identified synthetic system values.

Click **qhud** at the top left to open About. It shows the version embedded in the
running build, maker **Chenghao Quan (chquandogong)** and the
[maker's homepage](https://chquandogong.github.io/CHENGHAO-QUAN/).
Close with ×, Escape or a click outside the panel. For a local system-only JSON
diagnostic, run `qhud --system-dump`; it waits for two samples and exits without
loading account information or opening the widget.

## Controls and commands

Drag the header/footer to move the widget, use the corner grip to resize, and
Ctrl+wheel to change zoom. Select an account or tile to expand its details.
The tray provides *Pin above windows* and *Quit qhud*.

These commands target an **already running** instance:

| Command | Action |
| --- | --- |
| `qhud --peek` | Toggle pinned-above and desktop-layer mode. |
| `qhud --refresh-all` | Request a refresh for all supported providers. |
| `qhud --refresh-claude` | Request Claude usage refresh. |
| `qhud --fetch-codex` | Request Codex workspace usage refresh. |

On Windows, use the executable path, for example `& '.\qhud.exe' --refresh-all`.
The relay flags do not perform their action on the first launch; start the
widget before using them.

These diagnostics run independently:

```sh
qhud --dump
qhud --system-dump
qhud --claude-usage
qhud --codex-usage
qhud --agy-usage
qhud --codex-appserver
```

`--dump` prints one local observation payload. The three `--*-usage` commands
fetch, print JSON, and save the result. `--codex-appserver` prints the CLI
fallback response without saving it. Use `qhud --demo` for example data after
closing the existing instance. Diagnostics may contain account identities,
local paths, and session details; redact them before sharing an issue.

## Configuration and local data

qhud keeps its configuration outside the source repository:

| Item | Location or behavior |
| --- | --- |
| Default Linux configuration | `~/.config/qhud/` |
| New Windows installation | `%APPDATA%\qhud\` |
| Display settings and registered accounts | `accounts.json` inside the configuration directory |
| Last explicitly fetched usage | `fetched-usage.json` inside the configuration directory |
| Default Codex home | `~/.codex`, or `CODEX_HOME` |
| Default Claude config directory | `~/.claude`, or `CLAUDE_CONFIG_DIR` |

Configuration directory precedence is: `QHUD_CONFIG_DIR` as given;
`XDG_CONFIG_HOME/qhud`; on Windows, an existing `%USERPROFILE%\.config\qhud`
then `%APPDATA%\qhud`; finally `~/.config/qhud`. Windows prefers `USERPROFILE`
for the home directory, while Linux prefers `HOME`.

`accounts.json` is ordinary JSON: no comments or trailing commas. The following
example uses placeholder account IDs; replace them with the identity reported
by your CLI or qhud diagnostics. Merge settings into your existing file rather
than discarding other entries.

```json
{
  "labels": {
    "codex:YOUR_ACCOUNT_ID": "work@example.com"
  },
  "workspace_names": {
    "YOUR_ACCOUNT_ID": "Work"
  },
  "workspace_plans": {
    "YOUR_ACCOUNT_ID": "My workspace plan"
  },
  "claude_config_dirs": ["~/claude-personal"],
  "codex_homes": ["~/.codex-personal"]
}
```

Codex automatically shows the email available in the local `auth.json` ID token.
This is a best-effort display hint: missing or malformed email data falls back
to the account/workspace ID. No separate label file is needed for ordinary email
display on a new computer. A matching `labels` entry still takes precedence over
the email, and neither changes the account/workspace identity used for usage.
Keep plan labels operator-supplied rather than guessing them from wire enums.

## More than one account

Keep each additional account signed in under its own provider directory, then
register that directory using `claude_config_dirs` or `codex_homes`.

```sh
CLAUDE_CONFIG_DIR=~/claude-personal claude
CODEX_HOME=~/.codex-personal codex login
```

For Windows, set an environment variable for the CLI session:

```powershell
$previousCodexHome = $env:CODEX_HOME
try {
  $env:CODEX_HOME = "$env:USERPROFILE\.codex-personal"
  codex login
} finally {
  $env:CODEX_HOME = $previousCodexHome
}
```

That example restores the shell's previous `CODEX_HOME` after login. Registered directory
strings expand a leading `~/` or bare `~`; `%ENV%`, `$ENV`, and `~\` are not
expanded inside the JSON file. Absolute paths also work.

Claude rows distinguish **account and organization**. Two directories signed
into the same account and organization do not produce separate quota pools.
Antigravity multiple-account support is not implemented. A `known` account
list is maintained by the operator, not automatically learned by qhud. Forgetting
a placeholder does not hide a live account.

## Network and freshness

| Provider | Passive observation | Explicit refresh |
| --- | --- | --- |
| Claude | Local identity, statusline, and usage cache where available. | Existing OAuth access token sent to the provider usage endpoint. |
| Codex | Local identity and supported pane statusline data. | Usage/account endpoints per saved credential; active-login fallback via `codex app-server`. |
| Antigravity | Local account and pane information where available. | Loopback RPC served by a running agy process. |

Local observation runs every 2 seconds and does not make provider API requests.
It can read identity fields from files that also contain credentials. Explicit
refresh may read an existing access token; qhud itself does not run OAuth refresh
grants. The provider CLI manages authentication and any delegated token rotation.

Fetched results are saved with their age. Account matching prevents a previous
login's snapshot from appearing under a new login. Concurrent refreshes preserve
each other's stored results. qhud observes the qmonster pipeline without writing
to qmonster's database.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Account missing | Confirm the provider CLI is signed in and qhud is reading the intended config/home directory. |
| Email replaced by an ID | Confirm qhud is v0.6.2 or newer and reads the intended Codex home. If its local ID token has no usable email, add a matching `labels` entry for a display override. |
| Spark or another model missing | Refresh and inspect the current account's response. An absent model is not synthesized. |
| Old reading or 401 error | Read the row's age/error and sign in again through that provider's CLI, in the affected directory. |
| No terminal panes | Check tmux/herdr on Linux. Native Windows terminal tabs are outside this release's support. |
| Widget not visible | Check the tray, use `--peek` on the running instance, and ensure the desktop is unlocked. |
| Rows extend beyond the window | Scroll the quota area or increase the window size. |
| Linux pixels freeze after display sleep | Consult the frame-guard diagnostics in the [runbook](05-ops/RUNBOOK.md#troubleshooting). |

Before reporting a bug, record the qhud version, OS, provider, how it was
installed, steps to reproduce, and a redacted error. See
[Contributing](../CONTRIBUTING.md) and the [test plan](04-quality/TEST_PLAN.md)
for the distinction between automated checks and actual desktop verification.
