# DASHBOARD — qhud

> Status: v0.6.0 validated for release · Date: 2026-09-07 · Owner: chquandogong
> Single source of truth = this git repo. This board is the handoff
> surface: read it first when resuming work on another session/agent.

## State

| Item               | Value                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------- |
| Version            | v0.6.0 in preparation; last published v0.5.3 (2026-09-04) — [releases](https://github.com/chquandogong/qhud/releases) |
| Pipeline dep       | qmonster @ `6a21c44`; canonical Git dependency on Linux, command-scoped Windows patch with lockfile restoration |
| Platforms          | Ubuntu 24.04 / GNOME and native Windows x64 / WebView2; Windows terminal-tab observation remains unsupported |
| Runtime evidence   | Historical Ubuntu: GNOME 46 Wayland, 2 monitors, herdr live (2026-09-03). Windows v0.6.0: native display, email, reset, Codex refresh and app-server fallback verified |
| Quality gates      | Ubuntu fmt/clippy/tests **98/98**/release build passed; Windows tests **98/98**/release build passed — [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064) |
| Input verification | Ubuntu: **compositor-path only** (Mutter RemoteDesktop injection or human hand — XTEST inadmissible, D-010) |
| Cross-validation   | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 adopted (CROSS_VALIDATION_LOG)                                  |
| Frame guard field  | 08-26 → 09-04 (journal coverage): **21 freezes, 21 remap heals, 0 re-execs, 0 visible incidents** (D-017)  |
| Known live gap     | Last Ubuntu report (2026-09-04): `~/claude-personal` credential expired; a fresh login is needed for that row. This release does not renew credentials |

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
| Native Windows account widget; local fallback without a mux; model/reset visibility and ownership safeguards (v0.6.0)       | implemented; local tests 98/98  | codex+chquandogong |
| Portable Linux dependency manifest; Windows script-scoped patch; Ubuntu + Windows CI/release gates                          | both platform CI jobs passed    | codex+chquandogong |
| v0.6.0 release build and Windows final smoke                                                                               | passed; publication uses gated tag workflow | codex+chquandogong |
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

v0.6.0 adds the native Windows account widget while preserving the
Ubuntu source-build path. With no mux, both platforms now show real
local accounts and dated quota; demo panes require `--demo`. Model-only
snapshots survive restarts, and the quota area scrolls when rows exceed
the window. Model pools missing from a live provider response are not
invented.

Both remote platform jobs passed on code revision `1514e09`: Ubuntu
fmt/clippy, 98 tests and release build; Windows 98 tests and release build.
The local Windows v0.6.0 executable also passed a display/account-refresh
smoke check, including its app-server fallback. Ubuntu desktop integration
was not re-exercised on this Windows host; the field evidence above remains
historical. Publication is gated by the tagged release workflow.

Next meaningful units, in order:

1. **Release verification** — confirm the v0.6.0 tag workflow and its
   Linux/Windows downloads on GitHub Releases. The workflow publishes only
   after both platform jobs succeed and uses
   `docs/05-ops/releases/v0.6.0.md` for the release body.
2. **Operator verification pass** — the TEST_PLAN ⏳ rows (overview /
   lock / suspend / hotplug / fullscreen), a plain-tmux backend check,
   and the personal-org login into `~/claude-personal` (registry
   already wired; pick the PERSONAL org at the CLI's organization step,
   which also clears the standing 401 on that row).
3. **Docs backfill** — review older specifications and risk entries
   against the implemented behavior and dated field evidence.
4. **Backlog** — tile→pane focus jump remains the highest-value small
   item.
