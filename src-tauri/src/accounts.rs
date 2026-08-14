//! Account identity for quota rows.
//!
//! Quota is an account fact (D-011), but v0.3 rendered one row per
//! provider with no indication of *whose* quota it was. On a machine
//! where the operator holds several logins per provider — and swaps
//! between them — an unlabelled percentage is ambiguous.
//!
//! Every source read here is a plain local file, so this module makes
//! no network calls and never touches a token. It reads only the
//! identity fields the CLIs already persist in cleartext; credential
//! material is never opened.

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
}

/// Parses `~/.codex/auth.json`. Only `tokens.account_id` is read; the
/// sibling token fields are deliberately never touched. Codex keeps no
/// cleartext email, so the display name comes from the inventory file.
pub fn codex_account(auth_json: &str) -> Option<AccountLabel> {
    let auth: CodexAuth = serde_json::from_str(auth_json).ok()?;
    let account_id = auth.tokens?.account_id?;
    Some(AccountLabel {
        account_id: Some(account_id),
        ..AccountLabel::default()
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
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        return Vec::new();
    };
    let read = |rel: &str| std::fs::read_to_string(home.join(rel)).ok();
    let mut found = detect_all_from(
        read(".claude.json").as_deref(),
        read(".codex/auth.json").as_deref(),
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

    let labels = read(".config/qhud/accounts.json")
        .map(|s| parse_inventory(&s))
        .unwrap_or_default();
    let plans = read(".config/qhud/accounts.json")
        .map(|s| parse_plans(&s))
        .unwrap_or_default();
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
    fn codex_account_reads_only_the_account_id() {
        let json = r#"{"auth_mode":"chatgpt","tokens":{
          "id_token":"REDACTED","access_token":"REDACTED","refresh_token":"REDACTED",
          "account_id":"3f13fa37-2915-46b3-b975-6f982c7e3c36"},
          "last_refresh":"2026-08-06T01:32:09Z"}"#;

        let acct = codex_account(json).expect("codex auth parses");

        assert_eq!(
            acct.account_id.as_deref(),
            Some("3f13fa37-2915-46b3-b975-6f982c7e3c36")
        );
        assert!(acct.email.is_none(), "codex keeps no cleartext email");
    }

    #[test]
    fn codex_account_absent_when_logged_out() {
        assert!(codex_account(r#"{"auth_mode":null,"tokens":null}"#).is_none());
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
        // Operator-maintained: the CLIs expose ids and emails but no
        // human name, and codex exposes no email at all.
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
