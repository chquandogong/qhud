//! Antigravity quota on explicit refresh via its own loopback Connect RPC.
//!
//! A running `agy` binds loopback listeners; the plaintext one answers
//! `POST /exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary`
//! with **no token and no CSRF** for the CLI surface (verified live
//! 2026-08-07 and 2026-08-10). Nothing leaves the machine and no
//! credential is read — the agy process owns its own auth.
//!
//! Without this, agy numbers exist only while an agy pane happens to be
//! running (statusline sidefile). With it, ⟳ works whenever agy runs
//! anywhere; when agy is closed the last stored reading renders, dated.
//!
//! Linux discovers agy's listening loopback sockets through `/proc`.
//! Windows also supports the native Antigravity app. Its daemon registration
//! supplies a process id, HTTP port and local CSRF token. The process executable
//! and listening-port ownership are verified before that token is sent, only
//! to 127.0.0.1. Google credentials remain with Antigravity. Native readings
//! use the identity returned by the same process, never Gemini CLI's login.

use crate::usage_cache::{CachedUsage, CachedWindow, ScopedLimit, parse_reset};

const RPC_PATH: &str = "/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary";
const STATUS_PATH: &str = "/exa.language_server_pb.LanguageServerService/GetUserStatus";

// Deliberately not Debug/Serialize: the CSRF value is a local session secret.
#[cfg(any(windows, test))]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopRegistration {
    pid: u32,
    http_port: u16,
    csrf_token: String,
}

#[cfg(any(windows, test))]
fn parse_registration(body: &str) -> Option<DesktopRegistration> {
    let registration: DesktopRegistration = serde_json::from_str(body).ok()?;
    (registration.pid > 0
        && registration.http_port > 0
        && !registration.csrf_token.is_empty()
        && registration.csrf_token.len() <= 512
        && registration
            .csrf_token
            .bytes()
            .all(|b| b.is_ascii_graphic()))
    .then_some(registration)
}

#[cfg(windows)]
fn desktop_registrations() -> Vec<DesktopRegistration> {
    let Some(root) = crate::paths::home_dir().map(|home| home.join(".gemini/antigravity/daemon"))
    else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("hub_") && name.ends_with(".json")
        })
        .take(16)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() || metadata.len() > 16_384 {
                return None;
            }
            parse_registration(&std::fs::read_to_string(entry.path()).ok()?)
        })
        .collect()
}

#[cfg(any(windows, test))]
fn desktop_executable(path: &std::path::Path) -> bool {
    let path = path
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    path.ends_with("\\antigravity\\resources\\bin\\language_server.exe")
}

#[cfg(windows)]
fn windows_runtimes(registrations: &[DesktopRegistration]) -> (bool, bool) {
    let mut system = sysinfo::System::new();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );
    let desktop = registrations.iter().any(|registration| {
        system
            .process(sysinfo::Pid::from_u32(registration.pid))
            .and_then(|process| process.exe())
            .is_some_and(desktop_executable)
    });
    let cli = system.processes().values().any(|process| {
        process
            .exe()
            .and_then(|path| path.file_name())
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("agy.exe"))
    });
    (desktop, cli)
}

/// Passive local prerequisite check. It never calls the language server, and
/// a running process does not establish a valid Google login or fresh quota.
pub fn local_health(now_ms: u64) -> crate::provider_health::ProviderHealth {
    use crate::provider_health::{HealthState, ProviderHealth};
    #[cfg(windows)]
    let (desktop_running, cli_running) = {
        // Process metadata is independent of the two-second usage poll.
        static CACHED: std::sync::Mutex<Option<(std::time::Instant, (bool, bool))>> =
            std::sync::Mutex::new(None);
        let mut cached = CACHED.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((when, running)) = *cached
            && when.elapsed() < std::time::Duration::from_secs(15)
        {
            running
        } else {
            let running = windows_runtimes(&desktop_registrations());
            *cached = Some((std::time::Instant::now(), running));
            running
        }
    };
    #[cfg(target_os = "linux")]
    let (desktop_running, cli_running) = (false, !agy_pids().is_empty());
    #[cfg(not(any(windows, target_os = "linux")))]
    let (desktop_running, cli_running) = (false, false);
    let running = desktop_running || cli_running;
    let mut health = ProviderHealth::new(
        "agy",
        if desktop_running {
            "antigravity-desktop"
        } else {
            "agy-cli"
        },
        now_ms,
    );
    health.state = if running {
        HealthState::Ready
    } else {
        HealthState::NotRunning
    };
    health.code = if running {
        "runtime_found"
    } else {
        "runtime_not_running"
    };
    health.message = if running {
        "Antigravity is running; refresh to verify sign-in and read quota."
    } else {
        "No running Antigravity quota service was found."
    };
    health.action = Some("open_antigravity");
    health.cli_available = cli_running;
    health
}

#[derive(PartialEq, Eq)]
struct NativeStatus {
    email: String,
    plan_name: Option<String>,
}

/// Identity and current entitlement selected from the process's own status
/// response. Neither another Google app's login nor an upgrade offer describes
/// the account and plan that own this quota snapshot.
fn status_identity(body: &str) -> Option<NativeStatus> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let status = value.get("userStatus")?;
    let email = status.get("email")?.as_str()?.trim();
    let plan_name = status
        .get("userTier")
        .and_then(|tier| tier.get("name"))
        .and_then(serde_json::Value::as_str)
        .filter(|name| !name.chars().any(char::is_control))
        .map(str::trim)
        .filter(|name| !name.is_empty() && name.chars().count() <= 80)
        .map(str::to_string);
    (!email.is_empty() && email.len() <= 320 && !email.chars().any(char::is_control)).then(|| {
        NativeStatus {
            email: email.to_string(),
            plan_name,
        }
    })
}

fn bind_identity(
    mut usage: CachedUsage,
    before: &NativeStatus,
    after: &NativeStatus,
) -> Result<CachedUsage, String> {
    if before != after {
        return Err("Antigravity account or plan changed during refresh; retry".into());
    }
    usage.account_id = Some(before.email.clone());
    usage.plan_name = before.plan_name.clone();
    Ok(usage)
}

/// Maps the RPC's four (or more) buckets onto the snapshot shape the rest
/// of qhud already renders:
///  - `gemini-5h`  → `five_hour`, `gemini-weekly` → `seven_day`
///    (the primary pool takes the primary gauges),
///  - every other bucket (`3p-5h`, `3p-weekly`, future pools) →
///    `scoped` as `pool_5h` / `pool_weekly` with the pool prefix as scope,
///    so an unknown pool shows up instead of vanishing.
///
/// A bucket with **no** `remainingFraction` is fully used: Connect's
/// proto3-JSON marshalling omits fields at their default value, and the
/// default for a fraction is 0 remaining. (Verified: an untouched bucket
/// arrives as an explicit `"remainingFraction": 1`.)
pub fn parse_quota_summary(body: &str, fetched_at_ms: u64) -> Option<CachedUsage> {
    #[derive(serde::Deserialize)]
    struct Resp {
        response: Inner, // absent ⇒ not this RPC's answer ⇒ no parse
    }
    #[derive(serde::Deserialize)]
    struct Inner {
        #[serde(default)]
        groups: Vec<Group>,
    }
    #[derive(serde::Deserialize)]
    struct Group {
        #[serde(default)]
        buckets: Vec<Bucket>,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Bucket {
        #[serde(default)]
        bucket_id: Option<String>,
        #[serde(default)]
        window: Option<String>,
        #[serde(default)]
        remaining_fraction: Option<f64>,
        #[serde(default)]
        reset_time: Option<String>,
    }

    let resp: Resp = serde_json::from_str(body).ok()?;
    let mut usage = CachedUsage {
        org_id: None,
        plan_name: None,
        reset_credits: None,
        fetched_at_ms,
        account_id: None,
        five_hour: None,
        seven_day: None,
        scoped: Vec::new(),
        extra: None,
    };
    let mut any = false;
    for bucket in resp.response.groups.into_iter().flat_map(|g| g.buckets) {
        let Some(id) = bucket.bucket_id else { continue };
        // proto3-JSON omits default values: an absent fraction is 0
        // remaining (fully used), never "unknown, call it fine".
        let remaining = bucket.remaining_fraction.unwrap_or(0.0);
        if !remaining.is_finite() || !(0.0..=1.0).contains(&remaining) {
            continue;
        }
        let used = ((1.0 - remaining) * 100.0).round().clamp(0.0, 100.0) as u8;
        let reset_unix = bucket.reset_time.as_deref().and_then(parse_reset);
        any = true;
        match id.as_str() {
            "gemini-5h" => {
                usage.five_hour = Some(CachedWindow {
                    pct: used,
                    reset_unix,
                })
            }
            "gemini-weekly" => {
                usage.seven_day = Some(CachedWindow {
                    pct: used,
                    reset_unix,
                })
            }
            other => {
                // "3p-5h" → scope "3p", kind pool_5h; unknown pools and
                // windows still land here instead of vanishing.
                let kind = match bucket.window.as_deref() {
                    Some("5h") => "pool_5h",
                    Some("weekly") => "pool_weekly",
                    _ => "pool_unknown",
                };
                let scope = if kind == "pool_unknown" {
                    other
                } else {
                    other
                        .rsplit_once('-')
                        .map(|(pool, _)| pool)
                        .unwrap_or(other)
                }
                .to_string();
                usage.scoped.push(ScopedLimit {
                    kind: kind.to_string(),
                    scope: Some(scope),
                    pct: used,
                    reset_unix,
                });
            }
        }
    }
    any.then_some(usage)
}

/// LISTEN rows of `/proc/net/tcp` bound to 127.0.0.1 whose socket inode
/// is in `inodes`, as ports. Pure so the hex parsing is testable.
#[cfg(any(target_os = "linux", test))]
pub fn listen_ports_from(tcp: &str, inodes: &std::collections::HashSet<u64>) -> Vec<u16> {
    tcp.lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let local = cols.nth(1)?; // "0100007F:9B2B"
            let state = cols.nth(1)?; // st column ("0A" = LISTEN)
            let inode: u64 = cols.nth(5)?.parse().ok()?;
            if state != "0A" || !inodes.contains(&inode) {
                return None;
            }
            let (addr, port) = local.split_once(':')?;
            if addr != "0100007F" {
                return None;
            }
            u16::from_str_radix(port, 16).ok()
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn agy_pids() -> Vec<u32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| {
            let pid: u32 = e.file_name().to_str()?.parse().ok()?;
            let comm = std::fs::read_to_string(e.path().join("comm")).ok()?;
            (comm.trim() == "agy").then_some(pid)
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn socket_inodes(pid: u32) -> std::collections::HashSet<u64> {
    let mut out = std::collections::HashSet::new();
    let Ok(fds) = std::fs::read_dir(format!("/proc/{pid}/fd")) else {
        return out;
    };
    for fd in fds.flatten() {
        if let Ok(target) = std::fs::read_link(fd.path())
            && let Some(rest) = target.to_str().and_then(|s| s.strip_prefix("socket:["))
            && let Some(inode) = rest.strip_suffix(']').and_then(|n| n.parse().ok())
        {
            out.insert(inode);
        }
    }
    out
}

/// Loopback ports a running agy is listening on, in ascending order.
/// Empty when agy is not running (the caller words the error).
#[cfg(target_os = "linux")]
pub async fn discover_ports() -> Result<Vec<u16>, String> {
    let mut ports: Vec<u16> = Vec::new();
    let Ok(tcp) = std::fs::read_to_string("/proc/net/tcp") else {
        return Ok(ports);
    };
    for pid in agy_pids() {
        ports.extend(listen_ports_from(&tcp, &socket_inodes(pid)));
    }
    ports.sort_unstable();
    ports.dedup();
    Ok(ports)
}

// Not Debug/Serialize: this holds the app-local CSRF secret in memory only.
struct Endpoint {
    port: u16,
    csrf_token: Option<String>,
    desktop: bool,
}

#[cfg(any(windows, test))]
fn verified_desktop_endpoints(
    registrations: Vec<DesktopRegistration>,
    owned: &std::collections::HashSet<(u32, u16)>,
) -> Vec<Endpoint> {
    registrations
        .into_iter()
        .filter(|registration| owned.contains(&(registration.pid, registration.http_port)))
        .map(|registration| Endpoint {
            port: registration.http_port,
            csrf_token: Some(registration.csrf_token),
            desktop: true,
        })
        .collect()
}

#[cfg(windows)]
async fn desktop_endpoints() -> Result<Vec<Endpoint>, String> {
    let registrations = desktop_registrations();
    if registrations.is_empty() {
        return Ok(Vec::new());
    }
    // Only PID:port pairs leave PowerShell. The registration's CSRF token
    // never enters shell arguments, stdout, diagnostics or persisted state.
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$ids = @(Get-CimInstance Win32_Process -Filter "Name = 'language_server.exe'" |
    Where-Object { $_.ExecutablePath -and $_.ExecutablePath.ToLowerInvariant().EndsWith('\antigravity\resources\bin\language_server.exe') } |
    Select-Object -ExpandProperty ProcessId)
if ($ids.Count -eq 0) { exit 0 }
Get-NetTCPConnection -State Listen -ErrorAction Stop |
    Where-Object { $_.LocalAddress -eq '127.0.0.1' -and $ids -contains $_.OwningProcess } |
    ForEach-Object { '{0}:{1}' -f $_.OwningProcess,$_.LocalPort }
"#;
    let powershell = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .map(|root| root.join("System32/WindowsPowerShell/v1.0/powershell.exe"))
        .unwrap_or_else(|| "powershell.exe".into());
    let mut command = tokio::process::Command::new(powershell);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            SCRIPT,
        ])
        .creation_flags(0x0800_0000)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let output = tokio::time::timeout(std::time::Duration::from_secs(10), command.output())
        .await
        .map_err(|_| "Antigravity local process discovery timed out".to_string())?
        .map_err(|_| "Antigravity local process discovery could not start".to_string())?;
    if !output.status.success() {
        return Err("Antigravity local listener ownership could not be verified".into());
    }
    let output = String::from_utf8(output.stdout)
        .map_err(|_| "Antigravity local process discovery returned invalid data".to_string())?;
    let owned: std::collections::HashSet<(u32, u16)> = output
        .lines()
        .filter_map(|line| {
            let (pid, port) = line.trim().split_once(':')?;
            Some((pid.parse().ok()?, port.parse().ok()?))
        })
        .collect();
    Ok(verified_desktop_endpoints(registrations, &owned))
}

async fn rpc(client: &reqwest::Client, endpoint: &Endpoint, path: &str) -> Result<String, String> {
    let mut request = client
        .post(format!("http://127.0.0.1:{}{path}", endpoint.port))
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1")
        .body("{}");
    if let Some(token) = &endpoint.csrf_token {
        request = request.header("x-codeium-csrf-token", token);
    }
    let mut response = request.send().await.map_err(|e| {
        format!(
            "Antigravity local request failed ({})",
            crate::diag::reqwest(&e)
        )
    })?;
    if !response.status().is_success() {
        return Err(format!(
            "Antigravity local service returned HTTP {}",
            response.status().as_u16()
        ));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Antigravity local response could not be read".to_string())?
    {
        if body.len() + chunk.len() > 1_048_576 {
            return Err("Antigravity local response exceeded the size limit".into());
        }
        body.extend_from_slice(&chunk);
    }
    String::from_utf8(body).map_err(|_| "Antigravity local response was not UTF-8".into())
}

#[cfg(any(windows, test))]
fn windows_ports_from(output: &str) -> Result<Vec<u16>, String> {
    let mut ports = Vec::new();
    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let port = line
            .parse::<u16>()
            .ok()
            .filter(|port| *port != 0)
            .ok_or_else(|| "Windows returned an invalid agy listening port".to_string())?;
        ports.push(port);
    }
    ports.sort_unstable();
    ports.dedup();
    Ok(ports)
}

#[cfg(windows)]
pub async fn discover_ports() -> Result<Vec<u16>, String> {
    // Only numeric ports leave this subprocess. It neither reads process
    // command lines nor credentials, and never probes another process's port.
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$agyPids = @(Get-Process -Name agy -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
if ($agyPids.Count -eq 0) { exit 0 }
Get-NetTCPConnection -State Listen -ErrorAction Stop |
    Where-Object { $_.LocalAddress -eq '127.0.0.1' -and $agyPids -contains $_.OwningProcess } |
    Select-Object -ExpandProperty LocalPort -Unique
"#;
    let powershell = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .map(|root| root.join("System32/WindowsPowerShell/v1.0/powershell.exe"))
        .unwrap_or_else(|| "powershell.exe".into());
    let mut command = tokio::process::Command::new(powershell);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            SCRIPT,
        ])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let output = tokio::time::timeout(std::time::Duration::from_secs(10), command.output())
        .await
        .map_err(|_| "Windows agy port discovery timed out".to_string())?
        .map_err(|e| {
            crate::diag::failure(
                "Windows agy port discovery PowerShell start",
                crate::diag::spawn(&e),
            )
        })?;
    if !output.status.success() {
        return Err("Windows could not read agy listening ports with Get-NetTCPConnection".into());
    }
    let ports = String::from_utf8(output.stdout)
        .map_err(|_| "Windows agy port discovery returned invalid text".to_string())?;
    windows_ports_from(&ports)
}

#[cfg(not(any(target_os = "linux", windows)))]
pub async fn discover_ports() -> Result<Vec<u16>, String> {
    Err("agy port discovery is supported on Windows and Linux only".into())
}

/// One RPC round per candidate port until one parses. The HTTPS listener
/// answers plain HTTP with an error body, which simply fails the parse.
pub async fn fetch(now_ms: u64) -> Result<CachedUsage, String> {
    // One overall deadline bounds process discovery plus every candidate RPC.
    // Dropping the future also kills an unfinished discovery subprocess.
    tokio::time::timeout(std::time::Duration::from_secs(30), fetch_inner(now_ms))
        .await
        .map_err(|_| "Antigravity usage refresh timed out".to_string())?
}

async fn fetch_inner(now_ms: u64) -> Result<CachedUsage, String> {
    // Bind the reading to the identity present when refresh began so a
    // subsequent sign-in cannot relabel a persisted quota snapshot.
    let cli_account_id = crate::accounts::detect_all()
        .into_iter()
        .find(|(provider, _)| provider == "agy")
        .and_then(|(_, account)| account.account_id.or(account.email));
    #[cfg(windows)]
    let mut endpoints = desktop_endpoints().await?;
    #[cfg(not(windows))]
    let mut endpoints = Vec::new();
    // Prefer the native app when it is open. A failure there must not switch
    // silently to a different CLI account.
    if endpoints.is_empty() {
        endpoints.extend(discover_ports().await?.into_iter().map(|port| Endpoint {
            port,
            csrf_token: None,
            desktop: false,
        }));
    }
    if endpoints.is_empty() {
        return Err(
            "no verified Antigravity quota listener — open Antigravity or agy and retry".into(),
        );
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| crate::diag::failure("HTTP client build", crate::diag::reqwest(&e)))?;
    let mut last = String::new();
    for endpoint in endpoints {
        let before = if endpoint.desktop {
            match rpc(&client, &endpoint, STATUS_PATH).await {
                Ok(body) => match status_identity(&body) {
                    Some(identity) => Some(identity),
                    None => {
                        last = "Antigravity did not report a signed-in account; sign in in the app"
                            .into();
                        continue;
                    }
                },
                Err(error) => {
                    last = error;
                    continue;
                }
            }
        } else {
            None
        };
        match rpc(&client, &endpoint, RPC_PATH).await {
            Ok(body) => {
                if let Some(mut usage) = parse_quota_summary(&body, now_ms) {
                    if let Some(before) = before {
                        let after_body = rpc(&client, &endpoint, STATUS_PATH).await?;
                        let after = status_identity(&after_body).ok_or_else(|| {
                            "Antigravity sign-in changed during refresh; retry".to_string()
                        })?;
                        return bind_identity(usage, &before, &after);
                    }
                    usage.account_id = cli_account_id;
                    return Ok(usage);
                }
                last = "Antigravity returned no supported quota buckets".into();
            }
            Err(error) => last = error,
        }
    }
    Err(if last.is_empty() {
        "no agy port answered".into()
    } else {
        last
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_registration_requires_a_bounded_local_session() {
        let registration = parse_registration(
            r#"{"pid":42,"httpPort":3000,"csrfToken":"synthetic-local-session"}"#,
        )
        .unwrap();
        assert_eq!(registration.pid, 42);
        assert_eq!(registration.http_port, 3000);
        assert!(parse_registration(r#"{"pid":0,"httpPort":3000,"csrfToken":"x"}"#).is_none());
        assert!(parse_registration(r#"{"pid":42,"httpPort":0,"csrfToken":"x"}"#).is_none());
        assert!(parse_registration(r#"{"pid":42,"httpPort":3000,"csrfToken":""}"#).is_none());
        assert!(parse_registration(r#"{"pid":42,"httpPort":3000,"csrfToken":"x\r\ny"}"#).is_none());
        assert!(parse_registration("{}").is_none());
    }

    #[test]
    fn desktop_discovery_does_not_trust_a_generic_language_server() {
        assert!(desktop_executable(std::path::Path::new(
            "C:/Programs/Antigravity/resources/bin/language_server.exe"
        )));
        assert!(!desktop_executable(std::path::Path::new(
            "C:/Programs/Other/resources/bin/language_server.exe"
        )));
        assert!(!desktop_executable(std::path::Path::new(
            "C:/Programs/Antigravity/resources/bin/language_server.exe.other"
        )));
    }

    #[test]
    fn local_csrf_is_used_only_for_the_registered_process_and_port() {
        let registration = || {
            parse_registration(
                r#"{"pid":42,"httpPort":3000,"csrfToken":"synthetic-local-session"}"#,
            )
            .unwrap()
        };
        assert!(
            verified_desktop_endpoints(vec![registration()], &[(43, 3000), (42, 3001)].into())
                .is_empty()
        );
        let endpoints = verified_desktop_endpoints(vec![registration()], &[(42, 3000)].into());
        assert_eq!(endpoints.len(), 1);
        assert!(endpoints[0].desktop);
        assert_eq!(endpoints[0].port, 3000);
        assert!(endpoints[0].csrf_token.is_some());
    }

    #[test]
    fn native_quota_keeps_the_runtime_identity_and_rejects_a_login_switch() {
        let before = status_identity(r#"{"userStatus":{"email":"app@example.test"}}"#).unwrap();
        let usage = parse_quota_summary(BODY, 7).unwrap();
        assert_eq!(
            bind_identity(usage.clone(), &before, &before)
                .unwrap()
                .account_id
                .as_deref(),
            Some("app@example.test")
        );
        let other =
            status_identity(r#"{"userStatus":{"email":"different@example.test"}}"#).unwrap();
        assert!(bind_identity(usage, &before, &other).is_err());
        assert!(status_identity(r#"{"userStatus":{"email":""}}"#).is_none());
        assert!(status_identity(r#"{"userStatus":{}}"#).is_none());
        assert!(status_identity(r#"{"message":"not signed in"}"#).is_none());
    }

    #[test]
    fn native_plan_uses_current_tier_name_not_the_upgrade_offer() {
        let status = status_identity(
            r#"{"userStatus":{"email":"app@example.test",
                "userTier":{"name":"Google AI Pro",
                    "upgradeSubscriptionText":"Upgrade to Google AI Ultra"}}}"#,
        )
        .unwrap();
        let usage = parse_quota_summary(BODY, 7).unwrap();
        assert!(usage.plan_name.is_none());
        let bound = bind_identity(usage, &status, &status).unwrap();
        assert_eq!(bound.plan_name.as_deref(), Some("Google AI Pro"));
        assert_eq!(bound.account_id.as_deref(), Some("app@example.test"));
    }

    #[test]
    fn native_plan_is_optional_bounded_and_control_free() {
        for name in [
            serde_json::Value::Null,
            serde_json::json!(true),
            serde_json::json!(12),
            serde_json::json!(""),
            serde_json::json!("   "),
            serde_json::json!("Google AI\nPro"),
            serde_json::json!("\tGoogle AI Pro"),
            serde_json::json!("x".repeat(81)),
        ] {
            let body = serde_json::json!({"userStatus": {
                "email": "app@example.test", "userTier": {"name": name}
            }});
            let status = status_identity(&body.to_string()).unwrap();
            assert!(status.plan_name.is_none());
        }
        for body in [
            r#"{"userStatus":{"email":"app@example.test"}}"#,
            r#"{"userStatus":{"email":"app@example.test","userTier":{"upgradeSubscriptionText":"Google AI Ultra"}}}"#,
        ] {
            let status = status_identity(body).unwrap();
            let bound =
                bind_identity(parse_quota_summary(BODY, 7).unwrap(), &status, &status).unwrap();
            assert!(bound.plan_name.is_none());
        }
        for name in [" Google AI Pro ".to_string(), "x".repeat(80)] {
            let body = serde_json::json!({"userStatus": {
                "email": "app@example.test", "userTier": {"name": name}
            }});
            let status = status_identity(&body.to_string()).unwrap();
            assert_eq!(status.plan_name.as_deref(), Some(name.trim()));
        }
    }

    #[test]
    fn native_quota_rejects_a_plan_change_during_refresh() {
        let before = status_identity(
            r#"{"userStatus":{"email":"app@example.test","userTier":{"name":"Google AI Pro"}}}"#,
        )
        .unwrap();
        let upgraded = status_identity(
            r#"{"userStatus":{"email":"app@example.test","userTier":{"name":"Google AI Ultra"}}}"#,
        )
        .unwrap();
        let missing = status_identity(r#"{"userStatus":{"email":"app@example.test"}}"#).unwrap();
        let usage = parse_quota_summary(BODY, 7).unwrap();
        assert!(bind_identity(usage.clone(), &before, &upgraded).is_err());
        assert!(bind_identity(usage.clone(), &before, &missing).is_err());
        assert!(bind_identity(usage, &missing, &before).is_err());
    }

    #[test]
    fn unknown_pool_duration_stays_unknown_and_invalid_fraction_is_not_zero() {
        let body = r#"{"response":{"groups":[{"buckets":[
            {"bucketId":"future-monthly","window":"monthly","remainingFraction":0.5},
            {"bucketId":"invalid-5h","window":"5h","remainingFraction":1.1},
            {"bucketId":"negative-weekly","window":"weekly","remainingFraction":-0.1}
        ]}]}}"#;
        let usage = parse_quota_summary(body, 7).unwrap();
        assert_eq!(usage.scoped.len(), 1);
        assert_eq!(usage.scoped[0].kind, "pool_unknown");
        assert_eq!(usage.scoped[0].scope.as_deref(), Some("future-monthly"));
        assert_eq!(usage.scoped[0].pct, 50);
    }

    #[test]
    fn windows_ports_accept_only_valid_ports_and_deduplicate() {
        assert_eq!(
            windows_ports_from("41912\r\n 3000 \r\n41912\r\n").unwrap(),
            vec![3000, 41912]
        );
        assert!(windows_ports_from("\r\n").unwrap().is_empty());
        assert!(windows_ports_from("0").is_err());
        assert!(windows_ports_from("65536").is_err());
        assert!(windows_ports_from("Access is denied").is_err());
    }

    // Live shape (2026-08-10), values synthetic. `3p-5h` has NO
    // remainingFraction on purpose: proto3-JSON omits default values,
    // and the default is 0 remaining.
    const BODY: &str = r#"{"response":{"groups":[
      {"displayName":"Gemini Models","buckets":[
        {"bucketId":"gemini-weekly","displayName":"Weekly Limit Remaining",
         "window":"weekly","remainingFraction":0.75,"resetTime":"2026-08-17T02:23:27Z"},
        {"bucketId":"gemini-5h","window":"5h","remainingFraction":0.9,
         "resetTime":"2026-08-10T07:23:27Z"}]},
      {"displayName":"Claude and GPT models","buckets":[
        {"bucketId":"3p-weekly","window":"weekly","remainingFraction":1,
         "resetTime":"2026-08-17T02:23:27Z"},
        {"bucketId":"3p-5h","window":"5h","resetTime":"2026-08-10T07:23:27Z"}]}]}}"#;

    #[test]
    fn gemini_pool_takes_the_primary_gauges() {
        let u = parse_quota_summary(BODY, 7).expect("summary parses");

        assert_eq!(u.fetched_at_ms, 7);
        assert_eq!(
            u.five_hour,
            Some(CachedWindow {
                pct: 10,
                reset_unix: parse_reset("2026-08-10T07:23:27Z"),
            })
        );
        assert_eq!(u.seven_day.as_ref().map(|w| w.pct), Some(25));
    }

    #[test]
    fn other_pools_become_scoped_windows_not_lost_data() {
        let u = parse_quota_summary(BODY, 7).unwrap();

        let pool_weekly = u
            .scoped
            .iter()
            .find(|s| s.kind == "pool_weekly")
            .expect("3p weekly survives");
        assert_eq!(pool_weekly.scope.as_deref(), Some("3p"));
        assert_eq!(pool_weekly.pct, 0);

        // Absent remainingFraction is DEFAULT-OMITTED zero remaining —
        // i.e. fully used — never "unknown, call it fine".
        let pool_5h = u
            .scoped
            .iter()
            .find(|s| s.kind == "pool_5h")
            .expect("3p 5h survives");
        assert_eq!(pool_5h.pct, 100);
    }

    #[test]
    fn junk_and_error_bodies_do_not_parse() {
        assert!(
            parse_quota_summary("Client sent an HTTP request to an HTTPS server.", 1).is_none()
        );
        assert!(parse_quota_summary("not json", 1).is_none());
        assert!(parse_quota_summary("{}", 1).is_none());
    }

    #[test]
    fn listen_ports_join_inodes_against_loopback_listen_rows() {
        // Two LISTEN rows on 127.0.0.1 (0100007F) with inodes 111/222, one
        // ESTABLISHED (st 01) and one LISTEN on 0.0.0.0 — only the
        // loopback LISTEN rows whose inode we own may come back.
        let tcp = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:9B2B 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 111 1 0000000000000000 100 0 0 10 0
   1: 0100007F:9743 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 222 1 0000000000000000 100 0 0 10 0
   2: 0100007F:1F90 0100007F:9B2B 01 00000000:00000000 00:00000000 00000000  1000        0 333 1 0000000000000000 100 0 0 10 0
   3: 00000000:0050 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 444 1 0000000000000000 100 0 0 10 0
";
        let inodes: std::collections::HashSet<u64> = [111, 222, 333, 444].into();

        let mut ports = listen_ports_from(tcp, &inodes);
        ports.sort_unstable();

        assert_eq!(ports, vec![0x9743, 0x9B2B]);

        // And an inode we do not own is someone else's socket.
        let none: std::collections::HashSet<u64> = [999].into();
        assert!(listen_ports_from(tcp, &none).is_empty());
    }
}
