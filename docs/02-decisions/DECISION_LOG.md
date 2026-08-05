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
