# Changelog

All notable changes to qhud. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [0.1.0] — 2026-08-05

First release — the smallest useful wedge.

### Added

- Desktop-layer widget window: keep-below + sticky + skip-taskbar via
  XWayland/EWMH (GNOME Wayland verified), frameless, transparent,
  drag-move, grip-resize, geometry persisted across restarts
  (`tauri-plugin-window-state`).
- qmonster observe bridge: links the qmonster crate at a pinned rev,
  polls `run_once_with_target` every 2 s with a no-write `NoopSink` +
  `SilentNotify`, shares `~/.qmonster/config/qmonster.toml` read-only.
- Widget UI ported from the reference mockup: pane tiles with status
  pills (active / done / wait / limit / stale / dead), CTX · 5H · 7D
  severity-banded gauges with reset countdowns, click-to-expand config
  chips (model / effort / flags / branch / cwd / mem / cost) and
  cross-pane conflict banner.
- Demo mode: mockup-parity payload with `DEMO` badge when no tmux
  server is reachable; live re-probe every 10 s.
- Tray icon (Show/Hide · Quit), best-effort.
- JSON payload contract schema v1 (`view.rs`) with unit tests.
- CI (fmt · clippy · test · build) and tag-driven release workflow
  (Linux x86_64 tarball + sha256 + build provenance attestation).
- Quetzalcoatl decision-doc set under `docs/`.
