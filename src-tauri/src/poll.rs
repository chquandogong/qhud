use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use qmonster::app::bootstrap::Context;
use qmonster::app::config::{MuxBackend, QmonsterConfig};
use qmonster::app::event_loop;
use qmonster::app::tmux_source::build_tmux_source;
use qmonster::domain::recommendation::Severity;
use qmonster::notify::desktop::NotifyBackend;
use qmonster::store::sink::NoopSink;
use qmonster::tmux::TmuxSource;

use crate::{accounts, demo, fetched_store, registry, usage_cache, view};

const POLL: Duration = Duration::from_secs(2);
const LIVE_RETRY: Duration = Duration::from_secs(10);

/// qhud is render-only: alerting stays with the qmonster TUI and the
/// providers themselves. A widget that also pops desktop notifications
/// would double-fire everything the TUI already raises.
struct SilentNotify;

impl NotifyBackend for SilentNotify {
    fn notify(&self, _title: &str, _body: &str, _severity: Severity) {}
}

type LiveCtx = Context<TmuxSource, SilentNotify>;

/// Poll loop: live mux observation when a server is reachable, the
/// demo payload otherwise. Re-probes for a live server every
/// `LIVE_RETRY` so starting herdr/tmux after qhud goes live without a
/// restart.
pub fn run(app: AppHandle) {
    use tauri_plugin_window_state::{AppHandleExt, StateFlags};

    let mut live: Option<(LiveCtx, &'static str)> = None;
    let mut last_attempt: Option<Instant> = None;
    let mut tick: u64 = 0;

    loop {
        // The window-state plugin only persists on graceful exit, and a
        // desktop widget usually dies by signal/logout — checkpoint
        // geometry periodically so position/size survive anyway.
        tick += 1;
        if tick.is_multiple_of(15) {
            let _ = app.save_window_state(StateFlags::all());
        }
        if live.is_none() && last_attempt.is_none_or(|t| t.elapsed() >= LIVE_RETRY) {
            last_attempt = Some(Instant::now());
            live = build_live();
        }

        let payload = match live.as_mut() {
            Some((ctx, backend)) => {
                match event_loop::run_once_with_target(ctx, Instant::now(), None) {
                    Ok((reports, _notices)) => {
                        let mut payload = view::payload(&reports);
                        payload.backend = Some((*backend).to_string());
                        let active = accounts::detect_all();
                        attach_snapshots(&mut payload, &active);
                        view::attach_accounts(&mut payload, &active);
                        view::attach_placeholders(&mut payload, &registry::load(), &active);
                        Some(payload)
                    }
                    Err(e) => {
                        // Mux server went away (stopped, socket gone):
                        // drop to demo mode and keep re-probing.
                        eprintln!("qhud: {backend} source lost ({e}); demo fallback");
                        live = None;
                        None
                    }
                }
            }
            None => None,
        };

        let payload = payload.unwrap_or_else(demo::payload);
        let _ = app.emit("qhud://report", &payload);
        std::thread::sleep(POLL);
    }
}

/// One-shot diagnostic: run a single observe tick and return the exact
/// payload the widget would render, as pretty JSON (`qhud --dump`).
pub fn dump_once() -> Option<String> {
    let (mut ctx, backend) = build_live()?;
    let (reports, _notices) =
        event_loop::run_once_with_target(&mut ctx, Instant::now(), None).ok()?;
    let mut payload = view::payload(&reports);
    payload.backend = Some(backend.to_string());
    let active = accounts::detect_all();
    attach_snapshots(&mut payload, &active);
    view::attach_accounts(&mut payload, &active);
    view::attach_placeholders(&mut payload, &registry::load(), &active);
    serde_json::to_string_pretty(&payload).ok()
}

/// Snapshot enrichment shared by the poll loop and `--dump`: the fresher
/// of Claude Code's on-disk cache and qhud's own last ⟳ feeds the Claude
/// row (labelled with its true origin); every extra Claude account
/// (D-015) gets its own row the same way; the stored Codex workspace
/// rows ride along dated. Local file reads only — the loop stays passive.
fn attach_snapshots(payload: &mut view::Payload, active: &[(String, accounts::AccountLabel)]) {
    let store = fetched_store::load();
    if let Some((cache, origin)) = usage_cache::fresher(usage_cache::detect(), store.claude) {
        view::attach_usage_cache(payload, Some(&cache), origin);
    }
    for (provider, acct) in active {
        if provider != "claude" {
            continue;
        }
        let Some(dir) = &acct.config_dir else {
            continue;
        };
        let cache = usage_cache::detect_at(&std::path::Path::new(dir).join(".claude.json"));
        let fetched = store.claude_extras.get(dir).cloned();
        let snap = usage_cache::fresher(cache, fetched);
        view::attach_extra_account(payload, acct.clone(), snap.as_ref().map(|(c, o)| (c, *o)));
    }
    view::attach_fetched_codex(payload, store.codex.as_ref());
}

/// Builds a live observe context through qmonster's own mux-backend
/// factory, so `[mux] backend` in the shared config keeps meaning the
/// same thing in both frontends. One widget-specific twist (D-007):
/// `auto` detects herdr via env vars that only exist *inside* a herdr
/// pane, and qhud normally runs outside any mux pane — so for that
/// case we probe herdr first, then fall back to tmux.
fn build_live() -> Option<(LiveCtx, &'static str)> {
    let base = load_config()?;
    let inside_mux_env =
        std::env::var_os("HERDR_ENV").is_some() || std::env::var_os("HERDR_SOCKET_PATH").is_some();

    let candidates: Vec<QmonsterConfig> =
        if matches!(base.mux.backend, MuxBackend::Auto) && !inside_mux_env {
            let mut herdr = base.clone();
            herdr.mux.backend = MuxBackend::Herdr;
            let mut tmux = base;
            tmux.mux.backend = MuxBackend::Tmux;
            vec![herdr, tmux]
        } else {
            vec![base]
        };

    for config in candidates {
        let attempt = config_label(&config);
        let source = match build_tmux_source(&config) {
            Ok(build) => {
                if let Some(notice) = build.startup_notice {
                    eprintln!("qhud: {}: {}", notice.title, notice.body);
                }
                build.source
            }
            Err(e) => {
                eprintln!("qhud: {attempt} source unavailable: {e}");
                continue;
            }
        };
        // Label from the *resolved* source, not the config: `auto`
        // resolves inside the factory (e.g. to herdr when qhud itself
        // was launched from a herdr pane and inherited its env).
        let backend = source_label(&source);
        // NoopSink: the TUI owns ~/.qmonster (sqlite audit, archives,
        // snapshots). A second writer would race it, so qhud persists
        // nothing and reads nothing back.
        let mut ctx = Context::new(config, source, SilentNotify, Box::new(NoopSink));
        match event_loop::run_once(&mut ctx, Instant::now()) {
            Ok(reports) => {
                let labels: Vec<String> = view::payload(&reports)
                    .panes
                    .iter()
                    .map(|p| p.label.clone())
                    .collect();
                eprintln!(
                    "qhud: live via {backend} ({} panes: {})",
                    reports.len(),
                    labels.join(", ")
                );
                return Some((ctx, backend));
            }
            Err(_) => continue,
        }
    }
    None
}

fn config_label(config: &QmonsterConfig) -> &'static str {
    match config.mux.backend {
        MuxBackend::Herdr => "herdr",
        MuxBackend::Tmux => "tmux",
        MuxBackend::Auto => "auto",
    }
}

fn source_label(source: &TmuxSource) -> &'static str {
    match source {
        TmuxSource::Herdr(_) => "herdr",
        TmuxSource::Polling(_) | TmuxSource::ControlMode(_) => "tmux",
    }
}

fn load_config() -> Option<QmonsterConfig> {
    if let Some(home) = std::env::var_os("HOME") {
        let path = std::path::PathBuf::from(home).join(".qmonster/config/qmonster.toml");
        if path.exists() {
            match qmonster::app::config::load_with_local_override(&path) {
                Ok(config) => return Some(config),
                Err(e) => {
                    eprintln!("qhud: failed to read qmonster config ({e}); using defaults");
                }
            }
        }
    }
    // Every QmonsterConfig field carries a serde default, so an empty
    // document deserializes to the same defaults the TUI starts with.
    toml::from_str::<QmonsterConfig>("").ok()
}
