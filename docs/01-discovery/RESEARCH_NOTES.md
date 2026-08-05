# RESEARCH_NOTES

> Status: done · Date: 2026-08-05 · Owner: chquandogong

## Window layering on Linux desktops

- GNOME Mutter has an **open, unimplemented** request for the layer-shell
  protocol — third-party apps cannot use a native Wayland widget layer on
  GNOME: <https://gitlab.gnome.org/GNOME/mutter/-/work_items/973>
- `wlr-layer-shell` protocol (implemented by wlroots, KWin, Smithay —
  everyone but GNOME): <https://wayland.app/protocols/wlr-layer-shell-unstable-v1>
- Consequence: on GNOME the desktop-widget layer is reachable only via
  XWayland EWMH states (`_NET_WM_STATE_BELOW` + `_NET_WM_STATE_STICKY`),
  which Mutter honors (verified live — FEASIBILITY_REPORT).

## Tauri capabilities

- `always_on_bottom` window option + `set_always_on_bottom`:
  <https://github.com/tauri-apps/tauri/commit/c1ec0f155118527361dd5645d920becbc8afd569>
- Underlying tao implementation (GTK `set_keep_below` on Linux):
  <https://github.com/tauri-apps/tao/pull/522>
- Window position/order management **silently no-ops on the Wayland
  backend** — the reason qhud forces `GDK_BACKEND=x11`:
  <https://github.com/tauri-apps/tauri/issues/14913>

## qmonster reuse surface (rev aa2bd39)

- `qmonster::app::bootstrap::Context::new(config, source, notifier, sink)`
  — public constructor, generic over `PaneSource` + `NotifyBackend`.
- `qmonster::store::sink::NoopSink` — upstream already ships the
  no-write audit sink qhud needs.
- `qmonster::app::event_loop::run_once_with_target` — one observe tick,
  returns `Vec<PaneReport>`; qhud maps it to widget JSON (schema v1).
- `build.rs` falls back to `v{CARGO_PKG_VERSION}-nogit` when `.git` is
  absent — safe to consume as a cargo git dependency.
- Gauge semantics: pressures are 0..1 fractions
  (`context_pressure`, `quota_5h_pressure`, `quota_weekly_pressure`);
  reset instants are unix seconds (`quota_*_resets_at`).
