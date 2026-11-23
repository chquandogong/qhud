//! Local prerequisites and the last explicit refresh outcome. No background
//! provider requests, login mutation, or secret-bearing diagnostics.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use base64::{Engine as _, engine::general_purpose};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Ready,
    Connected,
    LoginRequired,
    CredentialExpired,
    CliMissing,
    NotRunning,
    Unknown,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchErrorKind {
    LoginRequired,
    CredentialExpired,
    RateLimited,
    Network,
    InvalidResponse,
    Unavailable,
}

#[derive(Clone, Debug, Serialize)]
pub struct FetchError {
    pub kind: FetchErrorKind,
    pub message: String,
}

impl FetchError {
    pub fn new(kind: FetchErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProviderHealth {
    pub provider: &'static str,
    pub state: HealthState,
    pub source: &'static str,
    pub code: &'static str,
    pub message: &'static str,
    pub action: Option<&'static str>,
    pub checked_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub cli_available: bool,
    pub last_attempt_at_ms: Option<u64>,
    pub last_success_at_ms: Option<u64>,
    pub error_kind: Option<FetchErrorKind>,
    /// In-memory invalidation hints; never part of the webview/diagnostic JSON.
    #[serde(skip)]
    pub(crate) identity_fingerprint: Option<u64>,
    #[serde(skip)]
    pub(crate) credential_modified_at_ms: Option<u64>,
}

impl ProviderHealth {
    pub fn new(provider: &'static str, source: &'static str, now_ms: u64) -> Self {
        Self {
            provider,
            source,
            checked_at_ms: now_ms,
            state: HealthState::Unknown,
            code: "not_checked",
            message: "Connection has not been checked.",
            action: Some("refresh"),
            expires_at_ms: None,
            cli_available: false,
            last_attempt_at_ms: None,
            last_success_at_ms: None,
            error_kind: None,
            identity_fingerprint: None,
            credential_modified_at_ms: None,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct RefreshOutcome {
    attempt: Option<u64>,
    success: Option<u64>,
    error: Option<FetchErrorKind>,
    completed: bool,
    completed_at_ms: Option<u64>,
    identity_fingerprint: Option<u64>,
}

fn outcomes() -> &'static Mutex<HashMap<&'static str, RefreshOutcome>> {
    static OUTCOMES: OnceLock<Mutex<HashMap<&'static str, RefreshOutcome>>> = OnceLock::new();
    OUTCOMES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn begin_refresh(provider: &'static str, now_ms: u64) {
    let identity = current_identity(provider, now_ms);
    if let Ok(mut outcomes) = outcomes().lock() {
        let outcome = outcomes.entry(provider).or_default();
        if outcome.identity_fingerprint != identity {
            *outcome = RefreshOutcome::default();
        }
        outcome.attempt = Some(now_ms);
        outcome.completed = false;
        outcome.identity_fingerprint = identity;
    }
}

pub fn finish_refresh(provider: &'static str, now_ms: u64, result: Result<(), &str>) {
    let identity = current_identity(provider, now_ms);
    if let Ok(mut outcomes) = outcomes().lock() {
        let outcome = outcomes.entry(provider).or_default();
        if outcome.attempt.is_some() && outcome.identity_fingerprint != identity {
            // Login changed while a request was running. Neither its success
            // nor its failure establishes the new login's connection health.
            *outcome = RefreshOutcome::default();
            return;
        }
        outcome.attempt.get_or_insert(now_ms);
        outcome.identity_fingerprint = identity;
        outcome.completed = true;
        outcome.completed_at_ms = Some(now_ms);
        outcome.error = result.err().map(classify_error);
        if result.is_ok() {
            outcome.success = Some(now_ms);
        }
    }
}

fn current_identity(provider: &str, now_ms: u64) -> Option<u64> {
    match provider {
        "codex" => crate::codex_usage::local_health(now_ms).identity_fingerprint,
        "claude" => crate::claude_usage::local_health(now_ms).identity_fingerprint,
        _ => None,
    }
}

pub fn identity_fingerprint(parts: &[&str]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    parts.hash(&mut hasher);
    hasher.finish()
}

pub fn credential_modified_at_ms(path: &std::path::Path) -> Option<u64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let millis = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis();
    u64::try_from(millis).ok()
}

/// Existing provider command errors are already redacted. Classification is
/// bounded and never returns or serializes its input text.
pub fn classify_error(error: &str) -> FetchErrorKind {
    let e = error.to_ascii_lowercase();
    if e.contains("expired") {
        FetchErrorKind::CredentialExpired
    } else if e.contains("401")
        || e.contains("403")
        || e.contains("sign in")
        || e.contains("login")
        || e.contains("credentials unavailable")
        || e.contains("no usable")
        || e.contains("credentials have")
        || e.contains("authentication")
    {
        FetchErrorKind::LoginRequired
    } else if e.contains("429") || e.contains("rate limit") || e.contains("rate-limited") {
        FetchErrorKind::RateLimited
    } else if e.contains("network") || e.contains("timeout") || e.contains("timed out") {
        FetchErrorKind::Network
    } else if e.contains("parse") || e.contains("rpc error") || e.contains("another workspace") {
        FetchErrorKind::InvalidResponse
    } else {
        FetchErrorKind::Unavailable
    }
}

pub fn decorate(mut health: ProviderHealth) -> ProviderHealth {
    let outcome = outcomes()
        .lock()
        .ok()
        .and_then(|all| all.get(health.provider).copied());
    if let Some(outcome) = outcome {
        if !outcome_matches(&health, &outcome) {
            return health;
        }
        health.last_attempt_at_ms = outcome.attempt;
        health.last_success_at_ms = outcome.success;
        health.error_kind = outcome.error;
        if outcome.completed {
            if let Some(error) = outcome.error {
                // Current local evidence is more actionable than a prior
                // generic request failure (for example, an app now stopped).
                if matches!(
                    health.state,
                    HealthState::NotRunning
                        | HealthState::CliMissing
                        | HealthState::LoginRequired
                        | HealthState::CredentialExpired
                ) {
                    return health;
                }
                let (state, code, message) = match error {
                    FetchErrorKind::LoginRequired => (
                        HealthState::LoginRequired,
                        "login_rejected",
                        "The provider requires a fresh sign-in.",
                    ),
                    FetchErrorKind::CredentialExpired => (
                        HealthState::CredentialExpired,
                        "credential_expired",
                        "The local login has expired.",
                    ),
                    FetchErrorKind::RateLimited => (
                        HealthState::Error,
                        "rate_limited",
                        "The provider asked to retry later.",
                    ),
                    FetchErrorKind::Network => (
                        HealthState::Error,
                        "network_error",
                        "The usage request could not reach the provider.",
                    ),
                    FetchErrorKind::InvalidResponse => (
                        HealthState::Error,
                        "invalid_response",
                        "The provider response could not be used.",
                    ),
                    FetchErrorKind::Unavailable => (
                        HealthState::Error,
                        "refresh_unavailable",
                        "The usage source is unavailable.",
                    ),
                };
                health.state = state;
                health.code = code;
                health.message = message;
            } else if matches!(
                health.state,
                HealthState::Ready | HealthState::Unknown | HealthState::CliMissing
            ) {
                health.state = HealthState::Connected;
                health.code = "refresh_succeeded";
                health.message = "The last explicit refresh succeeded.";
            }
        }
    }
    health
}

fn outcome_matches(health: &ProviderHealth, outcome: &RefreshOutcome) -> bool {
    if health.identity_fingerprint != outcome.identity_fingerprint {
        return false;
    }
    // Compare with completion, not start: provider-owned renewal during a
    // successful refresh must not invalidate that just-completed result.
    let reference = if outcome.completed {
        outcome.completed_at_ms
    } else {
        outcome.attempt
    };
    !health
        .credential_modified_at_ms
        .zip(reference)
        .is_some_and(|(modified, completed)| modified > completed)
}

pub fn local_checks(now_ms: u64) -> Vec<ProviderHealth> {
    vec![
        decorate(crate::codex_usage::local_health(now_ms)),
        decorate(crate::claude_usage::local_health(now_ms)),
    ]
}

/// Only presence and expiry leave this parser. JWT expiry is an unverified
/// local hint, never authentication or quota ownership evidence.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AuthMetadata {
    pub has_credential: bool,
    pub expires_at_ms: Option<u64>,
}

pub fn claude_auth_metadata(body: &str) -> Option<AuthMetadata> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let oauth = value.get("claudeAiOauth")?;
    Some(AuthMetadata {
        has_credential: oauth
            .get("accessToken")
            .and_then(|t| t.as_str())
            .is_some_and(|t| !t.is_empty()),
        expires_at_ms: oauth
            .get("expiresAt")
            .and_then(|v| v.as_u64())
            .filter(|v| *v > 0),
    })
}

pub fn codex_auth_metadata(body: &str) -> Option<AuthMetadata> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let tokens = value.get("tokens")?;
    let token = tokens
        .get("access_token")
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty());
    Some(AuthMetadata {
        has_credential: token.is_some()
            && tokens
                .get("account_id")
                .and_then(|v| v.as_str())
                .is_some_and(|v| !v.is_empty()),
        expires_at_ms: token.and_then(jwt_expiry_ms),
    })
}

fn jwt_expiry_ms(token: &str) -> Option<u64> {
    if token.len() > 64 * 1024 {
        return None;
    }
    let mut parts = token.split('.');
    let header = parts.next()?;
    let payload = parts.next()?;
    let signature = parts.next()?;
    if header.is_empty() || payload.is_empty() || signature.is_empty() || parts.next().is_some() {
        return None;
    }
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| general_purpose::URL_SAFE.decode(payload))
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    value.get("exp")?.as_u64()?.checked_mul(1000)
}

pub fn read_metadata(path: &std::path::Path) -> Option<String> {
    // Bound malformed or unexpectedly large local files before reading.
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

pub fn path_binary(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names = if cfg!(windows) {
        vec![
            format!("{name}.exe"),
            format!("{name}.cmd"),
            format!("{name}.bat"),
        ]
    } else {
        vec![name.to_owned()]
    };
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |n| dir.join(n)))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_keeps_only_presence_and_expiry() {
        let m = claude_auth_metadata(r#"{"claudeAiOauth":{"accessToken":"secret-canary","refreshToken":"private-canary","expiresAt":1234}}"#).unwrap();
        assert_eq!(
            m,
            AuthMetadata {
                has_credential: true,
                expires_at_ms: Some(1234)
            }
        );
        assert!(!format!("{m:?}").contains("canary"));
        assert!(claude_auth_metadata("not json").is_none());
        assert!(
            !claude_auth_metadata(r#"{"claudeAiOauth":{"accessToken":""}}"#)
                .unwrap()
                .has_credential
        );
    }

    #[test]
    fn jwt_expiry_is_bounded_unverified_metadata() {
        let payload = general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"exp":1780000000,"email":"private-canary"}"#);
        assert_eq!(
            jwt_expiry_ms(&format!("header.{payload}.signature")),
            Some(1_780_000_000_000)
        );
        assert_eq!(jwt_expiry_ms("invalid"), None);
        assert_eq!(jwt_expiry_ms(&"a".repeat(65537)), None);
        let payload = general_purpose::URL_SAFE_NO_PAD.encode(br#"{"exp":18446744073709551615}"#);
        assert_eq!(jwt_expiry_ms(&format!("header.{payload}.signature")), None);
    }

    #[test]
    fn credential_errors_are_not_mislabelled_as_network_success() {
        assert_eq!(
            classify_error("Claude token rejected (401) — sign in again"),
            FetchErrorKind::LoginRequired
        );
        assert_eq!(
            classify_error("Claude credential expired — open Claude Code"),
            FetchErrorKind::CredentialExpired
        );
        assert_eq!(
            classify_error("rate limited (429)"),
            FetchErrorKind::RateLimited
        );
        assert_eq!(
            classify_error("request failed (timeout)"),
            FetchErrorKind::Network
        );
        assert_eq!(
            classify_error("active credential: usage request failed (network)"),
            FetchErrorKind::Network
        );
    }

    #[test]
    fn local_readiness_is_distinct_from_an_explicit_success() {
        let mut health = ProviderHealth::new("test-provider-health", "fixture", 10);
        health.state = HealthState::Ready;
        assert_eq!(decorate(health.clone()).state, HealthState::Ready);
        begin_refresh(health.provider, 11);
        finish_refresh(health.provider, 12, Ok(()));
        let connected = decorate(health.clone());
        assert_eq!(connected.state, HealthState::Connected);
        assert_eq!(connected.last_success_at_ms, Some(12));
        begin_refresh(health.provider, 13);
        finish_refresh(health.provider, 14, Err("401 private-canary"));
        let rejected = decorate(health);
        assert_eq!(rejected.state, HealthState::LoginRequired);
        assert_eq!(rejected.last_success_at_ms, Some(12));
        assert!(!serde_json::to_string(&rejected).unwrap().contains("canary"));
    }

    #[test]
    fn current_missing_source_is_not_hidden_by_an_old_request_error() {
        let mut health = ProviderHealth::new("test-stopped-source", "fixture", 30);
        health.state = HealthState::NotRunning;
        health.code = "app_not_running";
        finish_refresh(health.provider, 20, Err("request failed (network)"));
        let health = decorate(health);
        assert_eq!(health.state, HealthState::NotRunning);
        assert_eq!(health.code, "app_not_running");
        assert_eq!(health.error_kind, Some(FetchErrorKind::Network));
    }

    #[test]
    fn account_switch_and_new_credentials_invalidate_previous_outcomes() {
        let mut health = ProviderHealth::new("fixture", "fixture", 300);
        health.identity_fingerprint = Some(identity_fingerprint(&["account-a", "org-a"]));
        health.credential_modified_at_ms = Some(150);
        let mut outcome = RefreshOutcome {
            attempt: Some(100),
            completed_at_ms: Some(200),
            completed: true,
            identity_fingerprint: health.identity_fingerprint,
            ..Default::default()
        };
        // Provider-owned token rotation during the request is valid.
        assert!(outcome_matches(&health, &outcome));
        // A later sign-in must clear both a stale success and a stale error.
        health.credential_modified_at_ms = Some(201);
        assert!(!outcome_matches(&health, &outcome));
        outcome.error = Some(FetchErrorKind::LoginRequired);
        assert!(!outcome_matches(&health, &outcome));
        health.credential_modified_at_ms = Some(150);
        health.identity_fingerprint = Some(identity_fingerprint(&["account-a", "org-b"]));
        assert!(!outcome_matches(&health, &outcome));
        let json = serde_json::to_string(&health).unwrap();
        assert!(!json.contains("fingerprint"));
        assert!(!json.contains("credential_modified"));
        assert!(!json.contains("account-a"));
    }
}
