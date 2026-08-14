# DASHBOARD — qhud

> Status: v0.5.1 released · Date: 2026-08-14 · Owner: chquandogong
> Single source of truth = this git repo. This board is the handoff
> surface: read it first when resuming work on another session/agent.

## State

| Item               | Value                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------- |
| Version            | v0.5.1 — [releases](https://github.com/chquandogong/qhud/releases) (v0.5.0 2026-08-10, v0.5.1 2026-08-14) |
| Pipeline dep       | qmonster @ `6a21c44` (pinned rev — carries the cwd/attribution fixes)                                   |
| Verified on        | Ubuntu 24.04 · GNOME 46 Wayland · 2 monitors (scale 1) · herdr live (4 agent panes)                     |
| Quality gates      | fmt ✅ · clippy -D warnings ✅ · tests 75 ✅ · release build ✅ (CI was red on main 08-07→08-10; fixed) |
| Input verification | **compositor-path only** (Mutter RemoteDesktop injection or human hand — XTEST inadmissible, D-010)     |
| Cross-validation   | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 adopted (CROSS_VALIDATION_LOG)                                  |

## Decision index (full entries in DECISION_LOG)

D-001 second frontend · D-002 Tauri v2 · D-003 XWayland/EWMH layer ·
D-004 qmonster lib + NoopSink · D-005 demo=parity fixture · D-006
tarball releases · D-007 mux-backend factory (herdr) · D-008
self-driven geometry · D-009 pointerdown selection · D-010 DING
interception + verification protocol · D-011 scope-correct display ·
D-012 zoom + peek + signal prohibition · D-013 local account identity ·
D-014 passive by default, network on request · D-015 multi-account via
per-account CLI config dirs · D-016 delegated fetch paths (codex
app-server, agy loopback RPC) · D-017 the widget audits its own pixels (frame guard).

## Work board

| Task                                                                                                                             | Status                           | Owner               |
| -------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- | ------------------- |
| v0.1.0 wedge (widget + bridge + demo + docs + CI + release)                                                                      | done 2026-08-05                  | claude+chquandogong |
| herdr backend live (D-007) · input architecture (D-008/9/10)                                                                     | done                             | claude+chquandogong |
| Scope-correct display (D-011, v0.2.0)                                                                                            | done 2026-08-06                  | claude+chquandogong |
| Zoom · peek · single-instance (D-012, v0.3.0) + light tray glyph                                                                 | done 2026-08-06                  | claude+chquandogong |
| Quota you can trust: attribution fixes, identity, ⟳, workspaces (v0.4.0)                                                         | done 2026-08-07                  | claude+chquandogong |
| Every account at a glance: multi-account, persistence, extra usage, agy RPC, codex app-server, refresh-all (v0.5.0, D-015/D-016) | done 2026-08-10                  | claude+chquandogong |
| Tag + release v0.5.0 (CI green again — first since e6e31ed)                                                                      | done 2026-08-10                  | claude+chquandogong |
| "Selection doesn't work" saga: ptr/qsel input forensics → strip rows select, ⟳-only network → pixel frame guard (v0.5.1, D-017) | done 2026-08-14                  | claude+chquandogong |
| ⏳ TEST_PLAN pending rows (overview / lock / suspend / hotplug / fullscreen)                                                     | **todo — first on-machine pass** | operator            |
| Live verification with a plain tmux server (fallback path)                                                                       | todo                             | operator            |
| Real second-account setup (`CLAUDE_CONFIG_DIR` sign-in + registry entry)                                                         | todo — needs operator login      | operator            |
| Click tile → focus that pane in the terminal                                                                                     | backlog                          | —                   |
| GNOME Shell extension: overview-clean pinning + DING coexistence                                                                 | backlog                          | —                   |
| Upstream `ObserveSnapshot` export in qmonster, then unpin                                                                        | backlog                          | —                   |
| `.deb` package (CV deferred item)                                                                                                | backlog                          | —                   |
| agy multi-account (OS-keyring reverse engineering)                                                                               | backlog                          | —                   |

## Decision queue (human)

_None open._ DING stays disabled on the reference machine (empty
`~/Desktop`, operator-approved); re-enable costs widget interaction
until the companion extension exists.

## Resume point

v0.5.1 tagged and released; the frame guard healed 3 freezes in its
first 22 h with zero operator-visible incidents. Next meaningful
units: operator
verification pass (TEST_PLAN ⏳ rows, plain-tmux check, sign a second
Claude account in under its own `CLAUDE_CONFIG_DIR` and register it),
then pick from the backlog — tile→pane focus jump remains the
highest-value small item.
