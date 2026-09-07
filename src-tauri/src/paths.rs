//! Shared provider and qhud paths, including native Windows installations.
use std::path::PathBuf;

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
}

fn resolve_home(home: Option<PathBuf>, profile: Option<PathBuf>, windows: bool) -> Option<PathBuf> {
    if windows {
        profile.or(home)
    } else {
        home.or(profile)
    }
}

pub fn home_dir() -> Option<PathBuf> {
    resolve_home(env_path("HOME"), env_path("USERPROFILE"), cfg!(windows))
}

pub fn codex_home() -> Option<PathBuf> {
    env_path("CODEX_HOME").or_else(|| home_dir().map(|h| h.join(".codex")))
}

pub fn claude_config_dir() -> Option<PathBuf> {
    env_path("CLAUDE_CONFIG_DIR").or_else(|| home_dir().map(|h| h.join(".claude")))
}

pub fn claude_config_file() -> Option<PathBuf> {
    env_path("CLAUDE_CONFIG_DIR")
        .map(|d| d.join(".claude.json"))
        .or_else(|| home_dir().map(|h| h.join(".claude.json")))
}

/// Keep an existing legacy registry usable; new Windows installs use AppData.
pub fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = env_path("QHUD_CONFIG_DIR") {
        return Some(dir);
    }
    if let Some(dir) = env_path("XDG_CONFIG_HOME") {
        return Some(dir.join("qhud"));
    }
    let legacy = home_dir().map(|h| h.join(".config/qhud"));
    if cfg!(windows) {
        if let Some(dir) = legacy.as_ref().filter(|d| d.is_dir()) {
            return Some(dir.clone());
        }
        if let Some(dir) = env_path("APPDATA") {
            return Some(dir.join("qhud"));
        }
    }
    legacy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_prefers_native_profile_over_shell_home() {
        assert_eq!(
            resolve_home(Some("/git/home".into()), Some("C:/Users/user".into()), true),
            Some("C:/Users/user".into())
        );
        assert_eq!(
            resolve_home(
                Some("/home/user".into()),
                Some("C:/Users/user".into()),
                false
            ),
            Some("/home/user".into())
        );
    }

    #[test]
    fn missing_home_has_a_safe_fallback() {
        assert_eq!(
            resolve_home(None, Some("C:/Users/user".into()), false),
            Some("C:/Users/user".into())
        );
        assert_eq!(resolve_home(None, None, true), None);
    }
}
