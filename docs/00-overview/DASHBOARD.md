# DASHBOARD — qhud

> Status: v0.4.0 shipped · Date: 2026-08-07 · Owner: chquandogong
> Single source of truth = this git repo. This board is the handoff
> surface: read it first when resuming work on another session/agent.

## State

| Item               | Value                                                                                                      |
| ------------------ | ---------------------------------------------------------------------------------------------------------- |
| Version            | v0.3.1 — [releases](https://github.com/chquandogong/qhud/releases) (v0.1.0 → v0.3.x shipped 2026-08-05/06) |
| Pipeline dep       | qmonster @ `aa2bd39` (pinned rev)                                                                          |
| Verified on        | Ubuntu 24.04 · GNOME 46 Wayland · 2 monitors (scale 1) · herdr 0.7.5 live (4 agent panes)                  |
| Quality gates      | fmt ✅ · clippy -D warnings ✅ · tests 6 ✅ · release build ✅ (every push, CI)                            |
| Input verification | **compositor-path only** (Mutter RemoteDesktop injection or human hand — XTEST inadmissible, D-010)        |
| Cross-validation   | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 adopted (CROSS_VALIDATION_LOG)                                     |

## Decision index (full entries in DECISION_LOG)

D-001 second frontend · D-002 Tauri v2 · D-003 XWayland/EWMH layer ·
D-004 qmonster lib + NoopSink · D-005 demo=parity fixture · D-006
tarball releases · D-007 mux-backend factory (herdr) · D-008
self-driven geometry · D-009 pointerdown selection · D-010 DING
interception + verification protocol · D-011 scope-correct display ·
D-012 zoom + peek + signal prohibition.

## Work board

| Task                                                                         | Status                           | Owner               |
| ---------------------------------------------------------------------------- | -------------------------------- | ------------------- |
| v0.1.0 wedge (widget + bridge + demo + docs + CI + release)                  | done 2026-08-05                  | claude+chquandogong |
| herdr backend live (D-007) · input architecture (D-008/9/10)                 | done                             | claude+chquandogong |
| Scope-correct display (D-011, v0.2.0)                                        | done 2026-08-06                  | claude+chquandogong |
| Zoom · peek · single-instance (D-012, v0.3.0) + light tray glyph             | done 2026-08-06                  | claude+chquandogong |
| Docs/README professional pass (banner, fresh screenshots)                    | done 2026-08-06                  | claude+chquandogong |
| ⏳ TEST_PLAN pending rows (overview / lock / suspend / hotplug / fullscreen) | **todo — first on-machine pass** | operator            |
| Live verification with a plain tmux server (fallback path)                   | todo                             | operator            |
| Click tile → focus that pane in the terminal                                 | backlog                          | —                   |
| GNOME Shell extension: overview-clean pinning + DING coexistence             | backlog                          | —                   |
| Upstream `ObserveSnapshot` export in qmonster, then unpin                    | backlog                          | —                   |
| `.deb` package (CV deferred item)                                            | backlog                          | —                   |

## Decision queue (human)

_None open._ DING stays disabled on the reference machine (empty
`~/.Desktop`, operator-approved); re-enable costs widget interaction
until the companion extension exists.

## Resume point

Repo clean at `v0.3.1`. Next meaningful units: operator verification
pass (⏳ rows + plain-tmux check), then pick from the backlog —
tile→pane focus jump is the highest-value small item.
