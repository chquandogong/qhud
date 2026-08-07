# DECISION_LOG

> Status: living · Date: 2026-08-05 · Owner: chquandogong

Format: context → options → decision → rationale → residual risk.

---

## D-001 · Build a second frontend, not a TUI feature

- **Context**: the ambient-visibility gap (OFFICE_HOURS) could be
  patched by keeping a terminal always visible instead.
- **Options**: (a) dedicated always-visible terminal running the TUI,
  (b) new standalone widget app, (c) fork qmonster into a GUI.
- **Decision**: (b) — standalone `qhud` linking qmonster as a library.
- **Rationale**: (a) steals a terminal + alt-tab slot and cannot match
  the mockup; (c) forks the maintenance burden. (b) reuses 100% of the
  data pipeline and leaves qmonster untouched.
- **Residual risk**: two frontends share one implicit contract — see
  D-004.

## D-002 · Tauri v2 over Electron / GNOME extension / native GTK

- **Context**: need exact visual parity with a pure-HTML/CSS mockup on
  a ~15 MB always-running widget.
- **Options**: comparison table in `ALTERNATIVES.md` (A–E).
- **Decision**: Tauri v2, frameless transparent window, static
  frontend, `withGlobalTauri` (no bundler).
- **Rationale**: HTML mockup ports nearly verbatim; Rust backend links
  the qmonster crate directly (no IPC layer); ~15 MB binary and modest
  RSS beat Electron for an always-on widget; keep-below mechanism was
  live-verified before commitment.
- **Residual risk**: WebKitGTK rendering quirks (transparency /
  dmabuf). Mitigation: runbook env-var fallbacks; Electron remains the
  documented plan-B.

## D-003 · Force `GDK_BACKEND=x11` (XWayland) on GNOME

- **Context**: Mutter has no layer-shell; Wayland toplevels cannot be
  positioned globally nor kept below (tauri#14913 no-ops).
- **Options**: (a) XWayland + EWMH below/sticky, (b) GNOME Shell
  extension holding the layer, (c) native Wayland and accept a normal
  window, (d) wlr-layer-shell (non-GNOME only).
- **Decision**: (a), with `QHUD_NO_X11_FORCE=1` as the escape hatch and
  the pre-set `GDK_BACKEND` respected.
- **Rationale**: only (a) delivers below+sticky+global positioning on
  stock GNOME today; verified twice on the target machine.
- **Residual risk**: XWayland blur if fractional scaling is enabled
  later (A3); widget appears in the GNOME overview (accepted quirk —
  fixable only by (b), kept as a roadmap option).

## D-004 · Link qmonster at a pinned rev, observe-only (`NoopSink`)

- **Context**: the TUI writes sqlite/audit/archives under
  `~/.qmonster`; a second writer would race it. Upstream lib API has no
  stability promise.
- **Options**: (a) subprocess `qmonster --once` + parse, (b) shared
  sqlite reads, (c) direct lib link with no-write sink, (d) ask
  upstream for a JSON export contract first.
- **Decision**: (c) now — `Context::new(..., Box::new(NoopSink))` +
  `SilentNotify`, dependency pinned with `rev = "aa2bd39…"`; promote to
  (d) once stable.
- **Rationale**: (a) has no machine-readable output today; (b) creates
  read-time coupling to a private schema; (c) reuses parsers with zero
  contention. Config is shared **read-only** from
  `~/.qmonster/config/qmonster.toml`.
- **Residual risk**: rev bumps may break compilation — that is the
  point of the pin (loud, not silent). Roadmap: upstream
  `--emit-json` / versioned export shared by both frontends.

## D-005 · Demo payload mirrors the mockup exactly

- **Context**: no tmux server ⇒ nothing to render; and visual parity
  needs a fixture.
- **Decision**: `demo.rs` reproduces the mockup pane-for-pane and is
  labeled `DEMO` in the UI; it doubles as the parity check.

## D-006 · Release = plain binary tarball via tag-driven CI

- **Context**: qmonster ships npm + binary; qhud v0.1 needs the
  smallest credible release.
- **Decision**: GitHub Actions builds on tag push, uploads
  `qhud-vX.Y.Z-linux-x86_64.tar.gz` + sha256 + build provenance
  attestation. No npm/deb/AppImage yet.
- **Residual risk**: users must install webkit2gtk runtime — documented
  in RUNBOOK.

## D-007 · Mux backend follows qmonster's factory; widget-auto probes herdr → tmux

- **Context**: v0.1.0 hardcoded the tmux `PollingSource`, so herdr
  rigs (this machine's primary mux) never went live. qmonster's
  `[mux] backend = "auto"` resolves via `HERDR_ENV`/`HERDR_SOCKET_PATH`
  — env vars that only exist _inside_ a herdr pane, which a desktop
  widget usually isn't.
- **Options**: (a) require users to set `backend = "herdr"` explicitly,
  (b) qhud re-implements backend detection, (c) reuse
  `app::tmux_source::build_tmux_source` and, when the config says
  `auto` and no herdr env is inherited, probe herdr first then tmux.
- **Decision**: (c). Explicit `tmux`/`herdr` configs are passed through
  untouched; the backend label shown in the footer comes from the
  _resolved_ source, not the config.
- **Rationale**: (a) breaks the shared-config promise (same file, same
  meaning in both frontends); (b) duplicates upstream logic that will
  drift. Probing costs one failed CLI call at worst.
- **Evidence**: live transition verified on 2026-08-05 —
  `qhud: live via herdr (4 panes: …)` against a running herdr 0.7.5
  server with real agent workspaces; tmux fallback preserved.
- **Residual risk**: if herdr _and_ tmux both run, herdr wins under
  widget-auto (documented; set `[mux] backend` explicitly to override).

## D-008 · All window-geometry interaction is self-driven (no compositor interactive ops)

- **Context**: user report — selection, move, and resize all dead on the
  running widget. Instrumented on-machine (setTitle beacons + synthetic
  input) on 2026-08-05.
- **Findings** (each verified, each alone fatal):
  1. wry drag-regions / `startDragging` / `startResizeDragging` —
     compositor-side interactive move/resize — are unreliable for a
     keep-below XWayland window on GNOME (no-op or partial).
  2. tao's invisible borderless-resize inset swallows all pointer input
     within ~10px of the window border (measured: ≤8px eaten, ≥12px
     delivered) — the resize grip lived entirely inside it.
  3. WebKitGTK `event.screenX/Y` goes stale while the window itself is
     moving (constant-lag deltas).
  4. tao `outerPosition()`/`outerSize()` report a phantom ~37px frame
     for this undecorated window (y short by 37, height long by 37).
- **Decision**: drive geometry entirely ourselves — pointer events only
  arm/disarm an rAF loop over Tauri's global `cursorPosition()`;
  setPosition/setSize apply literally; grab metrics come from the DOM
  (`clientX/Y`, `innerWidth/Height`), never from tao outer metrics.
  Grip hit-zone enlarged well inside the tao inset. Capabilities
  trimmed to exactly cursor-position + set-position + set-size.
- **Evidence**: synthetic drags land pixel-exact on both monitors
  (move Δ=(70,55)/(70,55), resize Δ=(52,36)/(52,36), shrink
  Δ=(-45,-36)); DOM click delivery beacon-confirmed on both.
- **Bonus fixes surfaced by the same investigation**:
  `generate_context!` does not register `../ui` with cargo change
  tracking (stale-asset builds — build.rs now emits rerun-if-changed);
  window-state only saved on graceful exit (now checkpointed every
  30 s from the poll loop).
- **Residual risk**: rAF loop costs one IPC round-trip per frame while
  dragging (negligible); `document.title` is not WM_NAME in Tauri —
  future debugging must use `setTitle`.

## D-009 · Tile selection binds to pointerdown, not click

- **Context**: after D-008, move/resize worked but selection still
  didn't (user report, 2026-08-06).
- **Finding**: this WebKitGTK/X11 webview delivers `pointerdown`
  reliably (beacon-proven) but **never synthesizes `click`** from the
  down/up pair — the selection handler simply never ran. Verified by
  a beacon build: pointerdown-bound selection toggles and renders
  (`sel:wC:p1:R → sel:none:- → sel:wD:p1:R`).
- **Exoneration**: localStorage was a suspect (its backing directory
  was absent) but writes work fine — the directory was missing only
  because the click handler had never executed. Persistence is
  nevertheless wrapped in try/catch and ordered after `render()`, so
  storage can never gate the UI.
- **Decision**: all widget interactions bind to pointer events
  (`pointerdown`/`pointerup`), never to synthesized `click`.
- **Residual risk**: pointerdown-selection fires even when the user
  intended a drag — tiles are not drag surfaces (topbar/footer only),
  so no conflict today; revisit if tiles ever become draggable.

## D-010 · Ubuntu DING intercepts real pointer input; verification must use the compositor path

- **Context**: after D-009, selection passed synthetic (XTEST) tests
  but stayed dead for the user's real mouse (report, 2026-08-06).
- **Methodology failure acknowledged**: xdotool/XTEST injects events
  inside XWayland, **bypassing Mutter's surface picking** — the layer
  where real input is routed. Every earlier "verified" pass shared
  this blind spot.
- **New instrument**: compositor-path injection via
  `org.gnome.Mutter.RemoteDesktop` (+ ScreenCast stream for absolute
  coordinates) — events enter Mutter exactly like hardware
  (`scratchpad rd_abs_click.py`, one persistent D-Bus connection:
  sessions die with their creator's connection).
- **Finding (A/B proven)**: Ubuntu's **Desktop Icons NG (DING)**
  extension window swallows all real pointer input over the widget:
  DING off → compositor-path click toggles selection; DING on → the
  identical click vanishes. XWayland's frozen pointer view over the
  widget region independently confirmed a Wayland surface was being
  picked instead of qhud.
- **Disposition**: DING disabled on the reference machine
  (`gnome-extensions disable ding@rastersoft.com`) — `~/Desktop` is
  empty, so it rendered zero icons here. Operator can re-enable at the
  cost of widget interaction (RUNBOOK row). Coexistence for
  icon-users = companion GNOME Shell extension (backlog).
- **Code change**: permanent stderr breadcrumbs
  (`ui_event` command; `qhud ui: sel:…` lines) so real-input behavior
  is verifiable from logs without polluting WM_NAME.
- **Final evidence** (v0.1.4, compositor path, DING off):
  `sel:none:-` → `sel:wC:p3:R` toggle round-trip in stderr.

## D-011 · Scope-correct display: quota is an account fact, shown once per provider

- **Context**: operator critique (2026-08-06) — two `claude:1:main`
  tiles showed *different* 5H/7D values. Correct: 5h/7d windows are
  **account**-scoped, but v0.1 rendered each pane's sidefile snapshot
  as if quota were pane-scoped. Idle sessions hold stale snapshots, so
  the same account showed divergent numbers — misleading, and the
  duplicate labels were indistinguishable. The original mockup itself
  had this semantic error; porting it faithfully preserved it.
- **Decision**: display facts at the scope they are true.
  1. **Provider quota strip** (one row per provider, under the top
     bar): 5H/7D gauges from the freshest snapshot. Freshness rule:
     within a quota window usage only grows, so the **max percent
     across a provider's panes is the freshest reading** (each pane's
     snapshot is a lower bound). Source pane attributed via tooltip.
  2. **Tiles show pane facts only**: status pill + CTX (+ expanded
     config/conflicts). Per-pane quota rows removed.
  3. **Workspace badge** (`@workspace`) disambiguates identical
     labels across workspaces.
- **Payload**: schema v1 additive — `quotas[]` (provider, h5, d7,
  from_label, session), `panes[].session`.
- **Known limits**: assumes one account per provider on the machine
  (multi-account would need account identity from the provider
  surfaces — not exposed today); rollup unit-tested
  (`provider_quotas_takes_max_snapshot_per_window`).
- **Evidence**: live compositor-path selection re-verified on the new
  layout (`sel:wC:p1:R`); README screenshots regenerated.

## D-012 · Font zoom via Ctrl+wheel; layer peek via single-instance argv (signals are forbidden)

- **Context**: operator asked for (1) adjustable font size and (2) a
  way to see the widget when it sits at the very back by design.
- **Font size**: Ctrl+wheel over the widget drives webview page zoom
  (70–160%, step 10%), persisted in localStorage — pointer-only, so
  the no-keyboard-focus contract holds.
- **Peek**: tray check item "Pin above windows" + `qhud --peek` from a
  second process, relayed to the running instance by
  tauri-plugin-single-instance — bind a GNOME custom shortcut to
  `~/.local/bin/qhud --peek`. Footer shows `pinned ·` while above;
  toggling back re-asserts below+sticky.
- **Hard lesson (verified 3× by segfault)**: **never install Unix
  signal handlers in a Tauri/WebKitGTK process.** The first design
  used SIGUSR1 + signal-hook; the process died with SIGSEGV on the
  first signal before the handler thread even logged — WebKitGTK's
  JavaScriptCore reserves SIGUSR1 for thread suspension, and hooking
  it corrupts VM thread control. App-global hotkeys are equally
  unavailable to XWayland clients on Wayland, which is why the
  single-instance argv relay is the design: crash-free, and it
  absorbs accidental double launches (closing the single-instance
  backlog item).
- **Evidence**: `--peek` round-trip verified — BELOW → ABOVE
  (`layer:pinned`) → BELOW+STICKY (`layer:below`), duplicate launch
  absorbed (1 process), no crash.

## D-013 · Quota rows carry account identity; identity reads stay local

- **Context**: operator holds several logins per provider on one
  machine (two Google accounts for agy, two Codex credentials via
  `auth.json` file-swap, a Claude team seat) and asked "whose quota is
  this?" D-011's recorded known-limit claimed multi-account "would need
  account identity from the provider surfaces — not exposed today."
  That claim was **wrong**: every CLI persists its signed-in identity
  in cleartext next to the credential.
- **Decision**: read identity from local files only —
  `~/.claude.json:oauthAccount`, `~/.codex/auth.json:tokens.account_id`,
  `~/.gemini/google_accounts.json:active`. No network, no token use, and
  the credential fields themselves are never opened. Absent or malformed
  input degrades to "no label", never a failed tick.
- **Two tiers, not one**: a Claude team seat carries an organization
  pool *and* the member's own seat, each with its own rate-limit tier
  (`organizationRateLimitTier` / `userRateLimitTier`). `tiers` is
  therefore a list; collapsing it would hide a pool.
- **Display names**: optional operator inventory at
  `~/.config/qhud/accounts.json`, keyed
  `<provider>:<account_id-or-email>`. Deliberately **outside** the repo
  — qhud is a public repo and those keys are personal identifiers.
  Unmapped accounts fall back to email, then account id.
- **Payload**: schema v1 additive — `quotas[].account`, omitted when
  unknown.
- **Known limits**: labels the *active* account only. Showing every
  account's remaining quota at once needs a per-account fetch, and for a
  parked credential that means running the refresh grant — deferred, and
  gated on explicit operator approval, because Codex refresh tokens are
  single-use and rotated (a failed write-back breaks `codex login`).
  Codex exposes no cleartext email, so its inventory key is a UUID.
- **Evidence**: `cargo test` 17 pass; live `--dump` shows
  `claude → dogu/team <chquan@dogu.xyz> DOGU (claude_team)` with both
  tiers, and `codex → 3f13fa37…`.

## D-014 · Binary budget 20 MB → 25 MB; network is opt-in, not ambient

- **Context**: adding per-workspace Codex usage needs an HTTP client.
  `reqwest` + `rustls` took the release binary from 17.9 MB to 22.8 MB,
  past the `SPEC.md` non-functional budget of 20 MB.
- **Decision**: raise the budget to **25 MB**. The 20 MB figure was an
  opening guess from the v0.1 spec, not a measured constraint — no
  packaging, download, or memory limit depends on it, and `strip` +
  `lto` were already on, so the 2.8 MB is the real cost of the feature
  rather than slack. Rejected alternatives: `native-tls` (trades size
  for an OpenSSL link dependency, worse for a tarball release) and
  dropping the feature (the operator asked for it explicitly).
- **The more important half of this decision**: "no network" is
  replaced by "**passive by default, network only on request**". The
  2 s poll loop still opens no socket and touches no credential; the
  one outbound call runs from an operator click and never executes an
  OAuth refresh grant. That distinction is what keeps the widget's
  steady state as safe as it was when it had no network code at all.
- **Evidence**: 22.8 MB measured; `SPEC.md` non-functional section
  updated in the same change.
