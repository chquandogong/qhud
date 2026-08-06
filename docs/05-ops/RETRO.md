# RETRO — v0.1.0 cycle

> Status: done · Date: 2026-08-05 · Owner: chquandogong

## What worked (keep doing)

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

## What to fix next cycle

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

## Deferred (carried to backlog)

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

## Evidence trail

FEASIBILITY_REPORT (two spikes, xprop states) ·
CROSS_VALIDATION_LOG (codex thread id) · CI runs on GitHub ·
TEST_PLAN checklist with dated evidence column.
