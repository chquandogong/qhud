//! PaneReport -> widget JSON contract (schema v1).
//!
//! qhud deliberately re-serializes qmonster's rich `PaneReport` into a
//! small, stable, display-oriented payload. The webview never sees
//! qmonster types directly, so upstream refactors only touch this
//! file. Pressure values arrive as 0..1 fractions and leave as 0..100
//! integer percents; reset times leave as unix seconds so the frontend
//! owns countdown rendering.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use qmonster::app::event_loop::PaneReport;
use qmonster::domain::identity::{Provider, Role};
use qmonster::domain::signal::IdleCause;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Clone)]
pub struct Payload {
    pub schema: u32,
    /// "live" (mux observation) or "demo" (no mux server).
    pub source: &'static str,
    /// Which mux backend feeds the live payload ("herdr" | "tmux");
    /// `None` in demo mode. Additive schema-v1 field.
    pub backend: Option<String>,
    pub generated_at_ms: u64,
    pub poll_secs: u64,
    /// Account-scoped quota rollup, one entry per provider (D-011).
    /// 5h/7d windows are account facts, not pane facts: within a
    /// window usage is monotonically increasing, so the max reading
    /// across a provider's panes is the freshest snapshot.
    pub quotas: Vec<ProviderQuota>,
    pub panes: Vec<PaneView>,
    pub summary: Summary,
    /// Accounts that have connected before but have no live credential
    /// now (additive, v0.4.0). Rendered as dimmed, numberless rows: their
    /// quota is still ticking, so hiding them would be a lie of omission,
    /// but qhud cannot read it without an operator-approved re-auth.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub account_placeholders: Vec<crate::registry::Placeholder>,
    /// account_id -> human workspace name, so a fetched Codex workspace does
    /// not render as a hex id prefix (additive, v0.4.1).
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub workspace_names: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Clone)]
pub struct ProviderQuota {
    pub provider: String,
    pub h5: Option<Gauge>,
    pub d7: Option<Gauge>,
    /// Pane whose reading won (freshest snapshot) — shown as source.
    pub from_label: String,
    pub session: String,
    /// Which signed-in account this row belongs to (additive, v0.4.0).
    /// `None` when the provider's identity file is absent or unreadable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<crate::accounts::AccountLabel>,
    /// `"pane"` when a live mux pane fed this row, `"cache"` when it came
    /// from Claude's on-disk snapshot with no CLI running (additive,
    /// v0.4.0). A cache row must never be mistaken for a live one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<&'static str>,
    /// When the on-disk snapshot was last refreshed by the provider's own
    /// CLI. Present whenever cache data contributed anything to this row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_fetched_at_ms: Option<u64>,
    /// Per-model / per-surface windows. Only the cache carries these; the
    /// statusLine feed has no equivalent.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scoped: Vec<crate::usage_cache::ScopedLimit>,
}

#[derive(Serialize, Clone, Default)]
pub struct Summary {
    pub panes: usize,
    pub conflicts: usize,
    pub max_5h_pct: Option<u8>,
}

#[derive(Serialize, Clone, Default)]
pub struct PaneView {
    pub pane_id: String,
    /// e.g. "claude:1:main" — provider:instance:role, mirroring the TUI.
    pub label: String,
    /// Workspace/session the pane lives in (disambiguates identical
    /// labels across workspaces, D-011).
    pub session: String,
    pub provider: String,
    /// active | done | wait | limit | stale | dead (CSS class key)
    pub status: String,
    /// Human pill text, e.g. "wait approval".
    pub status_label: String,
    pub elapsed_secs: Option<u64>,
    pub cli_version: Option<String>,
    pub update_hint: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub branch: Option<String>,
    pub cwd: Option<String>,
    pub mem: Option<String>,
    pub cost_usd: Option<f64>,
    pub flags: Vec<String>,
    pub gauges: Gauges,
    pub conflicts: Vec<ConflictView>,
}

#[derive(Serialize, Clone, Default)]
pub struct Gauges {
    pub ctx: Option<Gauge>,
    pub h5: Option<Gauge>,
    pub d7: Option<Gauge>,
}

#[derive(Serialize, Clone)]
pub struct Gauge {
    pub pct: u8,
    /// Source authority label, lowercased (e.g. "providerofficial").
    pub source: String,
    pub reset_unix: Option<u64>,
    pub of_tokens: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct ConflictView {
    pub reason: String,
    pub severity: String,
    pub paths: Vec<String>,
    pub peers: Vec<String>,
}

pub fn payload(reports: &[PaneReport]) -> Payload {
    // First pass: pane_id -> label, so conflict peers render as labels.
    let labels: HashMap<String, String> = reports
        .iter()
        .map(|r| (r.pane_id.clone(), label(r)))
        .collect();

    let panes: Vec<PaneView> = reports.iter().map(|r| pane_view(r, &labels)).collect();

    let mut seen_conflicts: HashSet<String> = HashSet::new();
    for pane in &panes {
        for c in &pane.conflicts {
            seen_conflicts.insert(format!("{}|{}", c.reason, c.paths.join(",")));
        }
    }

    let quotas = provider_quotas(&panes);

    let summary = Summary {
        panes: panes.len(),
        conflicts: seen_conflicts.len(),
        max_5h_pct: quotas
            .iter()
            .filter_map(|q| q.h5.as_ref().map(|g| g.pct))
            .max(),
    };

    Payload {
        schema: SCHEMA_VERSION,
        source: "live",
        backend: None, // filled by the poll loop, which knows the mux
        generated_at_ms: now_ms(),
        poll_secs: 2,
        quotas,
        panes,
        summary,
        account_placeholders: Vec::new(),
        workspace_names: std::collections::HashMap::new(),
    }
}

/// Collapses per-pane quota snapshots into one account-scoped reading
/// per provider. Per window (5h/7d) the max percent wins: quota usage
/// only grows within a window, so every pane's snapshot is a lower
/// bound and the max is the freshest. Snapshots whose reset instant
/// has already passed belong to an EXPIRED window — their percent is
/// meaningless now and must not outrank a fresh reading (an idle
/// pane's 88% from yesterday would otherwise beat today's real 12%
/// forever). Known limit: assumes one account per provider on this
/// machine (SPEC scale envelope).
pub fn provider_quotas(panes: &[PaneView]) -> Vec<ProviderQuota> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 90 s grace: a snapshot taken moments before its own reset is
    // still honest about the *new* window for a beat.
    let expired = |g: &Gauge| g.reset_unix.is_some_and(|t| t + 90 < now);

    let mut by_provider: std::collections::BTreeMap<String, ProviderQuota> =
        std::collections::BTreeMap::new();
    for pane in panes {
        let entry = by_provider
            .entry(pane.provider.clone())
            .or_insert_with(|| ProviderQuota {
                provider: pane.provider.clone(),
                h5: None,
                d7: None,
                from_label: String::new(),
                session: String::new(),
                account: None,
                origin: Some("pane"),
                cache_fetched_at_ms: None,
                scoped: Vec::new(),
            });
        if let Some(g) = &pane.gauges.h5
            && !expired(g)
            && entry.h5.as_ref().is_none_or(|cur| g.pct > cur.pct)
        {
            entry.h5 = Some(g.clone());
            entry.from_label = pane.label.clone();
            entry.session = pane.session.clone();
        }
        if let Some(g) = &pane.gauges.d7
            && !expired(g)
            && entry.d7.as_ref().is_none_or(|cur| g.pct > cur.pct)
        {
            entry.d7 = Some(g.clone());
            if entry.h5.is_none() {
                entry.from_label = pane.label.clone();
                entry.session = pane.session.clone();
            }
        }
    }
    by_provider
        .into_values()
        .filter(|q| q.h5.is_some() || q.d7.is_some())
        .collect()
}

/// Folds Claude's on-disk snapshot into the quota strip.
///
/// Rules, in order of importance:
///  1. A live pane reading is never overridden. Within a window usage
///     only grows, and the statusline is refreshed every prompt, so the
///     pane number is at least as current as the cache.
///  2. The per-model `scoped` windows are attached regardless — only the
///     cache has them.
///  3. If Claude contributed **no** pane at all, synthesize a row from
///     the cache marked `origin: "cache"`. This is the whole point: the
///     numbers survive with no CLI running.
pub fn attach_usage_cache(payload: &mut Payload, cache: Option<&crate::usage_cache::CachedUsage>) {
    let Some(cache) = cache else { return };
    let to_gauge = |w: &crate::usage_cache::CachedWindow| Gauge {
        pct: w.pct,
        source: "providercache".to_string(),
        reset_unix: w.reset_unix,
        of_tokens: None,
    };

    if let Some(row) = payload.quotas.iter_mut().find(|q| q.provider == "claude") {
        row.cache_fetched_at_ms = Some(cache.fetched_at_ms);
        row.scoped = cache.scoped.clone();
        return;
    }
    if cache.five_hour.is_none() && cache.seven_day.is_none() {
        return;
    }
    payload.quotas.push(ProviderQuota {
        provider: "claude".to_string(),
        h5: cache.five_hour.as_ref().map(to_gauge),
        d7: cache.seven_day.as_ref().map(to_gauge),
        from_label: String::new(),
        session: String::new(),
        account: None,
        origin: Some("cache"),
        cache_fetched_at_ms: Some(cache.fetched_at_ms),
        scoped: cache.scoped.clone(),
    });
    payload.quotas.sort_by(|a, b| a.provider.cmp(&b.provider));
}

/// Adds the known-but-not-live account rows, using the live accounts the
/// caller already detected to decide what counts as "not live".
pub fn attach_placeholders(
    payload: &mut Payload,
    reg: &crate::registry::Registry,
    active: &[(String, crate::accounts::AccountLabel)],
) {
    let keys: Vec<(String, String)> = active
        .iter()
        .filter_map(|(provider, a)| {
            // Same key precedence accounts::apply_labels uses: id, else email.
            a.account_id
                .clone()
                .or_else(|| a.email.clone())
                .map(|k| (provider.clone(), k))
        })
        .collect();
    payload.account_placeholders = crate::registry::placeholders(reg, &keys);
    payload.workspace_names = reg.workspace_names.clone();
}

/// Stamps each quota row with the account that owns it. Kept out of
/// `payload` so that stays a pure function of the reports; the poll loop
/// and `--dump` each call this once per tick.
pub fn attach_accounts(
    payload: &mut Payload,
    accounts: &[(String, crate::accounts::AccountLabel)],
) {
    for quota in &mut payload.quotas {
        quota.account = accounts
            .iter()
            .find(|(provider, _)| *provider == quota.provider)
            .map(|(_, acct)| acct.clone());
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn pane_view(r: &PaneReport, labels: &HashMap<String, String>) -> PaneView {
    let (status, status_label) = status(r);
    let s = &r.signals;

    let ctx = s.context_pressure.as_ref().map(|m| Gauge {
        pct: fraction_pct(m.value),
        source: debug_lc(&m.source_kind),
        reset_unix: None,
        of_tokens: s.context_window_size.as_ref().map(|w| w.value),
    });
    // Claude/Codex expose 5h+weekly windows; Gemini exposes a single
    // quota column which we surface in the 5h slot rather than hiding.
    let h5 = s
        .quota_5h_pressure
        .as_ref()
        .or(s.quota_pressure.as_ref())
        .map(|m| Gauge {
            pct: fraction_pct(m.value),
            source: debug_lc(&m.source_kind),
            reset_unix: s.quota_5h_resets_at.as_ref().map(|t| t.value),
            of_tokens: None,
        });
    let d7 = s.quota_weekly_pressure.as_ref().map(|m| Gauge {
        pct: fraction_pct(m.value),
        source: debug_lc(&m.source_kind),
        reset_unix: s.quota_weekly_resets_at.as_ref().map(|t| t.value),
        of_tokens: None,
    });

    let cli_version = s
        .runtime_facts
        .iter()
        .find(|f| debug_lc(&f.kind).contains("version"))
        .map(|f| f.value.clone());

    let mem = s
        .agent_memory_bytes
        .as_ref()
        .map(|m| human_bytes(m.value))
        .or_else(|| {
            s.process_memory_mb
                .as_ref()
                .map(|m| format!("{:.0} MB rss", m.value))
        });

    PaneView {
        pane_id: r.pane_id.clone(),
        label: label(r),
        session: r.session_name.clone(),
        provider: provider_str(r.identity.identity.provider).to_string(),
        status: status.to_string(),
        status_label,
        elapsed_secs: r.idle_state_entered_at.map(|t| t.elapsed().as_secs()),
        cli_version,
        update_hint: None,
        model: s.model_name.as_ref().map(|m| m.value.clone()),
        effort: s.reasoning_effort.as_ref().map(|m| m.value.clone()),
        branch: s.git_branch.as_ref().map(|m| m.value.clone()),
        cwd: Some(tilde(&r.current_path)),
        mem,
        cost_usd: s.cost_usd.as_ref().map(|m| m.value),
        flags: Vec::new(),
        gauges: Gauges { ctx, h5, d7 },
        conflicts: r
            .cross_pane_findings
            .iter()
            .map(|f| ConflictView {
                reason: f.reason.clone(),
                severity: debug_lc(&f.severity),
                paths: f.paths.clone(),
                peers: f
                    .other_pane_ids
                    .iter()
                    .map(|id| labels.get(id).cloned().unwrap_or_else(|| id.clone()))
                    .collect(),
            })
            .collect(),
    }
}

fn status(r: &PaneReport) -> (&'static str, String) {
    if r.dead {
        return ("dead", "dead".into());
    }
    match r.idle_state {
        None => ("active", "active".into()),
        Some(IdleCause::PermissionWait) => ("wait", "wait approval".into()),
        Some(IdleCause::InputWait) => ("wait", "wait input".into()),
        Some(IdleCause::LimitHit) => ("limit", "limit hit".into()),
        Some(IdleCause::WorkComplete) => ("done", "done".into()),
        Some(IdleCause::Stale) => ("stale", "idle stale".into()),
    }
}

fn label(r: &PaneReport) -> String {
    let id = &r.identity.identity;
    match id.provider {
        Provider::Unknown => format!("{}:{}", r.session_name, short(&r.current_command, 14)),
        p => format!("{}:{}:{}", provider_str(p), id.instance, role_str(id.role)),
    }
}

fn provider_str(p: Provider) -> &'static str {
    match p {
        Provider::Claude => "claude",
        Provider::Codex => "codex",
        Provider::Gemini => "gemini",
        Provider::Antigravity => "agy",
        Provider::Qmonster => "qmonster",
        Provider::Unknown => "cli",
    }
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Main => "main",
        Role::Review => "review",
        Role::Research => "research",
        Role::Monitor => "monitor",
        Role::Unknown => "?",
    }
}

/// 0..1 fraction -> 0..100 integer percent, clamped.
fn fraction_pct(fraction: f32) -> u8 {
    (fraction * 100.0).round().clamp(0.0, 100.0) as u8
}

/// Debug-format, lowercased. Used for enums whose exact type we do not
/// want to couple to (SourceKind, Severity, RuntimeFactKind).
fn debug_lc<T: std::fmt::Debug>(v: &T) -> String {
    format!("{v:?}").to_lowercase()
}

fn human_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

fn tilde(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && path.starts_with(&home) => {
            format!("~{}", &path[home.len()..])
        }
        _ => path.to_string(),
    }
}

fn short(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fraction_pct_clamps_and_rounds() {
        assert_eq!(fraction_pct(0.64), 64);
        assert_eq!(fraction_pct(0.881), 88);
        assert_eq!(fraction_pct(-0.2), 0);
        assert_eq!(fraction_pct(1.7), 100);
    }

    #[test]
    fn human_bytes_units() {
        assert_eq!(human_bytes(48 * 1024), "48 KB");
        assert_eq!(human_bytes(3 * 1024 * 1024), "3.0 MB");
        assert_eq!(human_bytes(12), "12 B");
    }

    #[test]
    fn tilde_folds_home() {
        // HOME is inherited from the test environment; only assert the
        // invariant that a non-home path passes through untouched.
        assert_eq!(tilde("/nonexistent/abc"), "/nonexistent/abc");
    }

    #[test]
    fn short_truncates_with_ellipsis() {
        assert_eq!(short("cargo", 14), "cargo");
        assert_eq!(short("very-long-command-name", 8), "very-lo…");
    }

    fn cache(h5: u8, d7: u8) -> crate::usage_cache::CachedUsage {
        use crate::usage_cache::{CachedWindow, ScopedLimit};
        crate::usage_cache::CachedUsage {
            fetched_at_ms: 1_785_996_526_168,
            account_id: Some("acct-1".into()),
            five_hour: Some(CachedWindow {
                pct: h5,
                reset_unix: Some(1_786_011_600),
            }),
            seven_day: Some(CachedWindow {
                pct: d7,
                reset_unix: Some(1_786_557_600),
            }),
            scoped: vec![ScopedLimit {
                kind: "weekly_scoped".into(),
                scope: Some("Fable".into()),
                pct: 5,
                reset_unix: None,
            }],
        }
    }

    fn payload_of(panes: Vec<PaneView>) -> Payload {
        Payload {
            schema: SCHEMA_VERSION,
            source: "live",
            backend: None,
            generated_at_ms: 0,
            poll_secs: 2,
            quotas: provider_quotas(&panes),
            panes,
            summary: Summary::default(),
            account_placeholders: Vec::new(),
            workspace_names: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn cache_fills_claude_when_no_pane_is_running() {
        // The reason this exists: close every CLI and the strip would
        // otherwise go blank even though quota is still consumed.
        let mut p = payload_of(vec![pane("codex", "codex:1:main", Some(40), None)]);
        assert!(!p.quotas.iter().any(|q| q.provider == "claude"));

        attach_usage_cache(&mut p, Some(&cache(20, 3)));

        let claude = p.quotas.iter().find(|q| q.provider == "claude").unwrap();
        assert_eq!(claude.origin, Some("cache"));
        assert_eq!(claude.h5.as_ref().unwrap().pct, 20);
        assert_eq!(claude.h5.as_ref().unwrap().reset_unix, Some(1_786_011_600));
        assert_eq!(claude.cache_fetched_at_ms, Some(1_785_996_526_168));
        assert_eq!(p.quotas[0].provider, "claude", "rows stay provider-sorted");
    }

    #[test]
    fn cache_never_overrides_a_live_pane_reading() {
        // Cache was 20% when Claude Code last fetched; the pane says 36%
        // now. Within a window usage only grows, so the pane wins.
        let mut p = payload_of(vec![pane("claude", "claude:1:main", Some(36), Some(12))]);

        attach_usage_cache(&mut p, Some(&cache(20, 3)));

        let claude = p.quotas.iter().find(|q| q.provider == "claude").unwrap();
        assert_eq!(claude.h5.as_ref().unwrap().pct, 36, "live reading survives");
        assert_eq!(claude.origin, Some("pane"));
        // ...but the per-model window only the cache has is still attached,
        // along with the cache's own freshness stamp.
        assert_eq!(claude.scoped.len(), 1);
        assert_eq!(claude.scoped[0].scope.as_deref(), Some("Fable"));
        assert_eq!(claude.cache_fetched_at_ms, Some(1_785_996_526_168));
    }

    #[test]
    fn absent_cache_changes_nothing() {
        let mut p = payload_of(vec![pane("claude", "claude:1:main", Some(36), None)]);
        let before = p.quotas.len();

        attach_usage_cache(&mut p, None);

        assert_eq!(p.quotas.len(), before);
        assert!(p.quotas[0].cache_fetched_at_ms.is_none());
    }

    fn pane(provider: &str, label: &str, h5: Option<u8>, d7: Option<u8>) -> PaneView {
        let g = |pct: u8| Gauge {
            pct,
            source: "test".into(),
            reset_unix: None,
            of_tokens: None,
        };
        PaneView {
            provider: provider.into(),
            label: label.into(),
            gauges: Gauges {
                ctx: None,
                h5: h5.map(g),
                d7: d7.map(g),
            },
            ..Default::default()
        }
    }

    #[test]
    fn provider_quotas_takes_max_snapshot_per_window() {
        // Two claude panes: an idle one holding a stale (lower) quota
        // snapshot and a fresh one — the account rollup must show the
        // max, attributed to the fresher pane (D-011).
        let panes = vec![
            pane("claude", "claude:1:main", Some(61), Some(20)),
            pane("claude", "claude:1:main*", Some(88), Some(31)),
            pane("codex", "codex:1:review", Some(40), None),
            pane("agy", "agy:1:research", None, None),
        ];
        let q = provider_quotas(&panes);
        assert_eq!(q.len(), 2); // agy has no quota data → omitted
        let claude = q.iter().find(|p| p.provider == "claude").unwrap();
        assert_eq!(claude.h5.as_ref().unwrap().pct, 88);
        assert_eq!(claude.d7.as_ref().unwrap().pct, 31);
        assert_eq!(claude.from_label, "claude:1:main*");
        let codex = q.iter().find(|p| p.provider == "codex").unwrap();
        assert_eq!(codex.h5.as_ref().unwrap().pct, 40);
        assert!(codex.d7.is_none());
    }

    #[test]
    fn provider_quotas_ignores_expired_windows() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let gauge = |pct: u8, reset: u64| Gauge {
            pct,
            source: "test".into(),
            reset_unix: Some(reset),
            of_tokens: None,
        };
        // An idle pane still holds 88% from a window that reset an
        // hour ago; the active pane reads 12% of the current window.
        // The rollup must show 12, not resurrect yesterday's 88.
        let stale = PaneView {
            provider: "claude".into(),
            label: "claude:1:main".into(),
            gauges: Gauges {
                ctx: None,
                h5: Some(gauge(88, now - 3600)),
                d7: None,
            },
            ..Default::default()
        };
        let fresh = PaneView {
            provider: "claude".into(),
            label: "claude:1:main*".into(),
            gauges: Gauges {
                ctx: None,
                h5: Some(gauge(12, now + 3600)),
                d7: None,
            },
            ..Default::default()
        };
        let q = provider_quotas(&[stale, fresh]);
        let claude = q.iter().find(|p| p.provider == "claude").unwrap();
        assert_eq!(claude.h5.as_ref().unwrap().pct, 12);
        assert_eq!(claude.from_label, "claude:1:main*");

        // All snapshots expired ⇒ the window is omitted entirely
        // (showing nothing beats showing a lie).
        let only_stale = PaneView {
            provider: "codex".into(),
            gauges: Gauges {
                ctx: None,
                h5: Some(gauge(70, now - 60000)),
                d7: None,
            },
            ..Default::default()
        };
        assert!(provider_quotas(&[only_stale]).is_empty());
    }
}
