//! On-demand Codex usage, per workspace.
//!
//! Deliberately **not** part of the poll loop. This is the only code in
//! qhud that leaves the machine, so it runs when the operator explicitly
//! asks for it (a click), never on a timer. That keeps the widget's
//! steady state exactly as passive as before.
//!
//! It also never runs the refresh grant. Codex refresh tokens are
//! single-use and rotated: refreshing and failing to persist the rotation
//! breaks the operator's `codex login`. We read whatever `access_token`
//! is on disk — Codex keeps it fresh for ~10 days on its own — and on 401
//! we say "run codex login" instead of trying to be clever.
//!
//! One ChatGPT login can own several workspaces (a personal one and a
//! business one). They share the token and are selected by the
//! `chatgpt-account-id` header, so enumerating them costs no extra auth.

use serde::{Deserialize, Serialize};

const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const ACCOUNTS_URL: &str = "https://chatgpt.com/backend-api/wham/accounts/check";

/// A usage window with a label derived from its duration.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UsageWindow {
    /// `5h`, `daily`, `weekly`, `30d`, or `Nm` when it matches nothing known.
    pub label: String,
    pub used_percent: u8,
    pub reset_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct WorkspaceUsage {
    pub account_id: String,
    /// Workspace display name, when `accounts/check` supplied one.
    pub name: Option<String>,
    pub plan_type: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub credits_balance: Option<String>,
}

/// Maps a window duration to a human label.
///
/// **Never infer this from the `primary`/`secondary` lane.** Verified on
/// this machine: until 2026-07-13 primary was 5h and secondary weekly;
/// after that primary became the *weekly* window with secondary null, and
/// a 30-day window appeared. Codex itself derives the label from the
/// duration, and so must we.
pub fn window_label(seconds: u64) -> String {
    // Fuzzy match, mirroring Codex's own is_approximate_window: the
    // server has shipped slightly-off durations before.
    const KNOWN: [(u64, &str); 5] = [
        (18_000, "5h"),
        (86_400, "daily"),
        (604_800, "weekly"),
        (2_592_000, "30d"),
        (31_536_000, "yearly"),
    ];
    for (secs, label) in KNOWN {
        let tolerance = secs / 10;
        if seconds.abs_diff(secs) <= tolerance {
            return label.to_string();
        }
    }
    format!("{}m", seconds / 60)
}

#[derive(Deserialize)]
struct UsageBody {
    #[serde(default)]
    plan_type: Option<String>,
    #[serde(default)]
    rate_limit: Option<RateLimit>,
    /// Explicitly `null` on some plans, so this must be an Option:
    /// `#[serde(default)]` covers an absent field, NOT an explicit null, and
    /// a null here used to fail the whole parse and report "no data".
    #[serde(default)]
    additional_rate_limits: Option<Vec<AdditionalLimit>>,
    #[serde(default)]
    credits: Option<Credits>,
}

#[derive(Deserialize)]
struct RateLimit {
    #[serde(default)]
    primary_window: Option<RawWindow>,
    #[serde(default)]
    secondary_window: Option<RawWindow>,
}

#[derive(Deserialize)]
struct AdditionalLimit {
    #[serde(default)]
    rate_limit: Option<RateLimit>,
}

#[derive(Deserialize)]
struct RawWindow {
    #[serde(default)]
    used_percent: Option<f64>,
    #[serde(default)]
    limit_window_seconds: Option<u64>,
    #[serde(default)]
    reset_at: Option<i64>,
}

#[derive(Deserialize)]
struct Credits {
    #[serde(default)]
    balance: Option<String>,
}

impl RawWindow {
    fn resolve(self) -> Option<UsageWindow> {
        // `used_percent` is an integer on the wire but a float in the
        // rollout JSONL, so it is parsed as f64 and rounded here.
        Some(UsageWindow {
            label: window_label(self.limit_window_seconds.unwrap_or(0)),
            used_percent: self.used_percent?.round().clamp(0.0, 100.0) as u8,
            reset_unix: self.reset_at.and_then(|t| u64::try_from(t).ok()),
        })
    }
}

impl RateLimit {
    fn windows(self) -> impl Iterator<Item = UsageWindow> {
        [self.primary_window, self.secondary_window]
            .into_iter()
            .flatten()
            .filter_map(RawWindow::resolve)
    }
}

/// Parses a `/wham/usage` body.
pub fn parse_usage(account_id: &str, body: &str) -> Option<WorkspaceUsage> {
    let parsed: UsageBody = serde_json::from_str(body).ok()?;
    let mut windows: Vec<UsageWindow> = parsed
        .rate_limit
        .into_iter()
        .flat_map(RateLimit::windows)
        .collect();
    windows.extend(
        parsed
            .additional_rate_limits
            .unwrap_or_default()
            .into_iter()
            .filter_map(|a| a.rate_limit)
            .flat_map(RateLimit::windows),
    );
    Some(WorkspaceUsage {
        account_id: account_id.to_string(),
        name: None,
        plan_type: parsed.plan_type,
        windows,
        credits_balance: parsed.credits.and_then(|c| c.balance),
    })
}

// Live shape (verified 2026-08-07): `accounts` is an ARRAY of entries, each
// carrying its own `id` and `plan_type`, with `structure: "workspace"`. It is
// NOT a map keyed by account id.
#[derive(Deserialize)]
struct AccountsBody {
    #[serde(default)]
    accounts: Vec<AccountEntry>,
}

#[derive(Deserialize)]
struct AccountEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    plan_type: Option<String>,
    #[serde(default)]
    structure: Option<String>,
    /// Workspace display name, when the server supplies one.
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    title: Option<String>,
}

/// Parses `/wham/accounts/check` into one entry per workspace.
///
/// Returns `(account_id, display name, plan_type)`: the plan comes straight
/// from this call, so a workspace's plan is known without a second request.
pub fn parse_accounts(body: &str) -> Vec<(String, Option<String>, Option<String>)> {
    let Ok(parsed) = serde_json::from_str::<AccountsBody>(body) else {
        return Vec::new();
    };
    parsed
        .accounts
        .into_iter()
        .filter_map(|a| Some((a.id?, a.name.or(a.title), a.plan_type)))
        .collect()
}

/// Reads the access token and default workspace out of `~/.codex/auth.json`.
///
/// Read fresh on every call rather than cached: Codex rotates the token on
/// its own schedule and a cached copy goes stale. Only these two fields are
/// taken; the refresh token is never read, so it cannot be leaked or spent.
fn read_auth() -> Result<(String, Option<String>), String> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or("HOME is not set")?;
    let body = std::fs::read_to_string(home.join(".codex/auth.json"))
        .map_err(|_| "no ~/.codex/auth.json — run `codex login`".to_string())?;
    #[derive(Deserialize)]
    struct Auth {
        tokens: Option<Tokens>,
    }
    #[derive(Deserialize)]
    struct Tokens {
        access_token: Option<String>,
        account_id: Option<String>,
    }
    let tokens = serde_json::from_str::<Auth>(&body)
        .map_err(|e| format!("auth.json is not readable JSON: {e}"))?
        .tokens
        .ok_or("auth.json has no tokens — run `codex login`")?;
    // The id_token is NOT interchangeable here: as a Bearer it returns
    // 401 token_expired even while the access token is still valid.
    let access = tokens
        .access_token
        .filter(|t| !t.is_empty())
        .ok_or("auth.json has no access_token — run `codex login`")?;
    Ok((access, tokens.account_id))
}

async fn get(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    account_id: Option<&str>,
) -> Result<String, String> {
    let mut req = client.get(url).bearer_auth(token);
    if let Some(id) = account_id {
        // Selects WHICH workspace's quota comes back. Omitting it falls
        // back to the server default, which hides the other workspaces.
        req = req.header("chatgpt-account-id", id);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        // Deliberately NOT refreshing: Codex refresh tokens are single-use
        // and rotated, and a failed write-back breaks `codex login`.
        return Err("Codex token rejected (401) — run `codex login`".into());
    }
    if !status.is_success() {
        return Err(format!("Codex usage returned HTTP {}", status.as_u16()));
    }
    resp.text().await.map_err(|e| format!("read failed: {e}"))
}

/// Fetches usage for every workspace the signed-in Codex login owns.
///
/// One network round per workspace plus one to enumerate them. Invoked
/// from an explicit operator gesture only — never from the poll loop.
pub async fn fetch_all_workspaces() -> Result<Vec<WorkspaceUsage>, String> {
    let (token, default_account) = read_auth()?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("client build failed: {e}"))?;

    // Enumeration is best-effort: if it fails we still report the default
    // workspace rather than showing nothing.
    let mut targets = match get(&client, ACCOUNTS_URL, &token, default_account.as_deref()).await {
        Ok(body) => {
            let found = parse_accounts(&body);
            if found.is_empty() {
                let preview: String = body.chars().take(300).collect();
                eprintln!("qhud: accounts/check listed no workspace; body: {preview}");
            }
            found
        }
        Err(e) => {
            eprintln!("qhud: accounts/check failed ({e}); falling back to default account");
            Vec::new()
        }
    };
    if targets.is_empty() {
        let id = default_account
            .clone()
            .ok_or("auth.json has no account_id — run `codex login`")?;
        targets.push((id, None, None));
    }

    let mut out = Vec::new();
    let mut last_err = None;
    for (id, name, plan) in targets {
        match get(&client, USAGE_URL, &token, Some(&id)).await {
            Ok(body) => match parse_usage(&id, &body) {
                Some(mut usage) => {
                    usage.name = name;
                    // accounts/check already told us the plan; keep it when
                    // the usage body omits it.
                    if usage.plan_type.is_none() {
                        usage.plan_type = plan;
                    }
                    out.push(usage);
                }
                // A 200 whose body will not parse is NOT "no data" — the
                // schema is undocumented and churns, so say what came back
                // instead of reporting an empty result.
                None => {
                    let preview: String = body.chars().take(400).collect();
                    last_err = Some(format!(
                        "workspace {id}: HTTP 200 but body did not parse; first 400 chars: {preview}"
                    ));
                }
            },
            Err(e) => last_err = Some(e),
        }
    }
    if out.is_empty() {
        return Err(last_err.unwrap_or_else(|| "no workspace returned usage".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape verified against the live endpoint (2026-08-07), values
    // altered. Note `primary_window` here IS the weekly window and
    // `secondary_window` is null — the post-2026-07-13 shape.
    const USAGE_BODY: &str = r#"{
      "plan_type": "prolite",
      "rate_limit": {
        "allowed": true,
        "primary_window": {"used_percent": 80, "limit_window_seconds": 604800,
                           "reset_after_seconds": 257588, "reset_at": 1786330868},
        "secondary_window": null
      },
      "additional_rate_limits": [
        {"limit_name": "GPT-5.3-Codex-Spark", "metered_feature": "codex_bengalfox",
         "rate_limit": {"primary_window": {"used_percent": 4,
            "limit_window_seconds": 604800, "reset_at": 1786678081},
            "secondary_window": null}}
      ],
      "credits": {"has_credits": true, "unlimited": false, "balance": "345.5709000000"}
    }"#;

    #[test]
    fn window_label_comes_from_duration_not_lane_name() {
        assert_eq!(window_label(300 * 60), "5h");
        assert_eq!(window_label(86_400), "daily");
        assert_eq!(window_label(604_800), "weekly");
        assert_eq!(window_label(2_592_000), "30d");
    }

    #[test]
    fn unknown_window_duration_degrades_to_minutes_not_a_wrong_guess() {
        assert_eq!(window_label(7_200), "120m");
    }

    #[test]
    fn parse_usage_labels_a_weekly_primary_as_weekly() {
        let u = parse_usage("acct-1", USAGE_BODY).expect("usage parses");

        assert_eq!(u.plan_type.as_deref(), Some("prolite"));
        assert_eq!(u.credits_balance.as_deref(), Some("345.5709000000"));
        // The trap: this is `primary_window`, but it is the WEEKLY window.
        let main = &u.windows[0];
        assert_eq!(main.label, "weekly");
        assert_eq!(main.used_percent, 80);
        assert_eq!(main.reset_unix, Some(1786330868));
    }

    #[test]
    fn parse_usage_keeps_per_model_extra_limits() {
        let u = parse_usage("acct-1", USAGE_BODY).unwrap();
        assert!(
            u.windows.iter().any(|w| w.used_percent == 4),
            "additional_rate_limits carry per-model pools and must survive"
        );
    }

    #[test]
    fn parse_usage_tolerates_missing_lanes_and_junk() {
        let thin = r#"{"rate_limit":{"primary_window":null,"secondary_window":null}}"#;
        let u = parse_usage("a", thin).expect("a body with no windows is still valid");
        assert!(u.windows.is_empty(), "no lane must not become 0%");
        assert!(parse_usage("a", "not json").is_none());
    }

    #[test]
    fn parse_accounts_lists_every_workspace_with_its_plan() {
        // LIVE shape (2026-08-07): an ARRAY of entries each carrying its own
        // id and plan_type. It was previously modelled as a map keyed by id,
        // which deserialized to nothing — so the fetch silently fell back to
        // one default workspace and reported that as the whole truth.
        let body = r#"{"accounts":[
            {"id":"aaa","account_user_role":"account-owner","structure":"workspace",
             "plan_type":"team","is_zdr":false},
            {"id":"bbb","structure":"workspace","plan_type":"plus","name":"Personal"}]}"#;

        let got = parse_accounts(body);

        assert_eq!(got.len(), 2, "both workspaces must be offered");
        assert_eq!(got[0].0, "aaa");
        assert_eq!(
            got[0].2.as_deref(),
            Some("team"),
            "plan comes from this call"
        );
        assert_eq!(got[1].1.as_deref(), Some("Personal"));
        assert_eq!(got[1].2.as_deref(), Some("plus"));
    }

    #[test]
    fn usage_parses_when_additional_rate_limits_is_explicitly_null() {
        // Regression: #[serde(default)] covers an ABSENT field, not an
        // explicit null. A null here failed the whole parse, and the fetch
        // reported "no workspace returned usage" for a perfectly good 200.
        let body = r#"{"plan_type":"team","additional_rate_limits":null,
          "rate_limit":{"primary_window":{"used_percent":0,
            "limit_window_seconds":604800,"reset_at":1786696100},
            "secondary_window":null}}"#;

        let u = parse_usage("acct", body).expect("a null extras list is still valid");

        assert_eq!(u.plan_type.as_deref(), Some("team"));
        assert_eq!(u.windows.len(), 1);
        assert_eq!(u.windows[0].label, "weekly");
    }

    #[test]
    fn parse_accounts_is_empty_on_junk() {
        assert!(parse_accounts("not json").is_empty());
        assert!(parse_accounts("{}").is_empty());
    }
}
