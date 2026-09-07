# TEST_PLAN

> Status: v0.6.0 automated gates passed; desktop gaps recorded · Date: 2026-09-07 · Owner: chquandogong

## Automated gates

CI runs on pushes to `main` and `codex/**`, and on pull requests. Ubuntu
24.04 uses the canonical pinned Git dependency and runs:

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

Windows MSVC uses the scoped dependency patch through the build script:

```powershell
./scripts/Build-Windows.ps1 -Test -VisualStudioPath '<Visual Studio installation>'
git diff --exit-code -- Cargo.toml Cargo.lock
```

`-Test` runs QHUD tests and then builds in the same dependency context.
The script restores the canonical lockfile after Windows commands; Linux
builds do not require a sibling checkout or the Windows patch. Release
publication waits for both Ubuntu and Windows tests and builds to succeed.

2026-09-07: [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064)
passed on `1514e09`: Ubuntu fmt/clippy, **98/98 tests** and release build;
Windows **98/98 tests**, release build and canonical dependency-file check.
The local Windows script also passed 98/98 tests and the release build.

2026-09-07, Ubuntu re-verification of the published v0.6.0 Linux asset —
the pass v0.6.0 recorded as missing. The release tarball was downloaded,
its SHA-256 matched its sidecar, and that exact binary (installed to
`~/.local/bin/qhud`, hash-compared against the asset) was relaunched on
the reference machine. Environment: Ubuntu 24.04.3, GNOME Shell 46.0
Wayland, kernel 7.0.0-28, eDP-1 2560×1600 + HDMI-1 3840×2160 both at
scale 1, webkit2gtk 2.52.6, herdr 0.7.5, rustc 1.94.1. The same tree
also passed locally: fmt clean, clippy `-D warnings` clean, 98/98 tests,
and a release build in 6 m 21 s from the pinned Git dependency with no
sibling checkout.

## Unit coverage

- `view.rs`: percent clamping/rounding, byte humanization, `~` folding,
  label truncation. (Full `PaneReport` construction is upstream-private;
  mapping is covered by the live checklist below.)
- Saved quota ownership after an account switch; local account rows with
  no running mux; scoped-only and extra-only snapshots after restart.
- Codex model pools preserve the model name, 5H/7D duration, usage and
  reset through both provider parsers and the account-row merge. Fixtures
  do not establish that a live account currently has those pools.

## Manual verification checklist — Ubuntu desktop layer

Run on the target machine after any window-layer change:

| Check                 | Command / action                                                                       | Pass criteria — evidence date                                                           |
| --------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Below + sticky + skip | `xprop -id $(xdotool search --class qhud \| tail -1) _NET_WM_STATE _NET_WM_DESKTOP` | all four states present; desktop `4294967295` — **passed 2026-09-07 on the published v0.6.0 asset** |
| Stays under windows   | drag any app over the widget                                                           | widget never raises                                                                    |
| Workspace pinned      | switch workspaces                                                                      | widget visible on all                                                                  |
| Cross-monitor move    | drag topbar to the other monitor                                                       | position persists after restart                                                        |
| Resize                | drag ◢ grip                                                                            | content reflows; persists after restart                                                |
| Local fallback        | stop the active tmux/herdr server                                                       | real accounts and dated quota remain; no example panes or `DEMO` badge                 |
| Real data by default  | `qhud --dump` and `qhud --demo --dump`                                                  | without the flag `source` is `live`/`local` with real panes; with it `source` is `demo` with the three mockup panes — **passed 2026-09-07** |
| Explicit demo         | launch `qhud --demo` after closing the existing instance                               | `DEMO` badge; tiles mirror the mockup                                                  |
| Live recovery         | start tmux + an AI CLI                                                                 | live data within ≤12 s (10 s re-probe + 2 s poll) — 2026-08-05                          |
| Live observation      | read the startup line in the widget's stderr                                            | `live via herdr` with the pane list — **passed 2026-09-07** (8 panes, strip built 3 sections / 7 rows) |
| Every provider refreshes | `qhud --refresh-all`, then read the widget's stderr                                  | all three providers answer with no error — **passed 2026-09-07** (claude 5h 3% / 7d 67% / 3 scoped, agy 2 pools, codex 1 workspace) |
| Pixels are painting   | `xwd -id <window> \| md5sum` twice, a few seconds apart                                 | the two hashes **differ** (the footer clock repaints every second) — **passed 2026-09-07**; identical hashes mean a frozen frame (D-017) |
| Frame guard is armed  | grep the widget's stderr for `frame guard armed`                                        | exactly one line per process start — **passed 2026-09-07**; its absence means the sampler failed and a freeze would go undetected |
| Visual parity         | compare with `docs/assets/widget-*.png`                                                | palette/tiles/gauges/pills match mockup                                                |
| GNOME overview        | open Activities / workspace gestures                                                   | ⏳ widget may appear as a window (accepted quirk R2); must return below afterwards     |
| Lock / unlock         | lock screen, unlock                                                                    | ⏳ still below + sticky (`xprop` re-check)                                             |
| Suspend / resume      | suspend, resume                                                                        | ⏳ same as lock/unlock                                                                 |
| Fullscreen app        | fullscreen a window on the widget's monitor                                            | ⏳ widget never bleeds through                                                         |
| Monitor hotplug       | unplug/replug the external monitor                                                     | ⏳ recoverable via tray → Reset position                                               |

⏳ rows were added from the Codex cross-validation (CV log) and are
pending their first on-machine verification pass. Two of them —
lock/unlock and suspend/resume — are exactly the display-sleep
conditions that trigger the D-017 freeze, which was found in the field
rather than by this checklist. Until they are run, the frame guard's
field tally is the only evidence for that path: **28 detections, 28
first-rung heals, 0 re-execs, 0 operator-visible incidents** over the
journal-captured window 2026-08-26 → 09-07.

2026-08-05 (D-008): click delivery, drag-move, and grip-resize
re-verified pixel-exact on both monitors with the self-driven geometry
implementation (synthetic-input evidence in DECISION_LOG D-008).

## Manual verification checklist — Windows and account usage

- Start the native WebView2 app, move and resize it, and verify the tray,
  existing-instance `--peek`, and explicit refresh controls.
- With no tmux/herdr backend, confirm zero panes and real account rows.
  Refresh results remain dated after restart; no mock usage is substituted.
- Shrink the window or use multiple account/model rows. Every quota and
  reset remains reachable by scrolling; pane and footer controls remain.
- Confirm readable model names and reset countdowns. Server-absent model
  limits remain absent; no zero-filled values or another account's usage.
- Verify that background CLI probes do not open console windows. Native
  Windows terminal-tab monitoring is outside the current supported scope.

Local Windows v0.6.0 display, account email, per-model/reset rows, explicit
Codex refresh and the actual app-server fallback were verified after install.
Installed executable SHA-256:
`3D688DB2F90E6D4780C0F016CB0CED1572189AC83FC1658F73A6688D1477F9FF`.
This identifies the local MSVC build, not the separate GitHub release artifact.

The final tag `v0.6.0` (`cfdd850`) passed
[main CI](https://github.com/chquandogong/qhud/actions/runs/34127575130) and
[Release](https://github.com/chquandogong/qhud/actions/runs/34127575141).
Both published archives were downloaded and matched their SHA-256 files.
The published Windows executable was installed; its SHA-256 is
`1233E4E7D58B8E6A0C855D3B528C7810FFD99DA476509F9C19509A3B5F094917`.
The installed published process responds and emits its account rows. Its final
compositor screenshot is pending because the Windows session entered the lock
screen; the earlier local-build visual check remains the direct pixel evidence.

## Ubuntu input-verification protocol (mandatory since D-010)

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
