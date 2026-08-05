# RUNBOOK

> Status: living · Date: 2026-08-05 · Owner: chquandogong

## Install

**Release tarball** (Linux x86_64):

```bash
gh release download --repo chquandogong/qhud --pattern '*linux-x86_64.tar.gz'
tar -xzf qhud-v*-linux-x86_64.tar.gz && cd qhud-v*/
./qhud &
```

Runtime deps (Ubuntu 24.04 names): `libwebkit2gtk-4.1-0`, `libgtk-3-0`,
`libayatana-appindicator3-1` (tray, optional).

**From source**:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
git clone https://github.com/chquandogong/qhud && cd qhud
cargo build --release          # binary at target/release/qhud
```

## Run / quit

- Start: `./qhud &` — the widget appears on the desktop layer and
  shows `DEMO` until a tmux server with AI CLI panes exists.
- Move: drag the top bar (or footer). Resize: drag the ◢ grip.
- Quit: tray icon → _Quit qhud_, or `pkill qhud` (no titlebar by
  design).

## Autostart + app launcher (GNOME)

Install the binary to a stable path first — pointing autostart at
`target/release/` breaks on the next `cargo clean`:

```bash
install -Dm755 target/release/qhud ~/.local/bin/qhud
install -Dm644 src-tauri/icons/128x128.png \
  ~/.local/share/icons/hicolor/128x128/apps/qhud.png

mkdir -p ~/.config/autostart ~/.local/share/applications
cat > ~/.config/autostart/qhud.desktop <<EOF
[Desktop Entry]
Type=Application
Name=qhud
Comment=Ambient desktop HUD for AI CLI sessions
Exec=$HOME/.local/bin/qhud
Icon=qhud
Terminal=false
Categories=System;Monitor;
StartupNotify=false
StartupWMClass=qhud
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=3
EOF
cp ~/.config/autostart/qhud.desktop ~/.local/share/applications/qhud.desktop
```

The `applications` copy also puts qhud in the GNOME app grid. The 3 s
autostart delay lets the desktop (and the AppIndicator extension, for
the tray) settle first. There is no single-instance guard yet: if a
widget is already running, launching again stacks a second one —
`pkill -x qhud` first.

**After rebuilding a new version**, refresh the installed copy:
`install -m755 target/release/qhud ~/.local/bin/qhud && pkill -x qhud && ~/.local/bin/qhud &`

## Troubleshooting

| Symptom                                               | Fix                                                                                                                                            |
| ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Widget blank / not transparent                        | `WEBKIT_DISABLE_DMABUF_RENDERER=1 ./qhud` (WebKitGTK dmabuf quirk, common on NVIDIA)                                                           |
| Widget raises above windows                           | confirm XWayland: `xprop WM_CLASS` on the window should answer; if you set `QHUD_NO_X11_FORCE=1`, layering is your compositor's job            |
| Wrong monitor after unplug                            | geometry restore points at a gone monitor — delete the window-state file under `~/.config/xyz.dogu.qhud/` and restart                          |
| No tray icon                                          | AppIndicator extension missing — widget still runs; quit via `pkill qhud`                                                                      |
| Blurry on HiDPI                                       | fractional scaling + XWayland on GNOME 46 blurs X11 clients; run displays at integer scale or upgrade to GNOME 47+ (`xwayland-native-scaling`) |
| Stuck in `DEMO` with tmux running                     | qhud probes every 10 s; check the same config the TUI uses (`~/.qmonster/config/qmonster.toml` `[mux]/[tmux]` target)                          |
| Drag/resize ignores the outermost ~10px of the window | that border strip belongs to tao's built-in edge handler (D-008) — grab the topbar/footer interior to move, the ◢ glyph to resize              |

## Update the qmonster pipeline

Bump the `rev` in `src-tauri/Cargo.toml`, `cargo build`, fix whatever
the compiler surfaces in `view.rs`/`poll.rs`, re-run the TEST_PLAN
manual checklist.
