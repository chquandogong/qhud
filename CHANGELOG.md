# Changelog

All notable changes to qhud. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [0.5.3] — 2026-09-04

The usage endpoint changed how it writes one number. qhud stopped
reading the whole body for two days.

### Fixed

- **A wire number that turned into a float killed every Claude ⟳.**
  From 2026-09-01 18:26 every refresh reported "usage response did not
  parse" while HTTP was 200 and the Codex and agy fetches in the SAME
  click succeeded; the strip went on rendering the 08-31 19:00
  snapshot, correctly dated, for two days. Cause, found by running the
  shipped diagnostic (`QHUD_EXTRA_DIAG=1 qhud --claude-usage`): the
  body is valid JSON, but `extra_usage.used_credits` now arrives as
  `4997.0` where it used to be `4997`, and serde rejects a float for
  `i64` — so ONE optional fallback field failed the entire response,
  including the 5h/7d windows that had nothing to do with it. The same
  4997 minor units still arrive as an integer in
  `spend.used.amount_minor`, which is what proves a float there is
  minor units and not dollars. Integer-meaning money fields
  (`used_credits`, `decimal_places`, `amount_minor`, `exponent`) now
  read an integral float as that integer; a fractional float keeps the
  existing scale-guess prohibition and is dropped — but neither can
  fail the body any more. Verified live: the widget's own ⟳ answered
  `claude usage ok [default] (5h 49%, 7d 7%, 3 scoped)` with the
  extra-usage row intact, and the result persisted to the fetched
  store.

### Changed

- **A rejected usage body now names the field that drifted.** The error
  was the bare string "usage response did not parse" — a two-day
  diagnosis. It now carries serde's own field-and-type message with its
  position. The body holds account uuid and email; this message holds
  neither, by construction: unknown fields are skipped untyped, and
  every typed field is a number, a boolean, or a window/plan string.
  HTTP 200 plus an unexplained "no data" is the same shape of report
  that once hid two Codex bugs (`a937a3b`), and it is now a shape qhud
  cannot produce.

### Noted

- **The frame guard is working far harder than v0.5.2 recorded.** Field
  tally over the journal's full coverage, 2026-08-26 → 09-04: **21
  freezes detected, 21 remap heals, 0 re-execs, 0 operator-visible
  incidents** (the v0.5.2 entry said 4 through 08-17, measured over the
  guard's first days). Freezes cluster
  — three inside three minutes on 09-01, two inside two minutes on
  09-04 — so each heal is real (the ladder never escalated) but the
  underlying stall recurs quickly once the display has slept. D-017's
  detector-and-heal stance is what makes this invisible; the trigger is
  still not reproducible on demand.
- The extra account in `~/claude-personal` is still 401: its token
  expired 2026-08-17 and no login has replaced it. Unrelated to this
  drift, and it never hid the default account's numbers — partial
  failure stayed partial, as D-015 requires.
- `tauri.conf.json` had carried `"version": "0.4.0"` since v0.4.0;
  it now tracks `Cargo.toml` again.

## [0.5.2] — 2026-08-17

### Fixed

- **Row identity is (account, organization), not account alone.** Wiring
  the operator's "second account" revealed there is no second account:
  one claude.ai login (same email, same accountUuid) belongs to TWO
  organizations — a team seat and a personal free org — each with its
  own quota pools, and a CLI login is scoped to one org per config dir.
  D-015's dedupe by account id silently discarded any second org as "a
  duplicate of the default". `AccountLabel` now carries `org_id`
  (organizationUuid) and exposes `config_dir`; dedupe keys on
  (account, org); the frontend keys strip rows the same way and matches
  each row to its ⟳ slice by config dir — matching by account id alone
  would have fed one org's numbers to both rows of the same login.
  Same account re-logged into the SAME org still dedupes (verified live:
  a team re-login in the extra dir adds no row). Field verification of
  an actual second-org row is pending an operator login that picks the
  personal org at the CLI's organization step — the OAuth flow keeps
  auto-continuing with the browser's active team session.

### Noted

- The v0.5.1 frame guard is earning its keep in the field: freezes
  keep occurring at display sleep (4 detected through 08-17) and every
  one was healed by the remap rung, zero re-execs, zero
  operator-visible incidents.

## [0.5.1] — 2026-08-14

"Selection doesn't work" — three real causes, peeled in order with the
new input breadcrumbs, ending at a frozen renderer.

### Fixed

- **The widget's pixels froze after display sleep** (the root of the
  operator's recurring morning "selection doesn't work" and the daily
  restarts). Proven live: the window pixmap stayed byte-identical
  across seconds while JS, input and IPC all ran — tile and row
  selections fired their breadcrumbs, fetches went out, and the
  operator was looking at an hours-old frame the whole time. Fixed in
  depth, because the trigger (idle blank + lock) is not reproducible
  on demand:
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1` (the GPU path was degraded from
    launch — libEGL "DRI3 device" errors). Alone it was NOT enough:
    the freeze recurred the same day at an idle blank, with the
    variable confirmed in the WebKit child's environment.
  - `WEBKIT_DISABLE_COMPOSITING_MODE=1` — the frozen instance showed
    WebKit's VBlankMonitor waiting on a DRM vblank, so the threaded-
    compositor frame clock goes entirely. Software rendering is
    effortless at this size. Opt-outs: `QHUD_KEEP_DMABUF=1`,
    `QHUD_KEEP_COMPOSITING=1`.
  - Neither env proved sufficient alone (the freeze recurred with both
    set), and a JS rAF watchdog stayed blind through a real freeze —
    software rendering decouples rAF from the screen. What holds is
    measuring the SYMPTOM: **a Rust-side pixel guard** hashes a strip
    of the widget's own window every ~28 s (the footer clock there
    repaints every second). Two identical samples ⇒ frozen ⇒ heal:
    unmap/remap first (verified live to resume painting on an actual
    frozen instance; layer states re-asserted after), then re-exec
    with `--respawned` if still static. Arms itself with one
    "frame guard armed" stderr line; every detection and heal is
    logged. The widget can no longer stay frozen — it heals within a
    minute or says exactly why not. Field result, first 22 h on the
    reference machine: three freezes, three remap heals, zero
    re-execs, and the operator noticed none of them.

- **Strip rows now SELECT.** The operator's recurring "selection does
  nothing" report was diagnosed live: the new `ptr:` breadcrumbs showed
  every click arriving — and every one landing on quota-strip rows,
  which had no selection behavior at all (tiles, which do select, were
  not where the operator was clicking; daily restarts had never been
  the variable). Clicking any strip row — account, workspace, ghost —
  now expands its detail inline: the same facts the hover tooltip
  carries, which barely works on a keep-below window.
- **Ghost rows had TWO identical pointerdown listeners**, so a click
  toggled twice and visibly did nothing. One unified strip handler now
  owns ⟳ dispatch, ghost dismissal, and row selection.
- **Clicking the codex row no longer fires a network fetch.** Fetching
  moved to an explicit ⟳ on the row (like Claude and agy) — selecting
  a row must never be a network action. `--fetch-codex` unchanged.

### Added

- **`ptr:` breadcrumbs** — every real pointerdown logs coordinates,
  button and resolved target at the CAPTURE phase, before any handler
  can swallow it. Presence proves delivery and names what was hit;
  absence during a "clicks do nothing" moment relocates the bug to the
  compositor/session layer. `qsel:` mirrors the tiles' `sel:` for
  strip-row selection.

## [0.5.0] — 2026-08-10

Every account, at a glance, without the provider web pages: several
Claude accounts at once, agy without a pane, Codex that survives an
expired token, extra-usage spend, and readings that survive a restart.

### Added

- **Several Claude accounts at once** (D-015, FR-22). The registry
  gains `claude_config_dirs` / `codex_homes`: each Claude config dir
  kept signed in via `CLAUDE_CONFIG_DIR` contributes its own strip
  row — identity from that dir's `.claude.json`, numbers from the
  fresher of its usage cache and qhud's last ⟳ for it, dated with
  true origin. One ⟳ walks every account; an expired one logs a
  partial error and never hides the others. "One live login per
  provider" is now a default, not a ceiling.
- **Explicit-fetch results persist** (FR-20).
  `~/.config/qhud/fetched-usage.json` (temp+rename, outside the repo)
  keeps the last ⟳ per provider and per extra account; on restart the
  freshest snapshot renders dated (`⟳ 12m ago` vs `~22h old`) instead
  of falling back to a day-old CLI cache. Stored Codex workspace rows
  ride the payload dimmed, and synthesize a provider row when no
  codex pane runs.
- **Claude extra-usage spend** (FR-19): the usage endpoint's
  `spend`/`extra_usage` pair — the last thing the provider's own page
  shows that qhud dropped — normalized to minor-unit money and shown
  on the claude account line (`extra $X.XX[/limit]`),
  severity-tinted, detailed in the tooltip. A bare-float limit is
  dropped rather than scale-guessed.
- **agy quota on ⟳, no pane required** (FR-23). A running agy binds
  loopback Connect listeners; `RetrieveUserQuotaSummary` answers the
  CLI surface with no token and no CSRF. Port discovery is
  /proc-only; gemini pools take the primary gauges, every other pool
  (3p-\*, future ones) becomes a scoped `pool_5h`/`pool_weekly` chip.
  `qhud --agy-usage` is the CLI twin.
- **Codex app-server fallback** (FR-24): when the active login's raw
  `/wham` fetch fails (typically an expired access token), a
  short-lived `codex -s read-only -a untrusted app-server` child
  answers `account/rateLimits/read` — Codex owns its own rotation, so
  the reading works without qhud touching a credential.
  `qhud --codex-appserver` exercises the path on demand (FR-18).
- **One ⟳ for everything** (FR-21): a topbar button (excluded from
  the drag region) refreshes every provider concurrently and mirrors
  the union of fetch states; `qhud --refresh-all` relays from a GNOME
  shortcut. Per-provider triggers stay; the agy row gets its own ⟳.
- **`QHUD_EXTRA_DIAG=1`**: env-gated, identity-free dump of the live
  body's `extra_usage`/`spend` sub-objects, for shape drift. Used to
  prove a surprise was real data, not a parse bug: the org had
  disabled extra usage while the day-old cache still said enabled.

### Changed

- Snapshot merging: a pane row's MISSING window is now filled from
  the snapshot (filling a hole is not overriding a reading — seen
  live on an agy pane that carried only the weekly window). Scoped
  windows render as gauges only on rows whose provenance caption
  covers them (snapshot-origin rows; dimmed with the stale marker on
  CLI-cache rows); pane rows keep them tooltip-only (the v0.4.0
  Fable lesson).
- **One window vocabulary** (operator report): the strip said `7D`
  while codex workspace rows said `weekly` and scoped chips said `wk` —
  three names for the same 7-day rolling window. The frontend now
  displays duration names everywhere (`5H`/`1D`/`7D`/`30D`); wire
  labels are unchanged. `labels(...)` breadcrumbs now include gauge
  labels, so a wrong window name is visible from stderr.
- `fetch_claude_usage` returns one entry per account;
  `attach_usage_cache` is provider-generic; strip rows are keyed by
  (provider, account) instead of provider.
- tokio gains `process`/`io-util`/`time` for the app-server child.

### Fixed

- **Two identically-labelled "weekly" chips with different values on a
  Codex workspace row** (operator report — the D-011 class of mistake
  again). The main pool and the per-model pools share a 7-day duration,
  and the label was derived from duration alone, dropping the pool
  name. `UsageWindow` now carries `scope` (from `limit_name` /
  `limitName`, wire id as fallback) on both the HTTP and app-server
  paths; the chip reads "Codex-Spark wk" (version prefix dropped for
  width, full name in the tooltip) and the main pool stays "weekly".
- **The codex account row and its `↳` workspace row showed the same 7D
  pool twice, clock-skewed** (operator report, same D-011 root). The
  pane statusline on the account row IS the active login's workspace,
  so the fetched copy of that workspace now merges into the account row
  (pane wins, holes fill, per-model pools become chips) and the row
  names the workspace it shows (`chquan17@gmail.com · personal`, plan
  and credits in the tooltip). `↳` rows render only for OTHER
  workspaces, where a differing value means something.
- **CI was red on main since e6e31ed**: clippy `-D warnings` tripped
  over dead `read_auth()` and the unmodelled `structure` field left
  behind by the every-credential change. Both removed.
- `attach_accounts` used to stamp the first detected account onto
  EVERY row of a provider — with multi-account rows that would have
  relabelled one account's numbers with another's name (caught by
  test before it shipped).

## [0.4.0] — 2026-08-07

Quota you can trust: the numbers now match what the providers' own
screens say, and where they cannot, the widget says so instead of
guessing.

### Fixed

- **Quota attribution was silently broken for any pane running an MCP
  plugin child.** `current_path` followed the pane's foreground
  descendant, so a Claude pane with a `bun`/`node` plugin child reported
  the plugin's cache directory. Provider sidefiles are matched to a pane
  by exact cwd equality, so the match failed and every sidefile-only
  signal was dropped: `cost_usd`, both reset timestamps, and
  `context_window_size`. **FR-8's `resets 47m` countdown was marked done
  but had no timestamp to count down from.** Fixed upstream in qmonster
  (`eff5f63`, `f5c9ed6`, `6a21c44`) across three layers — the cwd itself,
  the fallback order for when `pane process-info` fails, and a
  process-confirmation gate that excluded `pane_pid` even when herdr
  reports the agent process directly as the pane pid.
- **Statusline sidefiles were written truncate-in-place**, so the 2 s
  poller could read a torn file and lose cost/reset for that tick. The
  operator's statusline scripts now write temp+rename.
- **A per-model window showed a 27-hour-old value as if live** (Fable 5%
  against an actual 22%). Per-chip provenance is now distinct from row
  provenance.

### Added

- **Account identity on every quota row** (D-013). Read from files the
  CLIs already keep in cleartext — no network, no token use, and the
  credential fields themselves are never opened. A Claude team seat
  carries two quota-bearing tiers (org pool and member seat), so both
  are surfaced.
- **Provider-grouped layout.** Provider is the outer axis as a section
  header; account and plan sit on the identity line; each window gets
  its own gauge line. Eight accounts across three providers do not fit
  one row each at the widget's real width.
- **A registry of accounts that have ever connected**
  (`~/.config/qhud/accounts.json`, deliberately outside this public
  repo). Accounts with no live credential render as dimmed, dated
  placeholders — their quota is still ticking — collapsed behind one
  line and dismissable. A live credential is never hidden.
- **Claude usage refresh (⟳)** — the only affordance that reaches the
  network, on click, never on a timer, and never running the OAuth
  refresh grant. It exists because nothing else can produce the
  per-model windows: the statusLine feed has none, and nothing qhud can
  run refreshes Claude's on-disk cache.
- **Codex per-workspace quota**, click-triggered. One login can own
  several workspaces with separate pools.
- **Diagnostics that make wrong output visible from outside the
  webview**, since `scrot` cannot capture XWayland-composited windows
  (D-010): `QMONSTER_SIDEFILE_DIAG=1` names which of three silent
  attribution declines fired; the widget reports the structure, the
  rendered text, and the gauge counts it actually built; and
  `--claude-usage` / `--codex-usage` / `--fetch-codex` /
  `--refresh-claude` expose each path without synthesizing pointer
  input.

### Changed

- **"No network" becomes "passive by default, network only on request"**
  (D-014). The 2 s poll loop still opens no socket and touches no
  credential. Binary budget raised 20 → 25 MB; actual 22.9 MB.

### Known limits

- **One live login per provider.** All three store a single active
  credential; `codex login` revokes the previous token, so parked
  credential files answer 401. Showing two accounts of one provider at
  once is not possible through the CLI-credential path.
- **Codex will not re-scope a token to another workspace.**
  `chatgpt-account-id` is ignored; a body describing a different
  workspace is dropped rather than mislabelled.
- **Per-model windows are only current right after ⟳.**
- Wire `plan_type` values are not display names (`prolite` is shown as
  ChatGPT Pro 5x, `team` as ChatGPT Business). Display names come from
  the registry and must never be "corrected" from a wire value.

## [0.3.2] — 2026-08-06

### Fixed

- **Expired-window quota poisoning** (preemptive): the max-snapshot
  rollup let an idle pane's pre-reset percentage outrank the fresh
  post-reset reading forever. Snapshots whose reset instant has passed
  are now excluded (90 s grace); if every snapshot is expired the
  window is omitted rather than shown wrong. Unit-tested.

### Added

- `qhud --dump` — one-shot diagnostic printing the exact payload the
  widget renders (pretty JSON), for "is this number right?" moments.
  Documented in the RUNBOOK.

## [0.3.0] — 2026-08-06

### Added

- **Font zoom**: Ctrl+wheel over the widget scales the whole UI
  (70–160%), persisted across restarts (D-012).
- **Layer peek**: tray "Pin above windows" check item and
  `qhud --peek` (relayed to the running instance) flip the widget
  above all windows and back — bind a GNOME custom shortcut to
  `~/.local/bin/qhud --peek`. Footer shows `pinned ·` while above.
- **Single-instance guard**: launching qhud twice no longer stacks a
  second widget (tauri-plugin-single-instance).

### Fixed

- First peek design (SIGUSR1) segfaulted the webview — WebKitGTK's
  JavaScriptCore reserves SIGUSR1 for thread suspension; signals are
  now documented as off-limits in this codebase (D-012).

## [0.2.0] — 2026-08-06

### Changed

- **Information architecture rebuilt around fact scope** (D-011, from
  operator feedback): 5h/7d quota windows are account facts and now
  render once per provider in a quota strip (freshest-snapshot rollup
  — max percent wins, since usage within a window only grows). Tiles
  show pane facts only (status + CTX + expanded detail) and gain a
  `@workspace` badge so identical labels are distinguishable. Top-bar
  summary simplified (quota lives in the strip).
- Payload schema v1 additive fields: `quotas[]`, `panes[].session`.
- README screenshots regenerated for the new layout.

## [0.1.4] — 2026-08-06

### Fixed

- **Real-mouse interaction was still dead on Ubuntu GNOME** while every
  synthetic test passed: Ubuntu's Desktop Icons NG (DING) extension
  window swallows real pointer input over the desktop layer, and
  XTEST-based verification bypasses Mutter's surface picking entirely
  — a methodology blind spot, now closed with compositor-path
  injection via the Mutter RemoteDesktop API (D-010). A/B proven:
  identical compositor-path clicks toggle selection with DING
  disabled and vanish with it enabled. Reference machine runs with
  DING disabled (its `~/Desktop` is empty); icon users: see RUNBOOK
  and the companion-extension backlog item.

### Added

- Permanent stderr interaction breadcrumbs (`qhud ui: …` via a
  `ui_event` command) — real-input behavior is now verifiable from
  logs on any machine.

### Changed

- `core:window:allow-set-title` capability removed again (debug
  beacons replaced by breadcrumbs).

## [0.1.3] — 2026-08-06

### Fixed

- **Tile selection (click-to-expand) never fired** (D-009): this
  WebKitGTK/X11 webview reliably delivers `pointerdown` but fails to
  synthesize `click` from the down/up pair, so the selection handler
  simply never ran. Selection now triggers on `pointerdown`
  (beacon-verified end-to-end: select → deselect → reselect with
  rendered state applied). localStorage was investigated and
  exonerated — writes work; the storage directory was missing only
  because the click handler had never executed. Persistence is now
  try/catch-wrapped and ordered after `render()` regardless, so
  storage can never gate the UI.

## [0.1.2] — 2026-08-05

### Fixed

- **Move, resize, and tile selection were dead in practice** (D-008).
  Four independent upstream quirks, each verified on-machine:
  compositor-side interactive move/resize (drag regions,
  `startResizeDragging`) no-ops for a keep-below XWayland window on
  GNOME; tao's invisible borderless-resize inset ate all pointer input
  within ~10px of the border — where the resize grip lived; WebKitGTK
  `screenX/Y` goes stale while the window moves; tao
  `outerPosition/outerSize` report a phantom ~37px frame. Geometry
  interaction is now fully self-driven: an rAF loop over the global
  `cursorPosition()` with DOM-derived grab metrics. Verified
  pixel-exact on both monitors.
- **Stale UI assets in rebuilds**: `tauri::generate_context!` does not
  register `../ui` with cargo's change tracking; `build.rs` now emits
  `cargo:rerun-if-changed=../ui`.
- **Window position/size lost on logout/kill**: the window-state
  plugin only persists on graceful exit; geometry is now checkpointed
  every 30 s from the poll loop.

### Changed

- Resize grip enlarged with an inner hit-zone that clears the tao edge
  inset; capabilities trimmed to exactly `cursor-position`,
  `set-position`, `set-size`, `window-state:default`.

## [0.1.1] — 2026-08-05

### Fixed

- **herdr rigs never went live**: v0.1.0 hardcoded the tmux polling
  source. qhud now builds its pane source through qmonster's own
  `build_tmux_source` factory, so `[mux] backend` (`auto` / `tmux` /
  `herdr`) means the same thing in both frontends (D-007).

### Added

- Widget-flavored `auto`: when qhud runs outside any mux pane (no
  herdr env inherited), it probes herdr first, then falls back to
  tmux.
- Payload schema v1 additive field `backend` ("herdr" | "tmux"),
  rendered in the footer (`live·herdr · poll 2s · …`).
- Live/demo transition logs on stderr with resolved backend and pane
  labels — greppable evidence for the TEST_PLAN checklist.

## [0.1.0] — 2026-08-05

First release — the smallest useful wedge.

### Added

- Desktop-layer widget window: keep-below + sticky + skip-taskbar via
  XWayland/EWMH (GNOME Wayland verified), frameless, transparent,
  drag-move, grip-resize, geometry persisted across restarts
  (`tauri-plugin-window-state`).
- qmonster observe bridge: links the qmonster crate at a pinned rev,
  polls `run_once_with_target` every 2 s with a no-write `NoopSink` +
  `SilentNotify`, shares `~/.qmonster/config/qmonster.toml` read-only.
- Widget UI ported from the reference mockup: pane tiles with status
  pills (active / done / wait / limit / stale / dead), CTX · 5H · 7D
  severity-banded gauges with reset countdowns, click-to-expand config
  chips (model / effort / flags / branch / cwd / mem / cost) and
  cross-pane conflict banner.
- Demo mode: mockup-parity payload with `DEMO` badge when no tmux
  server is reachable; live re-probe every 10 s.
- Tray icon (Show/Hide · Quit), best-effort.
- JSON payload contract schema v1 (`view.rs`) with unit tests.
- CI (fmt · clippy · test · build) and tag-driven release workflow
  (Linux x86_64 tarball + sha256 + build provenance attestation).
- Quetzalcoatl decision-doc set under `docs/`.
