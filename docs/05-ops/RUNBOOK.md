# RUNBOOK

> Status: living · Date: 2026-08-07 · Owner: chquandogong

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
- **Font size**: hold Ctrl and scroll the mouse wheel over the widget
  (70–160%, remembered).
- **Peek (bring to front temporarily)**: tray → _Pin above windows_
  (again to send back), or run `~/.local/bin/qhud --peek`. For a
  keyboard shortcut: GNOME Settings → Keyboard → Custom Shortcuts →
  command `/home/USER/.local/bin/qhud --peek` (e.g. Super+Q). Never
  send Unix signals to qhud — WebKitGTK reserves them (D-012).
- Launching `qhud` while one is running is absorbed by the running
  instance (single-instance guard).
- **Doubting a number?** `~/.local/bin/qhud --dump` prints the exact
  payload the widget renders (one observe tick, pretty JSON).
- **A quota row lost its cost or reset countdown?** Sidefile attribution
  declines silently in three places by design. Name which one:
  `QMONSTER_SIDEFILE_DIAG=1 qhud --dump 2>&1 >/dev/null` reports
  no-cwd-match, the 60 s same-cwd ambiguity guard, or a descendant-CLI
  mismatch.
- **Is the widget actually rendering?** It reports what it built, to
  stderr — `strip: N sections, M rows, K gauges`, the rendered text of
  every row (`labels(...)`), and `js-error …` for any frontend
  exception. The pixels are unverifiable from outside the webview
  (`scrot` cannot capture XWayland-composited windows, D-010), so these
  breadcrumbs are the verification, not a nicety. An absent error is
  **not** proof anything painted.
- **Fetch paths without clicking** (a keep-below widget does not receive
  synthesized pointer input, D-010):
  `qhud --refresh-all`, `qhud --refresh-claude` and `qhud --fetch-codex`
  relay to the running widget through the single-instance channel —
  bindable to a shortcut. `qhud --claude-usage`, `qhud --codex-usage` and
  `qhud --agy-usage` run the same fetches standalone, print JSON, and
  record to the fetched store exactly like a click. `qhud
--codex-appserver` exercises the expired-token fallback on demand.
  `QHUD_EXTRA_DIAG=1 qhud --claude-usage` prints the live body's
  identity-free `extra_usage`/`spend` sub-objects for shape drift.
- **Accounts and plans** live in `~/.config/qhud/accounts.json`,
  deliberately outside this public repo. `labels` / `plans` /
  `workspace_names` / `workspace_plans` set display text; `known[]`
  lists ever-connected accounts; `forgotten` hides a placeholder (never
  a live account). Display names are operator-supplied and must never be
  "corrected" from a wire `plan_type` — `prolite` is shown as ChatGPT
  Pro 5x, `team` as ChatGPT Business.
- **Several accounts per provider** (D-015): keep each extra account
  signed in under its own dir, then register the dir —

  ```bash
  CLAUDE_CONFIG_DIR=~/claude-personal claude   # sign in once, keep it
  CODEX_HOME=~/.codex-dogu codex login          # same idea for codex
  ```

  ```jsonc
  // ~/.config/qhud/accounts.json
  {
    "claude_config_dirs": ["~/claude-personal"],
    "codex_homes": ["~/.codex-dogu"],
  }
  ```

  Each Claude dir renders as its own row (identity + its own snapshot
  - ⟳); Codex extra homes join the every-credential scan. A dir whose
    account matches the default is skipped, not duplicated.

- **qhud's own ⟳ results** persist in `~/.config/qhud/fetched-usage.json`
  (same outside-the-repo privacy rule; written temp+rename). Deleting it
  is always safe — the next ⟳ rebuilds it.

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
the tray) settle first. Duplicate launches are absorbed by the running
instance (single-instance guard, v0.3.0).

**After rebuilding a new version**, refresh the installed copy:
`install -m755 target/release/qhud ~/.local/bin/qhud && pkill -x qhud && ~/.local/bin/qhud &`

## Troubleshooting

| Symptom                                                                                                                                          | Fix                                                                                                                                                                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Widget blank / not transparent / **pixels frozen** (clicks work — breadcrumbs fire — but the screen never changes; typical after overnight DPMS) | qhud sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` itself since v0.5.1 (the DMABUF path froze at display power cycles; libEGL DRI3 errors at launch are the tell). If you overrode it with `QHUD_KEEP_DMABUF=1`, unset that. Freeze check: `xwd -id $(xdotool search --class qhud \| tail -1) \| md5sum` twice a few seconds apart — identical hashes = frozen (the footer clock repaints every second) |
| Widget raises above windows                                                                                                                      | confirm XWayland: `xprop WM_CLASS` on the window should answer; if you set `QHUD_NO_X11_FORCE=1`, layering is your compositor's job                                                                                                                                                                                                                                                               |
| Wrong monitor after unplug                                                                                                                       | geometry restore points at a gone monitor — delete the window-state file under `~/.config/xyz.dogu.qhud/` and restart                                                                                                                                                                                                                                                                             |
| No tray icon                                                                                                                                     | AppIndicator extension missing — widget still runs; quit via `pkill qhud`                                                                                                                                                                                                                                                                                                                         |
| Blurry on HiDPI                                                                                                                                  | fractional scaling + XWayland on GNOME 46 blurs X11 clients; run displays at integer scale or upgrade to GNOME 47+ (`xwayland-native-scaling`)                                                                                                                                                                                                                                                    |
| Stuck in `DEMO` with tmux running                                                                                                                | qhud probes every 10 s; check the same config the TUI uses (`~/.qmonster/config/qmonster.toml` `[mux]/[tmux]` target)                                                                                                                                                                                                                                                                             |
| Drag/resize ignores the outermost ~10px of the window                                                                                            | that border strip belongs to tao's built-in edge handler (D-008) — grab the topbar/footer interior to move, the ◢ glyph to resize                                                                                                                                                                                                                                                                 |
| Widget visible but ignores ALL real mouse input (synthetic/xdotool works)                                                                        | Ubuntu's Desktop Icons NG extension swallows real pointer input over the desktop layer (D-010): `gnome-extensions disable ding@rastersoft.com`. Icon users: companion-extension coexistence is on the backlog. Check `qhud ui:` stderr breadcrumbs to confirm whether clicks reach the widget                                                                                                     |

## Update the qmonster pipeline

Bump the `rev` in `src-tauri/Cargo.toml`, `cargo build`, fix whatever
the compiler surfaces in `view.rs`/`poll.rs`, re-run the TEST_PLAN
manual checklist.
