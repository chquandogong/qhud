//! agy quota on explicit refresh, via the CLI's own loopback Connect RPC.
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
//! Windows asks the local TCP table for 127.0.0.1 listeners owned by an
//! `agy.exe` process. The HTTPS listener rejects plain HTTP, so each
//! candidate port is simply tried in order.

use crate::usage_cache::{CachedUsage, CachedWindow, ScopedLimit, parse_reset};

const RPC_PATH: &str = "/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary";

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
        let used = ((1.0 - bucket.remaining_fraction.unwrap_or(0.0)) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8;
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
                let scope = other
                    .rsplit_once('-')
                    .map(|(pool, _)| pool)
                    .unwrap_or(other)
                    .to_string();
                let kind = match bucket.window.as_deref() {
                    Some("5h") => "pool_5h",
                    _ => "pool_weekly",
                };
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
        .map_err(|e| format!("Windows agy port discovery could not start PowerShell: {e}"))?;
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
    // Bind the reading to the identity present when refresh began so a
    // subsequent sign-in cannot relabel a persisted quota snapshot.
    let account_id = crate::accounts::detect_all()
        .into_iter()
        .find(|(provider, _)| provider == "agy")
        .and_then(|(_, account)| account.account_id.or(account.email));
    let ports = discover_ports().await?;
    if ports.is_empty() {
        return Err(
            "no running agy process has a 127.0.0.1 quota listener — start agy and retry".into(),
        );
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("client build failed: {e}"))?;
    let mut last = String::new();
    for port in ports {
        let url = format!("http://127.0.0.1:{port}{RPC_PATH}");
        match client
            .post(&url)
            .header("content-type", "application/json")
            .body("{}")
            .send()
            .await
        {
            Ok(resp) => {
                let body = resp.text().await.unwrap_or_default();
                if let Some(mut usage) = parse_quota_summary(&body, now_ms) {
                    usage.account_id = account_id;
                    return Ok(usage);
                }
                last = format!("port {port}: body did not parse");
            }
            Err(e) => last = format!("port {port}: {e}"),
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
