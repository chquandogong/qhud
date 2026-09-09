<!-- qhud:languages -->
<p align="center">
  <a href="../i18n/ko/docs/00-overview/PROJECT_BRIEF.md">한국어</a> · <strong>English</strong> · <a href="../i18n/zh-CN/docs/00-overview/PROJECT_BRIEF.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# PROJECT_BRIEF — qhud

> Status: v0.6.0 release preparation · Date: 2026-09-07 · Owner: chquandogong

## Problem

Multi-agent tmux work (Claude Code, Codex, Gemini, Antigravity side by
side) produces operational pressure — context fill, 5h/7d quota burn,
panes silently waiting for approval — that the operator only sees by
switching _into_ the [qmonster](https://github.com/chquandogong/qmonster)
TUI pane. While you are focused on the work itself, the meters are out
of sight. Quota surprises ("5h hit 100% mid-task") are exactly the
failures that happen while you are _not_ looking at the monitor.

## Solution

qhud is an **ambient desktop HUD**: a small always-on-desktop widget
(below every window, above the wallpaper, sticky across workspaces)
that renders qmonster's pane tiles and CTX / 5H / 7D gauges on a spare
corner of a monitor. Glanceability like a wall clock — zero focus
cost, zero pane switching.

qhud links the qmonster crate and reuses its tmux/herdr observation,
provider parsing, and policy pipeline. Account quota also remains useful
with no running terminal: local identities and dated saved readings stay
visible, and an explicit refresh requests current usage.

## First user

The qmonster author's own multi-monitor Ubuntu GNOME workstation.
Secondary: any qmonster operator on Linux/X11 or GNOME Wayland.
v0.6.0 adds a native Windows x64 WebView2 widget for account usage and
reset visibility. Native Windows terminal-tab monitoring is not included.

## Success criteria (v0.6.0)

1. Widget stays on the desktop layer (keep-below + sticky) on GNOME
   Wayland via XWayland — verified with `_NET_WM_STATE`.
2. Movable anywhere across monitors, resizable, position persisted.
3. Visual parity with the design mockup (demo payload doubles as the
   parity fixture).
4. Live tmux/herdr data when available; real local account rows and dated
   quota otherwise. Sample panes appear only with `--demo`.
5. Observe-only: **zero writes** to `~/.qmonster` (the TUI owns it).
6. Native Windows account usage, model-specific pools when the provider
   returns them, readable reset times, and persistence after restart.
7. Retain the Ubuntu source-build path and Linux tarball release. Publish
   a Windows ZIP only after both platforms pass their release gates.

## Non-goals (v0.6.0)

- No actuation, no notifications (the TUI and providers own alerting).
- No native Windows terminal-tab observation or macOS support.
- No npm/deb/AppImage/MSI packaging — Linux tarball and Windows ZIP.
- No invented quotas for model pools that a provider does not return.

## Current verification

On 2026-09-07 both remote Ubuntu and Windows CI jobs passed 98/98 QHUD
tests and release builds; Ubuntu fmt/clippy also passed. The local Windows
build passed the native display/account-refresh smoke check. See
[CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064)
and TEST_PLAN for evidence and the remaining Ubuntu desktop checks.

## Documents

Quetzalcoatl-style decision docs live under `docs/`:
discovery (`01-discovery/`), decisions (`02-decisions/`), spec
(`03-spec/`), quality (`04-quality/`), ops (`05-ops/`).
