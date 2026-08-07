//! Demo payload — shown when no tmux server is reachable.
//!
//! Mirrors the original design mockup ("Qmonster · AI CLI 모니터")
//! pane-for-pane, so demo mode doubles as a visual-parity fixture: if
//! the widget renders this payload identically to the mockup, the UI
//! port is faithful.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::view::{
    ConflictView, Gauge, Gauges, PaneView, Payload, SCHEMA_VERSION, Summary, now_ms,
};

const MIN: u64 = 60;
const HOUR: u64 = 60 * MIN;
const DAY: u64 = 24 * HOUR;

pub fn payload() -> Payload {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let official = || "providerofficial".to_string();

    let claude = PaneView {
        pane_id: "%25".into(),
        session: "demo".into(),
        label: "claude:1:main".into(),
        provider: "claude".into(),
        status: "active".into(),
        status_label: "active".into(),
        elapsed_secs: None,
        cli_version: Some("2.1.4".into()),
        update_hint: None,
        model: Some("opus-4.8".into()),
        effort: Some("max".into()),
        branch: Some("main".into()),
        cwd: Some("~/qhud".into()),
        mem: Some("48 KB".into()),
        cost_usd: None,
        flags: vec!["⏵⏵ bypass on".into()],
        gauges: Gauges {
            ctx: Some(Gauge {
                pct: 64,
                source: official(),
                reset_unix: None,
                of_tokens: Some(1_000_000),
            }),
            h5: Some(Gauge {
                pct: 88,
                source: official(),
                reset_unix: Some(now + 47 * MIN),
                of_tokens: None,
            }),
            d7: Some(Gauge {
                pct: 31,
                source: official(),
                reset_unix: Some(now + 4 * DAY + 6 * HOUR),
                of_tokens: None,
            }),
        },
        conflicts: vec![ConflictView {
            reason: "same-file edits: src/ui/panels/mod.rs".into(),
            severity: "warning".into(),
            paths: vec!["src/ui/panels/mod.rs".into()],
            peers: vec!["codex:1:review".into()],
        }],
    };

    let codex = PaneView {
        pane_id: "%27".into(),
        session: "demo".into(),
        label: "codex:1:review".into(),
        provider: "codex".into(),
        status: "stale".into(),
        status_label: "idle stale".into(),
        elapsed_secs: Some(42),
        cli_version: Some("0.142".into()),
        update_hint: Some("0.143".into()),
        model: None,
        effort: None,
        branch: None,
        cwd: Some("~/qhud".into()),
        mem: None,
        cost_usd: None,
        flags: Vec::new(),
        gauges: Gauges {
            ctx: Some(Gauge {
                pct: 36,
                source: official(),
                reset_unix: None,
                of_tokens: Some(258_000),
            }),
            h5: Some(Gauge {
                pct: 61,
                source: official(),
                reset_unix: Some(now + HOUR + 5 * MIN),
                of_tokens: None,
            }),
            d7: Some(Gauge {
                pct: 44,
                source: official(),
                reset_unix: Some(now + 5 * DAY),
                of_tokens: None,
            }),
        },
        conflicts: Vec::new(),
    };

    let agy = PaneView {
        pane_id: "%28".into(),
        session: "demo".into(),
        label: "agy:1:research".into(),
        provider: "agy".into(),
        status: "stale".into(),
        status_label: "idle stale".into(),
        elapsed_secs: Some(35),
        cli_version: Some("1.0.14".into()),
        update_hint: None,
        model: None,
        effort: None,
        branch: None,
        cwd: Some("~/research".into()),
        mem: None,
        cost_usd: None,
        flags: Vec::new(),
        gauges: Gauges {
            ctx: Some(Gauge {
                pct: 41,
                source: official(),
                reset_unix: None,
                of_tokens: Some(1_050_000),
            }),
            h5: Some(Gauge {
                pct: 8,
                source: official(),
                reset_unix: Some(now + 3 * HOUR + 40 * MIN),
                of_tokens: None,
            }),
            d7: Some(Gauge {
                pct: 3,
                source: official(),
                reset_unix: Some(now + 6 * DAY),
                of_tokens: None,
            }),
        },
        conflicts: Vec::new(),
    };

    let panes = vec![claude, codex, agy];
    let quotas = crate::view::provider_quotas(&panes);

    Payload {
        schema: SCHEMA_VERSION,
        source: "demo",
        backend: None,
        generated_at_ms: now_ms(),
        poll_secs: 2,
        summary: Summary {
            panes: 3,
            conflicts: 1,
            max_5h_pct: Some(88),
        },
        quotas,
        panes,
        account_placeholders: Vec::new(),
        workspace_names: std::collections::HashMap::new(),
        workspace_plans: std::collections::HashMap::new(),
    }
}
