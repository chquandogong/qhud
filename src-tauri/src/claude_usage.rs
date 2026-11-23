//! Claude subscription usage, fetched on an explicit operator click.
//!
//! This is the ONE place qhud sends a credential off the machine, and it
//! exists because nothing else can produce the per-model windows. Verified:
//! the statusLine feed carries only 5h/7d, and nothing qhud can run refreshes
//! `~/.claude.json:cachedUsageUtilization` — not `claude --version`, `doctor`,
//! `mcp list`, nor a real headless `claude --print`. That cache moves only when
//! the operator opens `/usage` themselves, so it is stale exactly when a widget
//! would be useful (measured: Fable 5% cached against 22% actual).
//!
//! Unlike Codex (whose `app-server` RPC lets the CLI own the token) and agy
//! (loopback, no token at all), Claude offers no delegated path. So the rules
//! here are strict:
//!   - never on a timer; only from a click
//!   - never run the OAuth refresh grant (rotating tokens breaks `claude`)
//!   - read the credential fresh each call, never cache or copy it
//!   - the response carries account uuid and email — never log the raw body

use crate::provider_health::{FetchError, FetchErrorKind, HealthState, ProviderHealth};
use crate::usage_cache::CachedUsage;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

/// Reads only `claudeAiOauth.accessToken`. The refresh token sitting beside it
/// is never read, so it cannot be spent or leaked.
fn read_token_at(cred: &std::path::Path) -> Result<String, String> {
    let body = std::fs::read_to_string(cred).map_err(|e| {
        format!(
            "Claude credentials unavailable ({}) — run `claude` and sign in",
            crate::diag::io(&e)
        )
    })?;
    let v: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("credentials are not valid JSON: {}", crate::diag::json(&e)))?;
    v.get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "credentials have no accessToken — sign in again".to_string())
}

fn default_credentials() -> Result<std::path::PathBuf, String> {
    crate::paths::claude_config_dir()
        .map(|dir| dir.join(".credentials.json"))
        .ok_or_else(|| "Claude configuration directory could not be determined".to_string())
}

fn claude_binary() -> Option<std::path::PathBuf> {
    crate::provider_health::path_binary("claude").or_else(|| {
        let home = crate::paths::home_dir()?;
        let path = home.join(if cfg!(windows) {
            ".local/bin/claude.exe"
        } else {
            ".local/bin/claude"
        });
        path.is_file().then_some(path)
    })
}

pub fn local_health(now_ms: u64) -> ProviderHealth {
    local_health_paths(
        default_credentials().ok(),
        crate::paths::claude_config_file(),
        now_ms,
    )
}

/// Local evidence for one isolated profile. Does not merge the provider-wide
/// last-refresh outcome, so one account cannot make another appear connected.
#[cfg(test)]
fn local_health_at(
    config_dir: &std::path::Path,
    config_file: &std::path::Path,
    now_ms: u64,
) -> ProviderHealth {
    local_health_paths(
        Some(config_dir.join(".credentials.json")),
        Some(config_file.to_path_buf()),
        now_ms,
    )
}

fn local_health_paths(
    credential_path: Option<std::path::PathBuf>,
    config_file: Option<std::path::PathBuf>,
    now_ms: u64,
) -> ProviderHealth {
    let mut health = ProviderHealth::new("claude", "claude_code", now_ms);
    health.cli_available = claude_binary().is_some();
    health.credential_modified_at_ms = credential_path
        .as_ref()
        .and_then(|path| crate::provider_health::credential_modified_at_ms(path));
    health.identity_fingerprint = config_file
        .and_then(|path| crate::provider_health::read_metadata(&path))
        .and_then(|body| crate::accounts::claude_account(&body))
        .and_then(|account| {
            account.account_id.map(|id| {
                crate::provider_health::identity_fingerprint(&[
                    "claude",
                    &id,
                    account.org_id.as_deref().unwrap_or(""),
                ])
            })
        });
    let metadata = credential_path
        .and_then(|path| crate::provider_health::read_metadata(&path))
        .and_then(|body| crate::provider_health::claude_auth_metadata(&body));
    health.expires_at_ms = metadata.as_ref().and_then(|m| m.expires_at_ms);
    if metadata.is_none_or(|m| !m.has_credential) {
        health.state = HealthState::LoginRequired;
        health.code = "claude_login_missing";
        health.message = "Sign in through Claude Code; Claude Chat app login is not shared here.";
        health.action = Some("claude_login");
    } else if health
        .expires_at_ms
        .is_some_and(|expires| expires <= now_ms)
    {
        health.state = HealthState::CredentialExpired;
        health.code = "claude_credential_expired";
        health.message = "The Claude Code login has expired. Open Claude Code and sign in again.";
        health.action = Some("claude_login");
    } else {
        health.state = HealthState::Ready;
        health.code = "claude_ready";
        health.message = "Claude Code login was found; refresh to check usage.";
    }
    health
}

/// `claude-code/<installed version>`. The User-Agent is load-bearing: without
/// a real one this endpoint answers 429 with no Retry-After.
fn user_agent() -> String {
    let mut command =
        std::process::Command::new(claude_binary().unwrap_or_else(|| "claude".into()));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let ver = command
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.split_whitespace()
                .find(|w| w.chars().next().is_some_and(|c| c.is_ascii_digit()))
                .map(str::to_string)
        });
    format!("claude-code/{}", ver.unwrap_or_else(|| "2.1.0".into()))
}

/// One ⟳ result for one account. `key` is `"default"` or the expanded
/// config-dir path; `account_id` comes from that dir's own `.claude.json`
/// (the live body's identity fields are deliberately never parsed).
#[derive(serde::Serialize)]
pub struct AccountFetch {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    pub usage: CachedUsage,
}

#[derive(Clone, PartialEq, Eq)]
struct AccountScope {
    account_id: String,
    org_id: Option<String>,
}

fn account_scope(config_json: &std::path::Path) -> Result<AccountScope, String> {
    let account = crate::provider_health::read_metadata(config_json)
        .and_then(|body| crate::accounts::claude_account(&body))
        .ok_or_else(|| {
            "Claude account identity unavailable — open Claude Code and sign in".to_string()
        })?;
    let account_id = account
        .account_id
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            "Claude account identity unavailable — open Claude Code and sign in".to_string()
        })?;
    Ok(AccountScope {
        account_id,
        org_id: account.org_id.filter(|id| !id.is_empty()),
    })
}

fn bind_scope(
    mut usage: CachedUsage,
    before: &AccountScope,
    after: &AccountScope,
) -> Result<CachedUsage, String> {
    if before != after {
        return Err("Claude account or organization changed during refresh; retry".into());
    }
    usage.account_id = Some(before.account_id.clone());
    usage.org_id = before.org_id.clone();
    Ok(usage)
}

async fn fetch_scoped(
    credentials: &std::path::Path,
    config_json: &std::path::Path,
    now_ms: u64,
) -> Result<CachedUsage, String> {
    let before = account_scope(config_json)?;
    let usage = fetch_from(credentials, now_ms).await?;
    let after = account_scope(config_json)?;
    bind_scope(usage, &before, &after)
}

/// ⟳ for every signed-in Claude account: the default (`~/.claude`) plus
/// each registry config dir (D-015). Every success is recorded to the
/// fetched store under its own key. Partial failure is partial: an
/// expired extra account must not hide the default's numbers.
pub async fn fetch_all(now_ms: u64) -> Result<Vec<AccountFetch>, String> {
    let mut out = Vec::new();
    let mut errs = Vec::new();

    let home = crate::paths::home_dir();
    match fetch(now_ms).await {
        Ok(usage) => {
            crate::fetched_store::record_claude(&usage);
            out.push(AccountFetch {
                key: "default".into(),
                account_id: usage.account_id.clone(),
                org_id: usage.org_id.clone(),
                usage,
            });
        }
        Err(e) => errs.push(format!("default: {e}")),
    }

    if let Some(home) = &home {
        let home_str = home.to_string_lossy().to_string();
        for d in crate::registry::load().claude_config_dirs {
            let dir = crate::registry::expand_tilde(&d, &home_str);
            let base = std::path::Path::new(&dir);
            match fetch_scoped(
                &base.join(".credentials.json"),
                &base.join(".claude.json"),
                now_ms,
            )
            .await
            {
                Ok(usage) => {
                    crate::fetched_store::record_claude_extra(&dir, &usage);
                    out.push(AccountFetch {
                        key: dir,
                        account_id: usage.account_id.clone(),
                        org_id: usage.org_id.clone(),
                        usage,
                    });
                }
                Err(e) => errs.push(format!("configured account: {e}")),
            }
        }
    }

    if out.is_empty() {
        return Err(errs.join("; "));
    }
    if !errs.is_empty() {
        eprintln!("qhud: claude fetch partial: {}", errs.join("; "));
    }
    Ok(out)
}

/// One request, from a click, for the default account.
pub async fn fetch(now_ms: u64) -> Result<CachedUsage, String> {
    let config = crate::paths::claude_config_file()
        .ok_or_else(|| "Claude account configuration could not be determined".to_string())?;
    fetch_scoped(&default_credentials()?, &config, now_ms).await
}

/// One request against a specific credential file. Returns the same shape
/// the on-disk cache uses so the renderer needs no second code path.
pub async fn fetch_from(cred: &std::path::Path, now_ms: u64) -> Result<CachedUsage, String> {
    fetch_from_typed(cred, now_ms)
        .await
        .map_err(|error| error.to_string())
}

async fn fetch_from_typed(cred: &std::path::Path, now_ms: u64) -> Result<CachedUsage, FetchError> {
    if crate::provider_health::read_metadata(cred)
        .and_then(|body| crate::provider_health::claude_auth_metadata(&body))
        .and_then(|metadata| metadata.expires_at_ms)
        .is_some_and(|expires| expires <= now_ms)
    {
        return Err(FetchError::new(
            FetchErrorKind::CredentialExpired,
            "Claude credential expired — open Claude Code and sign in again",
        ));
    }
    let token = read_token_at(cred)
        .map_err(|error| FetchError::new(crate::provider_health::classify_error(&error), error))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            FetchError::new(
                FetchErrorKind::Unavailable,
                crate::diag::failure("HTTP client build", crate::diag::reqwest(&e)),
            )
        })?;
    let resp = client
        .get(USAGE_URL)
        .bearer_auth(&token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("user-agent", user_agent())
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            FetchError::new(
                FetchErrorKind::Network,
                crate::diag::failure("usage request", crate::diag::reqwest(&e)),
            )
        })?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        // Deliberately NOT refreshing: the refresh token rotates, and racing
        // Claude Code's own refresh is what breaks a login.
        return Err(FetchError::new(
            FetchErrorKind::LoginRequired,
            format!(
                "Claude token rejected ({}) — sign in again with `claude`",
                status.as_u16()
            ),
        ));
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        // Response headers are provider-controlled too. Do not copy even a
        // syntactically valid Retry-After value into ambient diagnostics.
        return Err(FetchError::new(
            FetchErrorKind::RateLimited,
            "rate limited (429)",
        ));
    }
    if !status.is_success() {
        let code = status.as_u16();
        return Err(FetchError::new(
            FetchErrorKind::Unavailable,
            format!(
                "usage endpoint returned HTTP {code} ({})",
                crate::diag::http_status(code)
            ),
        ));
    }
    let body = resp.text().await.map_err(|e| {
        FetchError::new(
            FetchErrorKind::Network,
            crate::diag::failure("usage response read", crate::diag::reqwest(&e)),
        )
    })?;
    // The body carries account uuid and email — never log it. The env-gated
    // diagnostic reports only field presence, for diagnosing shape drift:
    // QHUD_EXTRA_DIAG=1 qhud --claude-usage
    if std::env::var_os("QHUD_EXTRA_DIAG").is_some()
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&body)
    {
        let extra_usage = v.get("extra_usage").is_some_and(|value| !value.is_null());
        let spend = v.get("spend").is_some_and(|value| !value.is_null());
        eprintln!("qhud diag fields: extra_usage={extra_usage}, spend={spend}");
    }
    crate::usage_cache::parse_utilization_detailed(&body, now_ms).map_err(|e| {
        FetchError::new(
            FetchErrorKind::InvalidResponse,
            format!("usage response did not parse: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn quota_binding_rejects_account_and_organization_switches() {
        use super::{AccountScope, bind_scope};
        let usage = crate::usage_cache::parse_utilization_detailed(
            r#"{"seven_day":{"utilization":23}}"#,
            7,
        )
        .unwrap();
        let before = AccountScope {
            account_id: "same-account".into(),
            org_id: Some("personal-org".into()),
        };
        let bound = bind_scope(usage.clone(), &before, &before).unwrap();
        assert_eq!(bound.account_id.as_deref(), Some("same-account"));
        assert_eq!(bound.org_id.as_deref(), Some("personal-org"));
        for after in [
            AccountScope {
                account_id: "same-account".into(),
                org_id: Some("team-org".into()),
            },
            AccountScope {
                account_id: "another-account".into(),
                org_id: before.org_id.clone(),
            },
            AccountScope {
                account_id: "same-account".into(),
                org_id: None,
            },
        ] {
            let error = bind_scope(usage.clone(), &before, &after).unwrap_err();
            assert!(error.contains("changed during refresh"));
            assert!(!error.contains("same-account"));
            assert!(!error.contains("personal-org"));
            assert!(!error.contains("team-org"));
        }
    }

    #[test]
    fn isolated_profile_health_and_scope_use_only_that_profile() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("qhud-profile-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let config = dir.join(".claude.json");
        let credential = dir.join(".credentials.json");
        let missing = super::local_health_at(&dir, &config, 101);
        assert_eq!(
            missing.state,
            crate::provider_health::HealthState::LoginRequired
        );
        assert!(super::account_scope(&config).is_err());
        std::fs::write(&config,
            r#"{"oauthAccount":{"accountUuid":"fixture-account","organizationUuid":"fixture-org"}}"#,
        ).unwrap();
        std::fs::write(
            &credential,
            r#"{"claudeAiOauth":{"accessToken":"never-send-profile-canary","expiresAt":100}}"#,
        )
        .unwrap();
        let expired = super::local_health_at(&dir, &config, 101);
        assert_eq!(
            expired.state,
            crate::provider_health::HealthState::CredentialExpired
        );
        let scope = super::account_scope(&config).unwrap();
        assert_eq!(scope.account_id, "fixture-account");
        assert_eq!(scope.org_id.as_deref(), Some("fixture-org"));
        let serialized = serde_json::to_string(&expired).unwrap();
        assert!(!serialized.contains("canary"));
        assert!(!serialized.contains("fixture-account"));
        assert!(!serialized.contains("fixture-org"));
        std::fs::remove_file(credential).unwrap();
        std::fs::remove_file(config).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn expired_login_is_reported_before_any_provider_request() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("qhud-expired-{}-{nonce}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"{"claudeAiOauth":{"accessToken":"never-send-canary","expiresAt":100}}"#,
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = runtime.block_on(super::fetch_from_typed(&path, 101));
        std::fs::remove_file(path).unwrap();
        let error = result.err().expect("expired login must fail before HTTP");
        assert_eq!(
            error.kind,
            crate::provider_health::FetchErrorKind::CredentialExpired
        );
        assert!(!error.to_string().contains("canary"));
    }

    #[test]
    fn credential_errors_name_a_category_not_the_path_or_token() {
        let missing = std::path::Path::new("/nonexistent/person@example.com/.credentials.json");
        let err = super::read_token_at(missing).expect_err("the file does not exist");
        assert!(err.contains("(not-found)"), "got {err}");
        assert!(!err.contains("person@example.com"), "path leaked: {err}");
        assert!(!err.contains("/nonexistent"), "path leaked: {err}");

        let dir = std::env::temp_dir().join(format!("qhud-cred-canary-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cred = dir.join(".credentials.json");
        // Truncated mid-object: serde reports EOF, never the token text.
        std::fs::write(
            &cred,
            r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat01-CANARYTOKEN","email":"person@example.com""#,
        )
        .unwrap();
        let err = super::read_token_at(&cred).expect_err("truncated JSON is rejected");
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            err.starts_with("credentials are not valid JSON: "),
            "got {err}"
        );
        assert!(!err.contains("CANARYTOKEN"), "token leaked: {err}");
        assert!(!err.contains('@'), "email leaked: {err}");
        assert!(!err.contains("qhud-cred-canary"), "path leaked: {err}");
    }

    #[test]
    fn user_agent_is_claude_code_shaped() {
        // Wrong or absent UA earns a 429 with no Retry-After, so the shape
        // matters more than the exact version.
        let ua = super::user_agent();
        assert!(ua.starts_with("claude-code/"), "got {ua}");
        assert!(
            ua.chars().last().is_some_and(|c| c.is_ascii_digit()),
            "version must end in a digit, got {ua}"
        );
    }
}
