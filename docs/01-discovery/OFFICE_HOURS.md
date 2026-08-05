# OFFICE_HOURS — pressure review result

> Status: done · Date: 2026-08-05 · Owner: chquandogong

## 1. Problem sharpened

- **Pain**: quota/context pressure is invisible while the operator is
  focused on actual work in other windows. The qmonster TUI answers
  every question, but only when you look _at_ it — inside a tmux pane.
- **Recent instance**: 5h window at 88% discovered only when a CLI
  refused work; the reset countdown was in a pane behind the editor.
- **Frequency**: continuous during any multi-agent session.
- **Workaround today**: keep one monitor's corner dedicated to a
  terminal running the TUI — costs a terminal, steals alt-tab focus,
  disappears under maximized windows.

## 2. Narrowest customer

The qmonster author on a 2-monitor GNOME Wayland workstation; then
qmonster's existing operators (Linux, tmux, multi-CLI).

## 3. Why now

- qmonster v3.2.0 already ships the entire data pipeline as a Rust
  library (`lib.rs` exports adapters/domain/policy/tmux) — the widget
  is _only_ a presentation layer.
- The mockup ("Qmonster · AI CLI 모니터") is pure HTML/CSS — a webview
  shell reproduces it with near-zero translation cost.
- Verified on 2026-08-05 that GNOME Wayland accepts the XWayland
  keep-below + sticky combination (live spike, see FEASIBILITY_REPORT).

## 4. Smallest useful wedge

A read-only widget that shows pane tiles + CTX/5H/7D gauges from the
existing qmonster pipeline, movable/resizable/persistent, demo fallback
when tmux is absent. No alerts, no actions, no settings UI.

## 5. Framing shake — "exactly the same UI/UX"

The mockup is a 960px document-shaped page and includes mockup-only
explainer elements (mode note, legend). Copying it verbatim would make
a bad widget. **Reinterpretation adopted**: keep the visual language
100% (palette, tiles, gauges, status pills, severity bands), re-set it
at widget scale (~360–420px), translate ↑/↓ selection to click-to-
expand, drop explainer chrome. A widget must never steal keyboard
focus — pointer-only interaction.

## 6. 10-star sketch (later)

Click a tile → jump to that tmux pane. Critical quota pulses at the
edge of vision. Wallpaper-aware theming. Multi-widget layouts
(per-monitor). Cross-platform (Windows WorkerW / macOS desktop level).

## Verdict

Proceed. Wedge is one week of work; the riskiest unknown (desktop
layer on GNOME Wayland) was de-risked by live spike before any code.
