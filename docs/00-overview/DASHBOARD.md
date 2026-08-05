# DASHBOARD — qhud

> Status: v0.1.0 shipped · Date: 2026-08-05 · Owner: chquandogong
> Single source of truth = this git repo. This board is the handoff
> surface: read it first when resuming work on another session/agent.

## State

| Item                   | Value                                                                              |
| ---------------------- | ---------------------------------------------------------------------------------- |
| Version                | v0.1.0 (tag) — [release](https://github.com/chquandogong/qhud/releases/tag/v0.1.0) |
| Pipeline dep           | qmonster @ `aa2bd39` (pinned rev)                                                  |
| Verified on            | Ubuntu 24.04 · GNOME 46 Wayland · 2 monitors (scale 1)                             |
| Quality gates          | fmt ✅ · clippy -D warnings ✅ · tests 4/4 ✅ · release build ✅                   |
| Desktop-layer evidence | `_NET_WM_STATE_BELOW/STICKY/SKIP_*` + cross-monitor move (FEASIBILITY_REPORT)      |
| Cross-validation       | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 adopted (CROSS_VALIDATION_LOG)             |

## Work board

| Task                                                                         | Status                           | Owner               |
| ---------------------------------------------------------------------------- | -------------------------------- | ------------------- |
| v0.1.0 wedge (widget + bridge + demo + docs + CI + release)                  | done                             | claude+chquandogong |
| ⏳ TEST_PLAN pending rows (overview / lock / suspend / hotplug / fullscreen) | **todo — first on-machine pass** | operator            |
| Live-data verification — **herdr backend, 4 real agent panes** (D-007)       | done 2026-08-05                  | claude+chquandogong |
| Live-data verification with a plain tmux server (fallback path)              | todo                             | operator            |
| `.deb` package (CV deferred item)                                            | backlog                          | —                   |
| Upstream `ObserveSnapshot` export in qmonster, then unpin                    | backlog                          | —                   |
| GNOME Shell extension layer (overview-clean pinning)                         | backlog                          | —                   |
| Click tile → focus that tmux pane                                            | backlog                          | —                   |

## Decision queue (human)

_None open._ D-001..D-006 recorded in DECISION_LOG; deferred items
above are backlog, not blockers.

## Resume point

Repo is clean at `v0.1.0`. Next meaningful unit of work is the
operator-run verification pass (TEST_PLAN ⏳ rows + live tmux check);
file findings as issues, then pick from the backlog.
