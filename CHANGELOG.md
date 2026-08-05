# Changelog

All notable changes to qhud. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [0.1.1] — 2026-08-05

### Fixed

- **herdr rigs never went live**: v0.1.0 hardcoded the tmux polling
  source. qhud now builds its pane source through qmonster's own
  `build_tmux_source` factory, so `[mux] backend` (`auto` / `tmux` /
  `herdr`) means the same thing in both frontends (D-007).

### Added

- Widget-flavored `auto`: when qhud runs outside any mux pane (no
  herdr env inherited), it probes herdr first, then falls back to
  tmux.
- Payload schema v1 additive field `backend` ("herdr" | "tmux"),
  rendered in the footer (`live·herdr · poll 2s · …`).
- Live/demo transition logs on stderr with resolved backend and pane
  labels — greppable evidence for the TEST_PLAN checklist.

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
