//! Operator intent about accounts, persisted across restarts.
//!
//! Identity *detection* (`accounts`) can only see accounts with a live
//! credential on disk. But an account you signed out of still has quota
//! ticking, and the operator still wants to know about it. So the
//! registry records accounts that have **ever** connected and lets qhud
//! show them as placeholders — visible, clearly not-live, and actionable
//! by a click rather than by a background fetch.
//!
//! Three rules, in order:
//!  1. A live credential always shows. Present facts are never hidden.
//!  2. A known account with no live credential shows as a placeholder
//!     with `state: "needs_reauth"` and a hint on how to restore it.
//!  3. An account the operator explicitly forgot never shows as a
//!     placeholder. Forgetting does **not** hide a live account — log
//!     out for that — because silently hiding something in active use is
//!     the worse failure.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// An account the operator has connected at some point. `key` matches
/// the identity key `accounts` derives: account id when the provider
/// exposes one, else email.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct KnownAccount {
    pub provider: String,
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    /// What the operator must do to make this account live again.
    #[serde(default)]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Registry {
    pub labels: HashMap<String, String>,
    pub known: Vec<KnownAccount>,
    pub forgotten: HashSet<String>,
}

/// A row for a known-but-not-live account.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Placeholder {
    pub provider: String,
    pub key: String,
    pub label: Option<String>,
    pub hint: Option<String>,
}

/// Composite key used by `labels` and `forgotten`.
pub fn id(provider: &str, key: &str) -> String {
    format!("{provider}:{key}")
}

/// Parses the registry file. Malformed content yields an empty registry:
/// none of this is load-bearing enough to fail a tick over.
pub fn parse(json: &str) -> Registry {
    #[derive(Deserialize, Default)]
    struct Raw {
        #[serde(default)]
        labels: HashMap<String, String>,
        #[serde(default)]
        known: Vec<KnownAccount>,
        #[serde(default)]
        forgotten: Vec<String>,
    }
    let raw: Raw = serde_json::from_str(json).unwrap_or_default();
    Registry {
        labels: raw.labels,
        known: raw.known,
        forgotten: raw.forgotten.into_iter().collect(),
    }
}

/// Known accounts that have no live credential and were not forgotten.
///
/// `active` is the provider→key list that `accounts::detect_all` found.
pub fn placeholders(reg: &Registry, active: &[(String, String)]) -> Vec<Placeholder> {
    reg.known
        .iter()
        .filter(|k| {
            let live = active
                .iter()
                .any(|(p, key)| *p == k.provider && *key == k.key);
            !live && !reg.forgotten.contains(&id(&k.provider, &k.key))
        })
        .map(|k| Placeholder {
            provider: k.provider.clone(),
            key: k.key.clone(),
            // An explicit label from `labels` wins over the inline one.
            label: reg
                .labels
                .get(&id(&k.provider, &k.key))
                .cloned()
                .or_else(|| k.label.clone()),
            hint: k.hint.clone(),
        })
        .collect()
}

/// Adds an account to the forgotten set, returning the JSON to write back.
/// Preserves everything else in the file, including keys qhud does not
/// model, so hand-edits are never clobbered.
pub fn forget(existing_json: &str, provider: &str, key: &str) -> Result<String, String> {
    // Edit the generic Value, not a typed struct: the file is
    // operator-owned and carries keys qhud does not model (`_readme`,
    // `known_accounts`). Round-tripping through a struct would delete them.
    let mut doc: serde_json::Value = if existing_json.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(existing_json)
            .map_err(|e| format!("registry file is not valid JSON: {e}"))?
    };
    if !doc.is_object() {
        return Err("registry file is not a JSON object".into());
    }
    let entry = id(provider, key);
    let list = doc
        .as_object_mut()
        .expect("checked is_object above")
        .entry("forgotten")
        .or_insert_with(|| serde_json::json!([]));
    let arr = list
        .as_array_mut()
        .ok_or("registry `forgotten` is not an array")?;
    if !arr.iter().any(|v| v.as_str() == Some(entry.as_str())) {
        arr.push(serde_json::Value::String(entry));
    }
    serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())
}

fn path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .map(|h| h.join(".config/qhud/accounts.json"))
}

/// Reads the registry from disk; an absent file is an empty registry.
pub fn load() -> Registry {
    path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| parse(&s))
        .unwrap_or_default()
}

/// Records the operator's decision to stop showing an account.
///
/// Written temp+rename so a reader never sees a torn file — the same
/// hazard that made the statusline sidefiles blink out.
pub fn forget_and_save(provider: &str, key: &str) -> Result<(), String> {
    let p = path().ok_or("HOME is not set")?;
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {dir:?}: {e}"))?;
    }
    let existing = std::fs::read_to_string(&p).unwrap_or_default();
    let next = forget(&existing, provider, key)?;
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, next).map_err(|e| format!("write failed: {e}"))?;
    std::fs::rename(&tmp, &p).map_err(|e| format!("rename failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REG: &str = r#"{
      "schema": 1,
      "labels": {"codex:aaa": "work"},
      "known": [
        {"provider":"codex","key":"aaa","label":"work"},
        {"provider":"codex","key":"bbb","label":"personal",
         "hint":"parked at ~/.codex/auth.json.dogu — run `codex login` to switch"},
        {"provider":"agy","key":"old@example.com","hint":"sign in again in agy"}
      ],
      "forgotten": ["agy:old@example.com"]
    }"#;

    #[test]
    fn parse_reads_labels_known_and_forgotten() {
        let r = parse(REG);
        assert_eq!(r.labels.get("codex:aaa").map(String::as_str), Some("work"));
        assert_eq!(r.known.len(), 3);
        assert!(r.forgotten.contains("agy:old@example.com"));
    }

    #[test]
    fn parse_of_junk_is_empty_not_an_error() {
        assert!(parse("not json").known.is_empty());
        assert!(parse("{}").forgotten.is_empty());
    }

    #[test]
    fn known_account_without_live_credential_becomes_a_placeholder() {
        // codex:aaa is live; codex:bbb is not; agy:old was forgotten.
        let active = vec![("codex".to_string(), "aaa".to_string())];

        let out = placeholders(&parse(REG), &active);

        assert_eq!(out.len(), 1, "only bbb qualifies");
        assert_eq!(out[0].key, "bbb");
        assert_eq!(out[0].label.as_deref(), Some("personal"));
        assert!(
            out[0].hint.as_deref().unwrap().contains("codex login"),
            "a placeholder must say how to restore the account"
        );
    }

    #[test]
    fn forgotten_account_never_becomes_a_placeholder() {
        let out = placeholders(&parse(REG), &[]);
        assert!(
            !out.iter().any(|p| p.key == "old@example.com"),
            "an explicitly forgotten account stays hidden"
        );
        // Nothing is live here, so both codex entries qualify; the point
        // is that the forgotten agy entry is the only one filtered out.
        assert_eq!(out.len(), 2, "only agy:old is suppressed");
    }

    #[test]
    fn forgetting_a_live_account_does_not_hide_it() {
        // Rule 1 beats rule 3: hiding something in active use is worse
        // than showing something the operator tried to dismiss.
        let reg = parse(
            r#"{"known":[{"provider":"codex","key":"aaa"}],
                            "forgotten":["codex:aaa"]}"#,
        );
        let active = vec![("codex".to_string(), "aaa".to_string())];

        let out = placeholders(&reg, &active);

        assert!(out.is_empty(), "live account is not a placeholder at all");
        // ...and the caller still renders it from detection, so it stays
        // visible. Nothing here can suppress a detected account.
    }

    #[test]
    fn forget_appends_and_preserves_unmodelled_keys() {
        let before = r#"{"schema":1,"labels":{"codex:aaa":"work"},
          "_readme":["keep me"],"forgotten":["agy:x@y.z"]}"#;

        let after = forget(before, "codex", "aaa").expect("writes back");
        let v: serde_json::Value = serde_json::from_str(&after).unwrap();

        let f: Vec<&str> = v["forgotten"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        assert!(f.contains(&"codex:aaa"), "new entry added");
        assert!(f.contains(&"agy:x@y.z"), "existing entry kept");
        assert_eq!(v["_readme"][0], "keep me", "hand-edited keys survive");
        assert_eq!(v["labels"]["codex:aaa"], "work");
    }

    #[test]
    fn forget_is_idempotent_and_seeds_a_missing_file() {
        let once = forget("{}", "codex", "aaa").unwrap();
        let twice = forget(&once, "codex", "aaa").unwrap();
        let v: serde_json::Value = serde_json::from_str(&twice).unwrap();
        assert_eq!(v["forgotten"].as_array().unwrap().len(), 1);
    }
}
