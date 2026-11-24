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
///
/// `Deserialize` exists because these types round-trip through qhud's own
/// fetched-usage store (`fetched_store`), not because any provider wire
/// format matches them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedWindow {
    pub pct: u8,
    #[serde(default)]
    pub reset_unix: Option<u64>,
}

/// A model- or surface-scoped weekly window, e.g. the per-model cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopedLimit {
    /// `session`, `weekly_all`, `weekly_scoped`, …
    pub kind: String,
    /// Model display name when the limit is model-scoped.
    #[serde(default)]
    pub scope: Option<String>,
    pub pct: u8,
    #[serde(default)]
    pub reset_unix: Option<u64>,
}

/// Usage-credit spend beyond the plan windows ("extra usage") — the last
/// piece the provider's own usage page shows that the plan windows do not.
/// Normalized from the response's `spend` object (the richer shape) with
/// `extra_usage` as fallback; amounts stay in minor units so no float
/// money ever round-trips.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraUsage {
    pub enabled: bool,
    /// Minor units — cents when `exponent` is 2.
    pub used_minor: i64,
    pub currency: String,
    pub exponent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_minor: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    pub limit_reached: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedUsage {
    /// When Claude Code last refreshed this snapshot.
    pub fetched_at_ms: u64,
    #[serde(default)]
    pub account_id: Option<String>,
    /// Organization owning this snapshot. Never borrow the current login's
    /// organization for an older account-only cache entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// Provider-reported entitlement for this snapshot's account. Missing is
    /// unknown; never infer a plan from quota percentages or a current login.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_name: Option<String>,
    #[serde(default)]
    pub five_hour: Option<CachedWindow>,
    #[serde(default)]
    pub seven_day: Option<CachedWindow>,
    /// Per-model / per-surface windows, absent from the statusLine feed.
    #[serde(default)]
    pub scoped: Vec<ScopedLimit>,
    /// Usage-credit spend, when the plan carries it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<ExtraUsage>,
    /// Available manual limit resets, only when a verified provider field
    /// supplies the count. Missing means unknown, not zero available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_credits: Option<u32>,
}

/// Picks between Claude Code's own on-disk snapshot (`"cache"`) and
/// qhud's last explicit ⟳ result (`"fetched"`) — whichever is fresher.
/// A tie goes to "fetched": it is qhud's own read of the same endpoint,
/// and at least as complete.
pub fn fresher(
    disk: Option<CachedUsage>,
    fetched: Option<CachedUsage>,
) -> Option<(CachedUsage, &'static str)> {
    match (disk, fetched) {
        (None, None) => None,
        (Some(d), None) => Some((d, "cache")),
        (None, Some(f)) => Some((f, "fetched")),
        (Some(d), Some(f)) => {
            if d.fetched_at_ms > f.fetched_at_ms {
                Some((d, "cache"))
            } else {
                Some((f, "fetched"))
            }
        }
    }
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
    #[serde(default)]
    extra_usage: Option<RawExtraUsage>,
    #[serde(default)]
    spend: Option<RawSpend>,
}

#[derive(Deserialize)]
struct RawExtraUsage {
    #[serde(default)]
    is_enabled: Option<bool>,
    /// Shape unconfirmed when set (observed only as null) — kept as a
    /// raw value and interpreted defensively by `minor_units`.
    #[serde(default)]
    monthly_limit: serde_json::Value,
    #[serde(default, deserialize_with = "lenient_i64")]
    used_credits: Option<i64>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default, deserialize_with = "lenient_u8")]
    decimal_places: Option<u8>,
    #[serde(default)]
    spend_limit_reached: Option<bool>,
}

#[derive(Deserialize)]
struct RawSpend {
    #[serde(default)]
    used: Option<RawMoney>,
    #[serde(default)]
    limit: serde_json::Value,
    #[serde(default)]
    percent: Option<f64>,
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
}

#[derive(Deserialize)]
struct RawMoney {
    #[serde(default, deserialize_with = "lenient_i64")]
    amount_minor: Option<i64>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default, deserialize_with = "lenient_u8")]
    exponent: Option<u8>,
}

/// Integer-meaning wire numbers that the endpoint may serialize as an
/// integral float. Observed 2026-09-01: `extra_usage.used_credits` turned
/// from `4997` into `4997.0` (the same 4997 minor units `spend.used`
/// still carries as an integer) and every ⟳ failed for two days because
/// serde rejects a float for `i64`. An integral float is that integer; a
/// fractional float has no known unit and is dropped, like a float
/// limit — but neither may fail the whole body over one fallback field.
fn integral(v: &serde_json::Value) -> Option<i64> {
    if let Some(i) = v.as_i64() {
        return Some(i);
    }
    let f = v.as_f64()?;
    (f.fract() == 0.0 && f.abs() < 9.0e15).then_some(f as i64)
}

fn lenient_i64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    let v = <Option<serde_json::Value> as serde::Deserialize>::deserialize(d)?;
    Ok(v.as_ref().and_then(integral))
}

fn lenient_u8<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u8>, D::Error> {
    let v = <Option<serde_json::Value> as serde::Deserialize>::deserialize(d)?;
    Ok(v.as_ref()
        .and_then(integral)
        .and_then(|n| u8::try_from(n).ok()))
}

/// Money field that may be a bare integer (minor units) or a
/// `{amount_minor, ...}` object. A bare float is dropped: its unit is
/// ambiguous, and guessing a scale could show $50 as $0.50.
fn minor_units(v: &serde_json::Value) -> Option<i64> {
    v.as_i64().or_else(|| v.get("amount_minor")?.as_i64())
}

/// Prefers `spend` (the richer object: percent + severity + typed money)
/// and falls back to `extra_usage`. `spend_limit_reached` only exists on
/// `extra_usage`, so the two are merged rather than either/or.
fn resolve_extra(extra: Option<RawExtraUsage>, spend: Option<RawSpend>) -> Option<ExtraUsage> {
    if extra.is_none() && spend.is_none() {
        return None;
    }
    let used = spend.as_ref().and_then(|s| s.used.as_ref());
    Some(ExtraUsage {
        enabled: spend
            .as_ref()
            .and_then(|s| s.enabled)
            .or(extra.as_ref().and_then(|e| e.is_enabled))
            .unwrap_or(false),
        used_minor: used
            .and_then(|m| m.amount_minor)
            .or(extra.as_ref().and_then(|e| e.used_credits))
            .unwrap_or(0),
        currency: used
            .and_then(|m| m.currency.clone())
            .or(extra.as_ref().and_then(|e| e.currency.clone()))
            .unwrap_or_else(|| "USD".to_string()),
        exponent: used
            .and_then(|m| m.exponent)
            .or(extra.as_ref().and_then(|e| e.decimal_places))
            .unwrap_or(2),
        limit_minor: spend
            .as_ref()
            .and_then(|s| minor_units(&s.limit))
            .or(extra.as_ref().and_then(|e| minor_units(&e.monthly_limit))),
        percent: spend.as_ref().and_then(|s| s.percent).map(pct),
        severity: spend.and_then(|s| s.severity),
        limit_reached: extra.and_then(|e| e.spend_limit_reached).unwrap_or(false),
    })
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

/// Test convenience: the detailed parse below, flattened to an Option.
#[cfg(test)]
fn parse_utilization(json: &str, fetched_at_ms: u64) -> Option<CachedUsage> {
    parse_utilization_detailed(json, fetched_at_ms).ok()
}

/// Parses a bare `utilization` object — the shape `/api/oauth/usage` returns
/// directly, which is also exactly what Claude Code caches. `fetched_at_ms` is
/// supplied by the caller because a live response carries no timestamp.
/// A rejection reports only a coarse category and position. Serde type errors
/// can quote the unexpected JSON value, and provider bodies carry account UUIDs
/// and emails, so the raw error text must never reach stderr or the UI.
pub fn parse_utilization_detailed(json: &str, fetched_at_ms: u64) -> Result<CachedUsage, String> {
    let util: Utilization = serde_json::from_str(json).map_err(|e| crate::diag::json(&e))?;
    Ok(build(util, fetched_at_ms, None))
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
        // The verified Claude cache shape has accountUuid but no snapshot
        // organization identity. Only explicit scoped fetches bind an org.
        org_id: None,
        plan_name: None,
        five_hour: util.five_hour.and_then(RawWindow::resolve),
        seven_day: util.seven_day.and_then(RawWindow::resolve),
        scoped,
        extra: resolve_extra(util.extra_usage, util.spend),
        // The Claude OAuth usage response verified on 2026-11-17 does not
        // expose the manual reset offers/count shown in Settings > Usage.
        // Window reset timestamps and usage-credit balances are unrelated.
        reset_credits: None,
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

/// Reads the snapshot out of a specific `.claude.json` — extra accounts
/// (D-015) keep their own copy under their `CLAUDE_CONFIG_DIR`.
/// Best-effort: any failure is simply "no cache".
pub fn detect_at(config_json: &std::path::Path) -> Option<CachedUsage> {
    let body = std::fs::read_to_string(config_json).ok()?;
    parse_cached_usage(&body)
}

/// Reads the default account's snapshot from the Claude config file
/// (`$CLAUDE_CONFIG_DIR/.claude.json`, else `~/.claude.json`).
pub fn detect() -> Option<CachedUsage> {
    detect_at(&crate::paths::claude_config_file()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example cache with a synthetic account ID and the `limits[]` array
    // the statusLine feed lacks.
    const REAL_CACHE: &str = r#"{
      "cachedUsageUtilization": {
        "fetchedAtMs": 1785996526168,
        "accountUuid": "00000000-0000-4000-8000-000000000001",
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

    fn snap(at: u64) -> CachedUsage {
        CachedUsage {
            fetched_at_ms: at,
            account_id: None,
            org_id: None,
            plan_name: None,
            five_hour: None,
            seven_day: None,
            scoped: Vec::new(),
            extra: None,
            reset_credits: None,
        }
    }

    #[test]
    fn manual_reset_count_round_trips_without_inventing_a_legacy_zero() {
        let legacy = r#"{"fetched_at_ms":7,"scoped":[]}"#;
        let parsed: CachedUsage = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.reset_credits, None);
        assert_eq!(parsed.org_id, None);
        assert!(
            !serde_json::to_string(&parsed)
                .unwrap()
                .contains("reset_credits")
        );
        for count in [0, 3] {
            let mut usage = snap(7);
            usage.reset_credits = Some(count);
            let encoded = serde_json::to_string(&usage).unwrap();
            let decoded: CachedUsage = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded.reset_credits, Some(count));
        }
    }

    #[test]
    fn plan_metadata_is_optional_and_round_trips_with_snapshot_ownership() {
        let legacy: CachedUsage =
            serde_json::from_str(r#"{"fetched_at_ms":7,"account_id":"account-a"}"#).unwrap();
        assert!(legacy.plan_name.is_none());
        assert!(
            !serde_json::to_string(&legacy)
                .unwrap()
                .contains("plan_name")
        );

        let mut owned = legacy;
        owned.org_id = Some("org-a".into());
        owned.plan_name = Some("Google AI Pro".into());
        let restored: CachedUsage =
            serde_json::from_str(&serde_json::to_string(&owned).unwrap()).unwrap();
        assert_eq!(restored, owned);
        assert_eq!(restored.account_id.as_deref(), Some("account-a"));
        assert_eq!(restored.org_id.as_deref(), Some("org-a"));
        assert_eq!(restored.plan_name.as_deref(), Some("Google AI Pro"));
    }

    #[test]
    fn newer_snapshot_without_plan_does_not_inherit_an_old_entitlement() {
        let mut older = snap(10);
        older.plan_name = Some("Google AI Pro".into());
        let newer = snap(20);
        for (disk, fetched) in [(older.clone(), newer.clone()), (newer, older)] {
            let (selected, _) = fresher(Some(disk), Some(fetched)).unwrap();
            assert_eq!(selected.fetched_at_ms, 20);
            assert!(selected.plan_name.is_none());
        }
    }

    #[test]
    fn old_cache_does_not_borrow_the_current_login_organization() {
        let body = r#"{
            "oauthAccount":{"accountUuid":"same-account","organizationUuid":"new-org"},
            "cachedUsageUtilization":{"accountUuid":"same-account","fetchedAtMs":7,
              "utilization":{"seven_day":{"utilization":40}}}
        }"#;
        let usage = parse_cached_usage(body).unwrap();
        assert_eq!(usage.account_id.as_deref(), Some("same-account"));
        assert_eq!(usage.org_id, None);
        let mut scoped = usage;
        scoped.org_id = Some("verified-org".into());
        let decoded: CachedUsage =
            serde_json::from_str(&serde_json::to_string(&scoped).unwrap()).unwrap();
        assert_eq!(decoded.org_id.as_deref(), Some("verified-org"));
    }

    #[test]
    fn claude_manual_resets_are_not_inferred_from_window_resets_or_money() {
        let body = r#"{
            "seven_day":{"utilization":30,"resets_at":"2026-11-23T07:59:59Z"},
            "limits":[{"kind":"weekly_scoped","percent":10,
              "resets_at":"2026-11-23T07:59:59Z",
              "scope":{"model":{"display_name":"Synthetic model"}}}],
            "extra_usage":{"is_enabled":true,"used_credits":4},
            "spend":{"balance":{"amount_minor":5,"currency":"USD","exponent":2}}
        }"#;
        let usage = parse_utilization(body, 7).unwrap();
        assert_eq!(usage.seven_day.as_ref().map(|window| window.pct), Some(30));
        assert_eq!(usage.reset_credits, None);
        assert_eq!(parse_cached_usage(REAL_CACHE).unwrap().reset_credits, None);
    }

    #[test]
    fn fresher_picks_by_timestamp_with_ties_to_fetched() {
        assert!(fresher(None, None).is_none());

        let (u, origin) = fresher(Some(snap(10)), None).unwrap();
        assert_eq!((u.fetched_at_ms, origin), (10, "cache"));

        let (u, origin) = fresher(None, Some(snap(10))).unwrap();
        assert_eq!((u.fetched_at_ms, origin), (10, "fetched"));

        let (_, origin) = fresher(Some(snap(20)), Some(snap(10))).unwrap();
        assert_eq!(origin, "cache", "a newer CLI snapshot must win");

        let (_, origin) = fresher(Some(snap(10)), Some(snap(20))).unwrap();
        assert_eq!(origin, "fetched");

        // Tie goes to qhud's own read: same endpoint, at least as complete.
        let (_, origin) = fresher(Some(snap(10)), Some(snap(10))).unwrap();
        assert_eq!(origin, "fetched");
    }

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
            Some("00000000-0000-4000-8000-000000000001")
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

    // Synthetic values in the real cache's shape (2026-08-10): both
    // `spend` (the richer, newer object) and `extra_usage` present.
    const CACHE_WITH_EXTRA: &str = r#"{
      "cachedUsageUtilization": {
        "fetchedAtMs": 1786000000000,
        "accountUuid": "00000000-0000-0000-0000-000000000001",
        "utilization": {
          "five_hour": {"utilization": 10, "resets_at": "2026-08-06T10:20:00Z"},
          "seven_day": {"utilization": 5, "resets_at": "2026-08-12T18:00:00Z"},
          "limits": [],
          "extra_usage": {"is_enabled": true, "monthly_limit": null,
            "used_credits": 250, "utilization": null, "currency": "USD",
            "decimal_places": 2, "spend_limit_reached": false},
          "spend": {"used": {"amount_minor": 1234, "currency": "USD", "exponent": 2},
            "limit": {"amount_minor": 5000, "currency": "USD", "exponent": 2},
            "percent": 25, "severity": "normal", "enabled": true}
        }
      }
    }"#;

    #[test]
    fn spend_wins_over_extra_usage_for_the_money_numbers() {
        let u = parse_cached_usage(CACHE_WITH_EXTRA).expect("cache parses");
        let e = u.extra.expect("extra usage must survive parsing");

        assert!(e.enabled);
        assert_eq!(e.used_minor, 1234, "spend.used outranks used_credits");
        assert_eq!(e.currency, "USD");
        assert_eq!(e.exponent, 2);
        assert_eq!(e.limit_minor, Some(5000));
        assert_eq!(e.percent, Some(25));
        assert_eq!(e.severity.as_deref(), Some("normal"));
        assert!(!e.limit_reached);
    }

    #[test]
    fn extra_usage_alone_still_yields_extra() {
        // Older cache bodies carry only `extra_usage`. A bare integer
        // monthly_limit is accepted as minor units; used_credits is the
        // spend source when no `spend` object exists.
        let json = r#"{"cachedUsageUtilization":{"fetchedAtMs":1,
          "utilization":{"five_hour":null,"seven_day":null,"limits":[],
            "extra_usage":{"is_enabled":true,"monthly_limit":10000,
              "used_credits":250,"currency":"USD","decimal_places":2,
              "spend_limit_reached":true}}}}"#;

        let e = parse_cached_usage(json)
            .expect("still a valid snapshot")
            .extra
            .expect("extra_usage alone is enough");

        assert!(e.enabled);
        assert_eq!(e.used_minor, 250);
        assert_eq!(e.limit_minor, Some(10000));
        assert_eq!(e.percent, None, "no spend object, no percent");
        assert!(e.limit_reached);
    }

    #[test]
    fn extra_is_absent_when_neither_field_exists() {
        // Plans without usage credits (e.g. the REAL_CACHE fixture) must
        // not grow a zeroed extra row.
        assert!(parse_cached_usage(REAL_CACHE).unwrap().extra.is_none());
    }

    #[test]
    fn live_utilization_body_carries_extra_too() {
        // The ⟳ path parses the live body directly; extra must not be
        // cache-only.
        let body = r#"{"five_hour":{"utilization":8,"resets_at":"2026-08-06T10:20:00Z"},
          "limits":[],
          "spend":{"used":{"amount_minor":42,"currency":"USD","exponent":2},
            "percent":0,"severity":"normal","enabled":true}}"#;

        let e = parse_utilization(body, 7)
            .expect("live body parses")
            .extra
            .expect("live extra survives");

        assert_eq!(e.used_minor, 42);
        assert_eq!(
            e.limit_minor, None,
            "no limit configured is not a zero limit"
        );
    }

    #[test]
    fn a_float_limit_is_dropped_rather_than_scale_guessed() {
        // A bare float's unit is ambiguous (dollars? cents?). Guessing a
        // scale could show $50 as $0.50 — omit the limit instead.
        let json = r#"{"cachedUsageUtilization":{"fetchedAtMs":1,
          "utilization":{"limits":[],
            "extra_usage":{"is_enabled":true,"monthly_limit":50.0,
              "used_credits":0,"currency":"USD","decimal_places":2}}}}"#;

        let e = parse_cached_usage(json).unwrap().extra.unwrap();
        assert_eq!(e.limit_minor, None);
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

    #[test]
    fn live_body_with_float_used_credits_still_parses() {
        // Observed 2026-09-01 → 09-03 on the live endpoint: `used_credits`
        // arrived as `4997.0` (the same 4997 minor units `spend.used`
        // carries as an integer). A float in one fallback field must not
        // reject the whole body — every ⟳ failed for two days on this.
        let body = r#"{"five_hour":{"utilization":8,"resets_at":"2026-09-03T10:00:00Z"},
          "seven_day":{"utilization":40,"resets_at":"2026-09-08T18:00:00Z"},
          "limits":[],
          "extra_usage":{"credits_ever_enabled":true,"currency":"USD","daily":null,
            "decimal_places":2,"disabled_reason":null,"is_enabled":true,
            "monthly_limit":null,"spend_limit_reached":false,"used_credits":4997.0,
            "user_disabled":false,"utilization":null,"weekly":null},
          "spend":{"auto_reload":null,"balance":null,"can_purchase_credits":false,
            "can_toggle":false,"cap":null,"disabled_reason":null,"enabled":true,
            "limit":null,"percent":0,"severity":"normal",
            "used":{"amount_minor":4997,"currency":"USD","exponent":2}}}"#;

        let u = parse_utilization(body, 7).expect("a float used_credits is not a broken body");
        assert_eq!(u.five_hour.as_ref().map(|w| w.pct), Some(8));
        let e = u.extra.expect("extra survives");
        assert_eq!(e.used_minor, 4997);
        assert_eq!(e.percent, Some(0));
        assert_eq!(e.limit_minor, None);
    }

    #[test]
    fn integral_float_used_credits_counts_as_minor_units_without_spend() {
        // Without a `spend` object, used_credits is the only spend source.
        // `4997.0` is the integer 4997 in a float's clothing (verified
        // against spend.used.amount_minor on the live body), so it is
        // taken as minor units; a non-integral float is ambiguous and is
        // dropped like a float limit, but never fails the parse.
        let integral = r#"{"limits":[],"extra_usage":{"is_enabled":true,
          "used_credits":4997.0,"currency":"USD","decimal_places":2.0}}"#;
        let e = parse_utilization(integral, 1)
            .expect("parses")
            .extra
            .expect("extra");
        assert_eq!(e.used_minor, 4997);
        assert_eq!(e.exponent, 2);

        let fractional = r#"{"limits":[],"extra_usage":{"is_enabled":true,
          "used_credits":49.97,"currency":"USD","decimal_places":2}}"#;
        let e = parse_utilization(fractional, 1)
            .expect("still parses")
            .extra
            .expect("extra");
        assert!(e.enabled);
        assert_eq!(
            e.used_minor, 0,
            "ambiguous unit is dropped, not scale-guessed"
        );
    }

    #[test]
    fn a_rejected_body_reports_shape_without_echoing_values() {
        // Serde normally quotes the unexpected string in its error. The ⟳ path
        // surfaces this message, so a canary must never survive formatting.
        let canary = "person@example.com/token-workspace-123";
        let body = format!(r#"{{"five_hour":"{canary}","limits":[]}}"#);
        let err = parse_utilization_detailed(&body, 1)
            .expect_err("a string where a window object belongs is a real rejection");
        assert!(err.starts_with("schema mismatch at line "), "got {err}");
        assert!(!err.contains(canary), "provider value leaked: {err}");
        assert!(!err.contains('@'), "email marker leaked: {err}");
        assert!(!err.contains("token-workspace"), "identifier leaked: {err}");
    }
}
