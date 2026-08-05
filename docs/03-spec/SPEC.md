# SPEC — qhud v0.1

> Status: implemented · Date: 2026-08-05 · Owner: chquandogong

## Functional requirements

| ID    | Requirement                                                                                                                                              | Status                              |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| FR-1  | Frameless, transparent, rounded widget window on the desktop layer: below all windows, sticky on all workspaces, absent from taskbar/pager               | done (verified via `_NET_WM_STATE`) |
| FR-2  | User can drag-move the widget (top/footer bars) anywhere across monitors                                                                                 | done (`data-tauri-drag-region`)     |
| FR-3  | User can resize via the ◢ grip; content reflows                                                                                                          | done (`startResizeDragging`)        |
| FR-4  | Position/size persist across restarts                                                                                                                    | done (`tauri-plugin-window-state`)  |
| FR-5  | Poll qmonster pipeline every 2 s; render pane tiles with status pill + CTX/5H/7D gauges                                                                  | done                                |
| FR-6  | Click a tile → expand: config chips (model/effort/flags/branch/cwd/mem/cost) + conflict banner; click again → compact. Selection persists (localStorage) | done                                |
| FR-7  | Severity bands on gauges: `<60` good · `60–74` concern · `75–84` warn · `≥85` crit (mockup legend)                                                       | done                                |
| FR-8  | Reset countdowns (`resets 47m`) and idle-elapsed badges tick locally between polls                                                                       | done                                |
| FR-9  | No tmux server ⇒ demo payload (mockup fixture) with a visible `DEMO` badge; re-probe live every 10 s                                                     | done                                |
| FR-10 | Tray icon: Show/Hide, Quit; widget survives without tray                                                                                                 | done (best-effort)                  |

## Non-functional

- **Observe-only**: zero writes to `~/.qmonster` (NoopSink), zero
  notifications (SilentNotify), no network.
- Release binary ≤ 20 MB; idle CPU ≈ one qmonster observe tick / 2 s.
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
  "generated_at_ms": 0,
  "poll_secs": 2,
  "summary": { "panes": 3, "conflicts": 1, "max_5h_pct": 88 },
  "panes": [{
    "pane_id": "%25",
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

Contract rules: pressures leave the backend as integer percents;
reset instants as unix seconds (frontend owns countdown text); the
webview never sees qmonster types.

## Out of scope (v0.1)

Actuation, notifications, settings UI, Windows/macOS, npm/deb
packaging, per-gauge threshold configuration (inherits mockup bands).
