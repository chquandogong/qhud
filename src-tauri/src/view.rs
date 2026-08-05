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
    pub panes: Vec<PaneView>,
    pub summary: Summary,
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

    let summary = Summary {
        panes: panes.len(),
        conflicts: seen_conflicts.len(),
        max_5h_pct: panes
            .iter()
            .filter_map(|p| p.gauges.h5.as_ref().map(|g| g.pct))
            .max(),
    };

    Payload {
        schema: SCHEMA_VERSION,
        source: "live",
        backend: None, // filled by the poll loop, which knows the mux
        generated_at_ms: now_ms(),
        poll_secs: 2,
        panes,
        summary,
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
}
