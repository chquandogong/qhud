# Changelog

All notable changes to qhud. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [0.3.0] — 2026-08-06

### Added

- **Font zoom**: Ctrl+wheel over the widget scales the whole UI
  (70–160%), persisted across restarts (D-012).
- **Layer peek**: tray "Pin above windows" check item and
  `qhud --peek` (relayed to the running instance) flip the widget
  above all windows and back — bind a GNOME custom shortcut to
  `~/.local/bin/qhud --peek`. Footer shows `pinned ·` while above.
- **Single-instance guard**: launching qhud twice no longer stacks a
  second widget (tauri-plugin-single-instance).

### Fixed

- First peek design (SIGUSR1) segfaulted the webview — WebKitGTK's
  JavaScriptCore reserves SIGUSR1 for thread suspension; signals are
  now documented as off-limits in this codebase (D-012).

## [0.2.0] — 2026-08-06

### Changed

- **Information architecture rebuilt around fact scope** (D-011, from
  operator feedback): 5h/7d quota windows are account facts and now
  render once per provider in a quota strip (freshest-snapshot rollup
  — max percent wins, since usage within a window only grows). Tiles
  show pane facts only (status + CTX + expanded detail) and gain a
  `@workspace` badge so identical labels are distinguishable. Top-bar
  summary simplified (quota lives in the strip).
- Payload schema v1 additive fields: `quotas[]`, `panes[].session`.
- README screenshots regenerated for the new layout.

## [0.1.4] — 2026-08-06

### Fixed

- **Real-mouse interaction was still dead on Ubuntu GNOME** while every
  synthetic test passed: Ubuntu's Desktop Icons NG (DING) extension
  window swallows real pointer input over the desktop layer, and
  XTEST-based verification bypasses Mutter's surface picking entirely
  — a methodology blind spot, now closed with compositor-path
  injection via the Mutter RemoteDesktop API (D-010). A/B proven:
  identical compositor-path clicks toggle selection with DING
  disabled and vanish with it enabled. Reference machine runs with
  DING disabled (its `~/Desktop` is empty); icon users: see RUNBOOK
  and the companion-extension backlog item.

### Added

- Permanent stderr interaction breadcrumbs (`qhud ui: …` via a
  `ui_event` command) — real-input behavior is now verifiable from
  logs on any machine.

### Changed

- `core:window:allow-set-title` capability removed again (debug
  beacons replaced by breadcrumbs).

## [0.1.3] — 2026-08-06

### Fixed

- **Tile selection (click-to-expand) never fired** (D-009): this
  WebKitGTK/X11 webview reliably delivers `pointerdown` but fails to
  synthesize `click` from the down/up pair, so the selection handler
  simply never ran. Selection now triggers on `pointerdown`
  (beacon-verified end-to-end: select → deselect → reselect with
  rendered state applied). localStorage was investigated and
  exonerated — writes work; the storage directory was missing only
  because the click handler had never executed. Persistence is now
  try/catch-wrapped and ordered after `render()` regardless, so
  storage can never gate the UI.

## [0.1.2] — 2026-08-05

### Fixed

- **Move, resize, and tile selection were dead in practice** (D-008).
  Four independent upstream quirks, each verified on-machine:
  compositor-side interactive move/resize (drag regions,
  `startResizeDragging`) no-ops for a keep-below XWayland window on
  GNOME; tao's invisible borderless-resize inset ate all pointer input
  within ~10px of the border — where the resize grip lived; WebKitGTK
  `screenX/Y` goes stale while the window moves; tao
  `outerPosition/outerSize` report a phantom ~37px frame. Geometry
  interaction is now fully self-driven: an rAF loop over the global
  `cursorPosition()` with DOM-derived grab metrics. Verified
  pixel-exact on both monitors.
- **Stale UI assets in rebuilds**: `tauri::generate_context!` does not
  register `../ui` with cargo's change tracking; `build.rs` now emits
  `cargo:rerun-if-changed=../ui`.
- **Window position/size lost on logout/kill**: the window-state
  plugin only persists on graceful exit; geometry is now checkpointed
  every 30 s from the poll loop.

### Changed

- Resize grip enlarged with an inner hit-zone that clears the tao edge
  inset; capabilities trimmed to exactly `cursor-position`,
  `set-position`, `set-size`, `window-state:default`.

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
