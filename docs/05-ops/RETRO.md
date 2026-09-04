# RETRO — qhud

> Status: living (cycles 1–5) · Date: 2026-09-04 · Owner: chquandogong

## Cycle 1 — v0.1.0 (2026-08-05)

### What worked (keep doing)

- **Spike before framework**: a 20-line GJS window answered the only
  existential question (does Mutter honor keep-below for XWayland?)
  before any scaffolding existed. The second spike with the real
  binary confirmed the mechanism transfers.
- **Reuse over rebuild**: qmonster already exporting `lib.rs` +
  `NoopSink` + a 4-arg `Context::new` made the entire data layer free.
  The wedge was genuinely presentation-only.
- **Demo payload as parity fixture**: one artifact serves three jobs —
  empty-state UX, development preview (browser fallback in `app.js`),
  and the visual-parity regression reference.
- **Real cross-validation**: the Codex review independently confirmed
  the no-write construction path at source level and contributed four
  adopted changes (CV-1..4). Worth the quota.

### What to fix next cycle

- **Formatter hooks vs. table edits**: markdown tables get re-aligned
  by the post-write hook, so later `Edit` anchors must be re-read
  first. Corrective action: prefer append-anchors or re-read before
  editing formatted files (applied mid-cycle).
- **Config field guessing**: `config.mux.capture_lines` vs
  `config.tmux.capture_lines` cost one build round-trip. Corrective
  action: grep the exact struct before writing bridge code against a
  pinned rev (cheap, deterministic).
- **No live tmux during the ship window** means FR-5's live path is
  verified only by construction (probe + types), not by observation.
  This is the top item on the DASHBOARD work board, deliberately not
  claimed as done.

### Deferred (carried to backlog)

`.deb` packaging · upstream ObserveSnapshot facade + unpin · GNOME
Shell extension layer · tile→pane focus jump. See DASHBOARD.

## Cycle 2 — v0.1.1 → v0.3.x (2026-08-05 → 06)

### What worked

- **Evidence-first debugging under user pressure**: every "still
  broken" report converged in one or two instrumented rounds because
  observability was built before theories (title beacons → stderr
  breadcrumbs → compositor-path injection).
- **User critique as design input**: "두 개 값이 서로 달라 의미가
  없다" produced D-011 (scope-correct display) — a better product than
  the mockup it replaced.

### What to fix next cycle

- **The synthetic-verification blind spot cost three false "fixed"
  claims** (XTEST bypasses Mutter picking). Corrective action, already
  enforced: interaction claims require compositor-path injection or a
  human hand (TEST_PLAN protocol, R12).
- **Platform reservations bite silently**: SIGUSR1 (JSC) segfault,
  DING input theft, tao phantom frame, stale screenX/Y, proc-macro
  asset caching. Corrective action: all recorded in the shared
  `tauri-linux-pitfalls` memory and D-008/D-010/D-012 — check the
  pitfall list before touching window/input code.
- **Self-matching `pkill -f` killed our own shell twice** — use
  bracket patterns (`[q]hud`) or exact `-x`.

## Cycle 3 — v0.4.0 → v0.5.0 (2026-08-07 → 10)

### What worked

- **The operator's product statement as the plan**: "every account and
  workspace at a glance, explicit refresh, never open the web again"
  turned directly into the cycle's six units (extra usage, persistence,
  refresh-all, D-015 multi-account, agy RPC, codex app-server) — no
  speculative features.
- **Delegated fetch paths (D-016)**: letting the provider's own process
  answer (codex app-server, agy loopback) resolved the two problems raw
  HTTP cannot — token rotation and keyring-held credentials — without
  qhud ever touching auth.
- **Test-first on every parser and merge rule** kept the multi-source
  merge honest: the attach_accounts overwrite bug was caught by a RED
  test before it ever rendered.

### What to fix next cycle

- **D-011 violations recur every time a NEW data source lands.** Three
  in one day, all operator-caught, all the same shape: two values one
  name (twin "weekly" chips), one fact two names (weekly vs 7D), one
  fact two rows (account row vs its own active workspace). Corrective
  action, already partly enforced: every new window/pool/row must carry
  its scope NAME end to end, display vocabulary is fixed
  (5H/1D/7D/30D), and `labels(...)` breadcrumbs now print the rendered
  gauge labels so the next violation is visible from stderr instead of
  waiting for the operator's eye.
- **A NUL byte written into app.js** made grep treat the file as binary
  (silently empty matches) while node still parsed it. Corrective
  action: separators in generated keys are plain spaces, and "grep
  suddenly finds nothing" is now a recognized symptom of a corrupted
  file, not a missing string.
- **`cargo clippy`/`cargo test` do not refresh the debug binary** — a
  stale `target/debug/qhud` reported the OLD payload during
  verification once. Corrective action: `cargo build` before any
  `--dump`-based claim.

## Cycle 4 — the selection saga (2026-08-11 → 14, v0.5.1)

### What worked

- **Instrument before theorizing, then let the instrument talk**: the
  ptr: capture-phase breadcrumbs turned "selection doesn't work" from
  four contradictory hypotheses into one glance — every click arrived,
  and none were on tiles. Two real fixes fell out (strip rows select;
  duplicate ghost listeners), and the third cause could not have been
  found without them.
- **Live experiments on the broken instance** instead of restarting it:
  xwd pixel hashing proved the freeze (byte-identical frames while the
  DOM clock repainted), external-resize proved jiggles void (D-008),
  unmap/remap proved the heal. Each experiment eliminated a fix class
  before it shipped wrong.
- **Measuring the symptom beat diagnosing the mechanism.** After two
  failed renderer-layer bets, the pixel guard (hash your own footer,
  heal on stasis) ended the bug class in one afternoon — and healed 3
  real freezes in its first 22 h, unnoticed.

### What to fix next cycle

- **The operator's report and the logs contradicted each other, and
  both were right.** "Clicks do nothing" + "every click logged as
  working" IS the frozen-presentation signature — hash the pixels
  FIRST next time, not on day three. Corrective action: the check is
  now one RUNBOOK line and the guard runs it forever.
- **"Fixed" was claimed twice on unverifiable mitigations** (DMABUF
  off, compositing off) because the trigger could not be reproduced on
  demand. Both recurred within hours. Corrective action: when the
  trigger is not reproducible, say "mitigation deployed, unverified" —
  and prefer a detector+heal that does not need the trigger understood.
- **rAF is not presentation.** In software rendering, rAF keeps firing
  while nothing reaches the screen; the JS watchdog built on it was
  blind through a real freeze. In-page signals cannot police the
  window's own visibility.

## Cycle 5 — the two-day silent ⟳ (2026-09-01 → 09-04, v0.5.3)

### What worked

- **The shipped diagnostic found it in one run.** `QHUD_EXTRA_DIAG=1
  qhud --claude-usage` existed because of a v0.5.0 shape question, and
  it answered this one in a single invocation: the two sub-objects
  printed fine, which proved the body was valid JSON and moved the
  suspicion from "dead token / empty body" to "one field's type". A
  diagnostic built for a past question paid for itself on a new one.
- **Partial failure staying partial kept the widget honest.** The
  default account's numbers were two days old, and the strip said so —
  dated, with its age caption growing — rather than blanking or showing
  a stale number as current. FR-20's dating and D-015's partial-error
  rule are why a two-day outage was a nuisance and not a wrong number.
- **A test written from the live body verbatim.** The failing test is
  the actual 09-03 response, so the regression is pinned to reality
  rather than to a hand-written approximation of it.

### What to fix next cycle

- **"Did not parse" without the field name cost two days.** Cycle 3
  already learned this on Codex (`a937a3b`: HTTP 200 plus a "no data"
  report hid two bugs, and a body preview diagnosed both), and the
  Claude path shipped the same shape of blind error anyway. Corrective
  action: every parse failure that faces the operator carries the
  parser's own message (D-019); the lesson generalizes as **a rejection
  must name what it rejected**, and it is now worth grepping the other
  fetch paths for bare `ok_or_else` rejections.
- **One optional field must never fail a whole response.** The 5h/7d
  windows had nothing to do with `used_credits` and were lost with it.
  Corrective action: integer-meaning wire numbers go through the
  lenient reader, and new fields on a provider body are assumed to be
  formatted however the provider feels like this week.
- **The field tally in the docs was 5× stale.** v0.5.2 recorded "4
  freezes through 08-17" from the guard's first days; the journal's
  whole coverage (08-26 → 09-04) shows 21, all remap-healed, clustering minutes apart
  after a display sleep. Nobody noticed because the heal is invisible —
  which is the point of D-017, and also the reason a self-healing
  subsystem needs its counter read on purpose, not when a human happens
  to look. Corrective action: the tally is a DASHBOARD row now.

## Evidence trail

FEASIBILITY_REPORT (two spikes, xprop states) ·
CROSS_VALIDATION_LOG (codex thread id) · CI runs on GitHub ·
TEST_PLAN checklist with dated evidence column.
