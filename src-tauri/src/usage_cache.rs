//! Claude's on-disk usage snapshot (`~/.claude.json:cachedUsageUtilization`).
//!
//! Claude Code caches the whole subscription-usage response to disk,
//! keyed by account. That makes it the one source that still answers
//! "how much is left, and when does it reset?" when **no CLI is
//! running at all** — the statusLine sidefile only exists while a
//! session is alive. Zero network, no token.
//!
//! It is also the only local source for the newer per-model `limits[]`
//! windows, which the statusLine JSON does not carry.
//!
//! The catch is freshness: it refreshes only when Claude Code itself
//! fetches. So it is a **fallback**, never an override — a live
//! statusline reading always wins, and `fetched_at_ms` must be shown
//! so a stale number is never mistaken for a current one.

use serde::{Deserialize, Serialize};

/// One usage window resolved to the widget's units: integer percent and
/// a unix-seconds reset instant.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CachedWindow {
    pub pct: u8,
    pub reset_unix: Option<u64>,
}

/// A model- or surface-scoped weekly window, e.g. the per-model cap.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScopedLimit {
    /// `session`, `weekly_all`, `weekly_scoped`, …
    pub kind: String,
    /// Model display name when the limit is model-scoped.
    pub scope: Option<String>,
    pub pct: u8,
    pub reset_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CachedUsage {
    /// When Claude Code last refreshed this snapshot.
    pub fetched_at_ms: u64,
    pub account_id: Option<String>,
    pub five_hour: Option<CachedWindow>,
    pub seven_day: Option<CachedWindow>,
    /// Per-model / per-surface windows, absent from the statusLine feed.
    pub scoped: Vec<ScopedLimit>,
}

/// Parses RFC3339 with fractional seconds and offset, e.g.
/// `2026-08-06T10:20:00.661169+00:00`, to unix seconds.
pub fn parse_reset(iso: &str) -> Option<u64> {
    let ts = chrono::DateTime::parse_from_rfc3339(iso).ok()?.timestamp();
    u64::try_from(ts).ok()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Wrapper {
    cached_usage_utilization: Option<Cached>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Cached {
    #[serde(default)]
    fetched_at_ms: u64,
    #[serde(default)]
    account_uuid: Option<String>,
    #[serde(default)]
    utilization: Option<Utilization>,
}

#[derive(Deserialize)]
struct Utilization {
    #[serde(default)]
    five_hour: Option<RawWindow>,
    #[serde(default)]
    seven_day: Option<RawWindow>,
    #[serde(default)]
    limits: Vec<RawLimit>,
}

#[derive(Deserialize)]
struct RawWindow {
    #[serde(default)]
    utilization: Option<f64>,
    #[serde(default)]
    resets_at: Option<String>,
}

#[derive(Deserialize)]
struct RawLimit {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    percent: Option<f64>,
    #[serde(default)]
    resets_at: Option<String>,
    #[serde(default)]
    scope: Option<RawScope>,
}

#[derive(Deserialize)]
struct RawScope {
    #[serde(default)]
    model: Option<RawModel>,
}

#[derive(Deserialize)]
struct RawModel {
    #[serde(default)]
    display_name: Option<String>,
}

fn pct(v: f64) -> u8 {
    v.round().clamp(0.0, 100.0) as u8
}

impl RawWindow {
    fn resolve(self) -> Option<CachedWindow> {
        // A window with no percent is "no such limit", not zero used.
        Some(CachedWindow {
            pct: pct(self.utilization?),
            reset_unix: self.resets_at.as_deref().and_then(parse_reset),
        })
    }
}

/// Parses a bare `utilization` object — the shape `/api/oauth/usage` returns
/// directly, which is also exactly what Claude Code caches. `fetched_at_ms` is
/// supplied by the caller because a live response carries no timestamp.
pub fn parse_utilization(json: &str, fetched_at_ms: u64) -> Option<CachedUsage> {
    let util: Utilization = serde_json::from_str(json).ok()?;
    Some(build(util, fetched_at_ms, None))
}

fn build(util: Utilization, fetched_at_ms: u64, account_id: Option<String>) -> CachedUsage {
    let scoped = util
        .limits
        .into_iter()
        .filter_map(|l| {
            Some(ScopedLimit {
                kind: l.kind?,
                scope: l.scope.and_then(|s| s.model).and_then(|m| m.display_name),
                pct: pct(l.percent?),
                reset_unix: l.resets_at.as_deref().and_then(parse_reset),
            })
        })
        .collect();
    CachedUsage {
        fetched_at_ms,
        account_id,
        five_hour: util.five_hour.and_then(RawWindow::resolve),
        seven_day: util.seven_day.and_then(RawWindow::resolve),
        scoped,
    }
}

/// Reads `cachedUsageUtilization` out of a `~/.claude.json` body.
pub fn parse_cached_usage(claude_json: &str) -> Option<CachedUsage> {
    let cached = serde_json::from_str::<Wrapper>(claude_json)
        .ok()?
        .cached_usage_utilization?;
    let util = cached.utilization?;
    Some(build(util, cached.fetched_at_ms, cached.account_uuid))
}

/// Reads the snapshot from `$HOME/.claude.json`. Best-effort: any
/// failure is simply "no cache".
pub fn detect() -> Option<CachedUsage> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;
    let body = std::fs::read_to_string(home.join(".claude.json")).ok()?;
    parse_cached_usage(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed verbatim from the real ~/.claude.json (2026-08-07),
    // including the `limits[]` array the statusLine feed lacks.
    const REAL_CACHE: &str = r#"{
      "cachedUsageUtilization": {
        "fetchedAtMs": 1785996526168,
        "accountUuid": "67c22197-ede6-4221-b095-4c89789546dd",
        "utilization": {
          "five_hour": {"utilization": 20, "resets_at": "2026-08-06T10:20:00.661169+00:00"},
          "seven_day": {"utilization": 3, "resets_at": "2026-08-12T18:00:00.661188+00:00"},
          "seven_day_opus": null,
          "limits": [
            {"kind":"session","group":"session","percent":20,"is_active":true,
             "resets_at":"2026-08-06T10:20:00.661169+00:00","scope":null},
            {"kind":"weekly_all","group":"weekly","percent":3,"is_active":false,
             "resets_at":"2026-08-12T18:00:00.661188+00:00","scope":null},
            {"kind":"weekly_scoped","group":"weekly","percent":5,"is_active":false,
             "resets_at":"2026-08-12T18:00:00.661402+00:00",
             "scope":{"model":{"id":null,"display_name":"Fable"},"surface":null}}
          ]
        }
      }
    }"#;

    #[test]
    fn parse_reset_handles_fractional_seconds_and_offset() {
        // 2026-08-06T10:20:00Z
        assert_eq!(
            parse_reset("2026-08-06T10:20:00.661169+00:00"),
            Some(1786011600)
        );
        assert_eq!(parse_reset("2026-08-06T10:20:00Z"), Some(1786011600));
        assert_eq!(parse_reset("not a date"), None);
    }

    #[test]
    fn parses_both_windows_with_percent_and_reset() {
        let u = parse_cached_usage(REAL_CACHE).expect("cache parses");

        assert_eq!(u.fetched_at_ms, 1785996526168);
        assert_eq!(
            u.account_id.as_deref(),
            Some("67c22197-ede6-4221-b095-4c89789546dd")
        );
        assert_eq!(
            u.five_hour,
            Some(CachedWindow {
                pct: 20,
                reset_unix: Some(1786011600)
            })
        );
        assert_eq!(u.seven_day.as_ref().map(|w| w.pct), Some(3));
        assert!(u.seven_day.as_ref().unwrap().reset_unix.is_some());
    }

    #[test]
    fn surfaces_per_model_scoped_limits_the_statusline_lacks() {
        let u = parse_cached_usage(REAL_CACHE).unwrap();

        let scoped: Vec<&ScopedLimit> = u
            .scoped
            .iter()
            .filter(|l| l.kind == "weekly_scoped")
            .collect();
        assert_eq!(scoped.len(), 1, "the per-model window must survive parsing");
        assert_eq!(scoped[0].scope.as_deref(), Some("Fable"));
        assert_eq!(scoped[0].pct, 5);
    }

    #[test]
    fn absent_or_malformed_cache_yields_none() {
        assert!(parse_cached_usage(r#"{"oauthAccount":{}}"#).is_none());
        assert!(parse_cached_usage("not json").is_none());
    }

    #[test]
    fn null_windows_do_not_become_zero_percent() {
        // A plan without a window reports null. Rendering that as 0%
        // would claim "nothing used" when the truth is "no such limit".
        let json = r#"{"cachedUsageUtilization":{"fetchedAtMs":1,
          "utilization":{"five_hour":null,"seven_day":null,"limits":[]}}}"#;

        let u = parse_cached_usage(json).expect("still a valid snapshot");

        assert!(u.five_hour.is_none());
        assert!(u.seven_day.is_none());
        assert!(u.scoped.is_empty());
    }
}
