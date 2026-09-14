<!-- qhud:languages -->
<p align="center">
  <a href="../i18n/ko/docs/00-overview/DASHBOARD.md">한국어</a> · <strong>English</strong> · <a href="../i18n/zh-CN/docs/00-overview/DASHBOARD.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# DASHBOARD — qhud

> Status: v0.7.1 released · Updated: 2026-09-14 · Owner: chquandogong
> Public project snapshot. Git history, tagged releases, and workflow runs are
> the authoritative records; dated observations below keep their original scope.

## State

| Item | Value |
| --- | --- |
| Current release | [v0.7.1](https://github.com/chquandogong/qhud/releases/tag/v0.7.1) — Intel GPU measurement on Linux, completing the conditional GPU support introduced by [v0.7.0](https://github.com/chquandogong/qhud/releases/tag/v0.7.0). Linux x86_64 tarball and Windows x86_64 ZIP. |
| Pipeline dependency | qmonster @ `6a21c44`; canonical Git dependency on Linux, command-scoped Windows patch with lockfile restoration. |
| Supported platforms | Ubuntu 24.04 / GNOME and native Windows x64 / WebView2. Native Windows terminal-tab observation remains outside the supported scope. |
| Current runtime evidence | **Ubuntu 2026-09-11, v0.7.1:** Intel Arc (Meteor Lake, `i915`, gt0+gt1) measured. `--system-dump` reported 24.8%; an independent rc6-residency calculation over the same interval returned 24.8%. The live widget rendered CPU, memory, GPU, disk and network history. v0.7.1 changes only the Linux GPU sampler, so the Windows runtime was not re-exercised for that patch. |
| Earlier cross-platform evidence | **Published v0.6.0 Linux asset, Ubuntu 2026-09-07:** checksum matched; GNOME/Wayland layer states, pixel liveness, eight herdr panes and all three provider refreshes passed. **Windows v0.6.0:** native display, account email, reset times, Codex refresh and app-server fallback passed. See TEST_PLAN for exact environment and hashes. |
| Quality evidence | **v0.7.1 local Ubuntu, 2026-09-11:** format clean, clippy `-D warnings` clean, 127/127 Rust tests, 6/6 Node tests, release build in 2 m 08 s. Earlier v0.6.0 Ubuntu and Windows CI passed 98/98 Rust tests and release builds: [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064). |
| Input verification rule | Linux interaction claims require compositor-path injection or a human hand; XTEST alone is inadmissible (D-010). |
| Frame-guard field evidence | Journal window 2026-08-26 → 09-07: 28 freezes, 28 remap heals, 0 re-execs, 0 operator-visible incidents (D-017). Earlier terminal-attached counts cannot be re-derived because their stderr never reached the journal. |
| Open field breadth | System metrics have direct v0.7.1 evidence on one Intel/i915 Linux host. AMD, NVIDIA, WDDM, a plain tmux server, a second live organization row, and several desktop lifecycle checks still need broader field evidence. Automated coverage remains the gate where hardware evidence is absent. |

## Decision index

D-001 second frontend · D-002 Tauri v2 · D-003 XWayland/EWMH layer ·
D-004 qmonster lib + NoopSink · D-005 demo=parity fixture · D-006
tarball releases · D-007 mux-backend factory (herdr) · D-008
self-driven geometry · D-009 pointerdown selection · D-010 input
verification protocol · D-011 scope-correct display · D-012 zoom + peek +
signal prohibition · D-013 local account identity · D-014 passive by default,
network on request · D-015 multi-account via per-account CLI config dirs ·
D-016 delegated fetch paths · D-017 frame guard · D-018 row identity is
(account, organization) · D-019 lenient wire numbers · D-020 scoped Windows
adaptation · D-021 typed and redacted ambient diagnostics.

Full entries and evidence are in [DECISION_LOG](../02-decisions/DECISION_LOG.md).

## Delivery record

| Delivery | Status |
| --- | --- |
| v0.1.0–v0.3.0: widget wedge, herdr/tmux observation, input architecture, zoom, peek and single instance | released 2026-08-05/06 |
| v0.4.0–v0.5.3: scope-correct quotas, identity, multiple accounts, persistence, delegated fetch paths, frame guard, organization identity and wire-drift repair | released 2026-08-07 → 09-04 |
| v0.6.0–v0.6.2: native Windows account widget, mux-less real-account view, ownership safeguards, portable packaging, rewritten core docs and automatic Codex email display | released 2026-09-07/08 |
| v0.7.0: bounded system history for CPU, memory, conditional GPU, disk and network; accessible About | released 2026-09-10 |
| v0.7.1: Intel i915/xe idle-residency GPU sampler and same-window field comparison | released and verified 2026-09-11 |

## Open verification and backlog

| Item | State |
| --- | --- |
| GNOME overview, lock/unlock, suspend/resume, monitor hotplug and fullscreen rows in TEST_PLAN | pending on-machine pass |
| Plain tmux live observation fallback | pending field pass |
| Second organization row from a live multi-organization login | pending field pass; keep credentials and machine paths outside the repository |
| Windows, AMD and NVIDIA system-metrics breadth | pending additional hardware evidence |
| Tile → terminal-pane focus jump | backlog |
| GNOME Shell extension for overview-clean pinning and desktop-icons coexistence | backlog |
| Upstream `ObserveSnapshot`, `.deb` packaging, agy multi-account | backlog |

## Evidence notes

v0.7.0 added the system strip and About without changing the account and pane
ownership rules. Sampling stays in the existing 2 s poll thread, retains at most
30 points in memory, pauses while the widget is hidden or minimized, and uses
gaps rather than invented zeros. v0.7.1 added Intel measurement on Linux; the
same-interval 24.8% comparison above validates one i915 host and does not imply
coverage of every adapter.

The 2026-09-07 published-asset pass closed the Ubuntu desktop gap left by the
Windows-hosted v0.6.0 release. It remains useful evidence for layer state,
painting, herdr observation and provider refreshes, while the v0.7.1 observation
is the current system-metrics evidence. Exact procedures and unresolved rows
live in [TEST_PLAN](../04-quality/TEST_PLAN.md); current requirements and
implementation boundaries live in [SPEC](../03-spec/SPEC.md) and
[ARCHITECTURE](../03-spec/ARCHITECTURE.md).
