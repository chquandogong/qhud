// Interactive sign-in is routed by a registered profile key. The native
// backend resolves its directory and verifies identity after login.
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory(require("./account-identities.js"));
  else if (root && root.document) root.QhudProfileLogin = factory(root.QhudAccountIdentities);
})(typeof window === "undefined" ? null : window, function (identities) {
  "use strict";

  const { email } = identities;
  function sameProfile(a, b) {
    return !!a && !!b && a.provider === b.provider && a.key === b.key &&
      a.scope === b.scope && !!email(a.email) && email(a.email) === email(b.email);
  }

  function eligible(p) {
    return p?.can_reauthenticate === true && ["claude", "codex"].includes(p.provider) &&
      ["personal", "team"].includes(p.scope) && ["credential_expired", "login_required", "wrong_account", "wrong_scope"].includes(p.state) &&
      typeof p.key === "string" && /^[A-Za-z0-9][A-Za-z0-9_-]{0,100}$/.test(p.key) && !!email(p.email) &&
      !(p.provider === "claude" && p.scope === "personal" && p.observed_plan?.plan === "free" &&
        email(p.email) === email(p.observed_plan.email));
  }

  function hasFreshResult(profile, latest, rows) {
    if (!profile?.account_id || !Array.isArray(rows)) return false;
    if (["wrong_account", "wrong_scope"].includes(latest.state)) return false;
    const percent = (value) => Number.isFinite(value) && value >= 0 && value <= 100;
    if (profile.provider === "claude") {
      if (latest.state === "ready" && (latest.account_id !== profile.account_id || latest.org_id !== profile.org_id)) return false;
      return !!profile.org_id && rows.some((r) => r?.account_id === profile.account_id && r.org_id === profile.org_id &&
        (percent(r.usage?.five_hour?.pct) || percent(r.usage?.seven_day?.pct) ||
          r.usage?.scoped?.some((w) => percent(w.pct))));
    }
    // A verified team seat can refer to another workspace under the same login.
    const workspace = identities.seatAnnotationForProfile(profile)?.workspace_id || profile.account_id;
    if (latest.has_row && latest.account_id !== workspace) return false;
    return profile.provider === "codex" && rows.some((r) => r?.account_id === workspace &&
      email(r.email) === email(profile.email) && (r.windows?.some((w) => percent(w.used_percent)) ||
        (Number.isInteger(r.reset_credits) && r.reset_credits >= 0)) &&
      identities.accountScope("codex", r.plan_type) === profile.scope);
  }

  const errors = new Set(["auth_busy", "auth_unavailable", "auth_profile_changed", "auth_login_failed",
    "auth_wrong_account", "auth_wrong_scope", "auth_not_ready"]);
  function controller({ lookup, authenticate, refresh, changed, now = Date.now }) {
    let attempt = null;
    function pending() { return attempt?.stage === "auth_waiting" || attempt?.stage === "auth_refreshing"; }
    function messageFor(profile) {
      if (!sameProfile(profile, attempt?.profile) || (!pending() && now() >= attempt.until)) return null;
      return attempt.stage;
    }
    async function start(profile) {
      const current = lookup(profile.key);
      if (pending() || !sameProfile(current, profile) || !eligible(current)) return;
      attempt = { profile: current, stage: "auth_waiting", until: 0 };
      changed();
      try {
        // The native reply reflects the login that just finished. The report
        // on screen may still have no account or organization ID until the next poll.
        const verified = await authenticate(current.key);
        if (!sameProfile(current, verified) || !sameProfile(current, lookup(current.key))) throw "auth_profile_changed";
        attempt.stage = "auth_refreshing";
        changed();
        const rows = await refresh(current.provider);
        const latest = lookup(current.key);
        if (!sameProfile(current, latest)) throw "auth_profile_changed";
        // The latest report may still describe the wrong login we just repaired.
        // In that case use the native result; a different mismatch is new evidence.
        const oldMismatch = ["wrong_account", "wrong_scope"].includes(current.state) && latest.state === current.state &&
          latest.account_id === current.account_id && latest.org_id === current.org_id;
        attempt.stage = hasFreshResult(verified, oldMismatch ? verified : latest, rows) ? "auth_updated" : "auth_usage_unavailable";
      } catch (e) {
        const code = String(e);
        attempt.stage = errors.has(code) ? code : attempt.stage === "auth_refreshing" ? "auth_usage_unavailable" : "auth_login_failed";
      } finally {
        attempt.until = now() + 30_000;
        changed();
      }
    }
    return { start, pending, messageFor };
  }

  return { eligible, sameProfile, controller };
});
