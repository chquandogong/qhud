# FEASIBILITY_REPORT

> Status: done · Date: 2026-08-05 · Owner: chquandogong

## Environment under test

| Fact     | Value                                                                 |
| -------- | --------------------------------------------------------------------- |
| OS       | Ubuntu 24.04.3 LTS                                                    |
| Desktop  | GNOME 46, **Wayland** session                                         |
| Monitors | HDMI-1 3840×2160 @ (0,0) · eDP-1 2560×1600 @ (3840,560), both scale=1 |
| Rust     | 1.94.1 (qmonster requires 1.88+)                                      |

## The core constraint

GNOME Mutter does **not** implement `wlr-layer-shell`, the Wayland
protocol every other major compositor uses for desktop widgets. Native
Wayland clients also cannot position their own windows globally. Both
capabilities exist for **XWayland** clients via EWMH hints, which
Mutter honors.

## Spike 1 — GTK keep-below (2026-08-05, pre-code)

A GJS/GTK3 window (`set_keep_below` + `stick` + skip hints — the same
calls Tauri's `tao` uses on Linux) under `GDK_BACKEND=x11`:

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                      _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY, _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 4294967295   # all workspaces
move  (300,300) → (4200,800)             # cross-monitor: OK
resize 320×180 → 480×300                 # programmatic resize: OK
```

## Spike 2 — the real qhud binary (2026-08-05, post-build)

`target/release/qhud` (Tauri v2, `alwaysOnBottom` + runtime asserts):

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                      _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY, _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 4294967295
WM_CLASS(STRING) = "qhud", "Qhud"
move (50,50) → (4300,700) → (3400,60)    # both monitors: OK
binary size: 15 MB (release, LTO, stripped)
```

## Scorecard

| Axis         | Score (1–5) | Note                                                                     |
| ------------ | ----------- | ------------------------------------------------------------------------ |
| Desirability | 4           | Solves the author's daily glanceability gap                              |
| Feasibility  | 5           | Both spikes green on the exact target machine                            |
| Viability    | 5           | Data layer reused from qmonster; ~1 week wedge                           |
| Risk         | 4           | Residual: WebKitGTK rendering quirks, GNOME overview exposure (accepted) |

## Decision Gate 1

**Proceed to build.** Alternatives and the full rationale live in
`../02-decisions/ALTERNATIVES.md` and `DECISION_LOG.md`.
