# ARCHITECTURE — qhud

> Status: living (rewritten from scratch against v0.6.0 source) · Date:
> 2026-09-07 · Owner: chquandogong
> Companion to SPEC (what it must do) and DECISION_LOG (why it is built this
> way). Every claim here was read off the tree at `v0.6.1`; line counts and
> test counts are from that tree.

## 1. What qhud is, structurally

A single-process Tauri v2 desktop application: a Rust core that observes and
fetches, and a bundler-free web frontend that renders. It is the **second
frontend** for qmonster's observe pipeline (D-001), linking that crate as a
library at a pinned git revision and never writing to its state.

Two platforms, one codebase:

|                | Linux (reference)                                      | Windows x64                                      |
| -------------- | ------------------------------------------------------ | ------------------------------------------------ |
| Webview        | WebKitGTK via GTK 3                                    | WebView2 (Edge runtime, required)                |
| Desktop layer  | XWayland + EWMH `_NET_WM_STATE_BELOW`/`STICKY` (D-003) | native always-on-bottom                          |
| Terminal panes | tmux or herdr through qmonster (D-007)                 | **not implemented** — account quota only         |
| Frame guard    | active (GTK pixel sampling, D-017)                     | compiled out; WebView2 has no equivalent failure |
| Dependency     | plain git pin, no sibling checkout                     | command-scoped patch of a sibling checkout       |

The Windows port added no second code path for data: providers, parsing,
merging and rendering are shared, and only path resolution, process spawning
and the frame guard are platform-gated.

## 2. Process and thread model

`main()` runs, in order:

1. **Linux-only environment forcing**, before any thread exists. Three
   set-if-unset pairs, each with an escape hatch: `GDK_BACKEND=x11`
   (`QHUD_NO_X11_FORCE`), `WEBKIT_DISABLE_DMABUF_RENDERER=1`
   (`QHUD_KEEP_DMABUF`), `WEBKIT_DISABLE_COMPOSITING_MODE=1`
   (`QHUD_KEEP_COMPOSITING`). Any value counts as set, including empty. The
   whole block is absent on Windows.
2. **Argv pre-flight.** The diagnostic and relay flags are handled here,
   before Tauri or GTK initializes, so a diagnostic run never creates a
   window. `--respawned` sleeps 1.5 s first so a re-exec is not swallowed by
   the single-instance guard.
3. **Tauri builder**, in registration order: the single-instance plugin (which
   doubles as the argv-relay channel), the five commands, the window-state
   plugin, then `setup`.
4. **`setup`**: fetch the `main` window (a hard panic if absent — the label is
   a build-time contract), re-assert the layer states that `tauri.conf.json`
   already requested (X11 only honors them once the window is realized), build
   the tray (failure is non-fatal and logged), and spawn the poll thread.
5. `run()` — the main thread becomes the GTK or WebView2 event loop.

Exactly one thread is spawned: the poll loop. Everything touching the window
runs on the main thread, including the frame guard, which marshals its sample
through `run_on_main_thread`. The `fetch_*` commands are `async` and run on
Tauri's runtime; the standalone diagnostic flags each build their own Tokio
runtime and exit.

## 3. Tray and the layer state machine

The tray menu is `Show / Hide`, `Pin above windows` (a checkbox), `Reset
position`, `Quit qhud`. Pin state lives in one process-wide atomic: false is
the desktop layer (keep-below, the default), true is pinned above.

Applying a layer always re-asserts sticky-on-all-workspaces, skip-taskbar and
visibility alongside the bottom/top flip, then emits `qhud://layer` and logs
`qhud ui: layer:pinned` or `qhud ui: layer:below`. The frame guard calls the
same re-assertion after a heal, so a remap never silently drops the operator's
pin choice. `Reset position` exists because a monitor unplug can leave the
geometry restore pointing at a gone display (CV-1).

## 4. The poll loop

Cadence 2 s. Per tick, in order: bump the counter; every 15th tick checkpoint
the window geometry (the window-state plugin only persists on a graceful exit,
and a widget usually dies by signal or logout); probe for a live multiplexer if
none is attached and the last attempt is at least 10 s old; observe; select and
enrich the payload; emit `qhud://report`; run the frame guard; sleep.

**Backend selection** (D-007). qmonster's own `auto` detects herdr from
environment variables that exist only _inside_ a herdr pane, and qhud normally
runs outside every pane. So when the config says `auto` and neither
`HERDR_ENV` nor `HERDR_SOCKET_PATH` is present, qhud builds two candidates and
tries **herdr first, then tmux**; an explicit `[mux] backend` is used as
given. A candidate must survive one validation observe tick before it is
accepted, and the reported backend label comes from the resolved source rather
than from the config that asked for it. Config is read from
`~/.qmonster/config/qmonster.toml`; a parse failure logs and falls back to
serde defaults.

**No multiplexer is no longer a demo.** Before v0.6.0, a mux-less start
rendered the demo fixture. Now it renders a real local payload: zero panes,
`source = "local"`, plus every locally discoverable account identity and the
dated snapshots qhud has saved, with missing usage left missing rather than
zero-filled. Example data requires `--demo`, which also disables live probing
entirely. Two tests hold this line: one asserting no invented usage with no
mux, one asserting the default account stays reachable when only an extra
account has a row.

Enrichment order is fixed: detect accounts, attach saved snapshots, attach
accounts, attach identity-only rows, attach placeholders, then recompute the
summary's five-hour maximum. Identity-only rows exist so that an account with
no saved reading still renders — and therefore still has a reachable refresh
control.

**Ownership gate.** A saved snapshot is used only if its account matches the
current login, by account id or email. This prevents a fresher reading from a
previous login masking this login's older one — the failure v0.6.0 closed.
Saved Codex workspaces likewise have their `active` flag recomputed against
the current account rather than trusted from disk.

**Degradation.** No config falls back to defaults; one backend failing falls
through to the next; all failing falls back to the local payload and retries
every 10 s; a live source lost mid-run falls back to the local payload while
keeping real accounts and saved usage; emit and geometry-save failures are
ignored by design.

## 5. Paths

`paths.rs` (added in v0.6.0) is the single place that resolves where anything
lives, and every lookup treats an empty environment variable as unset.

| What               | Resolution order                                                                                                                                                                      |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Home               | Windows: `USERPROFILE` then `HOME`. Elsewhere: `HOME` then `USERPROFILE`.                                                                                                             |
| qhud config        | `QHUD_CONFIG_DIR` verbatim → `XDG_CONFIG_HOME/qhud` (both platforms) → on Windows, the legacy `~/.config/qhud` if it already exists, else `APPDATA/qhud` → otherwise `~/.config/qhud` |
| Codex home         | `CODEX_HOME` → `~/.codex`                                                                                                                                                             |
| Claude config dir  | `CLAUDE_CONFIG_DIR` → `~/.claude`                                                                                                                                                     |
| Claude config file | `CLAUDE_CONFIG_DIR/.claude.json` → `~/.claude.json`                                                                                                                                   |

The Windows home preference is deliberate: a Git-Bash-style `HOME` must not
beat the native profile. The Windows config-dir order is deliberate too — an
existing `~/.config/qhud` keeps working rather than being orphaned by a move
to `APPDATA`.

Other environment variables the core reads: `QHUD_CODEX_BIN` and
`LOCALAPPDATA` plus `PATH` for Codex CLI discovery, `SystemRoot` for the
Windows helper process, `QHUD_EXTRA_DIAG` for identity-free Claude response
diagnostics.

## 6. Frame guard (Linux only, D-017)

Every 14th poll tick — about 28 s — the widget hashes a strip of its own
window: 24 pixels tall at the bottom edge, at most 420 wide, read from the
GdkWindow. The footer clock there repaints every second, so a byte-identical
strip means the compositor is not presenting frames.

The ladder is a two-rung escalation with a reset: a changed hash clears any
ladder in progress; one static sample hides and re-shows the window and
re-asserts the layer states; a second static sample re-execs the process with
`--respawned`; a third does nothing, because a restart is already in flight. A
hidden window resets the state instead of tripping it, because static pixels
are correct when nothing is visible. The first successful sample of a session
logs `qhud: frame guard armed (first pixel sample ok)` — proof that the
sampler itself works, so a silently broken sampler cannot be mistaken for a
healthy screen.

The hash is FNV-1a 64-bit with one adjustment: a result of zero is remapped to
one, because zero is the reserved "no sample yet" sentinel.

Non-Linux builds compile the whole mechanism out; the decision logic stays
unit-testable on every platform.

## 7. Window and capability surface

The window is 392×520, undecorated, transparent, always-on-bottom,
skip-taskbar, visible on all workspaces, resizable, minimum 320×200. No
position is declared — geometry comes from the window-state plugin. The
frontend runs with `withGlobalTauri` because it has no bundler, and the
content-security policy allows only self, inline styles and data-URI images.

Capabilities are deliberately minimal and enumerated: the defaults plus
cursor position, webview zoom, set position and set size, and the window-state
plugin's own set. Everything the widget does to itself — drag, resize, zoom —
is done through those four window permissions rather than by asking the
compositor (D-008).

`bundle.active` is false: releases ship the raw binary in an archive, not an
installer (D-006).

## 8. Build and release

One CI workflow, two platform jobs, on every push to `main` and every pull
request:

- **Ubuntu 24.04**: `cargo fmt --all --check`, `cargo clippy --locked
--all-targets -- -D warnings`, `cargo test --locked --all-targets`, `cargo
build --locked --release`.
- **Windows 2022**: the Windows build script in test mode, then a
  `git diff --exit-code` on `Cargo.toml` and `Cargo.lock` to prove the
  canonical dependency files were restored byte for byte.

The release workflow fires on any `v*` tag, runs both platform builds again
with their tests, packages a Linux tarball and a Windows ZIP each with a
SHA-256 sidecar and a build-provenance attestation, and publishes only after
both jobs pass. The publish step requires `docs/05-ops/releases/<tag>.md` to
exist — a release without written notes fails rather than shipping unexplained.

**The Windows dependency patch** exists because the pinned qmonster revision
spawns console windows when hosted by a GUI process and has tests that assume
Unix. The patch adds one helper that applies `CREATE_NO_WINDOW` on Windows and
nothing elsewhere, routes every process-probe site through it, and gates
Unix-only tests. It is applied only to a sibling checkout outside the qhud
tree, wired in through a temporary `cargo --config` patch table that is deleted
afterwards, with one validated `Cargo.lock` line removed and restored byte for
byte in a `finally` block. A plain Linux `cargo build` therefore resolves
qmonster from git with no sibling checkout and no patch.

The release profile — `strip`, `lto`, `codegen-units = 1` — lives in the
workspace root manifest. It sat in the member manifest until v0.6.0, where
cargo ignores it, which is why every release before v0.6.0 shipped an
unoptimized-size binary: 25.13 MiB for v0.5.3 against 15.07 MiB for v0.6.0.
Cargo printed that warning on every build for a month.

## 9. The payload contract

One event, `qhud://report`, carries everything the frontend knows. The core
re-serializes qmonster's rich pane reports into a small display-oriented
payload, so the webview never sees upstream types and an upstream refactor
touches one file. Two conversions are fixed by that boundary: pressure arrives
as a 0..1 fraction and leaves as an integer percent, and reset instants leave
as unix seconds so the frontend owns countdown rendering.

Schema version is 1 and has stayed 1 through every release: fields have only
ever been added, and additive fields that are usually empty are omitted rather
than sent as null.

```
schema, source, backend, generated_at_ms, poll_secs
quotas[]   provider, h5, d7, from_label, session, account?, origin?,
           cache_fetched_at_ms?, scoped[]?, extra?
panes[]    pane_id, label, session, provider, status, status_label,
           elapsed_secs, cli_version, update_hint, model, effort, branch,
           cwd, mem, cost_usd, flags[], gauges{ctx,h5,d7}, conflicts[]
summary    panes, conflicts, max_5h_pct
account_placeholders[]?, workspace_names{}?, workspace_plans{}?,
codex_workspaces[]?, codex_fetched_at_ms?
```

`source` is `live`, `local` or `demo`. `backend` is `herdr` or `tmux` when a
multiplexer feeds the payload and null otherwise. A gauge is `{pct, source,
reset_unix, of_tokens}`; `origin` is `pane`, `cache` or `fetched` and is the
field the frontend uses to decide whether a row may look live at all.

`qhud --dump` prints this exact payload as pretty JSON, which is why the
runbook answers "do you doubt a number?" with that command rather than with a
screenshot.

## 10. Merge rules

These rules exist because the same fact reaches qhud from up to three places
at different ages, and the widget must never present the stale one as current.

**Per-provider rollup.** Quota is an account fact, not a pane fact (D-011), so
panes of one provider collapse into one row and, per window, the highest
percent wins: usage only grows inside a window, so every pane's reading is a
lower bound and the maximum is the freshest. The pane that won the five-hour
window also supplies the row's attribution; the seven-day winner supplies it
only if the five-hour window is absent.

**Expired snapshots are excluded, with 90 seconds of grace.** A reading whose
reset instant has passed describes a window that no longer exists, and an idle
pane's 88% from yesterday would otherwise outrank today's real 12% forever.
The grace exists because a reading taken moments before its own reset is still
honest about the new window for a beat. If every reading for a window is
expired the window is omitted entirely — showing nothing beats showing a lie.

**A live pane reading is never overridden.** A stored snapshot may fill a
window the pane does not have, attach per-model windows and spend that only
the snapshot carries, and date itself — but it may not replace a number the
pane reported. Filling a hole is not overriding.

**A provider with no pane still gets a row**, synthesized from the snapshot
and stamped with that snapshot's origin. This is the entire point of the
persistence layer: the numbers survive with no CLI running. Since v0.6.0 the
bar for synthesizing is lower — a snapshot carrying only model-specific pools,
or only extra spend, is still a real reading and now produces a row instead of
being discarded for lacking a primary window.

**Ownership before age.** A saved snapshot is used only if it belongs to the
account currently signed in. Comparing ages first is how a fresher reading
from a previous login could mask this login's older one, which v0.6.0 fixed.
Saved Codex workspaces have their active flag recomputed against the current
login for the same reason.

**One account's numbers never wear another's name.** Only rows that do not
already carry an identity get one attached, because an extra-account row
arrives with its own; and the active Codex workspace merges into the provider
row rather than rendering a second, clock-skewed copy of the same weekly
window.

## 11. Window vocabulary

Every window is named by its duration — 5H, 1D, 7D, 30D — everywhere in the
interface, translated in exactly one table in the frontend. The wire says
"weekly" and "30d" on Codex windows, and the operator rightly asked whether
"weekly" and "7D" were two different things; they are one seven-day rolling
window, so they carry one name.

On the wire, a Codex window's label is derived from its **duration**, never
from its primary/secondary lane. That was verified on the reference machine:
before 2026-07-13 primary was the five-hour window and secondary the weekly
one; afterwards primary became weekly, secondary went null, and a thirty-day
window appeared. Codex itself labels by duration and so does qhud, with a ten
percent tolerance and a minutes fallback for anything unrecognized.

A per-model pool has the same duration as its primary pool, so the scope
**name** is the only thing distinguishing two 7D chips. Scope is therefore
carried end to end from each provider's own wire field into the payload and
into the rendered label, and an unknown pool kind degrades to a tooltip line
rather than vanishing.

## 12. The frontend

No framework and no bundler: the DOM is patched in place so a 2 s refresh
never restarts a CSS transition, and the whole app is one file served through
Tauri's asset protocol with `withGlobalTauri`.

**Render order** is summary, demo badge, refresh-all state, then the quota
strip (sections, rows, Codex sub-rows, collapsed placeholder rows) inside a
try/catch that routes any exception to stderr, then the tiles as a keyed patch
in payload order, then the footer. A frontend exception used to be invisible
from outside the webview, which is how a broken strip once shipped.

**Row identity in the strip is (provider, account, organization)** — the same
key the core dedupes on (D-018) — and a new row is inserted after that
provider's last existing row so a second account never drags a section header
out of place. Provider is the outer axis because it is what the operator picks
when deciding where to run the next task, it matches the pane vocabulary, and
there are exactly three of them, so the scan path stays stable as accounts
multiply.

**Age and origin are visible, always.** Only a snapshot-origin row wears an
age caption; on a live row the snapshot merely contributed per-model windows,
so badging it would wrongly imply the visible gauges are stale. Per-model
chips from a snapshot are dimmed and marked, because a scoped gauge can only
come from a cache and must not read as current — the v0.4.0 bug where a cached
5% sat beside live gauges while the truth was 22%.

**Three independent fetch state machines**, one per provider, each idle →
loading → done or error, each with its own failure surface, so one provider's
401 never hides another's numbers. The refresh-all control mirrors their union.
Live results are matched to a row by config directory **and** account id;
v0.6.0 added the account-id conjunct so a refresh cannot land on the wrong row.

**Selection expands a row inline.** Hover tooltips barely exist for a
keep-below window you are rarely pointing at, so the facts move into the row on
click — and network runs only from the refresh controls, never from selecting a
row. Every interaction binds `pointerdown`, never `click`, because this webview
delivers pointerdown reliably and does not synthesize click (D-009).

**Geometry is self-driven** (D-008). Compositor-side interactive move and
resize are unreliable for a keep-below XWayland window, and this webview's
screen coordinates go stale while the window itself moves, so a
requestAnimationFrame loop reads Tauri's global cursor position and sets
position or size directly; pointer events only arm and disarm the loop. The
window's own reported position and size are deliberately unused because the
toolkit mis-reports both by a phantom frame height for an undecorated X11
window. Ctrl+wheel zooms 70–160% and persists; the widget never takes keyboard
focus.

**Observability is deliberate.** Breadcrumbs go to stderr through one command
and are the only proof available that anything rendered: what the strip built,
the rendered text of every identity line and gauge label, every real
pointerdown with where it landed, selection changes, zoom, and any JavaScript
error. Counts alone are not enough — a wrong window _name_ is exactly the bug
a count cannot catch — and none of it proves paint, which is why the frame
guard exists.

A one-second ticker re-renders countdowns and marks the footer stale if no
payload has arrived for eight seconds, so a stopped backend says so instead of
freezing plausible numbers (CV-2).

**What is persisted where.** Selected pane, placeholder-expansion state and
zoom live in browser storage behind a wrapper that tolerates being denied
outright; window geometry lives in the window-state plugin, checkpointed every
30 s by the poll loop.

## 13. Demo as a parity fixture

The demo payload mirrors the original design mockup pane for pane, so
rendering it identically to the mockup is the visual-parity test (D-005). It
carries three panes, one conflict, and computes its quota rows through the
real rollup, so demo mode exercises the production merge path rather than a
parallel one.

Since v0.6.0 it is opt-in: `--demo` is required, and it also disables live
probing. An empty pane list is no longer a reason to show sample data, because
a mux-less machine can still show real identities and the dated results of the
operator's last refresh.

## 14. Providers

Two questions decide everything about a provider path: does the fact arrive
without asking, and who holds the credential.

|        | Passive, every 2 s                                                                               | On an explicit refresh                                                                                                                                  | Credential custody                                                                         |
| ------ | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Claude | statusline through qmonster; Claude Code's own on-disk usage cache; identity from `.claude.json` | `GET https://api.anthropic.com/api/oauth/usage`, once per signed-in config dir                                                                          | qhud, for the length of one request — the only such case                                   |
| Codex  | pane statusline for the active login; identity from `auth.json`                                  | `wham/accounts/check` then `wham/usage` per saved credential; on failure of the active login, a short-lived `codex app-server` child answers over stdio | qhud reads an access token for the HTTP path; the CLI owns everything on the fallback path |
| agy    | pane statusline when a pane exists; identity from `google_accounts.json`                         | a loopback Connect RPC to the running agy process, no token and no CSRF                                                                                 | agy itself — qhud reads no credential at all                                               |

Three rules hold across all of them (D-014, D-016):

- **Never on a timer.** The poll loop reads local files and the multiplexer.
  Every byte that leaves the machine leaves because the operator asked.
- **Never run a refresh grant.** Codex refresh tokens are single-use and
  rotated; failing to persist a rotation breaks the operator's login. Claude's
  is not raced either, because racing Claude Code's own refresh is what breaks
  a login.
- **Prefer delegation.** Where a provider's own process can answer, let it.
  That is what makes token rotation and keyring-held credentials someone
  else's problem.

**Claude.** The credential is read fresh per call, only the access token,
never the refresh token beside it. The user agent is load-bearing: without a
real `claude-code/<version>` this endpoint answers 429 with no retry hint.
Identity comes from the config directory's own file, never from the response,
because the response carries an account uuid and an email and is never logged
— the single exception is an environment-gated diagnostic that prints two
identity-free sub-objects for shape drift. One refresh walks the default
account and every registered extra config directory, records each under its
own store key, and stays partial on partial failure: an expired extra account
must not hide the default's numbers. A rejected body reports serde's own
field-and-type message, which is identity-free by construction, because "did
not parse" with no field name is a two-day diagnosis (D-019).

**Codex.** Every `auth.json` and parked sibling in every registered home is
read, deduped by account id with the live file sorted first so it wins. Each
credential reads its own workspace: the server refuses to re-scope one token
to another workspace, so a body that describes a different account than the
one requested is dropped rather than mislabelled, with the mismatch logged.
Window labels come from the window's **duration**, never from its
primary/secondary lane. When the active login's HTTP path fails, a
`codex app-server` child is spawned read-only with on-request approvals,
handed a three-line JSON-RPC handshake over stdio with stdin deliberately held
open — the server exits on EOF, possibly before answering — and given 25
seconds before it is killed. Since v0.6.0 the per-limit map may be absent,
null or empty and the top-level pool may be null; either alone is enough, the
main pool is consumed exactly once so one weekly window is never rendered
twice, and a null top-level value cannot hide real per-model readings.

**agy.** A running agy binds a loopback listener that answers the quota
summary with no token and no CSRF. Ports are discovered from the operating
system's own tables: on Linux by joining the agy process's socket inodes
against the loopback listeners in `/proc/net/tcp`, on Windows by a hidden
PowerShell query whose script returns nothing but port numbers. Candidates are
tried in order; the HTTPS listener rejects plain HTTP and so excludes itself.
The wire format is proto3 JSON, which omits default values — an absent
remaining fraction means nothing is left, never "unknown, call it fine". The
reading is stamped with the identity present when the refresh began, so a
later sign-in cannot relabel a saved snapshot.

## 15. Parsing and the snapshot shape

One shape — a snapshot with two primary windows, a list of scoped windows and
optional spend — carries every provider's reading, so the renderer needs no
second code path. Claude's on-disk cache and qhud's own refresh results parse
into it directly; Codex workspaces and agy pools are mapped into it.

Three parsing rules earn their place:

- **A missing percent is "no such limit", not zero.** A plan without a window
  reports null, and rendering that as 0% would claim nothing is used when the
  truth is that no such cap exists. Limits missing a kind or a percent are
  dropped rather than zeroed, and a model absent from the current response is
  removed from the display rather than shown at 0%.
- **Integer-meaning numbers are read for their meaning** (D-019). An integral
  float is that integer; a fractional float is dropped as unit-ambiguous,
  because guessing a scale could show fifty dollars as fifty cents. Neither
  outcome may fail the surrounding body.
- **Money never round-trips as a float.** Spend is minor units plus a currency
  and an exponent, and the richer of the two wire objects wins field by field
  rather than either/or, because one of them is the only source of the
  limit-reached flag.

When both Claude Code's own cache and qhud's last refresh exist, the fresher
wins, and a tie goes to qhud's own read because it is at least as complete.
Either way the row is dated with which source it came from.

## 16. Identity

Identity is read from files the CLIs already keep in cleartext, with zero
network and without ever opening credential material: Claude's config file,
Codex's `auth.json` account id, agy's active Google account. A provider that
is logged out contributes no row at all, and any unreadable or malformed file
is simply an absent account — this is enrichment and must never fail a tick.

**A row's identity is the pair (account, organization)** (D-018). One
claude.ai login can hold a team seat and a personal organization, and those
are separate quota pools; only the same account in the same organization is a
duplicate. The frontend keys rows the same way and matches a refresh result to
its row by config directory **and** account id, because matching on the
account alone would feed one organization's numbers to both rows of one login.

Display names are operator-supplied and layered on top from a registry file
that lives outside this repository. They are never "corrected" from a wire
plan string: those are the provider's internal words, not the name the
operator sees.

## 17. Registry and placeholders

One operator-owned file holds display labels, workspace names and plans, the
list of accounts that have ever connected, dismissals, and the extra Claude
config directories and Codex homes that make multi-account work (D-015).

Three placeholder rules, in priority order:

1. A live credential always shows. Present facts are never hidden.
2. A known account with no live credential shows as a placeholder carrying a
   hint on how to restore it — its quota is still ticking, so hiding it would
   be a lie of omission.
3. An account the operator explicitly dismissed never shows as a placeholder.
   Dismissing does **not** hide a live account; log out for that. Silently
   hiding something in active use is the worse failure.

qhud writes exactly one key in that file, the dismissal list, and it edits the
file as generic JSON rather than through a typed struct so hand-maintained
keys survive. Malformed content reads as an empty registry: none of it is
load-bearing enough to fail a tick over.

## 18. Persistence

Two files, both outside this repository because they carry account ids, both
written temp-then-rename because the poll loop re-reads them every tick and a
truncate-in-place writer is exactly how the statusline sidefiles once produced
torn reads.

- `accounts.json` — the operator's registry, described above.
- `fetched-usage.json` — the results of explicit refreshes: the default Claude
  account, each extra Claude account keyed by its config directory, the Codex
  workspaces with their fetch time, and agy. Deleting it is always safe; the
  next refresh rebuilds it.

Junk, an old schema or a missing file all read as an empty store, and unknown
fields are ignored so a store written by a newer build does not break an older
one.

**Concurrent refreshes cannot erase each other.** The three provider refreshes
run at once, and before v0.6.0 each one loaded, mutated and saved
independently, so two finishing together could each write a store built from a
pre-merge read. A process-wide lock is now held across load, merge and atomic
replacement. A regression test drives eight barrier-synchronized writers and
asserts all eight survive.

qhud writes nothing under qmonster's own directory. The TUI owns that state,
and a second writer would race it (D-004).

## 19. Invariants

The rules below are the ones that took a field failure to learn. Each is
enforced in code and asserted by tests.

- **A stale number must never pass for a current one.** Snapshot-origin rows
  wear their age; per-model chips from a cache are dimmed and marked; an
  expired window is omitted rather than shown. The bug behind this rule: a
  cached 5% for one model rendered beside live gauges while the truth was 22%.
- **Absence of an error is not proof anything painted.** Breadcrumbs prove
  logic, never pixels; the pixel guard exists because that distinction once
  cost three days (D-010, D-017).
- **One fact, one name; one name, one fact** (D-011). Every window is named by
  its duration everywhere, scope names travel end to end, and a per-model pool
  is never collapsed into the primary pool it shares a duration with.
- **Missing stays missing.** No zero-fill, no invented pools, no synthesized
  0% for a model the provider did not mention.
- **Partial failure stays partial.** One provider's 401, or one expired extra
  account, never hides another's numbers.
- **The poll loop is passive.** Local files and the multiplexer only. Network
  and RPC live behind explicit controls and their command-line twins.
- **Ownership before age.** A snapshot is only used for the account it belongs
  to.
- **No refresh grants, ever.** And no credential material is opened for
  identity.

## 20. Module map

Line and test counts are from the `v0.6.1` tree; the test column sums to the
98 the suite reports.

| File | Role | Lines | Tests |
| --- | --- | --- | ---: |
| `src-tauri/src/main.rs` | Entry point, platform env forcing, argv surface, the five commands, tray, layer state machine, respawn | 421 | 0 |
| `src-tauri/src/poll.rs` | The 2 s loop: backend selection, observe tick, local fallback, snapshot and identity attachment, dump | 501 | 6 |
| `src-tauri/src/paths.rs` | Every path and environment override, Linux and Windows | 84 | 2 |
| `src-tauri/src/view.rs` | qmonster reports to payload; the rollup and every merge rule | 1154 | 19 |
| `src-tauri/src/accounts.rs` | Local identity discovery, (account, organization) dedupe, label overlay | 561 | 14 |
| `src-tauri/src/registry.rs` | The operator's registry, placeholder rules, dismissal writes | 342 | 10 |
| `src-tauri/src/usage_cache.rs` | The snapshot shape, Claude's on-disk cache, lenient wire numbers, freshness | 661 | 14 |
| `src-tauri/src/claude_usage.rs` | Claude explicit refresh, per config directory | 207 | 1 |
| `src-tauri/src/codex_usage.rs` | Codex workspaces, scope guard, duration labels, app-server fallback | 1146 | 20 |
| `src-tauri/src/agy_usage.rs` | agy loopback RPC and port discovery on both platforms | 401 | 5 |
| `src-tauri/src/fetched_store.rs` | Refresh persistence, atomic writes, concurrent-write lock | 268 | 4 |
| `src-tauri/src/frame_guard.rs` | Pixel sampling and the heal ladder (Linux) | 171 | 3 |
| `src-tauri/src/demo.rs` | The parity fixture | 173 | 0 |
| `ui/app.js` | The whole frontend: render pipeline, strip, tiles, fetch states, breadcrumbs, geometry, zoom | 1667 | — |
| `ui/style.css` | Widget styling, gauge severity, scrolling, staleness marks | 921 | — |
| `ui/index.html` | Skeleton | 36 | — |

Where to look first when something is wrong:

| Symptom | File |
| --- | --- |
| A number is wrong or too old | `view.rs` merge rules, then `poll.rs` attachment order |
| A refresh fails | the provider's own module; the error names the field or the credential |
| A row is missing or duplicated | `accounts.rs` dedupe, then the frontend's row key |
| Nothing renders, or renders stale | `frame_guard.rs` and the `xwd` check in the runbook |
| A path is wrong on one platform | `paths.rs` |
