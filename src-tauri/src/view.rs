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
}

#[derive(Serialize, Clone)]
pub struct ProviderQuota {
    pub provider: String,
    pub h5: Option<Gauge>,
    pub d7: Option<Gauge>,
    /// Pane whose reading won (freshest snapshot) — shown as source.
    pub from_label: String,
    pub session: String,
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
