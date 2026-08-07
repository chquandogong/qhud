# ARCHITECTURE — qhud

> Status: living (implemented through v0.4.0) · Date: 2026-08-07 · Owner: chquandogong

## The one-sentence shape

A Rust thread polls [qmonster](https://github.com/chquandogong/qmonster)'s
observation pipeline every 2 s, flattens the result into a small display-only
JSON payload, and emits it to a static WebKitGTK page that patches its DOM in
place. Everything else in this document exists to keep that loop **passive**:
it reads local files and the mux, and nothing else.

```
 mux (herdr | tmux)          local provider files            explicit click
        │                            │                             │
        ▼                            ▼                             ▼
  qmonster observe            accounts / registry          claude_usage
  (pinned rev)                usage_cache                  codex_usage
        │                            │                             │
        └──────────► view::payload ◄─┘                             │
                          │  (+ attach_*)                          │
                          ▼                                        │
                  Tauri event "qhud://report" ──► ui/app.js ◄──────┘
                          ▲                            │             (invoke)
                          └──── poll.rs, every 2 s ────┘
```

## Why the payload layer exists

`view.rs` re-serializes qmonster's rich `PaneReport` into a deliberately
small, stable, display-oriented shape. The webview never sees a qmonster type.
Two consequences worth keeping:

- **Upstream refactors touch one file.** qmonster is pinned by git rev
  (`src-tauri/Cargo.toml`); bumping it means re-running the contract tests in
  `view.rs`, not chasing types through the frontend.
- **Units convert once, at the boundary.** Pressures arrive as 0..1 fractions
  and leave as 0..100 integers; reset instants leave as unix seconds so the
  frontend owns countdown rendering and can tick between polls.

## Modules

| File              | Lines | Responsibility                                                                                                        |
| ----------------- | ----- | --------------------------------------------------------------------------------------------------------------------- |
| `main.rs`         | 282   | Tauri setup, tray, X11 window states, single-instance argv relay, `#[tauri::command]`s, and the diagnostic CLI flags  |
| `poll.rs`         | 193   | the 2 s loop: build a live context, run one observe tick, assemble the payload, emit; demo fallback and live re-probe |
| `view.rs`         | 697   | `PaneReport` → payload (schema v1). Owns the account-scoped quota rollup and the `attach_*` enrichment steps          |
| `accounts.rs`     | 428   | who is signed in, from local files only (D-013)                                                                       |
| `registry.rs`     | 298   | operator intent: display names, ever-connected accounts, dismissals                                                   |
| `usage_cache.rs`  | 267   | Claude's on-disk usage snapshot, and the parser shared with the live fetch                                            |
| `claude_usage.rs` | 112   | the one outbound request, on click                                                                                    |
| `codex_usage.rs`  | 536   | Codex per-workspace usage, on request                                                                                 |
| `demo.rs`         | 171   | the mockup fixture rendered when no mux is reachable                                                                  |
| `ui/app.js`       | 1264  | DOM patching, gauges, the collapsed placeholder list, and the render breadcrumbs                                      |

## The rollup: facts render at the scope where they are true

`view::provider_quotas` is the least obvious code in the repo, so its rules
are stated here rather than inferred.

Quota is an **account** fact, not a pane fact (D-011). Each pane holds a
snapshot of the same account-wide window, so per window the **max percent
wins**: usage only grows inside a window, making every snapshot a lower bound
and the largest one the freshest.

The exception is what makes it correct: a snapshot whose reset instant has
already passed belongs to an **expired** window. Its percentage is meaningless
now and must not outrank a fresh reading — otherwise an idle pane's 88% from
yesterday beats today's real 12% forever. Expired snapshots are excluded with
a 90 s grace, and if every snapshot for a window is expired the window is
omitted rather than shown wrong.

Enrichment is layered on afterwards by three `attach_*` functions, kept
separate so `payload()` stays a pure function of the reports:

1. `attach_usage_cache` — folds in Claude's on-disk snapshot. **A live pane
   reading is never overridden.** If Claude contributed no pane at all, a row
   is synthesized and marked `origin: "cache"`.
2. `attach_accounts` — stamps each row with the identity that owns it.
3. `attach_placeholders` — adds the ever-connected accounts that have no live
   credential.

## Credential posture

This is the part to read before changing anything that touches a provider.

**The poll loop opens no socket and reads no credential.** It reads identity
fields the CLIs already keep in cleartext beside their tokens — never the
token fields themselves.

| Provider         | How usage is obtained            | Who holds the credential       |
| ---------------- | -------------------------------- | ------------------------------ |
| Claude 5H / 7D   | statusLine JSON the CLI writes   | the CLI                        |
| Claude per-model | `GET /api/oauth/usage`, on click | **qhud, for one request**      |
| Codex            | `/wham/usage`, on request        | qhud reads `access_token` only |
| Antigravity      | the CLI's loopback Connect RPC   | nobody — no token needed       |

Rules that hold everywhere:

- **Never run an OAuth refresh grant.** Refresh tokens are single-use and
  rotated; racing the vendor CLI's own refresh is how a login breaks. On 401,
  say "sign in again".
- **Read credentials fresh per call**, never cache or copy them.
- **Never log a usage response body** — they carry account uuid and email.
- **Prefer delegating to the vendor CLI** over holding a token (Codex's
  `app-server` RPC and agy's loopback RPC are why those two never expose one).
  Claude has no delegated path — verified, not assumed — which is the only
  reason `claude_usage.rs` exists.

Machine-specific account data lives in `~/.config/qhud/accounts.json`,
**outside this repository**: it holds emails and account ids, and the repo is
public.

## Provider quirks that shaped the code

Each of these cost a debugging session; they are recorded so the next change
does not re-earn them.

- **MCP plugin children masquerade as the pane's CLI.** herdr's pane-level
  `foreground_cwd` follows whichever descendant holds the foreground, and
  herdr can report the _agent process itself_ as `pane_pid` — which the
  descendant walk skips by design. Both broke sidefile attribution, silently
  dropping cost and reset windows. Fixed upstream in qmonster; a mismatch is
  now attributable via `QMONSTER_SIDEFILE_DIAG=1`.
- **Sidefiles must be written temp+rename.** Truncate-in-place lets the 2 s
  poller read a torn file.
- **Codex ignores `chatgpt-account-id`.** It will not re-scope a token to
  another workspace, so `parse_usage` compares the body's own `account_id`
  against the request and drops a mismatch. A missing reading is honest; a
  mislabelled one is not.
- **Never infer a Codex window from its lane.** `primary`/`secondary` changed
  meaning mid-2026; derive the label from the window duration.
- **`#[serde(default)]` does not cover an explicit `null`.** A null
  `additional_rate_limits` failed an entire parse and was reported as "no
  data".
- **Wire plan values are not display names.** `prolite` is _ChatGPT Pro 5x_,
  `team` is _ChatGPT Business_. Display names come from the registry only.

## Desktop-layer constraints

GNOME on Wayland exposes no layer-shell to third-party apps, so the
desktop-widget layer exists only through XWayland; `GDK_BACKEND=x11` is forced
before any GTK code runs (opt out with `QHUD_NO_X11_FORCE=1`). X11 window
states only stick once the window is realized, so keep-below, sticky and
skip-taskbar are re-asserted after setup and after every layer flip.

Three hard-won prohibitions:

- **Never install Unix signal handlers** (D-012). WebKitGTK's JavaScriptCore
  reserves SIGUSR1 for thread suspension; hooking it segfaults the process on
  the first signal. Peek is therefore an argv relay through
  `tauri-plugin-single-instance`, which doubles as the trigger channel for
  `--peek`, `--refresh-claude` and `--fetch-codex`.
- **All geometry is self-driven** (D-008). Compositor interactive move/resize
  ops are not used.
- **Bind to `pointerdown`, not `click`** (D-009). A keep-below widget does not
  reliably receive synthesized click pairs — which is also why every network
  path has a CLI trigger.

## Verifying it works

The pixels are **not** verifiable from outside the webview: `scrot` cannot
capture XWayland-composited windows (D-010). So the widget reports what it
built, to stderr — the structure it rendered, the text of every row, gauge
counts, and any frontend exception routed through `ui_event`.

**The absence of an error is not proof anything painted.** That distinction is
load-bearing: a render-time `ReferenceError` once stopped the quota strip
entirely while the Rust payload looked perfect. Counts alone are also
insufficient — they are blind to wrong _text_, which is why the rendered
labels are emitted too.

Diagnostic surface: `--dump` (the exact payload), `--claude-usage` /
`--codex-usage` (each fetch, standalone), `QMONSTER_SIDEFILE_DIAG=1` (which of
three silent attribution declines fired).

## Boundaries held on purpose

- **Observe-only**: `NoopSink` — the qmonster TUI owns `~/.qmonster` and a
  second sqlite writer would race it (D-004).
- **Silent**: `SilentNotify` — the TUI and the providers already alert; a
  widget that also notified would double-fire everything.
- **No bundler**: the frontend is static HTML/CSS/JS, no `node_modules`.
- **Passive by default, network only on request** (D-014).

Requirements and the full payload contract: [SPEC](SPEC.md) ·
Decisions and their reasoning: [DECISION_LOG](../02-decisions/DECISION_LOG.md)
