use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use qmonster::app::bootstrap::Context;
use qmonster::app::config::{MuxBackend, QmonsterConfig};
use qmonster::app::event_loop;
use qmonster::app::tmux_source::build_tmux_source;
use qmonster::domain::recommendation::Severity;
use qmonster::notify::desktop::NotifyBackend;
use qmonster::store::sink::NoopSink;
use qmonster::tmux::TmuxSource;

use crate::{accounts, demo, diag, fetched_store, registry, system_metrics, usage_cache, view};

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
    let mut collector: Option<system_metrics::Collector> = None;
    let mut metrics_paused = false;

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
                        eprintln!(
                            "qhud: {backend} source lost ({}); local accounts fallback",
                            diag::mux(&e)
                        );
                        live = None;
                        None
                    }
                }
            }
            None => None,
        };

        let mut payload = if demo_mode {
            demo::payload()
        } else {
            enrich_payload(payload.unwrap_or_else(local_payload))
        };
        if !demo_mode {
            let visible = app.get_webview_window("main").is_some_and(|win| {
                win.is_visible().unwrap_or(false) && !win.is_minimized().unwrap_or(false)
            });
            if visible {
                let collector = collector.get_or_insert_with(system_metrics::Collector::new);
                if metrics_paused {
                    collector.reset();
                }
                payload.system = Some(collector.sample());
            }
            metrics_paused = !visible;
        }
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
    let mut payload = enrich_payload(payload.unwrap_or_else(local_payload));
    payload.system = Some(system_metrics::Collector::new().sample());
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
    let now = view::now_ms();
    payload.provider_health = crate::provider_health::local_checks(now);
    payload
        .provider_health
        .push(crate::provider_health::decorate(
            crate::agy_usage::local_health(now),
        ));
    for health in &mut payload.provider_health {
        // A success time describes the actual visible snapshot, never a
        // previous account's success or a credential-file modification time.
        health.last_success_at_ms = provider_success_at(
            &payload.quotas,
            health.provider,
            default_account(&active, health.provider),
        );
    }
    payload.configured_connections =
        crate::connections::statuses(now, &payload.quotas, &payload.codex_workspaces);
    payload.summary.max_5h_pct = payload
        .quotas
        .iter()
        .filter_map(|q| q.h5.as_ref().map(|g| g.pct))
        .max();
    payload
}

fn provider_success_at(
    quotas: &[view::ProviderQuota],
    provider: &str,
    default: Option<&accounts::AccountLabel>,
) -> Option<u64> {
    quotas
        .iter()
        .filter(|row| row.provider == provider)
        .filter(|row| {
            let Some(owner) = row.account.as_ref() else {
                return false;
            };
            if provider == "agy" {
                return true;
            }
            let Some(current) = default else { return false };
            owner.config_dir.is_none()
                && owner
                    .account_id
                    .as_deref()
                    .or(owner.email.as_deref())
                    .is_some_and(|id| !id.is_empty())
                && owner.org_id == current.org_id
                && owner.account_id == current.account_id
                && crate::codex_usage::same_login_workspace(
                    owner.account_id.as_deref().unwrap_or(""),
                    owner.email.as_deref(),
                    current.account_id.as_deref().unwrap_or(""),
                    current.email.as_deref(),
                )
        })
        .filter(|row| {
            row.h5.is_some()
                || row.d7.is_some()
                || !row.scoped.is_empty()
                || row.extra.is_some()
                || row.reset_credits.is_some()
        })
        .filter_map(|row| row.cache_fetched_at_ms)
        .max()
}

/// Bounded connection diagnostics, without account labels, tokens or paths.
pub fn provider_health_snapshot() -> Vec<crate::provider_health::ProviderHealth> {
    enrich_payload(local_payload()).provider_health
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
            reset_credits: None,
            plan_name: None,
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
    attach_agy_snapshot(payload, store.agy, default_account(active, "agy"));
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

/// The native Antigravity app does not maintain Gemini CLI's identity file.
/// Retain the identity returned by its quota RPC as a dated historical row;
/// never relabel that quota with a different CLI login found on this machine.
fn attach_agy_snapshot(
    payload: &mut view::Payload,
    snapshot: Option<usage_cache::CachedUsage>,
    current: Option<&accounts::AccountLabel>,
) {
    let Some(snapshot) = snapshot else { return };
    let Some(identity) = snapshot.account_id.as_deref().filter(|id| !id.is_empty()) else {
        return;
    };
    if current.and_then(|a| a.account_id.as_deref().or(a.email.as_deref())) == Some(identity) {
        view::attach_usage_cache(payload, "agy", Some(&snapshot), "fetched");
        return;
    }
    let mut saved = view::payload(&[]);
    view::attach_usage_cache(&mut saved, "agy", Some(&snapshot), "saved-account");
    for row in &mut saved.quotas {
        row.account = Some(accounts::AccountLabel {
            account_id: Some(identity.to_string()),
            email: identity.contains('@').then(|| identity.to_string()),
            display_name: Some(identity.to_string()),
            ..Default::default()
        });
    }
    payload.quotas.extend(saved.quotas);
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
    let matches = |snapshot: &usage_cache::CachedUsage| {
        snapshot.account_id.as_deref() == Some(identity)
            && snapshot.org_id.as_deref() == account.and_then(|a| a.org_id.as_deref())
    };
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
    let current_email = account.and_then(|a| a.email.as_deref());
    if let Some(saved) = saved.as_mut() {
        for workspace in &mut saved.workspaces {
            workspace.active = current_id.is_some_and(|id| {
                crate::codex_usage::same_login_workspace(
                    id,
                    current_email,
                    &workspace.account_id,
                    workspace.email.as_deref(),
                )
            });
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

    let tried = candidates.len();
    let mut outcomes: Vec<ProbeOutcome> = Vec::with_capacity(tried);
    for config in candidates {
        let attempt = config_label(&config);
        let (source, notice) = match build_tmux_source(&config) {
            Ok(build) => {
                let notice = build.startup_notice.as_ref().map(startup_notice_kind);
                (build.source, notice)
            }
            Err(e) => {
                outcomes.push(ProbeOutcome {
                    backend: attempt,
                    stage: ProbeStage::Build,
                    kind: diag::mux_build(&*e),
                    notice: None,
                });
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
                if let Some(notice) = notice {
                    eprintln!("qhud: {backend} startup notice: {}", notice.label());
                }
                eprintln!("qhud: live via {backend} ({} panes)", reports.len());
                report_probe(None);
                return Some((ctx, backend));
            }
            Err(e) => outcomes.push(ProbeOutcome {
                backend,
                stage: ProbeStage::Observe,
                kind: diag::mux(&e),
                notice,
            }),
        }
    }
    report_probe(Some(probe_summary(tried, &outcomes)));
    None
}

/// A startup notice's body embeds the raw control-mode attach error, so it
/// is never printed. The title is matched against qmonster's known fixed
/// titles; anything else is reported only as "other".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupNotice {
    ControlModeFallback,
    Other,
}

impl StartupNotice {
    fn label(self) -> &'static str {
        match self {
            Self::ControlModeFallback => "control-mode attach failed, polling this session",
            Self::Other => "other",
        }
    }
}

fn startup_notice_kind(notice: &qmonster::app::system_notice::SystemNotice) -> StartupNotice {
    if notice.title == "tmux source fallback" {
        StartupNotice::ControlModeFallback
    } else {
        StartupNotice::Other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeStage {
    Build,
    Observe,
}

/// One backend's failed attempt: only static labels and a typed category.
#[derive(Clone, Copy, Debug)]
struct ProbeOutcome {
    backend: &'static str,
    stage: ProbeStage,
    kind: diag::ErrorKind,
    notice: Option<StartupNotice>,
}

fn probe_summary(tried: usize, outcomes: &[ProbeOutcome]) -> String {
    let detail = outcomes
        .iter()
        .map(|o| {
            let stage = match o.stage {
                ProbeStage::Build => "build",
                ProbeStage::Observe => "observe",
            };
            let fallback = match o.notice {
                Some(StartupNotice::ControlModeFallback) => " after control-mode fallback",
                Some(StartupNotice::Other) => " after startup notice",
                None => "",
            };
            format!("{}={stage} {}{fallback}", o.backend, o.kind)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("qhud: no live mux source ({tried} tried: {detail}); local accounts fallback")
}

/// `build_live` re-probes every `LIVE_RETRY` while no mux answers. Print a
/// probe summary only when it changes, so a stopped mux is diagnosable
/// without a breadcrumb every ten seconds; a success re-arms the report.
fn report_probe(summary: Option<String>) {
    static LAST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(line) = &summary
        && last.as_ref() != Some(line)
    {
        eprintln!("{line}");
    }
    *last = summary;
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
                    eprintln!(
                        "qhud: failed to read qmonster config ({}); using defaults",
                        diag::qmonster_config(&e)
                    );
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
            org_id: None,
            plan_name: None,
            reset_credits: None,
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
        let mut old_fetch = snapshot(Some("account-a"), 200, 90);
        old_fetch.plan_name = Some("Previous account plan".into());
        let current_cache = snapshot(Some("account-b"), 100, 12);

        assert!(account_snapshot(None, Some(old_fetch.clone()), Some(&current)).is_none());
        let (selected, origin) =
            account_snapshot(Some(current_cache.clone()), Some(old_fetch), Some(&current)).unwrap();
        assert_eq!(selected, current_cache);
        assert!(selected.plan_name.is_none());
        assert_eq!(origin, "cache");
    }

    #[test]
    fn same_claude_account_in_different_organizations_keeps_separate_usage() {
        let mut current = account("same-login", None);
        current.org_id = Some("team-org".into());
        let mut personal = snapshot(Some("same-login"), 200, 90);
        personal.org_id = Some("personal-org".into());
        let mut team = snapshot(Some("same-login"), 100, 12);
        team.org_id = Some("team-org".into());
        let (selected, origin) =
            account_snapshot(Some(team.clone()), Some(personal.clone()), Some(&current)).unwrap();
        assert_eq!(selected, team);
        assert_eq!(origin, "cache");
        assert!(account_snapshot(None, Some(personal.clone()), Some(&current)).is_none());
        current.org_id = None;
        assert!(
            account_snapshot(None, Some(personal), Some(&current)).is_none(),
            "unknown current organization must not adopt a known organization's snapshot"
        );
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
    fn native_antigravity_snapshot_keeps_its_owner_without_cli_identity() {
        let mut payload = local_payload();
        attach_agy_snapshot(
            &mut payload,
            Some(snapshot(Some("native@example.invalid"), 200, 12)),
            None,
        );
        assert_eq!(payload.quotas.len(), 1);
        let row = &payload.quotas[0];
        assert_eq!(row.origin, Some("saved-account"));
        assert_eq!(
            row.account.as_ref().unwrap().account_id.as_deref(),
            Some("native@example.invalid")
        );
        assert_eq!(row.cache_fetched_at_ms, Some(200));
        assert_eq!(row.h5.as_ref().unwrap().pct, 12);
    }

    #[test]
    fn native_antigravity_snapshot_is_not_relabelled_with_gemini_cli_login() {
        let mut payload = local_payload();
        let cli = account("different-cli-account", None);
        let mut saved = snapshot(Some("native@example.invalid"), 200, 12);
        saved.plan_name = Some("Google AI Pro".into());
        attach_agy_snapshot(&mut payload, Some(saved), Some(&cli));
        let active = [("agy".into(), cli)];
        view::attach_accounts(&mut payload, &active);
        assert_eq!(
            payload.quotas[0]
                .account
                .as_ref()
                .unwrap()
                .account_id
                .as_deref(),
            Some("native@example.invalid")
        );
        assert_eq!(payload.quotas[0].origin, Some("saved-account"));
        assert_eq!(
            payload.quotas[0].plan_name.as_deref(),
            Some("Google AI Pro")
        );
        attach_identity_rows(&mut payload, &active);
        assert_eq!(payload.quotas.len(), 2);
        let current = payload
            .quotas
            .iter()
            .find(|row| row.account.as_ref() == Some(&active[0].1))
            .unwrap();
        assert!(
            current.plan_name.is_none(),
            "the current CLI identity must not inherit the saved account's plan"
        );
        let mut unowned = local_payload();
        let mut unidentified = snapshot(None, 200, 90);
        unidentified.plan_name = Some("Google AI Pro".into());
        attach_agy_snapshot(&mut unowned, Some(unidentified), None);
        assert!(unowned.quotas.is_empty());
    }

    #[test]
    fn switching_codex_login_keeps_previous_workspace_out_of_current_gauges() {
        let mut payload = local_payload();
        let current = account("workspace-b", None);
        let saved = fetched_store::CodexFetched {
            fetched_at_ms: 200,
            workspaces: vec![crate::codex_usage::WorkspaceUsage {
                email: None,
                reset_credits: None,
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
    fn same_codex_workspace_for_two_emails_has_only_one_active_seat() {
        use crate::codex_usage::{UsageWindow, WorkspaceUsage};
        let mut current = account("shared-team", None);
        current.email = Some("second@example.test".into());
        let workspace = |email: &str, percent, resets| WorkspaceUsage {
            account_id: "shared-team".into(),
            email: Some(email.into()),
            active: true, // saved flags from prior logins are not authoritative
            windows: vec![UsageWindow {
                label: "weekly".into(),
                used_percent: percent,
                reset_unix: None,
                scope: None,
            }],
            reset_credits: Some(resets),
            ..Default::default()
        };
        let saved = fetched_store::CodexFetched {
            fetched_at_ms: 200,
            workspaces: vec![
                workspace("first@example.test", 90, 3),
                workspace("SECOND@example.test", 12, 0),
            ],
        };
        let mut payload = local_payload();
        attach_codex_snapshot(&mut payload, Some(saved.clone()), Some(&current));
        view::attach_accounts(&mut payload, &[("codex".into(), current.clone())]);
        assert_eq!(
            payload
                .codex_workspaces
                .iter()
                .filter(|workspace| workspace.active)
                .count(),
            1
        );
        assert!(!payload.codex_workspaces[0].active);
        assert!(payload.codex_workspaces[1].active);
        assert_eq!(payload.quotas[0].account.as_ref(), Some(&current));
        assert_eq!(payload.quotas[0].d7.as_ref().unwrap().pct, 12);
        assert_eq!(payload.quotas[0].reset_credits, Some(0));
        assert_eq!(payload.codex_workspaces[0].windows[0].used_percent, 90);
        assert_eq!(payload.codex_workspaces[0].reset_credits, Some(3));

        // A legacy/unknown current email is not proof of either seat.
        current.email = None;
        let mut unknown = local_payload();
        attach_codex_snapshot(&mut unknown, Some(saved), Some(&current));
        assert!(
            unknown
                .codex_workspaces
                .iter()
                .all(|workspace| !workspace.active)
        );
        assert!(
            unknown
                .quotas
                .iter()
                .all(|row| row.h5.is_none() && row.d7.is_none() && row.reset_credits.is_none())
        );
    }

    #[test]
    fn default_health_success_ignores_another_email_or_organization() {
        let mut current = account("shared-workspace", None);
        current.email = Some("default@example.test".into());
        let mut payload = local_payload();
        let mut cache = snapshot(Some("shared-workspace"), 100, 12);
        cache.five_hour = None;
        cache.reset_credits = Some(0);
        view::attach_usage_cache(&mut payload, "codex", Some(&cache), "fetched");
        view::attach_accounts(&mut payload, &[("codex".into(), current.clone())]);
        let mut other = payload.quotas[0].clone();
        other.cache_fetched_at_ms = Some(200);
        other.account.as_mut().unwrap().email = Some("other@example.test".into());
        payload.quotas.push(other);
        assert_eq!(
            provider_success_at(&payload.quotas, "codex", Some(&current)),
            Some(100)
        );
        assert_eq!(
            provider_success_at(&payload.quotas[1..], "codex", Some(&current)),
            None
        );

        for row in &mut payload.quotas {
            row.provider = "claude".into();
        }
        current.org_id = Some("personal".into());
        payload.quotas[0].account = Some(current.clone());
        payload.quotas[1].account = Some(accounts::AccountLabel {
            org_id: Some("team".into()),
            ..current.clone()
        });
        assert_eq!(
            provider_success_at(&payload.quotas, "claude", Some(&current)),
            Some(100)
        );
        assert_eq!(
            provider_success_at(&payload.quotas[1..], "claude", Some(&current)),
            None
        );
        assert_eq!(provider_success_at(&payload.quotas, "claude", None), None);

        // Native Antigravity has an explicit, dated saved owner even without
        // Gemini CLI identity; retain that independent behavior.
        payload.quotas[1].provider = "agy".into();
        assert_eq!(provider_success_at(&payload.quotas, "agy", None), Some(200));
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
            org_id: None,
            plan_name: None,
            reset_credits: None,
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

    const MUX_CANARY: &str =
        "person@example.com sk-ant-oat01-CANARY /home/person/.qmonster %42 claude@work";

    #[test]
    fn startup_notices_are_classified_by_title_and_never_echo_the_body() {
        use qmonster::app::system_notice::SystemNotice;
        use qmonster::domain::origin::SourceKind;
        let fallback = SystemNotice {
            title: "tmux source fallback".into(),
            body: format!("control_mode attach failed: {MUX_CANARY}"),
            severity: Severity::Warning,
            source_kind: SourceKind::ProjectCanonical,
        };
        assert_eq!(
            startup_notice_kind(&fallback),
            StartupNotice::ControlModeFallback
        );
        let unknown = SystemNotice {
            title: MUX_CANARY.into(),
            ..fallback
        };
        assert_eq!(startup_notice_kind(&unknown), StartupNotice::Other);
        for kind in [StartupNotice::ControlModeFallback, StartupNotice::Other] {
            assert!(!kind.label().contains('@'));
        }
    }

    #[test]
    fn probe_summary_reports_backends_stages_and_categories_only() {
        use qmonster::tmux::PollingError;
        let herdr = PollingError::Command(MUX_CANARY.into());
        let tmux = PollingError::NonZero(MUX_CANARY.into());
        let line = probe_summary(
            2,
            &[
                ProbeOutcome {
                    backend: "herdr",
                    stage: ProbeStage::Observe,
                    kind: diag::mux(&herdr),
                    notice: None,
                },
                ProbeOutcome {
                    backend: "tmux",
                    stage: ProbeStage::Observe,
                    kind: diag::mux(&tmux),
                    notice: Some(StartupNotice::ControlModeFallback),
                },
            ],
        );
        assert_eq!(
            line,
            "qhud: no live mux source (2 tried: herdr=observe spawn, \
             tmux=observe unavailable after control-mode fallback); local accounts fallback"
        );
        for fragment in MUX_CANARY.split_whitespace() {
            assert!(!line.contains(fragment), "{fragment} leaked: {line}");
        }
    }
}
