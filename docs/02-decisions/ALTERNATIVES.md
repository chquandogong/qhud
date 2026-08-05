# ALTERNATIVES — desktop-widget implementation

> Status: done · Date: 2026-08-05 · Owner: chquandogong
> Chosen: **A (Tauri v2)** — see DECISION_LOG D-002.

| #     | Option                                         | UI parity with mockup           | qmonster integration                | Always-on cost                              | Desktop layer on GNOME Wayland                                         | Fails when                              |
| ----- | ---------------------------------------------- | ------------------------------- | ----------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------- | --------------------------------------- |
| **A** | **Tauri v2** (Rust + WebKitGTK)                | ★★★ HTML ports verbatim         | ★★★ direct lib link (same language) | ~15 MB bin, moderate RSS                    | `alwaysOnBottom` via GTK keep-below under XWayland — **live-verified** | WebKitGTK transparency defects          |
| B     | Electron                                       | ★★★ HTML ports verbatim         | ★☆ subprocess/IPC only              | ~250 MB RSS — heavy for an always-on widget | `type:'desktop'` / below hints, mature ecosystem                       | resource cost unacceptable              |
| C     | GNOME Shell extension (GJS/Clutter)            | ★☆ no webview — full UI rewrite | ★★ file/JSON handoff                | minimal                                     | **only true native layer** (clean in overview too)                     | GNOME API churn per release; GNOME-only |
| D     | Native Rust GUI (GTK4-rs / egui / iced)        | ★☆ CSS re-implementation        | ★★★ direct lib link                 | minimal                                     | GTK4-rs keep-below OK; winit-based (egui/iced) has no keep-below       | UI rewrite cost explodes                |
| E     | Pin the existing TUI in a transparent terminal | ☆ TUI, not the mockup           | already done                        | minimal                                     | same EWMH tricks via wmctrl/xdotool                                    | it's a validation hack, not a product   |

Notes

- E was effectively executed as the pre-code spike (same EWMH
  mechanics) and retired.
- C remains the long-term answer to the two accepted GNOME quirks
  (overview exposure, XWayland dependence) if they ever matter enough.
- B remains plan-B if WebKitGTK rendering defects prove unfixable.
