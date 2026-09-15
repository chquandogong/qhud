<!-- qhud:languages -->
<p align="center">
  <a href="../i18n/ko/docs/05-ops/RUNBOOK.md">한국어</a> · <strong>English</strong> · <a href="../i18n/zh-CN/docs/05-ops/RUNBOOK.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# RUNBOOK

> Status: living · Date: 2026-09-15 · Owner: chquandogong

## Install

**Windows x64:** use the ZIP release or the checked-in build script described
in [WINDOWS](WINDOWS.md). The Linux instructions below remain unchanged in scope.

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
cargo build --release --locked # binary at target/release/qhud
```

## Run / quit

- Start: `./qhud &` — the widget appears on the desktop layer. Without a
  multiplexer it shows real local account snapshots and zero panes. `--demo`
  explicitly selects example data.
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
  payload the widget renders (one observe tick, pretty JSON). It can
  contain account, workspace, session, command, and path identifiers;
  redact it before sharing.
- **A quota row lost its cost or reset countdown?** Sidefile attribution
  declines silently in three places by design. Name which one:
  `QMONSTER_SIDEFILE_DIAG=1 qhud --dump 2>&1 >/dev/null` reports
  no-cwd-match, the 60 s same-cwd ambiguity guard, or a descendant-CLI
  mismatch.
- **Is the widget actually rendering?** It reports redacted render and
  interaction stages to stderr — strip/label completion, pointer and
  selection delivery, and frontend-error presence. Account labels,
  workspace and pane IDs, paths, usage values, and exception text are
  deliberately omitted. These breadcrumbs
  prove LOGIC, not pixels — an absent error is **not** proof anything
  painted (learned the hard way: a frame-presentation freeze ran
  clicks and fetches invisibly for days). Pixels ARE verifiable:
  `xwd -id <qhud window> | md5sum` twice a few seconds apart must
  differ (the footer clock repaints every second), and the xwd decodes
  to a viewable image. A `framestall:` breadcrumb means the built-in
  watchdog caught a frozen frame clock and is healing (jiggle →
  re-exec).
- **Fetch paths without clicking** (a keep-below widget does not receive
  synthesized pointer input, D-010):
  `qhud --refresh-all`, `qhud --refresh-claude` and `qhud --fetch-codex`
  relay to the running widget through the single-instance channel —
  bindable to a shortcut. `qhud --claude-usage`, `qhud --codex-usage` and
  `qhud --agy-usage` run the same fetches standalone, print JSON, and
  record to the fetched store exactly like a click. `qhud --codex-appserver`
  exercises the expired-token fallback on demand.
  `QHUD_EXTRA_DIAG=1 qhud --claude-usage >/dev/null` logs only whether
  the live body contains `extra_usage` and `spend` as a shape-drift clue.
- **⟳ says "usage response did not parse: …"?** The parser reports only
  a coarse category (`schema mismatch`, JSON syntax, incomplete JSON, or
  I/O) and line/column. It does **not** name a field or show a provider
  value; the position alone cannot establish whether a window, percent,
  or money field changed. For an extra-usage clue, run
  `QHUD_EXTRA_DIAG=1 qhud --claude-usage >/dev/null`: its diagnostic
  breadcrumb on stderr reports only whether `extra_usage` and `spend`
  are present. Keep provider responses and the local Claude cache private.
  Compare any locally held response with the expected types in
  `src-tauri/src/usage_cache.rs`, then make a
  synthetic, redacted minimal JSON reproduction before proposing a parser
  fix. The 2026-09-01, v0.5.3 `extra_usage.used_credits` change from an
  integer to an integral float was one confirmed case (D-019); `lenient_i64` /
  `lenient_u8` handle integer-meaning money fields. A confirmed change to
  a displayed window or percent needs its own contract review. Do not
  post the raw response, account identifiers, or token-bearing files.
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

  Save valid JSON in `accounts.json` (comments and trailing commas are not supported):

  ```json
  {
    "claude_config_dirs": ["~/claude-personal"],
    "codex_homes": ["~/.codex-dogu"]
  }
  ```

  Each Claude dir renders as its own row (identity + its own snapshot
  - ⟳); Codex extra homes join the every-credential scan. Row identity
    is **(account, organization)**: one claude.ai login can hold a team
    seat AND a personal org — pick the org you want at the CLI's
    organization-selection step after the browser OAuth (the browser
    auto-continues with its active session, so the org step is where
    the choice actually happens). A dir matching the default's
    (account, org) is skipped, not duplicated.

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

| Symptom                                                                                                                                          | Fix                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Widget blank / not transparent / **pixels frozen** (clicks work — breadcrumbs fire — but the screen never changes; typical after overnight DPMS) | qhud disables both fragile WebKitGTK paths itself since v0.5.1 (`WEBKIT_DISABLE_DMABUF_RENDERER=1` + `WEBKIT_DISABLE_COMPOSITING_MODE=1` — the threaded compositor's frame clock died at display sleep; libEGL DRI3 errors at launch are the tell) AND self-heals: a Rust-side pixel guard hashes the footer strip every ~28 s; on two static samples it logs `frame freeze detected`, unmaps/remaps the window (the proven unfreezer), then re-execs if still static. If you overrode with `QHUD_KEEP_DMABUF=1` / `QHUD_KEEP_COMPOSITING=1`, unset those. Freeze check: `xwd -id $(xdotool search --class qhud \| tail -1) \| md5sum` twice a few seconds apart — identical hashes = frozen (the footer clock repaints every second) |
| Claude quota row stuck on an old ⟳ (age caption keeps growing) and the topbar ⟳ shows an error tooltip                          | `usage response did not parse: <redacted category and position>` means the endpoint changed shape (see the diagnostics section, D-019); `Claude token rejected (401)` means that config dir needs `claude` re-run — an expired EXTRA account cannot hide the default account's numbers, so if only one row is stale, only that dir is at fault |
| Widget raises above windows                                                                                                                      | confirm XWayland: `xprop WM_CLASS` on the window should answer; if you set `QHUD_NO_X11_FORCE=1`, layering is your compositor's job                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Wrong monitor after unplug                                                                                                                       | geometry restore points at a gone monitor — delete the window-state file under `~/.config/xyz.dogu.qhud/` and restart                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| No tray icon                                                                                                                                     | AppIndicator extension missing — widget still runs; quit via `pkill qhud`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Blurry on HiDPI                                                                                                                                  | fractional scaling + XWayland on GNOME 46 blurs X11 clients; run displays at integer scale or upgrade to GNOME 47+ (`xwayland-native-scaling`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| No panes detected with tmux running                                                                                                              | qhud probes every 10 s; check the same config the TUI uses (`~/.qmonster/config/qmonster.toml` `[mux]/[tmux]` target). Restart without `--demo` if example data was explicitly selected.                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Drag/resize ignores the outermost ~10px of the window                                                                                            | that border strip belongs to tao's built-in edge handler (D-008) — grab the topbar/footer interior to move, the ◢ glyph to resize                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Widget visible but ignores ALL real mouse input (synthetic/xdotool works)                                                                        | Ubuntu's Desktop Icons NG extension swallows real pointer input over the desktop layer (D-010): `gnome-extensions disable ding@rastersoft.com`. Icon users: companion-extension coexistence is on the backlog. Check `qhud ui:` stderr breadcrumbs to confirm whether clicks reach the widget                                                                                                                                                                                                                                                                                                                   |

## Update the qmonster pipeline

Bump the `rev` in `src-tauri/Cargo.toml`, `cargo build`, fix whatever
the compiler surfaces in `view.rs`/`poll.rs`, re-run the TEST_PLAN
manual checklist.

## Release procedure

1. Update README, CHANGELOG, platform instructions, and the English, Korean,
   and Chinese `docs/05-ops/releases/v<version>.md` notes. Align the Cargo
   package, Tauri config, and Cargo lock package versions.
2. Push a `codex/` preparation branch and open a PR. Update it against `main`
   for strict checks, resolve review conversations, and wait for all seven
   required statuses (Ubuntu, Windows, docs/metadata, dependency review, and
   three CodeQL Analyze contexts). The current `main` ruleset requires a PR,
   linear history, and squash merge; it requires zero approving reviews for
   the sole maintainer. Windows CI uses `scripts/Build-Windows.ps1 -Test` and
   verifies manifest/lock restoration; do not run Cargo concurrently there.
3. Squash-merge the gated PR into `main`. Create and push an annotated stable
   `vX.Y.Z` tag on a commit already contained in `origin/main`; protected
   `v*` tags cannot be changed or deleted. Release preflight also requires
   matching Cargo/Tauri versions, release notes, and a CHANGELOG entry.
4. The tag-triggered Release workflow runs the full CI quality gate, then
   packages the quality-gated Linux x86_64 binary and Windows x86_64 binary
   as a tarball and ZIP, with SHA-256 files and provenance attestations. It
   verifies exactly four assets and both checksums before the `release`
   environment gates publication. As last verified on 2026-09-14, that gate
   needs the maintainer's review and permits self-review. After approval,
   the publish job creates or resumes a **draft first**, checks draft assets
   byte-for-byte, uploads missing assets, and publishes only the complete
   draft. It refuses to replace an already published release.
5. Confirm the workflow succeeded, inspect the release notes and all four
   downloadable assets, verify checksums and attestations, and record any
   desktop integration not exercised on a live supported platform.
