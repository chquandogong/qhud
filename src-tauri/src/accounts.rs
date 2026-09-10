//! Account identity for quota rows.
//!
//! Quota is an account fact (D-011), but v0.3 rendered one row per
//! provider with no indication of *whose* quota it was. On a machine
//! where the operator holds several logins per provider — and swaps
//! between them — an unlabelled percentage is ambiguous.
//!
//! Every source read here is a local file, so this module makes no
//! network calls. Codex's ID-token payload is decoded only for an email
//! display hint; it is not verified or used to authorize anything. Access
//! and refresh tokens are ignored, and no credentials are logged or written.

use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

/// One quota-bearing scope on an account. A Claude team seat carries
/// two: the organization pool and the member's own seat, each with its
/// own rate-limit tier, which is why this is a list rather than one
/// tier string.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TierScope {
    /// `"org"` or `"user"`.
    pub kind: String,
    pub tier: String,
}

/// The identity behind a provider's quota row.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct AccountLabel {
    /// Operator-supplied short name from the inventory file (e.g.
    /// `"work"`), when one is mapped. `None` means fall back to email.
    pub label: Option<String>,
    pub email: Option<String>,
    pub account_id: Option<String>,
    pub org: Option<String>,
    /// Provider's own plan/seat words, e.g. `claude_team`, `prolite`.
    pub org_type: Option<String>,
    pub tiers: Vec<TierScope>,
    /// Operator-supplied plan for a LIVE account whose provider does not
    /// report one (agy exposes no plan locally at all). Mirrors the
    /// `plan` a placeholder carries, so the strip reads the same either way.
    pub plan: Option<String>,
    /// Precomputed `display()` so the frontend does not reimplement the
    /// label→email→id precedence in JS.
    #[serde(rename = "display")]
    pub display_name: Option<String>,
    /// Which ORGANIZATION this login is scoped to. One claude.ai
    /// account can belong to several orgs (team seat + personal free),
    /// each with its own quota pools — so (account, org) is the row
    /// identity, never the account alone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// For an EXTRA account (D-015): the expanded `CLAUDE_CONFIG_DIR`
    /// this identity was read from. The frontend matches a ⟳ result to
    /// its row by this key ("default" when absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_dir: Option<String>,
}

impl AccountLabel {
    /// Short display string for the quota strip: the operator's label
    /// if mapped, else the email, else the account id, else `None`.
    pub fn display(&self) -> Option<String> {
        self.label
            .clone()
            .or_else(|| self.email.clone())
            .or_else(|| self.account_id.clone())
    }
}

// `~/.claude.json` is camelCase on the wire; every field below relies on
// this rename rather than matching by accident.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeConfig {
    #[serde(default)]
    oauth_account: Option<ClaudeOauthAccount>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeOauthAccount {
    #[serde(default)]
    email_address: Option<String>,
    #[serde(default)]
    account_uuid: Option<String>,
    #[serde(default)]
    organization_uuid: Option<String>,
    #[serde(default)]
    organization_name: Option<String>,
    #[serde(default)]
    organization_type: Option<String>,
    #[serde(default)]
    organization_rate_limit_tier: Option<String>,
    #[serde(default)]
    user_rate_limit_tier: Option<String>,
}

/// Parses `~/.claude.json`. Claude Code stores the signed-in identity
/// under `oauthAccount`, including BOTH rate-limit tiers for a team
/// seat — the org pool and the member's own seat.
pub fn claude_account(config_json: &str) -> Option<AccountLabel> {
    let cfg: ClaudeConfig = serde_json::from_str(config_json).ok()?;
    let acct = cfg.oauth_account?;
    let mut tiers = Vec::new();
    if let Some(t) = acct.organization_rate_limit_tier {
        tiers.push(TierScope {
            kind: "org".into(),
            tier: t,
        });
    }
    if let Some(t) = acct.user_rate_limit_tier {
        tiers.push(TierScope {
            kind: "user".into(),
            tier: t,
        });
    }
    Some(AccountLabel {
        label: None,
        email: acct.email_address,
        account_id: acct.account_uuid,
        org: acct.organization_name,
        org_type: acct.organization_type,
        tiers,
        plan: None,
        display_name: None,
        org_id: acct.organization_uuid,
        config_dir: None,
    })
}

#[derive(Deserialize)]
struct CodexAuth {
    #[serde(default)]
    tokens: Option<CodexTokens>,
}

#[derive(Deserialize)]
struct CodexTokens {
    #[serde(default)]
    account_id: Option<String>,
    // A malformed optional display hint must not discard a valid account id.
    #[serde(default)]
    id_token: Option<serde_json::Value>,
}

/// Parses `~/.codex/auth.json`. `tokens.account_id` remains the identity;
/// the ID token may supply an email for display when no inventory label is
/// configured. Reading this local hint does not refresh or validate a login.
pub fn codex_account(auth_json: &str) -> Option<AccountLabel> {
    let auth: CodexAuth = serde_json::from_str(auth_json).ok()?;
    let tokens = auth.tokens?;
    let account_id = tokens.account_id?;
    let email = tokens
        .id_token
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .and_then(codex_id_token_email);
    Some(AccountLabel {
        account_id: Some(account_id),
        email,
        ..AccountLabel::default()
    })
}

/// Best-effort, unverified display metadata from a JWT payload. Do not use
/// these claims as an identity or authentication source. A workspace switch
/// can change `tokens.account_id` while keeping the same user's ID token.
fn codex_id_token_email(token: &str) -> Option<String> {
    // This is cosmetic enrichment, so bound the work for unexpected input.
    if token.len() > 64 * 1024 {
        return None;
    }
    let mut parts = token.split('.');
    let header = parts.next()?;
    let payload = parts.next()?;
    let signature = parts.next()?;
    if header.is_empty()
        || payload.is_empty()
        || payload.len() > 16 * 1024
        || signature.is_empty()
        || parts.next().is_some()
    {
        return None;
    }
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| general_purpose::URL_SAFE.decode(payload))
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let email = |value: &serde_json::Value| {
        let address = value.as_str()?.trim();
        // Reject empty, non-address, and whitespace/control-containing
        // claims without imposing a full mail-delivery validation policy.
        let (local, domain) = address.split_once('@')?;
        if address.len() > 254
            || local.is_empty()
            || domain.is_empty()
            || domain.contains('@')
            || address.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return None;
        }
        Some(address.to_owned())
    };
    claims.get("email").and_then(email).or_else(|| {
        claims
            .get("https://api.openai.com/profile")?
            .get("email")
            .and_then(email)
    })
}

#[derive(Deserialize)]
struct GoogleAccounts {
    #[serde(default)]
    active: Option<String>,
}

/// Parses `~/.gemini/google_accounts.json`. `active` is the signed-in
/// address; the `old` list is history with no usable credential, so it
/// is not surfaced as an account.
pub fn agy_account(accounts_json: &str) -> Option<AccountLabel> {
    let accts: GoogleAccounts = serde_json::from_str(accounts_json).ok()?;
    Some(AccountLabel {
        email: Some(accts.active?),
        ..AccountLabel::default()
    })
}

/// Parses the operator's inventory file into `provider:key → label`.
/// Unreadable or malformed content yields an empty map: a HUD label is
/// cosmetic and must never break a tick.
pub fn parse_inventory(inventory_json: &str) -> std::collections::HashMap<String, String> {
    #[derive(Deserialize, Default)]
    struct Inventory {
        #[serde(default)]
        labels: std::collections::HashMap<String, String>,
    }
    serde_json::from_str::<Inventory>(inventory_json)
        .unwrap_or_default()
        .labels
}

/// Parses the `plans` map: `provider:key -> plan`.
pub fn parse_plans(inventory_json: &str) -> std::collections::HashMap<String, String> {
    #[derive(Deserialize, Default)]
    struct Inv {
        #[serde(default)]
        plans: std::collections::HashMap<String, String>,
    }
    serde_json::from_str::<Inv>(inventory_json)
        .unwrap_or_default()
        .plans
}

/// Fills `account.label` from the inventory, keyed by `provider:account_id`
/// and falling back to `provider:email` for providers that expose no id.
pub fn apply_labels(
    provider: &str,
    account: &mut AccountLabel,
    labels: &std::collections::HashMap<String, String>,
) {
    let keys = [account.account_id.as_deref(), account.email.as_deref()];
    account.label = keys
        .iter()
        .flatten()
        .find_map(|key| labels.get(&format!("{provider}:{key}")).cloned());
}

/// Provider→account map built from already-read file contents. Split
/// from the filesystem read so the composition is unit-testable without
/// fixtures on disk; `detect_all` is the thin reader over it.
pub fn detect_all_from(
    claude_json: Option<&str>,
    codex_json: Option<&str>,
    agy_json: Option<&str>,
) -> Vec<(String, AccountLabel)> {
    // Order matches the quota strip's provider order.
    let parsed = [
        ("claude", claude_json.and_then(claude_account)),
        ("codex", codex_json.and_then(codex_account)),
        ("agy", agy_json.and_then(agy_account)),
    ];
    parsed
        .into_iter()
        .filter_map(|(provider, acct)| acct.map(|a| (provider.to_string(), a)))
        .collect()
}

/// Extra Claude accounts from per-account config dirs (D-015). Input is
/// `(expanded_dir, contents of <dir>/.claude.json)`; a dir whose
/// (account, org) matches the default login is skipped (listing the
/// default dir again must not duplicate its row), as is anything
/// unreadable. Same account in a DIFFERENT org stays — separate pools.
pub fn extra_claude_accounts(
    dirs: &[(String, Option<String>)],
    default: Option<(&str, Option<&str>)>,
) -> Vec<(String, AccountLabel)> {
    // Identity = (account, org): one claude.ai account can hold a team
    // seat AND a personal org, and those are separate quota pools. Only
    // the same account in the same org is a duplicate.
    let key = |id: &str, org: Option<&str>| format!("{id}/{}", org.unwrap_or("-"));
    let mut seen: std::collections::HashSet<String> =
        default.iter().map(|(id, org)| key(id, *org)).collect();
    dirs.iter()
        .filter_map(|(dir, json)| {
            let mut acct = claude_account(json.as_deref()?)?;
            if let Some(id) = &acct.account_id
                && !seen.insert(key(id, acct.org_id.as_deref()))
            {
                return None;
            }
            acct.config_dir = Some(dir.clone());
            Some(("claude".to_string(), acct))
        })
        .collect()
}

/// Reads the identity files under `$HOME` and returns the provider→account
/// map. A missing or unreadable file is simply an absent account: this is
/// best-effort enrichment and must never fail a poll tick.
pub fn detect_all() -> Vec<(String, AccountLabel)> {
    let Some(home) = crate::paths::home_dir() else {
        return Vec::new();
    };
    let read = |rel: &str| std::fs::read_to_string(home.join(rel)).ok();
    let claude = crate::paths::claude_config_file().and_then(|p| std::fs::read_to_string(p).ok());
    let codex =
        crate::paths::codex_home().and_then(|p| std::fs::read_to_string(p.join("auth.json")).ok());
    let mut found = detect_all_from(
        claude.as_deref(),
        codex.as_deref(),
        read(".gemini/google_accounts.json").as_deref(),
    );

    // Extra Claude accounts (D-015): each registry config dir keeps its
    // own `.claude.json`. Appended AFTER the defaults so "first entry per
    // provider" keeps meaning "the default account" everywhere.
    let home_str = home.to_string_lossy().to_string();
    let extra_inputs: Vec<(String, Option<String>)> = crate::registry::load()
        .claude_config_dirs
        .iter()
        .map(|d| {
            let dir = crate::registry::expand_tilde(d, &home_str);
            let json =
                std::fs::read_to_string(std::path::Path::new(&dir).join(".claude.json")).ok();
            (dir, json)
        })
        .collect();
    let default_claude = found
        .iter()
        .find(|(p, _)| p == "claude")
        .and_then(|(_, a)| a.account_id.clone().map(|id| (id, a.org_id.clone())));
    found.extend(extra_claude_accounts(
        &extra_inputs,
        default_claude
            .as_ref()
            .map(|(id, org)| (id.as_str(), org.as_deref())),
    ));

    let inventory = crate::paths::config_dir()
        .and_then(|d| std::fs::read_to_string(d.join("accounts.json")).ok());
    let labels = inventory
        .as_deref()
        .map(parse_inventory)
        .unwrap_or_default();
    let plans = inventory.as_deref().map(parse_plans).unwrap_or_default();
    for (provider, acct) in &mut found {
        apply_labels(provider, acct, &labels);
        let keys = [acct.account_id.as_deref(), acct.email.as_deref()];
        acct.plan = keys
            .iter()
            .flatten()
            .find_map(|k| plans.get(&format!("{provider}:{k}")).cloned());
        acct.display_name = acct.display();
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed from the real ~/.claude.json (2026-08-07): a team seat,
    // which is the case that carries two distinct rate-limit tiers.
    const CLAUDE_TEAM_SEAT: &str = r#"{
      "oauthAccount": {
        "accountUuid": "67c22197-ede6-4221-b095-4c89789546dd",
        "emailAddress": "user@example.com",
        "organizationName": "EXAMPLECO",
        "organizationType": "claude_team",
        "seatTier": "team_tier_1",
        "organizationRateLimitTier": "default_raven",
        "userRateLimitTier": "default_claude_max_5x"
      }
    }"#;

    #[test]
    fn claude_team_seat_exposes_both_org_and_user_tiers() {
        let acct = claude_account(CLAUDE_TEAM_SEAT).expect("team seat parses");

        assert_eq!(acct.email.as_deref(), Some("user@example.com"));
        assert_eq!(acct.org.as_deref(), Some("EXAMPLECO"));
        assert_eq!(acct.org_type.as_deref(), Some("claude_team"));
        // The operator's "team vs personal on one login" distinction is
        // exactly this pair; collapsing it to one tier loses a pool.
        assert_eq!(
            acct.tiers,
            vec![
                TierScope {
                    kind: "org".into(),
                    tier: "default_raven".into()
                },
                TierScope {
                    kind: "user".into(),
                    tier: "default_claude_max_5x".into()
                },
            ]
        );
    }

    #[test]
    fn claude_personal_account_yields_a_single_user_tier() {
        let json = r#"{"oauthAccount":{"emailAddress":"solo@example.com",
          "organizationType":"claude_pro","userRateLimitTier":"default_claude_max_5x"}}"#;

        let acct = claude_account(json).expect("personal account parses");

        assert_eq!(acct.tiers.len(), 1, "no org pool on a personal plan");
        assert_eq!(acct.tiers[0].kind, "user");
        assert!(acct.org.is_none());
    }

    #[test]
    fn claude_account_absent_when_signed_out() {
        assert!(claude_account(r#"{"someOtherKey":1}"#).is_none());
        assert!(claude_account("not json").is_none());
    }

    #[test]
    fn codex_account_keeps_the_account_id_when_the_id_token_is_unreadable() {
        let json = r#"{"auth_mode":"chatgpt","tokens":{
          "id_token":"REDACTED","access_token":"REDACTED","refresh_token":"REDACTED",
          "account_id":"3f13fa37-2915-46b3-b975-6f982c7e3c36"},
          "last_refresh":"2026-08-06T01:32:09Z"}"#;

        let acct = codex_account(json).expect("codex auth parses");

        assert_eq!(
            acct.account_id.as_deref(),
            Some("3f13fa37-2915-46b3-b975-6f982c7e3c36")
        );
        assert!(acct.email.is_none());
        assert_eq!(acct.display().as_deref(), acct.account_id.as_deref());
    }

    #[test]
    fn codex_account_absent_when_logged_out() {
        assert!(codex_account(r#"{"auth_mode":null,"tokens":null}"#).is_none());
    }

    // Synthetic tokens only: no real credentials belong in test fixtures.
    fn display_token(payload: &[u8], padded: bool) -> String {
        let encoded = if padded {
            general_purpose::URL_SAFE.encode(payload)
        } else {
            general_purpose::URL_SAFE_NO_PAD.encode(payload)
        };
        format!("e30.{encoded}.c2lnbmF0dXJl")
    }

    fn codex_auth_with_token(token: serde_json::Value) -> String {
        serde_json::json!({
            "tokens": {"account_id": "workspace-1", "id_token": token}
        })
        .to_string()
    }

    #[test]
    fn codex_detects_email_without_a_machine_specific_inventory() {
        let token = display_token(br#"{"email":"person@example.com"}"#, false);
        let auth = codex_auth_with_token(serde_json::json!(token));
        let detected = detect_all_from(None, Some(&auth), None);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].0, "codex");
        let acct = &detected[0].1;
        assert_eq!(acct.account_id.as_deref(), Some("workspace-1"));
        assert_eq!(acct.email.as_deref(), Some("person@example.com"));
        assert_eq!(acct.display().as_deref(), Some("person@example.com"));

        let serialized = serde_json::to_string(acct).unwrap();
        assert!(
            !serialized.contains(&token),
            "never send the token to the UI"
        );
    }

    #[test]
    fn codex_accepts_padded_and_unpadded_payloads_and_unicode_email() {
        let payload = r#"{"email":"  사용자@example.com  "}"#.as_bytes();
        let padded = display_token(payload, true);
        assert!(padded.contains('='), "exercise actual base64 padding");
        for token in [padded, display_token(payload, false)] {
            let acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
            assert_eq!(acct.email.as_deref(), Some("사용자@example.com"));
        }
    }

    #[test]
    fn codex_decodes_the_url_safe_jwt_alphabet() {
        // Independent fixtures contain '-' and '_' in the payload, where
        // ordinary base64 would use '+' and '/'. The extra name is ignored.
        for token in [
            "e30.eyJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJuYW1lIjoi8J-YgCJ9.c2ln",
            "e30.eyJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJuYW1lIjoiPz8_In0=.c2ln",
        ] {
            let acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
            assert_eq!(acct.email.as_deref(), Some("user@example.com"));
        }
    }

    #[test]
    fn codex_uses_only_supported_email_claims_and_prefers_the_standard_claim() {
        for (claims, expected) in [
            (
                serde_json::json!({
                    "email": "primary@example.com",
                    "https://api.openai.com/profile": {"email": "profile@example.com"}
                }),
                Some("primary@example.com"),
            ),
            (
                serde_json::json!({
                    "email": 123,
                    "https://api.openai.com/profile": {"email": "profile@example.com"}
                }),
                Some("profile@example.com"),
            ),
            (
                serde_json::json!({
                    "https://api.openai.com/profile": {"email": "profile@example.com"}
                }),
                Some("profile@example.com"),
            ),
            (
                serde_json::json!({
                    "email": "",
                    "https://api.openai.com/profile": {"email": "profile@example.com"}
                }),
                Some("profile@example.com"),
            ),
            (
                serde_json::json!({"other": {"email": "unrelated@example.com"}}),
                None,
            ),
        ] {
            let token = display_token(claims.to_string().as_bytes(), false);
            let acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
            assert_eq!(acct.email.as_deref(), expected);
            assert_eq!(acct.account_id.as_deref(), Some("workspace-1"));
        }
    }

    #[test]
    fn codex_ignores_malformed_optional_tokens_without_losing_the_account() {
        for token in [
            serde_json::Value::Null,
            serde_json::json!(false),
            serde_json::json!(123),
            serde_json::json!([]),
            serde_json::json!({}),
            serde_json::json!(""),
            serde_json::json!("not-a-jwt"),
            serde_json::json!("e30.e30"),
            serde_json::json!("e30.e30.sig.extra"),
            serde_json::json!(".e30.sig"),
            serde_json::json!("e30..sig"),
            serde_json::json!("e30.e30."),
            serde_json::json!("e30.%%%.sig"),
            serde_json::json!(display_token(b"not json", false)),
            serde_json::json!(display_token(&[0xff, 0xfe], false)),
        ] {
            let acct = codex_account(&codex_auth_with_token(token)).unwrap();
            assert_eq!(acct.email, None);
            assert_eq!(acct.display().as_deref(), Some("workspace-1"));
        }
        let acct = codex_account(r#"{"tokens":{"account_id":"workspace-1"}}"#).unwrap();
        assert_eq!(acct.display().as_deref(), Some("workspace-1"));
    }

    #[test]
    fn codex_ignores_missing_non_string_and_invalid_email_claims() {
        for value in [
            serde_json::Value::Null,
            serde_json::json!(true),
            serde_json::json!(123),
            serde_json::json!([]),
            serde_json::json!({}),
            serde_json::json!(""),
            serde_json::json!("   "),
            serde_json::json!("not-an-email"),
            serde_json::json!("@example.com"),
            serde_json::json!("user@"),
            serde_json::json!("user@@example.com"),
            serde_json::json!("user name@example.com"),
            serde_json::json!("user\nname@example.com"),
            serde_json::json!("user\u{0000}name@example.com"),
            serde_json::json!(format!("{}@example.com", "x".repeat(255))),
        ] {
            for claims in [
                serde_json::json!({"email": value}),
                serde_json::json!({"https://api.openai.com/profile": {"email": value}}),
            ] {
                let token = display_token(claims.to_string().as_bytes(), false);
                let acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
                assert_eq!(acct.email, None);
                assert_eq!(acct.display().as_deref(), Some("workspace-1"));
            }
        }
        for claims in [
            "{}",
            "[]",
            "null",
            "123",
            r#"{"https://api.openai.com/profile":false}"#,
        ] {
            let token = display_token(claims.as_bytes(), false);
            let acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
            assert_eq!(acct.display().as_deref(), Some("workspace-1"));
        }
    }

    #[test]
    fn codex_bounds_cosmetic_token_decoding() {
        let oversized_payload = serde_json::json!({
            "email": "person@example.com", "extra": "x".repeat(16 * 1024)
        });
        for token in [
            display_token(oversized_payload.to_string().as_bytes(), false),
            format!("{}.e30.signature", "x".repeat(64 * 1024)),
        ] {
            let acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
            assert_eq!(acct.email, None);
            assert_eq!(acct.display().as_deref(), Some("workspace-1"));
        }
    }

    #[test]
    fn codex_never_uses_access_or_refresh_tokens_for_email_or_identity() {
        let token = display_token(br#"{"email":"wrong@example.com"}"#, false);
        let auth = serde_json::json!({"tokens": {
            "account_id": "workspace-1", "access_token": token, "refresh_token": token
        }});
        let acct = codex_account(&auth.to_string()).unwrap();
        assert_eq!(acct.email, None);
        assert_eq!(acct.display().as_deref(), Some("workspace-1"));

        let missing_id = serde_json::json!({"tokens": {"id_token": token}});
        assert!(codex_account(&missing_id.to_string()).is_none());
        assert!(codex_account(r#"{"auth_mode":"apikey","OPENAI_API_KEY":"dummy"}"#).is_none());
    }

    #[test]
    fn codex_keeps_workspace_ids_distinct_when_email_is_shared() {
        let token = display_token(
            br#"{"email":"person@example.com","https://api.openai.com/auth":{"chatgpt_account_id":"personal-workspace"}}"#,
            false,
        );
        let parse = |id| {
            codex_account(
                &serde_json::json!({"tokens": {
                    "account_id": id, "id_token": token
                }})
                .to_string(),
            )
            .unwrap()
        };
        let personal = parse("personal-workspace");
        let team = parse("team-workspace");
        assert_eq!(personal.email.as_deref(), Some("person@example.com"));
        assert_eq!(team.email, personal.email);
        assert_eq!(personal.account_id.as_deref(), Some("personal-workspace"));
        assert_eq!(team.account_id.as_deref(), Some("team-workspace"));
        assert_ne!(personal.account_id, team.account_id);
    }

    #[test]
    fn codex_inventory_labels_override_detected_email_with_account_id_precedence() {
        let token = display_token(br#"{"email":"person@example.com"}"#, false);
        let mut acct = codex_account(&codex_auth_with_token(serde_json::json!(token))).unwrap();
        let labels = parse_inventory(
            r#"{"labels":{
            "codex:workspace-1":"work", "codex:person@example.com":"personal"
        }}"#,
        );
        apply_labels("codex", &mut acct, &labels);
        assert_eq!(acct.display().as_deref(), Some("work"));
        assert_eq!(acct.email.as_deref(), Some("person@example.com"));

        let labels = parse_inventory(r#"{"labels":{"codex:person@example.com":"personal"}}"#);
        apply_labels("codex", &mut acct, &labels);
        assert_eq!(acct.display().as_deref(), Some("personal"));
    }

    #[test]
    fn agy_account_uses_active_and_ignores_history() {
        let json = r#"{"active":"person@gmail.com","old":["work@example.com"]}"#;

        let acct = agy_account(json).expect("google accounts parses");

        assert_eq!(acct.email.as_deref(), Some("person@gmail.com"));
        assert_eq!(
            acct.display().as_deref(),
            Some("person@gmail.com"),
            "history entries have no credential and must not become the label"
        );
    }

    #[test]
    fn inventory_labels_are_keyed_by_provider_and_account_id_or_email() {
        // Operator-maintained names override the detected id or email.
        let inv = r#"{"schema":1,"labels":{
          "claude:67c22197-ede6-4221-b095-4c89789546dd":"work",
          "codex:3f13fa37-2915-46b3-b975-6f982c7e3c36":"personal",
          "agy:person@gmail.com":"personal"
        }}"#;
        let labels = parse_inventory(inv);

        let mut claude = claude_account(CLAUDE_TEAM_SEAT).unwrap();
        claude.account_id = Some("67c22197-ede6-4221-b095-4c89789546dd".into());
        apply_labels("claude", &mut claude, &labels);
        assert_eq!(claude.display().as_deref(), Some("work"));

        // agy has no account_id, so the email is the key.
        let mut agy = agy_account(r#"{"active":"person@gmail.com"}"#).unwrap();
        apply_labels("agy", &mut agy, &labels);
        assert_eq!(agy.display().as_deref(), Some("personal"));
    }

    #[test]
    fn unmapped_account_keeps_its_email_and_a_bad_inventory_is_ignored() {
        let labels = parse_inventory(r#"{"schema":1,"labels":{"claude:other":"x"}}"#);
        let mut acct = claude_account(CLAUDE_TEAM_SEAT).unwrap();
        apply_labels("claude", &mut acct, &labels);
        assert_eq!(acct.display().as_deref(), Some("user@example.com"));

        assert!(parse_inventory("not json").is_empty());
        assert!(parse_inventory("{}").is_empty());
    }

    #[test]
    fn plans_map_supplies_a_plan_for_providers_that_report_none() {
        // agy exposes no plan locally, so without this the strip can never
        // show one for a live agy account.
        let plans = parse_plans(r#"{"plans":{"agy:me@x.com":"pro"}}"#);
        assert_eq!(plans.get("agy:me@x.com").map(String::as_str), Some("pro"));
        assert!(parse_plans("nope").is_empty());
    }

    #[test]
    fn detect_all_from_keys_each_provider_and_skips_absent_ones() {
        let out = detect_all_from(
            Some(CLAUDE_TEAM_SEAT),
            None, // codex logged out
            Some(r#"{"active":"person@gmail.com","old":[]}"#),
        );

        let providers: Vec<&str> = out.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(
            providers,
            vec!["claude", "agy"],
            "a logged-out provider contributes no row rather than an empty one"
        );
        assert_eq!(out[0].1.email.as_deref(), Some("user@example.com"));
        assert_eq!(out[1].1.email.as_deref(), Some("person@gmail.com"));
    }

    #[test]
    fn extra_claude_accounts_dedupe_the_default_and_carry_their_dir() {
        let second = r#"{"oauthAccount":{"emailAddress":"second@example.com",
          "accountUuid":"acct-2","organizationType":"claude_pro",
          "userRateLimitTier":"default_claude_max_5x"}}"#;
        // The SAME account and the SAME org as the default — a re-login
        // of what is already shown. This is the only true duplicate.
        let dupe_of_default = r#"{"oauthAccount":{"emailAddress":"user@example.com",
          "accountUuid":"acct-1","organizationUuid":"org-team"}}"#;
        let dirs = vec![
            (
                "/home/u/claude-personal".to_string(),
                Some(second.to_string()),
            ),
            (
                "/home/u/claude-dupe".to_string(),
                Some(dupe_of_default.to_string()),
            ),
            ("/home/u/claude-empty".to_string(), None),
        ];

        let out = extra_claude_accounts(&dirs, Some(("acct-1", Some("org-team"))));

        assert_eq!(out.len(), 1, "dupe-of-default and unreadable are skipped");
        let (provider, acct) = &out[0];
        assert_eq!(provider, "claude");
        assert_eq!(acct.email.as_deref(), Some("second@example.com"));
        assert_eq!(
            acct.config_dir.as_deref(),
            Some("/home/u/claude-personal"),
            "the dir is the store key and credential path"
        );
    }

    #[test]
    fn same_account_in_a_different_org_is_a_row_not_a_duplicate() {
        // One claude.ai account can belong to several organizations
        // (a team seat AND a personal free org). Their quotas are
        // separate pools; the CLI login is scoped to ONE org per config
        // dir. Deduping by account id alone would silently discard the
        // second org — seen live 2026-08-14.
        let personal_org = r#"{"oauthAccount":{"emailAddress":"user@example.com",
          "accountUuid":"acct-1","organizationUuid":"org-personal",
          "organizationType":"claude_free"}}"#;
        let dirs = vec![(
            "/home/u/claude-personal".to_string(),
            Some(personal_org.to_string()),
        )];

        let out = extra_claude_accounts(&dirs, Some(("acct-1", Some("org-team"))));

        assert_eq!(
            out.len(),
            1,
            "same account, different org = a different quota pool"
        );
        assert_eq!(out[0].1.org_id.as_deref(), Some("org-personal"));
    }

    #[test]
    fn detect_all_from_is_empty_when_nothing_is_signed_in() {
        assert!(detect_all_from(None, None, None).is_empty());
        assert!(detect_all_from(Some("garbage"), Some("{}"), None).is_empty());
    }

    #[test]
    fn display_prefers_operator_label_then_email_then_id() {
        let mut acct = AccountLabel {
            email: Some("user@example.com".into()),
            account_id: Some("uuid-1".into()),
            ..AccountLabel::default()
        };
        assert_eq!(acct.display().as_deref(), Some("user@example.com"));

        acct.label = Some("work".into());
        assert_eq!(acct.display().as_deref(), Some("work"));

        let id_only = AccountLabel {
            account_id: Some("uuid-1".into()),
            ..AccountLabel::default()
        };
        assert_eq!(id_only.display().as_deref(), Some("uuid-1"));
        assert!(AccountLabel::default().display().is_none());
    }
}
