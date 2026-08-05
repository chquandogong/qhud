# ASSUMPTIONS

> Status: living · Date: 2026-08-05 · Owner: chquandogong

| #   | Assumption                                                                           | Status             | Evidence / revisit trigger                                                                       |
| --- | ------------------------------------------------------------------------------------ | ------------------ | ------------------------------------------------------------------------------------------------ |
| A1  | Primary target is one GNOME Wayland workstation; other DEs are best-effort           | accepted           | Revisit if issues arrive from KDE/wlroots users (layer-shell path would then be worth adding)    |
| A2  | Mutter honors `_NET_WM_STATE_BELOW`/`STICKY` for XWayland windows                    | **verified**       | FEASIBILITY_REPORT spikes 1 & 2                                                                  |
| A3  | Both monitors run scale=1, so XWayland fractional-scaling blur does not apply        | verified today     | Revisit if the operator enables fractional scaling (GNOME 46 blurs XWayland; GNOME 47+ improves) |
| A4  | The qmonster TUI owns `~/.qmonster` writes; a second writer would race sqlite        | accepted by design | qhud uses `NoopSink` and never persists — see DECISION_LOG D-004                                 |
| A5  | qmonster's lib API at rev `aa2bd39` is the contract; upstream may drift              | pinned             | Cargo `rev=` pin; bump deliberately and re-run `cargo test`                                      |
| A6  | Alerting stays in the TUI/providers; a widget that notifies would double-fire        | accepted           | `SilentNotify` backend                                                                           |
| A7  | Demo payload mirroring the mockup is an acceptable stand-in when tmux is down        | accepted           | Also serves as the visual-parity fixture                                                         |
| A8  | `visibleOnAllWorkspaces` + `alwaysOnBottom` in tauri.conf are honored on X11 backend | verified           | Spike 2 `_NET_WM_DESKTOP=0xFFFFFFFF`                                                             |
