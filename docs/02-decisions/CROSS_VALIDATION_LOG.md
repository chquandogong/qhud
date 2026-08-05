# CROSS_VALIDATION_LOG

> Status: done · Date: 2026-08-05 · Owner: chquandogong

## Session 1 — architecture decision (pre-ship)

- **Question**: Is "Tauri v2 + forced XWayland keep-below + direct
  qmonster lib reuse with a no-write sink" the right architecture for
  qhud v0.1 on GNOME 46 Wayland?
- **Proposer**: Claude (Fable 5), with two live spikes on the target
  machine as evidence.
- **Independent reviewer**: OpenAI Codex CLI 0.146 (GPT), thread
  `019fd114-4dbb-7522-ab71-e8a130634a49`, reviewing the pinned
  qmonster rev `aa2bd39` at source level. (Local command execution was
  sandbox-blocked for the reviewer; it verified via read-only GitHub
  access to the same rev — file:line citations included.)

### Verdict: **AGREE-WITH-CHANGES**

### Where the reviewer independently confirmed the design

- `Context::new(config, source, notifier, sink)` is the correct public
  construction path; struct literals are impossible (private fields) —
  matches implementation.
- Upstream `NoopSink` + a one-method custom `NotifyBackend` suffice;
  `Context::new` leaves every persistence sink `None`, so the loop is
  read-only w.r.t. `~/.qmonster` (verified against
  `event_loop.rs:362–654`) — matches implementation.
- **Never call `build_startup_runtime` / `with_anomaly_sink`** — those
  create the qmonster directory layout, open sqlite, run retention.
  qhud does not call them.
- `PaneReport` is not `Serialize` (and has a `pub(crate)` field) — a
  qhud-owned DTO is mandatory, not optional. Matches `view.rs`
  schema v1.
- Demo fallback must be explicit and watermarked, never silent —
  matches the `DEMO` badge + `source` field.
- Exact `rev=` pin until an upstream export contract exists — matches.

### Changes adopted from the review

| ID   | Change                                                                                                                          | Where                 |
| ---- | ------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| CV-1 | Tray gains **Reset position** (recovery when geometry restores onto a gone monitor)                                             | `main.rs`; RISK R10   |
| CV-2 | **Stale watchdog** in the frontend: footer turns warn + "stalled" if no payload for >8 s, instead of freezing plausible numbers | `app.js`, `style.css` |
| CV-3 | Acceptance matrix extended: GNOME overview, lock/unlock, suspend/resume, fullscreen, monitor hotplug (pending first pass)       | TEST_PLAN             |
| CV-4 | Risk register: transparent-region click interception (R9), geometry-restore topology risk (R10)                                 | RISK_REGISTER         |

### Reviewer suggestions deferred, with rationale

| Suggestion                                              | Disposition                                                                                                                                                                              |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ship `.deb` as primary artifact                         | **Deferred to roadmap.** v0.1 matches the qmonster family pattern (binary tarball + documented runtime deps). `.deb` needs tauri-cli bundling; queued behind first external user demand. |
| Upstream versioned `ObserveSnapshot` facade in qmonster | **Already the plan** (DECISION_LOG D-004 promotion path). Requires an upstream release; out of scope for qhud v0.1.                                                                      |
| GNOME Shell extension fallback if the invariant fails   | Recorded as ALTERNATIVES C / roadmap; today's live evidence says the XWayland invariant holds.                                                                                           |
| Dedicated polling worker, no overlapping ticks          | **Already satisfied**: single thread, strictly sequential tick → sleep; a slow poll stretches the period rather than overlapping.                                                        |
| Separate `_NET_WM_STATE_SKIP_PAGER` hint                | **Already satisfied**: live `xprop` shows SKIP_PAGER set (Tauri's skipTaskbar maps to both GTK hints).                                                                                   |
| Pricing/ClaudeSettings not loaded ⇒ not full TUI parity | Accepted v0.1 limitation: gauges (ctx/5h/7d) come from adapters, unaffected; Codex `cost` may be absent. Noted in SPEC out-of-scope.                                                     |

### Method note (honesty)

This was a real two-model validation (Claude proposer, GPT adversarial
reviewer with independent-approaches-first protocol), not a simulated
one. Final arbitration criterion was evidence (live `xprop` states,
source citations at the pinned rev), not model identity.
