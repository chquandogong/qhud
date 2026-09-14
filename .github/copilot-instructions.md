# qhud coding instructions

Read `/AGENTS.md`, `/CONTRIBUTING.md`, and the relevant files under `/docs`
before editing. Keep each change focused and preserve unrelated work.

- qhud is a Tauri 2 desktop app. Its frontend is plain HTML, CSS, and JavaScript;
  there is no npm dependency install step. The Rust workspace requires Rust
  1.88 or newer, and JavaScript tests use Node.js 22.
- Never expose real credentials, tokens, cookies, provider configuration, email
  addresses, account or organization IDs, session content, or local paths. Use
  synthetic fixtures. Treat provider authentication and cache files as read-only.
- Preserve quota ownership, model scope, duration, reset time, freshness, and
  source. Missing data is unknown, not zero. Do not add implicit provider
  network calls or authentication refreshes.
- Keep Windows-only dependency adaptation inside `scripts/Build-Windows.ps1` and
  `patches/`. Linux must build from a normal clone. Do not run Cargo concurrently
  with the Windows build script.
- For frontend changes, run `node --test tests/system-metrics.test.cjs`. For Rust
  changes, run `cargo fmt --all --check`, Clippy with warnings denied,
  `cargo test --locked --all-targets`, and `cargo build --locked --release`.
- Update the Korean and Simplified Chinese mirror for every user-facing English
  documentation change. Preserve commands, identifiers, dates, IDs, and tables.
- Pin third-party GitHub Actions to full commit SHAs, use minimal permissions,
  and give executable jobs a timeout.
- Do not publish releases, change repository settings, or include live-provider
  or desktop verification claims unless those actions were actually authorized
  and performed.
