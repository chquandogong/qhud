# DECISION_LOG

> Status: living · Date: 2026-09-04 · Owner: chquandogong

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

## D-015 · Multi-account = per-account CLI config dirs, not qhud-owned logins

- **Context**: the operator's stated purpose for qhud (2026-08-10) is
  every account's usage and reset times at a glance — several accounts
  per provider — with explicit refresh, so they never open the provider
  web pages again. v0.4.0's known limit said "one live login per
  provider"; that was a fact of reading only the default credential
  path, not of the machine: Claude Code keeps a complete identity +
  usage cache + credential per `CLAUDE_CONFIG_DIR`, and Codex per
  `CODEX_HOME`.
- **Decision**: the registry (`~/.config/qhud/accounts.json`) gains
  `claude_config_dirs` and `codex_homes`. Each Claude dir contributes
  its own strip row — identity from that dir's `.claude.json`, numbers
  from the fresher of its own usage cache and qhud's last ⟳ for it —
  and one ⟳ walks every account, recording each under its own store
  key. Codex extra homes join the existing every-credential scan.
  Login and token rotation stay with the CLIs; qhud still never runs a
  refresh grant.
- **Rejected**: qhud holding its own per-account OAuth logins (device
  flow). It is the only route to full independence from the CLIs, but
  it moves credential custody, rotation failure ("bricked login") and
  policy risk into a HUD. Revisit only if the config-dir route proves
  insufficient in practice.
- **Known limits**: a pane's account is not attributable, so pane-fed
  gauges always land on the default account's row. agy multi-account
  needs OS-keyring reverse engineering — not attempted.
- **Evidence**: live `--dump` with a second config dir shows two claude
  rows with their own tiers/origins/ages; `fetch_all` logs a partial
  error for a credential-less dir while the default still fetches.

## D-016 · Delegated fetch paths: the provider's own process may do the talking

- **Context**: two failure modes the raw paths cannot fix — an expired
  Codex access token 401s (and qhud must never run the rotated,
  single-use refresh grant), and agy exposes no HTTP usage endpoint
  qhud could call with a stored credential at all (live token in the
  OS keyring).
- **Decision**: extend D-014's "network only on request" with a third
  kind of on-request path — asking the provider's OWN process:
  `codex -s read-only -a untrusted app-server` (JSON-RPC over stdio,
  `account/rateLimits/read`) as a fallback for the active login, and
  agy's loopback Connect RPC `RetrieveUserQuotaSummary` (tokenless,
  machine-local, /proc port discovery). In both, credential custody and
  rotation stay entirely with the CLI; qhud reads no token.
- **Ordering**: raw HTTP first for Codex (fast), app-server only on
  failure of the active login; agy has no raw path, so loopback is
  primary. Both remain click-only — the 2 s poll loop is unchanged.
- **Evidence**: `--codex-appserver` returns the active workspace with
  plan/credits/windows; `--agy-usage` discovered the port and parsed
  all four pools live.

## D-017 · The widget audits its own pixels (frame guard)

- **Context**: three days of "selection doesn't work" reports against a
  widget whose logs showed every click working. The screen was frozen
  on an hours-old frame while JS, input and IPC ran — at display sleep,
  Mutter stops scheduling frames for the keep-below window and the GTK
  frame clock never resumes. Every mechanism-level fix failed live:
  DMABUF-off recurred the same day; compositing-off plus a JS rAF
  watchdog recurred within two hours with the watchdog blind (rAF keeps
  firing in software mode, decoupled from the screen). An external 1 px
  resize is ignored for this window (D-008), so jiggle heals are void;
  unmap/remap was proven live to resume painting.
- **Decision**: stop betting on mechanisms; measure the symptom. A
  Rust-side guard hashes a footer pixel strip of the widget's own
  window every ~28 s (the clock there repaints every second). Two
  identical samples ⇒ frozen ⇒ hide+show and re-assert layer states;
  still static one sample later ⇒ re-exec with `--respawned` (the child
  waits out the dying process so the single-instance guard does not
  absorb it). One "frame guard armed" line at startup proves the
  sampler itself; every detection and heal is logged.
- **Rejected**: env-only mitigation (kept as cheap belts —
  `WEBKIT_DISABLE_DMABUF_RENDERER`, `WEBKIT_DISABLE_COMPOSITING_MODE`,
  opt-outs `QHUD_KEEP_*` — but proven insufficient alone); in-page rAF
  watchdogs (blind by construction); resize jiggles (no-op for this
  window).
- **Corollary, written into the RUNBOOK**: breadcrumbs prove logic,
  never paint. Pixels are verifiable — `xwd | md5sum` twice, or the
  guard's own log — and D-010's "absence of error is not proof
  anything painted" now has its enforcement mechanism.
- **Evidence**: unit-tested decision ladder; "armed" line on deploy;
  field result 2026-08-13→14: three freezes, three sub-minute remap
  heals, zero re-execs, zero operator-visible incidents.

## D-018 · A quota row's identity is (account, organization)

> Shipped in v0.5.2 (2026-08-14/17, `dab68af`); recorded here 2026-09-04,
> when a docs audit found the decision had reached the CHANGELOG,
> DASHBOARD and RUNBOOK but never this log.

- **Context**: wiring what the operator called their "second account"
  showed there was no second account. One claude.ai login — same email,
  same `accountUuid` — belongs to two organizations: a team seat and a
  personal free org, each with its own quota pools. A CLI login is
  scoped to one org per config dir, and the org is chosen at the CLI's
  organization step after the browser OAuth, not by the browser.
  D-015's dedupe keyed on account id alone, so a second org was
  discarded as "a duplicate of the default".
- **Decision**: identity is the pair. `AccountLabel` carries `org_id`
  (`organizationUuid`) alongside the account id and exposes its
  `config_dir`; dedupe keys on (account, org); the frontend keys strip
  rows the same way and matches each row to its ⟳ slice **by config
  dir**. Matching a ⟳ result by account id would feed one org's numbers
  to both rows of the same login — the failure the pair exists to
  prevent.
- **Rejected**: keying rows by config dir alone (two dirs holding the
  same (account, org) are genuinely one row, and re-logins create
  exactly that); keying by email (an email is not a quota scope).
- **Known limits**: field verification of a real second-org row still
  waits on a login that picks the PERSONAL org at the CLI's
  organization step — the OAuth flow keeps auto-continuing with the
  browser's active team session.
- **Evidence**: a team re-login into the extra dir adds no row (same
  account, same org, correctly deduped); the extra dir's own
  `.claude.json` supplies its identity with zero network.

## D-019 · Wire numbers are read for their meaning; a rejected body must name its field

- **Context**: on 2026-09-01 `/api/oauth/usage` began serializing
  `extra_usage.used_credits` as `4997.0` instead of `4997`. serde
  rejects a float for `i64`, so one optional fallback field failed the
  entire response — 5h/7d windows included — and every Claude ⟳ failed
  for two days. The error string was "usage response did not parse",
  which named nothing, so the failure was indistinguishable from a dead
  token or an empty body. The provider's own client shipped the same
  week with in-band handling for empty and fieldless bodies on this
  endpoint, so its number formatting is not a contract qhud may lean
  on.
- **Decision**: two rules. (1) Integer-meaning money fields
  (`used_credits`, `decimal_places`, `amount_minor`, `exponent`) parse
  through a lenient reader: an integral float IS that integer, a
  fractional float is dropped as unit-ambiguous (the existing
  scale-guess prohibition — $50 must never render as $0.50), and
  neither outcome may fail the surrounding body. Optional fields stay
  optional all the way down: no single field may cost the operator the
  windows. (2) A parse rejection carries serde's field-and-type message
  with its position. That message is identity-free by construction —
  unknown fields are skipped untyped and every typed field is a number,
  a boolean, or a window/plan string — so the D-013 rule against
  logging this body still holds.
- **Rejected**: `#[serde(untagged)]` enums per field (a silent
  catch-all that would also swallow strings); parsing the whole body as
  `serde_json::Value` and hand-walking it (loses the type contract that
  caught the earlier Codex drift); rounding a fractional float to minor
  units (a scale guess, forbidden since v0.5.0); echoing the response
  body into the error (identity leak).
- **Evidence**: three tests — the live 09-03 body verbatim, an
  integral-vs-fractional pair, and a rejection asserting the message
  names the mismatch and does not echo the body. Live after the fix,
  the widget's own ⟳ logged
  `claude usage ok [default] (5h 49%, 7d 7%, 3 scoped)`.
