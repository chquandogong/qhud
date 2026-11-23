//! Passive status for requested logins. Local identity hints never manufacture
//! a provider account or quota row, and no provider request is made here.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    accounts::AccountLabel,
    codex_usage::WorkspaceUsage,
    provider_health::{self, AuthMetadata},
    registry::{ConnectionProfile, ObservedPlan, SeatAnnotation},
    view::ProviderQuota,
};

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub provider: String,
    pub key: String,
    pub label: String,
    pub email: String,
    pub scope: String,
    pub state: &'static str,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    pub has_row: bool,
    /// A dated manual observation; it never establishes authentication or quota.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat_annotation: Option<SeatAnnotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_plan: Option<ObservedPlan>,
}

fn same_email(left: &str, right: &str) -> bool {
    !left.trim().is_empty() && left.trim().eq_ignore_ascii_case(right.trim())
}

fn claude_scope(org_type: Option<&str>) -> Option<&'static str> {
    match org_type? {
        "claude_free" | "claude_pro" | "claude_max" | "claude_ultra" => Some("personal"),
        "claude_team" | "claude_enterprise" => Some("team"),
        _ => None,
    }
}

fn codex_scope(plan: Option<&str>) -> Option<&'static str> {
    match plan? {
        "free" | "go" | "plus" | "prolite" | "pro" | "promax" => Some("personal"),
        "team" | "business" | "enterprise" => Some("team"),
        _ => None,
    }
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.filter(|s| !s.trim().is_empty())
}

fn status_from_metadata(
    profile: &ConnectionProfile,
    now_ms: u64,
    identity: Option<AccountLabel>,
    auth: Option<AuthMetadata>,
    quotas: &[ProviderQuota],
    workspaces: &[WorkspaceUsage],
) -> ConnectionStatus {
    let known_wrong_account = identity
        .as_ref()
        .and_then(|account| nonempty(account.email.as_deref()))
        .is_some_and(|email| !same_email(email, &profile.email));
    let mut out = login_status_from_metadata(profile, now_ms, identity, auth, quotas, workspaces);
    // Decorate the existing connection result after all identity checks. A
    // manual observation never creates a row, authenticates a profile or
    // substitutes its workspace for one returned by the provider.
    if !known_wrong_account && !matches!(out.state, "wrong_account" | "wrong_scope") {
        out.seat_annotation = validated_seat_annotation(profile, now_ms).filter(|annotation| {
            !out.has_row || out.account_id.as_deref() == Some(annotation.workspace_id.as_str())
        });
    }
    // A team credential for the same email can coexist with a web-only
    // personal subscription. Keep its observation out of provider usage rows.
    if !known_wrong_account && !out.has_row {
        out.observed_plan = validated_observed_plan(profile, now_ms);
    }
    out
}

fn validated_observed_plan(profile: &ConnectionProfile, now_ms: u64) -> Option<ObservedPlan> {
    let note = profile.observed_plan.as_ref()?;
    let (local, domain) = note.email.split_once('@')?;
    (profile.provider == "claude"
        && profile.scope == "personal"
        && note.plan == "free"
        && !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && note.email.len() <= 254
        && !note
            .email
            .chars()
            .any(|c| c.is_control() || c.is_whitespace())
        && same_email(&note.email, &profile.email)
        && (1..=now_ms).contains(&note.verified_at_ms))
    .then(|| note.clone())
}

fn validated_seat_annotation(profile: &ConnectionProfile, now_ms: u64) -> Option<SeatAnnotation> {
    let annotation = profile.seat_annotation.as_ref()?;
    let bounded_text = |text: &str, limit: usize| {
        !text.trim().is_empty()
            && text.chars().count() <= limit
            && !text.chars().any(char::is_control)
    };
    (profile.provider == "codex"
        && profile.scope == "team"
        && bounded_text(&annotation.email, 320)
        && same_email(&annotation.email, &profile.email)
        && bounded_text(&annotation.workspace_id, 128)
        && bounded_text(&annotation.workspace_name, 160)
        && matches!(annotation.seat_type.as_str(), "standard" | "premium")
        && annotation.verified_at_ms > 0
        && annotation.verified_at_ms <= now_ms)
        .then(|| annotation.clone())
}

fn login_status_from_metadata(
    profile: &ConnectionProfile,
    now_ms: u64,
    identity: Option<AccountLabel>,
    auth: Option<AuthMetadata>,
    quotas: &[ProviderQuota],
    workspaces: &[WorkspaceUsage],
) -> ConnectionStatus {
    let mut out = ConnectionStatus {
        provider: profile.provider.clone(),
        key: profile.key.clone(),
        label: profile.label.clone(),
        email: profile.email.clone(),
        scope: profile.scope.clone(),
        state: "login_required",
        message: "Sign in to this isolated profile with the requested account.",
        expires_at_ms: auth.as_ref().and_then(|a| a.expires_at_ms),
        account_id: None,
        org_id: None,
        has_row: false,
        seat_annotation: None,
        observed_plan: None,
    };
    if !matches!(profile.provider.as_str(), "claude" | "codex")
        || !matches!(profile.scope.as_str(), "personal" | "team")
    {
        out.state = "wrong_scope";
        out.message = "This profile has an unsupported provider or scope.";
        return out;
    }
    let Some(auth) = auth.filter(|a| a.has_credential) else {
        return out;
    };
    let Some(identity) = identity else {
        out.message = "The login identity is unavailable. Sign in through the provider CLI.";
        return out;
    };
    out.account_id = identity.account_id.clone().filter(|s| !s.is_empty());
    out.org_id = identity.org_id.clone().filter(|s| !s.is_empty());
    if out.account_id.is_none() || nonempty(identity.email.as_deref()).is_none() {
        out.message = "The login identity is incomplete. Sign in through the provider CLI.";
        return out;
    }
    if !same_email(identity.email.as_deref().unwrap_or(""), &profile.email) {
        out.state = "wrong_account";
        out.message = "This profile is signed in with a different email address.";
        return out;
    }
    if profile.provider == "claude"
        && (out.org_id.is_none()
            || claude_scope(identity.org_type.as_deref()) != Some(profile.scope.as_str())
            || profile
                .expected_org_id
                .as_ref()
                .is_some_and(|expected| identity.org_id.as_deref() != Some(expected.as_str())))
    {
        out.state = "wrong_scope";
        out.message = "Choose the requested personal or team organization in the provider login.";
        return out;
    }
    if auth.expires_at_ms.is_some_and(|expires| expires <= now_ms) {
        out.state = "credential_expired";
        out.message = "This profile's credential has expired. Reconnect through the provider CLI.";
        return out;
    }

    out.state = "ready";
    out.message = "The expected local login was found. Refresh to check provider usage.";
    if profile.provider == "claude" {
        out.has_row = quotas.iter().any(|q| {
            q.provider == "claude"
                && q.account.as_ref().is_some_and(|row| {
                    row.account_id == identity.account_id
                        && row.org_id == identity.org_id
                        && row
                            .email
                            .as_deref()
                            .is_some_and(|email| same_email(email, &profile.email))
                })
        });
    } else {
        let mut matching = workspaces.iter().filter(|w| {
            w.email
                .as_deref()
                .is_some_and(|email| same_email(email, &profile.email))
                && codex_scope(w.plan_type.as_deref()) == Some(profile.scope.as_str())
        });
        // A verified note can select among already observed workspaces for
        // this login and scope. It cannot create or authorize a workspace.
        let annotation = validated_seat_annotation(profile, now_ms);
        let row = annotation
            .as_ref()
            .and_then(|annotation| {
                matching
                    .clone()
                    .find(|row| row.account_id == annotation.workspace_id)
            })
            .or_else(|| matching.next());
        if let Some(row) = row {
            out.has_row = true;
            // One Codex login can enumerate both personal and team workspaces.
            // Only a real provider workspace can replace the credential's own ID.
            out.account_id = Some(row.account_id.clone());
        }
    }
    out
}

fn same_directory(left: &Path, right: &Path) -> bool {
    let normalize = |path: &Path| {
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let text = path.to_string_lossy().replace('\\', "/");
        let text = text.trim_end_matches('/').to_string();
        if cfg!(windows) {
            text.to_ascii_lowercase()
        } else {
            text
        }
    };
    normalize(left) == normalize(right)
}

fn claude_config_file(directory: &Path) -> PathBuf {
    if crate::paths::claude_config_dir()
        .as_ref()
        .is_some_and(|default| same_directory(directory, default))
        && let Some(config) = crate::paths::claude_config_file()
    {
        return config;
    }
    directory.join(".claude.json")
}

pub fn statuses(
    now_ms: u64,
    quotas: &[ProviderQuota],
    workspaces: &[WorkspaceUsage],
) -> Vec<ConnectionStatus> {
    let registry = crate::registry::load();
    let home = crate::paths::home_dir().unwrap_or_default();
    registry
        .profiles
        .iter()
        .map(|profile| {
            let directory = PathBuf::from(crate::registry::expand_tilde(
                &profile.directory,
                &home.to_string_lossy(),
            ));
            let (identity, auth) = match profile.provider.as_str() {
                "claude" => {
                    let identity = provider_health::read_metadata(&claude_config_file(&directory))
                        .and_then(|body| crate::accounts::claude_account(&body));
                    let auth = provider_health::read_metadata(&directory.join(".credentials.json"))
                        .and_then(|body| provider_health::claude_auth_metadata(&body));
                    (identity, auth)
                }
                "codex" => {
                    let body = provider_health::read_metadata(&directory.join("auth.json"));
                    let identity = body
                        .as_ref()
                        .and_then(|s| crate::accounts::codex_account(s));
                    let auth = body
                        .as_ref()
                        .and_then(|s| provider_health::codex_auth_metadata(s));
                    (identity, auth)
                }
                _ => (None, None),
            };
            status_from_metadata(profile, now_ms, identity, auth, quotas, workspaces)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(provider: &str, scope: &str) -> ConnectionProfile {
        ConnectionProfile {
            key: "test-profile".into(),
            provider: provider.into(),
            label: "Test profile".into(),
            email: "person@example.com".into(),
            scope: scope.into(),
            directory: "PRIVATE-PATH-MUST-NOT-SERIALIZE".into(),
            expected_org_id: None,
            seat_annotation: None,
            observed_plan: None,
        }
    }

    fn identity(org: &str, org_type: &str) -> AccountLabel {
        AccountLabel {
            email: Some("person@example.com".into()),
            account_id: Some("account-1".into()),
            org_id: Some(org.into()),
            org_type: Some(org_type.into()),
            ..Default::default()
        }
    }

    fn auth(expires: u64) -> Option<AuthMetadata> {
        Some(AuthMetadata {
            has_credential: true,
            expires_at_ms: Some(expires),
        })
    }

    fn observed_free_plan() -> ObservedPlan {
        ObservedPlan {
            email: "PERSON@example.com".into(),
            plan: "free".into(),
            verified_at_ms: 7,
        }
    }

    #[test]
    fn free_plan_observation_does_not_claim_team_usage_or_change_connection_state() {
        let mut personal = profile("claude", "personal");
        let team = identity("org-team", "claude_team");
        let check =
            |p: &ConnectionProfile, who| status_from_metadata(p, 10, Some(who), auth(20), &[], &[]);
        let unobserved = check(&personal, team.clone());
        assert!(unobserved.observed_plan.is_none());
        personal.observed_plan = Some(observed_free_plan());
        let observed = check(&personal, team.clone());
        assert_eq!(observed.state, "wrong_scope");
        assert!(!observed.has_row);
        assert_eq!(observed.observed_plan, personal.observed_plan);
        let mut wire = serde_json::to_value(observed).unwrap();
        wire.as_object_mut().unwrap().remove("observed_plan");
        assert_eq!(wire, serde_json::to_value(unobserved).unwrap());
        let wrong = AccountLabel {
            email: Some("other@example.com".into()),
            ..team
        };
        assert!(check(&personal, wrong).observed_plan.is_none());
    }

    #[test]
    fn observed_plan_requires_matching_personal_claude_email_free_and_valid_date() {
        for (provider, scope, email, plan, at) in [
            ("codex", "personal", "person@example.com", "free", 7),
            ("claude", "team", "person@example.com", "free", 7),
            ("claude", "personal", "other@example.com", "free", 7),
            ("claude", "personal", "person@example.com", "pro", 7),
            ("claude", "personal", "person@example.com", "free", 0),
            ("claude", "personal", "person@example.com", "free", 11),
        ] {
            let mut profile = profile(provider, scope);
            profile.observed_plan = Some(ObservedPlan {
                email: email.into(),
                plan: plan.into(),
                verified_at_ms: at,
            });
            assert!(validated_observed_plan(&profile, 10).is_none());
        }
    }

    fn workspace(email: Option<&str>, plan: &str, id: &str) -> WorkspaceUsage {
        WorkspaceUsage {
            account_id: id.into(),
            email: email.map(str::to_string),
            name: None,
            plan_type: Some(plan.into()),
            windows: vec![],
            credits_balance: None,
            reset_credits: None,
            active: false,
        }
    }

    fn annotated_profile() -> ConnectionProfile {
        let mut profile = profile("codex", "team");
        profile.seat_annotation = Some(SeatAnnotation {
            email: "PERSON@example.com".into(),
            workspace_id: "team-1".into(),
            workspace_name: "Example workspace".into(),
            seat_type: "standard".into(),
            verified_at_ms: 7,
        });
        profile
    }

    #[test]
    fn manual_seat_annotation_does_not_create_login_or_quota_success() {
        let profile = annotated_profile();
        let plain = ConnectionProfile {
            seat_annotation: None,
            ..profile.clone()
        };
        for (identity, expires) in [
            (None, None),
            (Some(identity("unused", "unused")), Some(10)),
            (Some(identity("unused", "unused")), Some(20)),
        ] {
            let expected = status_from_metadata(
                &plain,
                10,
                identity.clone(),
                expires.and_then(auth),
                &[],
                &[],
            );
            let status =
                status_from_metadata(&profile, 10, identity, expires.and_then(auth), &[], &[]);
            assert!(!status.has_row);
            assert_eq!(status.seat_annotation, profile.seat_annotation);
            let mut serialized = serde_json::to_value(&status).unwrap();
            serialized
                .as_object_mut()
                .unwrap()
                .remove("seat_annotation");
            assert_eq!(serialized, serde_json::to_value(expected).unwrap());
            assert!(!serialized.to_string().contains("PRIVATE-PATH"));
        }
    }

    #[test]
    fn manual_seat_annotation_binds_only_to_the_exact_visible_workspace() {
        let profile = annotated_profile();
        let check = |id| {
            status_from_metadata(
                &profile,
                10,
                Some(identity("unused", "unused")),
                auth(20),
                &[],
                &[workspace(Some("person@example.com"), "business", id)],
            )
        };
        let matching = check("team-1");
        assert!(matching.has_row);
        assert_eq!(matching.seat_annotation, profile.seat_annotation);
        let different = check("team-2");
        assert!(different.has_row);
        assert_eq!(different.account_id.as_deref(), Some("team-2"));
        assert!(different.seat_annotation.is_none());
    }

    #[test]
    fn manual_seat_annotation_selects_matching_workspace_after_another_team() {
        let profile = annotated_profile();
        let rows = vec![
            workspace(Some("person@example.com"), "team", "team-2"),
            workspace(Some("person@example.com"), "business", "team-1"),
        ];
        let original_rows = rows.clone();
        let check = |profile: &ConnectionProfile| {
            status_from_metadata(
                profile,
                10,
                Some(identity("unused", "unused")),
                auth(20),
                &[],
                &rows,
            )
        };
        let plain = ConnectionProfile {
            seat_annotation: None,
            ..profile.clone()
        };
        let first = check(&plain);
        assert_eq!(first.account_id.as_deref(), Some("team-2"));
        let annotated = check(&profile);
        assert_eq!(annotated.state, first.state);
        assert_eq!(annotated.has_row, first.has_row);
        assert_eq!(annotated.account_id.as_deref(), Some("team-1"));
        assert_eq!(annotated.seat_annotation, profile.seat_annotation);
        assert_eq!(rows, original_rows);

        let mut invalid = profile.clone();
        invalid.seat_annotation.as_mut().unwrap().verified_at_ms = 11;
        let unverified = check(&invalid);
        assert_eq!(unverified.account_id.as_deref(), Some("team-2"));
        assert!(unverified.seat_annotation.is_none());
    }

    #[test]
    fn manual_seat_annotation_does_not_select_wrong_email_or_scope_workspace() {
        let profile = annotated_profile();
        let rows = vec![
            workspace(Some("person@example.com"), "team", "team-2"),
            workspace(Some("other@example.com"), "business", "team-1"),
            workspace(Some("person@example.com"), "pro", "team-1"),
        ];
        let check = |rows: &[WorkspaceUsage]| {
            status_from_metadata(
                &profile,
                10,
                Some(identity("unused", "unused")),
                auth(20),
                &[],
                rows,
            )
        };
        let fallback = check(&rows);
        assert!(fallback.has_row);
        assert_eq!(fallback.account_id.as_deref(), Some("team-2"));
        assert!(fallback.seat_annotation.is_none());

        // Neither the wrong user's team nor this user's personal row is an
        // eligible team workspace, even when its ID matches the manual note.
        let absent = check(&rows[1..]);
        assert_eq!(absent.state, "ready");
        assert!(!absent.has_row);
        assert_eq!(absent.account_id.as_deref(), Some("account-1"));
        assert_eq!(absent.seat_annotation, profile.seat_annotation);
    }

    #[test]
    fn manual_seat_annotation_hides_wrong_accounts_and_unsupported_scopes() {
        let profile = annotated_profile();
        let wrong_identity = AccountLabel {
            email: Some("other@example.com".into()),
            ..identity("unused", "unused")
        };
        for auth in [None, auth(20)] {
            let status =
                status_from_metadata(&profile, 10, Some(wrong_identity.clone()), auth, &[], &[]);
            assert!(status.seat_annotation.is_none());
        }
        for (provider, scope) in [
            ("claude", "team"),
            ("codex", "personal"),
            ("codex", "unsupported"),
            ("unsupported", "team"),
        ] {
            let other = ConnectionProfile {
                provider: provider.into(),
                scope: scope.into(),
                ..profile.clone()
            };
            let status = status_from_metadata(&other, 10, None, None, &[], &[]);
            assert!(status.seat_annotation.is_none());
        }
    }

    #[test]
    fn manual_seat_annotation_requires_bounded_metadata_and_a_past_verification() {
        let invalid: &[fn(&mut SeatAnnotation)] = &[
            |a| a.email = "other@example.com".into(),
            |a| a.email = "person@example.com\n".into(),
            |a| a.email = "x".repeat(321),
            |a| a.workspace_id = " ".into(),
            |a| a.workspace_id = "x".repeat(129),
            |a| a.workspace_id = "team-1\n".into(),
            |a| a.workspace_name = String::new(),
            |a| a.workspace_name = "x".repeat(161),
            |a| a.workspace_name = "Example\tworkspace".into(),
            |a| a.seat_type = "unknown".into(),
            |a| a.seat_type = "Standard".into(),
            |a| a.verified_at_ms = 0,
            |a| a.verified_at_ms = 11,
        ];
        for change in invalid {
            let mut profile = annotated_profile();
            change(profile.seat_annotation.as_mut().unwrap());
            let status = status_from_metadata(&profile, 10, None, None, &[], &[]);
            assert!(status.seat_annotation.is_none());
            assert_eq!(status.state, "login_required");
        }
        let mut profile = annotated_profile();
        let annotation = profile.seat_annotation.as_mut().unwrap();
        annotation.workspace_id = "x".repeat(128);
        annotation.workspace_name = "x".repeat(160);
        annotation.seat_type = "premium".into();
        annotation.verified_at_ms = 10;
        assert!(
            status_from_metadata(&profile, 10, None, None, &[], &[])
                .seat_annotation
                .is_some()
        );
    }

    #[test]
    fn pending_profiles_do_not_fabricate_an_account_or_expose_paths() {
        let out = status_from_metadata(&profile("claude", "personal"), 10, None, None, &[], &[]);
        assert_eq!(out.state, "login_required");
        assert!(!out.has_row);
        assert!(out.account_id.is_none());
        assert!(out.org_id.is_none());
        assert!(
            !serde_json::to_string(&out)
                .unwrap()
                .contains("PRIVATE-PATH")
        );
    }

    #[test]
    fn claude_requires_exact_email_known_scope_and_pinned_organization() {
        let mut p = profile("claude", "team");
        let a = identity("org-team", "claude_team");
        let check = |p: &ConnectionProfile, a: AccountLabel| {
            status_from_metadata(p, 10, Some(a), auth(20), &[], &[])
        };
        assert_eq!(check(&p, a.clone()).state, "ready");
        p.expected_org_id = Some("other-team".into());
        assert_eq!(check(&p, a.clone()).state, "wrong_scope");
        p.expected_org_id = None;
        assert_eq!(
            check(&p, identity("org-personal", "claude_max")).state,
            "wrong_scope"
        );
        assert_eq!(
            check(&p, identity("org-team", "future_unknown")).state,
            "wrong_scope"
        );
        let wrong = AccountLabel {
            email: Some("other@example.com".into()),
            ..a
        };
        assert_eq!(check(&p, wrong).state, "wrong_account");
    }

    #[test]
    fn an_expired_profile_never_claims_a_visible_workspace() {
        let rows = vec![workspace(Some("person@example.com"), "pro", "personal-1")];
        let out = status_from_metadata(
            &profile("codex", "personal"),
            20,
            Some(identity("org-irrelevant", "claude_max")),
            auth(20),
            &[],
            &rows,
        );
        assert_eq!(out.state, "credential_expired");
        assert!(!out.has_row);
    }

    fn claude_row(account: AccountLabel) -> ProviderQuota {
        ProviderQuota {
            provider: "claude".into(),
            plan_name: None,
            reset_credits: None,
            h5: None,
            d7: None,
            from_label: String::new(),
            session: String::new(),
            account: Some(account),
            origin: None,
            cache_fetched_at_ms: None,
            scoped: vec![],
            extra: None,
        }
    }

    #[test]
    fn observed_free_plan_yields_to_a_matching_live_personal_row() {
        let mut p = profile("claude", "personal");
        p.observed_plan = Some(observed_free_plan());
        let current = identity("org-personal", "claude_max");
        let quota = claude_row(current.clone());
        let status = status_from_metadata(&p, 10, Some(current), auth(20), &[quota], &[]);
        assert!(status.has_row);
        assert!(status.observed_plan.is_none());
    }

    #[test]
    fn claude_row_requires_the_same_account_email_and_organization() {
        let p = profile("claude", "team");
        let a = identity("org-team", "claude_team");
        let check = |row| status_from_metadata(&p, 10, Some(a.clone()), auth(20), &[row], &[]);
        assert!(check(claude_row(a.clone())).has_row);
        let other_org = AccountLabel {
            org_id: Some("org-personal".into()),
            ..a.clone()
        };
        assert!(!check(claude_row(other_org)).has_row);
        let other_account = AccountLabel {
            account_id: Some("account-other".into()),
            ..a.clone()
        };
        assert!(!check(claude_row(other_account)).has_row);
        let other_email = AccountLabel {
            email: Some("other@example.com".into()),
            ..a.clone()
        };
        assert!(!check(claude_row(other_email)).has_row);
    }

    #[test]
    fn one_codex_login_can_have_two_scopes_but_rows_require_source_email() {
        for plan in ["prolite", "pro", "promax"] {
            assert_eq!(codex_scope(Some(plan)), Some("personal"));
        }
        let a = identity("unused", "unused");
        let rows = vec![
            workspace(Some("person@example.com"), "pro", "personal-1"),
            workspace(Some("other@example.com"), "team", "wrong-team"),
            workspace(None, "team", "legacy-team"),
        ];
        let p = profile("codex", "personal");
        let personal = status_from_metadata(&p, 10, Some(a.clone()), auth(20), &[], &rows);
        assert!(personal.has_row);
        assert_eq!(personal.account_id.as_deref(), Some("personal-1"));
        let p = profile("codex", "team");
        let team = status_from_metadata(&p, 10, Some(a.clone()), auth(20), &[], &rows);
        assert_eq!(team.state, "ready");
        assert!(!team.has_row);
        let rows = vec![workspace(Some("person@example.com"), "business", "team-1")];
        let team = status_from_metadata(&p, 10, Some(a), auth(20), &[], &rows);
        assert!(team.has_row);
        assert_eq!(team.account_id.as_deref(), Some("team-1"));
    }
}
