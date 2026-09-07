<!-- qhud:languages -->
<p align="center">
  <strong>English</strong> · <a href="../i18n/ko/docs/03-spec/SPEC.md">한국어</a> · <a href="../i18n/zh-CN/docs/03-spec/SPEC.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# SPEC — qhud

> Status: living (rewritten from scratch against v0.6.0 source) · Date:
> 2026-09-07 · Owner: chquandogong
> What qhud must do. ARCHITECTURE says how it is built, DECISION_LOG says why.
> FR-1 … FR-24 keep the numbers other documents already cite; FR-25 … FR-34
> are the requirements that shipped in v0.5.1 → v0.6.0 without a spec row.

## Product statement

One question, answered without switching to anything: **how much room is left
on each AI CLI account, and when does it reset?** Everything else the widget
shows is in service of that. Where a terminal multiplexer is available, the
per-pane view is a second, narrower answer: what each running session is doing
right now.

That ordering matters and it changed. v0.1 was a pane monitor that also showed
quota; from v0.5.0 the operator's stated purpose is every account and
workspace at a glance so they never open a provider's web page again, and
v0.6.0's flagship platform observes no panes at all.

## Provider vocabulary

Three names exist for some providers and the code is the authority. This table
is the mapping every other document should follow.

| Payload value | Section header | Prose name  | Notes                                                                                                                                               |
| ------------- | -------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `claude`      | CLAUDE         | Claude Code |                                                                                                                                                     |
| `codex`       | CODEX          | Codex       | One login may own several workspaces                                                                                                                |
| `agy`         | AGY            | Antigravity |                                                                                                                                                     |
| `gemini`      | GEMINI         | Gemini      | A sibling provider string **and** a pool name inside an agy reading — `gemini-*` buckets take agy's primary gauges, other pools become scoped chips |

## Functional requirements

### The widget as an object on the desktop

| ID    | Requirement                                                                                                                             | Status                                                      |
| ----- | --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| FR-1  | Frameless, transparent, rounded window on the desktop layer: below all windows, sticky on all workspaces, absent from taskbar and pager | done, Linux — verified via `_NET_WM_STATE` 2026-09-07       |
| FR-2  | Drag-move by the top and footer bars, anywhere across monitors                                                                          | done (self-driven, D-008)                                   |
| FR-3  | Resize via the ◢ grip; content reflows                                                                                                  | done (self-driven, D-008)                                   |
| FR-4  | Position and size persist across restarts                                                                                               | done (window-state plugin, checkpointed every ~30 s)        |
| FR-10 | Tray: Show/Hide, Pin above windows, Reset position, Quit; the widget survives without a tray                                            | done (best-effort)                                          |
| FR-11 | Ctrl+wheel zooms 70–160%, persisted, pointer-only                                                                                       | done (D-012)                                                |
| FR-12 | Layer peek via tray check and `qhud --peek`; duplicate launches are absorbed                                                            | done (D-012)                                                |
| FR-26 | The widget detects its own frozen presentation and heals it without operator action: sample its own pixels, hide-and-show, then re-exec | done (v0.5.1, D-017) — Linux only                           |
| FR-29 | A native Windows x64 widget renders the same account usage through WebView2, with Windows paths and hidden helper processes             | done (v0.6.0, D-020) — pane observation explicitly excluded |

### What it renders

| ID    | Requirement                                                                                                                                                                      | Status                                          |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| FR-5  | Poll the qmonster pipeline every 2 s; render pane tiles with a status pill and context gauge, and an account-scoped quota strip per provider                                     | done (D-011)                                    |
| FR-6  | Click a tile to expand config chips and any conflict banner; click again to compact; selection persists                                                                          | done                                            |
| FR-7  | Severity bands on gauges: under 60 good, 60–74 concern, 75–84 warn, 85 and over critical                                                                                         | done                                            |
| FR-8  | Reset countdowns and idle-elapsed badges tick locally between polls                                                                                                              | done                                            |
| FR-14 | Provider-grouped strip: provider as the section header, account and plan on the identity line, one gauge line per window                                                         | done (v0.4.0)                                   |
| FR-25 | Strip rows are selectable and expand their detail inline; selecting a row is never a network action                                                                              | done (v0.5.1)                                   |
| FR-31 | The usage strip scrolls when rows exceed the window; long model labels wrap rather than truncate; the exact reset date and time are available on a gauge and in the expanded row | done (v0.6.0)                                   |
| FR-9  | With no multiplexer, render real local account rows and dated quota with zero panes; example data requires `--demo` and shows a DEMO badge; re-probe live every 10 s             | done (v0.6.0 replaced the demo fallback, D-005) |
| FR-30 | A mux-less start never invents usage: missing readings stay missing, and an account with no saved reading still renders so its refresh control is reachable                      | done (v0.6.0)                                   |

### Whose numbers these are

| ID    | Requirement                                                                                                                                                                                                           | Status                                                                          |
| ----- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| FR-13 | Every quota row names the account it belongs to — email or id, organization, plan, and both tiers of a team seat — read from local files only                                                                         | done (D-013)                                                                    |
| FR-15 | Ever-connected accounts with no live credential render as dated placeholders collapsed behind one line, dismissable; a live credential is never hidden                                                                | done (D-013)                                                                    |
| FR-22 | Several Claude accounts render at once via registry config directories, one row per account with its own snapshot and refresh; partial fetch failure stays partial                                                    | done (D-015)                                                                    |
| FR-27 | A row's identity is (account, organization): one login holding a team seat and a personal organization is two rows with separate pools, and a refresh result is matched to its row by config directory and account id | done (v0.5.2, D-018) — a real second-organization row is still field-unverified |
| FR-34 | A saved snapshot is used only for the account currently signed in; ownership is checked before age, so a newer reading from a previous login cannot mask this login's older one                                       | done (v0.6.0)                                                                   |

Known limit, carried since D-015: a pane's account is not attributable, so
pane-fed gauges always belong to the default account's row.

### Getting the numbers

| ID    | Requirement                                                                                                                                                             | Status               |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------- |
| FR-16 | Claude per-model usage refreshes on an explicit control or `--refresh-claude`, never on a timer and never through the OAuth refresh grant                               | done (D-014)         |
| FR-17 | Codex per-workspace quota on explicit request; a response describing a different workspace is dropped, not mislabelled                                                  | done (v0.4.0)        |
| FR-19 | Claude usage-credit spend renders on the account line as minor-unit money with the provider's own severity, and is hidden for plans without it                          | done (v0.5.0)        |
| FR-20 | Explicit-refresh results persist across restarts and render with their true origin and age; live pane data always wins                                                  | done (v0.5.0)        |
| FR-21 | One control refreshes every provider concurrently and mirrors the union of their states; per-provider controls remain; `--refresh-all` relays from a shortcut           | done (v0.5.0)        |
| FR-23 | agy quota on explicit refresh via the CLI's own loopback RPC — no token, machine-local, ports discovered from the operating system; the last read persists              | done (v0.5.0)        |
| FR-24 | When the active Codex login's direct fetch fails, a short-lived `codex app-server` child answers instead, so the CLI owns token rotation and qhud does not use the token itself on this path | done (v0.5.0, D-016) |
| FR-33 | Provider refreshes running at the same time preserve each other's saved results                                                                                         | done (v0.6.0)        |

### Being trustworthy about it

| ID    | Requirement                                                                                                                                                                                                              | Status                         |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------ |
| FR-18 | Wrong output is visible from outside the webview: the widget reports its rendered structure, the text and labels it drew, every real pointer event, and any frontend exception; every fetch path has a command-line twin | done (v0.4.0, extended v0.5.1) |
| FR-28 | Integer-meaning wire numbers are read for their meaning, no single optional field can fail a whole response, and a rejected body names the field that drifted                                                            | done (v0.5.3, D-019)           |
| FR-32 | A model-only reading survives a restart with each window's own percentage and reset; a model absent from the provider's current response is never synthesized as 0%                                                      | done (v0.6.0)                  |

## Non-functional requirements

**Observe-only.** Zero writes anywhere under qmonster's own directory, zero
notifications. The TUI owns that state and the alerting (D-004).

**Passive by default; network only on request** (D-013, D-014, D-016). The 2 s
loop reads local files and the multiplexer, makes no provider API requests,
and selects identity fields without using tokens for observation. Its only other work is a window-geometry checkpoint every ~30 s
and, on Linux, a pixel sample every ~28 s. Everything that reaches further
runs from an explicit operator gesture or its command-line twin, and never
runs an OAuth refresh grant. On the fetch paths qhud reads an access token for
Claude and Codex; agy needs no credential, and the Codex fallback delegates authentication to the CLI.

**Selecting is not fetching.** Since v0.5.1 no selection, expansion or row
click performs network work. Only the refresh controls and the relay flags do.

**Size.** The Linux release binary is 15.07 MiB at v0.6.0, against a 25 MB
budget (D-014). Two corrections to the record: every earlier figure was a
local build measurement, and the budget was silently exceeded at 25.13 MiB in
v0.5.3 because the release profile sat in a non-root workspace manifest where
cargo ignores it. Measure the release asset, not the local build. No Windows
figure has been recorded.

**Frontend.** Static HTML, CSS and JavaScript. No bundler, no node_modules, no
Node or npm required to build on either platform.

**Scale envelope.** One workstation. The pane view is designed for 1–12 panes
and beyond that the interaction model should change rather than be stretched.
The account view is designed for the strip to outgrow the window: v0.6.0 made
it scroll because eight accounts across three providers do not fit, and zero
panes is now a normal, supported state rather than an error.

## The payload contract

One Tauri event, `qhud://report`, emitted every 2 s. Source of truth is
`src-tauri/src/view.rs`; schema version is 1 and every addition has been
additive.

```jsonc
{
  "schema": 1,
  "source": "live",        // "live" | "local" | "demo"
  "backend": "herdr",      // "herdr" | "tmux" | null
  "generated_at_ms": 1788790282397,
  "poll_secs": 2,

  "quotas": [{             // one row per (provider, account, organization)
    "provider": "claude",  // claude | codex | agy | gemini
    "h5": { "pct": 49, "source": "providerofficial",
            "reset_unix": 1788438599, "of_tokens": null },
    "d7": { "pct": 7, "source": "providercache",
            "reset_unix": 1788976799, "of_tokens": null },
    "from_label": "claude:1:main",   // pane whose reading won; "" if synthesized
    "session": "dogu-3d-studio",
    "account": {
      "display": "chquan@dogu.xyz", // precomputed label → email → id
      "label": null, "email": "…", "account_id": "…",
      "org": "…", "org_type": "claude_team", "org_id": "…",
      "tiers": [{ "kind": "org", "tier": "max_5x" }],
      "plan": "team", "config_dir": null   // null ⇒ the default account
    },
    "origin": "pane",              // pane | cache | fetched
    "cache_fetched_at_ms": 1788790243688,
    "scoped": [{ "kind": "weekly_scoped", "scope": "Fable",
                 "pct": 12, "reset_unix": 1788976799 }],
    "extra": { "enabled": true, "used_minor": 4997, "currency": "USD",
               "exponent": 2, "limit_minor": null, "percent": 0,
               "severity": "normal", "limit_reached": false }
  }],

  "panes": [{
    "pane_id": "wC:p1", "label": "claude:1:main", "session": "…",
    "provider": "claude", "status": "active", "status_label": "active",
    "elapsed_secs": null, "cli_version": null, "update_hint": null,
    "model": "…", "effort": "max", "branch": null, "cwd": "~/qhud",
    "mem": "165 KB", "cost_usd": 0.0, "flags": [],
    "gauges": { "ctx": { … }, "h5": null, "d7": { … } },
    "conflicts": [{ "reason": "…", "severity": "warning",
                    "paths": ["…"], "peers": ["codex:1:review"] }]
  }],

  "summary": { "panes": 8, "conflicts": 0, "max_5h_pct": 6 },

  "account_placeholders": [{ "provider": "claude", "key": "personal-free",
                             "label": "…", "hint": "…", "plan": "free" }],
  "workspace_names":  { "<account_id>": "business" },
  "workspace_plans":  { "<account_id>": "ChatGPT Business" },
  "codex_workspaces": [{ "account_id": "…", "name": null,
                         "plan_type": "prolite", "credits_balance": "0",
                         "active": true,
                         "windows": [{ "label": "weekly", "used_percent": 41,
                                       "reset_unix": 1789147159,
                                       "scope": null }] }],
  "codex_fetched_at_ms": 1788790245717
}
```

Fields that are usually empty are omitted rather than sent as null:
`account`, `origin`, `cache_fetched_at_ms`, `scoped`, `extra`,
`account_placeholders`, `workspace_names`, `workspace_plans`,
`codex_workspaces`, `codex_fetched_at_ms`. `qhud --dump` prints this exact
payload.

**Contract rules.**

- **Facts render at the scope where they are true** (D-011). Quota is an
  account fact and appears once per account row; context and status are pane
  facts and appear on tiles. Per-pane `gauges.h5`/`d7` remain in the payload
  because the rollup needs them, but no tile renders them.
- **Units convert once, at this boundary.** Pressures leave as integer
  percents, reset instants as unix seconds; the frontend owns countdown text.
  The webview never sees qmonster types.
- **Window labels are wire values** (`5h`, `daily`, `weekly`, `30d`,
  `yearly`); the interface displays one duration vocabulary — 5H, 1D, 7D, 30D
  — everywhere, because "weekly" and "7D" are one seven-day rolling window and
  one fact must not wear two names.
- **`scoped[].kind`** is the provider's own word for a non-primary window:
  `session`, `weekly_all`, `weekly_scoped` from Claude, and `pool_<label>`
  for any provider's extra pools, with `scope` carrying the model or pool
  name. An unrecognized kind degrades to a tooltip line rather than vanishing.
- **`origin` is the honesty field.** `pane` is a live reading; `cache` is the
  CLI's own on-disk copy; `fetched` is qhud's last explicit refresh. Anything
  but `pane` must render its age, and an absent origin means the row makes no
  claim at all.
- **Money is minor units** plus a currency and an exponent, so no float money
  round-trips.

## Verification

Automated gates on both platforms, and publication of a release waits for
both: format, lint with warnings as errors, the full test suite, and a release
build. The suite is 98 tests at v0.6.0.

Interaction claims have a protocol (D-010): synthetic X11 input alone is
inadmissible because it bypasses the compositor's surface picking, which is
where real input was being stolen. A claim requires compositor-path injection
or a human hand, confirmed through the widget's own breadcrumbs.

Rendering claims have a separate protocol (D-017): breadcrumbs prove logic,
never paint. Pixels **are** verifiable — hash the window twice a few seconds
apart, or read the frame guard's own log — and the absence of an error is not
proof that anything was drawn.

The manual checklist, its dated evidence and the still-pending rows live in
TEST_PLAN.

## Out of scope

- **Actuation.** qhud never sends keys to a pane, restarts a CLI, or changes a
  provider setting.
- **Notifications.** Alerting belongs to the TUI and the providers; a second
  notifier would double-fire (D-004).
- **A settings UI.** Display names and registered accounts are hand-edited in
  one operator-owned file.
- **macOS.**
- **Native Windows terminal-pane observation.** The Windows widget shows
  account usage; discovering and attributing Windows Terminal or PowerShell
  panes is not implemented (D-020). Linux `/proc`-derived process metrics may
  simply be empty there.
- **Installers and package formats.** The release set is a Linux tarball and a
  Windows ZIP: no npm, deb, AppImage or MSI, and on Windows no installer,
  autostart or shortcut registration.
- **Per-gauge threshold configuration.** The severity bands are inherited from
  the design mockup and are not configurable.
- **Invented data.** No quota is synthesized for a pool a provider did not
  return, and no missing reading is rendered as zero.
