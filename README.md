<p align="center">
  <img src="docs/assets/qhud-banner.svg" alt="qhud — ambient desktop HUD for AI CLI sessions" width="100%">
</p>

<p align="center">
  <a href="https://github.com/chquandogong/qhud/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/chquandogong/qhud/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/chquandogong/qhud/releases"><img alt="GitHub release" src="https://img.shields.io/github/v/release/chquandogong/qhud?display_name=tag&sort=semver"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/github/license/chquandogong/qhud"></a>
  <img alt="Rust 1.88+" src="https://img.shields.io/badge/Rust-1.88%2B-b7410e?logo=rust">
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white">
  <a href="https://github.com/chquandogong/qmonster"><img alt="qmonster family" src="https://img.shields.io/badge/family-qmonster-46b8b0"></a>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a>
  · <a href="#controls">Controls</a>
  · <a href="#how-it-stays-on-the-desktop-layer">How It Works</a>
  · <a href="#known-limits">Known Limits</a>
  · <a href="https://github.com/chquandogong/qhud/releases">Releases</a>
  · <a href="#documentation">Docs</a>
</p>

qhud pins a small, glanceable widget to your **desktop background
layer** — below every window, above the wallpaper, on every workspace
— showing the health of the AI CLIs running in your tmux/herdr panes.
It is a second frontend for
[qmonster](https://github.com/chquandogong/qmonster): the same observe
pipeline, zero terminal footprint. Park it on a spare corner of a
monitor and glance, like a wall clock for quota pressure.

<p align="center">
  <img src="docs/assets/widget-compact.png" width="330" alt="qhud compact — provider quota strip and per-pane status + CTX tiles">
  &nbsp;&nbsp;
  <img src="docs/assets/widget-expanded.png" width="330" alt="qhud expanded — selected pane shows model/effort/flags/cwd config and a cross-pane conflict banner">
</p>

## Why

Quota pressure bites precisely when you are _not_ looking at the
monitor pane. A TUI answers questions when you visit it; a HUD answers
them while you work. One glance gives you: **which account is close to
its 5h/7d limit and when it resets · which pane is waiting for
approval · whose context window is filling · who is editing the same
file.**

## Features

- **True desktop-layer widget** — keep-below + sticky + skip-taskbar,
  verified against GNOME Mutter (Wayland session, via XWayland).
- **Scope-correct display** — facts render at the scope they are true:
  a per-provider **5H/7D quota strip** (account facts, freshest
  snapshot wins) up top; **status pill + CTX gauge** per pane tile with
  an `@workspace` badge; click a tile for model/effort/flags/cwd/cost
  and cross-pane conflict detail.
- **The qmonster pipeline, unmodified** — links the crate directly:
  identity resolution, provider parsing (Claude / Codex / Gemini /
  Antigravity), tmux **and herdr** backends, cross-pane conflict
  findings — all upstream.
- **Observe-only by contract** — no writes to `~/.qmonster`, no
  notifications, no network. The TUI stays the single writer.
- **Layer peek** — `Ctrl+Alt+Q` (or tray → _Pin above windows_, or
  `qhud --peek`) flips the widget above everything for a look, then
  back to the wallpaper layer.
- **Zoom, drag, resize, remembered** — Ctrl+wheel scales the UI
  70–160%; position, size, and zoom persist across restarts and
  reboots. Single-instance guarded.
- **Demo mode** — no mux server? The widget renders reference data
  with a `DEMO` badge and re-probes for live panes every 10 s.
- **15 MB single binary** — Tauri 2, static frontend, no bundler, no
  node_modules.

## Quick start

```bash
# release binary
gh release download --repo chquandogong/qhud --pattern '*linux-x86_64.tar.gz'
tar -xzf qhud-v*-linux-x86_64.tar.gz && ./qhud-v*/qhud &

# or from source
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
git clone https://github.com/chquandogong/qhud && cd qhud
cargo build --release && ./target/release/qhud &
```

Have qmonster set up already? qhud reads the same
`~/.qmonster/config/qmonster.toml` (read-only) and observes the same
panes — `[mux] backend` (`auto`/`tmux`/`herdr`) means the same thing in
both frontends. No config? Defaults apply.

Autostart, app-grid entry, and the peek keyboard shortcut: see the
[RUNBOOK](docs/05-ops/RUNBOOK.md).

## Controls

| Action                | How                                                                    |
| --------------------- | ---------------------------------------------------------------------- |
| Move                  | drag the top or bottom bar                                             |
| Resize                | drag the ◢ grip (bottom-right)                                         |
| Expand a pane         | click its tile; click again to collapse                                |
| Font size             | **Ctrl + wheel** over the widget (70–160%, remembered)                 |
| Peek above windows    | tray → _Pin above windows_, or `qhud --peek`, or bind a shortcut to it |
| Hide / recover / quit | tray → _Show/Hide_ · _Reset position_ · _Quit qhud_                    |

The tray icon is the small light gauge glyph at the **right end of the
GNOME top bar**. Do not send Unix signals to qhud — WebKitGTK reserves
them (see D-012).

## How it stays on the desktop layer

GNOME's Wayland compositor has no widget-layer protocol for
third-party apps (`wlr-layer-shell` is
[unimplemented in Mutter](https://gitlab.gnome.org/GNOME/mutter/-/work_items/973)),
and native Wayland windows cannot even position themselves globally.
qhud therefore forces the X11 backend (XWayland) and uses the EWMH
states Mutter _does_ honor — the same mechanism Conky-style widgets
have used for years:

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                      _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY
_NET_WM_DESKTOP(CARDINAL) = 4294967295        # every workspace
```

Because compositor-side interactive move/resize is unreliable for
keep-below XWayland windows, qhud drives its own geometry from global
cursor position — the full story (and four upstream quirks it works
around) is in [D-008](docs/02-decisions/DECISION_LOG.md). On X11
sessions this all works natively; on wlroots/KDE you may prefer
`QHUD_NO_X11_FORCE=1` and your compositor's own layering rules.

## Known limits

- **Ubuntu's Desktop Icons NG (DING) extension swallows real mouse
  input over the desktop layer** — if the widget ignores clicks,
  `gnome-extensions disable ding@rastersoft.com`
  ([D-010](docs/02-decisions/DECISION_LOG.md); a companion-extension
  coexistence path is on the backlog).
- One account per provider is assumed for the quota strip (provider
  surfaces don't expose account identity).
- The widget appears as a window in the GNOME overview (accepted
  XWayland quirk).
- Linux x86_64 only today.

## Relationship to qmonster

|         | qmonster                                                        | qhud                                  |
| ------- | --------------------------------------------------------------- | ------------------------------------- |
| Surface | ratatui TUI in a tmux pane                                      | desktop widget on the wallpaper layer |
| Role    | operating console: alerts, recommendations, settings, snapshots | ambient meters: glanceability only    |
| Writes  | owns `~/.qmonster` (sqlite audit, archives)                     | **none** (NoopSink)                   |
| Data    | qmonster observe pipeline                                       | same crate, pinned rev                |

## Documentation

Decision-traceable docs (Quetzalcoatl layout) under [`docs/`](docs/):
[PROJECT_BRIEF](docs/00-overview/PROJECT_BRIEF.md) ·
[DASHBOARD](docs/00-overview/DASHBOARD.md) ·
[FEASIBILITY](docs/01-discovery/FEASIBILITY_REPORT.md) ·
[DECISION_LOG](docs/02-decisions/DECISION_LOG.md) (D-001…D-012) ·
[ALTERNATIVES](docs/02-decisions/ALTERNATIVES.md) ·
[CROSS_VALIDATION_LOG](docs/02-decisions/CROSS_VALIDATION_LOG.md) ·
[SPEC](docs/03-spec/SPEC.md) ·
[ARCHITECTURE](docs/03-spec/ARCHITECTURE.md) ·
[RISK_REGISTER](docs/04-quality/RISK_REGISTER.md) ·
[TEST_PLAN](docs/04-quality/TEST_PLAN.md) ·
[RUNBOOK](docs/05-ops/RUNBOOK.md) ·
[RETRO](docs/05-ops/RETRO.md)

Every release ships a Linux tarball with sha256 and SLSA build
provenance — verify with
`gh attestation verify qhud-vX.Y.Z-linux-x86_64.tar.gz --owner chquandogong`.

## Roadmap

- Click a tile → focus that pane in your terminal.
- Upstream versioned observe-snapshot export shared by TUI + HUD
  (today: pinned crate rev).
- GNOME Shell extension layer: overview-clean pinning + DING
  coexistence.
- Windows (WorkerW) / macOS (desktop window level) backends.

## Scope

qhud renders; it never acts. No actuation, no notifications, no
telemetry. Single-user, local, observe-only — the same conservative
contract as qmonster.

## License

MIT. See [LICENSE](LICENSE).
