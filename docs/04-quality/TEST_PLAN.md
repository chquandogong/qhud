# TEST_PLAN

> Status: living · Date: 2026-08-05 · Owner: chquandogong

## Automated gates (CI on every push/PR)

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets      # view.rs contract helpers
cargo build --release
```

## Unit coverage (v0.1)

- `view.rs`: percent clamping/rounding, byte humanization, `~` folding,
  label truncation. (Full `PaneReport` construction is upstream-private;
  mapping is covered by the live checklist below.)

## Manual verification checklist — desktop layer

Run on the target machine after any window-layer change:

| Check                 | Command / action                                                                       | Pass criteria (2026-08-05 evidence: ✅)                                                |
| --------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Below + sticky + skip | `xprop -id $(xdotool search --name '^qhud$' \| tail -1) _NET_WM_STATE _NET_WM_DESKTOP` | `_NET_WM_STATE_BELOW`, `_STICKY`, `_SKIP_TASKBAR`, `_SKIP_PAGER`; desktop `4294967295` |
| Stays under windows   | drag any app over the widget                                                           | widget never raises                                                                    |
| Workspace pinned      | switch workspaces                                                                      | widget visible on all                                                                  |
| Cross-monitor move    | drag topbar to the other monitor                                                       | position persists after restart                                                        |
| Resize                | drag ◢ grip                                                                            | content reflows; persists after restart                                                |
| Demo fallback         | stop tmux server                                                                       | `DEMO` badge within 2 s; tiles mirror the mockup                                       |
| Live recovery         | start tmux + an AI CLI                                                                 | live data within ≤12 s (10 s re-probe + 2 s poll), badge disappears                    |
| Visual parity         | compare with `docs/assets/widget-*.png`                                                | palette/tiles/gauges/pills match mockup                                                |
| GNOME overview        | open Activities / workspace gestures                                                   | ⏳ widget may appear as a window (accepted quirk R2); must return below afterwards     |
| Lock / unlock         | lock screen, unlock                                                                    | ⏳ still below + sticky (`xprop` re-check)                                             |
| Suspend / resume      | suspend, resume                                                                        | ⏳ same as lock/unlock                                                                 |
| Fullscreen app        | fullscreen a window on the widget's monitor                                            | ⏳ widget never bleeds through                                                         |
| Monitor hotplug       | unplug/replug the external monitor                                                     | ⏳ recoverable via tray → Reset position                                               |

⏳ rows were added from the Codex cross-validation (CV log) and are
pending their first on-machine verification pass.

2026-08-05 (D-008): click delivery, drag-move, and grip-resize
re-verified pixel-exact on both monitors with the self-driven geometry
implementation (synthetic-input evidence in DECISION_LOG D-008).

## Input-verification protocol (mandatory since D-010)

**XTEST (xdotool) alone is inadmissible for interaction claims** — it
injects inside XWayland and bypasses Mutter's surface picking, which
is exactly where real input was being stolen (D-010). Any "interaction
works" claim requires **compositor-path injection**: Mutter
RemoteDesktop absolute-pointer clicks (see the `rd_abs_click.py`
technique recorded in D-010) or a human hand, confirmed via the
`qhud ui:` stderr breadcrumbs.

2026-08-06 (D-010, v0.1.4): compositor-path selection toggle
round-trip verified (`sel:none:-` → `sel:wC:p3:R`) with DING disabled;
identical clicks vanish with DING enabled (A/B).

## Non-regression invariants

- No file under `~/.qmonster` is created/modified by qhud (R5):
  `find ~/.qmonster -newer /tmp/mark` after a 10-minute run ⇒ only
  TUI-attributable files.
- Widget never takes keyboard focus on click (pointer-only contract).
