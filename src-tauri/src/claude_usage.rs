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

use crate::usage_cache::CachedUsage;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

/// Reads only `claudeAiOauth.accessToken`. The refresh token sitting beside it
/// is never read, so it cannot be spent or leaked.
fn read_token() -> Result<String, String> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or("HOME is not set")?;
    let body = std::fs::read_to_string(home.join(".claude/.credentials.json"))
        .map_err(|_| "no ~/.claude/.credentials.json — run `claude` and sign in".to_string())?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("credentials are not valid JSON: {e}"))?;
    v.get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "credentials have no accessToken — sign in again".to_string())
}

/// `claude-code/<installed version>`. The User-Agent is load-bearing: without
/// a real one this endpoint answers 429 with no Retry-After.
fn user_agent() -> String {
    let ver = std::process::Command::new("claude")
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

/// One request, from a click. Returns the same shape the on-disk cache uses so
/// the renderer needs no second code path.
pub async fn fetch(now_ms: u64) -> Result<CachedUsage, String> {
    let token = read_token()?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("client build failed: {e}"))?;
    let resp = client
        .get(USAGE_URL)
        .bearer_auth(&token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("user-agent", user_agent())
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        // Deliberately NOT refreshing: the refresh token rotates, and racing
        // Claude Code's own refresh is what breaks a login.
        return Err("Claude token rejected (401) — sign in again with `claude`".into());
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let retry = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        return Err(format!("rate limited (429), retry-after {retry}"));
    }
    if !status.is_success() {
        return Err(format!("usage endpoint returned HTTP {}", status.as_u16()));
    }
    let body = resp.text().await.map_err(|e| format!("read failed: {e}"))?;
    // The body carries account uuid and email — never log it. The one
    // exception is env-gated (FR-18 style) and prints exactly two
    // identity-free sub-objects, for diagnosing extra-usage shape drift:
    // QHUD_EXTRA_DIAG=1 qhud --claude-usage
    if std::env::var_os("QHUD_EXTRA_DIAG").is_some()
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&body)
    {
        eprintln!(
            "qhud diag extra_usage: {}",
            v.get("extra_usage").unwrap_or(&serde_json::Value::Null)
        );
        eprintln!(
            "qhud diag spend: {}",
            v.get("spend").unwrap_or(&serde_json::Value::Null)
        );
    }
    crate::usage_cache::parse_utilization(&body, now_ms)
        .ok_or_else(|| "usage response did not parse".to_string())
}

#[cfg(test)]
mod tests {
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
