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

/// Poll loop: live mux observation when a server is reachable, local
/// accounts and saved usage otherwise. Demo data requires `--demo`.
/// Re-probes for a live server every
/// `LIVE_RETRY` so starting herdr/tmux after qhud goes live without a
/// restart.
pub fn run(app: AppHandle) {
    use tauri_plugin_window_state::{AppHandleExt, StateFlags};

    let mut live: Option<(LiveCtx, &'static str)> = None;
    let mut last_attempt: Option<Instant> = None;
    let mut tick: u64 = 0;
    let demo_mode = std::env::args().any(|arg| arg == "--demo");

    loop {
        // The window-state plugin only persists on graceful exit, and a
        // desktop widget usually dies by signal/logout — checkpoint
        // geometry periodically so position/size survive anyway.
        tick += 1;
        if tick.is_multiple_of(15) {
            let _ = app.save_window_state(StateFlags::all());
        }
        if !demo_mode && live.is_none() && last_attempt.is_none_or(|t| t.elapsed() >= LIVE_RETRY) {
            last_attempt = Some(Instant::now());
            live = build_live();
        }

        let payload = match live.as_mut() {
            Some((ctx, backend)) => {
                match event_loop::run_once_with_target(ctx, Instant::now(), None) {
                    Ok((reports, _notices)) => {
                        let mut payload = view::payload(&reports);
                        payload.backend = Some((*backend).to_string());
                        Some(payload)
                    }
                    Err(e) => {
                        // Mux server went away (stopped, socket gone):
                        // Retain real accounts and saved usage while
                        // waiting for the mux to become available again.
                        eprintln!("qhud: {backend} source lost ({e}); local accounts fallback");
                        live = None;
                        None
                    }
                }
            }
            None => None,
        };

        let payload = if demo_mode {
            demo::payload()
        } else {
            enrich_payload(payload.unwrap_or_else(local_payload))
        };
        let _ = app.emit("qhud://report", &payload);
        // Pixel-level freeze watchdog (~every 28 s; see frame_guard.rs).
        crate::frame_guard::tick(&app, tick);
        std::thread::sleep(POLL);
    }
}

/// One-shot diagnostic: run a single observe tick and return the exact
/// payload the widget would render, as pretty JSON (`qhud --dump`).
pub fn dump_once() -> Option<String> {
    if std::env::args().any(|arg| arg == "--demo") {
        return serde_json::to_string_pretty(&demo::payload()).ok();
    }
    let payload = build_live().and_then(|(mut ctx, backend)| {
        let (reports, _notices) =
            event_loop::run_once_with_target(&mut ctx, Instant::now(), None).ok()?;
        let mut payload = view::payload(&reports);
        payload.backend = Some(backend.to_string());
        Some(payload)
    });
    let payload = enrich_payload(payload.unwrap_or_else(local_payload));
    serde_json::to_string_pretty(&payload).ok()
}

/// No mux does not mean sample data: an empty pane list can still show
/// real identities and the dated results of the operator's last refresh.
fn local_payload() -> view::Payload {
    let mut payload = view::payload(&[]);
    payload.source = "local";
    payload
}

fn enrich_payload(mut payload: view::Payload) -> view::Payload {
    let active = accounts::detect_all();
    attach_snapshots(&mut payload, &active);
    view::attach_accounts(&mut payload, &active);
    attach_identity_rows(&mut payload, &active);
    view::attach_placeholders(&mut payload, &registry::load(), &active);
    payload.summary.max_5h_pct = payload
        .quotas
        .iter()
        .filter_map(|q| q.h5.as_ref().map(|g| g.pct))
        .max();
    payload
}

/// An account with no saved quota still needs a row so its refresh
/// control is reachable. Missing usage stays absent, never zero-filled.
fn attach_identity_rows(payload: &mut view::Payload, active: &[(String, accounts::AccountLabel)]) {
    for (provider, account) in active {
        if payload
            .quotas
            .iter()
            .any(|row| row.provider == *provider && row.account.as_ref() == Some(account))
        {
            continue;
        }
        let row = view::ProviderQuota {
            provider: provider.clone(),
            h5: None,
            d7: None,
            from_label: String::new(),
            session: String::new(),
            account: Some(account.clone()),
            origin: None,
            cache_fetched_at_ms: None,
            scoped: Vec::new(),
            extra: None,
        };
        // Keep the default row ahead of any extra accounts, including
        // when only an extra account supplied a saved snapshot.
        if account.config_dir.is_none() {
            let index = payload
                .quotas
                .iter()
                .position(|q| q.provider == *provider)
                .unwrap_or(payload.quotas.len());
            payload.quotas.insert(index, row);
        } else {
            payload.quotas.push(row);
        }
    }
    payload.quotas.sort_by(|a, b| a.provider.cmp(&b.provider));
}

/// Snapshot enrichment shared by the poll loop and `--dump`: the fresher
/// of Claude Code's on-disk cache and qhud's own last ⟳ feeds the Claude
/// row (labelled with its true origin); every extra Claude account
/// (D-015) gets its own row the same way; the stored Codex workspace
/// rows ride along dated. Local file reads only — the loop stays passive.
fn attach_snapshots(payload: &mut view::Payload, active: &[(String, accounts::AccountLabel)]) {
    let store = fetched_store::load();
    let default_claude = default_account(active, "claude");
    if let Some((cache, origin)) =
        account_snapshot(usage_cache::detect(), store.claude, default_claude)
    {
        view::attach_usage_cache(payload, "claude", Some(&cache), origin);
    }
    // agy has no on-disk provider cache; the only snapshot is qhud's own
    // last loopback read. Legacy snapshots without an account identity
    // cannot safely be attributed after a login switch.
    if let Some((cache, origin)) = account_snapshot(None, store.agy, default_account(active, "agy"))
    {
        view::attach_usage_cache(payload, "agy", Some(&cache), origin);
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
        let snap = account_snapshot(cache, fetched, Some(acct));
        view::attach_extra_account(payload, acct.clone(), snap.as_ref().map(|(c, o)| (c, *o)));
    }
    attach_codex_snapshot(payload, store.codex, default_account(active, "codex"));
}

fn default_account<'a>(
    active: &'a [(String, accounts::AccountLabel)],
    provider: &str,
) -> Option<&'a accounts::AccountLabel> {
    active
        .iter()
        .find(|(p, a)| p == provider && a.config_dir.is_none())
        .map(|(_, account)| account)
}

/// Check ownership before comparing age: a recent snapshot from the
/// previous login must not hide an older reading for this login.
fn account_snapshot(
    disk: Option<usage_cache::CachedUsage>,
    fetched: Option<usage_cache::CachedUsage>,
    account: Option<&accounts::AccountLabel>,
) -> Option<(usage_cache::CachedUsage, &'static str)> {
    let identity = account
        .and_then(|a| a.account_id.as_deref().or(a.email.as_deref()))
        .filter(|id| !id.is_empty())?;
    let matches =
        |snapshot: &usage_cache::CachedUsage| snapshot.account_id.as_deref() == Some(identity);
    usage_cache::fresher(disk.filter(matches), fetched.filter(matches))
}

/// `active` belongs to the current login, not the login that happened
/// to own the default auth file when a stored fetch was made. Other
/// workspaces retain their own IDs and may be shown as dated extra rows.
fn attach_codex_snapshot(
    payload: &mut view::Payload,
    mut saved: Option<fetched_store::CodexFetched>,
    account: Option<&accounts::AccountLabel>,
) {
    let current_id = account.and_then(|a| a.account_id.as_deref());
    if let Some(saved) = saved.as_mut() {
        for workspace in &mut saved.workspaces {
            workspace.active = current_id == Some(workspace.account_id.as_str());
        }
    }
    view::attach_fetched_codex(payload, saved.as_ref());
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
    if let Some(home) = crate::paths::home_dir() {
        let path = home.join(".qmonster/config/qmonster.toml");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, config_dir: Option<&str>) -> accounts::AccountLabel {
        accounts::AccountLabel {
            account_id: Some(id.to_string()),
            config_dir: config_dir.map(str::to_string),
            ..Default::default()
        }
    }

    fn snapshot(id: Option<&str>, fetched_at_ms: u64, pct: u8) -> usage_cache::CachedUsage {
        usage_cache::CachedUsage {
            fetched_at_ms,
            account_id: id.map(str::to_string),
            five_hour: Some(usage_cache::CachedWindow {
                pct,
                reset_unix: None,
            }),
            seven_day: None,
            scoped: Vec::new(),
            extra: None,
        }
    }

    #[test]
    fn switching_accounts_does_not_relabel_the_previous_logins_snapshot() {
        let current = account("account-b", None);
        let old_fetch = snapshot(Some("account-a"), 200, 90);
        let current_cache = snapshot(Some("account-b"), 100, 12);

        assert!(account_snapshot(None, Some(old_fetch.clone()), Some(&current)).is_none());
        let (selected, origin) =
            account_snapshot(Some(current_cache.clone()), Some(old_fetch), Some(&current)).unwrap();
        assert_eq!(selected, current_cache);
        assert_eq!(origin, "cache");
    }

    #[test]
    fn snapshots_without_a_verifiable_owner_are_not_shown_as_current() {
        let current = account("account-b", None);
        assert!(account_snapshot(None, Some(snapshot(None, 200, 90)), Some(&current)).is_none());
        assert!(account_snapshot(None, Some(snapshot(Some("account-b"), 200, 90)), None).is_none());

        let agy = accounts::AccountLabel {
            email: Some("test@example.invalid".into()),
            ..Default::default()
        };
        assert!(
            account_snapshot(
                None,
                Some(snapshot(Some("test@example.invalid"), 200, 12)),
                Some(&agy)
            )
            .is_some()
        );
    }

    #[test]
    fn switching_codex_login_keeps_previous_workspace_out_of_current_gauges() {
        let mut payload = local_payload();
        let current = account("workspace-b", None);
        let saved = fetched_store::CodexFetched {
            fetched_at_ms: 200,
            workspaces: vec![crate::codex_usage::WorkspaceUsage {
                account_id: "workspace-a".into(),
                name: Some("Previous workspace".into()),
                plan_type: None,
                windows: vec![crate::codex_usage::UsageWindow {
                    label: "5h".into(),
                    scope: None,
                    used_percent: 90,
                    reset_unix: None,
                }],
                credits_balance: None,
                active: true,
            }],
        };
        attach_codex_snapshot(&mut payload, Some(saved.clone()), Some(&current));
        view::attach_accounts(&mut payload, &[("codex".into(), current.clone())]);
        let row = &payload.quotas[0];
        assert_eq!(row.account.as_ref(), Some(&current));
        assert!(row.h5.is_none() && row.d7.is_none());
        assert_eq!(payload.codex_workspaces[0].account_id, "workspace-a");
        assert!(!payload.codex_workspaces[0].active);

        let mut original = local_payload();
        attach_codex_snapshot(
            &mut original,
            Some(saved),
            Some(&account("workspace-a", None)),
        );
        assert!(original.codex_workspaces[0].active);
        assert_eq!(original.quotas[0].h5.as_ref().unwrap().pct, 90);
    }

    #[test]
    fn no_mux_means_local_empty_panes_and_no_invented_usage() {
        let mut payload = local_payload();
        let codex = account("codex-test", None);
        attach_identity_rows(&mut payload, &[("codex".into(), codex.clone())]);

        assert_eq!(payload.source, "local");
        assert!(payload.backend.is_none());
        assert!(payload.panes.is_empty());
        assert_eq!(payload.summary.panes, 0);
        assert_eq!(payload.quotas.len(), 1);
        let row = &payload.quotas[0];
        assert_eq!(row.account.as_ref(), Some(&codex));
        assert!(row.h5.is_none() && row.d7.is_none());
        assert!(row.origin.is_none() && row.cache_fetched_at_ms.is_none());
    }

    #[test]
    fn default_account_remains_reachable_when_only_extra_has_a_row() {
        let mut payload = local_payload();
        let default = account("claude-default", None);
        let extra = account("claude-extra", Some("C:/accounts/extra"));
        view::attach_extra_account(&mut payload, extra.clone(), None);
        let active = vec![
            ("claude".into(), default.clone()),
            ("claude".into(), extra.clone()),
        ];
        view::attach_accounts(&mut payload, &active);
        attach_identity_rows(&mut payload, &active);
        attach_identity_rows(&mut payload, &active);

        assert_eq!(payload.quotas.len(), 2);
        assert_eq!(payload.quotas[0].account.as_ref(), Some(&default));
        assert_eq!(payload.quotas[1].account.as_ref(), Some(&extra));
    }

    #[test]
    fn local_snapshot_keeps_its_age_and_origin_without_duplicate_account() {
        let mut payload = local_payload();
        let claude = account("claude-default", None);
        let snapshot = usage_cache::CachedUsage {
            fetched_at_ms: 1_700_000_000_000,
            account_id: Some("claude-default".into()),
            five_hour: Some(usage_cache::CachedWindow {
                pct: 37,
                reset_unix: Some(1_700_010_000),
            }),
            seven_day: None,
            scoped: Vec::new(),
            extra: None,
        };
        view::attach_usage_cache(&mut payload, "claude", Some(&snapshot), "fetched");
        let active = [("claude".into(), claude.clone())];
        view::attach_accounts(&mut payload, &active);
        attach_identity_rows(&mut payload, &active);

        assert!(payload.panes.is_empty());
        assert_eq!(payload.quotas.len(), 1);
        let row = &payload.quotas[0];
        assert_eq!(row.account.as_ref(), Some(&claude));
        assert_eq!(row.h5.as_ref().unwrap().pct, 37);
        assert_eq!(row.origin, Some("fetched"));
        assert_eq!(row.cache_fetched_at_ms, Some(snapshot.fetched_at_ms));
    }
}
