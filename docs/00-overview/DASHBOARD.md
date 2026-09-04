# DASHBOARD — qhud

> Status: v0.5.3 released · Date: 2026-09-04 · Owner: chquandogong
> Single source of truth = this git repo. This board is the handoff
> surface: read it first when resuming work on another session/agent.

## State

| Item               | Value                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------- |
| Version            | v0.5.3 — [releases](https://github.com/chquandogong/qhud/releases) (v0.5.1 2026-08-14, v0.5.2 2026-08-17, v0.5.3 2026-09-04) |
| Pipeline dep       | qmonster @ `6a21c44` (pinned rev — carries the cwd/attribution fixes)                                   |
| Verified on        | Ubuntu 24.04 · GNOME 46 Wayland · 2 monitors (scale 1) · herdr live (8 agent panes on 2026-09-03)       |
| Quality gates      | fmt ✅ · clippy -D warnings ✅ · tests 79 ✅ · release build ✅ (CI was red on main 08-07→08-10; fixed) |
| Input verification | **compositor-path only** (Mutter RemoteDesktop injection or human hand — XTEST inadmissible, D-010)     |
| Cross-validation   | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 adopted (CROSS_VALIDATION_LOG)                                  |
| Frame guard field  | 08-26 → 09-04 (journal coverage): **21 freezes, 21 remap heals, 0 re-execs, 0 visible incidents** (D-017)  |
| Known live gap     | `~/claude-personal` credential expired 2026-08-17 → that row 401s on every ⟳ until re-login (harmless)  |

## Decision index (full entries in DECISION_LOG)

D-001 second frontend · D-002 Tauri v2 · D-003 XWayland/EWMH layer ·
D-004 qmonster lib + NoopSink · D-005 demo=parity fixture · D-006
tarball releases · D-007 mux-backend factory (herdr) · D-008
self-driven geometry · D-009 pointerdown selection · D-010 DING
interception + verification protocol · D-011 scope-correct display ·
D-012 zoom + peek + signal prohibition · D-013 local account identity ·
D-014 passive by default, network on request · D-015 multi-account via
per-account CLI config dirs · D-016 delegated fetch paths (codex
app-server, agy loopback RPC) · D-017 the widget audits its own pixels
(frame guard) · D-018 row identity is (account, organization) ·
D-019 lenient wire numbers, and a rejected body names its field.

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
| Row identity = (account, org) — one login, two orgs, separate pools (v0.5.2)                                                     | done 2026-08-17                  | claude+chquandogong |
| Claude ⟳ dead two days on a float `used_credits`: lenient wire numbers + a parse error that names the field (v0.5.3, D-019)      | done 2026-09-04                  | claude+chquandogong |
| ⏳ TEST_PLAN pending rows (overview / lock / suspend / hotplug / fullscreen)                                                     | **todo — first on-machine pass** | operator            |
| Live verification with a plain tmux server (fallback path)                                                                       | todo                             | operator            |
| Personal-org login into `~/claude-personal` (pick the PERSONAL org at the CLI org step; registry already wired; OAuth keeps auto-selecting the team session) | todo — operator, whenever wanted | operator            |
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

v0.5.3 tagged and released (the Claude ⟳ parse fix). The installed
binary at `~/.local/bin/qhud` is the v0.5.3 build, relaunched
2026-09-03 16:46; its own ⟳ answers live.

Next meaningful units, in order:

1. **Operator verification pass** — the TEST_PLAN ⏳ rows (overview /
   lock / suspend / hotplug / fullscreen), a plain-tmux backend check,
   and the personal-org login into `~/claude-personal` (registry
   already wired; pick the PERSONAL org at the CLI's organization step,
   which also clears the standing 401 on that row).
2. **Docs backfill** — SPEC still says "implemented through v0.5.0" and
   has no FRs for v0.5.1–v0.5.3; ARCHITECTURE is a v0.4.0 document (its
   module table predates `agy_usage.rs`, `fetched_store.rs`,
   `frame_guard.rs` and every line count is stale, and `:162` still
   claims pixels are not verifiable, which D-017 disproved); README's
   last edit was 08-12. RISK_REGISTER has no row for the display-sleep
   frame freeze, now the most frequent field event by far.
3. **Backlog** — tile→pane focus jump remains the highest-value small
   item.
