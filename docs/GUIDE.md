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

For Linux v0.8.2, after downloading both files:

```sh
sha256sum -c qhud-v0.8.2-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.8.2-linux-x86_64.tar.gz
install -Dm755 qhud-v0.8.2-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

For a Windows download, compare the displayed digest with the `.sha256` file:

```powershell
Get-FileHash -Algorithm SHA256 .\qhud-v0.8.2-windows-x86_64.zip
Get-Content .\qhud-v0.8.2-windows-x86_64.zip.sha256
```

Archives published by the current release workflow also carry GitHub build
provenance. Install GitHub CLI, substitute the exact downloaded filename, and
verify the archive itself rather than the `.sha256` sidecar:

```sh
gh attestation verify ./qhud-vX.Y.Z-linux-x86_64.tar.gz --repo chquandogong/qhud
```

```powershell
gh attestation verify .\qhud-vX.Y.Z-windows-x86_64.zip --repo chquandogong/qhud
```

The checksum detects transfer damage or replacement relative to its sidecar;
the attestation verifies that GitHub Actions built the archive for this
repository. A release created before attestations were enabled will not verify.

Releases after v0.7.2 also carry a CycloneDX 1.5 JSON software bill of
materials, `qhud-vX.Y.Z-sbom.cdx.json`, with its own `.sha256` sidecar and
build-provenance attestation. The release workflow generates it from the
tagged source and its locked `Cargo.lock` for every target platform, so it
lists the Rust crates of both the Linux and Windows builds; the frontend is
qhud's own HTML, CSS, and JavaScript and has no third-party packages. Local
build paths are removed before publication. Verify the SBOM itself (on
Windows, compare `Get-FileHash` with the sidecar as above):

```sh
sha256sum -c qhud-vX.Y.Z-sbom.cdx.json.sha256
gh attestation verify ./qhud-vX.Y.Z-sbom.cdx.json --repo chquandogong/qhud
```

For Codex and Claude Code, sign in through the provider CLI. For Antigravity on Windows, open the signed-in native app or keep the `agy` CLI running. Start qhud and press the provider’s ⟳ to fetch usage. The portable packages do not register shortcuts or automatic startup.

## Read the widget

Each provider groups rows by **email address**.
Under an address, **Personal** comes before **Team**. Each workspace or
organization keeps its own plan, usage windows and reset times. Configured
profiles awaiting a connection appear under the same address.

**Monthly price chip**

The amount beside a plan is its US monthly list price, per seat for teams. The higher-contrast fill measures $200 per unit. A $100 plan fills half a unit, $500 fills 2.5 units, and a confirmed Free plan shows $0 with an empty fill. The 5H and 7D bars measure the share of each usage limit consumed.

When Codex Business or Claude Team is identified and the seat type is unknown, the chip shows **$20–125** per seat per month. This covers Standard and Premium seats on annual and monthly billing. A verified Codex Standard seat narrows the range to **$20–25**, and a verified Premium seat to **$100–125**. Claude’s **Max 5x** member usage grade can appear alongside the team range. Open the row to check the basis. Prices were reviewed on **2026-11-24**. The invoice gives the actual charge.
A separate connection-status row without a recognized plan can omit the price chip. Plans with custom pricing have no public dollar amount.
[ChatGPT Pro prices](https://help.openai.com/en/articles/9793128-about-chatgpt-pro-tiers) ·
[ChatGPT Business seats](https://help.openai.com/en/articles/8792828-chatgpt-business-overview) ·
[Claude Team seats](https://support.claude.com/en/articles/9266767-what-is-the-team-plan)

For personal Claude Pro and a Google AI Pro plan reported by Antigravity,
the chip shows approximately $17–20 per month. The range covers the annual
price divided by twelve and the monthly rate. [Claude pricing](https://claude.com/pricing) ·
[Google AI Pro pricing](https://one.google.com/about/google-ai-plans/?gl=US)

- **5H / 7D** describe the window duration, not the time left until reset.
- **The time at the right** is the remaining time until reset. Hover for the exact local date and time.
- **Model names** distinguish separate quota pools. They appear only when the provider supplies them.
- **A snapshot age** identifies stored usage. Restarting qhud does not turn an old reading into a fresh response.
- **Zero panes** means no supported terminal source is available. Account usage can still be shown.

A missing model quota is not the same as zero usage. For example, Spark 5H/7D
is displayed only if the current Codex account's response contains those limits.
Refreshing another account does not establish the limits for the active account.

The **Reset credits: N** badge shows
only the count supplied by the provider. Codex uses the official
`rateLimitResetCredits.availableCount`: an explicit `0` is valid, while a missing
field stays unavailable. This is separate from extra-usage money and the reset
countdown; qhud displays the count but never consumes a reset credit. If Claude's
usage response supplies no manual-reset count, expanded details say that the
information is unavailable. Reset dates, quota percentages, and reset offers of
different types are not converted into an invented count. The account row's
snapshot age applies to the count as well.

Claude's legacy personal `claude_max` metadata with
`default_claude_max_20x` is displayed as **Ultra**, following the
[official Max 20x rename](https://support.claude.com/en/articles/17728952-what-is-the-ultra-plan).
The `default_claude_max_5x` tier displays as **Max 5x**. For Team/Enterprise,
the primary badge shows the member's explicit user usage grade: **Max 5x** or
**Ultra** for those two known tiers. The separate personal/team scope and
expanded **Organization plan: Team/Enterprise** preserve the actual organization
plan. An organization's tier never substitutes for an unknown member tier;
without a recognized user grade, the badge retains Team/Enterprise. Explicit
plan overrides still win. Exact `claude_ultra` or `ultra` metadata also displays
as Ultra; unknown organization values are not guessed. An absent 5H limit stays
absent, including when a fresh Ultra response no longer supplies that window.

Select **Supported models** next to a recognized plan or seat badge to expand
the provider guidance. This is an official-plan reference reviewed on
**2026-11-23**, not a live check of that account's model access or remaining
quota. Check the signed-in account and workspace in the provider's own picker:

- **Codex:** run `/model` in Codex CLI. Available models depend on the account,
  workspace permissions, and client version; a Team/Business plan name alone
  does not establish a member's seat type. See [official Codex models](https://developers.openai.com/codex/models).
- **Claude Code:** run `/model` in the terminal or Claude Desktop's Code tab.
  Max and Ultra plans and Team Premium seats include Fable usage within their
  limits; Team Standard seats require usage credits for Fable. A displayed
  **Max 5x** member usage grade does not independently verify a Premium seat.
  See [Claude Code model configuration](https://code.claude.com/docs/en/model-config)
  and [Fable eligibility and limits](https://support.claude.com/en/articles/15424964-claude-fable-models-on-your-plan).
- **Antigravity:** use its model picker. Paid Google AI Pro and trial
  subscriptions can have different model access; some models require a paid
  subscription or a qualifying trial. Work accounts follow their organization's
  access rules, including administrator approval for partner models. See
  [Antigravity plans and model conditions](https://antigravity.google/pricing).

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
qhud --provider-health
qhud --refresh-health
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
  "plans": {
    "codex:YOUR_ACCOUNT_ID": "Pro 200"
  },
  "workspace_names": {
    "YOUR_OTHER_WORKSPACE_ID": "Work"
  },
  "workspace_plans": {
    "YOUR_OTHER_WORKSPACE_ID": "Pro 500"
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

For Codex plan badges, qhud follows the current upstream status names for three
reviewed wire values: `prolite` becomes **Pro 100**, `pro` becomes **Pro 200**,
and `promax` becomes **Pro 500**. An explicit `plans` entry overrides the
account badge, and `workspace_plans` overrides the matching workspace badge.
Unknown wire values remain available in expanded details and tooltips but do not
become visible plan badges.

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

### Windows personal and team profiles

To keep two email addresses' personal and team scopes visible for both Codex
and Claude, register eight desired connections in `accounts.json` under
`profiles`. Registration creates a pending display entry; it does not create
credentials, switch an app's login, or invent an account ID. Use your own email
addresses and private directories in place of this synthetic example, and merge
the entries into the existing configuration:

```json
{
  "profiles": [
    { "key": "codex-first-personal", "provider": "codex", "label": "First personal", "email": "first@example.com", "scope": "personal", "directory": "~/qhud-accounts/codex-first" },
    { "key": "codex-first-team", "provider": "codex", "label": "First team", "email": "first@example.com", "scope": "team", "directory": "~/qhud-accounts/codex-first" },
    { "key": "codex-second-personal", "provider": "codex", "label": "Second personal", "email": "second@example.com", "scope": "personal", "directory": "~/qhud-accounts/codex-second" },
    { "key": "codex-second-team", "provider": "codex", "label": "Second team", "email": "second@example.com", "scope": "team", "directory": "~/qhud-accounts/codex-second" },
    { "key": "claude-first-personal", "provider": "claude", "label": "First personal", "email": "first@example.com", "scope": "personal", "directory": "~/qhud-accounts/claude-first-personal" },
    { "key": "claude-first-team", "provider": "claude", "label": "First team", "email": "first@example.com", "scope": "team", "directory": "~/qhud-accounts/claude-first-team" },
    { "key": "claude-second-personal", "provider": "claude", "label": "Second personal", "email": "second@example.com", "scope": "personal", "directory": "~/qhud-accounts/claude-second-personal" },
    { "key": "claude-second-team", "provider": "claude", "label": "Second team", "email": "second@example.com", "scope": "team", "directory": "~/qhud-accounts/claude-second-team" }
  ]
}
```

One Codex login can enumerate that email's accessible personal and team
workspaces, so its two profiles may share one isolated Codex home. Claude needs
separate authenticated profiles for the selected personal/team organizations.
During Claude login, select the requested organization, not just the right
email. If an exact organization must be pinned, add its verified
`expected_org_id` to that profile; do not guess it. Two emails sharing a Codex
team workspace are kept distinct by email plus workspace ID.

Open PowerShell in the extracted Windows package directory (the one containing `qhud.exe`) or the source repository root. The [connection script](../scripts/Connect-Account.ps1) lists registered profiles with `-List`. Use `-AllPending` to connect them, or `-Profile codex-first-personal` to select that example profile.

```powershell
.\scripts\Connect-Account.ps1 -List
.\scripts\Connect-Account.ps1 -AllPending
```

The script handles pending profiles and deduplicates provider/directory pairs,
so a shared Codex home is not logged in twice. Sign-in remains interactive in
the provider's own flow. It uses isolated profile directories and preserves the
default `~/.codex` and `~/.claude` logins rather than overwriting them. Refresh
qhud afterward. Unconnected profiles stay numberless as **Connect to qhud**;
expired credentials, a different email, and a different organization are shown
explicitly. A matching real account/workspace row replaces its pending entry.
Adding a profile does not grant access to a personal or team workspace that the
account does not own.

A Claude personal profile can record a Free plan
confirmed in the Claude web account menu. Set the profile's `observed_plan`
fields to its `email`, `plan` (`"free"`) and `verified_at_ms` (Unix time in
milliseconds). The personal row shows **Free** and reports that usage is
unavailable. A matching provider usage row takes precedence.

A Codex team profile can also carry a separately verified seat annotation from
the workspace's admin Members page. It appears as **Standard · Web verified**
or **Premium · Web verified**, including while the profile needs connection to
qhud. This is a dated manual observation, not an automatically refreshed API
value or proof that qhud is connected. The tooltip and expanded detail show the
source and verification time. On a usage row, both the email and workspace ID
must match; the annotation never replaces the provider's Business or other
plan, and qhud never infers a seat type from that plan.

## Network and freshness

### Windows provider connection status

Codex, Claude, and Antigravity remain visible even before a quota is available.
**Ready to fetch** means local prerequisites were found; **Fetch succeeded** means an explicit
refresh succeeded. Expand a provider's status to see its source, last attempt,
last successful snapshot, credential expiry when available, and recovery action.
Credential expiry describes the local login token, not your subscription expiry.
Old readings stay dated after a failed refresh; missing usage is never zero.

Codex uses the installed CLI's `app-server` for the active account and supports
the native Windows `Programs/OpenAI/Codex/bin` layout. Claude uses its Claude
Code login: a signed-in Desktop app does not renew an expired CLI credential.
Use `claude auth login` when prompted, then refresh qhud. qhud does not automate
the login or exchange refresh tokens itself.

Antigravity supports a running native Windows app's local daemon registration
as well as the existing `agy` listener. The native connection validates the
process and loopback port, then binds quota to the account returned by the app.
This local app interface may change between Antigravity versions. If no current
CLI identity matches a saved result, it is labelled **Last fetched account**;
it is not a claim about the app's current login. Start/open Antigravity and
refresh if its local service is unavailable.

The Antigravity plan badge uses the queried account's native `userTier.name`,
such as **Google AI Pro**, saved with that account's usage snapshot. An upgrade
prompt or quota shape is not treated as a current plan. A fresh response without
the plan name clears the previous badge; an explicit account `plans` override
still takes precedence. If the provider supplies no name and no override exists,
the badge remains absent. A stored plan name has the same snapshot age and account
ownership as its quota values.

`qhud --provider-health` prints bounded local connection diagnostics without
emails, account IDs, paths, or credentials. `qhud --refresh-health` explicitly
refreshes all three providers through the same commands as the widget, then
prints those diagnostics. The latter performs provider requests. Both run
without opening another window; ordinary background observation stays passive.

For a Windows source build, `scripts/Build-Windows.ps1 -DependenciesRoot <path>`
uses a separate dependency directory without overwriting an existing modified
qmonster checkout. The default remains the adjacent `qhud-deps` directory.

| Provider | Passive observation | Explicit refresh |
| --- | --- | --- |
| Claude | Local identity, statusline, and usage cache where available. | Existing OAuth access token sent to the provider usage endpoint. |
| Codex | Local identity and supported pane statusline data. | Active account via `codex app-server`; usage/account endpoints as fallback and for saved credentials. |
| Antigravity | Local account, native daemon readiness, and pane information where available. | Verified native Windows daemon or a running `agy` process, over loopback RPC. |

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
