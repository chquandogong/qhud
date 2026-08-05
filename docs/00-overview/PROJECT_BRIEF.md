# PROJECT_BRIEF — qhud

> Status: active · Date: 2026-08-05 · Owner: chquandogong

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

qhud is a **second frontend for qmonster**, not a fork: it links the
qmonster crate directly and reuses its tmux observation, provider
parsing, and policy pipeline wholesale.

## First user

The qmonster author's own multi-monitor Ubuntu GNOME workstation.
Secondary: any qmonster operator on Linux/X11 or GNOME Wayland.

## Success criteria (v0.1)

1. Widget stays on the desktop layer (keep-below + sticky) on GNOME
   Wayland via XWayland — verified with `_NET_WM_STATE`.
2. Movable anywhere across monitors, resizable, position persisted.
3. Visual parity with the design mockup (demo payload doubles as the
   parity fixture).
4. Live tmux data when available; graceful demo fallback otherwise.
5. Observe-only: **zero writes** to `~/.qmonster` (the TUI owns it).

## Non-goals (v0.1)

- No actuation, no notifications (the TUI and providers own alerting).
- No Windows/macOS support (window-layer code is isolated for later).
- No npm/deb/AppImage packaging — plain binary tarball only.

## Documents

Quetzalcoatl-style decision docs live under `docs/`:
discovery (`01-discovery/`), decisions (`02-decisions/`), spec
(`03-spec/`), quality (`04-quality/`), ops (`05-ops/`).
