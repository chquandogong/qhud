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
/// A rejection says WHY (serde's field/type message and
/// position). The body carries account uuid and email, and this message
/// never does: unknown fields are skipped untyped, and the typed fields
/// are numbers, booleans and window/plan strings — so the value serde
/// quotes back on a mismatch is never identity. HTTP 200 + "no data" was
/// exactly the report that hid two Codex bugs (a937a3b); the ⟳ error must
/// name the field that drifted.
pub fn parse_utilization_detailed(json: &str, fetched_at_ms: u64) -> Result<CachedUsage, String> {
    let util: Utilization = serde_json::from_str(json).map_err(|e| e.to_string())?;
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
        five_hour: util.five_hour.and_then(RawWindow::resolve),
        seven_day: util.seven_day.and_then(RawWindow::resolve),
        scoped,
        extra: resolve_extra(util.extra_usage, util.spend),
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

/// Reads the default account's snapshot from `$HOME/.claude.json`.
pub fn detect() -> Option<CachedUsage> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;
    detect_at(&home.join(".claude.json"))
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

    fn snap(at: u64) -> CachedUsage {
        CachedUsage {
            fetched_at_ms: at,
            account_id: None,
            five_hour: None,
            seven_day: None,
            scoped: Vec::new(),
            extra: None,
        }
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
    fn a_rejected_body_says_which_shape_broke() {
        // The ⟳ path surfaces this string in stderr — it must name the
        // mismatch, and it must not be the body itself.
        let err = parse_utilization_detailed(r#"{"five_hour": 5, "limits": []}"#, 1)
            .expect_err("a number where a window object belongs is a real rejection");
        assert!(
            err.contains("expected"),
            "serde's type message survives: {err}"
        );
        assert!(
            !err.contains("\"limits\""),
            "the body is not echoed back: {err}"
        );
    }
}
