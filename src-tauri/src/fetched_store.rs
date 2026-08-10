//! qhud's own record of explicit-refresh results.
//!
//! A ⟳ produces the freshest reading the widget ever has, and before this
//! module it lived only in the webview's JS state: restart qhud and the
//! numbers fell back to Claude Code's on-disk cache (often a day stale)
//! and the Codex workspace rows vanished entirely. Persisting the fetch
//! result — with its own `fetched_at_ms`, always rendered — keeps "the
//! last thing qhud actually knew" across restarts without adding any new
//! network path.
//!
//! Written temp+rename: the poll loop re-reads this file every tick, and
//! a truncate-in-place writer is exactly how the statusline sidefiles
//! produced torn reads (see the v0.4.0 changelog).

use serde::{Deserialize, Serialize};

use crate::codex_usage::WorkspaceUsage;
use crate::usage_cache::CachedUsage;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FetchedStore {
    #[serde(default)]
    pub schema: u32,
    /// Last successful Claude ⟳ (the full usage snapshot).
    #[serde(default)]
    pub claude: Option<CachedUsage>,
    /// Last successful Codex per-workspace fetch.
    #[serde(default)]
    pub codex: Option<CodexFetched>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexFetched {
    pub fetched_at_ms: u64,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceUsage>,
}

/// Junk, an old schema, or a missing file all read as an empty store: a
/// persistence layer for a HUD must never break a tick.
pub fn parse(json: &str) -> FetchedStore {
    serde_json::from_str(json).unwrap_or_default()
}

/// `~/.config/qhud/fetched-usage.json` — next to `accounts.json`, and like
/// it deliberately OUTSIDE this public repo: fetch results carry account
/// ids.
fn store_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;
    Some(home.join(".config/qhud/fetched-usage.json"))
}

pub fn load_from(path: &std::path::Path) -> FetchedStore {
    std::fs::read_to_string(path)
        .map(|s| parse(&s))
        .unwrap_or_default()
}

pub fn save_to(path: &std::path::Path, store: &FetchedStore) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| format!("serialize: {e}"))?;
    // Same-directory temp + rename, so the 2 s poll loop can never read
    // a torn file (rename within one filesystem is atomic to readers).
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("fetched-usage.json");
    let tmp = path.with_file_name(format!("{name}.tmp.{}", std::process::id()));
    std::fs::write(&tmp, json).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename into place: {e}"))
}

pub fn load() -> FetchedStore {
    store_path().map(|p| load_from(&p)).unwrap_or_default()
}

/// Best-effort merge-and-save; a failed write is logged, never surfaced —
/// the fetch that produced the data already succeeded.
fn record(update: impl FnOnce(&mut FetchedStore)) {
    let Some(path) = store_path() else { return };
    let mut store = load_from(&path);
    store.schema = SCHEMA_VERSION;
    update(&mut store);
    if let Err(e) = save_to(&path, &store) {
        eprintln!("qhud: fetched-usage store not saved: {e}");
    }
}

pub fn record_claude(usage: &CachedUsage) {
    record(|s| s.claude = Some(usage.clone()));
}

pub fn record_codex(workspaces: &[WorkspaceUsage], fetched_at_ms: u64) {
    record(|s| {
        s.codex = Some(CodexFetched {
            fetched_at_ms,
            workspaces: workspaces.to_vec(),
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex_usage::UsageWindow;

    fn sample() -> FetchedStore {
        FetchedStore {
            schema: SCHEMA_VERSION,
            claude: Some(CachedUsage {
                fetched_at_ms: 42,
                account_id: Some("acct".into()),
                five_hour: Some(crate::usage_cache::CachedWindow {
                    pct: 8,
                    reset_unix: Some(1_786_011_600),
                }),
                seven_day: None,
                scoped: Vec::new(),
                extra: None,
            }),
            codex: Some(CodexFetched {
                fetched_at_ms: 43,
                workspaces: vec![WorkspaceUsage {
                    account_id: "ws-1".into(),
                    name: Some("Personal".into()),
                    plan_type: Some("prolite".into()),
                    windows: vec![UsageWindow {
                        label: "weekly".into(),
                        used_percent: 80,
                        reset_unix: Some(1_786_330_868),
                    }],
                    credits_balance: None,
                }],
            }),
        }
    }

    #[test]
    fn roundtrips_both_providers_through_disk() {
        let dir = std::env::temp_dir().join(format!("qhud-store-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("fetched-usage.json");

        let store = sample();
        save_to(&path, &store).expect("save succeeds");
        assert_eq!(load_from(&path), store, "what was saved is what loads");

        // The atomic write must not leave its temp file behind.
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name() != "fetched-usage.json")
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn junk_missing_or_foreign_content_reads_as_empty() {
        assert_eq!(parse("not json"), FetchedStore::default());
        assert_eq!(parse("{}"), FetchedStore::default());
        // Unknown fields (a future schema) must not break an old binary.
        assert_eq!(
            parse(r#"{"schema":1,"someday":{"x":1}}"#).schema,
            1,
            "unknown fields are ignored, known ones still read"
        );
        assert_eq!(
            load_from(std::path::Path::new("/nonexistent/qhud/store.json")),
            FetchedStore::default()
        );
    }
}
