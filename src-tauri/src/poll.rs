use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use qmonster::app::bootstrap::Context;
use qmonster::app::config::QmonsterConfig;
use qmonster::app::event_loop;
use qmonster::domain::recommendation::Severity;
use qmonster::notify::desktop::NotifyBackend;
use qmonster::store::sink::NoopSink;
use qmonster::tmux::polling::PollingSource;

use crate::{demo, view};

const POLL: Duration = Duration::from_secs(2);
const LIVE_RETRY: Duration = Duration::from_secs(10);

/// qhud is render-only: alerting stays with the qmonster TUI and the
/// providers themselves. A widget that also pops desktop notifications
/// would double-fire everything the TUI already raises.
struct SilentNotify;

impl NotifyBackend for SilentNotify {
    fn notify(&self, _title: &str, _body: &str, _severity: Severity) {}
}

type LiveCtx = Context<PollingSource, SilentNotify>;

/// Poll loop: live tmux observation when a server is reachable, the
/// demo payload otherwise. Re-probes for a live server every
/// `LIVE_RETRY` so starting tmux after qhud goes live without a
/// restart.
pub fn run(app: AppHandle) {
    let mut live: Option<LiveCtx> = None;
    let mut last_attempt: Option<Instant> = None;

    loop {
        if live.is_none() && last_attempt.is_none_or(|t| t.elapsed() >= LIVE_RETRY) {
            last_attempt = Some(Instant::now());
            live = build_live();
        }

        let payload = match live.as_mut() {
            Some(ctx) => match event_loop::run_once_with_target(ctx, Instant::now(), None) {
                Ok((reports, _notices)) => Some(view::payload(&reports)),
                Err(_) => {
                    // tmux went away (server stopped, socket gone):
                    // drop to demo mode and keep re-probing.
                    live = None;
                    None
                }
            },
            None => None,
        };

        let payload = payload.unwrap_or_else(demo::payload);
        let _ = app.emit("qhud://report", &payload);
        std::thread::sleep(POLL);
    }
}

/// Builds a live observe context against the local tmux server,
/// sharing the operator's qmonster config (read-only) when present.
/// Returns `None` when tmux is not reachable.
fn build_live() -> Option<LiveCtx> {
    let config = load_config()?;
    let capture_lines = config.tmux.capture_lines;
    let source = PollingSource::new(capture_lines);
    // NoopSink: the TUI owns ~/.qmonster (sqlite audit, archives,
    // snapshots). A second writer would race it, so qhud persists
    // nothing and reads nothing back.
    let mut ctx = Context::new(config, source, SilentNotify, Box::new(NoopSink));
    match event_loop::run_once(&mut ctx, Instant::now()) {
        Ok(_) => Some(ctx),
        Err(_) => None,
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
