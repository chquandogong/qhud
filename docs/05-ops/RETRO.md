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

## Evidence trail

FEASIBILITY_REPORT (two spikes, xprop states) ·
CROSS_VALIDATION_LOG (codex thread id) · CI runs on GitHub ·
TEST_PLAN checklist with dated evidence column.
