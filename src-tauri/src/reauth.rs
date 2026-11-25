//! On a click, let the official CLI repair the selected Windows login. The
//! webview supplies a registry key only. Paths and commands are resolved here.
use crate::{connections, paths, registry::ConnectionProfile};

fn valid_key(key: &str) -> bool {
    let bytes = key.as_bytes();
    (1..=101).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

pub(crate) fn can_start(profile: &ConnectionProfile, state: &str) -> bool {
    if !valid_key(&profile.key)
        || !matches!(profile.provider.as_str(), "claude" | "codex")
        || !matches!(profile.scope.as_str(), "personal" | "team")
        || !matches!(
            state,
            "credential_expired" | "login_required" | "wrong_account" | "wrong_scope"
        )
    {
        return false;
    }
    let directory = connections::profile_directory(profile);
    if !directory.is_absolute() {
        return false;
    }
    // 'credential_expired' is assigned only after verifying the email and,
    // for Claude, the organization. Repair a missing or mismatched identity
    // only in an isolated directory. Preserve both possible default locations.
    if state == "credential_expired" {
        return true;
    }
    let (default, name) = if profile.provider == "claude" {
        (paths::claude_config_dir(), ".claude")
    } else {
        (paths::codex_home(), ".codex")
    };
    let conventional = paths::home_dir().map(|p| p.join(name));
    ![default, conventional]
        .into_iter()
        .flatten()
        .any(|p| connections::same_directory(&directory, &p))
}

#[cfg(any(windows, test))]
fn select(profiles: &[ConnectionProfile], key: &str) -> Result<ConnectionProfile, &'static str> {
    if !valid_key(key) {
        return Err("auth_profile_changed");
    }
    let mut matching = profiles.iter().filter(|p| p.key.eq_ignore_ascii_case(key));
    match (matching.next(), matching.next()) {
        (Some(p), None) if p.key == key => Ok(p.clone()),
        _ => Err("auth_profile_changed"),
    }
}

#[cfg(any(windows, test))]
fn same_target(before: &ConnectionProfile, after: &ConnectionProfile) -> bool {
    before.key == after.key
        && before.provider == after.provider
        && before.email.eq_ignore_ascii_case(&after.email)
        && before.scope == after.scope
        && before.seat_annotation.as_ref().map(|s| &s.workspace_id)
            == after.seat_annotation.as_ref().map(|s| &s.workspace_id)
        && connections::same_directory(
            &connections::profile_directory(before),
            &connections::profile_directory(after),
        )
        && (before.expected_org_id.is_none() || before.expected_org_id == after.expected_org_id)
}

#[cfg(any(windows, test))]
fn status_result(state: &str) -> Result<(), &'static str> {
    match state {
        "ready" => Ok(()),
        "wrong_account" => Err("auth_wrong_account"),
        "wrong_scope" => Err("auth_wrong_scope"),
        _ => Err("auth_not_ready"),
    }
}

#[tauri::command]
pub async fn reauthenticate_profile(
    key: String,
) -> Result<connections::ConnectionStatus, &'static str> {
    #[cfg(windows)]
    return windows::authenticate(&key).await;
    #[cfg(not(windows))]
    {
        let _ = key;
        Err("auth_unavailable")
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        ffi::{OsStr, OsString},
        fs::{File, OpenOptions},
        io::{Read, Write},
        os::windows::{
            ffi::OsStrExt,
            fs::OpenOptionsExt,
            io::{AsRawHandle, FromRawHandle, OwnedHandle},
        },
        path::{Path, PathBuf},
        ptr::null,
        sync::atomic::{AtomicBool, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    use windows_sys::Win32::{
        Foundation::WAIT_OBJECT_0,
        System::Threading::{
            CREATE_NEW_CONSOLE, CreateProcessW, GetExitCodeProcess, INFINITE, PROCESS_INFORMATION,
            STARTUPINFOW, WaitForSingleObject,
        },
    };

    const HELPER: &[u8] = include_bytes!("../../scripts/Connect-Account.ps1");
    static LOGIN_ACTIVE: AtomicBool = AtomicBool::new(false);

    // A shared Codex home may back several rows. Serialize login flows so
    // the browser, the provider credential and the registry have one writer.
    struct LoginGuard;
    impl Drop for LoginGuard {
        fn drop(&mut self) {
            LOGIN_ACTIVE.store(false, Ordering::Release);
        }
    }

    fn current_status(
        target: &ConnectionProfile,
    ) -> Result<connections::ConnectionStatus, &'static str> {
        let registry = crate::registry::load();
        let current = select(&registry.profiles, &target.key)?;
        if !same_target(target, &current) {
            return Err("auth_profile_changed");
        }
        Ok(connections::profile_status(
            &current,
            crate::view::now_ms(),
            &[],
            &[],
        ))
    }

    fn verified(
        status: connections::ConnectionStatus,
    ) -> Result<connections::ConnectionStatus, &'static str> {
        status_result(status.state)?;
        Ok(status)
    }

    pub(super) async fn authenticate(
        key: &str,
    ) -> Result<connections::ConnectionStatus, &'static str> {
        let target = select(&crate::registry::load().profiles, key)?;
        LOGIN_ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "auth_busy")?;
        let guard = LoginGuard;
        let before = current_status(&target)?;
        if !before.can_reauthenticate {
            return verified(before);
        }
        if target.provider == "codex" && before.state == "credential_expired" {
            // The original CODEX_HOME lets Codex persist its own rotation.
            // Even a later quota/network failure may follow a valid renewal.
            let quota = crate::codex_usage::fetch_via_app_server_at(
                &connections::profile_directory(&target),
            )
            .await;
            let after = current_status(&target)?;
            if !after.can_reauthenticate || after.state != "credential_expired" {
                return verified(after);
            }
            // A CLI-managed credential can already serve quota even when
            // file metadata is stale. Avoid a browser login when the CLI
            // just proved access to the same member and active workspace.
            if quota.as_ref().is_ok_and(|usage| {
                connections::codex_profile_usage(&target, usage, crate::view::now_ms())
                    && crate::codex_usage::same_login_workspace(
                        &usage.account_id,
                        usage.email.as_deref(),
                        after.account_id.as_deref().unwrap_or(""),
                        Some(&target.email),
                    )
            }) {
                return Ok(after);
            }
        }
        tauri::async_runtime::spawn_blocking(move || {
            // Keep the guard in this worker until the console exits, even if
            // the original webview stops waiting for its IPC result.
            let _guard = guard;
            let before_launch = current_status(&target)?;
            if !before_launch.can_reauthenticate {
                return verified(before_launch);
            }
            let code = run_helper(&target.key)?;
            let checked = current_status(&target)?;
            match code {
                0 => verified(checked),
                2 => status_result(checked.state).and(Err("auth_not_ready")),
                // The helper rejected the isolated login before replacing
                // the original credential. Its local state can still be expired.
                3 => Err("auth_wrong_account"),
                4 => Err("auth_wrong_scope"),
                _ => Err("auth_login_failed"),
            }
        })
        .await
        .map_err(|_| "auth_unavailable")?
    }

    struct TemporaryScript(PathBuf);
    impl Drop for TemporaryScript {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn materialize_helper() -> std::io::Result<(TemporaryScript, File)> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("qhud-connect-{}-{stamp}.ps1", std::process::id()));
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .share_mode(0)
            .open(&path)?;
        let script = TemporaryScript(path);
        let written = output.write_all(HELPER);
        drop(output);
        written?;
        // Reopen read-only and verify the exact bytes while denying write
        // and delete sharing until PowerShell and its provider CLI exit.
        let mut pinned = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&script.0)?;
        let mut contents = Vec::with_capacity(HELPER.len());
        (&mut pinned)
            .take(HELPER.len() as u64 + 1)
            .read_to_end(&mut contents)?;
        if contents != HELPER {
            return Err(std::io::Error::other("unexpected helper contents"));
        }
        Ok((script, pinned))
    }

    fn run_helper(key: &str) -> Result<u32, &'static str> {
        let system = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .ok_or("auth_unavailable")?;
        let powershell = system.join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
        let registry = paths::config_dir()
            .and_then(|p| std::path::absolute(p.join("accounts.json")).ok())
            .ok_or("auth_unavailable")?;
        let (script, pin) = materialize_helper().map_err(|_| "auth_unavailable")?;
        // RemoteSigned applies only to this process. A group policy still
        // takes precedence. No code or paths from the UI become commands.
        let result = console_exit(
            &powershell,
            &[
                "-NoLogo".as_ref(),
                "-NoProfile".as_ref(),
                "-ExecutionPolicy".as_ref(),
                "RemoteSigned".as_ref(),
                "-File".as_ref(),
                script.0.as_os_str(),
                "-Profile".as_ref(),
                key.as_ref(),
                "-Reauthenticate".as_ref(),
                "-RegistryPath".as_ref(),
                registry.as_os_str(),
            ],
        );
        drop(pin);
        result
    }

    // Windows' process API takes a single UTF-16 buffer. These are switches,
    // a validated key, or Windows *file* paths (never a directory argument).
    // Double quotes cannot occur in Windows filenames. Reject control/null
    // and trailing separators rather than introducing a second quoting grammar.
    fn command_line(binary: &Path, args: &[&OsStr]) -> Result<Vec<u16>, &'static str> {
        let mut line = OsString::new();
        for arg in std::iter::once(binary.as_os_str()).chain(args.iter().copied()) {
            let wide: Vec<_> = arg.encode_wide().collect();
            if wide.is_empty()
                || wide.iter().any(|c| *c < 32 || *c == 34)
                || wide.last() == Some(&92)
            {
                return Err("auth_unavailable");
            }
            if !line.is_empty() {
                line.push(" ");
            }
            line.push("\"");
            line.push(arg);
            line.push("\"");
        }
        Ok(line.encode_wide().chain(Some(0)).collect())
    }

    fn console_exit(binary: &Path, args: &[&OsStr]) -> Result<u32, &'static str> {
        let mut line = command_line(binary, args)?;
        let executable: Vec<_> = binary.as_os_str().encode_wide().chain(Some(0)).collect();
        let cwd: Vec<_> = binary
            .parent()
            .ok_or("auth_unavailable")?
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let startup = STARTUPINFOW {
            cb: size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut process = PROCESS_INFORMATION::default();
        // SAFETY: NUL-terminated buffers stay live through CreateProcessW;
        // Windows gets the actual executable separately from the arguments.
        // No inherited handles means the new console owns stdin/out/err
        // even when the widget was started with all its output redirected.
        let ok = unsafe {
            CreateProcessW(
                executable.as_ptr(),
                line.as_mut_ptr(),
                null(),
                null(),
                0,
                CREATE_NEW_CONSOLE,
                null(),
                cwd.as_ptr(),
                &startup,
                &mut process,
            )
        };
        if ok == 0 {
            return Err("auth_unavailable");
        }
        // SAFETY: a successful CreateProcessW transfers these two open
        // handles to its caller. OwnedHandle closes each exactly once.
        let child = unsafe { OwnedHandle::from_raw_handle(process.hProcess) };
        drop(unsafe { OwnedHandle::from_raw_handle(process.hThread) });
        let mut exit_code = 0;
        // SAFETY: 'child' owns a live process handle for both queries.
        if unsafe { WaitForSingleObject(child.as_raw_handle(), INFINITE) } != WAIT_OBJECT_0
            || unsafe { GetExitCodeProcess(child.as_raw_handle(), &mut exit_code) } == 0
        {
            return Err("auth_login_failed");
        }
        Ok(exit_code)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Opt-in: an external fixture supplies a fake CLI and an isolated
        // registry. Exercise the normal IPC function and the real console
        // launcher without ever selecting an operator account or credential.
        #[test]
        #[ignore = "requires an isolated Windows console-login fixture"]
        fn console_login_smoke() {
            let root = std::fs::canonicalize(
                std::env::var_os("QHUD_REAUTH_SMOKE_ROOT").expect("fixture only"),
            )
            .unwrap();
            for name in ["QHUD_CONFIG_DIR", "QHUD_CODEX_BIN", "CODEX_HOME"] {
                let path = std::fs::canonicalize(
                    std::env::var_os(name).expect("fixture environment required"),
                )
                .unwrap();
                assert!(
                    path.starts_with(&root) && path != root,
                    "expected isolated fixture"
                );
            }
            let profiles = crate::registry::load().profiles;
            assert_eq!(profiles.len(), 1);
            let target = select(&profiles, "fixture-team").unwrap();
            assert_eq!(target.provider, "codex");
            assert_eq!(target.email, "member@example.test");
            let directory = connections::profile_directory(&target)
                .canonicalize()
                .unwrap();
            assert!(directory.starts_with(&root) && directory != root);
            assert_eq!(current_status(&target).unwrap().state, "login_required");
            let verified =
                tauri::async_runtime::block_on(reauthenticate_profile(target.key.clone())).unwrap();
            assert_eq!(verified.email, target.email);
            assert_eq!(verified.key, target.key);
            assert_eq!(verified.scope, target.scope);
            assert_eq!(verified.state, "ready");
            assert_eq!(current_status(&target).unwrap().state, "ready");
        }

        #[test]
        fn launch_arguments_remain_literal_and_windows_file_paths_may_contain_spaces() {
            let path = Path::new(r"C:\test tools\ps.exe");
            let script = OsStr::new(r"C:\member's files\A&B.ps1");
            let args = [
                OsStr::new("-File"),
                script,
                OsStr::new("-Profile"),
                OsStr::new("test-team"),
            ];
            let line = command_line(path, &args).unwrap();
            assert_eq!(
                String::from_utf16(&line[..line.len() - 1]).unwrap(),
                r#""C:\test tools\ps.exe" "-File" "C:\member's files\A&B.ps1" "-Profile" "test-team""#
            );
            assert_eq!(line.last(), Some(&0));
            for bad in ["a\"b", "a\0b", "a\nb", "a\\", ""] {
                assert!(command_line(path, &[OsStr::new(bad)]).is_err());
            }
        }

        #[test]
        fn verified_embedded_helper_is_pinned_until_login_finishes() {
            let (script, pinned) = materialize_helper().unwrap();
            let path = script.0.clone();
            assert_eq!(std::fs::read(&path).unwrap(), HELPER);
            assert!(OpenOptions::new().write(true).open(&path).is_err());
            drop(pinned);
            drop(script);
            assert!(!path.exists());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> ConnectionProfile {
        serde_json::from_value(serde_json::json!({
            "provider":"claude", "scope":"team", "email":"staff@example.com",
            "key":"claude-team", "label":"work", "directory":"~/qhud-test-profile",
            "expected_org_id":"org-one"
        }))
        .unwrap()
    }

    #[test]
    fn only_registered_unambiguous_keys_are_accepted() {
        let p = profile();
        for bad in ["", "-Command", "x/y", "a b", "x'&", "x\n", "팀", "a;whoami"] {
            assert!(!valid_key(bad));
            assert!(select(std::slice::from_ref(&p), bad).is_err());
        }
        assert!(!valid_key(&"a".repeat(102)));
        assert!(valid_key("a_B-8"));
        assert!(select(std::slice::from_ref(&p), "unknown").is_err());
        assert!(select(std::slice::from_ref(&p), "CLAUDE-TEAM").is_err());
        let mut alias = p.clone();
        alias.key = alias.key.to_uppercase();
        assert!(select(&[p.clone(), alias], &p.key).is_err());
        assert_eq!(select(std::slice::from_ref(&p), &p.key).unwrap(), p);
    }

    #[test]
    fn login_is_scoped_to_identity_and_protected_defaults() {
        let p = profile();
        assert!(can_start(&p, "credential_expired"));
        assert!(can_start(&p, "login_required"));
        for state in ["ready", "error"] {
            assert!(!can_start(&p, state));
        }
        let default = ConnectionProfile {
            directory: "~/.claude".into(),
            ..p.clone()
        };
        for state in ["login_required", "wrong_account", "wrong_scope"] {
            assert!(can_start(&p, state));
            assert!(!can_start(&default, state));
        }
        assert!(can_start(&default, "credential_expired"));
        let changed = ConnectionProfile {
            email: "other@example.com".into(),
            ..p.clone()
        };
        assert!(!same_target(&p, &changed));
        let other_org = ConnectionProfile {
            expected_org_id: Some("org-two".into()),
            ..p.clone()
        };
        assert!(!same_target(&p, &other_org));
        let move_home = ConnectionProfile {
            directory: "~/another-home".into(),
            ..p.clone()
        };
        assert!(!same_target(&p, &move_home));
        let unpinned = ConnectionProfile {
            expected_org_id: None,
            ..p.clone()
        };
        assert!(same_target(&unpinned, &p)); // first verified org was pinned
        assert_eq!(status_result("wrong_scope"), Err("auth_wrong_scope"));
        assert_eq!(status_result("credential_expired"), Err("auth_not_ready"));
        assert_eq!(status_result("ready"), Ok(()));
    }
}
