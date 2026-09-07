<!-- qhud:languages -->
<p align="center">
  <strong>English</strong> · <a href="../i18n/ko/docs/00-overview/DASHBOARD.md">한국어</a> · <a href="../i18n/zh-CN/docs/00-overview/DASHBOARD.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# DASHBOARD — qhud

> Status: v0.6.1 released · Date: 2026-09-07 · Owner: chquandogong
> Single source of truth = this git repo. This board is the handoff
> surface: read it first when resuming work on another session/agent.

## State

| Item               | Value                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------- |
| Version            | [v0.6.1](https://github.com/chquandogong/qhud/releases/tag/v0.6.1) — rewritten SPEC/ARCHITECTURE and dated Ubuntu evidence; runtime behavior identical to [v0.6.0](https://github.com/chquandogong/qhud/releases/tag/v0.6.0) (Linux x86_64 tarball + Windows x86_64 ZIP) |
| Pipeline dep       | qmonster @ `6a21c44`; canonical Git dependency on Linux, command-scoped Windows patch with lockfile restoration |
| Platforms          | Ubuntu 24.04 / GNOME and native Windows x64 / WebView2; Windows terminal-tab observation remains unsupported |
| Runtime evidence   | **Ubuntu 2026-09-07 on the published v0.6.0 Linux asset** (checksum-verified, installed): Ubuntu 24.04.3 · GNOME Shell 46.0 Wayland · kernel 7.0.0-28 · eDP-1 2560×1600 + HDMI-1 3840×2160 (scale 1) · webkit2gtk 2.52.6 · herdr 0.7.5 live with 8 panes. `_NET_WM_STATE` = BELOW+STICKY+SKIP_PAGER+SKIP_TASKBAR, `_NET_WM_DESKTOP` = 0xFFFFFFFF, `WM_CLASS` = qhud/Qhud; two `xwd` hashes 3 s apart differed (painting); frame guard armed; all three providers answered ⟳. Windows v0.6.0: native display, email, reset, Codex refresh and app-server fallback verified |
| Quality gates      | Ubuntu fmt/clippy/tests **98/98**/release build passed; Windows tests **98/98**/release build passed — [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064). Re-run locally on Ubuntu 2026-09-07 against the v0.6.0 tree: fmt clean, clippy `-D warnings` clean, 98/98, release build 6 m 21 s from the pinned Git dependency with no sibling checkout |
| Input verification | Ubuntu: **compositor-path only** (Mutter RemoteDesktop injection or human hand — XTEST inadmissible, D-010) |
| Cross-validation   | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 adopted (CROSS_VALIDATION_LOG)                                  |
| Frame guard field  | 08-26 → 09-07 (journal-captured window): **28 freezes, 28 remap heals, 0 re-execs, 0 visible incidents** (D-017). The v0.5.1/v0.5.2 counts came from a terminal-attached instance whose stderr never reached the journal, so they cannot be re-derived |
| Known live gap     | Still open on 2026-09-07: the `~/claude-personal` credential expired 2026-08-17, so that row answers 401 on every ⟳. Only an operator login clears it; no release renews credentials. The default account is unaffected — partial failure stays partial |

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
D-019 lenient wire numbers, and a rejected body names its field ·
D-020 Windows adaptation stays scoped; usage keeps its account and window.

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
| v0.6.0 release publication and verified downloads                                                                         | [Release 34127575141 passed](https://github.com/chquandogong/qhud/actions/runs/34127575141); both archive checksums match | codex+chquandogong |
| Ubuntu re-verification of the published v0.6.0 build, closing the "not re-exercised on this Windows host" gap                    | done 2026-09-07                  | claude+chquandogong |
| SPEC and ARCHITECTURE rewritten from scratch for v0.6.0/v0.6.1; RISK_REGISTER and ASSUMPTIONS refreshed against field evidence   | done 2026-09-07                  | claude+chquandogong |
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

v0.6.1 is a documentation and verification release: identical runtime
behavior to v0.6.0, with `docs/03-spec/SPEC.md` and
`docs/03-spec/ARCHITECTURE.md` rewritten from scratch against the
v0.6.0 source, and RISK_REGISTER / ASSUMPTIONS refreshed against dated
field evidence.

**The Ubuntu gap v0.6.0 left open is now closed.** v0.6.0 shipped from a
Windows host and recorded that "Ubuntu desktop integration was not
re-exercised"; on 2026-09-07 the published Linux asset was
checksum-verified, installed and exercised on the reference Ubuntu
machine. Desktop-layer states, pixel liveness, herdr observation with 8
panes and all three provider refreshes were confirmed — see Runtime
evidence above and the dated rows in TEST_PLAN.

Also confirmed this session: the v0.5.3 wire-drift fix has held since
2026-09-04 (every Claude ⟳ succeeded), and v0.6.0's release profile move
to the workspace root cut the Linux binary from 25.13 MiB to 15.07 MiB —
`[profile.release]` had been sitting in a non-root workspace member where
cargo ignored it, so `strip`/`lto`/`codegen-units = 1` had never applied
to any earlier release.

Next meaningful units, in order:

1. **Operator verification pass** — the TEST_PLAN ⏳ rows that need a
   human at the machine (GNOME overview, lock/unlock, suspend/resume,
   monitor hotplug, fullscreen), a plain-tmux backend check, and the
   personal-org login into `~/claude-personal` (registry already wired;
   pick the PERSONAL org at the CLI's organization step, which also
   clears the standing 401 on that row).
2. **Windows field breadth** — Windows evidence is one host and one smoke
   check. Terminal-pane attribution there is unimplemented by design, so
   the honest Windows claim stays "account quota widget", not "session
   HUD".
3. **Backlog** — tile→pane focus jump remains the highest-value small
   item; then the GNOME Shell extension, upstream `ObserveSnapshot`
   export and unpin, `.deb` packaging, agy multi-account.
