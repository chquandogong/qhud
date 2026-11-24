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
    /// Plan/seat as the operator knows it (e.g. "pro", "free"). The
    /// provider only reports this once a credential is live, so for an
    /// account that needs re-auth this is the only way to show it at all.
    #[serde(default)]
    pub plan: Option<String>,
}

/// A requested login/scope, including profiles that have never signed in.
/// Directory paths stay in the local registry and are not part of HUD status.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ConnectionProfile {
    pub key: String,
    pub provider: String,
    pub label: String,
    pub email: String,
    pub scope: String,
    pub directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_org_id: Option<String>,
    /// Manually verified display metadata, independent of CLI authentication
    /// and provider quota. Validated against the requested login and workspace
    /// by `connections` before it can reach the HUD.
    #[serde(
        default,
        deserialize_with = "deserialize_optional_note",
        skip_serializing_if = "Option::is_none"
    )]
    pub seat_annotation: Option<SeatAnnotation>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_note",
        skip_serializing_if = "Option::is_none"
    )]
    pub observed_plan: Option<ObservedPlan>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SeatAnnotation {
    pub email: String,
    pub workspace_id: String,
    pub workspace_name: String,
    pub seat_type: String,
    pub verified_at_ms: u64,
}

/// Plan observed on the provider website, independent of the CLI's organization.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ObservedPlan {
    pub email: String,
    pub plan: String,
    pub verified_at_ms: u64,
}

/// Billing page observation for one member, workspace and plan. Kept
/// separately from login profiles so native Google accounts can use it too.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BillingCycleObservation {
    pub provider: String,
    pub email: String,
    pub workspace_id: String,
    pub plan: String,
    pub cycle: String,
    pub verified_at_ms: u64,
}

impl BillingCycleObservation {
    fn valid_at(&self, now_ms: u64) -> bool {
        let clean_id = |id: &str, limit| {
            !id.is_empty()
                && id.len() <= limit
                && !id.chars().any(|c| c.is_control() || c.is_whitespace())
        };
        let valid_email = clean_id(&self.email, 320)
            && self.email.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty() && !domain.is_empty() && !domain.contains('@')
            });
        let valid_workspace = match self.provider.as_str() {
            "agy" => self.workspace_id.eq_ignore_ascii_case(&self.email),
            "codex" | "claude" => clean_id(&self.workspace_id, 128),
            _ => false,
        };
        valid_email
            && valid_workspace
            && !self.plan.is_empty()
            && self.plan.len() <= 64
            && self.plan.trim() == self.plan
            && self.plan.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b' ' | b'_' | b'-')
            })
            && matches!(self.cycle.as_str(), "monthly" | "annual")
            && self.verified_at_ms > 0
            && self.verified_at_ms <= now_ms
    }
}

fn deserialize_optional_note<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    // A malformed optional note must not erase otherwise usable profiles or
    // registry settings. Semantic and timestamp checks happen at display time.
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).ok())
}

#[derive(Debug, Clone, Default)]
pub struct Registry {
    pub labels: HashMap<String, String>,
    /// account_id -> human workspace name. `accounts/check` omits the name
    /// for some workspaces, which otherwise render as a hex id prefix.
    pub workspace_names: HashMap<String, String>,
    /// account_id -> display plan for that workspace. The plan belongs to the
    /// WORKSPACE, not the login, so the account line must not carry one.
    pub workspace_plans: HashMap<String, String>,
    pub known: Vec<KnownAccount>,
    pub forgotten: HashSet<String>,
    /// Extra Claude config dirs (D-015), one per additional account the
    /// operator keeps signed in via `CLAUDE_CONFIG_DIR=<dir> claude`.
    /// Each contains that account's own `.claude.json` (identity + usage
    /// cache) and `.credentials.json`. `~` expands to $HOME.
    pub claude_config_dirs: Vec<String>,
    /// Extra Codex homes (D-015), one per additional account kept signed
    /// in via `CODEX_HOME=<dir> codex`. Each contains its own
    /// `auth.json` (+ parked siblings). `~` expands to $HOME.
    pub codex_homes: Vec<String>,
    /// Desired connections are separate from verified account identities.
    /// Adding a profile never activates its credential directory by itself.
    pub profiles: Vec<ConnectionProfile>,
    pub billing_cycles: Vec<BillingCycleObservation>,
}

/// Expands a single leading `~/` (or bare `~`) against the given home.
/// Anything else passes through untouched — no general shell expansion.
pub fn expand_tilde(path: &str, home: &str) -> String {
    if path == "~" {
        return home.to_string();
    }
    match path.strip_prefix("~/") {
        Some(rest) => format!("{home}/{rest}"),
        None => path.to_string(),
    }
}

/// A row for a known-but-not-live account.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Placeholder {
    pub provider: String,
    pub key: String,
    pub label: Option<String>,
    pub hint: Option<String>,
    pub plan: Option<String>,
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
        workspace_names: HashMap<String, String>,
        #[serde(default)]
        workspace_plans: HashMap<String, String>,
        #[serde(default)]
        known: Vec<KnownAccount>,
        #[serde(default)]
        forgotten: Vec<String>,
        #[serde(default)]
        claude_config_dirs: Vec<String>,
        #[serde(default)]
        codex_homes: Vec<String>,
        #[serde(default)]
        profiles: Vec<ConnectionProfile>,
        #[serde(default)]
        billing_cycles: serde_json::Value,
    }
    let raw: Raw = serde_json::from_str(json).unwrap_or_default();
    // Preserve usable settings and sibling notes if an optional entry was
    // mistyped in this operator-owned file.
    let billing_cycles = match raw.billing_cycles {
        serde_json::Value::Array(entries) => {
            let now_ms = crate::view::now_ms();
            entries
                .into_iter()
                .filter_map(|entry| serde_json::from_value::<BillingCycleObservation>(entry).ok())
                .filter(|note| note.valid_at(now_ms))
                .collect()
        }
        _ => Vec::new(),
    };
    Registry {
        labels: raw.labels,
        workspace_names: raw.workspace_names,
        workspace_plans: raw.workspace_plans,
        known: raw.known,
        forgotten: raw.forgotten.into_iter().collect(),
        claude_config_dirs: raw.claude_config_dirs,
        codex_homes: raw.codex_homes,
        profiles: raw.profiles,
        billing_cycles,
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
            plan: k.plan.clone(),
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
            .map_err(|e| format!("registry file is not valid JSON: {}", crate::diag::json(&e)))?
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
    serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("registry serialize failed: {}", crate::diag::json(&e)))
}

fn path() -> Option<std::path::PathBuf> {
    crate::paths::config_dir().map(|d| d.join("accounts.json"))
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
    let p = path().ok_or("qhud configuration directory is unavailable")?;
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| {
            crate::diag::failure("create qhud configuration directory", crate::diag::io(&e))
        })?;
    }
    let existing = std::fs::read_to_string(&p).unwrap_or_default();
    let next = forget(&existing, provider, key)?;
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, next)
        .map_err(|e| crate::diag::failure("registry write", crate::diag::io(&e)))?;
    std::fs::rename(&tmp, &p)
        .map_err(|e| crate::diag::failure("registry rename", crate::diag::io(&e)))
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
        assert!(parse("{}").claude_config_dirs.is_empty());
        assert!(parse("{}").profiles.is_empty());
        assert!(parse("{}").billing_cycles.is_empty());
    }

    #[test]
    fn billing_cycle_notes_keep_their_scope_and_ignore_malformed_siblings() {
        use serde_json::json;
        let note = json!({
            "provider":"codex", "email":"one@example.com", "workspace_id":"workspace-1",
            "plan":"business", "cycle":"annual", "verified_at_ms":123
        });
        let mut dirty = vec![note.clone(), json!(null), json!({"plan":"business"})];
        for (key, bad) in [
            ("verified_at_ms", json!(-1)),
            ("verified_at_ms", json!(0)),
            ("verified_at_ms", json!(u64::MAX)),
            ("cycle", json!("year")),
            ("provider", json!("elsewhere")),
            ("email", json!("a@@b")),
            ("workspace_id", json!("a\nb")),
            ("workspace_id", json!("w".repeat(129))),
            ("plan", json!("Business")),
        ] {
            let mut invalid = note.clone();
            invalid[key] = bad;
            dirty.push(invalid);
        }
        let load_notes = |value| {
            let reg = parse(&json!({"billing_cycles":value,"labels":{"x":"keep"}}).to_string());
            assert_eq!(reg.labels["x"], "keep");
            reg.billing_cycles
        };
        assert_eq!(
            serde_json::to_value(load_notes(json!(dirty))).unwrap(),
            json!([note])
        );
        assert!(load_notes(json!("annual")).is_empty());
    }

    #[test]
    fn google_billing_cycle_is_bound_to_the_account_email_without_a_profile() {
        use serde_json::json;
        let mut note = json!({
            "provider":"agy", "email":"one@example.com", "workspace_id":"One@Example.com",
            "plan":"google ai pro", "cycle":"monthly", "verified_at_ms":123
        });
        let input = |note: &serde_json::Value| json!({"billing_cycles":[note]}).to_string();
        let reg = parse(&input(&note));
        assert_eq!(reg.billing_cycles.len(), 1);
        assert!(reg.profiles.is_empty());
        note["workspace_id"] = json!("two@example.com");
        assert!(parse(&input(&note)).billing_cycles.is_empty());
    }

    #[test]
    fn requested_profiles_do_not_activate_credentials_or_invent_identities() {
        let reg = parse(
            r#"{"profiles":[{"key":"work-team","provider":"claude",
            "label":"Work team","email":"work@example.com","scope":"team",
            "directory":"~/private/work-team","expected_org_id":"org-team"}]}"#,
        );
        assert_eq!(reg.profiles.len(), 1);
        assert_eq!(reg.profiles[0].expected_org_id.as_deref(), Some("org-team"));
        assert!(reg.known.is_empty());
        assert!(reg.claude_config_dirs.is_empty());
        assert!(reg.codex_homes.is_empty());
        let legacy = parse(
            r#"{"profiles":[{"key":"personal","provider":"codex",
            "label":"Personal","email":"one@example.com","scope":"personal",
            "directory":"~/.codex"}]}"#,
        );
        assert!(legacy.profiles[0].expected_org_id.is_none());
        assert!(legacy.profiles[0].seat_annotation.is_none());
        assert!(legacy.profiles[0].observed_plan.is_none());
        assert!(
            serde_json::to_value(&legacy.profiles[0])
                .unwrap()
                .get("seat_annotation")
                .is_none()
        );
    }

    #[test]
    fn seat_annotation_round_trips_without_activating_a_profile() {
        let reg = parse(
            r#"{"profiles":[{"key":"team","provider":"codex","label":"Team",
                "email":"one@example.com","scope":"team","directory":"~/isolated",
                "seat_annotation":{"email":"one@example.com","workspace_id":"workspace-1",
                    "workspace_name":"Example","seat_type":"standard","verified_at_ms":7}}]}"#,
        );
        assert_eq!(reg.profiles.len(), 1);
        let annotation = reg.profiles[0].seat_annotation.as_ref().unwrap();
        assert_eq!(annotation.seat_type, "standard");
        assert_eq!(annotation.workspace_id, "workspace-1");
        assert_eq!(annotation.verified_at_ms, 7);
        let serialized = serde_json::to_string(&reg.profiles[0]).unwrap();
        let decoded: ConnectionProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decoded, reg.profiles[0]);
        assert!(reg.codex_homes.is_empty());
        assert!(reg.known.is_empty());
    }

    #[test]
    fn optional_observed_plan_preserves_unregistered_profile_and_other_settings() {
        let mut doc = serde_json::json!({
            "labels": {"claude:example": "Keep me"},
            "profiles": [{"key":"personal","provider":"claude","label":"Personal",
                "email":"one@example.com","scope":"personal","directory":"~/isolated",
                "observed_plan":{"email":"one@example.com","plan":"free","verified_at_ms":7}}]
        });
        let reg = parse(&doc.to_string());
        assert!(reg.claude_config_dirs.is_empty());
        assert_eq!(
            serde_json::to_value(&reg.profiles[0].observed_plan).unwrap(),
            doc["profiles"][0]["observed_plan"]
        );
        doc["profiles"][0]["observed_plan"]["verified_at_ms"] = serde_json::json!(-1);
        let reg = parse(&doc.to_string());
        assert!(reg.profiles[0].observed_plan.is_none());
        assert_eq!(reg.labels["claude:example"], "Keep me");
    }

    #[test]
    fn malformed_optional_seat_annotation_preserves_other_registry_data() {
        for annotation in [
            serde_json::Value::Null,
            serde_json::json!("standard"),
            serde_json::json!({"seat_type": "standard"}),
            serde_json::json!({
                "email": "one@example.com", "workspace_id": "workspace-1",
                "workspace_name": "Example", "seat_type": "standard", "verified_at_ms": -1
            }),
        ] {
            let doc = serde_json::json!({
                "labels": {"codex:one": "Keep me"},
                "profiles": [{"key": "team", "provider": "codex", "label": "Team",
                    "email": "one@example.com", "scope": "team", "directory": "~/isolated",
                    "seat_annotation": annotation}]
            });
            let reg = parse(&doc.to_string());
            assert_eq!(reg.profiles.len(), 1);
            assert!(reg.profiles[0].seat_annotation.is_none());
            assert_eq!(
                reg.labels.get("codex:one").map(String::as_str),
                Some("Keep me")
            );
        }
    }

    #[test]
    fn parse_reads_extra_config_dirs() {
        let r = parse(
            r#"{"claude_config_dirs":["~/claude-personal","/abs/dir"],
                "codex_homes":["~/.codex-dogu"]}"#,
        );
        assert_eq!(r.claude_config_dirs, vec!["~/claude-personal", "/abs/dir"]);
        assert_eq!(r.codex_homes, vec!["~/.codex-dogu"]);
    }

    #[test]
    fn expand_tilde_touches_only_a_leading_tilde() {
        assert_eq!(expand_tilde("~/x/y", "/home/u"), "/home/u/x/y");
        assert_eq!(expand_tilde("~", "/home/u"), "/home/u");
        assert_eq!(expand_tilde("/abs/~/x", "/home/u"), "/abs/~/x");
        assert_eq!(expand_tilde("rel/x", "/home/u"), "rel/x");
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
    fn placeholder_carries_the_operator_supplied_plan() {
        // The provider only reports plan once a credential is live, so for
        // an account needing re-auth the registry is the ONLY source.
        let reg = parse(
            r#"{"known":[{"provider":"codex","key":"zzz","label":"a@b.c",
                          "plan":"pro"}]}"#,
        );
        let out = placeholders(&reg, &[]);
        assert_eq!(out[0].plan.as_deref(), Some("pro"));
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
