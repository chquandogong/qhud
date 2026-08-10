# SPEC — qhud

> Status: living (implemented through v0.5.0) · Date: 2026-08-10 · Owner: chquandogong

## Functional requirements

| ID    | Requirement                                                                                                                                                                                                                                                   | Status                              |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| FR-1  | Frameless, transparent, rounded widget window on the desktop layer: below all windows, sticky on all workspaces, absent from taskbar/pager                                                                                                                    | done (verified via `_NET_WM_STATE`) |
| FR-2  | User can drag-move the widget (top/footer bars) anywhere across monitors                                                                                                                                                                                      | done (self-driven, D-008)           |
| FR-3  | User can resize via the ◢ grip; content reflows                                                                                                                                                                                                               | done (self-driven, D-008)           |
| FR-4  | Position/size persist across restarts                                                                                                                                                                                                                         | done (`tauri-plugin-window-state`)  |
| FR-5  | Poll qmonster pipeline every 2 s; render pane tiles with status pill + CTX gauge; account-scoped 5H/7D quota strip per provider (D-011)                                                                                                                       | done                                |
| FR-6  | Click a tile → expand: config chips (model/effort/flags/branch/cwd/mem/cost) + conflict banner; click again → compact. Selection persists (localStorage)                                                                                                      | done                                |
| FR-7  | Severity bands on gauges: `<60` good · `60–74` concern · `75–84` warn · `≥85` crit (mockup legend)                                                                                                                                                            | done                                |
| FR-8  | Reset countdowns (`resets 47m`) and idle-elapsed badges tick locally between polls                                                                                                                                                                            | done                                |
| FR-9  | No tmux server ⇒ demo payload (mockup fixture) with a visible `DEMO` badge; re-probe live every 10 s                                                                                                                                                          | done                                |
| FR-10 | Tray: Show/Hide, Pin above windows, Reset position, Quit; widget survives without tray                                                                                                                                                                        | done (best-effort)                  |
| FR-11 | Ctrl+wheel zooms the UI 70–160%, persisted (pointer-only)                                                                                                                                                                                                     | done (D-012)                        |
| FR-12 | Layer peek: tray check + `qhud --peek` argv relay; duplicate launches absorbed                                                                                                                                                                                | done (D-012)                        |
| FR-13 | Every quota row names the account it belongs to (email/id, org, plan, and both tiers of a team seat), read from local files only                                                                                                                              | done (D-013)                        |
| FR-14 | Provider-grouped strip: provider as section header, account+plan on the identity line, one gauge line per window                                                                                                                                              | done (v0.4.0)                       |
| FR-15 | Ever-connected accounts with no live credential render as dimmed dated placeholders, collapsed behind one line, dismissable; a live credential is never hidden                                                                                                | done (D-013)                        |
| FR-16 | Claude per-model usage refreshes on an explicit ⟳ (or `--refresh-claude`), never on a timer and never via the OAuth refresh grant                                                                                                                             | done (D-014)                        |
| FR-17 | Codex per-workspace quota on explicit request; a response describing a different workspace is dropped, not mislabelled                                                                                                                                        | done (v0.4.0)                       |
| FR-18 | Wrong output is visible from outside the webview: the widget reports its rendered structure, text and gauge counts; every fetch path has a CLI trigger                                                                                                        | done (v0.4.0)                       |
| FR-19 | Claude usage-credit spend ("extra usage") renders on the account line — minor-unit money end to end, severity from the provider's own signal, hidden for plans without it                                                                                     | done (v0.5.0)                       |
| FR-20 | Explicit-fetch results persist across restarts (`~/.config/qhud/fetched-usage.json`, temp+rename); the freshest snapshot renders with its true origin and age; live pane data always wins                                                                     | done (v0.5.0)                       |
| FR-21 | One topbar ⟳ refreshes every provider concurrently and mirrors the union of fetch states; per-provider triggers stay; `--refresh-all` relays from a shortcut                                                                                                  | done (v0.5.0)                       |
| FR-22 | Several Claude accounts render at once via registry `claude_config_dirs`, one row per account with its own snapshot and ⟳; partial fetch failure is partial; pane-fed gauges stay on the default account (a pane's account is not attributable — known limit) | done (D-015)                        |
| FR-23 | agy quota on explicit ⟳ via the CLI's own loopback RPC (no token, machine-local, /proc port discovery); gemini pools on the primary gauges, other pools as scoped chips; the last read persists                                                               | done (v0.5.0)                       |
| FR-24 | When the active Codex login's raw fetch fails, a short-lived `codex app-server` child answers `account/rateLimits/read` — the CLI owns token rotation, qhud touches no credential (D-016)                                                                     | done (v0.5.0)                       |

## Non-functional

- **Observe-only**: zero writes to `~/.qmonster` (NoopSink), zero
  notifications (SilentNotify).
- **Passive by default, network only on request** (D-013, D-014,
  D-016): the 2 s poll loop reads local files and the mux only — it
  never opens a socket and never touches a credential. Everything that
  reaches further runs from an explicit operator gesture (⟳ / row
  click / their CLI twins) and never runs an OAuth refresh grant:
  Claude's usage endpoint per signed-in account, Codex `/wham` per
  credential, agy's loopback RPC (machine-local, tokenless), and the
  `codex app-server` fallback (delegated to the CLI's own process,
  D-016).
- Release binary ≤ 25 MB; idle CPU ≈ one qmonster observe tick / 2 s.
  Raised from 20 MB in v0.4.0 — rustls + reqwest cost ≈ 2.8 MB and the
  original figure was an opening guess, not a measured constraint
  (D-014). `strip` and `lto` are already on.
- Frontend is static HTML/CSS/JS — no bundler, no node_modules.

## Scale envelope

Designed for **one workstation, 1–12 AI panes** (vertical tile stack
with scroll). Beyond ~12 panes the interaction paradigm should change
(grouping/filtering — explicitly out of scope for v0.1 and recorded as
a limit, not stretched).

## Payload contract (schema v1)

Emitted as Tauri event `qhud://report`; see `src-tauri/src/view.rs`.

```jsonc
{
  "schema": 1,
  "source": "live" | "demo",
  "backend": "herdr" | "tmux" | null,   // resolved mux backend; null in demo (additive, v0.1.1)
  "workspace_names": {"<account_id>": "personal"},          // additive, v0.4.0
  "workspace_plans": {"<account_id>": "ChatGPT Business"},  // additive, v0.4.0
  "account_placeholders": [{             // ever-connected, no live credential (additive, v0.4.0 — D-013)
    "provider": "codex", "key": "<id-or-email>", "label": "…", "plan": "…", "hint": "how to restore it"
  }],
  "quotas": [{                           // ACCOUNT-scoped rollup (additive, v0.2.0 — D-011).
                                         // One row per (provider, account): a provider may
                                         // carry several accounts since v0.5.0 (D-015), the
                                         // default account's row first.
    "provider": "claude",
    "account": {                         // additive, v0.4.0 — D-013; omitted when unknown
      "display": "chquan@dogu.xyz", "email": "…", "account_id": "…",
      "org": "DOGU", "org_type": "claude_team", "plan": "team (max_5x)",
      "tiers": [{"kind": "org", "tier": "…"}, {"kind": "user", "tier": "…"}]
    },
    "origin": "pane" | "cache" | "fetched", // additive, v0.4.0/v0.5.0 — a snapshot row must not
                                            // pass for live: "cache" is the CLI's own on-disk
                                            // copy, "fetched" is qhud's last explicit ⟳
    "cache_fetched_at_ms": 0,            // additive, v0.4.0 — freshness of the snapshot
    "scoped": [{"kind": "weekly_scoped" /* | "pool_5h" | "pool_weekly" */,
                "scope": "Fable", "pct": 22, "reset_unix": 0}],
    "extra": {                           // additive, v0.5.0 — usage-credit spend, minor units
      "enabled": true, "used_minor": 1234, "currency": "USD", "exponent": 2,
      "limit_minor": 5000, "percent": 25, "severity": "normal", "limit_reached": false
    },
    "h5": { "pct": 88, "source": "providerofficial", "reset_unix": 1754500000, "of_tokens": null },
    "d7": { "pct": 31, "source": "providerofficial", "reset_unix": 1754800000, "of_tokens": null },
    "from_label": "claude:1:main", "session": "~"   // freshest snapshot's pane
  }],
  "codex_workspaces": [{                 // additive, v0.5.0 — the last explicit Codex fetch,
    "account_id": "…", "name": "Personal", "plan_type": "prolite",
    "windows": [{"label": "weekly", "used_percent": 80, "reset_unix": 0},
                {"label": "weekly", "used_percent": 4, "reset_unix": 0,
                 "scope": "GPT-5.3-Codex-Spark"}], // scope = pool name; absent on the
                                                   // main pool. Duration alone labels
                                                   // both "weekly" (additive, v0.5.0)
    "credits_balance": "345.57",
    "active": true                       // the default login's own workspace — the SAME
                                         // pool the pane statusline feeds the codex row,
                                         // so it merges there (name on the row, windows
                                         // via snapshot rules); a ↳ row renders only for
                                         // active:false entries (additive, v0.5.0)
  }],                                    // dated by codex_fetched_at_ms; stored ≠ live
  "codex_fetched_at_ms": 0,              // additive, v0.5.0
  "generated_at_ms": 0,
  "poll_secs": 2,
  "summary": { "panes": 3, "conflicts": 1, "max_5h_pct": 88 },
  "panes": [{
    "pane_id": "%25",
    "session": "~",                    // workspace label (additive, v0.2.0)
    "label": "claude:1:main",          // provider:instance:role
    "provider": "claude",
    "status": "active|done|wait|limit|stale|dead",
    "status_label": "wait approval",
    "elapsed_secs": 42,                 // null when active
    "cli_version": "2.1.4",            // null when unknown
    "update_hint": "0.143",            // demo-only today
    "model": "opus-4.8", "effort": "max", "branch": "main",
    "cwd": "~/qhud", "mem": "48 KB", "cost_usd": 0.42,
    "flags": ["⏵⏵ bypass on"],
    "gauges": {
      "ctx": { "pct": 64, "source": "providerofficial", "reset_unix": null, "of_tokens": 1000000 },
      "h5":  { "pct": 88, "source": "providerofficial", "reset_unix": 1754500000, "of_tokens": null },
      "d7":  { "pct": 31, "source": "providerofficial", "reset_unix": 1754800000, "of_tokens": null }
    },
    "conflicts": [{ "reason": "…", "severity": "warning",
                    "paths": ["src/…"], "peers": ["codex:1:review"] }]
  }]
}
```

Contract rules: **facts render at the scope they are true** — 5h/7d
quota is account-scoped (provider strip; per-pane `gauges.h5/d7`
stay in the payload but tiles do not render them); CTX/status are
pane-scoped. Pressures leave the backend as integer percents;
reset instants as unix seconds (frontend owns countdown text); the
webview never sees qmonster types. Window labels are wire values
(`5h`/`daily`/`weekly`/`30d`); the frontend displays ONE duration
vocabulary — `5H`/`1D`/`7D`/`30D` — everywhere, because "weekly" and
"7D" are the same 7-day rolling window and one fact must not wear two
names (v0.5.0, operator report).

## Out of scope (v0.1)

Actuation, notifications, settings UI, Windows/macOS, npm/deb
packaging, per-gauge threshold configuration (inherits mockup bands).
