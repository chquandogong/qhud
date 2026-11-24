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
///
/// `Deserialize` exists for qhud's own fetched-usage store round-trip
/// (`fetched_store`), not for any provider wire format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageWindow {
    /// `5h`, `daily`, `weekly`, `30d`, or `Nm` when it matches nothing known.
    pub label: String,
    pub used_percent: u8,
    #[serde(default)]
    pub reset_unix: Option<u64>,
    /// Which pool this window belongs to, when it is NOT the account's
    /// main one (e.g. a per-model limit like "GPT-5.3-Codex-Spark").
    /// Duration alone labels both pools "weekly" — one name over two
    /// different values is the D-011 mistake all over again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkspaceUsage {
    pub account_id: String,
    /// Login email from this source's local ID token. It is display metadata,
    /// and also prevents two known users in one workspace being coalesced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Workspace display name, when `accounts/check` supplied one.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub plan_type: Option<String>,
    #[serde(default)]
    pub windows: Vec<UsageWindow>,
    #[serde(default)]
    pub credits_balance: Option<String>,
    /// Earned manual rate-limit resets still available. Missing is unknown,
    /// distinct from a provider-reported zero. This never consumes a reset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_credits: Option<u32>,
    /// True for the ACTIVE login's own workspace (the default
    /// `auth.json`). Its numbers are the same pool the pane statusline
    /// reports, so it merges into the provider row instead of rendering
    /// a second, clock-skewed copy of the same 7D window (D-011).
    #[serde(default)]
    pub active: bool,
}

/// The active workspace's fetch result in the snapshot shape, so the
/// existing provider-row merge (`view::attach_usage_cache`) can absorb
/// it: unscoped 5h/weekly windows become the primary gauges, every
/// scoped pool becomes a `pool_<label>` scoped window.
pub fn workspace_snapshot(
    ws: &WorkspaceUsage,
    fetched_at_ms: u64,
) -> crate::usage_cache::CachedUsage {
    use crate::usage_cache::{CachedUsage, CachedWindow, ScopedLimit};
    let mut cu = CachedUsage {
        fetched_at_ms,
        account_id: Some(ws.account_id.clone()),
        org_id: None,
        plan_name: None,
        five_hour: None,
        seven_day: None,
        scoped: Vec::new(),
        extra: None,
        reset_credits: ws.reset_credits,
    };
    for w in &ws.windows {
        let window = CachedWindow {
            pct: w.used_percent,
            reset_unix: w.reset_unix,
        };
        match (w.scope.as_ref(), w.label.as_str()) {
            (None, "5h") => cu.five_hour = Some(window),
            (None, "weekly") => cu.seven_day = Some(window),
            (scope, label) => {
                // Scoped pools keep their name; an unscoped window of an
                // unexpected duration (30d, daily) falls back to the
                // main pool's name rather than vanishing.
                cu.scoped.push(ScopedLimit {
                    kind: format!("pool_{label}"),
                    scope: Some(scope.cloned().unwrap_or_else(|| "codex".to_string())),
                    pct: w.used_percent,
                    reset_unix: w.reset_unix,
                });
            }
        }
    }
    cu
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
    /// The account the SERVER says this body describes. Compared against the
    /// account we asked for: if `chatgpt-account-id` is ignored or the token
    /// is not authorized for that workspace, the server answers with the
    /// default context and we would render one workspace's numbers twice
    /// under two different labels — worse than showing one.
    #[serde(default)]
    account_id: Option<String>,
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
    /// Human pool name ("GPT-5.3-Codex-Spark"); the window's scope.
    #[serde(default)]
    limit_name: Option<String>,
    /// Wire id ("codex_bengalfox") — the fallback when no name is given.
    #[serde(default)]
    metered_feature: Option<String>,
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
            scope: None,
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
    if let Some(got) = parsed.account_id.as_deref()
        && got != account_id
    {
        eprintln!(
            "qhud: scope mismatch; chatgpt-account-id was not honoured, so this \
             reading is dropped rather than mislabelled"
        );
        return None;
    }
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
            .filter_map(|a| {
                // The pool name is what distinguishes this window from the
                // main one — both are "weekly" by duration.
                let scope = a.limit_name.or(a.metered_feature);
                Some((scope, a.rate_limit?))
            })
            .flat_map(|(scope, rl)| {
                rl.windows()
                    .map(move |mut w| {
                        w.scope = scope.clone();
                        w
                    })
                    .collect::<Vec<_>>()
            }),
    );
    Some(WorkspaceUsage {
        account_id: account_id.to_string(),
        email: None,
        name: None,
        plan_type: parsed.plan_type,
        windows,
        credits_balance: parsed.credits.and_then(|c| c.balance),
        // The HTTP schema used here has no verified reset-credit field.
        reset_credits: None,
        active: false,
    })
}

/// Parses one `account/rateLimits/read` JSON-RPC response line from
/// `codex app-server` into the same `WorkspaceUsage` shape the raw HTTP
/// path produces (so the renderer and store need no second code path).
///
/// The response's `rateLimitsByLimitId` carries every pool — the main
/// `codex` one and per-model pools like `codex_bengalfox` — each with
/// `usedPercent`, `windowDurationMins` (MINUTES, unlike the HTTP path's
/// seconds) and `resetsAt` unix seconds. Plan and credits sit on the
/// top-level `rateLimits`.
pub fn parse_app_server_rate_limits(line: &str, account_id: &str) -> Option<WorkspaceUsage> {
    #[derive(Deserialize)]
    struct Rpc {
        id: Option<u64>,
        result: Option<RpcResult>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RpcResult {
        rate_limits: Option<Limits>,
        #[serde(default)]
        rate_limits_by_limit_id: Option<std::collections::BTreeMap<String, Limits>>,
        // Optional metadata must not make otherwise valid quota unreadable
        // when a server returns an unexpected reset-credit shape.
        #[serde(default)]
        rate_limit_reset_credits: Option<serde_json::Value>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Limits {
        #[serde(default)]
        limit_id: Option<String>,
        #[serde(default)]
        limit_name: Option<String>,
        #[serde(default)]
        primary: Option<Win>,
        #[serde(default)]
        secondary: Option<Win>,
        #[serde(default)]
        credits: Option<AppCredits>,
        #[serde(default)]
        plan_type: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Win {
        #[serde(default)]
        used_percent: Option<f64>,
        #[serde(default)]
        window_duration_mins: Option<u64>,
        #[serde(default)]
        resets_at: Option<i64>,
    }
    #[derive(Deserialize)]
    struct AppCredits {
        #[serde(default)]
        balance: Option<String>,
    }

    let win = |w: Win| -> Option<UsageWindow> {
        Some(UsageWindow {
            label: window_label(w.window_duration_mins.unwrap_or(0) * 60),
            used_percent: w.used_percent?.round().clamp(0.0, 100.0) as u8,
            reset_unix: w.resets_at.and_then(|t| u64::try_from(t).ok()),
            scope: None,
        })
    };

    let rpc: Rpc = serde_json::from_str(line).ok()?;
    if rpc.id != Some(2) {
        return None;
    }
    let result = rpc.result?;
    // https://learn.chatgpt.com/docs/app-server#6-rate-limits-chatgpt
    // availableCount is authoritative; detail rows can be capped or null.
    // Do not infer availability from the length/statuses of `credits`.
    let reset_credits = result
        .rate_limit_reset_credits
        .as_ref()
        .and_then(|credits| credits.get("availableCount"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|count| u32::try_from(count).ok());
    let top = result.rate_limits;
    let mut by_id = result.rate_limits_by_limit_id.unwrap_or_default();
    if top.is_none() && by_id.is_empty() && reset_credits.is_none() {
        return None;
    }

    let main_id = top
        .as_ref()
        .and_then(|l| l.limit_id.as_deref())
        .unwrap_or("codex");
    let mapped_main = by_id.remove(main_id);
    let plan_type = top
        .as_ref()
        .and_then(|l| l.plan_type.clone())
        .or_else(|| mapped_main.as_ref().and_then(|l| l.plan_type.clone()));
    let credits_balance = top
        .as_ref()
        .and_then(|l| l.credits.as_ref())
        .and_then(|c| c.balance.clone())
        .or_else(|| {
            mapped_main
                .as_ref()
                .and_then(|l| l.credits.as_ref())
                .and_then(|c| c.balance.clone())
        });

    // Prefer the main pool in the map, but retain the top-level main
    // when the map contains only extra pools. Consume it once so the
    // same weekly window is never rendered twice. A null top-level
    // value must not hide real per-model readings from the map.
    let pools = mapped_main.or(top).into_iter().map(|l| (None, l)).chain(
        by_id
            .into_iter()
            .map(|(id, l)| (Some(l.limit_name.clone().unwrap_or(id)), l)),
    );
    let windows = pools
        .flat_map(|(scope, l)| {
            [l.primary, l.secondary]
                .into_iter()
                .flatten()
                .filter_map(win)
                .map(move |mut w| {
                    w.scope = scope.clone();
                    w
                })
        })
        .collect();
    Some(WorkspaceUsage {
        account_id: account_id.to_string(),
        email: None,
        name: None,
        plan_type,
        windows,
        credits_balance,
        reset_credits,
        active: false,
    })
}

/// Prefer an explicit CLI, then PATH and supported per-user Windows layouts.
/// Desktop app updates must not depend on Explorer having a fresh PATH.
fn discover_codex_binary() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("QHUD_CODEX_BIN").filter(|p| !p.is_empty()) {
        return Some(path.into());
    }
    if let Some(path) = crate::provider_health::path_binary("codex") {
        return Some(path);
    }
    #[cfg(windows)]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            if let Some(binary) = windows_codex_binary(&std::path::PathBuf::from(local)) {
                return Some(binary);
            }
        }
    }
    None
}

#[cfg(any(windows, test))]
fn windows_codex_binary(local: &std::path::Path) -> Option<std::path::PathBuf> {
    for root in [
        local.join("Programs/OpenAI/Codex/bin"),
        local.join("OpenAI/Codex/bin"),
    ] {
        let direct = root.join("codex.exe");
        if direct.is_file() {
            return Some(direct);
        }
        if let Some(binary) = desktop_codex_binary(&root) {
            return Some(binary);
        }
    }
    None
}

fn codex_binary() -> std::path::PathBuf {
    discover_codex_binary().unwrap_or_else(|| "codex".into())
}

pub fn local_health(now_ms: u64) -> crate::provider_health::ProviderHealth {
    use crate::provider_health::{HealthState, ProviderHealth};
    let mut health = ProviderHealth::new("codex", "codex_app_server", now_ms);
    health.cli_available = discover_codex_binary().is_some_and(|path| path.is_file());
    let auth_path = crate::paths::codex_home().map(|dir| dir.join("auth.json"));
    let body = auth_path
        .as_ref()
        .and_then(|path| crate::provider_health::read_metadata(path));
    health.credential_modified_at_ms = auth_path
        .as_ref()
        .and_then(|path| crate::provider_health::credential_modified_at_ms(path));
    health.identity_fingerprint = body
        .as_deref()
        .and_then(crate::accounts::codex_account)
        .and_then(|account| {
            account.account_id.map(|id| {
                crate::provider_health::identity_fingerprint(&[
                    "codex",
                    &id,
                    &account.email.unwrap_or_default().to_ascii_lowercase(),
                ])
            })
        });
    let metadata = body
        .as_deref()
        .and_then(crate::provider_health::codex_auth_metadata);
    health.expires_at_ms = metadata.as_ref().and_then(|m| m.expires_at_ms);
    if metadata.is_none_or(|m| !m.has_credential) {
        health.state = HealthState::LoginRequired;
        health.code = "codex_login_missing";
        health.message = "No readable Codex subscription login was found.";
        health.action = Some("codex_login");
    } else if !health.cli_available {
        health.state = HealthState::CliMissing;
        health.code = "codex_cli_missing";
        health.message = "Codex CLI was not found; automatic login recovery is unavailable.";
        health.action = Some("install_codex_cli");
    } else {
        health.state = HealthState::Ready;
        health.code = if health
            .expires_at_ms
            .is_some_and(|expires| expires <= now_ms)
        {
            "codex_refresh_needed"
        } else {
            "codex_ready"
        };
        health.message = "Codex login and CLI were found; refresh to check usage.";
    }
    health
}

#[cfg(any(windows, test))]
fn desktop_codex_binary(root: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path().join("codex.exe"))
        .filter(|path| path.is_file())
        .max_by_key(|path| path.metadata().ok().and_then(|meta| meta.modified().ok()))
}

/// Fetches the ACTIVE login's usage by asking a short-lived
/// `codex app-server` child (read-only sandbox, on-request approvals) —
/// Codex owns all credentials and its own token rotation, so this path
/// still works when the on-disk access token has expired and the raw
/// HTTP path answers 401. qhud reads no token at all here.
pub async fn fetch_via_app_server() -> Result<WorkspaceUsage, String> {
    let home = crate::paths::codex_home().ok_or("Codex home could not be determined")?;
    fetch_via_app_server_at(&home).await
}

fn app_server_command(
    binary: &std::path::Path,
    home: &std::path::Path,
    force_file_store: bool,
) -> std::process::Command {
    let mut command = std::process::Command::new(binary);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    command.args(["-s", "read-only", "-a", "on-request"]);
    if force_file_store {
        command.args(["-c", "cli_auth_credentials_store=\"file\""]);
    }
    command.arg("app-server").env("CODEX_HOME", home);
    command
}

/// Each registered home keeps its own login. CODEX_HOME is set on this child
/// only, so observing another account never switches the user's active login.
pub async fn fetch_via_app_server_at(home: &std::path::Path) -> Result<WorkspaceUsage, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // The workspace this login is scoped to, for labelling — an identity
    // field read locally, not a credential.
    let account = std::fs::read_to_string(home.join("auth.json"))
        .ok()
        .and_then(|s| crate::accounts::codex_account(&s))
        .ok_or("no Codex auth.json identity — run `codex login`")?;
    let account_id = account
        .account_id
        .as_deref()
        .ok_or("no Codex auth.json identity — run `codex login`")?;

    let normalized =
        |path: &std::path::Path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let is_default_home =
        crate::paths::codex_home().is_some_and(|default| normalized(&default) == normalized(home));
    let mut child =
        tokio::process::Command::from(app_server_command(&codex_binary(), home, !is_default_home))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| crate::diag::failure("codex app-server spawn", crate::diag::spawn(&e)))?;

    let mut stdin = child.stdin.take().ok_or("no stdin handle")?;
    let stdout = child.stdout.take().ok_or("no stdout handle")?;

    let handshake = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"qhud","title":"qhud","version":"0.5.0"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","method":"initialized"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"account/rateLimits/read"}"#,
        "\n"
    );
    stdin
        .write_all(handshake.as_bytes())
        .await
        .map_err(|e| crate::diag::failure("app-server write", crate::diag::io(&e)))?;
    // Keep stdin open: the server exits on EOF, possibly before answering.

    let read = async {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(mut usage) = parse_app_server_rate_limits(&line, account_id) {
                let after = std::fs::read_to_string(home.join("auth.json"))
                    .ok()
                    .and_then(|body| crate::accounts::codex_account(&body));
                if !after.as_ref().is_some_and(|after| {
                    same_login_workspace(
                        account_id,
                        account.email.as_deref(),
                        after.account_id.as_deref().unwrap_or(""),
                        after.email.as_deref(),
                    )
                }) {
                    return Err("Codex account changed during refresh; retry".to_string());
                }
                usage.email = account.email.clone();
                return Ok(usage);
            }
            // An explicit RPC error for our request id is a real answer too.
            if line.contains("\"id\":2") && line.contains("\"error\"") {
                // RPC error payloads can include account-scoped data. Keep the
                // diagnostic useful without copying the server response into
                // stderr or the UI error surface.
                return Err("app-server returned an RPC error".to_string());
            }
        }
        Err("app-server closed without answering".to_string())
    };
    let out = tokio::time::timeout(std::time::Duration::from_secs(25), read)
        .await
        .unwrap_or_else(|_| Err("app-server timed out after 25s".into()));
    let _ = child.kill().await;
    out
}
// `accounts/check` answers with an ARRAY of workspace entries, each carrying
// its own `id` and `plan_type`, with `structure: "workspace"`. It is NOT a map
// keyed by account id — parsing it as one reported "no data" on a 200.
#[derive(Deserialize)]
struct AccountsBody {
    #[serde(default)]
    accounts: Vec<AccountEntry>,
}

// Entries also carry `structure: "workspace"` on the wire; it is not
// needed for anything qhud renders, so it is not modelled.
#[derive(Deserialize)]
struct AccountEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    plan_type: Option<String>,
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

/// Known different login emails must never share a seat's quota, even when
/// both logins belong to the same team workspace. Missing email is not proof
/// that a known email is the same user.
pub fn same_login_workspace(
    id: &str,
    email: Option<&str>,
    other_id: &str,
    other_email: Option<&str>,
) -> bool {
    id == other_id
        && match (email, other_email) {
            (Some(a), Some(b)) => a.trim().eq_ignore_ascii_case(b.trim()),
            (None, None) => true,
            _ => false,
        }
}

fn configured_homes() -> Vec<(std::path::PathBuf, bool)> {
    let Some(default_dir) = crate::paths::codex_home() else {
        return Vec::new();
    };
    let resolved = |path: std::path::PathBuf| path.canonicalize().unwrap_or(path);
    let mut dirs = vec![(resolved(default_dir), true)];
    if let Some(home) = crate::paths::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        for d in crate::registry::load().codex_homes {
            let dir = resolved(std::path::PathBuf::from(crate::registry::expand_tilde(
                &d, &home_str,
            )));
            if !dirs.iter().any(|(known, _)| *known == dir) {
                dirs.push((dir, false));
            }
        }
    }
    dirs
}

// Intentionally not Debug/Serialize: contains a provider access token.
struct AuthSource {
    token: String,
    account_id: String,
    email: Option<String>,
    active: bool,
}

/// Read fresh after the CLI has had a chance to renew its own login. The
/// refresh token is never selected, copied, persisted, or sent by qhud.
fn read_all_auth() -> Vec<AuthSource> {
    let mut out: Vec<AuthSource> = Vec::new();
    for (dir, default_home) in configured_homes() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut paths: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n == "auth.json" || n.starts_with("auth.json."))
            })
            .collect();
        // Deterministic order, with the live file first so it wins any dedupe.
        paths.sort();
        paths.sort_by_key(|p| p.file_name().and_then(|n| n.to_str()) != Some("auth.json"));

        for path in paths {
            let Ok(body) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) else {
                continue;
            };
            let tok = v
                .pointer("/tokens/access_token")
                .and_then(|t| t.as_str())
                .filter(|t| !t.is_empty());
            let account = crate::accounts::codex_account(&body);
            if let Some(tok) = tok
                && let Some(account) = account
                && let Some(id) = account.account_id
                && !out.iter().any(|known| {
                    same_login_workspace(
                        &id,
                        account.email.as_deref(),
                        &known.account_id,
                        known.email.as_deref(),
                    )
                })
            {
                out.push(AuthSource {
                    token: tok.to_string(),
                    account_id: id,
                    email: account.email,
                    active: default_home
                        && path.file_name().and_then(|n| n.to_str()) == Some("auth.json"),
                });
            }
        }
    }
    out
}

/// Additional workspaces need server-supplied ownership evidence. A server
/// that ignores the requested workspace header must not duplicate the token's
/// default quota under a different label; an omitted account_id is ambiguous.
fn parse_scoped_usage(
    account_id: &str,
    body: &str,
    require_identity: bool,
) -> Option<WorkspaceUsage> {
    if require_identity {
        let value: serde_json::Value = serde_json::from_str(body).ok()?;
        if value.get("account_id").and_then(|id| id.as_str()) != Some(account_id) {
            return None;
        }
    }
    parse_usage(account_id, body)
}

// NOTE (kept from the deleted single-credential reader): the id_token is
// NOT interchangeable with the access token — as a Bearer it returns 401
// token_expired even while the access token is still valid.

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
        .map_err(|e| crate::diag::failure("Codex request", crate::diag::reqwest(&e)))?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        // Deliberately NOT refreshing: Codex refresh tokens are single-use
        // and rotated, and a failed write-back breaks `codex login`.
        return Err("Codex token rejected (401) — run `codex login`".into());
    }
    if !status.is_success() {
        let code = status.as_u16();
        return Err(format!(
            "Codex usage returned HTTP {code} ({})",
            crate::diag::http_status(code)
        ));
    }
    resp.text()
        .await
        .map_err(|e| crate::diag::failure("Codex response read", crate::diag::reqwest(&e)))
}

/// Fetches usage for every workspace the signed-in Codex login owns.
///
/// One network round per workspace plus one to enumerate them. Invoked
/// from an explicit operator gesture only — never from the poll loop.
pub async fn fetch_all_workspaces() -> Result<Vec<WorkspaceUsage>, String> {
    // Every registered home delegates to the provider-owned app-server. The
    // child receives that home only; no process-global environment or default
    // login is changed. Duplicate logins do not race their token rotation.
    let mut out: Vec<WorkspaceUsage> = Vec::new();
    let mut attempted: Vec<(String, Option<String>)> = Vec::new();
    let mut last_err = None;
    for (home, active) in configured_homes() {
        let account = std::fs::read_to_string(home.join("auth.json"))
            .ok()
            .and_then(|body| crate::accounts::codex_account(&body));
        let Some(account) = account else { continue };
        let Some(id) = account.account_id else {
            continue;
        };
        if attempted.iter().any(|(known_id, known_email)| {
            same_login_workspace(
                &id,
                account.email.as_deref(),
                known_id,
                known_email.as_deref(),
            )
        }) {
            continue;
        }
        attempted.push((id, account.email));
        match fetch_via_app_server_at(&home).await {
            Ok(mut usage) => {
                usage.active = active;
                out.push(usage);
            }
            Err(error) => {
                eprintln!("qhud: codex configured app-server unavailable: {error}");
                last_err = Some(error);
            }
        }
    }
    // Read credentials after all provider-owned rotations, never before.
    let creds = read_all_auth();
    if creds.is_empty() {
        return if out.is_empty() {
            Err(last_err.unwrap_or_else(|| {
                "no usable Codex auth.json* credential — run `codex login`".into()
            }))
        } else {
            Ok(out)
        };
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| crate::diag::failure("HTTP client build", crate::diag::reqwest(&e)))?;

    eprintln!("qhud: codex credentials found: {}", creds.len());
    for credential in creds {
        let mut candidates = match get(
            &client,
            ACCOUNTS_URL,
            &credential.token,
            Some(&credential.account_id),
        )
        .await
        {
            Ok(body) => parse_accounts(&body),
            Err(_) => Vec::new(),
        };
        if !candidates
            .iter()
            .any(|(id, _, _)| *id == credential.account_id)
        {
            candidates.insert(0, (credential.account_id.clone(), None, None));
        }
        for (id, name, plan) in candidates {
            if let Some(existing) = out.iter_mut().find(|usage| {
                same_login_workspace(
                    &usage.account_id,
                    usage.email.as_deref(),
                    &id,
                    credential.email.as_deref(),
                )
            }) {
                if name.is_some() {
                    existing.name = name;
                }
                if existing.plan_type.is_none() {
                    existing.plan_type = plan;
                }
                continue;
            }
            let own_workspace = id == credential.account_id;
            match get(&client, USAGE_URL, &credential.token, Some(&id)).await {
                Ok(body) => match parse_scoped_usage(&id, &body, !own_workspace) {
                    Some(mut usage) => {
                        usage.email = credential.email.clone();
                        usage.name = name;
                        if usage.plan_type.is_none() {
                            usage.plan_type = plan;
                        }
                        usage.active = credential.active && own_workspace;
                        out.push(usage);
                    }
                    None => {
                        eprintln!(
                            "qhud: codex workspace omitted: response ownership not established"
                        );
                        last_err = Some(
                            "Codex usage response did not establish workspace ownership".into(),
                        );
                    }
                },
                Err(error) => {
                    eprintln!("qhud: codex configured workspace unavailable: {error}");
                    last_err = Some(error);
                }
            }
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

    #[test]
    fn two_logins_in_the_same_workspace_keep_separate_seat_readings() {
        assert!(!same_login_workspace(
            "team",
            Some("first@example.test"),
            "team",
            Some("second@example.test")
        ));
        assert!(same_login_workspace(
            "team",
            Some("First@Example.Test"),
            "team",
            Some("first@example.test")
        ));
        assert!(!same_login_workspace(
            "personal",
            Some("first@example.test"),
            "team",
            Some("first@example.test")
        ));
        assert!(!same_login_workspace(
            "team",
            None,
            "team",
            Some("first@example.test")
        ));
        assert!(same_login_workspace("team", None, "team", None));
    }

    #[test]
    fn cross_workspace_usage_requires_explicit_matching_server_identity() {
        let missing_identity = r#"{"plan_type":"team","rate_limit":{"primary_window":{"used_percent":9,"limit_window_seconds":604800}}}"#;
        assert!(parse_scoped_usage("team", missing_identity, true).is_none());
        assert!(parse_scoped_usage("personal", missing_identity, false).is_some());
        let mut body: serde_json::Value = serde_json::from_str(missing_identity).unwrap();
        body["account_id"] = serde_json::json!("personal");
        assert!(parse_scoped_usage("team", &body.to_string(), true).is_none());
        body["account_id"] = serde_json::json!("team");
        let accepted = parse_scoped_usage("team", &body.to_string(), true).unwrap();
        assert_eq!(accepted.windows[0].used_percent, 9);
    }

    #[test]
    fn extra_home_is_command_local_and_does_not_change_parent_environment() {
        let before = std::env::var_os("CODEX_HOME");
        let home = std::path::Path::new("fixture/extra-home");
        let command = app_server_command(std::path::Path::new("fixture/codex"), home, false);
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == "CODEX_HOME")
                .and_then(|(_, value)| value),
            Some(home.as_os_str())
        );
        assert_eq!(std::env::var_os("CODEX_HOME"), before);
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec!["-s", "read-only", "-a", "on-request", "app-server"]
        );
        let isolated = app_server_command(std::path::Path::new("fixture/codex"), home, true);
        assert_eq!(
            isolated.get_args().collect::<Vec<_>>(),
            vec![
                "-s",
                "read-only",
                "-a",
                "on-request",
                "-c",
                "cli_auth_credentials_store=\"file\"",
                "app-server"
            ]
        );
    }

    #[test]
    fn windows_finds_current_installer_layout_before_legacy_layout() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("qhud-codex-layout-{}-{nonce}", std::process::id()));
        let current = root.join("Programs/OpenAI/Codex/bin/codex.exe");
        let legacy = root.join("OpenAI/Codex/bin/v1/codex.exe");
        for path in [&current, &legacy] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, []).unwrap();
        }
        assert_eq!(windows_codex_binary(&root), Some(current.clone()));
        std::fs::remove_file(&current).unwrap();
        assert_eq!(windows_codex_binary(&root), Some(legacy.clone()));
        std::fs::remove_file(&legacy).unwrap();
        assert_eq!(windows_codex_binary(&root), None);
        // Remove only the uniquely-created fixture directory.
        assert!(
            root.canonicalize()
                .unwrap()
                .starts_with(std::env::temp_dir().canonicalize().unwrap())
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_cli_discovery_uses_latest_complete_version() {
        use std::fs::{self, File, FileTimes};
        use std::time::{Duration, SystemTime};

        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("qhud-codex-bin-{}-{nonce}", std::process::id()));
        for name in ["old", "new", "incomplete"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        for (name, seconds) in [("old", 1_700_000_000), ("new", 1_700_001_000)] {
            File::create(root.join(name).join("codex.exe"))
                .unwrap()
                .set_times(
                    FileTimes::new()
                        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
                )
                .unwrap();
        }
        assert_eq!(
            desktop_codex_binary(&root),
            Some(root.join("new/codex.exe"))
        );
        assert_eq!(desktop_codex_binary(&root.join("missing")), None);

        for name in ["old", "new"] {
            fs::remove_file(root.join(name).join("codex.exe")).unwrap();
        }
        for name in ["old", "new", "incomplete"] {
            fs::remove_dir(root.join(name)).unwrap();
        }
        fs::remove_dir(root).unwrap();
    }

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
    fn usage_is_dropped_when_the_body_describes_another_workspace() {
        // The server may ignore chatgpt-account-id and answer with the default
        // context. Accepting that would render one workspace's numbers under
        // another's label. A live response had a different workspace ID
        // and reset timestamps one second apart instead of days.
        let body = r#"{"account_id":"other","plan_type":"team",
          "rate_limit":{"primary_window":{"used_percent":0,
            "limit_window_seconds":604800,"reset_at":1786698607}}}"#;

        assert!(
            parse_usage("asked-for", body).is_none(),
            "a mislabelled reading is worse than a missing one"
        );
        assert!(
            parse_usage("other", body).is_some(),
            "a matching account_id still parses"
        );
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
    fn per_model_windows_carry_their_pool_name_the_main_one_does_not() {
        // Both pools are 7-day windows, so duration alone labels them
        // both "weekly" — two chips with one name and different values
        // (the operator read that as a bug, and rightly so). The pool
        // NAME is the distinguishing fact and must survive parsing.
        let u = parse_usage("acct-1", USAGE_BODY).unwrap();

        let main = u
            .windows
            .iter()
            .find(|w| w.used_percent == 80)
            .expect("main pool window");
        assert_eq!(main.scope, None, "the account's main pool is unscoped");

        let spark = u
            .windows
            .iter()
            .find(|w| w.used_percent == 4)
            .expect("per-model pool window");
        assert_eq!(
            spark.scope.as_deref(),
            Some("GPT-5.3-Codex-Spark"),
            "the per-model pool keeps its limit_name"
        );
        assert_eq!(spark.reset_unix, Some(1786678081));
    }

    #[test]
    fn http_usage_preserves_both_spark_windows_with_their_own_resets() {
        // Synthetic values in the HTTP response's actual window shape.
        let body = r#"{"additional_rate_limits":[{
          "limit_name":"GPT-5.3-Codex-Spark","metered_feature":"codex_bengalfox",
          "rate_limit":{
            "primary_window":{"used_percent":12,"limit_window_seconds":18000,"reset_at":1786350000},
            "secondary_window":{"used_percent":34,"limit_window_seconds":604800,"reset_at":1786937652}
          }}]}"#;
        let w = parse_usage("acct", body).unwrap();
        assert_eq!(w.windows.len(), 2);
        for (window, label, pct, reset) in [
            (&w.windows[0], "5h", 12, 1786350000),
            (&w.windows[1], "weekly", 34, 1786937652),
        ] {
            assert_eq!(window.scope.as_deref(), Some("GPT-5.3-Codex-Spark"));
            assert_eq!(window.label, label);
            assert_eq!(window.used_percent, pct);
            assert_eq!(window.reset_unix, Some(reset));
        }
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

    // Live shape from `codex app-server` (2026-08-10), values synthetic.
    // Note windowDurationMins is MINUTES (10080 = weekly) where the HTTP
    // path speaks seconds.
    const APP_SERVER_LINE: &str = r#"{"id":2,"result":{"rateLimits":{
      "limitId":"codex","limitName":null,
      "primary":{"usedPercent":41,"windowDurationMins":10080,"resetsAt":1786937652},
      "secondary":null,
      "credits":{"hasCredits":true,"unlimited":false,"balance":"12.5"},
      "planType":"prolite","spendControlReached":false},
      "rateLimitsByLimitId":{
        "codex_bengalfox":{"limitId":"codex_bengalfox","limitName":"GPT-5.3-Codex-Spark",
          "primary":{"usedPercent":4,"windowDurationMins":10080,"resetsAt":1786937652},
          "secondary":null,"planType":"prolite"},
        "codex":{"limitId":"codex","limitName":null,
          "primary":{"usedPercent":41,"windowDurationMins":10080,"resetsAt":1786937652},
          "secondary":{"usedPercent":9,"windowDurationMins":300,"resetsAt":1786350000},
          "planType":"prolite"}}}}"#;

    #[test]
    fn app_server_reset_credit_count_is_authoritative_including_zero() {
        for count in [0, 2, u32::MAX] {
            for details in [
                serde_json::Value::Null,
                serde_json::json!([]),
                serde_json::json!([{ "id": "synthetic-credit", "status": "available" }]),
            ] {
                let mut response: serde_json::Value =
                    serde_json::from_str(APP_SERVER_LINE).unwrap();
                response["result"]["rateLimitResetCredits"] = serde_json::json!({
                    "availableCount": count,
                    "credits": details,
                });
                let usage = parse_app_server_rate_limits(&response.to_string(), "acct").unwrap();
                assert_eq!(usage.reset_credits, Some(count));
                assert_eq!(usage.windows.len(), 3, "quota remains intact");
                assert_eq!(workspace_snapshot(&usage, 7).reset_credits, Some(count));
            }
        }
    }

    #[test]
    fn missing_or_malformed_reset_metadata_does_not_discard_valid_quota() {
        for reset in [
            None,
            Some(serde_json::Value::Null),
            Some(serde_json::json!({})),
            Some(serde_json::json!({"credits": [{"status": "available"}]})),
            Some(serde_json::json!({"availableCount": null})),
            Some(serde_json::json!({"availableCount": -1})),
            Some(serde_json::json!({"availableCount": 1.5})),
            Some(serde_json::json!({"availableCount": "2"})),
            Some(serde_json::json!({"availableCount": true})),
            Some(serde_json::json!({"availableCount": u64::from(u32::MAX) + 1})),
            Some(serde_json::json!("unexpected")),
            Some(serde_json::json!([])),
        ] {
            let mut response: serde_json::Value = serde_json::from_str(APP_SERVER_LINE).unwrap();
            if let Some(reset) = reset {
                response["result"]["rateLimitResetCredits"] = reset;
            }
            let usage = parse_app_server_rate_limits(&response.to_string(), "acct").unwrap();
            assert_eq!(usage.reset_credits, None);
            assert_eq!(usage.windows.len(), 3);
            assert_eq!(usage.credits_balance.as_deref(), Some("12.5"));
        }
    }

    #[test]
    fn reset_only_reading_preserves_zero_without_inventing_windows() {
        for count in [0, 2] {
            let response = serde_json::json!({"id": 2, "result": {
                "rateLimits": null, "rateLimitsByLimitId": null,
                "rateLimitResetCredits": {"availableCount": count}
            }});
            let usage = parse_app_server_rate_limits(&response.to_string(), "acct").unwrap();
            assert_eq!(usage.reset_credits, Some(count));
            assert!(usage.windows.is_empty());
        }
    }

    #[test]
    fn old_saved_workspaces_and_unverified_http_shape_keep_reset_count_unknown() {
        let old: WorkspaceUsage = serde_json::from_str(r#"{"account_id":"acct"}"#).unwrap();
        assert_eq!(old.reset_credits, None);
        assert!(
            !serde_json::to_value(old)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("reset_credits")
        );
        let mut http: serde_json::Value = serde_json::from_str(USAGE_BODY).unwrap();
        http["rateLimitResetCredits"] = serde_json::json!({"availableCount": 3});
        assert_eq!(
            parse_usage("acct", &http.to_string())
                .unwrap()
                .reset_credits,
            None
        );
    }

    #[test]
    fn app_server_line_parses_into_the_same_workspace_shape() {
        let w = parse_app_server_rate_limits(APP_SERVER_LINE, "acct-1").expect("rate limits parse");

        assert_eq!(w.account_id, "acct-1");
        assert_eq!(w.plan_type.as_deref(), Some("prolite"));
        assert_eq!(w.credits_balance.as_deref(), Some("12.5"));
        // Minutes-based durations map to the same labels as the HTTP path.
        assert!(
            w.windows
                .iter()
                .any(|x| x.label == "weekly" && x.used_percent == 41 && x.scope.is_none()),
            "main weekly pool survives unscoped: {:?}",
            w.windows
        );
        assert!(
            w.windows
                .iter()
                .any(|x| x.label == "5h" && x.used_percent == 9 && x.scope.is_none()),
            "secondary 5h window survives"
        );
        assert!(
            w.windows
                .iter()
                .any(|x| x.used_percent == 4 && x.scope.as_deref() == Some("GPT-5.3-Codex-Spark")),
            "per-model pool survives with its limitName as scope"
        );
        assert_eq!(w.windows.len(), 3, "the main pool is not duplicated");
        assert_eq!(
            w.windows
                .iter()
                .find(|x| x.scope.is_some())
                .unwrap()
                .reset_unix,
            Some(1786937652)
        );
    }

    #[test]
    fn app_server_uses_top_level_limits_when_the_map_is_absent_null_or_empty() {
        let mut response: serde_json::Value = serde_json::from_str(APP_SERVER_LINE).unwrap();
        for map in [
            None,
            Some(serde_json::Value::Null),
            Some(serde_json::json!({})),
        ] {
            let result = response["result"].as_object_mut().unwrap();
            result.remove("rateLimitsByLimitId");
            if let Some(map) = map {
                result.insert("rateLimitsByLimitId".into(), map);
            }
            let w = parse_app_server_rate_limits(&response.to_string(), "acct").unwrap();
            assert_eq!(w.windows.len(), 1);
            assert_eq!(w.windows[0].scope, None);
            assert_eq!(w.windows[0].label, "weekly");
            assert_eq!(w.windows[0].used_percent, 41);
            assert_eq!(w.windows[0].reset_unix, Some(1786937652));
        }
    }

    #[test]
    fn app_server_keeps_top_level_main_when_the_map_has_only_extra_pools() {
        let mut response: serde_json::Value = serde_json::from_str(APP_SERVER_LINE).unwrap();
        response["result"]["rateLimitsByLimitId"]
            .as_object_mut()
            .unwrap()
            .remove("codex");
        let w = parse_app_server_rate_limits(&response.to_string(), "acct").unwrap();
        assert_eq!(w.windows.len(), 2);
        let main = w.windows.iter().find(|x| x.scope.is_none()).unwrap();
        assert_eq!(main.used_percent, 41);
        assert_eq!(main.reset_unix, Some(1786937652));
        assert!(
            w.windows.iter().any(|x| {
                x.scope.as_deref() == Some("GPT-5.3-Codex-Spark") && x.used_percent == 4
            })
        );
    }

    #[test]
    fn app_server_reads_main_and_metadata_from_the_map_when_top_is_null() {
        let mut response: serde_json::Value = serde_json::from_str(APP_SERVER_LINE).unwrap();
        response["result"]["rateLimits"] = serde_json::Value::Null;
        response["result"]["rateLimitsByLimitId"]["codex"]["credits"] =
            serde_json::json!({"balance": "12.5"});
        let w = parse_app_server_rate_limits(&response.to_string(), "acct").unwrap();
        assert_eq!(w.plan_type.as_deref(), Some("prolite"));
        assert_eq!(w.credits_balance.as_deref(), Some("12.5"));
        assert_eq!(w.windows.len(), 3);
        assert_eq!(w.windows.iter().filter(|x| x.scope.is_none()).count(), 2);
    }

    #[test]
    fn app_server_preserves_spark_five_hour_and_weekly_resets_without_a_main_pool() {
        // Synthetic usage, not a claim that every plan exposes this pool.
        let line = r#"{"id":2,"result":{"rateLimits":null,"rateLimitsByLimitId":{
          "codex_bengalfox":{"limitName":"GPT-5.3-Codex-Spark",
            "primary":{"usedPercent":12,"windowDurationMins":300,"resetsAt":1786350000},
            "secondary":{"usedPercent":34,"windowDurationMins":10080,"resetsAt":1786937652}
          }}}}"#;
        let w = parse_app_server_rate_limits(line, "acct").unwrap();
        assert_eq!(w.windows.len(), 2);
        for (window, label, pct, reset) in [
            (&w.windows[0], "5h", 12, 1786350000),
            (&w.windows[1], "weekly", 34, 1786937652),
        ] {
            assert_eq!(window.scope.as_deref(), Some("GPT-5.3-Codex-Spark"));
            assert_eq!(window.label, label);
            assert_eq!(window.used_percent, pct);
            assert_eq!(window.reset_unix, Some(reset));
        }
        let snapshot = workspace_snapshot(&w, 7);
        assert!(snapshot.five_hour.is_none());
        assert!(snapshot.seven_day.is_none());
        assert_eq!(snapshot.scoped.len(), 2);
        assert_eq!(snapshot.scoped[0].reset_unix, Some(1786350000));
        assert_eq!(snapshot.scoped[1].reset_unix, Some(1786937652));
    }

    #[test]
    fn app_server_without_either_limit_source_is_not_a_zero_usage_reading() {
        for line in [
            r#"{"id":2,"result":{"rateLimits":null,"rateLimitsByLimitId":null}}"#,
            r#"{"id":2,"result":{"rateLimits":null,"rateLimitsByLimitId":{}}}"#,
            r#"{"id":2,"result":{}}"#,
        ] {
            assert!(parse_app_server_rate_limits(line, "acct").is_none());
        }
    }

    #[test]
    fn workspace_snapshot_maps_windows_onto_the_provider_row_shape() {
        let ws = WorkspaceUsage {
            account_id: "acct-1".into(),
            windows: vec![
                UsageWindow {
                    label: "5h".into(),
                    used_percent: 9,
                    reset_unix: Some(1_786_350_000),
                    scope: None,
                },
                UsageWindow {
                    label: "weekly".into(),
                    used_percent: 41,
                    reset_unix: Some(1_786_937_652),
                    scope: None,
                },
                UsageWindow {
                    label: "weekly".into(),
                    used_percent: 4,
                    reset_unix: Some(1_786_937_652),
                    scope: Some("GPT-5.3-Codex-Spark".into()),
                },
            ],
            ..Default::default()
        };

        let cu = workspace_snapshot(&ws, 7);

        assert_eq!(cu.fetched_at_ms, 7);
        assert_eq!(cu.account_id.as_deref(), Some("acct-1"));
        assert_eq!(cu.five_hour.as_ref().map(|w| w.pct), Some(9));
        assert_eq!(cu.seven_day.as_ref().map(|w| w.pct), Some(41));
        assert_eq!(cu.scoped.len(), 1, "scoped pools survive as chips");
        assert_eq!(cu.scoped[0].kind, "pool_weekly");
        assert_eq!(cu.scoped[0].scope.as_deref(), Some("GPT-5.3-Codex-Spark"));
        assert_eq!(cu.scoped[0].pct, 4);
    }

    #[test]
    fn app_server_junk_and_wrong_ids_do_not_parse() {
        assert!(parse_app_server_rate_limits("not json", "a").is_none());
        assert!(parse_app_server_rate_limits(r#"{"id":1,"result":{}}"#, "a").is_none());
        assert!(
            parse_app_server_rate_limits(r#"{"id":2,"error":{"code":-1}}"#, "a").is_none(),
            "an RPC error is not usage data"
        );
    }
}
